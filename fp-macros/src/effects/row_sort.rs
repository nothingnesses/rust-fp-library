//! Shared structural-sort helper for first-order and scoped effect rows.
//!
//! Both [`effects!`](crate::effects) and
//! [`scoped_effects!`](crate::scoped_effects) accept a comma-separated
//! list of types and emit a right-nested brand-level row in canonical
//! order.
//! The canonical order is the lexical sort of a structural key derived
//! from each parsed [`syn::Type`]. The key normalises syntax that proc
//! macros can observe directly, such as path segments, generic
//! arguments, references, tuples, groups, and parenthesised types. It
//! does not claim Rust semantic identity: aliases, imports, and name
//! resolution remain outside proc-macro visibility.
//!
//! Factoring the sort here means future canonicalisation refinements
//! land in one place rather than being duplicated across rows and
//! handler-list macros.

use {
	proc_macro2::TokenStream,
	quote::quote,
	std::collections::HashMap,
	syn::{
		AngleBracketedGenericArguments,
		GenericArgument,
		PathArguments,
		ReturnType,
		Token,
		Type,
		parse::Parser,
		punctuated::Punctuated,
		spanned::Spanned,
	},
};

/// Parses a comma-separated list of types from `input` and returns
/// them sorted by their structural row key.
///
/// An empty input produces an empty `Vec` (the caller decides what
/// to emit for the zero-length case).
pub(crate) fn parse_and_sort_types(input: TokenStream) -> syn::Result<Vec<Type>> {
	let parser = Punctuated::<Type, Token![,]>::parse_terminated;
	let parsed = parser.parse2(input)?;
	sort_types_unique(parsed)
}

/// Returns `types` sorted by their structural row key, rejecting duplicates.
pub(crate) fn sort_types_unique(types: impl IntoIterator<Item = Type>) -> syn::Result<Vec<Type>> {
	Ok(sort_type_keyed(types.into_iter().map(|ty| (ty, ())), "row entry")?
		.into_iter()
		.map(|(ty, ())| ty)
		.collect())
}

