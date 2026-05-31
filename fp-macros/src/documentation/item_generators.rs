//! Item generators owned by `#[document_module]`.
//!
//! These are not exported function-like procedural macros. They are
//! item-position markers that `#[document_module]` removes and replaces before
//! documentation validation runs.

use {
	crate::{
		core::constants::macros::{
			DEFINE_EFFECT,
			DEFINE_RUN_WRAPPER,
			DOCUMENTED_HELPER_IMPLS,
		},
		support::parsing::{
			parse_many,
			parse_non_empty,
		},
	},
	syn::{
		Ident,
		ImplItem,
		ImplItemMacro,
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
	syn::custom_keyword!(method);
	syn::custom_keyword!(wrapper);
}

struct DocumentedHelperImplsInput {
	impls: Vec<ItemImpl>,
}

struct DefineEffectInput {
	effect_name: Ident,
}

struct DefineRunWrapperInput {
	wrapper_name: Ident,
	effect_name: Ident,
	method_name: Ident,
}

struct GeneratedItems {
	items: Vec<Item>,
}

struct GeneratedImplItems {
	items: Vec<ImplItem>,
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

impl Parse for DefineRunWrapperInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		input.parse::<keyword::wrapper>()?;
		let wrapper_name = input.parse()?;
		input.parse::<Token![;]>()?;

		input.parse::<keyword::effect>()?;
		let effect_name = input.parse()?;
		input.parse::<Token![;]>()?;

		input.parse::<keyword::method>()?;
		let method_name = input.parse()?;
		input.parse::<Token![;]>()?;

		if !input.is_empty() {
			return Err(
				input.error(format!("{DEFINE_RUN_WRAPPER}! does not accept additional fields yet"))
			);
		}

		Ok(Self {
			wrapper_name,
			effect_name,
			method_name,
		})
	}
}

impl Parse for GeneratedImplItems {
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

fn is_define_run_wrapper(item_macro: &ItemMacro) -> bool {
	item_macro.mac.path.is_ident(DEFINE_RUN_WRAPPER)
}

fn impl_item_is_define_run_wrapper(item_macro: &ImplItemMacro) -> bool {
	item_macro.mac.path.is_ident(DEFINE_RUN_WRAPPER)
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
		"State" => expand_state_effect_items(),
		_ => Err(syn::Error::new(
			input.effect_name.span(),
			format!(
				"{DEFINE_EFFECT}! currently only supports `effect Reader;` and `effect State;`"
			),
		)),
	}
}

fn parse_generated_items(source: &str) -> syn::Result<Vec<Item>> {
	Ok(syn::parse_str::<GeneratedItems>(source)?.items)
}

fn parse_generated_impl_items(source: &str) -> syn::Result<Vec<ImplItem>> {
	Ok(syn::parse_str::<GeneratedImplItems>(source)?.items)
}

fn expand_reader_effect_items() -> syn::Result<Vec<Item>> {
	parse_generated_items(include_str!("reader_effect_items.rs"))
}

fn expand_state_effect_items() -> syn::Result<Vec<Item>> {
	parse_generated_items(include_str!("state_effect_items.rs"))
}

