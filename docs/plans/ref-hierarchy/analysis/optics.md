# Optics System: Ref-Hierarchy Impact Analysis

## Overview

The ref-hierarchy changes touched every optics-related file in the codebase.
The core renames were:

- `CloneableFn` -> `CloneFn` (type-checking trait, no construction)
- `CloneableFn::new` -> `LiftFn::new` (construction split into its own trait)
- `Function` -> `Arrow` (composable function wrappers for the optics system)
- `cloneable_fn_new` -> `lift_fn_new` (free function rename)

Additionally, `CloneFn` gained a `Mode` parameter (`Val` or `Ref`) for the
dispatch system, with `Val` as the default so existing code is unchanged.

This document analyzes whether these changes were applied correctly and
identifies design issues, inconsistencies, and improvement opportunities.

---

## 1. Rename Consistency

### 1.1 CloneableFn -> CloneFn / LiftFn

**Finding: Consistent.** All 28 optics source files have been updated. No
instances of `CloneableFn` or `cloneable_fn_new` remain anywhere in the
`fp-library/src/` tree. The old `cloneable_fn.rs` module was deleted.

The rename was split correctly:

- Struct field types use `<FunctionBrand as CloneFn>::Of<...>` (read-only
  access to the associated type).
- Construction calls use `<FunctionBrand as LiftFn>::new(...)`.
- Trait bounds on structs and impls use `LiftFn` when the type needs to
  construct wrapped functions, and `CloneFn` when it only needs the
  associated type.

### 1.2 Function -> Arrow

**Finding: Consistent.** The `Function` trait was removed and replaced by
`Arrow`. The two call sites in the optics system that used `Function::new`
(both in `setter.rs`) were updated to `Arrow::arrow`. The module-level
documentation table in `types/optics.rs` was updated from
`Optic Function s t a b` to `Optic Arrow s t a b`.

### 1.3 Documentation Examples

**Finding: Consistent.** All doc examples across the optics files were
updated to use the new function names (`lift_fn_new`, `LiftFn::new`,
`Arrow::arrow`) and the new import paths (`clone_fn::new as lift_fn_new`
instead of `cloneable_fn::new as cloneable_fn_new`).

---

## 2. CloneFn / LiftFn Trait Split in Optics

### 2.1 The Split Design

The old `CloneableFn` combined three concerns:

1. The associated type `Of<'a, A, B>` for type-checking.
2. The `PointerBrand` associated type.
3. The `new` constructor method.

The new design separates these:

- `CloneFn<Mode>`: Provides `Of<'a, A, B>` and `PointerBrand`. Parameterized
  by `Mode` (`Val` or `Ref`) for the dispatch system.
- `LiftFn: CloneFn<Val>`: Provides the `new` constructor. Only works for
  `Val` mode because construction from `Fn(A) -> B` is specific to by-value
  closures.

### 2.2 Bound Selection in Internal Profunctors

The internal profunctors (Exchange, Market, Shop, Stall, Bazaar, Grating,
Zipping) all require `LiftFn` bounds because they construct wrapped functions
in their `Profunctor::dimap`, `Strong::first`, `Choice::left`, and similar
implementations. This is correct.

**Potential concern:** The `LiftFn` bound is strictly stronger than `CloneFn`.
Any code that only needs to _read_ the `Of` associated type but is constrained
to `LiftFn` because it shares a generic parameter with code that constructs
values may be over-constrained. However, in practice all the internal
profunctors both store and construct wrapped functions, so `LiftFn` is
appropriate for all of them.

### 2.3 GrateOptic Uses CloneFn, Not LiftFn

**Finding: Correct and intentional.** The `GrateOptic` trait in
`classes/optics.rs` uses `CloneFn` as its bound:

```
pub trait GrateOptic<'a, FunctionBrand: CloneFn, S, T, A, B>
```

