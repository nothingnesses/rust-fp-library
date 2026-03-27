# Lazy Hierarchy Brand Definitions: Analysis

Analysis of brand definitions in `fp-library/src/brands.rs` and their `impl_kind!` invocations across the type modules.

## Inventory

### Infallible Computation Brands

| Brand | Line in brands.rs | Kind Signature | Maps To |
|-------|-------------------|----------------|---------|
| `ThunkBrand` | 222 | `Of<'a, A: 'a>: 'a` | `Thunk<'a, A>` |
| `SendThunkBrand` | 215 | `Of<'a, A: 'a>: 'a` | `SendThunk<'a, A>` |
| `LazyBrand<Config>` | 106 | `Of<'a, A: 'a>: 'a` | `Lazy<'a, A, Config>` |
| `RcLazyBrand` (type alias) | 109 | (via `LazyBrand<RcLazyConfig>`) | `Lazy<'a, A, RcLazyConfig>` |
| `ArcLazyBrand` (type alias) | 112 | (via `LazyBrand<ArcLazyConfig>`) | `Lazy<'a, A, ArcLazyConfig>` |
| `CatListBrand` | 84 | `Of<'a, A: 'a>: 'a` | `CatList<A>` |

### Fallible Computation Brands

| Brand | Line in brands.rs | Kind Signature | Maps To |
|-------|-------------------|----------------|---------|
| `TryThunkBrand` | 257 | `Of<'a, E: 'a, A: 'a>: 'a` | `TryThunk<'a, A, E>` |
| `TryThunkErrAppliedBrand<E>` | 277 | `Of<'a, A: 'a>: 'a` | `TryThunk<'a, A, E>` |
| `TryThunkOkAppliedBrand<A>` | 295 | `Of<'a, E: 'a>: 'a` | `TryThunk<'a, A, E>` |
| `TrySendThunkBrand` | 253 | `Of<'a, E: 'a, A: 'a>: 'a` | `TrySendThunk<'a, A, E>` |
| `TryLazyBrand<E, Config>` | 242 | `Of<'a, A: 'a>: 'a` | `TryLazy<'a, A, E, Config>` |
| `RcTryLazyBrand<E>` (type alias) | 245 | (via `TryLazyBrand<E, RcLazyConfig>`) | `TryLazy<'a, A, E, RcLazyConfig>` |
| `ArcTryLazyBrand<E>` (type alias) | 248 | (via `TryLazyBrand<E, ArcLazyConfig>`) | `TryLazy<'a, A, E, ArcLazyConfig>` |

### Step Brands (MonadRec Infrastructure)

| Brand | Line in brands.rs | Kind Signature | Maps To |
|-------|-------------------|----------------|---------|
| `StepBrand` | 200 | `Of<A, B>` and `Of<'a, A: 'a, B: 'a>: 'a` | `Step<A, B>` |
| `StepDoneAppliedBrand<B>` | 204 | `Of<'a, A: 'a>: 'a` | `Step<A, DoneType>` |
| `StepLoopAppliedBrand<A>` | 208 | `Of<'a, B: 'a>: 'a` | `Step<LoopType, B>` |

## Analysis

### 1. Design Correctness

All brand definitions correctly represent their type constructors:

- **ThunkBrand / SendThunkBrand**: Both use `Of<'a, A: 'a>: 'a`, which correctly captures `Thunk<'a, A>` and `SendThunk<'a, A>`. The lifetime parameter `'a` flows through properly since both types wrap closures with `'a` lifetimes.

- **LazyBrand<Config>**: The `Config` parameter is a brand-level parameter (not a Kind parameter), which is the correct design. The Kind signature `Of<'a, A: 'a>: 'a` means `LazyBrand` acts as a unary type constructor parameterized over the value type, with the Config strategy baked into the brand. This mirrors PureScript's approach where effect system configuration is a phantom parameter on the type constructor, not a parameter of the applied type.

