# Step 5: Memoization

## Goal

Implement `Memo` and `TryMemo`, the memoization layer that caches results using `std::cell::LazyCell` and `std::sync::LazyLock`.

## Files to Create

- `fp-library/src/types/memo.rs`
- `fp-library/src/types/try_memo.rs`

## Files to Modify

- `fp-library/src/types.rs`

## Implementation Details

### MemoConfig Trait

A trait abstracting over `Rc`/`Arc` and `LazyCell`/`LazyLock`.

- **RcMemoConfig**: Uses `Rc<LazyCell<...>>`.
- **ArcMemoConfig**: Uses `Arc<LazyLock<...>>`.

```rust
use std::cell::LazyCell;
use std::sync::LazyLock;
use std::rc::Rc;
use std::sync::Arc;

/// Configuration for memoization strategy.
///
/// This trait bundles together the choices for:
/// - Pointer type (Rc vs Arc)
/// - Lazy cell type (LazyCell vs LazyLock)
///
/// # Note on Standard Library Usage
///
/// This design leverages Rust 1.80's `LazyCell` and `LazyLock` types,
/// which encapsulate the initialization-once logic that we previously
/// implemented manually with `OnceCell` + `UnsafeCell<Option<thunk>>`.
pub trait MemoConfig: 'static {
    /// The lazy cell type for infallible memoization
    type Lazy<A: 'static>: Clone;

    /// The lazy cell type for fallible memoization
    type TryLazy<A: 'static, E: 'static>: Clone;

    /// Creates a new lazy cell from an initializer
    fn new_lazy<A: 'static>(f: impl FnOnce() -> A + 'static) -> Self::Lazy<A>;

    /// Creates a new fallible lazy cell from an initializer
    fn new_try_lazy<A: 'static, E: 'static>(
        f: impl FnOnce() -> Result<A, E> + 'static
    ) -> Self::TryLazy<A, E>;

    /// Forces evaluation and returns a reference
    fn force<A>(lazy: &Self::Lazy<A>) -> &A;

    /// Forces evaluation and returns a reference to the result
    fn force_try<A, E>(lazy: &Self::TryLazy<A, E>) -> Result<&A, &E>;
}

/// Single-threaded memoization using Rc<LazyCell>.
///
/// Not thread-safe. Use `ArcMemoConfig` for multi-threaded contexts.
pub struct RcMemoConfig;

impl MemoConfig for RcMemoConfig {
    type Lazy<A: 'static> = Rc<LazyCell<A, Box<dyn FnOnce() -> A>>>;
    type TryLazy<A: 'static, E: 'static> = Rc<LazyCell<Result<A, E>, Box<dyn FnOnce() -> Result<A, E>>>>;

    fn new_lazy<A: 'static>(f: impl FnOnce() -> A + 'static) -> Self::Lazy<A> {
        Rc::new(LazyCell::new(Box::new(f)))
    }

    fn new_try_lazy<A: 'static, E: 'static>(
        f: impl FnOnce() -> Result<A, E> + 'static
    ) -> Self::TryLazy<A, E> {
        Rc::new(LazyCell::new(Box::new(f)))
    }

    fn force<A>(lazy: &Self::Lazy<A>) -> &A {
        LazyCell::force(lazy)
    }

    fn force_try<A, E>(lazy: &Self::TryLazy<A, E>) -> Result<&A, &E> {
        LazyCell::force(lazy).as_ref()
    }
}

/// Thread-safe memoization using Arc<LazyLock>.
///
/// Requires `A: Send + Sync` for the value type.
pub struct ArcMemoConfig;

impl MemoConfig for ArcMemoConfig {
    type Lazy<A: 'static> = Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send>>>;
    type TryLazy<A: 'static, E: 'static> = Arc<LazyLock<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + Send>>>;

    fn new_lazy<A: 'static>(f: impl FnOnce() -> A + Send + 'static) -> Self::Lazy<A> {
        Arc::new(LazyLock::new(Box::new(f)))
    }

    fn new_try_lazy<A: 'static, E: 'static>(
        f: impl FnOnce() -> Result<A, E> + Send + 'static
    ) -> Self::TryLazy<A, E> {
        Arc::new(LazyLock::new(Box::new(f)))
    }

    fn force<A>(lazy: &Self::Lazy<A>) -> &A {
        LazyLock::force(lazy)
    }

    fn force_try<A, E>(lazy: &Self::TryLazy<A, E>) -> Result<&A, &E> {
        LazyLock::force(lazy).as_ref()
    }
}
```

### Memo<A, Config> - Memoized Value

A memoized value.

- **Constructors**: `new`, `from_task`, `from_eval`.
- **Access**: `get(&self) -> &A`.
- **Conversions**: `into_try<E>` (converts to `TryMemo` that always succeeds).

````rust
use std::cell::LazyCell;
use std::sync::LazyLock;

