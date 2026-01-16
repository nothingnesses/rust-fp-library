# SendClonableFn Extension Trait Implementation Plan

## 1. Executive Summary

This document outlines a comprehensive plan to implement **Solution 1: Pure Extension Trait** from the [Thread Safety and Parallelism](../limitations.md#thread-safety-and-parallelism) section of the limitations document. This solution introduces thread-safe function capabilities to `fp-library` without breaking changes to existing traits.

The implementation adds:

1. A new `SendClonableFn` extension trait for brands that support `Send + Sync` function wrappers
2. Implementation of `SendClonableFn` for `ArcFnBrand`
3. A new `ParFoldable` trait for parallel folding operations
4. Optional Rayon integration for parallel implementations
5. **Enhancement to `Apply!` macro** with optional `output` parameter for accessing `SendOf`

---

## 2. Current State Analysis

### 2.1 The Problem

The current `Foldable` trait and its default implementations (`fold_right`, `fold_left`) are **not thread-safe** in terms of sending computations across threads, even when using `ArcFnBrand`. This is documented in [limitations.md](../limitations.md#the-issue).

**Concrete limitations:**

- Cannot spawn a thread and pass a `fold_right` operation using `ArcFnBrand` into it
- Cannot implement a parallel `fold_map` using libraries like Rayon
- `ArcFnBrand` produces `!Send` function wrappers despite using `Arc`

### 2.2 Root Causes

From [`fp-library/src/classes/clonable_fn.rs`](../../fp-library/src/classes/clonable_fn.rs):

```rust
pub trait ClonableFn: Function {
    type Of<'a, A, B>: Clone + Deref<Target = dyn 'a + Fn(A) -> B>;

    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B>;
}
```

**Issue 1:** The `new` method accepts `impl Fn(A) -> B` without `Send` bounds. This is intentional to support `RcFnBrand` wrapping closures that capture `Rc` pointers.

**Issue 2:** The associated type `Of` has `Deref<Target = dyn 'a + Fn(A) -> B>`. The target type `dyn Fn` is different from `dyn Fn + Send + Sync`, so `ArcFnBrand` cannot provide a `Send` wrapper through this trait.

From [`fp-library/src/types/arc_fn.rs`](../../fp-library/src/types/arc_fn.rs):

```rust
impl ClonableFn for ArcFnBrand {
    type Of<'a, A, B> = Arc<dyn 'a + Fn(A) -> B>;  // Note: !Send

    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> ... {
        Arc::new(f)  // Cannot guarantee f is Send
    }
}
```

### 2.3 Why Solution 1 is Preferred

Solution 1 (Pure Extension Trait) is chosen over alternatives because:

| Criterion                 | Solution 1 | Solution 2 (Raw Closures) | Solution 3 (Parallel Hierarchy) |
| ------------------------- | ---------- | ------------------------- | ------------------------------- |
| Breaking changes          | None       | None                      | None                            |
| Code duplication          | Minimal    | Moderate                  | Significant                     |
| HKT abstraction preserved | Yes        | No                        | Partial                         |
| Maintenance burden        | Low        | Low                       | High                            |
| Compile-time safety       | Full       | Full                      | Full                            |

---

## 3. Proposed Solution

### 3.1 Solution Overview

Add a **pure extension trait** `SendClonableFn` that provides thread-safe capabilities without modifying the existing `Function` or `ClonableFn` traits.

**Key insight:** The base `Function::Of` type _must_ remain compatible with non-`Send` closures because `Function::new` accepts `impl Fn`. The `SendClonableFn` extension trait introduces a completely **separate** associated type (`SendOf`), making changes to the base `Function` trait unnecessary.

### 3.2 The SendClonableFn Trait

```rust
/// Extension trait for brands that support thread-safe function wrappers.
/// Only implemented by brands that can provide `Send + Sync` guarantees.
pub trait SendClonableFn: ClonableFn {
    /// The Send-capable wrapped function type.
    /// This is distinct from Function::Of and explicitly requires
    /// the deref target to be `Send + Sync`.
    type SendOf<'a, A, B>: Clone
        + Send
        + Sync
        + Deref<Target = dyn 'a + Fn(A) -> B + Send + Sync>;

    /// Creates a new Send-capable clonable function wrapper.
    fn new_send<'a, A, B>(
        f: impl 'a + Fn(A) -> B + Send + Sync
    ) -> Self::SendOf<'a, A, B>;
}
```

**Design decisions:**

- `SendClonableFn: ClonableFn` — extends existing trait, not replaces
- `SendOf` is separate from `Of` — allows different types
- `new_send` requires `Send + Sync` on input — enforces thread safety at construction
- `RcFnBrand` does **not** implement `SendClonableFn` — correct semantics

### 3.3 ArcFnBrand Implementation

```rust
impl SendClonableFn for ArcFnBrand {
    type SendOf<'a, A, B> = Arc<dyn 'a + Fn(A) -> B + Send + Sync>;

    fn new_send<'a, A, B>(
        f: impl 'a + Fn(A) -> B + Send + Sync
    ) -> Self::SendOf<'a, A, B> {
        Arc::new(f)
    }
}
```

### 3.4 The ParFoldable Trait

```rust
/// A type class for structures that can be folded in parallel.
///
/// This trait provides parallel versions of `Foldable` operations
/// that require `Send + Sync` bounds on elements and functions.
pub trait ParFoldable<FnBrand: SendClonableFn>: Foldable {
    /// Parallel version of fold_map using the branded SendOf function type.
    fn par_fold_map<'a, A, M>(
        fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
        f: FnBrand::SendOf<'a, A, M>,
    ) -> M
    where
        A: 'a + Clone + Send + Sync,
        M: Monoid + Send + Sync + 'a;

    /// Parallel version of fold_right.
    fn par_fold_right<'a, A, B>(
        f: FnBrand::SendOf<'a, (A, B), B>,
        init: B,
        fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
    ) -> B
    where
        A: 'a + Clone + Send + Sync,
        B: Send + Sync + 'a;
}
```

**Note:** The `ParFoldable` trait uses `FnBrand::SendOf` for the function parameter, maintaining the library's branded function abstraction rather than raw closures.

---

## 4. Implementation Details

### 4.1 Apply! Macro Enhancement (Prerequisite)

The `SendClonableFn` trait introduces a new associated type `SendOf` that differs from the standard `Of` used by other Kind traits. To maintain ergonomic consistency with existing `Apply!` usage, the macro must be enhanced **first** to support an optional `output` parameter.

#### 4.1.1 Current Apply! Limitation

The [`Apply!`](../../fp-macros/src/apply.rs) macro currently always projects to `::Of<...>`:

```rust
// Current expansion:
Apply!(brand: ArcFnBrand, kind: SendClonableFn, lifetimes: ('a), types: (A, B))
// Expands to: <ArcFnBrand as SendClonableFn>::Of<'a, A, B>  // WRONG!
// We need: <ArcFnBrand as SendClonableFn>::SendOf<'a, A, B>
```

Without enhancement, users must use verbose direct syntax:

```rust
<ArcFnBrand as SendClonableFn>::SendOf<'a, A, M>
```

#### 4.1.2 Proposed Enhancement

Add an optional `output` parameter that defaults to `Of`:

```rust
// Default behavior (backward compatible):
Apply!(brand: B, kind: SomeKind, lifetimes: ('a), types: (T))
// Expands to: <B as SomeKind>::Of<'a, T>

// With explicit associated type:
Apply!(brand: ArcFnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, B))
// Expands to: <ArcFnBrand as SendClonableFn>::SendOf<'a, A, B>

// Using signature syntax:
Apply!(brand: ArcFnBrand, signature: ('a, A, B), output: SendOf)
// Expands to: <ArcFnBrand as Kind_...>::SendOf<'a, A, B>
```

#### 4.1.3 Implementation Details

**File:** [`fp-macros/src/apply.rs`](../../fp-macros/src/apply.rs) (modify)

The current implementation uses label-based parsing where each parameter is parsed as an identifier followed by `:` and a value. The `output` parameter will be added as an additional recognized label.

**Modified `ApplyInput` struct:**

```rust
pub struct ApplyInput {
    pub brand: Type,
    pub kind_source: KindSource,
    /// Optional associated type name, defaults to "Of"
    pub output: Option<Ident>,
}
```

**Parsing updates:**

The current parser iterates through `label: value` pairs. Add handling for `output`:

```rust
impl Parse for ApplyInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut brand = None;
        let mut kind_source_type = None;
        let mut signature = None;
        let mut kind = None;
        let mut lifetimes = None;
        let mut types = None;
        let mut output = None;  // NEW

        while !input.is_empty() {
            let label: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            if label == "brand" {
                brand = Some(input.parse()?);
            } else if label == "signature" {
                // ... existing signature handling ...
            } else if label == "kind" {
                // ... existing kind handling ...
            } else if label == "lifetimes" {
                // ... existing lifetimes handling ...
            } else if label == "types" {
                // ... existing types handling ...
            } else if label == "output" {  // NEW
                output = Some(input.parse()?);
            } else {
                return Err(syn::Error::new(label.span(), "Unknown parameter"));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        // ... existing validation ...

        Ok(ApplyInput {
            brand,
            kind_source,
            output,  // NEW
        })
    }
}
```

**Code generation updates:**

Update `apply_impl` to use the provided associated type name or default to `Of`:

```rust
pub fn apply_impl(input: ApplyInput) -> TokenStream {
    let brand = &input.brand;
    let assoc_type = input.output
        .unwrap_or_else(|| Ident::new("Of", Span::call_site()));

    let (kind_name, params) = match &input.kind_source {
        KindSource::Generated(sig) => {
            // ... existing logic to build kind_name and params ...
        }
        KindSource::Explicit { kind, lifetimes, types } => {
            // ... existing logic to build kind_name and params ...
        }
    };

    quote! {
        <#brand as #kind_name>::#assoc_type<#params>
    }
}
```

**Note:** The explicit `kind` mode requires both `lifetimes` and `types` parameters (even if empty as `()`) - this is enforced by the existing parser.

#### 4.1.4 Testing the Enhancement

**Unit tests:**

```rust
#[test]
fn test_apply_with_output() {
    // Default behavior
    let input: ApplyInput = syn::parse_quote!(brand: B, kind: K, lifetimes: ('a), types: (T));
    assert!(input.output.is_none());

    // With explicit output
    let input: ApplyInput = syn::parse_quote!(
        brand: B, kind: K, output: SendOf, lifetimes: ('a), types: (T)
    );
    assert_eq!(input.output.unwrap().to_string(), "SendOf");
}
```

**Compile-fail tests:**

Add test in [`fp-macros/tests/ui/`](../../fp-macros/tests/ui/):

```rust
// apply_invalid_output.rs
use fp_macros::Apply;

struct TestBrand;

// Should fail: output must be an identifier
type Bad = Apply!(brand: TestBrand, kind: SomeKind, output: "Of", lifetimes: ('a), types: (i32));
```

#### 4.1.5 Backward Compatibility

This enhancement is **fully backward compatible**:

1. Existing code without `output` continues to work unchanged
2. The parameter is optional with a sensible default (`Of`)
3. No changes to existing trait definitions or implementations
4. Parameter can appear in any position after `brand`

#### 4.1.6 Future Benefits

This enhancement provides extensibility for future traits that may use non-`Of` associated types:

- `SendOf` for `SendClonableFn`
- Potential `MutOf` for mutable references
- Potential `RefOf` for borrowed references
- Any future trait with specialized associated types

### 4.2 File Structure

```
fp-library/src/
├── classes/
│   ├── clonable_fn.rs        # Existing (unchanged)
│   ├── send_clonable_fn.rs   # NEW: SendClonableFn trait
│   ├── foldable.rs           # Existing (unchanged)
│   ├── par_foldable.rs       # NEW: ParFoldable trait
│   └── mod.rs                # Update to export new modules
├── types/
│   ├── arc_fn.rs             # Update: add SendClonableFn impl
│   └── ...
└── lib.rs                    # Update: re-export new traits

fp-macros/src/
├── apply.rs                  # MODIFY: add output parameter
└── ...
```

### 4.3 SendClonableFn Module

**File:** [`fp-library/src/classes/send_clonable_fn.rs`](../../fp-library/src/classes/send_clonable_fn.rs) (new)

````rust
//! Thread-safe extension to ClonableFn.

use super::clonable_fn::ClonableFn;
use crate::Apply;
use std::ops::Deref;

/// Extension trait for brands that support thread-safe function wrappers.
///
/// This trait is implemented by "Brand" types (like [`ArcFnBrand`][crate::brands::ArcFnBrand])
/// that can provide `Send + Sync` guarantees on their wrapped functions. Brands like
/// [`RcFnBrand`][crate::brands::RcFnBrand] that cannot support thread safety do NOT
/// implement this trait.
///
/// # Thread Safety
///
/// The `SendOf` type is guaranteed to be `Send + Sync`, and the `new_send` method
/// requires the input function to also be `Send + Sync`. This ensures that all
/// function wrappers created through this trait are safe to send across threads.
///
/// # Usage
///
/// This trait is primarily used by `ParFoldable` and other parallel operations
/// that need to distribute work across threads.
///
/// # Examples
///
/// ```
/// use fp_library::classes::send_clonable_fn::SendClonableFn;
/// use fp_library::brands::ArcFnBrand;
/// use std::thread;
///
/// let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x * 2);
///
/// // f can be sent to another thread
/// let handle = thread::spawn(move || f(21));
/// assert_eq!(handle.join().unwrap(), 42);
/// ```
pub trait SendClonableFn: ClonableFn {
    /// The Send-capable wrapped function type.
    ///
    /// Unlike [`ClonableFn::Of`], this type is guaranteed to be `Send + Sync`.
    /// The `Deref` target includes `Send + Sync` bounds on the function.
    type SendOf<'a, A, B>: Clone
        + Send
        + Sync
        + Deref<Target = dyn 'a + Fn(A) -> B + Send + Sync>;

    /// Creates a new Send-capable clonable function wrapper.
    ///
    /// # Type Signature
    ///
    /// `forall a b. SendClonableFn f => (a -> b where Send + Sync) -> f a b`
    ///
    /// # Parameters
    ///
    /// * `f`: The closure to wrap. Must be `Send + Sync`.
    ///
    /// # Returns
    ///
    /// A thread-safe wrapped function.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::classes::send_clonable_fn::SendClonableFn;
    /// use fp_library::brands::ArcFnBrand;
    ///
    /// let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x * 2);
    /// assert_eq!(f(5), 10);
    /// ```
    fn new_send<'a, A, B>(
        f: impl 'a + Fn(A) -> B + Send + Sync
    ) -> Self::SendOf<'a, A, B>;
}

/// Creates a new thread-safe clonable function wrapper.
///
/// Free function version that dispatches to [the type class' associated function][`SendClonableFn::new_send`].
///
/// # Type Signature
///
/// `forall a b. SendClonableFn f => (a -> b where Send + Sync) -> f a b`
///
/// # Parameters
///
/// * `f`: The closure to wrap. Must be `Send + Sync`.
///
/// # Returns
///
/// A thread-safe wrapped function.
///
/// # Examples
///
/// ```
/// use fp_library::classes::send_clonable_fn::new_send;
/// use fp_library::brands::ArcFnBrand;
///
/// let f = new_send::<ArcFnBrand, _, _>(|x: i32| x * 2);
/// assert_eq!(f(5), 10);
/// ```
pub fn new_send<'a, F, A, B>(
    f: impl 'a + Fn(A) -> B + Send + Sync
) -> F::SendOf<'a, A, B>
where
    F: SendClonableFn,
{
    F::new_send(f)
}
````

