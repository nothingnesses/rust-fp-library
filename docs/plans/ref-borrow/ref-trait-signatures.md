# Ref Trait Signatures: Borrow Analysis

This document catalogs every method on every Ref trait, identifies which parameters
consume the container by value, and assesses whether each consuming parameter
could instead accept a borrow.

## Notation

- `Of<A>` is shorthand for `Apply!(<Self as Kind!(...)>::Of<'a, A>)`.
- "Consumes" means the parameter is taken by value (moved into the function).
- "Could borrow" is the assessment of whether a `&Of<A>` would suffice.

---

## 1. RefFunctor

File: `fp-library/src/classes/ref_functor.rs`

| Method    | Signature (simplified)                                   | Container params | Consumes? | Could borrow?   | Notes                                                                                                                                                                                                          |
| --------- | -------------------------------------------------------- | ---------------- | --------- | --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ref_map` | `fn ref_map(func: impl Fn(&A) -> B, fa: Of<A>) -> Of<B>` | `fa`             | Yes       | Depends on type | For Vec, impl calls `fa.iter().map(func).collect()` (borrow OK). For Lazy, impl moves `fa` into a new closure (`RcLazy::new(move \|\| f(self.evaluate()))`), requiring ownership. Required method, no default. |

---

## 2. RefPointed

File: `fp-library/src/classes/ref_pointed.rs`

| Method     | Signature (simplified)                       | Container params | Consumes? | Could borrow? | Notes                                                               |
| ---------- | -------------------------------------------- | ---------------- | --------- | ------------- | ------------------------------------------------------------------- |
| `ref_pure` | `fn ref_pure(a: &A) -> Of<A> where A: Clone` | None             | N/A       | N/A           | No container input; creates a new container from a value reference. |

---

## 3. RefLift

File: `fp-library/src/classes/ref_lift.rs`

| Method      | Signature (simplified)                                                    | Container params | Consumes?     | Could borrow?   | Notes                                                                                                                                                                                 |
| ----------- | ------------------------------------------------------------------------- | ---------------- | ------------- | --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ref_lift2` | `fn ref_lift2(func: impl Fn(&A, &B) -> C, fa: Of<A>, fb: Of<B>) -> Of<C>` | `fa`, `fb`       | Both consumed | Depends on type | For Vec, impl iterates both with `.iter()` (borrow OK). For Lazy, both moved into closure (`RcLazy::new(move \|\| func(fa.evaluate(), fb.evaluate()))`). Required method, no default. |

---

## 4. RefSemiapplicative

File: `fp-library/src/classes/ref_semiapplicative.rs`

Supertraits: `RefLift + RefFunctor`

| Method      | Signature (simplified)                                       | Container params | Consumes?     | Could borrow?   | Notes                                                                                                                                          |
| ----------- | ------------------------------------------------------------ | ---------------- | ------------- | --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `ref_apply` | `fn ref_apply(ff: Of<CloneFn::Of<A,B>>, fa: Of<A>) -> Of<B>` | `ff`, `fa`       | Both consumed | Depends on type | For Vec, impl uses `ff.iter().flat_map(\|f\| fa.iter().map(...))` (borrow OK). For Lazy, both moved into closure. Required method, no default. |

---

## 5. RefSemimonad

File: `fp-library/src/classes/ref_semimonad.rs`

