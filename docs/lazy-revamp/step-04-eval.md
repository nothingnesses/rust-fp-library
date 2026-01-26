# Step 04: Eval (HKT-Compatible Computation)

## Goal
Implement `Eval` and `TryEval`, the closure-based computation types. These types support Higher-Kinded Types (HKT) and borrowed references but are NOT stack-safe for deep recursion.

## Files to Create
- `fp-library/src/types/eval.rs`
- `fp-library/src/types/try_eval.rs`

## Files to Modify
- `fp-library/src/types.rs`

## Implementation Details

### Eval<'a, A> - Pure Deferred Computation

A wrapper around a boxed closure `Box<dyn FnOnce() -> A + 'a>`.
- **Lifetime**: `'a` (allows borrowing).
- **No 'static**: Unlike `Task`, `Eval` works with non-static data.
- **Constructors**: `new`, `pure`, `defer`.
- **Combinators**: `flat_map`, `map`.
- **Conversions**: `into_try` (converts to `TryEval` that always succeeds).

```rust
/// A deferred computation that produces a value of type `A`.
///
/// `Eval` is NOT memoized - each call to `run()` re-executes the computation.
/// This type exists to build computation chains without allocation overhead.
///
/// Unlike `Task<A>`, `Eval` does NOT require `'static` and CAN implement
/// HKT traits like `Functor`, `Semimonad`, etc.
///
/// # Trade-offs vs Task
///
/// | Aspect         | Eval<'a, A>               | Task<A>                    |
/// |----------------|---------------------------|----------------------------|
/// | HKT compatible | ✅ Yes                    | ❌ No (requires `'static`) |
/// | Stack-safe     | ❌ No (~8000 calls limit) | ✅ Yes (unlimited)         |
/// | Lifetime       | `'a` (can borrow)         | `'static` only             |
/// | Use case       | Glue code, composition    | Deep recursion, pipelines  |
///
/// # Algebraic Properties
///
/// `Eval` is a proper Monad:
/// - `pure(a).run() == a` (left identity)
/// - `eval.bind(pure) == eval` (right identity)
/// - `eval.bind(f).bind(g) == eval.bind(|a| f(a).bind(g))` (associativity)
///
/// # Examples
///
/// ```rust
/// let computation = Eval::new(|| expensive_calculation())
///     .map(|x| x * 2)
///     .map(|x| x + 1);
///
/// // No computation has happened yet!
/// // Only when we call run() does it execute:
/// let result = computation.run();
/// ```
pub struct Eval<'a, A> {
    thunk: Box<dyn FnOnce() -> A + 'a>,
}

impl<'a, A> Eval<'a, A> {
    /// Creates a new Eval from a thunk.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> A + 'a,
    {
        Eval {
            thunk: Box::new(f),
        }
    }

    /// Returns a pure value (already computed).
    pub fn pure(a: A) -> Self
    where
        A: 'a,
    {
        Eval::new(move || a)
    }

    /// Defers a computation that returns an Eval.
    pub fn defer<F>(f: F) -> Self
    where
        F: FnOnce() -> Eval<'a, A> + 'a,
    {
        Eval::new(move || f().run())
    }

    /// Monadic bind: chains computations.
    ///
    /// Note: Each `flat_map` adds to the call stack. For deep recursion
    /// (>1000 levels), use `Task` instead.
    pub fn flat_map<B, F>(self, f: F) -> Eval<'a, B>
    where
        F: FnOnce(A) -> Eval<'a, B> + 'a,
    {
        Eval::new(move || {
            let a = (self.thunk)();
            let eval_b = f(a);
            (eval_b.thunk)()
        })
    }

    /// Functor map: transforms the result.
    pub fn map<B, F>(self, f: F) -> Eval<'a, B>
    where
        F: FnOnce(A) -> B + 'a,
    {
        Eval::new(move || f((self.thunk)()))
    }

    /// Forces evaluation and returns the result.
    pub fn run(self) -> A {
        (self.thunk)()
    }

    /// Converts to a TryEval that always succeeds.
    pub fn into_try<E>(self) -> TryEval<'a, A, E> {
        TryEval::new(move || Ok(self.run()))
    }
}
```

**Design Decisions:**

1. **`Box<dyn FnOnce() -> A + 'a>`**: Uses `FnOnce` because computations are typically consumed once. The lifetime `'a` allows capturing borrowed data.

2. **`run(self) -> A`**: Consumes `self` and returns owned `A`. This is crucial for Monad compatibility - we can chain without requiring `Clone`.

3. **No `Rc`/`Arc`**: No shared ownership means no allocation beyond the `Box`. Building a chain of 10 maps creates 10 `Box` allocations, not 10 `Rc` + 10 `OnceCell`.

4. **Implements `Functor` and `Monad`**: Because `run()` returns owned values, we can implement the standard typeclasses properly.

### TryEval<'a, A, E> - Fallible Deferred Computation

A wrapper around `Box<dyn FnOnce() -> Result<A, E> + 'a>`.
- **Constructors**: `new`, `pure`, `ok`, `err`.
- **Combinators**: `flat_map`, `map`, `map_err`.

