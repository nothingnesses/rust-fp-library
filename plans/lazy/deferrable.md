# Analysis: `Deferrable` trait (`fp-library/src/classes/deferrable.rs`)

## Overview

`Deferrable` is a type class for types that can be constructed lazily from a computation. It provides a single method `defer(f: impl FnOnce() -> Self + 'a) -> Self` that wraps a thunk-producing closure. A companion free function `defer` dispatches to the trait method.

## 1. Design

### Comparison to PureScript's `Control.Lazy`

PureScript's `Lazy` class has two members:

```purescript
class Lazy l where
  defer :: (Unit -> l) -> l

fix :: forall l. Lazy l => (l -> l) -> l
fix f = go where go = defer \_ -> f go
```

The Rust `Deferrable` trait faithfully translates the `defer` method. The key differences:

- **Naming:** Renamed from `Lazy` to `Deferrable` to avoid collision with the concrete `Lazy` type. This is a good choice; it avoids ambiguity and reads naturally as an adjective describing a capability.
- **`Unit -> l` becomes `FnOnce() -> Self`:** Idiomatic Rust translation. PureScript uses `Unit -> l` (not a bare `l`) specifically to introduce laziness; in Rust, `FnOnce() -> Self` achieves the same effect naturally.
- **`fix` is deliberately omitted from the trait.** The documentation explains why: `fix` requires shared ownership and interior mutability (Rc/Arc + OnceCell/OnceLock), which are properties of `Lazy` specifically, not of all deferrable types. `Thunk` is consumed on evaluation, making self-referential construction impossible. This is a well-reasoned departure from PureScript. The concrete `rc_lazy_fix` and `arc_lazy_fix` functions fill this role for the types that can support it.
- **No `Lazy (a -> b)` instance equivalent.** PureScript provides `instance lazyFn :: Lazy (a -> b) where defer f = \x -> f unit x`. This makes sense in PureScript where functions are pervasively lazy, but would be unnatural in Rust where closures have concrete capture semantics and no uniform representation. Omitting it is appropriate.
- **Lifetime parameter `'a`.** The trait is parameterized by `'a`, which is necessary in Rust to express that the thunk may borrow data. This is a fundamental adaptation for Rust's ownership model.

### Trait-on-concrete-type vs. HKT brand pattern

Unlike `Functor`, `Applicative`, etc., `Deferrable` is implemented directly on concrete types (`Thunk<'a, A>`, `Lazy<'a, A, Config>`, etc.) rather than on brand types. This is correct because `Deferrable` is not a higher-kinded type class; it does not abstract over a type constructor `F[_]`. In PureScript, `Lazy l` is also kind `Type -> Constraint`, not `(Type -> Type) -> Constraint`. The library correctly identifies this distinction.

### Relationship with `SendDeferrable`

`SendDeferrable` is a parallel trait requiring `Send + Sync` on the thunk. Notably:

- `SendDeferrable` does **not** extend `Deferrable` as a supertrait.
- `ArcLazy` implements only `SendDeferrable`, not `Deferrable`.
- `RcLazy` implements only `Deferrable`, not `SendDeferrable`.

This clean separation mirrors the library's broader Rc/Arc split pattern (e.g., `RefCountedPointer` vs `SendRefCountedPointer`, `RefFunctor` vs `SendRefFunctor`). It is consistent, but has a trade-off: generic code that works with "any deferrable type" cannot be written once to cover both; you need two versions or a different abstraction. This is a known limitation of the library's approach to thread safety.

## 2. Implementation

### Correctness

The trait definition and free function are correct. There are no bugs.

- `fn defer(f: impl FnOnce() -> Self + 'a) -> Self where Self: Sized` correctly requires `Sized` (necessary because `Self` appears by value in both the closure return type and the method return type).
- The free function `defer<'a, D: Deferrable<'a>>(f: impl FnOnce() -> D + 'a) -> D` is a straightforward delegation.

### Implementations reviewed

All implementations follow a consistent pattern of delegating to an inherent `defer` method on the concrete type:

| Type | Lifetime | Notes |
|------|----------|-------|
| `Thunk<'a, A>` | `'a` | Flattens via `Thunk::new(move \|\| f().evaluate())`. |
| `Trampoline<A>` | `'static` | Required by Trampoline's design. |
| `TryTrampoline<A, E>` | `'static` | Same constraint. |
| `Lazy<'a, A, RcLazyConfig>` | `'a` | Requires `A: Clone`. Flattens via `RcLazy::new(move \|\| f().evaluate().clone())`. |
| `TryLazy<'a, A, E, RcLazyConfig>` | `'a` | Requires `A: Clone, E: Clone`. |
| `Free<ThunkBrand, A>` | `'static` | Only for Thunk-based Free. |
| `TryThunk<'a, A, E>` | `'a` | Fallible thunk variant. |

The `Lazy` implementation is worth noting: it forces and clones the inner value during deferral. This means `defer(|| lazy_value)` is not zero-cost; it eagerly evaluates and clones when the outer lazy is forced. This is inherent to `Lazy`'s memoization design (it stores `&A`, so it must own the value).

### Subtle point: double evaluation

For `Lazy`, `defer(f)` creates `RcLazy::new(move || f().evaluate().clone())`. When this outer lazy is forced, it:
1. Calls `f()` to get the inner `Lazy`.
2. Evaluates the inner `Lazy` (calling its thunk).
3. Clones the result.

