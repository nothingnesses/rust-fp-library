//! A functional programming library for Rust featuring your favourite higher-kinded types and type classes.

extern crate fp_macros;

pub mod brands;
pub mod classes;
pub mod functions;
pub mod kinds;
pub mod types;

pub use fp_macros::Apply;
pub use fp_macros::Kind;
pub use fp_macros::def_kind;
pub use fp_macros::impl_kind;
