# Free Monad Analysis

## 1. Type Design

### Core Representation

The Rust `Free` monad uses a three-variant enum `FreeInner<F, A>`:

```rust
enum FreeInner<F, A> {
    Pure(A),
    Wrap(F::Of<'static, Free<F, A>>),
    Bind {
        head: Box<Free<F, TypeErasedValue>>,
        continuations: CatList<Continuation<F>>,
        _marker: PhantomData<A>,
    },
}
```

Where:
- `TypeErasedValue = Box<dyn Any>`
- `Continuation<F> = Box<dyn FnOnce(TypeErasedValue) -> Free<F, TypeErasedValue>>`

The public type `Free<F, A>` wraps this in an `Option`:

```rust
pub struct Free<F, A>(Option<FreeInner<F, A>>);
```

The `Option` wrapper enables a linear consumption pattern: methods like `bind`, `erase_type`, and `evaluate` call `take_inner()` to move the `FreeInner` out. This avoids needing `unsafe` to move out of `&mut self` while keeping the struct in a valid (though empty) state.

### Comparison to PureScript's Representation

PureScript uses:

```purescript
data Free f a = Free (FreeView f Val Val) (CatList (ExpF f))
data FreeView f a b = Return a | Bind (f b) (b -> Free f a)
data Val
newtype ExpF f = ExpF (Val -> Free f Val)
```

PureScript's approach is more compact: it uses a single `Free` constructor that always pairs a `FreeView` with a `CatList`. The Rust version uses three distinct variants (`Pure`, `Wrap`, `Bind`), which is structurally different but semantically equivalent. The Rust approach makes the three states explicit, whereas PureScript encodes them by distinguishing an empty CatList from a non-empty one.

Key structural difference: PureScript stores the CatList alongside every Free value (even Pure ones, where it is empty). Rust only attaches a CatList to the `Bind` variant, making `Pure` and `Wrap` smaller. This is a reasonable optimization.

### Type Erasure Strategy

PureScript uses `unsafeCoerce` and a phantom `Val` type to erase types in the continuation queue. Rust uses `Box<dyn Any>` with runtime `downcast()`. Both are performing the same logical operation (hiding the intermediate types so continuations can be stored homogeneously in the CatList), but with different safety trade-offs:

- PureScript's `unsafeCoerce` is zero-cost but entirely unchecked.
- Rust's `Box<dyn Any>` adds a heap allocation per erased value and a runtime type check per `downcast`, but provides a runtime safety net. A type mismatch produces a panic with a meaningful message rather than undefined behavior.

The `dyn Any` approach is what forces the `'static` constraint, since `Any: 'static`.

## 2. "Reflection without Remorse"

### Correctness of the Technique

The "Reflection without Remorse" technique addresses the quadratic complexity of left-associated binds in naive Free monad implementations. The core insight: instead of nesting continuations (which produces a left-leaning tree that must be traversed on each step), store them in a flat queue.

The Rust implementation achieves this correctly:

1. **O(1) `bind`**: When binding on a `Bind` variant, the new continuation is appended to the existing `CatList` via `snoc`, which is O(1). When binding on `Pure` or `Wrap`, a new `Bind` node is created with a singleton CatList, also O(1).

2. **O(1) amortized `uncons`**: During `evaluate`, when a `Pure` value is reached, the next continuation is popped from the CatList via `uncons` (O(1) amortized). When a `Bind` is encountered, its inner CatList is merged with the outer one via `append` (O(1)).

3. **No left-association penalty**: A sequence like `pure(a).bind(f1).bind(f2).bind(f3)...bind(fn)` builds a single `Bind` node with the CatList `[f1, f2, ..., fn]`, rather than a nested tree of binds.

### CatList Backing

The Rust CatList uses `VecDeque<CatList<A>>` for the sublist queue rather than PureScript's `CatQueue` (a two-list queue). This is a pragmatic choice: `VecDeque` provides O(1) amortized operations on both ends with cache-friendly contiguous storage, whereas PureScript's `CatQueue` uses a pair of linked lists. The Rust approach should perform better in practice due to reduced allocation overhead and better locality.

