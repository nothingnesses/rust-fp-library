//! Unit tests for the `fp-macros` crate.
//!
//! This module contains comprehensive tests covering:
//! - **Canonicalizer**: Tests for canonicalization of type bounds and lifetimes
//! - **Name Generation**: Tests for deterministic Kind trait name generation
//! - **impl_kind!**: Tests for parsing and code generation
//! - **def_kind!**: Tests for trait definition generation
//! - **Apply!**: Tests for both named and positional argument syntax

use crate::{
	apply::{ApplyInput, KindSource, apply_impl},
	canonicalize::Canonicalizer,
	def_kind::def_kind_impl,
	generate::generate_name,
	impl_kind::{ImplKindInput, impl_kind_impl},
	parse::KindInput,
};
use {
	quote::quote,
	syn::{Token, TypeParamBound, parse_quote, punctuated::Punctuated},
};

/// Helper function to parse a KindInput from a string.
fn parse_kind_input(input: &str) -> KindInput {
	syn::parse_str(input).expect("Failed to parse KindInput")
}

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
// Name Generation Tests
// ===========================================================================

/// Tests that identical inputs produce identical Kind trait names.
///
/// This is critical for the HKT system - the same signature must always
/// map to the same trait name across different compilation units.
#[test]
fn test_generate_name_determinism() {
	let input1 = parse_kind_input("('a), (A: 'a), ('a)");
	let name1 = generate_name(&input1);

	let input2 = parse_kind_input("('a), (A: 'a), ('a)");
	let name2 = generate_name(&input2);

	assert_eq!(name1, name2);
	assert!(name1.to_string().starts_with("Kind_"));
}

/// Tests that different inputs produce different Kind trait names.
///
/// Verifies that the hash function produces distinct names for
/// semantically different signatures.
#[test]
fn test_generate_name_different_inputs() {
	let input1 = parse_kind_input("('a), (A: 'a), ('a)");
	let name1 = generate_name(&input1);

	let input2 = parse_kind_input("(), (A), ()");
	let name2 = generate_name(&input2);

	assert_ne!(name1, name2);
}

// ===========================================================================
// impl_kind! Parsing and Generation Tests (Original)
// ===========================================================================

/// Tests basic parsing of impl_kind! input.
///
/// Verifies that the parser correctly identifies the associated type name "Of".
#[test]
fn test_parse_impl_kind() {
	let input = "for OptionBrand { type Of<'a, A: 'a>: 'a = Option<A>; }";
	let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

	assert_eq!(parsed.definition.of_ident.to_string(), "Of");
}

/// Tests code generation for impl_kind!.
///
/// Verifies that the generated impl block contains the correct trait name,
/// brand type, and associated type definition.
#[test]
fn test_impl_kind_generation() {
	let input = "for OptionBrand { type Of<'a, A: 'a>: 'a = Option<A>; }";
	let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

	let output = impl_kind_impl(parsed);
	let output_str = output.to_string();

	assert!(output_str.contains("impl Kind_"));
	assert!(output_str.contains("for OptionBrand"));
	// Note: Bounds on associated types in impl blocks are not emitted because
	// they have no effect in Rust (bounds are only valid in trait definitions).
	assert!(output_str.contains("type Of < 'a , A : 'a > = Option < A >"));
}

// ===========================================================================
// Apply! Named Parameter Tests (Original)
// ===========================================================================

/// Tests parsing of Apply! with named parameters.
///
/// Verifies that the parser correctly extracts brand, signature, lifetimes,
/// and types from the named parameter syntax.
#[test]
fn test_parse_apply() {
	let input = "brand: OptionBrand, signature: ('a, A: 'a) -> 'a, lifetimes: ('a), types: (A)";
	let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse ApplyInput");

	assert_eq!(parsed.lifetimes.len(), 1);
	assert_eq!(parsed.types.len(), 1);
}

/// Tests code generation for Apply! with named parameters.
///
/// Verifies that the generated code projects the brand to its concrete type
/// using the correct Kind trait.
#[test]
fn test_apply_generation() {
	let input = "brand: OptionBrand, signature: ('a, A: 'a) -> 'a, lifetimes: ('a), types: (A)";
	let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse ApplyInput");

	let output = apply_impl(parsed);
	let output_str = output.to_string();

	assert!(output_str.contains("< OptionBrand as Kind_"));
	assert!(output_str.contains(":: Of < 'a , A >"));
}

