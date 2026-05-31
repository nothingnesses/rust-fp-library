//! Item generators owned by `#[document_module]`.
//!
//! These are not exported function-like procedural macros. They are
//! item-position markers that `#[document_module]` removes and replaces before
//! documentation validation runs.

use {
	super::{
		generator_builders,
		generator_descriptors::{
			self,
			EffectName,
			RunWrapperCoreMethod,
			RunWrapperMethod,
			WrapperName,
		},
	},
	crate::{
		core::constants::macros::{
			DEFINE_EFFECT,
			DEFINE_RUN_WRAPPER,
			DEFINE_RUN_WRAPPER_METHOD,
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

struct DefineRunWrapperMethodInput {
	wrapper_name: Ident,
	method_name: Ident,
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

impl Parse for DefineRunWrapperMethodInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		input.parse::<keyword::wrapper>()?;
		let wrapper_name = input.parse()?;
		input.parse::<Token![;]>()?;

		input.parse::<keyword::method>()?;
		let method_name = input.parse()?;
		input.parse::<Token![;]>()?;

		if !input.is_empty() {
			return Err(input.error(format!(
				"{DEFINE_RUN_WRAPPER_METHOD}! does not accept additional fields yet"
			)));
		}

		Ok(Self {
			wrapper_name,
			method_name,
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

fn is_define_run_wrapper_method(item_macro: &ItemMacro) -> bool {
	item_macro.mac.path.is_ident(DEFINE_RUN_WRAPPER_METHOD)
}

fn impl_item_is_define_run_wrapper(item_macro: &ImplItemMacro) -> bool {
	item_macro.mac.path.is_ident(DEFINE_RUN_WRAPPER)
}

fn impl_item_is_define_run_wrapper_method(item_macro: &ImplItemMacro) -> bool {
	item_macro.mac.path.is_ident(DEFINE_RUN_WRAPPER_METHOD)
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

	match EffectName::from_ident(&input.effect_name) {
		Some(effect_name) => {
			let _marker_tokens = generator_builders::define_effect_marker_tokens(effect_name);
			generator_builders::effect_items_from_descriptor(effect_name)
		}
		None => Err(syn::Error::new(
			input.effect_name.span(),
			format!(
				"{DEFINE_EFFECT}! currently only supports `effect Reader;` and `effect State;`"
			),
		)),
	}
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

	let wrapper_name = WrapperName::from_ident(&input.wrapper_name);
	let effect_name = EffectName::from_ident(&input.effect_name);
	let method_name = RunWrapperMethod::from_ident(&input.method_name);
	if let (Some(wrapper_name), Some(effect_name), Some(method_name)) =
		(wrapper_name, effect_name, method_name)
	{
		let _row_bounds = generator_descriptors::wrapper_method_row_bounds(
			wrapper_name,
			effect_name,
			method_name,
		);
		let _marker_tokens = generator_builders::define_run_wrapper_marker_tokens(
			wrapper_name,
			effect_name,
			method_name,
		);
		if let Some(items) = generator_builders::run_wrapper_impl_items_from_descriptor(
			wrapper_name,
			effect_name,
			method_name,
		) {
			return items;
		}
	}

	match (wrapper_name, effect_name, method_name) {
		(Some(_), Some(EffectName::Reader), _) => Err(syn::Error::new(
			input.method_name.span(),
			format!(
				"{DEFINE_RUN_WRAPPER}! currently only supports Reader methods `ask`, `asks`, and `run_reader`"
			),
		)),
		(Some(_), Some(EffectName::State), _) => Err(syn::Error::new(
			input.method_name.span(),
			format!(
				"{DEFINE_RUN_WRAPPER}! currently only supports State methods `get`, `put`, `modify`, and `run_state` for `wrapper Run;`, `wrapper RcRun;`, `wrapper ArcRun;`, `wrapper RunExplicit;`, `wrapper RcRunExplicit;`, and `wrapper ArcRunExplicit;`"
			),
		)),
		(None, Some(_), _) => Err(syn::Error::new(
			input.wrapper_name.span(),
			format!(
				"{DEFINE_RUN_WRAPPER}! currently only supports `wrapper Run;`, `wrapper RcRun;`, `wrapper ArcRun;`, `wrapper RunExplicit;`, `wrapper RcRunExplicit;`, and `wrapper ArcRunExplicit;`"
			),
		)),
		(_, None, _) => Err(syn::Error::new(
			input.effect_name.span(),
			format!(
				"{DEFINE_RUN_WRAPPER}! currently only supports `effect Reader;` and `effect State;`"
			),
		)),
	}
}

fn expand_define_run_wrapper_method_impl_item(
	item_macro: ImplItemMacro
) -> syn::Result<Vec<ImplItem>> {
	let span = item_macro.span();
	let input =
		syn::parse2::<DefineRunWrapperMethodInput>(item_macro.mac.tokens).map_err(|error| {
			syn::Error::new(
				span,
				format!(
					"{DEFINE_RUN_WRAPPER_METHOD}! expected `wrapper Run; method expand;`: {error}"
				),
			)
		})?;

	let wrapper_name = WrapperName::from_ident(&input.wrapper_name);
	let method_name = RunWrapperCoreMethod::from_ident(&input.method_name);
	if let (Some(wrapper_name), Some(method_name)) = (wrapper_name, method_name) {
		let _row_bounds =
			generator_descriptors::wrapper_core_method_row_bounds(wrapper_name, method_name);
		let _marker_tokens =
			generator_builders::define_run_wrapper_method_marker_tokens(wrapper_name, method_name);
		if let Some(items) = generator_builders::run_wrapper_method_impl_items_from_descriptor(
			wrapper_name,
			method_name,
		) {
			return items;
		}
	}

	match (wrapper_name, method_name) {
		(None, _) => Err(syn::Error::new(
			input.wrapper_name.span(),
			format!(
				"{DEFINE_RUN_WRAPPER_METHOD}! currently only supports `wrapper Run;`, `wrapper RcRun;`, `wrapper ArcRun;`, `wrapper RunExplicit;`, `wrapper RcRunExplicit;`, and `wrapper ArcRunExplicit;`"
			),
		)),
		(_, None) => Err(syn::Error::new(
			input.method_name.span(),
			format!(
				"{DEFINE_RUN_WRAPPER_METHOD}! currently only supports wrapper-wide methods `expand` and `weaken`"
			),
		)),
		(Some(_), Some(_)) => Err(syn::Error::new(
			input.method_name.span(),
			format!(
				"{DEFINE_RUN_WRAPPER_METHOD}! could not build the requested wrapper-wide method"
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
			ImplItem::Macro(item_macro) if impl_item_is_define_run_wrapper_method(&item_macro) => {
				expanded_items.extend(expand_define_run_wrapper_method_impl_item(item_macro)?);
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
			Item::Macro(item_macro) if is_define_run_wrapper_method(&item_macro) => {
				return Err(syn::Error::new(
					item_macro.span(),
					format!("{DEFINE_RUN_WRAPPER_METHOD}! must be used inside an impl block"),
				));
			}
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
