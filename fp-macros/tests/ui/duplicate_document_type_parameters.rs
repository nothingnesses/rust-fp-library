//! Test: Duplicate #[document_type_parameters] on a function

use fp_macros::document_type_parameters;

#[document_type_parameters("Type A")]
#[document_type_parameters("Type A")]
fn foo<A>() {}

fn main() {}
