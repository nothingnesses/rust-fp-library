# Plan: Multi-Brand Ergonomics via Closure-Directed Inference

**Status:** DRAFT

This plan extends the brand-inference system to handle multi-brand
concrete types (`Result`, `Pair`, `Tuple2`, `ControlFlow`, `TryThunk`)
using closure-directed inference.

## API stability stance

`fp-library` is pre-1.0. API-breaking changes are acceptable when they
lead to a better end state. This plan therefore prioritises design
correctness and internal coherence over preserving compatibility with
the current public surface. Specifically:

- Renaming, reshaping, or removing existing `explicit::` signatures,
  free functions, macros, or attributes is acceptable if the
  replacement is clearer or more consistent.
- `#[no_inferable_brand]` and similar public macro attributes can
  change semantics without a migration shim; a changelog entry is
  sufficient.
- Existing call sites (in doctests, UI tests, user code) can require
  updates. Breakage is documented and mass-updated in the same
  release rather than deferred.

Downstream impact (open question 11) and partial-rollout inconsistency
(open question 8) are still worth thinking about, but they do not
function as hard constraints. Where a cleaner design requires
breaking changes, prefer the cleaner design.

## Motivation

Brand inference
([docs/plans/brand-inference/plan.md](../brand-inference/plan.md),
implemented) lets users call free functions without a turbofish for
types with a single canonical brand. It deliberately refuses inference
for multi-brand types and forces them through `explicit::`:

```rust
// Today
explicit::map::<ResultErrAppliedBrand<String>, _, _, _, _>(
    |x: i32| x + 1,
    Ok::<i32, String>(5),
)
```

A feasibility POC
([fp-library/tests/closure_directed_inference_poc.rs](../../../fp-library/tests/closure_directed_inference_poc.rs))
showed that Rust's stable trait selection can disambiguate a brand from
`(container type, closure input type)` using an overlapping-but-distinct
impl pattern. The analysis
([analysis/multi-brand-evaluation.md](./analysis/multi-brand-evaluation.md))
concluded this is the strongest design: it treats all brands symmetrically,
avoids the silent-wrong-direction hazard of a canonical-primary design,
and surfaces ambiguity as a loud compile error.

After this plan lands, users write:

```rust
// After
map(|x: i32| x + 1, Ok::<i32, String>(5))         // Ok-mapping
map(|e: String| e.len(), Err::<i32, String>("hi".into()))  // Err-mapping
```

## Prerequisites

- Brand inference is implemented (see
  [brand-inference/plan.md](../brand-inference/plan.md)).
- `#[no_inferable_brand]` is in place on all multi-brand `impl_kind!`
  invocations.
- `explicit::` dispatch functions exist and cover every brand.
- The POC validates the `Slot<Brand, A>` pattern on stable rustc.

## Design overview

Replace `InferableBrand`'s role in `map`-like signatures with a new
`Slot<Brand, A>` trait. Trait selection uses both the container type
`FA` and the closure's input type `A` to identify a unique brand:

- **Single-brand types** (Option, Vec, Thunk, etc.): a blanket impl
  from `InferableBrand` to `Slot` makes this transparent. No direct
  `Slot` impl required per type. Behavior matches today.
- **Multi-brand types** (Result, Pair, Tuple2, ControlFlow, TryThunk):
  each brand provides a direct `Slot` impl. Trait selection picks the
  one whose `A` slot aligns with the closure's input type.
- **Diagonal cases** (`Result<T, T>`, `(T, T)`, etc.) and **unannotated
  closures on multi-brand types**: trait selection is ambiguous, Rust
  emits E0283, and the diagnostic points users at `explicit::map`.

`explicit::map` remains unchanged and handles every case Slot cannot.

## Design detail

### The `Slot` trait

```rust
pub trait Slot<'a, Brand, A>
where
    Brand: Kind_cdc7cd43dac7585f,
    A: 'a,
{
    type Out<B: 'a>: 'a;
    // Methods or dispatch hooks, TBD in implementation.
}
```

Actual signature depends on integration with the existing
`FunctorDispatch` machinery (see open questions). One impl exists per
brand per concrete type:

```rust
// Multi-brand impls, provided explicitly
impl<'a, A, E> Slot<'a, ResultErrAppliedBrand<E>, A> for Result<A, E> {
    type Out<B: 'a> = Result<B, E>;
}

impl<'a, T, A> Slot<'a, ResultOkAppliedBrand<T>, A> for Result<T, A> {
    type Out<B: 'a> = Result<T, B>;
}
```

