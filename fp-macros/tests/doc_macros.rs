use fp_macros::{doc_params, doc_type_params, document_impl, hm_signature};

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

pub trait MyTrait<T> {
	fn foo(
		&self,
		x: T,
	) -> T;
}

pub struct MyType<T>(T);

#[document_impl(doc_type_params("The element type"))]
impl<T: Clone> MyTrait<T> for MyType<T> {
	#[hm_signature]
	#[doc_type_params]
	fn foo(
		&self,
		x: T,
	) -> T {
		x.clone()
	}
}

/// Integration test to verify that `document_impl` generates valid Rust code that compiles.
/// - Defines a trait and a struct.
/// - Applies `#[document_impl]` with `#[hm_signature]` and `#[doc_type_params]`.
/// - This ensures that the macro expansion produces syntactically valid code (e.g., valid paths, no duplicate lifetimes)
///   and that the compiler accepts the transformed AST.
#[test]
fn test_macro_integration() {
	// This is a compile-time test mostly, but we can check if it compiles.
}