This is correct because `GrateOptic::evaluate` requires `Closed<FunctionBrand>`,
and `Closed` only requires `CloneFn`, not `LiftFn`. The trait itself does not
construct wrapped functions; it delegates to the `Closed` profunctor
implementation, which handles construction internally.

### 2.4 Closed Trait Uses CloneFn

**Finding: Correct.** The `Closed` trait bound is `CloneFn`:

```
pub trait Closed<FunctionBrand: CloneFn>: Profunctor
```

However, all concrete `Closed` implementations (GratingBrand, ZippingBrand,
TaggedBrand, FnBrand) actually require `LiftFn` in their impl bounds because
they call `LiftFn::new` internally. This means `Closed` could theoretically
accept a `CloneFn` that is not `LiftFn`, but no such implementation would be
possible in practice since `Closed::closed` needs to construct wrapped
functions.

**Design note:** This is a minor trait bound inconsistency. The `Closed` trait
signature permits `CloneFn` but all implementations require `LiftFn`. This
could cause confusing error messages if someone tries to implement `Closed`
for a type with only a `CloneFn` bound. However, tightening the `Closed`
trait to require `LiftFn` would be a breaking change to the public API, and
the current design works correctly for all existing use cases.

---

## 3. Arrow vs LiftFn: The Setter Divergence

### 3.1 Setter Uses Arrow, IndexedSetter Uses LiftFn

**Finding: Inconsistency.** The `Setter::evaluate` method constructs its
result using `<FnBrand<Q> as Arrow>::arrow(...)`, while
`IndexedSetterPrime::evaluate` and `IndexedSetter::evaluate` use
`<FnBrand<Q> as LiftFn>::new(...)`.

Both produce identical results since `Arrow::arrow` and `LiftFn::new` both
delegate to `P::coerce_fn(f)` for `FnBrand<P>`. The semantic distinction is:

- `Arrow` is for composable function wrappers in the profunctor/optics system.
- `LiftFn` is for cloneable function wrappers in applicative contexts.

The Setter's `evaluate` method conceptually constructs an optic result (a
function `S -> T`), which is an arrow operation. The indexed setter constructs
the same thing but calls it through `LiftFn`. Both are technically correct,
but the inconsistency is confusing.

**Recommendation:** Decide on a single convention. Since the optics system is
built on the profunctor/arrow hierarchy, `Arrow::arrow` is more semantically
appropriate for optic evaluation results. The indexed setter should be
updated to use `Arrow::arrow` for consistency.

### 3.2 The `optics::functions` Module

The helper functions `set` and `over` in `types/optics/functions.rs` use
`<FnBrand<PointerBrand> as Arrow>::arrow(...)` to construct the constant/
modifier functions passed to optic evaluation. This is consistent with the
Setter's approach.

---

## 4. Rc Usage and FnBrand\<P\> Parameterization

### 4.1 Current State

The optics system is already fully parameterized over `PointerBrand`/
`FnBrand<P>`. Every optic struct (Lens, Prism, Iso, AffineTraversal, Grate,
Traversal, etc.) takes a `PointerBrand` generic parameter. The internal
profunctors (Exchange, Market, Shop, Stall, Bazaar, Grating) take a
`FunctionBrand: LiftFn` parameter.

Despite this parameterization, all examples and tests use `RcBrand`/
`RcFnBrand` exclusively. There are zero uses of `ArcBrand`/`ArcFnBrand`
anywhere in the optics code.

### 4.2 Opportunities Created by Ref-Hierarchy

The ref-hierarchy changes make the following previously impossible or
difficult things possible:

1. **Send-safe optics.** With `ArcFnBrand`, optics could be shared across
   threads. The parameterization is already in place; only tests and examples
   need to validate this path.

2. **Ref-mode closures in optics.** The `CloneFn<Ref>` mode enables
   by-reference closures. Currently, all optic closures take owned values.
   For large structures, ref-mode lenses (`Lens::from_view_set` with
   `|s: &S| -> A` instead of `|s: S| -> A`) could avoid cloning.

