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
pub fn collapse(&self) -> RcCoyoneda<'a, F, A>
where
    F: Functor,
    <F as Kind>::Of<'a, A>: Clone,
{
    RcCoyoneda::lift(self.lower_ref())
}
```

Takes `&self` rather than `self` since `RcCoyoneda` is cheaply `Clone`; the caller can drop the original afterward if desired, but does not have to clone before collapsing. Flattens accumulated layers into a single base layer, resetting the recursion depth. Document that it requires `F: Functor` and clones the base value.

### 1.4 Add `collapse` method to `ArcCoyoneda`

**File:** `fp-library/src/types/arc_coyoneda.rs`

Same pattern as 1.3, with `&self`, and the additional `Send + Sync` bounds on `F::Of<'a, A>`:

```rust
pub fn collapse(&self) -> ArcCoyoneda<'a, F, A>
where
    F: Functor,
    <F as Kind>::Of<'a, A>: Clone + Send + Sync,
{
    ArcCoyoneda::lift(self.lower_ref())
}
```

### 1.5 Document stack overflow risk

Add a "Stack Safety" section to the module docs of both `rc_coyoneda.rs` and `arc_coyoneda.rs`, matching the existing section in `coyoneda.rs`.

---

## Phase 2: API Parity (Medium Priority, Completeness)

These changes bring `RcCoyoneda`/`ArcCoyoneda` up to feature parity with `Coyoneda`.

### 2.1 Add `new(f, fb)` constructor to `RcCoyoneda`

**File:** `fp-library/src/types/rc_coyoneda.rs`

Creates a single-layer `RcCoyoneda` from a function and a functor value, matching `Coyoneda::new`. Requires `F::Of<'a, B>: Clone` on the base value (needed for `RcCoyonedaLowerRef` trait object coercion).

**Architectural decision:** `Coyoneda::new` uses a dedicated `CoyonedaNewLayer` that stores the function inline (generic `Func`), erased by `Box<dyn CoyonedaInner>` (1 allocation). Two options for `RcCoyoneda`:

- **(a) Compose existing layers:** Create an `RcCoyonedaBase` + `RcCoyonedaMapLayer`. Simple but 3 `Rc` allocations (base, map layer, function).
- **(b) Dedicated `RcCoyonedaNewLayer`:** A new struct storing `fb` and `func: Rc<dyn Fn(B) -> A>`, implementing `RcCoyonedaLowerRef` directly. 1 `Rc` allocation (the layer itself), matching `Coyoneda::new`'s efficiency.

**Recommendation:** Option (b) for allocation efficiency.

### 2.2 Add `new(f, fb)` constructor to `ArcCoyoneda`

**File:** `fp-library/src/types/arc_coyoneda.rs`

Same pattern as 2.1 with `Send + Sync` bounds on the function and `Clone + Send + Sync` on `F::Of<'a, B>`. Use a dedicated `ArcCoyonedaNewLayer` (option b).

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

### 2.5 Add inherent methods to `RcCoyoneda` (revised: brand-level impls infeasible)

**File:** `fp-library/src/types/rc_coyoneda.rs`

~~Originally planned as brand-level trait impls (`Pointed`, `Lift`, `Semiapplicative`, `Semimonad` for `RcCoyonedaBrand`).~~

**Feasibility finding:** Brand-level trait impls are **not possible** due to a `Clone` bound that cannot be expressed in the trait method signatures. The root cause:

- `RcCoyoneda` wraps `Rc<dyn RcCoyonedaLowerRef>`. Constructing this requires `F::Of<'a, A>: Clone` because `RcCoyonedaBase`'s `RcCoyonedaLowerRef` impl has that bound, and the bound must be satisfied to coerce the struct to the trait object.
- `Coyoneda::lift` has no such requirement because `CoyonedaBase::lower` consumes `self: Box<Self>` (moving, no clone needed).
- Trait method signatures (`Pointed::pure`, `Semimonad::bind`, `Lift::lift2`, `Semiapplicative::apply`) don't include `Clone` bounds on `F::Of<'a, A>`, and Rust doesn't allow adding extra where clauses to methods in trait impls beyond what the trait definition specifies.

**Revised approach:** Add inherent methods (`pure`, `apply`, `bind`, `lift2`) directly on `RcCoyoneda`, specifying the `Clone` bound freely. This matches the approach for `ArcCoyoneda` and follows the pattern established by `CoyonedaExplicit`'s inherent methods.

Bounds for each:

- `pure(a)`: `F: Pointed`, `F::Of<'a, A>: Clone`
- `bind(self, f)`: `F: Functor + Semimonad`, `F::Of<'a, B>: Clone`
- `apply(ff, fa)`: `F: Functor + Semiapplicative`, `F::Of<'a, B>: Clone`
- `lift2(func, fa, fb)`: `F: Functor + Lift`, `F::Of<'a, C>: Clone`

### 2.6 Add inherent methods to `ArcCoyoneda`

