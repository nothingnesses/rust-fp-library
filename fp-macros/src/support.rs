//! Support utilities for procedural macros.
//!
//! This module provides reusable infrastructure for attribute parsing, input validation,
//! and syntax tree manipulation specifically tailored for the library's macro system.
//!
//! It is divided into several sub-modules:
//! - [`attributes`]: Utilities for parsing and filtering attributes, including doc-specific ones.
//! - [`parsing`]: Common parsing patterns for `syn` and input validation helpers.
//! - [`ast`]: RustAst enum for representing various Rust syntax items.
//! - [`documentation_parameters`]: Documentation argument parsing for documentation macros.
//! - [`generate_documentation`]: Documentation comment generation utilities.
//! - [`get_parameters`]: Logical parameter extraction from function signatures.
//! - [`type_visitor`]: Trait for traversing and transforming Rust type syntax trees.
//! - [`document_field`]: Unified field documentation generation for structs and enum variants.
//! - [`method_utils`]: Utilities for analyzing methods and impl blocks.

pub mod ast;
pub mod attributes;
pub mod document_field;
pub mod documentation_parameters;
pub mod generate_documentation;
pub mod get_parameters;
pub mod method_utils;
pub mod parsing;
pub mod type_visitor;

// Re-export commonly used items
pub use {
	get_parameters::{Parameter, get_parameters, is_phantom_data, last_path_segment},
	method_utils::{has_receiver, impl_has_receiver_methods},
	type_visitor::TypeVisitor,
};
