//! Hindley-Milner type conversion subsystem.
//!
//! This module handles converting Rust types to Hindley-Milner representations:
//! - AST definition for HM types
//! - Conversion logic from Rust types
//! - Visitor patterns for type transformation

pub mod ast;
pub mod ast_builder;
pub mod converter;

pub use {
	ast::HmAst,
	converter::type_to_hm,
};
