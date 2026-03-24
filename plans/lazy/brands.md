# Lazy-Evaluation Brands: Analysis

## Scope

This analysis covers the lazy-evaluation-related brands in `fp-library/src/brands.rs`:

- `ThunkBrand`
- `SendThunkBrand`
- `TryThunkBrand`, `TryThunkErrAppliedBrand<E>`, `TryThunkOkAppliedBrand<A>`
- `TrySendThunkBrand`
- `LazyBrand<Config>`, `RcLazyBrand`, `ArcLazyBrand`
- `TryLazyBrand<E, Config>`, `RcTryLazyBrand<E>`, `ArcTryLazyBrand<E>`
- `CatListBrand`

## 1. Overall Design Assessment

The brand hierarchy is well-structured and reflects a principled decomposition along three orthogonal axes:

1. **Fallibility:** infallible (`Thunk`, `Lazy`) vs. fallible (`TryThunk`, `TryLazy`).
2. **Thread safety:** `!Send` (`Thunk`, `RcLazy`) vs. `Send` (`SendThunk`, `ArcLazy`).
3. **Memoization:** non-memoized (`Thunk`, `SendThunk`) vs. memoized (`Lazy`, `TryLazy`).

The separation between brands (zero-sized marker types in `brands.rs`) and concrete types (in `types/`) is clean. Brands import nothing from `types/`; the `impl_kind!` invocations live alongside the concrete types. This respects the dependency ordering: brands -> classes -> types.

## 2. `impl_kind!` Mappings

All mappings are correct:

| Brand | `impl_kind!` mapping | Location |
|-------|---------------------|----------|
| `ThunkBrand` | `Of<'a, A: 'a>: 'a = Thunk<'a, A>` | `types/thunk.rs:369` |
| `SendThunkBrand` | `Of<'a, A: 'a>: 'a = SendThunk<'a, A>` | `types/send_thunk.rs:276` |
| `TryThunkBrand` | `Of<'a, E: 'a, A: 'a>: 'a = TryThunk<'a, A, E>` | `types/try_thunk.rs:1189` |
| `TryThunkErrAppliedBrand<E>` | `impl<E: 'static> Of<'a, A: 'a>: 'a = TryThunk<'a, A, E>` | `types/try_thunk.rs:718` |
| `TryThunkOkAppliedBrand<A>` | `impl<A: 'static> Of<'a, E: 'a>: 'a = TryThunk<'a, A, E>` | `types/try_thunk.rs:1436` |
| `TrySendThunkBrand` | `Of<'a, E: 'a, A: 'a>: 'a = TrySendThunk<'a, A, E>` | `types/try_send_thunk.rs:590` |
| `LazyBrand<Config>` | `impl<Config: LazyConfig> Of<'a, A: 'a>: 'a = Lazy<'a, A, Config>` | `types/lazy.rs:933` |
| `TryLazyBrand<E, Config>` | `impl<E: 'static, Config: TryLazyConfig> Of<'a, A: 'a>: 'a = TryLazy<'a, A, E, Config>` | `types/try_lazy.rs:860` |
| `CatListBrand` | `Of<'a, A: 'a>: 'a = CatList<A>` | `types/cat_list.rs:221` |

Note on `TryThunkBrand`: the `Of` parameter ordering is `E, A` (error first, then success), matching the `ResultBrand` convention. The concrete type `TryThunk<'a, A, E>` has the parameters in the opposite order (`A, E`). This is intentional and documented, but could surprise users who expect the HKT parameter order to match the struct definition.

## 3. Issues and Inconsistencies

### 3.1 Missing partially-applied brands for `TrySendThunk`

`TryThunk` has three brands: `TryThunkBrand` (bifunctor), `TryThunkErrAppliedBrand<E>` (functor over `Ok`), and `TryThunkOkAppliedBrand<A>` (functor over `Err`). `TrySendThunk` only has `TrySendThunkBrand` (bifunctor), with no partially-applied variants.

This is not necessarily a bug, since `TrySendThunk` cannot implement standard HKT traits (because trait signatures do not require `Send` on closures), so the partially-applied brands would have no trait impls to attach to. The bifunctor brand exists solely for the `impl_kind!` mapping. This is consistent, but the asymmetry should be documented explicitly.

**Verdict:** Acceptable; document the rationale.

### 3.2 `LazyBrand<Config>` has an unconstrained type parameter

```rust
pub struct LazyBrand<Config>(PhantomData<Config>);
```

