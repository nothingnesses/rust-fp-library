use crate::{
	hkt::{Apply0L1T, Kind0L1T},
	typeclasses::{ClonableFn, clonable_fn::ApplyFn},
};

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// If `x` has type `m a` and `f` has type `a -> m b`, then `bind(x)(f)` has type `m b`,
/// representing the result of executing `x` to get a value of type `a` and then
/// passing it to `f` to get a computation of type `m b`.
///
/// Note that `Bind` is a separate typeclass from [`Monad`][`crate::typeclasses::Monad`]. In this library's
/// hierarchy, [`Monad`][`crate::typeclasses::Monad`] is a typeclass that extends both
/// [`Applicative`][`crate::typeclasses::Applicative`] and `Bind`.
pub trait Bind: Kind0L1T {
	/// Sequences two computations, allowing the second to depend on the value computed by the first.
	///
	/// # Type Signature
	///
	/// `forall m a b. Bind m => m a -> (a -> m b) -> m b`
	///
	/// # Parameters
	///
	/// * `ma`: The first computation in the context.
	/// * `f`: A function that takes the result of the first computation and returns the second computation in the context.
	///
	/// # Returns
	///
	/// A computation that sequences the two operations.
	fn bind<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B>(
		ma: Apply0L1T<Self, A>
	) -> ApplyFn<
		'a,
		ClonableFnBrand,
		ApplyFn<'a, ClonableFnBrand, A, Apply0L1T<Self, B>>,
		Apply0L1T<Self, B>,
	>;
}

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// Free function version that dispatches to [the typeclass' associated function][`Bind::bind`].
///
/// # Type Signature
///
/// `forall m a b. Bind m => m a -> (a -> m b) -> m b`
///
/// # Parameters
///
/// * `ma`: The first computation in the context.
/// * `f`: A function that takes the result of the first computation and returns the second computation in the context.
///
/// # Returns
///
/// A computation that sequences the two operations.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::{OptionBrand, RcFnBrand}, functions::{bind, pure}};
/// use std::rc::Rc;
///
/// assert_eq!(bind::<RcFnBrand, OptionBrand, _, _>(Some(5))(Rc::new(|x| Some(x * 2))), Some(10));
/// ```
pub fn bind<'a, ClonableFnBrand: 'a + ClonableFn, Brand: Bind, A: 'a + Clone, B>(
	ma: Apply0L1T<Brand, A>
) -> ApplyFn<
	'a,
	ClonableFnBrand,
	ApplyFn<'a, ClonableFnBrand, A, Apply0L1T<Brand, B>>,
	Apply0L1T<Brand, B>,
> {
	Brand::bind::<ClonableFnBrand, A, B>(ma)
}
