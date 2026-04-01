# Coyoneda Plan Verification

This document assesses whether each step of the plan (`plans/coyoneda/plan.md`) was correctly implemented in the modified files. It also evaluates adherence to project conventions and identifies flaws.

---

## Phase 1: Stack Safety

### 1.1 Add `stacker` support to `RcCoyonedaMapLayer::lower_ref`

**Status: Implemented.**

`rc_coyoneda.rs` lines 194-207: `RcCoyonedaMapLayer::lower_ref` wraps its body in `stacker::maybe_grow(32 * 1024, 1024 * 1024, || { ... })` behind `#[cfg(feature = "stacker")]`, with a fallback `#[cfg(not(feature = "stacker"))]` block. Matches the pattern described in the plan.

### 1.2 Add `stacker` support to `ArcCoyonedaMapLayer::lower_ref`

**Status: Implemented.**

`arc_coyoneda.rs` lines 252-266: Same pattern as 1.1.

### 1.3 Add `collapse` method to `RcCoyoneda`

**Status: Implemented.**

`rc_coyoneda.rs` lines 406-411: `pub fn collapse(&self) -> RcCoyoneda<'a, F, A>` with bounds `F: Functor, <F as Kind>::Of<'a, A>: Clone`. Takes `&self` as specified. Implementation is `RcCoyoneda::lift(self.lower_ref())`. Matches the plan exactly.

### 1.4 Add `collapse` method to `ArcCoyoneda`

**Status: Implemented.**

`arc_coyoneda.rs` lines 484-489: `pub fn collapse(&self) -> ArcCoyoneda<'a, F, A>` with bounds `F: Functor, <F as Kind>::Of<'a, A>: Clone + Send + Sync`. Matches the plan.

### 1.5 Document stack overflow risk

**Status: Implemented.**

Both `rc_coyoneda.rs` (lines 18-31) and `arc_coyoneda.rs` (lines 16-29) have "Stack safety" sections in their module docs, listing the three mitigations: `stacker` feature, `collapse`, and `CoyonedaExplicit`.

---

## Phase 2: API Parity

### 2.1 Add `new(f, fb)` constructor to `RcCoyoneda`

**Status: Implemented (option b).**

`rc_coyoneda.rs` lines 506-516: `pub fn new<B: 'a>(f: impl Fn(B) -> A + 'a, fb: ...) -> Self` with `Of<'a, B>: Clone`. Uses a dedicated `RcCoyonedaNewLayer` struct (lines 216-221) storing `fb` and `func: Rc<dyn Fn(B) -> A + 'a>`, matching option (b) from the plan. The layer implements `RcCoyonedaLowerRef` directly (lines 230-266), with `stacker` support.

### 2.2 Add `new(f, fb)` constructor to `ArcCoyoneda`

**Status: Implemented (option b).**

`arc_coyoneda.rs` lines 586-596: Same pattern as 2.1 with `Send + Sync` bounds on the function and `Clone + Send + Sync` on `Of<'a, B>`. Uses a dedicated `ArcCoyonedaNewLayer` struct (lines 274-279) with proper `unsafe impl Send/Sync` and safety comments.

### 2.3 Add `hoist` to `RcCoyoneda`

**Status: Implemented.**

`rc_coyoneda.rs` lines 554-562: `pub fn hoist<G>(self, nat: impl NaturalTransformation<F, G>) -> RcCoyoneda<'a, G, A>` with bounds `F: Functor, <G as Kind>::Of<'a, A>: Clone`. Matches the plan's signature.

### 2.4 Add `hoist` to `ArcCoyoneda`

**Status: Implemented.**

`arc_coyoneda.rs` lines 634-642: Same as 2.3 with `Clone + Send + Sync` bounds on `G::Of<'a, A>`.

### 2.5 Add inherent methods to `RcCoyoneda`

**Status: Implemented.**

All four inherent methods are present:

- `pure` (line 583): bounds `F: Pointed, Of<'a, A>: Clone`. Matches plan.
- `bind` (lines 616-624): bounds `F: Functor + Semimonad, Of<'a, B>: Clone`. Matches plan.
- `apply` (lines 662-670): bounds `F: Functor + Semiapplicative, Of<'a, C>: Clone`. Matches plan.
- `lift2` (lines 696-706): bounds `F: Functor + Lift, A: Clone, Of<'a, C>: Clone`. Matches plan.

### 2.6 Add inherent methods to `ArcCoyoneda`

**Status: Implemented.**

All four inherent methods are present with `Send + Sync` bounds:

- `pure` (line 663): bounds `F: Pointed, Of<'a, A>: Clone + Send + Sync`.
- `bind` (lines 696-703): bounds `F: Functor + Semimonad, Of<'a, B>: Clone + Send + Sync`.
- `apply` (lines 740-748): bounds `F: Functor + Semiapplicative, Of<'a, C>: Clone + Send + Sync`.
- `lift2` (lines 774-784): bounds `F: Functor + Lift, A: Clone, Of<'a, C>: Clone + Send + Sync`.

---

## Phase 4: Testing

### 4.1 Property-based tests for Functor laws

**Status: Implemented.**

Both `rc_coyoneda.rs` (lines 1003-1053) and `arc_coyoneda.rs` (lines 1035-1065) contain QuickCheck-based property tests:

- Functor identity law with `VecBrand` and `OptionBrand`.
- Functor composition law with `VecBrand` and `OptionBrand`.

For `RcCoyoneda`, the tests go through the `RcCoyonedaBrand` `Functor` impl (via `map::<RcCoyonedaBrand<...>, _, _>(...)`). For `ArcCoyoneda`, the tests use inherent `map` (since `ArcCoyonedaBrand` has no `Functor`).

**Not tested:** `CoyonedaBrand<VecBrand>` and `CoyonedaExplicitBrand<VecBrand, _>` are mentioned in the plan at step 4.1 but are NOT tested in the modified files. However, those types are not in the list of modified files, so they may already exist elsewhere or were out of scope for this implementation pass.

### 4.2 Property-based tests for Foldable laws

**Status: Partially implemented.**

`rc_coyoneda.rs` line 1056 has a `foldable_consistency_vec` QuickCheck test that compares `fold_map` through `RcCoyonedaBrand<VecBrand>` with a direct `fold_map` on `VecBrand`.

**Missing:** No Foldable property test for `ArcCoyoneda`. The `arc_coyoneda.rs` property test module (lines 1025-1076) does not include a Foldable consistency test.

### 4.3 Stack overflow tests

**Status: Implemented.**

`fp-library/tests/stack_safety.rs` lines 162-239:

- `test_rc_coyoneda_collapse_resets_depth` (line 170): builds 20 maps, collapses, 20 more maps.
- `test_arc_coyoneda_collapse_resets_depth` (line 189): same pattern.
- `test_rc_coyoneda_deep_chain_with_stacker` (line 212, `#[cfg(feature = "stacker")]`): 1,000 maps on `OptionBrand`.
- `test_arc_coyoneda_deep_chain_with_stacker` (line 228, `#[cfg(feature = "stacker")]`): same.

The collapse tests use only 20 maps (documented as "small depth to avoid stack overflow in debug builds"), which is reasonable. The stacker tests use 1,000 maps. The plan said "thousands of maps" but 1,000 qualifies.

### 4.4 Compile-fail tests

**Status: Partially implemented.**

- `RcCoyoneda` is `!Send`: Implemented in `fp-library/tests/ui/rc_coyoneda_not_send.rs` and `.stderr`. Correctly verifies that `RcCoyoneda` fails the `Send` bound.

**Missing:**

- No compile-fail test for `ArcCoyoneda` with a `!Send` payload failing to compile when sent across threads.
- No compile-fail test for `Coyoneda` being `!Clone`.

### 4.5 Concurrent access tests for `ArcCoyoneda`

**Status: Implemented.**

`fp-library/tests/thread_safety.rs` lines 83-120:

- `test_arc_coyoneda_concurrent_lower_ref` (line 84): 4 threads concurrently calling `lower_ref` on clones of the same `ArcCoyoneda`.
- `test_arc_coyoneda_shared_across_threads` (line 101): 4 threads sharing an `Arc<ArcCoyoneda<...>>` and calling `lower_ref`.

---

## Phase 5: Benchmarks

### 5.1 Add `RcCoyoneda` benchmarks

**Status: Implemented.**

`fp-library/benches/benchmarks/coyoneda.rs` lines 72-119:

- Map chain construction + single `lower_ref` (lines 73-85).
- Multiple `lower_ref` calls (3x) on the same value (lines 103-119).

**Note:** The plan also suggested "Clone + map + lower_ref pattern" benchmarks. The `RcCoyoneda_3x_lower` benchmark tests repeated lowering but there is no explicit "clone + map + lower_ref" benchmark that clones, maps the clone, and lowers.

### 5.2 Add `ArcCoyoneda` benchmarks

**Status: Partially implemented.**

`fp-library/benches/benchmarks/coyoneda.rs` lines 87-100: Map chain construction + single `lower_ref`.

**Missing:** No multiple-`lower_ref` benchmark for `ArcCoyoneda` (only `RcCoyoneda` has the `3x_lower` variant). No clone-then-lower benchmark.

---

## Phase 6: Documentation and Polish

### 6.1 Enable documentation validation on `ArcCoyoneda`

**Status: Implemented.**

`arc_coyoneda.rs` line 56 uses `#[fp_macros::document_module]` (without `no_validation`).

### 6.2 Add `From` conversions for Rc/Arc variants

**Status: Implemented.**

- `rc_coyoneda.rs` lines 862-896: `From<RcCoyoneda<'a, F, A>> for Coyoneda<'a, F, A>` with `F: Functor`.
- `arc_coyoneda.rs` lines 899-934: `From<ArcCoyoneda<'a, F, A>> for Coyoneda<'a, F, A>` with `F: Functor`.

### 6.3 Add `Debug` implementations

**Status: Partially implemented.**

- `RcCoyoneda`: `rc_coyoneda.rs` lines 829-858. Outputs `RcCoyoneda(<opaque>)`.
- `ArcCoyoneda`: `arc_coyoneda.rs` lines 867-897. Outputs `ArcCoyoneda(<opaque>)`.

**Missing:** The plan says "All four Coyoneda files." No `Debug` implementation was added to `Coyoneda` or `CoyonedaExplicit`. The `coyoneda_explicit.rs` file is listed as modified but does not contain a `Debug` impl. The `coyoneda.rs` file is not listed as modified.

### 6.4 Add inherent `fold_map(&self)` to Rc/Arc variants

**Status: Implemented.**

- `rc_coyoneda.rs` lines 441-450: `pub fn fold_map<FnBrand, M>(&self, func: impl Fn(A) -> M + 'a) -> M`.
- `arc_coyoneda.rs` lines 519-528: Same signature.

Both take `&self` and delegate to `F::fold_map` after `lower_ref()`.

### 6.5 Document fusion barriers in `CoyonedaExplicit`

**Status: Implemented.**

- `traverse` (line 382): "This is a fusion barrier: all accumulated maps are composed into the traversal function..."
- `apply` (line 440): "This is a fusion barrier: it calls `lower()` on both arguments..."
- `bind` (line 495): "This is a fusion barrier: all accumulated maps are composed into the bind callback..."

### 6.6 Document `CoyonedaExplicitBrand` Functor re-boxing cost

**Status: Implemented.**

`coyoneda_explicit.rs` lines 739-742: The `Functor::map` documentation for `CoyonedaExplicitBrand` explicitly notes: "Note: each call through this brand-level `map` allocates a `Box` for the composed function (via `.boxed()`). Zero-cost fusion (no allocation per map) is only available via the inherent [`CoyonedaExplicit::map`] method..."

### 6.7 Document `CoyonedaExplicit::boxed()` loop overhead

**Status: Implemented.**

`coyoneda_explicit.rs` lines 541-546: The `boxed()` method documentation notes: "When used in a loop (e.g., `coyo = coyo.map(f).boxed()` per iteration), each iteration creates a closure that captures the previous boxed function. The composed function chain has O(k) per-element overhead at `lower` time, matching `Coyoneda`'s cost profile..."

