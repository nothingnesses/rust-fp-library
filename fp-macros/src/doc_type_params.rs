use proc_macro2::TokenStream;
use syn::GenericParam;

pub fn doc_type_params_impl(
	attr: TokenStream,
	item_tokens: TokenStream,
) -> TokenStream {
	crate::doc_utils::generate_doc_comments(attr, item_tokens, |generic_item| {
		let generics = generic_item.generics();
		Ok(generics
			.params
			.iter()
			.map(|param| match param {
				GenericParam::Type(t) => t.ident.to_string(),
				GenericParam::Lifetime(l) => l.lifetime.to_string(),
				GenericParam::Const(c) => c.ident.to_string(),
			})
			.collect())
	})
}

#[cfg(test)]
mod doc_type_params_tests {
	use super::*;
	use crate::doc_utils::get_doc;
	use quote::quote;
	use syn::ItemFn;

	#[test]
	fn test_doc_type_params_basic() {
		let attr = quote! { "Type A", "Type B" };
		let item = quote! {
			fn foo<A, B>(a: A, b: B) {}
		};

		let output = doc_type_params_impl(attr, item);
		let output_fn: ItemFn = syn::parse2(output).unwrap();

		assert_eq!(output_fn.attrs.len(), 2);
		assert_eq!(get_doc(&output_fn.attrs[0]), "* `A`: Type A");
		assert_eq!(get_doc(&output_fn.attrs[1]), "* `B`: Type B");
	}

	#[test]
	fn test_doc_type_params_with_overrides() {
		let attr = quote! { ("CustomA", "Type A"), "Type B" };
		let item = quote! {
			fn foo<A, B>(a: A, b: B) {}
		};

		let output = doc_type_params_impl(attr, item);
		let output_fn: ItemFn = syn::parse2(output).unwrap();

		assert_eq!(output_fn.attrs.len(), 2);
		assert_eq!(get_doc(&output_fn.attrs[0]), "* `CustomA`: Type A");
		assert_eq!(get_doc(&output_fn.attrs[1]), "* `B`: Type B");
	}

	#[test]
	fn test_doc_type_params_mixed() {
		let attr = quote! { "Lifetime a", "Type T", "Const N" };
		let item = quote! {
			fn foo<'a, T, const N: usize>() {}
		};

		let output = doc_type_params_impl(attr, item);
		let output_fn: ItemFn = syn::parse2(output).unwrap();

		assert_eq!(output_fn.attrs.len(), 3);
		assert_eq!(get_doc(&output_fn.attrs[0]), "* `'a`: Lifetime a");
		assert_eq!(get_doc(&output_fn.attrs[1]), "* `T`: Type T");
		assert_eq!(get_doc(&output_fn.attrs[2]), "* `N`: Const N");
	}

	#[test]
	fn test_doc_type_params_mismatch() {
		let attr = quote! { "Too few" };
		let item = quote! {
			fn foo<A, B>() {}
		};

		let output = doc_type_params_impl(attr, item);
		let error = output.to_string();
		assert!(error.contains("Expected 2 description arguments, found 1."));
	}
}
