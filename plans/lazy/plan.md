# Implementation Plan: Lazy Evaluation Hierarchy Improvements

## Phase 1: Immediate value, low effort

### Task 1.1: Add `#[inline]` annotations to `Thunk` and `Lazy` trivial methods

- **What changes:** Add `#[inline]` attribute to wrapper methods that delegate directly to an inner operation or perform trivial logic.
- **Where:**
  - `fp-library/src/types/thunk.rs`: Methods `new`, `pure`, `defer`, `bind`, `map`, `evaluate`, `memoize`, `memoize_arc`.
  - `fp-library/src/types/lazy.rs`: Methods `evaluate`, `new`, `pure`, `ref_map` (both Rc and Arc variants).
  - Also consider `TryThunk`, `TryTrampoline`, and `TryLazy` methods that are thin wrappers.
- **Dependencies:** None.
- **Complexity:** Small.
- **Acceptance criteria:** All listed methods have `#[inline]` annotations. `cargo clippy --workspace --all-features` passes. `cargo test --workspace --all-features` passes.

### Task 1.2: Fix unnecessary clone in `Lazy::ref_map`

- **What changes:** In both `RcLazy::ref_map` and `ArcLazy::ref_map`, `self` is taken by value but then cloned before capture. The clone is unnecessary; `self` should be moved directly into the closure.
- **Where:**
  - `fp-library/src/types/lazy.rs` `RcLazy::ref_map`: Replace `let fa = self.clone(); ... move || f(fa.evaluate())` with `move || f(self.evaluate())`.
  - `fp-library/src/types/lazy.rs` `ArcLazy::ref_map`: Same change.
- **Dependencies:** None.
- **Complexity:** Small.
- **Acceptance criteria:** The methods no longer call `.clone()` on `self`. All existing tests pass. Doc examples still compile and pass.

### Task 1.3: Add `catch_unwind` for `ArcTryLazy`

- **What changes:** Add a `catch_unwind` associated function to the `impl<'a, A> TryLazy<'a, A, String, ArcLazyConfig>` block, mirroring the existing `RcTryLazy` implementation.
- **Where:**
  - `fp-library/src/types/try_lazy.rs`: Add a new impl block `impl<'a, A: 'a> TryLazy<'a, A, String, ArcLazyConfig>` with a `catch_unwind` method. The method signature should be `pub fn catch_unwind(f: impl FnOnce() -> A + std::panic::UnwindSafe + Send + 'a) -> Self` (note the additional `Send` bound required by `ArcLazyConfig`). The body is identical to the `RcLazyConfig` version: `Self::new(move || std::panic::catch_unwind(f).map_err(crate::utils::panic_payload_to_string))`.
- **Dependencies:** None.
- **Complexity:** Small.
- **Acceptance criteria:** `ArcTryLazy::<i32, String>::catch_unwind(|| 42)` compiles and evaluates to `Ok(&42)`. A panic closure evaluates to `Err(&"...")`. Add a unit test mirroring the existing `RcTryLazy` `test_catch_unwind` test.

### Task 1.4: Correct README wording about Thunk/Trampoline "re-running" evaluate

- **What changes:** The README says "they re-run every time you call `.evaluate()`." This is misleading since `Thunk::evaluate` takes `self` by value, so it can only be called once. The wording should clarify that these types are non-memoizing (each instance evaluates once, but constructing a new instance produces a fresh computation), contrasting with `Lazy` which shares a single evaluation across clones.
- **Where:**
  - `README.md` around line 193.
  - Also check `fp-library/src/lib.rs` around line 125 for the same wording.
- **Dependencies:** None.
- **Complexity:** Small.
- **Acceptance criteria:** The text accurately describes that `evaluate()` consumes `self`, meaning each instance evaluates once, but the result is not cached across separate instances. The contrast with `Lazy` (shared cache) is clear.

---

## Phase 2: Significant value, moderate effort

### Task 2.1: Add Criterion benchmarks for lazy types

- **What changes:** Create a new benchmark file covering the lazy type hierarchy. Benchmarks should cover:
  - `Thunk`: `new` + `evaluate`, `map` chains (1, 10, 100 deep), `bind` chains (1, 10, 100 deep).
  - `Trampoline`: `new` + `evaluate`, `bind` chains (100, 1000, 10000 deep), `tail_rec_m` (deep recursion), `map` chains.
  - `Lazy` (both `RcLazy` and `ArcLazy`): first-access time, cached-access time, `ref_map` chains.
  - `Free`: left-associated vs right-associated bind chains, `evaluate` for various depths.
  - Comparative benchmarks: `Trampoline` deep recursion vs hand-written iterative loop.
