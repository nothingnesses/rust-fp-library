# Coyoneda Plan Verification Report

This document audits the implementation against the plan (`plan.md`) and summary (`summary.md`), covering correctness, completeness, and adherence to project conventions.

---

## Phase 1: Stack Safety

### 1.1 `stacker` support in `RcCoyonedaMapLayer::lower_ref` -- IMPLEMENTED

File: `fp-library/src/types/rc_coyoneda.rs`, lines 194-207.

The `#[cfg(feature = "stacker")]` / `#[cfg(not(feature = "stacker"))]` guards are present and match the pattern from `CoyonedaMapLayer::lower`. The constants (32 _ 1024 red zone, 1024 _ 1024 stack size) match the existing coyoneda.rs values. Correct.

### 1.2 `stacker` support in `ArcCoyonedaMapLayer::lower_ref` -- IMPLEMENTED

File: `fp-library/src/types/arc_coyoneda.rs`, lines 252-266.

Same pattern as 1.1. Correct.

### 1.3 `collapse` method on `RcCoyoneda` -- IMPLEMENTED

File: `fp-library/src/types/rc_coyoneda.rs`, lines 406-411.

Signature matches the plan: takes `&self`, requires `F: Functor` and `F::Of<'a, A>: Clone`. Implementation is `RcCoyoneda::lift(self.lower_ref())`. Correct.

### 1.4 `collapse` method on `ArcCoyoneda` -- IMPLEMENTED

File: `fp-library/src/types/arc_coyoneda.rs`, lines 484-489.

Signature matches the plan: takes `&self`, requires `F: Functor` and `F::Of<'a, A>: Clone + Send + Sync`. Correct.

### 1.5 Stack safety documentation -- IMPLEMENTED

Both `rc_coyoneda.rs` (lines 18-31) and `arc_coyoneda.rs` (lines 16-29) have "Stack safety" sections in their module docs, covering all three mitigations (stacker, collapse, CoyonedaExplicit). Correct.

---

## Phase 2: API Parity

### 2.1 `new(f, fb)` constructor on `RcCoyoneda` -- IMPLEMENTED (Option b)

File: `fp-library/src/types/rc_coyoneda.rs`, lines 506-516.

Uses a dedicated `RcCoyonedaNewLayer` (lines 216-266), matching the plan's recommended option (b). The layer stores `fb` and `func: Rc<dyn Fn(B) -> A>` and implements `RcCoyonedaLowerRef` directly. The `lower_ref` implementation also has stacker support. Correct.

### 2.2 `new(f, fb)` constructor on `ArcCoyoneda` -- IMPLEMENTED (Option b)

File: `fp-library/src/types/arc_coyoneda.rs`, lines 586-596.

Uses a dedicated `ArcCoyonedaNewLayer` (lines 274-348). Has appropriate `Send + Sync` bounds. Correct.

**Soundness concern with `ArcCoyonedaNewLayer`:** See findings below.

### 2.3 `hoist` on `RcCoyoneda` -- IMPLEMENTED

File: `fp-library/src/types/rc_coyoneda.rs`, lines 554-562.

Signature matches the plan. Takes `self`, requires `F: Functor`, `G::Of<'a, A>: Clone`. Correct.

### 2.4 `hoist` on `ArcCoyoneda` -- IMPLEMENTED

File: `fp-library/src/types/arc_coyoneda.rs`, lines 634-642.

Adds `Send + Sync` bound on `G::Of<'a, A>`. Correct.

### 2.5 Inherent methods on `RcCoyoneda` -- IMPLEMENTED

File: `fp-library/src/types/rc_coyoneda.rs`.

All four methods are present:

- `pure` (line 583): `F: Pointed`, `F::Of<'a, A>: Clone`. Matches plan.
- `bind` (lines 616-624): `F: Functor + Semimonad`, `F::Of<'a, B>: Clone`. Matches plan.
- `apply` (lines 662-670): `F: Functor + Semiapplicative`, `F::Of<'a, C>: Clone`. Matches plan.
- `lift2` (lines 696-706): `F: Functor + Lift`, `F::Of<'a, C>: Clone`, `A: Clone`. Matches plan.

