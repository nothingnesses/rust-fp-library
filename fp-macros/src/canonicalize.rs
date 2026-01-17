//! Canonicalization logic for Kind trait names.
//!
//! This module provides functionality to convert type bounds and signatures
//! into a canonical string representation that is deterministic and unique
//! for semantically equivalent signatures.
//!
//! It handles:
//! - Mapping lifetime names to positional indices (e.g., `'a` -> `l0`).
//! - Mapping type parameter names to positional indices (e.g., `T` -> `T0`).
//! - Sorting bounds to ensure order-independence.
//! - Recursively canonicalizing nested types and generic arguments.

use quote::quote;
use std::collections::BTreeMap;
use syn::{
	GenericArgument, GenericParam, Generics, PathArguments, ReturnType, Token, Type,
	TypeParamBound, punctuated::Punctuated,
};

/// Handles the canonicalization of type bounds and signatures.
///
/// This struct maintains mappings from original parameter names to their
/// canonical indices to ensure that renaming parameters doesn't change
/// the generated hash.
pub struct Canonicalizer {
	/// Maps lifetime names to their index (e.g., "a" -> 0).
	lifetime_map: BTreeMap<String, usize>,
	/// Maps type parameter names to their index (e.g., "T" -> 0).
	type_map: BTreeMap<String, usize>,
}

impl Canonicalizer {
	/// Creates a new `Canonicalizer` from a set of generics.
	///
	/// This initializes the mappings for lifetimes and type parameters based
	/// on their order in the `Generics` definition.
	pub fn new(generics: &Generics) -> Self {
		let mut lifetime_map = BTreeMap::new();
		let mut type_map = BTreeMap::new();

		let mut l_idx = 0;
		let mut t_idx = 0;

		for param in &generics.params {
			match param {
				GenericParam::Lifetime(lt) => {
					lifetime_map.insert(lt.lifetime.ident.to_string(), l_idx);
					l_idx += 1;
				}
				GenericParam::Type(ty) => {
					type_map.insert(ty.ident.to_string(), t_idx);
					t_idx += 1;
				}
				GenericParam::Const(_) => {
					// Const parameters are not currently supported for canonicalization mapping
					// They will be treated as literal values in bounds
				}
			}
		}

		Self { lifetime_map, type_map }
	}

	/// Canonicalizes a single type bound.
	///
	/// - Lifetimes are replaced by their index (e.g., `l0`).
	/// - Type parameters are replaced by their index (e.g., `T0`).
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
				// Check if it's a type parameter
				if let Some(ident) = type_path.path.get_ident() {
					if let Some(idx) = self.type_map.get(&ident.to_string()) {
						return format!("T{}", idx);
					}
				}

				type_path
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
					.join("::")
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

#[cfg(test)]
mod tests {
	use super::*;
	use syn::parse_quote;

	// ===========================================================================
	// Canonicalizer - Basic Bound Tests
	// ===========================================================================

	/// Tests canonicalization of a simple trait bound like `Clone`.
	#[test]
	fn test_canonicalize_simple_bound() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(Clone);
		assert_eq!(canon.canonicalize_bound(&bound), "tClone");
	}

