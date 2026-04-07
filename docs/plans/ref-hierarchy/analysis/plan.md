# Ref-Hierarchy Remediation Plan

This plan addresses all issues identified across the seven analysis
documents. Issues are grouped by priority, with alternatives and
trade-offs noted where applicable.

## Priority 1: Correctness Fixes

These items affect correctness, produce misleading test results, or
contain demonstrably wrong documentation.

### 1.1 Fix broken UI tests (3 files)

**Source:** [documentation-and-tests.md](documentation-and-tests.md), section 2.10

**Problem:** `new_send_not_send.rs`, `new_send_not_sync.rs`, and
`rc_fn_not_send.rs` import `SendCloneFn` but use `SendLiftFn` in the
test code. The tests pass because `SendLiftFn` is not in scope (name
resolution error), not because the intended trait bound violation is
caught. The `.stderr` files were updated to match these wrong errors.

**Fix:** Update the imports in each `.rs` file to bring `SendLiftFn`
into scope, then update the `.stderr` files to match the correct
trait-bound-violation error messages. Run `just test` with trybuild
`TRYBUILD=overwrite` to regenerate the expected output.

**Alternatives:** None. This is a straightforward bug.

### 1.2 Fix stale "Why FnOnce?" doc comments (2 files)

**Source:** [ref-trait-hierarchy.md](ref-trait-hierarchy.md), section 1;
[sendref-and-parref-traits.md](sendref-and-parref-traits.md), section 9

**Problem:** `ref_functor.rs` (lines 80-85) and `send_ref_functor.rs`
(lines 79-84) contain a "Why `FnOnce`?" section that describes the old
`FnOnce` design. The signatures were changed to `Fn` in step 3 of the
plan.

**Fix:** Replace the "Why `FnOnce`?" section with "Why `Fn`?" and
explain that `Fn` is needed for multi-element containers like Vec,
which call the closure once per element. Note that the minor loss of
FnOnce-only closures is an acceptable trade-off.

### 1.3 Fix phantom ref_sequence documentation

**Source:** [ref-trait-hierarchy.md](ref-trait-hierarchy.md), section 5

**Problem:** The `RefTraversable` doc comment claims `ref_sequence`
exists with a default implementation derived from `ref_traverse`. This
method does not exist in the codebase.

**Approach A (recommended):** Remove the false claim from the doc
comment. `ref_sequence` is unusual for a Ref trait (the inner values
would be accessed by reference, requiring Clone on `F::Of<A>`), and
no concrete use case has been identified.

**Approach B:** Add `ref_sequence` with a default implementation. This
would require `F::Of<'a, A>: Clone` to clone the inner applicative
values out of their references. The operation is semantically
questionable for the Ref context and adds complexity without clear
benefit.

**Recommendation:** Approach A. Remove the false claim.

### 1.4 Fix stale turbofish in limitations-and-workarounds.md

**Source:** [documentation-and-tests.md](documentation-and-tests.md), section 1.4

**Problem:** Line 16 reads `bind::<OptionBrand, _, _>(f, x)` with 3
type params. Should be `bind::<OptionBrand, _, _, _>(f, x)` with 4 type
params to match the dispatched `bind` signature.

**Fix:** Update the turbofish.

## Priority 2: API Gaps and Inconsistencies

These items affect API completeness, consistency, or usability but do
not cause incorrect behavior.

### 2.1 Add LiftRefFn trait (or coerce_ref_fn)

**Source:** [clonefn-arrow-closuremode.md](clonefn-arrow-closuremode.md),
sections 2, 5, 6

**Problem:** There is no generic way to construct a `CloneFn<Ref>::Of`
value. Users must manually write
`Rc::new(|x: &A| ...) as Rc<dyn Fn(&A) -> B>`, which exposes the
concrete pointer type and defeats the brand abstraction. The `LiftFn`
doc comment references a nonexistent `coerce_ref_fn`.

