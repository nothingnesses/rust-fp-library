# Step 2: Core Types

## Goal

Implement the core types `Step`, `Thunk`, and `Free` that form the building blocks for the stack-safe `Task` monad.

## Files to Create

- `fp-library/src/types/step.rs`
- `fp-library/src/types/thunk.rs`
- `fp-library/src/types/free.rs`

## Files to Modify

- `fp-library/src/types.rs`

## Implementation Details

### Step Type and MonadRec Trait

`Step` represents a step in a tail-recursive computation.

#### Design Rationale

The `Step` type and `MonadRec` trait are the foundation of stack-safe recursion. Rather than embedding trampolining logic into specific types, we define a generic interface that any monad can implement.

**Key insight from PureScript**:

```purescript
data Step a b = Loop a | Done b

class Monad m <= MonadRec m where
  tailRecM :: forall a b. (a -> m (Step a b)) -> a -> m b
```

The `tailRecM` function repeatedly applies `f` until it returns `Done`. The key constraint is that `m` must support this without growing the stack.

#### Step Type

````rust
/// Represents the result of a single step in a tail-recursive computation.
///
/// This type is fundamental to stack-safe recursion via `MonadRec`.
///
/// # Type Parameters
///
/// - `A`: The "loop" type - when we return `Loop(a)`, we continue with `a`
/// - `B`: The "done" type - when we return `Done(b)`, we're finished
///
/// # Example
///
/// ```rust
/// // Count down from n to 0, accumulating the sum
/// fn sum_to_zero(n: i32, acc: i32) -> Step<(i32, i32), i32> {
///     if n <= 0 {
///         Step::Done(acc)
///     } else {
///         Step::Loop((n - 1, acc + n))
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Step<A, B> {
    /// Continue the loop with a new value
    Loop(A),
    /// Finish the computation with a final value
    Done(B),
}

impl<A, B> Step<A, B> {
    /// Returns `true` if this is a `Loop` variant.
    #[inline]
    pub fn is_loop(&self) -> bool {
        matches!(self, Step::Loop(_))
    }

    /// Returns `true` if this is a `Done` variant.
    #[inline]
    pub fn is_done(&self) -> bool {
        matches!(self, Step::Done(_))
    }

    /// Maps a function over the `Loop` variant.
    pub fn map_loop<C>(self, f: impl FnOnce(A) -> C) -> Step<C, B> {
        match self {
            Step::Loop(a) => Step::Loop(f(a)),
            Step::Done(b) => Step::Done(b),
        }
    }

    /// Maps a function over the `Done` variant.
    pub fn map_done<C>(self, f: impl FnOnce(B) -> C) -> Step<A, C> {
        match self {
            Step::Loop(a) => Step::Loop(a),
            Step::Done(b) => Step::Done(f(b)),
        }
    }

    /// Applies functions to both variants (bifunctor map).
    pub fn bimap<C, D>(
        self,
        f: impl FnOnce(A) -> C,
        g: impl FnOnce(B) -> D,
    ) -> Step<C, D> {
        match self {
            Step::Loop(a) => Step::Loop(f(a)),
            Step::Done(b) => Step::Done(g(b)),
        }
    }
}
````

#### MonadRec Trait

````rust
use crate::{Apply, kinds::*};

