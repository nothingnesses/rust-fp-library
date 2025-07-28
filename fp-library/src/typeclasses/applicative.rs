use crate::typeclasses::{Apply, ApplyFirst, ApplySecond, Functor, Pure};

/// A typeclass for applicative functors.
///
/// `Applicative` extends `Functor` with the ability to lift values into a context
/// (`pure`) and to apply functions within a context to values within a context
/// (`apply`). It also provides additional operations for combining contexts
/// (`apply_first`, `apply_second`).
///
/// Applicative functors are more powerful than functors but less powerful than
/// monads. They allow for sequencing computations but with less flexibility
/// than monads since the structure of the computation must be known in advance.
///
/// # Laws
///
/// Applicative instances must satisfy the following laws:
/// * Identity: `apply(pure(identity))(v) = v`.
/// * Composition: `apply(apply(apply(pure(compose))(u))(v))(w) = apply(u)(apply(v)(w))`.
/// * Homomorphism: `apply(pure(f))(pure(x)) = pure(f(x))`.
/// * Interchange: `apply(u)(pure(y)) = apply(pure(f => f(y)))(u)`.
pub trait Applicative: Functor + Pure + Apply + ApplyFirst + ApplySecond {}

/// Blanket implementation for the [`Applicative`] typeclass.
///
/// Any type that implements all the required supertraits automatically implements [`Applicative`].
///
/// The supertraits are:
/// * [`Functor`]: for mapping functions over values in a context.
/// * [`Pure`]: for lifting values into a context.
/// * [`Apply`]: for applying functions in a context to values in a context.
/// * [`ApplyFirst`]: for combining two contexts, keeping the first value.
/// * [`ApplySecond`]: for combining two contexts, keeping the second value.
impl<Brand> Applicative for Brand where Brand: Functor + Pure + Apply + ApplyFirst + ApplySecond {}

#[cfg(test)]
mod tests {
	use crate::{
		brands::OptionBrand,
		typeclasses::Applicative,
		types::{ResultWithErrBrand, ResultWithOkBrand, SoloBrand, VecBrand},
	};

	/// Asserts that a type implements [`Applicative`].
	fn assert_applicative<T: Applicative>() {}

	#[test]
	/// Assert that brands implementing the required supertraits ([`Functor`],
	/// [`Pure`], [`Apply`], [`ApplyFirst`], [`ApplySecond`]) also implement
	/// [`Applicative`].
	fn test_brands_implement_applicative() {
		assert_applicative::<SoloBrand>();
		assert_applicative::<OptionBrand>();
		assert_applicative::<ResultWithErrBrand<()>>();
		assert_applicative::<ResultWithOkBrand<()>>();
		assert_applicative::<VecBrand>();
	}
}
