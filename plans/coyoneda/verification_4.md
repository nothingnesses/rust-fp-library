# Coyoneda Plan Verification Report

This report audits whether each step of `plan.md` was correctly implemented in the modified files, checks for bugs or inconsistencies, and verifies adherence to project conventions.

---

## Phase 1: Stack Safety

### 1.1 Add `stacker` support to `RcCoyonedaMapLayer::lower_ref` -- IMPLEMENTED

`rc_coyoneda.rs` lines 194-207: The `lower_ref` method wraps its body in `stacker::maybe_grow(32 * 1024, 1024 * 1024, || { ... })` behind `#[cfg(feature = "stacker")]`, with a fallback `#[cfg(not(feature = "stacker"))]` block. This matches the pattern described in the plan.

### 1.2 Add `stacker` support to `ArcCoyonedaMapLayer::lower_ref` -- IMPLEMENTED

`arc_coyoneda.rs` lines 252-265: Same pattern as 1.1, correctly applied.

### 1.3 Add `collapse` method to `RcCoyoneda` -- IMPLEMENTED

`rc_coyoneda.rs` lines 406-411: The method signature is `pub fn collapse(&self) -> RcCoyoneda<'a, F, A>` with bounds `F: Functor` and `F::Of<'a, A>: Clone`. Takes `&self` as specified. Implementation delegates to `RcCoyoneda::lift(self.lower_ref())`. Matches the plan exactly.

### 1.4 Add `collapse` method to `ArcCoyoneda` -- IMPLEMENTED

`arc_coyoneda.rs` lines 484-489: Signature is `pub fn collapse(&self) -> ArcCoyoneda<'a, F, A>` with bounds `F: Functor` and `F::Of<'a, A>: Clone + Send + Sync`. Matches the plan.

### 1.5 Document stack overflow risk -- IMPLEMENTED

Both `rc_coyoneda.rs` (lines 18-31) and `arc_coyoneda.rs` (lines 16-29) contain a "Stack safety" section in their module docs. The documentation describes three mitigations: `stacker` feature, `collapse`, and `CoyonedaExplicit` with `.boxed()`.

---

## Phase 2: API Parity

### 2.1 Add `new(f, fb)` constructor to `RcCoyoneda` -- IMPLEMENTED

`rc_coyoneda.rs` lines 506-516: Uses the recommended option (b) with a dedicated `RcCoyonedaNewLayer` struct (lines 216-221). The new layer stores `fb` and `func: Rc<dyn Fn(B) -> A + 'a>`, implementing `RcCoyonedaLowerRef` directly. This is a single `Rc` allocation for the layer (plus one for the function), matching the plan's efficiency recommendation.

The `RcCoyonedaNewLayer::lower_ref` (lines 250-265) also has `stacker` support, which is a good addition not explicitly required by the plan.

### 2.2 Add `new(f, fb)` constructor to `ArcCoyoneda` -- IMPLEMENTED

`arc_coyoneda.rs` lines 586-596: Uses a dedicated `ArcCoyonedaNewLayer` (lines 274-279) with `Send + Sync` bounds on the function and `Clone + Send + Sync` on `F::Of<'a, B>`. The `ArcCoyonedaNewLayer` has proper `unsafe impl Send/Sync` (lines 290-303) with safety comments, and `stacker` support in `lower_ref` (lines 335-347).

**Issue:** The `ArcCoyonedaNewLayer`'s `unsafe impl Send/Sync` has a safety comment (lines 281-283) that states "`fb` is bounded `Send + Sync` via the ArcCoyonedaLowerRef impl's where clause". This is true for soundness, but the `unsafe impl` itself has no `Send + Sync` bound on `fb`; it only requires `F: Kind`. The safety argument holds because the struct can only be constructed via `ArcCoyoneda::new` which constrains `F::Of<'a, B>: Clone + Send + Sync`, and the struct is private, so the `fb` field is guaranteed to satisfy `Send + Sync` at construction time. However, the compile-time assertions at the bottom of the file (lines 960-965) call `_assert_send::<ArcCoyonedaNewLayer<'a, F, B, A>>()` without any `Send + Sync` bound on `F::Of<'a, B>`, which means these assertions verify that the `unsafe impl` is unconditional, not that it is correct only when `fb: Send + Sync`. This is consistent with how `ArcCoyonedaMapLayer` is handled (also unconditional), but worth noting that the safety relies on the struct being private and only constructible through properly-bounded paths.

