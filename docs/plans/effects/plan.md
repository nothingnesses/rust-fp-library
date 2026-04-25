# Plan: Port purescript-run to fp-library

**Status:** NOT STARTED

## Current progress

Pre-implementation phase. Design and decisions are recorded in
[decisions.md](decisions.md). Research artifacts are in
[research/](research/) (13 codebase classifications, 3 Stage 2
deep dives, 1 synthesis). Two POC suites validate the most-novel
design choices:

- [poc-effect-row/](../../../poc-effect-row/) — 25 tests across two
  suites validating the row-encoding hybrid (workaround 1 macro
  plus workaround 3 `CoproductSubsetter` fallback), the
  `tstr_crates` Phase 2 refinement, and static-via-Coyoneda
  Functor dispatch end-to-end. See
  [poc-effect-row-canonicalisation.md](poc-effect-row-canonicalisation.md)
  for findings.
- [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs)
  — validates the `FreeExplicit` variant for non-`'static`
  effect payloads (6 tests passing, 1 intentionally `#[ignore]`d
  to document `Drop` overflow before the iterative custom `Drop`
  ships).

Implementation has not yet started. Phase 1 begins by promoting
`FreeExplicit` from POC and adding `RcFree`, `ArcFree` siblings.

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

None yet.

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

`runReader: Run<R + READER, A> -> Run<R, A>`-style row narrowing
matches PureScript Run; the macro layer plus
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
  `Mask`, `Bracket<'a, A>`, `Span<Tag>`). Day-one `'a` parameter,
  fixed `Run<R, A>` continuation, coproduct-of-constructors
  extension shape.
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

