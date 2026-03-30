# Analysis: `brands.rs` (Lazy Hierarchy Brands)

**File:** `fp-library/src/brands.rs`
**Scope:** Only the brands related to the lazy hierarchy.

## Brand Inventory

### Thunk Family

| Brand                        | Type Mapped                      | HKT Support                                    |
| ---------------------------- | -------------------------------- | ---------------------------------------------- |
| `ThunkBrand`                 | `Thunk<'a, A>`                   | Full (Functor, Monad, Comonad, Foldable, etc.) |
| `SendThunkBrand`             | `SendThunk<'a, A>`               | Minimal (Foldable, FoldableWithIndex only)     |
| `TryThunkBrand`              | `TryThunk<'a, A, E>` (bifunctor) | Bifunctor, Bifoldable                          |
| `TryThunkErrAppliedBrand<E>` | `TryThunk<'a, A, E>` (E fixed)   | Full tower (E: 'static)                        |
| `TryThunkOkAppliedBrand<A>`  | `TryThunk<'a, A, E>` (A fixed)   | Full tower (A: 'static)                        |

### Lazy Family

| Brand                        | Type Mapped                 | HKT Support                                        |
| ---------------------------- | --------------------------- | -------------------------------------------------- |
| `LazyBrand<Config>`          | `Lazy<'a, A, Config>`       | RefFunctor / SendRefFunctor, Foldable              |
| `RcLazyBrand` (alias)        | `RcLazy<'a, A>`             | RefFunctor, Foldable                               |
| `ArcLazyBrand` (alias)       | `ArcLazy<'a, A>`            | SendRefFunctor, Foldable                           |
| `TryLazyBrand<E, Config>`    | `TryLazy<'a, A, E, Config>` | RefFunctor / SendRefFunctor, Foldable (E: 'static) |
| `RcTryLazyBrand<E>` (alias)  | `RcTryLazy<'a, A, E>`       | RefFunctor, Foldable                               |
| `ArcTryLazyBrand<E>` (alias) | `ArcTryLazy<'a, A, E>`      | SendRefFunctor, Foldable                           |

### Free Monad Infrastructure

| Brand          | Type Mapped  | HKT Support                                        |
| -------------- | ------------ | -------------------------------------------------- |
| `CatListBrand` | `CatList<A>` | Full (Functor, Monad, Foldable, Traversable, etc.) |

### Notable Absences

| Missing Brand        | Type                     | Reason                                       |
| -------------------- | ------------------------ | -------------------------------------------- |
| `TrampolineBrand`    | `Trampoline<A>`          | `'static` constraint conflicts with HKT `'a` |
| `TryTrampolineBrand` | `TryTrampoline<A, E>`    | Same                                         |
| `FreeBrand<F>`       | `Free<F, A>`             | Same                                         |
| `TrySendThunkBrand`  | `TrySendThunk<'a, A, E>` | HKT closure signatures lack `Send`           |

## Assessment

### Correct decisions

1. **Type aliases for common configs.** `RcLazyBrand = LazyBrand<RcLazyConfig>` and similar aliases reduce verbosity at call sites.
2. **Partial application brands for bifunctors.** `TryThunkErrAppliedBrand<E>` and `TryThunkOkAppliedBrand<A>` enable full HKT trait towers on each channel independently.
3. **`'static` constraints documented.** Each brand with a `'static` requirement has a doc comment explaining the limitation.

### Issues

#### 1. `'static` constraints on partially-applied brands

`TryThunkErrAppliedBrand<E>` requires `E: 'static`, `TryThunkOkAppliedBrand<A>` requires `A: 'static`, and `TryLazyBrand<E, Config>` requires `E: 'static`. These are inherent to the brand pattern (the type parameter baked into the brand must outlive all possible `'a`), but they exclude borrowed types from HKT abstractions.

**Impact:** Moderate. Limits generic programming with borrowed error/success types.

#### 2. No bifunctor brand for `TryLazy`

`TryThunk` has `TryThunkBrand` (bifunctor), but `TryLazy` has no corresponding bifunctor brand. This means `TryLazy` cannot participate in generic bifunctor code, despite having a `bimap` inherent method.

**Impact:** Low-moderate.

#### 3. Asymmetric HKT support across the hierarchy

The hierarchy has widely varying levels of HKT support:

- `Thunk`: Full HKT (Functor through Comonad).
- `SendThunk`: Minimal HKT (Foldable only).
- `Lazy`: Partial HKT (RefFunctor/SendRefFunctor + Foldable only).
- `Trampoline`/`Free`: No HKT.

This means generic FP code can only target `ThunkBrand` for full composability. The other types require concrete method calls. This is a fundamental consequence of Rust's type system constraints, not a design flaw, but it limits the utility of the HKT abstraction for the lazy hierarchy.

**Impact:** Moderate. The asymmetry is inherent but worth documenting as a holistic limitation.

#### 4. `LazyBrand<Config>` implements different traits depending on `Config`

`LazyBrand<RcLazyConfig>` implements `RefFunctor` while `LazyBrand<ArcLazyConfig>` implements `SendRefFunctor`. Generic code over `LazyBrand<Config>` cannot assume either. A user writing `F: LazyBrand<Config>` in a generic context gets no functor capability unless they also bound on a specific config.

**Impact:** Low. In practice, users work with concrete configs.
