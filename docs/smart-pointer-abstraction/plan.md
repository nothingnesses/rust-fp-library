# Pointer Abstraction & Shared Lazy Evaluation

## Summary

This document outlines a plan to introduce a unified pointer type class hierarchy that allows library types to be parameterized over the choice of smart pointer. The design uses the "Additional Associated Type" pattern (following `SendClonableFn`'s approach) to build an extensible hierarchy:

```
Pointer                         (base: Deref + new)
└── RefCountedPointer           (adds: Clone via CloneableOf)
    └── SendRefCountedPointer   (adds: Send + Sync marker)
```

This abstraction enables:

1. **Unified Rc/Arc selection** across multiple library types via `RefCountedPointer`
2. **Shared memoization semantics** for `Lazy` (Haskell-like behavior)
3. **Reduced code duplication** by building multiple types on a single foundation
4. **Future extensibility** for Box, custom allocators, or alternative smart pointers via the `Pointer` base trait

**Note**: This is a breaking change. Backward compatibility is not a goal; the focus is on the best possible design.

### Design Rationale

The hierarchy uses **additional associated types** (like `SendClonableFn::SendOf`) rather than marker traits because:

1. **Consistency**: Follows the established `ClonableFn` → `SendClonableFn` pattern in this codebase
2. **Expressiveness**: Subtraits cannot strengthen bounds on inherited associated types in Rust
3. **Extensibility**: Each level can add capabilities without breaking existing code
4. **Self-documenting**: `RefCountedPointer::CloneableOf` clearly indicates Clone capability

***

## Background & Motivation

### Conversation Context

This plan originated from a code review of `fp-library/src/types/lazy.rs`, which revealed:

1. **The current `Lazy` implementation is correct** but uses value semantics:
   * Cloning a `Lazy` creates a deep copy of the `OnceCell`
   * Each clone maintains independent memoization state
   * Forcing one clone does not affect others

2. **This differs from Haskell's lazy evaluation**:
   * In Haskell, all references to a thunk share memoization
   * Once forced, all references see the cached result
   * This enables efficient graph-based computation sharing

3. **To achieve Haskell-like semantics**, the `OnceCell` must be wrapped in a shared smart pointer (`Rc` or `Arc`)

4. **The existing library already has similar patterns**:
   * `ClonableFn` abstracts over `RcFnBrand` vs `ArcFnBrand`
   * Users choose at call sites: `clonable_fn_new::<RcFnBrand, _, _>(...)`
   * This pattern can be generalized and unified

5. **`SendClonableFn` extends `ClonableFn`** with thread-safe semantics:
   * Uses a separate `SendOf` associated type
   * Only `ArcFnBrand` implements it (not `RcFnBrand`)
   * This pattern inspires the `Pointer` → `RefCountedPointer` → `SendRefCountedPointer` hierarchy

### Current Architecture Gap

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CURRENT: Ad-hoc Rc/Arc Abstraction                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ClonableFn ─────extends───▶ SendClonableFn                                │
│       │                            │                                        │
│   ┌───┴───┐                    ┌───┘                                        │
│   │       │                    │                                            │
│ RcFnBrand ArcFnBrand ◀─────────┘  (only Arc implements SendClonableFn)      │
│                                                                             │
│                                                                             │
│   Lazy (current)                                                            │
│       │                                                                     │
│   Uses OnceBrand (not shared across clones)                                 │
│                                                                             │
│   Problem: Rc/Arc choice is duplicated; no shared foundation                │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Proposed Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     PROPOSED: Unified Pointer Hierarchy                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Pointer                      (base: Of<T> + new)                           │
│     │                                                                       │
│     ├── BoxBrand              (future: unique ownership)                    │
│     │                                                                       │
│     └── RefCountedPointer     (adds: CloneableOf<T> + cloneable_new)        │
│            │                                                                │
│            ├── RcBrand                                                      │
│            │                                                                │
│            └── SendRefCountedPointer  (marker for Send + Sync)              │
│                   │                                                         │
│                   └── ArcBrand                                              │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ Library types use RefCountedPointer for shared semantics:             │  │
│  │                                                                       │  │
│  │  FnBrand<P: RefCountedPointer>                                        │  │
│  │    - Uses P::CloneableOf for clonable function wrappers               │  │
│  │    - Implements ClonableFn, SendClonableFn (when P: SendRefCounted)   │  │
│  │                                                                       │  │
│  │  Lazy<P: RefCountedPointer, OnceBrand, A>                             │  │
│  │    - Uses P::CloneableOf for shared memoization                       │  │
│  │    - All clones share the same OnceCell                               │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  Type Aliases (for convenience):                                            │
│    RcFnBrand  = FnBrand<RcBrand>                                            │
│    ArcFnBrand = FnBrand<ArcBrand>                                           │
│    RcLazy     = Lazy<RcBrand, OnceCellBrand, A>                             │
│    ArcLazy    = Lazy<ArcBrand, OnceLockBrand, A>                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

***

## Design Goals

### Primary Goals

1. **Introduce `Pointer` trait** as a minimal base abstraction for all heap-allocated pointers
2. **Introduce `RefCountedPointer` trait** extending `Pointer` with `CloneableOf` for shared ownership (Rc/Arc)
3. **Introduce `SendRefCountedPointer` marker** for thread-safe reference counting (Arc only)
4. **Refactor `ClonableFn` to use `RefCountedPointer`** via `FnBrand<PtrBrand>` pattern
5. **Create `Lazy` type** with Haskell-like shared memoization semantics using `RefCountedPointer`
6. **Enable future extensibility** - Box support via `Pointer` without `RefCountedPointer`

### Non-Goals

1. **Backward compatibility** - this is a breaking change; best design takes priority
2. **Migration path** - not needed since we're not maintaining backward compat
3. **Implementing Box/UniquePointer now** - the `Pointer` base is established but Box impl deferred
4. **Automatic selection** of Rc vs Arc based on context (user explicitly chooses)

***

## Technical Design

### Part 1: Pointer Type Class Hierarchy

The design uses a three-level trait hierarchy following the "Additional Associated Type" pattern established by `ClonableFn` → `SendClonableFn`:

```
Pointer                    (base: Of<T> with Deref)
    │
    └── RefCountedPointer  (adds: CloneableOf<T> with Clone + Deref)
            │
            └── SendRefCountedPointer  (marker: CloneableOf<T> is Send+Sync)
```

#### Why This Pattern?

**Problem**: Rust subtraits cannot strengthen bounds on inherited associated types.

```rust
// This DOES NOT WORK:
trait Pointer {
    type Of<T: ?Sized>: Deref<Target = T>;  // No Clone
}
trait RefCountedPointer: Pointer {}  // Cannot add Clone to Of<T>
```

**Solution**: Following `SendClonableFn`'s approach, each level adds a NEW associated type with stronger bounds:

```rust
trait Pointer {
    type Of<T: ?Sized>: Deref<Target = T>;                    // No Clone
}
trait RefCountedPointer: Pointer {
    type CloneableOf<T: ?Sized>: Clone + Deref<Target = T>;   // Has Clone
}
```

For Rc/Arc, `Of<T>` and `CloneableOf<T>` will be the same type (both Clone), but Box would only implement `Pointer` with `Of<T> = Box<T>` (not Clone unless T: Clone).

#### Trait Definitions

````rust
// fp-library/src/classes/pointer.rs

use std::ops::Deref;

/// Base type class for heap-allocated pointers.
///
/// This is the minimal abstraction: any type that can wrap a value and
/// dereference to it. Does NOT require Clone — that's added by subtraits.
///
/// ### Type Signature (Haskell-like)
///
/// `class Pointer p where`
/// `  type Of :: Type -> Type`
/// `  new :: a -> p a`
///
/// ### Implementors
///
/// - `RcBrand`: `Of<T> = Rc<T>`
/// - `ArcBrand`: `Of<T> = Arc<T>`
/// - `BoxBrand` (future): `Of<T> = Box<T>`
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::pointer::*};
///
/// // Generic over any pointer type
/// fn wrap_value<P: Pointer>(value: i32) -> P::Of<i32> {
///     P::new(value)
/// }
///
/// let boxed = wrap_value::<BoxBrand>(42);  // Future: Box<i32>
/// let rc = wrap_value::<RcBrand>(42);      // Rc<i32>
/// ```
pub trait Pointer {
    /// The pointer type constructor.
    /// For `RcBrand`, this is `Rc<T>`. For `BoxBrand`, this would be `Box<T>`.
    type Of<T: ?Sized>: Deref<Target = T>;

