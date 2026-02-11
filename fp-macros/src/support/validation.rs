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
				format!(
					"Missing documentation for {context} `{expected_name}`. All {context}s must be documented."
				),
			)));
		}
	}

	// Check that no extra entries are documented
	for provided_name in provided.keys() {
		if !expected.iter().any(|e| e == provided_name) {
			return Err(CoreError::Parse(syn::Error::new(
				provided_name.span(),
				format!(
					"{} `{provided_name}` does not exist. Available {context}s: {}",
					capitalize_first(context),
					expected.iter().map(|f| format!("`{f}`")).collect::<Vec<_>>().join(", ")
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
			format!("Duplicate documentation for {context} `{name}`"),
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

/// Helper function to capitalize the first character of a string.
fn capitalize_first(s: &str) -> String {
	let mut chars = s.chars();
	match chars.next() {
		None => String::new(),
		Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
	}
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
}
