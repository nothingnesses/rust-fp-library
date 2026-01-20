# Smart Pointer Abstraction & Shared Lazy Evaluation

## Summary

This document outlines a plan to introduce a unified `SmartPointer` type class abstraction that allows library types to be parameterized over the choice of reference-counted smart pointer (`Rc` vs `Arc`). This abstraction enables:

1. **Unified Rc/Arc selection** across multiple library types
2. **Shared memoization semantics** for `Lazy` (Haskell-like behavior)
3. **Reduced code duplication** by building multiple types on a single foundation
4. **Future extensibility** for custom allocators or alternative smart pointers

**Note**: This is a breaking change. Backward compatibility is not a goal; the focus is on the best possible design.

---

## Background & Motivation

### Conversation Context

This plan originated from a code review of `fp-library/src/types/lazy.rs`, which revealed:

1. **The current `Lazy` implementation is correct** but uses value semantics:
   - Cloning a `Lazy` creates a deep copy of the `OnceCell`
   - Each clone maintains independent memoization state
   - Forcing one clone does not affect others

2. **This differs from Haskell's lazy evaluation**:
   - In Haskell, all references to a thunk share memoization
   - Once forced, all references see the cached result
   - This enables efficient graph-based computation sharing

3. **To achieve Haskell-like semantics**, the `OnceCell` must be wrapped in a shared smart pointer (`Rc` or `Arc`)

4. **The existing library already has similar patterns**:
   - `ClonableFn` abstracts over `RcFnBrand` vs `ArcFnBrand`
   - Users choose at call sites: `clonable_fn_new::<RcFnBrand, _, _>(...)`
   - This pattern can be generalized and unified

5. **`SendClonableFn` extends `ClonableFn`** with thread-safe semantics:
   - Uses a separate `SendOf` associated type
   - Only `ArcFnBrand` implements it (not `RcFnBrand`)
   - This pattern can be applied to `SmartPointer` → `SendSmartPointer`

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
│                     PROPOSED: Unified SmartPointer Foundation               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                    SmartPointer ─────extends───▶ SendSmartPointer           │
│                          │                             │                    │
│                  ┌───────┴───────┐                 ┌───┘                    │
│                  │               │                 │                        │
│               RcBrand         ArcBrand ◀───────────┘                        │
│                  │               │                                          │
│      ┌───────────┴───────────────┴───────────┐                              │
│      │                                       │                              │
│      ▼                                       ▼                              │
│  ┌────────────────────────────┐    ┌────────────────────────────┐           │
│  │ FnBrand<PtrBrand>          │    │ Lazy<PtrBrand, ...>        │           │
│  │                            │    │                            │           │
│  │ implements ClonableFn      │    │ Uses SmartPointer for      │           │
│  │ implements SendClonableFn  │    │ shared memoization         │           │
│  │   when PtrBrand: Send...   │    │                            │           │
│  └────────────────────────────┘    └────────────────────────────┘           │
│                                                                             │
│  Type Aliases (for convenience):                                            │
│    RcFnBrand  = FnBrand<RcBrand>                                            │
│    ArcFnBrand = FnBrand<ArcBrand>                                           │
│    RcLazy     = Lazy<RcBrand, OnceCellBrand, ...>                           │
│    ArcLazy    = Lazy<ArcBrand, OnceLockBrand, ...>                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Design Goals

### Primary Goals

1. **Introduce `SmartPointer` trait** as a foundational abstraction for `Rc` and `Arc`
2. **Refactor `ClonableFn` to use `SmartPointer`** via `FnBrand<PtrBrand>` pattern
3. **Create `Lazy` type** with Haskell-like shared memoization semantics (replacing current value-semantic `Lazy`)
4. **Use extension trait pattern** (`SendSmartPointer` extends `SmartPointer`) for thread safety, mirroring `SendClonableFn`
5. **Enable composition** - users pick one smart pointer brand and use it consistently

### Non-Goals

1. **Backward compatibility** - this is a breaking change; best design takes priority
2. **Migration path** - not needed since we're not maintaining backward compat
3. **Supporting non-reference-counted smart pointers** initially (e.g., `Box` - could be added later)
4. **Automatic selection** of Rc vs Arc based on context (user explicitly chooses)

---

