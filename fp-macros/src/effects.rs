//! Procedural macros scoped to the effects subsystem (Run / handlers / ...).
//!
//! Houses:
//!
//! - [`im_do!`](crate::im_do): inherent-method-dispatched monadic
//!   do-notation for the six Run wrappers.
//! - [`effects!`](crate::effects) and the internal `raw_effects!`
//!   macros: right-nested
//!   [`CoproductBrand`](https://docs.rs/fp-library/latest/fp_library/brands/struct.CoproductBrand.html)
//!   row construction with lexical sorting.
//! - [`handlers!`](crate::handlers): right-nested
//!   [`HandlersCons`](https://docs.rs/fp-library/latest/fp_library/types/effects/handlers/struct.HandlersCons.html)
//!   handler-list construction with lexical sorting matching
//!   `effects!`.
//! - [`scoped_effects!`](crate::scoped_effects) and
//!   [`scoped_handlers!`](crate::scoped_handlers):
//!   scoped-row and scoped-handler-list construction with the same
//!   lexical sort.
//! - [`define_scoped_row!`](crate::define_scoped_row): concrete
//!   marker-row item generation for recursive scoped rows.
//! - [`define_effect_row_aliases!`](crate::define_effect_row_aliases):
//!   item-position type aliases for first-order, Rc first-order, Arc
//!   first-order, and scoped rows.
//! - [`row_sort`]: shared lexical-sort helper for `effects!`,
//!   `raw_effects!`, `scoped_effects!`, `define_scoped_row!`, and
//!   `define_effect_row_aliases!`.
//!
//! Future macros in this subsystem (`define_effect!`,
//! `define_scoped_effect!`, and the forward-reserved `ia_do!`) should
//! live here so all Run-related macro code remains grouped by domain.

pub mod effects_macro;
pub mod handlers;
pub mod im_do;
pub mod row_aliases;
pub mod row_sort;
pub mod scoped_row;
