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
}
