//! Efficient queue-like structure with O(1) append and O(1) amortized uncons.
//!
//! Implements the ["Reflection without Remorse"](http://okmij.org/ftp/Haskell/zseq.pdf) data structure used to enable O(1) left-associated [`bind`](crate::functions::bind) operations in the [`Free`](crate::types::Free) monad. In that context, `CatList` serves as the continuation queue: each `bind` appends a continuation in O(1), and [`Free::evaluate`](crate::types::Free::evaluate) pops continuations one at a time via `uncons`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::cat_list::CatList;
//!
//! let list = CatList::singleton(1).snoc(2).snoc(3).append(CatList::singleton(4));
//!
//! let mut result = Vec::new();
//! let mut current = list;
//! while let Some((head, tail)) = current.uncons() {
//! 	result.push(head);
//! 	current = tail;
//! }
//! assert_eq!(result, vec![1, 2, 3, 4]);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				CatListBrand,
				OptionBrand,
			},
			classes::*,
			dispatch::Ref,
			impl_kind,
			kinds::*,
		},
		core::ops::ControlFlow,
		fp_macros::*,
		std::{
			cmp::Ordering,
			collections::VecDeque,
			fmt,
			hash::{
				Hash,
				Hasher,
			},
		},
	};

	/// Internal representation of a `CatList`.
	///
	/// This enum is private; the public type is the newtype wrapper [`CatList`].
	/// Keeping `CatListInner` free of a custom `Drop` impl allows `uncons` to
	/// destructure it by move, eliminating the need for unsafe code.
	#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
	enum CatListInner<A> {
		/// Empty list
		#[default]
		Nil,
		/// Head element plus deque of sublists and total length
		Cons(A, VecDeque<CatList<A>>, usize),
	}

	/// A catenable list with O(1) append and O(1) amortized uncons.
	///
	/// This is the "Reflection without Remorse" data structure that enables
	/// O(1) left-associated bind operations in the Free monad.
	///
	/// ### Higher-Kinded Type Representation
	///
	/// The higher-kinded representation of this type constructor is [`CatListBrand`](crate::brands::CatListBrand),
	/// which is fully polymorphic over the element type.
	///
	/// ### Serialization
	///
	/// This type supports serialization and deserialization via [`serde`](https://serde.rs) when the `serde` feature is enabled.
	///
	/// ### Performance Notes
	///
	/// This implementation uses a [`VecDeque`] to store sublists, providing:
	///
	/// * **O(1) append**: Sublists are pushed to the back of the deque.
	/// * **O(1) amortized uncons**: Extracting the head is O(1) when the sublist deque
	///   is empty. When non-empty, `uncons` calls `flatten_deque`, which performs a
	///   right fold over the deque entries. Each entry is visited exactly once across
	///   the full sequence of `uncons` calls, so the cost is O(1) amortized per element.
	/// * **Low overhead deque operations**: The underlying `VecDeque` provides O(1)
	///   push and pop on both ends. Unlike a two-stack queue, it does not require
	///   periodic bulk reversal, though it may occasionally reallocate its backing
	///   buffer when capacity is exceeded.
	#[document_type_parameters("The type of the elements in the list.")]
	///
	#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Debug, Clone, Eq)]
	pub struct CatList<A>(CatListInner<A>);

	#[document_type_parameters("The type of the elements in the list.")]
	impl<A> Default for CatList<A> {
		#[document_signature]
		#[document_returns("An empty `CatList`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list: CatList<i32> = Default::default();
		/// assert!(list.is_empty());
		/// ```
		fn default() -> Self {
			CatList::empty()
		}
	}

	#[document_type_parameters("The type of the elements in the list.")]
	#[document_parameters("The list to compare.")]
	impl<A: PartialEq> PartialEq for CatList<A> {
		#[document_signature]
		#[document_parameters("The other list to compare to.")]
		#[document_returns("True if the values are equal, false otherwise.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		/// let list1: CatList<i32> = CatList::singleton(1);
		/// let list2: CatList<i32> = CatList::singleton(1);
		/// assert_eq!(list1, list2);
		/// ```
		fn eq(
			&self,
			other: &Self,
		) -> bool {
			if self.len() != other.len() {
				return false;
			}
			self.iter().eq(other.iter())
		}
	}

	#[document_type_parameters("The type of the elements in the list.")]
	#[document_parameters("The list to hash.")]
	impl<A: Hash> Hash for CatList<A> {
		#[document_signature]
		#[document_type_parameters("The type of the hasher.")]
		#[document_parameters("The hasher state to update.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::types::cat_list::CatList,
		/// 	std::{
		/// 		collections::hash_map::DefaultHasher,
		/// 		hash::{
		/// 			Hash,
		/// 			Hasher,
		/// 		},
		/// 	},
		/// };
		///
		/// let list = CatList::singleton(1);
		/// let mut hasher = DefaultHasher::new();
		/// list.hash(&mut hasher);
		/// assert!(hasher.finish() != 0);
		/// ```
		fn hash<H: Hasher>(
			&self,
			state: &mut H,
		) {
			self.len().hash(state);
			for a in self.iter() {
				a.hash(state);
			}
		}
	}

	#[document_type_parameters("The type of the elements in the list.")]
	#[document_parameters("The list to compare.")]
	impl<A: PartialOrd> PartialOrd for CatList<A> {
		#[document_signature]
		#[document_parameters("The other list to compare to.")]
		#[document_returns("An ordering if the values can be compared, none otherwise.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list1 = CatList::singleton(1);
		/// let list2 = CatList::singleton(2);
		/// assert!(list1 < list2);
		/// ```
		fn partial_cmp(
			&self,
			other: &Self,
		) -> Option<Ordering> {
			self.iter().partial_cmp(other.iter())
		}
	}

	#[document_type_parameters("The type of the elements in the list.")]
	#[document_parameters("The list to compare.")]
	impl<A: Ord> Ord for CatList<A> {
		#[document_signature]
		#[document_parameters("The other list to compare to.")]
		#[document_returns("The ordering of the values.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::types::cat_list::CatList,
		/// 	std::cmp::Ordering,
		/// };
		///
		/// let list1 = CatList::singleton(1);
		/// let list2 = CatList::singleton(2);
		/// assert_eq!(list1.cmp(&list2), Ordering::Less);
		/// ```
		fn cmp(
			&self,
			other: &Self,
		) -> Ordering {
			self.iter().cmp(other.iter())
		}
	}

	impl_kind! {
		for CatListBrand {
			type Of<'a, A: 'a>: 'a = CatList<A>;
		}
	}

	impl CatListBrand {
		/// Constructs a new list by prepending a value to an existing list.
		///
		/// This method creates a new list with the given head element followed by the elements of the tail list.
		#[document_signature]
		///
		#[document_type_parameters("The type of the elements in the list.")]
		///
		#[document_parameters("A value to prepend to the list.", "A list to prepend the value to.")]
		///
		#[document_returns(
			"A new list consisting of the `head` element prepended to the `tail` list."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let head = 1;
		/// let tail = CatList::singleton(2).snoc(3);
		/// let new_list = CatListBrand::construct(head, tail);
		/// let vec: Vec<_> = new_list.into_iter().collect();
		/// assert_eq!(vec, vec![1, 2, 3]);
		/// ```
		pub fn construct<A>(
			head: A,
			tail: CatList<A>,
		) -> CatList<A> {
			tail.cons(head)
		}

		/// Deconstructs a list into its head element and tail list.
		///
		/// This method splits a list into its first element and the rest of the elements as a new list.
		#[document_signature]
		///
		#[document_type_parameters("The type of the elements in the list.")]
		///
		#[document_parameters("The list to deconstruct.")]
		///
		#[document_returns(
			"An [`Option`] containing a tuple of the head element and the remaining tail list, or [`None`] if the list is empty."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2);
		/// let deconstructed = CatListBrand::deconstruct(&list);
		/// let (head, tail) = deconstructed.unwrap();
		/// assert_eq!(head, 1);
		/// let tail_vec: Vec<_> = tail.into_iter().collect();
		/// assert_eq!(tail_vec, vec![2]);
		/// ```
		pub fn deconstruct<A>(list: &CatList<A>) -> Option<(A, CatList<A>)>
		where
			A: Clone, {
			list.clone().uncons()
		}
	}

	impl Functor for CatListBrand {
		/// Maps a function over the list.
		///
		/// This method applies a function to each element of the list, producing a new list with the transformed values.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the list.",
			"The type of the elements in the resulting list."
		)]
		///
		#[document_parameters("The function to apply to each element.", "The list to map over.")]
		///
		#[document_returns("A new list containing the results of applying the function.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let mapped = explicit::map::<CatListBrand, _, _, _, _>(|x: i32| x * 2, list);
		/// let vec: Vec<_> = mapped.into_iter().collect();
		/// assert_eq!(vec, vec![2, 4, 6]);
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map(func)
		}
	}

	impl Lift for CatListBrand {
		/// Lifts a binary function into the list context (Cartesian product).
		///
		/// This method applies a binary function to all pairs of elements from two lists, producing a new list containing the results (Cartesian product).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the first list.",
			"The type of the elements in the second list.",
			"The type of the elements in the resulting list."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first list.",
			"The second list."
		)]
		///
		#[document_returns(
			"A new list containing the results of applying the function to all pairs of elements."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list1 = CatList::singleton(1).snoc(2);
		/// let list2 = CatList::singleton(10).snoc(20);
		/// let lifted = explicit::lift2::<CatListBrand, _, _, _, _, _, _>(|x, y| x + y, list1, list2);
		/// let vec: Vec<_> = lifted.into_iter().collect();
		/// assert_eq!(vec, vec![11, 21, 12, 22]);
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: Clone + 'a,
			B: Clone + 'a,
			C: 'a, {
			fa.into_iter()
				.flat_map(|a| {
					let f = &func;
					fb.clone().into_iter().map(move |b| f(a.clone(), b))
				})
				.collect()
		}
	}

	impl Pointed for CatListBrand {
		/// Wraps a value in a list.
		///
		/// This method creates a new list containing the single given value.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A list containing the single value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = pure::<CatListBrand, _>(5);
		/// let vec: Vec<_> = list.into_iter().collect();
		/// assert_eq!(vec, vec![5]);
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			CatList::pure(a)
		}
	}

	impl ApplyFirst for CatListBrand {}
	impl ApplySecond for CatListBrand {}

	impl Semiapplicative for CatListBrand {
		/// Applies wrapped functions to wrapped values (Cartesian product).
		///
		/// This method applies each function in the first list to each value in the second list, producing a new list containing all the results.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the input values.",
			"The type of the output values."
		)]
		///
		#[document_parameters(
			"The list containing the functions.",
			"The list containing the values."
		)]
		///
		#[document_returns(
			"A new list containing the results of applying each function to each value."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let funcs = CatList::singleton(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1))
		/// 	.snoc(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let vals = CatList::singleton(1).snoc(2);
		/// let applied = apply::<RcFnBrand, CatListBrand, _, _>(funcs, vals);
		/// let vec: Vec<_> = applied.into_iter().collect();
		/// assert_eq!(vec, vec![2, 3, 2, 4]);
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ff.into_iter().flat_map(|f| fa.clone().into_iter().map(move |a| f(a.clone()))).collect()
		}
	}

	impl Alt for CatListBrand {
		/// Concatenates two lists.
		///
		/// This is the same as [`Semigroup::append`] for `CatList`, providing an
		/// associative choice operation for the `CatList` type constructor.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters("The first list.", "The second list.")]
		///
		#[document_returns("The concatenated list.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = CatList::singleton(1).snoc(2);
		/// let y = CatList::singleton(3).snoc(4);
		/// let result: Vec<_> = explicit::alt::<CatListBrand, _, _, _>(x, y).into_iter().collect();
		/// assert_eq!(result, vec![1, 2, 3, 4]);
		/// ```
		fn alt<'a, A: 'a>(
			fa1: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fa2: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa1.append(fa2)
		}
	}

	impl RefAlt for CatListBrand {
		/// Concatenates two lists by reference.
		///
		/// Both input lists are borrowed and cloned before appending. Because
		/// `CatList` is backed by `Rc`-based structural sharing, cloning is O(1),
		/// and the subsequent `append` is also O(1). The `A: Clone` bound is
		/// required by the trait signature but is not exercised in this
		/// implementation.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters("The first list.", "The second list.")]
		///
		#[document_returns("The concatenated list.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = CatList::singleton(1).snoc(2);
		/// let y = CatList::singleton(3).snoc(4);
		/// let result: Vec<_> = explicit::alt::<CatListBrand, _, _, _>(&x, &y).into_iter().collect();
		/// assert_eq!(result, vec![1, 2, 3, 4]);
		/// ```
		fn ref_alt<'a, A: 'a + Clone>(
			fa1: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fa2: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa1.clone().append(fa2.clone())
		}
	}

	impl Plus for CatListBrand {
		/// Returns an empty list, the identity element for [`alt`](Alt::alt).
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_returns("An empty list.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x: CatList<i32> = plus_empty::<CatListBrand, i32>();
		/// assert!(x.is_empty());
		/// ```
		fn empty<'a, A: 'a>() -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			CatList::empty()
		}
	}

	impl Semimonad for CatListBrand {
		/// Chains list computations (`flat_map`).
		///
		/// This method applies a function that returns a list to each element of the input list, and then flattens the result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the input list.",
			"The type of the elements in the output list."
		)]
		///
		#[document_parameters(
			"The first list.",
			"The function to apply to each element, returning a list."
		)]
		///
		#[document_returns("A new list containing the flattened results.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2);
		/// let bound =
		/// 	explicit::bind::<CatListBrand, _, _, _, _>(list, |x| CatList::singleton(x).snoc(x * 2));
		/// let vec: Vec<_> = bound.into_iter().collect();
		/// assert_eq!(vec, vec![1, 2, 2, 4]);
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ma.bind(func)
		}
	}

	impl Foldable for CatListBrand {
		/// Folds the list from the right.
		///
		/// This method performs a right-associative fold of the list.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the list.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The list to fold.")]
		///
		#[document_returns("The final accumulator value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// assert_eq!(
		/// 	explicit::fold_right::<RcFnBrand, CatListBrand, _, _, _, _>(|x: i32, acc| x + acc, 0, list),
		/// 	6
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			fa.fold_right(func, initial)
		}

		/// Folds the list from the left.
		///
		/// This method performs a left-associative fold of the list.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the list.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The list to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// assert_eq!(
		/// 	explicit::fold_left::<RcFnBrand, CatListBrand, _, _, _, _>(|acc, x: i32| acc + x, 0, list),
		/// 	6
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			fa.fold_left(func, initial)
		}

		/// Maps the values to a monoid and combines them.
		///
		/// This method maps each element of the list to a monoid and then combines the results using the monoid's `append` operation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the list.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The list to fold.")]
		///
		#[document_returns("The combined monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// assert_eq!(
		/// 	explicit::fold_map::<RcFnBrand, CatListBrand, _, _, _, _>(|x: i32| x.to_string(), list),
		/// 	"123".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneFn + 'a, {
			fa.fold_map(func)
		}
	}

	impl Traversable for CatListBrand {
		/// Traverses the list with an applicative function.
		///
		/// This method maps each element of the list to a computation, evaluates them, and combines the results into an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning a value in an applicative context.",
			"The list to traverse."
		)]
		///
		#[document_returns("The list wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let traversed = explicit::traverse::<RcFnBrand, CatListBrand, _, _, OptionBrand, _, _>(
		/// 	|x| Some(x * 2),
		/// 	list,
		/// );
		/// let vec: Vec<_> = traversed.unwrap().into_iter().collect();
		/// assert_eq!(vec, vec![2, 4, 6]);
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			ta.into_iter().fold(F::pure(CatList::empty()), |acc, x| {
				F::lift2(|list, b| list.snoc(b), acc, func(x))
			})
		}

		/// Sequences a list of applicative.
		///
		/// This method evaluates the computations inside the list and accumulates the results into an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The list containing the applicative values.")]
		///
		#[document_returns("The list wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(Some(1)).snoc(Some(2));
		/// let sequenced = sequence::<CatListBrand, _, OptionBrand>(list);
		/// let vec: Vec<_> = sequenced.unwrap().into_iter().collect();
		/// assert_eq!(vec, vec![1, 2]);
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			ta.into_iter()
				.fold(F::pure(CatList::empty()), |acc, x| F::lift2(|list, a| list.snoc(a), acc, x))
		}
	}

	impl WithIndex for CatListBrand {
		type Index = usize;
	}

	impl FunctorWithIndex for CatListBrand {
		/// Maps a function over the list, providing the index of each element.
		///
		/// This is the trait form of [`CatList::map_with_index`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the list.",
			"The type of the elements in the resulting list."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and its index.",
			"The list to map over."
		)]
		///
		#[document_returns("A new list containing the mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::functor_with_index::FunctorWithIndex,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(10).snoc(20).snoc(30);
		/// let result = CatListBrand::map_with_index(|i, x: i32| x + i as i32, list);
		/// let vec: Vec<_> = result.into_iter().collect();
		/// assert_eq!(vec, vec![10, 21, 32]);
		/// ```
		fn map_with_index<'a, A: 'a, B: 'a>(
			f: impl Fn(usize, A) -> B + 'a,
			fa: CatList<A>,
		) -> CatList<B> {
			fa.map_with_index(f)
		}
	}

	impl FoldableWithIndex for CatListBrand {
		/// Folds the list using a monoid, providing the index of each element.
		///
		/// This is the trait form of [`CatList::fold_map_with_index`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the list.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and its index.",
			"The list to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::foldable_with_index::FoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(10).snoc(20).snoc(30);
		/// let result =
		/// 	CatListBrand::fold_map_with_index::<RcFnBrand, _, _>(|i, x: i32| format!("{i}:{x}"), list);
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		fn fold_map_with_index<'a, FnBrand, A: 'a + Clone, R: Monoid + 'a>(
			f: impl Fn(usize, A) -> R + 'a,
			fa: CatList<A>,
		) -> R
		where
			FnBrand: LiftFn + 'a, {
			fa.fold_map_with_index(f)
		}
	}

	impl TraversableWithIndex for CatListBrand {
		/// Traverses the list with an applicative function, providing the index of each element.
		///
		/// This is the trait form of [`CatList::traverse_with_index`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the list.",
			"The type of the elements in the resulting list.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and its index, returning a value in an applicative context.",
			"The list to traverse."
		)]
		///
		#[document_returns("The list wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::traversable_with_index::TraversableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let result = CatListBrand::traverse_with_index::<i32, i32, OptionBrand>(
		/// 	|_i, x| if x > 0 { Some(x * 2) } else { None },
		/// 	list,
		/// );
		/// let vec: Vec<_> = result.unwrap().into_iter().collect();
		/// assert_eq!(vec, vec![2, 4, 6]);
		/// ```
		fn traverse_with_index<'a, A: 'a, B: 'a + Clone, M: Applicative>(
			f: impl Fn(usize, A) -> M::Of<'a, B> + 'a,
			ta: CatList<A>,
		) -> M::Of<'a, CatList<B>>
		where
			CatList<B>: Clone,
			M::Of<'a, B>: Clone, {
			ta.into_iter().enumerate().fold(M::pure(CatList::empty()), |acc, (i, x)| {
				M::lift2(|list, b| list.snoc(b), acc, f(i, x))
			})
		}
	}

	impl ParFunctor for CatListBrand {
		/// Maps a function over the list in parallel.
		///
		/// Delegates to [`CatList::par_map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply to each element. Must be `Send + Sync`.",
			"The list to map over."
		)]
		///
		#[document_returns("A new list containing the mapped elements.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_functor::ParFunctor,
		/// 	types::CatList,
		/// };
		///
		/// let list: CatList<i32> = vec![1, 2, 3].into_iter().collect();
		/// let result: Vec<_> = CatListBrand::par_map(|x: i32| x * 2, list).into_iter().collect();
		/// assert_eq!(result, vec![2, 4, 6]);
		/// ```
		fn par_map<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.par_map(f)
		}
	}

	impl ParCompactable for CatListBrand {
		/// Compacts a list of options in parallel, discarding `None` values.
		///
		/// Delegates to [`CatList::par_compact`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		///
		#[document_parameters("The list of options.")]
		///
		#[document_returns("A new list containing the unwrapped `Some` values.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_compactable::ParCompactable,
		/// 	types::CatList,
		/// };
		///
		/// let list: CatList<Option<i32>> = vec![Some(1), None, Some(3)].into_iter().collect();
		/// let result: Vec<_> = CatListBrand::par_compact(list).into_iter().collect();
		/// assert_eq!(result, vec![1, 3]);
		/// ```
		fn par_compact<'a, A: 'a + Send>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa.par_compact()
		}

		/// Separates a list of results into `(errors, oks)` in parallel.
		///
		/// Delegates to [`CatList::par_separate`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The error type.",
			"The success type."
		)]
		///
		#[document_parameters("The list of results.")]
		///
		#[document_returns(
			"A pair `(errs, oks)` where `errs` contains the `Err` values and `oks` the `Ok` values."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_compactable::ParCompactable,
		/// 	types::CatList,
		/// };
		///
		/// let list: CatList<Result<i32, &str>> = vec![Ok(1), Err("a"), Ok(3)].into_iter().collect();
		/// let (errs, oks): (CatList<&str>, CatList<i32>) = CatListBrand::par_separate(list);
		/// assert_eq!(errs.into_iter().collect::<Vec<_>>(), vec!["a"]);
		/// assert_eq!(oks.into_iter().collect::<Vec<_>>(), vec![1, 3]);
		/// ```
		fn par_separate<'a, E: 'a + Send, O: 'a + Send>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			fa.par_separate()
		}
	}

	impl ParFilterable for CatListBrand {
		/// Maps and filters a list in parallel, discarding elements where `f` returns `None`.
		///
		/// Single-pass implementation via Vec intermediary. Delegates to
		/// [`CatList::par_filter_map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply. Must be `Send + Sync`.",
			"The list to filter and map."
		)]
		///
		#[document_returns("A new list containing the `Some` results of applying `f`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_filterable::ParFilterable,
		/// 	types::CatList,
		/// };
		///
		/// let list: CatList<i32> = vec![1, 2, 3, 4, 5].into_iter().collect();
		/// let result: Vec<_> =
		/// 	CatListBrand::par_filter_map(|x: i32| if x % 2 == 0 { Some(x * 10) } else { None }, list)
		/// 		.into_iter()
		/// 		.collect();
		/// assert_eq!(result, vec![20, 40]);
		/// ```
		fn par_filter_map<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(A) -> Option<B> + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.par_filter_map(f)
		}

		/// Filters a list in parallel, retaining only elements satisfying `f`.
		///
		/// Single-pass implementation via Vec intermediary. Delegates to
		/// [`CatList::par_filter`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		///
		#[document_parameters("The predicate. Must be `Send + Sync`.", "The list to filter.")]
		///
		#[document_returns("A new list containing only the elements satisfying `f`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_filterable::ParFilterable,
		/// 	types::CatList,
		/// };
		///
		/// let list: CatList<i32> = vec![1, 2, 3, 4, 5].into_iter().collect();
		/// let result: Vec<_> = CatListBrand::par_filter(|x: &i32| x % 2 == 0, list).into_iter().collect();
		/// assert_eq!(result, vec![2, 4]);
		/// ```
		fn par_filter<'a, A: 'a + Send>(
			f: impl Fn(&A) -> bool + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa.par_filter(f)
		}
	}

	impl ParFoldable for CatListBrand {
		/// Maps each element to a [`Monoid`] value and combines them in parallel.
		///
		/// Delegates to [`CatList::par_fold_map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The element type.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function mapping each element to a monoid value. Must be `Send + Sync`.",
			"The list to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_foldable::ParFoldable,
		/// 	types::CatList,
		/// };
		///
		/// let list: CatList<i32> = vec![1, 2, 3].into_iter().collect();
		/// let result = CatListBrand::par_fold_map(|x: i32| x.to_string(), list);
		/// assert_eq!(result, "123");
		/// ```
		fn par_fold_map<'a, A: 'a + Send, M: Monoid + Send + 'a>(
			f: impl Fn(A) -> M + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M {
			fa.par_fold_map(f)
		}
	}

	impl ParFunctorWithIndex for CatListBrand {
		/// Maps a function over the list in parallel, providing each element's index.
		///
		/// Delegates to [`CatList::par_map_with_index`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply to each index and element. Must be `Send + Sync`.",
			"The list to map over."
		)]
		///
		#[document_returns("A new list containing the mapped elements.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_functor_with_index::ParFunctorWithIndex,
		/// 	types::CatList,
		/// };
		///
		/// let list: CatList<i32> = vec![10, 20, 30].into_iter().collect();
		/// let result: Vec<_> =
		/// 	CatListBrand::par_map_with_index(|i, x: i32| x + i as i32, list).into_iter().collect();
		/// assert_eq!(result, vec![10, 21, 32]);
		/// ```
		fn par_map_with_index<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(usize, A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			usize: Send + Sync + Copy + 'a, {
			fa.par_map_with_index(f)
		}
	}

	impl ParFoldableWithIndex for CatListBrand {
		/// Maps each element and its index to a [`Monoid`] value and combines them in parallel.
		///
		/// Delegates to [`CatList::par_fold_map_with_index`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The element type.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function mapping each index and element to a monoid value. Must be `Send + Sync`.",
			"The list to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_foldable_with_index::ParFoldableWithIndex,
		/// 	types::CatList,
		/// };
		///
		/// let list: CatList<i32> = vec![10, 20, 30].into_iter().collect();
		/// let result = CatListBrand::par_fold_map_with_index(|i, x: i32| format!("{i}:{x}"), list);
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		fn par_fold_map_with_index<'a, A: 'a + Send, M: Monoid + Send + 'a>(
			f: impl Fn(usize, A) -> M + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			usize: Send + Sync + Copy + 'a, {
			fa.par_fold_map_with_index(f)
		}
	}

	impl Compactable for CatListBrand {
		/// Compacts a list of options.
		///
		/// This method flattens a list of options, discarding `None` values.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters("The list of options.")]
		///
		#[document_returns("The flattened list.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(Some(1)).snoc(None).snoc(Some(2));
		/// let compacted = explicit::compact::<CatListBrand, _, _, _>(list);
		/// let vec: Vec<_> = compacted.into_iter().collect();
		/// assert_eq!(vec, vec![1, 2]);
		/// ```
		fn compact<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa.into_iter().flatten().collect()
		}

		/// Separates a list of results.
		///
		/// This method separates a list of results into a pair of lists.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the error value.",
			"The type of the success value."
		)]
		///
		#[document_parameters("The list of results.")]
		///
		#[document_returns("A pair of lists.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(Ok(1)).snoc(Err("error")).snoc(Ok(2));
		/// let (errs, oks) = explicit::separate::<CatListBrand, _, _, _, _>(list);
		/// let oks_vec: Vec<_> = oks.into_iter().collect();
		/// let errs_vec: Vec<_> = errs.into_iter().collect();
		/// assert_eq!(oks_vec, vec![1, 2]);
		/// assert_eq!(errs_vec, vec!["error"]);
		/// ```
		fn separate<'a, E: 'a, O: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			let mut oks = CatList::empty();
			let mut errs = CatList::empty();
			for result in fa {
				match result {
					Ok(o) => oks = oks.snoc(o),
					Err(e) => errs = errs.snoc(e),
				}
			}
			(errs, oks)
		}
	}

	impl RefCompactable for CatListBrand {
		/// Compacts a borrowed list of options by reference.
		///
		/// Iterates over a borrowed [`CatList`] of [`Option`] values, discarding
		/// [`None`] values and cloning the inner values from [`Some`] variants
		/// into a new [`CatList`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the [`Option`]. Must be [`Clone`] because elements are extracted from a borrowed container."
		)]
		///
		#[document_parameters("A reference to the list of [`Option`] values.")]
		///
		#[document_returns(
			"A new list containing only the cloned values from the [`Some`] variants."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(Some(1)).snoc(None).snoc(Some(2));
		/// let compacted = explicit::compact::<CatListBrand, _, _, _>(&list);
		/// let vec: Vec<_> = compacted.into_iter().collect();
		/// assert_eq!(vec, vec![1, 2]);
		/// ```
		fn ref_compact<'a, A: 'a + Clone>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<A>>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa.iter().filter_map(|opt| opt.as_ref().cloned()).collect()
		}

		/// Separates a borrowed list of results by reference.
		///
		/// Iterates over a borrowed [`CatList`] of [`Result`] values, cloning each
		/// value and partitioning them into a pair of [`CatList`]s: one for the
		/// [`Err`] values and one for the [`Ok`] values.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the error values. Must be [`Clone`] because elements are extracted from a borrowed container.",
			"The type of the success values. Must be [`Clone`] because elements are extracted from a borrowed container."
		)]
		///
		#[document_parameters("A reference to the list of [`Result`] values.")]
		///
		#[document_returns(
			"A pair of lists: the first containing the cloned [`Err`] values, and the second containing the cloned [`Ok`] values."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(Ok(1)).snoc(Err("error")).snoc(Ok(2));
		/// let (errs, oks) = explicit::separate::<CatListBrand, _, _, _, _>(&list);
		/// let oks_vec: Vec<_> = oks.into_iter().collect();
		/// let errs_vec: Vec<_> = errs.into_iter().collect();
		/// assert_eq!(oks_vec, vec![1, 2]);
		/// assert_eq!(errs_vec, vec!["error"]);
		/// ```
		fn ref_separate<'a, E: 'a + Clone, O: 'a + Clone>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			let mut errs = Vec::new();
			let mut oks = Vec::new();
			for result in fa.iter() {
				match result {
					Ok(o) => oks.push(o.clone()),
					Err(e) => errs.push(e.clone()),
				}
			}
			(errs.into_iter().collect(), oks.into_iter().collect())
		}
	}

	impl Filterable for CatListBrand {
		/// Partitions a list based on a function that returns a result.
		///
		/// This method partitions a list based on a function that returns a result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input value.",
			"The type of the error value.",
			"The type of the success value."
		)]
		///
		#[document_parameters("The function to apply.", "The list to partition.")]
		///
		#[document_returns("A pair of lists.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
		/// let (errs, oks) = explicit::partition_map::<CatListBrand, _, _, _, _, _>(
		/// 	|a| if a % 2 == 0 { Ok(a) } else { Err(a) },
		/// 	list,
		/// );
		/// let oks_vec: Vec<_> = oks.into_iter().collect();
		/// let errs_vec: Vec<_> = errs.into_iter().collect();
		/// assert_eq!(oks_vec, vec![2, 4]);
		/// assert_eq!(errs_vec, vec![1, 3]);
		/// ```
		fn partition_map<'a, A: 'a, E: 'a, O: 'a>(
			func: impl Fn(A) -> Result<O, E> + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			let mut oks = CatList::empty();
			let mut errs = CatList::empty();
			for a in fa {
				match func(a) {
					Ok(o) => oks = oks.snoc(o),
					Err(e) => errs = errs.snoc(e),
				}
			}
			(errs, oks)
		}

		/// Partitions a list based on a predicate.
		///
		/// This method partitions a list based on a predicate.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters("The predicate.", "The list to partition.")]
		///
		#[document_returns("A pair of lists.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
		/// let (not_satisfied, satisfied) =
		/// 	explicit::partition::<CatListBrand, _, _, _>(|a| a % 2 == 0, list);
		/// let sat_vec: Vec<_> = satisfied.into_iter().collect();
		/// let not_sat_vec: Vec<_> = not_satisfied.into_iter().collect();
		/// assert_eq!(sat_vec, vec![2, 4]);
		/// assert_eq!(not_sat_vec, vec![1, 3]);
		/// ```
		fn partition<'a, A: 'a + Clone>(
			func: impl Fn(A) -> bool + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) {
			let mut satisfied = CatList::empty();
			let mut not_satisfied = CatList::empty();
			for a in fa {
				if func(a.clone()) {
					satisfied = satisfied.snoc(a);
				} else {
					not_satisfied = not_satisfied.snoc(a);
				}
			}
			(not_satisfied, satisfied)
		}

		/// Maps a function over a list and filters out `None` results.
		///
		/// This method maps a function over a list and filters out `None` results.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input value.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters("The function to apply.", "The list to filter and map.")]
		///
		#[document_returns("The filtered and mapped list.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
		/// let filtered = explicit::filter_map::<CatListBrand, _, _, _, _>(
		/// 	|a| if a % 2 == 0 { Some(a * 2) } else { None },
		/// 	list,
		/// );
		/// let vec: Vec<_> = filtered.into_iter().collect();
		/// assert_eq!(vec, vec![4, 8]);
		/// ```
		fn filter_map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> Option<B> + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.into_iter().filter_map(func).collect()
		}

		/// Filters a list based on a predicate.
		///
		/// This method filters a list based on a predicate.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters("The predicate.", "The list to filter.")]
		///
		#[document_returns("The filtered list.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
		/// let filtered = explicit::filter::<CatListBrand, _, _, _>(|a| a % 2 == 0, list);
		/// let vec: Vec<_> = filtered.into_iter().collect();
		/// assert_eq!(vec, vec![2, 4]);
		/// ```
		fn filter<'a, A: 'a + Clone>(
			func: impl Fn(A) -> bool + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa.into_iter().filter(|a| func(a.clone())).collect()
		}
	}

	impl FilterableWithIndex for CatListBrand {
		/// Partitions a list based on a function that receives the index and returns a [`Result`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input value.",
			"The type of the error value.",
			"The type of the success value."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and its index.",
			"The list to partition."
		)]
		///
		#[document_returns("A pair of lists.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
		/// let (errs, oks) = explicit::partition_map_with_index::<CatListBrand, _, _, _, _, _>(
		/// 	|i, a: i32| if i < 2 { Ok(a) } else { Err(a) },
		/// 	list,
		/// );
		/// let oks_vec: Vec<_> = oks.into_iter().collect();
		/// let errs_vec: Vec<_> = errs.into_iter().collect();
		/// assert_eq!(oks_vec, vec![1, 2]);
		/// assert_eq!(errs_vec, vec![3, 4]);
		/// ```
		fn partition_map_with_index<'a, A: 'a, E: 'a, O: 'a>(
			func: impl Fn(usize, A) -> Result<O, E> + 'a,
			fa: CatList<A>,
		) -> (CatList<E>, CatList<O>) {
			let mut oks = CatList::empty();
			let mut errs = CatList::empty();
			for (i, a) in fa.into_iter().enumerate() {
				match func(i, a) {
					Ok(o) => oks = oks.snoc(o),
					Err(e) => errs = errs.snoc(e),
				}
			}
			(errs, oks)
		}

		/// Partitions a list based on a predicate that receives the index.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters(
			"The predicate receiving the index and element.",
			"The list to partition."
		)]
		///
		#[document_returns("A pair of lists.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
		/// let (not_satisfied, satisfied) =
		/// 	explicit::partition_with_index::<CatListBrand, _, _, _>(|i, _a: i32| i < 2, list);
		/// let sat_vec: Vec<_> = satisfied.into_iter().collect();
		/// let not_sat_vec: Vec<_> = not_satisfied.into_iter().collect();
		/// assert_eq!(sat_vec, vec![1, 2]);
		/// assert_eq!(not_sat_vec, vec![3, 4]);
		/// ```
		fn partition_with_index<'a, A: 'a + Clone>(
			func: impl Fn(usize, A) -> bool + 'a,
			fa: CatList<A>,
		) -> (CatList<A>, CatList<A>) {
			let mut satisfied = CatList::empty();
			let mut not_satisfied = CatList::empty();
			for (i, a) in fa.into_iter().enumerate() {
				if func(i, a.clone()) {
					satisfied = satisfied.snoc(a);
				} else {
					not_satisfied = not_satisfied.snoc(a);
				}
			}
			(not_satisfied, satisfied)
		}

		/// Maps a function over a list with the index and filters out [`None`] results.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input value.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and its index.",
			"The list to filter and map."
		)]
		///
		#[document_returns("The filtered and mapped list.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
		/// let filtered = explicit::filter_map_with_index::<CatListBrand, _, _, _, _>(
		/// 	|i, a: i32| if i % 2 == 0 { Some(a * 2) } else { None },
		/// 	list,
		/// );
		/// let vec: Vec<_> = filtered.into_iter().collect();
		/// assert_eq!(vec, vec![2, 6]);
		/// ```
		fn filter_map_with_index<'a, A: 'a, B: 'a>(
			func: impl Fn(usize, A) -> Option<B> + 'a,
			fa: CatList<A>,
		) -> CatList<B> {
			fa.into_iter().enumerate().filter_map(|(i, a)| func(i, a)).collect()
		}

		/// Filters a list based on a predicate that receives the index.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters(
			"The predicate receiving the index and element.",
			"The list to filter."
		)]
		///
		#[document_returns("The filtered list.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
		/// let filtered = explicit::filter_with_index::<CatListBrand, _, _, _>(|i, _a: i32| i < 2, list);
		/// let vec: Vec<_> = filtered.into_iter().collect();
		/// assert_eq!(vec, vec![1, 2]);
		/// ```
		fn filter_with_index<'a, A: 'a + Clone>(
			func: impl Fn(usize, A) -> bool + 'a,
			fa: CatList<A>,
		) -> CatList<A> {
			fa.into_iter()
				.enumerate()
				.filter(|(i, a)| func(*i, a.clone()))
				.map(|(_, a)| a)
				.collect()
		}
	}

	impl ParFilterableWithIndex for CatListBrand {
		/// Maps and filters a list in parallel with the index, discarding elements where
		/// `f` returns `None`.
		///
		/// Single-pass implementation via Vec intermediary. Delegates to
		/// [`CatList::par_filter_map_with_index`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply to each index and element. Must be `Send + Sync`.",
			"The list to filter and map."
		)]
		///
		#[document_returns("A new list containing the `Some` results of applying `f`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_filterable_with_index::ParFilterableWithIndex,
		/// 	types::CatList,
		/// };
		///
		/// let list: CatList<i32> = vec![1, 2, 3, 4, 5].into_iter().collect();
		/// let result: Vec<_> = CatListBrand::par_filter_map_with_index(
		/// 	|i, x: i32| if i < 3 { Some(x * 10) } else { None },
		/// 	list,
		/// )
		/// .into_iter()
		/// .collect();
		/// assert_eq!(result, vec![10, 20, 30]);
		/// ```
		fn par_filter_map_with_index<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(usize, A) -> Option<B> + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			usize: Send + Sync + Copy + 'a, {
			fa.par_filter_map_with_index(f)
		}

		/// Filters a list in parallel with the index, retaining only elements satisfying `f`.
		///
		/// Single-pass implementation via Vec intermediary. Delegates to
		/// [`CatList::par_filter_with_index`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		///
		#[document_parameters(
			"The predicate receiving the index and a reference to the element. Must be `Send + Sync`.",
			"The list to filter."
		)]
		///
		#[document_returns("A new list containing only the elements satisfying `f`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_filterable_with_index::ParFilterableWithIndex,
		/// 	types::CatList,
		/// };
		///
		/// let list: CatList<i32> = vec![1, 2, 3, 4, 5].into_iter().collect();
		/// let result: Vec<_> =
		/// 	CatListBrand::par_filter_with_index(|i, x: &i32| i < 3 && x % 2 != 0, list)
		/// 		.into_iter()
		/// 		.collect();
		/// assert_eq!(result, vec![1, 3]);
		/// ```
		fn par_filter_with_index<'a, A: 'a + Send>(
			f: impl Fn(usize, &A) -> bool + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			usize: Send + Sync + Copy + 'a, {
			fa.par_filter_with_index(f)
		}
	}

	impl Witherable for CatListBrand {
		/// Partitions a list based on a function that returns a result in an applicative context.
		///
		/// This method partitions a list based on a function that returns a result in an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The applicative context.",
			"The type of the input value.",
			"The type of the error value.",
			"The type of the success value."
		)]
		///
		#[document_parameters("The function to apply.", "The list to partition.")]
		///
		#[document_returns("The partitioned list wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
		/// let wilted = explicit::wilt::<RcFnBrand, CatListBrand, OptionBrand, _, _, _, _, _>(
		/// 	|a| Some(if a % 2 == 0 { Ok(a) } else { Err(a) }),
		/// 	list,
		/// );
		/// let (errs, oks) = wilted.unwrap();
		/// let oks_vec: Vec<_> = oks.into_iter().collect();
		/// let errs_vec: Vec<_> = errs.into_iter().collect();
		/// assert_eq!(oks_vec, vec![2, 4]);
		/// assert_eq!(errs_vec, vec![1, 3]);
		/// ```
		fn wilt<'a, M: Applicative, A: 'a + Clone, E: 'a + Clone, O: 'a + Clone>(
			func: impl Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
			+ 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		(
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		),
	>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
			Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone, {
			ta.into_iter().fold(M::pure((CatList::empty(), CatList::empty())), |acc, x| {
				M::lift2(
					|mut pair, res| {
						match res {
							Ok(o) => pair.1 = pair.1.snoc(o),
							Err(e) => pair.0 = pair.0.snoc(e),
						}
						pair
					},
					acc,
					func(x),
				)
			})
		}

		/// Maps a function over a list and filters out `None` results in an applicative context.
		///
		/// This method maps a function over a list and filters out `None` results in an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The applicative context.",
			"The type of the elements in the input structure.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning an `Option` in an applicative context.",
			"The list to filter and map."
		)]
		///
		#[document_returns("The filtered and mapped list wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
		/// let withered = explicit::wither::<RcFnBrand, CatListBrand, OptionBrand, _, _, _, _>(
		/// 	|a| Some(if a % 2 == 0 { Some(a * 2) } else { None }),
		/// 	list,
		/// );
		/// let vec: Vec<_> = withered.unwrap().into_iter().collect();
		/// assert_eq!(vec, vec![4, 8]);
		/// ```
		fn wither<'a, M: Applicative, A: 'a + Clone, B: 'a + Clone>(
			func: impl Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
			Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone, {
			ta.into_iter().fold(M::pure(CatList::empty()), |acc, x| {
				M::lift2(
					|list, opt_b| {
						if let Some(b) = opt_b { list.snoc(b) } else { list }
					},
					acc,
					func(x),
				)
			})
		}
	}

	/// Cooperative extension for [`CatList`], following the same suffix semantics
	/// as the PureScript `Extend Array` instance.
	///
	/// `extend(f, list)` produces a new list where each element is `f` applied to
	/// the suffix of the original list starting at that position. Requires
	/// `A: Clone` because suffixes are materialized as owned lists.
	impl Extend for CatListBrand {
		/// Extends a local context-dependent computation to a global computation
		/// over [`CatList`].
		///
		/// Applies `f` to every suffix of the input list. For a list `[a, b, c]`,
		/// the result is `[f([a, b, c]), f([b, c]), f([c])]`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the list.",
			"The result type of the extension function."
		)]
		///
		#[document_parameters(
			"The function that consumes a suffix list and produces a value.",
			"The list to extend over."
		)]
		///
		#[document_returns(
			"A new list containing the results of applying the function to each suffix."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let result = extend::<CatListBrand, _, _>(|cl: CatList<i32>| cl.into_iter().sum::<i32>(), list);
		/// let vec: Vec<_> = result.into_iter().collect();
		/// assert_eq!(vec, vec![6, 5, 3]);
		/// ```
		fn extend<'a, A: 'a + Clone, B: 'a>(
			f: impl Fn(Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
			wa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			// Collect to a Vec for indexed suffix access, then build the result
			// as a CatList.
			let elements: Vec<A> = wa.into_iter().collect();
			(0 .. elements.len())
				.map(|i| f(elements.get(i ..).unwrap_or_default().iter().cloned().collect()))
				.collect()
		}
	}

	impl MonadRec for CatListBrand {
		/// Performs tail-recursive monadic computation over [`CatList`].
		///
		/// Since `CatList` represents nondeterminism, this performs a breadth-first
		/// expansion: each iteration maps all current `Loop` states through the step
		/// function, collecting `Done` results as they appear. The computation
		/// terminates when no `Loop` values remain.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns("A CatList of all completed results.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 		types::*,
		/// 	},
		/// };
		///
		/// // Branch into two paths, each running until done
		/// let result = tail_rec_m::<CatListBrand, _, _>(
		/// 	|n| {
		/// 		if n < 3 {
		/// 			CatList::singleton(ControlFlow::Continue(n + 1)).snoc(ControlFlow::Break(n * 10))
		/// 		} else {
		/// 			CatList::singleton(ControlFlow::Break(n * 10))
		/// 		}
		/// 	},
		/// 	0,
		/// );
		/// // Starting from 0: branches at 0,1,2; done at 3
		/// let vec: Vec<_> = result.into_iter().collect();
		/// assert_eq!(vec, vec![0, 10, 20, 30]);
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let mut done: CatList<B> = CatList::empty();
			let mut pending: CatList<A> = CatList::singleton(initial);
			while !pending.is_empty() {
				let mut next_pending: CatList<A> = CatList::empty();
				for a in pending {
					for step in func(a) {
						match step {
							ControlFlow::Continue(next) => next_pending = next_pending.snoc(next),
							ControlFlow::Break(b) => done = done.snoc(b),
						}
					}
				}
				pending = next_pending;
			}
			done
		}
	}

	#[document_type_parameters("The type of the elements in the list.")]
	impl<A> Semigroup for CatList<A> {
		/// Appends one list to another.
		///
		/// This method concatenates two lists.
		#[document_signature]
		///
		#[document_parameters("The first list.", "The second list.")]
		///
		#[document_returns("The concatenated list.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list1 = CatList::singleton(1).snoc(2);
		/// let list2 = CatList::singleton(3).snoc(4);
		/// let appended = append(list1, list2);
		/// let vec: Vec<_> = appended.into_iter().collect();
		/// assert_eq!(vec, vec![1, 2, 3, 4]);
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			a.append(b)
		}
	}

	#[document_type_parameters("The type of the elements in the list.")]
	impl<A> Monoid for CatList<A> {
		/// Returns an empty list.
		///
		/// This method returns a new, empty list.
		#[document_signature]
		///
		#[document_returns("An empty list.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = empty::<CatList<i32>>();
		/// assert!(list.is_empty());
		/// ```
		fn empty() -> Self {
			CatList::empty()
		}
	}

	#[document_type_parameters("The type of the elements in the list.")]
	#[document_parameters("The list to operate on.")]
	impl<A> CatList<A> {
		/// Creates an empty CatList.
		#[document_signature]
		///
		#[document_returns("An empty `CatList`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list: CatList<i32> = CatList::empty();
		/// assert!(list.is_empty());
		/// ```
		#[inline]
		pub const fn empty() -> Self {
			CatList(CatListInner::Nil)
		}

		/// Returns `true` if the list is empty.
		#[document_signature]
		///
		#[document_parameters]
		///
		#[document_returns("`true` if the list is empty, `false` otherwise.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list: CatList<i32> = CatList::empty();
		/// assert!(list.is_empty());
		/// ```
		pub fn is_empty(&self) -> bool {
			matches!(self.0, CatListInner::Nil)
		}

		/// Creates a CatList with a single element.
		#[document_signature]
		///
		#[document_parameters("The element to put in the list.")]
		///
		#[document_returns("A `CatList` containing the single element.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let list = CatList::singleton(1);
		/// assert!(!list.is_empty());
		/// ```
		pub fn singleton(a: A) -> Self {
			CatList(CatListInner::Cons(a, VecDeque::new(), 1))
		}

		/// Appends an element to the front of the list.
		#[document_signature]
		///
		#[document_parameters("The element to prepend.")]
		///
		#[document_returns("The new list with the element appended to the front.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let list = CatList::empty().cons(1);
		/// assert_eq!(list.len(), 1);
		/// ```
		pub fn cons(
			self,
			a: A,
		) -> Self {
			Self::link(CatList::singleton(a), self)
		}

		/// Appends an element to the back of the list.
		#[document_signature]
		///
		#[document_parameters("The element to append.")]
		///
		#[document_returns("The new list with the element appended to the back.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let list = CatList::empty().snoc(1);
		/// assert_eq!(list.len(), 1);
		/// ```
		pub fn snoc(
			self,
			a: A,
		) -> Self {
			Self::link(self, CatList::singleton(a))
		}

		/// Concatenates two CatLists.
		///
		/// This is the key operation that makes CatList special:
		/// concatenation is O(1), not O(n).
		#[document_signature]
		///
		#[document_parameters("The second list.")]
		///
		#[document_returns("The concatenated list.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let list1 = CatList::singleton(1);
		/// let list2 = CatList::singleton(2);
		/// let list3 = list1.append(list2);
		/// assert_eq!(list3.len(), 2);
		/// ```
		pub fn append(
			self,
			other: Self,
		) -> Self {
			Self::link(self, other)
		}

		/// Internal linking operation.
		///
		/// Links two `CatList`s by pushing the second onto the first's sublist deque.
		#[document_signature]
		///
		#[document_parameters("The first list.", "The second list.")]
		///
		#[document_returns("A new list consisting of the two input lists linked together.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// // link is internal, but we can use it via other methods
		/// let list1 = CatList::singleton(1);
		/// let list2 = CatList::singleton(2);
		/// let linked = list1.append(list2);
		/// assert_eq!(linked.len(), 2);
		/// ```
		fn link(
			mut left: Self,
			right: Self,
		) -> Self {
			if left.is_empty() {
				return right;
			}
			if right.is_empty() {
				return left;
			}
			if let CatListInner::Cons(_, q, len) = &mut left.0 {
				*len += right.len();
				q.push_back(right);
			}
			left
		}

		/// Removes and returns the first element.
		///
		/// Returns `None` if the list is empty. When the internal sublist deque
		/// is non-empty, this operation calls `flatten_deque` to restructure the
		/// remaining elements, which may traverse multiple deque entries. However,
		/// each entry is visited at most once across a full sequence of `uncons`
		/// calls, yielding O(1) amortized cost per element.
		#[document_signature]
		///
		#[document_parameters]
		///
		#[document_returns(
			"An option containing the first element and the rest of the list, or `None` if empty."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let list = CatList::singleton(1);
		/// let (a, list) = list.uncons().unwrap();
		/// assert_eq!(a, 1);
		/// assert!(list.is_empty());
		/// ```
		pub fn uncons(mut self) -> Option<(A, Self)> {
			let inner = std::mem::replace(&mut self.0, CatListInner::Nil);
			// SAFETY: `inner` now owns the original data, and `self.0` is `Nil`.
			// `mem::forget` skips `CatList`'s custom `Drop` (which would
			// redundantly walk the now-empty sentinel). This is sound because
			// `CatListInner` has no `Drop` impl; if one is ever added, this
			// code must be restructured to avoid skipping resource cleanup.
			std::mem::forget(self);
			match inner {
				CatListInner::Nil => None,
				CatListInner::Cons(a, q, _) =>
					if q.is_empty() {
						Some((a, CatList::empty()))
					} else {
						Some((a, Self::flatten_deque(q)))
					},
			}
		}

		/// Flattens a deque of CatLists into a single CatList.
		///
		/// This is equivalent to `foldr link CatNil deque` in PureScript.
		///
		/// Uses a stack-safe right fold (`rfold`) over the deque's `DoubleEndedIterator`.
		#[document_signature]
		///
		#[document_parameters("The deque of sublists to flatten.")]
		///
		#[document_returns("A single flattened `CatList`.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::types::cat_list::CatList,
		/// 	std::collections::VecDeque,
		/// };
		///
		/// let mut deque = VecDeque::new();
		/// deque.push_back(CatList::singleton(1));
		/// deque.push_back(CatList::singleton(2));
		/// // flatten_deque is internal, but used by uncons
		/// let list = CatList::singleton(0).append(CatList::singleton(1));
		/// assert_eq!(list.len(), 2);
		/// ```
		fn flatten_deque(deque: VecDeque<CatList<A>>) -> Self {
			// Right fold: link(list[0], link(list[1], ... link(list[n-1], Nil)))
			// We process from right to left using DoubleEndedIterator
			deque.into_iter().rfold(CatList::empty(), |acc, list| Self::link(list, acc))
		}

		/// Returns the number of elements.
		#[document_signature]
		///
		#[document_parameters]
		///
		#[document_returns("The number of elements in the list.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let list = CatList::singleton(1);
		/// assert_eq!(list.len(), 1);
		/// ```
		pub fn len(&self) -> usize {
			match &self.0 {
				CatListInner::Nil => 0,
				CatListInner::Cons(_, _, len) => *len,
			}
		}

		/// Returns a borrowing iterator over the list's elements.
		///
		/// This iterator yields shared references without consuming the list,
		/// using a stack-based depth-first traversal of the internal tree structure.
		#[document_signature]
		///
		#[document_parameters]
		///
		#[document_returns("A borrowing iterator over the elements of the list.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let refs: Vec<_> = list.iter().collect();
		/// assert_eq!(refs, vec![&1, &2, &3]);
		/// ```
		pub fn iter(&self) -> CatListIter<'_, A> {
			match &self.0 {
				CatListInner::Nil => CatListIter {
					stack: Vec::new(),
					current_head: None,
					remaining: 0,
				},
				CatListInner::Cons(a, deque, len) => {
					let mut stack = Vec::new();
					if !deque.is_empty() {
						stack.push(deque.iter());
					}
					CatListIter {
						stack,
						current_head: Some(a),
						remaining: *len,
					}
				}
			}
		}

		/// Wraps a value in a singleton list.
		///
		/// This is the inherent method form of [`Pointed::pure`](crate::classes::pointed::Pointed::pure).
		/// Equivalent to [`CatList::singleton`].
		#[document_signature]
		#[document_parameters("The value to wrap.")]
		#[document_returns("A `CatList` containing the single value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::pure(42);
		/// assert_eq!(list.len(), 1);
		/// ```
		pub fn pure(a: A) -> Self {
			CatList::singleton(a)
		}

		/// Maps a function over each element of the list.
		///
		/// This is the inherent method form of [`Functor::map`](crate::classes::functor::Functor::map).
		#[document_signature]
		#[document_type_parameters("The type of the elements in the resulting list.")]
		#[document_parameters("The function to apply to each element.")]
		#[document_returns("A new list containing the mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let mapped = list.map(|x| x * 2);
		/// let vec: Vec<_> = mapped.into_iter().collect();
		/// assert_eq!(vec, vec![2, 4, 6]);
		/// ```
		pub fn map<B>(
			self,
			f: impl FnMut(A) -> B,
		) -> CatList<B> {
			self.into_iter().map(f).collect()
		}

		/// Chains list computations (flat_map).
		///
		/// Applies a function returning a list to each element and flattens the result.
		/// This is the inherent method form of [`Semimonad::bind`](crate::classes::semimonad::Semimonad::bind).
		#[document_signature]
		#[document_type_parameters("The type of the elements in the resulting list.")]
		#[document_parameters("The function to apply to each element, returning a list.")]
		#[document_returns("A new list containing the flattened results.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(1).snoc(2);
		/// let bound = list.bind(|x| CatList::singleton(x).snoc(x * 2));
		/// let vec: Vec<_> = bound.into_iter().collect();
		/// assert_eq!(vec, vec![1, 2, 2, 4]);
		/// ```
		pub fn bind<B>(
			self,
			f: impl FnMut(A) -> CatList<B>,
		) -> CatList<B> {
			self.into_iter().flat_map(f).collect()
		}

		/// Folds the list from the right.
		///
		/// This is the inherent method form of [`Foldable::fold_right`](crate::classes::foldable::Foldable::fold_right).
		#[document_signature]
		#[document_type_parameters("The type of the accumulator.")]
		#[document_parameters("The folding function.", "The initial accumulator value.")]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// assert_eq!(list.fold_right(|x, acc| x + acc, 0), 6);
		/// ```
		pub fn fold_right<B>(
			self,
			f: impl Fn(A, B) -> B,
			initial: B,
		) -> B {
			self.into_iter().collect::<Vec<_>>().into_iter().rfold(initial, |acc, x| f(x, acc))
		}

		/// Folds the list from the left.
		///
		/// This is the inherent method form of [`Foldable::fold_left`](crate::classes::foldable::Foldable::fold_left).
		#[document_signature]
		#[document_type_parameters("The type of the accumulator.")]
		#[document_parameters("The folding function.", "The initial accumulator value.")]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// assert_eq!(list.fold_left(|acc, x| acc + x, 0), 6);
		/// ```
		pub fn fold_left<B>(
			self,
			f: impl Fn(B, A) -> B,
			initial: B,
		) -> B {
			self.into_iter().fold(initial, f)
		}

		/// Maps each element to a monoid and combines the results.
		///
		/// This is the inherent method form of [`Foldable::fold_map`](crate::classes::foldable::Foldable::fold_map).
		#[document_signature]
		#[document_type_parameters("The monoid type.")]
		#[document_parameters("The mapping function.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// assert_eq!(list.fold_map(|x: i32| x.to_string()), "123".to_string());
		/// ```
		pub fn fold_map<M: Monoid>(
			self,
			f: impl FnMut(A) -> M,
		) -> M {
			self.into_iter().map(f).fold(M::empty(), |acc, x| M::append(acc, x))
		}

		/// Maps a function over each element, providing the index.
		///
		/// This is the inherent method form of [`FunctorWithIndex::map_with_index`](crate::classes::functor_with_index::FunctorWithIndex::map_with_index).
		#[document_signature]
		#[document_type_parameters("The type of the elements in the resulting list.")]
		#[document_parameters("The function to apply to each element and its index.")]
		#[document_returns("A new list containing the mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(10).snoc(20).snoc(30);
		/// let mapped = list.map_with_index(|i, x| x + i as i32);
		/// let vec: Vec<_> = mapped.into_iter().collect();
		/// assert_eq!(vec, vec![10, 21, 32]);
		/// ```
		pub fn map_with_index<B>(
			self,
			mut f: impl FnMut(usize, A) -> B,
		) -> CatList<B> {
			self.into_iter().enumerate().map(|(i, a)| f(i, a)).collect()
		}

		/// Maps each element to a monoid and combines the results, providing the index.
		///
		/// This is the inherent method form of [`FoldableWithIndex::fold_map_with_index`](crate::classes::foldable_with_index::FoldableWithIndex::fold_map_with_index).
		#[document_signature]
		#[document_type_parameters("The monoid type.")]
		#[document_parameters("The function to apply to each element and its index.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(10).snoc(20).snoc(30);
		/// let result = list.fold_map_with_index(|i, x: i32| format!("{i}:{x}"));
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		pub fn fold_map_with_index<M: Monoid>(
			self,
			mut f: impl FnMut(usize, A) -> M,
		) -> M {
			self.into_iter()
				.enumerate()
				.map(|(i, a)| f(i, a))
				.fold(M::empty(), |acc, x| M::append(acc, x))
		}

		/// Traverses the list with an applicative function, providing the index.
		///
		/// This is the inherent method form of [`TraversableWithIndex::traverse_with_index`](crate::classes::traversable_with_index::TraversableWithIndex::traverse_with_index).
		#[document_signature]
		#[document_type_parameters(
			"The type of the elements in the resulting list.",
			"The applicative context."
		)]
		#[document_parameters(
			"The function to apply to each element and its index, returning a value in an applicative context."
		)]
		#[document_returns("The list wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::cat_list::CatList,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let result = list
		/// 	.traverse_with_index::<i32, OptionBrand>(|_i, x| if x > 0 { Some(x * 2) } else { None });
		/// let vec: Vec<_> = result.unwrap().into_iter().collect();
		/// assert_eq!(vec, vec![2, 4, 6]);
		/// ```
		pub fn traverse_with_index<B: Clone, M: Applicative>(
			self,
			f: impl Fn(usize, A) -> M::Of<'static, B>,
		) -> M::Of<'static, CatList<B>>
		where
			CatList<B>: Clone,
			M::Of<'static, B>: Clone, {
			self.into_iter().enumerate().fold(M::pure(CatList::empty()), |acc, (i, x)| {
				M::lift2(|list, b| list.snoc(b), acc, f(i, x))
			})
		}

		/// Maps a function over the list in parallel via a `Vec` intermediary.
		///
		/// Collects to `Vec`, applies `f` in parallel (or sequentially without rayon), then
		/// reconstructs a `CatList`.
		#[document_signature]
		#[document_type_parameters("The type of the elements in the resulting list.")]
		#[document_parameters("The function to apply to each element. Must be `Send + Sync`.")]
		#[document_returns("A new list containing the mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::CatList;
		///
		/// let list: CatList<i32> = vec![1, 2, 3].into_iter().collect();
		/// let result: Vec<_> = list.par_map(|x: i32| x * 2).into_iter().collect();
		/// assert_eq!(result, vec![2, 4, 6]);
		/// ```
		pub fn par_map<B: Send>(
			self,
			f: impl Fn(A) -> B + Send + Sync,
		) -> CatList<B>
		where
			A: Send, {
			let v: Vec<A> = self.into_iter().collect();
			#[cfg(feature = "rayon")]
			let result: Vec<B> = {
				use rayon::prelude::*;
				v.into_par_iter().map(f).collect()
			};
			#[cfg(not(feature = "rayon"))]
			let result: Vec<B> = v.into_iter().map(f).collect();
			result.into_iter().collect()
		}

		/// Maps and filters the list in parallel, discarding elements where `f` returns `None`.
		///
		/// Collects to `Vec`, applies `filter_map` in parallel (or sequentially without rayon),
		/// then reconstructs a `CatList`.
		#[document_signature]
		#[document_type_parameters("The type of the elements in the resulting list.")]
		#[document_parameters("The function to apply. Must be `Send + Sync`.")]
		#[document_returns("A new list containing the `Some` results of applying `f`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::CatList;
		///
		/// let list: CatList<i32> = vec![1, 2, 3, 4, 5].into_iter().collect();
		/// let result: Vec<_> = list
		/// 	.par_filter_map(|x: i32| if x % 2 == 0 { Some(x * 10) } else { None })
		/// 	.into_iter()
		/// 	.collect();
		/// assert_eq!(result, vec![20, 40]);
		/// ```
		pub fn par_filter_map<B: Send>(
			self,
			f: impl Fn(A) -> Option<B> + Send + Sync,
		) -> CatList<B>
		where
			A: Send, {
			let v: Vec<A> = self.into_iter().collect();
			#[cfg(feature = "rayon")]
			let result: Vec<B> = {
				use rayon::prelude::*;
				v.into_par_iter().filter_map(f).collect()
			};
			#[cfg(not(feature = "rayon"))]
			let result: Vec<B> = v.into_iter().filter_map(f).collect();
			result.into_iter().collect()
		}

		/// Filters the list in parallel, retaining only elements satisfying `f`.
		///
		/// Collects to `Vec`, filters in parallel (or sequentially without rayon), then
		/// reconstructs a `CatList`.
		#[document_signature]
		#[document_parameters("The predicate. Must be `Send + Sync`.")]
		#[document_returns("A new list containing only the elements satisfying `f`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::CatList;
		///
		/// let list: CatList<i32> = vec![1, 2, 3, 4, 5].into_iter().collect();
		/// let result: Vec<_> = list.par_filter(|x: &i32| x % 2 == 0).into_iter().collect();
		/// assert_eq!(result, vec![2, 4]);
		/// ```
		pub fn par_filter(
			self,
			f: impl Fn(&A) -> bool + Send + Sync,
		) -> CatList<A>
		where
			A: Send, {
			let v: Vec<A> = self.into_iter().collect();
			#[cfg(feature = "rayon")]
			let result: Vec<A> = {
				use rayon::prelude::*;
				v.into_par_iter().filter(|a| f(a)).collect()
			};
			#[cfg(not(feature = "rayon"))]
			let result: Vec<A> = v.into_iter().filter(|a| f(a)).collect();
			result.into_iter().collect()
		}

		/// Maps each element to a [`Monoid`] value and combines them in parallel.
		///
		/// Collects to `Vec`, then maps and reduces in parallel (or sequentially without rayon).
		#[document_signature]
		#[document_type_parameters("The monoid type.")]
		#[document_parameters(
			"The function mapping each element to a monoid value. Must be `Send + Sync`."
		)]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::CatList;
		///
		/// let list: CatList<i32> = vec![1, 2, 3].into_iter().collect();
		/// let result = list.par_fold_map(|x: i32| x.to_string());
		/// assert_eq!(result, "123");
		/// ```
		pub fn par_fold_map<M: Monoid + Send>(
			self,
			f: impl Fn(A) -> M + Send + Sync,
		) -> M
		where
			A: Send, {
			let v: Vec<A> = self.into_iter().collect();
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				v.into_par_iter().map(f).reduce(M::empty, |acc, m| M::append(acc, m))
			}
			#[cfg(not(feature = "rayon"))]
			v.into_iter().map(f).fold(M::empty(), |acc, m| M::append(acc, m))
		}

		/// Maps a function over the list in parallel, providing each element's index.
		///
		/// Collects to `Vec`, applies the indexed mapping in parallel (or sequentially without
		/// rayon), then reconstructs a `CatList`.
		#[document_signature]
		#[document_type_parameters("The type of the elements in the resulting list.")]
		#[document_parameters(
			"The function to apply to each index and element. Must be `Send + Sync`."
		)]
		#[document_returns("A new list containing the mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::CatList;
		///
		/// let list: CatList<i32> = vec![10, 20, 30].into_iter().collect();
		/// let result: Vec<_> = list.par_map_with_index(|i, x: i32| x + i as i32).into_iter().collect();
		/// assert_eq!(result, vec![10, 21, 32]);
		/// ```
		pub fn par_map_with_index<B: Send>(
			self,
			f: impl Fn(usize, A) -> B + Send + Sync,
		) -> CatList<B>
		where
			A: Send, {
			let v: Vec<A> = self.into_iter().collect();
			#[cfg(feature = "rayon")]
			let result: Vec<B> = {
				use rayon::prelude::*;
				v.into_par_iter().enumerate().map(|(i, a)| f(i, a)).collect()
			};
			#[cfg(not(feature = "rayon"))]
			let result: Vec<B> = v.into_iter().enumerate().map(|(i, a)| f(i, a)).collect();
			result.into_iter().collect()
		}

		/// Maps each element and its index to a [`Monoid`] value and combines them in parallel.
		///
		/// Collects to `Vec`, then maps with index and reduces in parallel (or sequentially
		/// without rayon).
		#[document_signature]
		#[document_type_parameters("The monoid type.")]
		#[document_parameters(
			"The function mapping each index and element to a monoid value. Must be `Send + Sync`."
		)]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::CatList;
		///
		/// let list: CatList<i32> = vec![10, 20, 30].into_iter().collect();
		/// let result = list.par_fold_map_with_index(|i, x: i32| format!("{i}:{x}"));
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		pub fn par_fold_map_with_index<M: Monoid + Send>(
			self,
			f: impl Fn(usize, A) -> M + Send + Sync,
		) -> M
		where
			A: Send, {
			let v: Vec<A> = self.into_iter().collect();
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				v.into_par_iter()
					.enumerate()
					.map(|(i, a)| f(i, a))
					.reduce(M::empty, |acc, m| M::append(acc, m))
			}
			#[cfg(not(feature = "rayon"))]
			v.into_iter()
				.enumerate()
				.map(|(i, a)| f(i, a))
				.fold(M::empty(), |acc, m| M::append(acc, m))
		}

		/// Maps and filters a list in parallel with the index, discarding elements where
		/// `f` returns `None`.
		///
		/// Collects to `Vec`, filters with index in parallel (or sequentially without rayon),
		/// then reconstructs a `CatList`.
		#[document_signature]
		#[document_type_parameters("The type of the elements in the resulting list.")]
		#[document_parameters(
			"The function to apply to each index and element. Must be `Send + Sync`."
		)]
		#[document_returns("A new list containing the `Some` results of applying `f`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::CatList;
		///
		/// let list: CatList<i32> = vec![1, 2, 3, 4, 5].into_iter().collect();
		/// let result: Vec<_> = list
		/// 	.par_filter_map_with_index(|i, x: i32| if i < 3 { Some(x * 10) } else { None })
		/// 	.into_iter()
		/// 	.collect();
		/// assert_eq!(result, vec![10, 20, 30]);
		/// ```
		pub fn par_filter_map_with_index<B: Send>(
			self,
			f: impl Fn(usize, A) -> Option<B> + Send + Sync,
		) -> CatList<B>
		where
			A: Send, {
			let v: Vec<A> = self.into_iter().collect();
			#[cfg(feature = "rayon")]
			let result: Vec<B> = {
				use rayon::prelude::*;
				v.into_par_iter().enumerate().filter_map(|(i, a)| f(i, a)).collect()
			};
			#[cfg(not(feature = "rayon"))]
			let result: Vec<B> = v.into_iter().enumerate().filter_map(|(i, a)| f(i, a)).collect();
			result.into_iter().collect()
		}

		/// Filters a list in parallel with the index, retaining only elements satisfying `f`.
		///
		/// Collects to `Vec`, filters with index in parallel (or sequentially without rayon),
		/// then reconstructs a `CatList`.
		#[document_signature]
		#[document_parameters(
			"The predicate receiving the index and a reference to the element. Must be `Send + Sync`."
		)]
		#[document_returns("A new list containing only the elements satisfying `f`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::CatList;
		///
		/// let list: CatList<i32> = vec![1, 2, 3, 4, 5].into_iter().collect();
		/// let result: Vec<_> =
		/// 	list.par_filter_with_index(|i, x: &i32| i < 3 && x % 2 != 0).into_iter().collect();
		/// assert_eq!(result, vec![1, 3]);
		/// ```
		pub fn par_filter_with_index(
			self,
			f: impl Fn(usize, &A) -> bool + Send + Sync,
		) -> CatList<A>
		where
			A: Send, {
			let v: Vec<A> = self.into_iter().collect();
			#[cfg(feature = "rayon")]
			let result: Vec<A> = {
				use rayon::prelude::*;
				v.into_par_iter().enumerate().filter(|(i, a)| f(*i, a)).map(|(_, a)| a).collect()
			};
			#[cfg(not(feature = "rayon"))]
			let result: Vec<A> =
				v.into_iter().enumerate().filter(|(i, a)| f(*i, a)).map(|(_, a)| a).collect();
			result.into_iter().collect()
		}
	}

	#[document_type_parameters("The type of the values inside the `Option`s.")]
	#[document_parameters("The list of options.")]
	impl<A> CatList<Option<A>> {
		/// Compacts a list of options in parallel, discarding `None` values.
		///
		/// Collects to `Vec<Option<A>>`, flattens in parallel (or sequentially without rayon),
		/// then reconstructs a `CatList`.
		#[document_signature]
		#[document_returns("A new list containing the unwrapped `Some` values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::CatList;
		///
		/// let list: CatList<Option<i32>> = vec![Some(1), None, Some(3)].into_iter().collect();
		/// let result: Vec<_> = list.par_compact().into_iter().collect();
		/// assert_eq!(result, vec![1, 3]);
		/// ```
		pub fn par_compact(self) -> CatList<A>
		where
			A: Send, {
			let v: Vec<Option<A>> = self.into_iter().collect();
			#[cfg(feature = "rayon")]
			let result: Vec<A> = {
				use rayon::prelude::*;
				v.into_par_iter().flatten().collect()
			};
			#[cfg(not(feature = "rayon"))]
			let result: Vec<A> = v.into_iter().flatten().collect();
			result.into_iter().collect()
		}
	}

	#[document_type_parameters("The error type.", "The success type.")]
	#[document_parameters("The list of results.")]
	impl<E, O> CatList<Result<O, E>> {
		/// Separates a list of results into `(errors, oks)` in parallel.
		///
		/// Collects to `Vec`, partitions in parallel (or sequentially without rayon), then
		/// reconstructs two `CatList` values.
		#[document_signature]
		#[document_returns(
			"A pair `(errs, oks)` where `errs` contains the `Err` values and `oks` the `Ok` values."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::CatList;
		///
		/// let list: CatList<Result<i32, &str>> = vec![Ok(1), Err("a"), Ok(3)].into_iter().collect();
		/// let (errs, oks): (CatList<&str>, CatList<i32>) = list.par_separate();
		/// assert_eq!(errs.into_iter().collect::<Vec<_>>(), vec!["a"]);
		/// assert_eq!(oks.into_iter().collect::<Vec<_>>(), vec![1, 3]);
		/// ```
		pub fn par_separate(self) -> (CatList<E>, CatList<O>)
		where
			E: Send,
			O: Send, {
			let v: Vec<Result<O, E>> = self.into_iter().collect();
			#[cfg(feature = "rayon")]
			{
				use rayon::{
					iter::Either,
					prelude::*,
				};
				let (errs, oks): (Vec<E>, Vec<O>) = v.into_par_iter().partition_map(|r| match r {
					Ok(o) => Either::Right(o),
					Err(e) => Either::Left(e),
				});
				(errs.into_iter().collect(), oks.into_iter().collect())
			}
			#[cfg(not(feature = "rayon"))]
			{
				let mut errs = Vec::new();
				let mut oks = Vec::new();
				for result in v {
					match result {
						Ok(o) => oks.push(o),
						Err(e) => errs.push(e),
					}
				}
				(errs.into_iter().collect(), oks.into_iter().collect())
			}
		}
	}

	// Iteration support
	#[document_type_parameters("The type of the elements in the list.")]
	#[document_parameters("The list to consume.")]
	impl<A> IntoIterator for CatList<A> {
		type IntoIter = CatListIterator<A>;
		type Item = A;

		#[document_signature]
		#[document_returns("An iterator that consumes the list and yields its elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(1).snoc(2);
		/// let vec: Vec<_> = list.into_iter().collect();
		/// assert_eq!(vec, vec![1, 2]);
		/// ```
		fn into_iter(self) -> Self::IntoIter {
			CatListIterator(self)
		}
	}

	/// An iterator that consumes a `CatList`.
	#[document_type_parameters("The type of the elements in the list.")]
	///
	pub struct CatListIterator<A>(
		/// The list being iterated over.
		CatList<A>,
	);

	#[document_type_parameters("The type of the elements in the list.")]
	#[document_parameters("The iterator state.")]
	impl<A> Iterator for CatListIterator<A> {
		type Item = A;

		#[document_signature]
		#[document_returns("The next element in the list, or `None` if the iterator is exhausted.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(1);
		/// let mut iter = list.into_iter();
		/// assert_eq!(iter.next(), Some(1));
		/// assert_eq!(iter.next(), None);
		/// ```
		fn next(&mut self) -> Option<Self::Item> {
			let (head, tail) = std::mem::take(&mut self.0).uncons()?;
			self.0 = tail;
			Some(head)
		}

		#[document_signature]
		#[document_returns(
			"A lower bound and optional exact upper bound on the number of remaining elements."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let mut iter = list.into_iter();
		/// assert_eq!(iter.size_hint(), (3, Some(3)));
		/// let _ = iter.next();
		/// assert_eq!(iter.size_hint(), (2, Some(2)));
		/// ```
		fn size_hint(&self) -> (usize, Option<usize>) {
			let len = self.0.len();
			(len, Some(len))
		}
	}

	#[document_type_parameters("The type of the elements in the list.")]
	impl<A> ExactSizeIterator for CatListIterator<A> {}

	/// A borrowing iterator over a `CatList`.
	///
	/// This iterator yields shared references to the elements without consuming
	/// the list. It uses a stack-based depth-first traversal of the internal
	/// tree structure.
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The type of the elements in the list."
	)]
	///
	pub struct CatListIter<'a, A> {
		/// Stack of deque iterators for depth-first traversal.
		stack: Vec<std::collections::vec_deque::Iter<'a, CatList<A>>>,
		/// The next head element to yield, if any.
		current_head: Option<&'a A>,
		/// The number of remaining elements.
		remaining: usize,
	}

	#[document_type_parameters(
		"The lifetime of the elements.",
		"The type of the elements in the list."
	)]
	#[document_parameters("The iterator state.")]
	impl<'a, A> Iterator for CatListIter<'a, A> {
		type Item = &'a A;

		#[document_signature]
		#[document_returns(
			"A shared reference to the next element in the list, or `None` if the iterator is exhausted."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let mut iter = list.iter();
		/// assert_eq!(iter.next(), Some(&1));
		/// assert_eq!(iter.next(), Some(&2));
		/// assert_eq!(iter.next(), Some(&3));
		/// assert_eq!(iter.next(), None);
		/// ```
		fn next(&mut self) -> Option<Self::Item> {
			// If we have a head element queued, yield it
			if let Some(head) = self.current_head.take() {
				self.remaining -= 1;
				return Some(head);
			}

			// Otherwise, pop from the stack until we find a non-empty deque iterator
			while let Some(deque_iter) = self.stack.last_mut() {
				if let Some(sublist) = deque_iter.next() {
					match &sublist.0 {
						CatListInner::Nil => continue,
						CatListInner::Cons(a, deque, _) => {
							// Push the deque's children onto the stack for later traversal
							if !deque.is_empty() {
								self.stack.push(deque.iter());
							}
							self.remaining -= 1;
							return Some(a);
						}
					}
				} else {
					self.stack.pop();
				}
			}

			None
		}

		#[document_signature]
		#[document_returns(
			"A lower bound and optional exact upper bound on the number of remaining elements."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let mut iter = list.iter();
		/// assert_eq!(iter.size_hint(), (3, Some(3)));
		/// let _ = iter.next();
		/// assert_eq!(iter.size_hint(), (2, Some(2)));
		/// ```
		fn size_hint(&self) -> (usize, Option<usize>) {
			(self.remaining, Some(self.remaining))
		}
	}

	#[document_type_parameters("The type of the elements in the list.")]
	impl<A> ExactSizeIterator for CatListIter<'_, A> {}

	#[document_type_parameters(
		"The lifetime of the elements.",
		"The type of the elements in the list."
	)]
	#[document_parameters("The list to borrow.")]
	impl<'a, A> IntoIterator for &'a CatList<A> {
		type IntoIter = CatListIter<'a, A>;
		type Item = &'a A;

		#[document_signature]
		#[document_returns("A borrowing iterator over the list's elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let list = CatList::singleton(1).snoc(2);
		/// let refs: Vec<_> = (&list).into_iter().collect();
		/// assert_eq!(refs, vec![&1, &2]);
		/// // list is still usable
		/// assert_eq!(list.len(), 2);
		/// ```
		fn into_iter(self) -> Self::IntoIter {
			self.iter()
		}
	}

	// FromIterator for easy construction
	#[document_type_parameters("The type of the elements in the list.")]
	impl<A> FromIterator<A> for CatList<A> {
		#[document_signature]
		#[document_type_parameters("The iterator type.")]
		#[document_parameters("The iterator to collect from.")]
		#[document_returns("A new `CatList` containing the elements from the iterator.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let vec = vec![1, 2, 3];
		/// let list: CatList<_> = CatList::from_iter(vec);
		/// assert_eq!(list.len(), 3);
		/// ```
		fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
			let mut iter = iter.into_iter();
			match iter.next() {
				None => CatList::empty(),
				Some(first) => {
					let mut deque = VecDeque::new();
					let mut count = 1usize;
					for item in iter {
						deque.push_back(CatList::singleton(item));
						count += 1;
					}
					CatList(CatListInner::Cons(first, deque, count))
				}
			}
		}
	}

	// --- Display ---

	#[document_type_parameters("The type of the elements in the list.")]
	#[document_parameters("The list to display.")]
	impl<A: fmt::Display> fmt::Display for CatList<A> {
		/// Displays the list in bracket notation (e.g., `[1, 2, 3]`).
		///
		/// An empty list displays as `[]`.
		#[document_signature]
		///
		#[document_parameters("The formatter.")]
		///
		#[document_returns("The formatting result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		///
		/// let empty: CatList<i32> = CatList::empty();
		/// assert_eq!(format!("{}", empty), "[]");
		///
		/// let single = CatList::singleton(42);
		/// assert_eq!(format!("{}", single), "[42]");
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// assert_eq!(format!("{}", list), "[1, 2, 3]");
		/// ```
		fn fmt(
			&self,
			f: &mut fmt::Formatter<'_>,
		) -> fmt::Result {
			f.write_str("[")?;
			let mut first = true;
			for item in self.iter() {
				if first {
					first = false;
				} else {
					f.write_str(", ")?;
				}
				fmt::Display::fmt(item, f)?;
			}
			f.write_str("]")
		}
	}

	// --- Drop ---

	#[document_type_parameters("The type of the elements in the list.")]
	#[document_parameters("The list to drop.")]
	impl<A> Drop for CatList<A> {
		#[document_signature]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::cat_list::CatList;
		/// {
		/// 	let _list = CatList::singleton(1).snoc(2).snoc(3);
		/// } // drop called here
		/// assert!(true);
		/// ```
		fn drop(&mut self) {
			let mut worklist: Vec<VecDeque<CatList<A>>> = Vec::new();

			// Take the current node's children, if any.
			if let CatListInner::Cons(_, deque, _) = &mut self.0
				&& !deque.is_empty()
			{
				worklist.push(std::mem::take(deque));
			}

			while let Some(mut deque) = worklist.pop() {
				for mut child in deque.drain(..) {
					if let CatListInner::Cons(_, inner_deque, _) = &mut child.0
						&& !inner_deque.is_empty()
					{
						worklist.push(std::mem::take(inner_deque));
					}
					// child is now Cons(a, empty_deque, _) or Nil,
					// which drops without recursion.
				}
			}
		}
	}
	// -- By-reference trait implementations --

	impl RefFunctor for CatListBrand {
		/// Maps a function over the list by reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the list.",
			"The type of the elements in the resulting list."
		)]
		///
		#[document_parameters(
			"The function to apply to each element reference.",
			"The list to map over."
		)]
		///
		#[document_returns("A new list containing the results.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let result: Vec<_> =
		/// 	explicit::map::<CatListBrand, _, _, _, _>(|x: &i32| *x * 2, &list).into_iter().collect();
		/// assert_eq!(result, vec![2, 4, 6]);
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.iter().map(func).collect()
		}
	}

	impl RefFoldable for CatListBrand {
		/// Folds the list by reference using a monoid.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function wrapper.",
			"The type of the elements.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function to map each element reference to a monoid.",
			"The list to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let result =
		/// 	explicit::fold_map::<RcFnBrand, CatListBrand, _, _, _, _>(|x: &i32| x.to_string(), &list);
		/// assert_eq!(result, "123");
		/// ```
		fn ref_fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(&A) -> M + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: LiftFn + 'a,
			M: Monoid + 'a, {
			fa.iter().fold(Monoid::empty(), |acc, a| Semigroup::append(acc, func(a)))
		}
	}

	impl RefFilterable for CatListBrand {
		/// Filters and maps the list by reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input elements.",
			"The type of the output elements."
		)]
		///
		#[document_parameters("The filter-map function.", "The list to filter.")]
		///
		#[document_returns("The filtered list.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4).snoc(5);
		/// let result: Vec<_> = explicit::filter_map::<CatListBrand, _, _, _, _>(
		/// 	|x: &i32| if *x > 3 { Some(*x) } else { None },
		/// 	&list,
		/// )
		/// .into_iter()
		/// .collect();
		/// assert_eq!(result, vec![4, 5]);
		/// ```
		fn ref_filter_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> Option<B> + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.iter().filter_map(func).collect()
		}
	}

	impl RefTraversable for CatListBrand {
		/// Traverses the list by reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function wrapper.",
			"The type of the input elements.",
			"The type of the output elements.",
			"The applicative functor brand."
		)]
		///
		#[document_parameters(
			"The function to apply to each element reference.",
			"The list to traverse."
		)]
		///
		#[document_returns("The combined result in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let result: Option<CatList<String>> = ref_traverse::<CatListBrand, RcFnBrand, _, _, OptionBrand>(
		/// 	|x: &i32| Some(x.to_string()),
		/// 	&list,
		/// );
		/// let vec: Vec<_> = result.unwrap().into_iter().collect();
		/// assert_eq!(vec, vec!["1".to_string(), "2".to_string(), "3".to_string()]);
		/// ```
		fn ref_traverse<'a, FnBrand, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			ta.iter().fold(
				F::pure::<CatList<B>>(CatList::empty()),
				|acc: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, CatList<B>>), a| {
					F::lift2(|list: CatList<B>, b: B| list.snoc(b), acc, func(a))
				},
			)
		}
	}

	impl RefWitherable for CatListBrand {}

	impl RefFunctorWithIndex for CatListBrand {
		/// Maps a function with index over the list by reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input elements.",
			"The type of the output elements."
		)]
		///
		#[document_parameters("The function to apply.", "The list to map over.")]
		///
		#[document_returns("The mapped list.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(10).snoc(20).snoc(30);
		/// let result: Vec<_> = explicit::map_with_index::<CatListBrand, _, _, _, _>(
		/// 	|i, x: &i32| format!("{}:{}", i, x),
		/// 	&list,
		/// )
		/// .into_iter()
		/// .collect();
		/// assert_eq!(result, vec!["0:10", "1:20", "2:30"]);
		/// ```
		fn ref_map_with_index<'a, A: 'a, B: 'a>(
			func: impl Fn(usize, &A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.iter().enumerate().map(|(i, a)| func(i, a)).collect()
		}
	}

	impl RefFoldableWithIndex for CatListBrand {
		/// Folds the list by reference with index using a monoid.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function to map each (index, element reference) pair.",
			"The list to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(10).snoc(20).snoc(30);
		/// let result = explicit::fold_map_with_index::<RcFnBrand, CatListBrand, _, _, _, _>(
		/// 	|i, x: &i32| format!("{}:{}", i, x),
		/// 	&list,
		/// );
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		fn ref_fold_map_with_index<'a, FnBrand, A: 'a + Clone, R: Monoid + 'a>(
			func: impl Fn(usize, &A) -> R + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> R
		where
			FnBrand: LiftFn + 'a, {
			fa.iter()
				.enumerate()
				.fold(Monoid::empty(), |acc, (i, a)| Semigroup::append(acc, func(i, a)))
		}
	}

	impl RefFilterableWithIndex for CatListBrand {
		/// Filters and maps the list by reference with index.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input elements.",
			"The type of the output elements."
		)]
		///
		#[document_parameters("The filter-map function.", "The list to filter.")]
		///
		#[document_returns("The filtered list.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(10).snoc(20).snoc(30).snoc(40).snoc(50);
		/// let result: Vec<_> = explicit::filter_map_with_index::<CatListBrand, _, _, _, _>(
		/// 	|i, x: &i32| if i >= 2 { Some(*x) } else { None },
		/// 	&list,
		/// )
		/// .into_iter()
		/// .collect();
		/// assert_eq!(result, vec![30, 40, 50]);
		/// ```
		fn ref_filter_map_with_index<'a, A: 'a, B: 'a>(
			func: impl Fn(usize, &A) -> Option<B> + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.iter().enumerate().filter_map(|(i, a)| func(i, a)).collect()
		}
	}

	impl RefTraversableWithIndex for CatListBrand {
		/// Traverses the list by reference with index.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input elements.",
			"The type of the output elements.",
			"The applicative functor brand."
		)]
		///
		#[document_parameters("The function to apply.", "The list to traverse.")]
		///
		#[document_returns("The combined result in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ref_traversable_with_index::ref_traverse_with_index,
		/// 	types::*,
		/// };
		///
		/// let list = CatList::singleton(10).snoc(20).snoc(30);
		/// let result: Option<CatList<String>> = ref_traverse_with_index::<CatListBrand, _, _, OptionBrand>(
		/// 	|i, x: &i32| Some(format!("{}:{}", i, x)),
		/// 	&list,
		/// );
		/// let vec: Vec<_> = result.unwrap().into_iter().collect();
		/// assert_eq!(vec, vec!["0:10".to_string(), "1:20".to_string(), "2:30".to_string()]);
		/// ```
		fn ref_traverse_with_index<'a, A: 'a + Clone, B: 'a + Clone, M: Applicative>(
			f: impl Fn(usize, &A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			ta.iter().enumerate().fold(
				M::pure::<CatList<B>>(CatList::empty()),
				|acc: Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, CatList<B>>),
				 (i, a)| { M::lift2(|list: CatList<B>, b: B| list.snoc(b), acc, f(i, a)) },
			)
		}
	}

	// -- By-reference monadic trait implementations --

	impl RefPointed for CatListBrand {
		/// Creates a singleton `CatList` from a reference by cloning.
		#[document_signature]
		#[document_type_parameters("The lifetime of the value.", "The type of the value.")]
		#[document_parameters("The reference to the value to wrap.")]
		#[document_returns("A singleton CatList containing a clone of the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::CatList,
		/// };
		///
		/// let x = 42;
		/// let cl: CatList<i32> = ref_pure::<CatListBrand, _>(&x);
		/// assert_eq!(cl.uncons().map(|(h, _)| h), Some(42));
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			CatList::singleton(a.clone())
		}
	}

	impl RefLift for CatListBrand {
		/// Combines two `CatList` values with a by-reference binary function.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"First input type.",
			"Second input type.",
			"Output type."
		)]
		#[document_parameters("The binary function.", "The first CatList.", "The second CatList.")]
		#[document_returns("The combined CatList.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::CatList,
		/// };
		///
		/// let a: CatList<i32> = vec![1, 2].into_iter().collect();
		/// let b: CatList<i32> = vec![10, 20].into_iter().collect();
		/// let result: CatList<i32> =
		/// 	explicit::lift2::<CatListBrand, _, _, _, _, _, _>(|x: &i32, y: &i32| *x + *y, &a, &b);
		/// let v: Vec<i32> = result.into_iter().collect();
		/// assert_eq!(v, vec![11, 21, 12, 22]);
		/// ```
		fn ref_lift2<'a, A: 'a, B: 'a, C: 'a>(
			func: impl Fn(&A, &B) -> C + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			let func = &func;
			fa.iter().flat_map(|a| fb.iter().map(move |b| func(a, b))).collect()
		}
	}

	impl RefSemiapplicative for CatListBrand {
		/// Applies wrapped by-ref functions to a CatList (Cartesian product).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The function brand.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters("The CatList of by-ref functions.", "The CatList of values.")]
		#[document_returns("The CatList of results.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::CatList,
		/// };
		///
		/// let f: std::rc::Rc<dyn Fn(&i32) -> i32> = std::rc::Rc::new(|x: &i32| *x + 1);
		/// let funcs: CatList<std::rc::Rc<dyn Fn(&i32) -> i32>> = vec![f].into_iter().collect();
		/// let vals: CatList<i32> = vec![10, 20].into_iter().collect();
		/// let result: Vec<i32> =
		/// 	ref_apply::<RcFnBrand, CatListBrand, _, _>(&funcs, &vals).into_iter().collect();
		/// assert_eq!(result, vec![11, 21]);
		/// ```
		fn ref_apply<'a, FnBrand: 'a + CloneFn<Ref>, A: 'a, B: 'a>(
			ff: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ff.iter().flat_map(|f| fa.iter().map(move |a| (**f)(a))).collect()
		}
	}

	impl RefSemimonad for CatListBrand {
		/// Chains CatList computations by reference.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The input CatList.", "The function to apply by reference.")]
		#[document_returns("The flattened CatList of results.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::CatList,
		/// };
		///
		/// let cl: CatList<i32> = vec![1, 2].into_iter().collect();
		/// let result: CatList<i32> =
		/// 	explicit::bind::<CatListBrand, _, _, _, _>(&cl, |x: &i32| CatList::singleton(*x * 10));
		/// let v: Vec<i32> = result.into_iter().collect();
		/// assert_eq!(v, vec![10, 20]);
		/// ```
		fn ref_bind<'a, A: 'a, B: 'a>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.iter().flat_map(f).collect()
		}
	}

	// -- Parallel by-reference trait implementations --

	impl ParRefFunctor for CatListBrand {
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function. Must be `Send + Sync`.", "The list.")]
		#[document_returns("A new list with mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_ref_functor::ParRefFunctor,
		/// 	types::*,
		/// };
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let result = CatListBrand::par_ref_map(|x: &i32| x * 2, &list);
		/// assert_eq!(result.into_iter().collect::<Vec<_>>(), vec![2, 4, 6]);
		/// ```
		fn par_ref_map<'a, A: Send + Sync + 'a, B: Send + 'a>(
			f: impl Fn(&A) -> B + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let v: Vec<&A> = fa.iter().collect();
			#[cfg(feature = "rayon")]
			let result: Vec<B> = {
				use rayon::prelude::*;
				v.into_par_iter().map(f).collect()
			};
			#[cfg(not(feature = "rayon"))]
			let result: Vec<B> = v.into_iter().map(f).collect();
			result.into_iter().collect()
		}
	}

	impl ParRefFoldable for CatListBrand {
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The element type.", "The monoid type.")]
		#[document_parameters("The function. Must be `Send + Sync`.", "The list.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_ref_foldable::ParRefFoldable,
		/// 	types::*,
		/// };
		/// let list = CatList::singleton(1).snoc(2).snoc(3);
		/// let result = CatListBrand::par_ref_fold_map(|x: &i32| x.to_string(), &list);
		/// assert_eq!(result, "123");
		/// ```
		fn par_ref_fold_map<'a, A: Send + Sync + 'a, M: Monoid + Send + 'a>(
			f: impl Fn(&A) -> M + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M {
			let v: Vec<&A> = fa.iter().collect();
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				v.into_par_iter().map(f).reduce(Monoid::empty, Semigroup::append)
			}
			#[cfg(not(feature = "rayon"))]
			v.into_iter().map(f).fold(Monoid::empty(), |acc, m| Semigroup::append(acc, m))
		}
	}

	impl ParRefFilterable for CatListBrand {
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function. Must be `Send + Sync`.", "The list.")]
		#[document_returns("A new list with filtered and mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_ref_filterable::ParRefFilterable,
		/// 	types::*,
		/// };
		/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
		/// let result = CatListBrand::par_ref_filter_map(
		/// 	|x: &i32| if *x > 2 { Some(x.to_string()) } else { None },
		/// 	&list,
		/// );
		/// assert_eq!(result.into_iter().collect::<Vec<_>>(), vec!["3", "4"]);
		/// ```
		fn par_ref_filter_map<'a, A: Send + Sync + 'a, B: Send + 'a>(
			f: impl Fn(&A) -> Option<B> + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let v: Vec<&A> = fa.iter().collect();
			#[cfg(feature = "rayon")]
			let result: Vec<B> = {
				use rayon::prelude::*;
				v.into_par_iter().filter_map(f).collect()
			};
			#[cfg(not(feature = "rayon"))]
			let result: Vec<B> = v.into_iter().filter_map(f).collect();
			result.into_iter().collect()
		}
	}

	impl ParRefFunctorWithIndex for CatListBrand {
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function with index. Must be `Send + Sync`.", "The list.")]
		#[document_returns("A new list with mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_ref_functor_with_index::ParRefFunctorWithIndex,
		/// 	types::*,
		/// };
		/// let list = CatList::singleton(10).snoc(20);
		/// let result = CatListBrand::par_ref_map_with_index(|i, x: &i32| format!("{}:{}", i, x), &list);
		/// assert_eq!(result.into_iter().collect::<Vec<_>>(), vec!["0:10", "1:20"]);
		/// ```
		fn par_ref_map_with_index<'a, A: Send + Sync + 'a, B: Send + 'a>(
			f: impl Fn(usize, &A) -> B + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let v: Vec<&A> = fa.iter().collect();
			#[cfg(feature = "rayon")]
			let result: Vec<B> = {
				use rayon::prelude::*;
				v.into_par_iter().enumerate().map(|(i, a)| f(i, a)).collect()
			};
			#[cfg(not(feature = "rayon"))]
			let result: Vec<B> = v.into_iter().enumerate().map(|(i, a)| f(i, a)).collect();
			result.into_iter().collect()
		}
	}

	impl ParRefFoldableWithIndex for CatListBrand {
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The element type.", "The monoid type.")]
		#[document_parameters("The function with index. Must be `Send + Sync`.", "The list.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_ref_foldable_with_index::ParRefFoldableWithIndex,
		/// 	types::*,
		/// };
		/// let list = CatList::singleton(10).snoc(20);
		/// let result =
		/// 	CatListBrand::par_ref_fold_map_with_index(|i, x: &i32| format!("{}:{}", i, x), &list);
		/// assert_eq!(result, "0:101:20");
		/// ```
		fn par_ref_fold_map_with_index<'a, A: Send + Sync + 'a, M: Monoid + Send + 'a>(
			f: impl Fn(usize, &A) -> M + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M {
			let v: Vec<&A> = fa.iter().collect();
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				v.into_par_iter()
					.enumerate()
					.map(|(i, a)| f(i, a))
					.reduce(Monoid::empty, Semigroup::append)
			}
			#[cfg(not(feature = "rayon"))]
			v.into_iter()
				.enumerate()
				.map(|(i, a)| f(i, a))
				.fold(Monoid::empty(), |acc, m| Semigroup::append(acc, m))
		}
	}

	impl ParRefFilterableWithIndex for CatListBrand {
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function with index. Must be `Send + Sync`.", "The list.")]
		#[document_returns("A new list with filtered and mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CatListBrand,
		/// 	classes::par_ref_filterable_with_index::ParRefFilterableWithIndex,
		/// 	types::*,
		/// };
		/// let list = CatList::singleton(10).snoc(20).snoc(30).snoc(40).snoc(50);
		/// let result = CatListBrand::par_ref_filter_map_with_index(
		/// 	|i, x: &i32| if i % 2 == 0 { Some(x.to_string()) } else { None },
		/// 	&list,
		/// );
		/// assert_eq!(result.into_iter().collect::<Vec<_>>(), vec!["10", "30", "50"]);
		/// ```
		fn par_ref_filter_map_with_index<'a, A: Send + Sync + 'a, B: Send + 'a>(
			f: impl Fn(usize, &A) -> Option<B> + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let v: Vec<&A> = fa.iter().collect();
			#[cfg(feature = "rayon")]
			let result: Vec<B> = {
				use rayon::prelude::*;
				v.into_par_iter().enumerate().filter_map(|(i, a)| f(i, a)).collect()
			};
			#[cfg(not(feature = "rayon"))]
			let result: Vec<B> = v.into_iter().enumerate().filter_map(|(i, a)| f(i, a)).collect();
			result.into_iter().collect()
		}
	}
}
pub use inner::*;

