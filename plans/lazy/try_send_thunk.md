# TrySendThunk Analysis

**File:** `fp-library/src/types/try_send_thunk.rs`
**Type:** `TrySendThunk<'a, A, E>` (thread-safe fallible deferred computation)
**Lines:** ~1,600 (including tests)

## 1. Type Design

```rust
pub struct TrySendThunk<'a, A, E>(SendThunk<'a, Result<A, E>>);
```

TrySendThunk wraps `SendThunk<'a, Result<A, E>>`, which itself wraps `Box<dyn FnOnce() -> Result<A, E> + Send + 'a>`. This is a two-layer newtype: TrySendThunk -> SendThunk -> Box.

**Assessment: Correct and consistent.**

The design follows the library's established pattern exactly:

| Type | Wraps |
|------|-------|
| `TryThunk<'a, A, E>` | `Thunk<'a, Result<A, E>>` |
| `TrySendThunk<'a, A, E>` | `SendThunk<'a, Result<A, E>>` |

This is the right approach. `TrySendThunk` relates to `SendThunk` the same way `TryThunk` relates to `Thunk`. The newtype adds error-aware combinators (`map`, `bind`, `map_err`, `bimap`, `catch`, `catch_with`) that understand `Result` semantics, while delegating the actual closure management and `Send` invariant to `SendThunk`.

**Trade-off acknowledged:** The two-layer indirection (TrySendThunk -> SendThunk -> Box) is zero-cost at runtime since both newtypes are repr-transparent. There is no performance concern.

## 2. HKT Support

**Contrary to the task description, TrySendThunk DOES have a brand and HKT support, but it is limited.**

### What exists

The file defines:

```rust
impl_kind! {
    for TrySendThunkBrand {
        type Of<'a, E: 'a, A: 'a>: 'a = TrySendThunk<'a, A, E>;
    }
}
```

The brand `TrySendThunkBrand` exists in `brands.rs` and the `impl_kind!` invocation maps it to `TrySendThunk`. The parameter ordering is `(E, A)`, matching `TryThunkBrand` and `ResultBrand` conventions.

### What is missing

Unlike `TryThunk`, which has three brands and extensive trait implementations:

| Brand | TryThunk | TrySendThunk |
|-------|----------|--------------|
| Bifunctor brand (`*Brand`) | `TryThunkBrand` with `Bifunctor`, `Bifoldable` | `TrySendThunkBrand` exists but has **no trait impls** |
| Error-applied brand (`*ErrAppliedBrand<E>`) | `TryThunkErrAppliedBrand<E>` with `Functor`, `Pointed`, `Lift`, `ApplyFirst`, `ApplySecond`, `Semiapplicative`, `Semimonad`, `MonadRec`, `Foldable`, `WithIndex`, `FunctorWithIndex`, `FoldableWithIndex` | **Does not exist** |
| Ok-applied brand (`*OkAppliedBrand<A>`) | `TryThunkOkAppliedBrand<A>` with `Functor`, `Pointed`, `Semimonad`, `Foldable`, `WithIndex`, `FunctorWithIndex`, `FoldableWithIndex` | **Does not exist** |

### Why the gap exists

The documentation on the struct explains this clearly: standard HKT traits like `Functor`, `Pointed`, `Semimonad`, and `Semiapplicative` cannot be implemented because their signatures do not require `Send` on the mapping or binding closures. Composing a `SendThunk<Result<A, E>>` with a non-`Send` closure would violate the `Send` invariant.

This is the same fundamental limitation that `SendThunk` faces, and it is correctly identified.

### Should it have more HKT support?

**No, the current state is correct.** The `Send` bound mismatch is a genuine type system constraint, not an oversight. Possible future directions:

- A hypothetical `SendFunctor` / `SendSemimonad` trait family that requires `Send` on closures could enable HKT-style programming for `Send` types. But this would be a significant library-wide addition.
- The bifunctor brand `TrySendThunkBrand` currently has no trait implementations. `Bifunctor` and `Bifoldable` could technically be implemented if their closure arguments were required to be `Send`, but the current trait definitions do not require this. This is consistent; the brand exists purely for type-level identification.

## 3. Type Class Implementations

### What TrySendThunk implements (via trait)

