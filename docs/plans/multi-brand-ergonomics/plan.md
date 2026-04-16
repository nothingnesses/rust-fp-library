# Plan: Multi-Brand Ergonomics via Slot Trait

**Status:** DRAFT (design validated; implementation ready)

Replace the `InferableBrand_*` trait family with a new `Slot_*` family
that supports closure-directed inference for multi-brand concrete types
(`Result`, `Pair`, `Tuple2`, `ControlFlow`, `TryThunk`) while preserving
the unified Val/Ref dispatch users have today.

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

### The `Slot` trait family

One trait per Kind arity (same pattern as today's `Kind_*` and
`InferableBrand_*` families):

```rust
pub trait Slot_cdc7cd43dac7585f<'a, Brand, A: 'a>
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
   `Slot<ResultErrAppliedBrand<E>, A>` and `Slot<ResultOkAppliedBrand<T>, A>`
   as structurally distinct trait heads even when both cover
   `Result<_, _>`.
2. **Marker is an associated type** projected from FA's reference-ness.
   Direct impls for owned types set `type Marker = Val`; a single
   `&T` blanket sets `type Marker = Ref` uniformly. When dispatch
   code projects `<FA as Slot<...>>::Marker`, the Marker commits from
   FA alone, before `(Brand, A)` are resolved - eliminating the
   Val/Ref cross-competition that otherwise blocks Ref + multi-brand
   inference.

### Impl landscape

Every brand gets a direct `Slot` impl. No blanket from `InferableBrand`
(that combination fails coherence; see POC 3 invalidation below).

Single-brand types have one impl per arity:

```rust
impl<'a, A: 'a> Slot_*<'a, OptionBrand, A> for Option<A> {
    type Marker = Val;
}
```

Multi-brand types have one impl per brand:

```rust
impl<'a, A: 'a, E: 'static> Slot_*<'a, ResultErrAppliedBrand<E>, A>
    for Result<A, E>
{
    type Marker = Val;
}

impl<'a, T: 'static, A: 'a> Slot_*<'a, ResultOkAppliedBrand<T>, A>
    for Result<T, A>
{
    type Marker = Val;
}
```

The reference blanket:

```rust
impl<'a, T: ?Sized, Brand, A: 'a> Slot_*<'a, Brand, A> for &T
where
    T: Slot_*<'a, Brand, A>,
    Brand: Kind_*,
{
    type Marker = Ref;
}
```

### The unified inference wrapper

`map` (and sibling closure-taking operations) binds on `Slot` with
`Marker` projected:

```rust
pub fn map<'a, FA, A: 'a, B: 'a, Brand>(
    f: impl FunctorDispatch<
        'a,
        Brand,
        A,
        B,
        FA,
        <FA as Slot_*<'a, Brand, A>>::Marker,
    >,
    fa: FA,
) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
    Brand: Kind_*,
    FA: Slot_*<'a, Brand, A>,
```

### Coverage matrix

| Case                         | Behaviour                                   |
| ---------------------------- | ------------------------------------------- |
| Val + single-brand           | Inference (no change from today)            |
| Val + multi-brand            | Inference via closure input                 |
| Ref + single-brand           | Inference (no change from today)            |
| Ref + multi-brand            | Inference via closure input                 |
| Multi-brand diagonal (`T=T`) | Compile error; use `explicit::`             |
| Unannotated multi-brand      | Compile error; annotate or use `explicit::` |

### How closure-directed inference resolves Brand

For `map(|x: i32| x + 1, Ok::<i32, String>(5))`:

1. `FA = Result<i32, String>` pinned by the argument.
2. `Marker` projected via Slot: Result is owned, so Marker = Val.
3. With Marker committed, FunctorDispatch picks the Val impl. Its
   `Fn(A) -> B` bound pins `A = i32` from the closure.
4. With `A = i32`, only the `ResultErrAppliedBrand<String>` Slot
   impl unifies with FA = `Result<i32, String>`. Brand commits.
5. Dispatch proceeds.

For `&Result<i32, String>` with `|x: &i32| *x + 1`:

1. `FA = &Result<i32, String>`.
2. The `&T` blanket projects Marker = Ref immediately.
3. FunctorDispatch Ref impl applies; `Fn(&A) -> B` pins A from `&i32`.
4. Inner Slot impl on `Result<i32, String>` resolves to
   `ResultErrAppliedBrand<String>` with A = i32.
5. Dispatch proceeds through `RefFunctor::ref_map`.

### Replacement of `InferableBrand`

Under Decision D3 (see Decisions), `InferableBrand_*` is removed
entirely. Single-brand types previously used
`<FA as InferableBrand>::Brand` as an associated-type shortcut; this
becomes `FA: Slot<Brand, A>` with Brand threaded explicitly through
the signature. Functionally equivalent; syntactically more uniform.

The `#[no_inferable_brand]` attribute is renamed to `#[multi_brand]`
(Decision Q14) and gains new semantics: it tells `impl_kind!` to
emit multiple Slot impls (one per applicable brand) rather than a
single unique-brand Slot impl.

