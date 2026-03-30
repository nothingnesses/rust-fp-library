# Analysis: `try_lazy.rs`

**File:** `fp-library/src/types/try_lazy.rs`
**Role:** `TryLazy<'a, A, E, Config>`, fallible memoized lazy evaluation.

## Design

`TryLazy<'a, A, E, Config: TryLazyConfig = RcLazyConfig>` is a newtype over `Config::TryLazy<'a, A, E>`, which stores `Result<A, E>` in a lazy cell. The `evaluate` method returns `Result<&A, &E>`.

Type aliases: `RcTryLazy<'a, A, E>`, `ArcTryLazy<'a, A, E>`.

## Relationship to `Lazy`

At the storage level, `TryLazy<'a, A, E, Config>` is structurally identical to `Lazy<'a, Result<A, E>, Config>`. Both use the same `LazyCell`/`LazyLock` with `Result<A, E>` as the value type. However, `TryLazy` is NOT implemented as a wrapper around `Lazy`; it is a parallel, independent type.

## Assessment

### Correct decisions

1. **`evaluate() -> Result<&A, &E>` splits references.** This is more ergonomic than `&Result<A, E>` for pattern matching and error handling.
2. **Separate HKT brand `TryLazyBrand<E, Config>`.** Enables Foldable/RefFunctor that operate on the success type only.
3. **Error-aware combinators.** `map`, `map_err`, `and_then`, `or_else`, `bimap`, `catch_unwind` provide a complete fallible API.

### Issues

#### 1. Massive code duplication with `lazy.rs`

This is the most significant issue. Nearly every impl block is duplicated between the two files:

- Clone, Hash, PartialEq, PartialOrd, Eq, Ord, Display, Debug
- new, evaluate, ref_map (inherent methods for both Rc and Arc variants)
- Deferrable, SendDeferrable
- Semigroup, Monoid
- Foldable, FoldableWithIndex, WithIndex
- RefFunctor, SendRefFunctor
- Fix-point combinators
- From conversions

The file is approximately 3830 lines, making it the largest in the hierarchy. If `TryLazy` were a newtype over `Lazy<'a, Result<A, E>, Config>`, most of these implementations could be derived or delegated, roughly halving the file.

**Impact:** High. The duplication is a significant maintenance burden. Every behavioral change or bugfix to `Lazy` must be manually replicated in `TryLazy`.

#### 2. `TryLazyConfig` is structurally redundant

For both built-in configs, `TryLazyConfig::TryLazy<'a, A, E>` is identical to `LazyConfig::Lazy<'a, Result<A, E>>`. The `TryLazyConfig` trait adds no new capability beyond convenience associated types. If `TryLazy` wrapped `Lazy<Result<A, E>>`, the `TryLazyConfig` trait could be eliminated.

**Impact:** Moderate. The trait adds complexity without functional value, and its extensibility point is unlikely to be used independently from `LazyConfig`.

#### 3. `map` and `ref_map` are identical for `RcTryLazy`

Both methods on `RcTryLazy` have the exact same implementation. The documentation explains the naming difference (Result-style vs Lazy-style), but two identical methods add API surface without adding functionality.

**Impact:** Low. Confusing but not harmful.

#### 4. Deferrable for `ArcTryLazy` evaluates eagerly

Same issue as `Lazy<ArcLazyConfig>`: the `Deferrable::defer` implementation calls `f()` immediately.

**Impact:** Moderate. Same as in `lazy.rs`.

#### 5. `E: 'static` in `impl_kind!` limits HKT usage

`TryLazyBrand<E, Config>` requires `E: 'static` due to the brand pattern's type erasure. Borrowed error types cannot participate in HKT abstractions.

**Impact:** Moderate.

#### 6. No Bifunctor HKT representation

`TryLazy` has `bimap` as an inherent method but no `Bifunctor` brand. This limits composability in generic bifunctor code. `TryThunk` has `TryThunkBrand` for this purpose; `TryLazy` does not.

**Impact:** Low-moderate. Limits generic programming over both type parameters.

#### 7. Semigroup uses fail-fast semantics (undocumented)

The `Semigroup` impl evaluates `a` first and short-circuits on `Err`, never evaluating `b`. This is a left-biased "fail-fast" strategy. An alternative would be error accumulation. The choice is reasonable but the impl doc does not explicitly state the short-circuit behavior.

**Impact:** Low.

#### 8. No property-based tests for combinator laws

QuickCheck tests cover memoization and deferrable transparency, but not `map`, `and_then`, `or_else`, `bimap`, `Semigroup`, or `Foldable` laws. The unit tests are thorough but law-based property tests would provide stronger guarantees.

**Impact:** Low-moderate.

## Should TryLazy wrap Lazy<Result<A, E>>?

**Arguments for:**

- Eliminates massive code duplication.
- `Lazy<Result<A, E>>::evaluate()` returns `&Result<A, E>`, from which `.as_ref()` gives `Result<&A, &E>`, so the API can be built on top.
- Clone, Hash, Eq, Ord, Debug, Deferrable would be inherited for free.
- `TryLazyConfig` could be eliminated entirely.

**Arguments against:**

- `TryLazyBrand<E, Config>` needs separate HKT semantics from `LazyBrand<Config>`.
- The combinator API (`map`, `map_err`, `and_then`, `or_else`) is specific to the fallible type.
- An extra layer of indirection in the type makes error messages more confusing.

**Recommendation:** A middle ground: define `TryLazy` as a newtype over `Lazy<'a, Result<A, E>, Config>`, delegating shared impls while providing its own brands and combinator API. This would cut the file roughly in half while preserving the full API surface. The `TryLazyConfig` trait could be deprecated or removed (breaking change).

## Strengths

- Comprehensive fallible API (map, map_err, and_then, or_else, bimap, catch_unwind).
- Correct shared memoization semantics inherited from the LazyConfig pattern.
- Well-documented cache chain behavior and error handling semantics.