    /// Wraps a sized value in the pointer.
    ///
    /// ### Type Signature
    ///
    /// `forall a. Pointer p => a -> p a`
    fn new<T>(value: T) -> Self::Of<T>
    where
        Self::Of<T>: Sized;
}

/// Extension trait for reference-counted pointers with shared ownership.
///
/// Adds `CloneableOf` associated type which is Clone + Deref. This follows
/// the pattern of `SendClonableFn` adding `SendOf` to `ClonableFn`.
///
/// ### Why a Separate Associated Type?
///
/// Rust doesn't allow subtraits to add bounds to inherited associated types.
/// By adding `CloneableOf`, we can express "Clone + Deref" without modifying
/// `Pointer::Of`. For Rc/Arc, both types are identical; for Box, only `Of`
/// would be implemented.
///
/// ### Type Signature (Haskell-like)
///
/// `class Pointer p => RefCountedPointer p where`
/// `  type CloneableOf :: Type -> Type`
/// `  cloneable_new :: a -> p a`
///
/// ### Implementors
///
/// - `RcBrand`: `CloneableOf<T> = Rc<T>` (same as `Of<T>`)
/// - `ArcBrand`: `CloneableOf<T> = Arc<T>` (same as `Of<T>`)
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::pointer::*};
///
/// // Requires Clone capability
/// fn shared_value<P: RefCountedPointer>(value: i32) -> P::CloneableOf<i32> {
///     P::cloneable_new(value)
/// }
///
/// let rc = shared_value::<RcBrand>(42);  // Rc<i32>, can clone
/// ```
pub trait RefCountedPointer: Pointer {
    /// The clonable pointer type constructor.
    /// For Rc/Arc, this is the same as `Of<T>`.
    type CloneableOf<T: ?Sized>: Clone + Deref<Target = T>;

    /// Wraps a sized value in a clonable pointer.
    ///
    /// ### Type Signature
    ///
    /// `forall a. RefCountedPointer p => a -> p a`
    fn cloneable_new<T>(value: T) -> Self::CloneableOf<T>
    where
        Self::CloneableOf<T>: Sized;
}

/// Marker trait for thread-safe reference-counted pointers.
///
/// This follows the same pattern as `SendClonableFn` extends `ClonableFn`.
/// Only implemented by brands whose `CloneableOf` type is `Send + Sync` when
/// the inner type is `Send + Sync` (i.e., `ArcBrand` but not `RcBrand`).
///
/// ### Design Rationale
///
/// Unlike `SendClonableFn` which adds `SendOf`, this is a marker trait because:
/// - For Arc, `CloneableOf<T: Send+Sync>` is naturally `Send + Sync`
/// - No need for a separate type — the bound propagates through T
/// - Simpler API: check `P: SendRefCountedPointer` for thread safety
///
/// ### Implementors
///
/// - `ArcBrand`: Implements this marker
/// - `RcBrand`: Does NOT implement (Rc is !Send)
pub trait SendRefCountedPointer: RefCountedPointer
where
    for<T: Send + Sync> Self::CloneableOf<T>: Send + Sync,
{
}
````

#### Free Functions

```rust
// fp-library/src/classes/pointer.rs (continued)

/// Wraps a value in a pointer.
///
/// ### Type Signature
///
/// `forall p a. Pointer p => a -> p a`
pub fn pointer_new<P: Pointer, T>(value: T) -> P::Of<T>
where
    P::Of<T>: Sized,
{
    P::new(value)
}

/// Wraps a value in a clonable pointer.
///
/// ### Type Signature
///
/// `forall p a. RefCountedPointer p => a -> p a`
pub fn ref_counted_new<P: RefCountedPointer, T>(value: T) -> P::CloneableOf<T>
where
    P::CloneableOf<T>: Sized,
{
    P::cloneable_new(value)
}
```

