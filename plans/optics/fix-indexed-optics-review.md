# Fix Indexed Optics Implementation Review

## Overview

This document analyses the staged diff (`git diff --cached -- . ':(exclude)plans' ':(exclude)docs'`) against the plan in `plans/optics/fix-indexed-optics-issues.md`, covering Steps 1–5 (Step 6 — doc test fixes — is intentionally deferred).

**Build status:** `cargo check --workspace` passes.

**Files modified (7):**
- `fp-library/src/classes/optics/indexed_traversal.rs` — Step 1
- `fp-library/src/types/optics/indexed_traversal.rs` — Steps 1, 3a
- `fp-library/src/types/optics/indexed_fold.rs` — Steps 2, 3b
- `fp-library/src/types/optics/indexed_getter.rs` — Step 3c
- `fp-library/src/types/optics/indexed_setter.rs` — Step 3d
- `fp-library/src/types/optics/functions.rs` — Step 4
- `fp-library/src/functions.rs` — Step 5

---

## 1. Correctly Implemented Components

### 1.1 Step 1: Remove `'b` from `IndexedTraversalFunc::apply` — Correct

The unused `'b` lifetime parameter has been removed from:
- **Trait definition** (`classes/optics/indexed_traversal.rs:12`): `fn apply<'b, M>` → `fn apply<M>` ✓
- **`Traversed<Brand>` impl** (`types/optics/indexed_traversal.rs:104`): removed ✓
- **All doc example `MyTraversal` structs** across `indexed_traversal.rs` (10 occurrences): all updated ✓
- **`PositionsTraversalFunc` impl** (`types/optics/functions.rs:991`): removed ✓

The `IWanderAdapter` impls of `TraversalFunc` correctly retain their `'b` — they implement `TraversalFunc`, not `IndexedTraversalFunc`, so the plan says to leave them alone.

### 1.2 Step 2: Remove `Q` from `IndexedFoldFunc::apply` — Correct

- **Trait definition** (`types/optics/indexed_fold.rs:33`): `fn apply<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>` → `fn apply<R: 'a + Monoid + 'static>` ✓
- **`Folded<Brand>` impl** (`indexed_fold.rs:190`): `Q` removed ✓
- **Call sites** (`indexed_fold.rs:341,611`): `fold_fn.apply::<R, Q>(...)` → `fold_fn.apply::<R>(...)` ✓
- **All 7 doc example `MyFold` structs**: all updated ✓

### 1.3 Step 3: Add Adapter Trait Impls — Correct (with issues noted in §2)

All **14 adapter impls** are present (2 traits × 7 types):

| Type | `IndexedOpticAdapter` | `IndexedOpticAdapterDiscardsFocus` |
|------|----------------------|-----------------------------------|
| `IndexedTraversal` | ✓ `P: Wander` | ✓ `P: Wander` |
| `IndexedTraversalPrime` | ✓ `P: Wander` | ✓ `P: Wander` |
| `IndexedFold` | ✓ `P = ForgetBrand<Q2, R>` | ✓ `P = ForgetBrand<Q2, R>` |
| `IndexedFoldPrime` | ✓ `P = ForgetBrand<Q2, R>` | ✓ `P = ForgetBrand<Q2, R>` |
| `IndexedGetter` | ✓ `P = ForgetBrand<Q2, R>` | ✓ `P = ForgetBrand<Q2, R>` |
| `IndexedSetter` | ✓ `P = FnBrand<Q2>` | ✓ `P = FnBrand<Q2>` |
| `IndexedSetterPrime` | ✓ `P = FnBrand<Q2>` | ✓ `P = FnBrand<Q2>` |

Each impl delegates to the correct evaluate method:
- Traversal → `IndexedTraversalOptic::evaluate(self, pab)` ✓
- Fold → `IndexedFoldOptic::evaluate::<R, Q2>(self, pab)` ✓
- Getter → `IndexedGetterOptic::evaluate::<R, Q2>(self, pab)` ✓
- Setter → `IndexedSetterOptic::evaluate(self, pab)` ✓

The `IndexedGetterPrime` type alias automatically inherits the `IndexedGetter` impls ✓

### 1.4 Step 4: Fix `positions` Semantics — Correct

- Old `Positions<Brand, A>` struct removed ✓
- New `PositionsTraversalFunc<F>` wraps a `TraversalFunc` ✓
- `#[derive(Clone)]` applied ✓
- `Cell<usize>` counter correctly increments per focus ✓
- `positions()` now takes a `Traversal<'a, P, S, T, A, B, F>` argument ✓
- Returns `IndexedTraversal<'a, P, usize, S, T, A, B, PositionsTraversalFunc<F>>` ✓
- Focus type is `A` (the original element), not `I` (the index) ✓
- `TraversableWithIndex` import removed, `Traversal` import added ✓
- Doc examples updated with correct semantics (element + position index) ✓

### 1.5 Step 5: Add `functions.rs` Re-exports — Correct

All 9 indexed functions plus `optics_compose` re-exported:
`optics_as_index`, `optics_compose`, `optics_indexed_fold_map`, `optics_indexed_over`, `optics_indexed_preview`, `optics_indexed_set`, `optics_indexed_view`, `optics_reindexed`, `optics_un_index`, `positions` ✓

---

## 2. Issues

### 2.1 Doc Examples for Fold/Getter Adapter Impls Create Unused Variables

**Severity: Low**

In all four fold adapter impls (`IndexedOpticAdapter` and `IndexedOpticAdapterDiscardsFocus` for both `IndexedFold` and `IndexedFoldPrime`), the doc examples create an `optics_un_index` / `optics_as_index` result but never assert on it. For example (`indexed_fold.rs:384-386`):

