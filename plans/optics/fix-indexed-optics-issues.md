# Plan: Fix Indexed Optics Issues

## Context

The indexed optics implementation (diff `5757637b`→`8630bc4`) was reviewed against the plan (`plans/optics/indexed-optics.md`) and the PureScript reference. The review identified 10 issues. This plan addresses all actionable ones (7 fixes, 3 documented as known limitations).

## Steps

### Step 1: Remove unused `'b` from `IndexedTraversalFunc::apply` (Issue 7)

Remove the `'b` lifetime parameter from the trait definition and all implementations.

**Files:**
- [indexed_traversal.rs](fp-library/src/classes/optics/indexed_traversal.rs) — trait definition: `fn apply<'b, M>` → `fn apply<M>`
- [indexed_traversal.rs](fp-library/src/types/optics/indexed_traversal.rs) — all `IndexedTraversalFunc` impls (`Traversed`, doc example structs): remove `'b`. Note: the `IWanderAdapter` impls `TraversalFunc` (not `IndexedTraversalFunc`) — leave those alone unless `TraversalFunc::apply` also has `'b` (check and remove if so)
- [functions.rs](fp-library/src/types/optics/functions.rs) — `Positions` impl of `IndexedTraversalFunc`: remove `'b`

### Step 2: Remove unused `Q` from `IndexedFoldFunc::apply` (Issue 8)

Remove the `Q: UnsizedCoercible + 'static` type parameter from the trait definition, all implementations, and all call sites.

**Files:**
- [indexed_fold.rs](fp-library/src/types/optics/indexed_fold.rs):
	- Trait definition (line ~32): `fn apply<R: ..., Q: UnsizedCoercible + 'static>` → `fn apply<R: ...>`
	- `Folded<Brand>` impl (line ~192): remove `Q`
	- `IndexedFoldOptic::evaluate` call sites (lines ~340, ~510): `fold_fn.apply::<R, Q>(...)` → `fold_fn.apply::<R>(...)`
	- All doc example `MyFold` structs: remove `Q` from their `apply` signatures

### Step 3: Add adapter trait impls for non-lens indexed optics (Issue 2)

Currently `IndexedOpticAdapter` and `IndexedOpticAdapterDiscardsFocus` are only on `IndexedLens`/`IndexedLensPrime`. Add impls for all other indexed optic types so `optics_un_index`, `optics_as_index`, and `optics_reindexed` work with them.

