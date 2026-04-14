# Ref Expansion: Open Questions Investigation 2

Concrete implementation feasibility analysis for each proposed Ref trait
and each listed concrete type.

## 1. RefBifunctor for Each of the 5 Types

### 1.1 ResultBrand

**By-value impl** (`types/result.rs:83-92`):

```
match p {
    Ok(c) => Ok(g(c)),
    Err(a) => Err(f(a)),
}
```

**Ref sketch**:

```
match p {
    Ok(c) => Ok(g(c)),     // g receives &C
    Err(a) => Err(f(a)),   // f receives &A
}
```

**Issues**: None. Both branches have a closure to handle their respective
reference. No Clone needed.

### 1.2 PairBrand

**By-value impl** (`types/pair.rs:436-442`): delegates to `p.bimap(f, g)`
which produces `Pair(f(self.0), g(self.1))`.

**Ref sketch**:

```
Pair(f(&p.0), g(&p.1))
```

**Issues**: None. Both closures receive references and produce owned
output. Both fields are always present, so both closures are always
called.

### 1.3 Tuple2Brand

**By-value impl** (`types/tuple_2.rs:71-77`): `(f(p.0), g(p.1))`.

**Ref sketch**:

```
(f(&p.0), g(&p.1))
```

**Issues**: None. Identical to Pair.

### 1.4 ControlFlowBrand

**By-value impl** (`types/control_flow.rs:741-747`): delegates to a helper
that matches `Continue(c) => Continue(f(c))` and `Break(b) => Break(g(b))`.

**Ref sketch**:

```
match p {
    ControlFlow::Continue(c) => ControlFlow::Continue(f(c)),
    ControlFlow::Break(b) => ControlFlow::Break(g(b)),
}
```

**Issues**: None. Same enum-match pattern as Result.

### 1.5 TryThunkBrand

**By-value impl** (`types/try_thunk.rs:1494-1503`):

```
TryThunk(p.0.map(move |result| match result {
    Ok(c) => Ok(g(c)),
    Err(a) => Err(f(a)),
}))
```

This works because `Thunk::map(self, f)` consumes `self` and wraps
the computation in a new `Thunk` that applies `f` after evaluation.

**Ref sketch**: `ref_bimap` receives `&TryThunk<'a, A, E>`. Internally,
`TryThunk` wraps `Thunk<'a, Result<A, E>>`, which wraps
`Box<dyn FnOnce() -> Result<A, E> + 'a>`. Thunk is not Clone (FnOnce
cannot be cloned). Therefore:

- We cannot call `p.0.map(...)` because `map` consumes `self`.
- We cannot clone the Thunk to avoid consuming it.
- We cannot evaluate the Thunk from a shared reference because
  `evaluate(self)` consumes `self`.

**Issue**: RefBifunctor CANNOT be implemented for TryThunkBrand.

The fundamental problem is that TryThunk is lazy and non-cloneable. Its
by-value Bifunctor impl works by chaining a new closure onto the existing
one (via `Thunk::map`), consuming the original. From a shared reference,
no such chaining is possible.

**Approaches**:

- **A: Exclude TryThunkBrand from RefBifunctor.** This mirrors how Thunk
  lacks RefFunctor. The plan already excludes TryThunk from
  RefBitraversable (since TryThunk is not Bitraversable). Excluding it
  from RefBifunctor is consistent.
- **B: Add a `clone_thunk` or make the inner Thunk use `Rc<dyn Fn()>`
  instead of `Box<dyn FnOnce()>`.** This changes TryThunk's semantics
  and performance characteristics, which is a much larger design change.

**Recommendation**: Approach A. Exclude TryThunkBrand from RefBifunctor.
Update the plan to list only 4 types for RefBifunctor (Result, Pair,
Tuple2, ControlFlow).

### RefBifunctor Summary

| Type        | Feasible | Notes                    |
| ----------- | -------- | ------------------------ |
| Result      | Yes      | No issues.               |
| Pair        | Yes      | No issues.               |
| Tuple2      | Yes      | No issues.               |
| ControlFlow | Yes      | No issues.               |
| TryThunk    | No       | Non-cloneable lazy type. |

---

## 2. RefBifoldable for Each of the 5 Types

