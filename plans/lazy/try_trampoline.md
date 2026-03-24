# TryTrampoline Analysis

**File:** `fp-library/src/types/try_trampoline.rs`

## 1. Design

### Core Design Decision: Newtype over `Trampoline<Result<A, E>>`

`TryTrampoline<A, E>` is a single-field newtype wrapping `Trampoline<Result<A, E>>`. This is the correct design. The alternative (discussed in section 5) of having users work with `Trampoline<Result<A, E>>` directly would be workable but ergonomically inferior.

The newtype provides:

- **Named constructors** (`ok`, `err`, `new`) that remove Result-wrapping boilerplate.
- **Error-aware `bind`** that short-circuits on `Err`, giving proper `MonadError`-like semantics.
- **`catch`** for error recovery, the dual of `bind`.
- **`map_err`** for transforming the error channel.
- **`tail_rec_m`** that correctly threads `Result` through the `Step` loop, short-circuiting on error.

The relationship to `Trampoline` is clean: `TryTrampoline` delegates entirely to the inner `Trampoline` and never reimplements the trampoline/free monad machinery. This is sound.

### Relationship to `TryThunk`

`TryTrampoline` mirrors `TryThunk` in the same way `Trampoline` mirrors `Thunk`:

| Property | `TryThunk<'a, A, E>` | `TryTrampoline<A, E>` |
|---|---|---|
| Underlying | `Thunk<'a, Result<A, E>>` | `Trampoline<Result<A, E>>` |
| Lifetime | `'a` (supports borrowing) | `'static` only |
| Stack safety | Partial (bind chains unsafe) | Full |
| HKT support | Yes (brands, Functor, Monad, etc.) | None |
| Memoized | No | No |

This is consistent with the library's broader architecture.

## 2. Implementation Correctness

### `bind` Implementation

```rust
pub fn bind<B: 'static>(
    self,
    f: impl FnOnce(A) -> TryTrampoline<B, E> + 'static,
) -> TryTrampoline<B, E> {
    TryTrampoline(self.0.bind(|result| match result {
        Ok(a) => f(a).0,
        Err(e) => Trampoline::pure(Err(e)),
    }))
}
```

This is correct. On `Err`, it produces a `Trampoline::pure(Err(e))`, which short-circuits without executing `f`. The continuation is allocated inside the `Trampoline`'s `CatList`, so bind remains O(1) in construction.

**Potential concern:** Each `Err` short-circuit in a long `bind` chain still traverses the remaining `CatList` entries, each time matching on `Err` and immediately re-wrapping. This is O(n) in the number of remaining continuations when an error occurs, but this is inherent to the `Free` monad approach and not a bug.

### `tail_rec_m` Implementation

```rust
pub fn tail_rec_m<S: 'static>(
    f: impl Fn(S) -> TryTrampoline<Step<S, A>, E> + Clone + 'static,
    initial: S,
) -> Self {
    TryTrampoline(Trampoline::tail_rec_m(
        move |state: Result<S, E>| match state {
            Err(e) => Trampoline::pure(Step::Done(Err(e))),
            Ok(s) => f(s).0.map(|result| match result {
                Ok(Step::Loop(next)) => Step::Loop(Ok(next)),
                Ok(Step::Done(a)) => Step::Done(Ok(a)),
                Err(e) => Step::Done(Err(e)),
            }),
        },
        Ok(initial),
    ))
}
```

This is correct and well-designed. The state type for the inner `Trampoline::tail_rec_m` is `Result<S, E>`. On error, it immediately produces `Step::Done(Err(e))`. On success, it unwraps the `TryTrampoline` step result and re-wraps it appropriately. The `Err(e) => Step::Done(Err(e))` arm ensures error short-circuiting.

**Stack safety is preserved:** The `Trampoline::tail_rec_m` itself uses `defer` internally (visible in the `go` helper), so arbitrarily deep fallible recursion will not overflow the stack. The tests confirm this with 100,000 iteration tests.

### `catch` Implementation

```rust
pub fn catch(
    self,
    f: impl FnOnce(E) -> TryTrampoline<A, E> + 'static,
) -> Self {
    TryTrampoline(self.0.bind(|result| match result {
        Ok(a) => Trampoline::pure(Ok(a)),
        Err(e) => f(e).0,
    }))
}
```

Correct. This is the error-channel dual of `bind`. On `Ok`, it rewraps without calling `f`. On `Err`, it delegates to the recovery function.

### `From` Implementations

