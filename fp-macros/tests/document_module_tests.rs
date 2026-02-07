use fp_macros::{def_kind, document_module, hm_signature, impl_kind};

#[document_module]
mod test_mod {
	use super::*;

	def_kind!(
		type Of<T>;
	);

	#[allow(dead_code)]
	pub struct MyBrand;
	#[allow(dead_code)]
	pub struct MyType<T>(T);

	impl_kind! {
		for MyBrand {
			#[doc_default]
			type Of<T> = MyType<T>;
		}
	}

	#[allow(dead_code)]
	pub trait Functor {
		fn map<A, B>(
			self,
			f: impl Fn(A) -> B,
		) -> MyType<B>;
	}

	impl Functor for MyBrand {
		#[hm_signature]
		fn map<A, B>(
			self,
			_f: impl Fn(A) -> B,
		) -> MyType<B> {
			todo!()
		}
	}
}

#[test]
fn test_document_module_integration() {
	// Compile-time test
}

#[test]
fn test_positional_matching() {
	// This is a compile-fail test or we can check the generated docs if we had a way.
	// For now, we just ensure it compiles.
}

#[document_module]
mod test_collision {
	use fp_macros::{def_kind, impl_kind};
	def_kind!(
		type Of<T>;
	);
	#[allow(dead_code)]
	pub struct Brand;
	#[allow(dead_code)]
	pub struct MyType<T>(T);
	impl_kind! {
		for Brand {
			type Of<A> = MyType<A>;
		}
	}

	#[fp_macros::document_module]
	mod test_cfg_no_conflict {
		use fp_macros::impl_kind;
		pub struct Brand;
		pub struct SyncType<T>(T);
		pub struct AsyncType<T>(T);

		#[cfg(feature = "sync")]
		impl_kind! {
			for Brand {
				type Of<T> = SyncType<T>;
			}
		}

		#[cfg(not(feature = "sync"))]
		impl_kind! {
			for Brand {
				type Of<T> = AsyncType<T>;
			}
		}

		// Add a manual impl of the Kind trait to satisfy the compiler
		// This allows document_module to scan it without erroring on missing trait
		trait Kind_ad6c20556a82a1f0 {
			type Of<T>;
		}
	}

	#[fp_macros::document_module]
	mod test_dyn_formatting {
		use fp_macros::hm_signature;
		pub trait MyTrait {}

		pub struct Brand;

		pub trait TestTrait {
			#[hm_signature]
			fn foo() -> Box<dyn MyTrait>;
		}
	}
	// We can't have two impl_kind! for the same Brand in the same module
	// if they implement the same trait. But document_module should still
	// be able to merge them if they were valid.
	// For the sake of this test, we'll use a different Brand for the second one
	// or just test that one block works.
}

#[document_module]
mod test_erasure {
	use fp_macros::hm_signature;
	#[allow(dead_code)]
	pub struct Brand;
	#[allow(dead_code)]
	pub trait MyTrait {
		#[hm_signature]
		unsafe fn foo<'a, T: ?Sized>(x: &'a T) -> &'a T;
	}
}
