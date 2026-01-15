//! Procedural macros for the `fp-library` crate.
//!
//! This crate provides macros for generating and working with Higher-Kinded Type (HKT) traits.
//! It includes:
//! - `Kind!`: Generates the name of a Kind trait based on its signature.
//! - `def_kind!`: Defines a new Kind trait.

use generate::generate_name;
use parse::KindInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub(crate) mod canonicalize;
pub(crate) mod generate;
pub(crate) mod parse;

#[cfg(test)]
mod tests;

#[proc_macro]
#[allow(non_snake_case)]
pub fn Kind(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as KindInput);
	let name = generate_name(&input);
	quote!(#name).into()
}

#[proc_macro]
pub fn def_kind(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as KindInput);
	let name = generate_name(&input);

	let lifetimes = &input.lifetimes;

	let types_with_bounds = input.types.iter().map(|t| {
		let ident = &t.ident;
		let bounds = &t.bounds;
		if bounds.is_empty() {
			quote! { #ident }
		} else {
			quote! { #ident: #bounds }
		}
	});

	let output_bounds = &input.output_bounds;
	let output_bounds_tokens =
		if output_bounds.is_empty() { quote!() } else { quote!(: #output_bounds) };

	let doc_string = format!("Auto-generated Kind trait: {}", name);

	let generics_inner = if input.lifetimes.is_empty() {
		quote! { #(#types_with_bounds),* }
	} else if input.types.is_empty() {
		quote! { #lifetimes }
	} else {
		quote! { #lifetimes, #(#types_with_bounds),* }
	};

	let expanded = quote! {
		#[doc = #doc_string]
		#[allow(non_camel_case_types)]
		pub trait #name {
			type Of < #generics_inner > #output_bounds_tokens;
		}
	};

	expanded.into()
}
