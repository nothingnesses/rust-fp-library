//! Effects subsystem: row-polymorphic first-order effects and heftia-style
//! scoped effects.
//!
//! See [decisions.md](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/decisions.md)
//! for the design rationale, and `fp-library/docs/run.md` (planned for
//! Phase 5 step 4) for the user guide.
//!
//! ## Submodules
//!
//! - [`coproduct`]: Brand-aware adapter layer over
//!   [`frunk_core::coproduct`]. Provides newtype wrappers and bridge
//!   impls so the project's `Brand` system can interact with the
//!   row-encoding machinery.

pub mod coproduct;