One concern: `flatten_deque` performs a right fold over the entire deque. The documentation claims O(1) amortized per element across a full sequence of `uncons` calls, which is correct by the standard amortization argument (each element enters the deque at most once and is visited by `flatten_deque` at most once).

## 3. Stack Safety

### `evaluate` Loop

The `evaluate` method is the core trampoline. It works as follows:

1. Type-erase the entire `Free` value via `erase_type()`.
2. Enter an iterative loop with two pieces of state: `current` (the current node) and `continuations` (a CatList of pending continuations).
3. On `Pure(val)`: pop the next continuation and apply it, or if empty, downcast to `A` and return.
4. On `Wrap(fa)`: call `F::evaluate(fa)` to extract the inner `Free` and continue.
5. On `Bind { head, continuations: inner }`: set `current = *head` and merge `inner.append(continuations)`.

This is fully iterative; no recursion occurs. The stack depth is constant regardless of the number of `bind` operations or `wrap` layers.

### `resume` Loop

`resume` follows the same iterative pattern as `evaluate` but stops at the first `Wrap` layer instead of evaluating through it. When it encounters a `Wrap`, it must reconstruct the remaining continuation chain by mapping over the functor to reattach all pending continuations. This is correct but involves some clever type-level gymnastics (using `Cell` to move the CatList into an `Fn` closure).

### `Drop` Implementation

The custom `Drop` is essential for stack safety: without it, dropping a deeply nested `Free` would recursively drop each node, overflowing the stack. The implementation uses a worklist-based approach:

1. Push the inner `FreeInner` onto a `Vec` worklist.
2. Pop nodes and process them: `Pure` nodes drop trivially, `Wrap` nodes are eagerly evaluated to extract the inner `Free`, `Bind` nodes have their head dropped and continuations drained via `uncons`.

One concern: for `Wrap` nodes, the `Drop` implementation calls `Evaluable::evaluate` to extract the inner `Free`. This forces evaluation of potentially expensive effects purely for the sake of drop safety. If the functor `F` has expensive side effects, dropping an unevaluated `Free` could be surprising. For the primary use case (`ThunkBrand`/`Trampoline`), thunks are cheap closures, so this is acceptable.

Another concern: for `Bind` nodes, the comment acknowledges that `head` (of type `Box<Free<F, TypeErasedValue>>`) cannot be folded into the worklist because it has a different type parameter. It relies on the head's own `Drop` impl to handle nested chains. This is correct (the head's `Drop` will also use the worklist pattern), but it means each `Bind` level creates a new `Drop` invocation. For a chain of `n` binds, this should still be O(n) total work with O(1) stack depth, because each `Bind`'s head is typically a leaf (`Pure` or `Wrap`), not another deeply nested `Bind`.

### `hoist_free` is NOT Stack-safe

The documentation correctly notes that `hoist_free` recurses over `Wrap` depth. Each `Wrap` layer produces one recursive call. This is acceptable in practice because `Wrap` depth corresponds to the number of distinct `lift_f` calls (effects), not the number of `bind` operations (which can be arbitrarily deep). However, for programmatically generated `Wrap` chains (e.g., the test `test_free_hoist_free_deep_wraps` uses only 100 layers), this could overflow for thousands of layers. The `fold_free` alternative is stack-safe because it delegates to `MonadRec::tail_rec_m`.

## 4. Comparison to PureScript's Free

### Semantic Fidelity

The Rust version captures the core semantics of PureScript's Free:

| Feature | PureScript | Rust | Match? |
|---------|-----------|------|--------|
| O(1) bind | Yes (CatList of ExpF) | Yes (CatList of Continuation) | Yes |
| Stack-safe evaluation | Yes (toView is iterative) | Yes (evaluate loop) | Yes |
| `pure` | `pure = fromView <<< Return` | `Free::pure(a)` | Yes |
| `wrap` | `wrap f = fromView (Bind (unsafeCoerce f) unsafeCoerce)` | `Free::wrap(fa)` | Yes |
| `liftF` | `liftF f = fromView (Bind ...)` | `Free::lift_f(fa) = wrap(map(pure, fa))` | Equivalent |
| `bind` | `bind (Free v s) k = Free v (snoc s ...)` | Matches for `Bind` variant; creates `Bind` for `Pure`/`Wrap` | Equivalent |
| `resume` | Pattern match on `toView` | Iterative loop collapsing binds | Equivalent |
| `foldFree` | `tailRecM go` | `G::tail_rec_m(...)` | Yes |
| `hoistFree` | `substFree (liftF <<< k)` | Recursive via `resume` | Different impl, same semantics |
| `substFree` | Recursive fold | Not provided | Missing |
| `runFree` | Recursive | Not provided | Missing |
| `runFreeM` | `tailRecM go` | Not provided | Missing |

### Key Differences

1. **Type erasure mechanism**: PureScript uses `unsafeCoerce` with a phantom `Val` type. Rust uses `Box<dyn Any>` with `downcast`. Both serve the same purpose: allowing heterogeneous continuations in the CatList.

2. **Evaluable constraint**: PureScript's Free is generic over any functor and provides multiple interpretation strategies (`foldFree`, `runFree`, `runFreeM`, `resume`). Rust's Free requires `F: Evaluable`, tying the functor to a specific evaluation strategy. This constrains flexibility but simplifies the API.

3. **Missing operations**: Rust lacks `substFree` (fold into another Free monad), `runFree` (step-by-step with a functor unwrapper), and `runFreeM` (step-by-step into a MonadRec). The `fold_free` method covers the most important use case (`foldFree`), and `resume` covers the step-by-step case.

4. **MonadTrans**: PureScript's Free implements `MonadTrans` (`lift = liftF`). Rust's Free does not, because it lacks HKT trait integration.

5. **Foldable/Traversable**: PureScript implements `Foldable` and `Traversable` for `Free f` when `f` is `Foldable`/`Traversable`. Rust does not, due to the lack of HKT integration.

6. **Eq/Ord**: PureScript derives `Eq` and `Ord` for Free via `resume`. Rust does not implement these (likely because `Box<dyn Any>` makes equality comparison difficult).

7. **bind implementation detail**: PureScript's `bind` always appends to the existing CatList (`snoc`), regardless of the current variant. The Rust version has three match arms for `Pure`, `Wrap`, and `Bind`. For `Pure` and `Wrap`, it creates a new `Bind` node. Only the `Bind` arm uses `snoc` directly. This means a sequence of binds on a `Pure` value will create nested `Bind` nodes until `evaluate` flattens them. PureScript avoids this because it always stores a CatList alongside the view. This is a minor structural difference; the flattening in `evaluate` handles it efficiently.

### `toView` vs `evaluate`

PureScript's `toView` is the workhorse: it iteratively collapses the Free structure into a `FreeView` (either `Return a` or `Bind (f b) (b -> Free f a)`). This is then pattern-matched by consumers like `resume`, `foldFree`, etc.

Rust's `evaluate` and `resume` each contain their own iterative loop, duplicating the pattern-collapsing logic. A `toView`-like helper would reduce this duplication, though the Rust ownership model makes a shared helper less natural (each consumer needs to handle the extracted value differently).

## 5. HKT Support

### No Brand for Free

`Free` does not have a brand type and does not implement `Kind`. The module documentation explains the fundamental conflict:

- `Kind` requires `type Of<'a, A: 'a>: 'a`, meaning the type constructor must accept any lifetime `'a`.
- `Free` uses `Box<dyn Any>`, which requires `A: 'static`.

These are irreconcilable: `Free` cannot promise to work with `&'a str` or other non-static references.

### Consequences

Without HKT integration, `Free` cannot be used with the library's generic functions (`map`, `bind`, `pure`, `fold`, etc.) or participate in type class hierarchies. Instead, it provides its own inherent methods (`Free::pure`, `Free::bind`, `Free::map`, `Free::evaluate`). This is a significant limitation for composability, but the trade-off (stack safety + O(1) bind) is documented and justified.

