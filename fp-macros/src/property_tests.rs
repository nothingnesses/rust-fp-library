//! Property-based tests for the `fp-macros` crate.
//!
//! This module contains property tests using quickcheck to verify:
//! - **Hash Determinism**: Same input always produces the same hash
//! - **Canonicalization Equivalence**: Equivalent bounds produce the same canonical form
//! - **Bound Order Independence**: Order of bounds doesn't affect the result
//! - **Lifetime Name Independence**: Lifetime names don't affect canonical representation

use crate::{canonicalize::Canonicalizer, generate::generate_name, parse::KindInput};
use quickcheck::{Arbitrary, Gen, quickcheck};
use syn::{Generics, Token, TypeParamBound, parse_quote, punctuated::Punctuated};

// ===========================================================================
// Arbitrary Implementations for Test Data Generation
// ===========================================================================

/// Represents a trait bound that can be randomly generated.
#[derive(Debug, Clone)]
struct ArbTraitBound {
	name: String,
}

/// A list of valid trait names for random selection.
const TRAIT_NAMES: &[&str] = &[
	"Clone",
	"Copy",
	"Send",
	"Sync",
	"Debug",
	"Display",
	"Default",
	"Eq",
	"PartialEq",
	"Ord",
	"PartialOrd",
	"Hash",
	"Iterator",
	"IntoIterator",
	"AsRef",
	"AsMut",
	"From",
	"Into",
	"TryFrom",
	"TryInto",
];

impl Arbitrary for ArbTraitBound {
	fn arbitrary(g: &mut Gen) -> Self {
		let idx = usize::arbitrary(g) % TRAIT_NAMES.len();
		ArbTraitBound { name: TRAIT_NAMES[idx].to_string() }
	}
}

/// Represents a type name that can be randomly generated.
#[derive(Debug, Clone)]
struct ArbTypeName {
	name: String,
}

/// A list of valid type names for random selection.
const TYPE_NAMES: &[&str] =
	&["A", "B", "C", "D", "E", "F", "G", "T", "U", "V", "W", "X", "Y", "Z", "Item", "Key", "Value"];

impl Arbitrary for ArbTypeName {
	fn arbitrary(g: &mut Gen) -> Self {
		let idx = usize::arbitrary(g) % TYPE_NAMES.len();
		ArbTypeName { name: TYPE_NAMES[idx].to_string() }
	}
}

/// Represents a lifetime name that can be randomly generated.
#[derive(Debug, Clone)]
struct ArbLifetime {
	name: char,
}

impl Arbitrary for ArbLifetime {
	fn arbitrary(g: &mut Gen) -> Self {
		// Generate lifetimes 'a through 'z
		let idx = usize::arbitrary(g) % 26;
		let name = (b'a' + idx as u8) as char;
		ArbLifetime { name }
	}
}

/// A set of unique lifetime names (no duplicates).
#[derive(Debug, Clone)]
struct UniqueLifetimes {
	names: Vec<char>,
}

impl Arbitrary for UniqueLifetimes {
	fn arbitrary(g: &mut Gen) -> Self {
		// Generate 0-4 unique lifetimes
		let count = usize::arbitrary(g) % 5;
		let mut names = Vec::with_capacity(count);
		let available: Vec<char> = ('a'..='z').collect();

		for i in 0..count {
			if i < available.len() {
				names.push(available[i]);
			}
		}

		UniqueLifetimes { names }
	}
}

/// A set of unique trait bounds (no duplicates).
#[derive(Debug, Clone)]
struct UniqueBounds {
	bounds: Vec<String>,
}

impl Arbitrary for UniqueBounds {
	fn arbitrary(g: &mut Gen) -> Self {
		// Generate 0-5 unique bounds
		let count = usize::arbitrary(g) % 6;
		let mut bounds = Vec::with_capacity(count);
		let mut used = std::collections::HashSet::new();

		for _ in 0..count {
			let arb = ArbTraitBound::arbitrary(g);
			if !used.contains(&arb.name) {
				used.insert(arb.name.clone());
				bounds.push(arb.name);
			}
		}

		UniqueBounds { bounds }
	}
}

