# Plan: Port purescript-run to fp-library

**Status:** Phase 1 in progress (steps 1 and 2 of 6 complete).

## Current progress

Phase 1 steps 1 and 2 complete.

**Step 1 (`FreeExplicit`).** `FreeExplicit<'a, F, A>` and
`FreeExplicitBrand<F>` are promoted from POC into production at
[fp-library/src/types/free_explicit.rs](../../../fp-library/src/types/free_explicit.rs)
and [fp-library/src/brands.rs](../../../fp-library/src/brands.rs).
The struct wraps its `Pure | Wrap` enum behind an
`Option<FreeExplicitView>` so the custom iterative `Drop` can take
the view via `Option::take` and walk a deep `Wrap` chain in a loop
via `Extract::extract`, mirroring
[`Free`](../../../fp-library/src/types/free.rs)'s strategy. The
struct-level bound is `F: Extract + Functor + 'a` per
[decisions.md](decisions.md) section 4.4. The POC test file imports
the production type and the previously-`#[ignore]`d
`q4_naive_drop_overflows` test is replaced by an actively-running
`q4_drop_deep_does_not_overflow` over a 100 000-deep chain.

**Step 2 (`RcFree`).** `RcFree<F, A>` lands at
[fp-library/src/types/rc_free.rs](../../../fp-library/src/types/rc_free.rs)
following the [`Free`](../../../fp-library/src/types/free.rs)
template. Continuation cells in the
[`CatList`](../../../fp-library/src/types/cat_list.rs) queue are
`Rc<dyn Fn>` (matching what
[`FnBrand<RcBrand>`](../../../fp-library/src/types/fn_brand.rs)
resolves to) instead of `Box<dyn FnOnce>`, so multi-shot effects
like `Choose` can drive the same stored continuation more than
once. The whole substrate lives behind an outer `Rc<Inner>` so
[`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) is
unconditional and O(1) (refcount bump), matching the
[`RcCoyoneda`](../../../fp-library/src/types/rc_coyoneda.rs)
cloning pattern. The inner state's `Drop` impl iteratively
dismantles deep `Suspend` chains via `Extract::extract` (taking
ownership through `Rc::try_unwrap` when uniquely held), and
dropping a 100 000-deep chain is exercised by
`deep_drop_does_not_overflow` in the unit tests.

The full set of inherent methods covered is `pure`, `wrap`,
`lift_f`, `bind`, `map`, `to_view`, `resume`, `evaluate`,
`hoist_free`, plus the new non-consuming
`lower_ref(&self)` / `peel_ref(&self)` (clone-then-consume,
cheap because Clone is O(1)). 12 unit tests cover construction,
chaining, multi-shot via clone, and deep evaluate / Drop.

Remaining Phase 1 work: step 3 (`ArcFree`), step 4 (brand
registrations + by-value and by-reference trait hierarchies for
all three new variants), step 5 (per-variant Criterion benches),
step 6 (per-variant unit and `compile_fail` tests).

Other artefacts unchanged from pre-implementation:

- [poc-effect-row/](../../../poc-effect-row/) — 25 tests across two
  suites validating the row-encoding hybrid (workaround 1 macro
  plus workaround 3 `CoproductSubsetter` fallback), the
  `tstr_crates` Phase 2 refinement, and static-via-Coyoneda
  Functor dispatch end-to-end. See
  [poc-effect-row-canonicalisation.md](poc-effect-row-canonicalisation.md)
  for findings. Migrates into production during Phase 2.

## Open questions, issues and blockers

None. All blockers from the design phase are resolved in
[decisions.md](decisions.md):

- Section 4 (six DECISIONs): row encoding, Functor dictionary,
  stack-safety, four-variant Free family, scoped-effect
  representation (heftia dual row), natural transformations as
  values.
- Section 9 (nine pre-implementation decisions): target audience,
  partial interpretation, async, IO/Effect story, higher-order
  effects, performance, lifetime constraints, macro
  infrastructure, testing strategy.

If a load-bearing question surfaces during implementation, record
it here and pause until it's resolved.

## Deviations

- **Phase 1 step 1: removed `OptionBrand`-using POC tests.** Adding
  the `F: Extract + Functor + 'a` bound to `FreeExplicit` (required
  by the iterative `Drop` impl per
  [decisions.md](decisions.md) section 4.4) means
  `OptionBrand` can no longer back a `FreeExplicit`, since `None`
  has no value to surrender and `OptionBrand` therefore cannot
  lawfully implement `Extract`. The POC's `q5_two_effect_run`
  short-circuit test and the `evaluate_option` helper were dropped;
  the same Run-shaped semantics are reachable via handler
  interpretation in Phase 3+. This is exactly the caveat the
  decision predicts ("this forces every effect functor used with
  `FreeExplicit` to implement `Extract`"), but the plan step text
  said to "replace the local definition with an import" without
  explicitly listing test removals, so it is recorded here.
- **Phase 1 step 1: introduced `FreeExplicitView` enum.** The POC's
  `FreeExplicit` was a two-variant enum directly. The production
  type wraps the variants in `view: Option<FreeExplicitView>` so
  the custom `Drop` impl can move the view out via `Option::take`
  without producing a sentinel `A` value. `FreeExplicitView` is
  `pub` and re-exported alongside `FreeExplicit` to keep the
  variants visible for users who want to pattern-match. No external
  test or bench needed to change shape; the POC tests only used
  `pure`, `wrap`, `bind`, and `evaluate` (no direct match on the
  variants).
- **Phase 1 step 2: `RcFree` uses `Rc<dyn Any>` (not `Box<dyn Any>`)
  for the type-erased value cell.** Decision 4.4's table summarises
  `RcFree`'s erasure as "`Box<dyn Any>` + CatList" while also
  committing to "Cloneable: Yes, O(1)". `Box<dyn Any>` is not
  `Clone`, so the literal table reading conflicts with the Clone
  commitment. The minimal resolution is to swap the Box-erased
  cell for an `Rc<dyn Any>`, which keeps the `dyn Any` erasure
  shape but lets the inner state participate in Clone. Recovering
  an owned `A` from the cell uses `Rc::try_unwrap` and falls back
  to `(*shared).clone()` when the cell is shared, which constrains
  the public methods that perform the final downcast (`to_view`,
  `resume`, `evaluate`, `lower_ref`, `peel_ref`, `hoist_free`) to
  require `A: Clone`. This matches the multi-shot semantics: a
  handler that wants to evaluate the same program more than once
  needs the result type to be reproducible.
- **Phase 1 step 2: `RcFree<F, A>` is `Rc<RcFreeInner<F, A>>`
  (outer `Rc` wrapping).** Step 2's text says "follow the `Free`
  template" without specifying outer-Rc-wrapping, but the unconditional
  O(1) Clone commitment plus the `Suspend` arm holding
  `F::Of<RcFree<F, RcTypeErasedValue>>` produce a recursive Clone
  bound that only resolves cleanly when `RcFree: Clone` is
  unconditional. Outer-Rc-wrapping (the
  [`RcCoyoneda`](../../../fp-library/src/types/rc_coyoneda.rs)
  pattern) makes Clone trivially `Rc::clone(&self.inner)`. State-
  extending operations (`bind`, `map`, `wrap`, `lift_f`, `cast_phantom`)
  use `Rc::try_unwrap` to move out when uniquely owned and clone
  the inner state otherwise.
- **Phase 1 step 2: `RcContinuation` is a newtype, not the bare
  `<RcFnBrand as CloneFn>::Of` projection.** Step 2's text says
  "expressed via `FnBrand<RcBrand>`". Using the macro-mediated GAT
  projection directly as a type alias does not parse (the type
  parameter `F` does not surface through the `Apply!` expansion).
  The production type uses a thin newtype `RcContinuation<F>(Rc<dyn Fn(...)>)`
  with the same in-memory shape as `<RcFnBrand as CloneFn>::Of`,
  and constructs values via `<RcFnBrand as LiftFn>::new(...)` so
  the library's unified function-pointer abstraction is still on
  the construction path. The newtype's `Clone` impl bumps the
  underlying `Rc`'s refcount.

## Implementation protocol

After completing each step within a phase:

1. Run verification: `just fmt`, `just check`, `just clippy`,
   `just deny`, `just doc`, `just test` (or `just verify` which
   runs all six in order).
2. If verification passes, update `Current progress`, `Open
questions, issues and blockers`, and `Deviations` sections at
   the top of this plan to reflect the current state.
3. Commit the step (including the plan updates).

---

Port `purescript-run`'s extensible algebraic effects to
`fp-library`, delivering a Rust `Run<Effects, ScopedEffects, A>`
type that supports row-polymorphic first-order effects and
heftia-style scoped effects, with macro ergonomics for common
cases and a four-variant `Free` substrate covering single-shot,
multi-shot, thread-safe, and non-`'static` payload combinations.

## API stability stance

`fp-library` is pre-1.0. API-breaking changes are acceptable when
they lead to a better end state. This plan prioritises design
correctness and internal coherence over preserving compatibility
with any pre-existing user surface for `Run` (there is none yet;
this is an additive port).

## Motivation

PureScript's `purescript-run` ships an extensible algebraic-effect
system shaped around row polymorphism, partial interpretation, and
multi-shot continuations. fp-library has the building blocks
(`Free<F, A>`, `Coyoneda<F>`, the Brand-and-Kind HKT machinery, and
the `MonadRec` interpreter family) but no public `Run` type. This
plan delivers `Run` and the surrounding effect machinery, ported to
match PureScript's user-facing semantics where stable Rust permits
and explicitly diverging where it doesn't (e.g., `pure` takes a
brand turbofish; multi-shot effects require choosing `RcFree` or
`ArcFree` rather than the default `Free`).

