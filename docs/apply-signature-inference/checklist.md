# Apply! Macro Unified Signature Syntax Checklist

This checklist tracks implementation progress for the [Unified Signature Syntax Plan](./plan.md).

---

## Legend

- `[ ]` Not started
- `[-]` In progress
- `[x]` Completed
- `[~]` Skipped / Not applicable

---

## Phase 1: Core Implementation

### 1.1 Data Structures

- [x] **Define `SignatureParam` enum**

  - [x] Create enum in [`fp-macros/src/apply.rs`](../../fp-macros/src/apply.rs) with `Lifetime` and `Type` variants
  - [x] Include `bounds` field on `Type` variant
  - [x] Add documentation

- [x] **Define `UnifiedSignature` struct**

  - [x] Create struct with `params` and `output_bounds` fields
  - [x] Implement `to_kind_input()` method for Kind name generation
  - [x] Implement `concrete_lifetimes()` method
  - [x] Implement `concrete_types()` method
  - [x] Add documentation and tests

- [x] **Modify `KindSource` enum**

  - [x] Change `Generated` variant to hold `UnifiedSignature`
  - [x] Change `Explicit` variant to hold `kind`, `lifetimes`, and `types` together
  - [x] Update all match expressions

- [x] **Simplify `ApplyInput` struct**
  - [x] Remove top-level `lifetimes` and `types` fields
  - [x] Keep only `brand` and `kind_source`

### 1.2 Parsing Implementation

- [x] **Replace `parse_signature()` function**

  - [x] Parse `Type` instead of `Ident` for type parameters
  - [x] Handle complex types: `Vec<T>`, `&'a T`, `Box<dyn Fn(A) -> B>`, tuples, arrays
  - [x] Parse optional bounds after `:`
  - [x] Preserve original type expression for projection
  - [x] Add unit tests for various type expressions

- [x] **Update `ApplyInput::parse()`**

  - [x] Reject `lifetimes:` and `types:` when `signature:` is provided
  - [x] Require `lifetimes:` and `types:` when `kind:` is provided
  - [x] Add clear error messages for invalid combinations
  - [x] Add tests for error cases

- [x] **Add `parse_bounds()` helper function**
  - [x] Extract bound parsing logic
  - [x] Handle multi-bound syntax (`Clone + Send + 'a`)
  - [x] Add unit tests

### 1.3 Code Generation

- [x] **Update `apply_impl()` function**

  - [x] Handle `KindSource::Generated` with `UnifiedSignature`
  - [x] Handle `KindSource::Explicit` with embedded lifetimes/types
  - [x] Remove old fallback/override logic
  - [x] Add unit tests for code generation

- [x] **Kind name generation integration**
  - [x] Verify `to_kind_input()` produces correct `KindInput`
  - [x] Verify `generate_name()` works with converted input
  - [x] Add integration tests

---

## Phase 2: Testing

### 2.1 Unit Tests

- [x] **Parsing tests** ([`fp-macros/src/apply.rs`](../../fp-macros/src/apply.rs))

  - [x] Simple lifetime: `('a)`
  - [x] Simple type: `(T)`
  - [x] Type with bounds: `(T: Clone)`
  - [x] Multiple bounds: `(T: Clone + Send)`
  - [x] Lifetime bound: `(T: 'a)`
  - [x] Mixed parameters: `('a, T: Clone)`
  - [x] Complex type: `(Vec<String>: Clone)`
  - [x] Reference type: `(&'a str: Display)`
  - [x] Output bounds: `('a, T) -> Debug`
  - [x] Multiple output bounds: `('a, T) -> Debug + Clone`

- [x] **Extraction tests**

  - [x] `to_kind_input()` produces correct lifetime count
  - [x] `to_kind_input()` produces correct type count
  - [x] `to_kind_input()` preserves bounds
  - [x] `concrete_lifetimes()` extracts correct values
  - [x] `concrete_types()` extracts correct values

- [x] **Generation tests**
  - [x] Unified syntax produces correct expansion
  - [x] Explicit kind syntax produces correct expansion

### 2.2 Integration Tests

- [ ] **End-to-end tests** ([`fp-macros/tests/`](../../fp-macros/tests/))

  - [ ] Complete Apply! with unified syntax compiles
  - [ ] Resulting type is correct
  - [ ] Works in function signatures
  - [ ] Works in struct definitions
  - [ ] Works in impl blocks

- [ ] **Explicit kind mode tests**
  - [ ] `kind:` with `lifetimes:`/`types:` works
  - [ ] Verify expansion is correct

### 2.3 Compile-Fail Tests

- [ ] **Error cases** ([`fp-macros/tests/ui/`](../../fp-macros/tests/ui/))
  - [ ] Missing `brand:` parameter
  - [ ] Invalid type syntax in signature
  - [ ] `signature:` with `lifetimes:` (should error)
  - [ ] `signature:` with `types:` (should error)
  - [ ] `kind:` without `lifetimes:` (should error)
  - [ ] `kind:` without `types:` (should error)
  - [ ] Both `signature:` and `kind:` provided (should error)

---

## Phase 3: Library Migration

### 3.1 Update Type Implementations

