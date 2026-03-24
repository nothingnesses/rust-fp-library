# Free Monad Analysis

File: `fp-library/src/types/free.rs`

## 1. Design

### Encoding

The Free monad uses the "Reflection without Remorse" encoding from Atze van der Ploeg and Oleg Kiselyov. The core data type has four variants:

- `Pure(A)` - a completed value.
- `Wrap(F<Free<F, A>>)` - a suspended computation in functor `F`.
- `Map { value, f }` - a deferred map operation (optimization to avoid type-erasure roundtrip).
- `Bind { head, continuations, _marker }` - a computation followed by a `CatList` of type-erased continuations.

This encoding is correct and well-established. The `CatList` (a deque-based catenable list) replaces the naive recursive `FreeT` structure to achieve O(1) `bind` for left-associated chains, avoiding the quadratic blowup that plagues naive Free monad implementations.

### Relationship to Trampoline

`Trampoline<A>` is a newtype around `Free<ThunkBrand, A>`. This is the standard relationship: a trampoline is the Free monad over a trivial "suspend" functor (thunks). The design correctly delegates `pure`, `bind`, `map`, `defer`, and `evaluate` to the underlying `Free`. `TryTrampoline` further wraps `Trampoline<Result<A, E>>`.

### Option wrapper

`Free<F, A>` is defined as `struct Free<F, A>(Option<FreeInner<F, A>>)`. The `Option` wrapper enables `take()`-based linear consumption, allowing `bind` and `evaluate` to move the inner value out without `unsafe`. This is a pragmatic Rust pattern for affine types. The downside is runtime panics if a `Free` value is consumed twice, but this is guarded by the library's ownership discipline.

## 2. Implementation Correctness

### The evaluation loop (`evaluate`)

The loop in `evaluate` is correct:

1. Type-erases `self` into `Free<F, TypeErasedValue>`.
2. Maintains an explicit `CatList<Continuation<F>>` stack.
3. On `Pure(val)`: applies the next continuation or, if none remain, downcasts to `A`.
4. On `Wrap(fa)`: calls `F::evaluate(fa)` to step the functor, producing a new `Free`.
5. On `Map { value, f }`: converts to a continuation and prepends to the stack.
6. On `Bind { head, continuations }`: appends inner continuations to the outer stack.

This is a faithful translation of the "Reflection without Remorse" run loop.

### The `bind` method

The `bind` implementation correctly handles all four variants:

- `Pure` and `Wrap` and `Map`: wrap in a `Bind` node with a singleton continuation.
- `Bind`: appends (`snoc`) the new continuation to the existing `CatList`.

The O(1) snoc on the `Bind` case is the key property that prevents left-associated bind chains from degrading to O(n^2).

### Type erasure

Type erasure via `Box<dyn Any>` with `downcast` is correct but inherently fragile. The `expect` calls on `downcast` are justified because a type mismatch would indicate an internal invariant violation, not a user error. The comments document this accurately.

### `resume`

The `resume` method correctly collapses `Bind`/`Map` chains iteratively until reaching a `Pure` or `Wrap`, then returns `Ok(a)` or `Err(F<Free<F, A>>)` respectively. The handling of remaining continuations when reaching `Wrap` is nuanced but correct: it uses `F::map` to attach continuations to the inner `Free` within the functor layer.

One subtle detail: the `Cell` trick in the `Wrap` branch with non-empty continuations is needed because `F::map` takes `impl Fn` (not `FnOnce`). This is safe because functors must call map exactly once per element, and the code panics if violated. This is documented.

### `erase_type`

The `erase_type` method handles all four variants. For `Bind`, it exploits the fact that the head and continuations are already type-erased (`Free<F, TypeErasedValue>` and `CatList<Continuation<F>>`), so it only needs to swap the `PhantomData` marker. This is correct.

### Drop implementation

The custom `Drop` iteratively walks through nested `Bind`/`Map` chains to prevent stack overflow during destruction of deep chains. This is necessary because a chain of 100k nested `Bind` nodes would otherwise overflow the stack during recursive drop. The test `test_free_drop_safety` validates this.

However, there is a potential gap: the `Drop` implementation only walks through `Bind` and `Map` chains but does not handle `Wrap` variants that might contain deeply nested `Free` values inside the functor. For `ThunkBrand` this is likely fine because the thunk's closure is a single allocation, but for other functors (e.g., a list functor), deeply nested `Wrap` chains could still overflow during drop.

### Potential bug: `Map` variant in `erase_type`

In `erase_type`, the `Map` case composes `map_fn` with boxing:

