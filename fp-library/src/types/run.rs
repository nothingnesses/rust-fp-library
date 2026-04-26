//! Effects subsystem: row-polymorphic first-order effects and heftia-style
//! scoped effects.
//!
//! See [decisions.md](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/decisions.md)
//! for the design rationale, and `fp-library/docs/run.md` (planned for
//! Phase 5 step 4) for the user guide.
//!
//! ## Submodules
//!
//! - [`coproduct`]: Re-export adapter over [`frunk_core::coproduct`],
//!   surfacing the row-encoding types and trait family the rest of the
//!   subsystem consumes.
//! - [`variant_f`]: [`Functor`](crate::classes::Functor) impls for the
//!   Coproduct-row brands [`CNilBrand`](crate::brands::CNilBrand) and
//!   [`CoproductBrand`](crate::brands::CoproductBrand), plus the
//!   [`VariantF`](variant_f::VariantF) alias. This is the open sum of
//!   first-order effect functors that PureScript spells `VariantF`.

pub mod coproduct;
pub mod variant_f;