/// A type class for monads that support stack-safe tail recursion.
///
/// Any monad implementing `MonadRec` guarantees that `tail_rec_m` will not
/// overflow the stack, regardless of how many iterations are required.
///
/// # Laws
///
/// 1. **Equivalence to recursion**: For a total function `f: A -> M<Step<A, B>>`,
///    `tail_rec_m(f, a)` should produce the same result as the (potentially
///    stack-overflowing) recursive definition:
///    ```text
///    rec(a) = f(a).bind(|step| match step {
///        Step::Loop(a') => rec(a'),
///        Step::Done(b) => pure(b),
///    })
///    ```
///
/// 2. **Stack safety**: `tail_rec_m` must not overflow the stack for any
///    terminating `f`, even with millions of iterations.
///
/// # Example
///
/// ```rust
/// use fp_library::{classes::MonadRec, types::Step};
///
/// // Factorial using tail recursion
/// fn factorial<M: MonadRec>(n: u64) -> Apply!(M, u64) {
///     M::tail_rec_m(|(n, acc)| {
///         if n <= 1 {
///             M::pure(Step::Done(acc))
///         } else {
///             M::pure(Step::Loop((n - 1, n * acc)))
///         }
///     }, (n, 1))
/// }
/// ```
pub trait MonadRec: Monad {
    /// Performs tail-recursive monadic computation.
    ///
    /// Repeatedly applies `f` to the current state until `f` returns `Done`.
    ///
    /// # Type Parameters
    ///
    /// - `A`: The loop state type
    /// - `B`: The final result type
    ///
    /// # Parameters
    ///
    /// - `f`: A function that takes the current state and returns a monadic
    ///        `Step`, either continuing with `Loop(a)` or finishing with `Done(b)`.
    ///        **Must be `Clone`** because the function is called multiple times
    ///        across recursive iterations, with each iteration potentially
    ///        needing its own owned copy of the closure.
    /// - `a`: The initial state
    ///
    /// # Returns
    ///
    /// A monadic value containing the final result `B`
    ///
    /// # Clone Bound Rationale
    ///
    /// The `Clone` bound on `F` is necessary because:
    /// 1. Each recursive step needs to pass `f` to the next iteration
    /// 2. In trampolined implementations, `f` must be moved into closures
    ///    multiple times (once per `defer` or continuation)
    /// 3. Most closures naturally implement `Clone` when their captures do
    ///
    /// For closures that cannot implement `Clone`, use `tail_rec_m_shared`
    /// which wraps `f` in `Arc` internally (with a small performance cost).
    fn tail_rec_m<'a, A: 'a, B: 'a, F>(
        f: F,
        a: A,
    ) -> Apply!(Self::Brand, B)
    where
        F: Fn(A) -> Apply!(Self::Brand, Step<A, B>) + Clone + 'a,
        Self::Brand: Kind_cdc7cd43dac7585f;  // type Of<'a, T: 'a>: 'a
}

/// Free function version of `tail_rec_m`.
pub fn tail_rec_m<'a, M, A: 'a, B: 'a, F>(
    f: F,
    a: A,
) -> Apply!(M::Brand, B)
where
    M: MonadRec,
    F: Fn(A) -> Apply!(M::Brand, Step<A, B>) + Clone + 'a,
    M::Brand: Kind_cdc7cd43dac7585f,
{
    M::tail_rec_m(f, a)
}

/// Arc-wrapped version of `tail_rec_m` for non-Clone closures.
///
/// This function wraps the provided closure in `Arc` internally, allowing
/// closures that don't implement `Clone` to be used with `tail_rec_m`.
///
/// # Trade-offs
///
/// - **Pro**: Works with any `Fn` closure, not just `Clone` ones
/// - **Con**: Small overhead from Arc allocation and atomic reference counting
///
/// # When to Use
///
/// Use this when your closure captures non-Clone state:
///
/// ```rust
/// // This closure captures a non-Clone Sender
/// let sender: Sender<i32> = /* ... */;
/// tail_rec_m_shared::<EvalBrand, _, _, _>(
///     |n| {
///         sender.send(n).ok();
///         if n == 0 { Eval::now(Step::Done(())) }
///         else { Eval::now(Step::Loop(n - 1)) }
///     },
///     100
/// )
/// ```
pub fn tail_rec_m_shared<'a, M, A: 'a, B: 'a, F>(
    f: F,
    a: A,
) -> Apply!(M::Brand, B)
where
    M: MonadRec,
    F: Fn(A) -> Apply!(M::Brand, Step<A, B>) + 'a,
    M::Brand: Kind_cdc7cd43dac7585f,
{
    use std::sync::Arc;

    // Wrap f in Arc to make it Clone
    let f = Arc::new(f);

    // Create a Clone wrapper that delegates to the Arc
    let wrapper = move |a: A| {
        let f = Arc::clone(&f);
        f(a)
    };

    M::tail_rec_m(wrapper, a)
}
````

