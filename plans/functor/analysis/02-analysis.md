# Coyoneda Implementations: Flaws, Limitations, and Recommendations

## Overview

This document analyzes the two Coyoneda implementations in `fp-library`:

- `Coyoneda` (`fp-library/src/types/coyoneda.rs`): HKT-integrated free functor using layered trait objects.
- `CoyonedaExplicit` (`fp-library/src/types/coyoneda_explicit.rs`): Explicit intermediate type for zero-cost map fusion.

Each issue is assigned a severity:

- **Critical**: Correctness or soundness problem.
- **High**: Defeats the stated design goal or causes significant overhead.
- **Medium**: Ergonomic or performance issue with practical impact.
- **Low**: Minor limitation or missing feature.

---

## Issue 1: CoyonedaExplicit boxes every composed function, undermining "zero-cost" claim

**Severity: High**

**Location:** `fp-library/src/types/coyoneda_explicit.rs`, lines 96-97, 167-175.

**Problem:**

The module documentation (line 1) claims "zero-cost map fusion" and the comparison table (line 22) claims "0" heap allocations per map. However, every call to `map` allocates a new `Box<dyn Fn(B) -> A + 'a>`:

```rust
pub fn map<C: 'a>(
    self,
    f: impl Fn(A) -> C + 'a,
) -> CoyonedaExplicit<'a, F, B, C> {
    CoyonedaExplicit {
        fb: self.fb,
        func: Box::new(compose(f, self.func)),
    }
}
```

The `compose` call itself is zero-cost (returns an `impl Fn`), but wrapping the result in `Box::new(...)` performs a heap allocation and introduces dynamic dispatch. For k chained maps, there are k heap allocations (one per `map` call). The old box is dropped each time, so only one box is live at a time, but the allocation traffic is O(k).

Additionally, the `lift` method (line 434-439) boxes the identity function, which is another unnecessary allocation.

The documentation's claim of "0 heap allocations per map" in the comparison table is incorrect. The accurate comparison is:

| Property                 | Coyoneda               | CoyonedaExplicit      |
| ------------------------ | ---------------------- | --------------------- |
| Heap allocations per map | 2 boxes (inner + func) | 1 box (composed func) |
| F::map calls at lower    | k                      | 1                     |

**Approaches:**

A. **Store the function as an unboxed generic parameter.** Change the struct to:

```rust
pub struct CoyonedaExplicit<'a, F, B: 'a, A: 'a, K>
where
    F: Kind_cdc7cd43dac7585f + 'a,
    K: Fn(B) -> A + 'a,
{
    fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
    func: K,
}
```

Each `map` call composes via `compose(f, self.func)`, producing a nested `impl Fn` with no boxing. The type of `K` grows with each `map` (it becomes `Compose<F3, Compose<F2, Compose<F1, Identity>>>>`), but this is entirely on the stack with static dispatch.

**Trade-offs:**

- (+) Truly zero-cost: no heap allocation, no dynamic dispatch.
- (+) Matches the documented behavior.
- (-) The type parameter `K` is viral; it appears in every signature and every variable binding. Type inference helps at call sites, but storing a `CoyonedaExplicit` in a struct field requires naming the full composed type, which is impossible without `impl Trait` in type aliases (stabilized in Rust 1.87, expected 2025-05-15).
- (-) `lift` must return `CoyonedaExplicit<'a, F, A, A, fn(A) -> A>` (or use identity), which is a different concrete type from a mapped variant, complicating uniform handling.

B. **Use `Box<dyn FnOnce>` instead of `Box<dyn Fn>`.** Since `lower` consumes `self`, the accumulated function only needs to be called once. Using `FnOnce` allows more closures to be boxed (those that capture by move) and documents the single-use intent. This does not eliminate boxing but is more correct.

**Trade-offs:**

