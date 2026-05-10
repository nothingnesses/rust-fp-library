//! Test: Duplicate #[document_examples] on a function

use fp_macros::document_examples;

#[document_examples]
#[document_examples]
///
/// ```
/// let value = 2;
/// assert_eq!(value, 2);
/// ```
fn foo() {}

fn main() {}
