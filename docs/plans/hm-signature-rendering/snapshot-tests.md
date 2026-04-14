# Plan: HM Signature Snapshot Tests and Robustness (Step 7-8g)

## Goal

1. Replace remaining heuristics in dispatch analysis and signature
   generation with direct information sources.
2. Add regression tests that assert the exact HM output string for
   every inference wrapper function.
3. Add edge case tests for robustness.

## Current progress

### Heuristic replacement: Done

All three heuristic replacements have been implemented and verified
(production docs produce correct output for all 37 functions).

**1a. `build_container_map` positional alignment (done).** Replaced
"next unmatched multi-letter ident" scanning with direct positional
lookup. Introduced `ContainerParam` struct with `name`, `position`,
and `element_types` fields. `extract_container_params` stores the
trait param index; `build_container_map` uses
`fn_type_args[position]` directly. Also extracted
`extract_dispatch_type_args` and `extract_dispatch_trait_args`
helpers to eliminate duplicated dispatch trait detection code.

**1b. `find_brand_param` from trait definition (done).** Added
`find_brand_param_from_trait_def` that finds the type param with a
`Kind_*` bound in the trait definition (direct source). Used as
primary in `extract_dispatch_info`, with the Val impl where-clause
scan as fallback.

**1c. InferableBrand fallback from `type_param_order` (done).** Added
a step in the InferableBrand fallback chain between
`self_type_elements` and return structure that extracts single-letter
element types from `type_param_order` (the dispatch trait's generic
params).

### Snapshot test infrastructure: Done

18 per-file tests read each dispatch submodule via `include_str!`,
extract the `mod inner` body, run it through `document_module_worker`,
and assert generated HM signatures against insta snapshots. All 37
inference wrapper function signatures are covered and correct.

**Closureless dispatch resolution:** The initial implementation
showed `FA` instead of branded types for closureless dispatch
functions (alt, compact, etc.). Root cause: each dispatch file's
`mod explicit` submodule contains functions with the same names as
the inference wrappers. The `collect_fn_signatures` helper descended
into `mod explicit` and the explicit function's signature (which
correctly does NOT substitute FA) overwrote the inference wrapper's
signature in the BTreeMap. Fixed by skipping `mod explicit` during
signature collection.

### Edge case tests: Done

14 edge case tests added covering unusual inputs and graceful
fallback behavior:

**Dispatch analysis (9 tests in `dispatch.rs`):** Brand param in
middle of param list, unusual Brand name, no semantic constraint, no
Val impl, extra type params, associated type extraction, container
param positions, type param ordering.

**Signature generation (5 tests in `signature_snapshot_tests.rs`):**
Missing Kind hash fallback, no `#[document_signature]` attribute,
`#[document_signature]` without dispatch trait, simple dispatch
full-pipeline, bifunctor two-element container.

## Approach

### Per-file snapshot tests

Each test reads one dispatch submodule file via `include_str!`,
extracts the `mod inner { ... }` body using brace counting, passes
it to `document_module_worker`, and extracts HM signatures from the
generated `#[doc]` attributes. Uses `insta::assert_snapshot!` for
assertion.

A `dispatch_test!` macro reduces boilerplate:

```rust
macro_rules! dispatch_test {
    ($name:ident, $file:expr) => {
        #[test]
        fn $name() {
            let source = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../fp-library/src/dispatch/",
                $file,
            ));
            let sigs = extract_signatures(source);
            assert!(!sigs.is_empty(), concat!("No HM signatures found in ", $file));
            let output = format_signatures(&sigs);
            insta::assert_snapshot!(output);
        }
    };
}
```

### Why in-crate (not integration tests)

All internal modules are `pub(crate)` in `fp-macros/src/lib.rs`.
Integration tests in `fp-macros/tests/` cannot access
`document_module_worker`. The tests must live inside the crate as a
`#[cfg(test)]` module.

## File inventory

| File                                                      | Purpose                                                              |
| --------------------------------------------------------- | -------------------------------------------------------------------- |
| `fp-macros/Cargo.toml`                                    | Add `insta` dev-dependency (done)                                    |
| `fp-macros/src/documentation.rs`                          | Add `#[cfg(test)] mod signature_snapshot_tests;` (done)              |
| `fp-macros/src/documentation/signature_snapshot_tests.rs` | Test module: helpers + 18 test functions (done)                      |
| `fp-macros/src/documentation/snapshots/*.snap`            | Generated snapshot files (pending)                                   |
| `fp-macros/src/analysis/dispatch.rs`                      | `ContainerParam`, `find_brand_param_from_trait_def` (done)           |
| `fp-macros/src/documentation/generation.rs`               | Positional `build_container_map`, `type_param_order` fallback (done) |

## Edge case test specification

### Dispatch analysis edge cases (`dispatch.rs`)

- Brand parameter in middle of generic param list (not first after
  lifetime).
- Brand parameter with unusual name (e.g., `F` instead of `Brand`).
- Dispatch trait with no semantic constraint in where clause.
- Dispatch trait with no arrow type (closureless, like alt).
- Dispatch trait with extra/unexpected type params.
- Empty dispatch trait (no Val impl found).

### Signature generation edge cases (`generation.rs`)

- Missing Kind hash (should return None, falling back to standalone
  macro).
- Container params with multiple element types (e.g., bifunctor with
  two-element Brand).
- Associated type projection with no matching entry in
  `associated_types`.
- Tuple closure with nested Apply! return types.
- Function with no `#[document_signature]` attribute (should be
  skipped).
- Function with `#[document_signature]` but no dispatch trait
  reference (should be left for standalone macro).

## Deviations

1. **Per-file tests instead of full-module test.** The original plan
   (first revision) proposed processing the full `dispatch.rs` module
   as one unit. Investigation revealed that `dispatch.rs`'s
   `mod inner` only contains marker types (`Val`, `Ref`,
   `ClosureMode`), not the dispatch submodules. Each submodule file
   has its own `#[document_module]`. Per-file processing is correct.

2. **18 tests instead of 1.** Per-file processing means one test per
   dispatch file. This is better for isolation (a failure pinpoints
   the exact module) and matches production behavior.

3. **Heuristic replacement done before snapshots accepted.** Tasks 1a,
   1b, 1c were completed first per the plan's recommended order. The
   snapshot tests were implemented concurrently to validate output.

4. **Closureless dispatch issue blocks snapshot acceptance.** 6 of 37
   functions produce incorrect output in the test context but correct
   output in production. This is a test infrastructure issue, not a
   macro bug. Needs investigation before snapshots can be accepted.