### 4.4 ParFoldable Module

**File:** [`fp-library/src/classes/par_foldable.rs`](../../fp-library/src/classes/par_foldable.rs) (new)

````rust
//! Parallel folding operations.

use super::{foldable::Foldable, monoid::Monoid, send_clonable_fn::SendClonableFn};
use crate::{Apply, kinds::*};

/// A type class for structures that can be folded in parallel.
///
/// This trait provides parallel versions of `Foldable` operations that require
/// `Send + Sync` bounds on elements and functions. It uses the branded
/// `SendOf` function type to maintain the library's HKT abstraction.
///
/// # Minimal Implementation
///
/// A minimal implementation requires [`ParFoldable::par_fold_map`].
///
/// # Thread Safety
///
/// All operations in this trait are designed to be safe for parallel execution:
/// - Element type `A` must be `Send + Sync`
/// - Accumulator/result types must be `Send + Sync`
/// - Functions are wrapped in `FnBrand::SendOf` which guarantees `Send + Sync`
///
/// # Examples
///
/// ```ignore
/// use fp_library::classes::par_foldable::ParFoldable;
/// use fp_library::brands::{VecBrand, ArcFnBrand};
/// use fp_library::classes::send_clonable_fn::SendClonableFn;
///
/// let v = vec![1, 2, 3, 4, 5];
/// let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x as i64);
/// let sum: i64 = VecBrand::par_fold_map::<ArcFnBrand, _, _>(v, f);
/// ```
pub trait ParFoldable<FnBrand: SendClonableFn>: Foldable {
    /// Parallel version of fold_map.
    ///
    /// Maps each element to a monoid value using `f`, then combines all values
    /// using the monoid's `append` operation. The mapping operations may be
    /// executed in parallel.
    ///
    /// # Type Signature
    ///
    /// `forall a m. (ParFoldable t, Monoid m, Send m, Sync m) => (f a m, t a) -> m`
    ///
    /// # Type Parameters
    ///
    /// * `FnBrand`: The brand of thread-safe function to use (must implement `SendClonableFn`)
    /// * `A`: The element type (must be `Send + Sync`)
    /// * `M`: The monoid type (must be `Send + Sync`)
    ///
    /// # Parameters
    ///
    /// * `fa`: The foldable structure
    /// * `f`: The mapping function wrapped using `Apply!` with `output: SendOf`
    ///
    /// # Returns
    ///
    /// The combined monoid value
    fn par_fold_map<'a, A, M>(
        fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
        f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, M)),
    ) -> M
    where
        A: 'a + Clone + Send + Sync,
        M: Monoid + Send + Sync + 'a;

    /// Parallel version of fold_right.
    ///
    /// Folds the structure by applying a function from right to left, potentially
    /// in parallel.
    ///
    /// # Type Signature
    ///
    /// `forall a b. ParFoldable t => (f (a, b) b, b, t a) -> b`
    ///
    /// # Type Parameters
    ///
    /// * `FnBrand`: The brand of thread-safe function to use
    /// * `A`: The element type (must be `Send + Sync`)
    /// * `B`: The accumulator type (must be `Send + Sync`)
    ///
    /// # Parameters
    ///
    /// * `f`: The folding function wrapped using `Apply!` with `output: SendOf`
    /// * `init`: The initial accumulator value
    /// * `fa`: The foldable structure
    ///
    /// # Returns
    ///
    /// The final accumulator value
    fn par_fold_right<'a, A, B>(
        f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: ((A, B), B)),
        init: B,
        fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
    ) -> B
    where
        A: 'a + Clone + Send + Sync,
        B: Send + Sync + 'a;
}

