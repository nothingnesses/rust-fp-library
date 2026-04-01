# Verification Report: Coyoneda Improvement Plan

This document audits the implementation against every numbered step in `plan.md` and every issue in `summary.md`.

---

## Phase 1: Stack Safety

### 1.1 Add `stacker` support to `RcCoyonedaMapLayer::lower_ref`

**Status: Implemented.**

`rc_coyoneda.rs` lines 194-207 wrap the body in `stacker::maybe_grow(32 * 1024, 1024 * 1024, || { ... })` behind `#[cfg(feature = "stacker")]`, with a non-stacker fallback. Matches the pattern from `coyoneda.rs`.

### 1.2 Add `stacker` support to `ArcCoyonedaMapLayer::lower_ref`

**Status: Implemented.**

`arc_coyoneda.rs` lines 252-266. Same pattern as 1.1.

### 1.3 Add `collapse` method to `RcCoyoneda`

**Status: Implemented.**

`rc_coyoneda.rs` lines 406-411. Signature matches the plan: takes `&self`, returns `RcCoyoneda<'a, F, A>`, requires `F: Functor` and `F::Of<'a, A>: Clone`. Body is `RcCoyoneda::lift(self.lower_ref())`. Documentation explains the purpose.

### 1.4 Add `collapse` method to `ArcCoyoneda`

**Status: Implemented.**

`arc_coyoneda.rs` lines 484-489. Takes `&self`, returns `ArcCoyoneda<'a, F, A>`, requires `F: Functor` and `F::Of<'a, A>: Clone + Send + Sync`. Matches the plan.

### 1.5 Document stack overflow risk

**Status: Implemented.**

Both `rc_coyoneda.rs` (lines 18-31) and `arc_coyoneda.rs` (lines 16-29) have "Stack safety" sections in module docs listing all three mitigations (stacker, collapse, CoyonedaExplicit).

---

## Phase 2: API Parity

### 2.1 Add `new(f, fb)` constructor to `RcCoyoneda`

**Status: Implemented (option b).**

`rc_coyoneda.rs` lines 216-266 define `RcCoyonedaNewLayer` with direct `RcCoyonedaLowerRef` implementation. The `new` method is at lines 506-516. Correctly requires `F::Of<'a, B>: Clone`. The new layer also includes `stacker` support (lines 253-264), which goes beyond the plan (the plan only mentioned stacker for `RcCoyonedaMapLayer::lower_ref`). This is a good addition since `RcCoyonedaNewLayer::lower_ref` also recurses (though only one level, the stacker guard is harmless and consistent).

### 2.2 Add `new(f, fb)` constructor to `ArcCoyoneda`

**Status: Implemented (option b).**

`arc_coyoneda.rs` lines 274-348 define `ArcCoyonedaNewLayer`. The `new` method at lines 586-596 requires `Send + Sync` on the function and `Clone + Send + Sync` on `F::Of<'a, B>`. Includes `unsafe impl Send/Sync` with safety comments (lines 281-303) and stacker support (lines 335-347).

**Finding:** The `unsafe impl Send/Sync` for `ArcCoyonedaNewLayer` (lines 290-303) does not condition `Send` on `F::Of<'a, B>: Send` like `ArcCoyonedaBase` does. Instead, it is unconditional (only `F: Kind`). The SAFETY comment at line 281-283 claims `fb` is bounded `Send + Sync` via the `ArcCoyonedaLowerRef` impl's where clause. This reasoning is correct: the struct can only be coerced to `Arc<dyn ArcCoyonedaLowerRef>` when the `ArcCoyonedaLowerRef` impl's where clause is satisfied (which requires `F::Of<'a, B>: Clone + Send + Sync`). However, the `unsafe impl Send` applies to the _struct itself_, not just when used through the trait object. In theory, someone could construct an `ArcCoyonedaNewLayer` with a `!Send` payload type `B` and the struct would still be `Send`. In practice, the struct is private (`struct ArcCoyonedaNewLayer` without `pub`), so only the module's own code can construct it, and the only construction site is `ArcCoyoneda::new` which requires `Clone + Send + Sync` on `fb`. The soundness is maintained, but the `unsafe impl` is technically broader than necessary. The compile-time assertion at lines 962-965 checks this but does not add the `Send + Sync` bound on `fb`, which means it verifies the `unsafe impl` is in place but not that it is conditionally correct. This is the same approach taken for `ArcCoyonedaMapLayer` and is acceptable given the struct is private.

