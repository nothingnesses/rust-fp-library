# Lazy Type Analysis

This document provides a comprehensive analysis of the `Lazy` type implementation in `fp-library/src/types/lazy.rs`, identifying design issues, limitations, and detailed approaches for improvement.

## Table of Contents

1. [Overview](#overview)
2. [Architecture Summary](#architecture-summary)
3. [Major Issues](#major-issues)
   - [Issue 1: Unnecessary Clone Bound](#issue-1-unnecessary-clone-bound-on-lazyclone)
   - [Issue 2: Missing Functor Implementation](#issue-2-missing-functorapplicativemonad-implementations)
   - [Issue 3: Awkward Fn(()) Interface](#issue-3-awkward-fn---a-interface)
   - [Issue 4: Documentation Mismatches](#issue-4-documentation-example-mismatches)
4. [Moderate Issues](#moderate-issues)
   - [Issue 5: Error Information Loss](#issue-5-error-information-loss-during-re-panic)
   - [Issue 6: No LazyDefer for ArcLazyConfig](#issue-6-no-lazydefer-for-arclazyconfig)
   - [Issue 7: Verbose Construction Pattern](#issue-7-verbose-construction-pattern)
5. [Minor Issues](#minor-issues)
   - [Issue 8: AssertUnwindSafe Risk](#issue-8-assertunwindsafe-risk)
   - [Issue 9: Sequential Error Evaluation](#issue-9-sequential-error-evaluation-in-semigroup)
6. [What's Well-Designed](#whats-well-designed)
7. [Summary of Recommendations](#summary-of-recommendations)

---

## Overview

The `Lazy` type in `fp-library` provides lazy evaluation with memoization, supporting both single-threaded (`RcLazy`) and thread-safe (`ArcLazy`) variants. The design uses a configuration-based approach through the `LazyConfig` trait.

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────────┐
│                         LazyConfig (trait)                       │
│  - PtrBrand: RefCountedPointer + ThunkWrapper                   │
│  - OnceBrand: Once                                               │
│  - FnBrand: CloneableFn                                          │
│  - ThunkOf<A>: Clone + Deref<Target: Fn(()) -> A>               │
└─────────────────────────────────────────────────────────────────┘
                    │                           │
                    ▼                           ▼
    ┌───────────────────────┐     ┌───────────────────────┐
    │     RcLazyConfig      │     │     ArcLazyConfig     │
    │  PtrBrand = RcBrand   │     │  PtrBrand = ArcBrand  │
    │  OnceBrand = OnceCell │     │  OnceBrand = OnceLock │
    │  FnBrand = RcFnBrand  │     │  FnBrand = ArcFnBrand │
    └───────────────────────┘     └───────────────────────┘
                    │                           │
                    ▼                           ▼
         ┌─────────────────┐         ┌─────────────────┐
         │  RcLazy<'a, A>  │         │ ArcLazy<'a, A>  │
         │  (not Send)     │         │ (Send + Sync)   │
         └─────────────────┘         └─────────────────┘
```

**Current Type Class Implementations:**

- ✅ `Semigroup` (via `LazySemigroup`)
- ✅ `Monoid` (via `LazyMonoid`)
- ✅ `Defer` (via `LazyDefer`, RcLazy only)
- ✅ `SendDefer` (ArcLazy only)
- ❌ `Functor` - **Missing**
- ❌ `Applicative` - **Missing**
- ❌ `Monad` - **Missing**

---

## Major Issues

### Issue 1: Unnecessary Clone Bound on `Lazy::Clone`

**Location:** Line 879

**Current Implementation:**

```rust
impl<'a, Config: LazyConfig, A: Clone> Clone for Lazy<'a, Config, A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())  // Only clones the Rc/Arc pointer!
    }
}
```

**Problem:**
The implementation requires `A: Clone`, but the actual cloning operation only clones the reference-counted pointer (`Rc` or `Arc`), not the value `A`. This unnecessarily restricts which types can have cloneable `Lazy` wrappers.

**Impact:**

- Cannot create `Lazy<NonCloneableType>` and clone the lazy wrapper
- Forces users to add `Clone` bounds even when semantically unnecessary
- Breaks the principle that `Lazy<A>` should work for any `A`

#### Approach 1: Remove the Clone Bound (Recommended)

```rust
impl<'a, Config: LazyConfig, A> Clone for Lazy<'a, Config, A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
```

**Trade-offs:**

- ✅ Allows `Lazy` to wrap non-cloneable types
- ✅ Zero performance impact
- ✅ Semantically correct
- ✅ Simple change
- ⚠️ May require updating call sites that relied on the transitive `Clone` bound

#### Approach 2: Conditional Clone Implementation

Use a marker trait to provide different clone semantics:

```rust
// Blanket impl for all A
impl<'a, Config: LazyConfig, A> Clone for Lazy<'a, Config, A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Separate method for cloning the inner value
impl<'a, Config: LazyConfig, A: Clone> Lazy<'a, Config, A> {
    /// Clones the lazy value and its computed result (if any).
    /// Creates a new independent lazy value with a cloned thunk.
    pub fn deep_clone(&self) -> Self
    where
        Config::ThunkOf<'a, A>: Clone,
    {
        // Implementation that creates a new independent lazy value
        // rather than sharing the memoization state
    }
}
```

**Trade-offs:**

- ✅ Provides both pointer-clone and deep-clone semantics
- ✅ Clear API distinction
- ⚠️ More complex API surface
- ⚠️ `deep_clone` may be confusing given shared semantics

**Recommendation:** Approach 1 is preferred for its simplicity and correctness.

---

### Issue 2: Missing Functor/Applicative/Monad Implementations

**Problem:**
The `Lazy` type implements `Semigroup`, `Monoid`, and `Defer`, but lacks the fundamental FP type class hierarchy:

- No `Functor` (no `map` operation)
- No `Applicative` (no `pure` with `apply`)
- No `Monad` (no `bind`/`flatMap`)

Without these, users cannot compose lazy computations without forcing evaluation.

**Impact:**

- Cannot transform `Lazy<A>` to `Lazy<B>` without forcing
- Cannot sequence lazy operations compositionally
- Significantly limits usefulness as an FP building block

#### Approach 1: Implement Functor with Lazy map (Recommended)

```rust
use crate::classes::functor::Functor;

impl<'a, Config: LazyConfig> Functor for LazyBrand<Config>
where
    Config: 'static,
{
    /// Maps a function over a lazy value, returning a new lazy value.
    ///
    /// The transformation is itself lazy - neither the original value
    /// nor the mapped result is computed until forced.
    ///
    /// ### Type Signature
    ///
    /// `forall config a b. (a -> b, Lazy config a) -> Lazy config b`
    fn map<'b, FnBrand, A, B>(
        f: <FnBrand as CloneableFn>::Of<'b, A, B>,
        fa: Lazy<'b, Config, A>,
    ) -> Lazy<'b, Config, B>
    where
        FnBrand: CloneableFn + 'b,
        A: Clone + 'b,
        B: Clone + 'b,
    {
        let thunk = Config::new_thunk(move |_| {
            match Lazy::force(&fa) {
                Ok(a) => f(a.clone()),
                Err(e) => std::panic::resume_unwind(Box::new(e.to_string())),
            }
        });
        Lazy::new(thunk)
    }
}
```

**Trade-offs:**

- ✅ Enables functional composition of lazy values
- ✅ Preserves laziness (computation deferred until forced)
- ✅ Follows standard FP patterns
- ⚠️ Requires `A: Clone` to extract value for mapping
- ⚠️ Creates a chain of lazy values (may accumulate thunks)

#### Approach 2: Implement Functor with Reference-Based map

```rust
impl<'a, Config: LazyConfig> Functor for LazyBrand<Config> {
    fn map<'b, FnBrand, A, B>(
        f: <FnBrand as CloneableFn>::Of<'b, &A, B>,  // Note: &A instead of A
        fa: Lazy<'b, Config, A>,
    ) -> Lazy<'b, Config, B>
    where
        FnBrand: CloneableFn + 'b,
        A: 'b,
        B: Clone + 'b,
    {
        let thunk = Config::new_thunk(move |_| {
            match Lazy::force(&fa) {
                Ok(a) => f(a),  // Pass reference directly
                Err(e) => std::panic::resume_unwind(Box::new(e.to_string())),
            }
        });
        Lazy::new(thunk)
    }
}
```

**Trade-offs:**

- ✅ No `Clone` requirement on input type `A`
- ✅ More efficient for large types
- ⚠️ Different signature from standard `Functor` (`&A` vs `A`)
- ⚠️ May not compose well with other library components
- ⚠️ Breaks the standard `Functor` contract

#### Approach 3: Provide Both via Separate Traits

```rust
/// Standard Functor with clone semantics
trait LazyFunctor<Config: LazyConfig>: Functor {
    fn lazy_map<'a, A, B, F>(f: F, fa: Lazy<'a, Config, A>) -> Lazy<'a, Config, B>
    where
        A: Clone + 'a,
        B: Clone + 'a,
        F: Fn(A) -> B + Clone + 'a;
}

/// Reference-based mapping without clone
trait LazyFunctorRef<Config: LazyConfig> {
    fn lazy_map_ref<'a, A, B, F>(f: F, fa: Lazy<'a, Config, A>) -> Lazy<'a, Config, B>
    where
        A: 'a,
        B: Clone + 'a,
        F: Fn(&A) -> B + Clone + 'a;
}
```

**Trade-offs:**

- ✅ Provides both semantics
- ✅ Users can choose based on their needs
- ⚠️ API complexity increases
- ⚠️ Multiple ways to do the same thing

**Recommendation:** Approach 1 is preferred as it follows standard FP conventions. The `Clone` requirement is acceptable given that lazy values typically need to be shareable.

---

### Issue 3: Awkward `Fn(()) -> A` Interface

**Location:** Throughout the file (thunk signatures)

**Current Implementation:**

```rust
// Every thunk creation requires |_| instead of ||
let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
```

**Problem:**
Thunks take `()` as an explicit parameter (`Fn(()) -> A`) rather than being zero-argument functions (`Fn() -> A`). This results in the awkward `|_|` pattern throughout all examples.

**Root Cause:**
The `CloneableFn` trait is designed for functions with parameters, and `Lazy` reuses it with `()` as a dummy parameter. This is a constraint from the broader library architecture.

#### Approach 1: Wrapper Constructor with Zero-Arg Closure

Add convenience constructors that accept `Fn() -> A`:

```rust
impl RcLazyConfig {
    /// Creates a thunk from a zero-argument closure.
    ///
    /// This is a convenience method that wraps a `Fn() -> A` into
    /// the required `Fn(()) -> A` signature.
    pub fn thunk<'a, A, F>(f: F) -> <Self as LazyConfig>::ThunkOf<'a, A>
    where
        A: 'a,
        F: Fn() -> A + Clone + 'a,
    {
        Self::new_thunk(move |_| f())
    }
}

impl ArcLazyConfig {
    /// Creates a thread-safe thunk from a zero-argument closure.
    pub fn thunk<'a, A, F>(f: F) -> <Self as LazyConfig>::ThunkOf<'a, A>
    where
        A: 'a,
        F: Fn() -> A + Send + Sync + Clone + 'a,
    {
        Self::new_thunk(move |_| f())
    }
}

// Usage becomes cleaner:
let lazy = RcLazy::new(RcLazyConfig::thunk(|| 42));
```

**Trade-offs:**

- ✅ Cleaner user-facing API
- ✅ No changes to underlying architecture
- ✅ Backward compatible (old API still works)
- ⚠️ Slight runtime overhead (extra closure wrapper)
- ⚠️ Two ways to create thunks

#### Approach 2: Add `delay` Constructor to Lazy

````rust
impl<'a, A> RcLazy<'a, A> {
    /// Creates a lazy value from a zero-argument closure.
    ///
    /// ### Examples
    ///
    /// ```
    /// use fp_library::types::lazy::*;
    ///
    /// let lazy = RcLazy::delay(|| 42);
    /// assert_eq!(Lazy::force_or_panic(&lazy), 42);
    /// ```
    pub fn delay<F>(f: F) -> Self
    where
        F: Fn() -> A + Clone + 'a,
    {
        Self::new(RcLazyConfig::new_thunk(move |_| f()))
    }
}

impl<'a, A> ArcLazy<'a, A> {
    /// Creates a thread-safe lazy value from a zero-argument closure.
    pub fn delay<F>(f: F) -> Self
    where
        F: Fn() -> A + Send + Sync + Clone + 'a,
    {
        Self::new(ArcLazyConfig::new_thunk(move |_| f()))
    }
}

// Clean usage:
let lazy = RcLazy::delay(|| expensive_computation());
````

**Trade-offs:**

- ✅ Significantly cleaner API
- ✅ Matches conventions from other languages (Scala's `lazy`, etc.)
- ✅ Single entry point for common use case
- ✅ Backward compatible
- ⚠️ Slight runtime overhead
- ⚠️ Doesn't address underlying architectural issue

#### Approach 3: Refactor CloneableFn to Support Zero-Arg Functions

This would require changes to `CloneableFn`:

```rust
// In classes/cloneable_fn.rs
pub trait CloneableFn: ... {
    // Existing
    type Of<'a, A, B>: Clone + Deref<Target = dyn 'a + Fn(A) -> B>;

    // New: zero-argument functions
    type OfUnit<'a, B>: Clone + Deref<Target = dyn 'a + Fn() -> B>;

    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B + Clone) -> Self::Of<'a, A, B>;
    fn new_unit<'a, B>(f: impl 'a + Fn() -> B + Clone) -> Self::OfUnit<'a, B>;
}
```

**Trade-offs:**

- ✅ Architecturally clean
- ✅ No runtime overhead
- ⚠️ Major breaking change to `CloneableFn`
- ⚠️ Requires updating all implementors
- ⚠️ Increases trait complexity

**Recommendation:** Approach 2 (`delay` constructor) is recommended as it provides the best ergonomics with minimal disruption.

---

### Issue 4: Documentation Example Mismatches

**Location:** Lines 239-246, 457-461, 500-504

**Problem:**
Documentation examples use the wrong types:

- `RcLazyConfig::append` docs show `ArcLazy` examples
- `ArcLazyConfig::append` docs show `RcLazy` examples
- `ArcLazyConfig::empty` docs show `RcLazy` example

**Current (Incorrect):**

```rust
/// // In RcLazyConfig::append documentation:
/// let x = ArcLazy::new(ArcLazyConfig::new_thunk(|_| "Hello, ".to_string()));
/// let y = ArcLazy::new(ArcLazyConfig::new_thunk(|_| "World!".to_string()));
/// let z = ArcLazyConfig::append(x, y);  // Wrong! Should be RcLazyConfig
```

#### Approach 1: Fix the Examples (Recommended)

Simply correct the documentation to use the matching types:

**For `RcLazyConfig::append` (line 239):**

````rust
/// ### Examples
///
/// ```
/// use fp_library::types::lazy::*;
///
/// let x = RcLazy::new(RcLazyConfig::new_thunk(|_| "Hello, ".to_string()));
/// let y = RcLazy::new(RcLazyConfig::new_thunk(|_| "World!".to_string()));
/// let z = RcLazyConfig::append(x, y);
/// assert_eq!(Lazy::force_or_panic(&z), "Hello, World!".to_string());
/// ```
````

**For `ArcLazyConfig::append` (line 454):**

````rust
/// ### Examples
///
/// ```
/// use fp_library::types::lazy::*;
///
/// let x = ArcLazy::new(ArcLazyConfig::new_thunk(|_| "Hello, ".to_string()));
/// let y = ArcLazy::new(ArcLazyConfig::new_thunk(|_| "World!".to_string()));
/// let z = ArcLazyConfig::append(x, y);
/// assert_eq!(Lazy::force_or_panic(&z), "Hello, World!".to_string());
/// ```
````

**For `ArcLazyConfig::empty` (line 499):**

````rust
/// ### Examples
///
/// ```
/// use fp_library::types::lazy::*;
///
/// let x: ArcLazy<String> = ArcLazyConfig::empty();
/// assert_eq!(Lazy::force_or_panic(&x), "".to_string());
/// ```
````

**Trade-offs:**

- ✅ Simple fix
- ✅ Makes doctest failures more obvious
- ✅ Improves documentation accuracy

---

## Moderate Issues

### Issue 5: Error Information Loss During Re-panic

**Location:** Lines 257-261, 339-341, 473-476, etc.

**Current Implementation:**

```rust
Err(e) => std::panic::resume_unwind(Box::new(e.to_string())),
```

**Problem:**
When propagating errors through composed lazy operations, the code converts `LazyError` to a `String` before re-panicking. This loses:

- The original panic payload type
- Structured error information
- The ability to programmatically inspect error causes

#### Approach 1: Propagate LazyError Directly

```rust
Err(e) => std::panic::resume_unwind(Box::new(e)),
```

**Trade-offs:**

- ✅ Preserves error structure
- ✅ Simple change
- ⚠️ May change panic behavior if handlers expect String

#### Approach 2: Return Result Instead of Panicking

Refactor composed operations to propagate `Result`:

```rust
impl<A> LazySemigroup<A> for RcLazyConfig {
    fn append<'a>(
        x: Lazy<'a, Self, A>,
        y: Lazy<'a, Self, A>,
    ) -> Lazy<'a, Self, A>
    where
        A: Semigroup + Clone + 'a,
    {
        let thunk = Self::new_thunk(move |_| {
            // Store references to both lazy values
            // The actual combination will be computed when forced
            // Errors are stored in the result's OnceCell
            let x_result = Lazy::force(&x);
            let y_result = Lazy::force(&y);

            match (x_result, y_result) {
                (Ok(x_val), Ok(y_val)) => Semigroup::append(x_val.clone(), y_val.clone()),
                (Err(e), _) | (_, Err(e)) => {
                    // Re-panic with the original error structure
                    std::panic::resume_unwind(Box::new(e))
                }
            }
        });
        Lazy::new(thunk)
    }
}
```

**Trade-offs:**

- ✅ Preserves error information
- ⚠️ Still uses panic for propagation within thunks
- ⚠️ Limited improvement

#### Approach 3: Error-Aware Lazy Composition (Comprehensive)

Create new traits for fallible lazy operations:

```rust
/// Error-aware lazy semigroup
pub trait TryLazySemigroup<A>: LazyConfig {
    type Error;

    fn try_append<'a>(
        x: Lazy<'a, Self, A>,
        y: Lazy<'a, Self, A>,
    ) -> Lazy<'a, Self, Result<A, Self::Error>>
    where
        A: Semigroup + Clone + 'a;
}

impl<A> TryLazySemigroup<A> for RcLazyConfig {
    type Error = LazyError;

    fn try_append<'a>(
        x: Lazy<'a, Self, A>,
        y: Lazy<'a, Self, A>,
    ) -> Lazy<'a, Self, Result<A, Self::Error>>
    where
        A: Semigroup + Clone + 'a,
    {
        let thunk = Self::new_thunk(move |_| {
            let x_result = Lazy::force(&x).map(Clone::clone);
            let y_result = Lazy::force(&y).map(Clone::clone);

            match (x_result, y_result) {
                (Ok(x_val), Ok(y_val)) => Ok(Semigroup::append(x_val, y_val)),
                (Err(e), _) => Err(e),
                (_, Err(e)) => Err(e),
            }
        });
        Lazy::new(thunk)
    }
}
```

**Trade-offs:**

- ✅ Full error information preserved
- ✅ Errors can be inspected and handled
- ✅ Composable error handling
- ⚠️ More complex API
- ⚠️ Different return type (`Lazy<Result<A, E>>` vs `Lazy<A>`)
- ⚠️ May require separate traits for fallible operations

**Recommendation:** Approach 1 is the minimal fix; Approach 3 is preferred for comprehensive error handling.

---

### Issue 6: No LazyDefer for ArcLazyConfig

**Location:** Line 514-516

**Current State:**

```rust
// Note: LazyDefer is NOT implemented for ArcLazyConfig because the Defer trait
// allows any FnBrand, but ArcLazy requires Send + Sync closures.
```

**Problem:**
`ArcLazy` cannot use the `Defer` trait, creating API asymmetry between `RcLazy` and `ArcLazy`.

#### Approach 1: Document and Use SendDefer (Current)

The current solution uses `SendDefer` for thread-safe deferred evaluation:

```rust
// For RcLazy:
let lazy = defer::<RcLazy<i32>, RcFnBrand>(...);

// For ArcLazy:
let lazy = send_defer::<LazyBrand<ArcLazyConfig>, _, _>(|| ...);
```

**Trade-offs:**

- ✅ Type-safe (compiler enforces correct usage)
- ✅ Clear semantic distinction
- ⚠️ Different APIs for Rc vs Arc variants
- ⚠️ User must remember which to use

#### Approach 2: Unified Defer API with Conditional Bounds

Create a wrapper type that dispatches to the correct implementation:

```rust
/// Unified defer function that works for both Rc and Arc variants.
///
/// Automatically selects the appropriate implementation based on the
/// config type and closure bounds.
pub fn lazy_defer<'a, Config, A, F>(f: F) -> Lazy<'a, Config, A>
where
    Config: LazyConfig,
    A: Clone + 'a,
    F: LazyThunk<'a, Config, A>,
{
    F::create_lazy(f)
}

/// Trait for thunks that can create lazy values.
pub trait LazyThunk<'a, Config: LazyConfig, A: 'a>: Sized {
    fn create_lazy(self) -> Lazy<'a, Config, A>;
}

// Implementation for RcLazy
impl<'a, A, F> LazyThunk<'a, RcLazyConfig, A> for F
where
    A: Clone + 'a,
    F: Fn() -> RcLazy<'a, A> + Clone + 'a,
{
    fn create_lazy(self) -> RcLazy<'a, A> {
        // ... implementation
    }
}

// Implementation for ArcLazy
impl<'a, A, F> LazyThunk<'a, ArcLazyConfig, A> for F
where
    A: Clone + Send + Sync + 'a,
    F: Fn() -> ArcLazy<'a, A> + Send + Sync + Clone + 'a,
{
    fn create_lazy(self) -> ArcLazy<'a, A> {
        // ... implementation
    }
}
```

**Trade-offs:**

- ✅ Unified API surface
- ✅ Automatic dispatch based on types
- ⚠️ More complex implementation
- ⚠️ May have confusing error messages when bounds aren't met

**Recommendation:** Approach 1 is acceptable given the clear semantic distinction between thread-safe and non-thread-safe operations.

---

### Issue 7: Verbose Construction Pattern

**Current:**

```rust
let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| expensive_computation()));
```

**Desired:**

```rust
let lazy = RcLazy::delay(|| expensive_computation());
```

See [Issue 3](#issue-3-awkward-fn---a-interface) Approach 2 for the recommended solution.

---

## Minor Issues

### Issue 8: AssertUnwindSafe Risk

**Location:** Line 700

**Current Implementation:**

```rust
std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| thunk(())))
```

**Problem:**
`AssertUnwindSafe` asserts that the closure is unwind-safe without proof. If the thunk modifies shared state and panics partway through, that state could be left inconsistent.

#### Approach 1: Document the Risk (Minimal)

Add clear documentation about the requirements for thunks:

```rust
/// Forces the evaluation of the thunk and returns the value.
///
/// # Panic Safety
///
/// The thunk is executed within `catch_unwind`. If your thunk modifies
/// shared mutable state, ensure it maintains invariants even if a panic
/// occurs partway through. Consider using:
///
/// - Immutable data structures
/// - Atomic operations with rollback
/// - RAII guards that restore state on drop
///
/// # Examples
/// ...
```

**Trade-offs:**

- ✅ Informs users of the risk
- ✅ No code changes
- ⚠️ Relies on users reading documentation

#### Approach 2: Require UnwindSafe Bound

```rust
pub fn force(this: &Self) -> Result<&A, LazyError>
where
    // Add UnwindSafe requirement to the thunk's captured state
    Config::ThunkOf<'a, A>: std::panic::UnwindSafe,
{
    // ... implementation
}
```

**Trade-offs:**

- ✅ Compiler-enforced safety
- ⚠️ Very restrictive - most closures won't satisfy this
- ⚠️ Would break existing code
- ⚠️ May require `AssertUnwindSafe` wrapper at call sites anyway

**Recommendation:** Approach 1 with comprehensive documentation is pragmatic.

---

### Issue 9: Sequential Error Evaluation in Semigroup

**Location:** Lines 254-266

**Current:**

```rust
fn append<'a>(x: Lazy<'a, Self, A>, y: Lazy<'a, Self, A>) -> Lazy<'a, Self, A> {
    let thunk = Self::new_thunk(move |_| {
        let x_val = match Lazy::force(&x) {
            Ok(v) => v.clone(),
            Err(e) => std::panic::resume_unwind(...),  // Stops here if x fails
        };
        let y_val = match Lazy::force(&y) {
            Ok(v) => v.clone(),
            Err(e) => std::panic::resume_unwind(...),
        };
        Semigroup::append(x_val, y_val)
    });
    Lazy::new(thunk)
}
```

**Problem:**
If `x` fails, `y` is never evaluated. The error doesn't indicate which operand failed.

#### Approach 1: Add Context to Errors

```rust
fn append<'a>(x: Lazy<'a, Self, A>, y: Lazy<'a, Self, A>) -> Lazy<'a, Self, A> {
    let thunk = Self::new_thunk(move |_| {
        let x_val = match Lazy::force(&x) {
            Ok(v) => v.clone(),
            Err(e) => {
                let msg = format!("left operand of append failed: {}", e);
                std::panic::resume_unwind(Box::new(msg))
            }
        };
        let y_val = match Lazy::force(&y) {
            Ok(v) => v.clone(),
            Err(e) => {
                let msg = format!("right operand of append failed: {}", e);
                std::panic::resume_unwind(Box::new(msg))
            }
        };
        Semigroup::append(x_val, y_val)
    });
    Lazy::new(thunk)
}
```

**Trade-offs:**

- ✅ Better error messages
- ✅ Helps debugging
- ⚠️ Still sequential evaluation
- ⚠️ String allocation for errors

#### Approach 2: Parallel Evaluation with Combined Errors

For `ArcLazy`, both operands could be evaluated in parallel:

```rust
impl<A: Send + Sync> LazySemigroup<A> for ArcLazyConfig {
    fn append<'a>(
        x: Lazy<'a, Self, A>,
        y: Lazy<'a, Self, A>,
    ) -> Lazy<'a, Self, A>
    where
        A: Semigroup + Clone + 'a,
    {
        let thunk = Self::new_thunk(move |_| {
            use std::thread;

            let x_handle = {
                let x = x.clone();
                thread::spawn(move || Lazy::force(&x).map(Clone::clone))
            };

            let y_result = Lazy::force(&y).map(Clone::clone);
            let x_result = x_handle.join().expect("thread panicked");

            match (x_result, y_result) {
                (Ok(x_val), Ok(y_val)) => Semigroup::append(x_val, y_val),
                (Err(e), Ok(_)) => panic!("left operand failed: {}", e),
                (Ok(_), Err(e)) => panic!("right operand failed: {}", e),
                (Err(e1), Err(e2)) => panic!("both operands failed: left={}, right={}", e1, e2),
            }
        });
        Lazy::new(thunk)
    }
}
```

**Trade-offs:**

- ✅ Reports both errors if both fail
- ✅ Potential performance improvement from parallelism
- ⚠️ Thread spawning overhead
- ⚠️ Only applicable to `ArcLazy`
- ⚠️ More complex implementation

**Recommendation:** Approach 1 for better error messages is a reasonable improvement.

---

## What's Well-Designed

| Aspect                       | Assessment                                                               |
| ---------------------------- | ------------------------------------------------------------------------ |
| **Configuration separation** | Excellent - Clean separation between `RcLazyConfig` and `ArcLazyConfig`  |
| **Memoization correctness**  | Sound - Uses `OnceCell`/`OnceLock` properly with correct synchronization |
| **Shared semantics**         | Correct - Clones share memoization state (Haskell-like behavior)         |
| **Panic handling**           | Good - Catches panics and caches errors (poisoned state)                 |
| **Thread safety**            | Correct - Proper `Send`/`Sync` bounds and thread-safe primitives         |
| **Poisoning detection**      | Good - `is_poisoned()` and `get_error()` for introspection               |
| **Test coverage**            | Comprehensive - Good coverage of core functionality                      |
| **Brand pattern**            | Consistent - Follows the library's HKT emulation pattern                 |

---

## Summary of Recommendations

### Priority 1 (High Impact, Low Effort)

1. **Remove unnecessary `Clone` bound** on `Lazy::Clone` impl (Issue 1, Approach 1)
2. **Fix documentation examples** (Issue 4, Approach 1)
3. **Add `delay` constructor** for ergonomic creation (Issue 3, Approach 2)

### Priority 2 (High Impact, Medium Effort)

4. **Implement `Functor`** with lazy map (Issue 2, Approach 1)
5. **Preserve error information** in re-panics (Issue 5, Approach 1)
6. **Add error context** to semigroup operations (Issue 9, Approach 1)

### Priority 3 (Consider for Future)

7. **Implement `Applicative` and `Monad`** for full FP utility
8. **Add `thunk` convenience method** to config types (Issue 3, Approach 1)
9. **Document `AssertUnwindSafe` risks** (Issue 8, Approach 1)

### Not Recommended

- Changing `CloneableFn` to support zero-arg functions (too disruptive)
- Requiring `UnwindSafe` bounds (too restrictive)
- Removing `Fn(()) -> A` pattern (architectural constraint)