All methods follow the "lower, delegate, re-lift" pattern. Documentation is thorough with examples.

### 2.6 Inherent methods on `ArcCoyoneda` -- IMPLEMENTED

File: `fp-library/src/types/arc_coyoneda.rs`.

All four methods are present with additional `Send + Sync` bounds:

- `pure` (line 663): Matches plan.
- `bind` (lines 696-703): Matches plan.
- `apply` (lines 740-748): Matches plan.
- `lift2` (lines 774-784): Matches plan.

Correct.

---

## Phase 3: Performance Optimization -- CORRECTLY REMOVED

The plan marked this phase as infeasible and removed it. No implementation expected. Correct.

---

## Phase 4: Testing

### 4.1 Property-based tests for Functor laws -- PARTIALLY IMPLEMENTED

Present for:

- `RcCoyoneda`: `rc_coyoneda.rs` lines 1003-1053 (identity and composition laws for Vec and Option).
- `ArcCoyoneda`: `arc_coyoneda.rs` lines 1035-1065 (identity and composition laws for Vec and Option).

Missing for:

- `CoyonedaBrand<VecBrand>` and `CoyonedaBrand<OptionBrand>`: no property tests in `coyoneda.rs`.
- `CoyonedaExplicitBrand<VecBrand, _>`: no property tests in `coyoneda_explicit.rs`.

The plan explicitly listed `CoyonedaBrand<VecBrand>` and `CoyonedaExplicitBrand<VecBrand, _>` as test targets.

### 4.2 Property-based tests for Foldable laws -- PARTIALLY IMPLEMENTED

Present for:

- `RcCoyoneda`: `rc_coyoneda.rs` lines 1056-1067 (`foldable_consistency_vec`).

Missing for:

- `ArcCoyoneda`: no Foldable property test despite having a `Foldable` implementation for `ArcCoyonedaBrand`.
- `Coyoneda` and `CoyonedaExplicit`: no Foldable property tests.

### 4.3 Stack overflow tests -- IMPLEMENTED

File: `fp-library/tests/stack_safety.rs`, lines 162-239.

Covers:

- `RcCoyoneda::collapse` resets depth (line 170).
- `ArcCoyoneda::collapse` resets depth (line 189).
- `RcCoyoneda` deep chain with stacker (line 212, `#[cfg(feature = "stacker")]`).
- `ArcCoyoneda` deep chain with stacker (line 228, `#[cfg(feature = "stacker")]`).

Depths are conservative (20 for collapse tests, 1000 for stacker tests). The plan mentioned "thousands of maps" but the stacker test uses 1000, which is reasonable for Option-based tests. Correct.

### 4.4 Compile-fail tests -- PARTIALLY IMPLEMENTED

Present:

- `RcCoyoneda` is `!Send`: `fp-library/tests/ui/rc_coyoneda_not_send.rs` and `.stderr`. Correct.

Missing:

- `ArcCoyoneda` with a `!Send` payload fails to compile when sent. The plan requested this.
- `Coyoneda` is `!Clone`. The plan requested this.

### 4.5 Concurrent access tests for `ArcCoyoneda` -- IMPLEMENTED

File: `fp-library/tests/thread_safety.rs`, lines 84-120.

Two tests:

- `test_arc_coyoneda_concurrent_lower_ref`: 4 threads calling `lower_ref` concurrently (line 84).
- `test_arc_coyoneda_shared_across_threads`: `Arc<ArcCoyoneda>` shared across 4 threads (line 102).

Both verify correct results. Correct.

---

## Phase 5: Benchmarks

### 5.1 `RcCoyoneda` benchmarks -- PARTIALLY IMPLEMENTED

File: `fp-library/benches/benchmarks/coyoneda.rs`.

Present:

- Map chain construction + single `lower_ref` (line 73, `RcCoyoneda` group).
- Multiple `lower_ref` calls (line 103, `RcCoyoneda_3x_lower`).