#### Brand Definitions

```rust
// fp-library/src/brands.rs

use std::marker::PhantomData;
use crate::classes::pointer::RefCountedPointer;

/// Brand for `std::rc::Rc` reference-counted pointer.
///
/// Implements: `Pointer`, `RefCountedPointer`
/// Does NOT implement: `SendRefCountedPointer` (Rc is !Send)
///
/// Use this for single-threaded code where cheap cloning with shared
/// ownership is needed.
pub struct RcBrand;

/// Brand for `std::sync::Arc` atomic reference-counted pointer.
///
/// Implements: `Pointer`, `RefCountedPointer`, `SendRefCountedPointer`
///
/// Use this for multi-threaded code requiring `Send + Sync` shared ownership.
pub struct ArcBrand;

/// Brand for `std::boxed::Box` unique ownership pointer.
///
/// Implements: `Pointer` only (Box is not Clone unless T: Clone)
/// Does NOT implement: `RefCountedPointer`
///
/// Reserved for future use with recursive types, trampolines, etc.
pub struct BoxBrand;

/// Generic function brand parameterized by reference-counted pointer choice.
/// This replaces the separate `RcFnBrand` and `ArcFnBrand` types.
///
/// Requires `RefCountedPointer` because clonable functions need Clone.
pub struct FnBrand<PtrBrand: RefCountedPointer>(PhantomData<PtrBrand>);

/// Type alias for Rc-based function wrapper (convenience).
pub type RcFnBrand = FnBrand<RcBrand>;

/// Type alias for Arc-based function wrapper (convenience).
pub type ArcFnBrand = FnBrand<ArcBrand>;
```

#### Pointer Implementations

```rust
// fp-library/src/types/rc_ptr.rs

use crate::{
    brands::RcBrand,
    classes::pointer::{Pointer, RefCountedPointer},
};
use std::rc::Rc;

impl Pointer for RcBrand {
    type Of<T: ?Sized> = Rc<T>;

    fn new<T>(value: T) -> Rc<T> {
        Rc::new(value)
    }
}

impl RefCountedPointer for RcBrand {
    type CloneableOf<T: ?Sized> = Rc<T>;  // Same as Of<T>

    fn cloneable_new<T>(value: T) -> Rc<T> {
        Rc::new(value)
    }
}

// Note: RcBrand does NOT implement SendRefCountedPointer
// because Rc<T> is !Send regardless of T
```

```rust
// fp-library/src/types/arc_ptr.rs

use crate::{
    brands::ArcBrand,
    classes::pointer::{Pointer, RefCountedPointer, SendRefCountedPointer},
};
use std::sync::Arc;

impl Pointer for ArcBrand {
    type Of<T: ?Sized> = Arc<T>;

    fn new<T>(value: T) -> Arc<T> {
        Arc::new(value)
    }
}

impl RefCountedPointer for ArcBrand {
    type CloneableOf<T: ?Sized> = Arc<T>;  // Same as Of<T>

    fn cloneable_new<T>(value: T) -> Arc<T> {
        Arc::new(value)
    }
}

// ArcBrand implements SendRefCountedPointer because
// Arc<T: Send+Sync> is Send+Sync
impl SendRefCountedPointer for ArcBrand {}
```

```rust
// fp-library/src/types/box_ptr.rs (FUTURE - not implemented in this work)

use crate::{brands::BoxBrand, classes::pointer::Pointer};

impl Pointer for BoxBrand {
    type Of<T: ?Sized> = Box<T>;

    fn new<T>(value: T) -> Box<T> {
        Box::new(value)
    }
}

// BoxBrand does NOT implement RefCountedPointer
// because Box<T> is only Clone when T: Clone (not unconditional sharing)
```

#### Implementation Summary

| Brand | `Pointer::Of<T>` | `RefCountedPointer::CloneableOf<T>` | `SendRefCountedPointer` |
|-------|------------------|-------------------------------------|------------------------|
| `RcBrand` | `Rc<T>` | `Rc<T>` (same) | ❌ |
| `ArcBrand` | `Arc<T>` | `Arc<T>` (same) | ✅ |
| `BoxBrand` | `Box<T>` | N/A (not impl) | N/A |

### Part 2: Refactored ClonableFn Using RefCountedPointer

This is a **required** part of the design, not optional. `ClonableFn` will be refactored to use `RefCountedPointer` as its foundation.

#### The Unsized Coercion Problem

**Problem**: `RefCountedPointer::cloneable_new` accepts `T` (sized), but `ClonableFn` needs to create `CloneableOf<dyn Fn(A) -> B>` (unsized).

**Why this happens**: When you write `Rc::new(closure)`, Rust performs implicit unsized coercion because it knows the target type. But `cloneable_new` is generic and can't know the target type.

**Solution**: Use a macro to implement `ClonableFn` for each `FnBrand<PtrBrand>` variant. The macro handles the unsized coercion by explicitly calling `Rc::new` or `Arc::new`.

#### Implementation Using Macro