Pattern to follow (from [indexed_lens.rs:238-262](fp-library/src/types/optics/indexed_lens.rs#L238-L262)):
```rust
impl<'a, P: Strong, I, S, T, A, B, Q> IndexedOpticAdapter<'a, P, I, S, T, A, B>
		for IndexedLens<'a, Q, I, S, T, A, B>
{
		fn evaluate_indexed(&self, pab: Indexed<'a, P, I, A, B>) -> P::Of<'a, S, T> {
				<Self as IndexedLensOptic>::evaluate::<P>(self, pab)
		}
}
```

#### 3a. IndexedTraversal / IndexedTraversalPrime

**File:** [indexed_traversal.rs](fp-library/src/types/optics/indexed_traversal.rs)

Both adapter traits, bound `P: Wander`, delegate to `IndexedTraversalOptic::evaluate::<P>`. Four impls total (2 traits × 2 structs).

#### 3b. IndexedFold / IndexedFoldPrime

**File:** [indexed_fold.rs](fp-library/src/types/optics/indexed_fold.rs)

Folds are monomorphic in profunctor — they only work with `ForgetBrand<Q, R>`. Impl:
```rust
impl<'a, Q2: UnsizedCoercible + 'static, R: 'a + Monoid + 'static, P, I, S, T, A, B, F>
		IndexedOpticAdapter<'a, ForgetBrand<Q2, R>, I, S, S, A, A>
		for IndexedFold<'a, P, I, S, T, A, B, F>
where F: IndexedFoldFunc<'a, I, S, A> + Clone + 'a
{
		fn evaluate_indexed(&self, pab: Indexed<'a, ForgetBrand<Q2, R>, I, A, A>) -> Forget<Q2, R, S, S> {
				IndexedFoldOptic::evaluate::<R, Q2>(self, pab)
		}
}
```
Four impls total (2 traits × 2 structs).

#### 3c. IndexedGetter (+ IndexedGetterPrime which is a type alias)

**File:** [indexed_getter.rs](fp-library/src/types/optics/indexed_getter.rs)

Same as fold but `R: 'a + 'static` (no `Monoid` required), delegate to `IndexedGetterOptic::evaluate::<R, Q2>`. Two impls (2 traits × 1 struct, alias gets it free).

#### 3d. IndexedSetter / IndexedSetterPrime

**File:** [indexed_setter.rs](fp-library/src/types/optics/indexed_setter.rs)

Fix `P = FnBrand<Q2>`, delegate to `IndexedSetterOptic::evaluate::<Q2>`. Four impls total.

### Step 4: Fix `positions` semantics (Issue 1 — HIGH)

**Current problem:** `positions()` takes no arguments, discards the element `_a`, and focuses on indices. Should take a `Traversal` and decorate each focus with its integer position.

**File:** [functions.rs](fp-library/src/types/optics/functions.rs)

**4a.** Replace `Positions<Brand, A>` with `PositionsTraversalFunc<F>`:
```rust
#[derive(Clone)]
pub struct PositionsTraversalFunc<F>(F);

impl<'a, S, T, A, B, F> IndexedTraversalFunc<'a, usize, S, T, A, B>
		for PositionsTraversalFunc<F>
where F: TraversalFunc<'a, S, T, A, B> + Clone + 'a
{
		fn apply<M: Applicative>(
				&self, f: Box<dyn Fn(usize, A) -> M::Of<'a, B> + 'a>, s: S,
		) -> M::Of<'a, T> {
				let counter = std::cell::Cell::new(0usize);
				self.0.apply::<M>(
						Box::new(move |a: A| {
								let i = counter.get();
								counter.set(i + 1);
								f(i, a)
						}),
						s,
				)
		}
}
```

**4b.** Update `positions` function to take a `Traversal` and return `IndexedTraversal<usize, S, T, A, B>`:
```rust
pub fn positions<'a, P, S, T, A, B, F>(
		traversal: Traversal<'a, P, S, T, A, B, F>,
) -> IndexedTraversal<'a, P, usize, S, T, A, B, PositionsTraversalFunc<F>>
where
		P: UnsizedCoercible,
		F: TraversalFunc<'a, S, T, A, B> + Clone + 'a,
{
		IndexedTraversal::new(PositionsTraversalFunc(traversal.traversal))
}
```

**4c.** Remove old `Positions<Brand, A>` struct and its impls.

### Step 5: Add `functions.rs` re-exports (Issue 3)

**File:** [functions.rs](fp-library/src/functions.rs)

Add after line 39 (`pub use crate::types::optics::optics_compose;`):
```rust
pub use crate::types::optics::{
		optics_indexed_view,
		optics_indexed_over,
		optics_indexed_set,
		optics_indexed_preview,
		optics_indexed_fold_map,
		optics_un_index,
		optics_as_index,
		optics_reindexed,
		positions,
};
```

### Step 6: Fix doc tests (Issue 9)

Run `cargo test --doc -p fp-library` and fix all failing doc examples. Key changes needed:
- Remove `'b` from any `IndexedTraversalFunc::apply` examples
- Remove `Q` from any `IndexedFoldFunc::apply` examples
- Update `positions` examples to pass a `Traversal` argument and assert on element foci (not index foci)
- Update adapter trait doc examples to show usage with non-lens types

**Files:** All indexed optic files (`indexed_traversal.rs`, `indexed_fold.rs`, `indexed_getter.rs`, `indexed_setter.rs`, `functions.rs`)

### Issues Documented as Known Limitations (no code changes)

- **Issue 4:** `optics_un_index`/`optics_as_index` return `impl Optic<P, ...>` monomorphic in `P`. This is a Rust limitation (no rank-2 types in return position). Each call site gets a fixed `P`.
- **Issue 5:** No standalone `iwander` function. `IndexedTraversal::new(f)` serves the same role — `iwander` would be a trivial wrapper. Not adding.
- **Issue 6:** `TraversableWithIndex` requires `Clone` bounds. Consistent with non-indexed `Traversable`.
- **Issue 10:** Indexed + Indexed composition not addressed. Future work.

## Verification

1. `cargo check --workspace` after each step
2. `cargo test --doc -p fp-library` after Step 6
3. `cargo test --workspace` for full test suite
4. `cargo clippy --workspace --all-features`
5. `cargo fmt --all -- --check`
