# Ref Trait Hierarchy: Design Analysis

## Overview

The Ref trait hierarchy mirrors the by-value type class hierarchy but changes
how closures receive elements: `&A` instead of `A`. Containers are still
consumed by value (moved into the trait method). The hierarchy enables
memoized types like `Lazy` to participate in the full Functor -> Applicative
-> Monad chain without forcing callers to clone cached values.

This analysis covers 17 Ref traits: RefFunctor, RefPointed, RefLift,
RefSemiapplicative, RefSemimonad, RefApplicative, RefMonad, RefApplyFirst,
RefApplySecond, RefFoldable, RefFoldableWithIndex, RefFunctorWithIndex,
RefFilterable, RefFilterableWithIndex, RefTraversable,
RefTraversableWithIndex, and RefWitherable.

## 1. FnOnce -> Fn Change on RefFunctor

### Decision

`RefFunctor::ref_map` was changed from `impl FnOnce(&A) -> B` to
`impl Fn(&A) -> B`. This was necessary for multi-element containers
like `Vec`, which call the closure once per element.

### Assessment

The change is correct and justified. `FnOnce` would have permanently
restricted `RefFunctor` to single-element containers (Lazy, Identity,
Option). Since the whole point of the Ref hierarchy is to unify memoized
types and collections under one interface, `Fn` is the only viable choice.

### Capabilities lost

- Closures that move out of their captures (FnOnce-only closures) can no
  longer be used with `ref_map`. For example:

  ```rust
  let buffer = vec![1, 2, 3];
  // This closure moves `buffer` out, so it is FnOnce, not Fn.
  ref_map(|x: &i32| { drop(buffer); *x }, lazy)
  ```

  Such closures are rare in practice and can always be restructured by
  extracting the move into a surrounding scope.

- For single-element containers, `FnOnce` was theoretically more precise.
  However, every `Fn` closure is also `FnOnce`, so any closure that works
  with `Fn` would also have worked with `FnOnce`. The constraint only
  matters in the other direction: closures that are `FnOnce` but not `Fn`.
  These are uncommon and represent a negligible practical loss.

### Stale documentation

The current `RefFunctor` trait doc (line 80-85 of `ref_functor.rs`) still
contains a "Why `FnOnce`?" section explaining the rationale for `FnOnce`.
The actual signature now uses `Fn`. This documentation is stale and should
be updated or replaced with a "Why `Fn`?" section explaining the change.

## 2. Clone Bound on RefPointed

### Design

`RefPointed::ref_pure` requires `A: Clone` because it must produce an owned
`Of<A>` from `&A`. The Clone bound is on the trait method, not the trait
itself.

### Assessment

The Clone bound is justified and unavoidable. Given only `&A`, producing
an owned `A` to place into the container requires cloning. There is no way
around this without changing the fundamental contract.

### Could it be avoided?

- **No, for the general case.** The trait promises to construct an `Of<A>`
  from a reference, which inherently requires producing owned data.

- **Alternative: `Cow`-based approach.** One could imagine
  `ref_pure<'b>(a: &'b A) -> Of<Cow<'b, A>>`, but this changes the output
  type and would not compose cleanly with the rest of the hierarchy (which
  expects `Of<A>`, not `Of<Cow<A>>`).

- **Alternative: bound at the trait level.** Moving the `Clone` bound from
  the method to the trait itself (i.e., `trait RefPointed where A: Clone`
  in effect) would be worse because it would prevent implementing
  `RefPointed` for types where some `A` values are not `Clone`. The
  current placement is correct: the bound is on the method, so the trait
  can be implemented for a brand even if not all element types are `Clone`.
  The bound only triggers when `ref_pure` is actually called.

### Observation

The plan's design principle is well-applied here: "Only `RefPointed`
requires `Clone`, because it is the only operation that must produce an
owned value from a reference without a user-supplied closure to mediate."
All other Ref traits pass `&A` to a user closure, delegating the
clone-or-not decision to the caller.

