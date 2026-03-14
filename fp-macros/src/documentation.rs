//! Documentation generation subsystem.
//!
//! This module provides macros for generating documentation:
//! - #[document_signature] - Hindley-Milner signatures
//! - #[document_parameters] - Parameter documentation
//! - #[document_type_parameters] - Type parameter documentation
//! - #[document_module] - Module-level orchestration

pub mod document_examples;
pub mod document_module;
pub mod document_parameters;
pub mod document_returns;
pub mod document_signature;
pub mod document_type_parameters;
pub mod generation;
pub mod templates;

pub use {
	document_examples::document_examples_worker,
	document_module::document_module_worker,
	document_parameters::document_parameters_worker,
	document_returns::document_returns_worker,
	document_signature::document_signature_worker,
	document_type_parameters::document_type_parameters_worker,
};
