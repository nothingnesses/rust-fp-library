use std::sync::Arc;

use crate::{
	aliases::ArcFn,
	functions::{identity, map},
	hkt::Apply1,
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
pub trait Traversable<'a>: Functor + Foldable {
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
	/// use fp_library::{brands::{VecBrand, OptionBrand}, functions::traverse};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     traverse::<VecBrand, OptionBrand, i32, i32>(Arc::new(|x| Some(x * 2)))(vec![1, 2, 3]),
	///     Some(vec![2, 4, 6])
	/// );
	/// ```
	fn traverse<F: Applicative, A: 'a + Clone, B: Clone>(
		f: ArcFn<'a, A, Apply1<F, B>>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<F, Apply1<Self, B>>>
	where
		Apply1<F, B>: 'a + Clone,
		Apply1<F, ArcFn<'a, Apply1<Self, B>, Apply1<Self, B>>>: Clone,
	{
		Arc::new(move |ta| Self::sequence::<F, B>(map::<Self, _, Apply1<F, B>>(f.clone())(ta)))
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
	/// use fp_library::{brands::{VecBrand, OptionBrand}, functions::sequence};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     sequence::<VecBrand, OptionBrand, i32>(vec![Some(1), Some(2), Some(3)]),
	///     Some(vec![1, 2, 3])
	/// );
	/// ```
	fn sequence<F: Applicative, A: Clone>(
		t: Apply1<Self, Apply1<F, A>>
	) -> Apply1<F, Apply1<Self, A>>
	where
		Apply1<F, A>: Clone,
		Apply1<F, ArcFn<'a, Apply1<Self, A>, Apply1<Self, A>>>: Clone,
	{
		(Self::traverse::<F, _, A>(Arc::new(identity)))(t)
	}
}

pub fn traverse<'a, Brand: Traversable<'a>, F: Applicative, A: 'a + Clone, B: Clone>(
	f: ArcFn<'a, A, Apply1<F, B>>
) -> ArcFn<'a, Apply1<Brand, A>, Apply1<F, Apply1<Brand, B>>>
where
	Apply1<F, B>: 'a + Clone,
	Apply1<F, ArcFn<'a, Apply1<Brand, B>, Apply1<Brand, B>>>: Clone,
{
	Brand::traverse::<F, _, B>(f)
}

pub fn sequence<'a, Brand: Traversable<'a>, F: Applicative, A: Clone>(
	t: Apply1<Brand, Apply1<F, A>>
) -> Apply1<F, Apply1<Brand, A>>
where
	Apply1<F, A>: Clone,
	Apply1<F, ArcFn<'a, Apply1<Brand, A>, Apply1<Brand, A>>>: Clone,
{
	Brand::sequence::<F, A>(t)
}
