# Plan: Fix Remaining Issues from Indexed Optics Fix Review

## Context

The fix implementation (Steps 1–5 of `plans/optics/fix-indexed-optics-issues.md`) was reviewed in `plans/optics/fix-indexed-optics-review.md`. All structural changes are correct, but 4 low-severity issues were found. This plan addresses all of them.

## Steps

### Step 1: Fix unused `unindexed` variables in fold adapter doc examples

**File:** [indexed_fold.rs](fp-library/src/types/optics/indexed_fold.rs)

Four doc examples create an `unindexed` variable from `optics_un_index`/`optics_as_index` but never use it — the assertion operates on the original `&l`. Change `let unindexed = ...` to `let _unindexed = ...` to suppress the unused-variable warning while still demonstrating the function compiles.

(The getter adapter doc examples at [indexed_getter.rs:241-242, 283-284](fp-library/src/types/optics/indexed_getter.rs#L241) are correct — they do use the `unindexed` variable with `optics_view`.)

**4 locations:**
- Line ~385: `IndexedFold` `IndexedOpticAdapter` example — `let unindexed = optics_un_index::<...>(&l);`
- Line ~435: `IndexedFold` `IndexedOpticAdapterDiscardsFocus` example — `let unindexed = optics_as_index::<...>(&l);`
- Line ~651: `IndexedFoldPrime` `IndexedOpticAdapter` example — `let unindexed = optics_un_index::<...>(&l);`
- Line ~697: `IndexedFoldPrime` `IndexedOpticAdapterDiscardsFocus` example — `let unindexed = optics_as_index::<...>(&l);`

### Step 2: Remove unused `'b` from `IWanderAdapter`'s `TraversalFunc::apply`

**File:** [indexed_traversal.rs](fp-library/src/types/optics/indexed_traversal.rs)

The `IWanderAdapter` structs implement `TraversalFunc::apply` with `fn apply<'b, M: Applicative>`, but `TraversalFunc::apply` does not have `'b` (confirmed at [traversal.rs:44](fp-library/src/classes/optics/traversal.rs#L44)). The `'b` is unused and should be removed.

**2 locations:**
- Line ~390: `IWanderAdapter` inside `IndexedTraversal`'s evaluate — `fn apply<'b, M: Applicative>` → `fn apply<M: Applicative>`
- Line ~779: `IWanderAdapter` inside `IndexedTraversalPrime`'s evaluate — same change

### Step 3: Remove redundant `Q2: UnsizedCoercible` from setter adapter where-clauses

**File:** [indexed_setter.rs](fp-library/src/types/optics/indexed_setter.rs)

All 4 setter adapter impls have `Q2: UnsizedCoercible + 'static` in the impl header AND `Q2: UnsizedCoercible` in the where clause. Remove the redundant where-clause entry.

**4 locations:**
- Line ~627: `IndexedSetter` `IndexedOpticAdapter` — remove `Q2: UnsizedCoercible,` from where clause
- Line ~667: `IndexedSetter` `IndexedOpticAdapterDiscardsFocus` — same
- Line ~704: `IndexedSetterPrime` `IndexedOpticAdapter` — same
- Line ~742: `IndexedSetterPrime` `IndexedOpticAdapterDiscardsFocus` — same

### Step 4: Make `PositionsTraversalFunc` field private

**File:** [functions.rs](fp-library/src/types/optics/functions.rs)

Change `pub struct PositionsTraversalFunc<F>(pub F)` to `pub struct PositionsTraversalFunc<F>(F)` at line ~955. Construction goes through the `positions()` function; the `pub` field is not needed. This is consistent with similar wrapper structs (`Traversed`, `Folded`, `Mapped`) which all have private fields.

Also update the doc example for `positions` if it directly constructs `PositionsTraversalFunc(t.traversal)` — change it to use `positions(t)` instead.

## Verification

1. `cargo check --workspace`
2. `cargo test --doc -p fp-library` (doc examples must still compile)
3. `cargo clippy --workspace --all-features`
4. `cargo fmt --all -- --check`
