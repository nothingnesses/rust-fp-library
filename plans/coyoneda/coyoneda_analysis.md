# Analysis: `coyoneda.rs`

Source: `fp-library/src/types/coyoneda.rs`
Based on: PureScript's `Data.Coyoneda` from `purescript-free`

## Design Overview

`Coyoneda` hides the intermediate type `B` behind a trait object (`Box<dyn CoyonedaInner>`), enabling full HKT integration via `CoyonedaBrand<F>`. Three internal struct types implement the `CoyonedaInner` trait:

- `CoyonedaBase<'a, F, A>` - created by `lift`, wraps `F A` directly.
- `CoyonedaNewLayer<'a, F, B, A>` - created by `new`, stores `F B` and `Box<dyn Fn(B) -> A>`.
- `CoyonedaMapLayer<'a, F, B, A>` - created by `map`, stores `Box<dyn CoyonedaInner<'a, F, B>>` and `Box<dyn Fn(B) -> A>`.

Type class implementations on `CoyonedaBrand<F>`: `Functor`, `Pointed` (requires `F: Pointed`), `Foldable` (requires `F: Functor + Foldable`), `Semimonad` (requires `F: Functor + Semimonad`).

## Comparison with PureScript

PureScript's `Coyoneda` uses `Exists` for existential quantification:

```purescript
data CoyonedaF f a i = CoyonedaF (i -> a) (f i)
newtype Coyoneda f a = Coyoneda (Exists (CoyonedaF f a))
```

The critical difference is that PureScript's `Exists` supports a rank-2 eliminator (`runExists`/`unCoyoneda`) that can "open" the existential to access the hidden type `i`. This enables:

1. **Map fusion:** `map f (Coyoneda e) = runExists (\(CoyonedaF k fi) -> coyoneda (f <<< k) fi) e` composes `f` with `k` eagerly, so `lower` calls `F::map` exactly once.
2. **Foldable without Functor:** `foldMap f = unCoyoneda \k -> foldMap (f <<< k)` composes the fold function with `k` and folds `F B` directly.
3. **Hoist without Functor:** `hoistCoyoneda nat (Coyoneda e) = runExists (\(CoyonedaF k fi) -> coyoneda k (nat fi)) e` transforms `F B` directly.

Rust's dyn-compatibility rules prevent generic methods on trait objects, so the hidden type `B` cannot be exposed to callers. This is the root cause of all the limitations below.

## Issues

### 1. No map fusion (k F::map calls instead of 1)

Each `map` wraps the previous value in a new `CoyonedaMapLayer`. At `lower` time, the chain unwinds recursively: each layer calls `self.inner.lower()` then applies its function via `F::map`. For k chained maps, this produces k calls to `F::map`.

This is the fundamental departure from PureScript's semantics. PureScript composes `f <<< k` eagerly inside the existential, producing a single composed function that is applied once at `lower` time.

**Root cause:** Composing a new function `Fn(A) -> C` with the stored function behind the trait object would require a method like `map_inner<C>(self: Box<Self>, f: impl Fn(A) -> C) -> Box<dyn CoyonedaInner<'a, F, C>>`, which is generic over `C` and therefore not dyn-compatible.

**Impact:** For eager containers like `Vec`, this means k separate traversals of the container instead of 1. The documentation correctly identifies this and recommends `CoyonedaExplicit` for fusion.

### 2. Double allocation per `map`

Each call to `Coyoneda::map` allocates two heap objects:

1. `Box::new(f)` - the mapping function.
2. `Box::new(CoyonedaMapLayer { inner, func })` - the layer itself.

Similarly, `Coyoneda::new` allocates two heap objects:

1. `Box::new(f)` - the mapping function.
2. `Box::new(CoyonedaNewLayer { fb, func })` - the layer.

This is unnecessary. By making `CoyonedaMapLayer` and `CoyonedaNewLayer` generic over the function type `Func`, the function can be stored inline in the layer struct, requiring only one allocation (the outer `Box<dyn CoyonedaInner>`).

Current:

```rust
struct CoyonedaMapLayer<'a, F, B: 'a, A: 'a> {
    inner: Box<dyn CoyonedaInner<'a, F, B> + 'a>,
    func: Box<dyn Fn(B) -> A + 'a>,  // separate allocation
}

// In Coyoneda::map:
Coyoneda(Box::new(CoyonedaMapLayer {     // allocation 1
    inner: self.0,
    func: Box::new(f),                   // allocation 2
}))
```

