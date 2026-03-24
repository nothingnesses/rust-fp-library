# Analysis: `Step<A, B>` (`fp-library/src/types/step.rs`)

## Summary

`Step<A, B>` is a two-variant enum (`Loop(A)` / `Done(B)`) used as the control type for tail-recursive monadic computations (`MonadRec`). It is structurally isomorphic to `Either<A, B>` (or `Result<B, A>` with swapped convention), but uses domain-specific naming that communicates its purpose in the `tail_rec_m` protocol. The file is approximately 2,255 lines, with roughly 60% dedicated to HKT type class implementations and their documentation, and 20% to tests.

## Design Assessment

### What Works Well

1. **Clear naming convention.** `Loop` and `Done` immediately communicate the semantics within `tail_rec_m`. Compared to a generic `Either<A, B>` or `Left`/`Right`, there is zero ambiguity about which variant continues iteration and which terminates.

2. **Comprehensive type class coverage.** The file provides three HKT brands:
   - `StepBrand` for bifunctor operations.
   - `StepLoopAppliedBrand<A>` for functor/monad over the `Done` type (fixing `Loop`).
   - `StepDoneAppliedBrand<B>` for functor/monad over the `Loop` type (fixing `Done`).

   This mirrors the PureScript approach where `Either` has both `Functor` (over `Right`) and `Bifunctor` instances.

3. **Dual-axis type class instances.** Both partially-applied brands implement the full stack: `Functor`, `Pointed`, `Lift`, `Semiapplicative`, `Semimonad`, `Foldable`, `Traversable`. This symmetry is principled and thorough.

4. **Concrete inherent methods.** The `Step` type provides direct methods (`map_loop`, `map_done`, `bimap`, `bind`, `bind_loop`, `fold_right`, `fold_left`, `fold_map`, `bi_fold_right`, `bi_fold_left`, `bi_fold_map`, `bi_traverse`) that mirror the type class methods. This lets users work ergonomically without needing to specify brand types.

5. **Property-based testing.** Functor laws, bifunctor laws, and monad laws are all checked via QuickCheck, with an `Arbitrary` instance.

6. **Derives are appropriate.** `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash` are all correct for a simple two-variant generic enum. `Copy` is especially valuable since `Step` is frequently pattern-matched in tight loops.

7. **`serde` support** is gated behind a feature flag, which is the correct approach.

### Issues and Concerns

#### 1. Missing `Applicative` and `Monad` trait implementations

Both `StepLoopAppliedBrand` and `StepDoneAppliedBrand` implement all the component traits (`Pointed`, `Semiapplicative`, `ApplyFirst`, `ApplySecond`, `Semimonad`) but never actually implement the marker traits `Applicative` or `Monad`. This means:

- `Step` cannot be used where an `Applicative` or `Monad` bound is required.
- The test section labels "Monad Laws for StepLoopAppliedBrand" / "Monad Laws for StepDoneAppliedBrand" are misleading, since `Monad` is not actually implemented.
- `MonadRec` is not implemented for either `Step` brand (which makes sense since `Step` is not itself a monad for recursion purposes, but the absence of `Applicative`/`Monad` marker traits still seems like an oversight).

**Recommendation:** Add explicit `impl Applicative for StepLoopAppliedBrand<LoopType>` and `impl Monad for StepLoopAppliedBrand<LoopType>` (and the `Done` counterpart) marker implementations, or document why they are intentionally omitted.

#### 2. Missing extraction/accessor methods

The type provides `is_loop()` and `is_done()` predicates but lacks:

- `unwrap_loop() -> A` / `unwrap_done() -> B` (panicking accessors, like `Option::unwrap`).
- `loop_or(default: B) -> B` / `done_or(default: A) -> A` (default-returning accessors).
- `into_result() -> Result<B, A>` (conversion to `Result`, since the types are isomorphic).
- `from_result(Result<B, A>) -> Step<A, B>` (conversion from `Result`).

