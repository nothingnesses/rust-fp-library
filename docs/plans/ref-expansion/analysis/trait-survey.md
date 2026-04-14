# Ref Trait Expansion: Trait Survey

This document surveys the seven by-value traits that lack Ref counterparts, documenting their definitions, concrete implementations, proposed Ref designs, and open questions.

## Existing Ref pattern summary

Before examining each trait, here is the pattern established by the existing Ref traits:

- **RefFunctor**: `ref_map(func: impl Fn(&A) -> B, fa: &F<A>) -> F<B>`. Container by reference, closure receives `&A`, returns owned `F<B>`.
- **RefFoldable**: `ref_fold_map(func: impl Fn(&A) -> M, fa: &F<A>) -> M`. Same pattern for folds.
- **RefTraversable**: `ref_traverse(func: impl Fn(&A) -> G<B>, ta: &F<A>) -> G<F<B>>`. Same pattern for traversals.
- **RefFilterable**: `ref_filter_map(func: impl Fn(&A) -> Option<B>, fa: &F<A>) -> F<B>`. Same pattern for filtering.

The dispatch system (in `classes/dispatch/`) uses marker types `Val` and `Ref` to route unified free functions (like `map`) to either the by-value or by-reference trait based on the closure's argument type. Each dispatch module defines a trait with two blanket impls.

---

## 1. Bifunctor

### Trait definition

- **File**: `fp-library/src/classes/bifunctor.rs`
- **Kind**: `type Of<'a, A: 'a, B: 'a>: 'a;` (two type parameters)
- **Supertraits**: None (inherits only from the Kind constraint)

**Required methods:**

```rust
fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
    f: impl Fn(A) -> B + 'a,
    g: impl Fn(C) -> D + 'a,
    p: Apply!(<Self as Kind!(...)>::Of<'a, A, C>),
) -> Apply!(<Self as Kind!(...)>::Of<'a, B, D>);
```

**Provided methods:** None. `bimap` is the only method.

The module also defines:

- `BifunctorFirstAppliedBrand<Brand, A>` and `BifunctorSecondAppliedBrand<Brand, B>` that implement `Functor` by delegating to `bimap` with `identity` on one side.

### Concrete implementations

| Brand              | Type                | File                    |
| ------------------ | ------------------- | ----------------------- |
| `ResultBrand`      | `Result<B, A>`      | `types/result.rs`       |
| `PairBrand`        | `Pair<A, B>`        | `types/pair.rs`         |
| `Tuple2Brand`      | `(A, B)`            | `types/tuple_2.rs`      |
| `ControlFlowBrand` | `ControlFlow<A, B>` | `types/control_flow.rs` |
| `TryThunkBrand`    | `TryThunk<A, B>`    | `types/try_thunk.rs`    |

### Proposed Ref variant

```rust
pub trait RefBifunctor {
    fn ref_bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
        f: impl Fn(&A) -> B + 'a,
        g: impl Fn(&C) -> D + 'a,
        p: &Apply!(<Self as Kind!(...)>::Of<'a, A, C>),
    ) -> Apply!(<Self as Kind!(...)>::Of<'a, B, D>);
}
```

### Issues, limitations, and open questions

1. **Bifunctorial passthrough requires `Clone` on the untouched field.** When mapping only one side (e.g., `bimap(f, identity)` on `Result::Ok(x)`), the value on the non-mapped side must be moved into the new container. With `ref_bimap`, the source is borrowed, so the untouched side must be cloned. This mirrors the existing pattern in `RefFunctor` implementations for Result/Pair, where the fixed error type must be `Clone`. However, none of the bifunctorial types (`ResultBrand`, `PairBrand`, etc.) currently implement `RefFunctor` at all, so this is new territory.

2. **Dispatch integration.** The existing `BifunctorFirstAppliedBrand` and `BifunctorSecondAppliedBrand` derive `Functor` from `Bifunctor`. A `RefBifunctor` should similarly allow deriving `RefFunctor` for these applied brands, so that the unified `map` free function can dispatch to `ref_bimap` when the closure takes `&A`.

3. **TryThunkBrand is lazy.** `TryThunk` is a lazy type that wraps a closure. Calling `ref_bimap` on `&TryThunk` means the thunk has not been evaluated yet. The implementation would need to clone the inner `Rc`/function pointer and create a new `TryThunk` that applies `f`/`g` after evaluation. This is structurally similar to how `RefFunctor` works for `LazyBrand`.

