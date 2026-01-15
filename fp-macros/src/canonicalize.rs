//! Canonicalization logic for Kind trait names.
//!
//! This module provides functionality to convert type bounds and signatures
//! into a canonical string representation that is deterministic and unique
//! for semantically equivalent signatures.

use crate::parse::TypeInput;
use quote::quote;
use std::collections::BTreeMap;
use syn::{
	GenericArgument, Lifetime, PathArguments, ReturnType, Token, Type, TypeParamBound,
	punctuated::Punctuated,
};

/// Handles the canonicalization of type bounds and signatures.
pub struct Canonicalizer {
	lifetime_map: BTreeMap<String, usize>,
}

impl Canonicalizer {
	/// Creates a new `Canonicalizer` with a mapping of lifetime names to indices.
	pub fn new(
		lifetimes: &Punctuated<Lifetime, Token![,]>,
		_types: &Punctuated<TypeInput, Token![,]>,
	) -> Self {
		let mut lifetime_map = BTreeMap::new();
		for (i, lt) in lifetimes.iter().enumerate() {
			lifetime_map.insert(lt.ident.to_string(), i);
		}

		Self { lifetime_map }
	}

	/// Canonicalizes a single type bound.
	///
	/// - Lifetimes are replaced by their index (e.g., `l0`).
	/// - Traits are represented by their full path with generic arguments.
	pub fn canonicalize_bound(
		&self,
		bound: &TypeParamBound,
	) -> String {
		match bound {
			TypeParamBound::Lifetime(lt) => {
				let idx = self.lifetime_map.get(&lt.ident.to_string()).expect("Unknown lifetime");
				format!("l{}", idx)
			}
			TypeParamBound::Trait(tr) => {
				// Full path with generic arguments
				let path = tr
					.path
					.segments
					.iter()
					.map(|seg| {
						let ident = seg.ident.to_string();
						match &seg.arguments {
							PathArguments::None => ident,
							PathArguments::AngleBracketed(args) => {
								let args_str = args
									.args
									.iter()
									.map(|a| self.canonicalize_generic_arg(a))
									.collect::<Vec<_>>()
									.join(",");
								format!("{}<{}>", ident, args_str)
							}
							PathArguments::Parenthesized(args) => {
								// Fn trait bounds: Fn(A) -> B
								let inputs = args
									.inputs
									.iter()
									.map(|t| self.canonicalize_type(t))
									.collect::<Vec<_>>()
									.join(",");
								let output = match &args.output {
									ReturnType::Default => "()".to_string(),
									ReturnType::Type(_, ty) => self.canonicalize_type(ty),
								};
								format!("{}({})->{}", ident, inputs, output)
							}
						}
					})
					.collect::<Vec<_>>()
					.join("::");
				format!("t{}", path)
			}
			_ => panic!("Unsupported bound type"),
		}
	}

	/// Canonicalizes a list of bounds, sorting them to ensure determinism.
	pub fn canonicalize_bounds(
		&self,
		bounds: &Punctuated<TypeParamBound, Token![+]>,
	) -> String {
		let mut parts: Vec<String> = bounds.iter().map(|b| self.canonicalize_bound(b)).collect();
		parts.sort(); // Ensure deterministic order
		parts.join("")
	}

	fn canonicalize_generic_arg(
		&self,
		arg: &GenericArgument,
	) -> String {
		match arg {
			GenericArgument::Type(ty) => self.canonicalize_type(ty),
			GenericArgument::Lifetime(lt) => {
				if let Some(idx) = self.lifetime_map.get(&lt.ident.to_string()) {
					format!("l{}", idx)
				} else {
					lt.ident.to_string()
				}
			}
			GenericArgument::AssocType(assoc) => {
				format!("{}={}", assoc.ident, self.canonicalize_type(&assoc.ty))
			}
			GenericArgument::Const(expr) => quote!(#expr).to_string().replace(" ", ""),
			_ => panic!("Unsupported generic argument"),
		}
	}

	fn canonicalize_type(
		&self,
		ty: &Type,
	) -> String {
		match ty {
			Type::Path(type_path) => {
				let path = type_path
					.path
					.segments
					.iter()
					.map(|seg| {
						let ident = seg.ident.to_string();
						match &seg.arguments {
							PathArguments::None => ident,
							PathArguments::AngleBracketed(args) => {
								let args_str = args
									.args
									.iter()
									.map(|a| self.canonicalize_generic_arg(a))
									.collect::<Vec<_>>()
									.join(",");
								format!("{}<{}>", ident, args_str)
							}
							PathArguments::Parenthesized(args) => {
								let inputs = args
									.inputs
									.iter()
									.map(|t| self.canonicalize_type(t))
									.collect::<Vec<_>>()
									.join(",");
								let output = match &args.output {
									ReturnType::Default => "()".to_string(),
									ReturnType::Type(_, ty) => self.canonicalize_type(ty),
								};
								format!("{}({})->{}", ident, inputs, output)
							}
						}
					})
					.collect::<Vec<_>>()
					.join("::");
				path
			}
			Type::Reference(type_ref) => {
				let lt = if let Some(lt) = &type_ref.lifetime {
					if let Some(idx) = self.lifetime_map.get(&lt.ident.to_string()) {
						format!("l{} ", idx)
					} else {
						format!("{} ", lt.ident)
					}
				} else {
					"".to_string()
				};
				let mutability = if type_ref.mutability.is_some() { "mut " } else { "" };
				format!("&{}{}{}", lt, mutability, self.canonicalize_type(&type_ref.elem))
			}
			Type::Tuple(tuple) => {
				let elems = tuple
					.elems
					.iter()
					.map(|t| self.canonicalize_type(t))
					.collect::<Vec<_>>()
					.join(",");
				format!("({})", elems)
			}
			Type::Slice(slice) => {
				format!("[{}]", self.canonicalize_type(&slice.elem))
			}
			Type::Array(array) => {
				let len = quote!(#array.len).to_string().replace(" ", "");
				format!("[{};{}]", self.canonicalize_type(&array.elem), len)
			}
			Type::Never(_) => "!".to_string(),
			Type::Infer(_) => "_".to_string(),
			_ => panic!("Unsupported type in canonicalization"),
		}
	}
}
