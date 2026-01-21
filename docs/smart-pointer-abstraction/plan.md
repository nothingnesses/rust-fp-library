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
│  │  Lazy<PtrBrand, OnceBrand, FnBrand, A>                                │  │
│  │    - Uses PtrBrand::CloneableOf for shared memoization                │  │
│  │    - Uses FnBrand for thunk storage (enables generic code)            │  │
│  │    - All clones share the same OnceCell                               │  │
│  │    - force_ref returns Result for panic safety                        │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  Type Aliases (for convenience):                                            │
│    RcFnBrand  = FnBrand<RcBrand>                                            │
│    ArcFnBrand = FnBrand<ArcBrand>                                           │
│    RcLazy     = Lazy<RcBrand, OnceCellBrand, RcFnBrand, A>                  │
│    ArcLazy    = Lazy<ArcBrand, OnceLockBrand, ArcFnBrand, A>                │
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

/// Extension trait for thread-safe reference-counted pointers.
///
/// This follows the same pattern as `SendClonableFn` extends `ClonableFn`,
/// adding a `SendOf` associated type with explicit `Send + Sync` bounds.
/// Only implemented by brands whose pointer type is `Send + Sync` when
/// the inner type is `Send + Sync` (i.e., `ArcBrand` but not `RcBrand`).
///
/// ### Design Rationale
///
/// Like `SendClonableFn` which adds `SendOf`, this trait adds an explicit
/// `SendOf` associated type with `Send + Sync` bounds because:
/// - Rust's `for<T: Trait>` syntax doesn't exist (only `for<'a>` works)
/// - Explicit bounds make the thread-safety contract clear in the type system
/// - Consistent with the established `SendClonableFn` pattern in this codebase
///
/// ### Implementors
///
/// - `ArcBrand`: `SendOf<T: Send+Sync> = Arc<T>` (Arc<T: Send+Sync> is Send+Sync)
/// - `RcBrand`: Does NOT implement (Rc is !Send)
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::pointer::*};
///
/// // Require thread-safe pointer
/// fn spawn_with_data<P: SendRefCountedPointer, T: Send + Sync>(
///     data: P::SendOf<T>
/// ) {
///     std::thread::spawn(move || {
///         // data is guaranteed Send + Sync
///     });
/// }
/// ```
pub trait SendRefCountedPointer: RefCountedPointer {
    /// The thread-safe pointer type constructor.
    /// For `ArcBrand`, this is `Arc<T>` where `T: Send + Sync`.
    type SendOf<T: ?Sized + Send + Sync>: Clone + Send + Sync + Deref<Target = T>;

