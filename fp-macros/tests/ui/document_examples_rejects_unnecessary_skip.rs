//! Test: #[document_examples] rejects stale skip_call_check on direct examples.

use fp_macros::document_examples;

#[document_examples(
	skip_call_check,
	reason = "direct call validation should catch this stale skip"
)]
///
/// ```
/// let value = documented_function();
/// assert_eq!(value, 3);
/// ```
fn documented_function() -> i32 {
	3
}

fn main() {}
