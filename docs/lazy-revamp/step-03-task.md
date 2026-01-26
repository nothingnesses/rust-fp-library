# Step 03: Task (Stack-Safe Computation)

## Goal

Implement `Task` and `TryTask`, the stack-safe computation types. `Task` is built on `Free<ThunkF, A>` and guarantees stack safety for deep recursion and long bind chains.

## Files to Create

- `fp-library/src/types/task.rs`
- `fp-library/src/types/try_task.rs`
- `fp-library/tests/stack_safety.rs`

## Files to Modify

- `fp-library/src/types.rs`

## Implementation Details

### Task

A wrapper around `Free<ThunkF, A>`.

- **Constraint**: `A: 'static + Send` (due to `Free`'s type erasure).
- **Constructors**: `now`, `later`, `always`, `defer`.
- **Combinators**: `flat_map`, `map`, `map2`, `and_then`.
- **Recursion**: `tail_rec_m` (standalone method, not trait impl).

````rust
/// A lazy, stack-safe computation that produces a value of type `A`.
///
/// `Task` is the "heavy-duty" monadic type for deferred computations that
/// require **guaranteed stack safety**. It is built on `Free<ThunkF, A>` with
/// CatList-based bind stack, ensuring O(1) bind operations and unlimited recursion
/// depth without stack overflow.
///
/// # Requirements
///
/// - `A: 'static + Send` — Required due to type erasure via `Box<dyn Any>`
///
/// # Guarantees
///
/// - **Stack safe**: Will not overflow regardless of recursion depth
/// - **O(1) bind**: Left-associated `flat_map` chains don't degrade
/// - **Lazy**: Computation is deferred until `run()` is called
///
/// # When to Use Task vs Eval
///
/// - Use **`Task<A>`** for deep recursion (1000+ levels), heavy monadic pipelines
/// - Use **`Eval<'a, A>`** for HKT integration, borrowed references, glue code
///
/// # Memoization
///
/// `Task` does NOT memoize. Each call to `run()` re-evaluates.
/// For memoization, wrap in `Memo`:
///
/// ```rust
/// let memo: Memo<i32> = Memo::new(|| Task::later(|| expensive()).run());
/// memo.get(); // Computes
/// memo.get(); // Returns cached
/// ```
///
/// # Example
///
/// ```rust
/// let task = Task::later(|| 1 + 1)
///     .flat_map(|x| Task::later(move || x * 2))
///     .flat_map(|x| Task::later(move || x + 10));
///
/// assert_eq!(task.run(), 14);
/// ```
pub struct Task<A> {
    inner: Free<ThunkF, A>,
}

impl<A: 'static + Send> Task<A> {
    /// Creates a `Task` from an already-computed value.
    ///
    /// Equivalent to Cats' `Eval.now`.
    ///
    /// # Complexity
    /// O(1) creation, O(1) run
    ///
    /// # Example
    ///
    /// ```rust
    /// let task = Task::now(42);
    /// assert_eq!(task.run(), 42);
    /// ```
    #[inline]
    pub fn now(a: A) -> Self {
        Task {
            inner: Free::pure(a),
        }
    }

    /// Alias for `now` - PureScript style.
    #[inline]
    pub fn pure(a: A) -> Self {
        Self::now(a)
    }

    /// Creates a lazy `Task` that computes `f` on first `run()`.
    ///
    /// This is equivalent to Cats' `Eval.later`, but note that
    /// in our design, `Task` does NOT memoize — each `run()`
    /// re-evaluates. Use `Memo` for caching.
    ///
    /// # Complexity
    /// O(1) creation
    ///
    /// # Example
    ///
    /// ```rust
    /// let task = Task::later(|| {
    ///     println!("Computing!");
    ///     expensive_computation()
    /// });
    ///
    /// // Nothing printed yet
    /// let result = task.run(); // Prints "Computing!"
    /// ```
    #[inline]
    pub fn later<F>(f: F) -> Self
    where
        F: FnOnce() -> A + Send + 'static,
    {
        Task {
            inner: Free::roll(Thunk::new(move || Free::pure(f()))),
        }
    }

    /// Alias for `later` - semantically same since we don't memoize.
    ///
    /// In Cats, `always` differs from `later` in that it re-evaluates.
    /// Since our `Task` always re-evaluates, this is just an alias.
    #[inline]
    pub fn always<F>(f: F) -> Self
    where
        F: FnOnce() -> A + Send + 'static,
    {
        Self::later(f)
    }

    /// Defers the construction of a `Task` itself.
    ///
    /// This is critical for stack-safe recursion: instead of
    /// building a chain of `Task`s directly (which grows the stack),
    /// we defer the construction.
    ///
    /// # Example
    ///
    /// ```rust
    /// fn recursive_sum(n: u64, acc: u64) -> Task<u64> {
    ///     if n == 0 {
    ///         Task::now(acc)
    ///     } else {
    ///         // Defer construction to avoid stack growth
    ///         Task::defer(move || recursive_sum(n - 1, acc + n))
    ///     }
    /// }
    ///
    /// // This works for n = 1_000_000 without stack overflow!
    /// let result = recursive_sum(1_000_000, 0).run();
    /// ```
    #[inline]
    pub fn defer<F>(f: F) -> Self
    where
        F: FnOnce() -> Task<A> + Send + 'static,
    {
        Task {
            inner: Free::roll(Thunk::new(move || f().inner)),
        }
    }

    /// Monadic bind (flatMap) with O(1) complexity.
    ///
    /// Chains computations together. The key property is that
    /// left-associated chains don't degrade to O(n²):
    ///
    /// ```rust
    /// // This is O(n), not O(n²)
    /// let mut task = Task::now(0);
    /// for i in 0..10000 {
    ///     task = task.flat_map(move |x| Task::now(x + i));
    /// }
    /// ```
    #[inline]
    pub fn flat_map<B: 'static + Send, F>(self, f: F) -> Task<B>
    where
        F: FnOnce(A) -> Task<B> + Send + 'static,
    {
        Task {
            inner: self.inner.flat_map(move |a| f(a).inner),
        }
    }

    /// Functor map: transforms the result without changing structure.
    #[inline]
    pub fn map<B: 'static + Send, F>(self, f: F) -> Task<B>
    where
        F: FnOnce(A) -> B + Send + 'static,
    {
        self.flat_map(move |a| Task::now(f(a)))
    }

    /// Forces evaluation and returns the result.
    ///
    /// This runs the trampoline loop, iteratively processing
    /// the CatList of continuations without growing the stack.
    ///
    /// # Example
    ///
    /// ```rust
    /// let task = Task::later(|| 1 + 1);
    /// assert_eq!(task.run(), 2);
    /// ```
    pub fn run(self) -> A {
        self.inner.run()
    }

    /// Combines two `Task`s, running both and combining results.
    pub fn map2<B: 'static + Send, C: 'static + Send, F>(
        self,
        other: Task<B>,
        f: F,
    ) -> Task<C>
    where
        F: FnOnce(A, B) -> C + Send + 'static,
    {
        self.flat_map(move |a| other.map(move |b| f(a, b)))
    }

    /// Sequences two `Task`s, discarding the first result.
    pub fn and_then<B: 'static + Send>(self, other: Task<B>) -> Task<B> {
        self.flat_map(move |_| other)
    }

    /// Creates a `Task` from a memoized value (via Memo).
    ///
    /// This is a convenience for integrating with the dual-type design.
    /// The Memo provides caching; Task provides computation structure.
    pub fn from_memo(memo: &Memo<A>) -> Self
    where
        A: Clone,
    {
        let value = memo.get().clone();
        Task::now(value)
    }
}
````

