# SendClonableFn Extension Trait Implementation Checklist

This checklist tracks implementation progress for the [SendClonableFn Extension Trait Plan](./plan.md).

---

## Legend

- `[ ]` Not started
- `[-]` In progress
- `[x]` Completed
- `[~]` Skipped / Not applicable

---

## Phase 1: Apply! Macro Enhancement (Prerequisite)

This phase must be completed **first** to enable the `output` parameter in `Apply!`, which is required for ergonomic access to `SendOf`.

### 1.1 Modify ApplyInput Struct

- [x] **Update [`fp-macros/src/apply.rs`](../../fp-macros/src/apply.rs)**
  - [x] Add `output: Option<Ident>` field to `ApplyInput` struct
  - [x] Update struct documentation

### 1.2 Update Parsing Logic

- [x] **Modify `ApplyInput::parse()`**
  - [x] Add `output` variable initialized to `None`
  - [x] Add new `else if label == "output"` branch in the label matching loop
  - [x] Parse the identifier value: `output = Some(input.parse()?);`
  - [x] Include `output` field in the returned `ApplyInput`
  - [x] Handle with both `signature` and `kind` modes (no additional validation needed)

### 1.3 Update Code Generation

- [x] **Modify `apply_impl()` function**
  - [x] Default `output` to `Of` when not specified using `unwrap_or_else`
  - [x] Replace hardcoded `Of` with the `assoc_type` variable in the `quote!` output
  - [x] Update `KindSource::Generated` branch to use `#assoc_type`
  - [x] Update `KindSource::Explicit` branch to use `#assoc_type`

### 1.4 Testing

- [x] **Unit tests**

  - [x] Test parsing without `output` (default behavior)
  - [x] Test parsing with `output: SendOf`
  - [x] Test with `signature` mode
  - [x] Test with `kind` mode

- [x] **Integration tests**

  - [x] Add test in [`fp-macros/tests/apply_integration.rs`](../../fp-macros/tests/apply_integration.rs)
  - [x] Verify expansion produces correct type projection

- [x] **Compile-fail tests**
  - [x] Add [`fp-macros/tests/ui/apply_invalid_output.rs`](../../fp-macros/tests/ui/apply_invalid_output.rs)
  - [x] Test: `output` must be an identifier (not string)
  - [x] Add corresponding `.stderr` file

### 1.5 Documentation

- [x] **Update macro documentation**
  - [x] Update docstrings in [`fp-macros/src/lib.rs`](../../fp-macros/src/lib.rs)
  - [x] Add examples with `output` parameter
  - [x] Document default behavior (`Of`)

---

## Phase 2: Core SendClonableFn Implementation

### 2.1 SendClonableFn Trait Definition

- [x] **Create `send_clonable_fn.rs` module**

  - [x] Create file [`fp-library/src/classes/send_clonable_fn.rs`](../../fp-library/src/classes/send_clonable_fn.rs)
  - [x] Add module-level documentation explaining thread safety
  - [x] Add imports (`ClonableFn`, `std::ops::Deref`)

- [x] **Define `SendClonableFn` trait**

  - [x] Define trait extending `ClonableFn`
  - [x] Add `SendOf<'a, A, B>` associated type with bounds:
    - [x] `Clone`
    - [x] `Send`
    - [x] `Sync`
    - [x] `Deref<Target = dyn 'a + Fn(A) -> B + Send + Sync>`
  - [x] Add `new_send` method signature
  - [x] Add comprehensive doc comments with examples

- [x] **Add free function `new_send`**

  - [x] Implement `new_send` free function
  - [x] Add documentation with examples

- [x] **Export from `classes/mod.rs`**
  - [x] Add `pub mod send_clonable_fn;`
  - [x] Re-export trait and function

### 2.2 ArcFnBrand Implementation

- [x] **Implement `SendClonableFn` for `ArcFnBrand`**

  - [x] Update [`fp-library/src/types/arc_fn.rs`](../../fp-library/src/types/arc_fn.rs)
  - [x] Add import for `SendClonableFn`
  - [x] Define `SendOf<'a, A, B> = Arc<dyn 'a + Fn(A) -> B + Send + Sync>`
  - [x] Implement `new_send` method with `Arc::new(f)`
  - [x] Add documentation with threading example

