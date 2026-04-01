# Coyoneda Plan Verification

This document audits the implementation against every numbered step in `plan.md` and every issue in `summary.md`.

---

## Phase 1: Stack Safety

### 1.1 Add `stacker` support to `RcCoyonedaMapLayer::lower_ref`

**Status: Implemented.**

`rc_coyoneda.rs` lines 194-207 contain `#[cfg(feature = "stacker")]` and `#[cfg(not(feature = "stacker"))]` branches wrapping the body in `stacker::maybe_grow(32 * 1024, 1024 * 1024, || { ... })`. Matches the pattern from `coyoneda.rs`.

### 1.2 Add `stacker` support to `ArcCoyonedaMapLayer::lower_ref`

**Status: Implemented.**

`arc_coyoneda.rs` lines 252-266 contain the same pattern.

### 1.3 Add `collapse` method to `RcCoyoneda`

**Status: Implemented.**

`rc_coyoneda.rs` lines 406-411. Signature matches the plan: takes `&self`, requires `F: Functor` and `F::Of<'a, A>: Clone`, returns `RcCoyoneda<'a, F, A>`. Implementation is `RcCoyoneda::lift(self.lower_ref())`.

### 1.4 Add `collapse` method to `ArcCoyoneda`

**Status: Implemented.**

`arc_coyoneda.rs` lines 484-489. Takes `&self`, requires `F: Functor` and `F::Of<'a, A>: Clone + Send + Sync`. Correct.

### 1.5 Document stack overflow risk

**Status: Implemented.**

Both `rc_coyoneda.rs` (lines 18-31) and `arc_coyoneda.rs` (lines 16-29) have "Stack safety" sections in the module docs, listing the three mitigations (stacker, collapse, CoyonedaExplicit).

---

## Phase 2: API Parity

### 2.1 Add `new(f, fb)` constructor to `RcCoyoneda`

**Status: Implemented via option (b), as recommended.**

`rc_coyoneda.rs` lines 506-516. Uses a dedicated `RcCoyonedaNewLayer` (lines 216-266) that stores `fb` and `func: Rc<dyn Fn(B) -> A>`, implementing `RcCoyonedaLowerRef` directly. Requires `F::Of<'a, B>: Clone`. The `RcCoyonedaNewLayer::lower_ref` also has stacker support (lines 253-264).

### 2.2 Add `new(f, fb)` constructor to `ArcCoyoneda`

**Status: Implemented via option (b).**

`arc_coyoneda.rs` lines 586-596. Uses `ArcCoyonedaNewLayer` (lines 274-348) with `Send + Sync` bounds on both `f` and `fb`. Stacker support present in `ArcCoyonedaNewLayer::lower_ref` (lines 335-347).

### 2.3 Add `hoist` to `RcCoyoneda`

**Status: Implemented.**

`rc_coyoneda.rs` lines 554-562. Signature: `pub fn hoist<G>(self, nat: impl NaturalTransformation<F, G>) -> RcCoyoneda<'a, G, A>`. Takes `self` (consuming), requires `F: Functor` and `G::Of<'a, A>: Clone`. Matches the plan.

### 2.4 Add `hoist` to `ArcCoyoneda`

**Status: Implemented.**

`arc_coyoneda.rs` lines 634-642. Same pattern with `Clone + Send + Sync` bound on `G::Of<'a, A>`.

### 2.5 Add inherent methods to `RcCoyoneda`

**Status: Implemented.**

All four inherent methods are present:

- `pure` (line 583): `F: Pointed`, `F::Of<'a, A>: Clone`. Matches plan.
- `bind` (line 616): `F: Functor + Semimonad`, `F::Of<'a, B>: Clone`. Matches plan.
- `apply` (line 662): `F: Functor + Semiapplicative`, `F::Of<'a, C>: Clone`. Matches plan.
- `lift2` (line 696): `F: Functor + Lift`, `A: Clone`, `F::Of<'a, C>: Clone`. Matches plan.

