# Coyoneda Implementation Assessment

## 1. Overview

This assessment covers the four Coyoneda implementations in `fp-library/src/types/`:

- `coyoneda.rs` -- `Coyoneda<'a, F, A>`: The primary free functor using boxed trait objects (`Box<dyn CoyonedaInner>`). Consumes self on `lower()`. Not `Clone`, not `Send`.
- `coyoneda_explicit.rs` -- `CoyonedaExplicit<'a, F, B, A, Func>`: Zero-cost variant exposing the intermediate type `B` and function type `Func`. Enables compile-time composition and single-pass fusion.
- `rc_coyoneda.rs` -- `RcCoyoneda<'a, F, A>`: Reference-counted variant using `Rc<dyn RcCoyonedaLowerRef>`. `Clone` but not `Send`.
- `arc_coyoneda.rs` -- `ArcCoyoneda<'a, F, A>`: Thread-safe variant using `Arc<dyn ArcCoyonedaLowerRef>`. `Clone`, `Send`, and `Sync`.

Supporting files examined: `brands.rs`, `kinds.rs`, `classes/functor.rs`, `classes/foldable.rs`, `classes/semimonad.rs`, `functions.rs`, `benches/benchmarks/coyoneda.rs`.

---

## 2. Identified Issues

### 2.1. No map fusion in `Coyoneda` (fundamental, by design)

**Files:** `coyoneda.rs` lines 279-294
**Severity:** Design limitation (documented)

`Coyoneda::lower()` recurses through k layers, calling `F::map` once per layer. For `VecBrand`, this means k full traversals of the Vec. PureScript's `Coyoneda` composes `f <<< k` eagerly so `lower` calls `map` exactly once.

The module documentation at lines 14-36 and 57-65 acknowledges this and explains why: Rust trait objects cannot have generic methods, preventing the `map_inner<C>` method needed for cross-layer function composition.

**Impact:** For eager containers like `Vec`, chaining k maps through `Coyoneda` is strictly worse than chaining k calls to `map` directly, since it adds boxing overhead on top of the same number of traversals. The only benefit is deferring the choice of when to evaluate, and providing a `Functor` instance for non-`Functor` types.

**Approaches:**

A. **Status quo (current).** Accept the limitation and steer users to `CoyonedaExplicit` for fusion.

- Trade-off: Clear and honest, but `Coyoneda` is a performance trap for naive users who expect fusion behavior like PureScript's.

B. **Add a `fuse()` method** that converts `Coyoneda` to `CoyonedaExplicit` then back, by lowering and re-lifting.

- Trade-off: This is just `collapse()` with a different name. Does not solve the fundamental problem, but might clarify intent.

C. **Internal composition via `Box<dyn Fn>` chaining.** Store a single `Box<dyn FnMut(?) -> ?>` and compose new functions into it.

- Trade-off: Impossible due to the existential type `B` being hidden. You cannot compose `Fn(B) -> A` with `Fn(A) -> C` into a `Fn(B) -> C` when `B` is erased. This is the core constraint.

**Recommendation:** The current approach is correct. The documentation is thorough. Consider adding a lint-style warning in the `map` method documentation itself (not just the module docs) to direct users to `CoyonedaExplicit` when fusion matters.

---

### 2.2. `Fn` bound instead of `FnOnce` for mapping functions

**Files:** `coyoneda.rs` line 525, `rc_coyoneda.rs` line 320, `arc_coyoneda.rs` line 358, `coyoneda_explicit.rs` line 210
**Severity:** Medium

All `map` methods require `impl Fn(A) -> B + 'a`, not `impl FnOnce(A) -> B + 'a`. The `Functor::map` trait itself also requires `impl Fn(A) -> B + 'a` (line 122 of `functor.rs`). This is consistent across the library but is a meaningful restriction: closures that move out of captured state (e.g., `move |x| (x, captured_vec)` where `captured_vec` is consumed) cannot be used.

For `RcCoyoneda` and `ArcCoyoneda`, `Fn` is actually required because `lower_ref()` can be called multiple times, so the function must be re-invokable. But for `Coyoneda` (which consumes self on `lower()`), `FnOnce` would be sufficient in theory.

However, `CoyonedaMapLayer` stores functions inline and passes them to `F::map`, which also requires `Fn`. So the constraint propagates from the `Functor` trait definition itself. This is a library-wide design decision, not specific to Coyoneda.

**Approaches:**