### MonadRec Implementation for Task

Note: `Task` does **not** implement the HKT-based `MonadRec` trait due to its `'static` requirement conflicting with HKT's `for<'a>` bounds. Instead, `Task` provides standalone `tail_rec_m` methods:

````rust
// Task provides its own tail_rec_m, not the trait-based MonadRec
impl<A: 'static + Send> Task<A> {
    /// Stack-safe tail recursion within Task.
    ///
    /// # Clone Bound
    ///
    /// The function `f` must implement `Clone` because each iteration
    /// of the recursion may need its own copy. Most closures naturally
    /// implement `Clone` when all their captures implement `Clone`.
    ///
    /// For closures that don't implement `Clone`, use `tail_rec_m_shared`
    /// which wraps the closure in `Arc` internally.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Fibonacci using tail recursion
    /// fn fib(n: u64) -> Task<u64> {
    ///     Task::tail_rec_m(|(n, a, b)| {
    ///         if n == 0 {
    ///             Task::now(Step::Done(a))
    ///         } else {
    ///             Task::now(Step::Loop((n - 1, b, a + b)))
    ///         }
    ///     }, (n, 0u64, 1u64))
    /// }
    ///
    /// assert_eq!(fib(50).run(), 12586269025);
    /// ```
    pub fn tail_rec_m<S: 'static + Send, F>(
        f: F,
        initial: S,
    ) -> Self
    where
        F: Fn(S) -> Task<Step<S, A>> + Clone + Send + 'static,
    {
        // Use defer to ensure each step is trampolined.
        fn go<A: 'static + Send, B: 'static + Send, F>(
            f: F,
            a: A,
        ) -> Task<B>
        where
            F: Fn(A) -> Task<Step<A, B>> + Clone + Send + 'static,
        {
            let f_clone = f.clone();
            Task::defer(move || {
                f(a).flat_map(move |step| match step {
                    Step::Loop(next) => go(f_clone.clone(), next),
                    Step::Done(b) => Task::now(b),
                })
            })
        }

        go(f, initial)
    }

    /// Arc-wrapped version for non-Clone closures.
    ///
    /// Use this when your closure captures non-Clone state.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Closure captures non-Clone state
    /// let counter = SomeNonCloneCounter::new();
    /// Task::tail_rec_m_shared(|n| {
    ///     counter.increment();
    ///     if n == 0 {
    ///         Task::now(Step::Done(counter.get()))
    ///     } else {
    ///         Task::now(Step::Loop(n - 1))
    ///     }
    /// }, 100)
    /// ```
    pub fn tail_rec_m_shared<S: 'static + Send, F>(
        f: F,
        initial: S,
    ) -> Self
    where
        F: Fn(S) -> Task<Step<S, A>> + Send + 'static,
    {
        use std::sync::Arc;
        let f = Arc::new(f);
        let wrapper = move |s: S| {
            let f = Arc::clone(&f);
            f(s)
        };
        Self::tail_rec_m(wrapper, initial)
    }
}
````

