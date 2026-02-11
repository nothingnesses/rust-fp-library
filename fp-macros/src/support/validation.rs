/// Validation utilities for documentation macros.
///
/// This module provides reusable validation functions for checking counts,
/// detecting duplicates, and validating field mappings.
use crate::core::Error as CoreError;
use std::collections::HashMap;
use syn::{Ident, LitStr};

/// Validates that the number of provided entries matches the expected count.
///
/// # Parameters
/// - `expected`: The expected number of entries
/// - `provided`: The actual number of entries provided
/// - `span`: The span for error reporting
/// - `context`: A string describing what is being validated (e.g., "field", "parameter")
///
/// # Returns
/// - `Ok(())` if counts match
/// - `Err` with a descriptive error if counts don't match
pub fn validate_entry_count(
	expected: usize,
	provided: usize,
	span: proc_macro2::Span,
	context: &str,
) -> Result<(), CoreError> {
	if expected != provided {
		return Err(CoreError::Parse(syn::Error::new(
			span,
			format!(
				"Expected {expected} description arguments (one for each {context}), found {provided}. All {context}s must be documented."
			),
		)));
	}
	Ok(())
}

/// Validates a mapping of named entries, checking for completeness and extra entries.
///
/// # Parameters
/// - `expected`: Vec of expected field names
/// - `provided`: HashMap of provided field names to descriptions
/// - `span`: The span for error reporting
/// - `context`: A string describing what is being validated (e.g., "field", "parameter")
///
/// # Returns
/// - `Ok(())` if all expected fields are present and no extra fields exist
/// - `Err` with a descriptive error otherwise
pub fn validate_named_entries(
	expected: &[Ident],
	provided: &HashMap<Ident, LitStr>,
	span: proc_macro2::Span,
	context: &str,
) -> Result<(), CoreError> {
	// Check that all expected entries have documentation
	for expected_name in expected {
		if !provided.contains_key(expected_name) {
			return Err(CoreError::Parse(syn::Error::new(
				span,
				format_missing_doc_error(context, &expected_name.to_string()),
			)));
		}
	}

	// Check that no extra entries are documented
	for provided_name in provided.keys() {
		if !expected.iter().any(|e| e == provided_name) {
			return Err(CoreError::Parse(syn::Error::new(
				provided_name.span(),
				format_nonexistent_item_error(
					context,
					&provided_name.to_string(),
					&expected.iter().map(|e| e.to_string()).collect::<Vec<_>>(),
				),
			)));
		}
	}

	Ok(())
}

/// Checks for duplicate entries in a HashMap during insertion.
///
/// This is typically used during HashMap building to ensure no duplicates are inserted.
///
/// # Parameters
/// - `name`: The identifier being checked
/// - `existing_value`: The result of HashMap::insert (Some if duplicate, None if new)
/// - `context`: A string describing what is being validated
///
/// # Returns
/// - `Ok(())` if no duplicate
/// - `Err` with a descriptive error if duplicate found
pub fn check_duplicate_entry<T>(
	name: &Ident,
	existing_value: Option<T>,
	context: &str,
) -> Result<(), CoreError> {
	if existing_value.is_some() {
		return Err(CoreError::Parse(syn::Error::new(
			name.span(),
			format_duplicate_doc_error(context, &name.to_string()),
		)));
	}
	Ok(())
}

/// Validates that a type has at least one field (not zero-sized).
///
/// # Parameters
/// - `field_count`: The number of fields
/// - `span`: The span for error reporting
/// - `type_kind`: A string describing the type (e.g., "struct", "variant")
/// - `attribute_name`: The name of the attribute being applied
///
/// # Returns
/// - `Ok(())` if field_count > 0
/// - `Err` if field_count == 0
pub fn validate_not_zero_sized(
	field_count: usize,
	span: proc_macro2::Span,
	type_kind: &str,
	attribute_name: &str,
) -> Result<(), CoreError> {
	if field_count == 0 {
		return Err(CoreError::Parse(syn::Error::new(
			span,
			format!(
				"{attribute_name} cannot be used on zero-sized types ({type_kind}s with no fields)"
			),
		)));
	}
	Ok(())
}

/// Validates that documentation arguments are provided (not empty).
///
/// This is used to check that a macro requiring documentation is not used
/// on items with nothing to document (e.g., functions with no parameters).
///
/// # Parameters
/// - `count`: The number of items to document
/// - `span`: The span for error reporting
/// - `attr_name`: The name of the attribute (e.g., "document_parameters")
/// - `item_description`: A description of what cannot be documented
///
/// # Returns
/// - `Ok(())` if count > 0
/// - `Err` with a descriptive error if count == 0
pub fn validate_has_documentable_items(
	count: usize,
	span: proc_macro2::Span,
	attr_name: &str,
	item_description: &str,
) -> Result<(), CoreError> {
	if count == 0 {
		return Err(CoreError::Parse(syn::Error::new(
			span,
			format!("Cannot use #[{attr_name}] on {item_description}"),
		)));
	}
	Ok(())
}

/// Validates parameter documentation count with a standardized error message.
///
/// This is a specialized version of `validate_entry_count` with error messages
/// tailored for parameter documentation.
///
/// # Parameters
/// - `expected`: The expected number of parameters
/// - `provided`: The actual number of descriptions provided
/// - `span`: The span for error reporting
///
/// # Returns
/// - `Ok(())` if counts match
/// - `Err` with a descriptive error if counts don't match
pub fn validate_parameter_doc_count(
	expected: usize,
	provided: usize,
	span: proc_macro2::Span,
) -> Result<(), CoreError> {
	if expected != provided {
		return Err(CoreError::Parse(syn::Error::new(
			span,
			format!(
				"Expected {} description argument{}, found {}.",
				expected,
				if expected == 1 { "" } else { "s" },
				provided
			),
		)));
	}
	Ok(())
}