A. **Accept as-is.** The `Fn` requirement comes from the `Functor` trait and is a library-wide invariant.

- Trade-off: Consistent, but less flexible than possible.

B. **Change `Functor::map` to accept `FnOnce`.** This would ripple through the entire library.

- Trade-off: Massive change. Would require careful analysis of every `Functor` implementation. Many implementations (e.g., `Vec::map` via `Iterator::map`) actually do accept `FnMut`/`FnOnce`. But it would break backward compatibility and complicate shared/cloneable function wrappers.

**Recommendation:** Leave as-is. This is a library-wide concern, not specific to Coyoneda.

---

### 2.3. No stack safety mitigations in `RcCoyoneda` or `ArcCoyoneda`

**Files:** `rc_coyoneda.rs` lines 172-178, `arc_coyoneda.rs` lines 212-218
**Severity:** Medium

`Coyoneda` has two stack overflow mitigations: the `stacker` feature (line 282) and the `collapse()` method (line 494). Neither `RcCoyoneda` nor `ArcCoyoneda` have either of these.

`RcCoyonedaMapLayer::lower_ref` (line 172-178) and `ArcCoyonedaMapLayer::lower_ref` (line 212-218) both recursively call `self.inner.lower_ref()`, meaning deep chains will overflow the stack just as `Coyoneda` would without mitigations.

Since `RcCoyoneda` and `ArcCoyoneda` are `Clone`, users are more likely to build long chains (e.g., in loops that clone and extend). Neither type documents the stack overflow risk.

**Approaches:**

A. **Add `stacker` support to `RcCoyonedaMapLayer::lower_ref` and `ArcCoyonedaMapLayer::lower_ref`.**

- Trade-off: Minimal code change, consistent with `Coyoneda`'s approach. Adds a conditional dependency.

B. **Add a `collapse()` method.**

- For `RcCoyoneda`: `collapse()` would require calling `lower_ref()` (which clones the base) then `lift()`. This is semantically straightforward.
- For `ArcCoyoneda`: Same approach.
- Trade-off: Provides a manual escape hatch. Does not solve the problem for users who forget to call it.

C. **Document the limitation.**

- Trade-off: Low effort, but users will still hit the problem.

**Recommendation:** Implement approach A (add `stacker` support) and B (add `collapse`). The `lower_ref` methods have the same recursive structure as `Coyoneda::lower`, so the same mitigations apply directly.

---

### 2.4. `lower_ref()` re-evaluates the entire chain on every call (no memoization)

**Files:** `rc_coyoneda.rs` lines 290-293, `arc_coyoneda.rs` lines 326-329
**Severity:** Medium

`RcCoyoneda::lower_ref()` and `ArcCoyoneda::lower_ref()` recompute the full chain of `F::map` calls every time they are invoked. The test `lower_ref_multiple_times` at `rc_coyoneda.rs` line 501 demonstrates this by calling `lower_ref()` three times, each time recomputing.

For `RcCoyoneda`, each `lower_ref()` also clones the base value (`RcCoyonedaBase::lower_ref` at line 131 calls `self.fa.clone()`).

This is documented ("Clones the underlying value on each call to `lower_ref`" at line 95), but it may surprise users who expect the `Rc`/`Arc` wrapping to imply some form of caching. The module doc at lines 11-14 mentions the cost clearly.

**Approaches:**

A. **Add memoization via `OnceCell`/`LazyCell`.** Cache the result of `lower_ref()` in the outer struct. For `RcCoyoneda`, use `RefCell<Option<...>>` or `OnceCell`. For `ArcCoyoneda`, use `OnceLock`.