### 2.3 Add `hoist` to `RcCoyoneda` -- IMPLEMENTED

`rc_coyoneda.rs` lines 554-562: Signature matches the plan: `pub fn hoist<G>(self, nat: impl NaturalTransformation<F, G>) -> RcCoyoneda<'a, G, A>` with bounds `F: Functor` and `G::Of<'a, A>: Clone`. Implementation: `RcCoyoneda::lift(nat.transform(self.lower_ref()))`.

### 2.4 Add `hoist` to `ArcCoyoneda` -- IMPLEMENTED

`arc_coyoneda.rs` lines 634-642: Same pattern with additional `Send + Sync` bounds on `G::Of<'a, A>`.

### 2.5 Add inherent methods to `RcCoyoneda` -- IMPLEMENTED

All four inherent methods are present:

- `pure` (lines 583-588): Bounds `F: Pointed`, `F::Of<'a, A>: Clone`. Matches plan.
- `bind` (lines 616-624): Bounds `F: Functor + Semimonad`, `F::Of<'a, B>: Clone`. Matches plan.
- `apply` (lines 662-670): Bounds `F: Functor + Semiapplicative`, `F::Of<'a, C>: Clone`. Matches plan (the result type bound is on `C`, not `B`, which is correct since the result is `RcCoyoneda<F, C>`).
- `lift2` (lines 696-706): Bounds `F: Functor + Lift`, `F::Of<'a, C>: Clone`. Also requires `A: Clone` and `B: Clone`. Matches plan.

### 2.6 Add inherent methods to `ArcCoyoneda` -- IMPLEMENTED

All four inherent methods are present:

- `pure` (lines 663-668): Bounds `F: Pointed`, `F::Of<'a, A>: Clone + Send + Sync`. Matches plan.
- `bind` (lines 696-704): Bounds `F: Functor + Semimonad`, `F::Of<'a, B>: Clone + Send + Sync`. Matches plan.
- `apply` (lines 740-748): Bounds `F: Functor + Semiapplicative`, `F::Of<'a, C>: Clone + Send + Sync`. Matches plan.
- `lift2` (lines 774-784): Bounds `F: Functor + Lift`, `F::Of<'a, C>: Clone + Send + Sync`. Matches plan.

**Issue (minor):** The `apply` method's doc example (lines 726-739) does not actually demonstrate `apply`; it instead demonstrates `lift2`. The comment explains why ("The apply method requires a CloneableFn brand whose Of type is Send + Sync"), which is a valid technical limitation. However, as a doc example for `apply`, it does not show how to call the function it documents. A working `apply` example using `ArcFnBrand` would be preferable.

---

## Phase 3: Performance Optimization -- CORRECTLY REMOVED

The plan states Phase 3 is removed as infeasible. No inline-function optimization was attempted. Correct.

---

## Phase 4: Testing

### 4.1 Add property-based tests for Functor laws -- IMPLEMENTED

`rc_coyoneda.rs` lines 992-1053: QuickCheck tests for:

- Identity law with `VecBrand` and `OptionBrand`.
- Composition law with `VecBrand` and `OptionBrand`.

`arc_coyoneda.rs` lines 1023-1065: QuickCheck tests for:

- Identity law with `VecBrand` and `OptionBrand`.
- Composition law with `VecBrand` and `OptionBrand`.

The plan specifies testing with `CoyonedaExplicitBrand<VecBrand, _>` as well, but no property-based tests for `CoyonedaExplicit` were observed in the diff. Since `coyoneda_explicit.rs` is listed as a modified file and is entirely new, it likely includes tests. Let me note this is partially implemented (Rc and Arc covered, but `CoyonedaExplicit` property tests are not visible in the listed changes; the file has unit tests but the property-based section was not examined).

### 4.2 Add property-based tests for Foldable laws -- PARTIALLY IMPLEMENTED

`rc_coyoneda.rs` lines 1056-1067: A `foldable_consistency_vec` QuickCheck test verifies that `fold_map` through `RcCoyonedaBrand` matches direct folding. Good.

**Missing:** No equivalent `foldable_consistency` property test exists for `ArcCoyonedaBrand`. The `arc_coyoneda.rs` property tests (lines 1023-1076) contain functor identity, functor composition, and collapse tests, but no foldable consistency test.

### 4.3 Add stack overflow tests -- IMPLEMENTED