### 2.6 Add inherent methods to `ArcCoyoneda`

**Status: Implemented.**

All four inherent methods are present with `Clone + Send + Sync` bounds:

- `pure` (line 663): `F: Pointed`, `F::Of<'a, A>: Clone + Send + Sync`.
- `bind` (line 696): `F: Functor + Semimonad`, `F::Of<'a, B>: Clone + Send + Sync`.
- `apply` (line 740): `F: Functor + Semiapplicative`, `F::Of<'a, C>: Clone + Send + Sync`.
- `lift2` (line 774): `F: Functor + Lift`, `A: Clone`, `F::Of<'a, C>: Clone + Send + Sync`.

---

## Phase 3: Performance Optimization

**Status: Correctly removed.** The plan marks this as infeasible and removed. No implementation attempted.

---

## Phase 4: Testing

### 4.1 Add property-based tests for Functor laws

**Status: NOT implemented.**

No QuickCheck or property-based test infrastructure exists for any Coyoneda variant. A search across `fp-library/tests/` found no `quickcheck` or `proptest` imports. The existing property test files (`property_tests/`) were deleted in the branch.

### 4.2 Add property-based tests for Foldable laws

**Status: NOT implemented.** Same finding as 4.1.

### 4.3 Add stack overflow tests

**Status: Partially implemented.**

`stack_safety.rs` (lines 162-239) contains:

- `test_rc_coyoneda_collapse_resets_depth` (depth 20+20): Verifies collapse correctness.
- `test_arc_coyoneda_collapse_resets_depth` (depth 20+20): Same for Arc.
- `test_rc_coyoneda_deep_chain_with_stacker` (depth 1000, `#[cfg(feature = "stacker")]`).
- `test_arc_coyoneda_deep_chain_with_stacker` (depth 1000, `#[cfg(feature = "stacker")]`).

Missing: tests for deep chains WITHOUT stacker that demonstrate the stack overflow (or document the depth limit). The plan says "Test deep chains (thousands of maps) with and without the `stacker` feature." The non-stacker tests only go to depth 20, which is too shallow to meaningfully test stack safety limits.

### 4.4 Add compile-fail tests for Send/Sync soundness

**Status: Partially implemented.**

Present:

- `ui/rc_coyoneda_not_send.rs` and `.stderr`: Verifies `RcCoyoneda` is `!Send`. Correct.

Missing:

- No compile-fail test for `ArcCoyoneda` with a `!Send` payload failing to compile when sent across threads.
- No compile-fail test for `Coyoneda` being `!Clone`.

### 4.5 Add concurrent access tests for `ArcCoyoneda`

**Status: Implemented.**

`thread_safety.rs` lines 83-120 contain:

- `test_arc_coyoneda_concurrent_lower_ref`: 4 threads concurrently call `lower_ref` on cloned values.
- `test_arc_coyoneda_shared_across_threads`: `Arc<ArcCoyoneda>` shared across 4 threads.

---

## Phase 5: Benchmarks

### 5.1 Add `RcCoyoneda` benchmarks

**Status: Partially implemented.**

`benches/benchmarks/coyoneda.rs` lines 72-119 contain:

- Map chain construction + single `lower_ref` at depths 1, 10, 100 (line 73).
- Multiple `lower_ref` calls (3x) on the same value (`RcCoyoneda_3x_lower`, line 103).

Missing: "Clone + map + lower_ref pattern" benchmark explicitly called for in the plan.

### 5.2 Add `ArcCoyoneda` benchmarks

**Status: Partially implemented.**

`benches/benchmarks/coyoneda.rs` lines 87-100 contain:

- Map chain construction + single `lower_ref` at depths 1, 10, 100.

Missing:

- No `ArcCoyoneda_3x_lower` benchmark (the plan says "Same cases as 5.1").
- No "Clone + map + lower_ref pattern" benchmark for Arc.

---

## Phase 6: Documentation and Polish

### 6.1 Enable documentation validation on `ArcCoyoneda`

