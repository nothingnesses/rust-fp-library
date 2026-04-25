//! POC proc-macro for the effect-row canonicalisation hybrid.
//!
//! `effects![A, B, C]` parses a comma-separated list of types,
//! lexically sorts them by their stringified representation (output
//! of `quote!{}.to_string()`), and emits a right-nested
//! `frunk_core::coproduct::Coproduct<...>` in canonical order.
//!
//! This is workaround 1 from decisions section 4.1 ordering
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

/// `effects_coyo![A, B, C]` is the same lexical sort as `effects!`,
/// but each emitted variant is wrapped in `Coyoneda<T, A>` from the
/// POC's stub Coyoneda. The user supplies an `A` type as the first
/// argument: `effects_coyo![A; Eff1, Eff2]` emits
/// `Coproduct<Coyoneda<Eff1, A>, Coproduct<Coyoneda<Eff2, A>, CNil>>`
/// in lexical order. The point is to exercise the macro / Coyoneda
/// integration story for decisions section 4.2's static option.
///
/// Sort order is determined by the inner effect type's stringified
/// form, NOT the wrapped form. The inner-vs-wrapped distinction does
/// not matter for our test types (Coyoneda<X, A> sorts identically by
/// inner-X-name and by wrapped-form, because the prefix "Coyoneda <"
/// is constant), but anchoring on the inner type makes the behaviour
/// explicit and robust against future wrapper changes.
#[proc_macro]
pub fn effects_coyo(input: TokenStream) -> TokenStream {
	// Custom parser: expect `A; T1, T2, ...` where A is the answer
	// type the Coyoneda is parameterised by, and the Ts are effects.
	let raw: TokenStream2 = input.into();
	let mut iter = raw.into_iter();
	let mut answer_tokens: Vec<proc_macro2::TokenTree> = Vec::new();
	let mut effects_tokens: Vec<proc_macro2::TokenTree> = Vec::new();
	let mut found_separator = false;
	for tok in iter.by_ref() {
		if let proc_macro2::TokenTree::Punct(p) = &tok
			&& p.as_char() == ';'
		{
			found_separator = true;
			break;
		}
		answer_tokens.push(tok);
	}
	if !found_separator {
		return syn::Error::new(
			proc_macro2::Span::call_site(),
			"expected `A; Eff1, Eff2, ...`; missing `;`",
		)
		.to_compile_error()
		.into();
	}
	for tok in iter {
		effects_tokens.push(tok);
	}
	let answer_stream: TokenStream2 = answer_tokens.into_iter().collect();
	let effects_stream: TokenStream2 = effects_tokens.into_iter().collect();

	let answer: Type = match syn::parse2(answer_stream) {
		Ok(t) => t,
		Err(e) => return e.to_compile_error().into(),
	};
	let parser = Punctuated::<Type, Token![,]>::parse_terminated;
	let parsed = match parser.parse2(effects_stream) {
		Ok(p) => p,
		Err(e) => return e.to_compile_error().into(),
	};

	// Sort by inner-effect type stringified form.
	let mut typed: Vec<(String, Type)> =
		parsed.into_iter().map(|t| (quote!(#t).to_string(), t)).collect();
	typed.sort_by(|a, b| a.0.cmp(&b.0));

	let mut acc: TokenStream2 = quote! { ::frunk_core::coproduct::CNil };
	for (_, ty) in typed.into_iter().rev() {
		acc = quote! {
			::frunk_core::coproduct::Coproduct<
				::poc_effect_row::coyoneda::Coyoneda<#ty, #answer>,
				#acc
			>
		};
	}
	acc.into()
}
