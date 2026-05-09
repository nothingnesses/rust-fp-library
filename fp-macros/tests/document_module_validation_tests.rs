//! Tests for document_module validation feature
//!
//! This module tests the validation warnings that are emitted when
//! impl blocks or methods are missing expected documentation attributes.
//! Since warnings are now emitted via `#[deprecated]` instead of `compile_error!`,
//! all tests compile successfully (warnings don't block compilation).

#![expect(deprecated, reason = "These fixtures intentionally trigger validation warnings.")]

use fp_macros::document_module;

// =========================================================================
// Existing tests
// =========================================================================

// Test that validation warnings do not block compilation.
// This module has undocumented items and expects the warning diagnostics.
#[document_module]
mod test_validation_warnings_compile {
	pub struct MyType;

	// This impl block is missing documentation attributes.
	impl MyType {
		pub fn new() -> Self {
			Self
		}

		#[expect(dead_code, reason = "Test fixture for document_module macro")]
		pub fn process<T>(
			&self,
			_value: T,
		) -> T {
			_value
		}
	}
}

#[test]
fn test_validation_warnings_do_not_block_compilation() {
	let _ = test_validation_warnings_compile::MyType::new();
}

// Test validation with type parameters on impl
#[document_module]
mod test_impl_type_params {
	pub struct MyType<T>(T);

	// Validation emits warnings for:
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
fn test_impl_type_params_validation_warnings_compile() {
	let instance = test_impl_type_params::MyType::new(100);
	assert_eq!(*instance.get(), 100);
}

// Test that nested modules are also validated
#[document_module]
mod test_nested_validation_warnings {
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
fn test_nested_validation_warnings_compile() {
	let outer = test_nested_validation_warnings::Outer;
	outer.outer_method();

	let inner = test_nested_validation_warnings::inner::Inner;
	inner.inner_method();
}

// Test that #[allow_named_generics] suppresses the lint
#[document_module]
mod test_impl_trait_lint_suppressed {
	pub struct MyType;

	impl MyType {
		#[allow(dead_code, reason = "Test fixture exists to exercise document_module macro")]
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

// Test that the impl Trait lint emits warnings without blocking compilation.
#[document_module]
mod test_impl_trait_lint_warnings_compile {
	pub struct MyType;

	impl MyType {
		#[allow(dead_code, reason = "Test fixture exists to exercise document_module macro")]
		pub fn apply<F: Fn(i32) -> i32>(
			f: F,
			x: i32,
		) -> i32 {
			f(x)
		}
	}
}

#[test]
fn test_impl_trait_lint_warnings_compile() {
	let result = test_impl_trait_lint_warnings_compile::MyType::apply(|x| x + 10, 5);
	assert_eq!(result, 15);
}

// =========================================================================
// 7e: Suppression attribute is properly stripped
// =========================================================================

// If #[allow_named_generics] is NOT stripped, this would cause
// "unknown attribute" error. Compiling successfully proves it's stripped.
#[document_module]
mod test_allow_named_generics_stripped {
	pub struct MyType;

	impl MyType {
		#[allow(dead_code, reason = "Test fixture exists to exercise document_module macro")]
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
