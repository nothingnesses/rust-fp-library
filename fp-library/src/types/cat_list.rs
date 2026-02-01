use fp_macros::doc_type_params;
use crate::{
	Apply,
	brands::{CatListBrand, OptionBrand},
	classes::{
		applicative::Applicative, apply_first::ApplyFirst, apply_second::ApplySecond,
		cloneable_fn::CloneableFn, compactable::Compactable, filterable::Filterable,
		foldable::Foldable, functor::Functor, lift::Lift, monoid::Monoid,
		par_foldable::ParFoldable, pointed::Pointed, semiapplicative::Semiapplicative,
		semigroup::Semigroup, semimonad::Semimonad, send_cloneable_fn::SendCloneableFn,
		traversable::Traversable, witherable::Witherable,
	},
	impl_kind,
	kinds::*,
	types::Pair,
};
use fp_macros::hm_signature;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

/// A catenable list with O(1) append and O(1) amortized uncons.
///
/// This is the "Reflection without Remorse" data structure that enables
/// O(1) left-associated bind operations in the Free monad.
///
/// ### Performance Notes
///
/// This implementation uses a [`VecDeque`] to store sublists, providing:
///
/// * **O(1) append**: Sublists are pushed to the back of the deque.
/// * **O(1) amortized uncons**: Elements are extracted by flattening the deque.
/// * **No reversal overhead**: Unlike two-stack queue implementations, `VecDeque`
///   provides true O(1) operations on both ends without periodic reversal.
///
/// ### Type Parameters
///
/// * `A`: The type of the elements in the list.
///
/// ### Examples
///
/// ```
/// use fp_library::types::cat_list::CatList;
///
/// let list: CatList<i32> = CatList::empty();
/// ```
#[derive(Clone, Debug, Default)]
pub enum CatList<A> {
	/// Empty list
	#[default]
	Nil,
	/// Head element plus deque of sublists and total length
	Cons(A, VecDeque<CatList<A>>, usize),
}

impl<A: PartialEq + Clone> PartialEq for CatList<A> {
	fn eq(
		&self,
		other: &Self,
	) -> bool {
		if self.len() != other.len() {
			return false;
		}
		(*self).clone().into_iter().eq(other.clone())
	}
}

impl<A: Eq + Clone> Eq for CatList<A> {}

impl<A: Hash + Clone> Hash for CatList<A> {
	fn hash<H: Hasher>(
		&self,
		state: &mut H,
	) {
		self.len().hash(state);
		for a in (*self).clone() {
			a.hash(state);
		}
	}
}

impl<A: PartialOrd + Clone> PartialOrd for CatList<A> {
	fn partial_cmp(
		&self,
		other: &Self,
	) -> Option<Ordering> {
		(*self).clone().into_iter().partial_cmp((other).clone())
	}
}

impl<A: Ord + Clone> Ord for CatList<A> {
	fn cmp(
		&self,
		other: &Self,
	) -> Ordering {
		(*self).clone().into_iter().cmp((other).clone())
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
	///
	/// ### Type Signature
	///
	/// `forall a. a -> CatList a -> CatList a`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The type of the elements in the list."
	)]	///
	/// ### Parameters
	///
	/// * `head`: A value to prepend to the list.
	/// * `tail`: A list to prepend the value to.
	///
	/// ### Returns
	///
	/// A new list consisting of the `head` element prepended to the `tail` list.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::brands::CatListBrand;
	/// use fp_library::types::cat_list::CatList;
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
	///
	/// ### Type Signature
	///
	/// `forall a. CatList a -> Option (a, CatList a)`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The type of the elements in the list."
	)]	///
	/// ### Parameters
	///
	/// * `list`: The list to deconstruct.
	///
	/// ### Returns
	///
	/// An [`Option`] containing a tuple of the head element and the remaining tail list,
	/// or [`None`] if the list is empty.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::brands::CatListBrand;
	/// use fp_library::types::cat_list::CatList;
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
		A: Clone,
	{
		list.clone().uncons()
	}
}

impl Functor for CatListBrand {
	/// Maps a function over the list.
	///
	/// This method applies a function to each element of the list, producing a new list with the transformed values.
	///
	/// ### Type Signature
	///
	#[hm_signature(Functor)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements in the list.",
		"The type of the elements in the resulting list.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element.
	/// * `fa`: The list to map over.
	///
	/// ### Returns
	///
	/// A new list containing the results of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let list = CatList::singleton(1).snoc(2).snoc(3);
	/// let mapped = map::<CatListBrand, _, _, _>(|x: i32| x * 2, list);
	/// let vec: Vec<_> = mapped.into_iter().collect();
	/// assert_eq!(vec, vec![2, 4, 6]);
	/// ```
	fn map<'a, A: 'a, B: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> B + 'a,
	{
		fa.into_iter().map(func).collect()
	}
}

