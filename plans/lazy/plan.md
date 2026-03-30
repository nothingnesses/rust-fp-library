# Lazy Hierarchy Improvement Plan

This plan addresses all issues found across the 17 files comprising the lazy evaluation hierarchy. Issues are grouped thematically and ordered by impact. Each issue includes the affected files, the chosen approach, and implementation notes.

## Files Analyzed

**Classes:** deferrable.rs, send_deferrable.rs, lazy_config.rs, try_lazy_config.rs, ref_functor.rs, send_ref_functor.rs
**Types:** thunk.rs, send_thunk.rs, try_thunk.rs, try_send_thunk.rs, lazy.rs, try_lazy.rs, trampoline.rs, try_trampoline.rs, free.rs, cat_list.rs
**Brands:** brands.rs (lazy-related subset)

---

## Implementation Phases

| Phase | Issues     | Description                                                                   |
| ----- | ---------- | ----------------------------------------------------------------------------- |
| 1     | 11, 12, 13 | Trivial fixes (doc, redundant bound, comment).                                |
| 2     | 4          | Add inherent `tail_rec_m` to TryThunk.                                        |
| 3     | 1          | TryLazy refactor: newtype over Lazy<Result<A, E>>, eliminate TryLazyConfig.   |
| 4     | 2          | Make SendDeferrable independent of Deferrable, remove eager Deferrable impls. |
| 5     | 7, 8       | Add Send + Sync bounds to ArcLazy::new; align error channel to fail-fast.     |
| 6     | 9, 10      | Fix combinator tests; document Drop-triggers-evaluation.                      |

Phases 1 and 2 can run in parallel. Phase 5's two items are independent of each other.

---

## Issue 1: TryLazy/TryLazyConfig duplication with Lazy/LazyConfig

**Phase:** 3
**Severity:** High
**Affected files:** try_lazy.rs (~3830 lines), try_lazy_config.rs, lazy.rs, lazy_config.rs, brands.rs

### Problem

`TryLazy<'a, A, E, Config>` is structurally identical to `Lazy<'a, Result<A, E>, Config>` at the storage level. The `TryLazyConfig` trait is redundant: its associated types and methods are just `LazyConfig`'s with `Result<A, E>` plugged in. This causes massive duplication: Clone, Hash, PartialEq, Eq, Ord, Display, Debug, Deferrable, SendDeferrable, Semigroup, Monoid, Foldable, FoldableWithIndex, RefFunctor, SendRefFunctor, fix-point combinators, and From conversions are all duplicated between the two files.

### Decision: TryLazy as newtype over Lazy<Result<A, E>>

Redefine `TryLazy` as:

```rust
pub struct TryLazy<'a, A, E, Config: LazyConfig = RcLazyConfig>(
    Lazy<'a, Result<A, E>, Config>,
);
```

**What changes:**

- `TryLazyConfig` is eliminated. `TryLazy` uses `LazyConfig` directly.
- Clone, Hash, PartialEq, Eq, Ord, Debug inherit from `Lazy` by delegation.
- Deferrable, Semigroup, Monoid can delegate to `Lazy`'s implementations.
- `evaluate()` calls `self.0.evaluate().as_ref()` to split `&Result<A, E>` into `Result<&A, &E>`.
- `TryLazyBrand<E, Config>` remains separate (different HKT semantics from `LazyBrand`).
- Error-aware combinators (`map`, `map_err`, `and_then`, `or_else`, `bimap`) remain on `TryLazy`.
- `Display` for `TryLazy` needs custom handling (cannot delegate to `Lazy<Result<A,E>>::Display` because the formatting differs).

**Why this is feasible:** `Lazy`'s `Deferrable` for `RcLazyConfig` requires `A: Clone`. With `A = Result<Ok, Err>`, that becomes `Ok: Clone + Err: Clone`. The current `TryLazy` Deferrable already requires both `A: Clone` and `E: Clone`, so bounds align. The brand system is unaffected since `impl_kind!` maps brands to concrete types regardless of internal structure. The library is pre-1.0 (v0.14.0), so removing the public `TryLazyConfig` trait is acceptable under semver.

---

## Issue 2: Deferrable for Send types evaluates eagerly

**Phase:** 4
**Severity:** Moderate
**Affected files:** deferrable.rs, send_deferrable.rs, send_thunk.rs, try_send_thunk.rs, lazy.rs (ArcLazy), try_lazy.rs (ArcTryLazy)

