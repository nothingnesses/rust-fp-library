# MonadRec Trait Analysis

## Overview

`MonadRec` is a type class for monads that support stack-safe tail recursion, defined in
`/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/monad_rec.rs`.
It extends `Monad` (which is `Applicative + Semimonad`) with a single method, `tail_rec_m`,
that performs iterative monadic computation using the `Step<A, B>` control type.

**Core signature:**

```rust
pub trait MonadRec: Monad {
    fn tail_rec_m<'a, A: 'a, B: 'a>(
        func: impl Fn(A) -> Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, Step<A, B>>) + 'a,
        initial: A,
    ) -> Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>);
}
```

## 1. Trait Design: Comparison with PureScript/Haskell

### PureScript reference

```purescript
class Monad m <= MonadRec m where
  tailRecM :: forall a b. (a -> m (Step a b)) -> a -> m b
```

### Assessment

The Rust `MonadRec` faithfully mirrors the PureScript design. The key structural elements are identical:

- **Superclass:** `Monad` (PureScript: `Monad m`).
- **Method:** `tail_rec_m` corresponds to `tailRecM`.
- **Step type:** `Step<A, B>` with `Loop(A)` and `Done(B)` corresponds to PureScript's `Step a b` with `Loop a` and `Done b`.
- **Semantics:** The step function `A -> M<Step<A, B>>` is called repeatedly; `Loop(a)` feeds `a` back in, `Done(b)` terminates with `b`.

**Naming:** The library uses `Loop`/`Done` rather than Haskell's `Left`/`Right` (via `Either`), which is clearer and more self-documenting. This matches PureScript's naming.

**Correctness verdict:** The trait correctly captures the tail-recursive monad abstraction.

## 2. Method Signature Analysis

### `tail_rec_m` signature

```rust
fn tail_rec_m<'a, A: 'a, B: 'a>(
    func: impl Fn(A) -> Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, Step<A, B>>) + 'a,
    initial: A,
) -> Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>);
```

**Observations:**

1. **Lifetime parameterization (`'a`):** The method is generic over `'a`, allowing implementations that borrow data (e.g., `Thunk<'a, A>`). This is more flexible than PureScript, which has no lifetime concept. The `'a` bound on `func` means the step function can capture references with that lifetime.

2. **`impl Fn(A)` vs `impl FnOnce(A)`:** The step function is `Fn`, not `FnOnce`. This is correct because the function may be called multiple times (once per `Loop` iteration). This matches PureScript's semantics.

3. **`+ 'a` on `func`:** The step function must live at least as long as `'a`. This is needed because some implementations (e.g., `Thunk`) capture the closure in a boxed thunk with lifetime `'a`.

4. **Macro-expanded types:** The `Apply!` and `Kind!` macros handle HKT type application. After expansion, the return type resolves to the concrete applied type (e.g., `Thunk<'a, B>`, `Option<B>`).

**Potential issue: `A` and `B` bounds.** Both `A: 'a` and `B: 'a` are required, which is correct for the general case (the values must outlive the computation). However, this means that for `'static` computations (like `Trampoline`), users must use `'static` types. Since `Trampoline` cannot implement this trait anyway (due to HKT limitations), this is not a practical problem.

**Design decision: no `Clone` on `func`.** The trait-level signature does not require `Clone` on the step function. However, `Trampoline::tail_rec_m` (an inherent method, not a trait impl) does require `Clone + 'static`. This discrepancy is because the trait is for HKT-compatible types, and `Trampoline` cannot implement the trait, so its inherent method has different bounds.

## 3. Relationship to Step Type

### Step type (`/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/step.rs`)

```rust
pub enum Step<A, B> {
    Loop(A),  // Continue with new state
    Done(B),  // Terminate with result
}
```

### HKT representations

Step has three brand representations:
- `StepBrand`: Bifunctor over both type parameters.
- `StepLoopAppliedBrand<LoopType>`: Fixed loop type, functor over done type. Monad that short-circuits on `Loop`.
- `StepDoneAppliedBrand<DoneType>`: Fixed done type, functor over loop type. Monad that short-circuits on `Done`.

Both applied brands implement `MonadRec`, creating a nested-Step pattern:
- `StepLoopAppliedBrand`: The step function returns `Step<LoopType, Step<A, B>>`, where `Loop(l)` short-circuits and `Done(Step::Loop(a))` continues.
- `StepDoneAppliedBrand`: The step function returns `Step<Step<A, B>, DoneType>`, where `Done(d)` short-circuits and `Loop(Step::Loop(a))` continues.

