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
		EffectOperationShape,
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

fn validate_effect_descriptor(spec: &EffectSpec) -> syn::Result<()> {
	let known_shape = generator_descriptors::known_operation_shapes()
		.iter()
		.any(|shape| *shape == spec.operation_shape);
	if !known_shape {
		return Err(syn::Error::new(
			Span::call_site(),
			format!(
				"{:?} effect spec uses unknown operation shape {:?}",
				spec.name, spec.operation_shape
			),
		));
	}

	let shape_uses_pointer_brand_siblings = spec.operation_shape.uses_pointer_brand_siblings();
	if spec.uses_pointer_brand_siblings != shape_uses_pointer_brand_siblings {
		return Err(syn::Error::new(
			Span::call_site(),
			format!(
				"{:?} {:?} effect spec has inconsistent pointer-brand sibling metadata",
				spec.name, spec.operation_shape
			),
		));
	}

	if !shape_uses_pointer_brand_siblings {
		if !spec.brand_siblings.is_empty() {
			return Err(syn::Error::new(
				Span::call_site(),
				format!(
					"{:?} {:?} effect spec must not declare pointer-brand siblings",
					spec.name, spec.operation_shape
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
	validate_effect_descriptor(spec)?;

	let tokens = match (spec.operation_shape, spec.name) {
		(EffectOperationShape::CoroutineYieldStatus, EffectName::Coroutine) =>
			first_order_effect_items::coroutine_effect_items_tokens(),
		(EffectOperationShape::FixedMessageAbort, EffectName::Fail) =>
			first_order_effect_items::fail_effect_items_tokens(),
		(EffectOperationShape::RequestValueContinuation, EffectName::Fresh) =>
			first_order_effect_items::fresh_effect_items_tokens(),
		(EffectOperationShape::RequestValueContinuation, EffectName::Input) =>
			first_order_effect_items::input_effect_items_tokens(),
		(EffectOperationShape::KeyValueStore, EffectName::KVStore) =>
			first_order_effect_items::kv_store_effect_items_tokens(),
		(EffectOperationShape::DirectPayload, EffectName::Log) =>
			first_order_effect_items::log_effect_items_tokens(),
		(EffectOperationShape::DirectPayload, EffectName::Output) =>
			first_order_effect_items::output_effect_items_tokens(),
		(EffectOperationShape::ReaderEnvironment, EffectName::Reader) =>
			reader_effect_items::reader_effect_items_tokens(),
		(EffectOperationShape::StateCell, EffectName::State) =>
			state_effect_items::state_effect_items_tokens(),
		_ =>
			return Err(syn::Error::new(
				Span::call_site(),
				format!(
					"{:?} effect spec has no builder for operation shape {:?}",
					spec.name, spec.operation_shape
				),
			)),
	};

	items_from_tokens(tokens)
}

pub(super) fn run_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	effect: EffectName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	let spec = generator_descriptors::effect_spec(effect)?;
	match (spec.operation_shape, effect) {
		(EffectOperationShape::RequestValueContinuation, EffectName::Fresh) =>
			fresh_wrapper_impl_items::fresh_wrapper_impl_items_from_descriptor(wrapper, method),
		(EffectOperationShape::RequestValueContinuation, EffectName::Input) =>
			input_wrapper_impl_items::input_wrapper_impl_items_from_descriptor(wrapper, method),
		(EffectOperationShape::KeyValueStore, EffectName::KVStore) =>
			kv_store_wrapper_impl_items::kv_store_wrapper_impl_items_from_descriptor(
				wrapper, method,
			),
		(EffectOperationShape::DirectPayload, EffectName::Output) =>
			output_wrapper_impl_items::output_wrapper_impl_items_from_descriptor(wrapper, method),
		(EffectOperationShape::ReaderEnvironment, EffectName::Reader) =>
			reader_wrapper_impl_items::reader_wrapper_impl_items_from_descriptor(wrapper, method),
		(EffectOperationShape::StateCell, EffectName::State) =>
			state_wrapper_impl_items::state_wrapper_impl_items_from_descriptor(wrapper, method),
		_ => Some(Err(syn::Error::new(
			Span::call_site(),
			format!(
				"{:?} effect spec has no wrapper builder for operation shape {:?}",
				effect, spec.operation_shape
			),
		))),
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
		super::{
			super::generator_descriptors::{
				BrandSibling,
				MethodSpec,
				PointerMode,
				Sendability,
			},
			*,
		},
		syn::{
			ItemMacro,
			Type,
			parse_quote,
		},
	};

	const TEST_METHODS: &[MethodSpec] = &[];
	const TEST_BRAND_SIBLINGS: &[BrandSibling] = &[BrandSibling {
		variant: EffectCellVariant::Plain,
		cell_type: "Test",
		brand_type: "TestBrand",
		pointer_mode: PointerMode::RcFn,
		sendability: Sendability::Local,
	}];

	fn clone_impl_self_type_names(items: &[Item]) -> Vec<String> {
		items
			.iter()
			.filter_map(|item| match item {
				Item::Impl(item_impl)
					if item_impl
						.trait_
						.as_ref()
						.is_some_and(|(_, path, _)| path.is_ident("Clone")) =>
					Some(&*item_impl.self_ty),
				_ => None,
			})
			.filter_map(|self_ty| match self_ty {
				Type::Path(type_path) =>
					type_path.path.segments.last().map(|segment| segment.ident.to_string()),
				_ => None,
			})
			.collect()
	}

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
	fn validates_operation_shape_pointer_brand_consistency() {
		let spec = EffectSpec {
			name: EffectName::Output,
			operation_shape: EffectOperationShape::DirectPayload,
			uses_pointer_brand_siblings: true,
			brand_siblings: &[],
			methods: TEST_METHODS,
		};

		let error = validate_effect_descriptor(&spec).expect_err("inconsistent shape should fail");
		assert!(
			error.to_string().contains("inconsistent pointer-brand sibling metadata"),
			"validation should explain the operation-shape mismatch; got: {error}",
		);
	}

	#[test]
	fn validates_direct_payload_shapes_reject_brand_siblings() {
		let spec = EffectSpec {
			name: EffectName::Output,
			operation_shape: EffectOperationShape::DirectPayload,
			uses_pointer_brand_siblings: false,
			brand_siblings: TEST_BRAND_SIBLINGS,
			methods: TEST_METHODS,
		};

		let error =
			validate_effect_descriptor(&spec).expect_err("direct payload should reject siblings");
		assert!(
			error.to_string().contains("must not declare pointer-brand siblings"),
			"validation should reject direct-payload siblings; got: {error}",
		);
	}

	#[test]
	fn validates_pointer_brand_shapes_require_all_siblings() {
		let spec = EffectSpec {
			name: EffectName::Fresh,
			operation_shape: EffectOperationShape::RequestValueContinuation,
			uses_pointer_brand_siblings: true,
			brand_siblings: TEST_BRAND_SIBLINGS,
			methods: TEST_METHODS,
		};

		let error = validate_effect_descriptor(&spec).expect_err("missing siblings should fail");
		assert!(
			error.to_string().contains("missing the Send brand sibling"),
			"validation should name the missing sibling variant; got: {error}",
		);
	}

	#[test]
	fn builds_effect_items_from_all_current_operation_shapes() -> syn::Result<()> {
		for effect in [
			EffectName::Coroutine,
			EffectName::Fail,
			EffectName::Fresh,
			EffectName::Input,
			EffectName::KVStore,
			EffectName::Log,
			EffectName::Output,
			EffectName::Reader,
			EffectName::State,
		] {
			let items = effect_items_from_descriptor(effect)?;
			assert!(
				!items.is_empty(),
				"{effect:?} should route through its operation-shape builder",
			);
		}

		Ok(())
	}

	#[test]
	fn builds_w12_effect_items_from_descriptors() -> syn::Result<()> {
		let coroutine_items = effect_items_from_descriptor(EffectName::Coroutine)?;
		assert!(
			coroutine_items
				.iter()
				.any(|item| matches!(item, Item::Enum(item) if item.ident == "Coroutine"))
		);
		assert!(
			coroutine_items
				.iter()
				.any(|item| matches!(item, Item::Enum(item) if item.ident == "SendCoroutine"))
		);
		assert!(
			coroutine_items
				.iter()
				.any(|item| matches!(item, Item::Enum(item) if item.ident == "BoxCoroutine"))
		);
		assert!(
			coroutine_items
				.iter()
				.any(|item| matches!(item, Item::Enum(item) if item.ident == "RunCoroutineStatus"))
		);
		assert!(coroutine_items.iter().any(
			|item| matches!(item, Item::Enum(item) if item.ident == "ArcRunExplicitCoroutineStatus")
		));
		let clone_impls = clone_impl_self_type_names(&coroutine_items);
		for cloneable_status in [
			"RcRunCoroutineStatus",
			"ArcRunCoroutineStatus",
			"RcRunExplicitCoroutineStatus",
			"ArcRunExplicitCoroutineStatus",
		] {
			assert!(
				clone_impls.iter().any(|name| name == cloneable_status),
				"multi-shot status should have a generated Clone impl: {cloneable_status}",
			);
		}
		for one_shot_status in ["RunCoroutineStatus", "RunExplicitCoroutineStatus"] {
			assert!(
				!clone_impls.iter().any(|name| name == one_shot_status),
				"one-shot status should remain non-Clone: {one_shot_status}",
			);
		}

		let log_items = effect_items_from_descriptor(EffectName::Log)?;
		assert!(
			log_items.iter().any(|item| matches!(item, Item::Enum(item) if item.ident == "Log"))
		);
		assert!(
			!log_items
				.iter()
				.any(|item| matches!(item, Item::Enum(item) if item.ident == "SendLog"))
		);

		let fail_items = effect_items_from_descriptor(EffectName::Fail)?;
		assert!(
			fail_items.iter().any(|item| matches!(item, Item::Enum(item) if item.ident == "Fail"))
		);
		assert!(
			!fail_items
				.iter()
				.any(|item| matches!(item, Item::Enum(item) if item.ident == "SendFail"))
		);

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
