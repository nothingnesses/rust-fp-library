//! Hindley-Milner type conversion subsystem.
//!
//! This module handles converting Rust types to Hindley-Milner representations:
//! - AST definition for HM types
//! - Conversion logic from Rust types
//! - Visitor patterns for type transformation
//! - Pattern detection (FnBrand, Apply!)

pub mod ast;
pub mod converter;
pub mod hm_ast_builder;
pub mod patterns;

pub use ast::HMAST;
pub use converter::type_to_hm;
pub use patterns::{FnBrandInfo, extract_apply_macro_info, extract_fn_brand_info};
