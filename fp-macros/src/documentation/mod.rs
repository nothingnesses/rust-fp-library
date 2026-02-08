//! Documentation generation subsystem.
//!
//! This module provides macros for generating documentation:
//! - #[hm_signature] - Hindley-Milner signatures
//! - #[doc_params] - Parameter documentation
//! - #[doc_type_params] - Type parameter documentation
//! - #[document_module] - Module-level orchestration

pub mod hm_signature;
pub mod doc_params;
pub mod doc_type_params;
pub mod document_module;
pub mod generation;
pub mod templates;

pub use hm_signature::hm_signature_impl;
pub use doc_params::doc_params_impl;
pub use doc_type_params::doc_type_params_impl;
pub use document_module::document_module_impl;
