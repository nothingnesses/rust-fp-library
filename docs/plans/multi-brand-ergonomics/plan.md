# Plan: Multi-Brand Ergonomics via InferableBrand Trait

**Status:** IN PROGRESS

## Current progress

Phase 0 complete. Phase 1 step 0 complete:

- Added `tests/non_regression_single_brand.rs` with 16 tests covering
  map, bind, join, filter, lift2, alt inference wrappers for
  single-brand types (Option, Vec). All pass against the current
  codebase.
- Added `tests/multi_brand_integration.rs` with phase 1 and phase 2
  tests, all `#[ignore]`d with stub bodies (actual calls commented
  out since they cannot compile until the Slot infrastructure exists).
- Compile-fail UI test stubs deferred to phase 1 step 6 (see
  Deviations).

Phase 1 step 1 complete: added `SLOT_PREFIX` constant to
`fp-macros/src/core/constants.rs` and updated `classify_trait`,
`is_semantic_type_class`, and `is_dispatch_container_param` to
recognise `Slot_`-prefixed traits.

Phase 1 steps 2-3 complete: added `generate_slot_name` to canonicalizer
and updated `trait_kind_worker` to emit `Slot_{hash}` alongside
`Kind_{hash}` and `InferableBrand_{hash}`. The Slot trait has Brand as
a trait parameter and `type Marker` as its associated type. A `&T`
blanket sets `Marker = crate::dispatch::Ref`. Added dispatch shim
modules to fp-macros test files to satisfy the `crate::dispatch::Ref`
path in test contexts.

Phase 1 step 4 complete: updated `impl_kind!` to emit direct Slot impls
with `type Marker = Val` for every brand. Renamed `#[no_inferable_brand]`
to `#[multi_brand]` across all source files and test files. Switched
the projection skip rule from string heuristic to structural AST checks
(Decision S). Updated fp-macros UI test `.stderr` snapshot. Marked
`hkt.md` doctest as `ignore` since `impl_kind!` Slot output references
`crate::dispatch::Val` which doesn't resolve in doctest context.

Phase 1 step 5 complete: rewrote the `map` inference wrapper in
`dispatch/functor.rs` to bind on Slot with Marker projected from
`<FA as Slot<Brand, A>>::Marker`. Brand is now a type parameter
(replacing the old Marker type parameter), and the closure's input
type disambiguates Brand for multi-brand types. The `explicit::map`
function is left unchanged (it already takes Brand via turbofish and
has no InferableBrand dependency). Removed `result_no_inferable_brand`
compile-fail test since `map` now accepts Result via Slot. Updated
`tuple2_no_inferable_brand` stderr snapshot (the test still fails for
diagonal `(i32, i32)` but with different error messages).

Phase 1 step 5 complete: rewrote the `map` inference wrapper in
`dispatch/functor.rs` to bind on Slot with Marker projected. The
inference wrapper uses `Brand` as a type parameter (replacing the
old `Marker` param), resolved via closure-directed inference.
`explicit::map` is left unchanged (no Slot bound, original 5-param
signature). See revised Decisions F and Q for rationale. Removed
`result_no_inferable_brand` compile-fail test since `map` now
accepts Result via Slot.

Phase 1 step 6 complete: added three compile-fail UI tests
(`multi_brand_diagonal.rs`, `multi_brand_unannotated.rs`,
`double_ref.rs`) with generated `.stderr` snapshots. The existing
`tuple2_no_inferable_brand.rs` test is retained (diagonal case for
tuples). The `result_no_inferable_brand.rs` test was removed in
step 5 (map now accepts Result via Slot).

Phase 1 complete. All 11 phase 1 multi-brand integration tests pass
(Val + Ref for Ok/Err mapping, passthrough, generic fixed params,
both-params-generic, Ref + generic fixed param). All 16 non-regression
single-brand tests remain green. 9 phase 2 tests correctly ignored.

Phase 2 dispatch migration complete. All 19 dispatch modules migrated
from InferableBrand to Slot (34 inference wrappers total). No
InferableBrand references remain in dispatch/. Explicit functions
unchanged per revised Decision Q. All 20 integration tests pass
(11 phase 1 + 9 phase 2). 16 non-regression tests green.

Remaining phase 2 items:

- Do-notation audit (Decision K) - next.
- `dispatch/semiapplicative.rs` (Decision P) - new module, deferred
  until after the do-notation audit since `a_do!` uses apply/lift
  for multi-bind expressions and audit findings could influence the
  dispatch module design.

## Open questions, issues and blockers

None at this time. Previously resolved:

1. Closureless Slot inference validated by `slot_closureless_poc.rs`
   (see Decision W).
2. `crate::dispatch::{Val, Ref}` hardcoding resolved by switching to
   absolute paths `::fp_library::dispatch::{Val, Ref}` with
   `extern crate self as fp_library;` in fp-library's lib.rs and
   fp-library as a dev-dependency of fp-macros. See deviation 9.

## Deviations

1. Compile-fail UI test stubs deferred from phase 1 step 0 to
   step 6. The trybuild runner uses `tests/ui/*.rs` as a glob, so
   any file placed there must be a valid compile-fail test with
   matching `.stderr`. Adding stubs before the Slot infrastructure
   exists would produce incorrect error messages.
2. Multi-brand integration tests use empty stub bodies with the
   intended calls as comments, rather than `#[ignore]`d tests with
   actual code. The actual `map(...)` calls cannot compile against
   the current InferableBrand-based dispatch because Result does not
   implement InferableBrand at arity 1. Tests will be filled in and
   un-ignored in step 7.
3. Steps 2 and 3 were combined. Both concern Slot trait generation
   (step 2: add traits to kinds.rs; step 3: update trait_kind! to
   emit them). Since kinds.rs uses trait_kind!, the traits are
   generated by the macro, not hand-written. Combining avoids
   intermediate manual trait definitions.
4. document_module_tests.rs test_cfg_no_conflict module needed a
   manual `Slot_ad6c20556a82a1f0` trait definition alongside its
   existing manual `Kind_ad6c20556a82a1f0` definition.
5. hkt.md doctest remains `rust,ignore` because `impl_kind!` for
   `Option<A>` violates the orphan rule in a doctest context (the
   generated trait impls are for a type not defined in the doctest
   crate).
6. fp-macros UI test `invalid_assoc_type_name.stderr` updated: now
   includes Slot and dispatch resolution errors in addition to the
   existing Kind and InferableBrand errors.