## Validated via POCs

Seven POCs on stable rustc establish feasibility:

| POC                                                                                  | Finding                                                                                                                                                     |
| ------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [slot_production_poc.rs](../../../fp-library/tests/slot_production_poc.rs)           | Slot type-level validation; A2 coherence works; lifetime-generic GATs OK.                                                                                   |
| [slot_valref_poc.rs](../../../fp-library/tests/slot_valref_poc.rs)                   | Unified signature via Slot + production FunctorDispatch - validated for Val + all, Ref + single-brand. Ref + multi-brand exposed Val/Ref cross-competition. |
| [slot_select_brand_poc.rs](../../../fp-library/tests/slot_select_brand_poc.rs)       | Alternative with Brand as associated-type projection - rejected by coherence.                                                                               |
| [slot_assoc_marker_poc.rs](../../../fp-library/tests/slot_assoc_marker_poc.rs)       | Alternative with Marker as dispatch-trait associated type - rejected by coherence.                                                                          |
| [slot_marker_via_slot_poc.rs](../../../fp-library/tests/slot_marker_via_slot_poc.rs) | **Adopted design.** Marker projected via Slot closes Ref + multi-brand gap in unified signature.                                                            |
| [slot_arity2_poc.rs](../../../fp-library/tests/slot_arity2_poc.rs)                   | Pattern generalises to arity 2 (bimap).                                                                                                                     |
| [slot_bind_poc.rs](../../../fp-library/tests/slot_bind_poc.rs)                       | Pattern generalises to `bind` (closure returns container); single-brand and multi-brand both work.                                                          |

Key generalisation findings:

- Trait family works across Kind arities (POC 6).
- Pattern works across closure shapes: `Fn(A) -> B` for Functor and
  `Fn(A) -> Of<B>` for Semimonad (POC 7).