/// Helper function to capitalize the first character of a string.
fn capitalize_first(s: &str) -> String {
	let mut chars = s.chars();
	match chars.next() {
		None => String::new(),
		Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
	}
}

/// Format a missing documentation error message.
///
/// # Parameters
/// - `context`: A string describing what is being validated (e.g., "field", "parameter")
/// - `name`: The name of the item missing documentation
///
/// # Returns
/// A formatted error message string
pub fn format_missing_doc_error(
	context: &str,
	name: &str,
) -> String {
	format!("Missing documentation for {context} `{name}`. All {context}s must be documented.")
}

/// Format a duplicate documentation error message.
///
/// # Parameters
/// - `context`: A string describing what is being validated (e.g., "field", "parameter")
/// - `name`: The name of the item with duplicate documentation
///
/// # Returns
/// A formatted error message string
pub fn format_duplicate_doc_error(
	context: &str,
	name: &str,
) -> String {
	format!("Duplicate documentation for {context} `{name}`")
}

/// Format a non-existent item error message.
///
/// # Parameters
/// - `context`: A string describing what is being validated (e.g., "field", "parameter")
/// - `name`: The name of the non-existent item
/// - `available`: List of available item names
///
/// # Returns
/// A formatted error message string
pub fn format_nonexistent_item_error(
	context: &str,
	name: &str,
	available: &[impl std::fmt::Display],
) -> String {
	format!(
		"{} `{name}` does not exist. Available {context}s: {}",
		capitalize_first(context),
		available.iter().map(|f| format!("`{f}`")).collect::<Vec<_>>().join(", ")
	)
}

#[cfg(test)]
mod tests {
	use super::*;
	use quote::format_ident;

	#[test]
	fn test_validate_entry_count_matches() {
		let result = validate_entry_count(3, 3, proc_macro2::Span::call_site(), "field");
		assert!(result.is_ok());
	}

	#[test]
	fn test_validate_entry_count_mismatch() {
		let result = validate_entry_count(3, 2, proc_macro2::Span::call_site(), "field");
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Expected 3"));
		assert!(error.contains("found 2"));
	}

	#[test]
	fn test_validate_named_entries_complete() {
		let expected = vec![format_ident!("x"), format_ident!("y")];
		let mut provided = HashMap::new();
		provided.insert(format_ident!("x"), syn::parse_quote!("X coord"));
		provided.insert(format_ident!("y"), syn::parse_quote!("Y coord"));

		let result =
			validate_named_entries(&expected, &provided, proc_macro2::Span::call_site(), "field");
		assert!(result.is_ok());
	}

	#[test]
	fn test_validate_named_entries_missing() {
		let expected = vec![format_ident!("x"), format_ident!("y")];
		let mut provided = HashMap::new();
		provided.insert(format_ident!("x"), syn::parse_quote!("X coord"));

		let result =
			validate_named_entries(&expected, &provided, proc_macro2::Span::call_site(), "field");
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Missing documentation for field `y`"));
	}

	#[test]
	fn test_validate_named_entries_extra() {
		let expected = vec![format_ident!("x")];
		let mut provided = HashMap::new();
		provided.insert(format_ident!("x"), syn::parse_quote!("X coord"));
		provided.insert(format_ident!("z"), syn::parse_quote!("Z coord"));

		let result =
			validate_named_entries(&expected, &provided, proc_macro2::Span::call_site(), "field");
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Field `z` does not exist"));
	}

	#[test]
	fn test_check_duplicate_entry_no_duplicate() {
		let name = format_ident!("x");
		let result = check_duplicate_entry(&name, None::<String>, "field");
		assert!(result.is_ok());
	}

	#[test]
	fn test_check_duplicate_entry_duplicate() {
		let name = format_ident!("x");
		let result = check_duplicate_entry(&name, Some("previous"), "field");
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Duplicate documentation for field `x`"));
	}

	#[test]
	fn test_validate_not_zero_sized_ok() {
		let result = validate_not_zero_sized(
			1,
			proc_macro2::Span::call_site(),
			"struct",
			"#[document_fields]",
		);
		assert!(result.is_ok());
	}

	#[test]
	fn test_validate_not_zero_sized_zero() {
		let result = validate_not_zero_sized(
			0,
			proc_macro2::Span::call_site(),
			"struct",
			"#[document_fields]",
		);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("zero-sized types"));
	}

	#[test]
	fn test_validate_has_documentable_items_ok() {
		let result = validate_has_documentable_items(
			1,
			proc_macro2::Span::call_site(),
			"document_parameters",
			"functions with no parameters",
		);
		assert!(result.is_ok());
	}

	#[test]
	fn test_validate_has_documentable_items_zero() {
		let result = validate_has_documentable_items(
			0,
			proc_macro2::Span::call_site(),
			"document_parameters",
			"functions with no parameters",
		);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Cannot use #[document_parameters]"));
		assert!(error.contains("functions with no parameters"));
	}

	#[test]
	fn test_validate_parameter_doc_count_matches() {
		let result = validate_parameter_doc_count(2, 2, proc_macro2::Span::call_site());
		assert!(result.is_ok());
	}

	#[test]
	fn test_validate_parameter_doc_count_mismatch() {
		let result = validate_parameter_doc_count(3, 1, proc_macro2::Span::call_site());
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Expected 3 description arguments"));
		assert!(error.contains("found 1"));
	}

	#[test]
	fn test_validate_parameter_doc_count_singular() {
		let result = validate_parameter_doc_count(1, 2, proc_macro2::Span::call_site());
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Expected 1 description argument")); // singular
		assert!(!error.contains("arguments")); // not plural
	}
}
