# Lazy Hierarchy: Implementation Plan

This plan translates the findings from `summary.md` into concrete, actionable work items organized by phase. Each item is self-contained: someone should be able to pick it up and implement it without re-reading the research files.

---

## Phase 1: Critical Documentation Fixes

**Goal:** Address documentation issues that could cause users to write incorrect or unsafe code. No code behavior changes; only doc comments and prose.

**Dependencies:** None. All items in this phase are independent and can be done in any order or in parallel.

### 1.1 Document `fold_free` stack safety limitation

- **File:** `fp-library/src/types/free.rs` (the `fold_free` function's doc comment)
- **What:** Add a `# Stack Safety` section warning that `fold_free` uses actual recursion (unlike `Free::evaluate`, which is iterative). For strict target monads (e.g., `OptionBrand`), each `Wrap` layer adds one stack frame. Deep `Free` computations will overflow.
- **Scope:** ~5 lines of doc comment.

### 1.2 Add stack safety warning to `TryThunk` struct docs

- **File:** `fp-library/src/types/try_thunk.rs` (the `TryThunk` struct doc comment)
- **What:** Add a warning matching the one on `Thunk` that `bind` chains are not stack-safe. Nesting `bind` calls builds up nested closures that consume stack on evaluation. Recommend `TryTrampoline` for deep recursion.
- **Scope:** ~5-8 lines of doc comment.

### 1.3 Add `# Panics` section to `TryLazy::evaluate()`

- **File:** `fp-library/src/types/try_lazy.rs` (the `evaluate` method doc comment)
- **What:** Document panic poisoning behavior. When the initializer panics, `RcTryLazy` will panic on subsequent `evaluate()` calls (matching `LazyCell` behavior). `ArcTryLazy` will panic on all threads (matching `LazyLock` behavior). Mirror the documentation already present on `Lazy::evaluate()`.
- **Scope:** ~5-8 lines of doc comment.

### 1.4 Add recursion limit warnings to `rc_lazy_fix` and `arc_lazy_fix`

- **File:** `fp-library/src/types/lazy.rs` (doc comments for both fix functions)
- **What:** Add a `# Panics` or `# Stack Safety` section warning that if `f` forces (evaluates) the self-reference during construction, infinite recursion and stack overflow will occur. The self-reference should only be forced after the `Lazy` value is fully constructed.
- **Scope:** ~4 lines per function (8 total).

### 1.5 Add `tail_rec_m` shallow-thunk warning to `Thunk`

- **File:** `fp-library/src/types/thunk.rs` (the `tail_rec_m` method doc comment)
- **What:** Add a note that the step function `f` should return shallow thunks (ideally `Thunk::pure` or a single-level `Thunk::new`). If `f` builds deep `bind` chains inside the returned thunk, the `evaluate()` call inside the trampoline loop can still overflow the stack.
- **Scope:** ~4 lines of doc comment.

---

## Phase 2: Stale References and Naming Fixes

**Goal:** Fix all incorrect references, stale names, and terminology issues. These are small, mechanical changes.

**Dependencies:** None. All items are independent.

### 2.1 Fix stale references in `Free` module docs

- **File:** `fp-library/src/types/free.rs`
- **What:** Three changes:
  1. Replace all references to `Runnable` trait with `Evaluable`.
  2. Replace all references to `Free::roll` with `Free::wrap`.
  3. Change `// SAFETY:` comments on non-`unsafe` downcast code to `// INVARIANT:` to follow Rust conventions (reserve "SAFETY" for `unsafe` blocks).
- **Scope:** Search-and-replace across the file. Likely 5-10 occurrences total.

### 2.2 Rename "eval" remnants in `Thunk` tests

- **File:** `fp-library/src/types/thunk.rs` (test module) or `fp-library/tests/` (wherever Thunk tests live)
- **What:** Rename test functions and doc strings that reference "eval" (a former name for `Thunk`). Known instances: `test_eval_from_memo`, `test_eval_semigroup`, and any doc strings containing "eval" when they mean "thunk."
- **Scope:** ~3-5 renames.

### 2.3 Rename stale `Trampoline` test names

- **File:** `fp-library/src/types/trampoline.rs` (test module) or relevant test file
- **What:** Rename `test_task_map2` to `test_task_lift2` and `test_task_and_then` to `test_task_then` to match the current method names.
- **Scope:** 2 renames.

### 2.4 Resolve `TryThunk` `pure` deprecation confusion

- **File:** `fp-library/src/types/try_thunk.rs`
- **What:** Investigate whether `TryThunk::pure` is intended to be deprecated. If yes, add `#[deprecated]` attribute to the method. If no, remove `#[allow(deprecated)]` from the test that calls it.
- **Scope:** 1-2 line change.

### 2.5 Fix `Lazy` lifetime parameter description inconsistency

- **File:** `fp-library/src/types/lazy.rs` (struct doc comment)
- **What:** Change the lifetime parameter description from "The lifetime of the reference" to "The lifetime of the computation" to match the wording used in `Deferrable` and the fix functions.
- **Scope:** 1 line change.

### 2.6 Fix `Trampoline` doc link for `bind`

- **File:** `fp-library/src/types/trampoline.rs`
- **What:** The doc link for `bind` points to `crate::functions::bind` (the free function) instead of the inherent method. Update the link target.
- **Scope:** 1 line change.

---

## Phase 3: Documentation Completeness

**Goal:** Fill in documentation gaps, add missing explanations, and bring all types to the same documentation standard as `Thunk`, `Lazy`, and `Deferrable`.

**Dependencies:** Phase 2 should be complete first so that stale references are not propagated into new documentation.

### 3.1 Add "Traversable limitation" note to `TryThunk`

- **File:** `fp-library/src/types/try_thunk.rs`
- **What:** Add a section explaining why `TryThunk` cannot implement `Traversable`, mirroring the note on `Thunk`. The reason: `FnOnce` closures cannot be cloned, so there is no way to "sequence" effects through a thunk without consuming it.
- **Scope:** ~5 lines of doc comment.

### 3.2 Add algebraic properties section to `TryThunk`

- **File:** `fp-library/src/types/try_thunk.rs`
- **What:** Add a section documenting monad laws for the success channel, matching `Thunk`'s algebraic properties section. State that `TryThunk` forms a monad over the success type `A` (with `pure` and `bind` short-circuiting on `Err`).
- **Scope:** ~10 lines of doc comment.

### 3.3 Add transparency law and examples to `SendDeferrable`

- **File:** `fp-library/src/classes/send_deferrable.rs` (or wherever the trait is defined)
- **What:**
  1. Add a `# Laws` section mirroring `Deferrable`'s transparency law: `send_defer(|| x) == x`.
  2. Add `#[document_examples]` attribute with a law-demonstrating example.
  3. Add a note about `arc_lazy_fix` (mirroring `Deferrable`'s discussion of `fix` and `rc_lazy_fix`).
- **Scope:** ~15-20 lines.

### 3.4 Add "why no `Functor`" explanation to `Lazy` module docs

- **File:** `fp-library/src/types/lazy.rs` (module-level doc comment)
- **What:** Add a section explaining that `Lazy` cannot implement the standard `Functor` trait because `evaluate()` returns `&A`, not `A`. Point users to `RefFunctor` / `SendRefFunctor` as the correct alternative, and explain that `ref_map` takes `&A -> B` instead of `A -> B`.
- **Scope:** ~8 lines of doc comment.

### 3.5 Add Clone requirements explanation to `TryLazy` docs

- **File:** `fp-library/src/types/try_lazy.rs`
- **What:** Explain why `map` requires `E: Clone` and `map_err` requires `A: Clone`. The inner cell holds `Result<A, E>`, and mapping one side requires cloning the other side out of the reference.
- **Scope:** ~4 lines per method.

### 3.6 Document cache chain behavior in `TryLazy`

- **File:** `fp-library/src/types/try_lazy.rs`
- **What:** Add a note explaining that chaining `map` calls on `TryLazy` creates a linked list of `Rc`/`Arc`-referenced cells. Each mapped `TryLazy` holds a reference to its predecessor. This keeps predecessor values alive in memory.
- **Scope:** ~5 lines of doc comment.

### 3.7 Add "when to use" guidance to `TryThunk`, `TryTrampoline`, and `TryLazy`

- **Files:** `fp-library/src/types/try_thunk.rs`, `fp-library/src/types/try_trampoline.rs`, `fp-library/src/types/try_lazy.rs`
- **What:** Add a `# When to Use` section to each type's struct doc comment. Brief guidance:
  - `TryThunk`: Lightweight fallible deferred computation with full HKT support. Not stack-safe for deep `bind` chains.
  - `TryTrampoline`: Stack-safe fallible recursion. No HKT brands, `'static` only.
  - `TryLazy`: Memoized fallible computation. Caches the `Result` on first evaluation.
- **Scope:** ~5 lines per type (15 total).

### 3.8 Add `SendRefFunctor` trait motivation to trait docs

- **File:** `fp-library/src/classes/send_ref_functor.rs` (or wherever the trait is defined)
- **What:** The trait doc should explain why a separate trait exists (rather than just adding `Send` bounds to `RefFunctor`). The reason: `RefFunctor` implementations for `RcLazy` use `Rc` which is `!Send`; a separate trait lets `ArcLazy` express thread-safe reference-based mapping. Currently this explanation only exists in `ArcLazy::ref_map`'s inline comment.
- **Scope:** ~5 lines of doc comment.

### 3.9 Remove unnecessary `brands::*` imports from `Deferrable` doc examples

- **File:** `fp-library/src/classes/deferrable.rs`
- **What:** Three doc examples import `brands::*` but use no brand types. Remove these unnecessary imports.
- **Scope:** 3 line deletions.

### 3.10 Fix minor `Deferrable` free function doc issue

- **File:** `fp-library/src/classes/deferrable.rs`
- **What:** Add a trailing period to the `defer` free function's type parameter description ("The lifetime of the computation" should be "The lifetime of the computation.").
- **Scope:** 1 character.

### 3.11 Explain the deref in `Trampoline`'s `memoize` doc example

- **File:** `fp-library/src/types/trampoline.rs`
- **What:** The `memoize` doc example uses `*lazy.evaluate()` without explaining the deref. Add a brief inline comment: `// evaluate() returns &i32, deref to get i32`.
- **Scope:** 1 comment line.

### 3.12 Document eager evaluation in `From<Lazy>` for `TryTrampoline`

- **File:** `fp-library/src/types/try_trampoline.rs` (or wherever the `From<Lazy>` impl lives)
- **What:** Add a note explaining that converting from `Lazy` to `TryTrampoline` forces evaluation at conversion time (the `Lazy` is evaluated, then the result is wrapped in `Trampoline::pure`). This matches the note already present for `From<TryLazy>` but is missing for `From<Lazy>`.
- **Scope:** ~3 lines of doc comment.

---

## Phase 4: Missing Trait Implementations for `TryLazy`

**Goal:** Close the most significant functional gap in the hierarchy: `TryLazy` has a `Kind` instance but no trait implementations.

**Dependencies:** None from earlier phases, but completing Phase 1.3 (panic docs) first is recommended so the new trait impls can reference consistent documentation.

### 4.1 Implement `RefFunctor` for `TryLazyBrand<E, RcLazyConfig>`

- **File:** `fp-library/src/types/try_lazy.rs`
- **What:** Implement `RefFunctor` for `TryLazyBrand<E, RcLazyConfig>`. The `ref_map` function should take `f: impl Fn(&Result<A, E>) -> B` (or, if the trait is over the success channel only, `f: impl Fn(&A) -> B` with `E: Clone`). Follow the same pattern used for `LazyBrand<RcLazyConfig>` in `lazy.rs`.
- **Scope:** ~15-25 lines of implementation.

### 4.2 Implement `SendRefFunctor` for `TryLazyBrand<E, ArcLazyConfig>`

- **File:** `fp-library/src/types/try_lazy.rs`
- **What:** Mirror 4.1 but for `ArcLazyConfig`, adding `Send + Sync` bounds as needed. Follow the pattern from `LazyBrand<ArcLazyConfig>`.
- **Scope:** ~15-25 lines of implementation.
- **Dependency:** Should mirror the approach taken in 4.1.

### 4.3 Implement `Semigroup` and `Monoid` for `TryLazy` (both configs)

- **File:** `fp-library/src/types/try_lazy.rs`
- **What:** Implement `Semigroup` for `TryLazy` where `A: Semigroup + Clone` and `E: Clone`. Semantics: if both are `Ok`, combine with `Semigroup::append`; if either is `Err`, propagate the first `Err`. `Monoid::empty` returns `Ok(A::empty())`. Implement for both `RcLazyConfig` and `ArcLazyConfig`.
- **Scope:** ~40-60 lines (two configs, two traits each).

### 4.4 Implement `Foldable` for `TryLazy` (both configs)

- **File:** `fp-library/src/types/try_lazy.rs`
- **What:** Implement `Foldable` for `TryLazyBrand<E, RcLazyConfig>` and `TryLazyBrand<E, ArcLazyConfig>`. Follow the pattern from `Lazy`'s `Foldable` implementation. The fold should evaluate the `TryLazy`, and if `Ok(a)`, fold over `a`; if `Err`, return the accumulator unchanged.
- **Scope:** ~30-40 lines.

---

## Phase 5: Missing Trait Implementations for Other Types

**Goal:** Fill remaining trait gaps across `TryTrampoline` and `Thunk`.

**Dependencies:** None from earlier phases.

### 5.1 Implement `Semigroup` and `Monoid` for `TryTrampoline`

- **File:** `fp-library/src/types/try_trampoline.rs`
- **What:** Implement `Semigroup` for `TryTrampoline<A, E>` where `A: Semigroup + 'static` and `E: 'static`. Semantics: evaluate both, combine if both `Ok`, propagate first `Err`. `Monoid::empty` returns a pure `Ok(A::empty())`. Follow the pattern from `Trampoline`'s `Semigroup`/`Monoid`.
- **Scope:** ~20-30 lines.

### 5.2 Add `bimap` method to `TryTrampoline`

- **File:** `fp-library/src/types/try_trampoline.rs`
- **What:** Add `pub fn bimap<B, F>(self, f: impl FnOnce(A) -> B, g: impl FnOnce(E) -> F) -> TryTrampoline<B, F>` that maps both sides of the inner `Result`. Currently only `map` and `map_err` exist separately.
- **Scope:** ~10 lines.

### 5.3 Implement `FunctorWithIndex` and `FoldableWithIndex` for `ThunkBrand`

- **File:** `fp-library/src/types/thunk.rs`
- **What:** Implement `FunctorWithIndex` with index type `()` and `FoldableWithIndex` with index type `()`. These are trivial: `map_with_index(f, thunk)` = `map(|a| f((), a), thunk)` and `fold_map_with_index(f, thunk)` = `fold_map(|a| f((), a), thunk)`.
- **Scope:** ~20 lines.

---

## Phase 6: Missing Conversions, Brands, and Standard Trait Impls

**Goal:** Ergonomic improvements, additional conversions, and brand aliases.

**Dependencies:** None from earlier phases.

### 6.1 Add brand type aliases

- **File:** `fp-library/src/brands.rs`
- **What:** Add the following type aliases to match the `RcFnBrand`/`ArcFnBrand` pattern:
  ```rust
  pub type RcLazyBrand = LazyBrand<RcLazyConfig>;
  pub type ArcLazyBrand = LazyBrand<ArcLazyConfig>;
  pub type RcTryLazyBrand<E> = TryLazyBrand<E, RcLazyConfig>;
  pub type ArcTryLazyBrand<E> = TryLazyBrand<E, ArcLazyConfig>;
  ```
- **Scope:** 4 lines + doc comments.

### 6.2 Add `Eq` and `Ord` for `Lazy`

- **File:** `fp-library/src/types/lazy.rs`
- **What:** Implement `Eq` for `Lazy<'a, A, Config>` where `A: Eq` and `Ord` for `Lazy<'a, A, Config>` where `A: Ord`. Both delegate to `self.evaluate()` comparison. Implement for both `RcLazyConfig` and `ArcLazyConfig`.
- **Scope:** ~15-20 lines.

### 6.3 Add `From<Thunk>` and `From<Trampoline>` conversions for `ArcLazy`

- **File:** `fp-library/src/types/lazy.rs`
- **What:** Implement `From<Thunk<'a, A>>` and `From<Trampoline<A>>` for `ArcLazy<'a, A>` where `A: Send + Sync + 'a`. For `Thunk`, the conversion must evaluate eagerly (since `Thunk` is `!Send`, it cannot be deferred into `ArcLazy`). For `Trampoline`, same constraint.
- **Scope:** ~15 lines.

### 6.4 Add `From<TryThunk>` and `From<TryTrampoline>` conversions for `ArcTryLazy`

- **File:** `fp-library/src/types/try_lazy.rs`
- **What:** Same pattern as 6.3 but for the fallible variants. Conversions must evaluate eagerly due to `!Send` on source types.
- **Scope:** ~15 lines.

---

## Phase 7: Test Coverage

**Goal:** Fill testing gaps identified in the research.

**Dependencies:** Phases 4 and 5 should be complete first, so that new trait implementations can be tested.

### 7.1 Add HKT-level trait tests for `Thunk`

- **File:** Appropriate test file (e.g., `fp-library/tests/thunk_hkt.rs` or inline test module)
- **What:** Test `Foldable`, `Lift::lift2`, `Semiapplicative::apply`, and `Evaluable::evaluate` through the brand/free-function interface (not inherent methods). Verify that `fold::<ThunkBrand, _, _, _>(...)`, `lift2::<ThunkBrand, _, _, _, _>(...)`, etc. work correctly.
- **Scope:** ~40-60 lines of tests.

### 7.2 Add `memoize` and `memoize_arc` tests for `Thunk`

- **File:** Thunk test module
- **What:** Test that `Thunk::memoize()` returns an `RcLazy` that caches the result, and `Thunk::memoize_arc()` returns an `ArcLazy`. Verify memoization (second evaluation does not re-run the closure) by using a counter.
- **Scope:** ~20-30 lines.

### 7.3 Add QuickCheck tests for `TryThunk` bifunctor laws

- **File:** Property test file (likely `fp-library/tests/property.rs` or similar)
- **What:** Add QuickCheck properties for bifunctor identity law (`bimap(id, id) == id`) and bifunctor composition law for `TryThunkBrand`.
- **Scope:** ~20-30 lines.

### 7.4 Add QuickCheck tests for `TryThunk` error-channel monad laws

- **File:** Property test file
- **What:** Test monad left identity, right identity, and associativity for `TryThunkOkAppliedBrand<A>` (the error channel as the "functor variable").
- **Scope:** ~20-30 lines.

### 7.5 Add `Semigroup`/`Monoid` law tests for `TryThunk`

- **File:** Property test file
- **What:** QuickCheck for semigroup associativity and monoid identity on `TryThunk`.
- **Scope:** ~15-20 lines.

### 7.6 Add `memoize_arc` thread safety test for `TryThunk`

- **File:** Thunk/TryThunk test module
- **What:** Test that `TryThunk::memoize_arc()` produces a `Send + Sync` value that can be shared across threads. Spawn threads that evaluate the resulting `ArcTryLazy` and verify they all see the same cached result.
- **Scope:** ~20 lines.

### 7.7 Deeper stack safety stress tests for `Trampoline`

- **File:** Trampoline test module
- **What:** Add a stress test with 100,000+ iterations for `Trampoline::tail_rec_m` to strengthen stack safety claims. The current tests use only 1,000 depth.
- **Scope:** ~10 lines.

### 7.8 Add tests for new `TryLazy` trait implementations

- **File:** Appropriate test module
- **What:** Test `RefFunctor`/`SendRefFunctor` for `TryLazy` (from Phase 4.1/4.2), `Semigroup`/`Monoid` (4.3), and `Foldable` (4.4). Include both `Ok` and `Err` paths.
- **Scope:** ~40-60 lines.
- **Dependency:** Phase 4 must be complete.

### 7.9 Add tests for `TryTrampoline` `Semigroup`/`Monoid` and `bimap`

- **File:** Appropriate test module
- **What:** Test the implementations from Phase 5.1 and 5.2.
- **Scope:** ~20-30 lines.
- **Dependency:** Phase 5 must be complete.

---

## Phase 8: Structural Improvements (Consider Carefully)

**Goal:** Larger design changes that improve consistency or reduce duplication. These require more thought and may have broader implications.

### 8.1 Reduce Rc/Arc code duplication with `macro_rules!`

- **Files:** `fp-library/src/types/lazy.rs`, `fp-library/src/types/try_lazy.rs`
- **What:** Introduce a `macro_rules!` helper that generates trait implementations for both `RcLazyConfig` and `ArcLazyConfig` from a single definition. The macro takes the trait being implemented, the bounds that differ (`Send + Sync` for Arc), and the implementation body. This could eliminate ~250+ lines of duplication.
- **Scope:** Medium. Requires careful macro design. The macro should be local to the lazy modules (not exported).
- **Risk:** Macros can reduce readability. Ensure the macro is well-commented and the expansion is easy to understand.

### 8.2 Consider `SendDeferrable: Deferrable` supertrait

- **Files:** `fp-library/src/classes/send_deferrable.rs`, `fp-library/src/classes/deferrable.rs`, `fp-library/src/types/lazy.rs`
- **What:** Make `SendDeferrable` extend `Deferrable` (following the `SendCloneableFn: CloneableFn` precedent). Then implement `Deferrable` for `ArcLazy` as well. This would allow generic code written against `Deferrable` to accept both `RcLazy` and `ArcLazy`.
- **Scope:** Medium. Requires adding a `Deferrable` impl for `ArcLazy` and updating `SendDeferrable` trait definition.
- **Trade-offs:** Improves consistency with other `Send*` traits in the library. May require bounds changes on existing generic code. Should verify no downstream breakage.

### 8.3 Consider a `SendThunk` variant

- **Files:** New type, likely `fp-library/src/types/send_thunk.rs`, plus brand in `brands.rs`
- **What:** Introduce `SendThunk<'a, A>` wrapping `Box<dyn FnOnce() -> A + Send + 'a>`. This enables truly lazy `memoize_arc()` (currently `Thunk::memoize_arc()` must evaluate eagerly because `Thunk` is `!Send`). Would also enable thread-safe deferred computation chains without memoization.
- **Scope:** Large. New type, brand, trait implementations (`Functor`, `Monad`, `Deferrable`/`SendDeferrable`, etc.), tests, documentation.
- **Trade-offs:** Adds another type to an already-complex hierarchy. Only worth it if there is real demand for lazy-but-not-memoized thread-safe computation.

### 8.4 Consider splitting `LazyConfig` into infallible and fallible traits

- **Files:** `fp-library/src/types/lazy.rs`, `fp-library/src/types/try_lazy.rs`
- **What:** Split `LazyConfig` into `LazyConfig` (infallible associated types only) and `TryLazyConfig` (fallible associated types). Currently any custom `LazyConfig` implementor must define both, even if only one is needed.
- **Scope:** Medium-large. Affects trait definitions, all implementations, and any code that references `LazyConfig` associated types.
- **Trade-offs:** Better separation of concerns, but adds a trait. The only two existing configs (`RcLazyConfig`, `ArcLazyConfig`) implement both anyway, so the practical benefit is limited to hypothetical third-party configs.

---

## Items That Should NOT Be Done

### Do not attempt to give `Trampoline`/`Free` HKT brands

The research confirms this is inherent to Rust's type system: `Box<dyn Any>` requires `'static`, while `Kind::Of<'a, A: 'a>` requires lifetime polymorphism. These are fundamentally incompatible. Any attempt to work around this would require `unsafe` code or a completely different type erasure mechanism, which is not justified.

### Do not unify `Deferrable`/`RefFunctor` with their `Send` counterparts into single traits

Rust does not support conditional `Send` bounds on associated types or trait methods. The split is inherent to the language. The supertrait approach (Phase 8.2) is the closest viable option, but full unification is not possible.

### Do not add implicit cloning to make `Lazy` implement standard `Functor`

`Lazy::evaluate()` returns `&A`, and the standard `Functor` trait expects owned `A`. Automatically cloning would violate the library's zero-cost abstraction principle. `RefFunctor` is the correct solution, and it is already implemented.

### Do not add a `LazyConfig` extensibility test

The summary notes this is untested, but adding a test-only third-party config would be artificial. The trait is correctly defined, and if a real use case emerges, it will be validated then. Documenting the extension point clearly (already done) is sufficient.

### Do not change `TryThunk`'s `Semigroup::append` to short-circuit

The current behavior (evaluating both sides) is correct: semigroup append needs both values to combine them. Short-circuiting would violate semigroup laws. The behavior only surprises users who confuse semigroup append with monadic bind; this is better addressed with documentation if needed.

---

## Summary of Phase Dependencies

```
Phase 1 (Critical Docs)        --- no dependencies
Phase 2 (Stale References)     --- no dependencies
Phase 3 (Doc Completeness)     --- after Phase 2 (to avoid propagating stale refs)
Phase 4 (TryLazy Traits)       --- no dependencies (recommend after Phase 1.3)
Phase 5 (Other Traits)         --- no dependencies
Phase 6 (Conversions/Brands)   --- no dependencies
Phase 7 (Tests)                --- after Phases 4 and 5 (for new trait tests)
Phase 8 (Structural)           --- after all earlier phases are stable
```

Phases 1, 2, 4, 5, and 6 can all proceed in parallel. Phase 3 should follow Phase 2. Phase 7 should follow Phases 4 and 5. Phase 8 should be done last, as it involves broader design decisions.
