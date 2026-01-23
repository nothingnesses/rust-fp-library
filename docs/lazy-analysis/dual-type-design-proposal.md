# Dual-Type Lazy Evaluation Design Proposal

## Executive Summary

This document proposes a redesign of the lazy evaluation types in `fp-library` based on the **Dual-Type Design** pattern, enhanced with explicit error handling, the existing Config/Brand abstraction for thread-safety, and **Rust's standard library `LazyCell`/`LazyLock` primitives** (stabilized in Rust 1.80) for robust memoization.

The proposed design splits lazy evaluation into **four complementary types**:

| Type                    | Memoized? | Fallible? | Primary Use Case              |
| ----------------------- | --------- | --------- | ----------------------------- |
| `Eval<A>`               | No        | No        | Building computation chains   |
| `TryEval<A, E>`         | No        | Yes       | Fallible computation chains   |
| `Memo<A, Config>`       | Yes       | No        | Caching computed values       |
| `TryMemo<A, E, Config>` | Yes       | Yes       | Caching fallible computations |

## Problem Statement

The current [`Lazy`](../../fp-library/src/types/lazy.rs) implementation conflates three orthogonal concerns:

1. **Deferred Computation**: The ability to delay execution until needed
2. **Memoization**: The ability to cache results for subsequent accesses
3. **Error Handling**: The strategy for dealing with computation failures

This conflation leads to several problems:

### Problem 1: Algebraic Incompatibility

A memoized lazy value cannot be a proper `Monad` because:

- `bind` (flatMap) requires ownership: `fn bind<B>(ma: M<A>, f: fn(A) -> M<B>) -> M<B>`
- But `force()` returns `&A` (a reference), not `A` (owned value)
- Requiring `A: Clone` everywhere is a workaround, not a solution

### Problem 2: Premature Caching

Every `Lazy` value allocates an `Rc`/`Arc` and `OnceCell`/`OnceLock` immediately, even when:

- You only want to defer computation without caching
- You're building a chain of transformations where only the final result needs caching
- Intermediate values are never accessed directly

### Problem 3: Implicit Error Handling

The current design catches panics and caches them as errors, which:

- Conflates "unexpected failures" (panics) with "expected failures" (Result)
- Provides no type-level indication that a computation might fail
- Makes it impossible to distinguish between "computation not yet run" and "computation failed"

## Design Philosophy

### Principle 1: Separate Computation from Caching

**Insight**: A computation that _can_ be cached is different from a computation that _is_ cached.

- `Eval<A>`: Represents "what to compute" - a pure, deferred computation
- `Memo<A>`: Represents "compute once, remember forever" - adds caching semantics

This separation allows:

- Building complex computations without allocation overhead
- Choosing when and where to introduce caching
- Implementing proper `Monad` for the computation type

### Principle 2: Make Error Handling Explicit

**Insight**: The type signature should reflect whether a computation can fail.

- Infallible types (`Eval`, `Memo`): Panics propagate naturally
- Fallible types (`TryEval`, `TryMemo`): Errors are explicit in the type

This provides:

- Compile-time documentation of failure modes
- No nested `Result<Result<A, E>, PanicError>` confusion
- Opt-in panic catching when needed

### Principle 3: Abstract Over Thread-Safety

**Insight**: The choice of `Rc` vs `Arc` should not require different type names.

Using the existing `LazyConfig`/Brand pattern:

- `Memo<A, RcMemoConfig>` for single-threaded use
- `Memo<A, ArcMemoConfig>` for thread-safe use
- Generic code works with any config

### Principle 4: Leverage Standard Library Primitives

**Insight**: Rust 1.80 stabilized `std::cell::LazyCell` and `std::sync::LazyLock`, which encapsulate the tricky initialization-once logic.

Benefits of using standard library primitives:

- **Battle-tested**: These types are used throughout the Rust ecosystem
- **Less unsafe code**: We delegate interior mutability concerns to the standard library
- **Future optimizations**: Any performance improvements in std automatically benefit us
- **Correctness**: The standard library team has extensively tested edge cases

This means `Memo` and `TryMemo` can be implemented as thin wrappers around `LazyCell`/`LazyLock`, rather than manually managing `OnceCell` + `UnsafeCell<Option<thunk>>`.

## Proposed Type Hierarchy

