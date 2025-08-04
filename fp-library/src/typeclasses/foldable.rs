use crate::{
	aliases::ClonableFn,
	functions::{compose, flip, identity},
	hkt::{Apply, Kind},
	typeclasses::Monoid,
};
use std::sync::Arc;

/// A typeclass for structures that can be folded to a single value.
///
/// A `Foldable` represents a structure that can be folded over to combine its elements
/// into a single result. This is useful for operations like summing values, collecting into a collection,
/// or applying monoidal operations.
pub trait Foldable {
	/// Folds the structure by applying a function from left to right.
	///
	/// The default implementation of `fold_left` is implemented in terms of [`fold_right`], [`flip`], [`compose`] and [`identity`] where:
	///
	/// `((fold_left f) b) fa = (((fold_right (((compose (flip compose)) (flip f)))) identity) fa) b`
	///
	/// # Type Signature
	///
	/// `forall f a b. Foldable f => (b -> a -> b) -> b -> f a -> b`
	///
	/// # Parameters
	///
	/// * `f`: A curried binary function that takes in the current value of the accumulator, the next item in the structure and returns the next value of accumulator.
	/// * `b`: Initial value of type `B`.
	/// * `fa`: A foldable structure containing values of type `A`.
	///
	/// # Returns
	///
	/// Final value of type `B` obtained from the folding operation.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::VecBrand, functions::fold_left};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     fold_left::<VecBrand, _, _>(Arc::new(|carry| Arc::new(move |item| carry * 2 + item)))(0)(vec![1, 2, 3]),
	///     11
	/// );
	/// ```
	fn fold_left<'a, A, B>(
		f: ClonableFn<'a, B, ClonableFn<'a, A, B>>
	) -> ClonableFn<'a, B, ClonableFn<'a, Apply<Self, (A,)>, B>>
	where
		Self: 'a + Kind<(A,)>,
		A: 'a + Clone,
		B: 'a + Clone,
		Apply<Self, (A,)>: 'a,
	{
		Arc::new(move |b| {
			Arc::new({
				let f = f.clone();
				move |fa| {
					(((Self::fold_right(compose(flip(Arc::new(compose)))(flip(f.clone()))))(
						Arc::new(identity),
					))(fa))(b.clone())
				}
			})
		})
	}

	/// Maps values to a monoid and combines them.
	///
	/// The default implementation of `fold_map` is implemented in terms of [`fold_right`], [`compose`], [`append`][crate::functions::append] and [`empty`][crate::functions::empty] where:
	///
	/// `fold_map f = (fold_right ((compose append) f)) empty`
	///
	/// # Type Signature
	///
	/// `forall f a m. Foldable f, Monoid m => (a -> m) -> f a -> m`
	///
	/// # Parameters
	///
	/// * `f`: A function that converts from values into monoidal elements.
	/// * `fa`: A foldable structure containing values of type `A`.
	///
	/// # Returns
	///
	/// Final monoid obtained from the folding operation.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{StringBrand, VecBrand}, functions::{fold_map, identity}};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     fold_map::<VecBrand, _, StringBrand>(Arc::new(identity))(vec![
	///         "Hello, ".to_string(),
	///         "World!".to_string()
	///     ]),
	///     "Hello, World!"
	/// );
	/// ```
	fn fold_map<'a, A, M>(
		f: ClonableFn<'a, A, Apply<M, ()>>
	) -> ClonableFn<'a, Apply<Self, (A,)>, Apply<M, ()>>
	where
		Self: Kind<(A,)>,
		A: 'a + Clone,
		M: Monoid,
		Apply<M, ()>: 'a + Clone,
	{
		Arc::new(move |fa| {
			let f = f.clone();
			((Self::fold_right(Arc::new(move |a: A| {
				let f = f.clone();
				Arc::new(move |m| ((compose(Arc::new(M::append))(f.clone()))(a.to_owned()))(m))
			})))(M::empty()))(fa)
		})
	}

	/// Folds the structure by applying a function from right to left.
	///
	/// # Type Signature
	///
	/// `forall f a b. Foldable f => (a -> b -> b) -> b -> f a -> b`
	///
	/// # Parameters
	///
	/// * `f`: A curried binary function that takes in the next item in the structure, the current value of the accumulator and returns the next value of accumulator.
	/// * `b`: Initial value of type `B`.
	/// * `fa`: A foldable structure containing values of type `A`.
	///
	/// # Returns
	///
	/// Final value of type `B` obtained from the folding operation.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::VecBrand, functions::fold_right};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     fold_right::<VecBrand, _, _>(Arc::new(|item| Arc::new(move |carry| carry * 2 + item)))(0)(vec![1, 2, 3]),
	///     17
	/// );
	/// ```
	fn fold_right<'a, A, B>(
		f: ClonableFn<'a, A, ClonableFn<'a, B, B>>
	) -> ClonableFn<'a, B, ClonableFn<'a, Apply<Self, (A,)>, B>>
	where
		Self: 'a + Kind<(A,)>,
		A: 'a + Clone,
		B: 'a + Clone,
		Apply<Self, (A,)>: 'a;
}