    /// Wraps a sized value in a thread-safe pointer.
    ///
    /// ### Type Signature
    ///
    /// `forall a. (SendRefCountedPointer p, Send a, Sync a) => a -> p a`
    fn send_new<T: Send + Sync>(value: T) -> Self::SendOf<T>
    where
        Self::SendOf<T>: Sized;
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

/// Wraps a value in a thread-safe pointer.
///
/// ### Type Signature
///
/// `forall p a. (SendRefCountedPointer p, Send a, Sync a) => a -> p a`
pub fn send_ref_counted_new<P: SendRefCountedPointer, T: Send + Sync>(value: T) -> P::SendOf<T>
where
    P::SendOf<T>: Sized,
{
    P::send_new(value)
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

// ArcBrand implements SendRefCountedPointer with explicit SendOf type
impl SendRefCountedPointer for ArcBrand {
    type SendOf<T: ?Sized + Send + Sync> = Arc<T>;

    fn send_new<T: Send + Sync>(value: T) -> Arc<T> {
        Arc::new(value)
    }
}
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

| Brand | `Pointer::Of<T>` | `RefCountedPointer::CloneableOf<T>` | `SendRefCountedPointer::SendOf<T>` |
|-------|------------------|-------------------------------------|-----------------------------------|
| `RcBrand` | `Rc<T>` | `Rc<T>` (same) | ❌ Not implemented |
| `ArcBrand` | `Arc<T>` | `Arc<T>` (same) | `Arc<T>` (T: Send+Sync) |
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

1. **Outer wrapper**: `P::CloneableOf<LazyInner>` — enables cheap cloning that shares memoization
2. **Thunk storage**: `Option<FnBrand<P>::Of<(), A>>` — stores the computation, cleared after forcing

By parameterizing on `RefCountedPointer`, both uses share the same pointer brand (Rc or Arc), ensuring consistency.

#### ValidLazyCombination Marker Trait

To prevent invalid `PtrBrand`/`OnceBrand`/`FnBrand` combinations at compile time and to specify the correct thunk type for thread safety, we introduce a marker trait with an associated type:

````rust
/// Marker trait for valid Lazy pointer/once-cell/function-brand combinations.
///
/// This serves two purposes:
/// 1. **Compile-time validation**: Prevents semantically invalid combinations like:
///    - `Lazy<ArcBrand, OnceCellBrand, _, _>` — Arc is Send but OnceCell is not
///    - `Lazy<RcBrand, OnceLockBrand, _, _>` — Wastes OnceLock's synchronization overhead
///    - `Lazy<ArcBrand, OnceLockBrand, RcFnBrand, _>` — Pointer/function brand mismatch
///
/// 2. **Thunk type selection**: The `ThunkOf` associated type ensures thread-safe
///    combinations use `SendClonableFn::SendOf` (with `Send + Sync` bounds) while
///    single-threaded combinations use `ClonableFn::Of`.
///
/// ### Why ThunkOf Associated Type?
///
/// For `ArcLazy` to be `Send + Sync`, the thunk must also be `Send + Sync`.
/// Simply using `ClonableFn::Of` would give `Arc<dyn Fn>` without thread-safety
/// bounds. The `ThunkOf` associated type ensures `ArcLazy` uses
/// `Arc<dyn Fn + Send + Sync>` instead.
///
/// ### Implementors
///
/// - `(RcBrand, OnceCellBrand, RcFnBrand)`: Single-threaded lazy evaluation
///   - `ThunkOf` = `ClonableFn::Of` (no Send + Sync)
/// - `(ArcBrand, OnceLockBrand, ArcFnBrand)`: Thread-safe lazy evaluation
///   - `ThunkOf` = `SendClonableFn::SendOf` (with Send + Sync)
///
/// ### Extending for Custom Brands
///
/// Third-party `RefCountedPointer` implementations must also implement:
/// 1. `ThunkWrapper` for thunk storage
/// 2. `ValidLazyCombination<YourPtrBrand, YourOnceBrand, YourFnBrand>` for compile-time validation
///
/// Example:
/// ```rust
/// impl ValidLazyCombination<MyRcBrand, OnceCellBrand, FnBrand<MyRcBrand>> for () {
///     type ThunkOf<'a, A> = <FnBrand<MyRcBrand> as ClonableFn>::Of<'a, (), A>;
/// }
/// ```
pub trait ValidLazyCombination<PtrBrand, OnceBrand, FnBrand> {
    /// The thunk type to use for this combination.
    /// - For single-threaded: `ClonableFn::Of` (no Send + Sync bounds)
    /// - For thread-safe: `SendClonableFn::SendOf` (with Send + Sync bounds)
    type ThunkOf<'a, A>: Clone;
}

impl ValidLazyCombination<RcBrand, OnceCellBrand, RcFnBrand> for () {
    // Single-threaded: use ClonableFn::Of (no Send + Sync)
    type ThunkOf<'a, A> = <RcFnBrand as ClonableFn>::Of<'a, (), A>;
}

impl ValidLazyCombination<ArcBrand, OnceLockBrand, ArcFnBrand> for () {
    // Thread-safe: use SendClonableFn::SendOf (with Send + Sync)
    type ThunkOf<'a, A> = <ArcFnBrand as SendClonableFn>::SendOf<'a, (), A>;
}
````

#### Alternative Considered: 5th Type Parameter for Thunk Brand

An alternative approach was considered: adding a 5th type parameter `ThunkFnBrand` to `Lazy` alongside expanding `ValidLazyCombination`:

```rust
pub struct Lazy<'a, PtrBrand, OnceBrand, FnBrand, ThunkFnBrand, A>(...)
where
    (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand, ThunkFnBrand>;
```

**Why this was rejected:**

1. **Limited practical benefit**: The only scenario where `FnBrand ≠ ThunkFnBrand` would be "Rc wrapper but Arc thunk" — a rare and questionable use case
2. **Complexity**: 5 type parameters is unwieldy; type aliases become verbose
3. **Redundancy**: The associated type approach achieves the same goal with less API surface

The associated type on `ValidLazyCombination` is the cleaner solution — it simultaneously validates the combination AND specifies the correct thunk type.

#### Thunk Cleanup Strategy

To avoid retaining thunks (and their captured values) after forcing, the thunk is wrapped in `Option` and cleared after evaluation. The wrapper type varies by pointer brand:

| PtrBrand | Thunk Wrapper | Reason |
|----------|---------------|--------|
| `RcBrand` | `RefCell<Option<Thunk>>` | Single-threaded, interior mutability |
| `ArcBrand` | `Mutex<Option<Thunk>>` | Thread-safe, interior mutability |

This is abstracted via a `ThunkCell` type alias:

````rust
/// Type alias for thunk storage wrapper.
/// - For RcBrand: RefCell<Option<Thunk>>
/// - For ArcBrand: Mutex<Option<Thunk>>
pub type ThunkCell<PtrBrand, Thunk> = <PtrBrand as ThunkWrapper>::Cell<Thunk>;

/// Trait for pointer-brand-specific thunk wrapper.
///
/// Provides interior mutability for thunk storage, matching the
/// pointer brand's thread-safety model:
///
/// - `RcBrand` → `RefCell` (single-threaded interior mutability)
/// - `ArcBrand` → `Mutex` (thread-safe interior mutability)
///
/// ### Why Different Cell Types?
///
/// - `RefCell` provides runtime borrow checking without synchronization overhead
///   — appropriate for single-threaded `RcBrand`
/// - `Mutex` provides thread-safe access with synchronization
///   — required for multi-threaded `ArcBrand`
///
/// ### Extending for Custom Brands
///
/// Third-party `RefCountedPointer` implementations must also implement
/// `ThunkWrapper` to enable `Lazy` support:
///
/// ```rust
/// impl ThunkWrapper for MyRcBrand {
///     type Cell<T> = RefCell<Option<T>>;
///     fn new_cell<T>(value: Option<T>) -> Self::Cell<T> { RefCell::new(value) }
///     fn take<T>(cell: &Self::Cell<T>) -> Option<T> { cell.borrow_mut().take() }
/// }
/// ```
pub trait ThunkWrapper {
    type Cell<T>;
    fn new_cell<T>(value: Option<T>) -> Self::Cell<T>;
    fn take<T>(cell: &Self::Cell<T>) -> Option<T>;
}

impl ThunkWrapper for RcBrand {
    type Cell<T> = std::cell::RefCell<Option<T>>;
    fn new_cell<T>(value: Option<T>) -> Self::Cell<T> { RefCell::new(value) }
    fn take<T>(cell: &Self::Cell<T>) -> Option<T> { cell.borrow_mut().take() }
}

impl ThunkWrapper for ArcBrand {
    type Cell<T> = std::sync::Mutex<Option<T>>;
    fn new_cell<T>(value: Option<T>) -> Self::Cell<T> { Mutex::new(value) }
    // Gracefully handle mutex poisoning: if another thread panicked while
    // holding the lock, we can still safely take the value since we're just
    // moving it out of the Option.
    fn take<T>(cell: &Self::Cell<T>) -> Option<T> {
        cell.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
    }
}
````

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
        send_clonable_fn::SendClonableFn,
        pointer::{RefCountedPointer, ThunkWrapper, ValidLazyCombination},
    },
    impl_kind,
    kinds::*,
};

