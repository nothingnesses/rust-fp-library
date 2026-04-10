# Plan: Ref Trait Expansion

## Motivation

The library's Ref trait hierarchy (RefFunctor, RefFoldable, RefTraversable,
etc.) enables by-reference operations on containers: closures receive `&A`
instead of `A`, and containers are borrowed instead of consumed. After the
ref-borrow refactor, all Ref traits take `&Self::Of<'a, A>`.

Seven by-value traits lack Ref counterparts. Analysis (see `analysis/`)
found that 5 of these are worth implementing and 2 should be skipped.

## Scope

### Implement (5 traits)

| Trait            | Implementors                                | Has closures   | Notes                                                                         |
| ---------------- | ------------------------------------------- | -------------- | ----------------------------------------------------------------------------- |
| RefBifunctor     | Result, Pair, Tuple2, ControlFlow, TryThunk | Yes (`f`, `g`) | No Clone needed; both closures handle their respective types.                 |
| RefBifoldable    | Result, Pair, Tuple2, ControlFlow, TryThunk | Yes (`f`, `g`) | Clone bounds already present in by-value trait (Endofunction defaults).       |
| RefBitraversable | Result, Pair, Tuple2, ControlFlow           | Yes (`f`, `g`) | TryThunk excluded (not Bitraversable). Requires RefBifunctor + RefBifoldable. |
| RefCompactable   | Option, Vec, CatList                        | No             | Requires `A: Clone` on methods (cloning out of `&Option<A>`).                 |
| RefAlt           | Option, Vec, CatList                        | No             | Requires `A: Clone`. Both args borrowed.                                      |

### Skip (2 traits)