`fp-library/tests/stack_safety.rs` lines 162-239:

- `test_rc_coyoneda_collapse_resets_depth` (lines 170-185): Builds 20 maps, collapses, adds 20 more.
- `test_arc_coyoneda_collapse_resets_depth` (lines 188-204): Same for Arc.
- `test_rc_coyoneda_deep_chain_with_stacker` (lines 211-223): 1,000 maps with `stacker` feature, `OptionBrand`.
- `test_arc_coyoneda_deep_chain_with_stacker` (lines 227-239): Same for Arc.

**Note:** The plan mentions testing with "thousands of maps" and testing with and without `stacker`. The tests use 20 maps for non-stacker tests (to avoid overflow in debug builds) and 1,000 with stacker. The depth of 1,000 is relatively modest; depending on stack frame size this may not actually overflow without stacker. The collapse property tests in both modules also verify correctness with QuickCheck.

### 4.4 Add compile-fail tests for Send/Sync soundness -- PARTIALLY IMPLEMENTED

`fp-library/tests/ui/rc_coyoneda_not_send.rs`: Proves `RcCoyoneda` is `!Send`. The `.stderr` file contains the expected error output. This is correct.

**Missing items from the plan:**

1. No compile-fail test proving `ArcCoyoneda` with a `!Send` payload fails when sent across threads.
2. No compile-fail test proving `Coyoneda` is `!Clone`.

### 4.5 Add concurrent access tests for `ArcCoyoneda` -- IMPLEMENTED

`fp-library/tests/thread_safety.rs` lines 83-120:

- `test_arc_coyoneda_concurrent_lower_ref`: 4 threads clone and call `lower_ref` concurrently.
- `test_arc_coyoneda_shared_across_threads`: Wraps in `Arc`, 4 threads share and call `lower_ref`.

Both tests verify correct values from concurrent access. Well done.

---

## Phase 5: Benchmarks

### 5.1 Add `RcCoyoneda` benchmarks -- IMPLEMENTED

`fp-library/benches/benchmarks/coyoneda.rs` lines 73-85: Benchmark for `RcCoyoneda` lift + chain + `lower_ref`.

Lines 103-119: `RcCoyoneda_3x_lower` benchmark measuring repeated `lower_ref` re-evaluation cost.

**Missing:** The plan calls for a "Clone + map + lower_ref pattern" benchmark. No such benchmark exists. The `3x_lower` benchmark measures re-evaluation but not the clone-then-extend pattern.

### 5.2 Add `ArcCoyoneda` benchmarks -- PARTIALLY IMPLEMENTED

`fp-library/benches/benchmarks/coyoneda.rs` lines 88-100: Benchmark for `ArcCoyoneda` lift + chain + `lower_ref`.

**Missing:**

1. No repeated `lower_ref` benchmark for `ArcCoyoneda` (equivalent to the `RcCoyoneda_3x_lower` benchmark).
2. No "Clone + map + lower_ref" benchmark for `ArcCoyoneda`.

---

## Phase 6: Documentation and Polish

### 6.1 Enable documentation validation on `ArcCoyoneda` -- IMPLEMENTED

`arc_coyoneda.rs` line 56: `#[fp_macros::document_module]` without `no_validation`. Confirmed.

### 6.2 Add `From` conversions for Rc/Arc variants -- IMPLEMENTED

- `From<RcCoyoneda<'a, F, A>> for Coyoneda<'a, F, A>`: `rc_coyoneda.rs` lines 868-896. Requires `F: Functor`. Implementation: `Coyoneda::lift(rc.lower_ref())`.
- `From<ArcCoyoneda<'a, F, A>> for Coyoneda<'a, F, A>`: `arc_coyoneda.rs` lines 906-934. Same pattern.

Both are correct.

### 6.3 Add `Debug` implementations -- PARTIALLY IMPLEMENTED

- `RcCoyoneda`: Implemented at `rc_coyoneda.rs` lines 829-859. Output: `RcCoyoneda(<opaque>)`.
- `ArcCoyoneda`: Implemented at `arc_coyoneda.rs` lines 867-897. Output: `ArcCoyoneda(<opaque>)`.

**Missing:**

1. No `Debug` for `Coyoneda`. The plan says "All four Coyoneda files." `coyoneda.rs` is not listed as a modified file and has no `Debug` impl.
2. No `Debug` for `CoyonedaExplicit`. The plan says to show the stored functor value when it implements `Debug` for `CoyonedaExplicit`. No such impl exists in `coyoneda_explicit.rs`.