/// Inner state of a Lazy value, shared across clones.
///
/// Note: The `OnceCell` stores `Result<A, LazyError>` rather than just `A`.
/// This design choice enables panic-safe evaluation using only stable Rust
/// features (no `get_or_try_init` which is nightly-only).
struct LazyInner<'a, PtrBrand, OnceBrand, FnBrand, A>
where
    PtrBrand: RefCountedPointer + ThunkWrapper,
    OnceBrand: Once,
    FnBrand: ClonableFn,
    (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>,
{
    /// The memoized result (computed at most once).
    /// Stores `Result<A, LazyError>` to capture both successful values and errors.
    once: <OnceBrand as Once>::Of<Result<A, LazyError>>,
    /// The thunk, wrapped in ThunkWrapper::Cell for interior mutability.
    /// Cleared after forcing to free captured values.
    /// Uses `ValidLazyCombination::ThunkOf` to ensure correct Send+Sync bounds.
    thunk: <PtrBrand as ThunkWrapper>::Cell<
        <() as ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>>::ThunkOf<'a, A>
    >,
}

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
/// ### Valid Combinations
///
/// Only certain `PtrBrand`/`OnceBrand` combinations are valid:
/// - `RcBrand` + `OnceCellBrand`: Single-threaded lazy evaluation
/// - `ArcBrand` + `OnceLockBrand`: Thread-safe lazy evaluation
///
/// Invalid combinations (e.g., `ArcBrand` + `OnceCellBrand`) will fail at compile time
/// due to the `ValidLazyCombination` bound.
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
///   ┌─────────────────────────────────────────┐
///   │  RefCounted<OnceCell<A>, Option<Thunk>> │  ← Single shared allocation
///   └─────────────────────────────────────────┘
///             ▲
///             │
///   lazy2 ────┘
///
/// When lazy1 is forced:
///   1. OnceCell computes and caches the value
///   2. Thunk is cleared (set to None) to free captured values
///   3. lazy2 sees the cached result immediately
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
/// let lazy = RcLazy::new(
///     clonable_fn_new::<RcFnBrand, _, _>(move |_| {
///         counter_clone.set(counter_clone.get() + 1);
///         42
///     })
/// );
///
/// let lazy2 = lazy.clone();  // Shares memoization state!
///
/// assert_eq!(counter.get(), 0);             // Not yet computed
/// assert_eq!(Lazy::force(&lazy), 42);       // First force computes
/// assert_eq!(counter.get(), 1);             // Computed once
/// assert_eq!(Lazy::force(&lazy2), 42);      // Second force uses cache
/// assert_eq!(counter.get(), 1);             // NOT recomputed - shared!
/// // Thunk has been cleared, freeing counter_clone
/// ```
pub struct Lazy<'a, PtrBrand, OnceBrand, FnBrand, A>(
    // CloneableOf wraps LazyInner for shared ownership
    <PtrBrand as RefCountedPointer>::CloneableOf<LazyInner<'a, PtrBrand, OnceBrand, FnBrand, A>>,
)
where
    PtrBrand: RefCountedPointer + ThunkWrapper,
    OnceBrand: Once,
    FnBrand: ClonableFn,
    (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>;  // Enforces valid combinations
````

**Key design decision**: The `Lazy` type uses `RefCountedPointer::CloneableOf` (not `Pointer::Of`) because:

1. **Clone requirement**: `Lazy::clone()` must be cheap (reference count increment)
2. **FnBrand constraint**: `FnBrand<P>` requires `P: RefCountedPointer`
3. **Consistency**: Same pointer brand for both outer wrapper and thunk storage
4. **Thunk cleanup**: Using `Option<Thunk>` in a wrapper cell allows clearing after forcing

#### Implementation

```rust
/// Error type for lazy evaluation failures.
///
/// This is a unit struct because only one error state is reachable:
/// the thunk panicked during evaluation. The `Lazy` value becomes
/// poisoned and cannot be forced again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LazyError;

impl std::fmt::Display for LazyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "thunk panicked during evaluation")
    }
}

impl std::error::Error for LazyError {}

impl<'a, PtrBrand, OnceBrand, FnBrand, A> Lazy<'a, PtrBrand, OnceBrand, FnBrand, A>
where
    PtrBrand: RefCountedPointer + ThunkWrapper,
    OnceBrand: Once,
    FnBrand: ClonableFn,
    (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>,
{
    /// Creates a new `Lazy` value from a thunk.
    ///
    /// ### Type Signature
    ///
    /// `new :: (() -> A) -> Lazy A`
    ///
    /// ### Note
    ///
    /// The thunk type is determined by `ValidLazyCombination::ThunkOf`, which ensures
    /// thread-safe combinations use `SendClonableFn::SendOf` (with `Send + Sync` bounds).
    pub fn new(
        thunk: <() as ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>>::ThunkOf<'a, A>
    ) -> Self {
        Self(PtrBrand::cloneable_new(LazyInner {
            once: OnceBrand::new(),
            thunk: PtrBrand::new_cell(Some(thunk)),
        }))
    }

    /// Forces the evaluation and returns a reference to the value.
    ///
    /// Takes `&self` because all clones share the same memoization state.
    /// The value is computed at most once across all clones.
    /// After forcing, the thunk is cleared to free captured values.
    ///
    /// ### Type Signature
    ///
    /// `force_ref :: &Lazy A -> Result<&A, LazyError>`
    ///
    /// ### Errors
    ///
    /// Returns `Err(LazyError)` if the thunk panics during evaluation.
    ///
    /// ### Panic Safety
    ///
    /// If the thunk panics, the `Lazy` value becomes "poisoned":
    /// - The thunk is consumed (cannot be retried)
    /// - Subsequent calls return `Err(LazyError)`
    /// - All clones are affected (shared state)
    ///
    /// ### Implementation Note: AssertUnwindSafe
    ///
    /// The `catch_unwind` call uses `AssertUnwindSafe` to wrap the thunk invocation.
    /// This is safe because:
    /// 1. The thunk is taken (moved out) before invocation, so panic cannot leave it
    ///    in an inconsistent state
    /// 2. The `OnceCell` stores the `Result`, so panic state is captured, not ignored
    /// 3. No mutable references to shared state exist during thunk execution
    ///
    /// The invariant being upheld: after `take()`, the thunk field is `None` and stays
    /// that way regardless of whether the thunk panics. The `Result` in the `OnceCell`
    /// captures whether evaluation succeeded or failed.
    pub fn force_ref(this: &Self) -> Result<&A, LazyError> {
        let inner = &*this.0;  // Deref through pointer
        
        // Use get_or_init (stable) instead of get_or_try_init (nightly).
        // The cell stores Result<A, LazyError>, so we can use the stable API.
        let result: &Result<A, LazyError> = <OnceBrand as Once>::get_or_init(&inner.once, || {
            // Take the thunk (clears it to None).
            // This cannot be None because get_or_init guarantees the closure runs exactly once,
            // and we only take the thunk inside this closure.
            let thunk = PtrBrand::take(&inner.thunk)
                .expect("unreachable: get_or_init guarantees single execution");
            
            // Catch panics from the thunk.
            // AssertUnwindSafe is safe here - see doc comment above.
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| thunk(())))
                .map_err(|_| LazyError)
        });
        
        // Convert &Result<A, LazyError> to Result<&A, LazyError>
        result.as_ref().map_err(|_| LazyError)
    }

    /// Forces the evaluation and returns a cloned value.
    ///
    /// Takes `&self` because all clones share the same memoization state.
    /// The value is computed at most once across all clones.
    ///
    /// ### Type Signature
    ///
    /// `force :: &Lazy A -> Result<A, LazyError>`
    ///
    /// ### Note
    ///
    /// This clones the cached value on every call. If you need repeated
    /// access without cloning, use `force_ref` instead.
    pub fn force(this: &Self) -> Result<A, LazyError>
    where
        A: Clone,
    {
        Self::force_ref(this).map(Clone::clone)
    }

    /// Forces the evaluation, panicking on error.
    ///
    /// This is a convenience method for cases where panic on thunk failure
    /// is acceptable. Prefer `force_ref` or `force` for explicit error handling.
    ///
    /// ### Panics
    ///
    /// Panics if the thunk panicked or was already consumed.
    pub fn force_or_panic(this: &Self) -> A
    where
        A: Clone,
    {
        Self::force(this).expect("Lazy::force_or_panic failed")
    }
    /// Returns a reference to the inner value if already computed successfully, None otherwise.
    ///
    /// Does NOT force evaluation. Returns `None` if:
    /// - The value has not been forced yet
    /// - The thunk panicked during evaluation (value is `Err`)
    pub fn try_get_ref(this: &Self) -> Option<&A> {
        let inner = &*this.0;
        // Cell stores Result<A, LazyError>, so we need to unwrap Ok case
        <OnceBrand as Once>::get(&inner.once).and_then(|r| r.as_ref().ok())
    }

    /// Returns a cloned inner value if already computed successfully, None otherwise.
    ///
    /// Does NOT force evaluation. Returns `None` if:
    /// - The value has not been forced yet
    /// - The thunk panicked during evaluation
    pub fn try_get(this: &Self) -> Option<A>
    where
        A: Clone,
    {
        Self::try_get_ref(this).cloned()
    }
    
    /// Returns true if the value has been computed successfully.
    ///
    /// Returns `false` if:
    /// - The value has not been forced yet
    /// - The thunk panicked during evaluation (poisoned state)
    pub fn is_forced(this: &Self) -> bool {
        Self::try_get_ref(this).is_some()
    }
    
    /// Returns true if the thunk panicked during evaluation (poisoned state).
    ///
    /// Returns `false` if:
    /// - The value has not been forced yet
    /// - The value was computed successfully
    pub fn is_poisoned(this: &Self) -> bool {
        let inner = &*this.0;
        <OnceBrand as Once>::get(&inner.once)
            .map(|r| r.is_err())
            .unwrap_or(false)
    }
}

impl<'a, PtrBrand, OnceBrand, FnBrand, A> Clone for Lazy<'a, PtrBrand, OnceBrand, FnBrand, A>
where
    PtrBrand: RefCountedPointer + ThunkWrapper,
    OnceBrand: Once,
    FnBrand: ClonableFn,
    (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>,
{
    fn clone(&self) -> Self {
        // Cheap Rc/Arc clone - shares memoization state
        // This is O(1) regardless of A's size
        Self(self.0.clone())
    }
}

/// Debug implementation for Lazy when A: Debug.
///
/// Shows the current state: Unforced, Forced(value), or Poisoned.
impl<'a, PtrBrand, OnceBrand, FnBrand, A> std::fmt::Debug for Lazy<'a, PtrBrand, OnceBrand, FnBrand, A>
where
    PtrBrand: RefCountedPointer + ThunkWrapper,
    OnceBrand: Once,
    FnBrand: ClonableFn,
    (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>,
    A: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match Self::try_get_ref(self) {
            Some(value) => f.debug_tuple("Lazy::Forced").field(value).finish(),
            None if Self::is_poisoned(self) => write!(f, "Lazy::Poisoned"),
            None => write!(f, "Lazy::Unforced"),
        }
    }
}
```

**Note on `Once` trait**: The `Once` trait does NOT require `get_or_try_init` (which is nightly-only). Instead, we store `Result<A, LazyError>` in the cell and use the stable `get_or_init`:

```rust
pub trait Once {
    type Of<A>;
    
    fn new<A>() -> Self::Of<A>;
    fn get<A>(this: &Self::Of<A>) -> Option<&A>;
    fn get_or_init<A>(this: &Self::Of<A>, f: impl FnOnce() -> A) -> &A;
}
```

By storing `Result<A, LazyError>` (i.e., `Once::Of<Result<A, LazyError>>`), we can capture both success and error states using only stable Rust APIs. This is a deliberate design choice to avoid nightly-only features.

#### Convenience Type Aliases

```rust
// fp-library/src/types/lazy.rs (continued)

/// Single-threaded lazy value using Rc + OnceCell + RcFnBrand.
/// Not Send or Sync.
///
/// Use this for single-threaded code where you need lazy evaluation
/// with shared memoization.
///
/// ### Note
///
/// This is the only valid Rc-based Lazy configuration due to the
/// `ValidLazyCombination` constraint.
pub type RcLazy<'a, A> = Lazy<'a, RcBrand, OnceCellBrand, RcFnBrand, A>;

/// Thread-safe lazy value using Arc + OnceLock + ArcFnBrand.
/// Send and Sync when A: Send + Sync.
///
/// Use this for multi-threaded code where lazy values may be
/// shared across threads.
///
/// ### Note
///
/// This is the only valid Arc-based Lazy configuration due to the
/// `ValidLazyCombination` constraint.
pub type ArcLazy<'a, A> = Lazy<'a, ArcBrand, OnceLockBrand, ArcFnBrand, A>;
```

#### Type Class Implementations

##### TrySemigroup and TryMonoid Traits

To enable safe composition of lazy values without hidden panics, we introduce `TrySemigroup` and `TryMonoid` traits that return `Result`:

```rust
// fp-library/src/classes/try_semigroup.rs

/// A semigroup where the combine operation can fail.
///
/// This is useful for types like `Lazy` where forcing the underlying
/// computation may fail (e.g., if the thunk panicked).
///
/// ### Laws
///
/// - **Associativity**: `try_combine(try_combine(x, y)?, z)? == try_combine(x, try_combine(y, z)?)?`
///   (when all operations succeed)
///
/// ### Type Signature (Haskell-like)
///
/// `class TrySemigroup a where`
/// `  type Error :: Type`
/// `  try_combine :: a -> a -> Result a Error`
pub trait TrySemigroup: Sized {
    /// The error type returned when combining fails.
    type Error;
    
    /// Attempts to combine two values.
    ///
    /// Returns `Err` if either operand cannot be evaluated or if the
    /// combination itself fails.
    fn try_combine(x: Self, y: Self) -> Result<Self, Self::Error>;
}

// fp-library/src/classes/try_monoid.rs

/// A monoid where the combine operation can fail.
///
/// Extends `TrySemigroup` with an identity element. Unlike `Monoid::empty`,
/// the identity element for `TryMonoid` is always successful (no computation).
pub trait TryMonoid: TrySemigroup {
    /// Returns the identity element.
    ///
    /// This should never fail - it returns a value that, when combined
    /// with any other value, yields that other value.
    fn try_empty() -> Self;
}
```

##### Lazy Type Class Implementations

```rust
// TrySemigroup: safely combine lazy values, propagating errors
impl<'a, PtrBrand, OnceBrand, FnBrand, A> TrySemigroup for Lazy<'a, PtrBrand, OnceBrand, FnBrand, A>
where
    PtrBrand: RefCountedPointer + ThunkWrapper,
    OnceBrand: Once,
    FnBrand: ClonableFn,
    (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>,
    A: Semigroup + Clone + 'a,
{
    type Error = LazyError;
    
    fn try_combine(x: Self, y: Self) -> Result<Self, LazyError> {
        // Force both lazy values, propagating any errors
        let a = Lazy::force(&x)?;
        let b = Lazy::force(&y)?;
        
        // Create a new lazy value with the combined result (already computed)
        Ok(Lazy::new(<FnBrand as ClonableFn>::new(move |_| {
            A::combine(a.clone(), b.clone())
        })))
    }
}

// TryMonoid: identity element for lazy values
impl<'a, PtrBrand, OnceBrand, FnBrand, A> TryMonoid for Lazy<'a, PtrBrand, OnceBrand, FnBrand, A>
where
    PtrBrand: RefCountedPointer + ThunkWrapper,
    OnceBrand: Once,
    FnBrand: ClonableFn,
    (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>,
    A: Monoid + Clone + 'a,
{
    fn try_empty() -> Self {
        // The identity element never fails - it's just a constant
        Lazy::new(<FnBrand as ClonableFn>::new(|_| A::empty()))
    }
}

// Standard Semigroup: kept for compatibility, delegates to TrySemigroup
// Note: This can panic if underlying thunks fail. Prefer TrySemigroup for safety.
impl<'a, PtrBrand, OnceBrand, FnBrand, A> Semigroup for Lazy<'a, PtrBrand, OnceBrand, FnBrand, A>
where
    PtrBrand: RefCountedPointer + ThunkWrapper,
    OnceBrand: Once,
    FnBrand: ClonableFn,
    (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>,
    A: Semigroup + Clone + 'a,
{
    /// Combines two lazy values.
    ///
    /// # Panics
    ///
    /// Panics if either lazy value's thunk fails during evaluation.
    /// For safe composition, use `TrySemigroup::try_combine` instead.
    fn combine(x: Self, y: Self) -> Self {
        TrySemigroup::try_combine(x, y).expect("Semigroup::combine failed: use TrySemigroup for safe composition")
    }
}

// Standard Monoid: kept for compatibility
impl<'a, PtrBrand, OnceBrand, FnBrand, A> Monoid for Lazy<'a, PtrBrand, OnceBrand, FnBrand, A>
where
    PtrBrand: RefCountedPointer + ThunkWrapper,
    OnceBrand: Once,
    FnBrand: ClonableFn,
    (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>,
    A: Monoid + Clone + 'a,
{
    fn empty() -> Self {
        TryMonoid::try_empty()
    }
}

// Defer: create lazy from a thunk-producing thunk
impl<PtrBrand, OnceBrand, FnBrand> Defer for LazyBrand<PtrBrand, OnceBrand, FnBrand>
where
    PtrBrand: RefCountedPointer + ThunkWrapper,
    OnceBrand: Once,
    FnBrand: ClonableFn,
    (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>,
{
    fn defer<'a, A>(thunk: impl 'a + Fn() -> Self::Of<'a, A>) -> Self::Of<'a, A>
    where
        A: Clone + 'a,
    {
        // Optimised implementation: the inner Lazy is created on-demand
        // when this outer Lazy is forced, avoiding storing it as a captured value.
        // The inner Lazy is immediately forced, streaming the computation through
        // without extra heap allocation for the inner Lazy's result.
        Lazy::new(<FnBrand as ClonableFn>::new(move |_| {
            Lazy::force_or_panic(&thunk())
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

**Problem**: `Arc<T>` is `Send + Sync` when `T: Send + Sync`. But `RefCountedPointer` is generic and can't enforce this at the trait level. Rust's `for<T: Trait>` syntax does **not exist** (only `for<'a>` works for lifetimes).

**Solution**: Use `SendRefCountedPointer` with an explicit `SendOf` associated type, following the same pattern as `SendClonableFn` which adds `SendOf`:

```rust
/// Extension trait for thread-safe reference-counted pointers.
/// Adds SendOf associated type with explicit Send + Sync bounds.
pub trait SendRefCountedPointer: RefCountedPointer {
    type SendOf<T: ?Sized + Send + Sync>: Clone + Send + Sync + Deref<Target = T>;
    
    fn send_new<T: Send + Sync>(value: T) -> Self::SendOf<T>
    where
        Self::SendOf<T>: Sized;
}

// Only ArcBrand implements this
impl SendRefCountedPointer for ArcBrand {
    type SendOf<T: ?Sized + Send + Sync> = Arc<T>;
    
    fn send_new<T: Send + Sync>(value: T) -> Arc<T> {
        Arc::new(value)
    }
}

// RcBrand does NOT implement SendRefCountedPointer
```

**Why this pattern?**

* Rust's `for<T: Trait>` syntax doesn't exist (only `for<'a>` works)
* Follows the established `SendClonableFn` pattern in this codebase
* The `T: Send + Sync` bound and `SendOf: Send + Sync` bound make the contract explicit

**Usage in constraints**:

```rust
// Require thread-safe reference-counted pointer
fn parallel_operation<P: SendRefCountedPointer, T: Send + Sync>(ptr: P::SendOf<T>) {
    std::thread::spawn(move || {
        // ptr is guaranteed Send + Sync
    });
}
```

### Challenge 3: Interaction with Once Brands and Function Brands

**Problem**: `OnceCellBrand` uses `std::cell::OnceCell` (not `Send`). `OnceLockBrand` uses `std::sync::OnceLock` (`Send + Sync`). Additionally, the function brand must match the pointer brand for thread safety. Invalid combinations would cause surprising behavior or silent performance issues:

* `Lazy<ArcBrand, OnceCellBrand, _, _>` — Arc is Send but OnceCell is not, defeating the purpose
* `Lazy<RcBrand, OnceLockBrand, _, _>` — Wastes OnceLock's synchronization overhead
* `Lazy<ArcBrand, OnceLockBrand, RcFnBrand, _>` — Pointer/function brand mismatch breaks thread safety

**Solution**: Enforce valid combinations at compile time with a 3-parameter marker trait:

```rust
/// Marker trait for valid Lazy pointer/once-cell/function-brand combinations.
pub trait ValidLazyCombination<PtrBrand, OnceBrand, FnBrand> {}

impl ValidLazyCombination<RcBrand, OnceCellBrand, RcFnBrand> for () {}
impl ValidLazyCombination<ArcBrand, OnceLockBrand, ArcFnBrand> for () {}
```

The `Lazy` struct includes this in its where clause:

```rust
pub struct Lazy<'a, PtrBrand, OnceBrand, FnBrand, A>(...)
where
    PtrBrand: RefCountedPointer + ThunkWrapper,
    OnceBrand: Once,
    FnBrand: ClonableFn,
    (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>;  // Compile-time enforcement
```

**Benefits**:

1. Invalid combinations fail immediately at `Lazy::new` with clear error
2. Users cannot accidentally create suboptimal configurations
3. The marker trait explicitly documents valid combinations
4. Third-party crates can add their own valid combinations if needed
5. Function brand is explicitly part of the type, enabling generic code over both local and thread-safe variants

**Type aliases** still provide convenient defaults:

```rust
pub type RcLazy<'a, A> = Lazy<'a, RcBrand, OnceCellBrand, RcFnBrand, A>;
pub type ArcLazy<'a, A> = Lazy<'a, ArcBrand, OnceLockBrand, ArcFnBrand, A>;
```

**Thread-safe thunks**: For `ArcLazy` to be `Send + Sync`, the thunk must also be `Send + Sync`. This is achieved by using `ArcFnBrand` which stores `Arc<dyn Fn + Send + Sync>` when created via `send_clonable_fn_new`. The `ValidLazyCombination` ensures `ArcLazy` always uses `ArcFnBrand`.

Users can create thread-safe lazy values with:

```rust
let lazy: ArcLazy<i32> = ArcLazy::new(
    send_clonable_fn_new::<ArcFnBrand, _, _>(|_| 42)
);
// lazy is Send + Sync, can be shared across threads
std::thread::spawn(move || Lazy::force(&lazy));
```

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

### Challenge 6: FnBrand Extensibility for Third-Party Pointer Brands

**Problem**: The `impl_fn_brand!` macro handles unsized coercion by explicitly calling `Rc::new` or `Arc::new`. Third-party crates implementing custom `RefCountedPointer` brands cannot automatically get `FnBrand<CustomBrand>` implementations.

**Why this happens**: Rust's unsized coercion (`impl Fn` → `dyn Fn`) requires the compiler to know the concrete target type at the call site. In generic code like `P::cloneable_new(f)`, the compiler can't perform this coercion.

**Solution**: Introduce an `UnsizedCoercible` trait that abstracts the coercion operation, enabling automatic `FnBrand` implementations for any pointer brand that implements it:

````rust
// fp-library/src/classes/pointer.rs (continued)

/// Trait for pointer brands that can perform unsized coercion to `dyn Fn`.
///
/// This enables automatic `FnBrand<PtrBrand>` implementations for custom
/// pointer brands. The trait abstracts the unsized coercion that Rust
/// can only perform with concrete types.
///
/// ### Why This Trait?
///
/// Rust's unsized coercion (`impl Fn` → `dyn Fn`) requires the compiler to
/// know the concrete target type. In generic code like `P::cloneable_new(f)`,
/// this information isn't available. By moving the coercion into a trait
/// method, each implementor can provide the concrete type.
///
/// ### Implementors
///
/// - `RcBrand`: Coerces via `Rc::new`
/// - `ArcBrand`: Coerces via `Arc::new`
/// - Third-party brands: Implement using their pointer's `new` method
///
/// ### Examples
///
/// ```rust
/// // Third-party implementation:
/// impl UnsizedCoercible for MyRcBrand {
///     fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::CloneableOf<dyn 'a + Fn(A) -> B> {
///         MyRc::new(f)  // Concrete type enables unsized coercion
///     }
///
///     fn coerce_fn_send<'a, A, B>(f: impl 'a + Fn(A) -> B + Send + Sync) -> Self::CloneableOf<dyn 'a + Fn(A) -> B + Send + Sync>
///     where
///         Self: SendRefCountedPointer,
///     {
///         unimplemented!("MyRcBrand is not Send")
///     }
/// }
/// ```
pub trait UnsizedCoercible: RefCountedPointer {
    /// Coerces a sized closure to a `dyn Fn` wrapped in this pointer type.
    fn coerce_fn<'a, A, B>(
        f: impl 'a + Fn(A) -> B
    ) -> Self::CloneableOf<dyn 'a + Fn(A) -> B>;
    
    /// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync`.
    ///
    /// Only meaningful for `SendRefCountedPointer` implementors.
    /// Other implementors should `unimplemented!()` or panic.
    fn coerce_fn_send<'a, A, B>(
        f: impl 'a + Fn(A) -> B + Send + Sync
    ) -> Self::CloneableOf<dyn 'a + Fn(A) -> B + Send + Sync>;
}