### 2.1 ResultBrand

**By-value impl** (`types/result.rs:144-154`):

```
match p {
    Err(a) => f(a, z),
    Ok(b) => g(b, z),
}
```

**Ref sketch**:

```
match p {
    Err(a) => f(a, z),    // f receives &A
    Ok(b) => g(b, z),     // g receives &B
}
```

**Issues**: None. The closures receive references. The accumulator `z`
is passed by value as before.

### 2.2 PairBrand

**By-value impl** (`types/pair.rs:482-489`): delegates to
`p.bi_fold_right(f, g, z)` which computes `f(self.0, g(self.1, z))`.

**Ref sketch**:

```
f(&p.0, g(&p.1, z))
```

**Issues**: None. Both closures receive references to the pair fields.

### 2.3 Tuple2Brand

**By-value impl** (analogous to Pair): `f(p.0, g(p.1, z))`.

**Ref sketch**: `f(&p.0, g(&p.1, z))`.

**Issues**: None.

### 2.4 ControlFlowBrand

**By-value impl** (`types/control_flow.rs:794-801`): same match pattern
as Result.

**Ref sketch**: Same pattern with references.

**Issues**: None.

### 2.5 TryThunkBrand

**By-value impl** (`types/try_thunk.rs:1557-1567`):

```
match p.evaluate() {
    Err(a) => f(a, z),
    Ok(b) => g(b, z),
}
```

`p.evaluate()` consumes `p` (Thunk::evaluate takes self). From
`&TryThunk`, we cannot call `evaluate()`.

**Issue**: RefBifoldable CANNOT be implemented for TryThunkBrand.

Same root cause as RefBifunctor: TryThunk's inner Thunk is non-cloneable
and evaluation is consuming.

**Approaches**: Same as section 1.5.

**Recommendation**: Exclude TryThunkBrand from RefBifoldable. Update the
plan to list 4 types.

### Endofunction-Based Defaults

The plan proposes that RefBifoldable have mutual defaults mirroring the
by-value trait. The by-value defaults use `Endofunction` and require
`A: Clone + B: Clone` because elements are captured in closures.

For the Ref variant, the defaults would capture `&A` and `&B` in
closures. This works if `A: Clone` and `B: Clone` are still required,
because the closures need to clone the references to own the values.
Specifically, in `ref_bi_fold_right`'s default (derived from
`ref_bi_fold_map`), the closure captures `&a` and clones it to `a`:

```
move |a: &A| {
    let a = a.clone();
    let f = f.clone();
    Endofunction::new(LiftFn::new(move |c| f((&a, c))))
}
```

Wait: this changes the semantics. In the by-value default, `a` is moved
into the closure. In the ref variant, `a` is a reference that must be
cloned. But the Clone bound is already on the trait method (`A: Clone`),
so this works.

**Issue**: The Endofunction-based defaults for RefBifoldable require
`A: Clone` and `B: Clone`. The plan already includes these bounds, so
no change is needed.

For the 4 concrete types that will implement RefBifoldable (Result, Pair,
Tuple2, ControlFlow), direct impls would be provided (not relying on
defaults), since they are simple match/field-access patterns. The Clone
bounds would still be present on the method signatures for trait
consistency but would not actually be used in the direct impls.

### RefBifoldable Summary

| Type        | Feasible | Notes                    |
| ----------- | -------- | ------------------------ |
| Result      | Yes      | No issues.               |
| Pair        | Yes      | No issues.               |
| Tuple2      | Yes      | No issues.               |
| ControlFlow | Yes      | No issues.               |
| TryThunk    | No       | Evaluation is consuming. |

---

## 3. RefBitraversable for Each of the 4 Types

TryThunk is already excluded by the plan (it is not Bitraversable at
all).

### 3.1 ResultBrand

**By-value impl** (`types/result.rs:325-342`):

```
match p {
    Err(a) => F::map(|c| Err(c), f(a)),
    Ok(b) => F::map(|d| Ok(d), g(b)),
}
```

**Ref sketch**:

```
match p {
    Err(a) => F::map(|c| Err(c), f(a)),   // f receives &A
    Ok(b) => F::map(|d| Ok(d), g(b)),     // g receives &B
}
```