### 2.3 Add `hoist` to `RcCoyoneda`

**Status: Implemented.**

`rc_coyoneda.rs` lines 554-562. Signature matches: takes `self`, a `NaturalTransformation<F, G>`, requires `F: Functor` and `G::Of<'a, A>: Clone`. Implementation lowers, transforms, and re-lifts.

### 2.4 Add `hoist` to `ArcCoyoneda`

**Status: Implemented.**

`arc_coyoneda.rs` lines 634-642. Adds `Clone + Send + Sync` bound on `G::Of<'a, A>`. Correct.

### 2.5 Add inherent methods to `RcCoyoneda`

**Status: Implemented.**

All four inherent methods are present:

- `pure` (line 583): `F: Pointed`, `F::Of<'a, A>: Clone`. Matches plan.
- `bind` (lines 616-624): `F: Functor + Semimonad`, `F::Of<'a, B>: Clone`. Matches plan.
- `apply` (lines 662-670): `F: Functor + Semiapplicative`, `F::Of<'a, C>: Clone`. Matches plan.
- `lift2` (lines 696-706): `F: Functor + Lift`, `A: Clone`, `F::Of<'a, C>: Clone`. Matches plan.

**Finding on `apply` bounds:** The plan specifies `F::Of<'a, B>: Clone` for `apply`, but the implementation uses `F::Of<'a, C>: Clone` (where `C` is the output type). This is correct because `apply` needs to re-lift the _result_ (`C` type) into `RcCoyoneda`, not the input `B`. The plan's bound listing appears to have a typo; the code is correct.

### 2.6 Add inherent methods to `ArcCoyoneda`

**Status: Implemented.**

All four inherent methods are present with `Clone + Send + Sync` bounds:

- `pure` (line 663): `F: Pointed`, `F::Of<'a, A>: Clone + Send + Sync`. Correct.
- `bind` (lines 696-703): `F: Functor + Semimonad`, `F::Of<'a, B>: Clone + Send + Sync`. Correct.
- `apply` (lines 740-748): `F: Functor + Semiapplicative`, `F::Of<'a, C>: Clone + Send + Sync`. Correct.
- `lift2` (lines 774-784): `F: Functor + Lift`, `A: Clone`, `F::Of<'a, C>: Clone + Send + Sync`. Correct.

**Finding (documentation quality):** The `apply` method's doc example on `ArcCoyoneda` (lines 726-739) does NOT demonstrate `apply` at all. Instead, it demonstrates `lift2` and includes a comment explaining why. While the explanation is valid (the `CloneableFn` brand's `Of` type needs to be `Send + Sync`), having a doc example that does not exercise the documented method is a quality issue. A proper example using `ArcFnBrand` would be better, or the comment should more clearly state that this is a usage recommendation rather than an example of `apply`.

---

## Phase 3: Performance Optimization (Removed)

**Status: Correctly removed.** No inline function changes were attempted. The plan correctly documents this as infeasible.

---

## Phase 4: Testing

### 4.1 Property-based tests for Functor laws

**Status: Implemented.**

`RcCoyoneda`: Property tests in `rc_coyoneda.rs` lines 1003-1053 cover:

- Identity law with `VecBrand` and `OptionBrand`.
- Composition law with `VecBrand` and `OptionBrand`.

`ArcCoyoneda`: Property tests in `arc_coyoneda.rs` lines 1035-1065 cover:

- Identity law with `VecBrand` and `OptionBrand`.
- Composition law with `VecBrand` and `OptionBrand`.

**Missing:** The plan also mentions testing `CoyonedaBrand<VecBrand>` and `CoyonedaExplicitBrand<VecBrand, _>`. There are no QuickCheck property tests for the original `Coyoneda` or `CoyonedaExplicit` brands. The `coyoneda_explicit.rs` has example-based functor law tests (lines 898-916) but no randomized QuickCheck tests.

### 4.2 Property-based tests for Foldable laws

**Status: Partially implemented.**

`RcCoyoneda` has `foldable_consistency_vec` (lines 1055-1067) verifying `fold_map` consistency between `RcCoyonedaBrand<VecBrand>` and direct `VecBrand`.

`ArcCoyoneda` does NOT have a corresponding Foldable property test. The `arc_coyoneda.rs` property module contains only Functor identity, Functor composition, and collapse tests (lines 1035-1076).

### 4.3 Stack overflow tests

