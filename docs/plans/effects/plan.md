# Plan: Port purescript-run to fp-library

**Status:** Phase 1 complete (all 9 steps); Phase 1 follow-up
both commits (`WrapDrop` migration plus `Functor` -> `Kind`
relaxation) landed; **Phase 2 complete (all 10 steps)**. Phase 3
(first-order effect handlers, interpreters, natural
transformations) is the next phase.

## Current progress

Phase 1 complete (steps 1-9). Phase 1 follow-up commits 1 and 2
complete. Phase 2 complete: steps 1, 2, 3, 4a, 4b, 5, 6, 7a, 7b,
7c.1, 7c.2a, 7c.2b, 8, 9 (all sub-steps 9a, 9b+9e, 9c+9f, 9d+9g,
9h, 9i), 10a (POC test migration into
`fp-library/tests/run_row_canonicalisation.rs`), and 10b
(`poc-effect-row/` workspace deleted). Phase 3 in progress: step
1 (`handlers!{...}` macro plus `nt()` builder fallback) and step
2 (`interpret`/`run`/`run_accum` recursive-target interpreter
family across all six Run wrappers) landed.

The three entries below carry the rolling detail for the most
recent steps. Older steps' detailed narratives live in commit
messages and [deviations.md](deviations.md); see the **Earlier
completed steps (commit log)** subsection further down.

**Phase 3 step 2 (this commit): `interpret`/`run`/`run_accum`
recursive-target interpreter family across all six Run wrappers.**
New module
[`fp-library/src/types/effects/interpreter.rs`](../../../fp-library/src/types/effects/interpreter.rs)
hosts the `DispatchHandlers<'a, Layer, NextProgram>` trait that
walks a [`HandlersCons`](../../../fp-library/src/types/effects/handlers.rs)
in lock-step with the row's value-level
[`Coproduct`](../../../fp-library/src/types/effects/coproduct.rs)
chain. Three cons-cell impls cover one Coyoneda variant each
(`Coyoneda` / `RcCoyoneda` / `ArcCoyoneda`); the empty case is
`HandlersNil`/`CNil`. Each wrapper exposes inherent
`interpret`/`run`/`run_accum` methods that loop on `peel`,
dispatch each `Node::First` layer through `DispatchHandlers`,
and panic on `Node::Scoped` (Phase 4 will route scoped
dispatch).

Per-wrapper deltas: ArcRun's loop pattern-matches `Node::First`
through a free-function helper
[`unwrap_first`](../../../fp-library/src/types/effects/arc_run.rs)
to sidestep the same struct-level HRTB-poisoning that drove
[`lift_node`](../../../fp-library/src/types/effects/arc_run.rs)
in Phase 2 step 5; the other five wrappers pattern-match inline.
RcRun / RcRunExplicit / ArcRun / ArcRunExplicit add the
substrate-specific `Clone` / `Send + Sync` bounds matching their
respective `peel` signatures. State threading in `run_accum` is
via closure captures (`Rc<RefCell<S>>` for single-threaded
substrates, `Arc<Mutex<S>>` for ArcRun / ArcRunExplicit), which
matches PureScript Run's `runAccum :: ... -> Run r a -> m a`
shape (state is internal to the loop; final result is `A` only).

