use crate::hkt::{Apply, Kind};

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// If x has type `m a` and `f` has type `a -> m b`, then `bind(x)(f)` has type `m b`,
/// representing the result of executing `x` to get a value of type `a` and then
/// passing it to `f` to get a computation of type `m b`.
///
/// Note that `Bind` is a separate typeclass from [`Monad`][`crate::typeclasses::Monad`]. In this library's
/// hierarchy, [`Monad`][`crate::typeclasses::Monad`] is a typeclass that extends both
/// [`Applicative`][`crate::typeclasses::Applicative`] and `Bind`.
pub trait Bind {
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
	fn bind<F, A, B>(ma: Apply<Self, (A,)>) -> impl Fn(F) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)> + Sized,
		Apply<Self, (A,)>: Clone,
		F: Fn(A) -> Apply<Self, (B,)>;
}

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// Free function version that dispatches to [the typeclass method][`Bind::bind`].
///
/// # Type Signature
///
/// `forall m a b. Bind m => m a -> (a -> m b) -> m b`
///
/// # Examples
///
/// ```
/// use fp_library::{brands::OptionBrand, functions::{bind, pure}};
///
/// assert_eq!(bind::<OptionBrand, _, _, _>(Some(5))(|x| Some(x * 2)), Some(10));
/// ```
pub fn bind<Brand, F, A, B>(ma: Apply<Brand, (A,)>) -> impl Fn(F) -> Apply<Brand, (B,)>
where
	Brand: Kind<(A,)> + Kind<(B,)> + Bind,
	Apply<Brand, (A,)>: Clone,
	F: Fn(A) -> Apply<Brand, (B,)>,
{
	move |f| Brand::bind::<F, A, B>(ma.to_owned())(f)
}