User surface after this plan:

```rust
// Declare a row of effects via the macro:
type AppEffects = effects![Reader<Env>, State<Counter>, Logger];

// Build a program in the do-notation with effect-row inference:
fn run_program() -> Run<AppEffects, NoScoped, String> {
    m_do!(RunBrand {
        cfg <- ask::<Env>();
        n <- get::<Counter>();
        log(format!("config = {cfg:?}, counter = {n}"));
        pure(format!("got {n}"))
    })
}

// Compose handlers as a pipeline that narrows the row at each step:
let result: String = run_program()
    .handle(run_reader(env))
    .handle(run_state(0))
    .handle(run_logger())
    .extract();
```

`runReader: Run<R + READER, S, A> -> Run<R, S, A>`-style row
narrowing matches PureScript Run (the scoped-effect row `S`
threads unchanged through first-order handlers and is narrowed
only by scoped-effect handlers); the macro layer plus
`CoproductSubsetter`-mediated permutation proofs handle the
ordering-mitigation problem (see
[decisions.md](decisions.md) section 4.1).

## Design

The design is recorded in full in
[decisions.md](decisions.md) sections 4 (six core DECISIONs) and
5 (draft architecture). Quick reference:

- **Row encoding (decisions §4.1):** Option 4 hybrid (frunk-style
  Peano-indexed `Coproduct<H, T>` plus `effects![...]` macro
  layer). Workaround 1 (macro lexical sort) is primary; workaround
  3 (`CoproductSubsetter` permutation proof) is fallback for
  hand-written rows.
- **Functor dictionary (decisions §4.2):** static option via
  `Coyoneda` per effect. Each row variant is `Coyoneda<E, A>`,
  which is a `Functor` for any `E` regardless of `E`'s own shape.
  `Coproduct<H, T>` implements `Functor` via recursive trait
  dispatch (`H: Functor + T: Functor`). The dynamic
  `DynFunctor` option is retained as a fallback only.
- **Stack safety (decisions §4.3):** ship both interpreter
  families, mirroring PureScript: `interpret`/`run`/`runAccum`
  (assume target stack-safe) and `interpretRec`/`runRec`/
  `runAccumRec` (require `MonadRec` on target).
- **Free family (decisions §4.4):** four variants — `Free`,
  `RcFree`, `ArcFree`, `FreeExplicit<'a, ...>` — covering the
  cross product of sharing model (Box / Rc / Arc) and existentiality
  (`Box<dyn Any>`-erased vs concrete recursive enum). `RcFreeExplicit`
  / `ArcFreeExplicit` deferred until concrete need surfaces.
- **Scoped effects (decisions §4.5):** heftia-style dual-row
  architecture. `Run` carries a separate higher-order row of
  scoped-effect constructors (`Catch<'a, E>`, `Local<'a, E>`,
  `Bracket<'a, A, B>`, `Span<'a, Tag>`). Day-one `'a` parameter,
  fixed `Run<R, A>` continuation, coproduct-of-constructors
  extension shape. (A `Mask<'a, E>` constructor for duplicated-effect
  masking was considered and deferred to a future revision; see
  [decisions.md](decisions.md) section 4.5's "Deferred to a future
  revision" sub-decision for the four options preserved on the
  shelf.)
- **Natural transformations (decisions §4.6):** `handlers!{...}`
  macro DSL primary, builder pattern (`nt().on::<E>(handler)...`)
  as fallback.

`Run` core type:

```text
Run<Effects, ScopedEffects, A> = FreeFamily<Node<Effects, ScopedEffects>, A>

Node<Effects, ScopedEffects>   = First(VariantF<Effects>)
                               | Scoped(ScopedCoproduct<ScopedEffects>)
```

