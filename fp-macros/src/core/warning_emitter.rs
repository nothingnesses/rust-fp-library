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
	std::sync::atomic::{
		AtomicUsize,
		Ordering,
	},
};

/// Global counter ensuring unique warning names across all proc-macro invocations.
static GLOBAL_WARNING_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Collects warnings and converts them to token streams for compile-time emission.
///
/// Analogous to `ErrorCollector` but produces warnings (via `#[deprecated]`)
/// instead of `compile_error!` invocations.
pub struct WarningEmitter {
	warnings: Vec<TokenStream>,
}

impl WarningEmitter {
	/// Create a new, empty warning emitter.
	pub fn new() -> Self {
		Self {
			warnings: Vec::new(),
		}
	}

	/// Emit a warning with the given span and message.
	///
	/// Each warning gets a globally unique name (`_fp_macros_warning_{id}`) to avoid
	/// name collisions across multiple macro invocations at the same scope.
	pub fn warn(
		&mut self,
		span: Span,
		message: impl Into<String>,
	) {
		let id = GLOBAL_WARNING_COUNTER.fetch_add(1, Ordering::Relaxed);
		let name = format!("_fp_macros_warning_{id}");

		let warning = FormattedWarning::new_deprecated(&name, message, span);
		self.warnings.push(warning.into_token_stream());
	}

	/// Returns `true` if no warnings have been emitted.
	#[allow(dead_code)]
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
#[expect(
	clippy::indexing_slicing,
	reason = "Tests use panicking operations for brevity and clarity"
)]
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
		assert!(token_strings[0].contains("_fp_macros_warning_"));
		assert!(token_strings[1].contains("_fp_macros_warning_"));
		assert!(token_strings[2].contains("_fp_macros_warning_"));

		// All three should be different
		assert_ne!(token_strings[0], token_strings[1]);
		assert_ne!(token_strings[1], token_strings[2]);
		assert_ne!(token_strings[0], token_strings[2]);
	}
}
