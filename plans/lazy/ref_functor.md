# RefFunctor Trait Analysis

## Overview

`RefFunctor` is a type class for types whose mapping operation receives a reference (`&A`) rather than an owned value (`A`). It exists to support memoized lazy types (`RcLazy`, `RcTryLazy`) where `evaluate()` returns `&A`, making a standard `Functor` implementation impossible without implicit cloning.

**File:** `fp-library/src/classes/ref_functor.rs`

---

## 1. Why RefFunctor Is Separate from Functor

The core issue: `Lazy::evaluate(&self) -> &A` returns a borrow. The standard `Functor` trait requires `map(f: impl Fn(A) -> B, fa: F<A>) -> F<B>`, where `f` takes an owned `A`. For `Lazy`, there are only two ways to get an owned `A` from `&A`:

1. **Clone it** (requires `A: Clone`, violates zero-cost abstraction principle).
2. **Consume the cell** (destroying the memoization, since other clones share the cell).

Neither is acceptable as an implicit operation in a library that prioritizes zero-cost abstractions. So `RefFunctor` honestly represents what memoized lazy types can do: `ref_map(f: impl FnOnce(&A) -> B, fa: F<A>) -> F<B>`.

This is the right call. The separation prevents a `Functor` implementation from silently cloning cached values. Users who want to clone must do so explicitly in their mapping function.

### Comparison to PureScript

PureScript's `Data.Lazy` implements a normal `Functor` because `force :: Lazy a -> a` returns an owned value (GC handles sharing). The Rust version cannot return owned values from shared cells without `Clone`, so the `RefFunctor` abstraction is a necessary Rust-specific adaptation.

```purescript
-- PureScript: force gives owned `a`, so Functor is trivial
instance functorLazy :: Functor Lazy where
  map f l = defer \_ -> f (force l)
```

```rust
// Rust: evaluate gives `&A`, so we need RefFunctor
fn ref_map<'a, A: 'a, B: 'a>(
    func: impl FnOnce(&A) -> B + 'a,
    fa: Lazy<'a, A, RcLazyConfig>,
) -> Lazy<'a, B, RcLazyConfig>
```

---

## 2. Method Signatures

### The trait method

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    func: impl FnOnce(&A) -> B + 'a,
    fa: Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
) -> Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>);
```

**Key design choices:**

- **`FnOnce` instead of `Fn`:** Correct. Since memoized types evaluate the closure at most once, `FnOnce` is sufficient. The documentation explains this well. Using `Fn` would unnecessarily constrain callers.

- **`'a` lifetime on the closure:** The closure is bounded by `'a`, the same lifetime as the container's contents. This is correct because the closure is captured inside a new `Lazy` cell, which itself lives for `'a`.

- **No `Clone` bounds on `A` or `B`:** Correct. The mapping function receives `&A` and produces an owned `B`, so neither needs to be `Clone`. This is an advantage over the `Foldable` implementation, which requires `A: Clone` because its callbacks take owned `A`.

- **No `Send` bounds:** Correct for the `!Send` variant (`RcLazy`). The `Send` variant is handled by the separate `SendRefFunctor` trait.

### Observation: `FnOnce` prevents reuse of the mapping function

Because `ref_map` takes `FnOnce`, if a user wants to apply the same mapping function to multiple `Lazy` values, they must either:
- Clone the closure manually.
- Use a reference to the function and wrap it in a new `FnOnce` closure each time.

This is a minor ergonomic trade-off for the flexibility of `FnOnce`. It is the right choice because the memoized cell only calls the closure once, and using `Fn` would force closures that capture non-cloneable state to be wrapped unnecessarily.

---

## 3. Relationship to Functor

### Could they be unified?

One approach would be to parameterize `Functor` over the "access mode" of the mapping function, e.g.:

```rust
// Hypothetical: Functor parameterized over access mode
trait Functor {
    fn map<'a, A: 'a, B: 'a>(
        f: impl FnOnce(/* &A or A */) -> B + 'a,
        fa: ...,
    ) -> ...;
}
```