- [x] **Verify RcFnBrand does NOT implement SendClonableFn**
  - [x] Confirm [`fp-library/src/types/rc_fn.rs`](../../fp-library/src/types/rc_fn.rs) has no `SendClonableFn` impl
  - [x] Add compile-fail test to verify this

### 2.3 Library Exports

- [x] **Update `fp-library/src/lib.rs`**

  - [x] Re-export `SendClonableFn` trait
  - [x] Re-export `new_send` function

- [x] **Update `fp-library/src/classes.rs`** (if exists)
  - [x] Include new module in public API

---

## Phase 3: ParFoldable Implementation

### 3.1 ParFoldable Trait Definition

- [x] **Create `par_foldable.rs` module**

  - [x] Create file [`fp-library/src/classes/par_foldable.rs`](../../fp-library/src/classes/par_foldable.rs)
  - [x] Add module-level documentation explaining parallel folding
  - [x] Add imports (`Foldable`, `Monoid`, `SendClonableFn`, `Apply`, `kinds`)

- [x] **Define `ParFoldable` trait**

  - [x] Define trait with `FnBrand: SendClonableFn` generic parameter
  - [x] Add supertrait bound `Foldable`
  - [x] Define `par_fold_map` method
    - [x] Use `Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, ...)` for function parameter
    - [x] `Send + Sync` bounds on `A` and `M`
    - [x] Monoid bound on `M`
  - [x] Define `par_fold_right` method
    - [x] Use `Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, ...)` for function parameter
    - [x] `Send + Sync` bounds on `A` and `B`
  - [x] Add comprehensive doc comments with examples

- [x] **Add free functions**

  - [x] Implement `par_fold_map` free function
  - [x] Implement `par_fold_right` free function
  - [x] Add documentation for both

- [x] **Export from `classes/mod.rs`**
  - [x] Add `pub mod par_foldable;`
  - [x] Re-export trait and functions

### 3.2 Library Exports for ParFoldable

- [x] **Update `fp-library/src/lib.rs`**
  - [x] Re-export `ParFoldable` trait
  - [x] Re-export free functions

---

## Phase 4: Type Implementations

### 4.1 VecBrand ParFoldable

- [x] **Implement `ParFoldable` for `VecBrand`**

  - [x] Update [`fp-library/src/types/vec.rs`](../../fp-library/src/types/vec.rs)
  - [x] Add import for `ParFoldable` and `SendClonableFn`
  - [x] Implement `par_fold_map` (sequential baseline)
  - [x] Implement `par_fold_right` (sequential baseline)
  - [x] Add documentation

- [x] **Add tests for VecBrand parallel folding**
  - [x] Test empty vector
  - [x] Test single element
  - [x] Test multiple elements
  - [x] Test with sum monoid
  - [x] Test with string monoid

### 4.2 OptionBrand ParFoldable

- [x] **Implement `ParFoldable` for `OptionBrand`**

  - [x] Update [`fp-library/src/types/option.rs`](../../fp-library/src/types/option.rs)
  - [x] Add import for `ParFoldable` and `SendClonableFn`
  - [x] Implement `par_fold_map`
  - [x] Implement `par_fold_right`
  - [x] Add documentation

- [x] **Add tests for OptionBrand parallel folding**
  - [x] Test `None` case
  - [x] Test `Some` case
  - [x] Test with sum monoid

### 4.3 Additional Types (Optional)

- [x] **ResultBrand ParFoldable** (if applicable)

  - [x] Implement `ParFoldable` for `ResultBrand`
  - [x] Add tests

- [x] **Other Foldable types**
  - [x] Identify other types that could benefit from `ParFoldable`
  - [x] Implement as needed

---

## Phase 5: Optional Rayon Integration

### 5.1 Feature Flag Setup

- [x] **Update `fp-library/Cargo.toml`**

  - [x] Add `rayon` feature flag: `rayon = ["dep:rayon"]`
  - [x] Add rayon dependency: `rayon = { version = "1.11", optional = true }`

