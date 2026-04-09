# Ref Trait Implementations: Borrow Feasibility Analysis

This document analyzes every concrete Ref trait implementation to determine
whether the container parameter `fa` truly needs to be consumed (owned), or
whether `&fa` would suffice.

## Summary Table

| Type                    | Consumes container?          | Could take `&fa`?   | Notes                                                                                                              |
| ----------------------- | ---------------------------- | ------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Vec                     | No (borrows via `.iter()`)   | Yes, trivially      | All impls call `.iter()`, `.par_iter()`, or `.as_ref()` on the owned Vec.                                          |
| Option                  | No (borrows via `.as_ref()`) | Yes, trivially      | Most impls use `.as_ref()` or `match fa { ... &a ... }`.                                                           |
| CatList                 | No (borrows via `.iter()`)   | Yes, trivially      | `CatList::iter()` takes `&self`. All impls iterate.                                                                |
| Identity                | No (borrows field)           | Yes, trivially      | All impls access `&fa.0`.                                                                                          |
| Tuple1                  | No (borrows field)           | Yes, trivially      | All impls access `&fa.0`.                                                                                          |
| Tuple2FirstApplied      | Partially                    | Nearly all yes      | `ref_map`, `ref_fold_map` borrow. `ref_lift2`, `ref_apply`, `ref_bind` move the non-functored field.               |
| Tuple2SecondApplied     | Partially                    | Nearly all yes      | Mirror of FirstApplied.                                                                                            |
| PairFirstApplied        | Partially                    | Nearly all yes      | Same pattern as Tuple2. Fixed field is moved out.                                                                  |
| PairSecondApplied       | Partially                    | Nearly all yes      | Mirror of PairFirstApplied.                                                                                        |
| Result (ErrApplied)     | Partially                    | Nearly all yes      | `ref_map`, `ref_fold_map`, `ref_bind` borrow. `ref_lift2`, `ref_apply` destructure to move E.                      |
| Result (OkApplied)      | Partially                    | Nearly all yes      | Mirror of ErrApplied.                                                                                              |
| Lazy (RcLazyConfig)     | No (Rc clone semantics)      | Yes, via `.clone()` | Lazy is `Rc<LazyCell<...>>`, so moving is semantically an Rc clone. Borrowing + explicit `.clone()` is equivalent. |
| Lazy (ArcLazyConfig)    | No (Arc clone semantics)     | Yes, via `.clone()` | Same as RcLazy but with Arc.                                                                                       |
| TryLazy (RcLazyConfig)  | No (Rc clone semantics)      | Yes, via `.clone()` | Same as Lazy.                                                                                                      |
| TryLazy (ArcLazyConfig) | No (Arc clone semantics)     | Yes, via `.clone()` | Same as Lazy.                                                                                                      |

---

## Per-Type Details

### Vec (VecBrand)

**Traits implemented:** RefFunctor, RefFoldable, RefFilterable, RefTraversable,
RefWitherable, RefFunctorWithIndex, RefFoldableWithIndex,
RefFilterableWithIndex, RefTraversableWithIndex, RefPointed, RefLift,
RefSemiapplicative, RefSemimonad, ParRefFunctor, ParRefFoldable,
ParRefFilterable, ParRefFunctorWithIndex, ParRefFoldableWithIndex,
ParRefFilterableWithIndex.

**Pattern:** Every implementation calls `.iter()`, `.par_iter()`, or
`.into_iter()` on the consumed Vec. However, `.iter()` and `.par_iter()`
only borrow, so ownership is not required for any of them.

- `ref_map`: `fa.iter().map(func).collect()` -> borrows only.
- `ref_fold_map`: `fa.iter().fold(...)` -> borrows only.
- `ref_filter_map`: `fa.iter().filter_map(func).collect()` -> borrows only.
- `ref_traverse`: delegates to `Self::traverse` which does consume, but
  this is a delegation pattern, not fundamental. The ref version wraps the
  owned traverse with `move |a: A| func(&a)`.
