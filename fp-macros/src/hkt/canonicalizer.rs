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

use {
	crate::{
		AssociatedTypes,
		core::{
			constants::markers::{
				INFERABLE_BRAND_PREFIX,
				KIND_PREFIX,
			},
			error_handling::{
				Error,
				UnsupportedFeature,
			},
		},
		support::type_visitor::TypeVisitor,
	},
	quote::{
		format_ident,
		quote,
	},
	std::collections::HashMap,
	syn::{
		GenericArgument,
		GenericParam,
		Generics,
		Ident,
		PathArguments,
		ReturnType,
		Token,
		Type,
		TypeParamBound,
		punctuated::Punctuated,
	},
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
	lifetime_map: HashMap<String, usize>,
	/// Maps type parameter names to their index (e.g., "T" -> 0).
	type_map: HashMap<String, usize>,
}

impl Canonicalizer {
	/// Creates a new `Canonicalizer` from a set of generics.
	///
	/// This initializes the mappings for lifetimes and type parameters based
	/// on their order in the `Generics` definition.
	pub fn new(generics: &Generics) -> Self {
		let mut lifetime_map = HashMap::new();
		let mut type_map = HashMap::new();

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
				GenericParam::Const(_) => {}
			}
		}

		Self {
			lifetime_map,
			type_map,
		}
	}

	/// Canonicalize a lifetime, replacing mapped lifetimes with positional indices.
	fn canonicalize_lifetime(
		&self,
		lt: &syn::Lifetime,
	) -> String {
		if let Some(idx) = self.lifetime_map.get(&lt.ident.to_string()) {
			format!("l{idx}")
		} else {
			lt.ident.to_string()
		}
	}

	/// Canonicalize path segments (shared by `canonicalize_bound` and `visit_path`).
	fn canonicalize_path_segments(
		&mut self,
		segments: &syn::punctuated::Punctuated<syn::PathSegment, Token![::]>,
	) -> Result<String> {
		let mut path_parts = Vec::new();
		for seg in segments {
			let ident = seg.ident.to_string();
			let segment_str = match &seg.arguments {
				PathArguments::None => ident,
				PathArguments::AngleBracketed(args) => {
					let mut args_vec = Vec::new();
					for arg in &args.args {
						args_vec.push(self.canonicalize_generic_arg(arg)?);
					}
					format!("{ident}<{}>", args_vec.join(","))
				}
				PathArguments::Parenthesized(args) => {
					let mut inputs_vec = Vec::new();
					for t in &args.inputs {
						inputs_vec.push(self.visit(t)?);
					}
					let output = self.canonicalize_return_type(&args.output)?;
					format!("{ident}({})->{output}", inputs_vec.join(","))
				}
			};
			path_parts.push(segment_str);
		}
		Ok(path_parts.join("::"))
	}

	/// Canonicalize a return type.
	fn canonicalize_return_type(
		&mut self,
		output: &ReturnType,
	) -> Result<String> {
		match output {
			ReturnType::Default => Ok("()".to_string()),
			ReturnType::Type(_, ty) => self.visit(ty),
		}
	}

	/// Canonicalizes a single type bound.
	///
	/// - Lifetimes are replaced by their index (e.g., `l0`).
	/// - Type parameters are replaced by their index (e.g., `T0`).
	/// - Traits are represented by their full path with generic arguments.
	pub fn canonicalize_bound(
		&mut self,
		bound: &TypeParamBound,
	) -> Result<String> {
		match bound {
			TypeParamBound::Lifetime(lt) => {
				let idx = self
					.lifetime_map
					.get(&lt.ident.to_string())
					.ok_or_else(|| Error::internal(format!("Unknown lifetime: {}", lt.ident)))?;
				Ok(format!("l{idx}"))
			}
			TypeParamBound::Trait(tr) => {
				let path = self.canonicalize_path_segments(&tr.path.segments)?;
				Ok(format!("t{path}"))
			}
			TypeParamBound::Verbatim(_tokens) =>
				Err(Error::Unsupported(UnsupportedFeature::VerbatimBounds {
					span: proc_macro2::Span::call_site(),
				})),
			other => Err(Error::Unsupported(UnsupportedFeature::BoundType {
				description: format!("Unsupported bound type: {}", quote!(#other)),
				span: proc_macro2::Span::call_site(),
			})),
		}
	}

	/// Canonicalizes a list of bounds, sorting them to ensure determinism.
	pub fn canonicalize_bounds(
		&mut self,
		bounds: &Punctuated<TypeParamBound, Token![+]>,
	) -> Result<String> {
		self.sorted_bounds(bounds, "")
	}

	/// Canonicalize bounds into a sorted, joined string with the given separator.
	fn sorted_bounds(
		&mut self,
		bounds: &Punctuated<TypeParamBound, Token![+]>,
		sep: &str,
	) -> Result<String> {
		let mut parts: Vec<String> = Vec::new();
		for b in bounds {
			parts.push(self.canonicalize_bound(b)?);
		}
		parts.sort();
		Ok(parts.join(sep))
	}

	/// Canonicalize a const expression, substituting mapped type parameter
	/// names with their canonical form.
	fn canonicalize_const_expr(
		&self,
		expr: &syn::Expr,
	) -> String {
		let tokens = quote!(#expr);
		let mut result = proc_macro2::TokenStream::new();
		for tt in tokens {
			match &tt {
				proc_macro2::TokenTree::Ident(ident) => {
					let name = ident.to_string();
					if let Some(idx) = self.type_map.get(&name) {
						result.extend(std::iter::once(proc_macro2::TokenTree::Ident(
							proc_macro2::Ident::new(&format!("T{idx}"), ident.span()),
						)));
					} else {
						result.extend(std::iter::once(tt));
					}
				}
				_ => result.extend(std::iter::once(tt)),
			}
		}
		result.to_string().replace(' ', "")
	}

	fn canonicalize_generic_arg(
		&mut self,
		arg: &GenericArgument,
	) -> Result<String> {
		match arg {
			GenericArgument::Type(ty) => self.visit(ty),
			GenericArgument::Lifetime(lt) => Ok(self.canonicalize_lifetime(lt)),
			GenericArgument::AssocType(assoc) =>
				Ok(format!("{}={}", assoc.ident, self.visit(&assoc.ty)?)),
			GenericArgument::Const(expr) => Ok(self.canonicalize_const_expr(expr)),
			GenericArgument::AssocConst(_) | GenericArgument::Constraint(_) =>
				Err(Error::Unsupported(UnsupportedFeature::GenericArgument {
					description: "Associated const or constraint".to_string(),
					span: proc_macro2::Span::call_site(),
				})),
			other => Err(Error::Unsupported(UnsupportedFeature::GenericArgument {
				description: format!("Unsupported generic argument: {}", quote!(#other)),
				span: proc_macro2::Span::call_site(),
			})),
		}
	}
}

impl TypeVisitor for Canonicalizer {
	type Output = Result<String>;

	fn default_output(&self) -> Self::Output {
		Err(Error::Unsupported(UnsupportedFeature::ComplexTypes {
			description: "Unsupported type variant in canonicalization".to_string(),
			span: proc_macro2::Span::call_site(),
		}))
	}

	fn visit_path(
		&mut self,
		type_path: &syn::TypePath,
	) -> Self::Output {
		// Handle qualified self types (e.g., <T as Iterator>::Item)
		if let Some(qself) = &type_path.qself {
			let qself_str = self.visit(&qself.ty)?;
			let path_str = self.canonicalize_path_segments(&type_path.path.segments)?;
			return Ok(format!("<{qself_str}>::{path_str}"));
		}

		// Check if it's a simple type parameter
		if let Some(ident) = type_path.path.get_ident()
			&& let Some(idx) = self.type_map.get(&ident.to_string())
		{
			return Ok(format!("T{idx}"));
		}

		self.canonicalize_path_segments(&type_path.path.segments)
	}

	fn visit_reference(
		&mut self,
		type_ref: &syn::TypeReference,
	) -> Self::Output {
		let lt = type_ref
			.lifetime
			.as_ref()
			.map_or(String::new(), |lt| format!("{} ", self.canonicalize_lifetime(lt)));
		let mutability = if type_ref.mutability.is_some() { "mut " } else { "" };
		Ok(format!("&{lt}{mutability}{}", self.visit(&type_ref.elem)?))
	}

	fn visit_tuple(
		&mut self,
		tuple: &syn::TypeTuple,
	) -> Self::Output {
		let mut elems_vec = Vec::new();
		for t in &tuple.elems {
			elems_vec.push(self.visit(t)?);
		}
		Ok(format!("({})", elems_vec.join(",")))
	}

	fn visit_slice(
		&mut self,
		slice: &syn::TypeSlice,
	) -> Self::Output {
		Ok(format!("[{}]", self.visit(&slice.elem)?))
	}

	fn visit_array(
		&mut self,
		array: &syn::TypeArray,
	) -> Self::Output {
		let len_expr = &array.len;
		let len = self.canonicalize_const_expr(len_expr);
		Ok(format!("[{};{len}]", self.visit(&array.elem)?))
	}

	fn visit_bare_fn(
		&mut self,
		bare_fn: &syn::TypeBareFn,
	) -> Self::Output {
		let mut prefix = String::new();
		if bare_fn.unsafety.is_some() {
			prefix.push_str("unsafe ");
		}
		if let Some(abi) = &bare_fn.abi {
			prefix.push_str("extern ");
			if let Some(name) = &abi.name {
				prefix.push_str(&format!("{} ", name.value()));
			}
		}
		let mut inputs_vec = Vec::new();
		for arg in &bare_fn.inputs {
			inputs_vec.push(self.visit(&arg.ty)?);
		}
		let output = self.canonicalize_return_type(&bare_fn.output)?;
		Ok(format!("{prefix}fn({})->{output}", inputs_vec.join(",")))
	}

	fn visit_trait_object(
		&mut self,
		trait_object: &syn::TypeTraitObject,
	) -> Self::Output {
		Ok(format!("dyn {}", self.sorted_bounds(&trait_object.bounds, "+")?))
	}

	fn visit_impl_trait(
		&mut self,
		impl_trait: &syn::TypeImplTrait,
	) -> Self::Output {
		Ok(format!("impl {}", self.sorted_bounds(&impl_trait.bounds, "+")?))
	}

	fn visit_other(
		&mut self,
		ty: &syn::Type,
	) -> Self::Output {
		match ty {
			Type::Never(_) => Ok("!".to_string()),
			Type::Infer(_) => Ok("_".to_string()),
			Type::Paren(paren) => self.visit(&paren.elem),
			Type::Group(group) => self.visit(&group.elem),
			_ => self.default_output(),
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

pub fn rapidhash(data: &[u8]) -> u64 {
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
pub fn generate_assoc_signature(signature: &crate::hkt::AssociatedTypeBase) -> Result<String> {
	let mut canon = Canonicalizer::new(&signature.generics);

	let mut l_count = 0;
	let mut t_count = 0;
	let mut type_bounds_parts = Vec::new();

	for param in &signature.generics.params {
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

	let mut parts = vec![signature.name.to_string(), format!("L{l_count}"), format!("T{t_count}")];
	parts.extend(type_bounds_parts);

	if !signature.output_bounds.is_empty() {
		let bounds_str = canon.canonicalize_bounds(&signature.output_bounds)?;
		parts.push(format!("O{bounds_str}"));
	}

	Ok(parts.join("_"))
}

/// Generates a 64-bit hash for a single associated type's signature.
pub fn hash_assoc_signature(signature: &crate::hkt::AssociatedTypeBase) -> Result<u64> {
	let repr = generate_assoc_signature(signature)?;
	Ok(rapidhash(repr.as_bytes()))
}

/// Generates a deterministic 64-bit hash for a set of associated type signatures.
///
/// The hash is computed by:
/// 1. Sorting associated types by name (order-independence).
/// 2. Generating a canonical string for each associated type.
/// 3. Joining them with `__` and hashing with rapidhash.
///
/// This is the shared foundation for both `Kind_{hash}` and
/// `InferableBrand_{hash}` name generation.
pub fn generate_hash(input: &AssociatedTypes) -> Result<u64> {
	let mut assoc_types: Vec<_> = input.associated_types.iter().collect();
	// Sort by identifier to ensure order-independence
	assoc_types.sort_by_key(|a| a.signature.name.to_string());

	let mut canonical_parts = Vec::new();

	for assoc in assoc_types {
		canonical_parts.push(generate_assoc_signature(&assoc.signature)?);
	}

	let canonical_repr = canonical_parts.join("__");

	Ok(rapidhash(canonical_repr.as_bytes()))
}

/// Generates a prefixed identifier from a signature hash.
fn generate_prefixed_name(
	prefix: &str,
	input: &AssociatedTypes,
) -> Result<Ident> {
	let hash = generate_hash(input)?;
	Ok(format_ident!("{prefix}{:016x}", hash))
}

/// Generates a `Kind_{hash}` identifier from the input signature.
pub fn generate_name(input: &AssociatedTypes) -> Result<Ident> {
	generate_prefixed_name(KIND_PREFIX, input)
}

/// Generates a `InferableBrand_{hash}` identifier from the input signature.
///
/// Uses the same content hash as [`generate_name`], so a `Kind_{hash}` trait
/// and its corresponding `InferableBrand_{hash}` trait always share the same hash suffix.
pub fn generate_inferable_brand_name(input: &AssociatedTypes) -> Result<Ident> {
	generate_prefixed_name(INFERABLE_BRAND_PREFIX, input)
}

#[cfg(test)]
#[expect(
	clippy::unwrap_used,
	clippy::expect_used,
	reason = "Tests use panicking operations for brevity and clarity"
)]
mod tests {
	use {
		super::*,
		syn::parse_quote,
	};

	// ===========================================================================
	// Canonicalizer - Basic Bound Tests
	// ===========================================================================

	/// Tests canonicalization of a simple trait bound like `Clone`.
	#[test]
	fn test_canonicalize_simple_bound() {
		let generics: Generics = parse_quote!(<>);
		let mut canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(Clone);
		assert_eq!(canon.canonicalize_bound(&bound).unwrap(), "tClone");
	}

	/// Tests canonicalization of a fully qualified path bound like `std::fmt::Debug`.
	#[test]
	fn test_canonicalize_path_bound() {
		let generics: Generics = parse_quote!(<>);
		let mut canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(std::fmt::Debug);
		assert_eq!(canon.canonicalize_bound(&bound).unwrap(), "tstd::fmt::Debug");
	}

	/// Tests canonicalization of a generic trait bound with associated types.
	#[test]
	fn test_canonicalize_generic_bound() {
		let generics: Generics = parse_quote!(<>);
		let mut canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(Iterator<Item = String>);
		assert_eq!(canon.canonicalize_bound(&bound).unwrap(), "tIterator<Item=String>");
	}

	/// Tests canonicalization of a `Fn` trait bound with parenthesized arguments.
	#[test]
	fn test_canonicalize_fn_bound() {
		let generics: Generics = parse_quote!(<>);
		let mut canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(Fn(i32) -> bool);
		assert_eq!(canon.canonicalize_bound(&bound).unwrap(), "tFn(i32)->bool");
	}

	/// Tests canonicalization of a lifetime bound.
	#[test]
	fn test_canonicalize_lifetime_bound() {
		let generics: Generics = parse_quote!(<'a>);
		let mut canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!('a);
		assert_eq!(canon.canonicalize_bound(&bound).unwrap(), "l0");
	}

	/// Tests that bounds are sorted to produce deterministic output.
	#[test]
	fn test_canonicalize_bounds_sorting() {
		let generics: Generics = parse_quote!(<>);
		let mut canon = Canonicalizer::new(&generics);

		let bounds1: Punctuated<TypeParamBound, Token![+]> = parse_quote!(Clone + std::fmt::Debug);
		let bounds2: Punctuated<TypeParamBound, Token![+]> = parse_quote!(std::fmt::Debug + Clone);

		assert_eq!(
			canon.canonicalize_bounds(&bounds1).unwrap(),
			canon.canonicalize_bounds(&bounds2).unwrap()
		);
	}

	// ===========================================================================
	// Canonicalizer - Type Parameter Mapping Tests
	// ===========================================================================

	/// Tests that type parameters are mapped to positional indices (T0, T1).
	#[test]
	fn test_canonicalize_type_param_mapping() {
		let generics: Generics = parse_quote!(<T, U>);
		let mut canon = Canonicalizer::new(&generics);

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
		let mut canon1 = Canonicalizer::new(&generics1);
		let bound1: TypeParamBound = parse_quote!(AsRef<A>);

		let generics2: Generics = parse_quote!(<B>);
		let mut canon2 = Canonicalizer::new(&generics2);
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
		let mut canon = Canonicalizer::new(&generics);

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
		let mut canon = Canonicalizer::new(&generics);

		// Test with deeply nested types involving T
		let bound: TypeParamBound = parse_quote!(AsRef<Vec<Option<T>>>);
		let result = canon.canonicalize_bound(&bound).unwrap();

		assert_eq!(result, "tAsRef<Vec<Option<T0>>>");
	}

	/// Tests canonicalization of types with multiple generic parameters.
	#[test]
	fn test_canonicalize_multiple_generic_params() {
		let generics: Generics = parse_quote!(<E>);
		let mut canon = Canonicalizer::new(&generics);

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
		let mut canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(Fn(T, String) -> Option<T>);
		let result = canon.canonicalize_bound(&bound).unwrap();

		assert_eq!(result, "tFn(T0,String)->Option<T0>");
	}

	/// Tests canonicalization of Fn bounds with no explicit return type.
	#[test]
	fn test_canonicalize_fn_no_return() {
		let generics: Generics = parse_quote!(<>);
		let mut canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(Fn(i32));
		let result = canon.canonicalize_bound(&bound).unwrap();

		assert_eq!(result, "tFn(i32)->()");
	}

	/// Tests canonicalization of FnMut bounds.
	#[test]
	fn test_canonicalize_fnmut() {
		let generics: Generics = parse_quote!(<>);
		let mut canon = Canonicalizer::new(&generics);

		let bound: TypeParamBound = parse_quote!(FnMut(String) -> i32);
		let result = canon.canonicalize_bound(&bound).unwrap();

		assert_eq!(result, "tFnMut(String)->i32");
	}

	/// Tests canonicalization of FnOnce bounds.
	#[test]
	fn test_canonicalize_fnonce() {
		let generics: Generics = parse_quote!(<>);
		let mut canon = Canonicalizer::new(&generics);

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
		let mut canon = Canonicalizer::new(&generics);

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
		let mut canon1 = Canonicalizer::new(&generics1);

		let generics2: Generics = parse_quote!(<'x>);
		let mut canon2 = Canonicalizer::new(&generics2);

		let bound1: TypeParamBound = parse_quote!('a);
		let bound2: TypeParamBound = parse_quote!('x);

		assert_eq!(
			canon1.canonicalize_bound(&bound1).unwrap(),
			canon2.canonicalize_bound(&bound2).unwrap()
		);
	}

	// ===========================================================================
	// Name Generation Tests
	// ===========================================================================

	/// Helper function to parse a KindInput from a string.
	fn parse_kind_input(input: &str) -> AssociatedTypes {
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

	// ===========================================================================
	// Type Visitor Tests
	// ===========================================================================

	/// Helper to canonicalize a type with the given generic parameters.
	fn canon_type(
		generics_str: &str,
		ty_str: &str,
	) -> String {
		let generics: Generics = syn::parse_str(generics_str).expect("Failed to parse generics");
		let ty: Type = syn::parse_str(ty_str).expect("Failed to parse type");
		let mut canon = Canonicalizer::new(&generics);
		canon.visit(&ty).expect("Failed to canonicalize type")
	}

	#[test]
	fn test_visit_reference() {
		assert_eq!(canon_type("<T>", "&T"), "&T0");
		assert_eq!(canon_type("<T>", "&mut T"), "&mut T0");
	}

	#[test]
	fn test_visit_reference_with_lifetime() {
		assert_eq!(canon_type("<'a, T>", "&'a T"), "&l0 T0");
		assert_eq!(canon_type("<'a, T>", "&'a mut T"), "&l0 mut T0");
	}

	#[test]
	fn test_visit_reference_static_lifetime() {
		assert_eq!(canon_type("<T>", "&'static T"), "&static T0");
	}

	#[test]
	fn test_visit_tuple() {
		assert_eq!(canon_type("<>", "()"), "()");
		assert_eq!(canon_type("<T, U>", "(T, U)"), "(T0,T1)");
	}

	#[test]
	fn test_visit_slice() {
		assert_eq!(canon_type("<T>", "[T]"), "[T0]");
	}

	#[test]
	fn test_visit_array() {
		assert_eq!(canon_type("<T>", "[T; 5]"), "[T0;5]");
	}

	#[test]
	fn test_visit_array_type_param_independence() {
		assert_eq!(canon_type("<A>", "[A; 3]"), canon_type("<B>", "[B; 3]"));
	}

	#[test]
	fn test_visit_bare_fn() {
		assert_eq!(canon_type("<>", "fn()"), "fn()->()");
		assert_eq!(canon_type("<T, U>", "fn(T) -> U"), "fn(T0)->T1");
		assert_eq!(canon_type("<A, B, C>", "fn(A, B) -> C"), "fn(T0,T1)->T2");
	}

	#[test]
	fn test_visit_bare_fn_unsafe() {
		assert_eq!(canon_type("<T>", "unsafe fn(T) -> T"), "unsafe fn(T0)->T0");
	}

	#[test]
	fn test_visit_bare_fn_extern() {
		assert_eq!(canon_type("<T>", "extern \"C\" fn(T) -> T"), "extern C fn(T0)->T0");
	}

	#[test]
	fn test_visit_bare_fn_unsafe_extern() {
		assert_eq!(
			canon_type("<T>", "unsafe extern \"C\" fn(T) -> T"),
			"unsafe extern C fn(T0)->T0"
		);
	}

	#[test]
	fn test_visit_bare_fn_distinguishes_unsafety() {
		let safe = canon_type("<T>", "fn(T) -> T");
		let unsafe_ = canon_type("<T>", "unsafe fn(T) -> T");
		assert_ne!(safe, unsafe_);
	}

	#[test]
	fn test_visit_bare_fn_distinguishes_abi() {
		let rust = canon_type("<T>", "fn(T) -> T");
		let c = canon_type("<T>", "extern \"C\" fn(T) -> T");
		assert_ne!(rust, c);
	}

	#[test]
	fn test_visit_trait_object() {
		let generics: Generics = parse_quote!(<>);
		let ty: Type = parse_quote!(dyn Clone);
		let mut canon = Canonicalizer::new(&generics);
		let result = canon.visit(&ty).unwrap();
		assert_eq!(result, "dyn tClone");
	}

	#[test]
	fn test_visit_trait_object_multiple_bounds_sorted() {
		let generics: Generics = parse_quote!(<>);
		let ty1: Type = parse_quote!(dyn Clone + Send);
		let ty2: Type = parse_quote!(dyn Send + Clone);
		let mut canon1 = Canonicalizer::new(&generics);
		let mut canon2 = Canonicalizer::new(&generics);
		assert_eq!(canon1.visit(&ty1).unwrap(), canon2.visit(&ty2).unwrap());
	}

	#[test]
	fn test_visit_impl_trait() {
		let generics: Generics = parse_quote!(<>);
		let ty: Type = parse_quote!(impl Clone);
		let mut canon = Canonicalizer::new(&generics);
		let result = canon.visit(&ty).unwrap();
		assert_eq!(result, "impl tClone");
	}

	#[test]
	fn test_visit_impl_trait_multiple_bounds_sorted() {
		let generics: Generics = parse_quote!(<>);
		let ty1: Type = parse_quote!(impl Clone + Send);
		let ty2: Type = parse_quote!(impl Send + Clone);
		let mut canon1 = Canonicalizer::new(&generics);
		let mut canon2 = Canonicalizer::new(&generics);
		assert_eq!(canon1.visit(&ty1).unwrap(), canon2.visit(&ty2).unwrap());
	}

	#[test]
	fn test_visit_never() {
		assert_eq!(canon_type("<>", "!"), "!");
	}

	#[test]
	fn test_visit_path_with_qself() {
		let generics: Generics = parse_quote!(<T>);
		let ty: Type = parse_quote!(<T as Iterator>::Item);
		let mut canon = Canonicalizer::new(&generics);
		let result = canon.visit(&ty).unwrap();
		assert_eq!(result, "<T0>::Iterator::Item");
	}

	#[test]
	fn test_visit_path_qself_type_param_independence() {
		let generics1: Generics = parse_quote!(<A>);
		let ty1: Type = parse_quote!(<A as Iterator>::Item);
		let mut canon1 = Canonicalizer::new(&generics1);

		let generics2: Generics = parse_quote!(<B>);
		let ty2: Type = parse_quote!(<B as Iterator>::Item);
		let mut canon2 = Canonicalizer::new(&generics2);

		assert_eq!(canon1.visit(&ty1).unwrap(), canon2.visit(&ty2).unwrap());
	}

	#[test]
	fn test_canonicalize_const_expr_with_type_param() {
		let generics: Generics = parse_quote!(<T>);
		let canon = Canonicalizer::new(&generics);
		let expr: syn::Expr = parse_quote!(std::mem::size_of::<T>());
		let result = canon.canonicalize_const_expr(&expr);
		assert!(result.contains("T0"), "Expected T to be canonicalized to T0, got: {result}");
		assert!(!result.contains("T>"), "Expected raw T to be replaced, got: {result}");
	}
}
