//! Test: #[document_module] rejects arguments.
//!
//! Module validation is always enabled, so `#[document_module]` no longer
//! accepts configuration arguments.

#[fp_macros::document_module(anything)]
mod inner {
	pub struct MyType;
}

fn main() {}
