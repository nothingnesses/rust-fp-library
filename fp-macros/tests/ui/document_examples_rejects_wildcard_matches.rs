//! Test: #[document_examples] rejects wildcard-only variant assertions

use fp_macros::document_examples;

#[document_examples]
///
/// ```
/// enum Example {
/// 	Variant { value: i32 },
/// }
///
/// let value = Example::Variant {
/// 	value: 42,
/// };
/// assert!(matches!(value, Example::Variant { .. }));
/// ```
fn weak_example() {}

fn main() {}
