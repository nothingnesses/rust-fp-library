# Ref Expansion: Implementation Concerns

Analysis of Clone bounds, lifetime constraints, SendRef/ParRef
counterparts, ArcLazy feasibility, and other implementation concerns
for the proposed Ref variant traits.

## 1. Clone Bounds for Bifunctorial Types

### Issue

When mapping over a borrowed bifunctorial container like `&Result<A, E>`
with a Ref trait, the "other" type parameter must be cloned out of the
reference to construct the output. For example, `ref_map` on
`ResultErrAppliedBrand<E>` maps over `&Result<A, E>`: when the value is
`Ok(a)`, the closure receives `&A` and produces `B`; but when the value
is `Err(e)`, the `E` must be cloned from `&E` to produce `Result<B, E>`.

### Research Findings

The existing codebase handles this by placing `Clone + 'static` on the
impl block's type parameter, not on the method:

- `impl<E: Clone + 'static> RefFunctor for ResultErrAppliedBrand<E>` -
  clones `E` in the `Err` branch.
- `impl<T: Clone + 'static> RefFunctor for ResultOkAppliedBrand<T>` -
  clones `T` in the `Ok` branch.
- `impl<First: Clone + 'static> RefFunctor for PairFirstAppliedBrand<First>` -
  clones `First` when mapping over `Second`.
- `impl<Second: Clone + 'static> RefFunctor for PairSecondAppliedBrand<Second>` -
  clones `Second` when mapping over `First`.

The same pattern applies to RefFoldable, RefTraversable, RefSemimonad,
and other Ref traits on these types.

### Implications for RefBifunctor, RefBifoldable, RefBitraversable

For bifunctorial Ref traits, the situation is different. These traits
operate on the full two-parameter brand (e.g., `ResultBrand`) rather
than a partially-applied brand. The `bimap` signature takes two
closures, one for each type parameter.

A `RefBifunctor::ref_bimap` with signature:

```
fn ref_bimap<'a, A, B, C, D>(
    f: impl Fn(&A) -> B,
    g: impl Fn(&C) -> D,
    p: &Apply!(Self::Of<'a, A, C>),
) -> Apply!(Self::Of<'a, B, D>);
```

For `ResultBrand`, the implementation would be:

```
match p {
    Ok(c) => Ok(g(c)),
    Err(a) => Err(f(a)),
}
```

No Clone is needed because both branches have a closure to handle their
respective type. The closures receive `&A` and `&C` and produce owned
`B` and `D`. This is strictly better than the partially-applied case.

For `PairBrand` and `Tuple2Brand`, `ref_bimap` on `&Pair(a, b)` would
call `f(&a)` and `g(&b)`, again requiring no Clone.

**RefBifoldable** similarly needs no Clone: `ref_bi_fold_map` takes two
folding functions that each receive references. No values need cloning.

**RefBitraversable** is the same: `ref_bi_traverse` takes two effectful
functions receiving references. No extra Clone bounds are needed.

### Flaws and Limitations

The absence of Clone requirements is a positive finding, but there is a
subtlety: `Bitraversable` requires `Bifunctor + Bifoldable` as
supertraits. If `RefBitraversable` similarly requires
`RefBifunctor + RefBifoldable`, then all three must be implemented for a
type before `RefBitraversable` can be used. This is not a flaw per se,
but it means all three traits should be introduced together for each
concrete type.

### Recommendation

Implement RefBifunctor, RefBifoldable, and RefBitraversable without any
additional Clone bounds beyond what the by-value traits already require.
The two-closure design naturally avoids the "other parameter must be
cloned" problem that plagues partially-applied brands.

## 2. SendRef and ParRef Counterparts

### Issue

For each of the 7 new Ref traits, should there be SendRef and/or ParRef
counterparts?

### Research Findings

The existing pattern in `fp-library/src/classes/`:

**SendRef traits (13 found):**

- SendRefFunctor, SendRefFunctorWithIndex
- SendRefPointed
- SendRefLift
- SendRefSemiapplicative
- SendRefApplyFirst, SendRefApplySecond
- SendRefSemimonad, SendRefMonad
- SendRefFoldable, SendRefFoldableWithIndex
- SendRefApplicative
- SendRefCountedPointer

**ParRef traits (6 found):**

- ParRefFunctor, ParRefFunctorWithIndex
- ParRefFoldable, ParRefFoldableWithIndex
- ParRefFilterable, ParRefFilterableWithIndex

