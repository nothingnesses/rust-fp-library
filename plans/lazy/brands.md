# Lazy Hierarchy Brand Definitions: Analysis

## Overview

The file `fp-library/src/brands.rs` centralizes all brand (HKT witness) types as leaf nodes in the dependency graph. The lazy evaluation hierarchy accounts for 14 of the file's brand definitions (including type aliases), plus 2 config trait implementations defined in `fp-library/src/types/lazy.rs`.

---

## 1. Inventory of Lazy-Related Brands

### Brands with full `impl_kind!` (HKT-enabled)

| Brand | Kind signature | Defined type | Type class depth |
|-------|---------------|--------------|-----------------|
| `ThunkBrand` | `type Of<'a, A: 'a>: 'a = Thunk<'a, A>` | Infallible deferred computation | Full: Functor, Pointed, Lift, Semiapplicative, Semimonad, MonadRec, Evaluable, Foldable, FunctorWithIndex, FoldableWithIndex |
| `SendThunkBrand` | `type Of<'a, A: 'a>: 'a = SendThunk<'a, A>` | Thread-safe deferred computation | Minimal: Foldable, FoldableWithIndex only |
| `LazyBrand<Config>` | `type Of<'a, A: 'a>: 'a = Lazy<'a, A, Config>` | Memoized computation | Partial: RefFunctor (Rc), SendRefFunctor (Arc), Foldable, FoldableWithIndex |
| `TryLazyBrand<E, Config>` | `type Of<'a, A: 'a>: 'a = TryLazy<'a, A, E, Config>` | Memoized fallible computation | Partial: RefFunctor (Rc, E: Clone), SendRefFunctor (Arc, E: Clone+Send+Sync), Foldable |
| `TryThunkBrand` | `type Of<'a, E: 'a, A: 'a>: 'a = TryThunk<'a, A, E>` | Bifunctor brand (unapplied) | Bifunctor, Bifoldable, Bitraversable |
| `TryThunkErrAppliedBrand<E>` | `type Of<'a, A: 'a>: 'a = TryThunk<'a, A, E>` | Error-fixed functor over Ok | Full: Functor, Pointed, Lift, Semiapplicative, Semimonad, MonadRec, Foldable, FunctorWithIndex, FoldableWithIndex |
| `TryThunkOkAppliedBrand<A>` | `type Of<'a, E: 'a>: 'a = TryThunk<'a, A, E>` | Ok-fixed functor over Err | Full: Functor, Pointed, Lift, Semiapplicative, Semimonad, MonadRec, Foldable, FunctorWithIndex, FoldableWithIndex |
| `TrySendThunkBrand` | `type Of<'a, E: 'a, A: 'a>: 'a = TrySendThunk<'a, A, E>` | Bifunctor brand | Bifunctor brand only (no type class impls found) |
| `CatListBrand` | `type Of<'a, A: 'a>: 'a = CatList<A>` | Catenable list (Free backbone) | Full: Functor, Pointed, Lift, Semiapplicative, Semimonad, Alt, Plus, Foldable, Traversable, Compactable, Filterable, Witherable, MonadRec, Par*, WithIndex variants |
| `StepBrand` | `type Of<A, B> = Step<A, B>` + `type Of<'a, A: 'a, B: 'a>: 'a = Step<A, B>` | Loop/Done signal for MonadRec | Bifunctor, Bifoldable, Bitraversable |
| `StepLoopAppliedBrand<L>` | `type Of<'a, B: 'a>: 'a = Step<L, B>` | Loop-fixed, functor over Done | Functor, Pointed, Lift, Foldable, Traversable |
| `StepDoneAppliedBrand<D>` | `type Of<'a, A: 'a>: 'a = Step<A, D>` | Done-fixed, functor over Loop | Functor, Pointed, Lift, Foldable, Traversable |

### Type aliases for convenience