**Approach A: Add `LiftRefFn: CloneFn<Ref>` trait**

```rust
trait LiftRefFn: CloneFn<Ref> {
    fn new_ref<'a, A: 'a, B: 'a>(
        f: impl 'a + Fn(&A) -> B
    ) -> <Self as CloneFn<Ref>>::Of<'a, A, B>;
}
```

Plus `SendLiftRefFn: SendCloneFn<Ref>` for the Send variant. This
mirrors the `LiftFn`/`SendLiftFn` pattern and would have free
functions `lift_ref_fn_new` and `send_lift_ref_fn_new`.

**Approach B: Add `coerce_ref_fn` to `UnsizedCoercible`**

```rust
fn coerce_ref_fn<'a, A, B>(
    f: impl 'a + Fn(&A) -> B
) -> Self::CloneableOf<'a, dyn 'a + Fn(&A) -> B>;
```

This is the simpler fix and was the original plan. However, it puts
construction on the pointer brand rather than a dedicated trait.

**Approach C: Do nothing**

`ref_apply` is rarely called directly; `ref_lift2` is the common
entry point and does not require wrapped functions. The gap only
matters for direct `ref_apply` usage.

**Recommendation:** Approach A. It is more consistent with the
existing `LiftFn`/`SendLiftFn` pattern and provides a trait-level
abstraction that generic code can use.

**Also fix:** Update the `LiftFn` doc comment to stop referencing
the nonexistent `coerce_ref_fn`. After implementing `LiftRefFn`,
reference that instead.

### 2.2 Standardize dispatch import paths

**Source:** [dispatch-system.md](dispatch-system.md), section 6

**Problem:** Three dispatch sub-modules (`functor.rs`, `semimonad.rs`,
`lift.rs`) reference `Val` and `Ref` via fragile `super::super::Val`
paths, while `foldable.rs` uses
`use crate::classes::dispatch::{Val, Ref}`.

**Fix:** Standardize all sub-modules on
`use crate::classes::dispatch::{Val, Ref}`. Mechanical change, no
semantic impact.

### 2.3 Remove redundant BindFlippedDispatch

**Source:** [dispatch-system.md](dispatch-system.md), section 3

**Problem:** `BindFlippedDispatch` duplicates `BindDispatch` but adds
no dispatch capability. The only difference is argument order in the
free function, which can be handled by the function itself.

**Fix:** Delete `BindFlippedDispatch` and its impls. Redefine
`bind_flipped` to reuse `BindDispatch`:

```rust
pub fn bind_flipped<...>(
    f: impl BindDispatch<..., Marker>,
    ma: ...,
) -> ... {
    f.dispatch_bind(ma)
}
```

Similarly for `ComposeKleisliFlippedDispatch`, which could delegate to
`ComposeKleisliDispatch` by swapping the tuple elements.

**Trade-off:** Slightly less documentation locality (the flipped
variants would reference the non-flipped dispatch trait). The code
savings (~200 lines) outweigh this.

### 2.4 Resolve SendRefFoldable supertrait inconsistency

**Source:** [sendref-and-parref-traits.md](sendref-and-parref-traits.md),
section 2

**Problem:** `SendRefFoldable: RefFoldable` requires its Ref
counterpart as a supertrait, but `SendRefFunctor`, `SendRefPointed`,
`SendRefLift`, `SendRefSemimonad` do not require their Ref
counterparts. This inconsistency is confusing.

**Root cause:** `ArcLazy` does not implement `RefFunctor` (only
`SendRefFunctor`) because `RefFunctor::ref_map` does not require
`Send` on closures, but `ArcLazy::new` needs `Send`. Adding
`RefFunctor` to `ArcLazy` would require accepting non-`Send` closures,
which the `ArcLazy` internals cannot support.

**Approach A (recommended): Remove `RefFoldable` supertrait from
`SendRefFoldable`.**

