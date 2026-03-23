# Lazy Hierarchy: Implementation Plan

This plan addresses the issues identified across Agents 1 through 10 in the lazy hierarchy analysis. It is organized into independently shippable phases, ordered by priority and dependency relationships. Each task includes specific implementation guidance grounded in the actual codebase patterns.

## Phase Ordering Rationale

1. **Phase 1 (TryTrampoline completions)** comes first because the missing `tail_rec_m` undermines the core value proposition of stack-safe fallible recursion, and the delegation pattern is already established in the file; these are straightforward additions with no cascading effects.
2. **Phase 2 (Lazy type class enrichment)** follows because it adds missing capabilities (`RefFunctor` for `ArcLazy`, `Semigroup`, `Monoid`, `Foldable`, standard trait impls) that bring `Lazy` closer to parity with `Thunk` and PureScript's `Data.Lazy`. No external dependencies.
3. **Phase 3 (Conversion graph completions)** fills gaps in the `From` conversion web. These depend on understanding the final shape of the types from Phases 1 and 2.
4. **Phase 4 (TryThunk completions)** adds `Bifoldable`, `MonadRec` for the error channel, and convenience constructors. Slightly lower priority; no external blockers.
5. **Phase 5 (Property-based tests)** adds QuickCheck law tests for `Thunk`, `Trampoline`, and `Lazy`. Placed after implementation phases so tests cover all new functionality.
6. **Phase 6 (Trampoline Send audit)** is a potentially breaking change that requires careful analysis of downstream effects. Placed late because it needs the most scrutiny.
7. **Phase 7 (fix combinator)** is a self-contained addition of PureScript's `fix` for `RcLazy`/`ArcLazy`. Requires careful self-referential design; placed after the core enrichment is done.
8. **Phase 8 (Documentation and polish)** collects all documentation improvements, `Debug` impls, and small ergonomic additions. No code-level dependencies; can be done at any point but is lowest priority.

---

## Phase 1: TryTrampoline API Completions

**Goal:** Complete the `TryTrampoline` API surface to match `Trampoline`'s capabilities.

**Rationale:** Per Agent 7's finding, `TryTrampoline` lacks `tail_rec_m`, `lift2`, and `then`. The absence of `tail_rec_m` is the most significant gap since the entire purpose of `Trampoline` is stack-safe recursion. These are straightforward delegations following the existing pattern in `try_trampoline.rs`.

### Tasks

- [ ] **1.1: Add `tail_rec_m` to `TryTrampoline`**
  - **Description:** Implement stack-safe tail recursion for fallible computations. The step function returns `TryTrampoline<Step<S, A>, E>`, and the loop short-circuits on error.
  - **Files:** `fp-library/src/types/try_trampoline.rs`
  - **Implementation:** Add to the `impl<A: 'static + Send, E: 'static + Send> TryTrampoline<A, E>` block. Delegate to `Trampoline::tail_rec_m` by mapping the step result through `Result`:
    ```rust
    pub fn tail_rec_m<S: 'static + Send>(
        f: impl Fn(S) -> TryTrampoline<Step<S, A>, E> + Clone + 'static,
        initial: S,
    ) -> Self {
        TryTrampoline(Trampoline::tail_rec_m(
            move |state: Result<S, E>| match state {
                Err(e) => Trampoline::pure(Step::Done(Err(e))),
                Ok(s) => f(s).0.map(|result| match result {
                    Ok(Step::Loop(next)) => Step::Loop(Ok(next)),
                    Ok(Step::Done(a)) => Step::Done(Ok(a)),
                    Err(e) => Step::Done(Err(e)),
                }),
            },
            Ok(initial),
        ))
    }
    ```
  - **Complexity:** Medium.
  - **Dependencies:** None.
  - **Tests:** Unit test with a recursive factorial that validates both the success path and an error short-circuit path. Add a stack safety test with 100,000+ iterations.

- [ ] **1.2: Add `arc_tail_rec_m` to `TryTrampoline`**
  - **Description:** Arc-wrapped variant for non-Clone closures, following `Trampoline::arc_tail_rec_m`.
  - **Files:** `fp-library/src/types/try_trampoline.rs`
  - **Implementation:** Wrap the step function in `Arc`, then delegate to `tail_rec_m`. Follow the exact pattern from `trampoline.rs` lines 422-433.
  - **Complexity:** Small.
  - **Dependencies:** Task 1.1.
  - **Tests:** Unit test with a non-Clone closure (capturing `Arc<AtomicUsize>`).

