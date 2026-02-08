//! Pattern detection and parsing for HM type conversion.
//!
//! This module handles:
//! - `Kind!` and `def_kind!` macro input parsing
//! - FnBrand pattern detection and extraction
//! - Apply! macro pattern detection and extraction

use crate::{
	hkt::ApplyInput,
	config::Config,
	common::errors::known_types,
	error::{Error, UnsupportedFeature},
};
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
	Attribute, GenericArgument, GenericParam, Generics, Ident, PathArguments, Token, TypeParamBound,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
};

use crate::analysis::traits::{TraitCategory, classify_trait};

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
	/// Attributes (e.g., doc comments).
	pub attrs: Vec<Attribute>,
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
		let mut assoc_types: Vec<KindAssocTypeInput> = Vec::new();
		while !input.is_empty() {
			assoc_types.push(input.parse()?);
		}
		
		// Validation: non-empty
		if assoc_types.is_empty() {
			return Err(Error::validation(
				Span::call_site(),
				"Kind definition must have at least one associated type"
			).into());
		}
		
		// Validation: no const generics
		for assoc in &assoc_types {
			for param in &assoc.generics.params {
				if let GenericParam::Const(const_param) = param {
					return Err(Error::Unsupported(
						UnsupportedFeature::ConstGenerics {
							span: const_param.ident.span()
						}
					).into());
				}
			}
		}
		
		Ok(KindInput { assoc_types })
	}
}

impl ToTokens for KindInput {
	fn to_tokens(
		&self,
		tokens: &mut proc_macro2::TokenStream,
	) {
		for assoc in &self.assoc_types {
			assoc.to_tokens(tokens);
		}
	}
}

impl ToTokens for KindAssocTypeInput {
	fn to_tokens(
		&self,
		tokens: &mut proc_macro2::TokenStream,
	) {
		for attr in &self.attrs {
			attr.to_tokens(tokens);
		}
		self._type_token.to_tokens(tokens);
		self.ident.to_tokens(tokens);
		self.generics.to_tokens(tokens);
		if let Some(colon) = &self._colon_token {
			colon.to_tokens(tokens);
			self.output_bounds.to_tokens(tokens);
		}
		self._semi_token.to_tokens(tokens);
	}
}

impl Parse for KindAssocTypeInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let attrs = input.call(Attribute::parse_outer)?;
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
			attrs,
			_type_token: type_token,
			ident,
			generics,
			_colon_token: colon_token,
			output_bounds,
			_semi_token: semi_token,
		})
	}
}

// ============================================================================
// FnBrand and Apply! Pattern Detection
// ============================================================================

/// Helper structure to hold the result of FnBrand detection from a TypePath.
///
/// FnBrands (like CloneableFn, SendCloneableFn, Function) encode function types
/// using associated type syntax. This structure contains the extracted type arguments.
pub struct FnBrandInfo {
	/// Type arguments extracted from the FnBrand (excluding the return type)
	pub inputs: Vec<syn::Type>,
	/// The return type (last type argument)
	pub output: syn::Type,
}

/// Attempts to extract FnBrand information from a TypePath with QSelf.
///
/// FnBrands use the pattern `<Brand as Trait>::Apply<Input1, Input2, ..., Output>`
/// where the last type argument is the return type and earlier arguments are inputs.
///
/// ### Returns
/// `Some(FnBrandInfo)` if this is a valid FnBrand pattern, `None` otherwise.
pub fn extract_fn_brand_info(
	type_path: &syn::TypePath,
	config: &Config,
) -> Option<FnBrandInfo> {
	if let Some(_qself) = &type_path.qself
		&& type_path.path.segments.len() >= 2
	{
		let trait_name = type_path.path.segments[0].ident.to_string();
		if let TraitCategory::FnBrand = classify_trait(&trait_name, config) {
			let last_segment = type_path.path.segments.last()?;
			if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
				let mut type_args: Vec<_> = args
					.args
					.iter()
					.filter_map(|arg| {
						if let GenericArgument::Type(t) = arg { Some(t.clone()) } else { None }
					})
					.collect();

				if !type_args.is_empty() {
					let output = type_args.pop()?;
					return Some(FnBrandInfo { inputs: type_args, output });
				}
			}
		}
	}
	None
}

/// Attempts to parse an Apply! macro invocation and extract its arguments.
///
/// ### Returns
/// `Some((brand, args))` if this is a valid Apply! macro, `None` otherwise.
pub fn extract_apply_macro_info(type_macro: &syn::TypeMacro) -> Option<(syn::Type, Vec<syn::Type>)> {
	if type_macro.mac.path.is_ident(known_types::APPLY_MACRO)
		&& let Ok(apply_input) = syn::parse2::<ApplyInput>(type_macro.mac.tokens.clone())
	{
		let brand = apply_input.brand;
		let args: Vec<_> = apply_input
			.args
			.args
			.iter()
			.filter_map(|arg| {
				if let syn::GenericArgument::Type(t) = arg { Some(t.clone()) } else { None }
			})
			.collect();
		return Some((brand, args));
	}
	None
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

	/// Tests that empty Kind input is rejected with a validation error.
	#[test]
	fn test_parse_kind_input_empty() {
		let input = "";
		let result: syn::Result<KindInput> = parse_str(input);
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
		let result: syn::Result<KindInput> = parse_str(input);
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
		let result: syn::Result<KindInput> = parse_str(input);
		assert!(result.is_err(), "Const generics should be rejected");
		let err_msg = result.unwrap_err().to_string();
		assert!(
			err_msg.contains("Const generic parameters are not supported"),
			"Error message should mention const generics not being supported"
		);
	}
}
