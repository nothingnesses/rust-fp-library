# Coyoneda Design Document

The free functor for fp-library's HKT/Brand system, enabling automatic map fusion.

Date: 2026-03-30

---

## Table of Contents

1. [Motivation](#motivation)
2. [What is Coyoneda](#what-is-coyoneda)
3. [Why not Yoneda](#why-not-yoneda)
4. [Encoding comparison](#encoding-comparison)
5. [Chosen encoding: trait-object](#chosen-encoding-trait-object)
6. [Detailed design](#detailed-design)
7. [Type class support](#type-class-support)
8. [Future extensions](#future-extensions)
9. [Open questions](#open-questions)

---

## Motivation

Chaining `Functor::map` on eager brands like `VecBrand` produces intermediate collections:

```rust
// 3 full traversals, 3 allocations, 2 intermediate Vecs
map::<VecBrand, _, _>(h, map::<VecBrand, _, _>(g, map::<VecBrand, _, _>(f, v)))
```

Users can manually compose functions to get a single pass:

```rust
// 1 traversal, 1 allocation
map::<VecBrand, _, _>(compose(h, compose(g, f)), v)
```

Coyoneda automates this: it accumulates `map` calls as function composition and applies them
in a single `Brand::map` when the user calls `lower`. This is the canonical functional
programming solution to the map fusion problem, known as the free functor.

See `plans/functor/functor-map-performance.md` for the full performance analysis.

---

## What is Coyoneda

Coyoneda is the free functor. For any type constructor `F`, `Coyoneda F` is a `Functor`,
even if `F` itself is not. Conceptually:

```
Coyoneda F A = exists B. (B -> A, F B)
```

It stores an original value `F B` alongside an accumulated function `B -> A`. The existential
type `B` is hidden; only `A` is visible to the outside world.

Operations:

- `lift(fa: F A) -> Coyoneda F A`: wraps with the identity function.
- `map(f: A -> C, coyo) -> Coyoneda F C`: composes `f` onto the accumulated function. Does
  not touch the inner `F B`.
- `lower(coyo) -> F A` (requires `F: Functor`): applies the fully composed function via a
  single `F::map` call.

After k chained maps, `lower` calls `F::map` exactly once with the composed function
`f_k . f_{k-1} . ... . f_1`. For `VecBrand`, this means one traversal instead of k.

### PureScript reference

From `purescript-free/src/Data/Coyoneda.purs`:

```purescript
data CoyonedaF f a i = CoyonedaF (i -> a) (f i)
newtype Coyoneda f a = Coyoneda (Exists (CoyonedaF f a))
```

PureScript uses `Exists` for existential quantification. In Rust, the equivalent is a trait
object that hides the inner type parameter.

---

## Why not Yoneda

Yoneda is the dual: `Yoneda F A = forall B. (A -> B) -> F B`. It also makes `map` free
(function composition), but `lift` requires `F: Functor` while `lower` does not.

Yoneda requires a rank-2 type (`forall B` as a struct field), which Rust cannot express:

- `dyn Trait` does not support generic methods.
- Closures cannot be polymorphic over return types.

Coyoneda uses an existential type (`exists B`), which Rust encodes naturally via trait objects.
The performance benefit is identical: O(1) per map, single `F::map` at materialization.

---

## Encoding Comparison

Five encodings were investigated. Three were eliminated.

### Eliminated

**Closure-based** (`Box<dyn FnOnce() -> F<A>>`): each `map` wraps a closure that calls
`Brand::map` internally. When `lower` unwinds the closures, `Brand::map` is called N times
(once per map). This defeats the entire purpose; it is operationally equivalent to chaining
`Thunk::map` with extra `Brand::map` overhead. No fusion occurs.

**Box\<dyn Any\>-based** (like `Free`): requires `'static` on all types because `Any: 'static`.
This prevents implementing the HKT `Kind` trait (which requires lifetime polymorphism
`Of<'a, A: 'a>: 'a`). Same limitation that prevents `Free` from participating in HKT.

**CPS/continuation-style**: collapses to the closure-based encoding when Rust forces the return
type to be fixed. Rank-2 polymorphism for closures is not available.

### Viable

**Trait-object** (recommended primary): stores `F<B>` and `B -> A` behind a `dyn` trait that
hides `B`. Each `map` composes the function; `lower` calls `Brand::map` once. Supports
lifetime polymorphism. Can implement Foldable without Functor.

**Generic B parameter** (recommended utility): exposes `B` in the type signature. Zero-cost
(no boxing). Cannot participate in HKT (extra type parameter). Suitable as a `FunctorPipeline`
builder API for performance-critical paths.

### Full Matrix

| Dimension                    | Trait-Object      | Closure-Based        | Generic B           | Box\<dyn Any\>   | CPS             |
| ---------------------------- | ----------------- | -------------------- | ------------------- | ---------------- | --------------- |
| Fuses N maps into 1          | Yes               | No                   | Yes                 | No               | No              |
| HKT Brand                    | Yes               | Yes                  | No                  | No (`'static`)   | No              |
| Functor trait impl           | Yes               | Yes                  | No                  | No               | No              |
| F: Functor required for map? | No (free functor) | Yes                  | No                  | Yes              | Yes             |
| Allocs per map               | 2                 | 1                    | 0                   | 2+               | 1               |
| Allocs per lower             | 0 + F::map cost   | 0 + N \* F::map cost | 0 + F::map cost     | 0 + N \* F::map  | 0 + N \* F::map |
| Clone                        | Via Rc/Arc        | No (FnOnce)          | If F<B>, func Clone | No               | No              |
| Send/Sync                    | Via Send variant  | Via Send variant     | If F<B>, func Send  | Via dyn Any+Send | N/A             |
| Lifetime                     | `'a` (arbitrary)  | `'a` (arbitrary)     | `'a` (arbitrary)    | `'static` only   | `'a`            |
| Foldable without Functor     | Yes               | No                   | Yes                 | No               | No              |
| Traversable                  | Via Rc/Arc Clone  | No                   | If Clone            | No               | No              |

---

## Chosen Encoding: Trait-Object

### Core Idea

Define a trait `CoyonedaInner<'a, F, A>` that hides the existential `B`. The concrete struct
`CoyonedaImpl<'a, F, B, A>` implements it. The outer `Coyoneda<'a, F, A>` wraps
`Box<dyn CoyonedaInner<'a, F, A> + 'a>`.

`map` composes the function (creating a new `CoyonedaImpl` with the same `fb` but a new
composed function), then re-boxes. `lower` calls `F::map(composed_func, fb)` exactly once.

### Why This Works in Rust

- The existential `B` is hidden by the trait object (no `dyn Any`, no `'static`).
- The `'a` lifetime on the trait object aligns with `Kind`'s `Of<'a, A: 'a>: 'a`.
- `impl_kind!` supports parameterized brands (precedents: `LazyBrand<Config>`,
  `ConstBrand<R>`, `TryThunkErrAppliedBrand<E>`, `FnBrand<P>`).
- `Apply!` expands mechanically; it splices the brand type verbatim.

### Why Not Closure-Based

The closure-based encoding nests `Brand::map` calls inside closures:

```rust
// Closure-based map (BROKEN):
fn map(self, f) -> Coyoneda {
    let prev = self.lower_fn;
    Coyoneda { lower_fn: Box::new(move || Brand::map(f, prev())) }
    //                                     ^^^^^^^^^^ called at every layer!
}
```

After 3 maps on VecBrand, `lower` calls `VecBrand::map` 3 times, traversing the Vec 3 times.
No fusion occurs. This is the same cost as calling `map` directly, plus closure allocation
overhead.

### The Dyn-Compatibility Constraint

The original design proposed a `map_inner<C>` method on `CoyonedaInner` that would compose
the function inside the trait object, keeping `fb` untouched:

```rust
// NOT dyn-compatible: C is a generic type parameter
fn map_inner<C: 'a>(
    self: Box<Self>,
    f: Box<dyn Fn(A) -> C + 'a>,
) -> Box<dyn CoyonedaInner<'a, F, C> + 'a>;
```

This achieves true fusion (one `F::map` call at `lower` time) but is NOT dyn-compatible because
`map_inner` has a generic type parameter `C`. Rust trait objects require all methods to have
fixed signatures at compile time (the vtable cannot contain generic entries).

To make `CoyonedaInner` dyn-compatible, the trait can only have the `lower` method (no generics).
This forces a **layered** design where each `map` creates a new wrapping struct.

### Actual Implementation: Layered Trait Objects

The implemented design uses two struct types implementing `CoyonedaInner`:

- `CoyonedaBase<'a, F, A>`: holds `fa: F::Of<'a, A>` directly. `lower` returns `fa` without
  calling `F::map`.
- `CoyonedaMapLayer<'a, F, B, A>`: holds `inner: Box<dyn CoyonedaInner<'a, F, B>>` and
  `func: Box<dyn Fn(B) -> A>`. `lower` calls `self.inner.lower()` then `F::map(self.func, ...)`.

After k chained maps, `lower` calls `F::map` k times (once per layer), same as direct
chaining. The layered encoding provides HKT integration and the foundation for optimized
Foldable/Traversable (where the fold function can be composed through layers), but does NOT
achieve map fusion for the Functor case.

For true single-pass fusion on eager brands, users should compose functions before mapping:
`map(compose(f, g), v)` instead of `map(f, map(g, v))`.

### Achieving True Fusion: Future Directions

True fusion (single `F::map` call regardless of k) requires composing `f: A -> C` with the
existing `g: B -> A` to produce `h: B -> C`, all while `B` is existentially hidden. Possible
approaches not yet implemented:

1. **`Box<dyn Any>` erasure**: Erase `fb` and the composed function via `Any`, downcast at
   `lower` time. Requires `'static` (same as `Free`), losing HKT trait integration.
2. **Unsafe pointer erasure**: Erase types via raw pointers, compose inside the impl.
   Achieves fusion without `'static` but requires `unsafe`.
3. **`FunctorPipeline` (generic B parameter)**: Expose `B` in the type. Zero-cost, true
   fusion, but cannot participate in HKT (extra type parameter).

---

## Detailed Design (as implemented)

### Inner Trait

```rust
trait CoyonedaInner<'a, F, A: 'a>: 'a
where
    F: Kind_cdc7cd43dac7585f + 'a,
{
    fn lower(self: Box<Self>) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
    where
        F: Functor;

    /// Compose another function, returning a new existential witness.
    fn map_inner<C: 'a>(
        self: Box<Self>,
        f: Box<dyn Fn(A) -> C + 'a>,
    ) -> Box<dyn CoyonedaInner<'a, F, C> + 'a>;

    /// Fold the inner structure directly, composing the fold function with
    /// the accumulated mapping function. Does not require F: Functor.
    fn fold_map_inner<FnBrand, M>(
        self: Box<Self>,
        f: impl Fn(A) -> M + 'a,
    ) -> M
    where
        F: Foldable,
        M: Monoid + 'a,
        A: Clone,
        FnBrand: CloneableFn + 'a;

    fn fold_right_inner<FnBrand, B2: 'a>(
        self: Box<Self>,
        f: impl Fn(A, B2) -> B2 + 'a,
        initial: B2,
    ) -> B2
    where
        F: Foldable,
        A: Clone,
        FnBrand: CloneableFn + 'a;
}
```

The trait grows with each type class Coyoneda supports. This is a bounded set (Functor,
Foldable, Traversable) and each method follows the same pattern: compose the caller's
function with the accumulated `func`, then delegate to `F`'s corresponding operation on `fb`.

### Concrete Implementation

```rust
struct CoyonedaImpl<'a, F, B: 'a, A: 'a>
where
    F: Kind_cdc7cd43dac7585f,
{
    fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
    func: Box<dyn Fn(B) -> A + 'a>,
}

impl<'a, F, B: 'a, A: 'a> CoyonedaInner<'a, F, A> for CoyonedaImpl<'a, F, B, A>
where
    F: Kind_cdc7cd43dac7585f,
{
    fn lower(self: Box<Self>) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
    where
        F: Functor,
    {
        F::map(self.func, self.fb)
    }

    fn map_inner<C: 'a>(
        self: Box<Self>,
        f: Box<dyn Fn(A) -> C + 'a>,
    ) -> Box<dyn CoyonedaInner<'a, F, C> + 'a> {
        let old_func = self.func;
        Box::new(CoyonedaImpl {
            fb: self.fb,
            func: Box::new(move |b: B| f((old_func)(b))),
        })
    }

    fn fold_map_inner<FnBrand, M>(
        self: Box<Self>,
        f: impl Fn(A) -> M + 'a,
    ) -> M
    where
        F: Foldable,
        M: Monoid + 'a,
        A: Clone,
        FnBrand: CloneableFn + 'a,
    {
        let func = self.func;
        F::fold_map::<FnBrand, B, M>(move |b: B| f((func)(b)), self.fb)
    }

    fn fold_right_inner<FnBrand, B2: 'a>(
        self: Box<Self>,
        f: impl Fn(A, B2) -> B2 + 'a,
        initial: B2,
    ) -> B2
    where
        F: Foldable,
        A: Clone,
        FnBrand: CloneableFn + 'a,
    {
        let func = self.func;
        F::fold_right::<FnBrand, B, B2>(
            move |b: B, acc: B2| f((func)(b.clone()), acc),
            initial,
            self.fb,
        )
    }
}
```

Note: `fold_right_inner` has a subtlety. `Foldable::fold_right` requires `A: Clone`
(the element type). Here, the element type seen by `F::fold_right` is `B`, not `A`. So the
`where` clause on the inner method needs `B: Clone`, which is known inside `CoyonedaImpl` but
not expressible on the `CoyonedaInner` trait (since `B` is hidden). This means `fold_right`
must propagate the `Clone` requirement through the existential. The practical solution:
the `CoyonedaImpl` can add `B: Clone` in its `where` clause, and the trait's method can
require that the implementor guarantees this. This needs careful handling during implementation.

### Outer Type

```rust
pub struct Coyoneda<'a, F, A: 'a>(Box<dyn CoyonedaInner<'a, F, A> + 'a>)
where
    F: Kind_cdc7cd43dac7585f;
```

### Brand

```rust
pub struct CoyonedaBrand<F>(PhantomData<F>);

impl_kind! {
    impl<F: Kind_cdc7cd43dac7585f> for CoyonedaBrand<F> {
        type Of<'a, A: 'a>: 'a = Coyoneda<'a, F, A>;
    }
}
```

The `Kind` bound on `F` is `Kind_cdc7cd43dac7585f` (the `type Of<'a, A: 'a>: 'a` signature),
not `Functor`. This means `CoyonedaBrand<F>` is a valid Brand for any type constructor with
the right arity, even if it lacks a Functor instance.

### Functor Implementation

```rust
impl<F: Kind_cdc7cd43dac7585f> Functor for CoyonedaBrand<F> {
    fn map<'a, A: 'a, B: 'a>(
        f: impl Fn(A) -> B + 'a,
        fa: Coyoneda<'a, F, A>,
    ) -> Coyoneda<'a, F, B> {
        Coyoneda(fa.0.map_inner(Box::new(f)))
    }
}
```

This does NOT require `F: Functor`. Coyoneda is a Functor for any `F`. The Functor constraint
only appears on `lower`.

### Allocation Profile

| Operation | Allocations                                             |
| --------- | ------------------------------------------------------- |
| `lift`    | 1 `Box<dyn CoyonedaInner>` + 1 `Box<dyn Fn>` (identity) |
| `map`     | 1 `Box<dyn CoyonedaInner>` + 1 `Box<dyn Fn>` (composed) |
| `lower`   | 0 from Coyoneda; plus whatever `F::map` allocates       |

Each `map` is 2 heap allocations (one for the composed function box, one for the new trait
object box), but the `fb` value is moved without copying. For `VecBrand`, this trades k Vec
allocations + k full traversals for k \* 2 small box allocations + 1 traversal. The boxes are
pointer-sized; the Vec traversals touch n elements each. For any non-trivial n, this is a
significant win.

### Lift and Lower

```rust
impl<'a, F, A: 'a> Coyoneda<'a, F, A>
where
    F: Kind_cdc7cd43dac7585f,
{
    pub fn lift(fa: <F as Kind_cdc7cd43dac7585f>::Of<'a, A>) -> Self {
        Coyoneda(Box::new(CoyonedaImpl {
            fb: fa,
            func: Box::new(|a| a), // identity
        }))
    }

    pub fn lower(self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
    where
        F: Functor,
    {
        self.0.lower()
    }
}
```

### hoistCoyoneda (natural transformation)

```rust
impl<'a, F, A: 'a> Coyoneda<'a, F, A>
where
    F: Kind_cdc7cd43dac7585f,
{
    pub fn hoist<G>(self, nat: impl FnOnce(...) -> ...) -> Coyoneda<'a, G, A>
    where
        G: Kind_cdc7cd43dac7585f,
    {
        // Apply nat to fb, then re-wrap with the same func.
        // Requires access to the inner fb, which means adding a
        // hoist_inner method to CoyonedaInner.
    }
}
```

This requires the inner trait to expose a `hoist_inner` method. Since `B` is known inside
`CoyonedaImpl`, the natural transformation `F ~> G` can be applied to `fb: F::Of<'a, B>` to
produce `G::Of<'a, B>`, then re-wrap with the same `func`.

---

## Type Class Support

### Functor

Works for any `F`. No `F: Functor` required. See above.

### Foldable

```rust
impl<F: Foldable> Foldable for CoyonedaBrand<F> {
    fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(...) -> B { ... }
    fn fold_map<'a, FnBrand, A: 'a + Clone, M>(...) -> M { ... }
}
```

Delegates to `fold_right_inner` / `fold_map_inner`, which compose the fold function with
the accumulated mapping function and fold over `fb` directly. This calls `F::fold_right` (or
`F::fold_map`) once, not `F::map` followed by `F::fold_right`. The Foldable instance requires
`F: Foldable` but NOT `F: Functor`.

**The Clone subtlety**: `Foldable::fold_right` requires the element type `A: Clone`. Inside
`CoyonedaImpl`, the actual element type of `fb` is `B`, not `A`. So `F::fold_right` requires
`B: Clone`. But `B` is existentially hidden; the outer Foldable impl only knows `A: Clone`.
Possible resolutions:

1. Require `B: Clone` at `lift` time by adding a `Clone` bound to `CoyonedaImpl`. This means
   only Clone-able values can be lifted into Coyoneda when Foldable is intended.
2. Use `fold_map` as the primary implementation (it also requires `A: Clone`, but the default
   `fold_right` is derived from `fold_map`). Same issue applies.
3. Accept that `Foldable for CoyonedaBrand<F>` lowers first, then folds. This loses the
   "fold without Functor" advantage but sidesteps the Clone propagation issue.

Resolution 1 is the most principled. The `lift` function can have a variant `lift_clone` with
the extra bound, or the standard `lift` can require `A: Clone` (since most Foldable use sites
require Clone anyway).

### Traversable

Requires `F: Traversable` (which implies `F: Functor + Foldable`). Also requires
`Self::Of<'a, B>: Clone`, meaning `Coyoneda<'a, F, B>: Clone`. The base trait-object encoding
does not support Clone. Options:

1. Implement Traversable only for the Rc/Arc hybrid variant.
2. Lower first, traverse with the underlying `F`, then re-lift. This works but adds an
   extra `F::map` call (for lowering) before the traversal.
3. Add a `traverse_inner` method to `CoyonedaInner` that composes the traversal function
   with the accumulated mapping function.

Option 3 combined with Clone support (via Rc/Arc wrapping) is the cleanest path, but
it can be deferred. Option 2 works as a first implementation.

### Applicative / Monad

Coyoneda is the free _functor_, not the free monad. `apply` and `bind` do not have natural
definitions that preserve the "deferred map" property. The standard approach (used in
PureScript) lowers both sides before applying:

```purescript
apply f g = liftCoyoneda $ lowerCoyoneda f <*> lowerCoyoneda g
```

This works but requires `F: Applicative` (or `F: Monad`) and triggers lowering. It can be
implemented but does not provide the same fusion benefit as Functor.

---

## Future Extensions

### Rc/Arc Hybrid Variant

Wrap the inner trait object in `Rc`/`Arc` instead of `Box`:

```rust
pub struct SharedCoyoneda<'a, F, A: 'a, P: RefCountedPointer>(
    P::Of<dyn CoyonedaInner<'a, F, A> + 'a>,
);
```

This enables Clone (via `Rc::clone`/`Arc::clone`), which unlocks Traversable and Applicative.
It also allows multiple consumers to share the same Coyoneda without lowering.

The accumulated function could use `FnBrand<P>` (i.e., `Rc<dyn Fn(B) -> A>` or
`Arc<dyn Fn(B) -> A>`) for consistency with the library's pointer abstraction hierarchy.

### FunctorPipeline (zero-cost utility)

A builder API that exposes `B` as a type parameter for zero-cost composition:

```rust
struct FunctorPipeline<'a, Brand, B, A, F: Fn(B) -> A> {
    fb: <Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>,
    func: F,
}
```

Each `.map(g)` changes the type to `FunctorPipeline<'a, Brand, B, C, impl Fn(B) -> C>`.
No boxing, no trait objects, no dynamic dispatch. Cannot participate in HKT (extra type
parameters), but excellent for performance-critical code.

### Send Variant

Mirror the `Thunk` / `SendThunk` split:

```rust
pub struct SendCoyoneda<'a, F, A: 'a>(
    Box<dyn CoyonedaInner<'a, F, A> + Send + 'a>,
);
```

Requires the accumulated function and `fb` to be `Send`.

---

## Open Questions

1. **Clone propagation for Foldable**: How to handle `B: Clone` when `B` is existentially
   hidden? Require it at `lift` time, or lower-then-fold?

2. **Fn boxing optimization**: Can the identity function in `lift` be special-cased to avoid
   the initial function box allocation? (e.g., an enum variant `Identity` vs `Composed(Box<dyn Fn>)`.)

3. **Inner trait size**: As more type classes add methods to `CoyonedaInner`, the vtable grows.
   Is this a concern? (Probably not; the set of relevant type classes is small and fixed.)

4. **Interaction with RefFunctor / Lazy**: `Lazy` uses `RefFunctor` (mapping with `&A -> B`).
   Should Coyoneda support a `RefFunctor` variant? This would require the accumulated function
   to work with references, which complicates the existential encoding.

5. **Naming**: `Coyoneda` is the standard categorical name. Should the library also provide
   a more approachable alias (e.g., `DeferredMap`, `FusedFunctor`)?

---

## Implementation Notes (2026-03-30)

### What Was Built

A working prototype in `fp-library/src/types/coyoneda.rs` with:

- `CoyonedaInner` trait (dyn-compatible, only `lower` method)
- `CoyonedaBase` struct (wraps `F<A>` directly, no mapping)
- `CoyonedaMapLayer` struct (wraps inner + function, applies `F::map` at lower time)
- `Coyoneda` outer type
- `CoyonedaBrand<F>` in `brands.rs` with `impl_kind!` and `Functor` impl
- 11 unit tests + 6 doc tests, all passing

### Key Findings During Implementation

1. **Dyn-compatibility blocks true fusion.** The `map_inner<C>` method proposed in the original
   design has a generic type parameter `C`, making `CoyonedaInner` not dyn-compatible. The
   trait object `Box<dyn CoyonedaInner>` cannot have methods with generic type parameters
   because the vtable must be fixed at compile time. This forced a layered design where each
   `map` wraps the previous inner in a new `CoyonedaMapLayer`, and `lower` calls `F::map`
   once per layer.

2. **Brand type parameter requires `'static`.** The `impl_kind!` macro generates
   `type Of<'a, A: 'a>: 'a = Coyoneda<'a, F, A>`. For this to work, `F` must outlive all
   possible `'a`, which means `F: 'static`. This is fine because all brands are zero-sized
   marker types (inherently `'static`).

3. **Base layer optimization.** `CoyonedaBase` returns `fa` directly without calling `F::map`.
   This means `lift(v).lower()` is a no-op (identity), avoiding the cost of
   `F::map(identity, v)` that would occur if the base case used a stored identity function.

4. **The layered approach still provides value** even without fusion:
   - HKT integration (CoyonedaBrand<F> implements Functor for any F)
   - Free functor property (F does not need to be a Functor for map, only for lower)
   - Clean polymorphic type for passing deferred-map values around
   - Foundation for Foldable optimization (composing the fold function through layers)

### Allocation Profile (actual implementation)

| Operation | Allocations                                                                |
| --------- | -------------------------------------------------------------------------- |
| `lift`    | 1 `Box<dyn CoyonedaInner>` (wrapping `CoyonedaBase`)                       |
| `map`     | 1 `Box<dyn CoyonedaInner>` (wrapping `CoyonedaMapLayer`) + 1 `Box<dyn Fn>` |
| `lower`   | k calls to `F::map` (one per accumulated map layer)                        |

For k chained maps on `VecBrand` with n elements:

- k `Box` allocs at map time (pointer-sized, O(1) each)
- k `F::map` calls at lower time, each traversing n elements
- Total: O(k \* n) traversal cost, same as direct chaining

The benefit vs direct chaining is structural, not performance: deferred execution, HKT
integration, and a clean abstraction boundary.
