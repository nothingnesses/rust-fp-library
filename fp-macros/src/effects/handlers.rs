//! Code generation for the [`handlers!`](crate::handlers) and
//! [`scoped_handlers!`](crate::scoped_handlers) macros.
//!
//! Both macros accept a comma-separated list of `Brand: expression`
//! entries. The first-order macro emits a right-nested
//! [`HandlersCons`](https://docs.rs/fp-library/latest/fp_library/types/effects/handlers/struct.HandlersCons.html)
//! chain terminated in
//! [`HandlersNil`](https://docs.rs/fp-library/latest/fp_library/types/effects/handlers/struct.HandlersNil.html),
//! with each `expression` wrapped in
//! [`Handler::<Brand, _>::new(...)`](https://docs.rs/fp-library/latest/fp_library/types/effects/handlers/struct.Handler.html).
//! The scoped macro emits the parallel
//! [`ScopedHandlersCons`](https://docs.rs/fp-library/latest/fp_library/types/effects/handlers/struct.ScopedHandlersCons.html)
//! / [`ScopedHandlersNil`](https://docs.rs/fp-library/latest/fp_library/types/effects/handlers/struct.ScopedHandlersNil.html)
//! carrier with each expression wrapped in
//! [`ScopedHandler::<Brand, _>::new(...)`](https://docs.rs/fp-library/latest/fp_library/types/effects/handlers/struct.ScopedHandler.html).
//! Entries are sorted by the shared structural brand-type key so the
//! emitted list aligns cell-for-cell with the row produced by
//! [`effects!`](crate::effects) or [`scoped_effects!`](crate::scoped_effects),
//! which use the same sort helper (shared via
//! [`crate::effects::row_sort`]).
//! The key is structural over the parsed type syntax the proc macro can
//! observe. It normalizes whitespace and grouping, but it does not
//! resolve Rust aliases or imports. Use the same brand spelling in the
//! row and handler macros; duplicate handler entries with the same
//! structural key are rejected during macro expansion.
//!
//! Empty input emits just `HandlersNil`.
//!
//! This macro is the primary surface for assembling a natural
//! transformation from a first-order effect row to a target program
//! type. The non-macro fallback for user-written manual code is
//! `handlers_ordered().on::<E, _>(handler).finish()`; the lower-level
//! representation seed is `nt().prepend::<E, _>(handler)`.

use {
	crate::effects::row_sort::sort_type_keyed,
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
/// (`Reader<Env>`, `State<i32>`) feed the same structural key used by
/// the row-side macros. The expression is parsed permissively as any
/// [`syn::Expr`] so closure literals, function items, and
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
/// Parses `Brand1: expr1, Brand2: expr2, ...`, sorts entries by the
/// shared structural row key (matching the
/// [`effects!`](crate::effects) row order), and emits the cons chain.
pub fn handlers_worker(input: TokenStream) -> syn::Result<TokenStream> {
	handler_list_worker(
		input,
		quote! { ::fp_library::types::effects::handlers::HandlersNil },
		quote! { ::fp_library::types::effects::handlers::HandlersCons },
		quote! { ::fp_library::types::effects::handlers::Handler },
	)
}

/// Worker for the [`scoped_handlers!`](crate::scoped_handlers) macro.
///
/// Parses `Brand1: expr1, Brand2: expr2, ...`, sorts entries by the
/// shared structural row key (matching the
/// [`scoped_effects!`](crate::scoped_effects) row order), and emits the
/// scoped handler cons chain.
pub fn scoped_handlers_worker(input: TokenStream) -> syn::Result<TokenStream> {
	handler_list_worker(
		input,
		quote! { ::fp_library::types::effects::handlers::ScopedHandlersNil },
		quote! { ::fp_library::types::effects::handlers::ScopedHandlersCons },
		quote! { ::fp_library::types::effects::handlers::ScopedHandler },
	)
}

fn handler_list_worker(
	input: TokenStream,
	nil_path: TokenStream,
	cons_path: TokenStream,
	handler_path: TokenStream,
) -> syn::Result<TokenStream> {
	let parser = Punctuated::<HandlerEntry, Token![,]>::parse_terminated;
	let parsed = parser.parse2(input)?;
	let entries = sort_type_keyed(
		parsed.into_iter().map(|entry| (entry.brand, entry.expr)),
		"handler entry",
	)?;

	let mut acc: TokenStream = nil_path;
	for (brand, expr) in entries.into_iter().rev() {
		acc = quote! {
			#cons_path {
				head: #handler_path::<#brand, _>::new(#expr),
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
	fn entries_sorted_by_structural_key_head_is_smallest() {
		// The shared structural key orders ReaderBrand before StateBrand,
		// so the emitted list's head should be the ReaderBrand handler.
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

	#[test]
	fn duplicate_handler_entries_are_rejected() {
		let err = handlers_worker(quote! {
			ReaderBrand<Env>: |op| op,
			(ReaderBrand<Env>): |op| op,
		})
		.expect_err("duplicate handler entry should fail");
		assert!(err.to_string().contains("duplicate handler entry"));
	}

	#[test]
	fn scoped_empty_input_yields_scoped_handlers_nil() {
		let out = scoped_handlers_worker(quote! {}).expect("worker failed").to_string();
		assert_eq!(out, ":: fp_library :: types :: effects :: handlers :: ScopedHandlersNil");
	}

	#[test]
	fn scoped_single_entry_wraps_in_scoped_handler_and_cons() {
		let out = scoped_handlers_worker(quote! { SpanBrand: Dispatcher })
			.expect("worker failed")
			.to_string();
		assert!(out.contains("ScopedHandlersCons"));
		assert!(out.contains("ScopedHandler"));
		assert!(out.contains("SpanBrand"));
		assert!(out.contains("ScopedHandlersNil"));
	}

	#[test]
	fn duplicate_scoped_handler_entries_are_rejected() {
		let err = scoped_handlers_worker(quote! {
			SpanBrand<Tag>: handler_a,
			(SpanBrand<Tag>): handler_b,
		})
		.expect_err("duplicate scoped handler entry should fail");
		assert!(err.to_string().contains("duplicate handler entry"));
	}

	#[test]
	fn scoped_entries_canonical_order_independent_of_input() {
		let a = scoped_handlers_worker(quote! {
			SpanBrand: span_handler,
			CatchBrand: catch_handler
		})
		.expect("worker failed")
		.to_string();
		let b = scoped_handlers_worker(quote! {
			CatchBrand: catch_handler,
			SpanBrand: span_handler
		})
		.expect("worker failed")
		.to_string();
		assert_eq!(a, b);
	}
}