- `ref_lift2`: `fa.iter().flat_map(|a| fb.iter().map(...))` -> borrows both.
- `ref_apply`: `ff.iter().flat_map(|f| fa.iter().map(...))` -> borrows both.
- `ref_bind`: `fa.iter().flat_map(f).collect()` -> borrows only.
- `ref_pure`: takes `&A`, clones. No container involved.
- All `par_ref_*` variants: use `.par_iter()` or `.iter()`, borrows only.

**Could take `&Vec<A>`?** Yes, trivially. No implementation actually needs
ownership. The Vec is only borrowed through iterator methods.

**Changes needed:** Change `fa: Vec<A>` to `fa: &Vec<A>` in all signatures.
The bodies need no modification since `.iter()` already borrows.

---

### Option (OptionBrand)

**Traits implemented:** RefFunctor, RefFoldable, RefFilterable,
RefTraversable, RefWitherable, RefFunctorWithIndex, RefFoldableWithIndex,
RefTraversableWithIndex, RefPointed, RefLift, RefSemiapplicative,
RefSemimonad.

**Pattern:** Most implementations use `.as_ref()` or pattern-match the owned
option and take `&a`.

- `ref_map`: `fa.as_ref().map(func)` -> borrows.
- `ref_fold_map`: `match fa { Some(a) => func(&a), ... }` -> consumes Option
  to destructure, but only borrows the inner value. With `&Option<A>`, would
  use `match fa { Some(a) => func(a), ... }` (a is already `&A`).
- `ref_filter_map`: `fa.as_ref().and_then(func)` -> borrows.
- `ref_traverse`: delegates to owned traverse via `move |a: A| func(&a)`.
- `ref_map_with_index`: `fa.as_ref().map(|a| func((), a))` -> borrows.
- `ref_fold_map_with_index`: `match fa { Some(a) => func((), &a), ... }` ->
  same as fold_map.
- `ref_lift2`: `match (fa.as_ref(), fb.as_ref()) { ... }` -> borrows both.
- `ref_apply`: `match (ff, fa.as_ref()) { ... }` -> borrows fa, consumes ff
  (to deref the function). With `&Option<Fn>`, ff would need `.as_ref()` too.
- `ref_bind`: `fa.as_ref().and_then(f)` -> borrows.
- `ref_pure`: takes `&A`, clones. No container.

**Could take `&Option<A>`?** Yes. For `ref_fold_map` and similar `match`
patterns, the destructured binding would change from `Some(a)` (where `a: A`)
to `Some(a)` (where `a: &A`), eliminating the need for the `&a` in the func
call. For `ref_apply`, `ff` would need `.as_ref()` to get `&Fn`.

**Changes needed:** Minimal. Replace `.as_ref()` calls (now redundant).
Pattern matches naturally produce references. `ref_apply` needs
`ff.as_ref()` for the function container.

---

### CatList (CatListBrand)

**Traits implemented:** RefFunctor, RefFoldable, RefFilterable,
RefTraversable, RefWitherable, RefFunctorWithIndex, RefFoldableWithIndex,
RefFilterableWithIndex, RefTraversableWithIndex, RefPointed, RefLift,
RefSemiapplicative, RefSemimonad, ParRefFunctor, ParRefFoldable,
ParRefFilterable, ParRefFunctorWithIndex, ParRefFoldableWithIndex,
ParRefFilterableWithIndex.

**Pattern:** Identical to Vec. All impls call `.iter()` which takes `&self`
on CatList.

- `ref_map`: `fa.iter().map(func).collect()` -> borrows.
- `ref_fold_map`: `fa.iter().fold(...)` -> borrows.
- `ref_filter_map`: `fa.iter().filter_map(func).collect()` -> borrows.
- `ref_traverse`: delegates to owned traverse.
- `ref_lift2`: `fa.iter().flat_map(|a| fb.iter().map(...))` -> borrows both.
- `ref_apply`: `ff.iter().flat_map(|f| fa.iter().map(...))` -> borrows both.
- `ref_bind`: `fa.iter().flat_map(f).collect()` -> borrows.
- All `par_ref_*` variants: collect to Vec first then parallel iterate.