// ===========================================================================
// Property: Hash Determinism
// ===========================================================================

/// Property: Parsing the same string twice produces identical generated names.
///
/// This verifies that the hash function is deterministic - the same input
/// will always produce the same Kind trait name.
#[test]
fn prop_hash_determinism_simple() {
	fn property(
		bounds: UniqueBounds,
		lifetimes: UniqueLifetimes,
	) -> bool {
		// Build a KindInput string: type Of<...>: ...;
		let mut params = Vec::new();

		// Add lifetimes
		for lt in &lifetimes.names {
			params.push(format!("'{}", lt));
		}

		// Add type parameter with bounds
		if !bounds.bounds.is_empty() && !lifetimes.names.is_empty() {
			let lt = lifetimes.names.first().unwrap();
			params.push(format!("A: '{} + {}", lt, bounds.bounds.join(" + ")));
		} else if !bounds.bounds.is_empty() {
			params.push(format!("A: {}", bounds.bounds.join(" + ")));
		} else {
			params.push("A".to_string());
		}

		let params_str = params.join(", ");
		let input_str = format!("type Of<{}>;", params_str);

		// Parse twice and compare
		let result1: Result<KindInput, _> = syn::parse_str(&input_str);
		let result2: Result<KindInput, _> = syn::parse_str(&input_str);

		match (result1, result2) {
			(Ok(input1), Ok(input2)) => {
				let name1 = generate_name(&input1);
				let name2 = generate_name(&input2);
				name1 == name2
			}
			_ => true, // Skip unparseable inputs
		}
	}

	quickcheck(property as fn(UniqueBounds, UniqueLifetimes) -> bool);
}

/// Property: The same canonical representation always produces the same hash.
#[test]
fn prop_hash_determinism_repeated() {
	fn property(iterations: u8) -> bool {
		let iterations = (iterations % 10) + 1; // 1-10 iterations
		let input_str = "type Of<'a, A: 'a>;";

		let first_name = {
			let input: KindInput = syn::parse_str(input_str).unwrap();
			generate_name(&input).to_string()
		};

		for _ in 0..iterations {
			let input: KindInput = syn::parse_str(input_str).unwrap();
			let name = generate_name(&input).to_string();
			if name != first_name {
				return false;
			}
		}

		true
	}

	quickcheck(property as fn(u8) -> bool);
}

// ===========================================================================
// Property: Canonicalization Equivalence
// ===========================================================================

/// Property: Bounds in any order produce the same canonical form when sorted.
///
/// This verifies that `Clone + Send` and `Send + Clone` canonicalize identically.
#[test]
fn prop_bound_order_independence() {
	fn property(
		b1: ArbTraitBound,
		b2: ArbTraitBound,
	) -> bool {
		// Skip if bounds are the same (trivially true)
		if b1.name == b2.name {
			return true;
		}

		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		// Parse individual bounds and create punctuated list
		let bound1: Result<TypeParamBound, _> = syn::parse_str(&b1.name);
		let bound2: Result<TypeParamBound, _> = syn::parse_str(&b2.name);

		match (bound1, bound2) {
			(Ok(b1_parsed), Ok(b2_parsed)) => {
				// Create bounds in order 1
				let mut bounds1: Punctuated<TypeParamBound, Token![+]> = Punctuated::new();
				bounds1.push(b1_parsed.clone());
				bounds1.push(b2_parsed.clone());

				// Create bounds in order 2
				let mut bounds2: Punctuated<TypeParamBound, Token![+]> = Punctuated::new();
				bounds2.push(b2_parsed);
				bounds2.push(b1_parsed);

				let canonical1 = canon.canonicalize_bounds(&bounds1);
				let canonical2 = canon.canonicalize_bounds(&bounds2);
				canonical1 == canonical2
			}
			_ => true, // Skip unparseable
		}
	}

	quickcheck(property as fn(ArbTraitBound, ArbTraitBound) -> bool);
}

