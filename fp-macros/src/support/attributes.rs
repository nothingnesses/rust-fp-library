//! Attribute parsing and filtering utilities.
//!
//! This module provides utilities for parsing, filtering, and working with attributes,
//! including documentation-specific attributes like `doc_default` and `doc_use`.

use crate::core::{Error, Result};
use proc_macro2::{Span, TokenStream};
use std::collections::HashSet;
use syn::{Attribute, spanned::Spanned};

/// Parser for macro attributes with validation
pub struct AttributeParser {
	allowed: HashSet<&'static str>,
}

impl AttributeParser {
	/// Create a new AttributeParser with allowed attribute names
	pub fn new(allowed: &[&'static str]) -> Self {
		AttributeParser { allowed: allowed.iter().copied().collect() }
	}

	/// Validate that a token stream is empty (no attributes)
	pub fn validate_empty(
		&self,
		attrs: TokenStream,
	) -> Result<()> {
		if !attrs.is_empty() {
			return Err(Error::validation(
				Span::call_site(),
				"This macro does not accept attributes",
			));
		}
		Ok(())
	}
}

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

/// Utility for filtering documentation-specific attributes.
///
/// This centralizes the logic for identifying and filtering out attributes
/// that are specific to the documentation generation system, preventing
/// duplication across the codebase.
pub struct DocAttributeFilter;

impl DocAttributeFilter {
	/// Returns true if the attribute should be kept in generated code.
	///
	/// This filters out documentation-specific attributes like `doc_default`
	/// and `doc_use` which are processed by the macro system but should not
	/// appear in the final generated code.
	///
	/// # Examples
	///
	/// ```ignore
	/// use syn::parse_quote;
	///
	/// let attr: Attribute = parse_quote!(#[doc_default]);
	/// assert!(!DocAttributeFilter::should_keep(&attr));
	///
	/// let attr: Attribute = parse_quote!(#[derive(Debug)]);
	/// assert!(DocAttributeFilter::should_keep(&attr));
	/// ```
	pub fn should_keep(attr: &Attribute) -> bool {
		!Self::is_doc_specific(attr)
	}

	/// Returns true if the attribute is documentation-specific.
	///
	/// Documentation-specific attributes include:
	/// - `doc_default`: Marks an associated type as the default for resolution
	/// - `doc_use`: Specifies which associated type to use for documentation
	///
	/// # Examples
	///
	/// ```ignore
	/// use syn::parse_quote;
	///
	/// let attr: Attribute = parse_quote!(#[doc_default]);
	/// assert!(DocAttributeFilter::is_doc_specific(&attr));
	///
	/// let attr: Attribute = parse_quote!(#[doc_use = "Of"]);
	/// assert!(DocAttributeFilter::is_doc_specific(&attr));
	/// ```
	pub fn is_doc_specific(attr: &Attribute) -> bool {
		attr.path().is_ident("doc_default") || attr.path().is_ident("doc_use")
	}

	/// Filters out documentation-specific attributes from a slice.
	///
	/// This is a convenience method that returns an iterator over attributes
	/// that should be kept (i.e., are not documentation-specific).
	///
	/// # Examples
	///
	/// ```ignore
	/// use syn::{Attribute, parse_quote};
	///
	/// let attrs: Vec<Attribute> = vec![
	///     parse_quote!(#[doc_default]),
	///     parse_quote!(#[derive(Debug)]),
	///     parse_quote!(#[doc_use = "Of"]),
	/// ];
	///
	/// let filtered: Vec<_> = DocAttributeFilter::filter_doc_attrs(&attrs).collect();
	/// assert_eq!(filtered.len(), 1); // Only #[derive(Debug)] remains
	/// ```
	pub fn filter_doc_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
		attrs.iter().filter(|attr| Self::should_keep(attr))
	}
}

/// Finds a string value from a name-value attribute, with duplicate checking.
///
/// For example, this extracts `"SomeType"` from `#[doc_use = "SomeType"]`.
pub fn find_attr_value_checked(
	attrs: &[Attribute],
	name: &str,
) -> Result<Option<String>> {
	let mut found = None;
	for attr in attrs {
		if attr.path().is_ident(name) {
			if found.is_some() {
				return Err(Error::validation(
					attr.span(),
					format!("Multiple `#[{}]` attributes found on same item", name),
				));
			}
			if let syn::Meta::NameValue(nv) = &attr.meta {
				if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
					found = Some(s.value());
				}
			}
		}
	}
	Ok(found)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_attribute_parser_validate_empty() {
		let parser = AttributeParser::new(&[]);
		let empty = TokenStream::new();
		assert!(parser.validate_empty(empty).is_ok());
	}

	#[test]
	fn test_has_attr() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> = vec![parse_quote!(#[doc = "test"])];
		assert!(has_attr(&attrs, "doc"));
		assert!(!has_attr(&attrs, "test"));
	}

	#[test]
	fn test_filter_doc_default() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> =
			vec![parse_quote!(#[doc_default]), parse_quote!(#[derive(Debug)])];

		let filtered: Vec<_> = DocAttributeFilter::filter_doc_attrs(&attrs).collect();

		assert_eq!(filtered.len(), 1);
		assert!(filtered[0].path().is_ident("derive"));
	}

	#[test]
	fn test_filter_doc_use() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> = vec![parse_quote!(#[doc_use = "Of"]), parse_quote!(#[inline])];

		let filtered: Vec<_> = DocAttributeFilter::filter_doc_attrs(&attrs).collect();

		assert_eq!(filtered.len(), 1);
		assert!(filtered[0].path().is_ident("inline"));
	}

	#[test]
	fn test_is_doc_specific() {
		use syn::parse_quote;
		let doc_default: Attribute = parse_quote!(#[doc_default]);
		let doc_use: Attribute = parse_quote!(#[doc_use = "Of"]);
		let derive: Attribute = parse_quote!(#[derive(Debug)]);

		assert!(DocAttributeFilter::is_doc_specific(&doc_default));
		assert!(DocAttributeFilter::is_doc_specific(&doc_use));
		assert!(!DocAttributeFilter::is_doc_specific(&derive));
	}

	#[test]
	fn test_should_keep() {
		use syn::parse_quote;
		let doc_default: Attribute = parse_quote!(#[doc_default]);
		let derive: Attribute = parse_quote!(#[derive(Debug)]);

		assert!(!DocAttributeFilter::should_keep(&doc_default));
		assert!(DocAttributeFilter::should_keep(&derive));
	}

	#[test]
	fn test_filter_empty() {
		let attrs: Vec<Attribute> = vec![];
		let filtered: Vec<_> = DocAttributeFilter::filter_doc_attrs(&attrs).collect();
		assert_eq!(filtered.len(), 0);
	}

	#[test]
	fn test_filter_all_doc_attrs() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> =
			vec![parse_quote!(#[doc_default]), parse_quote!(#[doc_use = "Of"])];

		let filtered: Vec<_> = DocAttributeFilter::filter_doc_attrs(&attrs).collect();
		assert_eq!(filtered.len(), 0);
	}

	#[test]
	fn test_filter_no_doc_attrs() {
		use syn::parse_quote;
		let attrs: Vec<Attribute> = vec![parse_quote!(#[derive(Debug)]), parse_quote!(#[inline])];

		let filtered: Vec<_> = DocAttributeFilter::filter_doc_attrs(&attrs).collect();
		assert_eq!(filtered.len(), 2);
	}
}
