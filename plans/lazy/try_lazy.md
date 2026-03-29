# TryLazy Analysis

File: `fp-library/src/types/try_lazy.rs` (1897 lines of implementation + ~1040 lines of tests)

## 1. Type Design

### Core Structure

```rust
pub struct TryLazy<'a, A, E, Config: TryLazyConfig = RcLazyConfig>(
    pub(crate) Config::TryLazy<'a, A, E>,
)
```

The concrete backing types are:
- `RcTryLazy`: `Rc<LazyCell<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + 'a>>>`
- `ArcTryLazy`: `Arc<LazyLock<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + Send + 'a>>>`

The `Config` parameter (defaulting to `RcLazyConfig`) controls the pointer and cell strategy via `TryLazyConfig`. Type aliases `RcTryLazy<'a, A, E>` and `ArcTryLazy<'a, A, E>` eliminate the config boilerplate for consumers.

### Memoization of Result<A, E>

The design memoizes the entire `Result<A, E>`, meaning errors are cached alongside successes. This is sound in the sense that `LazyCell`/`LazyLock` provide the same guarantee: the closure runs at most once, and whatever it returns (Ok or Err) is the permanent cached value.

The `evaluate` method returns `Result<&A, &E>` via `Result::as_ref()`, which is a clean projection that avoids cloning the cached value. This is consistent with how `Lazy::evaluate` returns `&A`.

### Soundness Verdict

The approach is structurally sound. It is a newtype over `Lazy<Result<A, E>>` with a richer combinator surface specialized for the Result pattern.


## 2. HKT Support (Partial)

### What TryLazy Supports

| Trait | Implemented? | Notes |
|-------|-------------|-------|
| `RefFunctor` | Yes (`TryLazyBrand<E, RcLazyConfig>`) | Requires `E: Clone + 'static`. |
| `SendRefFunctor` | Yes (`TryLazyBrand<E, ArcLazyConfig>`) | Requires `E: Clone + Send + Sync + 'static`. |
| `Foldable` | Yes (`TryLazyBrand<E, Config>`) | Generic over any `TryLazyConfig`. |
| `Deferrable` | Yes (both Rc and Arc) | Requires `A: Clone, E: Clone`. |
| `SendDeferrable` | Yes (ArcTryLazy only) | Requires `A: Clone + Send + Sync, E: Clone + Send + Sync`. |

### What TryLazy Does NOT Support

| Trait | Why Not |
|-------|---------|
| `Functor` | Same reason as `Lazy`: `evaluate()` returns `Result<&A, &E>`, not owned `Result<A, E>`. The standard `Functor` trait requires `A -> B`, which would need either cloning or consuming the cached value. |
| `Applicative` / `Monad` | Cannot provide `pure` or `bind` without owned values. The reference-based evaluation model prevents standard monadic composition. |
| `FoldableWithIndex` | Not implemented, unlike `Lazy` which implements it with `Index = ()`. This is a gap. |
| `WithIndex` | Missing, which is prerequisite for `FoldableWithIndex`. |
| `Traversable` | Not implemented. `Lazy` does not implement it either, so this is consistent. |
| `Bifunctor` | Not implemented at the HKT level, though `bimap` exists as an inherent method. |

### Why Partial

The HKT support is partial because TryLazy's fundamental evaluation model returns borrowed references (`&A`, `&E`), which is incompatible with the standard `Functor` trait that requires `A -> B` (owned values). `RefFunctor` was designed specifically for this use case. The `Foldable` implementation works around this by requiring `A: Clone` to extract owned values from the references.

### The `E: 'static` Constraint on TryLazyBrand

The HKT brand `TryLazyBrand<E, Config>` is parameterized by the error type `E`, which appears as `E: 'static` in the `impl_kind!` invocation:

```rust
impl_kind! {
    impl<E: 'static, Config: TryLazyConfig> for TryLazyBrand<E, Config> {
        type Of<'a, A: 'a>: 'a = TryLazy<'a, A, E, Config>;
    }
}
```

