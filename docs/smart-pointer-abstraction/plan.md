# Smart Pointer Abstraction & Shared Lazy Evaluation

## Summary

This document outlines a plan to introduce a unified `SmartPointer` type class abstraction that allows library types to be parameterized over the choice of reference-counted smart pointer (`Rc` vs `Arc`). This abstraction enables:

1. **Unified Rc/Arc selection** across multiple library types
2. **Shared memoization semantics** for `Lazy` (Haskell-like behavior)
3. **Reduced code duplication** by building multiple types on a single foundation
4. **Future extensibility** for custom allocators or alternative smart pointers

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
   - This pattern can be generalized

### Current Architecture Gap

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CURRENT: Ad-hoc Rc/Arc Abstraction                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ClonableFn                            Lazy (current)                      │
│       │                                     │                               │
│   ┌───┴───┐                          Uses OnceBrand (not shared)            │
│   │       │                                                                 │
│ RcFnBrand ArcFnBrand                 Clones create independent copies       │
│                                                                             │
│   Problem: No shared foundation for the Rc/Arc choice                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Proposed Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     PROPOSED: Unified SmartPointer Foundation               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                          SmartPointer (new trait)                           │
│                                  │                                          │
│                          ┌───────┴───────┐                                  │
│                          │               │                                  │
│                       RcBrand         ArcBrand                              │
│                          │               │                                  │
│            ┌─────────────┴───────────────┴─────────────┐                    │
│            │                                           │                    │
│            ▼                                           ▼                    │
│   ┌────────────────────┐                   ┌────────────────────┐           │
│   │  ClonableFn (via   │                   │ SharedLazy (uses   │           │
│   │  GenericFnBrand)   │                   │ SmartPointer for   │           │
│   │                    │                   │ shared OnceCell)   │           │
│   └────────────────────┘                   └────────────────────┘           │
│                                                                             │
│   Benefits: Unified Rc/Arc choice, shared foundation, less duplication      │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Design Goals

### Primary Goals

1. **Introduce `SmartPointer` trait** as a foundational abstraction for `Rc` and `Arc`
2. **Create `SharedLazy`** type with Haskell-like shared memoization semantics
3. **Maintain backward compatibility** with existing `Lazy`, `RcFnBrand`, `ArcFnBrand`
4. **Enable composition** - users pick one smart pointer brand and use it consistently

### Secondary Goals

1. **Potentially refactor `ClonableFn`** to use `SmartPointer` internally (optional, for consistency)
2. **Support thread-safe variants** via `SendSmartPointer` marker trait
3. **Provide clear migration path** for existing code
4. **Document trade-offs** between value semantics (`Lazy`) and shared semantics (`SharedLazy`)

### Non-Goals

1. **Replacing the existing `Lazy` type** - both semantics have valid use cases
2. **Supporting non-reference-counted smart pointers** initially (e.g., `Box` - though could be added later)
3. **Automatic selection** of Rc vs Arc based on context (user explicitly chooses)

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

/// Marker trait for thread-safe smart pointers.
///
/// Only implemented by brands whose `Of` type is `Send + Sync`
/// when the inner type is `Send + Sync`.
///
/// This enables compile-time enforcement of thread safety requirements.
pub trait SendSmartPointer: SmartPointer
where
    // This bound ensures Of<T> is Send+Sync when T is Send+Sync
    for<'a, T: Send + Sync + 'a> Self::Of<T>: Send + Sync,
{
}
```

#### Brand Definitions

```rust
// fp-library/src/brands.rs (additions)

/// Brand for `std::rc::Rc` smart pointer.
/// Use this for single-threaded code where cloning is cheap.
pub struct RcBrand;

/// Brand for `std::sync::Arc` smart pointer.
/// Use this for multi-threaded code requiring `Send + Sync`.
pub struct ArcBrand;
```

#### Implementations

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

impl SendSmartPointer for ArcBrand {}
```

### Part 2: SharedLazy Type

#### Core Structure

