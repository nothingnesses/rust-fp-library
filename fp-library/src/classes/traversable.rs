use crate::{
	classes::{Applicative, ClonableFn, Foldable, Functor, clonable_fn::ApplyFn},
	functions::{identity, map},
	hkt::Apply0L1T,
};

/// A type class for traversable functors.
///
/// `Traversable` functors can be traversed, which accumulates results and effects in some [`Applicative`] context.
///
/// A minimum implementation of `Traversable` requires the manual implementation of at least [`Traversable::traverse`] or [`Traversable::sequence`].
pub trait Traversable: Functor + Foldable {
	/// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
	///
	/// The default implementation of `traverse` is implemented in terms of [`sequence`] and [`map`] where:
	///
	/// `(traverse f) ta = sequence ((map f) ta)`
	///
	/// # Type Signature
	///
	/// `forall a b. Traversable t, Applicative f => (a -> f b) -> t a -> f (t b)`
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
	/// use fp_library::{brands::{VecBrand, OptionBrand, RcFnBrand}, functions::traverse};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     traverse::<RcFnBrand, VecBrand, OptionBrand, i32, i32>(Rc::new(|x| Some(x * 2)))(vec![1, 2, 3]),
	///     Some(vec![2, 4, 6])
	/// );
	/// ```
	fn traverse<
		'a,
		ClonableFnBrand: 'a + ClonableFn,
		F: Applicative,
		A: 'a + Clone,
		B: 'a + Clone,
	>(
		f: ApplyFn<'a, ClonableFnBrand, A, Apply0L1T<F, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<F, Apply0L1T<Self, B>>>
	where
		Apply0L1T<F, B>: Clone,
		Apply0L1T<F, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>>>: Clone,
		Apply0L1T<Self, B>: 'a,
		Apply0L1T<Self, Apply0L1T<F, B>>: 'a,
	{
		ClonableFnBrand::new(move |ta| {
			Self::sequence::<ClonableFnBrand, F, B>(
				map::<ClonableFnBrand, Self, _, Apply0L1T<F, B>>(f.clone())(ta),
			)
		})
	}

	/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
	///
	/// The default implementation of `sequence` is implemented in terms of [`traverse`] and [`identity`] where:
	///
	/// `sequence = traverse identity`
	///
	/// # Type Signature
	///
	/// `forall a. Traversable t, Applicative f => t (f a) -> f (t a)`
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
	/// use fp_library::{brands::{VecBrand, OptionBrand, RcFnBrand}, functions::sequence};
	///
	/// assert_eq!(
	///     sequence::<RcFnBrand, VecBrand, OptionBrand, i32>(vec![Some(1), Some(2), Some(3)]),
	///     Some(vec![1, 2, 3])
	/// );
	/// ```
	fn sequence<'a, ClonableFnBrand: 'a + ClonableFn, F: Applicative, A: 'a + Clone>(
		t: Apply0L1T<Self, Apply0L1T<F, A>>
	) -> Apply0L1T<F, Apply0L1T<Self, A>>
	where
		Apply0L1T<F, A>: 'a + Clone,
		Apply0L1T<F, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, A>>>: Clone,
		Apply0L1T<Self, A>: 'a,
		Apply0L1T<Self, Apply0L1T<F, A>>: 'a,
		Apply0L1T<F, Apply0L1T<Self, A>>: 'a,
	{
		(Self::traverse::<ClonableFnBrand, F, _, A>(ClonableFnBrand::new(identity)))(t)
	}
}

/// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
///
/// Free function version that dispatches to [the type class' associated function][`Traversable::traverse`].
///
/// The default implementation of `traverse` is implemented in terms of [`sequence`] and [`map`] where:
///
/// `(traverse f) ta = sequence ((map f) ta)`
///
/// # Type Signature
///
/// `forall a b. Traversable t, Applicative f => (a -> f b) -> t a -> f (t b)`
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
/// use fp_library::{brands::{VecBrand, OptionBrand, RcFnBrand}, functions::traverse};
/// use std::rc::Rc;
///
/// assert_eq!(
///     traverse::<RcFnBrand, VecBrand, OptionBrand, i32, i32>(Rc::new(|x| Some(x * 2)))(vec![1, 2, 3]),
///     Some(vec![2, 4, 6])
/// );
/// ```
pub fn traverse<
	'a,
	ClonableFnBrand: 'a + ClonableFn,
	Brand: Traversable,
	F: Applicative,
	A: 'a + Clone,
	B: 'a + Clone,
>(
	f: ApplyFn<'a, ClonableFnBrand, A, Apply0L1T<F, B>>
) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Brand, A>, Apply0L1T<F, Apply0L1T<Brand, B>>>
where
	Apply0L1T<F, B>: Clone,
	Apply0L1T<F, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Brand, B>, Apply0L1T<Brand, B>>>: Clone,
	Apply0L1T<Brand, B>: 'a,
	Apply0L1T<Brand, Apply0L1T<F, B>>: 'a,
{
	Brand::traverse::<ClonableFnBrand, F, _, B>(f)
}

/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
///
/// Free function version that dispatches to [the type class' associated function][`Traversable::sequence`].
///
/// The default implementation of `sequence` is implemented in terms of [`traverse`] and [`identity`] where:
///
/// `sequence = traverse identity`
///
/// # Type Signature
///
/// `forall a. Traversable t, Applicative f => t (f a) -> f (t a)`
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
/// use fp_library::{brands::{VecBrand, OptionBrand, RcFnBrand}, functions::sequence};
/// use std::rc::Rc;
///
/// assert_eq!(
///     sequence::<RcFnBrand, VecBrand, OptionBrand, i32>(vec![Some(1), Some(2), Some(3)]),
///     Some(vec![1, 2, 3])
/// );
/// ```
pub fn sequence<
	'a,
	ClonableFnBrand: 'a + ClonableFn,
	Brand: Traversable,
	F: Applicative,
	A: 'a + Clone,
>(
	t: Apply0L1T<Brand, Apply0L1T<F, A>>
) -> Apply0L1T<F, Apply0L1T<Brand, A>>
where
	Apply0L1T<F, A>: 'a + Clone,
	Apply0L1T<F, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Brand, A>, Apply0L1T<Brand, A>>>: Clone,
	Apply0L1T<Brand, A>: 'a,
	Apply0L1T<Brand, Apply0L1T<F, A>>: 'a,
	Apply0L1T<F, Apply0L1T<Brand, A>>: 'a,
{
	Brand::sequence::<ClonableFnBrand, F, A>(t)
}