- [ ] **1.3: Add `lift2` to `TryTrampoline`**
  - **Description:** Combines two `TryTrampoline`s, running both and combining results (short-circuiting on error).
  - **Files:** `fp-library/src/types/try_trampoline.rs`
  - **Implementation:** Delegate through `bind` and `map`, matching `Trampoline::lift2`:
    ```rust
    pub fn lift2<B: 'static + Send, C: 'static + Send>(
        self,
        other: TryTrampoline<B, E>,
        f: impl FnOnce(A, B) -> C + 'static,
    ) -> TryTrampoline<C, E> {
        self.bind(move |a| other.map(move |b| f(a, b)))
    }
    ```
  - **Complexity:** Small.
  - **Dependencies:** None.
  - **Tests:** Test combining two successful values, and test that an error in either side short-circuits.

- [ ] **1.4: Add `then` to `TryTrampoline`**
  - **Description:** Sequences two `TryTrampoline`s, discarding the first result (short-circuiting on error).
  - **Files:** `fp-library/src/types/try_trampoline.rs`
  - **Implementation:** `self.bind(move |_| other)`, matching `Trampoline::then`.
  - **Complexity:** Small.
  - **Dependencies:** None.
  - **Tests:** Test that the first value is discarded and the second is returned; test error short-circuit.

---

## Phase 2: Lazy Type Class Enrichment

**Goal:** Add missing type class instances and standard trait implementations for `Lazy`.

**Rationale:** Per Agents 1, 2, and 5, `Lazy` has a much narrower type class coverage than PureScript's `Data.Lazy` and even than this library's own `Thunk`. The items below are the low-hanging fruit that can be added with appropriate bounds.

### Tasks

- [ ] **2.1: Add `RefFunctor` for `LazyBrand<ArcLazyConfig>` and inherent `ref_map` on `ArcLazy`**
  - **Description:** The thread-safe `ArcLazy` variant currently cannot be ref-mapped. Add both the inherent method and the HKT trait impl.
  - **Files:** `fp-library/src/types/lazy.rs`
  - **Implementation:** Add an inherent `ref_map` method to the `impl<'a, A> Lazy<'a, A, ArcLazyConfig>` block (near line 576). The closure must be `Send` because `ArcLazy::new` requires it. Then add the `RefFunctor` impl for `LazyBrand<ArcLazyConfig>` near the existing `RefFunctor` impl for `RcLazyConfig` (line 705). The `RefFunctor` trait's `ref_map` signature takes `impl FnOnce(&A) -> B + 'a` without `Send`, but `ArcLazy::new` requires `Send` on the closure. To resolve this, the `RefFunctor` impl must add `A: Send + Sync` and `B: Send + Sync` bounds, or the `ref_map` call must construct the closure to satisfy `Send`. Since the closure captures `fa` (an `ArcLazy`, which is `Send`) and `f`, and since `f` needs to be `Send` for `ArcLazy::new`, but the trait does not require `Send` on `f`, this impl **cannot** be fully general. Instead, add only the inherent `ref_map` method with explicit `Send` bounds, and skip the `RefFunctor` trait impl unless the trait signature is changed. Document why.
    ```rust
    pub fn ref_map<B: Send + 'a>(
        self,
        f: impl FnOnce(&A) -> B + Send + 'a,
    ) -> Lazy<'a, B, ArcLazyConfig>
    where
        A: Send + Sync,
    {
        let fa = self.clone();
        ArcLazy::new(move || f(fa.evaluate()))
    }
    ```
  - **Complexity:** Medium (requires careful analysis of trait bounds vs. inherent method).
  - **Dependencies:** None.
  - **Tests:** Test that `ArcLazy::new(|| 10).ref_map(|x| *x * 2)` evaluates to 20. Test thread safety of the mapped value.

- [ ] **2.2: Add `Semigroup` for `Lazy` (both `RcLazy` and `ArcLazy`)**
  - **Description:** Per Agents 1, 2, and 5, `Lazy` should support `Semigroup` when the inner type does. Mirrors the existing `Thunk` implementation.
  - **Files:** `fp-library/src/types/lazy.rs`
  - **Implementation:** Follow the pattern from `thunk.rs` (line 704). The implementation requires `A: Clone + Semigroup` because `evaluate()` returns `&A`:
    ```rust
    impl<'a, A: Semigroup + Clone + 'a> Semigroup for Lazy<'a, A, RcLazyConfig> {
        fn append(a: Self, b: Self) -> Self {
            RcLazy::new(move || Semigroup::append(a.evaluate().clone(), b.evaluate().clone()))
        }
    }
    ```
    Add analogous impl for `ArcLazyConfig` with `A: Send + Sync + Clone + Semigroup + 'a`.
  - **Complexity:** Small.
  - **Dependencies:** None.
  - **Tests:** Test appending two `RcLazy<String>` values; test associativity via QuickCheck in Phase 5.

