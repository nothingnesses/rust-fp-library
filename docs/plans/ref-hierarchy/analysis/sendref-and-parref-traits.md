# Analysis: SendRef and ParRef Trait Families

## Overview

The library has three tiers of by-reference traits:

- **Ref traits** (e.g., `RefFunctor`) - by-reference access, no thread-safety requirements.
- **SendRef traits** (e.g., `SendRefFunctor`) - by-reference access with `Send + Sync` bounds on closures and elements.
- **ParRef traits** (e.g., `ParRefFunctor`) - by-reference access with parallel execution via rayon.

This analysis examines design flaws, inconsistencies, gaps, and improvement opportunities across the SendRef and ParRef families.

## 1. SendRef Hierarchy Completeness

### 1.1 Current SendRef traits

The SendRef family currently provides:

| Trait                      | Supertrait(s)                                                                      | Implementors                                                 |
| -------------------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| `SendRefFunctor`           | (none)                                                                             | `LazyBrand<ArcLazyConfig>`, `TryLazyBrand<E, ArcLazyConfig>` |
| `SendRefPointed`           | (none)                                                                             | `LazyBrand<ArcLazyConfig>`                                   |
| `SendRefLift`              | (none)                                                                             | `LazyBrand<ArcLazyConfig>`                                   |
| `SendRefSemiapplicative`   | `SendRefLift + SendRefFunctor`                                                     | `LazyBrand<ArcLazyConfig>`                                   |
| `SendRefSemimonad`         | (none)                                                                             | `LazyBrand<ArcLazyConfig>`                                   |
| `SendRefApplyFirst`        | `SendRefLift`                                                                      | blanket (all `SendRefLift` impls)                            |
| `SendRefApplySecond`       | `SendRefLift`                                                                      | blanket (all `SendRefLift` impls)                            |
| `SendRefApplicative`       | `SendRefPointed + SendRefSemiapplicative + SendRefApplyFirst + SendRefApplySecond` | blanket                                                      |
| `SendRefMonad`             | `SendRefApplicative + SendRefSemimonad`                                            | blanket                                                      |
| `SendRefFoldable`          | `RefFoldable`                                                                      | `LazyBrand<ArcLazyConfig>`                                   |
| `SendRefFoldableWithIndex` | `SendRefFoldable + WithIndex`                                                      | `LazyBrand<ArcLazyConfig>`                                   |
| `SendRefFunctorWithIndex`  | `SendRefFunctor + WithIndex`                                                       | `LazyBrand<ArcLazyConfig>`                                   |

### 1.2 Missing SendRef traits

The Ref family has traits that have no SendRef counterpart:

| Ref trait                 | SendRef counterpart           | Status  |
| ------------------------- | ----------------------------- | ------- |
| `RefFilterable`           | `SendRefFilterable`           | Missing |
| `RefFilterableWithIndex`  | `SendRefFilterableWithIndex`  | Missing |
| `RefTraversable`          | `SendRefTraversable`          | Missing |
| `RefTraversableWithIndex` | `SendRefTraversableWithIndex` | Missing |
| `RefWitherable`           | `SendRefWitherable`           | Missing |

**Assessment:** The plan (step 21) explicitly defers SendRef variants of `RefFilterable`, `RefTraversable`, and `RefWitherable` with the rationale: "no thread-safe memoized type needs filtering/traversal." This is a sound deferral since the only current implementor is `ArcLazy`, which is a single-element container where filtering and traversal are not meaningful operations. If a thread-safe collection type is added in the future (e.g., a concurrent `CatList` backed by `Arc`), these traits would become necessary.

**Recommendation:** The gap is intentional and well-documented. Add a note to the plan or a tracking issue so these traits are created when a concurrent collection type is introduced.

## 2. Relationship Between Ref, SendRef, and ParRef

### 2.1 Current supertrait structure

The three families have inconsistent supertrait relationships:

- **SendRef vs Ref:** Most SendRef traits have **no** Ref supertrait. `SendRefFunctor`, `SendRefPointed`, `SendRefLift`, `SendRefSemimonad` are all standalone traits. The exception is `SendRefFoldable: RefFoldable`, which requires its Ref counterpart as a supertrait.
- **ParRef vs Ref:** All ParRef traits require their Ref counterpart. `ParRefFunctor: RefFunctor`, `ParRefFoldable: RefFoldable`, `ParRefFilterable: ParRefFunctor + ParCompactable`, `ParRefFunctorWithIndex: ParRefFunctor + RefFunctorWithIndex`, etc.
- **ParRef vs SendRef:** No relationship. There is no `SendRefFunctor` supertrait on `ParRefFunctor`.
- **Par vs by-value:** For comparison, the by-value `ParFunctor` has **no** `Functor` supertrait, and `ParFoldable` has **no** `Foldable` supertrait.