- Trade-off: Adds memory overhead (stores both the chain and the result). Introduces interior mutability. The cached value's type `F::Of<'a, A>` must be `Clone` to return from `&self`. But since `lower_ref` already produces an owned value, caching it and cloning on subsequent calls would save the `F::map` traversals. This changes `lower_ref` from O(k \* n) per call to O(1) amortized (where k is chain depth and n is container size).

B. **Provide a separate `force` + `get` pattern.** Similar to the library's `Lazy` types, where `force()` evaluates and caches, then `get()` returns a reference.

- Trade-off: Different API contract. Would require the output type to implement `Clone` for returning owned values, or returning references.

C. **Accept as-is.** The current behavior is predictable and simple. Users who want memoization can call `lower_ref()` once and store the result.

- Trade-off: The simplest option. Users pay for re-evaluation only if they call `lower_ref()` multiple times.

**Recommendation:** Approach C (accept as-is) is reasonable given the existing `Lazy` types in the library. If memoization is desired, users should use the `Lazy` infrastructure. Adding memoization to `RcCoyoneda`/`ArcCoyoneda` would blur the boundary between these types and the `Lazy` types, creating overlap. However, the documentation should more prominently warn about repeated re-evaluation cost.

---

### 2.5. `RcCoyoneda` and `ArcCoyoneda` are missing many type class instances

**Files:** `rc_coyoneda.rs` lines 340-422, `arc_coyoneda.rs` lines 370-426
**Severity:** Medium

Comparing the implementations:

| Type class        | `CoyonedaBrand` | `RcCoyonedaBrand` | `ArcCoyonedaBrand` |
| ----------------- | --------------- | ----------------- | ------------------ |
| `Functor`         | Yes             | Yes               | No (documented)    |
| `Pointed`         | Yes             | No                | No                 |
| `Foldable`        | Yes             | Yes               | Yes                |
| `Lift`            | Yes             | No                | No                 |
| `ApplyFirst`      | Yes             | No                | No                 |
| `ApplySecond`     | Yes             | No                | No                 |
| `Semiapplicative` | Yes             | No                | No                 |
| `Semimonad`       | Yes             | No                | No                 |

`RcCoyoneda` is `Clone`, which should enable more type class instances than `Coyoneda` (which cannot implement `Traversable` or `Semiapplicative` precisely because it is not `Clone`). Despite this advantage, `RcCoyonedaBrand` has fewer instances than `CoyonedaBrand`.

For `ArcCoyonedaBrand`, the absence of `Functor` is explained at line 375-379: the HKT trait signatures lack `Send + Sync` bounds on closure parameters. This is a fundamental limitation of the current HKT encoding.

For `RcCoyonedaBrand`, there is no such fundamental blocker. The missing instances (`Pointed`, `Lift`, `Semiapplicative`, `Semimonad`) could all be implemented by analogy with `CoyonedaBrand`'s implementations, using `lower_ref()` to extract values and delegating to `F`'s instances.

**Approaches:**

A. **Implement the missing instances for `RcCoyonedaBrand`.** `Pointed`, `Lift`, `Semiapplicative`, `Semimonad` can all follow the `CoyonedaBrand` pattern: lower, delegate to `F`, re-lift.

- Trade-off: More code to maintain. Each instance lowers (clones base, traverses chain) before delegating, so performance characteristics should be documented.

B. **Add inherent methods on `RcCoyoneda` and `ArcCoyoneda`** for `pure`, `apply`, `bind`, etc., analogous to `CoyonedaExplicit`'s inherent `apply` and `bind` methods.

- Trade-off: Does not integrate with the HKT system, but provides the functionality for direct use.

**Recommendation:** Implement approach A for `RcCoyonedaBrand` (the missing instances are straightforward). For `ArcCoyonedaBrand`, implement approach B (inherent methods) since the HKT limitation prevents trait-level integration.

---

### 2.6. `CoyonedaExplicitBrand` requires `B: 'static`

**Files:** `coyoneda_explicit.rs` line 703, `brands.rs` line 160
**Severity:** Low-medium

```rust
impl_kind! {
    impl<F: Kind_cdc7cd43dac7585f + 'static, B: 'static> for CoyonedaExplicitBrand<F, B> {
        type Of<'a, A: 'a>: 'a = BoxedCoyonedaExplicit<'a, F, B, A>;
    }
}
```

The `B: 'static` bound is required by the `Kind` trait's design (brand type parameters must outlive all possible `'a`). This is documented in the module docs at line 39: "`B: 'static` required for brand | No | Yes". However, it means that borrowed data as the intermediate type is impossible when using the brand-level interface.

This rarely matters in practice since most intermediate types in map chains are owned values, but it is a difference from `CoyonedaBrand` where the existential `B` is hidden and can have any lifetime.

**Approaches:**

A. **Accept as-is.** The limitation is documented and rarely hits in practice.

B. **Allow non-static `B` by parameterizing the brand with a lifetime.** E.g., `CoyonedaExplicitBrand<'b, F, B>` where `B: 'b`.

- Trade-off: Adds a lifetime parameter to the brand, complicating its use. The `Kind` trait design may not support this.

**Recommendation:** Accept as-is. The `'static` requirement is a natural consequence of the brand system and is documented.