// ===========================================================================
// def_kind! Tests
// ===========================================================================

/// Tests def_kind! with a single type parameter.
///
/// Verifies that the generated trait has the correct structure with
/// a single unbounded type parameter.
#[test]
fn test_def_kind_simple_type() {
	let input = parse_kind_input("(), (A), ()");
	let output = def_kind_impl(input);
	let output_str = output.to_string();

	// Should generate a trait with Kind_ prefix
	assert!(output_str.contains("pub trait Kind_"));
	// Should have associated type Of with single type parameter
	assert!(output_str.contains("type Of < A >"));
}

/// Tests def_kind! with a lifetime and a type bounded by that lifetime.
///
/// Verifies that lifetime bounds on type parameters are correctly emitted.
#[test]
fn test_def_kind_with_lifetime() {
	let input = parse_kind_input("('a), (A: 'a), ()");
	let output = def_kind_impl(input);
	let output_str = output.to_string();

	assert!(output_str.contains("pub trait Kind_"));
	// Should have both lifetime and bounded type parameter
	assert!(output_str.contains("'a"));
	assert!(output_str.contains("A : 'a"));
}

/// Tests def_kind! with bounds on the output type.
///
/// Verifies that output bounds (bounds on the associated type itself)
/// are correctly emitted after a colon.
#[test]
fn test_def_kind_with_output_bounds() {
	let input = parse_kind_input("(), (A), (Clone)");
	let output = def_kind_impl(input);
	let output_str = output.to_string();

	assert!(output_str.contains("pub trait Kind_"));
	// Should have output bounds
	assert!(output_str.contains(": Clone"));
}

/// Tests def_kind! with multiple output bounds.
///
/// Verifies that multiple bounds on the associated type are correctly
/// joined with `+`.
#[test]
fn test_def_kind_with_multiple_output_bounds() {
	let input = parse_kind_input("(), (A), (Clone + Send)");
	let output = def_kind_impl(input);
	let output_str = output.to_string();

	assert!(output_str.contains("pub trait Kind_"));
	assert!(output_str.contains("Clone"));
	assert!(output_str.contains("Send"));
}

/// Tests def_kind! with bounds on a type parameter.
///
/// Verifies that bounds on type parameters (not output bounds) are
/// correctly emitted after the type name.
#[test]
fn test_def_kind_with_type_bounds() {
	let input = parse_kind_input("(), (A: Clone + Send), ()");
	let output = def_kind_impl(input);
	let output_str = output.to_string();

	assert!(output_str.contains("pub trait Kind_"));
	assert!(output_str.contains("A : Clone + Send"));
}

/// Tests def_kind! with only lifetimes (no type parameters).
///
/// Verifies that the macro handles lifetime-only signatures correctly.
#[test]
fn test_def_kind_only_lifetimes() {
	let input = parse_kind_input("('a, 'b), (), ()");
	let output = def_kind_impl(input);
	let output_str = output.to_string();

	assert!(output_str.contains("pub trait Kind_"));
	assert!(output_str.contains("'a"));
	assert!(output_str.contains("'b"));
}

/// Tests def_kind! with multiple type parameters.
///
/// Verifies that multiple type parameters are correctly comma-separated.
#[test]
fn test_def_kind_multiple_types() {
	let input = parse_kind_input("(), (A, B, C), ()");
	let output = def_kind_impl(input);
	let output_str = output.to_string();

	assert!(output_str.contains("pub trait Kind_"));
	assert!(output_str.contains("type Of < A , B , C >"));
}

// ===========================================================================
// Apply! Positional Arguments Tests
// ===========================================================================

