//! Common parsing patterns and input validation helpers.

use crate::{
	core::{Error, Result, constants::attributes::DOCUMENT_PARAMETERS},
	support::syntax::DocArg,
};
use proc_macro2::{Span, TokenStream};
use syn::{
	GenericParam, Generics,
	parse::{Parse, ParseStream},
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

/// Helper to try parsing multiple types and return the first successful one.
pub fn parse_first<T, F>(
	item: TokenStream,
	parsers: Vec<F>,
	error_msg: &str,
) -> syn::Result<T>
where
	F: Fn(TokenStream) -> syn::Result<T>,
{
	for parser in parsers {
		if let Ok(val) = parser(item.clone()) {
			return Ok(val);
		}
	}
	Err(syn::Error::new(Span::call_site(), error_msg))
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
/// try_parse_one_of(
///     item_tokens,
///     vec![
///         Box::new(|t| syn::parse2::<ItemEnum>(t).map(|e| handle_enum(attr.clone(), e))),
///         Box::new(|t| syn::parse2::<ItemStruct>(t).map(|s| handle_struct(attr.clone(), s))),
///     ],
///     "Expected struct or enum"
/// )
/// ```
pub fn try_parse_one_of<T>(
	tokens: TokenStream,
	attempts: Vec<Box<dyn Fn(TokenStream) -> syn::Result<T>>>,
	error_msg: &str,
) -> syn::Result<T> {
	for attempt in attempts {
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
	entries: Vec<DocArg>,
	span: Span,
) -> Result<Vec<(String, DocArg)>> {
	let expected = targets.len();
	let found = entries.len();

	// Error when using the macro on functions with no documentable parameters
	if expected == 0 {
		return Err(Error::validation(
			span,
			format!(
				r"Cannot use #[{DOCUMENT_PARAMETERS}] on functions with no parameters to document.
  Note: `self` parameters (including `&self` and `&mut self`) are not considered documentable parameters. Remove this attribute or add parameters to the function."
			),
		));
	}

	if expected != found {
		return Err(Error::validation(
			span,
			format!(
				"Expected {expected} description argument{}, found {found}.",
				if expected == 1 { "" } else { "s" }
			),
		));
	}

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
}
