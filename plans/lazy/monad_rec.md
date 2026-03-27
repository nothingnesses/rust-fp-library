# MonadRec Analysis

File: `fp-library/src/classes/monad_rec.rs`

## 1. Design: Faithfulness to PureScript/Haskell

PureScript's `MonadRec` has this signature:

```purescript
class Monad m <= MonadRec m where
  tailRecM :: forall a b. (a -> m (Step a b)) -> a -> m b
```

The Rust `MonadRec` trait (line 59) mirrors this faithfully:

```rust
pub trait MonadRec: Monad {
    fn tail_rec_m<'a, A: 'a, B: 'a>(
        func: impl Fn(A) -> Apply!(<Self as Kind!(...)>::Of<'a, Step<A, B>>) + Clone + 'a,
        initial: A,
    ) -> Apply!(<Self as Kind!(...)>::Of<'a, B>);
}
```

**Key differences from PureScript:**

- **`Clone` bound on `func`**: PureScript does not need this because closures are implicitly shareable in a garbage-collected runtime. In Rust, the step function is called repeatedly in a loop, so it must either be `Clone` (to survive multiple calls via `Fn`) or wrapped in `Arc`. The `Clone` bound is a reasonable Rust adaptation. The `Fn` (not `FnOnce`) bound already implies re-callability, so `Clone` is needed only for types that store `func` and need to copy it (like `Trampoline::tail_rec_m`'s inner `go` function which recursively passes `f`). For `Thunk`'s implementation (which uses a simple loop), the `Clone` bound is unnecessary since `Fn` alone suffices for repeated calls in a loop. This is a minor over-constraint.

- **Lifetime parameter `'a`**: PureScript has no lifetimes. The `'a` parameter is necessary for Rust's ownership model and allows `Thunk` (which supports non-`'static` lifetimes) to implement the trait. This is well-designed.

- **`Step` type**: Matches PureScript's `Step` exactly with `Loop(a)` and `Done(b)` variants.

**Verdict**: Faithful translation with appropriate Rust adaptations.

## 2. Implementation Quality and Stack Safety

### ThunkBrand (line 639-689 of `thunk.rs`)

```rust
fn tail_rec_m<'a, A: 'a, B: 'a>(f, a) -> Thunk<'a, B> {
    Thunk::new(move || {
        let mut current = a;
        loop {
            match f(current).evaluate() {
                Step::Loop(next) => current = next,
                Step::Done(res) => break res,
            }
        }
    })
}
```

This wraps the entire recursion in a single `Thunk::new`, deferring evaluation. Inside, it uses an imperative `loop` that eagerly evaluates each step. This is **genuinely stack-safe** because:
- Each iteration calls `f(current)` which returns a `Thunk<Step<A, B>>`.
- `.evaluate()` forces that thunk (one stack frame deep at most, assuming shallow thunks).
- The loop replaces `current` in place; no recursive calls accumulate stack frames.

**Caveat** (correctly documented at line 641-644): If the step function `f` builds deep `bind` chains inside the returned thunk, the inner `.evaluate()` can overflow. This is inherent to `Thunk`'s non-stack-safe `bind`, not a defect in `tail_rec_m`.

### Trampoline (line 465-486 of `trampoline.rs`)

```rust
pub fn tail_rec_m<S: 'static>(f, initial) -> Self {
    fn go<A, B, F>(f: F, a: A) -> Trampoline<B>
    where F: Fn(A) -> Trampoline<Step<A, B>> + Clone + 'static {
        Trampoline::defer(move || {
            let result = f(a);
            result.bind(move |step| match step {
                Step::Loop(next) => go(f, next),
                Step::Done(b) => Trampoline::pure(b),
            })
        })
    }
    go(f, initial)
}
```

This is stack-safe because:
- `Trampoline::defer` wraps the recursive call in a thunk, preventing stack growth.
- `result.bind(...)` uses `Free`'s O(1) bind (CatList-based).
- The recursive `go(f, next)` call happens inside a `defer` closure, so it produces a `Trampoline` value rather than recursing on the call stack.
- `Free::evaluate` (line 716-762 of `free.rs`) drives everything with an iterative loop.

This is **fully stack-safe** for any recursion depth. The `Clone` bound on `f` is essential here because `go` recursively passes `f` to itself, requiring ownership transfer at each step.

### TryThunkErrAppliedBrand (line 1027-1077 of `try_thunk.rs`)

Uses the same imperative loop pattern as `ThunkBrand`. Correctly short-circuits on `Err`. Stack-safe under the same caveats as `ThunkBrand`.

### TryThunkOkAppliedBrand (line 1776-1829 of `try_thunk.rs`)

Mirrors `TryThunkErrAppliedBrand` but recurses over the error channel. Loops on `Err(Step::Loop(...))`, terminates on `Err(Step::Done(...))`, and short-circuits on `Ok(a)`. Correct and stack-safe.

### Non-HKT inherent methods

`SendThunk`, `Trampoline`, and `TryTrampoline` all provide inherent `tail_rec_m` methods (not via the trait) because they cannot implement HKT traits. These follow the same patterns and are correct.

## 3. Laws

The documentation (lines 53-58) states two "laws":

1. **Equivalence**: `tail_rec_m(f, a)` produces the same result as the recursive definition.
2. **Safety varies**: `Thunk` is stack-safe for `tail_rec_m` but not for deep `bind` chains. `Trampoline` is guaranteed stack-safe for all operations.

**Issues:**

- The PureScript `MonadRec` has a single law: **stack safety**. The class exists specifically to guarantee that `tailRecM` can be used for unbounded recursion without stack overflow. The law is roughly: `tailRecM f a` must complete without stack overflow for any `f` that always eventually returns `Done`.

- The "Equivalence" law stated in the docs is really just a correctness requirement (the function does what it says), not a `MonadRec`-specific law. Every function should be equivalent to its specification.

- The "Safety varies" note is honest but unusual. In PureScript, all `MonadRec` instances must be stack-safe; that is the whole point of the class. Here, `ThunkBrand`'s `tail_rec_m` IS stack-safe (the loop is iterative), but the caveat about deep `bind` chains inside the step function is worth noting.

- **No formal law tests exist.** There are no property-based tests (QuickCheck) verifying MonadRec laws. The existing tests are example-based (factorial, counting to N, stress tests with 200K iterations). These are good smoke tests but do not systematically verify the algebraic properties.

- **Missing law**: PureScript documents the law as:
  ```
  tailRecM (\a -> pure (Done a)) a = pure a
  ```
  This "identity" or "pure Done" law is not stated anywhere.

## 4. API Surface

**Well-designed aspects:**

- The trait has a single method (`tail_rec_m`), matching PureScript's minimal design.
- A free function wrapper `tail_rec_m` (line 131-138) provides convenient dispatch.
- The module-level doc example (lines 5-28) is clear and demonstrates a practical use case (factorial).
- The method-level doc example (lines 74-89) is simple and shows the counting pattern.

**Observations:**

- `tail_rec_m` is NOT re-exported in `functions.rs`. The `generate_function_re_exports!` macro at `functions.rs:25` auto-generates re-exports from `src/classes`, but the module example at `monad_rec.rs:9` uses `functions::tail_rec_m`, suggesting it IS re-exported. This should work if the macro scans `monad_rec.rs` and finds the free function. The doc test passes, confirming it works.

- PureScript provides `tailRecM2` and `tailRecM3` convenience functions for multi-argument recursion. These are absent here, but Rust's tuple syntax `(a, b, c)` makes them less necessary (as shown in the factorial example using `(n, acc)`).

- There is no `forever` combinator (which PureScript derives from `MonadRec`). This could be useful but is a minor omission.

## 5. Consistency with Other Type Classes

**Superclass**: `MonadRec: Monad` (line 59) correctly mirrors the PureScript hierarchy.

**Pattern consistency:**
- The trait follows the same pattern as other type classes: trait definition in `classes/`, free function wrapper in the same file, re-export via `functions.rs`.
- The documentation attributes (`#[document_signature]`, `#[document_type_parameters]`, etc.) are consistently applied.
- The `inner` module pattern with `#[fp_macros::document_module]` matches other class modules.