**Criterion for SendRef:** SendRef traits exist for traits where
`ArcLazy` needs to implement the Ref variant but cannot implement the
plain Ref variant (because `ArcLazy::new` requires `Send` on closures,
which Ref signatures do not guarantee). SendRef traits add `Send` bounds
to closures. `ArcLazy` is currently the sole implementor of most
SendRef traits.

**Criterion for ParRef:** ParRef traits exist for traits where rayon can
provide parallel execution. Currently limited to collection types
(Vec, CatList). ParRef traits add `Send + Sync` bounds on both closures
and elements. ParRef traits require the plain Ref trait as a supertrait
(unlike SendRef, which does not).

### Analysis for Each Proposed Trait

| Trait            | SendRef needed? | ParRef needed? | Rationale                                                                                                                                                                                                                                    |
| ---------------- | --------------- | -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| RefBifunctor     | No              | No             | ArcLazy is not bifunctorial. No collection types implement Bifunctor.                                                                                                                                                                        |
| RefBifoldable    | No              | No             | Same reasoning. Bifoldable types are Result, Pair, Tuple2, ControlFlow, TryThunk; none are ArcLazy or parallelizable collections.                                                                                                            |
| RefBitraversable | No              | No             | Same reasoning.                                                                                                                                                                                                                              |
| RefCompactable   | No              | Possible       | Vec and CatList implement Compactable and have ParCompactable already. A ParRefCompactable could parallel-compact by reference. However, Compactable::compact takes no closure, so the "by-ref" version has limited utility (see concern 6). |
| RefAlt           | No              | No             | Alt types are Option, Vec, CatList. ArcLazy does not implement Alt. ParRef for Alt would just be parallel concatenation, which is already handled by standard parallel operations.                                                           |
| RefExtend        | No              | No             | Extend types are Identity, Thunk, Vec, CatList. ArcLazy does not implement Extend. Parallel extend would require materializing suffixes in parallel, which is niche.                                                                         |
| RefExtract       | No              | No             | Extract types are Identity and Thunk. Neither is ArcLazy. Parallel extract is nonsensical (single value).                                                                                                                                    |

### Recommendation

None of the 7 new Ref traits need SendRef or ParRef counterparts.

The bifunctorial traits apply to types (Result, Pair, Tuple2,
ControlFlow, TryThunk) that are not ArcLazy and are not
parallel-iterable collections. Compactable, Alt, Extend, and Extract
similarly lack the use cases that motivate SendRef (ArcLazy) or ParRef
(rayon collections).

If a future type requires SendRef or ParRef for any of these traits, the
traits can be added at that time without breaking changes.

## 3. ArcLazy Feasibility

### Issue

ArcLazy cannot implement Ref traits that construct new containers
because `ArcLazy::new` requires `Send` on closures, but Ref trait
signatures use `impl Fn(&A) -> B + 'a` without a `Send` bound. ArcLazy
can implement Ref traits that only consume containers (e.g.,
RefFoldable, RefSemimonad).

### Analysis for Each Proposed Trait

**Traits that construct new containers (ArcLazy CANNOT implement):**

- **RefBifunctor** - Constructs a new `P<B, D>` from `&P<A, C>`.
  ArcLazy is not bifunctorial, so this is moot.
- **RefCompactable** - Constructs a new `F<A>` from `&F<Option<A>>`.
  ArcLazy does not implement Compactable, so this is moot.
- **RefAlt** - Constructs a new `F<A>` from `&F<A>` and `&F<A>`.
  ArcLazy does not implement Alt, so this is moot.
- **RefExtend** - Constructs a new `W<B>` from `&W<A>`.
  ArcLazy does not implement Extend, so this is moot.
- **RefBitraversable** - Constructs a new `G<P<B, D>>` from
  `&P<A, C>`. ArcLazy is not bitraversable, so this is moot.

**Traits that only consume (ArcLazy CAN implement):**

- **RefBifoldable** - Folds `&P<A, B>` to a monoid value without
  constructing a new container. ArcLazy is not bifoldable, so moot.
- **RefExtract** - Extracts `A` from `&W<A>` without constructing a
  new container. ArcLazy does not implement Extract. However, if it
  did, `ref_extract` returning `&A` would be natural for ArcLazy
  (since `evaluate()` returns `&A`). See concern 5.

