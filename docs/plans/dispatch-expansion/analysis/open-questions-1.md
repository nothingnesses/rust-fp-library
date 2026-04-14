# Dispatch Expansion: Open Questions Investigation 1

Focus: dispatch trait pattern fidelity and correctness.

---

## Issue 1: Dispatch method naming convention is inconsistent

### Question

Existing dispatch traits use different method names: `dispatch` (FoldRight,
FoldLeft, FoldMap, Functor, ComposeKleisli), `dispatch_bind` (Bind),
`dispatch_filter_map` (FilterMap), `dispatch_traverse` (Traverse),
`dispatch_lift2` through `dispatch_lift5` (Lift). What convention should the
new traits follow?

### Research findings

The naming falls into two groups:

- **Bare `dispatch`:** FoldRightDispatch, FoldLeftDispatch, FoldMapDispatch,
  FunctorDispatch, ComposeKleisliDispatch. These use just `fn dispatch(...)`.
- **Qualified `dispatch_*`:** BindDispatch uses `dispatch_bind`, FilterMapDispatch
  uses `dispatch_filter_map`, TraverseDispatch uses `dispatch_traverse`,
  Lift2-5Dispatch use `dispatch_lift2` through `dispatch_lift5`.

The qualified names were likely added to avoid ambiguity when multiple dispatch
traits are in scope and a type could implement several. With bare `dispatch`,
the compiler cannot disambiguate without fully qualified syntax. With
`dispatch_foo`, each trait has a unique method name and the compiler resolves
calls unambiguously.

However, the bare `dispatch` group works because the dispatch trait method is
only ever called inside the unified free function, never directly by users.
The free function already knows which trait it is calling, so ambiguity in user
code does not arise.

### Approaches

**A. Use bare `dispatch` for all new traits.** Simpler, but inconsistent with
the existing qualified names.

**B. Use qualified `dispatch_*` for all new traits.** E.g.,
`dispatch_filter`, `dispatch_partition`, `dispatch_map_with_index`. Consistent
with the majority of non-foldable dispatch traits.

**C. Do not standardize; let each trait choose.** Continues the existing
inconsistency.

### Recommendation

Use qualified `dispatch_*` names (approach B). The qualified names are
self-documenting and prevent accidental method name collisions if a type
somehow implements multiple dispatch traits. The existing foldable dispatch
traits (FoldRight, FoldLeft, FoldMap) using bare `dispatch` are a minor
inconsistency that can be left as-is or renamed in a future cleanup pass.

---

## Issue 2: Bimap dispatch differs structurally from compose_kleisli

### Question

The plan says bimap dispatch follows the compose_kleisli precedent of using a
closure tuple `(F, G)`. Is this actually the same pattern? Are there structural
differences that affect correctness?

### Research findings

`ComposeKleisliDispatch` and the proposed `BimapDispatch` use tuples
differently:

**ComposeKleisliDispatch:**

- Trait parameters: `<'a, Brand, A, B, C, Marker>`. No `FA` parameter.
- The input `a: A` is always a plain value (not a container), so there is no
  owned-vs-borrowed container axis.
- Dispatch is driven solely by the closure types:
  Val `F: Fn(A) -> ...`, Ref `F: Fn(&A) -> ...`.
- The `Marker` is the only axis of dispatch.

**Proposed BimapDispatch:**

- Would need an `FA` parameter because the bifunctor container can be owned or
  borrowed.
- Trait parameters: `<'a, Brand, A, B, C, D, FA, Marker>` (8 params).
- Dispatch is driven by two axes: the closure tuple types (Val vs Ref) AND the
  container type (owned `Brand::Of<A, C>` vs borrowed `&Brand::Of<A, C>`).
- The Kind hash differs: bifunctors use `Kind_266801a817966495` (two type
  params in `Of<'a, A, B>`) rather than the standard `Kind_cdc7cd43dac7585f`
  (one type param). The dispatch trait must use the bifunctor kind hash.

