use crate::typeclasses::{Applicative, Bind};

pub trait Monad: Applicative + Bind {}

/// Blanket implementation for the [`Monad`] typeclass.
///
/// Any type that implements all the required supertraits automatically implements [`Monad`].
impl<T> Monad for T where T: Applicative + Bind {}

#[cfg(test)]
mod tests {
	use crate::{
		brands::OptionBrand,
		typeclasses::Monad,
		types::{ResultWithErrBrand, ResultWithOkBrand, SoloBrand, VecBrand},
	};

	/// Asserts that a type implements [`Monad`].
	fn assert_monad<T: Monad>() {}

	#[test]
	/// Assert that brands implementing the required supertraits
	/// ([`Applicative`], [`Bind`]) also implement [`Monad`].
	fn test_brands_implement_monad() {
		assert_monad::<SoloBrand>();
		assert_monad::<OptionBrand>();
		assert_monad::<ResultWithErrBrand<()>>();
		assert_monad::<ResultWithOkBrand<()>>();
		assert_monad::<VecBrand>();
	}
}