### 6.8 Harden `unsafe impl Send/Sync` on `ArcCoyonedaBase`

**Status: Implemented.**

`arc_coyoneda.rs` lines 941-951: Compile-time assertion verifies `ArcCoyonedaBase` is `Send + Sync` when `Of<'a, A>: Send + Sync`:

```rust
fn _check_base<'a, F: Kind + 'a, A: 'a>()
where
    <F as Kind>::Of<'a, A>: Send + Sync, {
    _assert_send::<ArcCoyonedaBase<'a, F, A>>();
    _assert_sync::<ArcCoyonedaBase<'a, F, A>>();
}
```

The existing `unsafe impl Send/Sync` (lines 125-137) is conditional on `Of<'a, A>: Send` and `Of<'a, A>: Sync` respectively, which is the correct and tightest possible approach.

### 6.9 Add compile-time assertion for `ArcCoyonedaMapLayer` Send/Sync

**Status: Implemented.**

`arc_coyoneda.rs` lines 953-958: Assertion checks that `ArcCoyonedaMapLayer` is unconditionally `Send + Sync` (no bounds on generic parameters in the assertion function). This is correct since both fields are `Arc<dyn ... + Send + Sync>`.

### 6.10 Document why `unsafe impl Send/Sync` is needed on `ArcCoyonedaMapLayer`

**Status: Implemented.**

`arc_coyoneda.rs` lines 183-201: Thorough safety comment covering:

- Why both fields are `Send + Sync`.
- Why the compiler cannot auto-derive.
- Why adding `Send + Sync` to `Kind::Of` is not feasible.
- Why a `SendKind` subtrait is not expressible.
- What would break soundness.
- Reference to compile-time assertions.

This matches all six points requested in the plan.

### 6.11 Document brand-level trait impl limitations in source code

**Status: Implemented.**

- `rc_coyoneda.rs` lines 717-732: Comment block above the brand-level impls explaining why `Pointed`, `Lift`, `Semiapplicative`, and `Semimonad` cannot be implemented. Covers the `Clone` bound blocker, contrast with `CoyonedaBrand`, Rust's limitation on extra where clauses, and notes that inherent methods are provided instead.
- `arc_coyoneda.rs` lines 795-810: Same documentation, additionally noting the `Send + Sync` limitation on `Functor::map` closures.

---

## Issues Found

### 1. Missing Foldable property test for `ArcCoyoneda` (plan step 4.2)

The `arc_coyoneda.rs` property test module has Functor identity, Functor composition, and `collapse` tests, but no Foldable consistency test. The `rc_coyoneda.rs` module has `foldable_consistency_vec` (line 1056), but `arc_coyoneda.rs` does not have an equivalent. Plan step 4.2 says "Verify `fold_map` consistency across all brands."

### 2. Missing compile-fail tests (plan step 4.4)

Only 1 of 3 specified compile-fail tests was implemented:

- **Implemented:** `RcCoyoneda` is `!Send` (`tests/ui/rc_coyoneda_not_send.rs`).
- **Missing:** `ArcCoyoneda` with a `!Send` payload fails to compile when sent across threads.
- **Missing:** `Coyoneda` is `!Clone`.

### 3. Missing `Debug` impls for `Coyoneda` and `CoyonedaExplicit` (plan step 6.3)

The plan says "All four Coyoneda files." Only `RcCoyoneda` and `ArcCoyoneda` have `Debug` impls. `Coyoneda` (`coyoneda.rs`, not in the modified files list) and `CoyonedaExplicit` (`coyoneda_explicit.rs`, listed as modified) do not have `Debug` impls.

### 4. Missing `ArcCoyoneda` repeated-lower benchmark (plan step 5.2)

The plan says "Same cases as 5.1" for `ArcCoyoneda` benchmarks. Step 5.1 includes "Multiple `lower_ref` calls on the same value (measures re-evaluation cost)." Only `RcCoyoneda` has the `3x_lower` benchmark variant (lines 103-119). `ArcCoyoneda` only has the single-lower benchmark.

