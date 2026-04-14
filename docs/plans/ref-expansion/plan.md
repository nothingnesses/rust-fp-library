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

| Trait            | Implementors                      | Has closures   | Notes                                                                   |
| ---------------- | --------------------------------- | -------------- | ----------------------------------------------------------------------- |
| RefBifunctor     | Result, Pair, Tuple2, ControlFlow | Yes (`f`, `g`) | No Clone needed; both closures handle their respective types.           |
| RefBifoldable    | Result, Pair, Tuple2, ControlFlow | Yes (`f`, `g`) | Clone bounds already present in by-value trait (Endofunction defaults). |
| RefBitraversable | Result, Pair, Tuple2, ControlFlow | Yes (`f`, `g`) | Requires RefBifunctor + RefBifoldable as supertraits.                   |
| RefCompactable   | Option, Vec, CatList              | No             | Requires `A: Clone` on methods (cloning out of `&Option<A>`).           |
| RefAlt           | Option, Vec, CatList              | No             | Requires `A: Clone`. Both args borrowed.                                |

TryThunk is excluded from ALL Ref bi-traits (not just RefBitraversable).
TryThunk wraps `Box<dyn FnOnce>` via Thunk, which cannot be evaluated
or chained from a shared reference. This matches how ThunkBrand lacks
RefFunctor.

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
`ref_bi_traverse_left`, `ref_bi_traverse_right`, `ref_bi_for`,
`ref_bi_for_left`, `ref_bi_for_right`. The `bi` prefix is retained in
all names to avoid confusion with single-parameter traversal functions.

Note: these free function names are temporary canonical names. After the
dispatch-expansion plan adds bifunctorial dispatch, these names will be
demoted to module-prefixed aliases (e.g., `ref_bitraversable_ref_bi_traverse`)
and the dispatch version will take the canonical name.

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

Steps 1-3 MUST be sequential: derived applied-brand impls chain across
steps (RefTraversable for applied brands requires RefFunctor from step 1
and RefFoldable from step 2). Steps 4-5 are independent of steps 1-3
and of each other.

Documentation (all five `#[document_*]` attributes) must be applied to
every public method and free function from the start, not deferred.
The zero-warnings policy for `just doc` requires this.

1. **RefBifunctor** trait definition and impls for 4 bifunctorial types
   (Result, Pair, Tuple2, ControlFlow). Also add derived RefFunctor
   impls for applied brands (`BifunctorFirstAppliedBrand`,
   `BifunctorSecondAppliedBrand`) with `Brand: Bifunctor + RefBifunctor`
   and `A: Clone + 'static` bounds.

2. **RefBifoldable** trait definition and impls for 4 bifunctorial
   types. Also add derived RefFoldable impls for applied brands.

3. **RefBitraversable** trait definition and impls for 4 types.
   Also add derived RefTraversable impls for applied brands.

4. **RefCompactable** trait definition and impls for Option, Vec, CatList.

5. **RefAlt** trait definition and impls for Option, Vec, CatList.

6. **Tests.** Property-based quickcheck tests in type files following
   the established pattern. Also add equivalence tests between generic
   applied brands and specific applied brands (e.g., verify
   `BifunctorFirstAppliedBrand<ResultBrand, E>` and
   `ResultErrAppliedBrand<E>` produce the same `ref_map` results).

   Existing gap: `result.rs` has Ref trait impls (RefFunctor, RefFoldable,
   RefTraversable) but zero quickcheck tests for them. This should be
   addressed alongside or before this plan.

   Laws to test per trait:
   - **RefBifunctor:** Identity (`ref_bimap(Clone::clone, Clone::clone, &p)
== p.clone()` given `A: Clone, C: Clone`). Composition.
   - **RefBifoldable:** Consistency between `ref_bi_fold_map` and
     `ref_bi_fold_right`.
   - **RefBitraversable:** `ref_bi_traverse(f, g, &p) ==
ref_bi_sequence(&ref_bimap(f, g, &p))`.
   - **RefCompactable:** `ref_compact(&ref_map(Some, &fa))` preserves
     values given `A: Clone`.
   - **RefAlt:** Associativity. Distributivity with RefFunctor.

   Compile-fail tests: RefCompactable and RefAlt with non-Clone element
   types should produce clear errors.

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