**Issues**: None. `F::map` takes an owned applicative value (the result
of `f(a)` or `g(b)`), which is already produced by the closures. No
Clone needed on A or B for the direct impl.

### 3.2 PairBrand

**By-value impl** (`types/pair.rs:627-641`): delegates to
`p.bi_traverse::<C, D, F>(f, g)` which uses `F::lift2(|c, d| Pair(c, d), f(self.0), g(self.1))`.

**Ref sketch**:

```
F::lift2(|c, d| Pair(c, d), f(&p.0), g(&p.1))
```

**Issues**: `F::lift2` requires `C: Clone` and `D: Clone` per the Lift
trait definition. These bounds are already present on the by-value
Bitraversable signature (`C: Clone, D: Clone`). The Ref variant would
carry the same bounds.

One subtlety: `F::lift2` takes two applicative values by value. Both
`f(&p.0)` and `g(&p.1)` produce owned `Apply!(F::Of<'a, C>)` and
`Apply!(F::Of<'a, D>)`, so this works without issue.

### 3.3 Tuple2Brand

**By-value impl** (`types/tuple_2.rs:270-285`):

```
let (a, b) = p;
F::lift2(|c, d| (c, d), f(a), g(b))
```

**Ref sketch**:

```
F::lift2(|c, d| (c, d), f(&p.0), g(&p.1))
```

**Issues**: None. Same analysis as Pair.

### 3.4 ControlFlowBrand

**By-value impl** (`types/control_flow.rs:951-965`): same match pattern
as Result, using `F::map`.

**Ref sketch**: Same pattern with references.

**Issues**: None.

### Applicative Machinery Concerns

The `ref_bi_traverse` signature in the plan includes `FnBrand: LiftFn`,
following the `RefTraversable` pattern. For the 4 concrete types listed,
`FnBrand` is needed for default methods (e.g., `ref_bi_sequence` derived
from `ref_bi_traverse`), not for the direct `ref_bi_traverse` impls
themselves. This matches how `RefTraversable` works: concrete impls
ignore `FnBrand`, but the trait signature includes it for the defaults.

The `ref_bi_sequence` default would call
`ref_bi_traverse(Clone::clone, Clone::clone, ta)`, where the cloned
values are `Apply!(F::Of<'a, A>)` and `Apply!(F::Of<'a, B>)`. These
must be `Clone`, which the by-value trait also requires.

**Issue**: The `ref_bi_sequence` signature borrows `&P<F<A>, F<B>>`. To
call `Clone::clone` on the inner `F<A>` and `F<B>`, we need access to
`&F<A>` and `&F<B>`. For Result: `&Result<F<B>, F<A>>` provides
`&F<A>` via `Err(fa)` and `&F<B>` via `Ok(fb)`. So `Clone::clone`
receives a reference and produces an owned value. This works.

For Pair/Tuple2: `&Pair<F<A>, F<B>>` provides `&p.0` and `&p.1`, which
are `&F<A>` and `&F<B>`. `Clone::clone` works fine.

No issues with the applicative machinery.

### RefBitraversable Summary

| Type        | Feasible | Notes                     |
| ----------- | -------- | ------------------------- |
| Result      | Yes      | No issues.                |
| Pair        | Yes      | Needs C: Clone, D: Clone. |
| Tuple2      | Yes      | Needs C: Clone, D: Clone. |
| ControlFlow | Yes      | No issues.                |

---

## 4. RefCompactable for Option, Vec, CatList

### 4.1 OptionBrand

**By-value impl** (`types/option.rs:641-648`): `fa.flatten()` on
`Option<Option<A>>`.

**Ref sketch**: `ref_compact` receives `&Option<Option<A>>`.

```
match fa {
    Some(Some(a)) => Some(a.clone()),
    _ => None,
}
```

**Issues**: Requires `A: Clone`. Straightforward.

**ref_separate on Option**: receives `&Option<Result<O, E>>`.

```
match fa {
    Some(Ok(o)) => (None, Some(o.clone())),
    Some(Err(e)) => (Some(e.clone()), None),
    None => (None, None),
}
```

Requires `O: Clone` and `E: Clone`. Straightforward.

### 4.2 VecBrand

**By-value impl** (`types/vec.rs:1492-1499`):
`fa.into_iter().flatten().collect()`.

