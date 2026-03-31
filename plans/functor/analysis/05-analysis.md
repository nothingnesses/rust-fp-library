# Coyoneda Implementation Analysis

An independent assessment of the Coyoneda implementation in `fp-library/src/types/coyoneda.rs`,
covering flaws, issues, and limitations with proposed approaches and recommendations.

Date: 2026-03-31

---

## Table of Contents

1. [No map fusion](#1-no-map-fusion)
2. [Fn instead of FnOnce for accumulated functions](#2-fn-instead-of-fnonce-for-accumulated-functions)
3. [Unnecessary double allocation in the new constructor](#3-unnecessary-double-allocation-in-the-new-constructor)
4. [Foldable requires F: Functor](#4-foldable-requires-f-functor)
5. [Hoist requires F: Functor](#5-hoist-requires-f-functor)
6. [Stack overflow risk with deeply nested layers](#6-stack-overflow-risk-with-deeply-nested-layers)
7. [No Send or Sync support](#7-no-send-or-sync-support)
8. [No Clone support](#8-no-clone-support)
9. [Identity function allocation in lift](#9-identity-function-allocation-in-lift)
10. [Missing type class instances](#10-missing-type-class-instances)

---

## 1. No Map Fusion

### Description

The primary motivation for Coyoneda is map fusion: composing k chained `map` calls into a
single `F::map` call at `lower` time. The current implementation does not achieve this. Each
`map` adds a `CoyonedaMapLayer` wrapper, and `lower` calls `F::map` once per layer. For k
chained maps on `VecBrand` with n elements, the cost is O(k \* n), the same as calling
`F::map` directly k times.

This is the most significant gap between the implementation and its stated purpose. The design
document acknowledges this and attributes it to Rust's dyn-compatibility rules: composing
functions across the existential boundary requires a generic `map_inner<C>` method, which
cannot exist on a trait object.

### Approaches

**A. Unsafe type erasure via raw pointers.** Erase the existential `B` using pointer casts,
compose functions in a type-erased representation, and reconstruct at `lower` time. This
achieves true O(1) map and single `F::map` at lower.

- Pro: Full fusion, no `'static` requirement, HKT integration preserved.
- Con: Requires `unsafe`, difficult to audit, potential for soundness bugs.

**B. `Box<dyn Any>` erasure.** Erase `fb` and the composed function via `Any`, downcast at
`lower` time.

- Pro: Safe Rust, true fusion.
- Con: Requires `A: 'static` and `B: 'static`, preventing HKT integration (the `Kind` trait
  needs lifetime polymorphism). Same limitation as `Free`.

**C. Enum-based composition within a single layer.** Instead of nesting layers, store a
`Vec<Box<dyn Fn>>` of functions and compose them at `lower` time into a single closure
before calling `F::map` once.

- Pro: Safe Rust, single `F::map` call.
- Con: Still requires type erasure for the intermediate types between composed functions.
  The types of successive functions (`A -> B`, `B -> C`, `C -> D`) are all different,
  so a homogeneous `Vec` cannot hold them without `dyn Any` or unsafe erasure.

**D. Accept the limitation and provide a separate `FunctorPipeline` API.** The design document
already proposes this: a generic struct that exposes the intermediate type `B` in its
signature, enabling zero-cost composition without boxing. It cannot participate in HKT but
serves the performance-critical use case.

- Pro: Zero-cost, safe, simple.
- Con: Does not improve `Coyoneda` itself; parallel API to maintain.

### Recommendation

Approach D is the most practical near-term solution. Provide `FunctorPipeline` as the
zero-cost fusion utility, and keep `Coyoneda` for its HKT and free functor properties. If
fusion within the HKT-compatible type is essential, approach A (unsafe erasure) is the only
path that preserves both fusion and lifetime polymorphism, but it should be pursued only with
thorough `unsafe` review and Miri testing.

---

## 2. Fn Instead of FnOnce for Accumulated Functions

### Description

`CoyonedaMapLayer` stores its function as `Box<dyn Fn(B) -> A + 'a>`, requiring `Fn` (callable
multiple times). However, each accumulated function is called exactly once during `lower`:
`F::map(self.func, lowered)`. Since `Functor::map` accepts `impl Fn(A) -> B + 'a` (not
`FnOnce`), the stored function must be `Fn` to satisfy `F::map`'s signature.

This is technically correct given the current `Functor` trait signature, but it creates an
unnecessary restriction on the functions that can be stored. A user cannot `lift` and then
`map` with a closure that captures a non-`Clone` value by move, because `Fn` requires the
closure to be callable multiple times, which generally means captured values must be `Clone`
(or be shared references).

For example, this would fail:

```rust
let owned = String::from("hello");
// This closure is FnOnce, not Fn, because it moves `owned`.
Coyoneda::<OptionBrand, _>::lift(Some(1)).map(move |_| owned)
// Error: cannot move out of captured variable in Fn closure
```

In practice, `Functor::map` in this library uses `impl Fn` (not `impl FnOnce`) as a design
decision for consistency and because some `Functor` implementations (like `Vec`) call the
function multiple times. So the `Fn` bound on the stored function is forced by the trait.

### Approaches

**A. Accept the status quo.** The `Fn` bound is a consequence of `Functor::map`'s signature.
Any change would need to happen at the trait level, which would be a sweeping change across
the entire library.

**B. Provide an `FnOnce`-based `map_once` method.** This would store `Box<dyn FnOnce(B) -> A>`
and only work for brands where the functor contains at most one element (like `OptionBrand`,
`IdentityBrand`). This is niche and adds complexity.

**C. Use interior mutability.** Wrap the function in a `Cell<Option<Box<dyn FnOnce>>>` and
take it on first call. This allows `FnOnce` closures to satisfy a `Fn` interface at the cost
of a runtime panic if called twice. This pattern is fragile and not idiomatic.

### Recommendation

Approach A. The `Fn` bound is a fundamental consequence of the library's `Functor` trait
design. Documenting why the bound is `Fn` rather than `FnOnce` would be helpful for users who
encounter this restriction.

---

## 3. Unnecessary Double Allocation in the New Constructor

### Description

`Coyoneda::new(f, fb)` creates a `CoyonedaMapLayer` wrapping a `CoyonedaBase`:

```rust
pub fn new<B: 'a>(f: impl Fn(B) -> A + 'a, fb: ...) -> Self {
    Coyoneda(Box::new(CoyonedaMapLayer {
        inner: Box::new(CoyonedaBase { fa: fb }),
        func: Box::new(f),
    }))
}
```

This performs 3 heap allocations: one `Box` for `CoyonedaBase`, one `Box` for the function,
and one `Box` for the outer `CoyonedaMapLayer` trait object. The design document's
`CoyonedaImpl` struct (from the "Detailed Design" section) stores `fb` and `func` together
in a single struct behind one trait object box, which would require only 2 allocations.

The implemented approach uses `CoyonedaBase` + `CoyonedaMapLayer` instead of a unified
`CoyonedaImpl`. While `CoyonedaBase` avoids calling `F::map(identity, fa)` on `lower` (a
genuine optimization for `lift`), the `new` constructor does not benefit from this
optimization since it always has a non-identity function.

### Approaches

**A. Add a `CoyonedaImpl` struct.** Introduce a third struct (matching the design document)
that stores `fb` and `func` together. Use it in `new` to save one allocation. `lift` would
continue to use `CoyonedaBase` for the identity optimization.

- Pro: Saves one allocation per `new` call.
- Con: Third struct to maintain; more code in the inner trait implementation.

**B. Accept the extra allocation.** The overhead of one extra pointer-sized `Box` allocation is
negligible for most use cases.

- Pro: Simpler code.
- Con: Slightly wasteful; `new` is documented as saving one allocation vs `lift(fb).map(f)`,
  but in practice it still allocates one more Box than necessary.

### Recommendation

Approach A. The `CoyonedaImpl` struct is already designed in the document, it saves a
meaningful allocation, and it makes `new` genuinely cheaper than `lift + map` (2 boxes vs 3).
The implementation complexity is modest since the trait only has one method (`lower`).

---

## 4. Foldable Requires F: Functor

### Description

The `Foldable` implementation for `CoyonedaBrand<F>` requires `F: Functor + Foldable`:

```rust
impl<F: Functor + Foldable + 'static> Foldable for CoyonedaBrand<F> { ... }
```

It works by calling `fa.lower()` (which requires `F: Functor`) and then folding the result
with `F::fold_map`. PureScript's Coyoneda only requires `F: Foldable` because it can open
the existential via `unCoyoneda` and compose the fold function with the accumulated mapping
function, folding the original `F B` directly.

This means `CoyonedaBrand<F>` cannot be `Foldable` when `F` is `Foldable` but not `Functor`,
which undermines the "free functor" property. In practice, most `Foldable` types are also
`Functor`, so this is rarely limiting, but it is a semantic divergence.

### Approaches

**A. Non-dyn-compatible inner trait with enum dispatch.** Replace the `Box<dyn CoyonedaInner>`
with an enum that has two variants: `Base(F<A>)` and `MapLayer(enum_inner, Box<dyn Fn>)`.
Since the enum is a concrete type, it can have generic methods. However, the enum cannot hide
the existential `B`; it would need to be a type parameter, which breaks HKT integration.

**B. Specialized inner trait per type class.** Add a separate trait
`CoyonedaFoldableInner<'a, F, A, M, FnBrand>` that is not generic in its methods. The
problem is that `M` (the monoid type) and `FnBrand` are only known at fold-call-site, not at
construction time, so they cannot be baked into the trait object type at `lift`/`map` time.

**C. Lower-then-fold (current approach).** Accept the `F: Functor` requirement.

- Pro: Simple, correct, works.
- Con: Extra `F: Functor` bound; for `VecBrand`, lowering before folding creates an
  intermediate `Vec` that is immediately consumed.

**D. Unsafe type erasure for fold composition.** Similar to approach 1A for map fusion:
erase `B` via raw pointers, compose `fold_fn . map_fn` in a type-erased representation.

- Pro: Removes `F: Functor` requirement.
- Con: `unsafe`, difficult to maintain.

### Recommendation

Approach C (the current implementation) is acceptable. The extra `F: Functor` bound is
well-documented and rarely limiting. If fusion is pursued in the future (issue 1), the same
mechanism that enables map fusion would likely also enable Foldable without `F: Functor`.

---

## 5. Hoist Requires F: Functor

### Description

`Coyoneda::hoist` applies a natural transformation `F ~> G` by lowering to `F A`, applying
the transformation, then lifting into `Coyoneda<G, A>`:

```rust
pub fn hoist<G>(self, nat: impl NaturalTransformation<F, G>) -> Coyoneda<'a, G, A>
where F: Functor {
    Coyoneda::lift(nat.transform(self.lower()))
}
```

PureScript's `hoistCoyoneda` does not require `Functor f` because it applies the natural
transformation directly to the hidden `F B` and re-wraps with the same accumulated function.
The Rust implementation cannot do this because a `hoist_inner<G>` method would be generic
over `G`, making the trait non-dyn-compatible.

Additionally, the current implementation discards all accumulated mapping layers by lowering
and then re-lifting into a base layer. If there were k pending maps, they are all eagerly
applied during `lower`, then the result is wrapped in a fresh `CoyonedaBase`. This means
`hoist` forces evaluation of all deferred maps.

### Approaches

**A. Accept the current implementation.** `hoist` is a specialized operation; requiring
`F: Functor` is a reasonable trade-off.

**B. Use a wrapper struct.** Create a `HoistedCoyoneda` struct that stores the original
`Coyoneda<F, A>` plus the natural transformation, and lowers through both at `lower` time.
However, the natural transformation `F ~> G` must be applied to `F B` where `B` is hidden,
which requires the same dyn-compatibility bypass.

**C. If unsafe erasure is adopted for map fusion (issue 1A), hoist can piggyback.** Once the
existential type is erased via raw pointers, applying a natural transformation to the erased
`F B` becomes possible without generic methods on the trait.

### Recommendation

Approach A. The `F: Functor` requirement for `hoist` is a minor limitation. If map fusion
via type erasure is ever implemented, `hoist` should be revisited at that time.

---

## 6. Stack Overflow Risk with Deeply Nested Layers

### Description

Each `map` call wraps the previous `Box<dyn CoyonedaInner>` in a new `CoyonedaMapLayer`.
At `lower` time, each layer calls `self.inner.lower()` recursively:

```rust
fn lower(self: Box<Self>) -> ... {
    let lowered = self.inner.lower();  // recursive call
    F::map(self.func, lowered)
}
```

For k chained maps, `lower` makes k nested function calls. This is a linear recursion
with depth proportional to k. For very large k (thousands of chained maps), this will
overflow the stack.

The test `many_chained_maps` only tests 100 layers. On a typical Rust stack (8 MB), the
actual limit depends on frame size, but 100,000 chained maps would likely cause a stack
overflow.

### Approaches

**A. Iterative lowering with a loop.** Change the `lower` protocol: instead of recursive
`CoyonedaInner::lower`, have the outer `Coyoneda::lower` iteratively peel layers. This
requires distinguishing `CoyonedaBase` from `CoyonedaMapLayer` at the outer level, which is
not possible through the current trait-object interface without adding a method that returns
an enum discriminant or downcasting.

A practical variant: add a method `fn try_peel(self: Box<Self>) -> Either<F<A>, (Box<dyn CoyonedaInner<F, B>>, Box<dyn Fn(B) -> A>)>`
to the inner trait. But the `B` in the return type is existential, so this cannot be expressed
directly.

**B. Trampoline the recursion.** Use the library's own `Trampoline` type to make `lower`
stack-safe. Each layer returns a `Continue` with the next step instead of recursing. This
requires `'static` bounds (a `Trampoline` limitation), which conflicts with the `'a` lifetime
parameter.

**C. Accumulate functions into a Vec at map time.** Instead of nesting layers, store all
mapping functions in a flat list. At `lower` time, compose them iteratively and apply once.
This requires type erasure for the intermediate types (same challenge as issue 1).

**D. Document the limitation.** Add documentation noting that Coyoneda is not stack-safe for
deeply nested maps and recommend composing functions before mapping for such cases.

### Recommendation

Approach D in the near term. The typical use case for Coyoneda involves a modest number of
chained maps (single digits to low hundreds), where stack depth is not a concern. If map
fusion (issue 1) is implemented, the recursion problem disappears because there would be only
one layer regardless of how many maps were chained. For now, documenting the limitation is
sufficient.

---

## 7. No Send or Sync Support

### Description

`Coyoneda` wraps `Box<dyn CoyonedaInner<'a, F, A> + 'a>`, which is neither `Send` nor `Sync`.
The stored `Box<dyn Fn(B) -> A + 'a>` in `CoyonedaMapLayer` is also not `Send`. This means
Coyoneda values cannot be shared across threads, even when the underlying `F A` is `Send`.

The design document proposes a `SendCoyoneda` variant with
`Box<dyn CoyonedaInner + Send + 'a>`, mirroring the `Thunk`/`SendThunk` split.

### Approaches

**A. Create a `SendCoyoneda` type.** A parallel type with `Send` bounds on the trait object
and stored functions, following the `Thunk`/`SendThunk` pattern.

- Pro: Follows existing library conventions; provides thread safety.
- Con: Code duplication between `Coyoneda` and `SendCoyoneda`. Every method must be
  duplicated or generated via macro.

**B. Parameterize over pointer brand.** Make `Coyoneda<'a, F, A, P>` generic over a pointer
brand `P`, where `P` controls whether the inner storage is `Box` (non-Send), `Box + Send`,
etc. This follows the `FnBrand<P>` pattern.

- Pro: Single implementation, DRY.
- Con: More complex type signatures; the pointer brand abstraction does not directly map to
  `Send` bounds on trait objects (it maps to `Rc` vs `Arc` for shared ownership).

**C. Defer until needed.** Thread safety for Coyoneda is a niche requirement; most use cases
are single-threaded map fusion.

### Recommendation

Approach A when the need arises. The `Thunk`/`SendThunk` pattern is well-established in the
library and provides a clear, predictable API. Approach C is fine for now.

---

## 8. No Clone Support

### Description

`Box<dyn CoyonedaInner>` is not `Clone`. This prevents implementing `Traversable` (which
requires `Self::Of<'a, B>: Clone`), `Semiapplicative`, and other type classes that need to
duplicate the structure.

The design document proposes an `Rc`/`Arc`-wrapped variant where the inner trait object is
behind a reference-counted pointer, enabling `Clone` via `Rc::clone`.

### Approaches

**A. Rc/Arc hybrid variant.** Wrap the inner in `Rc<dyn CoyonedaInner>` (or
`Arc<dyn CoyonedaInner>` for Send). Clone is then cheap pointer copy. The accumulated
functions would also need `Rc`/`Arc` wrapping (i.e., `Rc<dyn Fn(B) -> A>` via
`FnBrand<RcBrand>`).

- Pro: Enables Clone, Traversable, Semiapplicative.
- Con: Semantic subtlety: cloning shares the inner layers by reference. Two clones that are
  independently mapped diverge only in their outermost layers, sharing the inner structure.
  This is correct but may surprise users expecting deep copies.

**B. Clone by lowering.** Implement `Clone` on `Coyoneda` by lowering to `F A` (requiring
`F: Functor + Clone for F::Of<A>`) and re-lifting. This is semantically clean but expensive
for deeply layered Coyoneda values.

- Pro: Simple, no Rc/Arc overhead.
- Con: Requires `F: Functor` for Clone; expensive; defeats deferred evaluation.

### Recommendation

Approach A, when `Traversable` or `Semiapplicative` support is needed. The Rc/Arc variant
should be a distinct type (e.g., `SharedCoyoneda<'a, F, A, P>`) to keep the base `Coyoneda`
lightweight and avoid imposing reference-counting overhead on users who do not need Clone.

---

## 9. Identity Function Allocation in Lift

### Description

`Coyoneda::lift` does not allocate a function box, because `CoyonedaBase` stores only `fa`
and returns it directly from `lower`. This is good. However, the design document's open
question 2 asks whether `lift` could avoid the identity function allocation. In the
current implementation, this is already addressed: `CoyonedaBase` has no function at all.

There is, however, a minor inefficiency: when a `CoyonedaBase` layer is wrapped in a
`CoyonedaMapLayer`, the `lower` call on `CoyonedaBase` returns `fa` without calling `F::map`,
but then `CoyonedaMapLayer::lower` calls `F::map(self.func, lowered)`. This is optimal for
a single map layer. For the specific case of `lift(fa).lower()` with no maps, no `F::map`
is called at all. This is correct and well-optimized.

No action needed. This is noted here for completeness.

---

## 10. Missing Type Class Instances

### Description

The implementation provides `Functor`, `Pointed`, and `Foldable`. PureScript's Coyoneda also
provides `Apply`, `Applicative`, `Bind`, `Monad`, `Traversable`, `Extend`, `Comonad`, `Eq`,
`Ord`, and others. The design document defers these.

For `Semiapplicative`, `Applicative`, `Semimonad`, and `Monad`, the standard approach is to
lower both sides, apply the operation via the underlying `F`, and re-lift. This works but
requires `F: Applicative` or `F: Monad` and triggers eager evaluation, losing the
deferred-map benefit.

For `Eq` and `Ord`, lowering and comparing is straightforward but requires `F: Functor`.

For `Debug`, lowering to `F A` and printing requires `F: Functor + Debug`. Alternatively, a
structural debug showing "Coyoneda(k layers)" without lowering is possible.

### Approaches

**A. Implement lower-and-delegate instances.** For each missing type class, lower to `F`,
perform the operation, re-lift. Simple, correct, but loses deferred evaluation.

**B. Implement selectively.** Only add instances that are commonly needed. `Eq` and `Debug`
are high-value for debugging; `Monad` is useful for interoperability.

**C. Defer until needed.** The current set (`Functor`, `Pointed`, `Foldable`) covers the
primary use case.

### Recommendation

Approach B. Prioritize `Debug` (for usability), `Eq` (for testing), and `Semiapplicative`/
`Applicative` (for composability). `Monad` can follow once Applicative is in place. Each
requires `F: Functor` for lowering, which is acceptable since the deferred-map property is
inherently a Functor-level optimization.

---

## Summary of Recommendations

| Issue                         | Severity | Recommendation                                         |
| ----------------------------- | -------- | ------------------------------------------------------ |
| 1. No map fusion              | High     | Provide `FunctorPipeline` as a separate zero-cost API. |
| 2. Fn vs FnOnce               | Low      | Accept; forced by `Functor::map` signature.            |
| 3. Double allocation in `new` | Low      | Add `CoyonedaImpl` struct for single-box `new`.        |
| 4. Foldable requires Functor  | Medium   | Accept; document the divergence from PureScript.       |
| 5. Hoist requires Functor     | Low      | Accept; revisit if type erasure is adopted.            |
| 6. Stack overflow risk        | Medium   | Document the limitation.                               |
| 7. No Send/Sync               | Medium   | Add `SendCoyoneda` when needed.                        |
| 8. No Clone                   | Medium   | Add Rc/Arc variant when Traversable is needed.         |
| 9. Lift identity allocation   | None     | Already optimized via `CoyonedaBase`.                  |
| 10. Missing type classes      | Low      | Add `Debug`, `Eq`, `Applicative` incrementally.        |