impl Lift for CatListBrand {
	/// Lifts a binary function into the list context (Cartesian product).
	///
	/// This method applies a binary function to all pairs of elements from two lists, producing a new list containing the results (Cartesian product).
	///
	/// ### Type Signature
	///
	#[hm_signature(Lift)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements in the first list.",
		"The type of the elements in the second list.",
		"The type of the elements in the resulting list.",
		"The type of the binary function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The binary function to apply.
	/// * `fa`: The first list.
	/// * `fb`: The second list.
	///
	/// ### Returns
	///
	/// A new list containing the results of applying the function to all pairs of elements.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let list1 = CatList::singleton(1).snoc(2);
	/// let list2 = CatList::singleton(10).snoc(20);
	/// let lifted = lift2::<CatListBrand, _, _, _, _>(|x, y| x + y, list1, list2);
	/// let vec: Vec<_> = lifted.into_iter().collect();
	/// assert_eq!(vec, vec![11, 21, 12, 22]);
	/// ```
	fn lift2<'a, A, B, C, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
	where
		Func: Fn(A, B) -> C + 'a,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a,
	{
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
	///
	/// ### Type Signature
	///
	#[hm_signature(Pointed)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the value to wrap."
	)]	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A list containing the single value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::CatListBrand;
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list = pure::<CatListBrand, _>(5);
	/// let vec: Vec<_> = list.into_iter().collect();
	/// assert_eq!(vec, vec![5]);
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		CatList::singleton(a)
	}
}

impl ApplyFirst for CatListBrand {}
impl ApplySecond for CatListBrand {}

impl Semiapplicative for CatListBrand {
	/// Applies wrapped functions to wrapped values (Cartesian product).
	///
	/// This method applies each function in the first list to each value in the second list, producing a new list containing all the results.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semiapplicative)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function wrapper.",
		"The type of the input values.",
		"The type of the output values."
	)]	///
	/// ### Parameters
	///
	/// * `ff`: The list containing the functions.
	/// * `fa`: The list containing the values.
	///
	/// ### Returns
	///
	/// A new list containing the results of applying each function to each value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::cat_list::CatList};
	///
	/// let funcs = CatList::singleton(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1))
	///     .snoc(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// let vals = CatList::singleton(1).snoc(2);
	/// let applied = apply::<RcFnBrand, CatListBrand, _, _>(funcs, vals);
	/// let vec: Vec<_> = applied.into_iter().collect();
	/// assert_eq!(vec, vec![2, 3, 2, 4]);
	/// ```
	fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		ff.into_iter().flat_map(|f| fa.clone().into_iter().map(move |a| f(a.clone()))).collect()
	}
}

impl Semimonad for CatListBrand {
	/// Chains list computations (`flat_map`).
	///
	/// This method applies a function that returns a list to each element of the input list, and then flattens the result.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semimonad)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements in the input list.",
		"The type of the elements in the output list.",
		("A", "The type of the elements in the input list.")
	)]	///
	/// ### Parameters
	///
	/// * `ma`: The first list.
	/// * `f`: The function to apply to each element, returning a list.
	///
	/// ### Returns
	///
	/// A new list containing the flattened results.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::CatListBrand;
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list = CatList::singleton(1).snoc(2);
	/// let bound = bind::<CatListBrand, _, _, _>(list, |x| CatList::singleton(x).snoc(x * 2));
	/// let vec: Vec<_> = bound.into_iter().collect();
	/// assert_eq!(vec, vec![1, 2, 2, 4]);
	/// ```
	fn bind<'a, A: 'a, B: 'a, Func>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		func: Func,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		ma.into_iter().flat_map(func).collect()
	}
}

