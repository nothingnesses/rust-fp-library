use fp_macros::{doc_params, doc_type_params, hm_signature};

#[doc_type_params(
    "The type of the elements.",
    ("E", "The error type.")
)]
#[doc_params(
    "The input value.",
    ("y", "The second input value (curried).")
)]
#[hm_signature]
pub fn test_fn<T: Clone, ERR>(x: T) -> impl Fn(i32) -> T {
	move |_| x.clone()
}

#[test]
fn test_macro_integration() {
	// This is a compile-time test mostly, but we can check if it compiles.
}
