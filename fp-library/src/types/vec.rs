//! Implementations for [`Vec`].
//!
//! This module provides implementations of various type classes for the `Vec` type.

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::{
	Apply,
	brands::{OptionBrand, VecBrand},
	classes::{
		applicative::Applicative, apply_first::ApplyFirst, apply_second::ApplySecond,
		clonable_fn::ClonableFn, compactable::Compactable, filterable::Filterable,
		foldable::Foldable, functor::Functor, lift::Lift, monoid::Monoid,
		par_foldable::ParFoldable, pointed::Pointed, semiapplicative::Semiapplicative,
		semigroup::Semigroup, semimonad::Semimonad, send_clonable_fn::SendClonableFn,
		traversable::Traversable, witherable::Witherable,
	},
	impl_kind,
	kinds::*,
	types::Pair,
};

impl_kind! {
	for VecBrand {
		type Of<'a, A: 'a>: 'a = Vec<A>;
	}
}

impl VecBrand {
	/// Constructs a new vector by prepending a value to an existing vector.
	///
	/// This method creates a new vector with the given head element followed by the elements of the tail vector.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Vec a -> Vec a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the vector.
	///
	/// ### Parameters
	///
	/// * `head`: A value to prepend to the vector.
	/// * `tail`: A vector to prepend the value to.
	///
	/// ### Returns
	///
	/// A new vector consisting of the `head` element prepended to the `tail` vector.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::brands::VecBrand;
	///
	/// let head = 1;
	/// let tail = vec![2, 3];
	/// let new_vec = VecBrand::construct(head, tail);
	/// assert_eq!(new_vec, vec![1, 2, 3]);
	///
	/// let empty_tail = vec![];
	/// let single_element = VecBrand::construct(42, empty_tail);
	/// assert_eq!(single_element, vec![42]);
	/// ```
	pub fn construct<A>(
		head: A,
		tail: Vec<A>,
	) -> Vec<A>
	where
		A: Clone,
	{
		[vec![head], tail].concat()
	}

	/// Deconstructs a slice into its head element and tail vector.
	///
	/// This method splits a slice into its first element and the rest of the elements as a new vector.
	///
	/// ### Type Signature
	///
	/// `forall a. &[a] -> Option (a, Vec a)`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the vector.
	///
	/// ### Parameters
	///
	/// * `slice`: The vector slice to deconstruct.
	///
	/// ### Returns
	///
	/// An [`Option`] containing a tuple of the head element and the remaining tail vector,
	/// or [`None`] if the slice is empty.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::brands::VecBrand;
	///
	/// let vec = vec![1, 2, 3];
	/// let deconstructed = VecBrand::deconstruct(&vec);
	/// assert_eq!(deconstructed, Some((1, vec![2, 3])));
	///
	/// let empty: Vec<i32> = vec![];
	/// assert_eq!(VecBrand::deconstruct(&empty), None);
	/// ```
	pub fn deconstruct<A>(slice: &[A]) -> Option<(A, Vec<A>)>
	where
		A: Clone,
	{
		match slice {
			[] => None,
			[head, tail @ ..] => Some((head.clone(), tail.to_vec())),
		}
	}
}

impl Functor for VecBrand {
	/// Maps a function over the vector.
	///
	/// This method applies a function to each element of the vector, producing a new vector with the transformed values.
	///
	/// ### Type Signature
	///
	/// `forall a b. Functor Vec => (a -> b, Vec a) -> Vec b`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the function to apply.
	/// * `A`: The type of the elements in the vector.
	/// * `B`: The type of the elements in the resulting vector.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to each element.
	/// * `fa`: The vector to map over.
	///
	/// ### Returns
	///
	/// A new vector containing the results of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::functor::map;
	/// use fp_library::brands::VecBrand;
	///
	/// assert_eq!(map::<VecBrand, _, _, _>(|x: i32| x * 2, vec![1, 2, 3]), vec![2, 4, 6]);
	/// ```
	fn map<'a, F, A: 'a, B: 'a>(
		f: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> B + 'a,
	{
		fa.into_iter().map(f).collect()
	}
}

impl Lift for VecBrand {
	/// Lifts a binary function into the vector context (Cartesian product).
	///
	/// This method applies a binary function to all pairs of elements from two vectors, producing a new vector containing the results (Cartesian product).
	///
	/// ### Type Signature
	///
	/// `forall a b c. Lift Vec => ((a, b) -> c, Vec a, Vec b) -> Vec c`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the binary function.
	/// * `A`: The type of the elements in the first vector.
	/// * `B`: The type of the elements in the second vector.
	/// * `C`: The type of the elements in the resulting vector.
	///
	/// ### Parameters
	///
	/// * `f`: The binary function to apply.
	/// * `fa`: The first vector.
	/// * `fb`: The second vector.
	///
	/// ### Returns
	///
	/// A new vector containing the results of applying the function to all pairs of elements.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::lift::lift2;
	/// use fp_library::brands::VecBrand;
	///
	/// assert_eq!(
	///     lift2::<VecBrand, _, _, _, _>(|x, y| x + y, vec![1, 2], vec![10, 20]),
	///     vec![11, 21, 12, 22]
	/// );
	/// ```
	fn lift2<'a, F, A, B, C>(
		f: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
	where
		F: Fn(A, B) -> C + 'a,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a,
	{
		fa.iter().flat_map(|a| fb.iter().map(|b| f(a.clone(), b.clone()))).collect()
	}
}

