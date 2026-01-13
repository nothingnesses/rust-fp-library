//! Convenience type aliases for the [`Kind` traits][crate::hkt::kinds].
//!
//! The naming convention used by these aliases is `Apply_L{n}_T{m}[_B{bounds}][_O{output}]` (De Bruijn Index Notation) where:
//! * `L{n}` represents the number of lifetimes.
//! * `T{m}` represents the number of generic types.
//! * `_B{bounds}` (optional) describes constraints on generic types.
//!   * Format: `{type_index}{constraint}`.
//!   * `type_index`: 0-based index of the type parameter.
//!   * `constraint`:
//!     * `l{lifetime_index}`: Bounded by lifetime at 0-based index (e.g., `0l0` means Type 0 is bounded by Lifetime 0).
//!     * `t{Trait}`: Bounded by a trait (e.g., `0tCopy` means Type 0 is bounded by `Copy`).
//!   * Multiple constraints are concatenated (e.g., `0l0l1` means Type 0: 'a + 'b).
//! * `_O{output}` (optional) describes constraints on the associated `Output` type.
//!   * Format: `{constraint}` (same as above, but without type index).
//!   * Example: `Ol0` means Output is bounded by Lifetime 0.
//!
//! If no bounds are present, the suffix is omitted (e.g., `Apply_L0_T1`).
//!
//! If a [`Brand`][crate::brands] `FooBrand` for concrete type `Foo<A>`
//! implements the [`Kind_L0_T1`] trait, then `Apply_L0_T1<FooBrand, ()>`
//! represents `Foo<()>`.

use crate::{
	hkt::{Kind_L0_T1, Kind_L0_T2, Kind_L1_T0, Kind_L1_T1_B0l0_Ol0, Kind_L1_T2},
	make_type_apply,
};

make_type_apply!(
	Apply_L0_T1,
	Kind_L0_T1,
	(),
	(A),
	"Alias for [types][crate::types] of kind `* -> *`."
);

make_type_apply!(
	Apply_L0_T2,
	Kind_L0_T2,
	(),
	(A, B),
	"Alias for [types][crate::types] of kind `* -> * -> *`."
);

make_type_apply!(
	Apply_L1_T0,
	Kind_L1_T0,
	('a),
	(),
	"Alias for [types][crate::types] of kind `' -> *`."
);

make_type_apply!(
	Apply_L1_T2,
	Kind_L1_T2,
	('a),
	(A, B),
	"Alias for [types][crate::types] of kind `' -> * -> * -> *`."
);

make_type_apply!(
	Apply_L1_T1_B0l0_Ol0,
	Kind_L1_T1_B0l0_Ol0,
	('a),
	(A: 'a),
	"Alias for [types][crate::types] of kind `' -> * -> *`, with lifetime constraints.

This is the `Apply` alias corresponding to [`Kind_L1_T1_B0l0_Ol0`]. The name encodes:
* `B0l0`: `A: 'a`
* `Ol0`: `Output: 'a`
"
);
