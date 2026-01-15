//! Implementation of the `Apply!` macro.
//!
//! This module handles the parsing and expansion of the `Apply!` macro, which is used
//! to apply a Higher-Kinded Type (HKT) "brand" to a set of generic arguments.
//!
//! The macro uses named parameters syntax:
//! - `Apply!(brand: MyBrand, signature: ('a, T), lifetimes: ('a), types: (T))`
//! - `Apply!(brand: MyBrand, kind: SomeKind, lifetimes: ('a), types: (T))`
//!
//! Parameters:
//! - `brand`: The brand type to apply (required)
//! - `signature` OR `kind`: Either a signature to generate the Kind trait name, or an explicit Kind trait (required, mutually exclusive)
//! - `lifetimes`: Lifetime arguments to apply (optional)
//! - `types`: Type arguments to apply (optional)

use crate::{
	generate::generate_name,
	parse::{KindInput, TypeInput},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
	Ident, Lifetime, Token, Type, TypeParamBound, parenthesized,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
};

/// A parameter in the unified signature syntax.
#[derive(Debug)]
pub enum SignatureParam {
	/// A lifetime value (e.g., 'static, 'a)
	Lifetime(Lifetime),
	/// A type value with optional bounds (e.g., String: Clone)
	Type {
		ty: Type,
		bounds: Punctuated<TypeParamBound, Token![+]>,
	},
}

/// Parsed unified signature containing both schema and values.
#[derive(Debug)]
pub struct UnifiedSignature {
	/// Parameters (lifetimes and types with bounds)
	pub params: Vec<SignatureParam>,
	/// Output bounds (e.g., -> Debug)
	pub output_bounds: Punctuated<TypeParamBound, Token![+]>,
}

impl UnifiedSignature {
	/// Convert to KindInput for name generation.
	pub fn to_kind_input(&self) -> KindInput {
		let mut lifetimes = Punctuated::new();
		let mut types = Punctuated::new();

		for param in &self.params {
			match param {
				SignatureParam::Lifetime(lt) => {
					// Use original lifetime names for Kind generation
					// Canonicalizer will map them to indices
					lifetimes.push(lt.clone());
				}
				SignatureParam::Type { bounds, .. } => {
					// Use canonical type names for Kind generation
					let canonical_ident = Ident::new(&format!("T{}", types.len()), Span::call_site());
					types.push(TypeInput {
						ident: canonical_ident,
						bounds: bounds.clone(),
					});
				}
			}
		}

		KindInput {
			lifetimes,
			types,
			output_bounds: self.output_bounds.clone(),
		}
	}

	/// Extract concrete lifetime values for projection.
	pub fn concrete_lifetimes(&self) -> Vec<&Lifetime> {
		self.params
			.iter()
			.filter_map(|p| match p {
				SignatureParam::Lifetime(lt) => Some(lt),
				_ => None,
			})
			.collect()
	}

	/// Extract concrete type values for projection.
	pub fn concrete_types(&self) -> Vec<&Type> {
		self.params
			.iter()
			.filter_map(|p| match p {
				SignatureParam::Type { ty, .. } => Some(ty),
				_ => None,
			})
			.collect()
	}
}

/// Specifies the source of the Kind trait to be used.
#[derive(Debug)]
pub enum KindSource {
	/// The Kind trait name is generated from a signature.
	Generated(UnifiedSignature),
	/// The Kind trait name is explicitly provided.
	Explicit {
		kind: Type,
		lifetimes: Punctuated<Lifetime, Token![,]>,
		types: Punctuated<Type, Token![,]>,
	},
}

/// Input structure for the `Apply!` macro.
#[derive(Debug)]
pub struct ApplyInput {
	/// The brand type to apply (e.g., `OptionBrand`).
	pub brand: Type,
	/// The source of the Kind trait (generated or explicit).
	pub kind_source: KindSource,
}

