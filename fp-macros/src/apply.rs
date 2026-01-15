//! Implementation of the `Apply!` macro.
//!
//! This module handles the parsing and expansion of the `Apply!` macro, which is used
//! to apply a Higher-Kinded Type (HKT) "brand" to a set of generic arguments.
//!
//! The macro supports two syntaxes:
//! 1. **Named parameters**: `Apply!(brand: MyBrand, signature: (l1), types: (T1))`
//! 2. **Positional arguments**: `Apply!(MyBrand, Kind_..., (l1), (T1))`

use crate::{
	generate::generate_name,
	parse::{KindInput, TypeInput},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
	Ident, Lifetime, Token, Type, parenthesized,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
};

/// Specifies the source of the Kind trait to be used.
pub enum KindSource {
	/// The Kind trait name is generated from a signature.
	Generated(KindInput),
	/// The Kind trait name is explicitly provided.
	Explicit(Type),
}

/// Input structure for the `Apply!` macro.
pub struct ApplyInput {
	/// The brand type to apply (e.g., `OptionBrand`).
	pub brand: Type,
	/// The source of the Kind trait (generated or explicit).
	pub kind_source: KindSource,
	/// Lifetime arguments to apply.
	pub lifetimes: Punctuated<Lifetime, Token![,]>,
	/// Type arguments to apply.
	pub types: Punctuated<Type, Token![,]>,
}

impl Parse for ApplyInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		// Check if named parameters: starts with "brand:"
		// We check if the first token is an Ident and the second is a Colon.
		// Note: Type can also start with Ident, but usually not followed by Colon immediately unless it's a path?
		// But "brand:" is very specific.
		// However, `OptionBrand` is also an Ident. But it's followed by comma in positional args.

		let is_named = input.peek(Ident) && input.peek2(Token![:]);

		if is_named {
			let mut brand = None;
			let mut kind_input = None;
			let mut lifetimes = None;
			let mut types = None;

			while !input.is_empty() {
				let label: Ident = input.parse()?;
				input.parse::<Token![:]>()?;

				if label == "brand" {
					brand = Some(input.parse()?);
				} else if label == "signature" {
					kind_input = Some(parse_signature(input)?);
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
				kind_source: KindSource::Generated(
					kind_input.ok_or_else(|| input.error("Missing 'signature'"))?,
				),
				lifetimes: lifetimes.unwrap_or_default(),
				types: types.unwrap_or_default(),
			})
		} else {
			// Legacy positional arguments: Brand, Kind, (lifetimes), (types)

			let brand: Type = input.parse()?;
			input.parse::<Token![,]>()?;

			let kind_name: Type = input.parse()?;
			input.parse::<Token![,]>()?;

			let content;
			parenthesized!(content in input);
			let lifetimes = content.parse_terminated(Lifetime::parse, Token![,])?;

			input.parse::<Token![,]>()?;

			let content2;
			parenthesized!(content2 in input);
			let types = content2.parse_terminated(Type::parse, Token![,])?;

			Ok(ApplyInput { brand, kind_source: KindSource::Explicit(kind_name), lifetimes, types })
		}
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
	let kind_name = match input.kind_source {
		KindSource::Generated(input) => {
			let name = generate_name(&input);
			quote! { #name }
		}
		KindSource::Explicit(ty) => quote! { #ty },
	};

	let brand = input.brand;
	let lifetimes = input.lifetimes;
	let types = input.types;

	let params = if lifetimes.is_empty() {
		quote! { #types }
	} else if types.is_empty() {
		quote! { #lifetimes }
	} else {
		quote! { #lifetimes, #types }
	};

	quote! {
		<#brand as #kind_name>::Of<#params>
	}
}