The `Config` parameter has no trait bound at the definition site, but `impl_kind!` constrains it to `LazyConfig`. This means you can write `LazyBrand<i32>`, which is a valid type but has no `Kind` implementation. The same applies to `TryLazyBrand<E, Config>`.

In contrast, `FnBrand<PtrBrand: RefCountedPointer>` constrains its parameter at the definition site. This inconsistency means that some brands enforce their parameter constraints structurally while others rely on the downstream `impl_kind!` to catch misuse.

**Recommendation:** Add `Config: LazyConfig` bound to `LazyBrand` and `Config: TryLazyConfig` bound to `TryLazyBrand` at the struct definition. This would:
- Make invalid brand constructions a compile error at the type level.
- Be consistent with `FnBrand`.
- Make the doc links to `LazyConfig`/`TryLazyConfig` visible in the brand's rustdoc.

**Counterargument:** Rust's orphan rules and derive macro expansion sometimes conflict with trait bounds on struct definitions. If adding the bound causes problems downstream, the current design is acceptable as a pragmatic choice. But the inconsistency with `FnBrand` should be resolved one way or the other.

### 3.3 No brands for `Trampoline`, `TryTrampoline`, or `Free`

`Trampoline<A>`, `TryTrampoline<A, E>`, and `Free<F, A>` have no brands. This is correct, since all three require `'static` types and cannot implement the library's HKT traits (which require lifetime polymorphism via `Of<'a, A: 'a>`). The documentation on `ThunkBrand` explicitly notes this: "This is for `Thunk<'a, A>`, NOT for `Trampoline<A>`."

**Verdict:** Correct. No action needed.

### 3.4 `CatListBrand` is in scope but tangential to the lazy hierarchy

`CatListBrand` is used as a general-purpose collection brand (implementing `Functor`, `Foldable`, `Traversable`, `Filterable`, `Witherable`, `ParFoldable`, etc.) and as the internal data structure for `Free`. It is not conceptually a "lazy evaluation" brand. Its presence in the lazy hierarchy analysis is a categorization artifact. There is nothing wrong with its definition or placement.

**Verdict:** Correct. `CatListBrand` is a proper standalone brand.

### 3.5 Documentation quality

The doc comments on lazy brands are minimal but adequate. Each brand has a one-line doc comment linking to its concrete type. Observations:

- `ThunkBrand` has a useful clarifying note: "This is for `Thunk<'a, A>`, NOT for `Trampoline<A>`."
- `SendThunkBrand` has a helpful note about its relationship to `ThunkBrand`.
- `TrySendThunkBrand` and `TryThunkBrand` note the `(Bifunctor)` role.
- `LazyBrand<Config>` links to `Lazy` but does not mention the `Config` parameter's role. It would benefit from a note like "parameterized by a [`LazyConfig`] that determines the pointer and cell strategy (Rc/LazyCell vs Arc/LazyLock)."
- `TryLazyBrand<E, Config>` links to `TryLazy` but does not describe its two parameters.
- The type aliases (`RcLazyBrand`, `ArcLazyBrand`, `RcTryLazyBrand<E>`, `ArcTryLazyBrand<E>`) have clear, concise docs.

**Recommendation:** Expand documentation on `LazyBrand<Config>` and `TryLazyBrand<E, Config>` to describe their type parameters.

### 3.6 `TryThunkErrAppliedBrand` and `TryThunkOkAppliedBrand` naming

The names follow the `ResultErrAppliedBrand<E>` / `ResultOkAppliedBrand<T>` convention exactly, which is good for consistency.

However, the semantics can be confusing:
- `TryThunkErrAppliedBrand<E>` means "the error type `E` is applied (fixed)," so this is a functor over the *success* type.
- `TryThunkOkAppliedBrand<A>` means "the success type `A` is applied (fixed)," so this is a functor over the *error* type.

This is correct and consistent with the `Result` brands, but the "applied" terminology (meaning "the named parameter is fixed") is the opposite of what some readers might expect (they might read "ErrApplied" as "applied to the error channel"). The existing doc comments ("with the error value applied (Functor over Ok)") clarify this well.

**Verdict:** Naming is correct and consistent. Doc comments handle the potential confusion.

## 4. `LazyBrand<Config>` Parameterization Design

The parameterization of `LazyBrand` over `Config: LazyConfig` is well-designed. It achieves:

1. **Single brand definition** for both `Rc` and `Arc` variants, avoiding duplication.
2. **Extensibility** for third-party lazy cell implementations.
3. **Clean type alias ergonomics:** `RcLazyBrand` and `ArcLazyBrand` are simple, memorable names.
4. **Separate trait implementations** where needed: `RefFunctor` is implemented for `LazyBrand<RcLazyConfig>`, `SendRefFunctor` for `LazyBrand<ArcLazyConfig>`, and `Foldable` for each separately.

