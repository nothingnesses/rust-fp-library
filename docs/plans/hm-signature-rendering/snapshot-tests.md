# Plan: HM Signature Snapshot Tests (Step 7-8g)

## Goal

Add regression tests that assert the exact HM output string for every
inference wrapper function. Uses insta snapshot testing against the
real `dispatch.rs` module run through `document_module_worker`.

## Approach

Read the full `fp-library/src/dispatch.rs` file via `include_str!`,
run it through `document_module_worker` (the full proc macro
pipeline), extract the generated `#[doc]` attributes containing HM
signatures from all dispatch submodules, and assert them with insta's
`assert_snapshot!`.

### Why the full dispatch module (not per-file)

In production, `#[document_module]` is applied to the entire
`mod inner { ... }` in `dispatch.rs`, which includes all dispatch
submodules inline. The dispatch analysis pass (Pass 1b) runs on all
items in this module at once, so all dispatch traits are in
`Config.dispatch_traits` when any function's signature is generated.

Processing individual dispatch files in isolation does not work
because the `document_module_worker` pipeline expects the full module
context. Specifically, the `build_synthetic_signature` function relies
on the function's where clause referencing dispatch traits that were
analyzed in Pass 1b. When a single file is processed, its dispatch
trait IS found, but the InferableBrand parameter substitution depends
on context that is only fully populated when all submodules are
processed together.

Using the full dispatch module mirrors production exactly and avoids
false divergences between test and production behavior.

### Why in-crate (not integration tests)

All internal modules are `pub(crate)` in `fp-macros/src/lib.rs`.
Integration tests in `fp-macros/tests/` cannot access
`document_module_worker`. The tests must live inside the crate as a
`#[cfg(test)]` module.

## Steps

### 1. Add insta dev-dependency

Add to `fp-macros/Cargo.toml`:

```toml
[dev-dependencies]
insta = "1"
```

No feature flags needed (plain string snapshots only).

### 2. Create the test module

Create `fp-macros/src/documentation/signature_snapshot_tests.rs`.
Wire it from `fp-macros/src/documentation.rs`:

```rust
#[cfg(test)]
mod signature_snapshot_tests;
```

### 3. Write helpers

**Inner module extraction:** The `dispatch.rs` file has
`#[fp_macros::document_module] mod inner { ... }` wrapping all
content. Extract the body of `mod inner` as raw text using brace
matching, then tokenize and pass to `document_module_worker`.

**Signature extraction:** Walk the output token tree recursively,
descending into submodules (each dispatch type has its own
`mod alt { ... }`, `mod functor { ... }`, etc.). For each
`Item::Fn`, check `#[doc]` attributes for the `forall` keyword.
Collect into a `BTreeMap<String, String>` for deterministic ordering.

**Signature formatting:** One line per function, sorted by name.
This becomes the snapshot content.

### 4. Single test function covering all 37 signatures

One test reads the full dispatch module and snapshots all signatures
as a single multi-line string:

```rust
#[test]
fn dispatch_signatures() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fp-library/src/dispatch.rs"
    ));
    let sigs = extract_signatures(source);
    let output = format_signatures(&sigs);
    insta::assert_snapshot!(output);
}
```

All 37 signatures are visible together in one snapshot file. If any
signature changes, the diff shows exactly which one.

### 5. Initial snapshot creation

Run `just test` (test fails, no snapshot yet), then
`cargo insta review` to accept the initial snapshot. Verify it
matches the known-correct output before committing.

### 6. Commit snapshot files

The snapshot file lands in
`fp-macros/src/documentation/snapshots/signature_snapshot_tests__dispatch_signatures.snap`.
Commit `.snap` files to git. Do not commit `.snap.new` files
(pending reviews).

## File inventory

| File                                                      | Purpose                                          |
| --------------------------------------------------------- | ------------------------------------------------ |
| `fp-macros/Cargo.toml`                                    | Add `insta` dev-dependency                       |
| `fp-macros/src/documentation.rs`                          | Add `#[cfg(test)] mod signature_snapshot_tests;` |
| `fp-macros/src/documentation/signature_snapshot_tests.rs` | Test module: helpers + 1 test function           |
| `fp-macros/src/documentation/snapshots/*.snap`            | Generated snapshot file                          |

## Test coverage

One test function (`dispatch_signatures`) covers all 37 inference
wrapper functions across all 18 dispatch submodules:

alt, apply_first, apply_second, bi_fold_left, bi_fold_map,
bi_fold_right, bi_traverse, bimap, bind, bind_flipped, compact,
compose_kleisli, compose_kleisli_flipped, filter, filter_map,
filter_map_with_index, filter_with_index, fold_left,
fold_left_with_index, fold_map, fold_map_with_index, fold_right,
fold_right_with_index, join, lift2, lift3, lift4, lift5, map,
map_with_index, partition, partition_map, partition_map_with_index,
partition_with_index, separate, traverse, traverse_with_index,
wilt, wither.

## Risks and mitigations

**Large input size.** The full `dispatch.rs` inner module is large
(all 18 dispatch submodules inline). Processing it through
`document_module_worker` in a test is slower than processing individual
files but mirrors production exactly. Expected test time: under 1
second for token processing (no compilation involved).

**`include_str!` path.** Uses
`concat!(env!("CARGO_MANIFEST_DIR"), "/../fp-library/src/dispatch.rs")`
for robust compile-time path resolution.

**Validation warnings.** Pass 1.5 may emit warning tokens in the
output. These are not `Item::Fn` items and are ignored by the
signature extraction helper.

**Trait/impl method signatures.** Dispatch modules also have
`#[document_signature]` on trait methods and impl methods. The
extraction helper filters to `Item::Fn` items only (not trait methods
or impl methods), avoiding non-dispatch signatures in the output.

**Snapshot maintenance.** When dispatch modules are added or
signatures change, the single snapshot file needs updating via
`cargo insta review`.

## Heuristic replacement (robustness)

Before implementing the snapshot tests, replace remaining heuristics
in the dispatch analysis and signature generation with direct
information sources. This makes the macro more robust and less
dependent on naming conventions or extraction ordering.

### 1a. `build_container_map` positional alignment

**Current:** `build_container_map` in `generation.rs` scans the
function's dispatch trait type args for "next unmatched multi-letter
ident" to match container params. This is fragile and depends on
naming conventions.

**Fix:** Add a `position` field to `container_params` entries in
`DispatchTraitInfo`. `extract_container_params` already iterates by
position; store the index alongside the name and element types. In
`build_container_map`, use `fn_type_args[position]` directly instead
of scanning for idents.

**Files:** `dispatch.rs` (add position to container_params),
`generation.rs` (rewrite `build_container_map`).

### 1b. `find_brand_param` from trait definition

**Current:** `find_brand_param` scans the Val impl's where clause for
the first param with a semantic type class bound. This is a heuristic
that could fail if a non-brand param has a semantic bound listed first.

**Fix:** Add `find_brand_param_from_trait_def` that finds the type
param with a `Kind_*` bound in the trait definition (the direct
source). Use it as the primary source, falling back to the Val impl
when the trait definition is unavailable.

**Files:** `dispatch.rs` (add `find_brand_param_from_trait_def`,
update `extract_dispatch_info`).

### 1c. InferableBrand fallback from type_param_order

**Current:** For closureless dispatch with `()` self type (e.g., alt),
the input container element types are derived from the return
structure. This is indirect; the return structure describes the output,
not the input.

**Fix:** In the InferableBrand fallback chain, add a step between
`self_type_elements` and return structure that extracts single-letter
element types from `type_param_order` (the dispatch trait's generic
params). For alt, this gives `["A"]` directly from the trait
definition.

**Files:** `generation.rs` (add type_param_order fallback in
InferableBrand chain).

## Edge case tests (robustness)

Add unit tests that verify the signature generation handles unusual
inputs gracefully. These use synthetic code snippets (like the
existing `dispatch.rs` tests with `make_items`).

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

## Implementation order

1. **Heuristic replacement (1a, 1b, 1c):** Makes the macro robust
   before pinning behavior in snapshots.
2. **Snapshot tests:** Captures the correct, robust output as the
   baseline.
3. **Edge case tests:** Verifies robustness for unusual inputs.

## Deviation from original plan

The original plan proposed 18 test functions, one per dispatch file,
each processing an individual file in isolation. This approach was
abandoned because `document_module_worker` relies on the full module
context for correct signature generation. Processing individual files
produces incorrect output (InferableBrand parameters not substituted)
because the pipeline expects all dispatch traits and marker types to
be available at the same module level, as they are in production.