| Component                                 | Change                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| ----------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fp-library/src/types/free.rs`            | Existing `Free<F, A>` keeps its current shape; minor adjustments if integration with `Run` requires.                                                                                                                                                                                                                                                                                                                                                                                                  |
| `fp-library/src/types/free_explicit.rs`   | **New module.** Promote `FreeExplicit<'a, F, A>` from POC, add iterative custom `Drop`, add `Functor` / `Pointed` / `Semimonad` / `Monad` impls.                                                                                                                                                                                                                                                                                                                                                      |
| `fp-library/src/types/rc_free.rs`         | **New module.** `RcFree<F, A>` following the `Free` template with `Rc<dyn Fn>` continuations. Multi-shot effects (`Choose`, `Amb`).                                                                                                                                                                                                                                                                                                                                                                   |
| `fp-library/src/types/arc_free.rs`        | **New module.** `ArcFree<F, A>` following `ArcCoyoneda` template with `Arc<dyn Fn + Send + Sync>` and the `Send`/`Sync` Kind-trait pattern.                                                                                                                                                                                                                                                                                                                                                           |
| `fp-library/src/types/run.rs`             | **New module.** `Run<Effects, ScopedEffects, A>` plus `RcRun`, `ArcRun`, `RunExplicit` aliases. `Node<...>` enum dispatching first-order vs scoped.                                                                                                                                                                                                                                                                                                                                                   |
| `fp-library/src/types/run/variant_f.rs`   | **New submodule.** `VariantF<Effects>` first-order coproduct with Coyoneda-wrapped variants and recursive `Functor` impl on `Coproduct<H, T>`.                                                                                                                                                                                                                                                                                                                                                        |
| `fp-library/src/types/run/scoped.rs`      | **New submodule.** `ScopedCoproduct<ScopedEffects>` higher-order coproduct, standard scoped constructors (`Catch`, `Local`, `Mask`, `Bracket`, `Span`).                                                                                                                                                                                                                                                                                                                                               |
| `fp-library/src/types/run/handler.rs`     | **New submodule.** Handler-pipeline machinery (`Run::handle`), natural-transformation type, `peel` / `send` / `extract`.                                                                                                                                                                                                                                                                                                                                                                              |
| `fp-library/src/types/run/interpreter.rs` | **New submodule.** `interpret` / `run` / `runAccum` (recursive) and `interpretRec` / `runRec` / `runAccumRec` (`MonadRec`-targeted) families.                                                                                                                                                                                                                                                                                                                                                         |
| `fp-macros/src/effects/`                  | **New module tree.** `effects!`, `effects_coyo!`, `handlers!`, `define_effect!`, `define_scoped_effect!` proc-macros. Migration from POC.                                                                                                                                                                                                                                                                                                                                                             |
| `fp-library/src/brands.rs`                | Add brands for new types: `RunBrand`, `RcRunBrand`, `ArcRunBrand`, `RunExplicitBrand`, `RcFreeBrand`, `ArcFreeBrand`, `FreeExplicitBrand<F>`. `FreeExplicitBrand<F>` is a single-parameter `PhantomData<F>` struct mirroring [`CoyonedaBrand<F>`](../../../fp-library/src/brands.rs#L155); `'static` bounds live on impls (per [`CoyonedaExplicitBrand<F, B>`'s convention](../../../fp-library/src/brands.rs#L171)) so `FreeExplicit<'a, F, A>`'s `'a` and `A` stay in `Of<'a, A>` at instantiation. |
| `fp-library/tests/run_*.rs`               | **New test files.** Per-Free-variant unit tests (Phase 1), row-canonicalisation regression tests migrated from `poc-effect-row/` (Phase 2), TalkF + DinnerF integration test (Phase 4).                                                                                                                                                                                                                                                                                                               |
| `fp-library/benches/benchmarks/run_*.rs`  | **New bench files.** Per-Free-variant Criterion benches (bind-deep, bind-wide, peel-and-handle), row-canonicalisation benches (macro vs Subsetter), handler-composition benches.                                                                                                                                                                                                                                                                                                                      |

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
   following the `Free` template with `Rc<dyn Fn>` continuations
   and the `RcCoyoneda` cloning pattern. Add `lower_ref(&self)` /
   `peel_ref(&self)` for non-consuming reinterpretation.
3. Implement `ArcFree<F, A>` at `fp-library/src/types/arc_free.rs`
   following the `ArcCoyoneda` template (`Arc<dyn Fn + Send + Sync>`,
   the `Kind<Of<'a, A>: Send + Sync>` associated-type-bound trick).
4. Add brands and `Functor` / `Pointed` / `Semimonad` / `Monad` impls
   for `RcFreeBrand`, `ArcFreeBrand`, `FreeExplicitBrand<F>`. The
   default `Free`'s impls are the template.
5. Per-variant Criterion benches (bind-deep at depths 10 / 100 /
   1000 / 10000, bind-wide, peel-and-handle). Match the
   `FreeExplicit` POC bench shape.
6. Per-variant unit tests covering construction, evaluation,
   `fold_free` interpretation, and the property each variant
   promises (single-shot, multi-shot, thread-safe,
   non-`'static`). Plus `compile_fail` UI tests for the negative
   cases (e.g., trying to multi-shot a `Free`).

### Phase 2: Run substrate and first-order effects

1. `Coproduct<H, T>` and `CNil` types under `fp-library/src/types/`
   (or re-export from `frunk_core` if the project decides to
   depend on it; see Implementation note 1 below).
2. `VariantF<Effects>` at `fp-library/src/types/run/variant_f.rs`:
   Coyoneda-wrapped Coproduct row with recursive `Functor` impl
   on `Coproduct<H, T>` (where `H: Functor + T: Functor`) and base
   case on `CNil`. Migrate the trait-shape from
   [poc-effect-row/src/lib.rs](../../../poc-effect-row/src/lib.rs)
   under the production `Functor` trait.
3. `Member<E, Indices>` trait for first-order injection /
   projection, plus `CoproductSubsetter` if not via `frunk_core`.
4. `Run<Effects, ScopedEffects, A>` core type at
   `fp-library/src/types/run.rs` with `RcRun`, `ArcRun`,
   `RunExplicit` aliases. `Node<Effects, ScopedEffects>` enum.
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
2. Standard scoped-effect constructors:
   - `Catch<'a, E>` for `Error.catch` — `action: Run<R, S, A>`,
     `handler: Box<dyn FnOnce(E) -> Run<R, S, A>>`.
   - `Local<'a, E>` for `Reader.local` — `modify: Box<dyn FnOnce(E) -> E>`,
     `action: Run<R, S, A>`.
   - `Mask<'a, E>` — `action: Run<R, S, A>`,
     `effect: PhantomData<E>` (compile-time effect identifier;
     supports non-`'static` effects uniformly with the rest of
     the design, see [decisions.md](decisions.md) section 4.5
     sub-decisions).
   - `Bracket<'a, A, B>` — `acquire: Run<R, S, A>`,
     `release: Box<dyn FnOnce(A) -> Run<R, S, ()>>`,
     `body: Box<dyn FnOnce(&A) -> Run<R, S, B>>`. Two type
     parameters matching PureScript's
     `bracket :: Run r a -> (a -> Run r Unit) -> (a -> Run r b) -> Run r b`.
   - `Span<'a, Tag>` — `tag: Tag`, `action: Run<R, S, A>`.
3. Scoped-effect interpreter trait. Method per constructor;
   fixed `Run<R, A>` continuation
   ([decisions.md](decisions.md) section 4.5).
4. `scoped_effects!` macro and `define_scoped_effect!` macro,
   sharing the lexical-sort helper with Phase 2's `effects!` (one
   helper, two thin entry-point macros, distinct output shapes:
   Coyoneda-wrapped Coproduct vs `ScopedCoproduct`).