impl Pointed for VecBrand {
	/// Wraps a value in a vector.
	///
	/// This method creates a new vector containing the single given value.
	///
	/// ### Type Signature
	///
	/// `forall a. Pointed Vec => a -> Vec a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value to wrap.
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A vector containing the single value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::pointed::pure;
	/// use fp_library::brands::VecBrand;
	///
	/// assert_eq!(pure::<VecBrand, _>(5), vec![5]);
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		vec![a]
	}
}

impl ApplyFirst for VecBrand {}
impl ApplySecond for VecBrand {}

impl Semiapplicative for VecBrand {
	/// Applies wrapped functions to wrapped values (Cartesian product).
	///
	/// This method applies each function in the first vector to each value in the second vector, producing a new vector containing all the results.
	///
	/// ### Type Signature
	///
	/// `forall a b. Semiapplicative Vec => (Vec (a -> b), Vec a) -> Vec b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function wrapper.
	/// * `A`: The type of the input values.
	/// * `B`: The type of the output values.
	///
	/// ### Parameters
	///
	/// * `ff`: The vector containing the functions.
	/// * `fa`: The vector containing the values.
	///
	/// ### Returns
	///
	/// A new vector containing the results of applying each function to each value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::semiapplicative::apply;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::{VecBrand};
	/// use fp_library::brands::RcFnBrand;
	/// use std::rc::Rc;
	///
	/// let funcs = vec![
	///     <RcFnBrand as ClonableFn>::new(|x: i32| x + 1),
	///     <RcFnBrand as ClonableFn>::new(|x: i32| x * 2),
	/// ];
	/// assert_eq!(apply::<RcFnBrand, VecBrand, _, _>(funcs, vec![1, 2]), vec![2, 3, 2, 4]);
	/// ```
	fn apply<'a, FnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as ClonableFn>::Of<'a, A, B>>),
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		ff.iter().flat_map(|f| fa.iter().map(move |a| f(a.clone()))).collect()
	}
}

impl Semimonad for VecBrand {
	/// Chains vector computations (flat_map).
	///
	/// This method applies a function that returns a vector to each element of the input vector, and then flattens the result.
	///
	/// ### Type Signature
	///
	/// `forall a b. Semimonad Vec => (Vec a, a -> Vec b) -> Vec b`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the function to apply.
	/// * `A`: The type of the elements in the input vector.
	/// * `B`: The type of the elements in the output vector.
	///
	/// ### Parameters
	///
	/// * `ma`: The first vector.
	/// * `f`: The function to apply to each element, returning a vector.
	///
	/// ### Returns
	///
	/// A new vector containing the flattened results.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::semimonad::bind;
	/// use fp_library::brands::VecBrand;
	///
	/// assert_eq!(
	///     bind::<VecBrand, _, _, _>(vec![1, 2], |x| vec![x, x * 2]),
	///     vec![1, 2, 2, 4]
	/// );
	/// ```
	fn bind<'a, F, A: 'a, B: 'a>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: F,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		ma.into_iter().flat_map(f).collect()
	}
}

impl Foldable for VecBrand {
	/// Folds the vector from the right.
	///
	/// This method performs a right-associative fold of the vector.
	///
	/// ### Type Signature
	///
	/// `forall a b. Foldable Vec => ((a, b) -> b, b, Vec a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function to use.
	/// * `Func`: The type of the folding function.
	/// * `A`: The type of the elements in the vector.
	/// * `B`: The type of the accumulator.
	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial value.
	/// * `fa`: The vector to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_right;
	/// use fp_library::brands::VecBrand;
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(fold_right::<RcFnBrand, VecBrand, _, _, _>(|x: i32, acc| x + acc, 0, vec![1, 2, 3]), 6);
	/// ```
	fn fold_right<'a, FnBrand, Func, A: 'a, B: 'a>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(A, B) -> B + 'a,
		FnBrand: ClonableFn + 'a,
	{
		fa.into_iter().rev().fold(initial, |acc, x| func(x, acc))
	}

	/// Folds the vector from the left.
	///
	/// This method performs a left-associative fold of the vector.
	///
	/// ### Type Signature
	///
	/// `forall a b. Foldable Vec => ((b, a) -> b, b, Vec a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function to use.
	/// * `Func`: The type of the folding function.
	/// * `A`: The type of the elements in the vector.
	/// * `B`: The type of the accumulator.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to the accumulator and each element.
	/// * `initial`: The initial value of the accumulator.
	/// * `fa`: The vector to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_left;
	/// use fp_library::brands::VecBrand;
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(fold_left::<RcFnBrand, VecBrand, _, _, _>(|acc, x: i32| acc + x, 0, vec![1, 2, 3]), 6);
	/// ```
	fn fold_left<'a, FnBrand, Func, A: 'a, B: 'a>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(B, A) -> B + 'a,
		FnBrand: ClonableFn + 'a,
	{
		fa.into_iter().fold(initial, func)
	}