This is not viable in Rust because the function signature difference (`&A -> B` vs `A -> B`) is fundamentally different at the type level. You cannot abstract over "reference-ness" of a function parameter without GATs or higher-order trait abstractions that Rust does not support.

### Could RefFunctor be a supertrait or subtrait of Functor?

- **`Functor: RefFunctor`** (every Functor is a RefFunctor): Technically feasible via a blanket impl `impl<F: Functor> RefFunctor for F` that clones inside `ref_map`. But this violates the zero-cost principle, and `map` takes `Fn` while `ref_map` takes `FnOnce`, so the signatures are not directly compatible.

- **`RefFunctor: Functor`** (every RefFunctor is a Functor): Not possible because `ref_map` cannot provide an owned `A` to `map`'s callback.

**Verdict:** Separation is necessary. These are genuinely different abstractions, not just variants of the same one.

---

## 4. HKT Integration

`RefFunctor` uses the same Brand/Kind pattern as `Functor`:

```rust
#[kind(type Of<'a, A: 'a>: 'a;)]
pub trait RefFunctor { ... }
```

The `#[kind(...)]` attribute generates a supertrait bound equivalent to `Kind!(type Of<'a, A: 'a>: 'a;)`, tying the Brand to its type constructor. The `Apply!` macro resolves `<Self as Kind>::Of<'a, A>` to the concrete type (e.g., `Lazy<'a, A, RcLazyConfig>`).

The impl_kind for `LazyBrand<Config>`:

```rust
impl_kind! {
    impl<Config: LazyConfig> for LazyBrand<Config> {
        type Of<'a, A: 'a>: 'a = Lazy<'a, A, Config>;
    }
}
```

This is clean. `LazyBrand<Config>` serves as the brand type, and `Kind::Of<'a, A>` resolves to `Lazy<'a, A, Config>`. Both `RefFunctor` and `SendRefFunctor` can be implemented for specific `Config` instantiations, which is exactly what happens: `RefFunctor for LazyBrand<RcLazyConfig>` and `SendRefFunctor for LazyBrand<ArcLazyConfig>`.

### Limitation: No generic RefFunctor for LazyBrand<Config>

There is no `impl<Config: LazyConfig> RefFunctor for LazyBrand<Config>` because `RcLazy::new` and `ArcLazy::new` have different closure bounds (`ArcLazy` requires `Send`). This forces separate impls for each config, which is handled by the `RefFunctor` / `SendRefFunctor` split.

---

## 5. Implementors

### Current implementors of RefFunctor

| Brand | Concrete type | Notes |
|-------|---------------|-------|
| `LazyBrand<RcLazyConfig>` | `RcLazy<'a, A>` | Primary use case. |
| `TryLazyBrand<E, RcLazyConfig>` | `RcTryLazy<'a, A, E>` | Requires `E: 'static + Clone`. Error is cloned on the `Err` path. |

### Current implementors of SendRefFunctor

| Brand | Concrete type | Notes |
|-------|---------------|-------|
| `LazyBrand<ArcLazyConfig>` | `ArcLazy<'a, A>` | Thread-safe counterpart. Requires `A: Send + Sync`, closure `Send`. |
| `TryLazyBrand<E, ArcLazyConfig>` | `ArcTryLazy<'a, A, E>` | Requires `E: 'static + Clone + Send + Sync`. |

### Does the implementor set make sense?

Yes. The implementors are exactly the memoized types in the hierarchy. Non-memoized types (`Thunk`, `SendThunk`, `Trampoline`, `Free`) consume their value on evaluation (they take `self`, not `&self`), so they implement `Functor` instead. The split is clean:

- **Memoized, shared access (`&A`):** `RefFunctor` / `SendRefFunctor`.
- **One-shot, owned access (`A`):** `Functor`.

### TryLazy's `E: Clone` requirement

