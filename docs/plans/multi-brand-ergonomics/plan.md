# Plan: Multi-Brand Ergonomics via Closure-Directed Inference

**Status:** DRAFT

This plan extends the brand-inference system to handle multi-brand
concrete types (`Result`, `Pair`, `Tuple2`, `ControlFlow`, `TryThunk`)
using closure-directed inference.

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

This section consolidates items flagged during plan review that have
not yet been decided. Each entry lists the concern, the approaches
available, and the trade-offs; the appropriate response is for later
discussion.

### Blockers

Items that could invalidate portions of the plan if they do not
behave as assumed. All three share a common mitigation: extend the
POC to cover them before committing to the current design.

1. **Coherence between the blanket impl and direct `Slot` impls.**
   The blanket `impl<FA: InferableBrand_*> Slot_*<FA::Brand, A> for FA`
   is keyed on a where-clause bound; Rust's coherence checker must
   prove that no concrete type could satisfy both the blanket and a
   direct impl. Multi-brand concrete types (like `Result<_, _>`)
   deliberately do not implement `InferableBrand_*`, but coherence
   may or may not accept this argument through the where-clause. The
   POC did not test the blanket + direct combination. Verify this
   also holds for the `&T` blanket on InferableBrand (inherited
   references).

2. **Lifetime-generic GAT behavior.** The production `Slot` for
   `Kind_cdc7cd43dac7585f` has `'a` in the trait and in the GAT
   (`type Out<B: 'a>: 'a`). The POC used `'static` throughout.
   Lifetime-generic GATs have known edge cases around inference and
   normalization that `'static` GATs do not.

3. **Return type computation through `Slot::Out<B>`.** `map`'s return
   type under the new design is
   `<FA as Slot_*<'a, Brand, A>>::Out<B>` where `Brand` is itself
   inferred. Rust sometimes fails to normalize associated types when
   multiple inference steps chain through a single trait projection.
   The existing `map` routes through InferableBrand then Kind, a
   shallower path. Whether the new path normalizes at every real
   call site is unverified. In particular, the blanket impl's
   associated type must produce the same concrete type as direct
   impls so that existing dispatch machinery (for example,
   `Apply!(<Brand as Kind!>::Of<'a, B>)`-style projections)
   continues to compile.

### Open questions

Items to resolve before or during implementation.

4. **Val/Ref dispatch as a second selection axis.**
   `FunctorDispatch`'s `Marker` parameter is resolved via trait
   selection keyed on the closure input type. `Slot`'s `Brand`
   parameter is resolved via trait selection keyed on `(FA, A)`.
   Composing two trait-selection axes in one signature is more
   complex than today's single-axis InferableBrand dispatch. The
   intersection (`map(|x: &i32| ..., &Ok(5))` picking the correct
   Ref + Brand combination) is assumed but not validated. Verify via
   prototype.

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

9. **Closure-annotation fragility.** Today's multi-brand pattern is
   `explicit::map::<Brand, _, _, _, _>(|x| ..., ok)` — the brand
   turbofish pins `A`, so the closure can be unannotated. Under
   Slot-based map, `A` is driven by the closure. Writing
   `map(|x| x + 1, Ok::<i32, String>(5))` without annotating `x`
   would fail. This is a new annotation requirement for multi-brand
   types; the plan states it but does not quantify impact.

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

12. **Testing strategy.** All existing single-brand doctests should
    compile identically. All existing `explicit::map::<...>` doctests
    on multi-brand types should stay as-is (they document the
    explicit path). Add new positive doctests for closure-directed
    resolution and UI tests for the diagonal failure cases. The
    existing POC at
    `fp-library/tests/closure_directed_inference_poc.rs` should be
    promoted to a proper integration test or removed once the real
    implementation subsumes it.

### Design decisions with trade-offs

Decisions that the plan currently answers one way but where
alternatives exist. Listed with options and trade-offs; no choice is
locked in.

**Decision A: Coherence approach.**

- Option A1: Trust Rust's where-clause coherence with blanket +
  direct impls. Simplest if it works; catastrophic failure mode if
  not.
- Option A2: No blanket; generate direct `Slot` impls for every
  brand (single- and multi-). Single-brand types duplicate the
  effect of the blanket. Coherence is trivially safe (each impl is
  keyed on a distinct brand). Cost: more generated code, zero
  runtime impact. InferableBrand becomes purely the
  "unique-brand assertion" trait with no dispatch role.
- Option A3: Sealed marker trait. Introduce a private marker like
  `trait MultiBrand: Sealed` implemented by multi-brand concrete
  types; restructure the blanket around it. Adds complexity for
  limited gain.
- Option A4: Invert the design. Make `Slot` primary; derive
  InferableBrand from uniqueness of Slot resolution. Rewrites more
  of the existing brand machinery. High risk.

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

1. **Extend the POC before committing to implementation.** Add
   cases for the three blockers specifically: blanket + direct-impl
   coherence, full lifetime-parameterized Slot signature, and
   return-type normalization at a realistic call site. The POC is
   small; additional cases are cheap insurance.

2. **Document a coherence fallback.** If the extended POC shows the
   blanket approach (option A1) does not work, switch to option A2
   (generate direct Slot for every brand). Making the fallback
   explicit in the plan avoids a crisis discovered at implementation
   time.

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