This makes the SendRef family internally consistent: no SendRef trait
requires its Ref counterpart. The mutual derivation code in
`SendRefFoldable` would need to be self-contained rather than
delegating to `RefFoldable` methods.

**Approach B: Add `RefFunctor` (and other Ref traits) to `ArcLazy`,
then add Ref supertraits to all SendRef traits.**

Investigation revealed this is **not feasible**. `RefFunctor::ref_map`
for `ArcLazy` must construct a new `ArcLazy` internally (via
`ArcLazy::new`). `ArcLazy::new` requires the closure to be `Send`
because the closure is stored inside `Arc<LazyLock<...>>`, which must
be `Send + Sync`. But `RefFunctor::ref_map`'s signature only requires
`impl Fn(&A) -> B + 'a`, with no `Send` bound. So `ArcLazy` literally
cannot implement `RefFunctor`.

The same obstacle blocks `RefPointed`, `RefLift`,
`RefSemiapplicative`, and `RefFunctorWithIndex` for `ArcLazy`; all
of these construct new `ArcLazy` values internally.

The distinction is between traits that **construct** new containers
(functor, pointed, lift, applicative) and traits that **only consume**
containers (foldable, semimonad). Only the latter can have the
supertrait relationship.

This also explains why `ParRefFunctor: RefFunctor` works: `Vec` and
`CatList` constructors have no `Send` requirements, so they can
implement `RefFunctor` freely.

**Approach C: Document the inconsistency and leave it.**

The inconsistency does not cause bugs. Add a doc comment explaining
why `SendRefFoldable` has a `RefFoldable` supertrait (it only
consumes, never constructs) and other SendRef traits do not
(`ArcLazy::new` needs `Send` closures that `Ref` trait signatures
cannot guarantee).

**Recommendation:** Approach A, with documentation from Approach C.
Remove the `RefFoldable` supertrait from `SendRefFoldable` for
internal consistency, and add a doc note explaining why SendRef and
Ref hierarchies are independent (unlike ParRef and Ref, which can
share supertraits because collection constructors have no `Send`
requirements).

### 2.5 Unify Setter/IndexedSetter construction method

**Source:** [optics.md](optics.md), section 3

**Problem:** `Setter::evaluate` uses `Arrow::arrow` while
`IndexedSetter::evaluate` and `IndexedSetterPrime::evaluate` use
`LiftFn::new`. Both produce identical results but the semantic
inconsistency is confusing.

**Fix:** Update `IndexedSetter::evaluate` and
`IndexedSetterPrime::evaluate` to use `Arrow::arrow`, matching the
Setter convention. `Arrow::arrow` is more semantically appropriate for
optic evaluation results (these are arrows, not applicative functions).

### 2.6 Add 'a lifetime bound to SendCloneFn::Of

**Source:** [clonefn-arrow-closuremode.md](clonefn-arrow-closuremode.md),
section 9

**Problem:** `CloneFn::Of` has an explicit `'a` bound
(`type Of<...>: 'a + Clone + Deref<...>`), but `SendCloneFn::Of` is
missing it (`type Of<...>: Clone + Send + Sync + Deref<...>`). In
generic code, this means a caller with `Brand: SendCloneFn` cannot
assume `Brand::Of<'a, A, B>: 'a` without an explicit where clause.

**Fix:** Add `'a` to `SendCloneFn::Of` bounds:

```rust
type Of<'a, A: 'a, B: 'a>: 'a + Clone + Send + Sync
    + Deref<Target = Mode::SendTarget<'a, A, B>>;
```

Breaking change: any custom `SendCloneFn` impls must add the `'a`
bound. Since `FnBrand<P>` is the only implementor, this is
straightforward.

### 2.7 Tighten Closed trait to require LiftFn

**Source:** [optics.md](optics.md), section 2.4

**Problem:** `Closed` accepts `CloneFn` but all implementations need
`LiftFn` because `Closed::closed` constructs wrapped functions.

