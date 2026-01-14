//! Traits representing type-level application to simulate higher-kinded types.
//!
//! The naming convention used by these traits is `Kind_L{n}_T{m}[_B{bounds}][_O{output}]` (De Bruijn Index Notation) where:
//! * `L{n}` represents the number of lifetimes.
//! * `T{m}` represents the number of generic types.
//! * `_B{bounds}` (optional) describes constraints on generic types.
//!   * Format: `{type_index}{constraint}`.
//!   * `type_index`: 0-based index of the type parameter.
//!   * `constraint`:
//!     * `l{lifetime_index}`: Bounded by lifetime at 0-based index (e.g., `0l0` means Type 0 is bounded by Lifetime 0).
//!     * `t{Trait}`: Bounded by a trait (e.g., `0tCopy` means Type 0 is bounded by `Copy`).
//!   * Multiple constraints are concatenated (e.g., `0l0l1` means Type 0: 'a + 'b).
//! * `_O{output}` (optional) describes constraints on the associated `Of` type.
//!   * Format: `{constraint}` (same as above, but without type index).
//!   * Example: `Ol0` means Of is bounded by Lifetime 0.
//!
//! If no bounds are present, the suffix is omitted (e.g., `Kind_L0_T1`).
//!
//! # Examples
//!
//! * `Kind_L0_T1`: 0 lifetimes, 1 type.
//! * `Kind_L1_T2`: 1 lifetime, 2 types.
//! * `Kind_L1_T1_B0l0_Ol0`: 1 lifetime, 1 type. Type 0 is bounded by Lifetime 0 (`A: 'a`). Of is bounded by Lifetime 0 (`Of: 'a`).
//! * `Kind_L2_T1_B0l0l1`: 2 lifetimes, 1 type. Type 0 is bounded by Lifetime 0 and Lifetime 1 (`A: 'a + 'b`).
//! * `Kind_L1_T2_B0l0_B1tCopy`: 1 lifetime, 2 types. Type 0 is bounded by Lifetime 0 (`A: 'a`). Type 1 is bounded by `Copy` (`B: Copy`).
//!
//! As an example of how to use these traits, the trait [`Kind_L0_T1`] would be
//! implemented by a [`Brand`][crate::brands] representing type constructors
//! with 0 lifetimes and 1 generic type. A type `Foo<A>` would have a
//! higher-kinded representation `FooBrand` which implements [`Kind_L0_T1`].

use crate::make_trait_kind;

make_trait_kind!(
	Kind_L0_T1,
	(),
	(A),
	(),
	"Trait for [brands][crate::brands] of [types][crate::types] of kind `* -> *`."
);

make_trait_kind!(
	Kind_L0_T2,
	(),
	(A, B),
	(),
	"Trait for [brands][crate::brands] of [types][crate::types] of kind `* -> * -> *`."
);

make_trait_kind!(
	Kind_L1_T0,
	('a),
	(),
	(),
	"Trait for [brands][crate::brands] of [types][crate::types] of kind `' -> *`."
);

make_trait_kind!(
	Kind_L1_T2,
	('a),
	(A, B),
	(),
	"Trait for [brands][crate::brands] of [types][crate::types] of kind `' -> * -> * -> *`."
);

make_trait_kind!(
	Kind_L1_T1_B0l0_Ol0,
	('a),
	(A: 'a),
	(: 'a),
	"Trait for [brands][crate::brands] of [types][crate::types] of kind `' -> * -> *`, with lifetime constraints.

This is the bounded variant of the 1-lifetime / 1-type Kind trait:
* The type parameter is constrained by the lifetime: `A: 'a` (`B0l0` in the name).
* The resulting associated output type is constrained by the lifetime: `Of: 'a` (`Ol0` in the name).
"
);
