//! Result type helpers and conversion traits.

use proc_macro2::TokenStream;

pub use crate::core::error_handling::{Error, Result};

/// Trait for converting errors to compile-time errors
pub trait ToCompileError {
	fn to_compile_error(self) -> TokenStream;
}

impl ToCompileError for Error {
	fn to_compile_error(self) -> TokenStream {
		let syn_error: syn::Error = self.into();
		syn_error.to_compile_error()
	}
}

impl<T> ToCompileError for Result<T> {
	fn to_compile_error(self) -> TokenStream {
		match self {
			Ok(_) => panic!("Called to_compile_error on Ok value"),
			Err(e) => e.to_compile_error(),
		}
	}
}

/// Extension trait for Result with macro-specific operations
#[allow(dead_code)] // For future use
pub trait ResultExt<T> {
	/// Convert Result to TokenStream, returning output on success or error on failure
	fn into_token_stream(self) -> TokenStream;
}

impl<T: quote::ToTokens> ResultExt<T> for Result<T> {
	fn into_token_stream(self) -> TokenStream {
		match self {
			Ok(value) => quote::quote!(#value),
			Err(e) => e.to_compile_error(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use proc_macro2::Span;

	#[test]
	fn test_to_compile_error() {
		let err = Error::validation(Span::call_site(), "test error");
		let token_stream = err.to_compile_error();
		let output = token_stream.to_string();
		assert!(!output.is_empty());
	}

	#[test]
	#[should_panic(expected = "Called to_compile_error on Ok value")]
	fn test_result_ok_to_compile_error_panics() {
		let result: Result<i32> = Ok(42);
		let _ = result.to_compile_error();
	}
}
