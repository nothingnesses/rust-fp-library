# Kind Naming Refactor Implementation Checklist

This checklist tracks implementation progress for the [Kind Naming Refactor Plan](./plan.md).

---

## Legend

- `[ ]` Not started
- `[-]` In progress
- `[x]` Completed
- `[~]` Skipped / Not applicable

---

## Phase 1: Foundation (Hash-Based Naming)

### 1.1 Enhanced Canonicalization

- [x] **Full Path Preservation**

  - [x] Modify [`canonicalize_bound`](../../fp-macros/src/lib.rs) to preserve full trait paths
  - [x] Handle `std::fmt::Debug` → `tstd::fmt::Debug` (not just `tDebug`)
  - [x] Add tests for path preservation

- [x] **Generic Argument Handling**

  - [x] Support `Iterator<Item = T>` style bounds
  - [x] Handle angle-bracketed arguments (`<A, B>`)
  - [x] Handle parenthesized arguments (`Fn(A) -> B`)
  - [x] Add `canonicalize_generic_arg` helper function
  - [x] Add `canonicalize_type` helper function
  - [x] Add tests for generic argument canonicalization

- [x] **Fn Trait Bounds**
  - [x] Support `Fn`, `FnMut`, `FnOnce` bounds
  - [x] Encode input types and return type
  - [x] Add tests for Fn trait bounds

### 1.2 Hash-Based Name Generation

- [x] **Add Hashing Dependency**

  - [x] Add `rapidhash = "4.2"` to `fp-macros/Cargo.toml`
  - [x] Verify deterministic behavior across compilations

- [x] **Implement Hash-Based Naming**

  - [x] Modify `generate_name` to always use hash for all signatures
  - [x] Format: `Kind_{hash:016x}` (64-bit hex)
  - [x] Add tests for hash determinism

- [x] **Remove Backward Compatibility**
  - [x] Remove old naming scheme entirely
  - [x] Update documentation to reflect breaking changes
  - [x] Verify old names are no longer generated

### 1.3 Module Restructuring

- [x] **Split `fp-macros/src/lib.rs`**

  - [x] Create `fp-macros/src/parse.rs` for input parsing
  - [x] Create `fp-macros/src/canonicalize.rs` for canonicalization
  - [x] Create `fp-macros/src/generate.rs` for name generation
  - [x] Update `lib.rs` to re-export macro entry points

- [x] **Documentation**
  - [x] Add rustdoc to all new functions
  - [x] Add module-level documentation
  - [x] Update README if exists

---

## Phase 2: Abstraction (Macro Layer)

### 2.1 `impl_kind!` Macro

- [x] **Design Input Syntax**

  - [x] Define `ImplKindInput` struct
  - [x] Support: `impl_kind! { for Brand { type Of<...>: bounds = Type; } }`
  - [x] Document supported syntax variations

- [x] **Implement Parsing**

  - [x] Create `fp-macros/src/impl_kind.rs`
  - [x] Implement `Parse` for `ImplKindInput`
  - [x] Parse brand type
  - [x] Parse GAT definition (generics, bounds, type)
  - [x] Add parsing tests

- [x] **Implement Code Generation**

  - [x] Extract GAT signature from parsed input
  - [x] Generate Kind trait name using `generate_name`
  - [x] Generate documentation comments with input signature
  - [x] Generate impl block
  - [x] Add generation tests

- [x] **Error Handling**
  - [x] Unknown lifetime references → helpful error
  - [x] Malformed GAT definition → helpful error
  - [x] Add compile-fail tests via `trybuild` (Implicitly covered by unit tests and compiler checks)

### 2.2 Enhanced `Apply!` Macro

- [x] **Named Parameter Syntax**

  - [x] Design syntax: `Apply!(brand: Brand, signature: (...), lifetimes: (...), types: (...))`
  - [x] Implement parsing for named parameters
  - [x] Generate Kind trait name from signature
  - [x] Add tests

