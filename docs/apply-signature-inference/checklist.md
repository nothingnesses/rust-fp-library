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

- [ ] **Define `SignatureParam` enum**

  - [ ] Create enum in [`fp-macros/src/apply.rs`](../../fp-macros/src/apply.rs) with `Lifetime` and `Type` variants
  - [ ] Include `bounds` field on `Type` variant
  - [ ] Add documentation

- [ ] **Define `UnifiedSignature` struct**

  - [ ] Create struct with `params` and `output_bounds` fields
  - [ ] Implement `to_kind_input()` method for Kind name generation
  - [ ] Implement `concrete_lifetimes()` method
  - [ ] Implement `concrete_types()` method
  - [ ] Add documentation and tests

- [ ] **Modify `KindSource` enum**

  - [ ] Change `Generated` variant to hold `UnifiedSignature`
  - [ ] Change `Explicit` variant to hold `kind`, `lifetimes`, and `types` together
  - [ ] Update all match expressions

- [ ] **Simplify `ApplyInput` struct**
  - [ ] Remove top-level `lifetimes` and `types` fields
  - [ ] Keep only `brand` and `kind_source`

### 1.2 Parsing Implementation

- [ ] **Replace `parse_signature()` function**

  - [ ] Parse `Type` instead of `Ident` for type parameters
  - [ ] Handle complex types: `Vec<T>`, `&'a T`, `Box<dyn Fn(A) -> B>`, tuples, arrays
  - [ ] Parse optional bounds after `:`
  - [ ] Preserve original type expression for projection
  - [ ] Add unit tests for various type expressions

- [ ] **Update `ApplyInput::parse()`**

  - [ ] Reject `lifetimes:` and `types:` when `signature:` is provided
  - [ ] Require `lifetimes:` and `types:` when `kind:` is provided
  - [ ] Add clear error messages for invalid combinations
  - [ ] Add tests for error cases

- [ ] **Add `parse_bounds()` helper function**
  - [ ] Extract bound parsing logic
  - [ ] Handle multi-bound syntax (`Clone + Send + 'a`)
  - [ ] Add unit tests

### 1.3 Code Generation

- [ ] **Update `apply_impl()` function**

  - [ ] Handle `KindSource::Generated` with `UnifiedSignature`
  - [ ] Handle `KindSource::Explicit` with embedded lifetimes/types
  - [ ] Remove old fallback/override logic
  - [ ] Add unit tests for code generation

- [ ] **Kind name generation integration**
  - [ ] Verify `to_kind_input()` produces correct `KindInput`
  - [ ] Verify `generate_name()` works with converted input
  - [ ] Add integration tests

---

## Phase 2: Testing

### 2.1 Unit Tests

- [ ] **Parsing tests** ([`fp-macros/src/apply.rs`](../../fp-macros/src/apply.rs))

  - [ ] Simple lifetime: `('a)`
  - [ ] Simple type: `(T)`
  - [ ] Type with bounds: `(T: Clone)`
  - [ ] Multiple bounds: `(T: Clone + Send)`
  - [ ] Lifetime bound: `(T: 'a)`
  - [ ] Mixed parameters: `('a, T: Clone)`
  - [ ] Complex type: `(Vec<String>: Clone)`
  - [ ] Reference type: `(&'a str: Display)`
  - [ ] Output bounds: `('a, T) -> Debug`
  - [ ] Multiple output bounds: `('a, T) -> Debug + Clone`

- [ ] **Extraction tests**

  - [ ] `to_kind_input()` produces correct lifetime count
  - [ ] `to_kind_input()` produces correct type count
  - [ ] `to_kind_input()` preserves bounds
  - [ ] `concrete_lifetimes()` extracts correct values
  - [ ] `concrete_types()` extracts correct values

- [ ] **Generation tests**
  - [ ] Unified syntax produces correct expansion
  - [ ] Explicit kind syntax produces correct expansion

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
