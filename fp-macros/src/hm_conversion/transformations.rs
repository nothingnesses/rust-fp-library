//! Canonicalization and name generation for `Kind` traits.
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
//! - Generating unique, deterministic identifiers for `Kind` traits.

use super::patterns::KindInput;
use crate::error::{Error, UnsupportedFeature};
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use syn::{
	GenericArgument, GenericParam, Generics, Ident, PathArguments, ReturnType, Token, Type,
	TypeParamBound, punctuated::Punctuated,
};

/// Result type for canonicalization operations
type Result<T> = std::result::Result<T, Error>;

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
	) -> Result<String> {
		match bound {
			TypeParamBound::Lifetime(lt) => {
				let idx = self.lifetime_map
					.get(&lt.ident.to_string())
					.ok_or_else(|| Error::internal(format!("Unknown lifetime: {}", lt.ident)))?;
				Ok(format!("l{idx}"))
			}
			TypeParamBound::Trait(tr) => {
				// Full path with generic arguments
				let mut path_parts = Vec::new();
				for seg in &tr.path.segments {
					let ident = seg.ident.to_string();
					let segment_str = match &seg.arguments {
						PathArguments::None => ident,
						PathArguments::AngleBracketed(args) => {
							let mut args_vec = Vec::new();
							for arg in &args.args {
								args_vec.push(self.canonicalize_generic_arg(arg)?);
							}
							let args_str = args_vec.join(",");
							format!("{ident}<{args_str}>")
						}
						PathArguments::Parenthesized(args) => {
							// Fn trait bounds: Fn(A) -> B
							let mut inputs_vec = Vec::new();
							for t in &args.inputs {
								inputs_vec.push(self.canonicalize_type(t)?);
							}
							let inputs = inputs_vec.join(",");
							let output = match &args.output {
								ReturnType::Default => "()".to_string(),
								ReturnType::Type(_, ty) => self.canonicalize_type(ty)?,
							};
							format!("{ident}({inputs})->{output}")
						}
					};
					path_parts.push(segment_str);
				}
				let path = path_parts.join("::");
				Ok(format!("t{path}"))
			}
			TypeParamBound::Verbatim(_tokens) => {
				Err(Error::Unsupported(UnsupportedFeature::VerbatimBounds {
					span: proc_macro2::Span::call_site(),
				}))
			}
			_ => {
				Err(Error::Unsupported(UnsupportedFeature::BoundType {
					description: "Unknown bound type variant".to_string(),
					span: proc_macro2::Span::call_site(),
				}))
			}
		}
	}

	/// Canonicalizes a list of bounds, sorting them to ensure determinism.
	pub fn canonicalize_bounds(
		&self,
		bounds: &Punctuated<TypeParamBound, Token![+]>,
	) -> Result<String> {
		let mut parts: Vec<String> = Vec::new();
		for b in bounds {
			parts.push(self.canonicalize_bound(b)?);
		}
		parts.sort(); // Ensure deterministic order
		Ok(parts.join(""))
	}

	fn canonicalize_generic_arg(
		&self,
		arg: &GenericArgument,
	) -> Result<String> {
		match arg {
			GenericArgument::Type(ty) => self.canonicalize_type(ty),
			GenericArgument::Lifetime(lt) => {
				if let Some(idx) = self.lifetime_map.get(&lt.ident.to_string()) {
					Ok(format!("l{idx}"))
				} else {
					Ok(lt.ident.to_string())
				}
			}
			GenericArgument::AssocType(assoc) => {
				Ok(format!("{}={}", assoc.ident, self.canonicalize_type(&assoc.ty)?))
			}
			GenericArgument::Const(expr) => {
				Ok(quote!(#expr).to_string().replace(" ", ""))
			}
			GenericArgument::AssocConst(_) | GenericArgument::Constraint(_) => {
				Err(Error::Unsupported(UnsupportedFeature::GenericArgument {
					description: "Associated const or constraint".to_string(),
					span: proc_macro2::Span::call_site(),
				}))
			}
			_ => {
				Err(Error::Unsupported(UnsupportedFeature::GenericArgument {
					description: "Unknown generic argument variant".to_string(),
					span: proc_macro2::Span::call_site(),
				}))
			}
		}
	}

	fn canonicalize_type(
		&self,
		ty: &Type,
	) -> Result<String> {
		match ty {
			Type::Path(type_path) => {
				// Check if it's a type parameter
				if let Some(ident) = type_path.path.get_ident()
					&& let Some(idx) = self.type_map.get(&ident.to_string())
				{
					return Ok(format!("T{idx}"));
				}

				let mut path_parts = Vec::new();
				for seg in &type_path.path.segments {
					let ident = seg.ident.to_string();
					let segment_str = match &seg.arguments {
						PathArguments::None => ident,
						PathArguments::AngleBracketed(args) => {
							let mut args_vec = Vec::new();
							for a in &args.args {
								args_vec.push(self.canonicalize_generic_arg(a)?);
							}
							let args_str = args_vec.join(",");
							format!("{ident}<{args_str}>")
						}
						PathArguments::Parenthesized(args) => {
							let mut inputs_vec = Vec::new();
							for t in &args.inputs {
								inputs_vec.push(self.canonicalize_type(t)?);
							}
							let inputs = inputs_vec.join(",");
							let output = match &args.output {
								ReturnType::Default => "()".to_string(),
								ReturnType::Type(_, ty) => self.canonicalize_type(ty)?,
							};
							format!("{ident}({inputs})->{output}")
						}
					};
					path_parts.push(segment_str);
				}
				Ok(path_parts.join("::"))
			}
			Type::Reference(type_ref) => {
				let lt = if let Some(lt) = &type_ref.lifetime {
					if let Some(idx) = self.lifetime_map.get(&lt.ident.to_string()) {
						format!("l{idx} ")
					} else {
						format!("{} ", lt.ident)
					}
				} else {
					"".to_string()
				};
				let mutability = if type_ref.mutability.is_some() { "mut " } else { "" };
				Ok(format!("&{lt}{mutability}{}", self.canonicalize_type(&type_ref.elem)?))
			}
			Type::Tuple(tuple) => {
				let mut elems_vec = Vec::new();
				for t in &tuple.elems {
					elems_vec.push(self.canonicalize_type(t)?);
				}
				let elems = elems_vec.join(",");
				Ok(format!("({elems})"))
			}
			Type::Slice(slice) => {
				Ok(format!("[{}]", self.canonicalize_type(&slice.elem)?))
			}
			Type::Array(array) => {
				let len = quote!(#array.len).to_string().replace(" ", "");
				Ok(format!("[{};{len}]", self.canonicalize_type(&array.elem)?))
			}
			Type::Never(_) => Ok("!".to_string()),
			Type::Infer(_) => Ok("_".to_string()),
			Type::BareFn(_) | Type::ImplTrait(_) | Type::TraitObject(_) => {
				Err(Error::Unsupported(UnsupportedFeature::ComplexTypes {
					description: format!("Type {} in canonicalization", quote!(#ty)),
					span: proc_macro2::Span::call_site(),
				}))
			}
			_ => {
				Err(Error::Unsupported(UnsupportedFeature::ComplexTypes {
					description: "Unknown type variant in canonicalization".to_string(),
					span: proc_macro2::Span::call_site(),
				}))
			}
		}
	}
}

// ===========================================================================
// Name Generation
// ===========================================================================

// Deterministic hashing setup
// Using a fixed seed for reproducibility across builds
/// Fixed seed for deterministic hash generation.
///
/// This seed value MUST remain constant across all versions to ensure that
/// Kind trait names are stable between compilations and across different
/// machines. Changing this value will break all existing Kind implementations.
///
/// The specific value `0x1234567890abcdef` was chosen arbitrarily but serves
/// to distinguish our hashes from other hash functions that might use
/// default seeds (typically 0 or random values).
///
/// # Stability Guarantee
///
/// This constant is part of the public API surface (indirectly, through generated
/// trait names) and must never be changed in any release, as it would cause
/// a breaking change for all users of the `Kind!` macro.
const RAPID_SECRETS: rapidhash::v3::RapidSecrets =
	rapidhash::v3::RapidSecrets::seed(0x1234567890abcdef);

fn rapidhash(data: &[u8]) -> u64 {
	rapidhash::v3::rapidhash_v3_seeded(data, &RAPID_SECRETS)
}

/// Generates a unique, deterministic identifier for a `Kind` trait based on its input signature.
///
/// The name format is `Kind_{hash}`, where `{hash}` is a 16-character hexadecimal string
/// representing the 64-bit hash of the canonical signature.
///
/// The canonical signature is constructed by:
/// 1. Sorting associated types by name.
/// 2. For each associated type, creating a canonical string including:
///    - Name
///    - Lifetime count
///    - Type parameter count
///    - Canonicalized bounds on type parameters
///    - Canonicalized output bounds
/// 3. Joining these strings with `__`.
pub fn generate_name(input: &KindInput) -> Result<Ident> {
	let mut assoc_types: Vec<_> = input.assoc_types.iter().collect();
	// Sort by identifier to ensure order-independence
	assoc_types.sort_by(|a, b| a.ident.to_string().cmp(&b.ident.to_string()));

	let mut canonical_parts = Vec::new();

	for assoc in assoc_types {
		let canon = Canonicalizer::new(&assoc.generics);

		let mut l_count = 0;
		let mut t_count = 0;
		let mut type_bounds_parts = Vec::new();

		for param in &assoc.generics.params {
			match param {
				GenericParam::Lifetime(_) => l_count += 1,
				GenericParam::Type(ty) => {
					if !ty.bounds.is_empty() {
						let bounds_str = canon.canonicalize_bounds(&ty.bounds)?;
						// Use current type index for bound association
						type_bounds_parts.push(format!("B{t_count}{bounds_str}"));
					}
					t_count += 1;
				}
				_ => {}
			}
		}

		let mut parts = vec![assoc.ident.to_string(), format!("L{l_count}"), format!("T{t_count}")];
		parts.extend(type_bounds_parts);

		if !assoc.output_bounds.is_empty() {
			let bounds_str = canon.canonicalize_bounds(&assoc.output_bounds)?;
			parts.push(format!("O{bounds_str}"));
		}

		canonical_parts.push(parts.join("_"));
	}

	let canonical_repr = canonical_parts.join("__");

	// Always use hash for consistency and to avoid length issues
	let hash = rapidhash(canonical_repr.as_bytes());
	Ok(format_ident!("Kind_{:016x}", hash))
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
		assert_eq!(canon.canonicalize_bound(&bound).unwrap(), "tClone");
	}

	/// Tests canonicalization of a fully qualified path bound like `std::fmt::Debug`.
	#[test]
	fn test_canonicalize_path_bound() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(std::fmt::Debug);
		assert_eq!(canon.canonicalize_bound(&bound).unwrap(), "tstd::fmt::Debug");
	}

	/// Tests canonicalization of a generic trait bound with associated types.
	#[test]
	fn test_canonicalize_generic_bound() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(Iterator<Item = String>);
		assert_eq!(canon.canonicalize_bound(&bound).unwrap(), "tIterator<Item=String>");
	}

	/// Tests canonicalization of a `Fn` trait bound with parenthesized arguments.
	#[test]
	fn test_canonicalize_fn_bound() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(Fn(i32) -> bool);
		assert_eq!(canon.canonicalize_bound(&bound).unwrap(), "tFn(i32)->bool");
	}

	/// Tests canonicalization of a lifetime bound.
	#[test]
	fn test_canonicalize_lifetime_bound() {
		let generics: Generics = parse_quote!(<'a>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!('a);
		assert_eq!(canon.canonicalize_bound(&bound).unwrap(), "l0");
	}

	/// Tests that bounds are sorted to produce deterministic output.
	#[test]
	fn test_canonicalize_bounds_sorting() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bounds1: Punctuated<TypeParamBound, Token![+]> = parse_quote!(Clone + std::fmt::Debug);
		let bounds2: Punctuated<TypeParamBound, Token![+]> = parse_quote!(std::fmt::Debug + Clone);

		assert_eq!(canon.canonicalize_bounds(&bounds1).unwrap(), canon.canonicalize_bounds(&bounds2).unwrap());
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
		assert_eq!(canon.canonicalize_bound(&bound_t).unwrap(), "tAsRef<T0>");

		// U should be mapped to T1
		let bound_u: TypeParamBound = parse_quote!(AsRef<U>);
		assert_eq!(canon.canonicalize_bound(&bound_u).unwrap(), "tAsRef<T1>");
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

		assert_eq!(canon1.canonicalize_bound(&bound1).unwrap(), "tAsRef<T0>");
		assert_eq!(canon2.canonicalize_bound(&bound2).unwrap(), "tAsRef<T0>");
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
		let result = canon.canonicalize_bound(&bound).unwrap();

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
		let result = canon.canonicalize_bound(&bound).unwrap();

		assert_eq!(result, "tAsRef<Vec<Option<T0>>>");
	}

	/// Tests canonicalization of types with multiple generic parameters.
	#[test]
	fn test_canonicalize_multiple_generic_params() {
		let generics: Generics = parse_quote!(<E>);
		let canon = Canonicalizer::new(&generics);

		// Test with multiple type parameters
		let bound: TypeParamBound = parse_quote!(Into<Result<String, E>>);
		let result = canon.canonicalize_bound(&bound).unwrap();

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
		let result = canon.canonicalize_bound(&bound).unwrap();

		assert_eq!(result, "tFn(T0,String)->Option<T0>");
	}

	/// Tests canonicalization of Fn bounds with no explicit return type.
	#[test]
	fn test_canonicalize_fn_no_return() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(Fn(i32));
		let result = canon.canonicalize_bound(&bound).unwrap();

		assert_eq!(result, "tFn(i32)->()");
	}

	/// Tests canonicalization of FnMut bounds.
	#[test]
	fn test_canonicalize_fnmut() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(FnMut(String) -> i32);
		let result = canon.canonicalize_bound(&bound).unwrap();

		assert_eq!(result, "tFnMut(String)->i32");
	}

	/// Tests canonicalization of FnOnce bounds.
	#[test]
	fn test_canonicalize_fnonce() {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(FnOnce() -> String);
		let result = canon.canonicalize_bound(&bound).unwrap();

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

		assert_eq!(canon.canonicalize_bound(&bound_a).unwrap(), "l0");
		assert_eq!(canon.canonicalize_bound(&bound_b).unwrap(), "l1");
		assert_eq!(canon.canonicalize_bound(&bound_c).unwrap(), "l2");
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

		assert_eq!(canon1.canonicalize_bound(&bound1).unwrap(), canon2.canonicalize_bound(&bound2).unwrap());
	}

	// ===========================================================================
	// Name Generation Tests
	// ===========================================================================

	/// Helper function to parse a KindInput from a string.
	fn parse_kind_input(input: &str) -> KindInput {
		syn::parse_str(input).expect("Failed to parse KindInput")
	}

	/// Tests that identical inputs produce identical `Kind` trait names.
	#[test]
	fn test_generate_name_determinism() {
		let input1 = parse_kind_input("type Of<'a, A: 'a>: 'a;");
		let name1 = generate_name(&input1).unwrap();

		let input2 = parse_kind_input("type Of<'a, A: 'a>: 'a;");
		let name2 = generate_name(&input2).unwrap();

		assert_eq!(name1, name2);
		assert!(name1.to_string().starts_with("Kind_"));
	}

	/// Tests that different inputs produce different `Kind` trait names.
	#[test]
	fn test_generate_name_different_inputs() {
		let input1 = parse_kind_input("type Of<'a, A: 'a>: 'a;");
		let name1 = generate_name(&input1).unwrap();

		let input2 = parse_kind_input("type Of<A>;");
		let name2 = generate_name(&input2).unwrap();

		assert_ne!(name1, name2);
	}

	/// Tests that associated type order doesn't affect the generated name.
	#[test]
	fn test_generate_name_order_independence() {
		let input1 = parse_kind_input(
			"
			type Of<'a, T>: Display;
			type SendOf<U>: Send;
		",
		);
		let name1 = generate_name(&input1).unwrap();

		let input2 = parse_kind_input(
			"
			type SendOf<U>: Send;
			type Of<'a, T>: Display;
		",
		);
		let name2 = generate_name(&input2).unwrap();

		assert_eq!(name1, name2);
	}

	/// Tests name generation with complex bounded types.
	#[test]
	fn test_generate_name_complex_bounds() {
		let input = parse_kind_input("type Of<'a, A: Clone + Send>: Clone + Send;");
		let name = generate_name(&input).unwrap();

		assert!(name.to_string().starts_with("Kind_"));

		// Ensure determinism
		let input2 = parse_kind_input("type Of<'a, A: Clone + Send>: Clone + Send;");
		let name2 = generate_name(&input2).unwrap();
		assert_eq!(name, name2);
	}

	/// Tests that bound order doesn't affect the generated name.
	#[test]
	fn test_generate_name_bound_order_independence() {
		let input1 = parse_kind_input("type Of<A: Clone + Send>;");
		let input2 = parse_kind_input("type Of<A: Send + Clone>;");

		let name1 = generate_name(&input1).unwrap();
		let name2 = generate_name(&input2).unwrap();

		assert_eq!(name1, name2);
	}
}