## Technical Design

### Part 1: SmartPointer Type Class

#### Trait Definition

```rust
// fp-library/src/classes/smart_pointer.rs

use std::ops::Deref;

/// Type class for reference-counted smart pointers.
///
/// This trait abstracts over `Rc` and `Arc`, enabling library types
/// to be parameterized over the choice of smart pointer. Users select
/// the implementation at type level via brand types (`RcBrand`, `ArcBrand`).
///
/// ### Type Signature (Haskell-like)
///
/// `class SmartPointer p where`
/// `  type Of :: Type -> Type`
/// `  new :: a -> p a`
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let rc_ptr = smart_pointer_new::<RcBrand, _>(42);
/// let arc_ptr = smart_pointer_new::<ArcBrand, _>(42);
/// ```
pub trait SmartPointer {
    /// The smart pointer type constructor.
    /// For `RcBrand`, this is `Rc<T>`. For `ArcBrand`, this is `Arc<T>`.
    type Of<T: ?Sized>: Clone + Deref<Target = T>;

    /// Wraps a sized value in the smart pointer.
    ///
    /// ### Type Signature
    ///
    /// `forall a. SmartPointer p => a -> p a`
    fn new<T>(value: T) -> Self::Of<T>
    where
        Self::Of<T>: Sized;
}

/// Extension trait for thread-safe smart pointers.
///
/// This follows the same pattern as `SendClonableFn` extends `ClonableFn`.
/// Only implemented by brands whose `Of` type is `Send + Sync` when the
/// inner type is `Send + Sync` (i.e., `ArcBrand` but not `RcBrand`).
///
/// ### Design Rationale
///
/// This mirrors the existing `SendClonableFn` pattern in the library:
/// - Base trait (`SmartPointer`) works for all smart pointers
/// - Extension trait (`SendSmartPointer`) adds thread-safety guarantees
/// - Only thread-safe brands implement the extension trait
pub trait SendSmartPointer: SmartPointer
where
    // This bound ensures Of<T> is Send+Sync when T is Send+Sync
    for<T: Send + Sync> Self::Of<T>: Send + Sync,
{
}
```

#### Brand Definitions

```rust
// fp-library/src/brands.rs

/// Brand for `std::rc::Rc` smart pointer.
/// Use this for single-threaded code where cloning is cheap.
/// Does NOT implement `SendSmartPointer`.
pub struct RcBrand;

/// Brand for `std::sync::Arc` smart pointer.
/// Use this for multi-threaded code requiring `Send + Sync`.
/// Implements both `SmartPointer` and `SendSmartPointer`.
pub struct ArcBrand;

/// Generic function brand parameterized by smart pointer choice.
/// This replaces the separate `RcFnBrand` and `ArcFnBrand` types.
pub struct FnBrand<PtrBrand: SmartPointer>(PhantomData<PtrBrand>);

/// Type alias for Rc-based function wrapper (convenience).
pub type RcFnBrand = FnBrand<RcBrand>;

/// Type alias for Arc-based function wrapper (convenience).
pub type ArcFnBrand = FnBrand<ArcBrand>;
```

#### SmartPointer Implementations

```rust
// fp-library/src/types/rc_ptr.rs

use crate::{brands::RcBrand, classes::smart_pointer::SmartPointer};
use std::rc::Rc;

impl SmartPointer for RcBrand {
    type Of<T: ?Sized> = Rc<T>;

    fn new<T>(value: T) -> Rc<T> {
        Rc::new(value)
    }
}

// Note: RcBrand does NOT implement SendSmartPointer
```

```rust
// fp-library/src/types/arc_ptr.rs

use crate::{
    brands::ArcBrand,
    classes::smart_pointer::{SendSmartPointer, SmartPointer},
};
use std::sync::Arc;

impl SmartPointer for ArcBrand {
    type Of<T: ?Sized> = Arc<T>;

    fn new<T>(value: T) -> Arc<T> {
        Arc::new(value)
    }
}