### Recommendation

The ArcLazy construct-vs-consume distinction is irrelevant for all 7
proposed traits because ArcLazy does not implement any of the
corresponding by-value traits. No special accommodation is needed.

## 4. Extend's Closure Pattern

### Issue

`Extend::extend` has `f: impl Fn(W<A>) -> B` where the closure receives
the WHOLE container, not just an element. In a Ref variant, should the
closure receive `&W<A>`?

### Research Findings

The by-value `Extend::extend` signature is:

```
fn extend<'a, A: 'a + Clone, B: 'a>(
    f: impl Fn(Apply!(Self::Of<'a, A>)) -> B + 'a,
    wa: Apply!(Self::Of<'a, A>),
) -> Apply!(Self::Of<'a, B>);
```

Concrete implementations:

- **IdentityBrand**: `f(Identity(a))` - trivial, passes the whole
  Identity.
- **ThunkBrand**: `Thunk::new(move || f(thunk))` - captures the whole
  thunk and calls `f` when evaluated.
- **VecBrand**: For each suffix of the vector, materializes it as an
  owned `Vec<A>` and passes it to `f`. Requires `A: Clone`.
- **CatListBrand**: Same pattern as Vec; materializes each suffix as an
  owned `CatList<A>`.

### Analysis of `RefExtend`

A natural `RefExtend::ref_extend` would have:

```
fn ref_extend<'a, A: 'a, B: 'a>(
    f: impl Fn(&Apply!(Self::Of<'a, A>)) -> B + 'a,
    wa: &Apply!(Self::Of<'a, A>),
) -> Apply!(Self::Of<'a, B>);
```

The closure receives `&W<A>` (a reference to the whole container).

**IdentityBrand**: Straightforward. `f(&Identity(a))` produces `B`, wrap
in `Identity(B)`. No Clone needed for `A`.

**ThunkBrand**: Problematic. The thunk must be evaluated to provide
`&Thunk<A>` to `f`, but `Thunk::evaluate` consumes the thunk. A Ref
variant would need to evaluate the thunk and then pass a reference to
the _result_, not the thunk itself. This breaks the semantic contract.
Alternatively, we could pass `&Thunk<A>` directly, but then `f` would
receive an unevaluated thunk reference, which is semantically odd. This
implementation may not be feasible for Thunk.

**VecBrand**: For each index `i`, the closure would receive
`&vec[i..]` (a reference to a suffix). This is actually more efficient
than by-value extend because suffixes do not need to be materialized as
owned vectors. The closure receives a slice reference. However, the
signature says `&Vec<A>`, not `&[A]`. To provide `&Vec<A>` for each
suffix, we would need to materialize suffix vectors and pass references
to them, which defeats the purpose. Alternatively, if the Kind resolves
`Vec<A>` to `Vec<A>`, then `&Vec<A>` auto-derefs to `&[A]` in practice,
so the closure could still use slice methods.

Actually, the bigger issue is that `ref_extend` takes `&Vec<A>` as the
input container, but needs to produce `Vec<B>` as output. For each
suffix, we need a reference to that suffix. We cannot create `&Vec<A>`
references to suffixes without allocating suffix vectors. We could:

- Allocate suffix vectors and pass references (defeats the purpose of
  by-ref).
- Accept `&[A]` in the closure instead, but this changes the type
  signature away from the standard pattern.

**CatListBrand**: Same issues as Vec. Suffixes must be materialized.

### Flaws

1. Thunk cannot meaningfully implement RefExtend because `extend`'s
   semantics require passing the whole container by value (to be
   evaluated lazily).
2. Vec and CatList gain limited benefit from RefExtend because suffix
   materialization is still needed; only the _input_ container avoids
   cloning.
3. Identity is the only type where RefExtend is cleanly implementable.

### Approaches

**Approach A: Implement RefExtend with `f: impl Fn(&W<A>) -> B`.**
IdentityBrand implements it trivially. VecBrand materializes suffixes
as owned Vecs and passes `&Vec<A>` references (suffix materialization
still clones elements, but the input container is not consumed). Thunk
is excluded. This provides marginal value.

**Approach B: Skip RefExtend entirely.**
The primary benefit of Ref traits is avoiding consumption of the input
container. For Extend, the input container must be destructured into
suffixes regardless, so the by-ref advantage is minimal.

