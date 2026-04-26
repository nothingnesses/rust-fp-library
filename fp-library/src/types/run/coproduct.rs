//! Re-export adapter over [`frunk_core::coproduct`].
//!
//! Surfaces the row-encoding types and trait family the rest of the
//! effects subsystem (Phase 2 / 3 / 4) consumes:
//! [`Coproduct`], [`CNil`], plus the [`CoprodInjector`] /
//! [`CoprodUninjector`] / [`CoproductSubsetter`] / [`CoproductEmbedder`]
//! / [`CoproductSelector`] / [`CoproductTaker`] traits, and the
//! [`Here`] / [`There`] / [`HCons`] / [`HNil`] index types.
//!
//! ## Bridging to fp-library's Brand system
//!
//! No newtype wrapper around [`Coproduct`] is needed at this layer. The
//! Brand-level integration lands at `crate::brands::CoproductBrand`
//! (Phase 2 step 2): a local Brand parameterised by head and tail Brand
//! types, with `Of<'a, A>` resolving to
//! `Coproduct<H::Of<'a, A>, T::Of<'a, A>>`. Because `Kind_*` is
//! fp-library's trait, the Brand impl satisfies the orphan rules
//! (own-trait + local-Brand-type) without any wrapper.
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
//! exports, and what this module surfaces.

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

#[cfg(test)]
mod tests {
	use super::*;

	type Row = Coproduct<i32, Coproduct<&'static str, CNil>>;

	#[test]
	fn inject_and_uninject_present() {
		let raw: Row = Coproduct::inject(123_i32);
		let result: Result<i32, Coproduct<&'static str, CNil>> = raw.uninject();
		assert_eq!(result, Ok(123));
	}

	#[test]
	fn uninject_absent_returns_remainder() {
		let raw: Row = Coproduct::inject("present");
		let result: Result<i32, Coproduct<&'static str, CNil>> = raw.uninject();
		assert!(result.is_err());
	}
}
