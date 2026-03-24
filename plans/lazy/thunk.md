# Thunk Analysis

File: `fp-library/src/types/thunk.rs`

## Overview

`Thunk<'a, A>` is a non-memoizing, deferred computation wrapper built on `Box<dyn FnOnce() -> A + 'a>`. It occupies the "lightweight glue" niche in the lazy evaluation hierarchy: it supports HKT traits (Functor, Monad, MonadRec, Foldable) and lifetime polymorphism, at the cost of not being stack-safe for deep `bind` chains. It sits between `Lazy` (memoized, no Monad) and `Trampoline` (stack-safe, `'static` only).

## 1. Design

### Separation from Trampoline and Lazy

The three-way split is well-motivated:

- **Thunk**: HKT-compatible, lifetime-polymorphic, single-use (`FnOnce`). Suitable for composition and glue code where deep recursion is not a concern.
- **Trampoline**: Stack-safe via `Free<ThunkBrand, A>` with CatList-based continuations. Requires `'static` due to `Box<dyn Any>` type erasure. Cannot implement the library's HKT traits.
- **Lazy**: Memoized via `LazyCell`/`LazyLock`. Implements `RefFunctor` (returns references) but not `Functor` (which requires owned values). Clonable; supports shared caching.

The design correctly identifies that Rust's ownership and lifetime system makes it impossible to have all three properties (HKT, stack safety, memoization) in a single type. Each type picks a different subset.

### Structural design

The newtype `Thunk<'a, A>(Box<dyn FnOnce() -> A + 'a>)` is the simplest possible representation. The `Box` allocation is necessary because `dyn FnOnce` is unsized. This is a reasonable trade-off: the single heap allocation per Thunk is the minimal cost for type-erased closures.

The choice of `FnOnce` rather than `Fn` is correct for a single-use deferred computation. Since `evaluate` takes `self` by value, each Thunk can only be evaluated once, which is consistent with `FnOnce` semantics.

### Relationship to PureScript's Lazy

PureScript's `Lazy` is a memoizing type that implements `Functor`, `Apply`, `Bind`, `Monad`, `Extend`, `Comonad`, `Foldable`, and `Traversable`. Rust's `Thunk` corresponds to the non-memoizing subset of this functionality. The key difference is that PureScript's `defer` memoizes the result (the JS FFI backing ensures at-most-once evaluation), while Rust's `Thunk::defer` does not memoize.

PureScript's `Lazy` also implements `Comonad` (with `extract = force`). Rust's `Thunk` cannot implement `Comonad` because `evaluate` consumes `self`, and `Comonad::extract` would need `&self` semantics. This is a fundamental consequence of the ownership model, not a design flaw.

## 2. Implementation Correctness

### Functor

```rust
fn map<'a, A: 'a, B: 'a>(
    func: impl Fn(A) -> B + 'a,
    fa: Thunk<'a, A>,
) -> Thunk<'a, B> {
    fa.map(func)
}
```

Delegates to the inherent `map`, which creates `Thunk::new(move || f((self.0)()))`. Correct: evaluates the inner thunk, applies the function, returns the result.

The inherent `map` accepts `FnOnce` for flexibility; the HKT version requires `Fn` per the trait contract. Since `Fn: FnOnce`, delegation works correctly.

### Semimonad (bind)

The inherent `bind`:
```rust
pub fn bind<B: 'a>(self, f: impl FnOnce(A) -> Thunk<'a, B> + 'a) -> Thunk<'a, B> {
    Thunk::new(move || {
        let a = (self.0)();
        let thunk_b = f(a);
        (thunk_b.0)()
    })
}
```

Correct: evaluates `self`, passes result to `f`, evaluates the resulting thunk. The entire chain is deferred into a single closure. Each `bind` nests one more closure, which adds a stack frame on evaluation.

The HKT `Semimonad::bind` delegates to the inherent method. Since `Fn: FnOnce`, the delegation is sound.