### Problem

`SendThunk`, `TrySendThunk`, `ArcLazy`, and `ArcTryLazy` all implement `Deferrable::defer` with eager evaluation: `fn defer(f) { f() }`. This is because the trait's closure is `impl FnOnce() -> Self + 'a` (no `Send`), but these types require `Send` closures internally. The trait law (transparency) is satisfied, but the semantic contract ("defer") is violated.

The root cause is `SendDeferrable: Deferrable`. Since every `SendDeferrable` must also be `Deferrable`, and the `Deferrable` closure is not `Send`, the only option is eager evaluation.

### Decision: Make SendDeferrable independent of Deferrable

Remove the supertrait relationship. `SendDeferrable` becomes a standalone trait. Remove `Deferrable` impls from `SendThunk`, `TrySendThunk`, `ArcLazy`, and `ArcTryLazy`.

**What changes:**

- `send_deferrable.rs`: Remove `: Deferrable<'a>` from `SendDeferrable` trait definition.
- `send_thunk.rs`: Remove `Deferrable` impl for `SendThunk`.
- `try_send_thunk.rs`: Remove `Deferrable` impl for `TrySendThunk`.
- `lazy.rs`: Remove `Deferrable` impl for `Lazy<'a, A, ArcLazyConfig>`.
- `try_lazy.rs` (post-refactor): Remove `Deferrable` impl for `TryLazy<'a, A, E, ArcLazyConfig>`.
- `deferrable.rs`: Update documentation to note that `Deferrable` is for non-Send types only; Send types use `SendDeferrable`.

**Why this is safe:** Grep shows no library code uses `D: Deferrable` as a generic bound to accept both single-threaded and Send types. The only generic uses are the `defer` free function itself and the `SendDeferrable` supertrait declaration. No existing generic code breaks.

---

## Issue 3: Lazy types cannot implement standard Functor/Monad

**Phase:** N/A (deferred)
**Severity:** Moderate
**Affected files:** lazy.rs, try_lazy.rs, ref_functor.rs, send_ref_functor.rs

### Decision: Keep current design

Accept that `Lazy` has limited HKT support. The separation between `Thunk` (full HKT, no memoization) and `Lazy` (memoization, limited HKT via `RefFunctor`/`SendRefFunctor`) is a reasonable trade-off for Rust's ownership model.

---

## Issue 4: Missing inherent `tail_rec_m` on TryThunk

**Phase:** 2
**Severity:** Moderate
**Affected files:** try_thunk.rs

### Decision: Add inherent `tail_rec_m` to TryThunk

