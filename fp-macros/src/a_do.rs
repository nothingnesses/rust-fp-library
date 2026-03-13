//! Applicative do-notation macro.
//!
//! Provides the [`a_do!`](crate::a_do) macro for applicative syntax that desugars
//! into [`map`] / [`lift2`]–[`lift5`] calls, matching PureScript `ado` notation.

pub mod codegen;

pub use codegen::a_do_worker;