- [x] **Document feature flag**
  - [x] Add comment in Cargo.toml
  - [x] Update README if exists

### 5.2 Rayon-Powered Implementations

- [x] **Conditional VecBrand implementation**

  - [x] Add `#[cfg(feature = "rayon")]` version of `ParFoldable` for `VecBrand`
  - [x] Use `par_iter()` for truly parallel execution
  - [x] Ensure correct `reduce` semantics for monoids

- [ ] **Add rayon integration tests**
  - [ ] Test parallel execution actually occurs
  - [ ] Benchmark parallel vs sequential

---

## Phase 6: SendEndofunction Implementation

### 6.1 SendEndofunction Type

- [x] **Create `send_endofunction.rs` module**
  - [x] Create file [`fp-library/src/types/send_endofunction.rs`](../../fp-library/src/types/send_endofunction.rs)
  - [x] Define `SendEndofunction` struct wrapping `SendClonableFn::SendOf`
  - [x] Implement `Monoid` for `SendEndofunction` (using function composition)
  - [x] Implement `Semigroup` for `SendEndofunction`

### 6.2 ParFoldable Default Implementation

- [x] **Update `ParFoldable` trait**
  - [x] Provide default implementation for `par_fold_right`
  - [x] Use `par_fold_map` to map elements to `SendEndofunction`
  - [x] Reduce using `Monoid::append` (composition)
  - [x] Apply resulting function to initial value

### 6.3 VecBrand Update

- [x] **Update `VecBrand` implementation**
  - [x] Remove specialized `par_fold_right` implementation (use default)
  - [x] Verify `par_fold_map` (Rayon-powered) is used by default `par_fold_right`

---

## Phase 7: Testing

### 7.1 Unit Tests

- [x] **SendClonableFn unit tests** ([`fp-library/src/types/arc_fn.rs`](../../fp-library/src/types/arc_fn.rs))

  - [x] Test `new_send` creates callable function
  - [x] Test function can be cloned
  - [x] Test `SendOf` is `Send` (spawn thread)
  - [x] Test `SendOf` is `Sync` (share across threads)

- [x] **ParFoldable unit tests**
  - [x] Test `par_fold_map` with empty collection
  - [x] Test `par_fold_map` with single element
  - [x] Test `par_fold_map` with multiple elements
  - [x] Test `par_fold_right` correctness
  - [x] Test monoid laws are preserved

### 7.2 Integration Tests

- [x] **Thread safety integration tests**

  - [x] Create test file [`fp-library/tests/thread_safety.rs`](../../fp-library/tests/thread_safety.rs) (new)
  - [x] Test spawning thread with `SendOf` function
  - [x] Test multiple threads sharing `SendOf` function
  - [x] Test `ParFoldable` in threaded context

- [x] **Compatibility tests**
  - [x] Verify existing `Foldable` tests still pass
  - [x] Verify existing `ClonableFn` tests still pass
  - [x] No regression in functionality

### 7.3 Property-Based Tests

- [x] **Add QuickCheck tests**

  - [x] Test `par_fold_map` equals `fold_map` (for commutative monoids)
  - [x] Test `par_fold_right` equals `fold_right`
  - [x] Test empty/identity properties

- [x] **Thread safety properties**
  - [x] Concurrent access doesn't cause data races
  - [x] Results are deterministic (for commutative operations)

### 7.4 Compile-Fail Tests

- [x] **Create compile-fail tests** ([`fp-library/tests/ui/`](../../fp-library/tests/ui/) or similar)

  - [x] Test: cannot `new_send` with non-`Send` closure
  - [x] Test: cannot `new_send` with non-`Sync` closure
  - [x] Test: `RcFnBrand` does not implement `SendClonableFn`
  - [x] Test: cannot use `Send + Sync` function in non-`Send` context (expected failure)

- [x] **Setup trybuild for UI tests** (if not already present)
  - [x] Add `trybuild` dependency
  - [x] Create test harness

---

## Phase 8: Documentation

### 8.1 Code Documentation

- [x] **Document `SendClonableFn` trait**

  - [x] Module-level documentation
  - [x] Trait documentation
  - [x] Associated type documentation
  - [x] Method documentation with examples