```rust
// fp-library/src/types/fn_brand.rs

use crate::{
    brands::{ArcBrand, FnBrand, RcBrand},
    classes::{
        category::Category,
        clonable_fn::ClonableFn,
        function::Function,
        semigroupoid::Semigroupoid,
        send_clonable_fn::SendClonableFn,
        pointer::{RefCountedPointer, SendRefCountedPointer},
    },
};
use std::{rc::Rc, sync::Arc};

/// Macro to implement ClonableFn for FnBrand<PtrBrand>.
///
/// This handles the unsized coercion which can't be done generically.
/// Each FnBrand<PtrBrand> implementation uses the pointer brand's
/// CloneableOf type for its function wrapper.
macro_rules! impl_fn_brand {
    ($ptr_brand:ty, $ptr_type:ident) => {
        impl Function for FnBrand<$ptr_brand> {
            // Uses pointer brand's CloneableOf to wrap dyn Fn
            type Of<'a, A, B> = $ptr_type<dyn 'a + Fn(A) -> B>;

            fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
                // Direct $ptr_type::new handles unsized coercion
                $ptr_type::new(f)
            }
        }

        impl ClonableFn for FnBrand<$ptr_brand> {
            type Of<'a, A, B> = $ptr_type<dyn 'a + Fn(A) -> B>;

            fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
                $ptr_type::new(f)
            }
        }

        impl Semigroupoid for FnBrand<$ptr_brand> {
            fn compose<'a, B: 'a, D: 'a, C: 'a>(
                f: Self::Of<'a, C, D>,
                g: Self::Of<'a, B, C>,
            ) -> Self::Of<'a, B, D> {
                <Self as ClonableFn>::new(move |b| f(g(b)))
            }
        }

        impl Category for FnBrand<$ptr_brand> {
            fn identity<'a, A>() -> Self::Of<'a, A, A> {
                $ptr_type::new(|a| a)
            }
        }
    };
}

// Apply macro for both brands
impl_fn_brand!(RcBrand, Rc);
impl_fn_brand!(ArcBrand, Arc);

// SendClonableFn is only implemented for FnBrand<ArcBrand>
// because ArcBrand: SendRefCountedPointer
impl SendClonableFn for FnBrand<ArcBrand> {
    type SendOf<'a, A, B> = Arc<dyn 'a + Fn(A) -> B + Send + Sync>;

    fn send_clonable_fn_new<'a, A, B>(
        f: impl 'a + Fn(A) -> B + Send + Sync
    ) -> Self::SendOf<'a, A, B> {
        Arc::new(f)
    }
}

// Note: FnBrand<RcBrand> does NOT implement SendClonableFn
// because RcBrand does NOT implement SendRefCountedPointer
```

#### Why Not Generic Implementation?

One might ask: why not implement `ClonableFn` generically for all `FnBrand<P: RefCountedPointer>`?

```rust
// This DOES NOT WORK due to unsized coercion limitations:
impl<P: RefCountedPointer> ClonableFn for FnBrand<P> {
    type Of<'a, A, B> = P::CloneableOf<dyn 'a + Fn(A) -> B>;

    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
        P::cloneable_new(f)  // ERROR: can't coerce sized closure to unsized dyn Fn
    }
}
```

The problem is that Rust's unsized coercion (`impl Fn -> dyn Fn`) only works when the compiler knows the concrete target type at the call site. In generic code, `P::cloneable_new(f)` doesn't provide enough information for the compiler to perform the coercion.

#### Alternative: Specialization-Based Approach (Nightly Only)

If using nightly Rust, specialization could provide a cleaner solution:

```rust
#![feature(specialization)]

impl<PtrBrand: RefCountedPointer> ClonableFn for FnBrand<PtrBrand> {
    type Of<'a, A, B> = <PtrBrand as RefCountedPointer>::CloneableOf<dyn 'a + Fn(A) -> B>;

    default fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
        unimplemented!("Specialized implementation required")
    }
}

impl ClonableFn for FnBrand<RcBrand> {
    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
        Rc::new(f)  // Concrete type allows unsized coercion
    }
}

impl ClonableFn for FnBrand<ArcBrand> {
    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
        Arc::new(f)  // Concrete type allows unsized coercion
    }
}
```

**Recommendation**: Use the macro approach for stable Rust compatibility.

#### Relationship to RefCountedPointer

The `FnBrand<PtrBrand>` pattern demonstrates how library types build on `RefCountedPointer`:

```
RefCountedPointer (trait)
    │
    ├── RcBrand (impl)
    │      └── FnBrand<RcBrand> → ClonableFn using Rc<dyn Fn>
    │
    └── ArcBrand (impl SendRefCountedPointer)
           └── FnBrand<ArcBrand> → ClonableFn + SendClonableFn using Arc<dyn Fn>
```

The `FnBrand` constraint requires `PtrBrand: RefCountedPointer` because:

1. **Clonability**: `ClonableFn::Of` must be `Clone` (satisfied by `CloneableOf`)
2. **Deref**: Function wrappers must deref to `dyn Fn` (satisfied by `Deref`)
3. **new factory**: Creating wrapped functions requires `cloneable_new`

### Part 3: Lazy Type with Shared Memoization

The new `Lazy` type replaces the current value-semantic implementation with Haskell-like shared memoization.

#### Design Overview

The key insight is that `Lazy` needs **two** uses of the pointer brand:

1. **Outer wrapper**: `P::CloneableOf<(OnceCell, Thunk)>` — enables cheap cloning that shares memoization
2. **Thunk storage**: `FnBrand<P>::Of<(), A>` — stores the computation as a clonable function

By parameterizing on `RefCountedPointer`, both uses share the same pointer brand (Rc or Arc), ensuring consistency.

#### Core Structure

````rust
// fp-library/src/types/lazy.rs (replacement)

use crate::{
    brands::{FnBrand, LazyBrand, OnceCellBrand, OnceLockBrand, RcBrand, ArcBrand},
    classes::{
        clonable_fn::ClonableFn,
        defer::Defer,
        monoid::Monoid,
        once::Once,
        semigroup::Semigroup,
        pointer::RefCountedPointer,
    },
    impl_kind,
    kinds::*,
};