**Ref sketch**: `ref_compact` receives `&Vec<Option<A>>`.

```
fa.iter()
    .filter_map(|opt| opt.as_ref().map(Clone::clone))
    .collect()
```

**Issues**: Requires `A: Clone`. Straightforward. The `iter()` method
provides `&Option<A>`, and `as_ref()` gives `Option<&A>`, which is then
cloned.

**ref_separate on Vec**: receives `&Vec<Result<O, E>>`.

```
let mut oks = Vec::new();
let mut errs = Vec::new();
for result in fa.iter() {
    match result {
        Ok(o) => oks.push(o.clone()),
        Err(e) => errs.push(e.clone()),
    }
}
(errs, oks)
```

Requires `O: Clone` and `E: Clone`. Straightforward.

### 4.3 CatListBrand

**By-value impl** (`types/cat_list.rs:1260-1267`):
`fa.into_iter().flatten().collect()`.

**Ref sketch**: `ref_compact` receives `&CatList<Option<A>>`.

```
fa.iter()
    .filter_map(|opt| opt.as_ref().map(Clone::clone))
    .collect()
```

CatList has `iter()` returning `CatListIter<'_, A>` which yields `&A`
items (verified in the codebase). CatList also implements
`FromIterator`, so `.collect()` works.

**Issues**: Requires `A: Clone`. No issues.

**ref_separate on CatList**: same pattern as Vec, using `fa.iter()`,
matching on `Result` references, and cloning.

Requires `O: Clone` and `E: Clone`. No issues.

### RefCompactable Summary

| Type    | Feasible | Notes                             |
| ------- | -------- | --------------------------------- |
| Option  | Yes      | Requires A: Clone (method bound). |
| Vec     | Yes      | Requires A: Clone (method bound). |
| CatList | Yes      | Requires A: Clone (method bound). |

All three are straightforward. The Clone cost is inherent and unavoidable
when extracting values from borrowed containers.

---

## 5. RefAlt for Option, Vec, CatList

### 5.1 OptionBrand

**By-value impl** (`types/option.rs:251-256`): `fa1.or(fa2)`.

**Ref sketch**: `ref_alt` receives `&Option<A>` and `&Option<A>`.

```
match (fa1, fa2) {
    (Some(a), _) => Some(a.clone()),
    (None, Some(a)) => Some(a.clone()),
    (None, None) => None,
}
```

Or more concisely: `fa1.as_ref().or(fa2.as_ref()).cloned()`.

**Issues**: Requires `A: Clone`. Straightforward.

### 5.2 VecBrand

**By-value impl** (`types/vec.rs:322-329`):

```
let mut result = fa1;
result.extend(fa2);
result
```

**Ref sketch**: `ref_alt` receives `&Vec<A>` and `&Vec<A>`.

```
let mut result = fa1.clone();
result.extend(fa2.iter().cloned());
result
```

Or: `fa1.iter().chain(fa2.iter()).cloned().collect()`.

**Issues**: Requires `A: Clone`. Both vectors must be fully cloned to
produce the output. This is strictly more expensive than the by-value
version (which moves elements). However, this is the unavoidable cost
of working from references.

### 5.3 CatListBrand

**By-value impl** (`types/cat_list.rs:499-504`): `fa1.append(fa2)`.

CatList's `append` creates a shared structure (CatList uses `Rc`
internally for structural sharing). The by-value version consumes both
lists but can share their internal nodes.

**Ref sketch**: `ref_alt` receives `&CatList<A>` and `&CatList<A>`.

```
fa1.clone().append(fa2.clone())
```

**Issues**: Requires `A: Clone` on the method bound. However, there is a
subtlety: CatList uses `Rc`-based structural sharing internally. Cloning
a `CatList` is O(1) because it just bumps the reference count. The
`append` operation is also O(1) because it creates a new `Append` node
pointing to the two shared sub-lists.

So `fa1.clone().append(fa2.clone())` is O(1) regardless of list size,
without actually cloning any elements. The `A: Clone` bound on the
method would be required by the trait signature but would not actually
be exercised in the CatList implementation.