/// Property: Any permutation of N bounds produces the same canonical form.
#[test]
fn prop_bound_permutation_independence() {
	fn property(bounds: UniqueBounds) -> bool {
		if bounds.bounds.len() < 2 {
			return true; // Need at least 2 bounds to permute
		}

		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		// Parse all bounds
		let parsed_bounds: Result<Vec<TypeParamBound>, _> =
			bounds.bounds.iter().map(|b| syn::parse_str(b)).collect();

		match parsed_bounds {
			Ok(parsed) => {
				// Original order
				let mut original: Punctuated<TypeParamBound, Token![+]> = Punctuated::new();
				for b in parsed.iter() {
					original.push(b.clone());
				}

				// Reversed order
				let mut reversed: Punctuated<TypeParamBound, Token![+]> = Punctuated::new();
				for b in parsed.iter().rev() {
					reversed.push(b.clone());
				}

				let canonical_original = canon.canonicalize_bounds(&original);
				let canonical_reversed = canon.canonicalize_bounds(&reversed);
				canonical_original == canonical_reversed
			}
			_ => true,
		}
	}

	quickcheck(property as fn(UniqueBounds) -> bool);
}

// ===========================================================================
// Property: Lifetime Name Independence
// ===========================================================================

/// Property: Different lifetime names in the same position produce the same canonical form.
///
/// This verifies that `'a: 'b` and `'x: 'y` with lifetimes in the same positions
/// produce identical canonical representations.
#[test]
fn prop_lifetime_name_independence() {
	fn property(
		lt1: ArbLifetime,
		lt2: ArbLifetime,
	) -> bool {
		// Skip if same name
		if lt1.name == lt2.name {
			return true;
		}

		// Create two canonicalizers with different lifetime names
		let generics1: Generics = syn::parse_str(&format!("<'{}>", lt1.name)).unwrap();
		let canon1 = Canonicalizer::new(&generics1);

		let generics2: Generics = syn::parse_str(&format!("<'{}>", lt2.name)).unwrap();
		let canon2 = Canonicalizer::new(&generics2);

		// Both should canonicalize their respective lifetimes to "l0"
		let bound1: TypeParamBound = syn::parse_str(&format!("'{}", lt1.name)).unwrap();
		let bound2: TypeParamBound = syn::parse_str(&format!("'{}", lt2.name)).unwrap();

		let canonical1 = canon1.canonicalize_bound(&bound1);
		let canonical2 = canon2.canonicalize_bound(&bound2);

		canonical1 == canonical2 && canonical1 == "l0"
	}

	quickcheck(property as fn(ArbLifetime, ArbLifetime) -> bool);
}

/// Property: Multiple lifetimes in different positions are canonicalized consistently.
#[test]
fn prop_multiple_lifetime_positions() {
	fn property(lts: UniqueLifetimes) -> bool {
		if lts.names.len() < 2 {
			return true;
		}

		let lts_str = lts.names.iter().map(|n| format!("'{}", n)).collect::<Vec<_>>().join(", ");
		let generics: Generics = syn::parse_str(&format!("<{}>", lts_str)).unwrap();
		let canon = Canonicalizer::new(&generics);

		// Verify each lifetime maps to the correct index
		for (i, name) in lts.names.iter().enumerate() {
			let bound: TypeParamBound = syn::parse_str(&format!("'{}", name)).unwrap();
			let canonical = canon.canonicalize_bound(&bound);
			let expected = format!("l{}", i);
			if canonical != expected {
				return false;
			}
		}

		true
	}

	quickcheck(property as fn(UniqueLifetimes) -> bool);
}

// ===========================================================================
// Property: Generated Names Have Correct Format
// ===========================================================================

