//! Documentation generation subsystem.
//!
//! This module provides macros for generating documentation:
//! - #[document_signature] - Hindley-Milner signatures
//! - #[document_parameters] - Parameter documentation
//! - #[document_type_parameters] - Type parameter documentation
//! - #[document_fields] - Field documentation
//! - #[document_module] - Module-level orchestration

pub mod document_fields;
pub mod document_module;
pub mod document_parameters;
pub mod document_signature;
pub mod document_type_parameters;
pub mod generation;
pub mod templates;

pub use {
	document_fields::document_fields_worker,
	document_module::document_module_worker,
	document_parameters::document_parameters_worker,
	document_signature::document_signature_worker,
	document_type_parameters::document_type_parameters_worker,
};
