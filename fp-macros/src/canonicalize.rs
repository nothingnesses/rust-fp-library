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
			Type::Path(type_path) => type_path
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
				.join("::"),
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
	///
	/// Verifies that simple trait bounds are prefixed with 't' and the trait name
	/// is preserved exactly.
	#[test]
	fn test_canonicalize_simple_bound() {
		let lifetimes = Punctuated::new();
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		let bound: TypeParamBound = parse_quote!(Clone);
		assert_eq!(canon.canonicalize_bound(&bound), "tClone");
	}

	/// Tests canonicalization of a fully qualified path bound like `std::fmt::Debug`.
	///
	/// Verifies that path segments are joined with `::` and prefixed with 't'.
	#[test]
	fn test_canonicalize_path_bound() {
		let lifetimes = Punctuated::new();
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		let bound: TypeParamBound = parse_quote!(std::fmt::Debug);
		assert_eq!(canon.canonicalize_bound(&bound), "tstd::fmt::Debug");
	}

	/// Tests canonicalization of a generic trait bound with associated types.
	///
	/// Verifies that `Iterator<Item = String>` is correctly formatted with
	/// the associated type binding included.
	#[test]
	fn test_canonicalize_generic_bound() {
		let lifetimes = Punctuated::new();
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		let bound: TypeParamBound = parse_quote!(Iterator<Item = String>);
		assert_eq!(canon.canonicalize_bound(&bound), "tIterator<Item=String>");
	}

	/// Tests canonicalization of a `Fn` trait bound with parenthesized arguments.
	///
	/// Verifies that Fn-style bounds are formatted as `Fn(args)->return`.
	#[test]
	fn test_canonicalize_fn_bound() {
		let lifetimes = Punctuated::new();
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		let bound: TypeParamBound = parse_quote!(Fn(i32) -> bool);
		assert_eq!(canon.canonicalize_bound(&bound), "tFn(i32)->bool");
	}

	/// Tests canonicalization of a lifetime bound.
	///
	/// Verifies that lifetimes are mapped to positional indices (l0, l1, etc.)
	/// rather than their actual names, ensuring name-independence.
	#[test]
	fn test_canonicalize_lifetime_bound() {
		let mut lifetimes = Punctuated::new();
		lifetimes.push(parse_quote!('a));
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		let bound: TypeParamBound = parse_quote!('a);
		assert_eq!(canon.canonicalize_bound(&bound), "l0");
	}

	/// Tests that bounds are sorted to produce deterministic output.
	///
	/// Verifies that `Clone + Debug` and `Debug + Clone` produce the same
	/// canonical representation, ensuring order-independence.
	#[test]
	fn test_canonicalize_bounds_sorting() {
		let lifetimes = Punctuated::new();
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		// These should produce the same result regardless of order
		let bounds1: Punctuated<TypeParamBound, Token![+]> = parse_quote!(Clone + std::fmt::Debug);
		let bounds2: Punctuated<TypeParamBound, Token![+]> = parse_quote!(std::fmt::Debug + Clone);

		assert_eq!(canon.canonicalize_bounds(&bounds1), canon.canonicalize_bounds(&bounds2));
	}

	// ===========================================================================
	// Canonicalizer - Nested Types Tests
	// ===========================================================================

	/// Tests canonicalization of nested generic types.
	///
	/// Verifies that types like `Iterator<Item = Option<String>>` are correctly
	/// flattened into a canonical string representation.
	#[test]
	fn test_canonicalize_nested_generic() {
		let lifetimes = Punctuated::new();
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		// Test with nested Option<Vec<T>>
		let bound: TypeParamBound = parse_quote!(Iterator<Item = Option<String>>);
		let result = canon.canonicalize_bound(&bound);

		assert!(result.contains("Iterator"));
		assert!(result.contains("Option<String>"));
	}

	/// Tests canonicalization of deeply nested generic types (3 levels).
	///
	/// Verifies that the canonicalizer handles arbitrary nesting depth
	/// like `AsRef<Vec<Option<String>>>`.
	#[test]
	fn test_canonicalize_deeply_nested_generic() {
		let lifetimes = Punctuated::new();
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		// Test with deeply nested types
		let bound: TypeParamBound = parse_quote!(AsRef<Vec<Option<String>>>);
		let result = canon.canonicalize_bound(&bound);

		assert!(result.contains("AsRef"));
		assert!(result.contains("Vec<Option<String>>"));
	}

	/// Tests canonicalization of types with multiple generic parameters.
	///
	/// Verifies that types like `Result<String, Error>` with multiple
	/// type parameters are correctly formatted.
	#[test]
	fn test_canonicalize_multiple_generic_params() {
		let lifetimes = Punctuated::new();
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		// Test with multiple type parameters
		let bound: TypeParamBound = parse_quote!(Into<Result<String, Error>>);
		let result = canon.canonicalize_bound(&bound);

		assert!(result.contains("Into"));
		assert!(result.contains("Result<String,Error>"));
	}

	// ===========================================================================
	// Canonicalizer - Complex Fn Bounds Tests
	// ===========================================================================

	/// Tests canonicalization of Fn bounds with multiple arguments.
	///
	/// Verifies that function bounds with multiple parameters are correctly
	/// formatted as `Fn(arg1,arg2,arg3)->return`.
	#[test]
	fn test_canonicalize_fn_multiple_args() {
		let lifetimes = Punctuated::new();
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		let bound: TypeParamBound = parse_quote!(Fn(i32, String, bool) -> Option<u64>);
		let result = canon.canonicalize_bound(&bound);

		assert_eq!(result, "tFn(i32,String,bool)->Option<u64>");
	}

	/// Tests canonicalization of Fn bounds with no explicit return type.
	///
	/// Verifies that missing return types default to `()`.
	#[test]
	fn test_canonicalize_fn_no_return() {
		let lifetimes = Punctuated::new();
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		let bound: TypeParamBound = parse_quote!(Fn(i32));
		let result = canon.canonicalize_bound(&bound);

		assert_eq!(result, "tFn(i32)->()");
	}

	/// Tests canonicalization of FnMut bounds.
	///
	/// Verifies that FnMut is handled the same way as Fn.
	#[test]
	fn test_canonicalize_fnmut() {
		let lifetimes = Punctuated::new();
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		let bound: TypeParamBound = parse_quote!(FnMut(String) -> i32);
		let result = canon.canonicalize_bound(&bound);

		assert_eq!(result, "tFnMut(String)->i32");
	}

	/// Tests canonicalization of FnOnce bounds.
	///
	/// Verifies that FnOnce with no arguments is handled correctly.
	#[test]
	fn test_canonicalize_fnonce() {
		let lifetimes = Punctuated::new();
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		let bound: TypeParamBound = parse_quote!(FnOnce() -> String);
		let result = canon.canonicalize_bound(&bound);

		assert_eq!(result, "tFnOnce()->String");
	}

	// ===========================================================================
	// Canonicalizer - Multiple Lifetimes Tests
	// ===========================================================================

	/// Tests canonicalization of multiple lifetimes to positional indices.
	///
	/// Verifies that lifetimes are assigned sequential indices (l0, l1, l2)
	/// based on their declaration order.
	#[test]
	fn test_canonicalize_multiple_lifetimes() {
		let mut lifetimes = Punctuated::new();
		lifetimes.push(parse_quote!('a));
		lifetimes.push(parse_quote!('b));
		lifetimes.push(parse_quote!('c));
		let types = Punctuated::new();
		let canon = Canonicalizer::new(&lifetimes, &types);

		let bound_a: TypeParamBound = parse_quote!('a);
		let bound_b: TypeParamBound = parse_quote!('b);
		let bound_c: TypeParamBound = parse_quote!('c);

		assert_eq!(canon.canonicalize_bound(&bound_a), "l0");
		assert_eq!(canon.canonicalize_bound(&bound_b), "l1");
		assert_eq!(canon.canonicalize_bound(&bound_c), "l2");
	}

	/// Tests that lifetime names don't affect canonical representation.
	///
	/// Verifies that `'a` in position 0 produces the same result as `'x` in
	/// position 0, ensuring that the actual lifetime name is irrelevant.
	#[test]
	fn test_canonicalize_lifetime_independence() {
		// Different lifetime names should produce the same canonical form
		// if they're in the same position
		let mut lifetimes1 = Punctuated::new();
		lifetimes1.push(parse_quote!('a));
		let canon1 = Canonicalizer::new(&lifetimes1, &Punctuated::new());

		let mut lifetimes2 = Punctuated::new();
		lifetimes2.push(parse_quote!('x));
		let canon2 = Canonicalizer::new(&lifetimes2, &Punctuated::new());

		let bound1: TypeParamBound = parse_quote!('a);
		let bound2: TypeParamBound = parse_quote!('x);

		assert_eq!(canon1.canonicalize_bound(&bound1), canon2.canonicalize_bound(&bound2));
	}
}
