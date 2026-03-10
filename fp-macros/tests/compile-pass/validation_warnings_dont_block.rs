//! Compile-pass test: validation warnings must not block compilation.
//!
//! This module has undocumented items inside a `#[document_module]` (with
//! validation enabled). The validation pass emits warnings via `#[deprecated]`
//! but the code must still compile successfully.

use fp_macros::document_module;

#[document_module]
#[allow(deprecated)]
mod validated {
	pub struct MyType;

	impl MyType {
		pub fn new() -> Self {
			Self
		}

		#[allow(dead_code)]
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