#[cfg(test)]
#[expect(
	clippy::unwrap_used,
	clippy::indexing_slicing,
	reason = "Tests use panicking operations for brevity and clarity"
)]
mod tests {
	use super::*;

	/// Tests basic list operations: creation, emptiness check, and length.
	/// This ensures that a new list is empty and has length 0, and a singleton list is not empty and has length 1.
	#[test]
	fn test_basic_operations() {
		let list: CatList<i32> = CatList::empty();
		assert!(list.is_empty());
		assert_eq!(list.len(), 0);

		let list = CatList::singleton(1);
		assert!(!list.is_empty());
		assert_eq!(list.len(), 1);
	}

	/// Tests the concatenation of two lists.
	/// We create two lists and append them.
	/// We verify that the resulting list contains all elements in the correct order.
	#[test]
	fn test_concatenation() {
		let list1 = CatList::singleton(1).snoc(2);
		let list2 = CatList::singleton(3).snoc(4);
		let list3 = list1.append(list2);

		let vec: Vec<_> = list3.into_iter().collect();
		assert_eq!(vec, vec![1, 2, 3, 4]);
	}

	/// Tests the flattening of nested lists.
	/// We create a nested structure by appending multiple lists: ((1 ++ 2) ++ (3 ++ 4)).
	/// This exercises the `flatten_deque` logic in `uncons`.
	/// We verify that the list is flattened correctly and elements are retrieved in order.
	#[test]
	fn test_flattening() {
		// Create a nested structure: ((1 ++ 2) ++ (3 ++ 4))
		let l1 = CatList::singleton(1);
		let l2 = CatList::singleton(2);
		let l3 = CatList::singleton(3);
		let l4 = CatList::singleton(4);

		let left = l1.append(l2);
		let right = l3.append(l4);
		let combined = left.append(right);

		assert_eq!(combined.len(), 4);
		let vec: Vec<_> = combined.into_iter().collect();
		assert_eq!(vec, vec![1, 2, 3, 4]);
	}