```rust
// fp-library/src/types/shared_lazy.rs

use crate::{
    brands::SharedLazyBrand,
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
/// Unlike [`Lazy`], cloning a `SharedLazy` shares the memoization state.
/// When any clone is forced, all clones see the cached result.
///
/// ### Type Parameters
///
/// * `PtrBrand`: Smart pointer brand (`RcBrand` or `ArcBrand`)
/// * `OnceBrand`: Once cell brand (`OnceCellBrand` or `OnceLockBrand`)
/// * `FnBrand`: Clonable function brand (`RcFnBrand` or `ArcFnBrand`)
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
/// let lazy = SharedLazy::<RcBrand, OnceCellBrand, RcFnBrand, _>::new(
///     clonable_fn_new::<RcFnBrand, _, _>(move |_| {
///         counter_clone.set(counter_clone.get() + 1);
///         42
///     })
/// );
///
/// let lazy2 = lazy.clone();  // Shares memoization state!
///
/// assert_eq!(counter.get(), 0);
/// assert_eq!(SharedLazy::force(&lazy), 42);
/// assert_eq!(counter.get(), 1);  // Computed once
/// assert_eq!(SharedLazy::force(&lazy2), 42);
/// assert_eq!(counter.get(), 1);  // NOT recomputed - shared!
/// ```
pub struct SharedLazy<'a, PtrBrand, OnceBrand, FnBrand, A>(
    <PtrBrand as SmartPointer>::Of<(
        <OnceBrand as Once>::Of<A>,
        <FnBrand as ClonableFn>::Of<'a, (), A>,
    )>,
)
where
    PtrBrand: SmartPointer,
    OnceBrand: Once,
    FnBrand: ClonableFn;
```

#### Implementation

```rust
impl<'a, PtrBrand, OnceBrand, FnBrand, A> SharedLazy<'a, PtrBrand, OnceBrand, FnBrand, A>
where
    PtrBrand: SmartPointer,
    OnceBrand: Once,
    FnBrand: ClonableFn,
{
    /// Creates a new `SharedLazy` value from a thunk.
    ///
    /// ### Type Signature
    ///
    /// `forall ptr once fn a. (() -> a) -> SharedLazy ptr once fn a`
    pub fn new(thunk: <FnBrand as ClonableFn>::Of<'a, (), A>) -> Self {
        Self(<PtrBrand as SmartPointer>::new((OnceBrand::new(), thunk)))
    }

    /// Forces the evaluation and returns the value.
    ///
    /// If already computed, returns the cached result.
    /// All clones share the same cache, so forcing one forces all.
    ///
    /// ### Type Signature
    ///
    /// `forall ptr once fn a. SharedLazy ptr once fn a -> a`
    pub fn force(this: &Self) -> A
    where
        A: Clone,
    {
        // Dereference the smart pointer to access the shared (OnceCell, thunk) pair
        let (once_cell, thunk) = &*this.0;
        <OnceBrand as Once>::get_or_init(once_cell, || thunk(())).clone()
    }
}

impl<'a, PtrBrand, OnceBrand, FnBrand, A> Clone for SharedLazy<'a, PtrBrand, OnceBrand, FnBrand, A>
where
    PtrBrand: SmartPointer,
    OnceBrand: Once,
    FnBrand: ClonableFn,
    <PtrBrand as SmartPointer>::Of<(
        <OnceBrand as Once>::Of<A>,
        <FnBrand as ClonableFn>::Of<'a, (), A>,
    )>: Clone,
{
    fn clone(&self) -> Self {
        // Clones the Rc/Arc, sharing the underlying (OnceCell, thunk)
        Self(self.0.clone())
    }
}
```

#### Type Class Implementations

```rust
// Semigroup instance - lazy combination
impl<'a, PtrBrand, OnceBrand, FnBrand, A> Semigroup
    for SharedLazy<'a, PtrBrand, OnceBrand, FnBrand, A>
where
    PtrBrand: SmartPointer + 'a,
    OnceBrand: Once + 'a,
    FnBrand: ClonableFn + 'a,
    A: Semigroup + Clone + 'a,
    // ... additional bounds
{
    fn append(a: Self, b: Self) -> Self {
        SharedLazy::new(<FnBrand as ClonableFn>::new(move |_| {
            Semigroup::append(SharedLazy::force(&a), SharedLazy::force(&b))
        }))
    }
}

// Monoid instance
impl<'a, PtrBrand, OnceBrand, FnBrand, A> Monoid
    for SharedLazy<'a, PtrBrand, OnceBrand, FnBrand, A>
where
    // ... bounds
    A: Monoid + Clone + 'a,
{
    fn empty() -> Self {
        SharedLazy::new(<FnBrand as ClonableFn>::new(|_| Monoid::empty()))
    }
}

// Defer instance
impl<'a, PtrBrand, OnceBrand, FnBrand, A> Defer<'a>
    for SharedLazy<'a, PtrBrand, OnceBrand, FnBrand, A>
