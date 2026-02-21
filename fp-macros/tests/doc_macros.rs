use fp_macros::{
	document_module,
	document_parameters,
	document_signature,
	document_type_parameters,
};

#[document_type_parameters(
    "The type of the elements.",
    ("E", "The error type.")
)]
#[document_parameters(
    "The input value.",
    ("y", "The second input value (curried).")
)]
#[document_signature]
pub fn test_fn<T: Clone, ERR>(x: T) -> impl Fn(i32) -> T {
	move |_| x.clone()
}

#[document_module(no_validation)]
mod test_mod {
	#[allow(dead_code)]
	pub trait MyTrait<T> {
		fn foo(
			&self,
			x: T,
		) -> T;
	}

	#[allow(dead_code)]
	pub struct MyType<T>(T);

	impl<T: Clone> MyTrait<T> for MyType<T> {
		#[document_signature]
		fn foo(
			&self,
			x: T,
		) -> T {
			x.clone()
		}
	}
}

/// Integration test to verify that `document_impl` generates valid Rust code that compiles.
/// - Defines a trait and a struct.
/// - Applies `#[document_impl]` with `#[document_signature]` and `#[document_type_parameters]`.
/// - This ensures that the macro expansion produces syntactically valid code (e.g., valid paths, no duplicate lifetimes)
///   and that the compiler accepts the transformed AST.
#[test]
fn test_macro_integration() {
	// This is a compile-time test mostly, but we can check if it compiles.
}