/// Lazily-computed value with shared memoization (Haskell-like semantics).
///
/// Cloning a `Lazy` shares the memoization state via the underlying reference-counted
/// pointer. When any clone is forced, all clones see the cached result.
///
/// ### Type Parameters
///
/// * `PtrBrand`: Reference-counted pointer brand (`RcBrand` or `ArcBrand`)
/// * `OnceBrand`: Once cell brand (`OnceCellBrand` or `OnceLockBrand`)
/// * `A`: The type of the lazily-computed value
///
/// ### Shared Memoization
///
/// Unlike value-semantic lazy evaluation, this type provides true computation sharing:
///
/// ```text
/// Lazy clone semantics:
///
///   lazy1 ────┐
///             │
///             ▼
///   ┌─────────────────────────────────┐
///   │  RefCounted<OnceCell<A>, Thunk> │  ← Single shared allocation
///   └─────────────────────────────────┘
///             ▲
///             │
///   lazy2 ────┘
///
/// When lazy1 is forced:
///   - OnceCell computes and caches the value
///   - lazy2 sees the cached result immediately
/// ```
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
/// use std::cell::Cell;
/// use std::rc::Rc;
///
/// let counter = Rc::new(Cell::new(0));
/// let counter_clone = counter.clone();
///
/// // Create lazy value with memoized computation
/// let lazy = Lazy::<RcBrand, OnceCellBrand, _>::new(
///     clonable_fn_new::<RcFnBrand, _, _>(move |_| {
///         counter_clone.set(counter_clone.get() + 1);
///         42
///     })
/// );
///
/// let lazy2 = lazy.clone();  // Shares memoization state!
///
/// assert_eq!(counter.get(), 0);       // Not yet computed
/// assert_eq!(Lazy::force(&lazy), 42); // First force computes
/// assert_eq!(counter.get(), 1);       // Computed once
/// assert_eq!(Lazy::force(&lazy2), 42);// Second force uses cache
/// assert_eq!(counter.get(), 1);       // NOT recomputed - shared!
/// ```
pub struct Lazy<'a, PtrBrand, OnceBrand, A>(
    // CloneableOf wraps the (OnceCell, Thunk) pair for shared ownership
    <PtrBrand as RefCountedPointer>::CloneableOf<(
        <OnceBrand as Once>::Of<A>,
        <FnBrand<PtrBrand> as ClonableFn>::Of<'a, (), A>,
    )>,
)
where
    PtrBrand: RefCountedPointer,
    OnceBrand: Once;
````

**Key design decision**: The `Lazy` type uses `RefCountedPointer::CloneableOf` (not `Pointer::Of`) because:

1. **Clone requirement**: `Lazy::clone()` must be cheap (reference count increment)
2. **FnBrand constraint**: `FnBrand<P>` requires `P: RefCountedPointer`
3. **Consistency**: Same pointer brand for both outer wrapper and thunk storage

#### Implementation

```rust
impl<'a, PtrBrand, OnceBrand, A> Lazy<'a, PtrBrand, OnceBrand, A>
where
    PtrBrand: RefCountedPointer,
    OnceBrand: Once,
{
    /// Creates a new `Lazy` value from a thunk.
    ///
    /// ### Type Signature
    ///
    /// `new :: (() -> A) -> Lazy A`
    pub fn new(thunk: <FnBrand<PtrBrand> as ClonableFn>::Of<'a, (), A>) -> Self {
        Self(PtrBrand::cloneable_new((OnceBrand::new(), thunk)))
    }

    /// Forces the evaluation and returns the value.
    ///
    /// Takes `&self` because all clones share the same memoization state.
    /// The value is computed at most once across all clones.
    ///
    /// ### Type Signature
    ///
    /// `force :: Lazy A -> A`
    pub fn force(this: &Self) -> A
    where
        A: Clone,
    {
        let (once_cell, thunk) = &*this.0;  // Deref through pointer
        <OnceBrand as Once>::get_or_init(once_cell, || thunk(())).clone()
    }
    
    /// Returns the inner value if already computed, None otherwise.
    ///
    /// Does NOT force evaluation.
    pub fn try_get(this: &Self) -> Option<A>
    where
        A: Clone,
    {
        let (once_cell, _thunk) = &*this.0;
        <OnceBrand as Once>::get(once_cell).cloned()
    }
}

impl<'a, PtrBrand, OnceBrand, A> Clone for Lazy<'a, PtrBrand, OnceBrand, A>
where
    PtrBrand: RefCountedPointer,
    OnceBrand: Once,
{
    fn clone(&self) -> Self {
        // Cheap Rc/Arc clone - shares memoization state
        // This is O(1) regardless of A's size
        Self(self.0.clone())
    }
}
```

#### Convenience Type Aliases

```rust
// fp-library/src/types/lazy.rs (continued)

/// Single-threaded lazy value using Rc + OnceCell.
/// Not Send or Sync.
///
/// Use this for single-threaded code where you need lazy evaluation
/// with shared memoization.
pub type RcLazy<'a, A> = Lazy<'a, RcBrand, OnceCellBrand, A>;

/// Thread-safe lazy value using Arc + OnceLock.
/// Send and Sync when A: Send + Sync.
///
/// Use this for multi-threaded code where lazy values may be
/// shared across threads.
pub type ArcLazy<'a, A> = Lazy<'a, ArcBrand, OnceLockBrand, A>;
```

#### Type Class Implementations

```rust
// Semigroup: combine lazy values by combining their results
impl<'a, PtrBrand, OnceBrand, A> Semigroup for Lazy<'a, PtrBrand, OnceBrand, A>
where
    PtrBrand: RefCountedPointer,
    OnceBrand: Once,
    A: Semigroup + Clone + 'a,
{
    fn combine(x: Self, y: Self) -> Self {
        Lazy::new(<FnBrand<PtrBrand> as ClonableFn>::new(move |_| {
            A::combine(Lazy::force(&x), Lazy::force(&y))
        }))
    }
}

// Monoid: empty lazy value that produces the identity
impl<'a, PtrBrand, OnceBrand, A> Monoid for Lazy<'a, PtrBrand, OnceBrand, A>
where
    PtrBrand: RefCountedPointer,
    OnceBrand: Once,
    A: Monoid + Clone + 'a,
{
    fn empty() -> Self {
        Lazy::new(<FnBrand<PtrBrand> as ClonableFn>::new(|_| A::empty()))
    }
}

// Defer: create lazy from a thunk-producing thunk
impl<PtrBrand, OnceBrand> Defer for LazyBrand<PtrBrand, OnceBrand>
where
    PtrBrand: RefCountedPointer,
    OnceBrand: Once,
{
    fn defer<'a, A>(thunk: impl 'a + Fn() -> Self::Of<'a, A>) -> Self::Of<'a, A>
    where
        A: Clone + 'a,
    {
        // Create lazy that, when forced, forces the inner lazy
        Lazy::new(<FnBrand<PtrBrand> as ClonableFn>::new(move |_| {
            Lazy::force(&thunk())
        }))
    }
}
```