#### Standard Implementations

**Identity (trivial case)**

```rust
impl MonadRec for IdentityInstance {
    fn tail_rec_m<'a, A: 'a, B: 'a>(
        f: impl Fn(A) -> Identity<Step<A, B>> + 'a,
        mut a: A,
    ) -> Identity<B> {
        loop {
            match f(a).0 {
                Step::Loop(next) => a = next,
                Step::Done(b) => return Identity(b),
            }
        }
    }
}
```

**Option**

```rust
impl MonadRec for OptionInstance {
    fn tail_rec_m<'a, A: 'a, B: 'a>(
        f: impl Fn(A) -> Option<Step<A, B>> + 'a,
        mut a: A,
    ) -> Option<B> {
        loop {
            match f(a)? {
                Step::Loop(next) => a = next,
                Step::Done(b) => return Some(b),
            }
        }
    }
}
```

**Result**

```rust
impl<E> MonadRec for ResultInstance<E> {
    fn tail_rec_m<'a, A: 'a, B: 'a>(
        f: impl Fn(A) -> Result<Step<A, B>, E> + 'a,
        mut a: A,
    ) -> Result<B, E> {
        loop {
            match f(a)? {
                Step::Loop(next) => a = next,
                Step::Done(b) => return Ok(b),
            }
        }
    }
}
```

#### Why MonadRec Matters

Without stack-safe recursion, this function overflows:

```rust
// Using Eval (NOT stack-safe) - this will overflow for large n!
fn countdown_eval(n: u64) -> Eval<'static, u64> {
    if n == 0 {
        Eval::pure(0)
    } else {
        Eval::new(move || countdown_eval(n - 1).run())  // Stack overflow!
    }
}
```

With `Task::tail_rec_m`, we achieve guaranteed stack safety:

```rust
// Using Task (stack-safe) - works for any n
fn countdown(n: u64) -> Task<u64> {
    Task::tail_rec_m(|n| {
        if n == 0 {
            Task::now(Step::Done(0))
        } else {
            Task::now(Step::Loop(n - 1))
        }
    }, n)
}
```

The key difference: instead of building a chain of deferred computations, we express the loop _structure_ explicitly with `Step`, and let `tail_rec_m` handle it iteratively.

#### Relationship to Trampoline

`Trampoline` is essentially `MonadRec` specialized to the "thunk" monad:

```rust
type Trampoline<A> = Free<ThunkF, A>;

// Trampoline::done(a) ≈ Free::Pure(a)
// Trampoline::suspend(f) ≈ Free::Roll(ThunkF(f))
```

In this proposal, **`Task`** serves as our `Trampoline`, using the Free monad with CatList-based bind stack for guaranteed stack safety. Note that `Task` does NOT implement the HKT-based `MonadRec` trait (due to `'static` constraint conflicts), but provides equivalent standalone `tail_rec_m` methods.

The separate **`Eval<'a, A>`** type (closure-based) CAN implement HKT traits including `MonadRec`, but is NOT stack-safe for deep recursion.

### Free Monad with CatList-Based Bind Stack

The Free monad implementation using `CatList` for O(1) binds.

- **Val**: `Box<dyn Any + Send>` (Type erasure).
- **Cont**: `Box<dyn FnOnce(Val) -> Free<F, Val> + Send>`.
- **Variants**:
  - `Pure(A)`
  - `Roll(Apply!(F::Brand, Free<F, A>))`
  - `Bind { head: Box<Free<F, Val>>, conts: CatList<Cont<F>> }`

#### Design Rationale

The Free monad provides a generic way to build a monad from any functor `F`. The key insight of "Reflection without Remorse" is that by storing continuations in a CatList instead of nesting them directly, we achieve O(1) bind performance.

**PureScript's Free monad structure**:

```purescript
data Free f a = Pure a | Free (f (Free f a)) | Bind (Free f Val) (CatList (Val -> Free f Val))
```

The `Bind` constructor stores:

1. A suspended computation producing some value (type-erased as `Val`)
2. A CatList of continuations to apply (also type-erased)

#### Type-Erased Value Type