// ArcBrand implements SendSmartPointer because Arc<T: Send+Sync> is Send+Sync
impl SendSmartPointer for ArcBrand {}
```

### Part 2: Refactored ClonableFn Using SmartPointer

This is a **required** part of the design, not optional. `ClonableFn` will be refactored to use `SmartPointer` as its foundation.

#### The Unsized Coercion Problem

**Problem**: `SmartPointer::new` accepts `T` (sized), but `ClonableFn` needs to create `Of<dyn Fn(A) -> B>` (unsized).

**Why this happens**: When you write `Rc::new(closure)`, Rust performs implicit unsized coercion because it knows the target type. But `SmartPointer::new` is generic and can't know the target type.

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
        smart_pointer::{SendSmartPointer, SmartPointer},
    },
};
use std::{rc::Rc, sync::Arc};

/// Macro to implement ClonableFn for FnBrand<PtrBrand>.
/// This handles the unsized coercion which can't be done generically.
macro_rules! impl_fn_brand {
    ($ptr_brand:ty, $ptr_type:ident) => {
        impl Function for FnBrand<$ptr_brand> {
            type Of<'a, A, B> = $ptr_type<dyn 'a + Fn(A) -> B>;

            fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
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
impl SendClonableFn for FnBrand<ArcBrand> {
    type SendOf<'a, A, B> = Arc<dyn 'a + Fn(A) -> B + Send + Sync>;

    fn send_clonable_fn_new<'a, A, B>(
        f: impl 'a + Fn(A) -> B + Send + Sync
    ) -> Self::SendOf<'a, A, B> {
        Arc::new(f)
    }
}

// Note: FnBrand<RcBrand> does NOT implement SendClonableFn
```

#### Alternative: Specialization-Based Approach (Nightly Only)

If using nightly Rust, specialization could provide a cleaner solution:

```rust
#![feature(specialization)]

impl<PtrBrand: SmartPointer> ClonableFn for FnBrand<PtrBrand> {
    type Of<'a, A, B> = <PtrBrand as SmartPointer>::Of<dyn 'a + Fn(A) -> B>;

    default fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
        unimplemented!("Specialized implementation required")
    }
}

impl ClonableFn for FnBrand<RcBrand> {
    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
        Rc::new(f)
    }
}

impl ClonableFn for FnBrand<ArcBrand> {
    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
        Arc::new(f)
    }
}
```

**Recommendation**: Use the macro approach for stable Rust compatibility.

### Part 3: Lazy Type with Shared Memoization

The new `Lazy` type replaces the current value-semantic implementation with Haskell-like shared memoization.

#### Core Structure

```rust
// fp-library/src/types/lazy.rs (replacement)

use crate::{
    brands::LazyBrand,
    classes::{
        clonable_fn::ClonableFn,
        defer::Defer,
        monoid::Monoid,
        once::Once,
        semigroup::Semigroup,
        smart_pointer::SmartPointer,
    },
    impl_kind,
    kinds::*,
};

/// Lazily-computed value with shared memoization (Haskell-like semantics).
///
/// Cloning a `Lazy` shares the memoization state via the underlying smart pointer.
/// When any clone is forced, all clones see the cached result.
///
/// ### Type Parameters
///
/// * `PtrBrand`: Smart pointer brand (`RcBrand` or `ArcBrand`)
/// * `OnceBrand`: Once cell brand (`OnceCellBrand` or `OnceLockBrand`)
/// * `A`: The type of the lazily-computed value
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
/// let lazy = Lazy::<RcBrand, OnceCellBrand, _>::new(
///     clonable_fn_new::<RcFnBrand, _, _>(move |_| {
///         counter_clone.set(counter_clone.get() + 1);
///         42
///     })
/// );
///
/// let lazy2 = lazy.clone();  // Shares memoization state!
///
/// assert_eq!(counter.get(), 0);
/// assert_eq!(Lazy::force(&lazy), 42);
/// assert_eq!(counter.get(), 1);  // Computed once
/// assert_eq!(Lazy::force(&lazy2), 42);
/// assert_eq!(counter.get(), 1);  // NOT recomputed - shared!
/// ```
pub struct Lazy<'a, PtrBrand, OnceBrand, A>(
    <PtrBrand as SmartPointer>::Of<(
        <OnceBrand as Once>::Of<A>,
        <FnBrand<PtrBrand> as ClonableFn>::Of<'a, (), A>,
    )>,
)
where
    PtrBrand: SmartPointer,
    OnceBrand: Once;
