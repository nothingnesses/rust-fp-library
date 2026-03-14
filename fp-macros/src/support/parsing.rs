//! Common parsing patterns and input validation helpers.

use {
	crate::{
		core::{
			Error,
			Result,
			constants::attributes::DOCUMENT_PARAMETERS,
		},
		support::documentation_parameters::DocumentationParameter,
	},
	proc_macro2::{
		Span,
		TokenStream,
	},
	syn::{
		GenericParam,
		Generics,
		parse::{
			Parse,
			ParseStream,
		},
	},
};

/// Parse a stream of items until it's empty.
pub fn parse_many<T: Parse>(input: ParseStream) -> syn::Result<Vec<T>> {
	let mut items = Vec::new();
	while !input.is_empty() {
		items.push(input.parse()?);
	}
	Ok(items)
}

/// Validates that a collection is not empty, returning the collection if valid.
pub fn parse_non_empty<T>(
	items: Vec<T>,
	message: &str,
) -> Result<Vec<T>> {
	if items.is_empty() {
		return Err(Error::validation(Span::call_site(), message));
	}
	Ok(items)
}

/// Try parsing as multiple types, calling a different handler for each successful parse.
///
/// This is useful for documentation macros that can apply to different item types
/// (e.g., structs vs enums, functions vs impl blocks) where each type needs different handling.
///
/// # Parameters
/// - `tokens`: The token stream to parse
/// - `attempts`: A vector of (parser, handler) pairs to try in order
/// - `error_msg`: Error message to return if all parsing attempts fail
///
/// # Example
/// ```ignore
/// parse_with_dispatch(
///     item_tokens,
///     vec![
///         Box::new(|t| syn::parse2::<ItemEnum>(t).map(|e| handle_enum(attr.clone(), e))),
///         Box::new(|t| syn::parse2::<ItemStruct>(t).map(|s| handle_struct(attr.clone(), s))),
///     ],
///     "Expected struct or enum"
/// )
/// ```
pub fn parse_with_dispatch<T>(
	tokens: TokenStream,
	alternatives: Vec<Box<dyn Fn(TokenStream) -> syn::Result<T>>>,
	error_msg: &str,
) -> syn::Result<T> {
	for attempt in alternatives {
		if let Ok(result) = attempt(tokens.clone()) {
			return Ok(result);
		}
	}
	Err(syn::Error::new(Span::call_site(), error_msg))
}

/// Parse generics to ensure these don't contain unsupported features
pub fn parse_generics(generics: &Generics) -> Result<&Generics> {
	for param in &generics.params {
		if let GenericParam::Const(const_param) = param {
			return Err(Error::validation(
				const_param.ident.span(),
				"Const generic parameters are not supported in Kind definitions",
			)
			.with_suggestion("Remove const parameters or use a different approach"));
		}
	}
	Ok(generics)
}

/// Validates that a token stream of attributes is empty.
pub fn parse_empty_attributes(attrs: TokenStream) -> Result<TokenStream> {
	if !attrs.is_empty() {
		return Err(Error::validation(Span::call_site(), "This macro does not accept attributes"));
	}
	Ok(attrs)
}

/// Parses entry count, returning both expected and provided if they match.
///
/// # Parameters
/// - `expected`: The expected number of entries
/// - `provided`: The actual number of entries provided
/// - `span`: The span for error reporting
/// - `context`: A string describing what is being validated (e.g., "field", "parameter")
///
/// # Returns
/// - `Ok((expected, provided))` if counts match
/// - `Err` with a descriptive error if counts don't match
pub fn parse_entry_count(
	expected: usize,
	provided: usize,
	span: Span,
	context: &str,
) -> Result<(usize, usize)> {
	if expected != provided {
		return Err(Error::Parse(syn::Error::new(
			span,
			format!(
				"Expected exactly {expected} description arguments (one for each {context}), found {provided}. All {context}s must be documented."
			),
		)));
	}
	Ok((expected, provided))
}

/// Generic helper to validate that a count is non-zero.
///
/// Returns the count if valid, or an error with a custom message.
///
/// # Parameters
/// - `count`: The count to validate
/// - `span`: The span for error reporting
/// - `error_fn`: A closure that generates the error message if count is zero
///
/// # Returns
/// - `Ok(count)` if count > 0
/// - `Err` with the error message from `error_fn` if count == 0
pub fn parse_non_zero_count<F>(
	count: usize,
	span: Span,
	error_fn: F,
) -> Result<usize>
where
	F: FnOnce() -> String, {
	if count == 0 {
		return Err(Error::Parse(syn::Error::new(span, error_fn())));
	}
	Ok(count)
}