## 3. Supertrait Relationships

### RefApplicative

```
RefApplicative: RefPointed + RefSemiapplicative + RefApplyFirst + RefApplySecond
```

By-value counterpart:

```
Applicative: Pointed + Semiapplicative + ApplyFirst + ApplySecond
```

These are structurally identical, which is correct.

### RefMonad

```
RefMonad: RefApplicative + RefSemimonad
```

By-value counterpart:

```
Monad: Applicative + Semimonad
```

Structurally identical. Correct.

### RefSemiapplicative

```
RefSemiapplicative: RefLift + RefFunctor
```

By-value counterpart:

```
Semiapplicative: Lift + Functor
```

Structurally identical. Correct.

### RefApplyFirst / RefApplySecond

```
RefApplyFirst: RefLift
RefApplySecond: RefLift
```

By-value counterpart:

```
ApplyFirst: Lift
ApplySecond: Lift
```

Structurally identical. Both have blanket impls from their `Lift`/`RefLift`
supertrait. Correct.

### RefFilterable

```
RefFilterable: RefFunctor + Compactable
```

By-value counterpart:

```
Filterable: Compactable + Functor
```

The key difference is that `RefFilterable` uses non-Ref `Compactable` as a
supertrait. This is analyzed in section 4.

### RefTraversable

```
RefTraversable: RefFunctor + RefFoldable
```

By-value counterpart:

```
Traversable: Functor + Foldable
```

Structurally mirrors the by-value hierarchy. Correct.

### RefWitherable

```
RefWitherable: RefFilterable + RefTraversable
```

By-value counterpart:

```
Witherable: Filterable + Traversable
```

Structurally identical. Correct.

### Overall assessment

The supertrait relationships are correct and consistent with the by-value
hierarchy. Each Ref trait mirrors its by-value counterpart exactly,
substituting Ref variants where the trait's methods access elements.

## 4. Structural Traits and the Compactable Question

### The design principle

The plan states: "Traits whose methods take only containers (no closures)
don't need Ref variants. Compound traits use the non-Ref version of
structural supertraits and the Ref version of element-accessing
supertraits."

### Application to RefFilterable

`Compactable` defines `compact` and `separate`, which operate purely on
container structure without closures receiving `&A`:

```rust
fn compact(fa: Of<Option<A>>) -> Of<A>;
fn separate(fa: Of<Result<A, B>>) -> (Of<B>, Of<A>);
```

These methods do not pass elements to closures, so there is no `&A` vs `A`
distinction to make. The Ref/non-Ref distinction is irrelevant for them.
Using non-Ref `Compactable` as a supertrait of `RefFilterable` is correct.

### Consistency check

Looking at RefFilterable's default implementations:

- `ref_partition_map` calls `Self::separate(Self::ref_map(func, fa))`.
  This uses `ref_map` (Ref) for the closure-receiving part and `separate`
  (non-Ref) for the structural part.

- `ref_filter_map` calls `Self::compact(Self::ref_map(func, fa))`.
  Same pattern: `ref_map` (Ref) + `compact` (non-Ref).

This is the right factoring. The principle is consistently applied.

### WithIndex and RefFoldableWithIndex

`RefFoldableWithIndex: RefFoldable + WithIndex`. The `WithIndex` trait
defines `type Index`, which is purely a type-level declaration with no
element access. Correctly uses the non-Ref `WithIndex`.

`RefFilterableWithIndex: RefFilterable + RefFunctorWithIndex + WithIndex`.
Note the redundant `WithIndex` here: `RefFunctorWithIndex` already extends
`WithIndex`. The explicit mention is harmless (Rust merges duplicate
supertrait bounds) but could be simplified for clarity.

### Missing Ref structural traits

There is no `RefCompactable`, `RefPlus`, `RefAlt`, or `RefAlternative`.
This is correct since these are all structural operations without closures.

## 5. RefTraversable: Was the Deferral Concern Addressed?