- (+) Semantically accurate; the function is only called once.
- (+) Allows `map` to accept `impl FnOnce` rather than `impl Fn`.
- (-) Still allocates per map.
- (-) `FnOnce` closures in a `Box<dyn FnOnce>` require `#[feature(unsized_fn_params)]` or the `Box<dyn FnOnce()>` calling convention, which is stable but means `(self.func)(x)` must become `Box::call_once(self.func, (x,))` or similar.

C. **Keep the current design but fix the documentation.** Update the comparison table and module docs to accurately state that each `map` allocates one box but only one `F::map` call is made at `lower` time.

**Trade-offs:**

- (+) No code change.
- (-) The "zero-cost" branding is the primary selling point; weakening it reduces the value proposition.

**Recommendation:** Approach A is ideal for the long term, as it delivers on the zero-cost promise. If `impl Trait` in type aliases is not yet available, Approach C should be applied immediately to fix the misleading documentation, with Approach A deferred until the language support is stable.

---

## Issue 2: CoyonedaExplicit uses `Fn` where `FnOnce` would suffice

**Severity: Medium**

**Location:** `fp-library/src/types/coyoneda_explicit.rs`, lines 96, 130-131, 167-169.

**Problem:**

The struct stores `Box<dyn Fn(B) -> A + 'a>` and `map` requires `impl Fn(A) -> C + 'a`. Since `lower` consumes `self` and the function is applied exactly once (via `F::map`), `FnOnce` is the correct bound. Requiring `Fn` unnecessarily restricts callers: closures that move out of captured variables cannot implement `Fn`.

For example, this is rejected:

```rust
let name = String::from("hello");
let coyo = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(1))
    .map(move |_| name);  // Error: closure is FnOnce, not Fn
```

The `Fn` requirement also propagates to `new`, which takes `impl Fn(B) -> A + 'a`.

Note that the underlying `Functor::map` trait method (line 121 in `functor.rs`) also requires `impl Fn(A) -> B + 'a` rather than `FnOnce`. This is a library-wide design choice documented in CLAUDE.md ("uncurried semantics with `impl Fn` for zero-cost abstractions"). If `Functor::map` requires `Fn`, then `CoyonedaExplicit::lower` requires the accumulated function to be `Fn` as well. So the `Fn` bound on the accumulated function is forced by the downstream consumer.

However, the intermediate `map` closures could still accept `FnOnce` if the accumulated function were stored differently (e.g., as a chain that composes `FnOnce` closures and calls them once in sequence). This is complex and may not be worth the effort given the library's `Fn`-everywhere convention.

**Approaches:**

A. **Accept the `Fn` requirement as a library-wide invariant.** Document the restriction clearly.

B. **Change `Functor::map` to accept `FnOnce`.** This is a sweeping change across the entire library and may conflict with the stated design principle of `impl Fn` for zero-cost reuse.

**Recommendation:** Approach A. The `Fn` bound is a deliberate library-wide design decision. Document the limitation in `CoyonedaExplicit`'s module docs so users are not surprised.

---

## Issue 3: Coyoneda does not achieve map fusion, making it equivalent to direct mapping

**Severity: High**

**Location:** `fp-library/src/types/coyoneda.rs`, lines 207-250 (`CoyonedaMapLayer::lower`), lines 394-402 (`Coyoneda::map`).

**Problem:**

The module documentation already acknowledges this (lines 38-44), but the architectural consequence deserves emphasis. Each `map` call creates a new `CoyonedaMapLayer` with two heap allocations (one `Box<dyn CoyonedaInner>` and one `Box<dyn Fn>`). At `lower` time, each layer calls `F::map` independently. For k chained maps on a `Vec` of n elements, this is O(k \* n) work with k intermediate `Vec` allocations, identical to chaining `F::map` directly.

The Coyoneda construction's theoretical value is map fusion (reducing k maps to 1). Without fusion, the only remaining benefit is the free `Functor` instance for non-`Functor` types. But `lower` requires `F: Functor`, so this benefit only applies to intermediate usage before lowering, which is a narrow use case.

**Approaches:**