7. Decisions F and Q revised during implementation. The original
   plan required Slot bounds on all explicit functions, but this was
   found to be incompatible with Decision G (projection brands lack
   Slot impls and would lose explicit:: access). Explicit functions
   now keep their original signatures; only inference wrappers use
   Slot. See revised Decisions F and Q for details.
8. Macro output switched from `crate::dispatch::{Val, Ref}` to
   `::fp_library::dispatch::{Val, Ref}` (absolute crate path) so
   that `trait_kind!` and `impl_kind!` work in external crates.
   Added `extern crate self as fp_library;` to fp-library's lib.rs
   for self-resolution, and fp-library as a dev-dependency of
   fp-macros so its tests resolve the absolute path. Dispatch shim
   modules removed from fp-macros test files.
9. `document_module` macro false-positive: the named-generics
   validation incorrectly suggests `impl Trait` for `FA` in `join`
   when `FA` is used in where-clause projections (`<FA as
Slot<...>>::Marker`). Suppressed with `#[allow_named_generics]`.
   Two issues to fix separately:
   a) The validation counts only parameter-position uses of a type
   param, not where-clause uses. When a type param appears once
   in parameters but also in where-clause projections, it cannot
   use `impl Trait` but the macro still warns.
   b) The warning mechanism uses `#[deprecated]` to emit diagnostics,
   which causes a cascading `let_unit_value` clippy lint that
   reports "this let-binding has unit value" at the function
   signature, with no connection to any actual let-binding.
   To reproduce: remove `#[allow_named_generics]` from the `join`
   inference wrapper in `dispatch/semimonad.rs` and run `just clippy`.
   Both the false-positive deprecation warning and the spurious
   let-binding lint will appear at the `pub fn join` line.

## Implementation protocol

After completing each step within a phase:

1. Stage all changes (`git add`) so the test output cache is
   invalidated (the cache keys on `git ls-files` content hashes).
2. Run verification: `just fmt`, `just check`, `just clippy`,
   `just deny`, `just doc`, `just test` (or `just verify` which
   runs all six in order).
3. If verification passes, update the `Current progress`, `Open
questions, issues and blockers`, and `Deviations` sections at
   the top of this plan to reflect the current state.
4. Commit the step (including the plan updates).

---

Replace the existing `InferableBrand_*` trait family with a redesigned
version that supports closure-directed inference for multi-brand
concrete types (`Result`, `Pair`, `Tuple2`, `ControlFlow`, `TryThunk`)
while preserving the unified Val/Ref dispatch users have today. The
new trait has Brand as a trait parameter (not an associated type) and
carries a Marker associated type for Val/Ref dispatch.

## API stability stance

`fp-library` is pre-1.0. API-breaking changes are acceptable when they
lead to a better end state. This plan prioritises design correctness
and internal coherence over preserving compatibility with the current
public surface.

- Renaming, reshaping, or removing existing `explicit::` signatures,
  free functions, macros, or attributes is acceptable.
- Public macro attributes (e.g., `#[no_inferable_brand]`) can change
  name and semantics in one release; a changelog entry is sufficient.
- Call sites in doctests, UI tests, and user code can require updates.
  Breakage is documented and mass-updated in the release that ships
  the change.

## Motivation

Today's `InferableBrand`-based `map` refuses multi-brand types:

```rust
// Today: multi-brand requires explicit turbofish
explicit::map::<ResultErrAppliedBrand<String>, _, _, _, _>(
    |x: i32| x + 1,
    Ok::<i32, String>(5),
)
```

After this plan:

```rust
// After: multi-brand inference via closure-directed resolution
map(|x: i32| x + 1, Ok::<i32, String>(5))         // Ok-mapping
map(|e: String| e.len(), Err::<i32, String>("hi".into()))  // Err-mapping
```

Both directions work because the closure's input type disambiguates
which brand applies. Single-brand calls continue to work as today.

## Design

### The `InferableBrand` trait family

One trait per Kind arity (same pattern as today's `Kind_*` and
`InferableBrand_*` families):

```rust
pub trait InferableBrand_cdc7cd43dac7585f<'a, Brand, A: 'a>
where
    Brand: Kind_cdc7cd43dac7585f,
{
    type Marker;
}
```

Two design properties carry the full weight:

1. **Brand is a trait parameter** (not an associated type). This lets
   multiple impls per concrete type coexist under coherence, each
   keyed on a distinct Brand value. Coherence treats
   `InferableBrand<ResultErrAppliedBrand<E>, A>` and `InferableBrand<ResultOkAppliedBrand<T>, A>`
   as structurally distinct trait heads even when both cover
   `Result<_, _>`.
2. **Marker is an associated type** projected from FA's reference-ness.
   Direct impls for owned types set `type Marker = Val`; a single
   `&T` blanket sets `type Marker = Ref` uniformly. When dispatch
   code projects `<FA as InferableBrand<...>>::Marker`, the Marker commits from
   FA alone, before `(Brand, A)` are resolved - eliminating the
   Val/Ref cross-competition that otherwise blocks Ref + multi-brand
   inference.

### Impl landscape

Every brand gets a direct `InferableBrand` impl. No blanket from the
old `InferableBrand` (that combination fails coherence; see POC 3).

Single-brand types have one impl per arity:

```rust
impl<'a, A: 'a> InferableBrand_*<'a, OptionBrand, A> for Option<A> {
    type Marker = Val;
}
```

Multi-brand types have one impl per brand:

```rust
impl<'a, A: 'a, E: 'static> InferableBrand_*<'a, ResultErrAppliedBrand<E>, A>
    for Result<A, E>
{
    type Marker = Val;
}

impl<'a, T: 'static, A: 'a> InferableBrand_*<'a, ResultOkAppliedBrand<T>, A>
    for Result<T, A>
{
    type Marker = Val;
}
```

The reference blanket:

```rust
impl<'a, T: ?Sized, Brand, A: 'a> InferableBrand_*<'a, Brand, A> for &T
where
    T: InferableBrand_*<'a, Brand, A>,
    Brand: Kind_*,
{
    type Marker = Ref;
}
```

### The unified inference wrapper

`map` (and sibling closure-taking operations) binds on `InferableBrand` with
`Marker` projected:

```rust
pub fn map<'a, FA, A: 'a, B: 'a, Brand>(
    f: impl FunctorDispatch<
        'a,
        Brand,
        A,
        B,
        FA,
        <FA as InferableBrand_*<'a, Brand, A>>::Marker,
    >,
    fa: FA,
) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
    Brand: Kind_*,
    FA: InferableBrand_*<'a, Brand, A>,
```