**Status: Implemented.**

`arc_coyoneda.rs` line 56 now uses `#[fp_macros::document_module]` without `no_validation`. Docs build cleanly with no warnings.

### 6.2 Add `From` conversions for Rc/Arc variants

**Status: Implemented.**

- `From<RcCoyoneda<'a, F, A>> for Coyoneda<'a, F, A>`: `rc_coyoneda.rs` lines 868-896.
- `From<ArcCoyoneda<'a, F, A>> for Coyoneda<'a, F, A>`: `arc_coyoneda.rs` lines 906-933.

Both require `F: Functor` and use `lower_ref()` + `lift()`.

### 6.3 Add `Debug` implementations

**Status: Partially implemented.**

- `RcCoyoneda`: Implemented at `rc_coyoneda.rs` lines 829-859. Outputs `RcCoyoneda(<opaque>)`.
- `ArcCoyoneda`: Implemented at `arc_coyoneda.rs` lines 867-897. Outputs `ArcCoyoneda(<opaque>)`.

Missing:

- No `Debug` impl for `Coyoneda` (the plan says "All four Coyoneda files").
- No `Debug` impl for `CoyonedaExplicit` (should show stored functor value when it implements `Debug`, per the plan).

### 6.4 Add inherent `fold_map(&self)` to Rc/Arc variants

**Status: Implemented.**

- `RcCoyoneda::fold_map`: `rc_coyoneda.rs` lines 441-450.
- `ArcCoyoneda::fold_map`: `arc_coyoneda.rs` lines 519-528.

Both take `&self` and delegate via `lower_ref`.

### 6.5 Document fusion barriers in `CoyonedaExplicit`

**Status: Implemented.**

- `traverse` (line 382): "This is a fusion barrier: all accumulated maps are composed into the traversal function..."
- `bind` (line 495): "This is a fusion barrier: all accumulated maps are composed into the bind callback..."

### 6.6 Document `CoyonedaExplicitBrand` Functor re-boxing cost

**Status: Implemented.**

`coyoneda_explicit.rs` lines 739-742 document that "each call through this brand-level `map` allocates a `Box`" and that "Zero-cost fusion (no allocation per map) is only available via the inherent `CoyonedaExplicit::map` method."

### 6.7 Document `CoyonedaExplicit::boxed()` loop overhead

**Status: Implemented.**

`coyoneda_explicit.rs` lines 541-546 document that "The composed function chain has O(k) per-element overhead at `lower` time, matching `Coyoneda`'s cost profile."

### 6.8 Harden `unsafe impl Send/Sync` on `ArcCoyonedaBase`

**Status: Implemented.**

Compile-time assertion at `arc_coyoneda.rs` lines 946-951 verifies `ArcCoyonedaBase` is `Send + Sync` when `Of<'a, A>: Send + Sync`.

### 6.9 Add compile-time assertion for `ArcCoyonedaMapLayer` Send/Sync

**Status: Implemented.**

`arc_coyoneda.rs` lines 953-958. Assertion verifies the struct is unconditionally `Send + Sync`.

### 6.10 Document why `unsafe impl Send/Sync` is needed on `ArcCoyonedaMapLayer`

**Status: Implemented.**

`arc_coyoneda.rs` lines 183-201 contain a thorough SAFETY comment covering: both fields are `Arc<dyn ... + Send + Sync>`; the compiler cannot auto-derive due to `F: Kind`; adding `Send + Sync` to `Kind::Of` is infeasible; `SendKind` subtrait not expressible on stable Rust; soundness depends on field types; what would break soundness.

### 6.11 Document brand-level trait impl limitations in source code

**Status: Implemented.**

- `rc_coyoneda.rs` lines 717-732: Explains why `Pointed`, `Lift`, `Semiapplicative`, and `Semimonad` cannot be implemented for `RcCoyonedaBrand`, covering the `Clone` bound issue, why `CoyonedaBrand` avoids it, and that inherent methods are provided instead.
- `arc_coyoneda.rs` lines 795-810: Same explanation with additional `Send + Sync` limitation.

