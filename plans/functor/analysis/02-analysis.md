# Coyoneda Implementation Analysis

An independent assessment of the Coyoneda implementation in `fp-library`, covering
flaws, issues, and limitations with proposed approaches and recommendations.

Date: 2026-03-31

---

## Table of Contents

1. [No map fusion despite being the free functor](#1-no-map-fusion-despite-being-the-free-functor)
2. [Fn bound instead of FnOnce for stored mapping functions](#2-fn-bound-instead-of-fnonce-for-stored-mapping-functions)
3. [Redundant allocation in the new constructor](#3-redundant-allocation-in-the-new-constructor)
4. [Stack overflow risk from deeply nested layers](#4-stack-overflow-risk-from-deeply-nested-layers)
5. [Foldable requires F: Functor, diverging from the free functor contract](#5-foldable-requires-f-functor-diverging-from-the-free-functor-contract)
6. [hoist requires F: Functor, diverging from PureScript](#6-hoist-requires-f-functor-diverging-from-purescript)
7. [Not Clone, Send, or Sync](#7-not-clone-send-or-sync)
8. [Missing type class instances](#8-missing-type-class-instances)
9. [Identity function allocation in lift](#9-identity-function-allocation-in-lift)
10. [Foldable lower-then-fold defeats single-pass optimization](#10-foldable-lower-then-fold-defeats-single-pass-optimization)

---

## 1. No map fusion despite being the free functor

### Description

The primary motivation for Coyoneda, stated in both the design document and the module
documentation, is automated map fusion: accumulating chained `map` calls as function
composition so that `lower` performs a single `F::map` regardless of how many maps were
chained. The implementation does not achieve this. After k chained maps, `lower` calls
`F::map` exactly k times, once per `CoyonedaMapLayer`. The performance is identical to
calling `F::map` directly k times.

This is the single most significant issue. It means the implementation's advertised
primary benefit does not materialize. The design document acknowledges this and attributes
it to dyn-compatibility: the `map_inner<C>` method that would compose functions across
the existential boundary is generic over `C`, which prevents the trait from being used as
a trait object.

### Approaches

**A. Unsafe pointer erasure with type tags.**
Erase the inner type `B` via a raw pointer and a "compose" operation that operates on
type-erased function pointers. At `lower` time, downcast back to the concrete types.
This achieves true fusion with arbitrary lifetimes, but introduces `unsafe` code and
requires careful correctness verification.

- Trade-offs: True fusion; no `'static` requirement; introduces `unsafe`; complex to
  audit; potential for UB if type tags are incorrect.

**B. `Box<dyn Any>` erasure.**
Erase `fb` and the composed function via `Any`, downcast at `lower` time. This achieves
fusion but requires `'static` on all types, which prevents HKT integration (the `Kind`
trait requires lifetime polymorphism `Of<'a, A: 'a>: 'a`).

- Trade-offs: True fusion; safe Rust; loses HKT integration; loses lifetime flexibility.

**C. `FunctorPipeline` companion type (generic B parameter).**
Expose `B` as a type parameter in the struct, eliminating the need for trait objects.
Zero-cost, true fusion, but cannot participate in HKT because the extra type parameter
does not fit the `Kind` trait shape.

- Trade-offs: True fusion; zero-cost (no boxing); no HKT; excellent for
  performance-critical paths; can coexist alongside the current Coyoneda.

**D. Accept the limitation; reposition the type.**
Stop advertising map fusion as the motivation. Reposition Coyoneda as providing HKT
integration (any type constructor gets a `Functor` for free) and deferred mapping
(functions are stored, not applied until `lower`). This is honest and avoids misleading
users, though it removes the performance narrative.

### Recommendation

Pursue both C and D. Implement `FunctorPipeline` as a zero-cost companion for
performance-critical paths where true fusion matters. Reposition the trait-object
`Coyoneda` as the HKT-integrated free functor that provides structural benefits (deferred
execution, functor-for-free) rather than performance benefits. This gives users both
options with clear trade-offs. Option A could be revisited later if there is demand for
fused HKT-compatible Coyoneda, but the `unsafe` cost is substantial.

---

## 2. Fn bound instead of FnOnce for stored mapping functions

### Description

`CoyonedaMapLayer` stores its function as `Box<dyn Fn(B) -> A + 'a>`, using `Fn` rather
than `FnOnce`. Since `lower` consumes `self: Box<Self>`, the function is only ever called
via `F::map`, and `Functor::map` in this library takes `impl Fn(A) -> B + 'a`. So the `Fn`
bound is necessary to satisfy the `Functor::map` signature.

However, this means that closures capturing non-cloneable, move-only state cannot be used
with `Coyoneda::map`. For example, a closure that moves a `String` into itself and
consumes it cannot implement `Fn`, only `FnOnce`. This is a restriction inherited from
the library's `Functor::map` signature using `impl Fn` rather than `impl FnOnce`.

### Approaches

**A. Accept the limitation.**
The `Fn` bound is consistent with the library's `Functor::map` signature. Changing it
would require changing `Functor` itself, which has far-reaching implications.

**B. Change `Functor::map` to `FnOnce`.**
This is a library-wide design decision. The `CLAUDE.md` explains that `impl Fn` is
deliberate for zero-cost abstractions (uncurried semantics). `FnOnce` would allow
move-only closures but would prevent reusing the same function across multiple calls (e.g.,
in `fold_map` where the function is called once per element). This is not practical for
the general `Functor` trait.

**C. Provide a separate `map_once` method.**
Add a `map_once` that accepts `FnOnce` and stores it as `Box<dyn FnOnce(B) -> A>`. This
layer would need a different lowering strategy since `F::map` requires `Fn`. The layer
could compose `FnOnce` closures into a single `FnOnce` at lower time, but the current
layered design calls `F::map` per layer, so each layer needs its function to satisfy `Fn`.

- Trade-offs: Adds API complexity; does not compose well with the layered design; limited
  practical benefit.

### Recommendation

Accept the limitation (approach A). The `Fn` bound is consistent with the library's design
philosophy and `Functor::map` signature. Documenting this constraint explicitly in the
`map` method documentation would be helpful. The practical impact is small because most
mapping functions are pure transformations that naturally implement `Fn`.

---

## 3. Redundant allocation in the new constructor

### Description

`Coyoneda::new(f, fb)` creates a `CoyonedaMapLayer` wrapping a `CoyonedaBase`:

```rust
Coyoneda(Box::new(CoyonedaMapLayer {
    inner: Box::new(CoyonedaBase { fa: fb }),
    func: Box::new(f),
}))
```

This performs 3 heap allocations: one `Box` for the outer `CoyonedaMapLayer`, one `Box`
for the inner `CoyonedaBase`, and one `Box` for the function. The design document notes
that `new` "saves one box allocation vs `lift(fb).map(f)`", which is true (`lift` then
`map` would be 3 boxes too, but `lift` allocates 1 box, then `map` allocates 2 more, for
a total of 3 with the `lift` box becoming the inner). So `new` is equivalent in allocation
count, not cheaper.

A direct implementation using a single `CoyonedaImpl`-style struct (as described in the
design document's "Concrete Implementation" section) could reduce this to 2 allocations:
one `Box` for the outer trait object and one `Box` for the function. However, the design
document's `CoyonedaImpl` was not used in the actual implementation; instead, the layered
`CoyonedaBase`/`CoyonedaMapLayer` design was chosen.

### Approaches

**A. Introduce a `CoyonedaImpl` struct for `new`.**
Add a struct `CoyonedaImpl<'a, F, B, A>` that holds both `fb` and `func` directly,
implementing `CoyonedaInner`. Use it in `new` to avoid the intermediate `CoyonedaBase`
box. This saves 1 allocation in `new`.

- Trade-offs: Adds a third struct type; small code complexity increase; saves 1 allocation
  per `new` call.

**B. Accept the current allocation profile.**
The extra allocation is a single pointer-sized `Box`. For any non-trivial use case, this
is negligible.

### Recommendation

Approach A is straightforward and aligns with the design document's `CoyonedaImpl` struct.
It eliminates an unnecessary allocation with minimal code complexity. Given that the
design document already describes this struct, implementing it would also bring the code
closer to the documented design.

---

## 4. Stack overflow risk from deeply nested layers

### Description

Each `map` call adds a `CoyonedaMapLayer` wrapping the previous inner value. At `lower`
time, the implementation recurses through the layers: `CoyonedaMapLayer::lower` calls
`self.inner.lower()`, which may itself be a `CoyonedaMapLayer` that calls its own
`inner.lower()`, and so on. For k chained maps, this produces k frames of recursion on
the call stack.

The test `many_chained_maps` uses 100 layers, which is safe. But for large k (e.g.,
thousands of maps in a loop), this will overflow the stack. The `Free` monad in this
library uses CatList-based "Reflection without Remorse" for stack safety, so the library
is clearly aware of this class of problem.

### Approaches

**A. Trampolining the lower operation.**
Restructure `lower` to use an explicit loop rather than recursion. Since each layer holds
a `Box<dyn CoyonedaInner>`, the loop would need to unpack each layer iteratively. This
is difficult because each layer's `lower` returns a different type (`F::Of<'a, B>` for
various `B`), and the function `B -> A` must be applied via `F::map` at each step. The
types change at each layer, so a simple loop is not straightforward.

One approach: collect all the layers into a `Vec`, then apply `F::map` from the bottom up
in a loop. This requires type erasure for the intermediate `F::Of<'a, B>` values, which
brings back the `dyn Any`/`'static` problems.

- Trade-offs: Stack safe; significantly more complex; may require type erasure or `unsafe`.

**B. Document the limitation and set a practical bound.**
Document that deeply nested maps (thousands of layers) may overflow the stack. Recommend
that users compose functions manually for deep chains, or use `FunctorPipeline` (if
implemented) which has no layering.

- Trade-offs: Simple; honest; does not solve the problem; may surprise users who build
  Coyoneda values in recursive algorithms.

**C. Flatten layers eagerly using `F::map` during `map`.**
Instead of deferring all maps to `lower` time, apply `F::map` eagerly at each `map` call.
This eliminates layering entirely but also eliminates the deferred-execution benefit and
makes `map` require `F: Functor`.

- Trade-offs: Stack safe; eliminates the free functor property; defeats the purpose of
  Coyoneda.

### Recommendation

Approach B is appropriate for now. The typical use case for Coyoneda is a moderate number
of chained maps (single digits to low hundreds), not thousands. Document the stack depth
limitation clearly, and add a note in the `map` documentation. If `FunctorPipeline` is
implemented, it naturally avoids this issue since it uses type-level composition with no
runtime layering. For a future improvement, approach A could be explored, but the
complexity is substantial relative to the practical risk.

---

## 5. Foldable requires F: Functor, diverging from the free functor contract

### Description

The `Foldable` instance for `CoyonedaBrand<F>` requires `F: Functor + Foldable + 'static`.
It works by lowering the Coyoneda first (which calls `F::map` per layer to produce
`F::Of<'a, A>`), then folding the result with `F::fold_map`.

In PureScript, `Foldable (Coyoneda f)` only requires `Foldable f`, not `Functor f`. This
is because PureScript's `unCoyoneda` can open the existential to compose the fold function
with the accumulated mapping function, folding over the original `f b` in a single pass
without ever calling `map`.

The Rust implementation cannot do this because a `fold_map_inner` method on `CoyonedaInner`
would need to be generic over the monoid type `M` and the `FnBrand`, making the trait not
dyn-compatible.

### Approaches

**A. Accept the extra bound.**
The `F: Functor` requirement is a pragmatic consequence of the encoding. Most types that
are `Foldable` are also `Functor` in practice, so the restriction rarely bites.

- Trade-offs: Simple; honest; slightly weaker contract than PureScript.

**B. Enum-based inner representation.**
Replace the trait object with an enum that has variants for `Base` and `MapLayer`. Pattern
matching on the enum allows accessing the concrete types without trait object dispatch.
However, `MapLayer` still hides the type `B` existentially (it must, because the enum
variant needs a fixed type), so the same problem resurfaces unless type erasure is used.

- Trade-offs: Does not actually solve the problem without type erasure; adds complexity.

**C. Use a different trait per type class.**
Define separate inner traits, e.g., `CoyonedaFoldInner`, that are specialized for folding
and avoid generic methods by fixing the monoid type. This does not generalize well because
`fold_map` must work with any monoid.

- Trade-offs: Does not generalize; impractical.

### Recommendation

Accept the limitation (approach A). Document clearly that `Foldable for CoyonedaBrand<F>`
requires `F: Functor`, and explain why this diverges from PureScript. The existing
documentation already does this well. The practical impact is minimal because `Functor` is
nearly universal among `Foldable` types.

---

## 6. hoist requires F: Functor, diverging from PureScript

### Description

`Coyoneda::hoist` is implemented as `Coyoneda::lift(nat.transform(self.lower()))`. This
lowers the Coyoneda (requiring `F: Functor`), applies the natural transformation, then
re-lifts. PureScript's `hoistCoyoneda` opens the existential via `unCoyoneda` and applies
the nat transform directly to the inner `f b`, which does not require `Functor f`.

### Approaches

**A. Accept the extra bound.**
Same reasoning as the Foldable case. The `F: Functor` requirement is a pragmatic
consequence.

**B. Add a `hoist_inner` method to `CoyonedaInner`.**
This method would be `fn hoist_inner<G>(self: Box<Self>, nat: &dyn NaturalTransformationDyn<F, G>) -> Box<dyn CoyonedaInner<'a, G, A>>`.
The problem is that `G` is a generic type parameter, making the method not dyn-compatible.
Even with a trait-object version of `NaturalTransformation`, the return type
`Box<dyn CoyonedaInner<'a, G, A>>` varies with `G`.

- Trade-offs: Not feasible without violating dyn-compatibility.

**C. Enum-based inner representation.**
Same as for Foldable; does not solve the root problem without type erasure.

### Recommendation

Accept the limitation (approach A). The `F: Functor` bound on `hoist` is a minor
restriction. Natural transformations are typically applied between functors that both have
`Functor` instances. The existing documentation is clear about this divergence.

---

## 7. Not Clone, Send, or Sync

### Description

`Coyoneda` wraps `Box<dyn CoyonedaInner<'a, F, A> + 'a>`. `Box<dyn Trait>` is not `Clone`.
The stored functions are `Box<dyn Fn(B) -> A>`, which are also not `Clone`. This prevents:

- Implementing `Traversable` (requires `Self::Of<'a, B>: Clone`).
- Implementing `Semiapplicative`/`Applicative` (requires cloning one side).
- Sharing a Coyoneda value across multiple consumers.
- Sending a Coyoneda across threads.

### Approaches

**A. Rc/Arc-wrapped variant.**
Replace `Box` with `Rc` (or `Arc` for `Send + Sync`). The function would use
`Rc<dyn Fn(B) -> A>` (or `Arc<dyn Fn(B) -> A>`). `Clone` becomes `Rc::clone`/`Arc::clone`.
This enables `Traversable`, `Semiapplicative`, and sharing.

- Trade-offs: Reference counting overhead; two variants needed (Rc for single-threaded,
  Arc for multi-threaded); aligns with the library's existing `RcFnBrand`/`ArcFnBrand`
  split and the `Thunk`/`SendThunk` pattern.

**B. Parameterize over pointer type.**
Use the library's `RefCountedPointer` hierarchy to parameterize `Coyoneda` over the
pointer brand, similar to how `FnBrand<P>` is parameterized. This avoids code duplication
between Rc and Arc variants.

- Trade-offs: More complex type signatures; aligns with library conventions; single
  implementation covers both cases.

**C. Add Send bound to the existing type.**
Require `Send` on the trait object: `Box<dyn CoyonedaInner + Send + 'a>`. This forces all
stored functions and values to be `Send`, which is too restrictive for the default case.

- Trade-offs: Too restrictive; breaks non-Send use cases.

### Recommendation

Approach B is the most aligned with library conventions. The pointer parameterization
pattern is already established with `FnBrand<P>`, `LazyBrand<Config>`, and the
`Thunk`/`SendThunk` split. Implementing this would unlock `Traversable`, `Clone`, and
thread-safe variants in a unified way. However, this is a significant amount of work and
can be deferred; the current `Box`-based Coyoneda serves the core use case (deferred
mapping with HKT integration) adequately.

---

## 8. Missing type class instances

### Description

PureScript's `Coyoneda` provides instances for `Functor`, `Apply`, `Applicative`, `Bind`,
`Monad`, `Foldable`, `Traversable`, `Extend`, `Comonad`, `Eq`, `Ord`, and others. The
current implementation provides `Functor`, `Pointed`, and `Foldable`.

The missing instances fall into categories:

- **Blocked by Clone** (`Traversable`, `Semiapplicative`): require the Rc/Arc variant.
- **Require lowering** (`Semimonad`/`Bind`, `Eq`, `Ord`, `Debug`): these can lower, operate
  on the underlying `F`, then re-lift. They require `F: Functor` for lowering plus the
  respective constraint on `F`.
- **Require additional infrastructure** (`Extend`, `Comonad`): these type classes may not
  yet exist in the library.

### Approaches

**A. Implement lower-based instances incrementally.**
Add `Eq`, `Ord`, and `Debug` by lowering and delegating. These require `F: Functor` and
the corresponding trait on `F::Of<'a, A>`.

- Trade-offs: Straightforward; requires lowering (so the Coyoneda is consumed or must be
  cloned); useful for debugging and testing.

**B. Implement Semiapplicative/Semimonad via lowering.**
These lower both sides, apply the operation on `F`, then re-lift. Requires `F: Functor`
plus the respective class.

- Trade-offs: No fusion benefit for these operations; matches PureScript semantics; adds
  useful functionality.

**C. Defer until Rc/Arc variant exists.**
`Traversable` and other Clone-dependent instances should wait for the shared variant.

### Recommendation

Implement `Eq`, `Ord`, and `Debug` first (approach A), as these are most useful for
practical code (testing, logging). Then implement `Semiapplicative`/`Semimonad` via
lowering (approach B). Defer `Traversable` until the Rc/Arc variant is available (approach
C). This follows a natural priority order based on user value and implementation
difficulty.

---

## 9. Identity function allocation in lift

### Description

`Coyoneda::lift` wraps the value in a `CoyonedaBase` and boxes it. Unlike `new`, it does
not allocate a function box because `CoyonedaBase` stores `fa` directly and returns it
unchanged in `lower`. This is already optimized.

However, the design document raises an open question about whether `lift` could avoid the
identity function allocation. Since the actual implementation uses `CoyonedaBase` (no
stored function), this is already addressed. The design document's concern applies to the
`CoyonedaImpl`-based design, not the implemented layered design.

### Assessment

This is a non-issue. The implementation already avoids the identity function allocation
via the `CoyonedaBase` variant. No changes needed.

---

## 10. Foldable lower-then-fold defeats single-pass optimization

### Description

The `Foldable` implementation for `CoyonedaBrand<F>` calls `fa.lower()` (which produces
`F::Of<'a, A>` by applying k `F::map` calls), then calls `F::fold_map` on the result.
For `VecBrand` with n elements and k chained maps, this performs:

- k full traversals of n elements (from lowering, one `F::map` per layer).
- 1 full traversal of n elements (from the final fold).
- k intermediate Vec allocations (from the k `F::map` calls).

Total cost: O((k+1) \* n) traversals and k intermediate allocations.

An ideal implementation would compose the fold function with all the accumulated mapping
functions, then fold the original `f b` in a single pass: O(n) with 0 intermediate
allocations. This is what PureScript achieves.

### Approaches

**A. Accept the current implementation.**
The cost is the same as calling `map` k times then folding. Users who care about
performance can compose their functions manually before mapping.

**B. Implement `FunctorPipeline` with Foldable.**
A `FunctorPipeline<Brand, B, A, F>` that exposes `B` can trivially compose the fold
function with `F` and fold over the original `fb` in a single pass. This provides
true single-pass folding without trait objects.

- Trade-offs: Requires implementing `FunctorPipeline`; no HKT; excellent performance.

**C. Specialized inner trait for folding.**
Create a separate, non-dyn-compatible trait for fold operations, and use a different
mechanism (e.g., manually monomorphized dispatch) to compose through layers. This is
essentially approach A from issue 1 applied specifically to folding.

- Trade-offs: Complex; marginal benefit over approach B.

### Recommendation

Approach B, paired with the recommendation from issue 1. `FunctorPipeline` would provide
both true map fusion and true fold fusion as a zero-cost, non-HKT companion. The
trait-object Coyoneda retains its role as the HKT-integrated free functor with the
understanding that its performance profile is structural, not optimal.

---

## Summary of Recommendations

| Issue                        | Severity | Recommendation                                                            |
| ---------------------------- | -------- | ------------------------------------------------------------------------- |
| No map fusion                | High     | Implement `FunctorPipeline` companion; reposition Coyoneda documentation. |
| Fn bound vs FnOnce           | Low      | Accept; consistent with library design.                                   |
| Redundant allocation in new  | Low      | Introduce `CoyonedaImpl` struct to save 1 allocation.                     |
| Stack overflow risk          | Medium   | Document limitation; `FunctorPipeline` avoids it naturally.               |
| Foldable requires Functor    | Low      | Accept; document clearly (already done).                                  |
| hoist requires Functor       | Low      | Accept; document clearly (already done).                                  |
| Not Clone/Send/Sync          | Medium   | Implement Rc/Arc variant parameterized over pointer brand.                |
| Missing type class instances | Medium   | Implement incrementally: Eq/Ord/Debug first, then Applicative/Monad.      |
| Identity allocation in lift  | None     | Already optimized via CoyonedaBase.                                       |
| Foldable lower-then-fold     | Medium   | `FunctorPipeline` provides true single-pass folding.                      |

The two highest-impact improvements are:

1. **`FunctorPipeline`**: a zero-cost companion type that provides true map and fold fusion
   without HKT integration. This addresses issues 1, 4, and 10 simultaneously.

2. **Rc/Arc-wrapped variant**: parameterized over the pointer brand, enabling Clone, Send,
   Traversable, and Semiapplicative. This addresses issues 7 and 8.

Both can be implemented incrementally without modifying the existing `Coyoneda` type.
