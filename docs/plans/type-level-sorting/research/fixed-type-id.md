# fixed-type-id

**Status:** complete
**Last updated:** 2026-04-25
**Codebase location:** `/home/jessea/Documents/projects/type-level/fixed-type-id/`

## Purpose

Stage 1 research document: classify `fixed-type-id` against the ten
type-level approaches catalogued in [README.md](README.md), focusing
on whether its hash mechanism produces a type-level result (approach 9) or a runtime constant (approach 3, already covered by `stabby` and
`type-uuid`).

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

`fixed-type-id` provides a `FixedTypeId` trait that assigns a stable,
compile-time constant identifier to types via rapidhash. The trait
defines three associated constants: `const TYPE_NAME: &'static str`
(fixed_type_id/src/lib.rs:202), `const TYPE_ID: FixedId` (line 207),
and `const TYPE_VERSION: FixedVersion` (line 211). The `FixedId`
struct wraps a single `u64` (line 49). A proc-macro `fixed_type_id!`
generates implementations by accepting a type name and optional
version tuple; the macro expands to trait impl blocks with const values
(fixed_type_id_macros/src/lib.rs:68-71). The `FixedId::from_type_name`
const function hashes a type name string and optional version bytes
via `rapidhash::rapidhash()` (lines 79-96), computing a u64 at
compile-time. For generics, users manually specify concrete type
parameters: `fixed_type_id! { A<u8>; A<u16>; }` (fixed_type_id/src/lib.rs:460-463).

### Where the hash result lives

The hash result is a **runtime constant `u64`**, not a type-level
quantity. `FixedId::from_type_name()` is a `const fn` that returns
`FixedId(hash)` where `hash` is computed by `rapidhash()` (lines 79-96).
The result is embedded in the `TYPE_ID` associated constant as a
`FixedId` newtype wrapping u64. This is identical in principle to
`stabby` and `type-uuid`: the hash is available at compile time but
cannot parameterize types or drive trait dispatch. The `FixedId` type
is `#[repr(transparent)]` over `pub u64` (line 49) and implements
`Copy`, `Clone`, `Eq`, `PartialEq`, `Ord` (line 48), but these traits
operate on the runtime value, not on a type-level encoding. There is
no mechanism to lift the u64 into a const generic parameter, a typenum
integer, or any type-level quantity that would enable type-level
comparisons.

### Approach used (or enabled)

This is **approach 3 (hash-based runtime tagging)**, identical to
`stabby` and `type-uuid` in mechanism. The only distinction from
`type-uuid` is that `fixed-type-id` additionally supports semantic
versioning: users can attach a `FixedVersion` tuple (major, minor,
patch) to types and hash it into the ID (lines 82-93), allowing the
same type name to have different IDs across versions. This is a
**runtime encoding of version information**, not a type-level feature.
`fixed-type-id` does not implement or enable any other approach.
Approach 9 (type-level hashing with a type result) is not present: the
crate produces no types, only u64 values. The proc-macro generates
const assignments, not type families.

### Stable or nightly

The crate **requires nightly Rust**. The main library enables four
feature gates: `#![feature(str_from_raw_parts)]` (line 4),
`#![feature(generic_const_exprs)]` (line 5),
`#![feature(nonzero_internals)]` (line 6), and conditionally
`#![feature(specialization)]` (line 7). The rust-toolchain.toml
specifies `channel = "nightly"` (rust-toolchain.toml:2). The
`generic_const_exprs` feature is used for const type-name generation
with fixed-size arrays (see `CONST_TYPENAME_LEN` configuration and the
`const_create_from_str_slice` call in fixed_type_id/src/lib.rs:392-393
and fixedstr-ext dependency). The crate will not compile on stable.

### Ergonomics and compile-time profile

Users opt types in via the `fixed_type_id!` macro, declaring a type
name and optional version: `fixed_type_id! { #[version((0,1,0))] MyType; }`
(fixed_type_id_macros/src/lib.rs:68-71). For generic types, users must
list each instantiation: `fixed_type_id! { A<u8>; A<u16>; }` (tests in
lib.rs:460-463). The macro is keyword-driven; attributes like
`#[version(x,y,z)]`, `#[omit_version_hash]`, `#[equal_to(OtherType)]`,
and `#[random_id]` control behavior (fixed_type_id/README.md:212-216).
Compile-time cost is minimal: each `fixed_type_id!` call triggers
macro expansion and one `rapidhash()` invocation per type, both O(1)
relative to build time. The version tuple is hashed into the ID by
mixing name-hash and version-hash via `rapid_mix()` (lines 82-93);
enabling `#[omit_version_hash]` skips the version hash, producing
identical IDs for the same name across versions.

### Production status

The crate is **active and experimental**. Published to crates.io as
version 0.2.0 (fixed_type_id/Cargo.toml:3), with repository
https://github.com/c00t/fixed-type-id (line 8). The design is mature
for a nightly-only crate: trait is stable, macros are well-specified,
version support is intended production-quality. However, dependency on
multiple unstable feature gates and nightly-only toolchain limits
practical adoption. The checked-out version has comprehensive tests
(fixed_type_id/src/lib.rs:418-668) covering generics, versions, and
macro equivalence; no evidence of abandonment, but no recent release
date is visible in provided files.

### Applicability to coproduct row canonicalisation

**Verdict: not applicable, same limitation as `stabby` and `type-uuid`.**
The hash is a runtime u64 constant, not a type. To canonicalise
`Coproduct<A, Coproduct<B, Void>>` and `Coproduct<B, Coproduct<A, Void>>`,
one would need to (1) assign `FixedId`s to each type, (2) extract and
compare the u64 values, (3) reorder the coproduct at runtime. There is
no trait mechanism to ask "is `A::ID < B::ID`?" and dispatch to ordered
variants at compile time. The u64 cannot be a const generic parameter on
stable Rust; even on nightly, `const PARAM: u64` in trait bounds cannot
drive specialization without additional machinery (approach 8 const generics
plus custom comparison traits). Version support is a win for metadata but
irrelevant to canonicalisation. Adopting `fixed-type-id` would add ~5 lines
per type but would require a parallel approach (8 or 5) to achieve ordering.

### References

- **FixedTypeId trait:** /home/jessea/Documents/projects/type-level/fixed-type-id/fixed_type_id/src/lib.rs:195-230
- **FixedId struct and from_type_name():** fixed_type_id/src/lib.rs:49, 79-96
- **Macro definition:** fixed_type_id_macros/src/lib.rs:68-71
- **Version hashing (rapid_mix):** fixed_type_id/src/lib.rs:176-186, 82-93
- **Generic type tests:** fixed_type_id/src/lib.rs:456-631
- **README usage examples:** fixed_type_id/README.md:24-166
- **Crate metadata:** fixed_type_id/Cargo.toml:1-41
- **Nightly features required:** fixed_type_id/src/lib.rs:1-7, rust-toolchain.toml:1-2

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1200 (excluding this template boilerplate)