### Original concern

The plan initially deferred `RefTraversable`, saying: "Complex interaction
with the applicative used for the output context."

The concern was about which applicative constraint to place on the output
context `F`. If the closure produces `F<B>` values, the traversal
implementation needs to combine them using `F`'s applicative operations.
But should `F` be `Applicative` (by-value) or `RefApplicative` (by-ref)?

### How it was resolved

`RefTraversable::ref_traverse` uses `F: Applicative` (by-value):

```rust
fn ref_traverse<'a, FnBrand, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
    func: impl Fn(&A) -> F::Of<'a, B> + 'a,
    ta: Self::Of<'a, A>,
) -> F::Of<'a, Self::Of<'a, B>>
```

This is the correct choice. The "Ref" in `RefTraversable` refers to how
elements of the _input_ container are accessed (by reference). The _output_
applicative `F` is a separate concern: its elements are newly constructed
`B` values (owned), so by-value `Applicative` is appropriate. Using
`RefApplicative` for `F` would unnecessarily restrict which types can serve
as the output context.

### Adequacy

The concern was adequately addressed. The key insight is that "Ref" applies
to element access in the traversed container, not to the output context.
The implementation works correctly for all existing types (Vec, Option,
CatList, Identity all implement `RefTraversable` with `F: Applicative`).

### Missing: ref_sequence

The doc comment for `RefTraversable` mentions that "`ref_sequence` has a
default implementation derived from `ref_traverse` using the identity
function." However, `ref_sequence` does not actually exist in the codebase.
The doc comment is misleading. Either `ref_sequence` should be added, or
the doc comment should be corrected.

A `ref_sequence` for `RefTraversable` would be unusual because it would
need `Self::Of<'a, F::Of<'a, A>>` as input, and the inner `F::Of<'a, A>`
values would be accessed by reference (`&F::Of<'a, A>`), requiring
`Clone` on them to produce the output. It is not clear this operation is
useful enough to warrant adding, but the documentation should be accurate
regardless.

## 6. RefFoldable::ref_fold_map and the Clone Requirement

### The FnBrand parameter

`ref_fold_map` takes a `FnBrand: LiftFn` type parameter, matching the
by-value `Foldable::fold_map`. The `FnBrand` is needed because the default
implementations of `ref_fold_right` and `ref_fold_left` use
`Endofunction<FnBrand, B>`, which requires wrapping closures in the
function brand's pointer type (Rc or Arc).

This is the right abstraction. It allows both Rc-based and Arc-based
function wrapping, matching the pointer abstraction used elsewhere in the
library.

### The A: Clone requirement

The `ref_fold_map` signature requires `A: 'a + Clone`:

```rust
fn ref_fold_map<'a, FnBrand, A: 'a + Clone, M>(
    func: impl Fn(&A) -> M + 'a,
    fa: Self::Of<'a, A>,
) -> M
```

This `Clone` bound exists because the default implementation of
`ref_fold_right` (which `ref_fold_map` delegates to in its own default
impl) needs to clone elements to construct `Endofunction` closures:

```rust
fn ref_fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
    func: impl Fn(&A, B) -> B + 'a,
    initial: B,
    fa: Self::Of<'a, A>,
) -> B
where FnBrand: LiftFn + 'a, {
    let f = FnBrand::new(move |(a, b): (A, B)| func(&a, b));
    // ...
    move |a: &A| {
        let a = a.clone();  // <-- Clone is needed here
        // ...
    }
}
```

### Does Clone propagate unnecessarily?

Yes, this is a design flaw. The `A: Clone` bound on `ref_fold_map` is only
needed when the default implementation is used (which delegates to
`ref_fold_right`, which clones `A` to build `Endofunction` wrappers).
However, the bound is on the trait method signature itself, so it applies
even when an implementor provides a direct implementation that does not need
`Clone`.

For example, Vec's `ref_fold_map` implementation is:

```rust
fn ref_fold_map<'a, FnBrand, A: 'a + Clone, M>(
    func: impl Fn(&A) -> M + 'a,
    fa: Vec<A>,
) -> M {
    fa.iter().fold(Monoid::empty(), |acc, a| Semigroup::append(acc, func(a)))
}
```

This implementation never clones `A`. The `Clone` bound is dead weight
imposed by the trait definition.

Similarly, Lazy's `ref_fold_map` implementation is:

```rust
fn ref_fold_map<'a, FnBrand, A: 'a + Clone, M>(
    func: impl Fn(&A) -> M + 'a,
    fa: Lazy<A, Config>,
) -> M {
    func(fa.evaluate())
}
```

No cloning occurs. The `Clone` bound is purely an artifact of the trait
signature.

### Potential fix

One approach would be to split `ref_fold_map` from `ref_fold_right` and
`ref_fold_left`:

- `ref_fold_map` could have no `Clone` bound (it only passes `&A` to the
  closure).
- `ref_fold_right` and `ref_fold_left` could retain `A: Clone` (needed for
  their `Endofunction`-based mutual derivation).
- Default impls would still work for `ref_fold_right -> ref_fold_map` and
  vice versa, but the `ref_fold_map -> ref_fold_right` default would
  require the additional `Clone` bound.

However, this would break the current "all three methods are mutually
derivable from any one" design, which mirrors `Foldable`. The by-value
`Foldable::fold_map` does not have this problem because it receives `A` by
value (no Clone needed to construct closures capturing `A`). The Clone
bound is a tax specific to the Ref variant.

### Assessment

The current design prioritizes symmetry with the by-value hierarchy over
minimal bounds. This is a reasonable trade-off, but implementors should be
aware that `A: Clone` is required even when their implementation does not
use it. The FnBrand parameter is the right abstraction. The Clone bound
is an acceptable cost of the mutual derivation design, but it should be
documented explicitly as a limitation.

## 7. Missing Ref Traits

### Ref traits that probably should exist

**RefMonadRec**: The by-value hierarchy has `MonadRec: Monad` for stack-safe
recursive monadic computations. A `RefMonadRec` would be useful for
stack-safe by-ref monadic recursion, but only if `Lazy` needs it. Since
`Lazy` evaluates thunks eagerly in `ref_bind` (it just calls `f(ma.evaluate())`),
deep bind chains are not tail-recursive and could overflow. However, this
is a preexisting limitation of `Lazy`'s bind implementation, not specific
to the Ref hierarchy. Low priority.

### Ref traits that correctly do not exist

**RefCompactable**: `compact`/`separate` are structural; no element access.
Correct to omit.

**RefAlt / RefPlus / RefAlternative**: `alt`/`empty` operate on containers
without element closures. Correct to omit.

**RefMonadPlus**: Would be `RefMonad + RefAlternative`. Since
`RefAlternative` does not exist (correctly), `RefMonadPlus` also correctly
does not exist.

**RefBifunctor / RefBifoldable / RefBitraversable**: The plan explicitly
defers these, noting no memoized bifunctor type exists. Correct deferral.

### Partially missing

**RefLift3, RefLift4, RefLift5**: The by-value hierarchy has `lift3`
through `lift5` via the dispatch system. Ref variants of these exist via
the dispatch system (the unified `lift3`-`lift5` free functions dispatch
to Ref when closures take references). However, there are no dedicated
`RefLift3`-`RefLift5` traits. This matches the by-value hierarchy, where
`lift3`-`lift5` are defined on `Lift` directly rather than as separate
traits. Consistent.

## 8. Relationship Between Ref and By-Value Traits

### Independence

Ref traits and by-value traits are intentionally independent: no
sub/supertrait relationship exists between them. A type can implement:

- Both (Vec, Option, CatList, Identity).
- Only Ref variants (RcLazy for RefFunctor, etc.).
- Only by-value variants (theoretically possible, though no current type
  does this).