```rust
/// A deferred computation that may fail with error type `E`.
///
/// Like `Eval`, this is NOT memoized. Each `run()` re-executes.
/// Unlike `Eval`, the result is `Result<A, E>`.
///
/// # Examples
///
/// ```rust
/// let computation: TryEval<Config, ConfigError> = TryEval::new(|| {
///     let path = std::env::var("CONFIG_PATH")?;
///     parse_config(&path)
/// });
///
/// match computation.run() {
///     Ok(config) => use_config(config),
///     Err(e) => handle_error(e),
/// }
/// ```
pub struct TryEval<'a, A, E> {
    thunk: Box<dyn FnOnce() -> Result<A, E> + 'a>,
}

impl<'a, A, E> TryEval<'a, A, E> {
    /// Creates a new TryEval from a thunk.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> Result<A, E> + 'a,
    {
        TryEval {
            thunk: Box::new(f),
        }
    }

    /// Returns a pure value (already computed).
    pub fn pure(a: A) -> Self
    where
        A: 'a,
    {
        TryEval::new(move || Ok(a))
    }

    /// Returns a pure error.
    pub fn err(e: E) -> Self
    where
        E: 'a,
    {
        TryEval::new(move || Err(e))
    }

    /// Monadic bind: chains computations.
    pub fn flat_map<B, F>(self, f: F) -> TryEval<'a, B, E>
    where
        F: FnOnce(A) -> TryEval<'a, B, E> + 'a,
    {
        TryEval::new(move || {
            match (self.thunk)() {
                Ok(a) => (f(a).thunk)(),
                Err(e) => Err(e),
            }
        })
    }

    /// Functor map: transforms the result.
    pub fn map<B, F>(self, f: F) -> TryEval<'a, B, E>
    where
        F: FnOnce(A) -> B + 'a,
    {
        TryEval::new(move || (self.thunk)().map(f))
    }

    /// Map error: transforms the error.
    pub fn map_err<E2, F>(self, f: F) -> TryEval<'a, A, E2>
    where
        F: FnOnce(E) -> E2 + 'a,
    {
        TryEval::new(move || (self.thunk)().map_err(f))
    }

    /// Forces evaluation and returns the result.
    pub fn run(self) -> Result<A, E> {
        (self.thunk)()
    }
}
```

**Design Decisions:**

1. **Explicit `E` type parameter**: The error type is part of the signature. No surprise panics, no `Box<dyn Error>` - you know exactly what can fail.

2. **`run(self) -> Result<A, E>`**: Returns the full `Result`, not `Result<&A, &E>`. Ownership semantics preserved.

3. **No implicit panic catching**: If the thunk panics, the panic propagates. Use `TryEval::catch_unwind()` if you want to catch panics.

4. **Implements `Functor` for the success type**: `map` transforms `A`, `map_err` transforms `E`.

## Tests

### Eval Tests
1.  **Basic Execution**: `new`, `pure`, `run`.
2.  **Borrowing**: Verify `Eval` can capture references (e.g., `&str`).
3.  **Composition**: Chain `map` and `flat_map`.
4.  **Defer**: Verify `defer` works.

### TryEval Tests
1.  **Success/Failure**: Verify `ok` and `err` paths.
2.  **Combinators**: Verify `map` and `map_err`.
3.  **Borrowing**: Verify capturing references works.

## Checklist
- [ ] Create `fp-library/src/types/eval.rs`
    - [ ] Implement `Eval` struct
    - [ ] Implement constructors (`new`, `pure`, `defer`)
    - [ ] Implement combinators (`flat_map`, `map`)
    - [ ] Implement `into_try<E>` conversion to `TryEval`
    - [ ] Implement `run`
    - [ ] Add unit tests (including borrowing tests)
- [ ] Create `fp-library/src/types/try_eval.rs`
    - [ ] Implement `TryEval` struct
    - [ ] Implement constructors and combinators
    - [ ] Implement `run`
    - [ ] Add unit tests
- [ ] Update `fp-library/src/types.rs`
