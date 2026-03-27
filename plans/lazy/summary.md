# Lazy Evaluation Hierarchy: Consolidated Analysis Summary

## 1. Overview

The lazy evaluation hierarchy comprises 18 analyzed components spanning four categories:

- **Infallible computation types:** `Thunk`, `SendThunk`, `Trampoline`, `RcLazy`, `ArcLazy`, and the `Free` monad (backed by `CatList`).
- **Fallible computation types:** `TryThunk`, `TrySendThunk`, `TryTrampoline`, `RcTryLazy`, `ArcTryLazy`.
- **Type classes and traits:** `Deferrable`, `SendDeferrable`, `Evaluable`, `RefFunctor`, `SendRefFunctor`, `MonadRec`.
- **Supporting infrastructure:** `Step` (recursion control type), `CatList` (catenable list for `Free`), and brand definitions in `brands.rs`.

Each type makes different trade-offs across five axes: stack safety, memoization, lifetime polymorphism, thread safety (`Send`), and HKT compatibility. The hierarchy is well-structured, with clear upgrade paths between types (e.g., `Thunk` to `Trampoline` for stack safety, `Thunk` to `RcLazy` for memoization, `Thunk` to `SendThunk` for thread safety).

## 2. Cross-Cutting Themes

### 2.1 Eager Evaluation in `Deferrable` for `Send` Types

The most pervasive design tension across the hierarchy. `SendThunk`, `TrySendThunk`, `ArcLazy`, and `ArcTryLazy` all implement `Deferrable::defer` by calling `f()` eagerly, because the trait signature does not require `Send` on the closure. True deferral is only available through `SendDeferrable::send_defer`. This is documented on the `Deferrable` trait and on individual impls, but remains a potential surprise for users writing generic code against `Deferrable`.

*Affected files: deferrable.md, send_deferrable.md, send_thunk.md, try_send_thunk.md, lazy.md, try_lazy.md.*

### 2.2 `Send` Bounds Prevent HKT Trait Implementations

`SendThunk` and `TrySendThunk` cannot implement HKT traits (`Functor`, `Semimonad`, `MonadRec`, etc.) because the trait method signatures do not carry `Send` bounds on closure parameters. These types provide equivalent inherent methods instead. This is a fundamental limitation of the library's HKT encoding.

*Affected files: send_thunk.md, try_send_thunk.md, brands.md.*

### 2.3 Reference-Returning `evaluate` Prevents Standard Functor

`Lazy` and `TryLazy` return `&A` from `evaluate()` due to memoization behind shared pointers. This prevents implementing `Functor` (which requires owned `A`), leading to the separate `RefFunctor`/`SendRefFunctor` traits. The consequence is that `Lazy`/`TryLazy` cannot participate in the standard `Functor` -> `Applicative` -> `Monad` tower.

*Affected files: lazy.md, try_lazy.md, ref_functor.md, send_ref_functor.md.*

### 2.4 `'static` Requirement Blocks HKT for `Free`/`Trampoline`

`Free` uses `Box<dyn Any>` for type erasure, requiring `A: 'static`. This conflicts with the `Kind` trait's lifetime polymorphism (`Of<'a, A>`), so `Trampoline`, `TryTrampoline`, and `Free` itself have no HKT brands. They provide inherent methods (`pure`, `bind`, `map`) instead.

*Affected files: free.md, trampoline.md, try_trampoline.md, brands.md.*

### 2.5 `Clone` Requirements in Memoized Types

Operations on `RcLazy`, `ArcLazy`, `RcTryLazy`, and `ArcTryLazy` frequently require `Clone` on value types because `evaluate()` returns references and new cells need owned values. This excludes non-cloneable types from many combinators (`Deferrable`, `Semigroup`, `Monoid`, `Foldable`, `map`/`map_err` on `TryLazy`).

*Affected files: lazy.md, try_lazy.md, deferrable.md.*

### 2.6 Missing Property-Based Law Tests

Multiple traits and types lack QuickCheck property tests for their stated laws. The existing doc-test examples demonstrate correctness but do not systematically verify algebraic properties.

*Affected files: deferrable.md, evaluable.md, monad_rec.md, ref_functor.md, send_ref_functor.md.*

### 2.7 Consistent Documentation Quality

All types and traits use the project's documentation macros (`#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]`) consistently. Doc examples compile and contain assertions. This is a strength across the board.

## 3. Issues by Severity

### High Severity

1. **`Free::Drop` stack overflow for deeply nested `Wrap` chains** (free.md). The `Drop` impl handles `Bind` chains iteratively but delegates `Wrap` variants to the functor's `Drop`, which recurses. Deeply nested `Wrap`-only chains (without `Bind`) can overflow the stack on drop.

