# Coyoneda Implementation Analysis: Consolidated Summary

This document synthesizes findings from five independent analyses of the `Coyoneda` and
`CoyonedaExplicit` implementations in `fp-library/src/types/coyoneda.rs` and
`fp-library/src/types/coyoneda_explicit.rs`. The goal of these types is to improve functor
`map` performance via fusion. Each analysis independently read the source files and
identified issues; this summary consolidates their findings by consensus strength.

---

## Consensus Issues

Issues are ordered by how many analyses flagged them (consensus count) and severity.

### 1. CoyonedaExplicit's "zero-cost" claim is false: each `map` allocates a Box

**Consensus: 5/5 (all analyses)**
**Severity: High**

Every analysis identified this as a top issue. `CoyonedaExplicit::map` (line 167-175)
wraps the composed function in `Box::new(compose(f, self.func))`, performing one heap
allocation per `map` call. The documentation (line 1, line 22) claims "zero-cost map
fusion" and "0 heap allocations per map." Both claims are inaccurate.

The real benefit is genuine but narrower than documented: k chained maps produce only 1
call to `F::map` at `lower` time (vs. k calls for `Coyoneda`), reducing intermediate
container allocations for eager types like `Vec`. But the function composition chain
itself allocates once per `map`.

**Recommended approach (strong consensus):** Make the function a generic type parameter
instead of `Box<dyn Fn>`:

```rust
pub struct CoyonedaExplicit<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a> { ... }
```

This achieves truly zero-cost fusion with no boxing and no dynamic dispatch. The type
becomes unnameable after chaining (like Rust futures), but a `.boxed()` method can
provide type erasure when storage is needed. If this redesign is deferred, the
documentation must be corrected immediately (all 5 analyses agree on this).

**Trade-offs of generic approach:**

- (+) Truly zero-cost: no heap allocation, no dynamic dispatch, compiler can inline.
- (+) Automatically resolves `Send`/`Sync` (Issue 6) and identity boxing (Issue 9).
- (+) Substantially mitigates nested closure stack overflow (Issue 4).
- (-) Type grows with each `map`; unnameable without `impl Trait` in type aliases.
- (-) Leaks `Func` parameter into every signature.
- (-) Cannot store heterogeneous chains in collections without boxing.

---

### 2. Coyoneda performs no map fusion at all

**Consensus: 5/5**
**Severity: High**

All analyses confirmed that `Coyoneda` provides zero fusion benefit. Each `map` creates
a new `CoyonedaMapLayer` with 2 heap allocations (one `Box<dyn CoyonedaInner>`, one
`Box<dyn Fn>`). At `lower` time, each layer calls `F::map` independently. For k chained
maps on a `Vec` of n elements, this is O(k \* n) work with k intermediate allocations,
identical to (or worse than) calling `F::map` directly.

The root cause is fundamental: Rust's trait objects cannot have generic methods, so
function composition across the existential boundary is impossible.

**Recommended approach (strong consensus):** Accept the limitation. Keep `Coyoneda` for
HKT integration (it provides a `Functor` brand). Document prominently that it does NOT
fuse maps and direct users to `CoyonedaExplicit` for fusion. The `into_coyoneda` bridge
already exists for crossing from fusion-land to HKT-land.

---

### 3. `apply` and `bind` on CoyonedaExplicit are fusion barriers

**Consensus: 5/5**
**Severity: Medium**

Both methods call `lower()` on their inputs, forcing all accumulated maps to be applied
via `F::map`, then re-lift the result with the identity function. Maps before
`apply`/`bind` are fused into one `F::map` call (good), but the operation resets the
pipeline.

**Recommended approach (consensus):** Accept and document clearly. Fusion across monadic
boundaries is fundamentally impossible. Three analyses additionally recommend a
`map_then_bind` combinator:

```rust
pub fn map_then_bind<C: 'a>(
    self,
    f: impl Fn(A) -> <F as Kind>::Of<'a, C> + 'a,
) -> CoyonedaExplicit<'a, F, C, C>
where F: Semimonad {
    CoyonedaExplicit::lift(F::bind(self.fb, compose(f, self.func)))
}
```

This avoids the intermediate `F::map` before `F::bind` by composing the accumulated
function directly into the bind callback.

---

### 4. Coyoneda has stack overflow risk for deep chains

**Consensus: 5/5**
**Severity: Medium**

`CoyonedaMapLayer::lower` (line 244-249) is recursive with O(k) stack depth for k
chained maps. The test only covers 100 layers.

Two analyses also noted that `CoyonedaExplicit` has a related (though distinct) stack
overflow risk: the nested `Box<dyn Fn>` closures create a call chain of depth k when
the composed function is invoked, since dynamic dispatch prevents inlining. The
documentation table's "No" for stack overflow risk in `CoyonedaExplicit` is inaccurate
for very deep chains.

**Recommended approach:** Document both limitations. Direct users to `CoyonedaExplicit`
for deep chains (much higher threshold than `Coyoneda`). The generic `Func` approach
from Issue 1 would largely eliminate the `CoyonedaExplicit` variant of this problem
since the compiler can inline the composed closures.

---

### 5. Coyoneda's Foldable requires F: Functor (diverges from PureScript)

**Consensus: 5/5**
**Severity: Medium**

The `Foldable` instance for `CoyonedaBrand<F>` (line 533) requires `F: Functor +
Foldable + 'static` because it lowers before folding. PureScript only requires
`Foldable f` by composing the fold function with the accumulated mapping function
directly. The dyn-compatibility constraint on `CoyonedaInner` prevents adding a generic
`fold_map_inner` method.

`CoyonedaExplicit::fold_map` correctly avoids this by composing directly, requiring only
`F: Foldable`.

**Recommended approach (unanimous):** Accept the limitation. Direct users to
`CoyonedaExplicit::fold_map` when `F` is `Foldable` but not `Functor`.

---

### 6. Neither implementation is Send or Sync

**Consensus: 5/5**
**Severity: Medium**

Both types use `Box<dyn Trait>` without `Send` bounds. The library has an established
pattern for this (`Thunk`/`SendThunk`, `RcLazy`/`ArcLazy`).

**Recommended approach:** If the generic `Func` approach (Issue 1) is adopted,
`Send`/`Sync` is derived automatically when the closure and stored value are `Send`.
Otherwise, create `SendCoyonedaExplicit` with `Box<dyn Fn(...) + Send + 'a>` following
the existing library pattern.

---

### 7. Missing type class instances (especially Semimonad)

**Consensus: 5/5**
**Severity: Medium**

All analyses noted that `CoyonedaBrand<F>` only implements `Functor`, `Pointed`, and
`Foldable`. `Semimonad` is the most-requested missing instance (flagged by all 5) as it
is straightforward to implement via lower-bind-relift:

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

`Semiapplicative` and `Traversable` are blocked by `Clone` requirements on
`Box<dyn CoyonedaInner>`. An `Rc`-wrapped variant could enable these but adds overhead.

**Recommended approach (consensus):** Implement `Semimonad` now (low-hanging fruit).
Defer `Semiapplicative`/`Traversable` until there is a concrete use case or an
`Rc`-wrapped variant is designed.

---

### 8. Coyoneda::hoist requires F: Functor (diverges from PureScript)

**Consensus: 4/5**
**Severity: Medium**

`hoist` (line 443-450) lowers before transforming, requiring `F: Functor`.
`CoyonedaExplicit::hoist` applies the natural transformation directly to the stored
`fb` without lowering or requiring `Functor`.

**Recommended approach:** Accept the limitation. Point users to
`CoyonedaExplicit::hoist` when `F` is not a `Functor`.

---

