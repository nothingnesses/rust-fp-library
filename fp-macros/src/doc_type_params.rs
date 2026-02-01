use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, GenericParam, ItemFn, LitStr, Token, spanned::Spanned};

use crate::doc_utils::insert_doc_comment;

struct Args {
	descriptions: Punctuated<LitStr, Token![,]>,
}

impl Parse for Args {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(Args { descriptions: Punctuated::parse_terminated(input)? })
	}
}

pub fn doc_type_params_impl(
	attr: TokenStream,
	item: TokenStream,
) -> TokenStream {
	let mut input_fn = match syn::parse2::<ItemFn>(item) {
		Ok(f) => f,
		Err(e) => return e.to_compile_error(),
	};

	let args = match syn::parse2::<Args>(attr.clone()) {
		Ok(a) => a,
		Err(e) => return e.to_compile_error(),
	};

	let params = &input_fn.sig.generics.params;
	let descriptions: Vec<_> = args.descriptions.into_iter().collect();

	if params.len() != descriptions.len() {
		return Error::new(
			attr.span(),
			format!(
				"Expected {} description arguments, found {}.",
				params.len(),
				descriptions.len()
			),
		)
		.to_compile_error();
	}

	for (param, desc) in params.iter().zip(descriptions) {
		let name = match param {
			GenericParam::Type(t) => t.ident.to_string(),
			GenericParam::Lifetime(l) => l.lifetime.to_string(),
			GenericParam::Const(c) => c.ident.to_string(),
		};

		let doc_comment = format!("* `{}`: {}", name, desc.value());
		insert_doc_comment(&mut input_fn.attrs, doc_comment, proc_macro2::Span::call_site());
	}

	quote! {
		#input_fn
	}
}

#[cfg(test)]
mod doc_type_params_tests {
	use super::*;
	use syn::ItemFn;

	fn get_doc(attr: &syn::Attribute) -> String {
		if let syn::Meta::NameValue(nv) = &attr.meta {
			if let syn::Expr::Lit(lit) = &nv.value {
				if let syn::Lit::Str(s) = &lit.lit {
					return s.value();
				}
			}
		}
		panic!("Not a doc comment");
	}

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