```

**Key design change**: The `Lazy` type now uses `FnBrand<PtrBrand>` internally, so the smart pointer brand determines both the sharing semantics AND the function wrapper type. This reduces the number of type parameters from 4 to 3.

#### Implementation

```rust
impl<'a, PtrBrand, OnceBrand, A> Lazy<'a, PtrBrand, OnceBrand, A>
where
    PtrBrand: SmartPointer,
    OnceBrand: Once,
{
    /// Creates a new `Lazy` value from a thunk.
    pub fn new(thunk: <FnBrand<PtrBrand> as ClonableFn>::Of<'a, (), A>) -> Self {
        Self(<PtrBrand as SmartPointer>::new((OnceBrand::new(), thunk)))
    }

    /// Forces the evaluation and returns the value.
    ///
    /// Takes `&self` because all clones share the same memoization state.
    pub fn force(this: &Self) -> A
    where
        A: Clone,
    {
        let (once_cell, thunk) = &*this.0;
        <OnceBrand as Once>::get_or_init(once_cell, || thunk(())).clone()
    }
}

impl<'a, PtrBrand, OnceBrand, A> Clone for Lazy<'a, PtrBrand, OnceBrand, A>
where
    PtrBrand: SmartPointer,
    OnceBrand: Once,
{
    fn clone(&self) -> Self {
        // Cheap Rc/Arc clone - shares memoization state
        Self(self.0.clone())
    }
}
```

#### Convenience Type Aliases

```rust
// fp-library/src/types/lazy.rs (continued)

/// Single-threaded lazy value using Rc.
/// Not Send or Sync.
pub type RcLazy<'a, A> = Lazy<'a, RcBrand, OnceCellBrand, A>;

/// Thread-safe lazy value using Arc.
/// Send and Sync when A is Send and Sync.
---

## Challenges & Solutions

### Challenge 1: Unsized Coercion in ClonableFn

**Problem**: `SmartPointer::new` accepts `T` (sized), but `ClonableFn` needs to create `Of<dyn Fn(A) -> B>` (unsized).

**Why this happens**: When you write `Rc::new(closure)`, Rust performs implicit unsized coercion because it knows the target type. But `SmartPointer::new` is generic and can't know the target type.

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
- **nightly `CoerceUnsized`**: Would work but limits to nightly Rust
- **`new_unsized` method**: Can't pass unsized values by value
- **Specialization**: Also nightly-only

### Challenge 2: Thread Safety Bounds

**Problem**: `Arc<T>` is `Send + Sync` when `T: Send + Sync`. But `SmartPointer` is generic and can't enforce this at the trait level.

**Solution**: Use `SendSmartPointer` extension trait, following the same pattern as `SendClonableFn`:

```rust
/// Extension trait for thread-safe smart pointers.
/// Mirrors the SendClonableFn pattern.
pub trait SendSmartPointer: SmartPointer
where
    for<T: Send + Sync> Self::Of<T>: Send + Sync,
{
}

// Only ArcBrand implements this
impl SendSmartPointer for ArcBrand {}

// RcBrand does NOT implement SendSmartPointer
```

**Usage in constraints**:
```rust
// Require thread-safe smart pointer
fn parallel_operation<P: SendSmartPointer>(ptr: P::Of<Data>) { ... }
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
```

This maintains the same pattern: the extension trait is only implemented for thread-safe variants.

---

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
- Multiple clones exist
- Thunk is expensive
- Value is large (expensive to clone)

Old value semantics was only better for:
- Single-use lazy values (rare use case)

---

## Implementation Phases

### Phase 1: SmartPointer Foundation

1. Create `fp-library/src/classes/smart_pointer.rs`
   - Define `SmartPointer` trait
   - Define `SendSmartPointer` extension trait
   - Add free function `smart_pointer_new`
2. Add `RcBrand` and `ArcBrand` to `fp-library/src/brands.rs`
3. Create `fp-library/src/types/rc_ptr.rs` with `SmartPointer` impl for `RcBrand`
4. Create `fp-library/src/types/arc_ptr.rs` with `SmartPointer` and `SendSmartPointer` impls for `ArcBrand`
5. Update module re-exports

### Phase 2: FnBrand Refactor

