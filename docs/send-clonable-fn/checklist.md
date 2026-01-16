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

- [ ] **Create `par_foldable.rs` module**

  - [ ] Create file [`fp-library/src/classes/par_foldable.rs`](../../fp-library/src/classes/par_foldable.rs)
  - [ ] Add module-level documentation explaining parallel folding
  - [ ] Add imports (`Foldable`, `Monoid`, `SendClonableFn`, `Apply`, `kinds`)

- [ ] **Define `ParFoldable` trait**

  - [ ] Define trait with `FnBrand: SendClonableFn` generic parameter
  - [ ] Add supertrait bound `Foldable`
  - [ ] Define `par_fold_map` method
    - [ ] Use `Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, ...)` for function parameter
    - [ ] `Send + Sync` bounds on `A` and `M`
    - [ ] Monoid bound on `M`
  - [ ] Define `par_fold_right` method
    - [ ] Use `Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, ...)` for function parameter
    - [ ] `Send + Sync` bounds on `A` and `B`
  - [ ] Add comprehensive doc comments with examples

- [ ] **Add free functions**

  - [ ] Implement `par_fold_map` free function
  - [ ] Implement `par_fold_right` free function
  - [ ] Add documentation for both

- [ ] **Export from `classes/mod.rs`**
  - [ ] Add `pub mod par_foldable;`
  - [ ] Re-export trait and functions

### 3.2 Library Exports for ParFoldable

- [ ] **Update `fp-library/src/lib.rs`**
  - [ ] Re-export `ParFoldable` trait
  - [ ] Re-export free functions

---

## Phase 4: Type Implementations

### 4.1 VecBrand ParFoldable

- [ ] **Implement `ParFoldable` for `VecBrand`**

  - [ ] Update [`fp-library/src/types/vec.rs`](../../fp-library/src/types/vec.rs)
  - [ ] Add import for `ParFoldable` and `SendClonableFn`
  - [ ] Implement `par_fold_map` (sequential baseline)
  - [ ] Implement `par_fold_right` (sequential baseline)
  - [ ] Add documentation

- [ ] **Add tests for VecBrand parallel folding**
  - [ ] Test empty vector
  - [ ] Test single element
  - [ ] Test multiple elements
  - [ ] Test with sum monoid
  - [ ] Test with string monoid

### 4.2 OptionBrand ParFoldable

- [ ] **Implement `ParFoldable` for `OptionBrand`**

  - [ ] Update [`fp-library/src/types/option.rs`](../../fp-library/src/types/option.rs)
  - [ ] Add import for `ParFoldable` and `SendClonableFn`
  - [ ] Implement `par_fold_map`
  - [ ] Implement `par_fold_right`
  - [ ] Add documentation

- [ ] **Add tests for OptionBrand parallel folding**
  - [ ] Test `None` case
  - [ ] Test `Some` case
  - [ ] Test with sum monoid

### 4.3 Additional Types (Optional)

- [ ] **ResultBrand ParFoldable** (if applicable)

  - [ ] Implement `ParFoldable` for `ResultBrand`
  - [ ] Add tests

- [ ] **Other Foldable types**
  - [ ] Identify other types that could benefit from `ParFoldable`
  - [ ] Implement as needed

---

## Phase 5: Optional Rayon Integration

### 5.1 Feature Flag Setup

- [ ] **Update `fp-library/Cargo.toml`**

  - [ ] Add `rayon` feature flag: `rayon = ["dep:rayon"]`
  - [ ] Add rayon dependency: `rayon = { version = "1.11", optional = true }`

- [ ] **Document feature flag**
  - [ ] Add comment in Cargo.toml
  - [ ] Update README if exists

### 5.2 Rayon-Powered Implementations

- [ ] **Conditional VecBrand implementation**

  - [ ] Add `#[cfg(feature = "rayon")]` version of `ParFoldable` for `VecBrand`
  - [ ] Use `par_iter()` for truly parallel execution
  - [ ] Ensure correct `reduce` semantics for monoids

- [ ] **Add rayon integration tests**
  - [ ] Test parallel execution actually occurs
  - [ ] Benchmark parallel vs sequential

---

## Phase 6: SendEndofunction Implementation

### 6.1 SendEndofunction Type

- [ ] **Create `send_endofunction.rs` module**
  - [ ] Create file [`fp-library/src/types/send_endofunction.rs`](../../fp-library/src/types/send_endofunction.rs)
  - [ ] Define `SendEndofunction` struct wrapping `SendClonableFn::SendOf`
  - [ ] Implement `Monoid` for `SendEndofunction` (using function composition)
  - [ ] Implement `Semigroup` for `SendEndofunction`

### 6.2 ParFoldable Default Implementation

- [ ] **Update `ParFoldable` trait**
  - [ ] Provide default implementation for `par_fold_right`
  - [ ] Use `par_fold_map` to map elements to `SendEndofunction`
  - [ ] Reduce using `Monoid::append` (composition)
  - [ ] Apply resulting function to initial value

### 6.3 VecBrand Update

- [ ] **Update `VecBrand` implementation**
  - [ ] Remove specialized `par_fold_right` implementation (use default)
  - [ ] Verify `par_fold_map` (Rayon-powered) is used by default `par_fold_right`

---

## Phase 7: Testing

### 7.1 Unit Tests

