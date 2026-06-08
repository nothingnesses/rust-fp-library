//! Test: #[document_examples] rejects literal-only assertions

use fp_macros::document_examples;

#[document_examples]
///
/// ```
/// assert_eq!(0 + 1, 1);
/// ```
fn weak_example() {}

fn main() {}