**Key difference:** ComposeKleisliDispatch has no container parameter (it
receives a plain value `a: A`), so it only dispatches on the Marker. BimapDispatch
needs both the container parameter `FA` and the Marker, making it structurally
closer to FunctorDispatch than to ComposeKleisliDispatch, but operating on a
tuple `(F, G)` instead of a single closure.

### Potential flaw: mixed closures

The plan states mixed combinations (one owned, one borrowed) "do not match
either impl and fail to compile." This is correct. If a user writes
`bimap((|x: i32| x + 1, |y: &str| y.len()), container)`, neither the Val nor
Ref impl matches, producing a compile error. This is the desired behavior;
the error message should be reasonably clear since neither blanket impl applies.

### E0119 safety for bimap

The two impls would be:

- Val: `(F, G)` where `F: Fn(A) -> B, G: Fn(C) -> D`, `FA = Brand::Of<'a, A, C>`.
- Ref: `(F, G)` where `F: Fn(&A) -> B, G: Fn(&C) -> D`, `FA = &Brand::Of<'a, A, C>`.

These are distinguishable on two axes (closure args and container reference).
No E0119 risk.

### Recommendation

The bimap dispatch pattern is sound, but the plan should clarify that it is
structurally more like FunctorDispatch (with an `FA` container parameter) than
ComposeKleisliDispatch (which has no container). The tuple `(F, G)` as the
`Self` type is the shared element, not the overall trait shape. Documentation
should note the bifunctor kind hash difference.

---

## Issue 3: FoldMapWithIndexDispatch combining WithIndex and FnBrand

### Question

The plan adds dispatch traits that combine the WithIndex pattern (Brand::Index
projection) with the FnBrand pattern (from foldable dispatch). Does this
combination introduce any new complications?

### Research findings

The existing patterns compose cleanly:

**FoldMapDispatch** (existing, in `dispatch/foldable.rs`):

- Trait: `FoldMapDispatch<'a, FnBrand, Brand, A, M, FA, Marker>`.
- Val: `F: Fn(A) -> M`, `FA = Brand::Of<'a, A>`, Brand: Foldable.
- Ref: `F: Fn(&A) -> M`, `FA = &Brand::Of<'a, A>`, Brand: RefFoldable.
- FnBrand: `LiftFn + 'a` on both impls.

**FoldableWithIndex::fold_map_with_index** (existing free function):

- Signature: `fn fold_map_with_index<FnBrand, Brand: FoldableWithIndex, A, R: Monoid>(f: impl Fn(Brand::Index, A) -> R, fa: Brand::Of<'a, A>) -> R`.
- Adds `Brand: WithIndex` bound (via `FoldableWithIndex: Foldable + WithIndex`).
- Uses `Brand::Index` in the closure type.

**RefFoldableWithIndex::ref_fold_map_with_index** (existing free function):

- Signature: `fn ref_fold_map_with_index<FnBrand, Brand: RefFoldableWithIndex, A, R: Monoid>(f: impl Fn(Brand::Index, &A) -> R, fa: &Brand::Of<'a, A>) -> R`.
- Uses same `Brand::Index` in closure.

**Proposed FoldMapWithIndexDispatch:**

- Trait: `FoldMapWithIndexDispatch<'a, FnBrand, Brand, A, M, FA, Marker>`.
- Val: `F: Fn(Brand::Index, A) -> M`, Brand: FoldableWithIndex.
- Ref: `F: Fn(Brand::Index, &A) -> M`, Brand: RefFoldableWithIndex.

This is a straightforward merge. The `Brand::Index` appears in the same
position in both closures and does not affect the `A` vs `&A` dispatch axis.
The `FnBrand` parameter is already established in FoldMapDispatch and carries
over identically. The `Monoid` bound on `M` applies in both impls equally.

**Verified for all four WithIndex+FnBrand traits:**

| Dispatch trait             | Val closure              | Ref closure               | Distinguishable? |
| -------------------------- | ------------------------ | ------------------------- | ---------------- |
| FoldMapWithIndexDispatch   | `Fn(Idx, A) -> M`        | `Fn(Idx, &A) -> M`        | Yes.             |
| FoldRightWithIndexDispatch | `Fn(Idx, A, B) -> B`     | `Fn(Idx, &A, B) -> B`     | Yes.             |
| FoldLeftWithIndexDispatch  | `Fn(Idx, B, A) -> B`     | `Fn(Idx, B, &A) -> B`     | Yes.             |
| TraverseWithIndexDispatch  | `Fn(Idx, A) -> F::Of<B>` | `Fn(Idx, &A) -> F::Of<B>` | Yes.             |

