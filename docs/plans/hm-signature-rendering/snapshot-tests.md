# Plan: HM Signature Snapshot Tests (Step 7-8g)

## Goal

Add regression tests that assert the exact HM output string for every
inference wrapper function. Uses insta snapshot testing against real
dispatch module source files run through `document_module_worker`.

## Approach

Read each real `fp-library/src/dispatch/*.rs` source file via
`include_str!`, run it through `document_module_worker` (the full
proc macro pipeline), extract the generated `#[doc]` attributes
containing HM signatures, and assert each one with insta's
`assert_snapshot!`.

### Why this approach

- **No drift risk.** Tests against real dispatch module source, not
  synthetic duplicates. Changes to dispatch traits or the macro
  pipeline are caught immediately.
- **Full pipeline coverage.** Exercises parsing, dispatch analysis
  (Pass 1b), synthetic signature building, and HM generation
  (Pass 2) end-to-end.
- **insta snapshots.** When signatures change legitimately (e.g., a
  formatting tweak affecting all 37), `cargo insta review` provides
  an interactive TUI to accept/reject each change in bulk.

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

Separate file avoids bloating `document_module.rs`. Follows the
pattern of `dispatch.rs` having its own `mod tests`.

### 3. Write helpers

**Signature extraction:**

```rust
fn extract_signatures(source: &str) -> BTreeMap<String, String> {
    let tokens: TokenStream = source.parse().unwrap();
    let result = document_module_worker(TokenStream::new(), tokens).unwrap();
    let file: syn::File = syn::parse2(result).unwrap();
    // Walk items, find Item::Fn, find #[doc] attrs containing "forall"
    // Return map of function_name -> signature_string
}
```

`BTreeMap` for deterministic iteration order.

**Signature formatting:**

```rust
fn format_signatures(sigs: &BTreeMap<String, String>) -> String {
    sigs.iter()
        .map(|(name, sig)| format!("{name}: {sig}"))
        .collect::<Vec<_>>()
        .join("\n")
}
```

One line per function, sorted by name. This becomes the snapshot
content.

### 4. One test function per dispatch module

Each test reads the real source and snapshots all its signatures
as a single multi-line string:

```rust
#[test]
fn functor_signatures() {
    let source = include_str!("../../../fp-library/src/dispatch/functor.rs");
    let sigs = extract_signatures(source);
    let output = format_signatures(&sigs);
    insta::assert_snapshot!(output);
}
```

Snapshotting one string per module (not per function) means fewer
snapshot files (18 instead of 37) and all signatures for a module
are visible together. If any signature changes, the diff shows
exactly which one.

### 5. Initial snapshot creation

Run `just test` (tests fail, no snapshots yet), then
`cargo insta review` to accept the initial snapshots. Verify they
match the known-correct output before committing.

### 6. Commit snapshot files

Snapshot files land in
`fp-macros/src/documentation/snapshots/signature_snapshot_tests__*.snap`
(insta's default: `snapshots/` directory next to the test source).
Commit `.snap` files to git. Do not commit `.snap.new` files
(pending reviews).

## File inventory

| File                                                      | Purpose                                          |
| --------------------------------------------------------- | ------------------------------------------------ |
| `fp-macros/Cargo.toml`                                    | Add `insta` dev-dependency                       |
| `fp-macros/src/documentation.rs`                          | Add `#[cfg(test)] mod signature_snapshot_tests;` |
| `fp-macros/src/documentation/signature_snapshot_tests.rs` | Test module: helpers + 18 test functions         |
| `fp-macros/src/documentation/snapshots/*.snap`            | Generated snapshot files (one per test function) |

## Test coverage

| Test function                       | Source file                          | Functions                                                                                |
| ----------------------------------- | ------------------------------------ | ---------------------------------------------------------------------------------------- |
| `functor_signatures`                | `dispatch/functor.rs`                | map                                                                                      |
| `foldable_signatures`               | `dispatch/foldable.rs`               | fold_left, fold_right, fold_map                                                          |
| `filterable_signatures`             | `dispatch/filterable.rs`             | filter, filter_map, partition, partition_map                                             |
| `semimonad_signatures`              | `dispatch/semimonad.rs`              | bind, bind_flipped, compose_kleisli, compose_kleisli_flipped, join                       |
| `alt_signatures`                    | `dispatch/alt.rs`                    | alt                                                                                      |
| `compactable_signatures`            | `dispatch/compactable.rs`            | compact, separate                                                                        |
| `lift_signatures`                   | `dispatch/lift.rs`                   | lift2, lift3, lift4, lift5                                                               |
| `bifunctor_signatures`              | `dispatch/bifunctor.rs`              | bimap                                                                                    |
| `bifoldable_signatures`             | `dispatch/bifoldable.rs`             | bi_fold_left, bi_fold_right, bi_fold_map                                                 |
| `bitraversable_signatures`          | `dispatch/bitraversable.rs`          | bi_traverse                                                                              |
| `traversable_signatures`            | `dispatch/traversable.rs`            | traverse                                                                                 |
| `witherable_signatures`             | `dispatch/witherable.rs`             | wither, wilt                                                                             |
| `apply_first_signatures`            | `dispatch/apply_first.rs`            | apply_first                                                                              |
| `apply_second_signatures`           | `dispatch/apply_second.rs`           | apply_second                                                                             |
| `functor_with_index_signatures`     | `dispatch/functor_with_index.rs`     | map_with_index                                                                           |
| `foldable_with_index_signatures`    | `dispatch/foldable_with_index.rs`    | fold_left_with_index, fold_right_with_index, fold_map_with_index                         |
| `filterable_with_index_signatures`  | `dispatch/filterable_with_index.rs`  | filter_with_index, filter_map_with_index, partition_with_index, partition_map_with_index |
| `traversable_with_index_signatures` | `dispatch/traversable_with_index.rs` | traverse_with_index                                                                      |

18 test functions covering all 37 signatures.

## Risks and mitigations

**`include_str!` path fragility.** The path
`"../../../fp-library/src/dispatch/functor.rs"` is relative to the
test source file. If the file moves, compilation fails (not a silent
breakage).

**`document_module_worker` side effects.** `get_context` (Pass 1)
runs on the items but dispatch modules contain no `impl_kind!` or
projection sources, so it populates nothing in `Config`. Safe.

**Validation warnings.** Pass 1.5 may emit warning tokens in the
output. These are not `Item::Fn` items and are ignored by the
signature extraction helper.

**Trait/impl method signatures.** Dispatch modules also have
`#[document_signature]` on trait methods and impl methods. These are
processed by the standard (non-dispatch) path. The extraction helper
filters to top-level `Item::Fn` items only, avoiding trait/impl
method signatures in the output.

**Snapshot count.** If new dispatch modules are added, a new test
function must be added manually. This is intentional: adding a new
dispatch module is a rare event that warrants explicit test coverage.