| Method     | Signature (simplified)                                     | Container params | Consumes? | Could borrow?   | Notes                                                                                                                                                                                    |
| ---------- | ---------------------------------------------------------- | ---------------- | --------- | --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ref_bind` | `fn ref_bind(fa: Of<A>, f: impl Fn(&A) -> Of<B>) -> Of<B>` | `fa`             | Yes       | Depends on type | For Vec, impl calls `fa.iter().flat_map(f).collect()` (borrow OK). For Lazy, impl calls `f(fa.evaluate())` which needs the owned `fa` alive for the borrow. Required method, no default. |

### Free functions

| Function   | Signature (simplified)                                    | Container params | Consumes? | Could borrow? | Notes                                                                              |
| ---------- | --------------------------------------------------------- | ---------------- | --------- | ------------- | ---------------------------------------------------------------------------------- |
| `ref_join` | `fn ref_join(mma: Of<Of<A>>) -> Of<A> where Of<A>: Clone` | `mma`            | Yes       | No            | Calls `Brand::ref_bind(mma, \|ma\| ma.clone())`. Inner value cloned from `&Of<A>`. |

---

## 6. RefApplicative

File: `fp-library/src/classes/ref_applicative.rs`

Marker trait combining `RefPointed + RefSemiapplicative + RefApplyFirst + RefApplySecond`.
No own methods. Blanket-implemented.

---

## 7. RefMonad

File: `fp-library/src/classes/ref_monad.rs`

Marker trait combining `RefApplicative + RefSemimonad`.
No own methods. Blanket-implemented.

### Free functions

| Function       | Signature (simplified)                                                                            | Container params                     | Consumes?          | Could borrow? | Notes                                                                         |
| -------------- | ------------------------------------------------------------------------------------------------- | ------------------------------------ | ------------------ | ------------- | ----------------------------------------------------------------------------- |
| `ref_if_m`     | `fn ref_if_m(cond: Of<bool>, then_branch: Of<A>, else_branch: Of<A>) -> Of<A> where Of<A>: Clone` | `cond`, `then_branch`, `else_branch` | All three consumed | Partial       | `cond` consumed by `ref_bind`. Branches moved into closure and cloned inside. |
| `ref_unless_m` | `fn ref_unless_m(cond: Of<bool>, action: Of<()>) -> Of<()> where Of<()>: Clone`                   | `cond`, `action`                     | Both consumed      | Partial       | Same pattern as `ref_if_m`.                                                   |

---

## 8. RefApplyFirst

File: `fp-library/src/classes/ref_apply_first.rs`

Supertrait: `RefLift`

| Method            | Signature (simplified)                                             | Container params | Consumes?     | Could borrow?             | Notes                                                                                                         |
| ----------------- | ------------------------------------------------------------------ | ---------------- | ------------- | ------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `ref_apply_first` | `fn ref_apply_first(fa: Of<A>, fb: Of<B>) -> Of<A> where A: Clone` | `fa`, `fb`       | Both consumed | Inherits from `ref_lift2` | Default: `Self::ref_lift2(\|a: &A, _: &B\| a.clone(), fa, fb)`. Blanket-implemented for all `Brand: RefLift`. |

---

## 9. RefApplySecond

File: `fp-library/src/classes/ref_apply_second.rs`

Supertrait: `RefLift`

| Method             | Signature (simplified)                                              | Container params | Consumes?     | Could borrow?             | Notes                                                                                                         |
| ------------------ | ------------------------------------------------------------------- | ---------------- | ------------- | ------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `ref_apply_second` | `fn ref_apply_second(fa: Of<A>, fb: Of<B>) -> Of<B> where B: Clone` | `fa`, `fb`       | Both consumed | Inherits from `ref_lift2` | Default: `Self::ref_lift2(\|_: &A, b: &B\| b.clone(), fa, fb)`. Blanket-implemented for all `Brand: RefLift`. |

---

## 10. RefFoldable

File: `fp-library/src/classes/ref_foldable.rs`

All three methods have defaults in terms of each other (circular; implementor provides one).

| Method           | Signature (simplified)                                                                    | Container params | Consumes? | Could borrow? | Notes                                                                                                                                                                      |
| ---------------- | ----------------------------------------------------------------------------------------- | ---------------- | --------- | ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ref_fold_map`   | `fn ref_fold_map(func: impl Fn(&A) -> M, fa: Of<A>) -> M where A: Clone, M: Monoid`       | `fa`             | Yes       | **Yes**       | For Vec, impl iterates with `fa.iter()`. For Lazy, impl calls `func(fa.evaluate())`, and `evaluate(&self) -> &A` so `&fa` suffices. Default delegates to `ref_fold_right`. |
| `ref_fold_right` | `fn ref_fold_right(func: impl Fn(&A, B) -> B, initial: B, fa: Of<A>) -> B where A: Clone` | `fa`             | Yes       | **Yes**       | Default delegates to `ref_fold_map`. Direct impls iterate.                                                                                                                 |
| `ref_fold_left`  | `fn ref_fold_left(func: impl Fn(B, &A) -> B, initial: B, fa: Of<A>) -> B where A: Clone`  | `fa`             | Yes       | **Yes**       | Default delegates to `ref_fold_right` (via `ref_fold_map`).                                                                                                                |