impl UnsizedCoercible for RcBrand {
    fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Rc<dyn 'a + Fn(A) -> B> {
        Rc::new(f)
    }
    
    fn coerce_fn_send<'a, A, B>(
        _f: impl 'a + Fn(A) -> B + Send + Sync
    ) -> Rc<dyn 'a + Fn(A) -> B + Send + Sync> {
        panic!("RcBrand does not support Send + Sync functions")
    }
}

impl UnsizedCoercible for ArcBrand {
    fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Arc<dyn 'a + Fn(A) -> B> {
        Arc::new(f)
    }
    
    fn coerce_fn_send<'a, A, B>(
        f: impl 'a + Fn(A) -> B + Send + Sync
    ) -> Arc<dyn 'a + Fn(A) -> B + Send + Sync> {
        Arc::new(f)
    }
}
````

Now `FnBrand` can have a blanket implementation:

```rust
// fp-library/src/types/fn_brand.rs (updated)

/// Blanket implementation of ClonableFn for any FnBrand<P> where P: UnsizedCoercible.
///
/// This enables third-party pointer brands to automatically get FnBrand support
/// by implementing the UnsizedCoercible trait.
impl<P: UnsizedCoercible> Function for FnBrand<P> {
    type Of<'a, A, B> = P::CloneableOf<dyn 'a + Fn(A) -> B>;

    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
        P::coerce_fn(f)
    }
}

impl<P: UnsizedCoercible> ClonableFn for FnBrand<P> {
    type Of<'a, A, B> = P::CloneableOf<dyn 'a + Fn(A) -> B>;

    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
        P::coerce_fn(f)
    }
}

impl<P: UnsizedCoercible> Semigroupoid for FnBrand<P> {
    fn compose<'a, B: 'a, D: 'a, C: 'a>(
        f: Self::Of<'a, C, D>,
        g: Self::Of<'a, B, C>,
    ) -> Self::Of<'a, B, D> {
        P::coerce_fn(move |b| f(g(b)))
    }
}

impl<P: UnsizedCoercible> Category for FnBrand<P> {
    fn identity<'a, A>() -> Self::Of<'a, A, A> {
        P::coerce_fn(|a| a)
    }
}

// SendClonableFn only for UnsizedCoercible + SendRefCountedPointer
impl<P: UnsizedCoercible + SendRefCountedPointer> SendClonableFn for FnBrand<P> {
    type SendOf<'a, A, B> = P::CloneableOf<dyn 'a + Fn(A) -> B + Send + Sync>;

    fn send_clonable_fn_new<'a, A, B>(
        f: impl 'a + Fn(A) -> B + Send + Sync
    ) -> Self::SendOf<'a, A, B> {
        P::coerce_fn_send(f)
    }
}
```