/// Parallel fold_map operation.
///
/// Free function version that dispatches to [the type class' associated function][`ParFoldable::par_fold_map`].
pub fn par_fold_map<'a, FnBrand, Brand, A, M>(
    fa: Apply!(brand: Brand, signature: ('a, A: 'a) -> 'a),
    f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, M)),
) -> M
where
    FnBrand: SendClonableFn,
    Brand: ParFoldable<FnBrand>,
    A: 'a + Clone + Send + Sync,
    M: Monoid + Send + Sync + 'a,
{
    Brand::par_fold_map(fa, f)
}

/// Parallel fold_right operation.
///
/// Free function version that dispatches to [the type class' associated function][`ParFoldable::par_fold_right`].
pub fn par_fold_right<'a, FnBrand, Brand, A, B>(
    f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: ((A, B), B)),
    init: B,
    fa: Apply!(brand: Brand, signature: ('a, A: 'a) -> 'a),
) -> B
where
    FnBrand: SendClonableFn,
    Brand: ParFoldable<FnBrand>,
    A: 'a + Clone + Send + Sync,
    B: Send + Sync + 'a,
{
    Brand::par_fold_right(f, init, fa)
}
````

### 4.5 ArcFnBrand Updates

**File:** [`fp-library/src/types/arc_fn.rs`](../../fp-library/src/types/arc_fn.rs) (modify)