**Status: Implemented.**

`fp-library/tests/stack_safety.rs` lines 162-239 contain:

- `test_rc_coyoneda_collapse_resets_depth` (lines 169-185): Tests collapse correctness with 20+20 maps.
- `test_arc_coyoneda_collapse_resets_depth` (lines 188-204): Same for Arc.
- `test_rc_coyoneda_deep_chain_with_stacker` (lines 211-223): 1,000 maps with `OptionBrand`, gated on `#[cfg(feature = "stacker")]`.
- `test_arc_coyoneda_deep_chain_with_stacker` (lines 226-239): Same for Arc.

**Finding:** The collapse tests use a depth of only 20, which would not overflow the stack even without collapse. A depth like 500 (below the overflow threshold but meaningful) would better demonstrate that collapse actually resets depth. However, the test does verify functional correctness of `collapse` (the value `40` is correct).

### 4.4 Compile-fail tests for Send/Sync soundness

**Status: Partially implemented.**

- `RcCoyoneda` is `!Send`: Implemented in `tests/ui/rc_coyoneda_not_send.rs` with matching `.stderr` file.
- `ArcCoyoneda` with `!Send` payload: NOT implemented. No compile-fail test verifying that `ArcCoyoneda` with a non-Send payload fails.
- `Coyoneda` is `!Clone`: NOT implemented. No compile-fail test verifying this.

### 4.5 Concurrent access tests for `ArcCoyoneda`

**Status: Implemented.**

`fp-library/tests/thread_safety.rs` lines 83-120 contain:

- `test_arc_coyoneda_concurrent_lower_ref` (lines 84-99): 4 threads concurrently calling `lower_ref` on clones of the same `ArcCoyoneda`.
- `test_arc_coyoneda_shared_across_threads` (lines 101-120): Same pattern using `Arc<ArcCoyoneda>` for shared ownership.

---

## Phase 5: Benchmarks

### 5.1 Add `RcCoyoneda` benchmarks

**Status: Implemented.**

`benches/benchmarks/coyoneda.rs` lines 73-85: Map chain + `lower_ref` at various depths.
Lines 103-119: `RcCoyoneda_3x_lower` benchmark for repeated `lower_ref` cost.

**Missing:** The plan mentions a "Clone + map + lower_ref pattern" benchmark. This is not present. Only chain+single-lower and repeated-lower patterns are benchmarked.

### 5.2 Add `ArcCoyoneda` benchmarks

**Status: Partially implemented.**

Lines 87-100: Map chain + `lower_ref` at various depths.

**Missing:** No repeated `lower_ref` benchmark for `ArcCoyoneda` (only `RcCoyoneda` has the `_3x_lower` benchmark). No clone-then-lower benchmark.

---

## Phase 6: Documentation and Polish

### 6.1 Enable documentation validation on `ArcCoyoneda`

**Status: Implemented.**

`arc_coyoneda.rs` line 56 uses `#[fp_macros::document_module]` (no `no_validation`).

### 6.2 Add `From` conversions for Rc/Arc variants

**Status: Implemented.**

- `From<RcCoyoneda<'a, F, A>> for Coyoneda<'a, F, A>`: `rc_coyoneda.rs` lines 868-896.
- `From<ArcCoyoneda<'a, F, A>> for Coyoneda<'a, F, A>`: `arc_coyoneda.rs` lines 906-934.

Both require `F: Functor`, use `lower_ref()` + `Coyoneda::lift()`, and include documentation with examples.

### 6.3 Add `Debug` implementations

**Status: Partially implemented.**

- `RcCoyoneda`: Implemented at `rc_coyoneda.rs` lines 829-859. Outputs `RcCoyoneda(<opaque>)`.
- `ArcCoyoneda`: Implemented at `arc_coyoneda.rs` lines 867-897. Outputs `ArcCoyoneda(<opaque>)`.
- `Coyoneda`: NOT implemented. No `Debug` impl found in `coyoneda.rs`.
- `CoyonedaExplicit`: NOT implemented. No `Debug` impl found in `coyoneda_explicit.rs`.

The plan says "All four Coyoneda files" but only two have `Debug`.

### 6.4 Add inherent `fold_map(&self)` to Rc/Arc variants

**Status: Implemented.**

- `RcCoyoneda::fold_map`: `rc_coyoneda.rs` lines 441-450. Takes `&self`, delegates to `F::fold_map` after `lower_ref()`.
- `ArcCoyoneda::fold_map`: `arc_coyoneda.rs` lines 519-528. Same pattern.