---

### 2.7. `Coyoneda::hoist` requires `F: Functor` unnecessarily

**Files:** `coyoneda.rs` lines 572-579
**Severity:** Low

```rust
pub fn hoist<G: Kind_cdc7cd43dac7585f + 'a>(
    self,
    nat: impl NaturalTransformation<F, G>,
) -> Coyoneda<'a, G, A>
where
    F: Functor, {
    Coyoneda::lift(nat.transform(self.lower()))
}
```

The implementation lowers to `F A` (requiring `F: Functor`), applies the natural transformation, then re-lifts. PureScript's `hoistCoyoneda` applies the transformation directly to the hidden `F B` without lowering, so it does not require `Functor`. The module docs at lines 72-77 acknowledge this.

The root cause is the same as the fusion issue: the hidden existential `B` cannot be accessed through the trait object. `CoyonedaExplicit::hoist` (line 285-293) does not require `F: Functor` because `B` is visible.

**Approaches:**

A. **Accept as-is.** The limitation is fundamental to the trait-object encoding.

B. **Add a `hoist_inner` method to `CoyonedaInner`.** This would need to be generic over `G`, breaking dyn-compatibility. Not possible.

C. **Provide a combined `lift_hoist` constructor** that creates a `Coyoneda<G, A>` directly from an `F A` and a natural transformation, without going through layers.

- Trade-off: Only works for freshly lifted values, not for values that already have accumulated maps.

**Recommendation:** Accept as-is. Users who need `hoist` without `Functor` should use `CoyonedaExplicit`.

---

### 2.8. `Foldable` for `CoyonedaBrand` requires `F: Functor`

**Files:** `coyoneda.rs` lines 662-709
**Severity:** Low

```rust
impl<F: Functor + Foldable + 'static> Foldable for CoyonedaBrand<F> {
    fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
        func: impl Fn(A) -> M + 'a,
        fa: Apply!(...),
    ) -> M
    where
        M: Monoid + 'a,
        FnBrand: CloneableFn + 'a, {
        F::fold_map::<FnBrand, A, M>(func, fa.lower())
    }
}
```

The implementation lowers first (requiring `Functor`), then folds. PureScript's `Foldable Coyoneda` only requires `Foldable f`. The module docs at lines 66-71 explain why.

`CoyonedaExplicit::fold_map` (line 326-336) does not require `F: Functor`, demonstrating the correct semantics are achievable with the visible intermediate type.

**Recommendation:** Accept as-is. The same limitation applies to `RcCoyonedaBrand` and `ArcCoyonedaBrand` `Foldable` implementations. Users who need `Foldable` without `Functor` should use `CoyonedaExplicit`.

---

### 2.9. Unsafe `Send`/`Sync` implementations in `ArcCoyoneda`

**Files:** `arc_coyoneda.rs` lines 106-118, 171-184
**Severity:** Medium (safety concern)

The `ArcCoyonedaBase` and `ArcCoyonedaMapLayer` types have manual `unsafe impl Send` and `unsafe impl Sync`.

For `ArcCoyonedaBase` (lines 106-118):

```rust
unsafe impl<'a, F, A: 'a> Send for ArcCoyonedaBase<'a, F, A>
where
    F: Kind_cdc7cd43dac7585f + 'a,
    <F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Send,
{}
unsafe impl<'a, F, A: 'a> Sync for ArcCoyonedaBase<'a, F, A>
where
    F: Kind_cdc7cd43dac7585f + 'a,
    <F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Sync,
{}
```

These are sound: the only field is `fa: F::Of<'a, A>`, so `Send`/`Sync` correctly depends on whether `fa` is `Send`/`Sync`.

For `ArcCoyonedaMapLayer` (lines 171-184):

```rust
unsafe impl<'a, F, B: 'a, A: 'a> Send for ArcCoyonedaMapLayer<'a, F, B, A>
where
    F: Kind_cdc7cd43dac7585f + 'a
{}
unsafe impl<'a, F, B: 'a, A: 'a> Sync for ArcCoyonedaMapLayer<'a, F, B, A>
where
    F: Kind_cdc7cd43dac7585f + 'a
{}
```

The SAFETY comment at line 164 says "Both fields are `Arc<dyn ... + Send + Sync>`, which are `Send + Sync`." This reasoning is correct: `Arc<T>` where `T: Send + Sync` is itself `Send + Sync`. The trait object `dyn ArcCoyonedaLowerRef<...>` has `Send + Sync` as supertraits (line 70), and the function is `dyn Fn(B) -> A + Send + Sync` (line 161). So both fields are indeed `Arc<dyn ... + Send + Sync>`.

