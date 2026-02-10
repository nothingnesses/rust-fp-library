use super::generation::generate_documentation;
use crate::{
	core::{
		Result as OurResult, config::Config, constants::attributes::DOCUMENT_MODULE,
		error_handling::ErrorCollector,
	},
	resolution::extract_context,
	support::parsing::{parse_many, parse_non_empty},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
	Item, ItemMod,
	parse::{Parse, ParseStream},
	spanned::Spanned,
	visit_mut::{self, VisitMut},
};

pub struct DocumentModuleInput {
	pub items: Vec<Item>,
}

impl Parse for DocumentModuleInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let items = parse_many(input)?;
		let items = parse_non_empty(items, "Module documentation must contain at least one item")?;
		Ok(DocumentModuleInput { items })
	}
}

/// Represents the parsed input format for document_module macro.
enum ParsedInput {
	/// Outer attribute on a module: #[document_module] mod foo { ... }
	ModuleWrapper(ItemMod, syn::token::Brace, Vec<Item>),
	/// Inner attribute or direct items: #![document_module] ... items ...
	DirectItems(Vec<Item>),
}

/// Try to parse input as a module with outer attribute.
fn try_parse_module_wrapper(item: TokenStream) -> Option<ParsedInput> {
	if let Ok(mut item_mod) = syn::parse2::<ItemMod>(item) {
		if let Some((brace, mod_items)) = item_mod.content.take() {
			return Some(ParsedInput::ModuleWrapper(item_mod, brace, mod_items));
		} else {
			// mod foo; case - we can't see the content easily
			// Return None to fall through to error handling
			return None;
		}
	}
	None
}

/// Try to parse input as direct items (inner attribute case).
fn try_parse_direct_items(item: TokenStream) -> Option<ParsedInput> {
	if let Ok(input) = syn::parse2::<DocumentModuleInput>(item) {
		return Some(ParsedInput::DirectItems(input.items));
	}
	None
}

/// Try to parse input as a const block: const _: () = { ... };
fn try_parse_const_block(item: TokenStream) -> Result<ParsedInput, syn::Error> {
	let item_const = syn::parse2::<syn::ItemConst>(item)?;

	if let syn::Expr::Block(expr_block) = *item_const.expr {
		let items: Vec<Item> = expr_block
			.block
			.stmts
			.into_iter()
			.filter_map(|stmt| match stmt {
				syn::Stmt::Item(item) => Some(item),
				_ => None,
			})
			.collect();
		Ok(ParsedInput::DirectItems(items))
	} else {
		Err(syn::Error::new(
			item_const.span(),
			format!(
				"{DOCUMENT_MODULE} on a const item requires a block expression: const _: () = {{ ... }};"
			),
		))
	}
}

/// Parse the input token stream into one of the supported formats.
fn parse_document_module_input(item: TokenStream) -> Result<ParsedInput, syn::Error> {
	// Try to parse as ItemMod first (more specific), then fall back to DocumentModuleInput
	// This is critical: ItemMod must be checked first, otherwise `#[document_module] mod inner { ... }`
	// would be parsed as DocumentModuleInput containing a single module item, losing the wrapper.
	if let Some(parsed) = try_parse_module_wrapper(item.clone()) {
		return Ok(parsed);
	}

	if let Some(parsed) = try_parse_direct_items(item.clone()) {
		return Ok(parsed);
	}

	// Last attempt: const block
	if let Ok(parsed) = try_parse_const_block(item) {
		return Ok(parsed);
	}

	Err(syn::Error::new(
		proc_macro2::Span::call_site(),
		format!(
			"{DOCUMENT_MODULE} must be applied to a module, a const block, or used as an inner attribute in a module."
		),
	))
}

pub fn document_module_worker(
	_attr: TokenStream,
	item: TokenStream,
) -> OurResult<TokenStream> {
	let parsed_input = parse_document_module_input(item)?;

	let (module_wrapper, mut items) = match parsed_input {
		ParsedInput::ModuleWrapper(module, brace, items) => (Some((module, brace)), items),
		ParsedInput::DirectItems(items) => (None, items),
	};

	let mut config = Config::default();

	// Pass 1: Context Extraction (handles both top-level and nested)
	extract_context(&items, &mut config)?;

	// Also recursively extract from nested modules
	apply_to_nested_modules(&mut items, extract_context, &mut config)?;

	// Pass 2: Documentation Generation (handles both top-level and nested)
	generate_documentation(&mut items, &config)?;

	// Also recursively generate docs for nested modules (immutable config)
	apply_to_nested_modules_immut(&mut items, generate_documentation, &config)?;

	// Reconstruct module wrapper if needed (outer attribute case)
	if let Some((mut module, brace)) = module_wrapper {
		module.content = Some((brace, items));
		let output = quote!(#module);
		Ok(output)
	} else {
		let output = quote!(#(#items)*);
		Ok(output)
	}
}

/// Apply an operation to all nested modules recursively with mutable config.
fn apply_to_nested_modules<F>(
	items: &mut [Item],
	operation: F,
	config: &mut Config,
) -> syn::Result<()>
where
	F: Fn(&[Item], &mut Config) -> syn::Result<()> + Copy,
{
	let mut errors = ErrorCollector::new();
	let mut visitor = ModuleVisitor { operation, config, errors: &mut errors };

	for item in items {
		visitor.visit_item_mut(item);
	}

	errors.finish()
}

/// Apply an operation to all nested modules recursively with immutable config.
fn apply_to_nested_modules_immut<F>(
	items: &mut [Item],
	operation: F,
	config: &Config,
) -> syn::Result<()>
where
	F: Fn(&mut [Item], &Config) -> syn::Result<()> + Copy,
{
	let mut errors = ErrorCollector::new();
	let mut visitor = ModuleVisitorImmut { operation, config, errors: &mut errors };

	for item in items {
		visitor.visit_item_mut(item);
	}

	errors.finish()
}

/// Generic visitor for applying operations to nested modules (mutable config).
struct ModuleVisitor<'a, F>
where
	F: Fn(&[Item], &mut Config) -> syn::Result<()>,
{
	operation: F,
	config: &'a mut Config,
	errors: &'a mut ErrorCollector,
}

impl<'a, F> VisitMut for ModuleVisitor<'a, F>
where
	F: Fn(&[Item], &mut Config) -> syn::Result<()>,
{
	fn visit_item_mod_mut(
		&mut self,
		module: &mut ItemMod,
	) {
		if let Some((_, ref items)) = module.content {
			if let Err(e) = (self.operation)(items, self.config) {
				self.errors.push(e);
			}
			// Recursively process nested modules
			visit_mut::visit_item_mod_mut(self, module);
		}
	}
}

/// Generic visitor for applying operations to nested modules (immutable config).
struct ModuleVisitorImmut<'a, F>
where
	F: Fn(&mut [Item], &Config) -> syn::Result<()>,
{
	operation: F,
	config: &'a Config,
	errors: &'a mut ErrorCollector,
}

impl<'a, F> VisitMut for ModuleVisitorImmut<'a, F>
where
	F: Fn(&mut [Item], &Config) -> syn::Result<()>,
{
	fn visit_item_mod_mut(
		&mut self,
		module: &mut ItemMod,
	) {
		if let Some((_, ref mut items)) = module.content {
			if let Err(e) = (self.operation)(items, self.config) {
				self.errors.push(e);
			}
			// Recursively process nested modules
			visit_mut::visit_item_mod_mut(self, module);
		}
	}
}