### 2.2 Analysis of the inconsistency

The inconsistency in the SendRef hierarchy appears to be an oversight rather than a design decision. Consider:

- **`SendRefFoldable: RefFoldable`** requires the Ref supertrait, but `SendRefFunctor` does not require `RefFunctor`. There is no documented reason for the difference.
- **The plan states** (Design Decision 1): "By-ref and by-value traits are independent (no sub/supertrait relationship)." This applies to by-value vs by-ref, but the plan does not explicitly address the Ref vs SendRef relationship.
- **The practical impact:** A type that implements `SendRefFunctor` can be used with `send_ref_map` but not `ref_map`, even though it logically could support both. If `SendRefFunctor: RefFunctor` were required, then any `SendRefFunctor` type would automatically be usable in generic code expecting `RefFunctor`, providing subtype polymorphism.

**Why `SendRefFoldable: RefFoldable` exists:** The mutual derivation mechanism in `SendRefFoldable` (`send_ref_fold_right`, `send_ref_fold_left`) uses `Endofunction` patterns that share implementation strategy with `RefFoldable`. The supertrait ensures that if you can fold thread-safely, you can also fold sequentially. This is logically sound.

**Why other SendRef traits lack Ref supertraits:** For `ArcLazy`, the by-ref closure bounds differ between Ref and SendRef variants. `RefFunctor::ref_map` takes `impl Fn(&A) -> B + 'a`, while `SendRefFunctor::send_ref_map` takes `impl Fn(&A) -> B + Send + 'a`. A type can implement both independently, but making `SendRefFunctor: RefFunctor` would mean that `ArcLazy` must also implement `RefFunctor` (which it currently does not, since `ArcLazy` only implements `SendRefFunctor`).

**Finding:** The lack of `RefFunctor` impl for `ArcLazy` is the root cause. `ArcLazy` should logically support `ref_map` (a non-`Send` closure is a subset of operations); the `Send` bound only restricts what closures can be used, not the container's capability. However, `ArcLazy::new` requires `Send` closures, so the `ref_map` implementation would need to add `Send` bounds internally. This creates a mismatch: the `RefFunctor` trait signature does not require `Send`, but `ArcLazy` needs it.

**Recommendation:** The current approach is pragmatically correct. Making `SendRefFunctor: RefFunctor` would require `ArcLazy` to accept non-`Send` closures (which it cannot, since `Arc` requires thread-safe internals). The inconsistency with `SendRefFoldable: RefFoldable` should be documented, and the `RefFoldable` supertrait should be revisited: either add Ref supertraits to all SendRef traits and implement Ref traits for `ArcLazy`, or remove the `RefFoldable` supertrait from `SendRefFoldable`. The latter is easier but loses the guarantee that thread-safe foldables can be folded sequentially.

### 2.3 ParRef vs SendRef relationship

There is no connection between ParRef and SendRef. This means:

- `ParRefFunctor: RefFunctor` (has Ref supertrait)
- `SendRefFunctor` has no Ref supertrait

A type like `VecBrand` that implements `ParRefFunctor` automatically satisfies `RefFunctor`, but there is no way to express "this type supports parallel, thread-safe, by-reference mapping" as a single trait bound. Since ParRef traits are for collections and SendRef traits are for memoized types, the families do not currently overlap, so this is not a practical problem.

## 3. Send + Sync Bound Consistency

### 3.1 Closure bounds

| Trait                                  | Closure bound                                 |
| -------------------------------------- | --------------------------------------------- |
| `SendRefFunctor::send_ref_map`         | `impl Fn(&A) -> B + Send + 'a`                |
| `SendRefLift::send_ref_lift2`          | `impl Fn(&A, &B) -> C + Send + 'a`            |
| `SendRefSemimonad::send_ref_bind`      | `impl Fn(&A) -> Of<B> + Send + 'a`            |
| `SendRefFoldable::send_ref_fold_map`   | `impl Fn(&A) -> M + Send + Sync + 'a`         |
| `SendRefFoldableWithIndex`             | `impl Fn(Index, &A) -> M + Send + Sync + 'a`  |
| `ParRefFunctor::par_ref_map`           | `impl Fn(&A) -> B + Send + Sync + 'a`         |
| `ParRefFoldable::par_ref_fold_map`     | `impl Fn(&A) -> M + Send + Sync + 'a`         |
| `ParRefFilterable::par_ref_filter_map` | `impl Fn(&A) -> Option<B> + Send + Sync + 'a` |

