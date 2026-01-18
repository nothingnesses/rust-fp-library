//! Input parsing for `Kind` macros.
//!
//! This module defines the input structures and parsing logic for the `Kind!` and `def_kind!` macros.
//! It handles parsing of associated type definitions with generics and bounds.

use syn::{
	Generics, Ident, Token, TypeParamBound,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
};

/// Represents the parsed input for a `Kind` signature.
///
/// This structure captures a list of associated type definitions.
#[derive(Debug)]
pub struct KindInput {
	/// The list of associated type definitions.
	pub assoc_types: Vec<KindAssocTypeInput>,
}

/// Represents a single associated type definition in a `Kind` signature.
///
/// Example: `type Of<'a, T: 'a>: Display;`
#[derive(Debug)]
pub struct KindAssocTypeInput {
	/// The `type` keyword.
	pub _type_token: Token![type],
	/// The name of the associated type (e.g., `Of`).
	pub ident: Ident,
	/// Generics for the associated type (e.g., `<'a, T: 'a>`).
	pub generics: Generics,
	/// Optional colon for output bounds.
	pub _colon_token: Option<Token![:]>,
	/// Bounds on the associated type output (e.g., `Display`).
	pub output_bounds: Punctuated<TypeParamBound, Token![+]>,
	/// The semicolon terminating the definition.
	pub _semi_token: Token![;],
}

impl Parse for KindInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut assoc_types = Vec::new();
		while !input.is_empty() {
			assoc_types.push(input.parse()?);
		}
		Ok(KindInput { assoc_types })
	}
}

impl Parse for KindAssocTypeInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let type_token: Token![type] = input.parse()?;
		let ident: Ident = input.parse()?;
		let generics: Generics = input.parse()?;

		let mut colon_token: Option<Token![:]> = None;
		let mut output_bounds = Punctuated::new();

		if input.peek(Token![:]) {
			colon_token = Some(input.parse()?);
			// Parse bounds separated by `+` until `;`
			loop {
				if input.peek(Token![;]) {
					break;
				}
				output_bounds.push_value(input.parse()?);
				if input.peek(Token![+]) {
					output_bounds.push_punct(input.parse()?);
				} else {
					break;
				}
			}
		}

		let semi_token: Token![;] = input.parse()?;

		Ok(KindAssocTypeInput {
			_type_token: type_token,
			ident,
			generics,
			_colon_token: colon_token,
			output_bounds,
			_semi_token: semi_token,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use syn::parse_str;

	/// Tests parsing of a simple `Kind` signature with one associated type.
	///
	/// Verifies that:
	/// - The associated type name is parsed correctly.
	/// - Generics (lifetimes and types) are parsed.
	/// - Output bounds are parsed.
	#[test]
	fn test_parse_kind_input_simple() {
		let input = "type Of<'a, T>: Display;";
		let parsed: KindInput = parse_str(input).expect("Failed to parse");

		assert_eq!(parsed.assoc_types.len(), 1);
		let assoc = &parsed.assoc_types[0];

		// Check identifier
		assert_eq!(assoc.ident.to_string(), "Of");

		// Check generics: 'a and T
		assert_eq!(assoc.generics.params.len(), 2);

		// Check output bounds: Display
		assert_eq!(assoc.output_bounds.len(), 1);
	}

	/// Tests parsing of a `Kind` signature with multiple associated types.
	///
	/// Verifies that:
	/// - Multiple `type ...;` definitions are parsed into a list.
	/// - Each definition retains its specific properties (name, generics, bounds).
	#[test]
	fn test_parse_kind_input_multiple() {
		let input = "
			type Of<'a, T: 'a>: Display;
			type SendOf<U>: Send;
		";
		let parsed: KindInput = parse_str(input).expect("Failed to parse");

		assert_eq!(parsed.assoc_types.len(), 2);

		// Check first associated type
		let assoc1 = &parsed.assoc_types[0];
		assert_eq!(assoc1.ident.to_string(), "Of");
		assert_eq!(assoc1.generics.params.len(), 2);

		// Check second associated type
		let assoc2 = &parsed.assoc_types[1];
		assert_eq!(assoc2.ident.to_string(), "SendOf");
		assert_eq!(assoc2.generics.params.len(), 1);
	}
}
