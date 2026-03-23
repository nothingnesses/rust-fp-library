# Lazy Hierarchy: Consolidated Analysis Summary

## Verdict

The lazy evaluation hierarchy is fundamentally sound and well-designed. The three-type split (Thunk, Trampoline, Lazy) reflects genuine, irreconcilable constraints in Rust's type system: HKT compatibility conflicts with type erasure; owned evaluation conflicts with shared memoization; lifetime polymorphism conflicts with `Box<dyn Any>`. No fundamental redesign is warranted. The fallible variants (TryThunk, TryTrampoline, TryLazy) provide a clean parallel hierarchy. The suggested improvements are incremental: missing type class instances, missing combinators, documentation gaps, and test coverage.

---

## Prioritized Issues

### Critical Issues

No bugs, unsoundness, or correctness problems were found. All implementations use safe Rust, all type class laws hold extensionally, and the type-level invariants are correctly maintained.

### High-Priority Improvements

| # | Issue | Agents | Suggested Fix | Affected Files |
|---|-------|--------|---------------|----------------|
| H1 | `TryTrampoline` lacks `tail_rec_m`, undermining the primary value proposition of stack-safe fallible recursion. | 7 | Delegate to `Trampoline::tail_rec_m` with `Result<Step<S, A>, E>` unwrapping. | `fp-library/src/types/try_trampoline.rs` |
| H2 | `TryTrampoline` lacks `lift2` and `then`, leaving the API incomplete relative to `Trampoline`. | 7 | Straightforward delegation to `Trampoline`'s equivalents with `Result` wrapping. | `fp-library/src/types/try_trampoline.rs` |
| H3 | `Trampoline` requires `A: Send` on all methods, but `Trampoline` itself is `!Send` (internal `Thunk` closures are not `Send`). The `Send` bound may be unnecessarily restrictive, preventing use with `Rc<T>` and other `!Send` types. | 4, 9 | Audit whether `A: Send` is actually required. If not, remove it from `Trampoline` and offer a `Send`-bounded variant separately. | `fp-library/src/types/trampoline.rs` (impl block, ~line 77) |
| H4 | `RefFunctor` not implemented for `LazyBrand<ArcLazyConfig>`. The thread-safe `ArcLazy` variant cannot be ref-mapped through the type class. | 1, 2, 5 | Add `RefFunctor for LazyBrand<ArcLazyConfig>`. May require a `SendRefFunctor` trait or adjusting `RefFunctor` bounds, since `ArcLazy::new` requires `Send` on the closure. Also add an inherent `ref_map` method on the `ArcLazy` impl block. | `fp-library/src/types/lazy.rs`, `fp-library/src/classes/ref_functor.rs` |
| H5 | Missing `fix` combinator. PureScript's `Control.Lazy` exists primarily for `fix :: (l -> l) -> l`, which ties recursive knots. This is absent from the library. | 1, 6 | Add `fix` as standalone functions for `RcLazy` and `ArcLazy` (not as a trait method, since it requires `Clone` and self-referential structure). Use an `Rc<OnceCell<RcLazy<A>>>` approach. | `fp-library/src/classes/deferrable.rs` or a new `fp-library/src/functions/fix.rs` |
| H6 | No QuickCheck property tests for `Thunk`, despite it being a full Monad. Every other major type has comprehensive law tests. | 8 | Add QuickCheck tests for Functor identity/composition, Monad left/right identity and associativity, Semigroup associativity, and Monoid identity. Agent 8 provides example test code. | `fp-library/src/types/thunk.rs` (test module) or `fp-library/tests/property.rs` |

### Medium-Priority Improvements