**Could take `&CatList<A>`?** Yes, trivially. Same rationale as Vec.

**Changes needed:** Signature change only. Bodies unchanged.

---

### Identity (IdentityBrand)

**Traits implemented:** RefFunctor, RefFoldable, RefTraversable,
RefFunctorWithIndex, RefFoldableWithIndex, RefTraversableWithIndex,
RefPointed, RefLift, RefSemiapplicative, RefSemimonad.

**Pattern:** All impls access `&fa.0` to borrow the inner value.

- `ref_map`: `Identity(func(&fa.0))` -> borrows.
- `ref_fold_map`: `func(&fa.0)` -> borrows.
- `ref_traverse`: delegates to owned traverse.
- `ref_lift2`: `Identity(func(&fa.0, &fb.0))` -> borrows both.
- `ref_apply`: `Identity((*ff.0)(&fa.0))` -> borrows fa, derefs ff's function.
- `ref_bind`: `f(&fa.0)` -> borrows.

**Could take `&Identity<A>`?** Yes. With `&Identity<A>`, `fa.0` is already
`&A` so the explicit `&fa.0` would become just `&fa.0` (still works, the
auto-deref handles it). More precisely, `(&fa).0` gives `&A`.

**Changes needed:** Signature change only. For `ref_lift2`, change `&fa.0` to
`&fa.0` (same). For `ref_apply`, `ff` would be `&Identity<Fn>`, access
`&ff.0` to get `&Fn`, then deref.

---

### Tuple1 (Tuple1Brand)

**Traits implemented:** RefFunctor, RefFoldable, RefTraversable, RefPointed,
RefLift, RefSemiapplicative, RefSemimonad.

**Pattern:** Identical to Identity but with tuple syntax `fa.0`.

- `ref_map`: `(func(&fa.0),)` -> borrows.
- `ref_fold_map`: `func(&fa.0)` -> borrows.
- `ref_lift2`: `(func(&fa.0, &fb.0),)` -> borrows both.
- `ref_apply`: `((*ff.0)(&fa.0),)` -> borrows fa.
- `ref_bind`: `f(&fa.0)` -> borrows.

**Could take `&(A,)`?** Yes, trivially.

**Changes needed:** Signature change only.

---

### Tuple2FirstApplied (Tuple2FirstAppliedBrand)

**Traits implemented:** RefFunctor, RefFoldable, RefTraversable, RefPointed,
RefLift, RefSemiapplicative, RefSemimonad.

**Pattern:** The functored position is `.1` (second). The fixed position `.0`
(first) is sometimes moved out.

- `ref_map`: `(fa.0, func(&fa.1))` -> moves `fa.0`, borrows `fa.1`. With
  `&fa`, would need `fa.0.clone()`.
- `ref_fold_map`: `func(&fa.1)` -> borrows only. Works with `&fa`.
- `ref_traverse`: delegates to owned traverse.
- `ref_lift2`: `(Semigroup::append(fa.0, fb.0), func(&fa.1, &fb.1))` -> moves
  both `.0` fields (to append). Would need `.clone()` on both `.0` with `&fa`.
- `ref_apply`: `(Semigroup::append(ff.0, fa.0), (*ff.1)(&fa.1))` -> moves both
  `.0` fields. Same issue.
- `ref_bind`: `let (first, second) = fa; let (nf, ns) = f(&second);
(Semigroup::append(first, nf), ns)` -> destructures, moves `first`. Would
  need clone.

**Could take `&(First, A)`?** Mostly yes, but several impls would need to
clone the fixed `First` field. Since `First: Clone` is already required for
most of these traits (RefLift, RefSemiapplicative, RefSemimonad all require
`First: Clone + 'static`), the clone is always available.