	/// Tests canonicalization of a fully qualified path bound like `std::fmt::Debug`.
	#[test]
	fn test_canonicalize_path_bound() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(std::fmt::Debug);
		assert_eq!(canon.canonicalize_bound(&bound), "tstd::fmt::Debug");
	}

	/// Tests canonicalization of a generic trait bound with associated types.
	#[test]
	fn test_canonicalize_generic_bound() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(Iterator<Item = String>);
		assert_eq!(canon.canonicalize_bound(&bound), "tIterator<Item=String>");
	}

	/// Tests canonicalization of a `Fn` trait bound with parenthesized arguments.
	#[test]
	fn test_canonicalize_fn_bound() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(Fn(i32) -> bool);
		assert_eq!(canon.canonicalize_bound(&bound), "tFn(i32)->bool");
	}

	/// Tests canonicalization of a lifetime bound.
	#[test]
	fn test_canonicalize_lifetime_bound() {
		let generics: Generics = parse_quote!(<'a>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!('a);
		assert_eq!(canon.canonicalize_bound(&bound), "l0");
	}

	/// Tests that bounds are sorted to produce deterministic output.
	#[test]
	fn test_canonicalize_bounds_sorting() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bounds1: Punctuated<TypeParamBound, Token![+]> = parse_quote!(Clone + std::fmt::Debug);
		let bounds2: Punctuated<TypeParamBound, Token![+]> = parse_quote!(std::fmt::Debug + Clone);

		assert_eq!(canon.canonicalize_bounds(&bounds1), canon.canonicalize_bounds(&bounds2));
	}

	// ===========================================================================
	// Canonicalizer - Type Parameter Mapping Tests
	// ===========================================================================

	/// Tests that type parameters are mapped to positional indices (T0, T1).
	#[test]
	fn test_canonicalize_type_param_mapping() {
		let generics: Generics = parse_quote!(<T, U>);
		let canon = Canonicalizer::new(&generics);

		// T should be mapped to T0
		let bound_t: TypeParamBound = parse_quote!(AsRef<T>);
		assert_eq!(canon.canonicalize_bound(&bound_t), "tAsRef<T0>");

		// U should be mapped to T1
		let bound_u: TypeParamBound = parse_quote!(AsRef<U>);
		assert_eq!(canon.canonicalize_bound(&bound_u), "tAsRef<T1>");
	}

	/// Tests that renaming type parameters doesn't change the canonical output.
	#[test]
	fn test_canonicalize_type_param_independence() {
		// <A> vs <B> should produce same canonical form for same bounds
		let generics1: Generics = parse_quote!(<A>);
		let canon1 = Canonicalizer::new(&generics1);
		let bound1: TypeParamBound = parse_quote!(AsRef<A>);

		let generics2: Generics = parse_quote!(<B>);
		let canon2 = Canonicalizer::new(&generics2);
		let bound2: TypeParamBound = parse_quote!(AsRef<B>);

		assert_eq!(canon1.canonicalize_bound(&bound1), "tAsRef<T0>");
		assert_eq!(canon2.canonicalize_bound(&bound2), "tAsRef<T0>");
	}

	// ===========================================================================
	// Canonicalizer - Nested Types Tests
	// ===========================================================================

	/// Tests canonicalization of nested generic types.
	#[test]
	fn test_canonicalize_nested_generic() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		// Test with nested Option<Vec<String>>
		let bound: TypeParamBound = parse_quote!(Iterator<Item = Option<Vec<String>>>);
		let result = canon.canonicalize_bound(&bound);

		assert!(result.contains("Iterator"));
		assert!(result.contains("Option<Vec<String>>"));
	}

	/// Tests canonicalization of deeply nested generic types with type parameters.
	#[test]
	fn test_canonicalize_deeply_nested_with_params() {
		let generics: Generics = parse_quote!(<T>);
		let canon = Canonicalizer::new(&generics);

		// Test with deeply nested types involving T
		let bound: TypeParamBound = parse_quote!(AsRef<Vec<Option<T>>>);
		let result = canon.canonicalize_bound(&bound);

		assert_eq!(result, "tAsRef<Vec<Option<T0>>>");
	}

	/// Tests canonicalization of types with multiple generic parameters.
	#[test]
	fn test_canonicalize_multiple_generic_params() {
		let generics: Generics = parse_quote!(<E>);
		let canon = Canonicalizer::new(&generics);

		// Test with multiple type parameters
		let bound: TypeParamBound = parse_quote!(Into<Result<String, E>>);
		let result = canon.canonicalize_bound(&bound);

		assert_eq!(result, "tInto<Result<String,T0>>");
	}

	// ===========================================================================
	// Canonicalizer - Complex Fn Bounds Tests
	// ===========================================================================

	/// Tests canonicalization of Fn bounds with multiple arguments and type parameters.
	#[test]
	fn test_canonicalize_fn_complex() {
		let generics: Generics = parse_quote!(<T>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(Fn(T, String) -> Option<T>);
		let result = canon.canonicalize_bound(&bound);

		assert_eq!(result, "tFn(T0,String)->Option<T0>");
	}

	/// Tests canonicalization of Fn bounds with no explicit return type.
	#[test]
	fn test_canonicalize_fn_no_return() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(Fn(i32));
		let result = canon.canonicalize_bound(&bound);

		assert_eq!(result, "tFn(i32)->()");
	}

	/// Tests canonicalization of FnMut bounds.
	#[test]
	fn test_canonicalize_fnmut() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(FnMut(String) -> i32);
		let result = canon.canonicalize_bound(&bound);

		assert_eq!(result, "tFnMut(String)->i32");
	}

	/// Tests canonicalization of FnOnce bounds.
	#[test]
	fn test_canonicalize_fnonce() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(FnOnce() -> String);
		let result = canon.canonicalize_bound(&bound);

		assert_eq!(result, "tFnOnce()->String");
	}

	// ===========================================================================
	// Canonicalizer - Multiple Lifetimes Tests
	// ===========================================================================

	/// Tests canonicalization of multiple lifetimes to positional indices.
	#[test]
	fn test_canonicalize_multiple_lifetimes() {
		let generics: Generics = parse_quote!(<'a, 'b, 'c>);
		let canon = Canonicalizer::new(&generics);

		let bound_a: TypeParamBound = parse_quote!('a);
		let bound_b: TypeParamBound = parse_quote!('b);
		let bound_c: TypeParamBound = parse_quote!('c);

		assert_eq!(canon.canonicalize_bound(&bound_a), "l0");
		assert_eq!(canon.canonicalize_bound(&bound_b), "l1");
		assert_eq!(canon.canonicalize_bound(&bound_c), "l2");
	}

	/// Tests that lifetime names don't affect canonical representation.
	#[test]
	fn test_canonicalize_lifetime_independence() {
		let generics1: Generics = parse_quote!(<'a>);
		let canon1 = Canonicalizer::new(&generics1);

		let generics2: Generics = parse_quote!(<'x>);
		let canon2 = Canonicalizer::new(&generics2);

		let bound1: TypeParamBound = parse_quote!('a);
		let bound2: TypeParamBound = parse_quote!('x);

		assert_eq!(canon1.canonicalize_bound(&bound1), canon2.canonicalize_bound(&bound2));
	}
}
