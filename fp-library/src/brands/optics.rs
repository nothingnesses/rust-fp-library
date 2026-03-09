//! Brand types for optics profunctors.
//!
//! These zero-sized types serve as type-level witnesses for the [`Kind`](crate::kinds)
//! trait, enabling higher-kinded polymorphism over optics profunctors.

#[fp_macros::document_module]
mod inner {
	use {
		fp_macros::*,
		std::marker::PhantomData,
	};

	/// Brand for the [`Bazaar`](crate::types::optics::Bazaar) profunctor.
	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of focus values extracted from the source.",
		"The type of replacement values used during reconstruction."
	)]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct BazaarBrand<FunctionBrand, A, B>(PhantomData<(FunctionBrand, A, B)>);

	/// Brand for the [`BazaarList`](crate::types::optics::BazaarList) applicative.
	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of focus values extracted from the source.",
		"The type of replacement values used during reconstruction."
	)]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct BazaarListBrand<FunctionBrand, A, B>(PhantomData<(FunctionBrand, A, B)>);

	/// Brand for the [`Exchange`](crate::types::optics::Exchange) profunctor.
	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the forward function.",
		"The type of the value consumed by the backward function."
	)]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct ExchangeBrand<FunctionBrand, A, B>(PhantomData<(FunctionBrand, A, B)>);

	/// Brand for the [`Forget`](crate::types::optics::Forget) profunctor.
	#[document_type_parameters("The pointer brand.", "The return type of the function.")]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct ForgetBrand<PointerBrand, R>(PhantomData<(PointerBrand, R)>);

	/// Brand for the [`Grating`](crate::types::optics::Grating) profunctor.
	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the inner function.",
		"The type of the value consumed by the inner function."
	)]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct GratingBrand<FunctionBrand, A, B>(PhantomData<(FunctionBrand, A, B)>);

	/// Brand for the [`Indexed`](crate::types::optics::Indexed) profunctor wrapper.
	#[document_type_parameters("The underlying profunctor brand.", "The index type.")]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct IndexedBrand<P, I>(PhantomData<(P, I)>);

	/// Brand for the [`Market`](crate::types::optics::Market) profunctor.
	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the review function."
	)]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct MarketBrand<FunctionBrand, A, B>(PhantomData<(FunctionBrand, A, B)>);

	/// Brand for the [`Re`](crate::types::optics::Re) profunctor.
	///
	/// `ReBrand<InnerP, PointerBrand, S, T>` fixes the inner profunctor `InnerP` and the outer
	/// types `S` and `T`, leaving `A` and `B` free for kind application.
	#[document_type_parameters(
		"The inner profunctor brand whose instances are reversed.",
		"The outer cloneable function pointer brand for wrapping the `run` function.",
		"The fixed source type.",
		"The fixed target type."
	)]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct ReBrand<InnerP, PointerBrand, S, T>(PhantomData<(InnerP, PointerBrand, S, T)>);

	/// Brand for the [`Shop`](crate::types::optics::Shop) profunctor.
	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the getter.",
		"The type of the value consumed by the setter."
	)]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct ShopBrand<FunctionBrand, A, B>(PhantomData<(FunctionBrand, A, B)>);

	/// Brand for the [`Stall`](crate::types::optics::Stall) profunctor.
	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the setter."
	)]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct StallBrand<FunctionBrand, A, B>(PhantomData<(FunctionBrand, A, B)>);

	/// Brand for the [`Tagged`](crate::types::optics::Tagged) profunctor.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct TaggedBrand;

	/// Brand for the [`Zipping`](crate::types::optics::Zipping) profunctor.
	#[document_type_parameters("The cloneable function brand.")]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct ZippingBrand<FunctionBrand>(PhantomData<FunctionBrand>);
}

pub use inner::*;