**Changes needed:** Add `.clone()` calls on the fixed field in `ref_map`,
`ref_lift2`, `ref_apply`, `ref_bind`.

---

### Tuple2SecondApplied (Tuple2SecondAppliedBrand)

Mirror of Tuple2FirstApplied. The functored position is `.0`, and the fixed
position `.1` (Second) is moved in the same set of methods.

**Could take `&(A, Second)`?** Yes, with `.clone()` on the `Second` field.
`Second: Clone` is already required on all relevant trait impls.

**Changes needed:** Same as Tuple2FirstApplied but for the `.1` field.

---

### PairFirstApplied (PairFirstAppliedBrand)

**Traits implemented:** RefFunctor, RefFoldable, RefTraversable, RefPointed,
RefLift, RefSemiapplicative, RefSemimonad.

**Pattern:** Identical to Tuple2FirstApplied but with `Pair(first, second)`
instead of `(first, second)`.

- `ref_map`: `Pair(fa.0, func(&fa.1))` -> moves `fa.0`.
- `ref_fold_map`: `func(&fa.1)` -> borrows only.
- `ref_lift2`: `Pair(Semigroup::append(fa.0, fb.0), func(&fa.1, &fb.1))` ->
  moves both `.0`.
- `ref_apply`: moves both `.0` fields.
- `ref_bind`: destructures, moves `first`.

**Could take `&Pair<First, A>`?** Yes, with `.clone()` on `First`. Already
bounded by `First: Clone`.

**Changes needed:** Same as Tuple2FirstApplied.

---

### PairSecondApplied (PairSecondAppliedBrand)

Mirror of PairFirstApplied. Functored position is `.0`, fixed is `.1`.

**Could take `&Pair<A, Second>`?** Yes, with `.clone()` on `Second`.

**Changes needed:** Same as Tuple2SecondApplied.

---

### Result ErrApplied (ResultErrAppliedBrand<E>)

**Traits implemented:** RefFunctor, RefFoldable, RefTraversable, RefPointed,
RefLift, RefSemiapplicative, RefSemimonad.

**Pattern:** Pattern matches on `Ok(a)` / `Err(e)`. The functored position is
the `Ok` side. `Err(e)` is moved out in some methods.

- `ref_map`: `match fa { Ok(a) => Ok(func(&a)), Err(e) => Err(e) }` -> moves
  `e` out of Err variant. With `&fa`, would need `e.clone()`.
- `ref_fold_map`: `match fa { Ok(a) => func(&a), Err(_) => empty() }` ->
  borrows `a`, ignores `e`. Works with `&fa`.
- `ref_traverse`: delegates to owned traverse.
- `ref_lift2`: `match (fa, fb) { (Ok(a), Ok(b)) => Ok(func(&a, &b)),
(Err(e), _) => Err(e), ... }` -> moves `e`. Would need clone with `&fa`.
- `ref_apply`: `match (ff, fa) { (Ok(f), Ok(a)) => Ok((*f)(&a)),
(Err(e), _) => Err(e), ... }` -> moves `e` and `f`. Would need clones.
- `ref_bind`: `match fa { Ok(a) => f(&a), Err(e) => Err(e) }` -> moves `e`.
  Would need clone.

**Could take `&Result<A, E>`?** Yes, but `Err(e)` propagation would need
`e.clone()`. The bounds `E: Clone` are already present on RefLift,
RefSemiapplicative, RefSemimonad. For RefFunctor and RefFoldable, E: Clone
is not currently required, so either: (a) add `E: Clone` bound to those
traits, or (b) in `ref_map`, borrow `e` and clone it (which needs E: Clone).
Since `ref_map` must return an owned `Result<B, E>`, cloning is unavoidable
for the error path.

**Changes needed:** Add `E: Clone` bound to RefFunctor and RefFoldable (or
just `ref_map`). Add `.clone()` on `Err(e)` in all methods that propagate
errors.

