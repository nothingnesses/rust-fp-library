//! Attribute parsing and filtering utilities.
//!
//! This module provides utilities for parsing, filtering, and working with attributes,
//! including documentation-specific attributes like `document_default` and `document_use`.

use crate::core::{Result, constants::attributes};
use proc_macro2::TokenStream;
use syn::{Attribute, Expr, ExprLit, Lit, Meta, parse::Parse, spanned::Spanned};

/// Finds the index of the first attribute with the given name.
pub fn find_attribute(
	attrs: &[Attribute],
	name: &str,
) -> Option<usize> {
	attrs.iter().position(|attr| attr.path().is_ident(name))
}

/// Checks if an attribute with the given name exists.
pub fn has_attribute(
	attrs: &[Attribute],
	name: &str,
) -> bool {
	attrs.iter().any(|attr| attr.path().is_ident(name))
}

/// Returns true if the attribute should be kept in generated code.
///
/// This filters out documentation-specific attributes like `document_default`
/// and `document_use` which are processed by the macro system but should not
/// appear in the final generated code.
///
/// # Examples
///
/// ```ignore
/// use syn::parse_quote;
///
/// let attr: Attribute = parse_quote!(#[document_default]);
/// assert!(!should_keep_attribute(&attr));
///
/// let attr: Attribute = parse_quote!(#[derive(Debug)]);
/// assert!(should_keep_attribute(&attr));
/// ```
pub fn should_keep_attribute(attr: &Attribute) -> bool {
	!is_doc_attribute(attr)
}

/// Returns true if the attribute is documentation-specific.
///
/// Documentation-specific attributes include:
/// - `document_default`: Marks an associated type as the default for resolution
/// - `document_use`: Specifies which associated type to use for documentation
///
/// # Examples
///
/// ```ignore
/// use syn::parse_quote;
///
/// let attr: Attribute = parse_quote!(#[document_default]);
/// assert!(is_doc_attribute(&attr));
///
/// let attr: Attribute = parse_quote!(#[document_use = "Of"]);
/// assert!(is_doc_attribute(&attr));
/// ```
pub fn is_doc_attribute(attr: &Attribute) -> bool {
	attributes::DOCUMENT_SPECIFIC_ATTRS.iter().any(|name| attr.path().is_ident(name))
}

/// Filters out documentation-specific attributes from a slice.
///
/// This returns an iterator over attributes that should be kept
/// (i.e., are not documentation-specific).
///
/// # Examples
///
/// ```ignore
/// use syn::{Attribute, parse_quote};
///
/// let attrs: Vec<Attribute> = vec![
///     parse_quote!(#[document_default]),
///     parse_quote!(#[derive(Debug)]),
///     parse_quote!(#[document_use = "Of"]),
/// ];
///
/// let filtered: Vec<_> = filter_doc_attributes(&attrs).collect();
/// assert_eq!(filtered.len(), 1); // Only #[derive(Debug)] remains
/// ```
pub fn filter_doc_attributes(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
	attrs.iter().filter(|attr| should_keep_attribute(attr))
}

/// Remove an attribute at the given index and extract its tokens.
///
/// This function removes the attribute and returns its tokens for parsing.
/// If the attribute is not in list form (i.e., has no arguments), returns empty tokens.
///
/// # Returns
///
/// The tokens from the attribute's meta list, or empty tokens if not a list.
///
/// # Examples
///
/// ```ignore
/// let idx = find_attribute(&item.attrs, "document_fields").unwrap();
/// let tokens = remove_attribute_tokens(&mut item.attrs, idx)?;
/// let args: DocumentFieldParameters = syn::parse2(tokens)?;
/// ```
pub fn remove_attribute_tokens(
	attrs: &mut Vec<Attribute>,
	index: usize,
) -> syn::Result<TokenStream> {
	let attr = attrs.remove(index);

	// Try to get tokens from list form: #[attr(...)]
	if let Ok(meta_list) = attr.meta.require_list() {
		Ok(meta_list.tokens.clone())
	} else {
		// Attribute has no arguments: #[attr]
		Ok(TokenStream::new())
	}
}

