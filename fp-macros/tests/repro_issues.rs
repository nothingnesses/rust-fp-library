use fp_macros::document_module;

#[document_module]
mod test_context {
	#[allow(dead_code)]
	pub struct CatListBrand;
	#[allow(dead_code)]
	pub struct CatList<A>(A);

	impl<A> CatList<A> {
		#[hm_signature]
		#[allow(dead_code)]
		pub fn empty() -> Self {
			todo!()
		}

		#[hm_signature]
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