2. **`CatList` recursive `Drop` for deeply nested structures** (cat_list.md). No custom `Drop` implementation; deeply nested `CatList` trees can overflow the stack when dropped. Less likely in practice due to `flatten_deque` restructuring during iteration, but a latent risk for standalone usage.

3. **`Evaluable` naturality law is incorrectly stated** (evaluable.md). The stated law constrains natural transformations, not `evaluate` itself. It should be replaced with the standard comonad extract law: `evaluate(map(f, fa)) == f(evaluate(fa))`.

### Medium Severity

4. **`TrySendThunk` missing `tail_rec_m` / `arc_tail_rec_m`** (try_send_thunk.md). Every other thunk/trampoline variant has stack-safe recursion support; `TrySendThunk` does not. This leaves no stack-safe recursion path for fallible + thread-safe deferred computations without falling back to `TryTrampoline` (which requires `'static` and eager conversion).

5. **`MonadRec` missing implementations for standard types** (monad_rec.md). `OptionBrand`, `VecBrand`, `ResultErrAppliedBrand`, and `IdentityBrand` do not implement `MonadRec`, even though their `bind` is inherently stack-safe. This limits the utility of generic `MonadRec`-polymorphic code.

6. **`MonadRec` law documentation is weak** (monad_rec.md). The stated "Equivalence" law is a tautology (correctness requirement), and "Safety varies" is an implementation note, not a law. The actual PureScript law (`tail_rec_m(|a| pure(Done(a)), x) == pure(x)`) is not stated.

7. **`Free::fold_free` is not stack-safe** (free.md). Uses actual recursion; deeply nested computations with strict target monads will overflow. Documented, but a stack-safe variant (`fold_free` via `resume` in a loop) would be valuable.