**Finding: Inconsistent `Sync` bounds on closures.** The SendRef monadic traits (`SendRefFunctor`, `SendRefLift`, `SendRefSemimonad`) require `Send` on closures but not `Sync`. The SendRef foldable traits and all ParRef traits require `Send + Sync`. This discrepancy exists because:

- The monadic traits (functor, lift, semimonad) capture the closure into a new `ArcLazy`, which moves the closure into a single `Arc<OnceCell<...>>`. The closure is called at most once from a single thread, so `Send` suffices (the closure is sent to the evaluation thread but not shared).
- The foldable traits iterate over elements, potentially from multiple threads, so `Sync` is needed for the closure to be called from multiple threads.
- ParRef traits always use rayon, where closures are called from multiple threads, requiring `Sync`.

**Assessment:** The bounds are correctly applied based on usage patterns. The asymmetry is semantically justified, not an error. This should be documented more prominently since it can be confusing.

### 3.2 Element bounds

| Trait              | Element bound                                |
| ------------------ | -------------------------------------------- |
| `SendRefFunctor`   | `A: Send + Sync + 'a, B: Send + Sync + 'a`   |
| `SendRefFoldable`  | `A: Send + Sync + 'a + Clone`                |
| `ParRefFunctor`    | `A: Send + Sync + 'a, B: Send + 'a`          |
| `ParRefFoldable`   | `A: Send + Sync + 'a, M: Monoid + Send + 'a` |
| `ParRefFilterable` | `A: Send + Sync + 'a, B: Send + 'a`          |

**Finding: Asymmetric `Sync` bounds on output types.** In `SendRefFunctor`, both `A` and `B` require `Send + Sync`. In `ParRefFunctor`, `A` requires `Send + Sync` but `B` only requires `Send`. This is because:

- `SendRefFunctor` produces an `ArcLazy<B>`, which requires `B: Send + Sync` for the `Arc` internals.
- `ParRefFunctor` produces a `Vec<B>` (or `CatList<B>`), which only requires `B: Send` for rayon to collect results.

**Assessment:** Correct. The bounds follow from the concrete container types.

### 3.3 The `Clone` bound on `SendRefFoldable` element type

`SendRefFoldable::send_ref_fold_map` requires `A: Clone`, while `RefFoldable::ref_fold_map` also requires `A: Clone`. Both use this for the mutual derivation mechanism (endofunction encoding). In `ParRefFoldable::par_ref_fold_map`, there is no `Clone` bound because there is no mutual derivation; the trait only has `fold_map`, not `fold_right`/`fold_left`.

**Assessment:** The `Clone` bound is overly restrictive for direct implementations of `send_ref_fold_map` that do not use the default `fold_right`/`fold_left` derivations. A type implementing `send_ref_fold_map` directly should not need `A: Clone`. However, the bound is on the trait method, not the trait, so it cannot be relaxed without breaking the default implementations.

**Recommendation:** Consider splitting the `Clone` bound: make `send_ref_fold_map` not require `Clone`, and add `Clone` only to `send_ref_fold_right` and `send_ref_fold_left` (which use the endofunction encoding). This would require adjusting the default implementation of `send_ref_fold_map` to not call `send_ref_fold_right` (or gating the default impl behind a `Clone` bound).

## 4. ParRef Trait Coverage

### 4.1 Current ParRef implementors

ParRef traits are implemented for only two types:

- `VecBrand` - all 6 ParRef traits.
- `CatListBrand` - all 6 ParRef traits.

### 4.2 Types that could support ParRef

| Type                       | ParRef support?            | Notes                                                 |
| -------------------------- | -------------------------- | ----------------------------------------------------- |
| `VecBrand`                 | Yes (implemented)          | Natural fit, backed by `Vec` with rayon `par_iter()`. |
| `CatListBrand`             | Yes (implemented)          | Collects to `Vec` for parallel processing, rebuilds.  |
| `OptionBrand`              | Could, but pointless       | At most one element; parallelism has no benefit.      |
| `IdentityBrand`            | Could, but pointless       | Exactly one element; parallelism has no benefit.      |
| `ResultBrand<E>`           | Could, but pointless       | At most one element.                                  |
| `LazyBrand<ArcLazyConfig>` | Could for `ParRefFoldable` | Single-element, but already thread-safe. Not useful.  |

