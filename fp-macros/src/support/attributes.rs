//! Attribute parsing and filtering utilities.
//!
//! This module provides utilities for parsing, filtering, and working with attributes,
//! including documentation-specific attributes like `document_default` and `document_use`.

use crate::core::constants::attributes;
use syn::Attribute;

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
}
