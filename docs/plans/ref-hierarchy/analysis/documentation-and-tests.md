# Documentation and Tests Analysis

## 1. Documentation Updates

### 1.1 features.md

**Diff:** Adds comprehensive listing of the Ref, SendRef, and ParRef trait
families. Removes the old "RefFunctor, SendRefFunctor for mapping over memoized
types" text and replaces it with the full hierarchy.

**Assessment:** Well-structured. The new listing is organized into four clear
groups: by-reference core, thread-safe by-reference, parallel by-reference, and
laziness/effects.

**Issue:** The old text referenced `TryLazyConfig` in the laziness section; the
new text drops this. If `TryLazyConfig` still exists in the codebase, its removal
from the docs may be premature. However, this is a minor point; `TryLazyConfig`
is an implementation detail, and the features doc focuses on trait-level features.

**Issue:** The `ref` qualifier for `m_do!`/`a_do!` is mentioned in the opening
paragraph, which is good. However, there is no example showing the syntax. The
features doc may benefit from a short code snippet.

### 1.2 parallelism.md

**Diff:** Renames `SendCloneableFn` to `SendCloneFn`. Adds a new "Parallel
By-Reference Traits" subsection with a Mermaid diagram and table listing all
six `ParRef` traits and their supertraits.

**Assessment:** The new section is thorough. The table clearly lists operations
and supertraits. The key benefit ("avoids consuming elements that get filtered
out") is well-stated.

**No issues found.** The `par_map` and `par_fold_map` example code in the
existing code block uses `par_map::<VecBrand, _, _>` with 3 type params, which
is correct since `par_map` does not go through the dispatch system (it has no
`Marker` parameter).

### 1.3 pointer-abstraction.md

**Diff:** Renames `CloneableFn` to `CloneFn`, `SendCloneableFn` to `SendCloneFn`.
Adds explanation of `ClosureMode` parameterization (`Val`/`Ref`) and mentions
the `Arrow` variant for optics.

**Assessment:** Accurate and concise. Correctly describes the dual-mode
parameterization.

### 1.4 limitations-and-workarounds.md

**Diff:** Major update to the "Memoized Types Cannot Implement Functor" section,
expanding it from the old `RefFunctor`/`SendRefFunctor` description to the full
by-reference hierarchy. Renames all trait references from old names to new names
(`CloneableFn` -> `CloneFn`, `SendCloneableFn` -> `SendCloneFn`,
`Function` -> `Arrow`).

**Stale reference found (line 16):** The "No method syntax" bullet point still
reads `bind::<OptionBrand, _, _>(f, x)` with 3 type params. This should be
`bind::<OptionBrand, _, _, _>(f, x)` with 4 type params, matching the new
dispatched `bind` signature. The preceding bullet about turbofish was correctly
updated to `map::<OptionBrand, _, _, _>`.

**Assessment:** The expanded Ref hierarchy section is comprehensive. The trait
rename updates are thorough within this file. The stale `bind` turbofish is a
minor oversight.

### 1.5 CLAUDE.md

**Diff:** Two changes:

1. Updates the test cache description from timestamp-based to content-hashing
   (`git ls-files` + `md5sum`). Adds mention of `just clean`.
2. Renames `SendCloneableFn` to `SendCloneFn` in the thread-safe operations
   section.

**Assessment:** Both changes are accurate and align with the codebase state.

**Missing update:** CLAUDE.md still does not mention the `ref` qualifier for
`m_do!`/`a_do!` macros. Since CLAUDE.md is the primary guidance file for AI
assistants working on the codebase, adding a brief note about ref mode
dispatch would be beneficial. At minimum, the "Common Patterns" section could
mention that `m_do!(ref Brand { ... })` exists for by-reference dispatch.

### 1.6 README.md

**Diff:** Updates the example `map` call to use 4 type params
(`map::<OptionBrand, _, _, _>`). Updates test cache description. Adds `just clean`
recipe. Updates the LHKP paper link to an archive.org URL. Adds `haskell_bits`
reference.

**Assessment:** All changes are accurate. The archive.org link is a good
resilience measure against link rot.

### 1.7 docs/todo.md

**Diff:** Adds a "Deferred Ref-hierarchy items" section listing:

- SendRef variants for filterable/traversable/witherable
- Ref impls for collection types (noted as step 22 in plan)
- Par-Ref traits
- Dispatch unification for filterable/traversable/witherable
- RefBifunctor, RefBifoldable, RefBitraversable

Also adds a new idea about composing kinds from nested curried applications.

**Issue:** The todo says "Ref impls for collection types" is deferred, but step
22 in the plan is marked as done. This may be a stale entry if Vec, Option,
CatList, and Identity already implement the Ref traits. Or it may refer to
additional collection types not yet covered. The todo should be clarified.

**Assessment:** The deferred items list is a useful tracking mechanism. The
potential staleness of the collection types item should be investigated.

## 2. Test Coverage

### 2.1 Macro Integration Tests (m_do!/a_do! ref mode)

Tests are located in `fp-library/src/types/lazy.rs` (unit tests, not in the
`tests/` directory):

1. **m_do_ref_lazy_manual** -- Manual expansion of what `m_do!(ref ...)` should
   generate. Verifies the raw `bind` + `ref_pure` pattern.
2. **m_do_ref_lazy_macro** -- Single bind with typed annotation `a: &i32`.
3. **m_do_ref_lazy_multi_bind** -- Two binds with clone workaround for the
   multi-bind limitation.
4. **m_do_ref_lazy_untyped** -- Single bind without type annotation (uses `&_`
   inference).
5. **a_do_ref_lazy** -- Applicative do with two typed binds.

**Missing test scenarios:**

- **a_do! ref mode untyped** -- No test for `a_do!(ref Brand { a <- expr; ... })`
  without type annotations. Should verify `&_` inference works in the applicative
  case.
- **a_do! ref mode with Sequence** -- No test for sequence statements in ref
  mode applicative notation.
- **m_do! ref mode with Sequence** -- No test for `expr;` (discard) statements
  in ref mode monadic notation.
- **a_do! ref mode zero-bind** -- No test for `a_do!(ref Brand { final_expr })`
  which should produce `ref_pure::<Brand, _>(&(final_expr))`.
- **m_do! ref mode with let-only** -- No test verifying that `let` bindings in
  ref mode are not affected by the `&_` annotation.
- **Collection types in ref mode** -- No tests using `m_do!(ref VecBrand { ... })`
  or `m_do!(ref OptionBrand { ... })` to verify ref mode works with collection
  types, not just Lazy.

### 2.2 Parser Tests

Two new parser tests in `fp-macros/src/m_do/input.rs`:

- `parse_ref_mode` -- Verifies `ref OptionBrand { ... }` sets `ref_mode = true`.
- `parse_non_ref_mode` -- Verifies absence of `ref` sets `ref_mode = false`.

**Assessment:** Adequate for parser-level testing. No edge cases missing at
this level.

### 2.3 do_notation.rs Integration Tests

**Diff:** Two changes:

1. Manual bind calls updated from `bind::<OptionBrand, _, _>` to
   `bind::<OptionBrand, _, _, _>` (4 type params).
2. The `complex_expressions_in_bind` test changed from untyped
   `x <- Some(vec![1, 2, 3])` to typed `x: Vec<i32> <- Some(vec![1, 2, 3])`.

**Assessment:** The turbofish update is mechanical and correct. The typed bind
change may have been needed because the dispatch `Marker` inference requires
more type information, or it may have been a style preference. Either way, it
is correct.

**Missing:** No new ref-mode tests were added to `do_notation.rs`. All ref-mode
tests are in `lazy.rs`. Having at least one ref-mode test in the `do_notation.rs`
integration test file would improve discoverability.

### 2.4 ado_notation.rs Integration Tests

**Diff:** Manual combinator calls updated to 4 type params:

- `lift2::<OptionBrand, _, _, _>` -> `lift2::<OptionBrand, _, _, _, _>`
- `map::<OptionBrand, _, _>` -> `map::<OptionBrand, _, _, _>`

**Assessment:** Mechanical update, correct. No ref-mode tests added here either.

### 2.5 hkt_integration.rs

**Diff:** The `test_memo_ref_functor` test was updated to use the unified `map`
function with 4 type params instead of the old `ref_map` free function:

- `ref_map::<LazyBrand<RcLazyConfig>, _, _>(|x: &i32| ...)` ->
  `map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| ...)`

**Assessment:** Correct. Demonstrates that the dispatch system correctly routes
`|x: &i32|` closures to `RefFunctor::ref_map`.

### 2.6 optics_test.rs

**Diff:** All `cloneable_fn_new` calls renamed to `lift_fn_new`. Import changed
from specific profunctor imports to wildcard `profunctor::*`.

**Assessment:** Mechanical rename, correct. No functional changes.

### 2.7 pointer_integration.rs

**Diff:** `send_cloneable_fn_new` renamed to `send_lift_fn_new`. Doc comments
updated.

**Assessment:** Mechanical rename, correct.

### 2.8 thread_safety.rs

**Diff:** `send_cloneable_fn_new` renamed to `send_lift_fn_new`.

**Assessment:** Mechanical rename, correct.

### 2.9 Property-Based Tests (Ref Trait Laws)

According to the plan (step 26), quickcheck property tests were added for Ref
trait laws. These are located in the source files, not in the `tests/` directory:

- **Vec:** 6 tests (RefFunctor identity/composition, RefFoldable additive,
  RefSemimonad left identity/associativity, ParRefFunctor equivalence)
- **Option:** 4 tests (RefFunctor identity/composition, RefSemimonad left
  identity, RefFoldable fold_map)
- **CatList:** 5 tests (RefFunctor identity/composition, RefFoldable additive,
  RefSemimonad left identity, ParRefFunctor equivalence)
- **Identity:** 3 tests (RefFunctor identity/composition, RefSemimonad left
  identity)
- **Lazy:** 5 existing unit tests + TryLazy: 12 existing unit tests

**Missing law tests:**

- **RefApplicative laws** -- No property tests for applicative identity, composition,
  homomorphism, or interchange laws for the Ref hierarchy. These are important for
  verifying that `RefPointed` + `RefSemiapplicative` form a lawful applicative.
- **RefMonad laws** -- Only left identity is tested (via RefSemimonad). Right identity
  (`bind(pure(a), f) == f(a)`) and associativity at the monad level are not tested
  for the Ref variants.
- **RefTraversable laws** -- No property tests for traversal identity, composition,
  or naturality for the Ref hierarchy.
- **RefLift laws** -- No property tests verifying that `ref_lift2` satisfies the
  same laws as `lift2` (e.g., `ref_lift2(|a, b| a, fa, fb) == ref_map(id, fa)`).
- **SendRef variants** -- No property tests for any `SendRef*` traits. The ArcLazy
  impls have unit tests but no law verification.

### 2.10 Compile-Fail (UI) Tests

**Files changed:** 7 files across 4 test scenarios:

1. **arc_coyoneda_requires_send.stderr** -- Updated expected error messages to
   reflect new `Apply!`/`Kind!` macro expansion in error output (replacing raw
   `Kind_cdc7cd43dac7585f` names). This is a cosmetic change in error formatting.

2. **new_send_not_send.rs/.stderr** -- Updated to use `SendLiftFn` instead of
   `SendCloneableFn`. **Problem:** The `.rs` file uses `SendLiftFn` directly in
   the code but imports `SendCloneFn` (the trait) instead. The expected error
   changed from the original "Rc cannot be sent" error to a simpler "cannot find
   trait SendLiftFn in this scope" error. This means the test is no longer
   testing what it originally intended (that non-Send closures are rejected).
   Instead, it tests that a missing import is caught, which is a much weaker
   assertion.

3. **new_send_not_sync.rs/.stderr** -- Same issue as above: changed to use
   `SendLiftFn` but imports `SendCloneFn`, producing a "not found in this scope"
   error instead of the intended "RefCell cannot be shared between threads" error.

4. **rc_fn_not_send.rs/.stderr** -- Same pattern: uses `SendLiftFn` but imports
   `SendCloneFn`, producing "not found in this scope" instead of the intended
   "RcBrand does not implement SendUnsizedCoercible" error.

**Critical issue:** Three UI tests are broken in their intent. They import
`SendCloneFn` but use `SendLiftFn` in the code, causing the tests to pass for
the wrong reason (name resolution failure instead of trait bound violation).
The `SendLiftFn` trait needs to be properly imported for these tests to verify
their original purpose. This looks like an incomplete rename: the `.rs` files
were partially updated (code changed but import not matching), and the `.stderr`
files were updated to match the new (wrong) errors.

## 3. Benchmark Coverage

### 3.1 Existing Benchmark Updates

All benchmark files were mechanically updated:

- Turbofish annotations changed from 3 to 4 type params for dispatched functions
  (`map`, `bind`, `lift2`, `fold_right`, `fold_left`, `fold_map`).
- `cloneable_fn_new` renamed to `lift_fn_new`.
- Import blocks simplified from specific imports to `brands::*` and `functions::*`.

**Assessment:** All updates are mechanical and correct. No functional changes to
benchmark logic.

### 3.2 Missing Benchmark Coverage

The benchmarks do not include any Ref-mode operations:

- **No `ref_map` benchmark via dispatch.** The lazy.rs benchmarks have
  `ref_map` chain benchmarks using the method syntax (`lazy.ref_map(...)`), but
  no benchmarks for the dispatched `map::<LazyBrand, _, _, _>(|x: &i32| ...)`.
  This would verify that dispatch overhead is zero.
- **No `ref_bind` benchmark.** No benchmark for `bind` with ref dispatch
  (`bind::<LazyBrand, _, _, _>(lazy, |x: &i32| ...)`).
- **No `ref_lift2` benchmark.** No benchmark for `lift2` with ref dispatch.
- **No collection ref benchmarks.** Now that Vec, Option, CatList implement
  Ref traits, benchmarks comparing `map::<VecBrand, _, _, _>(|x: i32| ...)` vs
  `map::<VecBrand, _, _, _>(|x: &i32| ...)` would verify that ref dispatch
  has no overhead for collections.
- **No ParRef benchmarks.** No benchmarks for `par_ref_map`, `par_ref_fold_map`,
  or `par_ref_filter_map`.

## 4. Stale References Summary

| Location                                 | Issue             | Details                                                                      |
| ---------------------------------------- | ----------------- | ---------------------------------------------------------------------------- |
| `limitations-and-workarounds.md` line 16 | Stale turbofish   | `bind::<OptionBrand, _, _>` should be `bind::<OptionBrand, _, _, _>`         |
| `ref_functor.rs` lines 80-85             | Stale doc comment | "Why `FnOnce`?" section references `FnOnce` but the signature now uses `Fn`  |
| `todo.md`                                | Possibly stale    | "Ref impls for collection types" listed as deferred but plan step 22 is done |
| UI tests (3 files)                       | Broken intent     | `SendLiftFn` used but not imported; tests pass for wrong reason              |

## 5. Summary and Recommendations

### What is working well

- Documentation for the new Ref hierarchy is comprehensive across features.md,
  parallelism.md, pointer-abstraction.md, and limitations-and-workarounds.md.
- Trait rename updates (CloneableFn -> CloneFn, etc.) are consistently applied
  across documentation and test files.
- Turbofish updates for the new Marker type parameter are consistently applied.
- Property-based tests cover the core Ref trait laws for all four collection
  types plus Identity and Lazy.
- The plan is detailed and accurately reflects the implementation state.

### What needs attention

1. **Fix UI tests (high priority).** The `new_send_not_send`, `new_send_not_sync`,
   and `rc_fn_not_send` UI tests import `SendCloneFn` but use `SendLiftFn` in
   the test code. Fix the imports to use `SendLiftFn` so the tests verify trait
   bound violations rather than name resolution errors.

2. **Fix stale turbofish in limitations-and-workarounds.md.** Change
   `bind::<OptionBrand, _, _>(f, x)` to `bind::<OptionBrand, _, _, _>(f, x)`
   on line 16.

3. **Fix stale "Why FnOnce?" doc comment in ref_functor.rs.** Update to "Why Fn?"
   and explain that `Fn` is needed for types like Vec that call the closure
   per element.

4. **Add missing macro ref-mode tests.** At minimum: untyped `a_do!` ref mode,
   sequence in ref mode, zero-bind ref mode, and collection types in ref mode.

5. **Add RefApplicative/RefMonad law property tests.** Right identity, monad
   associativity, and applicative laws should be tested for the Ref hierarchy.

6. **Add dispatch benchmarks for Ref operations.** Verify zero overhead for
   `map`/`bind`/`lift2` ref dispatch.

7. **Clarify todo.md.** Remove or update the "Ref impls for collection types"
   entry if step 22 is complete, or specify which collection types remain.

8. **Document multi-bind limitation in m_do! docs.** Add a note in the macro
   doc comment about the reference capture limitation and the `a_do!` alternative.