**Benefits of `UnsizedCoercible` over macro approach**:

1. **Automatic support**: Third-party brands just implement `UnsizedCoercible` and get `FnBrand` for free
2. **Type-safe**: The trait enforces correct method signatures
3. **Discoverable**: Users can see what they need to implement via trait bounds
4. **Composable**: Works with existing `RefCountedPointer` hierarchy

**Third-party usage**:

```rust
// In third-party crate:
use fp_library::classes::pointer::{RefCountedPointer, UnsizedCoercible};

pub struct MyRcBrand;

impl Pointer for MyRcBrand { ... }
impl RefCountedPointer for MyRcBrand { ... }

// Just implement this trait to get FnBrand<MyRcBrand> support!
impl UnsizedCoercible for MyRcBrand {
    fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> MyRc<dyn 'a + Fn(A) -> B> {
        MyRc::new(f)
    }
    
    fn coerce_fn_send<'a, A, B>(_f: impl 'a + Fn(A) -> B + Send + Sync) -> MyRc<dyn 'a + Fn(A) -> B + Send + Sync> {
        panic!("MyRcBrand is not thread-safe")
    }
}

// FnBrand<MyRcBrand> now automatically implements ClonableFn!
```

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
   * Use separate `FnBrand` type parameter for thunk storage (4 type parameters total)
   * Store `Result<A, LazyError>` in OnceCell to enable panic-safe evaluation with stable Rust
   * Add `LazyError` unit struct (simplified from enum since only ThunkPanicked is reachable)
   * Change `force_ref` to return `Result<&A, LazyError>` using stable `get_or_init`
   * Add `force_or_panic` convenience method
   * Add `is_poisoned` method to check for thunk panic state
   * Add `Debug` implementation for `Lazy` where `A: Debug`
   * Use `ValidLazyCombination::ThunkOf` for thunk type selection (Send+Sync for Arc)
