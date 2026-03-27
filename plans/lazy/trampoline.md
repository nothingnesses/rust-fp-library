# Trampoline Analysis

File: `fp-library/src/types/trampoline.rs` (680 lines of source, 410 lines of tests)

## 1. Design: Trampoline = Free<ThunkBrand, A>

### Assessment: Strong design, well-motivated

The choice to define `Trampoline<A>` as a newtype over `Free<ThunkBrand, A>` (line 82-85) is sound and follows established FP tradition. In Haskell/PureScript, `Trampoline` is typically defined as `Free Identity`, where `Identity` is the trivial functor. Here, `ThunkBrand` (a deferred `FnOnce() -> A`) plays the role of `Identity` with added laziness, which is the right choice for Rust where laziness is not the default.

**Advantages of this design:**

- Stack safety is inherited from `Free`'s CatList-based "Reflection without Remorse" evaluation loop (free.rs lines 716-762), rather than being reimplemented.
- O(1) `bind` is inherited from `Free`'s `CatList` snoc operation (free.rs line 413).
- The `evaluate` method delegates directly to `Free::evaluate` (line 258-260), which uses an iterative loop over the CatList of type-erased continuations.
- Separation of concerns: `Free` handles the stack-safety machinery; `Trampoline` provides the ergonomic API.

**Comparison to typical trampoline implementations:**

Most Rust trampoline crates (e.g., `tramp`) use a simple `enum { Done(A), More(Box<dyn FnOnce() -> Trampoline<A>>) }` with an iterative `run` loop. This approach is simpler but has O(n) left-associated bind due to closure nesting. The `Free`-based approach here gives O(1) bind at the cost of more complex internals (type erasure via `Box<dyn Any>`, `CatList` of continuations).

This trade-off is appropriate for a library that emphasizes monadic composition, where left-associated bind chains are common.

## 2. Implementation Quality

### 2.1 Core Methods: Correct

- **`pure`** (line 110-112): Delegates to `Free::pure`. Correct.
- **`new`** (line 142-144): Wraps a `FnOnce` in a `Thunk` that produces a `Free::pure`. Correct, the closure is deferred until evaluation.
- **`defer`** (line 179-181): Wraps a closure that produces a `Trampoline` into a `Free::wrap(Thunk::new(...))`. This is the critical method for stack-safe recursion, and it correctly defers construction of the inner trampoline.
- **`bind`** (line 208-213): Delegates to `Free::bind`, unwrapping and rewrapping the newtype. Correct.
- **`map`** (line 233-238): Delegates to `Free::map`. Correct.
- **`evaluate`** (line 258-260): Delegates to `Free::evaluate`. Correct.

### 2.2 Stack Safety: Verified

Stack safety holds because:

1. `Free::evaluate` uses an iterative loop (free.rs lines 720-762) that never recurses.
2. `Free::bind` appends to a `CatList` in O(1) (free.rs lines 406-416).
3. `Trampoline::defer` wraps construction in `Free::wrap(Thunk::new(...))`, ensuring recursive calls become data on the heap rather than stack frames.
4. `Trampoline::tail_rec_m` (lines 465-486) uses `defer` + `bind` internally, making each recursive step a heap allocation processed by the iterative evaluator.
5. Tests at lines 1056-1088 verify 200,000 iterations without stack overflow.

### 2.3 tail_rec_m Implementation

The `tail_rec_m` implementation (lines 465-486) uses an interesting approach: a recursive helper function `go` that uses `Trampoline::defer` and `bind` to build up a chain of deferred steps. This is correct and stack-safe because each step is a `defer` (which creates a `Free::Wrap`), and the evaluator processes these iteratively.

However, there is a **performance concern**: each iteration allocates a `Thunk` (via `defer`) and a continuation (via `bind`), plus type erasure overhead. A more efficient implementation could use an imperative loop similar to `Thunk`'s `MonadRec` implementation (thunk.rs), but this would bypass the `Free` monad infrastructure. The current approach trades allocation efficiency for consistency and code reuse.

The `arc_tail_rec_m` variant (lines 535-546) neatly solves the `Clone` bound issue by wrapping the closure in `Arc`. This is a thoughtful ergonomic addition.

### 2.4 Potential Issues

**No bugs found.** The implementation is straightforward delegation to `Free`, which is well-tested.

**Minor observation:** The `Clone` bound on `tail_rec_m`'s `f` parameter (line 466) is necessary because the recursive `go` function clones `f` at each step. The documentation at lines 423-430 clearly explains this and points to `arc_tail_rec_m` as the alternative.

## 3. Type Class Instances

### Implemented:

| Instance | Lines | Correct? | Notes |
|----------|-------|----------|-------|
| `Deferrable<'static>` | 575-601 | Yes | Delegates to `Trampoline::defer`. |
| `Semigroup` (where `A: Semigroup`) | 604-632 | Yes | Uses `lift2` with `Semigroup::append`. |
| `Monoid` (where `A: Monoid`) | 635-656 | Yes | Uses `Trampoline::pure(Monoid::empty())`. |
| `Debug` | 660-678 | Yes | Always prints `"Trampoline(<unevaluated>)"`. |
| `From<Lazy<'static, A, Config>>` | 553-572 | Yes | Clones the lazy value; requires `A: Clone`. |

### Conversions (defined in other files):

| Conversion | Location | Notes |
|------------|----------|-------|
| `From<Thunk<'static, A>> for Trampoline<A>` | thunk.rs:384 | Evaluates thunk eagerly in a deferred wrapper. |
| `From<Trampoline<A>> for Thunk<'static, A>` | thunk.rs:366 | Wraps `trampoline.evaluate()` in a thunk. |
| `From<Trampoline<A>> for Lazy<'_, A, RcLazyConfig>` | lazy.rs:640 | Deferred evaluation with memoization. |
| `From<Trampoline<A>> for Lazy<'_, A, ArcLazyConfig>` | lazy.rs:688 | Eager evaluation (Trampoline is `!Send`). |

### Missing Instances:

**No HKT instances** (Functor, Monad, Foldable, etc.). This is intentional and well-documented. `Trampoline` requires `'static` due to `Box<dyn Any>` in `Free`, which conflicts with the `Kind` trait's lifetime polymorphism. The brands.rs file (lines 219-220) explicitly notes: "This is for `Thunk<'a, A>`, NOT for `Trampoline<A>`. `Trampoline` cannot implement HKT traits due to its `'static` requirement."

**Not missing but worth noting:**
- No `Eq`/`PartialEq`: Cannot compare without evaluating, which would require consuming the value. Reasonable omission.
- No `Clone`: The underlying `Free` uses `Box<dyn FnOnce>` which is not cloneable. Correct omission.
- No `Foldable`/`Traversable`: Would require HKT brands. Correct omission.
- No `MonadRec`: Cannot implement the HKT-based `MonadRec` trait, but provides an equivalent inherent method `tail_rec_m`. This is documented in monad_rec.rs (lines 46-48).

## 4. API Surface

### Assessment: Well-designed, complete

The API covers all essential operations:

| Method | Purpose | Signature Quality |
|--------|---------|-------------------|
| `pure` | Lift a value | Clean |
| `new` | Defer a computation | Clean |
| `defer` | Defer construction of a Trampoline | Critical for recursion |
| `bind` | Monadic sequencing | Clean |
| `map` | Functor mapping | Clean |
| `evaluate` | Force computation | Clean |
| `lift2` | Applicative lifting | Useful convenience |
| `then` | Sequence, discarding first result | Useful convenience |
| `append` | Semigroup combination | Consistent with hierarchy |
| `empty` | Monoid identity | Consistent with hierarchy |
| `tail_rec_m` | Stack-safe tail recursion | Essential |
| `arc_tail_rec_m` | Non-Clone variant | Thoughtful ergonomic addition |
| `into_rc_lazy` | Convert to memoized form | Good escape hatch |
| `into_arc_lazy` | Convert to thread-safe memoized form | Good escape hatch |

**One observation on `into_arc_lazy`** (lines 306-310): The doc comment (lines 286-292) explains that Trampoline is `!Send` so it must evaluate eagerly. But the implementation delegates to `Lazy::from(self)` (line 309), which is the `From<Trampoline<A>> for Lazy<'a, A, ArcLazyConfig>` impl at lazy.rs:688 that does `Self::pure(eval.evaluate())`. This means the computation is evaluated during conversion, not deferred. The documentation is accurate, but the naming `into_arc_lazy` could be slightly misleading since the result is already evaluated (just wrapped in a `Lazy::pure`). However, this is consistent with `Thunk::into_arc_lazy` which does the same thing.

## 5. Consistency with the Hierarchy

### Assessment: Highly consistent

- **Naming**: Uses the same method names as `Thunk` (`pure`, `new`, `defer`, `bind`, `map`, `evaluate`, `into_rc_lazy`, `into_arc_lazy`). Users can switch between them with minimal API changes.
- **Semigroup/Monoid**: Same pattern as other lazy types (lift the operation into the deferred context).
- **Deferrable**: Implements `Deferrable<'static>`, consistent with the trait's purpose.
- **Conversions**: Bidirectional conversions with `Thunk`, `Lazy`, and `ArcLazy` are comprehensive.
- **TryTrampoline**: Follows the same newtype-over-inner pattern (`TryTrampoline<A, E> = Trampoline<Result<A, E>>`), consistent with `TryThunk = Thunk<Result<A, E>>`.
- **Debug**: Opaque output (`"Trampoline(<unevaluated>)"`) is consistent with `Thunk`'s approach.

