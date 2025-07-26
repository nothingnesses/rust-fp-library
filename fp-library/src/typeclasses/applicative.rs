use crate::typeclasses::{Apply, ApplyFirst, ApplySecond, Functor, Pure};

pub trait Applicative: Functor + Pure + Apply + ApplyFirst + ApplySecond {}

/// Blanket implementation for the [`Applicative`] typeclass.
///
/// Any type that implements all the required supertraits automatically implements [`Applicative`].
impl<T> Applicative for T where T: Functor + Pure + Apply + ApplyFirst + ApplySecond {}

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