### Ergonomics

The API is ergonomic for the common case. Users construct `Step::Loop(a)` and `Step::Done(b)` directly, which is clear and readable:

```rust
tail_rec_m::<ThunkBrand, _, _>(
    |n| {
        if n < 10 { Thunk::pure(Step::Loop(n + 1)) }
        else { Thunk::pure(Step::Done(n)) }
    },
    0,
)
```

The nested-Step pattern for `StepLoopAppliedBrand`/`StepDoneAppliedBrand` is less ergonomic (`Step::Done(Step::Loop(n + 1))`), but this is inherent to using Step-as-a-monad and unlikely to be a common user-facing pattern.

### Rich API on Step

Step provides a comprehensive set of methods:
- Inspection: `is_loop`, `is_done`, `loop_val`, `done`.
- Transformation: `map_loop`, `map_done`, `bimap`, `swap`.
- Folding: `fold_right`, `fold_left`, `fold_map`, `bi_fold_right`, `bi_fold_left`, `bi_fold_map`.
- Binding: `bind` (over Done), `bind_loop` (over Loop).
- Traversal: `bi_traverse`.
- Conversions: `From<ControlFlow>` / `Into<ControlFlow>` (bidirectional).

The `ControlFlow` conversions are a nice touch, mapping `Step::Done` to `ControlFlow::Break` and `Step::Loop` to `ControlFlow::Continue`. This lets users interoperate with Rust's standard library.

## 4. Stack Safety Guarantees

### Does the design guarantee stack safety?

The trait documentation states a **class invariant**:

> `tail_rec_m` must execute in constant stack space regardless of how many `Step::Loop` iterations occur.

This is a structural requirement on implementations, not an algebraic law that can be checked at compile time. Each implementor must ensure its `tail_rec_m` uses a loop (or equivalent iterative mechanism) rather than recursion.

### Implementation patterns

The implementations fall into three categories:

**Category 1: Direct loop (truly stack-safe).** These implementations use a Rust `loop` with mutable state, consuming zero additional stack frames per iteration:
- `OptionBrand`: Loop, return `None` on short-circuit.
- `IdentityBrand`: Loop on inner value.
- `Tuple1Brand`: Loop on inner value.
- `ResultErrAppliedBrand<E>`: Loop, return `Err(e)` on short-circuit.
- `ResultOkAppliedBrand<E>`: Loop, return `Ok(t)` on short-circuit.
- `StepLoopAppliedBrand<L>`: Loop, return `Step::Loop(l)` on short-circuit.
- `StepDoneAppliedBrand<D>`: Loop, return `Step::Done(d)` on short-circuit.
- `VecBrand`: Breadth-first expansion loop.
- `CatListBrand`: Breadth-first expansion loop.
- `PairFirstAppliedBrand<F>`: Loop with monoid accumulation on first element.
- `PairSecondAppliedBrand<S>`: Loop with monoid accumulation on second element.
- `Tuple2FirstAppliedBrand<F>`: Loop with monoid accumulation.
- `Tuple2SecondAppliedBrand<S>`: Loop with monoid accumulation.

**Category 2: Lazy wrapper around a loop (stack-safe when evaluated).** These implementations wrap a loop in a thunk, deferring execution:
- `ThunkBrand`: Wraps the loop in `Thunk::new(move || { loop { ... } })`. The loop itself is stack-safe, but the thunk's `evaluate` call evaluates the step function at each iteration. The docs warn: "The step function `f` should return shallow thunks (ideally `Thunk::pure` or a single-level `Thunk::new`). If `f` builds deep `bind` chains inside the returned thunk, the internal `evaluate` call can still overflow the stack."
- `TryThunkErrAppliedBrand<E>`: Same pattern, loop in `TryThunk::new`.
- `TryThunkOkAppliedBrand<A>`: Same pattern, loop in `TryThunk::new`.