### Blanket impl from `InferableBrand`

For types with a canonical brand, `Slot` is derived automatically:

```rust
impl<'a, FA, A> Slot<'a, FA::Brand, A> for FA
where
    FA: InferableBrand_cdc7cd43dac7585f,
    A: 'a,
{
    type Out<B: 'a> = <FA::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>;
}
```

This means every single-brand type reachable today continues to work
with no source changes. The library only needs direct Slot impls for
multi-brand types.

### The unified `map` function

Replace the `InferableBrand` bound with `Slot`:

```rust
pub fn map<'a, Brand, FA, A, B, Marker>(
    f: impl FunctorDispatch<'a, Brand, A, B, FA, Marker>,
    fa: FA,
) -> <FA as Slot<'a, Brand, A>>::Out<B>
where
    FA: Slot<'a, Brand, A>,
    Brand: Kind_cdc7cd43dac7585f,
    A: 'a,
    B: 'a,
```

`Brand` is a function type parameter resolved by trait selection via
`Slot<Brand, A>`. In practice:

- Option<i32> with `|x| x+1`: blanket derives Slot<OptionBrand, i32>.
  Single impl matches, Brand = OptionBrand. Identical to today.
- Result<i32, String> with `|x: i32| x+1`: two direct impls exist.
  Only the ResultErrAppliedBrand impl unifies with A = i32. Single
  match, Brand = ResultErrAppliedBrand<String>.
- Result<i32, i32> with `|x: i32| x+1`: both direct impls unify.
  Ambiguous, compile error.

### Macro support

`impl_kind!` extensions:

- Brands without `#[no_inferable_brand]`: generate `InferableBrand` as
  today. Slot falls out via the blanket impl.
- Brands with `#[no_inferable_brand]`: generate a direct `Slot` impl
  instead (or in addition). The macro already has the `Of<'a, A>`
  signature needed to produce the Slot impl.

### Diagnostic

Attach `#[diagnostic::on_unimplemented]` or `#[rustc_on_unimplemented]`
to the `Slot` trait (or to a marker reflecting ambiguity) with a
message along the lines of:

```text
`T` does not uniquely determine a brand for this operation.
= help: annotate the closure parameter type to disambiguate (e.g., `|x: i32| ...`)
= help: or use `explicit::map::<SomeBrand, _, _, _, _>(...)` to specify the brand directly
```

For types that are ambiguous even with annotation (the diagonal case),
only the `explicit::map` suggestion applies. The diagnostic wording
should handle both cases.

### What changes for existing code

- **User-facing call sites with single-brand types:** no change. Blanket
  impl preserves today's behavior.
- **User-facing call sites with multi-brand types using `explicit::`:**
  no change. `explicit::` is not touched.
- **User-facing call sites with multi-brand types using inference (new):**
  now work if closure input type disambiguates; fail with the improved
  diagnostic otherwise.
- **`#[no_inferable_brand]` attribute:** semantics extended from "skip
  InferableBrand" to "skip InferableBrand and generate direct Slot impl
  instead." Existing invocations continue to work unchanged.

## Integration surface

### Will change alongside `map` (phase 1)

- **`InferableBrand_{hash}` family:** blanket impl from InferableBrand
  to Slot added. Existing InferableBrand impls and
  `#[diagnostic::on_unimplemented]` attributes stay in place and
  remain reachable for any code path that names the brand directly via
  InferableBrand.
- **`FunctorDispatch`:** internal structure unchanged, but the free
  function `map` rebinds its container constraint from InferableBrand
  to Slot.
- **`impl_kind!` macro:** new code path generating direct Slot impls
  for brands marked `#[no_inferable_brand]`. The macro already has the
  `Of<'a, A>` signature information required.
- **`trait_kind!` macro:** must generate a `Slot_{hash}` trait per
  Kind signature, analogous to `InferableBrand_{hash}`. See the
  higher-arity discussion below for scope.
- **UI tests:** delete or rewrite
  `fp-library/tests/ui/result_no_inferable_brand.rs` and
  `tuple2_no_inferable_brand.rs` (the current ambiguity assertions).
  Add new tests for closure-directed resolution (positive), diagonal
  failure, and unannotated-closure failure.

### Will change in phase 3 (other closure-taking operations)

The Slot pattern applies uniformly to any operation that takes a
closure consuming a type argument the brand disambiguates over. For
operations without such a closure, Slot provides no help and users
stay on `explicit::` for multi-brand types.