---

## Flaws and Issues Found

### 1. UNSOUND: `ArcCoyonedaNewLayer` unsafe `Send/Sync` impls lack bounds on `fb` (Critical)

`arc_coyoneda.rs` lines 290-302. The `unsafe impl Send` and `unsafe impl Sync` for `ArcCoyonedaNewLayer` only require `F: Kind + 'a`. They do NOT require `F::Of<'a, B>: Send` or `F::Of<'a, B>: Sync`, even though the struct stores `fb: F::Of<'a, B>` directly (not behind an `Arc`).

Compare with `ArcCoyonedaBase` (lines 125-137), which correctly requires `Of<'a, A>: Send` for `Send` and `Of<'a, A>: Sync` for `Sync`.

While the struct is private and only constructed via `ArcCoyoneda::new` (which enforces `Clone + Send + Sync` on `fb`), the `unsafe impl` is overly broad. If any future code in the module constructs an `ArcCoyonedaNewLayer` without those bounds, it would be unsound. The `unsafe impl` should carry the same defensive bounds as `ArcCoyonedaBase`:

```rust
unsafe impl<'a, F, B: 'a, A: 'a> Send for ArcCoyonedaNewLayer<'a, F, B, A>
where
    F: Kind_cdc7cd43dac7585f + 'a,
    <F as Kind_cdc7cd43dac7585f>::Of<'a, B>: Send,
{}
unsafe impl<'a, F, B: 'a, A: 'a> Sync for ArcCoyonedaNewLayer<'a, F, B, A>
where
    F: Kind_cdc7cd43dac7585f + 'a,
    <F as Kind_cdc7cd43dac7585f>::Of<'a, B>: Sync,
{}
```

The compile-time assertion at lines 960-965 is also misleading. It asserts `ArcCoyonedaNewLayer` is "unconditionally Send + Sync" with the comment "(both fields satisfy Send + Sync when used through ArcCoyonedaLowerRef)," but `fb` is NOT unconditionally `Send + Sync`. The assertion only proves the `unsafe impl` exists (which it does, because it was written unconditionally), not that it is correct. The assertion should have `where Of<'a, B>: Send + Sync` to actually verify the invariant, matching `_check_base`.

### 2. `ArcCoyoneda::apply` doc example does not demonstrate `apply` (Minor)

`arc_coyoneda.rs` lines 726-739. The doc example for `apply` actually calls `lift2` instead, with a comment explaining that the `CloneableFn` brand's `Of` type needs `Send + Sync`. A doc example should demonstrate the documented function, or the example should note that a working `apply` example requires `ArcFnBrand` and show that.

### 3. Missing property-based tests (Steps 4.1, 4.2)

No QuickCheck or property-based tests were added for any Coyoneda variant. The plan explicitly calls for testing Functor identity/composition laws and Foldable laws with randomized inputs. The existing property test infrastructure was removed from the branch.

### 4. Missing compile-fail tests (Step 4.4, partial)

Only `RcCoyoneda` `!Send` is tested. Missing:

- `ArcCoyoneda` with `!Send` payload.
- `Coyoneda` is `!Clone`.

### 5. Missing `Debug` impls for `Coyoneda` and `CoyonedaExplicit` (Step 6.3, partial)

The plan says "All four Coyoneda files" should get `Debug` impls. Only `RcCoyoneda` and `ArcCoyoneda` have them. `Coyoneda` and `CoyonedaExplicit` do not.

### 6. Incomplete benchmark coverage (Steps 5.1, 5.2)

- No "Clone + map + lower_ref" benchmark for either variant.
- No `ArcCoyoneda_3x_lower` (repeated lower_ref) benchmark for the Arc variant. The plan says "Same cases as 5.1" for ArcCoyoneda.

### 7. Shallow non-stacker stack safety tests (Step 4.3)

