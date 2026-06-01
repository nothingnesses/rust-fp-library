//! Token-building helpers for `#[document_module]` item generators.
//!
//! The current builders are deliberately small: they provide the shared
//! token-to-AST boundary and descriptor-backed marker reconstruction that the
//! Reader / State migration can build on without changing the public macro
//! syntax.

mod first_order_effect_items;
mod fresh_wrapper_impl_items;
mod input_wrapper_impl_items;
mod kv_store_wrapper_impl_items;
mod output_wrapper_impl_items;
mod reader_effect_items;
mod reader_wrapper_impl_items;
mod run_wrapper_method_impl_items;
mod state_effect_items;
mod state_wrapper_impl_items;

use {
	super::generator_descriptors::{
		self,
		EffectCellVariant,
		EffectName,
		EffectSpec,
		RunWrapperCoreMethod,
		RunWrapperMethod,
		WrapperName,
	},
	crate::{
		core::constants::macros::{
			DEFINE_EFFECT,
			DEFINE_RUN_WRAPPER,
			DEFINE_RUN_WRAPPER_METHOD,
		},
		support::parsing::parse_many,
	},
	proc_macro2::{
		Span,
		TokenStream,
	},
	quote::{
		format_ident,
		quote,
	},
	syn::{
		Ident,
		ImplItem,
		Item,
		parse::{
			Parse,
			ParseStream,
		},
	},
};

struct GeneratedItems {
	items: Vec<Item>,
}

struct GeneratedImplItems {
	items: Vec<ImplItem>,
}

impl Parse for GeneratedItems {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(Self {
			items: parse_many(input)?,
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

fn ident(name: &str) -> Ident {
	format_ident!("{}", name, span = Span::call_site())
}

fn validate_effect_cell_siblings(spec: &EffectSpec) -> syn::Result<()> {
	if !spec.uses_pointer_brand_siblings {
		if !spec.brand_siblings.is_empty() {
			return Err(syn::Error::new(
				Span::call_site(),
				format!(
					"{:?} direct-payload effect spec must not declare pointer-brand siblings",
					spec.name
				),
			));
		}

		return Ok(());
	}

	for variant in [EffectCellVariant::Plain, EffectCellVariant::Send, EffectCellVariant::Boxed] {
		if !spec.brand_siblings.iter().any(|sibling| sibling.variant == variant) {
			return Err(syn::Error::new(
				Span::call_site(),
				format!("{:?} effect spec is missing the {:?} brand sibling", spec.name, variant),
			));
		}
	}

	Ok(())
}

pub(super) fn items_from_tokens(tokens: TokenStream) -> syn::Result<Vec<Item>> {
	Ok(syn::parse2::<GeneratedItems>(tokens)?.items)
}

pub(super) fn impl_items_from_tokens(tokens: TokenStream) -> syn::Result<Vec<ImplItem>> {
	Ok(syn::parse2::<GeneratedImplItems>(tokens)?.items)
}

pub(super) fn effect_items_from_descriptor(effect: EffectName) -> syn::Result<Vec<Item>> {
	let spec = generator_descriptors::effect_spec(effect).ok_or_else(|| {
		syn::Error::new(Span::call_site(), format!("{:?} effect spec is not registered", effect))
	})?;
	validate_effect_cell_siblings(spec)?;

	let tokens = match spec.name {
		EffectName::Fresh => first_order_effect_items::fresh_effect_items_tokens(),
		EffectName::Input => first_order_effect_items::input_effect_items_tokens(),
		EffectName::KVStore => first_order_effect_items::kv_store_effect_items_tokens(),
		EffectName::Output => first_order_effect_items::output_effect_items_tokens(),
		EffectName::Reader => reader_effect_items::reader_effect_items_tokens(),
		EffectName::State => state_effect_items::state_effect_items_tokens(),
	};

	items_from_tokens(tokens)
}

pub(super) fn run_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	effect: EffectName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	match effect {
		EffectName::Fresh =>
			fresh_wrapper_impl_items::fresh_wrapper_impl_items_from_descriptor(wrapper, method),
		EffectName::Input =>
			input_wrapper_impl_items::input_wrapper_impl_items_from_descriptor(wrapper, method),
		EffectName::KVStore =>
			kv_store_wrapper_impl_items::kv_store_wrapper_impl_items_from_descriptor(
				wrapper, method,
			),
		EffectName::Output =>
			output_wrapper_impl_items::output_wrapper_impl_items_from_descriptor(wrapper, method),
		EffectName::Reader =>
			reader_wrapper_impl_items::reader_wrapper_impl_items_from_descriptor(wrapper, method),
		EffectName::State =>
			state_wrapper_impl_items::state_wrapper_impl_items_from_descriptor(wrapper, method),
	}
}

pub(super) fn run_wrapper_method_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperCoreMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	let _descriptor = generator_descriptors::wrapper_method_descriptor(wrapper, method)?;
	let _row_bounds = generator_descriptors::wrapper_core_method_row_bounds(wrapper, method)?;

	run_wrapper_method_impl_items::run_wrapper_method_impl_items_from_descriptor(wrapper, method)
}

pub(super) fn define_effect_marker_tokens(effect: EffectName) -> TokenStream {
	let macro_ident = ident(DEFINE_EFFECT);
	let effect_ident = ident(effect.as_str());

	quote! {
		#macro_ident! {
			effect #effect_ident;
		}
	}
}

