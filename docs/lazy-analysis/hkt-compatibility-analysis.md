# Lazy Type Redesigns: HKT and Typeclass Compatibility Analysis

This document provides a comprehensive analysis of the compatibility between the proposed `Lazy` type redesigns (documented in [`architectural-critique.md`](./architectural-critique.md)) and the library's Higher-Kinded Type (HKT) system and typeclass hierarchies (defined in [`fp-library/src/lib.rs`](../../fp-library/src/lib.rs) and related modules).

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Background: The Library's HKT System](#background-the-librarys-hkt-system)
   - [Kind Traits](#kind-traits)
   - [Brand Pattern](#brand-pattern)
   - [Typeclass Hierarchy](#typeclass-hierarchy)
3. [Background: The Proposed Redesigns](#background-the-proposed-redesigns)
4. [Detailed Incompatibility Analysis](#detailed-incompatibility-analysis)
   - [Issue 1: FnOnce vs Fn Semantics](#issue-1-fnonce-vs-fn-semantics)
   - [Issue 2: Reference vs Value Ownership](#issue-2-reference-vs-value-ownership)
   - [Issue 3: Clone Requirements](#issue-3-clone-requirements)
   - [Issue 4: Brand Integration](#issue-4-brand-integration)
5. [Interconnection of Incompatibilities](#interconnection-of-incompatibilities)
6. [Benefits of Hypothetical Integration](#benefits-of-hypothetical-integration)
7. [Compatibility Proposals](#compatibility-proposals)
   - [Proposal 1: Adapt Redesigns to Library Constraints](#proposal-1-adapt-redesigns-to-library-constraints)
   - [Proposal 2: Parallel Typeclass Hierarchies](#proposal-2-parallel-typeclass-hierarchies)
   - [Proposal 3: Conditional Implementation Strategy](#proposal-3-conditional-implementation-strategy)
   - [Proposal 4: Newtype Bridge Pattern](#proposal-4-newtype-bridge-pattern)
8. [Trade-off Analysis Matrix](#trade-off-analysis-matrix)
9. [Recommendations](#recommendations)
10. [Drastic HKT System Changes for Better Integration](#drastic-hkt-system-changes-for-better-integration)
11. [Conclusion](#conclusion)

---

## Executive Summary

**The proposed `Lazy` type redesigns and the library's HKT/typeclass system are fundamentally incompatible** due to deep structural mismatches in:

| Dimension        | Library Requirement   | Redesign Approach      |
| ---------------- | --------------------- | ---------------------- |
| Function calling | `Fn` (multi-call)     | `FnOnce` (single-call) |
| Value access     | By value (`A`)        | By reference (`&A`)    |
| Clone bounds     | Required on functions | Explicitly avoided     |
| Type integration | Brand-based HKT       | Standalone types       |

These incompatibilities stem from legitimate but opposing design goals:

- **Library goal**: Maximize reusability through standard typeclass abstractions
- **Redesign goal**: Maximize ergonomics and safety by removing unnecessary constraints

This document explores four proposals for achieving compatibility, each with different trade-offs between these goals.

---

## Background: The Library's HKT System

### Kind Traits

The library simulates Higher-Kinded Types using "Kind" traits defined in [`fp-library/src/kinds.rs`](../../fp-library/src/kinds.rs). The relevant trait for container types like `Lazy` is:

```rust
// From kinds.rs:57-59
def_kind! {
    type Of<'a, A: 'a>: 'a;
}

// This expands to a trait like:
pub trait Kind_cdc7cd43dac7585f {
    type Of<'a, A: 'a>: 'a;
}
```

The hash `cdc7cd43dac7585f` is deterministically generated from the signature, ensuring consistent trait identity across the library.

### Brand Pattern

Types participate in the HKT system via "brand" types that implement Kind traits:

```rust
// From lazy.rs:885-889
impl_kind! {
    impl<Config: LazyConfig> for LazyBrand<Config> {
        type Of<'a, A: 'a>: 'a = Lazy<'a, Config, A>;
    }
}
```

This allows code to be generic over type constructors:

```rust
fn example<Brand: Kind_cdc7cd43dac7585f>() {
    // Brand::Of<'a, A> could be:
    // - Option<A> (when Brand = OptionBrand)
    // - Result<A, E> (when Brand = ResultBrand<E>)
    // - Lazy<'a, Config, A> (when Brand = LazyBrand<Config>)
}
```

### Typeclass Hierarchy

The library defines a standard FP typeclass hierarchy. The key traits relevant to this analysis are:

#### Functor

```rust
// From functor.rs:25-63
pub trait Functor: Kind_cdc7cd43dac7585f {
    fn map<'a, B: 'a, A: 'a, F>(
        f: F,
        fa: Self::Of<'a, A>,
    ) -> Self::Of<'a, B>
    where
        F: Fn(A) -> B + 'a;  // ← Key: Fn, not FnOnce
}
```

**Critical observation**: The function `f` must implement `Fn`, meaning it can be called multiple times. This is standard in Haskell-like functional programming where functions are pure and can be applied arbitrarily.

#### Semiapplicative

```rust
// From semiapplicative.rs:26-68
pub trait Semiapplicative: Lift + Functor {
    fn apply<'a, FnBrand: 'a + CloneableFn, B: 'a, A: 'a + Clone>(
        ff: Self::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>,
        fa: Self::Of<'a, A>,
    ) -> Self::Of<'a, B>;
}
```

**Critical observations**:

1. Functions must be wrapped in `CloneableFn::Of` (e.g., `Rc<dyn Fn(A) -> B>`)
2. Input type `A` must be `Clone`

#### Semimonad

```rust
// From semimonad.rs:20-58
pub trait Semimonad: Kind_cdc7cd43dac7585f {
    fn bind<'a, B: 'a, A: 'a, F>(
        ma: Self::Of<'a, A>,
        f: F,
    ) -> Self::Of<'a, B>
    where
        F: Fn(A) -> Self::Of<'a, B> + 'a;  // ← Key: Fn, not FnOnce
}
```

#### CloneableFn

```rust
// From cloneable_fn.rs:25-57
pub trait CloneableFn: Function {
    type Of<'a, A, B>: Clone + Deref<Target = dyn 'a + Fn(A) -> B>;

    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B>;
}
```

**Critical observation**: `CloneableFn::Of` requires functions that are:

1. `Clone` (can be duplicated)
2. `Fn` (can be called multiple times)
3. Wrapped in a pointer (`Rc` or `Arc`)

---

## Background: The Proposed Redesigns

The [`architectural-critique.md`](./architectural-critique.md) document proposes four redesign approaches, all sharing common characteristics:

### Proposal A: Independent Types with FnOnce

```rust
pub struct RcLazy<A> {
    inner: Rc<LazyCell<A>>,
}

impl<A> RcLazy<A> {
    pub fn new<F: FnOnce() -> A + 'static>(thunk: F) -> Self { ... }

    pub fn map<B, F: FnOnce(&A) -> B + 'static>(self, f: F) -> RcLazy<B> { ... }
}
```

### Proposal B: Effect-Based Error Handling

```rust
pub struct Lazy<A> { ... }
pub struct TryLazy<A, E> { ... }

impl<A> Lazy<A> {
    pub fn new<F: FnOnce() -> A + 'static>(thunk: F) -> Self { ... }
    pub fn map<B, F: FnOnce(&A) -> B + 'static>(self, f: F) -> Lazy<B> { ... }
}
```

### Proposal C: Standard Library Foundation

```rust
pub struct FpLazy<A> {
    cell: LazyCell<A, Box<dyn FnOnce() -> A>>,
}

impl<A> FpLazy<A> {
    pub fn new<F: FnOnce() -> A + 'static>(f: F) -> Self { ... }
    pub fn map<B, F: FnOnce(&A) -> B + 'static>(self, f: F) -> FpLazy<B> { ... }
}
```

### Proposal D: Separated Computation and Memoization

```rust
pub struct Thunk<A> {
    compute: Box<dyn FnOnce() -> A>,
}

impl<A> Thunk<A> {
    pub fn new<F: FnOnce() -> A + 'static>(f: F) -> Self { ... }
    pub fn map<B, F: FnOnce(A) -> B + 'static>(self, f: F) -> Thunk<B> { ... }
}
```

### Common Characteristics

All proposals share:

1. **`FnOnce` thunks** - Closures that are called at most once
2. **Reference-based `map`** - Receives `&A`, not `A`
3. **No `Clone` requirements** on closures
4. **Standalone types** without brand-based HKT integration

---

## Detailed Incompatibility Analysis

### Issue 1: FnOnce vs Fn Semantics

#### The Conflict

The library's typeclasses uniformly require `Fn`:

```rust
// Functor::map
F: Fn(A) -> B + 'a

// Semimonad::bind
F: Fn(A) -> Self::Of<'a, B> + 'a
```

The redesigns use `FnOnce`:

```rust
// All proposals use this pattern
F: FnOnce(&A) -> B + 'static
```

#### Why This Matters

The `Fn` trait hierarchy in Rust represents different calling semantics:

| Trait    | Can be called  | Captures mutably | Moves captures |
| -------- | -------------- | ---------------- | -------------- |
| `FnOnce` | Once           | N/A              | Yes            |
| `FnMut`  | Multiple times | Yes              | No             |
| `Fn`     | Multiple times | No               | No             |

`FnOnce` is a supertrait of `FnMut` which is a supertrait of `Fn`:

```
FnOnce ⊇ FnMut ⊇ Fn
```

This means:

- Every `Fn` is also a `FnOnce`
- **Not every `FnOnce` is a `Fn`**

#### Concrete Example of the Problem

```rust
// This closure moves `expensive_resource` into itself
let expensive_resource = create_expensive_resource();
let thunk = move || expensive_resource.process();

// This thunk can ONLY implement FnOnce, not Fn
// Because calling it twice would require two `expensive_resource` values
```

For lazy evaluation, `FnOnce` is semantically correct: the thunk is evaluated exactly once. But the library's `Functor` requires the mapping function to be `Fn`.

#### Why the Library Uses `Fn`

The library's design follows Haskell's semantics where functions are pure and referentially transparent. In that model, calling a function multiple times with the same argument always produces the same result, so `Fn` is natural.

Additionally, many typeclass laws require multiple applications:

```rust
// Functor composition law:
// map(compose(f, g), fa) == map(f, map(g, fa))
//
// This requires calling `f` on the result of calling `g`,
// which works fine with Fn but not necessarily FnOnce if literals
// are involved in the composition.
```

### Issue 2: Reference vs Value Ownership

#### The Conflict

The library's `Functor::map` takes `A` by value:

```rust
fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Self::Of<'a, A>) -> Self::Of<'a, B>
where
    F: Fn(A) -> B + 'a;  // f takes A, not &A
```

The redesigns' `map` takes `&A`:

```rust
pub fn map<B, F: FnOnce(&A) -> B + 'static>(self, f: F) -> RcLazy<B>
```

#### Why Lazy Values Return References

A `Lazy<A>` wraps the value in shared ownership (via `Rc` or `Arc`) and memoizes it. When forced, it returns a reference to the cached value:

```rust
// From lazy.rs:693-704
pub fn force(this: &Self) -> Result<&A, LazyError> {
    // Returns &A, not A
    // Because the value is owned by the Lazy, not given away
}
```

This is the correct API for memoization: the value is computed once and shared by reference.

#### Why This Creates an Incompatibility

To implement `Functor::map` for `Lazy`, we would need:

```rust
impl Functor for LazyBrand {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Lazy<'a, A>) -> Lazy<'a, B>
    where
        F: Fn(A) -> B + 'a,  // Takes A by value
    {
        Lazy::new(move || {
            let a: &A = fa.force();  // We have &A...
            f(a)  // ...but f expects A!
            // ❌ Type error: expected A, found &A
        })
    }
}
```

The only way to bridge this is to require `A: Clone`:

```rust
fn map<'a, B: 'a, A: Clone + 'a, F>(f: F, fa: Lazy<'a, A>) -> Lazy<'a, B>
where
    F: Fn(A) -> B + 'a,
{
    Lazy::new(move || {
        let a: &A = fa.force();
        f(a.clone())  // Clone to get A from &A
    })
}
```

But the `Functor` trait doesn't have `A: Clone` in its signature, so this cannot be a general implementation.

### Issue 3: Clone Requirements

#### The Conflict in CloneableFn

The library's `Semiapplicative::apply` requires functions wrapped in `CloneableFn`:

```rust
// From semiapplicative.rs:65-68
fn apply<'a, FnBrand: 'a + CloneableFn, B: 'a, A: 'a + Clone>(
    ff: Self::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>,
    fa: Self::Of<'a, A>,
) -> Self::Of<'a, B>;
```

And `CloneableFn::Of` is defined as:

```rust
// From cloneable_fn.rs:26
type Of<'a, A, B>: Clone + Deref<Target = dyn 'a + Fn(A) -> B>;
```

This means:

1. The function wrapper must be `Clone`
2. The underlying function must be `Fn` (not just `FnOnce`)

#### How CloneableFn Works

```rust
// Implementation for RcFnBrand creates Rc<dyn Fn(A) -> B>
impl CloneableFn for RcFnBrand {
    type Of<'a, A, B> = Rc<dyn 'a + Fn(A) -> B>;

    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
        Rc::new(f)
    }
}
```

The key insight: `Rc<dyn Fn(A) -> B>` requires the closure to be `Fn`, not just `FnOnce`.

#### The Redesign's Explicit Goal

From the architectural critique's trade-off analysis:

> | `FnOnce` thunks | No `Clone` bound on closures | Thunks cannot be cloned (by design) |

The redesigns explicitly avoid `Clone` bounds on closures as a feature, not a bug. This directly conflicts with `CloneableFn`.

### Issue 4: Brand Integration

#### Current Integration

The current `Lazy` type integrates with the HKT system via:

```rust
// From lazy.rs:885-889
impl_kind! {
    impl<Config: LazyConfig> for LazyBrand<Config> {
        type Of<'a, A: 'a>: 'a = Lazy<'a, Config, A>;
    }
}
```

This allows `Lazy` to participate in generic functions:

```rust
fn use_any<'a, Brand: Functor, A: 'a>(fa: Brand::Of<'a, A>) -> Brand::Of<'a, String>
where
    A: ToString,
{
    Brand::map(|a: A| a.to_string(), fa)
}

// Can call with:
// - use_any::<OptionBrand, _>(Some(42))
// - use_any::<VecBrand, _>(vec![1, 2, 3])
// - (If Functor were implemented) use_any::<LazyBrand<RcLazyConfig>, _>(lazy)
```

#### Redesign's Standalone Approach

The redesigns propose types without brand integration:

```rust
pub struct RcLazy<A> {
    inner: Rc<LazyCell<A>>,
}
// No LazyBrand
// No impl_kind!
// No Kind_cdc7cd43dac7585f
```

This means:

1. `RcLazy` cannot be used with generic functions expecting a `Kind`
2. No participation in typeclass polymorphism
3. Simpler API but less composable

---

## Interconnection of Incompatibilities

The four issues are not independent; they form a causal web:

```
                    ┌─────────────────────────────────────────────────────┐
                    │        Design Goal: Ergonomic Lazy Values           │
                    │     (no Clone on closures, natural || syntax)       │
                    └────────────────────────┬────────────────────────────┘
                                             │
                                             ▼
                    ┌─────────────────────────────────────────────────────┐
                    │              Use FnOnce for Thunks                  │
                    │         (can move into closure, no Clone)           │
                    └──────┬─────────────────────────────┬────────────────┘
                           │                             │
                           ▼                             ▼
        ┌──────────────────────────────────┐   ┌──────────────────────────────────┐
        │   Cannot Implement CloneableFn   │   │   Cannot Implement Fn-Based      │
        │   (requires Clone + Fn)          │   │   Typeclasses (Functor, etc.)    │
        └──────────────────────────────────┘   └──────────────────────────────────┘
                           │                             │
                           │                             │
                           ▼                             ▼
        ┌──────────────────────────────────┐   ┌──────────────────────────────────┐
        │   Cannot Implement Apply         │   │   force() Returns &A, Not A      │
        │   (needs CloneableFn)            │   │   (shared memoization semantics) │
        └──────────────────────────────────┘   └──────────────────────────────────┘
                           │                             │
                           └──────────────┬──────────────┘
                                          ▼
                    ┌─────────────────────────────────────────────────────┐
                    │        Would Need A: Clone Everywhere               │
                    │        (defeats purpose of ergonomic API)           │
                    └─────────────────────────────────────────────────────┘
```

**Key insight**: The root cause is the fundamental tension between:

1. **Lazy semantics** requiring `FnOnce` (compute once) and returning references (shared value)
2. **Library typeclasses** requiring `Fn` (callable many times) and taking values (ownership transfer)

---

## Benefits of Hypothetical Integration

If the redesigns could integrate with the library's typeclass system, the benefits would include:

### 1. Unified Vocabulary

Code using `Lazy` would use the same operations as other types:

```rust
// Instead of:
let lazy = lazy.map(|a| a + 1);        // Lazy-specific method

// Would use:
let lazy = map::<LazyBrand, _, _, _>(|a| a + 1, lazy);  // Generic function

// More importantly, makes Lazy work in generic contexts:
fn increment_all<Brand: Functor>(fa: Brand::Of<'_, i32>) -> Brand::Of<'_, i32> {
    map::<Brand, _, _, _>(|x| x + 1, fa)
}

increment_all::<OptionBrand>(Some(5));
increment_all::<VecBrand>(vec![1, 2, 3]);
increment_all::<LazyBrand<RcLazyConfig>>(lazy);  // Would work!
```

### 2. Typeclass-Based Generic Programming

Many utility functions are defined in terms of typeclasses:

```rust
// void: Replace contents with ()
fn void<Brand: Functor>(fa: Brand::Of<'_, A>) -> Brand::Of<'_, ()> {
    map::<Brand, _, _, _>(|_| (), fa)
}

// as_unit: Alias for void
fn as_unit<Brand: Functor>(fa: Brand::Of<'_, A>) -> Brand::Of<'_, ()> {
    void(fa)
}

// <$ operator: Replace contents
fn replace<Brand: Functor, A, B>(a: A, fb: Brand::Of<'_, B>) -> Brand::Of<'_, A>
where
    A: Clone,
{
    map::<Brand, _, _, _>(|_| a.clone(), fb)
}
```

Without `Functor`, `Lazy` cannot use any of these.

### 3. Traversable Integration

`Lazy` could participate in traversals:

```rust
// Traverse a Vec of lazy values, producing a lazy Vec
let lazy_values: Vec<Lazy<i32>> = vec![lazy1, lazy2, lazy3];
let lazy_vec: Lazy<Vec<i32>> = traverse(lazy_values);  // Would require Applicative
```

### 4. Monad Composition

With `Monad`, `Lazy` could use `do`-notation-style sequencing:

```rust
// Sequencing lazy computations
let result = bind::<LazyBrand, _, _, _>(lazy1, |x| {
    bind::<LazyBrand, _, _, _>(lazy2, |y| {
        pure::<LazyBrand, _>(x + y)
    })
});
```

### 5. Law-Based Reasoning

Typeclass laws guarantee predictable behavior:

```rust
// Functor identity law: map(id, fa) ≡ fa
assert_eq!(map(identity, lazy), lazy);

// Functor composition law: map(f . g, fa) ≡ map(f, map(g, fa))
assert_eq!(
    map(compose(f, g), lazy),
    map(f, map(g, lazy))
);

// Monad left identity: pure(a) >>= f ≡ f(a)
assert_eq!(
    bind(pure(a), f),
    f(a)
);
```

---

## Compatibility Proposals

### Proposal 1: Adapt Redesigns to Library Constraints

**Philosophy**: Modify the redesigns to use `Fn` and require `Clone` where needed, sacrificing ergonomics for typeclass compatibility.

#### Implementation

```rust
use std::cell::OnceCell;
use std::rc::Rc;

/// A lazy value compatible with library typeclasses.
/// Requires Clone bounds and uses Fn semantics.
pub struct CompatLazy<A> {
    inner: Rc<CompatLazyInner<A>>,
}

struct CompatLazyInner<A> {
    cell: OnceCell<A>,
    // Thunk must be Fn + Clone to satisfy CloneableFn
    thunk: Rc<dyn Fn(()) -> A>,
}

impl<A> CompatLazy<A> {
    /// Creates a new lazy value.
    ///
    /// Note: The closure must be `Fn`, not just `FnOnce`.
    /// This is required for typeclass compatibility.
    pub fn new<F: Fn(()) -> A + 'static>(thunk: F) -> Self
    where
        F: Clone,  // Required for CloneableFn
    {
        Self {
            inner: Rc::new(CompatLazyInner {
                cell: OnceCell::new(),
                thunk: Rc::new(thunk),
            }),
        }
    }

    /// Forces evaluation.
    pub fn force(&self) -> &A {
        self.inner.cell.get_or_init(|| (self.inner.thunk)(()))
    }
}

// Now we can implement Functor!
impl_kind! {
    for CompatLazyBrand {
        type Of<'a, A: 'a>: 'a = CompatLazy<A>;
    }
}

impl Functor for CompatLazyBrand {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: CompatLazy<A>) -> CompatLazy<B>
    where
        F: Fn(A) -> B + 'a,
        A: Clone,  // ← Must add this bound!
    {
        // Problem: Functor doesn't have A: Clone in its definition!
        // We cannot implement this in a way that satisfies the trait.
        unimplemented!("Cannot implement without A: Clone in trait definition")
    }
}
```

#### Analysis

**The fundamental problem**: The `Functor` trait's signature is fixed:

```rust
fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Self::Of<'a, A>) -> Self::Of<'a, B>
where
    F: Fn(A) -> B + 'a;
// No A: Clone here!
```

We cannot add `A: Clone` in our implementation because it would violate the trait contract.

#### Workaround: Separate Generic Impl

```rust
// We can implement a SEPARATE map that requires Clone
impl<A: Clone> CompatLazy<A> {
    pub fn map_cloning<B, F: Fn(A) -> B + 'static>(self, f: F) -> CompatLazy<B> {
        CompatLazy::new(move |()| f(self.force().clone()))
    }
}

// But this is NOT Functor::map!
```

#### Trade-off Summary

| Aspect                    | Assessment                            |
| ------------------------- | ------------------------------------- |
| Functor compatibility     | ❌ Impossible without changing Functor |
| Applicative compatibility | ❌ Same issue                          |
| Monad compatibility       | ❌ Same issue                          |
| Ergonomics                | ❌ Requires `Fn(()) -> A` syntax       |
| Clone on closures         | ❌ Still required                      |
| Clone on A                | ❌ Required for map                    |

**Verdict**: This approach cannot achieve typeclass compatibility without modifying the library's core traits.

---

### Proposal 2: Parallel Typeclass Hierarchies

**Philosophy**: Create a separate hierarchy of typeclasses designed for reference-based, once-semantics types.

#### Implementation

```rust
// ============================================================================
// New Kind Trait for Reference-Based Access
// ============================================================================

/// Kind for types where contained values are accessed by reference.
def_kind! {
    type OfRef<'a, A: 'a>: 'a;  // Different signature = different hash
}

// ============================================================================
// RefFunctor: Functor for Reference-Based Types
// ============================================================================

/// A functor where the mapping function receives a reference.
///
/// This is appropriate for:
/// - Lazy values (memoized, shared by reference)
/// - Persistent data structures (values are borrowed)
/// - Any type where extracting the value is expensive
pub trait RefFunctor: Kind_XXXXXXXX /* hash for OfRef */ {
    /// Maps a function over the contained value.
    ///
    /// Unlike `Functor::map`, the function receives `&A` rather than `A`.
    /// This avoids requiring `A: Clone` for types that store values internally.
    fn map_ref<'a, B: 'a, A: 'a, F>(
        f: F,
        fa: Self::OfRef<'a, A>,
    ) -> Self::OfRef<'a, B>
    where
        F: FnOnce(&A) -> B + 'a;  // FnOnce is sufficient for lazy
}

/// Free function for RefFunctor::map_ref
pub fn map_ref<'a, Brand: RefFunctor, B: 'a, A: 'a, F>(
    f: F,
    fa: Brand::OfRef<'a, A>,
) -> Brand::OfRef<'a, B>
where
    F: FnOnce(&A) -> B + 'a,
{
    Brand::map_ref(f, fa)
}

// ============================================================================
// OnceSemimonad: Bind for Once-Callable Functions
// ============================================================================

/// A semimonad where the continuation is called at most once.
///
/// This is appropriate for:
/// - Lazy values (thunk runs once)
/// - IO operations (execute once for effects)
/// - Linear resources (cannot duplicate)
pub trait OnceSemimonad: Kind_XXXXXXXX {
    /// Sequences computations where the second may consume resources.
    fn bind_once<'a, B: 'a, A: 'a, F>(
        ma: Self::OfRef<'a, A>,
        f: F,
    ) -> Self::OfRef<'a, B>
    where
        F: FnOnce(&A) -> Self::OfRef<'a, B> + 'a;
}

// ============================================================================
// OnceApplicative: Apply for Once-Callable Functions
// ============================================================================

/// Applicative without CloneableFn requirement.
pub trait OnceApplicative: RefFunctor {
    /// Lifts a value into the applicative context.
    fn pure_once<'a, A: 'a>(a: A) -> Self::OfRef<'a, A>;

    /// Applies a function in context to a value in context.
    ///
    /// Note: The function is used once, so no Clone requirement.
    fn apply_once<'a, B: 'a, A: 'a, F: 'a>(
        ff: Self::OfRef<'a, F>,
        fa: Self::OfRef<'a, A>,
    ) -> Self::OfRef<'a, B>
    where
        F: FnOnce(&A) -> B;  // Function inside the context
}
```

#### Implementing for Lazy

```rust
pub struct RcLazy<A> {
    inner: Rc<LazyCell<A>>,
}

struct LazyCell<A> {
    state: OnceCell<A>,
    thunk: UnsafeCell<Option<Box<dyn FnOnce() -> A>>>,
}

// Brand type
pub struct RcLazyBrand;

impl_kind! {
    for RcLazyBrand {
        type OfRef<'a, A: 'a>: 'a = RcLazy<A>;
    }
}

impl RefFunctor for RcLazyBrand {
    fn map_ref<'a, B: 'a, A: 'a, F>(f: F, fa: RcLazy<A>) -> RcLazy<B>
    where
        F: FnOnce(&A) -> B + 'a,
    {
        RcLazy::new(move || f(fa.force()))
    }
}

impl OnceSemimonad for RcLazyBrand {
    fn bind_once<'a, B: 'a, A: 'a, F>(
        ma: RcLazy<A>,
        f: F,
    ) -> RcLazy<B>
    where
        F: FnOnce(&A) -> RcLazy<B> + 'a,
    {
        RcLazy::new(move || {
            let inner = f(ma.force());
            inner.force().clone()  // Need Clone here for the final value
        })
    }
}
```

#### Generic Code Using Parallel Hierarchy

```rust
/// Works with any RefFunctor (RcLazy, other reference-based types)
fn transform<Brand: RefFunctor>(fa: Brand::OfRef<'_, i32>) -> Brand::OfRef<'_, String> {
    map_ref::<Brand, _, _, _>(|x: &i32| x.to_string(), fa)
}

// Usage:
let lazy: RcLazy<i32> = RcLazy::new(|| 42);
let transformed: RcLazy<String> = transform::<RcLazyBrand>(lazy);
```

#### Trade-off Summary

| Aspect               | Assessment                           |
| -------------------- | ------------------------------------ |
| Type safety          | ✅ Full type safety                   |
| Ergonomics           | ✅ FnOnce with references             |
| Existing code compat | ❌ Cannot use with Functor-based code |
| Library complexity   | ❌ Doubles the number of typeclasses  |
| Learning curve       | ❌ Two parallel hierarchies to learn  |
| Maintenance burden   | ❌ Duplicate implementations needed   |

**Verdict**: Technically sound but significantly increases library complexity.

---

### Proposal 3: Conditional Implementation Strategy

**Philosophy**: Implement standard typeclasses only when additional bounds are satisfied, accepting that the implementation is partial.

#### Implementation

```rust
pub struct RcLazy<A> { /* ... */ }

pub struct RcLazyBrand;

impl_kind! {
    for RcLazyBrand {
        type Of<'a, A: 'a>: 'a = RcLazy<A>;
    }
}

// ============================================================================
// Conditional Functor: Only when A: Clone
// ============================================================================

// We CANNOT implement Functor directly because the trait doesn't have A: Clone.
// But we can provide a "map" method on the type:

impl<A> RcLazy<A> {
    /// Maps a function over the lazy value.
    ///
    /// This is equivalent to `Functor::map` but requires `A: Clone`
    /// because we access the value by reference and must clone it.
    pub fn map<B, F>(self, f: F) -> RcLazy<B>
    where
        A: Clone + 'static,
        B: 'static,
        F: Fn(A) -> B + 'static,
    {
        RcLazy::new(move || f(self.force().clone()))
    }
}

// ============================================================================
// Provide Type-Constrained Wrappers
// ============================================================================

/// A lazy value with Clone bound baked in.
///
/// This newtype exists solely to provide typeclass instances
/// that require Clone on the inner type.
pub struct CloneableLazy<A: Clone>(pub RcLazy<A>);

// Now we could (hypothetically) implement Functor for CloneableLazyBrand
// if the library trait allowed bound propagation.
```

#### The Core Limitation

Even with wrappers, we cannot implement `Functor` because:

```rust
// The trait signature is FIXED:
trait Functor {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Self::Of<'a, A>) -> Self::Of<'a, B>
    where
        F: Fn(A) -> B + 'a;
}

// We CANNOT add: where A: Clone
// Implementations must satisfy the more general signature
```

#### What We Can Do

1. **Implement compatible typeclasses** (already done in current code):

   - `Semigroup` / `Monoid` - work fine
   - `Defer` / `SendDefer` - work fine

2. **Provide inherent methods** that mirror typeclass signatures:

```rust
impl<A> RcLazy<A> {
    // Not Functor::map, but similar API
    pub fn map<B, F>(self, f: F) -> RcLazy<B>
    where
        A: Clone + 'static,
        F: Fn(A) -> B + 'static,
    { ... }

    // Not Semimonad::bind, but similar API
    pub fn flat_map<B, F>(self, f: F) -> RcLazy<B>
    where
        A: Clone + 'static,
        B: Clone + 'static,
        F: Fn(A) -> RcLazy<B> + 'static,
    { ... }
}
```

3. **Document the limitation** clearly.

#### Trade-off Summary

| Aspect                   | Assessment                           |
| ------------------------ | ------------------------------------ |
| Honest about constraints | ✅ Clone requirements explicit        |
| Ergonomics               | ⚠️ Methods exist but not typeclass    |
| Generic programming      | ❌ Cannot use in Functor-generic code |
| Simplicity               | ✅ Single type, clear API             |
| Discoverability          | ❌ Methods not visible in trait impls |

**Verdict**: Practical approach that acknowledges limitations but provides usable API.

---

### Proposal 4: Newtype Bridge Pattern

**Philosophy**: Provide newtype wrappers that satisfy typeclass requirements through explicit transformation.

#### Implementation

````rust
use std::rc::Rc;

// ============================================================================
// Core Lazy Type (Ergonomic, FnOnce-based)
// ============================================================================

/// The primary lazy type with ergonomic API.
/// Does NOT implement Functor/Monad due to ownership semantics.
pub struct Lazy<A> {
    inner: Rc<LazyInner<A>>,
}

impl<A> Lazy<A> {
    pub fn new<F: FnOnce() -> A + 'static>(f: F) -> Self { /* ... */ }
    pub fn force(&self) -> &A { /* ... */ }

    /// Map with reference semantics (the natural operation for Lazy).
    pub fn map<B, F: FnOnce(&A) -> B + 'static>(self, f: F) -> Lazy<B> { /* ... */ }
}

// ============================================================================
// Bridge Newtype (Typeclass-Compatible)
// ============================================================================

/// A wrapper around Lazy that provides typeclass instances.
///
/// To use Lazy in generic typeclass code, wrap it in FunctorLazy:
/// ```
/// let lazy = Lazy::new(|| 42);
/// let wrapped = FunctorLazy::from_lazy(lazy);
/// let result = map::<FunctorLazyBrand, _, _, _>(|x| x + 1, wrapped);
/// ```
///
/// # Constraints
///
/// This wrapper requires:
/// - `A: Clone` (to convert `&A` from `force()` to `A` for `Functor::map`)
/// - Functions must be `Clone` (for storage in thunks)
pub struct FunctorLazy<A: Clone> {
    inner: Lazy<A>,
}

impl<A: Clone> FunctorLazy<A> {
    /// Wraps a Lazy value for typeclass-compatible operations.
    pub fn from_lazy(lazy: Lazy<A>) -> Self {
        Self { inner: lazy }
    }

    /// Unwraps back to a regular Lazy.
    pub fn into_lazy(self) -> Lazy<A> {
        self.inner
    }

    /// Access the underlying value.
    pub fn force(&self) -> &A {
        self.inner.force()
    }
}

impl<A: Clone> Clone for FunctorLazy<A> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

// Brand for HKT
pub struct FunctorLazyBrand;

impl_kind! {
    for FunctorLazyBrand {
        type Of<'a, A: 'a>: 'a = FunctorLazy<A>;  // ← Problem: needs A: Clone!
    }
}
````

#### The Fundamental Issue

Even with a newtype, we hit the same problem:

The `Kind` trait's associated type doesn't have bounds:

```rust
// From the def_kind! expansion
trait Kind_cdc7cd43dac7585f {
    type Of<'a, A: 'a>: 'a;  // No bounds on A beyond 'a
}
```

When we write:

```rust
impl_kind! {
    for FunctorLazyBrand {
        type Of<'a, A: 'a>: 'a = FunctorLazy<A>;
        // Error: the trait bound `A: Clone` is not satisfied
        // FunctorLazy<A> requires A: Clone, but trait allows any A: 'a
    }
}
```

The `impl_kind!` macro would generate code that doesn't compile because `FunctorLazy<A>` requires `A: Clone` but the trait doesn't provide that bound.

#### Workaround: Unsafe Wrapper

```rust
/// UNSAFE: This newtype provides typeclass instances by internally
/// requiring Clone, but exposing it without the bound at the type level.
///
/// The invariant is maintained by construction: you can only create
/// WrappedLazy<A> when A: Clone.
#[repr(transparent)]
pub struct WrappedLazy<A> {
    // Actually stores Lazy<A> where A: Clone, but we hide the bound
    inner: Lazy<A>,
    _marker: PhantomData<A>,
}

impl<A: Clone + 'static> WrappedLazy<A> {
    // Construction requires Clone
    pub fn new(lazy: Lazy<A>) -> Self {
        Self { inner: lazy, _marker: PhantomData }
    }
}

// Now we CAN implement the Kind trait:
impl_kind! {
    for WrappedLazyBrand {
        type Of<'a, A: 'a>: 'a = WrappedLazy<A>;  // Compiles!
    }
}

impl Functor for WrappedLazyBrand {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: WrappedLazy<A>) -> WrappedLazy<B>
    where
        F: Fn(A) -> B + 'a,
    {
        // PROBLEM: We need A: Clone to call force().clone()
        // But we don't have that bound here!
        //
        // We could use unsafe to "trust" that A: Clone because
        // the only way to construct WrappedLazy is via new() which requires Clone.
        // But this is unsound if someone constructs WrappedLazy differently.
        unimplemented!()
    }
}
```

This approach is fragile and potentially unsound.

#### Trade-off Summary

| Aspect              | Assessment                               |
| ------------------- | ---------------------------------------- |
| Type safety         | ❌ Requires unsafe or unsound             |
| Ergonomics          | ⚠️ Extra wrapper overhead                 |
| Maintainability     | ❌ Invariants not enforced by type system |
| Generic code compat | ⚠️ Only for Clone types                   |

**Verdict**: Technically possible but unsafe or unsound; not recommended.

---

## Trade-off Analysis Matrix

| Proposal                | Type Safety  | Ergonomics | Generic Code         | Complexity | Recommended |
| ----------------------- | ------------ | ---------- | -------------------- | ---------- | ----------- |
| 1. Adapt to Library     | ❌ Incomplete | ❌ Worse    | ❌ Impossible         | ⚠️ Medium   | No          |
| 2. Parallel Hierarchies | ✅ Sound      | ✅ Good     | ⚠️ Separate hierarchy | ❌ High     | Maybe       |
| 3. Conditional Methods  | ✅ Sound      | ⚠️ Partial  | ❌ No typeclass       | ✅ Low      | Yes         |
| 4. Newtype Bridge       | ❌ Fragile    | ⚠️ Overhead | ⚠️ Limited            | ⚠️ Medium   | No          |

### Detailed Scoring

**Type Safety** (0-3):

- 3: Fully type-safe, soundness guaranteed by compiler
- 2: Type-safe with runtime checks
- 1: Relies on unsafe code or invariants
- 0: Potentially unsound

**Ergonomics** (0-3):

- 3: Natural API, no extra syntax or wrappers
- 2: Minor inconveniences (extra bounds, wrappers)
- 1: Significant boilerplate
- 0: Significantly worse than current

**Generic Code** (0-3):

- 3: Full typeclass integration
- 2: Partial integration or separate hierarchy
- 1: Methods available but not typeclass-based
- 0: No generic code possible

**Complexity** (0-3, inverted: lower is better):

- 3: Significant new concepts or types
- 2: Moderate additions
- 1: Minor additions
- 0: No additional complexity

| Proposal | Type Safety | Ergonomics | Generic Code | Complexity | Total |
| -------- | ----------- | ---------- | ------------ | ---------- | ----- |
| 1        | 0           | 0          | 0            | 2          | 2/12  |
| 2        | 3           | 3          | 2            | 0          | 8/12  |
| 3        | 3           | 2          | 1            | 3          | 9/12  |
| 4        | 1           | 2          | 2            | 2          | 7/12  |

---

## Recommendations

Based on this analysis, I recommend a **hybrid approach** combining Proposal 2 and Proposal 3:

### Immediate Term: Proposal 3 (Conditional Methods)

1. **Implement the redesigned `Lazy` type** with `FnOnce` semantics and reference-based access.

2. **Keep compatible typeclass implementations**:

   - `Semigroup` for `Lazy<A>` where `A: Semigroup + Clone`
   - `Monoid` for `Lazy<A>` where `A: Monoid + Clone`
   - `Defer` for `RcLazy`
   - `SendDefer` for `ArcLazy`

3. **Provide ergonomic inherent methods**:

```rust
impl<A> Lazy<A> {
    /// Transforms the lazy value.
    ///
    /// Note: This is not `Functor::map` because the trait requires
    /// `Fn(A) -> B`, but lazy values naturally work with `FnOnce(&A) -> B`.
    pub fn map<B, F: FnOnce(&A) -> B + 'static>(self, f: F) -> Lazy<B>;

    /// Chains lazy computations.
    ///
    /// Note: This is not `Semimonad::bind` for similar reasons.
    pub fn flat_map<B, F>(self, f: F) -> Lazy<B>
    where
        B: Clone + 'static,
        F: FnOnce(&A) -> Lazy<B> + 'static;

    /// Combines two lazy values.
    pub fn map2<B, C, F>(a: Lazy<A>, b: Lazy<B>, f: F) -> Lazy<C>
    where
        F: FnOnce(&A, &B) -> C + 'static;
}
```

4. **Document the design decision** in the module:

```rust
//! # Lazy Type
//!
//! `Lazy<A>` provides ergonomic lazy evaluation with `FnOnce` semantics.
//!
//! ## Typeclass Compatibility
//!
//! `Lazy` implements:
//! - `Semigroup` / `Monoid` (when `A` implements these)
//! - `Defer` / `SendDefer` (for deferred construction)
//!
//! `Lazy` does NOT implement `Functor`, `Applicative`, or `Monad` because:
//! 1. These traits require `Fn(A) -> B`, but lazy semantics are better
//!    expressed with `FnOnce(&A) -> B` (compute once, access by reference).
//! 2. The traits take ownership of `A`, but `Lazy::force()` returns `&A`.
//!
//! For `Functor`/`Monad`-like operations, use the inherent methods:
//! - `lazy.map(|a| ...)` instead of `Functor::map`
//! - `lazy.flat_map(|a| ...)` instead of `Monad::bind`
```

### Longer Term: Consider Proposal 2 (Parallel Hierarchies)

If demand for generic lazy programming arises:

1. **Add reference-based typeclass hierarchy**:

   - `RefFunctor` for types where `map` receives `&A`
   - `OnceBind` for types where the continuation runs once
   - `OnceApplicative` without `CloneableFn` requirement

2. **Implement for Lazy and other reference-based types**:

   - `Lazy` implements `RefFunctor`, `OnceBind`, etc.
   - Potentially `Cow`-based types could also implement these

3. **Provide utility functions** for the new hierarchy:
   - `map_ref::<Brand, _, _, _>(|a| ..., fa)`
   - `bind_once::<Brand, _, _, _>(ma, |a| ...)`

### Why Not Proposal 1 or 4?

**Proposal 1** (Adapt to Library) fails because:

- Cannot implement `Functor` without `A: Clone` in the trait
- Would require changing core library traits (breaking change)
- Loses the ergonomic benefits that motivated the redesign

**Proposal 4** (Newtype Bridge) fails because:

- Relies on unsafe code or convention
- Does not provide typeclass implementation at the fundamental level
- Adds wrapper overhead without full benefit

---

## Drastic HKT System Changes for Better Integration

This section analyzes what fundamental changes to the Kind traits and HKT system would enable better Lazy integration, and their cascading effects on the entire library.

### Overview: The Core Problem

The fundamental issue is that the library's typeclass signatures are fixed:

```rust
// Functor (functor.rs:58-63)
fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Self::Of<'a, A>) -> Self::Of<'a, B>
where
    F: Fn(A) -> B + 'a;  // Takes A by VALUE, requires Fn (multi-callable)

// Semimonad (semimonad.rs:53-58)
fn bind<'a, B: 'a, A: 'a, F>(ma: Self::Of<'a, A>, f: F) -> Self::Of<'a, B>
where
    F: Fn(A) -> Self::Of<'a, B> + 'a;  // Same: A by value, Fn bound
```

Lazy types naturally provide:

- `&A` (reference) from `force()`, not `A` (value)
- `FnOnce` semantics (thunk runs once)

To bridge this gap requires changing either the redesigns (losing benefits) or the library (breaking changes).

### Change Option 1: Add Bounds to Kind Traits

**Current Kind Definition:**

```rust
// kinds.rs:57-59
def_kind! {
    type Of<'a, A: 'a>: 'a;  // No bounds on A beyond lifetime
}
```

**Proposed Change:**

```rust
// Create multiple Kind variants with different bound requirements
def_kind! {
    type Of<'a, A: 'a>: 'a;                    // Existing: Kind_cdc7cd43...
}

def_kind! {
    type Of<'a, A: 'a + Clone>: 'a;            // New: Kind with Clone bound
}

def_kind! {
    type Of<'a, A: 'a + Clone + Send + Sync>: 'a;  // New: Thread-safe variant
}
```

**Effect on Lazy:**

```rust
// Lazy could now declare its Clone requirement at the Kind level
impl_kind! {
    for LazyBrand {
        type Of<'a, A: 'a + Clone>: 'a = Lazy<'a, A>;  // Uses Clone-bound Kind
    }
}
```

**Impact Assessment:**

| Aspect             | Effect                                                 |
| ------------------ | ------------------------------------------------------ |
| Existing types     | Continue using unconstrained Kind                      |
| Lazy               | Can participate in HKT with Clone requirement visible  |
| Generic code       | Must specify which Kind variant it accepts             |
| Trait bounds       | Proliferate (Clone, Clone+Send, Clone+Send+Sync, etc.) |
| Library complexity | **Significantly increased**                            |

**Code Changes Required:**

1. Add new `def_kind!` invocations in `kinds.rs`
2. Update `impl_kind!` macro to handle bounded Kinds
3. Some typeclasses may need multiple implementations for different Kind variants
4. All documentation must explain Kind variants

### Change Option 2: Add `map_ref` to Functor Trait

**Proposed Addition:**

```rust
trait Functor: Kind_cdc7cd43... {
    /// Standard map - takes ownership of A
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Self::Of<'a, A>) -> Self::Of<'a, B>
    where
        F: Fn(A) -> B + 'a;

    /// Reference-based map - borrows A
    /// Default implementation clones; types can override for efficiency
    fn map_ref<'a, B: 'a, A: Clone + 'a, F>(f: F, fa: Self::Of<'a, A>) -> Self::Of<'a, B>
    where
        F: Fn(&A) -> B + 'a
    {
        Self::map(|a| f(&a), fa)  // Default: pass reference to owned value
    }
}
```

**Effect on Lazy:**

```rust
impl Functor for LazyBrand {
    fn map<'a, B: 'a, A: Clone + 'a, F>(f: F, fa: Lazy<'a, A>) -> Lazy<'a, B>
    where
        F: Fn(A) -> B + 'a,
    {
        // Must clone since we only have &A
        Lazy::new(move || f(fa.force().clone()))
    }

    fn map_ref<'a, B: 'a, A: Clone + 'a, F>(f: F, fa: Lazy<'a, A>) -> Lazy<'a, B>
    where
        F: Fn(&A) -> B + 'a
    {
        // Native efficient implementation
        Lazy::new(move || f(fa.force()))
    }
}
```

**Critical Problem:**

The `map` implementation for `LazyBrand` requires `A: Clone`, but the trait signature doesn't have that bound:

```rust
// Trait says:
fn map<'a, B: 'a, A: 'a, F>(...) // No Clone on A

// Our impl needs:
fn map<'a, B: 'a, A: Clone + 'a, F>(...) // Clone on A

// ERROR: impl is more restrictive than trait!
```

**Verdict:** This approach doesn't work in Rust's trait system.

### Change Option 3: Change Core Signatures to Reference-Based (Most Impactful)

**Proposed Change:**

```rust
// functor.rs - BREAKING CHANGE
trait Functor: Kind_cdc7cd43... {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Self::Of<'a, A>) -> Self::Of<'a, B>
    where
        F: Fn(&A) -> B + 'a;  // Reference-based!
}

// semimonad.rs - BREAKING CHANGE
trait Semimonad: Kind_cdc7cd43... {
    fn bind<'a, B: 'a, A: 'a, F>(ma: Self::Of<'a, A>, f: F) -> Self::Of<'a, B>
    where
        F: Fn(&A) -> Self::Of<'a, B> + 'a;  // Reference-based!
}

// semiapplicative.rs - BREAKING CHANGE
trait Semiapplicative: Lift + Functor {
    fn apply<'a, FnBrand: 'a + CloneableFn, B: 'a, A: 'a + Clone>(
        ff: Self::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>,  // Function takes A
        fa: Self::Of<'a, A>,
    ) -> Self::Of<'a, B>;
    // Note: CloneableFn still wraps Fn(A) -> B, not Fn(&A) -> B
    // This creates inconsistency - would need CloneableRefFn
}
```

**Effect on Existing Implementations:**

```rust
// Option - changes from:
impl Functor for OptionBrand {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Option<A>) -> Option<B>
    where F: Fn(A) -> B + 'a
    {
        fa.map(f)  // Standard library map
    }
}

// To:
impl Functor for OptionBrand {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Option<A>) -> Option<B>
    where F: Fn(&A) -> B + 'a
    {
        fa.map(|a| f(&a))  // Pass reference to f
    }
}
```

```rust
// Vec - changes from:
impl Functor for VecBrand {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Vec<A>) -> Vec<B>
    where F: Fn(A) -> B + 'a
    {
        fa.into_iter().map(f).collect()
    }
}

// To:
impl Functor for VecBrand {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Vec<A>) -> Vec<B>
    where F: Fn(&A) -> B + 'a
    {
        fa.iter().map(f).collect()  // iter() not into_iter()
        // WAIT: this returns &A to f, and f returns B
        // but we need to collect into Vec<B>, which works!
    }
}
```

**Effect on Lazy:**

```rust
// NOW WORKS!
impl Functor for LazyBrand {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Lazy<'a, A>) -> Lazy<'a, B>
    where F: Fn(&A) -> B + 'a
    {
        Lazy::new(move || f(fa.force()))  // force() returns &A, perfect!
    }
}
```

**Impact on User Code:**

```rust
// Before:
let result = map::<OptionBrand, _, _, _>(|x| x + 1, Some(5));

// After (if x is Copy):
let result = map::<OptionBrand, _, _, _>(|x| *x + 1, Some(5));

// After (if x is not Copy but Clone):
let result = map::<OptionBrand, _, _, _>(|x| x.clone() + 1, Some(5));

// For transformations needing ownership, add helper:
let result = map_owned::<OptionBrand, _, _, _>(|s| transform(s), Some(string));
// where map_owned clones internally
```

**Full Breaking Change Inventory:**

1. **`Functor` trait** - signature change
2. **`Semimonad` trait** - signature change
3. **All 12+ Functor implementations** - must update
4. **All Semimonad implementations** - must update
5. **`CloneableFn`** - needs `CloneableRefFn` variant for `Fn(&A) -> B`
6. **`Traversable`** - function signature changes
7. **`Witherable`** - function signature changes
8. **`Filterable`** - some functions affected
9. **All library tests** - must update
10. **All user code** - must update function arguments

**Semantic Consideration:**

When mapping functions need to consume their input (ownership):

```rust
// Before: natural
map(|s: String| expensive_transform(s), some_option)

// After: must clone
map(|s: &String| expensive_transform(s.clone()), some_option)
```

This adds an implicit `Clone` cost when ownership is needed. The library could provide `map_owned`:

```rust
pub fn map_owned<'a, Brand: Functor, B: 'a, A: Clone + 'a, F>(
    f: F,
    fa: Brand::Of<'a, A>,
) -> Brand::Of<'a, B>
where
    F: Fn(A) -> B + 'a,
{
    Brand::map(|a: &A| f(a.clone()), fa)
}
```

### Change Option 4: Change `Fn` to `FnOnce` (Partial Solution)

**Proposed Change:**

```rust
trait Functor: Kind_cdc7cd43... {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Self::Of<'a, A>) -> Self::Of<'a, B>
    where
        F: FnOnce(A) -> B + 'a;  // FnOnce instead of Fn
}
```

**Problem with Multi-Element Containers:**

```rust
impl Functor for VecBrand {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Vec<A>) -> Vec<B>
    where
        F: FnOnce(A) -> B + 'a  // Called once, but Vec has many elements!
    {
        // IMPOSSIBLE: f is consumed on first call
        fa.into_iter().map(f).collect()  // Error: f moved
    }
}
```

**Workaround:**

```rust
trait Functor: Kind_cdc7cd43... {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Self::Of<'a, A>) -> Self::Of<'a, B>
    where
        F: FnOnce(A) -> B + Clone + 'a;  // FnOnce + Clone
}
```

Now `Vec` can clone `f` for each element:

```rust
impl Functor for VecBrand {
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Vec<A>) -> Vec<B>
    where
        F: FnOnce(A) -> B + Clone + 'a
    {
        fa.into_iter().map(|a| f.clone()(a)).collect()
    }
}
```

**Impact:**

| Aspect                       | Effect                                            |
| ---------------------------- | ------------------------------------------------- |
| Option, Identity, Lazy       | ✅ Work naturally with FnOnce                      |
| Vec, multi-element           | ⚠️ Require Clone on F (slight overhead)            |
| All mapping closures         | Must now be Clone                                 |
| Simple closures              | Usually fine (closures are Clone if captures are) |
| Closures capturing non-Clone | ❌ Won't compile                                   |

### Change Option 5: Hybrid Approach (Recommended if Breaking)

**Strategy:** Combine reference-based signatures with ownership convenience methods.

**Core Changes:**

```rust
// functor.rs
trait Functor: Kind_cdc7cd43... {
    /// Core map - function receives reference
    fn map<'a, B: 'a, A: 'a, F>(f: F, fa: Self::Of<'a, A>) -> Self::Of<'a, B>
    where
        F: Fn(&A) -> B + 'a;
}

// semimonad.rs
trait Semimonad: Kind_cdc7cd43... {
    /// Core bind - function receives reference
    fn bind<'a, B: 'a, A: 'a, F>(ma: Self::Of<'a, A>, f: F) -> Self::Of<'a, B>
    where
        F: Fn(&A) -> Self::Of<'a, B> + 'a;
}
```

**Convenience Functions:**

```rust
// functions.rs

/// Ownership-based map - clones A before passing to f
pub fn map_owned<'a, Brand: Functor, B: 'a, A: Clone + 'a, F>(
    f: F,
    fa: Brand::Of<'a, A>,
) -> Brand::Of<'a, B>
where
    F: Fn(A) -> B + 'a,
{
    Brand::map(|a: &A| f(a.clone()), fa)
}

/// Ownership-based bind - clones A before passing to f
pub fn bind_owned<'a, Brand: Semimonad, B: 'a, A: Clone + 'a, F>(
    ma: Brand::Of<'a, A>,
    f: F,
) -> Brand::Of<'a, B>
where
    F: Fn(A) -> Brand::Of<'a, B> + 'a,
{
    Brand::bind(ma, |a: &A| f(a.clone()))
}
```

**Migration Guide for Users:**

```rust
// Pattern 1: Transform value (common case)
// Old: map(|x| x + 1, opt)
// New: map(|x| *x + 1, opt)  // Deref for Copy types

// Pattern 2: Clone and transform
// Old: map(|s| s.to_uppercase(), opt)
// New: map(|s| s.to_uppercase(), opt)  // Works! &String has to_uppercase

// Pattern 3: Consume value
// Old: map(|s| consume(s), opt)
// New: map_owned(|s| consume(s), opt)  // Use the _owned variant
```

### Summary: Trade-off Matrix for HKT Changes

| Change Option           | Lazy Compat  | Breaking Level | Complexity | Semantic Fit |
| ----------------------- | ------------ | -------------- | ---------- | ------------ |
| 1. Bounded Kinds        | ⚠️ Partial    | Medium         | High       | Medium       |
| 2. Add map_ref          | ❌ Impossible | N/A            | N/A        | N/A          |
| 3. Reference-based      | ✅ Full       | **Very High**  | Medium     | Good         |
| 4. FnOnce + Clone       | ⚠️ Partial    | High           | Low        | Medium       |
| 5. Hybrid (Ref + Owned) | ✅ Full       | **Very High**  | Medium     | Good         |

### Recommendation

If breaking changes are acceptable:

**Choose Option 5 (Hybrid)** because:

1. Enables full Lazy integration
2. Provides clear migration path (`map` → `map_owned` for ownership needs)
3. Reference semantics are more flexible (work with both owned and borrowed)
4. Consistent with other FP libraries that use reference-based combinators

If breaking changes must be avoided:

**Keep current system** and:

1. Accept Lazy won't implement Functor/Monad
2. Provide inherent methods on Lazy (`.map()`, `.flat_map()`)
3. Document the limitation clearly
4. Consider parallel typeclass hierarchies as future enhancement

---

## Conclusion

The proposed `Lazy` type redesigns and the library's HKT/typeclass system are fundamentally incompatible due to:

1. **`FnOnce` vs `Fn`**: Lazy thunks run once; library traits require multi-callable functions
2. **`&A` vs `A`**: Lazy returns references; library traits expect ownership
3. **Clone requirements**: Library's `CloneableFn` requires cloning; redesigns avoid it
4. **Brand integration**: Redesigns propose standalone types; library requires brands

These incompatibilities are not bugs but reflect **different design philosophies**:

- **Library**: Maximize abstraction and reuse through standard typeclasses
- **Redesigns**: Maximize ergonomics and safety for lazy evaluation

The recommended path forward is to:

1. Implement the redesigned `Lazy` with inherent methods (`map`, `flat_map`)
2. Keep compatible typeclass implementations (`Semigroup`, `Monoid`, `Defer`)
3. Document the design rationale clearly
4. Consider parallel typeclass hierarchies if demand emerges

This approach provides an ergonomic, safe `Lazy` type while maintaining honesty about what can and cannot integrate with the library's typeclass system.