The `Config` trait design (bundling `PointerBrand`, `Lazy`, `Thunk`, `lazy_new`, `evaluate`) is clean and provides the right abstraction surface.

**One subtlety:** The `impl_kind!` is generic over `Config: LazyConfig`, meaning `Kind` is implemented for *any* `LazyBrand<Config>` where `Config: LazyConfig`. But the trait implementations (`RefFunctor`, `SendRefFunctor`, `Foldable`, `Deferrable`) are specialized to specific configs. This means a third-party `LazyConfig` implementation gets the `Kind` mapping for free but needs to implement traits separately. This is the right design choice, since the trait implementations genuinely differ between `Rc` and `Arc` variants.

## 5. Naming Consistency

The naming scheme is consistent across the hierarchy:

| Pattern | Infallible | Fallible |
|---------|-----------|----------|
| Non-memoized, `!Send` | `ThunkBrand` | `TryThunkBrand` |
| Non-memoized, `Send` | `SendThunkBrand` | `TrySendThunkBrand` |
| Memoized, configurable | `LazyBrand<Config>` | `TryLazyBrand<E, Config>` |
| Memoized, `Rc` alias | `RcLazyBrand` | `RcTryLazyBrand<E>` |
| Memoized, `Arc` alias | `ArcLazyBrand` | `ArcTryLazyBrand<E>` |

The `Try` prefix consistently marks fallibility. The `Send` prefix consistently marks thread safety. The `Rc`/`Arc` prefixes on aliases consistently indicate pointer strategy.

One minor note: `SendThunkBrand` uses a `Send` prefix, while `ArcLazyBrand` uses an `Arc` prefix to indicate thread safety. These are different naming strategies for the same axis. The distinction is justified because:
- `SendThunk` wraps a `Box<dyn FnOnce() + Send>` (the `Send` is the defining characteristic).
- `ArcLazy` wraps an `Arc<LazyLock<...>>` (the `Arc` is the defining characteristic).

So the names reflect the underlying implementation detail that distinguishes each variant, which is reasonable.

## 6. Potential Alternatives

### 6.1 Unified `ThunkBrand<S>` parameterized by send-ness

Instead of separate `ThunkBrand` and `SendThunkBrand`, one could imagine `ThunkBrand<SendMarker>` with `type NonSendThunkBrand = ThunkBrand<NotSend>` and `type SendableThunkBrand = ThunkBrand<IsSend>`.

**Assessment:** This would add complexity for little benefit. The two variants have fundamentally different trait implementation surfaces (`ThunkBrand` implements `Functor`, `Semimonad`, etc.; `SendThunkBrand` cannot). A unified brand would still need separate trait impls, so the parameterization would not reduce code. The current separate brands are simpler.

### 6.2 Eliminating `TryLazyBrand<E, Config>` in favor of `LazyBrand<Config>` over `Result`

Since `TryLazy<'a, A, E, Config>` is conceptually `Lazy<'a, Result<A, E>, Config>`, one might wonder if a separate `TryLazyBrand` is necessary.

**Assessment:** The separate brand is justified because `TryLazy` has different trait implementations. `LazyBrand<Config>` implements `RefFunctor` (mapping `&A -> B`), while `TryLazyBrand<E, Config>` implements `RefFunctor` with semantics that handle the `Ok`/`Err` split (mapping `&A -> B` over the success value, cloning the error). A `Lazy<Result<A, E>>` with `LazyBrand`'s `RefFunctor` would map `&Result<A, E> -> B`, which is not what you want for error-handling combinators.

## 7. Summary of Recommendations

1. **Add trait bounds to `LazyBrand<Config>` and `TryLazyBrand<E, Config>`** at the struct definition site, matching the `FnBrand<PtrBrand: RefCountedPointer>` pattern. If this causes downstream issues, document why the bounds are intentionally omitted.

2. **Expand documentation on `LazyBrand<Config>`** to describe the `Config` parameter and its role.

3. **Expand documentation on `TryLazyBrand<E, Config>`** to describe both the `E` and `Config` parameters.

4. **Document why `TrySendThunk` lacks partially-applied brands** (unlike `TryThunk`), either in the doc comment on `TrySendThunkBrand` or in the module docs for `try_send_thunk.rs`.

5. No structural changes are needed. The brand hierarchy is sound, the `impl_kind!` mappings are correct, and the naming is consistent.