**Approach C: Implement RefExtend only for Identity.**
Minimal effort, covers the one clean case.

### Recommendation

Approach B: Skip RefExtend. The Extend pattern inherently requires
suffix materialization for collection types, and Thunk cannot support
it. The effort-to-benefit ratio is poor. If users need by-ref extend
for Identity, they can use `extend(f, identity.clone())` since Identity
is trivially cloneable.

## 5. Extract Return Type

### Issue

`Extract::extract` returns an owned `A`. Should `RefExtract::ref_extract`
return `&A` or owned `A`?

### Research Findings

Types that implement Extract:

- **IdentityBrand**: `extract` returns `fa.0` (unwraps the newtype).
  Could provide `&A` by returning `&fa.0`.
- **ThunkBrand**: `extract` returns `fa.evaluate()`, which runs the
  thunk and returns an owned `A`. Cannot return `&A` because the value
  does not exist until the thunk is evaluated, and the thunk is consumed.

Note: `LazyBrand` does NOT implement Extract. The `Extract` doc comment
explicitly states: "Lazy cannot implement this trait because forcing it
returns `&A`, not an owned `A`."

### Analysis

**Option 1: Return `&A` (reference).**

- Identity can do this (return `&fa.0`).
- Thunk cannot: `evaluate()` consumes the thunk and returns owned `A`.
  There is no stored `A` to borrow.
- If we required `&A`, only Identity could implement `RefExtract`.

**Option 2: Return owned `A` with `A: Clone`.**

- Identity can clone from `&fa.0`.
- Thunk still cannot: you cannot evaluate a `&Thunk` because `evaluate`
  takes `self` by value.
- This approach adds a Clone bound but does not help Thunk.

**Option 3: Return owned `A` without Clone, but the trait takes `&W<A>`.**

- This is only possible if the type can produce an owned `A` from a
  reference without Clone. This is generally impossible.

### Flaws

The fundamental problem is that Thunk's `extract` is inherently
consuming: it runs a closure and returns the result. There is no stored
value to borrow. A `RefExtract` trait would exclude Thunk entirely,
leaving only Identity as an implementor.

With only one implementor, the trait provides no useful abstraction.
Generic code constrained by `RefExtract` would effectively only work
with Identity, which is too narrow to justify a trait.

### Recommendation

Skip RefExtract. With only Identity as a viable implementor, the trait
provides insufficient generality. Users who need to extract from
`&Identity<A>` can simply access `.0` directly.

## 6. Compactable Without Closures

### Issue

`Compactable::compact` and `Compactable::separate` take no closures.
`compact` takes `F<Option<A>>` and returns `F<A>` (keeping `Some`
values). `separate` takes `F<Result<O, E>>` and returns
`(F<E>, F<O>)`.

In a Ref variant, the container would be borrowed: `&F<Option<A>>`. The
inner `Option<A>` values are accessed by reference as `&Option<A>`. To
extract `A` from `&Some(A)`, `A` must be cloned.

### Analysis

A `RefCompactable::ref_compact` signature would be:

```
fn ref_compact<'a, A: 'a + Clone>(
    fa: &Apply!(Self::Of<'a, Option<A>>),
) -> Apply!(Self::Of<'a, A>);
```

**VecBrand**: Iterate `&Vec<Option<A>>`, for each `Some(a)`, clone `a`
into the output vector. This is equivalent to
`v.iter().filter_map(|opt| opt.as_ref().map(Clone::clone)).collect()`.

**OptionBrand**: For `&Option<Option<A>>`, flatten by cloning. If
`Some(Some(a))`, return `Some(a.clone())`. If `Some(None)` or `None`,
return `None`.

**CatListBrand**: Same pattern as Vec.

Similarly for `ref_separate` on `&F<Result<O, E>>`: both `O` and `E`
must be cloned out.

### Clone Requirements

The by-value `Compactable::compact` does NOT require `A: Clone` because
it moves values out of the container. RefCompactable would add a
`Clone` bound that the by-value version does not have.

### Is This Useful?

The primary benefit of Ref traits is avoiding consumption of the input
container. RefCompactable would:

- Preserve the input container (good).
- Require cloning every `Some` value (cost).
- Require `A: Clone` (restrictive).

The use case is: "I have a `&Vec<Option<A>>` and want to compact it
without consuming the vector." This is reasonable for shared data or
when the vector is used again afterward. The Clone cost is unavoidable
since we cannot move values out of a reference.

