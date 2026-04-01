# Verification Summary

Five independent agents audited the implementation of `plan.md` against the actual code. This document synthesizes their findings.

## Overall Assessment

No correctness bugs were found. The code compiles cleanly, passes all tests, and has no clippy or doc warnings. Phases 1 (Stack Safety) and 2 (API Parity) are fully and correctly implemented. The gaps are in testing coverage, benchmark completeness, and scope of `Debug` implementations.

## Issues by Consensus

### Flagged by all 5 agents

1. **Missing `Debug` for `Coyoneda` and `CoyonedaExplicit`.** Plan step 6.3 says "all four Coyoneda files." Only `RcCoyoneda` and `ArcCoyoneda` have `Debug` impls.

2. **Missing compile-fail test: `Coyoneda` is `!Clone`.** Plan step 4.4 specifies this test.

3. **Missing compile-fail test: `ArcCoyoneda` with `!Send` payload.** Plan step 4.4 specifies this test.

4. **Missing `ArcCoyoneda` repeated `lower_ref` benchmark.** Plan step 5.2 calls for multiple `lower_ref` measurement; only `RcCoyoneda` has the `3x_lower` benchmark.

5. **Missing "clone + map + lower_ref" benchmark pattern.** Plan steps 5.1 and 5.2 both specify this; neither variant has it.

### Flagged by 4 agents

6. **Missing Foldable property test for `ArcCoyoneda`.** Plan step 4.2 calls for Foldable consistency tests across all brands; `RcCoyoneda` has one but `ArcCoyoneda` does not.

### Flagged by 3 agents

7. **`ArcCoyonedaNewLayer` unsafe `Send/Sync` impls are overly broad.** The struct stores `fb: F::Of<'a, B>` directly (not behind `Arc`), but the `unsafe impl Send/Sync` does not condition on `F::Of<'a, B>: Send/Sync`, unlike `ArcCoyonedaBase` which correctly has conditional bounds. Not currently exploitable (the struct is private and the only constructor enforces bounds), but inconsistent and the compile-time assertion for this type is vacuous.

8. **Unnecessary `stacker` wrapping in `NewLayer::lower_ref`.** Both `RcCoyonedaNewLayer` and `ArcCoyonedaNewLayer` wrap their `lower_ref` in `stacker::maybe_grow`, but these are base-level layers with no recursion. The plan only specified stacker for `MapLayer` variants.

### Flagged by 2 agents

9. **`ArcCoyoneda::apply` doc example demonstrates `lift2` instead of `apply`.** The example was changed to avoid `Send/Sync` issues with `CloneableFn` wrappers, but the result is a misleading example.

10. **Missing property tests for `Coyoneda` and `CoyonedaExplicit` brands.** Plan step 4.1 specifies `CoyonedaBrand<VecBrand>` and `CoyonedaExplicitBrand<VecBrand, _>`.

11. **Collapse stack safety tests are shallow (depth 20).** Insufficient to demonstrate actual overflow prevention; only proves the API works, not that it mitigates stack overflow.

## Categorization

| Category           | Issues               | Description                                   |
| ------------------ | -------------------- | --------------------------------------------- |
| Soundness concern  | #7                   | `ArcCoyonedaNewLayer` unsafe impl bounds      |
| Missing tests      | #2, #3, #6, #10, #11 | Compile-fail, property, and stack safety gaps |
| Missing benchmarks | #4, #5               | Arc repeated lower, clone+map+lower pattern   |
| Missing impls      | #1                   | Debug for Coyoneda/CoyonedaExplicit           |
| Code quality       | #8, #9               | Unnecessary stacker, misleading doc example   |
