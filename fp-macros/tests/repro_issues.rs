use fp_macros::document_module;

#[document_module(no_validation)]
mod test_context {
	#[allow(dead_code)]
	pub struct CatListBrand;
	#[allow(dead_code)]
	pub struct CatList<A>(A);

	impl<A> CatList<A> {
		#[document_signature]
		#[allow(dead_code)]
		pub fn empty() -> Self {
			todo!()
		}

		#[document_signature]
		#[allow(dead_code)]
		pub fn is_empty(&self) -> bool {
			true
		}
	}
}

#[test]
fn test_repro_issues() {
	// This is a compile-time test to see debug logs
}