/// Property: All generated names start with "Kind_" prefix.
#[test]
fn prop_generated_name_format() {
	fn property(bounds: UniqueBounds) -> bool {
		let bounds_str = if bounds.bounds.is_empty() {
			"A".to_string()
		} else {
			format!("A: {}", bounds.bounds.join(" + "))
		};

		let input_str = format!("type Of<{}>;", bounds_str);
		let result: Result<KindInput, _> = syn::parse_str(&input_str);

		match result {
			Ok(input) => {
				let name = generate_name(&input).to_string();
				// Should start with Kind_ and have exactly 16 hex chars after
				name.starts_with("Kind_") && name.len() == "Kind_".len() + 16
			}
			_ => true,
		}
	}

	quickcheck(property as fn(UniqueBounds) -> bool);
}

/// Property: Generated name contains only valid identifier characters.
#[test]
fn prop_generated_name_valid_identifier() {
	fn property(
		bounds: UniqueBounds,
		lifetimes: UniqueLifetimes,
	) -> bool {
		let mut params = Vec::new();
		for lt in &lifetimes.names {
			params.push(format!("'{}", lt));
		}

		if !bounds.bounds.is_empty() {
			params.push(format!("A: {}", bounds.bounds.join(" + ")));
		} else {
			params.push("A".to_string());
		}

		let input_str = format!("type Of<{}>;", params.join(", "));
		let result: Result<KindInput, _> = syn::parse_str(&input_str);

		match result {
			Ok(input) => {
				let name = generate_name(&input).to_string();
				// Check valid Rust identifier: starts with letter/underscore,
				// contains only alphanumeric and underscore
				name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
					&& name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
			}
			_ => true,
		}
	}

	quickcheck(property as fn(UniqueBounds, UniqueLifetimes) -> bool);
}

// ===========================================================================
// Property: Different Inputs Produce Different Names
// ===========================================================================

/// Property: Adding a bound changes the generated name.
#[test]
fn prop_adding_bound_changes_name() {
	fn property(bound: ArbTraitBound) -> bool {
		let input_without: KindInput = syn::parse_str("type Of<A>;").unwrap();
		let input_with: Result<KindInput, _> =
			syn::parse_str(&format!("type Of<A: {}>;", bound.name));

		match input_with {
			Ok(input) => {
				let name_without = generate_name(&input_without);
				let name_with = generate_name(&input);
				name_without != name_with
			}
			_ => true,
		}
	}

	quickcheck(property as fn(ArbTraitBound) -> bool);
}

/// Property: Adding a lifetime changes the generated name.
#[test]
fn prop_adding_lifetime_changes_name() {
	fn property(lt: ArbLifetime) -> bool {
		let input_without: KindInput = syn::parse_str("type Of<A>;").unwrap();
		let input_with: Result<KindInput, _> =
			syn::parse_str(&format!("type Of<'{}, A: '{}>;", lt.name, lt.name));

		match input_with {
			Ok(input) => {
				let name_without = generate_name(&input_without);
				let name_with = generate_name(&input);
				name_without != name_with
			}
			_ => true,
		}
	}

	quickcheck(property as fn(ArbLifetime) -> bool);
}

// ===========================================================================
// Property: Canonicalization Consistency
// ===========================================================================

/// Property: Canonicalizing a bound twice produces the same result.
#[test]
fn prop_canonicalization_idempotent() {
	fn property(bound: ArbTraitBound) -> bool {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound_str = &bound.name;
		let parsed: Result<TypeParamBound, _> = syn::parse_str(bound_str);

		match parsed {
			Ok(b) => {
				let canonical1 = canon.canonicalize_bound(&b);
				let canonical2 = canon.canonicalize_bound(&b);
				canonical1 == canonical2
			}
			_ => true,
		}
	}

	quickcheck(property as fn(ArbTraitBound) -> bool);
}
/// Property: Canonicalizing bounds twice produces the same result.
#[test]
fn prop_canonicalize_bounds_idempotent() {
	fn property(bounds: UniqueBounds) -> bool {
		if bounds.bounds.is_empty() {
			return true;
		}

		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		// Parse all bounds
		let parsed_bounds: Result<Vec<TypeParamBound>, _> =
			bounds.bounds.iter().map(|b| syn::parse_str(b)).collect();

		match parsed_bounds {
			Ok(parsed) => {
				let mut punctuated: Punctuated<TypeParamBound, Token![+]> = Punctuated::new();
				for b in parsed {
					punctuated.push(b);
				}

				let canonical1 = canon.canonicalize_bounds(&punctuated);
				let canonical2 = canon.canonicalize_bounds(&punctuated);
				canonical1 == canonical2
			}
			_ => true,
		}
	}

	quickcheck(property as fn(UniqueBounds) -> bool);
}

