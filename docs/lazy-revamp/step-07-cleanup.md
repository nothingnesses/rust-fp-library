# Step 07: Cleanup & Integration

## Goal
Finalize the revamp by removing the old `Lazy` implementation and ensuring the library builds and tests pass with the new architecture.

## Files to Delete
- `fp-library/src/types/lazy.rs`

## Files to Modify
- `fp-library/src/types.rs`
- `fp-library/src/lib.rs` (if it references `Lazy`)

## Implementation Details

### Removal
Delete the `lazy.rs` file. This is a breaking change.

### Integration
Ensure `types.rs` no longer exports `Lazy`. Ensure any internal usage of `Lazy` (if any) is migrated to `Memo` or `Eval`.

## Tests
1.  **Full Suite**: Run `cargo test` to ensure all new and existing tests pass.
2.  **Benchmarks**: Run `cargo bench` (if applicable) to verify performance.

## Checklist
- [ ] Delete `fp-library/src/types/lazy.rs`
- [ ] Update `fp-library/src/types.rs` to remove `lazy` module and exports
- [ ] Check `fp-library/src/lib.rs` for any lingering references
- [ ] Run `cargo test` and fix any compilation errors
- [ ] Verify documentation builds with `cargo doc`
