//! Test: #[document_signature] on a trait
//!
//! This test verifies that using #[document_signature] on a trait
//! (rather than a function or method) produces a helpful error message.

use fp_macros::document_signature;

#[document_signature]
trait Functor {
	fn map();
}

fn main() {}
