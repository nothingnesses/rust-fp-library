# Lazy Hierarchy: Consolidated Research Summary

This document consolidates findings from individual analyses of all types, traits, and brands in the lazy evaluation hierarchy. It is intended to give a complete picture of the current state, all known issues, and prioritized recommendations, so that someone reading only this summary understands the full scope of findings.

## Table of Contents

- [High-Level Overview](#high-level-overview)
- [Architecture Recap](#architecture-recap)
- [What Is Working Well](#what-is-working-well)
- [Correctness Issues](#correctness-issues)
- [Design Concerns and Structural Gaps](#design-concerns-and-structural-gaps)
- [Documentation Gaps](#documentation-gaps)
- [Testing Gaps](#testing-gaps)
- [Naming and Terminology Issues](#naming-and-terminology-issues)
- [Missing Implementations](#missing-implementations)
- [Inconsistencies](#inconsistencies)
- [Cross-Cutting Themes](#cross-cutting-themes)
- [Inherent vs. Addressable Limitations](#inherent-vs-addressable-limitations)
- [Prioritized Recommendations](#prioritized-recommendations)

---

## High-Level Overview

The lazy hierarchy consists of three infallible computation types (`Thunk`, `Trampoline`, `Lazy`) and three fallible counterparts (`TryThunk`, `TryTrampoline`, `TryLazy`), supported by the `Free` monad infrastructure, six trait definitions (`Deferrable`, `SendDeferrable`, `RefFunctor`, `SendRefFunctor`, `Evaluable`, and the `LazyConfig` strategy trait), and a set of HKT brands. A companion `fix` mechanism is provided as concrete functions (`rc_lazy_fix`, `arc_lazy_fix`) rather than as a trait.

**Overall verdict: the hierarchy is well-designed, correctly implemented, and thoroughly tested.** No correctness bugs were found in any of the 12 analyzed components. The type system constraints are handled honestly, with clear documentation of trade-offs. The main areas for improvement are documentation completeness, a few missing trait implementations (particularly for `TryLazy`), and some minor naming/terminology inconsistencies.

---

## Architecture Recap

| Type | Underlying | HKT | Stack Safe | Memoized | Lifetimes | Send |
|------|-----------|-----|-----------|----------|-----------|------|
| `Thunk<'a, A>` | `Box<dyn FnOnce() -> A + 'a>` | Yes (full) | Partial (`tail_rec_m` only) | No | `'a` | No |
| `Trampoline<A>` | `Free<ThunkBrand, A>` | No | Yes | No | `'static` | No |
| `RcLazy<'a, A>` | `Rc<LazyCell<A, ...>>` | Partial (`RefFunctor`) | N/A | Yes | `'a` | No |
| `ArcLazy<'a, A>` | `Arc<LazyLock<A, ...>>` | Partial (`SendRefFunctor`) | N/A | Yes | `'a` | Yes |
| `TryThunk<'a, A, E>` | `Thunk<'a, Result<A, E>>` | Yes (full) | Partial (`tail_rec_m` only) | No | `'a` | No |
| `TryTrampoline<A, E>` | `Trampoline<Result<A, E>>` | No | Yes | No | `'static` | No |
| `RcTryLazy<'a, A, E>` | `Rc<LazyCell<Result<A, E>, ...>>` | Kind only (no traits) | N/A | Yes | `'a` | No |
| `ArcTryLazy<'a, A, E>` | `Arc<LazyLock<Result<A, E>, ...>>` | Kind only (no traits) | N/A | Yes | `'a` | Yes |
| `Free<F, A>` | CatList-based "Reflection without Remorse" | No | Yes | No | `'static` | No |

The trait landscape:

| Trait | Purpose | Implementors in hierarchy |
|-------|---------|--------------------------|
| `Deferrable<'a>` | Lazy construction from thunk | `Thunk`, `Trampoline`, `RcLazy`, `RcTryLazy`, `TryThunk`, `Free<ThunkBrand, A>` |
| `SendDeferrable<'a>` | Thread-safe lazy construction | `ArcLazy`, `ArcTryLazy` |
| `RefFunctor` | Mapping with `&A` input | `LazyBrand<RcLazyConfig>` |
| `SendRefFunctor` | Thread-safe mapping with `&A` input | `LazyBrand<ArcLazyConfig>` |
| `Evaluable` | Natural transformation `F ~> Id` | `ThunkBrand` |

---

## What Is Working Well

1. **Core correctness.** No bugs were found in any implementation. All memoization, evaluation, type-erasure, and continuation-management logic is sound. The delegation to `std` types (`LazyCell`, `LazyLock`) for memoization is a good choice that reduces the surface area for bugs.

2. **Stack safety.** The `Free` monad's "Reflection without Remorse" encoding with `CatList`-based continuations provides genuine O(1) bind and constant-stack evaluation. The trampoline loop in `Free::evaluate` is correct and well-tested. `Trampoline::tail_rec_m` and `TryTrampoline::tail_rec_m` both correctly defer recursive calls via `Trampoline::defer`.

3. **The three-type separation.** The division into `Thunk` (lightweight, HKT-compatible, lifetime-polymorphic), `Trampoline` (stack-safe, `'static`-only, no HKT), and `Lazy` (memoized, reference-returning) is well-motivated. Each type fills a distinct niche, and the trade-offs are clearly documented.

4. **The `Try*` variants.** The newtype-over-`Result` pattern (`TryThunk<'a, A, E> = Thunk<'a, Result<A, E>>`, etc.) is consistently applied across all three base types. The newtypes add genuine value through error-aware `bind` (short-circuiting on `Err`), `catch`, `map_err`, `catch_unwind`, and proper HKT brand support with separate success-channel and error-channel functors.

5. **The `LazyConfig` strategy pattern.** Parameterizing `Lazy` over a config type that bundles pointer, cell, and thunk choices is a clean abstraction. It avoids duplicating the entire type definition while allowing `RcLazy` and `ArcLazy` to have different thread-safety properties. The `PointerBrand` associated type links configs back to the pointer hierarchy.

6. **Trait design choices.** The decision to omit `fix` from `Deferrable` (because it requires shared ownership and interior mutability specific to `Lazy`) is well-reasoned and well-documented. The `RefFunctor`/`SendRefFunctor` split honestly represents what `Lazy` can do without implicit cloning. The `Deferrable`/`SendDeferrable` split mirrors the library-wide Rc/Arc pattern.

7. **Documentation quality.** All types and traits use the library's documentation macro system (`#[document_signature]`, `#[document_parameters]`, etc.) consistently. Doc examples compile and use assertions. The `Deferrable` trait's "Why there is no generic `fix`" section is exemplary. The `Free` module's "What it CAN do / What it CANNOT do" section is helpful.

8. **Test coverage.** Property-based tests (QuickCheck) verify functor laws, monad laws, semigroup associativity, and monoid identity across `Thunk`, `TryThunk`, `Trampoline`, `TryTrampoline`, and `Lazy`. Drop safety, panic poisoning, thread safety, and conversions are all tested.

---

## Correctness Issues

No outright correctness bugs were found. The following are subtle behavioral characteristics that could surprise users:

1. **`fold_free` is not stack-safe.** Unlike `Free::evaluate` (which is iterative), `fold_free` uses actual recursion. For strict target monads (e.g., `OptionBrand`), each `Wrap` layer produces one level of call-stack growth. Deep `Free` computations can overflow during `fold_free`. This is undocumented. (Source: free.md)

2. **`Free::Drop` may be incomplete for deeply nested `Wrap` chains.** The custom `Drop` implementation walks through `Bind` and `Map` chains iteratively to prevent stack overflow, but does not handle `Wrap` variants that might contain deeply nested `Free` values inside the functor. For `ThunkBrand` this is fine, but for other functors with nesting, it could overflow. (Source: free.md)

3. **`Semigroup::append` for `TryThunk` evaluates both sides eagerly.** When `a` succeeds but `b` fails, `b` is still evaluated. This does not short-circuit like `bind`. This is not a bug (both sides must be checked), but it diverges from the short-circuit semantics users might expect from a fallible type. (Source: try_thunk.md)

4. **`From<Lazy>` and `From<TryLazy>` conversions to `TryTrampoline` are eager.** These conversions force evaluation at conversion time, then wrap the result in `Trampoline::pure`. Users expecting lazy semantics may be surprised. The `From<TryThunk>` conversion, by contrast, correctly defers evaluation. This asymmetry is documented for `From<TryLazy>` but not for `From<Lazy>`. (Source: try_trampoline.md)

5. **`Deferrable::defer` for `Lazy` loses memoization of inner values.** `defer(f)` creates `RcLazy::new(move || f().evaluate().clone())`. The inner `Lazy` is created, immediately evaluated, and discarded. Its cache is not preserved. This is correct (transparency law holds) but may surprise users. (Source: deferrable.md)

---

## Design Concerns and Structural Gaps

### TryLazy has no HKT trait implementations (high impact)

`TryLazy` has a `Kind` instance via `impl_kind!` for `TryLazyBrand<E, Config>`, but no type class traits are implemented for this brand. No `RefFunctor`, no `Foldable`, no `Bifunctor`, nothing. The `map` and `map_err` methods exist only as inherent methods. For comparison, `TryThunk` has full HKT support (`Functor`, `Semimonad`, `Bifunctor`, `Bifoldable`, `Foldable`, `Pointed`, `Lift`, `MonadRec`).

This is the most significant gap in the hierarchy. `TryLazy` should have at minimum a `RefFunctor` implementation (matching `Lazy`'s pattern). A `RefBifunctor` trait could also be introduced for mapping both sides. (Source: try_lazy.md)

### No `Send` variants for `Thunk`, `Trampoline`, or `Free`

None of the non-memoized computation types (`Thunk`, `Trampoline`, `Free`) support `Send`. Their internal closures are `Box<dyn FnOnce() -> A + 'a>` without `Send` bounds. This means:

- `Thunk::memoize_arc()` must evaluate eagerly (loses laziness).
- `Trampoline::memoize_arc()` must evaluate eagerly.
- No stack-safe computation can be sent across threads without eager evaluation.

A `SendThunk` variant (wrapping `Box<dyn FnOnce() -> A + Send + 'a>`) would be a useful addition that enables truly lazy `memoize_arc` and thread-safe deferred computation chains. (Sources: thunk.md, trampoline.md, free.md)

### The `Deferrable`/`SendDeferrable` and `RefFunctor`/`SendRefFunctor` splits have no unification point

`SendDeferrable` does not extend `Deferrable`. `SendRefFunctor` does not extend `RefFunctor`. This means generic code written against `Deferrable` or `RefFunctor` cannot accept `ArcLazy`. Code must be written twice (once for each trait) or choose one variant.

This is consistent with some library patterns (`SendRefFunctor` stands alone) but inconsistent with others (`SendCloneableFn: CloneableFn` and `SendRefCountedPointer: RefCountedPointer` use supertraits). The current design is defensible (it makes the `Send`/non-`Send` distinction explicit), but the inconsistency across the library's `Send*` traits is worth noting. (Sources: deferrable.md, send_deferrable.md, ref_functor.md, send_ref_functor.md)

### `ArcLazy` does not implement `Deferrable`

`Deferrable` is only implemented for `Lazy<'a, A, RcLazyConfig>`. `ArcLazy` implements only `SendDeferrable`. This means you cannot call the `defer` free function on an `ArcLazy`. If `SendDeferrable` had `Deferrable` as a supertrait (following the `SendCloneableFn: CloneableFn` precedent), this would be resolved. (Source: send_deferrable.md)

### Stack-safe types cannot participate in HKT generic programming

`Trampoline`, `TryTrampoline`, and `Free` cannot have brands because `Box<dyn Any>` requires `'static`, while `Kind::Of<'a, A: 'a>` requires lifetime polymorphism. This means the most powerful computation types in the hierarchy are excluded from generic type class code (`map::<TrampolineBrand, _, _>(f, trampoline)` is impossible). Instead, these types provide direct method-based APIs. This is a significant gap: the most powerful computation types in the hierarchy are the ones that cannot participate in generic type class programming.

This is inherent to the "Reflection without Remorse" technique in Rust and cannot be addressed without either unsafe code or a fundamentally different approach to type erasure. (Sources: trampoline.md, free.md, brands.md)

### `E: 'static` required for HKT brands on `Try*` types

Both `TryThunkErrAppliedBrand<E>` and `TryThunkOkAppliedBrand<A>` require their fixed type parameter to be `'static`. This prevents using HKT-generic functions with `TryThunk` when the error or success type borrows from a local scope. The inherent methods (`.map`, `.bind`) still work with any lifetime, so this only affects brand-level generic code. (Sources: try_thunk.md, brands.md)

### Significant code duplication between Rc and Arc variants

The `Foldable`, `Semigroup`, `Monoid`, and other trait implementations for `Lazy` are duplicated nearly verbatim for `RcLazyConfig` and `ArcLazyConfig`, differing only in `Send + Sync` bounds. This is approximately 250+ lines of duplicated code. The same pattern affects `TryLazy`. A `macro_rules!` helper could reduce this duplication. The `LazyConfig` trait does not provide enough abstraction to write a single generic impl because `Send`/`Sync` requirements cannot be expressed conditionally in Rust's current type system. (Source: lazy.md)

### `LazyConfig` bundles fallible and infallible types together

The `LazyConfig` trait carries `TryLazy`, `TryThunk`, `try_lazy_new`, and `try_evaluate` alongside infallible variants. Any custom `LazyConfig` implementor must define both, even if only one is needed. Splitting into `LazyConfig` and `TryLazyConfig` would improve separation of concerns at the cost of more traits. (Sources: lazy.md, try_lazy.md)

---

## Documentation Gaps

### Missing warnings and explanations

1. **`fold_free` stack safety is undocumented.** Users may assume it is stack-safe like `evaluate`, but it uses actual recursion. (Source: free.md)
2. **`TryThunk` struct docs do not warn about stack unsafety of `bind` chains.** `Thunk` documents this prominently; `TryThunk` should match. (Source: try_thunk.md)
3. **`TryThunk` has no "Traversable limitation" note.** `Thunk` documents why it cannot implement `Traversable`; `TryThunk` should do the same. (Source: try_thunk.md)
4. **`TryThunk` has no algebraic properties section.** `Thunk` documents its monad laws; `TryThunk` should document the same for its success-channel monad. (Source: try_thunk.md)
5. **`SendDeferrable` has no laws documentation.** `Deferrable` documents a transparency law; `SendDeferrable` should state the same law. (Source: send_deferrable.md)
6. **`SendDeferrable` does not mention `arc_lazy_fix`.** `Deferrable` has a detailed discussion of `fix` and references `rc_lazy_fix`/`arc_lazy_fix`. `SendDeferrable` does not mention `fix` at all. (Source: send_deferrable.md)
7. **`TryLazy::evaluate()` has no `# Panics` section.** Panic poisoning behavior is documented on `Lazy` but not on `TryLazy`. (Source: try_lazy.md)
8. **`rc_lazy_fix`/`arc_lazy_fix` lack recursion limit warnings.** If the function `f` forces the self-reference, infinite recursion (stack overflow) occurs. (Source: lazy.md)
9. **`Lazy` module docs do not explain why `Functor` is not implemented.** Users will wonder why they cannot `map` over a `Lazy`. The docs should point to `RefFunctor` and explain the `&A` vs `A` issue. (Source: lazy.md)
10. **`Thunk`'s `tail_rec_m` does not warn that step functions should return shallow thunks.** If `f` builds deep `bind` chains inside the returned thunk, the `evaluate()` call in the loop could still overflow. (Source: thunk.md)
11. **`SendRefFunctor` trait doc does not explain why a separate trait is needed.** The explanation exists in `ArcLazy::ref_map`'s inline comment but is absent from the trait doc itself. (Source: send_ref_functor.md)
12. **No "when to use" guidance in `TryTrampoline` or `TryThunk` docs.** The crate-level docs cover this, but the type-level documentation lacks it. (Sources: try_trampoline.md, try_thunk.md)
13. **`TryLazy` docs do not explain Clone requirements for `map`/`map_err`.** The bounds enforce correctness, but the prose does not explain why `Clone` is needed on the "other" type parameter. (Source: try_lazy.md)
14. **`TryLazy` docs do not explain cache chain behavior.** Chaining `map` calls creates a linked list of cells holding `Rc`/`Arc` references to predecessors. (Source: try_lazy.md)

### Stale and inaccurate references

15. **`Free` module docs reference `Runnable` trait; the actual trait is `Evaluable`.** (Source: free.md)
16. **`Free` test comment references `Free::roll`; the method is named `Free::wrap`.** (Source: free.md)
17. **`Free` uses "SAFETY" comments on non-`unsafe` code.** These are invariant-preservation comments, not memory safety comments. "INVARIANT" would be more precise. (Source: free.md)
18. **`Lazy` has inconsistent lifetime parameter description.** The struct docs say "The lifetime of the reference," while `Deferrable` and fix functions say "The lifetime of the computation." The latter is more accurate. (Source: lazy.md)
19. **`Trampoline` doc link for `bind` points to `crate::functions::bind` (the free function) rather than the inherent method.** (Source: trampoline.md)

### Minor documentation issues

20. **Unnecessary `brands::*` imports in `Deferrable` doc examples.** Three examples import `brands::*` but use no brand types. (Source: deferrable.md)
21. **Missing period in `Deferrable` free function's type parameter description.** "The lifetime of the computation" lacks a trailing period. (Source: deferrable.md)
22. **`SendDeferrable` trait lacks `#[document_examples]`.** `Deferrable` has it; `SendDeferrable` does not. (Source: send_deferrable.md)
23. **`Trampoline`'s `memoize` doc example uses `*lazy.evaluate()` without explaining the deref.** (Source: trampoline.md)
24. **`Trampoline`'s `defer` doc claims "n = 1,000,000" works but tests only use n = 1,000.** The claim is correct but untested in CI. (Source: trampoline.md)

---

## Testing Gaps

1. **`Thunk`: Missing tests for `tail_rec_m` at non-trivial depth.** The doc example uses 1,000 but there is no dedicated stress test. (Source: thunk.md)
2. **`Thunk`: Missing tests for HKT-level trait operations.** `Foldable`, `Lift::lift2`, `Semiapplicative::apply`, and `Evaluable::evaluate` are untested via the HKT interface. (Source: thunk.md)
3. **`Thunk`: Missing tests for `memoize` and `memoize_arc`.** (Source: thunk.md)
4. **`TryThunk`: No QuickCheck tests for bifunctor laws (identity, composition).** (Source: try_thunk.md)
5. **`TryThunk`: No QuickCheck tests for error-channel monad laws (`TryThunkOkAppliedBrand`).** (Source: try_thunk.md)
6. **`TryThunk`: No tests for `Semigroup`/`Monoid` laws.** (Source: try_thunk.md)
7. **`TryThunk`: No test verifying `memoize_arc` thread safety.** (Source: try_thunk.md)
8. **`Trampoline`: Test names `test_task_map2` and `test_task_and_then` do not match current method names (`lift2` and `then`).** (Source: trampoline.md)
9. **`Trampoline`: A deeper stress test (e.g., 100k iterations) would strengthen stack safety claims.** Tests currently use only 1,000 depth. (Source: trampoline.md)
10. **`Lazy`: `LazyConfig` extensibility claim is untested.** The docs say the trait is "open for third-party implementations" but no third-party config exists in the codebase or tests. (Source: lazy.md)

---

## Naming and Terminology Issues

1. **"Eval" remnants in `Thunk` tests.** Test names like `test_eval_from_memo`, `test_eval_semigroup` and doc strings referencing "eval" appear to be remnants from when `Thunk` was called `Eval`. These should be updated. (Source: thunk.md)
2. **`Trampoline` test names reference old method names.** `test_task_map2` should be `test_task_lift2`; `test_task_and_then` should be `test_task_then`. (Source: trampoline.md)
3. **`TryThunk` test references `pure` as deprecated, but no `#[deprecated]` attribute exists.** Either the deprecation attribute should be added or the test annotation should be removed. (Source: try_thunk.md)

---

## Missing Implementations

### Trait implementations

1. **`TryLazy`: No `RefFunctor` or analogous HKT trait.** This is the most significant gap. `Lazy` has `RefFunctor`; `TryLazy` has nothing beyond `Kind`. (Source: try_lazy.md)
2. **`TryLazy`: No `Semigroup`/`Monoid`.** `Lazy` has both. Natural semantics would be: combine if both Ok, propagate first Err. (Source: try_lazy.md)
3. **`TryLazy`: No `Foldable`.** `Lazy` has `Foldable`; `TryLazy` does not. (Source: try_lazy.md)
4. **`TryTrampoline`: No `Semigroup`/`Monoid`.** `Trampoline` has both; `TryTrampoline` does not. (Source: try_trampoline.md)
5. **`TryTrampoline`: No `bimap` method.** `TryThunk` has `Bifunctor`; `TryTrampoline` only has separate `map` and `map_err`. A `bimap` convenience method is absent. (Source: try_trampoline.md)
6. **`TryLazy`: Missing `TryLazyBrand` bifunctor brand.** Unlike `TryThunk` (which has `TryThunkBrand` as a full bifunctor brand), `TryLazy` only has the error-applied `TryLazyBrand<E, Config>`. (Source: brands.md)
7. **`Thunk`: No `FunctorWithIndex` or `FoldableWithIndex`.** These are trivially implementable (index is always `()`). (Source: thunk.md)

### Conversions

8. **`Lazy`: No `From<Thunk>` or `From<Trampoline>` for `ArcLazy`.** Only `RcLazy` has these conversions. `ArcLazy` variants would need `Send` bounds. (Source: lazy.md)
9. **`TryLazy`: No `From<TryThunk>` or `From<TryTrampoline>` for `ArcLazyConfig`.** Same gap as above for the fallible variants. (Source: try_lazy.md)

### Standard trait impls

10. **`Lazy`: Missing `Eq` and `Ord`.** Only `PartialEq` and `PartialOrd` are implemented. `Eq` and `Ord` could be added with appropriate bounds. (Source: lazy.md)

### Brand aliases

11. **No type aliases for common brand+config combinations.** `FnBrand` has `RcFnBrand`/`ArcFnBrand` aliases; `LazyBrand` lacks corresponding `RcLazyBrand`/`ArcLazyBrand`. (Source: brands.md)

---

## Inconsistencies

1. **Supertrait patterns for `Send*` traits are inconsistent.** `SendCloneableFn: CloneableFn` and `SendRefCountedPointer: RefCountedPointer` use supertraits. `SendDeferrable`, `SendRefFunctor` do not. (Sources: deferrable.md, send_deferrable.md, ref_functor.md, send_ref_functor.md)

2. **`TryLazy` has far fewer HKT trait impls than `TryThunk`.** `TryThunk` has `Functor`, `Semimonad`, `Bifunctor`, `Bifoldable`, `Foldable`, `Pointed`, `Lift`, `MonadRec`. `TryLazy` has only a `Kind` mapping. While this reflects the `Lazy`/`Thunk` gap (reference vs. owned evaluation), `TryLazy` does not even have the `RefFunctor` that `Lazy` has. (Source: try_lazy.md)

3. **`TryTrampoline` has no `Semigroup`/`Monoid` while `Trampoline` does.** (Source: try_trampoline.md)

4. **`TryThunk` does not restate limitations documented on `Thunk`.** Stack unsafety of `bind` chains and `Traversable` impossibility are documented on `Thunk` but not on `TryThunk`. (Source: try_thunk.md)

5. **`TryLazy` bakes fallible types into `LazyConfig` while `TryThunk` wraps `Thunk` directly.** `TryThunk(Thunk<Result<A, E>>)` is a simple newtype. `TryLazy` uses `Config::TryLazy<'a, A, E>` as a separate associated type in `LazyConfig`. This means `LazyConfig` is larger than necessary and any new config must define both infallible and fallible variants. The `TryThunk`/`TryTrampoline` approach of wrapping the base type is simpler. (Sources: try_lazy.md, lazy.md)

6. **`Lazy` lifetime parameter description varies.** "The lifetime of the reference" on the struct vs. "The lifetime of the computation" on `Deferrable` and fix functions. (Source: lazy.md)

---

## Cross-Cutting Themes

### 1. Rust's ownership model forces reference-based evaluation for memoized types

This is the single most impactful theme. Because `Lazy::evaluate()` returns `&A` (not `A`), memoized types cannot implement `Functor`, `Monad`, `Applicative`, `Comonad`, or `Traversable` from the standard hierarchy. This cascades into:
- The need for `RefFunctor` / `SendRefFunctor` as separate traits.
- `Clone` requirements on operations that cross the reference/owned boundary (`map_err` on `TryLazy`, `Deferrable::defer` for `Lazy`, `Foldable` for `Lazy`).
- The absence of a full type class hierarchy for `Lazy` and `TryLazy`.

This is inherent to Rust and cannot be addressed without either implicit cloning (violating the zero-cost principle) or a fundamentally different memoization design.

### 2. The Rc/Arc split creates pervasive duplication

Every trait that applies to `Lazy` must be implemented twice: once for `RcLazyConfig` and once for `ArcLazyConfig`. The same applies to `TryLazy`. The duplication is mechanical (identical logic, different `Send`/`Sync` bounds) and accounts for hundreds of lines. Rust's type system does not support conditional bounds or specialization in stable, so this cannot be fully unified. A `macro_rules!` helper is the pragmatic fix.

### 3. `'static` requirements exclude stack-safe types from HKT

The "Reflection without Remorse" technique requires `Box<dyn Any>` for type erasure, which forces `'static`. The HKT `Kind` trait requires lifetime polymorphism. These are fundamentally incompatible, meaning `Trampoline`, `TryTrampoline`, and `Free` live outside the brand/type-class system. Users must work with inherent methods for these types.

### 4. No `Send` variant for non-memoized computation types

`Thunk`, `Trampoline`, and `Free` are all `!Send`. The `Lazy` hierarchy handles thread safety through `ArcLazy`, but there is no equivalent for non-memoized computations. This forces eager evaluation in `memoize_arc()` for all non-memoized types, losing the deferred computation benefit.

### 5. Fallible (`Try*`) variants are consistently designed but not equally complete

All three `Try*` types follow the same newtype-over-`Result` pattern, which is good. However, their trait coverage varies widely:
- `TryThunk`: full HKT support (Functor, Monad, Bifunctor, etc.).
- `TryTrampoline`: no HKT, but rich inherent API.
- `TryLazy`: no HKT traits, minimal API relative to `Lazy`.

The gap is widest for `TryLazy`, which has a `Kind` instance but no trait implementations for it.

### 6. Documentation standards are high but unevenly applied

Types that were developed first (`Thunk`, `Lazy`, `Deferrable`) have more thorough documentation (limitation notes, algebraic properties, comparison tables) than types developed later or derived from them (`TryThunk`, `TryTrampoline`, `SendDeferrable`). The documentation macro system ensures structural consistency, but the prose content varies.

---

## Inherent vs. Addressable Limitations

### Inherent to Rust's type system (cannot be fixed)

- `Lazy` cannot implement standard `Functor`/`Monad` because `evaluate()` returns `&A`.
- `Trampoline`/`Free` cannot have HKT brands because `Box<dyn Any>` requires `'static`.
- `Deferrable`/`RefFunctor` cannot be unified with `SendDeferrable`/`SendRefFunctor` into single traits because `Send` bounds cannot be conditionally added.
- Trait impls for `LazyBrand` must be written separately per config because Rust lacks specialization.
- `TryThunkErrAppliedBrand<E>` requires `E: 'static` because brand type parameters must be `'static`.
- `Thunk`/`TryThunk` cannot implement `Traversable` because `FnOnce` closures cannot be cloned.
- `Thunk` bind chains are not stack-safe (nesting closures is fundamental to the closure-based design).
- `Lazy` operations like `Foldable` and `Deferrable::defer` require `Clone` because they cross the reference/owned boundary.

### Addressable (could be fixed with implementation work)

- `TryLazy` has no `RefFunctor` or other HKT trait implementations.
- `TryLazy` has no `Semigroup`/`Monoid`.
- `TryTrampoline` has no `Semigroup`/`Monoid`.
- No `SendThunk` variant exists for thread-safe deferred computation.
- `fold_free` stack safety is undocumented.
- Various documentation gaps (missing warnings, stale references, missing "when to use" guidance).
- Test coverage gaps (missing HKT-level tests for `Thunk`, missing bifunctor law tests for `TryThunk`).
- Naming inconsistencies ("eval" remnants, stale test names).
- `Lazy` missing `Eq`/`Ord` implementations.
- Missing `From` conversions for `ArcLazy`/`ArcTryLazy`.
- Missing brand aliases (`RcLazyBrand`, `ArcLazyBrand`).
- Code duplication could be reduced with `macro_rules!` helpers.

---

## Prioritized Recommendations

### High priority (correctness, safety, or significant capability gaps)

1. **Document `fold_free` stack safety limitation.** Users may assume it is safe like `evaluate`. Add a warning to the `fold_free` doc comment explaining that it uses actual recursion and can overflow with strict target monads. (Source: free.md)

2. **Implement `RefFunctor` for `TryLazyBrand<E, RcLazyConfig>` and `SendRefFunctor` for `TryLazyBrand<E, ArcLazyConfig>`.** This is the largest functional gap in the hierarchy. `TryLazy` has a Kind instance but no traits, making the brand effectively dead weight. (Source: try_lazy.md)

3. **Add stack safety warnings to `TryThunk` documentation.** `TryThunk` inherits the bind-chain stack unsafety from `Thunk` but does not document it. (Source: try_thunk.md)

4. **Add `# Panics` section to `TryLazy::evaluate()`.** Panic poisoning behavior is documented on `Lazy` but missing from `TryLazy`. (Source: try_lazy.md)

### Medium priority (consistency, completeness, documentation)

5. **Add `Semigroup`/`Monoid` to `TryLazy` and `TryTrampoline`.** Both base types (`Lazy`, `Trampoline`) have these; the fallible variants should too, with standard Result-like combining semantics. (Sources: try_lazy.md, try_trampoline.md)

6. **Add transparency law documentation to `SendDeferrable`.** Mirror `Deferrable`'s law section, and add `#[document_examples]` with a law example. (Source: send_deferrable.md)

7. **Fix stale documentation references in `Free`.** Replace "Runnable" with "Evaluable," replace "roll" with "wrap," change "SAFETY" to "INVARIANT" on non-unsafe downcast comments. (Source: free.md)

8. **Add recursion limit warnings to `rc_lazy_fix`/`arc_lazy_fix`.** Document that forcing the self-reference inside `f` leads to infinite recursion. (Source: lazy.md)

9. **Add explanation of why `Functor` is not implemented to `Lazy` module docs.** Point to `RefFunctor` and explain the `&A` vs `A` issue. (Source: lazy.md)

10. **Clean up "eval" naming remnants in `Thunk` tests.** Rename `test_eval_from_memo`, `test_eval_semigroup`, etc. (Source: thunk.md)

11. **Rename `Trampoline` tests to match current method names.** `test_task_map2` to `test_task_lift2`; `test_task_and_then` to `test_task_then`. (Source: trampoline.md)

12. **Resolve `TryThunk` `pure` deprecation confusion.** Either add `#[deprecated]` to the method or remove `#[allow(deprecated)]` from the test. (Source: try_thunk.md)

13. **Fix `Lazy` lifetime parameter description inconsistency.** Standardize on "The lifetime of the computation" across all uses. (Source: lazy.md)

14. **Add `Thunk`'s `tail_rec_m` shallow-thunk warning.** Document that the step function should return shallow thunks, not deep bind chains. (Source: thunk.md)

### Low priority (ergonomics, completeness, minor gaps)

15. **Add `bimap` method to `TryTrampoline`.** (Source: try_trampoline.md)

16. **Add type aliases `RcLazyBrand`, `ArcLazyBrand`, `RcTryLazyBrand<E>`, `ArcTryLazyBrand<E>`.** Matches the `RcFnBrand`/`ArcFnBrand` pattern. (Source: brands.md)

17. **Add `Eq`/`Ord` implementations for `Lazy`.** Straightforward with appropriate bounds. (Source: lazy.md)

18. **Add `From<Thunk>`/`From<Trampoline>` conversions for `ArcLazy`.** Requires `Send` bounds on the inner values. (Source: lazy.md)

19. **Add `From<TryThunk>`/`From<TryTrampoline>` conversions for `ArcTryLazy`.** (Source: try_lazy.md)

20. **Implement `FunctorWithIndex` and `FoldableWithIndex` for `Thunk` (index is `()`).** Trivial implementation, improves PureScript parity. (Source: thunk.md)

21. **Expand test coverage:** Missing QuickCheck tests for `TryThunk` bifunctor laws; missing HKT-level tests for `Thunk`'s `Foldable`, `Lift`, `Apply`, `Evaluable`; missing `memoize`/`memoize_arc` tests for `Thunk`; add deeper stack safety stress tests for `Trampoline`. (Sources: thunk.md, try_thunk.md, trampoline.md)

22. **Remove unnecessary `brands::*` imports from `Deferrable` doc examples.** (Source: deferrable.md)

23. **Consider `SendDeferrable: Deferrable` supertrait** and implementing `Deferrable` for `ArcLazy`, following the `SendCloneableFn: CloneableFn` precedent. This is a design decision with trade-offs. (Source: send_deferrable.md)

24. **Consider a `SendThunk` variant** for thread-safe deferred computation. This would enable truly lazy `memoize_arc` and fill the gap between `Thunk` (not `Send`) and `ArcLazy` (memoized, `Send`). (Source: thunk.md)

25. **Reduce Rc/Arc code duplication** in `Lazy` and `TryLazy` trait implementations using `macro_rules!` helpers. (Source: lazy.md)

26. **Add "when to use" guidance** to `TryThunk`, `TryTrampoline`, and `TryLazy` type-level documentation. (Sources: try_thunk.md, try_trampoline.md, try_lazy.md)

27. **Add `SendRefFunctor` motivation explanation** to the trait doc itself (not just the `ArcLazy::ref_map` inline comment). (Source: send_ref_functor.md)

28. **Consider splitting `LazyConfig` to separate fallible and infallible associated types** into distinct traits. (Sources: lazy.md, try_lazy.md)
