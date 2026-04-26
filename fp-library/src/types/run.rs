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
//!   subsystem consumes. The Brand-level integration for the Coproduct
//!   row lands at `crate::brands::CoproductBrand` (Phase 2 step 2).

pub mod coproduct;