| Trait | Notes |
|-------|-------|
| `Deferrable<'a>` | Eagerly evaluates the closure (since `Deferrable::defer` does not require `Send`). |
| `SendDeferrable<'a>` | Truly deferred; wraps the `Send` closure in a new `SendThunk`. |
| `Semigroup` | Requires `A: Semigroup + Send`, `E: Send`. Short-circuits on error. |
| `Monoid` | Requires `A: Monoid + Send`, `E: Send`. Returns `Ok(Monoid::empty())`. |
| `Debug` | Prints `"TrySendThunk(<unevaluated>)"` without forcing. |
| `From<TryThunk>` | Eagerly evaluates (TryThunk is not Send). |
| `From<Result>` | Wraps via `SendThunk::pure`. |
| `From<SendThunk>` | Maps the value through `Ok`. |
| `From<ArcLazy>` | Eagerly evaluates and clones; wraps as `Ok`. |
| `From<TryTrampoline>` | Eagerly evaluates (Trampoline is not Send). |
| `From<ArcTryLazy>` | Eagerly evaluates and clones the result. |

### What TrySendThunk implements (via inherent methods)

| Method | Analogous Trait |
|--------|----------------|
| `map` | `Functor::map` |
| `map_err` | (error-channel functor) |
| `bimap` | `Bifunctor::bimap` |
| `bind` | `Semimonad::bind` |
| `pure` / `ok` | `Pointed::pure` |
| `err` | (error-channel pointed) |
| `defer` | `Deferrable::defer` (but with Send) |
| `catch` | (error recovery, same E) |
| `catch_with` | (error recovery, different E) |
| `lift2` | `Lift::lift2` |
| `then` | `ApplySecond::apply_second` |
| `tail_rec_m` | `MonadRec::tail_rec_m` |
| `arc_tail_rec_m` | (Arc-wrapped variant for non-Clone closures) |
| `catch_unwind_with` | (panic recovery with custom handler) |
| `catch_unwind` | (panic recovery to String, E=String only) |
| `into_inner` | (unwrap newtype) |
| `evaluate` | (force computation) |
| `into_arc_try_lazy` | (convert to memoized ArcTryLazy) |

### Comparison with TryThunk

TryThunk has all of the above inherent methods **plus** full HKT trait implementations via `TryThunkErrAppliedBrand<E>` and `TryThunkOkAppliedBrand<A>`:

- `Functor`, `Pointed`, `Lift`, `Semiapplicative`, `Semimonad`, `MonadRec`, `ApplyFirst`, `ApplySecond` on the error-applied brand.
- `Functor`, `Pointed`, `Semimonad`, `Foldable`, `FoldableWithIndex`, `FunctorWithIndex`, `WithIndex` on both applied brands.
- `Bifunctor`, `Bifoldable` on the bifunctor brand.

TrySendThunk has **none** of these trait impls. This is the major difference, and it is justified by the `Send` constraint.

### Comparison with SendThunk

SendThunk has trait implementations that TrySendThunk lacks:

- `Foldable` (via `SendThunkBrand`).
- `FoldableWithIndex` (via `SendThunkBrand`).
- `WithIndex` (via `SendThunkBrand`).

TrySendThunk does not implement `Foldable` or any indexed variant. This is a gap worth examining:

**Missing `Foldable`:** `TryThunk` implements `Foldable` via `TryThunkErrAppliedBrand<E>`, which folds over the success channel (returning `initial` on error). TrySendThunk could in principle provide the same behavior via an error-applied brand, but it cannot because the applied brand does not exist. This is a consequence of the fundamental HKT limitation.

## 4. Thread Safety

### Send bounds: Correct

- The struct wraps `SendThunk<'a, Result<A, E>>`, which wraps `Box<dyn FnOnce() -> Result<A, E> + Send + 'a>`.
- The `+ Send` bound on the inner closure ensures the entire `TrySendThunk` is `Send`.
- All inherent methods that compose closures (map, bind, map_err, bimap, catch, catch_with, etc.) require `+ Send` on their closure arguments.
- All methods that produce values from `A` or `E` require the respective `Send` bounds.

### Deferrable implementation: Subtly correct but potentially surprising