	/// Tests the iterator implementation.
	/// We create a list from a range and collect it back into a vector.
	/// We verify that the iterator yields all elements in the correct order.
	#[test]
	fn test_iteration() {
		let list: CatList<_> = (0 .. 10).collect();
		let vec: Vec<_> = list.into_iter().collect();
		assert_eq!(vec, (0 .. 10).collect::<Vec<_>>());
	}

	/// Tests the O(1) length tracking.
	/// We create a list with 100 elements.
	/// We verify that the length is reported correctly as 100.
	#[test]
	fn test_len() {
		let list: CatList<_> = (0 .. 100).collect();
		assert_eq!(list.len(), 100);
	}

	/// Tests that cons increases length by 1.
	/// We start with an empty list and verify its length is 0.
	/// Then we prepend an element using `cons` and verify the length becomes 1.
	/// Finally, we prepend another element and verify the length becomes 2.
	#[test]
	fn test_len_cons() {
		let list = CatList::empty();
		assert_eq!(list.len(), 0);
		let list = list.cons(1);
		assert_eq!(list.len(), 1);
		let list = list.cons(2);
		assert_eq!(list.len(), 2);
	}

	/// Tests that snoc increases length by 1.
	/// We start with an empty list and verify its length is 0.
	/// Then we append an element using `snoc` and verify the length becomes 1.
	/// Finally, we append another element and verify the length becomes 2.
	#[test]
	fn test_len_snoc() {
		let list = CatList::empty();
		assert_eq!(list.len(), 0);
		let list = list.snoc(1);
		assert_eq!(list.len(), 1);
		let list = list.snoc(2);
		assert_eq!(list.len(), 2);
	}