- **Where:**
  - New file: `fp-library/benches/benchmarks/lazy.rs`.
  - Modify `fp-library/benches/benchmarks.rs` to register the new `bench_lazy` group.
- **Dependencies:** None.
- **Complexity:** Medium.
- **Acceptance criteria:** `cargo bench -p fp-library --bench benchmarks -- Thunk` (and similar for Trampoline, Lazy, Free) produces benchmark output. Results are generated in `target/criterion/`. No panic or timeout during benchmark runs.

### Task 2.2: Add compile-fail tests for lazy types

- **What changes:** Add `trybuild` compile-fail test cases for common mistakes with lazy types. Specific test cases:
  1. Sending a `Thunk` across threads (Thunk is `!Send` because it contains `Box<dyn FnOnce()>`).
  2. Using borrowed (non-`'static`) data with `Trampoline`.
  3. Using non-`Send` closures with `ArcLazy::new`.
  4. Sending `RcLazy` across threads.
- **Where:**
  - New files in `fp-library/tests/ui/`:
    - `thunk_not_send.rs`
    - `trampoline_requires_static.rs`
    - `arc_lazy_requires_send.rs`
    - `rc_lazy_not_send.rs`
  - Corresponding `.stderr` files with expected error output.
  - The existing `fp-library/tests/compile_fail.rs` already loads `tests/ui/*.rs`, so no changes to the test runner are needed.