The `Deferrable::defer` implementation **eagerly evaluates** the closure:

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self
where Self: Sized {
    f()  // eager!
}
```

This is because `Deferrable::defer` does not require `Send` on `f`, so wrapping `f` inside a `SendThunk` would be unsound. The `SendDeferrable::send_defer` implementation correctly defers:

```rust
fn send_defer(f: impl FnOnce() -> Self + Send + 'a) -> Self {
    TrySendThunk(SendThunk::new(move || f().evaluate()))
}
```

This matches `SendThunk`'s own `Deferrable` implementation, which also eagerly evaluates for the same reason. The pattern is consistent across the library.

### Test coverage of thread safety

The test suite includes:
- `test_is_send`: Static assertion that `TrySendThunk<'static, i32, String>: Send`.
- `test_send_across_thread`: Actually spawns a thread and evaluates.
- `test_into_arc_try_lazy_thread_safety`: Spawns a thread with a cloned ArcTryLazy.

This is adequate.

## 5. Code Duplication

### High duplication with TryThunk

Nearly every inherent method on `TrySendThunk` is a mechanical copy of the corresponding `TryThunk` method with `Thunk` replaced by `SendThunk` and `+ Send` bounds added. The method bodies are structurally identical:

| Method | TryThunk body | TrySendThunk body |
|--------|---------------|-------------------|
| `new` | `TryThunk(Thunk::new(f))` | `TrySendThunk(SendThunk::new(f))` |
| `pure` | `TryThunk(Thunk::pure(Ok(a)))` | `TrySendThunk(SendThunk::pure(Ok(a)))` |
| `ok` | `TryThunk(Thunk::pure(Ok(a)))` | `TrySendThunk(SendThunk::pure(Ok(a)))` |
| `err` | `TryThunk(Thunk::pure(Err(e)))` | `TrySendThunk(SendThunk::pure(Err(e)))` |
| `bind` | `TryThunk(self.0.bind(...))` | `TrySendThunk(self.0.bind(...))` |
| `map` | `TryThunk(self.0.map(...))` | `TrySendThunk(self.0.map(...))` |
| `map_err` | `TryThunk(self.0.map(...))` | `TrySendThunk(self.0.map(...))` |
| `bimap` | `TryThunk(self.0.map(...))` | `TrySendThunk(self.0.map(...))` |
| `catch` | `TryThunk(self.0.bind(...))` | `TrySendThunk(self.0.bind(...))` |
| `catch_with` | `TryThunk(Thunk::new(...))` | `TrySendThunk(SendThunk::new(...))` |
| `evaluate` | `self.0.evaluate()` | `self.0.evaluate()` |
| `lift2` | `self.bind(...)` | `self.bind(...)` |
| `then` | `self.bind(...)` | `self.bind(...)` |
| `tail_rec_m` | loop pattern | identical loop pattern |
| `arc_tail_rec_m` | Arc wrapper pattern | identical Arc wrapper pattern |
| `catch_unwind_with` | `TryThunk::new(...)` | `TrySendThunk::new(...)` |
| `catch_unwind` | delegates to `catch_unwind_with` | delegates to `catch_unwind_with` |

The test suites are also near-duplicates.

### Why this duplication exists

Rust's type system does not offer a way to parameterize a struct over "the presence or absence of a `Send` bound on its inner closure." You cannot write a generic `TryThunkBase<Inner>` that works for both `Thunk` and `SendThunk` without losing the ergonomic method signatures.

Macro-based deduplication (a declarative macro that generates both `TryThunk` and `TrySendThunk`) is theoretically possible but would significantly harm readability and make the documentation generation (which relies on procedural macros like `#[document_signature]`) more complex.

### Verdict

The duplication is an acceptable cost of Rust's type system constraints. The code is consistent and well-maintained. If the two implementations ever diverge in a bug-introducing way, the nearly identical test suites should catch it.

## 6. Documentation Quality

**Excellent.** The documentation is thorough and well-structured:

- **Module-level docs:** Clear one-line description, cross-references to related types (`SendThunk`, `TryThunk`, `ArcTryLazy`).
- **Struct-level docs:** Explains the wrapper pattern, HKT representation, HKT trait limitations (with reasoning), when-to-use guidance, algebraic properties (monad laws), stack safety warning, and Traversable limitation.
- **Method-level docs:** Every method has `#[document_signature]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` with working code examples.
- **Trait impl docs:** Each `From`, `Deferrable`, `SendDeferrable`, `Semigroup`, `Monoid`, and `Debug` implementation has full documentation.

### Minor observations