	/// Tests that append results in sum of lengths.
	/// We create two lists: one with length 2 and another with length 3.
	/// We verify their individual lengths.
	/// Then we append the second list to the first and verify the resulting list has length 5 (2 + 3).
	#[test]
	fn test_len_append() {
		let list1 = CatList::singleton(1).snoc(2);
		let list2 = CatList::singleton(3).snoc(4).snoc(5);
		assert_eq!(list1.len(), 2);
		assert_eq!(list2.len(), 3);

		let list3 = list1.append(list2);
		assert_eq!(list3.len(), 5);
	}

	/// Tests that uncons decreases length by 1.
	/// We create a list with 3 elements and verify its length.
	/// We repeatedly call `uncons` to remove elements from the front.
	/// After each `uncons`, we verify that the length of the remaining tail decreases by 1,
	/// until the list is empty (length 0).
	#[test]
	fn test_len_uncons() {
		let list = CatList::singleton(1).snoc(2).snoc(3);
		assert_eq!(list.len(), 3);

		let (_, tail) = list.uncons().unwrap();
		assert_eq!(tail.len(), 2);

		let (_, tail) = tail.uncons().unwrap();
		assert_eq!(tail.len(), 1);

		let (_, tail) = tail.uncons().unwrap();
		assert_eq!(tail.len(), 0);
	}

