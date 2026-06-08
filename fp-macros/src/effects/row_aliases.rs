//! Code generation for the [`define_effect_row_aliases!`](crate::define_effect_row_aliases)
//! item macro.
//!
//! The macro emits ordinary type aliases for named effect rows. It is
//! intentionally narrow: callers still choose the row aliases, handler
//! constructors, and program aliases explicitly, but avoid hand-writing
//! nested `CoproductBrand` shapes for the common default, Rc, Arc, and
//! scoped row cases.
//!
//! Row entries use the same structural key as `effects!`,
//! `scoped_effects!`, `handlers!`, and `scoped_handlers!`. The key
//! normalizes syntax visible in the parsed type tree, but it does not
//! resolve aliases, imports, or semantic Rust type identity. Duplicate
//! entries with the same structural key are rejected per alias.

use {
	crate::effects::{
		effects_macro::{
			RowHeadWrap,
			build_coproduct_row,
		},
		row_sort::sort_types_unique,
	},
	proc_macro2::TokenStream,
	quote::quote,
	syn::{
		Ident,
		Token,
		Type,
		Visibility,
		bracketed,
		parse::{
			Parse,
			ParseStream,
		},
		punctuated::Punctuated,
	},
};

struct DefineEffectRowAliasesInput {
	aliases: Vec<RowAlias>,
}

struct RowAlias {
	vis: Visibility,
	ident: Ident,
	kind: RowAliasKind,
	types: Vec<Type>,
}

enum RowAliasKind {
	FirstOrder,
	RcFirstOrder,
	ArcFirstOrder,
	Scoped,
}

impl Parse for DefineEffectRowAliasesInput {
	fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
		let mut aliases = Vec::new();
		while !input.is_empty() {
			aliases.push(input.parse()?);
		}
		Ok(Self {
			aliases,
		})
	}
}

impl Parse for RowAlias {
	fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
		let vis: Visibility = input.parse()?;
		input.parse::<Token![type]>()?;
		let ident: Ident = input.parse()?;
		input.parse::<Token![=]>()?;
		let kind: RowAliasKind = input.parse()?;

		let content;
		bracketed!(content in input);
		let types =
			Punctuated::<Type, Token![,]>::parse_terminated(&content)?.into_iter().collect();
		input.parse::<Token![;]>()?;

		Ok(Self {
			vis,
			ident,
			kind,
			types,
		})
	}
}

impl Parse for RowAliasKind {
	fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
		let ident: Ident = input.parse()?;
		if ident == "first_order" {
			Ok(RowAliasKind::FirstOrder)
		} else if ident == "rc_first_order" {
			Ok(RowAliasKind::RcFirstOrder)
		} else if ident == "arc_first_order" {
			Ok(RowAliasKind::ArcFirstOrder)
		} else if ident == "scoped" {
			Ok(RowAliasKind::Scoped)
		} else {
			Err(syn::Error::new(
				ident.span(),
				"expected `first_order`, `rc_first_order`, `arc_first_order`, or `scoped`",
			))
		}
	}
}

/// Worker for the public `define_effect_row_aliases!` macro.
pub fn define_effect_row_aliases_worker(input: TokenStream) -> syn::Result<TokenStream> {
	let DefineEffectRowAliasesInput {
		aliases,
	} = syn::parse2(input)?;
	let definitions = aliases.into_iter().map(|alias| {
		let RowAlias {
			vis,
			ident,
			kind,
			types,
		} = alias;
		let wrap = match kind {
			RowAliasKind::FirstOrder => RowHeadWrap::Coyoneda,
			RowAliasKind::RcFirstOrder => RowHeadWrap::RcCoyoneda,
			RowAliasKind::ArcFirstOrder => RowHeadWrap::ArcCoyoneda,
			RowAliasKind::Scoped => RowHeadWrap::None,
		};
		let row = build_coproduct_row(sort_types_unique(types)?, wrap);
		Ok(quote! {
			#vis type #ident = #row;
		})
	});
	let definitions = definitions.collect::<syn::Result<Vec<_>>>()?;

	Ok(quote! {
		#(#definitions)*
	})
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "Tests use panicking operations for brevity and clarity")]
mod tests {
	use super::*;

	#[test]
	fn emits_first_order_and_scoped_aliases() {
		let out = define_effect_row_aliases_worker(quote! {
			pub type FirstRow = first_order [StateBrand, ReaderBrand];
			type ScopedRow = scoped [SpanBrand, CatchBrand];
		})
		.expect("worker failed")
		.to_string();

		assert!(out.contains("pub type FirstRow"));
		assert!(out.contains("CoyonedaBrand < ReaderBrand >"));
		assert!(out.contains("CoyonedaBrand < StateBrand >"));
		assert!(out.contains("type ScopedRow"));
		assert!(out.contains("CatchBrand"));
		assert!(out.contains("SpanBrand"));
	}

	#[test]
	fn emits_shared_wrapper_coyoneda_variants() {
		let out = define_effect_row_aliases_worker(quote! {
			type RcRow = rc_first_order [ReaderBrand<RcBrand, i32>];
			type ArcRow = arc_first_order [SendReaderBrand<ArcBrand, i32>];
		})
		.expect("worker failed")
		.to_string();

		assert!(out.contains("RcCoyonedaBrand < ReaderBrand < RcBrand , i32 > >"));
		assert!(out.contains("ArcCoyonedaBrand < SendReaderBrand < ArcBrand , i32 > >"));
	}

	#[test]
	fn canonical_order_independent_of_input() {
		let a = define_effect_row_aliases_worker(quote! {
			type Row = first_order [StateBrand, ReaderBrand];
		})
		.expect("worker failed")
		.to_string();
		let b = define_effect_row_aliases_worker(quote! {
			type Row = first_order [ReaderBrand, StateBrand];
		})
		.expect("worker failed")
		.to_string();

		assert_eq!(a, b);
	}

	#[test]
	fn empty_rows_emit_cnil() {
		let out = define_effect_row_aliases_worker(quote! {
			type EmptyFirstRow = first_order [];
			type EmptyScopedRow = scoped [];
		})
		.expect("worker failed")
		.to_string();

		assert!(out.contains("type EmptyFirstRow = :: fp_library :: brands :: CNilBrand"));
		assert!(out.contains("type EmptyScopedRow = :: fp_library :: brands :: CNilBrand"));
	}

	#[test]
	fn duplicate_alias_row_entries_are_rejected() {
		let err = define_effect_row_aliases_worker(quote! {
			type Duplicates = scoped [SpanBrand<Tag>, (SpanBrand<Tag>)];
		})
		.expect_err("duplicate row entry should fail");

		assert!(err.to_string().contains("duplicate row entry"));
	}

	#[test]
	fn rejects_unknown_row_kind() {
		let err = define_effect_row_aliases_worker(quote! {
			type Row = raw [IdentityBrand];
		})
		.expect_err("unknown row kind should fail");

		assert!(
			err.to_string().contains(
				"expected `first_order`, `rc_first_order`, `arc_first_order`, or `scoped`"
			)
		);
	}
}