/// Parses that documentable items are provided (not empty).
///
/// Returns the count if valid.
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
/// - `Ok(count)` if count > 0
/// - `Err` with a descriptive error if count == 0
pub fn parse_has_documentable_items(
	count: usize,
	span: Span,
	attr_name: &str,
	item_description: &str,
) -> Result<usize> {
	parse_non_zero_count(count, span, || format!("Cannot use #[{attr_name}] on {item_description}"))
}

/// Parses and validates parameter documentation pairs.
///
/// Takes parameter names (targets) and their documentation entries, validates they match,
/// and returns the paired data.
///
/// Returns the validated pairs of (parameter_name, documentation_entry) if successful.
///
/// # Errors
/// - No documentable parameters exist (including self-only functions)
/// - Number of descriptions doesn't match number of parameters
pub fn parse_parameter_documentation_pairs(
	targets: Vec<String>,
	entries: Vec<DocumentationParameter>,
	span: Span,
) -> Result<Vec<(String, DocumentationParameter)>> {
	let expected = targets.len();
	let found = entries.len();

	// Error when using the macro on functions with no documentable parameters
	parse_has_documentable_items(
		expected,
		span,
		DOCUMENT_PARAMETERS,
		r#"functions with no parameters to document.
	 Note: `self` parameters (including `&self` and `&mut self`) are not considered documentable parameters. Remove this attribute or add parameters to the function."#,
	)?;

	// Validate counts match
	parse_entry_count(expected, found, span, "parameter")?;

	Ok(targets.into_iter().zip(entries).collect())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_many() {
		use syn::parse::Parser;
		let input = "u32 i32 f64";
		let parser = |input: ParseStream| parse_many::<syn::Type>(input);
		let result = parser.parse_str(input);

		assert!(result.is_ok());
		let items = result.unwrap();
		assert_eq!(items.len(), 3);
	}

	#[test]
	fn test_parse_non_empty() {
		let items = vec![1, 2, 3];
		let result = parse_non_empty(items, "Error");
		assert!(result.is_ok());
		assert_eq!(result.unwrap().len(), 3);

		let empty: Vec<i32> = vec![];
		let result = parse_non_empty(empty, "Should not be empty");
		assert!(result.is_err());
		assert_eq!(result.unwrap_err().to_string(), "Validation error: Should not be empty");
	}

	#[test]
	fn test_parse_generics_with_const() {
		use syn::parse_quote;
		let generics: Generics = parse_quote!(<const N: usize>);
		let result = parse_generics(&generics);
		assert!(result.is_err());
	}

	#[test]
	fn test_parse_generics_without_const() {
		use syn::parse_quote;
		let generics: Generics = parse_quote!(<T, U>);
		let result = parse_generics(&generics);
		assert!(result.is_ok());
		let returned_generics = result.unwrap();
		assert_eq!(returned_generics.params.len(), 2);
	}

	#[test]
	fn test_parse_empty_attributes() {
		let empty = TokenStream::new();
		assert!(parse_empty_attributes(empty).is_ok());

		let not_empty = quote::quote!(#[attr]);
		assert!(parse_empty_attributes(not_empty).is_err());
	}

	#[test]
	fn test_parse_entry_count_matches() {
		let result = parse_entry_count(3, 3, Span::call_site(), "field");
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), (3, 3));
	}

	#[test]
	fn test_parse_entry_count_mismatch() {
		let result = parse_entry_count(3, 2, Span::call_site(), "field");
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Expected exactly 3"));
		assert!(error.contains("found 2"));
	}

	#[test]
	fn test_parse_has_documentable_items_ok() {
		let result = parse_has_documentable_items(
			1,
			Span::call_site(),
			"document_parameters",
			"functions with no parameters",
		);
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), 1);
	}

	#[test]
	fn test_parse_has_documentable_items_zero() {
		let result = parse_has_documentable_items(
			0,
			Span::call_site(),
			"document_parameters",
			"functions with no parameters",
		);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Cannot use #[document_parameters]"));
		assert!(error.contains("functions with no parameters"));
	}

	#[test]
	fn test_parse_non_zero_count_ok() {
		let result =
			parse_non_zero_count(5, Span::call_site(), || "Should not see this".to_string());
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), 5);
	}

	#[test]
	fn test_parse_non_zero_count_zero() {
		let result =
			parse_non_zero_count(0, Span::call_site(), || "Count cannot be zero".to_string());
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Count cannot be zero"));
	}
}
