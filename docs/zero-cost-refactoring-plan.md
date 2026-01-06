# Zero-Cost Abstractions Refactoring Plan

## Table of Contents

1. [Conversation Summary](#conversation-summary)
2. [Current Architecture Analysis](#current-architecture-analysis)
3. [Refactoring Plan](#refactoring-plan)
4. [Reasoning and Justification](#reasoning-and-justification)
5. [Migration Strategy](#migration-strategy)

---

## Conversation Summary

### Initial Questions

The conversation began with an analysis of the `Function` and `ClonableFn` traits in the fp-library:

**Questions Asked:**

1. Does the existence of the traits `Function` and `ClonableFn` make sense?
2. Is their existence reasonable and justified?
3. Are the traits appropriate/good abstractions?
4. Do they have drawbacks and limitations?
5. Are there better, more appropriate abstractions?

### Analysis Findings

#### What These Traits Accomplish

The `Function` and `ClonableFn` traits solve a fundamental challenge in Rust: **closures are anonymous types that don't implement `Clone`**. In functional programming, you frequently need to:

1. Pass functions generically (e.g., to `Functor::map`)
2. Clone functions (e.g., when applying the same function multiple times in `Semiapplicative::apply`)
3. Store functions in data structures

The library's solution wraps closures in smart pointers (`Rc`/`Arc`) and abstracts over this choice using "brand" types:

| Brand        | Wrapper               | Thread-safe? |
| ------------ | --------------------- | ------------ |
| `RcFnBrand`  | `Rc<dyn Fn(A) -> B>`  | No           |
| `ArcFnBrand` | `Arc<dyn Fn(A) -> B>` | Yes          |

#### Identified Drawbacks

1. **Ergonomic Overhead**: Every function call requires explicit wrapping and brand specification

   ```rust
   // Current verbose syntax:
   map::<RcFnBrand, VecBrand, _, _>(Rc::new(|x: i32| x * 2))(vec![1, 2, 3])
   ```

2. **Runtime Costs**:

   - Dynamic dispatch through `dyn Fn` (vtable lookup on every call)
   - Reference counting overhead (`Rc`/`Arc` increment/decrement)
   - Cache-unfriendly indirection due to heap-allocated closures

3. **Redundancy Between Traits**: `Function` and `ClonableFn` have nearly identical definitions, with the only difference being the `Clone` bound. `Function` is never used standalone in the codebase.

4. **Type Complexity**: Generic type parameters proliferate through the codebase, making error messages difficult to read.

### User's Proposed Change

The user proposed moving from curried semantics to uncurried semantics, where functions would take all parameters at once (likely in a tuple or struct) instead of returning closures.

### Impact Analysis of Uncurrying

#### Three Sources of Current Overhead

| Source           | Cause                                  | Example                                  |
| ---------------- | -------------------------------------- | ---------------------------------------- |
| **Currying**     | Partial application creates closures   | `map(f)` returns a closure capturing `f` |
| **Clonability**  | Closures must be wrapped in `Rc`/`Arc` | `ApplyClonableFn<'a, RcFnBrand, A, B>`   |
| **Type erasure** | `dyn Fn` for uniform function types    | Dynamic vtable dispatch                  |

#### What Uncurrying Eliminates

1. **Intermediate closures** from partial application
2. **`Rc`/`Arc` wrapping** for most operations (`map`, `bind`, `fold`)

#### What Uncurrying Does NOT Eliminate

- **Dynamic dispatch** → Requires switching from `dyn Fn` to `impl Fn`/generic bounds
- **Functions-as-data** (like in `apply`) → Inherent to the abstraction

### Recommended Approach

A **hybrid approach** was recommended:

1. **Use uncurried + `impl Fn`** for most operations (`map`, `bind`, `fold`, `traverse`):

   ```rust
   fn map<F: Fn(A) -> B>(f: F, fa: Vec<A>) -> Vec<B>
   ```

   → Zero-cost, fully monomorphized

2. **Accept type erasure only for `apply`** where functions are genuinely data:

   ```rust
   fn apply<A, B>(ff: Vec<Rc<dyn Fn(A) -> B>>, fa: Vec<A>) -> Vec<B>
   ```

   → Overhead is inherent to the abstraction

3. **Optionally provide defunctionalized variants** for performance-critical paths where the function set is known

---

## Current Architecture Analysis

### Files and Their Curried Patterns

#### Type Classes (`fp-library/src/classes/`)

| File                 | Traits/Functions                                  | Currying Pattern                                           |
| -------------------- | ------------------------------------------------- | ---------------------------------------------------------- |
| `function.rs`        | `Function`                                        | Core abstraction for wrapped functions                     |
| `clonable_fn.rs`     | `ClonableFn`                                      | Extends `Function` with `Clone`                            |
| `functor.rs`         | `Functor`, `map`                                  | `map(f) -> (fa -> fb)` - curried                           |
| `semiapplicative.rs` | `Semiapplicative`, `apply`                        | `apply(ff) -> (fa -> fb)` - curried, **functions-as-data** |
| `semimonad.rs`       | `Semimonad`, `bind`                               | `bind(ma) -> (f -> mb)` - curried                          |
| `foldable.rs`        | `Foldable`, `fold_left`, `fold_right`, `fold_map` | Heavily curried with nested closures                       |
| `traversable.rs`     | `Traversable`, `traverse`, `sequence`             | `traverse(f) -> (ta -> f(tb))` - curried                   |
| `apply_first.rs`     | `ApplyFirst`, `apply_first`                       | `apply_first(fa) -> (fb -> fa)` - curried                  |
| `apply_second.rs`    | `ApplySecond`, `apply_second`                     | `apply_second(fa) -> (fb -> fb)` - curried                 |
| `semigroup.rs`       | `Semigroup`, `append`                             | `append(a) -> (b -> ab)` - curried                         |
| `semigroupoid.rs`    | `Semigroupoid`, `compose`                         | `compose(f) -> (g -> fg)` - curried                        |
| `category.rs`        | `Category`, `identity`                            | Not curried                                                |
| `pointed.rs`         | `Pointed`, `pure`                                 | Not curried                                                |
| `monoid.rs`          | `Monoid`, `empty`                                 | Not curried                                                |
| `applicative.rs`     | `Applicative`                                     | Marker trait (blanket impl)                                |
| `monad.rs`           | `Monad`                                           | Marker trait (blanket impl)                                |
| `defer.rs`           | `Defer`                                           | Uses `ClonableFn`                                          |
| `once.rs`            | `Once`                                            | Not curried (uses `FnOnce`)                                |

#### Helper Functions (`fp-library/src/functions.rs`)

| Function   | Current Signature            | Notes           |
| ---------- | ---------------------------- | --------------- |
| `compose`  | `compose(f) -> (g -> fg)`    | Heavily curried |
| `constant` | `constant(a) -> (b -> a)`    | Curried         |
| `flip`     | `flip(f) -> (b -> (a -> c))` | Heavily curried |
| `identity` | `identity(a) -> a`           | Not curried     |

#### Type Implementations (`fp-library/src/types/`)

| File                        | Brand                                                    | Curried Methods                                                                        |
| --------------------------- | -------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `option.rs`                 | `OptionBrand`                                            | All type class implementations use curried `ClonableFn`                                |
| `vec.rs`                    | `VecBrand`                                               | All type class implementations use curried `ClonableFn`, plus `construct`              |
| `identity.rs`               | `IdentityBrand`                                          | All type class implementations use curried `ClonableFn`                                |
| `result/result_with_err.rs` | `ResultWithErrBrand<E>`                                  | All type class implementations use curried `ClonableFn`                                |
| `result/result_with_ok.rs`  | `ResultWithOkBrand<T>`                                   | All type class implementations use curried `ClonableFn`                                |
| `pair.rs`                   | `PairBrand`, `PairWithFirstBrand`, `PairWithSecondBrand` | `Pair::new` is curried                                                                 |
| `arc_fn.rs`                 | `ArcFnBrand`                                             | Implements `Function`, `ClonableFn`, `Semigroupoid`, `Category`, `Semigroup`, `Monoid` |
| `rc_fn.rs`                  | `RcFnBrand`                                              | Implements `Function`, `ClonableFn`, `Semigroupoid`, `Category`, `Semigroup`, `Monoid` |
| `endofunction.rs`           | `EndofunctionBrand`                                      | Uses `ClonableFn` internally, implements `Semigroup`, `Monoid`                         |
| `endomorphism.rs`           | `EndomorphismBrand`                                      | Uses `Category` internally, implements `Semigroup`, `Monoid`                           |
| `lazy.rs`                   | `LazyBrand`                                              | Uses `Defer` trait                                                                     |

#### Higher-Kinded Type Infrastructure (`fp-library/src/hkt/`)

| File       | Purpose                                                         | Impact            |
| ---------- | --------------------------------------------------------------- | ----------------- |
| `kinds.rs` | `Kind0L1T`, `Kind0L2T`, `Kind1L0T`, `Kind1L2T` traits           | No changes needed |
| `apply.rs` | `Apply0L1T`, `Apply0L2T`, `Apply1L0T`, `Apply1L2T` type aliases | No changes needed |

#### Macros (`fp-library/src/macros.rs`)

| Macro              | Purpose                        | Impact            |
| ------------------ | ------------------------------ | ----------------- |
| `make_trait_kind!` | Generates `Kind` traits        | No changes needed |
| `make_type_apply!` | Generates `Apply` type aliases | No changes needed |

---

## Refactoring Plan

### Phase 1: Remove/Simplify Function Wrapper Traits

**Goal**: Eliminate the `Function` and `ClonableFn` traits and their associated brand infrastructure.

#### Step 1.1: Delete Unused `Function` Trait

**File**: `fp-library/src/classes/function.rs`

**Action**: Delete the entire file. The `Function` trait is never used standalone—only `ClonableFn` is used throughout the codebase.

**Impact**:

- Remove from `fp-library/src/classes.rs` exports
- Update `ClonableFn` to not extend `Function`

#### Step 1.2: Delete `ClonableFn` Trait and Infrastructure

**File**: `fp-library/src/classes/clonable_fn.rs`

**Action**: Delete the entire file. With uncurried semantics, wrapped closures are no longer needed for most operations.

**Impact**:

- Remove from `fp-library/src/classes.rs` exports
- Remove all `ClonableFnBrand` generic parameters from type class traits and functions
- Remove all `ApplyClonableFn` usages

#### Step 1.3: Update Brand Types

**Files**:

- `fp-library/src/types/arc_fn.rs`
- `fp-library/src/types/rc_fn.rs`

**Action**: Remove `Function` and `ClonableFn` implementations. Keep these types only for the `apply` operation where functions-as-data is required.

**New Purpose**: These brands will only be used when functions need to be stored in containers (for `Semiapplicative::apply`).

---

### Phase 2: Uncurry Type Class Traits

**Goal**: Convert all curried type class methods to uncurried form with `impl Fn` parameters.

#### Step 2.1: Refactor `Functor` Trait

**File**: `fp-library/src/classes/functor.rs`

**Current**:

```rust
pub trait Functor: Kind0L1T {
    fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
        f: ApplyClonableFn<'a, ClonableFnBrand, A, B>
    ) -> ApplyClonableFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>>;
}
```

**Proposed**:

```rust
pub trait Functor: Kind0L1T {
    fn map<A, B, F: Fn(A) -> B>(f: F, fa: Apply0L1T<Self, A>) -> Apply0L1T<Self, B>;
}

pub fn map<Brand: Functor, A, B, F: Fn(A) -> B>(f: F, fa: Apply0L1T<Brand, A>) -> Apply0L1T<Brand, B> {
    Brand::map(f, fa)
}
```

#### Step 2.2: Refactor `Semiapplicative` Trait

**File**: `fp-library/src/classes/semiapplicative.rs`

**Note**: This trait requires special handling because `ff: F<A -> B>` contains functions-as-data.

**Current**:

```rust
pub trait Semiapplicative: Kind0L1T {
    fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
        ff: Apply0L1T<Self, ApplyClonableFn<'a, ClonableFnBrand, A, B>>
    ) -> ApplyClonableFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>>;
}
```

**Proposed** (Option A - Keep type erasure for functions-as-data):

```rust
pub trait Semiapplicative: Kind0L1T {
    fn apply<'a, A: 'a + Clone, B: 'a>(
        ff: Apply0L1T<Self, Rc<dyn 'a + Fn(A) -> B>>,
        fa: Apply0L1T<Self, A>
    ) -> Apply0L1T<Self, B>;
}
```

**Proposed** (Option B - Generic over function wrapper):

```rust
pub trait Semiapplicative: Kind0L1T {
    fn apply<A: Clone, B, Func: Fn(A) -> B + Clone>(
        ff: Apply0L1T<Self, Func>,
        fa: Apply0L1T<Self, A>
    ) -> Apply0L1T<Self, B>;
}
```

**Recommendation**: Use Option B for maximum flexibility, accepting that users may need to wrap in `Rc` when using heterogeneous function collections.

#### Step 2.3: Refactor `Semimonad` Trait

**File**: `fp-library/src/classes/semimonad.rs`

**Current**:

```rust
pub trait Semimonad: Kind0L1T {
    fn bind<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
        ma: Apply0L1T<Self, A>
    ) -> ApplyClonableFn<
        'a,
        ClonableFnBrand,
        ApplyClonableFn<'a, ClonableFnBrand, A, Apply0L1T<Self, B>>,
        Apply0L1T<Self, B>,
    >;
}
```

**Proposed**:

```rust
pub trait Semimonad: Kind0L1T {
    fn bind<A, B, F: Fn(A) -> Apply0L1T<Self, B>>(
        ma: Apply0L1T<Self, A>,
        f: F
    ) -> Apply0L1T<Self, B>;
}

pub fn bind<Brand: Semimonad, A, B, F: Fn(A) -> Apply0L1T<Brand, B>>(
    ma: Apply0L1T<Brand, A>,
    f: F
) -> Apply0L1T<Brand, B> {
    Brand::bind(ma, f)
}
```

#### Step 2.4: Refactor `Foldable` Trait

**File**: `fp-library/src/classes/foldable.rs`

**Current** (heavily curried):

```rust
fn fold_right<'a, ClonableFnBrand: 'a + ClonableFn, A: Clone, B: Clone>(
    f: ApplyClonableFn<'a, ClonableFnBrand, A, ApplyClonableFn<'a, ClonableFnBrand, B, B>>
) -> ApplyClonableFn<
    'a,
    ClonableFnBrand,
    B,
    ApplyClonableFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>,
>;
```

**Proposed**:

```rust
pub trait Foldable: Kind0L1T {
    fn fold_right<A, B, F: Fn(A, B) -> B>(f: F, init: B, fa: Apply0L1T<Self, A>) -> B;

    fn fold_left<A, B, F: Fn(B, A) -> B>(f: F, init: B, fa: Apply0L1T<Self, A>) -> B;

    fn fold_map<A, M: Monoid, F: Fn(A) -> M>(f: F, fa: Apply0L1T<Self, A>) -> M;
}
```

#### Step 2.5: Refactor `Traversable` Trait

**File**: `fp-library/src/classes/traversable.rs`

**Proposed**:

```rust
pub trait Traversable: Functor + Foldable {
    fn traverse<F: Applicative, A, B, Func: Fn(A) -> Apply0L1T<F, B>>(
        f: Func,
        ta: Apply0L1T<Self, A>
    ) -> Apply0L1T<F, Apply0L1T<Self, B>>
    where
        Apply0L1T<F, B>: Clone,
        Apply0L1T<Self, B>: Clone;

    fn sequence<F: Applicative, A>(
        t: Apply0L1T<Self, Apply0L1T<F, A>>
    ) -> Apply0L1T<F, Apply0L1T<Self, A>>
    where
        Apply0L1T<F, A>: Clone,
        Apply0L1T<Self, A>: Clone;
}
```

#### Step 2.6: Refactor `ApplyFirst` and `ApplySecond` Traits

**Files**:

- `fp-library/src/classes/apply_first.rs`
- `fp-library/src/classes/apply_second.rs`

**Proposed**:

```rust
pub trait ApplyFirst: Kind0L1T {
    fn apply_first<A, B>(fa: Apply0L1T<Self, A>, fb: Apply0L1T<Self, B>) -> Apply0L1T<Self, A>;
}

pub trait ApplySecond: Kind0L1T {
    fn apply_second<A, B>(fa: Apply0L1T<Self, A>, fb: Apply0L1T<Self, B>) -> Apply0L1T<Self, B>;
}
```

#### Step 2.7: Refactor `Semigroup` Trait

**File**: `fp-library/src/classes/semigroup.rs`

**Current**:

```rust
pub trait Semigroup<'b> {
    fn append<'a, ClonableFnBrand: 'a + 'b + ClonableFn>(
        a: Self
    ) -> ApplyClonableFn<'a, ClonableFnBrand, Self, Self>
    where
        Self: Sized,
        'b: 'a;
}
```

**Proposed**:

```rust
pub trait Semigroup {
    fn append(a: Self, b: Self) -> Self;
}

pub fn append<S: Semigroup>(a: S, b: S) -> S {
    S::append(a, b)
}
```

#### Step 2.8: Refactor `Semigroupoid` Trait

**File**: `fp-library/src/classes/semigroupoid.rs`

**Proposed**:

```rust
pub trait Semigroupoid: Kind1L2T {
    fn compose<'a, B, C, D>(
        f: Apply1L2T<'a, Self, C, D>,
        g: Apply1L2T<'a, Self, B, C>
    ) -> Apply1L2T<'a, Self, B, D>;
}
```

---

### Phase 3: Update Type Implementations

**Goal**: Update all type implementations to use the new uncurried signatures.

#### Step 3.1: Update `OptionBrand` Implementation

**File**: `fp-library/src/types/option.rs`

**Example transformation for `map`**:

**Current**:

```rust
fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
    f: ApplyClonableFn<'a, ClonableFnBrand, A, B>
) -> ApplyClonableFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
    <ClonableFnBrand as ClonableFn>::new(move |fa: Apply0L1T<Self, _>| fa.map(&*f))
}
```

**Proposed**:

```rust
fn map<A, B, F: Fn(A) -> B>(f: F, fa: Option<A>) -> Option<B> {
    fa.map(f)
}
```

#### Step 3.2: Update `VecBrand` Implementation

**File**: `fp-library/src/types/vec.rs`

Similar transformations for all type class implementations.

#### Step 3.3: Update `IdentityBrand` Implementation

**File**: `fp-library/src/types/identity.rs`

Similar transformations for all type class implementations.

#### Step 3.4: Update `ResultWithErrBrand` and `ResultWithOkBrand` Implementations

**Files**:

- `fp-library/src/types/result/result_with_err.rs`
- `fp-library/src/types/result/result_with_ok.rs`

Similar transformations for all type class implementations.

#### Step 3.5: Update `PairWithFirstBrand` and `PairWithSecondBrand` Implementations

**Files**:

- `fp-library/src/types/pair/pair_with_first.rs`
- `fp-library/src/types/pair/pair_with_second.rs`

Similar transformations.

---

### Phase 4: Update Helper Functions

**Goal**: Update standalone helper functions to use uncurried signatures.

#### Step 4.1: Update `compose` Function

**File**: `fp-library/src/functions.rs`

**Current**:

```rust
pub fn compose<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a, C: 'a>(
    f: ApplyClonableFn<'a, ClonableFnBrand, B, C>
) -> ApplyClonableFn<
    'a,
    ClonableFnBrand,
    ApplyClonableFn<'a, ClonableFnBrand, A, B>,
    ApplyClonableFn<'a, ClonableFnBrand, A, C>,
> {
    <ClonableFnBrand as ClonableFn>::new(move |g: ApplyClonableFn<'a, ClonableFnBrand, _, _>| {
        let f = f.clone();
        <ClonableFnBrand as ClonableFn>::new(move |a| f(g(a)))
    })
}
```

**Proposed**:

```rust
pub fn compose<A, B, C, F: Fn(B) -> C, G: Fn(A) -> B>(f: F, g: G) -> impl Fn(A) -> C {
    move |a| f(g(a))
}
```

#### Step 4.2: Update `constant` Function

**Current**:

```rust
pub fn constant<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
    a: A
) -> ApplyClonableFn<'a, ClonableFnBrand, B, A> {
    <ClonableFnBrand as ClonableFn>::new(move |_b| a.to_owned())
}
```

**Proposed**:

```rust
pub fn constant<A: Clone, B>(a: A) -> impl Fn(B) -> A {
    move |_| a.clone()
}
```

#### Step 4.3: Update `flip` Function

**Current**:

```rust
pub fn flip<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a + Clone, C: 'a>(
    f: ApplyClonableFn<'a, ClonableFnBrand, A, ApplyClonableFn<'a, ClonableFnBrand, B, C>>
) -> ApplyClonableFn<'a, ClonableFnBrand, B, ApplyClonableFn<'a, ClonableFnBrand, A, C>> {
    // ...
}
```

**Proposed**:

```rust
pub fn flip<A, B, C, F: Fn(A, B) -> C>(f: F) -> impl Fn(B, A) -> C {
    move |b, a| f(a, b)
}
```

---

### Phase 5: Update Endofunction/Endomorphism Types

**Goal**: Simplify or remove these types that were primarily needed for curried fold operations.

#### Step 5.1: Simplify `Endofunction`

**File**: `fp-library/src/types/endofunction.rs`

**Current**: Wraps `ApplyClonableFn<'a, ClonableFnBrand, A, A>` for monoidal composition.

**Proposed**: With uncurried semantics, this can be simplified to:

```rust
pub struct Endofunction<A>(pub Box<dyn Fn(A) -> A>);

impl<A> Semigroup for Endofunction<A> {
    fn append(a: Self, b: Self) -> Self {
        Endofunction(Box::new(move |x| a.0(b.0(x))))
    }
}

impl<A> Monoid for Endofunction<A> {
    fn empty() -> Self {
        Endofunction(Box::new(identity))
    }
}
```

**Note**: For zero-cost, consider providing a generic version:

```rust
pub struct Endofunction<F>(pub F);

impl<A, F: Fn(A) -> A, G: Fn(A) -> A> ... // More complex but zero-cost
```

#### Step 5.2: Simplify `Endomorphism`

**File**: `fp-library/src/types/endomorphism.rs`

Similar simplification.

---

### Phase 6: Update Brand Infrastructure

**Goal**: Decide what to keep for `apply` operations.

#### Step 6.1: Keep Minimal Wrapper Types

**Files**:

- `fp-library/src/types/arc_fn.rs`
- `fp-library/src/types/rc_fn.rs`

**Action**: Keep these as simple type aliases or wrapper types for use in `apply`:

```rust
// In arc_fn.rs
pub type ArcFn<'a, A, B> = Arc<dyn 'a + Fn(A) -> B>;

// In rc_fn.rs
pub type RcFn<'a, A, B> = Rc<dyn 'a + Fn(A) -> B>;
```

---

### Phase 7: Update Documentation and Examples

**Goal**: Update all documentation and examples to reflect the new API.

#### Step 7.1: Update Doc Comments

All trait and function documentation needs to be updated with new signatures and examples.

#### Step 7.2: Update README

Update the main README.md with new usage patterns.

#### Step 7.3: Update Doc Tests

All doc tests need to be rewritten to use the new uncurried API.

---

## Reasoning and Justification

### Why Uncurrying Achieves Zero-Cost

1. **Eliminates Intermediate Closures**

   Curried functions create a closure for each partial application:

   ```rust
   // Curried: creates closure on each call
   let map_f = map(f);     // Creates closure capturing f
   let result = map_f(fa); // Calls the closure
   ```

   Uncurried functions are called directly:

   ```rust
   // Uncurried: direct call, no closure creation
   let result = map(f, fa);
   ```

2. **Enables Monomorphization**

   With `impl Fn` or generic bounds:

   ```rust
   fn map<F: Fn(A) -> B>(f: F, fa: Vec<A>) -> Vec<B>
   ```

   The compiler generates specialized code for each concrete function type. No vtable, no indirection.

3. **Removes Reference Counting Overhead**

   Without needing to clone functions for capture, `Rc`/`Arc` wrapping is unnecessary for most operations.

### Why Functions-as-Data Still Requires Overhead

The `apply` operation has type `f (a -> b) -> f a -> f b`. The functions are **inside** the functor:

```rust
apply(Some(|x| x * 2), Some(5))  // The function is stored in `Option`
apply(vec![|x| x+1, |x| x*2], vec![1,2])  // Functions are stored in `Vec`
```

With `impl Fn`, each closure has a unique anonymous type. You cannot put different closures in the same `Vec` without type erasure:

```rust
// Won't compile - different closure types
let funcs: Vec<impl Fn(i32) -> i32> = vec![|x| x+1, |x| x*2];

// Requires type erasure
let funcs: Vec<Box<dyn Fn(i32) -> i32>> = vec![Box::new(|x| x+1), Box::new(|x| x*2)];
```

This overhead is **inherent to the abstraction**, not a design flaw.

### Trade-offs Summary

| Aspect                    | Current Design        | Proposed Design                     |
| ------------------------- | --------------------- | ----------------------------------- |
| **`map`, `bind`, `fold`** | Dynamic dispatch + Rc | Zero-cost (monomorphized)           |
| **`apply`**               | Dynamic dispatch + Rc | Dynamic dispatch + Rc (unavoidable) |
| **Ergonomics**            | Very verbose          | Much cleaner                        |
| **Type errors**           | Complex               | Simpler                             |
| **Currying**              | Native                | Requires manual wrapping            |
| **Partial application**   | Built-in              | Requires explicit closures          |

### Loss of Currying

The main trade-off is losing native currying. Current:

```rust
let double_all = map::<RcFnBrand, VecBrand, _, _>(Rc::new(|x: i32| x * 2));
// double_all can be reused
```

With uncurried:

```rust
// Need explicit closure for partial application
let double_all = |v| map(|x: i32| x * 2, v);
```

This is considered an acceptable trade-off because:

1. Rust idioms don't typically use currying
2. The performance benefits are significant
3. The ergonomic improvement outweighs the loss

---

## Migration Strategy

### Approach: Parallel Implementation

1. **Create new module structure** alongside existing code
2. **Implement new traits** with `_v2` suffix initially
3. **Migrate implementations** one type at a time
4. **Update tests** to cover both versions
5. **Deprecate old API** with compiler warnings
6. **Remove old API** in next major version

### Versioning

- **Current version**: 0.x.y (curried)
- **Intermediate version**: 0.(x+1).0 (both APIs available, old deprecated)
- **New version**: 1.0.0 (uncurried only)

### Testing Strategy

1. **Unit tests**: Update to use new API
2. **Property tests**: Verify laws hold for both APIs
3. **Benchmark tests**: Add benchmarks comparing both approaches
4. **Doc tests**: Rewrite all examples

---

## Files to Modify Summary

### Delete

- `fp-library/src/classes/function.rs`
- `fp-library/src/classes/clonable_fn.rs`

### Major Refactor

- `fp-library/src/classes/functor.rs`
- `fp-library/src/classes/semiapplicative.rs`
- `fp-library/src/classes/semimonad.rs`
- `fp-library/src/classes/foldable.rs`
- `fp-library/src/classes/traversable.rs`
- `fp-library/src/classes/apply_first.rs`
- `fp-library/src/classes/apply_second.rs`
- `fp-library/src/classes/semigroup.rs`
- `fp-library/src/classes/semigroupoid.rs`
- `fp-library/src/functions.rs`
- `fp-library/src/types/option.rs`
- `fp-library/src/types/vec.rs`
- `fp-library/src/types/identity.rs`
- `fp-library/src/types/result/result_with_err.rs`
- `fp-library/src/types/result/result_with_ok.rs`
- `fp-library/src/types/pair/pair_with_first.rs`
- `fp-library/src/types/pair/pair_with_second.rs`
- `fp-library/src/types/endofunction.rs`
- `fp-library/src/types/endomorphism.rs`

### Minor Updates

- `fp-library/src/classes.rs` (remove exports)
- `fp-library/src/brands.rs` (update exports)
- `fp-library/src/types/arc_fn.rs` (simplify)
- `fp-library/src/types/rc_fn.rs` (simplify)

### No Changes Needed

- `fp-library/src/hkt/kinds.rs`
- `fp-library/src/hkt/apply.rs`
- `fp-library/src/macros.rs`
- `fp-library/src/classes/category.rs`
- `fp-library/src/classes/pointed.rs`
- `fp-library/src/classes/monoid.rs`
- `fp-library/src/classes/applicative.rs`
- `fp-library/src/classes/monad.rs`
- `fp-library/src/classes/once.rs`
- `fp-library/src/classes/defer.rs` (minor update)