### Can implementing RefFunctor without Functor cause issues?

**In the type system: No.** The traits are independent, so implementing
one does not obligate implementing the other. Generic code constrained by
`Functor` will not accept a `RefFunctor`-only type, and vice versa.

**In the dispatch system: Handled correctly.** The unified `map` function
dispatches based on closure type: `|x: i32| ...` -> `Functor::map`,
`|x: &i32| ...` -> `RefFunctor::ref_map`. If a brand implements only
`RefFunctor`, calling `map` with a by-value closure will fail at compile
time with a missing trait bound error. This is the correct behavior.

**In practice: LazyBrand<RcLazyConfig> demonstrates this.** It implements
`RefFunctor` but not `Functor`. The `m_do!(ref ...)` macro generates
by-ref dispatches, and the unified `map`/`bind` free functions route
correctly. No issues observed.

### Potential confusion

Users might expect that any type implementing `Functor` also implements
`RefFunctor` (or vice versa). The documentation should clarify that these
are independent hierarchies. The dispatch system masks this at the
call-site level (users call `map` and the right trait is selected), but
generic code must explicitly choose which constraint to use.

### Cross-hierarchy interaction in RefWitherable

`RefWitherable::ref_wilt` and `ref_wither` have default implementations
that call `M::map` (by-value `Functor::map` on the output context `M`)
after `ref_traverse`. This means the output applicative `M` must implement
by-value `Functor` (via `Applicative`), even though the input container is
traversed by reference. This is correct: the output context produces newly
owned values, so by-value operations on it are appropriate.

## 9. Blanket Impls

### RefApplicative

```rust
impl<Brand> RefApplicative for Brand
where Brand: RefPointed + RefSemiapplicative + RefApplyFirst + RefApplySecond {}
```

This is correct and minimal. No methods need to be defined; the trait is
purely a combination of its supertraits.

### RefMonad

```rust
impl<Brand> RefMonad for Brand
where Brand: RefApplicative + RefSemimonad {}
```

Correct and minimal.

### RefApplyFirst / RefApplySecond

```rust
impl<Brand: RefLift> RefApplyFirst for Brand {}
impl<Brand: RefLift> RefApplySecond for Brand {}
```

These have blanket impls from `RefLift` alone, with default method
implementations using `ref_lift2`. This matches the by-value hierarchy
where `ApplyFirst` and `ApplySecond` are blanket-implemented from `Lift`.

### Potential issue: RefApplicative supertraits are all blanket

Since `RefApplyFirst` and `RefApplySecond` are blanket-implemented for all
`RefLift` types, and `RefSemiapplicative` already requires `RefLift`, any
type implementing `RefPointed + RefSemiapplicative` automatically gets
`RefApplicative`. This means the four-supertrait bound on
`RefApplicative` is effectively equivalent to `RefPointed + RefSemiapplicative`.
The `RefApplyFirst + RefApplySecond` supertraits are technically redundant
since they follow from `RefSemiapplicative: RefLift`.

However, listing them explicitly is correct for documentation and for
allowing types to override the blanket implementations of
`ref_apply_first`/`ref_apply_second` with more efficient versions.

## 10. Additional Findings

### 10.1 Clone bounds on RefApplyFirst / RefApplySecond

`RefApplyFirst::ref_apply_first` requires `A: Clone`:

```rust
fn ref_apply_first<'a, A: Clone + 'a, B: 'a>(fa, fb) -> Of<A>
```

The default implementation clones `A` because the closure receives `&A`
and must return an owned `A`:

```rust
Self::ref_lift2(|a: &A, _: &B| a.clone(), fa, fb)
```

Similarly, `RefApplySecond` requires `B: Clone`. These bounds are
justified by the same reasoning as `RefPointed`: producing owned values
from references requires cloning. The bounds are correctly placed on the
methods, not the traits.

### 10.2 ref_if_m and ref_unless_m require Clone on the container

