# Lazy Type: Architectural Critique and Redesign Proposals

This document provides a deep architectural analysis of the `Lazy` type implementation in [`fp-library/src/types/lazy.rs`](../../fp-library/src/types/lazy.rs), examining root causes of design issues, and proposing holistic alternatives that address fundamental flaws rather than surface symptoms.

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Assessment of Existing Analysis](#assessment-of-existing-analysis)
3. [Root Architectural Flaws](#root-architectural-flaws)
   - [Flaw 1: Panic-Based Error Propagation](#flaw-1-panic-based-error-propagation)
   - [Flaw 2: CloneableFn Reuse Constraint](#flaw-2-cloneablefn-reuse-constraint)
   - [Flaw 3: Over-Abstracted Configuration Pattern](#flaw-3-over-abstracted-configuration-pattern)
   - [Flaw 4: Implicit Shared Semantics](#flaw-4-implicit-shared-semantics)
   - [Flaw 5: Brand-Based HKT Complexity](#flaw-5-brand-based-hkt-complexity)
4. [Interconnection of Flaws](#interconnection-of-flaws)
5. [Redesign Proposals](#redesign-proposals)
   - [Proposal A: Independent Types with FnOnce](#proposal-a-independent-types-with-fnonce)
   - [Proposal B: Effect-Based Error Handling](#proposal-b-effect-based-error-handling)
   - [Proposal C: Standard Library Foundation](#proposal-c-standard-library-foundation)
   - [Proposal D: Separated Computation and Memoization](#proposal-d-separated-computation-and-memoization)
6. [Trade-off Analysis Matrix](#trade-off-analysis-matrix)
7. [Implementation Recommendations](#implementation-recommendations)
8. [Migration Strategy](#migration-strategy)

---

## Executive Summary

The existing analysis document ([`analysis.md`](./analysis.md)) identifies 12 issues with the `Lazy` type implementation. While these issues are valid, they represent **surface-level symptoms** rather than **root architectural causes**. This document demonstrates that:

1. **5-6 root architectural flaws** cause the 12 identified issues
2. **Incremental fixes** will not resolve the fundamental tensions in the design
3. **4 holistic redesign approaches** exist that would prevent most issues wholesale
4. The choice between approaches involves **explicit trade-offs** that should be consciously made

### Key Finding

The current design attempts to satisfy three competing goals:

1. **Library consistency** - Reuse of `CloneableFn`, brand patterns, and HKT emulation
2. **FP semantics** - Lazy evaluation with memoization and type class instances
3. **Rust idioms** - Ownership, thread safety, and error handling

These goals are in tension, and the current implementation inherits constraints from goal #1 that compromise goals #2 and #3.

---

## Assessment of Existing Analysis

### Issues Correctly Identified

| Issue # | Description                                | Category       |
| ------- | ------------------------------------------ | -------------- |
| 1       | Unnecessary `Clone` bound on `Lazy::Clone` | API ergonomics |
| 2       | Missing Functor/Applicative/Monad          | Feature gap    |
| 3       | Awkward `Fn(()) -> A` interface            | API ergonomics |
| 4       | Documentation mismatches                   | Documentation  |
| 5       | Error information loss during re-panic     | Error handling |
| 6       | No `LazyDefer` for `ArcLazyConfig`         | API asymmetry  |
| 7       | Verbose construction pattern               | API ergonomics |
| 8       | `AssertUnwindSafe` risk                    | Safety concern |
| 9       | Sequential error evaluation                | Error handling |
| 10      | `LazyError` incomplete payload capture     | Error handling |
| 11      | Missing `PartialEq`/`Eq`                   | Feature gap    |
| 12      | Missing `Default`                          | Feature gap    |

### What the Analysis Misses

The analysis treats each issue independently and proposes targeted fixes. However:

1. **Issues 3, 7** stem from the same root cause (CloneableFn reuse)
2. **Issues 5, 8, 9, 10** stem from the same root cause (panic-based error model)
3. **Issue 2** is unfixable without addressing CloneableFn constraints
4. **Issue 6** is a symptom of the config trait over-abstraction

The analysis does not ask: _"Why is the design this way, and could different foundational choices prevent these issues?"_

---

## Root Architectural Flaws

### Flaw 1: Panic-Based Error Propagation

#### Current Implementation

The `Lazy` type uses Rust's panic mechanism to handle thunk evaluation failures:

```rust
// In force() - catching panics
std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| thunk(())))
    .map_err(|payload| Arc::new(LazyError::from_panic(payload)))

// In append() and other composed operations - propagating errors via panic
Err(e) => std::panic::resume_unwind(Box::new(e.to_string())),
```

#### Why This Is Fundamentally Wrong

**1. Violates Rust idiom for error handling:**

Rust uses `Result<T, E>` for recoverable errors and reserves panics for:

- Programming bugs (e.g., array out of bounds)
- Unrecoverable states (e.g., memory allocation failure)
- Contract violations in `unsafe` code

Using panics for potentially recoverable thunk failures conflates these categories.

**2. Violates FP idiom for effect handling:**

In functional programming, effects (including failure) are represented explicitly in types:

- Haskell: `IO (Either Error a)`, `ExceptT Error IO a`
- Scala: `Try[A]`, `Either[E, A]`, `ZIO[R, E, A]`

The FP approach makes effects:

- Visible in type signatures
- Composable via standard combinators (`map`, `flatMap`)
- Testable without runtime surprises

**3. Creates cascading problems:**

| Symptom                              | Root Cause                                                        |
| ------------------------------------ | ----------------------------------------------------------------- |
| Issue 5: Error information loss      | Converting rich error types to strings for `resume_unwind`        |
| Issue 8: `AssertUnwindSafe` risk     | Panics bypass normal control flow, need explicit safety assertion |
| Issue 9: Sequential error evaluation | Cannot combine errors normally when using panic propagation       |
| Issue 10: Incomplete payload capture | Panic payloads are `Box<dyn Any>`, type info is erased            |

**4. Example of the problem:**

```rust
// User wants to compose two fallible lazy values
let a: RcLazy<Result<i32, MyError>> = ...;
let b: RcLazy<Result<i32, MyError>> = ...;

// Current approach: errors become panics, lose type info
let sum = Semigroup::append(a, b);  // If either panics, info is lost

// FP approach: errors compose naturally
let sum: RcLazy<Result<i32, MyError>> = lazy_map2(a, b, |x, y| {
    Ok(x? + y?)
});
```

#### Evidence from Code

```rust
// lazy.rs line 257-261
let x_val = match Lazy::force(&x) {
    Ok(v) => v.clone(),
    Err(e) => std::panic::resume_unwind(Box::new(e.to_string())),  // <-- loses error type
};
```

The pattern `e.to_string()` explicitly discards structured error information.

---

### Flaw 2: CloneableFn Reuse Constraint

#### Current Implementation

The `LazyConfig` trait requires thunks to implement:

```rust
type ThunkOf<'a, A>: Clone + Deref<Target: Fn(()) -> A>
```

This signature comes from reusing the library's `CloneableFn` trait:

```rust
// RcLazyConfig
type ThunkOf<'a, A> = <RcFnBrand as CloneableFn>::Of<'a, (), A>

// ArcLazyConfig
type ThunkOf<'a, A> = <ArcFnBrand as SendCloneableFn>::SendOf<'a, (), A>
```

#### Why This Is Problematic

**1. The `Fn(()) -> A` signature is semantically wrong:**

A lazy thunk is conceptually a zero-argument computation:

```
thunk : () -> A  // Haskell notation
thunk : => A     // Scala notation
```

The `Fn(()) -> A` signature means "a function taking unit as an argument":

```rust
thunk(())  // Must explicitly pass ()
|_| expr   // Must write |_| instead of ||
```

This is purely an artifact of `CloneableFn` supporting functions with arguments.

**2. The `Clone` requirement on thunks is overly restrictive:**

The library's `CloneableFn` wraps closures in `Rc`/`Arc` to make them `Clone`. This requires:

- All captured variables must be `Clone` (implicitly via the Rc/Arc)
- The mapper function `f` in `map(f, lazy)` must be `Clone`

However, Rust's standard `Fn` traits don't require `Clone`:

```rust
// Standard library Functor-like pattern
fn map<A, B, F: FnOnce(A) -> B>(self, f: F) -> Mapped<B>
```

The `Clone` requirement **prevents implementing standard `Functor`**:

```rust
// Library's Functor signature
fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Self::Of<'a, A>) -> Self::Of<'a, B>
where
    F: Fn(A) -> B + 'a;  // No Clone required!

// But Lazy needs:
// F: Fn(A) -> B + Clone + 'a
```

**3. Creates downstream issues:**

| Symptom                             | Root Cause                                       |
| ----------------------------------- | ------------------------------------------------ |
| Issue 3: Awkward `Fn(())` interface | `CloneableFn` designed for functions with params |
| Issue 7: Verbose construction       | Must wrap closure in `CloneableFn::new`          |
| Issue 2: Cannot implement `Functor` | `Functor::map` doesn't require `F: Clone`        |

---

### Flaw 3: Over-Abstracted Configuration Pattern

#### Current Implementation

The design uses a configuration trait hierarchy:

```rust
trait LazyConfig {
    type PtrBrand: RefCountedPointer + ThunkWrapper;
    type OnceBrand: Once;
    type FnBrand: CloneableFn;
    type ThunkOf<'a, A>;
}

trait LazySemigroup<A>: LazyConfig { ... }
trait LazyMonoid<A>: LazySemigroup<A> { ... }
trait LazyDefer<'a, A>: LazyConfig { ... }
```

With two implementations:

- `RcLazyConfig` - single-threaded
- `ArcLazyConfig` - thread-safe

#### Why This Is Over-Engineered

**1. The abstraction doesn't provide value:**

The two configurations differ in exactly three concrete types:

- Pointer: `Rc` vs `Arc`
- Once cell: `OnceCell` vs `OnceLock`
- Thunk wrapper: `RcFn` vs `ArcFn`

This could be achieved without traits:

```rust
// Alternative: just define two types
pub struct RcLazy<A> { /* uses Rc, OnceCell, RcFn */ }
pub struct ArcLazy<A> { /* uses Arc, OnceLock, ArcFn */ }
```

**2. Creates implementation duplication:**

Each trait method must be implemented separately for both configs:

```rust
impl<A> LazySemigroup<A> for RcLazyConfig { ... }  // Lines 218-266
impl<A: Send + Sync> LazySemigroup<A> for ArcLazyConfig { ... }  // Lines 433-481
```

The implementations are nearly identical, differing only in trait bounds.

**3. Causes API asymmetry:**

The config trait approach leads to Issue 6: `LazyDefer` cannot be implemented for `ArcLazyConfig` because the generic `Defer` trait doesn't require `Send + Sync`. This forces users to remember:

- Use `defer::<RcLazy<_>, RcFnBrand>(...)` for `RcLazy`
- Use `send_defer::<LazyBrand<ArcLazyConfig>, _, _>(...)` for `ArcLazy`

---

### Flaw 4: Implicit Shared Semantics

#### Current Implementation

```rust
impl<'a, Config: LazyConfig, A: Clone> Clone for Lazy<'a, Config, A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())  // Clones the Rc/Arc pointer, not the value
    }
}
```

#### Why This Is Problematic

**1. The `A: Clone` bound is misleading:**

Users see `A: Clone` in the signature and might expect `lazy.clone()` to create an independent copy with its own memoization state. Instead, it shares the memoization.

**2. No escape hatch:**

There's no way to create an independent `Lazy<A>` from an existing one. If a user wants two lazy values that compute the same thing but memoize separately, they must reconstruct from scratch:

```rust
// Cannot do: lazy.independent_clone()
// Must do: RcLazy::new(RcLazyConfig::new_thunk(|_| same_computation()))
```

**3. The behavior differs across languages:**

| Language  | Lazy Clone Semantics                    |
| --------- | --------------------------------------- |
| Haskell   | N/A (no mutation, sharing is invisible) |
| Scala     | `lazy val` cannot be cloned             |
| OCaml     | `Lazy.t` is mutable, sharing matters    |
| This impl | Shares memoization (Haskell-like)       |

The Haskell-like choice is valid but should be explicit in the API.

---

### Flaw 5: Brand-Based HKT Complexity

#### Current Implementation

The library emulates higher-kinded types (HKT) using "brand" types:

```rust
impl_kind! {
    impl<Config: LazyConfig> for LazyBrand<Config> {
        type Of<'a, A: 'a>: 'a = Lazy<'a, Config, A>;
    }
}
```

This enables type class instances like:

```rust
impl<'a, Config: LazySemigroup<A>, A: Semigroup + Clone + 'a> Semigroup for Lazy<'a, Config, A>
```

#### Why This Adds Complexity

**1. Multiple indirection layers:**

To understand `Lazy`, users must understand:

- `LazyBrand<Config>` - the type-level brand
- `LazyConfig` - the configuration trait
- `RcLazyConfig` / `ArcLazyConfig` - concrete configs
- How `impl_kind!` connects them

**2. Generic bounds become complex:**

```rust
// Simple version (what users want to write)
fn use_lazy<A>(lazy: RcLazy<A>) -> A

// Actual version (with full generics)
fn use_lazy<'a, Config: LazyConfig + LazyMonoid<A>, A: Monoid + Clone + 'a>(
    lazy: Lazy<'a, Config, A>
) -> A
```

**3. Error messages become cryptic:**

When type inference fails, errors reference `LazyBrand`, `Config::PtrBrand::CloneableOf`, etc., rather than concrete types.

---

## Interconnection of Flaws

The five root flaws are not independent. They form a causal chain:

```
┌──────────────────────────────────────────────────────────────────────────┐
│                     Library Architecture Decisions                       │
│  ┌─────────────────────┐    ┌──────────────────────┐                     │
│  │ CloneableFn trait   │    │ Brand-based HKT      │                     │
│  │ with Fn(A) -> B     │    │ emulation pattern    │                     │
│  └──────────┬──────────┘    └──────────┬───────────┘                     │
│             │                          │                                 │
│             ▼                          ▼                                 │
│  ┌─────────────────────────────────────────────────┐                     │
│  │           LazyConfig trait hierarchy             │◄─── Flaw 3         │
│  │  (abstracts over Rc/Arc, CloneableFn brands)    │                     │
│  └──────────────────────┬──────────────────────────┘                     │
│                         │                                                │
│                         ▼                                                │
│  ┌─────────────────────────────────────────────────┐                     │
│  │        Thunk type: Fn(()) -> A + Clone          │◄─── Flaw 2          │
│  └──────────┬───────────────────────────┬──────────┘                     │
│             │                           │                                │
│             ▼                           ▼                                │
│  ┌────────────────────┐    ┌───────────────────────┐                     │
│  │ Awkward |_| syntax │    │ Cannot impl Functor   │                     │
│  │ (Issue 3, 7)       │    │ (Issue 2)             │                     │
│  └────────────────────┘    └───────────────────────┘                     │
│                                                                          │
│  ┌─────────────────────────────────────────────────┐                     │
│  │   Panic-based error propagation                 │◄─── Flaw 1          │
│  └──────────────────────┬──────────────────────────┘                     │
│                         │                                                │
│                         ▼                                                │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │ Error info loss │ AssertUnwindSafe │ Sequential errors │ Payload    │ │
│  │ (Issue 5)       │ (Issue 8)        │ (Issue 9)         │ (Issue 10) │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─────────────────────────────────────────────────┐                     │
│  │   Clone bound + shared semantics                │◄─── Flaw 4          │
│  └──────────────────────┬──────────────────────────┘                     │
│                         │                                                │
│                         ▼                                                │
│  ┌────────────────────────────────────────────────┐                      │
│  │  Misleading Clone bound (Issue 1)              │                      │
│  │  No independent copy mechanism                 │                      │
│  └────────────────────────────────────────────────┘                      │
└──────────────────────────────────────────────────────────────────────────┘
```

**Key insight**: Fixing surface issues without addressing root flaws will leave the fundamental tensions unresolved. For example:

- Adding a `delay` constructor (Issue 3 fix) still requires `Clone` on the closure
- Adding `LazyFunctor` trait (Issue 2 workaround) doesn't give standard `Functor`
- Improving `LazyError` (Issue 10 fix) doesn't change the panic propagation model

---

## Redesign Proposals

### Proposal A: Independent Types with FnOnce

#### Philosophy

**Abandon the config trait abstraction. Create two independent, self-contained types that use `FnOnce` instead of `Fn + Clone`.**

#### Implementation

````rust
use std::cell::{OnceCell, UnsafeCell};
use std::rc::Rc;
use std::sync::{Arc, OnceLock};

// ============================================================================
// RcLazy - Single-threaded lazy value
// ============================================================================

/// A lazily-computed, memoized value for single-threaded contexts.
///
/// # Semantics
///
/// - Thunks are `FnOnce` - no Clone requirement on the closure
/// - Cloning creates a shared reference (Haskell-like semantics)
/// - Errors are represented explicitly via `Result`, not panics
///
/// # Examples
///
/// ```
/// let lazy = RcLazy::new(|| expensive_computation());
/// let value = lazy.force();  // Computes once
/// let value2 = lazy.force(); // Returns cached result
/// ```
pub struct RcLazy<A> {
    inner: Rc<LazyCell<A>>,
}

struct LazyCell<A> {
    state: OnceCell<A>,
    thunk: UnsafeCell<Option<Box<dyn FnOnce() -> A>>>,
}

impl<A> RcLazy<A> {
    /// Creates a new lazy value from a thunk.
    ///
    /// The thunk is a zero-argument closure that will be called at most once.
    ///
    /// # Examples
    ///
    /// ```
    /// let lazy = RcLazy::new(|| {
    ///     println!("Computing...");
    ///     42
    /// });
    /// // "Computing..." not yet printed
    /// ```
    pub fn new<F: FnOnce() -> A + 'static>(thunk: F) -> Self {
        Self {
            inner: Rc::new(LazyCell {
                state: OnceCell::new(),
                thunk: UnsafeCell::new(Some(Box::new(thunk))),
            }),
        }
    }

    /// Forces evaluation and returns a reference to the computed value.
    ///
    /// If the value has already been computed, returns the cached value.
    /// If the thunk panics, the panic is propagated (not caught).
    ///
    /// # Examples
    ///
    /// ```
    /// let lazy = RcLazy::new(|| 42);
    /// assert_eq!(*lazy.force(), 42);
    /// ```
    pub fn force(&self) -> &A {
        self.inner.state.get_or_init(|| {
            // SAFETY: We're inside get_or_init, which guarantees single execution.
            // No other code can access the thunk while we're taking it.
            let thunk = unsafe { &mut *self.inner.thunk.get() }
                .take()
                .expect("thunk already consumed");
            thunk()
        })
    }

    /// Returns `true` if the value has been computed.
    pub fn is_evaluated(&self) -> bool {
        self.inner.state.get().is_some()
    }

    /// Transforms the lazy value using a function.
    ///
    /// The transformation is itself lazy - neither the original value
    /// nor the result is computed until `force()` is called on the result.
    ///
    /// # Examples
    ///
    /// ```
    /// let lazy = RcLazy::new(|| 21);
    /// let doubled = lazy.map(|x| x * 2);
    /// assert_eq!(*doubled.force(), 42);
    /// ```
    pub fn map<B, F: FnOnce(&A) -> B + 'static>(self, f: F) -> RcLazy<B>
    where
        A: 'static,
    {
        let this = self;
        RcLazy::new(move || f(this.force()))
    }

    /// Composes two lazy values using a function.
    ///
    /// # Examples
    ///
    /// ```
    /// let a = RcLazy::new(|| 1);
    /// let b = RcLazy::new(|| 2);
    /// let sum = RcLazy::map2(a, b, |x, y| x + y);
    /// assert_eq!(*sum.force(), 3);
    /// ```
    pub fn map2<B, C, F>(a: RcLazy<A>, b: RcLazy<B>, f: F) -> RcLazy<C>
    where
        A: 'static,
        B: 'static,
        F: FnOnce(&A, &B) -> C + 'static,
    {
        RcLazy::new(move || f(a.force(), b.force()))
    }

    /// Flattens a nested lazy value.
    ///
    /// # Examples
    ///
    /// ```
    /// let lazy_lazy = RcLazy::new(|| RcLazy::new(|| 42));
    /// let flat = lazy_lazy.flatten();
    /// assert_eq!(*flat.force(), 42);
    /// ```
    pub fn flatten(self) -> RcLazy<A>
    where
        A: 'static,
        Self: 'static,
    {
        RcLazy::new(move || self.force().force().clone())
    }
}

impl<A> Clone for RcLazy<A> {
    /// Creates a new handle to the same lazy value.
    ///
    /// The clone shares memoization state with the original.
    /// This is Haskell-like semantics: lazy values are shared, not copied.
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

// ============================================================================
// ArcLazy - Thread-safe lazy value
// ============================================================================

/// A lazily-computed, memoized value for multi-threaded contexts.
///
/// # Thread Safety
///
/// `ArcLazy<A>` is `Send + Sync` when `A: Send + Sync`.
/// Multiple threads can race to force the lazy value; exactly one will succeed,
/// and all others will receive the computed result.
pub struct ArcLazy<A> {
    inner: Arc<SyncLazyCell<A>>,
}

struct SyncLazyCell<A> {
    state: OnceLock<A>,
    thunk: std::sync::Mutex<Option<Box<dyn FnOnce() -> A + Send>>>,
}

impl<A> ArcLazy<A> {
    /// Creates a new thread-safe lazy value from a thunk.
    ///
    /// The thunk must be `Send` to allow evaluation on any thread.
    pub fn new<F: FnOnce() -> A + Send + 'static>(thunk: F) -> Self {
        Self {
            inner: Arc::new(SyncLazyCell {
                state: OnceLock::new(),
                thunk: std::sync::Mutex::new(Some(Box::new(thunk))),
            }),
        }
    }

    /// Forces evaluation and returns a reference to the computed value.
    ///
    /// Thread-safe: if multiple threads call this concurrently, exactly one
    /// will execute the thunk, and all will receive the same result.
    pub fn force(&self) -> &A {
        self.inner.state.get_or_init(|| {
            let thunk = self.inner.thunk
                .lock()
                .expect("mutex poisoned")
                .take()
                .expect("thunk already consumed");
            thunk()
        })
    }

    /// Transforms the lazy value using a function.
    pub fn map<B, F: FnOnce(&A) -> B + Send + 'static>(self, f: F) -> ArcLazy<B>
    where
        A: Send + Sync + 'static,
    {
        let this = self;
        ArcLazy::new(move || f(this.force()))
    }
}

impl<A> Clone for ArcLazy<A> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

// SAFETY: ArcLazy is Send + Sync when A is Send + Sync
unsafe impl<A: Send + Sync> Send for ArcLazy<A> {}
unsafe impl<A: Send + Sync> Sync for ArcLazy<A> {}
````

#### Trade-offs

| Aspect                   | Benefit                      | Cost                                        |
| ------------------------ | ---------------------------- | ------------------------------------------- |
| `FnOnce` thunks          | No `Clone` bound on closures | Thunks cannot be cloned (by design)         |
| No config trait          | Simpler mental model         | Code duplication between `RcLazy`/`ArcLazy` |
| Direct panic propagation | No `catch_unwind` overhead   | User must handle panics if desired          |
| Reference-based `map`    | No `Clone` on `A` needed     | `map` takes `&A`, not `A`                   |
| No brand types           | Simpler generics             | Cannot use library's `Functor` trait\*      |

_\*Note: The library's `Functor` trait requires `Fn(A) -> B`. Shared lazy types can only provide `&A`, making implementation impossible without `A: Clone` bounds, which the trait does not support._

#### Issues Addressed

- ✅ Issue 1: No `Clone` bound on `Clone` impl
- ✅ Issue 2: Can implement `map` (though not library's `Functor`)
- ✅ Issue 3: Uses `|| expr`, not `|_| expr`
- ✅ Issue 7: Simple `RcLazy::new(|| ...)`
- ⚠️ Issue 5: Panics propagate directly (different design choice)
- ✅ Issue 6: No config trait asymmetry
- ✅ Issues 8, 10: No `catch_unwind` needed

---

### Proposal B: Effect-Based Error Handling

#### Philosophy

**Introduce explicit error handling via `Result` types. Create both infallible (`Lazy`) and fallible (`TryLazy`) variants.**

#### Implementation

````rust
use std::cell::OnceCell;
use std::rc::Rc;

// ============================================================================
// Infallible Lazy
// ============================================================================

/// A lazy value that cannot fail.
///
/// If the thunk might panic, consider using `TryLazy` instead.
pub struct Lazy<A> {
    inner: Rc<LazyCell<A>>,
}

struct LazyCell<A> {
    state: OnceCell<A>,
    thunk: std::cell::Cell<Option<Box<dyn FnOnce() -> A>>>,
}

impl<A> Lazy<A> {
    pub fn new<F: FnOnce() -> A + 'static>(thunk: F) -> Self {
        Self {
            inner: Rc::new(LazyCell {
                state: OnceCell::new(),
                thunk: std::cell::Cell::new(Some(Box::new(thunk))),
            }),
        }
    }

    pub fn force(&self) -> &A {
        self.inner.state.get_or_init(|| {
            let thunk = self.inner.thunk.take().expect("thunk consumed");
            thunk()
        })
    }

    /// Standard Functor map - no Clone bound on F!
    pub fn map<B, F: FnOnce(&A) -> B + 'static>(self, f: F) -> Lazy<B>
    where
        A: 'static,
    {
        Lazy::new(move || f(self.force()))
    }
}

impl<A> Clone for Lazy<A> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

// ============================================================================
// Fallible TryLazy
// ============================================================================

/// A lazy value that may fail with an error of type `E`.
///
/// Errors are represented explicitly in the type, not via panics.
///
/// # Examples
///
/// ```
/// let lazy: TryLazy<i32, ParseError> = TryLazy::new(|| {
///     "42".parse::<i32>().map_err(ParseError::from)
/// });
///
/// match lazy.force() {
///     Ok(n) => println!("Got: {}", n),
///     Err(e) => println!("Failed: {}", e),
/// }
/// ```
pub struct TryLazy<A, E> {
    inner: Rc<TryLazyCell<A, E>>,
}

struct TryLazyCell<A, E> {
    state: OnceCell<Result<A, E>>,
    thunk: std::cell::Cell<Option<Box<dyn FnOnce() -> Result<A, E>>>>,
}

impl<A, E> TryLazy<A, E> {
    /// Creates a fallible lazy value.
    pub fn new<F: FnOnce() -> Result<A, E> + 'static>(thunk: F) -> Self {
        Self {
            inner: Rc::new(TryLazyCell {
                state: OnceCell::new(),
                thunk: std::cell::Cell::new(Some(Box::new(thunk))),
            }),
        }
    }

    /// Creates a lazy value that always succeeds with the given value.
    pub fn pure(value: A) -> Self
    where
        A: 'static,
        E: 'static,
    {
        Self::new(move || Ok(value))
    }

    /// Creates a lazy value that always fails with the given error.
    pub fn fail(error: E) -> Self
    where
        A: 'static,
        E: 'static,
    {
        Self::new(move || Err(error))
    }

    /// Forces evaluation and returns a reference to the result.
    pub fn force(&self) -> Result<&A, &E> {
        let result = self.inner.state.get_or_init(|| {
            let thunk = self.inner.thunk.take().expect("thunk consumed");
            thunk()
        });
        result.as_ref()
    }

    /// Returns `true` if the lazy value has been forced and succeeded.
    pub fn is_ok(&self) -> bool {
        matches!(self.inner.state.get(), Some(Ok(_)))
    }

    /// Returns `true` if the lazy value has been forced and failed.
    pub fn is_err(&self) -> bool {
        matches!(self.inner.state.get(), Some(Err(_)))
    }

    /// Maps a function over the success value.
    ///
    /// The transformation is lazy.
    pub fn map<B, F: FnOnce(&A) -> B + 'static>(self, f: F) -> TryLazy<B, E>
    where
        A: 'static,
        E: Clone + 'static,
    {
        TryLazy::new(move || self.force().map(f).map_err(Clone::clone))
    }

    /// Maps a function over the error value.
    pub fn map_err<E2, F: FnOnce(&E) -> E2 + 'static>(self, f: F) -> TryLazy<A, E2>
    where
        A: Clone + 'static,
        E: 'static,
    {
        TryLazy::new(move || self.force().map(Clone::clone).map_err(f))
    }

    /// Chains two fallible lazy computations.
    ///
    /// If the first succeeds, `f` is called to produce the next lazy value.
    /// If the first fails, the error is propagated.
    pub fn and_then<B, F>(self, f: F) -> TryLazy<B, E>
    where
        A: 'static,
        E: Clone + 'static,
        F: FnOnce(&A) -> TryLazy<B, E> + 'static,
        B: 'static,
    {
        TryLazy::new(move || {
            match self.force() {
                Ok(a) => f(a).force().map(|b| b.clone()).map_err(Clone::clone),
                Err(e) => Err(e.clone()),
            }
        })
    }

    /// Combine two lazy values, propagating errors.
    pub fn map2<B, C, F>(a: Self, b: TryLazy<B, E>, f: F) -> TryLazy<C, E>
    where
        A: 'static,
        B: 'static,
        E: Clone + 'static,
        F: FnOnce(&A, &B) -> C + 'static,
    {
        TryLazy::new(move || {
            let a_result = a.force().map_err(Clone::clone)?;
            let b_result = b.force().map_err(Clone::clone)?;
            Ok(f(a_result, b_result))
        })
    }
}

impl<A, E> Clone for TryLazy<A, E> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

// ============================================================================
// Conversion Between Lazy and TryLazy
// ============================================================================

impl<A> Lazy<A> {
    /// Converts to a fallible lazy that always succeeds.
    pub fn into_try<E: 'static>(self) -> TryLazy<A, E>
    where
        A: Clone + 'static,
    {
        TryLazy::new(move || Ok(self.force().clone()))
    }
}

impl<A, E> TryLazy<A, E> {
    /// Wraps a potentially panicking thunk, capturing panics as errors.
    ///
    /// # Type Parameters
    ///
    /// * `F`: Any panic payload can be captured
    ///
    /// # Examples
    ///
    /// ```
    /// let lazy = TryLazy::<i32, String>::catch_unwind(|| {
    ///     if condition { panic!("oops"); }
    ///     42
    /// });
    /// ```
    pub fn catch_unwind<F: FnOnce() -> A + std::panic::UnwindSafe + 'static>(
        thunk: F,
    ) -> TryLazy<A, String>
    where
        A: 'static,
    {
        TryLazy::new(move || {
            std::panic::catch_unwind(thunk)
                .map_err(|payload| {
                    if let Some(s) = payload.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = payload.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    }
                })
        })
    }
}
````

#### Trade-offs

| Aspect                    | Benefit                     | Cost                                   |
| ------------------------- | --------------------------- | -------------------------------------- |
| Explicit `Result<A, E>`   | Type-safe error handling    | More verbose for infallible thunks     |
| `TryLazy` separate type   | Clear semantics             | Two types to learn                     |
| `catch_unwind` optional   | User chooses error handling | Panics escape by default               |
| Composable via `and_then` | FP-style sequencing         | Requires `Clone` for error propagation |

#### Issues Addressed

- ✅ Issue 5: Errors preserved in `Result`
- ✅ Issue 8: No implicit `AssertUnwindSafe`
- ✅ Issue 9: Can compose errors via `map2`, `and_then`
- ✅ Issue 10: Custom error types, not just strings

---

### Proposal C: Standard Library Foundation

#### Philosophy

**Build on Rust's standard library `LazyCell` and `LazyLock` types (stabilized in Rust 1.80). Add FP-style combinators on top.**

#### Implementation

```rust
use std::cell::LazyCell;
use std::sync::LazyLock;

// ============================================================================
// FpLazy - Functional wrapper over std::cell::LazyCell
// ============================================================================

/// A lazy value with functional programming combinators.
///
/// Built on `std::cell::LazyCell` for correctness and performance.
pub struct FpLazy<A> {
    // LazyCell handles all the memoization logic
    cell: LazyCell<A, Box<dyn FnOnce() -> A>>,
}

impl<A> FpLazy<A> {
    /// Creates a new lazy value.
    pub fn new<F: FnOnce() -> A + 'static>(f: F) -> Self {
        Self {
            cell: LazyCell::new(Box::new(f)),
        }
    }

    /// Creates a lazy value that is already computed.
    pub fn now(value: A) -> Self
    where
        A: 'static,
    {
        // Create a LazyCell and immediately initialize it
        let cell = LazyCell::new(Box::new(move || value) as Box<dyn FnOnce() -> A>);
        LazyCell::force(&cell);  // Force immediate evaluation
        Self { cell }
    }

    /// Forces evaluation and returns a reference.
    pub fn force(&self) -> &A {
        LazyCell::force(&self.cell)
    }

    /// Maps a function over the lazy value.
    pub fn map<B, F: FnOnce(&A) -> B + 'static>(self, f: F) -> FpLazy<B>
    where
        A: 'static,
    {
        FpLazy::new(move || f(self.force()))
    }

    /// Applicative: applies a lazy function to a lazy value.
    pub fn ap<B, F>(ff: FpLazy<F>, fa: FpLazy<A>) -> FpLazy<B>
    where
        A: 'static,
        F: FnOnce(&A) -> B + 'static,
    {
        FpLazy::new(move || {
            let f = ff.force();
            // Note: This requires cloning F to use it, which we can't do with FnOnce
            // This is a limitation - we'd need to restructure
            todo!("Applicative for FnOnce is complex")
        })
    }

    /// Monadic bind (flatMap).
    pub fn flat_map<B, F: FnOnce(&A) -> FpLazy<B> + 'static>(self, f: F) -> FpLazy<B>
    where
        A: 'static,
        B: Clone + 'static,
    {
        FpLazy::new(move || f(self.force()).force().clone())
    }
}

// Note: LazyCell is not Clone by default, so we need a wrapper approach
// for sharing semantics.

// ============================================================================
// SharedLazy - Shared lazy value using Rc<LazyCell>
// ============================================================================

use std::rc::Rc;

/// A shared lazy value - clones share the same memoization.
pub struct SharedLazy<A> {
    inner: Rc<LazyCell<A, Box<dyn FnOnce() -> A>>>,
}

impl<A> SharedLazy<A> {
    pub fn new<F: FnOnce() -> A + 'static>(f: F) -> Self {
        Self {
            inner: Rc::new(LazyCell::new(Box::new(f))),
        }
    }

    pub fn force(&self) -> &A {
        LazyCell::force(&*self.inner)
    }

    pub fn map<B, F: FnOnce(&A) -> B + 'static>(self, f: F) -> SharedLazy<B>
    where
        A: 'static,
    {
        let this = self;
        SharedLazy::new(move || f(this.force()))
    }
}

impl<A> Clone for SharedLazy<A> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

// ============================================================================
// SyncLazy - Thread-safe version using LazyLock
// ============================================================================

use std::sync::Arc;

/// A thread-safe shared lazy value.
pub struct SyncLazy<A> {
    inner: Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send>>>,
}

impl<A: Send + Sync> SyncLazy<A> {
    pub fn new<F: FnOnce() -> A + Send + 'static>(f: F) -> Self {
        Self {
            inner: Arc::new(LazyLock::new(Box::new(f))),
        }
    }

    pub fn force(&self) -> &A {
        LazyLock::force(&*self.inner)
    }
}

impl<A> Clone for SyncLazy<A> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

unsafe impl<A: Send + Sync> Send for SyncLazy<A> {}
unsafe impl<A: Send + Sync> Sync for SyncLazy<A> {}
```

#### Trade-offs

| Aspect         | Benefit                  | Cost                                 |
| -------------- | ------------------------ | ------------------------------------ |
| Uses std types | Battle-tested, optimized | Requires Rust 1.80+                  |
| Thin wrapper   | Minimal code to maintain | Less control over internals          |
| `FnOnce`       | No Clone bounds          | Cannot implement Applicative naively |

#### Issues Addressed

- ✅ Issue 1, 3, 7: Simple API with `FnOnce`
- ⚠️ Issue 2: `map` works, but `Applicative` is tricky with `FnOnce`
- Defers to std for correctness of memoization

---

### Proposal D: Separated Computation and Memoization

#### Philosophy

**Decompose lazy values into orthogonal components: the computation itself and the memoization strategy. This allows mixing and matching.**

#### Implementation

```rust
// ============================================================================
// Core Abstractions
// ============================================================================

/// A computation that can be run once to produce a value.
///
/// This is the "what to compute" - separate from "how to cache it".
pub struct Thunk<A> {
    compute: Box<dyn FnOnce() -> A>,
}

impl<A> Thunk<A> {
    pub fn new<F: FnOnce() -> A + 'static>(f: F) -> Self {
        Self { compute: Box::new(f) }
    }

    /// Runs the computation, consuming the thunk.
    pub fn run(self) -> A {
        (self.compute)()
    }

    /// Transforms the thunk's output.
    pub fn map<B, F: FnOnce(A) -> B + 'static>(self, f: F) -> Thunk<B>
    where
        A: 'static,
    {
        Thunk::new(move || f(self.run()))
    }

    /// Chains computations.
    pub fn and_then<B, F: FnOnce(A) -> Thunk<B> + 'static>(self, f: F) -> Thunk<B>
    where
        A: 'static,
        B: 'static,
    {
        Thunk::new(move || f(self.run()).run())
    }
}

/// A memoization cell - stores a value once computed.
///
/// This is the "how to cache" - separate from "what to compute".
pub trait Memo<A> {
    fn get(&self) -> Option<&A>;
    fn set(&self, value: A) -> &A;
    fn get_or_init<F: FnOnce() -> A>(&self, f: F) -> &A;
}

// ============================================================================
// Memoization Strategies
// ============================================================================

use std::cell::OnceCell;
use std::sync::OnceLock;

/// Single-threaded memoization.
pub struct LocalMemo<A>(OnceCell<A>);

impl<A> LocalMemo<A> {
    pub fn new() -> Self { Self(OnceCell::new()) }
}

impl<A> Memo<A> for LocalMemo<A> {
    fn get(&self) -> Option<&A> { self.0.get() }
    fn set(&self, value: A) -> &A {
        let _ = self.0.set(value);
        self.0.get().unwrap()
    }
    fn get_or_init<F: FnOnce() -> A>(&self, f: F) -> &A {
        self.0.get_or_init(f)
    }
}

/// Thread-safe memoization.
pub struct SyncMemo<A>(OnceLock<A>);

impl<A> SyncMemo<A> {
    pub fn new() -> Self { Self(OnceLock::new()) }
}

impl<A> Memo<A> for SyncMemo<A> {
    fn get(&self) -> Option<&A> { self.0.get() }
    fn set(&self, value: A) -> &A {
        let _ = self.0.set(value);
        self.0.get().unwrap()
    }
    fn get_or_init<F: FnOnce() -> A>(&self, f: F) -> &A {
        self.0.get_or_init(f)
    }
}

// ============================================================================
// Lazy = Thunk + Memo
// ============================================================================

use std::rc::Rc;
use std::cell::UnsafeCell;

/// A lazy value = a thunk + a memo, combined in a shared container.
pub struct Lazy<A, M: Memo<A> = LocalMemo<A>> {
    inner: Rc<LazyInner<A, M>>,
}

struct LazyInner<A, M: Memo<A>> {
    memo: M,
    thunk: UnsafeCell<Option<Thunk<A>>>,
}

impl<A> Lazy<A, LocalMemo<A>> {
    pub fn new<F: FnOnce() -> A + 'static>(f: F) -> Self {
        Self {
            inner: Rc::new(LazyInner {
                memo: LocalMemo::new(),
                thunk: UnsafeCell::new(Some(Thunk::new(f))),
            }),
        }
    }
}

impl<A, M: Memo<A>> Lazy<A, M> {
    pub fn force(&self) -> &A {
        self.inner.memo.get_or_init(|| {
            let thunk = unsafe { &mut *self.inner.thunk.get() }
                .take()
                .expect("thunk consumed");
            thunk.run()
        })
    }

    pub fn is_evaluated(&self) -> bool {
        self.inner.memo.get().is_some()
    }
}

impl<A, M: Memo<A>> Clone for Lazy<A, M> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

// ============================================================================
// Different Configurations Become Simple Type Aliases
// ============================================================================

/// Single-threaded lazy with local memoization.
pub type LocalLazy<A> = Lazy<A, LocalMemo<A>>;

// For thread-safe version, we need Arc instead of Rc
use std::sync::Arc;

pub struct SyncLazy<A> {
    inner: Arc<SyncLazyInner<A>>,
}

struct SyncLazyInner<A> {
    memo: SyncMemo<A>,
    thunk: std::sync::Mutex<Option<Thunk<A>>>,
}

impl<A: Send + 'static> SyncLazy<A> {
    pub fn new<F: FnOnce() -> A + Send + 'static>(f: F) -> Self {
        Self {
            inner: Arc::new(SyncLazyInner {
                memo: SyncMemo::new(),
                thunk: std::sync::Mutex::new(Some(Thunk::new(f))),
            }),
        }
    }

    pub fn force(&self) -> &A {
        self.inner.memo.get_or_init(|| {
            let thunk = self.inner.thunk
                .lock()
                .unwrap()
                .take()
                .expect("thunk consumed");
            thunk.run()
        })
    }
}

impl<A> Clone for SyncLazy<A> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}
```

#### Trade-offs

| Aspect             | Benefit                         | Cost                      |
| ------------------ | ------------------------------- | ------------------------- |
| Separated concerns | `Thunk` reusable independently  | More types to understand  |
| `Memo` trait       | Extensible caching strategies   | Trait indirection         |
| Composable thunks  | `Thunk::map`, `Thunk::and_then` | Separate from `Lazy::map` |

#### Issues Addressed

- ✅ Issue 2: `Thunk` has standard `map` and `and_then`
- ✅ Issue 3, 7: `Thunk::new(|| ...)` is natural
- ✅ Extensibility: Add new `Memo` implementations (weak refs, TTL cache, etc.)

---

## Trade-off Analysis Matrix

| Criterion               | Current    | Proposal A   | Proposal B    | Proposal C     | Proposal D       |
| ----------------------- | -----------| ------------ | ------------- | -------------- | ---------------- |
| **Simplicity**          | ❌ Complex  | ✅ Simple     | ⚠️ Two types   | ✅ Thin wrapper | ⚠️ Multiple parts |
| **Functor**             | ❌ Cannot   | ⚠️ Ref-based  | ⚠️ Ref-based   | ⚠️ Limited      | ✅ `Thunk::map`   |
| **Error handling**      | ❌ Panics   | ⚠️ Propagates | ✅ Explicit    | ⚠️ Propagates   | ⚠️ On thunk       |
| **Clone on closure**    | ❌ Required | ✅ Not needed | ✅ Not needed  | ✅ Not needed   | ✅ Not needed     |
| **Clone on A**          | ❌ Required | ⚠️ For map    | ⚠️ For compose | ⚠️ For flat_map | ⚠️ For compose    |
| **Thread safety**       | ✅ ArcLazy  | ✅ ArcLazy    | ⚠️ TrySyncLazy | ✅ SyncLazy     | ✅ SyncLazy       |
| **Library integration** | ✅ Brands   | ❌ Standalone | ❌ Standalone  | ❌ Standalone   | ⚠️ Partial        |
| **API ergonomics**      | ❌ Verbose  | ✅ Natural    | ✅ Natural     | ✅ Natural      | ✅ Natural        |
| **Std compatibility**   | ❌ Custom   | ⚠️ Custom     | ⚠️ Custom      | ✅ Uses std     | ⚠️ Custom         |
| **Extensibility**       | ❌ Fixed    | ⚠️ Limited    | ⚠️ Limited     | ⚠️ Limited      | ✅ Memo trait     |

### Scoring

- ✅ = Strong advantage (2 points)
- ⚠️ = Acceptable / trade-off (1 point)
- ❌ = Disadvantage (0 points)

| Proposal   | Total Score |
| ---------- | ----------- |
| Current    | 5           |
| Proposal A | 14          |
| Proposal B | 13          |
| Proposal C | 14          |
| Proposal D | 14          |

---

## Implementation Recommendations

### Recommended Approach: Hybrid of A, B, and C

Combine the simplicity of Proposal A, the error handling of Proposal B, and the standard library internals of Proposal C:

1. **API Structure (from A & B):** Independent types (`Lazy`, `SyncLazy`) with `FnOnce` thunks and optional `Try` variants.
2. **Internals (from C):** Use `std::cell::LazyCell` and `std::sync::LazyLock` (available in Rust 1.80+) to handle the memoization logic safely and efficiently.

```rust
// Core types: simple, no config traits, backed by std
pub struct Lazy<A> {
    inner: std::cell::LazyCell<A, Box<dyn FnOnce() -> A>>
}

pub struct SyncLazy<A> {
    inner: std::sync::LazyLock<A, Box<dyn FnOnce() -> A + Send>>
}

// Fallible variants for explicit error handling
pub struct TryLazy<A, E> { /* Result-based */ }
pub struct TrySyncLazy<A, E> { /* Thread-safe Result-based */ }

// All use FnOnce, no Clone bounds on closures
// Natural || syntax
// Optional catch_unwind for panic capture
```

### Implementation Priority

1. **Phase 1**: Implement `Lazy<A>` and `SyncLazy<A>` with `FnOnce`
2. **Phase 2**: Add `map`, `map2`, `flatten` combinators
3. **Phase 3**: Implement `TryLazy<A, E>` and `TrySyncLazy<A, E>`
4. **Phase 4**: Bridge to library's type classes if needed

### Breaking Changes

The recommended approach is a **breaking change** from the current API:

| Current API                                       | New API               |
| ------------------------------------------------- | --------------------- |
| `RcLazy::new(RcLazyConfig::new_thunk(\|_\| ...))` | `Lazy::new(\|\| ...)` |
| `Lazy::force(&lazy)`                              | `lazy.force()`        |
| `LazyConfig` trait                                | Removed               |
| `LazyBrand<Config>`                               | Removed               |

---

## Conclusion

The existing analysis document identifies valid issues but treats them as independent problems with incremental solutions. This architectural critique demonstrates that:

1. **12 surface issues trace to 5 root flaws**
2. **Root flaws are interconnected** through library-wide architectural decisions
3. **4 holistic redesign approaches** exist that would address most issues wholesale
4. **The recommended hybrid approach** (A + B + C elements) scores significantly higher than the current design

The choice of approach should be made explicitly, weighing:

- **Library consistency** vs **standalone simplicity**
- **Panic propagation** vs **explicit errors**
- **Clone bounds** vs **FnOnce flexibility**

Any chosen approach should be documented with its rationale, so future maintainers understand the trade-offs that were accepted.
