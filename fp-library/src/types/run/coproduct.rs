//! Brand-aware adapter layer over [`frunk_core::coproduct`].
//!
//! This module hosts the integration point between
//! [`frunk_core`](https://docs.rs/frunk_core)'s row-encoding machinery
//! (`Coproduct`, `CNil`, plus `CoprodInjector` / `CoprodUninjector` /
//! `CoproductSubsetter` / `CoproductEmbedder`) and fp-library's own
//! `Brand` and `Kind_*` system.
//!
//! ## Why this layer exists
//!
//! Implementing fp-library's own traits (e.g.,
//! [`Functor`](crate::classes::Functor)) directly on
//! [`frunk_core::coproduct::Coproduct`] is permitted by the orphan rules
//! (own-trait, foreign-type) and is the preferred shape for the recursive
//! row-`Functor` dispatch (see `variant_f.rs`, Phase 2 step 2). Conversely,
//! adding a Brand for the Coproduct (whose `impl_kind!` registration
//! would touch the foreign type as Brand) requires a local newtype to
//! satisfy the orphan rules. [`BrandedCoproduct`] and [`BrandedCNil`]
//! provide that local hook.
//!
//! ## Frunk trait names
//!
//! frunk_core 0.4 names these traits after the Coproduct operations they
//! perform: `CoprodInjector` (inject a single type), `CoprodUninjector`
//! (the inverse: pluck), `CoproductSubsetter` (sculpt to a subset),
//! `CoproductEmbedder` (embed into a superset). The
//! [effects plan](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/plan.md)
//! refers to these by their HList-style names ("Plucker" / "Sculptor" /
//! "Embedder"); the Coproduct-style names are what frunk_core actually
//! exports, and what this module bridges.
//!
//! ## Scope
//!
//! Per the implementation note in
//! [plan.md](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/plan.md),
//! the adapter is intentionally thin (target: under approximately 200
//! lines). Bridge impls land here as concrete Phase 2 / 3 / 4 use cases
//! surface them; the initial version covers the minimum to round-trip
//! between branded and unbranded Coproducts and demonstrates the
//! delegation pattern via [`CoprodInjector`] and [`CoprodUninjector`]
//! impls on [`BrandedCoproduct`].

pub use frunk_core::{
	HList,
	coproduct::{
		CNil,
		CoprodInjector,
		CoprodUninjector,
		Coproduct,
		CoproductEmbedder,
		CoproductSelector,
		CoproductSubsetter,
		CoproductTaker,
	},
	hlist::{
		HCons,
		HNil,
	},
	indices::{
		Here,
		There,
	},
};

/// Brand-aware newtype wrapper around [`frunk_core::coproduct::Coproduct`].
///
/// Stores the inner [`Coproduct<H, T>`] as a transparent field so the
/// in-memory layout matches the wrapped type. The wrapper exists so that
/// `Brand`-style trait impls (e.g., [`Kind`](crate::kinds::Kind_cdc7cd43dac7585f))
/// can target a local type, satisfying the orphan rules. Trait impls that
/// can target the foreign [`Coproduct`] directly (own-trait, foreign-type;
/// e.g., [`Functor`](crate::classes::Functor) in `variant_f.rs`) do not
/// need the wrapper.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrandedCoproduct<H, T>(pub Coproduct<H, T>);

/// Brand-aware newtype wrapper around the empty coproduct
/// [`frunk_core::coproduct::CNil`].
///
/// Like [`BrandedCoproduct`], this wrapper is the local hook for any
/// Brand-style trait impl that needs a base case alongside the cons
/// case. [`CNil`] is uninhabited, so values of this wrapper never exist
/// at runtime; the type is meaningful only at the type level.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrandedCNil(pub CNil);

impl<H, T> From<Coproduct<H, T>> for BrandedCoproduct<H, T> {
	fn from(inner: Coproduct<H, T>) -> Self {
		BrandedCoproduct(inner)
	}
}

impl<H, T> From<BrandedCoproduct<H, T>> for Coproduct<H, T> {
	fn from(branded: BrandedCoproduct<H, T>) -> Self {
		branded.0
	}
}

impl From<CNil> for BrandedCNil {
	fn from(inner: CNil) -> Self {
		BrandedCNil(inner)
	}
}

impl From<BrandedCNil> for CNil {
	fn from(branded: BrandedCNil) -> Self {
		branded.0
	}
}

// -- Frunk trait bridges on BrandedCoproduct --
//
// Each impl delegates to the inner Coproduct. Additional bridges
// (CoproductSubsetter, CoproductEmbedder, CoproductSelector,
// CoproductTaker, CoproductFoldable, CoproductMappable) land here as
// concrete Phase 2 / 3 / 4 use cases need them.

impl<H, T, I, Idx> CoprodInjector<I, Idx> for BrandedCoproduct<H, T>
where
	Coproduct<H, T>: CoprodInjector<I, Idx>,
{
	fn inject(to_insert: I) -> Self {
		BrandedCoproduct(<Coproduct<H, T> as CoprodInjector<I, Idx>>::inject(to_insert))
	}
}

impl<H, T, U, Idx> CoprodUninjector<U, Idx> for BrandedCoproduct<H, T>
where
	Coproduct<H, T>: CoprodUninjector<U, Idx>,
{
	type Remainder = <Coproduct<H, T> as CoprodUninjector<U, Idx>>::Remainder;

	fn uninject(self) -> Result<U, Self::Remainder> {
		<Coproduct<H, T> as CoprodUninjector<U, Idx>>::uninject(self.0)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	type Row = Coproduct<i32, Coproduct<&'static str, CNil>>;

	#[test]
	fn round_trip_branded_coproduct() {
		let raw: Row = Coproduct::inject(42_i32);
		let branded: BrandedCoproduct<_, _> = raw.into();
		let recovered: Row = branded.into();
		assert_eq!(recovered, Coproduct::Inl(42));
	}

	#[test]
	fn inject_via_branded_coproduct() {
		let branded: BrandedCoproduct<i32, Coproduct<&'static str, CNil>> = <BrandedCoproduct<
			i32,
			Coproduct<&'static str, CNil>,
		> as CoprodInjector<
			i32,
			Here,
		>>::inject(7);
		assert_eq!(branded.0, Coproduct::Inl(7));
	}

	#[test]
	fn uninject_via_branded_coproduct_present() {
		let raw: Row = Coproduct::inject(123_i32);
		let branded: BrandedCoproduct<_, _> = raw.into();
		let result: Result<i32, Coproduct<&'static str, CNil>> =
			<BrandedCoproduct<_, _> as CoprodUninjector<i32, Here>>::uninject(branded);
		assert_eq!(result, Ok(123));
	}

	#[test]
	fn uninject_via_branded_coproduct_absent() {
		let raw: Row = Coproduct::inject("present");
		let branded: BrandedCoproduct<_, _> = raw.into();
		let result: Result<i32, Coproduct<&'static str, CNil>> =
			<BrandedCoproduct<_, _> as CoprodUninjector<i32, Here>>::uninject(branded);
		assert!(result.is_err());
	}
}
