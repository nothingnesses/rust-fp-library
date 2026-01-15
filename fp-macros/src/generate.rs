//! Name generation for Kind traits.
//!
//! This module handles the generation of unique, deterministic identifiers
//! for Kind traits based on their signature. It uses `rapidhash` to create
//! a collision-resistant hash of the canonical signature.

use crate::canonicalize::Canonicalizer;
use crate::parse::KindInput;
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
