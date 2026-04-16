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

Four POCs have been run against stable rustc:

1. [slot_production_poc.rs](../../../fp-library/tests/slot_production_poc.rs) - Slot type-level validation with a bespoke `MapDispatch` shim.
2. [slot_valref_poc.rs](../../../fp-library/tests/slot_valref_poc.rs) - Slot composition with the production `FunctorDispatch` + Val/Ref `Marker`.
3. [slot_select_brand_poc.rs](../../../fp-library/tests/slot_select_brand_poc.rs) - attempted to project Brand as an associated type keyed on `(FA, A)`; rejected by coherence for multi-brand types.
4. [slot_assoc_marker_poc.rs](../../../fp-library/tests/slot_assoc_marker_poc.rs) - attempted to move Marker from trait parameter to associated type; rejected by coherence for the Val/Ref impl combination.

Findings:

| Item                              | Finding                                                                                                                                                                                                                    |
| --------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Blocker 1 (blanket + direct)      | **Invalidated.** Option A1 fails with E0119. Option A2 works cleanly.                                                                                                                                                      |
| Blocker 2 (lifetime-generic GAT)  | **Validated** (under option A2).                                                                                                                                                                                           |
| Blocker 3 (return-type normalise) | **Validated for standalone Slot; invalidated when combined with `FunctorDispatch` return type.** Fix: Slot must be a pure marker (no `Out<B>` GAT); return type uses `Apply!(<Brand as Kind>::Of<'a, B>)`.                 |
| Q4 (Val/Ref composition)          | **Unified signature: partial.** Val all-cases and Ref single-brand work via inference; Ref + multi-brand fails (E0283) due to Val/Ref cross-competition. **Split signature (Val-only or Ref-only): works for every case.** |
| Q9 (closure annotations)          | **Validated.** Closure-input annotation required for multi-brand; return-type-only does not suffice.                                                                                                                       |
| Q15 (explicit::map via Slot)      | **Validated.** Turbofish-pinned Brand + Slot bound works for every case including Ref + multi-brand.                                                                                                                       |

Items not addressed by POC (diagnostic routing, apply-side CDI,
partial-rollout UX, do-notation, downstream impact, testing strategy,
projection-type brand ambiguity Q13 - mechanical argument suffices
for Q13) require production code or non-technical decisions.

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
   **Finding (POC): validated under option A2 for a standalone Slot
   dispatch trait; invalidated when combined with the production
   `FunctorDispatch` return type.**
   The first POC
   ([slot_production_poc.rs](../../../fp-library/tests/slot_production_poc.rs))
   used a bespoke `MapDispatch` whose return type is expressed as
   `Self::Out<B>`; there, `<FA as Slot<...>>::Out<B>` normalises
   correctly. The second POC
   ([slot_valref_poc.rs](../../../fp-library/tests/slot_valref_poc.rs))
   attempted to combine a Slot-bounded function with the production
   `FunctorDispatch` (whose dispatcher returns
   `Apply!(<Brand as Kind>::Of<'a, B>)`) and hit an E0308 type
   mismatch: Rust treats `<FA as Slot<'a, Brand, A>>::Out<B>` and
   `<Brand as Kind>::Of<'a, B>` as distinct associated-type
   projections even when they resolve to the same concrete type.
   **Consequence:** if Slot-bounded functions are to share
   `FunctorDispatch` with today's signatures, Slot cannot carry an
   `Out<B>` GAT; it must be a pure marker trait asserting
   `Brand::Of<'a, A> = Self`, and the function return type must use
   `Apply!(<Brand as Kind>::Of<'a, B>)` directly. The second POC
   adopts this shape and compiles.

### Decisions

Cross-cutting architectural choices that span multiple implementation
items.

**Decision A: Coherence approach.**

_Status:_ resolved by POC.

Options:

- **A1. Trust Rust's where-clause coherence with blanket + direct
  impls.** _Trade-off:_ simplest if it works; catastrophic failure
  mode if not. **POC finding: does not work on stable rustc (E0119,
  see blocker 1 above).** No longer viable without specialization
  (unstable) or a different disambiguation mechanism.
- **A2. No blanket; generate direct `Slot` impls for every brand**
  (single- and multi-). _Trade-off:_ more generated code (zero
  runtime impact); coherence trivially safe (each impl keyed on a
  distinct brand); InferableBrand remains as a separate
  "unique-brand assertion" trait. **POC finding: works for
  coherence, lifetime GATs, and return-type normalisation under a
  pure-marker Slot shape (no `Out<B>` GAT, see blocker 3). The
  inference-based path composes with Val dispatch for all cases,
  and with Ref dispatch for single-brand types; Ref + multi-brand
  requires an explicit-brand fallback (see Q4 and Q15).**
- **A3. Sealed marker trait.** Private marker like
  `trait MultiBrand: Sealed` implemented by multi-brand concrete
  types, restructuring the blanket around it. _Trade-off:_ adds
  complexity for limited gain over A2.
- **A4. Invert the design: `Slot` primary, InferableBrand derived.**
  _Trade-off:_ conceptually cleaner; on stable Rust this degrades
  into something equivalent to A2 because the "exists unique Slot
  impl" predicate is not expressible. High migration risk.

_Recommendation:_ **A2.** The POC validates A2 end-to-end. A1 is
definitively off the table on stable. A3 adds complexity for no
meaningful gain. A4 collapses into A2 in practice.

**Decision B: Phase structure.**

_Status:_ open; recommendation tentative.

Options:

- **B1. Keep phases separate** (phase 1 ships, phase 3 follows).
  _Trade-off:_ simplest PR sequence; worst user-facing experience
  between releases — multi-brand `map` works but multi-brand `bind`
  does not.
- **B2. Bundle phase 1 and phase 3 into a single release.**
  _Trade-off:_ larger change per release; consistent public API
  throughout.
- **B3. Internally phased, released together.** Phase 1 lands
  on a development branch as a testbed; phase 3 extends it
  mechanically; release only after both stabilize.
  _Trade-off:_ users never see the intermediate state; slightly
  longer total delivery time.

_Recommendation:_ **B3.** Pre-1.0 stance removes the "bundled
release is too risky" argument. Internal phasing gives the
implementer a testbed while users see only a coherent shipped API.

**Decision C: Annotation requirement UX.**

_Status:_ open; recommendation clear.

Options:

- **C1. Accept the requirement.** Document that multi-brand types
  require closure-input annotations under the inference path.
  `explicit::` remains as the no-annotation alternative via
  turbofish. _Trade-off:_ minor call-site verbosity.
- **C2. Provide alternative signatures that accept annotation
  differently.** _Trade-off:_ unclear what this would look like;
  likely complicates the dispatch story without removing the
  underlying requirement.

_Recommendation:_ **C1.** The requirement follows directly from how
closure-directed inference works (validated by POC). C2 has no
concrete shape and cannot bypass Rust's type inference rules.

### Open questions

Each entry states the concern, enumerates options with trade-offs,
and gives a recommendation. Recommendations are provisional guidance
for the decision-maker, not commitments.

**Q4. Val/Ref dispatch as a second selection axis.**

_Finding (POC): the Ref + multi-brand failure is caused by Val/Ref
cross-competition in the solver, not by Brand ambiguity within the
Ref impl alone._

The second POC
([slot_valref_poc.rs](../../../fp-library/tests/slot_valref_poc.rs))
combined Slot with the production `FunctorDispatch` + Val/Ref
`Marker` in a single inference-based `map_via_slot` signature.
Results for that unified signature:

- Val + single-brand (Option, Vec): works.
- Val + multi-brand (Result with annotated closure): works.
- Ref + single-brand (&Option, &Vec, &Lazy): works.
- Ref + multi-brand (`&Result<i32, String>` with `|x: &i32| ...`):
  does not compile (E0283). The solver treats both Val and Ref
  `FunctorDispatch` impls as candidates and cannot commit a Brand.

