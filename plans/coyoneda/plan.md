# Coyoneda Improvement Plan

This plan addresses the actionable issues identified in `summary.md`. Work is organized into phases by priority and dependency order.

---

## Phase 1: Stack Safety (High Priority, Correctness)

These changes prevent runtime crashes and should be done first.

### 1.1 Add `stacker` support to `RcCoyonedaMapLayer::lower_ref`

**File:** `fp-library/src/types/rc_coyoneda.rs`

Wrap the body of `RcCoyonedaMapLayer::lower_ref` in `stacker::maybe_grow(RED_ZONE, STACK_SIZE, || { ... })` behind `#[cfg(feature = "stacker")]`, matching the pattern in `CoyonedaMapLayer::lower` (`coyoneda.rs:282-288`).

### 1.2 Add `stacker` support to `ArcCoyonedaMapLayer::lower_ref`

**File:** `fp-library/src/types/arc_coyoneda.rs`

Same pattern as 1.1.

### 1.3 Add `collapse` method to `RcCoyoneda`

**File:** `fp-library/src/types/rc_coyoneda.rs`

```rust
pub fn collapse(self) -> RcCoyoneda<'a, F, A>
where
    F: Functor,
    <F as Kind>::Of<'a, A>: Clone,
{
    RcCoyoneda::lift(self.lower_ref())
}
```

This flattens accumulated layers into a single base layer, resetting the recursion depth. Document that it requires `F: Functor` and clones the base value.

### 1.4 Add `collapse` method to `ArcCoyoneda`

**File:** `fp-library/src/types/arc_coyoneda.rs`

Same pattern as 1.3, with the additional `Send + Sync` bounds on `F::Of<'a, A>`.

### 1.5 Document stack overflow risk

Add a "Stack Safety" section to the module docs of both `rc_coyoneda.rs` and `arc_coyoneda.rs`, matching the existing section in `coyoneda.rs`.

---

## Phase 2: API Parity (Medium Priority, Completeness)

These changes bring `RcCoyoneda`/`ArcCoyoneda` up to feature parity with `Coyoneda`.

### 2.1 Add `new(f, fb)` constructor to `RcCoyoneda`

**File:** `fp-library/src/types/rc_coyoneda.rs`

Creates a single-layer `RcCoyoneda` from a function and a functor value, matching `Coyoneda::new`.

### 2.2 Add `new(f, fb)` constructor to `ArcCoyoneda`

**File:** `fp-library/src/types/arc_coyoneda.rs`

Same pattern, with `Send + Sync` bounds on the function.

### 2.3 Add `hoist` to `RcCoyoneda`

**File:** `fp-library/src/types/rc_coyoneda.rs`

```rust
pub fn hoist<G>(self, nat: impl NaturalTransformation<F, G>) -> RcCoyoneda<'a, G, A>
where
    F: Functor,
    <G as Kind>::Of<'a, A>: Clone,
{
    RcCoyoneda::lift(nat.transform(self.lower_ref()))
}
```

Requires `F: Functor` (same limitation as `Coyoneda::hoist`).

### 2.4 Add `hoist` to `ArcCoyoneda`

**File:** `fp-library/src/types/arc_coyoneda.rs`

Same pattern with `Send + Sync` bounds.

### 2.5 Implement `Pointed` for `RcCoyonedaBrand`

**File:** `fp-library/src/types/rc_coyoneda.rs`

Delegate to `F::pure` and `RcCoyoneda::lift`.

### 2.6 Implement `Lift` for `RcCoyonedaBrand`

**File:** `fp-library/src/types/rc_coyoneda.rs`

Lower both arguments via `lower_ref`, delegate to `F::lift2`, re-lift.

### 2.7 Implement `Semiapplicative` for `RcCoyonedaBrand`

**File:** `fp-library/src/types/rc_coyoneda.rs`

Lower both arguments, delegate to `F::apply`, re-lift.

### 2.8 Implement `Semimonad` for `RcCoyonedaBrand`

**File:** `fp-library/src/types/rc_coyoneda.rs`

Lower the input, delegate to `F::bind` (lowering inside the callback), re-lift.

### 2.9 Add inherent methods to `ArcCoyoneda`

**File:** `fp-library/src/types/arc_coyoneda.rs`

Since `ArcCoyonedaBrand` cannot implement `Functor` (and therefore most other type classes), add inherent methods for `pure`, `apply`, `bind`, and `lift2` on `ArcCoyoneda` directly. Follow the pattern of `CoyonedaExplicit`'s inherent methods.

---

## Phase 3: Performance Optimization (Medium Priority)

### 3.1 Store functions inline in Rc/Arc map layers

**Files:** `fp-library/src/types/rc_coyoneda.rs`, `fp-library/src/types/arc_coyoneda.rs`

Currently, each `map` creates two reference-counted allocations: one for the layer struct and one for the function (`Rc<dyn Fn(B) -> A>`). `Coyoneda` avoids this by making `CoyonedaMapLayer` generic over `Func: Fn(B) -> A`, storing the function inline, and erasing the type through `Box<dyn CoyonedaInner>`.

Apply the same technique to `RcCoyonedaMapLayer` and `ArcCoyonedaMapLayer`:

1. Make the map layer generic: `RcCoyonedaMapLayer<'a, F, B, A, Func: Fn(B) -> A>`.
2. Store `func: Func` inline (not behind `Rc`).
3. The outer `Rc<dyn RcCoyonedaLowerRef>` erases the `Func` type.

This reduces allocations from 2 to 1 per `map` call.