This `'static` bound on `E` is a significant restriction. It means HKT-polymorphic code cannot work with `TryLazy` values whose error type borrows from local data. The brands.rs file acknowledges this limitation. For non-HKT usage (direct inherent methods), `E` has no such restriction.


## 3. Comparison to Lazy: Shared vs. Duplicated Code

### What Is Shared

TryLazy reuses the infrastructure from `lazy.rs`:
- `LazyConfig` and `TryLazyConfig` traits (both defined in `lazy.rs`).
- `RcLazyConfig` and `ArcLazyConfig` structs and their `TryLazyConfig` implementations (also in `lazy.rs`).
- The pointer/cell machinery (`Rc<LazyCell<...>>`, `Arc<LazyLock<...>>`) via the config trait.

### What Is Duplicated

There is substantial structural duplication between `lazy.rs` and `try_lazy.rs`. Nearly every impl block is mirrored:

| Feature | Lazy | TryLazy |
|---------|------|---------|
| `Clone` | Yes | Yes (identical pattern) |
| `evaluate` | Returns `&A` | Returns `Result<&A, &E>` |
| `new` (Rc variant) | Yes | Yes |
| `new` (Arc variant) | Yes | Yes |
| `pure`/`ok`/`err` | `pure` | `ok` + `err` |
| `ref_map`/`map` | `ref_map` | `map`, `map_err`, `bimap` |
| `From<Thunk>` / `From<TryThunk>` | Both configs | Both configs |
| `From<Trampoline>` / `From<TryTrampoline>` | Both configs | Both configs |
| `From<SendThunk>` / `From<TrySendThunk>` | Arc only | Arc only |
| `From<RcLazy> for ArcLazy` / cross-config | Yes | No (missing) |
| `Deferrable` | Both configs | Both configs |
| `SendDeferrable` | ArcLazy | ArcTryLazy |
| `RefFunctor` | `LazyBrand<RcLazyConfig>` | `TryLazyBrand<E, RcLazyConfig>` |
| `SendRefFunctor` | `LazyBrand<ArcLazyConfig>` | `TryLazyBrand<E, ArcLazyConfig>` |
| `Foldable` | Yes | Yes (treats Err as empty) |
| `FoldableWithIndex` | Yes | **Missing** |
| `WithIndex` | Yes (`Index = ()`) | **Missing** |
| `Display` | Yes (forces evaluation) | **Missing** |
| `Debug` | Yes (non-forcing) | Yes (non-forcing) |
| `Semigroup` | Both configs | Both configs (short-circuiting) |
| `Monoid` | Both configs | Both configs |
| `Hash` | Yes | Yes |
| `PartialEq` | Yes | Yes |
| `PartialOrd` | Yes | Yes |
| `Eq` | Yes | Yes |
| `Ord` | Yes | Yes |
| Fix combinators | `rc_lazy_fix`, `arc_lazy_fix` | **Missing** |

### Duplication Pattern

The Rc/Arc duplication within TryLazy itself is significant. Methods like `map`, `map_err`, `bimap`, `and_then`, `or_else`, `catch_unwind`, `catch_unwind_with` are all duplicated between `RcTryLazy` and `ArcTryLazy` with the only difference being `Send + Sync` bounds and pointer type. This is the same pattern as `Lazy`, and reflects a fundamental limitation of Rust's trait system: there is no way to abstract over "has Send bounds" vs "no Send bounds" without either dynamic dispatch or macro-based code generation.


## 4. TryLazyConfig Trait

Defined in `lazy.rs`:

```rust
pub trait TryLazyConfig: LazyConfig {
    type TryLazy<'a, A: 'a, E: 'a>: Clone;
    type TryThunk<'a, A: 'a, E: 'a>: ?Sized;
    fn try_lazy_new<'a, A: 'a, E: 'a>(f: Box<Self::TryThunk<'a, A, E>>) -> Self::TryLazy<'a, A, E>;
    fn try_evaluate<'a, 'b, A: 'a, E: 'a>(lazy: &'b Self::TryLazy<'a, A, E>) -> Result<&'b A, &'b E>;
}
```

### How It Extends LazyConfig

