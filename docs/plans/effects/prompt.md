# Agent prompt: implement the effects port

Use this prompt to start or continue implementation work on the
purescript-run port. It is self-contained: an agent given this prompt
plus a working tree at the repo root has everything it needs.

## Your role

You are a software engineer implementing the multi-phase port of
`purescript-run` into `/home/jessea/Documents/projects/rust-fp-lib/fp-library`.
The design is fixed; your job is to land code, tests, and benches
against the phased steps in
[plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md),
one step per commit, until the phase is complete or you hit a blocker.

## Current resume point

Phase 1 complete; Phase 1 follow-up both commits landed
(`WrapDrop` migration plus the `Functor` -> `Kind` relaxation);
Phase 2 complete (all 10 steps). The `poc-effect-row/` workspace
was deleted in step 10b after its tests migrated to
[`fp-library/tests/run_row_canonicalisation.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_row_canonicalisation.rs)
in step 10a.

**Phase 3 (first-order effect handlers, interpreters, natural
transformations) is the active phase.** Steps 1 and 2 shipped;
**an active blocker pauses step 3**.

**Active blocker pending resolution.** plan.md's
`Active blockers` section contains a comprehensive entry
(`Active blocker (2026-04-29): Phase 3 step 2/3 interpreter
target-monad shape`) accumulated across three commits
(`9f9e07b` introduced; `8e59bb5` widened to a three-axis
analysis; `05539af` added per-decision approaches and
trade-offs). The blocker has **five pending decisions** with
documented approaches, trade-offs, and recommendations. **The
next session's first task is to surface these decisions to
the user, get a chosen set, and only then resume
implementation.** Do not treat this as a "find the next
numbered step and start coding" situation.

The five decisions, with the recommended set:

| #   | Question                                                              | Recommendation              |
| --- | --------------------------------------------------------------------- | --------------------------- |
| 1   | Axis 1 widening: ship row-narrowing pipeline + `extract`?             | **(1.A) Full widen**        |
| 2   | Axis 3 rec/non-rec: PureScript-faithful symmetric vs asymmetric?      | **(2.C) Asymmetric**        |
| 3   | Phase 3 step ordering: renumber after axis 1 widening?                | **(3.A) Insert + renumber** |
| 4   | Phase 6+ deferred entries for `runCont` / `interpose` / algebraic-FO? | **(4.A) Defer all three**   |
| 5   | Update decisions.md?                                                  | **(5.A) Keep frozen**       |

The full analysis (~600 lines under
`### Active blockers` in plan.md) covers PureScript Run's
reference shapes, the `MonadRec` class invariant, the Rust
constraints (closure-recursion-with-borrowed-state) and their
workarounds, the three orthogonal axes (which functions,
handler shape, rec/non-rec), Phase 4 implications, heftia row
architecture clarification, decisions.md alignment, and a
summary recommendation table. Read it end-to-end before
acting.

What Phase 2 shipped (commit-log order, oldest first):

- Steps 1-3: `frunk_core` dependency + brand-aware Coproduct
  adapter; `VariantF<Effects>` Coyoneda-wrapped Coproduct row;
  `Member<E, Idx>` trait for first-order injection / projection.
- Steps 4a, 4b: Run wrappers foundation
  (`Run`/`RcRun`/`ArcRun`); Explicit family
  (`RunExplicit`/`RcRunExplicit`/`ArcRunExplicit`) with brand
  registration + `Node`/`NodeBrand` machinery.
- Step 5: inherent `pure` / `peel` / `send` core operations on
  all six Run wrappers. Discovered the HRTB-poisoning pattern
  (see Lessons below).
- Step 6: Erased -> Explicit conversion via `From` impls.
- Steps 7a, 7b, 7c.1, 7c.2a, 7c.2b: inherent
  `bind` / `map` / `ref_map` / `ref_bind` / `ref_pure`;
  shared `DoInput` parser; `im_do!` proc-macro.
- Step 8: `effects!` (public, Coyoneda-wrapped row) and
  `raw_effects!` (internal, un-wrapped row) macros at
  [`fp-macros/src/effects/effects_macro.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/effects_macro.rs)
  with shared lexical-sort helper at
  [`row_sort.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/row_sort.rs).
- Step 9: universal `*Run::lift` across all six Run wrappers
  (sub-step 9h); `SendFunctor` cascade prerequisites
  (sub-steps 9a, 9b+9e, 9c+9f); brand-level coverage
  re-evaluations on the Arc Explicit family (sub-step 9d+9g);
  `SendRefPointed` on `ArcRunExplicitBrand` via inherent-method
  delegation (sub-step 9i; the rest of the SendRef cascade is
  documented as blocked by the per-`A` HRTB-over-types limit).
  Inherent `ArcFreeExplicit::map` lands as the concrete-type
  workaround for the unreachable brand-level
  `SendFunctor::send_map`.
- Step 10a: 21 of the POC's 25 row-canonicalisation tests
  migrated to
  [`fp-library/tests/run_row_canonicalisation.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_row_canonicalisation.rs)
  with full per-test mapping in
  [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md).
- Step 10b: `poc-effect-row/` workspace deleted; doc-link
  maintenance across the four cross-referencing files.

What Phase 3 has shipped:

- **Step 1** (`82dd7bb`): `handlers!{...}` macro at
  [`fp-macros/src/effects/handlers.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/handlers.rs)
  - `nt()` builder fallback. Runtime carrier types
    (`Handler<E, F>`, `HandlersNil`, `HandlersCons<H, T>`,
    inherent `.on::<E, F>(...)` builder methods) live at
    [`fp-library/src/types/effects/handlers.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/handlers.rs).
    The macro sorts entries lexically (matching `effects!`'s
    sort) and emits a right-nested cons chain; closures stored
    inside `Handler<E, F>` carry brand identity via
    `PhantomData<fn() -> E>`. 10 integration tests in
    [`fp-library/tests/handlers_macro.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/handlers_macro.rs)
  - 6 worker-token tests + 6 inline unit tests.
- **Step 2** (`d5efe2a`): `interpret` / `run` / `run_accum`
  inherent methods on all six Run wrappers, plus the
  [`DispatchHandlers<'a, Layer, NextProgram>`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/interpreter.rs)
  trait at
  `fp-library/src/types/effects/interpreter.rs`. The trait has
  three cons-cell impls (one per Coyoneda variant: `Coyoneda`,
  `RcCoyoneda`, `ArcCoyoneda`) per the per-wrapper Coyoneda
  variant pairing rule. The methods loop on `peel`, dispatch
  each `Node::First` layer through `DispatchHandlers`, and
  panic on `Node::Scoped` (gated by
  `#[expect(clippy::unreachable, ...)]` until Phase 4 routes
  scoped layers). ArcRun's loop uses a free-function helper
  `unwrap_first<R, S, A>` to sidestep struct-level
  HRTB-poisoning of the `Node::First` pattern match (mirrors
  the `lift_node` precedent from Phase 2 step 5). 12
  integration tests in
  [`fp-library/tests/run_interpret.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_interpret.rs)
  - per-method doctests.

  **Step 2 simplified:** the M target is implicit (= self Run
  wrapper); methods return `A` directly. This is a
  Rust-pragmatic specialization that doesn't fully mirror
  PureScript Run's `<m: Monad>` shape. The active blocker
  records this and lays out the choice between reshaping
  step 2 to expose M (option (i.a) / (i.b) per the blocker
  entry) and shipping a different-shape MonadRec sibling
  alongside (option (ii)). Recommendation in the blocker
  entry: option (ii) under axis 1 widening.

**Remaining Phase 3 steps (contingent on the active blocker
resolution).** If the recommended set is confirmed, Phase 3
re-orders as:

- Step 3 (NEW): row-narrowing pipeline + `extract`. Per
  wrapper: `interpret_with::<EBrand>(handler) -> Run<R_minus_E, S, A>`
  (PureScript `runPure` / heftia `interpretWith` analog) plus
  `extract(self) -> A` for empty-row Run. New
  `DispatchOneHandler` trait variant alongside the existing
  `DispatchHandlers`.
- Step 4 (was step 3): `interpret_rec` / `run_rec` /
  `run_accum_rec` `<MBrand: MonadRec>` externally-targeted
  interpreter family in the same module
  (`fp-library/src/types/effects/interpreter.rs`).
- Step 5 (was step 4): standard first-order effect types and
  their smart constructors (`State<S>`, `Reader<E>`,
  `Except<E>`, `Writer<W>`, `Choose` (multi-shot,
  `RcRun`-only)).
- Step 6 (was step 5): `define_effect!` macro at
  `fp-macros/src/effects/define_effect.rs`.
- Step 7 (was step 6): `compile_fail` UI tests for negative
  cases.

If a different decision set lands (e.g., (1.B) no widen +
(2.A) symmetric reshape), the step ordering differs; consult
the active blocker for the alternative.

**Process for the next session:**

1. Read plan.md's `Active blockers` section end-to-end.
2. Surface the five decisions to the user with the recommended
   set; ask them to confirm or override per-decision.
3. Once decisions land, do whatever doc updates the chosen
   decisions require (e.g., if (4.A): add the three Phase 6+
   deferred entries to plan.md; if (3.A): renumber Phase 3
   steps in plan.md).
4. Then implement the chosen step 3 (and onward).
5. After implementation, move the active-blocker entry to
   resolutions.md as a top-level dated entry per the standard
   procedure; replace it with a one-line summary in plan.md's
   `Active blockers` (or remove it).

If you encounter unexpected behaviour during Phase 3
implementation, plan.md's `Active blockers` section is the
place to record additional load-bearing questions; entries
should cite concrete file paths and line numbers so the next
implementor (or you in a future session) can verify claims
without conversational context.

## Lessons learned in Phase 2 (load-bearing for Phase 3)

These were learned during Phase 2 sub-steps 5, 9a-9i, and 10a.
Reading them up front saves re-discovery cycles in Phase 3.

### Coyoneda variant pairing rule (load-bearing for Phase 3 step 4)

Each Run wrapper's `lift` uses the Coyoneda variant whose
pointer kind matches its substrate:

- `Run` / `RunExplicit` -> bare `Coyoneda`.
- `RcRun` / `RcRunExplicit` -> `RcCoyoneda`.
- `ArcRun` / `ArcRunExplicit` -> `ArcCoyoneda`.

Driver: `*Run::send` and `*Run::peel` carry per-method
`Of<'_, *Free<..., *TypeErasedValue>>: Clone` bounds intrinsic
to the shared-`Rc`/`Arc` substrate state. Bare `Coyoneda`'s
`Box<dyn FnOnce>` continuation is not `Clone`, so only the
matching shared-pointer Coyoneda variant satisfies the bound.
The plan's per-wrapper delta table for step 9h initially
specified bare `Coyoneda` for `RcRun`/`RcRunExplicit`; that
proved unsatisfiable and was corrected in step 9h's deviations
entry.

When Phase 3 step 4's smart constructors (`ask`, `get`, `put`,
`tell`, `throw`) define one-liners over `*Run::lift`, they need
to carry the same pairing in their effect-row parameterisation.
A user-facing `ask` likely needs to either (a) be parameterised
over the Run wrapper, (b) ship six variants, or (c) pick one
canonical wrapper per effect family. Resolve this design
question before writing per-effect smart constructors.

### HRTB-poisoning workaround pattern (relevant whenever Arc-substrate code is touched)

`ArcRun`'s struct-level HRTB on the `Kind` projection
(`Of<'static, ArcFree<..., ArcTypeErasedValue>>: Send + Sync`)
poisons GAT normalization in any scope mentioning the HRTB.
Constructing a `Node::First(layer)` literal inside an
`ArcRun`-impl-block scope fails to unify with
`<NodeBrand<R, S> as Kind>::Of<'_, A>` even though `impl_kind!`
declares them equal.

**Workaround**: receive projection-typed values as parameters;
never construct projection-typed values inside an HRTB-bearing
scope. The probe at
[`fp-library/tests/arc_run_normalization_probe.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/arc_run_normalization_probe.rs)
documents four passing patterns and is the regression-test home
for this limit. The free
[`lift_node`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/arc_run.rs)
helper (used by `ArcRun::lift`) is the precedent fallback. If
Phase 3 handlers / smart constructors need to construct
projection-typed values in HRTB-bearing scopes, use the same
pattern.

### Per-`A` HRTB-over-types blocks brand-level type-class delegation

Stable Rust does not support `for<T>` HRTBs. When a brand-level
type-class trait requires per-`A` bounds (`A: Clone`, per-`A`
`F::Of<...>: Clone + Send + Sync`), the bounds cannot be
expressed in the trait method signature. This is the same wall
that blocked:

- Brand-level `Functor`/`Semimonad` on `RcFreeExplicitBrand`
  (step 4b).
- Brand-level `SendFunctor`/`SendSemimonad` on
  `ArcFreeExplicitBrand` (step 9d) and `ArcRunExplicitBrand`
  (step 9g).
- Brand-level `SendRefFunctor`/`SendRefSemimonad` cascade on
  `ArcRunExplicitBrand` (step 9i).

Workaround pattern: implement the operation on the concrete
type as an inherent method (where per-`A` bounds work in the
where-clause) and document the brand-level gap in
[`fp-library/docs/limitations-and-workarounds.md`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/docs/limitations-and-workarounds.md).
The `im_do!(ref ...)` macro form already routes around
brand-level gaps via inherent-method delegation; if Phase 3
brand-level handler dispatch on Arc-substrate types hits the
same wall, use the same pattern.

### `effects!` vs `raw_effects!` distinction (relevant for Phase 3 steps 1, 6)

[`effects!`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/effects_macro.rs)
is the public macro that produces Coyoneda-wrapped Coproduct
brand rows (each variant satisfies the row-Functor requirement
because Coyoneda is unconditionally Functor regardless of its
inner).
[`raw_effects!`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/lib.rs)
at `fp_library::__internal` is the internal macro that produces
raw Coproduct brand rows (no Coyoneda wrap), used in test
fixtures and lower-level combinators.

Phase 3 handlers should generally consume rows produced by
`effects!`; Phase 3 step 6's compile_fail UI tests may use
`raw_effects!` to construct test-only edge cases.

### Bundle substrate/wrapper migrations by default (load-bearing for any future cascade work)

`ArcFree`'s bound replacement in step 9b broke `ArcRun`'s
methods at every call site; 9b had to bundle with 9e to keep
the workspace compiling. Same coupling drove the 9c+9f bundle.
**Bundle migrations across substrate/wrapper pairs by default**;
surface the bundling decision to the user as a deviations.md
entry but don't try to land the substrate half independently.
Phase 3 is unlikely to surface substrate-vs-wrapper migrations
of this shape, but the principle generalises to any
cascade-coupled refactor.

### Mechanical operational gotchas

- **`compile_fail` `.stderr` files are sensitive to
  bound-changes.** Phase 2 migrations shifted error-message
  line numbers in the
  `arc_free_explicit_bind_requires_send` UI test; regenerate
  with `TRYBUILD=overwrite cargo test --test compile_fail`
  (raw `cargo`, not `just test`, to avoid `wip/` artifacts).
  Phase 3 step 6's compile_fail UI tests will need the same
  regeneration treatment whenever bounds shift.
- **Clippy's `type_repetition_in_bounds` lint requires a
  type's bounds to be in one place.** If you split bounds
  between the generic-param-list and the where-clause (e.g.,
  `<B: 'a>` plus `B: Send + Sync` in where), clippy fails.
  Consolidate: `<B>` plus `B: Send + Sync + 'a` in where.

## Lessons learned in Phase 3 (load-bearing for remaining Phase 3 + Phase 4)

These were learned during Phase 3 step 1 (`82dd7bb`) and
step 2 (`d5efe2a`) plus the active-blocker design analysis.

### Handler-list runtime carrier shape

`HandlersNil` / `HandlersCons<H, T>` is fp-library's own
cons-list, distinct from `frunk_core::hlist::{HCons, HNil}`
already re-exported under
[`crate::types::effects::coproduct`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/coproduct.rs).
The two are intentionally different types: frunk's HList
provides type-level position witnesses for row-membership
proofs (`Here` / `There`, `CoprodInjector`); fp-library's
`HandlersCons` carries runtime handler closures aligned with
the row's value-level shape. The distinction lets inherent
`.on::<E, F>(...)` builder methods live on the handler-list
types directly without an extension-trait dance. Don't conflate
them.

`Handler<E, F>` uses `PhantomData<fn() -> E>` (variance-neutral)
rather than `PhantomData<E>`. The brand `E` is a zero-sized
marker type used purely for tagging; `fn() -> E` keeps the
newtype free of variance and `Send`/`Sync` concerns inherited
from `E` itself. Standard "phantom for tagging" idiom.

The handler list's builder uses **prepend semantics**:
`nt().on::<A, _>(ha).on::<B, _>(hb)` produces
`HandlersCons<Handler<B>, HandlersCons<Handler<A>, HandlersNil>>`
(B at head). The macro sorts lexically, so users wanting
macro-equivalent ordering should call `.on()` in
reverse-lexical order. Documented at the module level in
[`handlers.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/handlers.rs).

### `DispatchHandlers` trait + per-Coyoneda-variant impls

The
[`DispatchHandlers<'a, Layer, NextProgram>`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/interpreter.rs)
trait walks a `HandlersCons` / `HandlersNil` against the
row's value-level `Coproduct` chain in lock-step. It has
**four impls**: a base case for `HandlersNil` paired with
`CNil`, plus three cons-cell impls — one per Coyoneda variant
(`Coyoneda`, `RcCoyoneda`, `ArcCoyoneda`). The duplication is
mechanical: identical body, different `lower*` method (bare
`Coyoneda::lower` consumes self; the Rc/Arc variants ship
`lower_ref(&self)` only).

When step 3 ships row-narrowing (per the active blocker's
recommendation 1.A), expect a parallel `DispatchOneHandler<'a, Layer, NextProgram>`
trait that handles **single-effect** matching against the
row's head: it succeeds on `Coproduct::Inl(...)` and forwards
unchanged on `Coproduct::Inr(rest)`. Three Coyoneda-variant
impls again. The code reuses `DispatchHandlers`'s pattern;
diff is mostly in the matching logic.

### Closure-mono-in-`A` constraint matches PureScript Run runtime

PureScript Run's
[`interpret`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
is the rank-2 polymorphic API; its actual implementation is
`run` (literally aliased). The `run` form's handler is
`(VariantF r (Run r a) -> m (Run r a))` — mono in `a`. fp-library
adopts the mono-in-`a` form so handler closures fit Rust's
non-generic-closure constraint. Each `Handler<E, F>` cell
carries a closure of shape
`FnOnce(<EBrand as Kind>::Of<'_, NextProgram>) -> NextProgram`
where `NextProgram` is the Run wrapper specialized to the
program's result type `A`.

Rank-2 polymorphic targets reach for
[`NaturalTransformation`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/natural_transformation.rs)
directly via
[`Free::fold_free`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/free.rs).
A future `interpret_nt`-style companion entry-point is recorded
in plan.md's Phase 6+ deferred items.

### HRTB-poisoning recurs in `ArcRun`'s interpret loop

Phase 2 step 5's HRTB-poisoning workaround (`*Run::send`
takes a Node-projection value rather than constructing one
inside the impl-block scope) **recurred** in Phase 3 step 2's
`ArcRun::interpret` body. Pattern-matching `Node::First(layer)`
inside the impl-block scope failed GAT normalization
symmetrically: the projection equality declared by
[`impl_kind!`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/lib.rs)
won't normalize under the struct-level HRTB.

Workaround: a free function `unwrap_first<R, S, A>` defined
outside the impl-block scope receives the Node-projection
value and pattern-matches inside its non-HRTB scope. Sibling
to `lift_node`'s precedent. See
[`arc_run.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/arc_run.rs)'s
`unwrap_first` helper. **Other five Run wrappers do not need
this workaround** — only `ArcRun` (Erased Arc) has the
struct-level HRTB; `ArcRunExplicit` carries the `Send + Sync`
bounds per-method, not at the struct level, so it
pattern-matches inline.

If Phase 3 step 3 (row-narrowing) needs to pattern-match
`Node` projections inside `ArcRun`'s impl block, expect to
extend `unwrap_first` (or add a sibling helper) for the new
pattern shape.

### Per-wrapper Coyoneda-variant brand in test rows

Phase 3 step 2's integration tests in
[`run_interpret.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_interpret.rs)
use the wrapper-appropriate Coyoneda-variant brand at the row
level:

- `Run` / `RunExplicit`: `CoproductBrand<CoyonedaBrand<...>, CNilBrand>`.
- `RcRun` / `RcRunExplicit`: `CoproductBrand<RcCoyonedaBrand<...>, CNilBrand>`.
- `ArcRun` / `ArcRunExplicit`: `CoproductBrand<ArcCoyonedaBrand<...>, CNilBrand>`.

Per the per-wrapper Coyoneda variant pairing rule from Phase
2 step 9h. Mismatching the brand to the wrapper triggers
peel-bound or Member-bound failures (not always with clear
error messages). When writing new tests, always use the
matching brand.

The `handlers!{}` macro takes the **inner brand** (e.g.,
`IdentityBrand`) regardless of which wrapper the row targets.
The DispatchHandlers impls bind the inner brand and dispatch
on the relevant Coyoneda value variant; users don't need to
reach into `CoyonedaBrand<IdentityBrand>` syntactic form for
the macro key.

### Step 2's M-target asymmetry (load-bearing for the active blocker)

Phase 3 step 2 shipped with the M target implicit (= self Run
wrapper, returns `A`). PureScript Run's `run`/`runRec` exposes
`<m: Monad>` / `<m: MonadRec>` symmetrically. The Rust port
deviates because Rust closures can't elegantly recurse with
borrowed handler state through `m`'s `bind`. The active
blocker analyzes the trade-offs of refactoring to symmetry
(option (i.a)) vs adding an externally-targeted MonadRec
sibling alongside (option (ii)). Read the blocker before
choosing.

## Where to start

**The active blocker is non-empty as of the most recent
commits** (see "Current resume point" above). The first task
is blocker resolution, not implementation. Adjust the
sequence below accordingly.

1. **Read plan.md's `Active blockers` section end-to-end first.**
   The `Active blocker (2026-04-29): Phase 3 step 2/3
interpreter target-monad shape` entry is ~600 lines and
   carries five pending decisions. The five decisions, their
   approaches, trade-offs, and recommendations are documented
   in plan.md; read them in order. You don't need to invent
   new analysis — the prior session did that work; your job is
   to surface the decisions to the user and get a chosen set.
2. Read [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)'s
   `Current progress` section to confirm what shipped under
   `82dd7bb` (Phase 3 step 1) and `d5efe2a` (Phase 3 step 2).
   The implementation phasing sections (Phase 1 through Phase
   5, plus Phase 6+ deferred) list numbered steps within each
   phase.
3. Surface the five decisions to the user. The recommended
   set in plan.md is (1.A) full widen, (2.C) asymmetric, (3.A)
   insert + renumber, (4.A) defer all three, (5.A) keep
   decisions.md frozen. Get the user's per-decision choice
   (confirm-or-override).
4. Apply doc updates the chosen decisions require:
   - If (4.A): add three Phase 6+ deferred entries to plan.md
     (`runCont` / `interpose` / algebraic-FO handler shape).
   - If (3.A) under (1.A): renumber Phase 3 steps in plan.md
     (current 3-6 become 4-7; new step 3 = row-narrowing
     pipeline + `extract`).
   - If (5.A): nothing in decisions.md changes.
   - Land doc updates as a `docs(plan):` commit before
     starting implementation, so the implementation commit
     stays focused on code.
5. Read [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md)
   for any sections referenced by the chosen step. Sections
   4.3 (interpreter families), 4.5 (scoped effects), and 4.6
   (natural transformations) are the most relevant for the
   immediate Phase 3 work.
6. Skim relevant entries under
   [research/](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/research/)
   only if a step names them. Do not re-read the full corpus.
7. If your step touches type-class impls, brand-level dispatch, or
   `Send + Sync` auto-derive, also skim
   [fp-library/docs/limitations-and-workarounds.md](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/docs/limitations-and-workarounds.md)'s
   "Unexpressible Bounds in Trait Method Signatures" table. Phase
   1 step 7 added rows for the Explicit Free family that record
   where stable Rust's lack of `for<T>` HRTB caps brand coverage.
   The pattern (Pointed at the brand level; `bind`/`map`
   inherent-only; Ref hierarchy as the by-reference dispatch
   path) is the precedent any new wrapper type with shared
   internal state will end up following. Saves rediscovering the
   constraint mid-implementation.
8. After implementation lands and `just verify` is clean,
   move the active-blocker entry from plan.md to resolutions.md
   as a top-level dated entry per the standard procedure;
   replace the entry in plan.md's `Active blockers` with a
   one-line summary plus an anchor link to resolutions.md (or
   remove the entry). The Phase 2 step 9 resolution is a
   recent precedent for this move.

## Per-step protocol

For each step you implement:

1. Implement the code, tests, benches, or docs the step requires.
   Use the LSP tool (`rust-analyzer` is wired through MCP, see the
   project's [CLAUDE.md](file:///home/jessea/Documents/projects/rust-fp-lib/CLAUDE.md)
   for usage) for type info, go-to-definition, and find-references.
   The Brand-and-Kind machinery and the existing four-variant
   `Coyoneda` family are the long-standing templates the new code
   follows. The recently committed `Free`, `RcFree`, `ArcFree`, and
   `FreeExplicit` modules in
   `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/`
   are direct structural templates for subsequent variants in the
   Free family (e.g., the outer `Rc<Inner>` wrapping pattern in
   `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/rc_free.rs`
   and the concrete recursive enum body in
   `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/free_explicit.rs`
   together inform `RcFreeExplicit`).
2. Run `just verify` (or the individual sub-recipes: `just fmt`,
   `just check`, `just clippy`, `just deny`, `just doc`, `just test`).
3. If verification fails, fix the underlying issue. Do not bypass
   hooks (`--no-verify`, `--no-gpg-sign`) and do not silence
   warnings without addressing them.
4. Update the docs that capture state and history:
   - [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)'s
     `Current progress` section to reflect what now exists.
     Trim older entries per plan.md's
     [`Implementation protocol`](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)
     step 3 when the rolling-detail window has grown past
     ~3 narratives (demote the oldest narrative to a
     one-line bullet in the "Earlier completed steps" commit
     log; verify any load-bearing context is preserved in
     deviations.md / resolutions.md / commit message before
     demoting).
   - [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md)
     (append-only) for any per-step deviation from the original
     plan text. Group entries by phase and step, matching the
     existing structure.
   - If you encounter a blocker, add an entry to plan.md's
     `Open questions, issues and blockers -> Active blockers`
     subsection (see "When you hit something unexpected" below).
     Once the blocker resolves, move the entry to
     [resolutions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md)
     as a new top-level entry, dated; replace the active-blocker
     subsection in plan.md with a one-line summary plus an
     anchor link to resolutions.md.
5. Commit. One step per commit; the commit message describes the
   step. Use conventional-commit prefixes (`feat`, `fix`, `refactor`,
   `test`, `bench`, `docs`, `chore`). Never include `Co-Authored-By`
   trailers.

Do not skip the protocol to "batch" steps; a step is the commit
boundary, even when two steps look small.

**Splitting an oversized step is permitted but exceptional.** If
a numbered step in plan.md is large enough that landing it as
one commit would risk leaving the working tree mid-step on
context exhaustion (rough rule of thumb: ~1500+ new lines, 7+
new files, or multiple new public types with mixed concerns),
you may split it into sub-commits (e.g., 4a foundation, 4b
follow-on) under the following conditions:

1. Surface the scope to the user before starting. Explain what's
   bundled and offer the split as an option; do not split
   unilaterally.
2. The split must be coherent: each sub-commit must compile and
   pass `just verify` independently, and each must be
   independently reviewable.
3. Record the split in
   [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md)
   under the step's heading, explaining the scope rationale and
   what each sub-commit lands. Phase 2 step 4's split into 4a
   (foundation) and 4b (Explicit family) is the existing
   precedent.

The default remains "one step per commit"; splitting is for
genuinely outsized steps, not for convenience.

## When you hit something unexpected

The plan and decisions are frozen. You do not have authority to
change them unilaterally. If you encounter:

- **A step that doesn't make sense given the current code state.**
  Stop. Add an entry under
  `Open questions, issues and blockers -> Active blockers` in
  [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)
  describing what's unclear, commit that single edit, and report
  back to the user. Do not invent an interpretation.
- **A genuine design conflict** (a decision in
  [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md)
  is incompatible with what stable Rust permits, with the existing
  fp-library code, or with another decision). Same protocol: record
  it under
  `Open questions, issues and blockers -> Active blockers` in
  plan.md, commit, report back. Do not edit
  [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md)
  yourself.
- **A simpler way to do something** (refactor opportunity, missing
  abstraction, etc.). If it is in scope for the step, do it inline.
  If it would expand the step's scope or touch unrelated code, note
  it under
  [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md)
  or as a follow-up `chore:` commit; do not silently expand the step.
- **Unexpected files, branches, or in-progress work.** Investigate
  before deleting or overwriting. The user's local state is real and
  may be load-bearing; ask before discarding it.

## Boundaries

- **`/home/jessea/Documents/projects/rust-fp-lib/fp-library/` is the
  production crate.** Code, tests, and benches go here.
- **`/home/jessea/Documents/projects/rust-fp-lib/fp-macros/` holds
  proc-macros.** The effects-subsystem macros live in
  `/home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/`.
  Already shipped: `im_do!` ("Inherent Monadic do") at
  [`im_do/codegen.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/im_do/codegen.rs)
  (Phase 2 step 7c.2b); `effects!` (public, Coyoneda-wrapped
  row) and `raw_effects!` (internal, un-wrapped row) at
  [`effects_macro.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/effects_macro.rs)
  with the shared lexical-sort helper at
  [`row_sort.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/row_sort.rs)
  (Phase 2 step 8); `handlers!` at
  [`handlers.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/handlers.rs)
  (Phase 3 step 1, commit `82dd7bb`). Pending:
  `define_effect!` (Phase 3 step 5 / 6 depending on blocker
  resolution), `define_scoped_effect!` (Phase 4),
  `scoped_effects!` (Phase 4 step 4) — all land in the same
  directory. `ia_do!` ("Inherent Applicative do") is
  forward-reserved as a future applicative companion to
  `im_do!`. The shared `DoInput` parser used by all four
  do-notation macros (`m_do!`, `a_do!`, `im_do!`, future
  `ia_do!`) lives at
  `/home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/support/do_input.rs`
  (Phase 2 step 7c.2a).
- **Documentation lives in
  `/home/jessea/Documents/projects/rust-fp-lib/docs/`.** Do not
  invent new top-level docs without an explicit step asking for
  them. Phase 5 step 4 schedules
  `/home/jessea/Documents/projects/rust-fp-lib/fp-library/docs/run.md`.
- **Out-of-scope items in
  [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)'s
  `Out of scope` section** are off-limits. Surveying alternatives,
  prototyping evidence-passing, exploring tag-based type-level
  sorting, etc. are not part of this implementation effort.

## Project conventions

- **Hard tabs for Rust indentation.** The project's
  `/home/jessea/Documents/projects/rust-fp-lib/rustfmt.toml` uses
  hard tabs. When using the Edit tool, the `old_string` must match
  the file's tab characters exactly. Do not fall back to `sed`,
  `awk`, or `python` to edit whitespace.
- **No em-dashes, en-dashes, or `--` as a dash substitute.** Use
  commas or semicolons. Hyphenated words are fine.
- **No emoji or unicode symbols** in code, comments, or docs. ASCII
  only: `->`, `<-`, `>=`, `!=`, plain dashes for dividers.
- **Always end bullet points with proper punctuation.**
- **Conventional commit prefixes** (`feat`, `fix`, `docs`,
  `refactor`, `bench`, `test`, `chore`). No `Co-Authored-By`
  trailers.
- **Default to writing no comments.** Comment only when the _why_
  is non-obvious (a hidden invariant, a workaround for a specific
  bug, behavior that would surprise a reader). Never reference the
  current task, fix, or callers in comments.
- **No backwards-compatibility shims, dead code preservation, or
  removed-code comments.** Delete what is no longer used.

## Common gotchas from prior steps

These bit prior steps repeatedly. Internalising them up front saves
debug cycles.

- **Stage new files before `just verify`.** Untracked files do not
  invalidate the test-output cache, so a green verify on untracked
  code is not trustworthy. After creating new files, run `git add`
  before `just verify`. If verify reformats existing files (via
  `treefmt`), `git status` will show `MM` on the staged file; re-stage
  with `git add` before retrying the commit.
- **`#[document_examples]` requires a real Rust code block.** It
  rejects `\`\`\`ignore`, `\`\`\`text`, and other non-Rust fences.
If no working example exists for a method whose impl depends on
scaffolding from a later step, options are: (a) add a working
example that uses an existing brand which already supports the
trait (e.g., `OptionBrand`for the`Send\*`family); (b) provide
a small one-off impl alongside so the example compiles; (c)
remove the macro and use plain`# Examples`markdown, but the
resulting deprecation warning is escalated by`-D warnings`in`just clippy`, so this only works after careful suppression.
- **Inherent-method bounds do not propagate into trait impl
  bodies.** When implementing a brand-level type-class trait by
  delegating to an inherent method (e.g.,
  `RcFreeExplicitBrand::bind` -> `RcFreeExplicit::bind`), the
  inherent method's `where A: Clone, F::Of<...>: Clone` bounds are
  not in scope inside the trait method body. Stable Rust does not
  let you add per-method `where` bounds beyond what the trait
  declares (no HRTB-over-types). When this hits, the right move
  is usually documenting the brand-level coverage gap (see the
  [`limitations-and-workarounds.md`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/docs/limitations-and-workarounds.md)
  precedent) and routing through the Ref hierarchy where possible,
  not fighting the constraint.
- **`Free<IdentityBrand, A>` is layout-cyclic.** `Free`'s `Wrap`
  arm holds `F::Of<Free<F, TypeErasedValue>>` where
  `TypeErasedValue = Box<dyn Any>`. For `IdentityBrand`,
  `Identity<T>` is `T` with no indirection, so layout recursion
  has no termination and rustc rejects with
  `error[E0391]: cycle detected when computing layout`. Tests
  and benches that wrap Free over an identity-shaped functor
  must use `ThunkBrand` instead (`Thunk<A>` holds a boxed
  closure, providing the indirection). The Rc/Arc Erased family
  escapes via outer `Rc<Inner>` / `Arc<Inner>` wrapping; the
  Explicit family escapes via `Box<...>` in `FreeExplicit`'s
  `Wrap` arm or the same outer wrapping for the Rc/Arc Explicit
  variants. See deviations.md's Phase 1 step 8 entry.
  **For Run-shaped programs**: `Run<R, S, A>` (over `Free`) hits
  this cycle when `R` has a no-indirection head (e.g.,
  `IdentityBrand`); use `CoyonedaBrand`-headed rows for `Run`'s
  doctests/tests. `RcRun` / `ArcRun` / all three Explicit
  variants escape via their respective outer-pointer or
  `Box`-in-Wrap indirection, so `IdentityBrand`-headed rows
  work for them.
- **HRTB on a GAT projection poisons normalization in scope.**
  Discovered while implementing `ArcRun::send` in Phase 2 step 5. When a struct, impl block, or function's where-clause
  carries an HRTB on a generic associated type at a specific
  instantiation (e.g.,
  `NodeBrand<R, S>: Kind<Of<'static, ArcFree<...>>: Send + Sync>`,
  which `ArcFree`'s struct propagates to every `ArcRun`
  impl-block context), rustc refuses to normalize that GAT at
  _other_ instantiations in the same scope. So a literal
  `Node::First(layer)` cannot be unified with
  `<NodeBrand<R, S> as Kind>::Of<'_, A>` even though
  `impl_kind!` declares them equal. The trigger is the HRTB
  itself, not the substrate: PhantomData-only structs with the
  HRTB hit it; free functions carrying the HRTB hit it;
  cross-substrate calls (e.g., `RcFree::lift_f` from inside an
  `ArcFree`-HRTB-bearing impl) hit it. **Workaround**: receive
  projection-typed values as parameters; never construct
  projection-typed values inside an HRTB-bearing scope. The
  caller (typically test code, smart-constructor macro output,
  or top-level concrete-type code with no HRTB in scope) builds
  the projection literal and passes it in. The probe file
  [`fp-library/tests/arc_run_normalization_probe.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/arc_run_normalization_probe.rs)
  documents four passing patterns and is the regression-test
  home for this limit. This is the design driver for
  `*Run::send` taking the `Node`-projection value (rather than
  the row-variant layer) symmetrically across all six Run
  wrappers.
- **The Wrap-depth probe at
  [`fp-library/tests/run_wrap_depth_probe.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_wrap_depth_probe.rs)
  is a regression test guarding the `WrapDrop` resolution.** It
  measures structural Wrap depth across Run-shaped Free
  programs and documents that Run-typical patterns have
  structural depth at most 1, which is the property the
  `WrapDrop::None` policy relies on for soundness (effect-row
  brands like `CoyonedaBrand` / `CoproductBrand` / `NodeBrand`
  all return `None` from `WrapDrop::drop` because they do not
  materially store the inner Free; `Drop` then falls through to
  recursive drop on the layer, which is sound only as long as
  the structural Wrap depth stays bounded). If a future Phase
  2-4 change appears to invalidate the probe (e.g., new patterns
  emit deeper structural Wrap chains), pause and re-evaluate
  before shipping; the probe finding is load-bearing for the
  no-`Extract`-bound semantics.
- **`Kind!(...)` macro invocations inside `Apply!(...)` do not
  require `use fp_macros::Kind;` to be in scope.** `Apply!` is a
  procedural macro that parses the inner `Kind!(...)` syntax
  itself; the inner macro never gets invoked as a real macro,
  so rustc's unused-import analysis flags the import as dead.
  Some older test files used to carry an
  `#[expect(unused_imports)] use fp_macros::Kind;` shim to
  suppress the warning; that shim is no longer needed and was
  removed during Phase 2 step 4a (commits `9adabd5` and
  `c3712f6`). When writing a test file that uses
  `Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)`
  patterns, do not import `Kind` from `fp_macros`. If you
  separately call `<F as Kind!(...)>::Of<...>` outside an
  `Apply!` (rare; the explicit form bypasses `Apply!`), then
  the import is needed.

## Bench and compile_fail test infrastructure

Many phases add benches or `compile_fail` UI tests. The
infrastructure pointers below apply across phases; reach for
them whenever a step asks for benchmarking or negative-case
testing.

- **Criterion benches** go in
  [`fp-library/benches/benchmarks/`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/benches/benchmarks/).
  Existing per-variant Free benches
  ([`free.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/benches/benchmarks/free.rs),
  [`free_explicit.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/benches/benchmarks/free_explicit.rs),
  etc.) are the baseline shape. Wire new bench files into the
  `criterion_group!` registration in
  [`fp-library/benches/benchmarks.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/benches/benchmarks.rs).
- **`compile_fail` UI tests** go in
  [`fp-library/tests/ui/`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/ui/).
  The
  [`fp-library/tests/compile_fail.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/compile_fail.rs)
  driver registers them via `trybuild::TestCases::new().compile_fail("tests/ui/*.rs")`,
  and `trybuild = "1.0"` is already in
  `fp-library/Cargo.toml`. Each negative case is one `.rs` file
  plus a sibling `.stderr` capturing expected error output;
  `.stderr` files are auto-generated on first run via
  `TRYBUILD=overwrite cargo test --test compile_fail` (use raw
  `cargo`, not `just test`, when bootstrapping `.stderr` files
  so the wip files do not persist under `fp-library/wip/`).
- **Probe / investigation tests** can also live in
  [`fp-library/tests/`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/).
  Existing examples include
  [`run_wrap_depth_probe.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_wrap_depth_probe.rs)
  (regression-guards a property load-bearing for the WrapDrop
  resolution) and
  [`free_explicit_poc.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/free_explicit_poc.rs)
  (integration-tests `FreeExplicit` against the questions the
  POC originally asked). Use the same shape when a step's work
  benefits from a self-documenting investigation as a test.

## Tooling

- All build / test / lint commands go through `just` (the project
  has a
  [justfile](file:///home/jessea/Documents/projects/rust-fp-lib/justfile)
  that handles the Nix environment). Examples: `just verify`,
  `just test`, `just clippy`, `just doc`.
- For one-off `cargo` commands not in the justfile, prefix with
  `direnv allow && eval "$(direnv export bash)" && cargo ...` so
  the project's Nix toolchain is used. Do not silence direnv errors
  with `2>/dev/null`.
- The LSP tool (`rust-analyzer` via MCP) is the right tool for type
  info on generic-heavy code: `LSP` with `operation: "hover"`,
  `"goToDefinition"`, `"findReferences"`, `"goToImplementation"`,
  etc. See the project's
  [CLAUDE.md](file:///home/jessea/Documents/projects/rust-fp-lib/CLAUDE.md)
  for examples. Reach for it whenever you would otherwise be tracing
  trait bounds by hand across multiple files.

## Done condition for one run

You can either:

- **Complete one phase end-to-end** (every numbered step ticked,
  `just verify` clean, `Current progress` reflects the new state)
  and stop. The user reviews and starts the next phase.
- **Complete a focused follow-up commit set** (e.g., the Phase 1
  follow-up `WrapDrop` migration's two commits, or the
  Phase 2 step 4a/4b split's two commits) and stop. The user
  reviews before proceeding to the next phase step that the
  follow-up unblocks.
- **Stop at the first blocker** you cannot resolve under the
  protocol above. Commit the active-blocker entry under
  plan.md's
  `Open questions, issues and blockers -> Active blockers`,
  summarise the blocker, and exit.

Do not work through multiple phases unprompted. Phases ship together
as a single feature release, but they review separately.

## Reference map

The four-corner doc taxonomy:

- [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md):
  the active working spec. Phased steps, current progress, active
  blockers, success criteria. The authoritative answer to "what do
  I do next."
- [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md):
  frozen design rationale. The authoritative answer to "why this
  way." Do not edit.
- [resolutions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md):
  append-only post-write log of resolved blockers. Holds full
  problem statements, investigations, alternatives considered,
  and rationale for each load-bearing question that paused
  implementation. Read this when plan.md's `Active blockers`
  section points at it for context, or when "why does X work this
  way?" cannot be answered from decisions.md alone.
- [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md):
  append-only post-write log of per-step implementation choices
  that diverged from the plan text. Grouped by phase and step.
  Read this when "the code doesn't match the step description"
  needs explanation; append a new entry when your own work
  diverges.

Other reference material:

- [research/](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/research/):
  per-codebase classifications, three Stage 2 deep dives, and a
  synthesis. Source material for the decisions.
- [type-level-sorting/research/](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/type-level-sorting/research/):
  the parallel research arc on type-level sorting. Cited from
  decisions section 4.1.
- `poc-effect-row/`: standalone Cargo workspace with the
  row-encoding hybrid POC. Migrated to
  [`fp-library/tests/run_row_canonicalisation.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_row_canonicalisation.rs)
  in Phase 2 step 10a; workspace deleted in step 10b. The
  preserved findings live in
  [`docs/plans/effects/poc-effect-row-canonicalisation.md`](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/poc-effect-row-canonicalisation.md).
- [fp-library/tests/free_explicit_poc.rs](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/free_explicit_poc.rs):
  import-based integration tests for the production `FreeExplicit`.
  The POC promotion is complete (Phase 1 step 1); the file now
  exercises the type imported from
  `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/free_explicit.rs`.
- [fp-library/tests/run_wrap_depth_probe.rs](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_wrap_depth_probe.rs):
  regression test for the property the WrapDrop resolution relies
  on (Run-typical structural Wrap depth at most 1). Background
  investigation, see resolutions.md's "Resolved (2026-04-27): introduce WrapDrop trait..."
  entry.
- [CLAUDE.md](file:///home/jessea/Documents/projects/rust-fp-lib/CLAUDE.md):
  project-wide agent instructions including LSP usage.
- [AGENTS.md](file:///home/jessea/Documents/projects/rust-fp-lib/AGENTS.md):
  broader agent contract for this repo.
