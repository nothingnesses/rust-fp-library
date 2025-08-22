use std::sync::Arc;

use crate::{
	aliases::ArcFn,
	functions::{identity, map},
	hkt::Apply1,
	typeclasses::{Applicative, Foldable, Functor},
};

/// A typeclass for traversable functors.
///
/// `Traversable` functors can be traversed, which accumulates results and effects in some [`Applicative`] context.
///
/// A minimum implementation of `Traversable` requires the manual implementation of at least [`Traversable::traverse`] or [`Traversable::sequence`].
pub trait Traversable<'a>: Functor + Foldable {
	/// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
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
	/// * `f`: A function that inputs the elements in the [`Traversable`] structure and outputs applicative computations for each.
	/// * `ta`: A [`Traversable`] structure containing values.
	///
	/// # Returns
	///
	/// An [`Applicative`] containing the accumulated results of the computations.
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
	fn traverse<F: Applicative, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, Apply1<F, B>>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<F, Apply1<Self, B>>>
	where
		Apply1<F, B>: 'a + Clone,
		Apply1<F, ArcFn<'a, Apply1<Self, B>, Apply1<Self, B>>>: Clone,
	{
		Arc::new(move |ta| Self::sequence::<F, B>(map::<Self, _, Apply1<F, B>>(f.clone())(ta)))
	}

	/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
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
	/// * `t`: A [`Traversable`] structure containing [`Applicative`] computations.
	///
	/// # Returns
	///
	/// An [`Applicative`] containing the accumulated results of the computations.
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
	fn sequence<F: Applicative, A: 'a + Clone>(
		t: Apply1<Self, Apply1<F, A>>
	) -> Apply1<F, Apply1<Self, A>>
	where
		Apply1<F, A>: 'a + Clone,
		Apply1<F, ArcFn<'a, Apply1<Self, A>, Apply1<Self, A>>>: Clone,
	{
		(Self::traverse::<F, _, A>(Arc::new(identity)))(t)
	}
}

/// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
///
/// Free function version that dispatches to [the typeclass' associated function][`Traversable::traverse`].
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
/// * `f`: A function that inputs the elements in the [`Traversable`] structure and outputs applicative computations for each.
/// * `ta`: A [`Traversable`] structure containing values.
///
/// # Returns
///
/// An [`Applicative`] containing the accumulated results of the computations.
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
pub fn traverse<'a, Brand: Traversable<'a>, F: Applicative, A: 'a + Clone, B: 'a + Clone>(
	f: ArcFn<'a, A, Apply1<F, B>>
) -> ArcFn<'a, Apply1<Brand, A>, Apply1<F, Apply1<Brand, B>>>
where
	Apply1<F, B>: 'a + Clone,
	Apply1<F, ArcFn<'a, Apply1<Brand, B>, Apply1<Brand, B>>>: Clone,
{
	Brand::traverse::<F, _, B>(f)
}

/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
///
/// Free function version that dispatches to [the typeclass' associated function][`Traversable::sequence`].
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
/// * `t`: A [`Traversable`] structure containing [`Applicative`] computations.
///
/// # Returns
///
/// An [`Applicative`] containing the accumulated results of the computations.
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
pub fn sequence<'a, Brand: Traversable<'a>, F: Applicative, A: 'a + Clone>(
	t: Apply1<Brand, Apply1<F, A>>
) -> Apply1<F, Apply1<Brand, A>>
where
	Apply1<F, A>: 'a + Clone,
	Apply1<F, ArcFn<'a, Apply1<Brand, A>, Apply1<Brand, A>>>: Clone,
{
	Brand::sequence::<F, A>(t)
}
