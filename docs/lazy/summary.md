# Lazy Hierarchy Analysis: Consolidated Summary

This document consolidates findings from 10 independent analyses of the Lazy type hierarchy in `fp-library`. Each analysis examined the design from a different angle. Below is a synthesis of the key findings, organized by theme.

## Overall Verdict

**The design is sound and well-motivated.** The three-type decomposition (Thunk/Trampoline/Lazy) correctly separates three genuinely independent concerns that Rust's ownership and lifetime system forces to be explicit: lifetime flexibility, stack safety, and memoization. The Try* variants add ergonomic error handling without architectural complexity. The `LazyConfig` trait cleanly handles the Rc/Arc split.

The design compares favorably to Scala's `Eval` (which conflates concerns Rust must separate) and is the natural Rust-native decomposition of what Haskell and PureScript provide via runtime support.

---

## Key Strengths

### Architecture
- **Three-type split is necessary and well-chosen** (Analysis 1). The `'a` vs `'static` lifetime divide, stack safety vs zero-overhead, and computation vs memoization are genuinely orthogonal in Rust.
- **Try* variants are justified** (Analyses 1, 5). They provide `EitherT`-style short-circuiting that would be painful to implement manually with `Thunk<'a, Result<A, E>>`.
- **LazyConfig trait is clever** (Analyses 4, 8). It bundles pointer, cell, and thunk types into a single configuration, preventing invalid combinations at compile time while remaining extensible.

### Correctness
- **No soundness issues identified** (Analyses 4, 7). All lifetime bounds are correct and conservative. Interior mutability is delegated to audited std types (`LazyCell`, `LazyLock`).
- **Type class laws are satisfied** (Analysis 2). Functor, Monad, RefFunctor, Deferrable laws all hold, with QuickCheck property tests providing evidence for Thunk, Trampoline, and Lazy.
- **Free monad encoding is correct** (Analysis 3). The "Reflection without Remorse" CatList optimization provides O(1) bind. Stack-safe drop is implemented. All four `tail_rec_m` implementations are correct.

### API Design
- **Consistent naming** across all types: `new`, `pure`, `defer`, `evaluate`, `memoize` (Analysis 10).
- **Comprehensive `From` conversion network** (Analyses 1, 5, 10) enables smooth composition between types.
- **Well-calibrated method surfaces** (Analysis 10): computation types have more methods than caching types.

---

## Issues and Concerns

### Critical

1. **README inaccuracy** (Analysis 9): Line 192 states Trampoline "requires types to be `'static` and `Send`." This is wrong; Trampoline only requires `A: 'static`. The test suite explicitly tests Trampoline with `Rc<T>` (a `!Send` type). The `Send` bound only appears in `From<Trampoline> for Thunk`.

2. **Zero QuickCheck property tests for all three Try* types** (Analysis 9). `TryThunk`, `TryTrampoline`, and `TryLazy` have unit tests but no property-based tests for type class laws.

### Significant

3. **`From<Trampoline<A>> for Thunk<'static, A>` requires `A: Send`** (Analyses 7, 10), which is unnecessarily restrictive. Neither Trampoline nor Thunk is `Send`. This prevents converting `Trampoline<Rc<T>>` to `Thunk`.

4. **ArcLazy lacks `RefFunctor` and `Foldable` at the HKT level** (Analysis 2). Only inherent `ref_map` exists. The `RefFunctor` trait cannot require `Send` on the mapping function without breaking other implementors. This limits generic HKT code over `LazyBrand<ArcLazyConfig>`.

5. **TryLazy has no `bind`, `map`, `map_err`, or `catch` methods** (Analysis 5). This is a significant API gap compared to `TryThunk` and `TryTrampoline`. While justified by memoization semantics, it hurts composability.

6. **No `memoize()` on any Try* type** (Analysis 5). Both `Thunk` and `Trampoline` have `memoize()`, but `TryThunk` and `TryTrampoline` lack this convenience.

7. **`From<TryTrampoline> for RcTryLazy` requires `A: Send, E: Send`** (Analysis 5), which is surprising for a single-threaded `RcTryLazy`. This prevents memoizing a `TryTrampoline<Rc<T>, E>` into `RcTryLazy`.

### Moderate