- The `pure` method requires `A: Send + 'a, E: Send + 'a` where TryThunk's `pure` requires only `'a`. This is correct (the value goes into a `SendThunk`), but the extra bounds are not explicitly called out in the documentation as a difference from TryThunk.
- The `Deferrable::defer` eagerly evaluates, and this is documented in its doc comment ("The thunk `f` is called eagerly because `Deferrable::defer` does not require `Send` on the closure."). Good.

## 7. Issues, Limitations, and Design Flaws

### 7.1 Deferrable::defer is eager (semantic mismatch)

The `Deferrable` trait exists to create lazily deferred values. TrySendThunk's implementation eagerly evaluates the closure `f()`. While this is the only sound option given the trait signature, it means that code written generically over `Deferrable` will get surprising behavior when instantiated with `TrySendThunk`. The trait contract arguably does not guarantee laziness, but users may expect it.

This is the same issue as `SendThunk`'s `Deferrable` implementation. It is a library-wide design tension, not specific to TrySendThunk.

### 7.2 No Foldable implementation

`SendThunk` implements `Foldable` (and `FoldableWithIndex`) via `SendThunkBrand`. `TryThunk` implements `Foldable` via `TryThunkErrAppliedBrand<E>`. `TrySendThunk` implements neither, despite having a brand (`TrySendThunkBrand`).

The bifunctor brand cannot support `Foldable` (it requires a unary type constructor). An error-applied brand (`TrySendThunkErrAppliedBrand<E>`) would be needed, but then implementing `Foldable` on it would require implementing `Functor` first (since the HKT machinery requires the brand to participate in the kind system). And `Functor` cannot be implemented due to the `Send` constraint.

This is a genuine limitation.

### 7.3 No Bifunctor or Bifoldable on TrySendThunkBrand

The brand exists but has zero trait implementations. `TryThunkBrand` implements both `Bifunctor` and `Bifoldable`. The same `Send` constraint issue applies: `Bifunctor::bimap` does not require `Send` on its closure arguments.

The brand currently serves only as a type-level tag. It could be removed without losing functionality, or it could be kept for future use if `Send`-aware bifunctor traits are introduced.

### 7.4 bind requires A: Send (potentially surprising)

```rust
pub fn bind<B>(
    self,
    f: impl FnOnce(A) -> TrySendThunk<'a, B, E> + Send + 'a,
) -> TrySendThunk<'a, B, E>
where
    A: Send + 'a,
    B: Send + 'a,
    E: Send + 'a,
