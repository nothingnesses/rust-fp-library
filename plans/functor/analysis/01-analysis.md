# Coyoneda Implementation Analysis

Date: 2026-03-31

This document analyzes the Coyoneda implementation in `fp-library/src/types/coyoneda.rs`,
identifies flaws, issues, and limitations, and proposes approaches to address each one.

---

## Table of Contents

1. [No map fusion: the core promise is undelivered](#1-no-map-fusion-the-core-promise-is-undelivered)
2. [Foldable requires F: Functor, diverging from the free functor contract](#2-foldable-requires-f-functor)
3. [hoist requires F: Functor, losing the natural transformation property](#3-hoist-requires-f-functor)
4. [Stack overflow risk from deeply nested layers](#4-stack-overflow-risk-from-deeply-nested-layers)
5. [Fn trait used where FnOnce would suffice](#5-fn-trait-used-where-fnonce-would-suffice)
6. [new constructor allocates unnecessarily](#6-new-constructor-allocates-unnecessarily)
7. [No Send or Sync support](#7-no-send-or-sync-support)
8. [No Clone support, blocking Traversable and Applicative](#8-no-clone-support)
9. [Design document describes fusion that does not exist](#9-design-document-describes-fusion-that-does-not-exist)
10. [Missing property-based tests for functor laws](#10-missing-property-based-tests)

---

## 1. No map fusion: the core promise is undelivered

### Description

The entire motivation for Coyoneda, as stated in the design document, is map fusion:
accumulating chained `map` calls as function composition so that `lower` performs a single
`F::map` call regardless of how many maps were chained. The implementation does not achieve
this. After k chained maps, `lower` calls `F::map` exactly k times, which is the same cost
as calling `F::map` directly k times without Coyoneda.

The layered encoding (`CoyonedaBase` + `CoyonedaMapLayer`) wraps each `map` call in a new
trait-object layer. At `lower` time, each layer calls `self.inner.lower()` (recursively
unwinding to the base), then applies its own function via `F::map`. For `VecBrand` with n
elements and k maps, this is O(k \* n) traversal work, identical to direct chaining but with
additional heap allocation overhead (k `Box<dyn CoyonedaInner>` + k `Box<dyn Fn>` allocations).

This means the implementation is strictly worse than direct `map` chaining for performance,
while the design document's motivation section describes the opposite. The module documentation
does acknowledge this in the "Performance characteristics" section, but the tension between the
stated motivation and actual behavior is a significant concern.

### Approaches

**A. Enum-based existential with unsafe pointer erasure.** Replace the trait object with an
enum that stores the original `fb` as type-erased raw pointer data alongside a composed
function chain. At `lower` time, reconstruct the typed value and apply the single composed
function. This achieves true fusion without `'static` but requires `unsafe` code.

- Trade-off: True fusion, but introduces `unsafe` that must be carefully audited. The
  type-erased data must maintain proper alignment and drop semantics.
- Risk: Soundness bugs in the erasure/reconstruction logic.

**B. Generic B parameter (FunctorPipeline).** Expose the existential type B as a type
parameter: `FunctorPipeline<'a, F, B, A>`. Each `.map(g)` changes the type. True fusion, zero
allocations, fully safe.

- Trade-off: Cannot participate in HKT (the extra type parameter prevents `impl_kind!`).
  Cannot be used with `CoyonedaBrand<F>` or the `Functor` trait.
- This is a complementary API, not a replacement.

**C. Accept the limitation and reframe the purpose.** Acknowledge that Rust's type system
prevents the canonical Coyoneda fusion. Reposition the type as providing HKT integration
(Functor for any F) and deferred execution, not performance optimization.

- Trade-off: The type loses its primary advertised benefit. Users seeking fusion must compose
  functions manually or use a FunctorPipeline builder.

**Recommendation:** Implement approach B as a separate `FunctorPipeline` type for users who
need actual fusion. Pursue approach C for the existing `Coyoneda`, clearly reframing its
purpose in documentation. Approach A could be explored later if there is strong demand for
fused HKT-integrated Coyoneda, but the `unsafe` burden is significant.

---

## 2. Foldable requires F: Functor

### Description

The `Foldable` implementation for `CoyonedaBrand<F>` requires `F: Functor + Foldable + 'static`
(line 533). In PureScript, `Foldable` for `Coyoneda` only requires `Foldable f`, because
`unCoyoneda` opens the existential to compose the fold function with the accumulated mapping
function, folding the original `F B` in a single pass without ever calling `F::map`.

The current implementation lowers first (which requires `F: Functor` to apply each layer's
`F::map`), then folds the resulting `F A`. This means:

- `CoyonedaBrand<F>` cannot be `Foldable` unless `F` is also a `Functor`.
- The fold performs k calls to `F::map` followed by a fold, rather than a single fold pass.

The design document identifies this issue and notes it is caused by dyn-compatibility: a
`fold_map_inner` method on `CoyonedaInner` would need to be generic over `M: Monoid` and
`FnBrand: CloneableFn`, making the trait not dyn-compatible.

### Approaches

**A. Enum dispatch with known monoid types.** Instead of a generic `fold_map_inner<M>`,
define `fold_map_inner` to return a type-erased result and use downcasting. Requires `'static`
on the monoid type.

- Trade-off: Loses lifetime polymorphism on the monoid, which may be acceptable since most
  monoid types are `'static` in practice.

**B. Visitor pattern.** Define a `FoldVisitor` trait with a non-generic `visit` method that
the inner layer calls back with the element. The visitor accumulates the fold result internally.

- Trade-off: Requires boxing the visitor and adds indirection. The visitor must be
  dyn-compatible itself, which constrains its design.

**C. Two-phase approach: compose functions in the Coyoneda layers at map time.** If each
`CoyonedaMapLayer` stored its function in a way that could be retrieved and composed into a
single function at fold time (without needing a generic method on the trait), fold could apply
the composed function during a single traversal.

- Trade-off: Requires the layers to expose their functions in a type-erased way, which circles
  back to the same dyn-compatibility problem.

**D. Accept the limitation.** Keep the `F: Functor` requirement and document it as a known
divergence from PureScript.

- Trade-off: Types that are `Foldable` but not `Functor` cannot benefit from Coyoneda's
  Foldable instance. In practice, most Foldable types in the library are also Functors, so the
  impact is limited.

**Recommendation:** Approach D is pragmatic for now. The set of types that are `Foldable`
but not `Functor` is small in this library. If a `FunctorPipeline` type is built (see issue 1),
it can provide fused folds natively since its B parameter is visible.

---

## 3. hoist requires F: Functor

### Description

The `hoist` method (line 443) lowers the Coyoneda, applies the natural transformation to the
lowered `F A`, then re-lifts into `Coyoneda<G, A>`. This requires `F: Functor` because lowering
applies all accumulated map layers via `F::map`.

In PureScript, `hoistCoyoneda` applies the natural transformation directly to the hidden `F B`
via `unCoyoneda`, which does not require `F: Functor`. The accumulated function `B -> A` is
preserved and re-wrapped around the transformed `G B`.

The current approach also loses deferred execution: all accumulated maps are eagerly applied
during the hoist, then the result is wrapped in a fresh `CoyonedaBase` with no pending maps.

### Approaches

**A. Add a `hoist_inner` method to `CoyonedaInner`.** This method would be
`fn hoist_inner(self: Box<Self>, nat: &dyn NatTransAny<F, G>) -> Box<dyn CoyonedaInner<'a, G, A>>`
where `NatTransAny` is a non-generic wrapper trait. The base layer applies the nat to its `fa`,
and map layers recursively hoist their inner, preserving the function layers.

- Trade-off: Requires a dyn-compatible natural transformation wrapper. The `NaturalTransformation`
  trait's `transform` method is generic over `A`, so it cannot be used as a trait object directly.
  A wrapper that erases the type parameter (via `Any` or unsafe) would be needed.

**B. Accept the limitation.** The `F: Functor` requirement is only triggered when `hoist` is
called. Users who need `hoist` on non-Functor types are an edge case.

- Trade-off: Diverges from PureScript semantics. Loses deferred execution across hoist
  boundaries.

**Recommendation:** Approach B. The `hoist` operation is relatively niche, and requiring
`F: Functor` is a reasonable constraint in Rust. The loss of deferred execution during hoist is
minor since hoist is typically called once, not in a tight loop.

---

## 4. Stack overflow risk from deeply nested layers

### Description

Each call to `map` adds a new `CoyonedaMapLayer` wrapping the previous one. At `lower` time,
the layers are unwound recursively: `CoyonedaMapLayer::lower` calls `self.inner.lower()`
(which calls the next layer's `lower`, and so on) until reaching `CoyonedaBase`. For k chained
maps, this produces a call stack k frames deep.

The test `many_chained_maps` (line 672) chains 100 maps, which is fine. But a user chaining
thousands or millions of maps (for example, in a loop building a transformation pipeline) would
overflow the stack.

This is analogous to the problem that `Free` monad implementations solve with trampolining,
but `Coyoneda::lower` has no such mechanism.

### Approaches

**A. Iterative lowering.** Restructure the inner representation to use a `Vec` of boxed
functions instead of nested trait objects. At `lower` time, iterate through the vec and compose
all functions into one, then apply a single `F::map`. This simultaneously solves the stack
overflow issue and achieves map fusion.

- Trade-off: Changes the internal representation significantly. A `Vec<Box<dyn Any>>` approach
  would require type erasure and `'static` bounds. A `Vec` of function layers with the same
  input/output type is not possible because each function has a different type signature.

**B. Trampoline the lower operation.** Convert the recursive `lower` into a loop using an
explicit stack (a `Vec` of closures or function pointers).

- Trade-off: Requires significant refactoring of `CoyonedaInner`. The heterogeneous types of
  each layer's function make it difficult to collect them into a homogeneous container without
  type erasure.

**C. Document the limitation and recommend bounded usage.** Note in the documentation that
Coyoneda is not designed for thousands of chained maps. For such cases, recommend manual
function composition.

- Trade-off: Users must be aware of the limit. Not a safety hazard since Rust stack overflows
  are defined behavior (abort, not UB), but it is still undesirable.

**Recommendation:** Approach C for the short term. The practical use case for thousands of
chained maps on Coyoneda is limited; users building large pipelines programmatically should
compose functions directly. If approach A (Vec-based representation) can be made to work with
the type system, it would be the ideal long-term solution since it also provides fusion.

---

## 5. Fn trait used where FnOnce would suffice

### Description

The `CoyonedaMapLayer` stores its function as `Box<dyn Fn(B) -> A + 'a>` (line 215), using
the `Fn` trait. Since `lower` consumes `self: Box<Self>` (taking ownership), each function is
called at most once per element. For types like `OptionBrand` that contain at most one element,
`FnOnce` would be sufficient and more permissive (accepting closures that capture owned values).

However, for types like `VecBrand` that contain multiple elements, the function must be called
multiple times, requiring `Fn`. The `Functor::map` trait method takes `impl Fn(A) -> B + 'a`,
not `FnOnce`, confirming that `Fn` is the correct bound for general use.

The `Coyoneda::map` inherent method (line 394) also takes `impl Fn(A) -> B + 'a`, matching the
trait's signature. This is consistent but means closures that move owned data into the mapping
function must ensure the data is `Clone`.

### Approaches

**A. Keep `Fn` as-is.** The `Functor` trait requires `Fn`, and Coyoneda must be compatible with
multi-element containers. Using `Fn` is correct.

**B. Provide a `map_once` variant for single-element contexts.** A separate method that accepts
`FnOnce` and is only valid when the underlying functor is known to contain at most one element.

- Trade-off: The type system cannot enforce the "at most one element" constraint generically.
  This would need to be an unsafe or specially-bounded API.

**Recommendation:** Approach A. The `Fn` bound is correct for the general case and consistent
with the library's `Functor` trait. No change needed.

---

## 6. new constructor allocates unnecessarily

### Description

The `new` method (line 306) creates a `CoyonedaMapLayer` wrapping a `CoyonedaBase`:

```rust
pub fn new<B: 'a>(f: impl Fn(B) -> A + 'a, fb: ...) -> Self {
    Coyoneda(Box::new(CoyonedaMapLayer {
        inner: Box::new(CoyonedaBase { fa: fb }),
        func: Box::new(f),
    }))
}
```

This performs 3 heap allocations: one for the outer `Box<dyn CoyonedaInner>` (the MapLayer),
one for the inner `Box<dyn CoyonedaInner>` (the Base), and one for `Box<dyn Fn>` (the
function). The design document notes that `new` "saves one box allocation vs `lift(fb).map(f)`,"
which is correct (lift + map would be 3 boxes total: 1 for lift's Base, 1 for map's MapLayer,
1 for map's function). But `new` still creates the unnecessary intermediate `CoyonedaBase`.

A single-layer struct that holds both the function and the value directly (like the
`CoyonedaImpl` described in the design document but not implemented) would need only 2
allocations: one for the outer trait object box and one for the function box.

### Approaches

**A. Add a `CoyonedaSingle` struct.** A new struct that holds both `fb` and `func` directly,
implementing `CoyonedaInner`. Its `lower` calls `F::map(self.func, self.fb)` in one step.

```
struct CoyonedaSingle<'a, F, B, A> {
    fb: F::Of<'a, B>,
    func: Box<dyn Fn(B) -> A + 'a>,
}
```

Use this in `new` instead of nesting `CoyonedaMapLayer` around `CoyonedaBase`. This saves one
`Box` allocation per `new` call.

- Trade-off: Adds another struct implementing `CoyonedaInner`, slightly increasing code
  complexity. The vtable grows by one entry (one more implementor).

**B. Keep the current approach.** The extra allocation is a pointer-sized box. For most use
cases the overhead is negligible.

- Trade-off: Slightly wasteful but simple.

**Recommendation:** Approach A. It is a small, localized change that eliminates a redundant
allocation. The `CoyonedaSingle` struct is essentially what the design document calls
`CoyonedaImpl`, and it would make `new` allocate 2 boxes instead of 3.

---

## 7. No Send or Sync support

### Description

`Coyoneda` wraps `Box<dyn CoyonedaInner<'a, F, A> + 'a>`, which is neither `Send` nor `Sync`.
The stored functions (`Box<dyn Fn(B) -> A + 'a>`) are also not `Send`. This means `Coyoneda`
values cannot be shared across threads or sent to other threads for lowering.

The library has established patterns for thread-safe variants: `Thunk`/`SendThunk`,
`RcLazy`/`ArcLazy`, `RcFnBrand`/`ArcFnBrand`. Coyoneda does not follow this pattern.

### Approaches

**A. Add `SendCoyoneda` as a separate type.** Mirror the `Thunk`/`SendThunk` split:

```
pub struct SendCoyoneda<'a, F, A: 'a>(
    Box<dyn CoyonedaInner<'a, F, A> + Send + 'a>,
);
```

With a corresponding `SendCoyonedaBrand<F>` and `SendCoyonedaInner` trait that adds `Send`
bounds on the function boxes.

- Trade-off: Code duplication between the two variants. Could be mitigated with a macro or
  generic parameter (similar to `LazyBrand<Config>`).

**B. Parameterize over a pointer/config type.** Similar to how `LazyBrand<RcLazyConfig>` and
`LazyBrand<ArcLazyConfig>` share code, define a `CoyonedaConfig` trait that controls whether
the function boxes are `Send`.

- Trade-off: More complex type machinery, but avoids duplication.

**C. Defer until needed.** Thread-safe Coyoneda is only useful if users are building
transformation pipelines on one thread and lowering on another, which is not a common pattern.

**Recommendation:** Approach C for now. If users request thread-safe Coyoneda, approach B
(parameterized config) would be the most consistent with the library's existing patterns.

---

## 8. No Clone support

### Description

`Coyoneda` is not `Clone` because `Box<dyn CoyonedaInner>` is not `Clone` and `Box<dyn Fn>` is
not `Clone`. This prevents implementing `Traversable` (which requires `Self::Of<'a, B>: Clone`)
and `Semiapplicative` (which requires cloning one argument).

The design document proposes an `Rc`/`Arc`-wrapped variant where the inner trait object is
wrapped in `Rc<dyn CoyonedaInner>` (cloneable via reference counting) and functions use
`Rc<dyn Fn>` (via `FnBrand<RcBrand>`).

### Approaches

**A. Rc/Arc hybrid variant.** As described in the design document's "Future Extensions" section.
Wrap the inner in `Rc`/`Arc` and use `FnBrand<P>` for functions.

- Trade-off: Clone is shallow (reference count increment), but lowering a cloned value shares
  mutation state with the original. For `Coyoneda` this is acceptable since `lower` is a pure
  operation that consumes the value.
- Note: `Rc::try_unwrap` could be used to avoid unnecessary reference counting when there is
  only one owner, but this adds complexity.

**B. Explicit `clone_inner` method on the trait.** Each implementor of `CoyonedaInner` provides
a method to clone itself into a new `Box<dyn CoyonedaInner>`. The base clones `fa` (requiring
`F::Of<'a, A>: Clone`), and map layers clone the inner and the function.

- Trade-off: Requires `F::Of<'a, A>: Clone` and function `Clone`, which is restrictive. Would
  need `CloneableFn` wrappers for the functions.

**Recommendation:** Approach A when Clone support is needed. The `Rc`/`Arc` hybrid is the
standard solution in the library and avoids deep-cloning the entire layer stack.

---

## 9. Design document describes fusion that does not exist

### Description

The design document at `plans/functor/coyoneda-design.md` contains two conflicting narratives.
The earlier sections (Motivation, What is Coyoneda, Chosen Encoding) describe and promise map
fusion. The later sections (Implementation Notes, Key Findings) explain that fusion was not
achieved. While the implementation notes are accurate, the document structure is misleading
because a reader encountering the earlier sections may form incorrect expectations.

Specific examples:

- The Motivation section says "Coyoneda automates this" (referring to function composition).
- The Chosen Encoding section says "Each map composes the function" and describes an allocation
  profile showing 0 calls to `F::map` at lower time (plus whatever `F::map` allocates).
- The "Concrete Implementation" subsection shows `CoyonedaImpl` with `map_inner` that composes
  functions, which is the unfused design that was rejected.

The actual allocation profile table at the bottom of the document corrects this, but it appears
after 600 lines of content that implies fusion works.

### Approaches

**A. Restructure the document.** Move the "Key Findings" and actual implementation details
to the top. Clearly mark the earlier sections as "original design" vs "implemented design."

**B. Split into two documents.** One for the theoretical design and analysis, one for the
actual implementation decisions and their rationale.

**C. Add a prominent note at the top.** A brief section at the start of the document that
states the implementation does not achieve fusion, with a forward reference to the explanation.

**Recommendation:** Approach C is the least disruptive. A note at the top of the design
document would prevent misunderstanding without requiring a full restructure.

---

## 10. Missing property-based tests

### Description

The test suite contains 23 unit tests that verify specific cases, but lacks property-based
tests (QuickCheck or proptest) for the functor laws. The two law tests (`functor_identity_law`
and `functor_composition_law`) each use a single hardcoded input (`vec![1, 2, 3]`). The
project's testing strategy (per CLAUDE.md) explicitly calls for property-based tests for type
class laws.

Property-based tests would verify:

- Functor identity: `map(id, fa).lower() == fa` for arbitrary `fa`.
- Functor composition: `map(f . g, fa).lower() == map(f, map(g, fa)).lower()` for arbitrary
  `f`, `g`, `fa`.
- Foldable consistency: `fold_map(f, coyo) == fold_map(f, coyo.lower())` for arbitrary `f`,
  `coyo`.
- Pointed/lift roundtrip: `pure(a).lower() == F::pure(a)` for arbitrary `a`.

### Approaches

**A. Add QuickCheck property tests.** Use the project's existing QuickCheck infrastructure to
generate arbitrary values and verify laws.

- Trade-off: QuickCheck tests for Coyoneda require generating arbitrary `Coyoneda` values,
  which means generating arbitrary functions. QuickCheck does not generate arbitrary functions
  easily, but specific function families (linear, polynomial) can be used.

**B. Add parameterized unit tests.** Test the laws with a variety of inputs (empty containers,
single elements, large containers, various types) without full property-based testing.

- Trade-off: Less coverage than property-based tests, but simpler to implement and still a
  meaningful improvement over single-input tests.

**Recommendation:** Approach A for the functor laws and foldable consistency, using standard
QuickCheck strategies for function generation. Approach B as a supplement for edge cases
(empty containers, None values) that QuickCheck might not generate frequently.

---

## Summary of Recommendations

| Issue                      | Severity | Recommendation                                                |
| -------------------------- | -------- | ------------------------------------------------------------- |
| 1. No map fusion           | High     | Build FunctorPipeline for fusion; reframe Coyoneda's purpose. |
| 2. Foldable needs Functor  | Medium   | Accept and document the divergence from PureScript.           |
| 3. hoist needs Functor     | Low      | Accept and document.                                          |
| 4. Stack overflow risk     | Medium   | Document the limitation; consider Vec-based repr long-term.   |
| 5. Fn vs FnOnce            | None     | No change needed; Fn is correct.                              |
| 6. new allocates extra     | Low      | Add CoyonedaSingle struct to save one allocation.             |
| 7. No Send/Sync            | Low      | Defer until needed; use parameterized config when ready.      |
| 8. No Clone                | Medium   | Rc/Arc variant when Traversable is needed.                    |
| 9. Misleading design doc   | Medium   | Add prominent note at document top.                           |
| 10. Missing property tests | Medium   | Add QuickCheck property tests for type class laws.            |