**Fix:** Change `Closed<FunctionBrand: CloneFn>` to
`Closed<FunctionBrand: LiftFn>`. This is a breaking change but makes
the trait bound honest and prevents confusing error messages.

## Priority 3: Coverage Gaps

These items improve completeness and consistency but are not blocking.

### 3.1 Add Ref trait impls for Result applied brands

**Source:** [type-implementations.md](type-implementations.md), section 3.2

**Problem:** `ResultErrAppliedBrand<E>` and `ResultOkAppliedBrand<OK>`
have full by-value traits but no Ref variants. `Result` is commonly
used in generic code.

**Fix:** Implement RefFunctor, RefFoldable, RefTraversable, RefPointed,
RefLift, RefSemiapplicative, RefSemimonad for both applied brands,
following the same pattern as Option. Also add RefFilterable and
RefWitherable for `ResultOkAppliedBrand`.

### 3.2 Add Ref trait impls for Pair, Tuple1, Tuple2

**Source:** [type-implementations.md](type-implementations.md),
sections 3.1, 3.3

**Problem:** These applied brands have full by-value traits but no Ref
variants.

**Fix:** Add the same suite of Ref traits as Identity (RefFunctor
through RefSemimonad, RefFoldable, RefTraversable, plus WithIndex
variants). Trivial implementations since these are single-element
containers.

### 3.3 Fix UI tests and add missing test coverage

**Source:** [documentation-and-tests.md](documentation-and-tests.md),
sections 2.1, 2.9, 2.10, 3.2

**Items:**

a. **Macro ref-mode tests:** Add tests for `a_do!` ref mode untyped,
sequence in ref mode, zero-bind ref mode, and collection types in
ref mode. (Currently all ref-mode tests are in `lazy.rs` only.)

b. **RefApplicative/RefMonad law tests:** Add quickcheck property tests
for right identity, monad associativity, applicative identity,
applicative composition, and applicative homomorphism for the Ref
hierarchy.

c. **RefTraversable/RefLift law tests:** Add quickcheck property tests
for traversal identity and naturality, and `ref_lift2` laws.

d. **Ref dispatch benchmarks:** Add benchmarks comparing `map`/`bind`/
`lift2` in Val vs Ref dispatch to verify zero overhead.

### 3.4 Document multi-bind limitation in m_do! macro docs

**Source:** [macros.md](macros.md), section 5

**Problem:** The multi-bind reference capture limitation in ref mode is
documented only in test files and the plan. The `m_do!` macro doc
comment in `lib.rs` does not mention it.

**Fix:** Add a note to the `m_do!` macro doc comment explaining:

- In ref mode, inner closures cannot capture references from outer
  binds.
- Use `let` bindings to dereference/clone values for use in later
  binds.
- Use `a_do!` when binds are independent (no nesting, so no capture
  issue).

### 3.5 Add dispatch for filter_map and traverse

**Source:** [dispatch-system.md](dispatch-system.md), section 7

**Problem:** `filter_map` and `traverse` have both Val and Ref variants
with closure arguments suitable for dispatch, but are not in the
dispatch system.

**Approach A (recommended): Add FilterMapDispatch and TraverseDispatch.**

These are the highest-value missing dispatch operations. Each follows
the same pattern as existing dispatch traits (trait + Val/Ref impls +
free function).

**Approach B: Defer.**

Users can call the specific free functions directly. The ergonomic
benefit is lower than for `map`/`bind`/`fold`, since `filter_map` and
`traverse` are less frequently used.

**Recommendation:** Approach A. The incremental cost is low given the
established dispatch pattern, and the ergonomic benefit is real.

### 3.6 Document Clone requirements on Ref lift3-5 paths

**Source:** [dispatch-system.md](dispatch-system.md), section 9