| Alias | Expands to |
|-------|-----------|
| `RcLazyBrand` | `LazyBrand<RcLazyConfig>` |
| `ArcLazyBrand` | `LazyBrand<ArcLazyConfig>` |
| `RcTryLazyBrand<E>` | `TryLazyBrand<E, RcLazyConfig>` |
| `ArcTryLazyBrand<E>` | `TryLazyBrand<E, ArcLazyConfig>` |

### Types with NO brand (intentionally)

| Type | Why no brand |
|------|-------------|
| `Trampoline<A>` | Requires `A: 'static` (wraps `Free<ThunkBrand, A>`). The HKT Kind trait introduces `'a`, so `A` must outlive all `'a`, requiring `'static`. Since `Trampoline` already requires `'static`, it could theoretically have a brand with `type Of<A> = Trampoline<A>` (no lifetime), but the library's type class traits uniformly use `type Of<'a, A: 'a>: 'a`, making a `'static`-only brand useless for polymorphic code. |
| `TryTrampoline<A, E>` | Same `'static` constraint as `Trampoline`. Inherits the limitation from `Free`. |
| `Free<F, A>` | Requires `F: Evaluable + 'static` and `A: 'static`. Uses `Box<dyn Any>` for type erasure, which demands `'static`. Cannot participate in the lifetime-polymorphic Kind system. |

---

## 2. Design Coherence

### Strengths

**Consistent bifunctor pattern.** The `TryThunk`/`TrySendThunk` types follow the same bifunctor brand pattern as `Result` and `Step`: a full bifunctor brand plus two partially-applied brands (one fixing each type parameter). `TryThunkBrand` parallels `ResultBrand` and `StepBrand`; `TryThunkErrAppliedBrand<E>` parallels `ResultErrAppliedBrand<E>` and `StepLoopAppliedBrand<L>`.

**Principled Send limitation.** The doc comments on `SendThunkBrand` and `TryThunkErrAppliedBrand` clearly explain why `SendThunk` cannot implement Functor/Monad: the HKT trait signatures accept `impl Fn`/`impl FnOnce` without a `Send` bound, so closures passed to `map`/`bind` cannot be guaranteed thread-safe. The library explicitly documents why there is no `TrySendThunkErrAppliedBrand`. This is an honest acknowledgment of a real constraint rather than a gap.

**Config-parameterized unification.** `LazyBrand<Config>` with `RcLazyBrand`/`ArcLazyBrand` as aliases avoids duplicating brand definitions and all their type class impls. A single `impl<Config: LazyConfig> Foldable for LazyBrand<Config>` covers both Rc and Arc variants.

**Clean `'static` documentation.** The `TryLazyBrand` and `TryThunkErrAppliedBrand` doc comments include a thorough explanation of why baked-in type parameters require `'static`, tracing the reasoning through the Kind trait's lifetime introduction.

### Concerns

**`TrySendThunkBrand` is a hollow brand.** It has an `impl_kind!` (bifunctor signature) but zero type class implementations. No `Bifunctor`, no `Bifoldable`, no partially-applied brands. By contrast, `TryThunkBrand` has `Bifunctor`, `Bifoldable`, and `Bitraversable`. This creates a structural asymmetry: `TrySendThunkBrand` exists purely as a marker with no functional value. It should either gain type class impls or be removed if it serves no purpose.

**CatList as "lazy hierarchy" member is debatable.** `CatListBrand` is an internal data structure for `Free` monad evaluation. It has an extraordinarily rich type class surface (Functor through Witherable, MonadRec, parallel variants). While it supports the lazy hierarchy mechanically, it is more of a general-purpose persistent list than a "lazy evaluation" type. Its brand is well-designed; the question is purely about conceptual grouping.

---

## 3. HKT Coverage Analysis

### Full HKT support