| # | Issue | Agents | Suggested Fix | Affected Files |
|---|-------|--------|---------------|----------------|
| M1 | `Lazy` lacks `Semigroup`/`Monoid` implementations, breaking symmetry with `Thunk`. | 1, 2, 5 | Implement with `A: Clone + Semigroup`/`Monoid` bounds: `append(a, b) = Lazy::new(\|\| Semigroup::append(a.evaluate().clone(), b.evaluate().clone()))`. | `fp-library/src/types/lazy.rs` |
| M2 | `Lazy` lacks `Foldable` implementation. | 1, 2, 5 | Implement `Foldable for LazyBrand<RcLazyConfig>` with `A: Clone`: `fold_right(f, init, lazy) = f(lazy.evaluate().clone(), init)`. | `fp-library/src/types/lazy.rs` |
| M3 | `TryThunkBrand` implements `Bifunctor` but not `Bifoldable` or `Bitraversable`. | 2 | Add `Bifoldable` for `TryThunkBrand` (straightforward single-element fold over both sides). | `fp-library/src/types/try_thunk.rs` |
| M4 | `MonadRec` missing for `TryThunkOkAppliedBrand<A>` (error-side monad). | 2 | Implement `tail_rec_m` with a loop that short-circuits on `Ok` instead of `Err`. Agent 2 provides the implementation sketch. | `fp-library/src/types/try_thunk.rs` |
| M5 | Missing `From<TryThunk> for TryTrampoline` conversion. Cannot easily upgrade a fallible thunk to stack-safe fallible computation. | 7 | Add `From<TryThunk<'static, A, E>> for TryTrampoline<A, E>` with `A + E: Send + 'static`. | `fp-library/src/types/try_trampoline.rs` |
| M6 | Missing `From<Thunk<'static, A>> for Trampoline<A>` and `From<Trampoline<A>> for Thunk<'static, A>` conversions. | 4 | Add both `From` impls to complete the conversion graph between Thunk and Trampoline. | `fp-library/src/types/trampoline.rs`, `fp-library/src/types/thunk.rs` |
| M7 | No QuickCheck property tests for `Trampoline` (Monad laws via inherent methods). | 8 | Add tests for left/right identity, associativity, and functor identity using inherent `bind`/`map`/`pure`. | `fp-library/src/types/trampoline.rs` or `fp-library/tests/property.rs` |
| M8 | No QuickCheck tests for `Lazy` covering `RefFunctor` laws or `Deferrable` transparency. | 8 | Add property tests: `ref_map(clone, lazy).evaluate() == lazy.evaluate()` and `defer(\|\| lazy).evaluate() == lazy.evaluate()`. | `fp-library/src/types/lazy.rs` or `fp-library/tests/property.rs` |
| M9 | `RefFunctor` and `Deferrable` traits have no documented laws. | 8 | Add law documentation to both traits (identity/composition for `RefFunctor`; transparency for `Deferrable`). | `fp-library/src/classes/ref_functor.rs`, `fp-library/src/classes/deferrable.rs` |
| M10 | Panic behavior of `Lazy` during initialization is undocumented. If the initializer panics, `LazyCell`/`LazyLock` poison the cell. | 5 | Document panic behavior in `Lazy`'s type-level docs. Consider a `catch_unwind` method on `Lazy` itself (not just `TryLazy`). | `fp-library/src/types/lazy.rs` |

### Low-Priority Polish