However, the blanket `unsafe impl` does not constrain on `F`'s associated types being `Send + Sync`. This is fine because the `Send + Sync` safety comes from the `Arc` wrapping, not from `F` itself. The inner `Arc`'s contents are already constrained to be `Send + Sync` by the trait object bounds.

**Assessment:** The unsafe impls appear sound, but the reasoning depends on the trait object bounds being correct. Since `ArcCoyonedaLowerRef` requires `Send + Sync` (line 70) and the `func` field requires `Send + Sync` (line 161), the analysis holds. However, any future change to these trait bounds could silently introduce unsoundness.

**Approaches:**

A. **Add a compile-time assertion** that `ArcCoyonedaMapLayer` fields are `Send + Sync`.

- Trade-off: Provides a safety net against future regressions. Minimal runtime cost.

B. **Use a wrapper type** that encapsulates the `Arc<dyn ... + Send + Sync>` pattern, making the safety argument more local.

- Trade-off: Adds indirection but makes the safety proof trivially verifiable.

**Recommendation:** Add compile-time assertions (approach A) as a regression guard. The current impls are sound.

---

### 2.10. `RcCoyonedaMapLayer::lower_ref` clones the function `Rc` unnecessarily

**Files:** `rc_coyoneda.rs` lines 172-178

```rust
fn lower_ref(&self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
where
    F: Functor, {
    let lowered = self.inner.lower_ref();
    let func = self.func.clone();
    F::map(move |b| (*func)(b), lowered)
}
```

The `self.func.clone()` clones the `Rc` (a cheap refcount bump) and then moves the cloned `Rc` into the closure. The same pattern appears in `ArcCoyonedaMapLayer::lower_ref` at `arc_coyoneda.rs` line 216.

The reason for the clone is that `F::map` takes `impl Fn(B) -> A + 'a`, not a reference to a function. The closure needs to own the `Rc` to call the function. An alternative would be to borrow `&self.func` and create a closure that references it, but that would require the closure to have a shorter lifetime than `&self`, which may conflict with `F::map`'s `'a` bound.

**Assessment:** This is correct and necessary given the API constraints. The `Rc::clone` is O(1). No action needed.

---

### 2.11. `CoyonedaExplicit` has type explosion for deep chains

**Files:** `coyoneda_explicit.rs` lines 210-218
**Severity:** Low (documented)

Each `map` call produces a new `impl Fn` type via `compose`, creating deeply nested function types:

```rust
pub fn map<C: 'a>(
    self,
    f: impl Fn(A) -> C + 'a,
) -> CoyonedaExplicit<'a, F, B, C, impl Fn(B) -> C + 'a> {
    CoyonedaExplicit {
        fb: self.fb,
        func: compose(f, self.func),
        _phantom: PhantomData,
    }
}
```

For 30 chained maps, the type becomes `CoyonedaExplicit<..., impl Fn(B) -> Z + 'a>` where the `impl Fn` is a composition of 30 closures. This can cause:

1. Slow compilation (deeply nested types).
2. Large type names in error messages.
3. Potential monomorphization bloat.

The module docs at lines 17-18 recommend inserting `.boxed()` for chains deeper than 20-30 maps. The benchmark (coyoneda.rs lines 56-68) uses `.boxed()` in a loop, demonstrating the pattern.

**Approaches:**

A. **Accept as-is.** The documentation clearly advises using `.boxed()` for deep chains.

B. **Provide a `map_boxed` convenience method** that maps and boxes in one step, reducing boilerplate.

- Trade-off: Minor ergonomic improvement. Users still need to know when to box.

**Recommendation:** Approach B could improve ergonomics slightly, but is not urgent. The current documentation is sufficient.

---

### 2.12. No `Traversable` instance for any Coyoneda brand

**Files:** `coyoneda.rs` line 85-88, `coyoneda_explicit.rs` lines 413-426
**Severity:** Low-medium

`CoyonedaExplicit` has a `traverse` inherent method (line 413), but neither `CoyonedaBrand` nor `CoyonedaExplicitBrand` implements the `Traversable` trait. The `Coyoneda` module docs at line 85 explain:

> Not `Clone`. The inner trait object `Box<dyn CoyonedaInner>` is not `Clone`. This prevents implementing `Traversable` (which requires `Self::Of<'a, B>: Clone`).

For `RcCoyonedaBrand`, which is `Clone`, this blocker does not apply. A `Traversable` implementation could be added.

For `CoyonedaExplicitBrand`, the `traverse` inherent method already works. Adding a `Traversable` trait impl for the brand would require `BoxedCoyonedaExplicit` which is the brand's `Of` type to satisfy all `Traversable` constraints, including `Clone` on `F::Of<'a, C>`.

**Approaches:**

A. **Implement `Traversable` for `RcCoyonedaBrand`.** This would lower, traverse, and re-lift.

- Trade-off: Requires `F: Traversable + Functor`, clones the base value, and produces a `RcCoyoneda` in identity position.

B. **Implement `Traversable` for `CoyonedaExplicitBrand`.** This can delegate to the inherent `traverse` method.

- Trade-off: Needs `B: Clone` and `F::Of<'a, C>: Clone`.

**Recommendation:** Both are worth pursuing but not urgent. They extend HKT integration.

---

### 2.13. Benchmark does not cover `RcCoyoneda` or `ArcCoyoneda`

**Files:** `benches/benchmarks/coyoneda.rs`
**Severity:** Low

The benchmark only compares `Direct`, `Coyoneda`, and `CoyonedaExplicit`. `RcCoyoneda` and `ArcCoyoneda` are not benchmarked, so there is no quantitative evidence of their overhead (Rc/Arc allocation per map, refcount bumps, re-evaluation on each `lower_ref()`).

**Recommendation:** Add benchmark cases for `RcCoyoneda` and `ArcCoyoneda`, including:

- Single `lower_ref()` cost vs `Coyoneda::lower()`.
- Cost of multiple `lower_ref()` calls (measuring re-evaluation).
- `clone()` + `lower_ref()` patterns.

---

### 2.14. `ArcCoyonedaBrand` lacks `Functor` due to HKT trait design

**Files:** `arc_coyoneda.rs` lines 375-379
**Severity:** Medium (architectural limitation)

```rust
// Note: ArcCoyonedaBrand does NOT implement Functor. The HKT trait signatures
// lack Send + Sync bounds on closure parameters...
```

The `Functor::map` signature takes `impl Fn(A) -> B + 'a`, which lacks `Send + Sync` bounds. Since `ArcCoyoneda::map` requires `impl Fn(A) -> B + Send + Sync + 'a`, there is a mismatch: a `Functor::map` closure cannot be stored in an `ArcCoyonedaMapLayer`.

This is the same limitation as `SendThunkBrand`.

**Approaches:**

A. **Accept as-is.** Use `ArcCoyoneda` via inherent methods; use `RcCoyonedaBrand` for HKT polymorphism.

B. **Introduce a `SendFunctor` trait** with `Send + Sync` bounds on closures.

- Trade-off: Major API addition. Creates a parallel hierarchy (`SendFunctor`, `SendPointed`, `SendSemimonad`, etc.), duplicating much of the type class system.

C. **Add `Send + Sync` bounds to `Functor::map` everywhere.**

- Trade-off: Breaks non-Send closures everywhere. Not feasible.

D. **Use conditional compilation or a marker trait** to toggle the bound.

- Trade-off: Complex, confusing, and fragile.

**Recommendation:** Accept as-is (approach A). The `SendFunctor` hierarchy (approach B) is worth considering as a broader library evolution, but is a large design decision beyond Coyoneda.

---

### 2.15. `Coyoneda::map` consumes `self`, preventing chaining from borrows

**Files:** `coyoneda.rs` line 523-531
**Severity:** Low

```rust
pub fn map<B: 'a>(
    self,
    f: impl Fn(A) -> B + 'a,
) -> Coyoneda<'a, F, B> {
```

`map` takes `self` by value. This is correct for `Coyoneda` (which owns a `Box`), but means you cannot map over a borrowed `Coyoneda`. Users must move the value.

For `RcCoyoneda`, `map` also takes `self` by value (line 318-326), but since it is `Clone`, users can `.clone().map(f)`. However, `map` could instead take `&self` since the inner `Rc` can be cloned cheaply.

**Approaches:**

A. **Accept as-is.** `Coyoneda::map` must consume self (it wraps the inner `Box`). `RcCoyoneda::map` consuming self is consistent.

B. **Make `RcCoyoneda::map` take `&self`** and clone the inner `Rc` internally.

