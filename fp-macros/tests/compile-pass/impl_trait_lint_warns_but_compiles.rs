//! Compile-pass test: impl trait lint emits warnings but does not block compilation.
//!
//! The named generic `F` could be `impl Fn(i32) -> i32`. The lint warns
//! about this via `#[deprecated]` but the code must still compile.

#![expect(deprecated, reason = "This fixture intentionally triggers validation warnings.")]

use fp_macros::document_module;

#[document_module]
mod lint_target {
	pub struct MyType;

	impl MyType {
		pub fn apply<F: Fn(i32) -> i32>(
			f: F,
			x: i32,
		) -> i32 {
			f(x)
		}
	}
}

fn main() {
	let result = lint_target::MyType::apply(|x| x + 1, 5);
	assert_eq!(result, 6);
}
