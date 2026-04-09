# SendRef and ParRef Impact Analysis

How would a Ref-hierarchy change from consuming containers to borrowing them
affect the SendRef and ParRef trait families?

## Part 1: SendRef Traits

### 1.1 Current container parameter conventions

All SendRef trait methods consume the container (`fa: Self::Of<'a, A>`), matching
the Ref hierarchy's current consume-container convention. There is no borrowing
variant.

Complete inventory of SendRef trait methods and their container parameters:

| Trait                      | Method                           | Container param                 |
| -------------------------- | -------------------------------- | ------------------------------- |
| `SendRefFunctor`           | `send_ref_map`                   | `fa: Self::Of<'a, A>` (owned)   |
| `SendRefFunctorWithIndex`  | `send_ref_map_with_index`        | `fa: Self::Of<'a, A>` (owned)   |
| `SendRefPointed`           | `send_ref_pure`                  | `a: &A` (no container)          |
| `SendRefLift`              | `send_ref_lift2`                 | `fa`, `fb`: both owned          |
| `SendRefSemiapplicative`   | `send_ref_apply`                 | `ff`, `fa`: both owned          |
| `SendRefSemimonad`         | `send_ref_bind`                  | `ma: Self::Of<'a, A>` (owned)   |
| `SendRefApplyFirst`        | `send_ref_apply_first`           | `fa`, `fb`: both owned          |
| `SendRefApplySecond`       | `send_ref_apply_second`          | `fa`, `fb`: both owned          |
| `SendRefApplicative`       | (marker trait, no methods)       | N/A                             |
| `SendRefMonad`             | (marker trait, no methods)       | N/A                             |
| `SendRefFoldable`          | `send_ref_fold_map`              | `fa: Self::Of<'a, A>` (owned)   |
|                            | `send_ref_fold_right`            | `fa: Self::Of<'a, A>` (owned)   |
|                            | `send_ref_fold_left`             | `fa: Self::Of<'a, A>` (owned)   |
| `SendRefFoldableWithIndex` | `send_ref_fold_map_with_index`   | `fa: Self::Of<'a, A>` (owned)   |
|                            | `send_ref_fold_right_with_index` | `fa: Self::Of<'a, A>` (owned)   |
|                            | `send_ref_fold_left_with_index`  | `fa: Self::Of<'a, A>` (owned)   |
| `SendRefCountedPointer`    | `send_new`                       | `value: T` (not a container op) |

### 1.2 Do SendRef traits have the same consume-container pattern as Ref traits?

Yes. Every SendRef method that takes a container parameter consumes it by value.
This directly mirrors the Ref hierarchy. The implementations on `ArcLazy` all call
`fa.ref_map(f)` or equivalent, which takes `self` (consuming the `ArcLazy`).

### 1.3 Should SendRef traits change in tandem with Ref traits?

Yes, they should change together. The reasoning:

1. **Structural mirroring.** The SendRef hierarchy is documented as "the
   thread-safe counterpart of Ref" in every trait's doc comment. Any semantic
   divergence between Ref and SendRef would create confusion and inconsistency.

2. **Shared implementation pattern.** The `ArcLazy` implementations of SendRef
   traits delegate to `fa.ref_map(f)`, the same inherent method used by the Ref
   implementations. If the Ref hierarchy changes to borrow, the inherent method
   will also need to change, and the SendRef impls will follow naturally.

3. **User expectations.** If `ref_map(&lazy, f)` borrows but `send_ref_map(lazy, f)`
   consumes, users would face inconsistent APIs for what are conceptually the same
   operation with different thread-safety requirements.

### 1.4 ArcLazy with borrowed `send_ref_map`

If `send_ref_map` took `&ArcLazy<A>` instead of `ArcLazy<A>`:

```rust
fn send_ref_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
    func: impl Fn(&A) -> B + Send + 'a,
    fa: &Self::Of<'a, A>,  // <-- borrowed
) -> Self::Of<'a, B>;
```

The implementation would clone the `ArcLazy` (which is an `Arc::clone`, a cheap
atomic increment) and then consume the clone:

```rust
fn send_ref_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
    func: impl Fn(&A) -> B + Send + 'a,
    fa: &Lazy<'a, A, ArcLazyConfig>,
) -> Lazy<'a, B, ArcLazyConfig> {
    fa.clone().ref_map(func)
}
```

This would work correctly. `ArcLazy` is `Clone` via `Arc::clone`, which is O(1)
(atomic reference count increment). The cloned `ArcLazy` shares the same
underlying memoization cell, so evaluating either the original or the clone
triggers a single computation. The memory overhead is one additional `Arc` strong
reference count.

**Caveats:**

