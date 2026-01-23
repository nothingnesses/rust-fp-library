//! Name generation for `Kind` traits.
//!
//! This module handles the generation of unique, deterministic identifiers
//! for `Kind` traits based on their signature. It uses `rapidhash` to create
//! a collision-resistant hash of the canonical signature.

use crate::{canonicalize::Canonicalizer, parse::KindInput};
use quote::format_ident;
use syn::{GenericParam, Ident};

// Deterministic hashing setup
// Using a fixed seed for reproducibility across builds
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
pub fn generate_name(input: &KindInput) -> Ident {
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
						let bounds_str = canon.canonicalize_bounds(&ty.bounds);
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
			let bounds_str = canon.canonicalize_bounds(&assoc.output_bounds);
			parts.push(format!("O{bounds_str}"));
		}

		canonical_parts.push(parts.join("_"));
	}

	let canonical_repr = canonical_parts.join("__");

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

	/// Tests that identical inputs produce identical `Kind` trait names.
	#[test]
	fn test_generate_name_determinism() {
		let input1 = parse_kind_input("type Of<'a, A: 'a>: 'a;");
		let name1 = generate_name(&input1);

		let input2 = parse_kind_input("type Of<'a, A: 'a>: 'a;");
		let name2 = generate_name(&input2);

		assert_eq!(name1, name2);
		assert!(name1.to_string().starts_with("Kind_"));
	}

	/// Tests that different inputs produce different `Kind` trait names.
	#[test]
	fn test_generate_name_different_inputs() {
		let input1 = parse_kind_input("type Of<'a, A: 'a>: 'a;");
		let name1 = generate_name(&input1);

		let input2 = parse_kind_input("type Of<A>;");
		let name2 = generate_name(&input2);

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
		let name1 = generate_name(&input1);

		let input2 = parse_kind_input(
			"
			type SendOf<U>: Send;
			type Of<'a, T>: Display;
		",
		);
		let name2 = generate_name(&input2);

		assert_eq!(name1, name2);
	}

	/// Tests name generation with complex bounded types.
	#[test]
	fn test_generate_name_complex_bounds() {
		let input = parse_kind_input("type Of<'a, A: Clone + Send>: Clone + Send;");
		let name = generate_name(&input);

		assert!(name.to_string().starts_with("Kind_"));

		// Ensure determinism
		let input2 = parse_kind_input("type Of<'a, A: Clone + Send>: Clone + Send;");
		let name2 = generate_name(&input2);
		assert_eq!(name, name2);
	}

	/// Tests that bound order doesn't affect the generated name.
	#[test]
	fn test_generate_name_bound_order_independence() {
		let input1 = parse_kind_input("type Of<A: Clone + Send>;");
		let input2 = parse_kind_input("type Of<A: Send + Clone>;");

		let name1 = generate_name(&input1);
		let name2 = generate_name(&input2);

		assert_eq!(name1, name2);
	}
}