	/// Maps the values to a monoid and combines them.
	///
	/// This method maps each element of the vector to a monoid and then combines the results using the monoid's `append` operation.
	///
	/// ### Type Signature
	///
	/// `forall a m. (Foldable Vec, Monoid m) => ((a) -> m, Vec a) -> m`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function to use.
	/// * `Func`: The type of the mapping function.
	/// * `A`: The type of the elements in the vector.
	/// * `M`: The type of the monoid.
	///
	/// ### Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The vector to fold.
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_map;
	/// use fp_library::brands::VecBrand;
	/// use fp_library::types::string; // Import to bring Monoid impl for String into scope
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(
	///     fold_map::<RcFnBrand, VecBrand, _, _, _>(|x: i32| x.to_string(), vec![1, 2, 3]),
	///     "123".to_string()
	/// );
	/// ```
	fn fold_map<'a, FnBrand, Func, A: 'a, M>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		M: Monoid + 'a,
		Func: Fn(A) -> M + 'a,
		FnBrand: ClonableFn + 'a,
	{
		fa.into_iter().map(func).fold(M::empty(), |acc, x| M::append(acc, x))
	}
}

impl Traversable for VecBrand {
	/// Traverses the vector with an applicative function.
	///
	/// This method maps each element of the vector to a computation, evaluates them, and combines the results into an applicative context.
	///
	/// ### Type Signature
	///
	/// `forall a b f. (Traversable Vec, Applicative f) => (a -> f b, Vec a) -> f (Vec b)`
	///
	/// ### Type Parameters
	///
	/// * `F`: The applicative context.
	/// * `Func`: The type of the function to apply.
	/// * `A`: The type of the elements in the vector.
	/// * `B`: The type of the elements in the resulting vector.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning a value in an applicative context.
	/// * `ta`: The vector to traverse.
	///
	/// ### Returns
	///
	/// The vector wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::traversable::traverse;
	/// use fp_library::brands::{OptionBrand, VecBrand};
	///
	/// assert_eq!(
	///     traverse::<VecBrand, OptionBrand, _, _, _>(|x| Some(x * 2), vec![1, 2, 3]),
	///     Some(vec![2, 4, 6])
	/// );
	/// ```
	fn traverse<'a, F: Applicative, Func, A: 'a + Clone, B: 'a + Clone>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	{
		let len = ta.len();
		ta.into_iter().fold(F::pure(Vec::with_capacity(len)), |acc, x| {
			F::lift2(
				|mut v, b| {
					v.push(b);
					v
				},
				acc,
				func(x),
			)
		})
	}
	/// Sequences a vector of applicative.
	///
	/// This method evaluates the computations inside the vector and accumulates the results into an applicative context.
	///
	/// ### Type Signature
	///
	/// `forall a f. (Traversable Vec, Applicative f) => (Vec (f a)) -> f (Vec a)`
	///
	/// ### Type Parameters
	///
	/// * `F`: The applicative context.
	/// * `A`: The type of the elements in the vector.
	///
	/// ### Parameters
	///
	/// * `ta`: The vector containing the applicative values.
	///
	/// ### Returns
	///
	/// The vector wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::traversable::sequence;
	/// use fp_library::brands::{OptionBrand, VecBrand};
	///
	/// assert_eq!(
	///     sequence::<VecBrand, OptionBrand, _>(vec![Some(1), Some(2)]),
	///     Some(vec![1, 2])
	/// );
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		let len = ta.len();
		ta.into_iter().fold(F::pure(Vec::with_capacity(len)), |acc, x| {
			F::lift2(
				|mut v, a| {
					v.push(a);
					v
				},
				acc,
				x,
			)
		})
	}
}

impl<A: Clone> Semigroup for Vec<A> {
	/// Appends one vector to another.
	///
	/// This method concatenates two vectors.
	///
	/// ### Type Signature
	///
	/// `forall a. Semigroup (Vec a) => (Vec a, Vec a) -> Vec a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the vector.
	///
	/// ### Parameters
	///
	/// * `a`: The first vector.
	/// * `b`: The second vector.
	///
	/// ### Returns
	///
	/// The concatenated vector.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::semigroup::append;
	///
	/// assert_eq!(append(vec![1, 2], vec![3, 4]), vec![1, 2, 3, 4]);
	/// ```
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		[a, b].concat()
	}
}

impl<A: Clone> Monoid for Vec<A> {
	/// Returns an empty vector.
	///
	/// This method returns a new, empty vector.
	///
	/// ### Type Signature
	///
	/// `forall a. Monoid (Vec a) => () -> Vec a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the vector.
	///
	/// ### Returns
	///
	/// An empty vector.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::monoid::empty;
	///
	/// assert_eq!(empty::<Vec<i32>>(), vec![]);
	/// ```
	fn empty() -> Self {
		Vec::new()
	}
}