### 9. CoyonedaExplicit uses `Fn` where `FnOnce` would suffice

**Consensus: 5/5**
**Severity: Low**

The stored function is `Box<dyn Fn>` and `map` takes `impl Fn`. Since `lower` consumes
`self`, the function is conceptually called once (per element). Using `Fn` prevents
closures that move out of captured variables.

**Recommended approach (unanimous):** Accept as an inherent constraint of
`Functor::map`'s `impl Fn` signature. The library's `Fn`-everywhere convention is a
deliberate design choice for multi-element containers. Document the limitation.

---

### 10. CoyonedaExplicit has no HKT brand

**Consensus: 5/5**
**Severity: Low**

The exposed intermediate type `B` prevents defining a standard `Kind` mapping. One
analysis proposed `CoyonedaExplicitBrand<F, B>` where
`Of<'a, A> = CoyonedaExplicit<'a, F, B, A>`, noting that `B` stays fixed while `A`
varies. The other four concluded this is a fundamental trade-off.

**Recommended approach:** Accept as a fundamental tension between type-level fusion and
HKT integration. The `into_coyoneda` bridge is the correct design pattern. The
`CoyonedaExplicitBrand<F, B>` proposal merits further investigation.

---

### 11. fold_map requires B: Clone

**Consensus: 5/5**
**Severity: Low**

The `B: Clone` bound on `CoyonedaExplicit::fold_map` is inherited from
`Foldable::fold_map`'s signature, not from any intrinsic need. The composed function
consumes `B` by value and produces `M`; cloning is not actually needed.

**Recommended approach (unanimous):** Accept as an upstream `Foldable` trait limitation.
Fixing it requires redesigning `Foldable`, which is out of scope.

---

### 12. No benchmarks validating fusion claims

**Consensus: 3/5**
**Severity: Medium**

Three analyses explicitly called out the absence of benchmarks comparing:

- Direct chained `F::map` calls
- `Coyoneda` with k maps then `lower`
- `CoyonedaExplicit` with k maps then `lower`
- Manual `compose` then single `F::map`

**Recommended approach:** Add Criterion benchmarks with varying container sizes and
chain depths. These would either validate or refute the performance claims and inform
whether Issue 1's boxing overhead matters in practice.

---

### 13. Coyoneda::new creates an unnecessary extra layer

**Consensus: 4/5**
**Severity: Low**

`new` allocates 3 boxes (outer `CoyonedaMapLayer`, inner `CoyonedaBase`, function).
A combined `CoyonedaSingle` struct holding both `fb` and `func` would reduce this to 2.

**Recommended approach:** Implement the optimization (minor, no downsides) or accept
the one-time constant-factor overhead.

---

### 14. into_coyoneda is a fusion boundary (underdocumented)

**Consensus: 4/5**
**Severity: Low**

After `into_coyoneda`, further `map` calls on the resulting `Coyoneda` do not fuse with
the previously accumulated function. This is correct behavior but should be documented.

**Recommended approach:** Add documentation noting that `into_coyoneda` should be the
last step before passing to HKT-generic code.

---

### 15. No Coyoneda -> CoyonedaExplicit conversion

**Consensus: 3/5**
**Severity: Low**

`into_coyoneda` goes one way. The reverse requires lowering (needs `F: Functor`), which
loses deferred computation. A convenience method would be useful:

```rust
pub fn into_explicit(self) -> CoyonedaExplicit<'a, F, A, A> where F: Functor {
    CoyonedaExplicit::lift(self.lower())
}
```

---

## Unique/Notable Findings

A few findings appeared in only one or two analyses but are worth noting:

- **Analysis 01:** `compose` creates deeply nested closures that inhibit inlining across
  `dyn Fn` boundaries, meaning the composed function pays an indirect call per
  intermediate step per element. This is an additional performance concern beyond the
  allocation overhead.