```rust
Box::new(move |val: TypeErasedValue| {
    Box::new(map_fn(val)) as TypeErasedValue
})
```

This is correct: `map_fn: Box<dyn FnOnce(TypeErasedValue) -> A>` produces an `A`, which gets boxed into `TypeErasedValue`. The types align.

## 3. Stack Safety

### `evaluate` is stack-safe

The evaluation loop is iterative (a `loop {}` with `match`), not recursive. The `CatList::append` and `CatList::uncons` operations are O(1) amortized. When a `Wrap` variant is encountered, `F::evaluate` is called, which for `ThunkBrand` simply runs a closure and returns a new `Free`. This means each iteration of the loop consumes one layer of the computation without growing the call stack.

The critical insight: even deeply nested `bind` chains are flattened by the `Bind` case in the loop, which splices inner continuations onto the outer continuation list in O(1) time via `CatList::append`.

### `resume` is stack-safe

The `resume` method also uses an iterative loop with the same pattern, so it is stack-safe.

### `fold_free` is NOT stack-safe

`fold_free` uses actual recursion:

```rust
G::bind(ga, move |inner_free: Free<F, A>| {
    inner_free.fold_free::<G>(nt2.clone())
})
```

For each `Wrap` layer, `fold_free` calls `G::bind` with a closure that recursively calls `fold_free`. Whether this overflows depends on the target monad `G`:

- If `G` is `OptionBrand` or similar strict monads, the recursive call happens immediately inside `bind`, growing the call stack proportionally to the number of `Wrap` layers.
- If `G` is a lazy/trampolining monad, the recursion is deferred.

For deep Free computations with many `Wrap` layers (rather than `Bind` chains), `fold_free` can overflow. The `resume` call inside `fold_free` does collapse `Bind` chains, but each `Wrap` layer produces one level of actual recursion.

This is a known limitation of the standard `foldFree` pattern. PureScript's implementation has the same structure but avoids the problem because its runtime uses constant-size stack frames. In Rust, this is a real concern.

### `Trampoline::defer` is the key to stack-safe recursion

The `defer` method wraps construction in a `Thunk`, ensuring recursive calls don't build the computation eagerly on the call stack. Instead, each recursive step becomes a `Wrap` node that is only evaluated during the iterative `evaluate` loop.

## 4. Consistency with Library Patterns

### HKT exclusion

The file correctly documents why `Free` cannot implement the library's HKT traits: `Box<dyn Any>` requires `'static`, but `Kind::Of<'a, A>` must accept any lifetime `'a`. This is a fundamental tension and the implementation makes the right tradeoff (stack safety over HKT compatibility).

### Documentation style

The documentation follows the library's conventions: `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]` macros are used consistently. Module-level docs include comparison with PureScript. Examples use assertions.

### Evaluable trait

The `Evaluable` trait used by `Free::evaluate` is a clean abstraction. It provides a natural transformation `F ~> Id`, which is exactly what is needed to step through `Wrap` layers. Only `ThunkBrand` implements it, which is correct since that is the only functor intended for direct evaluation.

### Deferrable trait

`Free<ThunkBrand, A>` implements `Deferrable<'static>`, consistent with the library's pattern for deferred computation types.

### Test coverage

Tests cover: `pure`, `wrap`, `bind`, stack safety (100k iterations), drop safety, `bind` on `Wrap`, `lift_f`, `resume` (all variants), `fold_free` (pure, wrap, chain), `map` (pure, chain, wrap, interop with bind). Coverage is thorough.

## 5. Limitations

### `'static` requirement

All values in `Free` must be `'static` due to `Box<dyn Any>`. This means `Free` cannot work with borrowed data, making it unsuitable for many Rust idioms. This is inherent to the type-erasure approach and is well-documented.

### No `Send`/`Sync`

`Free` contains `Box<dyn FnOnce(...)>` continuations and `Box<dyn Any>`, neither of which is `Send`. This means `Free` (and by extension `Trampoline`) cannot be used across threads. The `Trampoline::memoize_arc` method works around this by eagerly evaluating before wrapping in `ArcLazy`, but the computation itself cannot be sent across threads.

### `fold_free` stack safety

As discussed above, `fold_free` with strict target monads is not stack-safe for deep computations. This could be addressed by:

1. Making `fold_free` iterative (complex, requires the target monad to support an iterative interpretation protocol).
2. Documenting the limitation explicitly (current docs do not warn about this).
3. Providing a `fold_free_trampoline` variant that specifically targets `Trampoline` as the output monad, where the recursion is deferred.