| Operation                        | Closure input drives A? | Slot applicable?                       |
| -------------------------------- | ----------------------- | -------------------------------------- |
| `Functor::map`                   | Yes (`A -> B`)          | Yes (phase 1)                          |
| `Semimonad::bind`                | Yes (`A -> fb`)         | Yes                                    |
| `Lift::lift2`                    | Yes (`(A, B) -> C`)     | Yes                                    |
| `Foldable::fold_left` / `_right` | Yes (`(B, A) -> B`)     | Yes                                    |
| `Foldable::fold_map`             | Yes (`A -> M`)          | Yes                                    |
| `Filterable::filter`             | Yes (`A -> bool`)       | Yes                                    |
| `Traversable::traverse`          | Yes (`A -> g(B)`)       | Yes (outer brand only)                 |
| `Semiapplicative::apply`         | No direct closure       | Possibly via `Fn(A) -> B` payload type |
| `Traversable::sequence`          | No closure              | No                                     |
| `Alt::alt`, `Plus::empty`        | No closure              | No                                     |
| `Pointed::pure`                  | No closure              | No (return-type inference problem)     |

### Will require attention in phase 1 but is not primary scope

- **Ref-variant dispatch (`RefFunctor`, `RefSemimonad`, etc.):** the
  existing Val/Ref `Marker` pattern multiplexes owned and borrowed
  containers through a single dispatch trait. Slot must compose with
  it correctly: `map(|x: &i32| *x + 1, &Ok::<i32, String>(5))` should
  pick `ResultErrAppliedBrand<String>` (because `&i32` aligns with the
  Ok slot's reference form) and route through `RefFunctor::ref_map`.
  Prototype alongside the owned case before committing the design.
- **Do/Ado notation macros (`m_do!`, `a_do!`):** desugar to nested
  `bind` / `apply` calls. After phase 3 makes `bind` CDI-enabled,
  these macros should produce well-typed code for multi-brand types
  when user closures are annotated. Audit
  `fp-library/tests/do_notation.rs` and
  `fp-library/tests/ado_notation.rs` for regressions and missing
  coverage.
- **Existing `on_unimplemented` messages on `InferableBrand`:** remain
  in place; new Slot-specific diagnostic is attached to Slot (or a
  marker trait reflecting ambiguity). The plan should specify which
  attribute appears where.

### Not affected

- **Optics subsystem** (`Lens`, `Prism`, `Iso`, `Traversal`, etc.):
  profunctor-encoded with a separate dispatch mechanism. Brand
  inference does not touch optics.
- **Bifunctor / Bifoldable / Bitraversable at arity 2:** already
  unambiguous via `InferableBrand_266801a817966495` (e.g.
  `ResultBrand` has exactly one arity-2 brand). No change required.
- **Benchmarks:** no code changes. Performance validated
  post-implementation by running `benches/benchmarks/`; Slot is a
  pure trait-selection mechanism with no runtime cost.
- **Stack safety / `TailRec`, optics, serde integration:** unrelated.

## Higher-arity types

The `Slot<Brand, A>` design generalizes to any Kind arity. For an
arity-k Kind, the corresponding `Slot_k<Brand, A1, ..., Ak>` would
take as many closure-input parameters as the Kind_k it mirrors, and
impls would be keyed by which slots of the concrete type are free.

### The general pattern

For a hypothetical arity-3 type `Trifunctor<A, B, C>` with three
arity-1 brands (one per "remaining free slot"):

- `TrifunctorBCFixedBrand<B, C>` fixes B and C, maps over A.
  `Of<X> = Trifunctor<X, B, C>`.
- `TrifunctorACFixedBrand<A, C>` fixes A and C, maps over B.
  `Of<X> = Trifunctor<A, X, C>`.
- `TrifunctorABFixedBrand<A, B>` fixes A and B, maps over C.
  `Of<X> = Trifunctor<A, B, X>`.

Closure-directed inference works the same way as at arity 2:

- `map(|x: i32| ..., t: Trifunctor<i32, String, bool>)`: only the
  "free A" brand's Slot impl unifies with `A = i32` (since `String`
  and `bool` do not match). Unique resolution.
- `map(|x: String| ..., t: Trifunctor<i32, String, bool>)`: only the
  "free B" brand unifies. Unique.
- Diagonal cases: `Trifunctor<T, T, U>` with a closure consuming `T`
  is ambiguous across two brands. `Trifunctor<T, T, T>` with the same
  closure is triply ambiguous.

### Mixed-arity partial applications