where
    // ... bounds
    A: Clone + 'a,
{
    fn defer<FnBrand_>(f: <FnBrand_ as ClonableFn>::Of<'a, (), Self>) -> Self
    where
        FnBrand_: ClonableFn + 'a,
    {
        SharedLazy::new(<FnBrand as ClonableFn>::new(move |_| {
            SharedLazy::force(&f(()))
        }))
    }
}
```

### Part 3: Optional - Refactoring ClonableFn

This section describes how `ClonableFn` could be refactored to use `SmartPointer`. **This is optional** and would be a separate effort.

#### Current Implementation

```rust
// Current: RcFnBrand and ArcFnBrand are separate brands
pub struct RcFnBrand;
pub struct ArcFnBrand;

impl ClonableFn for RcFnBrand {
    type Of<'a, A, B> = Rc<dyn 'a + Fn(A) -> B>;
    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
        Rc::new(f)
    }
}

impl ClonableFn for ArcFnBrand {
    type Of<'a, A, B> = Arc<dyn 'a + Fn(A) -> B>;
    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
        Arc::new(f)
    }
}
```

#### Potential Refactored Implementation

```rust
// Refactored: GenericFnBrand parameterized by SmartPointer
pub struct GenericFnBrand<PtrBrand: SmartPointer>(PhantomData<PtrBrand>);

// Type aliases for backward compatibility
pub type RcFnBrand = GenericFnBrand<RcBrand>;
pub type ArcFnBrand = GenericFnBrand<ArcBrand>;

impl<PtrBrand: SmartPointer> ClonableFn for GenericFnBrand<PtrBrand> {
    type Of<'a, A, B> = <PtrBrand as SmartPointer>::Of<dyn 'a + Fn(A) -> B>;

    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
        // Challenge: SmartPointer::new expects a sized type,
        // but we need to coerce to unsized `dyn Fn(A) -> B`
        // See "Challenges" section for solutions
        todo!()
    }
}
```

---

## Challenges & Solutions

### Challenge 1: Unsized Coercion

**Problem**: `SmartPointer::new` accepts `T` (sized), but we need to create `Of<dyn Trait>` (unsized).

**Current workaround**: `Rc::new(f)` and `Arc::new(f)` perform implicit coercion from sized closure to `dyn Fn`. This coercion happens because Rust knows the concrete type being wrapped.

**Solutions**:

1. **Add `new_unsized` method** (if stabilized):
   ```rust
   trait SmartPointer {
       fn new_unsized<T: ?Sized>(value: /* ??? */) -> Self::Of<T>;
   }
   ```
   This is problematic because you can't pass an unsized value by value.

2. **Use CoerceUnsized** (nightly only):
   ```rust
   fn new_coerce<T, U: ?Sized>(value: T) -> Self::Of<U>
   where
       Self::Of<T>: CoerceUnsized<Self::Of<U>>;
   ```

3. **Separate trait for function wrapping** (recommended):
   ```rust
   // Keep SmartPointer for general use
   trait SmartPointer { ... }

   // Add specialized method to ClonableFn brands that uses SmartPointer internally
   impl<PtrBrand: SmartPointer> Function for GenericFnBrand<PtrBrand> {
       fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
           // Implementation specific to each PtrBrand variant
           // using macro or specialization
       }
   }
   ```

4. **Accept current duplication** (simplest):
   - Keep `RcFnBrand` and `ArcFnBrand` as separate implementations
   - Use `SmartPointer` only for new types like `SharedLazy`
   - The abstraction is still valuable for SharedLazy without changing ClonableFn

### Challenge 2: Thread Safety Bounds

**Problem**: `Arc<T>` requires `T: Send + Sync` for the `Arc` itself to be `Send + Sync`. But `SmartPointer` is generic and can't enforce this.

**Solution**: Use `SendSmartPointer` marker trait:

```rust
/// Marker for thread-safe smart pointers.
pub trait SendSmartPointer: SmartPointer
where
    for<'a, T: Send + Sync + 'a> Self::Of<T>: Send + Sync,
{
}

// Only Arc implements this
impl SendSmartPointer for ArcBrand {}

// Thread-safe SharedLazy
pub struct SendSharedLazy<PtrBrand: SendSmartPointer, ...>(...);
```

### Challenge 3: Interaction with Once Brands

**Problem**: `OnceCellBrand` uses `std::cell::OnceCell` (not `Send`). `OnceLockBrand` uses `std::sync::OnceLock` (`Send + Sync`).

**Solution**: Constrain valid combinations:

```rust
// Single-threaded: RcBrand + OnceCellBrand + RcFnBrand
type RcLazy<A> = SharedLazy<'static, RcBrand, OnceCellBrand, RcFnBrand, A>;

