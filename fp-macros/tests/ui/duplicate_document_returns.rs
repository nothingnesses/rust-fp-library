//! Test: Duplicate #[document_returns] on a function

use fp_macros::document_returns;

#[document_returns("The result")]
#[document_returns("The result")]
fn foo() -> i32 {
	42
}

fn main() {}