An arity-k type may also be partially applied to an intermediate
arity. For `Trifunctor<A, B, C>`:

- Arity-2 partial applications fix one of three slots:
  `TrifunctorAFixedBrand<A>` (maps over B and C),
  `TrifunctorBFixedBrand<B>`, `TrifunctorCFixedBrand<C>`. Each has
  an arity-2 `Of<X, Y>`.
- These arity-2 brands would then have their own arity-1 sub-brands,
  forming a tree of partial applications.

At each arity level, Slot_k disambiguates brands whose `Of` produces
the same concrete type. The mechanism is uniform; only the trait
arity changes.

### Scope decision for this plan

There are two separable things to decide: which arities the
`Slot_{hash}` trait family exists at, and which arities `impl_kind!`
emits direct Slot impls for.

**Trait family: generate at every Kind arity.** `trait_kind!` already
emits `Kind_{hash}` and `InferableBrand_{hash}` for every Kind
signature encountered in the codebase (arity 1 with and without
lifetime, arity 2 with and without lifetime). `Slot_{hash}` follows
the same pattern: for each `Kind_{hash}` that `trait_kind!` emits,
it also emits the corresponding `Slot_{hash}` trait plus the blanket
impl from `InferableBrand_{hash}` to `Slot_{hash}`. This keeps the
three-trait family uniform across arities and costs essentially
nothing (the traits are marker-style with no runtime representation).

**Direct impls: only where multi-brand ambiguity exists today.**
`impl_kind!` emits direct `Slot_{hash}` impls only for brands
carrying `#[no_inferable_brand]`. In the current library all such
brands are at arity 1 (`Result`, `Pair`, `Tuple2`, `ControlFlow`,
`TryThunk` partial applications), so phase 1 materializes direct
impls only at that arity. Higher-arity brands (`ResultBrand` at
arity 2, etc.) are single-brand at their arity and pick up `Slot`
coverage via the blanket from InferableBrand; no direct impl is
emitted.

If future library growth introduces a higher-arity type with
multiple partial-application brands at the same level (e.g. an
arity-3 type with three arity-1 brands, or an arity-2 type with
multiple arity-2 partial applications that map to the same concrete
type), the `#[no_inferable_brand]` attribute on those brands would
trigger direct Slot-impl generation at the appropriate arity. The
macro logic is uniform across arities; no further design change
required.

## Scope

### In scope

- `Functor::map` via the new `Slot` trait.
- Macro support for generating Slot impls on multi-brand brands.
- Diagnostic attribute on Slot for ambiguity.
- Doc updates.
- Delete the `result_no_inferable_brand.rs` and
  `tuple2_no_inferable_brand.rs` UI tests (or replace them with tests
  asserting the new closure-directed behavior and the diagonal failure
  case).

### Deferred (future phases of this plan)

- **Extension to other closure-taking operations** (`bind`, `apply`,
  `lift2`, `traverse`, `fold_left`, `fold_right`, `fold_map`). The
  same Slot pattern generalizes to each, but applying the change to
  every operation is a larger effort. Land `map` first (phase 1),
  validate the design end-to-end, then extend (phase 3).

### Out of scope

- **Named helpers** (`map_ok`, `map_err`, `map_fst`, etc.). Under
  closure-directed inference these would fire only on diagonal cases
  (`Result<T, T>`, `(T, T)`, etc.), which are rare. `explicit::map` with
  a brand turbofish handles them at slightly more call-site verbosity
  but without introducing new API surface. If user feedback later
  shows diagonal cases arise frequently enough to warrant dedicated
  ergonomic sugar, helpers can be proposed in a separate plan; they
  are not a phase of this one.
- **Primary brand designation** (`#[primary_brand]`). Superseded by
  Slot's symmetric treatment of all brands; the role it would have
  played does not exist under this design.
- **Non-closure operations** (`pure`, `empty`, `alt`, `sequence`, etc.).
  Closure-directed inference structurally cannot apply to operations
  without a closure. These continue to use `explicit::` and the
  existing InferableBrand-based path. Any future work on their
  ergonomics would be a separate proposal with a different mechanism.
- **Newtype disambiguation.** Conflicts with the library's design
  principles (users would have to wrap and unwrap values at
  boundaries).
- **Type-only priority without closure help.** Requires unstable
  features (specialization or negative impls).
- **Primary-brand default with closure-directed fallback.** Requires
  specialization to layer the two dispatch paths.

## Pending review

