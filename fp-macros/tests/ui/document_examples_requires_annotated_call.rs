//! Test: #[document_examples] requires function examples to call the documented item.

use fp_macros::document_examples;

#[document_examples]
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