**Issue**: The `A: Clone` bound on `ref_alt` is overly restrictive for
CatList, where the operation is structurally cheap. However, the trait
must have a uniform signature, and the bound is necessary for Option and
Vec. This is a minor ergonomic issue; users of CatList who want ref_alt
would need their element type to be Clone even though elements are not
actually cloned.

**Approaches**:

- **A: Keep the uniform `A: Clone` bound.** Simpler trait definition.
  CatList users who need ref_alt on non-Clone types can call
  `fa1.clone().append(fa2.clone())` directly (since CatList::clone is
  cheap and does not require element Clone).
- **B: Remove `A: Clone` from the trait and let implementors decide.**
  This breaks the pattern established by the plan and makes the trait
  less predictable. Implementors that need Clone would add it on the
  impl block instead.

**Recommendation**: Approach A. The uniform `A: Clone` bound is
consistent with RefCompactable and matches user expectations for Ref
traits. The CatList case is a minor ergonomic wart, not a blocker.

### RefAlt Summary

| Type    | Feasible | Notes                                         |
| ------- | -------- | --------------------------------------------- |
| Option  | Yes      | Requires A: Clone.                            |
| Vec     | Yes      | Requires A: Clone. Full element cloning.      |
| CatList | Yes      | Requires A: Clone (unused). Cheap O(1) clone. |

---

## 6. Generic RefFunctor Derivation for Applied Bifunctor Brands

### Current Pattern (By-Value)

In `classes/bifunctor.rs` (lines 166-246), two generic Functor impls are
defined:

```
impl<Brand: Bifunctor, A: 'static> Functor for BifunctorFirstAppliedBrand<Brand, A> {
    fn map(f, fa) { Brand::bimap(identity, f, fa) }
}

impl<Brand: Bifunctor, B: 'static> Functor for BifunctorSecondAppliedBrand<Brand, B> {
    fn map(f, fa) { Brand::bimap(f, identity, fa) }
}
```

These coexist with specific Functor impls on concrete applied brands
(e.g., `ResultErrAppliedBrand<E>`, `PairFirstAppliedBrand<First>`)
because the types are structurally different. `BifunctorFirstAppliedBrand`
and `ResultErrAppliedBrand` are distinct structs.

### Proposed RefFunctor Derivation

The plan proposes analogous generic RefFunctor impls:

```
impl<Brand: RefBifunctor, A: Clone + 'static> RefFunctor
    for BifunctorFirstAppliedBrand<Brand, A>
{
    fn ref_map(f, fa) {
        Brand::ref_bimap(Clone::clone, f, fa)
    }
}
```

For `BifunctorFirstAppliedBrand<Brand, A>`, this maps over the second
type parameter while cloning the first. The fixed parameter `A` must be
`Clone` because `ref_bimap` receives `&A` in the first closure and must
produce an owned `A`.

### E0119 (Overlapping Impls) Analysis

The generic impl is on `BifunctorFirstAppliedBrand<Brand, A>`. The
specific impls are on `ResultErrAppliedBrand<E>`,
`PairFirstAppliedBrand<First>`, `Tuple2FirstAppliedBrand<First>`, etc.

These are all different struct types. Rust's coherence rules check for
overlap based on the `Self` type. Since `BifunctorFirstAppliedBrand` is a
different struct from `ResultErrAppliedBrand`, there is no overlap.

**Issue**: No E0119 conflict. The generic and specific impls target
different types.

### Clone Bound Consistency

The specific RefFunctor impls already require Clone on the fixed type:

- `impl<E: Clone + 'static> RefFunctor for ResultErrAppliedBrand<E>`
- `impl<First: Clone + 'static> RefFunctor for PairFirstAppliedBrand<First>`
- `impl<First: Clone + 'static> RefFunctor for Tuple2FirstAppliedBrand<First>`

The generic impl would similarly require `A: Clone + 'static`. This is
consistent.

### Practical Value

The generic RefFunctor impl allows users who work with
`BifunctorFirstAppliedBrand<SomeBrand, A>` (the generic adapter) to use
`ref_map` without needing to know which specific applied brand to use.
This mirrors the by-value side.

### Issue: Brand Must Implement Both Bifunctor AND RefBifunctor