- Multi-brand coverage includes `Result` via `ResultErrAppliedBrand<E>`
  for both Functor and Semimonad (the latter already existed in the
  library matching PureScript's `Bind (Either e)`).

## Decisions

### Decision A: Impl layout

**A2** (adopted). Every brand gets a direct `Slot` impl. No blanket
from `InferableBrand`.

_Rationale:_ POC 3 demonstrated the blanket (`impl<FA: InferableBrand>
Slot<FA::Brand, A> for FA`) conflicts with direct multi-brand impls
via E0119 - Rust's coherence checker cannot prove non-overlap through
where-clauses in the face of potential downstream `InferableBrand`
impls. Direct impls per brand are trivially coherence-safe because
their trait-argument patterns differ by Brand.

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

**D3** (recommended). Eliminate `InferableBrand_*` entirely.
Introduce `Slot_*` as its replacement.

_Rationale:_ After D3, single-brand types have one Slot impl per
arity (Marker = Val), multi-brand types have multiple (all Marker =
Val), references have the blanket (Marker = Ref). One trait family,
one conceptual model, one macro generation path. POCs 5, 6, 7
validate this shape for `map`, `bimap`, and `bind`.

### Decision E: Attribute naming

Rename `#[no_inferable_brand]` to **`#[multi_brand]`** (Q14 option b).

_Rationale:_ The old name overpromises after D3 - types with the
attribute ARE inferable via closure direction. The new name
describes what is true (multiple brands) rather than what is no
longer true (no unique brand). Pre-1.0 stance accepts the breakage.

### Decision F: `explicit::map` shape

Rewrite `explicit::map` to bound on `Slot` with Brand pinned via
turbofish (Q15 option b).

_Rationale:_ Under D3, there is only one reverse-mapping trait family
(Slot). `explicit::map` becomes the universal fallback for cases
inference cannot handle (e.g., `Result<T, T>` diagonal). The
signature shape stays familiar:
`explicit::map::<Brand, _, _, _, _>(f, fa)`. POC 2 validated this
path works for every case including Ref + multi-brand.

### Decision G: Projection-type brands

Projection brands (e.g., `BifunctorFirstAppliedBrand<ResultBrand, A>`)
remain explicit-only (Q13 option a). `impl_kind!` skips Slot
generation when the brand's `Of` target contains `Apply!` or `::`.

_Rationale:_ These brands exist for architectural completeness
(showing Bifunctor subsumes Functor in one direction), not as primary
user paths. Generating Slot for them would create additional Slot
candidates for types like `Result<A, E>` at arity 1, amplifying
closure-input ambiguities without user-facing benefit.

## Integration surface

### Will change

| Component                                                                                                                                      | Change                                                                                                                                         |
| ---------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `fp-library/src/kinds.rs`                                                                                                                      | Remove `InferableBrand_*` family; add `Slot_*` family with associated `Marker`.                                                                |
| `fp-library/src/brands.rs`                                                                                                                     | No changes; brand struct definitions stay.                                                                                                     |
| `fp-library/src/types/*/mod.rs`                                                                                                                | Rename `#[no_inferable_brand]` to `#[multi_brand]`. Add `#[multi_brand]` to arity-1 multi-brand brands (already marked; just rename).          |
| `fp-macros/src/hkt/impl_kind.rs`                                                                                                               | Generate direct `Slot_*` impls for every brand (single and multi). Skip when brand's `Of` is a projection (contains `Apply!` or `::`).         |
| `fp-macros/src/hkt/trait_kind.rs`                                                                                                              | Generate `Slot_{hash}` alongside `Kind_{hash}` at every arity.                                                                                 |
| `fp-library/src/dispatch/functor.rs`                                                                                                           | `map` rebinds from `InferableBrand` to `Slot`; `Brand` becomes a function type parameter; `Marker` projected from Slot.                        |
| `fp-library/src/dispatch/semimonad.rs`                                                                                                         | Same pattern as functor.rs for `bind`, `bind_flipped`, `join`, `compose_kleisli*`.                                                             |
| `fp-library/src/dispatch/bifunctor.rs`                                                                                                         | Same pattern at arity 2 for `bimap`.                                                                                                           |
| `fp-library/src/dispatch/bifoldable.rs`, `bitraversable.rs`, `foldable.rs`, `traversable.rs`, `filterable.rs`, `lift.rs`, `semiapplicative.rs` | Same pattern for each operation's inference wrapper.                                                                                           |
| `fp-library/tests/ui/*.rs`                                                                                                                     | Delete or rewrite `result_no_inferable_brand.rs` and `tuple2_no_inferable_brand.rs`; add positive and negative UI tests for the new behaviour. |
| `fp-library/docs/brand-inference.md`                                                                                                           | Update to describe Slot; cross-link to `brand-dispatch-traits.md`.                                                                             |
| `fp-library/docs/brand-dispatch-traits.md`                                                                                                     | Update to reflect single-trait-family design (D3).                                                                                             |

### Unchanged

- **Optics subsystem** (`Lens`, `Prism`, `Iso`, `Traversal`, etc.).
- **Benchmarks**: no code changes. Performance validated
  post-implementation.
- **Stack safety / `TailRec`**: unrelated.
- **Serde integration**: unrelated.
- **FunctorDispatch, BindDispatch, BimapDispatch trait shapes**: their
  Val/Ref impls are untouched. Only the inference wrappers that call
  them are rebound.

### Operations that cannot use Slot inference

Operations without a closure cannot drive brand disambiguation from
closure input. They continue to require `explicit::` for multi-brand
types (mechanism unchanged from today; Slot just gives them a unique
dispatch path):

- `Pointed::pure` (return-type inference problem).
- `Alt::alt`, `Plus::empty` (no closure).
- `Traversable::sequence` (no closure).

`Semiapplicative::apply` has no direct closure but has an
`Fn(A) -> B` payload inside `ff`; whether to drive Slot from that is
Q7, deferred.

## Remaining open questions

Items that still need decisions. Each entry lists approaches with
trade-offs and a tentative recommendation.

### Q5: Diagnostic wording for ambiguity failures

`Slot`'s `#[diagnostic::on_unimplemented]` message must cover two
failure modes with the same static text: "forgot to annotate the
closure" and "diagonal case where annotation won't help."

Approaches:

- **a) Single combined message on `Slot`.** One static `on_unimplemented`
  message that mentions both remedies: "annotate the closure input;
  if that doesn't disambiguate, use `explicit::map::<Brand, ...>`."
  _Trade-off:_ works on stable Rust; same text for both failure
  modes, so the annotation hint is slightly misleading in the
  diagonal case.
- **b) Per-type diagnostics generated by `impl_kind!`.** The macro
  emits a custom `on_unimplemented` string per concrete type, naming
  the available brands.
  _Trade-off:_ more targeted; macro complexity grows; diagnostic
  strings must track brand renames.
