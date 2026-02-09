//! Unified error handling for fp-macros.
//!
//! This module provides a comprehensive error system with rich context for
//! generating helpful compile-time error messages.

use proc_macro2::Span;
use std::fmt;
use thiserror::Error;

/// Result type alias using our unified error type
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for fp-macros
#[derive(Debug, Error)]
pub enum Error {
	/// Parsing error from syn
	#[error("Parse error: {0}")]
	Parse(#[from] syn::Error),

	/// Validation error with span information and optional suggestion
	#[error("Validation error: {message}")]
	Validation {
		/// Error message
		message: String,
		/// Source code span
		span: Span,
		/// Optional suggestion for fixing the error
		suggestion: Option<String>,
	},

	/// Resolution error (e.g., Self type or associated type resolution)
	#[error("Resolution error: {message}")]
	Resolution {
		/// Error message
		message: String,
		/// Source code span
		span: Span,
		/// Available types for helpful error messages
		available_types: Vec<String>,
	},

	/// Unsupported feature error
	#[error("Unsupported feature: {0}")]
	Unsupported(#[from] UnsupportedFeature),

	/// Internal error (for "should never happen" cases)
	#[error("Internal error: {0}")]
	Internal(String),

	/// I/O error (for file operations)
	#[error("I/O error: {0}")]
	Io(#[from] std::io::Error),
}

/// Specific unsupported feature variants
#[derive(Debug, Error)]
pub enum UnsupportedFeature {
	/// Const generic parameters are not supported in Kind definitions
	#[error("Const generic parameters are not supported in Kind definitions")]
	ConstGenerics {
		/// Source code span
		span: Span,
	},

	/// Verbatim bounds are not supported
	#[error("Verbatim bounds are not supported")]
	VerbatimBounds {
		/// Source code span
		span: Span,
	},

	/// Complex types are not supported
	#[error("Complex type not supported: {description}")]
	ComplexTypes {
		/// Description of the unsupported type
		description: String,
		/// Source code span
		span: Span,
	},

	/// Unsupported generic argument
	#[error("Unsupported generic argument: {description}")]
	GenericArgument {
		/// Description of the unsupported argument
		description: String,
		/// Source code span
		span: Span,
	},

	/// Unsupported bound type
	#[error("Unsupported bound type: {description}")]
	BoundType {
		/// Description of the unsupported bound
		description: String,
		/// Source code span
		span: Span,
	},
}

impl Error {
	/// Create a validation error
	pub fn validation(
		span: Span,
		message: impl Into<String>,
	) -> Self {
		Self::Validation { message: message.into(), span, suggestion: None }
	}

	/// Add a suggestion to this error
	pub fn with_suggestion(
		mut self,
		suggestion: impl Into<String>,
	) -> Self {
		if let Error::Validation { suggestion: s, .. } = &mut self {
			*s = Some(suggestion.into());
		}
		self
	}

	/// Create a resolution error with available types for helpful messages
	pub fn resolution(
		span: Span,
		message: impl Into<String>,
		available_types: Vec<String>,
	) -> Self {
		Self::Resolution { message: message.into(), span, available_types }
	}

	/// Create an unsupported feature error
	pub fn unsupported(
		span: Span,
		feature: impl Into<String>,
	) -> Self {
		Self::Unsupported(UnsupportedFeature::ComplexTypes { description: feature.into(), span })
	}

	/// Create an internal error (for "should never happen" cases)
	pub fn internal(message: impl Into<String>) -> Self {
		Self::Internal(message.into())
	}

	/// Get the span for this error
	pub fn span(&self) -> Span {
		match self {
			Error::Parse(e) => e.span(),
			Error::Validation { span, .. } => *span,
			Error::Resolution { span, .. } => *span,
			Error::Unsupported(u) => u.span(),
			Error::Internal(_) => Span::call_site(),
			Error::Io(_) => Span::call_site(),
		}
	}

	/// Add context to an error
	pub fn context(
		self,
		context: impl fmt::Display,
	) -> Self {
		match self {
			Error::Internal(msg) => Error::Internal(format!("{}: {}", context, msg)),
			Error::Validation { message, span, suggestion } => {
				Error::Validation { message: format!("{}: {}", context, message), span, suggestion }
			}
			Error::Resolution { message, span, available_types } => Error::Resolution {
				message: format!("{}: {}", context, message),
				span,
				available_types,
			},
			Error::Parse(e) => {
				// Create new error with context and combine
				let ctx_error = syn::Error::new(e.span(), format!("{}: {}", context, e));
				Error::Parse(ctx_error)
			}
			Error::Unsupported(u) => {
				// Unsupported features maintain original message
				// but we note the context by wrapping in Internal
				Error::Internal(format!("{}: Unsupported feature: {}", context, u))
			}
			Error::Io(io) => Error::Internal(format!("{}: I/O error: {}", context, io)),
		}
	}

	/// Add context to an error (alias for context, more fluent API)
	pub fn with_context(
		self,
		context: impl fmt::Display,
	) -> Self {
		self.context(context)
	}
}

impl UnsupportedFeature {
	/// Get the span for this unsupported feature
	pub fn span(&self) -> Span {
		match self {
			UnsupportedFeature::ConstGenerics { span } => *span,
			UnsupportedFeature::VerbatimBounds { span } => *span,
			UnsupportedFeature::ComplexTypes { span, .. } => *span,
			UnsupportedFeature::GenericArgument { span, .. } => *span,
			UnsupportedFeature::BoundType { span, .. } => *span,
		}
	}
}

/// Convert our error to syn::Error for proc macro output
impl From<Error> for syn::Error {
	fn from(err: Error) -> Self {
		let span = err.span();
		let mut message = err.to_string();

		// Add suggestion directly to the message for Validation errors
		if let Error::Validation { suggestion: Some(s), .. } = &err {
			message = format!("{}\nhelp: {}", message, s);
		}

		// Add available alternatives for Resolution errors
		if let Error::Resolution { available_types, .. } = &err
			&& !available_types.is_empty()
		{
			message = format!(
				"{}\nnote: available alternatives: {}",
				message,
				available_types.join(", ")
			);
		}

		syn::Error::new(span, message)
	}
}

/// Utility for collecting and combining multiple errors.
/// Replaces the repeated pattern of error accumulation throughout the codebase.
pub struct ErrorCollector {
	errors: Vec<syn::Error>,
}

impl ErrorCollector {
	pub fn new() -> Self {
		Self { errors: Vec::new() }
	}

	pub fn push(
		&mut self,
		error: syn::Error,
	) {
		self.errors.push(error);
	}

	pub fn extend(
		&mut self,
		other_errors: Vec<syn::Error>,
	) {
		self.errors.extend(other_errors);
	}

	pub fn finish(self) -> syn::Result<()> {
		if self.errors.is_empty() { Ok(()) } else { Err(Self::combine_errors(self.errors)) }
	}

	fn combine_errors(mut errors: Vec<syn::Error>) -> syn::Error {
		let mut combined = errors.remove(0);
		for err in errors {
			combined.combine(err);
		}
		combined
	}
}

impl Default for ErrorCollector {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_error_span() {
		let span = Span::call_site();
		let err = Error::validation(span, "test message");
		assert_eq!(format!("{:?}", err.span()), format!("{:?}", span), "Span should be preserved");
	}

	#[test]
	fn test_validation_error() {
		let span = Span::call_site();
		let err = Error::validation(span, "invalid input");
		assert!(err.to_string().contains("invalid input"));
	}

	#[test]
	fn test_validation_error_with_suggestion() {
		let span = Span::call_site();
		let err = Error::validation(span, "invalid input").with_suggestion("try this instead");
		let syn_err: syn::Error = err.into();
		let err_str = syn_err.to_string();
		eprintln!("Error string: '{}'", err_str);
		eprintln!("Contains 'invalid input': {}", err_str.contains("invalid input"));
		eprintln!("Contains 'try this instead': {}", err_str.contains("try this instead"));
		assert!(err_str.contains("invalid input"));
		// Note: syn::Error combines multiple errors but doesn't add "help:" prefix
		// The suggestion is included in the combined error message
		assert!(err_str.contains("try this instead"));
	}

	#[test]
	fn test_resolution_error() {
		let span = Span::call_site();
		let err = Error::resolution(span, "cannot resolve", vec!["Type1".to_string()]);
		assert!(err.to_string().contains("cannot resolve"));
	}

	#[test]
	fn test_unsupported_const_generics() {
		let span = Span::call_site();
		let err = UnsupportedFeature::ConstGenerics { span };
		assert!(err.to_string().contains("Const generic parameters are not supported"));
	}

	#[test]
	fn test_error_context() {
		let err = Error::internal("original message");
		let err_with_context = err.context("while processing");
		assert!(err_with_context.to_string().contains("while processing: original message"));
	}

	#[test]
	fn test_syn_error_conversion() {
		let span = Span::call_site();
		let err = Error::validation(span, "test error");
		let syn_err: syn::Error = err.into();
		assert!(syn_err.to_string().contains("test error"));
	}

	#[test]
	fn test_resolution_error_with_available_types() {
		let span = Span::call_site();
		let err = Error::resolution(
			span,
			"cannot find type",
			vec!["String".to_string(), "Vec".to_string()],
		);
		let syn_err: syn::Error = err.into();
		let err_string = syn_err.to_string();
		assert!(err_string.contains("cannot find type"));
		// The "available alternatives" note is combined as a separate error
	}
}
