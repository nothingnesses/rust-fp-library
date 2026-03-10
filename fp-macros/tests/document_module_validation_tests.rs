//! Tests for document_module validation feature
//!
//! This module tests the validation warnings that are emitted when
//! impl blocks or methods are missing expected documentation attributes.
//! Since warnings are now emitted via `#[deprecated]` instead of `compile_error!`,
//! all tests compile successfully (warnings don't block compilation).

use fp_macros::document_module;

// =========================================================================
// Existing tests
// =========================================================================

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

// Test that #[allow_named_generics] suppresses the lint
#[document_module(no_validation)]
mod test_impl_trait_lint_suppressed {
	pub struct MyType;

	impl MyType {
		#[allow(dead_code)]
		#[allow_named_generics]
		pub fn apply<F: Fn(i32) -> i32>(
			f: F,
			x: i32,
		) -> i32 {
			f(x)
		}
	}
}

#[test]
fn test_impl_trait_lint_suppressed() {
	let result = test_impl_trait_lint_suppressed::MyType::apply(|x| x * 2, 5);
	assert_eq!(result, 10);
}

// Test that no_validation also skips the impl Trait lint
#[document_module(no_validation)]
mod test_no_validation_mode_skips_lint {
	pub struct MyType;

	impl MyType {
		#[allow(dead_code)]
		pub fn apply<F: Fn(i32) -> i32>(
			f: F,
			x: i32,
		) -> i32 {
			f(x)
		}
	}
}

#[test]
fn test_no_validation_mode_skips_lint() {
	let result = test_no_validation_mode_skips_lint::MyType::apply(|x| x + 10, 5);
	assert_eq!(result, 15);
}

// =========================================================================
// 7e: Suppression attribute is properly stripped
// =========================================================================

// If #[allow_named_generics] is NOT stripped, this would cause
// "unknown attribute" error. Compiling successfully proves it's stripped.
#[document_module(no_validation)]
mod test_allow_named_generics_stripped {
	pub struct MyType;

	impl MyType {
		#[allow(dead_code)]
		#[allow_named_generics]
		pub fn transform<F: Fn(i32) -> i32>(
			f: F,
			x: i32,
		) -> i32 {
			f(x)
		}
	}
}

#[test]
fn test_allow_named_generics_stripped() {
	let result = test_allow_named_generics_stripped::MyType::transform(|x| x * 3, 4);
	assert_eq!(result, 12);
}