#### Thread Safety Analysis

| Type Alias | Pointer | OnceCell | Send | Sync | Use Case |
|------------|---------|----------|------|------|----------|
| `RcLazy<A>` | `Rc` | `OnceCell` | ❌ | ❌ | Single-threaded |
| `ArcLazy<A>` | `Arc` | `OnceLock` | ✅\* | ✅\* | Multi-threaded |

\*When `A: Send + Sync`

**Invalid combinations** (would compile but not be thread-safe):

* `Lazy<ArcBrand, OnceCellBrand, _>` — Arc is Send but OnceCell is not
* `Lazy<RcBrand, OnceLockBrand, _>` — Wastes OnceLock's thread-safety

## The type aliases enforce valid combinations by design.

## Challenges & Solutions

### Challenge 1: Unsized Coercion in ClonableFn

**Problem**: `RefCountedPointer::cloneable_new` accepts `T` (sized), but `ClonableFn` needs to create `CloneableOf<dyn Fn(A) -> B>` (unsized).

**Why this happens**: When you write `Rc::new(closure)`, Rust performs implicit unsized coercion because it knows the target type. But `cloneable_new` is generic and can't know the target type.

**Solution**: Use a macro to implement `ClonableFn` for `FnBrand<RcBrand>` and `FnBrand<ArcBrand>` separately. The macro explicitly calls `Rc::new` or `Arc::new`, allowing the coercion to happen.

```rust
macro_rules! impl_fn_brand {
    ($ptr_brand:ty, $ptr_type:ident) => {
        impl ClonableFn for FnBrand<$ptr_brand> {
            type Of<'a, A, B> = $ptr_type<dyn 'a + Fn(A) -> B>;
            fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
                $ptr_type::new(f)  // Unsized coercion happens here
            }
        }
    };
}

impl_fn_brand!(RcBrand, Rc);
impl_fn_brand!(ArcBrand, Arc);
```

**Why not other solutions?**

* **nightly `CoerceUnsized`**: Would work but limits to nightly Rust
* **`cloneable_new_unsized` method**: Can't pass unsized values by value
* **Specialization**: Also nightly-only

### Challenge 2: Thread Safety Bounds

**Problem**: `Arc<T>` is `Send + Sync` when `T: Send + Sync`. But `RefCountedPointer` is generic and can't enforce this at the trait level.

**Solution**: Use `SendRefCountedPointer` marker trait, following the same pattern as `SendClonableFn`:

```rust
/// Marker trait for thread-safe reference-counted pointers.
/// Mirrors the SendClonableFn pattern.
pub trait SendRefCountedPointer: RefCountedPointer
where
    for<T: Send + Sync> Self::CloneableOf<T>: Send + Sync,
{
}

// Only ArcBrand implements this
impl SendRefCountedPointer for ArcBrand {}

// RcBrand does NOT implement SendRefCountedPointer
```

**Usage in constraints**:

```rust
// Require thread-safe reference-counted pointer
fn parallel_operation<P: SendRefCountedPointer>(ptr: P::CloneableOf<Data>) { ... }
```

### Challenge 3: Interaction with Once Brands

**Problem**: `OnceCellBrand` uses `std::cell::OnceCell` (not `Send`). `OnceLockBrand` uses `std::sync::OnceLock` (`Send + Sync`). Invalid combinations would cause surprising behavior.

**Solution**: Use type aliases that enforce valid combinations:

```rust
/// Single-threaded lazy: RcBrand + OnceCellBrand
/// Neither Send nor Sync.
pub type RcLazy<'a, A> = Lazy<'a, RcBrand, OnceCellBrand, A>;

/// Thread-safe lazy: ArcBrand + OnceLockBrand
/// Send + Sync when A: Send + Sync.
pub type ArcLazy<'a, A> = Lazy<'a, ArcBrand, OnceLockBrand, A>;
```

Users CAN create invalid combinations like `Lazy<ArcBrand, OnceCellBrand, A>`, but:

1. The result won't be `Send + Sync` (fails at use site)
2. Documentation clearly recommends the type aliases
3. Compiler errors will guide users to correct usage

### Challenge 4: SendClonableFn Integration

**Problem**: The existing `SendClonableFn` trait has a separate `SendOf` associated type. How does this integrate with `FnBrand<PtrBrand>`?

**Solution**: `SendClonableFn` is only implemented for `FnBrand<ArcBrand>`:

```rust
impl SendClonableFn for FnBrand<ArcBrand> {
    type SendOf<'a, A, B> = Arc<dyn 'a + Fn(A) -> B + Send + Sync>;

    fn send_clonable_fn_new<'a, A, B>(
        f: impl 'a + Fn(A) -> B + Send + Sync
    ) -> Self::SendOf<'a, A, B> {
        Arc::new(f)
    }
}

// FnBrand<RcBrand> does NOT implement SendClonableFn
// because RcBrand does NOT implement SendRefCountedPointer
```

This maintains the same pattern: the extension trait is only implemented for thread-safe variants.

### Challenge 5: Why Not Just Use Pointer::Of for Lazy?

**Problem**: Why does `Lazy` use `RefCountedPointer::CloneableOf` instead of `Pointer::Of`?

**Reasoning**:

1. **Clone requirement**: `Lazy::clone()` must work unconditionally. The `CloneableOf` associated type guarantees `Clone` without requiring `T: Clone`.

2. **FnBrand constraint**: `FnBrand<P>` requires `P: RefCountedPointer`, not just `P: Pointer`. The thunk stored in `Lazy` must be clonable.

3. **Semantic consistency**: Using `CloneableOf` for both the outer wrapper and the thunk storage (via `FnBrand`) ensures both use the same pointer type (Rc or Arc).

**Alternative considered**: Could `Lazy` use `Pointer::Of` with a `where P::Of<T>: Clone` bound? Yes, but this would be more verbose and less clear about intent. The `RefCountedPointer` bound directly expresses "I need shared ownership semantics."

***

## Efficiency Analysis

### Lazy Performance Characteristics