where `FreeFamily` is one of `Free` / `RcFree` / `ArcFree` /
`FreeExplicit`, and `Run<R, S, A>` has matching aliases (`RcRun`,
`ArcRun`, `RunExplicit`).

## Validated via POCs

| POC                                                                                                       | Findings                                                                                                                                                                                                                                                                                                                                                   |
| --------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [poc-effect-row/tests/feasibility.rs](../../../poc-effect-row/tests/feasibility.rs)                       | 17 tests covering workaround 1 (lexical-sort macro) plus workaround 3 (`CoproductSubsetter` fallback), generic-effect handling, lifetime parameters, 5- and 7-effect rows for trait-inference scaling, plus `tstr_crates` Phase 2 refinement (3 tests showing content-addressed naming + `tstr::cmp` compile-time ordering). All pass on stable Rust 1.94. |
| [poc-effect-row/tests/coyoneda.rs](../../../poc-effect-row/tests/coyoneda.rs)                             | 8 tests validating static-via-Coyoneda end-to-end: `effects_coyo!` macro emits Coyoneda-wrapped Coproducts canonically; `Coyoneda<F, A>` is `Functor` for any `F`; `Coproduct<H, T>` implements `Functor` via recursive trait dispatch with no specialization or runtime dictionary; row canonicalises across input orderings under wrapping.              |
| [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs)                   | 6 tests validating `FreeExplicit<'a, F, A>` integrates with the Brand-and-Kind machinery, supports non-`'static` payloads, supports two-effect Run-shaped composition. One `#[ignore]`d test documents that naive `Drop` overflows on deep chains; the iterative custom `Drop` ships in Phase 1.                                                           |
| [fp-library/benches/benchmarks/free_explicit.rs](../../../fp-library/benches/benchmarks/free_explicit.rs) | Criterion bench at depths 10 / 100 / 1000 / 10000 confirming `FreeExplicit`'s per-node cost is approximately 27ns in the linear regime. The Phase-1 baseline for measuring `RcFree` / `ArcFree` regressions.                                                                                                                                               |

The POC code (the `effects!` / `effects_coyo!` macros, the stub
Coyoneda) migrates into production during Phase 2 and Phase 3; the
POC repos remain as reference until then and are deleted once the
production tests cover the same surface.

## Key decisions

The full decision rationale is in [decisions.md](decisions.md).
Quick reference table:

| ID        | Decision                                                                                     | Rationale (one-line)                                                                        |
| --------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| 4.1       | Option 4 hybrid (macro + nested Coproduct) with corophage-style `'a` per effect              | Most production-credible reference (corophage) and best stable-Rust ergonomics              |
| 4.1       | Workaround 1 (macro canonicalisation) primary; workaround 3 (`CoproductSubsetter`) fallback  | Macro pays the sort cost once at row construction; Subsetter handles hand-written rows      |
| 4.1       | tstr_crates content-addressed naming as Phase 2 refinement                                   | Stable type-level identity across import paths; the only credible stable-Rust improvement   |
| 4.2       | Static option via `Coyoneda` per effect                                                      | Each row variant is trivially a Functor; section 5.2 commits to Coyoneda anyway             |
| 4.3       | Ship both `interpret` and `interpretRec` families                                            | Documentation parity with PureScript Run; few-percent runtime cost is small                 |
| 4.4       | Four-variant Free: `Free`, `RcFree`, `ArcFree`, `FreeExplicit`                               | Mirrors the four-variant Coyoneda family already shipping; covers all useful combinations   |
| 4.4       | `RcFreeExplicit` / `ArcFreeExplicit` deferred                                                | No concrete user request yet; intersections are non-breaking additions when needed          |
| 4.5       | Heftia dual-row for scoped effects                                                           | Cleanest higher-order effect encoding surveyed; preserves first-class programs              |
| 4.5       | `'a` lifetime parameter on every scoped-effect constructor from day one                      | Avoids breaking-change retrofit when `FreeExplicit` use cases want non-`'static` actions    |
| 4.5       | Fixed `Run<R, A>` interpreter continuation (no associated type)                              | Matches every Haskell library surveyed; associated type deferred until use case forces it   |
| 4.5       | Coproduct-of-constructors for user-defined scoped effects                                    | Mirrors the first-order row's structure; preserves first-class-programs property            |
| 4.6       | `handlers!{...}` macro DSL primary; builder pattern fallback                                 | Same shape as section 4.1's macro + mechanical-fallback hybrid                              |
| 9.3 / 9.4 | Sync interpreters in v1; async (and async IO) via `Future` as a `MonadRec` target in Phase 3 | "User picks the target monad" — single mechanism, no parallel `AsyncRun` family             |
| 9.8       | All effects-related macros live in `fp-macros`; split off a separate crate only if needed    | One crate, one release cadence, one place to coordinate macro semantics                     |
| 9.9       | TalkF + DinnerF integration test from `purescript-run` as the headline Phase 4 milestone     | Real-world reference; validates the port behaves like `purescript-run` for a worked example |

## Integration surface

### Will change