impl Foldable for CatListBrand {
	/// Folds the list from the right.
	///
	/// This method performs a right-associative fold of the list.
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the list.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial value.
	/// * `fa`: The list to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::{CatListBrand, RcFnBrand};
	/// use fp_library::types::cat_list::CatList;
	/// use fp_library::classes::foldable::fold_right;
	///
	/// let list = CatList::singleton(1).snoc(2).snoc(3);
	/// assert_eq!(fold_right::<RcFnBrand, CatListBrand, _, _, _>(|x: i32, acc| x + acc, 0, list), 6);
	/// ```
	fn fold_right<'a, FnBrand, A: 'a, B: 'a, Func>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(A, B) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		fa.into_iter().collect::<Vec<_>>().into_iter().rev().fold(initial, |acc, x| func(x, acc))
	}

	/// Folds the list from the left.
	///
	/// This method performs a left-associative fold of the list.
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the list.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to the accumulator and each element.
	/// * `initial`: The initial value of the accumulator.
	/// * `fa`: The list to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::{CatListBrand, RcFnBrand};
	/// use fp_library::types::cat_list::CatList;
	/// use fp_library::classes::foldable::fold_left;
	///
	/// let list = CatList::singleton(1).snoc(2).snoc(3);
	/// assert_eq!(fold_left::<RcFnBrand, CatListBrand, _, _, _>(|acc, x: i32| acc + x, 0, list), 6);
	/// ```
	fn fold_left<'a, FnBrand, A: 'a, B: 'a, Func>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(B, A) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		fa.into_iter().fold(initial, func)
	}

	/// Maps the values to a monoid and combines them.
	///
	/// This method maps each element of the list to a monoid and then combines the results using the monoid's `append` operation.
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the list.",
		"The type of the monoid.",
		"The type of the mapping function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The list to fold.
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::cat_list::CatList, classes::foldable::fold_map};
	///
	/// let list = CatList::singleton(1).snoc(2).snoc(3);
	/// assert_eq!(
	///     fold_map::<RcFnBrand, CatListBrand, _, _, _>(|x: i32| x.to_string(), list),
	///     "123".to_string()
	/// );
	/// ```
	fn fold_map<'a, FnBrand, A: 'a, M, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		M: Monoid + 'a,
		Func: Fn(A) -> M + 'a,
		FnBrand: CloneableFn + 'a,
	{
		fa.into_iter().map(func).fold(M::empty(), |acc, x| M::append(acc, x))
	}
}

impl Traversable for CatListBrand {
	/// Traverses the list with an applicative function.
	///
	/// This method maps each element of the list to a computation, evaluates them, and combines the results into an applicative context.
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements in the traversable structure.",
		"The type of the elements in the resulting traversable structure.",
		"The applicative context.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning a value in an applicative context.
	/// * `ta`: The list to traverse.
	///
	/// ### Returns
	///
	/// The list wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::{OptionBrand, CatListBrand};
	/// use fp_library::types::cat_list::CatList;
	/// use fp_library::classes::traversable::traverse;
	///
	/// let list = CatList::singleton(1).snoc(2).snoc(3);
	/// let traversed = traverse::<CatListBrand, _, _, OptionBrand, _>(|x| Some(x * 2), list);
	/// let vec: Vec<_> = traversed.unwrap().into_iter().collect();
	/// assert_eq!(vec, vec![2, 4, 6]);
	/// ```
	fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	{
		ta.into_iter().fold(F::pure(CatList::empty()), |acc, x| {
			F::lift2(|list, b| list.snoc(b), acc, func(x))
		})
	}

	/// Sequences a list of applicative.
	///
	/// This method evaluates the computations inside the list and accumulates the results into an applicative context.
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements in the traversable structure.",
		"The applicative context."
	)]	///
	/// ### Parameters
	///
	/// * `ta`: The list containing the applicative values.
	///
	/// ### Returns
	///
	/// The list wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::{OptionBrand, CatListBrand};
	/// use fp_library::types::cat_list::CatList;
	/// use fp_library::classes::traversable::sequence;
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
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		ta.into_iter()
			.fold(F::pure(CatList::empty()), |acc, x| F::lift2(|list, a| list.snoc(a), acc, x))
	}
}

impl ParFoldable for CatListBrand {
	/// Maps values to a monoid and combines them in parallel.
	///
	/// This method maps each element of the list to a monoid and then combines the results using the monoid's `append` operation. The mapping and combination operations may be executed in parallel.
	///
	/// **Note: The `rayon` feature must be enabled to use parallel iteration.**
	///
	/// ### Type Signature
	///
	#[hm_signature(SendCloneableFn)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		("A", "The element type (must be `Send + Sync`)."),
		"The element type (must be `Send + Sync`).",
		"The monoid type (must be `Send + Sync`)."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to map each element to a monoid.
	/// * `fa`: The list to fold.
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::cat_list::CatList};
	///
	/// let list = CatList::singleton(1).snoc(2).snoc(3);
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
	/// assert_eq!(par_fold_map::<ArcFnBrand, CatListBrand, _, _>(f, list), "123".to_string());
	/// ```
	fn par_fold_map<'a, FnBrand, A, M>(
		func: <FnBrand as SendCloneableFn>::SendOf<'a, A, M>,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		FnBrand: 'a + SendCloneableFn,
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a,
	{
		// CatList doesn't support parallel iteration directly, so we collect to Vec first.
		let vec: Vec<_> = fa.into_iter().collect();
		#[cfg(feature = "rayon")]
		{
			vec.into_par_iter().map(|a| func(a)).reduce(M::empty, |acc, m| M::append(acc, m))
		}
		#[cfg(not(feature = "rayon"))]
		{
			#[allow(clippy::redundant_closure)]
			vec.into_iter().map(|a| func(a)).fold(M::empty(), |acc, m| M::append(acc, m))
		}
	}
}