```
                    ┌─────────────────┐
                    │   Computation   │
                    │    (no cache)   │
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
       ┌──────┴──────┐               ┌──────┴──────┐
       │   Eval<A>   │               │TryEval<A,E> │
       │ (infallible)│               │ (fallible)  │
       └──────┬──────┘               └──────┬──────┘
              │ .memoize()                  │ .memoize()
              ▼                             ▼
       ┌──────┴──────┐               ┌──────┴──────┐
       │ Memo<A,Cfg> │               │TryMemo<A,E> │
       │ (infallible)│               │ (fallible)  │
       └─────────────┘               └─────────────┘
```

## Detailed Type Specifications

### Eval<A> - Pure Deferred Computation

````rust
/// A deferred computation that produces a value of type `A`.
///
/// `Eval` is NOT memoized - each call to `run()` re-executes the computation.
/// This type exists to build computation chains without allocation overhead.
///
/// # Algebraic Properties
///
/// `Eval` is a proper Monad:
/// - `pure(a).run() == a` (left identity)
/// - `eval.bind(pure) == eval` (right identity)
/// - `eval.bind(f).bind(g) == eval.bind(|a| f(a).bind(g))` (associativity)
///
/// # Examples
///
/// ```rust
/// let computation = Eval::new(|| expensive_calculation())
///     .map(|x| x * 2)
///     .map(|x| x + 1);
///
/// // No computation has happened yet!
/// // Only when we call run() does it execute:
/// let result = computation.run();
/// ```
pub struct Eval<A> {
    thunk: Box<dyn FnOnce() -> A>,
}
````

**Design Decisions:**

1. **`Box<dyn FnOnce() -> A>`**: Uses `FnOnce` because computations are typically consumed once. This allows move captures and avoids `Clone` bounds.

2. **`run(self) -> A`**: Consumes `self` and returns owned `A`. This is crucial for Monad compatibility - we can chain without requiring `Clone`.

3. **No `Rc`/`Arc`**: No shared ownership means no allocation beyond the `Box`. Building a chain of 10 maps creates 10 `Box` allocations, not 10 `Rc` + 10 `OnceCell`.

4. **Implements `Functor` and `Monad`**: Because `run()` returns owned values, we can implement the standard typeclasses properly.

### TryEval<A, E> - Fallible Deferred Computation

````rust
/// A deferred computation that may fail with error type `E`.
///
/// Like `Eval`, this is NOT memoized. Each `run()` re-executes.
/// Unlike `Eval`, the result is `Result<A, E>`.
///
/// # Examples
///
/// ```rust
/// let computation: TryEval<Config, ConfigError> = TryEval::new(|| {
///     let path = std::env::var("CONFIG_PATH")?;
///     parse_config(&path)
/// });
///
/// match computation.run() {
///     Ok(config) => use_config(config),
///     Err(e) => handle_error(e),
/// }
/// ```
pub struct TryEval<A, E> {
    thunk: Box<dyn FnOnce() -> Result<A, E>>,
}
````

**Design Decisions:**

1. **Explicit `E` type parameter**: The error type is part of the signature. No surprise panics, no `Box<dyn Error>` - you know exactly what can fail.

2. **`run(self) -> Result<A, E>`**: Returns the full `Result`, not `Result<&A, &E>`. Ownership semantics preserved.

3. **No implicit panic catching**: If the thunk panics, the panic propagates. Use `TryEval::catch_unwind()` if you want to catch panics.

4. **Implements `Functor` for the success type**: `map` transforms `A`, `map_err` transforms `E`.

### Memo<A, Config> - Memoized Value

````rust
use std::cell::LazyCell;
use std::sync::LazyLock;

/// A lazily-computed, memoized value with shared semantics.
///
/// The computation runs at most once; subsequent accesses return the cached value.
/// Cloning a `Memo` shares the underlying cache - all clones see the same value.
///
/// # Implementation Note
///
/// Internally uses `std::cell::LazyCell` (for `RcMemoConfig`) or `std::sync::LazyLock`
/// (for `ArcMemoConfig`) to handle the memoization logic. This delegates the tricky
/// initialization-once semantics to the well-tested standard library.
///
/// # Type Parameters
///
/// - `A`: The type of the computed value
/// - `Config`: The memoization configuration (determines Rc vs Arc)
///
/// # Examples
///
/// ```rust
/// let memo = Memo::<_, RcMemoConfig>::new(|| expensive_calculation());
/// let shared = memo.clone();
///
/// // First force computes and caches:
/// let value = memo.force();
///
/// // Second force returns cached value (shared sees same result):
/// assert_eq!(shared.force(), value);
/// ```
pub struct Memo<A, Config: MemoConfig = RcMemoConfig> {
    inner: Config::Lazy<A>,
}