| Scenario | Cost |
|----------|------|
| Create | `Rc/Arc::new(...)` ~20ns (heap allocation) |
| Clone | `Rc/Arc` clone ~3-5ns (reference count increment) |
| Force (first) | `Rc/Arc` deref + `OnceCell::get_or_init` + `thunk()` |
| Force (subsequent) | `Rc/Arc` deref + `OnceCell::get` + `clone()` |

### Comparison with Old Value-Semantic Lazy

| Operation | Old Lazy (Value) | New Lazy (Shared) |
|-----------|------------------|-------------------|
| Clone unforced | ~1ns (copy OnceCell) | ~3-5ns (Rc/Arc clone) |
| Clone forced | O(size of A) | ~3-5ns |
| Force 1 clone | O(thunk) | O(thunk) |
| Force 2nd clone | O(thunk) again! | O(1) - cached |
| Force nth clone | O(n × thunk) total | O(1) - all share |

**Conclusion**: New shared semantics is more efficient when:

* Multiple clones exist
* Thunk is expensive
* Value is large (expensive to clone)

Old value semantics was only better for:

* Single-use lazy values (rare use case)

***

## Implementation Phases

### Phase 1: Pointer Trait Foundation

1. Create `fp-library/src/classes/pointer.rs`
   * Define `Pointer` base trait with `Of<T>` and `new`
   * Define `RefCountedPointer` extension with `CloneableOf<T>` and `cloneable_new`
   * Define `SendRefCountedPointer` marker trait
   * Add free functions `pointer_new` and `ref_counted_new`
2. Add `RcBrand` and `ArcBrand` to `fp-library/src/brands.rs`
3. Create `fp-library/src/types/rc_ptr.rs` with `Pointer` and `RefCountedPointer` impls for `RcBrand`
4. Create `fp-library/src/types/arc_ptr.rs` with `Pointer`, `RefCountedPointer`, and `SendRefCountedPointer` impls for `ArcBrand`
5. Update module re-exports

### Phase 2: FnBrand Refactor

1. Add `FnBrand<PtrBrand: RefCountedPointer>` struct to `fp-library/src/brands.rs`
2. Add `RcFnBrand` and `ArcFnBrand` type aliases
3. Create `fp-library/src/types/fn_brand.rs`
   * Implement `Function`, `ClonableFn`, `Semigroupoid`, `Category` for `FnBrand<RcBrand>`
   * Implement same for `FnBrand<ArcBrand>`
   * Implement `SendClonableFn` for `FnBrand<ArcBrand>` only
   * Use macro to reduce duplication
4. Remove old `fp-library/src/types/rc_fn.rs` and `arc_fn.rs`
5. Update all code that referenced old brands

### Phase 3: Lazy Refactor

1. Rewrite `fp-library/src/types/lazy.rs`
   * Change to shared semantics using `RefCountedPointer::CloneableOf`
   * Use `FnBrand<PtrBrand>` for thunk storage
   * Reduce type parameters from 4 to 3
2. Add `RcLazy` and `ArcLazy` type aliases
3. Implement `Semigroup`, `Monoid`, `Defer` for new `Lazy`
4. Update `LazyBrand` and `impl_kind!`
5. Update all tests

### Phase 4: Integration & Polish

1. Update all documentation
2. Update `docs/std-coverage-checklist.md`
3. Update `docs/architecture.md` with new patterns
4. Ensure all tests pass
5. Run clippy and fix warnings
6. Generate and review documentation

***

## Files to Create

| File | Purpose |
|------|---------|
| `fp-library/src/classes/pointer.rs` | `Pointer`, `RefCountedPointer`, `SendRefCountedPointer` traits |
| `fp-library/src/types/rc_ptr.rs` | `Pointer` + `RefCountedPointer` impl for `RcBrand` |
| `fp-library/src/types/arc_ptr.rs` | All three traits impl for `ArcBrand` |
| `fp-library/src/types/fn_brand.rs` | `FnBrand<PtrBrand>` implementations |

## Files to Modify

| File | Changes |
|------|---------|
| `fp-library/src/brands.rs` | Add `RcBrand`, `ArcBrand`, `BoxBrand`, `FnBrand<P>`, type aliases |
| `fp-library/src/classes.rs` | Re-export `pointer` module |
| `fp-library/src/types.rs` | Re-export new modules, remove old |
| `fp-library/src/types/lazy.rs` | Complete rewrite with shared semantics using `RefCountedPointer` |
| `fp-library/src/functions.rs` | Re-export new free functions (`pointer_new`, `ref_counted_new`) |

## Files to Delete

| File | Reason |
|------|--------|
| `fp-library/src/types/rc_fn.rs` | Replaced by `fn_brand.rs` |
| `fp-library/src/types/arc_fn.rs` | Replaced by `fn_brand.rs` |

***

## Alternatives Considered

### Alternative 1: Separate SendRefCountedPointer with Own Associated Type

**Description**: Like `SendClonableFn`, have `SendRefCountedPointer` define its own `SendOf` type.

```rust
trait SendRefCountedPointer: RefCountedPointer {
    type SendOf<T: ?Sized + Send + Sync>: Clone + Send + Sync + Deref<Target = T>;
}
```

**Pros**: More explicit about thread-safe type
**Cons**: Duplication, harder to use generically
**Decision**: Rejected - marker trait is simpler and sufficient for this use case

### Alternative 2: No Pointer Trait, Just Refactor ClonableFn

**Description**: Keep separate `RcFnBrand`/`ArcFnBrand` but use them directly in `Lazy`.

**Pros**: Less new abstraction
**Cons**: Doesn't solve the problem of sharing semantics; FnBrand can't wrap arbitrary types
**Decision**: Rejected - need `RefCountedPointer` for `Lazy` to wrap `(OnceCell, Thunk)`

### Alternative 3: Keep Both Lazy Implementations

**Description**: Have `ValueLazy` (current) and `SharedLazy` (new).

**Pros**: No breaking change, both options available
**Cons**: Confusing API, maintenance burden, value semantics rarely useful
**Decision**: Rejected - clean break is better since backward compat isn't a goal

#### Detailed Analysis: Why Value-Semantic Lazy Has No Merit

