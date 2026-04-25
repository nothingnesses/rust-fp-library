//! POC proc-macro for the effect-row canonicalisation hybrid.
//!
//! `effects![A, B, C]` parses a comma-separated list of types,
//! lexically sorts them by their stringified representation (output
//! of `quote!{}.to_string()`), and emits a right-nested
//! `frunk_core::coproduct::Coproduct<...>` in canonical order.
//!
//! This is workaround 1 from port-plan section 4.1 ordering
//! mitigations, taken to its logical extreme: the macro always
//! produces the same canonical type regardless of input order.
//!
//! Empty input produces `CNil`.

use {
	proc_macro::TokenStream,
	proc_macro2::TokenStream as TokenStream2,
	quote::quote,
	syn::{
		Token,
		Type,
		parse::Parser,
		punctuated::Punctuated,
	},
};

#[proc_macro]
pub fn effects(input: TokenStream) -> TokenStream {
	let parser = Punctuated::<Type, Token![,]>::parse_terminated;
	let parsed = match parser.parse(input) {
		Ok(p) => p,
		Err(e) => return e.to_compile_error().into(),
	};

	// Sort by stringified type representation. Whitespace inside
	// `quote!{}.to_string()` is normalised by quote, so the same
	// type written different ways (e.g., `Reader<Env>` vs
	// `Reader < Env >`) yields the same string.
	let mut typed: Vec<(String, Type)> =
		parsed.into_iter().map(|t| (quote!(#t).to_string(), t)).collect();
	typed.sort_by(|a, b| a.0.cmp(&b.0));

	let mut acc: TokenStream2 = quote! { ::frunk_core::coproduct::CNil };
	for (_, ty) in typed.into_iter().rev() {
		acc = quote! { ::frunk_core::coproduct::Coproduct<#ty, #acc> };
	}
	acc.into()
}
