//! Attribute parsing and filtering utilities.
//!
//! This module provides utilities for parsing, filtering, and working with attributes,
//! including documentation-specific attributes like `document_default` and `document_use`.

use crate::core::constants::attributes;
use proc_macro2::TokenStream;
use syn::{Attribute, parse::Parse};

/// Finds the index of the first attribute with the given name.
pub fn find_attribute(
	attrs: &[Attribute],
	name: &str,
) -> Option<usize> {
	attrs.iter().position(|attr| attr.path().is_ident(name))
}

/// Checks if an attribute with the given name exists.
pub fn has_attr(
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
/// assert!(!should_keep_attr(&attr));
///
/// let attr: Attribute = parse_quote!(#[derive(Debug)]);
/// assert!(should_keep_attr(&attr));
/// ```
pub fn should_keep_attr(attr: &Attribute) -> bool {
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
/// let filtered: Vec<_> = filter_doc_attrs(&attrs).collect();
/// assert_eq!(filtered.len(), 1); // Only #[derive(Debug)] remains
/// ```
pub fn filter_doc_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
	attrs.iter().filter(|attr| should_keep_attr(attr))
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
/// let args: FieldDocArgs = syn::parse2(tokens)?;
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

/// Find, remove, and parse an attribute in one operation.
///
/// This is a convenience function that combines finding an attribute,
/// removing it, and parsing its arguments into the desired type.
///
/// # Returns
///
/// - `Ok(Some((index, parsed)))` if the attribute was found and parsed successfully
/// - `Ok(None)` if the attribute was not found
/// - `Err(...)` if parsing failed
///
/// # Examples
///
/// ```ignore
/// let result = remove_and_parse_attribute::<FieldDocArgs>(
///     &mut variant.attrs,
///     "document_fields"
/// )?;
///
/// if let Some((attr_idx, args)) = result {
///     // Process the parsed arguments
///     // The attribute has already been removed from the attrs vector
/// }
/// ```
pub fn remove_and_parse_attribute<T: Parse>(
	attrs: &mut Vec<Attribute>,
	name: &str,
) -> syn::Result<Option<(usize, T)>> {
	let Some(index) = find_attribute(attrs, name) else {
		return Ok(None);
	};

	let tokens = remove_attribute_tokens(attrs, index)?;
	let parsed = syn::parse2::<T>(tokens)?;

	Ok(Some((index, parsed)))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_has_attr() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> = vec![parse_quote!(#[doc = "test"])];
		assert!(has_attr(&attrs, "doc"));
		assert!(!has_attr(&attrs, "test"));
	}

	#[test]
	fn test_filter_document_default() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> =
			vec![parse_quote!(#[document_default]), parse_quote!(#[derive(Debug)])];

		let filtered: Vec<_> = filter_doc_attrs(&attrs).collect();

		assert_eq!(filtered.len(), 1);
		assert!(filtered[0].path().is_ident("derive"));
	}

	#[test]
	fn test_filter_document_use() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> =
			vec![parse_quote!(#[document_use = "Of"]), parse_quote!(#[inline])];

		let filtered: Vec<_> = filter_doc_attrs(&attrs).collect();

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
	fn test_should_keep_attr() {
		use syn::parse_quote;
		let document_default: Attribute = parse_quote!(#[document_default]);
		let derive: Attribute = parse_quote!(#[derive(Debug)]);

		assert!(!should_keep_attr(&document_default));
		assert!(should_keep_attr(&derive));
	}

	#[test]
	fn test_filter_empty() {
		let attrs: Vec<Attribute> = vec![];
		let filtered: Vec<_> = filter_doc_attrs(&attrs).collect();
		assert_eq!(filtered.len(), 0);
	}

	#[test]
	fn test_filter_all_doc_attrs() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> =
			vec![parse_quote!(#[document_default]), parse_quote!(#[document_use = "Of"])];

		let filtered: Vec<_> = filter_doc_attrs(&attrs).collect();
		assert_eq!(filtered.len(), 0);
	}

	#[test]
	fn test_filter_no_doc_attrs() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> = vec![parse_quote!(#[derive(Debug)]), parse_quote!(#[inline])];

		let filtered: Vec<_> = filter_doc_attrs(&attrs).collect();
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
		assert!(has_attr(&item.attrs, "derive"));
		assert!(!has_attr(&item.attrs, "test_attr"));

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
	fn test_remove_and_parse_attribute() {
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

		let result = remove_and_parse_attribute::<TestArgs>(&mut item.attrs, "test_attr").unwrap();

		assert!(result.is_some());
		let (idx, args) = result.unwrap();
		assert_eq!(idx, 0);
		assert_eq!(args.value.value(), "hello");
		assert_eq!(item.attrs.len(), 1);
		assert!(!has_attr(&item.attrs, "test_attr"));
	}

	#[test]
	fn test_remove_and_parse_attribute_not_found() {
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
		let result =
			remove_and_parse_attribute::<TestArgs>(&mut item.attrs, "nonexistent").unwrap();

		assert!(result.is_none());
		assert_eq!(item.attrs.len(), original_len);
	}
}