**Assessment:** The current coverage is appropriate. `Vec` and `CatList` are the only collection types where parallel processing provides meaningful speedup. Single-element containers would add implementation burden for no practical benefit.

### 4.3 Missing ParRef trait families

Comparing ParRef to Par (by-value), the following Par traits have no ParRef counterpart:

| Par trait                | ParRef counterpart          | Status                              |
| ------------------------ | --------------------------- | ----------------------------------- |
| `ParFunctor`             | `ParRefFunctor`             | Done                                |
| `ParFoldable`            | `ParRefFoldable`            | Done                                |
| `ParFilterable`          | `ParRefFilterable`          | Done                                |
| `ParCompactable`         | (none)                      | Not needed (structural, no closure) |
| `ParFunctorWithIndex`    | `ParRefFunctorWithIndex`    | Done                                |
| `ParFoldableWithIndex`   | `ParRefFoldableWithIndex`   | Done                                |
| `ParFilterableWithIndex` | `ParRefFilterableWithIndex` | Done                                |

The only missing one is `ParRefCompactable`, which follows the design principle that structural operations (`compact`, `separate`) do not need Ref variants because they do not access elements via closures.

**Assessment:** The ParRef family is complete for practical purposes.

### 4.4 Missing ParRef traits that have no Par counterpart

| Trait                   | Status  | Rationale for absence                                                                                                                                                           |
| ----------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ParRefTraversable`     | Missing | Traversal requires an applicative context for the output, which complicates parallel execution. The applicative effects must be sequenced, conflicting with parallel semantics. |
| `ParRefWitherable`      | Missing | Depends on `ParRefTraversable`.                                                                                                                                                 |
| `ParRefPointed`         | N/A     | `pure` is a single-element operation; parallelism is meaningless.                                                                                                               |
| `ParRefSemiapplicative` | N/A     | `apply` involves function application, not iteration.                                                                                                                           |
| `ParRefSemimonad`       | N/A     | `bind` produces nested containers; not a parallel operation.                                                                                                                    |

**Assessment:** The absences are well-justified. Parallel traversal is a known hard problem in FP; effects must be sequenced, which inherently conflicts with parallelism. If a use case emerges (e.g., independent IO operations in parallel), it could be revisited.

## 5. Rayon Integration in ParRef Traits

### 5.1 Thread-safety of implementations

The `Vec` implementations use `rayon::par_iter()` directly:

```rust
fn par_ref_map(...) -> ... {
    fa.par_iter().map(f).collect()
}
```

When the `rayon` feature is disabled, implementations fall back to sequential `iter()`:

```rust
fn par_ref_map(...) -> ... {
    fa.iter().map(f).collect()
}
```

**Finding:** The `Send + Sync` bounds are maintained even without rayon, ensuring API compatibility. This means code compiles identically with or without the feature flag, which is a good design choice.

### 5.2 CatList parallel implementation

`CatListBrand`'s ParRef impls collect to `Vec` first, then process in parallel, then rebuild:

```rust
fn par_ref_map(...) -> ... {
    let vec: Vec<_> = fa.iter().collect();
    vec.par_iter().map(|a| f(a)).collect::<Vec<_>>().into_iter().collect()
}
```

**Finding:** This involves two allocations (Vec for input, Vec for output) plus the CatList rebuild. The overhead may negate parallelism benefits for small collections. However, this is an inherent limitation of CatList's tree structure, which is not amenable to random access.

**Recommendation:** Consider adding a size threshold below which CatList ParRef operations fall back to sequential Ref operations. This would avoid overhead for small collections.

### 5.3 Potential race conditions

All ParRef trait methods take closures by `impl Fn(&A) -> B + Send + Sync`. The `Fn` bound (not `FnMut`) ensures the closure does not require exclusive access, so there are no data races from the closure itself. The `Send + Sync` bounds on elements ensure `&A` can be safely shared across threads.

**Assessment:** No thread-safety issues found. The bounds are correctly specified.

## 6. Code Duplication

### 6.1 Ref / SendRef duplication

The SendRef traits are near-identical copies of the Ref traits with added `Send + Sync` bounds. For example:

- `RefFunctor::ref_map` takes `impl Fn(&A) -> B + 'a`
- `SendRefFunctor::send_ref_map` takes `impl Fn(&A) -> B + Send + 'a`