**Key observation:** All three methods produce a plain value (not a container), so they never construct a new container. They only read elements. Borrowing is universally possible since `Lazy::evaluate()` takes `&self`.

---

## 11. RefFoldableWithIndex

File: `fp-library/src/classes/ref_foldable_with_index.rs`

Supertraits: `RefFoldable + WithIndex`

Same circular default pattern as RefFoldable.

| Method                      | Signature (simplified)                                                                                      | Container params | Consumes? | Could borrow? | Notes                                                                              |
| --------------------------- | ----------------------------------------------------------------------------------------------------------- | ---------------- | --------- | ------------- | ---------------------------------------------------------------------------------- |
| `ref_fold_map_with_index`   | `fn ref_fold_map_with_index(f: impl Fn(Index, &A) -> R, fa: Of<A>) -> R where A: Clone, R: Monoid`          | `fa`             | Yes       | **Yes**       | Same reasoning as `RefFoldable`. Default delegates to `ref_fold_right_with_index`. |
| `ref_fold_right_with_index` | `fn ref_fold_right_with_index(func: impl Fn(Index, &A, B) -> B, initial: B, fa: Of<A>) -> B where A: Clone` | `fa`             | Yes       | **Yes**       | Default delegates to `ref_fold_map_with_index`.                                    |
| `ref_fold_left_with_index`  | `fn ref_fold_left_with_index(func: impl Fn(B, Index, &A) -> B, initial: B, fa: Of<A>) -> B where A: Clone`  | `fa`             | Yes       | **Yes**       | Default delegates to `ref_fold_map_with_index`.                                    |

---

## 12. RefFunctorWithIndex

File: `fp-library/src/classes/ref_functor_with_index.rs`

Supertraits: `RefFunctor + WithIndex`