// ===========================================================================
// Property: Output Bounds Independence
// ===========================================================================

/// Property: Output bounds order doesn't affect the generated name.
#[test]
fn prop_output_bounds_order_independence() {
	fn property(
		b1: ArbTraitBound,
		b2: ArbTraitBound,
	) -> bool {
		if b1.name == b2.name {
			return true;
		}

		let input1_str = format!("type Of<A>: {} + {};", b1.name, b2.name);
		let input2_str = format!("type Of<A>: {} + {};", b2.name, b1.name);

		let result1: Result<KindInput, _> = syn::parse_str(&input1_str);
		let result2: Result<KindInput, _> = syn::parse_str(&input2_str);

		match (result1, result2) {
			(Ok(i1), Ok(i2)) => {
				let name1 = generate_name(&i1);
				let name2 = generate_name(&i2);
				name1 == name2
			}
			_ => true,
		}
	}

	quickcheck(property as fn(ArbTraitBound, ArbTraitBound) -> bool);
}

// ===========================================================================
// Property: Fn Trait Bounds
// ===========================================================================

/// A simple type for Fn bounds testing.
#[derive(Debug, Clone)]
struct ArbSimpleType {
	name: String,
}

const SIMPLE_TYPES: &[&str] = &["i32", "u32", "i64", "u64", "bool", "String", "usize", "isize"];

impl Arbitrary for ArbSimpleType {
	fn arbitrary(g: &mut Gen) -> Self {
		let idx = usize::arbitrary(g) % SIMPLE_TYPES.len();
		ArbSimpleType { name: SIMPLE_TYPES[idx].to_string() }
	}
}

/// Property: Fn bounds with same signature produce same canonical form.
#[test]
fn prop_fn_bound_determinism() {
	fn property(
		input_type: ArbSimpleType,
		output_type: ArbSimpleType,
	) -> bool {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		let bound_str = format!("Fn({}) -> {}", input_type.name, output_type.name);
		let bound: Result<TypeParamBound, _> = syn::parse_str(&bound_str);

		match bound {
			Ok(b) => {
				let canonical1 = canon.canonicalize_bound(&b);
				let canonical2 = canon.canonicalize_bound(&b);
				canonical1 == canonical2
			}
			_ => true,
		}
	}

	quickcheck(property as fn(ArbSimpleType, ArbSimpleType) -> bool);
}

// ===========================================================================
// Property: Path Preservation
// ===========================================================================

/// Property: Full paths are preserved in canonicalization.
#[test]
fn prop_path_preservation() {
	fn property() -> bool {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		// Test various paths
		let paths = vec![
			("std::fmt::Debug", "tstd::fmt::Debug"),
			("std::marker::Send", "tstd::marker::Send"),
			("core::clone::Clone", "tcore::clone::Clone"),
		];

		for (input_path, expected) in paths {
			let bound: TypeParamBound = syn::parse_str(input_path).unwrap();
			let canonical = canon.canonicalize_bound(&bound);
			if canonical != expected {
				return false;
			}
		}

		true
	}

	quickcheck(property as fn() -> bool);
}

// ===========================================================================
// Property: Empty Input Handling
// ===========================================================================

/// Property: Empty lifetimes, types, and bounds still produce valid names.
#[test]
fn prop_empty_inputs_valid() {
	fn property() -> bool {
		let input: KindInput = syn::parse_str("type Of;").unwrap();
		let name = generate_name(&input).to_string();

		// Should still be a valid name with Kind_ prefix
		name.starts_with("Kind_") && name.len() == "Kind_".len() + 16
	}

	quickcheck(property as fn() -> bool);
}

