use super::{generation::generate_documentation, validation::validate_documentation};
use crate::{
	core::{
		Result as OurResult, config::Config, constants::attributes::DOCUMENT_MODULE,
		error_handling::ErrorCollector,
	},
	resolution::get_context,
	support::parsing::{parse_many, parse_non_empty, parse_with_dispatch},
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

/// Configuration for document_module validation
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum ValidationMode {
	/// Validation enabled - emit warnings for missing documentation (default)
	#[default]
	On,
	/// Validation disabled - no warnings
	Off,
}

/// Parse validation mode from attribute arguments
fn parse_validation_mode(attr: TokenStream) -> syn::Result<ValidationMode> {
	if attr.is_empty() {
		return Ok(ValidationMode::default());
	}

	let attr_str = attr.to_string();
	match attr_str.trim() {
		"no_validation" => Ok(ValidationMode::Off),
		_ => Err(syn::Error::new(
			attr.span(),
			format!("Unknown validation mode '{attr_str}'. Valid option: 'no_validation'"),
		)),
	}
}

/// Represents the parsed input format for document_module macro.
enum ParsedInput {
	/// Outer attribute on a module: #[document_module] mod foo { ... }
	ModuleWrapper(ItemMod, syn::token::Brace, Vec<Item>),
	/// Direct items (used for const block pattern): const _: () = { ... items ... };
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

/// Try to parse input as direct items (const block pattern).
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
	parse_with_dispatch(
		item,
		vec![
			Box::new(|tokens| {
				try_parse_module_wrapper(tokens).ok_or_else(|| {
					syn::Error::new(proc_macro2::Span::call_site(), "Not a module wrapper")
				})
			}),
			Box::new(|tokens| {
				try_parse_direct_items(tokens).ok_or_else(|| {
					syn::Error::new(proc_macro2::Span::call_site(), "Not direct items")
				})
			}),
			Box::new(try_parse_const_block),
		],
		&format!(
			"{DOCUMENT_MODULE} must be applied to a module or a const block (e.g., const _: () = {{ ... }})."
		),
	)
}

pub fn document_module_worker(
	attr: TokenStream,
	item: TokenStream,
) -> OurResult<TokenStream> {
	let parsed_input = parse_document_module_input(item)?;

	let (module_wrapper, mut items) = match parsed_input {
		ParsedInput::ModuleWrapper(module, brace, items) => (Some((module, brace)), items),
		ParsedInput::DirectItems(items) => (None, items),
	};

	// Parse validation mode from attribute
	let validation_mode = parse_validation_mode(attr)?;

	let mut config = Config::default();

	// Pass 1: Context Extraction (handles both top-level and nested)
	get_context(&items, &mut config)?;

	// Also recursively extract from nested modules
	apply_to_nested_modules(&mut items, get_context, &mut config)?;

	// Pass 1.5: Validation (emit warnings for missing documentation attributes)
	let warning_tokens: Vec<TokenStream> = if validation_mode != ValidationMode::Off {
		let warnings = validate_documentation(&items);

		// Also recursively validate nested modules
		let nested_warnings = validate_nested_modules(&items);

		// Convert errors to compile-time errors
		warnings.into_iter().chain(nested_warnings).map(|e| e.to_compile_error()).collect()
	} else {
		Vec::new()
	};

	// Pass 2: Documentation Generation (handles both top-level and nested)
	generate_documentation(&mut items, &config)?;

	// Also recursively generate docs for nested modules (immutable config)
	apply_to_nested_modules_immut(&mut items, generate_documentation, &config)?;

	// Reconstruct module wrapper if needed (outer attribute case)
	let items_output = if let Some((mut module, brace)) = module_wrapper {
		module.content = Some((brace, items));
		quote!(#module)
	} else {
		quote!(#(#items)*)
	};

	// Combine warnings with output if validation is enabled
	// Warnings are emitted first so they appear in the compiler output
	let output = if !warning_tokens.is_empty() {
		quote! {
			#(#warning_tokens)*
			#items_output
		}
	} else {
		items_output
	};

	Ok(output)
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

/// Recursively validate all nested modules and collect warnings.
fn validate_nested_modules(items: &[Item]) -> Vec<syn::Error> {
	let mut warnings = Vec::new();

	for item in items {
		if let Item::Mod(module) = item
			&& let Some((_, ref nested_items)) = module.content
		{
			// Validate this module's items
			warnings.extend(validate_documentation(nested_items));

			// Recursively validate nested modules
			warnings.extend(validate_nested_modules(nested_items));
		}
	}

	warnings
}
