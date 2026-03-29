# Lazy Evaluation Hierarchy: Consolidated Analysis Summary

This document consolidates findings from individual analyses of all 18 components in the lazy evaluation hierarchy. It is organized by theme rather than by file, grouping related issues together and highlighting cross-cutting concerns.

---

## Table of Contents

1. [Architecture and Design Coherence](#1-architecture-and-design-coherence)
2. [Cross-Cutting Concerns](#2-cross-cutting-concerns)
3. [Significant Issues and Design Flaws](#3-significant-issues-and-design-flaws)
4. [Missing Implementations and Inconsistencies](#4-missing-implementations-and-inconsistencies)
5. [Strengths and Well-Designed Elements](#5-strengths-and-well-designed-elements)
6. [Suggested Improvements by Priority](#6-suggested-improvements-by-priority)

---

## 1. Architecture and Design Coherence

### 1.1 The Type Hierarchy

The hierarchy consists of 11 concrete types (plus supporting traits and brands) organized along three axes: infallible/fallible, Send/non-Send, and memoized/non-memoized/stack-safe.

| | Non-Send, non-memoized | Send, non-memoized | Non-Send, memoized | Send, memoized | Non-Send, stack-safe |
|---|---|---|---|---|---|
| **Infallible** | `Thunk<'a, A>` | `SendThunk<'a, A>` | `RcLazy<'a, A>` | `ArcLazy<'a, A>` | `Trampoline<A>` |
| **Fallible** | `TryThunk<'a, A, E>` | `TrySendThunk<'a, A, E>` | `RcTryLazy<'a, A, E>` | `ArcTryLazy<'a, A, E>` | `TryTrampoline<A, E>` |

Supporting types: `Free<F, A>` (the engine behind `Trampoline`), `CatList<A>` (the continuation queue for `Free`), `Step<A, B>` (control type for `MonadRec`).

### 1.2 HKT Coverage Gradient

The HKT support forms a deliberate gradient reflecting each type's capabilities:

- **Full HKT:** `ThunkBrand` (Functor through MonadRec, Evaluable, Foldable, indexed variants), `TryThunkErrAppliedBrand<E>` and `TryThunkOkAppliedBrand<A>` (full monadic stacks), `CatListBrand` (the richest brand in the hierarchy).
- **Partial HKT:** `LazyBrand<Config>` and `TryLazyBrand<E, Config>` (RefFunctor/SendRefFunctor, Foldable; cannot implement Functor due to `&A` evaluation), `SendThunkBrand` (Foldable only; cannot implement Functor due to Send constraints).
- **No HKT:** `Trampoline`, `TryTrampoline`, `Free` (all require `'static`, incompatible with the Kind trait's lifetime polymorphism).

This gradient is well-calibrated. The library does not force HKT support where it does not fit, and each limitation is documented with clear rationale.

### 1.3 Newtype Composition Pattern

The fallible types consistently wrap their infallible counterparts: `TryThunk` wraps `Thunk<Result<A, E>>`, `TrySendThunk` wraps `SendThunk<Result<A, E>>`, `TryTrampoline` wraps `Trampoline<Result<A, E>>`, and `TryLazy` wraps memoized `Result<A, E>`. This pattern maximizes code reuse (stack safety, CatList machinery, pointer management all come from the base types) while providing ergonomic error-aware combinators (`map`, `bind` with short-circuiting, `catch`, `catch_with`, `bimap`).

### 1.4 Config-Parameterized Unification

The `LazyConfig`/`TryLazyConfig` trait system unifies `RcLazy`/`ArcLazy` and `RcTryLazy`/`ArcTryLazy` under single brand definitions (`LazyBrand<Config>`, `TryLazyBrand<E, Config>`). This avoids duplicating brand definitions and all their type class impls, while remaining open to third-party extensions (e.g., `parking_lot`-based or async-aware cells).

---

## 2. Cross-Cutting Concerns

### 2.1 The Send Boundary Problem

The most pervasive tension in the hierarchy is between Rust's `Send` bound system and the HKT trait signatures. The library's HKT traits (`Functor::map`, `Semimonad::bind`, etc.) accept closures as `impl Fn(A) -> B + 'a` without a `Send` bound. This means:

- `SendThunk`, `TrySendThunk`, `ArcLazy`, and `ArcTryLazy` **cannot implement Functor, Monad, or any closure-accepting HKT trait**, because a caller could pass a non-Send closure that the type cannot store.
- The library uses three different patterns for the Send/non-Send split:
  - **Supertrait:** `SendDeferrable: Deferrable` (works because the eager-evaluation fallback allows Send types to satisfy the non-Send trait).
  - **Independent traits:** `RefFunctor` vs `SendRefFunctor` (cannot use supertrait because `ArcLazy::new` requires `Send` on the closure, preventing a valid `RefFunctor` impl).
  - **Inherent methods only:** `SendThunk` and `TrySendThunk` provide `map`, `bind`, `tail_rec_m` as inherent methods rather than through any trait.

This three-pattern approach is pragmatically correct but means there is no way to write code that is generic over "any mappable lazy type" spanning both Send and non-Send variants. The documentation correctly identifies this as a fundamental Rust constraint.

**Affected files:** brands.md, send_thunk.md, try_send_thunk.md, deferrable.md, send_deferrable.md, ref_functor.md, send_ref_functor.md, lazy.md.

### 2.2 The `'static` Constraint

`Free`, `Trampoline`, and `TryTrampoline` require `A: 'static` because `Free` uses `Box<dyn Any>` for type erasure in its CatList of continuations. This constraint:

- Prevents these types from having HKT brands (the Kind system requires lifetime polymorphism).
- Forces all closures passed to `bind`, `map`, `defer`, etc. to be `'static`.
- Creates a hard boundary: types that need borrowed data must use `Thunk`/`TryThunk` (with limited stack safety), while types that need stack safety must use `Trampoline`/`TryTrampoline` (with `'static` data only).

Additionally, partially-applied brands like `TryThunkErrAppliedBrand<E>` require `E: 'static`, preventing HKT-polymorphic code from working with borrowed error types. This is inherent to the Brand pattern's encoding.

**Affected files:** free.md, trampoline.md, try_trampoline.md, brands.md, evaluable.md.

### 2.3 The `Fn` vs `FnOnce` Tension

The library's HKT traits use `impl Fn` for closure parameters (to support multi-element containers like `Vec` where the function is called multiple times), but single-element types like `Thunk` only need `FnOnce`. This means:

- Users of HKT-polymorphic code must provide `Fn` closures even when working with `Thunk`, which only calls them once.
- The inherent methods on `Thunk`, `SendThunk`, etc. accept `FnOnce` for maximum flexibility.
- Both `RefFunctor` and `SendRefFunctor` correctly use `FnOnce` since memoized types evaluate at most once.

This is a fundamental design tension: the unified HKT approach sacrifices some per-type flexibility for the benefit of a single trait hierarchy.

**Affected files:** thunk.md, send_thunk.md, ref_functor.md, send_ref_functor.md, free.md.

### 2.4 The `Clone` Requirement Propagation

`Clone` bounds appear throughout the hierarchy for multiple reasons:

- **Memoized types:** `Lazy::evaluate()` returns `&A`, so extracting owned values requires cloning. This affects `Deferrable`, `Semigroup`, `Monoid`, and `Foldable` implementations for all `Lazy`/`TryLazy` types.
- **HKT trait signatures:** `Lift::lift2` and `Semiapplicative::apply` require `A: Clone` because multi-element containers need to reuse values. Single-element types like `Thunk` pay this cost unnecessarily.
- **`Trampoline::tail_rec_m`:** Requires `Clone + 'static` on the step function because each iteration captures `f` by value. The `arc_tail_rec_m` variant relaxes `Clone` via `Arc` wrapping.
- **Conversions:** `From<Lazy> for Trampoline` requires `A: Clone` to extract the memoized value.

**Affected files:** lazy.md, try_lazy.md, thunk.md, monad_rec.md, trampoline.md, deferrable.md.

### 2.5 Code Duplication

Substantial structural duplication exists across four axes:

1. **Send/non-Send pairs:** `Thunk`/`SendThunk` and `TryThunk`/`TrySendThunk` have nearly identical inherent methods differing only in `+ Send` bounds (~150 lines per pair).
2. **Infallible/Fallible pairs:** `Trampoline`/`TryTrampoline` and `Thunk`/`TryThunk` share the same combinator surface with added `Result` handling.
3. **Rc/Arc duplication within memoized types:** `RcLazy`/`ArcLazy` and `RcTryLazy`/`ArcTryLazy` duplicate methods with different `Send + Sync` bounds.
4. **HKT impls vs inherent methods:** Types without HKT brands (`Trampoline`, `TryTrampoline`, `SendThunk`, `TrySendThunk`) re-implement monadic operations as inherent methods.

This duplication is a known cost of Rust's type system (no way to parameterize over the presence of a `Send` bound, no way to abstract over trait objects with different auto-trait bounds). Macro-based deduplication is possible but would harm readability. The current approach is acceptable.

**Affected files:** send_thunk.md, try_send_thunk.md, try_trampoline.md, try_lazy.md, lazy.md.

### 2.6 Eager Evaluation in `Deferrable` for Send Types

Four types (`SendThunk`, `ArcLazy`, `TrySendThunk`, `ArcTryLazy`) implement `Deferrable::defer` by calling `f()` eagerly, because `Deferrable::defer` does not require `Send` on its closure parameter and these types need `Send` closures internally. This is:

- Documented in the `Deferrable` trait's Warning section.
- Mitigated by the `SendDeferrable` trait, which provides truly deferred evaluation with `Send`-bounded closures.
- Technically law-preserving (the transparency law `defer(|| x) == x` holds for values), but semantically surprising (side effects in `f` happen at defer-time, not at evaluation-time).

**Affected files:** deferrable.md, send_deferrable.md, lazy.md, send_thunk.md, try_send_thunk.md.

---

## 3. Significant Issues and Design Flaws

### 3.1 `TrySendThunkBrand` Is a Hollow Brand

`TrySendThunkBrand` has an `impl_kind!` (bifunctor signature) but zero type class implementations. No `Bifunctor`, no `Bifoldable`, no partially-applied brands. By contrast, `TryThunkBrand` has `Bifunctor`, `Bifoldable`, `Bitraversable`, plus two partially-applied brands with full monadic stacks. The brand currently serves no functional purpose and should either gain implementations or be removed.

### 3.2 `Evaluable` Has Only One Implementor

Only `ThunkBrand` implements `Evaluable`. This makes the trait's genericity over `F` in `Free<F, A>` purely theoretical. `Free` could have been hardcoded to `ThunkBrand` with identical observable behavior. Adding `IdentityBrand` as a second implementor would validate the abstraction. The trait also lacks documentation of the pure-extract law (`evaluate(pure(x)) == x`) that `Free::evaluate` implicitly relies on.

### 3.3 `Free`'s `Evaluable` Constraint Is Overly Broad

`Free<F, A>` requires `F: Evaluable` on all operations, including `pure`, `bind`, and `map`, which do not need to evaluate anything. Loosening the constraint to `F: Functor` on construction methods (and only requiring `Evaluable` on `evaluate`) would increase flexibility. `fold_free` already works with any functor via `NaturalTransformation`.

### 3.4 `hoist_free` Is Not Stack-Safe

`Free::hoist_free` recurses once per `Wrap` layer. For programmatically generated deep `Wrap` chains, this can overflow the stack. The `fold_free` alternative is stack-safe because it delegates to `MonadRec::tail_rec_m`. This is documented but represents a potential footgun.

### 3.5 `brands.rs` Depends on `types` Module

The import of `LazyConfig` and `TryLazyConfig` from `types` into `brands.rs` violates the stated dependency ordering (brands -> classes -> types). These config traits are more "class-like" than "type-like" and could be relocated to `classes/` to restore the intended layering.

### 3.6 Thunk's `MonadRec` Is Only Conditionally Stack-Safe

`ThunkBrand`'s `tail_rec_m` wraps the entire loop in a single `Thunk::new`. The loop itself is stack-safe, but if the step function builds deep `bind` chains inside the returned thunk, those chains blow the stack during evaluation. Users who reach for `MonadRec` expect unconditional stack safety, but `ThunkBrand` only provides it for "shallow" thunks. `Trampoline::tail_rec_m` provides unconditional safety via `Free`.

### 3.7 No Send-Safe Stack-Safe Recursion

There is no `SendTrampoline` or `SendFree`. Users needing both thread safety and stack safety have no direct option. `SendThunk::tail_rec_m` provides thread-safe recursion but with the same conditional stack safety as `ThunkBrand`. A `SendFree` would require `Send`-bounded continuations throughout `Free`, which would be a significant refactor.

---

## 4. Missing Implementations and Inconsistencies

### 4.1 Missing Trait Implementations

| Missing Implementation | Comparison | Priority |
|---|---|---|
| `FoldableWithIndex` for `TryLazyBrand<E, Config>` | `LazyBrand<Config>` has it; `TryLazyBrand` does not. Trivial addition (`Index = ()`). | High |
| `WithIndex` for `TryLazyBrand<E, Config>` | Prerequisite for `FoldableWithIndex`. | High |
| `Traversable` for `ThunkBrand` | Single-element Traversable is well-defined. `CatListBrand` has it. `Thunk` is `!Clone`, which may block this. | Medium |
| `Display` for `TryLazy` | `Lazy` has `Display`. `TryLazy` does not. | Low |
| `Bifunctor`/`Bifoldable` for `TrySendThunkBrand` | `TryThunkBrand` has both. `TrySendThunkBrand` has neither. | Low (blocked by Send constraints) |
| Cross-config conversions for `TryLazy` (`RcTryLazy <-> ArcTryLazy`) | `Lazy` has these conversions. `TryLazy` does not. | Low |
| Fix combinators for `TryLazy` | `Lazy` has `rc_lazy_fix`/`arc_lazy_fix`. `TryLazy` has none. | Low |

### 4.2 Missing Conversions

| Missing Conversion | Notes | Priority |
|---|---|---|
| `From<TrySendThunk> for TryThunk` | `From<SendThunk> for Thunk` exists (zero-cost unsizing coercion). The parallel for Try types is missing. | Medium |
| `From<Trampoline> for SendThunk` | The path `Trampoline -> Thunk -> SendThunk` exists via chaining, but a direct conversion (with eager evaluation) would be convenient. | Low |

### 4.3 Documentation Gaps

| Gap | Location | Priority |
|---|---|---|
| `Evaluable` does not document the pure-extract law | `evaluable.rs` | Medium |
| `MonadRec` states only the identity law; missing the unfolding/equivalence law | `monad_rec.rs` | Medium |
| `TryTrampoline` lacks algebraic properties section and limitations section | `try_trampoline.rs` | Medium |
| `SendDeferrable` has no property-based tests | `send_deferrable.rs` | Medium |
| `SendThunk` has no QuickCheck property tests for monad/functor laws on inherent methods | `send_thunk.rs` | Low |
| No documentation of nondeterministic termination caveat for `VecBrand`/`CatListBrand` MonadRec | `monad_rec.rs` | Low |

### 4.4 Naming Inconsistencies

- `Step` has asymmetric accessor names: `done()` vs `loop_val()`. A symmetric pair (`done`/`loop_val` or `done_val`/`loop_val`) would be more consistent.
- `Evaluable`'s parameter docs on the `ThunkBrand` impl use "eval" as a noun ("The eval to run"), while the trait-level doc consistently uses "evaluate."

---

## 5. Strengths and Well-Designed Elements

### 5.1 Overall Architecture

- **Principled trade-off documentation.** Every limitation (no HKT for Trampoline, no Functor for Lazy, eager Deferrable for Send types) is documented with clear reasoning, not just acknowledged as a gap.
- **Config-parameterized brands** (`LazyBrand<Config>`) avoid duplicating type class impls while supporting both Rc and Arc variants through a single generic implementation.
- **The newtype composition pattern** (TryThunk wraps Thunk, TryTrampoline wraps Trampoline) maximizes code reuse without introducing abstraction overhead.

### 5.2 CatList

- Pragmatic adaptation of PureScript's `CatList` using `VecDeque` (better cache locality, no bulk reversals) rather than a two-list queue.
- O(1) cached length (an enhancement over PureScript).
- Stack-safe iterative `Drop` with tested guarantee (100,000 right-associated appends).
- The richest type class surface in the hierarchy (Functor through Witherable, MonadRec, parallel variants).

### 5.3 Free Monad

- Correct "Reflection without Remorse" implementation with O(1) bind and iterative evaluation.
- Safe type erasure via `Box<dyn Any>` with runtime downcast (vs. PureScript's unsafe coercion).
- Stack-safe `Drop` implementation using a worklist pattern.
- Clean separation of concerns: `Free` handles stack safety and CatList management; `Evaluable` handles functor unwrapping; `Trampoline` provides the user-facing API.

### 5.4 RefFunctor / SendRefFunctor

- The `RefFunctor` trait is the correct Rust-specific adaptation for mapping over shared memoized values. The separation from `Functor` is well-justified by the `&A` vs `A` distinction.
- The independence of `RefFunctor` and `SendRefFunctor` (not a supertrait relationship) correctly prevents unsound implementations.
- Cache chain behavior is documented with appropriate warnings about memory accumulation.

### 5.5 Step Type

- Clean, zero-cost enum with `Loop`/`Done` variants that are self-documenting in MonadRec contexts.
- Three-brand HKT support (StepBrand, StepLoopAppliedBrand, StepDoneAppliedBrand) with full Monad + MonadRec towers.
- Bidirectional conversions with both `Result` and `ControlFlow`, with round-trip property tests.
- Derives `Copy` for zero-cost pattern matching in tight loops.

### 5.6 Documentation and Testing

- Every public method across all 18 files uses the library's documentation macro system (`#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]`).
- QuickCheck property tests verify algebraic laws (Functor, Monad, Semigroup, etc.) across most types.
- Stack safety stress tests (100,000+ to 200,000 iterations) exist for all relevant types.
- Conversion round-trip tests ensure the type conversion web is consistent.

### 5.7 Naming Consistency

- `XBrand` for simple types, `XBrand<Config>` for config-parameterized types.
- `Rc`/`Arc` prefixes for pointer-specific type aliases.
- `Try` prefix always comes before `Send` when both are present (`TrySendThunk`, not `SendTryThunk`).
- `Send` prefix for thread-safe variants consistently mirrors the non-Send names.

---

## 6. Suggested Improvements by Priority

### High Priority

1. **Add `FoldableWithIndex` and `WithIndex` for `TryLazyBrand<E, Config>`.** This is a trivial addition that restores parity with `LazyBrand` and fills an obvious gap in the hierarchy.

2. **Either implement type classes for `TrySendThunkBrand` or remove it.** The brand currently has zero implementations and serves no functional purpose. If `Bifunctor`/`Bifoldable` are blocked by the Send constraint, document this and consider removal.

3. **Document the unfolding/equivalence law for `MonadRec`.** The trait currently states only the identity law. The equivalence law (`tail_rec_m(f, a) == f(a) >>= match { Loop(a') => tail_rec_m(f, a'), Done(b) => pure(b) }`) is critical for correctness and should be explicitly stated.

4. **Document the pure-extract law for `Evaluable`.** The law `evaluate(pure(x)) == x` is implicitly relied upon by `Free::evaluate` but is not documented.

### Medium Priority

5. **Add `From<TrySendThunk> for TryThunk`.** This fills a gap in the conversion web; `From<SendThunk> for Thunk` already exists.

6. **Consider moving `LazyConfig`/`TryLazyConfig` trait definitions to `classes/`.** This restores the stated brands -> classes -> types dependency ordering while keeping concrete impls in `types/lazy.rs`.

7. **Relax `Free<F, A>`'s `Evaluable` constraint on construction methods.** Allow `pure`, `bind`, `map` to require only `F: Functor` (or no constraint at all), reserving `F: Evaluable` for `evaluate` only.

8. **Add `Traversable` for `ThunkBrand`.** A single-element Traversable is well-defined and useful for generic programming. (Note: may be blocked by `Thunk`'s `!Clone`; needs investigation.)

9. **Add QuickCheck property tests for `SendDeferrable` and `SendThunk`.** Currently, `Deferrable` has property tests but `SendDeferrable` does not. `Thunk` has QuickCheck law tests but `SendThunk` does not.

10. **Add algebraic properties and limitations sections to `TryTrampoline`'s documentation.** The type currently lacks the structured documentation that `TryThunk` and `Trampoline` have.

### Low Priority

11. **Add `Display` for `TryLazy`.** `Lazy` has it; `TryLazy` should too.

12. **Add cross-config conversions for `TryLazy`.** `Lazy` has `RcLazy <-> ArcLazy` conversions; `TryLazy` lacks the parallel `RcTryLazy <-> ArcTryLazy`.

13. **Relax `Clone` bound on `SendThunk::tail_rec_m`.** The step function is `Fn` (not `FnOnce`), so `Clone` is unnecessary; the function can be called by reference in the loop.

14. **Add `Evaluable` for `IdentityBrand`.** This validates the abstraction with a second implementor and enables `Free<IdentityBrand, A>` as a useful degenerate case.

15. **Consider adding derived `MonadRec` combinators.** `forever`, `while_m`, `until_m` from PureScript's ecosystem would make `MonadRec` more practically useful.

16. **Add `evaluate_owned` convenience method to `Lazy`.** Returns `A` where `A: Clone`, avoiding the `.evaluate().clone()` pattern.

17. **Consider weak references in fix combinators.** `rc_lazy_fix`/`arc_lazy_fix` create reference cycles that leak memory if the lazy value is dropped without evaluation. Weak references would eliminate this at the cost of slightly more complex implementation.

18. **Document the nondeterministic termination caveat for `VecBrand`/`CatListBrand` `MonadRec`.** If the step function always produces `Loop` values, the computation never terminates and consumes unbounded memory.

19. **Make `hoist_free` stack-safe.** Currently recurses over `Wrap` depth. Could use an explicit stack or iterate via `resume`.

20. **Add a visual diagram of the brand hierarchy** showing which brands exist, which Kind signatures they implement, and which type classes they support. The individual analysis files contain all the data needed to produce this.