**Trade-offs:** Changes internal architecture but not public API. Mirrors the proven `Coyoneda` pattern. The function call in `lower_ref` becomes a direct (monomorphized) call through the outer trait object vtable, reducing indirection from two levels to one.

---

## Phase 4: Testing (Medium Priority)

### 4.1 Add property-based tests for Functor laws

**File:** New test file or existing test modules

Use QuickCheck to verify:

- Identity law: `map(id, fa) == fa` for `CoyonedaBrand<VecBrand>`, `RcCoyonedaBrand<VecBrand>`, `CoyonedaExplicitBrand<VecBrand, _>`.
- Composition law: `map(f . g, fa) == map(f, map(g, fa))`.
- Test with `VecBrand` and `OptionBrand` as the underlying functor.

### 4.2 Add property-based tests for Foldable laws

Verify `fold_map` consistency across all brands.

### 4.3 Add stack overflow tests

Test deep chains (thousands of maps) with and without the `stacker` feature. Verify that `collapse` resets the depth.

### 4.4 Add compile-fail tests for `ArcCoyoneda` Send/Sync soundness

**File:** New `trybuild` test file

Write tests proving that:

- `ArcCoyoneda` with a `!Send` payload fails to compile when sent across threads.
- `RcCoyoneda` is `!Send`.
- `Coyoneda` is `!Clone`.

### 4.5 Add concurrent access tests for `ArcCoyoneda`

Test shared `ArcCoyoneda` with concurrent `lower_ref` calls from multiple threads.

---

## Phase 5: Benchmarks (Medium Priority)

### 5.1 Add `RcCoyoneda` benchmarks

**File:** `fp-library/benches/benchmarks/coyoneda.rs`

Benchmark cases:

- Map chain construction + single `lower_ref` (compare against `Coyoneda::lower`).
- Multiple `lower_ref` calls on the same value (measures re-evaluation cost).
- Clone + map + lower_ref pattern.

### 5.2 Add `ArcCoyoneda` benchmarks

Same cases as 5.1 with `ArcCoyoneda`.

### 5.3 Benchmark inline-function optimization (Phase 3)

Before/after benchmarks for the inline-function change in Phase 3.

---

## Phase 6: Documentation and Polish (Low Priority)

### 6.1 Enable documentation validation on `ArcCoyoneda`

**File:** `fp-library/src/types/arc_coyoneda.rs`

Remove `no_validation` from `#[fp_macros::document_module(no_validation)]` and fix any resulting warnings.

### 6.2 Add `From` conversions for Rc/Arc variants

**Files:** `fp-library/src/types/rc_coyoneda.rs`, `fp-library/src/types/arc_coyoneda.rs`

Add:

- `From<RcCoyoneda<'a, F, A>> for Coyoneda<'a, F, A>` (via `lower_ref` + `lift`, requires `F: Functor`).
- `From<ArcCoyoneda<'a, F, A>> for Coyoneda<'a, F, A>` (via `lower_ref` + `lift`, requires `F: Functor`).

### 6.3 Add `Debug` implementations

**Files:** All four Coyoneda files.

Implement `Debug` with opaque output (e.g., `Coyoneda(<opaque>)`). For `CoyonedaExplicit`, show the stored functor value when it implements `Debug`.

### 6.4 Add inherent `fold_map(&self)` to Rc/Arc variants

**Files:** `fp-library/src/types/rc_coyoneda.rs`, `fp-library/src/types/arc_coyoneda.rs`

Provide a non-consuming fold path that complements `lower_ref`.

### 6.5 Document fusion barriers in `CoyonedaExplicit`

**File:** `fp-library/src/types/coyoneda_explicit.rs`

Add "This is a fusion barrier" notes to `traverse` and `bind` doc comments, matching the existing note on `apply`.

### 6.6 Document `CoyonedaExplicitBrand` Functor re-boxing cost

**File:** `fp-library/src/types/coyoneda_explicit.rs` or `fp-library/src/brands.rs`

Note that zero-cost fusion applies only via inherent methods; the brand-level `Functor::map` allocates a `Box` per call.

### 6.7 Document `CoyonedaExplicit::boxed()` loop overhead

**File:** `fp-library/src/types/coyoneda_explicit.rs`

Clarify that `.boxed()` in a loop creates a closure chain with O(k) per-element overhead, matching `Coyoneda`'s cost profile. The fusion advantage is realized only with static (compile-time) composition.

### 6.8 Improve unsafe impl safety comments

**File:** `fp-library/src/types/arc_coyoneda.rs`

Make safety comments more explicit about the full invariant chain and what modifications would break soundness.

---

## Dependency Graph

```
Phase 1 (Stack Safety)
  |
  v
Phase 2 (API Parity)  --->  Phase 3 (Performance Optimization)
  |                              |
  v                              v
Phase 4 (Testing)  <---  Phase 5 (Benchmarks)
  |
  v
Phase 6 (Documentation and Polish)
```

Phases 1 and 2 are independent of each other but should precede testing. Phase 3 (inline functions) can proceed in parallel with Phase 2 but should be benchmarked (Phase 5) before and after. Phase 6 can proceed at any time.

---

## Estimated Scope

| Phase            | Steps | Complexity                                                |
| ---------------- | ----- | --------------------------------------------------------- |
| 1. Stack Safety  | 5     | Low; mechanical changes mirroring existing patterns       |
| 2. API Parity    | 9     | Medium; follows established patterns from `CoyonedaBrand` |
| 3. Performance   | 1     | Medium; internal refactor, no API change                  |
| 4. Testing       | 5     | Medium; test infrastructure setup                         |
| 5. Benchmarks    | 3     | Low; extends existing benchmark file                      |
| 6. Documentation | 8     | Low; documentation and small additions                    |