The `TryLazyBrand<E, RcLazyConfig>: RefFunctor` impl requires `E: 'static + Clone`. This is because when the inner result is `Err(e)`, `ref_map` must propagate the error into a new `TryLazy` cell. Since `evaluate()` returns `Result<&A, &E>` (a borrow), the error must be cloned to move it into the new cell's closure. The `'static` bound comes from the `TryLazyBrand` struct itself (the error type is part of the brand, which is `'static`).

This is a known limitation but a reasonable one. Most error types are `Clone`.

---

## 6. Documentation Quality

### Strengths

- The trait-level documentation clearly explains why `RefFunctor` exists (the `&A` vs `A` distinction).
- The relationship to `SendRefFunctor` is documented directly on the trait, including why they are not in a subtype relationship.
- Laws are stated with examples.
- The "Cache chain behavior" section warns about memory accumulation from chaining `ref_map`.
- The "Why `FnOnce`?" section explains the closure choice.
- All methods have `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, and `#[document_returns]` attributes.
- Property-based tests (QuickCheck) verify both identity and composition laws.

### Weaknesses

- The identity law states `ref_map(|x| x.clone(), fa)` but the actual test uses `|v: &i32| *v` (dereference-copy, not `clone()`). For `i32` these are equivalent, but the law statement implies `Clone` is needed for identity, which is only true when `A` is not `Copy`. The law could be more precise.

- The composition law in the doc comment uses `ref_map(|x| g(&f(x)), fa)`, where `f` receives `&A` and `g` receives `&B`. But in the sequential form `ref_map(g, ref_map(f, fa))`, the inner `ref_map(f, fa)` produces `F<B>`, and the outer `ref_map(g, ...)` receives `&B`. So `f` is `&A -> B` and `g` is `&B -> C`. The composed form must therefore be `|x: &A| g(&f(x))`. This is correct as stated but could benefit from explicit type annotations in the doc to make the reference threading clearer.

- There are no doc examples showing the "cache chain" behavior concretely (e.g., a chain of `ref_map` calls demonstrating that all predecessors remain alive).

---

## 7. Issues, Limitations, and Design Flaws

### 7.1 RefFunctor and SendRefFunctor are disconnected

The documentation explicitly notes this: `SendRefFunctor` is not a subtrait of `RefFunctor`, so generic code written against `RefFunctor` cannot be used with `ArcLazy`. If you write a function `fn foo<F: RefFunctor>(...)`, it cannot accept `ArcLazy` values. Similarly, a function `fn bar<F: SendRefFunctor>(...)` cannot accept `RcLazy` values.

This means there is no way to write code that is generic over "any mappable lazy type." You must choose one or the other, or duplicate the function.

This is a fundamental tension caused by Rust's `Send` bound system. Making `SendRefFunctor: RefFunctor` would require `ArcLazy` to implement `RefFunctor`, which would mean `ref_map` could be called with a non-`Send` closure on a type that requires `Send` closures. The current design correctly avoids this unsoundness.

### 7.2 `ref_map` creates a new Lazy cell (not a view)

Each `ref_map` call allocates a new `Rc`/`Arc`-wrapped lazy cell. The documentation warns about this ("Cache chain behavior"), but it means chaining `ref_map` builds up a linked list of reference-counted allocations. This is different from, say, a `map` on `Vec` which produces a single new `Vec`.

This is inherent to the memoized lazy design, not a flaw in the trait itself.

### 7.3 No `Applicative` or `Monad` for memoized types

Because `RefFunctor` sits outside the `Functor -> Applicative -> Monad` hierarchy, memoized types cannot participate in the standard type class tower. You cannot `pure` into a `RefFunctor` context generically, or `bind`/`flatMap` over one.

The `Lazy` type does have inherent `pure` methods, and `Deferrable` provides a kind of `bind`-like operation (deferred flattening), but these are not expressed through the standard HKT type class hierarchy.

### 7.4 Inconsistency between RefFunctor and Foldable

`Foldable for LazyBrand<Config>` is implemented generically over all `Config: LazyConfig`, but `RefFunctor` is only implemented for `LazyBrand<RcLazyConfig>`. The `Foldable` implementation works generically because its callbacks receive `A` (owned, via `Clone`), not `&A`. This means `Foldable` imposes `A: Clone` while `RefFunctor` does not.