- `From<Trampoline<A>>`: Wraps via `task.map(Ok)`. Correct.
- `From<Lazy<'static, A, Config>>`: Eagerly evaluates and clones. Correct but note the eager evaluation semantics.
- `From<TryLazy<'static, A, E, Config>>`: Eagerly evaluates and clones both Ok and Err. Correct.
- `From<TryThunk<'static, A, E>>`: Uses `TryTrampoline::new(move || thunk.evaluate())`. Correct; defers evaluation.
- `From<Result<A, E>>`: Uses `Trampoline::pure(result)`. Correct.

**Subtle issue with `From<Lazy>` and `From<TryLazy>`:** These conversions eagerly evaluate the `Lazy`/`TryLazy` at conversion time, then wrap the result in a `Trampoline::pure`. This means the conversion itself forces computation. The doc comment on `From<TryLazy>` says "clones both the success and error values eagerly," which is accurate. However, this could surprise users who expect the conversion to remain lazy. The `From<TryThunk>` conversion, by contrast, correctly defers evaluation. This asymmetry is worth noting but is arguably correct since `Lazy` is designed for memoization and may have already been evaluated.

### `memoize_arc` Eager Evaluation

```rust
pub fn memoize_arc(self) -> crate::types::ArcTryLazy<'static, A, E>
where
    A: Send + Sync,
    E: Send + Sync,
{
    let result = self.evaluate();
    crate::types::ArcTryLazy::new(move || result)
}
```

This eagerly evaluates the `TryTrampoline`, then wraps the result in an `ArcTryLazy`. The documentation correctly states "evaluated eagerly because its inner closures are not `Send`." This matches the same pattern in `Trampoline::memoize_arc`. No bug here.

## 3. Consistency

### Consistent with `Trampoline`

`TryTrampoline` faithfully mirrors `Trampoline`'s API surface:

| `Trampoline` | `TryTrampoline` |
|---|---|
| `pure(a)` | `ok(a)` |
| N/A | `err(e)` |
| `new(f)` | `new(f)` (where `f` returns `Result`) |
| `defer(f)` | `defer(f)` |
| `map(f)` | `map(f)`, `map_err(f)` |
| `bind(f)` | `bind(f)` (short-circuits on Err) |
| N/A | `catch(f)` |
| `lift2` | `lift2` |
| `then` | `then` |
| `tail_rec_m` | `tail_rec_m` |
| `arc_tail_rec_m` | `arc_tail_rec_m` |
| `evaluate()` | `evaluate()` (returns `Result`) |
| `memoize()` | `memoize()` (returns `RcTryLazy`) |
| `memoize_arc()` | `memoize_arc()` (returns `ArcTryLazy`) |
| `append` | Missing |
| `empty` | Missing |

### Consistent with `TryThunk`

`TryTrampoline` provides the same error-handling combinators (`map`, `map_err`, `bind`, `catch`, `lift2`, `then`), plus `catch_unwind` and `catch_unwind_with` for panic recovery. `TryThunk` also has `catch_unwind`/`catch_unwind_with`, so these are consistent.

### Missing Features vs `Trampoline`

- **`append`/`Semigroup` and `empty`/`Monoid`**: `Trampoline` has these, `TryTrampoline` does not. For `Result<A, E>` where `A: Semigroup`, a natural `append` would combine the success values and short-circuit on error. This is a minor gap.

### No HKT Support

Neither `Trampoline` nor `TryTrampoline` has a Brand type or implements any type classes (`Functor`, `Monad`, etc.) via the HKT machinery. The `'static` requirement conflicts with the `Kind` trait's `'a` lifetime parameter. The CLAUDE.md explicitly notes this: "`Trampoline` cannot implement HKT traits due to its `'static` requirement." This is a known, deliberate limitation.

## 4. Limitations

1. **`'static` requirement**: All types `A` and `E` must be `'static`. No borrowing is possible. This is inherited from `Trampoline`/`Free` and is fundamental to the type-erased continuation approach.

2. **No HKT integration**: Cannot be used generically where `Functor<F>` or `Monad<F>` is required. Users must work with inherent methods only.

3. **No `Semigroup`/`Monoid` instances**: Unlike `Trampoline`, there are no `append` or `empty` methods for combining `TryTrampoline` values.

4. **Error propagation cost in `bind` chains**: When an error occurs mid-chain, the remaining continuations in the `CatList` are each visited and immediately short-circuited. This is O(remaining-continuations), not O(1). For very long chains that fail early, this could be a performance concern, though each step is cheap (just a pattern match and re-wrap).