### 6.4 Add inherent `fold_map(&self)` to Rc/Arc variants -- IMPLEMENTED

- `RcCoyoneda::fold_map`: `rc_coyoneda.rs` lines 441-450. Takes `&self`, borrows via `lower_ref`.
- `ArcCoyoneda::fold_map`: `arc_coyoneda.rs` lines 519-528. Same pattern.

Both are correct.

### 6.5 Document fusion barriers in `CoyonedaExplicit` -- IMPLEMENTED

- `traverse` (lines 378-384): "This is a fusion barrier: all accumulated maps are composed into the traversal function and applied during the traversal."
- `bind` (lines 492-497): "This is a fusion barrier: all accumulated maps are composed into the bind callback and applied during the bind."
- `apply` (already had it, confirmed at lines 441-442): "This is a fusion barrier."

All three methods document their fusion barrier status. Matches the plan.

### 6.6 Document `CoyonedaExplicitBrand` Functor re-boxing cost -- IMPLEMENTED

`coyoneda_explicit.rs` lines 738-742: The brand-level `Functor::map` documentation includes: "Note: each call through this brand-level `map` allocates a `Box` for the composed function (via `.boxed()`). Zero-cost fusion (no allocation per map) is only available via the inherent `CoyonedaExplicit::map` method, which uses compile-time function composition without boxing."

### 6.7 Document `CoyonedaExplicit::boxed()` loop overhead -- IMPLEMENTED

`coyoneda_explicit.rs` lines 541-546: The `boxed()` method documentation includes: "When used in a loop (e.g., `coyo = coyo.map(f).boxed()` per iteration), each iteration creates a closure that captures the previous boxed function. The composed function chain has O(k) per-element overhead at `lower` time, matching `Coyoneda`'s cost profile. The single-`F::map`-call advantage of `CoyonedaExplicit` is fully realized only with static (compile-time) composition where the compiler can inline the function chain."

### 6.8 Harden `unsafe impl Send/Sync` on `ArcCoyonedaBase` -- IMPLEMENTED

`arc_coyoneda.rs` lines 941-951: Compile-time assertion for `ArcCoyonedaBase`:

```rust
fn _check_base<'a, F: Kind_cdc7cd43dac7585f + 'a, A: 'a>()
where
    <F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Send + Sync, {
    _assert_send::<ArcCoyonedaBase<'a, F, A>>();
    _assert_sync::<ArcCoyonedaBase<'a, F, A>>();
}
```

This correctly constrains on `Of<'a, A>: Send + Sync` before asserting, matching the plan's approach.

### 6.9 Add compile-time assertion for `ArcCoyonedaMapLayer` Send/Sync -- IMPLEMENTED

`arc_coyoneda.rs` lines 955-958: Compile-time assertion for `ArcCoyonedaMapLayer`:

```rust
fn _check_map_layer<'a, F: Kind_cdc7cd43dac7585f + 'a, B: 'a, A: 'a>() {
    _assert_send::<ArcCoyonedaMapLayer<'a, F, B, A>>();
    _assert_sync::<ArcCoyonedaMapLayer<'a, F, B, A>>();
}
```

No bounds on the function, asserting unconditional `Send + Sync`. This matches the plan's approach (the plan's example asserts on the individual `Arc<dyn ...>` fields, but asserting on the struct itself is equivalent and arguably better since it catches any future field additions).

Additionally, lines 960-965 add an assertion for `ArcCoyonedaNewLayer` (not explicitly in the plan since the plan was written before the new layer was added). Good.

### 6.10 Document why `unsafe impl Send/Sync` is needed on `ArcCoyonedaMapLayer` -- IMPLEMENTED

`arc_coyoneda.rs` lines 183-201: Thorough safety comment covering:

- Both fields are `Arc<dyn ... + Send + Sync>`.
- Compiler cannot auto-derive because of `F: Kind` in the where clause.
- Adding `Send + Sync` to `Kind::Of` is not feasible.
- `SendKind` subtrait is not expressible on stable Rust.
- What depends on for soundness.
- What would break soundness.

All six points from the plan are addressed.

### 6.11 Document brand-level trait impl limitations -- IMPLEMENTED

