//! Implementation of the `Apply!` macro.
//!
//! This module handles the parsing and expansion of the `Apply!` macro, which is used
//! to apply a Higher-Kinded Type (HKT) "brand" to a set of generic arguments.

use crate::{
	generate::generate_name,
	parse::{KindAssocTypeInput, KindInput},
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
	GenericParam, Generics, Ident, Lifetime, LifetimeParam, Token, Type, TypeParam, TypeParamBound,
	parenthesized,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
};

/// A parameter in the unified signature syntax.
#[derive(Debug)]
pub enum SignatureParam {
	/// A lifetime value (e.g., 'static, 'a)
	Lifetime(Lifetime),
	/// A type value with optional bounds (e.g., String: Clone)
	Type { ty: Box<Type>, bounds: Punctuated<TypeParamBound, Token![+]> },
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
	pub fn to_kind_input(
		&self,
		assoc_name: &Ident,
	) -> KindInput {
		let mut params = Punctuated::new();

		// Add lifetimes
		for param in &self.params {
			if let SignatureParam::Lifetime(lt) = param {
				params.push(GenericParam::Lifetime(LifetimeParam {
					attrs: Vec::new(),
					lifetime: lt.clone(),
					colon_token: None,
					bounds: Punctuated::new(),
				}));
			}
		}

		// Add types
		let mut t_idx = 0;
		for param in &self.params {
			if let SignatureParam::Type { bounds, .. } = param {
				let ident = format_ident!("T{}", t_idx as usize);
				t_idx += 1;
				params.push(GenericParam::Type(TypeParam {
					attrs: Vec::new(),
					ident,
					colon_token: if bounds.is_empty() {
						None
					} else {
						Some(Token![:](Span::call_site()))
					},
					bounds: bounds.clone(),
					eq_token: None,
					default: None,
				}));
			}
		}

		let generics = Generics {
			lt_token: Some(Token![<](Span::call_site())),
			params,
			gt_token: Some(Token![>](Span::call_site())),
			where_clause: None,
		};

		let assoc = KindAssocTypeInput {
			_type_token: Token![type](Span::call_site()),
			ident: assoc_name.clone(),
			generics,
			_colon_token: if self.output_bounds.is_empty() {
				None
			} else {
				Some(Token![:](Span::call_site()))
			},
			output_bounds: self.output_bounds.clone(),
			_semi_token: Token![;](Span::call_site()),
		};

		KindInput { assoc_types: vec![assoc] }
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
				SignatureParam::Type { ty, .. } => Some(ty.as_ref()),
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
		kind: Box<Type>,
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
	/// Optional associated type name for the output (defaults to `Of`).
	pub output: Option<Ident>,
}

impl Parse for ApplyInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut brand = None;
		let mut kind_source_type = None; // To track if we saw 'signature' or 'kind'
		let mut signature = None;
		let mut kind = None;
		let mut lifetimes = None;
		let mut types = None;
		let mut output = None;

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
				kind = Some(Box::new(input.parse()?));
			} else if label == "lifetimes" {
				let content;
				parenthesized!(content in input);
				lifetimes = Some(content.parse_terminated(Lifetime::parse, Token![,])?);
			} else if label == "types" {
				let content;
				parenthesized!(content in input);
				types = Some(content.parse_terminated(Type::parse, Token![,])?);
			} else if label == "output" {
				output = Some(input.parse()?);
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

		Ok(ApplyInput { brand, kind_source, output })
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

			params.push(SignatureParam::Type { ty: Box::new(ty), bounds });
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
pub fn apply_impl(input: ApplyInput) -> TokenStream {
	let brand = &input.brand;
	let assoc_type = input.output.clone().unwrap_or_else(|| Ident::new("Of", Span::call_site()));

	let (kind_name, params) = match &input.kind_source {
		KindSource::Generated(sig) => {
			let kind_input = sig.to_kind_input(&assoc_type);
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
		<#brand as #kind_name>::#assoc_type<#params>
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// ===========================================================================
	// Apply! Unified Signature Tests
	// ===========================================================================

	#[test]
	fn test_parse_apply_unified() {
		let input = "brand: OptionBrand, signature: ('a, A: 'a) -> 'a";
		let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse ApplyInput");

		match parsed.kind_source {
			KindSource::Generated(sig) => {
				assert_eq!(sig.params.len(), 2);
			}
			_ => panic!("Expected generated kind source"),
		}
	}

	#[test]
	fn test_apply_generation_unified() {
		let input = "brand: OptionBrand, signature: ('a, A: 'a) -> 'a";
		let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse ApplyInput");

		let output = apply_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("< OptionBrand as Kind_"));
		assert!(output_str.contains(":: Of < 'a , A >"));
	}

	#[test]
	fn test_to_kind_input_conversion() {
		let input = "brand: B, signature: ('a, T: Clone + 'a) -> Debug";
		let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse");

		match parsed.kind_source {
			KindSource::Generated(sig) => {
				let kind_input = sig.to_kind_input(&Ident::new("Of", Span::call_site()));

				assert_eq!(kind_input.assoc_types.len(), 1);
				let assoc = &kind_input.assoc_types[0];
				assert_eq!(assoc.ident.to_string(), "Of");
				assert_eq!(assoc.generics.params.len(), 2);
			}
			_ => panic!("Expected generated kind source"),
		}
	}
}