| Component                                 | Change                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ----------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fp-library/src/types/free.rs`            | Existing `Free<F, A>` keeps its current shape; minor adjustments if integration with `Run` requires.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `fp-library/src/types/free_explicit.rs`   | **New module.** Promote `FreeExplicit<'a, F, A>` from POC, add iterative custom `Drop`, add `Functor` / `Pointed` / `Semimonad` / `Monad` impls.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `fp-library/src/types/rc_free.rs`         | **New module.** `RcFree<F, A>` following the `Free` template with `FnBrand<RcBrand>`-shaped continuations (i.e., `Rc<dyn 'a + Fn(B) -> RcFree<F, A>>` via the unified [`FnBrand`](../../../fp-library/src/types/fn_brand.rs) abstraction). Multi-shot effects (`Choose`, `Amb`).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `fp-library/src/types/arc_free.rs`        | **New module.** `ArcFree<F, A>` following the `ArcCoyoneda` template with `FnBrand<ArcBrand>`-shaped continuations (i.e., `Arc<dyn 'a + Fn(B) -> ArcFree<F, A> + Send + Sync>` via [`FnBrand`](../../../fp-library/src/types/fn_brand.rs) parameterised by [`ArcBrand`](../../../fp-library/src/brands.rs#L43)) and the `Send`/`Sync` Kind-trait pattern via `SendRefCountedPointer`.                                                                                                                                                                                                                                                                                                                                                                                                           |
| `fp-library/src/types/run.rs`             | **New module.** `Run<Effects, ScopedEffects, A>` plus `RcRun`, `ArcRun`, `RunExplicit` aliases. `Node<...>` enum dispatching first-order vs scoped.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `fp-library/src/types/run/coproduct.rs`   | **New submodule.** Brand-aware adapter layer over `frunk_core::coproduct::{Coproduct, CNil, CoproductSubsetter}`: newtype wrappers, `impl` blocks bridging `frunk_core`'s Plucker / Sculptor / Embedder traits to the project's `Brand` system. Direct (non-newtyped) `Functor` impls on `frunk_core::Coproduct<H, T>` live here too (own-trait + foreign-type, orphan-permitted).                                                                                                                                                                                                                                                                                                                                                                                                              |
| `fp-library/src/types/run/variant_f.rs`   | **New submodule.** `VariantF<Effects>` first-order coproduct with Coyoneda-wrapped variants and recursive `Functor` impl on `Coproduct<H, T>` (delegating to the adapter in `coproduct.rs`).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `fp-library/src/types/run/scoped.rs`      | **New submodule.** `ScopedCoproduct<ScopedEffects>` higher-order coproduct, standard scoped constructors. `Catch<'a, E>` and `Span<'a, Tag>` ship Val-only. `Local` ships in Val and Ref flavours (`Local<'a, E>` + `RefLocal<'a, E>`); `Bracket` ships in Val and Ref flavours (`Bracket<'a, A, B>` + `RefBracket<'a, P, A, B>`) per [decisions.md](decisions.md) section 4.5 sub-decisions. `Mask` is deferred to a future revision per the same section.                                                                                                                                                                                                                                                                                                                                     |
| `fp-library/src/dispatch/run/`            | **New submodule.** Closure-driven Val/Ref dispatch for `bracket` and `local` smart constructors, mirroring the existing layout described in [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md). Files: `bracket.rs` (`BracketDispatch` trait + `Val` impl + `Ref<P>` impls per pointer brand + `bracket` inference wrapper + `explicit::bracket` brand-explicit wrapper); `local.rs` (`LocalDispatch` trait + `Val` and `Ref` impls + `local` inference wrapper + `explicit::local` wrapper). Re-exported from `fp-library/src/functions.rs` alongside `map`, `bind`, etc.                                                                                                                                                                                                   |
| `fp-library/src/types/run/handler.rs`     | **New submodule.** Handler-pipeline machinery (`Run::handle`), natural-transformation type, `peel` / `send` / `extract`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `fp-library/src/types/run/interpreter.rs` | **New submodule.** `interpret` / `run` / `runAccum` (recursive) and `interpretRec` / `runRec` / `runAccumRec` (`MonadRec`-targeted) families.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `fp-macros/src/effects/`                  | **New module tree.** `effects!`, `effects_coyo!`, `handlers!`, `define_effect!`, `define_scoped_effect!` proc-macros. Migration from POC.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `fp-library/src/brands.rs`                | Add brands for new types: `RcFreeBrand`, `ArcFreeBrand`, `FreeExplicitBrand<F>`, `RunBrand<R, S>`, `RcRunBrand<R, S>`, `ArcRunBrand<R, S>`, `RunExplicitBrand<R, S>`. `FreeExplicitBrand<F>` is a single-parameter `PhantomData<F>` struct mirroring [`CoyonedaBrand<F>`](../../../fp-library/src/brands.rs#L155); the four `*RunBrand<R, S>` variants are two-parameter `PhantomData<(R, S)>` structs mirroring [`CoyonedaExplicitBrand<F, B>`](../../../fp-library/src/brands.rs#L171). For all of these, `'static` bounds live on impls (so the row types `R`, `S` and the payload `'a`, `A` stay out of the brand identity and appear only in `Of<'a, A>` at instantiation, keeping brand types `'static`-clean while admitting non-`'static` payloads via `RunExplicit` / `FreeExplicit`). |
| `fp-library/tests/run_*.rs`               | **New test files.** Per-Free-variant unit tests (Phase 1), row-canonicalisation regression tests migrated from `poc-effect-row/` (Phase 2), TalkF + DinnerF integration test (Phase 4).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `fp-library/benches/benchmarks/run_*.rs`  | **New bench files.** Per-Free-variant Criterion benches (bind-deep, bind-wide, peel-and-handle), row-canonicalisation benches (macro vs Subsetter), handler-composition benches.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |

### Unchanged

- **`Free<F, A>` core** (`fp-library/src/types/free.rs`): the existing
  `Box<dyn FnOnce>`-based variant stays as-is. New variants are
  added beside it.
- **Coyoneda family** (`Coyoneda`, `RcCoyoneda`, `ArcCoyoneda`,
  `CoyonedaExplicit`): used by `Run`'s first-order row but not
  modified.
- **Brand-and-Kind machinery** (`fp-macros` HKT macros, `brands.rs`,
  `kinds.rs`): used by `Run` but not modified beyond the new brand
  registrations above.
- **Optics subsystem** (`Lens`, `Prism`, `Iso`, `Traversal`,
  etc.): unrelated to `Run`.
- **Existing `MonadRec` impls** (`Option`, `Result`, `Thunk`,
  etc.): used as interpretation targets but not modified.
- **Pre-existing dispatch traits and `m_do!` / `a_do!`**: continue
  to work for `Run` once the appropriate `Functor` / `Monad` impls
  on `RunBrand` ship.

## Out of scope

Permanently excluded from this plan. Revisit only if design
constraints change.

- **Multi-prompt delimited continuations** (Koka / MpEff style).
  Ruled out by [decisions.md](decisions.md) section 1.2; no Rust
  equivalent of GHC's `prompt#` / `control0#`.
- **Tag-based type-level sorting** (workaround 2 from
  decisions §4.1). Surveyed in
  [docs/plans/type-level-sorting/research/](../type-level-sorting/research/);
  the credible building blocks exist (`tstr_crates`) but the full
  sort engine on stable Rust requires the user to write it. The
  workaround-1 macro plus workaround-3 Subsetter hybrid is
  sufficient.
- **Evidence-passing dispatch** (EvEff style). Surveyed in
  [research/deep-dive-evidence-passing.md](research/deep-dive-evidence-passing.md);
  collapses to Option 1 (Peano) or Option 3 (TypeId) once removed
  from Haskell's closed-type-family setting.
- **Coroutine substrate without Free.** Surveyed in
  [research/deep-dive-coroutine-vs-free.md](research/deep-dive-coroutine-vs-free.md);
  loses 4 of 5 first-class-program properties section 4.4
  requires.
- **`mtl`-style trait-bound effect set** (Option 5 from
  decisions §4.1). Loses first-class programs.
- **Custom `Effect`-monad analogue.** Section 9.4 commits to
  `Thunk` (v1) and `Future` (Phase 3) as `MonadRec` targets;
  inventing a Rust-specific `Effect` monad is unnecessary.
- **`async fn`-shaped interpreters.** Section 9.3 commits to
  sync interpreters with async-via-target-monad. No parallel
  `AsyncRun` family.

## Implementation phasing

All five phases ship together as one feature release.

### Phase 1: Complete the Free family

Land the three missing Free variants. Phases 2-5 treat the choice
of variant as a user-level parameter, so completing the substrate
first prevents later refactor.

1. Promote `FreeExplicit<'a, F, A>` from POC to
   `fp-library/src/types/free_explicit.rs`. Add iterative custom
   `Drop` per [decisions.md](decisions.md) section 4.4 ("What to
   do about `Drop`"). Delete the POC's local copy in
   `fp-library/tests/free_explicit_poc.rs` and replace with a
   `use fp_library::types::FreeExplicit;` import. Move the bench at
   `fp-library/benches/benchmarks/free_explicit.rs` to use the
   imported type. Un-`#[ignore]` the deep-`Drop` test once the
   iterative `Drop` ships.
2. Implement `RcFree<F, A>` at `fp-library/src/types/rc_free.rs`
   following the `Free` template, with continuations expressed via
   [`FnBrand<RcBrand>`](../../../fp-library/src/types/fn_brand.rs)
   (yielding `Rc<dyn 'a + Fn(B) -> RcFree<F, A>>` after `Kind`
   resolution) and the `RcCoyoneda` cloning pattern. Add
   `lower_ref(&self)` / `peel_ref(&self)` for non-consuming
   reinterpretation. The `FnBrand`-based shape is preferred over a
   raw `Rc<dyn Fn>` field so the new module participates in the
   library's unified function-pointer abstraction from day one
   (see [`fn_brand.rs`](../../../fp-library/src/types/fn_brand.rs)).
3. Implement `ArcFree<F, A>` at `fp-library/src/types/arc_free.rs`
   following the `ArcCoyoneda` template, with continuations
   expressed via
   [`FnBrand<ArcBrand>`](../../../fp-library/src/types/fn_brand.rs)
   parameterised by
   [`ArcBrand`](../../../fp-library/src/brands.rs#L43) (yielding
   `Arc<dyn 'a + Fn(B) -> ArcFree<F, A> + Send + Sync>` after
   `Kind` resolution via `SendRefCountedPointer`) and the
   `Kind<Of<'a, A>: Send + Sync>` associated-type-bound trick.
4. Add brands and the by-value hierarchy
   (`Functor` / `Pointed` / `Semimonad` / `Monad`) plus the
   by-reference hierarchy
   (`RefFunctor` / `RefSemimonad` / `RefMonad`, etc., per
   [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md))
   for `RcFreeBrand`, `ArcFreeBrand`, `FreeExplicitBrand<F>`. The
   default `Free`'s impls are the template. Both hierarchies are
   required so that `dispatch::map` / `dispatch::bind` route
   correctly over each brand once `Run` and the scoped-effect
   smart constructors land in Phase 2 / Phase 4.
5. Per-variant Criterion benches (bind-deep at depths 10 / 100 /
   1000 / 10000, bind-wide, peel-and-handle). Match the
   `FreeExplicit` POC bench shape.
6. Per-variant unit tests covering construction, evaluation,
   `fold_free` interpretation, and the property each variant
   promises (single-shot, multi-shot, thread-safe,
   non-`'static`). Plus `compile_fail` UI tests for the negative
   cases (e.g., trying to multi-shot a `Free`).

### Phase 2: Run substrate and first-order effects

1. Add `frunk_core` as a direct dependency of `fp-library`
   (license check via `just deny`, MSRV verification, and
   workspace `Cargo.toml` registration). Introduce a thin
   Brand-aware adapter layer at `fp-library/src/types/run/coproduct.rs`:
   newtype wrappers around `frunk_core::coproduct::{Coproduct, CNil}`
   plus `impl` blocks bridging `frunk_core`'s Plucker / Sculptor /
   Embedder traits to the project's `Brand` system. Direct `impl`s
   of fp-library's own `Functor` for `frunk_core::Coproduct<H, T>`
   are permitted by the orphan rules; `Brand`-style impls require
   the newtype wrapper. See Implementation note 1 below.
2. `VariantF<Effects>` at `fp-library/src/types/run/variant_f.rs`:
   Coyoneda-wrapped Coproduct row with recursive `Functor` impl
   on `Coproduct<H, T>` (where `H: Functor + T: Functor`) and base
   case on `CNil`. Migrate the trait-shape from
   [poc-effect-row/src/lib.rs](../../../poc-effect-row/src/lib.rs)
   under the production `Functor` trait.
3. `Member<E, Indices>` trait for first-order injection /
   projection, layered on top of `frunk_core::CoproductSubsetter`
   via the adapter from step 1.
4. `Run<Effects, ScopedEffects, A>` core type at
   `fp-library/src/types/run.rs` with `RcRun`, `ArcRun`,
   `RunExplicit` aliases. `Node<Effects, ScopedEffects>` enum.
   Add brands and the by-value hierarchy
   (`Functor` / `Pointed` / `Semimonad` / `Monad`) plus the
   by-reference hierarchy
   (`RefFunctor` / `RefSemimonad` / `RefMonad`, etc.) for
   `RunBrand`, `RcRunBrand`, `ArcRunBrand`, `RunExplicitBrand`,
   delegating to the underlying Free variant's existing impls
   from Phase 1 step 4. Both hierarchies are required so
   `m_do!` / `a_do!` and `dispatch::map` / `dispatch::bind`
   work over `Run` regardless of whether the user's closures
   consume or borrow.
5. `Run::pure`, `Run::peel`, `Run::send` core operations,
   delegating to the underlying Free variant.
6. `effects!` macro in `fp-macros/src/effects/effects.rs`,
   migrated from
   [poc-effect-row/macros/src/lib.rs](../../../poc-effect-row/macros/src/lib.rs).
   Lexical-sort by `quote!{}.to_string()`; emit Coyoneda-wrapped
   variants. The un-wrapped Coproduct form lives at
   `crate::__internal::raw_effects!` for fp-library-internal use
   (test fixtures, lower-level combinators) and is not part of
   the public surface; see [decisions.md](decisions.md) section 4.6.
   Factor the lexical-sort logic into a shared `proc-macro2`
   helper used by both `effects!` and `scoped_effects!` (Phase 4
   step 4) so sort-correctness fixes land in one place.
7. Coyoneda-wrapping smart constructors (`lift_f` analogues for
   each effect type).
8. Migrate the 25 row-canonicalisation tests from
   `poc-effect-row/tests/` into
   `fp-library/tests/run_row_canonicalisation.rs` as the
   regression baseline. Verify all pass under the production
   types. Delete the POC repository once the migration lands.

### Phase 3: First-order effect handlers, interpreters, natural transformations

1. `handlers!{...}` macro in
   `fp-macros/src/effects/handlers.rs` producing tuple-of-closures
   keyed on the row's type-level structure. Builder fallback
   (`nt().on::<E>(handler)...`) as the non-macro path
   ([decisions.md](decisions.md) section 4.6).
2. `interpret` / `run` / `runAccum` recursive-target interpreter
   family in `fp-library/src/types/run/interpreter.rs`.
3. `interpretRec` / `runRec` / `runAccumRec` `MonadRec`-target
   interpreter family in the same module.
4. Standard first-order effect types and their smart
   constructors: `State<S>`, `Reader<E>`, `Except<E>`, `Writer<W>`,
   `Choose` (multi-shot, `RcRun`-only).
5. `define_effect!` macro at
   `fp-macros/src/effects/define_effect.rs` generating effect
   enum + smart constructors + label / brand registration.
6. `compile_fail` UI tests for negative cases (handler missing an
   effect, wrong type ascription, multi-shot via single-shot
   `Run`).

### Phase 4: Scoped effects (heftia dual row)

1. `ScopedCoproduct<ScopedEffects>` at
   `fp-library/src/types/run/scoped.rs` with the dual-row
   integration into `Run<Effects, ScopedEffects, A>`.
2. Standard scoped-effect constructors. Per
   [decisions.md](decisions.md) section 4.5 sub-decisions, `Bracket`
   and `Local` ship in two parallel flavours each (Val and Ref) that
   mirror the library's existing Val/Ref dispatch pattern at
   [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md);
   `Catch` and `Span` ship Val-only (Ref flavours rejected per the
   sub-decision).
   - `Catch<'a, E>` for `Error.catch`, with `action: Run<R, S, A>`,
     `handler: Box<dyn FnOnce(E) -> Run<R, S, A>>`. Val only.
   - `Local<'a, E>` (Val flavour) for `Reader.local` with a
     consuming modify, holding `modify: Box<dyn FnOnce(E) -> E>`,
     `action: Run<R, S, A>`.
   - `RefLocal<'a, E>` (Ref flavour) for `Reader.local` with a
     borrowing modify, holding `modify: Box<dyn FnOnce(&E) -> E>`,
     `action: Run<R, S, A>`. Removes the `E: Clone` requirement
     that the Val flavour imposes when users want to derive a
     sub-scope env from the parent without owning it.
   - `Bracket<'a, A, B>` (Val flavour) for `Run` / `RunExplicit`
     users, with `acquire: Run<R, S, A>`,
     `body: Box<dyn FnOnce(A) -> Run<R, S, (A, B)>>`,
     `release: Box<dyn FnOnce(A) -> Run<R, S, ()>>`. The body
     consumes `A`, threads it back to the interpreter via
     `(A, B)`, and the interpreter moves the returned `A` into
     `release`.
   - `RefBracket<'a, P, A, B>` (Ref flavour) for `RcRun` /
     `ArcRun` users, parameterised by
     [`P: RefCountedPointer`](../../../fp-library/src/classes/ref_counted_pointer.rs)
     ([`RcBrand`](../../../fp-library/src/brands.rs#L250) for
     `RcRun`, [`ArcBrand`](../../../fp-library/src/brands.rs#L43)
     for `ArcRun`), with `acquire: Run<R, S, A>`,
     `body: Box<dyn FnOnce(P::Of<A>) -> Run<R, S, B>>`,
     `release: Box<dyn FnOnce(P::Of<A>) -> Run<R, S, ()>>`. Body
     and release both receive a pointer clone; the resource lives
     until the last clone drops, mirroring PureScript's
     GC-aliased `bracket` semantics
     ([`Aff.purs:308`](https://github.com/purescript-contrib/purescript-aff/blob/master/src/Effect/Aff.purs#L308)).
   - `Span<'a, Tag>`, with `tag: Tag`, `action: Run<R, S, A>`.
     Val only (no closure to dispatch over).
3. Scoped-effect interpreter trait. Method per constructor;
   fixed `Run<R, A>` continuation
   ([decisions.md](decisions.md) section 4.5).
4. `scoped_effects!` macro and `define_scoped_effect!` macro,
   sharing the lexical-sort helper with Phase 2's `effects!` (one
   helper, two thin entry-point macros, distinct output shapes:
   Coyoneda-wrapped Coproduct vs `ScopedCoproduct`).
5. Smart constructors: `catch`, `span` (single-flavour
   wrappers); `bracket` and `local` (closure-driven dispatch over
   Val and Ref flavours, reusing the existing `Val` / `Ref`
   markers and dispatch machinery from
   [`fp-library/src/dispatch/`](../../../fp-library/src/dispatch/);
   `bracket`'s Ref impl additionally carries the pointer brand
   `P` so `Ref<RcBrand>` and `Ref<ArcBrand>` resolve to distinct
   `RefBracket` node types). Concretely:
   - `BracketDispatch<R, S, A, B, Marker>` trait with `Val` impl
     (closures of shape `FnOnce(A) -> Run<R, S, (A, B)>` plus
     `FnOnce(A) -> Run<R, S, ()>`) and `Ref<P>` impls for each
     `P: RefCountedPointer` (closures of shape
     `FnOnce(P::Of<A>) -> Run<R, S, B>` plus
     `FnOnce(P::Of<A>) -> Run<R, S, ()>`).
   - `LocalDispatch<R, S, E, A, Marker>` trait with `Val` impl
     (`FnOnce(E) -> E`) and `Ref` impl (`FnOnce(&E) -> E`).
   - The dispatch traits and their impls live at
     `fp-library/src/dispatch/run/bracket.rs` and
     `fp-library/src/dispatch/run/local.rs`, mirroring the
     existing layout described in
     [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md).
     No `mask` smart constructor in v1; the `Mask` constructor is
     deferred per [decisions.md](decisions.md) section 4.5
     sub-decisions.
6. Standard handlers (`run_reader`'s `local` clause,
   `run_except`'s `catch` clause, etc.) wired through the dual
   row.
7. Tests: scoped-effect unit tests covering each of the four
   standard constructors (`Catch`, `Local`, `Bracket`, `Span`)
   plus `compile_fail` cases. Reformulate relevant Phase 3 tests
   to use scoped operations where appropriate.

### Phase 5: Integration test, deferred items as needed

1. Port the canonical TalkF + DinnerF example from
   [`purescript-run/test/Examples.purs`](https://github.com/natefaubion/purescript-run/blob/master/test/Examples.purs#L13-L106)
   into
   `fp-library/tests/run_talkf_dinnerf_integration.rs`.
   Multi-effect program demonstrating Reader, State, Talk, and
   Dinner effects composed and handled in turn. Faithful port
   from PureScript's source.
2. Add row-canonicalisation Criterion benches (macro path vs
   `CoproductSubsetter` permutation-proof fallback path) and
   handler-composition benches per
   [decisions.md](decisions.md) section 9 item 6.
3. (Phase 3 deferred items, scheduled here so they're not lost):
   - Optional `tstr_crates` content-addressed-naming refinement
     for the macro layer
     ([decisions.md](decisions.md) section 4.1's Phase 2 note).
     Add only if real-world usage shows import-path-sensitive
     sorting causes confusion.
   - Compile-time index-table refinement (Koka-inspired). Add
     only if a benchmark shows Coproduct pattern-match dispatch
     is a measurable bottleneck.
4. Write `fp-library/docs/run.md` documenting the effects
   subsystem for users. Cross-link to
   [decisions.md](decisions.md) for design rationale.

### Phase 6+ (deferred, not in this plan)

These items arrive when concrete need surfaces. Each one names
the artifact, what it would deliver, why it is deferred, and a
revisit trigger; entries are ordered roughly from substrate
outward to user surface.

- **`RcFreeExplicit` / `ArcFreeExplicit` intersection variants.**
  Add the two missing entries in the cross-product of sharing
  model and existentiality so that (multi-shot or
  thread-crossing) Free programs can also carry non-`'static`
  payloads ([decisions.md](decisions.md) section 4.4). _What
  this is for:_ the rare program that needs both a refcounted
  Free substrate and borrowed effect data. _Why deferred:_ no
  concrete user request yet, and intersections are non-breaking
  additions when needed. _Trigger:_ a user program that wants
  borrowed effect payloads in a multi-shot or thread-crossing
  setting.
- **`Send`-constrained trait hierarchy and cargo feature gating
  for `RcFree` / `ArcFree`.** Either a parallel
  `Send`-constrained `Functor` / `Monad` hierarchy if the
  existing `Send`-families don't cover the `ArcFree` use case,
  or cargo feature gates that let downstream crates opt out of
  compiling `RcFree` / `ArcFree` if their compile cost becomes
  uncomfortable
  ([decisions.md](decisions.md) section 4.4). _Why deferred:_
  the current shape's adequacy is unverified; the answer
  depends on benchmarks and downstream feedback. _Trigger:_
  benchmark or compile-time evidence that the existing shape is
  insufficient.
- **`State::modify` Val/Ref split.** Add a `RefState<S>`
  first-order effect alongside `State<S>` whose `modify`
  operation takes `FnOnce(&S) -> S` instead of `FnOnce(S) -> S`,
  with a unified `modify(...)` smart constructor dispatching
  over closure shape via the same `Val` / `Ref` markers used by
  `Local` / `RefLocal` in Phase 4. _What this is for:_ users who
  want to derive a new state from the old without owning it,
  avoiding an `S: Clone` requirement. _Why deferred:_ Phase 3's
  standard first-order effect set ships Val-only to keep the
  surface small; the Run-brand by-ref hierarchy from Phase 2
  step 4 already supplies the trait routing this would build
  on. _Trigger:_ first user who hits the `S: Clone` wall on the
  Val flavour, or the first integration test that benefits from
  `&S` in `modify`.
- **`Writer::censor` Val/Ref split.** Add `censor` to the
  standard `Writer<W>` set (currently only `tell` ships in
  Phase 3 step 4), then ship a `RefWriter<W>` extension whose
  `censor` takes `FnOnce(&W) -> W` instead of
  `FnOnce(W) -> W`. _What this is for:_ deriving a transformed
  log without consuming the parent, the writer-log analogue of
  `State::modify`'s ergonomic story. _Why deferred:_ `censor`
  itself is not in v1's standard Writer set, and the Val/Ref
  split is a follow-up to adding it. _Trigger:_ a real
  log-censoring use case, plus the same `W: Clone` ergonomic
  wall.
- **`handlers!{...}` macro Val/Ref variants.** Extend the
  Phase 3 macro so each per-effect handler entry can be emitted
  as Val or Ref based on the user's closure type, reusing the
  same `Val` / `Ref` markers as the rest of the dispatch
  system. Each entry is conceptually a closure of shape
  `FnOnce(E::Operation) -> Run<R', S, A>`; the Ref variant
  takes `FnOnce(&E::Operation) -> Run<R', S, A>`. _What this is
  for:_ handlers that inspect operations without consuming them
  (e.g., a logging handler that records and then forwards),
  avoiding a `Clone` bound on operation payloads. _Why
  deferred:_ the macro is already non-trivial in v1; shipping it
  Val-only first and extending it once a real handler benefits
  is a smaller initial bite, and the extension is non-breaking.
  _Trigger:_ first handler (in the standard library or
  downstream) that needs inspect-without-consuming on an
  operation payload.
- **`generalBracket` and `BracketConditions`.** Port the
  more general bracket from PureScript Aff at
  [`Aff.purs:364-373`](https://github.com/purescript-contrib/purescript-aff/blob/master/src/Effect/Aff.purs#L364-L373):
  `generalBracket` accepts a `BracketConditions` record with
  separate `killed`, `failed`, and `completed` handlers, each
  receiving the resource. _What this is for:_ observing how a
  bracketed action terminated (success, failure, cancellation)
  and running different cleanup per outcome, instead of v1's
  single uniform `release`. _Why deferred:_ v1's interpreter is
  sync and has no cancellation event, so `killed` has no
  semantics until the async target monad lands; without async,
  `generalBracket` collapses to a more verbose `bracket` with
  two unused branches. The Ref-flavour `RefBracket<'a, P, A, B>`
  already shows that multiple closures can each receive
  `P::Of<A>` without contention, so the dispatch design extends
  cleanly when the time comes. _Trigger:_ the async target
  monad lands (next item), at which point cancellation becomes
  a real event handlers want to observe.
- **`MonadRec` impl for `Future` as an async target monad**
  ([decisions.md](decisions.md) section 9 items 3 + 4). _What
  this is for:_ asynchronous interpretation of `Run` programs
  via the same target-monad mechanism that already lets users
  pick `Identity` or `Thunk` for sync interpretation; no
  parallel `AsyncRun` family. _Why deferred:_ v1 ships sync
  interpreters and async users wrap calls in `spawn_blocking`
  or similar; adding `Future` as a `MonadRec` target requires
  designing around pinned futures, executor coupling, and
  multi-shot continuation friction, which is a separate body of
  work. _Trigger:_ first user request for async interpretation
  that cannot be satisfied by `spawn_blocking` around a sync
  interpreter call.
- **Split `fp-macros` into `fp-effects-macros`**
  ([decisions.md](decisions.md) section 9 item 8). _What this
  is for:_ a separate crate housing the effects-related
  proc-macros (`effects!`, `effects_coyo!`, `handlers!`,
  `define_effect!`, `define_scoped_effect!`,
  `scoped_effects!`) so their release cadence is independent of
  the HKT-system macros and do-notation macros that share
  `fp-macros` today. _Why deferred:_ v1 keeps everything in one
  crate to avoid multiplying release coordination and adding a
  parallel macro-resolution path. _Trigger:_ `fp-macros`
  compile time grows uncomfortably, or effects-related macro
  changes start blocking unrelated macro releases.

## Implementation notes

1. **`Coproduct` choice (Phase 2).** The POC depends on
   `frunk_core::coproduct::{Coproduct, CNil, CoproductSubsetter}`,
   and the production port adopts the same dependency. Phase 2
   step 1 adds `frunk_core` to `fp-library`'s `Cargo.toml`,
   confirms the license is permitted by `just deny`, and
   introduces a thin Brand-aware adapter layer at
   `fp-library/src/types/run/coproduct.rs` (newtypes plus `impl`
   blocks that bridge `frunk_core`'s Plucker / Sculptor / Embedder
   traits to the project's `Brand` system). Implementing
   fp-library's own `Functor` for `frunk_core::Coproduct<H, T>`
   directly is permitted by the orphan rules (own-trait +
   foreign-type) and is the preferred shape for the recursive
   `Functor` dispatch; `Brand`-style impls on the foreign type
   require the newtype wrapper. If the adapter ever exceeds
   approximately 200 lines, that signals real impedance mismatch
   and a fork to an in-house reimplementation should be
   considered, but the default is to stay on `frunk_core`.
2. **POC-to-production migration (Phases 2 + 3).** The POC at
   `poc-effect-row/` is a separate Cargo workspace and does not
   integrate with fp-library's `Brand` system; the production
   types use `Brand` machinery throughout. Migration is mostly
   mechanical (swap the stub Coyoneda for fp-library's, swap the
   raw Coproduct types for branded equivalents) but expect
   surface-area changes around the macro output (the `effects!`
   macro must emit Brand-shaped types in production). Plan one
   step per POC test as a regression-safety strategy.
3. **`Drop` correctness (Phase 1).** `RcFree` and `ArcFree`
   inherit `Free`'s iterative `Drop` strategy via the underlying
   `CatList`; `FreeExplicit` requires its own iterative `Drop`
   per the POC findings. Test deep-`Drop` for all four variants
   in Phase 1 unit tests.
4. **Async-via-target-monad gating (Phase 5+).** The interpreter
   functions stay sync; an async target monad arrives in Phase 6+.
   Until then, async users wrap the interpreter call in
   `spawn_blocking` or similar.

## Success criteria

The plan is complete when all of the following hold:

- `Run<Effects, ScopedEffects, A>` is publicly exported from
  `fp-library` with `RcRun`, `ArcRun`, `RunExplicit` aliases.
- The `effects!` macro accepts `effects![A, B, C]` over arbitrary
  effect types and produces a canonical row across input
  orderings; the same row composes with `CoproductSubsetter`
  permutation proofs for hand-written cases.
- `m_do!` and `a_do!` work over `RunBrand` for first-order effect
  programs.
- Each of the four Free variants supports its promised property
  (single-shot, multi-shot, thread-safe, non-`'static`) with
  per-variant unit tests passing.
- `Reader`, `State`, `Except`, `Writer`, `Choose` ship as standard
  first-order effects with smart constructors.
- `Catch<'a, E>` and `Span<'a, Tag>` ship as Val-only
  scoped-effect constructors; `Local` ships in Val and Ref
  flavours (`Local<'a, E>` + `RefLocal<'a, E>`); `Bracket`
  ships in Val and Ref flavours (`Bracket<'a, A, B>` +
  `RefBracket<'a, P, A, B>` parameterised over
  `P: RefCountedPointer`), each with scoped-handler
  interpreters. The `bracket` and `local` smart constructors
  use closure-driven Val/Ref dispatch (mirroring
  [`dispatch.md`](../../../fp-library/docs/dispatch.md)) so the
  user picks the flavour by closure type, not by turbofish.
  (`Mask` deferred per [decisions.md](decisions.md) section 4.5
  sub-decisions.)
- The by-value hierarchy (`Functor` / `Pointed` / `Semimonad` /
  `Monad`) and by-reference hierarchy
  (`RefFunctor` / `RefSemimonad` / `RefMonad`) are both
  implemented for every Free variant brand
  (`RcFreeBrand`, `ArcFreeBrand`, `FreeExplicitBrand<F>`) and
  every Run variant brand (`RunBrand`, `RcRunBrand`,
  `ArcRunBrand`, `RunExplicitBrand`); `dispatch::map` /
  `dispatch::bind` route correctly for both consuming and
  borrowing closures.
- The TalkF + DinnerF integration test passes.
- All 25 row-canonicalisation tests migrated from
  `poc-effect-row/` pass under the production types.
- Per-Free-variant Criterion benches show no regression beyond
  ~50% of the `FreeExplicit` POC baseline (~27ns / node in the
  linear regime).
- `just verify` passes (fmt, check, clippy, deny, doc, test).

## Reference material

- Design and decisions: [decisions.md](decisions.md).
- Effects research arc:
  [research/](research/) (13 codebase classifications,
  `_classification.md` synthesis, 3 Stage 2 deep dives).
- Type-level-sorting research arc:
  [../type-level-sorting/research/](../type-level-sorting/research/)
  (16 codebase classifications, `_classification.md` synthesis).
- POC validation:
  - [poc-effect-row/](../../../poc-effect-row/) — row-encoding
    hybrid, `tstr_crates` refinement, static-via-Coyoneda.
  - [poc-effect-row-canonicalisation.md](poc-effect-row-canonicalisation.md)
    — POC findings document.
  - [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs)
    — `FreeExplicit` POC.
  - [fp-library/benches/benchmarks/free_explicit.rs](../../../fp-library/benches/benchmarks/free_explicit.rs)
    — `FreeExplicit` Criterion bench.
- PureScript Run reference:
  [`purescript-run`](https://github.com/natefaubion/purescript-run).
- Comparison table for the Rust port versus PureScript Run and
  Hasura's `eff` is in [decisions.md](decisions.md) section 10.