Missing:

- "Clone + map + lower_ref pattern" as specified in the plan.

### 5.2 `ArcCoyoneda` benchmarks -- PARTIALLY IMPLEMENTED

Present:

- Map chain construction + single `lower_ref` (line 88, `ArcCoyoneda` group).

Missing:

- Multiple `lower_ref` calls on the same value (no `ArcCoyoneda_3x_lower`).
- "Clone + map + lower_ref pattern."

---

## Phase 6: Documentation and Polish

### 6.1 Enable documentation validation on `ArcCoyoneda` -- IMPLEMENTED

File: `fp-library/src/types/arc_coyoneda.rs`, line 56.

Uses `#[fp_macros::document_module]` (without `no_validation`). Correct.

### 6.2 `From` conversions for Rc/Arc variants -- IMPLEMENTED

- `From<RcCoyoneda> for Coyoneda`: `rc_coyoneda.rs` line 868. Requires `F: Functor`. Correct.
- `From<ArcCoyoneda> for Coyoneda`: `arc_coyoneda.rs` line 906. Requires `F: Functor`. Correct.

### 6.3 `Debug` implementations -- IMPLEMENTED

- `RcCoyoneda`: `rc_coyoneda.rs` lines 829-858. Outputs `RcCoyoneda(<opaque>)`.
- `ArcCoyoneda`: `arc_coyoneda.rs` lines 867-896. Outputs `ArcCoyoneda(<opaque>)`.

The plan also mentioned `Coyoneda` and `CoyonedaExplicit` Debug implementations ("All four Coyoneda files"). These were not verified as those files were not listed as modified files, though the plan listed all four.

### 6.4 Inherent `fold_map(&self)` for Rc/Arc variants -- IMPLEMENTED

- `RcCoyoneda`: `rc_coyoneda.rs` lines 441-450.
- `ArcCoyoneda`: `arc_coyoneda.rs` lines 519-528.

Both take `&self` and delegate to `F::fold_map` via `lower_ref`. Correct.

### 6.5 Document fusion barriers in `CoyonedaExplicit` -- IMPLEMENTED

File: `fp-library/src/types/coyoneda_explicit.rs`.

- `traverse` (line 382): "This is a fusion barrier: all accumulated maps are composed into the traversal function and applied during the traversal."
- `bind` (line 495): "This is a fusion barrier: all accumulated maps are composed into the bind callback and applied during the bind."
- `apply` (line 440): already had the note prior to this plan.

Correct.

### 6.6 Document `CoyonedaExplicitBrand` Functor re-boxing cost -- IMPLEMENTED

File: `fp-library/src/types/coyoneda_explicit.rs`, lines 739-742.

The Functor impl for `CoyonedaExplicitBrand` includes a note: "each call through this brand-level `map` allocates a `Box` for the composed function (via `.boxed()`). Zero-cost fusion (no allocation per map) is only available via the inherent `CoyonedaExplicit::map` method." Correct.

### 6.7 Document `CoyonedaExplicit::boxed()` loop overhead -- IMPLEMENTED

File: `fp-library/src/types/coyoneda_explicit.rs`, lines 541-546.

The `boxed()` method's documentation explains: "When used in a loop (e.g., `coyo = coyo.map(f).boxed()` per iteration), each iteration creates a closure that captures the previous boxed function. The composed function chain has O(k) per-element overhead at `lower` time, matching `Coyoneda`'s cost profile." Correct.

### 6.8 Harden `unsafe impl Send/Sync` on `ArcCoyonedaBase` -- IMPLEMENTED

File: `fp-library/src/types/arc_coyoneda.rs`, lines 941-951.

Compile-time assertion verifies `ArcCoyonedaBase` is `Send + Sync` when `Of<'a, A>: Send + Sync`. The `unsafe impl` itself (lines 125-137) already has the correct conditional bounds. The assertion adds a regression guard. Correct.

### 6.9 Compile-time assertion for `ArcCoyonedaMapLayer` Send/Sync -- IMPLEMENTED