`TryLazyConfig` is a subtrait of `LazyConfig`, which means any config that supports fallible memoization also supports infallible memoization. This is a clean hierarchical design:
- `LazyConfig` provides `Lazy<'a, A>` and `evaluate -> &A`.
- `TryLazyConfig` adds `TryLazy<'a, A, E>` and `try_evaluate -> Result<&A, &E>`.

The trait is open for third-party implementations, so users can plug in custom pointer/cell strategies (e.g., `parking_lot`-based locks or async-aware cells).

### Design Observation

The `try_evaluate` method returns `Result<&'b A, &'b E>` with the borrow lifetime `'b` tied to the lazy cell reference. This is correct: the references point into the `Rc`/`Arc`-managed cell, so they are valid as long as the borrow of the cell is alive. The `.as_ref()` call on `Result<A, E>` is what provides this projection.


## 5. Error Handling Semantics

### Error Memoization: Is It Right?

Error memoization is the correct default for the stated use case: "the computation itself may fail, and you want memoization of the entire outcome." Once a fallible computation has been run, the result (success or failure) is cached. This matches the semantics of `LazyCell`/`LazyLock` in the standard library, which provide no retry mechanism.

Scenarios where error memoization is appropriate:
- Configuration parsing that should fail fast and consistently.
- Resource initialization where retrying would be side-effectful.
- Pure computations that deterministically fail.

### Should Failed Computations Be Retryable?

The module documentation addresses this question implicitly through the panic-handling section. For truly retryable computations, `TryLazy` is the wrong abstraction. The `FnOnce` closure model means the computation cannot be re-run by design.

If retry semantics are needed, users should use:
- `TryThunk` (non-memoized, re-evaluable via cloning the thunk).
- A custom cell type that supports retry (e.g., wrapping `OnceLock` with `set` rather than `LazyLock`).

The documentation could be more explicit about this trade-off. The "Choosing between TryLazy, Lazy<Result>, and Result<Lazy>" section at the module level is good but does not mention the retry question.

### Panic Poisoning

The `catch_unwind` and `catch_unwind_with` methods provide an escape hatch for panic-safe memoization. If the closure panics without `catch_unwind`, the `LazyCell`/`LazyLock` is poisoned, and subsequent evaluations will panic again. This is documented but could be more prominently warned about.


## 6. Type Class Implementations

### RefFunctor (`TryLazyBrand<E, RcLazyConfig>`)

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    f: impl FnOnce(&A) -> B + 'a,
    fa: TryLazy<'a, A, E, RcLazyConfig>,
) -> TryLazy<'a, B, E, RcLazyConfig>
```

Requires `E: Clone + 'static`. Clones the error when the result is `Err`. This is correct: the new cell must own its own `Result<B, E>`, so the error must be cloned from the reference.

### SendRefFunctor (`TryLazyBrand<E, ArcLazyConfig>`)

Same pattern with additional `Send + Sync` bounds. Correct.

### Foldable (`TryLazyBrand<E, Config>`)

Treats `TryLazy` as a zero-or-one container:
- `Ok(a)`: fold includes `a` (cloned from reference).
- `Err(_)`: fold returns initial accumulator; error is silently discarded.

This is the standard encoding used by Haskell's `Either` Foldable instance (where `Left` values are ignored). It is correct but the silent error discarding is a potential surprise. The module-level documentation explicitly warns about this, which is good.

All three Foldable methods (`fold_right`, `fold_left`, `fold_map`) are implemented consistently. They all clone `A` from the reference, which requires `A: Clone` (encoded in the trait bound as `A: 'a + Clone`).

### Deferrable (RcTryLazy)

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self {
    Self::new(move || match f().evaluate() {
        Ok(a) => Ok(a.clone()),
        Err(e) => Err(e.clone()),
    })
}
```

This flattens `TryLazy<TryLazy<A, E>, E>` into `TryLazy<A, E>` by evaluating the inner TryLazy and cloning both Ok and Err variants. Requires `A: Clone, E: Clone`. Correct.

### Deferrable (ArcTryLazy)

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self {
    f()
}
```

Eagerly calls `f()` and returns the result directly. This is because `Deferrable::defer` does not require `Send` on the thunk, but `ArcTryLazy::new` does. Documented and correct, though it means `defer` for ArcTryLazy provides no actual deferral; the thunk runs immediately.

