use crate::{
	core::{Result, constants::attributes::DOCUMENT_TYPE_PARAMETERS},
	support::{parsing, syntax::generate_doc_comments},
};
use proc_macro2::TokenStream;
use syn::{GenericParam, spanned::Spanned};

pub fn document_type_parameters_worker(
	attr: TokenStream,
	item_tokens: TokenStream,
) -> Result<TokenStream> {
	generate_doc_comments(attr, item_tokens, |generic_item| {
		let generics = generic_item.generics();

		// Error if there are no type parameters
		let _count = parsing::parse_has_documentable_items(
			generics.params.len(),
			generics.span(),
			DOCUMENT_TYPE_PARAMETERS,
			"items with no type parameters",
		)?;

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
	use crate::support::syntax::get_doc;
	use quote::quote;
	use syn::ItemFn;

	#[test]
	fn test_doc_type_params_basic() {
		let attr = quote! { "Type A", "Type B" };
		let item = quote! {
			fn foo<A, B>(a: A, b: B) {}
		};

		let output = document_type_parameters_worker(attr, item).unwrap();
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

		let output = document_type_parameters_worker(attr, item).unwrap();
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

		let output = document_type_parameters_worker(attr, item).unwrap();
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

		let output = document_type_parameters_worker(attr, item).unwrap_err();
		let error = output.to_string();
		assert!(error.contains("Expected 2 description arguments"));
		assert!(error.contains("found 1"));
	}

	#[test]
	fn test_doc_type_params_no_type_parameters() {
		let attr = quote! {};
		let item = quote! {
			fn foo() {}
		};

		let output = document_type_parameters_worker(attr, item).unwrap_err();
		let error = output.to_string();
		assert!(
			error.contains(
				"Cannot use #[document_type_parameters] on items with no type parameters"
			)
		);
	}

	#[test]
	fn test_doc_type_params_on_impl() {
		let attr = quote! { "The base functor.", "The result type." };
		let item = quote! {
			impl<F, A> MyType<F, A> {
				fn foo(&self) {}
			}
		};

		let output = document_type_parameters_worker(attr, item).unwrap();
		let output_impl: syn::ItemImpl = syn::parse2(output).unwrap();

		assert_eq!(output_impl.attrs.len(), 2);
		assert_eq!(get_doc(&output_impl.attrs[0]), "* `F`: The base functor.");
		assert_eq!(get_doc(&output_impl.attrs[1]), "* `A`: The result type.");
	}

	#[test]
	fn test_doc_type_params_on_method_only_shows_method_params() {
		let attr = quote! { "The new result type." };
		let item = quote! {
			fn bind<B>(self) -> Free<F, B> {}
		};

		let output = document_type_parameters_worker(attr, item).unwrap();
		let output_fn: ItemFn = syn::parse2(output).unwrap();

		// Only method-level type parameter B should be documented
		assert_eq!(output_fn.attrs.len(), 1);
		assert_eq!(get_doc(&output_fn.attrs[0]), "* `B`: The new result type.");
	}
}