// For RcMemoConfig: uses Rc<LazyCell<A, Box<dyn FnOnce() -> A>>>
// For ArcMemoConfig: uses Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send>>>
````

**Design Decisions:**

1. **`force(&self) -> &A`**: Returns a reference, not owned value. This is the key difference from `Eval` - we're returning a reference to the cached value.

2. **Shared semantics via `Rc`/`Arc`**: Cloning shares the cache. This is essential for "compute once, use everywhere" patterns.

3. **Generic over `Config`**: Thread-safety is a configuration choice, not a type choice. Same `Memo` type works with `RcMemoConfig` or `ArcMemoConfig`.

4. **Uses `LazyCell`/`LazyLock` internally**: Instead of manually managing `OnceCell` + `UnsafeCell<Option<thunk>>`, we delegate to the standard library's `LazyCell` (single-threaded) or `LazyLock` (thread-safe). This:

   - Eliminates custom unsafe code for thunk management
   - Leverages battle-tested standard library primitives
   - Automatically benefits from future std optimizations

5. **Does NOT implement `Monad`**: Because `force()` returns `&A`, we can't implement proper `bind`. Instead, implements `RefFunctor` (a variant that works with references).

6. **Panics propagate**: If the thunk panics, the panic propagates. No implicit error caching. Use `TryMemo` for fallible computations.

### TryMemo<A, E, Config> - Memoized Fallible Value

````rust
/// A lazily-computed, memoized value that may fail.
///
/// The computation runs at most once. If it succeeds, the value is cached.
/// If it fails, the error is cached. Subsequent accesses return the cached result.
///
/// # Implementation Note
///
/// Like `Memo`, internally uses `LazyCell`/`LazyLock` to store `Result<A, E>`.
/// The standard library handles the initialization-once logic.
///
/// # Examples
///
/// ```rust
/// let memo: TryMemo<Config, ConfigError, RcMemoConfig> = TryMemo::new(|| {
///     parse_config_file()
/// });
///
/// match memo.force() {
///     Ok(config) => println!("Config: {:?}", config),
///     Err(e) => println!("Error (cached): {:?}", e),
/// }
///
/// // Second call returns cached result (success or error)
/// let _ = memo.force();
/// ```
pub struct TryMemo<A, E, Config: MemoConfig = RcMemoConfig> {
    // Stores Result<A, E> in the lazy cell
    inner: Config::TryLazy<A, E>,
}

// For RcMemoConfig: uses Rc<LazyCell<Result<A, E>, Box<dyn FnOnce() -> Result<A, E>>>>
// For ArcMemoConfig: uses Arc<LazyLock<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + Send>>>
````

**Design Decisions:**

1. **`force(&self) -> Result<&A, &E>`**: Returns references to the cached result. Both success and error values are borrowed.

2. **Errors are cached**: Once the computation fails, it stays failed. No retry logic. This matches the "memoization" semantics - compute once, remember forever.

3. **Explicit `E` type**: Like `TryEval`, the error type is part of the signature.

4. **`catch_unwind` as opt-in**: Static method `TryMemo::catch_unwind(f)` wraps a potentially-panicking thunk and converts panics to errors.

## Configuration System

### MemoConfig Trait

