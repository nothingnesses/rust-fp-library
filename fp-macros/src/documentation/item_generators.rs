//! Item generators owned by `#[document_module]`.
//!
//! These are not exported function-like procedural macros. They are
//! item-position markers that `#[document_module]` removes and replaces before
//! documentation validation runs.

use {
	crate::{
		core::constants::macros::DOCUMENTED_HELPER_IMPLS,
		support::parsing::{
			parse_many,
			parse_non_empty,
		},
	},
	syn::{
		Item,
		ItemImpl,
		ItemMacro,
		parse::{
			Parse,
			ParseStream,
		},
		spanned::Spanned,
	},
};

struct DocumentedHelperImplsInput {
	impls: Vec<ItemImpl>,
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

fn is_documented_helper_impls(item_macro: &ItemMacro) -> bool {
	item_macro.mac.path.is_ident(DOCUMENTED_HELPER_IMPLS)
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

/// Expands item-position helper generators before documentation validation.
pub(super) fn expand_item_generators(items: &mut Vec<Item>) -> syn::Result<()> {
	let original_items = core::mem::take(items);
	let mut expanded_items = Vec::with_capacity(original_items.len());

	for mut item in original_items {
		match item {
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