5. **`From<Lazy>` and `From<TryLazy>` are eager**: These conversions force evaluation at conversion time, which may surprise users.

6. **No `bimap` or `Bifunctor` support**: `TryThunk` implements `Bifunctor` via HKT, but `TryTrampoline` only has `map` and `map_err` as separate methods. A combined `bimap(f_ok, f_err)` method is absent.

7. **`catch` is not stack-safe for deep error recovery chains**: If a user chains many `.catch(...)` calls, each adds a continuation to the CatList. This is fine for construction (O(1) per call), but a deeply nested error recovery pattern where each `catch` itself fails and triggers the next `catch` would still be O(n) in the CatList traversal. This is not a bug, just a characteristic worth noting.

## 5. Alternatives: Could This Just Be `Trampoline<Result<A, E>>`?

Yes, `Trampoline<Result<A, E>>` is functionally equivalent. The question is whether the newtype wrapper adds sufficient value.

**Arguments for the newtype (`TryTrampoline`):**

- `bind` on `Trampoline<Result<A, E>>` would map over the entire `Result`, not short-circuit on `Err`. Users would need to manually pattern-match in every `bind` callback. `TryTrampoline::bind` automates this.
- `tail_rec_m` with error short-circuiting requires the `Result<S, E>` state threading trick. This is non-trivial to get right and worth encapsulating.
- `catch`, `map_err`, `catch_unwind` have no analogue in plain `Trampoline`.
- The type name `TryTrampoline<A, E>` communicates intent better than `Trampoline<Result<A, E>>`.

**Arguments against:**

- It's another type to learn and maintain.
- It cannot participate in HKT generic programming anyway, so the ergonomic benefit is purely at the call site.
- A `MonadError` type class could theoretically provide these combinators generically for any `Monad` wrapping a `Result`.

**Verdict:** The newtype is well-justified. The error-aware `bind` and `tail_rec_m` alone provide enough value to warrant the type. The alternative of manually threading `Result` through `Trampoline` operations is error-prone and verbose.

## 6. Documentation

### Quality

The documentation is thorough. Every public method has:

- A short description.
- `document_signature`, `document_type_parameters`, `document_parameters`, `document_returns` annotations.
- At least one doc-tested example.

The module-level doc comment accurately describes the type as a wrapper around `Trampoline<Result<A, E>>`.

### Accuracy

All examples are correct and consistent with the implementation. The `tail_rec_m` example demonstrates both success and error paths. The `defer` example shows the stack-safe recursion pattern.

### Gaps

- The `Clone` bound on `tail_rec_m`'s function parameter is documented, along with the `arc_tail_rec_m` alternative. Good.
- The `From<Lazy>` eager evaluation behavior could be more prominently documented. The `From<TryLazy>` impl does mention eager cloning, but `From<Lazy>` does not explicitly state it evaluates the `Lazy`.
- There is no documentation explaining *when* to use `TryTrampoline` vs `TryThunk`. This guidance exists in the crate-level docs (`lib.rs`) but not in the module or type documentation. A "When to use" section (like `Trampoline` has for `Trampoline` vs `Thunk`) would be helpful.

## 7. Test Coverage

The test suite is comprehensive:

- Basic operations: `ok`, `err`, `new`, `map`, `map_err`, `bind`, `catch`.
- Conversions: `From<Trampoline>`, `From<Lazy>`, `From<TryLazy>`, `From<TryThunk>`, `From<Result>`.
- `lift2` and `then` with success, first-error, and second-error cases.
- `tail_rec_m` with success, error, stack safety (100k iterations), and error-after-many-iterations.
- `arc_tail_rec_m` with success and error paths.
- `catch_unwind` and `catch_unwind_with` with both panicking and non-panicking closures.
- `!Send` type tests with `Rc<T>`.
- QuickCheck property tests for Functor laws, Monad laws, and error short-circuiting.

No obvious gaps in test coverage.

## 8. Summary

`TryTrampoline` is a well-designed, correctly implemented fallible wrapper around `Trampoline<Result<A, E>>`. It provides genuine ergonomic value through error-aware `bind`, `catch`, and `tail_rec_m`. The implementation correctly delegates to `Trampoline` without reimplementing trampoline machinery, and stack safety is preserved for all operations.

**Minor issues to consider:**

- Adding `append`/`Semigroup` support for consistency with `Trampoline`.
- Adding a `bimap` method for convenience.
- Adding a "When to use" section to the type-level documentation.
- Clarifying eager evaluation semantics in the `From<Lazy>` conversion doc.
