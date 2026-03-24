# RefFunctor Analysis

## Overview

`RefFunctor` (`fp-library/src/classes/ref_functor.rs`) is a type class for types that can be mapped over where the mapping function receives a reference (`&A`) rather than an owned value (`A`). It exists primarily to serve `Lazy`, which returns `&A` from `evaluate()` rather than `A`.

Signature:

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    func: impl FnOnce(&A) -> B + 'a,
    fa: Lazy<'a, A, Config>,
) -> Lazy<'a, B, Config>;
```

Compared to `Functor::map`:

```rust
fn map<'a, A: 'a, B: 'a>(
    f: impl Fn(A) -> B + 'a,
    fa: F<A>,
) -> F<B>;
```

## 1. Design

### Why Lazy needs RefFunctor instead of regular Functor

The core issue is that `Lazy::evaluate()` returns `&A`, not `A`. Rust's `LazyCell`/`LazyLock` memoize a value and hand out borrows to it. Unlike PureScript's `force :: Lazy a -> a` (which returns an owned value, since everything is GC'd), Rust's ownership model means the cached value lives inside the cell, and you only get a reference.

This means a regular `Functor` impl would need to clone the inner value to pass it by value to `f: impl Fn(A) -> B`. That has two problems:
- It would require `A: Clone`, adding a trait bound not present in `Functor`'s signature.
- It would be semantically misleading: `Functor::map` implies zero-copy transformation, but you'd silently be cloning.

`RefFunctor` solves this cleanly: the mapping function gets `&A`, acknowledging the reference semantics.

**Verdict:** The design is well-motivated and appropriate for Rust. This is a genuine case where Rust's ownership model requires a different abstraction than PureScript/Haskell use.

### PureScript comparison

PureScript's `Lazy` implements the standard `Functor`:

```purescript
instance functorLazy :: Functor Lazy where
  map f l = defer \_ -> f (force l)
```

This works because `force` returns an owned value (GC handles memory). The Rust `RefFunctor` is the honest translation of this pattern into a language with ownership, where `evaluate` returns a borrow.

### FnOnce vs Fn

`RefFunctor::ref_map` uses `FnOnce`, while `Functor::map` uses `Fn`. This is a deliberate and correct choice. `Lazy` is a single-element container, so the mapping function is called at most once. `Functor::map` uses `Fn` because types like `Vec` call the function multiple times. Using `FnOnce` for `RefFunctor` is more permissive for callers (accepts move closures) since Lazy only has one value to map over.

## 2. Implementation

### Correctness

The implementation is correct. The `ref_map` on `RcLazy` works by:

```rust
pub fn ref_map<B: 'a>(self, f: impl FnOnce(&A) -> B + 'a) -> Lazy<'a, B, RcLazyConfig> {
    let init: Box<dyn FnOnce() -> B + 'a> = Box::new(move || f(self.evaluate()));
    Lazy(RcLazyConfig::lazy_new(init))
}
```

This captures `self` (the original `Lazy`) by move into the new thunk. When the new `Lazy` is evaluated, it forces the original, gets `&A`, and passes it to `f`. The original `Lazy` is kept alive inside the closure, which is correct since `Rc`-based lazy values are reference-counted.

### Potential issue: chain amplification

Each `ref_map` creates a new `Lazy` that holds a reference (via `Rc` clone inside `self`) to the previous one. A long chain of `ref_map` calls creates a linked list of `Lazy` values, each holding the previous. This is not a bug, but it means:
- Memory is not freed until the entire chain is dropped.
- Evaluation traverses the full chain on first access.

This is the same behavior as PureScript's `map f l = defer \_ -> f (force l)` and is expected.

### No bugs found

The trait definition, free function, and implementation all appear correct. Lifetime bounds are properly threaded.

## 3. Consistency

### Structural consistency

The file follows the same pattern as `functor.rs` and other type class files:
- `#[fp_macros::document_module]` wrapper.
- Inner `mod inner` with `pub use inner::*`.
- Trait definition with `#[kind(...)]` attribute.
- Free function wrapper.
- Documentation macros (`document_signature`, `document_type_parameters`, etc.).

This is fully consistent with the rest of the library.

### Naming consistency

The `ref_` prefix convention is clear and consistent across `RefFunctor`, `SendRefFunctor`, `ref_map`, and `send_ref_map`.

### Hierarchy gap

`RefFunctor` does not inherit from or relate to `Functor` in the trait hierarchy. This is correct, since a `RefFunctor` is not a `Functor` (different function signatures). However, there is no blanket impl like "every `Functor` where `A: Clone` is also a `RefFunctor`," which could be useful for types like `Option` or `Vec`. This is likely intentional since `RefFunctor` currently only serves `Lazy`.

## 4. Limitations

### Single implementor