### Coverage matrix

| Case                              | Behaviour                                   |
| --------------------------------- | ------------------------------------------- |
| Val + single-brand                | Inference (no change from today)            |
| Val + multi-brand                 | Inference via closure input                 |
| Ref + single-brand                | Inference (no change from today)            |
| Ref + multi-brand                 | Inference via closure input                 |
| Multi-brand + generic fixed param | Inference works (POC 9 validated)           |
| Multi-brand diagonal (`T=T`)      | Compile error; use `explicit::`             |
| Unannotated multi-brand           | Compile error; annotate or use `explicit::` |

### How closure-directed inference resolves Brand

For `map(|x: i32| x + 1, Ok::<i32, String>(5))`:

1. `FA = Result<i32, String>` pinned by the argument.
2. `Marker` projected via InferableBrand: Result is owned, so Marker = Val.
3. With Marker committed, FunctorDispatch picks the Val impl. Its
   `Fn(A) -> B` bound pins `A = i32` from the closure.
4. With `A = i32`, only the `ResultErrAppliedBrand<String>` InferableBrand
   impl unifies with FA = `Result<i32, String>`. Brand commits.
5. Dispatch proceeds.

For `&Result<i32, String>` with `|x: &i32| *x + 1`:

1. `FA = &Result<i32, String>`.
2. The `&T` blanket projects Marker = Ref immediately.
3. FunctorDispatch Ref impl applies; `Fn(&A) -> B` pins A from `&i32`.
4. Inner InferableBrand impl on `Result<i32, String>` resolves to
   `ResultErrAppliedBrand<String>` with A = i32.
5. Dispatch proceeds through `RefFunctor::ref_map`.

### Replacement of today's `InferableBrand`

Under Decision D, today's `InferableBrand_*` (which has Brand as
an associated type and requires a unique brand per concrete type) is
replaced entirely by the new `InferableBrand_*` (which has Brand as
a trait parameter and supports multiple brands per concrete type).

The old `<FA as InferableBrand>::Brand` associated-type projection
is replaced by an explicit Brand type parameter resolved via trait
selection. For single-brand types, the single impl resolves Brand
uniquely (no closure needed). For multi-brand types, the closure's
input type disambiguates.

The `#[no_inferable_brand]` attribute is renamed to `#[multi_brand]`
(Decision E). Multi-brand types get multiple InferableBrand impls
(one per brand). Single-brand types get one.

## Validated via POCs

Nine POCs on stable rustc establish feasibility:

| POC                                                                                          | Finding                                                                                                                                                                                         |
| -------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [slot_production_poc.rs](../../../fp-library/tests/slot_production_poc.rs)                   | InferableBrand type-level validation; A2 coherence works; lifetime-generic GATs OK.                                                                                                             |
| [slot_valref_poc.rs](../../../fp-library/tests/slot_valref_poc.rs)                           | Unified signature via InferableBrand + production FunctorDispatch - validated for Val + all, Ref + single-brand. Ref + multi-brand exposed Val/Ref cross-competition.                           |
| [slot_select_brand_poc.rs](../../../fp-library/tests/slot_select_brand_poc.rs)               | Alternative with Brand as associated-type projection - rejected by coherence.                                                                                                                   |
| [slot_assoc_marker_poc.rs](../../../fp-library/tests/slot_assoc_marker_poc.rs)               | Alternative with Marker as dispatch-trait associated type - rejected by coherence.                                                                                                              |
| [slot_marker_via_slot_poc.rs](../../../fp-library/tests/slot_marker_via_slot_poc.rs)         | **Adopted design.** Marker projected via InferableBrand closes Ref + multi-brand gap in unified signature.                                                                                      |
| [slot_arity2_poc.rs](../../../fp-library/tests/slot_arity2_poc.rs)                           | Pattern generalises to arity 2 (bimap).                                                                                                                                                         |
| [slot_bind_poc.rs](../../../fp-library/tests/slot_bind_poc.rs)                               | Pattern generalises to `bind` (closure returns container); single-brand and multi-brand both work.                                                                                              |
| [slot_apply_poc.rs](../../../fp-library/tests/slot_apply_poc.rs)                             | Pattern generalises to `apply` and `ref_apply` (two containers sharing a Brand); Brand inferred from `ff`'s Fn-payload and `fa`'s value simultaneously; multi-brand works for both Val and Ref. |
| [slot_generic_fixed_param_poc.rs](../../../fp-library/tests/slot_generic_fixed_param_poc.rs) | Generic fixed parameters (`fn process<E>(r: Result<i32, E>)`) infer correctly; the solver commits Brand from the concrete closure input without needing to prove the generic param differs.     |

Key generalisation findings:

- Trait family works across Kind arities (POC 6).
- Pattern works across closure shapes: `Fn(A) -> B` for Functor and
  `Fn(A) -> Of<B>` for Semimonad (POC 7). `apply` has no direct
  closure but works via the Fn payload inside `ff` (POC 8).
- Multi-brand coverage includes `Result` via `ResultErrAppliedBrand<E>`
  for Functor, Semimonad, and Semiapplicative (the latter two already
  existed in the library matching PureScript's `Bind (Either e)` and
  `Apply (Either e)`).
- Two InferableBrand bounds sharing the same Brand parameter (POC 8) resolve
  correctly via the solver - Brand commits from the intersection of
  both bounds.

## Decisions

### Decision A: Impl layout

**A2** (adopted). Every brand gets a direct `InferableBrand` impl. No
blanket from the old InferableBrand trait.

_Rationale:_ POC 3 demonstrated that a blanket bridging the old
associated-type-based InferableBrand to the new trait-parameter-based
InferableBrand conflicts with direct multi-brand impls via E0119.
Rust's coherence checker cannot prove non-overlap through
where-clauses. Direct impls per brand are trivially coherence-safe
because their trait-argument patterns differ by Brand.

### Decision B: Phase packaging

**B3** (recommended). Phases are implemented sequentially internally
but released together.

_Rationale:_ Pre-1.0 stance removes the "bundled release is too risky"
argument. Internal phasing gives a testbed for each operation while
users see only a coherent shipped API with all closure-taking
operations consistently supporting multi-brand inference.

### Decision C: Closure annotation UX

**C1** (accepted). Document that multi-brand types require
closure-input annotations under the inference path; `explicit::`
remains the no-annotation alternative.

