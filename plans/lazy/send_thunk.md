# SendThunk Analysis

File: `fp-library/src/types/send_thunk.rs`

## Overview

`SendThunk<'a, A>` wraps `Box<dyn FnOnce() -> A + Send + 'a>`, providing a thread-safe, non-memoized deferred computation. It is the `Send` counterpart to `Thunk<'a, A>`, which wraps `Box<dyn FnOnce() -> A + 'a>` without a `Send` bound. The type registers a brand (`SendThunkBrand`) via `impl_kind!`, implements `Deferrable`, `SendDeferrable`, `Semigroup`, `Monoid`, and `Debug`, and provides inherent methods `new`, `pure`, `defer`, `bind`, `map`, `evaluate`, and `memoize_arc`.

## Design Assessment

### Overall verdict

The design is sound and well-motivated. `SendThunk` fills a clear niche: lightweight deferred computation that can cross thread boundaries. The implementation is clean, the documentation is thorough, and the trait implementations are correct.

### Why HKT traits are not implemented (justified)

The `Functor` trait signature is:

```rust
fn map<'a, A: 'a, B: 'a>(
    f: impl Fn(A) -> B + 'a,
    fa: ...,
) -> ...;
```

The mapping function `f` has no `Send` bound. If `Functor` were implemented for `SendThunkBrand`, the resulting `SendThunk` would contain a composed closure capturing a non-`Send` `f`, breaking the `Send` invariant. The same reasoning applies to `Semimonad::bind`, `Semiapplicative::apply`, and `Pointed::pure` (though `pure` would actually work since `A: Send` could be required by the `Kind` mapping).

This is a fundamental limitation of the HKT encoding, not a design flaw. The library correctly documents this and provides inherent methods as the alternative. There is no practical workaround short of duplicating the entire type class hierarchy with `Send` bounds, which would be unreasonable.

### Relationship with Thunk

**Well designed.** The two types are structurally parallel: same methods, same semantics, only differing in the `Send` bound. Key observations:

1. **`From<Thunk> for SendThunk`**: Correctly requires `A: Send` and eagerly evaluates the `Thunk` (since its closure is not `Send`). This is the only sound approach.

2. **No `From<SendThunk> for Thunk`**: This conversion would be trivially safe (every `Send` closure is also a valid non-`Send` closure), but it is not implemented. This is a minor omission; adding it would improve ergonomics.

3. **Method parity**: `Thunk` has `memoize()` (returns `RcLazy`) and `memoize_arc()` (eagerly evaluates, returns `ArcLazy`). `SendThunk` has only `memoize_arc()` (lazily wraps, returns `ArcLazy`). This asymmetry is correct: `SendThunk` cannot produce an `RcLazy` without evaluation because `Rc` is not `Send`, and the lazy `memoize_arc` is the key advantage of `SendThunk` over `Thunk`.

### SendDeferrable implementation (sound)

The `SendDeferrable` impl wraps the `Send + Sync` closure into a `SendThunk` that calls `f().evaluate()` on demand. This is lazy (deferred) and correct.

The `Deferrable` impl eagerly evaluates via `f()` because `Deferrable::defer` does not require `Send` on the closure, so the closure cannot be stored inside `SendThunk`. This is the correct and only sound approach, documented clearly.

### Semigroup / Monoid implementations (correct)

Both are straightforward: `append` defers the combination, `empty` defers the identity. Both correctly require `A: Send` to maintain the `Send` invariant.

## Issues and Suggestions

### 1. Missing `From<SendThunk> for Thunk`

A `SendThunk<'a, A>` should be convertible to `Thunk<'a, A>` cheaply (zero-cost, just widening the trait object). This is safe because `Box<dyn FnOnce() -> A + Send + 'a>` coerces to `Box<dyn FnOnce() -> A + 'a>`. The conversion does not exist today.

### 2. No `ap` / `zip_with` / `apply` methods

`Thunk` has HKT-level `Semiapplicative`, so users get `apply` and derived combinators for free. `SendThunk` has no equivalent inherent methods. For practical use (combining multiple `SendThunk` values), users must chain `bind` calls, which is less ergonomic.

Consider adding:

```rust
pub fn zip_with<B: 'a, C: 'a>(
    self,
    other: SendThunk<'a, B>,
    f: impl FnOnce(A, B) -> C + Send + 'a,
) -> SendThunk<'a, C> {
    SendThunk::new(move || {
        let a = self.evaluate();
        let b = other.evaluate();
        f(a, b)
    })
}
```

### 3. No `Evaluable` implementation

