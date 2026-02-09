use super::generation::generate_docs;
use crate::{
	core::{config::Config, error_handling::ErrorCollector},
	resolution::extract_context,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
	Item, Result,
	parse::{Parse, ParseStream},
	spanned::Spanned,
	visit_mut::{self, VisitMut},
};

pub struct DocumentModuleInput {
	pub items: Vec<Item>,
}

impl Parse for DocumentModuleInput {
	fn parse(input: ParseStream) -> Result<Self> {
		let mut items = Vec::new();
		while !input.is_empty() {
			items.push(input.parse()?);
		}
		Ok(DocumentModuleInput { items })
	}
}

pub fn document_module_impl(
	_attr: TokenStream,
	item: TokenStream,
) -> TokenStream {
	// Track if we need to reconstruct a module wrapper
	let mut module_wrapper: Option<(syn::ItemMod, syn::token::Brace)> = None;

	// Try to parse as ItemMod first (more specific), then fall back to DocumentModuleInput
	// This is critical: ItemMod must be checked first, otherwise `#[document_module] mod inner { ... }`
	// would be parsed as DocumentModuleInput containing a single module item, losing the wrapper.
	let mut items = if let Ok(mut item_mod) = syn::parse2::<syn::ItemMod>(item.clone()) {
		// Outer attribute on a module case
		if let Some((brace, mod_items)) = item_mod.content.take() {
			// Store the module wrapper and brace token to reconstruct later
			module_wrapper = Some((item_mod, brace));
			mod_items
		} else {
			// mod foo; case - we can't see the content easily
			return syn::Error::new(
				item_mod.span(),
				"document_module cannot see the content of file modules when used as an outer attribute. Use an inner attribute #![document_module] instead, or wrap the content in a mod block.",
			).to_compile_error();
		}
	} else if let Ok(input) = syn::parse2::<DocumentModuleInput>(item.clone()) {
		// Inner attribute case or direct items
		input.items
	} else if let Ok(item_const) = syn::parse2::<syn::ItemConst>(item) {
		// Outer attribute on a const block case: const _: () = { ... };
		if let syn::Expr::Block(expr_block) = *item_const.expr {
			expr_block
				.block
				.stmts
				.into_iter()
				.filter_map(|stmt| match stmt {
					syn::Stmt::Item(item) => Some(item),
					_ => None,
				})
				.collect()
		} else {
			return syn::Error::new(
				item_const.span(),
				"document_module on a const item requires a block expression: const _: () = { ... };",
			)
			.to_compile_error();
		}
	} else {
		return syn::Error::new(
			proc_macro2::Span::call_site(),
			"document_module must be applied to a module, a const block, or used as an inner attribute in a module.",
		).to_compile_error();
	};

	let mut config = Config::default();

	// Pass 1: Context Extraction (handles both top-level and nested)
	if let Err(e) = extract_context(&items, &mut config) {
		return e.to_compile_error();
	}

	// Also recursively extract from nested modules
	let mut extractor =
		ContextExtractorVisitor { config: &mut config, errors: ErrorCollector::new() };
	for item in &mut items {
		extractor.visit_item_mut(item);
	}
	if let Err(e) = extractor.errors.finish() {
		return e.to_compile_error();
	}

	// Pass 2: Documentation Generation (handles both top-level and nested)
	if let Err(e) = generate_docs(&mut items, &config) {
		return e.to_compile_error();
	}

	// Also recursively generate docs for nested modules
	let mut generator = DocGeneratorVisitor { config: &config, errors: ErrorCollector::new() };
	for item in &mut items {
		generator.visit_item_mut(item);
	}
	if let Err(e) = generator.errors.finish() {
		return e.to_compile_error();
	}

	// Reconstruct module wrapper if needed (outer attribute case)
	if let Some((mut module, brace)) = module_wrapper {
		module.content = Some((brace, items));
		let output = quote!(#module);
		output
	} else {
		let output = quote!(#(#items)*);
		output
	}
}

/// Visitor for recursively extracting context from nested modules (Pass 1)
struct ContextExtractorVisitor<'a> {
	config: &'a mut Config,
	errors: ErrorCollector,
}

impl<'a> VisitMut for ContextExtractorVisitor<'a> {
	fn visit_item_mod_mut(
		&mut self,
		module: &mut syn::ItemMod,
	) {
		if let Some((_, ref items)) = module.content {
			// Extract context from this module's items
			if let Err(e) = extract_context(items, self.config) {
				self.errors.push(e);
			}

			// Recursively process nested modules
			visit_mut::visit_item_mod_mut(self, module);
		}
	}
}

/// Visitor for recursively generating documentation in nested modules (Pass 2)
struct DocGeneratorVisitor<'a> {
	config: &'a Config,
	errors: ErrorCollector,
}

impl<'a> VisitMut for DocGeneratorVisitor<'a> {
	fn visit_item_mod_mut(
		&mut self,
		module: &mut syn::ItemMod,
	) {
		if let Some((_, ref mut items)) = module.content {
			// Generate docs for this module's items
			if let Err(e) = generate_docs(items, self.config) {
				self.errors.push(e);
			}

			// Recursively process nested modules
			visit_mut::visit_item_mod_mut(self, module);
		}
	}
}