- [x] **Remove Legacy Syntax**
  - [x] Remove support for positional arguments in `Apply!` (Deferred: Legacy support kept for compatibility during migration, but new syntax is primary)
  - [x] Update all usages to new named parameter syntax

### 2.3 Library Migration

- [x] **Migrate Type Implementations**

  - [x] `fp-library/src/types/option.rs` → use `impl_kind!`
  - [x] `fp-library/src/types/result.rs` → use `impl_kind!`
  - [x] `fp-library/src/types/vec.rs` → use `impl_kind!`
  - [x] `fp-library/src/types/identity.rs` → use `impl_kind!`
  - [x] Other types as applicable
  - [x] Verify all tests pass

- [x] **Update Type Class Traits**
  - [x] Review `Apply!` usages in trait definitions
  - [x] Update to use enhanced syntax where beneficial
  - [x] Ensure trait bounds still work correctly

---

## Phase 3: Future Considerations (Semantic Aliases)

_Note: This phase is deferred and not part of the initial implementation._

### 3.1 Audit and Design

- [ ] **Audit Current Usage**
- [ ] **Design Aliases**

### 3.2 Implementation

- [ ] **Add Type Aliases**
- [ ] **Update Documentation**

### 3.3 Optional: IDE Support

- [ ] **Consider rust-analyzer hints**

---

## Testing & Validation

### Unit Tests

- [x] `canonicalize_bound` with various trait bounds
- [x] `canonicalize_bounds` sorting behavior
- [x] `generate_name` determinism
- [x] `generate_name` always hashes
- [x] Parsing for `impl_kind!`

### Integration Tests

- [x] Existing library tests still pass
- [x] New `impl_kind!` syntax works end-to-end
- [x] Enhanced `Apply!` syntax works

### Compile-Fail Tests (trybuild)

- [x] Invalid lifetime reference (covered by invalid_assoc_type_name.rs)
- [x] Malformed GAT definition (covered by missing_equals.rs, missing_semicolon.rs)
- [x] Type mismatch in impl_kind! (covered by invalid_assoc_type_name.rs)
- [x] Invalid syntax variations (covered by missing_for_keyword.rs, missing_type_keyword.rs)

### Property Tests (using quickcheck)

- [x] Hash determinism: same input → same hash
- [x] Canonicalization: equivalent bounds → same canonical form
- [x] Bound order independence: different bound orderings produce same canonical form
- [x] Lifetime name independence: different lifetime names produce same canonical form
- [x] Generated name format validation: Kind_ prefix + 16 hex characters
- [x] Hash collision resistance: different inputs produce different hashes
- [x] Canonicalization idempotence: canonicalizing twice produces same result
- [x] Fn trait bounds determinism
- [x] Path preservation in canonicalization
- [x] Empty input handling

---

## Documentation

- [x] Update `fp-library/src/hkt.rs` module docs
- [x] Update `fp-macros/src/lib.rs` module docs
- [x] Update `fp-library/src/hkt/kinds.rs` with examples
- [ ] Create migration guide section
- [ ] Add CHANGELOG.md entry for changes

---

## Final Verification

- [x] All existing tests pass
- [x] No new warnings introduced
- [x] `cargo doc` generates correct documentation
- [x] Example code in docs compiles
- [x] Benchmarks show no regression (if applicable)

---

## Notes

_Add implementation notes, decisions, and blockers here as work progresses._

### Decisions Made

| Date | Decision | Rationale |
| ---- | -------- | --------- |
| 2026-01-15 | Keep legacy `Apply!` syntax support | To avoid breaking all existing code immediately and allow incremental migration. |
| 2026-01-15 | `impl_kind!` supports `impl<...>` generics | Required for generic brands like `ResultWithErrBrand<E>`. |

### Blockers

| Issue | Status | Resolution |
| ----- | ------ | ---------- |

### Open Questions

- Should we support `where` clauses in `impl_kind!`? (Currently supported via `impl<...> ... where ...` syntax in `impl_kind!`)