- [ ] **Migrate Apply! usages**

  - [ ] [`fp-library/src/types/option.rs`](../../fp-library/src/types/option.rs)
  - [ ] [`fp-library/src/types/result.rs`](../../fp-library/src/types/result.rs)
  - [ ] [`fp-library/src/types/vec.rs`](../../fp-library/src/types/vec.rs)
  - [ ] [`fp-library/src/types/identity.rs`](../../fp-library/src/types/identity.rs)
  - [ ] [`fp-library/src/types/lazy.rs`](../../fp-library/src/types/lazy.rs)
  - [ ] [`fp-library/src/types/pair.rs`](../../fp-library/src/types/pair.rs)
  - [ ] [`fp-library/src/types/arc_fn.rs`](../../fp-library/src/types/arc_fn.rs)
  - [ ] [`fp-library/src/types/rc_fn.rs`](../../fp-library/src/types/rc_fn.rs)
  - [ ] [`fp-library/src/types/endofunction.rs`](../../fp-library/src/types/endofunction.rs)
  - [ ] [`fp-library/src/types/endomorphism.rs`](../../fp-library/src/types/endomorphism.rs)
  - [ ] [`fp-library/src/types/once_cell.rs`](../../fp-library/src/types/once_cell.rs)
  - [ ] [`fp-library/src/types/once_lock.rs`](../../fp-library/src/types/once_lock.rs)
  - [ ] [`fp-library/src/types/string.rs`](../../fp-library/src/types/string.rs)

- [ ] **Verify all tests pass**
  - [ ] Run `cargo test` in `fp-library`
  - [ ] Run `cargo test` in `fp-macros`
  - [ ] Run `cargo test` in workspace root

### 3.2 Update Type Class Traits

- [ ] **Review and update trait definitions**

  - [ ] [`fp-library/src/classes/functor.rs`](../../fp-library/src/classes/functor.rs)
  - [ ] [`fp-library/src/classes/applicative.rs`](../../fp-library/src/classes/applicative.rs)
  - [ ] [`fp-library/src/classes/monad.rs`](../../fp-library/src/classes/monad.rs)
  - [ ] [`fp-library/src/classes/foldable.rs`](../../fp-library/src/classes/foldable.rs)
  - [ ] [`fp-library/src/classes/traversable.rs`](../../fp-library/src/classes/traversable.rs)
  - [ ] Other traits as applicable

- [ ] **Verify trait bounds work correctly**
  - [ ] Generic functions compile
  - [ ] Trait impls compile
  - [ ] Documentation examples compile

---

## Phase 4: Documentation

### 4.1 Macro Documentation

- [ ] **Update `Apply!` macro docs** ([`fp-macros/src/lib.rs`](../../fp-macros/src/lib.rs))

  - [ ] Document unified signature syntax as primary
  - [ ] Document explicit `kind:` syntax for advanced cases
  - [ ] Add examples for common patterns
  - [ ] Remove old syntax documentation

- [ ] **Update module documentation**
  - [ ] [`fp-macros/src/apply.rs`](../../fp-macros/src/apply.rs) module docs
  - [ ] [`fp-library/src/lib.rs`](../../fp-library/src/lib.rs) if needed

### 4.2 Changelog

- [ ] **Update CHANGELOG.md**
  - [ ] Document breaking change to `Apply!` syntax
  - [ ] Document removal of `lifetimes:` and `types:` in signature mode
  - [ ] Document new unified syntax

---

## Final Verification

### Build and Test

- [ ] `cargo build` succeeds for entire workspace
- [ ] `cargo test` passes for entire workspace
- [ ] `cargo clippy` has no new warnings
- [ ] `cargo doc` generates correct documentation

### Quality Checks

- [ ] All new code has documentation
- [ ] All public items have doc comments
- [ ] Examples in docs compile (`cargo test --doc`)
- [ ] No regression in existing functionality (explicit `kind:` mode)

---

## Notes

_Add implementation notes, decisions, and blockers here as work progresses._

### Implementation Status (2026-01-15)
- **Phase 1.1 Completed**: Data structures defined and `apply_impl` updated.
- **Phase 1.2 Completed**: Parsing implementation updated and verified with unit tests.
- **Phase 1.3 Completed**: Code generation updated and verified with integration tests.
- **Phase 2.1 Completed**: Comprehensive unit tests added for parsing, extraction, and generation.
- **Current Issues**:
  - `fp-library` code is broken due to syntax changes (expected).
- **Next Steps (Phase 2)**:
  - Add integration tests.
  - Add compile-fail tests.

### Decisions Made

| Date | Decision | Rationale |
| ---- | -------- | --------- |
| 2026-01-15 | Remove `lifetimes:` and `types:` in signature mode | Simplifies API; values are now embedded in signature |
| 2026-01-15 | No deprecation period | Pre-1.0 library; breaking change is acceptable |
| 2026-01-15 | Keep explicit `kind:` mode | Needed for advanced cases with custom Kind traits |

### Blockers

| Issue | Status | Resolution |
| ----- | ------ | ---------- |

### Open Questions

1. Should complex types like `impl Trait` be supported in signatures?
   - Proposed: Yes, if `syn::Type` can parse them; test during implementation
2. Should empty signatures be allowed: `signature: ()`?
   - Proposed: Yes, for zero-argument Kinds
