//! Implementations for [`Vec`].
//!
//! This module provides implementations of various type classes for the `Vec` type.

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::{
	Apply,
	brands::VecBrand,
	classes::{
		applicative::Applicative, apply_first::ApplyFirst, apply_second::ApplySecond,
		clonable_fn::ClonableFn, foldable::Foldable, functor::Functor, lift::Lift, monoid::Monoid,
		par_foldable::ParFoldable, pointed::Pointed, semiapplicative::Semiapplicative,
		semigroup::Semigroup, semimonad::Semimonad, send_clonable_fn::SendClonableFn,
		traversable::Traversable,
	},
	impl_kind,
	kinds::*,
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
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
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
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
		fb: Apply!(brand: Self, signature: ('a, B: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, C: 'a) -> 'a)
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
	fn pure<'a, A: 'a>(a: A) -> Apply!(brand: Self, signature: ('a, A: 'a) -> 'a) {
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
		ff: Apply!(brand: Self, signature: ('a, Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, B)): 'a) -> 'a),
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) {
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
		ma: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
		f: F,
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
	where
		F: Fn(A) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) + 'a,
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
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
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
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
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
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
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
		ta: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): 'a) -> 'a)
	where
		Func: Fn(A) -> Apply!(brand: F, signature: ('a, B: 'a) -> 'a) + 'a,
		Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): Clone,
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
		ta: Apply!(brand: Self, signature: ('a, Apply!(brand: F, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	where
		Apply!(brand: F, signature: ('a, A: 'a) -> 'a): Clone,
		Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): Clone,
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
		func: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, M)),
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::{ArcFnBrand, RcFnBrand},
		classes::{
			functor::map,
			monoid::empty,
			par_foldable::{par_fold_map, par_fold_right},
			pointed::pure,
			semiapplicative::apply,
			semigroup::append,
			semimonad::bind,
			send_clonable_fn::new_send,
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
}
