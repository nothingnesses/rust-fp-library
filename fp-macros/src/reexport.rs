use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Item, LitStr, Result, Token, Visibility, braced, parse_file};

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

pub fn generate_reexports_impl(input: ReexportInput) -> TokenStream {
	let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
	let base_path = Path::new(&manifest_dir).join(input.path.value());

	let mut reexports = Vec::new();

	if let Ok(entries) = fs::read_dir(&base_path) {
		for entry in entries.flatten() {
			let path = entry.path();
			if path.extension().and_then(|s| s.to_str()) == Some("rs") {
				let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap();
				if file_stem == "mod" {
					continue;
				}

				let content = fs::read_to_string(&path).expect("Failed to read file");
				if let Ok(file) = parse_file(&content) {
					for item in file.items {
						if let Item::Fn(func) = item
							&& let Visibility::Public(_) = func.vis
						{
							let fn_name = func.sig.ident;
							let module_name = Ident::new(file_stem, fn_name.span());
							let full_name = format!("{}::{}", file_stem, fn_name);

							let use_stmt = if let Some(alias) = input
								.aliases
								.get(&full_name)
								.or_else(|| input.aliases.get(&fn_name.to_string()))
							{
								quote! {
									#module_name::#fn_name as #alias
								}
							} else {
								quote! {
									#module_name::#fn_name
								}
							};
							reexports.push(use_stmt);
						}
					}
				}
			}
		}
	} else {
		panic!("Failed to read directory: {:?}", base_path);
	}

	// Sort reexports for deterministic output
	reexports.sort_by_key(|tokens| tokens.to_string());

	quote! {
		pub use crate::classes::{
			#(#reexports),*
		};
	}
}