`Evaluable` requires `Functor` as a supertrait, so `SendThunkBrand` cannot implement it. This is an inherent limitation of the trait hierarchy, not a bug. However, it means `SendThunk` cannot be used as the base functor for `Free`. This is documented in the architecture table (the "No" in the HKT column) but not explicitly called out in the `SendThunk` docs.

### 4. `Deferrable` implementation eagerly evaluates

The `Deferrable::defer` impl for `SendThunk` calls `f()` immediately, which means `defer(|| expensive())` is NOT deferred at all. This satisfies the `Deferrable` transparency law (`defer(|| x)` is observationally equivalent to `x`), but it may surprise users who expect laziness. The doc comment on the impl clearly says "called eagerly," which is good, but there is a deeper question: should `SendThunk` implement `Deferrable` at all if it cannot provide actual deferral?

The argument for keeping it: generic code written against `Deferrable` can still accept `SendThunk`, and the transparency law holds. The argument against: the whole point of `Deferrable` is lazy construction, and this impl does not provide it.

On balance, keeping the impl is the right call because it enables `SendDeferrable: Deferrable` to be implemented, which is the real goal. But it is worth noting in the docs that `send_defer` (not `defer`) is the method that provides true laziness for `SendThunk`.

### 5. `memoize_arc` could accept `self` more precisely

Currently:

```rust
pub fn memoize_arc(self) -> ArcLazy<'a, A> {
    Lazy::<'a, A, ArcLazyConfig>::new(move || self.evaluate())
}
```

This wraps the `SendThunk` in a new closure. Since `SendThunk` is itself `Send`, and `Lazy::new` presumably accepts `impl FnOnce() -> A + Send`, this should work by directly passing the inner closure. The current approach adds one unnecessary level of indirection (calling `evaluate` which calls `(self.0)()`). A more direct approach would be:

```rust
pub fn memoize_arc(self) -> ArcLazy<'a, A> {
    Lazy::<'a, A, ArcLazyConfig>::new(self.0)
}
```

This would avoid the double closure overhead, though the optimizer likely eliminates it regardless.

### 6. No `into_inner` or similar escape hatch

There is no way to extract the inner `Box<dyn FnOnce() -> A + Send + 'a>` without evaluating it. While this is fine for most uses, it prevents composition with APIs that want a raw `FnOnce`. A low-priority consideration.

### 7. Missing tests for `Send` across actual threads

The test `test_send_thunk_is_send` only checks the trait bound at compile time. There is no test that actually sends a `SendThunk` to another thread and evaluates it there. A test using `std::thread::spawn` or a scoped thread would increase confidence. For example:

```rust
#[test]
fn test_send_thunk_across_threads() {
    let thunk = SendThunk::new(|| 42);
    let handle = std::thread::spawn(move || thunk.evaluate());
    assert_eq!(handle.join().unwrap(), 42);
}
```

### 8. `pure` requires `A: Send` but `new` does not enforce `A: Send` on the return type

`SendThunk::new` only requires the closure to be `Send`, not `A` itself. This is correct: the value `A` is produced inside the closure and consumed by the caller, never stored in a shared context. `pure` correctly adds `A: Send` because it captures `a` by move into the closure, and the closure must be `Send`. This distinction is sound.

### 9. Documentation quality

The documentation is thorough and accurate:

- The module-level doc explains the purpose and HKT limitation.
- The struct-level doc explains the design, limitations, and relationship with `Thunk`.
- Each method has signature docs, parameter docs, return docs, and examples.
- The `Deferrable` impl explicitly notes eager evaluation.

One minor point: the doc mentions "Stack Safety" but does not suggest `Trampoline` as a stack-safe alternative for `SendThunk`. It references it for `Thunk`, but `Trampoline` is not `Send` either, so the only stack-safe option for cross-thread use is unclear. The docs could note that there is no `Send`-capable stack-safe lazy type in the hierarchy.

## Summary

`SendThunk` is a well-designed, correctly implemented type that fills its intended role cleanly. The decision not to implement HKT traits is the only sound choice given the trait signatures. The `Deferrable`/`SendDeferrable` split is handled correctly, with appropriate documentation about the eager evaluation trade-off.

The main areas for improvement are:

- **Missing `From<SendThunk> for Thunk`** (easy win for ergonomics).
- **Missing `zip_with` or similar combinators** (important for practical use without HKT).
- **No cross-thread integration test** (increases confidence in the core promise).
- **Minor closure indirection in `memoize_arc`** (micro-optimization, likely eliminated by the compiler).