- **Dependencies:** None.
- **Complexity:** Medium.
- **Acceptance criteria:** `cargo test -p fp-library --test compile_fail` passes. Each UI test file fails to compile with a relevant error message about the specific constraint being violated (Send, 'static, etc.).

### Task 2.3: Fill test coverage gaps

- **What changes:** Add missing unit tests and property tests for recently added methods and uncovered functionality.
- **Where and what:**
  - `fp-library/src/types/try_thunk.rs` (tests module): Add tests for `lift2` (ok/ok, err/ok, ok/err), `then` (ok/ok, err/ok, ok/err), `memoize` (basic usage, caching verification), `memoize_arc` (basic usage, Send+Sync requirement).
  - `fp-library/src/types/try_lazy.rs` (tests module): Add tests for `RcTryLazy::map` (ok case, err case), `RcTryLazy::map_err` (ok case, err case), `ArcTryLazy::map` (ok case, err case), `ArcTryLazy::map_err` (ok case, err case).
  - Consider adding QuickCheck property tests for Bifunctor identity and composition laws for `TryThunkBrand`.
  - Consider adding Foldable law property tests for `LazyBrand<RcLazyConfig>` and `LazyBrand<ArcLazyConfig>`.
- **Dependencies:** None.
- **Complexity:** Medium.
- **Acceptance criteria:** All new tests pass under `cargo test --workspace --all-features`. Coverage for `lift2`, `then`, `memoize`, `memoize_arc` on TryThunk, and `map`/`map_err` on TryLazy (both Rc and Arc variants) is complete.

### Task 2.4: Expand comparison table in `lib.rs` and README to include Try* types

- **What changes:** The comparison tables only cover `Thunk`, `Trampoline`, and `Lazy`. Add rows for `TryThunk`, `TryTrampoline`, and `TryLazy` showing their fallible counterpart relationship, stack safety, memoization, lifetime, and HKT support properties.
- **Where:**
  - `fp-library/src/lib.rs` around lines 114-120.
  - `README.md` around lines 183-186.
- **Dependencies:** None.
- **Complexity:** Small.
- **Acceptance criteria:** Both tables include all six types. The rows for Try* types clearly indicate they wrap `Result<A, E>` and specify which base type they correspond to. `cargo doc --workspace --all-features --no-deps` builds without warnings.

### Task 2.5: Make Thunk's bind-chain stack overflow warning more prominent

- **What changes:** The current documentation on `Thunk::bind` has a brief note "Each `bind` adds to the call stack." This should be expanded with a dedicated "Stack Safety" section in the struct-level docs for `Thunk`, explaining:
  - `bind` chains are not stack-safe.
  - Deep `bind` chains will overflow the stack.
  - Use `tail_rec_m` for stack-safe recursion within `Thunk`.
  - For unlimited stack safety, convert to `Trampoline` instead.
- **Where:**
  - `fp-library/src/types/thunk.rs` struct-level docs: Add a "# Stack Safety" section after "### Algebraic Properties."
- **Dependencies:** None.
- **Complexity:** Small.
- **Acceptance criteria:** Struct-level docs for `Thunk` include a clear "Stack Safety" section. `cargo doc` renders it properly. The section links to `Trampoline` and `tail_rec_m`.

---

## Phase 3: Nice to have, higher effort

### Task 3.1: Add `resume` and `fold_free` to `Free`

- **What changes:** Add two methods to `Free`:
  - `resume`: Returns `Result<A, Apply!(F, Free<F, A>)>`, allowing step-by-step inspection of the computation. `Ok(a)` for `Pure(a)`, `Err(f_free)` for suspended computations.
  - `fold_free`: Takes a natural transformation `F ~> G` (where `G` is a monad) and folds the Free structure into `G<A>`. This is the key missing piece for using `Free` as a general-purpose AST interpreter.
- **Where:**
  - `fp-library/src/types/free.rs`: Add methods to `impl<F, A> Free<F, A>`.
- **Dependencies:** May need a `NaturalTransformation` trait or a way to express `F ~> G`. The `'static` constraint on `Free` limits the generality.
- **Complexity:** Large.
- **Acceptance criteria:** `resume` on a `Pure` returns `Ok(value)`; on a `Wrap` returns `Err(functor_layer)`. `fold_free` can interpret a `Free<ThunkBrand, A>` into a `Trampoline<A>` (or similar). Tests demonstrate step-by-step execution and custom interpretation.

### Task 3.2: Add a dedicated `Map` variant to `FreeInner`

- **What changes:** Currently, `Trampoline::map` delegates to `self.bind(move |a| Trampoline::pure(f(a)))`, which goes through the full type erasure path. A `Map` variant on `FreeInner` would allow direct transformation without type erasure overhead.
- **Where:**
  - `fp-library/src/types/free.rs`: Add `Map` variant to `FreeInner`. Update `evaluate` loop to handle `Map` directly. Update `Drop` implementation.
  - `fp-library/src/types/trampoline.rs`: Change `map` implementation to use the new variant.
- **Dependencies:** Should be done after Task 2.1 (benchmarks) so the optimization can be validated with actual performance data.
- **Complexity:** Medium-Large.
- **Acceptance criteria:** `Trampoline::map` chains of depth N do not create N type-erased continuations. Benchmarks show measurable improvement for map-heavy workloads. All existing tests pass.

### Task 3.3: Consider a `SendRefFunctor` trait

- **What changes:** Currently, `ArcLazy` provides `ref_map` as an inherent method because the `RefFunctor` trait does not require `Send` on the mapping function. A `SendRefFunctor` trait would mirror the `Deferrable`/`SendDeferrable` pattern, giving `ArcLazy` trait-level `ref_map`.
- **Where:**
  - New file: `fp-library/src/classes/send_ref_functor.rs`.
  - Modify `fp-library/src/classes.rs` to export the module.
  - Implement for `LazyBrand<ArcLazyConfig>` in `fp-library/src/types/lazy.rs`.
- **Dependencies:** None.
- **Complexity:** Medium.
- **Acceptance criteria:** `ArcLazy` implements `SendRefFunctor`. Generic code can use the trait to map over `ArcLazy` values. Existing `ref_map` inherent method can delegate to the trait. All tests pass.

### Task 3.4: Generalize `catch_unwind` to accept a conversion function

- **What changes:** Currently, `catch_unwind` on all Try* types hardcodes `E = String` by using `panic_payload_to_string`. A more general version would accept a `Box<dyn Any> -> E` conversion function, allowing users to choose their error representation.
- **Where:**
  - `fp-library/src/types/try_thunk.rs`
  - `fp-library/src/types/try_trampoline.rs`
  - `fp-library/src/types/try_lazy.rs`
- **Dependencies:** Task 1.3 should be done first so ArcTryLazy has `catch_unwind`.
- **Complexity:** Medium.
- **Acceptance criteria:** A new method `catch_unwind_with(f, convert)` exists on all Try* types. The existing `catch_unwind` (E = String) remains as a convenience wrapper that delegates to `catch_unwind_with` using `panic_payload_to_string`.

### Task 3.5: Add `Semigroup`/`Monoid` as inherent methods on `Trampoline`

- **What changes:** `Trampoline` already has `lift2`, so `append` for `Semigroup<A>` can delegate to `lift2(other, Semigroup::append)`. Similarly, `empty` can wrap `Monoid::empty()` in `Trampoline::pure`.
- **Where:**
  - `fp-library/src/types/trampoline.rs`: Add `append` and `empty` methods gated on `A: Semigroup` and `A: Monoid` bounds.
- **Dependencies:** None.
- **Complexity:** Small.
- **Acceptance criteria:** `Trampoline::pure(vec![1]).append(Trampoline::pure(vec![2])).evaluate() == vec![1, 2]`. Tests demonstrate both `append` and `empty`.

### Task 3.6: Consider adding `Traversable` for `Lazy` (with `A: Clone` bound)

- **What changes:** PureScript's `Data.Lazy` supports `Traversable`. In Rust, this requires `A: Clone` since `evaluate()` returns `&A` and `Traversable` needs owned values.
- **Where:**
  - `fp-library/src/types/lazy.rs`: Add `impl Traversable for LazyBrand<RcLazyConfig>` (and potentially for `ArcLazyConfig`).
- **Dependencies:** Need to verify that the `Clone` bound is acceptable for the intended use cases.
- **Complexity:** Medium.
- **Acceptance criteria:** `traverse` over a `Lazy` value compiles and produces expected results. Traversable laws (identity, composition) hold as verified by property tests.

### Task 3.7: Link `LazyConfig` to `RefCountedPointer` hierarchy

- **What changes:** Add an associated type to `LazyConfig` that links to the corresponding `RefCountedPointer` brand. This would enable generic code that composes lazy evaluation with pointer-parameterized abstractions.
- **Where:**
  - `fp-library/src/types/lazy.rs`: Add `type PointerBrand: RefCountedPointer;` to `LazyConfig` trait, with `RcLazyConfig::PointerBrand = RcBrand` and `ArcLazyConfig::PointerBrand = ArcBrand`.
- **Dependencies:** Requires understanding how `RefCountedPointer` is used in the optics system and elsewhere.
- **Complexity:** Medium.
- **Acceptance criteria:** Generic code can write `fn foo<C: LazyConfig>() where C::PointerBrand: SendRefCountedPointer` to constrain to thread-safe lazy configs. No breaking changes to existing code.

---

## Dependency Graph

```
Phase 1 tasks are all independent of each other.

Phase 2 tasks are all independent of each other,
  but Phase 2 generally benefits from Phase 1 being complete.

Phase 3 dependencies:
  3.1 (resume/fold_free)   <- independent
  3.2 (Map variant)        <- 2.1 (benchmarks, to validate)
  3.3 (SendRefFunctor)     <- independent
  3.4 (generalize catch)   <- 1.3 (ArcTryLazy catch_unwind)
  3.5 (Semigroup/Monoid)   <- independent
  3.6 (Traversable Lazy)   <- independent
  3.7 (LazyConfig pointer) <- independent
```

## Recommended Execution Order

1. All Phase 1 tasks in parallel (1.1, 1.2, 1.3, 1.4).
2. Phase 2 tasks: start with 2.1 (benchmarks, needed by 3.2), then 2.2, 2.3, 2.4, 2.5 in any order.
3. Phase 3 tasks can be tackled individually as time permits. Prioritize 3.2 (Map variant, depends on benchmarks being available) and 3.4 (generalize catch_unwind, depends on 1.3).

## Critical Files

| File | Relevance |
|------|-----------|
| `fp-library/src/types/thunk.rs` | `#[inline]` annotations, stack safety docs |
| `fp-library/src/types/lazy.rs` | `#[inline]` annotations, `ref_map` clone fix, `LazyConfig` trait |
| `fp-library/src/types/try_lazy.rs` | Missing `ArcTryLazy::catch_unwind`, `map`/`map_err` tests |
| `fp-library/src/types/try_thunk.rs` | `lift2`/`then`/`memoize` tests |
| `fp-library/src/types/trampoline.rs` | Semigroup/Monoid methods |
| `fp-library/src/types/free.rs` | `resume`/`fold_free`, `Map` variant |
| `fp-library/benches/benchmarks.rs` | Benchmark registration |
| `README.md` | Wording fix, comparison table |
| `fp-library/src/lib.rs` | Comparison table |
