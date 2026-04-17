# Plan: Multi-Brand Ergonomics via InferableBrand Trait

**Status:** DRAFT (design validated; implementation ready)

Add a new `InferableBrand_*` trait family that supports closure-directed
inference for multi-brand concrete types (`Result`, `Pair`, `Tuple2`,
`ControlFlow`, `TryThunk`) while preserving the unified Val/Ref dispatch
users have today. The existing `UniqueBrand_*` family (renamed from
today's `InferableBrand_*`) is retained for non-closure operations
(`pure`, `empty`, `join`, `alt`, `sequence`) that need a unique brand
projection.

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

Today's `UniqueBrand`-based `map` refuses multi-brand types:

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
`UniqueBrand_*` families):

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

Every brand gets a direct `InferableBrand` impl. No blanket from `UniqueBrand`
(that combination fails coherence; see POC 3 invalidation below).

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

### Relationship to `UniqueBrand`

Under Decision D, today's `InferableBrand_*` is renamed to
`UniqueBrand_*` and retained for non-closure operations.
Closure-taking operations move from `UniqueBrand` to the new
`InferableBrand`: `<FA as UniqueBrand>::Brand` (associated-type
projection) is replaced by `FA: InferableBrand<Brand, A>` (Brand as
an explicit type parameter resolved via closure input).

The `#[no_inferable_brand]` attribute is renamed to `#[multi_brand]`
(Decision E). Multi-brand types get InferableBrand impls (one per
brand) but no UniqueBrand impl. Single-brand types get both.

## Validated via POCs

Seven POCs on stable rustc establish feasibility:

| POC                                                                                  | Finding                                                                                                                                                                                         |
| ------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [slot_production_poc.rs](../../../fp-library/tests/slot_production_poc.rs)           | InferableBrand type-level validation; A2 coherence works; lifetime-generic GATs OK.                                                                                                             |
| [slot_valref_poc.rs](../../../fp-library/tests/slot_valref_poc.rs)                   | Unified signature via InferableBrand + production FunctorDispatch - validated for Val + all, Ref + single-brand. Ref + multi-brand exposed Val/Ref cross-competition.                           |
| [slot_select_brand_poc.rs](../../../fp-library/tests/slot_select_brand_poc.rs)       | Alternative with Brand as associated-type projection - rejected by coherence.                                                                                                                   |
| [slot_assoc_marker_poc.rs](../../../fp-library/tests/slot_assoc_marker_poc.rs)       | Alternative with Marker as dispatch-trait associated type - rejected by coherence.                                                                                                              |
| [slot_marker_via_slot_poc.rs](../../../fp-library/tests/slot_marker_via_slot_poc.rs) | **Adopted design.** Marker projected via InferableBrand closes Ref + multi-brand gap in unified signature.                                                                                      |
| [slot_arity2_poc.rs](../../../fp-library/tests/slot_arity2_poc.rs)                   | Pattern generalises to arity 2 (bimap).                                                                                                                                                         |
| [slot_bind_poc.rs](../../../fp-library/tests/slot_bind_poc.rs)                       | Pattern generalises to `bind` (closure returns container); single-brand and multi-brand both work.                                                                                              |
| [slot_apply_poc.rs](../../../fp-library/tests/slot_apply_poc.rs)                     | Pattern generalises to `apply` and `ref_apply` (two containers sharing a Brand); Brand inferred from `ff`'s Fn-payload and `fa`'s value simultaneously; multi-brand works for both Val and Ref. |

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

**A2** (adopted). Every brand gets a direct `InferableBrand` impl. No blanket
from `UniqueBrand`.

_Rationale:_ POC 3 demonstrated the blanket (`impl<FA: UniqueBrand>
InferableBrand<FA::Brand, A> for FA`) conflicts with direct multi-brand impls
via E0119 - Rust's coherence checker cannot prove non-overlap through
where-clauses in the face of potential downstream `UniqueBrand`
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

### Decision D: Trait naming and coexistence

Rename the current `InferableBrand_*` to `UniqueBrand_*`. Introduce
a new `InferableBrand_*` trait family for closure-directed inference
(the trait previously called Slot in POCs). Both trait families
coexist permanently:

- **`UniqueBrand_*`** (renamed from today's `InferableBrand_*`):
  retains its current shape (`trait UniqueBrand { type Brand; }`).
  Used by non-closure operations (`pure`, `empty`, `join`, `alt`,
  `sequence`, `apply_first`, `apply_second`) that need the
  associated-type projection `<FA as UniqueBrand>::Brand`.
- **`InferableBrand_*`** (new): has Brand and A as trait parameters
  plus an associated `type Marker` (`trait InferableBrand<Brand, A>
{ type Marker; }`). Used by closure-taking operations (`map`,
  `bind`, `traverse`, etc.) where the closure's input type
  disambiguates which brand applies.

_Rationale:_ Non-closure operations like `pure(5)` rely on a unique
brand projection to infer their return type. Eliminating the
unique-brand trait would regress inference for ALL types on these
operations (review finding H1). The name swap gives the more
user-facing trait (appearing in `map`/`bind` signatures and error
messages) the more descriptive name, while the restricted trait gets
a name that emphasizes its key property (uniqueness).

The two traits are independent at the type level (no supertrait
relationship, no blanket impl bridging them). A blanket from
UniqueBrand to InferableBrand fails coherence (same structural
issue as Decision A / POC 3). The `impl_kind!` macro enforces the
invariant that single-brand types get both impls by generating
them together.

Implementation: bulk-rename the existing `InferableBrand` to
`UniqueBrand` via `sed` across `*.rs` files, then introduce the new
`InferableBrand` trait family. POCs 5, 6, 7, 8 validate the new
trait's shape for `map`, `bimap`, `bind`, and `apply`.

### Decision E: Attribute naming

Rename `#[no_inferable_brand]` to **`#[multi_brand]`**.

_Rationale:_ Under Decision D, types with this attribute have
InferableBrand impls (so they ARE inferable via closure direction)
but no UniqueBrand impl (so they lack a unique brand). The new
name describes what is true (multiple brands) rather than what is
no longer accurate. Pre-1.0 stance accepts the breakage.

### Decision F: `explicit::map` shape

Rewrite `explicit::map` to bound on `InferableBrand` with Brand pinned via
turbofish. This unifies dispatch under InferableBrand (rather than maintaining
a separate UniqueBrand-based explicit path) and naturally
contracts the turbofish surface (only Brand is user-specified; the
rest is inferred through InferableBrand).

_Rationale:_ `explicit::map` becomes the universal fallback for cases
inference cannot handle (e.g., `Result<T, T>` diagonal). The
signature shape stays familiar:
`explicit::map::<Brand, _, _, _, _>(f, fa)`. POC 2 validated this
path works for every case including Ref + multi-brand.

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
as a follow-up.

## Integration surface

### Will change

| Component                                                                                                                                      | Change                                                                                                                                           |
| ---------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `fp-library/src/kinds.rs`                                                                                                                      | Rename `InferableBrand_*` to `UniqueBrand_*`; add new `InferableBrand_*` family with associated `Marker`.                                        |
| `fp-library/src/brands.rs`                                                                                                                     | No changes; brand struct definitions stay.                                                                                                       |
| `fp-library/src/types/*/mod.rs`                                                                                                                | Rename `#[no_inferable_brand]` to `#[multi_brand]`. Add `#[multi_brand]` to arity-1 multi-brand brands (already marked; just rename).            |
| `fp-macros/src/hkt/impl_kind.rs`                                                                                                               | Generate direct `InferableBrand_*` impls for every brand (single and multi). Skip when brand's `Of` is a projection (contains `Apply!` or `::`). |
| `fp-macros/src/hkt/trait_kind.rs`                                                                                                              | Generate `InferableBrand_{hash}` alongside `Kind_{hash}` at every arity.                                                                         |
| `fp-library/src/dispatch/functor.rs`                                                                                                           | `map` rebinds from `UniqueBrand` to `InferableBrand`; `Brand` becomes a function type parameter; `Marker` projected from InferableBrand.         |
| `fp-library/src/dispatch/semimonad.rs`                                                                                                         | Same pattern as functor.rs for `bind`, `bind_flipped`, `join`, `compose_kleisli*`.                                                               |
| `fp-library/src/dispatch/bifunctor.rs`                                                                                                         | Same pattern at arity 2 for `bimap`.                                                                                                             |
| `fp-library/src/dispatch/bifoldable.rs`, `bitraversable.rs`, `foldable.rs`, `traversable.rs`, `filterable.rs`, `lift.rs`, `semiapplicative.rs` | Same pattern for each operation's inference wrapper.                                                                                             |
| `fp-library/tests/ui/*.rs`                                                                                                                     | Delete or rewrite `result_no_inferable_brand.rs` and `tuple2_no_inferable_brand.rs`; add positive and negative UI tests for the new behaviour.   |
| `fp-library/docs/brand-inference.md`                                                                                                           | Update to describe InferableBrand; cross-link to `brand-dispatch-traits.md`.                                                                     |
| `fp-library/docs/brand-dispatch-traits.md`                                                                                                     | Update to reflect two-trait-family design (Decision D): UniqueBrand + InferableBrand.                                                            |

### Unchanged

- **Optics subsystem** (`Lens`, `Prism`, `Iso`, `Traversal`, etc.).
- **Benchmarks**: no code changes. Performance validated
  post-implementation.
- **Stack safety / `TailRec`**: unrelated.
- **Serde integration**: unrelated.
- **FunctorDispatch, BindDispatch, BimapDispatch trait shapes**: their
  Val/Ref impls are untouched. Only the inference wrappers that call
  them are rebound.

### Operations that cannot use InferableBrand inference

Operations without any payload that exposes a brand-disambiguating
type cannot drive InferableBrand-based inference. They continue to require
`explicit::` for multi-brand types:

- `Pointed::pure` (return-type inference problem).
- `Alt::alt`, `Plus::empty` (no payload that mentions A).
- `Traversable::sequence` (no closure; the inner Brand is inferred
  from the container shape but the outer Brand may remain ambiguous
  for multi-brand outer types).

`Semiapplicative::apply` has no direct closure but does have an
`Fn(A) -> B` payload inside `ff`. Decision H / POC 8 confirms InferableBrand can
drive inference from that payload; apply is therefore moved into
the inference-supported set in phase 2.

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

### Phase 1: Core InferableBrand and single-operation integration

1. Add `InferableBrand_*` trait family to `fp-library/src/kinds.rs`. Include
   associated type `Marker`. The module-level doc summarises the
   trait trio (`Kind_*`, `UniqueBrand_*`, `InferableBrand_*`).
   `UniqueBrand_*` is the renamed version of today's `InferableBrand_*`.
2. Update `trait_kind!` to emit `InferableBrand_{hash}` at every arity it
   already emits `Kind_{hash}` for.
3. Update `impl_kind!` to emit direct `InferableBrand` impls. Single brands get
   `Marker = Val`. The `&T` blanket for references (generated once
   globally) gives `Marker = Ref`. Projection brands (whose `Of`
   contains `Apply!` or `::`) are skipped.
4. Rename `#[no_inferable_brand]` to `#[multi_brand]` in macro input
   and all use sites. For multi-brand brands, generate one InferableBrand impl
   per brand variant.
5. Rename `InferableBrand_*` to `UniqueBrand_*` via bulk `sed` across
   all `*.rs` files (Decision D). Non-closure dispatch wrappers
   (`pure`, `empty`, `join`, `alt`, `sequence`, `apply_first`,
   `apply_second`) continue to bind on `UniqueBrand`.
6. Rewrite `map` in `fp-library/src/dispatch/functor.rs` to bind on
   `InferableBrand` with Marker projected.
7. Rewrite `explicit::map` to bind on `InferableBrand` with Brand pinned via
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
- `apply`, `ref_apply`, `apply_first`, `apply_second` (Decision H;
  InferableBrand keyed on the Fn payload type inside `ff` + the value type
  inside `fa`).
- `bimap` at arity 2.
- `fold_left`, `fold_right`, `fold_map`.
- `traverse` (outer brand only).
- `filter`.
- `lift2`.

Each is a mechanical analogue of phase 1 for its dispatch trait.
Audit do-notation macros (Decision K).

### Phase 3: Diagnostic polish and docs

1. Attach `#[diagnostic::on_unimplemented]` to `InferableBrand` with wording
   determined from phase 1/2 observations (Decision J).
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
- Clean-build wall-clock time under ~36s (50% regression threshold
  versus 24.0s baseline; see Decision I).
- No regression in existing test suites, benchmarks, or doctests.

## Reference material

- Design analysis: [analysis/multi-brand-evaluation.md](./analysis/multi-brand-evaluation.md).
- POCs: listed in the Validated via POCs section above.
- Brand-dispatch traits overview: [fp-library/docs/brand-dispatch-traits.md](../../../fp-library/docs/brand-dispatch-traits.md).
- Parent brand-inference plan: [../brand-inference/plan.md](../brand-inference/plan.md).