### 5. `ArcCoyonedaNewLayer` has unconditional `unsafe impl Send/Sync` despite storing `Of<'a, B>`

`arc_coyoneda.rs` lines 290-303: The `unsafe impl Send/Sync` for `ArcCoyonedaNewLayer` is unconditional (only `F: Kind` bound), but the struct stores `fb: <F as Kind>::Of<'a, B>` which is not inherently `Send + Sync`. This is safe in practice because:

- The struct is private.
- The only construction site (`ArcCoyoneda::new`, line 592) has `Of<'a, B>: Clone + Send + Sync`.

However, this is inconsistent with `ArcCoyonedaBase` (lines 125-137), which correctly makes `unsafe impl Send` conditional on `Of<'a, A>: Send` and `unsafe impl Sync` conditional on `Of<'a, A>: Sync`. The same conditional approach should be used for `ArcCoyonedaNewLayer` for consistency and defense in depth. The compile-time assertion at lines 962-964 is also unconditional, which means it validates the overly-broad `unsafe impl` rather than catching a regression where someone constructs the struct without the bounds.

**Recommendation:** Change the `unsafe impl Send/Sync` for `ArcCoyonedaNewLayer` to be conditional on `Of<'a, B>: Send` / `Of<'a, B>: Sync` respectively, matching the pattern used for `ArcCoyonedaBase`. Update the compile-time assertion to include the bound.

### 6. `RcCoyonedaNewLayer::lower_ref` has `stacker` support unnecessarily

`rc_coyoneda.rs` lines 253-264: The `lower_ref` on `RcCoyonedaNewLayer` wraps its body in `stacker::maybe_grow`. However, `RcCoyonedaNewLayer` is a _base-level_ layer (it stores the functor value directly, not an inner `RcCoyoneda`). It does not recurse into another layer. The `stacker` wrapping serves no purpose here since there is no recursion to protect against. The same applies to `ArcCoyonedaNewLayer::lower_ref` (`arc_coyoneda.rs` lines 335-347).

This is not a bug (it just adds trivial overhead), but it is unnecessary code that deviates from the plan. The plan's step 1.1 and 1.2 specifically target `RcCoyonedaMapLayer::lower_ref` and `ArcCoyonedaMapLayer::lower_ref`, not the `NewLayer` variants.

---

## Summary of Issues by Severity

### Potential Improvement (style/consistency)

| #   | Issue                                                                             | Location                                            |
| --- | --------------------------------------------------------------------------------- | --------------------------------------------------- |
| 5   | `ArcCoyonedaNewLayer` unconditional `unsafe impl Send/Sync` should be conditional | `arc_coyoneda.rs:290-303`                           |
| 6   | Unnecessary `stacker` in `NewLayer::lower_ref`                                    | `rc_coyoneda.rs:253-264`, `arc_coyoneda.rs:335-347` |

### Missing Steps

| #   | Issue                                                        | Plan Step |
| --- | ------------------------------------------------------------ | --------- |
| 1   | No Foldable property test for `ArcCoyoneda`                  | 4.2       |
| 2   | 2 of 3 compile-fail tests missing                            | 4.4       |
| 3   | `Debug` not implemented for `Coyoneda` or `CoyonedaExplicit` | 6.3       |
| 4   | No repeated-lower benchmark for `ArcCoyoneda`                | 5.2       |

---

## Convention Compliance

### Formatting

The code uses hard tabs for indentation, vertical layout for function parameters, and single imports per line, consistent with the project's `rustfmt.toml` rules.

### Documentation attributes

All public functions and trait impls in the modified files include `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` with working code examples. This matches the project's documentation standards described in `CLAUDE.md`.

### No emoji or unicode

No emoji or unicode symbols found in any modified file. ASCII-only text throughout.

### Commit message style

Not assessed (not part of the implementation audit).

### Writing style

All bullet points in doc comments end with proper punctuation. No em dashes or en dashes found. Hyphens used correctly for hyphenated terms.