2. Add `RcLazy` and `ArcLazy` type aliases (4 parameters each: PtrBrand, OnceBrand, FnBrand, A)
3. Create `fp-library/src/classes/try_semigroup.rs` with `TrySemigroup` trait
4. Create `fp-library/src/classes/try_monoid.rs` with `TryMonoid` trait
5. Implement `TrySemigroup`, `TryMonoid`, `Semigroup`, `Monoid`, `Defer` for new `Lazy`
   * `Semigroup::combine` delegates to `TrySemigroup::try_combine` with panic on error
   * `TrySemigroup::try_combine` propagates errors safely
6. Update `LazyBrand` to take 3 parameters: `LazyBrand<PtrBrand, OnceBrand, FnBrand>`
7. Update `impl_kind!` for new `LazyBrand`
8. Update all tests to handle `Result` return type from `force_ref` and `force`

### Phase 4: Integration & Polish

1. Update all documentation
2. Update `docs/std-coverage-checklist.md`
3. Update `docs/architecture.md` with new patterns
4. Ensure all tests pass
5. Run clippy and fix warnings
6. Generate and review documentation

***

### Phase 5: Concurrency Testing with Loom

For thorough verification of the `ArcLazy` synchronization code, we use the `loom` crate for deterministic concurrency testing:

1. Add `loom` as a dev dependency in `fp-library/Cargo.toml`:
   ```toml
   [dev-dependencies]
   loom = "0.7"
   ```

2. Create `fp-library/tests/loom_tests.rs` with concurrent lazy tests:
   ```rust
   #![cfg(loom)]

   use loom::thread;
   use loom::sync::Arc;

   #[test]
   fn arc_lazy_concurrent_force() {
       loom::model(|| {
           // Create a lazy value that tracks execution count
           let counter = Arc::new(loom::sync::atomic::AtomicUsize::new(0));
           let counter_clone = counter.clone();
           
           let lazy = Arc::new(ArcLazy::new(
               send_clonable_fn_new::<ArcFnBrand, _, _>(move |_| {
                   counter_clone.fetch_add(1, loom::sync::atomic::Ordering::SeqCst);
                   42
               })
           ));
           
           let lazy1 = lazy.clone();
           let lazy2 = lazy.clone();
           
           let t1 = thread::spawn(move || Lazy::force(&*lazy1));
           let t2 = thread::spawn(move || Lazy::force(&*lazy2));
           
           let r1 = t1.join().unwrap();
           let r2 = t2.join().unwrap();
           
           // Both should succeed with the same value
           assert_eq!(r1, Ok(42));
           assert_eq!(r2, Ok(42));
           
           // Thunk should have been called exactly once
           assert_eq!(counter.load(loom::sync::atomic::Ordering::SeqCst), 1);
       });
   }

   #[test]
   fn arc_lazy_panic_propagation() {
       loom::model(|| {
           let lazy = Arc::new(ArcLazy::new(
               send_clonable_fn_new::<ArcFnBrand, _, _>(|_| -> i32 {
                   panic!("intentional test panic")
               })
           ));
           
           let lazy1 = lazy.clone();
           let lazy2 = lazy.clone();
           
           let t1 = thread::spawn(move || {
               std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                   Lazy::force_ref(&*lazy1)
               }))
           });
           let t2 = thread::spawn(move || {
               std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                   Lazy::force_ref(&*lazy2)
               }))
           });
           
           // Both threads should see the error (either ThunkPanicked from catching,
           // or the panic propagated)
           let r1 = t1.join().unwrap();
           let r2 = t2.join().unwrap();
           
           // At least one should have caught the error
           assert!(r1.is_ok() || r2.is_ok());
       });
   }
   ```