### Semiapplicative (apply)

```rust
fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
    ff: Thunk<'a, CloneableFn::Of<'a, A, B>>,
    fa: Thunk<'a, A>,
) -> Thunk<'a, B> {
    ff.bind(move |f| fa.map(move |a| f(a)))
}
```

Correct for a single-element container: evaluates the function thunk, evaluates the value thunk, applies the function. The `Clone` bound on `A` and the `CloneableFn` requirement come from the trait signature (designed for multi-element containers like `Vec`) and are unnecessarily strict for `Thunk` but harmless.

### Lift (lift2)

```rust
fn lift2<'a, A, B, C>(
    func: impl Fn(A, B) -> C + 'a,
    fa: Thunk<'a, A>,
    fb: Thunk<'a, B>,
) -> Thunk<'a, C>
where A: Clone + 'a, B: Clone + 'a, C: 'a {
    fa.bind(move |a| fb.map(move |b| func(a, b)))
}
```

Correct. `fa.bind(...)` calls the inherent bind (accepting `FnOnce`), so `fb` can be moved into the closure and consumed by `map`. The `Clone` bounds come from the trait and are unnecessary for Thunk but harmless.

### Foldable

```rust
fn fold_right<...>(func: impl Fn(A, B) -> B + 'a, initial: B, fa: Thunk<'a, A>) -> B {
    func(fa.evaluate(), initial)
}
```

Correct. For a single-element container, right fold simply applies the function to the element and the initial value. Left fold and fold_map follow the same pattern.

### Semigroup / Monoid

```rust
fn append(a: Self, b: Self) -> Self {
    Thunk::new(move || Semigroup::append(a.evaluate(), b.evaluate()))
}
fn empty() -> Self {
    Thunk::new(|| Monoid::empty())
}
```

Correct. Defers the combination until evaluation. Both values are consumed (moved into the closure).

### Conversions

- `From<Lazy<'a, A, Config>> for Thunk<'a, A>`: Requires `A: Clone` because `Lazy::evaluate` returns `&A`. Correct.
- `From<Trampoline<A>> for Thunk<'static, A>`: Correctly constrains to `'static` since `Trampoline` requires it. Evaluates the trampoline when the thunk is forced.

### memoize / memoize_arc

- `memoize(self) -> Lazy<'a, A, RcLazyConfig>`: Converts via `Lazy::from(self)`. Correct.
- `memoize_arc(self) -> Lazy<'a, A, ArcLazyConfig>`: Evaluates eagerly because `Thunk`'s inner closure is not `Send`. The result is stored in an `ArcLazy`. This is the correct approach; there is no way to lazily evaluate a non-Send thunk on a potentially different thread.

## 3. Stack Safety

### `tail_rec_m` Implementation

```rust
fn tail_rec_m<'a, A: 'a, B: 'a>(
    f: impl Fn(A) -> Thunk<'a, Step<A, B>> + Clone + 'a,
    a: A,
) -> Thunk<'a, B> {
    Thunk::new(move || {
        let mut current = a;
        loop {
            match f(current).evaluate() {
                Step::Loop(next) => current = next,
                Step::Done(res) => break res,
            }
        }
    })
}
```

This is genuinely stack-safe. The loop runs in a single stack frame, evaluating each step and either continuing (with mutation) or breaking. Each `f(current)` creates a `Thunk<Step<A, B>>`, which is immediately evaluated. Since `f` returns a thunk that wraps a single closure (not a chain of binds), evaluating it adds only one extra stack frame at most.

### The "partial stack safety" claim is accurate

The documentation correctly states:
- `tail_rec_m` is stack-safe (uses a loop).
- `bind` chains are NOT stack-safe (each bind nests a closure).

A chain of `n` binds will require `O(n)` stack frames when evaluated. For most practical uses this is fine (thousands of binds would be needed to overflow), but it is a real limitation for algorithms that build long monadic chains programmatically.

### Subtle detail: `f` must return shallow thunks

