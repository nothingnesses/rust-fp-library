# type-uuid

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/type-level/type-uuid/`

## Purpose

Stage 1 research document: classify `type-uuid` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`type-uuid` assigns compile-time stable UUID constants to types. The
classification should determine whether its UUID-as-tag mechanism is
type-level (drivable by trait resolution) or only runtime (drivable by
matching at runtime).

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

`type-uuid` provides a `TypeUuid` trait that assigns a stable, compile-time `const UUID: Bytes` constant to any type. The trait is implemented via a proc-macro derive (`#[derive(TypeUuid)]`) that accepts a UUID string attribute (`#[uuid = "..."]`) and expands it into a 16-byte array at compile time (type-uuid-derive/src/lib.rs, lines 8-61). Users annotate types like `#[derive(TypeUuid)] #[uuid = "d4adfc76-..."] struct MyType;` and the macro generates `impl TypeUuid for MyType { const UUID: Bytes = [...]; }`. The `Bytes` type is a fixed alias `[u8; 16]` (src/lib.rs, line 42). A sealed trait `TypeUuidDynamic` allows trait-object access to UUIDs at runtime (src/lib.rs, lines 71-79), but the primary API is the const `UUID` associated constant.

### Type-level sorting capability

The UUID is stored as a `const [u8; 16]` associated constant on the `TypeUuid` trait (src/lib.rs, line 61). This makes it available at compile time as a const item, not merely runtime metadata. However, it cannot be used directly in trait dispatch or generic const parameters because Rust does not yet support using `[u8; 16]` arrays as const generic discriminators in stable Rust (const generic parameters must be scalar types: integers, bools, or chars). The UUID could theoretically be converted to a `u128` by the proc-macro, but the codebase does not do this, and there is no mechanism to lift the UUID into a typenum tag. The UUID exists at compile time but is isolated from the type system; it cannot drive an ordering comparison in trait bounds.

### Approach used (or enabled)

This aligns partially with approach 3 (hash-based tagging) in that it assigns a stable, unique identifier to each type. However, unlike a true hash or const generic sort, the UUID is not parameterized into the type signature; it is an associated constant. The mechanism is closer to approach 6 (TypeId-equivalent) because the UUID is a runtime-queryable tag accessible via the sealed trait. It does not implement any of the other approaches (1 Peano+typenum, 2 proc-macro ordering, 4 adt_const_params, 5 specialization, 7 marker-trait inequality, 8 const generics+const fn). The derive macro is approach-2-adjacent but does not generate sorting logic.

### Stable or nightly

The crate targets stable Rust (edition 2018, Cargo.toml line 5) and requires no feature gates or nightly compiler. The `TypeUuid` trait and the derive macro work on stable without restrictions. The proc-macro uses standard `quote!` and `syn` machinery. No MSRV is documented; the crate targets 2018 edition, implying Rust 1.31+.

### Ergonomics and compile-time profile

Users assign UUIDs by adding a `#[derive(TypeUuid)]` and `#[uuid = "..."]` attribute to a type. The proc-macro converts the string UUID to a byte array at compile time (type-uuid-derive/src/lib.rs, lines 47-51) and inlines it into the impl. For external types (like primitives), a `external_type_uuid!` macro is provided (lines 77-97). The crate pre-implements `TypeUuid` for all standard library primitives (src/lib.rs, lines 88-136). No cost analysis is documented; the overhead is negligible (a const 16-byte array per type). The design is straightforward: one attribute per type.

### Production status

The crate is stable and minimally maintained. Version 0.1.2 is published on crates.io; the repository is https://github.com/randomPoison/type-uuid. The code is clean and complete. No recent activity is documented in the provided files, but there is no evidence of abandonment; the crate is feature-complete and requires no ongoing work. It is suitable for production use in contexts where stable type identification is needed.

### Applicability to coproduct row canonicalisation

`type-uuid` cannot drive a type-level sort of coproducts because the UUID constant cannot be used in trait dispatch or generic const parameters. Canonicalising `Coproduct<A, Coproduct<B, Void>>` and `Coproduct<B, Coproduct<A, Void>>` requires comparing types at the type level (in trait bounds) to decide which branch comes first. While a proc-macro could theoretically assign UUIDs and generate a total order, this crate provides no such mechanism. The UUID is a runtime tag, not a type-level comparator. To use `type-uuid` for sorting, one would need to: (1) assign UUIDs to each type, (2) extract the UUID at runtime, (3) compare them, and (4) dynamically dispatch to the correct sorted variant. This defeats the purpose of type-level canonicalisation, which aims to produce a single canonical type at compile time, not a decision tree at runtime. The gap is fundamental: a const array cannot parameterize a type; only scalars (integers, bools) can.

### References

- **TypeUuid trait definition:** src/lib.rs, line 60-62
- **Bytes type alias:** src/lib.rs, line 42
- **Proc-macro derive implementation:** type-uuid-derive/src/lib.rs, lines 8-61
- **Runtime trait object access:** src/lib.rs, lines 71-79
- **Primitive type implementations:** src/lib.rs, lines 88-136
- **Example usage:** examples/derive.rs, lines 1-11
- **Crate metadata:** Cargo.toml, lines 1-21

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1200 (excluding this template boilerplate)