A follow-up probe (`map_via_slot_ref_only` in the same POC) pins
the Marker parameter to `Ref`, eliminating the Val impl as a
candidate. **Result: Ref + multi-brand compiles and runs.** The
failure in the unified signature is therefore not a fundamental
limitation of Slot, of lifetime-bearing GATs, or of reference
dispatch; it is a cross-impl resolution issue where the solver
considers both Val and Ref candidates until a Marker is committed,
preventing Brand commitment within the Ref impl alone.

Additionally, an explicit-brand fallback (Q15 prototype) works for
every case including Ref + multi-brand in the unified signature,
because the turbofish pins Brand directly and sidesteps the solver
ordering issue.

Options:

- **a) Accept the inference-level limitation of the unified
  signature.** Inference-based `map` works for Val all-cases and
  Ref single-brand. Ref + multi-brand requires
  `explicit::map::<Brand, ...>` (the Slot-bounded variant
  validated under Q15). _Trade-off:_ Ref multi-brand loses
  inference ergonomics; users learn one consistent fallback rule.
- **b) Redesign the dispatch trait** to project the
  disambiguating parameter as an associated type rather than a
  free trait parameter. Two variants were prototyped against this
  idea: (i) project Brand via a `SelectBrand<'a, A>` trait with
  associated `type Brand`, and (ii) project Marker via an
  `AssocMarkerDispatch` trait with associated `type Marker`.
  **Both prototypes rejected by coherence (E0119).** Rust's
  coherence checker cannot prove non-overlap between the two
  multi-brand impls (variant i) or the two Val/Ref impls
  (variant ii) without a distinguishing trait-argument
  pattern. The parameter-as-disambiguator role is load-bearing on
  stable Rust; removing it trips coherence.
  _Trade-off:_ not achievable without specialization or negative
  impls (both unstable).
- **c) Split Val and Ref into separate Slot-bounded inference
  functions.** Unified `map(f, fa)` is replaced by `map(f, fa)`
  (Val-only) and `ref_map(f, fa)` or `map_ref(f, fa)`
  (Ref-only). **POC probe confirms the Ref-only factoring works
  for Ref + multi-brand.** _Trade-off:_ two inference entry
  points instead of one; user-facing API asymmetry between Val
  and Ref forms; closes the Ref + multi-brand inference gap
  entirely.

_Recommendation:_ **c)**. Option b has been invalidated by two
distinct prototypes that both hit coherence walls, so a unified
inference signature covering Ref + multi-brand is not achievable
on stable Rust without unstable features. Splitting Val and Ref
inference closes the gap at the cost of one additional entry
point. Option a remains a viable fallback if the split proves
awkward in production code. Q15 option b (explicit-brand
turbofish bounded on Slot) is the universal fallback regardless
of which is chosen here.

The general pattern discovered across prototypes: any time a
multi-brand disambiguator (Brand or Marker) is moved from a free
trait parameter into an associated-type projection, coherence
rejects the resulting impls for multi-brand types. The parameter
position is load-bearing on stable Rust.

**Q5. Diagnostic wording precision.**

Two failure modes produce different user errors: "forgot to annotate
the closure" and "diagonal case where annotation won't help." Rust's
`#[diagnostic::on_unimplemented]` is static, so dynamic messages
keyed on the failure mode are not directly supported.

Options:

- **a) Single combined message** covering both cases ("annotate the
  closure input; if that doesn't disambiguate, use `explicit::`").
  _Trade-off:_ works on stable; slightly less targeted.
- **b) Two messages via a custom mechanism** (sealed helper trait,
  procedural macro generating per-type diagnostics). _Trade-off:_
  more targeted; significantly more complex.

_Recommendation:_ **a).** Single message is the only stable option
and covers the common case acceptably. Revisit only if phase 1 user
testing shows confusion.