**Problem:** The Ref impls for `Lift3Dispatch` through `Lift5Dispatch`
require `Clone` on intermediate types (`A`, `B` for lift3; `A`, `B`,
`C` for lift4; etc.) because they build N-ary lifts from binary
`ref_lift2` calls. This is not called out in the free function
documentation.

**Fix:** Add a note to the `lift3`-`lift5` free function docs
explaining that the Ref path requires Clone on intermediate types due
to tuple construction.

## Priority 4: Improvement Opportunities

These items would improve the design but are not necessary for
correctness or consistency.

### 4.1 Consider promoting brand inference POC

**Source:** [dispatch-system.md](dispatch-system.md), section 5

**Problem:** The `DefaultBrand` POC in `dispatch.rs` enables
turbofish-free calls like `map_infer(|x: i32| x * 2, Some(5))`, but
is test-only code.

**Trade-offs:**

- Works well for simple operations (map, bind, lift2) on common types.
- Does not scale to operations with `FnBrand` parameter (fold).
- Orphan rule prevents third-party types from implementing.
- Worth providing as an opt-in convenience alongside turbofish API.

**Recommendation:** Promote as an ergonomic convenience layer in a
future iteration. Not blocking.

### 4.2 Consider RcCoyoneda/ArcCoyoneda RefFunctor

**Source:** [type-implementations.md](type-implementations.md), section 8

These types have `lower_ref` for by-reference lowering. RefFunctor
could map over the lowered result by reference, but this loses the
fusion benefit. Document the trade-off and add if demand emerges.

### 4.3 Validate Arc-based optics with tests

**Source:** [optics.md](optics.md), section 4

Despite full `PointerBrand` parameterization, no tests validate
`ArcFnBrand`-based optics. Add test cases to verify thread-safe
optics work correctly.

### 4.4 Add CatList ParRef size threshold

**Source:** [sendref-and-parref-traits.md](sendref-and-parref-traits.md),
section 5.2

CatList ParRef operations collect to Vec first, potentially negating
parallelism benefits for small collections. A size threshold fallback
to sequential Ref operations would avoid this overhead.

### 4.5 Document RefTraversable naming ambiguity

**Source:** [type-implementations.md](type-implementations.md),
section 5.2

"Ref" in RefTraversable refers to how elements are accessed (by
reference), not to how the container is held (it is still consumed).
This should be clarified in the trait documentation to prevent
confusion.

### 4.6 Clarify todo.md staleness

**Source:** [documentation-and-tests.md](documentation-and-tests.md),
section 1.7

The "Ref impls for collection types" entry is listed as deferred but
plan step 22 is marked done. Update or remove the entry.

## Implementation Order

The recommended implementation order, grouping items that can be done
together:

**Phase 1: Correctness fixes (Priority 1)**

All four items can be done in parallel:

- 1.1 Fix UI tests
- 1.2 Fix stale FnOnce docs
- 1.3 Fix phantom ref_sequence doc
- 1.4 Fix stale turbofish

**Phase 2: API consistency (Priority 2)**

Items 2.2, 2.3, 2.5, 2.6, 2.7 are independent and can be parallelized.
Items 2.1 and 2.4 are more involved and should be done sequentially.

- 2.2 Standardize dispatch imports
- 2.3 Remove BindFlippedDispatch
- 2.5 Unify Setter/IndexedSetter
- 2.6 Add 'a to SendCloneFn::Of
- 2.7 Tighten Closed to require LiftFn
- 2.1 Add LiftRefFn trait
- 2.4 Remove RefFoldable supertrait from SendRefFoldable, add docs

**Phase 3: Coverage gaps (Priority 3)**

- 3.1 Result Ref impls
- 3.2 Pair/Tuple Ref impls
- 3.3 Test coverage
- 3.4 Macro docs
- 3.5 Dispatch for filter_map/traverse
- 3.6 Document lift3-5 Clone requirements

**Phase 4: Improvements (Priority 4)**

Address opportunistically or as follow-up work.