This section consolidates items flagged during plan review. Each entry
lists the concern, the approaches available, and the trade-offs. Items
marked "POC finding" have been validated or invalidated against stable
rustc via
[fp-library/tests/slot_production_poc.rs](../../../fp-library/tests/slot_production_poc.rs);
other items remain for later discussion.

### POC findings (summary)

The production-style POC validated or invalidated the three blockers
and two of the open questions:

| Item                              | Finding                                                                              |
| --------------------------------- | ------------------------------------------------------------------------------------ |
| Blocker 1 (blanket + direct)      | **Invalidated.** Option A1 fails with E0119. Option A2 works cleanly.                |
| Blocker 2 (lifetime-generic GAT)  | **Validated** (under option A2). Lifetimes propagate and normalise.                  |
| Blocker 3 (return-type normalise) | **Validated** (under option A2). Resolves in match arms, generic fns.                |
| Open question 4 (Val/Ref)         | **Type-level validated.** `&T` blanket works; dispatch-level untested.               |
| Open question 9 (annotations)     | **Validated.** Closure-input annotation required; return-type-only does not suffice. |

Items not addressed by the POC (diagnostic routing, apply-side CDI,
partial-rollout UX, do-notation, downstream impact, testing strategy)
require either production code or non-technical decisions.

### Blockers

1. **Coherence between the blanket impl and direct `Slot` impls.**
   **Finding (POC): invalidated.** The blanket
   `impl<FA: InferableBrand_*> Slot_*<FA::Brand, A> for FA` combined
   with direct `Slot` impls on multi-brand types produces E0119
   (`conflicting implementations of trait ... for type Result<_, _>`).
   Rust's coherence checker cites that "upstream crates may add a new
   impl of trait `InferableBrand_cdc7cd43dac7585f` for type
   `Result<_, _>` in future versions," and refuses to prove
   non-overlap through the where-clause bound. A symmetric conflict
   occurs between the InferableBrand `&T` blanket and a Slot `&T`
   blanket used together.
   **Consequence:** Option A1 (blanket + direct impls) is not viable
   on stable rustc. The plan must adopt option A2 (direct `Slot`
   impls for every brand, no InferableBrand-based blanket) or a
   different strategy entirely. Decision A below reflects this.

2. **Lifetime-generic GAT behavior.**
   **Finding (POC): validated under option A2.** A Slot trait with
   lifetime `'a` in its signature and a lifetime-bounded GAT
   (`type Out<B: 'a>: 'a`) compiles and resolves correctly for both
   lifetime-free types (`Option<A>`, `Vec<A>`, `Result<A, E>`) and
   lifetime-bearing types (`Lazy<'a, A, Config>`). Associated-type
   projections normalise at call sites without additional annotations.
   No GAT-specific edge cases triggered by the POC's exercises.

3. **Return type computation through `Slot::Out<B>`.**
   **Finding (POC): validated under option A2.**
   `<FA as Slot_*<'a, Brand, A>>::Out<B>` normalises to the expected
   concrete type across all tested call-site shapes: flowing into
   match arms, into generic function parameters, and chaining
   through multiple `slot_map` calls. No inference-ordering
   pathologies observed for the option-A2 impl layout.
   **Caveat:** these findings apply to option A2. If option A1 were
   resurrected via some future mechanism (specialization, sealed
   traits), blocker 3 would need re-validation because the projection
   path would differ.

### Open questions

Items to resolve before or during implementation.

4. **Val/Ref dispatch as a second selection axis.**
   **Finding (POC): type-level validated; dispatch-level untested.**
   A `&T` blanket for `Slot` (parallel to the existing one on
   InferableBrand) lets `slot_map(f, &fa)` resolve the brand through
   the reference for both single-brand (`&Option`, `&Vec`) and
   multi-brand (`&Result`) concrete types. The POC's MapDispatch for
   `&T` delegates through `Clone`, so it does not exercise the
   production `FunctorDispatch`/`RefFunctor` split. The remaining
   uncertainty is whether `Slot`'s brand axis composes with the
   existing Val/Ref `Marker` axis in a single dispatch signature;
   this requires a second POC against the production
   `FunctorDispatch` trait or validation during phase 1 implementation.

5. **Diagnostic wording precision.** Does the "user forgot
   annotation" case need a different message from the "diagonal,
   annotation won't help" case? Rust's diagnostic attributes are not
   dynamic, so one message covering both is the likely outcome.