impl Compactable for CatListBrand {
	/// Compacts a list of options.
	///
	/// This method flattens a list of options, discarding `None` values.
	///
	/// ### Type Signature
	///
	#[hm_signature(Compactable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements."
	)]	///
	/// ### Parameters
	///
	/// * `fa`: The list of options.
	///
	/// ### Returns
	///
	/// The flattened list.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::CatListBrand;
	/// use fp_library::types::cat_list::CatList;
	/// use fp_library::classes::compactable::compact;
	///
	/// let list = CatList::singleton(Some(1)).snoc(None).snoc(Some(2));
	/// let compacted = compact::<CatListBrand, _>(list);
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
	///
	/// ### Type Signature
	///
	#[hm_signature(Compactable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the success value.",
		"The type of the error value."
	)]	///
	/// ### Parameters
	///
	/// * `fa`: The list of results.
	///
	/// ### Returns
	///
	/// A pair of lists.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	/// use fp_library::types::cat_list::CatList;
	/// use fp_library::classes::compactable::separate;
	///
	/// let list = CatList::singleton(Ok(1)).snoc(Err("error")).snoc(Ok(2));
	/// let Pair(oks, errs) = separate::<CatListBrand, _, _>(list);
	/// let oks_vec: Vec<_> = oks.into_iter().collect();
	/// let errs_vec: Vec<_> = errs.into_iter().collect();
	/// assert_eq!(oks_vec, vec![1, 2]);
	/// assert_eq!(errs_vec, vec!["error"]);
	/// ```
	fn separate<'a, O: 'a, E: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
	> {
		let mut oks = CatList::empty();
		let mut errs = CatList::empty();
		for result in fa {
			match result {
				Ok(o) => oks = oks.snoc(o),
				Err(e) => errs = errs.snoc(e),
			}
		}
		Pair(oks, errs)
	}
}

impl Filterable for CatListBrand {
	/// Partitions a list based on a function that returns a result.
	///
	/// This method partitions a list based on a function that returns a result.
	///
	/// ### Type Signature
	///
	#[hm_signature(Filterable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the input value.",
		"The type of the success value.",
		"The type of the error value.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply.
	/// * `fa`: The list to partition.
	///
	/// ### Returns
	///
	/// A pair of lists.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	/// use fp_library::types::cat_list::CatList;
	/// use fp_library::classes::filterable::partition_map;
	///
	/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
	/// let Pair(oks, errs) = partition_map::<CatListBrand, _, _, _, _>(|a| if a % 2 == 0 { Ok(a) } else { Err(a) }, list);
	/// let oks_vec: Vec<_> = oks.into_iter().collect();
	/// let errs_vec: Vec<_> = errs.into_iter().collect();
	/// assert_eq!(oks_vec, vec![2, 4]);
	/// assert_eq!(errs_vec, vec![1, 3]);
	/// ```
	fn partition_map<'a, A: 'a, O: 'a, E: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
	>
	where
		Func: Fn(A) -> Result<O, E> + 'a,
	{
		let mut oks = CatList::empty();
		let mut errs = CatList::empty();
		for a in fa {
			match func(a) {
				Ok(o) => oks = oks.snoc(o),
				Err(e) => errs = errs.snoc(e),
			}
		}
		Pair(oks, errs)
	}

