//! Code generation for the [`handlers!`](crate::handlers) macro.
//!
//! Accepts a comma-separated list of `Brand: expression` entries and
//! emits a right-nested
//! [`HandlersCons`](https://docs.rs/fp-library/latest/fp_library/types/effects/handlers/struct.HandlersCons.html)
//! chain terminated in
//! [`HandlersNil`](https://docs.rs/fp-library/latest/fp_library/types/effects/handlers/struct.HandlersNil.html),
//! with each `expression` wrapped in
//! [`Handler::<Brand, _>::new(...)`](https://docs.rs/fp-library/latest/fp_library/types/effects/handlers/struct.Handler.html).
//! Entries are sorted lexically by the stringified brand type so the
//! emitted list aligns cell-for-cell with the row produced by
//! [`effects!`](crate::effects), which uses the same lexical sort
//! (shared via [`crate::effects::row_sort`]).
//!
//! Empty input emits just `HandlersNil`.
//!
//! Per [decisions.md](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/decisions.md)
//! section 4.6, this macro is the primary surface for assembling a
//! natural transformation `VariantF<R> ~> M`. The non-macro fallback
//! is `nt().on::<E, _>(handler)` (a chained-builder over the same
//! runtime types); both paths produce values consumable by the Phase 3
//! step 2 interpreter.

use {
	proc_macro2::TokenStream,
	quote::quote,
	syn::{
		Expr,
		Token,
		Type,
		parse::{
			Parse,
			ParseStream,
			Parser,
		},
		punctuated::Punctuated,
	},
};

/// One `Brand: expression` entry inside `handlers!{ ... }`.
///
/// The brand is parsed as a [`syn::Type`] so generic parameters
/// (`Reader<Env>`, `State<i32>`) round-trip through
/// [`quote!`](quote::quote)'s stringification, matching the row brand
/// the row-side macro emits. The expression is parsed permissively as
/// any [`syn::Expr`] so closure literals, function items, and
/// already-constructed handler values all work.
struct HandlerEntry {
	brand: Type,
	_colon: Token![:],
	expr: Expr,
}

impl Parse for HandlerEntry {
	fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
		Ok(HandlerEntry {
			brand: input.parse()?,
			_colon: input.parse()?,
			expr: input.parse()?,
		})
	}
}

/// Worker for the [`handlers!`](crate::handlers) macro.
///
/// Parses `Brand1: expr1, Brand2: expr2, ...`, sorts entries lexically
/// by `quote!(brand).to_string()` (matching the
/// [`effects!`](crate::effects) row order), and emits the cons chain.
pub fn handlers_worker(input: TokenStream) -> syn::Result<TokenStream> {
	let parser = Punctuated::<HandlerEntry, Token![,]>::parse_terminated;
	let parsed = parser.parse2(input)?;
	let mut entries: Vec<(String, HandlerEntry)> = parsed
		.into_iter()
		.map(|e| {
			let key = {
				let b = &e.brand;
				quote!(#b).to_string()
			};
			(key, e)
		})
		.collect();
	entries.sort_by(|a, b| a.0.cmp(&b.0));

	let mut acc: TokenStream = quote! {
		::fp_library::types::effects::handlers::HandlersNil
	};
	for (_, entry) in entries.into_iter().rev() {
		let brand = &entry.brand;
		let expr = &entry.expr;
		acc = quote! {
			::fp_library::types::effects::handlers::HandlersCons {
				head: ::fp_library::types::effects::handlers::Handler::<#brand, _>::new(#expr),
				tail: #acc,
			}
		};
	}
	Ok(acc)
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "Tests use panicking operations for brevity and clarity")]
mod tests {
	use super::*;

	#[test]
	fn empty_input_yields_handlers_nil() {
		let out = handlers_worker(quote! {}).expect("worker failed").to_string();
		assert_eq!(out, ":: fp_library :: types :: effects :: handlers :: HandlersNil");
	}

	#[test]
	fn single_entry_wraps_in_handler_and_cons() {
		let out =
			handlers_worker(quote! { StateBrand: |op| op }).expect("worker failed").to_string();
		assert!(out.contains("HandlersCons"));
		assert!(out.contains("Handler"));
		assert!(out.contains("StateBrand"));
		assert!(out.contains("HandlersNil"));
	}

	#[test]
	fn two_entries_canonical_order_independent_of_input() {
		let a = handlers_worker(quote! {
			ReaderBrand: |op| op,
			StateBrand: |op| op
		})
		.expect("worker failed")
		.to_string();
		let b = handlers_worker(quote! {
			StateBrand: |op| op,
			ReaderBrand: |op| op
		})
		.expect("worker failed")
		.to_string();
		assert_eq!(a, b);
	}

	#[test]
	fn entries_sorted_lexically_head_is_smallest() {
		// Lexical sort puts ReaderBrand (R) before StateBrand (S), so
		// the emitted list's head should be the ReaderBrand handler.
		let out = handlers_worker(quote! {
			StateBrand: |op| op,
			ReaderBrand: |op| op
		})
		.expect("worker failed")
		.to_string();
		let reader_pos = out.find("ReaderBrand").expect("ReaderBrand missing");
		let state_pos = out.find("StateBrand").expect("StateBrand missing");
		assert!(
			reader_pos < state_pos,
			"expected ReaderBrand before StateBrand in canonical order, got {out}"
		);
	}

	#[test]
	fn brand_with_generic_params_parses() {
		let out = handlers_worker(quote! {
			ReaderBrand<Env>: |op| op
		})
		.expect("worker failed")
		.to_string();
		assert!(out.contains("ReaderBrand"));
		assert!(out.contains("Env"));
	}

	#[test]
	fn trailing_comma_accepted() {
		let out = handlers_worker(quote! {
			StateBrand: |op| op,
		})
		.expect("worker failed")
		.to_string();
		assert!(out.contains("StateBrand"));
		assert!(out.contains("HandlersNil"));
	}
}