### SendDeferrable (ArcTryLazy)

```rust
fn send_defer(f: impl FnOnce() -> Self + Send + 'a) -> Self {
    Self::new(move || match f().evaluate() {
        Ok(a) => Ok(a.clone()),
        Err(e) => Err(e.clone()),
    })
}
```

This actually defers, since the `Send` bound on `f` is compatible with `ArcTryLazy::new`. Correct.

### Semigroup

Short-circuits on the first `Err`: if `a` fails, `b` is never evaluated. This is a deliberate and documented choice. The implementation clones both values out of their references, which requires `A: Semigroup + Clone` and `E: Clone`. Correct.

### Monoid

`empty()` returns `Ok(A::empty())`, wrapping the monoid identity in the success case. This is the natural lift of `Monoid` into the error context. Correct.


## 7. Documentation Quality

### Module-Level Documentation

Excellent. The module doc covers:
- The purpose and caching semantics.
- The `Foldable` error-discarding behavior (explicitly warned).
- When to use `TryLazy` vs `Lazy<Result>` vs `Result<Lazy>` (clear comparison table in prose).
- The naming rationale for `map` vs `ref_map`.

### Method-Level Documentation

Thorough. Every method and impl has:
- `#[document_signature]` attributes.
- `#[document_type_parameters(...)]` with descriptions.
- `#[document_parameters(...)]` with descriptions.
- `#[document_returns(...)]` descriptions.
- `#[document_examples]` with runnable code.
- Where relevant, "Why X?" subsections explaining design constraints (e.g., "Why `E: Clone`?", "Why `A: Clone`?").

### Gaps

- The relationship between inherent `map`/`bimap`/`and_then` and the HKT `RefFunctor::ref_map` could be clearer. A user might wonder why both exist.
- The panic poisoning behavior is documented on `evaluate` but not prominently featured in the type-level docs.
- No mention of the retry question (see Section 5).


## 8. Issues, Limitations, and Design Flaws

### 8.1 Missing Implementations Compared to Lazy

1. **No `Display` implementation.** `Lazy` has `impl Display` that forces evaluation and displays the value. `TryLazy` has no equivalent. A `Display` impl that shows `Ok(value)` or `Err(error)` would be natural.

2. **No `FoldableWithIndex` / `WithIndex`.** `Lazy` implements both with `Index = ()`. `TryLazy` does not. Since `TryLazy` models zero-or-one elements (like `Option`), `Index = ()` would be appropriate.

3. **No fix combinators.** `Lazy` has `rc_lazy_fix` and `arc_lazy_fix` for self-referential lazy values. `TryLazy` has no equivalent. Whether this is needed is debatable, since self-referential fallible lazy values are a niche use case.

4. **No `From<RcTryLazy> for ArcTryLazy` or `From<ArcTryLazy> for RcTryLazy`.** `Lazy` has cross-config conversions (`RcLazy <-> ArcLazy`). `TryLazy` has conversions from `Lazy` and from external types (Thunk, Trampoline) but no direct cross-config conversion between `RcTryLazy` and `ArcTryLazy`.

### 8.2 Clone Requirements in Combinators

The `map`, `map_err`, `and_then`, `or_else`, `bimap` methods all create new `TryLazy` cells that must own their own `Result<B, E>`. This forces cloning the "other" variant:
- `map` requires `E: Clone` (clones error on `Err` path).
- `map_err` requires `A: Clone` (clones value on `Ok` path).
- `and_then` requires both `A: Clone` and `E: Clone`.
- `or_else` requires both `A: Clone` and `E: Clone`.
- `bimap` requires neither (both variants are transformed).

The `bimap` approach is the most efficient, as it avoids cloning entirely. Users who want to transform both sides should prefer `bimap` over chaining `map` + `map_err`.

### 8.3 Deferrable for ArcTryLazy Is Eagerly Evaluated