- [x] **Document `ParFoldable` trait**

  - [x] Module-level documentation
  - [x] Trait documentation
  - [x] Method documentation with examples

- [x] **Document implementations**
  - [x] `ArcFnBrand::SendClonableFn` implementation
  - [x] `VecBrand::ParFoldable` implementation
  - [x] `OptionBrand::ParFoldable` implementation

### 8.2 Project Documentation

- [x] **Update limitations.md**

  - [x] Mark Solution 1 as implemented
  - [x] Add link to new traits
  - [x] Update status of thread safety limitation

- [x] **Update todo.md**

  - [x] Check off parallelization task
  - [x] Add any new discovered tasks

- [x] **Update CHANGELOG.md**
  - [x] Add `Apply!` macro `output` parameter
  - [x] Add new `SendClonableFn` trait
  - [x] Add new `ParFoldable` trait
  - [x] Add `ArcFnBrand` implementation
  - [x] Add optional `rayon` feature

### 8.3 README Updates

- [x] **Update fp-library README** (if exists)

  - [x] Document thread-safe capabilities
  - [x] Document `rayon` feature flag
  - [x] Add usage examples

- [x] **Update fp-macros README**
  - [x] Document `output` parameter

---

## Final Verification

### Build and Test

- [x] `cargo build` succeeds for entire workspace
- [x] `cargo test` passes for entire workspace
- [x] `cargo test --features rayon` passes (if implemented)
- [x] `cargo clippy` has no new warnings
- [x] `cargo doc` generates correct documentation

### Quality Checks

- [x] All new code has documentation
- [x] All public items have doc comments
- [x] Examples in docs compile (`cargo test --doc`)
- [x] No regression in existing functionality
- [x] Thread safety verified by compile-time checks

### Thread Safety Verification

- [x] `SendOf` type is actually `Send` (verified by spawning thread in test)
- [x] `SendOf` type is actually `Sync` (verified by sharing across threads in test)
- [x] Non-`Send` closures cannot be wrapped with `new_send` (compile-fail test)
- [x] `RcFnBrand` does not implement `SendClonableFn` (compile-fail test)

---

## Notes

_Add implementation notes, decisions, and blockers here as work progresses._

### Implementation Status

- **Phase 1 Status**: Completed (Apply! macro enhancement - prerequisite)
- **Phase 2 Status**: Completed (SendClonableFn trait)
- **Phase 3 Status**: Completed (ParFoldable trait)
- **Phase 4 Status**: Completed (Type implementations)
- **Phase 5 Status**: Completed (Optional rayon integration)
- **Phase 6 Status**: Completed (SendEndofunction Implementation)
- **Phase 7 Status**: Completed (Testing)
- **Phase 8 Status**: Completed (Documentation)
- **Current Issues**: None

### Decisions Made

| Date | Decision | Rationale |
| ---- | -------- | --------- |
|      |          |           |

### Blockers

| Issue | Status | Resolution |
| ----- | ------ | ---------- |

### Open Questions

1. **Should `par_fold_left` be included in `ParFoldable`?**

   - Consider if left vs right distinction matters for parallel operations
   - Proposed: Start with `par_fold_right` and `par_fold_map`, add `par_fold_left` if needed

2. **Should the rayon feature include parallel Traversable?**

   - Proposed: Keep rayon feature focused on `ParFoldable` initially
   - Future: Consider `ParTraversable` as separate enhancement

3. **What monoid laws should be tested for parallel operations?**

   - Associativity is critical for parallel reduce
   - Commutativity may affect result ordering
   - Proposed: Test with both commutative (sum) and non-commutative (string) monoids

4. **Should there be a `SendFunction` trait (non-clonable)?**
   - Proposed: Not initially, `SendClonableFn` covers the primary use case
   - Future: Add if there's demand for non-clonable thread-safe functions

---

## Related Documents

- [Implementation Plan](./plan.md)
- [Limitations: Thread Safety and Parallelism](../limitations.md#thread-safety-and-parallelism)
- [Current ClonableFn Implementation](../../fp-library/src/classes/clonable_fn.rs)
- [Current Foldable Implementation](../../fp-library/src/classes/foldable.rs)