6. **Diagnostic routing between InferableBrand and Slot.**
   InferableBrand retains its `on_unimplemented` message ("does not
   have a unique brand"). Slot gets a new one ("closure input could
   not disambiguate"). In a failure scenario, which diagnostic fires
   depends on which trait Rust reports against. Possibly both,
   possibly neither. Unclear without prototyping.

7. **Apply-side closure-directed inference.**
   `Semiapplicative::apply` has no outer closure but carries an
   `Fn(A) -> B` payload inside `ff`. Could the payload's function
   type drive Slot dispatch in phase 3? Decision can defer to phase 3
   but affects whether apply becomes CDI-capable or stays
   explicit-only for multi-brand types.

8. **Partial-rollout inconsistency during phase 1.** Phase 1
   unblocks `map` for multi-brand types; phase 3 extends the same to
   `bind`, `fold_*`, `traverse`, `lift2`, etc. Between phases the
   library is inconsistent: multi-brand `map` works, multi-brand
   `bind` still fails. Users may reasonably question the asymmetry.
   **Weight reduced** by the pre-1.0 stance: bundling phase 1 and
   phase 3 into a single release is acceptable if it yields a more
   coherent public API.

9. **Closure-annotation fragility.**
   **Finding (POC): validated.** Single-brand types (Option, Vec)
   accept unannotated closures (`|x| x + 1`) because `A` flows from
   the container via the Slot impl. Multi-brand non-diagonal types
   (Result<i32, String>) require an explicit closure-input annotation
   (`|x: i32| ...`); without it the Slot impls cannot be
   disambiguated. Annotating the call-site return type alone (e.g.,
   `let r: Result<i64, String> = slot_map(|x| ..., Ok::<i32, String>(5))`)
   is **not** sufficient; the annotation must pin the closure's input.
   Documented in the POC (commented test case).

10. **Do-notation macro behavior (`m_do!`, `a_do!`).** These desugar
    to chained `bind`/`apply` calls. After phase 3, their closures
    will require annotation when operating on multi-brand types. The
    macros themselves may need an audit for how types propagate
    through their expansions.

11. **Downstream crate impact.** The `#[no_inferable_brand]`
    attribute's semantics change (previously: suppress InferableBrand
    generation; after: suppress InferableBrand + emit direct Slot).
    Downstream crates using this attribute experience a semantic
    shift in a public macro API.
    **Weight reduced** by the pre-1.0 stance: a changelog entry is
    acceptable; no migration shim is required. See Q14 for the
    related question of whether to rename the attribute while making
    this change.

12. **Testing strategy.** All existing single-brand doctests should
    compile identically. All existing `explicit::map::<...>` doctests
    on multi-brand types should stay as-is (they document the
    explicit path). Add new positive doctests for closure-directed
    resolution and UI tests for the diagonal failure cases. The
    existing POC at
    `fp-library/tests/closure_directed_inference_poc.rs` should be
    promoted to a proper integration test or removed once the real
    implementation subsumes it.

13. **Slot generation scope for projection-type brands.** `impl_kind!`
    currently auto-skips InferableBrand for brands whose `Of` target
    contains `Apply!` or `::` (e.g.,
    `BifunctorFirstAppliedBrand<ResultBrand, A>`). Under option A2,
    should these derived brands also get direct `Slot` impls?
    Generating Slot for them would make `Result<A, E>` match 4 brands
    at arity 1 instead of 2, amplifying closure-input-type
    ambiguities. Skipping them leaves bifunctor-derived mapping as
    explicit-only.
    Options: - **a) Keep the projection auto-skip rule for Slot too.**
    Projection-type brands remain explicit-only. Simplest; no
    ambiguity expansion. - **b) Generate Slot for projection-type brands.** Uniform brand
    landscape; risks ambiguity in cases where a bifunctor-derived
    brand and a direct brand both match the same `(FA, A)`. - **c) Generate Slot routing through `Bifunctor::bimap(identity,
f, _)`.** Technically elegant; macro logic grows.

14. **Attribute naming under option A2.**
    `#[no_inferable_brand]` now means "suppress InferableBrand AND
    emit direct Slot." Types with this attribute ARE inferable (via
    closure-directed inference); the name overpromises. The pre-1.0
    stance permits renaming.
    Options:
    - **a) Keep the name.** Update documentation to clarify semantics.
      No breakage; misleading.
    - **b) Rename** to `#[multi_brand]`, `#[no_unique_brand]`, or
      similar. Accurate; one-time breakage within the pre-1.0
      window.
    - **c) Split** into `#[no_inferable_brand]` (suppression) and
      `#[multi_brand]` (semantic flag). Composable; more attribute
      surface.