Both `ref_if_m` and `ref_unless_m` require
`Of<A>: Clone` / `Of<()>: Clone` respectively, because the closure passed
to `ref_bind` must be `Fn` (not `FnOnce`), so the branch values must be
cloneable to be captured and potentially returned multiple times. For
single-element containers like Lazy, the closure is called once, but the
trait cannot know this. The `Clone` bound is necessary.

### 10.3 ref_join requires Clone on the inner container

```rust
pub fn ref_join<'a, Brand: RefSemimonad, A: 'a>(mma) -> Of<A>
where Of<A>: Clone
```

The `Clone` bound is needed because `ref_bind(mma, |ma| ma.clone())`
receives `&Of<A>` and must return `Of<A>`. This is correct.

### 10.4 RefTraversable has A: Clone unlike RefFunctor

`RefTraversable::ref_traverse` requires `A: Clone`, while
`RefFunctor::ref_map` does not. This is because `ref_traverse`
implementations typically need to iterate by reference and reconstruct
elements, which may involve cloning when building the output context
via `Applicative::lift2`. The bound is justified for the general case.

### 10.5 No dispatch for ref_traverse

The unified dispatch system covers `map`/`bind`/`lift2`-`lift5`/`fold_map`/
`fold_right`/`fold_left` but not `traverse`. Both `traverse` and
`ref_traverse` remain as separate free functions. This is reasonable since
`traverse` has complex type parameters (two brands: the traversable and the
applicative) that make dispatch inference harder. However, it creates an
inconsistency where some operations dispatch automatically and others do
not.

### 10.6 RefTraversableWithIndex lacks FnBrand parameter

`RefTraversable::ref_traverse` takes a `FnBrand` type parameter, but
`RefTraversableWithIndex::ref_traverse_with_index` does not. Looking at
the by-value hierarchy, `Traversable::traverse` also lacks `FnBrand` while
`TraversableWithIndex::traverse_with_index` also lacks it. So this is
consistent with the by-value hierarchy. The `FnBrand` on
`RefTraversable::ref_traverse` appears to be there because some
implementations may need it for `Endofunction`-based folding in the
traversal, but it is not used by all implementors.

## Summary of Issues

### Must fix

1. **Stale documentation in RefFunctor**: The "Why `FnOnce`?" doc comment
   at lines 80-85 of `ref_functor.rs` describes the old `FnOnce` design.
   The signature now uses `Fn`. Update or remove this section.

2. **Phantom ref_sequence**: The `RefTraversable` doc comment claims
   `ref_sequence` exists with a default implementation. It does not exist.
   Correct the documentation or add the method.

### Should consider

3. **A: Clone on ref_fold_map is unnecessarily broad**: The `Clone` bound
   propagates to all callers even when implementations do not need it.
   Consider documenting this limitation or exploring whether the bound can
   be relaxed without breaking mutual derivation.

4. **RcLazy missing RefFoldable (or not?)**: The implementation uses
   `impl<Config: LazyConfig> RefFoldable for LazyBrand<Config>`, which
   covers both RcLazy and ArcLazy generically. This is correct, but the
   generic-over-Config approach means ArcLazy gets `RefFoldable` in addition
   to `SendRefFoldable`. This is fine since `RefFoldable` and
   `SendRefFoldable` are independent traits.

### Acceptable trade-offs

5. **FnOnce -> Fn on RefFunctor**: Correct; minor capability loss is
   acceptable for multi-element container support.

6. **Clone on RefPointed**: Unavoidable; correctly placed on the method.

7. **Independent Ref/by-value hierarchies**: Correct design; prevents
   forcing types to implement both.

8. **RefFilterable uses non-Ref Compactable**: Correct application of the
   "structural traits don't need Ref variants" principle.

9. **RefTraversable uses by-value Applicative for output**: Correct; the
   output context operates on newly created owned values.

10. **Blanket impls are correct and minimal**: All blanket impls match the
    by-value hierarchy structure.
