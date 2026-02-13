# Stack-Safe Evaluation Architecture

## Executive Summary

This document proposes a library-wide architectural change to address **stack safety**. Currently, several core abstractions (`Endofunction`, `Lazy`, `Foldable`) rely on nested closures ("direct style") to compose computations. While performant for shallow chains, this pattern causes **stack overflows** on deep recursion or long composition chains (e.g., folding a large list).

We propose introducing a core **`Eval`** primitive (based on the "Trampoline" pattern) to convert recursion into iteration. This primitive will serve as the foundation for stack-safe `Lazy` evaluation, `Endofunction` composition, and `Foldable` operations.

## 1. The Problem: Nested Closures

In Rust, functions that call other functions consume stack space. When we compose functions dynamically, we create a chain of calls.

### Case A: `Endofunction`

```rust
// Current implementation of append (f . g)
move |x| f(g(x))
```

Chaining 10,000 endofunctions results in a closure that, when called, creates 10,000 stack frames.

### Case B: `Lazy`

```rust
// Current implementation of bind
move || f(ma.run()).run()
```

Chaining `bind` creates a nested thunk structure `|| f(g(h(...).run()).run()).run()`. Forcing this thunk blows the stack.

### Case C: `Foldable`

The default `fold_right` implementation maps elements to `Endofunction`s and composes them. This inherits the stack overflow issue from `Endofunction`.

## 2. The Solution: `Eval` (The Trampoline)

We introduce a new data type, `Eval<A>`, which represents a **computation** rather than a value. Instead of executing recursively on the stack, `Eval` builds a data structure describing the computation, which is then executed by a loop (the "trampoline") on the heap.

### 2.1 Data Structure

To support `flatMap` (which changes types) in a strongly-typed enum, we must type-erase the intermediate result.

```rust
use std::any::Any;

pub enum Eval<A> {
    /// A value that is ready immediately.
    Pure(A),

    /// A computation to be run later (trampolined).
    /// Used to break stack recursion.
    /// Note: This does NOT memoize. It corresponds to Cats' `Always`.
    /// For memoization (Cats' `Later`), use `Lazy` which wraps `Eval`.
    Defer(Box<dyn FnOnce() -> Eval<A>>),

    /// A chain of computations.
    /// We use `Box<dyn Any>` to hide the intermediate type `B`.
    /// `step`: Produces some value `B`.
    /// `cont`: Takes `B` and produces `Eval<A>`.
    FlatMap(Box<dyn FnOnce() -> Eval<Box<dyn Any>>>, Box<dyn FnOnce(Box<dyn Any>) -> Eval<A>>),
}
```

_Note: The `FlatMap` definition above is a conceptual simplification. In practice, we might use a trait object `EvalTrait` or a specific "Compute" variant to handle the type erasure more ergonomically, or restrict `Eval` to `A` where `A` is the same (for `Endofunction`), but for a general Monad, type erasure is required._

### 2.2 The Interpreter (Run Loop)

The `run` method executes the `Eval` program iteratively.

```rust
impl<A: 'static> Eval<A> {
    /// Helper to create a deferred computation.
    /// This is the primary tool for stack-safe recursion.
    pub fn defer<F>(f: F) -> Eval<A>
    where F: FnOnce() -> Eval<A> + 'static
    {
        Eval::Defer(Box::new(f))
    }

    pub fn run(self) -> A {
        let mut current: Eval<Box<dyn Any>> = self.map(|a| Box::new(a) as Box<dyn Any>);
        let mut stack: Vec<Box<dyn FnOnce(Box<dyn Any>) -> Eval<Box<dyn Any>>>> = Vec::new();

        loop {
            match current {
                Eval::Pure(val) => {
                    if let Some(cont) = stack.pop() {
                        current = cont(val);
                    } else {
                        return *val.downcast().expect("Type mismatch in Eval");
                    }
                }
                Eval::Defer(thunk) => {
                    current = thunk().map(|a| Box::new(a) as Box<dyn Any>);
                }
                Eval::FlatMap(step, cont) => {
                    stack.push(cont);
                    current = step();
                }
            }
        }
    }
}

impl Eval<()> {
    pub const UNIT: Eval<()> = Eval::Pure(());
}
```

**Justification:**

- **Stack Safety:** The recursion is moved to the `stack` Vec (heap). The CPU stack depth remains constant (inside the `loop`).
- **General Purpose:** This structure supports `map`, `flatMap`, and `defer`, making it a full Monad.

## 3. Integration Strategy

### 3.1 `Lazy` Refactor

The `Lazy` type proposed in the "Dual-Type Design" should be backed by `Eval`.

