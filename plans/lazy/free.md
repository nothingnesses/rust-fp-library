# Analysis: `free.rs`

**File:** `fp-library/src/types/free.rs`
**Role:** `Free<F, A>`, stack-safe free monad using Reflection without Remorse.

## Design

`Free<F, A>` implements the free monad with O(1) bind and stack-safe evaluation, based on the "Reflection without Remorse" technique (Atze van der Ploeg, Oleg Kiselyov, 2014).

```rust
struct Free<F, A> {
    view: Option<FreeView<F>>,
    continuations: CatList<Continuation<F>>,
    _marker: PhantomData<A>,
}
enum FreeView<F> { Return(TypeErasedValue), Suspend(F::Of<Free<F, TypeErasedValue>>) }
type Continuation<F> = Box<dyn FnOnce(TypeErasedValue) -> Free<F, TypeErasedValue>>;
type TypeErasedValue = Box<dyn Any>;
```

## Comparison with PureScript

| Aspect        | PureScript                                            | Rust                                                              |
| ------------- | ----------------------------------------------------- | ----------------------------------------------------------------- |
| FreeView      | `Return a / Bind (f b) (b -> Free f a)`               | `Return(Box<dyn Any>) / Suspend(F<Free<F, Box<dyn Any>>>)`        |
| Type erasure  | `unsafeCoerce` (zero-cost)                            | `Box<dyn Any>` + `downcast` (runtime check + allocation)          |
| Continuations | `CatList (ExpF f)` where `ExpF f = Val -> Free f Val` | `CatList<Box<dyn FnOnce(Box<dyn Any>) -> Free<F, Box<dyn Any>>>>` |
| Lifetime      | Any                                                   | `'static` only                                                    |

**Key simplification:** PureScript's `FreeView` has a `Bind(f b, b -> Free f a)` variant that pairs a functor layer with a distinguished continuation. Rust simplifies to just `Return`/`Suspend`, with ALL continuations uniformly in the CatList. This is cleaner; `bind` is always a CatList snoc regardless of the current view.

## Assessment

### Correct decisions

1. **Simplified FreeView.** Removing the `Bind` variant makes the data structure more uniform and the `to_view` loop simpler.
2. **`Box<dyn Any>` over `unsafe` coercion.** Trades performance for safety. All downcasts are guarded by internal invariants.
3. **Iterative `to_view` and `evaluate`.** No recursive calls in the evaluation path.
4. **Custom `Drop`.** Iteratively dismantles deep chains using a worklist, preventing stack overflow.
5. **`Extract` bound on `F`.** Ensures `Free` can only be used with single-valued functors, which is required for the CatList continuation model to work correctly.

### Issues

#### 1. Per-step allocation overhead

Every `pure` value is boxed (`Box::new(a) as Box<dyn Any>`). Every `bind` continuation boxes a closure. Every `to_view` call may allocate a downcast continuation. PureScript avoids all of this with zero-cost `unsafeCoerce`.

For a chain of `n` binds, there are O(n) allocations. For tight loops like `tail_rec_m` over millions of iterations, this could be a significant overhead.

**Impact:** Moderate. This is the fundamental cost of safe type erasure. Benchmarks should validate.

#### 2. `Cell::take()` pattern restricts to single-valued functors

In `to_view`'s `Suspend` case, the continuations are moved into a `Cell<Option<CatList>>` and taken via `.take()`. The closure passed to `F::map` calls `take()`, so it can only be called once. If the functor's `map` calls the closure more than once (e.g., for a multi-element functor like `Vec`), it panics at runtime.

The `Extract` bound already implies single-valued functors, making this consistent. However, the panic is a runtime check for a constraint that could be better enforced at compile time.

**Impact:** Low. The `Extract` bound provides compile-time enforcement. The runtime panic is a safety net.

#### 3. `erase_type` adds rebox overhead

`erase_type` appends a rebox continuation that wraps `Box<dyn Any>` in another `Box<dyn Any>`:

```rust
Box::new(val) as TypeErasedValue
```

This means values get an extra layer of boxing each time `erase_type` is called. PureScript avoids this with `unsafeCoerce`.

**Impact:** Low. `erase_type` is not called frequently in typical usage.

#### 4. Drop triggers thunk evaluation

The custom `Drop` calls `Extract::extract(fa)` on `Suspend` nodes to access the inner `Free`. For `ThunkBrand`, this means evaluating the thunk. Dropping a partially-built `Free` chain can trigger side effects.

**Impact:** Low-moderate. Necessary for correct resource cleanup but could surprise users.

#### 5. No HKT traits

`Free` cannot implement the library's `Kind` trait due to the `'static` requirement conflicting with lifetime-polymorphic HKT. `map`, `bind`, `pure` are inherent methods only.

**Impact:** Moderate. Fundamental limitation.

#### 6. `fold_free` requires `Clone` on the natural transformation

The step function in `tail_rec_m` is `Fn`, which requires the natural transformation to be cloneable for each iteration. This is an ergonomic limitation.

**Impact:** Low.

## Strengths

- Correct implementation of Reflection without Remorse.
- O(1) `bind` (CatList snoc).
- Guaranteed stack safety (iterative evaluation loop).
- Clean API: `pure`, `bind`, `map`, `lift_f`, `wrap`, `evaluate`, `resume`, `fold_free`, `hoist_free`, `substitute_free`.
- Stack-safe `Drop` implementation.
- Comprehensive tests: monad laws, stack safety (100k chains), drop safety, mixed construction patterns.