/// Folds the structure by applying a function from left to right.
///
/// Free function version that dispatches to [the typeclass' associated function][`Foldable::fold_left`].
///
/// The default implementation of `fold_left` is implemented in terms of [`fold_right`], [`flip`], [`compose`] and [`identity`] where:
///
/// `((fold_left f) b) fa = (((fold_right (((compose (flip compose)) (flip f)))) identity) fa) b`
///
/// # Type Signature
///
/// `forall f a b. Foldable f => (b -> a -> b) -> b -> f a -> b`
///
/// # Parameters
///
/// * `f`: A curried binary function that takes in the current value of the accumulator, the next item in the structure and returns the next value of accumulator.
/// * `b`: Initial value of type `B`.
/// * `fa`: A foldable structure containing values of type `A`.
///
/// # Returns
///
/// Final value of type `B` obtained from the folding operation.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::VecBrand, functions::fold_left};
/// use std::sync::Arc;
///
/// assert_eq!(
///     fold_left::<VecBrand, _, _>(Arc::new(|carry| Arc::new(move |item| carry * 2 + item)))(0)(vec![1, 2, 3]),
///     11
/// );
/// ```
pub fn fold_left<'a, Brand, A, B>(
	f: ClonableFn<'a, B, ClonableFn<'a, A, B>>
) -> ClonableFn<'a, B, ClonableFn<'a, Apply<Brand, (A,)>, B>>
where
	Brand: 'a + Kind<(A,)> + Foldable,
	A: 'a + Clone,
	B: 'a + Clone,
	Apply<Brand, (A,)>: 'a,
{
	Brand::fold_left(f)
}

/// Maps values to a monoid and combines them.
///
/// Free function version that dispatches to [the typeclass' associated function][`Foldable::fold_map`].
///
/// The default implementation of `fold_map` is implemented in terms of [`fold_right`], [`compose`], [`append`][crate::functions::append] and [`empty`][crate::functions::empty] where:
///
/// `fold_map f = (fold_right ((compose append) f)) empty`
///
/// # Type Signature
///
/// `forall f a m. Foldable f, Monoid m => (a -> m) -> f a -> m`
///
/// # Parameters
///
/// * `f`: A function that converts from values into monoidal elements.
/// * `fa`: A foldable structure containing values of type `A`.
///
/// # Returns
///
/// Final monoid obtained from the folding operation.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::{StringBrand, VecBrand}, functions::{fold_map, identity}};
/// use std::sync::Arc;
///
/// assert_eq!(
///     fold_map::<VecBrand, _, StringBrand>(Arc::new(identity))(vec![
///         "Hello, ".to_string(),
///         "World!".to_string()
///     ]),
///     "Hello, World!"
/// );
/// ```
pub fn fold_map<'a, Brand, A, M>(
	f: ClonableFn<'a, A, Apply<M, ()>>
) -> ClonableFn<'a, Apply<Brand, (A,)>, Apply<M, ()>>
where
	Brand: Kind<(A,)> + Foldable,
	A: 'a + Clone,
	M: Monoid,
	Apply<M, ()>: 'a + Clone,
{
	Brand::fold_map::<_, M>(f)
}

/// Folds the structure by applying a function from right to left.
///
/// Free function version that dispatches to [the typeclass' associated function][`Foldable::fold_right`].
///
/// # Type Signature
///
/// `forall f a b. Foldable f => (a -> b -> b) -> b -> f a -> b`
///
/// # Parameters
///
/// * `f`: A curried binary function that takes in the next item in the structure, the current value of the accumulator and returns the next value of accumulator.
/// * `b`: Initial value of type `B`.
/// * `fa`: A foldable structure containing values of type `A`.
///
/// # Returns
///
/// Final value of type `B` obtained from the folding operation.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::VecBrand, functions::fold_right};
/// use std::sync::Arc;
///
/// assert_eq!(
///     fold_right::<VecBrand, _, _>(Arc::new(|item| Arc::new(move |carry| carry * 2 + item)))(0)(vec![1, 2, 3]),
///     17
/// );
/// ```
pub fn fold_right<'a, Brand, A, B>(
	f: ClonableFn<'a, A, ClonableFn<'a, B, B>>
) -> ClonableFn<'a, B, ClonableFn<'a, Apply<Brand, (A,)>, B>>
where
	Brand: 'a + Kind<(A,)> + Foldable,
	A: 'a + Clone,
	B: 'a + Clone,
	Apply<Brand, (A,)>: 'a,
{
	Brand::fold_right(f)
}