File: `fp-library/src/types/arc_coyoneda.rs`, lines 953-958.

Assertion verifies `ArcCoyonedaMapLayer` is `Send + Sync` unconditionally. Since both fields are `Arc<dyn ... + Send + Sync>`, this is correct.

### 6.10 Document why `unsafe impl Send/Sync` needed on `ArcCoyonedaMapLayer` -- IMPLEMENTED

File: `fp-library/src/types/arc_coyoneda.rs`, lines 183-201.

Thorough safety comment covering:

- Both fields are `Arc<dyn ... + Send + Sync>`.
- Why the compiler cannot auto-derive.
- Why `Send + Sync` on `Kind::Of` is infeasible.
- What would break soundness.
- Reference to compile-time assertions.

Correct and comprehensive.

### 6.11 Document brand-level trait impl limitations -- IMPLEMENTED

- `RcCoyonedaBrand`: `rc_coyoneda.rs` lines 717-732. Explains Functor and Foldable are implemented, but NOT Pointed/Lift/Semiapplicative/Semimonad, with the Clone bound blocker explained.
- `ArcCoyonedaBrand`: `arc_coyoneda.rs` lines 795-810. Explains only Foldable is implemented, with both the Send+Sync closure limitation and the Clone bound blocker.

Both mention inherent methods as the alternative. Correct.

---

## Bugs and Soundness Issues

### BUG: `ArcCoyonedaNewLayer` unsafe Send/Sync is overly broad

**Severity: Medium (mitigated by private visibility).**

File: `fp-library/src/types/arc_coyoneda.rs`, lines 290-303.

The `unsafe impl Send` and `unsafe impl Sync` for `ArcCoyonedaNewLayer` require only `F: Kind + 'a`, but the struct contains a field `fb: <F as Kind>::Of<'a, B>` which may not be `Send` or `Sync`. The safety comment (line 282) claims "`fb` is bounded `Send + Sync` via the ArcCoyonedaLowerRef impl's where clause," but this is incorrect reasoning: the `ArcCoyonedaLowerRef` impl's bounds constrain when `lower_ref` is available, not when the struct itself is `Send/Sync`.

Compare with `ArcCoyonedaBase` (lines 125-137), which correctly conditions its `unsafe impl Send` on `Of<'a, A>: Send` and `unsafe impl Sync` on `Of<'a, A>: Sync`.

**Why it does not currently cause harm:** The struct is private, and the only constructor (`ArcCoyoneda::new`) requires `Of<'a, B>: Clone + Send + Sync`, so a `!Send` value can never be stored in the `fb` field in practice. However, if internal code were to construct the struct without the `Send + Sync` bound on `fb`, the unsound `unsafe impl` would silently permit it.

**Fix:** Add `<F as Kind>::Of<'a, B>: Send` to the `unsafe impl Send` bound and `<F as Kind>::Of<'a, B>: Sync` to the `unsafe impl Sync` bound, matching the `ArcCoyonedaBase` pattern.

### BUG: `ArcCoyonedaNewLayer` compile-time assertion is vacuous

**Severity: Low.**

File: `fp-library/src/types/arc_coyoneda.rs`, lines 960-965.

The assertion:

```rust
fn _check_new_layer<'a, F: Kind_cdc7cd43dac7585f + 'a, B: 'a, A: 'a>() {
    _assert_send::<ArcCoyonedaNewLayer<'a, F, B, A>>();
    _assert_sync::<ArcCoyonedaNewLayer<'a, F, B, A>>();
}
```

This passes trivially because the `unsafe impl` already makes the type unconditionally `Send + Sync`. The assertion does not actually verify that the fields are `Send + Sync`; it only confirms the `unsafe impl` exists. It provides no regression protection. The comment "both fields satisfy Send + Sync when used through ArcCoyonedaLowerRef" is misleading.

**Fix:** If the `unsafe impl` bounds are corrected per the bug above, this assertion would need matching where clauses to remain useful:

```rust
fn _check_new_layer<'a, F: Kind_cdc7cd43dac7585f + 'a, B: 'a, A: 'a>()
where
    <F as Kind_cdc7cd43dac7585f>::Of<'a, B>: Send + Sync,
{
    _assert_send::<ArcCoyonedaNewLayer<'a, F, B, A>>();
    _assert_sync::<ArcCoyonedaNewLayer<'a, F, B, A>>();
}
```

---

## Gaps and Missing Items

### Missing compile-fail tests (Plan 4.4)

The plan specified three compile-fail tests:

1. `ArcCoyoneda` with a `!Send` payload fails to compile -- **NOT IMPLEMENTED**.
2. `RcCoyoneda` is `!Send` -- **IMPLEMENTED** (`rc_coyoneda_not_send.rs`).
3. `Coyoneda` is `!Clone` -- **NOT IMPLEMENTED**.

### Missing property tests for `Coyoneda` and `CoyonedaExplicit` (Plan 4.1, 4.2)

The plan explicitly listed `CoyonedaBrand<VecBrand>` and `CoyonedaExplicitBrand<VecBrand, _>` as targets for property-based Functor and Foldable law tests. Neither file contains `quickcheck` tests.

### Missing Foldable property test for `ArcCoyoneda` (Plan 4.2)

`RcCoyoneda` has `foldable_consistency_vec` but `ArcCoyoneda` does not.

### Missing `ArcCoyoneda_3x_lower` benchmark (Plan 5.2)

The plan called for "multiple `lower_ref` calls on the same value" for `ArcCoyoneda`. Only `RcCoyoneda_3x_lower` exists; the `ArcCoyoneda` equivalent is absent.

### Missing "clone + map + lower_ref" benchmarks (Plan 5.1, 5.2)

The plan called for a "clone + map + lower_ref pattern" benchmark for both Rc and Arc variants. Neither exists.

### Missing `Debug` for `Coyoneda` and `CoyonedaExplicit` (Plan 6.3)

The plan specified "All four Coyoneda files." Only `RcCoyoneda` and `ArcCoyoneda` have `Debug` impls. `Coyoneda` and `CoyonedaExplicit` were not in the modified file list and appear to lack `Debug` implementations.

---

## Convention Compliance

### Formatting

The code uses hard tabs, vertical layout for imports and parameters, and single imports per line, consistent with `rustfmt.toml`. No issues observed.

### No emoji or unicode

No emoji or unicode symbols found in the modified files. Correct.

### Documentation attributes

All public and trait methods in the modified files have `#[document_signature]`, `#[document_type_parameters(...)]`, `#[document_parameters(...)]`, `#[document_returns(...)]`, and `#[document_examples]` with code examples containing assertions. No missing documentation attributes observed in the reviewed files.

### Commit message style

Not audited (outside scope of code review).

### Writing style

No em dashes or en dashes found. Bullet points in module docs use proper punctuation. Correct.

---

## Summary

| Phase                    | Steps | Fully Implemented | Partially    | Missing                       |
| ------------------------ | ----- | ----------------- | ------------ | ----------------------------- |
| 1. Stack Safety          | 5     | 5                 | 0            | 0                             |
| 2. API Parity            | 6     | 6                 | 0            | 0                             |
| 3. Performance (removed) | 0     | N/A               | N/A          | N/A                           |
| 4. Testing               | 5     | 2 (4.3, 4.5)      | 2 (4.1, 4.4) | 1 (4.2 for Arc/Coyo/Explicit) |
| 5. Benchmarks            | 2     | 0                 | 2 (5.1, 5.2) | 0                             |
| 6. Documentation/Polish  | 11    | 10                | 1 (6.3)      | 0                             |

**Critical issues:** 1 (ArcCoyonedaNewLayer unsafe impl bounds too broad).

**Non-critical gaps:** Missing compile-fail tests (Coyoneda !Clone, ArcCoyoneda !Send payload), missing property tests for Coyoneda/CoyonedaExplicit/ArcCoyoneda Foldable, missing ArcCoyoneda repeated-lower and clone-map-lower benchmarks, missing Debug impls for Coyoneda/CoyonedaExplicit.
