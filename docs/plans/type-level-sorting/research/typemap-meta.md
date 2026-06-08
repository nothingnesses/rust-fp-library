# typemap-meta

**Status:** complete
**Last updated:** 2026-04-25
**Codebase location:** `/home/jessea/Documents/projects/type-level/typemap-meta/`

## Purpose

Stage 1 research document: classify `typemap-meta` against the ten
type-level approaches catalogued in [README.md](README.md), focusing
on whether its derive-macro-driven type-to-value map provides a
genuinely type-level lookup (approach 10) or reduces to runtime
TypeId dispatch (approach 6). The discovery survey flagged this
codebase as "compile-time lookup with generics + proc macro", which
suggests it might be the most type-level of the hash-map family.

## Required findings

### What this codebase does

`typemap-meta` provides a proc-macro derive that generates `Get<T>` and `GetMut<T>` trait implementations for tuple structs. A user declares a tuple struct of heterogeneous types and derives `Typemap`; the macro (lines 31-62 of `typemap-meta-derive/src/lib.rs`) iterates over tuple fields, extracts each field's type, and generates one `impl Get<T>` block per field. The user then calls the `get!` or `get_mut!` convenience macros (lines 58-71 of `typemap-meta/src/lib.rs`) to retrieve a field by type: `get!(struct_instance, SomeType)` expands to `Get::<SomeType>::get(&struct_instance)`, which returns `&SomeType` or a compile error if the type is not a field.

### Type-level vs runtime keying

The lookup is _entirely compile-time via trait resolution_. The proc macro generates monomorphic trait impls with concrete types at line 40-44 of `typemap-meta-derive/src/lib.rs`:

```rust
impl #generics Get<#types> for #name #generics {
    fn get(&self) -> &#types {
        &self.#indices
    }
}
```

Each `impl Get<ConcreteType> for StructName` is a distinct trait impl with the target type baked in. Trait resolution at the call site selects the correct impl; if a requested type is not a field, the compiler produces a trait-resolution error. There is zero runtime dispatch, no `TypeId`, no dynamic lookup. The `get!` macro is pure syntactic sugar; all work happens at monomorphization time.

### Approach used (or enabled)

**Approach 10 (type-level hash-map)**, but in an extremely restrictive form. The "map" is compile-time-only and finite: it is the set of fields declared in the struct. This is a type-level index, not a type-level hash-map in the sense of supporting dynamic insertion or flexible key spaces. No approach 6 runtime machinery; no approach 3 hash tags. This is genuine type-level deduplication via trait dispatch, achieved through proc-macro code generation, not sorting.

### Stable or nightly

Fully stable. Edition 2021. No feature gates, no unstable library features. MSRV is not documented in Cargo.toml; the dependencies (`syn` 1.0, `quote` 1.0) have stable MSRV of Rust 1.31+, but the codebase's own floor is not tested. The macro uses only basic `syn` and `quote` AST manipulation, which has been stable for years.

### Ergonomics and compile-time profile

User defines a map by declaring a tuple struct:

```rust
#[derive(Typemap)]
struct Config(DbConnection, Logger, Settings);
```

All values are _static_ (fixed at compile time in the struct definition). The derive macro then generates trait impls at expansion time; macros 40-62 of `typemap-meta-derive/src/lib.rs` shows a single pass over fields, generating code proportional to field count. Compilation cost is O(N) in field count, with zero runtime cost beyond struct field layout. The documentation makes no mention of measured compile-time overhead and claims only "compile-time safety and faster execution".

### Production status

Active and published. Repository at `https://github.com/enlightware/typemap-meta`. Version 0.2.0 on Crates.io (MIT OR Apache-2.0). Latest commit is "Updated documentation", suggesting maintenance but not recent active feature work. The codebase is small (68 lines of derive logic, ~150 lines of lib), well-tested (comprehensive test cases in `src/lib.rs` lines 74-495), and released under permissive dual license.

### Applicability to coproduct row canonicalisation

`typemap-meta` _cannot_ deduplicate or canonicalise a coproduct row. The lookup index is compile-time-finite: you can only retrieve a type if it was declared as a field at struct definition. If you have `Coproduct<A, Coproduct<B, Void>>` and `Coproduct<B, Coproduct<A, Void>>`, they are different struct types; you cannot "merge" them into a single typemap. The macro solves a different problem: fast, type-safe access to a fixed heterogeneous collection, assuming no duplicates and no reordering. For row canonicalisation you would need either recursive type merging (not present here) or runtime deduplication (approach 6, which this codebase explicitly rejects). The struct must be declared with exact field order and composition up front.

### References

- `typemap-meta-derive/src/lib.rs` lines 8-16: macro entry and dispatch
- `typemap-meta-derive/src/lib.rs` lines 31-62: core code generation loop
- `typemap-meta/src/lib.rs` lines 44-71: trait defs and macros
- `typemap-meta/src/lib.rs` lines 74-495: comprehensive monomorphic test coverage
- `Cargo.toml` (root): workspace structure
- `typemap-meta/Cargo.toml`: v0.2.0, edition 2021, no features, no nightly gates
- README.md: explicit contrast with runtime-TypeId crates (`typemap`, `type-map`)

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1200 (excluding this template boilerplate)