8. **Trampoline's `'static` requirement is an implementation artifact** (Analysis 7). It arises from `Box<dyn Any>` in the Free monad, not from a semantic necessity. A different type erasure strategy could theoretically relax this, but it would require a fundamental rewrite.

9. **`catch_unwind` is missing from `TryTrampoline`** (Analysis 5) but present on `TryThunk` and `TryLazy`. Since `TryTrampoline` is specifically for deep recursion (where stack overflow panics are a real concern), this is a notable gap.

10. **Duplicate `catch_unwind` panic-to-string logic** (Analysis 5) is copy-pasted between `TryThunk` and `TryLazy`. Should be extracted to a shared helper.

11. **`TryThunk::pure` and `TryThunk::ok` coexist as aliases** (Analysis 10). `TryTrampoline` only has `ok`, not `pure`. This inconsistency adds confusion.

12. **`evaluate` name overloading** (Analysis 1): `Thunk::evaluate(self) -> A` vs `Lazy::evaluate(&self) -> &A` differ in ownership semantics but share the same name. This can surprise users.

13. **Doc inaccuracies in trampoline.rs** (Analysis 9): Line 60 says "run" but the method is `evaluate`; lines 128-130 say "Prints 'Computing!'" but the print is commented out.

### Minor

14. **No Foldable for ArcLazy** (Analyses 2, 9). Likely just an oversight; the RcLazy implementation would translate directly.

15. **`Evaluable` trait doc typo** (Analysis 6): Line 72 says "to evaluable" instead of "to evaluate."

16. **No `memoize_arc()` convenience** (Analysis 10). The existing `.memoize()` always returns `RcLazy`.

17. **No `From<TryTrampoline> for TryThunk`** (Analysis 5) for the reverse direction.

18. **No grouping submodule** for lazy types in `types.rs` (Analysis 10); they are interspersed with unrelated types.

19. **`From` conversion costs involving `.clone()` are undocumented** (Analysis 10).

20. **No panic poisoning test for Lazy/TryLazy** (Analysis 9), despite it being documented.

---

## Recommendations

### High Priority

1. **Fix the README** to remove the incorrect `Send` claim about Trampoline.
2. **Add QuickCheck property tests** for TryThunk, TryTrampoline, and TryLazy.
3. **Relax the `A: Send` bound** on `From<Trampoline<A>> for Thunk<'static, A>`.
4. **Add `memoize()` methods** to `TryThunk` and `TryTrampoline`.
5. **Add `catch_unwind` to `TryTrampoline`**.

### Medium Priority

6. **Investigate the `Send` bound** on `From<TryTrampoline> for RcTryLazy`.
7. **Add Foldable impl for ArcLazy**.
8. **Extract duplicate panic-to-string logic** into a shared helper.
9. **Resolve `pure`/`ok` inconsistency** on TryThunk (deprecate `pure` in favor of `ok`).
10. **Add inherent `lift2` and `then` to TryThunk** for ergonomic parity with TryTrampoline.
11. **Fix doc inaccuracies** in trampoline.rs.

### Low Priority / Future Consideration

12. Add a "choosing your lazy type" decision flowchart to docs.
13. Add `memoize_arc()` convenience methods.
14. Document `From` conversion costs.
15. Consider a `lazy` submodule grouping all six types.
16. Document why generic `fix` is not provided for `Deferrable` (PureScript's `fix` requires lazy self-reference only achievable with Rc/Arc in Rust).
17. Consider adding `map`/`map_err` to TryLazy (returning new TryLazy instances).
18. Add `From<TryTrampoline> for TryThunk` for bidirectional conversion.
19. Investigate whether Trampoline's `'static` requirement could be relaxed with a different Free monad encoding.

---

## Design Alternatives Considered and Rejected

| Alternative | Why Rejected |
|---|---|
| **Single `Eval` type (Scala-style)** | The `FlatMap` variant would force `'static` on all variants. Strictly worse than the current design in Rust. (Analysis 1) |
| **Combined Defer+Evaluate trait** | Return type differences (`A` vs `&A`) make unification awkward. (Analysis 6) |
| **Functor/Monad for Lazy** | Would require `A: Clone` bounds, adding hidden allocations that conflict with zero-cost philosophy. `RefFunctor` is the right compromise. (Analysis 4) |
| **Generic `fix` for all `Deferrable` types** | Requires lazy self-reference impossible without Rc/Arc in Rust. Concrete `rc_lazy_fix`/`arc_lazy_fix` are correct. (Analysis 6) |
| **TrampolineBrand for HKT integration** | The `'static` requirement makes the Kind trait's lifetime parameter vacuous. Would create a fragile abstraction. (Analysis 2) |

---

## Analysis Index

| # | Title | Focus |
|---|-------|-------|
| [1](1.md) | Overall Design Evaluation | Three-type split, FP literature comparison, trade-offs |
| [2](2.md) | HKT Integration and Type Class Correctness | Brands, Kind mappings, law compliance |
| [3](3.md) | Stack Safety Analysis | Free monad, trampoline loop, tail_rec_m correctness |
| [4](4.md) | Memoization Implementation | LazyCell/LazyLock, interior mutability, RefFunctor |
| [5](5.md) | Try Variant Design and Consistency | Code duplication, API gaps, error handling |
| [6](6.md) | Deferrable and Evaluable Trait Design | PureScript comparison, trait bounds, fix combinator |
| [7](7.md) | Lifetime Design and Ergonomics | `'a` vs `'static`, variance, composability |
| [8](8.md) | Pointer Abstraction and Thread Safety | Rc/Arc, LazyConfig, FnBrand, Send/Sync |
| [9](9.md) | Testing and Documentation Quality | Coverage gaps, doc accuracy, missing tests |
| [10](10.md) | API Ergonomics and Practical Usability | Discoverability, footguns, naming, performance |
