use fp_macros::{document_module, hm_signature};

#[document_module]
mod test_context {
	use fp_macros::hm_signature;

	pub struct CatListBrand;
	pub struct CatList<A>(A);

	impl<A> CatList<A> {
		#[hm_signature]
		pub fn empty() -> Self {
			todo!()
		}

		#[hm_signature]
		pub fn is_empty(&self) -> bool {
			true
		}
	}
}

#[test]
fn test_repro_issues() {
	// This is a compile-time test to see debug logs
}
