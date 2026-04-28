//! Procedural macros scoped to the effects subsystem (Run / handlers / ...).
//!
//! Currently houses [`im_do!`](crate::im_do), the inherent-method-dispatched
//! monadic do-notation macro for the six Run wrappers. Future macros in this
//! subsystem (`effects!`, `effects_coyo!`, `handlers!`, `define_effect!`,
//! `define_scoped_effect!`, `scoped_effects!`, and the forward-reserved
//! `ia_do!`) will land here per the
//! [implementation plan](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/plan.md).

pub mod im_do;
