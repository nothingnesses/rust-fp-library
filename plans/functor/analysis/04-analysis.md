# Coyoneda Implementation Analysis

Date: 2026-03-31

This document independently analyzes the Coyoneda implementation in
`fp-library/src/types/coyoneda.rs` and the associated design document at
`plans/functor/coyoneda-design.md`. Each section identifies an issue, proposes
approaches to address it, discusses trade-offs, and gives a recommendation.

---

## Table of Contents

1. [No map fusion: layered lower calls F::map k times](#1-no-map-fusion)
2. [Fn trait used where FnOnce would suffice](#2-fn-trait-used-where-fnonce-would-suffice)
3. [new constructor performs redundant double-boxing](#3-new-constructor-performs-redundant-double-boxing)
4. [hoist requires F: Functor due to lower-then-relift strategy](#4-hoist-requires-f-functor)
5. [Foldable requires F: Functor, diverging from PureScript](#5-foldable-requires-f-functor)
6. [Stack overflow risk with deeply nested map layers](#6-stack-overflow-risk-with-deeply-nested-map-layers)
7. [No Clone, Send, Sync, or Debug](#7-no-clone-send-sync-or-debug)
8. [Identity function allocation in lift](#8-identity-function-allocation-in-lift)
9. [Missing type class instances](#9-missing-type-class-instances)
10. [Foldable lower-then-fold defeats single-pass optimization](#10-foldable-lower-then-fold-defeats-single-pass-optimization)

---

## 1. No Map Fusion

### Description

The central promise of Coyoneda in the functional programming literature is map
fusion: accumulating mapped functions via composition so that `lower` calls
`F::map` exactly once regardless of how many maps were chained. This
implementation does not achieve fusion. After k chained maps, `lower` calls
`F::map` k times (once per `CoyonedaMapLayer`), which is identical in
asymptotic cost to calling `F::map` directly k times.

The root cause is Rust's dyn-compatibility constraint. True fusion requires a
`map_inner<C>` method on `CoyonedaInner` that composes `f: A -> C` with the
stored `g: B -> A` to produce `h: B -> C`. Because `C` is a generic type
parameter, this method cannot appear on a trait object.

### Approaches

**A. Unsafe pointer erasure.** Erase the existential type `B` via raw pointer
casts. Store `fb` as an erased pointer alongside a composed function
`*const () -> A` that knows how to reconstruct the type at `lower` time. This
achieves true fusion without the `'static` restriction.

- Pro: True single-pass fusion; retains lifetime polymorphism and HKT integration.
- Con: Requires `unsafe` code with careful soundness reasoning; increases maintenance burden; any mistake is unsound.

**B. `Box<dyn Any>` erasure.** Erase `fb` and the composed function via `Any`,
downcast at `lower` time. Achieves true fusion but requires `'static` on all
types.

- Pro: True fusion; no unsafe code.
- Con: Loses lifetime polymorphism; cannot implement `Kind` (which requires `Of<'a, A: 'a>: 'a`); same limitation as `Free`.

**C. Enum-based composition with a bounded type list.** Instead of a trait
object, use an enum that can hold a composed function chain of up to N levels,
collapsing compositions eagerly within the enum.

- Pro: No unsafe; partial fusion up to N levels.
- Con: Arbitrary bound N; complex implementation; does not generalize.

**D. Accept the limitation; provide `FunctorPipeline` as a zero-cost alternative.**
Keep the current layered implementation for HKT integration and provide a
separate `FunctorPipeline<'a, Brand, B, A, F>` struct that exposes `B` as a
type parameter for zero-cost fusion (no boxing, no dynamic dispatch).

- Pro: Zero-cost for performance-critical paths; clear separation of concerns.
- Con: Two APIs for the same conceptual operation; `FunctorPipeline` cannot participate in HKT.

### Recommendation

Approach D is the most pragmatic. The current Coyoneda provides genuine value
through HKT integration, the free functor property, and a clean abstraction
boundary. Documenting the limitation clearly (which is already done) and
providing `FunctorPipeline` as a companion type gives users the best of both
worlds without introducing unsafety. Approach A could be explored later behind
a feature flag for users who need true fusion with HKT integration, but it
should not be the default.

---

## 2. Fn Trait Used Where FnOnce Would Suffice

### Description

The accumulated mapping function in `CoyonedaMapLayer` is stored as
`Box<dyn Fn(B) -> A + 'a>`, using the `Fn` trait. However, `lower` consumes
the `Coyoneda` by value (`self: Box<Self>`), meaning each function is called
at most once during lowering. The `Fn` bound is strictly stronger than
necessary; `FnOnce` would suffice for the lowering path.

The `Fn` bound exists because `Functor::map` in the library takes
`impl Fn(A) -> B`, not `impl FnOnce(A) -> B`. This is a library-wide design
choice (uncurried semantics with `impl Fn` for zero-cost abstractions). But it
means closures that capture non-Clone, non-Copy values by move cannot be used
as mapping functions in Coyoneda, even though they would only ever be called
once per element.

The `Fn` bound is also what forces `CoyonedaMapLayer::lower` to call
`F::map(self.func, ...)` where `self.func` is `Box<dyn Fn(B) -> A>`. Since
`Functor::map` takes `impl Fn`, this works. But the double indirection (the
caller's closure is boxed into `Box<dyn Fn>`, then `F::map` receives
`Box<dyn Fn>` which itself is `impl Fn`) means there is a layer of dynamic
dispatch on every element during `F::map`.

### Approaches

**A. Keep `Fn` for consistency.** The library universally uses `impl Fn` for
mapping functions. Changing Coyoneda alone would create an inconsistency.

- Pro: Consistent API surface; no surprises for users familiar with the library.
- Con: Prevents move-only closures in mapping functions.

**B. Use `FnOnce` internally, convert at boundaries.** Store `Box<dyn FnOnce(B) -> A>`
internally, but accept `impl Fn(A) -> B` in the public `map` method (as
required by `Functor::map`). This relaxes the internal storage without
changing the public API.

- Pro: Internal optimization; no API change.
- Con: `FnOnce` closures are harder to compose (can only be called once, but `F::map` needs to call the function per element). This fundamentally does not work for multi-element containers.

**C. Accept the limitation.** `Fn` is correct here because `F::map` may call
the function multiple times (once per element in `Vec`, for instance). `FnOnce`
would only be correct for single-element containers like `Option` or `Identity`.

### Recommendation

Approach C is correct. On further analysis, this is not actually a flaw. The
`Fn` bound is required because `F::map` calls the function once per element,
and for multi-element containers like `Vec`, that means multiple calls. The
`FnOnce` suggestion is a misunderstanding of the semantics. The current
implementation is correct. However, the dynamic dispatch overhead through
`Box<dyn Fn>` at each layer is a genuine (if small) cost; this is inherent to
the trait-object encoding and cannot be eliminated without changing the
approach.

---

## 3. new Constructor Performs Redundant Double-Boxing

### Description

The `new` constructor creates a `CoyonedaMapLayer` wrapping a `CoyonedaBase`:

```rust
pub fn new<B: 'a>(
    f: impl Fn(B) -> A + 'a,
    fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
) -> Self {
    Coyoneda(Box::new(CoyonedaMapLayer {
        inner: Box::new(CoyonedaBase { fa: fb }),
        func: Box::new(f),
    }))
}
```

This allocates three boxes: one for the outer `CoyonedaMapLayer` trait object,
one for the inner `CoyonedaBase` trait object, and one for the function. The
design document notes that `new` "saves one box allocation vs
`lift(fb).map(f)`," which is true (`lift` then `map` would allocate four
boxes). But conceptually, `new(f, fb)` should be equivalent to a single
`CoyonedaImpl { fb, func: Box::new(f) }` from the design document's original
(non-layered) encoding, which would need only two boxes.

The extra box comes from the layered architecture: `CoyonedaMapLayer` stores
its predecessor as `Box<dyn CoyonedaInner>`, so even the base case must be
boxed.

### Approaches

**A. Add a `CoyonedaNewLayer` struct.** Create a third struct that directly
stores `fb` and `func` without an inner trait object, analogous to the design
document's `CoyonedaImpl`. This eliminates the inner `CoyonedaBase` box.

- Pro: Reduces `new` from 3 allocations to 2; matches the design document's intent.
- Con: Adds a third struct implementing `CoyonedaInner`; more code to maintain.

**B. Accept the overhead.** The extra box is pointer-sized. For any non-trivial
`fb` value, this cost is negligible compared to the data being processed.

- Pro: Simpler codebase; fewer types.
- Con: Leaves a known inefficiency.

### Recommendation

Approach A is worthwhile. The `CoyonedaImpl` struct from the design document
already describes this type. Adding it eliminates a heap allocation on every
`new` call and brings the implementation closer to the design document's
original intent. The implementation cost is low (a ~20-line struct with a
`CoyonedaInner` impl).

---

## 4. hoist Requires F: Functor

### Description

The `hoist` method transforms `Coyoneda<F, A>` into `Coyoneda<G, A>` by
lowering to `F`, applying the natural transformation, then lifting into
`Coyoneda<G, _>`:

```rust
pub fn hoist<G>(self, nat: impl NaturalTransformation<F, G>) -> Coyoneda<'a, G, A>
where
    F: Functor,
{
    Coyoneda::lift(nat.transform(self.lower()))
}
```

PureScript's `hoistCoyoneda` does not require `Functor f` because it opens the
existential directly via `unCoyoneda`, applying the natural transformation to
the hidden `F B` and preserving the accumulated function. The Rust version
lowers first (requiring `F: Functor`), applies the transformation to the
fully-evaluated `F A`, then lifts back. This means:

1. All accumulated map layers are materialized (k calls to `F::map`).
2. The resulting `Coyoneda<G, A>` has no deferred maps, losing the
   "accumulated mapping" structure.
3. Users cannot hoist over a type constructor that is not a `Functor`.

### Approaches

**A. Add `hoist_inner` to `CoyonedaInner`.** A `hoist_inner` method would
apply the natural transformation to the stored `fb` directly. However,
`hoist_inner<G>` is generic over `G`, making it not dyn-compatible.

- Pro: Would achieve the PureScript semantics exactly.
- Con: Not possible with trait objects.

**B. Use unsafe type erasure for the natural transformation.** Erase the brand
parameter, apply the transformation inside the impl, reconstruct the types.

- Pro: Achieves hoist without Functor.
- Con: Unsafe; complex soundness argument.

**C. Accept the limitation.** Document that `hoist` requires `F: Functor` and
materializes all layers. This is already done in the implementation.

- Pro: Simple; correct; well-documented.
- Con: Diverges from PureScript semantics; cannot hoist non-Functor types.

**D. Provide a specialized `hoist` on `CoyonedaBase` only.** When no maps have
been chained (the inner is a `CoyonedaBase`), `hoist` could apply the
transformation directly without lowering. This could be exposed as a method on
the return value of `lift` before any `map` calls, using a builder pattern.

- Pro: Covers the common case of `lift(fa).hoist(nat)` without requiring `Functor`.
- Con: Only works before any `map` calls; does not generalize.

### Recommendation

Approach C is acceptable for now. The `hoist` with `F: Functor` is correct and
well-documented. If the unsafe erasure approach (from Issue 1) is ever
implemented for map fusion, the same technique would enable `hoist` without
`Functor` as a side benefit. Until then, the limitation is a known consequence
of dyn-compatibility.

---

## 5. Foldable Requires F: Functor

### Description

The `Foldable` implementation for `CoyonedaBrand<F>` requires
`F: Functor + Foldable`:

```rust
impl<F: Functor + Foldable + 'static> Foldable for CoyonedaBrand<F> {
    fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
        func: impl Fn(A) -> M + 'a,
        fa: Apply!(...),
    ) -> M
    where
        M: Monoid + 'a,
        FnBrand: CloneableFn + 'a,
    {
        F::fold_map::<FnBrand, A, M>(func, fa.lower())
    }
}
```

This lowers first (requiring `F: Functor`), then folds. PureScript only needs
`Foldable f` because it composes the fold function with the accumulated mapping
function via `unCoyoneda`, folding the original `F B` in a single pass.

The design document identifies a `fold_map_inner` method that would compose
the fold function through the layers. This method is generic over `M: Monoid`
and `FnBrand: CloneableFn`, making it not dyn-compatible.

### Approaches

**A. Accept the `F: Functor` requirement.** The current implementation is
correct and well-documented. In practice, most types that are `Foldable` are
also `Functor` in this library.

- Pro: Simple; no additional complexity.
- Con: Diverges from PureScript; prevents folding over non-Functor type constructors.

**B. Use a monomorphized fold method.** Instead of making `fold_map_inner`
generic over `M`, fix the monoid to a specific type (e.g., a boxed
`dyn Any + Monoid`). This would make the method dyn-compatible at the cost
of type erasure.

- Pro: Removes `F: Functor` requirement.
- Con: Requires a `Monoid` trait object, which may not be dyn-compatible itself; `Any` requires `'static`.

**C. Enumerate common monoid types.** Provide `fold_map_inner` variants for a
fixed set of monoid types (e.g., `String`, `Vec`, numeric sums).

- Pro: dyn-compatible for known types.
- Con: Not general; does not scale.

**D. Use the enum approach with a universal monoid wrapper.** Define a
`DynMonoid` trait that is dyn-compatible (no generic methods) and implement
`fold_map_inner` in terms of it.

- Pro: General solution.
- Con: `Monoid` typically has `empty()` and `append()`, which are dyn-compatible. The issue is that `fold_map_inner` needs to produce an `M` from an `A` via the user's function, and that function is generic. The user's `impl Fn(A) -> M` becomes `impl Fn(B) -> M` after composition, but `B` is hidden. The real problem is composing the user's function with the accumulated function across the existential boundary, not the monoid itself.

### Recommendation

Approach A is the right choice. The `F: Functor` requirement is a minor
constraint in practice, and the alternative approaches either introduce
unsafety, lose generality, or require `'static`. The design document already
identifies this as a known dyn-compatibility limitation. If the unsafe erasure
approach for map fusion is ever adopted, it would also resolve this issue.

---

## 6. Stack Overflow Risk with Deeply Nested Map Layers

### Description

Each `map` call wraps the previous value in a new `CoyonedaMapLayer`. At
`lower` time, the layers are unwound recursively:

```rust
fn lower(self: Box<Self>) -> ... {
    let lowered = self.inner.lower();  // recursive call
    F::map(self.func, lowered)
}
```

For k chained maps, `lower` recurses k levels deep. The `many_chained_maps`
test chains 100 maps, which is fine, but for very large k (e.g., 10,000+
maps in a loop), this could overflow the stack.

This is the same class of problem that `Free` / `Trampoline` solves for monadic
computations. There is no trampolining in the Coyoneda `lower` path.

### Approaches

**A. Iterative lowering.** Convert the recursive `lower` to an iterative loop
that peels layers off one at a time. This is difficult because each layer's
`func` has a different type (the existential `B` changes per layer), and the
trait object boundary prevents collecting them into a homogeneous container.

- Pro: Would eliminate stack overflow risk.
- Con: Extremely difficult to implement correctly with the current trait-object design; the types change at each layer.

**B. Trampoline the lower path.** Wrap the recursive call in a `Trampoline`.
This requires `'static` on all types (since `Trampoline` wraps `Free<ThunkBrand, A>`
which requires `'static`).

- Pro: Stack-safe.
- Con: Requires `'static`; loses lifetime polymorphism.

**C. Document the limitation and recommend bounded usage.** Note that chaining
thousands of maps on a single Coyoneda value is not recommended. In practice,
most uses chain a small number of maps (single digits to low dozens).

- Pro: Simple; honest.
- Con: Does not prevent the issue.

**D. Add a `collapse` method that periodically lowers and re-lifts.** Users
could call `collapse()` every N maps to flatten the layer stack. This requires
`F: Functor`.

- Pro: User-controlled; simple implementation.
- Con: Requires `F: Functor`; manual intervention.

### Recommendation

Approach C combined with approach D. Document the stack depth limitation and
provide a `collapse` method for users who need to chain many maps in a loop.
The test suite should include a test that demonstrates the collapse pattern.
Iterative lowering (approach A) is theoretically ideal but impractical with the
current type-level architecture.

---

## 7. No Clone, Send, Sync, or Debug

### Description

`Coyoneda` wraps `Box<dyn CoyonedaInner<'a, F, A> + 'a>`, which is:

- Not `Clone` (trait objects behind `Box` are not cloneable).
- Not `Send` (the trait object bound does not include `Send`).
- Not `Sync` (same reason).
- Not `Debug` (no `Debug` bound on the trait; the inner function is opaque).

This prevents:

- `Traversable` implementation (requires `Self::Of<'a, B>: Clone`).
- `Semiapplicative` / `Semimonad` (may require `Clone`).
- Using Coyoneda in multi-threaded contexts.
- Printing or logging Coyoneda values during debugging.

### Approaches

**A. Rc/Arc hybrid variant.** As described in the design document, wrap the
inner trait object in `Rc` or `Arc` instead of `Box`. This enables `Clone` via
reference counting and, with `Arc`, enables `Send + Sync`.

- Pro: Enables `Clone`, `Send + Sync`; unlocks `Traversable` and `Applicative`.
- Con: Adds reference-counting overhead; two variants to maintain.

**B. Parameterize over pointer brand.** Use the library's existing pointer
abstraction hierarchy (`RefCountedPointer`, `SendRefCountedPointer`) to
parameterize `Coyoneda` over the pointer type.

- Pro: Unified API; follows library conventions (like `FnBrand<P>`).
- Con: More complex generic signatures; may require additional brand types.

**C. Implement `Debug` via lowering.** For `Debug`, lower the Coyoneda and
delegate to the underlying type's `Debug` impl. This requires `F: Functor`
and `F::Of<'a, A>: Debug`.

- Pro: Useful for development and debugging.
- Con: Forces evaluation; may have side effects for lazy types.

**D. Provide a `debug_lower` method.** Instead of implementing `Debug` on
`Coyoneda` directly, provide a method that lowers and returns a `Debug`-able
value.

- Pro: Explicit about the cost; no surprising evaluation.
- Con: Not usable in generic `Debug` contexts.

### Recommendation

Approach B for `Clone` and `Send + Sync`, following the library's established
`FnBrand<P>` pattern. The `Coyoneda` type could be parameterized by a pointer
brand, with `Box`-based as the default. For `Debug`, approach D is preferable
to avoid surprising evaluation. These changes should be prioritized in roughly
this order: `Clone` (unlocks Traversable), `Send + Sync` (unlocks concurrent
use), then `Debug` (developer experience).

---

## 8. Identity Function Allocation in lift

### Description

`lift` creates a `CoyonedaBase` that stores `fa` directly and returns it
without calling `F::map`:

```rust
pub fn lift(fa: ...) -> Self {
    Coyoneda(Box::new(CoyonedaBase { fa }))
}
```

This is already optimized: no identity function is boxed. The design document's
open question about special-casing the identity function is resolved by the
`CoyonedaBase` / `CoyonedaMapLayer` split. `CoyonedaBase` is the identity
case, and `CoyonedaMapLayer` is the composed case.

However, `lift` still allocates one `Box` for the trait object. For
`lift(v).lower()` roundtrips, the allocation is wasted.

### Approaches

**A. Accept the allocation.** A single small heap allocation is negligible for
any realistic use case. The `lift`/`lower` roundtrip is a degenerate case that
users should not be concerned about.

- Pro: Simple; no additional complexity.
- Con: One unnecessary allocation in the degenerate case.

**B. Inline-storage optimization.** Use a `SmallBox` or similar inline-storage
type to avoid heap allocation for small inner types. The `CoyonedaBase` struct
is typically small (it contains `fa` which may itself be stack-allocated for
types like `Option`).

- Pro: Avoids heap allocation for small types.
- Con: Adds a dependency or custom implementation; complicates the type.

### Recommendation

Approach A. The allocation overhead is negligible compared to the work that
Coyoneda is designed for (deferred mapping over potentially large containers).
Optimizing this would add complexity with no meaningful performance benefit.

---

## 9. Missing Type Class Instances

### Description

The implementation provides `Functor`, `Pointed`, and `Foldable`. PureScript's
Coyoneda also provides `Apply`, `Applicative`, `Bind`, `Monad`, `Traversable`,
`Extend`, `Comonad`, `Eq`, `Ord`, and `Show`. Several of these are blocked by
the lack of `Clone` (Issue 7), but some could be implemented today.

Currently missing instances that could be implemented without `Clone`:

- `Eq` and `Ord` (via lowering, requires `F: Functor` and the underlying type's `Eq`/`Ord`).
- `FoldableWithIndex` (via lowering, similar pattern to `Foldable`).

Missing instances that require `Clone` (and thus the Rc/Arc variant):

- `Traversable`.
- `Semiapplicative` / `Apply`.
- `Semimonad` / `Bind`.

Missing instances that require additional design work:

- `Extend` / `Comonad` (the library may not have these type classes yet).

### Approaches

**A. Implement `Eq` and `Ord` now.** These only require lowering and comparing.

- Pro: Useful for testing and assertions.
- Con: Forces evaluation; may be surprising if users expect lazy comparison.

**B. Defer all missing instances until the Rc/Arc variant.** Implement
everything together as a coherent release.

- Pro: Consistent; all instances work together.
- Con: Delays useful functionality.

**C. Implement what is possible now; plan the rest for the Rc/Arc variant.**

- Pro: Incremental progress.
- Con: Two rounds of implementation.

### Recommendation

Approach C. Implement `Eq` and `Ord` via lowering now (matching how `Foldable`
works). Plan `Traversable`, `Semiapplicative`, and `Semimonad` for after the
Rc/Arc variant is available.

---

## 10. Foldable Lower-then-Fold Defeats Single-Pass Optimization

### Description

The current `Foldable` implementation lowers the entire Coyoneda (materializing
all map layers) and then folds the result:

```rust
fn fold_map<'a, FnBrand, A: 'a + Clone, M>(...) -> M {
    F::fold_map::<FnBrand, A, M>(func, fa.lower())
}
```

For `VecBrand` with k chained maps and n elements, this performs:

1. k passes over n elements (during `lower`, one `F::map` per layer).
2. 1 pass over n elements (during `F::fold_map`).
3. k intermediate `Vec` allocations (one per `F::map` call).

Total cost: O(k \* n) traversals plus k allocations.

If the fold function could be composed with the accumulated mapping function
and applied in a single pass over the original `F B`, the cost would be
O(n): one pass, zero intermediate allocations. This is precisely what
PureScript achieves.

### Approaches

**A. Compose the fold function at each layer.** Instead of calling
`self.inner.lower()` then `F::map(self.func, lowered)` then `F::fold_map`,
each `CoyonedaMapLayer` could provide a `fold_map` that composes `func` with
the user's fold function and delegates to `self.inner.fold_map(composed)`.
However, this requires a `fold_map_inner` method that is generic over the
monoid type, which is not dyn-compatible.

- Pro: True single-pass fold.
- Con: Not possible with trait objects.

**B. Use a type-erased fold accumulator.** Define a non-generic `fold_inner`
method that folds into a `Box<dyn Any>`, then downcast the result. Requires
`'static`.

- Pro: Single-pass fold.
- Con: Requires `'static`; loses lifetime polymorphism; runtime downcast overhead and potential panics.

**C. Specialize for common monoids.** Provide `fold_to_string`,
`fold_to_vec`, etc. methods that hard-code the monoid type, making the inner
method dyn-compatible for those specific cases.

- Pro: Single-pass for common cases; no `'static` requirement.
- Con: Not general; ad-hoc.

**D. Accept the limitation.** The lower-then-fold approach is correct, if not
optimal. For small k (the common case), the overhead is minimal.

### Recommendation

Approach D for now. The lower-then-fold approach has the same asymptotic
behavior as calling `F::map` k times followed by `F::fold_map` directly, which
is what users would do without Coyoneda anyway. The overhead is structural, not
a regression. If performance is critical and k is non-trivial, users should
compose their functions before mapping (`map(compose(f, g), v)`) and fold
directly, bypassing Coyoneda entirely. If the unsafe erasure approach from
Issue 1 is adopted, single-pass folding becomes achievable as a natural
extension.

---

## Summary of Recommendations

| Issue                           | Severity | Recommendation                                                           |
| ------------------------------- | -------- | ------------------------------------------------------------------------ |
| 1. No map fusion                | Medium   | Accept; provide `FunctorPipeline` as zero-cost alternative.              |
| 2. Fn vs FnOnce                 | None     | Current usage is correct; `Fn` is required for multi-element containers. |
| 3. Double-boxing in `new`       | Low      | Add `CoyonedaImpl` struct to eliminate one allocation.                   |
| 4. hoist requires Functor       | Low      | Accept; well-documented limitation.                                      |
| 5. Foldable requires Functor    | Low      | Accept; most Foldable types are also Functor.                            |
| 6. Stack overflow risk          | Low      | Document limitation; add `collapse` method.                              |
| 7. No Clone/Send/Sync/Debug     | Medium   | Parameterize over pointer brand following library conventions.           |
| 8. Identity allocation in lift  | None     | Already optimized via `CoyonedaBase`.                                    |
| 9. Missing type class instances | Low      | Implement `Eq`/`Ord` now; defer rest to Rc/Arc variant.                  |
| 10. Lower-then-fold overhead    | Low      | Accept; same cost as direct chaining.                                    |

The two most impactful improvements are the Rc/Arc variant (Issue 7, which
unblocks Traversable and concurrent use) and `FunctorPipeline` (Issue 1, which
provides true zero-cost fusion for performance-critical code). The remaining
issues are either inherent to the trait-object encoding, already well-documented,
or low-priority quality-of-life improvements.