/// A lazily-computed, memoized value with shared semantics.
///
/// The computation runs at most once; subsequent accesses return the cached value.
/// Cloning a `Memo` shares the underlying cache - all clones see the same value.
///
/// # Implementation Note
///
/// Internally uses `std::cell::LazyCell` (for `RcMemoConfig`) or `std::sync::LazyLock`
/// (for `ArcMemoConfig`) to handle the memoization logic. This delegates the tricky
/// initialization-once semantics to the well-tested standard library.
///
/// # Type Parameters
///
/// - `A`: The type of the computed value
/// - `Config`: The memoization configuration (determines Rc vs Arc)
///
/// # Examples
///
/// ```rust
/// let memo = Memo::<_, RcMemoConfig>::new(|| expensive_calculation());
/// let shared = memo.clone();
///
/// // First force computes and caches:
/// let value = memo.force();
///
/// // Second force returns cached value (shared sees same result):
/// assert_eq!(shared.force(), value);
/// ```
pub struct Memo<A, Config: MemoConfig = RcMemoConfig> {
    inner: Config::Lazy<A>,
}

// For RcMemoConfig: uses Rc<LazyCell<A, Box<dyn FnOnce() -> A>>>
// For ArcMemoConfig: uses Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send>>>
````

### TryMemo<A, E, Config> - Memoized Fallible Value

A memoized fallible value.

- **Constructors**: `new`, `from_try_task`, `from_try_eval`, `catch_unwind`.
- **Access**: `get(&self) -> Result<&A, &E>`.
- **`catch_unwind`**: Static factory method that wraps a potentially-panicking thunk and converts panics to errors (opt-in panic catching).

````rust
/// A lazily-computed, memoized value that may fail.
///
/// The computation runs at most once. If it succeeds, the value is cached.
/// If it fails, the error is cached. Subsequent accesses return the cached result.
///
/// # Implementation Note
///
/// Like `Memo`, internally uses `LazyCell`/`LazyLock` to store `Result<A, E>`.
/// The standard library handles the initialization-once logic.
///
/// # Examples
///
/// ```rust
/// let memo: TryMemo<Config, ConfigError, RcMemoConfig> = TryMemo::new(|| {
///     parse_config_file()
/// });
///
/// match memo.force() {
///     Ok(config) => println!("Config: {:?}", config),
///     Err(e) => println!("Error (cached): {:?}", e),
/// }
///
/// // Second call returns cached result (success or error)
/// let _ = memo.force();
/// ```
pub struct TryMemo<A, E, Config: MemoConfig = RcMemoConfig> {
    // Stores Result<A, E> in the lazy cell
    inner: Config::TryLazy<A, E>,
}

// For RcMemoConfig: uses Rc<LazyCell<Result<A, E>, Box<dyn FnOnce() -> Result<A, E>>>>
// For ArcMemoConfig: uses Arc<LazyLock<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + Send>>>
````

### Type Aliases for Ergonomics

```rust
// Single-threaded variants
pub type RcMemo<A> = Memo<A, RcMemoConfig>;
pub type RcTryMemo<A, E> = TryMemo<A, E, RcMemoConfig>;

// Thread-safe variants
pub type ArcMemo<A> = Memo<A, ArcMemoConfig>;
pub type ArcTryMemo<A, E> = TryMemo<A, E, ArcMemoConfig>;
```

### Conversions Between Types

To avoid circular dependencies between computation types (`Eval`, `Task`) and caching types (`Memo`), conversions are implemented as factory methods on the `Memo` types.

```rust
impl<A, Config: MemoConfig> Memo<A, Config> {
    /// Creates a new Memo that will run `f` on first access.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> A + Send + 'static, // Send bound depends on Config, simplified here
    {
        Memo {
            inner: Config::new_lazy(f),
        }
    }

    /// Gets the memoized value, computing on first access.
    pub fn get(&self) -> &A {
        Config::force(&self.inner)
    }

    /// Creates a Memo from an Eval (HKT-compatible computation).
    ///
    /// # Requirements
    ///
    /// The Eval must be `'static` because `Memo` stores the thunk in a
    /// `LazyCell`/`LazyLock` which typically requires `'static` data.
    pub fn from_eval(eval: Eval<'static, A>) -> Self {
        Memo::new(move || eval.run())
    }

    /// Creates a Memo from a Task (stack-safe computation).
    ///
    /// Note: Task is introduced in the Hybrid Stack-Safety proposal.
    pub fn from_task(task: Task<A>) -> Self
    where
        A: 'static + Send,
    {
        Memo::new(move || task.run())
    }

    /// Converts to a TryMemo that always succeeds.
    pub fn into_try<E>(self) -> TryMemo<A, E, Config>
    where
        A: Clone + 'static,
        E: 'static,
    {
        TryMemo::new(move || Ok(self.get().clone()))
    }
}

impl<A, E, Config: MemoConfig> TryMemo<A, E, Config> {
    /// Creates a new TryMemo that will run `f` on first access.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> Result<A, E> + Send + 'static,
    {
        TryMemo {
            inner: Config::new_try_lazy(f),
        }
    }