```rust
let l = IndexedFold::<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, _>::folded::<VecBrand>();
let unindexed = optics_un_index::<ForgetBrand<RcBrand, String>, _, _, _, _, _, _, _>(&l);
assert_eq!(optics_indexed_fold_map::<RcBrand, _, _, _, _, String, _>(&l, |_, x| x.to_string(), vec![1, 2]), "12");
```

The `unindexed` variable is assigned but never used — the assertion operates on `&l` (the indexed fold), not `&unindexed`. This demonstrates that `optics_un_index` can be called but does not demonstrate that the returned optic works correctly.

The same pattern appears in the `optics_as_index` examples — the `unindexed` variable is dead.

This is likely because using an un-indexed fold requires calling non-indexed fold functions with the specific `ForgetBrand` profunctor, which is cumbersome. However, the examples should ideally demonstrate end-to-end usage or omit the dead variable.

**Affected locations:** `indexed_fold.rs` lines 384-386, 434-436, 650-652, 696-698.

### 2.2 `IWanderAdapter`'s `TraversalFunc::apply` Has Unused `'b`

**Severity: Low (pre-existing, cosmetic)**

The `IWanderAdapter` structs (inside `IndexedTraversalOptic::evaluate` for both `IndexedTraversal` at line 390 and `IndexedTraversalPrime` at line 779) implement `TraversalFunc::apply` with `fn apply<'b, M: Applicative>`. However, the `TraversalFunc` trait defines `fn apply<M: Applicative>` (no `'b`). The `'b` is unused.

This was pre-existing code not introduced by this diff. The plan says "leave those alone unless `TraversalFunc::apply` also has `'b`" — and `TraversalFunc::apply` does NOT have `'b`. However, since this code compiled before and the plan explicitly says to leave it, this is not a regression but a missed opportunity for cleanup.

### 2.3 Redundant `Q2: UnsizedCoercible` Bound on Setter Adapter Impls

**Severity: Low (cosmetic)**

All four `IndexedSetter`/`IndexedSetterPrime` adapter impls have `Q2: UnsizedCoercible + 'static` in the impl generics AND `Q2: UnsizedCoercible` in the where clause. The where clause is redundant since the impl header already states the bound. For example (`indexed_setter.rs:623-627`):

```rust
impl<'a, Q2: UnsizedCoercible + 'static, P, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a, F>
    IndexedOpticAdapter<'a, FnBrand<Q2>, I, S, T, A, B> for IndexedSetter<'a, P, I, S, T, A, B, F>
where
    F: IndexedSetterFunc<'a, I, S, T, A, B> + Clone + 'a,
    Q2: UnsizedCoercible,  // redundant — already in impl header
```

Not a bug but is inconsistent with the other adapter impls (fold, getter, traversal) which don't duplicate bounds.

**Affected locations:** `indexed_setter.rs` lines 627, 667, 704, 742.

### 2.4 `PositionsTraversalFunc` Has Public Field

**Severity: Low (design note)**

`PositionsTraversalFunc<F>` is declared as `pub struct PositionsTraversalFunc<F>(pub F)` — the inner field is `pub`. This means users can construct and destructure it directly rather than going through `positions()`. The doc example demonstrates this direct construction:

```rust
let p = PositionsTraversalFunc(t.traversal);
```

This is fine functionally, but if encapsulation is desired, the field could be made `pub(crate)`. The old `Positions<Brand, A>` also had public visibility through `PhantomData`. This is consistent with other internal structs in the codebase (e.g., `Traversed`, `Folded`, `Mapped`).

---

## 3. Completeness Check Against Plan

| Plan Step | Status | Notes |
|-----------|--------|-------|
| Step 1: Remove `'b` from `IndexedTraversalFunc::apply` | **Complete** | Trait + all impls + all doc examples updated |
| Step 2: Remove `Q` from `IndexedFoldFunc::apply` | **Complete** | Trait + all impls + all call sites + all doc examples updated |
| Step 3a: Traversal adapter impls | **Complete** | 4 impls (2 traits × 2 structs) |
| Step 3b: Fold adapter impls | **Complete** | 4 impls (2 traits × 2 structs) |
| Step 3c: Getter adapter impls | **Complete** | 2 impls (2 traits × 1 struct, alias gets it free) |
| Step 3d: Setter adapter impls | **Complete** | 4 impls (2 traits × 2 structs) |
| Step 4: Fix `positions` semantics | **Complete** | Old struct removed, new Cell-based impl, takes `Traversal` arg |
| Step 5: Re-exports | **Complete** | All 9 indexed functions re-exported |
| Step 6: Fix doc tests | **Skipped** | Intentionally deferred |

---

## 4. Summary

### What's correct (all structural changes):
- `IndexedTraversalFunc::apply` no longer has unused `'b` lifetime
- `IndexedFoldFunc::apply` no longer has unused `Q` type parameter
- All 14 adapter trait impls added for non-lens indexed optics
- `positions` now matches PureScript semantics (takes a `Traversal`, uses `Cell<usize>` counter, preserves element type as focus)
- All indexed bridge functions re-exported from `functions.rs`

### What needs attention:

| # | Issue | Severity | Category |
|---|-------|----------|----------|
| 1 | Fold/getter adapter doc examples have unused variables | Low | Documentation |
| 2 | `IWanderAdapter::apply` has unused `'b` (pre-existing) | Low | Cosmetic |
| 3 | Redundant `Q2: UnsizedCoercible` where-clause on setter impls | Low | Cosmetic |
| 4 | `PositionsTraversalFunc` has `pub` inner field | Low | Design note |