pub(super) fn define_run_wrapper_marker_tokens(
	wrapper: WrapperName,
	effect: EffectName,
	method: RunWrapperMethod,
) -> Option<TokenStream> {
	generator_descriptors::method_spec(effect, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let macro_ident = ident(DEFINE_RUN_WRAPPER);
	let wrapper_ident = ident(wrapper.as_str());
	let effect_ident = ident(effect.as_str());
	let method_ident = ident(method.as_str());

	Some(quote! {
		#macro_ident! {
			wrapper #wrapper_ident;
			effect #effect_ident;
			method #method_ident;
		}
	})
}

pub(super) fn define_run_wrapper_method_marker_tokens(
	wrapper: WrapperName,
	method: RunWrapperCoreMethod,
) -> Option<TokenStream> {
	generator_descriptors::wrapper_method_descriptor(wrapper, method)?;

	let macro_ident = ident(DEFINE_RUN_WRAPPER_METHOD);
	let wrapper_ident = ident(wrapper.as_str());
	let method_ident = ident(method.as_str());

	Some(quote! {
		#macro_ident! {
			wrapper #wrapper_ident;
			method #method_ident;
		}
	})
}

#[cfg(test)]
mod tests {
	use {
		super::*,
		syn::{
			ItemMacro,
			parse_quote,
		},
	};