impl<FnBrand: SendClonableFn> ParFoldable<FnBrand> for VecBrand {
	/// Maps values to a monoid and combines them in parallel.
	///
	/// This method maps each element of the vector to a monoid and then combines the results using the monoid's `append` operation. The mapping and combination operations may be executed in parallel.
	///
	/// ### Type Signature
	///
	/// `forall a m. (ParFoldable Vec, Monoid m, Send m, Sync m) => (f a m, Vec a) -> m`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of thread-safe function to use (must implement `SendClonableFn`).
	/// * `A`: The element type (must be `Send + Sync`).
	/// * `M`: The monoid type (must be `Send + Sync`).
	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to map each element to a monoid.
	/// * `fa`: The vector to fold.
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::par_foldable::par_fold_map;
	/// use fp_library::brands::{VecBrand, ArcFnBrand};
	/// use fp_library::classes::send_clonable_fn::SendClonableFn;
	/// use fp_library::types::string;
	///
	/// let v = vec![1, 2, 3];
	/// let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x.to_string());
	/// assert_eq!(par_fold_map::<ArcFnBrand, VecBrand, _, _>(f, v), "123".to_string());
	/// ```
	fn par_fold_map<'a, A, M>(
		func: <FnBrand as SendClonableFn>::SendOf<'a, A, M>,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a,
	{
		#[cfg(feature = "rayon")]
		{
			fa.into_par_iter().map(|a| func(a)).reduce(M::empty, |acc, m| M::append(acc, m))
		}
		#[cfg(not(feature = "rayon"))]
		{
			#[allow(clippy::redundant_closure)]
			fa.into_iter().map(|a| func(a)).fold(M::empty(), |acc, m| M::append(acc, m))
		}
	}
}

impl Compactable for VecBrand {
	/// Compacts a vector of options.
	///
	/// This method flattens a vector of options, discarding `None` values.
	///
	/// ### Type Signature
	///
	/// `forall a. Compactable Vec => Vec (Option a) -> Vec a`
	///
	/// ### Parameters
	///
	/// * `fa`: The vector of options.
	///
	/// ### Returns
	///
	/// The flattened vector.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::compactable::Compactable;
	/// use fp_library::brands::VecBrand;
	///
	/// let x = vec![Some(1), None, Some(2)];
	/// let y = VecBrand::compact(x);
	/// assert_eq!(y, vec![1, 2]);
	/// ```
	fn compact<'a, A: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		fa.into_iter().flatten().collect()
	}

	/// Separates a vector of results.
	///
	/// This method separates a vector of results into a pair of vectors.
	///
	/// ### Type Signature
	///
	/// `forall e o. Compactable Vec => Vec (Result o e) -> (Vec o, Vec e)`
	///
	/// ### Parameters
	///
	/// * `fa`: The vector of results.
	///
	/// ### Returns
	///
	/// A pair of vectors.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::compactable::Compactable;
	/// use fp_library::brands::VecBrand;
	/// use fp_library::types::Pair;
	///
	/// let x = vec![Ok(1), Err("error"), Ok(2)];
	/// let Pair(oks, errs) = VecBrand::separate(x);
	/// assert_eq!(oks, vec![1, 2]);
	/// assert_eq!(errs, vec!["error"]);
	/// ```
	fn separate<'a, E: 'a, O: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
	> {
		let mut oks = Vec::new();
		let mut errs = Vec::new();
		for result in fa {
			match result {
				Ok(o) => oks.push(o),
				Err(e) => errs.push(e),
			}
		}
		Pair(oks, errs)
	}
}

impl Filterable for VecBrand {
	/// Partitions a vector based on a function that returns a result.
	///
	/// This method partitions a vector based on a function that returns a result.
	///
	/// ### Type Signature
	///
	/// `forall a e o. Filterable Vec => (a -> Result o e) -> Vec a -> (Vec o, Vec e)`
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply.
	/// * `fa`: The vector to partition.
	///
	/// ### Returns
	///
	/// A pair of vectors.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::filterable::Filterable;
	/// use fp_library::brands::VecBrand;
	/// use fp_library::types::Pair;
	///
	/// let x = vec![1, 2, 3, 4];
	/// let Pair(oks, errs) = VecBrand::partition_map(|a| if a % 2 == 0 { Ok(a) } else { Err(a) }, x);
	/// assert_eq!(oks, vec![2, 4]);
	/// assert_eq!(errs, vec![1, 3]);
	/// ```
	fn partition_map<'a, Func, A: 'a, E: 'a, O: 'a>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
	>
	where
		Func: Fn(A) -> Result<O, E> + 'a,
	{
		let mut oks = Vec::new();
		let mut errs = Vec::new();
		for a in fa {
			match func(a) {
				Ok(o) => oks.push(o),
				Err(e) => errs.push(e),
			}
		}
		Pair(oks, errs)
	}