A. **Implement an enum-based Coyoneda that stores a type-erased function chain.** Replace the layered trait-object approach with a single struct holding `Box<dyn Any>` for the hidden `F B` and a `Vec<Box<dyn FnOnce(Box<dyn Any>) -> Box<dyn Any>>>` for the function chain. At `lower` time, compose all functions, then apply once.

**Trade-offs:**

- (+) Achieves single-pass fusion.
- (-) Requires unsafe `Any` downcasting, which is fragile and loses type safety.
- (-) Each function in the chain still allocates.

B. **Use the `CoyonedaExplicit` as the internal representation when possible, and fall back to trait objects only when the intermediate type must be hidden.** Provide a `From<CoyonedaExplicit>` conversion (already exists as `into_coyoneda`).

**Trade-offs:**

- (+) Users get fusion when they can use `CoyonedaExplicit`.
- (-) Does not solve the problem for `Coyoneda` itself.

C. **Accept the limitation and document it as the primary recommendation to use `CoyonedaExplicit` for fusion.** Keep `Coyoneda` solely for the free `Functor` instance use case.

**Trade-offs:**

- (+) No unsafe code.
- (+) Clear API guidance.
- (-) `Coyoneda` remains a misleading name for users familiar with Haskell/PureScript where it implies fusion.

**Recommendation:** Approach C. The limitation is fundamental to Rust's type system (no rank-2 types, no generic methods on trait objects). Document prominently that `Coyoneda` does NOT fuse maps and direct users to `CoyonedaExplicit` for fusion.

---

## Issue 4: Coyoneda has stack overflow risk for deeply nested maps

**Severity: Medium**

**Location:** `fp-library/src/types/coyoneda.rs`, lines 244-249.

**Problem:**

`CoyonedaMapLayer::lower` is recursive: it calls `self.inner.lower()`, which may itself be another `CoyonedaMapLayer::lower`. For k chained maps, the call stack depth is O(k). The `many_chained_maps` test (line 672-678) only tests 100 maps; larger chains will overflow the stack. The module documentation mentions this risk at line 22.

`CoyonedaExplicit` does not have this problem because it composes functions into a single closure rather than nesting layers.

**Approaches:**

A. **Convert the recursive lowering to an iterative loop.** This requires a uniform representation for the inner layers (e.g., a `Vec` of boxed functions applied in sequence). This is essentially Approach A from Issue 3.

B. **Use a trampoline for the lowering step.** The library already has `Trampoline` for stack-safe recursion. However, integrating it here would require `'static` bounds and complicate the implementation significantly.

C. **Document the limitation and recommend `CoyonedaExplicit` for long chains.**

**Recommendation:** Approach C in the short term. The stack overflow requires thousands of chained maps, which is unusual in practice. For long chains, `CoyonedaExplicit` is the correct choice anyway.

---

## Issue 5: CoyonedaExplicit::fold_map requires B: Clone unnecessarily

**Severity: Medium**

**Location:** `fp-library/src/types/coyoneda_explicit.rs`, lines 281-291.

**Problem:**

The `fold_map` method has the bound `B: Clone`:

```rust
pub fn fold_map<FnBrand, M>(
    self,
    func: impl Fn(A) -> M + 'a,
) -> M
where
    B: Clone,
    M: Monoid + 'a,
    F: Foldable,
    FnBrand: CloneableFn + 'a, {
    F::fold_map::<FnBrand, B, M>(compose(func, self.func), self.fb)
}
```

The `B: Clone` bound comes from `Foldable::fold_map`'s signature (line 211 in `foldable.rs`), which requires `A: Clone`. Since `CoyonedaExplicit::fold_map` delegates to `F::fold_map` with element type `B`, it needs `B: Clone`.

This is a real limitation: the underlying `Foldable` trait requires `Clone` on the element type because `fold_right`'s default implementation (which `fold_map` may delegate to) clones elements. However, for `CoyonedaExplicit` specifically, the fold function is `compose(func, self.func)` which consumes each `B` by value and produces `M`. The `Clone` bound is needed only because the trait signature requires it, not because the operation itself needs it.