- `rc_coyoneda.rs` lines 717-732: Comment block explaining why `Pointed`, `Lift`, `Semiapplicative`, `Semimonad` cannot be implemented for `RcCoyonedaBrand`. Covers the `Clone` bound blocker, why `CoyonedaBrand` avoids it, Rust's limitation on adding where clauses to trait impls, and that inherent methods are provided instead.

- `arc_coyoneda.rs` lines 795-810: Comment block explaining the same for `ArcCoyonedaBrand`, covering both the `Send + Sync` limitation on `Functor` and the `Clone` limitation on other traits.

Both match the plan's requirements.

---

## Summary of Issues Found

### Missing Items (plan steps not fully implemented)

1. **No `Debug` for `Coyoneda` or `CoyonedaExplicit`** (step 6.3). The plan says "All four Coyoneda files." Only `RcCoyoneda` and `ArcCoyoneda` have `Debug` impls. `Coyoneda` and `CoyonedaExplicit` do not. Since `coyoneda.rs` is not listed as a modified file, this was likely skipped.

2. **No `foldable_consistency` property test for `ArcCoyonedaBrand`** (step 4.2). `RcCoyonedaBrand` has one; `ArcCoyonedaBrand` does not.

3. **Missing compile-fail tests** (step 4.4):
   - No test proving `ArcCoyoneda` with a `!Send` payload fails to compile when sent across threads.
   - No test proving `Coyoneda` is `!Clone`.

4. **Missing `ArcCoyoneda` repeated `lower_ref` benchmark** (step 5.2). The `RcCoyoneda_3x_lower` benchmark exists but no equivalent for `ArcCoyoneda`.

5. **Missing "Clone + map + lower_ref" benchmark pattern** (steps 5.1/5.2). The plan calls for this pattern but it is not benchmarked for either variant.

6. **No `CoyonedaExplicit` property-based Functor tests** (step 4.1). The plan specifies `CoyonedaExplicitBrand<VecBrand, _>` as a target for property tests. While `coyoneda_explicit.rs` contains unit tests covering identity and composition laws, it does not appear to have QuickCheck property-based tests.

### Potential Bugs or Concerns

7. **`ArcCoyoneda::apply` doc example does not demonstrate `apply`** (arc_coyoneda.rs lines 726-739). The doc example for the `apply` method calls `lift2` instead. While the comment explains the technical reason, the `#[document_examples]` attribute presumably validates that the example compiles; it does not validate that the documented function is actually called. This is misleading for users reading API docs.

8. **`ArcCoyonedaNewLayer` unsafe Send/Sync has no field-level bounds** (arc_coyoneda.rs lines 290-303). The `unsafe impl Send` and `unsafe impl Sync` for `ArcCoyonedaNewLayer` have no bounds on `F::Of<'a, B>`, making them unconditional. The safety is ensured by the struct being private and only constructible through `ArcCoyoneda::new` which requires `Clone + Send + Sync`. The compile-time assertion (lines 960-965) verifies this unconditional property, so any regression in the unsafe impl would be caught, but only if the struct fields change. If someone added a new code path constructing `ArcCoyonedaNewLayer` with a `!Send` type for `fb`, the unsafe impl would silently allow it. This is a low-risk concern since the struct is private.

### Convention Compliance

9. **No emoji or unicode detected.** All documentation uses plain ASCII, arrows use `->`, etc. Compliant.

10. **Indentation uses tabs.** Verified throughout. Compliant.

11. **Documentation attributes present.** All public methods have `#[document_signature]`, `#[document_parameters(...)]`, `#[document_returns(...)]`, `#[document_examples]` where applicable. Compliant.

12. **`#[fp_macros::document_module]` used correctly.** Both `rc_coyoneda.rs` (line 50) and `arc_coyoneda.rs` (line 56) use it without `no_validation`. Compliant.

13. **Commit message style.** Not audited as part of this verification (the plan does not modify commit messages).

14. **No em dashes or en dashes.** Verified. Documentation uses commas, semicolons, and restructured sentences. Compliant.

---

## Step-by-Step Checklist