- [ ] **SendClonableFn unit tests** ([`fp-library/src/types/arc_fn.rs`](../../fp-library/src/types/arc_fn.rs))

  - [ ] Test `new_send` creates callable function
  - [ ] Test function can be cloned
  - [ ] Test `SendOf` is `Send` (spawn thread)
  - [ ] Test `SendOf` is `Sync` (share across threads)

- [ ] **ParFoldable unit tests**
  - [ ] Test `par_fold_map` with empty collection
  - [ ] Test `par_fold_map` with single element
  - [ ] Test `par_fold_map` with multiple elements
  - [ ] Test `par_fold_right` correctness
  - [ ] Test monoid laws are preserved

### 7.2 Integration Tests

- [ ] **Thread safety integration tests**

  - [ ] Create test file [`fp-library/tests/thread_safety.rs`](../../fp-library/tests/thread_safety.rs) (new)
  - [ ] Test spawning thread with `SendOf` function
  - [ ] Test multiple threads sharing `SendOf` function
  - [ ] Test `ParFoldable` in threaded context

- [ ] **Compatibility tests**
  - [ ] Verify existing `Foldable` tests still pass
  - [ ] Verify existing `ClonableFn` tests still pass
  - [ ] No regression in functionality

### 7.3 Property-Based Tests

- [ ] **Add QuickCheck tests**

  - [ ] Test `par_fold_map` equals `fold_map` (for commutative monoids)
  - [ ] Test `par_fold_right` equals `fold_right`
  - [ ] Test empty/identity properties

- [ ] **Thread safety properties**
  - [ ] Concurrent access doesn't cause data races
  - [ ] Results are deterministic (for commutative operations)

### 7.4 Compile-Fail Tests

- [ ] **Create compile-fail tests** ([`fp-library/tests/ui/`](../../fp-library/tests/ui/) or similar)

  - [ ] Test: cannot `new_send` with non-`Send` closure
  - [ ] Test: cannot `new_send` with non-`Sync` closure
  - [x] Test: `RcFnBrand` does not implement `SendClonableFn`
  - [ ] Test: cannot use `Send + Sync` function in non-`Send` context (expected failure)

- [x] **Setup trybuild for UI tests** (if not already present)
  - [x] Add `trybuild` dependency
  - [x] Create test harness

---

## Phase 8: Documentation

### 8.1 Code Documentation

- [ ] **Document `SendClonableFn` trait**

  - [ ] Module-level documentation
  - [ ] Trait documentation
  - [ ] Associated type documentation
  - [ ] Method documentation with examples

- [ ] **Document `ParFoldable` trait**

  - [ ] Module-level documentation
  - [ ] Trait documentation
  - [ ] Method documentation with examples

- [ ] **Document implementations**
  - [ ] `ArcFnBrand::SendClonableFn` implementation
  - [ ] `VecBrand::ParFoldable` implementation
  - [ ] `OptionBrand::ParFoldable` implementation

### 8.2 Project Documentation

- [ ] **Update limitations.md**

  - [ ] Mark Solution 1 as implemented
  - [ ] Add link to new traits
  - [ ] Update status of thread safety limitation

- [ ] **Update todo.md**

  - [ ] Check off parallelization task
  - [ ] Add any new discovered tasks

- [ ] **Update CHANGELOG.md**
  - [ ] Add `Apply!` macro `output` parameter
  - [ ] Add new `SendClonableFn` trait
  - [ ] Add new `ParFoldable` trait
  - [ ] Add `ArcFnBrand` implementation
  - [ ] Add optional `rayon` feature

### 8.3 README Updates

- [ ] **Update fp-library README** (if exists)

  - [ ] Document thread-safe capabilities
  - [ ] Document `rayon` feature flag
  - [ ] Add usage examples

- [ ] **Update fp-macros README**
  - [ ] Document `output` parameter

---

## Final Verification

### Build and Test

- [ ] `cargo build` succeeds for entire workspace
- [ ] `cargo test` passes for entire workspace
- [ ] `cargo test --features rayon` passes (if implemented)
- [ ] `cargo clippy` has no new warnings
- [ ] `cargo doc` generates correct documentation

### Quality Checks

- [ ] All new code has documentation
- [ ] All public items have doc comments
- [ ] Examples in docs compile (`cargo test --doc`)
- [ ] No regression in existing functionality
- [ ] Thread safety verified by compile-time checks

### Thread Safety Verification

- [ ] `SendOf` type is actually `Send` (verified by spawning thread in test)
- [ ] `SendOf` type is actually `Sync` (verified by sharing across threads in test)
- [ ] Non-`Send` closures cannot be wrapped with `new_send` (compile-fail test)
- [ ] `RcFnBrand` does not implement `SendClonableFn` (compile-fail test)

---

## Notes

_Add implementation notes, decisions, and blockers here as work progresses._

### Implementation Status

- **Phase 1 Status**: Completed (Apply! macro enhancement - prerequisite)
- **Phase 2 Status**: Completed (SendClonableFn trait)
- **Phase 3 Status**: Completed (ParFoldable trait)
- **Phase 4 Status**: Completed (Type implementations)
- **Phase 5 Status**: Completed (Optional rayon integration)
- **Phase 6 Status**: Not started (SendEndofunction Implementation)
- **Phase 7 Status**: In progress (Testing)
- **Phase 8 Status**: Not started (Documentation)
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