**Naming**: `tail_rec_m` uses snake_case for PureScript's `tailRecM`, following Rust conventions consistently.

**Implementors**: Only `ThunkBrand`, `TryThunkErrAppliedBrand<E>`, and `TryThunkOkAppliedBrand<A>` implement the HKT trait. `Trampoline`, `SendThunk`, `TryTrampoline`, and `TrySendThunk` provide inherent `tail_rec_m` methods instead. This split is consistent with the library's general approach to types that cannot satisfy HKT lifetime requirements.

## 6. Limitations

1. **Only three HKT implementors**: `ThunkBrand`, `TryThunkErrAppliedBrand`, and `TryThunkOkAppliedBrand`. Standard types like `Option`, `Vec`, `Result`, and `Identity` do not implement `MonadRec`, even though they trivially could (their `bind` is already stack-safe for `tail_rec_m` purposes). In PureScript, `Identity`, `Maybe`, `Either`, and `Effect` all implement `MonadRec`.

2. **No implementation for `OptionBrand`, `VecBrand`, `ResultErrAppliedBrand`, `IdentityBrand`**: These types have inherently stack-safe `bind` (no deferred computation), so `tail_rec_m` could simply be implemented as a loop that unwraps and re-applies. This limits the genericity of code written against `MonadRec`.

