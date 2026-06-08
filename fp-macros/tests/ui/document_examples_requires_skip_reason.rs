//! Test: #[document_examples(skip_call_check)] requires a concrete reason.

use fp_macros::document_examples;

#[document_examples(skip_call_check)]
///
/// ```
/// let value = helper();
/// assert_eq!(value, 3);
/// ```
fn documented_function() -> i32 {
	3
}

fn helper() -> i32 {
	3
}

fn main() {}