### Approaches

**Approach A: Implement RefCompactable with `A: Clone`.**
Useful for Vec and CatList where the container may be shared or reused.
The Clone bound is on the method, not the impl block (unlike bifunctorial
Ref traits), because the "other type" is always `Option` or `Result`,
which are not type parameters of the brand.

**Approach B: Skip RefCompactable.**
Users can manually clone the container and use by-value compact. The
ergonomic gain of RefCompactable is modest.

### Recommendation

Approach A: Implement RefCompactable with `A: Clone` on the method.
The pattern is common enough (compacting shared collections) and the
Clone bound is explicit and expected. For `ref_separate`, require both
`O: Clone` and `E: Clone`.

## 7. Module Dependency Ordering

### Issue

The library has a strict dependency graph:
`brands -> classes -> types -> functions`. Adding new Ref traits in
`classes/` and new impls in `types/` must respect this ordering.

### Analysis

**New trait definitions** (in `classes/`):

- `ref_bifunctor.rs`, `ref_bifoldable.rs`, `ref_bitraversable.rs`
- `ref_compactable.rs`
- `ref_alt.rs`

These traits depend on:

- `kinds` module (for `Kind!`, `Apply!` macros) - OK, `kinds` is at the
  same level as `brands`.
- Other traits in `classes/` (supertraits) - OK, traits within `classes/`
  can depend on each other.
- `brands` module (e.g., `OptionBrand` in Compactable) - OK, `brands`
  precedes `classes`.

**New trait implementations** (in `types/`):

- Implementations in `result.rs`, `pair.rs`, `tuple_2.rs`,
  `control_flow.rs`, `try_thunk.rs`, `option.rs`, `vec.rs`,
  `cat_list.rs`.

These depend on:

- The trait definitions in `classes/` - OK, `classes` precedes `types`.
- Brand types in `brands/` - OK, `brands` precedes `types`.
- Concrete types defined in the same `types/` files - OK, same module.

### Circular Dependency Risks

**RefBitraversable supertraits:** If `RefBitraversable: RefBifunctor + RefBifoldable`, all three traits must be in `classes/` and the
supertrait relationship creates no cycle (all within `classes/`).

**RefCompactable and OptionBrand:** The by-value `Compactable` already
references `OptionBrand` in its signature. RefCompactable would do the
same. This works because `brands` precedes `classes`. No cycle.

**RefAlt supertraits:** If `RefAlt: RefFunctor`, this is a dependency
within `classes/`. No cycle.

**Free functions:** If free functions for new Ref traits are defined in
the same `classes/` module files (following the existing pattern), they
create no new dependencies.

### Recommendation

No circular dependency risks exist. The proposed additions follow the
established `brands -> classes -> types -> functions` ordering. All
new traits go in `classes/`, all implementations go in `types/`, and
the dependency arrows point in the correct direction.

## Summary

| Concern                                     | Finding                                                        | Recommendation                              |
| ------------------------------------------- | -------------------------------------------------------------- | ------------------------------------------- |
| 1. Clone bounds for bifunctorial Ref traits | Not needed; two-closure design avoids the problem.             | Implement without extra Clone bounds.       |
| 2. SendRef/ParRef counterparts              | None of the 7 traits apply to ArcLazy or parallel collections. | Skip all SendRef and ParRef counterparts.   |
| 3. ArcLazy feasibility                      | Irrelevant; ArcLazy implements none of the 7 by-value traits.  | No accommodation needed.                    |
| 4. Extend's closure pattern                 | Thunk cannot support RefExtend; Vec/CatList gain little.       | Skip RefExtend entirely.                    |
| 5. Extract return type                      | Only Identity can implement RefExtract; Thunk cannot.          | Skip RefExtract entirely.                   |
| 6. Compactable without closures             | Needs `A: Clone` on the method. Useful for shared collections. | Implement RefCompactable with Clone bounds. |
| 7. Module dependency ordering               | No circular dependency risks.                                  | Standard placement in classes/ and types/.  |

Net result: of the 7 originally proposed traits, this analysis
recommends implementing 4 Ref variants (RefBifunctor, RefBifoldable,
RefBitraversable, RefCompactable, RefAlt) and skipping 2
(RefExtend, RefExtract). No SendRef or ParRef counterparts are needed.