| Step | Description                                     | Status                                                                         |
| ---- | ----------------------------------------------- | ------------------------------------------------------------------------------ |
| 1.1  | `stacker` in `RcCoyonedaMapLayer::lower_ref`    | Done                                                                           |
| 1.2  | `stacker` in `ArcCoyonedaMapLayer::lower_ref`   | Done                                                                           |
| 1.3  | `RcCoyoneda::collapse`                          | Done                                                                           |
| 1.4  | `ArcCoyoneda::collapse`                         | Done                                                                           |
| 1.5  | Stack safety docs                               | Done                                                                           |
| 2.1  | `RcCoyoneda::new`                               | Done                                                                           |
| 2.2  | `ArcCoyoneda::new`                              | Done                                                                           |
| 2.3  | `RcCoyoneda::hoist`                             | Done                                                                           |
| 2.4  | `ArcCoyoneda::hoist`                            | Done                                                                           |
| 2.5  | `RcCoyoneda` inherent methods                   | Done                                                                           |
| 2.6  | `ArcCoyoneda` inherent methods                  | Done                                                                           |
| 4.1  | Property tests: Functor laws                    | Partial (Rc/Arc done; CoyonedaExplicit missing)                                |
| 4.2  | Property tests: Foldable laws                   | Partial (Rc done; Arc missing)                                                 |
| 4.3  | Stack overflow tests                            | Done                                                                           |
| 4.4  | Compile-fail tests                              | Partial (RcCoyoneda !Send done; Arc !Send payload and Coyoneda !Clone missing) |
| 4.5  | Concurrent access tests                         | Done                                                                           |
| 5.1  | `RcCoyoneda` benchmarks                         | Partial (single lower + 3x lower done; clone+map+lower missing)                |
| 5.2  | `ArcCoyoneda` benchmarks                        | Partial (single lower done; 3x lower and clone+map+lower missing)              |
| 6.1  | Remove `no_validation` from ArcCoyoneda         | Done                                                                           |
| 6.2  | `From` conversions                              | Done                                                                           |
| 6.3  | `Debug` implementations                         | Partial (Rc/Arc done; Coyoneda and CoyonedaExplicit missing)                   |
| 6.4  | Inherent `fold_map(&self)`                      | Done                                                                           |
| 6.5  | Document fusion barriers                        | Done                                                                           |
| 6.6  | Document brand Functor re-boxing cost           | Done                                                                           |
| 6.7  | Document `boxed()` loop overhead                | Done                                                                           |
| 6.8  | Harden ArcCoyonedaBase Send/Sync                | Done                                                                           |
| 6.9  | Compile-time assertions for ArcCoyonedaMapLayer | Done                                                                           |
| 6.10 | Document ArcCoyonedaMapLayer unsafe             | Done                                                                           |
| 6.11 | Document brand-level limitations                | Done                                                                           |

---

## Summary Issues from `summary.md`

| Issue | Summary Item                       | Addressed?                                           |
| ----- | ---------------------------------- | ---------------------------------------------------- |
| 1.1   | No stack safety in Rc/Arc          | Yes                                                  |
| 1.2   | Missing type class instances       | Yes (as inherent methods, with documented rationale) |
| 2.1   | Missing API methods                | Yes                                                  |
| 2.2   | No benchmarks                      | Partially (some benchmark patterns missing)          |
| 2.3   | `no_validation` on ArcCoyoneda     | Yes                                                  |
| 2.4   | No property-based tests            | Partially (some variants missing)                    |
| 2.5   | Unsafe Send/Sync verification      | Yes                                                  |
| 2.6   | Two allocations per map            | Documented as accepted (plan Phase 3 removed)        |
| 2.7   | No conversions                     | Yes                                                  |
| 3.1   | No Debug                           | Partially (Rc/Arc done, other two missing)           |
| 3.2   | Inherent fold_map                  | Yes                                                  |
| 3.3   | Missing consuming lower            | Deferred (per plan)                                  |
| 3.4   | CoyonedaExplicitBrand re-boxing    | Yes (documented)                                     |
| 3.5   | Document bind/traverse as barriers | Yes                                                  |
| 3.6   | Document boxed() loop overhead     | Yes                                                  |
| 3.7   | Improve unsafe comments            | Yes                                                  |

---

## Overall Assessment

The implementation is thorough and covers all high-priority and medium-priority items from the plan. The core functionality (stack safety, API parity, inherent methods, conversions, documentation) is complete and correct. The main gaps are in testing completeness (missing a few property tests and compile-fail tests), benchmark coverage (missing some patterns for ArcCoyoneda), and two missing `Debug` impls for `Coyoneda` and `CoyonedaExplicit`. No correctness bugs were identified in the implemented code. The `unsafe` code is well-documented and guarded by compile-time assertions.
