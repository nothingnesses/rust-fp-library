//! Validation for checking that items have appropriate documentation attributes.
//!
//! This module provides validation logic to warn when impl blocks or methods
//! are missing expected documentation attributes based on their characteristics.

use crate::{
	analysis::get_all_parameters,
	core::{
		constants::attributes::{
			DOCUMENT_PARAMETERS, DOCUMENT_SIGNATURE, DOCUMENT_TYPE_PARAMETERS,
		},
		error_handling::ErrorCollector,
	},
	support::attributes::has_attribute,
};
use syn::{FnArg, ImplItem, Item, spanned::Spanned};

/// Check if a method has a receiver parameter (self, &self, &mut self, etc.)
fn has_receiver(method: &syn::ImplItemFn) -> bool {
	method.sig.inputs.iter().any(|arg| matches!(arg, FnArg::Receiver(_)))
}

/// Check if a method has non-receiver parameters
fn has_non_receiver_parameters(method: &syn::ImplItemFn) -> bool {
	method.sig.inputs.iter().any(|arg| matches!(arg, FnArg::Typed(_)))
}

/// Validate that a method has appropriate documentation attributes.
fn validate_method_documentation(
	method: &syn::ImplItemFn,
	warnings: &mut ErrorCollector,
) {
	let method_name = &method.sig.ident;
	let method_generics = &method.sig.generics;

	// Check for document_signature
	if !has_attribute(&method.attrs, DOCUMENT_SIGNATURE) {
		let warning = syn::Error::new(
			method.span(),
			format!("Method `{method_name}` should have #[{DOCUMENT_SIGNATURE}] attribute",),
		);
		warnings.push(warning);
	}

	// Check for document_type_parameters if method has type parameters
	let has_type_params = !method_generics.params.is_empty();
	let has_doc_type_params = has_attribute(&method.attrs, DOCUMENT_TYPE_PARAMETERS);

	if has_type_params && !has_doc_type_params {
		let type_param_names: Vec<String> = get_all_parameters(method_generics);
		let warning = syn::Error::new(
			method.span(),
			format!(
				"Method `{method_name}` has type parameters <{}> but no #[{DOCUMENT_TYPE_PARAMETERS}] attribute",
				type_param_names.join(", "),
			),
		);
		warnings.push(warning);
	}

	// Check for document_parameters if method has non-receiver parameters
	if has_non_receiver_parameters(method) && !has_attribute(&method.attrs, DOCUMENT_PARAMETERS) {
		let warning = syn::Error::new(
			method.span(),
			format!(
				"Method `{method_name}` has parameters but no #[{DOCUMENT_PARAMETERS}] attribute",
			),
		);
		warnings.push(warning);
	}
}

/// Validate that an impl block has appropriate documentation attributes.
fn validate_impl_documentation(
	item_impl: &syn::ItemImpl,
	warnings: &mut ErrorCollector,
) {
	let impl_generics = &item_impl.generics;
	let has_type_params = !impl_generics.params.is_empty();
	let has_doc_type_params = has_attribute(&item_impl.attrs, DOCUMENT_TYPE_PARAMETERS);

	// Check if any methods have receivers
	let has_methods_with_receivers = item_impl
		.items
		.iter()
		.any(|item| if let ImplItem::Fn(method) = item { has_receiver(method) } else { false });

	// Warn if impl has type parameters but no document_type_parameters
	if has_type_params && !has_doc_type_params {
		let type_param_names: Vec<String> = get_all_parameters(impl_generics);
		let warning = syn::Error::new(
			item_impl.span(),
			format!(
				"Impl block has type parameters <{}> but no #[{DOCUMENT_TYPE_PARAMETERS}] attribute",
				type_param_names.join(", "),
			),
		);
		warnings.push(warning);
	}

	// Warn if impl has methods with receivers but no document_parameters at impl level
	// Note: This checks for impl-level document_parameters, which documents the receiver type
	if has_methods_with_receivers && !has_attribute(&item_impl.attrs, DOCUMENT_PARAMETERS) {
		let warning = syn::Error::new(
			item_impl.span(),
			format!(
				"Impl block contains methods with receiver parameters but no #[{DOCUMENT_PARAMETERS}] attribute",
			),
		);
		warnings.push(warning);
	}

	// Validate each method in the impl block
	for impl_item in &item_impl.items {
		if let ImplItem::Fn(method) = impl_item {
			validate_method_documentation(method, warnings);
		}
	}
}

