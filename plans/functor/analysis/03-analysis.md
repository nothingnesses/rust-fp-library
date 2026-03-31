# Coyoneda Implementation Analysis

Independent analysis of the Coyoneda free functor implementation in
`fp-library/src/types/coyoneda.rs`, assessing flaws, issues, and limitations,
with proposed approaches and recommendations.

Date: 2026-03-31

---

## Table of Contents

1. [No map fusion](#1-no-map-fusion)
2. [Foldable requires F: Functor](#2-foldable-requires-f-functor)
3. [Hoist requires F: Functor](#3-hoist-requires-f-functor)
4. [Fn trait choice prevents consuming the stored function](#4-fn-trait-choice-prevents-consuming-the-stored-function)
5. [Redundant allocation in the `new` constructor](#5-redundant-allocation-in-the-new-constructor)
6. [Unnecessary identity function box in `lift`](#6-unnecessary-identity-function-box-in-lift)
7. [Not Clone, not Send, not Sync](#7-not-clone-not-send-not-sync)
8. [Stack overflow risk with deep map chains](#8-stack-overflow-risk-with-deep-map-chains)
9. [Missing type class instances](#9-missing-type-class-instances)
10. [No Debug or Display](#10-no-debug-or-display)
11. [Design document divergence from implementation](#11-design-document-divergence-from-implementation)
12. [Test coverage gaps](#12-test-coverage-gaps)

---

## 1. No map fusion

### Description

The primary motivation for Coyoneda is map fusion: chaining k calls to `map`
should result in exactly one call to `F::map` at `lower` time, with the k
functions composed together. This implementation does not achieve this. Each
`map` call creates a new `CoyonedaMapLayer` that wraps the previous one, and
`lower` traverses the layers, calling `F::map` once per layer. For `VecBrand`
with n elements and k chained maps, the cost is O(k \* n), the same as calling
`F::map` directly k times.

This is a fundamental limitation, not a bug. The design document identifies the
root cause correctly: composing functions across the existential boundary
requires a generic `map_inner<C>` method, which is not dyn-compatible.

### Approaches

**A. Type-erased function composition via `Box<dyn Any>`.**
Store the accumulated function and the original `F B` value behind `Box<dyn Any>`,
downcast at `lower` time. This achieves true fusion but requires `'static` on all
types, preventing HKT integration (the `Kind` trait requires lifetime
polymorphism). The `Free` monad in this library has the same `'static` limitation
for the same reason.

Trade-offs: True fusion. Loses HKT participation. Two separate types would be
needed (one for HKT, one for fusion).

**B. Unsafe pointer erasure.**
Erase the intermediate type `B` via raw pointers or `transmute` to compose
functions inside the trait object without exposing `B` as a generic parameter.
This preserves lifetime polymorphism and achieves fusion.

Trade-offs: True fusion with full HKT support. Requires `unsafe` code with
careful soundness proofs. The complexity of maintaining soundness across
composition chains is substantial.

**C. `FunctorPipeline` zero-cost builder (complementary).**
Expose `B` as a type parameter: `FunctorPipeline<'a, Brand, B, A, F>`. Each
`.map(g)` changes the type signature. No boxing, no dynamic dispatch, true
fusion. Cannot participate in HKT because of the extra type parameters.

Trade-offs: Zero-cost, true fusion. No HKT integration. Excellent for
performance-critical code paths where HKT polymorphism is not needed.

**D. Accept the status quo with clear documentation.**
The current implementation provides structural benefits (HKT integration, free
functor property, deferred execution) even without fusion. The documentation
already explains this clearly.

### Recommendation

Implement approach C (`FunctorPipeline`) as a complementary utility for users
who need actual fusion, while keeping the current `Coyoneda` for HKT contexts.
Approach D is the pragmatic choice for the existing type. Approach B could be
explored if fusion with HKT support becomes a hard requirement, but the
`unsafe` burden is significant.

---

## 2. Foldable requires F: Functor

### Description

The `Foldable` implementation for `CoyonedaBrand<F>` requires
`F: Functor + Foldable + 'static`. In PureScript, `Foldable` for `Coyoneda`
requires only `Foldable f`, not `Functor f`. This is because PureScript's
`unCoyoneda` can open the existential to compose the fold function with the
accumulated mapping function, folding the original `F B` in a single pass.

The current implementation calls `fa.lower()` (which requires `F: Functor`),
producing an `F A`, and then delegates to `F::fold_map`. This means any brand
that is `Foldable` but not `Functor` cannot benefit from `Coyoneda`'s Foldable
instance.

### Approaches

**A. Accept the extra bound.**
In practice, most useful brands implement both `Functor` and `Foldable`. The
extra bound is a minor ergonomic cost rather than a blocking limitation.

Trade-offs: Simple. Does not regress any current usage. Diverges from
PureScript's semantics.

**B. Add operation-specific inner trait methods (breaks dyn-compatibility).**
Adding `fold_map_inner` to `CoyonedaInner` that is generic over `M: Monoid` and
`FnBrand: CloneableFn` would enable folding without lowering. However, this
makes the trait not dyn-compatible, which means `Box<dyn CoyonedaInner>` cannot
be formed.

Trade-offs: Would achieve PureScript parity. Fundamentally incompatible with
the trait-object encoding.

**C. Split into two inner traits.**
Keep `CoyonedaInner` dyn-compatible (only `lower`). Introduce a separate
`CoyonedaFoldInner` trait that is not used as a trait object but instead uses
enum dispatch or a different encoding. This adds significant complexity.

Trade-offs: Could work in theory. The implementation complexity is high and
the benefit is narrow (only matters for Foldable-but-not-Functor brands).

**D. Use a single-layer `CoyonedaImpl` encoding for Foldable.**
Store `fb` and `func` directly (as in the design document's `CoyonedaImpl`)
rather than layering. The `fold_map` implementation can then compose
`func` with the fold function and call `F::fold_map` once. This requires
restructuring how `map` works: instead of layering, compose into a single
`Box<dyn Fn>`. But this is exactly the map fusion problem from issue 1.

Trade-offs: Tied to the map fusion problem. Same dyn-compatibility blocker.

### Recommendation

Accept approach A. The `F: Functor` bound is a reasonable pragmatic choice.
Document the divergence from PureScript (which is already done in the module
docs). If a `Foldable`-but-not-`Functor` brand becomes important, revisit
approach C.

---

## 3. Hoist requires F: Functor

### Description

`Coyoneda::hoist` applies a natural transformation `F ~> G` by lowering to `F`,
applying the transformation, then lifting into `Coyoneda<G, A>`. This requires
`F: Functor` for the lower step. PureScript's `hoistCoyoneda` does not require
`Functor f` because it opens the existential directly.

This has the same root cause as issues 1 and 2: dyn-compatibility prevents
adding a `hoist_inner<G>` method to the inner trait.

### Approaches

**A. Accept the extra bound.**
Same reasoning as issue 2. Most brands used with `hoist` will be `Functor`.

**B. Add `hoist_inner` that takes a boxed natural transformation.**
If `NaturalTransformation` could be invoked through a trait object (i.e., if
`transform` were not generic over `A`), `hoist_inner` could accept a
`&dyn NaturalTransformation<F, G>` without making `CoyonedaInner` not
dyn-compatible. However, `NaturalTransformation::transform` is generic over `A`,
so a reference to it cannot be a trait object either. The problem propagates.

Trade-offs: Not feasible given current trait definitions.

**C. Use a function pointer or closure for the natural transformation.**
Replace the `NaturalTransformation` trait with a type-erased callback. This
would require knowing `B` at the call site, which is hidden by the existential.

Trade-offs: Does not solve the fundamental issue.

### Recommendation

Accept approach A. The `F: Functor` bound is documented and reasonable. This
is a known consequence of the encoding choice.

---

## 4. Fn trait choice prevents consuming the stored function

### Description

The stored mapping function in `CoyonedaMapLayer` is `Box<dyn Fn(B) -> A + 'a>`.
The `Fn` trait requires the function to be callable by shared reference (`&self`),
which means it can be called multiple times. This is correct for `Foldable` (which
may need to apply the function to multiple elements) but overly restrictive for
`lower`, which consumes the `Coyoneda` and only needs to call the function once
per element via `F::map`.

Since `F::map` also takes `impl Fn(A) -> B` (not `FnOnce`), this is consistent
with the library's design. The `Fn` bound is not actually a restriction in
practice because `Functor::map` already requires `Fn`.

However, there is a subtle issue: because `Fn` is required instead of `FnOnce`,
the mapping function cannot capture values by move if those values are not `Clone`.
For example, `coyo.map(move |x| expensive_resource.consume(x))` would not compile
if `expensive_resource` does not implement `Clone` or `Fn` traits. This is a
restriction inherited from the library's `Functor::map` signature, not specific
to Coyoneda, but it is worth noting.

### Approaches

**A. No change needed.**
The `Fn` bound is consistent with `Functor::map` across the library. Changing
it in Coyoneda alone would create an inconsistency.

**B. Library-wide consideration of `FnOnce` for map.**
This is a broader design question for the library, not specific to Coyoneda.

### Recommendation

No action needed for Coyoneda specifically. This is a library-wide design
decision that is already documented and intentional (the CLAUDE.md explains the
uncurried `impl Fn` choice for zero-cost abstractions).

---

## 5. Redundant allocation in the `new` constructor

### Description

The `new` constructor creates three heap allocations:

1. `Box::new(CoyonedaBase { fa: fb })` for the inner base layer.
2. `Box::new(f)` for the mapping function.
3. `Box::new(CoyonedaMapLayer { inner: ..., func: ... })` for the outer layer.

This is because `new` constructs a `CoyonedaMapLayer` wrapping a `CoyonedaBase`.
The design document's original `CoyonedaImpl` struct stored `fb` and `func`
together, which would need only two allocations (one for the impl, one for the
function box). The design document even notes that `new` "saves one box
allocation vs `lift(fb).map(f)`", which is true, but the savings compared to
the original `CoyonedaImpl` design are lost.

Actually, examining more carefully: `lift(fb).map(f)` would create:

1. `Box::new(CoyonedaBase { fa: fb })` (from lift).
2. `Box::new(f)` (from map).
3. `Box::new(CoyonedaMapLayer { inner: ..., func: ... })` (from map).

So `new` and `lift(fb).map(f)` have the same allocation count (3 boxes). The
design document's claim about saving one allocation appears to be incorrect,
unless the intent was that `new` would use a single-struct encoding.

### Approaches

**A. Introduce a `CoyonedaWithFunc` struct.**
A struct that stores both `fb` and `func` directly (like the design document's
`CoyonedaImpl`), implementing `CoyonedaInner`. This reduces `new` to two
allocations (one for the struct as a trait object, one for the boxed function).

```
struct CoyonedaWithFunc<'a, F, B, A> {
    fb: F::Of<'a, B>,
    func: Box<dyn Fn(B) -> A + 'a>,
}
```

Trade-offs: Reduces `new` from 3 to 2 allocations. Adds a third implementor of
`CoyonedaInner`, slightly increasing code surface. The optimization is minor
(one small heap allocation).

**B. Keep the current encoding.**
The third allocation is for a `CoyonedaBase`, which is a thin wrapper around
`fb`. The overhead is one pointer-sized allocation. For most use cases this is
negligible.

### Recommendation

Approach A is a minor optimization that aligns the implementation with the
design document's intent. It is worth doing if the code is being revised for
other reasons, but not worth a standalone change.

---

## 6. Unnecessary identity function box in `lift`

### Description

The design document's original `CoyonedaImpl`-based encoding stored an identity
function `Box::new(|a| a)` in `lift`. The current implementation avoids this by
using `CoyonedaBase`, which returns `fa` directly without calling `F::map`. This
is a good optimization noted in the design document as "Base layer optimization."

No issue here. This is listed for completeness to confirm the optimization is
correctly implemented.

### Recommendation

No action needed. The `CoyonedaBase` approach is correct and avoids both the
identity function allocation and the unnecessary `F::map(identity, fa)` call.

---

## 7. Not Clone, not Send, not Sync

### Description

`Coyoneda` wraps `Box<dyn CoyonedaInner<'a, F, A> + 'a>`, which is:

- Not `Clone` (Box is not Clone for trait objects).
- Not `Send` (the trait object bound does not include `Send`).
- Not `Sync` (same reason).

This prevents:

- `Traversable` implementation (requires `Self::Of<'a, B>: Clone`).
- `Semiapplicative`/`Applicative` (typically needs Clone for multi-use).
- Thread-safe usage.

### Approaches

**A. Rc/Arc-wrapped variant.**
The design document proposes `SharedCoyoneda<'a, F, A, P: RefCountedPointer>`
that wraps the inner trait object in `Rc` or `Arc` instead of `Box`. This
enables Clone (via reference counting) and, for the `Arc` variant, Send + Sync
(if the inner values are Send + Sync).

Trade-offs: Adds reference counting overhead. Enables Clone, Send, Sync.
Unlocks Traversable, Semiapplicative, and thread safety. Aligns with the
library's existing `Rc`/`Arc` pointer abstraction hierarchy.

**B. Send variant only.**
Add `SendCoyoneda<'a, F, A>` wrapping
`Box<dyn CoyonedaInner<'a, F, A> + Send + 'a>`. This provides Send but not
Clone.

Trade-offs: Simpler than the Rc/Arc approach. Does not enable Traversable.
Mirrors the `Thunk`/`SendThunk` split already in the library.

**C. Both.**
Provide the current `Box`-based `Coyoneda` for the common single-owner case,
plus a `SharedCoyoneda` for Clone-needing contexts, plus a `SendCoyoneda` for
thread-safe single-owner contexts.

Trade-offs: Most flexible. Most code to maintain. Three variants mirrors the
library's existing pattern (e.g., `Thunk`, `SendThunk`, `RcLazy`, `ArcLazy`).

### Recommendation

Implement approach A (Rc/Arc variant) as the next priority, parameterized by
`RefCountedPointer` to handle both `Rc` and `Arc` cases in a single type. This
follows the library's established pattern with `FnBrand<P>` and `LazyBrand<Config>`.
Approach B can be added later if there is demand for a Send-but-not-Clone variant.

---

## 8. Stack overflow risk with deep map chains

### Description

Each `map` call adds a layer of nesting. At `lower` time, each layer calls
`self.inner.lower()` recursively before calling `F::map`. For k chained maps,
this creates k frames of recursion on the call stack. The test
`many_chained_maps` verifies 100 layers, which is fine, but a chain of
thousands or millions of maps would overflow the stack.

This is the same class of problem that `Free` and `Trampoline` solve for monadic
bind chains. However, Coyoneda's use case (map fusion) typically involves modest
chain lengths (tens of maps, not millions), so this may be an academic concern.

### Approaches

**A. Iterative lowering.**
Convert the recursive `lower` chain to an iterative loop. This would require
either:

- Storing layers in a flat data structure (e.g., a `Vec<Box<dyn Fn>>`) instead
  of nested trait objects.
- Using a trampoline-style approach.

The flat-vector approach changes the core data structure significantly. The
trampoline approach adds overhead.

Trade-offs: Eliminates stack overflow risk. Significant refactoring. The
flat-vector approach could also enable map fusion (compose all functions in the
vector, apply once).

**B. Document the limitation.**
Add a note that Coyoneda is designed for moderate chain lengths and not
suitable for unbounded recursive map accumulation.

**C. No action.**
The current 100-layer test passes. Real-world usage is unlikely to exceed
stack depth limits through map chaining alone.

### Recommendation

Approach C is sufficient for now. If the flat-vector approach (approach A) is
pursued, it could simultaneously address issue 1 (map fusion) and issue 8
(stack safety), making it a compelling combined solution. See the discussion in
issue 1, approach C, for the related `FunctorPipeline` design.

---

## 9. Missing type class instances

### Description

The implementation provides Functor, Pointed, and Foldable. PureScript's
Coyoneda provides Apply, Applicative, Bind, Monad, Extend, Comonad, Eq, Ord,
and more. The design document lists several of these as deferred.

Key missing instances:

- **Semiapplicative/Applicative**: Requires lowering both operands. Needs
  `F: Applicative`. Loses the "deferred map" benefit but provides API
  completeness.
- **Semimonad/Monad**: Same pattern; lower, bind, lift.
- **Traversable**: Blocked on Clone (issue 7).
- **Eq/Ord/Debug**: Can be implemented via lowering, but require the
  corresponding bounds on `F`'s applied type.

### Approaches

**A. Implement Semiapplicative and Semimonad via lowering.**
Lower both operands, apply the underlying `F`'s instance, then lift the result.
This is the standard approach (used in PureScript).

Trade-offs: Straightforward. Requires `F: Applicative` or `F: Monad`.
Does not preserve deferred-map benefits across apply/bind boundaries, but this
is expected (Coyoneda is the free functor, not the free monad).

**B. Implement Eq and Ord via lowering.**
Add `impl Eq for Coyoneda where F: Functor, F::Of<'a, A>: Eq`. Requires
consuming `self` (since `lower` takes ownership), which is incompatible with
`PartialEq`'s `&self` signature. Alternatively, restrict to the Rc/Arc variant
where lowering can be done from a clone.

Trade-offs: Eq via lowering is destructive for the Box variant. The Rc/Arc
variant would handle this naturally.

### Recommendation

Implement approach A (Semiapplicative and Semimonad via lowering) as a next
step. Defer Eq/Ord/Debug to the Rc/Arc variant where non-destructive lowering
is possible.

---

## 10. No Debug or Display

### Description

`Coyoneda` does not implement `Debug` or `Display`. The inner trait object
`Box<dyn CoyonedaInner>` cannot derive `Debug` because trait objects do not
implement `Debug` by default. This makes debugging difficult; printing a
`Coyoneda` value requires lowering it first, which is destructive.

### Approaches

**A. Implement Debug with opaque output.**

```rust
impl<'a, F, A: 'a> fmt::Debug for Coyoneda<'a, F, A>
where F: Kind_cdc7cd43dac7585f + 'a {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Coyoneda").finish_non_exhaustive()
    }
}
```

Trade-offs: Provides some Debug output without requiring any bounds on `F` or
`A`. The output is opaque (no inner value), but at least the type is printable.

**B. Add a Debug-via-lowering method.**
Provide `fn debug_lower(self) -> ...` that lowers and returns a Debug-able
value. Destructive, but explicit.

**C. Defer to Rc/Arc variant.**
The Rc/Arc variant can clone, lower, and format without consuming the original.

### Recommendation

Implement approach A (opaque Debug) for basic usability. The Rc/Arc variant
(approach C) will enable richer Debug output in the future.

---

## 11. Design document divergence from implementation

### Description

The design document describes two encoding approaches in different sections,
which can be confusing:

1. The "Detailed Design" section (starting around line 236) describes a
   `CoyonedaImpl` struct with `fb` and `func`, along with `map_inner` and
   `fold_map_inner` methods on `CoyonedaInner`. This is the theoretically
   ideal encoding that cannot be implemented due to dyn-compatibility.

2. The "Actual Implementation: Layered Trait Objects" section describes
   `CoyonedaBase` and `CoyonedaMapLayer`, which is what was actually built.

3. The "Detailed Design (as implemented)" section (line 236) shows
   `CoyonedaInner` with both `lower` and `map_inner` methods, but the actual
   implementation only has `lower`.

The document also claims `new` "saves one box allocation vs `lift(fb).map(f)`",
but as analyzed in issue 5, both paths produce the same number of allocations
(3 boxes).

### Recommendation

Update the design document to:

- Clearly separate the "ideal but infeasible" design from the "actual
  implementation" design.
- Correct the allocation count claim for `new`.
- Remove or clearly label the `CoyonedaImpl`/`map_inner`/`fold_map_inner` code
  as the infeasible ideal, to avoid confusion with the actual layered encoding.

---

## 12. Test coverage gaps

### Description

The test suite has 23 unit tests and covers the main happy paths well. However,
several gaps exist:

- **No property-based tests.** The functor laws (identity and composition) are
  tested with single fixed inputs. QuickCheck/proptest would provide stronger
  coverage that the laws hold for arbitrary inputs and chain lengths.
- **No test for `new` vs `lift` + `map` equivalence with non-trivial types.**
  The `new_is_equivalent_to_lift_then_map` test uses simple closures. Testing
  with types that have non-trivial Clone or Drop behavior would increase
  confidence.
- **No test for type-changing maps.** Most tests map `i32 -> i32` or
  `i32 -> String`. Testing chains that change types multiple times (e.g.,
  `i32 -> bool -> String -> Vec<u8>`) would verify the existential encoding
  handles heterogeneous type chains correctly.
- **No test for large structures.** The `many_chained_maps` test uses
  `vec![0i64]` (one element). Testing with large vectors would verify that the
  layered lowering does not introduce unexpected performance regressions or
  correctness issues.
- **No test for lifetime behavior.** All tests use `'static` values. Testing
  with borrowed data (e.g., `Coyoneda<VecBrand, &str>`) would verify the
  lifetime parameterization works correctly.
- **No compile-fail tests.** There are no `trybuild` tests verifying that
  incorrect usage (e.g., calling `lower` without `F: Functor`) produces
  reasonable error messages.
- **Foldable tests do not verify single-pass behavior.** The Foldable tests
  check correctness but do not verify that folding happens in a single pass
  over the lowered structure (as opposed to, say, lowering and then folding
  with multiple passes).

### Recommendation

Add property-based tests for the functor laws as the highest-priority
improvement. Add compile-fail tests for the `lower` requires `F: Functor`
constraint. Add a lifetime test with borrowed data. The other gaps are lower
priority.

---

## Summary of priorities

| Priority | Issue                              | Recommendation                                              |
| -------- | ---------------------------------- | ----------------------------------------------------------- |
| 1        | No map fusion (issue 1)            | Add `FunctorPipeline` as a complementary zero-cost builder. |
| 2        | Not Clone/Send/Sync (issue 7)      | Implement Rc/Arc-wrapped variant.                           |
| 3        | Missing instances (issue 9)        | Add Semiapplicative and Semimonad via lowering.             |
| 4        | Test gaps (issue 12)               | Add property-based and compile-fail tests.                  |
| 5        | No Debug (issue 10)                | Add opaque Debug impl.                                      |
| 6        | Redundant alloc in `new` (issue 5) | Add `CoyonedaWithFunc` struct.                              |
| 7        | Design doc divergence (issue 11)   | Update document for clarity.                                |
| --       | Foldable needs Functor (issue 2)   | Accept; already documented.                                 |
| --       | Hoist needs Functor (issue 3)      | Accept; already documented.                                 |
| --       | Fn trait choice (issue 4)          | No action; consistent with library design.                  |
| --       | Stack overflow (issue 8)           | No action; academic concern for typical usage.              |
