//! Shared lexical-sort helper for first-order and scoped effect rows.
//!
//! Both [`effects!`](crate::effects) (Phase 2 step 8) and
//! `scoped_effects!` (Phase 4 step 4) accept a comma-separated list of
//! types and emit a right-nested brand-level row in canonical order.
//! The "canonical order" is the lexical sort of `quote!(#t).to_string()`
//! for each input type. Whitespace inside the stringified form is
//! normalised by `quote`, so the same type written different ways
//! (e.g., `Reader<Env>` vs `Reader < Env >`) yields the same string and
//! sorts to the same position.
//!
//! Factoring the sort here means future canonicalisation refinements
//! (e.g., handling of fully-generic effect type parameters, surfaced as
//! an open question in
//! [`decisions.md`](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/decisions.md)
//! section 4.1's POC validation paragraph) land in one place rather
//! than being duplicated across `effects!` and `scoped_effects!`.

use {
	proc_macro2::TokenStream,
	quote::quote,
	syn::{
		Token,
		Type,
		parse::Parser,
		punctuated::Punctuated,
	},
};

/// Parses a comma-separated list of types from `input` and returns
/// them sorted by `quote!(#t).to_string()`.
///
/// An empty input produces an empty `Vec` (the caller decides what
/// to emit for the zero-length case).
pub(crate) fn parse_and_sort_types(input: TokenStream) -> syn::Result<Vec<Type>> {
	let parser = Punctuated::<Type, Token![,]>::parse_terminated;
	let parsed = parser.parse2(input)?;
	let mut typed: Vec<(String, Type)> =
		parsed.into_iter().map(|t| (quote!(#t).to_string(), t)).collect();
	typed.sort_by(|a, b| a.0.cmp(&b.0));
	Ok(typed.into_iter().map(|(_, t)| t).collect())
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "Tests use panicking operations for brevity and clarity")]
mod tests {
	use super::*;

	#[test]
	fn sorts_by_stringified_form() {
		let input: TokenStream = quote! { OptionBrand, IdentityBrand };
		let sorted = parse_and_sort_types(input).expect("parse failed");
		assert_eq!(quote!(#(#sorted),*).to_string(), "IdentityBrand , OptionBrand");
	}

	#[test]
	fn empty_input_yields_empty_vec() {
		let input: TokenStream = quote! {};
		let sorted = parse_and_sort_types(input).expect("parse failed");
		assert!(sorted.is_empty());
	}

	#[test]
	fn whitespace_normalised_by_quote() {
		// Whitespace inside the input is normalised by quote's stringification,
		// so equivalent types sort to the same position regardless of source spacing.
		let a: TokenStream = quote! { Reader<Env> };
		let b: TokenStream = quote! { Reader < Env > };
		let sorted_a = parse_and_sort_types(a).expect("parse failed");
		let sorted_b = parse_and_sort_types(b).expect("parse failed");
		assert_eq!(quote!(#(#sorted_a),*).to_string(), quote!(#(#sorted_b),*).to_string());
	}

	#[test]
	fn generic_effect_types_sort_by_full_string() {
		// Generic parameters are part of the stringified form, so different
		// instantiations sort into different positions (usually correct, but
		// users should be aware).
		let input: TokenStream = quote! { Reader<Env>, Reader<Other> };
		let sorted = parse_and_sort_types(input).expect("parse failed");
		assert_eq!(quote!(#(#sorted),*).to_string(), "Reader < Env > , Reader < Other >");
	}
}
