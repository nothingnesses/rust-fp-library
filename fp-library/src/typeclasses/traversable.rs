use std::{convert::identity, sync::Arc};

use crate::{
	aliases::ClonableFn,
	functions::map,
	hkt::{Apply, Kind},
	typeclasses::{Applicative, Foldable, Functor},
};

/// A typeclass for structures that can be traversed with applicative effects,
/// allowing the transformation of structures while preserving their shape.
///
/// A `Traversable` represents a structure that can be traversed from left to right,
/// applying an applicative action to each element and combining the results.
/// This is useful for operations like validating data structures,
/// performing I/O on collections, or applying transformations within an effectful context.
///
/// A minimum implementation of `Traversable` requires the manual implementation of at least [`Traversable::traverse`] or [`Traversable::sequence`].
pub trait Traversable: Functor + Foldable {
	/// Traverses the structure with an applicative action, producing a new structure within the applicative context.
	///
	/// The default implementation of `traverse` is implemented in terms of [`sequence`] and [`map`] where:
	///
	/// `(traverse f) ta = sequence ((map f) ta)`
	///
	/// # Type Signature
	///
	/// `forall t f a b. Traversable t, Applicative f => (a -> f b) -> t a -> f (t b)`
	///
	/// # Parameters
	///
	/// * `f`: A function that converts values of type `A` into applicative actions of type `F<B>`.
	/// * `ta`: A traversable structure containing values of type `A`.
	///
	/// # Returns
	///
	/// An applicative containing the traversed structure of type `F<T<B>>` where `T` is the traversable structure.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::VecBrand, functions::traverse};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     traverse::<VecBrand, Option, i32, i32>(Arc::new(|x| Arc::new(Some(x * 2))))(vec![1, 2, 3]),
	///     Some(vec![2, 4, 6])
	/// );
	/// ```
	fn traverse<'a, F, A, B>(
		f: ClonableFn<'a, A, Apply<F, (B,)>>
	) -> ClonableFn<'a, Apply<Self, (A,)>, Apply<F, (Apply<Self, (B,)>,)>>
	where
		Self: Kind<(A,)> + Kind<(B,)> + Kind<(Apply<F, (B,)>,)>,
		A: 'a,
		F: 'a + Kind<(B,)> + Kind<(Apply<Self, (B,)>,)> + Applicative,
		Apply<F, (B,)>: 'a,
	{
		Arc::new(move |ta| Self::sequence::<F, B>(map::<Self, _, Apply<F, (B,)>>(f.clone())(ta)))
	}

	/// Collects applicative actions within a traversable structure into a single applicative action containing the traversed structure.
	///
	/// The default implementation of `sequence` is implemented in terms of [`traverse`] and [`identity`] where:
	///
	/// `sequence = traverse identity`
	///
	/// # Type Signature
	///
	/// `forall t f a. Traversable t, Applicative f => t (f a) -> f (t a)`
	///
	/// # Parameters
	///
	/// * `t`: A traversable structure containing applicative values of type `F<A>`.
	///
	/// # Returns
	///
	/// An applicative containing the traversed structure of type `F<T<A>>`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::VecBrand, functions::sequence};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     sequence::<VecBrand, Option, i32>(vec![Some(1), Some(2), Some(3)]),
	///     Some(vec![1, 2, 3])
	/// );
	/// ```
	fn sequence<F, A>(t: Apply<Self, (Apply<F, (A,)>,)>) -> Apply<F, (Apply<Self, (A,)>,)>
	where
		Self: Kind<(Apply<F, (A,)>,)> + Kind<(A,)>,
		F: Kind<(A,)> + Kind<(Apply<Self, (A,)>,)> + Applicative,
	{
		(Self::traverse::<F, _, A>(Arc::new(identity)))(t)
	}
}

pub fn traverse<'a, Brand, F, A, B>(
	f: ClonableFn<'a, A, Apply<F, (B,)>>
) -> ClonableFn<'a, Apply<Brand, (A,)>, Apply<F, (Apply<Brand, (B,)>,)>>
where
	Brand: Kind<(A,)> + Kind<(B,)> + Kind<(Apply<F, (B,)>,)> + Traversable,
	A: 'a,
	F: 'a + Kind<(B,)> + Kind<(Apply<Brand, (B,)>,)> + Applicative,
	Apply<F, (B,)>: 'a,
{
	Brand::traverse::<F, _, B>(f)
}

pub fn sequence<Brand, F, A>(t: Apply<Brand, (Apply<F, (A,)>,)>) -> Apply<F, (Apply<Brand, (A,)>,)>
where
	Brand: Kind<(Apply<F, (A,)>,)> + Kind<(A,)> + Traversable,
	F: Kind<(A,)> + Kind<(Apply<Brand, (A,)>,)> + Applicative,
{
	Brand::sequence::<F, A>(t)
}