/// Validate documentation attributes on all items.
///
/// This function checks that impl blocks and methods have appropriate
/// documentation attributes based on their characteristics (type parameters,
/// parameters, etc.).
///
/// Returns a list of warnings (as syn::Error objects) that can be emitted
/// or collected for reporting.
pub fn validate_documentation(items: &[Item]) -> Vec<syn::Error> {
	let mut warnings = ErrorCollector::new();

	for item in items {
		if let Item::Impl(item_impl) = item {
			validate_impl_documentation(item_impl, &mut warnings);
		}
	}

	warnings.into_errors()
}

#[cfg(test)]
mod tests {
	use super::*;
	use syn::parse_quote;

	#[test]
	fn test_validate_method_without_signature() {
		let method: syn::ImplItemFn = parse_quote! {
			fn foo() -> i32 { 42 }
		};

		let mut warnings = ErrorCollector::new();
		validate_method_documentation(&method, &mut warnings);

		assert!(warnings.has_errors());
		assert_eq!(warnings.len(), 1);
		let warning_msg = warnings.into_errors()[0].to_string();
		assert!(warning_msg.contains("should have #[document_signature]"));
	}

	#[test]
	fn test_validate_method_with_signature() {
		let method: syn::ImplItemFn = parse_quote! {
			#[document_signature]
			fn foo() -> i32 { 42 }
		};

		let mut warnings = ErrorCollector::new();
		validate_method_documentation(&method, &mut warnings);

		// Should not warn about missing signature
		assert!(!warnings.has_errors());
	}

	#[test]
	fn test_validate_method_with_type_params_missing_doc() {
		let method: syn::ImplItemFn = parse_quote! {
			#[document_signature]
			fn foo<T>() -> T { todo!() }
		};

		let mut warnings = ErrorCollector::new();
		validate_method_documentation(&method, &mut warnings);

		assert!(warnings.has_errors());
		let warning_msg = warnings.into_errors()[0].to_string();
		assert!(warning_msg.contains("type parameters"));
		assert!(warning_msg.contains("document_type_parameters"));
	}

	#[test]
	fn test_validate_method_with_type_params_documented() {
		let method: syn::ImplItemFn = parse_quote! {
			#[document_signature]
			#[document_type_parameters("T" = "Type parameter")]
			fn foo<T>() -> T { todo!() }
		};

		let mut warnings = ErrorCollector::new();
		validate_method_documentation(&method, &mut warnings);

		// Should only warn about missing signature, not type params
		assert!(!warnings.has_errors());
	}

	#[test]
	fn test_validate_method_with_params_missing_doc() {
		let method: syn::ImplItemFn = parse_quote! {
			#[document_signature]
			fn foo(x: i32) -> i32 { x }
		};

		let mut warnings = ErrorCollector::new();
		validate_method_documentation(&method, &mut warnings);

		assert!(warnings.has_errors());
		let warning_msg = warnings.into_errors()[0].to_string();
		assert!(warning_msg.contains("parameters"));
		assert!(warning_msg.contains("document_parameters"));
	}

	#[test]
	fn test_validate_method_with_params_documented() {
		let method: syn::ImplItemFn = parse_quote! {
			#[document_signature]
			#[document_parameters("x" = "Input value")]
			fn foo(x: i32) -> i32 { x }
		};

		let mut warnings = ErrorCollector::new();
		validate_method_documentation(&method, &mut warnings);

		assert!(!warnings.has_errors());
	}

	#[test]
	fn test_validate_impl_with_type_params_missing_doc() {
		let item_impl: syn::ItemImpl = parse_quote! {
			impl<T> MyTrait for MyType<T> {
				fn foo() {}
			}
		};

		let mut warnings = ErrorCollector::new();
		validate_impl_documentation(&item_impl, &mut warnings);

		assert!(warnings.has_errors());
		// Should have warning about impl type params and method signature
		assert!(warnings.len() >= 2);
	}

	#[test]
	fn test_validate_impl_with_receiver_missing_doc() {
		let item_impl: syn::ItemImpl = parse_quote! {
			impl MyType {
				fn foo(&self) {}
			}
		};

		let mut warnings = ErrorCollector::new();
		validate_impl_documentation(&item_impl, &mut warnings);

		assert!(warnings.has_errors());
		// Should warn about impl-level document_parameters and method signature
		let errors = warnings.into_errors();
		let has_impl_warning = errors
			.iter()
			.any(|e| e.to_string().contains("Impl block contains methods with receiver"));
		assert!(has_impl_warning);
	}
}
