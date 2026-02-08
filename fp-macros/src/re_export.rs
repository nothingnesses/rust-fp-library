use proc_macro2::TokenStream;
use quote::quote;
use std::{collections::HashMap, fs, path::Path};
use syn::{
	parse::{Parse, ParseStream},
	{Ident, Item, LitStr, Result, Token, Visibility, braced, parse_file},
};

/// The kind of item to re-export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemKind {
	Function,
	Trait,
}

pub struct ReexportInput {
	path: LitStr,
	aliases: HashMap<String, Ident>,
}

impl Parse for ReexportInput {
	fn parse(input: ParseStream) -> Result<Self> {
		let path: LitStr = input.parse()?;
		let mut aliases = HashMap::new();

		if input.peek(Token![,]) {
			input.parse::<Token![,]>()?;
			let content;
			braced!(content in input);
			while !content.is_empty() {
				let key_str = if content.peek(LitStr) {
					let s: LitStr = content.parse()?;
					s.value()
				} else {
					let i: Ident = content.parse()?;
					i.to_string()
				};

				content.parse::<Token![:]>()?;
				let value: Ident = content.parse()?;
				aliases.insert(key_str, value);
				if content.peek(Token![,]) {
					content.parse::<Token![,]>()?;
				}
			}
		}

		Ok(ReexportInput { path, aliases })
	}
}

/// Detects if a file uses a `pub use module_name::*;` re-export pattern.
/// Returns the module name if found (e.g., "inner").
fn detect_reexport_pattern(file: &syn::File) -> Option<String> {
	for item in &file.items {
		if let Item::Use(use_item) = item
			&& matches!(use_item.vis, Visibility::Public(_))
		{
			// Check if it's a glob use like `pub use inner::*;`
			if let syn::UseTree::Path(path) = &use_item.tree
				&& let syn::UseTree::Glob(_) = &*path.tree
			{
				return Some(path.ident.to_string());
			}
		}
	}
	None
}

/// Collects items from a file, handling both top-level and nested module patterns.
/// Applies both a visibility filter and an item filter.
fn collect_items<V, F>(
	file: &syn::File,
	reexport_module: Option<&str>,
	mut visibility_filter: V,
	mut item_filter: F,
) -> Vec<String>
where
	V: FnMut(&Item) -> bool,
	F: FnMut(&Item) -> Option<String>,
{
	if let Some(module_name) = reexport_module {
		// Collect items from the re-exported nested module
		file.items
			.iter()
			.filter_map(|item| {
				if let Item::Mod(mod_item) = item
					&& mod_item.ident == module_name
					&& let Some((_, items)) = &mod_item.content
				{
					return Some(
						items
							.iter()
							.filter(|item| visibility_filter(item))
							.filter_map(&mut item_filter)
							.collect::<Vec<_>>(),
					);
				}
				None
			})
			.flatten()
			.collect()
	} else {
		// Collect items from top level
		file.items
			.iter()
			.filter(|item| visibility_filter(item))
			.filter_map(item_filter)
			.collect()
	}
}

