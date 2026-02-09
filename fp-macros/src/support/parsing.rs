//! Common parsing patterns and input validation helpers.

use crate::core::{Error, Result};
use proc_macro2::Span;
use syn::{GenericParam, Generics, Token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

/// Parse a comma-separated list of items
#[allow(dead_code)] // For future use
pub fn parse_comma_separated<T: Parse>(input: ParseStream) -> syn::Result<Vec<T>> {
	let punctuated = Punctuated::<T, Token![,]>::parse_terminated(input)?;
	Ok(punctuated.into_iter().collect())
}

/// Parse an optional item
#[allow(dead_code)] // For future use
pub fn parse_optional<T: Parse>(input: ParseStream) -> syn::Result<Option<T>> {
	if input.is_empty() { Ok(None) } else { Ok(Some(input.parse()?)) }
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

/// Parse a list to ensure it is non-empty
#[allow(dead_code)] // For future use
pub fn parse_non_empty<'a, T>(
	items: &'a [T],
	span: Span,
	what: &str,
) -> Result<&'a [T]> {
	if items.is_empty() {
		return Err(Error::validation(span, format!("{} must not be empty", what)));
	}
	Ok(items)
}

#[cfg(test)]
mod tests {
	use super::*;
	use syn::Ident;

	#[test]
	fn test_parse_comma_separated() {
		use syn::parse::Parser;
		let parser = |input: ParseStream| parse_comma_separated::<Ident>(input);
		let result: Vec<Ident> = parser.parse_str("a, b, c").unwrap();
		assert_eq!(result.len(), 3);
		assert_eq!(result[0].to_string(), "a");
		assert_eq!(result[1].to_string(), "b");
		assert_eq!(result[2].to_string(), "c");
	}

	#[test]
	fn test_parse_optional_some() {
		use syn::parse::Parser;
		let parser = |input: ParseStream| parse_optional::<Ident>(input);
		let result: Option<Ident> = parser.parse_str("test").unwrap();
		assert!(result.is_some());
		assert_eq!(result.unwrap().to_string(), "test");
	}

	#[test]
	fn test_parse_optional_none() {
		use syn::parse::Parser;
		let parser = |input: ParseStream| parse_optional::<Ident>(input);
		let result: Option<Ident> = parser.parse_str("").unwrap();
		assert!(result.is_none());
	}

	#[test]
	fn test_parse_non_empty() {
		let items: Vec<i32> = vec![];
		let result = parse_non_empty(&items, Span::call_site(), "Items");
		assert!(result.is_err());

		let items = vec![1, 2, 3];
		let result = parse_non_empty(&items, Span::call_site(), "Items");
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), &[1, 2, 3]);
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
}
