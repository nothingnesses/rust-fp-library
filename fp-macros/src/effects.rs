//! Procedural macros scoped to the effects subsystem (Run / handlers / ...).
//!
//! Houses:
//!
//! - [`im_do!`](crate::im_do): inherent-method-dispatched monadic
//!   do-notation for the six Run wrappers (Phase 2 step 7c.2b).
//! - [`effects!`](crate::effects) and the internal `raw_effects!`
//!   (Phase 2 step 8): right-nested
//!   [`CoproductBrand`](https://docs.rs/fp-library/latest/fp_library/brands/struct.CoproductBrand.html)
//!   row construction with lexical sorting.
//! - [`handlers!`](crate::handlers) (Phase 3 step 1): right-nested
//!   [`HandlersCons`](https://docs.rs/fp-library/latest/fp_library/types/effects/handlers/struct.HandlersCons.html)
//!   handler-list construction with lexical sorting matching
//!   `effects!`.
//! - [`row_sort`]: shared lexical-sort helper for `effects!` and the
//!   future `scoped_effects!` (Phase 4 step 4).
//!
//! Future macros in this subsystem (`define_effect!`,
//! `define_scoped_effect!`, `scoped_effects!`, and the
//! forward-reserved `ia_do!`) will land here per the
//! [implementation plan](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/plan.md).

pub mod effects_macro;
pub mod handlers;
pub mod im_do;
pub mod row_sort;