- Trade-off: Changes the API contract. Would mean `map` does not consume the original, which might be surprising since the original can still be used (it does not include the new map layer). This is actually the natural behavior for a persistent/shared structure.

**Recommendation:** Consider approach B for `RcCoyoneda` and `ArcCoyoneda`. Since they are already reference-counted and `Clone`, taking `&self` for `map` would align with their semantics as shareable structures. The current value can be used as-is (without the new layer), and the new `RcCoyoneda` includes the layer. This is analogous to how persistent data structures work.

---

### 2.16. No `Debug` implementation for any Coyoneda type

**Files:** All four files
**Severity:** Low

None of the Coyoneda types implement `Debug`. This makes debugging difficult, since printing a `Coyoneda` value shows nothing useful. The inner trait objects and stored functions are not `Debug`-able in general.

**Approaches:**

A. **Implement `Debug` manually** to show structural information (e.g., "Coyoneda { layers: 3 }").

- Trade-off: Limited usefulness since the actual values and functions cannot be shown. But better than nothing.

B. **Accept as-is.** Functions are not `Debug`, so there is inherently limited information to show.

**Recommendation:** Approach A would provide minimal debugging support. A simple `Debug` impl that shows the type name and layer count (or at least "Coyoneda(...)") would be helpful.

---

### 2.17. No conversion between `RcCoyoneda`/`ArcCoyoneda` and `Coyoneda`/`CoyonedaExplicit`

**Files:** All four files
**Severity:** Low

There are `From` conversions between `Coyoneda` and `CoyonedaExplicit` (lines 663-697 and 869-902 in `coyoneda_explicit.rs` and `coyoneda.rs`), but no conversions involving `RcCoyoneda` or `ArcCoyoneda`.

Useful conversions might include:

- `Coyoneda` -> `RcCoyoneda` (wrap in Rc, but requires Clone on base).
- `RcCoyoneda` -> `Coyoneda` (lower then re-lift, losing the Rc benefit).
- `RcCoyoneda` -> `ArcCoyoneda` (not possible without re-lifting due to Rc vs Arc).

**Recommendation:** Add `From<RcCoyoneda<'a, F, A>> for Coyoneda<'a, F, A>` (via lower_ref + lift) and `From<ArcCoyoneda<'a, F, A>> for Coyoneda<'a, F, A>`, both requiring `F: Functor`. These provide escape hatches from the Rc/Arc variants back to the consuming variant.

---

### 2.18. Inconsistent `document_module` annotation on `ArcCoyoneda`

**Files:** `arc_coyoneda.rs` line 42
**Severity:** Very low

`ArcCoyoneda` uses `#[fp_macros::document_module(no_validation)]` while the other three files use `#[fp_macros::document_module]`. This means documentation validation (checking for missing doc attributes) is skipped for `ArcCoyoneda`.

This is likely because the module was added more recently and validation was suppressed to avoid dealing with warnings during development. It should be enabled for consistency.

**Recommendation:** Enable validation by removing `no_validation` and addressing any warnings.

---

## 3. Summary of Recommendations

**High priority (correctness/safety):**

1. Add `stacker` support to `RcCoyoneda::lower_ref` and `ArcCoyoneda::lower_ref` (issue 2.3).
2. Add compile-time `Send + Sync` assertions for `ArcCoyonedaMapLayer` fields (issue 2.9).

**Medium priority (completeness):** 3. Implement missing type class instances (`Pointed`, `Lift`, `Semiapplicative`, `Semimonad`) for `RcCoyonedaBrand` (issue 2.5). 4. Add inherent methods (`pure`, `apply`, `bind`) to `ArcCoyoneda` (issue 2.5). 5. Add `collapse()` methods to `RcCoyoneda` and `ArcCoyoneda` (issue 2.3). 6. Add benchmark coverage for `RcCoyoneda` and `ArcCoyoneda` (issue 2.13).

**Low priority (ergonomics/polish):** 7. Consider making `RcCoyoneda::map` and `ArcCoyoneda::map` take `&self` (issue 2.15). 8. Add `From` conversions between Rc/Arc variants and `Coyoneda` (issue 2.17). 9. Add basic `Debug` implementations (issue 2.16). 10. Enable documentation validation on `ArcCoyoneda` (issue 2.18). 11. Add inline documentation warnings on `Coyoneda::map` about the lack of fusion (issue 2.1).