---

### Result OkApplied (ResultOkAppliedBrand<T>)

Mirror of ErrApplied. Functored position is the `Err` side. The `Ok(t)` value
is moved when propagated.

**Could take `&Result<T, A>`?** Yes, with same pattern. `T: Clone` is needed
for propagating the `Ok(t)` variant. Already present on RefLift,
RefSemiapplicative, RefSemimonad. Would need to be added for RefFunctor.

**Changes needed:** Same as ErrApplied but for `T` instead of `E`.

---

### Lazy RcLazy (LazyBrand<RcLazyConfig>)

**Traits implemented:** RefFunctor, RefPointed, RefLift, RefSemiapplicative,
RefSemimonad, RefFoldable (generic over Config), RefFoldableWithIndex
(generic), RefFunctorWithIndex.

**Internal structure:** `Lazy<'a, A, RcLazyConfig>` is a newtype around
`Rc<LazyCell<A, Box<dyn FnOnce() -> A>>>`. Moving a Lazy is semantically
identical to cloning it (Rc refcount bump). The `ref_map` inherent method
takes `self` (which moves the Rc) and captures it in a closure.

**Pattern:**

- `ref_map` (trait): delegates to `fa.ref_map(f)`, which is
  `RcLazy::new(move || f(self.evaluate()))`. Captures `self` (the Rc) by move.
- `ref_fold_map`: `func(fa.evaluate())` -> calls `evaluate()` on owned Lazy.
  `evaluate()` takes `&self` on Lazy, so this works with `&fa` too.
- `ref_fold_right`/`ref_fold_left`: same pattern, `fa.evaluate()` borrows.
- `ref_lift2`: `RcLazy::new(move || func(fa.evaluate(), fb.evaluate()))` ->
  captures both by move into closure. With `&fa`, would need `fa.clone()` (Rc
  bump) before capturing.
- `ref_apply`: `RcLazy::new(move || { let f = ff.evaluate(); let a =
fa.evaluate(); (**f)(a) })` -> captures both by move. Same as above.
- `ref_bind`: `f(fa.evaluate())` -> calls evaluate (borrows) then passes
  result to f. Works directly with `&fa` since evaluate takes `&self`.
- `ref_pure`: takes `&A`, clones. No container.
- `ref_map_with_index`: delegates to `Self::ref_map`.
- `ref_fold_map_with_index`: `f((), fa.evaluate())` -> borrows.

**Could take `&Lazy<A>`?** Yes. For methods that capture the Lazy in a
closure (`ref_map`, `ref_lift2`, `ref_apply`), an explicit `.clone()` (Rc
refcount bump) is needed before moving into the closure. For methods that
just call `.evaluate()` directly (`ref_fold_map`, `ref_bind`,
`ref_fold_right`, `ref_fold_left`), no changes needed since `evaluate()`
takes `&self`.

**Lazy-specific question: would `lazy.clone()` suffice?** Yes. `Lazy` is
`Rc<LazyCell<...>>`, so `clone()` is just an Rc refcount increment. This is
the exact same operation that moving an Rc performs. The cloned Lazy shares
the same underlying LazyCell, so memoization is preserved.

**Changes needed:** In `ref_map`, `ref_lift2`, `ref_apply`: clone the Lazy
before capturing in the closure. Cost: one Rc refcount bump per capture
(negligible).

---

### Lazy ArcLazy (LazyBrand<ArcLazyConfig>)

**Traits implemented:** SendRefFunctor, SendRefPointed, SendRefLift,
SendRefSemiapplicative, SendRefSemimonad, SendRefFoldable,
SendRefFoldableWithIndex, SendRefFunctorWithIndex. Also RefFoldable and
RefFoldableWithIndex (via generic Config impl).

**Internal structure:** `Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send>>>`.
Same semantics as RcLazy but thread-safe.

**Pattern:** Identical to RcLazy. All the same analysis applies.

