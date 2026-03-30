# Analysis: `trampoline.rs`

**File:** `fp-library/src/types/trampoline.rs`
**Role:** `Trampoline<A>`, stack-safe recursion via `Free<ThunkBrand, A>`.

## Design

`Trampoline<A: 'static>` is a newtype over `Free<ThunkBrand, A>`. It provides stack-safe monadic recursion by wrapping recursive calls in `Thunk` suspensions that are evaluated iteratively.

Key properties:

- **Stack-safe**: Guaranteed by `Free`'s iterative evaluation loop.
- **O(1) bind**: Via `Free`'s `CatList`-based continuation queue.
- **`'static` required**: Inherited from `Free`'s `Box<dyn Any>` type erasure.
- **Not `Send`**: `Thunk` is `!Send`, so `Trampoline` is `!Send`.
- **Not memoized**: Each evaluation recomputes from the beginning.

## Relationship to Free

`Trampoline` is a specialization of `Free` where the functor is `ThunkBrand`. The `defer` method wraps a recursive call in `Free::wrap(Thunk::new(...))`, creating a `Suspend` node that is evaluated lazily. The `evaluate` method delegates directly to `Free::evaluate`, which iteratively peels `Suspend` nodes by extracting the `Thunk`.

## Assessment

### Correct decisions

1. **Newtype over `Free`, not type alias.** Provides encapsulation and a focused API. Users interact with `Trampoline` methods rather than raw `Free` internals.
2. **`defer` as the primary recursion primitive.** Wraps `f` in a `Thunk` -> `Suspend` node, ensuring the recursive function returns immediately.
3. **`resume` for custom interpreters.** Allows stepping through the computation one layer at a time.
4. **Comprehensive conversion ecosystem.** `From<Lazy>`, `From<Thunk>`, `From<TryThunk>` etc.

### Issues

#### 1. No HKT brand

`Trampoline` cannot implement `Kind` because the `'static` requirement conflicts with lifetime-polymorphic HKT trait signatures (`Of<'a, A: 'a>`). This means generic code written against `Functor`/`Monad` cannot be instantiated with `Trampoline`.

**Impact:** Moderate. This is fundamental and unavoidable. `ThunkBrand` (for `Thunk`) fills the HKT role in the hierarchy.

#### 2. Heap allocation overhead per step

Every `pure` value is boxed via `Box::new(a) as Box<dyn Any>`, and every `bind` continuation boxes a closure and performs a runtime `downcast`. PureScript avoids this with `unsafeCoerce`. For tight loops (e.g., `tail_rec_m` over millions of iterations), this could have meaningful overhead compared to manual trampolining.

**Impact:** Moderate. This is the price of safe type erasure in Rust. Benchmarks should validate the overhead.

#### 3. `tail_rec_m` requires `Clone` on the step function

The step function must be `impl Fn(S) -> ... + Clone + 'static` because each iteration moves the closure into a new `defer`/`bind` step. The `arc_tail_rec_m` variant wraps in `Arc` to avoid requiring `Clone`, but at the cost of `Send + Sync` bounds.

**Impact:** Low. Pragmatic Rust-specific limitation.

#### 4. `From<Lazy> for Trampoline` requires `A: Clone`

Converting a `Lazy` to a `Trampoline` calls `lazy.evaluate().clone()`, adding a clone. The memoization benefit is partially lost since `Trampoline` is not memoized.

**Impact:** Low. Correctly documented.

#### 5. Custom `Drop` on `Free` may trigger thunk evaluation

The `Free::Drop` implementation calls `Extract::extract(fa)` on `Suspend` nodes to get the inner `Free`. For `ThunkBrand`, this evaluates the thunk. If the thunk has side effects or is expensive, dropping a `Trampoline` triggers unexpected computation.

**Impact:** Low-moderate. Necessary for stack-safe dropping, but the documentation does not explicitly call out this behavior.

## Strengths

- Genuine stack safety through iterative evaluation.
- O(1) bind via CatList.
- Clean API: `pure`, `defer`, `bind`, `map`, `evaluate`, `resume`.
- Comprehensive tests including QuickCheck monad laws and stack safety tests (100k+ chains).
- Well-documented `'static` constraint rationale.
