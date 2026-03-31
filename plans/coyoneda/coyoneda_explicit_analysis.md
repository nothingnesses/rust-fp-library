# Analysis: `coyoneda_explicit.rs`

Source: `fp-library/src/types/coyoneda_explicit.rs`
Based on: PureScript's `Data.Coyoneda` (same origin as `coyoneda.rs`)

## Design Overview

`CoyonedaExplicit` exposes the intermediate type `B` as a type parameter instead of hiding it behind a trait object. This enables compile-time function composition (map fusion) at the cost of limited HKT integration.

```rust
pub struct CoyonedaExplicit<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a = Box<dyn Fn(B) -> A + 'a>>
```

Key components:

- `fb: F::Of<'a, B>` - the underlying functor value.
- `func: Func` - the accumulated function from `B` to `A`.
- Default `Func = Box<dyn Fn(B) -> A + 'a>` for use in struct fields and collections.

`BoxedCoyonedaExplicit<'a, F, B, A>` is the type alias for the boxed variant.

Type class implementations on `CoyonedaExplicitBrand<F, B>`: `Functor`, `Foldable` (requires only `F: Foldable`, not `F: Functor`).

## Comparison with PureScript

`CoyonedaExplicit` is closer to PureScript's `Coyoneda` semantics than `Coyoneda` is, in the following ways:

1. **Map fusion.** `map` composes `f` with the accumulated function via `compose(f, self.func)`, producing a single composed function. At `lower` time, one call to `F::map` applies it. This matches PureScript's `map f (Coyoneda e) = runExists (\(CoyonedaF k fi) -> coyoneda (f <<< k) fi) e`.

2. **Foldable without Functor.** `fold_map` composes the fold function with the accumulated function and folds `F B` directly. Only requires `F: Foldable`. This matches PureScript's `foldMap f = unCoyoneda \k -> foldMap (f <<< k)`.

3. **Hoist without Functor.** `hoist` applies the natural transformation directly to `F B`. No lowering needed. This matches PureScript's `hoistCoyoneda`.

The trade-off is that `B` is visible as a type parameter, which prevents the rank-2 abstraction that PureScript achieves via `Exists`.

## Issues

### 1. `CoyonedaExplicitBrand<F, B>` has a fixed intermediate type

The brand `CoyonedaExplicitBrand<F, B>` includes `B` as a type parameter. This means:

- A brand instance is specific to a particular intermediate type.
- `Pointed` cannot be implemented on the brand, because `pure(a)` would need to produce `BoxedCoyonedaExplicit<'a, F, B, A>` with `B = A`, but `B` is fixed by the brand.
- `Semimonad` cannot be implemented on the brand, because `bind` changes the underlying structure.
- `Semiapplicative` cannot be implemented on the brand, for similar reasons.

This severely limits the brand's usefulness for HKT-generic programming. Code generic over `Functor` can use it, but code generic over `Pointed`, `Monad`, etc. cannot.

The brand docs correctly note this limitation. This is an inherent consequence of exposing `B`.

### 2. `B: 'static` requirement on the brand

```rust
impl<F: Kind_cdc7cd43dac7585f + 'static, B: 'static> for CoyonedaExplicitBrand<F, B>
```

The `B: 'static` bound is required because the `Kind` trait's `Of<'a, A>` introduces its own lifetime `'a`, so `B` must outlive all possible `'a`. Unlike `CoyonedaBrand<F>` (where only the brand `F` must be `'static`, and the hidden `B` can be any `B: 'a`), `CoyonedaExplicitBrand` requires the actual data type `B` to be `'static`.

This prevents using the brand with borrowed intermediate types (e.g., `CoyonedaExplicitBrand<VecBrand, &str>` is not valid). In practice this is rarely an issue since most types used in functorial positions are owned, but it is a theoretical limitation compared to `CoyonedaBrand`.

### 3. `bind` is strictly worse than `flat_map`

Two monadic binding operations are provided:

**`bind`:** Requires `F: Functor + Semimonad`. Calls `self.lower()` (which calls `F::map` to apply accumulated functions), then `F::bind`, then `f(a).lower()`. The callback must return a `CoyonedaExplicit` in identity position (`B = C`).

**`flat_map`:** Requires only `F: Semimonad`. Composes the accumulated function with the bind callback directly: `F::bind(self.fb, move |b| f(func(b)))`. No intermediate lowering, no `F::map` call, no `F: Functor` constraint.

`flat_map` is strictly better:

- Weaker constraints (`Semimonad` vs `Functor + Semimonad`).
- More efficient (no intermediate materialization via `lower`).
- Simpler callback signature (`Fn(A) -> F::Of<'a, C>` vs `Fn(A) -> CoyonedaExplicit<..., FuncOut>`).
- Achieves the same composition that PureScript's `unCoyoneda` enables.

`bind` exists only for API symmetry with `Coyoneda`, but its restrictive signature and extra constraints make it less useful.

### 4. `apply` signature is complex and forms a fusion barrier

```rust
pub fn apply<FnBrand, Bf, C, FuncF>(
    ff: CoyonedaExplicit<'a, F, Bf, <FnBrand as CloneableFn>::Of<'a, A, C>, FuncF>,
    fa: Self,
) -> CoyonedaExplicit<'a, F, C, C, fn(C) -> C>
where A: Clone, F: Semiapplicative
```

Issues:

- Four type parameters on a static method, two of which (`Bf`, `FuncF`) are internal to the function container.
- The result resets to identity position (`B = C`), discarding the fusion pipeline.
- Requires lowering both arguments, materializing all accumulated maps.
- Requires `A: Clone` (inherited from `Semiapplicative`).

This is inherent to the design since `apply` fundamentally requires two structures to interact, but the ergonomics could be improved. A more PureScript-like approach would lower only when necessary.