**Q6. Diagnostic routing between InferableBrand and Slot.**

InferableBrand retains its `on_unimplemented` message; Slot gets a
new one. Under a failure, which diagnostic Rust reports depends on
which trait it resolves against.

Options:

- **a) Prototype and observe.** Adjust wording based on what Rust
  actually reports under real failure scenarios. _Trade-off:_
  empirical; defers full resolution to phase 1.
- **b) Engineer the failure path** so exactly one diagnostic fires
  (for example, remove InferableBrand's message now that Slot's
  covers the same ground). _Trade-off:_ potentially loses coverage
  for code paths that still bound on InferableBrand directly.

_Recommendation:_ **a).** The right wording depends on what Rust's
error-reporting machinery actually does in the new configuration.
Decide from evidence during phase 1.

**Q7. Apply-side closure-directed inference.**

`Semiapplicative::apply` takes no outer closure but carries an
`Fn(A) -> B` payload inside `ff`. In principle the payload's function
type could drive `Slot` dispatch in phase 3.

Options:

- **a) Implement apply with CDI via the Fn payload.** _Trade-off:_
  consistent with the rest of phase 3; macro/trait-resolution
  complexity to validate.
- **b) Keep apply explicit-only for multi-brand types.**
  _Trade-off:_ fewer moving parts; surface asymmetry (apply alone
  requires explicit:: while other operations don't).

_Recommendation:_ Decide in phase 3 with evidence from a targeted
prototype. If payload-driven inference works as cleanly as the
closure-driven case, option a). Otherwise b).

**Q8. Partial-rollout inconsistency during phase 1.**

If phase 1 ships before phase 3, multi-brand `map` works while
multi-brand `bind`/`fold_*`/etc. do not. The pre-1.0 stance reduces
this concern's weight (see Decision B).

Options:

- **a) Accept the inconsistency** (Decision B option B1). Ship phase
  1 and phase 3 separately. _Trade-off:_ faster phase 1 delivery;
  transient UX inconsistency.
- **b) Avoid the inconsistency** (Decision B options B2/B3). Ship
  phase 1 and phase 3 together. _Trade-off:_ longer delivery; no
  transient state visible to users.

_Recommendation:_ **b), via Decision B's B3.** Pre-1.0 stance makes
the delivery delay acceptable in exchange for a coherent user-facing
rollout.

**Q9. Closure-annotation fragility.**

_Finding (POC): validated._ Single-brand types accept unannotated
closures. Multi-brand non-diagonal types require closure-input
annotations (`|x: i32| ...`); return-type-only annotations do not
suffice.

Not a decision point; noted for documentation. The finding should
be prominently documented in user-facing docs (both in
`fp-library/docs/brand-inference.md` and in `map`'s doc comment
after phase 1).

**Q10. Do-notation macro behavior (`m_do!`, `a_do!`).**

These macros desugar to chained `bind`/`apply` calls. After phase 3
they will require closure annotations when operating on multi-brand
types.

Options:

- **a) Audit before phase 3 completes.** Run the existing
  do-notation tests against multi-brand types with annotations; add
  new tests covering edge cases. _Trade-off:_ catches issues
  early.
- **b) Only audit if issues surface** in normal testing.
  _Trade-off:_ lower upfront cost; risk of late-discovered
  incompatibility.

_Recommendation:_ **a).** Low cost; ensures the macros remain
usable in realistic multi-brand contexts. Listed as workflow
note 4.

**Q11. Downstream crate impact.**

The `#[no_inferable_brand]` attribute's semantics change (previously:
suppress InferableBrand; after: suppress InferableBrand + emit direct
Slot). Pre-1.0 stance reduces this concern's weight.

Options:

- **a) Document in changelog.** Accept the breakage; provide a
  migration note for downstream brand authors. _Trade-off:_
  simplest.
- **b) Add a migration shim** that re-emits old-semantics behavior
  under a compatibility flag. _Trade-off:_ prolongs attribute
  duality; incompatible with Q14 renaming.