### TryTask: Fallible Stack-Safe Computations

A wrapper around `Task<Result<A, E>>`.

- **Constructors**: `ok`, `err`, `try_later`.
- **Combinators**: `map`, `map_err`, `and_then`, `or_else`.

For computations that might fail, we provide `TryTask`:

```rust
/// A lazy, stack-safe computation that may fail with an error.
///
/// This is `Task<Result<A, E>>` with ergonomic combinators.
pub struct TryTask<A, E> {
    inner: Task<Result<A, E>>,
}

impl<A: 'static + Send, E: 'static + Send> TryTask<A, E> {
    /// Creates a successful `TryTask`.
    pub fn ok(a: A) -> Self {
        TryTask {
            inner: Task::now(Ok(a)),
        }
    }

    /// Creates a failed `TryTask`.
    pub fn err(e: E) -> Self {
        TryTask {
            inner: Task::now(Err(e)),
        }
    }

    /// Creates a lazy `TryTask` that may fail.
    pub fn try_later<F>(f: F) -> Self
    where
        F: FnOnce() -> Result<A, E> + Send + 'static,
    {
        TryTask {
            inner: Task::later(f),
        }
    }

    /// Maps over the success value.
    pub fn map<B: 'static + Send, F>(self, f: F) -> TryTask<B, E>
    where
        F: FnOnce(A) -> B + Send + 'static,
    {
        TryTask {
            inner: self.inner.map(|result| result.map(f)),
        }
    }

    /// Maps over the error value.
    pub fn map_err<E2: 'static + Send, F>(self, f: F) -> TryTask<A, E2>
    where
        F: FnOnce(E) -> E2 + Send + 'static,
    {
        TryTask {
            inner: self.inner.map(|result| result.map_err(f)),
        }
    }

    /// Chains fallible computations.
    pub fn and_then<B: 'static + Send, F>(self, f: F) -> TryTask<B, E>
    where
        F: FnOnce(A) -> TryTask<B, E> + Send + 'static,
    {
        TryTask {
            inner: self.inner.flat_map(|result| match result {
                Ok(a) => f(a).inner,
                Err(e) => Task::now(Err(e)),
            }),
        }
    }

    /// Recovers from an error.
    pub fn or_else<F>(self, f: F) -> Self
    where
        F: FnOnce(E) -> TryTask<A, E> + Send + 'static,
    {
        TryTask {
            inner: self.inner.flat_map(|result| match result {
                Ok(a) => Task::now(Ok(a)),
                Err(e) => f(e).inner,
            }),
        }
    }

    /// Runs the computation, returning the result.
    pub fn run(self) -> Result<A, E> {
        self.inner.run()
    }
}
```

### Usage Examples

#### Example 1: Deep Recursion with Task