This means the inner `Lazy`'s memoization is effectively lost; the inner lazy is created and immediately evaluated, then discarded. This is correct behavior (transparency law holds), but users might expect memoization of intermediate results to be preserved. The documentation on the `Lazy` impl mentions "this flattens the nested structure," which is accurate.

## 3. Consistency

### With library patterns

- **Module structure:** Follows the `mod inner { ... } pub use inner::*;` pattern with `#[fp_macros::document_module]`, consistent with all other class files.
- **Documentation macros:** Uses `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]` consistently.
- **Free function:** Provides a free function version alongside the trait method, re-exported via `functions.rs`, following the standard pattern.
- **Naming:** The `Deferrable` naming convention (adjective form) is consistent with other non-HKT traits like `Evaluable`.

### Minor inconsistency

The module doc example uses `brands::*` import:
```rust
use fp_library::{
    brands::*,
    functions::*,
    types::*,
};
```
But `brands::*` is not actually needed for this example (no brand types are used). The `Thunk` and `defer` function come from `types::*` and `functions::*` respectively. This appears in the module-level doc, the trait's `defer` method doc, and the free function doc. The extra import is harmless but slightly misleading.

## 4. Limitations

### Inherent limitations

- **No `fix` combinator at the trait level.** As documented, this is a fundamental limitation of Rust's ownership model. The concrete `rc_lazy_fix`/`arc_lazy_fix` functions are the correct workaround. This is well-explained in the trait documentation.
- **No HKT abstraction.** You cannot write `defer::<F, A>(...)` parameterized over a brand `F`. This is fine because `Deferrable` is inherently a `Type -> Constraint` class, not `(Type -> Type) -> Constraint`.
- **`Sized` bound.** The `where Self: Sized` constraint prevents implementing `Deferrable` for unsized types. This is acceptable; all current implementors are sized.

### Addressable limitations

- **`SendDeferrable` is completely separate from `Deferrable`.** It would be possible to make `SendDeferrable: Deferrable` a supertrait, since any `Send + Sync` thunk is also a valid non-Send thunk. However, this would require `ArcLazy` to implement `Deferrable` as well, which may introduce confusion about which variant to use. The current design trades some generality for clarity. This matches the library's established pattern for the Rc/Arc split, so changing it here alone would create inconsistency.
- **Clone requirement for Lazy.** `Deferrable` for `Lazy` requires `A: Clone`, which comes from `Lazy`'s memoization design. This is inherent to the type, not the trait.

## 5. Alternatives

### Could `Deferrable` be HKT-based?

One might consider making `Deferrable` a brand-level trait:
```rust
pub trait Deferrable: Kind {
    fn defer<'a, A: 'a>(f: impl FnOnce() -> Apply!(Self::Of<'a, A>) + 'a) -> Apply!(Self::Of<'a, A>);
}
```
This would allow writing generic code over `F: Deferrable`, but it creates problems:
- `Lazy`'s `defer` needs `A: Clone`, which cannot be expressed uniformly.
- `Trampoline` does not have a brand (as noted in the `Evaluable` docs).
- The extra abstraction provides little value since `defer` is rarely used in generic contexts (confirmed by grep: it appears only in the free function definition).

The current concrete-type design is the right call.

### Could `fix` be a separate trait?

A `Fixable` trait could be introduced:
```rust
pub trait Fixable<'a>: Deferrable<'a> + Clone {
    fn fix(f: impl Fn(Self) -> Self + 'a) -> Self;
}
```
This would formalize the `fix` capability. However, since only `Lazy` (in two variants) supports it, and the implementation requires Rc/Arc-specific machinery, a trait adds ceremony without enabling meaningful generic programming. The concrete functions are sufficient.

## 6. Documentation

### Strengths

- The "Why there is no generic `fix`" section is excellent. It proactively addresses the obvious question from anyone familiar with PureScript's `Lazy` class, and the explanation is technically precise.
- Cross-references to `rc_lazy_fix`, `arc_lazy_fix`, `Lazy`, `Thunk` are all present.
- The transparency law is clearly stated.
- Doc examples compile and test (they use assertion macros).

### Issues

- **Unnecessary `brands::*` import in examples.** Three doc examples import `brands::*` but do not use any brand types. This could confuse readers into thinking a brand is needed for `defer`.
- **The transparency law could be stronger.** The stated law is "defer(|| x) is observationally equivalent to x when evaluated." This is correct but informal. A more precise formulation might distinguish between:
  - For consumed types (Thunk): `defer(|| x).evaluate() == x.evaluate()`.
  - For memoized types (Lazy): `*defer(|| x).evaluate() == *x.evaluate()`.
  The current formulation is adequate for a library doc, but the "observationally equivalent" phrasing is doing some heavy lifting.
- **Missing period on type parameter docs.** `#[document_type_parameters("The lifetime of the computation.")]` on the trait is fine, but `#[document_type_parameters("The lifetime of the computation", "The type of the deferred value.")]` on the free function is missing a period after "computation". (The first entry lacks the trailing period.) This is a minor formatting inconsistency.

## Summary

`Deferrable` is a clean, well-motivated translation of PureScript's `Lazy` class to Rust. The decision to omit `fix` from the trait and provide it as concrete functions on `Lazy` is well-reasoned and well-documented. The trait follows all library conventions consistently. The only actionable items are minor: removing unnecessary `brands::*` imports from doc examples and adding a missing period in a type parameter description on the free function.
