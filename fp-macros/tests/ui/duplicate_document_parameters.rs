//! Test: Duplicate #[document_parameters] on a function

use fp_macros::document_parameters;

#[document_parameters("The value")]
#[document_parameters("The value")]
fn foo(x: i32) {}

fn main() {}