Note: FoldLeftWithIndexDispatch closure types listed here assume the argument
order fix (issue 4) has been applied.

### Potential complication: where clause complexity

The combined dispatch trait will require `Brand::Index: 'a` in the where
clause, just as the non-dispatch free functions do. This adds one extra where
clause to each impl but is not a correctness concern.

### Recommendation

The combination works without issues. Each new dispatch trait simply adds
`Brand::Index` to the closure bound while keeping the FnBrand handling
identical to the existing foldable dispatch. No novel patterns needed.

---

## Issue 4: ref_fold_left_with_index argument order fix blast radius

### Question

The plan requires fixing `ref_fold_left_with_index` from `Fn(B, Self::Index, &A) -> B`
to `Fn(Self::Index, B, &A) -> B` before adding dispatch. What is the actual
blast radius?

### Research findings

**Trait definition sites (must change the signature):**

1. `RefFoldableWithIndex::ref_fold_left_with_index` in
   `fp-library/src/classes/ref_foldable_with_index.rs` (line 183).
   Signature: `func: impl Fn(B, Self::Index, &A) -> B`.
   Also has a default implementation body that calls `func(b, i, &a)`.
2. `SendRefFoldableWithIndex::send_ref_fold_left_with_index` in
   `fp-library/src/classes/send_ref_foldable_with_index.rs` (line 198).
   Signature: `func: impl Fn(B, Self::Index, &A) -> B + Send + Sync`.
   Also has a default implementation body that calls `func(b, i, &a)`.

**Concrete implementations that override the default:**
None found. No type in `fp-library/src/types/` overrides `ref_fold_left_with_index`
or `send_ref_fold_left_with_index`. All types use the default implementation
derived from `ref_fold_map_with_index`.

**Free functions:**
There is no standalone `ref_fold_left_with_index` free function (unlike
`ref_fold_map_with_index` and `ref_fold_right_with_index`, which have free
functions). This means no free function signature needs to change.

**Call sites in user code / tests / doc tests:**

- The doc test on `ref_fold_left_with_index` (line 176-181 in
  `ref_foldable_with_index.rs`): `|acc: i32, _, x: &i32| acc + *x`.
- The doc test on `send_ref_fold_left_with_index` (line 191-196 in
  `send_ref_foldable_with_index.rs`): `|acc: i32, _, x: &i32| acc + *x`.
- No other call sites found in the entire repository.

**Default implementation internals:**

- In `RefFoldableWithIndex`, the default body constructs a `LiftFn` closure
  with `(b, i, a): (B, Self::Index, A)` and calls `func(b, i, &a)`. This
  must change to `(i, b, a): (Self::Index, B, A)` and `func(i, b, &a)`.
- Same pattern in `SendRefFoldableWithIndex`.

**Also needs fixing:** The val variant `fold_left_with_index` uses
`Fn(Self::Index, B, A) -> B` (index first). The corresponding non-indexed
`fold_left` and `ref_fold_left` use `Fn(B, A) -> B` and `Fn(B, &A) -> B`
(accumulator first). The Val `fold_left_with_index` deviates from both the
non-indexed convention and PureScript. However, this deviation is
pre-existing and the plan only proposes fixing the Ref variant to match the
Val variant. Aligning both with the non-indexed convention (`Fn(B, Idx, A)`)
would be a separate decision with larger scope.

### Blast radius summary

| Category            | Count | Notes                                              |
| ------------------- | ----- | -------------------------------------------------- |
| Trait definitions   | 2     | RefFoldableWithIndex, SendRefFoldableWithIndex.    |
| Default impl bodies | 2     | LiftFn tuple reordering in each.                   |
| Concrete overrides  | 0     | No types override the default.                     |
| Free functions      | 0     | No standalone ref_fold_left_with_index function.   |
| Doc tests           | 2     | One per trait definition.                          |
| External call sites | 0     | No usage found outside trait definitions and docs. |

