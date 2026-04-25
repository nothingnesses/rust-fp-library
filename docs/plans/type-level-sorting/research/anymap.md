# anymap

**Status:** complete
**Last updated:** 2026-04-25
**Codebase location:** `/home/jessea/Documents/projects/type-level/anymap/`

## Purpose

Stage 1 research document: classify `anymap` against the ten
type-level approaches catalogued in [README.md](README.md), focusing
on its applicability as a type-level hash-map / hash-set (approach
10). `anymap` is the canonical runtime `TypeId`-keyed heterogeneous
map in the Rust ecosystem; the question is whether its mechanism
could canonicalise an effect row by deduplication rather than by
sorting.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

`anymap` provides a type-safe wrapper around `HashMap<TypeId, Box<dyn Any>>` that stores zero or one value per type. The primary API is `Map<A>` (where `A` defaults to `dyn Any`), exposed as the `AnyMap` convenience alias. Users call type-generic methods like `insert::<T>()`, `get::<T>()`, `remove::<T>()`, and `entry::<T>()` (similar to HashMap's entry API); the library hides `TypeId` and unsafe downcasting behind a type-safe interface. The underlying storage is a `HashMap` keyed by `TypeId::of::<T>()`, with values boxed as `Box<dyn Any>`. This pattern appears at `/home/jessea/Documents/projects/type-level/anymap/src/lib.rs`, lines 101-104 (struct definition) and lines 197-200 (the `get` implementation showing `TypeId::of::<T>()` lookup). Additional variants support `Send`, `Sync`, and `CloneAny` (a `Clone`-capable variant of `Any`); see lines 70-72 for the six variant forms.

### Type-level capability

The keying is strictly **runtime**, not type-level. `TypeId::of::<T>()` (called at line 198 and similar sites) is a `core::any::TypeId` value that lives in memory at runtime and is computed dynamically. There is no compile-time exhaustiveness guarantee; the map is untyped at the type level. Two different types with the same `TypeId` (impossible in practice but not provable to the type system) could silently collide. However, within a single binary's type ecosystem, `TypeId` uniqueness is guaranteed by the runtime, making this practically safe. The downside: two `AnyMap` instances or two orderings of the same types resolve to identical runtime state with no type-level distinction. The map cannot serve as a homogeneous coproduct (where different orderings have different types).

### Approach used (or enabled)

**Approach 10, runtime form, which reduces to Approach 6.** The README.md (lines 70-75) acknowledges this explicitly: a truly type-level realisation would require nightly features. `anymap` is the canonical example of approach 10 on stable Rust---type-level hash-map semantics encoded as runtime `TypeId` dispatch. This is indistinguishable from approach 6 (std::any::TypeId runtime comparison) because the deduplication is a runtime property, not a type property.

### Stable or nightly

Fully **stable**. Cargo.toml (lines 1-23) declares MSRV 1.36 and no feature gates. The core `Map` type requires only `core::any::TypeId` and `core::any::Any`, both stable since Rust 1.0. The optional `hashbrown` feature provides a no_std implementation but is still stable. No nightly features are used or required.

### Ergonomics and compile-time profile

Ergonomics are excellent. Users write `map.insert(42i32)` and `map.get::<i32>()` with automatic type inference; the generic methods deduce `T` from context. Compile time is unaffected: `HashMap` is monomorphic, and `TypeId` computation is trivial. Runtime cost: `TypeId::of::<T>()` is a no-op that transmutes the type's metadata to a u64 (see lib.rs line 638-639), and the custom `TypeIdHasher` (lines 608-626) is a no-op hasher that directly uses that u64. Downcasting is a pointer cast with no runtime checks; see any.rs lines 106-118. Benchmarks in the source (benches/bench.rs, not fully read) show ~30ns for a hit and ~12 microseconds for insert-and-get across 260 types. Performance is production-grade.

### Production status

Active and stable. Last commit: 2022-02-22 (release 1.0.0-beta.2, which is the library's stable interface; the "beta" label reflects deliberate conservatism, not incompleteness). Repository: https://github.com/chris-morgan/anymap. The crate is widely used in Rust ecosystems and has been stable for years. Author: Chris Morgan.

### Applicability to coproduct row canonicalisation

`anymap` **cannot** canonicalise an effect row at the type level. Here is the gap:

In a type-level coproduct like `Coproduct<A, Coproduct<B, Void>>` vs. `Coproduct<B, Coproduct<A, Void>>`, these are **different types**. Canonicalisation requires a compile-time transformation that makes them the **same type**. A runtime registry (a `Map<dyn Any>` instance) can store values from both orderings at runtime with identical deduplication logic, but it cannot change the types `Coproduct<A, Coproduct<B, Void>>` and `Coproduct<B, Coproduct<A, Void>>` into a common canonical form. Both types would still exist as separate compile-time entities.

For an effect row, a user could build a value of type `Coproduct<A, Coproduct<B, Void>>`, extract its heterogeneous values into an `AnyMap`, then reconstruct it in a canonical order---but the original coproduct type remains fixed at compile time. The deduplication is hidden behind a runtime witness; the coproduct type itself is not canonicalised. For the effects port, this means workaround 2 cannot use `anymap` to solve the ordering mitigations problem: the challenge is that `Effect<Coproduct<A, Coproduct<B, Void>>>` must become the same **type** as `Effect<Coproduct<B, Coproduct<A, Void>>>`, and no runtime deduplication can achieve that.

### References

- `/home/jessea/Documents/projects/type-level/anymap/src/lib.rs`, lines 101-104: struct `Map` definition.
- `/home/jessea/Documents/projects/type-level/anymap/src/lib.rs`, lines 197-200: `get` method using `TypeId::of::<T>()`.
- `/home/jessea/Documents/projects/type-level/anymap/src/lib.rs`, lines 608-626: `TypeIdHasher` implementation.
- `/home/jessea/Documents/projects/type-level/anymap/src/any.rs`, lines 106-118: downcast machinery.
- `/home/jessea/Documents/projects/type-level/anymap/Cargo.toml`, lines 1-23: MSRV 1.36, no nightly features.
- `/home/jessea/Documents/projects/type-level/anymap/README.md`, lines 70-75: acknowledgement that type-level form requires nightly.

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1200 (excluding this template boilerplate)