3. **Unified dispatch for optic helpers.** The dispatch system's `map` and
   `bind` free functions could potentially support ref-qualified optic
   traversals, though this would require significant API design work.

### 4.3 Remaining Rc Hard-Coding

**Finding: No Rc hard-coding in optic struct definitions.** The structs are
fully generic. However, the `Setter` type has a `Box<dyn Fn(A) -> B>`
hard-coded in its constructor signature:

```
pub fn new(over: impl 'a + Fn((S, Box<dyn Fn(A) -> B + 'a>)) -> T) -> Self
```

This `Box<dyn Fn>` is independent of the pointer brand and cannot be
parameterized without changing the Setter's fundamental design. This is a
known limitation, not a regression from the ref-hierarchy changes.

### 4.4 Concrete Rc in Grate

The `Grate` type's field types involve deeply nested `CloneFn::Of` types:

```
pub grate: <FnBrand<PointerBrand> as CloneFn>::Of<
    'a,
    <FnBrand<PointerBrand> as CloneFn>::Of<
        'a,
        <FnBrand<PointerBrand> as CloneFn>::Of<
            'a,
            <PointerBrand as RefCountedPointer>::CloneableOf<'a, S>,
            A,
        >,
        B,
    >,
    T,
>
```

While this is correctly parameterized, the triple nesting of `CloneFn::Of`
makes the type extremely verbose. A type alias like
`type GrateFn<P, S, A, B, T> = ...` would improve readability. This
predates the ref-hierarchy changes but the rename from `CloneableFn::Of` to
`CloneFn::Of` did not worsen it.

---

## 5. Profunctor Trait Bounds After Restructuring

### 5.1 Profunctor, Strong, Choice

**Finding: Unchanged and correct.** These traits have no bounds related to
`CloneFn` or `Arrow`. They operate on the profunctor's own associated types,
not on function wrappers directly. The ref-hierarchy changes did not affect
their definitions.

### 5.2 Closed

**Finding: Correctly updated.** The `Closed` trait's `FunctionBrand` parameter
was updated from `CloneableFn` to `CloneFn`. As discussed in section 2.4,
this is technically a weaker bound than needed, but it works because all
implementors provide `LiftFn`.

### 5.3 Wander

**Finding: Unchanged.** The `Wander` trait (used for traversals) does not
depend on function wrapper traits directly. It requires `Strong + Choice +
Profunctor`, all of which are correctly bounded.

---

## 6. Ref Dispatch System and Optics

### 6.1 Current Integration

The optics system does not use the dispatch system at all. All optic
operations take by-value closures. The `FunctionBrand` parameter in optics
traits defaults to or is constrained to `Val`-mode `CloneFn`.

### 6.2 Potential Benefits

**Ref-mode lenses and traversals.** The most impactful opportunity is
by-reference traversal. Currently, `over` requires:

```
optic.evaluate(f)  // f: Rc<dyn Fn(A) -> B>
```

With ref-mode support, this could become:

```
optic.evaluate(f)  // f: Rc<dyn Fn(&A) -> B>
```

This would eliminate cloning of focus values during traversal. However, this
requires:

1. A `RefOptic` trait (or parameterization of `Optic` by closure mode).
2. Internal profunctors that can work with `Fn(&A) -> B` closures.
3. Modified `Strong` and `Choice` implementations that handle references.

This is a substantial design effort that goes beyond the current ref-hierarchy
scope.

### 6.3 Recommendation

The dispatch system's `Ref` mode is not yet useful for the optics system.
The optics system should remain on `Val` mode until a dedicated ref-optics
design is undertaken. The ref-hierarchy changes have laid the groundwork by
making `CloneFn` mode-parameterized, but the optics-specific work is a
separate project.

---

## 7. Regressions and Missing Updates