This is consistent with the design principles, but it means `Foldable` and `RefFunctor` cannot be composed in generic code (e.g., you cannot write a generic function that both folds and maps over a lazy type using a single trait bound).

### 7.5 The free function `ref_map` is not re-exported in `functions.rs`

Looking at `fp-library/src/functions.rs`, `ref_map` is auto-generated through the `generate_function_re_exports!` macro (since the macro scans `src/classes`). The integration test at `fp-library/tests/hkt_integration.rs` imports `ref_map` from `crate::classes::ref_functor::ref_map`, not from `functions::*`, which suggests the re-export is working (the module-level example imports from `functions::*` successfully). This is fine.

---

## 8. Alternatives

### 8.1 Functor with A: Clone bound

```rust
// Hypothetical: Functor impl that clones
impl Functor for LazyBrand<RcLazyConfig> {
    fn map<'a, A: Clone + 'a, B: 'a>(
        f: impl Fn(A) -> B + 'a,
        fa: Lazy<'a, A, RcLazyConfig>,
    ) -> Lazy<'a, B, RcLazyConfig> {
        RcLazy::new(move || f(fa.evaluate().clone()))
    }
}
```

**Problem:** `Functor::map` does not have a `Clone` bound on `A` in its trait definition. Adding `Clone` to the trait would break all other implementations. Adding it only to this impl is not possible because trait method bounds must match the trait definition.

This could work if `Functor` were redesigned with an additional `A: Clone` bound, but that would be too restrictive for types like `Option` and `Vec` where `map` does not need `Clone`.

### 8.2 A `Functor`-like trait with `FnOnce(&A) -> B`

This is essentially what `RefFunctor` already is. The question is whether it should be called `Functor` with a different "mode." But as analyzed in section 3, Rust's type system cannot abstract over reference-ness of parameters.

### 8.3 Comonad approach

In Haskell/PureScript, `Lazy` is a `Comonad` with `extract = force` and `extend f x = defer (\_ -> f x)`. The `extract` gives you the value, and `extend` gives you a derived lazy computation.

In Rust, `extract` would need to return an owned `A`, which requires `Clone` (same issue as `Functor`). You could define a `RefComonad` with `extract(&self) -> &A`, but this adds another custom trait without solving the core problem.

### 8.4 GAT-based approach

A more general approach using GATs could parameterize `Functor` over the "access pattern":

```rust
trait Functor {
    type Access<'b, A: 'b>;  // Either `A` or `&'b A`
    fn map<'a, A: 'a, B: 'a>(
        f: impl FnOnce(Self::Access<'_, A>) -> B + 'a,
        fa: ...,
    ) -> ...;
}
```

This is theoretically interesting but would massively complicate the trait hierarchy. The current `RefFunctor` approach is simpler and more direct.

### 8.5 Accept the status quo

The current design is pragmatic. `RefFunctor` is a niche trait used by exactly the types that need it (memoized lazy values). It has clear laws, good documentation, and the separation from `Functor` is well-justified. The main cost is that generic code cannot be polymorphic over both `Functor` and `RefFunctor` types, but this reflects a genuine semantic difference in what these types can do.

---

## Summary

`RefFunctor` is a well-designed, Rust-specific adaptation to the problem of mapping over shared, memoized values that can only be accessed by reference. The trait is clean, the laws are correct, the documentation is thorough, and the implementation set is exactly right.

The main design tensions are:

1. **Disconnection from `Functor`:** Unavoidable given Rust's ownership model.
2. **Disconnection between `RefFunctor` and `SendRefFunctor`:** Unavoidable given Rust's `Send` bound system. Well-documented.
3. **No participation in the type class tower** (no `Applicative`, `Monad`, etc.): A consequence of (1), and inherent to the approach.
4. **Cache chain memory accumulation:** Inherent to memoized lazy evaluation, well-documented.

No changes are recommended for the trait itself. The design is sound for its purpose.