```rust
/// Computes factorial using stack-safe recursion.
fn factorial(n: u64) -> Task<u64> {
    Task::tail_rec_m(|(n, acc)| {
        if n <= 1 {
            Task::now(Step::Done(acc))
        } else {
            Task::now(Step::Loop((n - 1, n * acc)))
        }
    }, (n, 1u64))
}

// Works for any n without stack overflow
assert_eq!(factorial(100_000).run(), /* very large number */);
```

#### Example 2: Lazy Tree Traversal with Task

```rust
enum Tree<A> {
    Leaf(A),
    Branch(Box<Tree<A>>, Box<Tree<A>>),
}

fn sum_tree(tree: Tree<i64>) -> Task<i64> {
    match tree {
        Tree::Leaf(x) => Task::now(x),
        Tree::Branch(left, right) => {
            // Defer to avoid stack growth on deep trees
            Task::defer(move || {
                sum_tree(*left).flat_map(move |l| {
                    sum_tree(*right).map(move |r| l + r)
                })
            })
        }
    }
}
```

#### Example 3: Stack-Safe Pipeline with TryTask

```rust
fn parse_config(path: &Path) -> TryTask<Config, ConfigError> {
    let path = path.to_owned();
    TryTask::try_later(move || {
        let content = std::fs::read_to_string(&path)?;
        parse_toml(&content)
    })
}

fn validate_config(config: Config) -> TryTask<ValidConfig, ConfigError> {
    TryTask::try_later(move || config.validate())
}

fn load_config(path: &Path) -> TryTask<ValidConfig, ConfigError> {
    parse_config(path)
        .and_then(validate_config)
        .map(|c| c.normalize())
}

// Nothing executes until .run()
let result = load_config(Path::new("app.toml")).run();
```

#### Example 4: With Memoization

```rust
use std::sync::Arc;

// Expensive computation via Task
let counter = Arc::new(AtomicUsize::new(0));
let counter_clone = Arc::clone(&counter);

// Without Memo: runs every time
let task1 = Task::later({
    let c = Arc::clone(&counter);
    move || { c.fetch_add(1, Ordering::SeqCst); heavy_computation() }
});
let task2 = Task::later({
    let c = Arc::clone(&counter);
    move || { c.fetch_add(1, Ordering::SeqCst); heavy_computation() }
});
let result1 = task1.run();
let result2 = task2.run();
assert_eq!(counter.load(Ordering::SeqCst), 2); // Ran twice

// With Memo: memoized
let memoized = Memo::new({
    let c = Arc::clone(&counter);
    move || { c.fetch_add(1, Ordering::SeqCst); heavy_computation() }
});
let result3 = memoized.get();
let result4 = memoized.get();
assert_eq!(counter.load(Ordering::SeqCst), 3); // Only ran once more
assert_eq!(result3, result4);
```

## Tests

### Task Tests

1.  **Basic Execution**: `now`, `later`, `run`.
2.  **Defer**: Verify `defer` delays execution.
3.  **FlatMap**: Chain multiple operations.
4.  **Tail Recursion**: Implement factorial or fibonacci using `tail_rec_m`.

### TryTask Tests

1.  **Success/Failure**: Verify `ok` and `err` paths.
2.  **Combinators**: Verify `map` only affects success, `map_err` only affects error.
3.  **Short-circuiting**: Verify `and_then` stops at first error.

### Stack Safety Tests (`tests/stack_safety.rs`)

1.  **Deep Recursion**: `tail_rec_m` with 1,000,000 iterations.
2.  **Deep Bind Chain**: 10,000 left-associated `flat_map` calls.
3.  **Deep Defer**: 10,000 nested `defer` calls.

## Checklist

- [ ] Create `fp-library/src/types/task.rs`
  - [ ] Implement `Task` struct
  - [ ] Implement constructors (`now`, `later`, `always`, `defer`)
  - [ ] Implement combinators (`flat_map`, `map`, etc.)
  - [ ] Implement `run`
  - [ ] Implement `tail_rec_m` and `tail_rec_m_shared`
  - [ ] Add unit tests
- [ ] Create `fp-library/src/types/try_task.rs`
  - [ ] Implement `TryTask` struct
  - [ ] Implement constructors and combinators
  - [ ] Implement `run`
  - [ ] Add unit tests
- [ ] Create `fp-library/tests/stack_safety.rs`
  - [ ] Add deep recursion tests
  - [ ] Add deep bind chain tests
- [ ] Update `fp-library/src/types.rs`