// Multi-threaded: ArcBrand + OnceLockBrand + ArcFnBrand
type ArcLazy<A> = SharedLazy<'static, ArcBrand, OnceLockBrand, ArcFnBrand, A>;

// Invalid combination prevented by bounds:
// SharedLazy<ArcBrand, OnceCellBrand, ...> would not be Send!
```

### Challenge 4: Maintaining Backward Compatibility

**Problem**: Adding `RcBrand`/`ArcBrand` might conflict with existing names or patterns.

**Solutions**:

1. **New names**: Use `RcPtrBrand`/`ArcPtrBrand` to avoid confusion with `RcFnBrand`/`ArcFnBrand`
2. **Feature flag**: Gate under `smart-pointer` feature initially
3. **Documentation**: Clear migration guide showing old vs new patterns

---

## Efficiency Analysis

### Value Semantics (`Lazy`) vs Shared Semantics (`SharedLazy`)

| Scenario | Lazy (Value) | SharedLazy (Shared) |
|----------|--------------|---------------------|
| Create | `OnceCell::new()` ~1ns | `Rc/Arc::new(...)` ~20ns |
| Clone (unforced) | `OnceCell` clone ~1ns | `Rc/Arc` clone ~3-5ns |
| Clone (forced) | Clone `OnceCell` + Clone `A` | `Rc/Arc` clone ~3-5ns |
| Force (first) | `thunk()` | `Rc/Arc` deref + `thunk()` |
| Force (nth clone) | `thunk()` again! | Return cached (shared) |

### When to Use Each

| Use Case | Recommended Type |
|----------|------------------|
| Single owner, force once | `Lazy` |
| Many clones, expensive thunk | `SharedLazy` |
| Recursive data structures | `SharedLazy` |
| Simple memoization | `Lazy` |
| Dynamic programming | `SharedLazy` |
| Multi-threaded access | `SharedLazy<ArcBrand, OnceLockBrand, ArcFnBrand, _>` |

---

## Implementation Phases

### Phase 1: Foundation (Required)

1. Add `SmartPointer` trait
2. Add `RcBrand` and `ArcBrand` brands
3. Implement `SmartPointer` for both brands
4. Add `SendSmartPointer` marker trait
5. Add free functions and re-exports

### Phase 2: SharedLazy (Required)

1. Create `SharedLazy` type
2. Implement `Clone`, `force`, `new`
3. Implement `Semigroup`, `Monoid`, `Defer`
4. Add `SharedLazyBrand` and `impl_kind!`
5. Add comprehensive tests

### Phase 3: Integration & Polish (Required)

1. Add type aliases for common configurations
2. Write documentation with examples
3. Add doc tests
4. Update `docs/std-coverage-checklist.md`
5. Update `docs/architecture.md`

### Phase 4: ClonableFn Refactor (Optional)

1. Investigate unsized coercion options
2. If viable, create `GenericFnBrand<PtrBrand>`
3. Type alias `RcFnBrand = GenericFnBrand<RcBrand>`
4. Type alias `ArcFnBrand = GenericFnBrand<ArcBrand>`
5. Ensure all existing tests pass

---

## Alternatives Considered

### Alternative 1: Separate SharedLazy Module

**Description**: Don't abstract Rc/Arc; just create a hardcoded `SharedLazy` using `Rc`.

**Pros**: Simpler, fewer type parameters
**Cons**: No Arc variant, duplicates code if we later need thread-safe version
**Decision**: Rejected - the abstraction is worth the complexity

### Alternative 2: Generic Over Any Clone + Deref Type

**Description**: Make SharedLazy generic over any type satisfying `Clone + Deref<Target = (OnceCell, Thunk)>`.

**Pros**: Maximum flexibility
**Cons**: Loses brand-based selection pattern, harder to use
**Decision**: Rejected - doesn't fit library's design patterns

### Alternative 3: Use once_cell Crate's Lazy

**Description**: Just wrap `once_cell::sync::Lazy` or use std's `LazyLock`.

**Pros**: Zero implementation effort
**Cons**: Can't integrate with library's brand system, no FnBrand flexibility
**Decision**: Rejected - need integration with existing type classes

---

## References

- [Haskell's Data.Lazy](https://hackage.haskell.org/package/lazy)
- [PureScript's Data.Lazy](https://pursuit.purescript.org/packages/purescript-lazy)
- [std::rc::Rc documentation](https://doc.rust-lang.org/std/rc/struct.Rc.html)
- [std::sync::Arc documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html)
- [Existing ClonableFn trait](../fp-library/src/classes/clonable_fn.rs)
- [Existing Lazy implementation](../fp-library/src/types/lazy.rs)
