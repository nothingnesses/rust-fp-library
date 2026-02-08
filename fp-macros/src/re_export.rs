use proc_macro2::TokenStream;
use quote::quote;
use std::{collections::HashMap, fs, path::Path};
use syn::{
	parse::{Parse, ParseStream},
	{Ident, Item, LitStr, Result, Token, Visibility, braced, parse_file},
};

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

/// Collects public items from a file, handling both top-level and nested module patterns.
fn collect_public_items<F>(
	file: &syn::File,
	reexport_module: Option<&str>,
	mut filter: F,
) -> Vec<String>
where
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
					return Some(items.iter().filter_map(&mut filter).collect::<Vec<_>>());
				}
				None
			})
			.flatten()
			.collect()
	} else {
		// Collect items from top level
		file.items.iter().filter_map(filter).collect()
	}
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

pub fn generate_function_re_exports_impl(input: ReexportInput) -> TokenStream {
	let re_exports = scan_directory_and_collect(&input, |file_stem, file, reexport_module| {
		// Collect public functions
		let functions = collect_public_items(file, reexport_module, |item| {
			if let Item::Fn(func) = item
				&& matches!(func.vis, Visibility::Public(_))
			{
				return Some(func.sig.ident.to_string());
			}
			None
		});

		// Generate re-export tokens for each function
		functions
			.into_iter()
			.map(|fn_name| {
				let fn_ident = Ident::new(&fn_name, proc_macro2::Span::call_site());
				let module_name = Ident::new(file_stem, proc_macro2::Span::call_site());
				let full_name = format!("{file_stem}::{fn_name}");

				let tokens = if let Some(alias) =
					input.aliases.get(&full_name).or_else(|| input.aliases.get(&fn_name))
				{
					quote! { #module_name::#fn_ident as #alias }
				} else {
					quote! { #module_name::#fn_ident }
				};

				(fn_name, tokens)
			})
			.collect()
	});

	quote! {
		pub use crate::classes::{
			#(#re_exports),*
		};
	}
}

pub fn generate_trait_re_exports_impl(input: ReexportInput) -> TokenStream {
	let re_exports = scan_directory_and_collect(&input, |file_stem, file, reexport_module| {
		// Collect public traits
		let traits = collect_public_items(file, reexport_module, |item| {
			if let Item::Trait(trait_item) = item
				&& matches!(trait_item.vis, Visibility::Public(_))
			{
				return Some(trait_item.ident.to_string());
			}
			None
		});

		// Generate re-export tokens for each trait
		traits
			.into_iter()
			.map(|trait_name| {
				let trait_ident = Ident::new(&trait_name, proc_macro2::Span::call_site());
				let module_name = Ident::new(file_stem, proc_macro2::Span::call_site());

				let tokens = if let Some(alias) = input.aliases.get(&trait_name) {
					quote! { pub use #module_name::#trait_ident as #alias; }
				} else {
					quote! { pub use #module_name::#trait_ident; }
				};

				(trait_name, tokens)
			})
			.collect()
	});

	quote! {
		#(#re_exports)*
	}
}
