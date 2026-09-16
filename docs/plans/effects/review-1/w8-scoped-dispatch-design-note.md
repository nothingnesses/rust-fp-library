# W8 Scoped-Dispatch Design Note

This note consolidates the rationale for the scoped-effect dispatch
design, which is otherwise spread across `pub(crate)` trait docs in
`interpreter.rs`.
It addresses the first recommendation of `findings.md` section 5 (write a
dedicated design note with one worked example). It documents the design
as shipped; it does not propose changes. The consolidation evaluation is
the separate second half of work item W8.

## Substrate context: the dual row

A `Run<R, S, A>` program carries two effect rows through
`NodeBrand<R, S>`:

- The first-order row `R` holds algebraic effects (`State`, `Reader`,
  `Except`, `Writer`, `Choose`, ...), each `Coyoneda`-wrapped so any
  effect is a `Functor`. First-order layers are interpreted by
  `Functor::map` through `DispatchHandlers`.
- The scoped row `S` holds higher-order, around-action effects (`Catch`,
  `Local`, `Bracket`, `Span`, Writer `listen` / `censor`, ...) as raw
  brands. Scoped layers are interpreted by case analysis, not
  `Functor::map`. See
  `scoped.rs`.

The six Run wrappers (`Run`, `RcRun`, `ArcRun`, `RunExplicit`,
`RcRunExplicit`, `ArcRunExplicit`) each expose inherent `handle` / `run`
methods that loop over `peel` and dispatch one `Node` layer at a time.

## Two scoped-dispatch shapes

### Ordinary one-slot dispatch

For a scoped layer whose selected action program type is the same as the
program produced after it resumes, one type variable suffices. This is the
ordinary route:

- `DispatchScopedHandler` dispatches one handler cell.
- `DispatchScopedHandlers` walks the scoped-handler cons-list against the
  scoped row's `Coproduct` in lock-step, `Inl` to the head handler, `Inr`
  to the tail. The layer is `SBrand::Of<'a, NextProgram>`: the action
  program slot is `NextProgram` itself. See
  `interpreter.rs` cons-cell impl
  (`dispatch_scoped`).

This is the mono-in-`A` route, the same shape PureScript Run's `run`
uses internally.

### Around-action three-role split: boundary, carrier, residual

Some scoped effects must observe or transform the selected action's
_result_ before the wrapper resumes the outer continuation, and the
resumed program has a _different_ type from the selected action. Writer
`listen` is the motivating example: the selected action produces `A`, the
handler observes the writer output emitted by just that action, and the
outer continuation resumes with `(A, Log)`. One shared type variable
cannot express this without erasure. The split keeps `NextProgram` (the
program produced after the boundary resumes) independent from
`ActionProgram` (the program selected inside the scoped operation). Three
cooperating roles implement it:

- Boundary. `DispatchScopedBoundaryHandlers` is the public facade. An
  around-action constructor returns a typed boundary value that keeps the
  selected action program and the outer continuation typed separately.
  `into_scoped_boundary_parts` splits the boundary into a scoped layer
  plus a continuation carrier, and `DispatchScopedBoundaryHeadHandlers`
  uses the consumed scoped brand and its row index to walk directly to
  the member that constructed the boundary. See
  `interpreter.rs`
  (`DispatchScopedBoundaryHandlers`,
  `DispatchScopedBoundaryHeadHandlers`).
- Carrier. `DispatchScopedCarrierHandler` is the around-action handler
  cell. It consumes the selected scoped layer together with a
  `ScopedContinuation<Carrier>`, so the handler can run post-action work
  (observe, rewrite, recover, release) and then resume the saved
  continuation. `Carrier: ScopedResumeTypes` carries the typed slots that
  keep the action and the resumed program distinct. See
  `interpreter.rs`
  (`DispatchScopedCarrierHandler`, `DispatchScopedCarrierHandlers`).
- Residual. After the boundary-selected member is consumed by its
  carrier handler, `DispatchResidualScopedHandlers` walks the same
  scoped-handler list and scoped row for every _other_ member, skipping
  exactly the consumed position and using ordinary one-slot dispatch
  there. See
  `interpreter.rs`
  (`DispatchResidualScopedHandlers`).

So an around-action operation routes its own consumed member to the
carrier-aware handler and every other scoped member to ordinary dispatch.

## The invariant each role protects

The shared vocabulary lives in
`scoped_resume.rs`
(`ScopedResumeTypes`), which names four slots that must stay distinct:

- `ActionValue`: the value the selected action produces before the
  carrier resumes the action's outer continuation.
- `ActionProgram`: the peeled action program, before the outer
  continuation is reattached.
- `OperationValue`: the value the scoped operation passes into the outer
  continuation after the selected action has run.
- `OperationProgram`: the program that produces the operation value.

The load-bearing invariant is that `NextProgram` stays independent from
`ActionProgram`. Flattening them would lose the around-action boundary or
force dynamic erasure where the type system can otherwise preserve the
distinction.

## Worked example: Writer `listen`

`listen(action)` runs `action`, observes the writer log that `action`
alone emits, and resumes the outer continuation with
`(action_result, observed_log)`.

1. The `listen` constructor returns a typed boundary: the selected action
   program (producing `A`) and the outer continuation (consuming
   `(A, Log)`) are stored separately.
2. Boundary dispatch splits the boundary and routes the consumed Writer
   member to the Writer carrier handler
   (`WriterPreHandler` / `WriterPostHandler`).
3. The carrier handler interposes on the first-order `Tell` operations
   inside the selected action (the Rust analogue of heftia's
   `interposeInWith`, via `Run::interpose`, carrying row-membership
   evidence). It accumulates the action's log, then resumes the saved
   continuation carrier with `(action_value, observed_log)`.
4. Residual dispatch handles any other scoped members in the row.

An ordinary `WriterListen<NextProgram>` handler cannot do step 3: with
only the `NextProgram` slot it has no sound way to produce the observed
log, because the action's result type `A` differs from the resumed
`(A, Log)`. This is exactly why the around-action split exists.

## Per-wrapper realization

Each of the six wrappers provides its own continuation carrier and a
family-specific resume trait, because the carrier's sharing model differs
per wrapper:

- The erased wrappers (`Run`, `RcRun`, `ArcRun`) keep the pending `Free`
  continuation queue outside the selected suspended action and resume
  through raw-step dispatch. See
  `run/representation.rs`,
  `rc_run/raw_scoped.rs`,
  `arc_run/raw_scoped.rs`.
- The explicit wrappers (`RunExplicit`, `RcRunExplicit`,
  `ArcRunExplicit`) carry a typed boundary value. See
  `run_explicit/boundary.rs`,
  `rc_run_explicit/boundary.rs`,
  `arc_run_explicit/boundary.rs`.

The shared `ScopedContinuation` handle plus family-specific resume traits
(default-erased, single-shot explicit, Rc-shared, Arc-shared, and the
action-supplied variants used by lifecycle effects such as `Bracket`)
keep the around-action protocol uniform at the handler API while letting
each wrapper own its concrete carrier shape.
