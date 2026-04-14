#![expect(clippy::todo, reason = "Tests use panicking operations for brevity and clarity")]

use fp_macros::document_module;

#[document_module(no_validation)]
mod test_context {
	#[expect(dead_code, reason = "Test fixture for document_module macro")]
	pub struct CatListBrand;
	#[allow(dead_code, reason = "Test fixture exists to exercise document_module macro")]
	pub struct CatList<A>(A);

	impl<A> CatList<A> {
		#[document_signature]
		#[expect(dead_code, reason = "Test fixture for document_module macro")]
		pub fn empty() -> Self {
			todo!()
		}

		#[document_signature]
		#[expect(dead_code, reason = "Test fixture for document_module macro")]
		pub fn is_empty(&self) -> bool {
			true
		}
	}
}

#[test]
fn test_repro_issues() {
	// This is a compile-time test to see debug logs
}