_Rationale:_ The requirement follows directly from how closure-directed
inference works. Documenting prominently is the only stable-Rust
option; there is no alternative signature shape that removes the
requirement.

### Decision D: Trait consolidation

Eliminate the current `InferableBrand_*` entirely. Replace it with
a new `InferableBrand_*` trait family that has Brand and A as trait
parameters plus an associated `type Marker`:

```rust
trait InferableBrand<'a, Brand: Kind, A: 'a> {
    type Marker;
}
```

_Rationale:_ The review's H1 finding (that removing InferableBrand
breaks `pure`/`empty`) is invalid. Investigation shows that `pure`
and `empty` are defined in `classes/pointed.rs` and `classes/plus.rs`
as simple free functions taking Brand via turbofish
(`pure::<OptionBrand, _>(5)`). They have no dispatch trait, no
InferableBrand dependency, and are unaffected by this change.

Non-closure dispatch operations (`join`, `alt`, `apply_first`,
`apply_second`) DO use InferableBrand today, but they take FA from
an argument (not a return type). The new InferableBrand handles
these correctly: for single-brand types, the single impl resolves
Brand uniquely from FA alone, without needing a closure. For
multi-brand types, the result is the same as today (ambiguous;
use `explicit::`). Brand appears as a type parameter in the return
type, replacing the old `<FA as InferableBrand>::Brand` projection.

Implementation: bulk-rename the current `InferableBrand` references
to the new trait's hash via `sed`, then introduce the new trait
shape. The old trait family is fully removed. POCs 5, 6, 7, 8
validate the new trait's shape for `map`, `bimap`, `bind`, and
`apply`.

### Decision E: Attribute naming

Rename `#[no_inferable_brand]` to **`#[multi_brand]`**.

_Rationale:_ Under Decision D, types with this attribute have
InferableBrand impls (so they ARE inferable via closure direction)
but have multiple impls (so no unique brand). The new
name describes what is true (multiple brands) rather than what is
no longer accurate. Pre-1.0 stance accepts the breakage.

### Decision F: `explicit::map` shape (revised)

Keep `explicit::map` unchanged: no Slot bound, Marker as a free type
parameter, original turbofish shape `explicit::map::<Brand, _, _, _, _>(f, fa)`.

_Rationale (revised):_ Adding a Slot bound to `explicit::map` is
incompatible with Decision G (projection brands remain explicit-only).
Projection brands do not have Slot impls because generating them
would amplify closure-input ambiguities (see Decision G). If
`explicit::map` required Slot, projection brands could no longer use
the explicit dispatch path, contradicting Decision G's guarantee.

Keeping `explicit::map` free of Slot bounds makes it the true
universal fallback: it works for single-brand, multi-brand, projection
brands, and diagonal cases. The `FunctorDispatch` trait bound already
validates that Brand implements the correct type class. Slot-based
dispatch is confined to the inference wrappers, where it provides
closure-directed Brand resolution.

### Decision G: Projection-type brands

Projection brands (e.g., `BifunctorFirstAppliedBrand<ResultBrand, A>`)
remain explicit-only. `impl_kind!` skips InferableBrand
generation when the brand's `Of` target contains `Apply!` or `::`.

_Rationale:_ These brands exist for architectural completeness
(showing Bifunctor subsumes Functor in one direction), not as primary
user paths. Generating InferableBrand for them would create additional InferableBrand
candidates for types like `Result<A, E>` at arity 1, amplifying
closure-input ambiguities without user-facing benefit.

### Decision H: Apply-side closure-directed inference

Extend InferableBrand-based inference to
`Semiapplicative::apply` and `RefSemiapplicative::ref_apply` via
the Fn payload inside `ff`.

The signature uses two InferableBrand bounds sharing the Brand parameter:

- `FF: InferableBrand<Brand, <FnBrand as CloneFn>::Of<A, B>>` keys on the
  function payload type inside `ff`.
- `FA: InferableBrand<Brand, A>` keys on the value type inside `fa`.

Rust's solver intersects the two bounds to commit a unique Brand.
Both Val and Ref are validated by POC 8 (11 tests: 7 Val + 4 Ref).