### No `hoistFree`

The documentation notes that `hoistFree` (which transforms the functor layer via a natural transformation without interpreting) is missing. This limits the ability to transform between different effect types without fully evaluating.

### `Map` variant complexity

The `Map` variant is an optimization but adds complexity to every method that pattern-matches on `FreeInner` (evaluate, resume, bind, erase_type, drop). An alternative would be to implement `map` via `bind`, accepting the type-erasure overhead. The current approach is a reasonable performance tradeoff but increases maintenance burden.

### `CatList` overhead for short chains

For computations with few bind operations, the `CatList`-based approach has higher constant overhead than a simple recursive structure. The `VecDeque` allocation inside `CatList::Cons` is non-trivial. For the library's primary use case (deep recursion), this is the right tradeoff.

## 6. Alternatives

### Other Rust FP libraries

- **`frunk`**: Does not provide a Free monad.
- **`cats-rs`** (archived): Had a basic Free monad without "Reflection without Remorse" optimization.
- **`tramp`/`tailcall` crates**: Provide standalone trampolines without the Free monad abstraction, typically as simple `enum { Done(A), More(Box<dyn FnOnce() -> Self>) }`. These are simpler but lack O(1) bind.
- **`recursion` crate**: Provides stack-safe recursion schemes but uses a different approach (explicit recursion scheme combinators rather than Free monads).

This library's approach is among the most complete Rust implementations of Free monad with the "Reflection without Remorse" optimization.

### Alternative: Continuation-Passing Style (CPS)

A CPS-based Free monad (`Codensity` transformation) can also achieve O(1) left-associated bind without `CatList`. However, CPS in Rust requires boxing closures, and the resulting type is harder to inspect (no `resume`). The `CatList` approach is more concrete and inspectable.

### Alternative: GAT-based Free (no type erasure)

With Generic Associated Types (stable since Rust 1.65), one could potentially encode the Free monad without `dyn Any` by using GATs to express the continuation type. However, this would require the continuation chain to be homogeneous in type, which is incompatible with heterogeneous bind chains. Type erasure remains necessary for the general case.

### Alternative: Iterative `fold_free`

The stack-unsafe `fold_free` could potentially be made iterative if the target monad `G` exposes a way to "step" its bind operation. This would require a trait like:

```rust
trait IterableMonad: Monad {
    fn step<A, B>(ga: G<A>, f: impl FnOnce(A) -> G<B>) -> Either<G<A>, G<B>>;
}
```

This is complex and unlikely worth the effort for the current use cases.

## 7. Documentation

### Strengths

- Module-level docs provide excellent context: comparison with PureScript, capabilities/limitations, lifetime discussion.
- The "What it CAN do" / "What it CANNOT do" section is very helpful.
- Each method has examples with assertions.
- The HKT limitation section in the struct docs is thorough and well-reasoned.

### Issues

- The `fold_free` docs do not warn about stack safety with strict target monads. Given that `evaluate` is documented as stack-safe, users may reasonably expect `fold_free` to be as well.
- The module docs reference `Runnable` trait (line 40: "because `DatabaseOp` must implement a single `Runnable` trait"), but the actual trait is `Evaluable`. This appears to be stale documentation.
- Test comment on line 969 references `Free::roll`, but the method is named `Free::wrap`. This appears to be a leftover from an earlier API.
- The `SAFETY` comments on `downcast().expect()` calls are not actually `unsafe` blocks, so the word "SAFETY" is misleading. These are invariant-preservation comments, not memory safety comments. A label like "INVARIANT" would be more precise.

### Missing documentation

- No doc-comment on the `Map` variant explaining why it exists as an optimization (only an inline comment).
- No mention of `Send`/thread-safety limitations in the module docs.
- `fold_free` should note it uses actual recursion and may overflow for deep `Wrap` chains with strict target monads.

## Summary

The Free monad implementation is well-designed, following the established "Reflection without Remorse" technique. The core operations (`pure`, `bind`, `map`, `evaluate`) are correct and stack-safe. The `CatList`-based continuation queue provides O(1) bind. The main concerns are:

1. **`fold_free` is not stack-safe** for strict target monads (undocumented).
2. **Stale documentation references** (`Runnable` instead of `Evaluable`, `roll` instead of `wrap`).
3. **Drop safety may be incomplete** for deeply nested `Wrap` chains with non-trivial functors.
4. **No `Send`/`Sync` support**, limiting use in concurrent contexts.

These are largely documentation and edge-case issues. The core design and implementation are sound.
