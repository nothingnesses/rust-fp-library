use crate::function_utils::{LogicalParam, get_logical_params, load_config};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Error;

pub fn doc_params_impl(
	attr: TokenStream,
	item_tokens: TokenStream,
) -> TokenStream {
	crate::doc_utils::generate_doc_comments(attr, item_tokens, |generic_item| {
		let config = load_config();

		let sig = match generic_item.sig() {
			Some(s) => s,
			None => {
				return Err(Error::new(
					proc_macro2::Span::call_site(),
					"doc_params can only be used on functions",
				));
			}
		};

		let logical_params = get_logical_params(sig, &config);

		Ok(logical_params
			.into_iter()
			.map(|param| match param {
				LogicalParam::Explicit(pat) => {
					let s = pat.to_token_stream().to_string();
					s.replace(" , ", ", ")
				}
				LogicalParam::Implicit(_) => "_".to_string(),
			})
			.collect())
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::doc_utils::get_doc;
	use quote::quote;
	use syn::ItemFn;

	#[test]
	fn test_doc_params_basic() {
		let attr = quote! { "Arg 1", "Arg 2" };
		let item = quote! {
			fn foo(a: i32, b: String) {}
		};

		let output = doc_params_impl(attr, item);
		let output_fn: ItemFn = syn::parse2(output).unwrap();

		assert_eq!(output_fn.attrs.len(), 2);
		assert_eq!(get_doc(&output_fn.attrs[0]), "* `a`: Arg 1");
		assert_eq!(get_doc(&output_fn.attrs[1]), "* `b`: Arg 2");
	}

	#[test]
	fn test_doc_params_trait() {
		let attr = quote! { "Arg 1" };
		let item = quote! {
			fn foo(a: i32);
		};

		let output = doc_params_impl(attr, item);
		let output_fn: syn::TraitItemFn = syn::parse2(output).unwrap();

		assert_eq!(output_fn.attrs.len(), 1);
		assert_eq!(get_doc(&output_fn.attrs[0]), "* `a`: Arg 1");
	}

	#[test]
	fn test_doc_params_with_overrides() {
		let attr = quote! { ("custom_a", "Arg 1"), "Arg 2" };
		let item = quote! {
			fn foo(a: i32, b: String) {}
		};

		let output = doc_params_impl(attr, item);
		let output_fn: ItemFn = syn::parse2(output).unwrap();

		assert_eq!(output_fn.attrs.len(), 2);
		assert_eq!(get_doc(&output_fn.attrs[0]), "* `custom_a`: Arg 1");
		assert_eq!(get_doc(&output_fn.attrs[1]), "* `b`: Arg 2");
	}

	#[test]
	fn test_doc_params_curried() {
		let attr = quote! { "Arg 1", "Curried Arg" };
		let item = quote! {
			fn foo(a: i32) -> impl Fn(i32) -> i32 { todo!() }
		};

		let output = doc_params_impl(attr, item);
		let output_fn: ItemFn = syn::parse2(output).unwrap();

		assert_eq!(output_fn.attrs.len(), 2);
		assert_eq!(get_doc(&output_fn.attrs[0]), "* `a`: Arg 1");
		assert_eq!(get_doc(&output_fn.attrs[1]), "* `_`: Curried Arg");
	}

	#[test]
	fn test_doc_params_mismatch() {
		let attr = quote! { "Too few" };
		let item = quote! {
			fn foo(a: i32, b: i32) {}
		};

		let output = doc_params_impl(attr, item);
		let error = output.to_string();
		assert!(error.contains("Expected 2 description arguments, found 1."));
	}

	#[test]
	fn test_doc_params_skips_self() {
		let attr = quote! { "Arg 1" };
		let item = quote! {
			fn foo(&self, a: i32) {}
		};

		let output = doc_params_impl(attr, item);
		let output_fn: ItemFn = syn::parse2(output).unwrap();

		assert_eq!(output_fn.attrs.len(), 1);
		assert_eq!(get_doc(&output_fn.attrs[0]), "* `a`: Arg 1");
	}
}