The handler closure's mono-in-`A` shape matches PureScript Run's
actual runtime model
([`Run.purs:184-217`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs#L178-L217)
shows `interpret = run` aliasing). The handler closure receives
the Coyoneda-lowered effect (`<EBrand as Kind>::Of<'_, NextProgram>`)
and returns the next program. Users name the _inner_ effect
brand (`IdentityBrand`, `StateBrand`, etc.) in the `handlers!`
macro for all six wrappers, matching `effects!`'s sort key; the
DispatchHandlers impls bind the inner brand and dispatch on the
relevant Coyoneda value variant.

Tests: 12 integration tests in
[`fp-library/tests/run_interpret.rs`](../../../fp-library/tests/run_interpret.rs)
covering single-effect interpretation, bind-chain interpretation,
the `run` alias matching `interpret`, and `run_accum` with state
threading via `Rc<RefCell<...>>` (single-threaded) or
`Arc<Mutex<...>>` (Send + Sync). Per-wrapper doctests on each
`interpret` / `run` / `run_accum` method exercise the
canonical-row-and-handler combination. `just verify` clean: 2456
unit tests + 12 integration tests + doctests on each method
compile and pass.

Plan.md Phase 6+ deferred-items section gains an entry for a
future `interpret_nt`-style companion entry-point taking
[`NaturalTransformation`](../../../fp-library/src/classes/natural_transformation.rs)
directly (the existing rank-2 polymorphic trait used by
[`Free::fold_free`](../../../fp-library/src/types/free.rs)),
preserving the future-revisit context per the user's request.

Per-step deviation entry in
[deviations.md](deviations.md) Phase 3 step 2 records: the
three-impl Coyoneda-variant traversal pattern; the mono-in-`A`
step-function rationale; per-wrapper inherent-method layout;
ArcRun's `unwrap_first` HRTB-poisoning workaround mirroring
`lift_node`'s precedent; the `Scoped` arm panic gated by
`#[expect(clippy::unreachable, ...)]` until Phase 4; state
threading via closure captures vs a separate stateful trait;
and the inner-brand handler-list key matching `effects!`'s
sort.

Step 3 (`interpret_rec` / `run_rec` / `run_accum_rec` MonadRec-target
interpreter family) is the immediate next work; the
`DispatchHandlers` trait reuses unchanged, only the loop body
switches from host-stack recursion to `tail_rec_m` trampolining.

**Phase 3 step 1: `handlers!{...}` macro plus
`nt()` builder fallback for assembling natural transformations
`VariantF<R> ~> M`.** New runtime carrier in
[`fp-library/src/types/effects/handlers.rs`](../../../fp-library/src/types/effects/handlers.rs):
`Handler<E, F>` newtype (pins brand identity at the type level
via `PhantomData<fn() -> E>`); `HandlersNil` / `HandlersCons<H,
T>` cons-list types whose cells mirror the row brand
`CoproductBrand<H, T>` / `CNilBrand` chain cell-for-cell;
inherent `.on::<E, F>(self, handler)` builder methods on both
list types using prepend semantics; `nt()` entry-point.

New macro at
[`fp-macros/src/effects/handlers.rs`](../../../fp-macros/src/effects/handlers.rs)
with `#[proc_macro] pub fn handlers` entry-point in
[`fp-macros/src/lib.rs`](../../../fp-macros/src/lib.rs). Parses
`Brand: expression, Brand: expression, ...` syntax (brand parses
as `syn::Type`, expression as `syn::Expr` so closure literals,
function references, and pre-built handler values all work),
sorts entries lexically by `quote!(brand).to_string()` (matching
[`effects!`](../../../fp-macros/src/effects/effects_macro.rs)'s
sort key exactly), and emits a right-nested `HandlersCons` chain
terminated in `HandlersNil`. Each entry's expression is wrapped
in `Handler::<Brand, _>::new(...)`. Empty input emits just
`HandlersNil`.

Re-export pattern: per the optics A+B hybrid, the handler types
ship at the subsystem-scope `crate::types::effects::*`
(`Handler`, `HandlersCons`, `HandlersNil`, `nt`) but not at the
top-level `crate::types::*`; the Run wrappers hold the
headline-types tier. The `handlers!` macro re-exports through
the standard `pub use fp_macros::*` path and is user-facing (no
`__internal` marker).

The closure shape `F` carried inside each `Handler<E, F>` is left
fully generic. Phase 3 step 2's interpreter family
(`interpret`/`run`/`runAccum`) will pin the concrete shape
via an interpreter-side trait bound.

Tests: 10 integration tests in
[`fp-library/tests/handlers_macro.rs`](../../../fp-library/tests/handlers_macro.rs)
covering empty input, single-entry shape, two- and three-entry
canonical-ordering equivalence (input order does not affect the
emitted type), trailing-comma acceptance, brand-pinning in the
emitted `Handler<Brand, _>` shape, the `nt()` builder's
prepend-semantics property, and macro/builder shape-equivalence
when the builder is called in reverse-lexical order. 6
token-string assertion tests in
[`fp-macros/src/effects/handlers.rs`](../../../fp-macros/src/effects/handlers.rs)
cover the worker function directly. 6 inline unit tests in the
runtime-carrier module exercise the builder methods and
struct-literal construction. `just verify` clean (2437 unit
tests; 10 new integration tests; 6 worker-token tests).

Per-step deviation entry in
[deviations.md](deviations.md) Phase 3 step 1 records: (1) the
runtime-carrier shape choice (dedicated `HandlersNil`/`HandlersCons`
distinct from `frunk_core::hlist::{HCons, HNil}` to keep the
type-level position vs runtime closure carrier roles separate
and to enable inherent `.on()` methods); (2) prepend semantics
on the builder, with macro lexical sorting handling alignment
to canonical row order; (3) the macro lives next to
[`effects_macro.rs`](../../../fp-macros/src/effects/effects_macro.rs)
but does not reuse the
[`row_sort.rs`](../../../fp-macros/src/effects/row_sort.rs)
helper because the parser shapes differ
(`Punctuated<Type, ...>` vs `Punctuated<HandlerEntry, ...>`);
(4) `Handler<E, F>` uses `PhantomData<fn() -> E>` for variance
neutrality; (5) re-export at subsystem scope only; (6) no
`__internal` re-export.

Step 2 (`interpret`/`run`/`runAccum` family) is the immediate
next work and will consume the handler-list shape this step
ships. The interpreter trait will dispatch over `Run::peel` /
`*Run::peel` results, recursing through the handler list in
lock-step with the row's `Coproduct::Inl`/`Inr` branches.

**Phase 2 step 10a (`162ab1e`): row-canonicalisation regression
baseline at
[`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs).**
20 integration tests migrated from
`poc-effect-row/tests/` (workspace deleted in 10b),
substituting fp-library's existing brands
(`IdentityBrand` / `OptionBrand` / `ResultBrand` / `BoxBrand` /
`CatListBrand` / `ThunkBrand` / `SendThunkBrand` /
`TryThunkBrand` / `ArcCoyonedaBrand`) for the POC's local
`struct A(i32)` / `struct B(...)` placeholder types.

Coverage: canonicalisation across all 6 permutations of 3
brands; 5- and 7-brand scaling; lexical-canonical-form check;
empty / single-brand edge cases; same-root different-params sort
(via `CoyonedaBrand<Identity>` vs `CoyonedaBrand<Option>`);
`CoproductSubsetter` (workaround 3) on runtime
`Coproduct` values across 3- and 5-element permutations;
`effects!` vs `raw_effects!` Coyoneda wrapping difference;
each row brand drives all six Run wrappers
(`Run`/`RcRun`/`ArcRun`/`RunExplicit`/`RcRunExplicit`/`ArcRunExplicit`).
Arc-family wrappers use `ArcCoyonedaBrand`-headed rows to
satisfy the substrate's struct-level `Send + Sync` HRTB.

Tests intentionally skipped from the POC (4 of 25 not migrated;
a fifth, `c08`, is implicitly covered):

- `feasibility::t08` (1 test): lifetime-parameter-bearing raw
  effect type. Production effect brands are zero-sized `'static`
  markers (no lifetime params at the brand level); the test
  exercises a property production brands cannot have.
- `feasibility::t10` (1 test): "handler accepts macro output as
  runtime value". In POC, effect types served as both row brands
  AND runtime value types; in production the brand is a
  zero-sized marker and the analog is the all-six-Run-wrappers
  integration tests (a row brand drives the wrapper's type
  parameters).
- `feasibility::t14`-`t16` (3 tests): `tstr_crates` compile-time
  string-ordering demos; not testing fp-library; would require
  adding `tstr` as a dev-dep to no regression-baseline benefit.
- `coyoneda::c03`-`c05` (3 tests): POC-local `Coyoneda` lift +
  decoder-closure mechanics. Production
  [`Coyoneda`](../../../fp-library/src/types/coyoneda.rs) has
  no decoder closure (uses brand-Kind machinery directly); the
  mechanics-tests don't translate.
- `coyoneda::c08` (1 test, implicitly covered): Coproduct-of-
  Coyoneda end-to-end fmap dispatch, exercised by the
  [`tests/run_lift.rs`](../../../fp-library/tests/run_lift.rs)
  round-trip tests on all six Run wrappers (lift -> peel -> lower
  recovers the value, which only works if the Coproduct +
  Coyoneda dispatch composes correctly).

Per-test mapping is documented in
[deviations.md](deviations.md) under step 10a, with a follow-up
on `coyoneda::c06`: that test was initially landed as a
PhantomData-only placeholder during the migration; on review it
was replaced with `subsetter_over_runtime_coyoneda_wrapped_values`
that constructs production `Coyoneda::lift(Identity(7))` runtime
values, builds a non-canonical Coproduct of Coyoneda variants,
and runs `.subset()` to recover the canonical permutation. This
is the production analog of the POC's `c06` and the gap the
review surfaced.

Plus net-new coverage that wasn't in the POC (all 6 permutations
vs POC's 3, `effects!` vs `raw_effects!` contrast,
all-six-Run-wrappers integration). All 2437 unit tests pass
under `just verify` (no `src/` changes in this commit;
integration-test-only addition); 21 new integration tests in
`tests/run_row_canonicalisation.rs` all pass.

Step 10b (delete `poc-effect-row/` workspace) is the only
remaining work and is held for explicit user confirmation
because it is destructive. The POC workspace is detached
(declares its own `[workspace]` block) and not built by the
outer cargo workspace, so the deletion is safe but
irreversible.

### Earlier completed steps (commit log)

Each entry's design choices are recorded in
[deviations.md](deviations.md) under the corresponding step
heading; the commit message has the full implementation
summary; resolved blockers are in
[resolutions.md](resolutions.md). Listed newest-first.

Phase 2:

- `fe4ad59` (step 10b): `poc-effect-row/` workspace deleted; the
  POC's job is done after 10a migrated 21 of 25 tests to
  [`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs)
  (4 documented as not-applicable; 1 implicitly covered). 8
  files removed (~97MB including untracked `target/` cache).
  The POC declared its own `[workspace]` block so the outer
  cargo workspace was unaffected by its presence and absence.
  Doc-link maintenance across four cross-referencing files;
  Phase 2 ships complete with this commit.
- `df99ff6` (step 9i): `SendRefPointed` lands on
  `ArcRunExplicitBrand` via inherent-method delegation; the rest
  of the SendRef cascade (`SendRefFunctor` /
  `SendRefSemimonad` / `SendRefSemiapplicative` /
  `SendRefApplicative` / `SendRefMonad`) is blocked by the
  closure-bound mismatch (`Fn(&A) -> B + Send + 'a` vs
  `ArcRunExplicit::ref_map`'s `Send + Sync` requirement) plus
  three per-`A` HRTB walls (per-`A` `Clone`, per-`A`
  `<R as Kind>::Of<...>: Clone + Send + Sync`, same for `S` and
  `NodeBrand<R, S>`). Documented in the
  [Send-aware Ref coverage table](../../../fp-library/docs/limitations-and-workarounds.md)
  parallel to 9d's by-value table on `ArcFreeExplicitBrand`. The
  user-facing by-reference Send-aware surface is the inherent
  `ArcRunExplicit::ref_map` / `ref_bind` / `ref_pure` methods;
  `im_do!(ref ArcRunExplicit { ... })` desugars to these so user
  code is unaffected.
- `199370b` (step 9h): universal `*Run::lift` across all six Run
  wrappers. Plan's per-wrapper delta table corrected: each
  wrapper's `lift` uses the Coyoneda variant whose pointer kind
  matches its substrate (Run/RunExplicit -> Coyoneda;
  RcRun/RcRunExplicit -> RcCoyoneda; ArcRun/ArcRunExplicit ->
  ArcCoyoneda) because `*Run::send`/`*Run::peel` carry per-method
  `Of<'_, *Free<..., *TypeErasedValue>>: Clone` bounds intrinsic
  to the shared substrate state. `ArcRun::lift` uses the
  `lift_node` HRTB-poisoning fallback. Side artefact: added the
  missing `RcCoyonedaBrand: WrapDrop` impl (step 9a's commit
  message claimed mirroring this pattern but it didn't exist).
  11 new integration tests in `tests/run_lift.rs`.
- `42e698a` (step 9d+9g bundle): brand-level Send-aware surface
  unchanged on both `ArcFreeExplicitBrand` and
  `ArcRunExplicitBrand` (per-`A` HRTB-over-types blocker confirmed
  via rustc probe). Inherent `ArcFreeExplicit::map` lands as the
  concrete-type workaround; the bare name `map` (Send + Sync in
  the where-clause) matches Arc-substrate naming convention. 9g
  is a strict logical consequence of 9d (no code changes), bundled
  with 9d to avoid a content-free follow-up commit. Limitations doc
  gains a Send-aware brand-level coverage table.
- `9295a26` (step 9c+9f bundle): replace `F: Functor` with
  `F: SendFunctor` on `ArcFreeExplicit`; switch `ArcRunExplicit`
  to `SendFunctor`-routed dispatch. Mirrors the 9b+9e bundle for
  the Explicit family. `ArcFreeExplicit`'s impl-block bound
  switches; the single `F::map` call inside `bind_boxed` becomes
  `F::send_map`. `SendPointed` impl on `ArcFreeExplicitBrand`
  similarly switches to `F: SendFunctor`. `ArcRunExplicit`'s
  struct/impl-block bounds switch to `R/S: SendFunctor`; two
  `<NodeBrand as Functor>::map` calls become `<as SendFunctor>::send_map`;
  per-method bounds gain `Send + Sync` cascade on `A`, `B`, and the
  row projections. `arc_free_explicit_bind_requires_send`'s
  `.stderr` regenerated.
- `f86c150` (step 9b+9e bundle): replace `F: Functor` with
  `F: SendFunctor` on `ArcFree`; switch `ArcRun` to
  `SendFunctor`-routed dispatch. Bundled because `ArcRun::peel`/
  `send` route through `ArcFree`'s methods, so the bound
  replacement cascades. Eight per-method bound updates on
  `ArcFree`; three `F::map`->`F::send_map` calls; `wrap` gains
  `A: Send + Sync`; `hoist_free` switches to `G: SendFunctor`.
  `ArcRun`'s two per-method `Functor` bounds become `SendFunctor`,
  one `<NodeBrand as Functor>::map` becomes `<as SendFunctor>::send_map`.
  `arc_run_normalization_probe.rs`'s pattern-A and
  `arc_run_explicit.rs`'s `From<ArcRun>` impl track the change.
- `779651e` (step 9a): brand-level `SendFunctor` cascade
  prerequisites for sub-steps 9b through 9g. Adds
  `IdentityBrand: SendFunctor`, `CNilBrand: SendFunctor`,
  `CoproductBrand<H, T>: SendFunctor`,
  `NodeBrand<R, S>: SendFunctor`, plus the missing
  `ArcCoyonedaBrand: WrapDrop` impl. Each is a near-mirror of
  the same brand's existing `Functor` or `WrapDrop` impl with
  the `Send + Sync` bound added to closure parameters. Six new
  doctests; `arc_coyoneda.rs` module docs updated to list
  `WrapDrop` alongside `Foldable` and `SendFunctor`.
- `9929563` (step 8): `effects!` proc-macro migration to
  [`fp-macros/src/effects/effects_macro.rs`](../../../fp-macros/src/effects/effects_macro.rs)
  plus `raw_effects!` companion at
  [`fp_library::__internal`](../../../fp-library/src/lib.rs).
  Lexical-sort helper at
  [`fp-macros/src/effects/row_sort.rs`](../../../fp-macros/src/effects/row_sort.rs)
  shared with the future `scoped_effects!`. Ten integration
  tests verify canonical-ordering and explicit-shape via
  `assert_type_eq` / `PhantomData`; six fp-macros unit tests
  cover the worker functions and sort helper directly.
- `2121174` (step 7c.2b): `im_do!` proc-macro at
  [`fp-macros/src/effects/im_do/codegen.rs`](../../../fp-macros/src/effects/im_do/codegen.rs).
  Inherent-method dispatch (`expr.bind(...)` /
  `expr.ref_bind(...)`); `pure(x)` rewriting to
  `Wrapper::pure(x)` / `Wrapper::ref_pure(&(x))`. 16 integration
  tests cover all six wrappers; one compile_fail UI test
  demonstrates the natural rejection of `im_do!(ref Run { ... })`
  on non-`Clone` wrappers.
- `e4cf7b5` (step 7c.2a): shared `DoInput` parser extraction
  from `fp-macros/src/m_do/input.rs` to
  [`fp-macros/src/support/do_input.rs`](../../../fp-macros/src/support/do_input.rs).
  Reused by all four do-notation macros (`m_do!`, `a_do!`,
  `im_do!`, future `ia_do!`); pure refactor with no behavior
  change.
- `10d17fe` (step 7c.1): inherent `ref_pure` on the four
  `Clone`-able wrappers (`RcRun`, `ArcRun`, `RcRunExplicit`,
  `ArcRunExplicit`). Pattern `Self::pure(a.clone())`; bounds
  `A: Clone` (plus `+ Send + Sync` on `ArcRun`). Rounds out the
  inherent by-reference surface so `im_do!(ref Wrapper {
... pure(x) })` rewrites `pure(x)` -> `Wrapper::ref_pure(&x)`
  parallel to `m_do!`'s brand-level path.
- `6dc802e` (step 7b): inherent `ref_bind`/`ref_map` on the
  four `Clone`-able wrappers (`RcRun`, `ArcRun`,
  `RcRunExplicit`, `ArcRunExplicit`). Pattern
  `self.clone().bind(move |a| f(&a))`; `O(1)` clone sidesteps
  the `R: RefFunctor` cascade brand-level dispatch requires.
- `ef6257e` (step 7a): inherent `bind`/`map` on `Run`,
  `RcRun`, `ArcRun`, `RunExplicit`. The other two wrappers
  shipped them in step 4b.
- `7f5be3c` (step 6 follow-up): refactored conversion surface
  from inherent `into_explicit`/`from_erased` methods to
  [`From`](https://doc.rust-lang.org/std/convert/trait.From.html)
  impls. Matches the codebase's ~35 sibling-type `From`
  precedent; users get both `Explicit::from(erased)` and
  `erased.into()` for free via the blanket
  [`Into`](https://doc.rust-lang.org/std/convert/trait.Into.html).
- `11a89bc` (step 6): three Erased -> Explicit Run conversions
  via [`From`](https://doc.rust-lang.org/std/convert/trait.From.html).
  Each walks the underlying Free chain via `peel` and rebuilds
  via `wrap`; preserves multi-shot/`Send + Sync` per substrate.
  O(N) in chain depth (one stack frame per suspended `Wrap`
  layer; structural depth at most 1 for Run-typical patterns
  per the Wrap-depth probe).
- `4950c50` (step 5): inherent `pure`/`peel`/`send` on each of
  the six Run wrappers. `send` takes a pre-constructed
  `Node`-projection value (rather than a row-variant layer) to
  sidestep HRTB-poisoning under `ArcFree`'s impl-block scope;
  see [resolutions.md](resolutions.md) for the full
  investigation. Step 5 also adds
  [`FreeExplicit::to_view`](../../../fp-library/src/types/free_explicit.rs)
  as a precursor.
- `289d3c6` (step 4b): three Explicit Run wrappers (`RunExplicit`,
  `RcRunExplicit`, `ArcRunExplicit`); three `*RunExplicitBrand`s
  with brand-level type-class hierarchy delegating to
  `*FreeExplicitBrand`'s impls; row-brand `RefFunctor`/`Extract`
  cascade on `CNilBrand`/`CoproductBrand`/`NodeBrand`; `Node`
  `Clone` impl; A+B hybrid re-export pattern (top-level +
  subsystem-scoped, mirrors the optics precedent). `Monad` /
  `RefMonad` / `SendMonad` / `SendRef`-family are not reachable
  through brand-level delegation; inherent `bind`/`map` on
  `RcRunExplicit`/`ArcRunExplicit` cover the by-value monadic
  surface.
- `c3712f6` (step 4a): foundation. Row-brand `WrapDrop` impls
  on `CNilBrand`/`CoproductBrand`/`CoyonedaBrand`;
  `Node`/`NodeBrand` machinery (Kind, Functor, WrapDrop, then
  `RefFunctor`/`Extract` added in 4b); three Erased Run
  wrappers (`Run`, `RcRun`, `ArcRun`). Renamed
  `fp-library/src/types/run/` to
  `fp-library/src/types/effects/`.
- `26ed053` (step 3): `Member<E, Idx>` trait at
  [`fp-library/src/types/effects/member.rs`](../../../fp-library/src/types/effects/member.rs)
  for first-order injection / projection over Coproduct rows.
  Blanket impl over `frunk_core::CoprodInjector` +
  `CoprodUninjector`. Single-effect by design; row narrowing
  stays through `CoproductSubsetter`.
- `26ef01a` (step 2): `VariantF<Effects>` Coyoneda-wrapped
  Coproduct row at
  [`fp-library/src/types/effects/variant_f.rs`](../../../fp-library/src/types/effects/variant_f.rs).
  Recursive `Functor` impl on `CoproductBrand<H, T>`
  dispatching by `Inl`/`Inr`; uninhabited base case on
  `CNilBrand` (`match fa {}`). `VariantF<H, T>` alias to
  `CoproductBrand<H, T>` exposed for canonical naming per
  [decisions.md](decisions.md) section 5.1.
- `a1d0258` (step 1): `frunk_core` dependency (license-checked)
  - Brand-aware Coproduct adapter at
    [`fp-library/src/types/effects/coproduct.rs`](../../../fp-library/src/types/effects/coproduct.rs).
    Re-exports `Coproduct`, `CNil`, `CoprodInjector`,
    `CoprodUninjector`, `CoproductSubsetter`, `CoproductEmbedder`,
    `CoproductSelector`, `CoproductTaker`, plus list helpers.

Phase 1 follow-up:

- `834f8af` (commit 2): `Functor` -> `Kind` relaxation on the
  six Free struct/`*View`/`*Step`/`Inner`/`Continuation` data
  declarations. The `Suspend`-arm `Kind` requirement is
  inherited from `WrapDrop`'s `Kind` supertrait, so no extra
  bound at the data-type sites; methods that need `F::map`
  carry `where F: Functor` per-method.
- `3dee27e` (commit 1): `WrapDrop` trait migration. New
  [`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs)
  trait at `fp-library/src/classes/wrap_drop.rs` decouples
  Drop's structural cleanup from `Extract`'s semantic
  interpretation; all six Free variants migrated their
  struct/Drop bounds from `F: Extract + Functor` to
  `F: WrapDrop + Functor`. Methods that genuinely call
  `F::extract` (`evaluate`, `lower_ref`) keep the per-method
  `F: Extract` bound. See
  [resolutions.md](resolutions.md) for the full
  investigation.

Phase 1 (the Free family, all nine steps): six Free variants
(`Free`, `RcFree`, `ArcFree`, `FreeExplicit`, `RcFreeExplicit`,
`ArcFreeExplicit`); per-variant unit tests covering
construction, chaining, multi-shot via clone where applicable,
deep evaluate / Drop, non-`'static` payloads, and
cross-thread + `Send + Sync` witness for the Arc variants;
per-variant Criterion benches (per-variant + cross-family
comparison) under
[`fp-library/benches/benchmarks/`](../../../fp-library/benches/benchmarks/);
promotion of the POC `FreeExplicit` to production at
[`fp-library/src/types/free_explicit.rs`](../../../fp-library/src/types/free_explicit.rs);
the
[`SendFunctor`](../../../fp-library/src/classes/send_functor.rs)
trait family (Phase 1 step 6) for thread-safe auto-derive on
`Arc`-substrate types; brand-level type-class hierarchies on
the three Explicit Free brands (Phase 1 step 7) with the
realistic blocked subset (`Lift` / `Semiapplicative` /
`Applicative` / `Monad` cascade + the `SendRef*` hierarchy on
`ArcFreeExplicitBrand`) documented in
[`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md);
four `compile_fail` UI tests under
[`fp-library/tests/ui/`](../../../fp-library/tests/ui/)
exercising single-shot, no-brand-on-Erased, Send-bound on
`ArcFreeExplicit::bind`, and `Clone`-bound on `RcFree::bind`
properties.

Other artefacts:

- The `poc-effect-row/` workspace was deleted in Phase 2 step
  10b after its 25 tests were either migrated to
  [`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs)
  (21 tests) or documented as not-applicable to production
  (4 tests; tstr_crates demos and a lifetime-parameter test that
  production brands cannot express); a fifth (`coyoneda::c08`)
  is implicitly covered by
  [`tests/run_lift.rs`](../../../fp-library/tests/run_lift.rs).
  The standalone planning doc
  [poc-effect-row-canonicalisation.md](poc-effect-row-canonicalisation.md)
  is preserved as research history; the deletion does not
  invalidate the findings it documents.

## Open questions, issues and blockers

This section tracks **active** blockers only. Resolved blockers
are logged in [resolutions.md](resolutions.md) for design
history. Per-step deviations from the plan are logged in
[deviations.md](deviations.md) for code-review context.

### Active blockers

#### Active blocker (2026-04-29): Phase 3 step 2/3 interpreter family shape

##### TL;DR

Phase 3 step 2 (`d5efe2a`) shipped `interpret`/`run`/`run_accum`
on the six Run wrappers with the target monad implicit
(`M = self Run wrapper`, returns `A`). Before step 3 ships,
five decisions gate the implementation. The first three change
what code lands; the last two only change documentation.

| #   | Decision                                                                  | Recommendation              |
| --- | ------------------------------------------------------------------------- | --------------------------- |
| 1   | Schedule `runPure`-style row-narrowing pipeline + `extract`?              | **(1.A) Yes**               |
| 2   | Reshape step 2 to expose target M (PureScript-faithful symmetry)?         | **(2.C) No, asymmetric**    |
| 3   | Renumber Phase 3 if decision 1 widens scope?                              | **(3.A) Insert + renumber** |
| 4   | Add Phase 6+ deferred entries for `runCont` / `interpose` / algebraic-FO? | **(4.A) Defer all three**   |
| 5   | Update decisions.md to acknowledge implementation choices?                | **(5.A) Keep frozen**       |

If the recommended set is confirmed, the implementation
sequence in section 5 below resumes Phase 3 work.

##### 1. Background

###### 1.1 What step 2 shipped

Phase 3 step 2 (commit `d5efe2a`) shipped `interpret`,
`run`, and `run_accum` as inherent methods on each of the six
Run wrappers. The methods take a handler list and return `A`
directly:

```rust
pub fn interpret(self, handlers: H) -> A
```

The M target is implicit (M = self Run wrapper). The
interpreter loop is a `while`-loop with `prog =
handlers.dispatch(layer)` (assignment-driven, not bind-driven).
The DispatchHandlers trait has three cons-cell impls covering
the three Coyoneda variants (Coyoneda / RcCoyoneda /
ArcCoyoneda).

###### 1.2 PureScript Run reference shapes

PureScript Run ships eleven interpreter functions. The most
relevant
([`Run.purs:178-261`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs#L178-L261)):

```haskell
-- interpret/interpretRec are aliases of run/runRec respectively
interpret    :: Monad m    => (VariantF r ~> m) -> Run r a -> m a
interpretRec :: MonadRec m => (VariantF r ~> m) -> Run r a -> m a

-- The actual workhorses (mono-in-a, take a step function)
run          :: Monad m    => (VariantF r (Run r a) -> m (Run r a)) -> Run r a -> m a
runRec       :: MonadRec m => (VariantF r (Run r a) -> m (Run r a)) -> Run r a -> m a
runAccum     :: Monad m    => (s -> ... -> m (Tuple s ...)) -> s -> Run r a -> m a
runAccumRec  :: MonadRec m => (s -> ... -> m (Tuple s ...)) -> s -> Run r a -> m a

-- Loop bodies:
run k = loop where loop = resume (\a -> loop =<< k a) pure   -- bind-driven
runRec k = runFreeM (coerceM k) <<< unwrap                   -- tailRecM-driven
```

PureScript's `MonadRec` class
([`Control.Monad.Rec.Class:58-59`](https://github.com/purescript/purescript-tailrec/blob/master/src/Control/Monad/Rec/Class.purs#L58-L59))
defines `tailRecM` with a constant-stack class invariant.
fp-library's
[`MonadRec`](../../../fp-library/src/classes/monad_rec.rs)
mirrors this exactly. `IdentityBrand` / `OptionBrand` /
`ThunkBrand` / `VecBrand` already implement it.

The Rust precedent at
[`Free::fold_free<G: MonadRec>`](../../../fp-library/src/types/free.rs)
already implements the PureScript-faithful pattern for the
externally-targeted MonadRec form: takes a
[`NaturalTransformation<F, G>`](../../../fp-library/src/classes/natural_transformation.rs),
drives the loop via `G::tail_rec_m`, returns `G::Of<'_, A>`.

###### 1.3 Why the rec/non-rec distinction exists in PureScript

PureScript's `run` body recurses through `m`'s bind:
`loop =<< k a` builds nested `m`-bind frames in the host stack,
which blows for `m`s with non-stack-safe bind. `runRec`'s
`runFreeM` swaps for `tailRecM` (constant stack by class
invariant). The distinction is purely a stack-safety guarantee
on the target monad.

###### 1.4 Why Rust differs

fp-library's step 2 uses a `while`-loop, not bind-driven
recursion. The interpreter loop is structurally stack-safe
regardless of `M`. The PureScript rec/non-rec distinction does
not apply to step 2 as-shipped.

To match PureScript Run's `<M: Monad>` shape, step 2 would
need to expose M and route through `M::bind`. That requires
the Rust workarounds in 1.5 below.

###### 1.5 Rust workarounds for the bind-driven loop

PureScript's `loop =<< k a` recurses through `m`'s bind. In
Rust, the closure passed to `M::bind` must:

1. Be `Fn` (multi-callable) so non-deterministic `m`s like
   `Vec` can call the continuation multiple times.
2. Capture the handler list to recurse with it.
3. Not mutably alias the handler list across calls.

Three workarounds, all cheap in practice:

- **`Fn` closures (not `FnMut`).** Most user closures are
  naturally `Fn` already (mutation through `RefCell::borrow_mut`
  / `Mutex::lock` works from `&self`). Mostly free.
- **Handler list `Clone`.** Each step clones before passing to
  the recursive bind closure. Trivial closures are auto-`Clone`;
  closures capturing `Rc<RefCell<T>>` / `Arc<Mutex<T>>` are
  `Clone` (refcount bump). `HandlersCons` / `HandlersNil`
  already derive `Clone`. Per-step cost: one shallow clone
  (HCons cells × handler closure clones).
- **`'static` bound.** Already implicit for the Erased family;
  tightening for the Explicit family adds friction but isn't a
  hard wall.

With these workarounds, a bind-driven `<M: Monad>` interpreter
is implementable in stable Rust. They influence decision 2's
trade-offs.

###### 1.6 Heftia and the dual-row framing

[Heftia](https://github.com/sayo-hs/heftia) uses **one** effect
list (`es`) where each element has a `KnownOrder` (FO or HO).
The `FOEs es` constraint requires all members to be
first-order. fp-library's
[decisions.md](decisions.md) section 4.5 takes the IDEA
(separate FO vs HO dispatch) and ships a value-level dual row
(`Run<R, S, A>` with `Node<R, S> = First | Scoped`). So
fp-library's "heftia-inspired" framing is inspiration, not
direct port; we don't need heftia's mixed-order constraint
machinery, just `Node::First` vs `Node::Scoped` dispatch.
Heftia's primary interpreter mode is row-narrowing
(`interpretWith` strips the head effect from the row),
relevant to decision 1 below.

###### 1.7 The three orthogonal axes the question covers

The blocker started narrow (rec/non-rec for the M-targeted
family) but a close read of PureScript Run + heftia revealed
three orthogonal axes:

- **Axis 1: which interpreter functions ship.** PureScript Run
  ships eleven; current plan.md schedules six
  (`interpret`/`run`/`run_accum` × rec/non-rec). Gaps: `extract`,
  `runPure`/`runAccumPure` (pipeline row-narrowing),
  `runCont`/`runAccumCont` (continuation-passing handlers),
  `interpose` (heftia hook-without-removing). Decision 1 covers
  pipeline + `extract`; decision 4 covers the rest as deferred.
- **Axis 2: handler shape.** Heftia's `AlgHandler e m n a =
forall x. e x -> (x -> m a) -> n a` (algebraic, explicit
  continuation) vs PureScript Run's "handler returns m-wrapped
  next program" (implicit continuation). fp-library currently
  uses implicit-continuation for FO + algebraic-shape for HO
  (scoped constructors carry body/release closures inside). The
  asymmetry is reasonable: FO effects rarely need continuation
  control, HO effects often do. Decision 4 defers
  algebraic-shape for FO to Phase 6+.
- **Axis 3: rec/non-rec for the externally-targeted M family.**
  The original blocker question. Decision 2 covers it.

##### 2. Decisions

Listed in dependency order. Decisions 1, 2, 3 change what code
ships. Decisions 4, 5 only change documentation.

###### Decision 1: Schedule row-narrowing pipeline + `extract`?

**Question.** Should Phase 3 schedule
`extract(self) -> A` (empty-row pure extract) and
`interpret_with::<EBrand>(handler) -> Run<R_minus_E, S, A>`
(single-effect row-narrowing pipeline)? These are heftia's
primary mode + PureScript's `runPure`/`extract` analogs.

**Approaches.**

- **(1.A) Full widen.** Ship both `extract` and
  `interpret_with::<E>` as a new Phase 3 step.
- **(1.B) No widen.** Keep current six-method schedule; defer
  pipeline-style entirely to Phase 6+.
- **(1.C) Partial widen.** Ship `extract` only; defer
  `interpret_with::<E>` pipeline.

**Trade-offs.**

- **(1.A)**: Closes the heftia/PureScript ergonomic gap.
  Enables `prog.interpret_with(state).interpret_with(reader).extract()`.
  Phase 4 inherits parallel scoped-row-narrowing naturally.
  Brings plan.md back toward decisions.md section 3's full
  inventory. Cost: new method per wrapper + new
  `DispatchOneHandler` trait variant + tests.
- **(1.B)**: Cheapest now. Real ergonomic gap: users can only
  use one mega `handlers!{}` block with all effects. Future
  widening is non-breaking but current users live without it.
- **(1.C)**: `extract` alone is mostly useless (only matters
  AFTER all effects are stripped, which `interpret_with::<E>`
  enables). Not a meaningful intermediate.

**Recommendation: (1.A).** Pipeline-style is heftia's primary
mode; the absence is a real gap current users will hit.

###### Decision 2: Reshape step 2 to expose target M?

**Question.** When the MonadRec-target step ships, should
step 2 be reshaped to symmetric `<M: Monad>` (matching
PureScript Run 1:1)?

**Approaches.** Originally framed as options (i.a) / (i.b) /
(ii); the (2.A) / (2.B) / (2.C) labels below are equivalent.

- **(2.A) (i.a) Symmetric.** Reshape step 2 to take
  `<MBrand: Monad>` (bind-driven loop, requires the
  workarounds in 1.5). Add the rec family bounded
  `<MBrand: MonadRec>`. Mirrors PureScript Run's `run` /
  `runRec`. Breaking change to `d5efe2a`.
- **(2.B) (i.b) MonadRec uniform.** Reshape step 2 to take
  `<MBrand: MonadRec>`. Drop the `_rec` naming distinction.
  Six methods total instead of twelve (under axis 1 widening).
  Breaking change to `d5efe2a`.
- **(2.C) (ii) Asymmetric.** Step 2 stays as-shipped (M = self
  Run, returns A). The MonadRec step adds an
  externally-targeted `<MBrand: MonadRec>` family alongside.
  Three distinct primitives under axis 1 widening:
  `interpret -> A` (simple value extract),
  `interpret_with -> Run<R', S, A>` (pipeline),
  `interpret_rec<M> -> M::Of<A>` (MonadRec target).

**Trade-offs.**

| Dimension                  | (2.A) Symmetric                                     | (2.B) MonadRec uniform      | (2.C) Asymmetric                                            |
| -------------------------- | --------------------------------------------------- | --------------------------- | ----------------------------------------------------------- |
| PureScript fidelity        | 1:1                                                 | Diverges (no Monad form)    | Step 3+ faithful; step 2 documented as specialization       |
| decisions.md 4.3 alignment | Full                                                | Partial (one family)        | Step 3+ honors; step 2 honest specialization                |
| Implementation effort      | High: reshape + Clone bounds                        | Medium: reshape; one family | Low: step 2 unchanged; add new step                         |
| Breaking change to d5efe2a | Yes                                                 | Yes                         | No                                                          |
| Method count under (1.A)   | 18/wrapper × 6 = 108                                | 9/wrapper × 6 = 54          | 9/wrapper × 6 = 54                                          |
| Closure ergonomics         | Closures produce `M::Of<Run>`; verbose for Identity | Same                        | Step 2 closures unchanged; new family produces `M::Of<Run>` |
| Handler list bounds        | `Clone` everywhere                                  | `Clone` everywhere          | None on step 2; `Clone` on new family                       |
| Stack-safety story         | Explicit choice via bound                           | Implicit (always)           | Step 2 structurally safe; new family `tail_rec_m`-driven    |
| Conceptual redundancy      | Two methods for similar loops                       | None                        | None: methods functionally distinct                         |

**Recommendation: (2.C)** under decision (1.A). Three
genuinely-distinct primitives map to three distinct user use
cases without redundancy. (2.A) would multiply rec/non-rec
across all three families (108 methods); marginal value over
(2.C) is small for the cost. Decision 4.3's "Monad m" family
gets deferred to Phase 6+ as `interpret_with<M: Monad>` (this
deferred entry is already drafted).

###### Decision 3: Phase 3 step ordering after axis 1 widening?

**Question.** If decision 1.A ships, Phase 3 gets a new step.
How to order?

**Approaches.**

- **(3.A) Insert + renumber.** Steps 1-2 done; new step 3 ships
  row-narrowing pipeline; step 4 is the (was step 3) MonadRec
  family; renumber 5-7 (was 4-6).
- **(3.B) Insert without renumbering.** Add a step "2.5" or
  "3a" alongside the existing.
- **(3.C) Combine.** Step 3 becomes "row-narrowing + MonadRec
  extraction" as one combined step.

**Trade-offs.**

- **(3.A) Renumber**: Clean canonical ordering. Reference-sweep
  cost in plan.md / deviations.md but bounded.
- **(3.B) Step 2.5**: Avoids renumbering. Non-canonical;
  confusing to future readers.
- **(3.C) Combine**: One commit ships two substantively
  different things. Violates "one logical thing per step"
  convention.

**Recommendation: (3.A)** if decision 1.A ships. **N/A** if
decision 1.B (no widen).

###### Decision 4: Phase 6+ deferred entries for axis-1/2 items not in scope?

**Question.** Do `runCont` / `runAccumCont` (continuation-passing,
axis 1), `interpose` (heftia hook-without-removing, axis 1), and
algebraic-shape FO handlers (axis 2) get explicit Phase 6+
deferred-items entries (vs being unscheduled)?

**Approaches.**

- **(4.A) Defer all three.** Add Phase 6+ entries for each,
  naming use case, deferral cost, and trigger.
- **(4.B) Mixed.** Defer some, mark others as out-of-scope.
- **(4.C) Skip Phase 6+ entries.** Rely on decisions.md
  section 3 inventory.

**Trade-offs.**

- **(4.A) Defer all**: Most discoverable. Matches existing
  Phase 6+ pattern (7 existing entries). Future implementers
  see explicit trigger conditions.
- **(4.B) Mixed**: Inconsistent (some get triggers, others
  implicit "if needed").
- **(4.C) Skip**: decisions.md section 3 is an inventory, not
  a deferral plan; not actionable for future implementers.

**Recommendation: (4.A) Defer all three** with explicit
trigger conditions per the existing pattern.

###### Decision 5: Update decisions.md?

**Question.** Should decisions.md be edited to reflect the
implementation choices (especially axis 3 deviation if 2.C
ships)?

**Approaches.**

- **(5.A) Keep decisions.md frozen.** All implementation
  choices recorded in plan.md / deviations.md / resolutions.md
  per the existing convention.
- **(5.B) Refine 4.3.** Add a short note to section 4.3
  acknowledging axis 3 deviation.
- **(5.C) Add 4.7 for axis 2.** Make the implicit
  "PureScript-Run shape for FO + algebraic for HO" decision
  explicit.

**Trade-offs.**

- **(5.A) Frozen**: Honors CLAUDE.md / AGENTS.md convention
  ("decisions are frozen"). Original design rationale
  preserved.
- **(5.B) Refine 4.3**: Visible at the design-rationale
  layer. Violates "frozen decisions" convention.
- **(5.C) Add 4.7**: Completeness. Same convention violation.

**Recommendation: (5.A) Keep frozen.** Per existing
convention. Resolutions.md gets a top-level entry once this
blocker resolves; deviations.md gets per-step entries when
each step ships.

##### 3. Phase 4 implications

The decision set affects Phase 4's interpreter family work:

- **Decision 1 (axis 1)**: If (1.A), Phase 4 inherits a parallel
  `interpret_scoped_with::<E>` row-narrowing pattern for scoped
  effects. Same dispatch loop, parallel methods. If (1.B),
  Phase 4 stays "all-handlers-at-once" for scoped, matching
  step 2's shape.
- **Decision 2 (axis 3)**: If (2.A), scoped-effect interpreters
  also get rec/non-rec siblings (method count doubles in
  Phase 4 alongside Phase 3). If (2.C), scoped interpreters
  get a single MonadRec-bound externally-targeted form;
  minimal addition.
- **Decision 4 (axis 2)**: If algebraic-shape FO handlers are
  deferred (recommended (4.A)), Phase 4's HO scoped handlers
  remain de-facto algebraic (their constructors carry body /
  release closures), and the FO/HO asymmetry persists. No
  Phase 4 macro reshape needed.

##### 4. Cross-references

- [decisions.md](decisions.md) section 3 ("Entirely missing")
  lists the full PureScript Run combinator family as needing
  to be built; current plan.md schedule is narrower.
- [decisions.md](decisions.md) section 4.3 ("Ship both
  interpreter families") commits to the rec/non-rec split.
- [decisions.md](decisions.md) section 4.5 mentions `runPure`
  ("Run-to-bare-value") as a free-function companion; under
  (1.A) we'd ship it as scheduled methods instead.
- [`fp-library/src/types/free.rs`](../../../fp-library/src/types/free.rs)'s
  `Free::fold_free<G: MonadRec>` is the existing Rust
  precedent for the externally-targeted MonadRec form.
- [`fp-library/src/classes/natural_transformation.rs`](../../../fp-library/src/classes/natural_transformation.rs)
  is the rank-2 polymorphic abstraction; complementary to the
  handler-list path. Already drafted as a Phase 6+ deferred
  `interpret_nt` companion entry.
- [PureScript Run](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
  source.
- [PureScript MonadRec](https://github.com/purescript/purescript-tailrec/blob/master/src/Control/Monad/Rec/Class.purs)
  source.
- [Heftia](https://github.com/sayo-hs/heftia) interpreter
  machinery.
- Phase 3 step 2 commit `d5efe2a`: the API that (2.A) / (2.B)
  reshapes would break and that (2.C) preserves.

##### 5. What happens if the recommended set is confirmed

Action sequence to unblock:

1. **Doc updates land first** (single `docs(plan):` commit):
   - Renumber Phase 3 steps in the implementation phasing
     section: current step 3 (`interpretRec` family) becomes
     step 4; current step 4 (effects) becomes step 5; etc.
     New step 3 = `runPure`-style row-narrowing pipeline +
     `extract`.
   - Add three Phase 6+ deferred-items entries:
     `runCont`/`runAccumCont` family (axis 1
     continuation-passing), `interpose` (heftia
     hook-without-removing, axis 1), algebraic handler shape
     for FO (axis 2). Trigger conditions per the existing
     Phase 6+ pattern.
   - Decisions.md unchanged.
2. **Implementation resumes** with the renumbered Phase 3
   sequence:
   - Step 3 (new): pipeline-style row-narrowing.
     `interpret_with::<EBrand>(handler) -> Run<R_minus_E, S, A>`
     and `extract(self) -> A` per wrapper. New
     `DispatchOneHandler` trait variant.
   - Step 4 (was step 3): MonadRec-target
     `interpret_rec`/`run_rec`/`run_accum_rec` per wrapper.
   - Steps 5-7 (was 4-6): standard first-order effects;
     `define_effect!` macro; `compile_fail` UI tests.
3. **Resolution** once steps 3 and 4 ship: move this active
   blocker entry to resolutions.md as a top-level dated
   entry; replace it with a one-line summary plus an anchor
   link in this `Active blockers` section. Phase 2 step 9
   resolution (resolutions.md) is the precedent.

The Phase 2 step 9 under-specification (logged 2026-04-28) is
resolved; full investigation, alternatives, and resolution moved
to [resolutions.md](resolutions.md#resolved-2026-04-28-phase-2-step-9-scope-is-under-specified).
The one-line summary is in the
[Resolved blockers (summary)](#resolved-blockers-summary) section
below.

#### Previously resolved blockers

The three blockers that surfaced 2026-04-27 while preparing
Phase 2 step 4b have all been resolved as part of the step 4b
commit:

- Brand-level type-class coverage gap on the Explicit Run
  brands: shipped achievable subset, documented gaps; see
  [resolutions.md](resolutions.md#resolved-2026-04-27-brand-level-type-class-coverage-gap-on-the-explicit-run-brands).
- Row-brand `RefFunctor` and `Extract` cascade impls land in
  step 4b: see
  [resolutions.md](resolutions.md#resolved-2026-04-27-row-brand-reffunctor-and-extract-cascade-impls-land-in-step-4b).
- Re-export pattern for the effects subsystem types follows
  the optics A+B hybrid: see
  [resolutions.md](resolutions.md#resolved-2026-04-27-re-export-pattern-for-the-effects-subsystem-types-follows-the-optics-ab-hybrid).

### Procedure for new blockers

If a load-bearing question surfaces during implementation:

1. Add an `### Active blocker (date): <summary>` subsection
   under `### Active blockers` above and pause work.
2. When the blocker resolves, move the entry verbatim (or with
   added resolution detail) to [resolutions.md](resolutions.md)
   as a new top-level entry, dated.
3. Replace the active-blocker subsection here with a one-line
   pointer if useful for cross-referencing, or remove it.

### Resolved blockers (summary)

For full investigation, alternatives, and rationale on each
resolved blocker, see [resolutions.md](resolutions.md). One-line
summaries:

- [Resolved (2026-04-28 implementation expansion): step 9 SendFunctor cascade prerequisites for Arc family](resolutions.md#resolved-2026-04-28-implementation-expansion-step-9-sendfunctor-cascade-prerequisites-for-arc-family)
  -- discovered while implementing the original 2026-04-28
  resolution: `ArcRun::lift` and `ArcRunExplicit::lift` cannot
  use the same `Coyoneda::lift` chain as the other four wrappers
  because `Coyoneda` isn't `Send + Sync` and the Send-aware
  sibling `ArcCoyonedaBrand` doesn't implement `Functor` (a
  deliberate fp-library design choice; the `Functor` trait's
  `map` signature lacks `Send + Sync` bounds on closures).
  Resolution: expand step 9 with sub-steps 9a-9g landing the
  `SendFunctor` cascade prerequisites (replace `F: Functor` with
  `F: SendFunctor` on `ArcFree`/`ArcFreeExplicit`; add missing
  `SendFunctor` impls on the row-cascade brands; expand
  brand-level coverage on `ArcFreeExplicitBrand` /
  `ArcRunExplicitBrand`); then complete `*Run::lift` for all six
  wrappers in 9h; add `SendRefFunctor` on `ArcRunExplicitBrand`
  via inherent-method delegation in 9i.
- [Resolved (2026-04-28): Phase 2 step 9 scope is under-specified](resolutions.md#resolved-2026-04-28-phase-2-step-9-scope-is-under-specified)
  -- generic combinator interpretation locked in, named `lift`
  to match PureScript Run's
  [`Run.lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs);
  inherent associated function on each of the six Run wrappers
  mirroring `*Run::send`'s shape; takes the raw effect (not
  pre-lifted Coyoneda) and does Coyoneda lift -> row inject ->
  `Node::First` -> `*Run::send` inline; falls back to a free
  `lift_node` helper for `ArcRun::lift` only if HRTB-poisoning
  recurs. Followed up by the implementation-expansion entry
  above when the Arc-family Coyoneda/Functor conflict surfaced.
- [Resolved (2026-04-27): `*Run::send` takes a `Node`-projection value to sidestep GAT-normalization poisoning under `ArcFree`'s HRTB](resolutions.md#resolved-2026-04-27-runsend-takes-a-node-projection-value-to-sidestep-gat-normalization-poisoning-under-arcfrees-hrtb)
  -- discovered while implementing `ArcRun::send`: the HRTB at
  `ArcFree`'s struct level poisons `<NodeBrand as Kind>::Of<...>`
  normalization in any scope mentioning it. Workaround: pass
  the `Node`-projection value as a parameter rather than
  constructing it inside the HRTB scope. Applied symmetrically
  to all six Run wrappers' `send` for API uniformity.
- [Resolved (2026-04-27): brand-level type-class coverage gap on the Explicit Run brands](resolutions.md#resolved-2026-04-27-brand-level-type-class-coverage-gap-on-the-explicit-run-brands)
  -- ship `Functor / Pointed / Semimonad` plus the by-reference
  equivalents for `RunExplicitBrand`; `Pointed` plus by-reference
  for `RcRunExplicitBrand`; `SendPointed` only for
  `ArcRunExplicitBrand`. `Monad` / `RefMonad` / `SendMonad` and
  the `SendRef`-family hierarchy are unreachable through
  brand-level delegation; inherent `bind` and `map` cover the
  by-value monadic surface.
- [Resolved (2026-04-27): row-brand `RefFunctor` and `Extract` cascade impls land in step 4b](resolutions.md#resolved-2026-04-27-row-brand-reffunctor-and-extract-cascade-impls-land-in-step-4b)
  -- add `RefFunctor` and `Extract` impls to `CNilBrand`,
  `CoproductBrand<H, T>`, and `NodeBrand<R, S>`, plus a `Clone`
  impl on `Node`. Required by the Run-Explicit brand
  Ref-hierarchy delegation and by `Rc`/`Arc`-Free's `evaluate`
  fallback.
- [Resolved (2026-04-27): re-export pattern for the effects subsystem types follows the optics A+B hybrid](resolutions.md#resolved-2026-04-27-re-export-pattern-for-the-effects-subsystem-types-follows-the-optics-ab-hybrid)
  -- six Run wrapper headline types at top-level
  (`crate::types::*`); same six plus `Node` and `VariantF` at
  subsystem-scoped (`crate::types::effects::*`). Mirrors the
  optics precedent.
- [Resolved (2026-04-27): introduce `WrapDrop` trait for Free's struct-level Drop concern](resolutions.md#resolved-2026-04-27-introduce-wrapdrop-trait-for-frees-struct-level-drop-concern)
  -- replace Free's struct-level `Extract` bound with a new
  `WrapDrop` trait that decouples Drop's structural cleanup
  from `Extract`'s semantic interpretation. Two-commit migration
  before Phase 2 step 4 resumes.
- [Resolved (2026-04-26): brand-level dispatch for the multi-shot Explicit Free family lands on the by-reference hierarchy](resolutions.md#resolved-2026-04-26-brand-level-dispatch-for-the-multi-shot-explicit-free-family-lands-on-the-by-reference-hierarchy)
  -- `RcFreeExplicitBrand` and `ArcFreeExplicitBrand` get
  `Pointed`/`SendPointed` on the by-value side and full Ref/SendRef
  hierarchies; remaining by-value operations ship as inherent
  methods.
- [Resolved earlier: Erased / Explicit dispatch split for the Free family](resolutions.md#resolved-earlier-erased--explicit-dispatch-split-for-the-free-family)
  -- Erased family (`Free`, `RcFree`, `ArcFree`) is
  inherent-method only; Explicit family (`FreeExplicit`,
  `RcFreeExplicit`, `ArcFreeExplicit`) is Brand-dispatched.
- [Design-phase blockers (resolved in decisions.md)](resolutions.md#design-phase-blockers-resolved-in-decisionsmd)
  -- pointer aggregating decisions.md sections 4 and 9.

<!-- The full problem statement, investigation, resolution,
migration plan, and alternatives for the WrapDrop blocker live
in resolutions.md per the link above. The phasing-side checklist
lives in "Phase 1 follow-up: WrapDrop migration" below. -->

<!-- old content removed; see resolutions.md -->

## Deviations

Per-step deviations from the original plan text (where the
shipped code or design diverged from what the step description
said) are logged in [deviations.md](deviations.md), grouped by
phase and step. New deviations are appended there as steps land.

## Implementation protocol

After completing each step within a phase:

1. Run verification: `just fmt`, `just check`, `just clippy`,
   `just deny`, `just doc`, `just test` (or `just verify` which
   runs all six in order).
2. If verification passes, update `Current progress`, `Open
questions, issues and blockers`, and `Deviations` sections at
   the top of this plan to reflect the current state.
3. **Trim `Current progress` if it has grown.** The section
   has two subsections:
   - **"Most recent steps (rolling detail)"** holds the latest
     ~3 step narratives in detail. Each new step's narrative
     lands at the top of this subsection.
   - **"Earlier completed steps (commit log)"** holds older
     entries as one-line bullets:
     `- ``<commit-hash>`` (step <N>): <one-line summary>.`
     with cross-references to deviations.md / resolutions.md /
     commit messages where the deeper narrative lives.

   When the rolling-detail subsection grows past ~3 entries,
   demote the oldest narrative to a bullet in the commit-log
   subsection. Before demoting, verify the narrative's
   load-bearing context lives somewhere persistent: design
   choices in [deviations.md](deviations.md), load-bearing
   investigations in [resolutions.md](resolutions.md),
   "what changed" in the commit message. If a piece of
   context lives only in plan.md, move it to the right home
   first.

   Goal: keep `Current progress` under ~250 lines so a new
   agent reading the plan reaches actionable content quickly.
   Detailed history stays accessible via `git show <hash>`,
   deviations.md, and resolutions.md.

   Demotion can ride in the same commit as the new step or
   land separately as a `docs(plan): trim Current progress`
   follow-up; pick whichever keeps the new step's diff clean.
   For larger structural rearrangements (e.g., the multi-step
   trim that landed `97b7e73`), a dedicated commit is
   preferable.

4. Commit the step (including the plan updates and any inline
   trim).

---

Port `purescript-run`'s extensible algebraic effects to
`fp-library`, delivering Rust `Run` types that support
row-polymorphic first-order effects and heftia-style scoped
effects, with macro ergonomics for common cases and a six-variant
`Free` substrate covering single-shot, multi-shot, thread-safe,
non-`'static` payload, and Brand-dispatched-vs-inherent-method
combinations via the Erased/Explicit dispatch split.

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
brand turbofish; multi-shot effects require choosing `RcRun` or
`ArcRun` rather than the default `Run`; typeclass-generic dispatch
requires the corresponding Explicit Run variant).

User surface after this plan, fast-path inherent-method version:

```rust
// Declare a row of effects via the macro:
type AppEffects = effects![Reader<Env>, State<Counter>, Logger];

// Build a program with the im_do! macro (inherent monadic do,
// inherent-method-based, O(1) bind, no Brand dispatch):
fn run_program() -> Run<AppEffects, NoScoped, String> {
    im_do! {
        cfg <- ask::<Env>();
        n <- get::<Counter>();
        log(format!("config = {cfg:?}, counter = {n}"));
        pure(format!("got {n}"))
    }
}

// Compose handlers as a pipeline that narrows the row at each step:
let result: String = run_program()
    .handle(run_reader(env))
    .handle(run_state(0))
    .handle(run_logger())
    .extract();
```

For Brand-dispatched typeclass-generic code (or programs with
non-`'static` payloads), use the corresponding Explicit variant.
The single-shot single-thread variant `RunExplicit` (built on
`FreeExplicit`) keeps full by-value brand coverage and is the
ergonomic default:

```rust
fn run_program_explicit<'a>() -> RunExplicit<'a, AppEffects, NoScoped, String> {
    m_do!(RunExplicitBrand {
        cfg <- ask::<Env>();
        n <- get::<Counter>();
        pure(format!("got {n}"))
    })
}
```

The multi-shot variants `RcRunExplicit` / `ArcRunExplicit` get
brand dispatch via the by-reference hierarchy (`RefFunctor` /
`RefSemimonad` / `RefMonad` and their `SendRef*` parallels),
matching `Lazy`'s precedent for the same constraint. The existing
`m_do!` / `a_do!` macros support a `ref` qualifier
(`m_do!(ref Brand { ... })`) that routes through
`RefSemimonad::ref_bind`; closures take `&A`:

```rust
fn run_program_rc_explicit<'a>() -> RcRunExplicit<'a, AppEffects, NoScoped, String> {
    m_do!(ref RcRunExplicitBrand {
        cfg <- ask::<Env>();          // cfg: &Env
        n <- get::<Counter>();         // n: &Counter
        pure(format!("got {n}"))
    })
}
```

For inherent-method calls on multi-shot Explicit Run programs
(e.g., when `A: Clone` is satisfied and consuming continuations
are preferred), the by-value `bind` / `map` ship as inherent
methods on `RcRunExplicit` / `ArcRunExplicit` directly, with their
natural `Clone` bounds, mirroring the
[`RcCoyoneda`/`ArcCoyoneda` precedent](../../../fp-library/docs/limitations-and-workarounds.md).

Convert between Erased and Explicit on demand:
`run_program().into_explicit()` walks the structure once and
returns the corresponding Explicit Run of the same program,
suitable for handing into typeclass-generic consumers.

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
- **Free family (decisions §4.4):** six variants in two rows.
  Erased family (`Free`, `RcFree`, `ArcFree`) is inherent-method
  only with O(1) bind via `dyn Any` erasure plus CatList; pins
  `A: 'static`. Explicit family (`FreeExplicit<'a, ...>`,
  `RcFreeExplicit<'a, ...>`, `ArcFreeExplicit<'a, ...>`) carries
  Brand dispatch with O(N) bind via concrete recursive enum;
  supports `A: 'a`. The Erased/Explicit split is the dispatch
  story: typeclass-generic code uses the Explicit row, fast-path
  code uses the Erased row, and `into_explicit()` converts between
  them when needed. The `ArcFreeExplicitBrand` `Functor` impl
  lands via the new `SendFunctor` trait family (Phase 1 step 6).
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

| POC                                                                                                                                                                         | Findings                                                                                                                                                                                                                                                                                                                                                   |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `poc-effect-row/tests/feasibility.rs` (deleted in 10b; migrated to [`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs)) | 17 tests covering workaround 1 (lexical-sort macro) plus workaround 3 (`CoproductSubsetter` fallback), generic-effect handling, lifetime parameters, 5- and 7-effect rows for trait-inference scaling, plus `tstr_crates` Phase 2 refinement (3 tests showing content-addressed naming + `tstr::cmp` compile-time ordering). All pass on stable Rust 1.94. |
| `poc-effect-row/tests/coyoneda.rs` (deleted in 10b; migrated to [`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs))    | 8 tests validating static-via-Coyoneda end-to-end: `effects_coyo!` macro emits Coyoneda-wrapped Coproducts canonically; `Coyoneda<F, A>` is `Functor` for any `F`; `Coproduct<H, T>` implements `Functor` via recursive trait dispatch with no specialization or runtime dictionary; row canonicalises across input orderings under wrapping.              |
| [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs)                                                                                     | 6 tests validating `FreeExplicit<'a, F, A>` integrates with the Brand-and-Kind machinery, supports non-`'static` payloads, supports two-effect Run-shaped composition. One `#[ignore]`d test documents that naive `Drop` overflows on deep chains; the iterative custom `Drop` ships in Phase 1.                                                           |
| [fp-library/benches/benchmarks/free_explicit.rs](../../../fp-library/benches/benchmarks/free_explicit.rs)                                                                   | Criterion bench at depths 10 / 100 / 1000 / 10000 confirming `FreeExplicit`'s per-node cost is approximately 27ns in the linear regime. The Phase-1 baseline for measuring `RcFree` / `ArcFree` regressions.                                                                                                                                               |

The POC code (the `effects!` / `effects_coyo!` macros, the stub
Coyoneda) migrates into production during Phase 2 and Phase 3; the
POC repos remain as reference until then and are deleted once the
production tests cover the same surface.

## Key decisions

The full decision rationale is in [decisions.md](decisions.md).
Quick reference table:

| ID        | Decision                                                                                                                | Rationale (one-line)                                                                                                                                  |
| --------- | ----------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| 4.1       | Option 4 hybrid (macro + nested Coproduct) with corophage-style `'a` per effect                                         | Most production-credible reference (corophage) and best stable-Rust ergonomics                                                                        |
| 4.1       | Workaround 1 (macro canonicalisation) primary; workaround 3 (`CoproductSubsetter`) fallback                             | Macro pays the sort cost once at row construction; Subsetter handles hand-written rows                                                                |
| 4.1       | tstr_crates content-addressed naming as Phase 2 refinement                                                              | Stable type-level identity across import paths; the only credible stable-Rust improvement                                                             |
| 4.2       | Static option via `Coyoneda` per effect                                                                                 | Each row variant is trivially a Functor; section 5.2 commits to Coyoneda anyway                                                                       |
| 4.3       | Ship both `interpret` and `interpretRec` families                                                                       | Documentation parity with PureScript Run; few-percent runtime cost is small                                                                           |
| 4.4       | Six-variant Free: `Free`, `RcFree`, `ArcFree` (Erased) + `FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit` (Explicit) | Erased family is inherent-method-only with O(1) bind; Explicit family is Brand-dispatched with O(N) bind; Erased/Explicit split is the dispatch story |
| 4.4       | `SendFunctor` / `SendPointed` / `SendSemimonad` / `SendMonad` trait family for `ArcFreeExplicitBrand`                   | By-value parallel of existing `SendRef*` family; closes the same gap that today prevents `ArcCoyonedaBrand` from implementing `Functor`               |
| 4.5       | Heftia dual-row for scoped effects                                                                                      | Cleanest higher-order effect encoding surveyed; preserves first-class programs                                                                        |
| 4.5       | `'a` lifetime parameter on every scoped-effect constructor from day one                                                 | Avoids breaking-change retrofit when `FreeExplicit` use cases want non-`'static` actions                                                              |
| 4.5       | Fixed `Run<R, A>` interpreter continuation (no associated type)                                                         | Matches every Haskell library surveyed; associated type deferred until use case forces it                                                             |
| 4.5       | Coproduct-of-constructors for user-defined scoped effects                                                               | Mirrors the first-order row's structure; preserves first-class-programs property                                                                      |
| 4.6       | `handlers!{...}` macro DSL primary; builder pattern fallback                                                            | Same shape as section 4.1's macro + mechanical-fallback hybrid                                                                                        |
| 9.3 / 9.4 | Sync interpreters in v1; async (and async IO) via `Future` as a `MonadRec` target in Phase 3                            | "User picks the target monad" -- single mechanism, no parallel `AsyncRun` family                                                                      |
| 9.8       | All effects-related macros live in `fp-macros`; split off a separate crate only if needed                               | One crate, one release cadence, one place to coordinate macro semantics                                                                               |
| 9.9       | TalkF + DinnerF integration test from `purescript-run` as the headline Phase 4 milestone                                | Real-world reference; validates the port behaves like `purescript-run` for a worked example                                                           |

## Integration surface

### Will change

| Component                                                                                         | Change                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| ------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fp-library/src/types/free.rs`                                                                    | Existing `Free<F, A>` keeps its current shape; inherent-method only (no Brand). Minor adjustments if integration with `Run` requires.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `fp-library/src/types/free_explicit.rs`                                                           | **New module (Phase 1 step 1).** Promote `FreeExplicit<'a, F, A>` from POC, add iterative custom `Drop`, add full by-value `Functor` / `Pointed` / `Semimonad` / `Monad` impls plus full `RefFunctor` / `RefPointed` / `RefSemimonad` / `RefMonad` impls (Phase 1 step 7). The naive recursive enum has no Clone bound on bind, so both hierarchies land cleanly.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `fp-library/src/types/rc_free.rs`                                                                 | **New module (Phase 1 step 2).** `RcFree<F, A>` following the `Free` template with `FnBrand<RcBrand>`-shaped continuations (i.e., `Rc<dyn 'a + Fn(B) -> RcFree<F, A>>` via the unified [`FnBrand`](../../../fp-library/src/types/fn_brand.rs) abstraction). Multi-shot effects (`Choose`, `Amb`). Inherent-method only; no `RcFreeBrand`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `fp-library/src/types/arc_free.rs`                                                                | **New module (Phase 1 step 3).** `ArcFree<F, A>` following the `ArcCoyoneda` template with `FnBrand<ArcBrand>`-shaped continuations (i.e., `Arc<dyn 'a + Fn(B) -> ArcFree<F, A> + Send + Sync>` via [`FnBrand`](../../../fp-library/src/types/fn_brand.rs) parameterised by [`ArcBrand`](../../../fp-library/src/brands.rs#L43)) and the `Send`/`Sync` Kind-trait pattern via `SendRefCountedPointer`. Inherent-method only; no `ArcFreeBrand`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `fp-library/src/types/rc_free_explicit.rs`                                                        | **New module (Phase 1 step 4).** `RcFreeExplicit<'a, F, A>` extending `FreeExplicit`'s concrete recursive enum with an outer `Rc<RcFreeExplicitInner>` wrapper plus `Rc<dyn Fn>` continuations. O(N) bind, multi-shot, `A: 'a`, Brand-compatible (`RcFreeExplicitBrand<F>` registered in step 4). Custom iterative `Drop` via `Extract` + `Rc::try_unwrap`. Brand-level dispatch in step 7: `Pointed` only on by-value (`pure` has no Clone bound); full `RefFunctor` / `RefSemimonad` / `RefMonad` plus supporting Ref traits per [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md). By-value `bind` / `map` ship as inherent methods with their natural `A: Clone` bounds.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `fp-library/src/types/arc_free_explicit.rs`                                                       | **New module (Phase 1 step 5).** `ArcFreeExplicit<'a, F, A>` extending `RcFreeExplicit`'s shape with `Arc<...>` wrapping and `Arc<dyn Fn + Send + Sync>` continuations. Same `Kind<Of<'a, A>: Send + Sync>` associated-type-bound trick as `ArcFree`. Brand-compatible (`ArcFreeExplicitBrand<F>` registered in step 5). Brand-level dispatch in step 7: `SendPointed` (added by step 6) on by-value; full `SendRefFunctor` / `SendRefSemimonad` / `SendRefMonad` plus supporting `SendRef*` traits. By-value `bind` / `map` ship as inherent methods with `A: Clone + Send + Sync` bounds.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `fp-library/src/classes/send_functor.rs`, `send_pointed.rs`, `send_semimonad.rs`, `send_monad.rs` | **New trait files (Phase 1 step 6).** By-value parallels of the existing `send_ref_*` family with `Send + Sync` bounds on the closure parameters and on values entering the structure (`SendPointed::pure(a: A)` requires `A: Send + Sync`). `SendPointed` lands as the brand-level `pure` for `ArcCoyonedaBrand` (closing the open gap module docs flag) and `ArcFreeExplicitBrand`. `SendFunctor` / `SendSemimonad` / `SendMonad` carry trait impls for `ArcCoyonedaBrand` (whose by-value path has no Clone bound). The multi-shot Explicit Free family does not implement `SendFunctor` / `SendSemimonad` / `SendMonad` at the brand level (Clone bound on bind makes them unexpressible) and instead routes brand-level dispatch through the existing `SendRef*` hierarchy in step 7.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `fp-library/src/types/effects.rs`                                                                 | **New module (Phase 2 step 4).** Six concrete Run types: `Run<R, S, A>`, `RcRun<R, S, A>`, `ArcRun<R, S, A>` (Erased family, inherent-method only) and `RunExplicit<'a, R, S, A>` (Explicit, full by-value brand-dispatched), `RcRunExplicit<'a, R, S, A>`, `ArcRunExplicit<'a, R, S, A>` (Explicit, Pointed/SendPointed by-value plus full Ref/SendRef brand coverage). `Node<R, S>` enum dispatching first-order vs scoped layers. `into_explicit()` / `from_erased()` conversion API between paired Erased and Explicit Run variants.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `fp-library/src/types/effects/coproduct.rs`                                                       | **New submodule.** Brand-aware adapter layer over `frunk_core::coproduct::{Coproduct, CNil, CoproductSubsetter}`: newtype wrappers, `impl` blocks bridging `frunk_core`'s Plucker / Sculptor / Embedder traits to the project's `Brand` system. Direct (non-newtyped) `Functor` impls on `frunk_core::Coproduct<H, T>` live here too (own-trait + foreign-type, orphan-permitted).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `fp-library/src/types/effects/variant_f.rs`                                                       | **New submodule.** `VariantF<Effects>` first-order coproduct with Coyoneda-wrapped variants and recursive `Functor` impl on `Coproduct<H, T>` (delegating to the adapter in `coproduct.rs`).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `fp-library/src/types/effects/scoped.rs`                                                          | **New submodule.** `ScopedCoproduct<ScopedEffects>` higher-order coproduct, standard scoped constructors. `Catch<'a, E>` and `Span<'a, Tag>` ship Val-only. `Local` ships in Val and Ref flavours (`Local<'a, E>` + `RefLocal<'a, E>`); `Bracket` ships in Val and Ref flavours (`Bracket<'a, A, B>` + `RefBracket<'a, P, A, B>`) per [decisions.md](decisions.md) section 4.5 sub-decisions. `Mask` is deferred to a future revision per the same section.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `fp-library/src/dispatch/run/`                                                                    | **New submodule.** Closure-driven Val/Ref dispatch for `bracket` and `local` smart constructors, mirroring the existing layout described in [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md). Files: `bracket.rs` (`BracketDispatch` trait + `Val` impl + `Ref<P>` impls per pointer brand + `bracket` inference wrapper + `explicit::bracket` brand-explicit wrapper); `local.rs` (`LocalDispatch` trait + `Val` and `Ref` impls + `local` inference wrapper + `explicit::local` wrapper). Re-exported from `fp-library/src/functions.rs` alongside `map`, `bind`, etc.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `fp-library/src/types/effects/handler.rs`                                                         | **New submodule.** Handler-pipeline machinery (`Run::handle`), natural-transformation type, `peel` / `send` / `extract`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `fp-library/src/types/effects/interpreter.rs`                                                     | **New submodule.** `interpret` / `run` / `runAccum` (recursive) and `interpretRec` / `runRec` / `runAccumRec` (`MonadRec`-targeted) families.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `fp-macros/src/effects/`                                                                          | **New module tree.** `effects!`, `effects_coyo!`, `handlers!`, `define_effect!`, `define_scoped_effect!`, `scoped_effects!`, and `im_do!` proc-macros (with `ia_do!` planned as a future companion). `im_do!` (Inherent Monadic do) is the inherent-method-based monadic do-notation that desugars to chained `.bind(...)` / `.ref_bind(...)` method calls and works uniformly across all six Run wrappers (the Erased family `Run`/`RcRun`/`ArcRun`, plus the Explicit family `RunExplicit`/`RcRunExplicit`/`ArcRunExplicit` for cases where brand-level dispatch isn't reachable, e.g., canonical Coyoneda-headed rows on `RcRunExplicit` or any use of `ArcRunExplicit`'s by-reference path). The Explicit Run family also supports the existing brand-dispatched `m_do!` / `a_do!` over `RunExplicitBrand` (full by-value brand coverage) and the `ref` qualifier (`m_do!(ref ...)` / `a_do!(ref ...)`) over `RcRunExplicitBrand` for synthetic rows whose row brand satisfies `RefFunctor`; canonical Coyoneda-headed rows route through `im_do!(ref RcRunExplicit { ... })` instead. `ia_do!` (Inherent Applicative do) is the inherent-method-based applicative companion to `im_do!`, deferred to a future phase but named in advance to lock in the convention. Migration from POC for the row-construction macros. |
| `fp-library/src/brands.rs`                                                                        | Add brands for the Brand-dispatched (Explicit) types only: `FreeExplicitBrand<F>`, `RcFreeExplicitBrand<F>`, `ArcFreeExplicitBrand<F>`, `RunExplicitBrand<R, S>`, `RcRunExplicitBrand<R, S>`, `ArcRunExplicitBrand<R, S>`. The Erased family (`Free`, `RcFree`, `ArcFree`, `Run`, `RcRun`, `ArcRun`) does NOT get brands; those types remain inherent-method only. `*FreeExplicitBrand<F>` are single-parameter `PhantomData<F>` structs mirroring [`CoyonedaBrand<F>`](../../../fp-library/src/brands.rs#L155); the three `*RunExplicitBrand<R, S>` variants are two-parameter `PhantomData<(R, S)>` structs mirroring [`CoyonedaExplicitBrand<F, B>`](../../../fp-library/src/brands.rs#L171). For all of them, `'static` bounds live on impls (so the row types `R`, `S` and the payload `'a`, `A` stay out of the brand identity and appear only in `Of<'a, A>` at instantiation, keeping brand types `'static`-clean while admitting non-`'static` payloads via the Explicit family).                                                                                                                                                                                                                                                                                                                                   |
| `fp-library/tests/run_*.rs`                                                                       | **New test files.** Per-Free-variant unit tests for all six variants (Phase 1 step 9, including `compile_fail` cases for Brand-dispatched calls against Erased variants and missing `Send + Sync` on `ArcFreeExplicit::bind` closures), row-canonicalisation regression tests migrated from `poc-effect-row/` (Phase 2), `Run <-> RunExplicit` conversion tests (Phase 2 step 6), TalkF + DinnerF integration test (Phase 4).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `fp-library/benches/benchmarks/run_*.rs`                                                          | **New bench files.** Per-Free-variant Criterion benches for all six variants (bind-deep, bind-wide, peel-and-handle) plus a cross-variant comparison documenting the O(1) vs O(N) bind-cost asymmetry between the Erased and Explicit families. Row-canonicalisation benches (macro vs Subsetter), handler-composition benches, and `Run <-> RunExplicit` conversion benches.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |

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
  to work for `RunExplicit` (full by-value brand coverage) once
  the corresponding `RunExplicitBrand` impls from Phase 2 step 4
  ship. `RcRunExplicit` / `ArcRunExplicit` carry only `Pointed` /
  `SendPointed` on the by-value side (the Clone bound on bind
  makes `Functor` / `Semimonad` / `Monad` unexpressible at the
  brand level, per `RcCoyoneda` / `ArcCoyoneda` precedent
  documented in
  [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md));
  brand-dispatched typeclass-generic code over them uses the
  existing `m_do!` / `a_do!` macros with the `ref` qualifier
  (`m_do!(ref RcRunExplicitBrand { ... })`), routing through
  `RefFunctor` / `RefSemimonad` (with the constraint that the
  row brand must implement `RefFunctor` — synthetic rows like
  `CoproductBrand<IdentityBrand, CNilBrand>` qualify, but
  canonical Coyoneda-headed rows generated by the `effects!`
  macro do not, because `CoyonedaBrand` cannot implement
  `RefFunctor` on stable Rust per the HRTB-over-types
  limitation in
  [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md)).
  `ArcRunExplicitBrand`'s `SendRef*` hierarchy is permanently
  unreachable on stable Rust regardless of row shape (the
  `ArcFreeExplicitBrand`-side `SendRefFunctor` impl is
  unimplementable for the same reason). The Erased Run family
  (`Run`, `RcRun`, `ArcRun`) is not Brand-dispatched at all.
  Both gaps route through the new `im_do!` (Inherent Monadic
  do) macro, which uses inherent `.bind(...)` /
  `.ref_bind(...)` method calls and works uniformly across all
  six wrappers.

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

Land the five missing Free variants and the `SendFunctor` trait
family. Phases 2-5 treat the choice of variant as a user-level
parameter, so completing the substrate first prevents later
refactor. The Erased family (`Free`, `RcFree`, `ArcFree`) is
inherent-method only; the Explicit family (`FreeExplicit`,
`RcFreeExplicit`, `ArcFreeExplicit`) carries Brand dispatch. See
[decisions.md](decisions.md) section 4.4 for rationale.

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
   Inherent-method only; no `RcFreeBrand` (the `'static` requirement
   from `Rc<dyn Any>` erasure is incompatible with `Kind`'s
   `Of<'a, A: 'a>: 'a` signature).
3. Implement `ArcFree<F, A>` at `fp-library/src/types/arc_free.rs`
   following the `ArcCoyoneda` template, with continuations
   expressed via
   [`FnBrand<ArcBrand>`](../../../fp-library/src/types/fn_brand.rs)
   parameterised by
   [`ArcBrand`](../../../fp-library/src/brands.rs#L43) (yielding
   `Arc<dyn 'a + Fn(B) -> ArcFree<F, A> + Send + Sync>` after
   `Kind` resolution via `SendRefCountedPointer`) and the
   `Kind<Of<'a, A>: Send + Sync>` associated-type-bound trick.
   Inherent-method only; no `ArcFreeBrand` (same `'static` reason).
4. Implement `RcFreeExplicit<'a, F, A>` at
   `fp-library/src/types/rc_free_explicit.rs` extending
   `FreeExplicit`'s concrete recursive enum with an outer
   `Rc<RcFreeExplicitInner>` wrapper plus
   [`FnBrand<RcBrand>`](../../../fp-library/src/types/fn_brand.rs)-shaped
   continuations. `A: 'a` (no `'static` requirement) because the
   structure has no `dyn Any` cell; O(N) bind via spine recursion
   through `F::map`. Add `lower_ref(&self)` / `peel_ref(&self)` and
   custom iterative `Drop` (the same `Extract`-driven dismantling
   pattern as `FreeExplicit`, with `Rc::try_unwrap` inside the
   loop). Brand-compatible: this is the multi-shot variant that
   carries Brand dispatch in Phase 1 step 7.
5. Implement `ArcFreeExplicit<'a, F, A>` at
   `fp-library/src/types/arc_free_explicit.rs` extending
   `RcFreeExplicit`'s shape with `Arc<...>` wrapping and
   `Arc<dyn Fn + Send + Sync>` continuations (constructed via
   [`<ArcFnBrand as SendLiftFn>::new`](../../../fp-library/src/classes/send_clone_fn.rs)).
   Same `Kind<Of<'a, A>: Send + Sync>` associated-type-bound trick
   as `ArcFree`. `Send + Sync`-capable; Brand-compatible (with the
   `SendFunctor` family from step 6 supplying the missing
   trait-method bounds).
6. Add the `SendFunctor` trait family at
   `fp-library/src/classes/`: `send_functor.rs`,
   `send_pointed.rs`, `send_semimonad.rs`, `send_monad.rs` (the
   by-value parallels of the existing `send_ref_*` files).
   `SendFunctor` / `SendSemimonad` / `SendMonad` take their
   closure parameter as `impl Fn(...) + Send + Sync`;
   `SendPointed::pure(a: A)` requires `A: Send + Sync`. Resolves
   the gap that today prevents `ArcCoyonedaBrand` from
   implementing `Functor` and gives `ArcFreeExplicitBrand` a
   brand-level `pure`. Add `SendFunctor` / `SendPointed` /
   `SendSemimonad` / `SendMonad` implementations for
   `ArcCoyonedaBrand` as a bonus integration, closing the open
   gap that
   [arc_coyoneda.rs](../../../fp-library/src/types/arc_coyoneda.rs)'s
   module docs flag (`ArcCoyoneda`'s by-value path has no Clone
   bound, so the full hierarchy lands).
7. Add by-value and by-reference trait hierarchies for the three
   Explicit Free brands. The brand structs (`FreeExplicitBrand<F>`,
   `RcFreeExplicitBrand<F>`, `ArcFreeExplicitBrand<F>`) and their
   `Kind` registrations land alongside the type definitions in
   steps 1, 4, and 5; this step implements the type-class traits
   on top of them. Brand-level coverage matches the resolved
   open question above:
   - `FreeExplicitBrand`: full by-value (`Functor` / `Pointed` /
     `Semimonad` / `Monad`) plus full by-reference (`RefFunctor`
     / `RefPointed` / `RefSemimonad` / `RefMonad` and supporting
     Ref traits per
     [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md)).
     The naive recursive enum has no Clone bound on bind, so
     both hierarchies land cleanly.
   - `RcFreeExplicitBrand`: `Pointed` only on the by-value side
     (`pure` has no Clone bound); full Ref hierarchy
     (`RefFunctor` / `RefSemimonad` / `RefMonad`, plus
     `RefPointed` and supporting Ref traits). The remaining
     by-value operations (`bind`, `map`, etc.) ship as inherent
     methods on `RcFreeExplicit` with their natural `A: Clone`
     bounds.
   - `ArcFreeExplicitBrand`: `SendPointed` (from step 6) on the
     by-value side; full `SendRef*` hierarchy (`SendRefFunctor`
     / `SendRefSemimonad` / `SendRefMonad` plus supporting
     `SendRef*` traits, which already exist in
     [`fp-library/src/classes/`](../../../fp-library/src/classes/)).
     The remaining by-value operations ship as inherent methods
     on `ArcFreeExplicit` with `A: Clone + Send + Sync` bounds.
   - The Erased family (`Free`, `RcFree`, `ArcFree`) does not get
     brands; those types remain inherent-method only.
   - Both hierarchies are required so `dispatch::map` /
     `dispatch::bind` route correctly over each Brand-dispatched
     Free variant once `Run` and the scoped-effect smart
     constructors land in Phase 2 / Phase 4. The Ref hierarchy
     is the dispatch path for typeclass-generic code over the
     multi-shot Explicit Run variants.
   - Update
     [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md)'s
     "Unexpressible Bounds in Trait Method Signatures"
     classification table to add rows for the three Explicit
     Free variants documenting the brand-level coverage above
     (matching the existing `RcCoyoneda` / `ArcCoyoneda` rows).
8. Per-variant Criterion benches for all six variants (bind-deep
   at depths 10 / 100 / 1000 / 10000, bind-wide, peel-and-handle),
   plus a cross-variant comparison bench documenting the O(1) vs
   O(N) bind-cost asymmetry. The existing
   [`free_explicit.rs`](../../../fp-library/benches/benchmarks/free_explicit.rs)
   POC bench has the `bind-deep` shape only; step 8 extends that
   shape with `bind-wide` (single bind closure mapping over a
   wide-but-shallow chain) and `peel-and-handle` (single-step
   `to_view` / `peel_ref` cost) and replicates the full set
   across all six variants.
9. Per-variant unit tests covering construction, evaluation, and
   the property each variant promises (single-shot vs.
   multi-shot, thread-safe, `'static` vs `'a`, Brand-dispatched
   vs inherent-method-only). The canonical interpretation method
   varies by variant: `Free::fold_free` for `Free` (the only
   variant with that inherent method); `evaluate` for
   `RcFree`/`ArcFree`/`FreeExplicit`/`RcFreeExplicit`/`ArcFreeExplicit`.
   Plus `compile_fail` UI tests under
   [`fp-library/tests/ui/`](../../../fp-library/tests/ui/)
   (registered via the existing
   [`fp-library/tests/compile_fail.rs`](../../../fp-library/tests/compile_fail.rs)
   `trybuild` harness) for the negative cases: multi-shot via
   `Free`, Brand-dispatched call against an Erased variant,
   missing `Send + Sync` on a closure passed to
   `ArcFreeExplicit::bind`, missing `Clone` on a closure passed
   to `RcFree::bind`, etc.

### Phase 1 follow-up: `WrapDrop` migration (resolves Phase 2 step 4 blocker)

These commits land between Phase 1 and Phase 2 step 4. They lift
the Free family's struct-level `Extract` bound so the Phase 2
step 4 architectural commitment
`Run<R, S, A> = Free<NodeBrand<R, S>, A>` can compile over
typical effect rows whose effect types do not implement
`Extract`. Full rationale, problem statement, probe results, and
per-F policy decisions live at
`## Open questions, issues and blockers -> ### Resolved (2026-04-27): introduce WrapDrop trait for Free's struct-level Drop concern`;
this section is the phasing-side checklist.

1. **Introduce the `WrapDrop` trait and migrate the Free family.**
   New trait at `fp-library/src/classes/wrap_drop.rs` with
   signature
   `fn drop<'a, X: 'a>(fa: Self::Of<'a, X>) -> Option<X>`.
   `WrapDrop` impls for the two existing
   `Extract`-implementing brands (`IdentityBrand`,
   `ThunkBrand`), each delegating to their existing
   `Extract` impl by returning
   `Some(<Self as Extract>::extract(fa))`. Replace
   `F: Extract + Functor + 'static` with
   `F: WrapDrop + Functor + 'static` on the struct, `FreeView`,
   `FreeStep`, and `Drop` declarations of all six Free
   variants (`Free`, `RcFree`, `ArcFree`, `FreeExplicit`,
   `RcFreeExplicit`, `ArcFreeExplicit`). Inventory: 71
   occurrences of `F: Extract` across the six variant source
   files, mechanically migrated. Methods that call
   `F::extract` semantically (`evaluate`, `resume`, etc.)
   keep `where F: Extract` on their impl blocks. Rewrite
   Free's `Drop` loop to call `F::drop` and switch on the
   returned `Option`: `Some(inner)` follows the existing
   iterative path; `None` lets the layer drop in place.
   Existing Phase 1 tests (including
   `deep_drop_does_not_overflow` for both `Free<ThunkBrand>`
   and `FreeExplicit<IdentityBrand>`) must all pass.
2. **Relax `Functor` bound to `Kind` on Free's struct.**
   Change the struct, `FreeView`, `FreeStep`, and `Drop`
   bounds from `F: WrapDrop + Functor + 'static` to
   `F: WrapDrop + 'static` (the `Kind` GAT requirement is
   inherited from `WrapDrop`'s `Kind` supertrait). Add
   `where F: Functor` to impl blocks that call `F::map`
   (`wrap`, `lift_f`, `to_view`, and methods that go through
   them transitively such as `evaluate`, `resume`,
   `fold_free`). Methods like `pure`, `bind`, `map` (the
   inherent method, not `Functor::map`) do not need the
   bound. Process: relax the struct bound, run
   `cargo check`, add `where F: Functor` to every impl block
   the compiler flags, repeat until clean. Same six Free
   variants. Same tests must pass.

### Phase 2: Run substrate and first-order effects

1. Add `frunk_core` as a direct dependency of `fp-library`
   (license check via `just deny`, MSRV verification, and
   workspace `Cargo.toml` registration). Introduce a thin
   Brand-aware adapter layer at `fp-library/src/types/effects/coproduct.rs`:
   newtype wrappers around `frunk_core::coproduct::{Coproduct, CNil}`
   plus `impl` blocks bridging `frunk_core`'s Plucker / Sculptor /
   Embedder traits to the project's `Brand` system. Direct `impl`s
   of fp-library's own `Functor` for `frunk_core::Coproduct<H, T>`
   are permitted by the orphan rules; `Brand`-style impls require
   the newtype wrapper. See Implementation note 1 below.
2. `VariantF<Effects>` at `fp-library/src/types/effects/variant_f.rs`:
   Coyoneda-wrapped Coproduct row with recursive `Functor` impl
   on `Coproduct<H, T>` (where `H: Functor + T: Functor`) and base
   case on `CNil`. Migrate the trait-shape from
   `poc-effect-row/src/lib.rs` (workspace deleted in 10b)
   under the production `Functor` trait.
3. `Member<E, Indices>` trait for first-order injection /
   projection, layered on top of `frunk_core::CoproductSubsetter`
   via the adapter from step 1.
4. Six `Run` types at `fp-library/src/types/effects.rs` (and
   sibling files), one per Free variant: `Run<R, S, A>`,
   `RcRun<R, S, A>`, `ArcRun<R, S, A>` (Erased family,
   inherent-method only) and `RunExplicit<'a, R, S, A>`,
   `RcRunExplicit<'a, R, S, A>`, `ArcRunExplicit<'a, R, S, A>`
   (Explicit family, Brand-dispatched). Each is a thin wrapper
   over its Free variant with a shared `Node<R, S>` enum
   dispatching first-order vs scoped layers.
   This step depends on the Phase 1 follow-up commits above
   (the `WrapDrop` migration); without them,
   `Free<NodeBrand<R, S>, A>` does not compile because effect
   types do not implement `Extract`. As part of this step,
   `WrapDrop` impls also land for the row brands that this
   step exercises:
   - `NodeBrand<R, S>`: dispatches by First/Scoped, delegating
     to `R::drop` and `S::drop` respectively. (New brand
     defined in this step.)
   - `CoproductBrand<H, T>` (already exists from Phase 2 step 2):
     dispatches by `Inl`/`Inr`, delegating to `H::drop` and
     `T::drop`.
   - `CNilBrand` (already exists from Phase 2 step 2): the
     uninhabited base case, `match fa {}`.
   - `CoyonedaBrand<E>` (already exists): returns `None`. The
     Coyoneda's stored function would construct a Free if
     called, but does not store one in its environment;
     recursive drop on the Coyoneda is sound for Run-typical
     patterns per the Wrap-depth probe findings recorded in
     the resolution above.
   - `RunExplicitBrand`: full by-value (`Functor` / `Pointed` /
     `Semimonad` / `Monad`) plus full by-reference (`RefFunctor`
     / `RefPointed` / `RefSemimonad` / `RefMonad` and supporting
     Ref traits) by delegating to `FreeExplicitBrand`'s impls
     from Phase 1 step 7.
   - `RcRunExplicitBrand`: `Pointed` only on the by-value side
     (delegating to `RcFreeExplicitBrand::pure`); full Ref
     hierarchy delegating to `RcFreeExplicitBrand`'s `Ref*`
     impls. By-value `bind` / `map` ship as inherent methods on
     `RcRunExplicit`, mirroring `RcFreeExplicit`'s inherent
     surface.
   - `ArcRunExplicitBrand`: `SendPointed` (added by step 6) on
     the by-value side; full `SendRef*` hierarchy delegating to
     `ArcFreeExplicitBrand`'s impls. By-value `bind` / `map`
     ship as inherent methods on `ArcRunExplicit` with
     `A: Clone + Send + Sync` bounds.
   - The three Erased Run types do NOT get brands. They expose
     identical inherent-method APIs (`pure`, `peel`, `send`,
     `bind`, `map`, `lift_f`, `handle`, `extract`, etc.) but
     `m_do!` / `a_do!` do not work over them; `im_do!` from
     step 7 below is the inherent-method-based macro analogue.
   - The hierarchies on the Explicit brands are scoped so
     `dispatch::map` / `dispatch::bind` and the matching
     do-notation macros (`m_do!` / `a_do!` for `RunExplicit`;
     `m_do!(ref ...)` / `a_do!(ref ...)` for `RcRunExplicit` /
     `ArcRunExplicit`) route correctly. Inherent by-value `bind`
     / `map` on the multi-shot Explicit Run variants cover the
     non-generic case where the user has `A: Clone` available.
5. `Run::pure`, `Run::peel`, `Run::send` core operations on
   each of the six Run variants, delegating to the underlying
   Free variant.
6. Conversion methods between paired Erased and Explicit Run
   variants: `Run::into_explicit() -> RunExplicit`,
   `RcRun::into_explicit() -> RcRunExplicit`,
   `ArcRun::into_explicit() -> ArcRunExplicit`, and the reverse
   `RunExplicit::from_erased(...)`, etc. Walks the Free
   structure once via `peel` / `to_view`, rebuilds in the other
   shape; O(N) in the chain depth. Preserves multi-shot /
   `Send + Sync` properties of the underlying substrate
   (`RcRun -> RcRunExplicit` keeps multi-shot via `Rc<dyn Fn>`
   continuations on both sides; `ArcRun -> ArcRunExplicit`
   keeps `Send + Sync`).
7. **Inherent monadic do-notation: `im_do!` macro plus the
   inherent-method scaffolding it desugars against.** Three
   sub-tasks:

   **7a. Inherent `bind` and `map` on the four Run wrappers
   that don't already have them.** `Run`, `RcRun`, `ArcRun`,
   and `RunExplicit` need both inherent methods at the
   wrapper level (delegating to their underlying Free
   variant's `bind` / `map`); `RcRunExplicit` / `ArcRunExplicit`
   already ship them from step 4b. Bounds match the underlying
   substrate's: `Run` / `RunExplicit` have no extra bounds
   beyond their impl block; `RcRun` / `ArcRun` need
   `A: Clone` plus the projection `Clone` bound that their
   `peel` carries; `ArcRun` additionally needs
   `A: Send + Sync` and `NodeBrand<R, S>: Functor` per-method
   (the impl block carries only the `Send + Sync` projection
   HRTB).

   **7b. Inherent `ref_bind` and `ref_map` on the four
   `Clone`-able wrappers.** `RcRun`, `ArcRun`, `RcRunExplicit`,
   `ArcRunExplicit` are all `Clone` (via `Rc`/`Arc`-shared
   substrate); `Run` and `RunExplicit` are not (they wrap an
   unboxed substrate). On the `Clone`-able four, `ref_bind`
   is implementable as `self.clone().bind(move |a| f(&a))`
   and `ref_map` analogously. The clone is `O(1)` (Rc/Arc
   refcount bump), so the by-reference path adds one cheap
   refcount operation per layer.

   The structural reason this matters: it sidesteps the
   `R: RefFunctor` cascade that brand-level
   `RcFreeExplicitBrand: RefSemimonad` / `RcRunExplicitBrand: RefSemimonad`
   require. The inherent `ref_bind` walks the substrate
   by-value (with the wrapping closure converting `A` to
   `&A` for the user-supplied closure), so the row brand
   doesn't need `RefFunctor`. This is the path that lets
   users get by-reference semantics over canonical
   Coyoneda-headed rows generated by `effects!`, where
   brand-level `m_do!(ref ...)` cannot reach because
   `CoyonedaBrand: RefFunctor` is unimplementable on stable
   Rust per
   [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md).

   **7c. `im_do!` macro in `fp-macros/src/effects/im_do.rs`**
   ("Inherent Monadic do"). Inherent-method-based monadic
   do-notation that desugars to chained
   `expr.bind(|x| ...)` calls (or `expr.ref_bind(|x| ...)`
   for the `ref` form). Mirrors `m_do!`'s surface syntax so
   users moving between brand-dispatched and inherent paths
   do not have to re-learn anything; the only differences
   are the macro name (`im_do!` vs `m_do!`) and the type
   the macro takes (concrete wrapper, e.g.,
   `im_do!(RcRun { ... })`, vs brand, e.g.,
   `m_do!(RcRunExplicitBrand { ... })`). Both inferred and
   explicit-wrapper modes ship: `im_do!({ ... })` (inferred,
   monomorphizes against the leading bind expression's type)
   and `im_do!(Wrapper { ... })` (explicit, useful when the
   wrapper type can't be inferred or for `pure(x)` rewriting
   to `Wrapper::pure(x)`).

   The `ref` qualifier (`im_do!(ref Wrapper { ... })`) is
   accepted only for the four `Clone`-able wrappers. Using
   it on `Run` or `RunExplicit` produces a clear "cannot use
   `ref` form on non-`Clone` wrapper" diagnostic, demonstrated
   by a `compile_fail` UI test in
   `fp-library/tests/ui/im_do_ref_on_non_clone_wrapper.rs`.

   The macro coverage matrix:

   | Wrapper          | `im_do!` | `im_do!(ref ...)`             | Brand-dispatched alternative                    |
   | :--------------- | :------- | :---------------------------- | :---------------------------------------------- |
   | `Run`            | works    | not implementable (not Clone) | none (Erased family is not Brand-dispatched)    |
   | `RcRun`          | works    | works                         | none                                            |
   | `ArcRun`         | works    | works                         | none                                            |
   | `RunExplicit`    | works    | not implementable (not Clone) | `m_do!`/`a_do!` (full brand coverage)           |
   | `RcRunExplicit`  | works    | works                         | `m_do!(ref ...)` (synthetic rows only)          |
   | `ArcRunExplicit` | works    | works                         | `m_do!`/`a_do!` (only `pure`; bind unreachable) |

   Naming note: `im_do!` ("Inherent Monadic do") parallels
   `m_do!` ("Monadic do"); a future companion `ia_do!`
   ("Inherent Applicative do") is reserved as the
   inherent-method-based applicative analogue (parallel to
   `a_do!`), to be added when a concrete need arises (e.g.,
   handler-side independent-bind composition over `ArcRun`
   in Phase 3+). The names share a common length so neither
   the monadic nor the applicative form is favored
   typographically; users should prefer `ia_do!` over
   `im_do!` whenever binds are independent, just as they
   should prefer `a_do!` over `m_do!`.

   Both forms route their codegen through a shared input
   parser at `fp-macros/src/support/do_input.rs` (extracted
   from the existing `m_do/input.rs` / `a_do/input.rs`)
   so surface-syntax features (typed binds, `let`-in-bind,
   `pure(x)` rewriting, `ref` qualifier, etc.) stay
   consistent across all four macros.

   Implementation reference: the existing
   [`m_do!`](../../../fp-macros/src/lib.rs) and
   [`a_do!`](../../../fp-macros/src/lib.rs) macros are the
   structural template. The differential against `m_do!` is
   only the codegen target: brand-dispatched
   `<Brand as Semimonad>::bind(expr, f)` becomes inherent
   `expr.bind(f)`. Same input parser, same statement-form
   handling, same `ref`-mode lifetime concerns.

8. `effects!` macro in `fp-macros/src/effects/effects.rs`,
   migrated from `poc-effect-row/macros/src/lib.rs`
   (workspace deleted in 10b).
   Lexical-sort by `quote!{}.to_string()`; emit Coyoneda-wrapped
   variants. The un-wrapped Coproduct form lives at
   `crate::__internal::raw_effects!` for fp-library-internal use
   (test fixtures, lower-level combinators) and is not part of
   the public surface; see [decisions.md](decisions.md) section 4.6.
   Factor the lexical-sort logic into a shared `proc-macro2`
   helper used by both `effects!` and `scoped_effects!` (Phase 4
   step 4) so sort-correctness fixes land in one place.
9. **Generic `lift` combinator** (PureScript Run's
   [`Run.lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
   analog) as an inherent associated function on each of the six
   Run wrappers, mirroring `*Run::send`'s shape. Per the
   [2026-04-28 resolution](resolutions.md#resolved-2026-04-28-phase-2-step-9-scope-is-under-specified)
   and the
   [2026-04-28 expansion](resolutions.md#resolved-2026-04-28-implementation-expansion-step-9-sendfunctor-cascade-prerequisites-for-arc-family):
   take the raw effect (an `EBrand::Of<'a, A>` value, not a
   pre-lifted Coyoneda), do the full chain inside the body
   (Coyoneda lift -> row inject -> `Node::First` -> `*Run::send`),
   and let `Idx` be type-inferred at call sites where the row is
   unambiguous (turbofish only on duplicate-effect-type rows). The
   bare name `lift` matches PureScript Run's `Run.lift`; the `_f`
   suffix is reserved for the Free-only operation
   ([`Free::lift_f`](../../../fp-library/src/types/free.rs), the
   snake_case translation of PureScript's `Free.liftF`).

   The work breaks into nine sub-steps; each lands as a separate
   commit and verifies under `just verify` independently. Sub-steps
   9a-9g establish the `SendFunctor` cascade prerequisites for the
   Arc family (the architectural finding documented in the 2026-04-28
   expansion); sub-steps 9h-9i complete the universal `lift` and
   `SendRefFunctor` work. Already landed: the `Run::lift` reference
   implementation at commit
   [`34b6a97`](../../../fp-library/src/types/effects/run.rs) (which
   names + signs off on the chosen design but doesn't extend to the
   Arc family).

   **9a. Brand-level `SendFunctor` cascade plus missing `WrapDrop`.**
   Add `SendFunctor` impls on the row-cascade brands that don't have
   them: `IdentityBrand`, `CNilBrand`, `CoproductBrand<H, T>` (recursive,
   requiring `H: SendFunctor + T: SendFunctor`), and
   `NodeBrand<R, S>` (delegates to the first-order and scoped row
   brands' `SendFunctor` impls). Add the missing
   [`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs) impl on
   [`ArcCoyonedaBrand`](../../../fp-library/src/types/arc_coyoneda.rs)
   (returns `None`, mirroring the existing
   [`CoyonedaBrand`](../../../fp-library/src/types/coyoneda.rs) /
   [`RcCoyonedaBrand`](../../../fp-library/src/types/rc_coyoneda.rs)
   pattern; the Coyoneda's stored function does not materially store
   an inner Free, so structural-recursive drop is sound). All impls
   are mechanical mirrors of the existing `Functor` / `WrapDrop`
   patterns; no novel algorithm.

   **9b. Replace `F: Functor` with `F: SendFunctor` on `ArcFree`.**
   The substrate at
   [`fp-library/src/types/arc_free.rs`](../../../fp-library/src/types/arc_free.rs)
   currently bounds `lift_f`, `wrap`, `bind`, `evaluate`, `fold_free`,
   `hoist_free`, etc. on `F: Functor` and routes `F::map` calls
   through it. Switch all such bounds to `F: SendFunctor` and replace
   `F::map(...)` calls with `F::send_map(...)`. The closures passed
   at the call sites (`ArcFree::pure`, user-supplied
   `Send + Sync`-bound continuations from `bind`, etc.) are already
   `Send + Sync`, so the migration is mechanical. This is a breaking
   change for any pre-existing caller passing a non-Send `Functor`
   row brand, but `ArcFree`'s struct-level
   `Of<'static, ArcFree<...>>: Send + Sync` HRTB already restricts
   concrete callers to row brands that satisfy `Send + Sync`, so
   adding `SendFunctor` impls (sub-step 9a) keeps existing callers
   working.

   **9c. Replace `F: Functor` with `F: SendFunctor` on
   `ArcFreeExplicit`.** Same migration as 9b for the substrate at
   [`fp-library/src/types/arc_free_explicit.rs`](../../../fp-library/src/types/arc_free_explicit.rs).
   Method signatures and internal `F::map` call sites switch to
   `F::send_map`.

   **9d. Expand brand-level type-class surface on
   `ArcFreeExplicitBrand`.** With `ArcFreeExplicit`'s machinery now
   routed through `SendFunctor`, the brand-level coverage gap
   documented in step 4b (per-`A` HRTB blocking SendRef-family
   impls) shifts. Re-evaluate the cascade and land newly-reachable
   impls: at minimum `SendFunctor`; potentially `SendSemimonad`,
   `SendApplicative`, `SendMonad` if their dependencies are
   satisfiable through the same SendFunctor-aware substrate. Document
   any remaining unreachable subset in
   [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md).
   Don't add `SendRefFunctor` here; that's sub-step 9i.

   **9e. Switch `ArcRun` to `SendFunctor`-routed dispatch.** The
   wrapper at
   [`fp-library/src/types/effects/arc_run.rs`](../../../fp-library/src/types/effects/arc_run.rs)
   currently routes `peel`/`send`/`bind`/`map` through
   `<NodeBrand<R, S> as Functor>::map`. Switch to
   `<NodeBrand<R, S> as SendFunctor>::send_map`, calling the
   Send-aware `ArcFree` siblings from 9b. The struct-level HRTB
   stays (the `Of<'static, ArcFree<...>>: Send + Sync` bound is
   orthogonal to the trait bound on `R`).

   **9f. Switch `ArcRunExplicit` to `SendFunctor`-routed dispatch.**
   Same migration as 9e for
   [`fp-library/src/types/effects/arc_run_explicit.rs`](../../../fp-library/src/types/effects/arc_run_explicit.rs).

   **9g. Expand brand-level type-class surface on
   `ArcRunExplicitBrand`.** Step 4b documented the brand-level
   coverage as `SendPointed` only. With the SendFunctor-aware
   substrate machinery from 9c-9d, this expands to `SendPointed`
   plus whatever cascades from `ArcFreeExplicitBrand`'s expanded
   surface (sub-step 9d). Land the newly-reachable impls; document
   any remaining gap.

   **9h. Add `lift` inherent method to all six Run wrappers.** The
   originally-planned step 9 work, now unblocked for the Arc family
   by the 9a-9g cascade. Reference signature for `Run`:

   ```rust
   impl<R: Kind, S: Kind, A: 'static> Run<R, S, A> {
       pub fn lift<EBrand, Idx>(
           effect: Apply!(<EBrand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'static, A>),
       ) -> Self
       where
           Apply!(<R as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'static, A>):
               Member<Coyoneda<'static, EBrand, A>, Idx>,
           EBrand: Kind_cdc7cd43dac7585f + 'static,
       {
           let coyo: Coyoneda<'static, EBrand, A> = Coyoneda::lift(effect);
           let layer = <Apply!(<R as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'static, A>)
               as Member<Coyoneda<'static, EBrand, A>, Idx>>::inject(coyo);
           Self::send(Node::First(layer))
       }
   }
   ```

   For the Erased Rc and Explicit families, the wrapper substitutes
   their own Coyoneda variant (`Coyoneda` for non-Arc; `ArcCoyoneda`
   for `ArcRun` / `ArcRunExplicit`) and the corresponding Member
   bound. Per-wrapper delta table:

   | Wrapper          | `'a`         | Coyoneda variant             | Extra `A` bound            |
   | :--------------- | :----------- | :--------------------------- | :------------------------- |
   | `Run`            | `'static`    | `Coyoneda<'static, _, _>`    | `A: 'static`               |
   | `RcRun`          | `'static`    | `Coyoneda<'static, _, _>`    | `A: 'static`               |
   | `ArcRun`         | `'static`    | `ArcCoyoneda<'static, _, _>` | `A: Send + Sync + 'static` |
   | `RunExplicit`    | `'a` (param) | `Coyoneda<'a, _, _>`         | `A: 'a`                    |
   | `RcRunExplicit`  | `'a` (param) | `Coyoneda<'a, _, _>`         | `A: 'a`                    |
   | `ArcRunExplicit` | `'a` (param) | `ArcCoyoneda<'a, _, _>`      | `A: 'a + Send + Sync`      |

   `Run::lift` already landed at commit
   [`34b6a97`](../../../fp-library/src/types/effects/run.rs); 9h
   adds the remaining five. `ArcRun::lift` may need the
   `lift_node` HRTB-fallback helper from the original 2026-04-28
   resolution; defer to the implementer's experience.

   **HRTB-poisoning fallback.** Try the inline body first on every
   wrapper. If `ArcRun::lift` fails to compile due to GAT-normalization
   recurring under `ArcFree`'s HRTB-bearing impl-block scope (the
   2026-04-27 limit), factor the literal-build step into a free
   helper outside the HRTB scope:

   ```rust
   pub fn lift_node<R, S, EBrand, Idx, A>(
       effect: Apply!(<EBrand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'static, A>),
   ) -> Apply!(<NodeBrand<R, S> as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'static, A>)
   where /* SendFunctor cascade bounds */
   {
       Node::First(<_ as Member<_, Idx>>::inject(ArcCoyoneda::lift(effect)))
   }
   ```

   Then `ArcRun::lift` calls
   `Self::send(lift_node::<R, S, EBrand, Idx, A>(effect))`. Don't
   pre-bake `lift_node` for the other five wrappers prophylactically.

   **Tests.** One integration test per wrapper at
   `fp-library/tests/run_lift.rs` covering: lift -> peel two-step
   round-trip (lower the inner Coyoneda's stored continuation, peel
   that to recover the value) on a single-effect row; second-branch
   injection through a multi-effect row (proves `Member` resolves
   the position correctly); inferred-`Idx` compiles unambiguously;
   `*Run::lift::<EBrand, _>(effect).bind(...)` composition. Erased
   Rc/Arc-family `peel` carries a row-projection `Clone` bound that
   Coyoneda-headed rows don't satisfy; substitute construction-only
   tests for those two wrappers.

   This is the "thin wrapper over `inj + liftF`/`send`"
   infrastructure that [decisions.md](decisions.md) section 6 names
   as the prerequisite for Phase 3's per-effect smart constructors
   (`ask`, `get`, `put`, `modify`, `tell`, `throw`). Each of those
   becomes a one-liner over `*Run::lift`, parallel to PureScript
   Run's `liftEffect = lift (Proxy :: "effect")` pattern.

   **9i. `SendRefFunctor` on `ArcRunExplicitBrand` via inherent-method
   delegation.** Step 4b documented the SendRef-family hierarchy on
   `ArcRunExplicitBrand` as unreachable through brand-level
   delegation because `ArcFreeExplicitBrand` doesn't implement it
   (per-`A` HRTB on `Kind` projection, unexpressible in trait method
   signatures). 9i sidesteps that gap with a different delegation
   strategy: implement `SendRefFunctor` on `ArcRunExplicitBrand` by
   calling the wrapper's inherent
   [`ref_map`](../../../fp-library/src/types/effects/arc_run_explicit.rs)
   directly, which uses the clone-trick
   (`self.clone().map(move |a| f(&a))`) to bypass the brand-level
   cascade. The `O(1)` `Arc::clone` makes this cheap; the inherent
   ref methods already handle the per-`A` constraints at the wrapper
   level.

   Reference shape:

   ```rust
   impl<R, S> SendRefFunctor for ArcRunExplicitBrand<R, S>
   where
       R: WrapDrop + SendFunctor + 'static,
       S: WrapDrop + SendFunctor + 'static,
   {
       fn send_ref_map<'a, A: 'a, B: 'a, Func>(
           f: Func,
           fa: &Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
       ) -> Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
       where
           Func: Fn(&A) -> B + Send + Sync + 'a,
       {
           fa.ref_map(f)
       }
   }
   ```

   Applies the same delegation pattern to other reachable
   SendRef-family traits (`SendRefSemimonad` via `ref_bind`,
   `SendRefPointed` via `ref_pure`) where the wrapper's inherent
   counterpart admits direct delegation. Document anything that
   doesn't admit delegation in
   [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md).
   `ArcRun` (the Erased family) has no brand, so its SendRef
   coverage stays inherent-method-only via
   [`im_do!(ref ArcRun { ... })`](../../../fp-macros/src/effects/im_do.rs).

10. Migrate the 25 row-canonicalisation tests from
    `poc-effect-row/tests/` into
    `fp-library/tests/run_row_canonicalisation.rs` as the
    regression baseline. Verify all pass under the production
    types (exercise both Erased and Explicit Run families).
    Delete the POC repository once the migration lands.

### Phase 3: First-order effect handlers, interpreters, natural transformations

1. `handlers!{...}` macro in
   `fp-macros/src/effects/handlers.rs` producing tuple-of-closures
   keyed on the row's type-level structure. Builder fallback
   (`nt().on::<E>(handler)...`) as the non-macro path
   ([decisions.md](decisions.md) section 4.6).
2. `interpret` / `run` / `runAccum` recursive-target interpreter
   family in `fp-library/src/types/effects/interpreter.rs`.
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
   `fp-library/src/types/effects/scoped.rs` with the dual-row
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
   - `Bracket<'a, A, B>` (Val flavour) for non-refcounted-substrate
     users (`Run` / `RunExplicit`), with `acquire: Run<R, S, A>`,
     `body: Box<dyn FnOnce(A) -> Run<R, S, (A, B)>>`,
     `release: Box<dyn FnOnce(A) -> Run<R, S, ()>>`. The body
     consumes `A`, threads it back to the interpreter via
     `(A, B)`, and the interpreter moves the returned `A` into
     `release`.
   - `RefBracket<'a, P, A, B>` (Ref flavour) for refcounted-substrate
     users (`RcRun`, `ArcRun`, `RcRunExplicit`, `ArcRunExplicit`),
     parameterised by
     [`P: RefCountedPointer`](../../../fp-library/src/classes/ref_counted_pointer.rs)
     ([`RcBrand`](../../../fp-library/src/brands.rs#L250) for
     `RcRun` / `RcRunExplicit`,
     [`ArcBrand`](../../../fp-library/src/brands.rs#L43) for
     `ArcRun` / `ArcRunExplicit`), with `acquire: Run<R, S, A>`,
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
5. **Documentation finalization.** Update the documents listed
   below so they reflect the production state of the effects
   subsystem once Phases 1-5 are complete.

   **Living step:** Each implementation phase, on completion,
   must review the bullets here and add any new public items,
   behavioural surprises, or constraints that surfaced during
   that phase's work, under the relevant document. The goal is
   that when this step finally runs, every documentation change
   it lists is accurate and nothing has been forgotten. Treat
   the per-document bullets as a checklist that grows over time;
   do not rely on memory or `git log` to reconstruct the change
   set at the end. If a phase finds that a planned doc update
   is no longer needed (e.g., a feature was deferred), strike
   it through with rationale rather than deleting it.

   Documents and what to add:
   - **[fp-library/docs/features.md](../../../fp-library/docs/features.md):**
     Add a "Free family" table parallel to the existing "Free
     functors" Coyoneda table (currently around lines 199-204),
     listing the six variants (`Free`, `RcFree`, `ArcFree`,
     `FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit`) with
     columns for Family (Erased / Explicit), Clone, Send,
     `'a`-payload, Bind cost (O(1) vs O(N)). Add a "Run
     subsystem" section listing the six concrete Run types,
     the dual-row structure, the Erased/Explicit dispatch
     split, and the `into_explicit` / `from_erased` API.
   - **[fp-library/docs/limitations-and-workarounds.md](../../../fp-library/docs/limitations-and-workarounds.md):**
     The "Unexpressible Bounds in Trait Method Signatures"
     classification table already has rows for the three
     Explicit Free variants (added by Phase 1 step 7). Append
     rows for any `*RunExplicitBrand` (Phase 2 step 4) or
     scoped-effect dispatch (Phase 4) impls that hit further
     HRTB-over-types or per-`A` Clone-bound walls.
   - **[fp-library/CHANGELOG.md](../../../fp-library/CHANGELOG.md):**
     Populate the `[Unreleased]` section under `Added` with the
     new public items: six-variant Free family (one promoted
     from POC, five new), `SendFunctor` trait family, six
     `Run` types, `Node` / `VariantF` / `ScopedCoproduct`,
     standard first-order effects (`State`, `Reader`, `Except`,
     `Writer`, `Choose`), standard scoped effects (`Catch`,
     `Local` / `RefLocal`, `Bracket` / `RefBracket`, `Span`),
     the macro family (`effects!`, `effects_coyo!`,
     `handlers!`, `define_effect!`, `define_scoped_effect!`,
     `scoped_effects!`, `im_do!`), the
     `interpret`/`interpretRec`/`run*` interpreter pair, and
     the natural-transformation builder. If any pre-existing
     public API changed shape during the port, record it under
     `Changed`. Match the categorization style established in
     0.17.x entries.
   - **[README.md](../../../README.md):** Add a brief
     "Effects" entry alongside the existing "Dispatch System"
     summary, pointing at `fp-library/docs/run.md` (created by
     step 4) for details.
   - **[docs/todo.md](../../../docs/todo.md):** Strike through
     or remove the "Algebraic effects/effect system" bullet
     (and its sub-bullets pointing at
     [plans/effects/effects.md](effects.md) and external Eff
     references); the work it tracks is now landed.
   - **[fp-library/docs/architecture.md](../../../fp-library/docs/architecture.md):**
     If the effects subsystem warrants top-level architectural
     description (parallel to existing "Free Functions" /
     "Dispatch" sections), add one summarising the
     six-variant Free substrate, the Erased/Explicit dispatch
     split, the dual-row Run shape, and the heftia-style
     scoped-effect encoding. Skip if `run.md` (step 4) already
     covers this depth and an architecture-level summary
     would duplicate.
   - **[fp-library/docs/dispatch.md](../../../fp-library/docs/dispatch.md):**
     If Phase 4's `BracketDispatch` / `LocalDispatch` Val/Ref
     dispatch introduces a pattern that doesn't follow the
     existing convention this doc describes, add a section
     covering the new shape. Skip if the new dispatch is a
     direct application of the existing pattern.

   Per-phase records (append as phases complete):
   - **Phase 1 (complete).** Six Free variants land
     (`FreeExplicit` promoted, `RcFree` / `ArcFree` /
     `RcFreeExplicit` / `ArcFreeExplicit` new), `SendFunctor`
     trait family lands across nine files, brand impls for the
     three Explicit Free brands land. The
     `limitations-and-workarounds.md` "Unexpressible Bounds"
     table gained six new rows (three by-value, three
     by-reference) for the Explicit Free family;
     `features.md` and `CHANGELOG.md` are not yet updated for
     these (waiting for this finalization step). The Phase 1
     step 8 finding that `Free<IdentityBrand, A>` is
     layout-cyclic should be mentioned in `run.md` (step 4)'s
     "When to use which" section because it constrains
     concrete-`F` choices.
   - **Phase 2 (in progress).** Phase 2 ships the `WrapDrop`
     trait at
     `fp-library/src/classes/wrap_drop.rs`
     as a Phase 1 retroactive refinement before step 4 resumes
     (see Open questions resolution above for the rationale).
     `WrapDrop` is a new public trait that needs to land in
     `features.md` (effects subsystem section) and the
     `CHANGELOG.md` `[Unreleased]` Added list. Free's struct
     bound migrates from
     `F: Extract + Functor + 'static` to
     `F: WrapDrop + 'static`; this is technically a breaking
     change to the bound but is purely-additive in practice
     because every existing F that implements `Extract` gains
     a paired `WrapDrop` impl.
     Append remaining findings here when Phase 2 completes:
     new public items in `Run`, `VariantF`, `Node`, the
     `effects!` macro, the `im_do!` macro, the conversion
     API, plus any unexpressible-bound rows that surface in
     the `*RunExplicitBrand` impls.
   - **Phase 3 (TBD).** Append findings here when complete:
     handler-pipeline machinery, interpreter family, standard
     first-order effects, `handlers!` and `define_effect!`
     macros, plus negative-case `compile_fail` UI tests.
   - **Phase 4 (TBD).** Append findings here when complete:
     scoped-effect coproduct, dual-row integration, the four
     standard scoped-effect constructors and their Val/Ref
     flavours where applicable, dispatch additions for
     `bracket` / `local`, plus any new dispatch.md /
     limitations-and-workarounds.md material.

### Phase 6+ (deferred, not in this plan)

These items arrive when concrete need surfaces. Each one names
the artifact, what it would deliver, why it is deferred, and a
revisit trigger; entries are ordered roughly from substrate
outward to user surface.

- **Cargo feature gating for the Free family.** Cargo feature
  gates that let downstream crates opt out of compiling
  individual Free variants if their compile cost becomes
  uncomfortable ([decisions.md](decisions.md) section 4.4
  "Open questions left after this decision"). _Why deferred:_
  the compile cost of shipping six variants plus the
  `SendFunctor` trait family is unverified; a real
  feature-gating design needs benchmark or downstream-feedback
  evidence to be motivated. _Trigger:_ benchmark or compile-time
  evidence that the unified compile cost is meaningfully painful
  for downstream crates.
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
- **`interpret_nt`-style entry-point taking [`NaturalTransformation`](../../../fp-library/src/classes/natural_transformation.rs)
  directly.** Companion to the handler-list-driven
  [`interpret`](../../../fp-library/src/types/effects/interpreter.rs)
  shipped in Phase 3 step 2. The trait-based form lets users
  bypass the per-effect closure pattern entirely, supplying a
  rank-2-polymorphic struct impl
  (`fn transform<A>(&self, fa: <RowBrand as Kind>::Of<'_, A>) -> <MBrand as Kind>::Of<'_, A>`)
  as a single value; [`Free::fold_free`](../../../fp-library/src/types/free.rs)
  already consumes the same trait, so the internal machinery
  is largely shared. _What this is for:_ users whose
  transformation genuinely doesn't depend on `A` (e.g., a
  cross-row hoist, a logging shim that ignores the program's
  result type, or programmatic composition of transformations
  outside the per-effect handler pattern); the existing
  closure-based path is mono-in-`A` per [decisions.md](decisions.md)
  section 4.6's resolution and Phase 3 step 2's deviations
  entry, so users who want true rank-2 polymorphism currently
  have to reach for `Free::fold_free` directly with their own
  loop. _Why deferred:_ the closure-based path covers the
  common-case ergonomic surface, and shipping a parallel
  `interpret_nt` per Run wrapper would have doubled Phase 3
  step 2's interpreter surface (six wrappers x three methods x
  two paths). _Trigger:_ first user request for an
  A-polymorphic transformation that the closure path's
  mono-in-`A` constraint blocks; or a benchmark / DX motivation
  for offering trait-impl handlers as a first-class user-facing
  alternative.
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
   `fp-library/src/types/effects/coproduct.rs` (newtypes plus `impl`
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
2. **POC-to-production migration (Phases 2 + 3).** Historical
   strategy note: when this plan was written, the POC at
   `poc-effect-row/` was a separate Cargo workspace not
   integrated with fp-library's `Brand` system. The production
   migration was expected to be mostly mechanical (swap the
   stub Coyoneda for fp-library's, swap the raw Coproduct types
   for branded equivalents) with surface-area changes around the
   macro output. The actual migration landed in Phase 2 step 8
   (`effects!` / `raw_effects!` macros) and Phase 2 step 10a
   (regression baseline at
   [`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs));
   the POC workspace was deleted in step 10b.
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

- All six Run types are publicly exported from `fp-library`:
  `Run`, `RcRun`, `ArcRun` (Erased family, inherent-method only)
  and `RunExplicit`, `RcRunExplicit`, `ArcRunExplicit` (Explicit
  family, Brand-dispatched). Conversion methods
  (`into_explicit()` / `from_erased(...)`) link paired Erased
  and Explicit variants.
- The `effects!` macro accepts `effects![A, B, C]` over arbitrary
  effect types and produces a canonical row across input
  orderings; the same row composes with `CoproductSubsetter`
  permutation proofs for hand-written cases.
- `m_do!` and `a_do!` work over the three Explicit Run brands
  (`RunExplicitBrand`, `RcRunExplicitBrand`,
  `ArcRunExplicitBrand`) for first-order effect programs.
  `im_do!` (Inherent Monadic do) provides the equivalent
  monadic do-notation for the three Erased Run types via
  inherent methods, plus by-reference do-notation over
  canonical Coyoneda-headed rows on the four `Clone`-able
  wrappers (`RcRun`, `ArcRun`, `RcRunExplicit`,
  `ArcRunExplicit`) where brand-level `m_do!(ref ...)` cannot
  reach.
- Each of the six Free variants supports its promised property
  (single-shot vs. multi-shot, thread-safe, `'static` vs `'a`,
  Brand-dispatched vs inherent-method-only) with per-variant unit
  tests passing.
- The `SendFunctor` / `SendPointed` / `SendSemimonad` /
  `SendMonad` trait family ships and is used by
  `ArcFreeExplicitBrand` and `ArcRunExplicitBrand` for their
  by-value Brand impls. `ArcCoyonedaBrand` also gains
  `SendFunctor` (and downstream) impls, retroactively closing
  the gap that
  [arc_coyoneda.rs](../../../fp-library/src/types/arc_coyoneda.rs)'s
  module docs flag.
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
  `Monad`, with `SendFunctor` / etc. for the `Arc`-affected
  brand) and by-reference hierarchy
  (`RefFunctor` / `RefSemimonad` / `RefMonad`, etc.) are both
  implemented for every Brand-dispatched Free variant
  (`FreeExplicitBrand<F>`, `RcFreeExplicitBrand<F>`,
  `ArcFreeExplicitBrand<F>`) and every Brand-dispatched Run
  variant (`RunExplicitBrand`, `RcRunExplicitBrand`,
  `ArcRunExplicitBrand`); `dispatch::map` / `dispatch::bind`
  route correctly for both consuming and borrowing closures over
  these brands. The Erased family (`Free`, `RcFree`, `ArcFree`,
  `Run`, `RcRun`, `ArcRun`) does NOT participate in dispatch by
  design; users access those types via inherent methods or
  convert to the corresponding Explicit variant.
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
  - [poc-effect-row-canonicalisation.md](poc-effect-row-canonicalisation.md)
    -- findings from the (now-deleted) `poc-effect-row/`
    workspace; the workspace was migrated to
    [`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs)
    and deleted in Phase 2 step 10b.
  - [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs)
    -- `FreeExplicit` POC.
  - [fp-library/benches/benchmarks/free_explicit.rs](../../../fp-library/benches/benchmarks/free_explicit.rs)
    -- `FreeExplicit` Criterion bench.
- PureScript Run reference:
  [`purescript-run`](https://github.com/natefaubion/purescript-run).
- Comparison table for the Rust port versus PureScript Run and
  Hasura's `eff` is in [decisions.md](decisions.md) section 10.
