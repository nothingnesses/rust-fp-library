//! Hindley-Milner type conversion subsystem.
//!
//! This module handles converting Rust types to Hindley-Milner representations:
//! - AST definition for HM types
//! - Conversion logic from Rust types
//! - Visitor patterns for type transformation
//! - Pattern detection (FnBrand, Apply!)
//! - Canonicalization and name generation

pub mod ast;
pub mod converter;
pub mod patterns;
pub mod transformations;
pub mod visitors;

pub use ast::HMType;
pub use converter::type_to_hm;
pub use patterns::{extract_apply_macro_info, extract_fn_brand_info, KindAssocTypeInput, KindInput};
pub use transformations::generate_name;

// Only needed for tests
#[cfg(test)]
pub use transformations::Canonicalizer;