- [ ] **2.3: Add `Monoid` for `Lazy` (both `RcLazy` and `ArcLazy`)**
  - **Description:** `Monoid::empty()` creates a `Lazy` whose value is the inner type's empty value.
  - **Files:** `fp-library/src/types/lazy.rs`
  - **Implementation:**
    ```rust
    impl<'a, A: Monoid + Clone + 'a> Monoid for Lazy<'a, A, RcLazyConfig> {
        fn empty() -> Self {
            RcLazy::new(|| Monoid::empty())
        }
    }
    ```
    Analogous impl for `ArcLazyConfig` with `A: Send + Sync + Monoid + Clone + 'a`.
  - **Complexity:** Small.
  - **Dependencies:** Task 2.2.
  - **Tests:** Test identity laws: `append(empty(), lazy)` and `append(lazy, empty())` both equal `lazy`.

- [ ] **2.4: Add `Foldable` for `LazyBrand<RcLazyConfig>`**
  - **Description:** Per Agents 1, 2, and 5, `Lazy` is a single-element container and should support `Foldable`. Mirrors `ThunkBrand`'s `Foldable` impl.
  - **Files:** `fp-library/src/types/lazy.rs`
  - **Implementation:** Follow the `Foldable for ThunkBrand` pattern (thunk.rs line 584). The key difference: `Lazy::evaluate()` returns `&A`, so the fold function receives a cloned value (requiring `A: Clone`). Look at the exact `Foldable` trait signature (uses `CloneableFn` for the fold function) and replicate:
    ```rust
    impl Foldable for LazyBrand<RcLazyConfig> {
        fn fold_right<'a, FnBrand: 'a + CloneableFn, A: 'a, B: 'a>(
            f: <FnBrand as Kind!(...)>::Of<'a, A, B>,
            initial: B,
            fa: Lazy<'a, A, RcLazyConfig>,
        ) -> B
        where
            A: Clone,
        {
            f(fa.evaluate().clone(), initial)
        }
    }
    ```
    Note: The `Foldable` trait's `fold_right` does not require `A: Clone` in its signature. The `LazyBrand` impl must add this bound. Check whether the trait's GAT signature allows this; if not, use the inherent method pattern instead.
  - **Complexity:** Medium (depends on whether the `Foldable` trait allows additional bounds).
  - **Dependencies:** None.
  - **Tests:** Test folding a `Lazy<i32>` with addition.

- [ ] **2.5: Add `PartialEq` for `Lazy`**
  - **Description:** Delegate equality comparison to the cached value. Per Agents 1 and 5.
  - **Files:** `fp-library/src/types/lazy.rs`
  - **Implementation:**
    ```rust
    impl<'a, A: PartialEq + 'a, Config: LazyConfig> PartialEq for Lazy<'a, A, Config> {
        fn eq(&self, other: &Self) -> bool {
            self.evaluate() == other.evaluate()
        }
    }
    ```
    Note: This forces evaluation of both sides. Document this behavior.
  - **Complexity:** Small.
  - **Dependencies:** None.
  - **Tests:** Test equality of two `RcLazy` values with same and different contents.

- [ ] **2.6: Add `PartialOrd` for `Lazy`**
  - **Description:** Delegate ordering comparison to the cached value.
  - **Files:** `fp-library/src/types/lazy.rs`
  - **Implementation:**
    ```rust
    impl<'a, A: PartialOrd + 'a, Config: LazyConfig> PartialOrd for Lazy<'a, A, Config> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.evaluate().partial_cmp(other.evaluate())
        }
    }
    ```
  - **Complexity:** Small.
  - **Dependencies:** Task 2.5.
  - **Tests:** Test ordering of `RcLazy<i32>` values.

---

## Phase 3: Conversion Graph Completions

**Goal:** Fill the remaining gaps in the `From` conversion web between lazy types.

**Rationale:** Per Agents 4 and 7, several natural conversions are missing. These enable users to upgrade computations when their requirements change (e.g., thunk to trampoline for stack safety).

### Tasks

- [ ] **3.1: Add `From<Thunk<'static, A>> for Trampoline<A>`**
  - **Description:** Per Agent 4, allows migrating from Thunk to Trampoline when stack safety becomes a concern.
  - **Files:** `fp-library/src/types/trampoline.rs`
  - **Implementation:**
    ```rust
    impl<A: 'static + Send> From<Thunk<'static, A>> for Trampoline<A> {
        fn from(thunk: Thunk<'static, A>) -> Self {
            Trampoline::new(move || thunk.evaluate())
        }
    }
    ```
    Add appropriate doc attributes following the existing pattern (e.g., `From<Lazy>` at line 440).
  - **Complexity:** Small.
  - **Dependencies:** None.
  - **Tests:** Test roundtrip: `Thunk::pure(42)` -> `Trampoline` -> evaluate == 42.