- **ThunkBrand**: The flagship lazy brand. Full monadic stack (Functor through MonadRec), plus Evaluable (critical for `Free`), Foldable, and indexed variants. This is the "workhorse" for HKT-polymorphic lazy code.
- **TryThunkErrAppliedBrand<E>**: Full monadic stack mirroring `ThunkBrand`, but for fallible computations. The `E: 'static` constraint is unfortunate but inherent.
- **TryThunkOkAppliedBrand<A>**: Full monadic stack for the error channel. Symmetric with `TryThunkErrAppliedBrand`.
- **CatListBrand**: Full type class coverage, including `Traversable` and parallel variants.

### Partial HKT support

- **LazyBrand<Config>**: Has `RefFunctor`/`SendRefFunctor` (not `Functor`!) and `Foldable`. Cannot implement `Functor` because evaluation returns `&A` (a reference), not an owned `A`. The `ref_map` pattern (mapping `&A -> B`) is the correct abstraction. This partial support is well-motivated.
- **TryLazyBrand<E, Config>**: Same pattern as `LazyBrand`, with `RefFunctor`/`SendRefFunctor` and `Foldable`. Missing `FoldableWithIndex` (which `LazyBrand` has). This may be an oversight or may reflect a design choice about fallible containers.
- **SendThunkBrand**: Has a Kind impl but only `Foldable` and `FoldableWithIndex`. No Functor because of the Send constraint problem. This is the correct minimal surface.

### No HKT support

- **Trampoline**, **TryTrampoline**, **Free**: Intentionally excluded due to `'static` constraints. The reasoning is sound: the Kind system requires lifetime polymorphism that `'static`-only types cannot provide.

### Assessment

The HKT coverage is well-calibrated to each type's capabilities. The library does not force HKT support where it does not fit. The hierarchy of `ThunkBrand` (full HKT) > `LazyBrand` (partial, ref-based) > `SendThunkBrand` (foldable only) > `Trampoline` (none) reflects a genuine gradient of capability, not arbitrary omissions.

One potential gap: `TryLazyBrand` lacks `FoldableWithIndex` while `LazyBrand` has it, and `TryThunkErrAppliedBrand` has it. This looks like an omission rather than a deliberate design choice.

---

## 4. Config Type Design

### Current approach

```
trait LazyConfig: 'static {
    type PointerBrand: RefCountedPointer;
    type Lazy<'a, A: 'a>: Clone;
    type Thunk<'a, A: 'a>: ?Sized;
    fn lazy_new<'a, A: 'a>(f: Box<Self::Thunk<'a, A>>) -> Self::Lazy<'a, A>;
    fn evaluate<'a, 'b, A: 'a>(lazy: &'b Self::Lazy<'a, A>) -> &'b A;
}

trait TryLazyConfig: LazyConfig {
    type TryLazy<'a, A: 'a, E: 'a>: Clone;
    type TryThunk<'a, A: 'a, E: 'a>: ?Sized;
    fn try_lazy_new<'a, A: 'a, E: 'a>(f: Box<Self::TryThunk<'a, A, E>>) -> Self::TryLazy<'a, A, E>;
    fn try_evaluate<'a, 'b, A: 'a, E: 'a>(lazy: &'b Self::TryLazy<'a, A, E>) -> Result<&'b A, &'b E>;
}
```

Two concrete impls: `RcLazyConfig` (Rc + LazyCell) and `ArcLazyConfig` (Arc + LazyLock).

### Strengths

1. **Single brand, dual behavior.** `LazyBrand<Config>` avoids duplicating brand definitions. Generic impls like `impl<Config: LazyConfig> Foldable for LazyBrand<Config>` work across both Rc and Arc variants.

2. **Extensibility.** The doc comments explicitly invite third-party configs (e.g., `parking_lot`-based locks, async-aware cells). The trait surface is minimal (2 associated types + 2 methods for infallible; 2+2 more for fallible).

3. **PointerBrand linkage.** `LazyConfig::PointerBrand` connects the config to the pointer hierarchy, allowing generic code to recover the pointer brand without hard-coding it.