/// Extracts a single string value from a name-value attribute, with duplicate checking.
///
/// For example, this extracts `"SomeType"` from `#[document_use = "SomeType"]`.
///
/// Returns `Ok(Some(value))` if a unique name-value attribute is found,
/// `Ok(None)` if the attribute doesn't exist or isn't in name-value form,
/// or `Err(...)` if duplicate attributes are found.
pub fn parse_unique_attribute_value(
	attrs: &[syn::Attribute],
	name: &str,
) -> Result<Option<String>> {
	let mut found = None;
	for attr in attrs {
		if attr.path().is_ident(name) {
			if found.is_some() {
				return Err(crate::core::Error::validation(
					attr.span(),
					format!("Multiple `#[{name}]` attributes found on same item"),
				));
			}
			if let syn::Meta::NameValue(nv) = &attr.meta
				&& let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value
			{
				found = Some(s.value());
			}
		}
	}
	Ok(found)
}

/// Extension trait for ergonomic attribute manipulation on `Vec<Attribute>`.
///
/// This trait provides convenient methods for common attribute operations,
/// combining finding, removing, and parsing into single operations.
///
/// # Examples
///
/// ```ignore
/// use crate::support::attributes::AttributeExt;
///
/// // Find and remove a parsed attribute
/// if let Some(args) = item.attrs.find_and_remove::<DocumentFieldParameters>("document_fields")? {
///     // Process args
/// }
///
/// // Find and extract a name-value attribute's string value
/// let document_use = method.attrs.find_value("document_use")?;
///
/// // Check for attribute existence
/// if item.attrs.has_attribute("inline") {
///     // Handle inline attribute
/// }
/// ```
#[allow(dead_code)]
pub trait AttributeExt {
	/// Find, remove, and parse an attribute in one operation.
	///
	/// Returns `Ok(Some(parsed))` if the attribute was found and parsed successfully,
	/// `Ok(None)` if the attribute was not found, or `Err(...)` if parsing failed.
	///
	/// # Examples
	///
	/// ```ignore
	/// use crate::support::attributes::AttributeExt;
	///
	/// if let Some(args) = item.attrs.find_and_remove::<DocumentFieldParameters>("document_fields")? {
	///     // The attribute has been removed and args are parsed
	/// }
	/// ```
	fn find_and_remove<T: Parse>(
		&mut self,
		name: &str,
	) -> Result<Option<T>>;

	/// Find and extract a name-value attribute's string value without removing it.
	///
	/// For example, extracts `"SomeType"` from `#[document_use = "SomeType"]`.
	///
	/// Returns `Ok(Some(value))` if the attribute was found with a string value,
	/// `Ok(None)` if the attribute was not found or not in name-value form,
	/// or `Err(...)` if duplicate attributes exist.
	///
	/// # Examples
	///
	/// ```ignore
	/// use crate::support::attributes::AttributeExt;
	///
	/// let document_use = method.attrs.find_value("document_use")?;
	/// ```
	fn find_value(
		&self,
		name: &str,
	) -> Result<Option<String>>;

	/// Find, remove, and extract a name-value attribute's string value.
	///
	/// This is like `find_value` but also removes the attribute from the list.
	///
	/// Returns `Ok(Some(value))` if the attribute was found and removed,
	/// or `Ok(None)` if the attribute was not found or not in name-value form.
	///
	/// # Examples
	///
	/// ```ignore
	/// use crate::support::attributes::AttributeExt;
	///
	/// if let Some(value) = item.attrs.find_and_remove_value("document_use")? {
	///     // Use value, attribute has been removed
	/// }
	/// ```
	fn find_and_remove_value(
		&mut self,
		name: &str,
	) -> Result<Option<String>>;

	/// Check if an attribute with the given name exists.
	///
	/// # Examples
	///
	/// ```ignore
	/// use crate::support::attributes::AttributeExt;
	///
	/// if item.attrs.has_attribute("inline") {
	///     // Handle inline attribute
	/// }
	/// ```
	fn has_attribute(
		&self,
		name: &str,
	) -> bool;