_Rationale:_ gives `apply` the same inference experience as `map`
and `bind`. Keeping apply explicit-only for multi-brand would create
surface asymmetry ("why does apply need `explicit::` when map
doesn't?"). Deferring was the original recommendation before POC 8
validated feasibility; now that the Fn-payload approach works,
deferring has no benefit.

### Decision I: Compile-time regression threshold

Measure clean-build wall-clock time before and after implementation.
Investigate only if regression exceeds ~50% of the baseline (~36s).
Below that threshold, accept the regression. Runtime performance
matters more than compile-time for this project.

**Baseline (2026-04-17, commit cc165b2):**

- Command: `cargo build --workspace --all-targets --all-features`
  (clean build after `just clean`).
- Wall-clock: **24.0s** (real), 113.7s (user), 17.9s (sys).
- Toolchain: rustc 1.94.1, debug profile.

If investigation is needed, options include: only generating InferableBrand
for brands that participate in closure-directed dispatch (exclude
tag-only brands like `SendThunkBrand`), or hand-optimizing the
`impl_kind!` expansion.

### Decision J: Diagnostic wording for ambiguity failures

Single combined `#[diagnostic::on_unimplemented]`
message on `InferableBrand`: "annotate the closure input type; if that doesn't
disambiguate, use `explicit::map::<Brand, ...>`."

_Rationale:_ works on stable Rust; simple to implement. The same
static text covers both failure modes (missing annotation and
diagonal case). The annotation hint is slightly misleading in the
diagonal case, but phase 3 can revisit if user testing shows
confusion. Escalating later is easier than pulling back premature
complexity.

If upstream trait bounds (`Brand: Functor`, `Brand: RefFunctor`)
fire before `InferableBrand`'s diagnostic and produce a less helpful message,
phase 3 should add `on_unimplemented` overrides to those type-class
traits pointing users back at the InferableBrand diagnostic. Decide from
observed behaviour during implementation.

### Decision K: Do-notation macro behaviour

Audit `m_do!` and `a_do!` against multi-brand
types before shipping. Run existing `tests/do_notation.rs` and
`tests/ado_notation.rs` with multi-brand containers; add tests for
annotated and unannotated closures; document the annotation
requirement in macro docs.

_Rationale:_ low cost relative to the risk of shipping broken
do-notation. If the audit reveals that the annotation requirement is
severely disruptive in practice, extending the macros to automatically
emit closure-input type annotations in their expansion can be explored
as a follow-up. Note: inferred-mode `m_do!` with multi-brand types
will fail at `pure` (which takes Brand via turbofish), so multi-brand
`m_do!` must use explicit mode (`m_do!(Brand { ... })`). Document
this in the macro docs.

### Decision L: Stale documentation

Rewrite `fp-library/docs/brand-dispatch-traits.md` before
implementation begins. The document currently describes a Slot
design with `type Out<B>` GAT and an InferableBrand blanket, which
contradicts the adopted Marker-only design with direct impls.

_Rationale:_ implementers reading the docs before the code would
get a misleading picture. Update early to avoid confusion.

### Decision M: Marker-agreement invariant

Document the invariant that all InferableBrand impls for a given
Self type must agree on the same Marker value (Val for owned types,
Ref for references). Add the invariant to the InferableBrand
trait's rustdoc and as a comment in `impl_kind!`.

_Rationale:_ the Marker projection mechanism depends on this
invariant but it was never stated explicitly. `impl_kind!` is the
sole generator, so enforcement is by construction. Documentation
prevents future hand-written impls from violating it.

### Decision N: Solver evaluation order risk

Accept that the Marker projection timing ("commits from FA alone,
before Brand and A are resolved") depends on current rustc solver
behaviour, not a language guarantee. The design is validated on
stable rustc 1.94.1 via all POCs.

_Rationale:_ the new trait solver (rust-lang/rust#107374) could
theoretically change evaluation order, but it is years from
stabilisation. Restructuring pre-emptively would be premature.
Consider adding a periodic nightly CI check with `-Znext-solver`
for early warning.

### Decision O: Attribute semantics clarification

`#[multi_brand]` is a documentation marker, not a codegen switch.
Each `impl_kind!` invocation independently emits at most one
InferableBrand impl. Multiple impls for a given concrete type come
from multiple `impl_kind!` invocations. The attribute signals to
human readers that this brand shares its target type with other
brands.

_Rationale:_ the plan previously said `#[multi_brand]` "tells
impl_kind! to emit multiple InferableBrand impls," which is
misleading. Each invocation handles one brand. Clarify to prevent
misunderstanding during implementation.

### Decision P: `apply`/`ref_apply` dispatch module

Create `dispatch/semiapplicative.rs` as a new dispatch module in
phase 2, with `ApplyDispatch` trait, Val/Ref impls, inference
wrapper with dual InferableBrand bounds (per Decision H), and
explicit wrapper. This is new work, not a mechanical rebinding of
an existing module.

_Rationale:_ POC 8 validates the signature shape. The plan
previously characterised this as "mechanical analogue," which
understates the work.

### Decision Q: `explicit::` function scope (revised)

Explicit functions are NOT rewritten to use Slot bounds. Only
inference wrappers are migrated to Slot. Explicit functions keep
their current signatures with Marker as a free type parameter and
no Slot bound.

_Rationale (revised):_ The original Decision Q required Slot bounds
on all 37 explicit functions. During implementation, this was found
to be incompatible with Decision G: projection brands do not have
Slot impls, so adding Slot bounds to explicit functions would
exclude projection brands from the explicit dispatch path. Since
explicit functions are the universal fallback (including for
projection brands and diagonal cases), they must remain
unconstrained. The `FunctorDispatch`/`BindDispatch`/etc. trait
bounds already validate type class membership. Slot is only needed
in inference wrappers for Brand resolution.

### Decision R: Hash coordination

Add `SLOT_PREFIX` constant (temporary name during phases 1-3) to
`fp-macros/src/core/constants.rs`. Update `classify_trait`,
`is_semantic_type_class`, and `is_dispatch_container_param` to
recognise `Slot_`-prefixed traits as part of the Kind category
(not semantic type classes). In phase 4, rename to
`INFERABLE_BRAND_PREFIX`.

_Rationale:_ without this, Slot-bounded dispatch wrappers produce
incorrect HM signatures because `Slot_` bounds fall through to
`TraitCategory::Other`.

### Decision S: Projection skip rule improvement

Switch the projection auto-skip rule in `impl_kind!` from the
current string heuristic (`contains("::")` / `contains("Apply")`)
to structural AST checks: match on `syn::Type::Macro` for `Apply!`
invocations and check `syn::TypePath` segment count for `::`.

_Rationale:_ the string heuristic works for all current brands but
could false-positive on downstream types named `Applicable` or
using fully-qualified paths. The `syn::visit::Visit` infrastructure
is already used in this module.

### Decision T: Known limitations to document

Document the following known limitations in phase 5 (docs):

- `'static` bounds on multi-brand Slot impls prevent non-static
  fixed parameters from using inference. Pre-existing limitation
  of the Brand pattern, not introduced by InferableBrand.
- `&&T` (double reference) is not supported by FunctorDispatch's
  Ref impl. Add a compile-fail UI test in phase 1 to lock in the
  expected behaviour.
- Pre-bound closures (`let f = |x| x + 1; map(f, Ok(5))`) may
  lose deferred inference context for multi-brand types. Annotate
  the closure parameter.

### Decision U: Multi-brand runtime benchmark

Add one Criterion benchmark comparing Slot-based multi-brand
inference dispatch against explicit dispatch:
`map(|x: i32| x + 1, Ok::<i32, String>(5))` vs
`explicit::map::<ResultErrAppliedBrand<String>, _, _, _, _>(...)`.

_Rationale:_ validates that inference-based dispatch produces
identical codegen to explicit dispatch (zero-cost property).

### Decision V: Test-driven implementation

The exhaustive list of test cases is in
[analysis/07-test-matrix.md](./analysis/07-test-matrix.md) (58 cases
across 6 categories with assumptions-to-tests traceability).

Use a test-driven approach: write tests encoding expected outcomes
from the plan as early as possible, initially `#[ignore]`d with a
reason annotation indicating which phase they belong to. As each
phase progresses, un-ignore the relevant tests; a phase is complete
when all its tests pass.

**Test structure:**

Two test files, added at the start of phase 1:

1. `tests/multi_brand_integration.rs` - positive integration tests
   grouped by phase:

   Phase 1 (un-ignore as map is migrated):
   - Val + single-brand: `map(|x: i32| x + 1, Some(5))`.
   - Val + multi-brand: `map(|x: i32| x + 1, Ok::<i32, String>(5))`.
   - Ref + single-brand: `map(|x: &i32| *x + 1, &Some(5))`.
   - Ref + multi-brand: `map(|x: &i32| *x + 1, &Ok::<i32, String>(5))`.
   - Multi-brand err direction: `map(|e: String| e.len(), Err::<i32, String>("hi".into()))`.
   - Generic fixed param: `fn process<E: 'static>(r: Result<i32, E>) -> Result<i32, E> { map(|x: i32| x + 1, r) }`.
   - Passthrough: `map(|x: i32| x + 1, Err::<i32, String>("fail".into()))` returns `Err("fail")`.

   Phase 2 (un-ignore per-operation as each is migrated):
   - `bind` Val + multi-brand.
   - `bind` Ref + multi-brand.
   - `bimap` at arity 2.
   - `fold_left` / `fold_right` / `fold_map` with multi-brand.
   - `traverse` with multi-brand outer container.
   - `filter` / `filter_map` with multi-brand.
   - `lift2` with multi-brand.
   - `apply` / `ref_apply` with multi-brand (dual Slot bounds).
   - Closureless single-brand: `join(Some(Some(5)))`.
   - Closureless multi-brand explicit: `explicit::join::<ResultErrAppliedBrand<String>, _, _, _>(...)`.

2. `tests/ui/multi_brand_*.rs` - compile-fail UI tests (phase 1):
   - Diagonal: `map(|x: i32| x + 1, Ok::<i32, i32>(5))` fails.
   - Unannotated multi-brand: `map(|x| x + 1, Ok::<i32, String>(5))` fails.
   - Double reference: `map(|x: &i32| *x + 1, &&Some(5))` fails.

**Non-regression safety net** (added at the very start of phase 1,
before any migration begins):

3. `tests/non_regression_single_brand.rs` - exercises every
   existing single-brand inference call pattern that must continue
   to work identically throughout the migration:
   - `map(f, Some(5))`, `map(f, vec![1,2,3])`, `map(f, &lazy)`.
   - `bind(Some(5), f)`, `fold_left(init, f, vec)`, etc.
   - These tests run against the OLD InferableBrand initially and
     must stay green as each module migrates to Slot.

_Rationale:_ the POCs validate the Slot pattern in isolation
(hand-written traits). Production tests validate the same patterns
through the actual macro pipeline, catching divergence between POC
assumptions and macro-generated reality. Phase completion is
concrete: all un-ignored tests pass. Non-regression tests prevent
silent breakage of existing single-brand inference during the
strangler-fig migration.

### Decision W: Closureless Slot inference

Closureless dispatch operations (`join`, `alt`, `apply_first`,
`apply_second`) resolve Brand from the single Slot impl on FA
without needing a closure. Validated by `slot_closureless_poc.rs`
(11 tests covering Val and Ref for both `alt` and `join` with
Option and Vec).

Two structural patterns:

1. **Flat operations** (`alt`, `apply_first`, `apply_second`): the
   Slot bound `FA: Slot<Brand, A>` resolves both Brand and A from
   FA's unique Slot impl. No extra type parameters needed.

2. **Nested operations** (`join`): FA is `Brand::Of<Brand::Of<A>>`,
   so the Slot bound decomposes FA into Brand and an inner type
   `MidA` that is itself a branded container. The inference wrapper
   needs an extra `MidA` type parameter:
   `fn join<'a, FA, A, Brand, MidA>(mma: FA)` with
   `FA: Slot<Brand, MidA> + JoinDispatch<Brand, A, Marker>`.
   `MidA` is fully inferred and never specified by callers.

_Rationale:_ the plan originally described closureless migration as
"mechanical." The POC confirmed this for flat operations but
revealed that nested operations require the extra `MidA` parameter,
which was not anticipated. The pattern is straightforward once
identified but must be applied consistently during phase 2's
`semimonad.rs` migration.

## Integration surface

### Will change

| Component                                                                                                                          | Change                                                                                                                                           |
| ---------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `fp-library/src/kinds.rs`                                                                                                          | Remove old `InferableBrand_*` family; add redesigned `InferableBrand_*` with Brand as trait param and associated `Marker`.                       |
| `fp-library/src/brands.rs`                                                                                                         | No changes; brand struct definitions stay.                                                                                                       |
| `fp-library/src/types/*/mod.rs`                                                                                                    | Rename `#[no_inferable_brand]` to `#[multi_brand]`. Add `#[multi_brand]` to arity-1 multi-brand brands (already marked; just rename).            |
| `fp-macros/src/hkt/impl_kind.rs`                                                                                                   | Generate direct `InferableBrand_*` impls for every brand (single and multi). Skip when brand's `Of` is a projection (contains `Apply!` or `::`). |
| `fp-macros/src/hkt/trait_kind.rs`                                                                                                  | Generate `InferableBrand_{hash}` alongside `Kind_{hash}` at every arity.                                                                         |
| `fp-library/src/dispatch/functor.rs`                                                                                               | `map` rebinds to redesigned `InferableBrand`; `Brand` becomes a function type parameter; `Marker` projected from InferableBrand.                 |
| `fp-library/src/dispatch/semimonad.rs`                                                                                             | Same pattern as functor.rs for `bind`, `bind_flipped`, `join`. `compose_kleisli*` already take Brand via turbofish; no migration needed (L4).    |
| `fp-library/src/dispatch/semiapplicative.rs`                                                                                       | **New module** (Decision P). Create `ApplyDispatch` trait, Val/Ref impls, inference wrapper with dual InferableBrand bounds, explicit wrapper.   |
| `fp-library/src/dispatch/bifunctor.rs`                                                                                             | Same pattern at arity 2 for `bimap`.                                                                                                             |
| `fp-library/src/dispatch/bifoldable.rs`, `bitraversable.rs`, `foldable.rs`, `traversable.rs`, `filterable.rs`, `lift.rs`           | Same pattern for each operation's inference wrapper.                                                                                             |
| `fp-library/src/dispatch/alt.rs`, `apply_first.rs`, `apply_second.rs`                                                              | Mechanical migration. Multi-brand stays explicit-only (closureless).                                                                             |
| `fp-library/src/dispatch/functor_with_index.rs`, `foldable_with_index.rs`, `filterable_with_index.rs`, `traversable_with_index.rs` | Same pattern (closure-taking; gain multi-brand inference).                                                                                       |
| `fp-library/src/dispatch/witherable.rs`, `compactable.rs`, `contravariant.rs`                                                      | Same pattern (closure-taking; gain multi-brand inference).                                                                                       |
| `fp-library/tests/ui/*.rs`                                                                                                         | Delete or rewrite `result_no_inferable_brand.rs` and `tuple2_no_inferable_brand.rs`; add positive and negative UI tests for the new behaviour.   |
| `fp-library/docs/brand-inference.md`                                                                                               | Update to describe InferableBrand; cross-link to `brand-dispatch-traits.md`.                                                                     |
| `fp-library/docs/brand-dispatch-traits.md`                                                                                         | Update to reflect redesigned InferableBrand (Decision D).                                                                                        |

### Unchanged

- **Optics subsystem** (`Lens`, `Prism`, `Iso`, `Traversal`, etc.).
- **Benchmarks**: no code changes. Performance validated
  post-implementation.
- **Stack safety / `TailRec`**: unrelated.
- **Serde integration**: unrelated.
- **FunctorDispatch, BindDispatch, BimapDispatch trait shapes**: their
  Val/Ref impls are untouched. Only the inference wrappers that call
  them are rebound.

### Operations outside the dispatch system

`pure` (`classes/pointed.rs`) and `empty` (`classes/plus.rs`) are
simple free functions that take Brand as an explicit turbofish
parameter: `pure::<OptionBrand, _>(5)`. They have no dispatch
trait, no InferableBrand dependency, and no Val/Ref Marker. They
are completely unaffected by this plan and require no migration.

### Closureless dispatch operations

Several dispatch operations use InferableBrand but lack a closure
for Brand disambiguation:

- `join` (`dispatch/semimonad.rs`).
- `alt` (`dispatch/alt.rs`).
- `apply_first`, `apply_second` (`dispatch/apply_first.rs`,
  `dispatch/apply_second.rs`).

Under the redesigned InferableBrand, these work unchanged for
single-brand types: FA is known from the argument, and the single
InferableBrand impl resolves Brand uniquely without needing a
closure. For multi-brand types, there is no disambiguating context,
so multi-brand `join`, `alt`, `apply_first`, `apply_second` require
`explicit::` (same outcome as today where multi-brand types have no
InferableBrand impl at all).

`Traversable::sequence` is similar: the outer Brand may be
ambiguous for multi-brand outer types; `explicit::` is required.

### Closureless operations with Fn-payload disambiguation

`Semiapplicative::apply` has no direct closure but does have an
`Fn(A) -> B` payload inside `ff`. Decision H / POC 8 confirms
InferableBrand can drive inference from that payload; apply is
therefore moved into the inference-supported set in phase 2.

## Out of scope

Permanently excluded from this plan; revisit only if the design
constraints change.

- **Named helpers** (`map_ok`, `map_err`, `map_fst`, etc.). Under
  InferableBrand these would only fire for diagonal cases (`Result<T, T>` etc.)
  which are rare. `explicit::map::<Brand, ...>(f, fa)` covers the
  same ground. If real-world usage shows diagonal cases are frequent
  enough to warrant sugar, a separate proposal can add them.
- **Primary brand designation** (`#[primary_brand]`). Superseded by
  InferableBrand's symmetric treatment of all brands; no role for it.
- **Non-closure operations for multi-brand types** (`join`, `alt`,
  `apply_first`, `apply_second`, `sequence`). Closure-directed
  inference structurally cannot disambiguate for multi-brand; these
  continue to require `explicit::` for multi-brand (same as today).
  Single-brand inference works via the sole InferableBrand impl.
  `pure` and `empty` are outside the dispatch system entirely
  (defined in `classes/`, take Brand via turbofish) and are
  unaffected.
- **Newtype wrappers** for disambiguation. Conflicts with the
  library's "use your normal types" design principle.
- **Split `map` into `map` + `ref_map`**. This would regress the
  unified Val/Ref dispatch users have today, since POC 5 shows the
  unified signature handles all four Val/Ref x single/multi-brand
  cases.
- **Specialization or negative-impl-based approaches.** All
  require unstable features.

## Implementation phasing

All four phases are implemented on a development branch and
released together (Decision B). The new trait uses the temporary
name `Slot` during phases 1-3 to avoid conflicts with the existing
`InferableBrand`. After the old trait is removed, phase 4 renames
`Slot` to `InferableBrand`.

### Phase 0: Pre-implementation

1. Rewrite `fp-library/docs/brand-dispatch-traits.md` to reflect
   the adopted Marker-only Slot design (Decision L). The current
   content describes a contradictory Out-GAT + InferableBrand-blanket
   design and would mislead implementers.

### Phase 1: Add Slot and migrate `map`

0. Add non-regression test file (`tests/non_regression_single_brand.rs`)
   and multi-brand integration test file
   (`tests/multi_brand_integration.rs`) with phase-2+ tests
   `#[ignore]`d (Decision V). Add compile-fail UI test stubs. Run
   `just verify` to confirm non-regression tests pass against the
   current codebase before any changes.
1. Add `SLOT_PREFIX` constant to `fp-macros/src/core/constants.rs`
   (Decision R). Update `classify_trait`, `is_semantic_type_class`,
   and `is_dispatch_container_param` to recognise `Slot_`-prefixed
   traits.
2. Add `Slot_*` trait family to `fp-library/src/kinds.rs` alongside
   the existing `InferableBrand_*`. Include associated type `Marker`.
   Document the Marker-agreement invariant in the trait's rustdoc
   (Decision M). Both trait families coexist; the branch compiles at
   every step.
3. Update `trait_kind!` to emit `Slot_{hash}` at every arity it
   already emits `Kind_{hash}` for.
4. Update `impl_kind!` to emit direct `Slot` impls for every brand.
   Single brands get `Marker = Val`. The `&T` blanket for references
   (generated once globally) gives `Marker = Ref`. Projection brands
   are skipped using structural AST checks (Decision S). In the same
   change, rename `#[no_inferable_brand]` to `#[multi_brand]` in
   macro input and all use sites (Decision O). Add a comment in
   `impl_kind!` explaining the Marker-agreement invariant
   (Decision M).
5. Rewrite the `map` inference wrapper in
   `fp-library/src/dispatch/functor.rs` to bind on `Slot` with
   Marker projected. `explicit::map` keeps its original signature
   (revised Decision F/Q: explicit functions do not use Slot bounds).
6. Update UI tests: remove `result_no_inferable_brand.rs` and
   `tuple2_no_inferable_brand.rs`; add compile-fail UI tests for
   diagonal, unannotated-multi-brand, and `&&T` cases (Decision T).
7. Un-ignore phase 1 tests in `multi_brand_integration.rs`; verify
   all pass (Decision V). Confirm non-regression tests still green.

### Phase 2: Migrate remaining dispatch modules

Migrate each remaining dispatch module from old `InferableBrand` to
`Slot`. For each module, rewrite the inference wrapper to use Slot
bounds (explicit functions keep their original signatures per revised
Decision Q). After migrating each module,
un-ignore its corresponding tests in `multi_brand_integration.rs`
and run `just verify` to confirm the branch compiles, the newly
un-ignored tests pass, and non-regression tests stay green.

Closure-taking modules (gain multi-brand inference):

- `bind`, `bind_flipped` (`semimonad.rs`).
- `bimap` at arity 2 (`bifunctor.rs`).
- `fold_left`, `fold_right`, `fold_map` (`foldable.rs`).
- `traverse` (`traversable.rs`, outer brand only).
- `filter`, `filter_map`, `partition`, `partition_map`
  (`filterable.rs`).
- `lift2` through `lift5` (`lift.rs`).
- `apply`, `ref_apply`: **create new** `dispatch/semiapplicative.rs`
  module (Decision P). ApplyDispatch trait, Val/Ref impls, inference
  wrapper with dual Slot bounds (Decision H), explicit wrapper.
- `functor_with_index.rs`, `foldable_with_index.rs`,
  `filterable_with_index.rs`, `traversable_with_index.rs`.
- `witherable.rs`, `compactable.rs`, `contravariant.rs`.

Closureless modules (mechanical migration; multi-brand stays
explicit-only):

- `join` (`semimonad.rs`).
- `alt` (`alt.rs`).
- `apply_first` (`apply_first.rs`), `apply_second`
  (`apply_second.rs`).

No migration needed:

- `compose_kleisli`, `compose_kleisli_flipped` (already take Brand
  via turbofish; no InferableBrand usage).
- `pure`, `empty` (defined in `classes/`, not `dispatch/`; take
  Brand via turbofish; unaffected).

Audit do-notation macros (Decision K).

### Phase 3: Remove old InferableBrand

Once all 19 dispatch modules are migrated to `Slot` and no code
references the old `InferableBrand_*`:

1. Remove old `InferableBrand_*` trait family and all its impls
   from `fp-library/src/kinds.rs`.
2. Remove `InferableBrand_*` generation from `trait_kind!` and
   `impl_kind!`.
3. Remove `resolve_inferable_brand()` preprocessing from `Apply!`.
4. Remove the `InferableBrand!` proc macro from `fp-macros/src/lib.rs`.
5. Remove `INFERABLE_BRAND_PREFIX` / `INFERABLE_BRAND_MACRO`
   constants.
6. Update `is_dispatch_container_param()` and `classify_trait()` in
   the analysis/documentation modules to check for `Slot_` instead
   of `InferableBrand_`.
7. Regenerate signature snapshot test expectations.

### Phase 4: Rename Slot to InferableBrand

With the old `InferableBrand` gone, rename `Slot` to
`InferableBrand` throughout the codebase:

1. Bulk `sed` across all `*.rs` files: `Slot_` -> `InferableBrand_`,
   `Slot<` -> `InferableBrand<`, etc. This includes the POC test
   files (`slot_production_poc.rs`, `slot_valref_poc.rs`,
   `slot_marker_via_slot_poc.rs`, `slot_bind_poc.rs`,
   `slot_arity2_poc.rs`, `slot_apply_poc.rs`, and the two
   negative-result POCs).
2. Rename `SLOT_PREFIX` constant to `INFERABLE_BRAND_PREFIX` in
   `fp-macros/src/core/constants.rs` and all consumers.
3. Update `trait_kind!` and `impl_kind!` to use the
   `InferableBrand` prefix.
4. Update `is_dispatch_container_param()`, `classify_trait()`, and
   signature snapshot tests for the `InferableBrand_` prefix.

### Phase 5: Diagnostics, benchmarks, and docs

1. Attach `#[diagnostic::on_unimplemented]` to `InferableBrand`
   with wording determined from phase 1/2 observations (Decision J).
2. Update `fp-library/docs/brand-inference.md` and
   `fp-library/docs/brand-dispatch-traits.md` for the redesigned
   InferableBrand.
3. Update all dispatch module doc comments that reference the old
   InferableBrand to describe the redesigned trait.
4. Update `map`'s doc comment to document the closure-annotation
   requirement for multi-brand types (Decision C).
5. Document known limitations (Decision T): `'static` bounds on
   multi-brand fixed parameters, `&&T` not supported, pre-bound
   closures need parameter annotations.
6. Add multi-brand runtime benchmark (Decision U).
7. Update `CLAUDE.md` and any other developer-facing docs.

## Success criteria

The plan is complete when:

- `map(|x: i32| x + 1, Ok::<i32, String>(5))` compiles and maps over
  Ok. Analogous single calls work for `bind`, `bimap`, `fold_*`,
  `traverse`, `filter`, `lift2`.
- `map(|e: String| e.len(), Err::<i32, String>("hi".into()))`
  compiles and maps over Err.
- `map(|x: &i32| *x + 1, &Ok::<i32, String>(5))` compiles and maps
  over Ok by reference.
- `map(|x: i32| x + 1, Ok::<i32, i32>(5))` fails at compile time
  with a diagnostic mentioning `explicit::map`.
- All existing single-brand call sites (`map(f, Some(5))`,
  `map(f, &lazy)`, `bind(ma, f)`, etc.) continue to work identically.
- All existing `explicit::map::<...>(f, value)` call sites continue
  to work (with turbofish shape per Decision F).
- Clean-build wall-clock time under ~36s (50% regression threshold
  versus 24.0s baseline; see Decision I).
- No regression in existing test suites, benchmarks, or doctests.

## Reference material

- Design analysis: [analysis/multi-brand-evaluation.md](./analysis/multi-brand-evaluation.md).
- Test matrix: [analysis/07-test-matrix.md](./analysis/07-test-matrix.md).
- Review findings: [analysis/06-consolidated-review-findings.md](./analysis/06-consolidated-review-findings.md).
- POCs: listed in the Validated via POCs section above.
- Brand-dispatch traits overview: [fp-library/docs/brand-dispatch-traits.md](../../../fp-library/docs/brand-dispatch-traits.md).
- Parent brand-inference plan: [../brand-inference/plan.md](../brand-inference/plan.md).