4. **Clean separation.** `TryLazyConfig` extends `LazyConfig` rather than duplicating it. A config can implement only `LazyConfig` if fallible memoization is not needed.

### Weaknesses

1. **Config traits live in `types/lazy.rs`, not `brands.rs` or `classes/`.** The `brands.rs` file imports `LazyConfig` and `TryLazyConfig` from `types`, creating a dependency from brands (supposed leaf nodes) into types. This is a mild violation of the stated dependency ordering (brands -> classes -> types). In practice, the config traits are trait definitions (closer to "classes" than "types"), so they could be moved to `classes/` to restore the intended layering.

2. **`'static` bound on `LazyConfig`.** The trait itself requires `'static`, which is needed because brand type parameters must outlive all `'a` in the Kind system. However, this means custom configs cannot capture non-`'static` data (not that they would need to, since configs are zero-sized marker types, but the constraint is more restrictive than strictly necessary for the trait definition itself).

3. **The `Box<Self::Thunk<'a, A>>` parameter.** Both `lazy_new` and `try_lazy_new` accept `Box<Self::Thunk<...>>`. Since `Thunk` is `?Sized` (it is `dyn FnOnce() -> A + 'a`), boxing is required. This means every lazy cell construction allocates. This is inherent to the design (you cannot store an unsized type without indirection), but it is worth noting that the allocation is unavoidable at the trait level.

### Alternatives considered

**Alternative 1: Separate `RcLazyBrand` and `ArcLazyBrand` as independent structs.** This would eliminate the config machinery but double the type class implementation burden. Every trait impl for `LazyBrand` would need to be written twice. The current approach is clearly better.

**Alternative 2: Parameterize `LazyBrand` directly over a pointer brand (e.g., `LazyBrand<PtrBrand: RefCountedPointer>`) instead of a config trait.** This would simplify the trait hierarchy but lose the ability to customize the lazy cell type (e.g., using `LazyLock` vs `LazyCell` vs a custom cell). The config approach is more general.

**Alternative 3: Move config traits to `classes/`.** Since `LazyConfig` and `TryLazyConfig` are trait definitions that describe behavior (not concrete type implementations), they fit the `classes/` module's role. This would restore the clean dependency ordering: brands depend only on classes, not on types. The concrete impls (`RcLazyConfig`, `ArcLazyConfig`) would stay in `types/lazy.rs`.

---

## 5. Naming Consistency

### Conventions observed

- **`XBrand`** for simple types: `ThunkBrand`, `SendThunkBrand`, `CatListBrand`, `StepBrand`. Consistent.
- **`XBrand<Config>`** for config-parameterized types: `LazyBrand<Config>`, `TryLazyBrand<E, Config>`. Consistent.
- **Type aliases** use the pointer prefix: `RcLazyBrand`, `ArcLazyBrand`, `RcTryLazyBrand<E>`, `ArcTryLazyBrand<E>`. Consistent with `RcFnBrand`/`ArcFnBrand`.
- **Partially-applied bifunctor brands** follow `XYAppliedBrand<T>` pattern: `TryThunkErrAppliedBrand<E>`, `TryThunkOkAppliedBrand<A>`, `StepLoopAppliedBrand<A>`, `StepDoneAppliedBrand<B>`. Consistent with `ResultErrAppliedBrand<E>`, `ResultOkAppliedBrand<T>`.
- **`Try` prefix** for fallible variants: `TryThunkBrand`, `TrySendThunkBrand`, `TryLazyBrand`. Consistent.
- **`Send` prefix** for thread-safe variants: `SendThunkBrand`, `TrySendThunkBrand`. Consistent with `SendDeferrable`, `SendRefFunctor`.

### Minor observations

- The `Send` prefix comes after `Try` for `TrySendThunkBrand`, reading as "try, then send". This parallels how `TrySendThunk` is named. The alternative `SendTryThunkBrand` would read as "send, then try". The current ordering is consistent: `Try` always comes first when both prefixes are present.
- `StepBrand` has two `impl_kind!` invocations (one for `type Of<A, B>` without lifetimes, one for `type Of<'a, A: 'a, B: 'a>: 'a` with lifetimes). This dual registration lets `Step` participate in both lifetime-free and lifetime-aware HKT contexts.

