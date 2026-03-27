# TrySendThunk Analysis

**File:** `fp-library/src/types/try_send_thunk.rs`
**Lines:** 1394

## 1. Design: Does the Send+Try combination make sense?

Yes. `TrySendThunk<'a, A, E>` is the product of two orthogonal axes: fallibility (`Result<A, E>`) and thread safety (`Send`). It fills the same cell in the design matrix as `SendThunk` does for the infallible case, and as `TryThunk` does for the single-threaded fallible case.

The representation is clean: a newtype over `SendThunk<'a, Result<A, E>>` (line 101-103). This mirrors `TryThunk`'s representation as a newtype over `Thunk<'a, Result<A, E>>`. The delegation to `SendThunk` is the right call; it reuses the `Send`-enforcing closure boxing without duplicating machinery.

**Practical motivation:** Sending a fallible deferred computation to a thread pool (e.g., `std::thread::spawn`, rayon, tokio) is a real use case. Without `TrySendThunk`, users would have to use `SendThunk<Result<A, E>>` directly, losing the ergonomic combinators (`bind`, `map_err`, `catch`, `bimap`, etc.).

## 2. Implementation Quality

### Correctness

The core operations are correct:

- **`bind` (lines 243-255):** Properly short-circuits on `Err`, delegates to `SendThunk::bind`. The `Send` bounds on all type parameters (`A`, `B`, `E`) and the closure `f` are correct and necessary.
- **`map` (lines 275-280):** Correctly delegates `result.map(func)` through `SendThunk::map`.
- **`map_err` (lines 300-305):** Correctly delegates `result.map_err(f)` through `SendThunk::map`.
- **`bimap` (lines 334-343):** Correct match-based implementation, both closures carry `Send + 'a`.
- **`catch` (lines 362-373):** Correct: on `Ok`, wraps in `SendThunk::pure(Ok(a))`; on `Err`, delegates to recovery function. The `Send` bounds on `A` and `E` are required for `SendThunk::pure`.
- **`catch_with` (lines 405-413):** Correct: creates a new `SendThunk::new` that evaluates `self` and dispatches. This avoids the type-mismatch issues that would arise from using `SendThunk::bind` when the error type changes.
- **`Semigroup` (lines 934-943):** Correctly short-circuits via `?` operator. The `Semigroup` bound on `A` ensures the inner values can be combined.
- **`into_arc_try_lazy` (lines 554-559):** Correctly delegates through `SendThunk::into_arc_lazy`, wrapping back in `TryLazy`. This is a genuine lazy conversion (no eager evaluation), which is the key advantage of the `Send` variant.

### No bugs found

The implementation is straightforward delegation to `SendThunk` with `Result` wrapping/unwrapping. The patterns are consistent and correct.

### Bound consistency

One minor observation: `bind` requires `A: Send + 'a`, `B: Send + 'a`, `E: Send + 'a` in `where` clauses (lines 248-250), while `map` only requires `Send + 'a` on the closure (line 277). This asymmetry is correct: `bind` must construct a `SendThunk::pure(Err(e))` in the error branch, which requires `E: Send`, and `SendThunk::bind` internally needs `A: Send`. `map` only wraps around the existing `SendThunk::map`, which handles the `Send` constraint internally.

## 3. Type Class Instances

### Implemented

| Instance | Lines | Correct? |
|---|---|---|
| `Deferrable` | 842-873 | Yes, eagerly evaluates (documented) |
| `SendDeferrable` | 880-905 | Yes, truly deferred via `SendThunk::new` |
| `Semigroup` | 912-944 | Yes, requires `A: Semigroup + Send`, `E: Send` |
| `Monoid` | 951-971 | Yes, delegates to `Monoid::empty()` |
| `Debug` | 979-997 | Yes, prints without evaluating |

### Correctly absent (cannot be implemented)

The following HKT traits cannot be implemented for `TrySendThunk` because their signatures do not require `Send` on closure parameters. This is well-documented in the module-level docs (lines 7-9 of `send_thunk.rs`) and the struct docs (lines 49-62 of `try_send_thunk.rs`):

- `Functor` / `FunctorWithIndex`
- `Pointed`
- `Semimonad` / `Semiapplicative`
- `Foldable` / `FoldableWithIndex`
- `Bifunctor` / `Bifoldable`
- `MonadRec`
- `Lift`, `ApplyFirst`, `ApplySecond`
- `WithIndex`