**File:** `fp-library/src/types/arc_coyoneda.rs`

Same approach as 2.5, with additional `Send + Sync` bounds on `F::Of<'a, A>`. Confirmed feasible: the "lower, delegate, re-lift" pattern works because the closures passed to `F`'s trait methods are plain `impl Fn + 'a` (no `Send + Sync` needed at the trait level); the `Send + Sync` constraint that blocks `Functor` for `ArcCoyonedaBrand` only applies to closures _stored_ inside Arc-wrapped layers.

Bounds for each:

- `pure(a)`: `F: Pointed`, `F::Of<'a, A>: Clone + Send + Sync`
- `bind(self, f)`: `F: Functor + Semimonad`, `F::Of<'a, B>: Clone + Send + Sync`
- `apply(ff, fa)`: `F: Functor + Semiapplicative`, `F::Of<'a, B>: Clone + Send + Sync`
- `lift2(func, fa, fb)`: `F: Functor + Lift`, `F::Of<'a, C>: Clone + Send + Sync`

---

## Phase 3: ~~Performance Optimization~~ (Removed)

### ~~3.1 Store functions inline in Rc/Arc map layers~~

**Status: Infeasible. Removed from plan.**

The original proposal was to make `RcCoyonedaMapLayer` generic over `Func: Fn(B) -> A`, storing the function inline and erasing `Func` through `Rc<dyn RcCoyonedaLowerRef>`, mirroring `Coyoneda`'s `CoyonedaMapLayer` pattern.

**Why it doesn't work:** `lower_ref(&self)` borrows `self`, so `self.func` cannot be moved into the closure passed to `F::map` (which requires `impl Fn(B) -> A + 'a`). The borrow lifetime of `&self.func` is shorter than `'a`, so a closure capturing `&self.func` fails to satisfy the `'a` bound. The only workarounds are:

- **`Func: Clone`**: Clone the function into the closure. But concrete closure types generally don't implement `Clone` (only when all captures are `Clone`), so `map` would still need to wrap the function in `Rc`/`Arc` to make it cloneable, eliminating the allocation savings.
- **Restructure `Functor::map` to relax the `'a` bound**: Enormous blast radius across the entire library.

This is the fundamental difference from `Coyoneda`: `CoyonedaMapLayer::lower` takes `self: Box<Self>` (consuming), so it can move `self.func` into the closure with no lifetime constraint. `RcCoyonedaMapLayer::lower_ref` takes `&self` (borrowing), which prevents this.

The current approach (2 `Rc`/`Arc` allocations per `map`: one for the layer, one for `Rc<dyn Fn(B) -> A>`) is the correct design given the borrowing constraint.

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

### ~~5.3 Benchmark inline-function optimization (Phase 3)~~

Removed. Phase 3 (inline function optimization) was found to be infeasible.

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

### 6.8 ~~Eliminate~~ Harden `unsafe impl Send/Sync` on `ArcCoyonedaBase`

**File:** `fp-library/src/types/arc_coyoneda.rs`

**Original proposal (infeasible):** Add `Send + Sync` bounds to the struct's where clause to let the compiler auto-derive. This does NOT work because Rust's auto-`Send/Sync` derivation requires ALL type parameters to be `Send/Sync`, not just fields. The `F` parameter blocks auto-derivation even though it only appears in where-clauses, never as stored data.

**Alternative: Add `F: Send + Sync` to the struct's where clause.** Brand types are zero-sized marker structs, which are auto-`Send/Sync`. This would work but adds a bound that is always satisfied in practice yet clutters the API. It also doesn't help `ArcCoyonedaMapLayer` (see 6.9).

**Revised approach:** Keep the `unsafe impl` but add a compile-time assertion (same approach as 6.9):

```rust
const _: () = {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn check<'a, F: Kind_cdc7cd43dac7585f + 'a, A: 'a>()
    where
        <F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Send + Sync,
    {
        assert_send::<ArcCoyonedaBase<'a, F, A>>();
        assert_sync::<ArcCoyonedaBase<'a, F, A>>();
    }
};
```

This verifies the `unsafe impl` at compile time and catches regressions if the struct's fields change.

### 6.9 Add compile-time assertion for `ArcCoyonedaMapLayer` Send/Sync

**File:** `fp-library/src/types/arc_coyoneda.rs`

The `unsafe impl Send/Sync` on `ArcCoyonedaMapLayer` cannot be eliminated. Both fields are `Arc<dyn ... + Send + Sync>` (unconditionally `Send + Sync`), but the compiler cannot auto-derive because the struct carries `F: Kind` in its where clause, and the compiler conservatively assumes `F`'s associated types might be `!Send`, even though `F` only appears in type-level positions within trait object bounds, never as stored data. Approaches assessed and ruled out:

- Adding `Send + Sync` to the `Kind` trait: enormous blast radius (breaks `Thunk`, `RcCoyoneda`, `FnBrand<RcBrand>`, etc.).
- Separate `SendKind` trait: requires `for<'a, A: 'a>` quantification over types, not supported on stable Rust.
- `Send + Sync` as supertraits of `ArcCoyonedaLowerRef`: already the case; not sufficient because the blocker is the `F` parameter, not the trait object.
- Wrapper types, `PhantomData` tricks: do not address the root cause (the generic `F` in the where clause).

Add a compile-time assertion as a regression guard:

```rust
const _: () = {
    fn assert_send_sync<T: Send + Sync>() {}
    fn check<'a, F: Kind_cdc7cd43dac7585f + 'a, B: 'a, A: 'a>() {
        assert_send_sync::<Arc<dyn ArcCoyonedaLowerRef<'a, F, B> + 'a>>();
        assert_send_sync::<Arc<dyn Fn(B) -> A + Send + Sync + 'a>>();
    }
};
```

### 6.10 Document why `unsafe impl Send/Sync` is needed on `ArcCoyonedaMapLayer`

**File:** `fp-library/src/types/arc_coyoneda.rs`

Replace the existing SAFETY comments on `ArcCoyonedaMapLayer`'s `unsafe impl Send/Sync` with a thorough explanation covering:

- Both fields are `Arc<dyn ... + Send + Sync>`, which are unconditionally `Send + Sync` regardless of `F`.
- The compiler cannot auto-derive because the struct has a `where F: Kind` clause, and `Kind`'s associated type `Of` has no `Send/Sync` bounds. The compiler conservatively blocks auto-derivation for any struct parameterized over a type whose associated types lack these bounds, even when the generic parameter only appears in type-level positions within trait object bounds and is never stored as data.
- Adding `Send + Sync` to `Kind::Of` is not feasible because it would break all `!Send`/`!Sync` types in the library (`Thunk`, `RcCoyoneda`, `FnBrand<RcBrand>`, etc.).
- A `SendKind` subtrait is not expressible on stable Rust (requires higher-ranked type quantification, not just lifetime quantification).
- The compile-time assertion (step 6.9) guards against regressions if the struct's fields are modified.
- List what would break soundness: adding a field that is not `Send + Sync`, or removing `Send + Sync` from `ArcCoyonedaLowerRef`'s supertraits or from the `func` field's trait object bounds.

### 6.11 Document brand-level trait impl limitations in source code

**Files:** `fp-library/src/types/rc_coyoneda.rs`, `fp-library/src/types/arc_coyoneda.rs`

Add documentation (in the module docs and near the existing brand trait impls) explaining why `Pointed`, `Lift`, `Semiapplicative`, and `Semimonad` cannot be implemented for `RcCoyonedaBrand` or `ArcCoyonedaBrand`. The explanation should cover:

- The `Clone` bound on `F::Of<'a, A>` required by `RcCoyonedaBase`'s `RcCoyonedaLowerRef` impl (needed to coerce to the trait object at construction time).
- Why `CoyonedaBrand` avoids this: `Coyoneda::lift` has no `Clone` requirement because `CoyonedaBase::lower` consumes `self: Box<Self>`.
- That Rust does not allow adding extra where clauses to trait method impls beyond what the trait definition specifies, so the `Clone` bound cannot be expressed.
- For `ArcCoyonedaBrand`, the additional `Send + Sync` limitation on `Functor::map` closures (already documented).
- That inherent methods (`pure`, `apply`, `bind`, `lift2`) are provided instead, with the `Clone` (and `Send + Sync` for Arc) bounds specified directly.

---

## Dependency Graph

```
Phase 1 (Stack Safety)
  |
  v
Phase 2 (API Parity)
  |
  v
Phase 4 (Testing)  <---  Phase 5 (Benchmarks)
  |
  v
Phase 6 (Documentation and Polish)
```

Phase 1 and Phase 2 are independent of each other but should both precede testing. Phase 3 has been removed (infeasible). Phase 6 can proceed at any time.

---

## Deferred Items

The following items from the summary are not addressed in this plan and are deferred for future consideration:

- **Consuming `lower(self)` for Rc/Arc variants** (summary item 3.3): Would attempt `Rc::try_unwrap`/`Arc::try_unwrap` to avoid cloning when the refcount is 1. Deferred due to implementation complexity (the inner trait would need a `lower_owned(self: Box<Self>)` fallback method) and low priority.

---

## Estimated Scope

| Phase              | Steps | Complexity                                                 |
| ------------------ | ----- | ---------------------------------------------------------- |
| 1. Stack Safety    | 5     | Low; mechanical changes mirroring existing patterns        |
| 2. API Parity      | 6     | Medium; inherent methods (brand impls infeasible, see 2.5) |
| ~~3. Performance~~ | ~~0~~ | ~~Removed; inline function optimization infeasible~~       |
| 4. Testing         | 5     | Medium; test infrastructure setup                          |
| 5. Benchmarks      | 2     | Low; extends existing benchmark file                       |
| 6. Documentation   | 11    | Low; documentation, unsafe hardening, and small additions  |
