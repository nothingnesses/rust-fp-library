//! Test: Duplicate #[document_examples] on a function

use fp_macros::document_examples;

#[document_examples]
#[document_examples]
///
/// ```
/// assert!(true);
/// ```
fn foo() {}

fn main() {}