	/// Tests appending empty lists.
	/// We verify that appending an empty list to a non-empty list (and vice versa)
	/// preserves the non-empty list's content and length.
	/// We also verify that appending two empty lists results in an empty list.
	#[test]
	fn test_append_empty() {
		let empty: CatList<i32> = CatList::empty();
		let list = CatList::singleton(1);

		// empty ++ list
		let res = empty.clone().append(list.clone());
		assert_eq!(res.len(), 1);
		assert_eq!(res.into_iter().collect::<Vec<_>>(), vec![1]);

		// list ++ empty
		let res = list.clone().append(empty.clone());
		assert_eq!(res.len(), 1);
		assert_eq!(res.into_iter().collect::<Vec<_>>(), vec![1]);

		// empty ++ empty
		let res = empty.clone().append(empty);
		assert_eq!(res.len(), 0);
		assert!(res.is_empty());
	}

	/// Tests uncons edge cases.
	/// We verify that uncons on an empty list returns None.
	/// We verify that uncons on a singleton list returns the element and an empty tail.
	#[test]
	fn test_uncons_edge_cases() {
		let empty: CatList<i32> = CatList::empty();
		assert!(empty.uncons().is_none());

		let list = CatList::singleton(1);
		let (head, tail) = list.uncons().unwrap();
		assert_eq!(head, 1);
		assert!(tail.is_empty());
		assert_eq!(tail.len(), 0);
	}