	/// Partitions a list based on a predicate.
	///
	/// This method partitions a list based on a predicate.
	///
	/// ### Type Signature
	///
	#[hm_signature(Filterable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements.",
		"The type of the predicate."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The predicate.
	/// * `fa`: The list to partition.
	///
	/// ### Returns
	///
	/// A pair of lists.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	/// use fp_library::types::cat_list::CatList;
	/// use fp_library::classes::filterable::partition;
	///
	/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
	/// let Pair(satisfied, not_satisfied) = partition::<CatListBrand, _, _>(|a| a % 2 == 0, list);
	/// let sat_vec: Vec<_> = satisfied.into_iter().collect();
	/// let not_sat_vec: Vec<_> = not_satisfied.into_iter().collect();
	/// assert_eq!(sat_vec, vec![2, 4]);
	/// assert_eq!(not_sat_vec, vec![1, 3]);
	/// ```
	fn partition<'a, A: 'a + Clone, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	>
	where
		Func: Fn(A) -> bool + 'a,
	{
		let mut satisfied = CatList::empty();
		let mut not_satisfied = CatList::empty();
		for a in fa {
			if func(a.clone()) {
				satisfied = satisfied.snoc(a);
			} else {
				not_satisfied = not_satisfied.snoc(a);
			}
		}
		Pair(satisfied, not_satisfied)
	}

	/// Maps a function over a list and filters out `None` results.
	///
	/// This method maps a function over a list and filters out `None` results.
	///
	/// ### Type Signature
	///
	#[hm_signature(Filterable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the input value.",
		"The type of the result of applying the function.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply.
	/// * `fa`: The list to filter and map.
	///
	/// ### Returns
	///
	/// The filtered and mapped list.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::CatListBrand;
	/// use fp_library::types::cat_list::CatList;
	/// use fp_library::classes::filterable::filter_map;
	///
	/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
	/// let filtered = filter_map::<CatListBrand, _, _, _>(|a| if a % 2 == 0 { Some(a * 2) } else { None }, list);
	/// let vec: Vec<_> = filtered.into_iter().collect();
	/// assert_eq!(vec, vec![4, 8]);
	/// ```
	fn filter_map<'a, A: 'a, B: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Option<B> + 'a,
	{
		fa.into_iter().filter_map(func).collect()
	}

	/// Filters a list based on a predicate.
	///
	/// This method filters a list based on a predicate.
	///
	/// ### Type Signature
	///
	#[hm_signature(Filterable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements.",
		"The type of the predicate."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The predicate.
	/// * `fa`: The list to filter.
	///
	/// ### Returns
	///
	/// The filtered list.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::CatListBrand;
	/// use fp_library::types::cat_list::CatList;
	/// use fp_library::classes::filterable::filter;
	///
	/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
	/// let filtered = filter::<CatListBrand, _, _>(|a| a % 2 == 0, list);
	/// let vec: Vec<_> = filtered.into_iter().collect();
	/// assert_eq!(vec, vec![2, 4]);
	/// ```
	fn filter<'a, A: 'a + Clone, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Func: Fn(A) -> bool + 'a,
	{
		fa.into_iter().filter(|a| func(a.clone())).collect()
	}
}

