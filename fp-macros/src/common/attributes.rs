//! Utilities for working with attributes in procedural macros.
//!
//! This module provides reusable functions for finding and extracting information
//! from attributes on Rust items.

use syn::{Attribute, Error, Result, spanned::Spanned};

/// Finds the index of the first attribute with the given name.
///
/// ### Parameters
///
/// * `attrs` - The attributes to search
/// * `name` - The name of the attribute to find
///
/// ### Returns
///
/// The index of the first matching attribute, or `None` if not found.
pub fn find_attribute(
	attrs: &[Attribute],
	name: &str,
) -> Option<usize> {
	attrs.iter().position(|attr| attr.path().is_ident(name))
}

/// Checks if an attribute with the given name exists.
///
/// ### Parameters
///
/// * `attrs` - The attributes to search
/// * `name` - The name of the attribute to check
///
/// ### Returns
///
/// `true` if the attribute exists, `false` otherwise.
pub fn has_attr(
	attrs: &[Attribute],
	name: &str,
) -> bool {
	attrs.iter().any(|attr| attr.path().is_ident(name))
}

/// Finds a string value from a name-value attribute, with duplicate checking.
///
/// For example, this extracts `"SomeType"` from `#[doc_use = "SomeType"]`.
///
/// ### Parameters
///
/// * `attrs` - The attributes to search
/// * `name` - The name of the attribute to find
///
/// ### Returns
///
/// * `Ok(Some(value))` if exactly one matching attribute with a string value is found
/// * `Ok(None)` if no matching attribute is found
/// * `Err(...)` if multiple matching attributes are found
///
/// ### Errors
///
/// Returns an error if multiple attributes with the same name are found.
pub fn find_attr_value_checked(
	attrs: &[Attribute],
	name: &str,
) -> Result<Option<String>> {
	let mut found = None;
	for attr in attrs {
		if attr.path().is_ident(name) {
			if found.is_some() {
				return Err(Error::new(
					attr.span(),
					format!("Multiple `#[{}]` attributes found on same item", name),
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
