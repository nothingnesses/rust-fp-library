use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, GenericParam, spanned::Spanned};

use crate::doc_utils::{DocArg, GenericArgs, GenericItem, insert_doc_comment};

pub fn doc_type_params_impl(
	attr: TokenStream,
	item_tokens: TokenStream,
) -> TokenStream {
	let mut generic_item = match GenericItem::parse(item_tokens) {
		Ok(i) => i,
		Err(e) => return e.to_compile_error(),
	};

	let args = match syn::parse2::<GenericArgs>(attr.clone()) {
		Ok(a) => a,
		Err(e) => return e.to_compile_error(),
	};

	let (params_count, entries) = {
		let generics = generic_item.generics();
		(generics.params.len(), args.entries.into_iter().collect::<Vec<_>>())
	};

	if params_count != entries.len() {
		return Error::new(
			attr.span(),
			format!("Expected {} description arguments, found {}.", params_count, entries.len()),
		)
		.to_compile_error();
	}

	for i in 0..params_count {
		let entry = &entries[i];
		let (name, desc) = match entry {
			DocArg::Override(n, d) => (n.value(), d.value()),
			DocArg::Desc(d) => {
				let name = {
					let param = &generic_item.generics().params[i];
					match param {
						GenericParam::Type(t) => t.ident.to_string(),
						GenericParam::Lifetime(l) => l.lifetime.to_string(),
						GenericParam::Const(c) => c.ident.to_string(),
					}
				};
				(name, d.value())
			}
		};

		let doc_comment = format!("* `{}`: {}", name, desc);
		insert_doc_comment(generic_item.attrs(), doc_comment, proc_macro2::Span::call_site());
	}

	quote! {
		#generic_item
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