impl Parse for ApplyInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut brand = None;
		let mut kind_source_type = None; // To track if we saw 'signature' or 'kind'
		let mut signature = None;
		let mut kind = None;
		let mut lifetimes = None;
		let mut types = None;

		while !input.is_empty() {
			let label: Ident = input.parse()?;
			input.parse::<Token![:]>()?;

			if label == "brand" {
				brand = Some(input.parse()?);
			} else if label == "signature" {
				if kind_source_type.is_some() {
					return Err(syn::Error::new(
						label.span(),
						"Cannot specify both 'signature' and 'kind'",
					));
				}
				kind_source_type = Some("signature");
				signature = Some(parse_signature(input)?);
			} else if label == "kind" {
				if kind_source_type.is_some() {
					return Err(syn::Error::new(
						label.span(),
						"Cannot specify both 'signature' and 'kind'",
					));
				}
				kind_source_type = Some("kind");
				kind = Some(input.parse()?);
			} else if label == "lifetimes" {
				let content;
				parenthesized!(content in input);
				lifetimes = Some(content.parse_terminated(Lifetime::parse, Token![,])?);
			} else if label == "types" {
				let content;
				parenthesized!(content in input);
				types = Some(content.parse_terminated(Type::parse, Token![,])?);
			} else {
				return Err(syn::Error::new(label.span(), "Unknown parameter"));
			}

			if input.peek(Token![,]) {
				input.parse::<Token![,]>()?;
			}
		}

		let brand = brand.ok_or_else(|| input.error("Missing 'brand'"))?;
		let source_type =
			kind_source_type.ok_or_else(|| input.error("Missing 'signature' or 'kind'"))?;

		let kind_source = match source_type {
			"signature" => {
				if lifetimes.is_some() {
					return Err(syn::Error::new(
						Span::call_site(),
						"'lifetimes' parameter is not allowed with 'signature'",
					));
				}
				if types.is_some() {
					return Err(syn::Error::new(
						Span::call_site(),
						"'types' parameter is not allowed with 'signature'",
					));
				}
				KindSource::Generated(signature.unwrap())
			}
			"kind" => {
				if lifetimes.is_none() {
					return Err(syn::Error::new(
						Span::call_site(),
						"'lifetimes' parameter is required with 'kind'",
					));
				}
				if types.is_none() {
					return Err(syn::Error::new(
						Span::call_site(),
						"'types' parameter is required with 'kind'",
					));
				}
				KindSource::Explicit {
					kind: kind.unwrap(),
					lifetimes: lifetimes.unwrap(),
					types: types.unwrap(),
				}
			}
			_ => unreachable!(),
		};

		Ok(ApplyInput { brand, kind_source })
	}
}

fn parse_signature(input: ParseStream) -> syn::Result<UnifiedSignature> {
	let content;
	parenthesized!(content in input);

	let mut params = Vec::new();

	while !content.is_empty() {
		if content.peek(Lifetime) {
			// Lifetime parameter: 'a, 'static, etc.
			params.push(SignatureParam::Lifetime(content.parse()?));
		} else {
			// Type parameter: T, String, Vec<u8>, etc.
			let ty: Type = content.parse()?;

			// Optional bounds after ':'
			let bounds = if content.peek(Token![:]) {
				content.parse::<Token![:]>()?;
				parse_bounds(&content)?
			} else {
				Punctuated::new()
			};

			params.push(SignatureParam::Type { ty, bounds });
		}

		// Handle comma separator
		if content.peek(Token![,]) {
			content.parse::<Token![,]>()?;
		}
	}

	// Parse optional output bounds: -> Bound1 + Bound2
	let output_bounds = if input.peek(Token![->]) {
		input.parse::<Token![->]>()?;
		parse_bounds(input)?
	} else {
		Punctuated::new()
	};

	Ok(UnifiedSignature { params, output_bounds })
}

fn parse_bounds(input: ParseStream) -> syn::Result<Punctuated<TypeParamBound, Token![+]>> {
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
	Ok(bounds)
}