/// Property: Single lifetime with empty types and bounds is valid.
#[test]
fn prop_single_lifetime_valid() {
	fn property(lt: ArbLifetime) -> bool {
		let input_str = format!("type Of<'{} >;", lt.name);
		let result: Result<KindInput, _> = syn::parse_str(&input_str);

		match result {
			Ok(input) => {
				let name = generate_name(&input).to_string();
				name.starts_with("Kind_") && name.len() == "Kind_".len() + 16
			}
			_ => true,
		}
	}

	quickcheck(property as fn(ArbLifetime) -> bool);
}

// ===========================================================================
// Property: Type Parameter Bounds
// ===========================================================================

/// Property: Type parameters with bounds produce consistent names.
#[test]
fn prop_type_param_bounds_consistent() {
	fn property(
		type_name: ArbTypeName,
		bounds: UniqueBounds,
	) -> bool {
		if bounds.bounds.is_empty() {
			return true;
		}

		let bounds_str = bounds.bounds.join(" + ");
		let input_str = format!("type Of<{}: {}>;", type_name.name, bounds_str);

		let result1: Result<KindInput, _> = syn::parse_str(&input_str);
		let result2: Result<KindInput, _> = syn::parse_str(&input_str);

		match (result1, result2) {
			(Ok(i1), Ok(i2)) => {
				let name1 = generate_name(&i1);
				let name2 = generate_name(&i2);
				name1 == name2
			}
			_ => true,
		}
	}

	quickcheck(property as fn(ArbTypeName, UniqueBounds) -> bool);
}

// ===========================================================================
// Property: Hash Collision Resistance (Statistical)
// ===========================================================================

/// Property: Different inputs should generally produce different hashes.
///
/// Note: This is a statistical property - collisions can occur but should be rare.
#[test]
fn prop_hash_collision_resistance() {
	fn property(
		b1: ArbTraitBound,
		b2: ArbTraitBound,
	) -> bool {
		if b1.name == b2.name {
			return true; // Same input, same hash is expected
		}

		let input1_str = format!("type Of<A: {}>;", b1.name);
		let input2_str = format!("type Of<A: {}>;", b2.name);

		let result1: Result<KindInput, _> = syn::parse_str(&input1_str);
		let result2: Result<KindInput, _> = syn::parse_str(&input2_str);

		match (result1, result2) {
			(Ok(i1), Ok(i2)) => {
				let name1 = generate_name(&i1);
				let name2 = generate_name(&i2);
				// Different inputs should produce different names
				name1 != name2
			}
			_ => true,
		}
	}

	quickcheck(property as fn(ArbTraitBound, ArbTraitBound) -> bool);
}

// ===========================================================================
// Additional Edge Case Properties
// ===========================================================================

/// Property: Nested generic bounds are handled correctly.
#[test]
fn prop_nested_generics_determinism() {
	fn property() -> bool {
		let generics: Generics = parse_quote!(<>);
		let canon = Canonicalizer::new(&generics);

		// Test nested generics
		let bound: TypeParamBound = parse_quote!(Iterator<Item = Option<String>>);
		let canonical1 = canon.canonicalize_bound(&bound);
		let canonical2 = canon.canonicalize_bound(&bound);

		canonical1 == canonical2 && canonical1.contains("Iterator") && canonical1.contains("Option")
	}

	quickcheck(property as fn() -> bool);
}

/// Property: Reference types with lifetimes are canonicalized correctly.
#[test]
fn prop_reference_lifetime_canonicalization() {
	fn property() -> bool {
		let generics: Generics = parse_quote!(<'a>);
		let canon = Canonicalizer::new(&generics);

		// The implementation canonicalizes reference types, test that it's consistent
		let bound: TypeParamBound = parse_quote!(AsRef<&'a str>);
		let canonical1 = canon.canonicalize_bound(&bound);
		let canonical2 = canon.canonicalize_bound(&bound);

		canonical1 == canonical2
	}

	quickcheck(property as fn() -> bool);
}