- The `clone()` call is not zero-cost; it performs an atomic increment. However,
  this is already the case when users clone `ArcLazy` manually before passing it
  to the current consuming API. Borrowing just moves the clone site from the
  caller into the implementation.

- For `send_ref_bind`, the continuation `f` captures `&A` and returns a new
  `ArcLazy<B>`. Borrowing the container means the `ArcLazy<A>` must remain alive
  while the result is constructed. With `ArcLazy` this is trivially satisfied
  because `clone()` extends the lifetime via reference counting.

### 1.5 TryLazy with borrowed `send_ref_map`

`TryLazyBrand<E, ArcLazyConfig>` also implements `SendRefFunctor`. Its `ArcTryLazy`
is likewise backed by `Arc`, so the same clone-based strategy applies. No issues.

## Part 2: ParRef Traits

### 2.1 Current container parameter conventions

All ParRef trait methods consume the container, identical to the Ref and SendRef
hierarchies.

| Trait                       | Method                          | Container param               |
| --------------------------- | ------------------------------- | ----------------------------- |
| `ParRefFunctor`             | `par_ref_map`                   | `fa: Self::Of<'a, A>` (owned) |
| `ParRefFunctorWithIndex`    | `par_ref_map_with_index`        | `fa: Self::Of<'a, A>` (owned) |
| `ParRefFoldable`            | `par_ref_fold_map`              | `fa: Self::Of<'a, A>` (owned) |
| `ParRefFoldableWithIndex`   | `par_ref_fold_map_with_index`   | `fa: Self::Of<'a, A>` (owned) |
| `ParRefFilterable`          | `par_ref_filter_map`            | `fa: Self::Of<'a, A>` (owned) |
|                             | `par_ref_filter`                | `fa: Self::Of<'a, A>` (owned) |
| `ParRefFilterableWithIndex` | `par_ref_filter_map_with_index` | `fa: Self::Of<'a, A>` (owned) |
|                             | `par_ref_filter_with_index`     | `fa: Self::Of<'a, A>` (owned) |

### 2.2 Do ParRef traits already borrow containers, or do they consume?

They consume. Despite the fact that the function closures receive `&A` (references
to elements), the container itself is taken by value.

### 2.3 Implementations: `par_iter()` vs `into_par_iter()`

**VecBrand ParRef implementations** use `fa.par_iter()` (which borrows the `Vec`).
This means the implementations already borrow the `Vec` internally, even though
the trait signature consumes it. The `Vec` is owned by the function body, and
`par_iter()` borrows from it for the duration of the parallel operation.

**CatListBrand ParRef implementations** collect to `Vec<&A>` via `fa.iter()` (which
borrows the `CatList`), then use `v.into_par_iter()` on the temporary `Vec<&A>`.
Again, the `CatList` is consumed by the function signature but only borrowed
internally.

**Key observation:** Both `Vec` and `CatList` ParRef implementations only need a
borrow of the container to perform their work. The consume-by-value signature is
strictly more restrictive than what the implementation requires. Changing to
borrow-by-reference would align the trait signature with the actual implementation
needs.

### 2.4 Should ParRef traits also change to borrow?

Yes, for the same consistency reasons as SendRef, plus an additional benefit:

- **Alignment with `par_iter()`.** Rayon's `par_iter()` borrows the collection.
  The ParRef trait signatures should match this natural borrowing pattern rather
  than consuming the container and then borrowing internally.

- **Reuse without cloning.** Currently, if a user wants to `par_ref_map` over a
  `Vec` and then use that `Vec` again, they must clone it first. With borrowed
  signatures, the `Vec` remains available after the operation.

- **CatList efficiency.** CatList's `iter()` already borrows. Switching to a
  borrowed signature eliminates the need to consume the `CatList` that is only
  borrowed internally anyway.

## Part 3: Consistency Analysis

### 3.1 Should all three hierarchies change together?

Yes. The three trait families form a coherent system:

- **Ref** = by-reference element access, single-threaded.
- **SendRef** = by-reference element access, `Send + Sync` bounds.
- **ParRef** = by-reference element access, parallel execution.

If Ref changes to borrow containers, SendRef and ParRef should follow. The
alternatives and their consequences:

| Scenario                     | Consistency | User confusion | Implementor burden |
| ---------------------------- | ----------- | -------------- | ------------------ |
| All three borrow             | High        | Low            | Low                |
| Only Ref borrows             | Low         | High           | Medium             |
| Ref + ParRef borrow, SendRef | Low         | High           | Medium             |

### 3.2 Types where SendRef borrowing would NOT work

No types have been identified where SendRef borrowing would be problematic:

- **`ArcLazy`:** `Clone` via `Arc::clone` (O(1) atomic increment). Borrowing works
  by cloning internally.
- **`ArcTryLazy`:** Same as `ArcLazy`, backed by `Arc`.

If a future type implemented `SendRefFunctor` but was not `Clone`, borrowing would
require the implementation to work with a shared reference to the container. For
`ArcLazy`-style memoized types, this is always possible because the memoization
cell is shared via reference counting. For a hypothetical non-Clone, non-reference-
counted type, borrowing could be problematic. However, no such type exists in the
library, and it is difficult to imagine one that would be both thread-safe (for
SendRef) and non-Clone.

### 3.3 Types where ParRef borrowing would NOT work

No types have been identified where ParRef borrowing would be problematic:

- **`Vec`:** Already uses `par_iter()` (borrows). No issue.
- **`CatList`:** Uses `iter()` (borrows) to collect references. No issue.

Both types are `Clone`, so even if an implementation needed ownership, it could
clone from the borrow. But as shown above, the current implementations already
operate on borrows.

### 3.4 Supertrait implications

ParRef traits have Ref supertraits (`ParRefFunctor: RefFunctor`). If both Ref
and ParRef change to borrow simultaneously, the supertrait relationship remains
valid. If only Ref changes, the ParRef impls would still satisfy the Ref supertrait
(they would need to provide a borrowing `ref_map` alongside a consuming
`par_ref_map`), but the inconsistency would be confusing.

SendRef traits mostly do not have Ref supertraits (exception: `SendRefFoldable:
RefFoldable`). Changing both together avoids introducing new inconsistencies.

## Part 4: By-Value Hierarchy Context

### 4.1 Does `Functor::map` truly need to consume containers?

`Functor::map` has signature:

```rust
fn map<'a, A: 'a, B: 'a>(
    f: impl Fn(A) -> B + 'a,
    fa: Self::Of<'a, A>,
) -> Self::Of<'a, B>;
```

The closure takes `A` by value (`Fn(A) -> B`), not by reference. This means the
implementation must extract owned `A` values from the container. For this reason,
the container is consumed: the elements are moved out of it.

**Could it work with borrows?** Not without changing the closure signature. If the
container were borrowed, the implementation would have `&Self::Of<'a, A>`, which
only provides `&A` references to elements. But the closure expects owned `A`, so
the implementation would need to clone each element, requiring `A: Clone`. This
would be a strictly weaker API (extra bound) for no benefit.

The by-value hierarchy's consume-container pattern is fundamentally different from
the Ref hierarchy's. In the Ref hierarchy, the closure already receives `&A`, so
the container does not need to give up ownership of elements. The container is
consumed for an unrelated reason (the `Lazy::ref_map` implementation moves `self`
into a new closure). That implementation detail is what the ref-borrow proposal
aims to change.

### 4.2 Where by-value borrow could work (hypothetically)

For types where the container uses shared ownership internally (e.g., `Vec` with
`Clone`), one could imagine `map(&vec, f)` that clones elements. But this would:

1. Add a `Clone` bound on `A`, restricting generality.
2. Be equivalent to `map(vec.clone(), f)`, which callers can already do.
3. Not save any work (elements are cloned either way).

For `ArcLazy`, `map(&lazy, f)` would require `A: Clone` because the closure
receives an owned `A`, which would have to be cloned from the memoized value.
The current API avoids this by consuming the `ArcLazy` and building a chain.

**Conclusion:** The by-value hierarchy genuinely requires container consumption
because the closure consumes elements. The Ref hierarchy's consume-container
pattern, by contrast, is an implementation artifact that can be changed to
borrowing without affecting the closure signature or requiring additional bounds.

## Summary

| Question                                                           | Answer                                                                                                 |
| ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------ |
| Do SendRef traits consume containers?                              | Yes, all of them.                                                                                      |
| Should SendRef change with Ref?                                    | Yes, for consistency.                                                                                  |
| Would ArcLazy work with borrowed `send_ref_map`?                   | Yes, via cheap `Arc::clone`.                                                                           |
| Do ParRef traits consume containers?                               | Yes, all of them.                                                                                      |
| Do Vec/CatList ParRef impls use `par_iter()` or `into_par_iter()`? | Vec: `par_iter()` (borrows). CatList: `iter()` then `into_par_iter()` on refs. Both borrow internally. |
| Should ParRef change with Ref?                                     | Yes, for consistency and to match `par_iter()`.                                                        |
| Any types where SendRef/ParRef borrowing fails?                    | None identified.                                                                                       |
| Does by-value `Functor::map` need to consume?                      | Yes, because `Fn(A) -> B` consumes elements.                                                           |