4. **Pair and Tuple2 always have both fields.** Unlike `Result` (which is either `Ok` or `Err`), `Pair` and `(A, B)` always contain both values. `ref_bimap` would need to apply both `f` and `g` to references of the two fields. Since `f: Fn(&A) -> B` and `g: Fn(&C) -> D` produce owned output from references, no additional `Clone` bound on `A` or `C` is strictly required for the direct `ref_bimap` call. The closures handle the conversion.

**Recommendation:** Implement `RefBifunctor` with the signature above. No `Clone` bounds on `A` or `C` are needed for `ref_bimap` itself, since the closures produce owned values from references. `Clone` bounds may be needed for derived `RefFunctor` impls on applied brands (where one side uses `identity` and the other side's value must be cloned from the borrow).

---

## 2. Bifoldable

### Trait definition

- **File**: `fp-library/src/classes/bifoldable.rs`
- **Kind**: `type Of<'a, A: 'a, B: 'a>: 'a;`
- **Supertraits**: None (inherits only from the Kind constraint)

**Methods (all provided with mutual defaults):**

```rust
fn bi_fold_right<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
    f: impl Fn(A, C) -> C + 'a,
    g: impl Fn(B, C) -> C + 'a,
    z: C,
    p: Apply!(<Self as Kind!(...)>::Of<'a, A, B>),
) -> C;

fn bi_fold_left<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
    f: impl Fn(C, A) -> C + 'a,
    g: impl Fn(C, B) -> C + 'a,
    z: C,
    p: Apply!(<Self as Kind!(...)>::Of<'a, A, B>),
) -> C;

fn bi_fold_map<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, M: Monoid + 'a>(
    f: impl Fn(A) -> M + 'a,
    g: impl Fn(B) -> M + 'a,
    p: Apply!(<Self as Kind!(...)>::Of<'a, A, B>),
) -> M;
```

Minimal implementation: define either `bi_fold_map` or `bi_fold_right`.

### Concrete implementations

| Brand              | Type                | File                    |
| ------------------ | ------------------- | ----------------------- |
| `ResultBrand`      | `Result<B, A>`      | `types/result.rs`       |
| `PairBrand`        | `Pair<A, B>`        | `types/pair.rs`         |
| `Tuple2Brand`      | `(A, B)`            | `types/tuple_2.rs`      |
| `ControlFlowBrand` | `ControlFlow<A, B>` | `types/control_flow.rs` |
| `TryThunkBrand`    | `TryThunk<A, B>`    | `types/try_thunk.rs`    |

### Proposed Ref variant

```rust
pub trait RefBifoldable {
    fn ref_bi_fold_right<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
        f: impl Fn(&A, C) -> C + 'a,
        g: impl Fn(&B, C) -> C + 'a,
        z: C,
        p: &Apply!(<Self as Kind!(...)>::Of<'a, A, B>),
    ) -> C;

    fn ref_bi_fold_left<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
        f: impl Fn(C, &A) -> C + 'a,
        g: impl Fn(C, &B) -> C + 'a,
        z: C,
        p: &Apply!(<Self as Kind!(...)>::Of<'a, A, B>),
    ) -> C;

    fn ref_bi_fold_map<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, M: Monoid + 'a>(
        f: impl Fn(&A) -> M + 'a,
        g: impl Fn(&B) -> M + 'a,
        p: &Apply!(<Self as Kind!(...)>::Of<'a, A, B>),
    ) -> M;
}
```

### Issues, limitations, and open questions

1. **Default implementation circular dependency.** The by-value trait has `bi_fold_map` default from `bi_fold_right` and vice versa. The Ref variant should mirror this pattern: `ref_bi_fold_map` defaults from `ref_bi_fold_right`, and `ref_bi_fold_left` defaults from `ref_bi_fold_right`. The `Endofunction` machinery in the defaults clones `A` and `B` values (hence the `Clone` bounds), which works fine since `&A` can be cloned when `A: Clone`.

2. **Clone bounds remain necessary.** The `Endofunction`-based defaults for `ref_bi_fold_right` (from `ref_bi_fold_map`) and `ref_bi_fold_left` (from `ref_bi_fold_right`) need to capture element values in closures. Since elements are accessed by reference, `Clone` is needed to move owned copies into the closures. This matches the existing `RefFoldable` pattern.

3. **Dispatch integration.** The same Val/Ref dispatch pattern used for `fold_right`/`fold_left`/`fold_map` could be extended to `bi_fold_right`/`bi_fold_left`/`bi_fold_map`, routing `Fn(&A, C) -> C` to `ref_bi_fold_right` and `Fn(A, C) -> C` to `bi_fold_right`.

**Recommendation:** Straightforward to implement following the `RefFoldable` pattern. The `Clone` bounds on `A` and `B` are already present in the by-value trait (needed for the `Endofunction` defaults) and remain appropriate for the Ref variant.

---

## 3. Bitraversable

### Trait definition

- **File**: `fp-library/src/classes/bitraversable.rs`
- **Kind**: `type Of<'a, A: 'a, B: 'a>: 'a;`
- **Supertraits**: `Bifunctor + Bifoldable`

**Required method:**

```rust
fn bi_traverse<'a, A: 'a + Clone, B: 'a + Clone, C: 'a + Clone, D: 'a + Clone, F: Applicative>(
    f: impl Fn(A) -> F<C> + 'a,
    g: impl Fn(B) -> F<D> + 'a,
    p: Apply!(<Self as Kind!(...)>::Of<'a, A, B>),
) -> F<Apply!(<Self as Kind!(...)>::Of<'a, C, D>)>;
```

**Provided method:**

```rust
fn bi_sequence<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
    ta: Apply!(<Self as Kind!(...)>::Of<'a, F<A>, F<B>>),
) -> F<Apply!(<Self as Kind!(...)>::Of<'a, A, B>)>;
```

Free functions: `bi_traverse`, `bi_sequence`, `traverse_left`, `traverse_right`, `bi_for`, `for_left`, `for_right`.

### Concrete implementations

| Brand              | Type                | File                    |
| ------------------ | ------------------- | ----------------------- |
| `ResultBrand`      | `Result<B, A>`      | `types/result.rs`       |
| `PairBrand`        | `Pair<A, B>`        | `types/pair.rs`         |
| `Tuple2Brand`      | `(A, B)`            | `types/tuple_2.rs`      |
| `ControlFlowBrand` | `ControlFlow<A, B>` | `types/control_flow.rs` |

Note: `TryThunkBrand` implements `Bifunctor` and `Bifoldable` but NOT `Bitraversable`.

### Proposed Ref variant

```rust
pub trait RefBitraversable: RefBifunctor + RefBifoldable {
    fn ref_bi_traverse<'a, FnBrand, A: 'a + Clone, B: 'a + Clone, C: 'a + Clone, D: 'a + Clone, F: Applicative>(
        f: impl Fn(&A) -> F<C> + 'a,
        g: impl Fn(&B) -> F<D> + 'a,
        p: &Apply!(<Self as Kind!(...)>::Of<'a, A, B>),
    ) -> F<Apply!(<Self as Kind!(...)>::Of<'a, C, D>)>;
}
```

The `ref_bi_sequence` default would not change structurally from `bi_sequence` since it calls `bi_traverse(identity, identity, ta)` on containers already holding `F<A>` values, not applying a closure to `A`.

### Issues, limitations, and open questions

1. **Supertrait cascade.** `RefBitraversable` must require `RefBifunctor + RefBifoldable`, which means all three Ref bi-traits must be implemented together. This is a larger surface area than the unary Ref traits but mirrors the by-value hierarchy.

2. **FnBrand parameter.** The existing `RefTraversable` takes a `FnBrand` parameter (for `LiftFn`-based cloneable closures used in the applicative machinery), while the by-value `Bitraversable` does not. The Ref variant should include `FnBrand` if the implementation needs `LiftFn` for the applicative accumulation, matching `RefTraversable`.

3. **ref_bi_sequence semantics.** `bi_sequence` takes ownership of the container (which holds `F<A>` and `F<B>` values). A `ref_bi_sequence` would borrow the container, but the inner `F<A>` and `F<B>` values need to be cloned out to construct the result. This is viable since `F<A>: Clone` is already bounded in the by-value version.

4. **Free function variants.** The by-value trait has 7 free functions (`bi_traverse`, `bi_sequence`, `traverse_left`, `traverse_right`, `bi_for`, `for_left`, `for_right`). The Ref variant would need corresponding `ref_bi_traverse`, `ref_bi_sequence`, etc. This is a significant API surface.

**Recommendation:** Implement after `RefBifunctor` and `RefBifoldable` are in place. Include `FnBrand` in the trait signature following `RefTraversable`. Start with just `ref_bi_traverse` as required method and `ref_bi_sequence` as provided. Add ref variants of the convenience free functions as needed.

---

## 4. Compactable

### Trait definition

- **File**: `fp-library/src/classes/compactable.rs`
- **Kind**: `type Of<'a, A: 'a>: 'a;` (single type parameter)
- **Supertraits**: None (inherits only from the Kind constraint)

**Required methods:**

```rust
fn compact<'a, A: 'a>(
    fa: Apply!(<Self as Kind!(...)>::Of<'a, Option<A>>),
) -> Apply!(<Self as Kind!(...)>::Of<'a, A>);

fn separate<'a, E: 'a, O: 'a>(
    fa: Apply!(<Self as Kind!(...)>::Of<'a, Result<O, E>>),
) -> (
    Apply!(<Self as Kind!(...)>::Of<'a, E>),
    Apply!(<Self as Kind!(...)>::Of<'a, O>),
);
```

### Concrete implementations

| Brand          | Type         | File                |
| -------------- | ------------ | ------------------- |
| `OptionBrand`  | `Option<A>`  | `types/option.rs`   |
| `VecBrand`     | `Vec<A>`     | `types/vec.rs`      |
| `CatListBrand` | `CatList<A>` | `types/cat_list.rs` |

### Proposed Ref variant

```rust
pub trait RefCompactable {
    fn ref_compact<'a, A: 'a + Clone>(
        fa: &Apply!(<Self as Kind!(...)>::Of<'a, Option<A>>),
    ) -> Apply!(<Self as Kind!(...)>::Of<'a, A>);

    fn ref_separate<'a, E: 'a + Clone, O: 'a + Clone>(
        fa: &Apply!(<Self as Kind!(...)>::Of<'a, Result<O, E>>),
    ) -> (
        Apply!(<Self as Kind!(...)>::Of<'a, E>),
        Apply!(<Self as Kind!(...)>::Of<'a, O>),
    );
}
```

### Issues, limitations, and open questions

1. **No user-supplied closures.** Unlike every other Ref trait, `compact` and `separate` take no closures. The "Ref" part only applies to the container parameter (`&fa` instead of `fa`). The element values (`Option<A>` and `Result<O, E>`) must be cloned out of the borrowed container to construct the output, requiring `A: Clone`, `E: Clone`, and `O: Clone`. This is a different cost model: the by-value versions consume the input and can move elements, while the Ref versions must clone.

2. **Dispatch is not closure-based.** The existing dispatch system routes based on closure argument types (`Fn(A) -> B` vs `Fn(&A) -> B`). Since `compact` and `separate` have no closures, dispatch cannot use the same mechanism. Options:
   - a. Separate free functions: `ref_compact` and `ref_separate` (no dispatch unification).
   - b. Overload on the container argument type: detect `&F<Option<A>>` vs `F<Option<A>>`. This would require a different dispatch mechanism.
   - c. Skip Ref variants entirely and rely on users calling `.clone()` before passing to the by-value version.

3. **RefFilterable already uses Compactable.** The existing `RefFilterable` trait has `RefFunctor + Compactable` as supertraits and derives its defaults by calling `Self::compact(Self::ref_map(...))` and `Self::separate(Self::ref_map(...))`. So `RefFilterable` uses `Compactable` (by-value compact/separate) on the result of `ref_map`, which produces an owned container. This means `RefCompactable` may not actually be needed for `RefFilterable` to work. The question is whether standalone `ref_compact`/`ref_separate` on borrowed containers is useful on its own.

**Recommendation:** The value of `RefCompactable` is lower than other Ref traits because there are no closures to benefit from `&A` access and the implementation necessarily clones all retained elements. Consider deferring this trait or providing it only as convenience functions (not a trait) that clone and delegate to `Compactable`. If implemented as a trait, use option (a): separate free functions without dispatch unification.

---

## 5. Alt

### Trait definition

- **File**: `fp-library/src/classes/alt.rs`
- **Kind**: `type Of<'a, A: 'a>: 'a;`
- **Supertraits**: `Functor`

**Required method:**

```rust
fn alt<'a, A: 'a>(
    fa1: Apply!(<Self as Kind!(...)>::Of<'a, A>),
    fa2: Apply!(<Self as Kind!(...)>::Of<'a, A>),
) -> Apply!(<Self as Kind!(...)>::Of<'a, A>);
```

### Concrete implementations

| Brand          | Type         | File                |
| -------------- | ------------ | ------------------- |
| `OptionBrand`  | `Option<A>`  | `types/option.rs`   |
| `VecBrand`     | `Vec<A>`     | `types/vec.rs`      |
| `CatListBrand` | `CatList<A>` | `types/cat_list.rs` |

### Proposed Ref variant

```rust
pub trait RefAlt: RefFunctor {
    fn ref_alt<'a, A: 'a + Clone>(
        fa1: &Apply!(<Self as Kind!(...)>::Of<'a, A>),
        fa2: &Apply!(<Self as Kind!(...)>::Of<'a, A>),
    ) -> Apply!(<Self as Kind!(...)>::Of<'a, A>);
}
```

### Issues, limitations, and open questions

1. **No user-supplied closures.** Like `Compactable`, `alt` takes no closures. Both container parameters would be borrowed, and elements must be cloned to construct the output. The trait provides clone-and-combine semantics.

2. **Dispatch.** Same problem as Compactable: no closure to dispatch on. Options are the same: separate `ref_alt` free function, or some container-type-based dispatch. Since `alt` takes two containers of the same type, an overloaded `alt` that accepts `(&F<A>, &F<A>)` vs `(F<A>, F<A>)` might work via a dispatch trait on tuples or via two marker parameters.

3. **Semantic question: should one or both arguments be borrowed?** Mixed signatures are possible:
   - `ref_alt(&F<A>, &F<A>) -> F<A>` (both borrowed, most flexible for callers).
   - `ref_alt(&F<A>, F<A>) -> F<A>` (first borrowed, second consumed; useful for "try this, else use that" where the fallback is already owned).
     The simplest approach is both borrowed, which covers all cases at the cost of cloning.

4. **Efficiency for Vec.** `ref_alt` on `&Vec<A>` would clone both vectors and concatenate. This is strictly worse than the by-value version which can move elements. For `Option`, `ref_alt` on `&Option<A>` returns a clone of whichever is `Some`, which is reasonable.

**Recommendation:** Lower priority. The lack of closures means the primary benefit of Ref traits (avoiding consumption of borrowed containers) comes at a clone cost. Implement with both arguments borrowed. Use a separate `ref_alt` free function without attempting dispatch unification.

---

## 6. Extend

### Trait definition

- **File**: `fp-library/src/classes/extend.rs`
- **Kind**: `type Of<'a, A: 'a>: 'a;`
- **Supertraits**: `Functor`

**Required method:**

```rust
fn extend<'a, A: 'a + Clone, B: 'a>(
    f: impl Fn(Apply!(<Self as Kind!(...)>::Of<'a, A>)) -> B + 'a,
    wa: Apply!(<Self as Kind!(...)>::Of<'a, A>),
) -> Apply!(<Self as Kind!(...)>::Of<'a, B>);
```

**Provided methods:**

```rust
fn duplicate<'a, A: 'a + Clone>(wa: F<A>) -> F<F<A>>;
fn extend_flipped<'a, A: 'a + Clone, B: 'a>(wa: F<A>, f: impl Fn(F<A>) -> B) -> F<B>;
fn compose_co_kleisli<'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
    f: impl Fn(F<A>) -> B, g: impl Fn(F<B>) -> C, wa: F<A>,
) -> C;
fn compose_co_kleisli_flipped<'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
    f: impl Fn(F<B>) -> C, g: impl Fn(F<A>) -> B, wa: F<A>,
) -> C;
```

### Concrete implementations

| Brand           | Type          | File                |
| --------------- | ------------- | ------------------- |
| `IdentityBrand` | `Identity<A>` | `types/identity.rs` |
| `ThunkBrand`    | `Thunk<A>`    | `types/thunk.rs`    |
| `VecBrand`      | `Vec<A>`      | `types/vec.rs`      |
| `CatListBrand`  | `CatList<A>`  | `types/cat_list.rs` |

### Proposed Ref variant

The closure in `extend` receives the _whole container_ `F<A>`, not just an element `A`. This creates a fundamental design question for the Ref variant.

**Option A: Closure receives `&F<A>` (borrowed container):**

```rust
pub trait RefExtend: RefFunctor {
    fn ref_extend<'a, A: 'a + Clone, B: 'a>(
        f: impl Fn(&Apply!(<Self as Kind!(...)>::Of<'a, A>)) -> B + 'a,
        wa: &Apply!(<Self as Kind!(...)>::Of<'a, A>),
    ) -> Apply!(<Self as Kind!(...)>::Of<'a, B>);
}
```

**Option B: Container borrowed but closure receives owned sub-containers:**

```rust
pub trait RefExtend: RefFunctor {
    fn ref_extend<'a, A: 'a + Clone, B: 'a>(
        f: impl Fn(Apply!(<Self as Kind!(...)>::Of<'a, A>)) -> B + 'a,
        wa: &Apply!(<Self as Kind!(...)>::Of<'a, A>),
    ) -> Apply!(<Self as Kind!(...)>::Of<'a, B>);
}
```

### Issues, limitations, and open questions

1. **The closure's argument type determines the Ref semantics.** In `Functor`, the closure receives an element `A`. In `Extend`, the closure receives the _whole container_ `F<A>`. The Ref pattern for Functor changes `Fn(A) -> B` to `Fn(&A) -> B`. For Extend, the analogous change is `Fn(F<A>) -> B` to `Fn(&F<A>) -> B`. But this changes what "by reference" means: the closure sees a borrowed container, not a borrowed element.

2. **Vec's extend produces sub-containers.** `VecBrand::extend` applies `f` to every suffix of the input vector. Each suffix is an owned `Vec<A>`. With `ref_extend` Option A, `f` would receive `&Vec<A>` (a slice-like view of each suffix). With Option B, `f` still receives owned `Vec<A>` sub-containers, but the top-level `wa` is borrowed. Option B is more natural for Vec because constructing suffixes already requires allocation.

3. **Identity and Thunk have trivial extend.** `IdentityBrand::extend(f, wa) = Identity(f(wa))`. The Ref variant would be `Identity(f(&wa))`, which is straightforward with Option A. For `ThunkBrand`, extend evaluates the thunk to get an `A` and constructs a new `Thunk` containing `f(Thunk(A))`. The Ref variant with Option A would mean `f` receives `&Thunk<A>`, which the closure can evaluate.

4. **Dispatch.** `Fn(F<A>) -> B` vs `Fn(&F<A>) -> B` can be distinguished by the same dispatch mechanism used for `Functor`. The marker types `Val`/`Ref` would apply to the closure's argument type.

5. **duplicate semantics.** `duplicate(wa) = extend(identity, wa)`, producing `F<F<A>>`. The Ref variant `ref_duplicate(&wa) = ref_extend(|w| w.clone(), &wa)` would produce `F<F<A>>` by cloning each sub-context. For Vec, this means cloning each suffix, which is expensive but well-defined.

**Recommendation:** Use Option A (closure receives `&F<A>`). This is consistent with the RefFunctor pattern where the "Ref" prefix means the closure receives a reference. The container `wa` is also borrowed. Implementors for Vec would construct suffix slices and pass them as `&Vec<A>`. This allows dispatch unification where `Fn(&F<A>) -> B` routes to `ref_extend` and `Fn(F<A>) -> B` routes to `extend`.

---

## 7. Extract

### Trait definition

- **File**: `fp-library/src/classes/extract.rs`
- **Kind**: `type Of<'a, A: 'a>: 'a;`
- **Supertraits**: None (inherits only from the Kind constraint)

**Required method:**

```rust
fn extract<'a, A: 'a>(
    fa: Apply!(<Self as Kind!(...)>::Of<'a, A>),
) -> A;
```

### Concrete implementations

| Brand           | Type          | File                |
| --------------- | ------------- | ------------------- |
| `IdentityBrand` | `Identity<A>` | `types/identity.rs` |
| `ThunkBrand`    | `Thunk<A>`    | `types/thunk.rs`    |

### Proposed Ref variant

**Option A: Return `&A` (true by-reference extraction):**

```rust
pub trait RefExtract {
    fn ref_extract<'a, A: 'a>(
        fa: &Apply!(<Self as Kind!(...)>::Of<'a, A>),
    ) -> &A;
}
```

**Option B: Return owned `A` from borrowed container (requires `Clone`):**

```rust
pub trait RefExtract {
    fn ref_extract<'a, A: 'a + Clone>(
        fa: &Apply!(<Self as Kind!(...)>::Of<'a, A>),
    ) -> A;
}
```

### Issues, limitations, and open questions

1. **Return type: `&A` vs `A`.** This is the central design question. Option A (`&A`) is the "honest" Ref pattern, returning a borrow. Option B (owned `A`) requires `Clone` and is just a convenience wrapper for `extract(fa.clone())`.

2. **Lazy types cannot implement by-value Extract but could implement Option A.** The documentation for `Extract` explicitly notes that `Lazy` cannot implement `Extract` because "forcing it returns `&A`, not owned `A`". Ironically, `Lazy` is the one type that _could_ implement `RefExtract` with Option A (returning `&A`), since `Lazy::evaluate()` returns `&A`. But `LazyBrand` does not currently implement `Extract` (by-value), so `RefExtract` would be its only extraction trait.

3. **Identity implements Option A trivially.** `Identity(a)` can return `&a` via `&identity.0`.

4. **Thunk cannot implement Option A.** `Thunk` evaluates a closure and returns the result by value. It has no cached `&A` to return. `Thunk` could implement Option B by evaluating and returning the owned result.

5. **Lifetime issues with Option A.** Returning `&A` tied to the lifetime of `&fa` is straightforward for `Identity` (the value lives inside the struct). For `Lazy`, the value is behind `Rc`/`Arc` and `evaluate()` returns a reference valid for the cell's lifetime. But the return lifetime in Option A must be tied to the input borrow, which may not match the `Rc`'s actual lifetime.

6. **Dispatch.** Extract has no closures, so dispatch cannot use the closure-based mechanism. A separate `ref_extract` free function is the simplest approach.

7. **Comonad hierarchy.** `Extract` is paired with `Extend` to form `Comonad` (conceptually). If `RefExtend` is added, `RefExtract` may be needed for a `RefComonad` hierarchy. But this is a future consideration.

**Recommendation:** This trait has the most complex design trade-offs. Two approaches:

- **If the goal is a unified extraction that works for both `Identity` and `Lazy`:** Use Option A (`&A` return). `Identity` and `Lazy` can implement it. `Thunk` cannot (it has no cached value to reference). This would give `LazyBrand` an extraction capability it currently lacks entirely.

- **If the goal is a simple borrow-friendly `extract`:** Use Option B (owned `A` with `Clone`). This is just convenience and does not enable new capabilities.

Given that `Extract` currently has only two implementors (`IdentityBrand`, `ThunkBrand`), and `ThunkBrand` cannot implement Option A, the value of `RefExtract` is limited. Consider deferring this trait unless `LazyBrand` extraction is a priority.

---

## Summary table

| Trait         | Implementors                                    | Has closures                   | Ref variant complexity      | Dispatch viable        | Priority |
| ------------- | ----------------------------------------------- | ------------------------------ | --------------------------- | ---------------------- | -------- |
| Bifunctor     | 5 (Result, Pair, Tuple2, ControlFlow, TryThunk) | Yes (`f`, `g`)                 | Medium                      | Yes (closure arg type) | High     |
| Bifoldable    | 5 (same as Bifunctor)                           | Yes (`f`, `g`)                 | Medium                      | Yes (closure arg type) | High     |
| Bitraversable | 4 (no TryThunk)                                 | Yes (`f`, `g`)                 | High (supertrait cascade)   | Yes (closure arg type) | Medium   |
| Compactable   | 3 (Option, Vec, CatList)                        | No                             | Low (but limited value)     | No (no closures)       | Low      |
| Alt           | 3 (Option, Vec, CatList)                        | No                             | Low (but limited value)     | No (no closures)       | Low      |
| Extend        | 4 (Identity, Thunk, Vec, CatList)               | Yes (receives whole container) | High (semantic question)    | Yes (closure arg type) | Medium   |
| Extract       | 2 (Identity, Thunk)                             | No                             | High (return type question) | No (no closures)       | Low      |

## Recommended implementation order

1. **RefBifunctor** - Enables RefFunctor for applied bifunctor brands; high value, clear design.
2. **RefBifoldable** - Follows naturally; enables RefFoldable for applied brands.
3. **RefBitraversable** - Depends on items 1 and 2; larger surface area but follows established patterns.
4. **RefExtend** - Independent of bi-traits; Option A (closure receives `&F<A>`) is the recommended approach.
5. **RefCompactable** - Low value without closures; consider deferring or implementing as convenience functions only.
6. **RefAlt** - Low value without closures; similar considerations as RefCompactable.
7. **RefExtract** - Defer unless LazyBrand extraction is a near-term priority. If implemented, use Option A (`&A` return) and accept that only Identity and Lazy can implement it.
