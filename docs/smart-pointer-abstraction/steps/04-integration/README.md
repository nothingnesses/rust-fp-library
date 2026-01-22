# Step 4: Integration & Polish

This step focuses on integrating the changes, updating documentation, and ensuring the codebase is clean and consistent.

## Goals

1.  Update module re-exports in `fp-library/src/classes.rs`, `fp-library/src/types.rs`, and `fp-library/src/functions.rs`.
2.  Update `docs/std-coverage-checklist.md`.
3.  Update `docs/architecture.md` with new patterns.
4.  Ensure all tests pass.
5.  Run clippy and fix warnings.
6.  Generate and review documentation.

## Technical Design

### Module Re-exports

We need to ensure the new modules are properly exposed and the old ones are removed.

```rust
// fp-library/src/classes.rs
pub mod pointer;
pub mod try_semigroup;
pub mod try_monoid;
pub mod send_defer;
// ... re-export traits ...

// fp-library/src/types.rs
pub mod rc_ptr;
pub mod arc_ptr;
pub mod fn_brand;
// ... remove rc_fn and arc_fn ...

// fp-library/src/functions.rs
// ... re-export pointer_new, ref_counted_new, send_ref_counted_new ...
```

### Documentation Updates

The architecture documentation needs to reflect the new pointer hierarchy and the shared memoization semantics of `Lazy`.

-   **Pointer Hierarchy**: Document `Pointer`, `RefCountedPointer`, `SendRefCountedPointer`.
-   **Lazy Evaluation**: Explain the shift from value semantics to shared semantics, the `LazyConfig` pattern, and thread safety guarantees.
-   **Function Brands**: Explain the `FnBrand<P>` pattern and how it generalizes `RcFnBrand` and `ArcFnBrand`.

### Testing & Verification

-   **Unit Tests**: Verify basic functionality of new pointer types and `Lazy`.
-   **Integration Tests**: Verify interaction between `Lazy`, `FnBrand`, and other library types.
-   **Clippy**: Ensure code quality and adherence to Rust idioms.

## Checklist

- [ ] Update module re-exports in `fp-library/src/classes.rs`
- [ ] Update module re-exports in `fp-library/src/types.rs`
- [ ] Update module re-exports in `fp-library/src/functions.rs`
- [ ] Update `docs/std-coverage-checklist.md`
- [ ] Update `docs/architecture.md` with new patterns
- [ ] Ensure all tests pass
- [ ] Run clippy and fix warnings
- [ ] Generate and review documentation