The generic Functor impl requires `Brand: Bifunctor`. The generic
RefFunctor impl would require `Brand: RefBifunctor`. Since TryThunkBrand
cannot implement RefBifunctor (per section 1.5),
`BifunctorFirstAppliedBrand<TryThunkBrand, A>` would not get a RefFunctor
impl. This is correct behavior: if the underlying bifunctor does not
support ref operations, neither should its applied brands.

However, `BifunctorFirstAppliedBrand<TryThunkBrand, A>` would still have
a Functor impl (from the by-value side). So `map` works but `ref_map`
does not. This is consistent with how TryThunk's specific applied brands
(`TryThunkErrAppliedBrand<E>`) also lack RefFunctor.

### Issue: The Generic Impl Requires Bifunctor + RefBifunctor

The `Kind` impl for `BifunctorFirstAppliedBrand<Brand, A>` requires
`Brand: Bifunctor`. If RefBifunctor has a different supertrait (or no
Bifunctor supertrait), the impl block for RefFunctor might need:

```
impl<Brand: Bifunctor + RefBifunctor, A: Clone + 'static>
    RefFunctor for BifunctorFirstAppliedBrand<Brand, A>
```

This requires the Brand to implement both. This is fine in practice
because every type that implements RefBifunctor should also implement
Bifunctor (the ref variant is strictly a supplement, not a replacement).
Whether RefBifunctor has `Bifunctor` as a supertrait is a design choice.
If it does not, the bound `Bifunctor + RefBifunctor` would be needed
explicitly on the generic impl.

**Recommendation**: Make RefBifunctor a standalone trait (not requiring
Bifunctor as a supertrait), and use `Brand: Bifunctor + RefBifunctor` on
the generic RefFunctor impl. This avoids forcing all RefBifunctor
implementors to also implement Bifunctor at the trait level, while
ensuring the generic applied brand machinery works correctly.

---

## 7. Cross-Cutting Issue: TryThunkBrand Exclusion

The investigation found that TryThunkBrand cannot implement any of the
three Ref bi-traits (RefBifunctor, RefBifoldable, RefBitraversable).
The plan currently lists TryThunk as an implementor of RefBifunctor and
RefBifoldable (5 types each) and excludes it only from RefBitraversable.

**Root cause**: TryThunk wraps `Thunk<'a, Result<A, E>>`, which wraps
`Box<dyn FnOnce() -> Result<A, E> + 'a>`. FnOnce closures are consumed
on evaluation and cannot be cloned. All three operations (bimap,
bi_fold_right, bi_traverse) require either consuming the thunk or
accessing its computed result, neither of which is possible from a shared
reference.

**Impact**: The plan's RefBifunctor and RefBifoldable sections must be
updated to list 4 types, not 5. The implementation order remains the
same. No downstream issues, because TryThunk's specific applied brands
(`TryThunkErrAppliedBrand`, `TryThunkOkAppliedBrand`) also lack
RefFunctor, RefFoldable, and RefTraversable, so the exclusion is
consistent with the existing pattern.

---

## Summary

| Question                                | Finding                                                                           | Action Needed                            |
| --------------------------------------- | --------------------------------------------------------------------------------- | ---------------------------------------- |
| RefBifunctor for 5 types                | TryThunk infeasible; 4 of 5 types work.                                           | Update plan: 4 types, not 5.             |
| RefBifoldable for 5 types               | TryThunk infeasible; 4 of 5 types work.                                           | Update plan: 4 types, not 5.             |
| RefBitraversable for 4 types            | All 4 feasible. No applicative machinery issues.                                  | None.                                    |
| RefCompactable for 3 types              | All 3 feasible. A: Clone required and sufficient.                                 | None.                                    |
| RefAlt for 3 types                      | All 3 feasible. CatList's A: Clone bound is unused but harmless.                  | None (minor ergonomic note for CatList). |
| Generic RefFunctor for applied brands   | No E0119 conflict. Requires Brand: Bifunctor + RefBifunctor, A: Clone.            | None (design is sound).                  |
| Endofunction defaults for RefBifoldable | Work correctly with existing Clone bounds.                                        | None.                                    |
| TryThunk exclusion (cross-cutting)      | Must exclude from RefBifunctor and RefBifoldable in addition to RefBitraversable. | Update plan tables.                      |