fn expand_define_run_wrapper_impl_item(item_macro: ImplItemMacro) -> syn::Result<Vec<ImplItem>> {
	let span = item_macro.span();
	let input = syn::parse2::<DefineRunWrapperInput>(item_macro.mac.tokens).map_err(|error| {
		syn::Error::new(
			span,
			format!(
				"{DEFINE_RUN_WRAPPER}! expected `wrapper Run; effect Reader; method <name>;`: {error}"
			),
		)
	})?;

	let wrapper_name = input.wrapper_name.to_string();
	let effect_name = input.effect_name.to_string();
	let method_name = input.method_name.to_string();

	match (wrapper_name.as_str(), effect_name.as_str(), method_name.as_str()) {
		("Run", "Reader", "ask") =>
			parse_generated_impl_items(include_str!("run_reader_ask_impl_item.rs")),
		("Run", "Reader", "asks") =>
			parse_generated_impl_items(include_str!("run_reader_asks_impl_item.rs")),
		("Run", "Reader", "run_reader") =>
			parse_generated_impl_items(include_str!("run_reader_run_reader_impl_item.rs")),
		("RcRun", "Reader", "ask") =>
			parse_generated_impl_items(include_str!("rcrun_reader_ask_impl_item.rs")),
		("RcRun", "Reader", "asks") =>
			parse_generated_impl_items(include_str!("rcrun_reader_asks_impl_item.rs")),
		("RcRun", "Reader", "run_reader") =>
			parse_generated_impl_items(include_str!("rcrun_reader_run_reader_impl_item.rs")),
		("ArcRun", "Reader", "ask") =>
			parse_generated_impl_items(include_str!("arcrun_reader_ask_impl_item.rs")),
		("ArcRun", "Reader", "asks") =>
			parse_generated_impl_items(include_str!("arcrun_reader_asks_impl_item.rs")),
		("ArcRun", "Reader", "run_reader") =>
			parse_generated_impl_items(include_str!("arcrun_reader_run_reader_impl_item.rs")),
		("RunExplicit", "Reader", "ask") =>
			parse_generated_impl_items(include_str!("run_explicit_reader_ask_impl_item.rs")),
		("RunExplicit", "Reader", "asks") =>
			parse_generated_impl_items(include_str!("run_explicit_reader_asks_impl_item.rs")),
		("RunExplicit", "Reader", "run_reader") =>
			parse_generated_impl_items(include_str!("run_explicit_reader_run_reader_impl_item.rs")),
		("RcRunExplicit", "Reader", "ask") =>
			parse_generated_impl_items(include_str!("rcrun_explicit_reader_ask_impl_item.rs")),
		("RcRunExplicit", "Reader", "asks") =>
			parse_generated_impl_items(include_str!("rcrun_explicit_reader_asks_impl_item.rs")),
		("RcRunExplicit", "Reader", "run_reader") => parse_generated_impl_items(include_str!(
			"rcrun_explicit_reader_run_reader_impl_item.rs"
		)),
		("ArcRunExplicit", "Reader", "ask") =>
			parse_generated_impl_items(include_str!("arcrun_explicit_reader_ask_impl_item.rs")),
		("ArcRunExplicit", "Reader", "asks") =>
			parse_generated_impl_items(include_str!("arcrun_explicit_reader_asks_impl_item.rs")),
		("ArcRunExplicit", "Reader", "run_reader") => parse_generated_impl_items(include_str!(
			"arcrun_explicit_reader_run_reader_impl_item.rs"
		)),
		("Run", "State", "get") =>
			parse_generated_impl_items(include_str!("run_state_get_impl_item.rs")),
		("Run", "State", "put") =>
			parse_generated_impl_items(include_str!("run_state_put_impl_item.rs")),
		("Run", "State", "modify") =>
			parse_generated_impl_items(include_str!("run_state_modify_impl_item.rs")),
		("Run", "State", "run_state") =>
			parse_generated_impl_items(include_str!("run_state_run_state_impl_item.rs")),
		(
			"Run" | "RcRun" | "ArcRun" | "RunExplicit" | "RcRunExplicit" | "ArcRunExplicit",
			"Reader",
			_,
		) => Err(syn::Error::new(
			input.method_name.span(),
			format!(
				"{DEFINE_RUN_WRAPPER}! currently only supports Reader methods `ask`, `asks`, and `run_reader`"
			),
		)),
		(
			"Run" | "RcRun" | "ArcRun" | "RunExplicit" | "RcRunExplicit" | "ArcRunExplicit",
			"State",
			_,
		) => Err(syn::Error::new(
			input.method_name.span(),
			format!(
				"{DEFINE_RUN_WRAPPER}! currently only supports State methods `get`, `put`, `modify`, and `run_state` for `wrapper Run;`"
			),
		)),
		(_, "Reader" | "State", _) => Err(syn::Error::new(
			input.wrapper_name.span(),
			format!(
				"{DEFINE_RUN_WRAPPER}! currently only supports `wrapper Run;`, `wrapper RcRun;`, `wrapper ArcRun;`, `wrapper RunExplicit;`, `wrapper RcRunExplicit;`, and `wrapper ArcRunExplicit;`"
			),
		)),
		_ => Err(syn::Error::new(
			input.effect_name.span(),
			format!(
				"{DEFINE_RUN_WRAPPER}! currently only supports `effect Reader;` and `effect State;`"
			),
		)),
	}
}

fn expand_impl_item_generators(items: &mut Vec<ImplItem>) -> syn::Result<()> {
	let original_items = core::mem::take(items);
	let mut expanded_items = Vec::with_capacity(original_items.len());

	for item in original_items {
		match item {
			ImplItem::Macro(item_macro) if impl_item_is_define_run_wrapper(&item_macro) => {
				expanded_items.extend(expand_define_run_wrapper_impl_item(item_macro)?);
			}
			_ => expanded_items.push(item),
		}
	}

	*items = expanded_items;
	Ok(())
}

/// Expands item-position helper generators before documentation validation.
pub(super) fn expand_item_generators(items: &mut Vec<Item>) -> syn::Result<()> {
	let original_items = core::mem::take(items);
	let mut expanded_items = Vec::with_capacity(original_items.len());

	for mut item in original_items {
		match item {
			Item::Macro(item_macro) if is_define_run_wrapper(&item_macro) => {
				return Err(syn::Error::new(
					item_macro.span(),
					format!("{DEFINE_RUN_WRAPPER}! must be used inside an impl block"),
				));
			}
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
			Item::Impl(ref mut item_impl) => {
				expand_impl_item_generators(&mut item_impl.items)?;
				expanded_items.push(item);
			}
			_ => expanded_items.push(item),
		}
	}

	*items = expanded_items;
	Ok(())
}