Add `tail_rec_m` as an inherent method, mirroring `TrySendThunk`'s implementation. This is purely additive and non-breaking. Fills the gap for non-`'static` error types that cannot use the `MonadRec for TryThunkErrAppliedBrand<E>` trait.

---

## Issue 5: Code duplication across Try\* variants

**Phase:** N/A (accepted)
**Severity:** Moderate
**Affected files:** try_thunk.rs vs thunk.rs, try_send_thunk.rs vs send_thunk.rs, try_trampoline.rs vs trampoline.rs

### Decision: Accept duplication

The duplication is a consequence of Rust's type system. Each Try\* type has slightly different bounds and behaviors. Maintaining parallel code is the pragmatic choice. If the duplication becomes a source of divergence bugs, reconsider macro-based code generation.

---

## Issue 6: Per-step allocation overhead in Free

**Phase:** N/A (accepted)
**Severity:** Moderate
**Affected files:** free.rs, trampoline.rs, try_trampoline.rs

### Decision: Keep current design

The safe `Box<dyn Any>` approach prioritizes correctness and maintainability. The overhead is acceptable for the library's use cases.

---

## Issue 7: ArcLazy::new does not require Send + Sync on A

**Phase:** 5
**Severity:** Low-moderate
**Affected files:** lazy.rs

### Decision: Add `A: Send + Sync` bounds

Add `A: Send + Sync` bounds to `ArcLazy::new`, `ArcLazy::pure`, and other `ArcLazy`-specific constructors. Every existing usage in the codebase already passes `Send + Sync` types, so no internal code breaks. The library is pre-1.0, and the bounds match the type's intent.

Also propagate these bounds to `ArcTryLazy` constructors (`new`, `ok`, `err`, `pure`) where applicable.

---

## Issue 8: TryThunkOkAppliedBrand Lift/Semiapplicative vs Semimonad semantics

**Phase:** 5
**Severity:** Low-moderate
**Affected files:** try_thunk.rs

### Decision: Align Lift/Semiapplicative with Semimonad (fail-fast)

Change `Lift::lift2` and `Semiapplicative::apply` on the error channel (`TryThunkOkAppliedBrand<A>`) to use fail-fast semantics, matching `Semimonad::bind`. This restores the law `liftA2 f x y = x >>= \a -> fmap (f a) y`.

If error accumulation is needed, it belongs in a dedicated abstraction (e.g., a `Validation` type), not in the standard `Lift`/`Semiapplicative` interface.

---

## Issue 9: Fix combinator tests are incomplete

**Phase:** 6
**Severity:** Low-moderate
**Affected files:** lazy.rs, try_lazy.rs

### Decision: Add knot-tying tests

Add tests that build cyclic data structures using the fix combinators, demonstrating:

1. A lazy value that references itself and produces a correct result when the self-reference is used after initialization completes.
2. Proper panic behavior when the self-reference is forced during initialization.

---

## Issue 10: Drop on Free triggers thunk evaluation

**Phase:** 6
**Severity:** Low-moderate
**Affected files:** free.rs, trampoline.rs, try_trampoline.rs

### Decision: Document the behavior

Add documentation to `Free`, `Trampoline`, and `TryTrampoline` noting that dropping a partially-built computation chain may trigger thunk evaluation. The Drop must evaluate thunks to avoid memory leaks; this is an inherent consequence of the design.

---

## Issue 11: SendThunk tail_rec_m documentation mentions Clone incorrectly

**Phase:** 1
**Severity:** Low
**Affected files:** send_thunk.rs

### Decision: Fix the documentation

The doc comment incorrectly says the function needs `Clone`. The actual signature requires `impl Fn(S) -> ...`, which is already multi-callable. Remove the incorrect `Clone` mention.

---

## Issue 12: Thunk::pure has redundant `where A: 'a` bound

**Phase:** 1
**Severity:** Low
**Affected files:** thunk.rs

### Decision: Remove the redundant where clause

The impl block already constrains `A: 'a`, making the where clause on `pure` redundant.

---

## Issue 13: mem::forget in CatList::uncons is fragile

**Phase:** 1
**Severity:** Low
**Affected files:** cat_list.rs

### Decision: Add a safety comment

Add a comment documenting the invariant: `mem::forget` is safe here because `CatListInner` does not have a custom `Drop`. If `CatListInner` ever gains a custom `Drop`, this code must be restructured.

---

## Issue 14: No RefApplicative/RefMonad for Lazy types

**Phase:** N/A (deferred)
**Severity:** Low
**Affected files:** ref_functor.rs, send_ref_functor.rs, lazy.rs

### Decision: Defer

No action unless there is concrete demand for monadic `Lazy` composition via references.

---

## Summary

| Phase | Issue                      | Decision                        | Effort       |
| ----- | -------------------------- | ------------------------------- | ------------ |
| 1     | 11. SendThunk doc fix      | Fix documentation               | Trivial      |
| 1     | 12. Redundant where clause | Remove                          | Trivial      |
| 1     | 13. CatList mem::forget    | Add safety comment              | Trivial      |
| 2     | 4. TryThunk tail_rec_m     | Add inherent method             | Small        |
| 3     | 1. TryLazy duplication     | Newtype over Lazy<Result>       | Large        |
| 4     | 2. Deferrable eagerness    | Make SendDeferrable independent | Medium       |
| 5     | 7. ArcLazy bounds          | Add Send + Sync                 | Small        |
| 5     | 8. Error channel semantics | Align to fail-fast              | Small-medium |
| 6     | 9. Fix combinator tests    | Add knot-tying tests            | Small        |
| 6     | 10. Drop documentation     | Document behavior               | Small        |
| --    | 3. No Functor for Lazy     | Deferred                        | --           |
| --    | 5. Try\* duplication       | Accepted                        | --           |
| --    | 6. Free allocation         | Keep current                    | --           |
| --    | 14. No RefApplicative      | Deferred                        | --           |
