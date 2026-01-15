//! Name generation for Kind traits.
//!
//! This module handles the generation of unique, deterministic identifiers
//! for Kind traits based on their signature. It uses `rapidhash` to create
//! a collision-resistant hash of the canonical signature.

use crate::{canonicalize::Canonicalizer, parse::KindInput};
use quote::format_ident;
use syn::Ident;

// Deterministic hashing setup
// Using a fixed seed for reproducibility across builds
const RAPID_SECRETS: rapidhash::v3::RapidSecrets =
	rapidhash::v3::RapidSecrets::seed(0x1234567890abcdef);

fn rapidhash(data: &[u8]) -> u64 {
	rapidhash::v3::rapidhash_v3_seeded(data, &RAPID_SECRETS)
}

/// Generates a unique, deterministic identifier for a Kind trait based on its input signature.
///
/// The name format is `Kind_{hash}`, where `{hash}` is a 16-character hexadecimal string
/// representing the 64-bit hash of the canonical signature.
pub fn generate_name(input: &KindInput) -> Ident {
	let canon = Canonicalizer::new(&input.lifetimes, &input.types);

	let l_count = input.lifetimes.len();
	let t_count = input.types.len();

	let mut canonical_parts = vec![format!("L{}", l_count), format!("T{}", t_count)];

	// Type bounds
	for (i, ty) in input.types.iter().enumerate() {
		if !ty.bounds.is_empty() {
			let bounds_str = canon.canonicalize_bounds(&ty.bounds);
			canonical_parts.push(format!("B{}{}", i, bounds_str));
		}
	}

	// Output bounds
	if !input.output_bounds.is_empty() {
		let bounds_str = canon.canonicalize_bounds(&input.output_bounds);
		canonical_parts.push(format!("O{}", bounds_str));
	}

	let canonical_repr = canonical_parts.join("_");

	// Always use hash for consistency and to avoid length issues
	let hash = rapidhash(canonical_repr.as_bytes());
	format_ident!("Kind_{:016x}", hash)
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Helper function to parse a KindInput from a string.
	fn parse_kind_input(input: &str) -> KindInput {
		syn::parse_str(input).expect("Failed to parse KindInput")
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
}