3. Run loom tests with:
   ```bash
   RUSTFLAGS="--cfg loom" cargo test --test loom_tests
   ```

**Why Loom?**

Loom exhaustively tests all possible thread interleavings, finding race conditions that random testing might miss. This is critical for verifying that:

1. `OnceLock::get_or_init` correctly synchronizes access
2. `Mutex::lock` on the thunk cell doesn't cause deadlocks
3. Panic propagation works correctly across threads
4. The memoized value is visible to all threads after forcing

***

## Files to Create

| File | Purpose |
|------|---------|
| `fp-library/src/classes/pointer.rs` | `Pointer`, `RefCountedPointer`, `SendRefCountedPointer`, `UnsizedCoercible` traits |
| `fp-library/src/classes/try_semigroup.rs` | `TrySemigroup` trait for fallible combination |
| `fp-library/src/classes/try_monoid.rs` | `TryMonoid` trait extending `TrySemigroup` |
| `fp-library/src/types/rc_ptr.rs` | `Pointer` + `RefCountedPointer` + `UnsizedCoercible` impl for `RcBrand` |
| `fp-library/src/types/arc_ptr.rs` | All four traits impl for `ArcBrand` |
| `fp-library/src/types/fn_brand.rs` | `FnBrand<PtrBrand>` blanket implementations |
| `fp-library/tests/loom_tests.rs` | Loom-based concurrency tests for `ArcLazy` |

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
    fn send_new<T: Send + Sync>(value: T) -> Self::SendOf<T>;
}
```

**Pros**:

* More explicit about thread-safe type
* Consistent with `SendClonableFn` pattern
* Type bounds are clear in the trait definition
* Required because `for<T: Trait>` syntax doesn't exist in Rust

**Cons**:

* `SendOf<T>` and `CloneableOf<T>` are the same type for Arc (just with different bounds)
* Slightly more API surface

**Decision**: ✅ Adopted - necessary because Rust's `for<T: Trait>` higher-ranked bounds don't exist. A marker trait with the invalid syntax would not compile.

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

3. **`SendRefCountedPointer` (extends RefCountedPointer)**: Indicates thread safety. Like `SendClonableFn` which adds `SendOf`, this trait adds a `SendOf` associated type with explicit `Send + Sync` bounds. This is required because Rust's `for<T: Trait>` higher-ranked bounds syntax doesn't exist (only `for<'a>` works).

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