`RefFunctor` is only implemented for `LazyBrand<RcLazyConfig>`. `LazyBrand<ArcLazyConfig>` does not implement `RefFunctor`; it implements `SendRefFunctor` instead. This means:
- Generic code written against `RefFunctor` cannot accept `ArcLazy`.
- There is no trait that abstracts over both `RcLazy` and `ArcLazy` mapping.

This is noted in the source (line 679 of lazy.rs): `ArcLazy` cannot implement `RefFunctor` because `RefFunctor` does not require `Send` on the function, but `ArcLazy::new` requires it.

**Possible fix:** `SendRefFunctor` could have a supertrait bound on `RefFunctor`, with a blanket impl. But this would require `ArcLazy` to accept non-`Send` functions in `RefFunctor`, which is impossible. The current split is the correct approach.

### No higher abstractions

Because `Lazy` uses `RefFunctor` instead of `Functor`, it cannot participate in abstractions built on `Functor` (like `Applicative`, `Monad`, `Traversable`). In PureScript, `Lazy` implements the full hierarchy (`Functor`, `Apply`, `Applicative`, `Bind`, `Monad`, `Foldable`, `Traversable`, `Extend`, `Comonad`). In Rust, `Lazy` is limited to:
- `RefFunctor` / `SendRefFunctor`
- `Foldable` (which clones the inner value)
- `Semigroup` / `Monoid`
- `Deferrable` / `SendDeferrable`

This is an inherent limitation of Rust's ownership model combined with the library's zero-cost design principle (no implicit cloning).

### Could Lazy implement Functor with A: Clone?

Technically yes, but the `Functor` trait signature does not have `A: Clone` bounds, so you cannot add them in an impl. You would need a separate `CloneFunctor` or similar, which would fragment the hierarchy further. The `RefFunctor` approach is cleaner.

## 5. Alternatives

### Alternative 1: Functor with Clone bound via a wrapper

One could define a `CloneLazy<'a, A: Clone>` newtype that implements `Functor` by cloning on map. This would let `Lazy` participate in the standard hierarchy for types that are `Clone`. Downside: it requires wrapping/unwrapping and the `Clone` bound leaks into generic code.

### Alternative 2: Comonadic approach

In PureScript, `Lazy` is a `Comonad` with `extract = force`. In Rust, `Comonad::extract` would need to return an owned value, requiring `Clone`. A `RefComonad` trait (with `extract` returning `&A`) could complement `RefFunctor`, giving `Lazy` an `extend`/`extract` pair. This could be useful but adds complexity for limited gain.

### Alternative 3: Make RefFunctor a subtrait or related trait of Functor

One could define `RefFunctor` as providing a default impl of `Functor` (where `A: Clone`). However, Rust's trait system does not support conditional blanket impls well (specialization is unstable), so this is not practical today.

### Recommendation

The current design is the right choice. `RefFunctor` honestly represents what `Lazy` can do without implicit cloning. The limitation to `Foldable` (which does clone) and `RefFunctor` (which does not) is a fair reflection of the ownership tradeoffs.

## 6. Documentation

### Accuracy

The documentation is accurate. The laws are correctly stated:
- Identity: `ref_map(|x| x.clone(), fa)` produces an equal value. (Note: this uses `clone` in the identity law, which is correct since you go from `&A -> A` and the result is a new `Lazy<A>`.)
- Composition: Correctly stated.

### Completeness

The documentation includes:
- Module-level example.
- Trait-level explanation of why `RefFunctor` exists (references to `Lazy`).
- Law statements with runnable examples.
- Method-level docs with full signature/parameter/return documentation.
- Free function docs.

### Suggestions

1. The trait doc could more explicitly state the relationship to `Functor`, e.g., "This trait exists because some types (like `Lazy`) return references from their accessor, making the standard `Functor` trait inapplicable without requiring `Clone`."
2. The identity law says `ref_map(|x| x.clone(), fa)` "evaluates to a value equal to `fa`'s evaluated value," but the example uses `|x: &i32| *x` (dereference copy, not `clone`). This is fine for `i32` but the law statement and example could be more aligned. For non-`Copy` types, the law would need `clone()`.
3. The `document_type_parameters` on the trait lists three parameters ("The lifetime," "The type of the value(s)," "The type of the result(s)"), but the trait itself has only the kind-level parameter, not explicit generic params at the trait level. The type parameters are on the method `ref_map`, not the trait. This is consistent with how `Functor` does it, so it is a library-wide pattern rather than a `RefFunctor`-specific issue.

## Summary

| Aspect | Assessment |
|--------|-----------|
| Design motivation | Strong. Correctly addresses Rust's ownership constraints for memoized types. |
| Correctness | No bugs found. Lifetime threading and implementation are sound. |
| Consistency | Fully consistent with library patterns and conventions. |
| Limitations | Inherent to Rust's ownership model. Cannot participate in the standard Functor hierarchy. Only one implementor. |
| Alternatives | Current approach is the best available option. |
| Documentation | Good. Minor suggestions for clarity on the Functor relationship and identity law alignment. |
