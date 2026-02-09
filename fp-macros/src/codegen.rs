//! Code generation utilities.
//!
//! This module provides code generation functionality including:
//! - Re-export generation for functions and traits

pub mod re_export;

pub use re_export::{
	ReExportInput, generate_function_re_exports_worker, generate_trait_re_exports_worker,
};