The `Trampoline` type is a newtype wrapper around `Free<ThunkBrand, A>` that provides a more ergonomic API for the common case.

## 6. Type Class Implementations

### What is Implemented

Free implements very few type class traits from the library:

1. **`Deferrable<'static>`** for `Free<ThunkBrand, A>`: Allows `defer(|| ...)` to create a deferred Free computation. This delegates to `Free::wrap(Thunk::new(f))`.

### What is NOT Implemented

- **Functor**: `Free::map` is an inherent method, not a `Functor` trait impl.
- **Monad**: `Free::bind` and `Free::pure` are inherent methods.
- **MonadRec**: Not implemented as a trait (cannot be, due to `'static` constraint). However, `Free::evaluate` effectively provides the same functionality.
- **Foldable/Traversable**: Not implemented.
- **Semigroup/Monoid**: Not implemented.

PureScript implements all of these (`Functor`, `Apply`, `Applicative`, `Bind`, `Monad`, `MonadRec`, `MonadTrans`, `Foldable`, `Traversable`, `Semigroup`, `Monoid`) for its Free type. The Rust version only has inherent methods for the core monadic operations.

### Correctness of Inherent Methods

The inherent methods are correct:

- `map` is implemented via `bind` + `pure`, matching PureScript's `map k f = pure <<< k =<< f`.
- `bind` appends to the CatList, matching the O(1) guarantee.
- `pure` creates a `Pure` variant.

The monad laws are tested extensively (left identity, right identity, associativity) with both `pure` and `lift_f` based computations.

## 7. The `'static` Requirement

### Why Required

`Box<dyn Any>` requires `A: 'static` because `Any` is defined as:

```rust
pub trait Any: 'static { ... }
```

This exists so that `TypeId::of::<T>()` is well-defined; type IDs are only meaningful for `'static` types.

### Can it be Relaxed?

**Short answer: No, not without fundamental changes.**

Possible approaches:

1. **Unsafe type erasure**: Replace `Box<dyn Any>` with raw pointer casts (like PureScript's `unsafeCoerce`). This would remove the `'static` requirement but sacrifice runtime type safety. A type mismatch would be silent UB rather than a panic.

2. **GATs or higher-rank trait bounds**: Even with Rust's current GAT support, the fundamental issue is that the CatList must store continuations of heterogeneous types. Without type erasure, there is no way to store `FnOnce(A) -> Free<F, B>` and `FnOnce(B) -> Free<F, C>` in the same collection.

3. **Naive Free without CatList**: A naive `enum Free { Pure(A), Wrap(F<Free<F, A>>) }` with a recursive `Bind` variant can support arbitrary lifetimes but loses O(1) bind and stack safety.

The `'static` requirement is a fundamental trade-off inherent to the "Reflection without Remorse" technique in Rust's type system.

## 8. Performance

### Allocation Overhead

Each `bind` operation:
- Boxes the continuation closure: `Box::new(move |val| ...)` (1 heap allocation).
- May box the erased value: `Box::new(a) as TypeErasedValue` (1 heap allocation).
- Appends to CatList via `snoc` (O(1), may trigger VecDeque reallocation).

For `Pure` and `Wrap` variants, `bind` also creates a `Bind` node, boxing the head `Free` value.

Compared to PureScript (which allocates freely via GC), the Rust version is more explicit about allocations but likely has similar overhead in practice. Each continuation is a boxed closure in both languages.

### `evaluate` Overhead

Each step in the evaluation loop involves:
- Taking the inner value from the `Option` wrapper.
- Pattern matching on `FreeInner`.
- For `Pure`: downcast (`downcast::<A>()`) on the final value.
- For continuations: downcast per continuation application.

The `downcast` calls involve a `TypeId` comparison (cheap) but also an `expect` with string formatting on failure (should be optimized away on the happy path).

### Type Erasure Cost

The `erase_type` method traverses the `Free` structure, boxing values:
- `Pure(a)` becomes `Pure(Box::new(a))`.
- `Wrap(fa)` maps over the functor to erase the inner type.
- `Bind` can simply change the `PhantomData` marker.

This traversal happens once at the start of `evaluate`. For a `Pure` value, it is O(1). For a `Wrap`, it maps the functor (one function call). For a `Bind`, it is O(1) since only the marker changes.

### Potential Concern: Bind on Pure/Wrap Creates Extra Indirection

When `bind` is called on a `Pure(a)`:
1. It creates `Free::from_inner(FreeInner::Pure(Box::new(a)))` as the head.
2. Wraps it in `FreeInner::Bind { head: Box::new(head), ... }`.

This adds two layers of boxing. PureScript avoids this by always having a CatList attached to the view. In the Rust version, a single `pure(a).bind(f)` produces:

```
Bind {
    head: Box(Free(Some(Pure(Box(a))))),
    continuations: CatList([f]),
}
```

Whereas PureScript produces `Free(Return(a), CatList([f]))`, a flatter structure. However, the Rust version flattens this during `evaluate`, so the runtime cost is similar (just one extra level of boxing per bind on a non-Bind variant).

## 9. Documentation Quality

### Strengths

- The module-level documentation is excellent: it clearly explains the PureScript comparison, the intended use case, the HKT limitation, and the `'static` constraint rationale.
- Each method has comprehensive doc comments with signature annotations, parameter descriptions, return descriptions, and runnable examples.
- The `Consuming a Free: evaluate vs fold_free` section provides clear guidance.
- The "Capabilities and Limitations" section is honest about what Free cannot do.
- The "Linear consumption invariant" documentation explains the `Option` wrapper pattern.

### Weaknesses

- The `evaluate` method's doc comment could explain the trampoline algorithm in more detail (the current description is "iteratively processes the CatList of continuations without growing the stack," which is accurate but brief).
- The relationship between `Free` and `Trampoline` is mentioned in the module docs for `trampoline.rs` but not in `free.rs`. A cross-reference would help.
- The `FreeInner` enum is documented but the Evaluable bound on `F` could be explained more (why `Evaluable` specifically, rather than just `Functor`).
- The `erase_type` method's invariant about `Functor::map` calling the mapping function exactly once is documented inline but could be elevated to a more prominent location, since violating it causes UB-like behavior (double-free or leak).

## 10. Issues, Limitations, Design Flaws

### Issue 1: `Evaluable` Constraint is Overly Restrictive

`Free<F, A>` requires `F: Evaluable` on all operations, including `pure`, `bind`, and `map`, which do not need to evaluate anything. This prevents constructing a `Free` over a functor that is not `Evaluable`. In PureScript, `Free` only requires `Functor f` for operations like `resume` and no constraint at all for `pure` and `bind`.

Loosening the constraint to `F: Functor` on construction methods and only requiring `F: Evaluable` on `evaluate` would increase flexibility. The `fold_free` method already works with any functor (it just needs `NaturalTransformation`).

### Issue 2: No `Apply` / `Applicative`

PureScript's Free implements `Apply` (via `ap`) and `Applicative` (via `pure`). The Rust version has `pure` and `bind` but no `apply`. While `apply` can be derived from `bind` (as PureScript does), having it as an inherent method would be convenient.

### Issue 3: `hoist_free` Stack Safety

As noted above, `hoist_free` recurses over `Wrap` depth. For programmatically generated deep `Wrap` chains, this can overflow. A stack-safe version could use an explicit stack or iterate via `resume`.

### Issue 4: Bind on Pure Creates Unnecessary Nesting

When calling `bind` on `Pure(a)`, the implementation boxes `a` into a new `Pure(Box::new(a))`, wraps that in a `Free`, boxes that into a `Bind` head. This is two heap allocations that PureScript avoids. A more direct approach: store the continuation directly and defer application, rather than creating a `Bind` with a `Pure` head.

Alternatively, the PureScript approach of always pairing a view with a CatList would eliminate this special case entirely.

### Issue 5: `evaluate` Calls in `Drop`

The `Drop` implementation calls `Evaluable::evaluate` on `Wrap` nodes to extract the inner `Free` for iterative dismantling. This means dropping an unevaluated `Free` will force evaluation of all wrapped functor layers. For thunks this is typically cheap, but for a hypothetical functor with expensive or side-effectful `evaluate`, this could be surprising. The documentation does not mention this behavior.

### Issue 6: `map` Uses FnOnce but Functor::map Requires Fn

`Free::map` takes `FnOnce` but internally calls `bind`, which creates a continuation that calls `Free::pure(f(a))`. This is fine. However, `erase_type` and `resume` call `F::map` with a closure, and `Functor::map` in this library requires `impl Fn` (not `FnOnce`). The code uses `Cell` tricks to move owned values into `Fn` closures, which works but adds complexity. This is a recurring friction point between Rust's ownership model and the FP library's `Fn`-based traits.

### Issue 7: No Debug Implementation

`Free<F, A>` does not implement `Debug`, making it harder to inspect values during development. The type-erased internals make this non-trivial but not impossible (at least for the `Pure` variant).

### Issue 8: Missing `substFree`

PureScript's `substFree` folds a Free into another Free without the `MonadRec` overhead. This is useful for rewriting Free computations. The Rust version only has `fold_free` (which requires `MonadRec` on the target) and `hoist_free` (which only changes the functor, not the structure).

## 11. Alternatives and Improvements

### Alternative 1: Church-encoded Free

Instead of the CatList approach, a Church-encoded Free monad (also known as codensity-transformed Free) avoids the quadratic bind issue by CPS-transforming the structure:

```rust
struct Free<F, A> {
    run: Box<dyn FnOnce(&dyn Fn(A) -> R, &dyn Fn(F<Free<F, A>>) -> R) -> R>
}
```

This eliminates the need for type erasure and CatList entirely but makes `resume` difficult and has its own complexity trade-offs. It would also still struggle with Rust's ownership and lifetime constraints.

### Alternative 2: Unsafe Type Erasure

Replace `Box<dyn Any>` with unsafe pointer casts to remove the `'static` requirement:

```rust
type TypeErasedValue = *mut ();
```

This would allow `Free` to work with non-static lifetimes and potentially integrate with the HKT system. The downside is the loss of runtime type safety; a bug in the continuation chain would be silent UB rather than a caught panic. Given the internal-only nature of the type erasure (users never interact with `TypeErasedValue` directly), this could be justified with sufficient testing and careful invariant maintenance.

### Alternative 3: Split the Constraints

Separate construction from evaluation:

```rust
// Construction: no constraints on F
impl<F, A: 'static> Free<F, A> {
    pub fn pure(a: A) -> Self { ... }
    pub fn bind<B: 'static>(self, f: impl FnOnce(A) -> Free<F, B> + 'static) -> Free<F, B> { ... }
}

// Evaluation: requires Evaluable
impl<F: Evaluable, A: 'static> Free<F, A> {
    pub fn evaluate(self) -> A { ... }
}
```

This would allow building Free computations over arbitrary brands and only requiring `Evaluable` when actually running them.

### Alternative 4: Generalized Interpretation

Add `substFree` and `runFreeM` to match PureScript's API:

```rust
pub fn subst_free<G: Evaluable + 'static>(
    self,
    nt: impl NaturalTransformation<F, FreeBrand<G>> + Clone + 'static,
) -> Free<G, A> { ... }
```

This would enable the "interpret into another Free" pattern without `MonadRec` overhead.

### Improvement: Flatten `bind` on `Pure`

Instead of creating a `Bind { head: Pure(a), continuations: [f] }`, directly apply `f(a)`:

```rust
FreeInner::Pure(a) => f(a).erase_type_to_b(),
```

However, this would make `bind` non-O(1) if `f` itself returns a deep chain. The current approach correctly defers all work to `evaluate`. The extra indirection is the price of guaranteed O(1) `bind`.

### Improvement: Shared `toView` Helper

Factor the iterative loop from `evaluate` and `resume` into a shared helper function, similar to PureScript's `toView`. This would reduce code duplication and make the core algorithm easier to audit. The challenge is designing the return type to work with Rust's ownership model.