| Trait      | Why skip                                                                                                                                                                          |
| ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| RefExtend  | Thunk cannot support it (extend captures the whole container; `evaluate` consumes `self`). Vec/CatList gain little (suffix materialization still needed). Only Identity benefits. |
| RefExtract | Only Identity can implement it (Thunk's `evaluate` consumes `self`). One implementor is too narrow for a trait abstraction.                                                       |

## Design

### RefBifunctor

```
pub trait RefBifunctor: Kind!( type Of<'a, A: 'a, B: 'a>: 'a; ) {
    fn ref_bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
        f: impl Fn(&A) -> B + 'a,
        g: impl Fn(&C) -> D + 'a,
        p: &Apply!(Self::Of<'a, A, C>),
    ) -> Apply!(Self::Of<'a, B, D>);
}
```

No Clone bounds needed on `A` or `C`. Both closures produce owned output
from references. For Result: `Ok(c)` calls `g(&c)`, `Err(a)` calls
`f(&a)`. For Pair/Tuple2: calls both `f` and `g` on the two fields.

**Derived RefFunctor for applied brands:** After RefBifunctor is
implemented, `BifunctorFirstAppliedBrand<Brand, A>` and
`BifunctorSecondAppliedBrand<Brand, B>` should get generic RefFunctor
impls that delegate to `ref_bimap` with `Clone::clone` on the fixed
side (requires Clone on the fixed type parameter). These coexist with
the existing manually-written RefFunctor impls for specific applied
brands (`ResultErrAppliedBrand`, `PairFirstAppliedBrand`, etc.)
because they are different types; no E0119 conflict. This matches the
by-value side where both specific and generic applied brands have
Functor impls.

### RefBifoldable

```
pub trait RefBifoldable: Kind!( type Of<'a, A: 'a, B: 'a>: 'a; ) {
    fn ref_bi_fold_right<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
        f: impl Fn(&A, C) -> C + 'a,
        g: impl Fn(&B, C) -> C + 'a,
        z: C,
        p: &Apply!(Self::Of<'a, A, B>),
    ) -> C;

    fn ref_bi_fold_left<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
        f: impl Fn(C, &A) -> C + 'a,
        g: impl Fn(C, &B) -> C + 'a,
        z: C,
        p: &Apply!(Self::Of<'a, A, B>),
    ) -> C;

    fn ref_bi_fold_map<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, M: Monoid + 'a>(
        f: impl Fn(&A) -> M + 'a,
        g: impl Fn(&B) -> M + 'a,
        p: &Apply!(Self::Of<'a, A, B>),
    ) -> M;
}
```

Clone bounds on `A` and `B` are already present in the by-value trait
(needed for Endofunction-based defaults). The Ref variant preserves them.
Default implementations follow the same mutual-derivation pattern as
the by-value trait.

### RefBitraversable

```
pub trait RefBitraversable: RefBifunctor + RefBifoldable {
    fn ref_bi_traverse<'a, FnBrand: LiftFn + 'a,
        A: 'a + Clone, B: 'a + Clone, C: 'a + Clone, D: 'a + Clone,
        F: Applicative>(
        f: impl Fn(&A) -> Apply!(F::Of<'a, C>) + 'a,
        g: impl Fn(&B) -> Apply!(F::Of<'a, D>) + 'a,
        p: &Apply!(Self::Of<'a, A, B>),
    ) -> Apply!(F::Of<'a, Apply!(Self::Of<'a, C, D>)>);

    fn ref_bi_sequence<'a, FnBrand: LiftFn + 'a,
        A: 'a + Clone, B: 'a + Clone,
        F: Applicative>(
        ta: &Apply!(Self::Of<'a, Apply!(F::Of<'a, A>), Apply!(F::Of<'a, B>)>),
    ) -> Apply!(F::Of<'a, Apply!(Self::Of<'a, A, B>)>);
}
```

Includes `FnBrand` following the RefTraversable pattern. Supertraits
require RefBifunctor + RefBifoldable, so all three must be implemented
together for each concrete type.

Free function variants: `ref_bi_traverse`, `ref_bi_sequence`,
`ref_traverse_left`, `ref_traverse_right`, `ref_bi_for`, `ref_for_left`,
`ref_for_right`.

### RefCompactable

```
pub trait RefCompactable: Kind!( type Of<'a, A: 'a>: 'a; ) {
    fn ref_compact<'a, A: 'a + Clone>(
        fa: &Apply!(Self::Of<'a, Option<A>>),
    ) -> Apply!(Self::Of<'a, A>);

    fn ref_separate<'a, E: 'a + Clone, O: 'a + Clone>(
        fa: &Apply!(Self::Of<'a, Result<O, E>>),
    ) -> (Apply!(Self::Of<'a, E>), Apply!(Self::Of<'a, O>));
}
```

Clone bounds on methods (not impl blocks) because the "other type"
(`Option`/`Result`) is not a brand type parameter. The by-value trait
does not need Clone (it moves values out); the Ref variant does.

No dispatch unification: these functions have no closures, so the
existing closure-type-based dispatch cannot apply. Separate `ref_compact`
and `ref_separate` free functions.

### RefAlt

```
pub trait RefAlt: RefFunctor {
    fn ref_alt<'a, A: 'a + Clone>(
        fa1: &Apply!(Self::Of<'a, A>),
        fa2: &Apply!(Self::Of<'a, A>),
    ) -> Apply!(Self::Of<'a, A>);
}
```

Both arguments borrowed. `A: Clone` required to construct the output.
For Option: clone whichever is `Some`. For Vec: clone and concatenate.

No dispatch unification (no closures). Separate `ref_alt` free function.

## Implementation Concerns

### Clone bounds for bifunctorial Ref traits

Not needed on `ref_bimap` itself (closures handle both sides). Clone IS
needed on derived RefFunctor impls for applied brands, placed on the
impl block (e.g., `impl<E: Clone + 'static> RefFunctor for ...`). This
follows the established pattern.

### SendRef and ParRef counterparts

None needed. None of the 5 traits apply to ArcLazy (the sole motivator
for SendRef) or to parallel collection operations (the motivator for
ParRef). The bifunctorial types (Result, Pair, Tuple2, ControlFlow,
TryThunk) are neither ArcLazy-based nor parallel-iterable.

### ArcLazy feasibility

Irrelevant. ArcLazy implements none of the 5 by-value traits.

### Module dependency ordering

No circular dependency risks. New traits go in `classes/`, implementations
go in `types/`. All dependency arrows point in the correct direction
(`brands -> classes -> types -> functions`).

## Implementation Order

1. **RefBifunctor** trait definition and impls for all 5 bifunctorial types.
   Also add derived RefFunctor impls for applied brands
   (`BifunctorFirstAppliedBrand`, `BifunctorSecondAppliedBrand`).

2. **RefBifoldable** trait definition and impls for all 5 bifunctorial
   types. Also add derived RefFoldable impls for applied brands.

3. **RefBitraversable** trait definition and impls for 4 types
   (TryThunk excluded). Also add derived RefTraversable impls for
   applied brands.

4. **RefCompactable** trait definition and impls for Option, Vec, CatList.

5. **RefAlt** trait definition and impls for Option, Vec, CatList.

6. **Tests.** Unit tests for each new trait. Property-based tests for
   RefBifunctor bimap identity and composition. Compile-fail tests if
   applicable.

7. **Documentation.** Doc comments on all new traits and methods following
   the existing documentation standards.

## Verification

After each step, run `just verify` (fmt, clippy, doc, test) with a
90-second timeout per command. The timeout guards against rustc trait
solver divergence.

## References

- Trait survey: `docs/plans/ref-expansion/analysis/trait-survey.md`
- Implementation concerns: `docs/plans/ref-expansion/analysis/implementation-concerns.md`
- Existing Ref trait pattern: `fp-library/src/classes/ref_functor.rs`,
  `ref_foldable.rs`, `ref_traversable.rs`
- Bifunctorial Clone bound pattern: `fp-library/src/types/result.rs`
  (RefFunctor impl for `ResultErrAppliedBrand<E: Clone>`)