	/// Partitions a vector based on a predicate.
	///
	/// This method partitions a vector based on a predicate.
	///
	/// ### Type Signature
	///
	/// `forall a. Filterable Vec => (a -> bool) -> Vec a -> (Vec a, Vec a)`
	///
	/// ### Parameters
	///
	/// * `func`: The predicate.
	/// * `fa`: The vector to partition.
	///
	/// ### Returns
	///
	/// A pair of vectors.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::filterable::Filterable;
	/// use fp_library::brands::VecBrand;
	/// use fp_library::types::Pair;
	///
	/// let x = vec![1, 2, 3, 4];
	/// let Pair(satisfied, not_satisfied) = VecBrand::partition(|a| a % 2 == 0, x);
	/// assert_eq!(satisfied, vec![2, 4]);
	/// assert_eq!(not_satisfied, vec![1, 3]);
	/// ```
	fn partition<'a, Func, A: 'a + Clone>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	>
	where
		Func: Fn(A) -> bool + 'a,
	{
		let (satisfied, not_satisfied): (Vec<A>, Vec<A>) =
			fa.into_iter().partition(|a| func(a.clone()));
		Pair(satisfied, not_satisfied)
	}

	/// Maps a function over a vector and filters out `None` results.
	///
	/// This method maps a function over a vector and filters out `None` results.
	///
	/// ### Type Signature
	///
	/// `forall a b. Filterable Vec => (a -> Option b) -> Vec a -> Vec b`
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply.
	/// * `fa`: The vector to filter and map.
	///
	/// ### Returns
	///
	/// The filtered and mapped vector.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::filterable::Filterable;
	/// use fp_library::brands::VecBrand;
	///
	/// let x = vec![1, 2, 3, 4];
	/// let y = VecBrand::filter_map(|a| if a % 2 == 0 { Some(a * 2) } else { None }, x);
	/// assert_eq!(y, vec![4, 8]);
	/// ```
	fn filter_map<'a, Func, A: 'a, B: 'a>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Option<B> + 'a,
	{
		fa.into_iter().filter_map(func).collect()
	}

	/// Filters a vector based on a predicate.
	///
	/// This method filters a vector based on a predicate.
	///
	/// ### Type Signature
	///
	/// `forall a. Filterable Vec => (a -> bool) -> Vec a -> Vec a`
	///
	/// ### Parameters
	///
	/// * `func`: The predicate.
	/// * `fa`: The vector to filter.
	///
	/// ### Returns
	///
	/// The filtered vector.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::filterable::Filterable;
	/// use fp_library::brands::VecBrand;
	///
	/// let x = vec![1, 2, 3, 4];
	/// let y = VecBrand::filter(|a| a % 2 == 0, x);
	/// assert_eq!(y, vec![2, 4]);
	/// ```
	fn filter<'a, Func, A: 'a + Clone>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Func: Fn(A) -> bool + 'a,
	{
		fa.into_iter().filter(|a| func(a.clone())).collect()
	}
}

impl Witherable for VecBrand {
	/// Partitions a vector based on a function that returns a result in an applicative context.
	///
	/// This method partitions a vector based on a function that returns a result in an applicative context.
	///
	/// ### Type Signature
	///
	/// `forall a e o m. (Witherable Vec, Applicative m) => (a -> m (Result o e)) -> Vec a -> m (Vec o, Vec e)`
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply.
	/// * `ta`: The vector to partition.
	///
	/// ### Returns
	///
	/// The partitioned vector wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::witherable::Witherable;
	/// use fp_library::brands::{VecBrand, OptionBrand};
	/// use fp_library::types::Pair;
	///
	/// let x = vec![1, 2, 3, 4];
	/// let y = VecBrand::wilt::<_, OptionBrand, _, _, _>(|a| Some(if a % 2 == 0 { Ok(a) } else { Err(a) }), x);
	/// assert_eq!(y, Some(Pair(vec![2, 4], vec![1, 3])));
	/// ```
	fn wilt<'a, Func, M: Applicative, A: 'a + Clone, E: 'a + Clone, O: 'a + Clone>(
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
		ta.into_iter().fold(M::pure(Pair(Vec::new(), Vec::new())), |acc, x| {
			M::lift2(
				|mut pair, res| {
					match res {
						Ok(o) => pair.0.push(o),
						Err(e) => pair.1.push(e),
					}
					pair
				},
				acc,
				func(x),
			)
		})
	}

