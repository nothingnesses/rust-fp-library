//! Kind input parsing for HKT macros.
//!
//! This module handles parsing of `Kind!` and `def_kind!` macro input,
//! defining the syntax for Kind trait signatures with associated types.

use super::AssociatedTypeBase;
use crate::support::parsing::{parse_generics, parse_many, parse_non_empty};
use quote::ToTokens;
use syn::{
	Token,
	parse::{Parse, ParseStream},
};

/// Represents the parsed input for a `Kind` signature.
///
/// This structure captures a list of associated type definitions.
#[derive(Debug)]
pub struct AssociatedTypes {
	/// The list of associated type definitions.
	pub associated_types: Vec<AssociatedType>,
}

/// Represents a single associated type definition in a `Kind` signature.
///
/// Example: `type Of<'a, T: 'a>: Display;`
#[derive(Debug)]
pub struct AssociatedType {
	/// The common signature parts.
	pub signature: AssociatedTypeBase,
	/// The semicolon terminating the definition.
	pub semi_token: Token![;],
}

impl Parse for AssociatedTypes {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let assoc_types: Vec<AssociatedType> = parse_many(input)?;

		// Validation: non-empty
		let assoc_types = parse_non_empty(assoc_types, "Kind definition must have at least one associated type")?;

		// Validation: no const generics (using centralized validation)
		for assoc in &assoc_types {
			parse_generics(&assoc.signature.generics)?;
		}

		Ok(AssociatedTypes { associated_types: assoc_types })
	}
}

impl ToTokens for AssociatedTypes {
	fn to_tokens(
		&self,
		tokens: &mut proc_macro2::TokenStream,
	) {
		for assoc in &self.associated_types {
			assoc.to_tokens(tokens);
		}
	}
}

impl ToTokens for AssociatedType {
	fn to_tokens(
		&self,
		tokens: &mut proc_macro2::TokenStream,
	) {
		self.signature.to_tokens(tokens);
		self.semi_token.to_tokens(tokens);
	}
}

impl Parse for AssociatedType {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let signature = AssociatedTypeBase::parse_signature(input, |i| i.peek(Token![;]))?;
		let semi_token: Token![;] = input.parse()?;

		Ok(AssociatedType { signature, semi_token })
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
		let parsed: AssociatedTypes = parse_str(input).expect("Failed to parse");

		assert_eq!(parsed.associated_types.len(), 1);
		let assoc = &parsed.associated_types[0];

		// Check identifier
		assert_eq!(assoc.signature.name.to_string(), "Of");

		// Check generics: 'a and T
		assert_eq!(assoc.signature.generics.params.len(), 2);

		// Check output bounds: Display
		assert_eq!(assoc.signature.output_bounds.len(), 1);
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
		let parsed: AssociatedTypes = parse_str(input).expect("Failed to parse");

		assert_eq!(parsed.associated_types.len(), 2);

		// Check first associated type
		let assoc1 = &parsed.associated_types[0];
		assert_eq!(assoc1.signature.name.to_string(), "Of");
		assert_eq!(assoc1.signature.generics.params.len(), 2);

		// Check second associated type
		let assoc2 = &parsed.associated_types[1];
		assert_eq!(assoc2.signature.name.to_string(), "SendOf");
		assert_eq!(assoc2.signature.generics.params.len(), 1);
	}

	/// Tests that empty Kind input is rejected with a validation error.
	#[test]
	fn test_parse_kind_input_empty() {
		let input = "";
		let result: syn::Result<AssociatedTypes> = parse_str(input);
		assert!(result.is_err(), "Empty input should be rejected");
		let err_msg = result.unwrap_err().to_string();
		assert!(
			err_msg.contains("at least one associated type"),
			"Error message should mention requirement for associated types"
		);
	}

	/// Tests that const generics are rejected with an unsupported feature error.
	#[test]
	fn test_parse_kind_input_const_generics() {
		let input = "type Of<const N: usize>;";
		let result: syn::Result<AssociatedTypes> = parse_str(input);
		assert!(result.is_err(), "Const generics should be rejected");
		let err_msg = result.unwrap_err().to_string();
		assert!(
			err_msg.contains("Const generic parameters are not supported"),
			"Error message should mention const generics not being supported"
		);
	}

	/// Tests that const generics in the second associated type are also caught.
	#[test]
	fn test_parse_kind_input_const_generics_in_second() {
		let input = "
			type Of<T>;
			type Array<const N: usize>;
		";
		let result: syn::Result<AssociatedTypes> = parse_str(input);
		assert!(result.is_err(), "Const generics should be rejected");
		let err_msg = result.unwrap_err().to_string();
		assert!(
			err_msg.contains("Const generic parameters are not supported"),
			"Error message should mention const generics not being supported"
		);
	}
}