- **Analysis 01:** `CoyonedaExplicit`'s stack overflow documentation claim ("No") is
  inaccurate; nested `Box<dyn Fn>` closures create O(k) stack depth when invoked.

- **Analysis 02:** `into_coyoneda` may double-box the function because `Coyoneda::new`
  takes `impl Fn` and `self.func` is already a `Box<dyn Fn>`, potentially wrapping
  it in another `Box<dyn Fn>`. A constructor accepting `Box<dyn Fn>` directly would
  avoid this.

- **Analysis 03:** The `apply` method's complex type signature
  (`<FnBrand as CloneableFn>::Of<'a, A, C>`) could be improved with `apply_rc`/`apply_arc`
  convenience methods.

- **Analysis 04:** `CoyonedaExplicitBrand<F, B>` is feasible and would enable HKT
  integration, since `B` stays fixed while only `A` varies through `map`.

---

## Priority Recommendations

### Immediate (documentation fixes, no code changes)

1. Correct the "zero-cost" and "0 heap allocations per map" claims in
   `CoyonedaExplicit` documentation.
2. Fix the "No stack overflow risk" claim for `CoyonedaExplicit` to "Partial."
3. Add prominent guidance directing users to `CoyonedaExplicit` for fusion and to
   `Coyoneda` only for HKT integration.
4. Document fusion barrier behavior of `apply`, `bind`, and `into_coyoneda`.

### Short-term (low-effort code changes)

5. Implement `Semimonad` for `CoyonedaBrand<F>` (all 5 analyses agree; straightforward).
6. Add `Coyoneda::into_explicit` convenience method.
7. Add Criterion benchmarks comparing the four approaches to chained maps.
8. Add `CoyonedaNewLayer` to eliminate the extra allocation in `Coyoneda::new`.

### Medium-term (design work required)

9. Redesign `CoyonedaExplicit` with a generic `Func` type parameter for truly zero-cost
   fusion. Provide a `.boxed()` method for type erasure when needed.
10. Add a `map_then_bind` combinator to avoid the intermediate `F::map` before `F::bind`.
11. Investigate `CoyonedaExplicitBrand<F, B>` for HKT integration.

### Long-term / deferred

12. `Send` variants (resolved automatically by the generic `Func` approach).
13. `Rc`-wrapped `Coyoneda` for `Semiapplicative`/`Traversable` instances.
14. `Foldable` trait redesign to remove the `Clone` bound.

---

## Analysis Agreement Matrix

| Issue                            | 01  | 02  | 03  | 04  | 05  |
| -------------------------------- | --- | --- | --- | --- | --- |
| Boxing per map (misleading docs) | Yes | Yes | Yes | Yes | Yes |
| Coyoneda lacks fusion            | Yes | Yes | Yes | Yes | Yes |
| apply/bind fusion barriers       | Yes | Yes | Yes | Yes | Yes |
| Stack overflow (Coyoneda)        | Yes | Yes | Yes | Yes | Yes |
| Foldable requires Functor        | Yes | Yes | Yes | Yes | Yes |
| Not Send/Sync                    | Yes | Yes | Yes | Yes | Yes |
| Missing Semimonad                | Yes | Yes | Yes | Yes | Yes |
| Fn vs FnOnce                     | Yes | Yes | Yes | Yes | Yes |
| No HKT for Explicit              | Yes | Yes | Yes | Yes | Yes |
| fold_map B: Clone                | Yes | Yes | Yes | Yes | Yes |
| hoist requires Functor           | Yes | Yes | Yes | Yes | Yes |
| No benchmarks                    | --  | Yes | Yes | Yes | --  |
| Coyoneda::new extra layer        | Yes | Yes | --  | --  | Yes |
| into_coyoneda fusion boundary    | Yes | Yes | --  | Yes | Yes |
| Stack overflow (Explicit)        | Yes | --  | --  | --  | Yes |
| Generic Func recommended         | Yes | Yes | Yes | Yes | Yes |
