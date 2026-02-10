use crate::{
	core::{Result, config::get_config, constants::attributes::DOCUMENT_PARAMETERS},
	support::{LogicalParam, get_logical_params},
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

pub fn document_parameters_worker(
	attr: TokenStream,
	item_tokens: TokenStream,
) -> Result<TokenStream> {
	// Try parsing as impl block first
	if let Ok(item_impl) = syn::parse2::<syn::ItemImpl>(item_tokens.clone()) {
		// For impl blocks, just pass through - they will be handled during module processing
		// The attribute will be consumed during context extraction and generation phases
		return Ok(quote!(#item_impl));
	}

	// Otherwise, process as a function with generate_doc_comments
	crate::support::syntax::generate_doc_comments(attr, item_tokens, |generic_item| {
		let config = get_config();

		let sig = generic_item.sig().ok_or_else(|| {
			syn::Error::new(
				proc_macro2::Span::call_site(),
				format!("{DOCUMENT_PARAMETERS} can only be used on functions or impl blocks"),
			)
		})?;

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
	use crate::support::syntax::get_doc;
	use quote::quote;
	use syn::ItemFn;

	#[test]
	fn test_doc_params_basic() {
		let attr = quote! { "Arg 1", "Arg 2" };
		let item = quote! {
			fn foo(a: i32, b: String) {}
		};

		let output = document_parameters_worker(attr, item).unwrap();
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

		let output = document_parameters_worker(attr, item).unwrap();
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

		let output = document_parameters_worker(attr, item).unwrap();
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

		let output = document_parameters_worker(attr, item).unwrap();
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

		let output = document_parameters_worker(attr, item).unwrap_err();
		let error = output.to_string();
		assert!(error.contains("Expected 2 description arguments, found 1."));
	}

	#[test]
	fn test_doc_params_skips_self() {
		let attr = quote! { "Arg 1" };
		let item = quote! {
			fn foo(&self, a: i32) {}
		};

		let output = document_parameters_worker(attr, item).unwrap();
		let output_fn: ItemFn = syn::parse2(output).unwrap();

		assert_eq!(output_fn.attrs.len(), 1);
		assert_eq!(get_doc(&output_fn.attrs[0]), "* `a`: Arg 1");
	}

	#[test]
	fn test_doc_params_impl_block_passthrough() {
		// When applied to an impl block, the attribute should be passed through
		// without modification, to be processed later by document_module
		let attr = quote! { "The receiver parameter" };
		let item = quote! {
			impl<A> MyType<A> {
				fn foo(&self) -> usize { 0 }
			}
		};

		let output = document_parameters_worker(attr.clone(), item.clone()).unwrap();

		// The output should be the impl block unchanged (attribute will be processed later)
		let output_impl: syn::ItemImpl = syn::parse2(output).unwrap();
		let input_impl: syn::ItemImpl = syn::parse2(item).unwrap();

		// Both should have the same structure (attribute processing happens in document_module)
		assert_eq!(output_impl.items.len(), input_impl.items.len());
	}

	#[test]
	fn test_doc_params_impl_block_no_methods() {
		// This error would be caught during context extraction in document_module,
		// not in the attribute processor itself
		let attr = quote! { "The receiver" };
		let item = quote! {
			impl<A> MyType<A> {
				const VALUE: i32 = 42;
			}
		};

		// Should pass through successfully - validation happens in document_module
		let output = document_parameters_worker(attr, item).unwrap();
		let _: syn::ItemImpl = syn::parse2(output).unwrap();
	}

	#[test]
	fn test_doc_params_standalone_function_no_params() {
		let attr = quote! {};
		let item = quote! {
			fn foo() {}
		};

		// Should error - function has no parameters
		let result = document_parameters_worker(attr, item);
		assert!(result.is_err());
	}
}