```

The `A: Send` bound on `bind` is necessary because the Err path constructs `SendThunk::pure(Err(e))`, which requires `E: Send`, and the overall closure capturing `self` (which contains a `Result<A, E>`) requires `A: Send`. This is correct but means you cannot use `bind` if `A` is not `Send`, even though the error path does not use `A`. This is an inherent limitation of the approach.

By contrast, TryThunk's `bind` has no `Send` bounds at all.

### 7.5 The `defer` inherent method differs subtly from `Deferrable::defer`

The inherent `TrySendThunk::defer` method:
```rust
pub fn defer(f: impl FnOnce() -> TrySendThunk<'a, A, E> + Send + 'a) -> Self {
    TrySendThunk(SendThunk::new(move || f().evaluate()))
}
```

This is truly lazy (wraps in `SendThunk::new`). But `Deferrable::defer` is eager. Users calling `TrySendThunk::defer(...)` directly get laziness; users calling `Deferrable::defer(...)` get eagerness. The naming collision could be confusing, though the trait method and inherent method have different signatures (`Send` vs no `Send` on the closure).

### 7.6 No `into_rc_try_lazy` conversion

TryThunk has `into_rc_try_lazy` (converting to single-threaded memoized). TrySendThunk only has `into_arc_try_lazy`. This is reasonable since a `Send` type should convert to a `Send`-compatible lazy type. Including `into_rc_try_lazy` would be technically possible but would lose the `Send` guarantee, which would be unexpected.

### 7.7 `into_arc_try_lazy` uses internal details

```rust
pub fn into_arc_try_lazy(self) -> ArcTryLazy<'a, A, E> {
    TryLazy(self.0.into_arc_lazy().0)
}
```

This reaches into `TryLazy`'s internal tuple field `.0` and `ArcLazy`'s internal `.0` to construct the result. This works but couples the implementation to the internal representation of `TryLazy` and `ArcLazy`. If those types' internals change, this breaks. A constructor method on `TryLazy` or `ArcTryLazy` would be more robust.

## 8. Alternatives and Improvements

### 8.1 Macro-based deduplication

A declarative macro could generate both `TryThunk` and `TrySendThunk` from a single template, parameterized by the base thunk type and the Send bound. This would eliminate the duplication identified in section 5. However, it would make the code harder to read and would complicate the documentation generation pipeline. Not recommended unless the types diverge further and maintenance becomes burdensome.

### 8.2 SendFunctor / SendSemimonad trait family

A parallel hierarchy of type classes requiring `Send` on closure arguments would allow TrySendThunk to participate in HKT abstractions. This is a significant design decision affecting the entire library. Traits like `SendFunctor`, `SendSemimonad`, `SendApplicative` would mirror the existing hierarchy but with `+ Send` bounds.

**Pros:** Enables generic programming over Send-capable types.
**Cons:** Doubles the trait hierarchy; users must choose which family to program against; may not compose well with the existing non-Send hierarchy.

### 8.3 Remove TrySendThunkBrand

Since the brand has no trait implementations, it adds no value beyond type-level tagging. Removing it would simplify the code. However, keeping it is harmless and preserves the option of adding trait implementations in the future.

### 8.4 Add From<TrySendThunk> for TryThunk

Currently, conversion goes TryThunk -> TrySendThunk (via `From`, eagerly evaluated). The reverse direction (TrySendThunk -> TryThunk) is not implemented. Since `SendThunk` has `into_inner()` which erases the `Send` bound via unsizing coercion, a zero-cost conversion is possible:

```rust
impl<'a, A: 'a, E: 'a> From<TrySendThunk<'a, A, E>> for TryThunk<'a, A, E> {
    fn from(t: TrySendThunk<'a, A, E>) -> Self {
        TryThunk(Thunk::from(t.0))  // uses existing From<SendThunk> for Thunk
    }
}
```

This would enable round-tripping and allow users to "downgrade" a TrySendThunk back to a TryThunk for use in non-Send HKT contexts.

### 8.5 Encapsulate `into_arc_try_lazy` construction

Replace the direct field access with a proper constructor on `ArcTryLazy`:

```rust
pub fn into_arc_try_lazy(self) -> ArcTryLazy<'a, A, E> {
    ArcTryLazy::from_send_thunk(self.0)  // hypothetical method
}
```

This would reduce coupling to internal representations.

## 9. Test Coverage Summary

The test module contains 32 tests covering:

- Basic constructors: `ok`, `err`, `new`, `pure`, `defer`.
- Combinators: `map`, `map_err`, `bimap`, `bind`, `catch`, `catch_with`, `lift2`, `then`.
- Error propagation: map on error, bind on error, catch recovery fails.
- Conversions: `From<TryThunk>`, `From<Result>`, `From<SendThunk>`, `From<ArcLazy>`, `From<TryTrampoline>`, `From<ArcTryLazy>`, `into_inner`.
- Memoization: `into_arc_try_lazy` (value + thread safety).
- Algebra: `Semigroup::append` (success, error, short-circuit), `Monoid::empty`.
- Traits: `Deferrable`, `SendDeferrable`, `Debug`.
- Thread safety: `is_send` static check, `send_across_thread`.
- Stack safety: `tail_rec_m` (success, early error, 100k iterations), `arc_tail_rec_m` (success, early error).
- Panic handling: `catch_unwind_with` (panic + no panic), `catch_unwind` (panic + no panic).

**Missing test coverage:**
- No test for `From<ArcTryLazy>` with error path (there is `test_from_arc_try_lazy_err` which covers it, so this is fine).
- No property-based tests verifying monad laws, though these may exist in a separate property test file.

## 10. Summary Table

| Aspect | Rating | Notes |
|--------|--------|-------|
| Type design | Good | Correct wrapper pattern, consistent with library conventions. |
| HKT support | Acceptable | Brand exists but has no trait impls; justified by Send constraint. |
| Type class impls | Good | Comprehensive inherent methods mirror TryThunk; Deferrable + SendDeferrable + Semigroup + Monoid. |
| Thread safety | Good | Send bounds correctly propagated throughout. |
| Code duplication | High but acceptable | Nearly identical to TryThunk; unavoidable without macros or Send-parameterized generics. |
| Documentation | Excellent | Thorough, with working examples on every method. |
| Test coverage | Very good | 32 tests covering all major paths. |
| Overall | Solid | Well-implemented with clear limitations documented. Main improvement opportunity is `From<TrySendThunk> for TryThunk` and encapsulating `into_arc_try_lazy`. |
