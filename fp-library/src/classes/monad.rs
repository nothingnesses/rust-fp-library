use crate::classes::{Applicative, Bind};

/// A type class for monads.
///
/// `Monad` combines the capabilities of [`Applicative`] and [`Bind`], providing
/// a powerful abstraction for sequencing computations with context.
///
/// Monads are more powerful than applicative functors because they allow
/// the structure of subsequent computations to depend on the results of
/// previous computations.
///
/// # Laws
///
/// Monad instances must satisfy the following laws:
/// * Left identity: `bind(pure(a))(f) = f(a)`.
/// * Right identity: `bind(m)(pure) = m`.
/// * Associativity: `bind(bind(m)(f))(g) = bind(m)(x => bind(f(x))(g))`.
pub trait Monad: Applicative + Bind {}

/// Blanket implementation for the `Monad` type class.
///
/// Any type that implements all the required supertraits automatically implements `Monad`.
impl<Brand> Monad for Brand where Brand: Applicative + Bind {}

#[cfg(test)]
mod tests {
	use crate::{
		brands::{IdentityBrand, OptionBrand, ResultWithErrBrand, ResultWithOkBrand, VecBrand},
		classes::Monad,
	};

	/// Asserts that a type implements [`Monad`].
	fn assert_monad<T: Monad>() {}

	#[test]
	/// Assert that brands implementing the required supertraits
	/// ([`Applicative`][crate::classes::Applicative], [`Bind`][crate::classes::Bind])
	/// also implement [`Monad`].
	fn test_brands_implement_monad() {
		assert_monad::<IdentityBrand>();
		assert_monad::<OptionBrand>();
		assert_monad::<ResultWithErrBrand<()>>();
		assert_monad::<ResultWithOkBrand<()>>();
		assert_monad::<VecBrand>();
	}
}