| # | Issue | Agents | Suggested Fix | Affected Files |
|---|-------|--------|---------------|----------------|
| L1 | No `Debug` implementation for any of the three types or their fallible variants. | 3, 9 | Implement `Debug` showing `Thunk(<unevaluated>)`, `Lazy(<value>)` or `Lazy(<unevaluated>)` (checking if evaluated), and `Trampoline(<unevaluated>)`. | All type files |
| L2 | `Thunk` doc comment says "each call to `evaluate` re-executes the computation," but `evaluate(self)` consumes the thunk, so it can only be called once. | 1, 3 | Reword to: "Thunk does not cache results. If you need the same computation's result more than once, wrap it in `Lazy`." | `fp-library/src/types/thunk.rs` |
| L3 | `TryLazy` lacks `ok()`/`err()` convenience constructors, breaking symmetry with `TryThunk` and `TryTrampoline`. | 7 | Add `TryLazy::ok(a)` and `TryLazy::err(e)` methods. | `fp-library/src/types/try_lazy.rs` |
| L4 | No `From<Result<A, E>>` for the `Try*` types. | 7 | Add `From<Result<A, E>>` for `TryThunk`, `TryTrampoline`, and `TryLazy` for ergonomic conversion from already-computed results. | All `try_*` type files |
| L5 | Missing `catch_unwind` on `TryThunk` (currently only on `TryLazy`). | 7 | Add a `catch_unwind` constructor to `TryThunk`. | `fp-library/src/types/try_thunk.rs` |
| L6 | The `FnOnce` vs `Fn` discrepancy between `Thunk`'s inherent `bind` and `Semimonad::bind` should be documented. | 3, 9 | Add a note explaining that HKT-level `bind`/`map` require `Fn` closures (for types like `Vec`), while inherent methods accept `FnOnce`. | `fp-library/src/types/thunk.rs` |
| L7 | `Trampoline`/`Free` documentation could explain the "Trampoline = Free over Thunk" connection more explicitly, noting why `Thunk` is used instead of `Identity` in a strict language. | 4 | Add a module-level doc note to `trampoline.rs` and/or `free.rs`. | `fp-library/src/types/trampoline.rs`, `fp-library/src/types/free.rs` |
| L8 | The "When to Use" comparison table in `thunk.rs` does not mention the `Send` requirement on `Trampoline`. | 2 | Update the comparison table to include the `Send` constraint. | `fp-library/src/types/thunk.rs` |
| L9 | Consider a `Thunk::memoize()` and `Trampoline::memoize()` convenience method for discoverability. | 10 | Add inherent methods that delegate to the existing `From` conversions. | `fp-library/src/types/thunk.rs`, `fp-library/src/types/trampoline.rs` |
| L10 | `LazyConfig` extensibility is undocumented. Users could provide custom configs (e.g., `parking_lot`-based). | 5 | Document `LazyConfig`'s extensibility in its trait docs. | `fp-library/src/types/lazy.rs` |
| L11 | Consider documenting the Cats `Eval` correspondence for users coming from Scala. | 10 | Add a doc section mapping `Eval.now` to `Thunk::pure`, `Eval.always` to `Thunk::new`, `Eval.later` to `Lazy::new`, etc. | `fp-library/src/types/thunk.rs` or module-level docs |
| L12 | `Lazy` lacks `PartialEq`/`PartialOrd`/`Display` implementations. | 1, 5 | Implement by delegating to the cached value: `lazy1 == lazy2` iff `*lazy1.evaluate() == *lazy2.evaluate()`. | `fp-library/src/types/lazy.rs` |
| L13 | Consider `TryThunk` refactoring as a newtype over `Thunk<'a, Result<A, E>>` (following `TryTrampoline`'s pattern) to reduce code duplication. | 7 | Rewrite `TryThunk` to delegate to `Thunk<'a, Result<A, E>>` with thin wrappers for `ok`, `err`, `map_err`, `catch`. | `fp-library/src/types/try_thunk.rs` |

---

## Design Validation: What Is Correct and Should Not Change

The following aspects were confirmed as correct and well-designed by multiple agents:

| Aspect | Confirmed By | Notes |
|--------|-------------|-------|
| Three-type split (Thunk/Trampoline/Lazy) is necessary and well-motivated. | 1, 4, 9, 10 | Reflects irreconcilable Rust type system constraints: HKT vs type erasure, owned vs shared evaluation, `'a` vs `'static`. |
| `Thunk` single-variant `Box<dyn FnOnce>` design (no enum). | 3, 10 | Uniform closure representation; `pure` wraps in closure anyway; avoids branching for zero benefit. |
| `Thunk` consuming `evaluate(self) -> A` semantics. | 3, 8, 9 | Correct for `FnOnce`; prevents double-evaluation; enables full Monad. |
| `Lazy` using `Rc<LazyCell>`/`Arc<LazyLock>` from std. | 5, 10 | Idiomatic Rust 1.80+; no unsafe code; delegates interior mutability to battle-tested primitives. |
| `LazyConfig` trait for Rc/Arc abstraction. | 1, 2, 5, 9, 10 | Clean strategy pattern; avoids code duplication; enables generic programming with aliases for ergonomics. |
| `Lazy::evaluate(&self) -> &A` returning a reference. | 5, 10 | Avoids unnecessary cloning; enables shared caching; correct trade-off against full Monad. |
| `Lazy` implementing `RefFunctor` instead of `Functor`. | 1, 2, 5, 8 | `Functor` would require `Clone` on `A` (not in trait signature); `RefFunctor` matches reference-returning semantics. |
| Trampoline as newtype over `Free<ThunkBrand, A>`. | 4, 10 | Ergonomic naming; curated API surface; reusable Free monad; O(1) bind via CatList. |
| Trampoline lacking HKT brand. | 1, 2, 4, 10 | `Kind` requires lifetime polymorphism; `Trampoline` is `'static`-only. Correct omission. |
| `Free` using "Reflection without Remorse" with CatList. | 4 | O(1) bind; O(n) evaluation; no left-associated bind degradation. |
| Iterative `Drop` for `Free`. | 4, 9 | Prevents stack overflow during destruction of deep `Bind` chains. Tested for 100,000+ depth. |
| `MonadRec::tail_rec_m` for `Thunk` is genuinely stack-safe. | 3, 8 | Uses explicit loop; no recursion; tested at 1,000,000 iterations. |
| `Deferrable` trait design (minimal, no supertraits, lifetime-parameterized). | 6, 8 | Clean, comprehensive implementation coverage across all six types plus `Free`. |
| Naming choice of `Deferrable` (avoids collision with `Lazy` type and `defer` method). | 6 | Reasonable compromise; `Lazy` would cause confusion. |
| All type class laws hold extensionally for Thunk and Lazy. | 8 | Functor, Monad, Applicative, RefFunctor, Semigroup, Monoid, Deferrable transparency. |
| `SendDeferrable` as independent trait from `Deferrable`. | 1, 6 | Correct; `ArcLazy` requires `Send` on closures, which `Deferrable`'s signature cannot express. |
| No unsafe code in any lazy type or `Deferrable`. | 9 | Type erasure in `Free` uses safe `downcast()`. |
| Conversion graph via `From` impls is comprehensive and correctly bounded. | 1, 2, 4 | Clone bounds on Lazy-to-Thunk; `'static` bounds on Trampoline conversions; `Send` propagation. |
| Current parallel `Try*` types approach over monad transformer approach. | 7, 10 | Transformer not practical given `'static` on Trampoline, reference-returning Lazy, and `LazyConfig` integration. |

---

## Per-Type Issue Map

### Thunk (`fp-library/src/types/thunk.rs`)

| Priority | Issue | Ref |
|----------|-------|-----|
| High | No QuickCheck property tests for type class laws. | H6 |
| Medium | Missing `From<Thunk<'static, A>> for Trampoline<A>` conversion. | M6 |
| Low | Doc comment about re-execution is misleading for consuming evaluate. | L2 |
| Low | `Fn` vs `FnOnce` discrepancy undocumented. | L6 |
| Low | Comparison table does not mention Trampoline's `Send` requirement. | L8 |
| Low | Consider `memoize()` convenience method. | L9 |

### Trampoline (`fp-library/src/types/trampoline.rs`)

| Priority | Issue | Ref |
|----------|-------|-----|
| High | `A: Send` bound may be unnecessarily restrictive (Trampoline is `!Send` anyway). | H3 |
| Medium | Missing `From<Trampoline<A>> for Thunk<'static, A>` conversion. | M6 |
| Medium | No QuickCheck property tests for Monad laws. | M7 |
| Low | Module docs could explain Free-over-Thunk connection more explicitly. | L7 |
| Low | No `Debug` implementation. | L1 |
| Low | Consider `memoize()` convenience method. | L9 |

### Lazy (`fp-library/src/types/lazy.rs`)

| Priority | Issue | Ref |
|----------|-------|-----|
| High | `RefFunctor` missing for `ArcLazyConfig`. | H4 |
| High | `fix` combinator missing. | H5 |
| Medium | Missing `Semigroup`/`Monoid`. | M1 |
| Medium | Missing `Foldable`. | M2 |
| Medium | No QuickCheck tests for `RefFunctor` laws. | M8 |
| Medium | Panic behavior during initialization undocumented. | M10 |
| Low | No `Debug`/`Display` implementation. | L1 |
| Low | Missing `PartialEq`/`PartialOrd`. | L12 |
| Low | `LazyConfig` extensibility undocumented. | L10 |

### TryThunk (`fp-library/src/types/try_thunk.rs`)

| Priority | Issue | Ref |
|----------|-------|-----|
| Medium | `Bifoldable`/`Bitraversable` missing for `TryThunkBrand`. | M3 |
| Medium | `MonadRec` missing for `TryThunkOkAppliedBrand<A>`. | M4 |
| Low | Missing `catch_unwind` constructor. | L5 |
| Low | Consider refactoring as newtype over `Thunk<'a, Result<A, E>>`. | L13 |
| Low | Missing `From<Result<A, E>>`. | L4 |

### TryTrampoline (`fp-library/src/types/try_trampoline.rs`)

| Priority | Issue | Ref |
|----------|-------|-----|
| High | Missing `tail_rec_m` (stack-safe fallible recursion). | H1 |
| High | Missing `lift2` and `then`. | H2 |
| Medium | Missing `From<TryThunk> for TryTrampoline` conversion. | M5 |
| Low | Missing `From<Result<A, E>>`. | L4 |

### TryLazy (`fp-library/src/types/try_lazy.rs`)

| Priority | Issue | Ref |
|----------|-------|-----|
| Low | Missing `ok()`/`err()` convenience constructors. | L3 |
| Low | Missing `From<Result<A, E>>`. | L4 |

### Deferrable Trait (`fp-library/src/classes/deferrable.rs`)

| Priority | Issue | Ref |
|----------|-------|-----|
| High | Missing `fix` combinator (PureScript's primary use of `Control.Lazy`). | H5 |
| Medium | No documented laws. | M9 |

### RefFunctor Trait (`fp-library/src/classes/ref_functor.rs`)

| Priority | Issue | Ref |
|----------|-------|-----|
| Medium | No documented laws. | M9 |

---

## Recommended Implementation Order

**Phase 1: High-impact, low-risk additions**

1. Add `tail_rec_m`, `lift2`, and `then` to `TryTrampoline` (H1, H2). These are straightforward delegations.
2. Add `RefFunctor` for `ArcLazyConfig` and inherent `ref_map` on `ArcLazy` (H4).
3. Add QuickCheck property tests for `Thunk` type class laws (H6).

**Phase 2: Trampoline Send audit and conversion completions**

4. Audit and potentially remove `A: Send` from `Trampoline` (H3). This may have cascading effects; verify all downstream types.
5. Add missing `From` conversions: Thunk/Trampoline bidirectional (M6), TryThunk to TryTrampoline (M5).

**Phase 3: Lazy type class enrichment**

6. Add `Semigroup`/`Monoid` for `Lazy` (M1).
7. Add `Foldable` for `LazyBrand<RcLazyConfig>` (M2).
8. Add `PartialEq`/`PartialOrd` for `Lazy` (L12).

**Phase 4: TryThunk completions**

9. Add `Bifoldable` for `TryThunkBrand` (M3).
10. Add `MonadRec` for `TryThunkOkAppliedBrand` (M4).

**Phase 5: fix combinator and test coverage**

11. Implement `fix` for `RcLazy` and `ArcLazy` (H5).
12. Add QuickCheck tests for `Trampoline` and `Lazy` (M7, M8).

**Phase 6: Documentation and polish**

13. Document laws for `RefFunctor`, `Deferrable`, `SendDeferrable` (M9).
14. Document panic behavior in `Lazy` (M10).
15. Fix misleading `Thunk` doc comment (L2).
16. Add `Debug` implementations (L1).
17. Add convenience constructors and conversions (L3, L4, L5, L9).
18. Documentation improvements (L6, L7, L8, L10, L11).
19. Consider `TryThunk` refactoring (L13), as a larger effort.
