# tstr_crates

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/type-level/tstr_crates/`

## Purpose

Stage 1 research document: classify `tstr_crates` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`tstr_crates` provides type-level strings (`TStr<...>`) with const
comparison operators. The classification should determine whether the
comparison primitives are usable for ordering arbitrary types and
whether the crate provides a sort routine.

## Required findings

### What this codebase does

`tstr_crates` encodes type-level strings as `TStr<S>` where `S` is a
const-generic representation of characters. On stable, the default
encoding uses tuples of `char` const-parameters: `TStr<___<(tstr::__<'H',
'e', 'l', 'l', 'o'>, ...), 5>>` (tstr_impl_with_chars.rs:277-282). On
nightly with the `"nightly_str_generics"` feature, it uses bare `&'static
str` const-parameters: `TStr<___<"hello">>` (readme.md:146-148). The crate
provides const-fn operators for comparing two `TStr` values: `tstr::eq`,
`tstr::ne`, and crucially `tstr::cmp` which returns a `core::cmp::Ordering`
(tstr_fns.rs:138-144). All comparison operations are runtime-zero: `TStr`
is a zero-sized type that implements `Hash` by doing nothing
(tstr_type.rs:430-439).

### Type-level sorting capability

`tstr_crates` does NOT ship a sort routine. However, it provides complete
`core::cmp` traits (`PartialEq`, `PartialOrd`, `Ord`, `Eq`) on `TStr<S>`
for any `S: TStrArg`, and these delegates to standard string comparison
(tstr_type.rs:295-427). The `const fn` `tstr::cmp` function compares two
`IsTStr` impl types and returns `Ordering::Equal`, `Less`, or `Greater`
via `__ToTStrArgBinary::<Lhs::Arg, Rhs::Arg>::__CMP` (tstr_fns.rs:138-144,
tstr_impl_with_chars.rs:112-113). This is usable as a comparison predicate
for a sort algorithm, but only for sorting values whose type is parameterized
by a `TStr`. For a heterogeneous coproduct like `Coproduct<A, Coproduct<B,
Void>>`, each variant would need to carry a `TStr` name as its type
parameter; sorting the coproduct row would then require marshalling effect
names into `TStr` and driving a type-level sort via Ordering results.

### Approach used (or enabled)

Approach 4: adt_const_params with strings (stable variant). On stable, `char`
const-parameters are fully available (Rust 1.79+) and the crate uses them
directly in marker tuples. The nightly feature plugs in `&'static str`
const-parameters (behind `"nightly_str_generics"`, which enables
`typewit/adt_const_marker`) once the Rust const-parameter syntax stabilizes.
No specialized libraries like `typenum` or proc-macro compile-time compute
are needed; the comparison is done at compile-time via associated consts
in trait impls (tstr_impl_with_chars.rs:94-121).

### Stable or nightly

Default is **stable Rust 1.88.0+** (Cargo.toml:6). The `"str_generics"`
feature (disabled by default) was intended to enable `&'static str`
const-parameters on a future stable release. The `"nightly_str_generics"`
feature (also disabled by default) enables the same on nightly only
(readme.md:180-181, Cargo.toml:24-25). The changelog (changelog.md:46)
notes that `&'static str` const-parameter support on nightly requires
repeated fixes as the language feature evolves. No unsafe code is used;
the design is fully sound on both stable and nightly.

### Ergonomics and compile-time profile

Construction uses two styles. The `TS!(name)` and `TS!("literal")` macros
expand to the marker-type encoding at compile-time (readme.md:16-39). For
runtime values, the `ts!(name)` and `ts!("literal")` macros create
zero-sized `TStr` instances. The proc-macro `tstr_proc_macros` parses the
literal and generates the nested tuple-of-chars or `&str` representation.
No documented compile-time cost is cited; the representation is pure type
machinery, not compute. Comparing two `TStr` types via `tstr::cmp` is a
const-fn evaluation of associated const values, no overhead beyond trait
resolution.

### Production status

**Active and mature**. The crate is published on crates.io (v0.3.2 as of
changelog.md), appears to be maintained by @rodrimati1992, and has a well-
documented API, tests, and serde support (Cargo.toml:23-25). The repo
shows recent changes to support nightly const-parameter improvements
(changelog.md:46). MSRV is pinned at 1.88.0. No evidence of abandonment.

### Applicability to coproduct row canonicalisation

**Not directly applicable.** A coproduct row like `Coproduct<A, Coproduct<B,
Void>>` has its ordering encoded in the _type structure_, not in a value-
level or type-level string. To canonicalise such a coproduct, you would
need to either:

1. Annotate each variant with a `TStr` field (e.g., `struct A { name:
TStr<TS!(a)>, ... }`) and sort the variants by that name, or
2. Use a parallel type-level list encoding variant names as `TStr`s,
   compute a sorted permutation, and apply it to the coproduct structure.

Approach (1) requires manual markup at the variant definition. Approach (2)
would use `tstr::cmp` to drive a type-level quicksort or merge-sort on
the list of names, but the coproduct row itself remains a nested sum type
that does not automatically reorder. Additionally, `tstr_crates` provides
no type-level sort algorithm; you would need to write one (in a separate
crate or in your own code) that uses `tstr::cmp` as the comparison oracle.
For practical effect canonicalisation, a hash-based approach (approach 3)
or TypeId-based approach (approach 6) would be more ergonomic.

### References

- `/home/jessea/Documents/projects/type-level/tstr_crates/readme.md` (lines 1-150)
- `/home/jessea/Documents/projects/type-level/tstr_crates/tstr/Cargo.toml` (lines 1-75)
- `/home/jessea/Documents/projects/type-level/tstr_crates/tstr/src/tstr_type.rs` (lines 1-177, 294-427)
- `/home/jessea/Documents/projects/type-level/tstr_crates/tstr/src/tstr_fns.rs` (lines 138-144)
- `/home/jessea/Documents/projects/type-level/tstr_crates/tstr/src/tstr_impl_with_chars.rs` (lines 1-121, 277-282)
- `/home/jessea/Documents/projects/type-level/tstr_crates/changelog.md` (lines 1-50)

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1200 (excluding this template boilerplate)