15. **`explicit::` module reorganization under Slot.** Under option
    A2, `explicit::` still exists for the diagonal case and for
    deliberate explicit-brand usage. Today it dispatches through
    direct Brand+FA; should it be rewritten to route through Slot
    internally?
    Options:
    - **a) Keep today's `explicit::map::<Brand, A, B, FA, Marker>`**
      shape. No churn for existing users; two dispatch pipelines
      coexist internally.
    - **b) Rewrite `explicit::map` to bound on Slot,** with Brand
      pinned via turbofish: `explicit::map::<Brand>(f, fa)`. Unifies
      dispatch; turbofish surface naturally contracts.
    - **c) Remove `explicit::` entirely** in favour of inherent
      methods or direct `Brand::method` calls. Probably too far even
      pre-1.0.

16. **Compile-time regression risk from per-brand Slot generation.**
    Option A2 approximately doubles the macro-generated trait-impl
    code per brand (InferableBrand + Slot instead of just
    InferableBrand). The effect on compile times is unknown.
    Options:
    - **a) Measure post-implementation.** Accept regression if small
      (for example, under 5%). Revisit only if worse.
    - **b) Fuse InferableBrand and Slot generation** into a single
      macro pass to reduce expansion overhead. Minor win at best.
    - **c) Only generate Slot for brands participating in
      closure-directed dispatch** (exclude tags like `SendThunkBrand`
      that carry no type-class impls). Complicates macro logic for
      uncertain savings.

17. **Macro hash coordination for `Slot_{hash}`.** `Slot_{hash}`
    must share the content hash used by `Kind_{hash}` and
    `InferableBrand_{hash}` so `impl_kind!`-emitted impls target the
    right trait. This is more implementation note than open question;
    it needs to be addressed as part of phase 1 rather than decided
    in advance.
    Options:
    - **a) Share the existing hash generator** across Kind,
      InferableBrand, and Slot in the macro code. Obvious choice.
    - **b) Re-derive hashes independently** in each macro pass.
      Risks silent drift.

### Design decisions with trade-offs

Decisions that the plan currently answers one way but where
alternatives exist. Listed with options and trade-offs; no choice is
locked in.

**Decision A: Coherence approach.** (POC-informed.)

- Option A1: Trust Rust's where-clause coherence with blanket +
  direct impls. **POC finding: does not work on stable rustc**
  (E0119, see blocker 1 above). No longer a viable option without
  either specialization (unstable) or a different disambiguation
  mechanism.
- Option A2: No blanket; generate direct `Slot` impls for every
  brand (single- and multi-). **POC finding: works; all blocker
  validations passed under this scheme.** Single-brand types
  duplicate the effect the blanket would have provided. Coherence
  is trivially safe (each impl is keyed on a distinct brand). Cost:
  more generated code, zero runtime impact. InferableBrand remains
  as the "unique-brand assertion" trait but no longer has a
  dispatch role tied to Slot.
- Option A3: Sealed marker trait. Introduce a private marker like
  `trait MultiBrand: Sealed` implemented by multi-brand concrete
  types; restructure the blanket around it. Adds complexity for
  limited gain over A2.
- Option A4: Invert the design. Make `Slot` primary; derive
  InferableBrand from uniqueness of Slot resolution. Rewrites more
  of the existing brand machinery. High risk.

The POC's findings reduce this decision to "adopt option A2" unless
an unstable-feature path becomes acceptable.

**Decision B: Phase structure.**

- Option B1: Keep phases separate (current plan). Simplest PR
  sequence. Worst user-facing experience between phase 1 and phase 3
  releases.
- Option B2: Bundle phase 1 and phase 3 into a single release.
  Larger change per release but consistent public API throughout.
- Option B3: Internally phased, released together. Phase 1 as a
  testbed; phase 3 extends mechanically; release only after both
  stabilize. Users never see the intermediate state.

**Decision C: Annotation requirement UX.**

- Option C1: Accept the requirement. Document prominently that
  multi-brand types need closure annotations under the inference
  path; `explicit::` remains as the no-annotation alternative via
  turbofish.
- Option C2: Provide alternative signatures that accept annotation
  differently. Unclear how this would look; probably not worth
  pursuing.

### Tentative mitigations suggested during review

Suggestions to evaluate during decision-making. Not authoritative;
listed as starting points for discussion.

