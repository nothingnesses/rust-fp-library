# Analysis: `fp-library/src/types/step.rs`

## Overview

`Step<A, B>` is a two-variant enum (`Loop(A)` / `Done(B)`) used by `MonadRec` to signal whether a tail-recursive monadic computation should continue or terminate. It is structurally isomorphic to `Result<B, A>` (and `Either<A, B>` in Haskell/PureScript) but carries domain-specific semantics for recursion control.

The file is 2478 lines, comprising the enum definition, inherent methods, HKT brand/kind integrations, type class implementations for three brands (`StepBrand`, `StepLoopAppliedBrand<A>`, `StepDoneAppliedBrand<B>`), conversions, and an extensive test suite with property-based (QuickCheck) law verification.

---

## 1. Design

**Verdict: Sound, well-structured.**

The core design is correct. `Step<A, B>` is the standard approach for `MonadRec`, matching PureScript's `Step` from `Control.Monad.Rec.Class`. The choice to make it a standalone type rather than reusing `Result` is the right call: it provides clear domain semantics (`Loop`/`Done` vs `Ok`/`Err`) and avoids conflating error handling with recursion control.

The three-brand HKT strategy is well-considered:

- `StepBrand`: bifunctor-level operations over both type parameters.
- `StepLoopAppliedBrand<A>`: functor/monad over the `Done` (B) position, with `Loop` type fixed. This is the primary brand used by `MonadRec`, where `pure` wraps into `Done`.
- `StepDoneAppliedBrand<B>`: functor/monad over the `Loop` (A) position, with `Done` type fixed. This is the symmetric dual, where `pure` wraps into `Loop`.

This mirrors how `Result` is handled in the library (`ResultBrand`, `ResultErrAppliedBrand<E>`, `ResultOkAppliedBrand<T>`), ensuring consistency.

**One design nuance worth noting**: `StepLoopAppliedBrand` is the "natural" brand for `MonadRec` usage, since `MonadRec::tail_rec_m` produces `Step<A, B>` where `B` is the type that varies (it is the final result type). `Pointed::pure` for `StepLoopAppliedBrand` correctly wraps into `Done`, meaning `pure(x)` signals termination, which is the correct semantic for `MonadRec`.

---

## 2. Correctness

**Verdict: No bugs found.**

All implementations are logically correct:

- `map_loop` and `map_done` correctly target their respective variants and pass through the other.
- `bimap` correctly applies `f` to `Loop` and `g` to `Done`.
- `bind` for `StepLoopAppliedBrand` correctly chains on `Done` and short-circuits on `Loop`.
- `bind` for `StepDoneAppliedBrand` correctly chains on `Loop` and short-circuits on `Done`.
- `Foldable` implementations return `initial` for the "empty" variant and apply the function for the "full" variant.
- `Traversable::traverse` correctly wraps the unchanged variant in `F::pure` and maps over the active variant.
- `Semiapplicative::apply` and `Lift::lift2` correctly handle all variant combinations, returning the first short-circuiting variant when not both sides are the active variant.
- All `From` conversions are correct and consistent in both directions.
- The `Bifoldable` and `Bitraversable` implementations delegate correctly to inherent methods.

The QuickCheck property tests verify functor laws (identity, composition), bifunctor laws (identity, composition), monad laws (left identity, right identity, associativity), and round-trip properties for conversions. This is thorough.

---

## 3. Type Class Instances

**Implemented (for `StepBrand`):**
- `Bifunctor`
- `Bifoldable`
- `Bitraversable`

**Implemented (for `StepLoopAppliedBrand<A>` and `StepDoneAppliedBrand<B>`):**
- `Functor`
- `Lift`
- `Pointed`
- `ApplyFirst`
- `ApplySecond`
- `Semiapplicative`
- `Semimonad`
- `Foldable`
- `Traversable`
- `Applicative` (via blanket impl)
- `Monad` (via blanket impl)

**Not implemented but potentially applicable:**

- **`Semigroup` / `Monoid`**: Could be implemented for `Step<A, B>` where `A: Semigroup` (or `B: Semigroup` depending on the bias). PureScript's `Either` has `Semigroup` when the right side is a semigroup. However, `Step` is primarily a control-flow type, not a data type meant for combination, so omitting these is reasonable.

