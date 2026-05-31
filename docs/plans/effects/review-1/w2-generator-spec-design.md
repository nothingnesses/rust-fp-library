# W2 Generator Spec Design: Reader Vertical Slice

This note makes the W2 generator/spec surface concrete enough to begin
implementation without re-opening the already-adopted generator-first
decision.

## Result

Recommendation: implement the first vertical slice as a `Reader` plus
default `Run` slice.

The first generated surface should prove the generator path without
coupling it to every wrapper at once:

- Generate the Reader first-order effect cells and brands through a
  `define_effect!` item generator inside the existing
  `#[document_module]` expansion pipeline.
- Generate the default `Run` Reader smart constructor/helper slice
  through a `define_run_wrapper!` item generator once the effect-cell
  generation is stable.
- Compare generated expansion with the current hand-written Reader /
  `Run` code using `just cargo expand ...`.
- Keep the generated public paths identical to the hand-written paths
  before replacing the hand-written items.

## Approach 1: wrapper generator first

Start by generating one Run wrapper's inherent methods and class impls,
then add effect specs afterward.

Trade-offs:

- Pro: attacks the largest files first.
- Con: wrapper methods depend on effect-cell naming, row functor choice,
  pointer mode, and capability rules, so the wrapper generator would need
  placeholder effect metadata immediately.
- Con: harder to verify with a small diff because wrapper files contain
  first-order handlers, scoped handlers, boundary logic, and helper
  methods together.

Recommendation: reject for the first slice. It is a better second step
after Reader effect metadata exists.

## Approach 2: effect-cell generator first

Start in `reader.rs` by generating the first-order effect cells and their
brands:

- `BoxReader` / `BoxReaderBrand`
- `Reader` / `ReaderBrand`
- `SendReader` / `SendReaderBrand`

Trade-offs:

- Pro: small enough to diff directly against current hand-written code.
- Pro: exercises `#[document_module]` item-generator expansion,
  `impl_kind!`, doc attributes, doctests, `Functor`, and `SendFunctor`
  without crossing into wrapper handler machinery.
- Pro: creates the metadata the wrapper generator needs later.
- Con: does not by itself reduce the six-wrapper duplication; the payoff
  starts when wrapper helpers consume the same effect spec.

Recommendation: adopt as the first code step.

## Approach 3: big-bang Reader across all wrappers

Generate Reader effect cells plus every Reader smart constructor and
runner across `Run`, `RunExplicit`, `RcRun`, `RcRunExplicit`, `ArcRun`,
and `ArcRunExplicit` in one pass.

Trade-offs:

- Pro: demonstrates the full cross-product immediately.
- Con: too large for a reliable first diff, especially with Arc
  `Send + Sync` projection bounds and explicit-lifetime variants.
- Con: makes it harder to tell whether failures come from the macro
  parser, documentation generation, wrapper metadata, or one wrapper's
  bounds.

Recommendation: reject for the first slice. Use it as the migration
target after the smaller Reader/default-`Run` slice is green.

## First Wrapper Choice

Recommended first wrapper: default `Run`.

Alternatives:

- `Run`: primary user-facing wrapper; exercises Box/default closure
  storage and the current `BoxReaderBrand<BoxBrand, E>` row shape.
- `RcRun`: exercises multi-shot `Fn` continuation storage, but it is not
  the default surface and adds clone constraints before the generator
  shape is proven.
- `ArcRun`: exercises the hardest `Send + Sync` surface, but should wait
  until the generator can already emit simpler wrapper bounds.
- Explicit wrappers: useful for lifetime coverage, but they are less
  representative of the default public path and would force lifetime
  machinery into the first generator pass.

Recommendation: choose `Run` first, then add `RcRun`, then `ArcRun`, and
only then the explicit siblings. This order grows the bound matrix
incrementally while keeping each generated diff reviewable.

## Implementation Order

1. Complete. Add a `define_effect!` item generator to
   `fp-macros/src/documentation/item_generators.rs`, parallel to
   `documented_helper_impls!`, so generated items are expanded before
   `#[document_module]` validation.
2. Complete for the first slice. Scope the initial parser to
   `define_effect! { effect Reader; }`, reject unsupported effects and
   trailing fields explicitly, and keep additional effect metadata out
   until the Reader surface has been diffed.
3. Complete for Reader cells. Generate fully documented Rust items with
   `#[document_signature]`,
   `#[document_type_parameters]`, `#[document_parameters]`,
   `#[document_returns]`, and `#[document_examples]` attributes carried
   from the spec or from fixed templates for mechanical impls.
4. Complete for the first slice. Add macro unit tests in `fp-macros`
   that parse the Reader spec and assert the generated token stream
   contains the expected type names, brand impls, and documentation
   attributes.
5. Complete. Replace the hand-written Reader cell families with
   `define_effect! { effect Reader; }` after confirming that
   `just cargo expand -p fp-library --lib types::effects::reader`
   matches the pre-replacement expansion exactly.
6. Complete for default `Run`. Added a method-level
   `define_run_wrapper!` generator for default `Run` Reader `ask`,
   `asks`, and `run_reader`. `named_helpers::reader` matches the
   pre-replacement expansion exactly; `run::smart_constructors` differs
   only by rustfmt's associated-item reordering of the macro invocation,
   which moves generated `ask` before hand-written `get`. Keep rustfmt
   enabled and do not add skip attributes for this order-only artifact.
   The `Run::ask` type-mismatch trybuild snapshot now points the
   bound-location note through `#[document_module]`; accept this for
   generated methods as long as the diagnostic still names the helper
   and trait obligation.
7. Complete for `RcRun`. Added `RcRun` support to the method-level
   `define_run_wrapper!` generator for Reader `ask`, `asks`, and
   `run_reader`. `named_helpers::reader` matches the pre-replacement
   expansion exactly for the RcRun helpers; `rc_run::smart_constructors`
   differs only by the same rustfmt associated-item reordering that moves
   generated `ask` before hand-written `get`. Keep rustfmt enabled and do
   not add skip attributes for this order-only artifact.
8. Complete for `ArcRun`. Added `ArcRun` support to the method-level
   `define_run_wrapper!` generator for Reader `ask`, `asks`, and
   `run_reader`. `arc_run::smart_constructors` differs only by the same
   rustfmt associated-item reordering that moves generated `ask` before
   hand-written `get`; `named_helpers::reader` differs only by rustfmt
   reducing the generated `run_reader` closure body from `{ match ... }`
   to `match ...`. Keep rustfmt enabled and do not add skip attributes for
   these formatting-only artifacts.
9. Next. Extend the Reader helper generator to the explicit wrapper
   siblings, comparing each wrapper slice with `just cargo expand`.
10. Update W2 status after each committed slice so the next session can
    resume from the exact generated surface that is already proven.