/// Sorts values keyed by a brand type and rejects duplicate structural keys.
pub(crate) fn sort_type_keyed<T>(
	items: impl IntoIterator<Item = (Type, T)>,
	duplicate_label: &str,
) -> syn::Result<Vec<(Type, T)>> {
	let mut seen = HashMap::new();
	let mut typed = Vec::new();
	for (ty, value) in items {
		let key = type_sort_key(&ty);
		if seen.insert(key.clone(), ty.span()).is_some() {
			return Err(syn::Error::new(
				ty.span(),
				format!(
					"duplicate {duplicate_label} `{}` after row-key normalization",
					quote!(#ty)
				),
			));
		}
		typed.push((key, ty, value));
	}
	typed.sort_by(|a, b| a.0.cmp(&b.0));
	Ok(typed.into_iter().map(|(_, ty, value)| (ty, value)).collect())
}

/// Returns the structural sort key used by row and handler macros.
pub(crate) fn type_sort_key(ty: &Type) -> String {
	match ty {
		Type::Path(ty) => {
			let mut key = String::from("path:");
			if let Some(qself) = &ty.qself {
				key.push('<');
				key.push_str(&type_sort_key(&qself.ty));
				key.push_str(" as ");
				key.push_str(&qself.position.to_string());
				key.push('>');
			}
			if ty.path.leading_colon.is_some() {
				key.push_str("::");
			}
			key.push_str(
				&ty.path
					.segments
					.iter()
					.map(|segment| {
						format!("{}{}", segment.ident, path_arguments_key(&segment.arguments))
					})
					.collect::<Vec<_>>()
					.join("::"),
			);
			key
		}
		Type::Reference(ty) => {
			let lifetime = ty.lifetime.as_ref().map(|lt| lt.ident.to_string()).unwrap_or_default();
			let mutability = if ty.mutability.is_some() { "mut" } else { "shared" };
			format!("ref:{lifetime}:{mutability}:{}", type_sort_key(&ty.elem))
		}
		Type::Tuple(ty) =>
			format!("tuple:({})", ty.elems.iter().map(type_sort_key).collect::<Vec<_>>().join(",")),
		Type::Paren(ty) => type_sort_key(&ty.elem),
		Type::Group(ty) => type_sort_key(&ty.elem),
		Type::Slice(ty) => format!("slice:[{}]", type_sort_key(&ty.elem)),
		Type::Array(ty) => {
			let len = &ty.len;
			format!("array:[{};{}]", type_sort_key(&ty.elem), quote!(#len))
		}
		Type::Ptr(ty) => {
			let mutability = if ty.mutability.is_some() { "mut" } else { "const" };
			format!("ptr:{mutability}:{}", type_sort_key(&ty.elem))
		}
		Type::Infer(_) => "infer:_".to_owned(),
		Type::Never(_) => "never:!".to_owned(),
		other => format!("tokens:{}", quote!(#other)),
	}
}

fn path_arguments_key(args: &PathArguments) -> String {
	match args {
		PathArguments::None => String::new(),
		PathArguments::AngleBracketed(args) => angle_bracketed_arguments_key(args),
		PathArguments::Parenthesized(args) => {
			let inputs = args.inputs.iter().map(type_sort_key).collect::<Vec<_>>().join(",");
			let output = match &args.output {
				ReturnType::Default => String::new(),
				ReturnType::Type(_, ty) => format!("->{}", type_sort_key(ty)),
			};
			format!("({inputs}){output}")
		}
	}
}

fn angle_bracketed_arguments_key(args: &AngleBracketedGenericArguments) -> String {
	format!("<{}>", args.args.iter().map(generic_argument_key).collect::<Vec<_>>().join(","))
}

fn generic_argument_key(arg: &GenericArgument) -> String {
	match arg {
		GenericArgument::Lifetime(lt) => format!("lifetime:{}", lt.ident),
		GenericArgument::Type(ty) => format!("type:{}", type_sort_key(ty)),
		GenericArgument::Const(expr) => format!("const:{}", quote!(#expr)),
		GenericArgument::AssocType(binding) => {
			let generics =
				binding.generics.as_ref().map(angle_bracketed_arguments_key).unwrap_or_default();
			format!("assoc_type:{}{}={}", binding.ident, generics, type_sort_key(&binding.ty))
		}
		GenericArgument::AssocConst(binding) => {
			let generics =
				binding.generics.as_ref().map(angle_bracketed_arguments_key).unwrap_or_default();
			let value = &binding.value;
			format!("assoc_const:{}{}={}", binding.ident, generics, quote!(#value))
		}
		GenericArgument::Constraint(constraint) => {
			let generics =
				constraint.generics.as_ref().map(angle_bracketed_arguments_key).unwrap_or_default();
			let bounds = &constraint.bounds;
			format!("constraint:{}{}:{}", constraint.ident, generics, quote!(#bounds))
		}
		other => format!("tokens:{}", quote!(#other)),
	}
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "Tests use panicking operations for brevity and clarity")]
mod tests {
	use super::*;

	#[test]
	fn sorts_by_structural_key() {
		let input: TokenStream = quote! { OptionBrand, IdentityBrand };
		let sorted = parse_and_sort_types(input).expect("parse failed");
		assert_eq!(quote!(#(#sorted),*).to_string(), "IdentityBrand , OptionBrand");
	}

	#[test]
	fn empty_input_yields_empty_vec() {
		let input: TokenStream = quote! {};
		let sorted = parse_and_sort_types(input).expect("parse failed");
		assert!(sorted.is_empty());
	}

	#[test]
	fn whitespace_does_not_change_structural_key() {
		let a: TokenStream = quote! { Reader<Env> };
		let b: TokenStream = quote! { Reader < Env > };
		let sorted_a = parse_and_sort_types(a).expect("parse failed");
		let sorted_b = parse_and_sort_types(b).expect("parse failed");
		assert_eq!(quote!(#(#sorted_a),*).to_string(), quote!(#(#sorted_b),*).to_string());
	}

	#[test]
	fn generic_effect_types_include_arguments_in_key() {
		let input: TokenStream = quote! { Reader<Env>, Reader<Other> };
		let sorted = parse_and_sort_types(input).expect("parse failed");
		assert_eq!(quote!(#(#sorted),*).to_string(), "Reader < Env > , Reader < Other >");
	}

	#[test]
	fn duplicate_structural_keys_are_rejected() {
		let err =
			parse_and_sort_types(quote! { Reader<Env>, (Reader<Env>) }).expect_err("duplicate");
		assert!(err.to_string().contains("duplicate row entry"));
	}

	#[test]
	fn parenthesised_and_grouped_types_share_key_with_inner_type() {
		let plain: Type = syn::parse_quote!(Reader<Env>);
		let parenthesised: Type = syn::parse_quote!((Reader<Env>));
		assert_eq!(type_sort_key(&plain), type_sort_key(&parenthesised));
	}

	#[test]
	fn references_and_tuples_have_structural_keys() {
		let reference: Type = syn::parse_quote!(&'a mut Reader<Env>);
		let tuple: Type = syn::parse_quote!((Reader<Env>, Writer<Log>));
		assert_eq!(type_sort_key(&reference), "ref:a:mut:path:Reader<type:path:Env>");
		assert_eq!(
			type_sort_key(&tuple),
			"tuple:(path:Reader<type:path:Env>,path:Writer<type:path:Log>)"
		);
	}
}
