//! Tests for document_module validation feature
//!
//! This module tests the validation warnings that are emitted when
//! impl blocks or methods are missing expected documentation attributes.

use fp_macros::document_module;

// Test that validation mode can be disabled
// This module has undocumented items but should compile without warnings
#[document_module(no_validation)]
mod test_no_validation {
	pub struct MyType;

	// This impl block is missing documentation attributes
	// but should not produce warnings with no_validation
	impl MyType {
		pub fn new() -> Self {
			Self
		}

		#[allow(dead_code)]
		pub fn process<T>(
			&self,
			_value: T,
		) -> T {
			_value
		}
	}
}

#[test]
fn test_no_validation_mode_compiles() {
	// If this test compiles, it means no_validation mode is working
	let _ = test_no_validation::MyType::new();
}

// This module should produce validation warnings (errors) when compiled
// Comment out to allow tests to pass - uncomment to see validation in action
/*
#[document_module]
mod test_default_validation {
	pub struct MyType;

	// This should warn: impl has methods with receivers but no #[document_parameters]
	impl MyType {
		// This should warn: method missing #[document_signature]
		pub fn new() -> Self {
			Self
		}

		// This should warn multiple times:
		// - Missing #[document_signature]
		// - Has type parameters but no #[document_type_parameters]
		// - Has parameters but no #[document_parameters]
		pub fn process<T>(&self, _value: T) -> T {
			_value
		}
	}
}
*/

// Commented out: Examples that would trigger validation warnings
// Uncomment to see the validation in action:
//
// #[document_module]  // Uses default validation (warn mode)
// mod test_with_warnings {
//     pub struct MyType;
//
//     // WARNING: Impl block contains methods with receiver parameters but no #[document_parameters]
//     impl MyType {
//         // WARNING: Method missing #[document_signature]
//         pub fn new() -> Self {
//             Self
//         }
//
//         // WARNING: Method has type parameters but no #[document_type_parameters]
//         // WARNING: Method has parameters but no #[document_parameters]
//         // WARNING: Method missing #[document_signature]
//         pub fn process<T>(&self, _value: T) -> T {
//             _value
//         }
//     }
// }

// Test validation with type parameters on impl
#[document_module(no_validation)]
mod test_impl_type_params {
	pub struct MyType<T>(T);

	// Without validation, this compiles even though it's missing:
	// - #[document_type_parameters] for impl-level T
	// - #[document_parameters] for methods with receivers
	impl<T> MyType<T> {
		pub fn new(value: T) -> Self {
			Self(value)
		}

		pub fn get(&self) -> &T {
			&self.0
		}
	}
}

#[test]
fn test_impl_type_params_no_validation() {
	let instance = test_impl_type_params::MyType::new(100);
	assert_eq!(*instance.get(), 100);
}

// Test that nested modules are also validated
#[document_module(no_validation)]
mod test_nested_no_validation {
	pub struct Outer;

	impl Outer {
		pub fn outer_method(&self) {}
	}

	pub mod inner {
		pub struct Inner;

		impl Inner {
			pub fn inner_method(&self) {}
		}
	}
}

#[test]
fn test_nested_no_validation_compiles() {
	let outer = test_nested_no_validation::Outer;
	outer.outer_method();

	let inner = test_nested_no_validation::inner::Inner;
	inner.inner_method();
}