| Method               | Signature (simplified)                                                                      | Container params | Consumes? | Could borrow?   | Notes                                                                                                                                                                                   |
| -------------------- | ------------------------------------------------------------------------------------------- | ---------------- | --------- | --------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ref_map_with_index` | `fn ref_map_with_index(f: impl Fn(Index, &A) -> B, fa: Self::Of<'a, A>) -> Self::Of<'a, B>` | `fa`             | Yes       | Depends on type | Same as `ref_map`: Vec iterates (borrow OK), Lazy captures in closure (needs ownership). Required method, no default. Note: uses `Self::Of<'a, A>` directly rather than `Apply!` macro. |

---

## 13. RefFilterable

File: `fp-library/src/classes/ref_filterable.rs`

Supertraits: `RefFunctor + Compactable`

| Method              | Signature (simplified)                                                                    | Container params | Consumes? | Could borrow?   | Notes                                                                                                                   |
| ------------------- | ----------------------------------------------------------------------------------------- | ---------------- | --------- | --------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `ref_partition_map` | `fn ref_partition_map(func: impl Fn(&A) -> Result<O, E>, fa: Of<A>) -> (Of<E>, Of<O>)`    | `fa`             | Yes       | Depends on type | Default: `Self::separate(Self::ref_map(func, fa))`. Consumed by `ref_map`.                                              |
| `ref_partition`     | `fn ref_partition(func: impl Fn(&A) -> bool, fa: Of<A>) -> (Of<A>, Of<A>) where A: Clone` | `fa`             | Yes       | Depends on type | Default delegates to `ref_partition_map`. Returns same-type containers.                                                 |
| `ref_filter_map`    | `fn ref_filter_map(func: impl Fn(&A) -> Option<B>, fa: Of<A>) -> Of<B>`                   | `fa`             | Yes       | Depends on type | Default: `Self::compact(Self::ref_map(func, fa))`. Vec direct impl: `fa.iter().filter_map(func).collect()` (borrow OK). |
| `ref_filter`        | `fn ref_filter(func: impl Fn(&A) -> bool, fa: Of<A>) -> Of<A> where A: Clone`             | `fa`             | Yes       | Depends on type | Default delegates to `ref_filter_map`. Returns same-type container.                                                     |

**Note on filter/partition returning `Of<A>`:** These return the same container type with the same element type. Ownership could enable in-place `Vec::retain`-style optimization, but current impls all allocate new containers via `.collect()`, so borrowing would not regress performance.

---

## 14. RefFilterableWithIndex

File: `fp-library/src/classes/ref_filterable_with_index.rs`

Supertraits: `RefFilterable + RefFunctorWithIndex + WithIndex`

| Method                         | Signature (simplified)                                                                                      | Container params | Consumes? | Could borrow?   | Notes                                                                                       |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------- | ---------------- | --------- | --------------- | ------------------------------------------------------------------------------------------- |
| `ref_filter_map_with_index`    | `fn ref_filter_map_with_index(func: impl Fn(Index, &A) -> Option<B>, fa: Of<A>) -> Of<B>`                   | `fa`             | Yes       | Depends on type | Required method, no default. Vec impl: `fa.iter().enumerate().filter_map(...)` (borrow OK). |
| `ref_filter_with_index`        | `fn ref_filter_with_index(func: impl Fn(Index, &A) -> bool, fa: Of<A>) -> Of<A> where A: Clone`             | `fa`             | Yes       | Depends on type | Default delegates to `ref_filter_map_with_index`.                                           |
| `ref_partition_map_with_index` | `fn ref_partition_map_with_index(func: impl Fn(Index, &A) -> Result<O, E>, fa: Of<A>) -> (Of<E>, Of<O>)`    | `fa`             | Yes       | Depends on type | Default: `Self::separate(Self::ref_map_with_index(func, fa))`.                              |
| `ref_partition_with_index`     | `fn ref_partition_with_index(func: impl Fn(Index, &A) -> bool, fa: Of<A>) -> (Of<A>, Of<A>) where A: Clone` | `fa`             | Yes       | Depends on type | Default delegates to `ref_partition_map_with_index`.                                        |

---

## 15. RefTraversable

File: `fp-library/src/classes/ref_traversable.rs`

Supertraits: `RefFunctor + RefFoldable`

| Method         | Signature (simplified)                                                                                               | Container params | Consumes? | Could borrow?   | Notes                                                                                                             |
| -------------- | -------------------------------------------------------------------------------------------------------------------- | ---------------- | --------- | --------------- | ----------------------------------------------------------------------------------------------------------------- |
| `ref_traverse` | `fn ref_traverse(func: impl Fn(&A) -> F::Of<B>, ta: Of<A>) -> F::Of<Of<B>> where A: Clone, B: Clone, F: Applicative` | `ta`             | Yes       | Depends on type | Required method, no default. For Vec, impl iterates via fold (borrow OK). Lazy does not implement RefTraversable. |

---

## 16. RefTraversableWithIndex

File: `fp-library/src/classes/ref_traversable_with_index.rs`

Supertraits: `RefTraversable + RefFunctorWithIndex + WithIndex`

| Method                    | Signature (simplified)                                                                                                              | Container params | Consumes? | Could borrow?   | Notes                                                |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | ---------------- | --------- | --------------- | ---------------------------------------------------- |
| `ref_traverse_with_index` | `fn ref_traverse_with_index(f: impl Fn(Index, &A) -> M::Of<B>, ta: Of<A>) -> M::Of<Of<B>> where A: Clone, B: Clone, M: Applicative` | `ta`             | Yes       | Depends on type | Required method, no default. Same as `ref_traverse`. |

---

## 17. RefWitherable

File: `fp-library/src/classes/ref_witherable.rs`

Supertraits: `RefFilterable + RefTraversable`

| Method       | Signature (simplified)                                                                                           | Container params | Consumes? | Could borrow?   | Notes                                                                                                     |
| ------------ | ---------------------------------------------------------------------------------------------------------------- | ---------------- | --------- | --------------- | --------------------------------------------------------------------------------------------------------- |
| `ref_wilt`   | `fn ref_wilt(func: impl Fn(&A) -> M::Of<Result<O, E>>, ta: Of<A>) -> M::Of<(Of<E>, Of<O>)> where A, E, O: Clone` | `ta`             | Yes       | Depends on type | Default: maps `Self::separate` over `Self::ref_traverse(func, ta)`. Container consumed by `ref_traverse`. |
| `ref_wither` | `fn ref_wither(func: impl Fn(&A) -> M::Of<Option<B>>, ta: Of<A>) -> M::Of<Of<B>> where A, B: Clone`              | `ta`             | Yes       | Depends on type | Default: maps `Self::compact` over `Self::ref_traverse(func, ta)`. Container consumed by `ref_traverse`.  |

---

## Consolidated Summary

### All methods with container parameters

| Trait                   | Method                         | Params by value | Could borrow? | Blocking issue                                |
| ----------------------- | ------------------------------ | --------------- | ------------- | --------------------------------------------- |
| RefFunctor              | `ref_map`                      | `fa`            | No (Lazy)     | Lazy captures `fa` in closure                 |
| RefPointed              | `ref_pure`                     | (none)          | N/A           | N/A                                           |
| RefLift                 | `ref_lift2`                    | `fa`, `fb`      | No (Lazy)     | Lazy captures both in closure                 |
| RefSemiapplicative      | `ref_apply`                    | `ff`, `fa`      | No (Lazy)     | Lazy captures both in closure                 |
| RefSemimonad            | `ref_bind`                     | `fa`            | No (Lazy)     | Lazy needs owned `fa` for `evaluate()` borrow |
| RefApplicative          | (none)                         | N/A             | N/A           | Marker trait                                  |
| RefMonad                | (none)                         | N/A             | N/A           | Marker trait                                  |
| RefApplyFirst           | `ref_apply_first`              | `fa`, `fb`      | No            | Delegates to `ref_lift2`                      |
| RefApplySecond          | `ref_apply_second`             | `fa`, `fb`      | No            | Delegates to `ref_lift2`                      |
| RefFoldable             | `ref_fold_map`                 | `fa`            | **Yes**       | None                                          |
| RefFoldable             | `ref_fold_right`               | `fa`            | **Yes**       | None                                          |
| RefFoldable             | `ref_fold_left`                | `fa`            | **Yes**       | None                                          |
| RefFoldableWithIndex    | `ref_fold_map_with_index`      | `fa`            | **Yes**       | None                                          |
| RefFoldableWithIndex    | `ref_fold_right_with_index`    | `fa`            | **Yes**       | None                                          |
| RefFoldableWithIndex    | `ref_fold_left_with_index`     | `fa`            | **Yes**       | None                                          |
| RefFunctorWithIndex     | `ref_map_with_index`           | `fa`            | No (Lazy)     | Same as `ref_map`                             |
| RefFilterable           | `ref_partition_map`            | `fa`            | No            | Delegates to `ref_map`                        |
| RefFilterable           | `ref_partition`                | `fa`            | No            | Delegates to `ref_partition_map`              |
| RefFilterable           | `ref_filter_map`               | `fa`            | No            | Default delegates to `ref_map`                |
| RefFilterable           | `ref_filter`                   | `fa`            | No            | Delegates to `ref_filter_map`                 |
| RefFilterableWithIndex  | `ref_filter_map_with_index`    | `fa`            | No (Lazy)     | Vec could borrow; Lazy-style types block      |
| RefFilterableWithIndex  | `ref_filter_with_index`        | `fa`            | No            | Delegates to above                            |
| RefFilterableWithIndex  | `ref_partition_map_with_index` | `fa`            | No            | Delegates to `ref_map_with_index`             |
| RefFilterableWithIndex  | `ref_partition_with_index`     | `fa`            | No            | Delegates to above                            |
| RefTraversable          | `ref_traverse`                 | `ta`            | No (Lazy)     | Depends on impl                               |
| RefTraversableWithIndex | `ref_traverse_with_index`      | `ta`            | No (Lazy)     | Same                                          |
| RefWitherable           | `ref_wilt`                     | `ta`            | No            | Delegates to `ref_traverse`                   |
| RefWitherable           | `ref_wither`                   | `ta`            | No            | Delegates to `ref_traverse`                   |

### Free functions with container parameters

| Module        | Function       | Params by value                      | Could borrow?                              | Notes                         |
| ------------- | -------------- | ------------------------------------ | ------------------------------------------ | ----------------------------- |
| ref_semimonad | `ref_join`     | `mma`                                | No                                         | Passed to `ref_bind`          |
| ref_monad     | `ref_if_m`     | `cond`, `then_branch`, `else_branch` | `cond`: No. Branches: captured in closure. | `cond` consumed by `ref_bind` |
| ref_monad     | `ref_unless_m` | `cond`, `action`                     | `cond`: No.                                | Same pattern as `ref_if_m`    |

---

## Analysis: Two Categories of Container Usage

### Category 1: Pure reads (could accept `&Of<A>`) - 6 methods

The **RefFoldable** and **RefFoldableWithIndex** families are pure consumers that
produce a summary value, never a new container. All 6 methods in this category
could accept `&Of<A>` because:

- `Vec` implementations use `.iter()`, which borrows.
- `Lazy` implementations call `.evaluate()`, which takes `&self`.
- No new container of the same brand is constructed from the input.

### Category 2: Container transformations (ownership currently required) - 22 methods

All other traits produce a new `Of<B>` (or `Of<A>`) from the input `Of<A>`.
The reason they currently require ownership is the **Lazy** implementation pattern:

```rust
fn ref_map(func, fa) -> Of<B> {
    RcLazy::new(move || func(fa.evaluate()))
    //                      ^^ moved into closure
}
```

For `Vec`, `Option`, `Identity`, `Tuple1`, and `CatList`, the implementations
only iterate the input, so `&Of<A>` would work. The ownership requirement is
driven by `Lazy`'s need to capture the input container in a deferred closure.

### The Rc/Arc clone escape hatch

`Lazy` values are `Rc`-wrapped internally, so cloning is cheap. If the trait
accepted `&Of<A>`, the `Lazy` implementation could simply clone the `Rc` handle:

```rust
fn ref_map(func, fa: &Of<A>) -> Of<B> {
    let fa = fa.clone(); // cheap Rc clone
    RcLazy::new(move || func(fa.evaluate()))
}
```

This would work for all types:

- `Vec`: borrow directly, iterate.
- `Lazy`: clone the Rc handle, capture in closure.
- `Option`/`Identity`/`Tuple1`: borrow, pattern-match.

The tradeoff is an extra `Rc::clone()` for Lazy types, which is a pointer-width
reference count increment, essentially free.

### Methods returning `Of<A>` (same type as input)

The following methods return `Of<A>` where the input is also `Of<A>`:

- `ref_filter`, `ref_filter_with_index`
- `ref_partition`, `ref_partition_with_index`

These could theoretically benefit from in-place mutation when the container is
owned (e.g., `Vec::retain`). However, the current implementations all create new
containers via `.collect()`, so no in-place optimization is currently used.
Switching to `&Of<A>` would not regress current performance.

### Default implementation delegation chains

The default implementations form chains where ownership propagates:

**Functor chain (transforms):**

- `ref_partition_map` -> `ref_map` (ownership)
- `ref_filter_map` -> `ref_map` + `compact` (ownership)
- `ref_wilt` -> `ref_traverse` (ownership)
- `ref_wither` -> `ref_traverse` (ownership)
- `ref_apply_first` -> `ref_lift2` (ownership)
- `ref_apply_second` -> `ref_lift2` (ownership)

**Foldable chain (reads only):**

- `ref_fold_right` -> `ref_fold_map` (could be borrow)
- `ref_fold_left` -> `ref_fold_right` -> `ref_fold_map` (could be borrow)
- `ref_fold_right_with_index` -> `ref_fold_map_with_index` (could be borrow)
- `ref_fold_left_with_index` -> `ref_fold_map_with_index` (could be borrow)

If the root methods (`ref_map`, `ref_lift2`, `ref_traverse`) were changed to
accept borrows, the entire functor chain would follow. The foldable chain
can be changed independently since it has no dependency on the functor chain.

### Implementors per trait

Checked implementations to confirm the pattern:

| Trait          | Implementors                                                                                         |
| -------------- | ---------------------------------------------------------------------------------------------------- |
| RefFunctor     | `VecBrand`, `OptionBrand`, `CatListBrand`, `Tuple1Brand`, `IdentityBrand`, `LazyBrand<RcLazyConfig>` |
| RefFoldable    | `VecBrand`, `OptionBrand`, `CatListBrand`, `Tuple1Brand`, `IdentityBrand` (not Lazy)                 |
| RefFilterable  | `VecBrand`, `OptionBrand`, `CatListBrand`                                                            |
| RefTraversable | `VecBrand`, `OptionBrand`, `CatListBrand`, `Tuple1Brand`, `IdentityBrand`                            |

Note: `LazyBrand` implements `RefFunctor`, `RefLift`, `RefSemiapplicative`,
`RefSemimonad`, `RefFoldable`, `RefFoldableWithIndex`, and `RefFunctorWithIndex`,
but not `RefFilterable`, `RefTraversable`, `RefWitherable`, or their indexed variants.