impl Witherable for CatListBrand {
	/// Partitions a list based on a function that returns a result in an applicative context.
	///
	/// This method partitions a list based on a function that returns a result in an applicative context.
	///
	/// ### Type Signature
	///
	#[hm_signature(Witherable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The applicative context.",
		"The type of the input value.",
		"The type of the success value.",
		"The type of the error value.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply.
	/// * `ta`: The list to partition.
	///
	/// ### Returns
	///
	/// The partitioned list wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	/// use fp_library::types::cat_list::CatList;
	/// use fp_library::classes::witherable::wilt;
	///
	/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
	/// let wilted = wilt::<CatListBrand, OptionBrand, _, _, _, _>(|a| Some(if a % 2 == 0 { Ok(a) } else { Err(a) }), list);
	/// let Pair(oks, errs) = wilted.unwrap();
	/// let oks_vec: Vec<_> = oks.into_iter().collect();
	/// let errs_vec: Vec<_> = errs.into_iter().collect();
	/// assert_eq!(oks_vec, vec![2, 4]);
	/// assert_eq!(errs_vec, vec![1, 3]);
	/// ```
	fn wilt<'a, M: Applicative, A: 'a + Clone, O: 'a + Clone, E: 'a + Clone, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Pair<
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		>,
	>)
	where
		Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
	{
		ta.into_iter().fold(M::pure(Pair(CatList::empty(), CatList::empty())), |acc, x| {
			M::lift2(
				|mut pair, res| {
					match res {
						Ok(o) => pair.0 = pair.0.snoc(o),
						Err(e) => pair.1 = pair.1.snoc(e),
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
	///
	/// ### Type Signature
	///
	#[hm_signature(Witherable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The applicative context.",
		"The type of the elements in the input structure.",
		"The type of the result of applying the function.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning an `Option` in an applicative context.
	/// * `ta`: The list to filter and map.
	///
	/// ### Returns
	///
	/// The filtered and mapped list wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::{CatListBrand, OptionBrand};
	/// use fp_library::types::cat_list::CatList;
	/// use fp_library::classes::witherable::wither;
	///
	/// let list = CatList::singleton(1).snoc(2).snoc(3).snoc(4);
	/// let withered = wither::<CatListBrand, OptionBrand, _, _, _>(|a| Some(if a % 2 == 0 { Some(a * 2) } else { None }), list);
	/// let vec: Vec<_> = withered.unwrap().into_iter().collect();
	/// assert_eq!(vec, vec![4, 8]);
	/// ```
	fn wither<'a, M: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	>)
	where
		Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
	{
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

impl<A> Semigroup for CatList<A> {
	/// Appends one list to another.
	///
	/// This method concatenates two lists.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semigroup)]
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the list.
	///
	/// ### Parameters
	///
	/// * `a`: The first list.
	/// * `b`: The second list.
	///
	/// ### Returns
	///
	/// The concatenated list.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::types::cat_list::CatList;
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

impl<A> Monoid for CatList<A> {
	/// Returns an empty list.
	///
	/// This method returns a new, empty list.
	///
	/// ### Type Signature
	///
	#[hm_signature(Monoid)]
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the list.
	///
	/// ### Returns
	///
	/// An empty list.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list = empty::<CatList<i32>>();
	/// assert!(list.is_empty());
	/// ```
	fn empty() -> Self {
		CatList::empty()
	}
}

impl<A> CatList<A> {
	/// Creates an empty CatList.
	///
	/// ### Type Signature
	///
	/// `forall a. () -> CatList a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the list.
	///
	/// ### Returns
	///
	/// An empty `CatList`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list: CatList<i32> = CatList::empty();
	/// assert!(list.is_empty());
	/// ```
	#[inline]
	pub const fn empty() -> Self {
		CatList::Nil
	}

	/// Returns `true` if the list is empty.
	///
	/// ### Type Signature
	///
	/// `forall a. CatList a -> bool`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the list.
	///
	/// ### Parameters
	///
	/// * `self`: The list to check.
	///
	/// ### Returns
	///
	/// `true` if the list is empty, `false` otherwise.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list: CatList<i32> = CatList::empty();
	/// assert!(list.is_empty());
	/// ```
	#[inline]
	pub fn is_empty(&self) -> bool {
		matches!(self, CatList::Nil)
	}

	/// Creates a CatList with a single element.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> CatList a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the element.
	///
	/// ### Parameters
	///
	/// * `a`: The element to put in the list.
	///
	/// ### Returns
	///
	/// A `CatList` containing the single element.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list = CatList::singleton(1);
	/// assert!(!list.is_empty());
	/// ```
	#[inline]
	pub fn singleton(a: A) -> Self {
		CatList::Cons(a, VecDeque::new(), 1)
	}

	/// Appends an element to the front of the list.
	///
	/// ### Type Signature
	///
	/// `forall a. (CatList a, a) -> CatList a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the element.
	///
	/// ### Parameters
	///
	/// * `self`: The list.
	/// * `a`: The element to append.
	///
	/// ### Returns
	///
	/// The new list with the element appended to the front.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list = CatList::empty().cons(1);
	/// ```
	#[inline]
	pub fn cons(
		self,
		a: A,
	) -> Self {
		Self::link(CatList::singleton(a), self)
	}

	/// Appends an element to the back of the list.
	///
	/// ### Type Signature
	///
	/// `forall a. (CatList a, a) -> CatList a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the element.
	///
	/// ### Parameters
	///
	/// * `self`: The list.
	/// * `a`: The element to append.
	///
	/// ### Returns
	///
	/// The new list with the element appended to the back.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list = CatList::empty().snoc(1);
	/// ```
	#[inline]
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
	///
	/// ### Type Signature
	///
	/// `forall a. (CatList a, CatList a) -> CatList a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements.
	///
	/// ### Parameters
	///
	/// * `self`: The first list.
	/// * `other`: The second list.
	///
	/// ### Returns
	///
	/// The concatenated list.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list1 = CatList::singleton(1);
	/// let list2 = CatList::singleton(2);
	/// let list3 = list1.append(list2);
	/// ```
	pub fn append(
		self,
		other: Self,
	) -> Self {
		Self::link(self, other)
	}

	/// Internal linking operation.
	///
	/// Links two CatLists by pushing the second onto the first's sublist deque.
	fn link(
		left: Self,
		right: Self,
	) -> Self {
		match (left, right) {
			(CatList::Nil, cat) => cat,
			(cat, CatList::Nil) => cat,
			(CatList::Cons(a, mut q, len), cat) => {
				let new_len = len + cat.len();
				q.push_back(cat);
				CatList::Cons(a, q, new_len)
			}
		}
	}

	/// Removes and returns the first element.
	///
	/// Returns `None` if the list is empty.
	///
	/// ### Type Signature
	///
	/// `forall a. CatList a -> Option (a, CatList a)`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements.
	///
	/// ### Parameters
	///
	/// * `self`: The list.
	///
	/// ### Returns
	///
	/// An option containing the first element and the rest of the list, or `None` if empty.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list = CatList::singleton(1);
	/// let (a, list) = list.uncons().unwrap();
	/// assert_eq!(a, 1);
	/// assert!(list.is_empty());
	/// ```
	pub fn uncons(self) -> Option<(A, Self)> {
		match self {
			CatList::Nil => None,
			CatList::Cons(a, q, _) => {
				if q.is_empty() {
					Some((a, CatList::Nil))
				} else {
					// Flatten the deque of sublists into a single CatList
					let tail = Self::flatten_deque(q);
					Some((a, tail))
				}
			}
		}
	}

	/// Flattens a deque of CatLists into a single CatList.
	///
	/// This is equivalent to `foldr link CatNil deque` in PureScript.
	///
	/// We use an iterative approach to avoid stack overflow on deeply nested structures.
	fn flatten_deque(deque: VecDeque<CatList<A>>) -> Self {
		// Right fold: link(list[0], link(list[1], ... link(list[n-1], Nil)))
		// We process from right to left using DoubleEndedIterator
		deque.into_iter().rfold(CatList::Nil, |acc, list| Self::link(list, acc))
	}

	/// Returns the number of elements.
	///
	/// ### Type Signature
	///
	/// `forall a. CatList a -> usize`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements.
	///
	/// ### Parameters
	///
	/// * `self`: The list.
	///
	/// ### Returns
	///
	/// The number of elements in the list.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list = CatList::singleton(1);
	/// assert_eq!(list.len(), 1);
	/// ```
	#[inline]
	pub fn len(&self) -> usize {
		match self {
			CatList::Nil => 0,
			CatList::Cons(_, _, len) => *len,
		}
	}
}

// Iteration support
impl<A> IntoIterator for CatList<A> {
	type Item = A;
	type IntoIter = CatListIter<A>;

	fn into_iter(self) -> Self::IntoIter {
		CatListIter { list: self }
	}
}

/// An iterator that consumes a `CatList`.
///
/// ### Type Parameters
///
/// * `A`: The type of the elements in the list.
///
/// ### Fields
///
/// * `list`: The list being iterated over.
pub struct CatListIter<A> {
	list: CatList<A>,
}

impl<A> Iterator for CatListIter<A> {
	type Item = A;

	fn next(&mut self) -> Option<Self::Item> {
		let (head, tail) = std::mem::take(&mut self.list).uncons()?;
		self.list = tail;
		Some(head)
	}
}

// FromIterator for easy construction
impl<A> FromIterator<A> for CatList<A> {
	fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
		iter.into_iter().fold(CatList::Nil, |acc, a| acc.snoc(a))
	}
}

#[cfg(test)]
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
		let list: CatList<_> = (0..10).collect();
		let vec: Vec<_> = list.into_iter().collect();
		assert_eq!(vec, (0..10).collect::<Vec<_>>());
	}

	/// Tests the O(1) length tracking.
	/// We create a list with 100 elements.
	/// We verify that the length is reported correctly as 100.
	#[test]
	fn test_len() {
		let list: CatList<_> = (0..100).collect();
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

	use crate::{brands::*, functions::*};
	use quickcheck_macros::quickcheck;

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(x: Vec<i32>) -> bool {
		let x: CatList<_> = x.into_iter().collect();
		map::<CatListBrand, _, _, _>(identity, x.clone()) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(x: Vec<i32>) -> bool {
		let x: CatList<_> = x.into_iter().collect();
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<CatListBrand, _, _, _>(compose(f, g), x.clone())
			== map::<CatListBrand, _, _, _>(f, map::<CatListBrand, _, _, _>(g, x))
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(v: Vec<i32>) -> bool {
		let v: CatList<_> = v.into_iter().collect();
		apply::<RcFnBrand, CatListBrand, _, _>(
			pure::<CatListBrand, _>(<RcFnBrand as CloneableFn>::new(identity)),
			v.clone(),
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, CatListBrand, _, _>(
			pure::<CatListBrand, _>(<RcFnBrand as CloneableFn>::new(f)),
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
		bind::<CatListBrand, _, _, _>(pure::<CatListBrand, _>(a), f) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(m: Vec<i32>) -> bool {
		let m: CatList<_> = m.into_iter().collect();
		bind::<CatListBrand, _, _, _>(m.clone(), pure::<CatListBrand, _>) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(m: Vec<i32>) -> bool {
		let m: CatList<_> = m.into_iter().collect();
		let f = |x: i32| CatList::singleton(x.wrapping_mul(2));
		let g = |x: i32| CatList::singleton(x.wrapping_add(1));
		bind::<CatListBrand, _, _, _>(bind::<CatListBrand, _, _, _>(m.clone(), f), g)
			== bind::<CatListBrand, _, _, _>(m, |x| bind::<CatListBrand, _, _, _>(f(x), g))
	}

	// Edge Cases

	/// Tests `map` on an empty list.
	#[test]
	fn map_empty() {
		assert_eq!(
			map::<CatListBrand, _, _, _>(|x: i32| x + 1, CatList::empty() as CatList<i32>),
			CatList::empty() as CatList<i32>
		);
	}

	/// Tests `bind` on an empty list.
	#[test]
	fn bind_empty() {
		assert_eq!(
			bind::<CatListBrand, _, _, _>(CatList::empty() as CatList<i32>, |x: i32| {
				CatList::singleton(x + 1)
			}),
			CatList::empty() as CatList<i32>
		);
	}

	/// Tests `bind` returning an empty list.
	#[test]
	fn bind_returning_empty() {
		let list: CatList<_> = vec![1, 2, 3].into_iter().collect();
		assert_eq!(
			bind::<CatListBrand, _, _, _>(list, |_| CatList::empty() as CatList<i32>),
			CatList::empty() as CatList<i32>
		);
	}

	/// Tests `fold_right` on an empty list.
	#[test]
	fn fold_right_empty() {
		assert_eq!(
			crate::classes::foldable::fold_right::<RcFnBrand, CatListBrand, _, _, _>(
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
			crate::classes::foldable::fold_left::<RcFnBrand, CatListBrand, _, _, _>(
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
			crate::classes::traversable::traverse::<CatListBrand, _, _, OptionBrand, _>(
				|x: i32| Some(x + 1),
				CatList::empty()
			),
			Some(CatList::empty())
		);
	}

	// ParFoldable Tests

	/// Tests `par_fold_map` on an empty list.
	#[test]
	fn par_fold_map_empty() {
		let v: CatList<i32> = CatList::empty();
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, CatListBrand, _, _>(f, v), "".to_string());
	}

	/// Tests `par_fold_map` on a single element.
	#[test]
	fn par_fold_map_single() {
		let v = CatList::singleton(1);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, CatListBrand, _, _>(f, v), "1".to_string());
	}

	/// Tests `par_fold_map` on multiple elements.
	#[test]
	fn par_fold_map_multiple() {
		let v: CatList<_> = vec![1, 2, 3].into_iter().collect();
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, CatListBrand, _, _>(f, v), "123".to_string());
	}

	// Filterable Laws

	/// Tests `filterMap identity ≡ compact`.
	#[quickcheck]
	fn filterable_filter_map_identity(x: Vec<Option<i32>>) -> bool {
		let x: CatList<_> = x.into_iter().collect();
		filter_map::<CatListBrand, _, _, _>(identity, x.clone()) == compact::<CatListBrand, _>(x)
	}

	/// Tests `filterMap Just ≡ identity`.
	#[quickcheck]
	fn filterable_filter_map_just(x: Vec<i32>) -> bool {
		let x: CatList<_> = x.into_iter().collect();
		filter_map::<CatListBrand, _, _, _>(Some, x.clone()) == x
	}

	/// Tests `partitionMap identity ≡ separate`.
	#[quickcheck]
	fn filterable_partition_map_identity(x: Vec<Result<i32, i32>>) -> bool {
		let x: CatList<_> = x.into_iter().collect();
		partition_map::<CatListBrand, _, _, _, _>(identity, x.clone())
			== separate::<CatListBrand, _, _>(x)
	}

	// Witherable Laws

	/// Tests `wither (pure <<< Just) ≡ pure`.
	#[quickcheck]
	fn witherable_identity(x: Vec<i32>) -> bool {
		let x: CatList<_> = x.into_iter().collect();
		wither::<CatListBrand, OptionBrand, _, _, _>(|i| Some(Some(i)), x.clone()) == Some(x)
	}
}