- **c) Sealed helper traits with structurally-distinct impls per
  failure mode.** Rust reports the most-specific unsatisfied trait
  first. Design separate `MissingAnnotation` and `DiagonalAmbiguity`
  markers.
  _Trade-off:_ most targeted; significant elaboration of the trait
  landscape; uncertain whether Rust's error reporting consistently
  picks the intended trait.

_Recommendation:_ **a)**. Start simple. Revisit only if phase 1/2
user testing shows confusion. Escalating later is easier than
pulling back from premature complexity.

A secondary concern (previously tracked as Q6): which trait's
diagnostic fires when. Under Decision D3, `InferableBrand` is
removed, so `Slot` is the only reverse-mapping trait with a
diagnostic. But upstream trait bounds (`Brand: Functor`,
`Brand: RefFunctor`, etc.) may fire first and report a less helpful
message. If that happens, phase 3 should add `on_unimplemented`
overrides to the type-class traits pointing back at the Slot
diagnostic. Decide from observed behaviour in phase 1.

### Q7: Apply-side closure-directed inference

`Semiapplicative::apply` takes no outer closure, but `ff` carries
an `Fn(A) -> B` payload. In principle the payload's function type
could drive Slot dispatch.

Approaches:

- **a) Implement `apply` with CDI via the `Fn` payload.** Key Slot
  on `(FA, payload_input_type)` where the payload is an
  `Fn(A) -> B`. A multi-brand `apply` call would infer Brand from
  the combination of `ff`'s payload input and `fa`'s container
  shape.
  _Trade-off:_ uniform dispatch experience across all closure-taking
  operations; non-trivial trait-resolution machinery - the payload
  is nested inside the container, which complicates the Slot bound.
  Feasibility should be validated by a targeted POC before
  committing.
- **b) Keep `apply` explicit-only for multi-brand.** `apply`
  continues to require `explicit::apply::<Brand, ...>` on multi-brand
  types; single-brand types work via inference as today.
  _Trade-off:_ simplest; surface asymmetry - users who rely on
  `apply` on multi-brand types hit `explicit::` while `map`/`bind`
  do not.
- **c) Defer the decision entirely.** Ship phases 1/2 with `apply`
  untouched (it currently uses InferableBrand at arity 1, which is
  gone under D3; at minimum, rebind to Slot but keep the "no inference
  for multi-brand" behaviour). Revisit in a follow-up plan.
  _Trade-off:_ same end state as b) for now; leaves room for a
  cleaner eventual answer.

_Recommendation:_ **c)**, escalating to a) if a phase 2 prototype
shows the Fn-payload path is tractable. Deferring is safer than
committing to a) without evidence or to b) as a permanent decision
that may turn out to be suboptimal.

### Q10: Do-notation macro behaviour

`m_do!` and `a_do!` desugar to chained `bind`/`apply` calls. Under
phase 2, multi-brand do-notation will require closure annotations
for the bound variables.

Approaches:

- **a) Audit before shipping.** Run existing `tests/do_notation.rs`
  and `tests/ado_notation.rs` against multi-brand types; add tests
  for annotated and unannotated closures; document the annotation
  requirement in macro docs.
  _Trade-off:_ catches incompatibilities during implementation; some
  upfront investigation cost.
- **b) Ship without audit; react if issues surface.** Rely on the
  existing test suite to catch problems post-implementation.
  _Trade-off:_ less upfront work; risks late-discovered
  incompatibility that pushes the release.
- **c) Extend the macros to emit type annotations.** `m_do!` could
  detect multi-brand types and inject closure-input annotations in
  the expansion based on the initial container's type.
  _Trade-off:_ potentially nicer ergonomics at the call site;
  significant macro complexity; may not work when the macro can't
  statically determine the container type.

_Recommendation:_ **a)**. Low cost relative to the risk of shipping
broken do-notation. Start with c) only if a) reveals the annotation
requirement is severely disruptive in practice.

### Q16: Compile-time regression risk

Every brand gets a direct Slot impl under Decision A2. Compile-time
impact is unknown.

Approaches:

- **a) Measure post-implementation.** Build the full workspace
  before and after the change; compare. Accept regressions below
  ~5%; investigate if worse.
  _Trade-off:_ empirical; no upfront optimization. Data-driven
  response.
- **b) Only generate Slot for brands that participate in
  closure-directed dispatch.** Exclude tag-only brands like
  `SendThunkBrand` (which carry no type-class impls and so can't
  appear in Slot-routed dispatch).
  _Trade-off:_ reduces generated code; complicates macro logic
  (need a signal for which brands to include); uncertain savings
  without measurement.