- **`FunctorWithIndex` / `FoldableWithIndex` / `TraversableWithIndex`**: Could use a unit index (like `Option` does) for the applied brands. These are low priority since `Step` typically contains at most one element per variant.

- **`Alt`**: `StepLoopAppliedBrand<A>` could implement `Alt` (try the first, if `Loop`, try the second). This would give `Step` "first success" semantics. However, `Step` is not a container type where alternative choice is natural, so omitting this is defensible.

- **`Compactable` / `Filterable`**: These require `Option`-based filtering. Not applicable to `Step`'s semantics.

- **`MonadRec`**: Not implemented for `StepLoopAppliedBrand` or `StepDoneAppliedBrand`. This would allow `Step` itself to be used as a `MonadRec`, which is theoretically possible (just run the loop inline). However, `Step` is a pure data type with no deferred computation, so stack safety is not a concern, and implementing `MonadRec` on it would be unusual.

Overall, the set of implemented type classes is comprehensive and appropriate for the type's role.

---

## 4. API Surface

**Verdict: Well-designed, minor gaps.**

The inherent methods provide a clean, ergonomic API:

- Predicates: `is_loop()`, `is_done()`.
- Mapping: `map_loop()`, `map_done()`, `bimap()`.
- Folding: `bi_fold_right()`, `bi_fold_left()`, `bi_fold_map()`, `fold_right()`, `fold_left()`, `fold_map()`.
- Binding: `bind()` (over Done), `bind_loop()` (over Loop).
- Traversal: `bi_traverse()`.
- Conversions: `From<Result>`, `Into<Result>`, `From<ControlFlow>`, `Into<ControlFlow>`.

**Potentially missing operations (minor):**

- **`unwrap_done()` / `unwrap_loop()`**: Panicking extractors, analogous to `Result::unwrap()` / `unwrap_err()`. These are convenient for tests and prototyping. However, the library may deliberately avoid panicking APIs, which is a valid choice.

- **`done()` / `loop_val()`**: Non-panicking extractors returning `Option<B>` / `Option<A>`, analogous to `Result::ok()` / `Result::err()`. These would be useful for pattern-matching avoidance.

- **`map_loop_or()` / `map_done_or()`**: Extractors with defaults, analogous to `Result::map_or()`.

- **`traverse()` (single-variant)**: A method `traverse_done()` or similar that traverses only the `Done` side with an effectful function. The current `bi_traverse` covers the general case, and the HKT `Traversable` covers the applied-brand case, so this gap is minor.

- **`swap()`**: Swaps `Loop(a)` to `Done(a)` and vice versa. This is a simple utility that can be useful, analogous to `Either.swap()`.

None of these are critical; the existing API covers the essential operations, and users can always pattern-match directly.

---

## 5. HKT Integration

**Verdict: Correct.**

The `impl_kind!` invocations are correct:

```rust
// Bifunctor-level (two type params, no lifetime)
impl_kind! { for StepBrand { type Of<A, B> = Step<A, B>; } }

// Bifunctor-level (two type params, with lifetime)
impl_kind! { for StepBrand { type Of<'a, A: 'a, B: 'a>: 'a = Step<A, B>; } }

// Applied brands (one type param, with lifetime)
impl_kind! { impl<LoopType: 'static> for StepLoopAppliedBrand<LoopType> { type Of<'a, B: 'a>: 'a = Step<LoopType, B>; } }
impl_kind! { impl<DoneType: 'static> for StepDoneAppliedBrand<DoneType> { type Of<'a, A: 'a>: 'a = Step<A, DoneType>; } }
```

The `'static` bound on `LoopType` / `DoneType` in the applied brands is the standard limitation of the Brand pattern in this library (the baked-in type parameter must outlive all possible `'a`). This is consistent with how `ResultErrAppliedBrand<E>` requires `E: 'static`.

The brand types in `brands.rs` (`StepBrand`, `StepLoopAppliedBrand<B>`, `StepDoneAppliedBrand<A>`) are correctly defined as zero-sized marker types with standard derives.

---

## 6. Documentation