8. **`TryThunkOkAppliedBrand` applicative/monad inconsistency** (try_thunk.md). `apply` uses fail-last semantics while `bind` uses fail-fast. This violates the standard applicative/monad consistency law. Documented and intentional (mirrors Haskell's `Validation` vs `Either`), but may surprise users.

9. **Missing `From<SendThunk> for Thunk` conversion** (send_thunk.md, thunk.md). A `SendThunk` closure is a subtype of a `Thunk` closure (`Box<dyn FnOnce() -> A + Send>` is a subtype of `Box<dyn FnOnce() -> A>`). This zero-cost conversion is missing.

10. **`SendRefFunctor` `B` bound lacks `Sync`** (send_ref_functor.md). The trait requires `B: Send` but not `Sync`. The resulting `ArcLazy<B>` will be `!Send` if `B` is `Send` but not `Sync`, which silently defeats the purpose of using `ArcLazy`.

### Low Severity

11. **`MonadRec` `Clone` bound on `func` is over-constraining** (monad_rec.md). The trait requires `Clone` on the step function, but all current HKT implementors use simple loops where `Fn` alone suffices. The `Clone` bound exists for `Trampoline`'s recursive `go` pattern, but `Trampoline` cannot implement the HKT trait anyway.

12. **`SendThunk::into_arc_lazy` bypasses `Lazy` abstraction boundary** (send_thunk.md). Constructs `Lazy` directly via its tuple struct field rather than using a `From` impl or constructor.

13. **Minor copy-paste doc errors in `TryThunk`** (try_thunk.md). `fold_map` parameter descriptions say "The Thunk to fold" instead of "The TryThunk to fold" (lines 1180, 1933).

14. **`TryTrampoline::into_trampoline` naming inconsistency** (try_trampoline.md). `TryThunk` uses `into_inner`; `TryTrampoline` uses `into_trampoline`. Should be standardized.

15. **`TrySendThunk::ok` missing "Alias for `pure`" doc note** (try_send_thunk.md). `TryThunk::ok` documents itself as an alias for `pure`; `TrySendThunk::ok` does not.

16. **`SendRefFunctor` missing "Cache chain behavior" and "Why `FnOnce`?" doc sections** (send_ref_functor.md). These sections exist in `RefFunctor` and apply equally to `SendRefFunctor`.

17. **`RefFunctor` composition law doc uses reversed variable naming between abstract law and example** (ref_functor.md). Minor readability issue.

18. **`flatten_deque` doc says "iterative approach" but uses `rfold`** (cat_list.md). The `VecDeque` iterator's `rfold` is stack-safe, so the claim is effectively correct, but the wording is slightly misleading.

19. **`TryLazy` lacks `PartialEq`/`Eq`/`Hash`/`Ord` unlike `Lazy`** (try_lazy.md). Not documented why.

## 4. Missing Implementations

### Missing Type Class Instances

| Type | Missing Instance | Feasibility | Notes |
|------|-----------------|-------------|-------|
| `SendThunkBrand` | `Foldable`, `FoldableWithIndex` | Feasible | Fold functions do not need `Send`; they run after evaluation. |
| `LazyBrand<Config>` | `FoldableWithIndex` (Index = `()`) | Feasible | Straightforward; only requires `Foldable + WithIndex`. |
| `LazyBrand<Config>` | Semiring-family traits | Feasible | If the library has `Semiring`, `Ring`, etc. |
| `OptionBrand` | `MonadRec` | Trivial | Inherently stack-safe `bind`. |
| `VecBrand` | `MonadRec` | Trivial | Inherently stack-safe `bind`. |
| `ResultErrAppliedBrand<E>` | `MonadRec` | Trivial | Inherently stack-safe `bind`. |
| `IdentityBrand` | `MonadRec` | Trivial | Inherently stack-safe `bind`. |

### Missing Conversions

| From | To | Type | Notes |
|------|----|------|-------|
| `SendThunk<'a, A>` | `Thunk<'a, A>` | Zero-cost | Erase `Send` bound on closure. |
| `RcLazy` | `ArcLazy` | Eager | Force `RcLazy`, wrap result. |
| `ArcLazy` | `RcLazy` | Eager | Downgrade pointer. |
| `SendThunk` | `ArcLazy` | Lazy | `SendThunk` closure is already `Send`. (Partially exists via `into_arc_lazy`.) |
| `TrySendThunk` | Single-threaded types (`Thunk`, `Lazy`, `RcTryLazy`) | Eager | Cross-thread-boundary conversions. |
| `TryThunk` | `TryTrampoline` | With `'static` | Reverse of existing `From<TryTrampoline> for TryThunk`. |
| `TrySendThunk` | `ArcTryLazy` | Lazy | Via `From` impl. Currently only `into_arc_try_lazy` inherent method. |

### Missing Operations

| Type | Operation | Notes |
|------|-----------|-------|
| `TrySendThunk` | `tail_rec_m`, `arc_tail_rec_m` | Stack-safe fallible recursion for `Send` types. |
| `Free` | `hoist_free` | Transform `Free<F, A>` to `Free<G, A>` via natural transformation. Acknowledged missing in docs. |
| `Free` | Stack-safe `fold_free` variant | Via `resume` in iterative loop. |
| `TryLazy` | `bimap` | Map both `A` and `E` in one pass. |
| `Step` | `done() -> Option<B>`, `loop_val() -> Option<A>` | Non-panicking extractors. |
| `Step` | `swap()` | Swap `Loop`/`Done` variants. |

## 5. Documentation Gaps

### Incorrect or Weak Law Statements

- **`Evaluable`:** Naturality law is incorrectly stated; should be the map-extract law.
- **`MonadRec`:** "Equivalence" and "Safety varies" are not proper algebraic laws. The PureScript identity law is missing.
- **`Deferrable`:** Nesting law (`defer(|| defer(|| x)) == defer(|| x)`) is implied by transparency but not stated.

### Missing Documentation Sections

- `SendRefFunctor` lacks "Cache chain behavior" and "Why `FnOnce`?" sections present in `RefFunctor`.
- `SendThunk` lacks a comparison table (like `Thunk` has) and monad law documentation for its inherent methods.
- `TryLazy` does not document the absence of `PartialEq`/`Eq`/`Hash`/`Ord`.
- `CatListBrand` docs could note its role as the backbone of `Free` monad evaluation.
- `StepBrand` docs could mention its role in `MonadRec`.
- `SendThunkBrand` docs could note HKT trait limitations.

### Missing Property-Based Tests

- `Deferrable` transparency and nesting laws.
- `Evaluable` map-extract law.
- `MonadRec` identity law.
- `RefFunctor` identity and composition laws (only doc tests exist).
- `SendRefFunctor` identity and composition laws (only tested via inherent method, not trait/free function).

### Minor Doc Issues

- `TryThunk` fold_map parameter descriptions say "Thunk" instead of "TryThunk" (copy-paste).
- `SendRefFunctor` line 26 says "returning references" but should say "receiving references."
- `Evaluable` line 37 "Currently only ThunkBrand implements this trait" is fragile.
- `TryTrampoline::defer` has unusual `#[document_examples]` placement.

## 6. Design Strengths

1. **Config-parameterized `Lazy`/`TryLazy`:** The `LazyConfig`/`TryLazyConfig` traits cleanly abstract over `Rc`/`Arc` pointer choice, eliminating code duplication while preserving distinct `Send` bounds.

2. **"Reflection without Remorse" `Free` monad:** CatList-based implementation achieves O(1) bind and genuinely stack-safe evaluation. No unsafe code. Type erasure via `Box<dyn Any>` is sound.

3. **Comprehensive brand system:** Every type that can participate in HKT has a brand. Every type that cannot has a documented rationale. The naming is systematic and consistent.

4. **Rich conversion graph:** Extensive `From` implementations connect types across the hierarchy, with clear documentation of which conversions are eager vs lazy and why.

5. **Consistent API surface across variants:** `Thunk`, `SendThunk`, `TryThunk`, `TrySendThunk`, `Trampoline`, and `TryTrampoline` all share a common method vocabulary (`new`, `pure`, `defer`, `bind`, `map`, `evaluate`, `into_rc_lazy`/`into_arc_lazy`).

6. **`Step` type with triple HKT encoding:** Clean separation of bifunctor, Loop-applied, and Done-applied brands with comprehensive type class coverage and thorough QuickCheck law tests.

7. **CatList correctness:** Right data structure for `Free`; O(1) snoc, O(1) amortized uncons, O(1) append. Complete type class coverage. Correct amortized analysis.

8. **Documentation quality:** Thorough, consistent use of documentation macros across all components. Design rationale, limitations, and trade-offs are generally well-explained.

9. **`Deferrable`/`SendDeferrable` supertrait design:** Follows the library's established `Send` extension pattern. The eager-evaluation compromise is the correct trade-off for maintaining the supertrait relationship.

10. **Separation of `RefFunctor` and `SendRefFunctor`:** Correctly independent due to incompatible `Send` bound requirements. The alternative (`Functor` with `Clone`) would be strictly worse.

## 7. Recommendations (Prioritized)

### Priority 1: Correctness and Safety

1. **Add iterative `Drop` for `Free` `Wrap` chains.** Prevents stack overflow when dropping deeply nested wrap-only structures.
2. **Add iterative `Drop` for `CatList`.** Prevents stack overflow when dropping deeply nested `CatList` trees.
3. **Fix `Evaluable` naturality law.** Replace with `evaluate(map(f, fa)) == f(evaluate(fa))`.
4. **Add `tail_rec_m` and `arc_tail_rec_m` to `TrySendThunk`.** Closes the last gap in stack-safe recursion coverage.

### Priority 2: Completeness

5. **Implement `MonadRec` for `OptionBrand`, `VecBrand`, `ResultErrAppliedBrand<E>`, `IdentityBrand`.** Trivial implementations that significantly increase `MonadRec` utility.
6. **Add `From<SendThunk> for Thunk` conversion.** Zero-cost, enables ergonomic interop.
7. **Add `From<TryThunk> for TryTrampoline` conversion.** Completes the bidirectional conversion symmetry.
8. **Implement `Foldable` and `FoldableWithIndex` for `SendThunkBrand`.** Feasible since fold functions do not require `Send`.
9. **Implement `FoldableWithIndex` for `LazyBrand<Config>`.** Straightforward with index type `()`.
10. **Add `hoist_free` to `Free`.** Standard `Free` monad operation, acknowledged as missing.

### Priority 3: Documentation and Testing

11. **Rewrite `MonadRec` laws section.** State the PureScript identity law; reframe stack safety as a class invariant.
12. **Add QuickCheck property tests** for `Deferrable`, `Evaluable`, `MonadRec`, `RefFunctor`, and `SendRefFunctor` laws.
13. **Add "Cache chain behavior" and "Why `FnOnce`?" sections to `SendRefFunctor` docs.** Parity with `RefFunctor`.
14. **Fix `TryThunk` copy-paste doc errors** ("The Thunk to fold" at lines 1180, 1933).
15. **Standardize `into_inner` vs `into_trampoline` naming** across `TryThunk` and `TryTrampoline`.
16. **Add `SendThunk` comparison table and monad law docs** to match `Thunk` documentation depth.

### Priority 4: Nice-to-Have Enhancements

17. **Add a stack-safe `fold_free` variant** that uses `resume` in an iterative loop.
18. **Add `RcLazy` <-> `ArcLazy` conversions** (requires eager evaluation).
19. **Consider `Sync` bound on `SendRefFunctor::B`** or document why it is omitted.
20. **Add `bimap` to `TryLazy`.** Convenience for mapping both `A` and `E`.
21. **Add `From<TrySendThunk>` for `ArcTryLazy`.** Completes the conversion graph.
22. **Consider relaxing `MonadRec` `Clone` bound** on the step function, since all HKT implementors use simple loops.
