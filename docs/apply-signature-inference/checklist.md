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

- [x] **End-to-end tests** ([`fp-macros/tests/`](../../fp-macros/tests/))

  - [x] Complete Apply! with unified syntax compiles
  - [x] Resulting type is correct
  - [x] Works in function signatures
  - [x] Works in struct definitions
  - [x] Works in impl blocks

- [x] **Explicit kind mode tests**
  - [x] `kind:` with `lifetimes:`/`types:` works
  - [x] Verify expansion is correct

### 2.3 Compile-Fail Tests

- [x] **Error cases** ([`fp-macros/tests/ui/`](../../fp-macros/tests/ui/))
  - [x] Missing `brand:` parameter
  - [x] Invalid type syntax in signature
  - [x] `signature:` with `lifetimes:` (should error)
  - [x] `signature:` with `types:` (should error)
  - [x] `kind:` without `lifetimes:` (should error)
  - [x] `kind:` without `types:` (should error)
  - [x] Both `signature:` and `kind:` provided (should error)

---

## Phase 3: Library Migration

### 3.1 Update Type Implementations

- [x] **Migrate Apply! usages**

  - [x] [`fp-library/src/types/option.rs`](../../fp-library/src/types/option.rs)
  - [x] [`fp-library/src/types/result.rs`](../../fp-library/src/types/result.rs)
  - [x] [`fp-library/src/types/vec.rs`](../../fp-library/src/types/vec.rs)
  - [x] [`fp-library/src/types/identity.rs`](../../fp-library/src/types/identity.rs)
  - [x] [`fp-library/src/types/lazy.rs`](../../fp-library/src/types/lazy.rs)
  - [x] [`fp-library/src/types/pair.rs`](../../fp-library/src/types/pair.rs)
  - [x] [`fp-library/src/types/arc_fn.rs`](../../fp-library/src/types/arc_fn.rs)
  - [x] [`fp-library/src/types/rc_fn.rs`](../../fp-library/src/types/rc_fn.rs)
  - [x] [`fp-library/src/types/endofunction.rs`](../../fp-library/src/types/endofunction.rs)
  - [x] [`fp-library/src/types/endomorphism.rs`](../../fp-library/src/types/endomorphism.rs)
  - [x] [`fp-library/src/types/once_cell.rs`](../../fp-library/src/types/once_cell.rs)
  - [x] [`fp-library/src/types/once_lock.rs`](../../fp-library/src/types/once_lock.rs)
  - [x] [`fp-library/src/types/string.rs`](../../fp-library/src/types/string.rs)

- [x] **Verify all tests pass**
  - [x] Run `cargo test` in `fp-library`
  - [x] Run `cargo test` in `fp-macros`
  - [x] Run `cargo test` in workspace root

### 3.2 Update Type Class Traits

- [x] **Review and update trait definitions**

  - [x] [`fp-library/src/classes/functor.rs`](../../fp-library/src/classes/functor.rs)
  - [x] [`fp-library/src/classes/applicative.rs`](../../fp-library/src/classes/applicative.rs)
  - [x] [`fp-library/src/classes/monad.rs`](../../fp-library/src/classes/monad.rs)
  - [x] [`fp-library/src/classes/foldable.rs`](../../fp-library/src/classes/foldable.rs)
  - [x] [`fp-library/src/classes/traversable.rs`](../../fp-library/src/classes/traversable.rs)
  - [x] Other traits as applicable

- [x] **Verify trait bounds work correctly**
  - [x] Generic functions compile
  - [x] Trait impls compile
  - [x] Documentation examples compile

---

## Phase 4: Documentation

### 4.1 Macro Documentation

- [x] **Update `Apply!` macro docs** ([`fp-macros/src/lib.rs`](../../fp-macros/src/lib.rs))

  - [x] Document unified signature syntax as primary
  - [x] Document explicit `kind:` syntax for advanced cases
  - [x] Add examples for common patterns
  - [x] Remove old syntax documentation

- [x] **Update module documentation**
  - [x] [`fp-macros/src/apply.rs`](../../fp-macros/src/apply.rs) module docs
  - [x] [`fp-library/src/lib.rs`](../../fp-library/src/lib.rs) if needed

### 4.2 Changelog

- [x] **Update CHANGELOG.md**
  - [x] Document breaking change to `Apply!` syntax
  - [x] Document removal of `lifetimes:` and `types:` in signature mode
  - [x] Document new unified syntax

---

## Final Verification

### Build and Test

- [x] `cargo build` succeeds for entire workspace
- [x] `cargo test` passes for entire workspace
- [x] `cargo clippy` has no new warnings
- [x] `cargo doc` generates correct documentation

### Quality Checks

- [x] All new code has documentation
- [x] All public items have doc comments
- [x] Examples in docs compile (`cargo test --doc`)
- [x] No regression in existing functionality (explicit `kind:` mode)

---

## Notes

_Add implementation notes, decisions, and blockers here as work progresses._

### Implementation Status (2026-01-15)

- **Phase 1.1 Completed**: Data structures defined and `apply_impl` updated.
- **Phase 1.2 Completed**: Parsing implementation updated and verified with unit tests.
- **Phase 1.3 Completed**: Code generation updated and verified with integration tests.
- **Phase 2.1 Completed**: Comprehensive unit tests added for parsing, extraction, and generation.
- **Phase 2.2 Completed**: Integration tests added and verified (`fp-macros/tests/apply_integration.rs`).
- **Phase 2.3 Completed**: Compile-fail tests added and verified (`fp-macros/tests/ui/`).
- **Phase 3.1 Completed**: Library code migrated to use new syntax.
- **Phase 3.2 Completed**: Type class traits updated.
- **Phase 4.1 Completed**: Macro and module documentation updated.
- **Phase 4.2 Completed**: CHANGELOG.md updated.
- **Final Verification Completed**: Build, test, clippy, and doc generation verified for entire workspace.
- **Current Issues**:
  - None. All tests pass.

### Notes

- **Nesting Support**: `Apply!` macro nesting is supported and verified by `test_nested_apply` in `fp-macros/tests/apply_integration.rs`. The macro expansion works correctly because the inner `Apply!` is parsed as a `Type` by the outer `Apply!`.
- **Kind Trait Mismatch**: When migrating, ensure that the signature provided to `Apply!` reflects the arity of the Kind trait implemented by the brand. For example, if a brand implements a Kind with 2 type parameters (e.g., `ArcFnBrand`), the signature must include 2 type parameters (e.g., `signature: ('a, A, B)`), even if they are the same type (e.g., `signature: ('a, A, A)`). This ensures the generated Kind trait name matches the implementation.
- **Type Inference**: In some cases (e.g., `fp-library/src/types/vec.rs`), explicit type annotations were needed for closures when using `Apply!` types, likely due to the complexity of the macro expansion confusing type inference.

### Decisions Made

| Date       | Decision                                           | Rationale                                            |
| ---------- | -------------------------------------------------- | ---------------------------------------------------- |
| 2026-01-15 | Remove `lifetimes:` and `types:` in signature mode | Simplifies API; values are now embedded in signature |
| 2026-01-15 | No deprecation period                              | Pre-1.0 library; breaking change is acceptable       |
| 2026-01-15 | Keep explicit `kind:` mode                         | Needed for advanced cases with custom Kind traits    |

### Blockers

| Issue | Status | Resolution |
| ----- | ------ | ---------- |

### Open Questions

1. Should complex types like `impl Trait` be supported in signatures?
   - Proposed: Yes, if `syn::Type` can parse them; test during implementation
2. Should empty signatures be allowed: `signature: ()`?
   - Proposed: Yes, for zero-argument Kinds
