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

	/// Comprehensive parsing tests covering various signature patterns.
	#[test]
	fn test_parse_variations() {
		// Helper to parse and get signature
		fn parse_sig(sig_str: &str) -> UnifiedSignature {
			let input = format!("brand: B, signature: {}", sig_str);
			let parsed: ApplyInput = syn::parse_str(&input).expect(&format!("Failed to parse: {}", sig_str));
			match parsed.kind_source {
				KindSource::Generated(sig) => sig,
				_ => panic!("Expected generated kind source"),
			}
		}

		// Simple lifetime: ('a)
		let sig = parse_sig("('a)");
		assert_eq!(sig.params.len(), 1);
		match &sig.params[0] {
			SignatureParam::Lifetime(lt) => assert_eq!(lt.to_string(), "'a"),
			_ => panic!("Expected lifetime"),
		}

		// Simple type: (T)
		let sig = parse_sig("(T)");
		assert_eq!(sig.params.len(), 1);
		match &sig.params[0] {
			SignatureParam::Type { ty, bounds } => {
				assert_eq!(quote!(#ty).to_string(), "T");
				assert!(bounds.is_empty());
			}
			_ => panic!("Expected type"),
		}

		// Type with bounds: (T: Clone)
		let sig = parse_sig("(T: Clone)");
		match &sig.params[0] {
			SignatureParam::Type { bounds, .. } => {
				assert_eq!(bounds.len(), 1);
				assert_eq!(quote!(#bounds).to_string(), "Clone");
			}
			_ => panic!("Expected type"),
		}

		// Multiple bounds: (T: Clone + Send)
		let sig = parse_sig("(T: Clone + Send)");
		match &sig.params[0] {
			SignatureParam::Type { bounds, .. } => {
				assert_eq!(bounds.len(), 2);
				assert_eq!(quote!(#bounds).to_string(), "Clone + Send");
			}
			_ => panic!("Expected type"),
		}

		// Lifetime bound: (T: 'a)
		let sig = parse_sig("(T: 'a)");
		match &sig.params[0] {
			SignatureParam::Type { bounds, .. } => {
				assert_eq!(bounds.len(), 1);
				assert_eq!(quote!(#bounds).to_string(), "'a");
			}
			_ => panic!("Expected type"),
		}

		// Mixed parameters: ('a, T: Clone)
		let sig = parse_sig("('a, T: Clone)");
		assert_eq!(sig.params.len(), 2);
		match &sig.params[0] {
			SignatureParam::Lifetime(lt) => assert_eq!(lt.to_string(), "'a"),
			_ => panic!("Expected lifetime"),
		}
		match &sig.params[1] {
			SignatureParam::Type { ty, bounds } => {
				assert_eq!(quote!(#ty).to_string(), "T");
				assert_eq!(quote!(#bounds).to_string(), "Clone");
			}
			_ => panic!("Expected type"),
		}

		// Complex type: (Vec<String>: Clone)
		let sig = parse_sig("(Vec<String>: Clone)");
		match &sig.params[0] {
			SignatureParam::Type { ty, .. } => {
				assert_eq!(quote!(#ty).to_string(), "Vec < String >");
			}
			_ => panic!("Expected type"),
		}

		// Reference type: (&'a str: Display)
		let sig = parse_sig("(&'a str: Display)");
		match &sig.params[0] {
			SignatureParam::Type { ty, .. } => {
				assert_eq!(quote!(#ty).to_string(), "& 'a str");
			}
			_ => panic!("Expected type"),
		}

		// Output bounds: ('a, T) -> Debug
		let sig = parse_sig("('a, T) -> Debug");
		assert_eq!(sig.output_bounds.len(), 1);
		let output_bounds = &sig.output_bounds;
		assert_eq!(quote!(#output_bounds).to_string(), "Debug");

		// Multiple output bounds: ('a, T) -> Debug + Clone
		let sig = parse_sig("('a, T) -> Debug + Clone");
		assert_eq!(sig.output_bounds.len(), 2);
		let output_bounds = &sig.output_bounds;
		assert_eq!(quote!(#output_bounds).to_string(), "Debug + Clone");
	}

	/// Tests extraction of KindInput and concrete values from UnifiedSignature.
	#[test]
	fn test_extraction_variations() {
		// Helper to parse and get signature
		fn parse_sig(sig_str: &str) -> UnifiedSignature {
			let input = format!("brand: B, signature: {}", sig_str);
			let parsed: ApplyInput = syn::parse_str(&input).expect(&format!("Failed to parse: {}", sig_str));
			match parsed.kind_source {
				KindSource::Generated(sig) => sig,
				_ => panic!("Expected generated kind source"),
			}
		}

		// Case: ('a, T: Clone)
		let sig = parse_sig("('a, T: Clone)");
		
		// Test to_kind_input()
		let kind_input = sig.to_kind_input();
		assert_eq!(kind_input.lifetimes.len(), 1);
		assert_eq!(kind_input.lifetimes[0].to_string(), "'a");
		
		assert_eq!(kind_input.types.len(), 1);
		assert_eq!(kind_input.types[0].ident.to_string(), "T0"); // Canonicalized
		let bounds = &kind_input.types[0].bounds;
		assert_eq!(quote!(#bounds).to_string(), "Clone");

		// Test concrete_lifetimes()
		let concrete_lts = sig.concrete_lifetimes();
		assert_eq!(concrete_lts.len(), 1);
		assert_eq!(concrete_lts[0].to_string(), "'a");

		// Test concrete_types()
		let concrete_tys = sig.concrete_types();
		assert_eq!(concrete_tys.len(), 1);
		let ty0 = concrete_tys[0];
		assert_eq!(quote!(#ty0).to_string(), "T");

		// Case: (Vec<T>: Debug, &'a str)
		let sig = parse_sig("(Vec<T>: Debug, &'a str)");
		
		let kind_input = sig.to_kind_input();
		assert_eq!(kind_input.types.len(), 2);
		assert_eq!(kind_input.types[0].ident.to_string(), "T0");
		let bounds0 = &kind_input.types[0].bounds;
		assert_eq!(quote!(#bounds0).to_string(), "Debug");
		assert_eq!(kind_input.types[1].ident.to_string(), "T1");
		assert!(kind_input.types[1].bounds.is_empty());

		let concrete_tys = sig.concrete_types();
		assert_eq!(concrete_tys.len(), 2);
		let ty0 = concrete_tys[0];
		assert_eq!(quote!(#ty0).to_string(), "Vec < T >");
		let ty1 = concrete_tys[1];
		assert_eq!(quote!(#ty1).to_string(), "& 'a str");
	}

	/// Tests code generation for various scenarios.
	#[test]
	fn test_generation_variations() {
		// Helper to generate output string
		fn generate(input_str: &str) -> String {
			let parsed: ApplyInput = syn::parse_str(input_str).expect("Failed to parse");
			apply_impl(parsed).to_string()
		}

		// Unified syntax: ('a, T)
		let output = generate("brand: B, signature: ('a, T)");
		assert!(output.contains("< B as Kind_"));
		assert!(output.contains(":: Of < 'a , T >"));

		// Unified syntax with bounds: (T: Clone)
		// Bounds affect the Kind name but not the projection arguments
		let output = generate("brand: B, signature: (T: Clone)");
		assert!(output.contains("< B as Kind_"));
		assert!(output.contains(":: Of < T >"));

		// Unified syntax with complex types: (Vec<T>)
		let output = generate("brand: B, signature: (Vec<T>)");
		assert!(output.contains("< B as Kind_"));
		assert!(output.contains(":: Of < Vec < T > >"));

		// Explicit kind syntax
		let output = generate("brand: B, kind: MyKind, lifetimes: ('a), types: (T)");
		assert!(output.contains("< B as MyKind >"));
		assert!(output.contains(":: Of < 'a , T >"));

		// Explicit kind syntax (only types)
		let output = generate("brand: B, kind: MyKind, lifetimes: (), types: (T)");
		assert!(output.contains("< B as MyKind >"));
		assert!(output.contains(":: Of < T >"));

		// Explicit kind syntax (only lifetimes)
		let output = generate("brand: B, kind: MyKind, lifetimes: ('a), types: ()");
		assert!(output.contains("< B as MyKind >"));
		assert!(output.contains(":: Of < 'a >"));
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

	// ===========================================================================
	// Integration Tests
	// ===========================================================================

	/// Tests that `to_kind_input()` correctly converts a UnifiedSignature.
	#[test]
	fn test_to_kind_input_conversion() {
		let input = "brand: B, signature: ('a, T: Clone + 'a) -> Debug";
		let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse");

		match parsed.kind_source {
			KindSource::Generated(sig) => {
				let kind_input = sig.to_kind_input();

				// Check lifetimes
				assert_eq!(kind_input.lifetimes.len(), 1);
				assert_eq!(kind_input.lifetimes[0].to_string(), "'a");

				// Check types
				assert_eq!(kind_input.types.len(), 1);
				// Type name should be canonicalized to T0
				assert_eq!(kind_input.types[0].ident.to_string(), "T0");
				// Bounds should be preserved
				assert_eq!(kind_input.types[0].bounds.len(), 2);

				// Check output bounds
				assert_eq!(kind_input.output_bounds.len(), 1);
				let output_bounds = &kind_input.output_bounds;
				assert_eq!(quote!(#output_bounds).to_string(), "Debug");
			}
			_ => panic!("Expected generated kind source"),
		}
	}

	/// Tests that `generate_name()` works correctly with the output of `to_kind_input()`.
	#[test]
	fn test_generate_name_integration() {
		let input = "brand: B, signature: ('a, T: 'a)";
		let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse");

		match parsed.kind_source {
			KindSource::Generated(sig) => {
				let kind_input = sig.to_kind_input();
				let name = generate_name(&kind_input);
				assert!(name.to_string().starts_with("Kind_"));
			}
			_ => panic!("Expected generated kind source"),
		}
	}

	/// Tests that `generate_name()` produces consistent names for alpha-equivalent signatures.
	#[test]
	fn test_generate_name_consistency() {
		// Case 1: ('a, T: 'a)
		let input1 = "brand: B, signature: ('a, T: 'a)";
		let parsed1: ApplyInput = syn::parse_str(input1).expect("Failed to parse 1");
		let sig1 = match parsed1.kind_source {
			KindSource::Generated(s) => s,
			_ => panic!("Expected generated kind source"),
		};
		let name1 = generate_name(&sig1.to_kind_input());

		// Case 2: ('b, U: 'b) - alpha equivalent
		let input2 = "brand: B, signature: ('b, U: 'b)";
		let parsed2: ApplyInput = syn::parse_str(input2).expect("Failed to parse 2");
		let sig2 = match parsed2.kind_source {
			KindSource::Generated(s) => s,
			_ => panic!("Expected generated kind source"),
		};
		let name2 = generate_name(&sig2.to_kind_input());

		assert_eq!(name1, name2, "Alpha-equivalent signatures should produce same Kind name");
	}
}