5. Smart constructors: `local`, `catch`, `mask`, `bracket`, `span`.
6. Standard handlers (`run_reader`'s `local` clause,
   `run_except`'s `catch` clause, etc.) wired through the dual
   row.
7. Tests: scoped-effect unit tests covering each of the five
   standard constructors plus `compile_fail` cases. Reformulate
   relevant Phase 3 tests to use scoped operations where
   appropriate.

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

These are listed for completeness; they arrive when concrete
need surfaces:

- `RcFreeExplicit` / `ArcFreeExplicit` intersection variants for
  the rare combination of (multi-shot or thread-crossing) with
  non-`'static` payloads
  ([decisions.md](decisions.md) section 4.4).
- `MonadRec` impl for `Future` as an async target monad
  ([decisions.md](decisions.md) section 9 items 3 + 4).
- Optional split of `fp-macros` into `fp-effects-macros` if the
  crate becomes too large
  ([decisions.md](decisions.md) section 9 item 8).
- Open questions left after section 4.4: parallel
  `Send`-constrained `Functor` / `Monad` trait hierarchy for
  `ArcFree` if the existing `Send`-families don't cover, cargo
  feature gating for `RcFree` / `ArcFree` if compile cost
  matters.

## Implementation notes

1. **`Coproduct` choice (Phase 2).** The POC depends on
   `frunk_core::coproduct::{Coproduct, CNil, CoproductSubsetter}`.
   Decision: depend on `frunk_core` directly with a thin
   Brand-aware adapter layer (newtypes plus `impl` blocks that
   bridge `frunk_core`'s Plucker / Sculptor / Embedder traits to
   the project's `Brand` system). Switch to an in-house
   reimplementation only if the adapter exceeds approximately 200
   lines, which would indicate the impedance mismatch is real
   enough to justify the maintenance cost. Implementing the
   project's own `Functor` for `frunk_core::Coproduct<H, T>` is
   permitted by the orphan rules (own-trait + foreign-type);
   `Brand`-style impls on the foreign type require the newtype
   wrapper.
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
- `Catch`, `Local`, `Mask`, `Bracket`, `Span` ship as standard
  scoped-effect constructors with smart constructors and
  scoped-handler interpreters.
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
