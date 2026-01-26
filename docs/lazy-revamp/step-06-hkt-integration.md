# Step 06: HKT Integration

## Goal
Integrate the new types with the library's Higher-Kinded Types (HKT) system. This involves defining brands, implementing the `MonadRec` trait, and implementing standard type classes (`Functor`, `Monad`, etc.) for `Eval` and `Memo`.

## Important: Task Does NOT Get HKT Integration

**`Task<A>` cannot implement HKT traits** due to a fundamental conflict:

- HKT trait methods require working for any lifetime `'a` (e.g., `fn bind<'a, A: 'a, B: 'a, ...>`)
- Task requires `A: 'static` due to type erasure via `Box<dyn Any>`

These constraints are mutually exclusive. `'static` is a specific lifetime, not "any `'a`". Therefore:

- **No `TaskBrand`** is defined
- Task provides standalone `tail_rec_m` methods instead of implementing the `MonadRec` trait
- Use `Eval<'a, A>` when you need HKT polymorphism
- Use `Task<A>` when you need guaranteed stack safety

## Files to Create
- `fp-library/src/classes/monad_rec.rs`
- `fp-library/src/classes/ref_functor.rs` (new trait for reference-returning functors)

## Files to Modify
- `fp-library/src/brands.rs`
- `fp-library/src/classes.rs`
- `fp-library/src/types/eval.rs` (to add trait impls)
- `fp-library/src/types/memo.rs` (to add trait impls)
- `fp-library/src/types/thunk.rs` (to add trait impls)
- `fp-library/src/types/free.rs` (to add trait impls)

## Implementation Details

### Brands
Define marker types for HKT dispatch.
- `EvalBrand`: For `Eval<'a, A>`.
- `ThunkFBrand`: For `Thunk<A>`.
- `FreeBrand<F>`: For `Free<F, A>`.
- `MemoBrand<Config>`: For `Memo<A, Config>`.

**Note**: No `TaskBrand` or `TryTaskBrand` — Task cannot implement HKT traits.

### EvalBrand and HKT Integration

We define `EvalBrand` for the **closure-based** `Eval<'a, A>`:

```rust
/// Brand type for the closure-based Eval in the HKT system.
///
/// Note: This is for Eval<'a, A>, NOT for Task<A>.
/// Task cannot implement HKT traits due to its 'static requirement.
pub struct EvalBrand;

impl_kind! {
    for EvalBrand {
        // The lifetime 'a flows through to Eval<'a, A>
        type Of<'a, A: 'a>: 'a = Eval<'a, A>;
    }
}
```

### Functor Implementation for Eval

```rust
impl Functor for EvalBrand {
    /// Maps a function over the result of an Eval computation.
    ///
    /// ### Type Signature
    ///
    /// `forall b a. Functor Eval => (a -> b, Eval a) -> Eval b`
    fn map<'a, B: 'a, A: 'a, F>(
        f: F,
        fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
    where
        F: Fn(A) -> B + 'a,
    {
        fa.map(f)  // No Send bound needed - Eval<'a, A> is flexible
    }
}
```

### Pointed Implementation for Eval

```rust
impl Pointed for EvalBrand {
    /// Wraps a value in an Eval context.
    ///
    /// ### Type Signature
    ///
    /// `forall a. Pointed Eval => a -> Eval a`
    fn pure<'a, A: 'a>(
        a: A
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
    {
        Eval::pure(a)  // No Send bound needed
    }
}
```

### Semimonad Implementation for Eval

```rust
impl Semimonad for EvalBrand {
    /// Chains Eval computations.
    ///
    /// ### Type Signature
    ///
    /// `forall b a. Semimonad Eval => (Eval a, a -> Eval b) -> Eval b`
    fn bind<'a, B: 'a, A: 'a, F>(
        ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
        f: F,
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
    where
        F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
    {
        // Closure-based flat_map works for any 'a, not just 'static
        ma.flat_map(move |a| f(a))
    }
}
```

### RefFunctor Trait
A variant of `Functor` for types where `map` receives/returns references. Required because `Memo::get()` returns `&A`, not `A`.

```rust
pub trait RefFunctor {
    fn map_ref<'a, B: 'a, A: 'a, F>(f: F, fa: Self::Of<'a, A>) -> Self::Of<'a, B>
    where
        F: FnOnce(&A) -> B + 'a;
}
```

### MonadRec Trait Definition

A trait for monads supporting tail recursion.

The `MonadRec` trait follows the project's HKT patterns. **Note**: `Eval` can implement this trait for HKT polymorphism, but for truly stack-safe deep recursion, use `Task::tail_rec_m` directly:

```rust
use crate::{Apply, kinds::*, classes::Monad};

/// A type class for monads that support stack-safe tail recursion.
///
/// ### Important Design Note
///
/// `Eval<'a, A>` CAN implement this trait (HKT-compatible).
/// `Task<A>` CANNOT implement this trait (requires `'static`).
///
/// For deep recursion (10,000+ calls), prefer `Task::tail_rec_m` which is
/// guaranteed stack-safe. `Eval`'s trait-based `tail_rec_m` will overflow
/// the stack at ~8000 recursive calls.
///
/// ### Laws
///
/// 1. **Equivalence**: `tail_rec_m(f, a)` produces the same result as the
///    recursive definition.
///
/// 2. **Safety varies**: Eval is NOT stack-safe for deep recursion.
///    Use Task for guaranteed stack safety.
pub trait MonadRec: Monad {
    /// Performs tail-recursive monadic computation.
    fn tail_rec_m<'a, A: 'a, B: 'a, F>(
        f: F,
        a: A,
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
    where
        F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>) + Clone + 'a;
}

/// Free function version of tail_rec_m.
pub fn tail_rec_m<'a, Brand, A: 'a, B: 'a, F>(
    f: F,
    a: A,
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
    Brand: MonadRec,
    F: Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>) + 'a,
{
    Brand::tail_rec_m(f, a)
}
```

#### Clone Bound Rationale

The `Clone` bound on `F` is necessary because:
1. Each recursive step needs to pass `f` to the next iteration
2. In trampolined implementations, `f` must be moved into closures multiple times (once per `defer` or continuation)
3. Most closures naturally implement `Clone` when their captures do

For closures that cannot implement `Clone`, use `tail_rec_m_shared` which wraps `f` in `Arc` internally (with a small performance cost).

### Trait Implementations
- **Eval**: `Functor`, `Pointed`, `Semimonad`, `Monad`, `MonadRec`, `Foldable`.
- **Memo**: `RefFunctor` (since `get` returns reference).
- **ThunkF**: `Functor`.
- **Free**: `Functor`, `Pointed`, `Semimonad`, `Monad`.

**Note**: `Eval`'s `MonadRec` implementation is NOT stack-safe for deep recursion (~8000 call limit). For truly stack-safe deep recursion, use `Task::tail_rec_m` directly.

### MonadRec Implementation for OptionBrand

```rust
impl MonadRec for OptionBrand {
    fn tail_rec_m<'a, A: 'a, B: 'a, F>(
        f: F,
        mut a: A,
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
    where
        F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>) + 'a,
    {
        loop {
            match f(a)? {
                Step::Loop(next) => a = next,
                Step::Done(b) => return Some(b),
            }
        }
    }
}
```

### MonadRec Implementation for EvalBrand

```rust
impl MonadRec for EvalBrand {
    fn tail_rec_m<'a, A: 'a, B: 'a, F>(
        f: F,
        initial: A,
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
    where
        F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>) + Clone + 'a,
    {
        // Use defer for trampolining.
        // The Clone bound allows us to clone `f` for each recursive step.
        fn go<'a, A, B, F>(
            f: F,
            a: A,
        ) -> Eval<'a, B>
        where
            F: Fn(A) -> Eval<'a, Step<A, B>> + Clone + 'a,
            A: 'a,
            B: 'a,
        {
            let f_clone = f.clone();  // Clone for the recursive call
            Eval::defer(move || {
                f(a).flat_map(move |step| match step {
                    Step::Loop(next) => go(f_clone.clone(), next),
                    Step::Done(b) => Eval::pure(b),
                })
            })
        }

        go(f, initial)
    }
}
```

### Foldable Implementation for Eval

```rust
impl Foldable for EvalBrand {
    fn fold_right<'a, FnBrand, B: 'a, A: 'a, Func>(
        func: Func,
        initial: B,
        fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    ) -> B
    where
        Func: Fn(A, B) -> B + 'a,
        FnBrand: CloneableFn + 'a,
        A: Send,
    {
        func(fa.run(), initial)
    }

    fn fold_left<'a, FnBrand, B: 'a, A: 'a, Func>(
        func: Func,
        initial: B,
        fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    ) -> B
    where
        Func: Fn(B, A) -> B + 'a,
        FnBrand: CloneableFn + 'a,
        A: Send,
    {
        func(initial, fa.run())
    }

    fn fold_map<'a, FnBrand, M, A: 'a, Func>(
        func: Func,
        fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    ) -> M
    where
        M: Monoid + 'a,
        Func: Fn(A) -> M + 'a,
        FnBrand: CloneableFn + 'a,
        A: Send,
    {
        func(fa.run())
    }
}
```

### ThunkF Brand and Functor

```rust
/// Brand type for ThunkF - the functor underlying trampolining.
pub struct ThunkFBrand;

impl_kind! {
    for ThunkFBrand {
        type Of<'a, A: 'a>: 'a = Thunk<A> where A: Send;
    }
}

