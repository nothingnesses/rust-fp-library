//! Test: Duplicate #[document_examples] on a function

use fp_macros::document_examples;

#[document_examples]
#[document_examples]
///
/// ```
/// assert_eq!(1 + 1, 2);
/// ```
fn foo() {}

fn main() {}
