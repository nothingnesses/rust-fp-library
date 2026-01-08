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

**Goal**: Eliminate the `Function` trait and restrict `ClonableFn` usage.

#### Step 1.1: Delete Unused `Function` Trait

**File**: `fp-library/src/classes/function.rs`

**Action**: Delete the entire file. The `Function` trait is never used standalone—only `ClonableFn` is used throughout the codebase.

**Impact**:

- Remove from `fp-library/src/classes.rs` exports
- Update `ClonableFn` to not extend `Function`

#### Step 1.2: Restrict `ClonableFn` Usage

**File**: `fp-library/src/classes/clonable_fn.rs`

**Action**: Keep this trait, but remove it from the signatures of `Functor`, `Monad`, etc. It will only be used for types that inherently need to store functions (`Lazy`, `Defer`, `Endofunction`).

**Reasoning**: `Lazy` and `Defer` store thunks that must be clonable. We cannot easily replace this with `Box<dyn Fn>` because `Box` is not `Clone`.

---

### Phase 2: Uncurry Type Class Traits

**Goal**: Convert all curried type class methods to uncurried form with `impl Fn` parameters, while **preserving HKT infrastructure**.

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
    fn map<A, B, F>(f: F, fa: Apply0L1T<Self, A>) -> Apply0L1T<Self, B>
    where
        F: Fn(A) -> B;
}

pub fn map<Brand: Functor, A, B, F>(f: F, fa: Apply0L1T<Brand, A>) -> Apply0L1T<Brand, B>
where
    F: Fn(A) -> B
{
    Brand::map(f, fa)
}
```

**Reasoning**:

- **HKT Preserved**: `Apply0L1T<Self, A>` is kept.
- **Zero-Cost**: `F` is generic, allowing monomorphization and inlining.
- **Uncurried**: `f` and `fa` are passed together.

#### Step 2.2: Refactor `Semiapplicative` Trait

**File**: `fp-library/src/classes/semiapplicative.rs`

**Proposed**:

```rust
pub trait Semiapplicative: Kind0L1T {
    // Primary method: apply (functions in context)
    fn apply<A, B, F>(
        ff: Apply0L1T<Self, F>,
        fa: Apply0L1T<Self, A>
    ) -> Apply0L1T<Self, B>
    where
        F: Fn(A) -> B + Clone, // Clone often needed for cartesian products (e.g. Vec)
        A: Clone;

    // New method: map2 (lift2) - Enables zero-cost combination
    fn map2<A, B, C, F>(
        fa: Apply0L1T<Self, A>,
        fb: Apply0L1T<Self, B>,
        f: F
    ) -> Apply0L1T<Self, C>
    where
        F: Fn(A, B) -> C,
        A: Clone,
        B: Clone;
}
```

**Reasoning**:

- **`apply`**: Keeps `ff` as `Apply0L1T<Self, F>`. For `Vec`, `F` must be a concrete type. To store multiple different functions, users must use `Box<dyn Fn>` or `Rc<dyn Fn>` as `F`. This preserves the "functions as data" capability while making the cost explicit.
- **`map2`**: Added to allow combining two contexts _without_ creating intermediate closures stored in the container. This enables zero-cost `traverse` and `apply_first`.

#### Step 2.3: Refactor `Semimonad` Trait

**File**: `fp-library/src/classes/semimonad.rs`

**Proposed**:

```rust
pub trait Semimonad: Kind0L1T {
    fn bind<A, B, F>(
        ma: Apply0L1T<Self, A>,
        f: F
    ) -> Apply0L1T<Self, B>
    where
        F: Fn(A) -> Apply0L1T<Self, B>;
}
```

**Reasoning**: Standard `flat_map` signature. Zero-cost.

#### Step 2.4: Refactor `Foldable` Trait

**File**: `fp-library/src/classes/foldable.rs`

**Proposed**:

```rust
pub trait Foldable: Kind0L1T {
    fn fold_right<A, B, F>(f: F, init: B, fa: Apply0L1T<Self, A>) -> B
    where
        F: Fn(A, B) -> B;

    fn fold_left<A, B, F>(f: F, init: B, fa: Apply0L1T<Self, A>) -> B
    where
        F: Fn(B, A) -> B;

    fn fold_map<A, M, F>(f: F, fa: Apply0L1T<Self, A>) -> M
    where
        M: Monoid,
        F: Fn(A) -> M;
}
```

**Reasoning**: Standard uncurried fold signatures.

#### Step 2.5: Refactor `Traversable` Trait

**File**: `fp-library/src/classes/traversable.rs`

**Proposed**:

```rust
pub trait Traversable: Functor + Foldable {
    fn traverse<F, A, B, Func>(
        f: Func,
        ta: Apply0L1T<Self, A>
    ) -> Apply0L1T<F, Apply0L1T<Self, B>>
    where
        F: Applicative,
        Func: Fn(A) -> Apply0L1T<F, B>,
        Apply0L1T<F, B>: Clone,
        Apply0L1T<Self, B>: Clone;