	use {
		crate::{
			brands::*,
			classes::*,
			functions::*,
		},
		quickcheck_macros::quickcheck,
	};

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(x: Vec<i32>) -> bool {
		let x: CatList<_> = x.into_iter().collect();
		explicit::map::<CatListBrand, _, _, _, _>(identity, x.clone()) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(x: Vec<i32>) -> bool {
		let x: CatList<_> = x.into_iter().collect();
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		explicit::map::<CatListBrand, _, _, _, _>(compose(f, g), x.clone())
			== explicit::map::<CatListBrand, _, _, _, _>(
				f,
				explicit::map::<CatListBrand, _, _, _, _>(g, x),
			)
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(v: Vec<i32>) -> bool {
		let v: CatList<_> = v.into_iter().collect();
		apply::<RcFnBrand, CatListBrand, _, _>(
			pure::<CatListBrand, _>(<RcFnBrand as LiftFn>::new(identity)),
			v.clone(),
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, CatListBrand, _, _>(
			pure::<CatListBrand, _>(<RcFnBrand as LiftFn>::new(f)),
			pure::<CatListBrand, _>(x),
		) == pure::<CatListBrand, _>(f(x))
	}

	// Semigroup Laws

	/// Tests the associativity law for Semigroup.
	#[quickcheck]
	fn semigroup_associativity(
		a: Vec<i32>,
		b: Vec<i32>,
		c: Vec<i32>,
	) -> bool {
		let a: CatList<_> = a.into_iter().collect();
		let b: CatList<_> = b.into_iter().collect();
		let c: CatList<_> = c.into_iter().collect();
		append(a.clone(), append(b.clone(), c.clone())) == append(append(a, b), c)
	}

	// Monoid Laws

	/// Tests the left identity law for Monoid.
	#[quickcheck]
	fn monoid_left_identity(a: Vec<i32>) -> bool {
		let a: CatList<_> = a.into_iter().collect();
		append(empty::<CatList<i32>>(), a.clone()) == a
	}

	/// Tests the right identity law for Monoid.
	#[quickcheck]
	fn monoid_right_identity(a: Vec<i32>) -> bool {
		let a: CatList<_> = a.into_iter().collect();
		append(a.clone(), empty::<CatList<i32>>()) == a
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| CatList::singleton(x.wrapping_mul(2));
		explicit::bind::<CatListBrand, _, _, _, _>(pure::<CatListBrand, _>(a), f) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(m: Vec<i32>) -> bool {
		let m: CatList<_> = m.into_iter().collect();
		explicit::bind::<CatListBrand, _, _, _, _>(m.clone(), pure::<CatListBrand, _>) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(m: Vec<i32>) -> bool {
		let m: CatList<_> = m.into_iter().collect();
		let f = |x: i32| CatList::singleton(x.wrapping_mul(2));
		let g = |x: i32| CatList::singleton(x.wrapping_add(1));
		explicit::bind::<CatListBrand, _, _, _, _>(
			explicit::bind::<CatListBrand, _, _, _, _>(m.clone(), f),
			g,
		) == explicit::bind::<CatListBrand, _, _, _, _>(m, |x| {
			explicit::bind::<CatListBrand, _, _, _, _>(f(x), g)
		})
	}

	// Edge Cases

	/// Tests `map` on an empty list.
	#[test]
	fn map_empty() {
		assert_eq!(
			explicit::map::<CatListBrand, _, _, _, _>(
				|x: i32| x + 1,
				CatList::empty() as CatList<i32>
			),
			CatList::empty() as CatList<i32>
		);
	}

	/// Tests `bind` on an empty list.
	#[test]
	fn bind_empty() {
		assert_eq!(
			explicit::bind::<CatListBrand, _, _, _, _>(
				CatList::empty() as CatList<i32>,
				|x: i32| { CatList::singleton(x + 1) }
			),
			CatList::empty() as CatList<i32>
		);
	}

	/// Tests `bind` returning an empty list.
	#[test]
	fn bind_returning_empty() {
		let list: CatList<_> = vec![1, 2, 3].into_iter().collect();
		assert_eq!(
			explicit::bind::<CatListBrand, _, _, _, _>(list, |_| CatList::empty() as CatList<i32>),
			CatList::empty() as CatList<i32>
		);
	}

	/// Tests `fold_right` on an empty list.
	#[test]
	fn fold_right_empty() {
		assert_eq!(
			crate::functions::explicit::fold_right::<RcFnBrand, CatListBrand, _, _, _, _>(
				|x: i32, acc| x + acc,
				0,
				CatList::empty()
			),
			0
		);
	}

	/// Tests `fold_left` on an empty list.
	#[test]
	fn fold_left_empty() {
		assert_eq!(
			crate::functions::explicit::fold_left::<RcFnBrand, CatListBrand, _, _, _, _>(
				|acc, x: i32| acc + x,
				0,
				CatList::empty()
			),
			0
		);
	}

	/// Tests `traverse` on an empty list.
	#[test]
	fn traverse_empty() {
		use crate::brands::OptionBrand;
		assert_eq!(
			crate::classes::traversable::traverse::<CatListBrand, _, _, OptionBrand>(
				|x: i32| Some(x + 1),
				CatList::empty()
			),
			Some(CatList::empty())
		);
	}

	// Parallel Trait Tests

	/// Tests `par_map` on a list.
	#[test]
	fn par_map_basic() {
		let v: CatList<_> = vec![1, 2, 3].into_iter().collect();
		let result: CatList<i32> = par_map::<CatListBrand, _, _>(|x: i32| x * 2, v);
		assert_eq!(result.into_iter().collect::<Vec<_>>(), vec![2, 4, 6]);
	}

	/// Tests `par_filter` on a list.
	#[test]
	fn par_filter_basic() {
		let v: CatList<_> = vec![1, 2, 3, 4, 5].into_iter().collect();
		let result: CatList<i32> = par_filter::<CatListBrand, _>(|x: &i32| x % 2 == 0, v);
		assert_eq!(result.into_iter().collect::<Vec<_>>(), vec![2, 4]);
	}

	/// Tests `par_filter_map` on a list.
	#[test]
	fn par_filter_map_basic() {
		let v: CatList<_> = vec![1, 2, 3, 4, 5].into_iter().collect();
		let result: CatList<i32> = par_filter_map::<CatListBrand, _, _>(
			|x: i32| if x % 2 == 0 { Some(x * 10) } else { None },
			v,
		);
		assert_eq!(result.into_iter().collect::<Vec<_>>(), vec![20, 40]);
	}

	/// Tests `par_compact` on a list of options.
	#[test]
	fn par_compact_basic() {
		let v: CatList<_> = vec![Some(1), None, Some(3)].into_iter().collect();
		let result: CatList<i32> = par_compact::<CatListBrand, _>(v);
		assert_eq!(result.into_iter().collect::<Vec<_>>(), vec![1, 3]);
	}

	/// Tests `par_separate` on a list of results.
	#[test]
	fn par_separate_basic() {
		let v: CatList<Result<i32, &str>> = vec![Ok(1), Err("e"), Ok(3)].into_iter().collect();
		let (errs, oks): (CatList<&str>, CatList<i32>) = par_separate::<CatListBrand, _, _>(v);
		assert_eq!(errs.into_iter().collect::<Vec<_>>(), vec!["e"]);
		assert_eq!(oks.into_iter().collect::<Vec<_>>(), vec![1, 3]);
	}

	/// Tests `par_fold_map` on an empty list.
	#[test]
	fn par_fold_map_empty() {
		let v: CatList<i32> = CatList::empty();
		assert_eq!(par_fold_map::<CatListBrand, _, _>(|x: i32| x.to_string(), v), "".to_string());
	}

	/// Tests `par_fold_map` on multiple elements.
	#[test]
	fn par_fold_map_multiple() {
		let v: CatList<_> = vec![1, 2, 3].into_iter().collect();
		assert_eq!(
			par_fold_map::<CatListBrand, _, _>(|x: i32| x.to_string(), v),
			"123".to_string()
		);
	}

	/// Tests `par_map_with_index` on a list.
	#[test]
	fn par_map_with_index_basic() {
		let v: CatList<_> = vec![10, 20, 30].into_iter().collect();
		let result: CatList<i32> =
			par_map_with_index::<CatListBrand, _, _>(|i, x: i32| x + i as i32, v);
		assert_eq!(result.into_iter().collect::<Vec<_>>(), vec![10, 21, 32]);
	}

	/// Tests `par_fold_map_with_index` on a list.
	#[test]
	fn par_fold_map_with_index_basic() {
		let v: CatList<_> = vec![10, 20, 30].into_iter().collect();
		let result: String =
			par_fold_map_with_index::<CatListBrand, _, _>(|i, x: i32| format!("{i}:{x}"), v);
		assert_eq!(result, "0:101:202:30");
	}

	// Parallel Trait Laws

	/// Property: `par_map` agrees with sequential `map`.
	#[quickcheck]
	fn prop_par_map_equals_map(xs: Vec<i32>) -> bool {
		let xs: CatList<_> = xs.into_iter().collect();
		let f = |x: i32| x.wrapping_add(1);
		let seq_res: CatList<_> = explicit::map::<CatListBrand, _, _, _, _>(f, xs.clone());
		let par_res: CatList<_> = par_map::<CatListBrand, _, _>(f, xs);
		seq_res == par_res
	}

	/// Property: `par_fold_map` agrees with sequential `fold_map`.
	#[quickcheck]
	fn prop_par_fold_map_equals_fold_map(xs: Vec<i32>) -> bool {
		use crate::types::Additive;

		let xs: CatList<_> = xs.into_iter().collect();
		let f = |x: i32| Additive(x as i64);
		let seq_res = crate::functions::explicit::fold_map::<
			crate::brands::RcFnBrand,
			CatListBrand,
			_,
			_,
			_,
			_,
		>(f, xs.clone());
		let par_res = par_fold_map::<CatListBrand, _, _>(f, xs);
		seq_res == par_res
	}

	// Filterable Laws

	/// Tests `filterMap identity ≡ compact`.
	#[quickcheck]
	fn filterable_filter_map_identity(x: Vec<Option<i32>>) -> bool {
		let x: CatList<_> = x.into_iter().collect();
		explicit::filter_map::<CatListBrand, _, _, _, _>(identity, x.clone())
			== explicit::compact::<CatListBrand, _, _, _>(x)
	}

	/// Tests `filterMap Just ≡ identity`.
	#[quickcheck]
	fn filterable_filter_map_just(x: Vec<i32>) -> bool {
		let x: CatList<_> = x.into_iter().collect();
		explicit::filter_map::<CatListBrand, _, _, _, _>(Some, x.clone()) == x
	}

	/// Tests `partitionMap identity ≡ separate`.
	#[quickcheck]
	fn filterable_partition_map_identity(x: Vec<Result<i32, i32>>) -> bool {
		let x: CatList<_> = x.into_iter().collect();
		explicit::partition_map::<CatListBrand, _, _, _, _, _>(identity, x.clone())
			== explicit::separate::<CatListBrand, _, _, _, _>(x)
	}

	// Witherable Laws

	/// Tests `wither (pure <<< Just) ≡ pure`.
	#[quickcheck]
	fn witherable_identity(x: Vec<i32>) -> bool {
		let x: CatList<_> = x.into_iter().collect();
		explicit::wither::<RcFnBrand, CatListBrand, OptionBrand, _, _, _, _>(
			|i| Some(Some(i)),
			x.clone(),
		) == Some(x)
	}

	// Alt Laws

	/// Tests the associativity law for Alt.
	#[quickcheck]
	fn alt_associativity(
		x: Vec<i32>,
		y: Vec<i32>,
		z: Vec<i32>,
	) -> bool {
		let cx: CatList<_> = x.into_iter().collect();
		let cy: CatList<_> = y.into_iter().collect();
		let cz: CatList<_> = z.into_iter().collect();
		let lhs: Vec<_> = explicit::alt::<CatListBrand, _, _, _>(
			explicit::alt::<CatListBrand, _, _, _>(cx.clone(), cy.clone()),
			cz.clone(),
		)
		.into_iter()
		.collect();
		let rhs: Vec<_> = explicit::alt::<CatListBrand, _, _, _>(
			cx,
			explicit::alt::<CatListBrand, _, _, _>(cy, cz),
		)
		.into_iter()
		.collect();
		lhs == rhs
	}

	/// Tests the distributivity law for Alt.
	#[quickcheck]
	fn alt_distributivity(
		x: Vec<i32>,
		y: Vec<i32>,
	) -> bool {
		let cx: CatList<_> = x.into_iter().collect();
		let cy: CatList<_> = y.into_iter().collect();
		let f = |i: i32| i.wrapping_mul(2).wrapping_add(1);
		let lhs: Vec<_> = explicit::map::<CatListBrand, _, _, _, _>(
			f,
			explicit::alt::<CatListBrand, _, _, _>(cx.clone(), cy.clone()),
		)
		.into_iter()
		.collect();
		let rhs: Vec<_> = explicit::alt::<CatListBrand, _, _, _>(
			explicit::map::<CatListBrand, _, _, _, _>(f, cx),
			explicit::map::<CatListBrand, _, _, _, _>(f, cy),
		)
		.into_iter()
		.collect();
		lhs == rhs
	}

	// Plus Laws

	/// Tests the left identity law for Plus.
	#[quickcheck]
	fn plus_left_identity(x: Vec<i32>) -> bool {
		let cx: CatList<_> = x.into_iter().collect();
		let lhs: Vec<_> =
			explicit::alt::<CatListBrand, _, _, _>(plus_empty::<CatListBrand, i32>(), cx.clone())
				.into_iter()
				.collect();
		let rhs: Vec<_> = cx.into_iter().collect();
		lhs == rhs
	}

	/// Tests the right identity law for Plus.
	#[quickcheck]
	fn plus_right_identity(x: Vec<i32>) -> bool {
		let cx: CatList<_> = x.into_iter().collect();
		let lhs: Vec<_> =
			explicit::alt::<CatListBrand, _, _, _>(cx.clone(), plus_empty::<CatListBrand, i32>())
				.into_iter()
				.collect();
		let rhs: Vec<_> = cx.into_iter().collect();
		lhs == rhs
	}

	/// Tests the annihilation law for Plus.
	#[test]
	fn plus_annihilation() {
		let f = |i: i32| i.wrapping_mul(2);
		let lhs: Vec<_> =
			explicit::map::<CatListBrand, _, _, _, _>(f, plus_empty::<CatListBrand, i32>())
				.into_iter()
				.collect();
		let rhs: Vec<_> = plus_empty::<CatListBrand, i32>().into_iter().collect();
		assert_eq!(lhs, rhs);
	}

	// Compactable Laws (Plus-dependent)

	/// Tests the functor identity law for Compactable.
	#[quickcheck]
	fn compactable_functor_identity(x: Vec<i32>) -> bool {
		let cx: CatList<_> = x.into_iter().collect();
		let lhs: Vec<_> =
			explicit::compact::<CatListBrand, _, _, _>(explicit::map::<CatListBrand, _, _, _, _>(
				Some,
				cx.clone(),
			))
			.into_iter()
			.collect();
		let rhs: Vec<_> = cx.into_iter().collect();
		lhs == rhs
	}

	// Data Structure Property Tests

	/// Property: cons adds element to the front.
	#[quickcheck]
	fn prop_cons_adds_to_front(
		head: i32,
		tail: Vec<i32>,
	) -> bool {
		let list: CatList<_> = tail.iter().cloned().collect();
		let list = list.cons(head);

		let mut expected = vec![head];
		expected.extend(tail);

		let result: Vec<_> = list.into_iter().collect();
		result == expected
	}

	/// Property: snoc adds element to the back.
	#[quickcheck]
	fn prop_snoc_adds_to_back(
		init: Vec<i32>,
		last: i32,
	) -> bool {
		let list: CatList<_> = init.iter().cloned().collect();
		let list = list.snoc(last);

		let mut expected = init;
		expected.push(last);

		let result: Vec<_> = list.into_iter().collect();
		result == expected
	}

	/// Property: append concatenates two lists.
	#[quickcheck]
	fn prop_append_concatenates(
		xs: Vec<i32>,
		ys: Vec<i32>,
	) -> bool {
		let list1: CatList<_> = xs.iter().cloned().collect();
		let list2: CatList<_> = ys.iter().cloned().collect();
		let list3 = list1.append(list2);

		let mut expected = xs;
		expected.extend(ys);

		let result: Vec<_> = list3.into_iter().collect();
		result == expected
	}

	/// Property: append is associative.
	#[quickcheck]
	fn prop_append_associative(
		xs: Vec<i32>,
		ys: Vec<i32>,
		zs: Vec<i32>,
	) -> bool {
		let l1: CatList<_> = xs.iter().cloned().collect();
		let l2: CatList<_> = ys.iter().cloned().collect();
		let l3: CatList<_> = zs.iter().cloned().collect();

		let left = l1.clone().append(l2.clone()).append(l3.clone());
		let right = l1.append(l2.append(l3));

		let r1: Vec<_> = left.into_iter().collect();
		let r2: Vec<_> = right.into_iter().collect();
		r1 == r2
	}

	/// Property: append with empty is identity.
	#[quickcheck]
	fn prop_append_identity(xs: Vec<i32>) -> bool {
		let list: CatList<_> = xs.iter().cloned().collect();
		let empty = CatList::empty();

		let left = empty.clone().append(list.clone());
		let right = list.clone().append(empty);

		let r1: Vec<_> = left.into_iter().collect();
		let r2: Vec<_> = right.into_iter().collect();

		r1 == xs && r2 == xs
	}

	/// Property: len returns correct length.
	#[quickcheck]
	fn prop_len_correct(xs: Vec<i32>) -> bool {
		let list: CatList<_> = xs.iter().cloned().collect();
		list.len() == xs.len()
	}

	/// Property: is_empty is true iff length is 0.
	#[quickcheck]
	fn prop_is_empty_correct(xs: Vec<i32>) -> bool {
		let list: CatList<_> = xs.iter().cloned().collect();
		list.is_empty() == xs.is_empty()
	}

	/// Property: uncons returns head and tail.
	#[quickcheck]
	fn prop_uncons_correct(xs: Vec<i32>) -> bool {
		let list: CatList<_> = xs.iter().cloned().collect();

		match list.uncons() {
			None => xs.is_empty(),
			Some((head, tail)) =>
				if xs.is_empty() {
					false
				} else {
					let expected_head = xs[0];
					let expected_tail = &xs[1 ..];
					let tail_vec: Vec<_> = tail.into_iter().collect();

					head == expected_head && tail_vec == expected_tail
				},
		}
	}

	/// Property: cons increases length by 1.
	#[quickcheck]
	fn prop_cons_increases_len(
		head: i32,
		tail: Vec<i32>,
	) -> bool {
		let list: CatList<_> = tail.iter().cloned().collect();
		let initial_len = list.len();
		let list = list.cons(head);
		list.len() == initial_len + 1
	}

	/// Property: snoc increases length by 1.
	#[quickcheck]
	fn prop_snoc_increases_len(
		init: Vec<i32>,
		last: i32,
	) -> bool {
		let list: CatList<_> = init.iter().cloned().collect();
		let initial_len = list.len();
		let list = list.snoc(last);
		list.len() == initial_len + 1
	}

	/// Property: append sums lengths.
	#[quickcheck]
	fn prop_append_sums_len(
		xs: Vec<i32>,
		ys: Vec<i32>,
	) -> bool {
		let list1: CatList<_> = xs.iter().cloned().collect();
		let list2: CatList<_> = ys.iter().cloned().collect();
		let len1 = list1.len();
		let len2 = list2.len();

		let list3 = list1.append(list2);
		list3.len() == len1 + len2
	}

	/// Property: uncons decreases length by 1.
	#[quickcheck]
	fn prop_uncons_decreases_len(xs: Vec<i32>) -> bool {
		let list: CatList<_> = xs.iter().cloned().collect();
		let initial_len = list.len();

		match list.uncons() {
			None => initial_len == 0,
			Some((_, tail)) => tail.len() == initial_len - 1,
		}
	}

	/// Property: cons equivalence with LinkedList.
	#[quickcheck]
	fn prop_cons_equivalence_linked_list(xs: Vec<i32>) -> bool {
		use std::collections::LinkedList;

		let mut cat_list = CatList::empty();
		let mut linked_list = LinkedList::new();

		for &x in &xs {
			cat_list = cat_list.cons(x);
			linked_list.push_front(x);
		}

		let cat_vec: Vec<_> = cat_list.into_iter().collect();
		let linked_vec: Vec<_> = linked_list.into_iter().collect();

		cat_vec == linked_vec
	}

	/// Property: uncons equivalence with LinkedList.
	#[quickcheck]
	fn prop_uncons_equivalence_linked_list(xs: Vec<i32>) -> bool {
		use std::collections::LinkedList;

		let mut cat_list: CatList<_> = xs.iter().cloned().collect();
		let mut linked_list: LinkedList<_> = xs.iter().cloned().collect();

		loop {
			let cat_res = cat_list.uncons();
			let linked_res = linked_list.pop_front();

			match (cat_res, linked_res) {
				(Some((a, tail)), Some(b)) => {
					if a != b {
						return false;
					}
					cat_list = tail;
				}
				(None, None) => return true,
				_ => return false,
			}
		}
	}

	/// Tests that the borrowing iterator yields the same elements as the consuming iterator.
	#[test]
	fn test_borrowing_iter_matches_consuming_iter() {
		let list = CatList::singleton(1).snoc(2).snoc(3).append(CatList::singleton(4).snoc(5));
		let borrowed: Vec<_> = list.iter().collect();
		let owned: Vec<_> = list.clone().into_iter().collect();
		assert_eq!(borrowed.len(), owned.len());
		for (b, o) in borrowed.iter().zip(owned.iter()) {
			assert_eq!(*b, o);
		}
	}

	/// Tests the borrowing iterator on an empty list.
	#[test]
	fn test_borrowing_iter_empty() {
		let list: CatList<i32> = CatList::empty();
		let borrowed: Vec<_> = list.iter().collect();
		assert!(borrowed.is_empty());
	}

	/// Tests the borrowing iterator on a singleton list.
	#[test]
	fn test_borrowing_iter_singleton() {
		let list = CatList::singleton(42);
		let borrowed: Vec<_> = list.iter().collect();
		assert_eq!(borrowed, vec![&42]);
	}

	/// Tests ExactSizeIterator for the consuming iterator.
	#[test]
	fn test_consuming_iter_exact_size() {
		let list: CatList<_> = (0 .. 5).collect();
		let iter = list.into_iter();
		assert_eq!(iter.len(), 5);
	}

	/// Tests ExactSizeIterator for the borrowing iterator.
	#[test]
	fn test_borrowing_iter_exact_size() {
		let list: CatList<_> = (0 .. 5).collect();
		let iter = list.iter();
		assert_eq!(iter.len(), 5);
	}

	/// Property: borrowing iterator produces same elements as consuming iterator.
	#[quickcheck]
	fn prop_borrowing_iter_matches_consuming(xs: Vec<i32>) -> bool {
		let list: CatList<_> = xs.into_iter().collect();
		let borrowed: Vec<_> = list.iter().copied().collect();
		let owned: Vec<_> = list.into_iter().collect();
		borrowed == owned
	}

	/// Property: borrowing iterator size_hint is exact.
	#[quickcheck]
	fn prop_borrowing_iter_size_hint(xs: Vec<i32>) -> bool {
		let list: CatList<_> = xs.into_iter().collect();
		let iter = list.iter();
		let (lo, hi) = iter.size_hint();
		lo == list.len() && hi == Some(list.len())
	}

	/// Tests that dropping a deeply nested `CatList` does not overflow the stack.
	///
	/// A right-associated chain of appends creates deep nesting in the sublist
	/// deques. Without the iterative `Drop` implementation, this would cause a
	/// stack overflow from recursive destructor calls.
	#[test]
	fn test_deep_drop_does_not_overflow_stack() {
		let depth = 100_000;
		let mut list = CatList::singleton(0);
		for i in 1 .. depth {
			// Right-associated: each append nests the accumulator inside a new node's deque.
			list = CatList::singleton(i).append(list);
		}
		assert_eq!(list.len(), depth);
		// Dropping `list` here exercises the iterative Drop implementation.
		drop(list);
	}

	// MonadRec tests

	/// Tests the MonadRec identity law: `tail_rec_m(|a| pure(Done(a)), x) == pure(x)`.
	#[quickcheck]
	fn monad_rec_identity(x: i32) -> bool {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let result: Vec<_> =
			tail_rec_m::<CatListBrand, _, _>(|a| CatList::singleton(ControlFlow::Break(a)), x)
				.into_iter()
				.collect();
		let expected: Vec<_> = CatList::singleton(x).into_iter().collect();
		result == expected
	}

	/// Tests a recursive computation that sums a range via `tail_rec_m`.
	#[test]
	fn monad_rec_sum_range() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		// Sum numbers from 1 to 100: tail_rec_m with accumulator
		let result = tail_rec_m::<CatListBrand, _, _>(
			|(n, acc)| {
				if n > 100 {
					CatList::singleton(ControlFlow::Break(acc))
				} else {
					CatList::singleton(ControlFlow::Continue((n + 1, acc + n)))
				}
			},
			(1i64, 0i64),
		);
		let vec: Vec<_> = result.into_iter().collect();
		assert_eq!(vec, vec![5050]);
	}

	/// Tests that `tail_rec_m` is stack-safe with 200,000 iterations.
	#[test]
	fn monad_rec_stack_safety() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let iterations = 200_000i64;
		let result = tail_rec_m::<CatListBrand, _, _>(
			|n| {
				if n >= iterations {
					CatList::singleton(ControlFlow::Break(n))
				} else {
					CatList::singleton(ControlFlow::Continue(n + 1))
				}
			},
			0i64,
		);
		let vec: Vec<_> = result.into_iter().collect();
		assert_eq!(vec, vec![iterations]);
	}

	// --- Display tests ---

	/// Tests `Display` for an empty list.
	#[test]
	fn test_cat_list_display_empty() {
		let list: CatList<i32> = CatList::empty();
		assert_eq!(format!("{}", list), "[]");
	}

	/// Tests `Display` for a singleton list.
	#[test]
	fn test_cat_list_display_singleton() {
		let list = CatList::singleton(42);
		assert_eq!(format!("{}", list), "[42]");
	}

	/// Tests `Display` for a multi-element list.
	#[test]
	fn test_cat_list_display_multiple() {
		let list = CatList::singleton(1).snoc(2).snoc(3);
		assert_eq!(format!("{}", list), "[1, 2, 3]");
	}

	/// Tests `Display` for a list built by appending two lists.
	#[test]
	fn test_cat_list_display_appended() {
		let left = CatList::singleton(1).snoc(2);
		let right = CatList::singleton(3).snoc(4);
		let list = left.append(right);
		assert_eq!(format!("{}", list), "[1, 2, 3, 4]");
	}

	/// Tests `Display` for a list of strings.
	#[test]
	fn test_cat_list_display_strings() {
		let list = CatList::singleton("hello".to_string()).snoc("world".to_string());
		assert_eq!(format!("{}", list), "[hello, world]");
	}

	// --- Extend tests ---

	/// Tests basic `extend` on `CatList`: sum of suffixes.
	#[test]
	fn extend_sum_of_suffixes() {
		use crate::classes::extend::extend;
		let list = CatList::singleton(1).snoc(2).snoc(3);
		let result =
			extend::<CatListBrand, _, _>(|cl: CatList<i32>| cl.into_iter().sum::<i32>(), list);
		let vec: Vec<_> = result.into_iter().collect();
		assert_eq!(vec, vec![6, 5, 3]);
	}

	/// Extend associativity: `extend(f, extend(g, w)) == extend(|w| f(extend(g, w)), w)`.
	#[test]
	fn extend_associativity() {
		use crate::classes::extend::extend;
		let g = |cl: CatList<i32>| cl.into_iter().map(|x| x * 2).sum::<i32>();
		let f = |cl: CatList<i32>| cl.into_iter().map(|x| x + 1).sum::<i32>();
		let w = CatList::singleton(1).snoc(2).snoc(3);
		let lhs = extend::<CatListBrand, _, _>(f, extend::<CatListBrand, _, _>(g, w.clone()));
		let rhs = extend::<CatListBrand, _, _>(
			|w: CatList<i32>| f(extend::<CatListBrand, _, _>(g, w)),
			w,
		);
		let lhs_vec: Vec<_> = lhs.into_iter().collect();
		let rhs_vec: Vec<_> = rhs.into_iter().collect();
		assert_eq!(lhs_vec, rhs_vec);
	}

	/// Tests that `duplicate` produces suffixes.
	#[test]
	fn extend_duplicate_suffixes() {
		use crate::classes::extend::duplicate;
		let list = CatList::singleton(1).snoc(2).snoc(3);
		let result = duplicate::<CatListBrand, _>(list);
		let vecs: Vec<Vec<_>> = result.into_iter().map(|cl| cl.into_iter().collect()).collect();
		assert_eq!(vecs, vec![vec![1, 2, 3], vec![2, 3], vec![3]]);
	}

	/// Tests `extend` on an empty list.
	#[test]
	fn extend_empty() {
		use crate::classes::extend::extend;
		let result = extend::<CatListBrand, _, _>(
			|cl: CatList<i32>| cl.into_iter().sum::<i32>(),
			CatList::empty(),
		);
		assert!(result.is_empty());
	}

	/// Tests `extend` on a singleton list.
	#[test]
	fn extend_singleton() {
		use crate::classes::extend::extend;
		let result = extend::<CatListBrand, _, _>(
			|cl: CatList<i32>| cl.into_iter().sum::<i32>(),
			CatList::singleton(42),
		);
		let vec: Vec<_> = result.into_iter().collect();
		assert_eq!(vec, vec![42]);
	}

	// -- Ref trait property tests --

	#[quickcheck]
	fn ref_functor_identity(v: Vec<i32>) -> bool {
		let list: CatList<i32> = v.iter().copied().collect();
		let mapped: Vec<i32> = CatListBrand::ref_map(|x: &i32| *x, &list).into_iter().collect();
		mapped == v
	}

	#[quickcheck]
	fn ref_functor_composition(v: Vec<i32>) -> bool {
		let f = |x: &i32| x.wrapping_add(1);
		let g = |x: &i32| x.wrapping_mul(2);
		let list1: CatList<i32> = v.iter().copied().collect();
		let list2: CatList<i32> = v.iter().copied().collect();
		let composed: Vec<i32> =
			CatListBrand::ref_map(|x: &i32| g(&f(x)), &list1).into_iter().collect();
		let sequential: Vec<i32> =
			CatListBrand::ref_map(g, &CatListBrand::ref_map(f, &list2)).into_iter().collect();
		composed == sequential
	}

	#[quickcheck]
	fn ref_foldable_additive(v: Vec<i32>) -> bool {
		use crate::types::Additive;
		let list: CatList<i32> = v.iter().copied().collect();
		let result = CatListBrand::ref_fold_map::<RcFnBrand, _, _>(|x: &i32| Additive(*x), &list);
		let expected = v.iter().copied().fold(0i32, |a, b| a.wrapping_add(b));
		result.0 == expected
	}

	#[quickcheck]
	fn ref_semimonad_left_identity(x: i32) -> bool {
		let result: Vec<i32> =
			CatListBrand::ref_bind(&CatList::singleton(x), |a: &i32| CatList::singleton(*a))
				.into_iter()
				.collect();
		result == vec![x]
	}

	#[quickcheck]
	fn par_ref_functor_equivalence(v: Vec<i32>) -> bool {
		let f = |x: &i32| x.wrapping_add(1);
		let list1: CatList<i32> = v.iter().copied().collect();
		let list2: CatList<i32> = v.iter().copied().collect();
		let par_result: Vec<i32> = CatListBrand::par_ref_map(f, &list1).into_iter().collect();
		let seq_result: Vec<i32> = CatListBrand::ref_map(f, &list2).into_iter().collect();
		par_result == seq_result
	}

	// RefAlt Laws

	/// RefAlt associativity
	#[quickcheck]
	fn ref_alt_associativity(
		xv: Vec<i32>,
		yv: Vec<i32>,
		zv: Vec<i32>,
	) -> bool {
		let x: CatList<i32> = xv.into_iter().collect();
		let y: CatList<i32> = yv.into_iter().collect();
		let z: CatList<i32> = zv.into_iter().collect();
		let lhs: Vec<i32> = explicit::alt::<CatListBrand, _, _, _>(
			&explicit::alt::<CatListBrand, _, _, _>(&x, &y),
			&z,
		)
		.into_iter()
		.collect();
		let rhs: Vec<i32> = explicit::alt::<CatListBrand, _, _, _>(
			&x,
			&explicit::alt::<CatListBrand, _, _, _>(&y, &z),
		)
		.into_iter()
		.collect();
		lhs == rhs
	}
}