This is the same limitation as `SendThunk`, and for exactly the same reason.

### Correctly absent brands

There are no `TrySendThunkErrAppliedBrand` or `TrySendThunkOkAppliedBrand` types. This is correct and documented in `brands.rs` (lines 271-275, 291-294): without HKT trait support, partially-applied brands serve no purpose.

The bifunctor brand `TrySendThunkBrand` exists (line 649-658) and maps `Of<'a, E, A> = TrySendThunk<'a, A, E>`. This is consistent with `TryThunkBrand`'s convention. However, no `Bifunctor` or `Bifoldable` is implemented for it, which is correct since those traits also lack `Send` bounds on their closures.

### Missing: `tail_rec_m` / `arc_tail_rec_m` inherent methods

`SendThunk` has `tail_rec_m` (lines 267-282) and `arc_tail_rec_m` (lines 326-338), providing stack-safe recursion for infallible computations. `TryThunk` has `MonadRec` via its brand (line 1027-1078), providing stack-safe recursion for fallible computations.

**`TrySendThunk` has neither.** This is a gap. A `tail_rec_m` inherent method would be valuable for stack-safe fallible recursion across thread boundaries. The implementation would be straightforward:

```rust
pub fn tail_rec_m<S>(
    f: impl Fn(S) -> TrySendThunk<'a, Step<S, A>, E> + Clone + Send + 'a,
    initial: S,
) -> Self
where
    S: Send + 'a,
    A: Send + 'a,
    E: Send + 'a,
{
    TrySendThunk::new(move || {
        let mut state = initial;
        loop {
            match f(state).evaluate() {
                Ok(Step::Loop(next)) => state = next,
                Ok(Step::Done(a)) => break Ok(a),
                Err(e) => break Err(e),
            }
        }
    })
}
```

## 4. API Surface

### Core API (inherent methods)

| Method | Present? | Notes |
|---|---|---|
| `new` | Yes (line 129) | |
| `pure` | Yes (line 149) | |
| `ok` | Yes (line 192) | |
| `err` | Yes (line 215) | |
| `defer` | Yes (line 172) | |
| `map` | Yes (line 275) | |
| `map_err` | Yes (line 300) | |
| `bimap` | Yes (line 334) | |
| `bind` | Yes (line 243) | |
| `catch` | Yes (line 362) | |
| `catch_with` | Yes (line 405) | |
| `evaluate` | Yes (line 447) | |
| `lift2` | Yes (line 481) | |
| `then` | Yes (line 523) | |
| `into_inner` | Yes (line 429) | |
| `into_arc_try_lazy` | Yes (line 554) | Truly lazy (not eager) |
| `catch_unwind_with` | Yes (line 601) | |
| `catch_unwind` | Yes (line 644) | E=String convenience |
| `tail_rec_m` | **No** | Missing |
| `arc_tail_rec_m` | **No** | Missing |

### Conversions (`From` impls)

| From | Present? | Eager? | Notes |
|---|---|---|---|
| `TryThunk` | Yes (line 665) | Yes | Must eagerly evaluate (not `Send`) |
| `Result` | Yes (line 696) | N/A | Already a value |
| `SendThunk` | Yes (line 725) | No | Maps with `Ok` |
| `ArcLazy` | Yes (line 751) | Yes | Forces + clones |
| `TryTrampoline` | Yes (line 777) | Yes | Must eagerly evaluate (not `Send`) |
| `ArcTryLazy` | Yes (line 809) | Yes | Forces + clones |

