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

### Snapshot test infrastructure: Partially done

The test module and 18 per-file tests are implemented. The
infrastructure works: `document_module_worker` processes each dispatch
file's `mod inner` body and generates signatures.

**Status by dispatch pattern:**

- Closure-based dispatch (31 functions): All produce correct
  signatures in the per-file test context. These include map,
  fold_left/right, fold_map, filter, filter_map, partition,
  partition_map, bind, bind_flipped, compose_kleisli,
  compose_kleisli_flipped, lift2-5, bimap, bi_fold_left/right/map,
  bi_traverse, traverse, wither, wilt, all WithIndex variants.

- Closureless dispatch (6 functions): Produce `FA` instead of the
  correct branded type (e.g., `Brand A`) when processed in per-file
  isolation. Affected: alt, compact, separate, join, apply_first,
  apply_second.

**Root cause of closureless dispatch issue:** In production, each
dispatch file's `mod inner` is annotated with `#[document_module]`
and processed independently by rustc. The production output IS
correct. In the test context, `document_module_worker` is called
with the same token stream and the dispatch traits ARE found (verified
with debug logging), yet the InferableBrand parameter substitution
does not trigger. The exact failure point within `build_synthetic_signature`
needs to be traced. The `is_inferable_brand_param` function checks
the original function signature's where clause for `InferableBrand_*`
bounds; this check may behave differently when the tokens come from
`include_str!` + `str::parse::<TokenStream>()` vs rustc's own
token stream.

**Note on module structure:** Each dispatch file (e.g., `alt.rs`) has
its own `#[fp_macros::document_module] pub(crate) mod inner { ... }`.
The parent `dispatch.rs` has a separate `#[document_module] mod inner`
that only contains `Val`, `Ref`, and `ClosureMode`. The submodule
files are `pub mod alt;` declarations outside that inner module. This
means per-file processing IS the correct production model (each file
processes its own `mod inner` independently), not the full-module
approach initially assumed.

### Edge case tests: Not started

### Remaining steps

1. **Debug closureless dispatch in test context.** Trace
   `build_synthetic_signature` for alt to find where InferableBrand
   substitution fails. The function has a where clause
   `FA: InferableBrand_cdc7cd43dac7585f + AltDispatch<...>`. Check
   whether `is_inferable_brand_param("FA", sig)` returns true when
   the signature comes from `str::parse::<TokenStream>()`.

2. **Accept snapshots.** Once all 37 produce correct output in the
   test context, run `cargo insta review` to accept initial snapshots.

3. **Edge case tests.** Add synthetic-code unit tests for unusual
   inputs (see specification below).

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