The trait definitions, free functions, and documentation are duplicated. For the monadic chain alone (`Functor`, `Pointed`, `Lift`, `Semiapplicative`, `Semimonad`, `ApplyFirst`, `ApplySecond`, `Applicative`, `Monad`), there are 9 Ref traits and 9 SendRef traits with nearly identical structure.

**Quantification:** Approximately 1,500+ lines of SendRef code are structural duplicates of Ref code, differing only in `Send + Sync` bounds and method/trait name prefixes.

### 6.2 ParRef / Ref duplication

The ParRef traits duplicate the structure of Ref traits with `Send + Sync` bounds and different method names. An additional 6 ParRef traits add approximately 600+ lines of code.

### 6.3 Could duplication be reduced?

**Approach 1: Macro-generated trait families.** A declarative macro could generate Ref, SendRef, and ParRef variants from a single definition. For example:

```rust
define_ref_trait_family! {
    base_name: Functor,
    method: map,
    signature: fn(impl Fn(&A) -> B, fa: Self::Of<A>) -> Self::Of<B>,
}
```

This would generate `RefFunctor`, `SendRefFunctor`, and `ParRefFunctor` with appropriate bounds.

**Challenges:**

- Each family has different supertrait requirements.
- ParRef traits have different semantics (parallel execution, no fold_right/fold_left).
- Documentation must differ between variants.
- The `Apply!` macro and `#[kind(...)]` attribute interact with code generation.

**Approach 2: Generic over a marker trait.** A single `RefFunctor<Mode>` with `Mode` selecting the bound level (plain, send, par). This was investigated and rejected for the by-value/by-ref split (plan section "Rejected alternative: Mode-parameterized Functor") due to lifetime provenance issues. The same problems would apply here, compounded by the need to parameterize `Send + Sync` bounds.

**Assessment:** The duplication is a pragmatic trade-off. The traits are simple enough that the duplication is manageable, and attempting to reduce it via macros would sacrifice clarity and make the codebase harder to navigate. A declarative macro approach is feasible but increases the learning curve. The current approach prioritizes readability over DRY.

**Recommendation:** Accept the duplication for now. If a fourth tier is ever needed (e.g., `AsyncRef` for async by-reference operations), the proliferation would become unsustainable, and a macro-based approach should be revisited.

## 7. SendEndofunction

### 7.1 Purpose

`SendEndofunction<FnBrand, A>` wraps a function `A -> A` in an `Arc<dyn Fn(A) -> A + Send + Sync>` (via `SendCloneFn`). It provides `Semigroup` (composition) and `Monoid` (identity) instances. This enables the mutual derivation mechanism in `SendRefFoldable`: `send_ref_fold_right` is derived from `send_ref_fold_map` using `SendEndofunction<FnBrand, B>` as the monoid.

### 7.2 Comparison with `Endofunction`

`Endofunction<FnBrand, A>` uses `Rc<dyn Fn(A) -> A>` (via `CloneFn`). `SendEndofunction` is its `Arc`-based counterpart.

**Finding:** The parallel structure is clean: `Endofunction` is to `RefFoldable` as `SendEndofunction` is to `SendRefFoldable`. Both provide the same mutual derivation pattern.

### 7.3 Is this the right approach?

**Alternatives considered:**

1. **Parameterize `Endofunction` over a pointer brand.** `Endofunction<P: PointerBrand, A>` where `P` selects `Rc` or `Arc`. This would avoid a separate type. However, `Endofunction` currently uses `CloneFn`'s `Of` type, and `SendEndofunction` uses `SendCloneFn`'s `Of` type. These are different trait families (deliberately independent per the plan), so a unified `Endofunction` would need to be generic over the trait family, not just the pointer brand.

2. **Use a newtype wrapper.** `SendEndofunction = Endofunction<ArcFnBrand, A>`. This would work if `Endofunction` were generic over the fn brand, but it is already `Endofunction<FnBrand: LiftFn, A>`, and `SendEndofunction` needs `FnBrand: SendLiftFn`. Since `SendLiftFn` does not extend `LiftFn` (they are independent), this does not unify.

**Assessment:** `SendEndofunction` as a separate type is the correct approach given the independence of `CloneFn` and `SendCloneFn`. The design follows the same pattern used throughout the library.

## 8. Naming Convention Consistency

### 8.1 Current naming patterns