	/// Maps a function over a vector and filters out `None` results in an applicative context.
	///
	/// This method maps a function over a vector and filters out `None` results in an applicative context.
	///
	/// ### Type Signature
	///
	/// `forall a b m. (Witherable Vec, Applicative m) => (a -> m (Option b)) -> Vec a -> m (Vec b)`
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply.
	/// * `ta`: The vector to filter and map.
	///
	/// ### Returns
	///
	/// The filtered and mapped vector wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::witherable::Witherable;
	/// use fp_library::brands::{VecBrand, OptionBrand};
	///
	/// let x = vec![1, 2, 3, 4];
	/// let y = VecBrand::wither::<_, OptionBrand, _, _>(|a| Some(if a % 2 == 0 { Some(a * 2) } else { None }), x);
	/// assert_eq!(y, Some(vec![4, 8]));
	/// ```
	fn wither<'a, Func, M: Applicative, A: 'a + Clone, B: 'a + Clone>(
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
		ta.into_iter().fold(M::pure(Vec::new()), |acc, x| {
			M::lift2(
				|mut v, opt_b| {
					if let Some(b) = opt_b {
						v.push(b);
					}
					v
				},
				acc,
				func(x),
			)
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::{ArcFnBrand, RcFnBrand},
		classes::{
			compactable::{compact, separate},
			filterable::{filter, filter_map, partition, partition_map},
			functor::map,
			monoid::empty,
			par_foldable::{par_fold_map, par_fold_right},
			pointed::pure,
			semiapplicative::apply,
			semigroup::append,
			semimonad::bind,
			send_clonable_fn::new_send,
			traversable::traverse,
			witherable::{wilt, wither},
		},
		functions::{compose, identity},
	};
	use quickcheck_macros::quickcheck;

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(x: Vec<i32>) -> bool {
		map::<VecBrand, _, _, _>(identity, x.clone()) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(x: Vec<i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<VecBrand, _, _, _>(compose(f, g), x.clone())
			== map::<VecBrand, _, _, _>(f, map::<VecBrand, _, _, _>(g, x))
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(v: Vec<i32>) -> bool {
		apply::<RcFnBrand, VecBrand, _, _>(
			pure::<VecBrand, _>(<RcFnBrand as ClonableFn>::new(identity)),
			v.clone(),
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, VecBrand, _, _>(
			pure::<VecBrand, _>(<RcFnBrand as ClonableFn>::new(f)),
			pure::<VecBrand, _>(x),
		) == pure::<VecBrand, _>(f(x))
	}

	/// Tests the composition law for Applicative.
	#[quickcheck]
	fn applicative_composition(
		w: Vec<i32>,
		u_seeds: Vec<i32>,
		v_seeds: Vec<i32>,
	) -> bool {
		let u_fns: Vec<_> = u_seeds
			.iter()
			.map(|&i| <RcFnBrand as ClonableFn>::new(move |x: i32| x.wrapping_add(i)))
			.collect();
		let v_fns: Vec<_> = v_seeds
			.iter()
			.map(|&i| <RcFnBrand as ClonableFn>::new(move |x: i32| x.wrapping_mul(i)))
			.collect();

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, VecBrand, _, _>(v_fns.clone(), w.clone());
		let rhs = apply::<RcFnBrand, VecBrand, _, _>(u_fns.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		// equivalent to (u . v) <*> w
		// We construct (u . v) manually as the cartesian product of compositions
		let uv_fns: Vec<_> = u_fns
			.iter()
			.flat_map(|uf| {
				v_fns.iter().map(move |vf| {
					let uf = uf.clone();
					let vf = vf.clone();
					<RcFnBrand as ClonableFn>::new(move |x| uf(vf(x)))
				})
			})
			.collect();

		let lhs = apply::<RcFnBrand, VecBrand, _, _>(uv_fns, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = vec![<RcFnBrand as ClonableFn>::new(f)];

		let lhs = apply::<RcFnBrand, VecBrand, _, _>(u.clone(), pure::<VecBrand, _>(y));

		let rhs_fn = <RcFnBrand as ClonableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, VecBrand, _, _>(pure::<VecBrand, _>(rhs_fn), u);

		lhs == rhs
	}

	// Semigroup Laws

	/// Tests the associativity law for Semigroup.
	#[quickcheck]
	fn semigroup_associativity(
		a: Vec<i32>,
		b: Vec<i32>,
		c: Vec<i32>,
	) -> bool {
		append(a.clone(), append(b.clone(), c.clone())) == append(append(a, b), c)
	}

	// Monoid Laws

	/// Tests the left identity law for Monoid.
	#[quickcheck]
	fn monoid_left_identity(a: Vec<i32>) -> bool {
		append(empty::<Vec<i32>>(), a.clone()) == a
	}

	/// Tests the right identity law for Monoid.
	#[quickcheck]
	fn monoid_right_identity(a: Vec<i32>) -> bool {
		append(a.clone(), empty::<Vec<i32>>()) == a
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| vec![x.wrapping_mul(2)];
		bind::<VecBrand, _, _, _>(pure::<VecBrand, _>(a), f) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(m: Vec<i32>) -> bool {
		bind::<VecBrand, _, _, _>(m.clone(), pure::<VecBrand, _>) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(m: Vec<i32>) -> bool {
		let f = |x: i32| vec![x.wrapping_mul(2)];
		let g = |x: i32| vec![x.wrapping_add(1)];
		bind::<VecBrand, _, _, _>(bind::<VecBrand, _, _, _>(m.clone(), f), g)
			== bind::<VecBrand, _, _, _>(m, |x| bind::<VecBrand, _, _, _>(f(x), g))
	}

	// Edge Cases

	/// Tests `map` on an empty vector.
	#[test]
	fn map_empty() {
		assert_eq!(
			map::<VecBrand, _, _, _>(|x: i32| x + 1, vec![] as Vec<i32>),
			vec![] as Vec<i32>
		);
	}

	/// Tests `bind` on an empty vector.
	#[test]
	fn bind_empty() {
		assert_eq!(
			bind::<VecBrand, _, _, _>(vec![] as Vec<i32>, |x: i32| vec![x + 1]),
			vec![] as Vec<i32>
		);
	}

	/// Tests `bind` returning an empty vector.
	#[test]
	fn bind_returning_empty() {
		assert_eq!(
			bind::<VecBrand, _, _, _>(vec![1, 2, 3], |_| vec![] as Vec<i32>),
			vec![] as Vec<i32>
		);
	}

	/// Tests `fold_right` on an empty vector.
	#[test]
	fn fold_right_empty() {
		assert_eq!(
			crate::classes::foldable::fold_right::<RcFnBrand, VecBrand, _, _, _>(
				|x: i32, acc| x + acc,
				0,
				vec![]
			),
			0
		);
	}

	/// Tests `fold_left` on an empty vector.
	#[test]
	fn fold_left_empty() {
		assert_eq!(
			crate::classes::foldable::fold_left::<RcFnBrand, VecBrand, _, _, _>(
				|acc, x: i32| acc + x,
				0,
				vec![]
			),
			0
		);
	}

	/// Tests `traverse` on an empty vector.
	#[test]
	fn traverse_empty() {
		use crate::brands::OptionBrand;
		assert_eq!(
			crate::classes::traversable::traverse::<VecBrand, OptionBrand, _, _, _>(
				|x: i32| Some(x + 1),
				vec![]
			),
			Some(vec![])
		);
	}

	/// Tests `traverse` returning an empty vector.
	#[test]
	fn traverse_returning_empty() {
		use crate::brands::OptionBrand;
		assert_eq!(
			crate::classes::traversable::traverse::<VecBrand, OptionBrand, _, _, _>(
				|_: i32| None::<i32>,
				vec![1, 2, 3]
			),
			None
		);
	}

	/// Tests `construct` with an empty tail.
	#[test]
	fn construct_empty_tail() {
		assert_eq!(VecBrand::construct(1, vec![]), vec![1]);
	}

	/// Tests `deconstruct` on an empty slice.
	#[test]
	fn deconstruct_empty() {
		assert_eq!(VecBrand::deconstruct::<i32>(&[]), None);
	}

	// ParFoldable Tests

	/// Tests `par_fold_map` on an empty vector.
	#[test]
	fn par_fold_map_empty() {
		let v: Vec<i32> = vec![];
		let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, VecBrand, _, _>(f, v), "".to_string());
	}

	/// Tests `par_fold_map` on a single element.
	#[test]
	fn par_fold_map_single() {
		let v = vec![1];
		let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, VecBrand, _, _>(f, v), "1".to_string());
	}

	/// Tests `par_fold_map` on multiple elements.
	#[test]
	fn par_fold_map_multiple() {
		let v = vec![1, 2, 3];
		let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, VecBrand, _, _>(f, v), "123".to_string());
	}

	/// Tests `par_fold_right` on multiple elements.
	#[test]
	fn par_fold_right_multiple() {
		let v = vec![1, 2, 3];
		let f = new_send::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, VecBrand, _, _>(f, 0, v), 6);
	}

	// Filterable Laws

	/// Tests `filterMap identity ≡ compact`.
	#[quickcheck]
	fn filterable_filter_map_identity(x: Vec<Option<i32>>) -> bool {
		filter_map::<VecBrand, _, _, _>(identity, x.clone()) == compact::<VecBrand, _>(x)
	}

	/// Tests `filterMap Just ≡ identity`.
	#[quickcheck]
	fn filterable_filter_map_just(x: Vec<i32>) -> bool {
		filter_map::<VecBrand, _, _, _>(Some, x.clone()) == x
	}

	/// Tests `filterMap (l <=< r) ≡ filterMap l <<< filterMap r`.
	#[quickcheck]
	fn filterable_filter_map_composition(x: Vec<i32>) -> bool {
		let r = |i: i32| if i % 2 == 0 { Some(i) } else { None };
		let l = |i: i32| if i > 5 { Some(i) } else { None };
		let composed = |i| r(i).and_then(l);

		filter_map::<VecBrand, _, _, _>(composed, x.clone())
			== filter_map::<VecBrand, _, _, _>(l, filter_map::<VecBrand, _, _, _>(r, x))
	}

	/// Tests `filter ≡ filterMap <<< maybeBool`.
	#[quickcheck]
	fn filterable_filter_consistency(x: Vec<i32>) -> bool {
		let p = |i: i32| i % 2 == 0;
		let maybe_bool = |i| if p(i) { Some(i) } else { None };

		filter::<VecBrand, _, _>(p, x.clone()) == filter_map::<VecBrand, _, _, _>(maybe_bool, x)
	}

	/// Tests `partitionMap identity ≡ separate`.
	#[quickcheck]
	fn filterable_partition_map_identity(x: Vec<Result<i32, i32>>) -> bool {
		partition_map::<VecBrand, _, _, _, _>(identity, x.clone()) == separate::<VecBrand, _, _>(x)
	}

	/// Tests `partitionMap Right ≡ identity` (on the right side).
	#[quickcheck]
	fn filterable_partition_map_right_identity(x: Vec<i32>) -> bool {
		let Pair(oks, _) = partition_map::<VecBrand, _, _, _, _>(Ok::<_, i32>, x.clone());
		oks == x
	}

	/// Tests `partitionMap Left ≡ identity` (on the left side).
	#[quickcheck]
	fn filterable_partition_map_left_identity(x: Vec<i32>) -> bool {
		let Pair(_, errs) = partition_map::<VecBrand, _, _, _, _>(Err::<i32, _>, x.clone());
		errs == x
	}

	/// Tests `f <<< partition ≡ partitionMap <<< eitherBool`.
	#[quickcheck]
	fn filterable_partition_consistency(x: Vec<i32>) -> bool {
		let p = |i: i32| i % 2 == 0;
		let either_bool = |i| if p(i) { Ok(i) } else { Err(i) };

		let Pair(satisfied, not_satisfied) = partition::<VecBrand, _, _>(p, x.clone());
		let Pair(oks, errs) = partition_map::<VecBrand, _, _, _, _>(either_bool, x);

		satisfied == oks && not_satisfied == errs
	}

	// Witherable Laws

	/// Tests `wither (pure <<< Just) ≡ pure`.
	#[quickcheck]
	fn witherable_identity(x: Vec<i32>) -> bool {
		wither::<VecBrand, _, OptionBrand, _, _>(|i| Some(Some(i)), x.clone()) == Some(x)
	}

	/// Tests `wilt p ≡ map separate <<< traverse p`.
	#[quickcheck]
	fn witherable_wilt_consistency(x: Vec<i32>) -> bool {
		let p = |i: i32| Some(if i % 2 == 0 { Ok(i) } else { Err(i) });

		let lhs = wilt::<VecBrand, _, OptionBrand, _, _, _>(p, x.clone());
		let rhs = map::<OptionBrand, _, _, _>(
			|res| separate::<VecBrand, _, _>(res),
			traverse::<VecBrand, OptionBrand, _, _, _>(p, x),
		);

		lhs == rhs
	}

	/// Tests `wither p ≡ map compact <<< traverse p`.
	#[quickcheck]
	fn witherable_wither_consistency(x: Vec<i32>) -> bool {
		let p = |i: i32| Some(if i % 2 == 0 { Some(i) } else { None });

		let lhs = wither::<VecBrand, _, OptionBrand, _, _>(p, x.clone());
		let rhs = map::<OptionBrand, _, _, _>(
			|opt| compact::<VecBrand, _>(opt),
			traverse::<VecBrand, OptionBrand, _, _, _>(p, x),
		);

		lhs == rhs
	}

	// Edge Cases

	/// Tests `compact` on an empty vector.
	#[test]
	fn compact_empty() {
		assert_eq!(compact::<VecBrand, _>(vec![] as Vec<Option<i32>>), vec![]);
	}

	/// Tests `compact` on a vector with `None`.
	#[test]
	fn compact_with_none() {
		assert_eq!(compact::<VecBrand, _>(vec![Some(1), None, Some(2)]), vec![1, 2]);
	}

	/// Tests `separate` on an empty vector.
	#[test]
	fn separate_empty() {
		let Pair(oks, errs) = separate::<VecBrand, _, _>(vec![] as Vec<Result<i32, i32>>);
		assert_eq!(oks, vec![]);
		assert_eq!(errs, vec![]);
	}

	/// Tests `separate` on a vector with `Ok` and `Err`.
	#[test]
	fn separate_mixed() {
		let Pair(oks, errs) = separate::<VecBrand, _, _>(vec![Ok(1), Err(2), Ok(3)]);
		assert_eq!(oks, vec![1, 3]);
		assert_eq!(errs, vec![2]);
	}

	/// Tests `partition_map` on an empty vector.
	#[test]
	fn partition_map_empty() {
		let Pair(oks, errs) =
			partition_map::<VecBrand, _, _, _, _>(|x: i32| Ok::<i32, i32>(x), vec![]);
		assert_eq!(oks, vec![]);
		assert_eq!(errs, vec![]);
	}

	/// Tests `partition` on an empty vector.
	#[test]
	fn partition_empty() {
		let Pair(satisfied, not_satisfied) = partition::<VecBrand, _, _>(|x: i32| x > 0, vec![]);
		assert_eq!(satisfied, vec![]);
		assert_eq!(not_satisfied, vec![]);
	}

	/// Tests `filter_map` on an empty vector.
	#[test]
	fn filter_map_empty() {
		assert_eq!(filter_map::<VecBrand, _, _, _>(|x: i32| Some(x), vec![]), vec![]);
	}

	/// Tests `filter` on an empty vector.
	#[test]
	fn filter_empty() {
		assert_eq!(filter::<VecBrand, _, _>(|x: i32| x > 0, vec![]), vec![]);
	}

	/// Tests `wilt` on an empty vector.
	#[test]
	fn wilt_empty() {
		let res =
			wilt::<VecBrand, _, OptionBrand, _, _, _>(|x: i32| Some(Ok::<i32, i32>(x)), vec![]);
		assert_eq!(res, Some(Pair(vec![], vec![])));
	}

	/// Tests `wither` on an empty vector.
	#[test]
	fn wither_empty() {
		let res = wither::<VecBrand, _, OptionBrand, _, _>(|x: i32| Some(Some(x)), vec![]);
		assert_eq!(res, Some(vec![]));
	}
}