The current implementation has a peculiar hybrid structure:

```rust
pub struct Lazy<'a, OnceBrand: Once, FnBrand: ClonableFn, A>(
    pub <OnceBrand as Once>::Of<A>,              // OnceCell<A> - DEEP cloned
    pub <FnBrand as ClonableFn>::Of<'a, (), A>,  // Rc<dyn Fn> - SHALLOW cloned (shared!)
);
```

This means cloning shares the thunk but not the memoization — the worst of both worlds:

| Behavior | Value-Semantic Lazy | Shared Lazy | Direct Function Call |
|----------|---------------------|-------------|---------------------|
| Clone + force both | Thunk runs **twice** | Thunk runs **once** | N/A |
| Memory per clone | OnceCell + Rc refcount | Rc refcount only | None |
| Computation sharing | **None** | Full | None |

**Potential use cases examined:**

1. **"I want independent computations per clone"** → Just call the function directly. Lazy adds OnceCell overhead without benefit.

2. **"I want snapshot isolation for impure thunks"** → Side-effectful thunks violate referential transparency. This is a bug, not a feature.

3. **"I want to avoid Rc/Arc overhead"** → The thunk is already wrapped in Rc via ClonableFn, so you pay the overhead anyway.

4. **"I want thread-local memoization"** → Use `thread_local!` with `OnceCell` directly, or use shared `Lazy` with thread-local access patterns.

5. **"I want deterministic destruction order"** → Use explicit resource management or `Drop` guards.

**Conclusion**: Every legitimate use case is better served by either:

* Shared `Lazy` (for memoization with sharing)
* `OnceCell` directly (for simple one-time initialization)
* Direct function application (for independent computation)

The value-semantic `Lazy` is an accidental design — not useful, just confusing.

***

## Design Decisions

### Trait Hierarchy: Why Pointer → RefCountedPointer → SendRefCountedPointer

The three-level trait hierarchy was chosen after careful analysis of naming and extensibility concerns.

#### Why Three Levels?

1. **`Pointer` (base)**: Minimal abstraction for any heap-allocated pointer — just `Deref<Target=T>` and `new`. This allows future `BoxBrand` support without reference counting.

2. **`RefCountedPointer` (extends Pointer)**: Adds `CloneableOf` associated type with `Clone` bound. This captures the key property of Rc/Arc: unconditional cheap cloning with shared state.

3. **`SendRefCountedPointer` (marker)**: Indicates thread safety. Unlike `SendClonableFn` which adds `SendOf`, this is a marker trait because `Arc<T: Send+Sync>` is naturally `Send+Sync` — no separate type needed.

#### Naming Decision: `Pointer` + `RefCountedPointer`

After considering multiple options, the final names were chosen for:

| Name | Rationale |
|------|-----------|
| `Pointer` | Minimal, accurate descriptor for `new` + `Deref` |
| `RefCountedPointer` | Precise — describes Rc/Arc's reference counting |
| `SendRefCountedPointer` | Follows `SendClonableFn` naming pattern |

**Rejected alternatives:**

* `SmartPointer`: Too broad — implies Box, Cow, etc.
* `SharedPtr`: C++ terminology, less precise
* `CloneablePtr`: Doesn't convey sharing semantics

### Why Additional Associated Type (CloneableOf) Instead of Marker Trait?

**Pattern**: Following `SendClonableFn`'s approach where subtraits add NEW associated types rather than marker traits.

**Reason**: Rust doesn't allow subtraits to strengthen bounds on inherited associated types:

```rust
// This DOES NOT work:
trait Pointer { type Of<T>: Deref; }
trait RefCountedPointer: Pointer { /* cannot add Clone to Of<T> */ }
```

By adding `CloneableOf`, `RefCountedPointer` can express "Clone + Deref" without modifying `Pointer::Of`.

### Extensibility Strategy

The design explicitly supports future extensibility:

#### What Works Now

| Brand | `Pointer` | `RefCountedPointer` | `SendRefCountedPointer` |
|-------|-----------|---------------------|-------------------------|
| `RcBrand` | ✅ | ✅ | ❌ |
| `ArcBrand` | ✅ | ✅ | ✅ |
| `BoxBrand` | ✅ (future) | ❌ | N/A |

#### Future Extensions (Out of Scope)

1. **`BoxBrand`**: Can implement `Pointer` only — `Box::clone()` requires `T: Clone`
2. **Custom allocators**: Third-party crates can implement the traits
3. **Weak reference support**: Could add `RefCountedPointer::downgrade()` later

```rust
// Example: future custom allocator support
impl Pointer for MyCustomRcBrand {
    type Of<T: ?Sized> = my_crate::CustomRc<T, MyAllocator>;
    
    fn new<T>(value: T) -> Self::Of<T> {
        my_crate::CustomRc::new_in(value, MyAllocator::default())
    }
}

impl RefCountedPointer for MyCustomRcBrand {
    type CloneableOf<T: ?Sized> = my_crate::CustomRc<T, MyAllocator>;
    
    fn cloneable_new<T>(value: T) -> Self::CloneableOf<T> {
        my_crate::CustomRc::new_in(value, MyAllocator::default())
    }
}
```

This allows the FP library's abstractions (Lazy, FnBrand, etc.) to work with custom allocators without library changes.

***

## References

* [Haskell's Data.Lazy](https://hackage.haskell.org/package/lazy)
* [PureScript's Data.Lazy](https://pursuit.purescript.org/packages/purescript-lazy)
* [std::rc::Rc documentation](https://doc.rust-lang.org/std/rc/struct.Rc.html)
* [std::sync::Arc documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html)
* [std::boxed::Box documentation](https://doc.rust-lang.org/std/boxed/struct.Box.html)
* [std::borrow::Cow documentation](https://doc.rust-lang.org/std/borrow/enum.Cow.html)
* [Existing SendClonableFn trait](../fp-library/src/classes/send_clonable_fn.rs)
* [Existing ClonableFn trait](../fp-library/src/classes/clonable_fn.rs)
* [Current Lazy implementation](../fp-library/src/types/lazy.rs)