```rust
use std::cell::LazyCell;
use std::sync::LazyLock;
use std::rc::Rc;
use std::sync::Arc;

/// Configuration for memoization strategy.
///
/// This trait bundles together the choices for:
/// - Pointer type (Rc vs Arc)
/// - Lazy cell type (LazyCell vs LazyLock)
///
/// # Note on Standard Library Usage
///
/// This design leverages Rust 1.80's `LazyCell` and `LazyLock` types,
/// which encapsulate the initialization-once logic that we previously
/// implemented manually with `OnceCell` + `UnsafeCell<Option<thunk>>`.
pub trait MemoConfig: 'static {
    /// The lazy cell type for infallible memoization
    type Lazy<A: 'static>: Clone;

    /// The lazy cell type for fallible memoization
    type TryLazy<A: 'static, E: 'static>: Clone;

    /// Creates a new lazy cell from an initializer
    fn new_lazy<A: 'static>(f: impl FnOnce() -> A + 'static) -> Self::Lazy<A>;

    /// Creates a new fallible lazy cell from an initializer
    fn new_try_lazy<A: 'static, E: 'static>(
        f: impl FnOnce() -> Result<A, E> + 'static
    ) -> Self::TryLazy<A, E>;

    /// Forces evaluation and returns a reference
    fn force<A>(lazy: &Self::Lazy<A>) -> &A;

    /// Forces evaluation and returns a reference to the result
    fn force_try<A, E>(lazy: &Self::TryLazy<A, E>) -> Result<&A, &E>;
}

/// Single-threaded memoization using Rc<LazyCell>.
///
/// Not thread-safe. Use `ArcMemoConfig` for multi-threaded contexts.
pub struct RcMemoConfig;

impl MemoConfig for RcMemoConfig {
    type Lazy<A: 'static> = Rc<LazyCell<A, Box<dyn FnOnce() -> A>>>;
    type TryLazy<A: 'static, E: 'static> = Rc<LazyCell<Result<A, E>, Box<dyn FnOnce() -> Result<A, E>>>>;

    fn new_lazy<A: 'static>(f: impl FnOnce() -> A + 'static) -> Self::Lazy<A> {
        Rc::new(LazyCell::new(Box::new(f)))
    }

    fn new_try_lazy<A: 'static, E: 'static>(
        f: impl FnOnce() -> Result<A, E> + 'static
    ) -> Self::TryLazy<A, E> {
        Rc::new(LazyCell::new(Box::new(f)))
    }

    fn force<A>(lazy: &Self::Lazy<A>) -> &A {
        LazyCell::force(lazy)
    }

    fn force_try<A, E>(lazy: &Self::TryLazy<A, E>) -> Result<&A, &E> {
        LazyCell::force(lazy).as_ref()
    }
}

/// Thread-safe memoization using Arc<LazyLock>.
///
/// Requires `A: Send + Sync` for the value type.
pub struct ArcMemoConfig;

impl MemoConfig for ArcMemoConfig {
    type Lazy<A: 'static> = Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send>>>;
    type TryLazy<A: 'static, E: 'static> = Arc<LazyLock<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + Send>>>;

    fn new_lazy<A: 'static>(f: impl FnOnce() -> A + Send + 'static) -> Self::Lazy<A> {
        Arc::new(LazyLock::new(Box::new(f)))
    }

    fn new_try_lazy<A: 'static, E: 'static>(
        f: impl FnOnce() -> Result<A, E> + Send + 'static
    ) -> Self::TryLazy<A, E> {
        Arc::new(LazyLock::new(Box::new(f)))
    }

    fn force<A>(lazy: &Self::Lazy<A>) -> &A {
        LazyLock::force(lazy)
    }

    fn force_try<A, E>(lazy: &Self::TryLazy<A, E>) -> Result<&A, &E> {
        LazyLock::force(lazy).as_ref()
    }
}
```

**Design Decisions:**

1. **Uses `LazyCell`/`LazyLock` directly**: Instead of composing `OnceCell` + `UnsafeCell`, we use the standard library's `LazyCell` (Rust 1.80+) which handles this internally.

2. **Simplified trait**: The `MemoConfig` trait now directly exposes `Lazy<A>` and `TryLazy<A, E>` associated types, rather than separate `PtrBrand` and `OnceBrand`.

3. **Less unsafe code**: By delegating to std, we eliminate the `UnsafeCell<Option<Box<dyn FnOnce>>>` pattern from our codebase.

4. **Two standard configurations**: `RcMemoConfig` and `ArcMemoConfig` cover the common cases.

5. **Minimum Rust version**: Requires Rust 1.80 or later for `LazyCell`/`LazyLock` stabilization.

## Type Aliases for Ergonomics