	#[test]
	fn builds_define_effect_marker_from_descriptor() {
		let tokens = define_effect_marker_tokens(EffectName::Reader);
		let item: ItemMacro = parse_quote!(#tokens);
		assert!(item.mac.path.is_ident(DEFINE_EFFECT));
		assert_eq!(item.mac.tokens.to_string(), "effect Reader ;");
	}

	#[test]
	fn builds_define_run_wrapper_marker_from_descriptors() -> syn::Result<()> {
		let tokens = define_run_wrapper_marker_tokens(
			WrapperName::ArcRunExplicit,
			EffectName::State,
			RunWrapperMethod::RunState,
		)
		.ok_or_else(|| {
			syn::Error::new(Span::call_site(), "ArcRunExplicit State run_state should be supported")
		})?;
		let item: ItemMacro = parse_quote!(#tokens);
		assert!(item.mac.path.is_ident(DEFINE_RUN_WRAPPER));
		assert_eq!(
			item.mac.tokens.to_string(),
			"wrapper ArcRunExplicit ; effect State ; method run_state ;",
		);
		Ok(())
	}

	#[test]
	fn builds_define_run_wrapper_method_marker_from_descriptors() -> syn::Result<()> {
		let tokens = define_run_wrapper_method_marker_tokens(
			WrapperName::ArcRunExplicit,
			RunWrapperCoreMethod::Expand,
		)
		.ok_or_else(|| {
			syn::Error::new(
				Span::call_site(),
				"ArcRunExplicit expand should be a supported wrapper-wide method",
			)
		})?;
		let item: ItemMacro = parse_quote!(#tokens);
		assert!(item.mac.path.is_ident(DEFINE_RUN_WRAPPER_METHOD));
		assert_eq!(item.mac.tokens.to_string(), "wrapper ArcRunExplicit ; method expand ;",);
		Ok(())
	}

	#[test]
	fn rejects_marker_for_method_not_on_effect() {
		assert!(
			define_run_wrapper_marker_tokens(
				WrapperName::Run,
				EffectName::Reader,
				RunWrapperMethod::RunState,
			)
			.is_none()
		);
	}

	#[test]
	fn parses_generated_items_from_tokens() -> syn::Result<()> {
		let items = items_from_tokens(quote! {
			pub struct Generated;
			pub enum Other {
				Variant,
			}
		})?;
		assert_eq!(items.len(), 2);
		Ok(())
	}

	#[test]
	fn parses_generated_impl_items_from_tokens() -> syn::Result<()> {
		let items = impl_items_from_tokens(quote! {
			#[inline]
			pub fn generated(&self) {}
		})?;
		assert_eq!(items.len(), 1);
		Ok(())
	}

	#[test]
	fn builds_reader_effect_items_from_descriptor() -> syn::Result<()> {
		let items = effect_items_from_descriptor(EffectName::Reader)?;
		assert!(
			items.iter().any(|item| matches!(item, Item::Enum(item) if item.ident == "Reader"))
		);
		assert!(items.iter().any(|item| matches!(item, Item::Impl(item) if item.trait_.is_some())),);
		Ok(())
	}

	#[test]
	fn builds_state_effect_items_from_descriptor() -> syn::Result<()> {
		let items = effect_items_from_descriptor(EffectName::State)?;
		assert!(items.iter().any(|item| matches!(item, Item::Enum(item) if item.ident == "State")));
		assert!(items.iter().any(|item| matches!(item, Item::Impl(item) if item.trait_.is_some())),);
		Ok(())
	}

	#[test]
	fn builds_reader_wrapper_impl_items_from_descriptor() -> syn::Result<()> {
		let items = run_wrapper_impl_items_from_descriptor(
			WrapperName::ArcRunExplicit,
			EffectName::Reader,
			RunWrapperMethod::RunReader,
		)
		.ok_or_else(|| {
			syn::Error::new(
				Span::call_site(),
				"ArcRunExplicit Reader run_reader should be supported",
			)
		})??;

		assert!(
			items
				.iter()
				.any(|item| matches!(item, ImplItem::Fn(item) if item.sig.ident == "run_reader"))
		);
		Ok(())
	}

	#[test]
	fn builds_state_wrapper_impl_items_from_descriptor() -> syn::Result<()> {
		let items = run_wrapper_impl_items_from_descriptor(
			WrapperName::ArcRunExplicit,
			EffectName::State,
			RunWrapperMethod::RunState,
		)
		.ok_or_else(|| {
			syn::Error::new(Span::call_site(), "ArcRunExplicit State run_state should be supported")
		})??;

		assert!(
			items
				.iter()
				.any(|item| matches!(item, ImplItem::Fn(item) if item.sig.ident == "run_state"))
		);
		Ok(())
	}

	#[test]
	fn validates_wrapper_core_method_descriptors() -> syn::Result<()> {
		let items = run_wrapper_method_impl_items_from_descriptor(
			WrapperName::ArcRunExplicit,
			RunWrapperCoreMethod::Weaken,
		)
		.ok_or_else(|| {
			syn::Error::new(
				Span::call_site(),
				"ArcRunExplicit weaken should be a supported wrapper-wide method",
			)
		})??;

		assert!(
			items
				.iter()
				.any(|item| matches!(item, ImplItem::Fn(item) if item.sig.ident == "weaken")),
			"ArcRunExplicit weaken should emit a method body"
		);
		Ok(())
	}

	#[test]
	fn builds_default_run_expand_wrapper_method_from_descriptor() -> syn::Result<()> {
		let items = run_wrapper_method_impl_items_from_descriptor(
			WrapperName::Run,
			RunWrapperCoreMethod::Expand,
		)
		.ok_or_else(|| syn::Error::new(Span::call_site(), "Run expand should be supported"))??;

		assert!(
			items
				.iter()
				.any(|item| matches!(item, ImplItem::Fn(item) if item.sig.ident == "expand")),
			"default Run expand should emit a method body",
		);
		Ok(())
	}

	#[test]
	fn builds_expand_wrapper_methods_for_all_wrappers() -> syn::Result<()> {
		for wrapper in [
			WrapperName::Run,
			WrapperName::RcRun,
			WrapperName::ArcRun,
			WrapperName::RunExplicit,
			WrapperName::RcRunExplicit,
			WrapperName::ArcRunExplicit,
		] {
			let items = run_wrapper_method_impl_items_from_descriptor(
				wrapper,
				RunWrapperCoreMethod::Expand,
			)
			.ok_or_else(|| {
				syn::Error::new(
					Span::call_site(),
					format!("{wrapper:?} expand should be supported"),
				)
			})??;

			assert!(
				items
					.iter()
					.any(|item| matches!(item, ImplItem::Fn(item) if item.sig.ident == "expand")),
				"{wrapper:?} expand should emit a method body",
			);
		}

		Ok(())
	}

	#[test]
	fn builds_weaken_wrapper_methods_for_all_wrappers() -> syn::Result<()> {
		for wrapper in [
			WrapperName::Run,
			WrapperName::RcRun,
			WrapperName::ArcRun,
			WrapperName::RunExplicit,
			WrapperName::RcRunExplicit,
			WrapperName::ArcRunExplicit,
		] {
			let items = run_wrapper_method_impl_items_from_descriptor(
				wrapper,
				RunWrapperCoreMethod::Weaken,
			)
			.ok_or_else(|| {
				syn::Error::new(
					Span::call_site(),
					format!("{wrapper:?} weaken should be supported"),
				)
			})??;

			assert!(
				items
					.iter()
					.any(|item| matches!(item, ImplItem::Fn(item) if item.sig.ident == "weaken")),
				"{wrapper:?} weaken should emit a method body",
			);
		}

		Ok(())
	}
}