- [ ] **3.2: Add `From<Trampoline<A>> for Thunk<'static, A>`**
  - **Description:** Per Agent 4, enables using Trampoline results in HKT-generic code expecting a Thunk.
  - **Files:** `fp-library/src/types/thunk.rs`
  - **Implementation:**
    ```rust
    impl<A: 'static + Send> From<Trampoline<A>> for Thunk<'static, A> {
        fn from(trampoline: Trampoline<A>) -> Self {
            Thunk::new(move || trampoline.evaluate())
        }
    }
    ```
  - **Complexity:** Small.
  - **Dependencies:** None.
  - **Tests:** Test roundtrip: `Trampoline::pure(42)` -> `Thunk` -> evaluate == 42.

- [ ] **3.3: Add `From<TryThunk<'static, A, E>> for TryTrampoline<A, E>`**
  - **Description:** Per Agent 7, allows upgrading a fallible thunk to a stack-safe fallible computation.
  - **Files:** `fp-library/src/types/try_trampoline.rs`
  - **Implementation:**
    ```rust
    impl<A: 'static + Send, E: 'static + Send> From<TryThunk<'static, A, E>>
        for TryTrampoline<A, E>
    {
        fn from(thunk: TryThunk<'static, A, E>) -> Self {
            TryTrampoline::new(move || thunk.evaluate())
        }
    }
    ```
    Add the necessary import for `TryThunk` at the top of the module.
  - **Complexity:** Small.
  - **Dependencies:** None.
  - **Tests:** Test conversion of both Ok and Err `TryThunk` values.

- [ ] **3.4: Add `From<Result<A, E>>` for `TryThunk`, `TryTrampoline`, and `TryLazy`**
  - **Description:** Per Agent 7, ergonomic conversion from already-computed results.
  - **Files:** `fp-library/src/types/try_thunk.rs`, `fp-library/src/types/try_trampoline.rs`, `fp-library/src/types/try_lazy.rs`
  - **Implementation:** For each type, add `From<Result<A, E>>` that wraps the result in the appropriate constructor. For example:
    ```rust
    // TryThunk
    impl<'a, A: 'a, E: 'a> From<Result<A, E>> for TryThunk<'a, A, E> {
        fn from(result: Result<A, E>) -> Self {
            TryThunk::new(move || result)
        }
    }
    ```
    For `TryTrampoline`, require `A + E: 'static + Send`. For `TryLazy<RcLazyConfig>`, wrap in `TryLazy::new(move || result)`.
  - **Complexity:** Small.
  - **Dependencies:** None.
  - **Tests:** Test both Ok and Err paths for each type.

---

## Phase 4: TryThunk Type Class Completions

**Goal:** Add missing type class instances for `TryThunk`.

**Rationale:** Per Agent 2, `TryThunkBrand` implements `Bifunctor` but not `Bifoldable`, and `TryThunkOkAppliedBrand<A>` lacks `MonadRec`.

### Tasks

