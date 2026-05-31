//! Item generators owned by `#[document_module]`.
//!
//! These are not exported function-like procedural macros. They are
//! item-position markers that `#[document_module]` removes and replaces before
//! documentation validation runs.

use {
	crate::{
		core::constants::macros::{
			DEFINE_EFFECT,
			DOCUMENTED_HELPER_IMPLS,
		},
		support::parsing::{
			parse_many,
			parse_non_empty,
		},
	},
	syn::{
		Ident,
		Item,
		ItemImpl,
		ItemMacro,
		Token,
		parse::{
			Parse,
			ParseStream,
		},
		spanned::Spanned,
	},
};

mod keyword {
	syn::custom_keyword!(effect);
}

struct DocumentedHelperImplsInput {
	impls: Vec<ItemImpl>,
}

struct DefineEffectInput {
	effect_name: Ident,
}

struct GeneratedItems {
	items: Vec<Item>,
}

impl Parse for DocumentedHelperImplsInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let impls = parse_many(input)?;
		let impls = parse_non_empty(
			impls,
			&format!("{DOCUMENTED_HELPER_IMPLS}! requires at least one impl block"),
		)?;
		Ok(Self {
			impls,
		})
	}
}

impl Parse for DefineEffectInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		input.parse::<keyword::effect>()?;
		let effect_name = input.parse()?;
		input.parse::<Token![;]>()?;

		if !input.is_empty() {
			return Err(
				input.error(format!("{DEFINE_EFFECT}! does not accept additional fields yet"))
			);
		}

		Ok(Self {
			effect_name,
		})
	}
}

impl Parse for GeneratedItems {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(Self {
			items: parse_many(input)?,
		})
	}
}

fn is_documented_helper_impls(item_macro: &ItemMacro) -> bool {
	item_macro.mac.path.is_ident(DOCUMENTED_HELPER_IMPLS)
}

fn is_define_effect(item_macro: &ItemMacro) -> bool {
	item_macro.mac.path.is_ident(DEFINE_EFFECT)
}

fn expand_documented_helper_impls(item_macro: ItemMacro) -> syn::Result<Vec<Item>> {
	let span = item_macro.span();
	let input =
		syn::parse2::<DocumentedHelperImplsInput>(item_macro.mac.tokens).map_err(|error| {
			syn::Error::new(
				span,
				format!("{DOCUMENTED_HELPER_IMPLS}! only accepts Rust impl blocks: {error}"),
			)
		})?;

	Ok(input.impls.into_iter().map(Item::Impl).collect())
}

fn expand_define_effect(item_macro: ItemMacro) -> syn::Result<Vec<Item>> {
	let span = item_macro.span();
	let input = syn::parse2::<DefineEffectInput>(item_macro.mac.tokens).map_err(|error| {
		syn::Error::new(span, format!("{DEFINE_EFFECT}! expected `effect Reader;`: {error}"))
	})?;

	match input.effect_name.to_string().as_str() {
		"Reader" => expand_reader_effect_items(),
		_ => Err(syn::Error::new(
			input.effect_name.span(),
			format!("{DEFINE_EFFECT}! currently only supports `effect Reader;`"),
		)),
	}
}

fn parse_generated_items(source: &str) -> syn::Result<Vec<Item>> {
	Ok(syn::parse_str::<GeneratedItems>(source)?.items)
}

fn expand_reader_effect_items() -> syn::Result<Vec<Item>> {
	parse_generated_items(include_str!("reader_effect_items.rs"))
}

/// Expands item-position helper generators before documentation validation.
pub(super) fn expand_item_generators(items: &mut Vec<Item>) -> syn::Result<()> {
	let original_items = core::mem::take(items);
	let mut expanded_items = Vec::with_capacity(original_items.len());

	for mut item in original_items {
		match item {
			Item::Macro(item_macro) if is_define_effect(&item_macro) => {
				expanded_items.extend(expand_define_effect(item_macro)?);
			}
			Item::Macro(item_macro) if is_documented_helper_impls(&item_macro) => {
				expanded_items.extend(expand_documented_helper_impls(item_macro)?);
			}
			Item::Mod(ref mut module) => {
				if let Some((_, ref mut nested_items)) = module.content {
					expand_item_generators(nested_items)?;
				}
				expanded_items.push(item);
			}
			_ => expanded_items.push(item),
		}
	}

	*items = expanded_items;
	Ok(())
}