1. Add `FnBrand<PtrBrand>` struct to `fp-library/src/brands.rs`
2. Add `RcFnBrand` and `ArcFnBrand` type aliases
3. Create `fp-library/src/types/fn_brand.rs`
   - Implement `Function`, `ClonableFn`, `Semigroupoid`, `Category` for `FnBrand<RcBrand>`
   - Implement same for `FnBrand<ArcBrand>`
   - Implement `SendClonableFn` for `FnBrand<ArcBrand>` only
   - Use macro to reduce duplication
4. Remove old `fp-library/src/types/rc_fn.rs` and `arc_fn.rs`
5. Update all code that referenced old brands

### Phase 3: Lazy Refactor

1. Rewrite `fp-library/src/types/lazy.rs`
   - Change to shared semantics using `SmartPointer`
   - Use `FnBrand<PtrBrand>` for thunk storage
   - Reduce type parameters from 4 to 3
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

---

## Files to Create

| File | Purpose |
|------|---------|
| `fp-library/src/classes/smart_pointer.rs` | `SmartPointer` and `SendSmartPointer` traits |
| `fp-library/src/types/rc_ptr.rs` | `SmartPointer` impl for `RcBrand` |
| `fp-library/src/types/arc_ptr.rs` | `SmartPointer` + `SendSmartPointer` impl for `ArcBrand` |
| `fp-library/src/types/fn_brand.rs` | `FnBrand<PtrBrand>` implementations |

## Files to Modify

| File | Changes |
|------|---------|
| `fp-library/src/brands.rs` | Add `RcBrand`, `ArcBrand`, `FnBrand<P>`, type aliases |
| `fp-library/src/classes.rs` | Re-export `smart_pointer` module |
| `fp-library/src/types.rs` | Re-export new modules, remove old |
| `fp-library/src/types/lazy.rs` | Complete rewrite with shared semantics |
| `fp-library/src/functions.rs` | Re-export new free functions |

## Files to Delete

| File | Reason |
|------|--------|
| `fp-library/src/types/rc_fn.rs` | Replaced by `fn_brand.rs` |
| `fp-library/src/types/arc_fn.rs` | Replaced by `fn_brand.rs` |

---

## Alternatives Considered

### Alternative 1: Separate SendSmartPointer with Own Associated Type

**Description**: Like `SendClonableFn`, have `SendSmartPointer` define its own `SendOf` type.

```rust
trait SendSmartPointer: SmartPointer {
    type SendOf<T: ?Sized + Send + Sync>: Clone + Send + Sync + Deref<Target = T>;
}
```

**Pros**: More explicit about thread-safe type
**Cons**: Duplication, harder to use generically
**Decision**: Rejected - marker trait is simpler and sufficient

### Alternative 2: No SmartPointer, Just Refactor ClonableFn

**Description**: Keep separate `RcFnBrand`/`ArcFnBrand` but use them directly in `Lazy`.

**Pros**: Less new abstraction
**Cons**: Doesn't solve the problem of sharing semantics; FnBrand can't wrap arbitrary types
**Decision**: Rejected - need `SmartPointer` for `Lazy` to wrap `(OnceCell, Thunk)`

### Alternative 3: Keep Both Lazy Implementations

**Description**: Have `ValueLazy` (current) and `SharedLazy` (new).

**Pros**: No breaking change, both options available
**Cons**: Confusing API, maintenance burden, value semantics rarely useful
**Decision**: Rejected - clean break is better since backward compat isn't a goal

---

## References

- [Haskell's Data.Lazy](https://hackage.haskell.org/package/lazy)
- [PureScript's Data.Lazy](https://pursuit.purescript.org/packages/purescript-lazy)
- [std::rc::Rc documentation](https://doc.rust-lang.org/std/rc/struct.Rc.html)
- [std::sync::Arc documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html)
- [Existing SendClonableFn trait](../fp-library/src/classes/send_clonable_fn.rs)
- [Existing ClonableFn trait](../fp-library/src/classes/clonable_fn.rs)
- [Current Lazy implementation](../fp-library/src/types/lazy.rs)
- [std::sync::Arc documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html)
- [Existing ClonableFn trait](../fp-library/src/classes/clonable_fn.rs)
- [Existing Lazy implementation](../fp-library/src/types/lazy.rs)
