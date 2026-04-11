use {
	crate::core::constants::{
		configuration,
		re_export,
	},
	proc_macro2::TokenStream,
	quote::quote,
	std::{
		collections::{
			HashMap,
			HashSet,
		},
		fs,
		path::Path,
	},
	syn::{
		Ident,
		Item,
		LitStr,
		Result,
		Token,
		Visibility,
		braced,
		parse::{
			Parse,
			ParseStream,
		},
		parse_file,
	},
};

/// Trait for formatting re-exports based on item type.
///
/// This trait abstracts the differences between function and trait re-export formatting,
/// eliminating code duplication through polymorphism.
pub trait ReExportFormatter {
	/// Formats a single re-export statement for an item.
	fn format_item(
		&self,
		module_name: &Ident,
		item_ident: &Ident,
		alias: Option<&Ident>,
	) -> TokenStream;

	/// Formats the final output combining all re-exports.
	fn format_output(
		&self,
		re_exports: Vec<TokenStream>,
		base_path: &syn::Path,
	) -> TokenStream;

	/// Determines if this formatter should process the given item.
	fn matches_item(
		&self,
		item: &Item,
	) -> Option<String>;
}

/// Formatter for function re-exports.
pub struct FunctionFormatter;

impl ReExportFormatter for FunctionFormatter {
	fn format_item(
		&self,
		module_name: &Ident,
		item_ident: &Ident,
		alias: Option<&Ident>,
	) -> TokenStream {
		if let Some(alias) = alias {
			quote! { #module_name::#item_ident as #alias }
		} else {
			quote! { #module_name::#item_ident }
		}
	}

	fn format_output(
		&self,
		re_exports: Vec<TokenStream>,
		base_path: &syn::Path,
	) -> TokenStream {
		// Functions are grouped in a single `pub use` statement
		quote! {
			pub use #base_path::{
				#(#re_exports),*
			};
		}
	}

	fn matches_item(
		&self,
		item: &Item,
	) -> Option<String> {
		if let Item::Fn(func) = item { Some(func.sig.ident.to_string()) } else { None }
	}
}

/// Formatter for trait re-exports.
pub struct TraitFormatter;

impl ReExportFormatter for TraitFormatter {
	fn format_item(
		&self,
		module_name: &Ident,
		item_ident: &Ident,
		alias: Option<&Ident>,
	) -> TokenStream {
		// Traits include `pub use` per item
		if let Some(alias) = alias {
			quote! { pub use #module_name::#item_ident as #alias; }
		} else {
			quote! { pub use #module_name::#item_ident; }
		}
	}

	fn format_output(
		&self,
		re_exports: Vec<TokenStream>,
		_base_path: &syn::Path,
	) -> TokenStream {
		// Traits have individual `pub use` statements
		quote! {
			#(#re_exports)*
		}
	}

	fn matches_item(
		&self,
		item: &Item,
	) -> Option<String> {
		if let Item::Trait(trait_item) = item { Some(trait_item.ident.to_string()) } else { None }
	}
}

pub struct ReExportInput {
	path: LitStr,
	aliases: HashMap<String, Ident>,
	exclusions: HashSet<String>,
}

impl Parse for ReExportInput {
	fn parse(input: ParseStream) -> Result<Self> {
		let path: LitStr = input.parse()?;
		let mut aliases = HashMap::new();
		let mut exclusions = HashSet::new();

		// Parse optional alias map: { "module::name": alias, ... }
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

		// Parse optional exclusion list: exclude { "module::name", ... }
		if input.peek(Token![,]) {
			input.parse::<Token![,]>()?;
			let exclude_kw: Ident = input.parse()?;
			if exclude_kw != "exclude" {
				return Err(syn::Error::new(exclude_kw.span(), "expected `exclude`"));
			}
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
				exclusions.insert(key_str);
				if content.peek(Token![,]) {
					content.parse::<Token![,]>()?;
				}
			}
		}

		Ok(ReExportInput {
			path,
			aliases,
			exclusions,
		})
	}
}

/// Detects if a file uses a `pub use module_name::*;` re-export pattern.
/// Returns the module name if found (e.g., "inner").
fn detect_re_export_pattern(file: &syn::File) -> Option<String> {
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
	F: FnMut(&Item) -> Option<String>, {
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
		file.items.iter().filter(|item| visibility_filter(item)).filter_map(item_filter).collect()
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
		Item::Impl(_) => false,       // Impl blocks don't have visibility
		Item::Macro(_) => false,      // Macros don't have visibility in the same way
		Item::Mod(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Static(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Struct(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Trait(i) => matches!(i.vis, Visibility::Public(_)),
		Item::TraitAlias(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Type(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Union(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Use(i) => matches!(i.vis, Visibility::Public(_)),
		Item::Verbatim(_) => false, // Cannot determine visibility
		_ => false,                 // Future-proofing for new syn variants
	}
}

/// Collects public items from a file, handling both top-level and nested module patterns.
fn collect_public_items<F>(
	file: &syn::File,
	reexport_module: Option<&str>,
	item_filter: F,
) -> Vec<String>
where
	F: FnMut(&Item) -> Option<String>, {
	collect_items(file, reexport_module, is_public_item, item_filter)
}

/// Generic function to scan a directory and collect re-exports using a formatter.
fn scan_directory_and_collect(
	input: &ReExportInput,
	formatter: &dyn ReExportFormatter,
) -> Vec<TokenStream> {
	// SAFETY: CARGO_MANIFEST_DIR is always set by Cargo during compilation
	#[expect(clippy::expect_used, reason = "CARGO_MANIFEST_DIR is always set by Cargo")]
	let manifest_dir =
		std::env::var(configuration::CARGO_MANIFEST_DIR).expect("CARGO_MANIFEST_DIR not set");
	let base_path = Path::new(&manifest_dir).join(input.path.value());

	let mut re_exports = Vec::new();

	if let Ok(entries) = fs::read_dir(&base_path) {
		for entry in entries.flatten() {
			let path = entry.path();
			if path.extension().and_then(|s| s.to_str()) == Some(re_export::RS_EXTENSION) {
				let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) else {
					continue; // Skip files with invalid UTF-8 names
				};
				if file_stem == re_export::MOD_FILE_STEM {
					continue;
				}

				let Ok(content) = fs::read_to_string(&path) else {
					continue; // Skip files that can't be read
				};
				if let Ok(file) = parse_file(&content) {
					let reexport_module = detect_re_export_pattern(&file);

					// Collect public items using the formatter
					let items = collect_public_items(&file, reexport_module.as_deref(), |item| {
						formatter.matches_item(item)
					});

					// Generate re-export tokens for each item
					for item_name in items {
						// Check exclusions (both qualified and unqualified)
						let full_name = format!("{file_stem}::{item_name}");
						if input.exclusions.contains(&full_name)
							|| input.exclusions.contains(&item_name)
						{
							continue;
						}

						let item_ident = Ident::new(&item_name, proc_macro2::Span::call_site());
						let module_name = Ident::new(file_stem, proc_macro2::Span::call_site());

						// Determine alias (functions support full qualified names)
						let alias =
							input.aliases.get(&full_name).or_else(|| input.aliases.get(&item_name));

						let tokens = formatter.format_item(&module_name, &item_ident, alias);
						re_exports.push(tokens);
					}
				}
			}
		}
	} else {
		// Generate a compile error instead of panicking
		let path_str = input.path.value();
		return vec![quote! {
			compile_error!(concat!(
				"Failed to read directory for re-export generation: '",
				#path_str,
				"'. Please ensure the path exists and is accessible."
			));
		}];
	}

	// Sort re_exports for deterministic output
	re_exports.sort_by_key(|tokens| tokens.to_string());
	re_exports
}

/// Unified implementation for generating re-exports using a formatter and base path.
pub fn generate_re_exports_worker(
	input: &ReExportInput,
	formatter: &dyn ReExportFormatter,
) -> TokenStream {
	let base_path = parse_base_path_from_input(input);
	let re_exports = scan_directory_and_collect(input, formatter);
	formatter.format_output(re_exports, &base_path)
}

/// Parses the base crate path from the input directory path.
///
/// For example, "src/classes" becomes "crate::classes".
/// This allows the macros to be flexible and work with any directory structure.
fn parse_base_path_from_input(input: &ReExportInput) -> syn::Path {
	let path_str = input.path.value();

	// Extract the module path from the directory path
	// e.g., "src/classes" -> "crate::classes"
	// or "fp-library/src/types" -> "crate::types"
	let parts: Vec<&str> =
		path_str.split('/').filter(|p| *p != re_export::SRC_DIR && !p.is_empty()).collect();

	// Build the path starting with crate::
	let mut segments = syn::punctuated::Punctuated::new();
	segments.push(syn::PathSegment {
		ident: Ident::new(re_export::CRATE_KEYWORD, proc_macro2::Span::call_site()),
		arguments: syn::PathArguments::None,
	});

	for part in parts {
		segments.push(syn::PathSegment {
			ident: Ident::new(part, proc_macro2::Span::call_site()),
			arguments: syn::PathArguments::None,
		});
	}

	syn::Path {
		leading_colon: None,
		segments,
	}
}