impl Functor for ThunkFBrand {
    fn map<'a, B: 'a, A: 'a, F>(
        f: F,
        fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
    where
        F: Fn(A) -> B + 'a,
        A: Send,
        B: Send,
    {
        Thunk::new(move || f(fa.force()))
    }
}
```

### FreeBrand (Higher-Kinded Free)

The `Free` monad is itself parameterized by a functor. To represent this in the HKT system, we use a "curried" brand:

```rust
/// Brand for Free monad parameterized by a functor brand.
pub struct FreeBrand<FBrand>(PhantomData<FBrand>);

impl<FBrand: Functor> Kind_cdc7cd43dac7585f for FreeBrand<FBrand> {
    type Of<'a, A: 'a> = Free<FBrand, A>
    where
        A: Send;
}
```

This allows writing generic code over any Free monad:

```rust
fn lift_free<'a, FBrand: Functor, A: 'a + Send>(
    fa: Apply!(<FBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
) -> Apply!(<FreeBrand<FBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
    Free::roll(FBrand::map(Free::pure, fa))
}
```

### Example: Generic Stack-Safe Algorithm

With the HKT integration, we can write generic stack-safe algorithms:

```rust
/// Folds a list using a monadic function, stack-safely.
fn fold_m<'a, M, A, B, F>(
    xs: Vec<A>,
    init: B,
    f: F,
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
    M: MonadRec,
    A: 'a + Clone,
    B: 'a + Clone,
    F: Fn(B, A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a + Clone,
{
    M::tail_rec_m(
        move |(mut xs, acc): (Vec<A>, B)| {
            if xs.is_empty() {
                M::pure(Step::Done(acc))
            } else {
                let head = xs.remove(0);
                M::bind(f.clone()(acc, head), move |new_acc| {
                    M::pure(Step::Loop((xs, new_acc)))
                })
            }
        },
        (xs, init),
    )
}

// Usage with Eval
let result: Eval<i64> = fold_m::<EvalBrand, _, _, _>(
    vec![1, 2, 3, 4, 5],
    0i64,
    |acc, x| Eval::now(acc + x),
);
assert_eq!(result.run(), 15);

// Usage with Option
let result: Option<i64> = fold_m::<OptionBrand, _, _, _>(
    vec![1, 2, 3, 4, 5],
    0i64,
    |acc, x| Some(acc + x),
);
assert_eq!(result, Some(15));
```

## Tests

### HKT Tests
1.  **Generic Functions**: Write a function generic over `Monad` and use it with `Eval`.
2.  **MonadRec**: Verify `Eval` implements `MonadRec` (even if not stack-safe for deep recursion, it should work for shallow).
3.  **RefFunctor**: Verify `Memo` works with `RefFunctor`.

## Checklist
- [ ] Update `fp-library/src/brands.rs`
    - [ ] Add `EvalBrand`
    - [ ] Add `ThunkFBrand`
    - [ ] Add `FreeBrand`
    - [ ] Add `MemoBrand`
    - [ ] Document: NO `TaskBrand` (Task cannot implement HKT traits due to `'static` requirement)
- [ ] Create `fp-library/src/classes/ref_functor.rs`
    - [ ] Define `RefFunctor` trait for reference-returning map operations
    - [ ] Add documentation explaining why standard `Functor` doesn't work for `Memo`
- [ ] Create `fp-library/src/classes/monad_rec.rs`
    - [ ] Define `MonadRec` trait with `Clone` bound on `F`
    - [ ] Document Clone bound rationale in doc comments
    - [ ] Implement `tail_rec_m` free function
    - [ ] Implement `tail_rec_m_shared` (Arc-wrapped for non-Clone closures)
- [ ] Update `fp-library/src/classes.rs` to export `monad_rec` and `ref_functor`
- [ ] Implement traits for `Eval` in `src/types/eval.rs`
    - [ ] `Functor`, `Pointed`, `Semimonad`, `Monad`
    - [ ] `MonadRec` (note: NOT stack-safe for deep recursion)
    - [ ] `Foldable` (with `fold_right`, `fold_left`, `fold_map`)
- [ ] Implement traits for `Memo` in `src/types/memo.rs`
    - [ ] `RefFunctor`
- [ ] Implement traits for `Thunk` in `src/types/thunk.rs`
    - [ ] `Functor`
- [ ] Implement traits for `Free` in `src/types/free.rs`
    - [ ] `Functor`, `Pointed`, `Semimonad`, `Monad`

## Scope Notes

The following types intentionally do NOT get HKT brands:
- `Task<A>` / `TryTask<A, E>` — requires `'static`, incompatible with HKT's `for<'a>` bounds
- `TryEval<'a, A, E>` / `TryMemo<A, E, Config>` — fallible variants are out of scope for initial HKT integration