Proposed:

```rust
struct CoyonedaMapLayer<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a> {
    inner: Box<dyn CoyonedaInner<'a, F, B> + 'a>,
    func: Func,  // stored inline, no separate allocation
}

// In Coyoneda::map:
Coyoneda(Box::new(CoyonedaMapLayer {     // 1 allocation only
    inner: self.0,
    func: f,
}))
```

The `Func` type parameter is erased by the `Box<dyn CoyonedaInner>` wrapper, so it does not leak into the public API. The only cost is slightly more monomorphization, which is negligible.

### 3. `Foldable` requires `F: Functor`

The current `Foldable` implementation for `CoyonedaBrand<F>` requires `F: Functor + Foldable`:

```rust
impl<F: Functor + Foldable + 'static> Foldable for CoyonedaBrand<F> {
    fn fold_map<'a, FnBrand, A: 'a + Clone, M>(...) -> M {
        F::fold_map::<FnBrand, A, M>(func, fa.lower())  // lower requires Functor
    }
}
```

PureScript's `Foldable` for `Coyoneda` only requires `Foldable f`:

```purescript
instance foldableCoyoneda :: Foldable f => Foldable (Coyoneda f) where
  foldMap f = unCoyoneda \k -> foldMap (f <<< k)
```

The PureScript version composes the fold function `f: A -> M` with the accumulated mapping function `k: B -> A` to get `f <<< k: B -> M`, then folds `F B` directly. No `Functor` constraint needed.

**Root cause:** Adding a `fold_map_inner` method to `CoyonedaInner` would require it to be generic over the monoid type `M` and take a generic function parameter, both of which break dyn-compatibility.

**Impact:** You cannot use `CoyonedaBrand` to fold a non-Functor type constructor, defeating one of Coyoneda's key use cases (making non-Functors foldable). `CoyonedaExplicit` correctly handles this case.

### 4. `hoist` requires `F: Functor`

The current `hoist` method lowers, transforms, and re-lifts:

```rust
pub fn hoist<G>(...) -> Coyoneda<'a, G, A> where F: Functor {
    Coyoneda::lift(nat.transform(self.lower()))
}
```

PureScript's `hoistCoyoneda` applies the natural transformation directly to `F B`:

```purescript
hoistCoyoneda nat (Coyoneda e) = runExists (\(CoyonedaF k fi) -> coyoneda k (nat fi)) e
```

**Root cause:** A `hoist_inner<G>` method on the trait would be generic over the target brand `G`, breaking dyn-compatibility.

**Impact:** You cannot use `hoist` to change the underlying functor of a `Coyoneda` when `F` is not a `Functor`. Furthermore, `hoist` eagerly materializes the intermediate `F A` value (via lowering), which is wasteful for lazy functors. `CoyonedaExplicit::hoist` handles this correctly.

### 5. Stack overflow risk with deep nesting

At `lower` time, each `CoyonedaMapLayer` calls `self.inner.lower()` recursively. For k chained maps, the call stack grows to depth k. This can overflow the stack for large k values.

The `many_chained_maps` test uses k=100, which is fine. But k=10,000+ could overflow with default stack sizes.

**Possible mitigations:**

- Convert the recursive lowering into an iterative loop using unsafe pointer manipulation or a stack-allocated work list.
- Document the limitation and recommend `CoyonedaExplicit` (with `.boxed()`) for deep chains, as it does not have this issue.

### 6. Not `Clone`

`Box<dyn CoyonedaInner>` is not cloneable. This prevents:

- `Traversable` - requires `Self::Of<'a, B>: Clone` in the library's trait definition.
- `Semiapplicative` - requires cloning the structure.
- `Extend` / `Comonad` - PureScript's instances use the structure multiple times.

**Possible mitigation:** Provide an `Rc`-wrapped variant (`RcCoyoneda`) using `Rc<dyn CoyonedaInner>` instead of `Box<dyn CoyonedaInner>`. This would make the type `Clone` and enable additional type class instances.

### 7. Missing type class instances

PureScript provides many more instances than the current Rust implementation:

| Instance                | PureScript              | Rust `CoyonedaBrand`              | Notes                        |
| ----------------------- | ----------------------- | --------------------------------- | ---------------------------- |
| Functor                 | Yes                     | Yes                               |                              |
| Pointed (Applicative)   | Yes                     | Yes                               | Requires `F: Pointed`        |
| Foldable                | Yes (`F: Foldable`)     | Partial (`F: Functor + Foldable`) | Extra Functor constraint     |
| Semimonad (Bind)        | Yes (`F: Bind`)         | Yes (`F: Functor + Semimonad`)    | Extra Functor constraint     |
| Monad                   | Yes (`F: Monad`)        | Not yet                           | Requires Pointed + Semimonad |
| Semiapplicative (Apply) | Yes (`F: Apply`)        | No                                | Blocked by Clone             |
| Traversable             | Yes (`F: Traversable`)  | No                                | Blocked by Clone             |
| Eq / Ord                | Yes                     | No                                | Requires lowering            |
| Extend                  | Yes (`F: Extend`)       | No                                | Blocked by Clone             |
| Comonad                 | Yes (`F: Comonad`)      | No                                | Blocked by Clone             |
| Foldable1               | Yes (`F: Foldable1`)    | No                                |                              |
| Traversable1            | Yes (`F: Traversable1`) | No                                | Blocked by Clone             |
| Distributive            | Yes (`F: Distributive`) | No                                |                              |

Instances that could be added without Clone:

- `Eq` / `Ord` (via lowering, requires `F: Functor`)
- `Monad` (combine existing Pointed + Semimonad)
- `Foldable1` (with `F: Functor + Foldable1`, same limitation as Foldable)

Instances that require Clone (or an Rc variant):

- `Semiapplicative`, `Traversable`, `Extend`, `Comonad`, `Traversable1`

### 8. `Semimonad` requires `F: Functor`

The `Semimonad` implementation lowers the input before binding:

```rust
impl<F: Functor + Semimonad + 'static> Semimonad for CoyonedaBrand<F> {
    fn bind<'a, A: 'a, B: 'a>(ma, func) {
        Coyoneda::lift(F::bind(ma.lower(), move |a| func(a).lower()))
    }
}
```

PureScript's `Bind` for `Coyoneda` also lowers:

```purescript
instance bindCoyoneda :: Bind f => Bind (Coyoneda f) where
  bind (Coyoneda e) f = liftCoyoneda $
    runExists (\(CoyonedaF k fi) -> lowerCoyoneda <<< f <<< k =<< fi) e
```

PureScript's version opens the existential and applies `k` inside the bind, but still calls `lowerCoyoneda` on the result. So it needs `f: Functor` implicitly (via `lowerCoyoneda`). The Rust version's explicit `F: Functor` bound is honest about this.

However, PureScript's version is more efficient because it only calls `F::map` once (to apply `k` inside the bind), whereas the Rust version calls it k times (to fully lower before binding). The Rust `bind` could potentially be improved if the inner trait had a method to compose the bind callback with the accumulated function, but this would face the same dyn-compatibility issues as fold.

### 9. Documentation quality

The module-level documentation is excellent. It thoroughly explains:

- Performance characteristics
- Root cause of limitations (dyn-compatibility)
- Comparison with PureScript
- When to use which variant
- Code examples

Minor improvements:

- The limitation list could mention the stack overflow risk explicitly.
- The "Heap allocation per map" row in the comparison table with CoyonedaExplicit says "2 boxes" which would become "1 box" if issue #2 is fixed.

## Strengths

1. **Full HKT integration.** `CoyonedaBrand<F>` implements `Functor`, `Pointed`, `Foldable`, and `Semimonad`, enabling Coyoneda values to participate in generic code polymorphic over Functor.

2. **Correct type-safety.** The trait-object encoding correctly hides the intermediate type while preserving type safety. No unsafe code or `Any` downcasting.

3. **Ergonomic API.** `lift`, `map`, `lower`, `new`, `hoist` are clean and well-documented. The inherent methods mirror the PureScript API where possible.

4. **Good test coverage.** Tests cover identity, composition, chained maps, roundtrips, all type class instances, and edge cases (None, empty).

5. **Conversions with CoyonedaExplicit.** `From` impls enable moving between the two representations. `CoyonedaExplicit -> Coyoneda` is zero-cost; `Coyoneda -> CoyonedaExplicit` requires lowering (correctly documented).

6. **Lifetime flexibility.** `Coyoneda<'a, F, A>` supports arbitrary lifetimes, unlike types that require `'static`.
