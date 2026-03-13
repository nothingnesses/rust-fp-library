//! Applicative do-notation macro.
//!
//! Provides the [`ado!`](crate::ado) macro for applicative syntax that desugars
//! into [`map`] / [`lift2`]–[`lift5`] calls, matching PureScript `ado` notation.

pub mod codegen;

pub use codegen::ado_worker;
