# `impl Trait` vs Named Generics

This document describes when to use `impl Trait` vs named generic type parameters for function arguments in `fp-library`, and the reasoning behind those choices.

## Background

In Rust, `impl Trait` in argument position is syntactic sugar for a named generic:

```rust
fn map<A, B>(f: impl Fn(A) -> B, fa: Option<A>) -> Option<B>
// desugars to:
fn map<A, B, Func: Fn(A) -> B>(f: Func, fa: Option<A>) -> Option<B>
```

Both are universally quantified (the caller chooses the concrete type). The difference is whether the type parameter is **named** and **visible** in the signature.

## Correspondence to PureScript/Haskell

PureScript's Functor:

```purescript
class Functor f where
  map :: forall a b. (a -> b) -> f a -> f b
```

Here `(a -> b)` is not a type variable. The `forall` quantifies over `a` and `b`, not over the function type. The function argument is simply "a function from `a` to `b`."

The `impl Fn(A) -> B` encoding mirrors this: the function parameter is not a separately quantified type dimension. The named `Func` generic introduces an extra type parameter (`forall a b func.`) that the PureScript/Haskell version does not have.

## Guidelines

### Prefer `impl Trait` when the type parameter is incidental

Use `impl Trait` when the type only appears once and the caller has no reason to name it:

```rust
// The closure type is an implementation detail
fn map<'a, A: 'a, B: 'a>(
    f: impl Fn(A) -> B + 'a,
    fa: Apply!(<Self as Kind!(...)>::Of<'a, A>),
) -> Apply!(<Self as Kind!(...)>::Of<'a, B>);
```

This applies to most function/closure parameters in the type class hierarchy: `map`, `bind`, `fold_map`, `traverse`, etc. The concrete closure type is an implementation detail that callers never need to reference.

Benefits:

- **Fewer type parameters** in turbofish: `map::<Brand, _, _>` vs `map::<Brand, _, _, _>`.
- **Matches the PureScript/Haskell convention** where function types are not type variables.
- **Simpler signatures** with less syntactic noise.

### Prefer named generics when the type parameter is structural

Use a named generic when the type serves a structural role in the signature.

#### The type appears in multiple positions

```rust
fn combine<T: Semigroup>(a: T, b: T) -> T
```

With `impl Trait`, each occurrence introduces an **independent** anonymous type parameter:

```rust
// BAD: a and b can be DIFFERENT types
fn combine(a: impl Semigroup, b: impl Semigroup) -> impl Semigroup
```

#### The return type depends on the input type

```rust
fn identity<T>(x: T) -> T       // caller gets the same type back
fn identity(x: impl Any) -> impl Any  // caller gets an opaque type
```

The named generic preserves the connection between input and output.

#### Bounds reference other type parameters

When a where clause relates the type to other parameters, it must be named:

```rust
fn apply<'a, A: 'a, B: 'a, F>(f: F, xs: Vec<A>) -> Vec<B>
where
    F: Fn(A) -> B + Clone + Send + 'a,
```

If the bounds are simple enough to fit inline, `impl Trait` still works:

```rust
fn map<'a, A: 'a, B: 'a>(f: impl Fn(A) -> B + 'a, fa: ...) -> ...
```

Use judgment based on readability.

#### Callers need turbofish for that specific parameter

If callers must specify the type explicitly, it must be named:

```rust
let x = default::<u32>();  // T must be nameable
```

## Summary

| Situation | Use | Reason |
|---|---|---|
| Closure/function passed once | `impl Fn(A) -> B` | Incidental type, matches PureScript convention |
| Same type in multiple positions | Named generic | `impl Trait` creates independent types |
| Input type = return type | Named generic | Preserves type identity |
| Complex where clause relating types | Named generic | Must be nameable for cross-referencing |
| Caller needs turbofish | Named generic | `impl Trait` params can't be specified |
| Type appears once, simple bounds | `impl Trait` | Less noise, fewer type parameters |

## Application in `fp-library`

Most type class methods take function arguments where the concrete closure type is incidental. These should use `impl Trait`:

- `Functor::map(f: impl Fn(A) -> B, ...)`
- `Monad::bind(fa: ..., f: impl Fn(A) -> Self::Of<B>)`
- `Foldable::fold_map(f: impl Fn(A) -> M, ...)`
- `Traversable::traverse(f: impl Fn(A) -> G::Of<B>, ...)`

Named generics are appropriate when the function type needs additional bounds that benefit from a where clause (e.g., `+ Clone + Send + Sync` in `ParFoldable`), or when the type parameter participates in relationships with other parameters.
