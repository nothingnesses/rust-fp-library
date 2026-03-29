# Deferrable Trait Analysis

## Overview

`Deferrable<'a>` is the Rust equivalent of PureScript's `Lazy` type class. It provides a single operation, `defer`, which constructs a value from a thunk (`FnOnce() -> Self`). The companion trait `SendDeferrable<'a>` extends it with a `Send`-bounded variant.

**File:** `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/deferrable.rs`

## Trait Definition

```rust
pub trait Deferrable<'a> {
    fn defer(f: impl FnOnce() -> Self + 'a) -> Self
    where
        Self: Sized;
}
```

Free function wrapper:

```rust
pub fn defer<'a, D: Deferrable<'a>>(f: impl FnOnce() -> D + 'a) -> D
```

## Comparison with PureScript's Lazy

PureScript defines:

```purescript
class Lazy l where
  defer :: (Unit -> l) -> l

fix :: forall l. Lazy l => (l -> l) -> l
fix f = go where go = defer \_ -> f go
```

### Key differences

1. **`Unit -> l` vs `FnOnce() -> Self`.** PureScript uses `Unit -> l` (a function from Unit) as its thunking mechanism. Rust uses `FnOnce() -> Self`, which is semantically identical but idiomatic for Rust. Good translation.

2. **Lifetime parameter `'a`.** PureScript's `Lazy` has no lifetime parameter because PureScript is garbage-collected. The Rust version adds `'a` to the trait, which constrains both the thunk closure (`+ 'a`) and allows implementors to tie the lifetime to their own internal storage. This is necessary and correct.

3. **No `fix`.** PureScript's `fix` enables lazy self-reference ("tying the knot"). The Rust trait correctly omits this because `fix` requires shared ownership and interior mutability, which not all Deferrable types have. Thunks are consumed on evaluation, making self-reference impossible. The documentation explicitly explains this and points to the concrete `rc_lazy_fix` and `arc_lazy_fix` functions.

4. **No blanket instance for functions.** PureScript provides `instance lazyFn :: Lazy (a -> b)` which makes all function types lazy by eta-expansion. The Rust version does not provide a blanket impl for `Fn` types. This is reasonable since Rust functions are already lazy (they only execute when called), so the blanket instance would be trivial and potentially confusing.

5. **No `Unit` instance.** PureScript provides `instance lazyUnit :: Lazy Unit`. There is no corresponding `impl Deferrable<'_> for ()` in the Rust code. This could be added trivially (`fn defer(f) -> () { f() }`) but its utility is minimal.

## Implementors Survey

| Type | Lifetime | Behavior | Notes |
|------|----------|----------|-------|
| `Thunk<'a, A>` | `'a` | Truly deferred: wraps `f` in a new thunk | Delegates to inherent `Thunk::defer` |
| `SendThunk<'a, A>` | `'a` | **Eager**: calls `f()` immediately | Cannot store non-`Send` closure |
| `Trampoline<A>` | `'static` | Truly deferred via `Trampoline::defer` | Restricted to `'static` |
| `RcLazy<'a, A>` | `'a` | Truly deferred: wraps in `RcLazy::new` | Requires `A: Clone` |
| `ArcLazy<'a, A>` | `'a` | **Eager**: calls `f()` immediately | Cannot store non-`Send` closure |
| `TryThunk<'a, A, E>` | `'a` | Truly deferred: delegates to `TryThunk::defer` | |
| `TrySendThunk<'a, A, E>` | `'a` | **Eager**: calls `f()` immediately | Cannot store non-`Send` closure |
| `TryTrampoline<A, E>` | `'static` | Truly deferred via inner `Trampoline::defer` | Restricted to `'static` |
| `RcTryLazy<'a, A, E>` | `'a` | Truly deferred: wraps in new `TryLazy` | Requires `A: Clone, E: Clone` |
| `ArcTryLazy<'a, A, E>` | `'a` | **Eager**: calls `f()` immediately | Cannot store non-`Send` closure |
| `Free<ThunkBrand, A>` | `'static` | Truly deferred via `Free::wrap` + `Thunk::new` | Restricted to `'static` |

### The eager evaluation problem

Three implementors (`SendThunk`, `ArcLazy`, `TrySendThunk`, `ArcTryLazy`) execute the thunk eagerly in their `Deferrable::defer` implementation. The reason is that `Deferrable::defer` does not require `Send` on the closure, but these types need `Send` closures internally. They cannot store the non-`Send` closure, so they evaluate it immediately and wrap the resulting value.

This is documented in the trait-level `# Warning` section. The transparency law (`defer(|| x) == x`) is technically preserved because the *value* produced is the same, but the *deferred evaluation semantics* are lost. From the caller's perspective, side effects in `f` happen at `defer` time rather than at evaluation time.

**Severity:** Medium. This is a fundamental tension between the trait's single-method design and Rust's ownership/Send constraints. The current design handles it pragmatically. The `SendDeferrable` trait exists as the correct alternative for callers who need guaranteed deferred evaluation with `Send` types.

**Possible alternative:** The trait could have been designed without `Deferrable` implementations for Send types at all, with only `SendDeferrable` serving them. However, the current approach allows generic code written against `Deferrable` to accept both single-threaded and thread-safe types, which is valuable for library ergonomics.

## Method Signature Analysis

### `fn defer(f: impl FnOnce() -> Self + 'a) -> Self where Self: Sized`

**`FnOnce` is correct.** A deferred computation only needs to be called once. Using `Fn` or `FnMut` would over-constrain callers.

**`+ 'a` on the closure is correct.** It ties the closure's lifetime to the trait's lifetime parameter, ensuring the closure does not outlive its captured references.

**`Self: Sized` is necessary.** The function returns `Self` by value, which requires `Sized`. This prevents implementing `Deferrable` for trait objects, which is appropriate since deferred construction of dynamically-sized types is not meaningful in this context.

**Static dispatch via `impl FnOnce`.** This matches the library's design philosophy of uncurried semantics with `impl Fn` for zero-cost abstractions. No boxing occurs at the trait boundary; implementors decide whether and how to box.

**The signature is a constructor (associated function), not a method.** There is no `self` parameter. This is correct for `Deferrable` since the operation creates a new value rather than transforming an existing one. It does mean the trait cannot be used as a trait object (no `dyn Deferrable`), which is acceptable.

## Free Function Analysis

```rust
pub fn defer<'a, D: Deferrable<'a>>(f: impl FnOnce() -> D + 'a) -> D {
    D::defer(f)
}
```

The free function is a straightforward delegation. It enables the common `let x: Thunk<i32> = defer(|| ...)` call pattern, which is more ergonomic than `Deferrable::defer(|| ...)` or `Thunk::defer(|| ...)`.

The return type `D` must be inferable from context (either via type annotation on the binding or from a surrounding function signature). This is standard for Rust type inference with associated-function-style traits.

## Relationship with Evaluable

`Deferrable` and `Evaluable` are conceptual duals:
- `Deferrable::defer: (() -> Self) -> Self` (injects a computation)
- `Evaluable::evaluate: F<A> -> A` (extracts a value)

However, they are not formally linked. `Evaluable` is an HKT trait on brands (`Evaluable: Functor`), while `Deferrable` is a value-level trait on concrete types. This asymmetry exists because:
- `Evaluable` needs HKT to express the natural transformation `F ~> Id`.
- `Deferrable` operates on concrete types (`Self`), not parameterized type constructors.

The asymmetry means you cannot write a single generic function that both defers and evaluates over an abstract type while preserving HKT polymorphism. This is a design limitation, but a natural one given Rust's type system.

## Documentation Quality

### Strengths

- The trait-level documentation is thorough, including the transparency law, the explanation of why `fix` is omitted, and the warning about eager evaluation.
- Cross-references to `SendDeferrable`, `rc_lazy_fix`, and `arc_lazy_fix` are provided.
- Doc examples are included on the trait, the method, and the free function.
- Property-based tests verify the transparency law and a nesting law.

### Weaknesses

- The examples all use `Thunk`, which is the simplest implementor. A supplementary example showing `RcLazy` or `Trampoline` would illustrate the broader applicability.
- The warning about eager evaluation could link more explicitly to the specific types that evaluate eagerly (currently it only mentions `ArcLazy` by name).
- The nesting law in the tests (`defer(|| defer(|| x)) == defer(|| x)`) is not documented in the trait-level laws section. It follows from transparency, so it is a derived property rather than an independent law, but stating it explicitly could help users.

## Design Issues and Considerations

### 1. No connection to HKT / Brand system

`Deferrable` is a value-level trait (`trait Deferrable<'a>` on concrete types), not an HKT brand-level trait. This means:
- You cannot write `F: Functor + Deferrable` as a bound.
- Generic HKT code cannot abstract over "functors that support deferred construction."

This is a deliberate design choice. Making `Deferrable` brand-level would require it to work with `Kind::Of<'a, A>`, but `defer` constructs a value from nothing (not from a `Kind::Of<'a, A>`), so the HKT encoding would be awkward.

**Verdict:** The current value-level design is the right call. The trait is primarily used for constructing individual values, not for abstracting across type constructors.

### 2. The trait is used almost exclusively for ad-hoc dispatch

Searching the codebase, `Deferrable` is used as a trait bound only in:
- The `defer` free function.
- The `SendDeferrable` supertrait.

No other generic function or trait in the library uses `Deferrable` as a bound. This means the trait's primary role is providing a uniform API (`defer(|| ...)`) across lazy types, rather than enabling generic programming. This is not necessarily a problem; a uniform vocabulary is valuable even without heavy generic usage.

### 3. Extra bounds on Lazy implementations

`RcLazy`'s `Deferrable` impl requires `A: Clone`, and `ArcLazy`'s requires `A: Send + Sync`. The `Clone` requirement comes from `Lazy`'s memoization semantics (it returns `&A` from `evaluate`, and `defer` flattens via `.evaluate().clone()`). The `Send + Sync` requirement comes from `Arc`/`LazyLock`.

These extra bounds mean `Deferrable` is not uniformly available for all `Lazy` values. A `Lazy<'a, NonCloneType, RcLazyConfig>` cannot be deferred. This is an inherent tension between the memoized types and the trait's generality.

### 4. The `Self: Sized` bound

The `where Self: Sized` bound on `defer` prevents implementing `Deferrable` for unsized types. Since all current implementors are sized, this is fine in practice. However, it does prevent the trait from being object-safe, which means you cannot have `Box<dyn Deferrable<'a>>`. This is appropriate since `defer` is a constructor, not a method.

### 5. Asymmetric `Clone` requirements between `Deferrable` and `SendDeferrable` for `ArcLazy`

`ArcLazy`'s `Deferrable` impl requires `A: Send + Sync` (no `Clone`), while its `SendDeferrable` impl requires `A: Clone + Send + Sync`. The difference is that:
- `Deferrable::defer` for `ArcLazy` evaluates eagerly (just calls `f()`), so no flattening is needed.
- `SendDeferrable::send_defer` for `ArcLazy` truly defers (wraps in `ArcLazy::new`), which requires flattening via `.evaluate().clone()`.

This asymmetry is correct but subtle. Users might be surprised that `defer` on `ArcLazy` has weaker bounds than `send_defer`, given that `send_defer` is the "better" (truly deferred) option.

## Alternatives Considered

### Alternative 1: Split into `Deferrable` and `SendDeferrable` only (no eager impls)

Remove the `Deferrable` implementations for `SendThunk`, `ArcLazy`, `TrySendThunk`, and `ArcTryLazy`. Only provide `SendDeferrable` for those types.

**Pros:** Eliminates the confusing eager-evaluation behavior. `Deferrable` would always mean truly deferred.
**Cons:** Generic code written against `Deferrable` would not accept `SendThunk` or `ArcLazy`, reducing composability.

**Verdict:** The current approach is better for library ergonomics. The eager-evaluation warning is sufficient.

### Alternative 2: Add a `DeferrableBrand` for HKT integration

Create a brand-level trait that connects `Deferrable` to the Kind system.

**Pros:** Would enable `F: Functor + DeferrableBrand` bounds.
**Cons:** Awkward encoding since `defer` is a constructor, not a natural transformation. Would require `Kind::Of<'a, A>: Deferrable<'a>` bounds, adding complexity.

**Verdict:** Not worth the complexity. The trait's current usage does not demand HKT integration.

### Alternative 3: Make `defer` take `&self` (factory pattern)

Instead of a constructor, make `defer` an instance method on a "factory" type.

**Cons:** Completely changes the semantics. The PureScript model (and the current design) treats `Deferrable` as a property of the type itself, not of a factory instance.

**Verdict:** Not appropriate for this library's design philosophy.

## Test Coverage

The module includes two QuickCheck property tests:
1. **Transparency:** `evaluate(defer(|| pure(x))) == x` (for `Thunk`).
2. **Nesting:** `evaluate(defer(|| defer(|| pure(x)))) == evaluate(defer(|| pure(x)))` (for `Thunk`).

These tests only cover `Thunk`. The eager-evaluation behavior of `SendThunk`, `ArcLazy`, etc. is tested implicitly through those types' own test suites, but there are no property tests that specifically verify the transparency law across all implementors.

## Summary

The `Deferrable` trait is a clean, well-documented translation of PureScript's `Lazy` class. The key Rust-specific adaptations (lifetime parameter, `FnOnce`, omission of `fix`, `Send`-aware companion trait) are all well-motivated. The main design tension is the eager evaluation of `Send`-requiring types, which is documented and mitigated by `SendDeferrable`. The trait sees limited use as a generic bound in the codebase, serving primarily as a uniform API rather than an abstraction point for polymorphic code.
