//! Input parsing for Kind macros.
//!
//! This module defines the input structures and parsing logic for the `Kind!` and `def_kind!` macros.
//! It handles parsing of lifetimes, type parameters with bounds, and output bounds.

use syn::{
	Ident, Lifetime, Result, Token, TypeParamBound,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
};

/// Represents the parsed input for a Kind signature.
///
/// This structure captures the lifetimes, type parameters (with bounds), and output bounds
/// that define a Higher-Kinded Type signature.
pub struct KindInput {
	/// Lifetimes involved in the signature.
	pub lifetimes: Punctuated<Lifetime, Token![,]>,
	/// Type parameters involved in the signature.
	pub types: Punctuated<TypeInput, Token![,]>,
	/// Bounds on the output type.
	pub output_bounds: Punctuated<TypeParamBound, Token![+]>,
}

/// Represents a single type parameter in a Kind signature.
pub struct TypeInput {
	/// The identifier of the type parameter.
	pub ident: Ident,
	/// Bounds on the type parameter.
	pub bounds: Punctuated<TypeParamBound, Token![+]>,
}

impl Parse for KindInput {
	fn parse(input: ParseStream) -> Result<Self> {
		let content;
		let _ = syn::parenthesized!(content in input);
		let lifetimes = content.parse_terminated(Lifetime::parse, Token![,])?;

		input.parse::<Token![,]>()?;

		let content;
		let _ = syn::parenthesized!(content in input);
		let types = content.parse_terminated(TypeInput::parse, Token![,])?;

		input.parse::<Token![,]>()?;

		let content;
		let _ = syn::parenthesized!(content in input);
		let output_bounds = content.parse_terminated(TypeParamBound::parse, Token![+])?;

		Ok(KindInput { lifetimes, types, output_bounds })
	}
}

impl Parse for TypeInput {
	fn parse(input: ParseStream) -> Result<Self> {
		let ident: Ident = input.parse()?;
		let bounds = if input.peek(Token![:]) {
			input.parse::<Token![:]>()?;
			// Manual parsing loop to ensure we stop at comma or closing parenthesis
			let mut bounds = Punctuated::new();
			loop {
				if input.peek(Token![,]) || input.is_empty() {
					break;
				}
				bounds.push_value(input.parse()?);
				if input.peek(Token![+]) {
					bounds.push_punct(input.parse()?);
				} else {
					break;
				}
			}
			bounds
		} else {
			Punctuated::new()
		};
		Ok(TypeInput { ident, bounds })
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use syn::parse_str;

	/// Tests parsing of a simple Kind signature.
	///
	/// Verifies that the parser correctly handles a signature with:
	/// - One lifetime ('a)
	/// - One type parameter (T)
	/// - No output bounds
	#[test]
	fn test_parse_kind_input_simple() {
		let input = "('a), (T), ()";
		let parsed: KindInput = parse_str(input).expect("Failed to parse");
		assert_eq!(parsed.lifetimes.len(), 1);
		assert_eq!(parsed.types.len(), 1);
		assert!(parsed.output_bounds.is_empty());
	}

	/// Tests parsing of a complex Kind signature.
	///
	/// Verifies that the parser correctly handles:
	/// - Multiple lifetimes ('a, 'b)
	/// - Multiple type parameters with bounds (T: Clone + Send, U)
	/// - Output bounds (std::fmt::Debug)
	#[test]
	fn test_parse_kind_input_complex() {
		let input = "('a, 'b), (T: Clone + Send, U), (std::fmt::Debug)";
		let parsed: KindInput = parse_str(input).expect("Failed to parse");
		assert_eq!(parsed.lifetimes.len(), 2);
		assert_eq!(parsed.types.len(), 2);
		assert_eq!(parsed.output_bounds.len(), 1);

		// Check types
		let types: Vec<_> = parsed.types.iter().collect();
		assert_eq!(types[0].ident.to_string(), "T");
		assert_eq!(types[0].bounds.len(), 2);
		assert_eq!(types[1].ident.to_string(), "U");
		assert!(types[1].bounds.is_empty());
	}

	/// Tests parsing of an empty Kind signature.
	///
	/// Verifies that the parser handles empty lists for all components:
	/// - No lifetimes
	/// - No type parameters
	/// - No output bounds
	#[test]
	fn test_parse_kind_input_empty() {
		let input = "(), (), ()";
		let parsed: KindInput = parse_str(input).expect("Failed to parse");
		assert!(parsed.lifetimes.is_empty());
		assert!(parsed.types.is_empty());
		assert!(parsed.output_bounds.is_empty());
	}

	#[test]
	fn test_parse_type_input_with_bounds() {
		let input = "T: Clone + Send";
		let parsed: TypeInput = parse_str(input).expect("Failed to parse TypeInput");
		assert_eq!(parsed.ident.to_string(), "T");
		assert_eq!(parsed.bounds.len(), 2);
	}
}
