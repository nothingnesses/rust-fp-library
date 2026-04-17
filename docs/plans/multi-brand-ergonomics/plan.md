# Plan: Multi-Brand Ergonomics via InferableBrand Trait

**Status:** DRAFT (design validated; implementation ready)

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

### Decision F: `explicit::map` shape

Rewrite `explicit::map` to bound on `InferableBrand` with Brand pinned via
turbofish. This unifies dispatch under InferableBrand (rather than maintaining
a separate explicit path with different trait plumbing) and naturally
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

### Decision Q: `explicit::` function scope

Decision F applies to ALL 37 explicit functions across all 19
dispatch modules, not just `explicit::map`. Each explicit function
that uses `<FA as InferableBrand>::Brand` must be rewritten to use
the redesigned InferableBrand with Brand as a turbofish parameter.

_Rationale:_ the plan's integration table previously mentioned only
`explicit::map`. All explicit functions share the same signature
pattern and migration is mechanical, but the scope must be explicit
to avoid underestimating the work.

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
5. Rewrite `map` and `explicit::map` in
   `fp-library/src/dispatch/functor.rs` to bind on `Slot` with
   Marker projected. Rewrite all explicit functions in functor.rs
   to use Slot bounds (Decision Q applies to all explicit functions,
   not just `explicit::map`).
6. Update UI tests: remove `result_no_inferable_brand.rs` and
   `tuple2_no_inferable_brand.rs`; add UI tests for closure-directed
   success cases, diagonal failures, unannotated-multi-brand
   failures, and `&&T` compile-fail (Decision T).
7. Add integration tests covering every Val/Ref x single/multi-brand
   cell of the coverage matrix, including the generic fixed-parameter
   case (POC 9).

### Phase 2: Migrate remaining dispatch modules

Migrate each remaining dispatch module from old `InferableBrand` to
`Slot`. For each module, rewrite both the inference wrapper AND all
explicit functions (Decision Q). Run `just verify` after each
migration to confirm the branch compiles and tests pass.

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
- POCs: listed in the Validated via POCs section above.
- Brand-dispatch traits overview: [fp-library/docs/brand-dispatch-traits.md](../../../fp-library/docs/brand-dispatch-traits.md).
- Parent brand-inference plan: [../brand-inference/plan.md](../brand-inference/plan.md).