Total changes: 4 code locations + 2 doc tests = 6 edits. The blast radius is
very small.

### Approaches

**A. Fix Ref to match Val (index first).** Change `Fn(B, Idx, &A) -> B` to
`Fn(Idx, B, &A) -> B`. This is what the plan proposes. Makes the only
difference between Val and Ref be `A` vs `&A`, enabling dispatch.

**B. Fix both Val and Ref to match non-indexed convention (accumulator first).**
Change Val to `Fn(B, Idx, A) -> B` and Ref to `Fn(B, Idx, &A) -> B`.
Consistent with `fold_left` but deviates from PureScript's `(i -> b -> a -> b)`.
Would also require changing all Val `fold_left_with_index` call sites and doc
tests.

### Recommendation

Approach A. It has minimal blast radius (6 edits), matches PureScript, and
satisfies the dispatch requirement. Approach B is a larger refactor that can
be considered separately if desired.

---

## Issue 5: Two-impl pattern verification for all proposed traits

### Question

Does the two-impl pattern work correctly for every proposed dispatch trait?
Are the Val and Ref impls distinguishable in all cases?

### Research findings

I verified each proposed dispatch trait against the established pattern
from FilterMapDispatch (the simplest existing example).

**FilterMapDispatch pattern (reference):**

- Val impl: `F` implements `FilterMapDispatch<..., Apply!(Brand::Of<'a, A>), Val>`
  where `F: Fn(A) -> Option<B>`.
- Ref impl: `F` implements `FilterMapDispatch<..., &'b Apply!(Brand::Of<'a, A>), Ref>`
  where `F: Fn(&A) -> Option<B>`.
- Distinction: `Fn(A)` vs `Fn(&A)` AND `FA` vs `&FA`.

**Simple group (FilterDispatch, PartitionDispatch, PartitionMapDispatch):**

| Trait                | Val closure type        | Ref closure type         | Val FA             | Ref FA              | Val return                             | Ref return                             | OK? |
| -------------------- | ----------------------- | ------------------------ | ------------------ | ------------------- | -------------------------------------- | -------------------------------------- | --- |
| FilterDispatch       | `Fn(A) -> bool`         | `Fn(&A) -> bool`         | `Brand::Of<'a, A>` | `&Brand::Of<'a, A>` | `Brand::Of<'a, A>`                     | `Brand::Of<'a, A>`                     | Yes |
| PartitionDispatch    | `Fn(A) -> bool`         | `Fn(&A) -> bool`         | `Brand::Of<'a, A>` | `&Brand::Of<'a, A>` | `(Brand::Of<'a, A>, Brand::Of<'a, A>)` | `(Brand::Of<'a, A>, Brand::Of<'a, A>)` | Yes |
| PartitionMapDispatch | `Fn(A) -> Result<O, E>` | `Fn(&A) -> Result<O, E>` | `Brand::Of<'a, A>` | `&Brand::Of<'a, A>` | `(Brand::Of<'a, E>, Brand::Of<'a, O>)` | `(Brand::Of<'a, E>, Brand::Of<'a, O>)` | Yes |

Return types match between Val and Ref in all three cases. The `A: Clone`
bound is needed on FilterDispatch and PartitionDispatch (same as filter/partition
free functions), and must appear on both impls.

**WithIndex group (no FnBrand):**

All five traits (MapWithIndexDispatch, FilterWithIndexDispatch,
FilterMapWithIndexDispatch, PartitionWithIndexDispatch,
PartitionMapWithIndexDispatch) follow the same pattern as their non-indexed
counterparts, with `Brand::Index` added as the first closure argument. The
`Brand: WithIndex` bound is added to the trait definition (or equivalently,
Val bounds `Brand: FunctorWithIndex` / `FilterableWithIndex` which imply
`WithIndex`). Return types match between Val and Ref. All verified.

**WithIndex + FnBrand group:**