### 7.1 No Regressions Found

All renames were applied consistently. The test suite (doc tests, unit tests,
property tests) was updated with the new function names. No stale references
to `CloneableFn`, `cloneable_fn_new`, or `Function` remain.

### 7.2 Import Cleanup

Several files switched from importing specific items (`CloneableFn`,
`UnsizedCoercible`) to glob imports (`*`). For example:

```diff
-use crate::classes::{CloneableFn, UnsizedCoercible, ...};
+use crate::classes::{*, ...};
```

This is a minor style change. Glob imports are convenient but can mask
dependencies and make it harder to find where a trait comes from. The project
already uses per-line imports (`imports_granularity = "One"`) for top-level
modules, but inner module imports are less strict. This is not a regression
but is worth noting for future cleanup.

### 7.3 IndexedFoldFunc Gained a FnBrand Parameter

The `IndexedFoldFunc` trait gained an additional type parameter:

```diff
-pub trait IndexedFoldFunc<'a, I, S, A> {
+pub trait IndexedFoldFunc<'a, I, S, A, FnBrand: LiftFn + 'a = crate::brands::RcFnBrand> {
```

This was necessary because `FoldableWithIndex::fold_map_with_index` gained
a `FnBrand` parameter in the ref-hierarchy changes (for the dispatch system).
The default is `RcFnBrand`, preserving backward compatibility.

**Concern:** The default `RcFnBrand` hard-codes a pointer choice at the trait
level. Code that uses `ArcFnBrand` must specify the parameter explicitly. This
is acceptable for now but should be revisited if thread-safe optics become a
priority.

---

## 8. Summary of Findings

### Correctly Applied

- All renames (CloneableFn -> CloneFn/LiftFn, Function -> Arrow) are
  complete and consistent across all 28 optics files.
- No stale references remain.
- Trait bound selection (CloneFn vs LiftFn) is appropriate for each use site.
- The profunctor hierarchy (Profunctor, Strong, Choice, Closed, Wander) has
  correct bounds after the restructuring.
- The FnBrand\<P\> parameterization remains fully intact.

### Inconsistencies

1. **Setter vs IndexedSetter construction method.** `Setter::evaluate` uses
   `Arrow::arrow` while `IndexedSetter::evaluate` uses `LiftFn::new`. Both
   produce identical results but the inconsistency is confusing. Pick one
   convention.

2. **Closed trait bound is weaker than implementations require.** `Closed`
   accepts `CloneFn` but all implementations need `LiftFn`. This could cause
   confusing error messages for downstream implementors.

3. **Import style inconsistency.** Some files use glob imports (`*`) while
   others import specific items. The glob imports were introduced during this
   refactor as a convenience.

### Limitations

1. **No ref-mode optics.** The dispatch system's `Ref` mode is not used by the
   optics system. Ref-mode lenses/traversals would require a separate design
   effort.

2. **No Arc-based optics tested.** Despite full parameterization, no tests
   validate `ArcFnBrand`-based optics. Thread-safe optics are theoretically
   possible but unvalidated.

3. **IndexedFoldFunc defaults to RcFnBrand.** This hard-codes a pointer choice
   at the trait level.

4. **Setter uses Box\<dyn Fn\>.** The `Setter` type's constructor takes
   `Box<dyn Fn(A) -> B>`, which cannot be parameterized by the pointer brand.
   This is a pre-existing limitation, not a regression.

### Improvement Opportunities

1. **Validate Arc-based optics** by adding test cases with `ArcFnBrand`.
2. **Unify Setter/IndexedSetter construction** to use a single method
   (`Arrow::arrow` recommended).
3. **Add type aliases for deeply nested Grate types** to improve readability.
4. **Consider tightening Closed to require LiftFn** if backward compatibility
   permits.
5. **Design ref-mode optics** as a follow-up project, leveraging the
   `CloneFn<Ref>` infrastructure.