Both include documentation and examples.

### 6.5 Document fusion barriers in `CoyonedaExplicit`

**Status: Implemented.**

- `traverse` (line 382): "This is a fusion barrier: all accumulated maps are composed into the traversal function..."
- `apply` (line 440): "This is a fusion barrier: it calls `lower()` on both arguments..."
- `bind` (line 495): "This is a fusion barrier: all accumulated maps are composed into the bind callback..."

All three methods now document their fusion barrier status.

### 6.6 Document `CoyonedaExplicitBrand` Functor re-boxing cost

**Status: Implemented.**

`coyoneda_explicit.rs` lines 738-742 contain a "Note:" paragraph explaining that each brand-level `map` allocates a `Box` and that zero-cost fusion is only available via the inherent `CoyonedaExplicit::map` method.

### 6.7 Document `CoyonedaExplicit::boxed()` loop overhead

**Status: Implemented.**

`coyoneda_explicit.rs` lines 541-546 document the O(k) per-element overhead when `boxed()` is used in a loop, and that the single-`F::map` advantage is only realized with static composition.

### 6.8 Harden `unsafe impl Send/Sync` on `ArcCoyonedaBase`

**Status: Implemented.**

Compile-time assertions at `arc_coyoneda.rs` lines 941-966 verify:

- `ArcCoyonedaBase` is `Send + Sync` when `Of<'a, A>: Send + Sync` (lines 946-951).
- `ArcCoyonedaMapLayer` is unconditionally `Send + Sync` (lines 953-958).
- `ArcCoyonedaNewLayer` is unconditionally `Send + Sync` (lines 960-965).

The assertion for `ArcCoyonedaBase` matches the plan's intended verification pattern. The plan also suggested verifying `ArcCoyonedaMapLayer` field types specifically (`Arc<dyn ArcCoyonedaLowerRef + 'a>` and `Arc<dyn Fn(B) -> A + Send + Sync + 'a>`), but the implementation asserts on the struct directly, which is equivalent and simpler.

### 6.9 Compile-time assertion for `ArcCoyonedaMapLayer` Send/Sync

**Status: Implemented.**

Covered by the same `const _` block at lines 953-958, asserting `ArcCoyonedaMapLayer` is `Send + Sync`. Also includes `ArcCoyonedaNewLayer` (not in the original plan but necessary due to step 2.2 adding the new layer type).

### 6.10 Document why `unsafe impl Send/Sync` is needed on `ArcCoyonedaMapLayer`

**Status: Implemented.**

`arc_coyoneda.rs` lines 183-201 contain a thorough SAFETY comment covering:

- Both fields are `Arc<dyn ... + Send + Sync>`.
- Compiler cannot auto-derive due to `F: Kind`.
- Adding `Send + Sync` to `Kind::Of` is not feasible.
- `SendKind` subtrait is not expressible.
- What depends on for soundness.
- What would break soundness.
- References compile-time assertions.

This matches all items listed in the plan.

### 6.11 Document brand-level trait impl limitations

**Status: Implemented.**

`RcCoyoneda`: `rc_coyoneda.rs` lines 717-732 contain a block comment explaining:

- The `Clone` bound blocker.
- Why `CoyonedaBrand` avoids it (consuming `Box<Self>`).
- That Rust disallows extra where clauses in trait impls.
- That inherent methods are provided instead.

`ArcCoyoneda`: `arc_coyoneda.rs` lines 795-810 contain a similar comment explaining both the `Clone` blocker and the `Send + Sync` limitation, referencing `rc_coyoneda.rs` for details.

---

## Summary of Issues in `summary.md` and Their Resolution