The four traits (FoldMapWithIndexDispatch, FoldRightWithIndexDispatch,
FoldLeftWithIndexDispatch, TraverseWithIndexDispatch) combine Index projection
with the FnBrand pattern from existing foldable/traversable dispatch. Verified
in issue 3 above. Return types match between Val and Ref. All verified.

**Complex group (WiltDispatch, WitherDispatch):**

Both follow TraverseDispatch closely. The `FnBrand` parameter is unused by Val
but present for uniformity. The `M` (applicative brand) parameter appears in
both impls. Return types match between Val and Ref (both return owned values
wrapped in `M::Of<...>`). Val bounds: `Brand: Witherable`. Ref bounds:
`Brand: RefWitherable, FnBrand: LiftFn`. Verified.

### One subtlety: filter/partition require `A: Clone`

FilterDispatch and PartitionDispatch need `A: Clone` because the by-value
Filterable trait methods require it (the predicate `Fn(A) -> bool` consumes
the value, but the element may need to be kept). The RefFilterable versions
also require `A: Clone` because they produce owned output from borrowed input.
This bound must appear on the dispatch trait itself (or on both impls), not
just on the free function. The existing FilterMapDispatch does NOT require
`A: Clone` because filter_map's closure returns `Option<B>` (a new value), not
a decision to keep the original.

### Recommendation

All proposed traits are verified correct. The key item to note is that
FilterDispatch and PartitionDispatch must add `A: Clone` to the trait
definition, matching the underlying trait methods.

---

## Issue 6: Bimap dispatch uses a different Kind hash

### Question

Bifunctor traits use a two-parameter kind
(`Kind!(type Of<'a, A: 'a, B: 'a>: 'a;)` -> `Kind_266801a817966495`) instead of
the standard one-parameter kind (`Kind_cdc7cd43dac7585f`). Does this affect
the dispatch trait pattern?

### Research findings

All existing dispatch traits use `Kind_cdc7cd43dac7585f` (one type param) for
the `Brand` bound. The bifunctor family uses `Kind_266801a817966495` (two type
params). `BimapDispatch` would be the first dispatch trait to use the
two-parameter kind.

This affects:

- The `Brand` bound on the trait definition and impls.
- The `Apply!` macro invocations in the FA type and return type, which must use
  the two-param form: `Apply!(<Brand as Kind!(type Of<'a, A: 'a, B: 'a>: 'a;)>::Of<'a, A, C>)`.
- The trait bound must be `Brand: Kind_266801a817966495` (or equivalently
  `Brand: Bifunctor` since Bifunctor requires this kind).

The pattern is mechanically identical; only the kind hash and Apply macro
invocation differ. No E0119 risk or structural concern.

### Recommendation

The plan should explicitly note that bifunctorial dispatch traits use the
two-parameter kind hash. Implementers should model the Apply! invocations on
the existing `bifunctor.rs` free function rather than on other dispatch traits.

---

## Summary of findings

1. **Method naming:** Use `dispatch_*` qualified names for consistency with the
   majority of existing dispatch traits. The bare `dispatch` pattern in foldable
   is a minor existing inconsistency.

2. **Bimap vs compose_kleisli:** The bimap dispatch pattern is structurally closer
   to FunctorDispatch (with an FA container parameter) than ComposeKleisliDispatch
   (no container parameter). The plan's comparison to compose_kleisli is about
   the tuple `(F, G)` as Self type, which is correct, but the overall trait
   shape is different. Uses a different Kind hash (two-param).

3. **WithIndex + FnBrand combination:** Composes cleanly. No novel patterns needed.
   Brand::Index appears in the same position in both closures and does not
   interfere with dispatch.

4. **ref_fold_left_with_index fix:** Very small blast radius: 2 trait
   definitions, 2 default impl bodies, 2 doc tests, 0 concrete overrides,
   0 external call sites. Safe to proceed. The send variant
   (`send_ref_fold_left_with_index`) must also be fixed.

5. **All proposed traits verified correct.** Val and Ref impls are
   distinguishable for every proposed dispatch trait. Return types match between
   Val and Ref in all cases. FilterDispatch and PartitionDispatch must include
   `A: Clone` in their bounds.