| Pattern           | Examples                                              |
| ----------------- | ----------------------------------------------------- |
| `Ref*`            | `RefFunctor`, `RefFoldable`, `RefPointed`             |
| `SendRef*`        | `SendRefFunctor`, `SendRefFoldable`, `SendRefPointed` |
| `Par*` (by-value) | `ParFunctor`, `ParFoldable`, `ParFilterable`          |
| `ParRef*`         | `ParRefFunctor`, `ParRefFoldable`, `ParRefFilterable` |

### 8.2 Method naming patterns

| Pattern            | Examples                                                |
| ------------------ | ------------------------------------------------------- |
| `ref_*`            | `ref_map`, `ref_fold_map`, `ref_pure`                   |
| `send_ref_*`       | `send_ref_map`, `send_ref_fold_map`, `send_ref_pure`    |
| `par_*` (by-value) | `par_map`, `par_fold_map`, `par_filter_map`             |
| `par_ref_*`        | `par_ref_map`, `par_ref_fold_map`, `par_ref_filter_map` |

### 8.3 Assessment

The naming is consistent within each family and follows a clear compositional pattern:

- Prefix `ref_` indicates by-reference element access.
- Prefix `send_` indicates `Send + Sync` bounds.
- Prefix `par_` indicates parallel execution.
- These compose: `par_ref_` = parallel + by-reference.

**Finding:** There is no `send_par_ref_*` family (parallel + send + by-reference), but this is unnecessary because ParRef traits already require `Send + Sync`.

**Minor inconsistency:** Free function names follow the pattern `send_ref_map`, `par_ref_map`, etc. But the dispatch system only handles `Val`/`Ref` dispatching (for `map`, `bind`, `lift2`, etc.). There is no dispatch for `send_ref_map` or `par_ref_map`; these remain as separate free functions. This is consistent with the plan's note that dispatch is only meaningful when a closure argument disambiguates the mode.

## 9. Stale Documentation

### 9.1 FnOnce doc comment in SendRefFunctor

The `SendRefFunctor` trait documentation (lines 79-84) contains a "Why `FnOnce`?" section that explains why the closure takes `FnOnce`. However, the diff shows the closure was changed from `FnOnce` to `Fn` (step 3 of the plan). The documentation was not updated to reflect this change.

**Recommendation:** Remove or rewrite the "Why `FnOnce`?" section to explain why `Fn` is used instead (to support multi-element containers like `Vec`).

## 10. Summary of Findings

### Well-designed aspects

1. The SendRef monadic chain (Functor through Monad) is complete and follows the same blanket-trait pattern as the Ref chain.
2. ParRef traits cover all practically useful operations (functor, foldable, filterable, with index variants).
3. Send + Sync bounds are semantically correct throughout, with the `Sync` asymmetry on closures justified by usage patterns.
4. The `SendEndofunction` type cleanly mirrors `Endofunction` for mutual derivation.
5. Naming conventions are consistent and compositional.

### Issues requiring attention

1. **Stale documentation**: `SendRefFunctor` has a "Why FnOnce?" section that is incorrect after the `FnOnce -> Fn` migration (Severity: Low).
2. **Inconsistent supertrait relationships**: `SendRefFoldable: RefFoldable` but `SendRefFunctor` has no `RefFunctor` supertrait. Either all SendRef traits should require their Ref counterparts, or none should (Severity: Medium, affects consistency but not correctness).
3. **`Clone` bound on `SendRefFoldable::send_ref_fold_map`**: The `A: Clone` bound is needed only for default derivation, not for direct implementations. This leaks implementation details into the trait interface (Severity: Low).

### Intentional gaps (well-justified)

1. No `SendRefFilterable`, `SendRefTraversable`, or `SendRefWitherable` (no thread-safe collection type needs them).
2. No `ParRefTraversable` or `ParRefWitherable` (traversal requires sequential effect processing, conflicting with parallelism).
3. ParRef only implemented for `Vec` and `CatList` (single-element containers get no benefit from parallelism).
4. No dispatch support for SendRef/ParRef operations (no closure argument to disambiguate).

### Potential improvements

1. Add a size threshold to CatList ParRef operations to avoid overhead for small collections.
2. Consider splitting the `Clone` bound on foldable trait methods so that `fold_map` does not require `Clone` when implemented directly.
3. If the trait family count continues to grow, investigate a declarative macro to generate Ref/SendRef/ParRef variants from a single definition.
4. Add `RefFunctor` (and other Ref trait) implementations for `ArcLazy`, enabling a `SendRefFunctor: RefFunctor` supertrait relationship and restoring hierarchy consistency.
