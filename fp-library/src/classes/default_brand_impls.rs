//! [`DefaultBrand`] implementations for all unambiguous types.
//!
//! These implementations provide the reverse mapping from concrete types to
//! their canonical brands, enabling brand inference for free functions.
//!
//! Types with multiple brands at this arity (Result, Pair, Tuple2,
//! ControlFlow, TryThunk) do NOT implement `DefaultBrand` and require
//! explicit brand specification.

mod inner {
	use crate::{
		brands::*,
		classes::default_brand::DefaultBrand,
		types::*,
	};

	// -- Core types (simple, non-parameterized brands) --

	impl<A> DefaultBrand for Option<A> {
		type Brand = OptionBrand;
	}

	impl<A> DefaultBrand for Vec<A> {
		type Brand = VecBrand;
	}

	impl<A> DefaultBrand for Identity<A> {
		type Brand = IdentityBrand;
	}

	impl<'a, A: 'a> DefaultBrand for Thunk<'a, A> {
		type Brand = ThunkBrand;
	}

	impl<'a, A: 'a> DefaultBrand for SendThunk<'a, A> {
		type Brand = SendThunkBrand;
	}

	impl<A> DefaultBrand for CatList<A> {
		type Brand = CatListBrand;
	}

	impl<A> DefaultBrand for (A,) {
		type Brand = Tuple1Brand;
	}

	// -- Parameterized brands --

	impl<'a, A: 'a, Config: crate::classes::LazyConfig + 'a> DefaultBrand for Lazy<'a, A, Config> {
		type Brand = LazyBrand<Config>;
	}

	impl<'a, A: 'a, E: 'static, Config: crate::classes::LazyConfig + 'a> DefaultBrand
		for TryLazy<'a, A, E, Config>
	{
		type Brand = TryLazyBrand<E, Config>;
	}

	impl<'a, F: crate::kinds::Kind_cdc7cd43dac7585f + 'static, A: 'a> DefaultBrand
		for Coyoneda<'a, F, A>
	{
		type Brand = CoyonedaBrand<F>;
	}

	impl<'a, F: crate::kinds::Kind_cdc7cd43dac7585f + 'static, A: 'a> DefaultBrand
		for RcCoyoneda<'a, F, A>
	{
		type Brand = RcCoyonedaBrand<F>;
	}

	impl<'a, F: crate::kinds::Kind_cdc7cd43dac7585f + 'static, A: 'a> DefaultBrand
		for ArcCoyoneda<'a, F, A>
	{
		type Brand = ArcCoyonedaBrand<F>;
	}

	impl<'a, F: crate::kinds::Kind_cdc7cd43dac7585f + 'static, B: 'static, A: 'a> DefaultBrand
		for BoxedCoyonedaExplicit<'a, F, B, A>
	{
		type Brand = CoyonedaExplicitBrand<F, B>;
	}

	impl<'a, R: 'static, A: 'a> DefaultBrand for crate::types::const_val::Const<'a, R, A> {
		type Brand = ConstBrand<R>;
	}
}