	/// Find and extract a name-value attribute's string value, collecting errors instead of returning them.
	///
	/// This is a convenience method that combines `find_value` with error collection,
	/// useful when processing multiple attributes where you want to collect all errors
	/// rather than short-circuit on the first failure.
	///
	/// For example, extracts `"SomeType"` from `#[document_use = "SomeType"]`.
	///
	/// Returns `Some(value)` if the attribute was found with a string value,
	/// or `None` if the attribute was not found or not in name-value form.
	/// Errors (such as duplicate attributes) are pushed to the error collector.
	///
	/// # Examples
	///
	/// ```ignore
	/// use crate::support::attributes::AttributeExt;
	/// use crate::core::error_handling::ErrorCollector;
	///
	/// let mut errors = ErrorCollector::new();
	/// let document_use = method.attrs.find_value_or_collect("document_use", &mut errors);
	/// ```
	fn find_value_or_collect(
		&self,
		name: &str,
		errors: &mut crate::core::error_handling::ErrorCollector,
	) -> Option<String>;
}

impl AttributeExt for Vec<Attribute> {
	fn find_and_remove<T: Parse>(
		&mut self,
		name: &str,
	) -> Result<Option<T>> {
		let Some(index) = find_attribute(self, name) else {
			return Ok(None);
		};

		let tokens = remove_attribute_tokens(self, index).map_err(crate::core::Error::Parse)?;

		if tokens.is_empty() {
			return Ok(None);
		}

		let parsed = syn::parse2::<T>(tokens).map_err(crate::core::Error::Parse)?;
		Ok(Some(parsed))
	}

	fn find_value(
		&self,
		name: &str,
	) -> Result<Option<String>> {
		parse_unique_attribute_value(self, name)
	}

	fn find_and_remove_value(
		&mut self,
		name: &str,
	) -> Result<Option<String>> {
		let Some(index) = find_attribute(self, name) else {
			return Ok(None);
		};

		let attr = self.remove(index);
		if let Meta::NameValue(nv) = &attr.meta
			&& let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = &nv.value
		{
			return Ok(Some(s.value()));
		}
		Ok(None)
	}

	fn has_attribute(
		&self,
		name: &str,
	) -> bool {
		has_attribute(self, name)
	}

