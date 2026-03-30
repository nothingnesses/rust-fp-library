# Analysis: `thunk.rs`

**File:** `fp-library/src/types/thunk.rs`
**Role:** `Thunk<'a, A>`, a lightweight, non-memoized, single-use deferred computation with full HKT support.

## Design

`Thunk<'a, A>` is a newtype over `Box<dyn FnOnce() -> A + 'a>`. It represents a deferred computation that:

- Is consumed on evaluation (`FnOnce`, takes `self`).
- Is NOT memoized (no caching).
- Is lifetime-polymorphic (can capture borrows).
- Is NOT `Send` (the closure is not required to be `Send`).
- Cannot be cloned (`Box<dyn FnOnce>` is not `Clone`).

The `evaluate(self) -> A` method consumes the thunk, calling the inner closure exactly once. This clean ownership model makes `Thunk` the most flexible and fully-featured lazy type in the hierarchy.

## Trait Implementations

**HKT (via `ThunkBrand`):** Functor, Pointed, Lift, ApplyFirst, ApplySecond, Semiapplicative, Semimonad, MonadRec, Extract, Extend, Foldable, FoldableWithIndex, FunctorWithIndex, WithIndex.

**Value-level:** Deferrable, Semigroup (when `A: Semigroup`), Monoid (when `A: Monoid`), Debug.

**Not implemented (documented):** Traversable (requires `Clone` on the container), Eq/Ord/Show (require non-destructive evaluation via `&self`, impossible with `FnOnce`).

## Comparison with PureScript

PureScript's `Data.Lazy` is memoized and supports `Functor`, `Applicative`, `Monad`, `Comonad`, `Traversable`, `Eq`, `Ord`, `Show`, `Semiring`, `Ring`, etc. Rust's `Thunk` corresponds to a non-memoized, non-repeatable version of `Lazy`. The memoized equivalent is `Lazy`/`RcLazy`/`ArcLazy`.

| PureScript Instance | Rust `ThunkBrand` Equivalent |
| ------------------- | ---------------------------- |
| Functor             | Functor                      |
| Applicative         | Pointed + Semiapplicative    |
| Monad               | Semimonad                    |
| Comonad             | Extract + Extend             |
| Foldable            | Foldable                     |
| Traversable         | Not implemented (no Clone)   |
| Eq, Ord, Show       | Not possible (FnOnce)        |
| Semigroup, Monoid   | Semigroup, Monoid            |

## Issues

### 1. `Semimonad::bind` accepts `Fn` but inherent `bind` accepts `FnOnce`

The HKT trait `Semimonad::bind` requires `impl Fn(A) -> ...` (multi-call), while the inherent `Thunk::bind` accepts `impl FnOnce(A) -> ...` (single-call). For a single-element container, the function is only called once either way, so `Fn` is satisfied. However, this means the trait-level API is strictly weaker than what `Thunk` naturally supports. Users going through the free function `bind::<ThunkBrand, _, _>(f, thunk)` must provide an `Fn` closure even though `FnOnce` would suffice.

**Impact:** Low. This is a fundamental mismatch between Rust's function trait hierarchy and the HKT trait signatures, which must support multi-element containers too.

### 2. `Extend::extend` requires `A: Clone` unnecessarily

The `Extend` trait requires `A: Clone` in its signature (needed for types like `Vec` where `extend` duplicates the container). For `Thunk`, the implementation never clones `A`, making the bound a dead constraint that narrows usability.

**Impact:** Low. This is a trait-level constraint, not specific to `Thunk`.

### 3. `Lift::lift2` relies on inherent `bind` (FnOnce) rather than trait `bind` (Fn)

The `lift2` implementation captures `fb` by move inside a closure passed to `fa.bind(...)`. This closure consumes `fb`, so it is `FnOnce`, not `Fn`. It works because `lift2` calls the inherent `bind` method (which accepts `FnOnce`), not the trait method. If `lift2` were ever refactored to go through trait dispatch, it would fail to compile.

**Impact:** Low. The current code is correct but fragile to refactoring.

### 4. `Thunk::pure` has a redundant `where A: 'a` bound

The impl block already constrains `A: 'a`, making the where clause on `pure` redundant.

**Impact:** Negligible.

### 5. No `Eq`, `Ord`, `Display` implementations

These are impossible with `FnOnce` semantics (evaluation is destructive). PureScript can implement them because `force` is non-destructive. This is a fundamental limitation of the non-memoized design, not a flaw.

**Impact:** Expected limitation. Users who need these should use `Lazy`/`RcLazy` instead.

## Strengths

- Clean ownership semantics with `FnOnce`.
- Full HKT trait support (the most complete of any lazy type).
- Lifetime-polymorphic (unlike `Trampoline` which requires `'static`).
- Zero-overhead newtype over `Box<dyn FnOnce>`.
- Comprehensive QuickCheck tests for all type class laws.
- Well-documented limitations and design rationale.