Since Rust's type system cannot express existential types directly, we use `Box<dyn Any>` for type erasure:

```rust
use std::any::Any;

/// A type-erased value used internally by Free.
///
/// This is the equivalent of PureScript's `Val` type or the polymorphic
/// existential in the "Reflection without Remorse" paper.
pub type Val = Box<dyn Any + Send>;

/// A type-erased continuation: Val -> Free<F, Val>
pub type ErasedCont<F> = Box<dyn FnOnce(Val) -> Free<F, Val> + Send>;
```

#### Free Monad Implementation

````rust
use std::any::Any;
use std::marker::PhantomData;

/// A type-erased value for internal use.
type Val = Box<dyn Any + Send>;

/// A type-erased continuation.
type Cont<F> = Box<dyn FnOnce(Val) -> Free<F, Val> + Send>;

/// The Free monad with O(1) bind via CatList.
///
/// This implementation follows "Reflection without Remorse" to ensure
/// that left-associated binds do not degrade performance.
///
/// # Type Parameters
///
/// - `F`: The base functor (must implement `Functor`)
/// - `A`: The result type
///
/// # Variants
///
/// - `Pure(a)`: A finished computation with result `a`
/// - `Roll(f)`: A suspended computation `f` containing a `Free<F, A>`
/// - `Bind(free, conts)`: A computation `free` with continuations `conts`
///
/// # Example
///
/// ```rust
/// // ThunkF is () -> A, making Free<ThunkF, A> a Trampoline
/// let free = Free::pure(42)
///     .flat_map(|x| Free::pure(x + 1))
///     .flat_map(|x| Free::pure(x * 2));
///
/// assert_eq!(free.run(), 86);
/// ```
pub enum Free<F, A>
where
    F: Functor,
{
    /// A pure value, computation finished.
    Pure(A),

    /// A suspended effect containing a continuation.
    Roll(Apply!(F::Brand, Free<F, A>)),

    /// A computation with a CatList of continuations.
    /// Uses type erasure internally for heterogeneous continuation chains.
    Bind {
        /// The initial computation (type-erased)
        head: Box<Free<F, Val>>,
        /// The queue of continuations (type-erased)
        conts: CatList<Cont<F>>,
        /// Phantom data for the result type
        _marker: PhantomData<A>,
    },
}

impl<F: Functor, A: 'static + Send> Free<F, A> {
    /// Creates a pure Free value.
    #[inline]
    pub fn pure(a: A) -> Self {
        Free::Pure(a)
    }

    /// Creates a suspended computation from a functor value.
    pub fn roll(fa: Apply!(F::Brand, Free<F, A>)) -> Self {
        Free::Roll(fa)
    }

    /// Monadic bind (flatMap) with O(1) complexity.
    ///
    /// This is where the CatList magic happens: instead of nesting
    /// the continuation, we snoc it onto the CatList.
    pub fn flat_map<B: 'static + Send>(
        self,
        f: impl FnOnce(A) -> Free<F, B> + 'static + Send,
    ) -> Free<F, B> {
        // Type-erase the continuation
        let erased_f: Cont<F> = Box::new(move |val: Val| {
            let a: A = *val.downcast().expect("Type mismatch in Free::flat_map");
            let free_b: Free<F, B> = f(a);
            free_b.erase_type()
        });

        match self {
            // Pure: create a Bind with this continuation
            Free::Pure(a) => {
                let head: Free<F, Val> = Free::Pure(Box::new(a) as Val);
                Free::Bind {
                    head: Box::new(head),
                    conts: CatList::singleton(erased_f),
                    _marker: PhantomData,
                }
            }

            // Roll: wrap in a Bind
            Free::Roll(fa) => {
                let head = Free::Roll(fa).erase_type_boxed();
                Free::Bind {
                    head,
                    conts: CatList::singleton(erased_f),
                    _marker: PhantomData,
                }
            }

            // Bind: snoc the new continuation onto the CatList (O(1)!)
            Free::Bind { head, conts, .. } => {
                Free::Bind {
                    head,
                    conts: conts.snoc(erased_f),
                    _marker: PhantomData,
                }
            }
        }
    }

    /// Converts to type-erased form.
    fn erase_type(self) -> Free<F, Val> {
        match self {
            Free::Pure(a) => Free::Pure(Box::new(a) as Val),
            Free::Roll(fa) => {
                // Map over the functor to erase the inner type
                let erased = F::map(|inner: Free<F, A>| inner.erase_type(), fa);
                Free::Roll(erased)
            }
            Free::Bind { head, conts, .. } => Free::Bind {
                head,
                conts,
                _marker: PhantomData,
            },
        }
    }

    /// Converts to boxed type-erased form.
    fn erase_type_boxed(self) -> Box<Free<F, Val>> {
        Box::new(self.erase_type())
    }
}
````