_Recommendation:_ **a).** Pre-1.0 stance explicitly accepts
changelog-documented breakage in exchange for a cleaner end state.
See Q14 for whether renaming happens at the same time.

**Q12. Testing strategy.**

Implementation checklist, not a decision point:

- All existing single-brand doctests should compile identically.
- All existing `explicit::map::<...>` doctests on multi-brand types
  should stay as-is (they document the explicit path).
- Add new positive doctests for closure-directed resolution.
- Add UI tests for diagonal failure cases.
- Promote the production POC to a proper integration test (or
  remove it once the real implementation subsumes every case it
  covers).

**Q13. Slot generation scope for projection-type brands.**

`impl_kind!` auto-skips InferableBrand for brands whose `Of` target
contains `Apply!` or `::` (for example
`BifunctorFirstAppliedBrand<ResultBrand, A>`). Under option A2,
should these derived brands also get direct `Slot` impls?

Options:

- **a) Keep the projection auto-skip rule for Slot too.**
  Projection-type brands remain explicit-only. _Trade-off:_ simplest
  macro change; bifunctor-derived mapping is not CDI-accessible.
- **b) Generate Slot for projection-type brands.** _Trade-off:_
  uniform brand landscape; `Result<A, E>` at arity 1 would match 4
  brands instead of 2, amplifying closure-input-type ambiguities.
- **c) Generate Slot routing through
  `Bifunctor::bimap(identity, f, _)`.** _Trade-off:_ technically
  elegant; macro logic grows substantially.

_Recommendation:_ **a).** Projection-type brands exist primarily for
architectural completeness (showing Bifunctor subsumes Functor in
one direction), not as primary user-facing paths. Keeping them
explicit-only avoids ambiguity explosion without losing capability.

**Q14. Attribute naming under option A2.**

`#[no_inferable_brand]` now means "suppress InferableBrand AND emit
direct Slot." Types with this attribute ARE inferable via closure
direction; the name overpromises.

Options:

- **a) Keep the name.** Document to clarify semantics.
  _Trade-off:_ no breakage; name remains misleading.
- **b) Rename** to something semantically accurate:
  `#[multi_brand]`, `#[no_unique_brand]`, etc. _Trade-off:_
  one-time breakage within the pre-1.0 window; accurate.
- **c) Split** into `#[no_inferable_brand]` (suppression) and
  `#[multi_brand]` (semantic flag), composable. _Trade-off:_ most
  flexible; more attribute surface.

_Recommendation:_ **b).** Pre-1.0 stance accepts the breakage;
accurate naming compounds positively over time. Reasonable
candidates: `#[multi_brand]` (affirms what is true), or
`#[no_unique_brand]` (inverse of the unique-brand concept).

**Q15. `explicit::` module reorganization under Slot.**

_Finding (POC): option b validated._ The second POC
([slot_valref_poc.rs](../../../fp-library/tests/slot_valref_poc.rs))
prototyped a `map_explicit` variant with a Slot bound and
turbofish-pinned Brand. It compiles and works for every tested
case, **including Ref + multi-brand** (which fails under inference
see Q4). Pinning Brand removes the trait-selection ambiguity that
defeats the inference-based path.

Under A2, `explicit::` still exists for the diagonal case and for
deliberate explicit-brand usage. Today it dispatches through direct
Brand+FA. Should it route through Slot internally?

Options:

- **a) Keep today's `explicit::map::<Brand, A, B, FA, Marker>`
  signature.** _Trade-off:_ no churn for existing users; two
  dispatch pipelines coexist internally.
- **b) Rewrite `explicit::map` to bound on Slot,** with Brand
  pinned via turbofish: `explicit::map::<Brand, _, _, _, _>(f, fa)`.
  _Trade-off:_ unifies dispatch; the turbofish surface stays the
  same shape (Brand + inference placeholders for A, B, FA, Marker)
  but the function becomes the canonical fallback for every case
  inference cannot handle (including Ref + multi-brand per Q4).
  **POC-validated.**
