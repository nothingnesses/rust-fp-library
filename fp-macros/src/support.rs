//! Support utilities for procedural macros.
//!
//! This module provides reusable infrastructure for attribute parsing, input validation,
//! and syntax tree manipulation specifically tailored for the library's macro system.
//!
//! It is divided into several sub-modules:
//! - [`attributes`]: Utilities for parsing and filtering attributes, including doc-specific ones.
//! - [`parsing`]: Common parsing patterns for `syn` and input validation helpers.
//! - [`syntax`]: High-level abstractions for working with Rust items and generating documentation.
//! - [`type_visitor`]: Trait for traversing and transforming Rust type syntax trees.

pub mod attributes;
pub mod parsing;
pub mod syntax;
pub mod type_visitor;

// Re-export commonly used items
pub use syntax::{LogicalParam, get_logical_params, is_phantom_data, last_path_segment};
pub use type_visitor::TypeVisitor;