The non-stacker collapse tests only go to depth 20+20=40, which is too shallow to demonstrate that collapse actually prevents stack overflow. A depth of at least several hundred would be more meaningful for validating that collapse resets recursion depth. The stacker tests go to depth 1000, which is reasonable but still modest compared to the "thousands of maps" mentioned in the plan.

---

## Convention Compliance

### Formatting

The code uses hard tabs for indentation and follows the project's vertical layout for function parameters and imports, consistent with `rustfmt.toml`.

### No Emoji or Unicode

No emoji or unicode symbols found in any of the modified files.

### Documentation Attributes

All public functions and trait implementations have the required documentation attributes (`#[document_signature]`, `#[document_type_parameters(...)]`, `#[document_parameters(...)]`, `#[document_returns(...)]`, `#[document_examples]`) followed by code examples with assertions. The documentation follows the template specified in `CLAUDE.md`.

### Writing Style

Bullet points and documentation text use proper punctuation. No em dashes or en dashes found. Hyphens used appropriately for compound terms.

---

## Summary

| Step | Status  | Notes                                                                             |
| ---- | ------- | --------------------------------------------------------------------------------- |
| 1.1  | Done    | Stacker in `RcCoyonedaMapLayer::lower_ref`                                        |
| 1.2  | Done    | Stacker in `ArcCoyonedaMapLayer::lower_ref`                                       |
| 1.3  | Done    | `RcCoyoneda::collapse`                                                            |
| 1.4  | Done    | `ArcCoyoneda::collapse`                                                           |
| 1.5  | Done    | Stack safety docs                                                                 |
| 2.1  | Done    | `RcCoyoneda::new` with dedicated layer (option b)                                 |
| 2.2  | Done    | `ArcCoyoneda::new` with dedicated layer (option b)                                |
| 2.3  | Done    | `RcCoyoneda::hoist`                                                               |
| 2.4  | Done    | `ArcCoyoneda::hoist`                                                              |
| 2.5  | Done    | Inherent methods on `RcCoyoneda`                                                  |
| 2.6  | Done    | Inherent methods on `ArcCoyoneda`                                                 |
| 4.1  | Missing | No property-based Functor law tests                                               |
| 4.2  | Missing | No property-based Foldable law tests                                              |
| 4.3  | Partial | Collapse tests present but shallow; stacker tests present                         |
| 4.4  | Partial | Only `RcCoyoneda !Send` tested; missing Arc `!Send` payload and `Coyoneda !Clone` |
| 4.5  | Done    | Concurrent `ArcCoyoneda` tests                                                    |
| 5.1  | Partial | Missing clone+map+lower pattern benchmark                                         |
| 5.2  | Partial | Missing 3x_lower and clone+map+lower benchmarks                                   |
| 6.1  | Done    | `no_validation` removed                                                           |
| 6.2  | Done    | `From` conversions                                                                |
| 6.3  | Partial | Only Rc/Arc variants; missing `Coyoneda` and `CoyonedaExplicit`                   |
| 6.4  | Done    | Inherent `fold_map`                                                               |
| 6.5  | Done    | Fusion barrier docs                                                               |
| 6.6  | Done    | Re-boxing cost docs                                                               |
| 6.7  | Done    | `boxed()` loop overhead docs                                                      |
| 6.8  | Done    | Compile-time assertion for `ArcCoyonedaBase`                                      |
| 6.9  | Done    | Compile-time assertion for `ArcCoyonedaMapLayer`                                  |
| 6.10 | Done    | Safety comment on `ArcCoyonedaMapLayer`                                           |
| 6.11 | Done    | Brand-level limitation docs                                                       |

**Critical finding:** `ArcCoyonedaNewLayer`'s `unsafe impl Send/Sync` is overly broad and should carry `Of<'a, B>: Send`/`Of<'a, B>: Sync` bounds for defense in depth, matching `ArcCoyonedaBase`'s pattern.

**Total:** 22 of 29 steps fully implemented, 5 partially implemented, 2 missing entirely.