/// Extracts the visibility from any item type.
fn is_public_item(item: &Item) -> bool {
	match item {
		Item::Const(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Enum(i) => matches!(i.vis, Visibility::Public(_)),
		Item::ExternCrate(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Fn(i) => matches!(i.vis, Visibility::Public(_)),
		Item::ForeignMod(_) => false, // Foreign modules don't have visibility
		Item::Impl(_) => false, // Impl blocks don't have visibility
		Item::Macro(_) => false, // Macros don't have visibility in the same way
		Item::Mod(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Static(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Struct(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Trait(i) => matches!(i.vis, Visibility::Public(_)),
		Item::TraitAlias(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Type(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Union(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Use(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Verbatim(_) => false, // Cannot determine visibility
		_ => false, // Future-proofing for new syn variants
	}
}

/// Collects public items from a file, handling both top-level and nested module patterns.
fn collect_public_items<F>(
	file: &syn::File,
	reexport_module: Option<&str>,
	item_filter: F,
) -> Vec<String>
where
	F: FnMut(&Item) -> Option<String>,
{
	collect_items(file, reexport_module, is_public_item, item_filter)
}

/// Generic function to scan a directory and collect re-exports.
fn scan_directory_and_collect<F>(
	input: &ReexportInput,
	mut item_collector: F,
) -> Vec<TokenStream>
where
	F: FnMut(&str, &syn::File, Option<&str>) -> Vec<(String, TokenStream)>,
{
	let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
	let base_path = Path::new(&manifest_dir).join(input.path.value());

	let mut re_exports = Vec::new();

	if let Ok(entries) = fs::read_dir(&base_path) {
		for entry in entries.flatten() {
			let path = entry.path();
			if path.extension().and_then(|s| s.to_str()) == Some("rs") {
				let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) else {
					continue; // Skip files with invalid UTF-8 names
				};
				if file_stem == "mod" {
					continue;
				}

				let Ok(content) = fs::read_to_string(&path) else {
					continue; // Skip files that can't be read
				};
				if let Ok(file) = parse_file(&content) {
					let reexport_module = detect_reexport_pattern(&file);
					let items = item_collector(file_stem, &file, reexport_module.as_deref());

					for (_name, tokens) in items {
						re_exports.push(tokens);
					}
				}
			}
		}
	} else {
		panic!("Failed to read directory: {:?}", base_path);
	}

	// Sort re_exports for deterministic output
	re_exports.sort_by_key(|tokens| tokens.to_string());
	re_exports
}

/// Unified implementation for generating re-exports.
/// This function handles both function and trait re-exports based on the `kind` parameter.
fn generate_re_exports_impl(input: &ReexportInput, kind: ItemKind) -> TokenStream {
	let re_exports = scan_directory_and_collect(input, |file_stem, file, reexport_module| {
		// Collect public items based on kind (visibility is already filtered by collect_public_items)
		let items = collect_public_items(file, reexport_module, |item| match kind {
			ItemKind::Function => {
				if let Item::Fn(func) = item {
					return Some(func.sig.ident.to_string());
				}
				None
			}
			ItemKind::Trait => {
				if let Item::Trait(trait_item) = item {
					return Some(trait_item.ident.to_string());
				}
				None
			}
		});

		// Generate re-export tokens based on kind
		items
			.into_iter()
			.map(|item_name| {
				let item_ident = Ident::new(&item_name, proc_macro2::Span::call_site());
				let module_name = Ident::new(file_stem, proc_macro2::Span::call_site());

				let tokens = match kind {
					ItemKind::Function => {
						// Functions support both full qualified name and short name aliases
						let full_name = format!("{file_stem}::{item_name}");
						if let Some(alias) =
							input.aliases.get(&full_name).or_else(|| input.aliases.get(&item_name))
						{
							quote! { #module_name::#item_ident as #alias }
						} else {
							quote! { #module_name::#item_ident }
						}
					}
					ItemKind::Trait => {
						// Traits only support short name aliases and include `pub use` per item
						if let Some(alias) = input.aliases.get(&item_name) {
							quote! { pub use #module_name::#item_ident as #alias; }
						} else {
							quote! { pub use #module_name::#item_ident; }
						}
					}
				};

				(item_name, tokens)
			})
			.collect()
	});

	// Format the final output based on kind
	match kind {
		ItemKind::Function => {
			// Functions are grouped in a single `pub use` statement
			quote! {
				pub use crate::classes::{
					#(#re_exports),*
				};
			}
		}
		ItemKind::Trait => {
			// Traits have individual `pub use` statements
			quote! {
				#(#re_exports)*
			}
		}
	}
}

pub fn generate_function_re_exports_impl(input: ReexportInput) -> TokenStream {
	generate_re_exports_impl(&input, ItemKind::Function)
}

pub fn generate_trait_re_exports_impl(input: ReexportInput) -> TokenStream {
	generate_re_exports_impl(&input, ItemKind::Trait)
}