**Approaches:**

A. **Accept the bound.** It is inherited from the `Foldable` trait and cannot be removed without changing the trait.

B. **Provide a separate `fold_map_consuming` method that uses a custom fold implementation.** This would bypass `Foldable` and implement the fold directly, but then it would not work generically over any `F: Foldable`.

**Recommendation:** Approach A. The `Clone` bound is a library-wide constraint from `Foldable`. Fixing it requires changing `Foldable`, which is out of scope for Coyoneda.

---

## Issue 6: CoyonedaExplicit::apply and bind defeat map fusion

**Severity: Medium**

**Location:** `fp-library/src/types/coyoneda_explicit.rs`, lines 358-366 (`apply`), lines 395-402 (`bind`).

**Problem:**

Both `apply` and `bind` call `self.lower()` and then re-`lift` the result:

```rust
pub fn apply<FnBrand: CloneableFn + 'a, Bf: 'a, C: 'a>(
    ff: CoyonedaExplicit<'a, F, Bf, <FnBrand as CloneableFn>::Of<'a, A, C>>,
    fa: Self,
) -> CoyonedaExplicit<'a, F, C, C>
where
    A: Clone,
    F: Semiapplicative, {
    CoyonedaExplicit::lift(F::apply::<FnBrand, A, C>(ff.lower(), fa.lower()))
}
```

This forces a full `F::map` call on both `ff` and `fa` to lower their accumulated functions before delegating to `F::apply`. Any maps accumulated before `apply`/`bind` are applied eagerly, and the result is re-lifted with the identity function. Maps after `apply`/`bind` start a new fusion chain.

The documentation (lines 320-322, 371-373) notes this: "After the operation the fusion pipeline is reset." This is correct behavior, but it means that a chain like `.map(f).map(g).apply(ff).map(h).map(i)` will produce two `F::map` calls (one for the `f . g` portion at `apply` time, and one for the `h . i` portion at `lower` time), not one. Users who expect end-to-end fusion may be surprised.

**Approaches:**

A. **Accept the behavior and document it clearly.** The reset is inherent: `apply` and `bind` must materialize the underlying `F` to delegate to `F::apply`/`F::bind`.

B. **Do not provide `apply`/`bind` on `CoyonedaExplicit`.** If the type's purpose is map fusion, operations that defeat fusion are misleading. Users who need `apply`/`bind` should `lower` explicitly and work with `F` directly.

**Trade-offs for B:**

- (+) Clearer API contract: `CoyonedaExplicit` is purely for map fusion.
- (-) Less convenient; users must manually lower and re-lift.

**Recommendation:** Approach A. The methods are useful for convenience, and the documentation already explains the reset behavior. Add a note in each method's documentation that prior maps are materialized.

---

## Issue 7: Coyoneda lacks Semiapplicative, Semimonad, and other type class instances

**Severity: Low**

**Location:** `fp-library/src/types/coyoneda.rs`, lines 70-71.

**Problem:**

The module documentation lists missing instances: `Apply`, `Applicative`, `Bind`, `Monad`, `Traversable`, `Extend`, `Comonad`, `Eq`, `Ord`. The library's equivalents would be `Semiapplicative`, `Monad` (or `Semimonad`), `Traversable`, `Extend`, `Comonad`.

Key obstacles:

- `Semiapplicative` requires `Self::Of<'a, B>: Clone` for holding wrapped functions in containers. `Coyoneda` is not `Clone` (line 64).
- `Traversable` also requires `Clone`.
- `Semimonad` could be implemented by lowering and re-lifting, similar to how `CoyonedaExplicit::bind` works. This would require `F: Functor + Semimonad`.
- `Eq`/`Ord` cannot be implemented because the inner trait object has no way to compare values.

**Approaches:**

A. **Implement `Semimonad` for `CoyonedaBrand<F>` where `F: Functor + Semimonad`.** This is straightforward:

```rust
impl<F: Functor + Semimonad + 'static> Semimonad for CoyonedaBrand<F> {
    fn bind<'a, A: 'a, B: 'a>(
        fa: Coyoneda<'a, F, A>,
        f: impl Fn(A) -> Coyoneda<'a, F, B> + 'a,
    ) -> Coyoneda<'a, F, B> {
        Coyoneda::lift(F::bind(fa.lower(), move |a| f(a).lower()))
    }
}
```

B. **Make `Coyoneda` cloneable** by wrapping the inner in `Rc<dyn CoyonedaInner>` instead of `Box<dyn CoyonedaInner>`. This enables `Semiapplicative` and `Traversable`, but requires adding a `&self` lower method (or `Clone`-based sharing).

**Trade-offs for B:**

- (+) Unlocks several type class instances.
- (-) Changes the ownership model; `Rc` is `!Send`.
- (-) The inner trait would need `lower(&self)` instead of `lower(self: Box<Self>)`, requiring the stored `F B` to be cloneable or the lowering to use interior mutability.

**Recommendation:** Implement `Semimonad` (Approach A) as it is low-hanging fruit. Defer `Clone`-based instances (Approach B) unless there is a concrete use case.

---

## Issue 8: CoyonedaExplicit has no brand and cannot participate in HKT-generic code

**Severity: Low**

**Location:** `fp-library/src/types/coyoneda_explicit.rs`, line 19 (comparison table).

**Problem:**

`CoyonedaExplicit` has no brand type and does not implement any type class traits. It cannot be used in code generic over `Functor` or any other type class. The `into_coyoneda` method (line 314) provides an escape hatch, but it boxes the accumulated function (losing the fusion benefit if the user then chains more maps on the `Coyoneda`).

This is an inherent trade-off: the exposed intermediate type `B` prevents defining a `Kind` with the standard `type Of<'a, A: 'a>: 'a` signature because `B` would need to be part of the brand, but `B` changes with each `map`.

**Approaches:**

A. **Accept the limitation.** `CoyonedaExplicit` is a concrete utility type, not an HKT-integrated one.

B. **Provide an `into_coyoneda_fused` method** that lowers and re-lifts instead of wrapping:

```rust
pub fn into_coyoneda_fused(self) -> Coyoneda<'a, F, A>
where
    F: Functor,
{
    Coyoneda::lift(self.lower())
}
```

This produces a `Coyoneda` with no accumulated map layers (the fusion has already been applied), so subsequent maps on the `Coyoneda` start fresh.

**Recommendation:** Approach A with Approach B as a convenience. The `into_coyoneda` method already exists; adding `into_coyoneda_fused` gives users a choice between preserving the deferred function (current `into_coyoneda`) and materializing the fusion (new method).

---

## Issue 9: Neither implementation is Send or Sync

**Severity: Low**

**Location:**

- `fp-library/src/types/coyoneda.rs`, line 260-261.
- `fp-library/src/types/coyoneda_explicit.rs`, line 96.

**Problem:**

`Coyoneda` wraps `Box<dyn CoyonedaInner<'a, F, A> + 'a>`, which is `!Send + !Sync` because `dyn` trait objects default to `!Send`.

`CoyonedaExplicit` stores `Box<dyn Fn(B) -> A + 'a>`, also `!Send + !Sync`.

Neither type can be sent across threads. The library has a pattern for this (`SendThunk`, `ArcLazy`, `ArcFnBrand`), but no `SendCoyoneda` or `SendCoyonedaExplicit` exists.

**Approaches:**

A. **Create `Send` variants** (`SendCoyoneda`, `SendCoyonedaExplicit`) that use `Box<dyn ... + Send + 'a>`.

B. **Parameterize over `Send` using the pointer brand pattern.** The library already does this with `FnBrand<P>`.

C. **Defer until there is a concrete use case.**