**Verdict: Thorough and accurate, with minor issues.**

Positive aspects:
- Every public method has a doc comment, `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]` with tested code examples.
- The module-level doc comment explains the purpose with a working example.
- The HKT representation section in the `Step` enum docs clearly explains the three-brand strategy.
- Serde support is documented.
- Conversion impls have clear docs explaining the mapping.

Minor issues:
- The `fold_right` and `fold_left` inherent methods document their parameter as `"The step value."` via the `impl` block's `#[document_parameters]`, but this refers to `self` which is not typically listed as a parameter in Rust doc conventions. This is a library-wide convention though, not specific to `Step`.
- The `bi_traverse` method's doc says "See `Bitraversable::bi_traverse` for the type class version" but the inherent method predates the trait method in the file. This is just a cross-reference, not an issue.

---

## 7. Consistency

**Verdict: Highly consistent with library patterns.**

The file follows the library's conventions closely:

- Uses `#[fp_macros::document_module]` / `mod inner` / `pub use inner::*` pattern.
- Tab indentation matching `rustfmt.toml`.
- Import style matches (grouped, one per line).
- Type class implementations follow the same structure as `Result`, `Option`, `Pair`, etc.
- Applied brand naming convention (`StepLoopAppliedBrand` / `StepDoneAppliedBrand`) matches `ResultErrAppliedBrand` / `ResultOkAppliedBrand`.
- `Clone` bounds on `Lift` and `Semiapplicative` impls match other types.
- The test module structure (unit tests, property-based law tests, trait marker verification) is consistent.
- `QuickCheck` `Arbitrary` implementation is provided in the test module.

---

## 8. Limitations and Issues

### Inherent Limitations

1. **`'static` requirement on applied brand type parameters**: `StepLoopAppliedBrand<A>` requires `A: 'static`, and `StepDoneAppliedBrand<B>` requires `B: 'static`. This is a fundamental limitation of the Brand pattern and cannot be avoided without changing the HKT encoding.

2. **No `PartialOrd` / `Ord` derives**: The enum derives `Clone, Copy, Debug, PartialEq, Eq, Hash` but not `PartialOrd` or `Ord`. For a control-flow type this is fine; ordering on `Step` values has no clear semantic meaning. If ordering were added, the question of whether `Loop < Done` or `Done < Loop` would be arbitrary.

3. **No `Default` derive**: `Step` has no natural default value (unlike `Option` which defaults to `None`). This is correct.

4. **Duplication between applied brands**: The `StepLoopAppliedBrand` and `StepDoneAppliedBrand` implementations are nearly symmetric mirrors of each other. This is roughly 800 lines of near-duplicate code. This is an inherent consequence of providing both orientations, and the library has the same pattern for `Result` (`ResultErrAppliedBrand` / `ResultOkAppliedBrand`). There is no obvious way to reduce this duplication without macros or higher-level abstraction.

5. **`Semimonad` without `Monad` for `StepDoneAppliedBrand`**: While `StepDoneAppliedBrand` gets `Monad` via blanket impl, its semantics are somewhat unusual: `pure` wraps into `Loop`, meaning "continue the computation." This is mathematically valid but semantically backwards from `MonadRec`'s perspective, where `Done` means termination. Users should prefer `StepLoopAppliedBrand` for `MonadRec`-related work. The `StepDoneAppliedBrand` monad is the dual and exists for completeness.

### No Issues Found

- No bugs or logic errors detected.
- No unsoundness concerns (the type is a simple enum with no unsafe code).
- The `From` conversions are correct and the mapping (`Loop <-> Err/Continue`, `Done <-> Ok/Break`) is semantically sensible.
- The `serde` feature gate is correctly applied.

### Summary

`Step` is a well-implemented, thoroughly tested, and correctly documented type that serves its purpose as the control type for `MonadRec`. The type class coverage is comprehensive for a two-variant sum type. The main trade-off is the ~800 lines of near-symmetric duplication between the two applied brands, which is an acceptable cost of the library's HKT encoding approach. No changes are required; potential enhancements (convenience extractors like `done() -> Option<B>`, a `swap()` method) are nice-to-haves rather than necessities.