## 6. Limitations

### 6.1 The `'static` Requirement

The most significant limitation. All values in a `Trampoline` must be `'static` because `Free` uses `Box<dyn Any>` for type erasure, and `Any: 'static`. This means:

- Cannot use borrowed data (e.g., `&str`, `&[u8]`).
- Cannot capture non-`'static` references in closures passed to `new`, `defer`, `bind`.
- Cannot implement HKT traits.

This is well-documented (lines 55, 82) and inherent to the "Reflection without Remorse" technique in Rust. The documentation correctly directs users to `Thunk` for lifetime-polymorphic use cases.

### 6.2 Performance Overhead

Each `bind` operation involves:
1. Type erasure via `Box::new(a) as Box<dyn Any>` (allocation).
2. A `Continuation<F>` closure allocation (boxed `FnOnce`).
3. `CatList::snoc` (O(1) amortized but involves allocation).
4. `downcast` on evaluation (dynamic type check, though practically free).

For shallow chains, a simple `Thunk` chain will be faster. The overhead pays off only for deep chains (hundreds/thousands of binds) where stack safety matters.

### 6.3 Not `Send`

`Trampoline` is `!Send` because `Thunk` wraps `Box<dyn FnOnce() -> A>` without a `Send` bound, and `Free` internally stores `Box<dyn FnOnce(Box<dyn Any>) -> Free<F, Box<dyn Any>>>` continuations that are also `!Send`. This limits `Trampoline` to single-threaded contexts.

The `into_arc_lazy` method provides an escape hatch by eagerly evaluating and then wrapping in a thread-safe container.

### 6.4 No Memoization

Documented at line 70-79. Each call to `evaluate` re-runs the computation. The documentation suggests wrapping in `Lazy` for caching, which is the correct approach.

### 6.5 Missing Deep Bind Chain Test

The test suite includes a 200,000-iteration stress test for `tail_rec_m` (lines 1056-1069) but no equivalent deep bind chain test for `Trampoline` specifically. The `Free` tests do cover this (free.rs:909-917 tests 100,000 bind iterations), so stack safety is verified at the `Free` level, but a dedicated `Trampoline` test would improve confidence. This is a minor gap.

## 7. Documentation

### Assessment: Thorough and accurate

- **Module-level docs** (lines 1-20): Clear explanation of trampolining, with a working example.
- **Struct docs** (lines 46-85): Comprehensive coverage of requirements, guarantees, when-to-use guidance, and memoization note with code example.
- **Method docs**: Every method has `#[document_signature]`, type parameter descriptions, parameter descriptions, return descriptions, and examples with assertions.
- **`tail_rec_m` Clone bound** (lines 423-430): Explicitly explains why `Clone` is needed and points to `arc_tail_rec_m`.
- **Comparison table** in thunk.rs (lines 60-67): Clear side-by-side comparison between `Thunk` and `Trampoline`.

**One minor documentation gap:** The `lift2` method (lines 334-340) is documented but does not mention that it is implemented as `self.bind(|a| other.map(|b| f(a, b)))`, which means evaluation is sequential (left then right). For a type that claims no parallelism, this is fine, but explicit mention of sequential evaluation would be helpful.

## 8. Summary

| Aspect | Rating | Notes |
|--------|--------|-------|
| Design | Excellent | `Free<ThunkBrand, A>` is the right abstraction. |
| Correctness | No bugs found | Delegation to well-tested `Free` is reliable. |
| Stack safety | Verified | Iterative evaluation loop, CatList, tests at 200K depth. |
| Type class instances | Complete for constraints | No HKT instances (correctly impossible). Semigroup, Monoid, Deferrable present. |
| API surface | Complete and ergonomic | All essential operations present, good convenience methods. |
| Consistency | High | Matches Thunk/TryTrampoline patterns closely. |
| Performance | Acceptable overhead | Type erasure cost is inherent to the "Reflection without Remorse" technique. |
| Documentation | Thorough | Every method documented with examples. Trade-offs clearly explained. |

**Recommendations:**

1. Add a dedicated deep bind chain stack-safety test (e.g., 100,000 binds) to the Trampoline test module, mirroring the Free-level test.
2. Consider whether `tail_rec_m` could use an imperative loop (like Thunk's `MonadRec`) for better allocation performance, since Trampoline's evaluate is already iterative. This would bypass the `defer`/`bind` overhead per iteration while still being stack-safe.
3. No structural changes needed. The implementation is sound.