1. **Extend the POC before committing to implementation.** **Done.**
   The production-style POC
   ([fp-library/tests/slot_production_poc.rs](../../../fp-library/tests/slot_production_poc.rs))
   covers the three blockers plus Val/Ref reference-blanket resolution
   and the closure-annotation matrix. Remaining POC gap: full
   `FunctorDispatch`/`Marker` composition (open question 4).

2. **Adopt option A2 for coherence.** POC invalidated option A1 on
   stable rustc; option A2 (direct `Slot` impls per brand) is the
   path that actually compiles. The plan should treat this as
   decided unless an unstable-feature path is on the table.

3. **Release phase 1 and phase 3 together (option B3).** Implement
   phase 1 first as a testbed, extend to other closure-taking
   operations before publishing, and release only after both are
   stable.

4. **Audit do-notation before phase 3.** Verify that `m_do!` and
   `a_do!` can produce well-typed code for multi-brand types with
   reasonable closure annotations. Add doc-level guidance if
   needed.

5. **Add a migration note for downstream crates.** Changelog entry
   and doc update explaining the `#[no_inferable_brand]` semantic
   shift when the release ships.

6. **Defer open questions 5 and 7** (diagnostic wording, apply via
   Fn payload) until there is a working phase 1 prototype; decide
   with evidence rather than upfront.

7. **Treat the POC as specification, not throwaway code.** Every
   case the POC compiles should continue to compile in the
   production implementation. Regressions from POC behavior are
   regressions from the plan's stated capability.

8. **Run a second POC for Val/Ref + FunctorDispatch + Slot
   composition.** The current production-style POC uses a Clone-based
   `&T` shim and does not exercise the production Marker parameter.
   Before phase 1 implementation, validate that Slot's brand-dispatch
   axis composes with FunctorDispatch's Val/Ref Marker axis in a
   single signature. Covers the remaining uncertainty in open
   question 4.

9. **Benchmark compile-time impact** of per-brand Slot generation
   as part of phase 1 acceptance. Targets open question 16; detects
   regressions early rather than post-release.

## Implementation phasing

### Phase 1: Slot trait and map integration

1. Define `Slot` in `fp-library/src/kinds.rs` (alongside `InferableBrand`).
   The module-level doc comment must summarize the trait trio
   (`Kind_*`, `InferableBrand_*`, `Slot_*`), their complementary roles,
   and why Slot does not replace InferableBrand. Source material for
   this content lives in
   [fp-library/docs/brand-dispatch-traits.md](../../../fp-library/docs/brand-dispatch-traits.md);
   the module docs should either paraphrase or link to it.
2. Add blanket impl from `InferableBrand` to `Slot`.
3. Add direct Slot impls for each multi-brand brand.
4. Change `map` in `fp-library/src/dispatch/functor.rs` to use Slot.
5. Update `impl_kind!` macro to emit Slot impls for brands with
   `#[no_inferable_brand]`.
6. Add integration tests covering non-diagonal and diagonal cases.
7. Update or replace the existing `result_no_inferable_brand.rs` and
   `tuple2_no_inferable_brand.rs` UI tests.
8. Update user-facing docs: `fp-library/docs/brand-inference.md` should
   describe the Slot extension. The design reference
   `fp-library/docs/brand-dispatch-traits.md` should be cross-linked
   from the Slot trait's module docs and from `brand-inference.md`.

### Phase 2: Diagnostic polish

1. Attach `#[diagnostic::on_unimplemented]` to Slot (or a marker
   trait) with helpful messages for ambiguity.
2. Update UI test `.stderr` snapshots to reflect the new messages.
3. Document the diagnostic in user-facing docs.

### Phase 3 (future): Extend to other operations

Apply the same Slot pattern to `bind`, `apply`, `lift2`, `traverse`,
`fold_left`, `fold_right`, `fold_map`, etc. Each is a straightforward
analog of phase 1 for that operation's dispatch trait. Only pursue
after phase 1 is validated in practice.

## Success criteria

- `map(|x: i32| x + 1, Ok::<i32, String>(5))` compiles and maps over
  Ok.
- `map(|e: String| e.len(), Err::<i32, String>("hi".into()))` compiles
  and maps over Err.
- `map(|x: i32| x + 1, Ok::<i32, i32>(5))` fails at compile time with
  a diagnostic mentioning `explicit::map`.
- All existing `map(f, Some(5))` / `map(f, vec![1, 2, 3])` /
  `map(f, &lazy)` style calls continue to work identically.
- All existing `explicit::map::<...>(f, value)` calls continue to
  work unchanged.
- No regression in any existing test suite.