---

## 6. Issues and Limitations

### `TrySendThunkBrand` has no implementations

`TrySendThunkBrand` is defined with an `impl_kind!` but has zero type class implementations. No `Bifunctor`, no `Bifoldable`, no partially-applied error/ok brands. This makes it a dead brand. By contrast:

- `TryThunkBrand` has `Bifunctor`, `Bifoldable`, `Bitraversable`, plus the two partially-applied brands with full monadic stacks.
- `TrySendThunkBrand` has nothing.

If the brand exists only for future use or symmetry, it should be documented as such. If it will never gain impls (because of the Send constraint), it should be removed.

### `'static` on baked-in type parameters

Both `TryThunkErrAppliedBrand<E>` and `TryThunkOkAppliedBrand<A>` require their type parameter to be `'static`. This is correctly documented but has practical consequences: you cannot use `TryThunkErrAppliedBrand<&str>` in HKT-polymorphic code. The same applies to `TryLazyBrand<E, Config>` where `E: 'static`. This is an inherent limitation of the Brand pattern, not a fixable issue.

### `brands.rs` depends on `types` module

The import `crate::types::{ArcLazyConfig, LazyConfig, RcLazyConfig, TryLazyConfig}` creates a dependency from `brands` (supposed to be leaf nodes) into `types`. The config traits are more "class-like" than "type-like" and could be relocated to `classes/` to maintain the stated dependency ordering.

### Missing `FoldableWithIndex` for `TryLazyBrand`

`LazyBrand<Config>` implements both `Foldable` and `FoldableWithIndex`. `TryLazyBrand<E, Config>` implements `Foldable` but not `FoldableWithIndex`. Since both are single-element containers, the indexed variant should be trivially implementable for `TryLazyBrand` as well.

### No Traversable for `ThunkBrand` or `LazyBrand`

`ThunkBrand` implements Foldable but not Traversable. For a single-element container, `Traversable` should be straightforward (it would be equivalent to mapping and wrapping). The same applies to `LazyBrand`. `CatListBrand` does implement Traversable, showing the pattern is established in the codebase.

---

## 7. Suggestions

### Short-term

1. **Add `FoldableWithIndex` for `TryLazyBrand<E, Config>`.** This is a trivial addition that restores parity with `LazyBrand`.

2. **Either implement type classes for `TrySendThunkBrand` or remove it.** If the bifunctor impls are blocked by the Send constraint (same as why `SendThunkBrand` lacks Functor), document that and consider removing the brand. If bifunctor impls are feasible (since `bimap` takes two separate closures, the Send constraint may be enforceable differently for bifunctors), implement them.

3. **Consider moving `LazyConfig`/`TryLazyConfig` trait definitions to `classes/`.** Keep the concrete impls (`RcLazyConfig`, `ArcLazyConfig`) in `types/lazy.rs`, but move the trait definitions to `classes/lazy_config.rs` to restore the brands -> classes -> types dependency ordering.

### Medium-term

4. **Add `Traversable` for `ThunkBrand`.** A single-element Traversable is well-defined and useful for generic programming (e.g., `sequence` on a Thunk inside an applicative).

5. **Consider a `Foldable` impl for `TrySendThunkBrand` (via partially-applied brands).** Even if Functor is blocked, Foldable only requires consuming the value, which does not need closures stored in the thunk. If `TrySendThunkErrAppliedBrand<E>` could at least implement `Foldable`, it would provide some utility.

### Long-term

6. **Document the brand hierarchy visually.** A diagram showing which brands exist, which Kind signatures they implement, and which type classes they support would help newcomers navigate the 14 lazy-related brands. This analysis could serve as the basis for such documentation.