| Summary Issue                              | Status                  | Notes                                                                                       |
| ------------------------------------------ | ----------------------- | ------------------------------------------------------------------------------------------- |
| 1.1 No stack safety (5/5)                  | Addressed               | stacker + collapse + docs                                                                   |
| 1.2 Missing type class instances (5/5)     | Addressed (revised)     | Inherent methods instead of brand impls                                                     |
| 2.1 Missing API methods (5/5)              | Addressed               | `new`, `collapse`, `hoist` added                                                            |
| 2.2 No benchmarks (5/5)                    | Partially addressed     | Missing clone+lower, ArcCoyoneda repeated lower                                             |
| 2.3 `document_module(no_validation)` (5/5) | Addressed               | Validation enabled                                                                          |
| 2.4 No property-based tests (4/5)          | Partially addressed     | Missing Coyoneda/CoyonedaExplicit props, missing ArcCoyoneda Foldable props                 |
| 2.5 Unsafe Send/Sync verification (3/5)    | Mostly addressed        | Compile-time assertions added; missing compile-fail test for ArcCoyoneda with !Send payload |
| 2.6 Two allocations per map (2/5)          | Correctly not addressed | Documented as inherent limitation                                                           |
| 2.7 No conversions (5/5)                   | Addressed               | `From` impls added                                                                          |
| 3.1 No Debug (4/5)                         | Partially addressed     | Only Rc/Arc variants; Coyoneda and CoyonedaExplicit missing                                 |
| 3.2 Inherent fold_map (2/5)                | Addressed               |                                                                                             |
| 3.3 Consuming lower(self) (2/5)            | Correctly deferred      | Per plan                                                                                    |
| 3.4 CoyonedaExplicitBrand re-boxing (1/5)  | Addressed               | Documented                                                                                  |
| 3.5 Fusion barrier docs (2/5)              | Addressed               | traverse, bind, apply all documented                                                        |
| 3.6 boxed() loop overhead (1/5)            | Addressed               | Documented                                                                                  |
| 3.7 Improve unsafe comments (3/5)          | Addressed               | Comprehensive SAFETY comments                                                               |

---

## Flaws and Issues Found

### Bug: None found

The code logic appears correct. The lowering, collapsing, and inherent method implementations all follow the "lower, delegate, re-lift" pattern consistently.

### Documentation Issues

1. **`ArcCoyoneda::apply` doc example (arc_coyoneda.rs lines 726-739):** The doc example does not demonstrate `apply`; it demonstrates `lift2`. While this compiles and runs, it misleads readers about how to use `apply`. A proper example should either use `ArcFnBrand` to show actual `apply` usage, or the method should state that the example shows the recommended alternative (`lift2`).

2. **Missing `Debug` on `Coyoneda` and `CoyonedaExplicit`:** Plan step 6.3 says "All four Coyoneda files," but only `RcCoyoneda` and `ArcCoyoneda` have `Debug` implementations.

### Missing Tests

1. **No compile-fail test for `ArcCoyoneda` with `!Send` payload** (plan step 4.4): Only `RcCoyoneda` is `!Send` has a compile-fail test. The plan specifies a test proving `ArcCoyoneda` with `!Send` payload fails to compile when sent across threads.

2. **No compile-fail test for `Coyoneda` being `!Clone`** (plan step 4.4): The plan specifies verifying `Coyoneda` is `!Clone`.

3. **No Foldable property test for `ArcCoyoneda`** (plan step 4.2): `RcCoyoneda` has `foldable_consistency_vec` but `ArcCoyoneda` does not.

4. **No property tests for `Coyoneda` or `CoyonedaExplicit`** (plan step 4.1): The plan mentions testing `CoyonedaBrand<VecBrand>` and `CoyonedaExplicitBrand<VecBrand, _>`.

### Missing Benchmarks

1. **No "clone + map + lower_ref" benchmark** for `RcCoyoneda` (plan step 5.1).
2. **No repeated `lower_ref` benchmark** for `ArcCoyoneda` (plan step 5.2 says "same cases as 5.1").
3. **No clone-then-lower benchmark** for `ArcCoyoneda`.

### Minor Style Observation

The codebase follows project conventions correctly: hard tabs, no emoji/unicode, proper documentation attributes (`document_signature`, `document_type_parameters`, `document_parameters`, `document_returns`, `document_examples`), and the commit message style matches the repository's existing patterns.

---

## Overall Assessment

The implementation covers the high-priority and medium-priority items from the plan thoroughly. Phase 1 (Stack Safety) and Phase 2 (API Parity) are fully implemented. Phase 6 (Documentation and Polish) is largely complete with minor gaps. The main shortfalls are in Phase 4 (Testing) and Phase 5 (Benchmarks), where several specified test cases and benchmark scenarios are missing. No bugs or unsound code were found.

### Items fully implemented: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 6.1, 6.2, 6.4, 6.5, 6.6, 6.7, 6.8, 6.9, 6.10, 6.11

### Items partially implemented: 4.1, 4.2, 4.3, 4.4, 5.1, 5.2, 6.3

### Items not implemented: None (all steps have at least partial coverage)
