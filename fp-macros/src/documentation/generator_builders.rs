//! Token-building helpers for `#[document_module]` item generators.
//!
//! The current builders are deliberately small: they provide the shared
//! token-to-AST boundary and descriptor-backed marker reconstruction that the
//! Reader / State migration can build on without changing the public macro
//! syntax.

mod reader_effect_items;
mod state_effect_items;

use {
	super::generator_descriptors::{
		self,
		EffectCellVariant,
		EffectName,
		EffectSpec,
		RunWrapperMethod,
		WrapperName,
	},
	crate::{
		core::constants::macros::{
			DEFINE_EFFECT,
			DEFINE_RUN_WRAPPER,
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

pub(super) fn impl_items_from_source(source: &str) -> syn::Result<Vec<ImplItem>> {
	let tokens: TokenStream = source.parse()?;
	impl_items_from_tokens(tokens)
}

pub(super) fn effect_items_from_descriptor(effect: EffectName) -> syn::Result<Vec<Item>> {
	let spec = generator_descriptors::effect_spec(effect).ok_or_else(|| {
		syn::Error::new(Span::call_site(), format!("{:?} effect spec is not registered", effect))
	})?;
	validate_effect_cell_siblings(spec)?;

	let tokens = match spec.name {
		EffectName::Reader => reader_effect_items::reader_effect_items_tokens(),
		EffectName::State => state_effect_items::state_effect_items_tokens(),
	};

	items_from_tokens(tokens)
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
}
