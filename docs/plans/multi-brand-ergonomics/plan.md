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

Implement `Slot_{hash}` only at the Kind arity used by
Functor/Monad/Foldable/Traversable/etc. (arity 1 with lifetime:
`Kind_cdc7cd43dac7585f`). Higher-arity Slot traits are not needed by
any type in the library today:

- Arity 2 is already unambiguous for every existing type
  (`ResultBrand`, `Tuple2Brand`, etc. each have exactly one
  arity-2 brand).
- No arity-3-or-higher types exist.

If future library growth introduces higher-arity types with multiple
partial-application brands at the same level, the Slot pattern
extends mechanically. `trait_kind!` would generate the additional
`Slot_{hash}` trait per new Kind signature, `impl_kind!` would emit
direct impls for multi-brand cases, and the relevant free functions
would bind on the higher-arity Slot. No design change required.

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

### Deferred (not in this plan)

- **Extension to other closure-taking operations** (`bind`, `apply`,
  `lift2`, `traverse`, `fold_left`, `fold_right`, `fold_map`). The
  same Slot pattern generalizes to each, but applying the change to
  every operation is a larger effort. Land `map` first, validate the
  design end-to-end, then extend.
- **Named helpers** (`map_ok`, `map_err`, `map_fst`, etc.). Under
  closure-directed inference these would only fire on diagonal cases,
  which are rare. `explicit::map` covers the same ground. Revisit
  based on real-world usage after phase 1 ships.
- **Primary brand designation** (`#[primary_brand]`). Not needed under
  closure-directed inference; all brands are treated symmetrically.
- **Non-closure operations** (`pure`). The Slot pattern doesn't apply
  to operations without a closure; these remain as-is.

### Out of scope (rejected alternatives)

- **Newtype disambiguation:** conflicts with the library's design
  principles.
- **Type-only priority without closure help:** requires unstable
  features (specialization or negative impls).
- **Primary-brand default with closure-directed fallback:** requires
  specialization to layer the two dispatch paths.

## Open questions

1. **Exact `Slot` trait signature.** Does Slot's `Out<B>` need to
   match the existing `Apply!(<Brand as Kind!>::Of<'a, B>)` exactly?
   The blanket impl from InferableBrand must produce the same
   associated type as direct impls so that existing dispatch machinery
   still compiles. Validate by prototype before committing.
2. **Coherence around the blanket impl.** The blanket
   `impl<FA: InferableBrand> Slot<..., FA::Brand, ...> for FA` must
   not overlap with direct Slot impls on multi-brand types. Since
   multi-brand types don't implement `InferableBrand`, the blanket's
   bound should exclude them. Verify this holds for the `&T` blanket
   on InferableBrand (inherited references).
3. **Val/Ref dispatch composition.** The existing `FunctorDispatch`
   routes by-value and by-ref through a closure-input-type-based
   Marker parameter. Slot introduces another dispatch axis (brand
   selection). Verify the two compose correctly via prototype:
   `map(|x: &i32| *x * 2, &Ok::<i32, String>(5))` should pick the Ok
   brand (because `&i32` aligns with the Ok slot's reference form) and
   route to `RefFunctor::ref_map`.
4. **Diagnostic wording precision.** Does the "user forgot annotation"
   case need a different message from the "diagonal, annotation won't
   help" case? Rust's diagnostic attributes aren't dynamic, so one
   message covering both is the likely outcome.
5. **Apply-side closure-directed inference.** `Semiapplicative::apply`
   has no outer closure but carries an `Fn(A) -> B` payload inside
   `ff`. Could the payload's function type drive Slot dispatch in
   phase 3? Decision can defer to phase 3 but affects whether apply
   becomes CDI-capable or stays explicit-only for multi-brand types.
6. **Testing strategy.** All existing single-brand doctests should
   compile identically. All existing `explicit::map::<...>` doctests
   on multi-brand types should stay as-is (they document the explicit
   path). Add new positive doctests for closure-directed resolution
   and UI tests for the diagonal failure cases. The existing POC at
   `fp-library/tests/closure_directed_inference_poc.rs` should be
   promoted to a proper integration test or removed once the real
   implementation subsumes it.

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

### Phase 4 (contingent): Named helpers

If user feedback shows diagonal cases arise frequently, add
`map_ok` / `map_err` / `map_fst` / `map_snd` / `map_break` /
`map_continue` as thin wrappers around `explicit::map`. Until that
feedback arrives, `explicit::map` suffices.

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
