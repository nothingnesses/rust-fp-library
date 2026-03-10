use fp_macros::{
	document_module,
	impl_kind,
	trait_kind,
};

#[document_module(no_validation)]
mod test_mod {
	use super::*;

	trait_kind!(
		type Of<T>;
	);

	#[allow(dead_code)]
	pub struct MyBrand;
	#[allow(dead_code)]
	pub struct MyType<T>(T);

	impl_kind! {
		for MyBrand {
			#[document_default]
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
		#[document_signature]
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

#[document_module(no_validation)]
mod test_collision {
	use fp_macros::{
		impl_kind,
		trait_kind,
	};
	trait_kind!(
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
	#[allow(unexpected_cfgs)]
	mod test_cfg_no_conflict {
		use fp_macros::impl_kind;
		#[allow(dead_code)]
		pub struct Brand;
		#[allow(dead_code)]
		pub struct SyncType<T>(T);
		#[allow(dead_code)]
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
		#[allow(dead_code, non_camel_case_types)]
		trait Kind_ad6c20556a82a1f0 {
			type Of<T>;
		}
	}

	#[fp_macros::document_module]
	mod test_dyn_formatting {
		#[allow(dead_code)]
		pub trait MyTrait {}

		#[allow(dead_code)]
		pub struct Brand;

		#[allow(dead_code)]
		pub trait TestTrait {
			#[document_signature]
			fn foo() -> Box<dyn MyTrait>;
		}
	}
	// We can't have two impl_kind! for the same Brand in the same module
	// if they implement the same trait. But document_module should still
	// be able to merge them if they were valid.
	// For the sake of this test, we'll use a different Brand for the second one
	// or just test that one block works.
}

#[document_module(no_validation)]
mod test_erasure {
	#[allow(dead_code)]
	pub struct Brand;
	#[allow(dead_code)]
	pub trait MyTrait {
		#[document_signature]
		#[allow(clippy::needless_lifetimes)]
		unsafe fn foo<'a, T: ?Sized>(x: &'a T) -> &'a T;
	}
}

#[document_module(no_validation)]
mod test_impl_level_document_parameters {
	use fp_macros::document_parameters;

	#[allow(dead_code)]
	pub struct MyList<T>(Vec<T>);

	/// Test impl-level document_parameters with receiver-only method
	#[document_type_parameters("The type of elements in the list")]
	#[document_parameters("The list instance")]
	impl<T> MyList<T> {
		#[allow(dead_code)]
		#[document_signature]
		#[document_parameters]
		pub fn len(&self) -> usize {
			self.0.len()
		}

		#[allow(dead_code)]
		#[document_signature]
		#[document_parameters]
		pub fn is_empty(&self) -> bool {
			self.0.is_empty()
		}

		#[allow(dead_code)]
		#[document_signature]
		#[document_parameters("The element to append")]
		pub fn push(
			&mut self,
			item: T,
		) {
			self.0.push(item)
		}

		#[allow(dead_code)]
		#[document_signature]
		#[document_parameters("The element to prepend")]
		pub fn cons(
			self,
			item: T,
		) -> Self {
			let mut new_vec = vec![item];
			new_vec.extend(self.0);
			MyList(new_vec)
		}

		// Static method (no receiver) should work without impl-level receiver doc
		#[allow(dead_code)]
		#[document_signature]
		#[document_parameters("The initial capacity")]
		pub fn with_capacity(capacity: usize) -> Self {
			MyList(Vec::with_capacity(capacity))
		}
	}

	/// Test multiple impl blocks for the same type
	#[document_parameters("The list to operate on")]
	impl<T: Clone> MyList<T> {
		#[allow(dead_code)]
		#[document_signature]
		#[document_parameters]
		pub fn clone_list(&self) -> Self {
			MyList(self.0.clone())
		}
	}
}

#[test]
fn test_impl_level_document_parameters_integration() {
	// Compile-time test to ensure impl-level document_parameters works
}

// --- Trait support tests ---

/// Trait with fully documented methods - no validation warnings expected.
#[document_module]
mod test_trait_fully_documented {
	use fp_macros::{
		document_examples,
		document_parameters,
		document_returns,
	};

	#[allow(dead_code)]
	#[fp_macros::document_type_parameters("The element type.")]
	#[document_parameters("The collection instance.")]
	pub trait MyCollection<T> {
		/// Returns the number of elements.
		#[fp_macros::document_signature]
		#[document_returns("The number of elements in the collection.")]
		#[document_examples]
		///
		/// ```
		/// // Example placeholder
		/// assert!(true);
		/// ```
		fn len(&self) -> usize;

		/// Adds an element.
		#[fp_macros::document_signature]
		#[fp_macros::document_type_parameters("The lifetime of the value.")]
		#[document_parameters("The element to add.")]
		#[document_returns("Whether the element was added.")]
		#[document_examples]
		///
		/// ```
		/// assert!(true);
		/// ```
		fn add<'a>(
			&mut self,
			item: T,
		) -> bool;
	}
}

/// Marker trait (no methods) - no validation warnings expected.
#[document_module]
mod test_marker_trait {
	#[allow(dead_code)]
	pub trait Marker {}
}

/// Module with both trait and impl - both validated/generated.
#[document_module(no_validation)]
mod test_trait_and_impl_together {
	use fp_macros::document_parameters;

	#[allow(dead_code)]
	pub trait Greet {
		#[document_signature]
		fn greet(&self) -> String;
	}

	#[allow(dead_code)]
	pub struct Greeter;

	#[document_parameters("The greeter instance.")]
	impl Greet for Greeter {
		#[document_signature]
		#[document_parameters]
		fn greet(&self) -> String {
			"hello".into()
		}
	}
}

/// Trait-level #[document_parameters] - receiver doc applied to methods.
#[document_module(no_validation)]
mod test_trait_level_document_parameters {
	use fp_macros::document_parameters;

	#[allow(dead_code)]
	#[document_parameters("The stack instance.")]
	pub trait Stack<T> {
		#[document_parameters("The element to push.")]
		fn push(
			&mut self,
			item: T,
		);

		#[document_parameters]
		fn pop(&mut self) -> Option<T>;
	}
}

#[test]
fn test_trait_support_integration() {
	// Compile-time test: trait support in document_module
}

#[test]
fn test_trait_level_document_parameters_integration() {
	// Compile-time test: trait-level #[document_parameters] with receiver doc
}