	fn find_value_or_collect(
		&self,
		name: &str,
		errors: &mut crate::core::error_handling::ErrorCollector,
	) -> Option<String> {
		self.find_value(name).unwrap_or_else(|e| {
			errors.push(e.into());
			None
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_has_attr() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> = vec![parse_quote!(#[doc = "test"])];
		assert!(has_attribute(&attrs, "doc"));
		assert!(!has_attribute(&attrs, "test"));
	}

	#[test]
	fn test_filter_document_default() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> =
			vec![parse_quote!(#[document_default]), parse_quote!(#[derive(Debug)])];

		let filtered: Vec<_> = filter_doc_attributes(&attrs).collect();

		assert_eq!(filtered.len(), 1);
		assert!(filtered[0].path().is_ident("derive"));
	}

	#[test]
	fn test_filter_document_use() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> =
			vec![parse_quote!(#[document_use = "Of"]), parse_quote!(#[inline])];

		let filtered: Vec<_> = filter_doc_attributes(&attrs).collect();

		assert_eq!(filtered.len(), 1);
		assert!(filtered[0].path().is_ident("inline"));
	}

	#[test]
	fn test_is_doc_attribute() {
		use syn::parse_quote;
		let document_default: Attribute = parse_quote!(#[document_default]);
		let document_use: Attribute = parse_quote!(#[document_use = "Of"]);
		let derive: Attribute = parse_quote!(#[derive(Debug)]);

		assert!(is_doc_attribute(&document_default));
		assert!(is_doc_attribute(&document_use));
		assert!(!is_doc_attribute(&derive));
	}

	#[test]
	fn test_should_keep_attribute() {
		use syn::parse_quote;
		let document_default: Attribute = parse_quote!(#[document_default]);
		let derive: Attribute = parse_quote!(#[derive(Debug)]);

		assert!(!should_keep_attribute(&document_default));
		assert!(should_keep_attribute(&derive));
	}

	#[test]
	fn test_filter_empty() {
		let attrs: Vec<Attribute> = vec![];
		let filtered: Vec<_> = filter_doc_attributes(&attrs).collect();
		assert_eq!(filtered.len(), 0);
	}

	#[test]
	fn test_filter_all_doc_attrs() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> =
			vec![parse_quote!(#[document_default]), parse_quote!(#[document_use = "Of"])];

		let filtered: Vec<_> = filter_doc_attributes(&attrs).collect();
		assert_eq!(filtered.len(), 0);
	}

	#[test]
	fn test_filter_no_doc_attrs() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> = vec![parse_quote!(#[derive(Debug)]), parse_quote!(#[inline])];

		let filtered: Vec<_> = filter_doc_attributes(&attrs).collect();
		assert_eq!(filtered.len(), 2);
	}

	#[test]
	fn test_remove_attribute_tokens() {
		use syn::{ItemStruct, LitStr, parse_quote};
		let mut item: ItemStruct = parse_quote! {
			#[test_attr("hello")]
			#[derive(Debug)]
			struct Foo;
		};

		let idx = find_attribute(&item.attrs, "test_attr").unwrap();
		let tokens = remove_attribute_tokens(&mut item.attrs, idx).unwrap();

		// Should have removed the attribute
		assert_eq!(item.attrs.len(), 1);
		assert!(has_attribute(&item.attrs, "derive"));
		assert!(!has_attribute(&item.attrs, "test_attr"));

		// Should have extracted the tokens
		let lit: LitStr = syn::parse2(tokens).unwrap();
		assert_eq!(lit.value(), "hello");
	}

	#[test]
	fn test_remove_attribute_tokens_empty() {
		use syn::{ItemStruct, parse_quote};
		let mut item: ItemStruct = parse_quote! {
			#[test_attr]
			struct Foo;
		};

		let idx = find_attribute(&item.attrs, "test_attr").unwrap();
		let tokens = remove_attribute_tokens(&mut item.attrs, idx).unwrap();

		// Should have removed the attribute
		assert_eq!(item.attrs.len(), 0);

		// Should have empty tokens
		assert!(tokens.is_empty());
	}

	#[test]
	fn test_parse_unique_attribute_value() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> = vec![parse_quote!(#[test = "value"])];

		let result = parse_unique_attribute_value(&attrs, "test");
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), Some("value".to_string()));

		let multi_attrs: Vec<Attribute> =
			vec![parse_quote!(#[test = "v1"]), parse_quote!(#[test = "v2"])];
		let result = parse_unique_attribute_value(&multi_attrs, "test");
		assert!(result.is_err());

		// Test invalid format (not name-value)
		let invalid_attrs: Vec<Attribute> = vec![parse_quote!(#[test])];
		let result = parse_unique_attribute_value(&invalid_attrs, "test");
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), None);
	}

	// Tests for AttributeExt trait
	#[test]
	fn test_attribute_ext_find_and_remove() {
		use syn::{ItemStruct, LitStr, parse::Parse, parse_quote};

		struct TestArgs {
			value: LitStr,
		}

		impl Parse for TestArgs {
			fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
				Ok(TestArgs { value: input.parse()? })
			}
		}

		let mut item: ItemStruct = parse_quote! {
			#[test_attr("hello")]
			#[derive(Debug)]
			struct Foo;
		};

		let result = item.attrs.find_and_remove::<TestArgs>("test_attr").unwrap();

		assert!(result.is_some());
		let args = result.unwrap();
		assert_eq!(args.value.value(), "hello");
		assert_eq!(item.attrs.len(), 1);
		assert!(!item.attrs.has_attribute("test_attr"));
		assert!(item.attrs.has_attribute("derive"));
	}

	#[test]
	fn test_attribute_ext_find_and_remove_not_found() {
		use syn::{ItemStruct, LitStr, parse::Parse, parse_quote};

		struct TestArgs {
			_value: LitStr,
		}

		impl Parse for TestArgs {
			fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
				Ok(TestArgs { _value: input.parse()? })
			}
		}

		let mut item: ItemStruct = parse_quote! {
			#[derive(Debug)]
			struct Foo;
		};

		let original_len = item.attrs.len();
		let result = item.attrs.find_and_remove::<TestArgs>("nonexistent").unwrap();

		assert!(result.is_none());
		assert_eq!(item.attrs.len(), original_len);
	}

	#[test]
	fn test_attribute_ext_find_and_remove_empty_attr() {
		use syn::{ItemStruct, LitStr, parse::Parse, parse_quote};

		struct TestArgs {
			_value: LitStr,
		}

		impl Parse for TestArgs {
			fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
				Ok(TestArgs { _value: input.parse()? })
			}
		}

		let mut item: ItemStruct = parse_quote! {
			#[test_attr]
			struct Foo;
		};

		let result = item.attrs.find_and_remove::<TestArgs>("test_attr").unwrap();

		// Empty attributes should return None after removing
		assert!(result.is_none());
		// But the attribute should still be removed
		assert_eq!(item.attrs.len(), 0);
	}

	#[test]
	fn test_attribute_ext_find_value() {
		use syn::{ItemStruct, parse_quote};

		let item: ItemStruct = parse_quote! {
			#[document_use = "SomeType"]
			#[derive(Debug)]
			struct Foo;
		};

		let result = item.attrs.find_value("document_use").unwrap();
		assert_eq!(result, Some("SomeType".to_string()));

		// Attribute should not be removed
		assert_eq!(item.attrs.len(), 2);
	}

	#[test]
	fn test_attribute_ext_find_value_not_found() {
		use syn::{ItemStruct, parse_quote};

		let item: ItemStruct = parse_quote! {
			#[derive(Debug)]
			struct Foo;
		};

		let result = item.attrs.find_value("document_use").unwrap();
		assert_eq!(result, None);
	}

	#[test]
	fn test_attribute_ext_find_value_duplicate_error() {
		use syn::{ItemStruct, parse_quote};

		let item: ItemStruct = parse_quote! {
			#[test = "v1"]
			#[test = "v2"]
			struct Foo;
		};

		let result = item.attrs.find_value("test");
		assert!(result.is_err());
	}

	#[test]
	fn test_attribute_ext_find_and_remove_value() {
		use syn::{ItemStruct, parse_quote};

		let mut item: ItemStruct = parse_quote! {
			#[document_use = "SomeType"]
			#[derive(Debug)]
			struct Foo;
		};

		let result = item.attrs.find_and_remove_value("document_use").unwrap();
		assert_eq!(result, Some("SomeType".to_string()));

		// Attribute should be removed
		assert_eq!(item.attrs.len(), 1);
		assert!(!item.attrs.has_attribute("document_use"));
		assert!(item.attrs.has_attribute("derive"));
	}

	#[test]
	fn test_attribute_ext_find_and_remove_value_not_found() {
		use syn::{ItemStruct, parse_quote};

		let mut item: ItemStruct = parse_quote! {
			#[derive(Debug)]
			struct Foo;
		};

		let original_len = item.attrs.len();
		let result = item.attrs.find_and_remove_value("document_use").unwrap();
		assert_eq!(result, None);
		assert_eq!(item.attrs.len(), original_len);
	}

	#[test]
	fn test_attribute_ext_find_and_remove_value_not_name_value() {
		use syn::{ItemStruct, parse_quote};

		let mut item: ItemStruct = parse_quote! {
			#[test_attr]
			struct Foo;
		};

		let result = item.attrs.find_and_remove_value("test_attr").unwrap();
		assert_eq!(result, None);
		// Attribute should still be removed even if not name-value form
		assert_eq!(item.attrs.len(), 0);
	}

	#[test]
	fn test_attribute_ext_has_attribute() {
		use syn::{ItemStruct, parse_quote};

		let item: ItemStruct = parse_quote! {
			#[derive(Debug)]
			#[inline]
			struct Foo;
		};

		assert!(item.attrs.has_attribute("derive"));
		assert!(item.attrs.has_attribute("inline"));
		assert!(!item.attrs.has_attribute("test"));
	}

	#[test]
	fn test_attribute_ext_has_attribute_empty() {
		use syn::{ItemStruct, parse_quote};

		let item: ItemStruct = parse_quote! {
			struct Foo;
		};

		assert!(!item.attrs.has_attribute("derive"));
	}
}