- `send_ref_map`: `fa.ref_map(f)` -> captures by move.
- `send_ref_lift2`: `ArcLazy::new(move || func(fa.evaluate(), fb.evaluate()))`
  -> captures by move.
- `send_ref_apply`: captures by move.
- `send_ref_bind`: `f(ma.evaluate())` -> borrows via evaluate.
- `send_ref_fold_map`: `func(fa.evaluate())` -> borrows.

**Could take `&ArcLazy<A>`?** Yes, via `.clone()` (Arc bump) where needed.
Same reasoning as RcLazy.

**Changes needed:** Same as RcLazy: clone before capturing in closures.

---

### TryLazy RcTryLazy (TryLazyBrand<E, RcLazyConfig>)

**Traits implemented:** RefFoldable, RefFoldableWithIndex, RefFunctor.

**Internal structure:** Wraps a `Lazy<Result<A, E>>` internally. Same Rc-based
memoization.

**Pattern:**

- `ref_fold_map`: `match fa.evaluate() { Ok(a) => func(a), Err(_) => empty() }`
  -> `evaluate()` takes `&self`. Borrows only.
- `ref_fold_right`/`ref_fold_left`: same pattern.
- `ref_fold_map_with_index`: same pattern.
- `ref_map` (trait): delegates to `fa.ref_map(f)` inherent method, which is
  `RcTryLazy::new(move || match self.evaluate() { Ok(a) => Ok(f(a)),
Err(e) => Err(e.clone()) })`. Captures `self` by move.

**Could take `&RcTryLazy<A, E>`?** Yes. Same as Lazy: fold methods already
work with `&self` via `evaluate()`. The `ref_map` impl would need
`.clone()` before capturing.

**Changes needed:** Same as Lazy.

---

### TryLazy ArcTryLazy (TryLazyBrand<E, ArcLazyConfig>)

**Traits implemented:** SendRefFunctor.

**Pattern:** `send_ref_map` delegates to `fa.ref_map(f)`, same as RcTryLazy.

**Could take `&ArcTryLazy<A, E>`?** Yes, via `.clone()`.

**Changes needed:** Same as Lazy/TryLazy.

---

## Cross-cutting Observations

### 1. No implementation truly requires ownership for mutation

None of the Ref trait implementations perform in-place mutation. Every one
produces a new container. The owned `fa` parameter is immediately borrowed
(via `.iter()`, `.as_ref()`, `.evaluate()`, or field access `&fa.0`) and
never consumed destructively.

### 2. The Pair/Tuple2/Result "fixed field" pattern

For bifunctorial types (Pair, Tuple2, Result), the non-functored field is
moved out of the owned container. With `&fa`, this requires cloning. However,
the relevant trait impls already require `Clone` on these fixed types
(via Semigroup bounds on Pair/Tuple2 first fields, and Clone on Result error
types). The only exceptions are RefFunctor and RefFoldable for Result, which
currently don't require `E: Clone` / `T: Clone`.

### 3. Lazy types: move = clone

For Lazy/TryLazy, "consuming" the container is just an Rc/Arc move, which is
semantically identical to a clone (refcount bump). Switching to `&fa` and
adding `.clone()` has zero semantic difference and negligible performance
cost (one atomic increment for Arc, one non-atomic increment for Rc).

### 4. Delegation to owned Traversable

Several RefTraversable implementations delegate to the owned
`Self::traverse()` with a wrapper `move |a: A| func(&a)`. This pattern
consumes the container via the owned traverse. If the trait signature changes
to take `&fa`, this delegation would need to either: (a) clone the container
before delegating, or (b) be rewritten to iterate by reference directly.
Option (b) is preferable for Vec, CatList, and Option. For Pair/Tuple2, the
owned delegation is fine after cloning the fixed field.

### 5. Types NOT found in the codebase

- **ConstVal**: No type or brand named ConstVal exists in the types directory.
  The user may have been referring to a planned type.