- [ ] **4.1: Add `Bifoldable` for `TryThunkBrand`**
  - **Description:** Per Agent 2, `TryThunk` is a single-element bifoldable container (either Ok or Err).
  - **Files:** `fp-library/src/types/try_thunk.rs`
  - **Implementation:** Follow the pattern of existing `Bifoldable` impls (e.g., for `ResultBrand` or `PairBrand`). Check `fp-library/src/classes/bifoldable.rs` for the exact trait signature. The implementation evaluates the `TryThunk` and folds over whichever side is present:
    ```rust
    impl Bifoldable for TryThunkBrand {
        fn bifold_right<'a, FnBrand: 'a + CloneableFn, A: 'a, B: 'a, C: 'a>(
            f: <FnBrand as Kind!(...)>::Of<'a, A, C>,
            g: <FnBrand as Kind!(...)>::Of<'a, B, C>,
            initial: C,
            fab: TryThunk<'a, B, A>,
        ) -> C {
            match fab.evaluate() {
                Ok(b) => g(b, initial),
                Err(a) => f(a, initial),
            }
        }
    }
    ```
    Note: Check the exact parameter ordering in the `Bifoldable` trait; `TryThunkBrand`'s Kind has `Of<'a, E, A>` where E is the error (first) and A is the success (second).
  - **Complexity:** Medium.
  - **Dependencies:** None.
  - **Tests:** Test folding over both Ok and Err variants.

- [ ] **4.2: Add `MonadRec` for `TryThunkOkAppliedBrand<A>`**
  - **Description:** Per Agent 2, the error-side monad should support tail recursion, short-circuiting on `Ok`.
  - **Files:** `fp-library/src/types/try_thunk.rs`
  - **Implementation:** Follow the existing `MonadRec for TryThunkErrAppliedBrand<E>` pattern, but swap the roles of Ok and Err:
    ```rust
    impl<A: 'static> MonadRec for TryThunkOkAppliedBrand<A> {
        fn tail_rec_m<'a, E: 'a, E2: 'a>(
            f: impl Fn(E) -> TryThunk<'a, A, Step<E, E2>> + Clone + 'a,
            e: E,
        ) -> TryThunk<'a, A, E2> {
            TryThunk::new(move || {
                let mut current = e;
                loop {
                    match f(current).evaluate() {
                        Err(Step::Loop(next)) => current = next,
                        Err(Step::Done(res)) => break Err(res),
                        Ok(a) => break Ok(a),
                    }
                }
            })
        }
    }
    ```
    Note: Verify the exact Kind mapping; `TryThunkOkAppliedBrand<A>::Of<'a, E>` = `TryThunk<'a, A, E>`, so `MonadRec` operates over the error type `E`.
  - **Complexity:** Medium.
  - **Dependencies:** None.
  - **Tests:** Test tail recursion over the error channel; test short-circuit on Ok.

- [ ] **4.3: Add `ok()` and `err()` convenience constructors to `TryLazy`**
  - **Description:** Per Agent 7, `TryLazy` lacks the convenience constructors that `TryThunk` and `TryTrampoline` have.
  - **Files:** `fp-library/src/types/try_lazy.rs`
  - **Implementation:** Add to both the `RcLazyConfig` and `ArcLazyConfig` impl blocks:
    ```rust
    pub fn ok(a: A) -> Self { Self::new(move || Ok(a)) }
    pub fn err(e: E) -> Self { Self::new(move || Err(e)) }
    ```
    For `ArcLazyConfig`, require `A: Send, E: Send`.
  - **Complexity:** Small.
  - **Dependencies:** None.
  - **Tests:** Test both constructors for both config variants.

- [ ] **4.4: Add `catch_unwind` to `TryThunk`**
  - **Description:** Per Agent 7, panic catching is useful beyond memoized contexts. Currently only on `TryLazy`.
  - **Files:** `fp-library/src/types/try_thunk.rs`
  - **Implementation:** Follow the `TryLazy::catch_unwind` pattern:
    ```rust
    pub fn catch_unwind(f: impl FnOnce() -> A + std::panic::UnwindSafe + 'a) -> TryThunk<'a, A, String> {
        TryThunk::new(move || {
            std::panic::catch_unwind(f)
                .map_err(|e| {
                    e.downcast::<String>()
                        .map(|s| *s)
                        .or_else(|e| e.downcast::<&str>().map(|s| s.to_string()))
                        .unwrap_or_else(|_| "unknown panic".to_string())
                })
        })
    }
    ```
  - **Complexity:** Small.
  - **Dependencies:** None.
  - **Tests:** Test with a panicking closure and a non-panicking closure.

---

## Phase 5: Property-Based Tests

**Goal:** Add QuickCheck law tests for `Thunk`, `Trampoline`, and `Lazy`.

**Rationale:** Per Agent 8, `Thunk` has zero QuickCheck property tests despite being a full Monad. Every other major type in the library has comprehensive law tests. `Trampoline` and `Lazy` also lack law verification.

### Tasks

- [ ] **5.1: Add QuickCheck law tests for `Thunk`**
  - **Description:** Per Agent 8, add Functor identity/composition, Monad left/right identity and associativity, Semigroup associativity, Monoid left/right identity.
  - **Files:** `fp-library/src/types/thunk.rs` (add to the existing `#[cfg(test)] mod tests` block)
  - **Implementation:** Follow the pattern from `pair.rs` (lines 1577-1650). Use `quickcheck_macros::quickcheck` attribute. Functions take primitive inputs, construct `Thunk` values via `pure::<ThunkBrand, _>()`, and compare `.evaluate()` results. Example:
    ```rust
    #[quickcheck]
    fn functor_identity(x: i32) -> bool {
        map::<ThunkBrand, _, _>(identity, pure::<ThunkBrand, _>(x)).evaluate() == x
    }
    ```
    Agent 8 provides complete test code in Section 8 of document `8.md`.
  - **Complexity:** Medium.
  - **Dependencies:** None (but benefits from all Phase 1-4 changes being in place).
  - **Tests:** This task IS the tests. Verify all pass with `cargo test -p fp-library`.

- [ ] **5.2: Add QuickCheck law tests for `Trampoline`**
  - **Description:** Per Agent 8, add Monad left/right identity, associativity, and Functor identity using inherent methods.
  - **Files:** `fp-library/src/types/trampoline.rs` (add to the existing `#[cfg(test)] mod tests` block)
  - **Implementation:** Since `Trampoline` has no HKT brand, use inherent methods directly:
    ```rust
    #[quickcheck]
    fn monad_left_identity(a: i32) -> bool {
        let f = |x: i32| Trampoline::pure(x.wrapping_mul(2));
        Trampoline::pure(a).bind(f).evaluate() == f(a).evaluate()
    }
    ```
  - **Complexity:** Medium.
  - **Dependencies:** None.
  - **Tests:** This task IS the tests.

- [ ] **5.3: Add QuickCheck law tests for `Lazy`**
  - **Description:** Per Agent 8, add RefFunctor identity/composition and Deferrable transparency tests. Also add Semigroup/Monoid laws if Phase 2 is complete.
  - **Files:** `fp-library/src/types/lazy.rs` (add to the existing `#[cfg(test)] mod tests` block)
  - **Implementation:**
    ```rust
    #[quickcheck]
    fn ref_functor_identity(x: i32) -> bool {
        let lazy = RcLazy::pure(x);
        *lazy.clone().ref_map(|v| *v).evaluate() == *lazy.evaluate()
    }

    #[quickcheck]
    fn deferrable_transparency(x: i32) -> bool {
        let lazy = RcLazy::pure(x);
        let deferred = RcLazy::defer(move || RcLazy::pure(x));
        *deferred.evaluate() == *lazy.evaluate()
    }
    ```
  - **Complexity:** Medium.
  - **Dependencies:** Task 2.2 and 2.3 for Semigroup/Monoid law tests.
  - **Tests:** This task IS the tests.

---

## Phase 6: Trampoline `Send` Bound Audit

**Goal:** Determine whether the `A: Send` bound on `Trampoline` can be relaxed.

**Rationale:** Per Agents 4 and 9, `Trampoline` requires `A: Send` on all methods, but `Trampoline` itself is `!Send` (because the internal `Thunk` closures are `!Send`). The `Send` bound prevents using `Trampoline` with `Rc<T>` and other `!Send` types for single-threaded stack-safe recursion. This is potentially the highest-impact change, but it is also the riskiest because it may have cascading effects.

### Tasks

- [ ] **6.1: Audit `A: Send` requirement on `Trampoline`**
  - **Description:** Determine whether removing `A: Send` from the `Trampoline` impl block causes compilation failures anywhere in the codebase.
  - **Files:** `fp-library/src/types/trampoline.rs` (line 77), `fp-library/src/types/try_trampoline.rs`, `fp-library/src/types/free.rs`
  - **Implementation:**
    1. Temporarily remove `+ Send` from `impl<A: 'static + Send> Trampoline<A>` (line 77).
    2. Run `cargo check --workspace --all-features` and catalog all errors.
    3. Trace where `Send` is actually needed (likely in the `Free` monad's type erasure or `CatList` operations).
    4. If `Send` is not needed, remove it permanently. If it is needed at specific points only, consider splitting into `Trampoline<A: 'static>` (general, `!Send`) and a `SendTrampoline<A: 'static + Send>` wrapper.
    5. If `Send` is removed, also update `TryTrampoline` to remove the `Send` bounds on `A` and `E`.
  - **Complexity:** Large (potentially cascading changes).
  - **Dependencies:** All other phases should be complete first, so the audit covers the final codebase shape.
  - **Tests:** All existing tests must continue to pass. Add a new test that uses `Trampoline` with an `Rc`-based type to verify the relaxation works.

---

## Phase 7: `fix` Combinator

**Goal:** Add PureScript's `fix` combinator for `RcLazy` and `ArcLazy`.

**Rationale:** Per Agents 1 and 6, PureScript's `Control.Lazy` exists primarily to support `fix :: (l -> l) -> l`, which ties recursive knots in lazy data structures. This is the most significant missing derived combinator.

### Tasks

- [ ] **7.1: Implement `fix` for `RcLazy`**
  - **Description:** Create a lazy value as the fixed point of a function, using `Rc<OnceCell>` for the self-referential setup.
  - **Files:** `fp-library/src/types/lazy.rs` (add as an inherent method on the `RcLazy` impl block, or as a standalone function)
  - **Implementation:** Use an `Rc<OnceCell<RcLazy<A>>>` to break the circularity:
    ```rust
    pub fn fix<'a, A: Clone + 'a>(
        f: impl Fn(RcLazy<'a, A>) -> RcLazy<'a, A> + 'a,
    ) -> RcLazy<'a, A> {
        use std::cell::OnceCell;
        let cell: Rc<OnceCell<RcLazy<'a, A>>> = Rc::new(OnceCell::new());
        let cell_ref = cell.clone();
        let result = RcLazy::new(move || {
            let go = cell_ref.get().expect("fix: cell not initialized").clone();
            f(go).evaluate().clone()
        });
        cell.set(result.clone()).ok();
        result
    }
    ```
    Add this as a standalone function in the lazy module (not as a trait method, since it requires `Clone` and self-referential structure per Agent 6's recommendation).
  - **Complexity:** Large (self-referential design requires careful testing).
  - **Dependencies:** None, but conceptually builds on the complete `Lazy` surface.
  - **Tests:** Test tying a simple recursive knot (e.g., a lazy value that references its own output). Test that evaluation produces the correct fixed point. Test that the `OnceCell` is properly initialized.

- [ ] **7.2: Implement `fix` for `ArcLazy`**
  - **Description:** Thread-safe variant using `Arc<OnceLock>`.
  - **Files:** `fp-library/src/types/lazy.rs`
  - **Implementation:** Same pattern as 7.1 but with `Arc<OnceLock<ArcLazy<'a, A>>>` and `Send + Sync` bounds on `A` and the closure.
  - **Complexity:** Medium (follows the pattern from 7.1).
  - **Dependencies:** Task 7.1.
  - **Tests:** Test thread-safe fixed point computation.

---

## Phase 8: Documentation and Polish

**Goal:** Address documentation gaps, add `Debug` impls, and small ergonomic improvements.

**Rationale:** These are low-risk, low-priority improvements that enhance the developer experience. Per Agents 3, 8, 9, and 10.

### Tasks

- [ ] **8.1: Document laws for `RefFunctor` trait**
  - **Description:** Per Agent 8, `RefFunctor` has no documented laws. Add identity and composition laws.
  - **Files:** `fp-library/src/classes/ref_functor.rs`
  - **Implementation:** Add a `### Laws` section to the trait doc comment:
    ```
    /// ### Laws
    ///
    /// * Identity: `ref_map(|x| x.clone(), fa)` evaluates to a value equal to `fa`'s evaluated value.
    /// * Composition: `ref_map(|x| f(&g(x)), fa)` evaluates to the same value as `ref_map(f, ref_map(g, fa))`.
    ```
  - **Complexity:** Small.
  - **Dependencies:** None.

- [ ] **8.2: Document laws for `Deferrable` trait**
  - **Description:** Per Agent 8, `Deferrable` has no documented laws. Add the transparency law.
  - **Files:** `fp-library/src/classes/deferrable.rs`
  - **Implementation:** Add a `### Laws` section:
    ```
    /// ### Laws
    ///
    /// * Transparency: for any `x: Self`, `defer(|| x)` is observationally equivalent to `x` when evaluated.
    ```
  - **Complexity:** Small.
  - **Dependencies:** None.

- [ ] **8.3: Document panic behavior in `Lazy`**
  - **Description:** Per Agent 5, if the initializer panics, `LazyCell`/`LazyLock` poison the cell. This is undocumented.
  - **Files:** `fp-library/src/types/lazy.rs`
  - **Implementation:** Add a `# Panics` section to the `Lazy` struct's doc comment explaining the poisoning behavior and recommending `TryLazy::catch_unwind` for panic-safe memoization.
  - **Complexity:** Small.
  - **Dependencies:** None.

- [ ] **8.4: Fix misleading `Thunk` doc comment**
  - **Description:** Per Agents 1 and 3, the module-level doc says "Each call to `evaluate` re-executes the computation," but `evaluate(self)` consumes the thunk so it can only be called once.
  - **Files:** `fp-library/src/types/thunk.rs`
  - **Implementation:** Change the module doc (line 3) from "Each call to `Thunk::evaluate` re-executes the computation" to "Does not cache results; if you need the same computation's result more than once, wrap it in `Lazy`." Also update the struct-level doc comment (line 40) similarly.
  - **Complexity:** Small.
  - **Dependencies:** None.

- [ ] **8.5: Add `Debug` implementations for lazy types**
  - **Description:** Per Agents 3 and 9, none of the lazy types implement `Debug`, making debugging difficult.
  - **Files:** `fp-library/src/types/thunk.rs`, `fp-library/src/types/trampoline.rs`, `fp-library/src/types/lazy.rs`, and the three `try_*` variants.
  - **Implementation:** For `Thunk` and `Trampoline`, show a static string since the value is unevaluated:
    ```rust
    impl<'a, A> fmt::Debug for Thunk<'a, A> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("Thunk(<unevaluated>)")
        }
    }
    ```
    For `Lazy`, since we cannot tell if it has been evaluated without accessing `LazyCell` internals (which are not public), also show a static string:
    ```rust
    impl<'a, A, Config: LazyConfig> fmt::Debug for Lazy<'a, A, Config>
    where
        A: 'a,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("Lazy(..)")
        }
    }
    ```
    Apply the same pattern to `TryThunk`, `TryTrampoline`, and `TryLazy`.
  - **Complexity:** Small.
  - **Dependencies:** None.

- [ ] **8.6: Update comparison table in `thunk.rs` to mention `Send` requirement**
  - **Description:** Per Agent 2, the "When to Use" table does not mention `Trampoline`'s `Send` requirement.
  - **Files:** `fp-library/src/types/thunk.rs`
  - **Implementation:** Add a row to the table at lines 53-58:
    ```
    /// | Thread safety  | Not `Send`                    | Requires `A: Send`           |
    ```
  - **Complexity:** Small.
  - **Dependencies:** Phase 6 (if `Send` is removed, this row changes).

- [ ] **8.7: Add `memoize()` convenience methods**
  - **Description:** Per Agent 10, add `Thunk::memoize()` and `Trampoline::memoize()` for discoverability of the `From` conversions.
  - **Files:** `fp-library/src/types/thunk.rs`, `fp-library/src/types/trampoline.rs`
  - **Implementation:**
    ```rust
    // In Thunk
    pub fn memoize(self) -> Lazy<'a, A, RcLazyConfig> {
        Lazy::from(self)
    }

    // In Trampoline
    pub fn memoize(self) -> Lazy<'static, A, RcLazyConfig> {
        Lazy::from(self)
    }
    ```
  - **Complexity:** Small.
  - **Dependencies:** None.

- [ ] **8.8: Document `LazyConfig` extensibility**
  - **Description:** Per Agent 5, the `LazyConfig` trait is open for extension but this is undocumented.
  - **Files:** `fp-library/src/types/lazy.rs`
  - **Implementation:** Add a doc section to the `LazyConfig` trait explaining that users can define custom configs (e.g., `parking_lot`-based or async-aware) by implementing the four associated types and methods.
  - **Complexity:** Small.
  - **Dependencies:** None.

- [ ] **8.9: Document `Fn` vs `FnOnce` discrepancy on `Thunk`**
  - **Description:** Per Agents 3 and 9, the inherent `bind` takes `FnOnce` but `Semimonad::bind` requires `Fn`. This should be documented.
  - **Files:** `fp-library/src/types/thunk.rs`
  - **Implementation:** Add a note to the inherent `bind` method's doc comment explaining that the HKT-level `bind` (via `Semimonad`) requires `Fn` closures because some types like `Vec` need to call the function multiple times, while the inherent method accepts the more permissive `FnOnce`.
  - **Complexity:** Small.
  - **Dependencies:** None.

---

## Out of Scope

The following items were mentioned in the analysis but are explicitly excluded from this plan:

- **Full `Functor`/`Monad` for `Lazy`:** Per Agents 1, 2, 5, and 10, implementing standard `Functor`/`Monad` for `Lazy` would require `A: Clone` bounds not present in the trait signatures. The `RefFunctor` approach is the correct adaptation for Rust's ownership model. No change warranted.

- **`Comonad`/`Extend` for `Lazy`:** Per Agent 10, a `RefComonad` trait could be defined, but the lifetime semantics of `extract` returning `&'a A` from a `Lazy<'a, A>` are problematic because the reference is tied to the `Rc`/`Arc`, not the `Lazy` value itself. This requires deeper design work and is better handled in a separate proposal.

- **`SendThunk<'a, A>` variant:** Per Agents 3 and 9, a `Send`-bounded thunk would fill the gap between `Thunk` (not `Send`) and `Trampoline` (requires `'static`). However, this introduces a new type to the hierarchy. The current design covers the use case through `Trampoline` for most practical scenarios. Defer until a concrete need arises.

- **`TryThunk` refactoring as newtype over `Thunk<'a, Result<A, E>>`:** Per Agent 7, this would reduce code duplication significantly but is a large refactor with many test updates. It does not add new capabilities. Track as a separate cleanup initiative.

- **`StaticKind` for `Trampoline`:** Per Agent 4, a separate Kind variant for `'static`-only types could allow `Trampoline` to participate in the HKT hierarchy. This would require a parallel `StaticFunctor`, `StaticMonad`, etc., hierarchy, which is a major architectural change. Out of scope.

- **Monad transformer (`ExceptT`) approach:** Per Agent 7, a generic `ExceptT` could theoretically replace the parallel `Try*` types. However, `Trampoline` has no HKT brand, `Lazy` returns references, and `LazyConfig` bakes in fallibility. The transformer approach is not practical with the current architecture.

- **Async/generators as lazy foundations:** Per Agent 10, these solve different problems (I/O concurrency and iterator generation) and are not suitable replacements.

- **`Deferrable` for function types:** Per Agent 6, PureScript has `Lazy (a -> b)`. In Rust, `Deferrable` requires `Self: Sized` and closures have opaque types. A newtype wrapper approach is possible but has limited practical value. Defer.

- **`resume` for `Free`:** Per Agent 4, structural inspection of the computation requires exposing `FreeInner`, which would break encapsulation. The current `Evaluable`-based interpretation is sufficient.