    // sequence remains similar but uncurried
}
```

**Reasoning**: `traverse` can now use `Applicative::map2` (if available) or `apply` to combine results.

#### Step 2.6: Refactor `ApplyFirst` and `ApplySecond` Traits

**Files**:

- `fp-library/src/classes/apply_first.rs`
- `fp-library/src/classes/apply_second.rs`

**Proposed**:

```rust
pub trait ApplyFirst: Kind0L1T {
    fn apply_first<A, B>(
        fa: Apply0L1T<Self, A>,
        fb: Apply0L1T<Self, B>
    ) -> Apply0L1T<Self, A>;
}
```

**Reasoning**: Default implementation can use `map2` (zero-cost) or `apply` (requires boxing in default impl).

#### Step 2.7: Refactor `Semigroup` Trait

**File**: `fp-library/src/classes/semigroup.rs`

**Proposed**:

```rust
pub trait Semigroup {
    fn append(a: Self, b: Self) -> Self;
}
```

**Reasoning**: Simplified to standard Rust style. Removed lifetime `'b` and `ClonableFnBrand` as they are implementation details of specific semigroups (like `Endofunction`), not the trait itself.

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
impl Functor for OptionBrand {
    fn map<A, B, F>(f: F, fa: Apply0L1T<Self, A>) -> Apply0L1T<Self, B>
    where
        F: Fn(A) -> B
    {
        // fa is Option<A>
        fa.map(f)
    }
}
```

#### Step 3.2: Update `VecBrand` Implementation

**File**: `fp-library/src/types/vec.rs`

**Action**: Implement `map`, `apply`, `bind`, etc. using standard iterator methods (`map`, `flat_map`).
**Optimization**: Implement `fold_right` / `fold_left` directly using `DoubleEndedIterator` / `Iterator::fold` to avoid `Endofunction` overhead.

#### Step 3.3: Update Other Brands

Update `IdentityBrand`, `ResultWithErrBrand`, `ResultWithOkBrand`, `PairWithFirstBrand`, `PairWithSecondBrand` similarly.

---

### Phase 4: Update Helper Functions

**Goal**: Update standalone helper functions to use uncurried signatures.

#### Step 4.1: Update `compose` Function

**File**: `fp-library/src/functions.rs`

**Proposed**:

```rust
pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(B) -> C,
    G: Fn(A) -> B
{
    move |a| f(g(a))
}
```

#### Step 4.2: Update `constant` Function

**Proposed**:

```rust
pub fn constant<A: Clone, B>(a: A) -> impl Fn(B) -> A {
    move |_| a.clone()
}
```

#### Step 4.3: Update `flip` Function

**Proposed**:

```rust
pub fn flip<A, B, C, F>(f: F) -> impl Fn(B, A) -> C
where
    F: Fn(A, B) -> C
{
    move |b, a| f(a, b)
}
```

---

### Phase 5: Update Endofunction/Endomorphism Types

**Goal**: Maintain these types for `Foldable` defaults but allow optimization.

#### Step 5.1: Keep `Endofunction` Wrapper

**File**: `fp-library/src/types/endofunction.rs`

**Action**: Keep the current design (wrapping `ClonableFn`).
**Reasoning**: `Endofunction` is primarily used in the default implementation of `fold_right`. Since `fold_right` composes functions dynamically based on the list length, the composed function type changes at runtime (conceptually). Rust requires a single concrete type for the accumulator. `Rc<dyn Fn>` / `Arc<dyn Fn>` provides this type erasure.

**Optimization**: Ensure concrete types like `Vec` implement `fold_right` directly to bypass this wrapper.

---

### Phase 6: Update Brand Infrastructure

**Goal**: Decide what to keep for `apply` operations.

#### Step 6.1: Keep Minimal Wrapper Types

**Files**:

- `fp-library/src/types/arc_fn.rs`
- `fp-library/src/types/rc_fn.rs`

**Action**: Keep these brands. They are needed when users want to use `apply` with heterogeneous functions (e.g. `Vec<Rc<dyn Fn>>`).

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

### Why `map2` is Necessary for Zero-Cost

Relying solely on `apply` (even uncurried) forces currying of combining functions.
To combine `fa` and `fb` using `apply`:

1. `map(|a| |b| (a, b), fa)` -> produces `F<Closure>`
2. `apply(F<Closure>, fb)` -> produces `F<(A, B)>`

This forces the creation of an intermediate closure stored in the container. For `Vec`, this means `Vec<Closure>`. While Rust handles `Vec<Closure>` efficiently if they are homogeneous, `map2` avoids this entirely:
`map2(fa, fb, |a, b| (a, b))` -> combines directly without intermediate storage.

### Why `Endofunction` Must Remain Dynamic

`Endofunction` implements `Monoid`. The `append` operation composes two functions: `f ∘ g`.
In Rust, the type of `f ∘ g` is distinct from the type of `f` and `g`.
`Monoid::append` requires `(Self, Self) -> Self`.
Therefore, `Self` must be a type that can represent any composition of functions. Only a trait object (`dyn Fn`) satisfies this. Thus, `Endofunction` must wrap a dynamic function pointer (`Rc` or `Arc`).

### Trade-offs Summary

| Aspect                  | Current Design             | Proposed Design                                      |
| ----------------------- | -------------------------- | ---------------------------------------------------- |
| **`map`, `bind`**       | Dynamic dispatch + Rc      | Zero-cost (monomorphized)                            |
| **`traverse`**          | Dynamic dispatch + Rc      | Zero-cost (via `map2`)                               |
| **`apply`**             | Dynamic dispatch + Rc      | Zero-cost for homogeneous, Dynamic for heterogeneous |
| **`fold`**              | Dynamic dispatch (default) | Zero-cost (direct impls)                             |
| **Ergonomics**          | Very verbose               | Much cleaner                                         |
| **Type errors**         | Complex                    | Simpler                                              |
| **Currying**            | Native                     | Requires manual wrapping                             |
| **Partial application** | Built-in                   | Requires explicit closures                           |

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