- **c) Hand-optimize the `impl_kind!` expansion** if measurement
  reveals a regression. Options include inlining repeated
  substructure, caching hash computations, or deduplicating shared
  trait paths.
  _Trade-off:_ reactive optimization; tackles real regressions
  without pre-emptive complexity.

_Recommendation:_ **a), then c) if needed**. Measure as part of
phase 1 acceptance. Option b) is a targeted optimization that
should only be considered after measurement shows a problem; don't
complicate the macro for uncertain savings.

## Out of scope

Permanently excluded from this plan; revisit only if the design
constraints change.

- **Named helpers** (`map_ok`, `map_err`, `map_fst`, etc.). Under
  Slot these would only fire for diagonal cases (`Result<T, T>` etc.)
  which are rare. `explicit::map::<Brand, ...>(f, fa)` covers the
  same ground. If real-world usage shows diagonal cases are frequent
  enough to warrant sugar, a separate proposal can add them.
- **Primary brand designation** (`#[primary_brand]`). Superseded by
  Slot's symmetric treatment of all brands; no role for it.
- **Non-closure operations** (`pure`, `empty`, `alt`, `sequence`).
  Closure-directed inference structurally cannot apply. These
  continue to use `explicit::` for multi-brand.
- **Newtype wrappers** for disambiguation. Conflicts with the
  library's "use your normal types" design principle.
- **Split `map` into `map` + `ref_map`**. This would regress the
  unified Val/Ref dispatch users have today, since POC 5 shows the
  unified signature handles all four Val/Ref x single/multi-brand
  cases.
- **Specialization or negative-impl-based approaches.** All
  require unstable features.

## Implementation phasing

All three phases are implemented on a development branch and
released together (Decision B3).

### Phase 1: Core Slot and single-operation integration

1. Add `Slot_*` trait family to `fp-library/src/kinds.rs`. Include
   associated type `Marker`. The module-level doc summarises the
   trait trio (`Kind_*`, `Slot_*`, and the historical `InferableBrand_*`
   if kept as a compatibility re-export - likely not needed under D3).
2. Update `trait_kind!` to emit `Slot_{hash}` at every arity it
   already emits `Kind_{hash}` for.
3. Update `impl_kind!` to emit direct `Slot` impls. Single brands get
   `Marker = Val`. The `&T` blanket for references (generated once
   globally) gives `Marker = Ref`. Projection brands (whose `Of`
   contains `Apply!` or `::`) are skipped.
4. Rename `#[no_inferable_brand]` to `#[multi_brand]` in macro input
   and all use sites. For multi-brand brands, generate one Slot impl
   per brand variant.
5. Remove `InferableBrand_*` trait family and all impls (Decision D3).
6. Rewrite `map` in `fp-library/src/dispatch/functor.rs` to bind on
   `Slot` with Marker projected.
7. Rewrite `explicit::map` to bind on `Slot` with Brand pinned via
   turbofish (Decision F).
8. Update UI tests: remove `result_no_inferable_brand.rs` and
   `tuple2_no_inferable_brand.rs`; add UI tests for closure-directed
   success cases, diagonal failures, and unannotated-multi-brand
   failures.
9. Add integration tests covering every Val/Ref x single/multi-brand
   cell of the coverage matrix.

### Phase 2: Extend to remaining closure-taking operations

Repeat the phase 1 rebinding for:

- `bind`, `bind_flipped`, `join`, `compose_kleisli*`.
- `bimap` at arity 2.
- `fold_left`, `fold_right`, `fold_map`.
- `traverse` (outer brand only).
- `filter`.
- `lift2`.

Each is a mechanical analogue of phase 1 for its dispatch trait.
Audit do-notation macros (Q10).

### Phase 3: Diagnostic polish and docs

1. Attach `#[diagnostic::on_unimplemented]` to `Slot` with wording
   determined from phase 1/2 observations (Q5/Q6).
2. Update `fp-library/docs/brand-inference.md` and
   `fp-library/docs/brand-dispatch-traits.md` for the single-family
   design.
3. Update `map`'s doc comment to document the closure-annotation
   requirement for multi-brand types (Decision C).
4. Update `CLAUDE.md` and any other developer-facing docs.

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
- Compile-time regression under 5% versus pre-change baseline (Q16).
- No regression in existing test suites, benchmarks, or doctests.

## Reference material

- Design analysis: [analysis/multi-brand-evaluation.md](./analysis/multi-brand-evaluation.md).
- POCs: listed in the Validated via POCs section above.
- Brand-dispatch traits overview: [fp-library/docs/brand-dispatch-traits.md](../../../fp-library/docs/brand-dispatch-traits.md).
- Parent brand-inference plan: [../brand-inference/plan.md](../brand-inference/plan.md).