- **c) Remove `explicit::` entirely.** _Trade-off:_ largest
  breakage; probably too far even pre-1.0.

_Recommendation:_ **b).** Unified dispatch is cleaner; the
Q4-identified Ref + multi-brand case has no inference path that
works on stable rustc, so `explicit::` carries genuine value as the
universal fallback. Making it Slot-bounded keeps the code paths
consistent.

**Q16. Compile-time regression risk from per-brand Slot generation.**

Option A2 approximately doubles the macro-generated trait-impl code
per brand. Compile-time impact is unknown.

Options:

- **a) Measure post-implementation.** Accept if small (for example
  under 5%). Revisit only if worse. _Trade-off:_ empirical; defers
  optimization until there is data.
- **b) Fuse InferableBrand and Slot generation** into a single
  macro pass. _Trade-off:_ minor win at best; more complex macro
  code.
- **c) Only generate Slot for brands participating in
  closure-directed dispatch** (exclude tags like `SendThunkBrand`).
  _Trade-off:_ complicates macro logic for uncertain savings.

_Recommendation:_ **a).** Don't optimize without data. Listed as
workflow note 9.

**Q17. Macro hash coordination for `Slot_{hash}`.**

`Slot_{hash}` must share the content hash used by `Kind_{hash}` and
`InferableBrand_{hash}` so `impl_kind!`-emitted impls target the
right trait. More implementation detail than open decision.

Options:

- **a) Share the existing hash generator.** _Trade-off:_ consistent
  with today's Kind/InferableBrand coordination.
- **b) Re-derive hashes independently.** _Trade-off:_ risks silent
  drift; strictly worse.

_Recommendation:_ **a).** Obvious choice; b) is a hazard, not a
real alternative.

### Workflow notes

Process recommendations about how to execute the plan, as opposed
to design decisions about what to build.

1. **Extend the POC before committing to implementation.** _Done._
   The production-style POC
   ([fp-library/tests/slot_production_poc.rs](../../../fp-library/tests/slot_production_poc.rs))
   covers the three blockers plus Val/Ref reference-blanket
   resolution and the closure-annotation matrix. Remaining POC gap:
   full `FunctorDispatch`/`Marker` composition (Q4).

2. **Adopt option A2 for coherence.** POC invalidated A1 on stable
   rustc; A2 is the path that actually compiles. The plan now
   records this as Decision A's recommendation.

3. **Release phase 1 and phase 3 together (Decision B's B3).**
   Implement phase 1 as a testbed, extend to other closure-taking
   operations before publishing, release only after both are stable.

4. **Audit do-notation before phase 3** (see Q10). Verify `m_do!`
   and `a_do!` produce well-typed code for multi-brand types with
   reasonable closure annotations.

5. **Add a migration note for downstream crates** (see Q11).
   Changelog entry and doc update explaining the
   `#[no_inferable_brand]` semantic shift (plus any rename from
   Q14) when the release ships.

6. **Defer Q5 and Q7** (diagnostic wording, apply via Fn payload)
   until there is a working phase 1 prototype; decide with evidence
   rather than upfront.

7. **Treat the POC as specification, not throwaway code.** Every
   case the POC compiles should continue to compile in the
   production implementation. Regressions from POC behavior are
   regressions from the plan's stated capability.

8. **Run a second POC for Val/Ref + FunctorDispatch + Slot
   composition** (Q4). _Done._
   [slot_valref_poc.rs](../../../fp-library/tests/slot_valref_poc.rs)
   combines Slot (as a pure marker, per blocker 3's caveat) with
   the production `FunctorDispatch` and Val/Ref `Marker`. Findings
   absorbed into Q4 and Q15.

9. **Benchmark compile-time impact** of per-brand Slot generation
   as part of phase 1 acceptance (Q16). Detects regressions early
   rather than post-release.

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
