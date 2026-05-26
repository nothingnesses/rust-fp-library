//! Test: #[document_examples] rejects skip_call_check on non-function items.

use fp_macros::document_examples;

#[document_examples(skip_call_check, reason = "struct documentation has no direct call target")]
///
/// ```
/// let value = 3;
/// assert_eq!(value, 3);
/// ```
struct DocumentedStruct;

fn main() {}