The stack safety of `tail_rec_m` depends on `f` returning "shallow" thunks (direct `Thunk::pure(Step::Loop(...))` or `Thunk::pure(Step::Done(...))`). If `f` itself builds deep bind chains inside the returned thunk, those chains could still overflow when `evaluate()` is called in the loop. The documentation does not explicitly call this out, though the examples correctly show shallow thunks.

## 4. Consistency

### With library conventions

- Follows the Brand pattern correctly via `impl_kind!`.
- Implements the expected trait hierarchy: `Functor`, `Pointed`, `Lift`, `Semiapplicative`, `Semimonad`, `MonadRec`, `Foldable`, `Evaluable`, `Deferrable`.
- Provides both inherent methods (accepting `FnOnce`) and HKT trait implementations (accepting `Fn`), consistent with the library's pattern of offering ergonomic inherent methods alongside generic trait dispatch.
- Documentation follows the `#[document_signature]` / `#[document_parameters]` / `#[document_examples]` convention.
- Uses hard tabs for indentation per `rustfmt.toml`.

### Missing compared to PureScript's Lazy

PureScript's `Lazy` implements these type classes that Thunk does not:
- **Traversable**: Documented as impossible due to `FnOnce` + lack of `Clone`. Correct assessment.
- **Comonad/Extend**: Cannot work because `evaluate` consumes `self`. Correct trade-off.
- **Eq, Ord, Show (with evaluated display)**: `Debug` is implemented but shows `"Thunk(<unevaluated>)"` rather than forcing evaluation. This is the right choice for a non-memoizing type (forcing would be a side effect and the value cannot be recovered).

### With Trampoline

The API surface is well-aligned: both types provide `new`, `pure`, `defer`, `bind`, `map`, `evaluate`, `memoize`, `memoize_arc`. Trampoline additionally has `lift2`, `then`, `append`, `empty` as inherent methods; Thunk provides these through trait implementations (Lift, ApplyFirst/ApplySecond, Semigroup, Monoid).

## 5. Limitations

### No Clone

`Thunk` cannot implement `Clone` because `Box<dyn FnOnce()>` is not clonable. This is fundamental and correctly identified in the docs. It prevents:
- `Traversable` implementation.
- Using the same thunk in multiple places without first evaluating it.

### Not Send/Sync

`Thunk` uses `Box<dyn FnOnce() -> A + 'a>`, which is not `Send` or `Sync` by default (trait objects are `!Send` unless explicitly bounded). A `SendThunk` variant could be useful for concurrent scenarios, analogous to how `RcLazy`/`ArcLazy` handle the single-threaded/thread-safe split.

### Eager evaluation in some conversions

`memoize_arc` evaluates eagerly. This is documented and unavoidable without a `Send` closure, but it means the "deferred" semantics are lost. Users who want a truly lazy, thread-safe computation would need to construct an `ArcLazy` directly.

### No FunctorWithIndex or FoldableWithIndex

PureScript's `Lazy` implements `FunctorWithIndex Unit` and `FoldableWithIndex Unit`. These could be trivially added for `Thunk` (the index is always `()`). This is a minor omission.

### The `Clone` bounds in trait methods are overly strict

`Lift::lift2`, `Semiapplicative::apply`, and `Foldable` methods all require `A: Clone` (or similar) because they are designed for multi-element containers. For `Thunk` (a single-element container), these bounds are unnecessary. This is not a bug but a consequence of the trait design; it means users must add `Clone` bounds even when logically unnecessary.

## 6. Alternatives

### Enum-based representation

Instead of `Box<dyn FnOnce() -> A>`, one could use an enum:
```rust
enum Thunk<'a, A> {
    Value(A),
    Deferred(Box<dyn FnOnce() -> A + 'a>),
}
```
This would avoid a heap allocation for `pure` values and enable a non-consuming `evaluate` for the `Value` case (via borrowing). However, it adds complexity and a branch on every operation. The current design is simpler and consistent with the "zero-cost abstraction" philosophy, where the `Box` is the only cost.

