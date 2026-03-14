//! Test: #[document_signature] on a trait inside #[document_module]
//!
//! This test verifies that using #[document_signature] on a trait
//! inside a #[document_module] produces an error, since it is only
//! valid on functions and methods.

#[fp_macros::document_module(no_validation)]
mod inner {
	#[allow(dead_code)]
	#[document_signature]
	pub trait Functor {
		fn map();
	}
}

fn main() {}
