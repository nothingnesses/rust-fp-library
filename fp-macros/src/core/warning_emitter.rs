//! Shared warning emission for proc-macro diagnostics.
//!
//! Uses `proc-macro-warning` to emit compile-time warnings via `#[deprecated]`
//! items, allowing code to continue compiling while surfacing diagnostic messages.

use {
	proc_macro_warning::FormattedWarning,
	proc_macro2::{
		Span,
		TokenStream,
	},
	quote::ToTokens,
};

/// Collects warnings and converts them to token streams for compile-time emission.
///
/// Analogous to `ErrorCollector` but produces warnings (via `#[deprecated]`)
/// instead of `compile_error!` invocations.
pub struct WarningEmitter {
	counter: usize,
	warnings: Vec<TokenStream>,
}

impl WarningEmitter {
	/// Create a new, empty warning emitter.
	pub fn new() -> Self {
		Self {
			counter: 0,
			warnings: Vec::new(),
		}
	}

	/// Emit a warning with the given span and message.
	///
	/// Each warning gets a unique name (`_fp_macros_warning_{counter}`) to avoid
	/// name collisions when multiple warnings are emitted in a single expansion.
	pub fn warn(
		&mut self,
		span: Span,
		message: impl Into<String>,
	) {
		let name = format!("_fp_macros_warning_{}", self.counter);
		self.counter += 1;

		let warning = FormattedWarning::new_deprecated(&name, message, span);
		self.warnings.push(warning.into_token_stream());
	}

	/// Returns `true` if no warnings have been emitted.
	pub fn is_empty(&self) -> bool {
		self.warnings.is_empty()
	}

	/// Consume the emitter and return all warning token streams.
	pub fn into_tokens(self) -> Vec<TokenStream> {
		self.warnings
	}
}

impl Default for WarningEmitter {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_is_empty() {
		let emitter = WarningEmitter::new();
		assert!(emitter.is_empty());
	}

	#[test]
	fn test_warn_makes_nonempty() {
		let mut emitter = WarningEmitter::new();
		emitter.warn(Span::call_site(), "test warning");
		assert!(!emitter.is_empty());
	}

	#[test]
	fn test_into_tokens_empty() {
		let emitter = WarningEmitter::new();
		assert!(emitter.into_tokens().is_empty());
	}

	#[test]
	fn test_into_tokens_count() {
		let mut emitter = WarningEmitter::new();
		emitter.warn(Span::call_site(), "warning 1");
		emitter.warn(Span::call_site(), "warning 2");
		emitter.warn(Span::call_site(), "warning 3");
		assert_eq!(emitter.into_tokens().len(), 3);
	}

	#[test]
	fn test_unique_names() {
		let mut emitter = WarningEmitter::new();
		emitter.warn(Span::call_site(), "first");
		emitter.warn(Span::call_site(), "second");
		emitter.warn(Span::call_site(), "third");

		let tokens = emitter.into_tokens();
		let token_strings: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();

		// Each token stream should contain a distinct _fp_macros_warning_ identifier
		assert!(token_strings[0].contains("_fp_macros_warning_0"));
		assert!(token_strings[1].contains("_fp_macros_warning_1"));
		assert!(token_strings[2].contains("_fp_macros_warning_2"));
	}
}