Add the `SendClonableFn` implementation:

````rust
use crate::classes::send_clonable_fn::SendClonableFn;

impl SendClonableFn for ArcFnBrand {
    type SendOf<'a, A, B> = Arc<dyn 'a + Fn(A) -> B + Send + Sync>;

    /// Creates a new thread-safe `Arc`-wrapped function.
    ///
    /// # Type Signature
    ///
    /// `forall a b. SendClonableFn ArcFnBrand => (a -> b where Send + Sync) -> ArcFnBrand a b`
    ///
    /// # Parameters
    ///
    /// * `f`: The function to wrap. Must be `Send + Sync`.
    ///
    /// # Returns
    ///
    /// A thread-safe `Arc`-wrapped function.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::brands::ArcFnBrand;
    /// use fp_library::classes::send_clonable_fn::SendClonableFn;
    /// use std::thread;
    ///
    /// let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x * 2);
    /// let handle = thread::spawn(move || f(21));
    /// assert_eq!(handle.join().unwrap(), 42);
    /// ```
    fn new_send<'a, A, B>(
        f: impl 'a + Fn(A) -> B + Send + Sync
    ) -> Self::SendOf<'a, A, B> {
        Arc::new(f)
    }
    ```

    ### 4.6 Vec and Option ParFoldable Implementations

    Provide `ParFoldable` implementations for common types. Note: The function parameter `f` uses the `Apply!` macro with `output: SendOf` as shown in section 4.4.

    **For VecBrand:**

    ```rust
    impl<FnBrand: SendClonableFn> ParFoldable<FnBrand> for VecBrand {
        fn par_fold_map<'a, A, M>(
            fa: Vec<A>,
            f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, M)),
        ) -> M
        where
            A: 'a + Clone + Send + Sync,
            M: Monoid + Send + Sync + 'a,
        {
            // Sequential implementation - can be replaced with rayon
            fa.into_iter()
                .map(|a| f(a))
                .fold(M::empty(), |acc, m| M::append(acc, m))
        }

        fn par_fold_right<'a, A, B>(
            f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: ((A, B), B)),
            init: B,
            fa: Vec<A>,
        ) -> B
        where
            A: 'a + Clone + Send + Sync,
            B: Send + Sync + 'a,
        {
            fa.into_iter()
                .rev()
                .fold(init, |b, a| f((a, b)))
        }
    }
    ```

    **For OptionBrand:**

    ```rust
    impl<FnBrand: SendClonableFn> ParFoldable<FnBrand> for OptionBrand {
        fn par_fold_map<'a, A, M>(
            fa: Option<A>,
            f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, M)),
        ) -> M
        where
            A: 'a + Clone + Send + Sync,
            M: Monoid + Send + Sync + 'a,
        {
            match fa {
                Some(a) => f(a),
                None => M::empty(),
            }
        }

        fn par_fold_right<'a, A, B>(
            f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: ((A, B), B)),
            init: B,
            fa: Option<A>,
        ) -> B
        where
            A: 'a + Clone + Send + Sync,
            B: Send + Sync + 'a,
        {
            match fa {
                Some(a) => f((a, init)),
                None => init,
            }
        }
    }
    ```

    ### 4.7 Optional: Rayon Feature Flag

    Add an optional `rayon` feature for truly parallel implementations:

    **Cargo.toml:**

    ```toml
    [features]
    default = []
    rayon = ["dep:rayon"]

    [dependencies]
    rayon = { version = "1.11", optional = true }
    ```

    **Rayon-powered implementation (in `vec.rs` under feature flag):**

    ```rust
    #[cfg(feature = "rayon")]
    use rayon::prelude::*;

    #[cfg(feature = "rayon")]
    impl<FnBrand: SendClonableFn> ParFoldable<FnBrand> for VecBrand {
        fn par_fold_map<'a, A, M>(
            fa: Vec<A>,
            f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, M)),
        ) -> M
        where
            A: 'a + Clone + Send + Sync,
            M: Monoid + Send + Sync + 'a,
        {
            fa.into_par_iter()
                .map(|a| f(a))
                .reduce(M::empty, |a, b| M::append(a, b))
        }

        // ... par_fold_right implementation
    }
    ```
````

---

## 5. Testing Strategy

### 5.1 Unit Tests

**SendClonableFn tests:**

1. `new_send` creates callable function
2. `SendOf` type is actually `Send + Sync` (compile-time check)
3. Function can be cloned
4. Function can be sent to another thread

**ParFoldable tests:**

1. `par_fold_map` with identity monoid
2. `par_fold_map` with sum/product monoids
3. `par_fold_right` correctness
4. Empty collection handling
5. Single element handling

### 5.2 Integration Tests

1. Thread spawn with `SendClonableFn` wrapped function
2. `ParFoldable` operations complete correctly across threads
3. Multiple threads accessing shared `SendOf` function
4. Compatibility with existing `Foldable` implementations

### 5.3 Property-Based Tests

Using QuickCheck:

1. `par_fold_map` produces same result as sequential `fold_map` (with commutative monoids)
2. `par_fold_right` produces same result as sequential `fold_right`
3. Thread safety: concurrent access doesn't cause data races

### 5.4 Compile-Fail Tests

1. Cannot call `new_send` with non-`Send` closure
2. Cannot call `new_send` with non-`Sync` closure
3. `RcFnBrand` does not implement `SendClonableFn` (should fail type check)

---

## 6. Documentation Updates

### 6.1 Module Documentation

- Add comprehensive module-level docs to `send_clonable_fn.rs`
- Add comprehensive module-level docs to `par_foldable.rs`
- Update `classes/mod.rs` documentation to mention new traits

### 6.2 API Documentation

Each public item needs:

- Summary line
- Type signature in Haskell-like notation
- Type parameter descriptions
- Parameter descriptions
- Return value description
- Example with doc-test

### 6.3 Limitations.md Update

Update the Thread Safety section to reference the implementation:

- Mark Solution 1 as implemented
- Link to the new traits
- Provide usage examples

### 6.4 CHANGELOG.md

Document the new feature:

- New `SendClonableFn` trait
- New `ParFoldable` trait
- `ArcFnBrand` now implements `SendClonableFn`
- Optional rayon feature flag

---

## 7. Risk Assessment

| Risk                           | Likelihood | Impact | Mitigation                                              |
| ------------------------------ | ---------- | ------ | ------------------------------------------------------- |
| Type complexity                | Medium     | Low    | Comprehensive examples and tests                        |
| Performance overhead           | Low        | Low    | Benchmark against sequential; rayon handles parallelism |
| Breaking existing code         | Very Low   | High   | Pure extension trait pattern; no changes to existing    |
| Incorrect thread safety claims | Low        | High   | Compile-time enforcement via trait bounds               |
| API ergonomics issues          | Medium     | Medium | User testing; iterate on API design                     |

---

## 8. Success Criteria

1. **Compilation:** All new code compiles without errors or warnings
2. **Thread Safety:** `ArcFnBrand::new_send` produces `Send + Sync` wrappers (verified by compile-time tests)
3. **Non-Breaking:** Existing code using `ClonableFn` and `Foldable` continues to work unchanged
4. **Tests Pass:** All unit, integration, property-based, and compile-fail tests pass
5. **Documentation:** All public items have comprehensive documentation with examples
6. **Parallelism Works:** Can spawn thread and pass `SendOf` function to it
7. **Rayon Integration:** Optional rayon feature provides true parallelism for `ParFoldable`

---

## 9. Future Extensions

This implementation provides a foundation for further thread-safe type classes:

1. **`SendMonad`:** Thread-safe monad operations
2. **`ParTraversable`:** Parallel traversal
3. **`SendApplicative`:** Thread-safe applicative functor
4. **Additional `ParFoldable` implementations:** For `Result`, `BTreeMap`, etc.

---

## 10. References

- [Limitations - Thread Safety and Parallelism](../limitations.md#thread-safety-and-parallelism)
- [Current ClonableFn Implementation](../../fp-library/src/classes/clonable_fn.rs)
- [Current Foldable Implementation](../../fp-library/src/classes/foldable.rs)
- [ArcFnBrand Implementation](../../fp-library/src/types/arc_fn.rs)
- [Rayon Crate Documentation](https://docs.rs/rayon/latest/rayon/)
- [Rust Send and Sync Traits](https://doc.rust-lang.org/nomicon/send-and-sync.html)
