//! Implementation of the `Apply!` macro.
//!
//! This module handles the parsing and expansion of the `Apply!` macro, which is used
//! to apply a Higher-Kinded Type (HKT) "brand" to a set of generic arguments.

use crate::{generate::generate_name, parse::KindInput};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
	AngleBracketedGenericArguments, Ident, Token, Type,
	parse::{Parse, ParseStream},
};

/// Input structure for the `Apply!` macro.
///
/// Syntax: `Apply!(<Brand as Kind!( type Of...; )>::Of<T, U>)`
#[derive(Debug)]
pub struct ApplyInput {
	/// The brand type (e.g., `OptionBrand`).
	pub brand: Type,
	/// The `Kind` signature definition.
	pub kind_input: KindInput,
	/// The associated type name to project (e.g., `Of`).
	pub assoc_name: Ident,
	/// The generic arguments for the projection (e.g., `<T, U>`).
	pub args: AngleBracketedGenericArguments,
}

impl Parse for ApplyInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		// Parse `<`
		input.parse::<Token![<]>()?;

		// Parse Brand
		let brand: Type = input.parse()?;

		// Parse `as`
		input.parse::<Token![as]>()?;

		// Parse `Kind` identifier
		let kind_ident: Ident = input.parse()?;
		if kind_ident != "Kind" {
			return Err(syn::Error::new(kind_ident.span(), "expected `Kind`"));
		}

		// Parse `!`
		input.parse::<Token![!]>()?;

		// Parse `(...)` containing KindInput
		let content;
		syn::parenthesized!(content in input);
		let kind_input: KindInput = content.parse()?;

		// Parse `>`
		input.parse::<Token![>]>()?;

		// Parse `::`
		input.parse::<Token![::]>()?;

		// Parse Assoc Name
		let assoc_name: Ident = input.parse()?;

		// Parse `<...>` Args
		let args: AngleBracketedGenericArguments = input.parse()?;

		Ok(ApplyInput { brand, kind_input, assoc_name, args })
	}
}

/// Generates the implementation for the `Apply!` macro.
pub fn apply_impl(input: ApplyInput) -> TokenStream {
	let brand = &input.brand;
	let kind_name = generate_name(&input.kind_input);
	let assoc_name = &input.assoc_name;
	let args = &input.args;

	quote! {
		<#brand as #kind_name>::#assoc_name #args
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use syn::parse_str;

	#[test]
	fn test_parse_apply_new_syntax() {
		let input = "<OptionBrand as Kind!(type Of<'a, T>: 'a;)>::Of<'static, i32>";
		let parsed: ApplyInput = parse_str(input).expect("Failed to parse ApplyInput");

		assert_eq!(parsed.assoc_name.to_string(), "Of");
		assert_eq!(parsed.kind_input.assoc_types.len(), 1);
		assert_eq!(parsed.args.args.len(), 2);
	}

	#[test]
	fn test_apply_generation_new_syntax() {
		let input = "<OptionBrand as Kind!(type Of<'a, T>: 'a;)>::Of<'static, i32>";
		let parsed: ApplyInput = parse_str(input).expect("Failed to parse ApplyInput");

		let output = apply_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("< OptionBrand as Kind_"));
		assert!(output_str.contains(":: Of < 'static , i32 >"));
	}
}
