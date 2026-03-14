//! Test: Duplicate #[document_signature] on a function
//!
//! This test verifies that using #[document_signature] more than once
//! on the same function produces a helpful error message.

use fp_macros::document_signature;

#[document_signature]
#[document_signature]
fn foo(x: i32) -> i32 {
	x + 1
}

fn main() {}