The `Deferrable` implementation for `ArcTryLazy` calls `f()` immediately, providing no actual deferral. This is documented but may surprise users who expect `defer` to delay computation. The `SendDeferrable` implementation does provide true deferral. This asymmetry is a consequence of the trait design where `Deferrable::defer` does not require `Send` on the thunk.

### 8.4 `evaluate()` Returns `Result<&A, &E>`, Not `&Result<A, E>`

The return type is `Result<&A, &E>` (obtained via `Result::as_ref()`), not `&Result<A, E>`. This is more ergonomic for pattern matching, but it means the caller cannot obtain a reference to the full `Result<A, E>` that the cell stores. This is a minor limitation.

### 8.5 No `unwrap` / `expect` Convenience Methods

While `Result` has `unwrap()` / `expect()`, `TryLazy` does not provide equivalent convenience methods. Users must call `evaluate()` and then handle the `Result` themselves. This is arguably correct (the library avoids panicking APIs) but could be a papercut for prototyping.

### 8.6 The `Hash` Implementation Calls `evaluate().hash(state)`

The `Hash` impl for `TryLazy` calls `self.evaluate().hash(state)`, which returns `Result<&A, &E>`. Since `Result<&A, &E>` implements `Hash` when both `A: Hash` and `E: Hash`, this works correctly, but it hashes the borrowed form, not the owned form. Fortunately, `Hash` for references is defined as `(&T).hash() == T.hash()`, and `Result<&A, &E>` hashes identically to `Result<A, E>` (both use the discriminant tag + inner hash). The doc test confirms this. Correct.

### 8.7 Semigroup Short-Circuit Semantics

The `Semigroup::append` implementation short-circuits on the first `Err`, which means errors from `a` take priority over errors from `b`. This is the standard "left-biased" error behavior (matching `Result`'s `and_then`), but it is not the only valid choice. An alternative would be to accumulate errors (requiring `E: Semigroup`). The current choice is simpler and consistent with typical Rust idioms.


## 9. Alternatives and Improvements

### 9.1 Add Missing Trait Implementations

- Add `Display` for `TryLazy` that forces evaluation and renders `Ok(value)` or `Err(error)`.
- Add `WithIndex` and `FoldableWithIndex` with `Index = ()` for consistency with `Lazy`.

### 9.2 Add Cross-Config Conversions

Add `From<RcTryLazy<A, E>> for ArcTryLazy<A, E>` (eager evaluation + clone) and `From<ArcTryLazy<A, E>> for RcTryLazy<A, E>` to match the `Lazy` conversion matrix.

### 9.3 Consider a `try_bimap` as the Preferred Combinator

Since `bimap` avoids the `Clone` requirements of `map` and `map_err`, it could be documented as the preferred combinator when both transformations are needed. A note in the `map` and `map_err` docs pointing to `bimap` as the more efficient alternative would be helpful.

### 9.4 Document the Retry Question

Add a section to the module docs explicitly addressing whether failed computations can be retried, and directing users to `TryThunk` or custom retry wrappers when retry semantics are needed.

### 9.5 Consider Adding Fix Combinators

Add `rc_try_lazy_fix` and `arc_try_lazy_fix` for self-referential fallible lazy values, analogous to the infallible fix combinators in `lazy.rs`. This is a low-priority enhancement since the use case is niche.

### 9.6 Consider a Blanket `From<Lazy<Result<A, E>>>` Conversion

Since `TryLazy` is essentially a newtype over `Lazy<Result<A, E>>` with a richer API, a conversion from `Lazy<Result<A, E>>` to `TryLazy<A, E>` (and vice versa) would make the relationship more explicit. Currently, conversions exist from `Lazy<A>` to `TryLazy<A, E>` (wrapping in Ok), but not from `Lazy<Result<A, E>>` to `TryLazy<A, E>`.

### 9.7 Reduce Rc/Arc Duplication with Macros

The inherent methods (`map`, `map_err`, `bimap`, `and_then`, `or_else`, `catch_unwind`, `catch_unwind_with`) are duplicated nearly identically between Rc and Arc variants, differing only in `Send + Sync` bounds. A declarative or proc macro could generate both variants from a single template, reducing the surface area for bugs and maintenance. However, this would need to be weighed against readability and the library's preference for explicit code.