/// Tests parsing of Apply! with legacy positional syntax.
///
/// Verifies that the parser correctly handles the positional syntax:
/// `Brand, Kind, (lifetimes), (types)` and uses KindSource::Explicit.
#[test]
fn test_apply_positional_parsing() {
	// Legacy positional syntax: Brand, Kind, (lifetimes), (types)
	let input = "OptionBrand, SomeKind, ('a), (String)";
	let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse ApplyInput positional");

	assert_eq!(parsed.lifetimes.len(), 1);
	assert_eq!(parsed.types.len(), 1);

	// Should use explicit kind source
	match parsed.kind_source {
		KindSource::Explicit(ty) => {
			assert_eq!(quote!(#ty).to_string(), "SomeKind");
		}
		KindSource::Generated(_) => panic!("Expected explicit kind source"),
	}
}

/// Tests code generation for Apply! with positional syntax.
///
/// Verifies that the generated projection uses the explicitly provided
/// Kind trait name rather than generating one.
#[test]
fn test_apply_positional_generation() {
	let input = "OptionBrand, SomeKind, ('a), (String)";
	let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse ApplyInput positional");

	let output = apply_impl(parsed);
	let output_str = output.to_string();

	assert!(output_str.contains("< OptionBrand as SomeKind >"));
	assert!(output_str.contains(":: Of < 'a , String >"));
}

/// Tests Apply! positional syntax with no lifetimes.
///
/// Verifies that empty lifetime parentheses are handled correctly.
#[test]
fn test_apply_positional_no_lifetimes() {
	let input = "MyBrand, MyKind, (), (T, U)";
	let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse ApplyInput positional");

	assert_eq!(parsed.lifetimes.len(), 0);
	assert_eq!(parsed.types.len(), 2);

	let output = apply_impl(parsed);
	let output_str = output.to_string();

	assert!(output_str.contains("< MyBrand as MyKind >"));
	assert!(output_str.contains(":: Of < T , U >"));
}

/// Tests Apply! positional syntax with no type arguments.
///
/// Verifies that empty type parentheses are handled correctly
/// when only lifetimes are provided.
#[test]
fn test_apply_positional_no_types() {
	let input = "MyBrand, MyKind, ('a, 'b), ()";
	let parsed: ApplyInput = syn::parse_str(input).expect("Failed to parse ApplyInput positional");

	assert_eq!(parsed.lifetimes.len(), 2);
	assert_eq!(parsed.types.len(), 0);

	let output = apply_impl(parsed);
	let output_str = output.to_string();

	assert!(output_str.contains("< MyBrand as MyKind >"));
	assert!(output_str.contains(":: Of < 'a , 'b >"));
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

// ===========================================================================
// impl_kind! with generics Tests
// ===========================================================================

/// Tests impl_kind! with a single impl-level generic parameter.
///
/// Verifies that `impl<E> for ResultBrand<E>` correctly passes the
/// generic parameter to both the impl block and the brand type.
#[test]
fn test_impl_kind_with_impl_generics() {
	let input = "impl<E> for ResultBrand<E> { type Of<A> = Result<A, E>; }";
	let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

	assert_eq!(parsed.definition.of_ident.to_string(), "Of");

	let output = impl_kind_impl(parsed);
	let output_str = output.to_string();

	assert!(output_str.contains("impl < E > Kind_"));
	assert!(output_str.contains("for ResultBrand < E >"));
}

/// Tests impl_kind! with multiple bounded impl-level generics.
///
/// Verifies that bounds on impl generics (e.g., `E: Clone, F: Send`)
/// are preserved in the generated output.
#[test]
fn test_impl_kind_with_multiple_impl_generics() {
	let input = "impl<E: Clone, F: Send> for MyBrand<E, F> { type Of<A> = MyType<A, E, F>; }";
	let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

	let output = impl_kind_impl(parsed);
	let output_str = output.to_string();

	assert!(output_str.contains("impl < E : Clone , F : Send > Kind_"));
	assert!(output_str.contains("for MyBrand < E , F >"));
}

/// Tests impl_kind! with a path-qualified bound.
///
/// Verifies that bounds like `std::fmt::Debug` are correctly preserved.
#[test]
fn test_impl_kind_with_bounded_impl_generic() {
	// Test that bounds on impl generics are preserved
	let input = "impl<E: std::fmt::Debug> for ResultBrand<E> { type Of<A> = Result<A, E>; }";
	let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

	let output = impl_kind_impl(parsed);
	let output_str = output.to_string();

	assert!(output_str.contains("impl < E : std :: fmt :: Debug > Kind_"));
	assert!(output_str.contains("for ResultBrand < E >"));
}

/// Tests impl_kind! with multiple lifetimes and multiple type parameters.
///
/// Verifies that complex signatures with multiple lifetimes and types
/// are correctly handled.
#[test]
fn test_impl_kind_multiple_lifetimes_and_types() {
	let input = "for MyBrand { type Of<'a, 'b, A: 'a, B: 'b> = MyType<'a, 'b, A, B>; }";
	let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

	let output = impl_kind_impl(parsed);
	let output_str = output.to_string();

	assert!(output_str.contains("impl Kind_"));
	assert!(output_str.contains("type Of < 'a , 'b , A : 'a , B : 'b >"));
}

// ===========================================================================
// impl_kind! with where clauses Tests
// ===========================================================================

/// Tests impl_kind! with a single where clause bound.
///
/// Verifies that `where E: Debug` is correctly parsed and emitted
/// in the generated impl block.
#[test]
fn test_impl_kind_with_where_clause() {
	let input =
		"impl<E> for ResultBrand<E> where E: std::fmt::Debug { type Of<A> = Result<A, E>; }";
	let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

	let output = impl_kind_impl(parsed);
	let output_str = output.to_string();

	assert!(output_str.contains("impl < E > Kind_"));
	assert!(output_str.contains("for ResultBrand < E >"));
	assert!(output_str.contains("where E : std :: fmt :: Debug"));
}

/// Tests impl_kind! with multiple where clause predicates.
///
/// Verifies that `where E: Clone, F: Send` with multiple predicates
/// is correctly parsed and emitted.
#[test]
fn test_impl_kind_with_multiple_where_bounds() {
	let input =
		"impl<E, F> for MyBrand<E, F> where E: Clone, F: Send { type Of<A> = MyType<A, E, F>; }";
	let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

	let output = impl_kind_impl(parsed);
	let output_str = output.to_string();

	assert!(output_str.contains("impl < E , F >"));
	assert!(output_str.contains("where E : Clone , F : Send"));
}

/// Tests impl_kind! with multiple trait bounds in a single where predicate.
///
/// Verifies that `where E: Clone + Send + Sync` with multiple traits
/// on one parameter is correctly handled.
#[test]
fn test_impl_kind_with_complex_where_bounds() {
	let input =
		"impl<E> for ResultBrand<E> where E: Clone + Send + Sync { type Of<A> = Result<A, E>; }";
	let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

	let output = impl_kind_impl(parsed);
	let output_str = output.to_string();

	assert!(output_str.contains("where E : Clone + Send + Sync"));
}

// ===========================================================================
// Name Generation Edge Cases
// ===========================================================================

/// Tests name generation with completely empty inputs.
///
/// Verifies that `(), (), ()` (no lifetimes, no types, no bounds)
/// still produces a valid Kind trait name.
#[test]
fn test_generate_name_empty_inputs() {
	let input = parse_kind_input("(), (), ()");
	let name = generate_name(&input);

	assert!(name.to_string().starts_with("Kind_"));
}

/// Tests name generation with complex bounded types.
///
/// Verifies determinism - parsing the same complex input twice
/// must produce the same name.
#[test]
fn test_generate_name_complex_bounds() {
	// Complex case with multiple bounded types (using simpler bounds to avoid parser issues)
	let input = parse_kind_input("('a), (A: Clone + Send), (Clone + Send)");
	let name = generate_name(&input);

	assert!(name.to_string().starts_with("Kind_"));
	// Ensure determinism
	let input2 = parse_kind_input("('a), (A: Clone + Send), (Clone + Send)");
	let name2 = generate_name(&input2);
	assert_eq!(name, name2);
}

/// Tests that bound order doesn't affect the generated name.
///
/// Verifies that `Clone + Send` produces the same name as `Send + Clone`,
/// ensuring that syntactically different but semantically equivalent
/// signatures map to the same Kind trait.
#[test]
fn test_generate_name_bound_order_independence() {
	// Bounds in different order should produce the same name
	let input1 = parse_kind_input("(), (A: Clone + Send), ()");
	let input2 = parse_kind_input("(), (A: Send + Clone), ()");

	let name1 = generate_name(&input1);
	let name2 = generate_name(&input2);

	assert_eq!(name1, name2);
}