### Thunk

A wrapper around a boxed closure `Box<dyn FnOnce() -> A + Send>`.

- **ThunkF**: A zero-sized struct representing the Functor for Thunk.
- **Runnable**: A trait for functors that can be "run" to produce a value.

#### The Run Loop (Interpreter)

The evaluation loop processes the Free structure iteratively:

```rust
impl<F, A> Free<F, A>
where
    F: Functor,
    A: 'static + Send,
{
    /// Executes the Free computation, returning the final result.
    ///
    /// This is the "trampoline" that iteratively processes the
    /// CatList of continuations without growing the stack.
    ///
    /// # Requirements
    ///
    /// `F` must be a "runnable" functor (e.g., ThunkF where we can
    /// force the thunk to get the inner value).
    pub fn run(self) -> A
    where
        F: Runnable,
    {
        // Start with a type-erased version
        let mut current: Free<F, Val> = self.erase_type();
        let mut conts: CatList<Cont<F>> = CatList::empty();

        loop {
            match current {
                Free::Pure(val) => {
                    // Try to apply the next continuation
                    match conts.uncons() {
                        Some((cont, rest)) => {
                            current = cont(val);
                            conts = rest;
                        }
                        None => {
                            // No more continuations - we're done!
                            return *val.downcast::<A>()
                                .expect("Type mismatch in Free::run final downcast");
                        }
                    }
                }

                Free::Roll(fa) => {
                    // Run the effect to get the inner Free
                    current = F::run_effect(fa);
                }

                Free::Bind { head, conts: inner_conts, .. } => {
                    // Merge the inner continuations with outer ones
                    // This is where CatList's O(1) append shines!
                    current = *head;
                    conts = inner_conts.append(conts);
                }
            }
        }
    }
}

/// A functor whose effects can be "run" to produce the inner value.
pub trait Runnable: Functor {
    /// Runs the effect, producing the inner value.
    fn run_effect<A>(fa: Apply!(Self::Brand, A)) -> A;
}

impl<F: Functor, A> Drop for Free<F, A> {
    fn drop(&mut self) {
        // Only `Bind` variants can cause deep recursion
        if let Free::Bind { head, .. } = self {
            // Create a dummy value to swap out the head
            let dummy: Val = Box::new(());
            // We need to cast this to `Free<F, Val>`.
            let dummy_free: Free<F, Val> = Free::Pure(dummy);

            let mut current_box = std::mem::replace(head, Box::new(dummy_free));

            loop {
                // `current_box` is now owned by the loop.
                // When it goes out of scope at end of iteration, it drops.
                // If it contains a `Bind`, that `Bind`'s head will drop recursively.
                // So we must extract the NEXT head before dropping `current_box`.

                if let Free::Bind { head: next_head, .. } = &mut *current_box {
                    // Create another dummy to swap
                    let dummy: Val = Box::new(());
                    let dummy_free: Free<F, Val> = Free::Pure(dummy);

                    // Swap next_head out, putting dummy in.
                    // Now `current_box` (the old head) has a dummy head.
                    // When `current_box` drops, it drops the dummy head (safe).
                    // We take ownership of the REAL next head.
                    let next_box = std::mem::replace(next_head, Box::new(dummy_free));

                    // Move to next
                    current_box = next_box;
                } else {
                    // Not a Bind, no recursion risk from head.
                    break;
                }
            }
        }
    }
}
```

