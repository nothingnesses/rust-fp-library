//! Inherent-method-dispatched monadic do-notation.
//!
//! Provides the [`im_do!`](crate::im_do) macro, which desugars do-notation
//! to inherent method calls (`expr.bind(|x| ...)` / `expr.ref_bind(|x| ...)`)
//! on the six Run wrappers. This is the dispatch path used when a wrapper's
//! brand cannot satisfy the brand-level `Semimonad` / `RefSemimonad`
//! cascade (e.g., the Erased Run family, or canonical Coyoneda-headed
//! effect rows under `ref` dispatch).
//!
//! Input parsing is shared with the other do-notation macros via
//! [`crate::support::do_input`](crate::support::do_input). Codegen lives
//! in [`codegen`] and reuses the brand-agnostic helpers from
//! [`crate::m_do::codegen`](crate::m_do::codegen) (`format_bind_param`,
//! `format_discard_param`); the `pure`-rewriting path is `im_do!`-specific
//! because it targets inherent associated functions
//! (`Wrapper::pure(x)` / `Wrapper::ref_pure(&x)`) rather than the
//! brand-dispatched free function `pure::<Brand, _>(x)`.

pub mod codegen;

pub use codegen::im_do_worker;
