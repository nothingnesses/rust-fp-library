//! Hindley-Milner type conversion subsystem.
//!
//! This module handles converting Rust types to Hindley-Milner representations:
//! - AST definition for HM types
//! - Conversion logic from Rust types
//! - Visitor patterns for type transformation

pub mod ast;
pub mod converter;
pub mod hm_ast_builder;

pub use ast::HmAst;
pub use converter::type_to_hm;