The `tail_rec_m` consumers always immediately pattern-match on `Step`, so the missing accessors are not blocking. But for general-purpose use of `Step` as a bifunctor (which the rich type class implementations invite), the lack of these utility methods is a gap.

**Recommendation:** At minimum, add `From<Result<B, A>> for Step<A, B>` and the reverse conversion. This is zero-cost and improves interop. Panicking accessors are lower priority.

#### 3. `Semiapplicative` and `Lift` short-circuit semantics are asymmetric

For `StepLoopAppliedBrand`, `apply` and `lift2` short-circuit on the **first** `Loop` encountered (left-biased). This mirrors how `Result`'s `Applicative` works (first error wins). However, this means:

```rust
lift2::<StepLoopAppliedBrand<i32>, _, _, _>(|x, y| x + y, Step::Done(1), Step::Loop(2))
// returns Step::Loop(2), which is the SECOND argument's loop value
```

vs.

```rust
lift2::<StepLoopAppliedBrand<i32>, _, _, _>(|x, y| x + y, Step::Loop(3), Step::Loop(2))
// returns Step::Loop(3), which is the FIRST argument's loop value
```

This is correct and standard (it matches `Either`'s left-biased behavior), but it silently discards the other `Loop` value. This is an inherent limitation of the `Applicative` approach (as opposed to a validation-style combinator that would collect all errors). No action required, but worth noting.

#### 4. `LoopType: 'static` bound on brands

The `impl_kind!` and type class implementations for `StepLoopAppliedBrand<LoopType>` and `StepDoneAppliedBrand<DoneType>` require `'static` on the fixed type parameter. This is a consequence of the HKT machinery and is consistent with how other brands in the library work, but it means `Step<&'a str, B>` cannot use the `StepDoneAppliedBrand<&'a str>` HKT path. This is an inherent limitation of the Brand pattern, not a bug.

#### 5. `Semimonad::bind` uses `Fn` not `FnOnce`

The `bind` inherent method on `Step` takes `FnOnce`, which is optimal. But the `Semimonad::bind` type class method takes `impl Fn(A) -> ...` (non-consuming). This is consistent with the library's design choice (type class methods use `Fn` for composability), but it means the type class path is slightly less flexible than the inherent method. This is a library-wide design decision, not a `Step`-specific issue.

#### 6. Documentation quality

The documentation is generally excellent, with `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` on nearly every method. A few observations:

- The module-level doc example for `step.rs` shows `Step` being used as a pure data type (not within `tail_rec_m`). This is fine as a basic usage example but could also show the primary use case (within `tail_rec_m`) to orient readers.
- The `bi_traverse` inherent method has a `where Step<C, D>: Clone` bound. This is always satisfied when both `C: Clone` and `D: Clone`, which are already required by the parameter bounds. The explicit where clause is redundant but harmless.
- Variant doc comments lack terminal periods: `"Continue the loop with a new value"` should be `"Continue the loop with a new value."` for consistency with the rest of the codebase's punctuation standards.

#### 7. No `Display` implementation

`Step` derives `Debug` but does not implement `Display`. For a type that is primarily used in control flow, `Display` is not critical. But if `Step` is ever used as a user-facing value (e.g., in error messages), a `Display` impl would be helpful.

#### 8. `bi_traverse` takes `Fn` not `FnOnce`

The inherent `bi_traverse` method takes `impl Fn(A) -> ...` rather than `impl FnOnce(A) -> ...`. Since `Step` has at most one value of each type, `FnOnce` would be sufficient and more permissive for callers. The likely reason is consistency with the type class version (which uses `Fn` for composability), but the inherent method could be more flexible.

### Edge Cases

1. **Zero-sized types.** `Step<(), ()>` is a simple boolean-like type. All operations work correctly; `bimap`, `fold_right`, etc. all handle this trivially. `Copy` derive ensures this is zero-cost.

2. **Same type for both parameters.** `Step<i32, i32>` (which is the common case in `tail_rec_m` when `A = S` and `B` differs) is well-handled. The `PartialEq` derive works correctly here.

3. **Recursive types.** `Step<Step<A, B>, C>` or `Step<A, Step<B, C>>` are valid and work correctly due to the structural nature of the enum. No special handling is needed.

4. **Large types.** Since `Step` is an enum, its size is `max(size_of::<A>(), size_of::<B>()) + discriminant`. For large types, this could waste memory in the unused variant. This is standard Rust enum behavior and not a concern.

## Relationship with `Free` and `MonadRec`

`Step` is used exclusively through pattern matching in the `tail_rec_m` implementations:

- **`ThunkBrand`**: `Thunk::tail_rec_m` evaluates the thunk, matches on `Step::Loop`/`Step::Done` in a `loop`, and is stack-safe.
- **`Trampoline`**: `Trampoline::tail_rec_m` uses `bind` + `defer` to trampoline each step, achieving stack safety for deep recursion.
- **`TryThunkErrAppliedBrand`**: Matches on `Ok(Step::Loop(...))`, `Ok(Step::Done(...))`, `Err(e)`, adding error short-circuiting.
- **`TryThunkOkAppliedBrand`**: Matches on `Err(Step::Loop(...))`, `Err(Step::Done(...))`, `Ok(a)`, recursing over the error channel.
- **`TryTrampoline`**: Delegates to `Trampoline::tail_rec_m` with a `Result<S, E>` state wrapper.

`Step` does NOT appear in `Free::evaluate` or `Free::bind`; the `Free` monad uses its own `FreeInner` enum (`Pure`/`Wrap`/`Map`/`Bind`) for its internal control flow. `Step` only enters the picture when `Trampoline` (which is `Free<ThunkBrand, A>`) is used with `tail_rec_m`.

The ergonomics are good: users write `Step::Loop(next_state)` and `Step::Done(result)`, which is clear and concise. The naming convention aligns well with the loop-based mental model.

## Alternatives Considered

1. **Using `Result<B, A>` directly.** This would eliminate the need for a custom type, but `Ok`/`Err` semantics do not communicate "continue"/"stop" clearly. `Step` is more self-documenting.

2. **Using `Either<A, B>`.** The library does not have an `Either` type, so this is not an option. Even if it did, `Loop`/`Done` naming is more ergonomic for the recursion use case.

3. **Using `ControlFlow<B, A>` from `std::ops`.** `ControlFlow::Continue(A)` / `ControlFlow::Break(B)` is semantically identical to `Step::Loop(A)` / `Step::Done(B)`. The standard library type exists precisely for this purpose. However, `ControlFlow` does not implement `Copy`, `Hash`, or `serde`, and the library's HKT machinery requires custom brands anyway. Using `Step` allows deriving these traits and maintaining full control over the type class implementations. That said, providing a `From<ControlFlow<B, A>>` conversion would improve interop.

4. **Collapsing `Step` into a method on the step function's return type.** Some libraries encode the loop/done distinction directly in the monad (e.g., returning `M<Either<A, B>>`). This is essentially what `Step` does, just with a named type. The current approach is cleaner.

## Recommendations (Prioritized)

1. **Add `Applicative` and `Monad` marker trait impls** for both partially-applied brands. This is likely an oversight, since all component traits are already implemented.
2. **Add `From` conversions** between `Step<A, B>` and `Result<B, A>`, and between `Step<A, B>` and `std::ops::ControlFlow<B, A>`.
3. **Add terminal periods** to variant doc comments for punctuation consistency.
4. **Consider using `FnOnce`** for the inherent `bi_traverse` method (the type class version must use `Fn`, but the inherent method could be more permissive).
5. **Low priority:** Add `Display` impl, `unwrap_loop`/`unwrap_done` accessors, and `loop_or`/`done_or` default-returning accessors.

## Verdict

The `Step` type is well-designed for its purpose. The naming is clear, the type class coverage is thorough (modulo the missing marker traits), the tests are comprehensive with property-based law checking, and the documentation follows the project's standards. The main actionable items are the missing `Applicative`/`Monad` marker impls and the lack of standard library interop conversions. The overall implementation is solid and consistent with the library's design patterns.
