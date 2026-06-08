//! Compile-pass test: validation warnings must not block compilation.
//!
//! This module has undocumented items inside a `#[document_module]` (with
//! validation enabled). The validation pass emits warnings via `#[deprecated]`
//! but the code must still compile successfully.

#![expect(deprecated, reason = "This fixture intentionally triggers validation warnings.")]

use fp_macros::document_module;

#[document_module]
mod validated {
	pub struct MyType;

	impl MyType {
		pub fn new() -> Self {
			Self
		}

		#[expect(dead_code, reason = "Test fixture for document_module macro")]
		pub fn process<T>(
			&self,
			_value: T,
		) -> T {
			_value
		}
	}
}

fn main() {
	let _ = validated::MyType::new();
}
