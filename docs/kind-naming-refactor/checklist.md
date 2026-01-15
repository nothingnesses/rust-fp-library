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

- [ ] **Full Path Preservation**

  - [ ] Modify [`canonicalize_bound`](../../fp-macros/src/lib.rs) to preserve full trait paths
  - [ ] Handle `std::fmt::Debug` → `tstd::fmt::Debug` (not just `tDebug`)
  - [ ] Add tests for path preservation

- [ ] **Generic Argument Handling**

  - [ ] Support `Iterator<Item = T>` style bounds
  - [ ] Handle angle-bracketed arguments (`<A, B>`)
  - [ ] Handle parenthesized arguments (`Fn(A) -> B`)
  - [ ] Add `canonicalize_generic_arg` helper function
  - [ ] Add `canonicalize_type` helper function
  - [ ] Add tests for generic argument canonicalization

- [ ] **Fn Trait Bounds**
  - [ ] Support `Fn`, `FnMut`, `FnOnce` bounds
  - [ ] Encode input types and return type
  - [ ] Add tests for Fn trait bounds

### 1.2 Hash-Based Name Generation

- [ ] **Add Hashing Dependency**

  - [ ] Add `rapidhash = "4.2"` to `fp-macros/Cargo.toml`
  - [ ] Verify deterministic behavior across compilations

- [ ] **Implement Hash-Based Naming**

  - [ ] Modify `generate_name` to always use hash for all signatures
  - [ ] Format: `Kind_{hash:016x}` (64-bit hex)
  - [ ] Add tests for hash determinism

- [ ] **Remove Backward Compatibility**
  - [ ] Remove old naming scheme entirely
  - [ ] Update documentation to reflect breaking changes
  - [ ] Verify old names are no longer generated

### 1.3 Module Restructuring

- [ ] **Split `fp-macros/src/lib.rs`**

  - [ ] Create `fp-macros/src/parse.rs` for input parsing
  - [ ] Create `fp-macros/src/canonicalize.rs` for canonicalization
  - [ ] Create `fp-macros/src/generate.rs` for name generation
  - [ ] Update `lib.rs` to re-export macro entry points

- [ ] **Documentation**
  - [ ] Add rustdoc to all new functions
  - [ ] Add module-level documentation
  - [ ] Update README if exists

---

## Phase 2: Abstraction (Macro Layer)

### 2.1 `impl_kind!` Macro

- [ ] **Design Input Syntax**

  - [ ] Define `ImplKindInput` struct
  - [ ] Support: `impl_kind! { for Brand { type Of<...>: bounds = Type; } }`
  - [ ] Document supported syntax variations

- [ ] **Implement Parsing**

  - [ ] Create `fp-macros/src/impl_kind.rs`
  - [ ] Implement `Parse` for `ImplKindInput`
  - [ ] Parse brand type
  - [ ] Parse GAT definition (generics, bounds, type)
  - [ ] Add parsing tests

- [ ] **Implement Code Generation**

  - [ ] Extract GAT signature from parsed input
  - [ ] Generate Kind trait name using `generate_name`
  - [ ] Generate documentation comments with input signature
  - [ ] Generate impl block
  - [ ] Add generation tests

- [ ] **Error Handling**
  - [ ] Unknown lifetime references → helpful error
  - [ ] Malformed GAT definition → helpful error
  - [ ] Add compile-fail tests via `trybuild`

### 2.2 Enhanced `Apply!` Macro

- [ ] **Named Parameter Syntax**

  - [ ] Design syntax: `Apply!(brand: Brand, signature: (...), lifetimes: (...), types: (...))`
  - [ ] Implement parsing for named parameters
  - [ ] Generate Kind trait name from signature
  - [ ] Add tests

- [ ] **Remove Legacy Syntax**
  - [ ] Remove support for positional arguments in `Apply!`
  - [ ] Update all usages to new named parameter syntax

### 2.3 Library Migration

- [ ] **Migrate Type Implementations**

  - [ ] `fp-library/src/types/option.rs` → use `impl_kind!`
  - [ ] `fp-library/src/types/result.rs` → use `impl_kind!`
  - [ ] `fp-library/src/types/vec.rs` → use `impl_kind!`
  - [ ] `fp-library/src/types/identity.rs` → use `impl_kind!`
  - [ ] Other types as applicable
  - [ ] Verify all tests pass

- [ ] **Update Type Class Traits**
  - [ ] Review `Apply!` usages in trait definitions
  - [ ] Update to use enhanced syntax where beneficial
  - [ ] Ensure trait bounds still work correctly

---

## Phase 3: Future Considerations (Semantic Aliases)

*Note: This phase is deferred and not part of the initial implementation.*

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

- [ ] `canonicalize_bound` with various trait bounds
- [ ] `canonicalize_bounds` sorting behavior
- [ ] `generate_name` determinism
- [ ] `generate_name` always hashes
- [ ] Parsing for `impl_kind!`

### Integration Tests

- [ ] Existing library tests still pass
- [ ] New `impl_kind!` syntax works end-to-end
- [ ] Enhanced `Apply!` syntax works

### Compile-Fail Tests (trybuild)

- [ ] Invalid lifetime reference
- [ ] Malformed GAT definition
- [ ] Type mismatch in impl_kind!
- [ ] Invalid syntax variations

### Property Tests (if using quickcheck/proptest)

- [ ] Hash determinism: same input → same hash
- [ ] Canonicalization: equivalent bounds → same canonical form
- [ ] Round-trip: parse → generate → parse matches

---

## Documentation

- [ ] Update `fp-library/src/hkt.rs` module docs
- [ ] Update `fp-macros/src/lib.rs` module docs
- [ ] Update `fp-library/src/hkt/kinds.rs` with examples
- [ ] Create migration guide section
- [ ] Add CHANGELOG.md entry for changes

---

## Final Verification

- [ ] All existing tests pass
- [ ] No new warnings introduced
- [ ] `cargo doc` generates correct documentation
- [ ] Example code in docs compiles
- [ ] Benchmarks show no regression (if applicable)

---

## Notes

_Add implementation notes, decisions, and blockers here as work progresses._

### Decisions Made

| Date | Decision | Rationale |
| ---- | -------- | --------- |

### Blockers

| Issue | Status | Resolution |
| ----- | ------ | ---------- |

### Open Questions

- Should we support `where` clauses in `impl_kind!`?