### Interior mutability for memoization

One could make `Thunk` memoizing using `OnceCell`, but that would duplicate `Lazy`'s purpose. The separation is well-motivated.

### A `SendThunk` variant

A `SendThunk<'a, A>` wrapping `Box<dyn FnOnce() -> A + Send + 'a>` would enable thread-safe deferred computation chains. This could follow the same pattern as `RcLazy`/`ArcLazy` and would enable `memoize_arc` to be truly lazy rather than eager. This seems like a worthwhile addition.

### Stack safety via explicit CPS

Instead of nesting closures in `bind`, one could accumulate continuations in a `Vec` or `CatList`, similar to what `Free` does for `Trampoline`. This would make all `bind` chains stack-safe at the cost of heap allocation per bind step. However, this would essentially recreate `Trampoline`, so the current design correctly delegates deep recursion to that type.

## 7. Documentation

### Strengths

- The module-level doc comment is concise and immediately tells users when to use `Thunk` vs alternatives.
- The comparison table (`Thunk` vs `Trampoline`) is excellent and covers the key trade-offs.
- The "Algebraic Properties" section correctly states the monad laws.
- The "Limitations" section honestly documents the `Traversable` incompatibility.
- The `bind` vs `Semimonad::bind` distinction (`FnOnce` vs `Fn`) is clearly explained.

### Issues

- **"Partial stack safety" phrasing in the comparison table**: The table says "Partial (tail_rec_m only)" which is accurate but could be clearer. It means "stack-safe only when using `tail_rec_m`; not stack-safe for `bind` chains." Someone skimming might misread "partial" as "sometimes works, sometimes doesn't" rather than "works for a specific subset of operations."

- **Missing note about shallow thunks in `tail_rec_m`**: The `tail_rec_m` documentation does not warn that the step function `f` should return shallow thunks (not deep bind chains). If `f` returns a thunk built from many nested binds, the `evaluate()` call inside the loop could still overflow.

- **The `Deferrable` implementation's doc example** uses `Deferrable::defer(|| Thunk::pure(42))` which requires a fully qualified call. A note about when to use `Deferrable::defer` vs the inherent `Thunk::defer` would help.

- **Minor**: Some doc comments refer to "eval" (e.g., `test_eval_from_memo`, `test_eval_semigroup`, the `Evaluable` doc says "Runs the eval"). This appears to be a remnant from a previous naming where `Thunk` was called `Eval`. The test names and some doc strings should be updated for consistency.

### Test coverage

The test suite is solid:
- Basic operations (new, pure, borrowing, map, bind, defer).
- Conversions (From<Lazy>, From<Trampoline>).
- Algebraic instances (Semigroup, Monoid).
- QuickCheck property tests for Functor laws, Monad laws, Semigroup associativity, and Monoid identity.

Missing tests:
- `tail_rec_m` with a non-trivial recursion depth (the doc example uses 1000 but there is no dedicated test).
- `Foldable` operations via the HKT interface.
- `Lift::lift2` and `Semiapplicative::apply` via the HKT interface.
- `memoize` and `memoize_arc`.
- `Evaluable::evaluate` via the HKT interface.

## Summary of Findings

**The implementation is correct and well-designed.** The type occupies a clear niche in the lazy evaluation hierarchy, the trait implementations follow the monad laws, and the documentation is thorough.

Key items to consider:
1. The "eval" naming remnants in tests and some docs should be cleaned up.
2. A `SendThunk` variant would be a useful addition for thread-safe scenarios.
3. The `tail_rec_m` documentation should note that the step function must return shallow thunks.
4. `FunctorWithIndex` and `FoldableWithIndex` are trivially implementable and would improve PureScript parity.
5. Test coverage for HKT-level trait operations (Foldable, Lift, Apply, Evaluable) and `tail_rec_m` could be expanded.