- **`Eval<A>`**: The unmemoized, stack-safe computation type. This corresponds to Cats' `Always` (when deferred) or `Now` (when pure).
- **`Memo<A>`**: The memoized wrapper. It holds a `OnceCell<A>` (or `LazyLock`). When forced, it constructs an `Eval` (if not already computed) and runs it. This corresponds to Cats' `Later`.

**Change:** `Lazy::bind` will no longer return a closure `|| ...`. Instead, it will return `Eval::FlatMap(...)`.

### 3.2 `SafeEndofunction`

We cannot easily change `Endofunction` because it wraps a raw `Fn(A) -> A`. We should introduce a stack-safe alternative.

```rust
pub struct SafeEndofunction<A>(Box<dyn Fn(A) -> Eval<A>>);

impl<A> Semigroup for SafeEndofunction<A> {
    fn append(self, other: Self) -> Self {
        // f . g = \x -> g(x).flatMap(f)
        Self(Box::new(move |x| {
            other.0(x).flat_map(move |y| self.0(y))
        }))
    }
}
```

**Justification:**

- This allows composing infinite chains of functions safely.
- Users can opt-in to safety when they know they have deep chains.

### 3.3 `Foldable` Enhancements

We should add a method to `Foldable` that supports stack-safe folding via `Eval`.

```rust
pub trait Foldable {
    // ... existing methods ...

    /// A stack-safe fold using Eval.
    fn fold_right_eval<A, B, F>(fa: Kind<F, A>, initial: Eval<B>, f: F) -> Eval<B>
    where F: Fn(A, Eval<B>) -> Eval<B>;
}
```

The default implementation of `fold_right` can then be rewritten to use `fold_right_eval` internally if the structure is known to be deep, or we can leave `fold_right` as is (fast, unsafe) and encourage `fold_right_eval` for safety.

**Better Approach:**
Implement `fold_right` _using_ `fold_right_eval` by wrapping the accumulator in `Eval::Pure` and calling `.run()` at the end. This makes the default `fold_right` stack-safe (though slower due to allocation).

## 4. Trade-offs and Justification

### 4.1 Performance vs. Safety

- **Closures (Current):** Zero allocation, static dispatch, very fast. Unsafe for recursion.
- **Eval (Proposed):** Heap allocation (Box), dynamic dispatch (dyn Fn), slower. Safe for recursion.

**Decision:**
We should provide **both**.

1.  Keep the "fast, unsafe" types for standard use cases (small lists, short chains).
2.  Provide `Eval` and "Safe" variants (`SafeEndofunction`) for heavy-duty functional programming.
3.  However, for `Lazy`, stack safety is often a primary requirement (users reach for `Lazy` specifically to handle complex/recursive cases). Therefore, `Lazy` should likely default to using `Eval` or offer a configuration option.

### 4.2 Type Erasure

Using `Box<dyn Any>` in `Eval` incurs runtime type checking overhead (though it should never fail if implemented correctly).

**Justification:**
Rust's type system cannot easily express the existential type of the intermediate step in a `flatMap` chain (`exists B. (Eval<B>, B -> Eval<A>)`) without generic associated types (GATs) or dynamic typing. Given the constraints, `dyn Any` is a pragmatic solution to achieve a true `Trampoline` in stable Rust.

## 5. Cats Parity & Improvements

This design achieves parity with the core functionality of Cats' `Eval` while adapting to Rust's idioms:

1.  **Trampolining**: We use the same `FlatMap`/`Defer` structure to reify the stack, ensuring safety for deep recursion.
2.  **Memoization**: Unlike Cats, which embeds memoization into the `Eval` interpreter, we separate it into a distinct `Lazy`/`Memo` type. This avoids carrying interior mutability (`RefCell`/`Mutex`) in the pure computation type, which is better for Rust's thread-safety model (`Send`/`Sync`).
3.  **Optimization**: We include constants like `UNIT` to reduce allocation overhead for common values.

## 6. Implementation Plan

1.  **Implement `Eval`**: Create `fp-library/src/types/eval.rs`. Implement `Functor`, `Applicative`, `Monad`. Include stack-safety tests (e.g., 1M iterations of flatMap).
2.  **Update `Lazy` Proposal**: Modify the "Dual-Type" proposal to use `Eval` as the backing computation engine.
3.  **Add `SafeEndofunction`**: Add to `fp-library/src/types/`.
4.  **Update `Foldable`**: Add `fold_right_eval` to the trait.

This architecture aligns `fp-library` with mature FP ecosystems (like Cats/Scalaz) while respecting Rust's memory model constraints.
