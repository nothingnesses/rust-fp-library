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

		// Create a mapping for lifetime canonicalization
		let mut lifetime_counter = 0;

		for param in &self.params {
			match param {
				SignatureParam::Lifetime(lt) => {
					// Use canonical lifetime names for Kind generation
					let canonical_lt = Lifetime::new(&format!("'_{}", lifetime_counter), lt.span());
					lifetimes.push(canonical_lt);
					lifetime_counter += 1;
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
pub struct ApplyInput {
	/// The brand type to apply (e.g., `OptionBrand`).
	pub brand: Type,
	/// The source of the Kind trait (generated or explicit).
	pub kind_source: KindSource,
}

impl Parse for ApplyInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut brand = None;
		let mut kind_source = None;
		let mut lifetimes = None;
		let mut types = None;

		while !input.is_empty() {
			let label: Ident = input.parse()?;
			input.parse::<Token![:]>()?;

			if label == "brand" {
				brand = Some(input.parse()?);
			} else if label == "signature" {
				if kind_source.is_some() {
					return Err(syn::Error::new(
						label.span(),
						"Cannot specify both 'signature' and 'kind'",
					));
				}
				kind_source = Some(KindSource::Generated(parse_signature(input)?));
			} else if label == "kind" {
				if kind_source.is_some() {
					return Err(syn::Error::new(
						label.span(),
						"Cannot specify both 'signature' and 'kind'",
					));
				}
				kind_source = Some(KindSource::Explicit(input.parse()?));
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

		Ok(ApplyInput {
			brand: brand.ok_or_else(|| input.error("Missing 'brand'"))?,
			kind_source: kind_source.ok_or_else(|| input.error("Missing 'signature' or 'kind'"))?,
			lifetimes: lifetimes.unwrap_or_default(),
			types: types.unwrap_or_default(),
		})
	}
}

fn parse_signature(input: ParseStream) -> syn::Result<KindInput> {
	let content;
	parenthesized!(content in input);

	let mut lifetimes = Punctuated::new();
	let mut types = Punctuated::new();

	while !content.is_empty() {
		if content.peek(Lifetime) {
			lifetimes.push(content.parse()?);
		} else {
			let ident: Ident = content.parse()?;
			let mut bounds = Punctuated::new();
			if content.peek(Token![:]) {
				content.parse::<Token![:]>()?;
				loop {
					if content.peek(Token![,]) || content.is_empty() {
						break;
					}
					bounds.push_value(content.parse()?);
					if content.peek(Token![+]) {
						bounds.push_punct(content.parse()?);
					} else {
						break;
					}
				}
			}
			types.push(TypeInput { ident, bounds });
		}

		if content.peek(Token![,]) {
			content.parse::<Token![,]>()?;
		}
	}

	let mut output_bounds = Punctuated::new();
	if input.peek(Token![->]) {
		input.parse::<Token![->]>()?;
		loop {
			if input.peek(Token![,]) || input.is_empty() {
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

	Ok(KindInput { lifetimes, types, output_bounds })
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
	// Apply! Named Parameter Tests (Original)
	// ===========================================================================

	/// Tests parsing of Apply! with named parameters.
	///
	/// Verifies that the parser correctly extracts brand, signature, lifetimes,
	/// and types from the named parameter syntax.
	#[test]
	fn test_parse_apply() {
		let input = "brand: OptionBrand, signature: ('a, A: 'a) -> 'a, lifetimes: ('a), types: (A)";
		let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse ApplyInput");

		assert_eq!(parsed.lifetimes.len(), 1);
		assert_eq!(parsed.types.len(), 1);
	}

	/// Tests code generation for Apply! with named parameters.
	///
	/// Verifies that the generated code projects the brand to its concrete type
	/// using the correct Kind trait.
	#[test]
	fn test_apply_generation() {
		let input = "brand: OptionBrand, signature: ('a, A: 'a) -> 'a, lifetimes: ('a), types: (A)";
		let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse ApplyInput");

		let output = apply_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("< OptionBrand as Kind_"));
		assert!(output_str.contains(":: Of < 'a , A >"));
	}

	// ===========================================================================
	// Apply! Explicit Kind Tests (Named Parameters)
	// ===========================================================================

	/// Tests parsing of Apply! with explicit kind using named parameters.
	///
	/// Verifies that the parser correctly handles the named syntax:
	/// `brand: Brand, kind: Kind, lifetimes: (lifetimes), types: (types)`
	/// and uses KindSource::Explicit.
	#[test]
	fn test_apply_explicit_kind_parsing() {
		let input = "brand: OptionBrand, kind: SomeKind, lifetimes: ('a), types: (String)";
		let parsed: ApplyInput =
			syn::parse_str(input).expect("Failed to parse ApplyInput explicit kind");

		assert_eq!(parsed.lifetimes.len(), 1);
		assert_eq!(parsed.types.len(), 1);

		// Should use explicit kind source
		match parsed.kind_source {
			KindSource::Explicit(ty) => {
				assert_eq!(quote!(#ty).to_string(), "SomeKind");
			}
			KindSource::Generated(_) => panic!("Expected explicit kind source"),
		}
	}

	/// Tests code generation for Apply! with explicit kind.
	///
	/// Verifies that the generated projection uses the explicitly provided
	/// Kind trait name rather than generating one.
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

	/// Tests Apply! explicit kind syntax with no lifetimes.
	///
	/// Verifies that empty lifetime parentheses are handled correctly.
	#[test]
	fn test_apply_explicit_kind_no_lifetimes() {
		let input = "brand: MyBrand, kind: MyKind, lifetimes: (), types: (T, U)";
		let parsed: ApplyInput =
			syn::parse_str(input).expect("Failed to parse ApplyInput explicit kind");

		assert_eq!(parsed.lifetimes.len(), 0);
		assert_eq!(parsed.types.len(), 2);

		let output = apply_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("< MyBrand as MyKind >"));
		assert!(output_str.contains(":: Of < T , U >"));
	}

	/// Tests Apply! explicit kind syntax with no type arguments.
	///
	/// Verifies that empty type parentheses are handled correctly
	/// when only lifetimes are provided.
	#[test]
	fn test_apply_explicit_kind_no_types() {
		let input = "brand: MyBrand, kind: MyKind, lifetimes: ('a, 'b), types: ()";
		let parsed: ApplyInput =
			syn::parse_str(input).expect("Failed to parse ApplyInput explicit kind");

		assert_eq!(parsed.lifetimes.len(), 2);
		assert_eq!(parsed.types.len(), 0);

		let output = apply_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("< MyBrand as MyKind >"));
		assert!(output_str.contains(":: Of < 'a , 'b >"));
	}
}