### 5. `fold_map` requires `B: Clone`

```rust
pub fn fold_map<FnBrand, M>(self, func: impl Fn(A) -> M + 'a) -> M
where B: Clone, M: Monoid + 'a, F: Foldable, FnBrand: CloneableFn + 'a
```

The `B: Clone` bound comes from the library's `Foldable` trait requiring `A: Clone` in `fold_map`. Since `CoyonedaExplicit` folds `F B` (not `F A`), the clone bound falls on `B`.

PureScript's `Foldable` has no `Clone` equivalent. This constraint prevents folding `CoyonedaExplicit` values with non-Clone intermediate types. The `B: Clone` bound is documented in the brand's `Foldable` impl (`B: Clone + 'static`).

This is a limitation of the library's `Foldable` design, not specific to `CoyonedaExplicit`.

### 6. Compile-time cost of deep `map` chains

Each `.map(f)` call produces a new `CoyonedaExplicit` with function type `impl Fn(B) -> C` wrapping the previous function type. After k maps, the function type is k levels deep. This can cause:

- Long compile times for chains deeper than ~20-30 maps.
- Large type names in error messages.

The documentation correctly recommends inserting `.boxed()` to bound type complexity. The `.boxed()` method erases the function type to `Box<dyn Fn(B) -> A>`, capping the type depth.

This is an inherent trade-off of the zero-cost approach: compile-time cost is traded for runtime cost.

### 7. `boxed()` always allocates even when already boxed

Calling `.boxed()` on a `BoxedCoyonedaExplicit` (where `Func` is already `Box<dyn Fn(B) -> A>`) will allocate a new box wrapping the existing box. There is no check or specialization for the already-boxed case.

This is a minor issue since users typically know whether their value is already boxed.

### 8. The brand's `Functor` re-boxes on every `map`

```rust
impl<F, B> Functor for CoyonedaExplicitBrand<F, B> {
    fn map<'a, A: 'a, C: 'a>(func, fa) {
        fa.map(func).boxed()  // allocates a new Box
    }
}
```

When used through the brand (e.g., `map::<CoyonedaExplicitBrand<VecBrand, i32>, _, _>`), each `map` allocates a box. This negates the zero-allocation benefit of direct `.map()` calls.

This is inherent to the brand design: `CoyonedaExplicitBrand` maps to `BoxedCoyonedaExplicit`, which needs a box. But the composed function replaces the old one (the old box is dropped), so there is still only one box at any point, unlike `CoyonedaBrand` which accumulates boxes.

The key advantage is preserved: single-pass fusion at `lower` time. Even with boxing per map through the brand, only one `F::map` call is made at the end.

### 9. Missing type class instances on the brand

| Instance        | PureScript          | `CoyonedaExplicitBrand<F, B>`   | Notes                                                |
| --------------- | ------------------- | ------------------------------- | ---------------------------------------------------- |
| Functor         | Yes                 | Yes                             |                                                      |
| Foldable        | Yes (`F: Foldable`) | Yes (`F: Foldable`, `B: Clone`) | Extra Clone constraint                               |
| Pointed         | Yes                 | No                              | `B` is fixed; cannot unify `B = A` for arbitrary `A` |
| Semimonad       | Yes                 | No                              | `bind` changes intermediate type                     |
| Semiapplicative | Yes                 | No                              | Same reason                                          |
| Traversable     | Yes                 | No                              | Would require `BoxedCoyonedaExplicit: Clone`         |

The Functor + Foldable coverage is the expected maximum for a brand with a fixed intermediate type.

### 10. No `FoldableWithIndex` instance

`CoyonedaExplicit` preserves the structure of `F B`, so it could support indexed folding by composing the index-aware fold function with the accumulated mapping function. This is not implemented but would follow the same pattern as `fold_map`.

### 11. No `Traversable` method

PureScript provides:

```purescript
instance traversableCoyoneda :: Traversable f => Traversable (Coyoneda f) where
  traverse f = unCoyoneda \k -> map liftCoyoneda <<< traverse (f <<< k)
```

A `traverse` inherent method on `CoyonedaExplicit` is feasible: compose the traversal function with the accumulated function, traverse `F B`, then re-lift the result. This would require `F: Traversable` (not `F: Functor`) and would preserve the single-pass property.

## Strengths

1. **True map fusion.** Each `map` composes at compile time; `lower` calls `F::map` exactly once. This matches PureScript's semantics and provides zero-cost abstraction for the common case.

2. **Foldable without Functor.** The `fold_map` method and `Foldable` brand impl only require `F: Foldable`. This is correct and matches PureScript, unlike `CoyonedaBrand`.

3. **Hoist without Functor.** The `hoist` method applies the natural transformation directly to `F B`. No lowering required. Correct and matches PureScript.

4. **`flat_map` composes through the accumulated function.** Only requires `F: Semimonad`, not `F: Functor`. This achieves the same composition that PureScript's `unCoyoneda` enables for bind.

5. **Send/Sync derived automatically.** No separate `SendCoyonedaExplicit` type is needed. The `.boxed_send()` method provides explicit Send boxing when needed.

6. **Ergonomic `.boxed()` escape hatch.** Clear, well-documented mechanism for when a uniform type is needed.

7. **Bidirectional conversion with `Coyoneda`.** `From` impls enable moving between representations. `CoyonedaExplicit -> Coyoneda` is zero-cost (via `Coyoneda::new`); `Coyoneda -> CoyonedaExplicit` requires lowering (explicit cost).

8. **Excellent documentation.** The module docs include a clear comparison table, trade-off analysis, usage guidance, and examples.

9. **Comprehensive test coverage.** Tests cover all methods, type class instances, conversions, edge cases, Send properties, and law verification.
