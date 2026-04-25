# rust-typemap

**Status:** complete
**Last updated:** 2026-04-25
**Codebase location:** `/home/jessea/Documents/projects/type-level/rust-typemap/`

## Purpose

Stage 1 research document: classify `rust-typemap` against the ten
type-level approaches catalogued in [README.md](README.md), focusing
on its position in the type-level hash-map / hash-set family
(approach 10). rust-typemap is the canonical type-keyed heterogeneous
map in the Rust ecosystem (alongside `anymap`); the question is
whether it differs from anymap in any way that matters for coproduct
canonicalisation.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

rust-typemap provides a `TypeMap<A>` struct that stores one value per type, keyed by the type itself rather than by an explicit runtime key. The core API uses a `Key` trait with an associated `Value` type (src/lib.rs:80-83): `pub trait Key: Any { type Value: Any; }`. Users define marker types implementing `Key`, assigning the value type via the associated type. Methods `insert`, `get`, `get_mut`, `remove`, and `entry` take a generic parameter `K: Key` and operate on the stored value of type `K::Value`. The underlying storage is a `HashMap<TypeId, Box<A>>` where `A` is a trait bound (typically `Any` or `Any + Send + Sync`), allowing value-type variation within bounds.

### Type-level vs runtime keying

The keying is **fundamentally runtime**, despite the presence of the `Key` trait. The associated `Value` type is resolved at compile time by trait coherence (the compiler looks up the impl of `Key for KeyType` and extracts the `type Value` bound), but the actual lookup is a `TypeId::of::<K>()` hash into a runtime HashMap (src/lib.rs:106, 114, 122, 129, 137). The value type resolution is erased; dispatch on the `Key` trait only tells the compiler which downcasting function to call. The distinction from anymap is minor: rust-typemap enforces a one-to-one Key-Value mapping via the trait, whereas anymap stores Any directly. This provides more static guarantees during insertion, but the keying mechanism remains a runtime `TypeId` lookup followed by unsafe downcasting.

### Approach used (or enabled)

**Approach 10 (runtime variant).** rust-typemap is a type-level hash-map in nomenclature only: it uses `TypeId` keying and `Box<dyn Any>` storage, placing it in approach 6 (runtime `TypeId` comparison) / decisions Option 3 territory. Compared to anymap, it differs in requiring explicit `Key` trait impls (stricter at the type level but not more type-level in execution), and providing ergonomic entry APIs. Both crates achieve the same deduplicated storage via `TypeId`; neither enables type-level row canonicalisation. rust-typemap's `Key`-with-`Value` pattern is compile-time ergonomic but not type-level.

### Stable or nightly

**Stable Rust only.** No feature gates or nightly features are required. The crate uses only `std::any::{Any, TypeId}` (both stable) and `std::collections::HashMap`. MSRV is not explicitly documented in the README or Cargo.toml, but the code uses only 2015/2018 Rust idioms and would likely work on Rust 1.20+.

### Ergonomics and compile-time profile

API is ergonomic: the `Key` trait enforces a static relationship between key and value, preventing type confusion. Generic `insert::<K>` and `get::<K>` methods rely on trait resolution, which is near-zero compile overhead. Type inference works cleanly: `map.get::<MyKey>()` returns `Option<&MyKey::Value>` with no ambiguity. Runtime cost is identical to anymap (HashMap lookup + downcasting); no allocation beyond the HashMap itself.

### Production status

**Abandoned or long-dormant.** Last commit: 2017-05-28 (over 8 years old); crate version 0.3.3 suggests early-stage API that never stabilized. The crate has no known major dependents and is not maintained. It remains functional on modern Rust, but lacks active development or community adoption compared to anymap.

### Differences from anymap

Both crates store one value per type via `TypeId`-keyed `HashMap<TypeId, Box<dyn Any>>`. Differences are minor: (1) rust-typemap requires explicit `Key` trait impls, enforcing a static one-to-one key-value mapping at compile time; anymap accepts any type as key implicitly. (2) rust-typemap offers a `Key` trait enabling ergonomic recovery of the value type within `get::<K>()` without external annotation; anymap requires the caller to know the value type. (3) rust-typemap's underlying storage type `A` can be parameterized (e.g., `TypeMap<dyn Any + Send>`); anymap's `Map<A>` allows the same. Neither is strictly more type-level; the `Key` trait is a facade over runtime `TypeId` dispatch.

### Applicability to coproduct row canonicalisation

**Not applicable.** Both rust-typemap and anymap are runtime-keyed. To deduplicate an effect row like `Coproduct<A, Coproduct<B, Void>>` to a canonical order, a type-level coproduct would need to resolve the row structure (and compare effects) at compile time. rust-typemap's `Key`-with-`Value` mechanism does not enable this: inserting two effects of the same type into the same row position is a runtime operation, and the coproduct structure itself is unaffected. No approach in this family (10: runtime hash-maps) solves the canonicalisation problem without falling back to runtime TypeId comparison, which the decisions already rejects as Option 3.

### References

- src/lib.rs: 80-83 (`Key` trait definition), 27-29 (`TypeMap` struct with `HashMap<TypeId, Box<A>>`), 104-140 (insert/get/remove methods with `TypeId::of::<K>()`).
- src/internals.rs: 1-125 (trait bounds for parameterized value types, unsafe casting).
- Cargo.toml: version 0.3.3, last commit 2017-05-28.
- README.md: "Key-value pairs, rather than enforcing that keys and values are the same type" (describes the trait design).

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1200 (excluding this template boilerplate)
