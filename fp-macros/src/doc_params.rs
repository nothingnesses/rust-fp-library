use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::collections::{HashMap, HashSet};
use syn::{Error, GenericParam, Type, TypeParamBound, WherePredicate, spanned::Spanned};

use crate::doc_utils::{DocArg, GenericArgs, GenericItem, insert_doc_comment};
use crate::function_utils::{LogicalParam, get_fn_type, get_logical_params, load_config};
use crate::hm_ast::HMType;

pub fn doc_params_impl(
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

	let config = load_config();

	let mut fn_bounds: HashMap<String, HMType> = HashMap::new();
	let mut generic_names = HashSet::new();

	let sig = match generic_item.sig() {
		Some(s) => s,
		None => {
			return Error::new(attr.span(), "doc_params can only be used on functions")
				.to_compile_error();
		}
	};

	for param in &sig.generics.params {
		if let GenericParam::Type(type_param) = param {
			generic_names.insert(type_param.ident.to_string());
		}
	}

	for param in &sig.generics.params {
		if let GenericParam::Type(type_param) = param {
			let name = type_param.ident.to_string();
			for bound in &type_param.bounds {
				if let TypeParamBound::Trait(trait_bound) = bound
					&& let Some(sig_ty) =
						get_fn_type(trait_bound, &fn_bounds, &generic_names, &config)
				{
					fn_bounds.insert(name.clone(), sig_ty);
				}
			}
		}
	}

	if let Some(where_clause) = &sig.generics.where_clause {
		for predicate in &where_clause.predicates {
			if let WherePredicate::Type(predicate_type) = predicate
				&& let Type::Path(type_path) = &predicate_type.bounded_ty
				&& type_path.path.segments.len() == 1
			{
				let name = type_path.path.segments[0].ident.to_string();
				for bound in &predicate_type.bounds {
					if let TypeParamBound::Trait(trait_bound) = bound
						&& let Some(sig_ty) =
							get_fn_type(trait_bound, &fn_bounds, &generic_names, &config)
					{
						fn_bounds.insert(name.clone(), sig_ty);
					}
				}
			}
		}
	}

	let logical_params = get_logical_params(sig, &fn_bounds, &generic_names, &config);
	let entries: Vec<_> = args.entries.into_iter().collect();

	if logical_params.len() != entries.len() {
		return Error::new(
			attr.span(),
			format!(
				"Expected {} description arguments, found {}.",
				logical_params.len(),
				entries.len()
			),
		)
		.to_compile_error();
	}

	for (param, entry) in logical_params.iter().zip(entries) {
		let (name, desc) = match entry {
			DocArg::Override(n, d) => (n.value(), d.value()),
			DocArg::Desc(d) => {
				let name = match param {
					LogicalParam::Explicit(pat) => {
						let s = pat.to_token_stream().to_string();
						s.replace(" , ", ", ")
					}
					LogicalParam::Implicit(_) => "_".to_string(),
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
mod tests {
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
		panic!("Not a doc comment: {:?}", attr);
	}

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