**Recommendation:** Approach C. Thread-safe lazy computation is handled by `ArcLazy` and `SendThunk`. Coyoneda is primarily useful for map fusion on sequential pipelines.

---

## Issue 10: Coyoneda::new creates a redundant CoyonedaBase inside a CoyonedaMapLayer

**Severity: Low**

**Location:** `fp-library/src/types/coyoneda.rs`, lines 306-316.

**Problem:**

```rust
pub fn new<B: 'a>(
    f: impl Fn(B) -> A + 'a,
    fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
) -> Self {
    Coyoneda(Box::new(CoyonedaMapLayer {
        inner: Box::new(CoyonedaBase {
            fa: fb,
        }),
        func: Box::new(f),
    }))
}
```

This creates three heap allocations: the outer `CoyonedaMapLayer` box, the inner `CoyonedaBase` box, and the function box. The `CoyonedaBase` layer is unnecessary because it just stores `fb` and returns it unchanged in `lower`. A specialized `CoyonedaNew` layer could hold both `fb` and `f` in a single allocation, reducing to two boxes.

However, this is minor since `new` is called once per Coyoneda creation.

**Recommendation:** Not worth the code complexity. The extra allocation is a constant-factor overhead at construction time.

---

## Issue 11: Coyoneda::hoist lowers and re-lifts, losing accumulated maps' laziness

**Severity: Medium**

**Location:** `fp-library/src/types/coyoneda.rs`, lines 443-450.

**Problem:**

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

`hoist` eagerly lowers (applying all accumulated maps via k calls to `F::map`), transforms, then lifts into a fresh `Coyoneda<G, A>` with no deferred maps. This requires `F: Functor`, which PureScript's `hoistCoyoneda` does not.

`CoyonedaExplicit::hoist` (line 241-249) does not have this problem: it applies the natural transformation directly to the stored `F B` without lowering, and without requiring `F: Functor`.

This is a fundamental limitation of the layered trait-object design, as documented in the module (lines 54-57).

**Approaches:**

A. **Add a `hoist_inner` method to `CoyonedaInner`.** This would need to be generic over `G`, making it dyn-incompatible. Not feasible.

B. **Accept the limitation.** The workaround is to use `CoyonedaExplicit` for hoist operations.

**Recommendation:** Approach B. Document that `CoyonedaExplicit::hoist` is preferred when `F: Functor` is not available.

---

## Issue 12: CoyonedaExplicit::apply has a surprising A: Clone bound

**Severity: Low**

**Location:** `fp-library/src/types/coyoneda_explicit.rs`, line 363.

**Problem:**

The `apply` method requires `A: Clone`:

```rust
pub fn apply<FnBrand: CloneableFn + 'a, Bf: 'a, C: 'a>(
    ff: CoyonedaExplicit<'a, F, Bf, <FnBrand as CloneableFn>::Of<'a, A, C>>,
    fa: Self,
) -> CoyonedaExplicit<'a, F, C, C>
where
    A: Clone,
    F: Semiapplicative, {
```

The `A: Clone` bound comes from `Semiapplicative::apply` requiring element cloneability. This is documented in the library's `Semiapplicative` design. However, it means `apply` cannot be used when `A` is a move-only type (e.g., containing a non-cloneable handle).

**Recommendation:** Accept the limitation; it is inherited from `Semiapplicative`.

---

## Issue 13: No conversion from Coyoneda to CoyonedaExplicit

**Severity: Low**

**Location:** Both files.

**Problem:**

`CoyonedaExplicit` has `into_coyoneda` (line 314) for converting to `Coyoneda`. However, there is no reverse conversion: `Coyoneda` cannot be converted to `CoyonedaExplicit` because the intermediate type `B` is existentially hidden behind the trait object.

A limited conversion is possible: `Coyoneda` -> lower (requires `F: Functor`) -> `CoyonedaExplicit::lift`. But this materializes all accumulated maps, defeating any future fusion benefit.

**Recommendation:** Provide a convenience method on `Coyoneda`:

```rust
pub fn into_explicit(self) -> CoyonedaExplicit<'a, F, A, A>
where
    F: Functor,
{
    CoyonedaExplicit::lift(self.lower())
}
```

This makes the intent explicit and provides a clean migration path between the two types.

---

## Issue 14: Documentation inconsistencies in CoyonedaExplicit

**Severity: Low**

**Location:** `fp-library/src/types/coyoneda_explicit.rs`, various lines.

**Problems:**

1. Line 22 claims "0" heap allocations per map. As discussed in Issue 1, this is incorrect; each `map` allocates one `Box`.

2. Line 144, comment says "No heap allocation occurs for the composition itself (only the initial boxed function is retained, replaced by a new box wrapping the composed result)." The parenthetical contradicts the first clause: replacing with a new box IS a heap allocation.

3. The `Foldable without Functor` row (line 23) says "No" for `Coyoneda` and "Yes" for `CoyonedaExplicit`. For `Coyoneda`, the `Foldable` instance requires `F: Functor` (line 533 in coyoneda.rs). For `CoyonedaExplicit`, `fold_map` requires `F: Foldable` but not `F: Functor`. This row is accurate.

4. The `Hoist without Functor` row (line 24) is also accurate: `Coyoneda::hoist` requires `F: Functor`, `CoyonedaExplicit::hoist` does not.

**Recommendation:** Fix items 1 and 2 to accurately describe the allocation behavior.

---

## Summary Table

| Issue                                  | Severity | Category      | Recommendation                                                                    |
| -------------------------------------- | -------- | ------------- | --------------------------------------------------------------------------------- |
| 1. CoyonedaExplicit boxes every map    | High     | Performance   | Unbox the function type parameter (Approach A); fix docs immediately (Approach C) |
| 2. Fn instead of FnOnce                | Medium   | Ergonomics    | Accept as library-wide invariant; document                                        |
| 3. Coyoneda lacks map fusion           | High     | Performance   | Accept and document; recommend CoyonedaExplicit                                   |
| 4. Coyoneda stack overflow risk        | Medium   | Correctness   | Document; recommend CoyonedaExplicit for long chains                              |
| 5. fold_map requires B: Clone          | Medium   | Ergonomics    | Accept; inherited from Foldable                                                   |
| 6. apply/bind reset fusion             | Medium   | Ergonomics    | Accept and document the reset behavior                                            |
| 7. Missing type class instances        | Low      | Completeness  | Implement Semimonad for CoyonedaBrand                                             |
| 8. No HKT integration for Explicit     | Low      | Design        | Accept; provide into_coyoneda_fused convenience                                   |
| 9. Not Send/Sync                       | Low      | Thread safety | Defer until needed                                                                |
| 10. Redundant base in new              | Low      | Performance   | Not worth fixing                                                                  |
| 11. hoist lowers eagerly               | Medium   | Performance   | Accept; recommend CoyonedaExplicit::hoist                                         |
| 12. apply requires A: Clone            | Low      | Ergonomics    | Accept; inherited from Semiapplicative                                            |
| 13. No Coyoneda -> Explicit conversion | Low      | Ergonomics    | Add into_explicit convenience method                                              |
| 14. Documentation inaccuracies         | Low      | Documentation | Fix incorrect allocation claims                                                   |

---

## Priority Order for Fixes

1. Fix documentation inaccuracies in `CoyonedaExplicit` (Issues 1, 14) - immediate.
2. Implement truly unboxed `CoyonedaExplicit` with generic function parameter (Issue 1, Approach A) - when `impl Trait` in type aliases is stable.
3. Implement `Semimonad` for `CoyonedaBrand<F>` (Issue 7) - low effort.
4. Add `Coyoneda::into_explicit` convenience method (Issue 13) - low effort.
5. Add `CoyonedaExplicit::into_coyoneda_fused` method (Issue 8) - low effort.
6. Document stack overflow risk more prominently in `Coyoneda` (Issue 4) - low effort.