- **TryThunkBrand**: Uses `Of<'a, E: 'a, A: 'a>: 'a` with the parameter order `E, A` (error first, success second). The mapping `TryThunk<'a, A, E>` swaps the order, which is correct: the Kind parameters follow the `ResultBrand` convention (error first in the HKT, matching Haskell's `Either e a`), while the Rust struct uses the idiomatic `Result<A, E>` ordering. Documented at try_thunk.rs:1282-1285.

- **TryThunkErrAppliedBrand<E> / TryThunkOkAppliedBrand<A>**: These correctly partial-apply one parameter, requiring `'static` on the fixed parameter (documented at brands.rs:262-268, 282-288). The `'static` requirement is an inherent limitation of the Brand pattern, well-explained in the documentation.

- **TrySendThunkBrand**: Uses the same `Of<'a, E: 'a, A: 'a>: 'a` bifunctor signature as `TryThunkBrand`, correctly mapping to `TrySendThunk<'a, A, E>`.

- **TryLazyBrand<E, Config>**: Both `E` and `Config` are brand-level parameters, leaving only `A` as the Kind parameter. This is the correct design for a type that is functorial over the success value while the error type and configuration strategy are fixed.

- **CatListBrand**: Maps `Of<'a, A: 'a>: 'a` to `CatList<A>`. The `CatList` type itself has no lifetime parameter (it is `CatList<A>`, not `CatList<'a, A>`), so the `'a` in the Kind signature is effectively unused in the output type. This is correct; the Kind trait requires the lifetime-bounded signature, and `CatList<A>` trivially satisfies `: 'a` when `A: 'a`.

- **StepBrand**: Has two impl_kind invocations (step.rs:528-531 and step.rs:534-538), one without lifetimes (`Of<A, B>`) and one with (`Of<'a, A: 'a, B: 'a>: 'a`). This dual registration is correct and matches `ResultBrand`'s pattern (result.rs:38-57), enabling `Step` to participate in both lifetime-free and lifetime-bounded HKT contexts.

- **StepLoopAppliedBrand<LoopType> / StepDoneAppliedBrand<DoneType>**: Correctly partial-apply one type parameter with a `'static` bound, exactly paralleling `ResultErrAppliedBrand<E>` / `ResultOkAppliedBrand<T>`.

### 2. Naming Consistency

**Strengths:**

- The `{Type}Brand` pattern is used uniformly: `ThunkBrand`, `SendThunkBrand`, `CatListBrand`, `StepBrand`.
- Type aliases for convenience brands follow a clear pattern: `RcLazyBrand = LazyBrand<RcLazyConfig>`, `ArcLazyBrand = LazyBrand<ArcLazyConfig>`, `RcTryLazyBrand<E> = TryLazyBrand<E, RcLazyConfig>`, etc.
- The `Try` prefix is consistently applied to fallible variants: `TryThunkBrand`, `TrySendThunkBrand`, `TryLazyBrand`.
- Applied brands use the `{Type}{FixedParam}AppliedBrand` pattern consistently: `TryThunkErrAppliedBrand`, `TryThunkOkAppliedBrand`, `StepDoneAppliedBrand`, `StepLoopAppliedBrand`.

**Observation on Send prefix positioning:**

- `SendThunkBrand` puts `Send` first.
- `TrySendThunkBrand` puts `Try` before `Send`.

This is consistent in the sense that `Try` is always the outermost prefix (matching how `TrySendThunk` wraps `SendThunk`), while `Send` describes the thread-safety property. The naming mirrors the type names themselves (`SendThunk`, `TrySendThunk`), which is the right approach.

### 3. Completeness

**Intentionally absent brands (correctly omitted):**

- **No `TrampolineBrand`**: `Trampoline<A>` is `Free<ThunkBrand, A>` and requires `'static`, which is incompatible with the `Kind` trait's `'a` lifetime parameter. Documented at brands.rs:219-220.

- **No `TryTrampolineBrand`**: Same reasoning; `TryTrampoline<A, E>` is `Trampoline<Result<A, E>>`, inheriting the `'static` constraint.

- **No `FreeBrand`**: `Free<F, A>` requires `F: Functor + 'static` and `A: 'static`, making it incompatible with lifetime-polymorphic HKT. This is correct.

- **No `TrySendThunkErrAppliedBrand` / `TrySendThunkOkAppliedBrand`**: Explicitly documented at brands.rs:271-275 and 292-293. Since `SendThunk` cannot implement HKT traits (the trait signatures lack `Send` bounds on closure parameters), partially-applied brands would serve no purpose. This is a well-reasoned design decision.

**All necessary brands are present.** Every type in the lazy hierarchy that can participate in HKT has a corresponding brand, and every type that cannot has a documented rationale for omission.

### 4. HKT Correctness

All Kind mappings are type-theoretically sound:

- **Unary type constructors** (`Thunk`, `SendThunk`, `Lazy`, `CatList`) all use `Of<'a, A: 'a>: 'a`, which is the standard Kind for `* -> *` with lifetime tracking.

- **Binary type constructors** (`TryThunk`, `TrySendThunk`, `Step`) use `Of<'a, E: 'a, A: 'a>: 'a` for the bifunctor brand, plus partially-applied brands with `Of<'a, A: 'a>: 'a` for functorial use on a single channel.

- **The E/A parameter swap** in `TryThunkBrand` and `TrySendThunkBrand` (Kind parameters `E, A` map to Rust struct `TryThunk<'a, A, E>`) correctly follows the Haskell convention where the last type parameter is the one that varies in functorial operations. This is consistent with `ResultBrand` and `StepBrand`.

- **`'static` bounds on applied brand parameters** (`E: 'static` in `TryThunkErrAppliedBrand<E>`, `A: 'static` in `TryThunkOkAppliedBrand<A>`, `LoopType: 'static`, `DoneType: 'static`) are necessary and correctly applied. The Brand pattern requires baked-in type parameters to outlive all possible `'a` values.

### 5. Consistency Across the Hierarchy

The lazy brands follow the exact same structural patterns as the non-lazy brands in the codebase:

| Pattern | Lazy Example | Non-Lazy Example |
|---------|-------------|-----------------|
| Simple brand | `ThunkBrand` | `OptionBrand`, `VecBrand` |
| Config-parameterized brand | `LazyBrand<Config>` | `FnBrand<PtrBrand>` |
| Convenience type alias | `RcLazyBrand = LazyBrand<RcLazyConfig>` | `RcFnBrand = FnBrand<RcBrand>` |
| Bifunctor brand | `TryThunkBrand` | `ResultBrand`, `StepBrand` |
| ErrApplied brand | `TryThunkErrAppliedBrand<E>` | `ResultErrAppliedBrand<E>` |
| OkApplied brand | `TryThunkOkAppliedBrand<A>` | `ResultOkAppliedBrand<T>` |

All derive the same set of traits: `Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash` (for structs) or a subset for type aliases (which inherit from their underlying struct).

One minor inconsistency: `BifunctorFirstAppliedBrand` and `BifunctorSecondAppliedBrand` (lines 63, 80) derive only `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash` (no `Default`), while most brand structs derive `Default` as well. The lazy brands with `PhantomData` parameters like `TryThunkErrAppliedBrand<E>` (line 277) also omit `Default`, which is consistent since `Default` for a brand with `PhantomData<E>` would require `E: Default`, which is not desirable for marker types. The simple brands (`ThunkBrand`, `SendThunkBrand`, `CatListBrand`, etc.) all include `Default`, which makes sense since they are zero-sized with no type parameters.

### 6. Issues

**No significant issues found.** The brand definitions are well-designed and correctly implemented.

Minor observations (not bugs):

1. **Alphabetical ordering**: The brands in `brands.rs` are sorted alphabetically, which aids discoverability. The lazy brands are interleaved with non-lazy brands rather than grouped together. This is a deliberate organizational choice consistent with the file's role as a flat registry of all brands.

2. **Documentation quality varies slightly**: `ThunkBrand` (line 217-222) has a brief note about `Trampoline`. `TryThunkErrAppliedBrand` (line 259-277) has extensive documentation about `'static` bounds and the absence of `TrySendThunk` counterparts. `CatListBrand` (line 82-84) has only a one-liner. The simpler brands arguably need less documentation, but there is room for more consistent depth; for instance, `SendThunkBrand` (line 210-215) could note why it exists as a brand despite not implementing standard HKT traits.

3. **`LazyBrand` documentation** (line 98-106) references `LazyConfig` but does not mention that `Lazy` implements `RefFunctor` / `SendRefFunctor` rather than `Functor`. This context is in the module-level docs of `lazy.rs` but a brief note in the brand doc would aid users browsing `brands.rs` directly.

### 7. Documentation Assessment

**Well-documented aspects:**

- The `'static` limitation on applied brands is explained thoroughly with the same boilerplate in `TryThunkErrAppliedBrand` (lines 262-268), `TryThunkOkAppliedBrand` (lines 282-288), and `TryLazyBrand` (lines 233-240). This repetition is appropriate since each brand is independently discoverable.

- The absence of `TrySendThunkErrAppliedBrand` / `TrySendThunkOkAppliedBrand` is explicitly documented with rationale (lines 271-275, 292-293).

- `ThunkBrand` clarifies that it is for `Thunk`, not `Trampoline` (line 219-220).

- `TrySendThunkBrand` (lines 250-253) notes it is for bifunctor use, enabling fallible deferred computation across thread boundaries.

**Areas for improvement:**

- `CatListBrand` (line 82-84) could note that `CatList` is the backbone of `Free` monad evaluation.
- `StepBrand` (line 198-200) could mention its role in `MonadRec` and tail-recursive computation.
- `SendThunkBrand` (line 210-215) could note the HKT trait limitations (no `Functor` impl due to `Send` bounds) as `TrySendThunkBrand` does, or at least cross-reference the `SendThunk` module docs.

## Summary

The lazy hierarchy brand definitions are well-designed, correctly implemented, and consistent with the rest of the codebase. The naming is systematic, the Kind mappings are type-theoretically correct, and the completeness is justified (every omission is documented). The main area for improvement is minor: a few brands could benefit from slightly richer documentation to match the standard set by the `TryThunk`-related brands.