3. **`Trampoline` cannot implement the trait**: Due to `'static` requirements conflicting with HKT lifetime polymorphism. This is a fundamental limitation of the Brand pattern, not a design flaw. The inherent method is the correct workaround.

4. **`Clone` bound on `func`**: As noted above, this is unnecessary for the `ThunkBrand` and `TryThunk*` implementations (which use a simple loop where `Fn` suffices). It is only needed for `Trampoline`'s recursive `go` pattern. The trait-level `Clone` bound forces all implementations to accept `Clone` closures, which can be inconvenient. `Trampoline` works around this for its inherent method by also providing `arc_tail_rec_m`.

5. **No `arc_tail_rec_m` variant on the trait**: The trait only provides the `Clone`-based version. Types that provide `Arc`-wrapped alternatives do so as inherent methods, not through the trait. This means generic code using `MonadRec` cannot opt into the `Arc` pattern.

6. **`SendThunkBrand` has no `MonadRec` impl**: `SendThunk` provides an inherent `tail_rec_m` but cannot implement the HKT trait because the trait's closure parameters lack `Send` bounds.

## 7. Documentation

**Accurate aspects:**
- The module-level docs (lines 1-28) correctly describe the purpose and provide a working example.
- The "Important Design Note" (lines 43-50) clearly explains the `Thunk` vs `Trampoline` distinction.
- The caveat about `Thunk`'s partial stack safety is honest and important.

**Issues:**
- Lines 46-47: "Trampoline CANNOT implement this trait (requires `'static`)." This is accurate but could benefit from a brief explanation of why `'static` prevents HKT trait implementation (the `Kind` trait requires lifetime polymorphism).
- The "Laws" section (lines 53-58) is weak. It lists an "Equivalence" law that is really just correctness, and a "Safety varies" note that is more of a caveat than a law. The actual MonadRec law from the literature (the "pure Done" identity) is not stated. See section 3 above.
- Line 57: "Safety varies" is listed as a "law" but it is an implementation note, not an algebraic law.
- The free function docs (lines 98-138) duplicate the trait method docs closely, which is fine for discoverability.

## Summary of Findings

| Aspect | Rating | Notes |
|--------|--------|-------|
| Design faithfulness | Good | Faithful PureScript translation with appropriate Rust adaptations. |
| Stack safety | Good | `ThunkBrand` uses iterative loop; `Trampoline` uses `defer`+`bind`. Both correct. |
| Laws | Weak | No formal MonadRec law stated; no property-based tests. |
| API surface | Good | Clean, minimal. Missing `MonadRec` impls for standard types (`Option`, `Vec`, etc.). |
| Consistency | Good | Follows library patterns for trait definition, free functions, and documentation. |
| Documentation | Adequate | Accurate on stack safety caveats; laws section needs improvement. |

### Recommended Improvements

1. **Add `MonadRec` implementations for `OptionBrand`, `VecBrand`, `ResultErrAppliedBrand<E>`, and `IdentityBrand`.** These are trivial (iterative loop) and would significantly increase the utility of generic `MonadRec` code.

2. **State the actual MonadRec law.** Replace the current "laws" section with the PureScript law: `tail_rec_m(|a| pure(Step::Done(a)), x)` equals `pure(x)`. The stack-safety guarantee should be stated as a class invariant, not a "law."

3. **Add property-based tests** for the MonadRec law across all implementations.

4. **Consider relaxing the `Clone` bound.** For implementations that use a simple loop (all current HKT impls), `Fn` alone is sufficient. The `Clone` bound exists to support the `Trampoline`-style recursive pattern, but `Trampoline` cannot implement the HKT trait anyway. Removing `Clone` from the trait would simplify usage. If a future HKT-compatible type needs `Clone`, it can be added then.

5. **Document why `Trampoline` cannot implement the trait** more explicitly in the trait-level docs, referencing the `'static` vs lifetime-polymorphism conflict.