**Category 3: Non-trait inherent methods (not MonadRec impls but equivalent).** These types cannot implement the HKT-based `MonadRec` trait but provide equivalent inherent methods:
- `Trampoline::tail_rec_m`: Uses `Free`'s `defer` + `bind` chain, which is fully stack-safe even with deep bind chains. Requires `Clone + 'static` on the step function.
- `Trampoline::arc_tail_rec_m`: Wraps non-Clone closures in `Arc`.
- `SendThunk::tail_rec_m`: Loop inside `SendThunk::new`, requires `Send + Clone`.
- `SendThunk::arc_tail_rec_m`: Arc-wrapped variant.
- `TryTrampoline::tail_rec_m`: Same as Trampoline but for fallible computations.
- `TryTrampoline::arc_tail_rec_m`: Arc-wrapped variant.
- `TrySendThunk::tail_rec_m`: Loop inside `TrySendThunk::new`, requires `Send + Clone`.
- `TrySendThunk::arc_tail_rec_m`: Arc-wrapped variant.

### Stack safety caveats

1. **Thunk's conditional safety:** `ThunkBrand`'s `tail_rec_m` is stack-safe for the iteration loop, but if the step function itself builds deep `bind` chains within each returned thunk, those chains will consume stack during `evaluate`. This is documented but could surprise users.

2. **No compile-time enforcement:** There is no way to enforce the stack-safety invariant at the type level. A buggy implementation could use recursion and violate the invariant. The tests (200,000 iteration stress tests) provide runtime confidence.

3. **Trampoline is fully safe:** `Trampoline::tail_rec_m` is the only implementation that provides *unconditional* stack safety, because it builds a `Free` monad chain that is evaluated iteratively by `Free::evaluate`. Even deep bind chains within the step function are safe.

## 5. Implementors

### Trait implementors (via `MonadRec` trait)

| Brand | Type | Short-circuit behavior |
|-------|------|------------------------|
| `OptionBrand` | `Option<B>` | `None` terminates |
| `IdentityBrand` | `Identity<B>` | No short-circuit |
| `Tuple1Brand` | `(B,)` | No short-circuit |
| `ThunkBrand` | `Thunk<'a, B>` | No short-circuit (lazy) |
| `VecBrand` | `Vec<B>` | Breadth-first nondeterminism |
| `CatListBrand` | `CatList<B>` | Breadth-first nondeterminism |
| `ResultErrAppliedBrand<E>` | `Result<B, E>` | `Err(e)` terminates |
| `ResultOkAppliedBrand<E>` | `Result<E, B>` | `Ok(t)` terminates |
| `StepLoopAppliedBrand<L>` | `Step<L, B>` | `Loop(l)` terminates |
| `StepDoneAppliedBrand<D>` | `Step<B, D>` | `Done(d)` terminates |
| `PairFirstAppliedBrand<F>` | `Pair<F, B>` | Accumulates first via `Monoid` |
| `PairSecondAppliedBrand<S>` | `Pair<B, S>` | Accumulates second via `Monoid` |
| `Tuple2FirstAppliedBrand<F>` | `(F, B)` | Accumulates first via `Monoid` |
| `Tuple2SecondAppliedBrand<S>` | `(B, S)` | Accumulates second via `Monoid` |
| `TryThunkErrAppliedBrand<E>` | `TryThunk<'a, B, E>` | `Err(e)` terminates (lazy) |
| `TryThunkOkAppliedBrand<A>` | `TryThunk<'a, A, B>` | `Ok(a)` terminates (lazy) |

### Inherent `tail_rec_m` methods (not trait impls)

| Type | Requires | Notes |
|------|----------|-------|
| `Trampoline<A>` | `Clone + 'static` on `f` | Fully stack-safe, uses `Free` |
| `SendThunk<'a, A>` | `Clone + Send + 'a` on `f` | Thread-safe, loop-based |
| `TryTrampoline<A, E>` | `Clone + 'static` on `f` | Fallible, fully stack-safe |
| `TrySendThunk<'a, A, E>` | `Clone + Send + 'a` on `f` | Fallible, thread-safe, loop-based |

All four also provide `arc_tail_rec_m` variants for non-Clone closures.

### Coverage assessment

**Good coverage.** Every type that has a `Monad` implementation and is HKT-compatible also implements `MonadRec`. The types that cannot implement the trait (`Trampoline`, `SendThunk`, `TrySendThunk`, `TryTrampoline`) provide equivalent inherent methods.

**Missing:** No `MonadRec` for `Free<F, A>` itself (as a trait impl). This is documented and inherent to the `'static` constraint on `Free`. Users access stack-safe recursion on `Free` through `Trampoline::tail_rec_m` or `Free::fold_free` (which takes a `G: MonadRec`).