/// Generates the implementation for the `Apply!` macro.
///
/// This function takes the parsed input and generates the code to project the
/// brand to its concrete type using the appropriate `Kind` trait.
///
/// # Example Output
///
/// ```ignore
/// <OptionBrand as Kind_...>::Of<T>
/// ```
pub fn apply_impl(input: ApplyInput) -> TokenStream {
	let brand = &input.brand;

	let (kind_name, params) = match &input.kind_source {
		KindSource::Generated(sig) => {
			let kind_input = sig.to_kind_input();
			let name = generate_name(&kind_input);

			let lifetimes = sig.concrete_lifetimes();
			let types = sig.concrete_types();

			let params = if lifetimes.is_empty() {
				quote! { #(#types),* }
			} else if types.is_empty() {
				quote! { #(#lifetimes),* }
			} else {
				quote! { #(#lifetimes),*, #(#types),* }
			};

			(quote! { #name }, params)
		}
		KindSource::Explicit { kind, lifetimes, types } => {
			let params = if lifetimes.is_empty() {
				quote! { #types }
			} else if types.is_empty() {
				quote! { #lifetimes }
			} else {
				quote! { #lifetimes, #types }
			};
			(quote! { #kind }, params)
		}
	};

	quote! {
		<#brand as #kind_name>::Of<#params>
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// ===========================================================================
	// Apply! Unified Signature Tests
	// ===========================================================================

	/// Tests parsing of Apply! with unified signature syntax.
	#[test]
	fn test_parse_apply_unified() {
		let input = "brand: OptionBrand, signature: ('a, A: 'a) -> 'a";
		let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse ApplyInput");

		match parsed.kind_source {
			KindSource::Generated(sig) => {
				assert_eq!(sig.params.len(), 2);
				// Check first param is lifetime
				match &sig.params[0] {
					SignatureParam::Lifetime(lt) => assert_eq!(lt.to_string(), "'a"),
					_ => panic!("Expected lifetime"),
				}
				// Check second param is type
				match &sig.params[1] {
					SignatureParam::Type { ty, bounds } => {
						assert_eq!(quote!(#ty).to_string(), "A");
						assert_eq!(bounds.len(), 1);
					}
					_ => panic!("Expected type"),
				}
			}
			_ => panic!("Expected generated kind source"),
		}
	}

	/// Tests code generation for Apply! with unified signature.
	#[test]
	fn test_apply_generation_unified() {
		let input = "brand: OptionBrand, signature: ('a, A: 'a) -> 'a";
		let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse ApplyInput");

		let output = apply_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("< OptionBrand as Kind_"));
		assert!(output_str.contains(":: Of < 'a , A >"));
	}

	/// Tests parsing of complex types and bounds in signature.
	#[test]
	fn test_parse_signature_complex() {
		let input = "brand: MyBrand, signature: (Vec<T>: Clone + Debug, &'a str: Display)";
		let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse complex signature");

		match parsed.kind_source {
			KindSource::Generated(sig) => {
				assert_eq!(sig.params.len(), 2);
				match &sig.params[0] {
					SignatureParam::Type { ty, bounds } => {
						assert_eq!(quote!(#ty).to_string(), "Vec < T >");
						assert_eq!(bounds.len(), 2);
					}
					_ => panic!("Expected type"),
				}
			}
			_ => panic!("Expected generated kind source"),
		}
	}

	// ===========================================================================
	// Apply! Explicit Kind Tests
	// ===========================================================================

	/// Tests parsing of Apply! with explicit kind.
	#[test]
	fn test_apply_explicit_kind_parsing() {
		let input = "brand: OptionBrand, kind: SomeKind, lifetimes: ('a), types: (String)";
		let parsed: ApplyInput =
			syn::parse_str(input).expect("Failed to parse ApplyInput explicit kind");

		match parsed.kind_source {
			KindSource::Explicit { kind, lifetimes, types } => {
				assert_eq!(quote!(#kind).to_string(), "SomeKind");
				assert_eq!(lifetimes.len(), 1);
				assert_eq!(types.len(), 1);
			}
			KindSource::Generated(_) => panic!("Expected explicit kind source"),
		}
	}

	/// Tests code generation for Apply! with explicit kind.
	#[test]
	fn test_apply_explicit_kind_generation() {
		let input = "brand: OptionBrand, kind: SomeKind, lifetimes: ('a), types: (String)";
		let parsed: ApplyInput =
			syn::parse_str(input).expect("Failed to parse ApplyInput explicit kind");

		let output = apply_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("< OptionBrand as SomeKind >"));
		assert!(output_str.contains(":: Of < 'a , String >"));
	}

	// ===========================================================================
	// Error Case Tests
	// ===========================================================================

	#[test]
	fn test_error_signature_with_lifetimes() {
		let input = "brand: B, signature: (T), lifetimes: ('a)";
		let err = syn::parse_str::<ApplyInput>(input).unwrap_err();
		assert_eq!(err.to_string(), "'lifetimes' parameter is not allowed with 'signature'");
	}

	#[test]
	fn test_error_signature_with_types() {
		let input = "brand: B, signature: (T), types: (U)";
		let err = syn::parse_str::<ApplyInput>(input).unwrap_err();
		assert_eq!(err.to_string(), "'types' parameter is not allowed with 'signature'");
	}

	#[test]
	fn test_error_kind_missing_lifetimes() {
		let input = "brand: B, kind: K, types: (T)";
		let err = syn::parse_str::<ApplyInput>(input).unwrap_err();
		assert_eq!(err.to_string(), "'lifetimes' parameter is required with 'kind'");
	}

	#[test]
	fn test_error_kind_missing_types() {
		let input = "brand: B, kind: K, lifetimes: ('a)";
		let err = syn::parse_str::<ApplyInput>(input).unwrap_err();
		assert_eq!(err.to_string(), "'types' parameter is required with 'kind'");
	}
}