```rust
// Single-threaded variants
pub type RcMemo<A> = Memo<A, RcMemoConfig>;
pub type RcTryMemo<A, E> = TryMemo<A, E, RcMemoConfig>;

// Thread-safe variants
pub type ArcMemo<A> = Memo<A, ArcMemoConfig>;
pub type ArcTryMemo<A, E> = TryMemo<A, E, ArcMemoConfig>;
```

## Conversions Between Types

```rust
impl<A> Eval<A> {
    /// Converts this computation to a memoized value.
    ///
    /// The resulting Memo will execute this computation at most once.
    pub fn memoize<Config: MemoConfig>(self) -> Memo<A, Config>
    where
        A: 'static,
    {
        Memo::new(move || self.run())
    }

    /// Converts to a TryEval that always succeeds.
    pub fn into_try<E>(self) -> TryEval<A, E>
    where
        A: 'static,
    {
        TryEval::new(move || Ok(self.run()))
    }
}

impl<A, E> TryEval<A, E> {
    /// Converts this fallible computation to a memoized value.
    pub fn memoize<Config: MemoConfig>(self) -> TryMemo<A, E, Config>
    where
        A: 'static,
        E: 'static,
    {
        TryMemo::new(move || self.run())
    }
}

impl<A, Config: MemoConfig> Memo<A, Config> {
    /// Converts to a TryMemo that always succeeds.
    pub fn into_try<E>(self) -> TryMemo<A, E, Config>
    where
        A: Clone + 'static,
        E: 'static,
    {
        TryMemo::new(move || Ok(self.force().clone()))
    }
}
```

## Error Handling Strategies

### Strategy 1: Let Panics Propagate (Default)

For `Eval` and `Memo`, panics propagate naturally. This follows Rust's "panics are bugs" philosophy.

```rust
let eval = Eval::new(|| panic!("oops"));
eval.run();  // Panics here, not caught
```

### Strategy 2: Explicit Result Types

For expected failures, use `TryEval` and `TryMemo` with explicit error types.

```rust
let try_eval: TryEval<Config, ParseError> = TryEval::new(|| {
    parse_config_file()  // Returns Result<Config, ParseError>
});

match try_eval.run() {
    Ok(config) => ...,
    Err(e) => ...,  // Handle ParseError
}
```

### Strategy 3: Opt-in Panic Catching

When you need to convert panics to errors, use `catch_unwind`:

```rust
let memo = TryMemo::<Data, String, RcConfig>::catch_unwind(|| {
    potentially_panicking_code()
});

match memo.force() {
    Ok(data) => ...,
    Err(panic_msg) => ...,  // Panic was caught and converted to String
}
```

## HKT Integration

### Eval Implements Full Monad

```rust
pub struct EvalBrand;

impl_kind! {
    for EvalBrand {
        type Of<'a, A: 'a>: 'a = Eval<A>;
    }
}

impl Functor for EvalBrand {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Eval<A>) -> Eval<B>
    where
        F: Fn(A) -> B + 'a,
    {
        Eval::new(move || f(fa.run()))
    }
}

impl Semimonad for EvalBrand {
    fn bind<'a, B: 'a, A: 'a, F>(ma: Eval<A>, f: F) -> Eval<B>
    where
        F: Fn(A) -> Eval<B> + 'a,
    {
        Eval::new(move || f(ma.run()).run())
    }
}

impl Pointed for EvalBrand {
    fn of<'a, A: 'a>(a: A) -> Eval<A> {
        Eval::new(move || a)
    }
}
```

### Memo Implements RefFunctor

Since `Memo::force()` returns `&A`, we cannot implement the standard `Functor`. Instead, we implement a reference-based variant:

```rust
pub struct MemoBrand<Config: MemoConfig>;

impl_kind! {
    impl<Config: MemoConfig> for MemoBrand<Config> {
        type Of<'a, A: 'a>: 'a = Memo<A, Config>;
    }
}

impl<Config: MemoConfig> RefFunctor for MemoBrand<Config> {
    fn map_ref<'a, B: 'a, A: 'a, F>(f: F, fa: Memo<A, Config>) -> Memo<B, Config>
    where
        F: FnOnce(&A) -> B + 'a,
    {
        Memo::new(move || f(fa.force()))
    }
}
```

## Comparison with Current Implementation