## 6. Free Function Wrapper

```rust
pub fn tail_rec_m<'a, Brand: MonadRec, A: 'a, B: 'a>(
    func: impl Fn(A) -> Apply!(...) + 'a,
    initial: A,
) -> Apply!(...) {
    Brand::tail_rec_m(func, initial)
}
```

The free function is a straightforward dispatch wrapper. It is re-exported via `fp-library/src/functions.rs` as `functions::tail_rec_m`.

**Ergonomics:** The brand type parameter must be specified explicitly (e.g., `tail_rec_m::<ThunkBrand, _, _>(...)`), which is typical for this library's design. The `A` and `B` parameters can be inferred.

## 7. Documentation Quality

### Trait-level docs

- **Module doc:** Includes a complete, working example (factorial with `ThunkBrand`).
- **Trait doc:** Explains the Thunk vs Trampoline distinction, documents the identity law, states the stack-safety class invariant, and includes a working example.
- **Method doc:** Uses the library's documentation macros (`#[document_signature]`, `#[document_type_parameters]`, etc.) for consistent formatting.

### Implementation docs

Each implementation has:
- A prose description of the specific behavior (short-circuit semantics, breadth-first expansion, etc.).
- Documented type parameters, parameters, and return values.
- A working code example.

### Tests

- **Property-based:** QuickCheck tests for the identity law (`OptionBrand`, `ThunkBrand`).
- **Stack safety:** 200,000-iteration stress tests in most implementations.
- **Short-circuit:** Tests verifying that `None`, `Err`, `Ok`, `Loop`, `Done` short-circuit correctly.
- **Coverage:** Tests exist for all trait implementors and all inherent methods.

### Assessment

Documentation quality is high. The trait docs clearly explain the design tradeoffs and the relationship between `Thunk` (HKT-compatible, conditionally stack-safe) and `Trampoline` (not HKT-compatible, unconditionally stack-safe).

**One area for improvement:** The trait doc mentions only the identity law. PureScript's `MonadRec` documentation also describes a "stack safety" law and an "equivalence" law (`tail_rec_m f a == f a >>= case _ of Loop a' -> tail_rec_m f a'; Done b -> pure b`). The equivalence law is implicitly expected but not stated.

## 8. Issues, Limitations, and Design Flaws

### 8.1. Thunk's partial stack safety

`ThunkBrand`'s `tail_rec_m` wraps the entire loop in a single `Thunk::new`, which means:
- The loop itself is stack-safe (it is a Rust `loop`).
- But the step function's returned thunk is eagerly evaluated via `.evaluate()` at each iteration.
- If the step function returns a thunk with deep bind chains, those chains will blow the stack.

This is documented but represents a footgun. Users who reach for `MonadRec` expect unconditional stack safety, but `ThunkBrand` only provides it under the assumption that the step function returns "shallow" thunks.

### 8.2. No trait impl for `Trampoline`, `SendThunk`, or `Free`

These types provide `tail_rec_m` as inherent methods rather than trait implementations:
- `Trampoline` and `TryTrampoline` require `'static` and `Clone`, incompatible with the trait's HKT signature.
- `SendThunk` and `TrySendThunk` require `Send` bounds not expressible in the trait.

This means generic code written against `MonadRec` cannot use `Trampoline` or `SendThunk`. Users must choose: either write code that is generic over `MonadRec` (and accept the HKT constraints), or write code that is specific to `Trampoline` (and get unconditional stack safety).

This is a fundamental tension that arises from Rust's type system limitations. It is well-documented in the codebase.

### 8.3. `Clone + 'static` requirement on `Trampoline::tail_rec_m`

`Trampoline::tail_rec_m` requires the step function to be `Clone + 'static` because internally it calls `go(f, next)` recursively (building a `Free` chain), and each recursive call needs its own copy of `f`. The `arc_tail_rec_m` variant relaxes `Clone` by wrapping in `Arc`, but still requires `'static`.

This is a practical limitation: closures that capture non-Clone, non-Arc-wrappable state cannot use `Trampoline::tail_rec_m`. However, such closures are rare in practice.

### 8.4. Only one law stated

The trait documents only the identity law:
```
tail_rec_m(|a| pure(Step::Done(a)), x) == pure(x)
```