#### ThunkF: The Thunk Functor

For `Eval`, we use `ThunkF` — a functor representing suspended computations:

```rust
/// A thunk functor: `() -> A`
///
/// This is the simplest functor for building a trampoline.
/// `Free<ThunkF, A>` is equivalent to PureScript's `Trampoline`.
pub struct ThunkF;

/// The concrete type for ThunkF applied to A.
pub struct Thunk<A>(Box<dyn FnOnce() -> A + Send>);

impl<A> Thunk<A> {
    pub fn new(f: impl FnOnce() -> A + Send + 'static) -> Self {
        Thunk(Box::new(f))
    }

    pub fn force(self) -> A {
        (self.0)()
    }
}

// Brand for HKT
pub struct ThunkFBrand;

impl Kind_cdc7cd43dac7585f for ThunkFBrand {
    type Of<'a, A: 'a> = Thunk<A>;
}

impl Functor for ThunkF {
    type Brand = ThunkFBrand;

    fn map<A, B>(f: impl FnOnce(A) -> B, fa: Thunk<A>) -> Thunk<B> {
        Thunk::new(move || f(fa.force()))
    }
}

impl Runnable for ThunkF {
    fn run_effect<A>(fa: Thunk<A>) -> A {
        fa.force()
    }
}
```

#### Why This Achieves O(1) Bind

Consider this sequence of binds:

```rust
Free::pure(0)
    .flat_map(|x| Free::pure(x + 1))
    .flat_map(|x| Free::pure(x + 2))
    .flat_map(|x| Free::pure(x + 3))
```

**Traditional nested structure** (O(n²)):

```
FlatMap(FlatMap(FlatMap(Pure(0), f1), f2), f3)
```

Concatenating requires traversing to the innermost `Pure`.

**CatList structure** (O(1)):

```
Bind {
    head: Pure(0),
    conts: CatList[f1, f2, f3]
}
```

Each `flat_map` just does `conts.snoc(f)` — O(1)!

#### Memory and Performance Considerations

**Allocation**:

- Each continuation is boxed: `Box<dyn FnOnce(Val) -> Free<F, Val>>`
- Each value is boxed for type erasure: `Box<dyn Any>`

**Downcasting**:

- `downcast` is a simple discriminant check + pointer cast
- Extremely cheap, but adds a small constant factor

**CatList overhead**:

- The nested CatList structure adds indirection
- But this is amortized across all operations

**When to use**:

- Use `Free`/`Eval` for _deep_ recursion or _long_ chains (1000+ binds)
- For shallow chains (<100 binds), direct closures may be faster
- The crossover point depends on the specific use case

## Tests

### Step Tests

1.  **Mapping**: `map_loop`, `map_done`, `bimap`.
2.  **State**: `is_loop`, `is_done`.

### Thunk Tests

1.  **Execution**: Create a thunk, force it, verify result.
2.  **Send**: Verify `Thunk` is `Send`.

### Free Tests

1.  **Pure**: `Free::pure(x).run()` returns `x`.
2.  **Roll**: `Free::roll(thunk).run()` executes thunk.
3.  **Bind**: `Free::pure(x).flat_map(f).run()` works.
4.  **Trampoline**: Verify `run()` loop handles `Bind` variants correctly without recursion.

## Checklist

- [ ] Create `fp-library/src/types/step.rs`
  - [ ] Implement `Step` enum
  - [ ] Implement helper methods
  - [ ] Add unit tests
- [ ] Create `fp-library/src/types/thunk.rs`
  - [ ] Implement `Thunk` struct
  - [ ] Implement `ThunkF` struct (marker)
  - [ ] Implement `Runnable` trait
  - [ ] Add unit tests
- [ ] Create `fp-library/src/types/free.rs`
  - [ ] Define `Val` and `Cont` types
  - [ ] Implement `Free` enum
  - [ ] Implement `pure`, `roll`, `flat_map` (with type erasure)
  - [ ] Implement `run` (trampoline loop)
  - [ ] Implement `Drop` to prevent stack overflow on drop
  - [ ] Add unit tests
- [ ] Update `fp-library/src/types.rs`