**Missing conversions compared to `TryThunk`:**
- `From<Lazy<Config>>` for `TrySendThunk`: `TryThunk` has this (line 649-668). For `TrySendThunk`, this would require eager evaluation (since `Lazy`'s inner closure is not `Send`). Could be added for completeness.
- `From<TryLazy<Config>>` for `TrySendThunk`: `TryThunk` has this (line 677-701). Same reasoning.
- `From<Thunk>` for `TrySendThunk`: `TryThunk` has this (line 708-723). Would require eager evaluation.

These are "cross-thread-boundary" conversions that all require eager evaluation. The `ArcLazy` and `ArcTryLazy` conversions (which are present) cover the thread-safe memoized types. The missing ones are for single-threaded types, which is a less common conversion path but could be useful.

## 5. Consistency with Other Variants

### Comparison matrix

| Feature | `Thunk` | `SendThunk` | `TryThunk` | `TrySendThunk` |
|---|---|---|---|---|
| `new` | Yes | Yes | Yes | Yes |
| `pure` | Yes | Yes | Yes | Yes |
| `ok`/`err` | N/A | N/A | Yes | Yes |
| `defer` | Yes | Yes | Yes | Yes |
| `map` | Yes | Yes | Yes | Yes |
| `bind` | Yes | Yes | Yes | Yes |
| `evaluate` | Yes | Yes | Yes | Yes |
| `tail_rec_m` | Yes | Yes | Via MonadRec | **Missing** |
| `arc_tail_rec_m` | N/A | Yes | N/A | **Missing** |
| HKT traits | Full | None | Full | None |
| `Deferrable` | Yes | Yes (eager) | Yes | Yes (eager) |
| `SendDeferrable` | N/A | Yes | N/A | Yes |
| `Semigroup`/`Monoid` | Yes | Yes | Yes | Yes |
| `into_rc_lazy` | Yes | N/A | Yes (`into_rc_try_lazy`) | N/A |
| `into_arc_lazy` | Yes (eager) | Yes (lazy) | Yes (eager) | Yes (lazy) |
| `catch`/`catch_with` | N/A | N/A | Yes | Yes |
| `bimap`/`map_err` | N/A | N/A | Yes | Yes |
| `lift2`/`then` | Via HKT | N/A | Yes | Yes |
| `catch_unwind` | N/A | N/A | Yes | Yes |
| `into_inner` | N/A | N/A | Yes | Yes |

The API is highly consistent. The main gap is `tail_rec_m`/`arc_tail_rec_m`.

### `ok` vs `pure` inconsistency

In `TryThunk`, `ok` is documented as an alias for `pure` (line 172-177: "Alias for `pure`, provided for readability"). Both have identical implementations (`Thunk::pure(Ok(a))`), and `ok` has no `Send` bounds because `Thunk::pure` does not need them.

In `TrySendThunk`, both `pure` (line 149) and `ok` (line 192) have identical implementations (`SendThunk::pure(Ok(a))`), identical bounds (`A: Send + 'a, E: Send + 'a`), and identical doc examples. However, `ok` lacks the "Alias for `pure`" documentation that `TryThunk::ok` has. This is a minor doc inconsistency.

### `Deferrable::defer` consistency

Both `SendThunk` and `TrySendThunk` implement `Deferrable::defer` by eagerly calling `f()` (lines 871 for `TrySendThunk`, 425 for `SendThunk`). This is correct and well-documented: the `Deferrable` trait does not require `Send` on the closure, so deferral is impossible without violating the `Send` invariant.

Both implement `SendDeferrable::send_defer` with true deferral. `SendThunk` delegates to `SendThunk::defer` (line 455), while `TrySendThunk` creates `SendThunk::new(move || f().evaluate())` (line 903). Both are correct.

## 6. Limitations and Issues

### Missing `tail_rec_m`

As noted in Section 3, `TrySendThunk` lacks `tail_rec_m` and `arc_tail_rec_m`. `SendThunk` has both. `TryThunk` has `MonadRec` via its brand. `TrySendThunk` has neither. This means there is no stack-safe recursion path for fallible + thread-safe deferred computations.

**Severity:** Moderate. Users who need stack-safe fallible recursion with `Send` must use `TryTrampoline` and then convert to `TrySendThunk` via `From`, which requires `'static` and eager evaluation.

### No `From<Thunk>`, `From<Lazy>`, `From<RcTryLazy>`, `From<RcLazy>`

These single-threaded to thread-safe conversions are absent. Each would require eager evaluation. This is arguably fine since the `Send` variants (`SendThunk`, `ArcLazy`, `ArcTryLazy`) are covered, and converting from a non-`Send` type to a `Send` type always requires eager evaluation anyway. But for completeness, they could be added.

### `TrySendThunkBrand` exists but has no trait implementations

`TrySendThunkBrand` (line 649-658) is defined as a bifunctor-style brand (`Of<'a, E, A> = TrySendThunk<'a, A, E>`), but no HKT traits are implemented for it. This is correctly documented as intentional. The brand exists for potential future use or for generic code that needs to reference the type constructor. This is fine.

### `Semigroup` does not short-circuit at the type level

The `Semigroup` implementation (lines 934-943) uses `?` to short-circuit on error. This is correct at runtime, but the bound is `A: Semigroup + Send`, not `Result<A, E>: Semigroup`. This means the semigroup operation combines success values only, and errors short-circuit. This is the correct behavior for a monadic error type (matching `TryThunk`'s behavior).

### `bind` has more restrictive bounds than `map`

`bind` requires `A: Send + 'a`, `B: Send + 'a`, `E: Send + 'a` (lines 248-250), while `map` only requires the closure to be `Send + 'a` (line 277). This is correct but may surprise users. The reason: `bind`'s error path must construct `SendThunk::pure(Err(e))`, which requires `Err(e): Send`, hence `E: Send`. The `A: Send` bound comes from `SendThunk::bind`'s requirement that the inner closure produces `Send` results.

## 7. Documentation

### Module-level docs (lines 1-6)

Brief and accurate. Correctly identifies the type as the fallible counterpart to `SendThunk` and notes re-execution semantics.

### Struct-level docs (lines 32-94)

Thorough and well-structured:
- Identifies the underlying representation (line 34).
- Explains the `Send` invariant (line 35).
- Documents non-memoized semantics (lines 39-41).
- HKT representation section (lines 43-47).
- HKT limitations section (lines 49-62) with clear explanation of why HKT traits cannot be implemented.
- "When to Use" section (lines 64-70) with alternatives.
- Algebraic properties section (lines 72-80) documenting monad laws and short-circuit behavior.
- Stack safety warning (lines 82-86).
- Traversable limitation (lines 88-94).

### Minor doc issues

1. **`ok` method (line 176):** Documented as "Creates a successful computation" but lacks the "Alias for `pure`" note that `TryThunk::ok` has (TryThunk line 172). This should be added for consistency.

2. **Right identity law (line 76):** States `thunk.bind(TrySendThunk::ok)`. This is correct but note that `TrySendThunk::ok` and `TrySendThunk::pure` are identical, so either could be cited. `TryThunk` uses `TryThunk::pure` in the same position (TryThunk line 82), which is slightly inconsistent across the two types.

3. **`catch_unwind_with` and `catch_unwind` (lines 567-646):** Well-documented with examples and clear `Send + UnwindSafe` bounds.

4. **All `From` impls:** Documented with doc comments explaining eager vs lazy semantics and the reason for eager evaluation where applicable.

### Test coverage

The test suite (lines 1001-1393) is comprehensive with 39 tests covering:
- Basic operations (`ok`, `err`, `new`, `pure`, `defer`).
- Combinators (`map`, `map_err`, `bimap`, `bind`, `catch`, `catch_with`).
- Applicative (`lift2`, `then`).
- Conversions (`From<TryThunk>`, `From<Result>`, `From<SendThunk>`, `From<ArcLazy>`, `From<ArcTryLazy>`, `From<TryTrampoline>`).
- Type class instances (`Semigroup`, `Monoid`, `Deferrable`, `SendDeferrable`).
- Thread safety (`is_send`, `send_across_thread`, `into_arc_try_lazy_thread_safety`).
- Panic handling (`catch_unwind`, `catch_unwind_with`).
- Error propagation and short-circuiting.

No missing test categories identified.

## Summary of Findings

| Category | Rating | Key Issues |
|---|---|---|
| Design | Good | Sound Send+Try product type |
| Correctness | Good | No bugs found |
| Type classes | Good, one gap | Missing `tail_rec_m`/`arc_tail_rec_m` inherent methods |
| API surface | Good, minor gaps | Missing some `From` impls for single-threaded types |
| Consistency | Good | Matches `SendThunk`/`TryThunk` patterns closely |
| Documentation | Good, minor issues | `ok` missing "alias" note; minor law citation inconsistency |
| Test coverage | Good | Comprehensive, 39 tests |

### Recommended actions

1. **Add `tail_rec_m` and `arc_tail_rec_m` inherent methods** to provide stack-safe fallible recursion for thread-safe computations. This is the most significant gap.
2. **Add "Alias for `pure`" note** to `TrySendThunk::ok` documentation for consistency with `TryThunk::ok`.
3. **Consider adding `From<Thunk>`, `From<Lazy>`, `From<TryLazy<RcLazyConfig>>`, `From<RcLazy>`** for single-threaded to thread-safe conversions (all would require eager evaluation). Lower priority since the thread-safe source conversions are already covered.
