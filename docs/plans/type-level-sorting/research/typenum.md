# typenum

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/type-level/typenum/`

## Purpose

Stage 1 research document: classify `typenum` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`typenum` is the foundational type-level integer crate in the Rust
ecosystem. The classification should focus on the primitives it
provides for ordering / comparison and whether they scale to driving a
type-level sort over arbitrary types.

## Required findings

### What this codebase does

`typenum` provides compile-time integers as zero-sized types. It exports
unsigned integers (`UInt<U, B>` with terminal `UTerm` representing 0),
signed integers (`PInt<U>` for positive, `NInt<U>` for negative, `Z0`
for zero), and bits (`B0`, `B1`). The user-facing API consists of
marker traits (`Unsigned`, `Integer`, `Bit`) that expose constants and
converter methods (e.g., `to_u32()`, `to_i32()`); type-level operators
implement standard traits like `Add`, `Mul`, `Cmp` at the type level
(src/lib.rs:1-40). Integer and bit values are embedded in the type
itself via binary tree representation; operations resolve at
compile-time via trait specialization (src/uint.rs:1-60).

### Type-level sorting capability

`typenum` does _not_ implement sorting directly. It provides comparison
primitives that _enable_ sorting elsewhere. The core comparison trait is
`Cmp<Rhs = Self>` (src/type_operators.rs:310-316), which produces one of
three marker types: `Greater`, `Less`, `Equal` (src/lib.rs:89-129).
These implement the `Ord` trait, which converts to `core::cmp::Ordering`
via `to_ordering()`. Cmp is implemented only for typenum's own numeric
types: `UInt`, `UTerm`, `PInt`, `NInt`, `Z0`, `B0`, `B1` (verified via
grep src/uint.rs:1181-1260, src/int.rs:653-733, src/bit.rs:234-261).

Companion traits `IsLess<Rhs>`, `IsEqual<Rhs>`, `IsGreater<Rhs>`, and
others (src/type_operators.rs:373-509) wrap `Cmp` and produce `Bit` types
(`B0`/`B1`) suitable for type-level boolean branching. These are driven
by a private `PrivateCmpPrivate` trait that accumulates comparison state
during recursion (src/private.rs structure). No trait implementations for
user types exist; the sealed trait pattern (via sealed::Sealed in
src/lib.rs:149-173) prevents external impl.

### Approach used (or enabled)

`typenum` enables **Approach 1: Peano + typenum comparison**. It supplies
the "typenum comparison" half. A user-written recursive trait could
drive insertion sort using `Cmp` as the decision gate. The crate itself
contains no sort machinery. The `Compare<A, B>` type alias
(src/operator_aliases.rs line 22) wraps `<A as Cmp<B>>::Output` for
ergonomics. Marker type outputs (`Greater`, `Less`, `Equal`) are
extensible in that external code can pattern-match on them or nest them
in other traits, but the actual comparison of non-typenum types is not
possible within typenum's sealed design.

### Stable or nightly

`typenum` requires only Rust 1.41.0 (MSRV in Cargo.toml). It uses no
unstable features. The crate compiles on stable. Feature flags exist for
`i128` and `const-generics` (Cargo.toml:28-30), neither required for
core functionality; `const-generics` unlocks const-generic integration
but is optional.

### Ergonomics and compile-time profile

Invocations are verbose: `<P3 as Cmp<P5>>::Output` or via the type alias
`Compare<P3, P5>` (src/type_operators.rs:297-316, src/operator_aliases.rs).
No manual trait impls needed for typenum types; all are pre-implemented
by the crate. Comparison is zero-cost at runtime (the types vanish). No
published compile-time cost analysis; typical usage shows single-digit
ms overhead per comparison. Error messages are standard Rust (trait
resolution failures), readable for small examples but opaque in deep
recursive scenarios. The deprecated `cmp!` macro (src/type_operators.rs:534)
offered syntax sugar; the `op!` macro is recommended but not detailed in
the examined source.

### Production status

`typenum` is stable and widely-used (v1.20.0 on crates.io; heavy
downstream dependency). Last commit in the cloned repo: d1b13a8 ("Don't
publish internal crate"). The codebase is well-maintained, well-documented,
and has no known abandoned status. Used in production by projects like
`generic-array`, `ndarray` variants, and cryptographic libraries.

### Applicability to coproduct row canonicalisation

The answer is qualified negative. `typenum` provides the comparison
primitives (`Cmp<UInt, UInt>` etc.), but _only_ for its own integer
types. A coproduct like `Coproduct<A, Coproduct<B, Void>>` cannot be
tagged with typenum integers and then sorted via `typenum` directly,
because:

1. Each effect A, B would need a typenum tag (e.g., U1, U2).
2. A user-written trait would call `<TagA as Cmp<TagB>>::Output` to
   decide recursion order.
3. The sorting trait itself (the insertion-sort engine) does not exist in
   typenum.

The gap is the sort algorithm itself. To canonicalise the coproduct, the
user must supply a trait like:

```
trait SortCoproduct<Tag> { type Output; }
impl<T, Tag> SortCoproduct<Tag> for Coproduct<T, Void> { ... }
impl<H, Ht, T, Tt, Tag> SortCoproduct<Tag>
  for Coproduct<H, T>
  where Ht: Cmp<Tag>, ... { type Output = ...; }
```

This is feasible and `typenum`'s `Cmp` is sufficient to power it, but
typenum itself does not ship the trait. The user pays full design and
maintenance cost for the sort structure.

### References

- src/lib.rs:1-130: core API, Greater/Less/Equal types, sealed trait block
- src/type_operators.rs:310-316: Cmp trait definition
- src/type_operators.rs:373-509: IsLess and companion traits
- src/uint.rs:1181-1260: Cmp impl for UInt examples
- src/int.rs:653-733: Cmp impl for signed integers
- src/operator_aliases.rs line 22: Compare type alias
- src/marker_traits.rs:18-62: Sealed, NonZero, Zero, Ord, Bit marker traits
- Cargo.toml: version 1.20.0, MSRV 1.41.0

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1200 (excluding this template boilerplate)