| Aspect         | Current `Lazy`                | New Dual-Type Design                             |
| -------------- | ----------------------------- | ------------------------------------------------ |
| Types          | 1 (`Lazy<Config, A>`)         | 4 (`Eval`, `TryEval`, `Memo`, `TryMemo`)         |
| Memoization    | Always                        | Optional (Memo only)                             |
| Error handling | Implicit panic catching       | Explicit (TryEval/TryMemo)                       |
| Monad support  | Partial (requires Clone)      | Full for Eval, RefFunctor for Memo               |
| Allocation     | Always Rc/Arc + OnceCell      | Eval: just Box; Memo: Rc/Arc + LazyCell/LazyLock |
| Thread-safety  | Via config                    | Via config (unchanged)                           |
| Unsafe code    | Manual OnceCell + UnsafeCell  | Delegated to std::cell::LazyCell                 |
| Minimum Rust   | 1.70 (OnceCell stabilization) | 1.80 (LazyCell/LazyLock stabilization)           |

## Migration Path

### Phase 1: Add New Types

Add `Eval`, `TryEval`, `Memo`, `TryMemo` as new types alongside existing `Lazy`.

### Phase 2: Deprecate Old API

Mark `Lazy` as deprecated with migration guidance:

```rust
/// @deprecated Use `Memo` for memoized values or `Eval` for pure computations.
#[deprecated(since = "0.2.0", note = "Use Memo or Eval instead")]
pub type Lazy<'a, Config, A> = Memo<A, Config>;
```

### Phase 3: Remove Old Implementation

In a future major version, remove the old `Lazy` type entirely.

## Open Questions

1. **Naming**: Should we use `Eval`/`Memo` (Cats/Scala convention) or `Thunk`/`Lazy` (more Rust-idiomatic)?

2. **TryEval Monad Instance**: Should `TryEval<A, E>` implement `Monad` for `A` (assuming fixed `E`), or should it implement `Bifunctor`/`MonadError`?

3. **Semigroup/Monoid**: Should `Memo` implement `Semigroup`/`Monoid` when `A` does? The current implementation does this.

4. **Defer Typeclass**: How should `Defer` work with the new types? `Eval` naturally supports defer (it IS deferred), but `Memo` might need special handling.

5. **Rust Version**: Is requiring Rust 1.80+ acceptable? If not, we could provide a fallback implementation using `OnceCell`/`OnceLock` for older Rust versions.

6. **Backward Compatibility with Existing Brands**: The current library uses `RefCountedPointer`, `ThunkWrapper`, and `Once` traits. Should we:
   - Keep these traits and implement `MemoConfig` in terms of them?
   - Simplify to just `MemoConfig` and deprecate the fine-grained traits?
   - Support both approaches?

## Conclusion

The Dual-Type Design provides a cleaner separation of concerns:

- **Computation vs Caching**: `Eval` builds chains, `Memo` caches results
- **Fallible vs Infallible**: Type-level distinction, not runtime surprise
- **Thread-safe vs Single-threaded**: Configuration, not type proliferation
- **Standard Library Foundation**: `Memo`/`TryMemo` leverage `LazyCell`/`LazyLock` for robust memoization

This design is more aligned with functional programming principles while remaining practical for Rust's ownership model. By building on standard library primitives, we reduce the amount of unsafe code in our implementation and benefit from the Rust team's extensive testing and optimization work.

## Appendix: Why LazyCell/LazyLock?

### Before (Manual Implementation)

```rust
struct MemoInner<A> {
    cell: OnceCell<A>,
    thunk: UnsafeCell<Option<Box<dyn FnOnce() -> A>>>,  // Unsafe!
}

impl<A> Memo<A> {
    fn force(&self) -> &A {
        self.inner.cell.get_or_init(|| {
            // SAFETY: We only access this once due to OnceCell guarantees
            let thunk = unsafe { (*self.inner.thunk.get()).take() };
            thunk.expect("thunk already consumed")()
        })
    }
}
```

### After (Standard Library Foundation)

```rust
struct Memo<A> {
    inner: Rc<LazyCell<A, Box<dyn FnOnce() -> A>>>,
}

impl<A> Memo<A> {
    fn force(&self) -> &A {
        LazyCell::force(&self.inner)  // No unsafe code!
    }
}
```

The standard library handles:

- Interior mutability for the thunk
- Synchronization for thread-safe variants
- Panic safety during initialization
- Memory ordering for concurrent access