    /// Gets the memoized result, computing on first access.
    pub fn get(&self) -> Result<&A, &E> {
        Config::force_try(&self.inner)
    }

    /// Creates a TryMemo from a TryEval.
    pub fn from_try_eval(eval: TryEval<'static, A, E>) -> Self {
        TryMemo::new(move || eval.run())
    }

    /// Creates a TryMemo from a TryTask.
    pub fn from_try_task(task: TryTask<A, E>) -> Self
    where
        A: 'static + Send,
        E: 'static + Send,
    {
        TryMemo::new(move || task.run())
    }
}
```

### Error Handling Strategies

#### Strategy 1: Let Panics Propagate (Default)

For `Eval` and `Memo`, panics propagate naturally. This follows Rust's "panics are bugs" philosophy.

```rust
let eval = Eval::new(|| panic!("oops"));
eval.run();  // Panics here, not caught
```

#### Strategy 2: Explicit Result Types

For expected failures, use `TryEval` and `TryMemo` with explicit error types.

```rust
let try_eval: TryEval<Config, ParseError> = TryEval::new(|| {
    parse_config_file()  // Returns Result<Config, ParseError>
});

match try_eval.run() {
    Ok(config) => ...,
    Err(e) => ...,  // Handle ParseError
}
```

#### Strategy 3: Opt-in Panic Catching

When you need to convert panics to errors, use `catch_unwind`:

```rust
let memo = TryMemo::<Data, String, RcConfig>::catch_unwind(|| {
    potentially_panicking_code()
});

match memo.force() {
    Ok(data) => ...,
    Err(panic_msg) => ...,  // Panic was caught and converted to String
}
```

### Lazy Recursive Structures

One powerful pattern is lazy recursive data structures. For potentially deep structures, use `Task` to ensure stack safety:

```rust
/// A lazy stream that computes elements on demand.
/// Uses Task for stack-safe recursive construction.
pub struct Stream<A> {
    head: A,
    tail: ArcMemo<Option<Stream<A>>>,
}

impl<A: Clone + Send + Sync + 'static> Stream<A> {
    /// Creates a finite stream from an iterator.
    /// Uses Task::defer for stack-safe lazy construction.
    pub fn from_iter<I: IntoIterator<Item = A> + Send + 'static>(iter: I) -> Option<Self> {
        let mut iter = iter.into_iter();
        iter.next().map(|head| {
            // Use Task for stack-safe deferred construction
            let tail = ArcMemo::from_task(
                Task::defer(move || {
                    Task::now(Self::from_iter(iter))
                })
            );
            Stream { head, tail }
        })
    }

    /// Maps a function over the stream lazily.
    pub fn map<B, F>(self, f: F) -> Stream<B>
    where
        B: Clone + Send + Sync + 'static,
        F: Fn(A) -> B + Clone + Send + Sync + 'static,
    {
        let f_clone = f.clone();
        Stream {
            head: f(self.head),
            tail: ArcMemo::from_task(
                Task::defer(move || {
                    Task::now(self.tail.get().clone().map(|t| t.map(f_clone)))
                })
            ),
        }
    }

    /// Takes the first n elements.
    /// Iterative, so no stack concerns.
    pub fn take(self, n: usize) -> Vec<A> {
        let mut result = Vec::with_capacity(n);
        let mut current = Some(self);

        for _ in 0..n {
            match current {
                Some(stream) => {
                    result.push(stream.head);
                    current = stream.tail.get().clone();
                }
                None => break,
            }
        }

        result
    }
}
```

## Tests

### Memo Tests

1.  **Caching**: Verify computation runs only once.
2.  **Sharing**: Verify clones share the cache.
3.  **Thread Safety**: Verify `ArcMemo` works across threads (compile check + runtime test).
4.  **Conversion**: Verify `from_task` and `from_eval` work.

### TryMemo Tests

1.  **Caching**: Verify result (success or error) is cached.
2.  **Sharing**: Verify clones share the result.

## Checklist

- [ ] Create `fp-library/src/types/memo.rs`
  - [ ] Implement `MemoConfig` trait
  - [ ] Implement `RcMemoConfig` and `ArcMemoConfig`
  - [ ] Implement `Memo` struct
  - [ ] Implement `new`, `get`
  - [ ] Implement `from_task` and `from_eval`
  - [ ] Implement `into_try<E>` conversion to `TryMemo`
  - [ ] Add type aliases `RcMemo`, `ArcMemo`
  - [ ] Add unit tests
- [ ] Create `fp-library/src/types/try_memo.rs`
  - [ ] Implement `TryMemo` struct
  - [ ] Implement `new`, `get`
  - [ ] Implement `from_try_task` and `from_try_eval`
  - [ ] Implement `catch_unwind` static factory method
  - [ ] Add type aliases `RcTryMemo`, `ArcTryMemo`
  - [ ] Add unit tests
- [ ] Update `fp-library/src/types.rs`