Missing laws that would strengthen the contract:
- **Equivalence/unfolding law:** `tail_rec_m(f, a)` should be equivalent to `f(a) >>= match { Loop(a') => tail_rec_m(f, a'), Done(b) => pure(b) }`. This law ensures that `tail_rec_m` is a valid optimization of the naive recursive definition.
- **Naturality:** For any monad homomorphism `h: M ~> N`, `h(tail_rec_m(f, a)) == tail_rec_m(h . f, a)`.

The unfolding law is particularly important because it is what makes `tail_rec_m` a "correct" optimization. Without it, an implementation could do something unrelated to the step function.

### 8.5. No `forever` or derived combinators

PureScript's `MonadRec` ecosystem includes derived combinators like `forever` (run an action indefinitely without stack overflow) and `whileM` / `untilM`. The Rust library does not provide these yet. They would be useful for long-running computations.

### 8.6. Breadth-first expansion for Vec/CatList may not terminate

`VecBrand` and `CatListBrand` use breadth-first expansion: each `Loop` value is fed back through the step function, and the results are accumulated. If the step function always produces at least one `Loop` for each input, the computation will never terminate (and will consume unbounded memory). This is inherent to the nondeterministic monad semantics, but it is not explicitly warned about in the docs.

## 9. Alternatives and Improvements

### 9.1. Add the unfolding/equivalence law to documentation

State the equivalence law in the trait doc:

```text
tail_rec_m(f, a) == bind(f(a), |step| match step {
    Step::Loop(a') => tail_rec_m(f, a'),
    Step::Done(b) => pure(b),
})
```

This makes the relationship between `tail_rec_m` and `bind` explicit.

### 9.2. Add property tests for the unfolding law

The current tests verify the identity law. Adding a test that checks the unfolding law (for types where the naive recursive version does not blow the stack, e.g., `OptionBrand` with small iteration counts) would strengthen confidence.

### 9.3. Consider a `SendMonadRec` trait

Currently, thread-safe types like `SendThunk` and `TrySendThunk` provide `tail_rec_m` as inherent methods because the trait cannot express `Send` bounds. A `SendMonadRec` trait (analogous to `SendDeferrable` and `SendRefFunctor`) could enable generic programming over thread-safe monadic recursion. However, this would only be useful if `SendThunk` / `TrySendThunk` could implement it as HKT-compatible types, which they currently cannot.

### 9.4. Add derived combinators

Useful combinators that could be provided as default methods or free functions:
- `forever<M: MonadRec>(action: M<A>) -> M<Void>`: Run an action indefinitely.
- `while_m<M: MonadRec>(cond: M<bool>, body: M<()>) -> M<()>`: Loop while condition holds.
- `until_m<M: MonadRec>(body: M<bool>) -> M<()>`: Loop until body returns true.
- `iterate<M: MonadRec>(f: A -> M<A>, initial: A) -> M<Void>`: Infinite iteration.

These are common in PureScript's ecosystem and would make `MonadRec` more practically useful.

### 9.5. Document the nondeterministic termination caveat

For `VecBrand` and `CatListBrand`, add a warning that the computation may not terminate if the step function always produces `Loop` values with at least one output for every input. Also warn about unbounded memory growth.

### 9.6. Consider `ControlFlow` integration

Since `Step` already has `From<ControlFlow>` / `Into<ControlFlow>` conversions, consider adding a convenience method that accepts `ControlFlow`-returning step functions:

```rust
fn tail_rec_m_cf<'a, A: 'a, B: 'a>(
    func: impl Fn(A) -> M<ControlFlow<B, A>> + 'a,
    initial: A,
) -> M<B>;
```

This would let users use Rust's `?` operator with `ControlFlow` in some cases, improving ergonomics.

## Summary

`MonadRec` is a well-designed, correctly implemented trait that faithfully captures the tail-recursive monad pattern from PureScript. The `Step` type is clean, well-documented, and richly featured. The implementation coverage is comprehensive, with every applicable type providing either a trait impl or an equivalent inherent method.

The main limitations are fundamental to Rust's type system: `Trampoline` and `Free` cannot implement the HKT-based trait due to `'static` and `Clone` requirements, and `SendThunk` cannot express `Send` bounds in the trait. These are well-documented and handled gracefully with inherent methods.

The most actionable improvements are: (1) documenting the unfolding/equivalence law, (2) adding property tests for it, (3) adding derived combinators like `forever`, and (4) documenting the nondeterministic termination caveat for `Vec`/`CatList`.
