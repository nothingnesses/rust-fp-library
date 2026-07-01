# Resolved blockers: effects port

This file is the post-write log of blockers and load-bearing
questions that surfaced during implementation of the effects
port and how they were resolved. Each entry is dated and stays
append-only; entries are added when blockers resolve, never
edited or removed.

The file complements [decisions.md](decisions.md) (frozen
design rationale) and [plan.md](plan.md) (active phasing and
status). Use this file when you want context on "why does X
work this way?" or "what alternatives were considered for Y?".

For per-step deviations from the original plan (smaller-grain
implementation differences that didn't require a paused
investigation), see [deviations.md](deviations.md).

## Resolved (2026-05-16): B72 Writer `listen` over NonDet duplicates pre-choice Writer output

**Disposition.** B72 surfaced while starting Phase 5 step 7.3, the
Writer-dependent NonDet semantic port from Heftia `Test/Semantics.hs`.
The focused port of `listen $ add 1 *> (add 2 $> True <|> add 3 $>
False)` showed that `runNonDet . runTell` matched the branch-local
expectation, but `runTell . runNonDet` produced global log `7` instead
of Heftia's expected `6`. The pre-choice `Tell(1)` was emitted once per
branch even though each branch should observe that prefix through
`listen` while the outer Writer handler emits it once. The failing proof
is preserved in the named git stash
`phase5-7.3-nondet-writer-heftia-failing-proof`.

**Options considered:**

- **A. Accept the current all-at-once handler semantics and expect
  `7`.** This preserves short-term implementation momentum but diverges
  from the reference semantics exactly where Phase 5 step 7.3 is meant
  to harden Writer + NonDet ordering. It also hides a likely
  prefix-sharing bug behind a test expectation.
- **B. Fix the Writer `listen` preserving path so pre-choice Writer
  operations remain outside the `Choose` split while still contributing
  to each branch's observed `listen` log.** This keeps the public
  Writer / NonDet semantics aligned with Heftia and directly addresses
  the failing behaviour. The cost is a deeper change in the preserving
  accumulation or raw scoped-dispatch path, with focused regressions
  needed before restoring the broad semantic port.
- **C. Add a public standard scoped-handler pipeline API before fixing
  this case.** This may be useful long term because users need to
  express handler order explicitly, but it does not by itself prove that
  Writer `listen` preserves first-order shape correctly under `Choose`.
- **D. Add a Writer + NonDet special-case handler.** This could make the
  port pass narrowly, but it couples two effects that should compose
  through the generic first-order/scoped protocols and would accumulate
  the kind of local debt this phase is trying to remove.

**Resolution: Option B first; keep Option C only as a follow-up if the
fix exposes a genuine public API ordering gap.** The semantic invariant
belongs in the generic composition path: a preserved Writer operation
before a choice must remain one preserved operation for outer Writer
handling, while branch-local `listen` observations include that prefix.
Proving that invariant is more important than adding another API layer.
Once the invariant is proven, revisit whether standard scoped handlers
need a separate row-narrowing pipeline surface for user ergonomics.

**Implementation outcome.** Restoring the focused proof showed that the
Writer `listen` preserving path already keeps the pre-choice `Tell`
structurally outside the `Choose`. The failing `7` came from the Rust
semantic-port helper for `runTell . runNonDet`: it implemented Writer
by adding the log in `next.map(...)`, so a multi-shot `Choose`
duplicated the addition. Heftia's `runTell` is state-threaded and
updates the Writer accumulator before invoking the continuation. The
Phase 5 step 7.3 fix changes the Rust helper to accumulate immediately
through `Rc<RefCell<SumLog>>`, giving global log `6` while preserving
the branch-local observed logs `3` and `4`.

Option A would choose compatibility with the current bug over reference
semantics. Option D would hard-code one effect pair and bypass the
generic handler protocol. Option C is valid only if Option B proves that
the existing public surface cannot express the required handler order.

**Plan amendments.** Phase 5 step 7.3 expands into concrete B72
implementation steps, all now shipped:

- 7.3.1 restores the focused proof from the named stash, validates that
  Writer `listen` prefix preservation is structural, and corrects the
  Rust `runTell . runNonDet` helper so it returns global log `6`, not
  `7`.
- 7.3.2 ports the pinned Heftia NonDet + Writer cases in
  `run_heftia_semantics.rs`.
- 7.3.3 leaves a separate scoped-handler pipeline API out of Phase 5
  because the existing surface expresses the case once the Writer
  helper models `runTell` faithfully.

## Resolved (2026-05-16): B71 Explicit boundary carrier walk over-constrains non-consumed scoped handlers

**Disposition.** B71 surfaced while adding the Phase 5 step
7.1.4d.4b focused proof for the B69 Explicit Writer `listen`
handler-list split. The proof attempted a scoped row with a Writer
`listen` boundary head and a normal Span tail. The local proof also
needs its manually constructed residual `send` layers adjusted so the
scoped layer uses the pre-`send` result shape, but the load-bearing
diagnostic is separate: the current `DispatchScopedBoundaryHandlers`
blanket walks the whole scoped-handler list through
`DispatchScopedCarrierHandlers`.

That full-list carrier walk makes every scoped handler in the row prove
it can consume the selected boundary carrier. For Writer `listen`, the
carrier is result-changing (`Action -> (Action, W)`). Ordinary
result-preserving handlers such as `SpanHandler` cannot satisfy that
carrier bound even though the boundary layer is at the Writer `listen`
head and the Span handler should only be needed later through residual
ordinary scoped dispatch.

**Options considered:**

- **A. Add result-changing carrier impls to every standard scoped
  handler that might appear beside Writer `listen`.** This is a narrow
  compile fix for the immediate Span-tail proof.
- **B. Add a member-indexed boundary-head dispatcher.** Use the
  boundary's consumed scoped brand / member index to project both the
  selected scoped layer and matching scoped-handler cell directly. Only
  that handler must implement the carrier-aware boundary protocol; all
  non-consumed handlers are used later through residual ordinary scoped
  dispatch.
- **C. Build a handler-list zipper/callback inside the carrier walk.**
  Let the recursive walk that finds the boundary head carry enough
  prefix/tail context to interpret the resumed program with a residual
  dispatcher.
- **D. Restrict Explicit Writer `listen` rows to no later ordinary
  scoped effects.** Keep the current full-list carrier obligation and
  test only the single-member Writer `listen` case.

**Resolution: Option B.** Boundary dispatch should require
carrier-aware semantics only from the scoped-handler member that the
boundary actually selected. The consumed scoped-brand / member-index
evidence already exists after B70; using it to project the boundary
head directly matches the long-term architecture better than forcing
every ordinary scoped handler to implement result-changing carriers it
will never consume.

Option A scales poorly and would keep spreading Writer `listen`'s
operation-result shape into unrelated handlers. Option C revives the
callback-heavy dispatcher shape and method-generic callback / privacy
risks that B70 avoided. Option D gives up the requirement that ordinary
scoped effects can appear after boundary resume.

**Plan amendments.** Phase 5 step 7.1.4d.4b now expands into concrete
B71 implementation steps before the full Writer `listen` suite is
restored:

- 7.1.4d.4b.0 adds an indexed boundary-head handler-list projection:
  `Here` dispatches the selected handler through
  `dispatch_scoped_carrier_head`; `There` recurses without requiring
  skipped non-consumed handlers to implement the selected carrier.
- 7.1.4d.4b.1 exposes the boundary's consumed `SBrand` / `Idx` to the
  boundary facade through `IntoScopedBoundaryParts` or an adjacent
  evidence trait, without exposing H2 carrier structs or carrier-handler
  internals.
- 7.1.4d.4b.2 rewires the `DispatchScopedBoundaryHandlers` blanket to
  use the indexed head projection instead of full-list carrier dispatch.
- 7.1.4d.4b.3 restores the focused proof preserved in `stash@{0}` and
  fixes its local residual `send` layer construction.
- 7.1.4d.4b.4 is the fallback gate if indexed boundary-head projection
  hits unsafe-code, privacy, or stable-Rust expressiveness walls.

## Resolved (2026-05-16): B70 Explicit boundary residual dispatch lacks boundary-position evidence

**Disposition.** B70 surfaced while implementing the B69 Option B
Explicit boundary handler-list split for Phase 5 step 7.1.4d.4a.
B69 requires result-changing Writer `listen` to dispatch through
`DispatchScopedBoundaryHandlers` without requiring fake ordinary
`DispatchScopedHandler` semantics for
`WriterListen<..., Action, RunExplicit<Final>>` and the Rc/Arc
parallels.

`RunExplicitBoundary`, `RcRunExplicitBoundary`, and
`ArcRunExplicitBoundary` currently store the whole scoped-row layer
(`S::Of<ActionProgram>`) plus the wrapper-owned continuation. The
boundary-aware dispatcher can find the active scoped handler by
walking the runtime `Coproduct`, but once it returns a resumed
`*RunExplicit<Final>` the outer `handle` loop has only
`S::Of<*RunExplicit<Final>>` and the original scoped-handler list. It
does not know, at the type level, which scoped-row member was already
consumed as the boundary head. A principled residual dispatcher must
skip exactly that handler cell and keep dispatch for the other scoped
row members; skipping every carrier-capable head or requiring the
boundary head to implement ordinary dispatch would either drop valid
ordinary handlers or recreate B69's fake ordinary semantics.

**Options considered:**

- **A. Add consumed-member evidence to the Explicit boundary types.**
  Extend the three boundary representations and smart-constructor
  return types with the boundary scoped brand and member index
  (`SBrand`, `Idx`, plus the projected remainder where needed). Add a
  scoped-handler-list removal/projection trait that produces the
  residual handler list for `S::Of<Final>` minus that exact member. The
  projection trait may be `#[doc(hidden)] pub` if public Explicit
  boundary methods must name it in their bounds, but it must not expose
  the H2 carrier structs or carrier-handler traits. Use `Member::project`
  to route post-boundary ordinary scoped layers: matching the consumed
  boundary member is treated as an invalid boundary-only ordinary layer,
  while the projected remainder dispatches through the residual handler
  list.
- **B. Build a handler-list zipper inside the boundary dispatch walk.**
  Make `DispatchScopedBoundaryHandlers` carry a continuation/callback
  so the recursive walk that finds the boundary head also owns enough
  prefix/tail context to interpret the resumed program with a residual
  dispatcher. This avoids adding more public boundary type parameters,
  but it makes the private carrier-dispatch traits substantially more
  complex and is likely to run into method-generic callback or privacy
  pressure.
- **C. Activate the B69 operation-result lowering fallback.** Change
  result-changing scoped operations so ordinary scoped dispatch sees an
  operation-result layer rather than the final-result layer. This makes
  ordinary `DispatchScopedHandler` semantically valid for `listen`, but
  reopens the boundary/carrier representation and is broader than the
  missing-evidence problem.
- **D. Restrict Explicit boundary `handle` to no post-boundary scoped
  layers.** Remove the ordinary scoped-handler-list obligation entirely
  and reject or panic if a scoped layer appears after the boundary
  resumes. This unblocks the narrow Writer `listen` test slice but
  violates B69's requirement to preserve ordinary scoped-layer dispatch
  after boundary resume.

**Resolution: Option A.** Make the boundary value carry the scoped-row
member evidence it already depends on semantically. That gives B69's
residual ordinary dispatch a static, auditable rule: remove exactly the
consumed boundary member and dispatch every other scoped member
normally. This is API-breaking and touches all Explicit boundary
constructor return types, but it keeps the missing type-level fact
explicit and preserves the privacy of the H2 carrier internals.

Option B keeps the boundary surface smaller but concentrates
complexity in a hard-to-read recursive dispatcher and risks a Rust
type-system wall around generic callbacks. Option C remains the
fallback if adding member evidence still cannot express the residual
path without unsafe code or public exposure of private carrier
internals. Option D is rejected because it gives up ordinary
post-boundary scoped dispatch.

**Plan amendments.** Phase 5 step 7.1.4d.4a is expanded into concrete
steps:

- 7.1.4d.4a.1 threads consumed scoped-brand / member-index evidence
  through `RunExplicitBoundary`, `RcRunExplicitBoundary`, and
  `ArcRunExplicitBoundary` plus their smart-constructor return types.
- 7.1.4d.4a.2 adds the residual scoped-handler-list projection and
  dispatch trait needed to remove exactly the consumed boundary member;
  it is public-hidden if required by public Explicit boundary bounds,
  while the H2 carrier internals remain private.
- 7.1.4d.4a.3 rewires the Explicit boundary `handle` / `run` loops to
  use the residual dispatcher after boundary dispatch resumes.
- 7.1.4d.4b proves the split with focused coverage, and 7.1.4d.4c
  remains the fallback gate before the preserved end-to-end Writer
  `listen` tests are restored.

## Resolved (2026-05-15): B69 Explicit `listen` boundary ordinary-handler obligation

**Disposition.** B69 surfaced when starting Phase 5 step 7.1.4d.5,
the end-to-end Writer `listen` tests across all six wrappers.

The default `Run`, `RcRun`, and `ArcRun` raw scoped paths can
interpret Writer `listen` because their raw continuation queues carry
the operation-result boundary: the handler converts the selected
action result plus observed log into `(Action, W)` and then reattaches
the saved continuation queue. The Explicit wrappers use typed
`RunExplicitBoundary`, `RcRunExplicitBoundary`, and
`ArcRunExplicitBoundary` values for the same result-changing boundary,
and Phase 5 step 7.1.4d.4 added carrier-aware `WriterPostHandler`
impls for that route.

The first end-to-end test slice showed a separate handler-list
obligation: the Explicit boundary `handle` methods also require the
supplied scoped-handler list to implement ordinary
`DispatchScopedHandlers` for `S::Of<RunExplicit<Final>>` and the
Rc/Arc Explicit parallels. For a result-changing `listen` brand, that
forces an ordinary `DispatchScopedHandler` obligation for
`WriterListen<..., Action, RunExplicit<Final>>`, even though the
ordinary scoped layer no longer has the operation-result continuation
needed to return `(Action, W)` semantically. The failing test slice is
preserved in the git stash named `wip writer listen e2e tests blocked
by explicit boundary ordinary handler bound`.

**Options considered:**

- **A. Add ordinary Writer `listen` handler impls that run the selected
  action, preserve its `Tell`s, and discard the observed log.** This is
  the smallest patch and would satisfy the current boundary `handle`
  bound, but it creates a misleading direct ordinary `listen`
  semantics: a manually constructed ordinary listen cell would not
  actually return the listened log.
- **B. Split the Explicit boundary `handle` obligations so boundary
  dispatch does not require a fake ordinary handler for the
  result-changing boundary head.** The handler list should dispatch
  the current boundary through `DispatchScopedBoundaryHandlers` and
  still support ordinary scoped layers that can appear after the
  boundary resumes, without requiring boundary-only heads to implement
  impossible ordinary semantics. This likely means adding a
  boundary-aware ordinary-list traversal or handler-list adapter rather
  than weakening `WriterPostHandler`.
- **C. Change result-changing scoped operations to lower into an
  ordinary operation-result layer before ordinary scoped dispatch.**
  This would make ordinary `DispatchScopedHandler` see the right
  operation result, but it is a larger representation change and risks
  reopening the boundary/carrier work that already keeps selected
  action and final continuation types separate.
- **D. Restrict Writer `listen` so the Explicit final result must
  remain `(Action, W)` or require an empty scoped row after the
  boundary.** This avoids the immediate bound in narrow cases, but it
  damages the expected `map` / `bind` ergonomics of scoped operations
  and conflicts with the result-changing boundary design.

**Resolution: Option B.** Split the Explicit boundary handler-list
obligations instead of adding fake ordinary Writer `listen` semantics.
Result-changing scoped operations should stay boundary-backed, and a
handler-list implementation detail should not force an ordinary
semantics that discards the observed log.

Option A is the short-term compatibility patch but would add the
wrong public and internal intuition for ordinary `listen`. Option C is
retained only as a fallback if the handler-list split requires unsafe
code or public exposure of private H2 carrier internals. Option D is
rejected because it weakens the boundary API's expected final-result
ergonomics.

**Plan amendments.** Phase 5 step 7.1.4d gains the following concrete
steps before the end-to-end `listen` tests:

- 7.1.4d.4a splits `RunExplicitBoundary`,
  `RcRunExplicitBoundary`, and `ArcRunExplicitBoundary` handler-list
  obligations so boundary-backed result-changing operations use
  `DispatchScopedBoundaryHandlers` without fake ordinary handler impls.
- 7.1.4d.4b proves the split with focused compile/runtime coverage for
  the three Explicit wrappers while preserving post-boundary ordinary
  scoped dispatch.
- 7.1.4d.4c is the B69 fallback gate. If Option B hits a concrete
  stable Rust, safety, or privacy wall, document it before considering
  Option C. Do not adopt Options A or D.
- 7.1.4d.5 then reapplies the preserved Writer `listen` end-to-end
  test slice and covers selected log observation, original log
  re-emission, and outer continuation ordering across all six
  wrappers.

## Resolved (2026-05-15): B68 Writer `listen` needs preserving accumulation, not consume-only accumulation

**Disposition.** B68 surfaced before Phase 5 step 7.1.4d, the
standard Writer `listen` handler across all six wrappers.

The B67 accumulation protocol used by `WriterPostHandler`
intentionally consumes selected-action `Writer::Tell` operations and
returns `(action_value, accumulated_log)`. That is correct for
post-applying `censor`, because `censor` replaces selected action logs
with one transformed aggregate. `listen` has a different contract: it
must observe the selected action's accumulated log while also making
the original selected `Tell`s visible to the surrounding Writer
handler. Re-running the selected action is invalid for Box-backed
single-shot actions and would duplicate effects for every wrapper.

**Options considered:**

- **A. Add a preserving accumulation protocol parallel to B67.** Walk
  the selected action once, accumulate matched first-order operations,
  and let the effect-specific accumulator rebuild each matched
  operation into the original row. The Writer implementation re-emits
  each original `Tell` and threads the accumulated log through the
  continuation. This protocol needs row-index evidence in the
  accumulator type so the matched operation can be lifted back into the
  same first-order row.
- **B. Reuse B67 and re-emit one aggregate `Tell`.** This is small but
  changes observable Writer semantics: a handler would see one
  aggregate log instead of the original sequence of selected `Tell`s.
- **C. Run the selected action once for accumulation and once for
  preservation.** This avoids a new traversal protocol but breaks
  Box-backed `FnOnce` actions and duplicates effects on multi-shot
  wrappers.
- **D. Write a private Writer-specific preserving traversal inside the
  standard handler module.** This can unblock `listen` without a new
  general protocol, but duplicates row projection, continuation
  preservation, and re-embedding logic that the wrapper-level
  protocols already own.

**Resolution: Option A.** Add the one-pass preserving accumulation
protocol before implementing standard Writer `listen`, with Option D
retained as the explicit fallback if the general protocol hits a
concrete stable Rust, safety, or privacy wall.

Option A adds private protocol surface and another set of
wrapper-specific bounds, but it preserves the single-pass semantics
required by Box-backed actions and keeps traversal ownership in the
wrapper substrates. Option B is easy but semantically wrong for any
handler that observes individual `Tell`s. Option C is unsound for the
single-shot representation and operationally wrong for effects with
side effects. Option D may be a practical fallback, but it compounds
the same bespoke traversal debt that B66 and B67 were created to
avoid.

**Plan amendments.** Phase 5 step 7.1.4d is split into bounded
sub-steps:

- 7.1.4d.1 adds preserving accumulation for default `Run`, `RcRun`,
  and `ArcRun`.
- 7.1.4d.2 proves preserving accumulation semantics with focused
  substrate tests.
- 7.1.4d.3 extends the protocol to `RunExplicit`, `RcRunExplicit`,
  and `ArcRunExplicit`.
- 7.1.4d.4 implements standard Writer `listen` dispatch through the
  preserving protocol.
- 7.1.4d.5 adds end-to-end `listen` tests proving selected logs are
  both observed in `(action_value, observed_log)` and re-emitted to
  the outer Writer handler in original order.
- 7.1.4d.6 is the fallback gate for activating Option D if the general
  preserving protocol hits a concrete wall.

## Resolved (2026-05-15): B67 result-changing Writer accumulation protocol for post-censor / listen

**Disposition.** B67 surfaced before Phase 5 step 7.1.4c, which
implements the post-applying standard Writer `censor` handler, and
also blocks the following `listen` handler step. These handlers need
to run a selected action, collect its first-order `Writer::Tell` logs,
and then continue with a different operation shape: post-censor resumes
with the action value after re-emitting `censor(accumulated_log)`,
while `listen` resumes with `(action_value, accumulated_log)` after
re-emitting the original logs.

The B66 same-row rewrite protocol is intentionally result-preserving:
it transforms `Writer::Tell(w, next)` into another
`Writer::Tell(w2, next)` without changing the selected action result
type. The older row-removing `interpose_with_replacer` protocol is
also result-preserving at the selected action boundary. It can replace
a matched first-order operation with a program of the same branch
result type, but it cannot turn `Run<R, S, A>` into
`Run<R, S, (A, W)>`. Post-censor and `listen` require that
result-changing accumulation shape.

**Options considered:**

- **A. Use side-effect accumulators inside the existing replacer
  protocol.** A `RefCell<W>` / `Mutex<W>` accumulator can be captured
  by a Writer replacer; each `Tell` appends into the cell and returns
  the continuation unchanged.
- **B. Add a result-changing selected-action accumulation protocol.**
  Introduce a private traversal that removes `WriterBrand<W>` from the
  selected action while threading an accumulated `W` in the action
  result, yielding `Run<R, S, (A, W)>` or the wrapper-family
  equivalent. `WriterPostHandler` and `listen` then re-emit logs and
  resume the outer continuation from that explicit operation result.
- **C. Interpret the selected action through a nested handler-list
  adapter with a custom Writer handler.** Build a handler-list path
  that consumes Writer locally and delegates all non-Writer operations
  to the inherited first-order handlers.
- **D. Add Writer-specific private post/listen interpreters over raw
  Free / RcFree / ArcFree / Explicit substrates.** Keep the machinery
  under `standard_scoped_handlers::writer`, avoiding a general
  protocol until another effect needs the same shape.

**Resolution: Option B.** Add the result-changing selected-action
accumulation protocol before implementing `WriterPostHandler` or
`listen`. This directly represents the needed action ->
operation-result -> final-result shape, preserves shared-wrapper
semantics without hidden mutable state, and aligns with the project
stance that cleaner long-term architecture wins over
status-quo-preserving patches.

Option A is the smallest patch and likely works for default
single-shot `Run`, but it is semantically weak for `RcRun` / `ArcRun`
and shared Explicit wrappers: cloned or repeated resumes can share
accumulator state across runs unless every resume can allocate a fresh
accumulator inside the program execution path. Option C is semantically
direct, but it needs handler-list subtraction or delegation machinery
that the current first-order handler API does not expose. Option D is
narrower than B and avoids handler-list refactoring, but it duplicates
traversal logic in a Writer-specific corner and repeats the
debt-accruing pattern this plan is trying to avoid.

If the Option B protocol hits a concrete stable Rust, safety, or
privacy wall, document the exact limitation and then choose between C
(broader handler-list architecture) and D (Writer-private fallback)
with that evidence in hand.

**Plan amendments.** Phase 5 step 7.1.4c now starts with a bounded
result-changing accumulation slice:

- 7.1.4c.1 adds private wrapper traversal entrypoints that consume
  `WriterBrand<W>` inside the selected action and return
  `(action_value, accumulated_log)` explicitly.
- 7.1.4c.2 proves accumulation semantics for default `Run`, `RcRun`,
  and `ArcRun` before `WriterPostHandler`.
- 7.1.4c.3 decides and implements the Explicit-family route.
- 7.1.4c.4 implements `WriterPostHandler` using the accumulation
  protocol.
- 7.1.4c.5 is the fallback gate for activating Option C or D if the
  general protocol hits a concrete wall.

## Resolved (2026-05-15): B66 pre-applying Writer censor needs same-row first-order layer rewriting

**Disposition.** B66 surfaced before Phase 5 step 7.1.4b, which
implements the pre-applying standard Writer `censor` handler.
Pre-applying `censor` must preserve each selected action
`Writer::Tell` operation while transforming its log value from `w` to
`censor(w)`. That differs from the existing Local / RefLocal / Catch
raw scoped-handler path: `RunFirstOrderReplacer`,
`RcRunFirstOrderReplacer`, and `ArcRunFirstOrderReplacer` are shaped
for consuming or replacing a matched first-order operation. Local
answers `Reader::Ask`; Catch turns `Except::Throw` into a recovery
program. Neither needs to re-emit the same operation in the original
row.

Using the existing replacer protocol for Writer would require the
effect-specific replacer implementation to call `Run::lift`,
`RcRun::lift`, or `ArcRun::lift` for arbitrary branch result `T`.
Those calls need row membership and embedding bounds at every `T`,
but stable Rust cannot express that for-all-`T` requirement in the
trait implementation's where-clause. Explicit-family wrappers have a
monomorphic `interpose` closure that can spell same-row re-emission
for one result type, but relying only on that would split the design
and leave default / shared selected-action paths without a clean
implementation.

**Options considered:**

- **A. Add a same-row first-order layer rewrite protocol parallel to
  the replacer protocol.** The wrapper traversal owns row projection,
  continuation preservation, and re-embedding; the effect-specific
  transformer only maps the matched first-order layer value, e.g.
  `Writer::Tell(w, next)` to `Writer::Tell(censor(w), next)`.
- **B. Add Writer-specific traversal helpers inside the standard
  Writer handler module.** Keep the helper private to Writer and
  manually copy the default, Rc, Arc, and Explicit wrapper traversal
  needed for this one handler.
- **C. Force the existing replacer protocol to re-emit Writer by
  adding more bounds at the implementation sites.** Try to express
  the needed `lift` bounds on the replacer implementation.
- **D. Avoid same-row pre-application and implement only
  post-applying `censor` semantics.** This sidesteps the rewrite
  problem by shrinking the standard Writer surface.

**Resolution: Option A.** Add a same-row first-order layer rewrite
protocol before implementing `WriterPreHandler`. This is the clean
long-term architecture because traversal mechanics stay in the
wrapper substrate, while Writer-specific code only describes how to
transform a matched `Writer::Tell` layer. It also creates reusable
infrastructure for future same-row transforms, such as log tagging,
tracing annotations, or metadata rewrites.

Option B is kept as a fallback if the general rewrite protocol hits a
concrete Rust type-system wall. It would preserve progress but at the
cost of duplicated traversal logic and a Writer-specific special case.
Option C is not expected to work because it runs into the same
for-all-`T` bound problem that motivated B66. Option D is not
acceptable because B61/B64 deliberately selected both pre- and
post-applying standard Writer handlers.

**Plan amendments.** Phase 5 step 7.1.4b now starts with a bounded
same-row rewrite protocol slice:

- 7.1.4b.1 adds default, Rc, and Arc traversal entrypoints parallel
  to the replacer protocols.
- 7.1.4b.2 proves same-row `Writer::Tell` transformation and
  continuation preservation before `WriterPreHandler`.
- 7.1.4b.3 decides and implements the Explicit-family route.
- 7.1.4b.4 implements `WriterPreHandler` using the rewrite path.
- 7.1.4b.5 is the fallback gate for activating private
  Writer-specific helpers if Option A hits a concrete wall.

## Resolved (2026-05-15): B65 Box-backed Writer censor transform callability

**Disposition.** B65 surfaced before Phase 5 step 7.1.4b, which
implements the pre-applying standard Writer `censor` handler.
`Run::censor` and `RunExplicit::censor` currently store the
Box-backed `censor` transform as `Box<dyn FnOnce(W) -> W>` via
`BoxWriterCensor`. Pre-applying `censor` matches PureScript Run's
`censorAt` shape: it interposes `WriterBrand<W>` inside the selected
action and rewrites every encountered `Tell(w)` to
`Tell(censor(w))`. A selected single-shot action can still contain
zero, one, or many first-order `Tell` operations, so the transform must
be callable once per encountered log.

`FnOnce` is sufficient for post-applying semantics, where the handler
accumulates the action log and applies `censor` once to the aggregate,
but it is not sufficient for pre-applying semantics. Rc-backed and
Arc-backed `WriterCensor` cells already store reusable `Fn(W) -> W`
transforms. The specific Box ownership concern was checked with a
focused POC, preserved in git stash message `preserve box writer
censor reusable fn POC`: `Box<dyn Fn(W) -> W>` is single-owner but
callable repeatedly by that owner, while the selected action remains
single-shot as `Box<dyn FnOnce(()) -> A>`.

**Options considered:**

- **A. Change Box-backed `WriterCensor` to store `Fn(W) -> W`.**
  Update `BoxWriterCensor` and the `Run` / `RunExplicit` `censor`
  constructors to require a reusable transform. Keep the selected
  action thunk single-shot.
- **B. Split Box-backed pre and post `censor` cells.** Keep the
  current `FnOnce` cell for post-applying `censor`, add a distinct
  reusable-transform cell for pre-applying `censor`, and expose
  separate constructor paths.
- **C. Implement pre-applying `censor` only for Rc / Arc families.**
  Leave Box-backed `Run` and `RunExplicit` with post-applying support
  only.
- **D. Take the `FnOnce` transform on the first `Tell` and fail on any
  later `Tell`.** This preserves the current type but makes valid
  multi-`Tell` programs fail at runtime.

**Resolution: Option A.** Migrate only the Box-backed Writer `censor`
transform to reusable `Fn(W) -> W`. This is not a general migration of
Box-backed scoped actions: selected actions remain single-shot
`FnOnce` thunks. The change is a narrow API tightening for the log
transform because pre-applying `censor` is semantically a reusable log
transformation. Closures that consume non-reusable captured state no
longer fit the Box-backed `censor` constructor, but accepting those
closures would make multi-`Tell` programs impossible to handle
correctly.

Option B preserves the existing Box post-applying shape but adds more
brands, constructors, handler impls, and user-facing choices before
the standard Writer surface is proven. Option C creates a
wrapper-family semantic gap. Option D encodes a type-system mismatch
as a runtime panic and is not acceptable.

**Plan amendments.** Phase 5 step 7.1.4b.0 is now concrete: migrate
`BoxWriterCensor` and the Box-backed `Run::censor` /
`RunExplicit::censor` constructors from `FnOnce(W) -> W` to reusable
`Fn(W) -> W`, update examples and tests to cover more than one `Tell`,
then implement the pre-applying standard Writer handler against that
contract.

## Resolved (2026-05-15): B64 standard Writer handler accumulation contract and surface

**Disposition.** B64 surfaced before Phase 5 step 7.1.4, which adds
explicit pre- and post-applying standard Writer handlers. The
neutral scoped Writer constructors from B61-B63 intentionally do not
bake in accumulation policy, but the standard handlers need a generic
log accumulation contract.

Pre-applying `censor` can be implemented by interposing
`WriterBrand<W>` inside the selected action and rewriting each
`Tell(w)` to `Tell(censor(w))`; it does not need to aggregate logs.
Post-applying `censor` and `listen` do need aggregation:
post-applying `censor` must confiscate the selected action's `Tell`s,
append them, apply the censor function once, and re-emit the
transformed aggregate; `listen` must append the selected action's
`Tell`s, re-emit the original `Tell`s so the outer Writer handler
still sees them, and resume the outer continuation with `(A, W)`.
The first-order `Writer::Tell` operation intentionally has no
`Monoid` bound, but these standard scoped Writer handlers need a
generic accumulation contract.

**Options considered:**

- **A. Use the existing `Monoid` / `Semigroup` type classes for
  standard accumulation.** Standard Writer handlers require
  `W: Monoid + Clone` where aggregation is needed, plus `Send + Sync`
  on Arc-family paths. Handler constructors stay zero-sized apart from
  row witnesses:
  `writer_pre_handler::<Idx, RMinusWriter, EmbedIndices>()` and
  `writer_post_handler::<Idx, RMinusWriter, EmbedIndices>()`.
- **B. Store explicit `empty` / `append` functions in each handler
  value.** This supports per-handler accumulation for log types that
  do not implement `Monoid`, but makes standard handler values
  closure-bearing, adds lifetime and `Send + Sync` propagation, and
  increases the public API surface.
- **C. Add a separate collector-style Writer handler that returns
  `(A, W)` or a `Pair` instead of re-emitting `Tell`s.** This gives a
  pure collection API, but it does not match Heftia `listen` semantics
  because the action's `Tell`s would no longer remain available to
  the outer `Tell` handler. It also pushes the design toward a broader
  `run_writer` target-monad API rather than the current scoped-handler
  step.
- **D. Ship only the pre-applying `censor` handler now and defer
  post-applying `censor` / `listen`.** This is the smallest local
  implementation, but it contradicts B61's resolution and leaves the
  Heftia Writer semantic port blocked.

**Resolution: Option A.** Standard Writer scoped handlers use the
existing `Monoid` / `Semigroup` classes for accumulation. Writer
aggregation is monoidal by definition, and the project already has
those classes for this exact abstraction. This keeps the standard
handlers small, typed, and predictable while preserving the neutral
scoped-operation constructors from B61.

The pre handler's `censor` implementation keeps the smallest bounds it
needs; the `listen` implementation and post handler paths add
`W: Monoid + Clone` only where they aggregate or re-emit logs. Arc
paths add `Send + Sync` as usual. If a later user needs per-handler
custom accumulation, add a separately named explicit accumulator
handler rather than complicating the standard `writer_pre_handler` /
`writer_post_handler` API.

**Plan amendments.** Phase 5 step 7.1.4 now owns the concrete
Monoid-based standard handler rollout: add
`standard_scoped_handlers::writer`, export zero-sized pre/post handler
constructors with row-witness parameters, implement pre-applying
`censor`, implement post-applying `censor`, implement `listen` on the
same accumulation contract, and keep non-`Monoid` custom accumulation
out of the standard API.

## Resolved (2026-05-15): B63 Writer `listen` needs a result-changing boundary / carrier stage

**Disposition.** B63 surfaced after the Phase 5 step 7.1.1 scoped
Writer substrate shipped. The substrate now preserves `listen`'s
selected action result type separately from the log type, but the
existing boundary / carrier resume protocol still mostly assumes
result-preserving around-action work.

The mismatch blocked the then-current Phase 5 step 7.1.2 (`listen`
and `censor` smart constructors) and the following standard Writer
handler step. `censor` is same-result: the selected action result and
the scoped operation result are both `A`. `listen` is result-changing:
the selected action returns `A`, the handler observes the action's log
`W`, and the scoped operation result is `(A, W)`. The constructor
cannot synthesize `W`, and a dummy outer continuation such as `A ->
(A, W)` would either bake handler semantics into the constructor or
require an unreachable placeholder.

**Options considered:**

- **A. Generalize the private boundary / carrier protocol to an
  action -> operation-result -> final-result shape.** Add an
  operation-result type between the selected action result and the
  mapped/bound final result. Existing same-result scoped effects set
  `Operation = Action`; Writer `listen` uses `Action = A` and
  `Operation = (A, W)`. Boundary continuations then resume from
  `Operation -> Final`, while handlers can run the selected action,
  build the operation result, and then resume the wrapper-owned outer
  continuation.
- **B. Add a WriterListen-specific boundary / carrier path.** Keep the
  existing generic boundary protocol unchanged and add dedicated
  `listen` constructors plus handler dispatch that know how to bridge
  `A` to `(A, W)`.
- **C. Fall back to Bracket-style result-specific `listen` brands.**
  Store enough result information in the scoped row brand to bypass the
  generic indexed-boundary protocol for `listen`.
- **D. Defer `listen` constructors and ship only `censor` first.**
  Complete the same-result Writer operation while postponing the
  result-changing operation.

**Resolution: Option A.** Generalize the private boundary / carrier
protocol before adding `listen` constructors. This follows the
project's long-term architecture stance: the H2-style carrier work
already exists so higher-order operations can keep selected action
programs separate from outer continuations, and `listen` shows that the
missing abstraction is an explicit operation-result stage rather than a
Writer-specific workaround.

Option A is the widest refactor, touching the Explicit boundary
families, family-specific carrier traits, and standard handlers, but it
directly models the three values that exist in result-changing
higher-order operations: selected action result, operation result, and
final mapped/bound result. Option B is narrower but creates a parallel
one-off path for the first operation whose result differs from its
selected action. Option C is retained only as the B62 fallback if the
generic protocol hits a concrete Rust compiler, safety, or privacy
wall. Option D is rejected because the scoped Writer rollout should not
ship around the Heftia-relevant `listen` operation.

**Plan amendments.** Phase 5 step 7.1.2 is now the private
boundary/carrier generalization step. The smart-constructor step moves
to 7.1.3, standard Writer handlers move to 7.1.4, Heftia `listen`
semantics move to 7.1.5, and focused tests move to 7.1.6.

## Resolved (2026-05-15): B62 scoped Writer `listen` substrate representation

**Disposition.** B62 surfaced while preparing Phase 5 step 7.1.1's
scoped Writer substrate. B61 resolved the semantic surface: `listen`
and `censor` are neutral scoped Writer operations, and standard
handlers carry the pre- vs post-applying interpretation policy. The
remaining substrate question was representation, specifically for
`listen`.

`censor` is structurally Span-like: the selected action and the outer
operation have the same result type. `listen` is not: the selected
action returns `A`, while the outer operation returns `(A, W)`. That
makes `listen` another action/final split, the same class of problem
the Phase 4 H2 carrier and Explicit boundary work was introduced to
handle.

**Options considered:**

- **A. Model `listen` as an indexed around-action operation and
  `censor` as a same-result around-action operation.** Give `listen` a
  substrate shape that names `Action`, `Final = (Action, W)`, the
  selected action program, and the wrapper-owned outer continuation
  explicitly. For default and shared Erased wrappers, use the existing
  raw-step / `ScopedContinuation` route. For Explicit wrappers, use the
  existing indexed boundary route rather than trying to reconstruct the
  action/final split from an ordinary `RunExplicit<Final>`. Keep
  `censor` simpler because its action result and final result are the
  same.
- **B. Model `listen` with Bracket-style result-specific brands that
  erase the GAT-filled `X` slot.** Carry `Action` and `W` in the brand,
  store the action program directly, and let standard handlers know
  that the operation result is `(Action, W)`.
- **C. Encode `listen` by rewriting the action to produce `(A, W)` at
  constructor time.** This would make the cell fit the same-result
  shape, but it requires log capture before the handler has selected
  pre/post Writer semantics and therefore bakes interpretation policy
  into the constructor.
- **D. Defer `listen` and implement only `censor` first.** This
  unblocks a narrow subset of scoped Writer, but leaves Heftia
  `WriterH` incomplete and weakens the pinned Writer semantic port.

**Resolution: Option A.** Use the existing around-action carrier /
indexed-boundary architecture for `listen`, while keeping `censor` in
the simpler same-result around-action shape. This matches the
project's long-term architecture stance: the H2 carrier and indexed
Explicit boundary work already exists so around-action effects can keep
the selected action program and final program result separate without
unsafe erasure or hidden continuation rewriting.

The trade-off is more substrate plumbing and row-brand specificity for
`listen`, but it prevents another compatibility patch when
`listen(...).map(...)`, `listen(...).bind(...)`, or Explicit wrapper
programs need the action/final split preserved. Option B stays on file
only as a fallback if Option A hits a concrete Rust compiler, safety,
or privacy wall. Option C is rejected because it would put handler
semantics in the constructor. Option D is rejected because the scoped
Writer rollout should not ship without the Heftia-relevant `listen`
operation.

**Plan amendments.** Phase 5 step 7.1.1 now expands into concrete
sub-steps: add same-result `censor` cells, add indexed `listen` cells,
route `listen` through the existing carrier/boundary machinery, keep
Bracket-style result-specific brands as fallback only, and add focused
substrate tests for same-result `censor` plus `listen` action/final
preservation through `map` / `bind` and Explicit indexed-boundary
construction.

## Resolved (2026-05-15): B61 scoped Writer semantics selection

**Disposition.** B61 surfaced before Phase 5 step 7.1, which adds
higher-order Writer semantics (`listen` and `censor`) on top of the
existing first-order `Writer::Tell` operation. The reference systems do
not expose one unambiguous `censor` ordering:

- PureScript Run's `Run.Writer.censorAt` rewrites each encountered
  `Writer w` operation while the action is walked, so the censor
  function applies to each emitted `tell` payload before log
  accumulation.
- Heftia exposes both `runWriterHPre` and `runWriterHPost`. Its Writer
  test distinguishes the two: an action that tells `"Hello"` and then
  `" world!"` yields `"Goodbye world!"` under pre-applying semantics
  and `"Hello world!!"` under post-applying semantics.
- Heftia's `listen` observes the log produced by the action while
  leaving the underlying `Tell` effects available to the outer `Tell`
  handler.

The same review pass also found a sequencing issue: the planned NonDet
and Writer semantic port depends on a real `Empty` effect, because the
target Heftia row contains `Empty`, `ChooseH`, `Tell`, and `WriterH`
separately.

**Options considered for scoped Writer semantics:**

- **A. Ship PureScript-compatible pre-applying `censor` only.** This is
  the smallest extension of the current Writer shape and preserves the
  PureScript Run behaviour, but it drops Heftia's post-applying handler
  and makes a later post handler feel bolted on.
- **B. Model scoped Writer operations neutrally and ship explicit pre-
  and post-applying standard handlers.** `listen` and `censor`
  constructors remain semantic operations; the selected handler decides
  whether `censor` transforms each `Tell` before accumulation or the
  action's accumulated log after confiscating the action's `Tell`s.
- **C. Ship post-applying `censor` only.** This matches the traditional
  `Control.Monad.Writer.censor` intuition, because it transforms the
  output of the whole action. It diverges from PureScript Run's
  `censorAt` and drops Heftia's pre-applying handler.
- **D. Defer scoped Writer and implement `Empty` / NonDet first.** This
  avoids choosing Writer semantics immediately, but leaves the
  Writer-completeness gap unresolved and blocks the Writer-dependent
  Heftia semantic ports.

**Options considered for NonDet + Writer sequencing:**

- **A. Move `Empty` before the NonDet + Writer semantic port.**
  Implement `Empty` as the next NonDet primitive after scoped Writer
  lands, then port the Heftia semantic case against the real `Empty` +
  `Choose` surface.
- **B. Temporarily simulate `Empty` in the semantic port.** This keeps
  the previous numbering but weakens the acceptance test and requires
  rewriting the test when `Empty` lands.
- **C. Fold `Empty` into `Choose::Alt`.** This minimizes new surface
  area but contradicts the cleaner reference-system split and makes
  failure indistinguishable from choice.

**Resolution: Writer Option B and sequencing Option A.** Keep the
scoped Writer operations neutral, add explicitly named pre- and
post-applying standard handlers, and move `Empty` before the NonDet +
Writer semantic-port step. Do not add an ambiguous `writer_handler()`
default alias while the pre/post choice is still semantically
load-bearing. Use names that carry the ordering, for example
`writer_pre_handler()` and `writer_post_handler()`.

This matches the project's long-term architecture stance. The effect
constructors describe the semantic operation, while handler names carry
the interpretation policy. Both reference behaviours remain available
and testable without a later API break. Moving `Empty` earlier avoids a
temporary or simulated semantic test and lets the Heftia row be ported
faithfully.

**Plan amendments.** Phase 5 step 7.1 now expands into concrete
sub-steps for the scoped Writer substrate, wrapper smart constructors,
explicit pre/post standard handlers, Heftia-compatible `listen`
semantics, and pinned tests distinguishing the pre and post outputs.
Phase 5 step 7.2 now implements `Empty` as a separate first-order
effect. Phase 5 step 7.3 now ports the NonDet + Writer semantic cases
after both scoped Writer and `Empty` exist.

## Resolved (2026-05-14): B60 Rc/Arc Local repeated-use raw scoped dispatch can re-enter with the wrong erased result shape

**Disposition.** B60 surfaced while starting Phase 5 step 4's
cross-cutting composition matrix. The attempted matrix cloned and
interpreted an `RcRun::local(...).map(...)` program twice to cover
repeated shared-wrapper scoped dispatch. The first interpretation
panicked in `RcFree::bind` with `Type mismatch in RcFree::bind`; the
Arc half of the same matrix did not run because the Rc assertion failed
first.

The failing attempt is preserved in the named stash
`preserve Phase 5 step 4 composition matrix exposing Rc Local repeated-use mismatch`.
The reproducer is small: a shared `RcRun` Local action answers
`Reader::Ask` with the modified environment, then an outer map resumes
after scoped dispatch. Existing Span repeated-use tests pass, and
existing Local tests cover single interpretation only. That points at
the raw Local dispatcher / shared erased-Free interpose path rather
than shared wrapper cloning in general.

**Options considered:**

- **A. Generalize the B59 result-polymorphic replacement protocol to
  RcRun and ArcRun raw scoped dispatch.** Add shared-wrapper
  `RunFirstOrderReplacer`-style protocols or wrapper-specific
  equivalents so Local / RefLocal / Catch raw dispatch can rewrite
  selected branch programs at the branch result type before reattaching
  shared erased continuation queues.
- **B. Patch only Rc/Arc Local raw dispatch normalization.** Keep the
  current closure-taking shared `interpose` API and adjust the Local
  raw dispatcher to use a different cast/rebox/continue sequence for
  `RcTypeErasedValue` / `ArcTypeErasedValue`.
- **C. Scope Phase 5 step 4 repeated-use coverage to Span only and
  leave Local for later.** This would allow the matrix to land quickly
  but would hide a concrete failure in an already-shipped standard
  scoped handler.

**Resolution: Option A.** Generalize the B59 result-polymorphic
replacement protocol to shared-wrapper raw scoped dispatch. Start with
the preserved Rc Local reproducer, prove the path on `RcRun`, then
apply the same shape to `ArcRun`. After Local passes on both shared
wrappers, migrate or audit RefLocal and Catch because they use the same
family of shared erased-Free interpose/reattach operations.

This is the only option that matches the project's architecture stance.
The failure is another branch-result / continuation-shape issue, so a
Local-only rebox patch would likely compound the same debt B59 removed
from default `Run`. If a shared-wrapper-generic protocol hits a
concrete Rust type-system wall, fall back to the narrow normalization
patch with the limitation documented; do not remove Local from the
Phase 5 composition matrix.

**Plan amendments.** Phase 5 step 4 now starts with concrete B60
implementation sub-steps: prove the shared-wrapper replacement protocol
on `RcRun`, extend it to `ArcRun`, migrate/audit shared-wrapper Local /
RefLocal / Catch raw dispatchers, then restore the preserved
cross-cutting composition matrix.

## Resolved (2026-05-14): B59 `Run::interpose` needs a result-polymorphic replacement protocol before it can be boundary-aware

**Disposition.** B59 surfaced during the Phase 5 step 2.16 audit after
`Run::interpret_with_handler` was made boundary-aware. The audit showed
that default `Run::interpose` still recurses through `peel()`. That is
unsafe for Box-backed scoped boundary frames for the same structural
reason B58 closed for `interpret_with_handler`: `peel()` lowers the
boundary to a public `Free` view and can copy the same single-shot
continuation into both the protected action and recovery/handler
branches.

Unlike `interpret_with_handler`, the current `interpose` replacement
closure is monomorphic in the final outer result type `A`:

```rust,ignore
Fn(EBrand::Of<Run<R, S, A>>) -> Run<R, S, A>
```

Boundary action/recovery slots and raw continuation outputs are stored
as `Run<R, S, TypeErasedValue>` / `RawRunFree<R, S>` while the final
program may be `Run<R, S, A>`. A final-`A` closure cannot soundly
replace selected effects inside those raw branches. Making the existing
closure path boundary-aware would either skip branch rewriting or
reattach the outer continuation before branch selection, both of which
preserve the single-shot duplication bug.

**Options considered:**

- **A. Add a result-polymorphic first-order replacement protocol and
  migrate general default `Run::interpose` to it.** Introduce a trait
  analogous to `RunFirstOrderHandler`, for example
  `RunFirstOrderReplacer<EBrand, R, S>`, with a generic `replace<T>`
  method. Use it for scoped-row-capable `interpose`, and keep the
  closure-taking `interpose` only for first-order-only
  `Run<R, CNilBrand, A>` programs where branch and final result shapes
  cannot diverge.
- **B. Add a private `TypeErasedValue`-only raw interpose helper for
  the standard Box-backed scoped dispatchers, and leave the public
  closure-taking `interpose` unchanged.** This is smaller and would
  unblock Catch / Local / RefLocal raw dispatch, but it leaves a public
  API path that can still duplicate single-shot continuations when a
  user calls `interpose` on a boundary-backed scoped-row program.
- **C. Keep `interpose` as-is and document scoped-row boundary usage as
  unsupported.** This avoids immediate API churn but conflicts with the
  API stability stance: the known unsafe shape remains available, and
  future semantic ports can trip it in less obvious ways.
- **D. Try to solve this by changing boundary frames to typed internals
  only.** Typed boundary internals can improve diagnostics, but they do
  not remove the need for a replacement function callable at each
  branch result type. This is a fallback or complementary refinement,
  not the core fix.

**Resolution: Option A.** Add a result-polymorphic first-order
replacement protocol and route general scoped-row default
`Run::interpose` through it. The closure-taking `interpose` convenience
is kept only for first-order-only `Run<R, CNilBrand, A>` programs,
matching the earlier `interpret_with` split.

This is the only option that aligns with the project's long-term
architecture preference. It makes the type system represent the actual
requirement: scoped-row `interpose` needs a replacement that is generic
in the current branch result type. The API break is acceptable because
preserving the old general closure surface would retain a known
single-shot continuation hazard.

**Implementation sequencing.** Phase 5 step 2.16 now implements the
adopted path:

1. Add `RunFirstOrderReplacer<EBrand, R, S>` with a generic
   `replace<T>` method returning `Run<R, S, T>`.
2. Add a general scoped-row `Run::interpose_with_replacer` that matches
   on the private representation, rewrites boundary raw branches and
   continuation queues at `TypeErasedValue`, and uses
   `Free::continue_from_reboxed_erased` after `erase_type` just like
   Phase 5 step 2.15.
3. Move or constrain the closure-taking `Run::interpose` convenience so
   it is only available for `Run<R, CNilBrand, A>` first-order-only
   programs.
4. Update the default `Run` Catch / Local / RefLocal raw scoped
   dispatchers to use the result-polymorphic replacement protocol or an
   adapter that is explicitly valid for their raw `TypeErasedValue`
   branch shape.
5. Add regressions covering State-before-Catch / Reader-before-Local
   with a mapped or bound outer result, plus a direct test proving a
   public boundary-backed `Run` no longer uses the `peel()` interpose
   path.

**Plan-text amendment.** B59 moved out of Active Blockers. Phase 5 step
2.16 now describes the adopted implementation work, and the next
greenfield handoff points at that concrete step.

## Resolved (2026-05-14): B58 `Run::interpret_with` needs a result-polymorphic first-order handler protocol before it can rewrite boundary-backed scoped actions

**Disposition.** B58 surfaced while preparing Phase 5 step 2.13. The
B57 broad default `Run` representation is necessary because it keeps a
Box-backed scoped boundary's selected action/recovery program separate
from the pending outer continuation queue. It is not sufficient by
itself. Boundary-backed Catch frames can store branch programs whose
pre-continuation result type differs from the final outer result type
after `map` or `bind`.

The current public `Run::interpret_with` handler shape is monomorphic
in the final program result `A`:

```rust,ignore
Fn(EBrand::Of<Run<RMinusE, S, A>>) -> Run<RMinusE, S, A>
```

That shape can rewrite ordinary Free-backed programs, where each
recursive step has the same final result type. It cannot rewrite a
selected Catch action/recovery program before the pending continuation
queue runs when that selected branch has a different intermediate
result type. Converting through the public `peel()` / `Free`
compatibility view would recover the final-`A` handler type only by
attaching the continuation queue first, which reproduces the
single-shot continuation duplication and State-before-Catch ordering
hole Phase 5 step 2 is meant to close.

**Options considered:**

- **A. Keep the monomorphic closure API and attach the continuation
  before rewriting boundary-backed branches.** This is the smallest
  change, but it preserves the semantic bug and makes the
  boundary-aware representation mostly cosmetic for `interpret_with`.
- **B. Add a result-polymorphic first-order handler protocol.** Replace
  the internal `interpret_with` recursion with a handler object or
  trait whose method is generic in the branch result type, for example
  `handle<T>(&self, EBrand::Of<Run<RMinusE, S, T>>) -> Run<RMinusE, S, T>`.
  The same handler can then narrow ordinary Free steps, selected Catch
  action/recovery programs, and pending continuation programs without
  forcing every branch to have the final result type. The cost is an API
  and ergonomics change: ordinary closures cannot implement a method
  generic over every `T`, so common handlers likely need small structs,
  helper constructors, or a macro layer.
- **C. Store more typed boundary internals and keep the current closure
  API.** Retaining the branch result type inside the boundary frame
  helps with downcasts and diagnostics, but it does not solve the
  handler problem. Once the outer result differs from the branch result,
  the first-order handler still has to run at both result types.
- **D. Special-case known standard handlers.** State, Reader, or Except
  could grow bespoke boundary-aware rewrite code. This would unblock a
  narrow regression, but it would fragment the generic row-narrowing
  story and make custom first-order effects second-class.

**Resolution: Option B.** Add a result-polymorphic first-order handler
protocol before reimplementing boundary-aware default
`Run::interpret_with`. This follows the project-wide API stability
stance: preserve the intended long-term architecture even if it breaks
the current closure surface. Option C remains on file only as a
complementary representation or diagnostic improvement if the
polymorphic protocol exposes brittle erased-boundary internals.

**Implementation sequencing.** Phase 5 step 2 now continues by
prototyping the result-polymorphic handler protocol for default `Run`,
then migrating `Run::interpret_with` internals and public surface
deliberately, then reimplementing boundary-aware branch rewriting with
State-before-Catch coverage. `Run::interpose` is re-audited under the
same handler-shape constraint before the Heftia semantic-port
acceptance suite is restored.

**Plan-text amendment.** B58 moved out of Active Blockers. Phase 5
steps 2.13-2.18 now describe the adopted handler protocol, the
boundary-aware `interpret_with` rollout, the `interpose` re-audit, the
Heftia semantic-port restore, and the typed-boundary-internals fallback
status.

## Resolved (2026-05-14): B57 B56 targeted rewrite requires the broad default `Run` boundary representation fallback

**Disposition.** B57 surfaced while preparing Phase 5 step 2.10.
B56 adopted a targeted continuation-aware default
`Run::interpret_with` rewrite path, but the code audit showed that the
targeted path still needs to rewrite Box-backed selected
action/recovery programs at their intermediate action result type
before the saved outer continuation queue is attached.

The current `Run::interpret_with` handler shape is intentionally mono
in the public final result type:

```rust,ignore
Fn(EBrand::Of<Run<RMinusE, S, A>>) -> Run<RMinusE, S, A>
```

That closure can only handle first-order operations after the outer
continuation queue has already turned a selected action's intermediate
result into the final `A`. Reattaching the queue first recovers the
handler type, but it is exactly the BoxCatch continuation duplication
that B56 was meant to remove. Switching to `Free::into_raw_step` keeps
the queue outside the scoped layer, but then selected action/recovery
branches must be rewritten at their own action result type, which the
current closure API cannot express.

**Options considered:**

- **A. Force the targeted B56 rewrite through the current closure API
  with BoxCatch-specific erasure.** This preserves the current public
  surface, but either relies on fragile dynamic erasure or reattaches
  the outer continuation before branch selection and reproduces the
  known bug.
- **B. Replace or supplement `interpret_with` with an
  action-result-polymorphic handler trait.** This models the semantic
  requirement directly and is closest to a natural-transformation
  surface, but ordinary Rust closures cannot express type-generic
  methods. Users would need handler structs, helper macros, or a
  generated adapter layer.
- **C. Activate the broad B55 fallback representation for default
  `Run`.** Default `Run` internals carry ordinary Free steps or
  around-action boundary frames. `map` / `bind` compose a boundary's
  outer continuation instead of pushing it into Box-backed
  action/recovery closures, so first-order rewriting can treat the
  surrounding Catch frame as continuation context rather than as two
  independently mapped closures.
- **D. Restrict default `Run::interpret_with` / `interpose` across
  Box-backed branch-selecting scoped rows.** This is small, but it
  encodes a handler-order limitation in the API and conflicts with the
  Heftia semantic-port goal.

**Resolution: Option C.** Activate the broad default `Run`
representation fallback. This follows the project-wide API stability
stance: prefer the cleaner long-term architecture over another
compatibility-preserving local patch. Option C keeps the closure-based
`interpret_with` surface viable while moving the action/outer
continuation split into the default `Run` representation, where scoped
branch selection actually happens. Option B remains a later revisit if
user-defined scoped effects need a public action-result-polymorphic
first-order handler protocol beyond the standard default `Run`
representation.

**Implementation sequencing.** Phase 5 step 2 now continues with the
broad fallback as concrete implementation work: prove a private
default-`Run` representation that carries pure/Free-backed programs and
one BoxCatch boundary frame; migrate `Run::catch` to construct that
boundary while keeping the public return type as `Run`; wire core
operations over the new representation; reimplement
`Run::interpret_with` over the representation with State-before-Catch
coverage; re-audit `Run::interpose`; then restore the B56 Heftia
semantic-port acceptance suite.

**Plan-text amendment.** B57 moved out of Active Blockers. Phase 5
steps 2.10-2.16 now describe the broad default `Run` representation
fallback, the `interpret_with` / `interpose` rollout, the Heftia
acceptance restoration, and the later Option B revisit gate.

## Resolved (2026-05-14): B56 default `Run::interpret_with` duplicates single-shot continuations across Box-backed Catch branches

**Disposition.** B56 surfaced after Phase 5 step 2.9, when the
preserved Heftia semantic-port work was restored and run against the
default `Run` implementation. Rc-backed Choose + Catch and Pythagorean
Choose cases passed, but default-`Run` State + Catch handler ordering
and custom first-order-effect lowering into Throw/Catch failed with
`Free::to_view map called more than once`.

The failing path is ordinary default `Run::interpret_with` rewriting a
program that contains a Box-backed `Catch` scoped layer and a pending
outer continuation. The current first-order rewrite path peels the
program and uses the scoped row's ordinary `Functor` implementation.
For `BoxCatchBrand`, that maps the same single-shot continuation into
both the protected action and the recovery handler. A Catch dispatcher
can observe both branches while handling a throw, so a continuation that
must run after exactly one selected branch becomes duplicated.

**Why this blocks the Heftia semantic port.** The State + Catch case
requires composing State before Catch with `interpret_with`, and the
custom-effect case requires interpreting a custom first-order effect
into Throw before Catch. Skipping those cases would hide the exact
current-effect semantics the Phase 5 port is meant to pin down.

**Options considered:**

- **A. Weaken or skip the failing Heftia cases for default `Run`.**
  This is cheap, but it preserves the semantic hole and conflicts with
  the API stability stance: tests would stop representing the desired
  long-term architecture.
- **B. Document `Run::interpret_with` / `Run::interpose` as unsupported
  across Box-backed around-action rows and require users to avoid that
  handler order.** This is also cheap, but it turns an architectural
  limitation into API debt and makes handler ordering less composable.
- **C. Add a continuation-aware default-`Run` scoped rewrite path for
  first-order rewrites.** Extend `Run::interpret_with` first, then
  re-audit `Run::interpose`, so Box-backed around-action cells keep the
  selected action and the saved outer continuation queue in separate
  slots while first-order row narrowing is applied inside the selected
  action or recovery branch.
- **D. Activate the broad Option C fallback from B55: replace the
  default `Run` internals with a composable representation carrying
  ordinary Free steps or around-action boundary frames.** This may be
  the cleanest endpoint if targeted scoped rewrites hit another Rust
  wall, but it is wider and should not be the first move while the
  private two-slot machinery is already working for raw scoped dispatch.

**Resolution: Option C first, Option D as fallback.** The next Phase 5
implementation work adds a targeted continuation-aware default-`Run`
first-order rewrite path. The selected action and saved outer
continuation queue remain distinct while first-order row narrowing runs
inside the selected branch. This keeps public constructors returning
`Run`, preserves handler-order semantics, and builds on the B55 two-slot
substrate instead of introducing a parallel representation. Activate
Option D only if the targeted rewrite path cannot remain private,
type-directed, and maintainable.

**Implementation sequencing.** Phase 5 step 2 gains four concrete
follow-ups: implement the B56 continuation-aware `Run::interpret_with`
rewrite path for the Box-backed Catch case that exposed the bug;
re-audit `Run::interpose` and route it through the same shape if it can
duplicate continuations in the same way; restore the named
`preserve failing Heftia semantic port for B56` stash as the acceptance
suite once the rewrite path is ready; and trigger the broad B55 fallback
only after documenting a concrete Rust, macro, inference, privacy, or
maintainability wall.

**Plan-text amendment.** B56 moved out of Active Blockers. The next
greenfield step is now Phase 5 step 2.10, followed by the
`interpose` audit, the Heftia semantic-port restoration, and the
explicit fallback gate before Phase 5 step 3 continues.

## Resolved (2026-05-14): B55 default `Run` around-action composition requires two-slot scoped rows first

**Disposition.** B55 surfaced while implementing B54's standalone
default-`Run` boundary-returning constructor shape. That shape solved
the top-level continuation attachment case, but it broke the ordinary
smart-constructor composition model: `Run::span(...)` stopped being a
`Run<_, _, _>` action and therefore could not be passed to
`Run::catch(...)`, nested inside another `Run::span(...)`, or used as
input to any constructor that expects a `Run` action. The Heftia
semantic ports need nested around-action programs, so a
top-level-only boundary surface would trade the B54 semantic hole for
a public API dead end.

**Options considered:**

- **A. Keep standalone boundary-returning constructors and require raw
  construction for nested cases.** Fastest continuation of B54, but it
  makes ergonomic smart constructors non-compositional and pushes users
  toward internal substrate shapes.
- **B. Add an ad hoc `RunBoundary -> Run` lowering conversion.** This
  restores type-checking at call sites, but lowering before a scoped
  handler observes the selected action risks reintroducing the same
  premature-continuation attachment B54 was meant to remove. If the
  lowering erases `Action`, it creates another dynamic-dispatch /
  downcast escape hatch.
- **C. Add a composable internal `Run` representation that can carry
  ordinary `Free` steps or around-action boundary frames.** Public
  constructors continue returning `Run`, and `Run::bind` / `map`
  compose boundary outer continuations when the representation is a
  boundary frame. This keeps the public API coherent, but it is a broad
  refactor and must handle the existential selected-action type without
  leaking `Any`-based erasure into normal paths.
- **D. Redesign scoped rows around a two-slot around-action substrate
  (`Action`, `Final`) instead of the current one-slot `Kind::Of<T>`
  projection.** This is the most principled model for scoped effects:
  handlers see the selected action at `Action`, while mapping/binding
  changes only `Final`. It best matches the long-term architecture
  goal, but it is the widest change because it touches row macros,
  effect cell brands, dispatcher traits, and likely every wrapper's
  scoped-operation path.

**Resolution: Option D first, Option C as fallback.** The project
should prioritize the elegant long-term architecture instead of
continuing the technical-debt loop. Phase 5 step 2 therefore starts
with a two-slot scoped-row prototype for around-action operations. The
fallback is the composable internal `Run` representation only if the
two-slot row prototype hits a concrete Rust, macro, or inference wall.
Standalone boundary-returning default constructors are not adopted.

**Implementation sequencing.** Phase 5 step 2 is now concrete:
add regressions for the original B54 top-level case and the B55 nested
composition cases; prototype the two-slot scoped-row vocabulary and
row macro spelling; wire one default Box-backed operation end-to-end
through the two-slot path; migrate the remaining default Box-backed
around-action constructors (`catch`, `local`, `ref_local`, `span`,
and `bracket`) through the chosen composable architecture; re-audit
`Run::interpret_with` / `Run::interpose`; preserve raw
reboxed-result normalization where raw dispatch remains; then restore
the broad Heftia semantic-port tests. Default `Run` intentionally has
no `ref_bracket` constructor; RefBracket remains on the existing
refcounted-pointer surfaces.

## Resolved (2026-05-14): B54 default `Run` Box-backed around-action boundary architecture

**Disposition.** B54 surfaced during Phase 5 semantic-port scoping.
The preserved WIP stash `preserve B54 Heftia semantics investigation`
showed that Rc-backed Choose + Catch and Pythagorean search cases were
passing, but default-`Run` State + Catch and custom-effect ordering
cases failed when a first-order rewrite happened before or through a
Box-backed `Catch` scoped row. The immediate symptoms were a
`Type mismatch in Free::bind` before raw reboxed results were
normalized, then `Free::to_view map called more than once` once that
normalization was locally corrected.

The common cause is that default `Run::interpret_with` and
`Run::interpose` still call `peel()`, which maps the pending
single-shot erased `Free` continuation into Box-backed scoped layers.
For `BoxCatch`, the same single-use continuation can end up behind
both the protected action and the recovery handler. B30 solved the
full `Run::interpret` dispatcher path by keeping the continuation
queue outside the scoped layer until the active branch is known, but
the first-order partial-interpretation and row-preserving-rewrite
paths did not receive a corresponding architecture.

**Implementation scoping result.** The previous raw-rewrite
recommendation was not sufficient for public `Run::interpret_with` /
`Run::interpose`. Those APIs are mono-in-`A`: the handler or
replacement closure is typed for `Run<_, _, A>` because `peel()` has
already reattached every pending continuation, so each observed
first-order operation continues to the final result type `A`. A raw
scoped-layer rewrite that keeps the pending continuation queue outside
the Box-backed branch would need to recursively rewrite selected
actions at their intermediate action result type, not necessarily
`A`. That requires a rank-polymorphic handler/replacement over the
action result type, which the current closure API cannot express.
Reattaching the outer continuation first restores the mono-in-`A`
shape, but it is exactly the continuation duplication that breaks
Box-backed branching scoped effects.

**Options considered:**

- **A. Narrow the Heftia port to currently passing surfaces.** Keep
  Rc-backed Choose + Catch and Pythagorean coverage while avoiding
  default-`Run` State-before-Catch and custom-effect-before-Catch
  cases. This is fastest but hides a real semantic hole and continues
  the technical-debt loop.
- **B. Keep the current API and try to make raw first-order rewrites
  internal.** This is viable only after a dispatcher has already
  selected one raw branch. Before branch selection it either needs a
  rank-polymorphic handler/replacement over action result types or
  reattaches the single-shot continuation too early.
- **C. Promote a default `Run` indexed around-action boundary,
  mirroring the Explicit-family architecture.** Box-backed
  around-action constructors store the selected action in the
  scoped-row projection and store the outer `Action -> Final`
  continuation separately. Boundary `map` / `bind` compose only the
  outer continuation. Scoped handlers observe or transform the
  selected action at its real action result type before resuming the
  final continuation.
- **D. Add a new rank-polymorphic first-order handler/replacement
  protocol for raw rewrites.** This could represent the raw rewrite
  requirement directly, but it replaces ergonomic closure handlers
  with custom structs or an erased protocol and likely still needs
  effect-specific escape hatches.
- **E. Restrict default `Run` scoped-row-preserving first-order
  rewrites to empty scoped rows or non-branching scoped rows.** This is
  smaller but makes default `Run` semantically weaker than the
  boundary-capable wrappers and keeps manual row-shape restrictions in
  user code.
- **F. Replace Box-backed scoped closure storage with cloneable
  closures.** This avoids single-shot continuation duplication by
  changing the storage model, but regresses the single-shot `FnOnce`
  distinction between `Run` and the shared-pointer wrappers.

**Resolution: Option C.** Default `Run` should stop representing
Box-backed around-action scoped constructors as ordinary
`Run<Final>` suspensions before the scoped handler has observed their
selected action. The Explicit-family boundary work already established
the clean architecture: keep `Action` and `Final` separate, keep the
selected action in the scoped-row projection, and compose the outer
continuation on the boundary. This is API-breaking and broader than
the initial B54 repair, but it addresses the architectural cause
instead of adding another local workaround. Option B remains available
only inside dispatcher-specific code after one raw branch is selected.

**Implementation sequencing update.** The first B54 implementation
shape (standalone boundary-returning default `Run` constructors)
surfaced B55: top-level boundaries are not composable as nested
`Run` actions. B55 now resolves that follow-up by making Phase 5 step
2 prototype a composable two-slot scoped-row architecture first, with
a composable internal `Run` representation as fallback only after a
concrete wall, then migrate default Box-backed `catch`, `local`,
`ref_local`, `span`, and `bracket` through the chosen design.
`ref_bracket` remains on the existing refcounted-pointer surfaces
because default `Run` intentionally has no `ref_bracket` constructor.
Re-audit ordinary `interpret_with` / `interpose`, keep the raw
reboxed-result normalization fix where raw dispatch still applies,
and only then restore the broad Heftia semantic-port tests.

## Resolved (2026-05-14): B53 Explicit interpreter facade avoids exposing private H2 carrier traits

**Disposition.** B53 surfaced before Phase 4 step 7.4.4c.4. The step
originally said to route `RunExplicit`, `RcRunExplicit`, and
`ArcRunExplicit` scoped interpreter paths through
`DispatchScopedCarrierHandlers` when an around-action carrier is
required. That private trait is intentionally crate-internal. B47
already concluded public `interpret` / `run` methods cannot name
`DispatchScopedCarrierHandlers` in public bounds without leaking the
private H2 protocol or triggering private-bound warnings. The
7.4.4c.1c through 7.4.4c.3c work exposed effect-specific boundary
dispatcher methods, but not a generic interpreter-like surface for
indexed boundaries.

**Options considered:**

- **A. Promote `DispatchScopedCarrierHandler` /
  `DispatchScopedCarrierHandlers` to public or doc-hidden public API.**
  This is the shortest wiring path, but it exposes still-fluid H2
  internals and contradicts B47's privacy conclusion.
- **B. Treat effect-specific boundary dispatcher methods as the public
  Explicit-family surface.** This preserves privacy and is already
  verified for the standard scoped effects, but leaves users with
  dispatcher-specific calls and risks cementing technical debt.
- **C. Add a small public H3-style facade over the private H2 carrier
  protocol, then wire indexed Explicit boundaries through that facade.**
  The public facade names stable concepts while keeping concrete H2
  carrier structs and handler-list walking private; indexed boundaries
  can gain `interpret` / `run`-style methods without exposing
  `DispatchScopedCarrierHandlers`.
- **D. Replace indexed-boundary API with a true private delayed frame
  inside ordinary Explicit programs.** This restores the familiar
  `program.interpret(...)` shape, but is the widest rewrite and reopens
  hidden-intermediate-type constraints.

**Resolution: Option C.** Add a small public boundary-handler facade
over the private H2 machinery. Around-action constructors continue
returning typed boundaries because they are not ordinary programs until
a scoped handler consumes their selected action. The facade should
provide a uniform interpreter-like surface without exposing
`DispatchScopedCarrierHandlers` or cementing effect-specific dispatcher
calls as the only public route. Option D remains a fallback only if the
public facade cannot stay small and type-directed.

**Implementation sequencing.** Plan step 7.4.4c.4 now splits into four
concrete steps: design the public boundary-handler facade; implement it
for `RunExplicitBoundary`; extend it to `RcRunExplicitBoundary` and
`ArcRunExplicitBoundary`; and recheck that ordinary Explicit
interpreters still use `DispatchScopedHandlers` for non-boundary scoped
layers.

## Resolved (2026-05-13): B52 production indexed boundary needs action-supplied resume vocabulary

**Disposition.** B52 surfaced at the start of Phase 4 step 7.4.4c.1c,
after the indexed `RunExplicit` Span boundary proof established the
correct production direction: the selected action lives in the scoped row
as

```rust,ignore
SBrand::Of<'a, ActionProgram>
```

while the wrapper-owned continuation stores only the typed outer
continuation from `Action` to `Final`.

The existing single-shot Explicit resume trait did not match that shape.
`ExplicitScopedResume` assumes the carrier owns the selected action:

```rust,ignore
RunExplicitScopedContinuation {
    action: RunExplicit<'a, R, S, Action>,
    outer: Rc<dyn Fn(Action) -> RunExplicit<'a, R, S, Final>>,
}
```

A production indexed boundary must instead pass the action program from
the scoped layer into an outer-only continuation. The existing
`ExplicitLifecycleScopedResume` path already has that method shape, but
its name and documentation are Bracket-specific, so using it for Span,
Local, RefLocal, and Catch would make lifecycle terminology the generic
around-action abstraction.

**Options considered:**

- **A. Duplicate the selected action in the carrier and keep
  `ExplicitScopedResume` unchanged.** This was the smallest code change
  because the focused carrier-dispatch proofs already used that shape.
  It was rejected because the selected action would live both in the
  scoped row and in the carrier, recreating the duplicate-carrier-row
  debt B49/B51 were meant to avoid.
- **B. Reuse `ExplicitLifecycleScopedResume` for all action-supplied
  boundaries.** This avoided a new trait and reused a proven method
  shape. It was rejected because Span, Local, RefLocal, and Catch would
  call lifecycle APIs even though their selected actions come directly
  from scoped rows rather than resource lifecycles.
- **C. Generalise the lifecycle path into an action-supplied scoped
  resume contract.** Rename or replace the lifecycle-specific trait and
  continuation carrier with an outer-only, action-supplied contract used
  by both direct indexed boundaries and Bracket / RefBracket lifecycle
  dispatch.
- **D. Add per-dispatcher action-supplied resume helpers without a
  shared trait.** This reduced the first patch size but duplicated the
  same method shape across every around-action dispatcher and Explicit
  wrapper family.

**Resolution: Option C.** The correct abstraction is action-supplied
resume: the scoped layer supplies the selected action program, and the
wrapper-owned continuation supplies the typed outer continuation.
Bracket and RefBracket are one producer of such an action; Span and the
other around-action effects are others. Generalising this vocabulary
before production migration keeps the indexed boundary honest, avoids
duplicate selected-action storage, and gives `RunExplicit`,
`RcRunExplicit`, and `ArcRunExplicit` one shared target.

**Implementation sequencing.** [plan.md step 7.4.4c.1c](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now starts with 7.4.4c.1c.0, which generalises the
`ExplicitLifecycleScopedResume` / lifecycle-continuation vocabulary into
an action-supplied / outer-only scoped-continuation contract across the
Explicit wrapper family and the Bracket / RefBracket dispatcher paths.
After that, 7.4.4c.1c proceeds to production `RunExplicitBoundary`,
Span migration, and ordinary-operation regression checks.

## Resolved (2026-05-13): B51 `RunExplicit<Final>` cannot hide the selected action type under the current `FreeExplicit` representation

**Disposition.** B51 surfaced after the 7.4.4c.1a private two-slot
Explicit boundary proof succeeded. The proof was useful but incomplete:
the selected `Action` type stayed visible in the local proof type. The
production target was harder because `RunExplicit::span(tag,
action).bind(f)` has public type `RunExplicit<'a, R, S, Final>`, while
an around-action interpreter must still be able to handle the selected
action row projection:

```rust,ignore
SBrand::Of<'a, RunExplicit<'a, R, S, Action>>
```

alongside a wrapper-owned continuation from `Action` to `Final`.

The current `FreeExplicit` representation is mono in the final result
slot:

```rust,ignore
FreeExplicitView::Wrap(F::Of<'a, Box<FreeExplicit<'a, F, A>>>)
```

That forces `FreeExplicit::bind` to map every suspended layer from `A`
to `B`. `RunExplicit::peel` mirrors the same one-slot shape by
returning `NodeBrand<R, S>::Of<'a, RunExplicit<'a, R, S, Final>>`.
Neither type has a place to expose `SBrand::Of<'a, ActionProgram>` for
an existential selected `Action`. Rust enum variants cannot introduce a
hidden type parameter, and an object-safe visitor cannot have the
handler-list-generic methods needed to resume the private H2 protocol.

- **Resolution: Option A, introduce an indexed around-action boundary
  as a first-class architecture.** Replace the narrow
  `RunExplicit<Final>`-only production step with a design/prototype step
  for an indexed boundary such as
  `RunExplicitBoundary<'a, R, S, Action, Final>` or an equivalent
  redesigned scoped-program representation. The selected action remains
  typed as `SBrand::Of<'a, ActionProgram>`, and the outer continuation
  remains a wrapper-owned typed boundary. This may be API-breaking; the
  project now explicitly prefers the clean long-term architecture over
  another private carrier-row workaround.
- **Fallback kept on file: Option B, B49 Option C private carrier-row
  shapes.** Use private standard-effect carrier rows only if the indexed
  boundary prototype hits a concrete compiler, safety, privacy, or
  HKT/class-composition wall. This fallback is smaller but continues the
  duplicate-row technical-debt pattern.
- **Rejected as mainline: Option C, erased private trait objects or
  `Any`.** This would keep the public type shape stable, but it gives up
  the Explicit family's typed, borrow-friendly purpose and risks the
  same object-safety wall.
- **Rejected as mainline: Option D, defer Explicit around-action
  support.** Deferral avoids churn but leaves a central Phase 4 feature
  unfinished.

**Implementation sequencing.** [plan.md step 7.4.4c.1b-alt](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now owns the concrete Option A prototype. It starts by choosing the
indexed boundary API, proves borrowed Span end-to-end on `RunExplicit`,
records the user-facing ergonomics and compatibility helpers, and only
then resumes the broader Explicit-family migration steps.

## Resolved (2026-05-13): B50 B49 two-slot boundary should use a private indexed protocol, not the unary class contract

**Disposition.** B50 surfaced during Phase 4 step 7.4.4c.1a.1, after
the Span-only `TypedBorrowedSpanBoundary` proof showed that the
Explicit boundary needs both a selected action program/value slot and a
final program/value slot. The HKT/class compatibility audit found that
this shape is not a drop-in implementation of
`RunExplicitBrand<R, S>::Of<'a, A>`.

The current ordinary class surface is unary:

```rust,ignore
type Of<'a, A: 'a>: 'a = RunExplicit<'a, R, S, A>;
```

`Functor`, `Semimonad`, `RefFunctor`, and `RefSemimonad` all transform
that one value slot. A selected-action boundary needs a second slot. If
the selected action stays hidden inside `RunExplicit<Final>`, the B48
existential problem returns: the representation needs a hidden
`Action`, but Rust enum variants cannot introduce their own hidden type
parameter and a trait-object frame would need dyn-incompatible
handler-list-generic methods. If `Action` moves into the brand so
`Final` remains the one class value slot, the brand has to define
`Of<'a, Final>` for arbitrary application lifetimes, which effectively
requires the action value to outlive every application lifetime and
loses the non-`'static` payloads the Explicit family exists to
preserve.

`cargo-expand` confirmed that the existing two-slot kind declaration in
`kinds.rs` already expands to the needed low-level shape:

```rust,ignore
pub trait Kind_266801a817966495 {
    type Of<'a, A: 'a, B: 'a>: 'a;
}
```

That shape can model a private boundary application carrying
`ActionProgram` and `FinalProgram` at the same application lifetime. It
does not make the existing unary `RunExplicitBrand<R, S>` class impls
fit, and the existing `Bifunctor` hierarchy is not the right semantic
target: `Bifunctor` maps two ordinary value slots, while the scoped
boundary runs a selected action and resumes a typed outer
continuation.

- **Resolution: Option B, keep ordinary `RunExplicit` unary and add a
  private indexed boundary protocol.** Use
  `Kind!(type Of<'a, A: 'a, B: 'a>: 'a;)` as private
  interpreter-only plumbing for the Explicit around-action boundary.
  The protocol must name `ActionProgram`, `ActionValue`, and
  `FinalProgram` explicitly, keep the selected action lifetime in the
  boundary application rather than in a lifetime-independent brand
  parameter, and remain private to the interpreter path.
- **Revisit kept on file: Option A, a public indexed HKT/class layer.**
  This is the most general architecture if custom user-defined
  around-action handlers eventually need a public two-slot protocol, but
  it is larger than the standard scoped-effect rollout needs now.
- **Why-not Option C, make the public Explicit wrapper/class shape
  indexed.** It is honest about the two-slot representation but would
  push an interpreter-internal boundary into every `RunExplicit` user
  and break the ordinary unary class expectation.
- **Fallback kept on file: Option D, private standard-effect carrier
  row-shapes.** Use this only after the private indexed protocol records
  a concrete compiler, privacy, or safety wall. It is stable and
  focused, but less elegant and repeats effect-specific machinery.

**Implementation sequencing.** [plan.md step 7.4.4c.1a.2](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now defines the private two-slot Explicit boundary protocol. Step
7.4.4c.1a.3 adds `RcRunExplicit` and `ArcRunExplicit` obligations, and
step 7.4.4c.1a.4 records whether the protocol is sufficient for the
production migration.

## Resolved (2026-05-13): B49 B48 Option A needs an HKT-compatible hidden-action representation

**Disposition.** B49 surfaced while turning B48 Option A into the first
7.4.4c.1a implementation patch. B48 picked the right long-term target:
the Explicit family should expose a real selected-action / outer-
continuation split instead of accumulating more local carrier-row
workarounds. The missing detail was how to express that split in a Rust
representation that still fits the library's HKT and brand APIs.

The current production shape is too narrow to directly store a hidden
selected action inside `RunExplicit<Final>`:

- `FreeExplicitView<'a, F, A>` has only `Pure(A)` and
  `Wrap(F::Of<'a, Box<FreeExplicit<'a, F, A>>>)`.
- `RunExplicit<'a, R, S, A>` is a tuple wrapper around
  `FreeExplicit<'a, NodeBrand<R, S>, A>`.
- `RunExplicitBrand<R, S>::Of<'a, A>` exposes only the final result
  type `A`.

A delayed frame also needs a scoped row at
`SBrand::Of<'a, RunExplicit<'a, R, S, Action>>` plus a typed
`Action -> RunExplicit<'a, R, S, Final>` continuation for a hidden
`Action`. Rust enum variants cannot introduce their own hidden type
parameter, and a trait-object frame would have to call into the private
H2 handler-list protocol through generic methods, which are not
dyn-compatible. `Any` or unsafe erasure would weaken the
non-`'static`, typed-payload reason the Explicit family exists.

- **Resolution: Option B first, prototype a broader HKT-compatible
  Explicit substrate.** The next implementation step is a focused
  Span-only proof that makes `ActionProgram` and `FinalProgram`
  separately representable without `Any`, unsafe erasure, dyn-generic
  handler methods, or public H2 bounds. The proof must record whether
  the current `RunExplicitBrand<R, S>::Of<'a, A>` contract can survive,
  or whether the clean architecture requires an intentional API break.
- **Fallback kept on file: Option C, private standard-effect carrier
  row-shapes for the Explicit family.** If the Option B proof hits a
  concrete Rust or HKT/brand-contract wall, adopt the carrier-row
  fallback openly as the best type-sound stable-Rust architecture for
  the standard effects. It should not enter through another implicit
  local workaround.
- **Why-not Option A, direct private existential frame inside the
  current wrapper shape.** This keeps the smallest surface if it worked,
  but the object-safety problem is load-bearing: the hidden frame needs
  access to handler-list-generic H2 dispatch.
- **Why-not Option D, drop Explicit-family around-action parity.** This
  would unblock default wrappers, but would leave the non-`'static`
  Explicit family with weaker semantics and break the six-wrapper
  parity goal.

**Trade-off.** Option B has the largest potential blast radius because
it may change the Explicit wrapper shape, the brand/type-class
contract, and then the Rc/Arc Explicit siblings. It is still the right
first step under the project's architecture-priority rule: establish
whether the elegant H2 boundary is actually representable before
settling for a narrower standard-effect carrier-row architecture.

**Implementation sequencing.** [plan.md step 7.4.4c.1](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now starts with concrete B49 Option B prototype steps: isolate a
Span-only substrate proof, check current HKT/class compatibility,
include shared Explicit wrapper obligations, and gate the Option C
fallback on a recorded compiler or brand-contract wall.

## Resolved (2026-05-13): B48 delayed typed frame proof is not reachable from `RunExplicit<Final>`

**Disposition.** B48 surfaced immediately after the 7.4.4c.0 delayed
typed frame proof compiled. That proof kept a scoped
`RunExplicit<'a, R, S, Action>` source and a typed
`Action -> RunExplicit<'a, R, S, Final>` outer continuation separate,
then peeled the selected scoped row as `SBrand::Of<'a, ActionProgram>`.
It proved the right H2 shape for Span, but only while `Action` remained
a named type parameter in a test-only frame.

The production `RunExplicit<'a, R, S, Final>` substrate cannot yet
store that value after `bind` hides the selected action type.
`RunExplicit` is currently a tuple wrapper around
`FreeExplicit<'a, NodeBrand<R, S>, Final>`, and the `FreeExplicit`
view only has a pure value or a wrapped node at the final result slot.
A production delayed frame needs to keep a scoped row at
`ActionProgram` plus a wrapper-owned outer continuation for some hidden
`Action`. That hidden typed boundary has to remain private, avoid
unsafe erasure, and preserve non-`'static` Explicit-family payloads.

- **Resolution: Option A, add a true private delayed-frame
  representation to the Explicit substrate.** Phase 4 step 7.4.4c now
  starts with representation design and a focused prototype that makes
  the delayed frame reachable from an ordinary
  `RunExplicit<'a, R, S, Final>` after `bind`. The target architecture
  is the clean H2 split: scoped rows own the selected action program,
  wrapper carriers own the outer continuation, and the
  carrier-handler protocol remains private.
- **Fallback kept on file: Option B, private carrier row-shape
  alternatives for the standard effects.** The carrier-row fallback is
  available only if the Option A prototype records a concrete Rust
  compiler or safety wall that cannot be worked around without unsafe
  erasure or losing the Explicit family's non-`'static` safety
  premise.
- **Why-not Option C, keep the test-only frame plus a narrow helper.**
  A helper would preserve the 7.4.4c.0 proof shape but would not solve
  the production problem: user programs reach interpreters as
  `RunExplicit<Final>`, where the selected action type is already
  hidden.
- **Why-not Option D, skip Explicit-family around-action carrier
  parity.** This would unblock default wrappers, but it would break the
  six-wrapper parity goal and leave nested around-action semantics
  weaker for the wrapper family that exists to support non-`'static`
  payloads.

**Trade-off.** Option A has the highest implementation blast radius:
it may change the private `RunExplicit` / `FreeExplicit`
representation, bind and peel invariants, and the shared Explicit
wrappers. It is still the right long-term direction. Continuing to add
local carrier-cell and carrier-row shapes would entrench the
debt-accruing loop that produced B42 through B48. The project now
prefers broad, API-breaking internal architecture changes when they
produce a cleaner and more durable substrate.

**Implementation sequencing.** [plan.md step 7.4.4c](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now starts with 7.4.4c.1a, a representation design/prototype step for
the true Explicit delayed frame. If it succeeds, 7.4.4c.1b makes Span
reachable from ordinary `RunExplicit<Final>`, 7.4.4c.1c migrates core
Explicit operations over the new representation, 7.4.4c.1d proves the
other standard around-action effects still fit, and 7.4.4c.3 extends
the boundary to `RcRunExplicit` and `ArcRunExplicit`. The carrier
row-shape fallback remains a gated later step, not the default
production architecture.

## Resolved (2026-05-13): B47 carrier-cell proofs have no interpreter row home

**Disposition.** B47 surfaced before Phase 4 step 7.4.4c. Step
7.4.4b proved private carrier-cell layer shapes and focused dispatcher
methods for Span, Local / RefLocal, Catch, Bracket, and RefBracket
across the Explicit wrapper family. Those proofs work as isolated
effect-specific cells, but they are not values of the scoped row
projection consumed by wrapper interpreters. The ordinary interpreter
loops see `SBrand::Of<'a, NextProgram>` through
`DispatchScopedHandlers`; the H2 carrier-aware path needs
`SBrand::Of<'a, ActionProgram>` plus a wrapper-owned
`ScopedContinuation<Carrier>`. The carrier-cell fallback stores a
concrete wrapper-family carrier cell directly, and that carrier type
contains the selected action, final result, and outer continuation
closure type. That concrete type has no home in the static scoped row
brand or the final `NextProgram` slot.

There was also an API boundary problem:
`DispatchScopedCarrierHandler` and `DispatchScopedCarrierHandlers` are
intentionally private. Public `interpret` / `run` methods cannot name
those traits as visible bounds without leaking the private H2 protocol
or triggering private-bound warnings under the repository's
warning-deny policy.

- **Resolution: Option C, reopen the Explicit substrate
  continuation-frame path and recover the action/outer split at the
  substrate boundary.** Start with a narrow `FreeExplicit` proof that
  delayed typed continuation frames can expose one scoped raw step as
  `SBrand::Of<'a, ActionProgram>` plus a typed outer continuation
  carrier while preserving borrowed payloads and hidden intermediate
  types without unsafe erasure. If that proof compiles, adopt the
  private substrate shape, then extend it to `RcFreeExplicit` and
  `ArcFreeExplicit` before wiring wrapper interpreters.
- **Fallback kept on file: Option B, add carrier row-shape
  alternatives scoped to the standard effects first.** If the
  `FreeExplicit` proof hits the hidden-intermediate-type wall, or if
  the later shared-wrapper boundary leaks private bounds or cannot
  preserve `Rc` / `Arc` obligations, stop and record the concrete
  compiler wall before evaluating private carrier row shapes.
- **Why-not Option A, promote the carrier-handler protocol to a public
  or doc-hidden public API.** This would let public interpreters name
  `DispatchScopedCarrierHandlers`, but it would expose a still-fluid H2
  implementation detail before the Span migration proves the protocol
  ergonomics. It also would not by itself give the existing
  Explicit-family carrier cells a row projection home.
- **Why-not Option D, ship a bespoke Span-only interpreter path.** This
  is the smallest route to nested Span ordering, but it breaks
  six-wrapper parity and leaves the same row-home issue waiting for
  Local / RefLocal, Catch, Bracket, RefBracket, and custom
  around-action handlers.

**Trade-off.** Option C is more substrate work than simply preserving
the latest carrier-cell fallback, but it attacks the structural
mismatch directly. The target shape is the original H2 model:
scoped-effect rows own the selected action program, wrapper-owned
carriers own the outer continuation, and the carrier-handler protocol
stays private. That is a better long-term boundary than multiplying
wrapper/effect-specific carrier row brands or making the private H2
dispatch protocol public before it has settled.

**Implementation sequencing.** [plan.md step 7.4.4c](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now starts with 7.4.4c.0, a focused `FreeExplicit` delayed typed
continuation-frame proof. If it compiles, 7.4.4c.1 adopts the private
`FreeExplicit` / `RunExplicit` extraction boundary, 7.4.4c.2 migrates
the Explicit-family production plan away from carrier-cell row shapes,
7.4.4c.3 extends the boundary to `RcFreeExplicit` and
`ArcFreeExplicit`, and 7.4.4c.4 through 7.4.4c.5 wire the Explicit and
default erased wrapper interpreters. Step 7.4.4c.6 keeps the carrier
row-shape fallback available if the proof path fails later.

**Implementation outcome.** The 7.4.4c.0 proof compiled. A focused
`run_explicit.rs` test keeps a scoped `RunExplicit` source and its
typed outer continuation in a delayed frame instead of distributing the
continuation through `RunExplicit::bind`. Peeling that frame exposes a
Span scoped row at `SBrand::Of<'a, ActionProgram>`, preserves a
borrowed selected action value, and verifies result-preserving
post-action work runs before the typed outer continuation. The next
step is to adopt that proof as the private `FreeExplicit` /
`RunExplicit` raw carrier extraction boundary.

## Resolved (2026-05-12): B46 Bracket / RefBracket selected action is lifecycle-generated

**Disposition.** B46 surfaced while preparing Phase 4 step 7.4.4b.3c.
The H2 carrier path used by Span, Local / RefLocal, and Catch assumes
the selected action program already exists when the carrier layer is
constructed. Bracket and RefBracket do not have that shape: the body
action can only be constructed after `acquire` runs and produces the
resource, and `release` must run after the body action but before the
outer continuation resumes. Reusing `RunExplicitScopedContinuation`
with a placeholder selected action would weaken the invariant that a
carrier owns the real selected action.

- **Resolution: Option C first, extend the private carrier protocol
  with a lifecycle-generated-action hook.** Add private
  family-specific resume operations where Bracket-family dispatchers
  can construct the selected body action after acquire, run effectful
  release after the body action, and resume the outer continuation only
  after release completes. Preserve the existing family split:
  single-shot Explicit, Rc-shared, and Arc-shared bounds stay on their
  private traits.
- **Fallback: Option B, add Bracket-specific lifecycle carrier layers
  and dispatcher methods.** If the focused proof shows the generalized
  lifecycle hook requires public API churn, unsafe erasure, or
  unbounded generic callbacks, fall back to private Bracket /
  RefBracket lifecycle carrier layers that store acquire, body,
  release, and the outer continuation directly.
- **Why-not Option A, force Bracket into the existing selected-action
  carrier.** This would be a smaller patch, but it would make the
  carrier field misleading and hide lifecycle ordering behind a dummy
  action. Future wrapper interpreter wiring would not be able to trust
  the selected-action invariant.
- **Why-not Option D, defer the carrier-backed Bracket / RefBracket
  path.** The ordinary dispatchers already preserve lifecycle ordering,
  but deferring this path leaves 7.4.4b.3c and the later wrapper
  interpreter route incomplete.

**Trade-off.** Option C expands the private carrier trait surface, but
it names the real capability: some around-action effects do not have a
selected action until an earlier lifecycle phase has run. Keeping this
private and family-specific is preferable to either a misleading dummy
action or an early public protocol-family split. The fallback is
deliberately effect-specific so failure of the generalized hook does
not contaminate the existing Span, Local / RefLocal, and Catch carrier
paths.

**Implementation sequencing.** [plan.md step 7.4.4b.3c](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now starts with 7.4.4b.3c.0, adding the lifecycle-generated-action
carrier hook and proving it on `RunExplicit`. Steps 7.4.4b.3c.1
through 7.4.4b.3c.3 add Bracket / RefBracket carrier metadata layers,
focused dispatcher paths for `RunExplicit`, `RcRunExplicit`, and
`ArcRunExplicit`, and lifecycle-ordering coverage.

## Resolved (2026-05-12): B45 Local / RefLocal need pre-action carrier transformation

**Disposition.** B45 surfaced while preparing the B44 Local /
RefLocal substep. The shipped carrier resume protocol was shaped
around Span-like post-action insertion: `resume_*_with_post_action`
runs the selected action first, then inserts a result-preserving
action program before the outer continuation. That is correct for
Span, where the handler observes the action result before outer
resume. It is not correct for Local / RefLocal. Local semantics require
the dispatcher to read the inherited Reader environment, compute the
modified environment, and run the selected action under Reader
interposition so asks inside the action see the modified environment.
Once post-action insertion receives an action value, the selected
action has already run and it is too late to affect its Reader asks.

- **Resolution: Option A, add a private selected-action transform hook
  to the family-specific carrier traits.** Extend the private resume
  contracts with methods such as `resume_*_with_action_transform`,
  where the transform receives the carrier's `ActionProgram` and
  returns the transformed `ActionProgram` before the outer continuation
  is reattached. Local / RefLocal can ask the inherited environment,
  compute the local environment, transform the selected action with
  Reader interposition, and then resume the outer continuation.
- **Why-not Option B, destructure concrete carrier cells in
  dispatchers.** The current concrete carrier fields are `pub(crate)`,
  so this would be short. It would also couple every effect dispatcher
  to carrier field layout and bypass the private family-specific
  resume contracts that were added to prevent exactly that kind of
  cross-wrapper leakage.
- **Why-not Option C, keep using post-action insertion.** This gives
  incorrect Local / RefLocal semantics because Reader asks inside the
  selected action would observe the inherited environment rather than
  the modified one.
- **Why-not Option D, add Local-specific deferred rewrite closures to
  carrier layers.** This preserves the current trait signatures but
  recreates the same action-transform capability as effect-specific
  layer machinery, making Catch and future around-action effects likely
  to add parallel bespoke hooks.

**Trade-off.** Option A expands the private carrier trait surface and
requires focused tests across the family-specific forwarding methods.
The upside is that the new capability matches the actual semantic
boundary: some around-action handlers need to transform the selected
action program before it runs, while Span needs to insert work after
the selected action value is produced and before outer resume. Keeping
both operations private and family-specific avoids unsafe erasure,
dyn-generic callbacks, and dispatcher-side carrier-field coupling.

**Implementation sequencing.** [plan.md step 7.4.4b.3a](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now starts with step 7.4.4b.3a.0, adding the selected-action transform
hook and forwarding methods on `ScopedContinuation`. Steps 7.4.4b.3a.1
through 7.4.4b.3a.3 then add the Local / RefLocal carrier metadata
layers, dispatcher paths, and focused coverage.

## Resolved (2026-05-12): B44 remaining around-action carrier retrofit splits by semantic class

**Disposition.** B44 surfaced while preparing Phase 4 step 7.4.4b.3.
The shipped Span carrier-cell proof is load-bearing, but it is not a
mechanical template for every remaining around-action effect. Span is a
metadata-plus-carrier operation: the handler observes a tag around the
selected action and does not itself have to run first-order recovery,
environment substitution, or resource finalization. `Catch`, `Local`,
`RefLocal`, `Bracket`, and `RefBracket` all need the carrier-cell path,
but they have different operational obligations.

- **Resolution: Option B, split the retrofit by semantic class.** Step
  7.4.4b.3 becomes a set of concrete substeps:
  `Local` / `RefLocal` first, then `Catch`, then `Bracket` /
  `RefBracket`, followed by optional private-helper consolidation only
  if the implementations reveal meaningful duplication.
- **Why-not Option A, retrofit all remaining effects in one broad
  pass.** This would minimize planning churn, but it would mix Reader
  environment substitution, Except recovery, and Bracket resource
  finalization in one diff. A failure in the most complex path would
  obscure whether the simpler carrier-cell protocol was working.
- **Why-not Option C, generalize a carrier metadata abstraction first.**
  A shared abstraction may become useful, but the remaining effects do
  not share the same metadata arity or control-flow obligations. Adding
  it before the effect-specific proofs risks another premature protocol
  shape.
- **Why-not Option D, implement only `Local` / `RefLocal` and defer the
  rest without plan detail.** This would be the smallest immediate
  code step, but it would leave the next implementer to rediscover that
  `Catch` and Bracket-family effects are materially different from
  Span.

**Trade-off.** Option B creates more commits and may temporarily leave
small private helper duplication in place. In exchange, each commit has
a clear semantic claim: Reader environment modification with a
carrier-cell action, Except recovery with a carrier-cell action, and
Bracket resource lifecycle ordering with a carrier-cell action. That
keeps the implementation reviewable and avoids turning the private
carrier protocol into a premature all-effects framework before the
hardest cases have been proven.

**Implementation sequencing.** [plan.md step 7.4.4b](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now resumes at step 7.4.4b.3a with the `Local` / `RefLocal`
carrier-cell retrofit. Step 7.4.4b.3b handles `Catch` recovery
ordering. Step 7.4.4b.3c handles `Bracket` / `RefBracket` acquire,
body, release, and result-return ordering. Step 7.4.4b.3d consolidates
private carrier-layer helpers only if the first three substeps show
real shared structure.

## Resolved (2026-05-12): B43 RunExplicit Span proof selects carrier-cell fallback

**Disposition.** B43 is the result of executing the B42 proof gate. A
focused `RunExplicit` Span test showed that the current
`RunExplicit::span(...).bind(...)` path peels to a `BoxSpan` whose
action slot already returns `RunExplicit<..., Final>` after
`FreeExplicit::bind` maps the scoped layer. The selected action value is
therefore no longer available as a separate action-owned slot at
handler dispatch time. An outer-only continuation carrier cannot insert
post-action work before the outer continuation using the existing
constructor/mapping shape.

- **Resolution: activate B42 Option C.** Carrier-backed scoped layers
  store the wrapper-family carrier cell directly instead of storing a
  raw selected action program and trying to recover the outer
  continuation later. For the first proof, Span's carrier-backed
  `RunExplicit` shape carries the tag plus a private carrier cell that
  already contains the selected action and typed outer continuation.
- **Why-not continue Option B.** The clean operation-owns-action model
  remains desirable in the abstract, but the shipped Explicit substrate
  maps scoped action slots to the final program before interpretation.
  Continuing Option B would require rewriting `FreeExplicit::bind` or
  adding another hidden continuation boundary before the mapped layer,
  which is larger than the documented fallback.
- **Why-not Option A metadata plus action-owning carrier.** This keeps
  the existing action-owning carrier but still requires an
  effect-specific metadata view for every around-action effect. Option
  C is more direct for the Explicit substrate because the mapped scoped
  layer can simply carry the already-prepared carrier cell.
- **Why-not Option D.** Synthesizing a placeholder action or ignoring
  one action copy would not prove single-shot ownership and would fail
  again when other around-action effects are moved to the same path.

**Trade-off.** Option C matches the substrate that exists today:
`FreeExplicit::bind` can map the scoped layer while preserving a
carrier cell that already owns the selected action and outer
continuation. The cost is a clearer split between ordinary scoped
layers and carrier-backed scoped layers. This may require private
carrier-cell layer types, additional dispatcher impls, and carefully
scoped smart-constructor changes so public APIs do not expose
wrapper-family internals unnecessarily.

**Implementation sequencing.** [plan.md step 7.4.4b](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now treats step 7.4.4b.2a as shipped proof-gate coverage. Step
7.4.4b.2b has shipped the private `RunExplicitSpanCarrierLayer` shape
for the `RunExplicit` Span proof. Step 7.4.4b.2c has shipped the
focused private `SpanDispatcher` path that consumes that carrier cell
and proves borrowed selected payload support plus
post-action-before-outer ordering. Step 7.4.4b.2d has shipped the same
carrier-cell proof for `RcRunExplicit` and `ArcRunExplicit`, covering
repeated shared resume, Arc `Send + Sync` obligations, and borrowed
selected action payloads.

## Resolved (2026-05-12): B42 carrier-aware handler protocol duplicates selected actions

**Disposition.** B42 surfaced after the B41 two-slot protocol and
macro-spelling proofs. Those proofs showed that a static Span row brand
can project a borrowed selected-action slot while `NextProgram` remains
the final mapped result. The production handler shape had a separate
ownership problem: `DispatchScopedCarrierHandler` received both the
full scoped layer, `SBrand::Of<'a, Carrier::ActionProgram>`, and a
`ScopedContinuation<Carrier>`, while the shipped continuation carriers
also owned the selected action program. For single-shot `BoxSpan`, the
same action thunk cannot live in both places without moving it twice,
requiring a clone that does not exist, or fabricating a placeholder
action.

- **Resolution: Option B first, with Option C as the fallback if the
  proof fails.** The preferred model is that the scoped operation owns
  the selected action, and the continuation carrier owns only the outer
  resume boundary. Handlers pass the selected `ActionProgram` from the
  scoped layer into `resume` / `resume_with_post_action`, avoiding
  duplicate action ownership. Before rewriting every carrier, run a
  narrow `RunExplicit` Span proof. If `FreeExplicit::bind` still forces
  the selected action and outer continuation to be captured together
  before the scoped layer is mapped, switch to Option C and make
  carrier-backed scoped layers store the wrapper-family carrier cell
  directly.
- **Fallback kept on file: Option C.** Carrier-backed scoped layers can
  store a carrier cell directly instead of a raw action program. This
  may be necessary if the Explicit substrate cannot expose an
  outer-only continuation boundary after mapping, but it risks more row
  and constructor churn.
- **Why-not Option A.** Splitting layers into effect metadata plus an
  action-owning carrier preserves the existing resume traits, but it
  makes every around-action effect introduce a private metadata view and
  keeps the action hidden in the carrier rather than in the operation
  that semantically owns it.
- **Why-not Option D.** Ignoring one copy of the action or synthesizing
  a placeholder layer action would make a narrow prototype compile
  without proving single-shot ownership. It would almost certainly
  resurface when Local, Bracket, or RefBracket are moved to the same
  carrier path.

**Trade-off.** Option B is the clearest long-term protocol: operation
plus outer continuation mirrors the usual handler model, makes
single-shot ownership explicit, and should be easiest to explain if the
private protocol later gains a public facade. The cost is that the
private carrier traits and their focused tests must be revised so the
selected action program is a resume-method input rather than a carrier
field. The proof gate is necessary because the Explicit substrate has
already shown that `bind` can push outer continuations into scoped
layers before interpretation sees them.

**Implementation sequencing.** [plan.md step 7.4.4b](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now resumes at step 7.4.4b.2a with a narrow `RunExplicit` Span proof of
the outer-only continuation carrier. If the proof succeeds, step
7.4.4b.2b migrates the private carrier traits and list-level dispatch
to the outer-only action-ownership model, and step 7.4.4b.2c extends
the proof to `RcRunExplicit` and `ArcRunExplicit`. If the proof fails
because Explicit mapping requires action-plus-outer storage before the
layer is mapped, the implementation switches to the Option C
carrier-cell layer shape before continuing.

## Resolved (2026-05-12): B41 action-indexed carrier brands conflict with static row-brand bounds

**Disposition.** B41 surfaced immediately after B40 selected an
action-result-indexed Span prototype for constructor-stored carriers.
B40 correctly identified that carrier-backed scoped handlers need to
remember the selected action type while `Functor::map` changes the
final next-program type. The proposed action-indexed row-brand shape,
however, put the selected action type on the scoped-effect brand.

That conflicts with the current substrate invariant: `Node<'a, R, S,
A>` and the `Run*` wrappers require first-order and scoped row brands
to be `'static`. Today the Explicit wrappers can still carry borrowed
action values because those values live in the GAT result position
(`SBrand::Of<'a, A>`), not in the row brand. Moving an action type such
as `&'a str` or an action program containing borrowed payloads into the
row brand would either fail the existing bounds or quietly prove only a
`'static` subset of the Explicit carrier path.

- **Resolution: Option D, promote the around-action protocol-family
  fallback now.** Ordinary scoped rows remain one-slot rows handled by
  `DispatchScopedHandlers`. Carrier-backed around-action handlers get a
  separate minimal two-slot protocol: one lifetime-indexed boundary for
  the selected action program/value and one ordinary mapped boundary
  for the final next-program. The selected action type stays in a
  method/GAT position instead of being baked into a `'static` row brand.
- **Fallback kept on file: Option C.** If the protocol sketch proves
  too large, revisit a static action-family witness that keeps row
  brands `'static` while passing the concrete action through a separate
  lifetime-indexed carrier path. This fallback must still prove that it
  does not recreate the dyn-generic callback wall from B34/B35.
- **Why-not Option A.** Relaxing row-brand `'static` bounds to
  row-lifetime bounds could preserve the action-indexed brand shape in
  principle, but it is a foundational rewrite across `Node`,
  `NodeBrand`, Explicit Free wrappers, erased `Any`-based substrates,
  row macros, and Arc-family HRTB workarounds.
- **Why-not Option B.** Keeping action-indexed brands only for
  `'static` actions is short, but it drops borrowed Explicit payload
  support exactly where the Explicit carrier path is supposed to prove
  it.
- **Why-not Option E.** Reopening the Explicit substrate rewrite keeps
  scoped-effect brands simpler, but it reintroduces the hidden
  intermediate-type problem that B39 avoided and still does not clarify
  the handler protocol surface.

**Trade-off.** Option D is a larger step than B40's initial bounded
Span prototype, but it addresses the architectural pressure directly:
around-action handlers have two typed boundaries, while ordinary
scoped rows have one. Keeping those as separate private protocols avoids
repeated attempts to smuggle the action boundary through the wrong type
slot and preserves non-`'static` Explicit payload support.

**Implementation sequencing.** [plan.md step 7.4.4b](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now starts by defining the minimal two-slot around-action protocol for
Span. Step 7.4.4b.0 defines the private protocol vocabulary and its
coexistence with ordinary `DispatchScopedHandlers`. Step 7.4.4b.1
specifies the row and macro spelling. Step 7.4.4b.2 proves the
stored-carrier interpreter path on Span, starting with `RunExplicit`
and then the shared Explicit wrappers. Step 7.4.4b.3 applies the
approved protocol to the remaining standard around-action effects.
Step 7.4.4b.4 adds coverage for borrowed Explicit payloads, shared
resume, macro spelling, final-result mapping, and the ordinary
one-slot handler route.

## Resolved (2026-05-12): B40 constructor-stored carriers need an action-type home across Functor map

**Disposition.** B40 surfaced while turning B39's
constructor-stored carrier fallback into Phase 4 step 7.4.4b.0. B39
established that Explicit-family substrates cannot recover the
selected action and typed outer continuation after `bind` recursively
maps the continuation into suspended layers. B40 identified the next
type-level obstacle: the current scoped-effect row shape has only one
GAT result slot, `SBrand::Of<'a, X>`. A constructor-stored carrier must
remember the original selected action type while the row's ordinary
`Functor::map` changes `X` to the final next-program type.

For example, `RunExplicit::span` initially injects a `BoxSpan` whose
action thunk returns `Box<FreeExplicit<NodeBrand<R, ScopedRow>, A>>`.
After `FreeExplicit::bind`, `BoxSpanBrand::map` rewrites the action
thunk so the row cell returns
`Box<FreeExplicit<NodeBrand<R, ScopedRow>, B>>`. The original action
type `A` is no longer present in the scoped row cell's type, but the
carrier-aware handler path needs that type to run post-action work
before the outer continuation.

- **Resolution: Option A, with a bounded Span-first prototype.**
  Carrier-backed scoped brands become action-result-indexed: the
  selected action result/program type lives on the scoped-effect brand,
  while `Kind::Of<'a, X>` continues to track the final next-program
  type after ordinary `Functor::map`. Prove the shape on `Span` first
  because Span is the smallest around-action operation and directly
  needs post-action lifecycle ordering.
- **Fallback kept on file: Option D.** If the Span prototype shows
  that action-result-indexed rows cause unacceptable row ergonomics or
  macro churn, pause and reconsider a broader around-action protocol
  family before spreading the shape to Catch, Local, RefLocal,
  Bracket, and RefBracket.
- **Why-not Option B.** A private existential runner object preserves
  today's row-brand surface, but it must later call generic
  handler-list machinery. That likely requires dynamic dispatch,
  erased handler lists, or a generic callback method on a trait
  object, repeating the B34/B35 wall.
- **Why-not Option C.** Reopening the Explicit substrate rewrite keeps
  scoped-effect rows cleaner, but it reintroduces the hidden
  intermediate-type problem that caused B39 to activate the
  constructor-stored fallback in the first place.

**Trade-off.** Option A is the most static Rust shape: it preserves
non-`'static` Explicit payloads, avoids unsafe erasure, and makes
`Functor::map` preserve the action boundary by construction. The cost
is row-brand churn for carrier-backed operations: the action type moves
into the row brand, so the same operation may need distinct row
members when it wraps actions of different result types. This is
acceptable as a bounded prototype because Bracket already has
result-specific scoped brands, and Span can prove whether the
ergonomics are tolerable before the pattern spreads.

**Implementation sequencing.** [plan.md step 7.4.4b](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now starts with a Span-first action-result-indexed carrier prototype.
Step 7.4.4b.0 defines the private carrier-cell vocabulary and proves
that `Functor::map` preserves the action type in the brand while
changing the final-program GAT slot. Step 7.4.4b.1 either validates
the Span row ergonomics or pauses for the Option D fallback. Step
7.4.4b.2 spreads the approved shape to the remaining standard scoped
effects. Step 7.4.4b.3 proves the stored-carrier interpreter path.
Step 7.4.4b.4 adds focused coverage before the six-wrapper interpreter
wiring in step 7.4.4c.

## Resolved (2026-05-12): B39 Explicit-family extraction cannot recover action-outer split from recursively mapped binds

**Disposition.** B39 surfaced after Phase 4 step 7.4.4a shipped
private raw-step extraction for default `Run` and the erased shared
`RcRun` / `ArcRun` substrates. Step 7.4.4b needed the same kind of
boundary for `RunExplicit`, `RcRunExplicit`, and `ArcRunExplicit`: peel
a real scoped suspension and produce both the selected action program
and its typed outer continuation.

The focused H2 carrier tests prove that the carrier shape works when
it is constructed directly as `(action, outer)`. They do not prove that
the shape can be recovered from an ordinary Explicit-family program.
`FreeExplicit`, `RcFreeExplicit`, and `ArcFreeExplicit` still implement
`bind` by recursively mapping the continuation into suspended layers.
Their `to_view` comments describe that view as canonical because there
is no pending continuation queue. Therefore, after `action.bind(outer)`
suspends at a scoped layer, the peeled layer contains an action program
whose result is already the final outer result. The original action
result type and typed outer continuation have already been distributed.

- **Resolution: Option B.** Activate the B38 Option C fallback for
  scoped-effect constructors. Scoped-effect constructors now become
  responsible for preserving the runner/carrier boundary before the
  Explicit substrate distributes the outer continuation. Wrapper
  interpreters consume the stored carrier shape instead of trying to
  reconstruct it from the recursively mapped substrate.
- **Why-not Option A.** A full Explicit substrate retrofit with delayed
  typed continuation frames would be the most general substrate-level
  fix, but it reopens the non-`'static` hidden-intermediate-type
  problem already documented for `FreeExplicit`. It is too broad for
  step 7.4.4b and risks a deep rewrite across three Explicit
  substrates before the scoped-handler path is proven end to end.
- **Why-not Option C.** Keeping the carrier-aware path only for default
  and erased shared wrappers would break six-wrapper parity and leave
  Explicit wrappers with different nested around-action semantics.
- **Why-not Option D.** Promoting H3 protocol families now may still
  be useful later as a public facade for custom handler APIs, but it
  does not recover a lost action/outer split by itself. H3 would still
  need either a substrate retrofit or constructor-stored carriers
  underneath.

**Trade-off.** The adopted path pays representation churn in scoped
effect cells and may require helper or macro updates, but it captures
the typed boundary at the only point where that boundary still exists.
It avoids unsafe erasure, preserves non-`'static` Explicit payloads,
and keeps six-wrapper parity. If the constructor-stored carrier leaks
too much wrapper-specific detail into public APIs, the later cleanup
candidate is a deliberate Explicit substrate rewrite, not ad-hoc
recovery from already-distributed binds.

**Implementation sequencing.** [plan.md step 7.4.4](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now switches from B38 Option A for every wrapper to B38 Option C for
the Explicit-family path. Step 7.4.4a remains shipped for default and
erased shared raw-step extraction. Step 7.4.4b now designs and stores
the constructor-time carrier shape for scoped effects while preserving
existing public smart-constructor signatures. Step 7.4.4c wires the six
wrapper interpreters through the stored carrier path and existing
raw-step boundaries. Step 7.4.4d records the deferred substrate-cleanup
fallback if constructor-stored carriers become too invasive.

## Resolved (2026-05-12): B38 wrapper interpreters need carrier extraction boundaries

**Disposition.** B38 surfaced while starting Phase 4 step 7.4.4,
after step 7.4.3 added `DispatchScopedCarrierHandler` /
`DispatchScopedCarrierHandlers`. The carrier-aware handler list can
thread an `SBrand::Of<ActionProgram>` action plus a wrapper-owned
`ScopedContinuation` through around-action handlers, but the wrapper
interpreters still need a private way to peel one scoped layer while
keeping the selected action separate from its outer continuation.

The default `Run` substrate already has a raw-step boundary through
`Free::into_raw_step`. The Rc / Arc erased substrates internally carry
`RcCatList` / `ArcCatList` continuation queues, but their ordinary
view/resume helpers reattach those queues around the selected action.
The Explicit substrates store typed continuations whose hidden
intermediate values must remain typed and non-`'static`; extracting a
selected scoped action by erasing those values would recreate the
earlier Explicit-family walls.

- **Resolution: Option A.** Add private per-substrate carrier
  raw-step extraction before wiring the six wrapper interpreters.
  Reuse `Free::into_raw_step` for default `Run`; add Rc / Arc private
  raw-step views that expose the selected scoped action while keeping
  the shared continuation queues outside it; add an Explicit-family
  extraction boundary that preserves non-`'static` typed payloads and
  keeps the selected action separate from its typed outer
  continuation. Once those private boundaries exist, thread the
  carrier-aware scoped-handler list through all six wrappers.
- **Fallback kept on file: Option C.** If private raw-step extraction
  requires public or macro surface churn, unsafe erasure, loss of
  Explicit non-`'static` payload support, or a substrate rewrite larger
  than the H2 private-carrier path can justify, pause and switch to a
  constructor-stored runner/carrier shape for scoped effects. That
  fallback would make each scoped-effect constructor store the carrier
  shape needed by around-action dispatch directly instead of deriving
  it from the wrapper substrate during interpretation. It is not the
  first path because it spreads interpreter concerns into scoped-effect
  representation and would make ordinary scoped operations carry more
  wrapper-specific machinery.
- **Why-not Option B.** A local Span-only patch would let the current
  test slice advance, but it would leave Local / Bracket /
  RefBracket / custom around-action handlers without a shared
  continuation boundary and would almost certainly reopen the same
  extraction problem at the next handler.
- **Why-not Option D.** Jumping directly to the H3 protocol-family
  facade would broaden the public-facing protocol before proving the
  private H2 carrier extraction can actually fail. H3 remains useful
  later if custom handler APIs need named protocol families, but it is
  larger than the current private wiring problem.

**Trade-off.** Option A pays for private substrate symmetry now. That
is the right long-term cost because the wrapper interpreters already
own the continuation queues and typed outer continuations; extracting
those boundaries privately keeps scoped-effect values simple and keeps
public APIs stable. The main risk is implementation churn inside the
wrapper substrates, which is why Option C is retained as an explicit
fallback rather than discarded.

**Implementation sequencing.** [plan.md step 7.4.4](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now splits into concrete substeps. Step 7.4.4a adds private carrier
raw-step extraction for default and erased shared wrappers. Step
7.4.4b adds the Explicit-family extraction boundary. Step 7.4.4c
wires the six wrapper interpreters through
`DispatchScopedCarrierHandlers` while preserving the ordinary
`DispatchScopedHandlers` route. Step 7.4.4d records the inactive
Option C fallback trigger.

## Resolved (2026-05-12): B37 Rc-family carriers need wrapper-specific bind bounds

**Disposition.** B37 surfaced during Phase 4 step 7.4.2c while
extending the H2 continuation carrier from default `Run` /
`RunExplicit` to `RcRun` and `RcRunExplicit`. The direct prototype
added cloneable `RcRunScopedContinuation` and
`RcRunExplicitScopedContinuation` carriers that store an action and its
typed outer continuation separately. That prototype is preserved in
`git stash` as
`wip(effects): rc scoped continuation carrier generic bound prototype`.

`just filtered check '^(error|warning|[[:space:]]*-->|note:)' -p fp-library --lib`
failed with `E0276` / `E0277`: the impl was stricter than the shared
`ScopedResume` trait, and the compiler could not prove the row
projection `Clone` obligations required by the Rc wrappers' `bind`
methods. `RcRun::bind` requires this projection to implement `Clone`:

```text
NodeBrand<R, S>::Of<'static, RcFree<NodeBrand<R, S>, RcTypeErasedValue>>
```

`RcRunExplicit::bind` requires the equivalent cloneable projection for
both the action result and the final result:

```text
NodeBrand<R, S>::Of<'a, RcFreeExplicit<'a, NodeBrand<R, S>, Action>>
```

The common `ScopedResume` trait had no place to express those
wrapper-specific method obligations. The same design pressure applies
to the later Arc-family carriers, where the hidden projection
obligations include `Send + Sync` propagation.

- **Resolution: Option A.** Split the private carrier protocol into
  wrapper-family traits. Keep `ScopedContinuation` as the shared
  wrapper-owned handle, but make the resume implementation contract
  family-specific: default erased, single-shot Explicit, Rc-shared,
  and Arc-shared carriers each get a private trait whose trait-level or
  impl-level bounds match that wrapper's substrate. The
  carrier-aware dispatcher path chooses the wrapper-specific trait at
  the private boundary rather than forcing every wrapper through one
  under-bounded trait.
- **Why-not Option B.** Pre-bound resume closures inside the Rc carrier
  would likely be the smallest local patch: the constructor could close
  over the action and outer continuation while the Rc projection
  `Clone` bounds are in scope, storing reusable `Rc<dyn Fn>` resume
  operations. That hides important substrate bounds behind dynamic
  dispatch and extra allocation, weakening the H2 goal that wrapper
  interpreters own a statically dispatched continuation carrier.
- **Why-not Option C.** Adding private raw-step / continuation-queue
  APIs to the Rc substrates may be a good later substrate symmetry
  improvement, but it is too large for the immediate carrier step and
  risks repeating the `FreeExplicit` hidden-intermediate-type problem
  for `RcFreeExplicit`.
- **Why-not Option D.** Restricting 7.4.2c to concrete proof rows would
  keep work moving only on paper. It would not prove the production
  generic carrier surface and would leave the same bounds problem to
  reappear during dispatcher wiring.

**Trade-off.** Option A adds private protocol surface and makes the
carrier-aware dispatcher family-indexed. That is the correct cost: the
six wrapper families already have different substrate obligations, and
the private protocol should expose those obligations where Rust can
check them instead of normalizing them through ad hoc closure storage
or a misleading common trait.

**Implementation sequencing.** [plan.md step 7.4.2c](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now starts with concrete substeps. Step 7.4.2c.0 adds the
family-specific private resume traits. Step 7.4.2c.1 migrates the
shipped default `Run` and `RunExplicit` carriers to the default-erased
and single-shot Explicit traits. Steps 7.4.2c.2 and 7.4.2c.3 implement
`RcRun` and `RcRunExplicit` carriers on the Rc-specific trait. Step
7.4.2c.4 records the fallback trigger: only if the family-indexed
static protocol still cannot express the Rc obligations without public
API churn should the plan revisit Option B's pre-bound closure carrier.
Step 7.4.2d uses the same family-indexed design for Arc carriers.

**Plan-text amendments.** [plan.md current progress](plan.md#current-progress)
now names 7.4.2c.0 as the next greenfield step, the active-blocker
section is empty, the resolved-blockers summary links this entry, and
the detailed Phase 4 phasing converts B37 into actionable 7.4.2c
implementation steps.

## Resolved (2026-05-11): B36 `RunExplicit` H2 carrier needs an existential-safe continuation boundary

**Disposition.** B36 surfaced after step 7.4.2a proved the H2 carrier
shape on default `Run`. The default erased substrate exposes
`RawRunFree` plus a homogeneous continuation queue:
`Continuation<NodeBrand<R, S>>` takes `TypeErasedValue` and returns
`RawRunFree`. That lets `RunScopedContinuation` append one post-action
raw continuation before reattaching the outer continuation queue.

`RunExplicit` does not currently expose the same boundary.
`FreeExplicit::bind` recursively maps the outer continuation into every
suspended layer. By the time `RunExplicit::interpret` sees a scoped
layer, the action program already contains the outer continuation. A
naive carrier based on `action.bind(post_action)` therefore places
post-action work after the outer continuation, reproducing the nested
Span ordering problem that B31/B32 exists to fix. Separating the action
result from that outer continuation requires a hidden intermediate
result type, which is the same existential class that made the B34
private visitor shape fail.

- **Resolution: Option A first.** Before implementing the
  `RunExplicit` carrier, add a half-day private prototype for an
  existential-safe Explicit substrate boundary. The proof must expose
  the action value and the action's outer continuation separately
  without `Any`, unsafe erasure, or dyn-generic callbacks, while
  preserving `FreeExplicit` public behavior and non-`'static` payload
  support. If the prototype succeeds, implement the private
  `RunExplicit` carrier on that proven boundary.
- **Fallback.** If the prototype still requires a callback generic over
  a hidden intermediate type, promote the H3 protocol-family path as
  the next concrete step. Do not ship a `bind`-based carrier as a
  temporary compatibility shim.
- **Why-not Option B.** A simple `RunExplicit::bind` carrier is small
  but semantically wrong for around-action handlers: it inserts
  post-action work after the outer continuation has already run.
- **Why-not Option C immediately.** H3 protocol families may be the
  right final architecture if the private Explicit boundary fails, but
  adopting them before the prototype would expand the public/protocol
  design surface without proving that the private path is impossible.
- **Why-not Option D.** Redesigning `FreeExplicit` around first-class
  continuation frames is too large to make the immediate next step.
  It remains a possible substrate rewrite only if both the private
  boundary and H3 facade approaches prove insufficient.

**Trade-off.** Option A preserves momentum without accepting another
status-quo-preserving shim. It acknowledges that default `Run`'s
successful proof does not transfer automatically to `RunExplicit`,
because `FreeExplicit`'s non-`'static` support removes the erased queue
that made the default proof easy.

**Implementation sequencing.** [plan.md step 7.4.2b](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now splits the `RunExplicit` carrier proof into concrete substeps:
7.4.2b.0 prototypes the existential-safe boundary, 7.4.2b.1 implements
the carrier on the proven boundary, and 7.4.2b.2 promotes H3 if the
prototype still hits the generic-callback-over-hidden-type wall.

**Implementation outcome.** The Option A proof succeeded in Phase 4
steps 7.4.2b.0 / 7.4.2b.1. The private
`RunExplicitScopedContinuation` carrier stores the selected
`RunExplicit` action and its typed outer continuation separately, so
`resume_with_post_action` can run a result-preserving action program
before reattaching the outer continuation. Focused tests cover normal
resume, post-action insertion before the outer continuation, and a
borrowed action value. The H3 fallback did not trigger.

**Plan-text amendments.** [plan.md current progress](plan.md#current-progress)
now states that B36 is resolved via Option A, the active-blocker
section is empty, and the next greenfield work is the 7.4.2c Rc-family
carrier extension.

## Resolved (2026-05-11): B35 private visitor proof requires a generic callback on an existential bind node

**Disposition.** B35 surfaced while auditing the B34 `FreeExplicit`
private visitor proof. B34 tried to keep the continuation-boundary fix
inside the Explicit substrate by adding a private raw-step visitor, but
delayed heterogeneous bind requires storing a hidden intermediate type
`X` in an existential bind node. The visitor would then need a
callback generic over `X` to expose
`F<Box<FreeExplicit<'a, F, X>>>` and the pending
`X -> FreeExplicit<'a, F, A>` continuation without erasing `X`.
Stable Rust trait objects cannot provide methods generic over type
parameters; the existing `Coyoneda` documentation records the same
dyn-compatibility limitation for opening existential types.

- **Resolution: Option A.** Promote B32 H2 into Phase 4 step 7.4 now.
  Scoped dispatch grows an internal wrapper-owned continuation carrier
  (`ScopedContinuation` / `ScopedResume`, or equivalent names chosen
  during implementation) instead of trying to open an existential bind
  node through a generic trait-object visitor. Wrapper interpreters own
  the carrier when they peel a scoped suspension, and standard handlers
  use it to resume the active branch normally or insert
  result-preserving post-action behavior before the branch's outer
  continuation.
- **Why-not Option B.** A narrow result-preserving post-action
  insertion primitive would probably unblock Span lifecycle ordering
  fastest, but it is Span-shaped rather than a general continuation
  boundary. It would leave the next result-transforming around-action
  handler or custom-handler API to reopen the same architecture
  problem.
- **Why-not Option C.** A non-object existential tower with concrete
  generic wrapper types duplicates H2's complexity while avoiding the
  name. It would likely change wrapper-visible types and still require
  a larger interpreter rewrite.
- **Why-not Option D.** Unsafe non-`'static` erasure rejects the safety
  premise of the Explicit family.
- **Why-not Option E.** Dropping Explicit-wrapper parity leaves
  lifecycle semantics inconsistent across the six-wrapper API surface.

**Trade-off.** H2 is the largest available path, but it is now the
honest size of the problem. The B31-B35 sequence shows that preserving
continuation placement with six-wrapper parity is not a narrow
per-substrate helper once Explicit-family non-`'static` payloads and
heterogeneous continuations are involved. H2 centralises the invariant
in wrapper interpreter code and keeps H3-style public protocol classes
available later as facades, instead of multiplying special-purpose
escape hatches.

**Implementation sequencing.** [plan.md step 7.4](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now converts B35 into concrete implementation steps: design the
private carrier contract, prove it on default `Run`, prove it on
`RunExplicit` without the failed visitor shape, extend it to Rc and Arc
wrapper families, add the carrier-aware scoped-handler path, wire the
six wrapper interpreters, migrate `SpanDispatcher`, and add focused
regression tests for nested Span ordering, result propagation,
borrowed Explicit payloads, Rc multi-shot behavior, and Arc Send/Sync
behavior.

**Plan-text amendments.** [plan.md current progress](plan.md#current-progress)
now states that B35 is resolved via Option A, the active-blocker
section is empty, H2 is part of Phase 4 step 7.4, and only H3 remains
as a Phase 6+ deferred public-facade revisit.

## Resolved (2026-05-11): B34 FreeExplicit raw steps need a non-static existential continuation boundary

**Disposition.** B34 surfaced while starting the B33 `FreeExplicit`
proof. B33 adopted continuation queues / raw-step decomposition for
the Explicit Free substrates, but the first design pass showed that
`FreeExplicit` cannot directly copy erased `Free`'s `Box<dyn Any>`
queue. Erased `Free` requires `A: 'static`, so it can store results
and continuations behind `Any` and expose a homogeneous
`FreeRawStep::Suspended` layer. `FreeExplicit` exists specifically to
support non-`'static` payloads, so it cannot hide the intermediate
result type that way.

The concrete problem is delayed heterogeneous bind. A delayed source
`FreeExplicit<'a, F, X>` followed by a continuation
`X -> FreeExplicit<'a, F, A>` may suspend before `X` has been
produced, while the enclosing program still has public result type
`A`. A raw-step enum cannot expose the suspended `F` layer without
either naming `X`, erasing `X`, or moving to a protocol that lets the
implementation reveal `X` only inside a generic callback.

- **Resolution: Option A with explicit H2 fallback.** Start with a
  private existential visitor raw-step proof inside the Explicit
  substrate. The visitor's suspended callback is generic over the
  hidden intermediate result type `X`, letting `FreeExplicit` expose
  the active suspension without `Any` and without leaking `X` into the
  public wrapper API. If the visitor cannot remain private to the
  Explicit substrates, or if it requires wrapper-wide, object-stored,
  or handler-list-visible carrier state, stop and reopen B32 H2 rather
  than continuing with a partial patch.
- **Why-not Option B.** A narrow result-preserving post-action
  insertion primitive probably covers Span lifecycle hooks, because
  Span's exit hook preserves the action result. It is still a
  special-purpose primitive and may not support future custom
  around-action handlers that need a general continuation boundary.
  Keeping it as a fallback is acceptable; making it the main path would
  repeat the status-quo-preserving patches that have been surfacing
  later blockers.
- **Why-not Option C as the first step.** Reopening H2 immediately may
  be the eventual answer, because an internal continuation carrier
  addresses the hidden-intermediate-type problem directly. It is also
  the largest rewrite. Try the private visitor proof first because it
  can still satisfy H1 without expanding the protocol surface; promote
  to H2 only if the proof leaks out of the substrate boundary.
- **Why-not Option D.** Unsafe non-`'static` erasure through raw
  pointers or unchecked casts would undermine the type-safety reason
  the Explicit family exists.
- **Why-not Option E.** Dropping Explicit-wrapper parity would finish
  the default erased path sooner, but it leaves Span lifecycle
  semantics inconsistent across the six wrapper families.

**Trade-off.** The adopted path is deliberately narrow but not
Span-specific. It tests whether `FreeExplicit` can preserve the
continuation boundary with a private existential protocol before the
project pays for the H2 carrier rewrite. The cost is that the proof
must be stopped quickly if it crosses its boundary: once the visitor
has to become wrapper-wide or handler-list-visible, the design has
already become H2 in practice and should be planned as H2.

**Implementation sequencing.** [plan.md step 7.4.2a](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now converts B34 into concrete implementation steps:
define the private raw-step visitor in `free_explicit.rs`, prove
delayed heterogeneous bind without naming or erasing `X`, preserve the
existing `FreeExplicit` API and non-`'static` payload support, add
substrate tests for inserted post-action ordering and borrowed payloads,
and pause to reopen H2 if the visitor cannot stay private and bounded.

**Plan-text amendments.** [plan.md current progress](plan.md#current-progress)
now states that B34 is resolved via Option A with an explicit H2
fallback, the active-blocker section is empty, and the next greenfield
work is the `FreeExplicit` private visitor proof.

## Resolved (2026-05-11): B33 Explicit substrates lack a continuation queue for H1 insertion

**Disposition.** B33 surfaced during the B32/H1 implementation audit.
B32 adopted substrate-level continuation insertion for B31's
around-action scoped handlers, but the audit found that the Explicit
Free substrates do not currently preserve the boundary H1 needs.
`FreeExplicit`, `RcFreeExplicit`, and `ArcFreeExplicit` implement
`bind` by recursively mapping the continuation into every suspended
layer immediately. After `action.bind(exit_outer)` is peeled to an
inner scoped action, `exit_outer` is already inside that nested action;
there is no pending continuation queue to splice a Span exit hook ahead
of.

- **Resolution: Option A.** Retrofit the Explicit substrates with
  continuation queues / raw-step decomposition. Start with the smallest
  `FreeExplicit` proof that preserves existing `pure`, `wrap`, `bind`,
  `map`, `to_view`, `evaluate`, and `Drop` behavior while exposing a
  raw step that keeps pending continuations outside suspended layers.
  If that proof stays bounded, extend the same shape to
  `RcFreeExplicit` and `ArcFreeExplicit`.
- **Why-not Option B.** An Explicit-wrapper sidecar around-action stack
  avoids rewriting the Explicit Free family, but it creates a local
  H2-style carrier only for Explicit wrappers. That diverges from H1
  and makes wrapper semantics harder to reason about.
- **Why-not Option C.** Limiting the lifecycle guarantee to erased
  wrappers is the smallest implementation, but it breaks six-wrapper
  parity and leaves a visible semantic hole in the standard
  `SpanDispatcher` set.
- **Why-not Option D.** Reopening H2 now could produce a unified
  scoped-continuation architecture, but it is a broader protocol and
  interpreter refactor than B32 intended. It remains the fallback if
  the `FreeExplicit` proof shows the substrate retrofit is unstable or
  conceptually wrong.
- **Why-not Option E.** Deferring Span lifecycle ordering leaves B31
  unresolved in practice and weakens the standard scoped-effect
  semantics just as lifecycle coverage is being completed.

**Trade-off.** Option A is larger than the originally expected H1
helper because it touches core Explicit substrate representation and
the wrapper peel paths. It is still the more direct fix than H2 for the
current problem: the bug is that the Explicit substrates erase the
continuation boundary too early, so the substrate should preserve that
boundary rather than asking wrapper-specific interpreter state to
reconstruct it later.

**Implementation sequencing.** [plan.md step 7.4](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now converts B33 into concrete implementation steps: audit the H1
insertion primitive, prove a `FreeExplicit` delayed-continuation /
raw-step retrofit, extend it to `RcFreeExplicit` and
`ArcFreeExplicit`, then add erased-family insertion helpers, the
continuation-aware scoped-handler path, wrapper wiring, Span migration,
and focused regression tests.

**Plan-text amendments.** [plan.md current progress](plan.md#current-progress)
now states that B33 is resolved via Option A, the active-blocker
section is empty, and the next greenfield work is the `FreeExplicit`
continuation-boundary proof.

## Resolved (2026-05-10): B32 Explicit-wrapper continuation-aware action runner lifetime wall

**Disposition.** B32 surfaced while implementing the B31
continuation-aware scoped-handler path for `Span`. The first prototype
added a compatible `dispatch_scoped_with` /
`dispatch_scoped_head_with` hook, then attempted to pass an action
runner that recursively interpreted the scoped action through borrowed
first-order and scoped handler lists. That shape was plausible for
non-explicit wrappers, but the Explicit-family wrappers
(`RunExplicit`, `RcRunExplicit`, and `ArcRunExplicit`) failed when the
action runner called `action.interpret(&handlers, &scoped_handlers)`:
rustc required the wrapper lifetime `'a` to outlive `'static`.

The failing prototype remains preserved in the named git stash
`wip(effects): b31 continuation-aware scoped handler prototype`. The
nested-Span ordering experiment remains preserved in
`wip(effects): span nested lifecycle ordering experiment`.

- **Resolution: Option D / H1.** Implement the B31 around-action path
  through substrate-level continuation insertion first. The primitive
  should let an around-action handler insert its post-action hook before
  the action's pending outer continuation queue, without recursively
  interpreting the action through borrowed handler lists. This addresses
  the bug at the level where it exists: the Free-family continuation
  boundary.
- **Why-not Option A.** Limiting the path to default / Rc / Arc erased
  wrappers would be the smallest implementation, but it violates the
  six-wrapper parity goal and leaves Explicit Span semantics weaker
  than the rest of the standard dispatcher set.
- **Why-not Option B.** Keeping the recursive action-runner design and
  adding explicit reference-based interpreter helpers preserves the
  original mental model, but it likely fights the same lifetime wall
  the prototype already exposed. It also keeps reentrant interpretation
  as the core mechanism even though the required ordering is really a
  continuation-queue placement issue.
- **Why-not Option C.** Requiring clonable / owned handler lists avoids
  borrowing through the Explicit wrapper lifetime, but it adds
  undesirable bounds to handler lists and closure captures and diverges
  from the existing `handlers!` / `scoped_handlers!` pattern.
- **Why-not Option E.** Trait-object handler contexts could erase the
  problematic concrete handler-list type, but they lose static dispatch
  on a central interpreter path and do not directly solve the
  continuation-placement invariant.

**Holistic architecture note.** B31 and B32 expose a real design split:
ordinary scoped handlers can produce the next program directly, while
around-action handlers need to place post-action behavior before an
action's outer continuation. The adopted H1 path fixes the missing
substrate primitive now without widening the whole public protocol.

- **H1, adopted now.** Add substrate-level continuation insertion in
  the Free-family substrates and wrapper adapters. This preserves the
  static handler-list API, keeps six-wrapper parity, and gives Span the
  ordering it needs.
- **H2, deferred.** Redesign scoped dispatch around an internal
  wrapper-owned `ScopedContinuation` / `ScopedResume` carrier. Each
  wrapper interpreter would create a control value when it peels a
  scoped suspension; handlers would ask that carrier to resume a branch
  normally or insert post-action behavior before the outer
  continuation. This centralises continuation attachment invariants and
  can host ordinary scoped resumption, around-action wrapping, and
  future variants behind one internal model. The cost is a broad
  refactor of `DispatchScopedHandler`, `DispatchScopedHandlers`,
  wrapper interpreter plumbing, standard dispatchers, and docs.
- **H3, deferred.** Split scoped handlers into protocol families:
  preserve the current `DispatchScopedHandler` path for ordinary
  "produce the next program" handlers and add a sibling around-action
  protocol for Span-like handlers. This has a smaller migration surface
  and clearer public semantics for custom handlers, but it permanently
  increases trait, macro, and mixed-row routing surface and still needs
  H1 underneath for correct ordering.
- **H2 versus H3.** H2 can host H3-style public facades later because
  multiple user-facing handler classes can compile down to one internal
  carrier. H3 by itself cannot provide H2's single continuation-boundary
  invariant; it only classifies which handlers may ask for
  around-action placement. Therefore H2 is the stronger internal
  architecture if continuation-sensitive scoped effects keep appearing,
  while H3 is a useful public-surface strategy if custom handlers need
  explicit classes.

**Implementation sequencing.** [plan.md step 7.4](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
now converts B32 into concrete implementation steps: recreate the
nested-Span proof, audit Free-family insertion points, implement the H1
primitive, add the continuation-aware scoped-handler path on top of it,
wire all six wrappers, migrate `SpanDispatcher`, and add focused
ordering / result-propagation regressions. If the H1 primitive expands
into a broad Free-family redesign, implementation pauses and a new
active blocker is opened instead of silently switching to H2 or H3.

**Plan-text amendments.** [plan.md current progress](plan.md#current-progress)
now states that B32 is resolved, the active-blocker section is empty,
H2 / H3 live under Phase 6+ revisit criteria, and the next greenfield
work is Phase 4 step 7.4's substrate-level continuation insertion.

## Resolved (2026-05-10): B31 Span nested lifecycle exit ordering under the public scoped-handler shape

**Disposition.** B31 surfaced during Phase 4 step 8 while expanding
Span lifecycle coverage beyond result propagation. A public custom
scoped handler prototype that records `enter`, returns
`action(()).bind(exit)`, and then interprets nested spans observes:

```text
enter outer, enter inner, exit outer, exit inner
```

The desired around-action instrumentation order is:

```text
enter outer, enter inner, exit inner, exit outer
```

The current `DispatchScopedHandler` shape asks a handler to return the
next program. It does not hand the scoped handler a continuation/runner
that interprets the action under the current first-order and scoped
handler context before the handler appends exit behavior. That shape is
sufficient for action-result propagation, but not for public Span
semantics that claim stack-like nested enter/exit ordering.

- **Resolution: Option B.** Add a continuation-aware scoped-handler
  path for around-action handlers. This path may be a sibling trait or
  carrier next to `DispatchScopedHandler` / `DispatchScopedHandlers`;
  its contract is that an around-action handler receives the scoped
  layer plus a continuation/runner for the action, so it can record or
  perform pre-action behavior, run the action to completion in the
  current handler context, and then record or perform post-action
  behavior before returning the action result.
- **Why-not Option A.** Accepting current ordering would keep the
  shipped dispatcher API unchanged, but it would make Span a
  tag-carrying resumption effect rather than true around-action
  instrumentation. That conflicts with the intended Span lifecycle
  tests and would likely surprise users.
- **Why-not Option C.** A Span-only instrumentation API would be a
  smaller patch, but it would encode a one-off control-flow path for
  the first standard effect that needs around-action semantics. A
  sibling continuation-aware handler path is a better substrate for
  future around-action scoped effects.
- **Why-not Option D.** Deferring observable Span lifecycle semantics
  would keep Phase 4 moving, but it would leave a visible semantic gap
  in the standard scoped-effect set after the rest of the dispatcher
  set already has end-to-end lifecycle coverage.
- **Implementation sequencing.** [plan.md step 7.4](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
  now contains the concrete rollout: prototype the continuation
  boundary, add the core continuation-aware trait/carrier, wire all six
  wrappers, migrate `SpanDispatcher`, and add focused nested ordering
  tests before finishing the remaining step 8 Span lifecycle coverage.
- **Preserved evidence.** The exploratory failing nested-Span test was
  preserved in the named git stash `wip(effects): span nested lifecycle
ordering experiment` so the implementation can reapply or recreate
  it as the first regression.
- **Plan-text amendments.** [plan.md current progress](plan.md#current-progress)
  now states that B31 is resolved via Option B, the active blocker
  section is empty, and the next greenfield work is Phase 4 step 7.4.

## Resolved (2026-05-09): B30 Box-backed CatchDispatcher single-shot continuation

**Disposition.** B30 surfaced during Phase 4 step 7.1 when the
interpose-backed `CatchDispatcher` shape passed on Rc/Arc wrappers but
failed on default `Run`. `Run::peel` uses `Free::to_view`, which maps
the remaining single-shot erased `Free` continuation into the suspended
`BoxCatch` layer. Because `BoxCatch` has both a protected action thunk
and a recovery handler thunk, a real catch dispatch can need the action
first and then the handler. Mapping the same continuation into both
branches trips the substrate guard with `Free::to_view map called more
than once`.

- **Resolution: Option C.** Add a continuation-aware raw scoped-step
  path for default `Run`. The raw step exposes the suspended
  `Node` layer with `Free<NodeBrand<R, S>, TypeErasedValue>` branch
  programs while keeping the pending continuation queue outside the
  layer. Box-backed `CatchDispatcher` chooses the action or recovery
  branch first, then attaches the continuation queue exactly once.
- **RunExplicit nuance.** `RunExplicit` has no erased `Free`
  continuation queue; its recursive explicit substrate can use the
  ordinary scoped dispatcher shape. The Box-backed `RunExplicit`
  `CatchDispatcher` still uses a single-shot handler cell internally
  because the stored recovery handler is `Box<dyn FnOnce>`, while
  `interpose` accepts an `Fn` replacement closure.
- **Why-not Option A.** Keeping standard Catch dispatch Rc/Arc-only
  would make the default wrappers less capable than the shared-pointer
  wrappers and leave the six-wrapper scoped-handler story false.
- **Why-not Option B.** A runtime single-shot cell around only the
  recovery handler targets the wrong value. The duplicated value was
  the erased `Free` continuation installed by `peel`, not merely the
  user recovery closure.
- **Why-not Option D.** Reopening the Box-backed scoped-effect
  representation would be cleaner if the continuation boundary could
  not be exposed, but the POC and production implementation showed the
  raw-step path is sufficient for Catch.
- **Evidence.** `fp-library/tests/run_standard_scoped_handlers.rs` covers
  default `Run`, `RunExplicit`, `RcRun`, and `ArcRun` for both
  successful recovery through a nested `Span` and recovery-handler
  rethrow escaping the same `Catch` frame. The original proof-of-concept
  remains in `Free` unit tests as a focused substrate regression.
- **Plan-text amendments.** [plan.md step 7](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
  now records that default `Run` uses the raw scoped-step path for
  Box-backed Catch while Rc/Arc wrappers stay on scoped-row-preserving
  interpose. The active blocker section is empty.

## Resolved (2026-05-09): B29 scoped dispatcher architecture checkpoint

**Disposition.** B29 escalated from B28 after the `CatchDispatcher`
lifetime/witness question looked like part of a broader standard
scoped-dispatcher architecture issue. The checkpoint prototyped
standard dispatcher signatures before production step 7 implementation.

- **Prototype evidence.** `poc_standard_scoped_handler_architecture.rs`
  validates the adopted shape for `RcRun` across `Catch`, `Local`,
  `RefLocal`, `Span`, `Bracket`, and `RefBracket`. It also validates
  the same lifetime/evidence pattern for `RcRunExplicit<'a>` on the
  highest-risk interpose-backed dispatchers: `Catch` and `Local`.
- **Resolution: Option A plus Option B surface polish.** Production
  scoped dispatcher implementation should use the wrapper's actual
  peeled-layer lifetime: `'static` for erased wrappers and the wrapper
  lifetime `'a` for Explicit wrappers. Interpose-backed dispatchers
  carry row-removal evidence at the dispatcher type level:
  `CatchDispatcher<Idx, RMinusE, EmbedIndices>`,
  `LocalDispatcher<Idx, RMinusE, EmbedIndices>`, and
  `RefLocalDispatcher<Idx, RMinusE, EmbedIndices>`. `SpanDispatcher`
  stays witness-free. `BracketDispatcher` and `RefBracketDispatcher`
  stay result-specific because their scoped brands are
  `BracketBrand<P, Sub, A, B>` and `RefBracketBrand<P, Sub, A, B>`.
  Add constructor/helper functions during production implementation so
  callers can write helper calls rather than naming witness-bearing
  dispatcher structs directly.
- **Why-not richer carrier now.** Reopening the B27 scoped-aware
  short-circuit carrier would add a new primitive across all wrappers
  before there is a second concrete user. The POC confirms `interpose`
  is sufficient for the standard scoped dispatchers.
- **Why-not restrict support.** Restricting `CatchDispatcher` to
  explicit wrappers or to `S = CNilBrand` would leave nested scoped
  actions unsupported inside catch and make the standard dispatcher set
  inconsistent across wrappers.
- **Plan-text amendments.** [plan.md step 7](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
  now calls out the production lifetime-bound adjustment and helper
  constructor layer before the dispatcher rollout proceeds.

## Resolved (2026-05-09): B27 `interpret_with_either` scoped-suspension return path

**Disposition.** B27 surfaced during the Phase 4 step 6a implementation
scoping pass. The plan had grouped `interpret_with_either` with
`interpret_with` and `interpose` as a scoped-row-preserving primitive,
but `interpret_with_either` is a terminal short-circuit loop returning
`Result<A, EBrand::Op<Run<..., A>>>` (and parallel wrapper forms). That
return type can represent pure completion or a matched first-order
operation, but it cannot represent a suspended `Node::Scoped` layer plus
its continuation.

- **Resolution: Option A.** Keep `interpret_with_either`
  scoped-row-empty / first-order-only (`S = CNilBrand`) and build
  `CatchDispatcher` on scoped-row-preserving `interpose`. `interpose`
  returns a program, so it can preserve nested scoped operations while
  replacing `Except::Throw(e, _)` with the catch handler result.
- **Why-not Option B.** A richer scoped-aware short-circuit carrier
  would allow future code to observe the first matching first-order
  operation while preserving scoped suspensions, but it would add a new
  API shape across all six wrappers before there is a second concrete
  user. The added proof surface is not justified for Phase 4's standard
  handler rollout.
- **Why-not Option C.** Restricting `CatchDispatcher` to actions with
  `S = CNilBrand` is smaller but breaks nested scoped effects inside
  `catch`, contradicting the dual-row design.
- **Plan-text amendments.** [plan.md Phase 4 step 6a](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
  keeps `interpret_with_either` first-order-only; [plan.md step 7.1](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
  builds `CatchDispatcher` on `interpose`; [plan.md Phase 6+](plan.md#phase-6-deferred-not-in-this-plan)
  records the richer scoped-aware short-circuit primitive as a deferred
  follow-up.

## Resolved (2026-05-09): Phase 4 step 6a / 7 scoped-row primitive and dispatcher semantics; B24 / B25 / B26

**Disposition.** B24-B26 surfaced during the pre-step-7 code audit after
the scoped-handler carrier and macros shipped. The audit found one
missing substrate primitive surface (B24) and two dispatcher semantic
questions (B25, B26). User confirmation on 2026-05-09 adopts the
recommended v1 paths and converts them into actionable Phase 4 steps:
step 6a for scoped-row-preserving primitives, step 7.2 for Local /
RefLocal, and step 7.3 for Bracket / RefBracket.

Follow-up implementation scoping surfaced B27: the
`interpret_with_either` portion of B24 cannot preserve non-empty scoped
rows with its current terminal return type. B27 is resolved above via
Option A: keep `interpret_with_either` first-order-only and use
scoped-row-preserving `interpose` for `CatchDispatcher`; B24 remains
adopted for `interpret_scoped_with`, `interpret_with`, and `interpose`.

### B24. Scoped-row-preserving primitives missing before standard dispatchers

- **Issue.** Q3 adopted
  `interpret_scoped_with::<EBrand, Idx, SMinusE>` as the scoped-row
  narrowing primitive, but the wrappers did not actually expose it. The
  existing `interpret_with`, `interpose`, and `interpret_with_either`
  primitives were scoped-row-empty only (`S = CNilBrand`), so standard
  scoped dispatchers would otherwise need to duplicate traversal logic
  or reject nested scoped operations.
- **Resolution: Option A.** Add a scoped-row-preserving primitive
  retrofit before standard dispatchers. Implement per-wrapper
  `interpret_scoped_with`; generalise `interpret_with` and `interpose`
  to preserve non-empty scoped rows by mapping `Node::Scoped`
  recursively, or ship explicitly named scoped-aware siblings if
  generalising the existing method names creates inference regressions.
  Validate first on `Run` and `RcRun`, then fan out across all six
  wrappers using the existing Arc-family HRTB workaround pattern where
  necessary. `interpret_with_either` is split out to B27 because its
  current return type cannot carry a suspended scoped layer; B27 keeps
  it first-order-only.
- **Why-not alternatives.** Dispatcher-local walkers duplicate substrate
  traversal and make custom scoped handlers less capable than standard
  handlers. Restricting standard scoped dispatchers to `S = CNilBrand`
  breaks nested scoped-effect semantics and contradicts the dual-row
  design.
- **Plan-text amendments.** [plan.md Phase 4 step 6a](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
  now contains the concrete retrofit steps and tests.

### B25. Local / RefLocal dispatcher environment protocol and clone bounds

- **Issue.** Step 7 said `Local` temporarily modifies the environment
  returned by the first-order `Reader` handler, but did not specify how
  the dispatcher obtains the environment, answers `Reader::Ask` inside
  the action, restores state, or states `E` bounds. Current
  `Reader::Ask` supplies `E` by value, so repeated asks require repeated
  environment values.
- **Resolution: Option A for v1.** Implement Local / RefLocal by
  scoped-row-preserving Reader interposition. The dispatcher obtains the
  current environment, computes the local environment through `E -> E`
  or `&E -> E`, then answers by-value Reader asks inside the action with
  the modified environment. Because the current Reader is by-value,
  repeated asks may require `E: Clone`. RefLocal's guarantee is narrower
  but still useful: computing the modified environment borrows the
  parent `E` rather than consuming or cloning it.
- **Long-term follow-up.** A borrow-oriented Reader effect is deferred
  to Phase 6+ for true no-`E: Clone` repeated environment access.
- **Why-not alternatives.** Adding borrow-oriented Reader before step 7
  expands the first-order effect surface and blocks scoped dispatcher
  progress. A shared mutable environment carrier couples standard
  dispatchers to handler internals and still cannot answer repeated
  by-value asks without cloning or moving the environment.
- **Plan-text amendments.** [plan.md Phase 4 step 7.2](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
  defines the v1 dispatcher path; [plan.md Phase 6+](plan.md#phase-6-deferred-not-in-this-plan)
  records the borrow-oriented Reader follow-up.

### B26. Bracket guard semantics conflict with effectful release payloads

- **Issue.** Earlier Q5 text described a `BracketGuard` whose `Drop`
  invoked `release` synchronously, but the shipped Bracket /
  RefBracket cells store `release` as a closure returning a
  `Free` / `RcFree` / `ArcFree` program over the effect substrate. A
  `Drop` impl cannot generically interpret that program with the current
  handler lists, and dropping the returned program is not equivalent to
  running its effects.
- **Resolution: Option A.** Adopt two-tier semantics. On normal
  completion, `BracketDispatcher` sequences acquire -> body ->
  effectful release and returns the body result after release runs. On
  panic/unwind, the library guarantees only ordinary Rust resource
  `Drop` behavior for the acquired resource and any cleanup encoded in
  that resource's own `Drop`; it does not claim to interpret the
  effectful release program during unwinding.
- **Long-term follow-up.** If users need panic-time cleanup beyond
  resource-owned `Drop`, add a separate synchronous panic-finalizer hook
  later. Do not replace the normal-path effectful release program with a
  synchronous-only closure.
- **Why-not alternatives.** Rewriting release into a synchronous cleanup
  closure would remove effectful release from the public Bracket API.
  `catch_unwind` adds `UnwindSafe` constraints, misses aborting panics,
  and reopens the Q5 rejection rationale.
- **Plan-text amendments.** [plan.md Phase 4 step 7.3](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
  defines the normal-path and panic-path semantics; [plan.md Phase 6+](plan.md#phase-6-deferred-not-in-this-plan)
  records the optional panic-finalizer follow-up.

## Resolved (2026-05-08): Phase 4 step 5b `define_scoped_row!` item macro adopted; B23 closed via Option B

**Disposition.** B23 surfaced after Phase 4 step 5's base
`scoped_effects!` / `scoped_handlers!` macro implementation. B18 had
recorded a planned step 5 sub-task for marker-struct generation over
recursive Bracket-containing scoped rows. The shipped
`scoped_effects![...]` macro is intentionally a type-position row macro
parallel to `effects![...]`, so it cannot also emit named item-level
marker structs and trait impls. Closed on user confirmation via Option
B: keep `scoped_effects![...]` as the type-position macro and add a
separate item-position `define_scoped_row!` macro as Phase 4 step 5b.

### B23. `scoped_effects!` cannot generate Bracket marker rows with its current type-position syntax

- **Issue.** Rows containing Bracket-family brands may need
  `Sub = NodeBrand<R, S>`, where `S` is the row currently being
  defined. Rust rejects direct recursive type aliases for those rows
  (B18), but accepts the marker-struct workaround validated in
  `fp-library/tests/poc_bracket_marker_row.rs`.
  A type-position macro can expand to a row type, but it cannot create
  the marker struct plus its `Kind` / `WrapDrop` / functor impls at the
  item level.

- **Resolution: Option B (separate item-position macro for named
  scoped rows).** Add `define_scoped_row!` as the marker-row macro and
  leave `scoped_effects![...]` unchanged for direct type-position row
  assembly. The split keeps macro positions explicit: use
  `scoped_effects![...]` when a type alias is enough, and
  `define_scoped_row! { ... }` when recursive Bracket rows need a
  named marker with delegating impls.

- **Adopted v1 syntax and semantics.**
  - Concrete rows only:

    ```rust,ignore
    define_scoped_row! {
        pub struct MyScopedRow;
        [
            BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, Self>, i32, i32>,
            BoxCatchBrand<BoxBrand, MyError>,
        ]
    }
    ```

  - Bare `Self` in the row body is a macro-local placeholder for the
    generated marker row. The macro substitutes `Self` structurally
    before lexical sorting so row order stays aligned with
    `scoped_handlers!`.
  - The macro generates the marker struct and delegating `Kind`,
    `WrapDrop`, `Functor`, `SendFunctor`, and `RefFunctor` impls. The
    delegation impls are guarded by the corresponding underlying-row
    trait bounds, so rows that only support the Arc-family
    `SendFunctor` path are not over-constrained by Rc/Box-only
    `RefFunctor` availability.
  - Generic scoped rows are explicitly deferred. Revisit generic row
    parameters only when a concrete standard-handler or custom-effect
    use case requires them.

- **Why-not-alternatives summary.**
  - **Option A (document manual marker rows for now):** rejected as the
    default path because it leaves B18's planned macro automation
    unshipped and makes Bracket-containing rows too boilerplate-heavy
    for users and tests.
  - **Option C (change or overload `scoped_effects!` into an item
    macro):** rejected because it conflates type-position and
    item-position macro contexts and breaks the clean mental model that
    `effects![...]` and `scoped_effects![...]` are row type macros.
  - **Attribute/derive macro over a marker struct:** viable but not
    adopted. It would also solve the item-position problem, but the
    repository's effects macro surface already uses function-like
    macros, and `define_scoped_row!` can own the whole syntax more
    directly.

- **Plan-text amendment.** Phase 4 step 5 is split into the already
  shipped base row/list macros and step 5b `define_scoped_row!`. Step
  5b lands before Phase 4 step 7's standard scoped-handler
  implementations. B20 remains separate: `define_scoped_row!` removes
  marker-row boilerplate and type-alias recursion, but it does not by
  itself redesign the `ArcRun::bracket` Send+Sync overflow path.

## Resolved (2026-05-08): Phase 4 step 4 `dispatch_scoped<FOH>` method-generic viability; Q4 closed via Option A

**Disposition.** Q4 was the first Phase 4 step 4 implementation
risk: the scoped-dispatch trait sketch needs a method generic over the
concrete first-order handler-list type, with a bound requiring that
type to implement the existing `DispatchHandlers` trait for the active
first-order row layer. Closed by
`fp-library/tests/poc_dispatch_scoped_method_generic.rs`,
which compiles and passes. The POC defines a local
method-generic dispatch method on an `RcRun` `Span` scoped-handler
prototype, takes `&impl DispatchHandlers<'a, FirstLayer<'a>, Prog>`,
and dispatches into the tail of a real two-cell `handlers!` cons-list.
The earlier named-`FOH` sketch and the production argument-position
`impl Trait` spelling are the same static-dispatch shape for this use:
the first-order handler-list type remains a method-level generic.

### Q4. `dispatch_scoped<FOH>` method-generic viability

- **Issue.** Method-level type generics are stable Rust, but the
  future scoped-handler trait needs the FOH bound to stay satisfiable
  when the method body calls `fo_handlers.dispatch(...)` against the
  existing recursive `DispatchHandlers` cons-list impls. If rustc
  required higher-ranked type polymorphism over FOH, the static
  dispatch design would hit the same class of wall previously seen in
  the F2A investigation.

- **Resolution: Option A (validate by prototype before dispatcher
  implementation).** The prototype validates the intended static
  dispatch shape before landing production APIs. It uses an
  `RcRun<FirstRow, ScopedRow, i32>` program whose scoped row contains
  an Rc-backed `Span`, then calls a scoped-handler method whose
  first-order handler-list parameter is generic at the method level.
  Inside the method, it builds a first-order row layer for a two-effect
  row and dispatches it through a real `handlers!` value. The row is
  ordered to match the macro's canonical lexical brand order, and the
  dispatched operation sits in the tail so the recursive cons-list
  implementation is exercised. The production trait spells the generic
  first-order handler-list parameter as `&impl DispatchHandlers<...>`
  to match the repository's anonymous-generic style where the type
  parameter is not referenced elsewhere.

- **Why-not-alternatives summary.**
  - **Option B (take `&dyn DispatchHandlers<...>`):** not needed. The
    prototype compiles with the static generic method shape, preserving
    monomorphisation and handler inlining opportunities.
  - **Option C (lift FOH to a brand-level type parameter):** not
    needed. Method-level FOH keeps scoped-handler values reusable
    across concrete handler-list instances while still satisfying the
    existing dispatch bound.

- **Plan-text amendment.** Phase 4 step 4 no longer starts with a
  pending Q4 risk. Proceed with the production
  `DispatchScopedHandlers` trait, scoped-handler cons-list carriers,
  and wrapper interpreter plumbing using the Q4-proven static-dispatch
  method shape (`&impl DispatchHandlers<...>` in production code). If
  the full implementation later surfaces a new concrete compiler wall,
  record a new active blocker before switching to a trait-object
  fallback.

## Resolved (2026-05-08): Phase 4 step 3.4 Span tag storage and clone/send bounds; B22 closed via Option A

**Disposition.** B22 surfaced before Phase 4 step 3.4 (Span) implementation began. B21 had adopted by-value tag storage plus a thunked action, but Span is the first standard scoped cell whose user data lives directly in the cell rather than only inside closure captures. Owned `Functor`, `WrapDrop`, and `Extract` can move the tag without extra bounds, but Rc/Arc substrates clone cells by refcounting their action thunks; a by-value tag must be cloned too. Arc-family rows additionally require the projected cell to be `Send + Sync`, so the tag's auto-traits are part of the public bound surface. Closed on user confirmation in this session via Option A: keep by-value tags and add clone/send bounds only where required.

### B22. Span tag storage and clone/send bounds

- **Issue.** Span's by-value `tag: Tag` field preserves the intended data shape, but Rc/Arc cell clone and ref-map paths cannot clone the cell unless the tag is cloneable. Arc-family cells also cannot satisfy the Arc substrate's thread-safety requirements unless the tag is `Send + Sync`. The plan needed to decide whether those bounds are local to the affected substrates or imposed globally.

- **Resolution: Option A (keep by-value tags and add bounds only where required).** `BoxSpan` keeps `Tag: 'a` only, so default `Run` / `RunExplicit` remain usable with non-`Clone` tags. Rc-backed `Span` paths require `Tag: Clone` where cell `Clone`, `RefFunctor`, or wrapper smart constructors need cloneable scoped rows. Arc-backed `SendSpan` paths require `Tag: Clone + Send + Sync` where the Arc substrate requires cloneable, thread-safe cells.

- **Why-not-alternatives summary.**
  - **Option B (store the tag behind the pointer brand):** rejected because it adds an allocation for every span tag, weakens the by-value data shape, and complicates dispatcher access by forcing handlers to observe a pointer-wrapped tag or dereference/clone it explicitly. It makes the default Box path worse without solving a default-wrapper problem.
  - **Option C (require `Tag: Clone` on every Span constructor):** rejected because it over-constrains single-shot default wrappers where the tag is never cloned.

- **Plan-text amendment.** Step 3.4 keeps by-value tag storage across all Span cells. The default Box-backed cell and smart constructors do not impose `Tag: Clone`; Rc-backed implementations impose `Tag: Clone` only where cloneable cells are required; Arc-backed implementations impose `Tag: Clone + Send + Sync` only where cloneable, thread-safe cells are required. `decisions.md` is amended to document the asymmetric tag-bound surface. A deviations.md entry should be added when the code lands, recording the sibling-cell choice from B21 plus the asymmetric tag bounds from B22.

## Resolved (2026-05-08): Phase 4 step 3.4 Span action storage versus no-pointer-brand shorthand; B21 closed via Option A

**Disposition.** B21 surfaced before Phase 4 step 3.4 (Span) implementation began. The adopted scoped-effect table described Span as `tag: Tag` plus `action: Run<R, S, A>`, with no Ref flavour and no pointer-brand parameter because no user closure is dispatched over. That shorthand remains correct for the public API, but the implementation cannot literally store the action program by value inside a scoped row that may itself contain Span: it would repeat the recursive layout cycle Catch and Local avoid by storing action programs behind unit-argument B-thunks. Closed on user confirmation in this session via Option A: mirror Catch and Local at the substrate level.

### B21. Span action storage versus no-pointer-brand shorthand

- **Issue.** Span is Val-only and has no Ref dispatch split, but it still carries a nested action program. A direct `action: A` field is expected to produce the same infinite-size recursive layout problem as Catch and Local. Once the action is stored behind a thunk, the closure-storage shape differs by wrapper: default wrappers use Box-backed FnOnce storage, Rc wrappers use Rc-backed Fn storage, and Arc wrappers use Arc-backed Send + Sync Fn storage.

- **Resolution: Option A (mirror Catch and Local at the substrate level).** Add Box/Rc/Arc sibling cells and brands for Span. Store the tag by value and the action as a unit-argument B-thunk. Public smart constructors remain one Val-only `span` operation per wrapper; the pointer split is an implementation-level storage detail, not a Ref flavour.

- **Why-not-alternatives summary.**
  - **Option B (direct pointer indirection instead of a thunk):** rejected because mapping and extraction become awkward. Box cannot move the action out through shared references, while Rc/Arc direct storage either needs clone-heavy action programs or runs into owned-value extraction limits. It also diverges from the established B-thunk helper APIs.
  - **Option C (store `action: A` directly):** rejected because it is expected to re-open the recursive layout cycle already solved for Catch and Local.

- **Plan-text amendment.** Step 3.4 ships Span as a Val-only operation with no Ref dispatch split, but implementation uses sibling cells and brands for the three closure-storage families:
  1. Default Run / RunExplicit pair with Box-backed action-thunk cells.
  2. RcRun / RcRunExplicit pair with Rc-backed action-thunk cells.
  3. ArcRun / ArcRunExplicit pair with Arc-backed Send + Sync action-thunk cells.

  Each cell stores `tag` by value, maps only over the thunked action, and implements the same substrate-required trait set as the other scoped-effect brands. `decisions.md` is amended to clarify that "no `P` parameter" is the user-facing semantic shape, while the per-pointer B-thunk split is the implementation-level storage shape. A deviations.md entry should be added when the code lands, recording the sibling-cell choice and any Free-family split details required by the implementation.

## Resolved (2026-05-08): Phase 4 step 3.3.4 `ArcRun::bracket` integration tests blocked by rustc Send+Sync overflow; B20 closed via Option A (skip `ArcRun::bracket` integration tests, defer to step 8 bracket dispatcher tests; escalate to Option D `SendBracketBrand` redesign if step 8 still cannot exercise it)

**Disposition.** B20 surfaced during the pre-implementation blocker scan for step 3.3.4 (Bracket Val integration tests). An empirical probe (created and deleted) verified that `ArcRun::bracket`'s marker-struct integration test fails with the same rustc overflow that the doctest hits, regardless of `recursion_limit` (tested up to 8192). Closed on user confirmation in this session via Option A: skip `ArcRun::bracket` integration tests; defer to step 8 (which tests the step 7 bracket dispatcher; the dispatcher's API surface may not require constructing a user-facing scoped row that exposes `SendBracketBrand` to the type system in the cycling way); if step 8 still cannot exercise it, escalate to Option D (redesign `SendBracketBrand` to drop the GAT-Send-Sync bound on `Sub`) at step 8a.

### B20. `ArcRun::bracket` marker-struct integration tests blocked by rustc Send+Sync overflow on `SendBracketBrand`'s GAT-Send-Sync bound

- **Issue.** Step 3.3.4 was scoped for ~22 shape-only integration tests at `fp-library/tests/run_bracket.rs` across the six Run wrappers. The tests need user-facing scoped rows containing the wrapper's bracket brand to verify peel-shape and cell extraction. For 4 of 6 wrappers (`Run`, `RcRun`, `RunExplicit`, `RcRunExplicit`), the marker-struct workaround validated by the `B18 POC` compiles and runs as integration tests. For `ArcRunExplicit`, the marker struct works with `#![recursion_limit = "512"]`. For `ArcRun::bracket`, the marker struct fails with `error[E0275]: overflow evaluating the requirement Node<...>: Send`. The cycle: [`SendBracketBrand`](../../../fp-library/src/brands/effects.rs)'s `Kind` impl bound requires `Sub: Kind...<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync>`. For `Sub = NodeBrand<CNilBrand, ScopedRow>`, this triggers a recursive `Send + Sync` evaluation: `<NodeBrand<...> as Kind>::Of<...>` -> `<ScopedRow as Kind>::Of<...>` -> marker's `Of` projection through `UnderlyingRow` -> `SendBracketBrand` again -> back to the bound check. Empirically verified: `recursion_limit = 8192` in the test crate does not help (probe deleted). The cycle is structural, not just deep; doctest-vs-integration-test makes no difference (the same compile-time cycle triggers in both contexts). The smart constructor itself compiles cleanly via `cargo check` (verified during step 3.3.3 implementation): the limitation is purely test-fixture-construction.

- **Resolution: Option A (skip `ArcRun::bracket` integration tests for step 3.3.4; defer end-to-end exercise to step 8 which tests the step 7 bracket dispatcher).** Three reasons:
  1. **The smart constructor's correctness is already established by other means.** `cargo check` verifies the type signature; brand-level impls (Functor / SendFunctor / WrapDrop / Extract / RefFunctor) are tested via 30 bracket.rs doctests at step 3.3.1 + B19 closure; the smart constructor's body is a mechanical mirror of `RcRun::bracket` plus the existing `make_node_scoped` / `wrap_first_arc` HRTB workaround helpers (which themselves are tested via ArcRun's other smart constructors).

  2. **Step 7 (bracket dispatcher implementation) + step 8 (tests) provide a different test surface that may not require the marker-struct row.** When the dispatcher is implemented at step 7 and tested at step 8, the user calls something like `interpret_with_bracket(...)` rather than constructing a marker-struct row to peel. The test pattern at that point may not require constructing a row that exposes `SendBracketBrand` to the type system in the cycling way.

  3. **The pure-shape integration test would only verify what `cargo check` already verifies.** A T1-style "peels to scoped layer" test for `ArcRun::bracket` would only confirm the smart constructor's signature is type-correct; the structural correctness it would add is duplicative with the cell-level doctests.

- **Why-not-alternatives summary.**
  - **Option B (manual Kind impl on the marker without `impl_kind!`):** Likely doesn't help. The cycle is on `SendBracketBrand`'s own Kind bound, not the macro's expansion; the macro doesn't add bounds beyond what's declared on the marker.
  - **Option C (unsafe `Send` / `Sync` on the marker struct):** Doesn't help. The bound is on the _projection type_ (`<ScopedRow as Kind>::Of<'static, ArcFree<...>>`), not the marker struct itself. Unsafe `Send + Sync` on `ScopedRow` doesn't propagate through `Kind::Of`.
  - **Option D (redesign `SendBracketBrand` to drop the GAT-Send-Sync bound on `Sub`):** Architecturally cleanest but high-cost. Requires either an unsafe `Send + Sync` impl on `ArcFree<F, A>` independent of `F`'s GAT projection (changes `arc_free.rs`) or a new propagation trait. Cascades through the Arc family. Out of scope for step 3.3.4 but **held as the escalation path** at step 8a if step 8's dispatcher tests also fail to exercise `ArcRun::bracket`.
  - **Option E (defer to step 7/8 explicitly):** Functionally identical to Option A; the deferral is implicit in A's "revisit at step 8" framing.

- **Plan-text amendment.** Three concrete steps, in order:
  1. **Step 3.3.4 ships ~18 tests across 5 wrappers** (Run, RcRun, RunExplicit, RcRunExplicit, ArcRunExplicit) instead of ~22 across 6. `tests/run_bracket.rs` includes a tracking comment in its preamble naming this resolutions.md entry as the rationale for the omission and pointing to step 8 + step 8a for end-to-end exercise. Deviation entry at [deviations.md Phase 4 step 3.3.4](deviations.md) records the scope reduction.

  2. **Step 8 (scoped-effect tests, including bracket dispatcher tests) attempts `ArcRun::bracket` end-to-end exercise.** When step 7's dispatcher API lands and step 8 writes its tests, a test that drives a real `ArcRun` program through the dispatcher exercises the smart constructor's runtime path. If the dispatcher's test surface does not require constructing a marker-struct row containing `SendBracketBrand` (because the dispatcher provides its own dispatch entry point), the `ArcRun::bracket` runtime coverage gap closes naturally at step 8.

  3. **Step 8a (conditional on step 8 still being blocked): Option D escalation.** If step 8's dispatcher test surface still requires constructing a user-facing scoped row containing `SendBracketBrand` (via marker struct), and the same rustc overflow recurs, redesign `SendBracketBrand` to drop the GAT-Send-Sync bound on `Sub`. Two implementation paths to investigate at that time:
     - **Path D1: Unsafe `Send + Sync` impl on `ArcFree<F, A>`.** Currently `ArcFree<F, A>: Send + Sync` is auto-derived through the GAT-Send-Sync bound on `F`. Replace with an explicit `unsafe impl<F, A> Send for ArcFree<F, A> where F: WrapDrop, A: Send` (similar for `Sync`). The auto-derive would be removed; the unsafe impl asserts the property is preserved by the substrate's structural invariants. Soundness audit required at the unsafe-impl site.
     - **Path D2: A new substrate trait `SendSyncFreeShape` that propagates `Send + Sync` through `ArcFree<F, A>` without the recursive GAT bound.** Higher-cost than D1; requires a trait redesign.
       The escalation step lands as a separate `feat(effects)` commit at step 8a, scoped only to `SendBracketBrand` and any cascading changes; the rest of the Arc family stays unchanged (the GAT-Send-Sync bound is preserved where it works, only relaxed on the bracket cell where it cycles). Deviation entry at deviations.md, plus a resolutions.md follow-up entry referencing this B20 entry.

  After step 3.3.4, plan-text continues with step 3.3.5 (RefBracket Ref foundational scaffold) per the per-step protocol; B20's escalation (if needed) lands at step 8a, not before.

## Resolved (2026-05-08): Phase 4 step 3.3.1 foundational-scaffold cells hardcode `Free<Sub, _>`; B19 closed via Option C (split into 6 cells per Free family)

**Disposition.** B19 surfaced before step 3.3.3 (Bracket Val smart constructors per wrapper) implementation began, when a probe of `Run::bracket` (stashed at `git stash@{0}`) compiled cleanly for `Run` (substrate = `Free<NodeBrand<R, S>, _>`) but the cell's hardcoded `Free<Sub, _>` revealed a structural mismatch with the other 5 wrappers' substrates (`RcFree` / `ArcFree` / `FreeExplicit` / `RcFreeExplicit` / `ArcFreeExplicit`). Closed on user confirmation in this session via Option C: split the cell into 6 per-Free-family siblings.

### B19. Bracket cell hardcodes `Free<Sub, _>` substrate; only `Run::bracket` is serviceable

- **Issue.** The Bracket Val foundational scaffold (step 3.3.1, commit `1be2af3e`) hardcodes `Free<Sub, _>` in all three sibling cells' field types (`BoxBracket` / `Bracket` / `SendBracket`). This is correct for `Run::bracket` only; the other 5 wrappers each have a distinct substrate, and the user's body closure for any non-Run wrapper returns its wrapper's program type, which doesn't match the cell's hardcoded `Free<Sub, _>`. Cross-family conversion (e.g., `RcFree` to `Free`) doesn't currently exist, would cost O(N) per scoped operation if added, and would re-introduce the Phase 3.5 F4-style runtime-erasure pattern this project deliberately moved away from.

- **Resolution: Option C (split into 6 cells per Free family).** Three reasons:
  1. **Lifetime bound asymmetry between Erased and Explicit Free families is the load-bearing constraint.** Options B (parameterise via 1-arity Kind brand) and B'' (parameterise via 2-arity Kind brand) both need GAT where clause support for Erased family Kind impls (Erased Free's `A: 'static` vs the trait's `A: 'a`). The existing `impl_kind!` macro doesn't use GAT where clauses; extending it is macro/substrate work off the Phase 4 critical path. Option C sidesteps the issue by hardcoding each cell's substrate.

  2. **Codebase precedent.** The library already splits per Free family at the Run-wrapper layer (six Run wrappers, one per `(PointerBrand, FreeFamily)` pair). Splitting Bracket cells along the same axis is structurally consistent.

  3. **Doubled surface is bounded and mechanical, not architectural.** Each new cell is a line-for-line mirror of its existing Erased sibling with `Free` swapped to the appropriate substrate.

- **Why-not-alternatives summary.**
  - **Option B (parameterise via 1-arity Kind brand `F` that bakes Sub):** rejected because the Erased family's `A: 'static` bound vs the trait's `A: 'a` requires GAT where clauses (`type Of<'a, A: 'a>: 'a = Free<F, A> where A: 'static`) that the existing `impl_kind!` macro doesn't support; extending the macro is off the Phase 4 critical path.

  - **Option B'' (parameterise via 2-arity Kind brand):** rejected for the same fundamental reason (the 2-arity Kind helps express the substrate function but doesn't solve the Erased/Explicit bound asymmetry); also adds a 5-param brand which complicates marker-struct doctests and `scoped_effects!` macro generation in step 5.

- **Plan-text amendment.** B19 closure rework ships as a single `feat(effects)` commit on top of `1be2af3e` and `46754fc0`, treating the foundational-scaffold defect as a fix-forward rather than rewriting history. The closure step is named "Phase 4 step 3.3.1 B19 closure: substrate split per Free family" (no per-step subnumbering; sits as a foundational-scaffold-correction commit between the existing step 3.3.2 commit `46754fc0` and the upcoming step 3.3.3 smart-constructor commit). Concrete contents:
  1. **Fix existing cells.** `fp-library/src/types/effects/bracket.rs` `Bracket` (RcBrand sibling) field types switch `Free<Sub, _>` to `RcFree<Sub, _>`; `SendBracket` (ArcBrand sibling) switches to `ArcFree<Sub, _>`; `BoxBracket` keeps `Free<Sub, _>` (already correct for `Run`). Doctests update to reference the correct substrate.

  2. **Add three Explicit-family cell siblings.** `BoxBracketExplicit<'a, P, Sub, A, B>` (stores `FreeExplicit<'a, Sub, _>`); `BracketExplicit<'a, P, Sub, A, B>` (stores `RcFreeExplicit<'a, Sub, _>`); `SendBracketExplicit<'a, P, Sub, A, B>` (stores `ArcFreeExplicit<'a, Sub, _>`). Each parallels its Erased sibling structurally (manual Clone for Rc/Arc-pointer cells, no Clone for Box-pointer cell).

  3. **Add three Explicit-family brand declarations** at [`fp-library/src/brands/effects.rs`](../../../fp-library/src/brands/effects.rs): `BoxBracketExplicitBrand<P, Sub, A, B>` / `BracketExplicitBrand<P, Sub, A, B>` / `SendBracketExplicitBrand<P, Sub, A, B>`.

  4. **Per-brand trait impls.** Each new brand gets four substrate-required impls (Functor identity / SendFunctor identity or stub / WrapDrop None / Extract panic-stub) plus `RefFunctor` per the 3.3.2 pattern (Box panic-stub; Rc `fa.clone()`; Send no impl).

  5. **POC unchanged.** `fp-library/tests/poc_bracket_marker_row.rs` was tested against `BoxBracket` + `Free<NodeBrand<CNilBrand, MarkerRow>, _>`; both are unchanged by B19 closure, so the POC remains valid as-is.

  6. **Deviation entry** at [deviations.md Phase 4 step 3.3.1 B19 closure](deviations.md) documents the substrate split with its rationale (the per-Free-family-cell pattern, doubled surface accepted as bounded mechanical work).

  After B19 closure, step 3.3.3 ships six per-wrapper smart constructors, each using the appropriate Erased or Explicit cell:
  - `Run::bracket` paired with `BoxBracket` (BoxBrand + Free).
  - `RcRun::bracket` paired with `Bracket` (RcBrand + RcFree).
  - `ArcRun::bracket` paired with `SendBracket` (ArcBrand + ArcFree).
  - `RunExplicit::bracket` paired with `BoxBracketExplicit` (BoxBrand + FreeExplicit).
  - `RcRunExplicit::bracket` paired with `BracketExplicit` (RcBrand + RcFreeExplicit).
  - `ArcRunExplicit::bracket` paired with `SendBracketExplicit` (ArcBrand + ArcFreeExplicit).

## Resolved (2026-05-08): Phase 4 step 3.3.3 user-facing recursive type alias rejection on Bracket-containing scoped rows; B18 closed via marker-struct workaround validated by POC

**Disposition.** B18 surfaced before step 3.3.3 (Bracket Val smart constructors per wrapper) implementation began, after the stashed `Run::bracket` probe revealed that user-facing rows containing `BoxBracketBrand<BoxBrand, NodeBrand<R, S>, A, B>` cannot be defined as type aliases (Rust rejects them with `error[E0391]: cycle detected when expanding type alias`). The smart constructor signature itself compiled cleanly; only user-facing rows (and their doctests) tripped the type-alias cyclicity rule. Closed on user confirmation in this session via the conditional adoption path: Option A (marker-struct workaround) validated by a feasibility POC at `fp-library/tests/poc_bracket_marker_row.rs` before committing.

### B18. User-facing recursive type alias rejection on Bracket-containing scoped rows

- **Issue.** Under Option A (the design adopted for B17 after Option C's WF rejection), `BracketBrand<P, Sub, A, B>` carries `Sub` as a brand-level type parameter. A user defining a scoped row that contains `BoxBracketBrand<BoxBrand, NodeBrand<R, S>, A, B>` writes a recursive type alias which Rust rejects. Catch / Local / RefLocal don't have this issue because their brands have no `Sub` parameter; Bracket is the first scoped effect with self-referential brand structure under Option A. The smart constructor's signature itself compiled cleanly via PhantomData zero-sizedness; the issue is purely user-facing.

- **Resolution: Option A (marker-struct workaround) confirmed by POC.** Users define a marker struct (zero-sized, manually implementing `Kind` / `WrapDrop` / `Functor` / `SendFunctor`) that breaks the type-alias cycle by moving the recursion into the trait impl body (Rust accepts recursion in impl bodies; only top-level type aliases reject). The marker's `Kind::Of` projection points to the underlying `CoproductBrand` row body, which references the marker recursively through `NodeBrand<R, Marker>`. The four delegating impls forward to the underlying row's impls.

- **POC validation.** `fp-library/tests/poc_bracket_marker_row.rs` (~225 lines) validates five risks the marker-struct workaround poses:
  - **R1: Does `impl_kind!` accept the recursive type expression?** Pass. The `<UnderlyingRow as Kind>::Of<'a, A>` projection inside the impl body resolves cleanly even though `UnderlyingRow` references `MarkerRow` transitively through `NodeBrand<CNilBrand, MarkerRow>`.
  - **R2: Do `WrapDrop` / `Functor` / `SendFunctor` delegating impls compile?** Pass. The four delegating impls (Kind via `impl_kind!` plus three trait delegations) compile without any HRTB-poisoning or normalization issues.
  - **R3: Does `Run<R, Marker, X>` typecheck?** Pass. `Run`'s implementation never observes whether `S` is a `CoproductBrand` directly or a marker-struct facade; the marker's transparent `Of` projection satisfies all bounds.
  - **R4: At runtime, do `peel` / `resume` thread correctly?** Pass. `prog.peel()` correctly returns `Err(Node::Scoped(Coproduct::Inl(BoxBracket::Bracket { .. })))`. The GAT projection unfolds at compile time so dispatch operations have no runtime indirection through the marker.
  - **R5: Does the `Member` trait check satisfy through the marker's projection?** Pass. Frunk's `CoprodInjector` dispatches against the unfolded coproduct shape (the underlying row's), and the marker's transparent `Kind::Of` makes this seamless.

- **Plan-text amendment.** Step 3.3.3 ships six per-wrapper `bracket` smart constructors (`Run::bracket`, `RcRun::bracket`, `ArcRun::bracket`, `RunExplicit::bracket`, `RcRunExplicit::bracket`, `ArcRunExplicit::bracket`); each smart-constructor doctest uses the marker-struct row pattern explicitly. The stashed Run::bracket probe (`git stash@{0}`) is restored and adapted to use a marker-struct doctest in the foundational 3.3.3 commit; the other 5 wrappers follow. A deviation entry at [deviations.md Phase 4 step 3.3.3](deviations.md) documents the marker-struct workaround, its rationale, and its planned macro-generation in step 5. Step 5's `scoped_effects!` macro design (already deferred) gains a sub-task: generate marker structs + delegating Kind / WrapDrop / Functor / SendFunctor impls for scoped rows containing `Bracket`-family brands.

- **Why-not-alternatives summary.**
  - **Option B (dynamic-typed acquire/release via `Box<dyn Any>`)** rejected: structurally re-introduces the Phase 3.5 F4-style runtime-erasure pattern this project deliberately moved away from; loses static type checking; substrate-type mismatch at dispatch becomes a runtime panic instead of a compile error.
  - **Option C (reopen B17 with deeper `FreeShape` exploration)** rejected: high uncertainty (the WF failure looked terminal, not a corner case); high cost (1-2 days of structural exploration); the closed B17 resolution adopted Option A as fallback for exactly this reason.

## Resolved (2026-05-08): Phase 4 step 3.3.1 Bracket cell's three-differently-typed program returns vs substrate's single-GAT-parameter pattern; B17 closed

**Disposition.** B17 surfaced before step 3.3.1 (Bracket Val foundational scaffold) implementation began. Closed on user confirmation in this session: Option C adopted as primary approach (HKT-trait `FreeShape` decomposition introduces a small ~10-line substrate addition: `pub trait FreeShape { type F; type Inner; }` with blanket impl on `Free<F, A>` and parallel impls on the other Free variants; Bracket struct uses `<X as FreeShape>::F` to derive alternate program types from the substrate's GAT-filled X = body's program type). Option A (5-param struct with explicit substrate brand `Sub`) adopted as explicit fallback if Option C's HRTB-bearing trait surfaces issues during implementation; in that case the recursive type cycle would be empirically verified for Rust acceptance.

### B17. Bracket cell's three-differently-typed program returns vs substrate's single-GAT-parameter pattern

- **Issue.** Catch and Local cells have ONE program type per cell (both action and handler in Catch return Run<R, S, A>; Local's action returns Run<R, S, A>). The substrate's GAT projection `Brand::Of<'a, X>` fills X with the program type, and Catch / Local use that single X consistently across all closure return positions. Bracket cells per [decisions.md table at line 480-481](decisions.md) have THREE differently-typed program returns: `acquire` returns `Run<R, S, A>` (resource value type), `body` returns `Run<R, S, (A, B)>` for Val or `Run<R, S, B>` for Ref (resource + body result), `release` returns `Run<R, S, ()>` (unit). Single-GAT-X can't express three different result types over the same Sub = NodeBrand<R, S>.
- **Resolution: Option C (HKT-trait FreeShape decomposition) as primary; Option A (5-param struct with explicit substrate brand) as explicit fallback.** Introduce a new substrate trait `FreeShape` with blanket impls on the Free family variants:

  ```rust,ignore
  pub trait FreeShape {
      type F;
      type Inner;
  }
  impl<F, A> FreeShape for Free<F, A> { type F = F; type Inner = A; }
  // parallel impls for RcFree, ArcFree, FreeExplicit, RcFreeExplicit, ArcFreeExplicit
  ```

  The Bracket struct uses `<X as FreeShape>::F` to derive alternate program types from the substrate's GAT-filled X (= body's program type, e.g. `Free<NodeBrand<R, S>, (A, B)>` for Val or `Free<NodeBrand<R, S>, B>` for Ref). Acquire's program type becomes `Free<<X as FreeShape>::F, A>`; release's program type becomes `Free<<X as FreeShape>::F, ()>`. Brand stays 3-param (uniform with Catch / Local: `BracketBrand<P, A, B>`). Variant name uniformly `Bracket` per the B12 precedent.

- **Why Option C over the alternatives.**
  - **Option A (5-param struct with explicit Sub):** would have required `Bracket<'a, P, Sub, A, B>` with Sub = NodeBrand<R, S>. Risk: recursive-type-cycle concern (Sub recurses through ScopedRow which contains BracketBrand). PhantomData<Sub> at the brand level might let Rust accept this, but unverified. Held in reserve as explicit fallback if Option C's FreeShape HRTB surfaces structural issues.
  - **Option B (tagged-union BracketStep<A, B> wrapping):** would have leaked internal implementation details into the user-visible result type, and added Run::map wrap overhead per program at smart-constructor time. Rejected.
  - **Option D (type-erased Box<dyn Any> storage):** would have lost type safety and likely broken the Functor / SendFunctor / RefFunctor cascade requirements. Rejected.
- **Plan-text amendment.** Step 3.3.1 (Bracket Val foundational scaffold) commit will land in this order: (i) FreeShape trait at `fp-library/src/types/free.rs` (or wherever Free is defined) with blanket impls on all six Free family variants; (ii) `bracket.rs` foundational scaffold with `BoxBracket` / `Bracket` / `SendBracket` cells using FreeShape to derive acquire's and release's program types; (iii) three brand declarations at `fp-library/src/brands/effects.rs`; (iv) four of five substrate-required trait impls per brand (Functor / SendFunctor / WrapDrop / Extract). Step 3.3.5 (RefBracket Ref foundational scaffold) reuses the FreeShape trait for the two RefBracket sibling cells. If FreeShape's HRTB-bearing trait surfaces issues during implementation (likely surfacing in 3.3.1's Functor::map closure where `<X as FreeShape>::F` needs to thread cleanly through closure storage), fall back to Option A and surface the FreeShape failure as a deviations.md entry.

## Resolved (2026-05-07): Phase 4 step 3.3 sub-step splitting + Bracket acquire field layout cycle reuse + RefBracket sibling-count asymmetry; B14 + B15 + B16 closed

**Disposition.** Three coupled blockers that surfaced before Phase 4 step 3.3 (Bracket / RefBracket scoped-effect constructor) implementation closed on user confirmation in this session: B14 (Bracket `acquire` field layout cycle) adopted Option A (apply B-thunk uniformly mirroring B7/B9 resolutions, bundled into 3.3.1 foundational scaffold commit); B15 (RefBracket has only 2 sibling types per [decisions.md line 484](decisions.md), surfaced for transparency; closed in decisions.md before this session); B16 (sub-step splitting given Val + Ref + asymmetric sibling counts) adopted Option A (8-commit symmetric split mirroring 3.2: 3.3.1-3.3.4 Val cycle, 3.3.5-3.3.8 Ref cycle).

### B14. Bracket `acquire` field layout cycle (B7 / B9 reuse)

- **Issue.** Per [decisions.md table at line 480-481](decisions.md), `Bracket<'a, P, A, B>` and `RefBracket<'a, P, A, B>` both hold `acquire: Run<R, S, A>` as a direct field. The substrate placement is identical to Catch's pre-B7 `action: A` field and Local's pre-B9 `action: A` field: `Free -> NodeBrand::Scoped -> Coproduct -> Bracket -> acquire: Free<...>` -> back to `Free`, completing a layout cycle. Body and release fields don't have this issue because their `Run<R, S, ...>` return values live inside their closures (materialised at call time, not stored as direct fields).
- **Resolution: Option A (apply B-thunk uniformly).** Store `acquire` as `<P>::Of<'a, dyn ClosureTrait(()) -> Run<R, S, A>>` for both Bracket Val and RefBracket Ref siblings. The smart constructor wraps the user's `acquire: Run<R, S, A>` parameter via `<P>::new(move |_: ()| acquire)`. The dispatcher (step 7's `BracketDispatcher`) calls the thunk to materialise the Run program at execution time. Mirrors the Catch (B7) and Local (B9) precedents; requires no new substrate machinery. Bundled into the foundational scaffold commit (3.3.1) per the per-step protocol's small-substrate-fix precedent (Phase 4 step 3.1.3 bundled B7's B-thunk action-representation refactor into the smart-constructor commit).
- **Plan-text amendment.** `fp-library/src/types/effects/bracket.rs` (lands in step 3.3.1) ships with `BoxBracket::acquire: Box<dyn 'a + FnOnce(()) -> Run<R, S, A>>`, `Bracket::acquire: Rc<dyn 'a + Fn(()) -> Run<R, S, A>>`, `SendBracket::acquire: Arc<dyn 'a + Fn(()) -> Run<R, S, A> + Send + Sync>` (all per-pointer-brand B-thunks). Same pattern in `fp-library/src/types/effects/ref_bracket.rs` (lands in step 3.3.5).

### B15. Bracket Val and Ref sibling counts asymmetric

- **Issue.** Per [decisions.md line 484](decisions.md), `RefBracket<'a, P, ...>` requires `P` to be a refcounted brand (`RcBrand` or `ArcBrand`); `BoxBrand` does not satisfy `P::Of<A>: Clone` and is rejected at the type level. Bracket Val ships 3 sibling types and 3 brands; RefBracket Ref ships only 2 sibling types and 2 brands; smart constructors ship on 6 wrappers for Val but only 4 for Ref. Compared to the Local cycle (where both Val and Ref had 3 siblings each / 6 smart constructors each), Bracket's Ref cycle has roughly two-thirds the surface area of its Val cycle. The asymmetry is structurally necessary because RefBracket's body / release closures take `P::Of<A>` (a pointer to A), which requires `Clone` on the projection , BoxBrand fails this.
- **Resolution: Already closed in decisions.md before this session.** Surfaced for transparency and step 3.3 sub-split planning (B16). Implementation plan: Bracket Val foundational scaffold (step 3.3.1) lands `BoxBracket` / `Bracket` / `SendBracket` and `BoxBracketBrand` / `BracketBrand` / `SendBracketBrand`; RefBracket Ref foundational scaffold (step 3.3.5) lands only `RefBracket` / `SendRefBracket` and `RefBracketBrand` / `SendRefBracketBrand`. Smart constructors: 6 `bracket` constructors (one per wrapper) but only 4 `ref_bracket` constructors (`RcRun`, `ArcRun`, `RcRunExplicit`, `ArcRunExplicit`); `Run::ref_bracket` and `RunExplicit::ref_bracket` are NOT defined.
- **Plan-text amendment.** Variant names uniformly `Bracket` across all five sibling enums per the B12 precedent (`BoxBracket::Bracket`, `Bracket::Bracket`, `SendBracket::Bracket`, `RefBracket::Bracket`, `SendRefBracket::Bracket`).

### B16. Step 3.3 sub-step splitting given Val + Ref + asymmetric Ref-sibling counts

- **Issue.** Step 3.3 covers both Bracket (Val flavour) and RefBracket (Ref flavour). The structural shape diverges from 3.2 in two ways: (i) more fields per type (3 fields: acquire / body / release vs Local's 2: modify / action), making foundational scaffold commits larger (more enum field destructuring, more Functor::map composition surface); (ii) asymmetric sibling counts per B15 (Val 3 siblings x 6 wrappers = 18 surface points; Ref 2 siblings x 4 wrappers = 8 surface points), making Ref-cycle commits roughly half the size of Val-cycle commits. The B-thunk on acquire (B14) bundles into the foundational scaffold commit without separate surface.
- **Resolution: Option A (8-commit symmetric split mirroring 3.2).** 3.3.1-3.3.4 Val cycle (foundational scaffold + B14 closure / `RefFunctor` + brand-projection helpers / smart constructors / integration tests); 3.3.5-3.3.8 Ref cycle (same pattern). Each commit is a focused vertical slice. Reasoning: (i) precedent works (3.1's 4-commit split and 3.2's 8-commit split produced focused, reviewable commits); (ii) smaller Ref-cycle commits in Option A are still bounded review surface and clearly typed; (iii) maintaining the 1:1 Val/Ref parallel simplifies cross-referencing during review and future maintenance; (iv) the 8-commit count is mechanically determined by the per-step protocol's structure (foundational scaffold / RefFunctor + helpers / smart constructors / integration tests x 2 flavours = 8); (v) Options B (7-commit) and C (6-commit) trade clarity for marginal commit-count reduction. Option A maintains per-step protocol consistency across Catch (4 commits), Local (8 commits), and Bracket (8 commits) cycles.
- **Plan-text amendment.** Step 3.3 sub-step labels: `3.3 Bracket + RefBracket` becomes `3.3.1 Bracket (Val) foundational scaffold + B14 closure (acquire B-thunk)` / `3.3.2 Bracket (Val) RefFunctor + brand-projection helpers` / `3.3.3 Bracket (Val) smart constructors per wrapper` / `3.3.4 Bracket (Val) integration tests` / `3.3.5 RefBracket (Ref) foundational scaffold` / `3.3.6 RefBracket (Ref) RefFunctor + brand-projection helpers` / `3.3.7 RefBracket (Ref) smart constructors per wrapper (4 wrappers per B15)` / `3.3.8 RefBracket (Ref) integration tests`.

## Resolved (2026-05-07): Phase 4 step 3.2.5 `ToDynFnOnce::ref_new` matrix gap + variant naming; B11 + B12 closed

**Disposition.** Two coupled blockers surfaced before Phase 4 step 3.2.5 (RefLocal Ref foundational scaffold) implementation closed on user confirmation in this session: B11 (`ToDynFnOnce::ref_new` matrix gap blocking `BoxRefLocal::modify` construction) adopted Option A (extend `ToDynFnOnce` with `ref_new` and bundle into 3.2.5 commit); B12 (variant naming for `BoxRefLocal::Local` vs `BoxRefLocal::RefLocal`) adopted Option A (variant is `Local`, mirroring the Val flavour for symmetry).

### B11. `ToDynFnOnce::ref_new` matrix gap blocks `BoxRefLocal::modify` construction

- **Issue.** `BoxRefLocal<'a, P, E, A>`'s `modify` field per [decisions.md table at line 478-479](decisions.md) is `<P>::Of<'a, dyn ClosureTrait(&E) -> E>` (the Ref flavour: closure borrows the env). For `P: ToDynFnOnce`, this elaborates to `Box<'a, dyn 'a + FnOnce(&E) -> E>`. The pointer-abstraction layer's [`ToDynFnOnce`](../../../fp-library/src/classes/to_dyn_fn_once.rs) trait shipped only a by-value `new<A, B>(f: impl FnOnce(A) -> B) -> Self::Of<dyn FnOnce(A) -> B>` and lacked a `ref_new` parallel for `FnOnce(&A) -> B` storage. The other three closure traits ([`ToDynFn`](../../../fp-library/src/classes/to_dyn_fn.rs), [`ToDynCloneFn`](../../../fp-library/src/classes/to_dyn_clone_fn.rs), [`ToDynSendFn`](../../../fp-library/src/classes/to_dyn_send_fn.rs)) all shipped both `new` and `ref_new`; the matrix asymmetry was invisible until step 3.2.5 introduced the first by-reference `FnOnce` storage.
- **Resolution: Option A (extend `ToDynFnOnce` with `ref_new`, bundle into 3.2.5 commit).** Add `fn ref_new<'a, A: 'a, B: 'a>(f: impl 'a + FnOnce(&A) -> B) -> Self::Of<'a, dyn 'a + FnOnce(&A) -> B>` to the trait, plus a `BoxBrand` impl, plus a free-function shim `to_ref_dyn_fn_once<'a, Brand: ToDynFnOnce, A, B>(...)` mirroring the existing `to_dyn_fn_once`. Mechanical translation of the existing `ref_new` pattern from [`to_dyn_fn.rs`](../../../fp-library/src/classes/to_dyn_fn.rs) / [`to_dyn_clone_fn.rs`](../../../fp-library/src/classes/to_dyn_clone_fn.rs) / [`to_dyn_send_fn.rs`](../../../fp-library/src/classes/to_dyn_send_fn.rs); ~30 lines including doctests. Bundling into 3.2.5 follows the precedent set by Phase 4 step 3.1.3's B-thunk action-representation refactor (substrate fixes ride with the step they unblock). Option B (hard-code `Box::new(closure)` in BoxRefLocal) was rejected because it fragments the closure-trait abstraction and creates inconsistency with the existing `BoxLocal` precedent that uses `<BoxBrand as ToDynFnOnce>::new` faithfully. Option C (drop `BoxRefLocal` entirely) was rejected because it breaks the three-sibling pattern and creates Val/Ref availability asymmetry across substrates.
- **Plan-text amendment.** [`fp-library/src/classes/to_dyn_fn_once.rs`](../../../fp-library/src/classes/to_dyn_fn_once.rs) gains the `ref_new` trait method and `to_ref_dyn_fn_once` free function. [`fp-library/src/types/box_ptr.rs`](../../../fp-library/src/types/box_ptr.rs)'s `BoxBrand: ToDynFnOnce` impl gains `ref_new` returning `Box::new(f)`. The matrix in [`pointer-abstraction.md`](../../../fp-library/docs/pointer-abstraction.md) is now complete: every closure trait ships both `new` and `ref_new`.

### B12. `BoxRefLocal` variant naming: `Local` vs `RefLocal`

- **Issue.** `BoxLocal::Local { modify, action }` (Val flavour) names the variant after the operation (`local`). For `BoxRefLocal`, the variant could be `Local` (mirror) or `RefLocal` (explicit). Decisions.md doesn't specify; the choice affects pattern-matching ergonomics in user code and integration tests.
- **Resolution: Option A (variant is `Local`, mirroring Val for symmetry).** Pattern-match: `BoxRefLocal::Local { .. }` / `RefLocal::Local { .. }` / `SendRefLocal::Local { .. }`. The type tag (`BoxRefLocal` vs `BoxLocal`) carries flavour info already; smart-constructor dispatch at the call site (`local(modify, action)` per [decisions.md line 549](decisions.md)) names the operation, not the variant; consistent with `Catch::Catch`'s simple naming. Option B (variant is `RefLocal`) was rejected on repetitive pattern grounds (`RefLocal::RefLocal` reads awkwardly).
- **Plan-text amendment.** All three sibling enums use `Local` as the single variant name: `BoxRefLocal::Local`, `RefLocal::Local`, `SendRefLocal::Local`. Future `Bracket` / `RefBracket` (step 3.3) parallels this decision: `RefBracket::Bracket` rather than `RefBracket::RefBracket`.

## Resolved (2026-05-07): Phase 4 step 3.2 sub-step splitting + Local action layout cycle reuse + file organization; B8 + B9 + B10 closed

**Disposition.** Three coupled blockers that surfaced before Phase 4 step 3.2 (Local + RefLocal scoped-effect constructors) implementation closed on user confirmation in this session: B8 (sub-step splitting given Val + Ref doubling step 3.1's surface) adopted Option B (8-commit separate Val and Ref cycles); B9 (Local action-field layout cycle reuse) adopted Option A (apply B-thunk uniformly mirroring catch.rs); B10 (file organization for Local vs RefLocal) adopted Option A (separate `local.rs` and `ref_local.rs` files). Step 3.2.1 (Local Val foundational scaffold) ships in this commit; sub-steps 3.2.2-3.2.4 follow the 3.1.x pattern. The Ref cycle (3.2.5-3.2.8) lands after the Val cycle closes.

### B8. Step 3.2 sub-step splitting given Val + Ref doubling step 3.1's surface

- **Issue.** Step 3.2 covers both `Local` (Val flavour) and `RefLocal` (Ref flavour) per [decisions.md section 4.5](decisions.md). Each flavour requires the same 3-sibling-type-per-pointer-brand pattern as Catch (BoxLocal / Local / SendLocal for Val plus BoxRefLocal / RefLocal / SendRefLocal for Ref), giving 6 enum types and 6 brand declarations against Catch's 3 of each. Smart-constructor surface doubles too (12 per-wrapper constructors total: 6 Run wrappers x 2 flavours), and Functor / SendFunctor / WrapDrop / Extract / RefFunctor must all be implemented per brand. The B6 closure committed step 3.1 to a 4-commit split; step 3.2's roughly-2x surface needs an analogous decision.
- **Resolution: Option B (8-commit separate Val and Ref cycles).** 3.2.1-3.2.4 ship the Local (Val) full cycle (foundational scaffold / `RefFunctor` + helpers / smart constructors / integration tests); 3.2.5-3.2.8 ship the RefLocal (Ref) full cycle in the same shape. Each commit stays a focused vertical slice at the size of 3.1's commits, easier to review and debug; Val/Ref differences (closure parameter type `E` vs `&E`, derived-sub-scope ergonomics, `E: Clone` requirement) surface cleanly per phase; if R2 risk on the Arc family surfaces during the Val cycle it can be triaged before Ref expansion. Option A (4-commit paired Val+Ref split) was rejected because each commit would be roughly 2x the size of the equivalent 3.1 commit (3.2.3 alone ships 12 smart constructors plus a multi-flavour B-thunk refactor); review surface gets dense. Option C (5-commit hybrid) was rejected because the combined-then-split rhythm is awkward and the foundational scaffold commit hits the same per-commit-size concern as Option A.
- **Plan-text amendment.** Step 3.2 sub-step labels: `3.2 Local + RefLocal` becomes `3.2.1 Local (Val) foundational scaffold` / `3.2.2 Local (Val) RefFunctor + brand-projection helpers` / `3.2.3 Local (Val) smart constructors per wrapper` / `3.2.4 Local (Val) integration tests` / `3.2.5 RefLocal (Ref) foundational scaffold` / `3.2.6 RefLocal (Ref) RefFunctor + brand-projection helpers` / `3.2.7 RefLocal (Ref) smart constructors per wrapper` / `3.2.8 RefLocal (Ref) integration tests`. Mirror split available for 3.3 (Bracket / RefBracket) as design risk surfaces, or simpler if it doesn't.

### B9. Local action-field layout cycle (B7 reuse)

- **Issue.** Per [decisions.md table at line 478-479](decisions.md), `Local<'a, P, E, A>` and `RefLocal<'a, P, E, A>` both hold `action: Run<R, S, A>`. The action's substrate placement is identical to ``Catch`'s`: `Free -> NodeBrand::Scoped -> Coproduct -> Local -> action: Free<...>`-> back to`Free`, completing a layout cycle. The B7 resolution adopted B-thunk for Catch; the question is whether to apply the same uniformly to Local / RefLocal.
- **Resolution: Option A (apply B-thunk uniformly).** Store `action` as `<P>::Of<'a, dyn 'a + FnOnce/Fn(()) -> A>` per the per-pointer-brand pattern (mirroring catch.rs). The layout-cycle reasoning is structurally identical to Catch's case; Free's variant payload contains a recursive Free without pointer indirection regardless of which scoped-effect cell holds the `action`. Applying B-thunk uniformly across scoped effects keeps the user-facing API consistent (`local(modify, action)` mirrors `catch(action, handler)` in the constructor's argument shape and dispatch behaviour) and reuses the existing pointer-abstraction `ToDyn*Fn::new` family without new machinery. Option B (test layout first) was rejected because the layout-cycle reasoning does not depend on which scoped-effect cell holds the action; the prototype was essentially guaranteed to fail and would just delay the inevitable. Option C (alternative layout-cycle breaks) was rejected for the same reasons recorded in the [B7 options analysis](#resolved-2026-05-07-phase-4-step-3.1.3-catch-action-field-layout-cycle-b7-closed): uniform `Box<A>` and per-pointer-brand pointer for raw `A` both impose `A: Clone` constraints that break Functor::map composition.
- **Plan-text amendment.** `local.rs` ships with `BoxLocal::action: Box<dyn 'a + FnOnce(()) -> A>`, `Local::action: Rc<dyn 'a + Fn(()) -> A>`, `SendLocal::action: Arc<dyn 'a + Fn(()) -> A + Send + Sync>` from step 3.2.1 forward. Manual `Clone` impls for `Local` (Rc-bump on both modify and action pointers) and `SendLocal` (Arc-bump); `BoxLocal` does NOT impl `Clone` (Box<dyn FnOnce> is structurally uncloneable). The B-thunk pattern is documented in the module-level docstring with a cross-reference to `catch.rs`.

### B10. File organization for Local and RefLocal

- **Issue.** `catch.rs` is already 1349 lines for a single scoped effect (one Val flavour with three pointer-brand siblings). Local + RefLocal in a combined `local.rs` would scale linearly: ~2700 lines. Plan and decisions documents are silent on file organization; existing precedent (state.rs / reader.rs / choose.rs / except.rs / writer.rs) puts each effect in its own file but those are first-order effects with simpler structure.
- **Resolution: Option A (separate `local.rs` and `ref_local.rs`).** Local (Val) in `fp-library/src/types/effects/local.rs` (~1000-1500 lines projected); RefLocal (Ref) in `fp-library/src/types/effects/ref_local.rs` (~1000-1500 lines projected; lands in step 3.2.5). File-size precedent (catch.rs at 1349 lines for one effect) is the load-bearing argument: combined would exceed reviewable bounds and complicate tool-assisted navigation. The Val/Ref distinction also fits naturally on the file axis: future scoped effects (Bracket / RefBracket in step 3.3) follow the same Val/Ref pattern, and the precedent set by 3.2 propagates to 3.3. Option B (combined `local.rs`) was rejected on file-size grounds.
- **Plan-text amendment.** [`fp-library/src/types/effects.rs`](../../../fp-library/src/types/effects.rs)'s module list gains `pub mod local;` (3.2.1) and will gain `pub mod ref_local;` (3.2.5). Brand declarations land in [`fp-library/src/brands/effects.rs`](../../../fp-library/src/brands/effects.rs) for both files following alphabetic insertion-point convention.

## Resolved (2026-05-07): Phase 4 step 3.1.3 Catch action-field layout cycle (B7) closed

**Disposition.** B7 surfaced during Phase 4 step 3.1.3's `Run::catch` smart constructor implementation: the doctest tripped a Rust layout-cycle error tracing through `Free<NodeBrand<R, S>, TypeErasedValue>` -> `<S>::Of<'_, Free<...>>` -> `BoxCatch<..., Free<...>>` -> unboxed `action: Free<...>` field -> `Free<...>` layout (cycle). Adopted **B-thunk with unit-arg `Fn(()) -> A` form** on user confirmation in this session.

### B7. `Catch` action-field layout cycle when embedded in substrate

- **Issue.** `BoxCatch` / `Catch` / `SendCatch` shipped in step 3.1.1 with `action: A` (unboxed) per a literal reading of [decisions.md section 4.5](decisions.md)'s "carry the action and handler payload" sketch. The layout-cycle implication only surfaces at substrate-embedding time (3.1.3 smart constructors), not when the effect type is exercised in isolation (3.1.1 / 3.1.2 doctests). The cycle is broken by any pointer-sized indirection on the `action` field; the question is which.
- **Resolution: B-thunk (per-pointer-brand pointer of unit-arg `Fn(()) -> A` thunk).** `BoxCatch` action becomes `<P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> A>` (single-shot via `Box<dyn FnOnce>`); `Catch` action becomes `<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>` (multi-shot via `Rc<dyn Fn>`); `SendCatch` action becomes `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>` (thread-safe via `Arc<dyn Fn + Send + Sync>`). The unit-arg `Fn(()) -> A` form (vs zero-arg `Fn() -> A`) lets the existing pointer-abstraction [`ToDynFnOnce::new`](../../../fp-library/src/classes/to_dyn_fn_once.rs) / [`ToDynCloneFn::new`](../../../fp-library/src/classes/to_dyn_clone_fn.rs) / [`ToDynSendFn::new`](../../../fp-library/src/classes/to_dyn_send_fn.rs) family construct both action and handler fields uniformly. The matrix in [`pointer-abstraction.md`](../../../fp-library/docs/pointer-abstraction.md)'s `(pointer-capability, closure-semantic)` table now has the action field's pointer-and-closure-trait combination matching the handler field's existing pointer-and-closure-trait combination per Catch sibling, parallel to the user-API-facing per-pointer-brand pattern. Call sites invoke as `action(())` and construct as `<P as ToDyn*Fn>::new(move |_: ()| value)`. Functor / SendFunctor / RefFunctor impls compose thunks lazily (no `A: Clone` constraint) by wrapping `f` in an `Rc<dyn Fn(A) -> B>` (or `Arc<dyn Fn(A) -> B + Send + Sync>` for SendCatch) shared between the new action and new handler closures so each can call `f` once via Rc/Arc-deref. The catch dispatcher (step 7) calls the thunk to materialise the action program at execution time.
- **Options considered and rejected.** _Option A (uniform `Box<A>`):_ pointer-sized layout breaks the cycle but pays a real perf cost on multi-shot wrappers (deep-clone vs refcount-bump). _Option B-literal (per-pointer-brand pointer for raw `A`: `Rc<A>` / `Arc<A>`):_ faithful per-pointer-brand pattern but `Functor::map` extraction `A` from `Rc<A>` requires `A: Clone` which can't be added to the trait method's signature; multi-shot path's `to_view` clones produce shared Rc layers at every map call site, so try-unwrap-or-panic would fire in `Choose` + `Catch` programs. _Option C (`BoxedAction<A>` wrapper):_ cosmetic-only; adds a public type for a fix that doesn't need one. _Option D (defer + redesign):_ disruptive given 3.1.1 / 3.1.2 already shipped.
- **Plan-text amendment.** `catch.rs`'s three Catch sibling enums updated per the B-thunk spec; impl bodies (Functor, SendFunctor, RefFunctor, WrapDrop, Extract, brand-projection helpers, doctests) revised to match. Manual `Clone` impls added for `Catch<'a, P, E, A>` and `SendCatch<'a, P, E, A>` (refcount-bump on action and handler pointers; no `A: Clone` requirement). `BoxCatch` does NOT impl `Clone` (Box<dyn FnOnce> is structurally uncloneable). The brand-projection helper `box_catch_action_ref` from step 3.1.2 was removed (the BoxCatch RefFunctor impl is now stub-everywhere because Box<dyn FnOnce> can't be invoked through a reference); `catch_action_ref` was renamed to `catch_action_thunk_ref` and returns `&Rc<dyn Fn(()) -> A>` (the thunk reference) for the CatchBrand RefFunctor impl's thunk-composition path. ArcRun gains a `make_node_scoped` HRTB-poisoning workaround helper paralleling the existing `make_node_first` (the existing `wrap_first_arc` is reusable for both First and Scoped paths since its body just forwards to `ArcFree::wrap`).

## Resolved (2026-05-07): Phase 4 step 3.1 sub-step splitting; B5 + B6 closed

**Disposition.** Two coupled blockers that surfaced during Phase 4 step 3.1 foundational implementation closed on user confirmation in this session: B5 (`RefFunctor` GAT-normalization on scoped-effect closure-cell brands) adopted Option A (brand-projection helpers); B6 (step 3.1 sub-step splitting) adopted Option B (4-commit split). Step 3.1.1 shipped at `abd3d1a3`; step 3.1.2 lands the `RefFunctor` work in this commit. Steps 3.1.3 (smart constructors) and 3.1.4 (integration tests) follow.

### B5. `RefFunctor` GAT-normalization for scoped-effect closure-cell brands

- **Issue.** The trait method [`RefFunctor::ref_map`](../../../fp-library/src/classes/ref_functor.rs) takes `fa: &<Self as Kind>::Of<'a, A>`. Inside an impl scope, the compiler refuses to unify that reference with the concrete enum type (e.g., `&BoxCatch<'a, BoxBrand, E, A>`) because GAT projections do not normalize through reference parameters in trait method bodies, even with explicit `let fa: &BoxCatch<...> = fa;` annotation. Pattern matching on the variants therefore fails to type-check. [`Identity`](../../../fp-library/src/types/identity.rs) sidesteps this via tuple-struct field access (`fa.0`), which doesn't require a match; sum types like `Catch` have no equivalent workaround in stable Rust.
- **Resolution: Option A (brand-projection helper).** Land `#[doc(hidden)]` free functions outside the impl scope whose where-clauses carry only `Kind` bounds; the function body's pattern match normalizes cleanly because no HRTB context is in scope. Mirrors `arc_run::unwrap_first`'s precedent. Three helpers ship in 3.1.2 (`box_catch_action_ref`, `catch_action_ref`, `catch_handler_ref`); steps 3.2-3.4 will land additional helpers per scoped-effect family. Option B (`unsafe { core::mem::transmute(fa) }`) was rejected: technically equivalent but introduces `unsafe` audit burden for what is structurally safe under [`impl_kind!`](../../../fp-macros/src/hkt/impl_kind.rs)'s expansion guarantees. Option C (substrate redesign dropping `RefFunctor` from the scoped-row trait surface) was rejected: reduces by-reference traversal capability and forces a non-local change to `scoped.rs`'s "all five required" claim. Option D (skip `RefFunctor` impls, accept compile-fail) was rejected: by-reference traversal is more pervasive than the SendCatchBrand-no-Functor case, so user programs would compile-fail with cryptic trait-bound messages. Step 3.1.2 also discovered that `SendCatchBrand` does not need `RefFunctor` because the cascade through [`ArcRunExplicitBrand`](../../../fp-library/src/brands/effects.rs) does not require it (`ArcFreeExplicitBrand: !RefFunctor` per the brand's docs); a hypothetical impl would face the same `Send + Sync` bound mismatch on `func` that prevents `Functor`. Logged at [deviations.md Phase 4 step 3.1.2](deviations.md), mirroring the [`SendCatchBrand`-no-`Functor` precedent](deviations.md). The `BoxCatch` impl uses an `unreachable!`-stub recovery handler (suppressed via `#[expect(clippy::unreachable, reason = "...")]`) because `Box<dyn FnOnce>` cannot be replicated through a reference; the path is structurally unreachable in real programs since `RunExplicitBrand: RefFunctor` is reachable only through synthetic non-Coyoneda rows. The `Catch` impl is faithful (Rc-clone the handler, share `func` via `<RcBrand as ToDynCloneFn>::ref_new`).
- **Plan-text amendment.** Phase 4 step 3.1 gains the per-sub-step labels per B6's Option B; the `RefFunctor` deferral language was removed from the Phase status / Next greenfield work paragraphs once 3.1.2 shipped.

### B6. Step 3.1 sub-step splitting given the B5 deferral

- **Issue.** The original step 3.1 plan committed to one bundled commit (per the 2026-05-06 step-3 splitting resolution) covering the Catch sibling types + brands + 5 trait impls + smart constructors + tests. The B5 deferral surfaced internal sub-step boundaries not visible at planning time: the foundational scaffold (4 of 5 trait impls) is mechanical from Phase 3.5's State template, while the `RefFunctor` work requires new helper-machinery design (per B5 Option A); smart constructors and tests are separate concerns again.
- **Resolution: Option B (4-commit split).** Step 3.1 splits into 3.1.1 (foundational scaffold; shipped at `abd3d1a3`), 3.1.2 (`RefFunctor` impls + brand-projection helpers; this commit), 3.1.3 (smart constructors per wrapper), 3.1.4 (integration tests). Mirrors step 2's empirically-validated splitting clause (step 2 split into 2.1-2.6 when per-wrapper risk surfaced). Option A (single bundled commit) was rejected for context-budget cost and review surface size. Option C (2-commit split with smart constructors and tests bundled into 3.1.1 using only 4 of 5 traits) was rejected because programs using `NodeBrand<R, S>: RefFunctor` with `S` containing Catch brands would fail to compile until the deferred `RefFunctor` follow-up landed; 3.1.1 would ship in a temporarily-incomplete state.
- **Plan-text amendment.** Step 3 sub-step labels updated: `3.1 Catch` becomes `3.1.1 Catch foundational scaffold` / `3.1.2 Catch RefFunctor + brand-projection helpers` / `3.1.3 Catch smart constructors` / `3.1.4 Catch integration tests`. Mirror split available for 3.2 (Local/RefLocal), 3.3 (Bracket/RefBracket), 3.4 (Span) as design risk surfaces, or simpler if it doesn't.

## Resolved (2026-05-06): Phase 4 implementation-kickoff sequencing K1 and K2 (POC 3 standalone commit first; plan.md numbering authoritative for commit boundaries)

**Disposition.** Two implementation-kickoff sequencing decisions surfaced at the Phase 3.5-to-Phase-4 transition (commit `2e97e812`); both adopted Option A on user confirmation in this session. Closes the previously-active `Phase 4 implementation-kickoff sequencing` subsection in plan.md and unblocks substantive Phase 4 implementation work. Q4, R1, R2, R3 mitigations remain inline in plan.md's [Phase 4 implementation prototypes and risk mitigations](plan.md#phase-4-implementation-prototypes-and-risk-mitigations) subsection, scheduled to land during R1 implementation kickoff (Q4 / R1 / R2 prototypes) or alongside the standard scoped-effect rollout (R3 benchmark commit).

### K1. POC 3 (`interpret_with_either`) validation ordering

- **Issue.** Plan.md commits to a new substrate primitive `interpret_with_either<EBrand, Idx>(self, fo_handlers: &impl DispatchHandlers<...>) -> Either<A, EBrand::Op>` on each Run wrapper, used by the `Catch` cons-cell impl in [Phase 4 step 4](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row). The primitive's POC validation on `RcRun` (POC 3) "must land before the step that introduces `interpret_with_either` ships generically across all six Run wrappers", but the literal commit ordering relative to other Phase 4 substrate work (steps 1, 2, the `Span` cons-cell) was unspecified.
- **Resolution: Option A.** POC 3 lands as a standalone commit at [`fp-library/tests/poc_rc_run_handle_with_either.rs`](../../../fp-library/tests/) before any other Phase 4 substrate work. Mirrors POC 1 ([`poc_send_catch_brand.rs`](../../../fp-library/tests/poc_send_catch_brand.rs)) and POC 2 (`poc_rc_run_interpose.rs`) precedent (each shipped as a standalone validation commit before its generic rollout). The half-day cost is amortised across Phase 4's 1-2-week budget for Sequencing Plan item 3. Option B (mixed-layer paired commit with the Catch cons-cell substrate primitive) was rejected because if POC 3 surfaces a wall, the entire `Catch` cons-cell design is blocked mid-Phase 4 and earlier non-trivial commits (Span cons-cell, dispatcher trait skeleton) would stand against a now-broken design. Option C (skip POC 3, inline rollout) was rejected because it removes the validation step entirely; the [R1 risk](plan.md#r1-explicit-family-interpose-generalisation) of HRTB-poisoning on the Explicit family makes this riskier than the half-day POC investment.
- **Plan-text amendment.** Phase 4 gains a new step 0 before step 1: "POC 3 validation: `interpret_with_either<EBrand, Idx>` substrate primitive on `RcRun` at [`fp-library/tests/poc_rc_run_handle_with_either.rs`](../../../fp-library/tests/), paralleling POC 1 / POC 2. Mechanical from ``interpret_with`'s body` with one branch substitution. Generic rollout across all six Run wrappers ships in step 2a after POC 3 validates."

### K2. Plan.md step numbering vs Sequencing Plan item numbering

- **Issue.** Phase 4 work was described twice with different boundaries. Plan.md numbered the steps 0, 1, 2, 2a, 3, 4, 5, 6, 7, 8 (after K1 adoption); the [Sequencing Plan in the remediation report](review/1_scoped_effects_design/remediation_proposals_phase_4.md#sequencing-plan) numbers items 1-9. The two schemes overlap differently: Sequencing items 1, 2, 4 are pre-implementation doc commits already shipped; item 3 spans plan.md steps 1, 2, 4 (the `Catch`/`Span` cons-cell slice); items 5-9 map to plan.md steps 3-8 with similar non-bijective mapping. The prompt's per-step protocol cited "one step per commit" without specifying which numbering.
- **Resolution: Option A.** Plan.md step numbering is the authoritative commit boundary. Each plan.md step (0, 1, 2, 2a, 3, 4, 5, 6, 7, 8) becomes one commit (or a small bundled-commit group with surfaced split per the per-step protocol's "splitting an oversized step" clause). The Sequencing Plan items group these commits but do not replace them. Mirrors Phase 3 step 5a precedent (sub-step splits within a single plan.md step). Option B (Sequencing Plan numbering authoritative) was rejected because Sequencing items 3, 5, 7 are 1-2-week each per the report's estimate and bundled commits clash with the per-step protocol; Option C (hybrid) was rejected because two parallel numbering schemes confuse future implementors.
- **Plan-text amendment.** No structural revision (semantic-only decision). Sequencing Plan numbering remains in the remediation report as planning context; plan.md is the authoritative source for commit boundaries. Implementer protocol: `git log` shows commits per plan.md step; for time-budget questions the Sequencing Plan in the remediation report is the right reference.

## Resolved (2026-05-06): Phase 3 prior-review F4 closed structurally via Phase 3.5 retrofit (sibling `Box*Brand` family on default Run substrates)

**Question.** The Phase 3 prior-review's
[F4 finding](review/0_first_order_effects_implementation/remediation_proposals.md#f4-the-single-shot-vs-multi-shot-property-promised-per-wrapper-is-not-enforced-at-the-effect-instance-level)
flagged that the "single-shot vs multi-shot" property advertised
per Run wrapper applied only to the Free-spine consumption, not to
per-effect closure cells. Concretely: on
`Run` and
`RunExplicit`
(both single-shot per the wrapper-level guarantee), the smart
constructors `get` / `put` / `ask` stored continuations as
[`StateBrand<RcBrand, A>`](../../../fp-library/src/brands/effects.rs)
/
[`ReaderBrand<RcBrand, A>`](../../../fp-library/src/brands/effects.rs)
whose closure cell is `Rc<dyn Fn(...) -> A>`. The cell was thus
multi-shot-callable at the effect-instance level even though the
surrounding program was single-shot, creating a semantic asymmetry:
a handler could call the State continuation many times even on a
wrapper advertised as single-shot. The review's F4 framing called
this "API claim does not match reality at the effect-instance
level".

**Original Phase 3 disposition (step 8, 2026-05-05):** Phase 3
step 8 adopted F4's recommended **Option A** (documentation-only):
weaken the [Success criteria](plan.md#success-criteria)'s
"single-shot vs multi-shot" claim to apply to Free spine
consumption only, and document at
``StateBrand`'s rustdoc`
that per-effect closures carry their multi-shot property at the
effect-instance level on every wrapper. Phase 3 closed under this
option.

**Phase 3.5 re-opening rationale.** Phase 4 design-question B3 (the
[Phase 4 pre-implementation design questions](#resolved-2026-05-05-phase-4-pre-implementation-design-questions-b1-b4-q1-q3-q5-closed-by-design-adoption-commit-6e960701))
re-examined the F4 framing in the context of user-supplied scoped
handlers. The key observation: Phase 3's State / Reader / Choose
continuations are _substrate-constructed_ (the smart constructor
emits a trivial `|s| s` or `|()| ()` closure), so the mismatch
surfaces only inside the substrate code at handler dispatch.
Phase 4's scoped-effect handlers (the body of `Catch::handler`,
`Local::modify`, `Bracket::body`, etc.) are _user-supplied_, so the
FnOnce-ergonomics friction would land on user code: a natural
recovery handler is `move |e: MyErr| recovery_built_from_captures`
where the captures are consumed in building the recovery program.
That is `FnOnce`, not `Fn`. Forcing it through `Rc<dyn Fn>` would
reject the natural pattern at the type level on default `Run`,
requiring users to wrap captures in `Rc` or constrain captures to
`Clone`. Phase 4 needs the FnOnce ergonomics; landing the same
pattern across Phase 3 + Phase 4 keeps the user-facing
pointer-brand surface uniform.

**Resolution.** Phase 3.5 (pointer-brand-pattern retrofit) closes
F4 structurally rather than as accepted-tradeoff. The retrofit
replaces, on default `Run` / `RunExplicit` only, the
[`StateBrand<RcBrand, S>`](../../../fp-library/src/brands/effects.rs)
/
[`ReaderBrand<RcBrand, E>`](../../../fp-library/src/brands/effects.rs)
threading with parallel sibling brands
[`BoxStateBrand<BoxBrand, S>`](../../../fp-library/src/brands/effects.rs)
/
[`BoxReaderBrand<BoxBrand, E>`](../../../fp-library/src/brands/effects.rs)
whose closure cells are `Box<dyn FnOnce(...) -> A>` via the new
[`ToDynFnOnce`](../../../fp-library/src/classes/to_dyn_fn_once.rs)
trait (sub-step 1). The new sibling effect types
`BoxState` /
`BoxReader` /
`BoxChoose`
mirror the structure of
`State` /
`Reader` /
`Choose` and
`SendState` /
`SendReader` /
`SendChoose` but
with `where P: ToDynFnOnce` bounds restricting instantiation to
`BoxBrand` (the only brand for which `<P as Pointer>::Of<dyn FnOnce>`
is operationally implementable). Multi-shot wrappers
(`RcRun` /
`RcRunExplicit`
/ `ArcRun` /
`ArcRunExplicit`)
keep their existing `Rc<dyn Fn>` / `Arc<dyn Fn + Send + Sync>`
paths unchanged, because their wrapper-level multi-shot guarantee
makes the multi-shot continuation cell semantically aligned.

After the retrofit, the single-shot-vs-multi-shot property is
enforced at the type level for both the Free spine _and_ the
per-effect closure cell on default Run substrates. A handler on
`Run::interpret` matches `BoxState::Get(k) => k(state)`; the
`k(state)` consumes the box on the single call, statically proving
the continuation is single-shot.

**Trade-off discussion (relative to the F4-recommended Option A).**
The retrofit does what Option A explicitly avoided: it re-opens the
[(3.a-1) "one effect type per operation"
sub-decision](#L209-L240) (locked 2026-05-03). The trade-off is
acceptable because:

1. **Doubling cost is amortised over Phase 4.** Option B's
   "doubles the per-effect type definitions" critique applied when
   only Phase 3 was in scope; with Phase 4 also needing the
   `BoxBrand` path for user-supplied scoped handlers (per Phase 4
   design-question B3), the cost is amortised across both phases.
   Phase 3.5 lands three sibling types
   (`BoxState` /
   `BoxReader` /
   `BoxChoose`)
   in one commit; Phase 4 reuses the same pattern for
   [`Catch`](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
   / `Local` / `Bracket` / `Span` / `RefLocal` / `RefBracket`
   without adding a parallel brand-family per scoped effect.

2. **Brand surface stays alphabetised, not duplicated.** Each
   `Box*Brand` lives in
   [`fp-library/src/brands/effects.rs`](../../../fp-library/src/brands/effects.rs)
   alongside its `*Brand` and `Send*Brand` siblings; the
   brand-implementations table at
   [`fp-library/docs/pointer-abstraction.md`](../../../fp-library/docs/pointer-abstraction.md)'s
   newly-added
   `(closure-semantic, pointer-capability)` matrix shows that the
   trait family is structurally complete (each pair maps to
   exactly the brands that legitimately implement it).

3. **Doc surface stays single-source.** The retrofit re-uses
   existing rustdoc on
   [`StateBrand`](../../../fp-library/src/brands/effects.rs)'s
   "Coyoneda-fusion at call-site" subsection (which applies
   equally to `BoxStateBrand`); module-level `state.rs` /
   `reader.rs` / `choose.rs` docs gain one new paragraph each
   describing the sibling. The combined incremental doc surface is
   ~30 lines, not the "doubles the documentation surface" Option
   B would have implied at Phase 3 step 5a's smaller scope.

4. **Macro complexity is unaffected.** Phase 3 step 6's
   [`define_effect!` macro](#resolved-2026-05-04-phase-3-step-6-define_effect-macro-deferred-until-phase-4-ships-or-user-demand-surfaces-design-research-preserved-for-later-revisit)
   is deferred indefinitely; the per-wrapper smart constructors
   are hand-written. Adding the `BoxBrand` path to the hand-written
   smart constructors is a per-method update on `Run` /
   `RunExplicit` only (six methods total: `get` / `put` / `ask`
   across two wrappers), not a macro extension. If `define_effect!` ever
   ships, the per-pointer-brand split becomes a macro parameter.

The 2026-05-03 (3.a-1) "one effect type per operation" decision
remains valid for the _user-facing_ effect family naming
convention: users see `BoxStateBrand` / `StateBrand` /
`SendStateBrand` as one State family with three pointer-substrate
variants, and the `Run`-family smart constructors thread the
appropriate variant per wrapper without users specifying the
pointer brand explicitly. The "one type per operation" promise
holds at the user-API level (one `get` on `Run`, one `get` on
`RcRun`, one `get` on `ArcRun`); the under-the-hood representation
splits into three siblings to satisfy the structural closure-trait
constraint.

**Why structural is now preferable to documentation-only.** Phase 3
step 8's Option A wrote: "the API claim does not match reality at
the effect-instance level; document the mismatch." After Phase 3.5,
the API claim _does_ match reality: `Run`'s `BoxState::Get(k)` /
`BoxState::Put(s, k)` / `BoxReader::Ask(k)` continuations are
exactly single-shot at the type level. Future-you reading
`Run::get`'s smart constructor signature does not need to consult
the rustdoc for "by the way, the inner closure is multi-shot but
the program is single-shot". The structural property is visible at
the type signature.

**Plan integration.** Phase 3.5 is sequenced between Phase 3 close
and Phase 4 implementation. Sub-steps:

1. `ToDynFnOnce` trait + `BoxBrand` impl (commit `b067f912`,
   2026-05-05).
2. Sibling effect types and brands; smart constructor updates on
   `Run` / `RunExplicit`; integration test updates (commit
   `a762fa27`, 2026-05-05).
3. `pointer-abstraction.md` documentation update (commit
   `89546709`, 2026-05-06).
4. Implicit (covered by sub-step 2's `just verify` clean run).
5. This resolutions.md entry (this commit, 2026-05-06).

Phase 4 implementation then follows, reusing the same
`BoxBrand` + `ToDynFnOnce` pattern for user-supplied scoped-effect
handlers per
[Phase 4 design-question B3's recommendation](#b3-pointer-brand-parameterisation-boxbrand--todynfnonce-on-default-run).

**Cross-references:**

- Phase 3 step 8 disposition (Option A doc-only, 2026-05-05):
  [deviations.md Phase 3 step 8](deviations.md).
- F4 finding's full restatement, root cause, options A/B/C
  evaluation:
  [`review/0_first_order_effects_implementation/remediation_proposals.md`](review/0_first_order_effects_implementation/remediation_proposals.md).
- Phase 4 design-question B3 (the original re-opening rationale):
  [plan.md Phase 4 pre-implementation design questions](#resolved-2026-05-05-phase-4-pre-implementation-design-questions-b1-b4-q1-q3-q5-closed-by-design-adoption-commit-6e960701).
- Three-sibling-types interpretation rationale (why
  `BoxStateBrand` rather than parameterising `StateBrand` over
  `BoxBrand`): [deviations.md Phase 3.5 sub-step 2](deviations.md).
- Trait-family completeness matrix:
  [`fp-library/docs/pointer-abstraction.md`](../../../fp-library/docs/pointer-abstraction.md)'s
  `ToDynFnOnce is BoxBrand-only by structural necessity`
  subsection.

## Resolved (2026-05-05): Phase 4 pre-implementation design questions B1-B4, Q1-Q3, Q5 closed by design-adoption commit `6e960701`

**Disposition.** Eight Phase 4 pre-implementation design questions had their plan-revision-required edits adopted into [plan.md](plan.md) and [decisions.md](decisions.md) by the 2026-05-05 design-adoption commit `6e960701` (B1, B2, B4, Q1, Q2, Q3, Q5) plus subsequent Phase 3.5 retrofit landings (B3 implementation: commits `b067f912`, `a762fa27`, `89546709`, `4471629d`). Each item's original framing (issue, options, recommendation, reasoning) lived in plan.md's `Phase 4 pre-implementation design questions` subsection prior to this 2026-05-06 cleanup; the pre-cleanup framing is recoverable via `git show 2e97e812:docs/plans/effects/plan.md` (last commit retaining the long-form entries). Compact summary below; outstanding follow-ups (POC 3 prerequisite for B4; Q4 / R1 / R2 / R3 prototype-and-risk items pending at R1 implementation kickoff) live in plan.md's [Phase 4 implementation prototypes and risk mitigations](plan.md#phase-4-implementation-prototypes-and-risk-mitigations) and [Phase 4 implementation-kickoff sequencing](plan.md#phase-4-implementation-kickoff-sequencing) subsections.

### B1. `Catch::action` type-parameter interpretation

- **Issue.** Plan and [decisions.md](decisions.md) describe `Catch<'a, P, E, A>` as storing `action: Run<R, S, A>`, but `CatchBrand<P, E>` does not carry `R` or `S` as parameters; the `Run<R, S, A>` rendering was therefore loose notation rather than a literal Rust type. Implementer needed to know whether `action` is genuinely `Run<R, S, A>` (forcing `CatchBrand` to grow `R, S` parameters) or `A` abstract (where `A` becomes "the next program" at the dispatch boundary, mirroring Phase 3 `State<'a, P, S, A>`).
- **Resolution: Option A.** `action: A` literal; `Run<R, S, A>` is loose notation for "the next program" bound by dispatch context, mirroring Phase 3 `State<'a, P, S, A>`'s use of `A`. Adding `R, S` parameters to every scoped-effect brand contradicts Phase 3 precedent without structural reason; brand parameter surface stays at 2 per scoped-effect type.
- **Plan-text amendments.** [decisions.md section 4.5](decisions.md#45-decision-scoped-effect-representation-via-a-heftia-inspired-dual-row) clarifying paragraph; [plan.md Phase 4 step 3](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) constructor-list paragraph specifying loose-notation semantics.

### B2. Per-scoped-effect-brand `Functor` / `SendFunctor` / `WrapDrop` / `RefFunctor` / `Extract` impls

- **Issue.** The substrate's `NodeBrand<R, S>` impls require `S: Functor + SendFunctor + WrapDrop + RefFunctor + Extract`. With `S = CNilBrand` (Phase 3) these are vacuously satisfied; with `S = CoproductBrand<CatchBrand<...>, ...>` (Phase 4), each scoped-effect brand must explicitly implement all five traits because [`RcFree::wrap`](../../../fp-library/src/types/rc_free.rs) and similar substrate operations call `<F as Functor>::map` directly. [decisions.md](decisions.md) had stated "the higher-order row does NOT require a Functor instance" which was misleading: the dispatcher trait does not go through Functor, but the substrate's program-traversal machinery still does.
- **Resolution: Option A.** Each scoped-effect brand provides explicit per-trait impls; Phase 3 per-effect impls are the template. Up to 30 trait impls total (5 brands \* ~5 traits, minus Span which has no closure); each is mechanical. Option B (Coyoneda wrapping per scoped effect) was rejected: doubles per-op allocation cost and contradicts the dual-row design's whole point. Option C (substrate redesign of `NodeBrand`'s Functor requirement) was rejected: justification was documentation alignment, not capability.
- **Plan-text amendments.** [decisions.md section 4.5](decisions.md#45-decision-scoped-effect-representation-via-a-heftia-inspired-dual-row) clarifying paragraph distinguishing dispatcher-trait vs program-traversal-trait requirements; [plan.md Phase 4 step 3](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) substrate-required-traits paragraph plus a code-block `impl Functor for CatchBrand<P, E>` template (added 2026-05-06 follow-up cleanup).

### B3. Pointer-brand parameterisation: `BoxBrand` + `ToDynFnOnce` on default Run

- **Issue.** The R2 plan revision specified a `BoxBrand` pointer brand for the default `Run` / `RunExplicit` substrate, with `BoxBrand`'s closure projection differing from `RcBrand` / `ArcBrand`'s (`Box<dyn FnOnce>` vs `Rc<dyn Fn>` vs `Arc<dyn Fn + Send + Sync>`). On cross-checking, Phase 3's State effect did not have a `BoxBrand`: State on `Run` used `RcBrand` per `Run::get`'s smart constructor signature. The original plan revision invented `BoxBrand` to symmetrise the wrapper-to-pointer-brand mapping, but the symmetry didn't hold in Phase 3 and forcing it into Phase 4 introduced a new brand whose closure-trait-object differed from its siblings.
- **Resolution: Option B (revised).** Use the existing pointer-abstraction infrastructure (`BoxBrand`, `RcBrand`, `ArcBrand` per [`fp-library/docs/pointer-abstraction.md`](../../../fp-library/docs/pointer-abstraction.md)) extended with a new trait `ToDynFnOnce` parallel to `ToDynFn`, implemented only by `BoxBrand`. `Rc<dyn FnOnce>` and `Arc<dyn FnOnce>` are operationally broken because `FnOnce::call_once` consumes `self` (the trait object), which cannot be moved out of a shared pointer without invalidating other clones. Default `Run` users get `Box<dyn FnOnce>` storage (single-shot at the closure level by construction); `RcRun` / `ArcRun` users get `Rc<dyn Fn>` / `Arc<dyn Fn + Send + Sync>` for multi-shot. Ergonomic gain: user-supplied scoped handlers on default `Run` can be `FnOnce`-natural (no Rc-wrapping or Clone-bounded captures). The retrofit applied to Phase 3's State / Reader / Choose closes the [F4 finding structurally](resolutions.md#resolved-2026-05-06-phase-3-prior-review-f4-closed-structurally-via-phase-35-retrofit-sibling-boxbrand-family-on-default-run-substrates) rather than as accepted-tradeoff.
- **Plan-text amendments.** [decisions.md section 4.5](decisions.md#45-decision-scoped-effect-representation-via-a-heftia-inspired-dual-row) trait-family table and `Closure-storage ceiling on default Run` subsection; new [Phase 3.5 sub-section in plan.md](plan.md#phase-35-pointer-brand-pattern-retrofit) with five sub-steps.
- **Implementation status.** Shipped in Phase 3.5 (sub-steps 1-5; commits `b067f912` / `a762fa27` / `89546709` / `4471629d`). The same pattern is reused in Phase 4 step 3 for user-supplied scoped handlers.

### B4. Catch dispatcher's sentinel mechanism

- **Issue.** The R1 plan revision said the catch dispatcher "interposes a `Throw` catcher that returns to a sentinel value, observes the sentinel via the dispatcher's outer `interpret` loop, and routes to `Catch::handler`". The sentinel's TYPE in the program was unspecified. `Run<R, S, A>`'s payload type is `A`; encoding "either A or thrown E" required either changing the program type or using a side channel.
- **Resolution: Option B with POC validation prerequisite.** New substrate primitive `interpret_with_either<EBrand, Idx>(self, fo_handlers: &impl DispatchHandlers<...>) -> Either<A, EBrand::Op>` (specialisation of `interpret_with` returning the matched effect's payload as `Right` instead of narrowing). The Catch dispatcher pattern-matches the Either; `Left(a)` becomes `Run::pure(a)`, `Right(thrown_e)` calls `Catch::handler`. No interior mutability; the type system structurally distinguishes "completed" from "thrown". Option A's interior-mutability cell with a placeholder-program return was rejected because the placeholder requires either `A: Default`, an unsafe sentinel, or a panic-on-evaluate `Box::leak`-style construct, none of which is clean. Option C (`std::panic::catch_unwind`) was rejected as unsound for non-`UnwindSafe` programs.
- **Plan-text amendments.** New [Phase 4 step 2a](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) substrate primitive entry; [Phase 4 step 4](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) Catch dispatcher description with `match interpret_with_either` body sketch.
- **Outstanding prerequisite.** POC 3 (`interpret_with_either` substrate primitive on `RcRun`) pending validation; commit-ordering decision is documented at [Phase 4 implementation-kickoff sequencing K1](plan.md#k1-poc-3-interpret_with_either-validation-ordering).
- **Superseded implementation detail (2026-05-09).** B27 keeps
  `interpret_with_either` as a first-order-only (`S = CNilBrand`)
  primitive. The standard scoped `CatchDispatcher` now uses
  scoped-row-preserving `interpose` so nested scoped operations inside a
  catch action survive.

### Q1. `scoped_handlers!` macro shape

- **Issue.** Phase 4 step 5 references a `scoped_handlers!{...}` macro as a companion to the existing `handlers!` macro for assembling the second list passed to `interpret`. Syntax and emitted shape were unspecified.
- **Resolution: Option A with DRY factoring.** `scoped_handlers!{CatchBrand<P, E>: |op| ..., ...}` mirrors `handlers!` syntax exactly; both factor through a new helper module [`fp-macros/src/effects/handler_list_emitter.rs`](../../../fp-macros/src/effects/) (new file) parameterised by cell type identifier (`Handler<E, F>` for FO; `ScopedHandler<S, F>` for scoped) and cons-list cell-and-tail type identifiers (`HandlersCons` / `HandlersNil` for FO; `ScopedHandlersCons` / `ScopedHandlersNil` for scoped). The `effects!` macro's lexical-sort helper is consumed inside the new emitter helper as well.
- **Plan-text amendments.** [Phase 4 step 5](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) helper-module reference + entry-point pattern. Deviations.md entry pending at the commit that retrofits `handlers!` through the helper.

### Q2. `define_scoped_effect!` macro fate

- **Issue.** Phase 4 step 5 referenced `define_scoped_effect!` "mirroring section 9's planned `define_effect!` for first-order effects". `define_effect!` was [deferred per the 2026-05-04 resolution](#resolved-2026-05-04-phase-3-step-6-define_effect-macro-deferred-until-phase-4-ships-or-user-demand-surfaces-design-research-preserved-for-later-revisit) (Phase 3 step 6) until Phase 4 ships or user demand surfaces; `define_scoped_effect!` had no precedent template.
- **Resolution: Option A.** Defer in parallel with the Phase 3 `define_effect!` deferral; users hand-write each scoped effect's brand + four-to-six trait impls (analog to Phase 3's `State` / `Reader` / etc.). Revisit triggers parallel the existing `define_effect!` deferred-item triggers (a user writing more than two custom scoped effects, or a Phase 6+ revisit of `define_effect!`).
- **Plan-text amendments.** [Phase 4 step 5](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) drops the `define_scoped_effect!` reference; [Phase 6+ deferred items](plan.md#phase-6-deferred-not-in-this-plan) gains a new entry parallel to the existing `define_effect!` entry.

### Q3. `interpret_scoped_with::<EBrand>` row-narrowing primitive

- **Issue.** Phase 4 step 7 references `interpret_scoped_with::<EBrand>` as a row-narrowing primitive on the scoped row paralleling Phase 3's `interpret_with`. Signature and substrate plumbing unspecified.
- **Resolution: Option A.** New per-wrapper method `interpret_scoped_with::<EBrand, Idx, SMinusE>(scoped_handler) -> Run<R, SMinusE, A>`, paralleling Phase 3's `interpret_with` on the scoped row. Mechanically derivable from Phase 3 pattern; users need pipeline-ordering control for non-commuting scoped effects (e.g., `Catch` before `Local` vs after). Option B (reuse `interpret_with` with type-level branching) has worse type-inference characteristics; Option C (no row-narrowing on scoped row) limits expressivity for handler libraries that ship narrowing handlers.
- **Plan-text amendments.** [Phase 4 step 7](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) callers-write-`interpret_scoped_with` paragraph already in canonical text. The per-wrapper method itself ships as part of Phase 4 step 4's interpret-rewrite.
- **Implementation status correction (2026-05-08 audit).** The design
  decision remains adopted, but the code did not actually ship
  `interpret_scoped_with` in Phase 4 step 4. Current wrappers expose
  `interpret`, `run`, `interpret_rec`, and `run_rec` over both rows,
  while `interpret_with`, `interpose`, and `interpret_with_either` are
  still scoped-row-empty primitives (`S = CNilBrand`). B24 later
  adopted the required scoped-row-preserving primitive retrofit for
  `interpret_scoped_with`, `interpret_with`, and `interpose` as Phase 4
  step 6a before standard scoped dispatchers proceed; B27 keeps
  `interpret_with_either` first-order-only.

### Q5. `BracketGuard<A, F>` lifecycle

- **Issue.** Phase 4 step 3's Bracket entries said the dispatcher wraps the resource in a `BracketGuard<A, F>` whose `Drop` impl invokes `release` synchronously. Specific lifecycle questions: when is the guard constructed? Does `body` receive the guard by ownership or reference? Does `release`'s `Run<R, S, ()>` get scheduled by Drop or executed synchronously? These small decisions combined into whether panic-during-body actually runs `release`.
- **Resolution: Option A.** RAII guard, ownership-passed to body, synchronous release-on-Drop. Guard constructed inside the bracket dispatcher after `acquire` evaluates; ownership-passed to the body closure; dropped when body returns (running release on drop). Release executes as a synchronous function call (not threaded through interpret) because the interpret loop has already exited the dispatcher's frame. Matches Rust's RAII conventions; release runs deterministically on body completion or panic; no reliance on `catch_unwind`'s `UnwindSafe` constraints. Option B (reference-passed + explicit-drop) is harder to reason about under panic; Option C (`catch_unwind`) is unsound for non-`UnwindSafe` programs.
- **Plan-text amendments.** [Phase 4 step 3 Bracket entries](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) `BracketGuard` lifecycle sentence specifying ownership-passed-to-body + synchronous-release-on-Drop semantics.
- **Superseded implementation detail (2026-05-09).** B26 supersedes the
  earlier "release executes as a synchronous function call" detail for
  the shipped Bracket cell shape. Release is effectful on the normal
  path and is interpreted by the dispatcher after body completion. Drop
  during unwind guarantees only ordinary resource cleanup, not
  interpretation of the effectful release program.

## Resolved (2026-05-04): Phase 3 step 6 (`define_effect!` macro) deferred until Phase 4 ships or user demand surfaces; design research preserved for later revisit

`define_effect!` was scoped as a proc-macro that mechanically
generates an effect enum + brand registration + per-wrapper
smart constructors from a single user declaration like:

```rust
define_effect! {
    Reader<E> {
        fn ask() -> E,
    }
}
```

Per-effect, the macro would emit ~200-400 lines of boilerplate:
the effect enum, `impl_kind!` brand registration, manual `Clone`,
[`Functor`](../../../fp-library/src/classes/functor.rs) /
[`SendFunctor`](../../../fp-library/src/classes/send_functor.rs)
impls, conditional Send-aware parallel brand + type pair, and
4 or 6 per-wrapper smart constructors. The macro itself was
estimated at ~1500+ lines of proc-macro code.

### Why deferred

Five reasons, in order of weight:

1. **Phase 4 (scoped effects, heftia dual row) may invalidate
   the codegen target.** Scoped effects use a different brand
   shape and a different per-wrapper rollout pattern than
   first-order effects. A `define_effect!` shipped now for
   first-order effects would either need significant rework or
   become a sibling-not-replacement when Phase 4 lands. Better
   to know the full target before mechanising.
2. **Pre-1.0 API instability bleeds into macros.** The recent
   step 5e substrate fix changed `RcFree`/`ArcFree`'s
   continuation queue from value-typed `CatList` to refcounted
   `RcCatList`/`ArcCatList`. A macro shipped before that would
   have hard-coded the wrong substrate and needed migration.
   The same risk exists for whatever refines next.
3. **No users yet to validate the input syntax.** Five design
   approaches were surveyed (see below) with non-trivial
   ergonomic differences; without real workloads it's not clear
   which is right.
4. **Library already ships 5 standard effects.** The boilerplate
   savings only apply to effects that don't yet exist. Without
   active demand for custom effects, the macro's break-even is
   ~4-8 future effects, which may take a long time to surface.
5. **Nothing in the rest of the plan depends on the macro.**
   Step 7 (`compile_fail` UI tests) and step 8 (review-
   remediation docs) validate / document the existing hand-
   written effects. Phase 4 (scoped) and Phase 5/6+ also do
   not depend on this macro.

### Trigger conditions for revisiting

Revisit when **either** of:

- **(a) Phase 4 ships** (scoped effects via heftia dual row).
  The full effect-shape design space is then settled; the macro
  can target both first-order and scoped effects, or be cleanly
  scoped to first-order if scoped effects are too different to
  share a macro.
- **(b) A real user surfaces concrete demand for custom
  effects** (a workload, not a hypothetical). The use case
  informs which design approach below is right.

If neither trigger fires within the foreseeable future, the
permanent answer "copy
`reader.rs`
and adapt for your effect" is also acceptable for a library
that already ships 5 standard effects covering the common
cases. A Phase 6+ HOWTO entry documenting that recipe would
close out the deferral.

### Design research (preserved for later revisit)

Five approaches were surveyed during the deferral discussion.
Listed roughly from most-terse to most-explicit.

#### Approach 1: PureScript-faithful (most terse)

```rust
define_effect! {
    Reader<E> {
        fn ask() -> E,
    }

    State<S> {
        fn get() -> S,
        fn put(s: S),
    }

    Writer<W> {
        fn tell(log: W),
    }

    #[multi_shot]
    Choose {
        fn choose() -> bool,
    }
}
```

The macro infers the variant encoding from the operation
signature:

- `fn op(args...) -> Ret` -> variant carries `args` (if any)
  plus a `dyn Fn(Ret) -> A` continuation.
- `fn op(args...)` (no return arrow) -> variant carries `args`
  plus `A` directly (no continuation;
  `PhantomData<&'a ()>` for the unused lifetime).

Pros: reads exactly like PureScript Run; minimal cognitive
overhead; naturally maps `fn ask() -> E` to "the operation
produces an `E` that the next program consumes".

Cons: the `fn op(...)` (no return) vs `fn op(...) -> ()`
distinction is subtle; users might expect them to mean the
same thing. Variant names are auto-derived from operation
names (`ask` -> `Ask`, `tell` -> `Tell`), which works for
PureScript-style naming but may clash on edge cases.

#### Approach 2: Explicit variant + constructor names

```rust
define_effect! {
    State<S> {
        Get: get() -> S,
        Put: put(s: S) -> (),
    }
}
```

`Variant: smart_constructor(args) -> ContType` syntax pairs
each enum variant with its smart-constructor entry-point.

Pros: variant names are explicit, matching PureScript
convention exactly. Operation-vs-variant distinction is
visible. The `-> ()` makes "this op has a unit continuation"
explicit (Put pattern).

Cons: slightly more verbose. Two names per op (variant +
constructor) when most map 1:1 (`Ask`/`ask`).

#### Approach 3: Plain Rust enum + attribute-driven generation

```rust
#[define_effect(brand = "ReaderBrand", multi_shot = false)]
pub enum Reader<E, A> {
    #[constructor(ask)]
    Ask(Continuation<E, A>),
}
```

Users write a normal Rust enum; marker types like
`Continuation<E, A>` signal what gets generated.

Pros: looks like normal Rust; users can mix custom variants
with macro-generated boilerplate; visible in `rust-analyzer`
even without macro expansion.

Cons: requires marker types in scope; doesn't match the
PureScript Run user-facing surface as closely; pointer-brand
`P` and lifetime `'a` parameters need separate inference rules.

#### Approach 4: Two-tier (separate brand registration from constructors)

```rust
define_effect_type! {
    Reader<E> {
        fn ask() -> E,
    }
}

// Generated: ReaderBrand, Reader<'a, P, E, A>, Functor,
// SendFunctor, SendReaderBrand, SendReader.
// User still hand-writes the smart constructors per wrapper.
```

The macro generates everything except the per-wrapper smart
constructors; users keep those hand-written for explicit
control over bounds.

Pros: smart constructors stay legible in source (where most of
the per-wrapper bound differences live); the macro is smaller
and more focused.

Cons: doesn't eliminate the largest single chunk of boilerplate
(~50-100 lines per smart constructor x 6 wrappers); users
still have to write them.

#### Approach 5: Single-tier with attribute escape hatches

Approach 1 plus attribute hooks for special cases:

```rust
define_effect! {
    State<S> {
        fn get() -> S,
        fn put(s: S),
    }

    #[no_send_aware]    // skip SendReaderBrand / SendReader
    Reader<E> {
        fn ask() -> E,
    }

    #[multi_shot]
    Choose {
        fn choose() -> bool,
    }

    #[wrappers(Run, RunExplicit)]    // override default 6-wrapper rollout
    SingleShotOnly {
        fn op() -> i32,
    }
}
```

Pros: default behavior is what users want 95% of the time;
attributes handle the corner cases without polluting the base
syntax.

Cons: attribute set is open-ended; risk of feature creep over
time.

### Open design questions (deferred along with the macro)

1. **Variant-encoding inference.** How does `fn op(args)` vs
   `fn op(args) -> Ret` distinguish "no continuation,
   `A` is owned directly" (Tell) from "continuation
   `Fn(Ret) -> A`" (Ask/Get/Put/Choose)? Three plausible rules:
   - **(a)** Missing return arrow -> no continuation;
     `-> ()` -> `Fn(()) -> A` continuation;
     `-> T` -> `Fn(T) -> A` continuation. Distinguishes
     Tell/Put/Ask cleanly.
   - **(b)** Always generate a continuation; the user signals
     "no continuation" via a marker
     (`fn tell(log: W) using direct;` or
     `#[direct] fn tell(log: W);`).
   - **(c)** Each declaration explicitly states its
     continuation type.
2. **Send-aware brand: auto or opt-in?** Auto-generate
   `SendXxxBrand` for every effect that has a `dyn Fn`
   continuation (matches existing State/Reader/Choose
   pattern), or require `#[send_aware]` opt-in?
3. **`multi_shot` placement.** Per-effect attribute, per-
   operation attribute, or syntax keyword?
4. **Pointer-brand parameter `P` inference.** Effects with
   `dyn Fn` continuations need a `P: ToDynCloneFn` parameter;
   effects without (Writer, Except) don't. Auto-detect from
   the presence of any continuation-bearing operation, or
   require explicit declaration?
5. **User-extensibility.** Can a user add custom variants
   alongside macro-generated ones? Or is the enum closed by
   the macro? PureScript users sidestep this because their
   data declarations are open by construction. Approach 3
   (proc-macro on a plain enum) handles this naturally;
   approach 1 doesn't without explicit support.
6. **Migration path.** If/when the macro ships, should the
   five existing effects be migrated to dogfood it, or shipped
   untouched? If migrated, the macro must produce
   byte-equivalent code (modulo doc comments) so the test
   suite continues to pass.
7. **Doc comment placement.** Where do user-supplied doc
   comments land? On the variant, the smart constructor, the
   brand, or all three? Per-component override?

### Tentative recommendation if revisited

Pre-deferral, **Approach 1 (PureScript-faithful) with rule
(1.a)** was the leading option: closest match to the existing
user-facing surface, smallest cognitive load, naturally
auto-generates Send-aware parallels for continuation-bearing
effects. The most uncertain question was **(5) user-
extensibility**; if a future workload needs that, approach 3
becomes more attractive.

This recommendation is non-binding; revisit with full Phase 4
context (or user-workload context, depending on which trigger
fires first).

## Resolved (2026-05-04): Phase 3 step 5e Erased Free family multi-shot dispatch via `RcCatList`/`ArcCatList` (option 1c-ii: parallel reference-counted CatList variants)

`Choose` smart constructors on `RcRun` and `ArcRun` were
panicking at runtime with "`RcFree::to_view map called more
than once`" / "`ArcFree::to_view map called more than once`"
because the Erased Free family's continuation queue was held
in a value-typed [`CatList`](../../../fp-library/src/types/cat_list.rs)
whose derived `Clone` is O(N) deep-recursive. To compensate,
[`RcFree::to_view`](../../../fp-library/src/types/rc_free.rs)
captured the queue inside a `Cell<Option<...>>` and consumed it
once via `take()`, making the closure structurally single-shot
even though its outer `Rc<dyn Fn>` wrapping permitted multiple
invocations. The `Choose` handler runs the continuation twice
(once per branch) and tripped the panic on the second call.

PureScript's `CatList` is naturally O(1)-cloneable because the
language is GC-managed and immutable: "cloning" is just copying
a heap reference. The Rust port chose `VecDeque<CatList<A>>`
with a derived `Clone`, which is cache-local and ergonomic for
a regular catenable list but has no structural sharing, so
clones cost O(N). The Cell/Mutex pattern in `to_view` was a
workaround for that ownership friction.

### What landed

Two new substrates: [`RcCatList<A>`](../../../fp-library/src/types/rc_cat_list.rs)
and [`ArcCatList<A>`](../../../fp-library/src/types/arc_cat_list.rs).
Each wraps `Rc<VecDeque<RcCatList<A>>>` (or `Arc<...>` for the
Send + Sync sibling), so `Clone` is a refcount bump. Mutation
methods (`snoc`, `append`, `cons`) use `Rc::make_mut`/`Arc::
make_mut` for copy-on-write; uniquely-owned deques mutate in
place, shared deques clone one level deep (each contained
element clones in O(1)). The `A: Clone` bound is required on
mutation/uncons; the Free family's continuation types
(`RcContinuation`, `ArcContinuation`) already implement Clone
via `Rc::clone`/`Arc::clone`.

`RcFree::to_view` and `ArcFree::to_view` switched to capture
the continuation list by move and clone it (`all_conts.clone()`)
on every invocation. The `Cell<Option<...>>` and
`Mutex<Option<...>>` workarounds are gone, including the per-
layer mutex acquire on the Arc path that was only ever in
place for `Sync` compatibility, never for actual concurrency
control. `ArcCatList` is structurally `Send + Sync` whenever
`A` is, so the closure satisfies the trait-object bounds on
`Arc<dyn Fn(...) + Send + Sync>` without any synchronisation
primitive.

The new substrates implement only the surface needed by the
Free family (`empty`, `is_empty`, `singleton`, `cons`, `snoc`,
`append`, `uncons`, `len`, `Clone`, `Default`, iterative
`Drop`). The full trait soup on
[`CatListBrand`](../../../fp-library/src/brands.rs) (Functor,
Foldable, Traversable, etc.) is intentionally not mirrored;
`RcCatList`/`ArcCatList` are continuation-queue substrates,
not general-purpose lists. Existing `CatList` is unchanged.

### Why option 1c-ii over the alternatives

Five sub-options were surveyed. Approach 1a (clone the
existing `CatList` per call) ran into the O(N) deep-clone
cost on the single-inner hot path (Identity, State, Reader,
Except, Writer). Approach 1b (Rc-wrap the existing CatList
at the use site) didn't actually help, since `append`
consumes the list and would still need a deep clone before
mutation. Approach 1c-i (a single Arc-everywhere CatList)
would penalise `RcFree`'s hot path with atomic refcount ops.
Approach 1c-iii (generic over `RefCountedPointer`) leaks
bound noise through every type signature that touches
CatList. Approach 1d (don't fold continuations into `F::map`
at all) would unwind the Phase 1 stack-safety story by
walking continuations one at a time per Suspend.

1c-ii (parallel `RcCatList`/`ArcCatList` types) mirrors the
codebase's existing Rc/Arc split (`RcCoyoneda`/`ArcCoyoneda`,
`RcFree`/`ArcFree`, `RcBrand`/`ArcBrand`). The duplication is
straightforward (the implementations differ only in `Rc` vs
`Arc` and the resulting auto-trait derivations) and matches
the convention every other type in the split already pays.

### Validation

All 4 `run_choose.rs`
integration tests pass on the new substrate (one per
multi-shot wrapper). The full pre-existing test suite passes
unchanged, confirming no regression on single-inner Free
workloads. Per-Suspend cost on those workloads adds one
`Rc::clone` (or `Arc::clone`), which is a refcount bump
rather than a structural copy.

### What's preserved

The 2026-05-03 wrapper-parameterization resolution Q4=ii
("`Choose` ships on all four multi-shot wrappers") is fully
honored; no demotion needed. The substrate fix unblocks the
same API surface that resolution committed to.

### Future direction

If a future workload surfaces a measurable need for a
fully-featured `RcCatList`/`ArcCatList` (Functor, Foldable,
Traversable, brand-level dispatch), the additional surface
can be added incrementally. The current scope deliberately
ships the minimum needed by the Free family.

## Resolved (2026-05-04): Phase 3 step 5 (`interpret_with_rec`) deferred indefinitely (option (c))

Pipeline row-narrowing combined with `MonadRec`-target stack
safety does not compose cleanly without a richer abstraction
(e.g., `Codensity`, `Eff`-style continuation passing, or
`Traversable` on every row brand). PureScript Run does not
ship the combination either; the gap is structural, not just
an implementation oversight. Phase 3 ships three orthogonal
interpreter primitives instead of four; users who want both
row narrowing and stack safety chain
`interpret_with`
(narrowing) and
`interpret_rec`
(stack safety) at the boundary of the pipeline.

### The structural obstacle

The
[prompt.md](prompt.md)
"Step 5 implementation pattern" subsection specified the
handler signature as

```rust
handler: impl Fn(<EBrand as Kind>::Of<'_, M::Of<'_, Wrapper<RMinusE, CNilBrand, A>>>)
    -> M::Of<'_, Wrapper<RMinusE, CNilBrand, A>>
```

i.e. inners arrive at the handler already narrowed
(`Run<RMinusE, ...>`) and M-wrapped. But producing narrowed
inners requires structural recursion through the inner
programs.
`Run::interpret_with_shared`'s
matched arm uses
`EBrand::Functor::map(|inner: Run<R, ...>| inner.interpret_with_shared(...), lowered)`
to recursively narrow each inner before handing the layer to
the handler. The recursion is structural (one frame per peel)
and lazy for closure-shaped effects (e.g.,
`State`'s
continuations defer the recursion).

`Run::interpret_rec`
sidesteps recursion via
[`tail_rec_m`](../../../fp-library/src/classes/monad_rec.rs)
because the row collapses fully (no narrowing): the step
closure produces `M(ControlFlow<A, Run<R, CNilBrand, A>>)`,
where the loop state is the un-narrowed program and the
final result is the plain `A`. There's no per-layer
structural-recursion need.

The conjunction (`interpret_with_rec`) requires per-layer
structural recursion (for narrowing) AND `tail_rec_m`-driven
linear iteration (for stack safety). For multi-inner row
layers (e.g.,
[`VecBrand`](../../../fp-library/src/types/vec.rs)-shaped
effects), the unmatched arm needs to swap
`RMinusE::Of<M::Of<...>>` to `M::Of<RMinusE::Of<...>>`,
which requires
[`Traversable`](../../../fp-library/src/classes/traversable.rs)
on `RMinusE` plus
[`Applicative`](../../../fp-library/src/classes/applicative.rs)
on `M`. Non-rec `interpret_with` sidesteps this by using just
[`Functor::map`](../../../fp-library/src/classes/functor.rs)
(no M to swap with). PureScript Run skips the combination for
the same reason.

### Options surveyed

**(a) Handler keeps `interpret_with`'s narrowed-inner shape;
implementation does inner structural recursion + outer
`tail_rec_m`.** Step closure type
`Run<R, ..., A> -> M(ControlFlow<Run<RMinusE, ..., A>, Run<R, ..., A>>)`.
Matched arm: structurally-recurse into each inner; M-fmap
pure to wrap; hand to handler; M-fmap `Break`. Outer loop
terminates after one peel: `tail_rec_m` is decorative.

- _Pros:_ matches the prompt's handler signature; user
  ergonomics consistent with non-rec `interpret_with`.
- _Cons:_ stack-safety benefit is whatever the matched
  effect's M-bind chain provides, which is independent of
  `tail_rec_m` and already available via plain
  `interpret_with` followed by M-bind composition. ~600
  lines plus ~12 Send + Sync clauses on the Arc family for
  ~zero genuine benefit.

**(b) Handler returns `M::Of<Run<R, CNilBrand, A>>`
(next-state in the un-narrowed row).** Step closure same
type; matched arm: hand original-row inners to handler;
handler returns `M::Of<Run<R, ...>>`; M-fmap `Continue`.
Outer loop iterates per matched-effect occurrence.
Unmatched arm still needs `Traversable` for multi-inner
layers.

- _Pros:_ `tail_rec_m` actually iterates; closure-shaped
  matched effects (`State`) get genuine stack safety.
- _Cons:_ handler signature differs from non-rec
  `interpret_with`: handler can't compose in the narrowed
  row, only the original row. Most non-rec `interpret_with`
  handlers don't translate. Unmatched arm requires
  `Traversable` on the row brand which is too heavy a
  requirement to put on row brands.

**(c) Defer step 5 indefinitely.** Document that the
combination doesn't compose cleanly without a richer
abstraction. Users who want both row narrowing and stack
safety chain `interpret_with` (narrowing, no stack safety)
followed by `interpret_rec` (stack safety on the
all-handlers-at-once form). Phase 3 ships the two
interpreter primitives separately; the combined form
becomes a Phase 6+ concern when a richer abstraction is in
scope.

- _Pros:_ zero implementation cost; honest about the
  abstraction limit; existing primitives are sufficient for
  most use cases.
- _Cons:_ closes off one of the four cells in the
  cognitive-model matrix (M-free pipeline / M-free
  all-handlers / M-target pipeline / M-target all-handlers).

**(d) Re-derive from PureScript Run; no analog exists.**
PS Run doesn't have `interpretWithRec`. Equivalent to (c)
with the explicit "no PS analog" justification.

### Resolution: option (c) (with (d)'s justification merged in)

Locked in. The abstraction tension reflects a real design
limit, not just an implementation gap. Deferring preserves
the option to revisit with a richer encoding (e.g.,
`Codensity`-style transformation) when one is in scope. The
"three interpreter primitives" framing replaces the original
four-cell matrix throughout the docs.

### Workaround for users wanting both

```rust
// Stack-safe pipeline: narrow effects one at a time
// (no stack safety on the narrowing itself), then drive
// the all-handlers stage via `interpret_rec` for stack
// safety on the long-bind-chain effects (e.g., State).
let narrowed: Run<RMinusE, CNilBrand, A> =
    prog.interpret_with::<E1, _, _>(handler1);
let result: ThunkBrand::Of<'static, A> =
    narrowed.interpret_rec::<ThunkBrand>(handlers! { /* ... */ });
```

The intermediate `interpret_with` calls have no stack-safety
guarantee, but the final `interpret_rec` provides
`tail_rec_m`-driven safety for the dispatched effects'
M-bind chains. For programs whose effect rows can be peeled
without unbounded recursion in the narrowing stage (the
common case), this composition is sufficient.

### Forward-looking: Rust-native iteration could make `interpret_with` itself stack-safe

PureScript needs `MonadRec` because the language lacks native
iteration; Rust has `loop`/`while` and unrestricted mutable
stacks. That changes the design space at the implementation
level (not the API level): the structural-recursion concern
in non-rec `interpret_with` (each `Identity`-shaped layer
adds a host stack frame) can in principle be replaced with a
manual work-stack state machine. Maintain a stack of
partially-narrowed layers, process one inner at a time,
accumulate results into the final narrowed program. CESK /
SECD-style; awkward in PS, implementable in Rust at ~500-800
lines.

This is a Rust-specific _implementation_ option for
`interpret_with`
itself, not a path back to the deferred `interpret_with_rec`
public API. The multi-inner unmatched-arm `Traversable`
obstacle that motivated the public-API deferral is
independent of how the per-inner recursion is driven; native
iteration changes the loop driver but not the M-swap
requirement.

If a user reports stack overflows in deeply-nested
`interpret_with` pipelines (the eager `Identity`-style layer
case), the appropriate response is to land an
internal-iteration deviation against `interpret_with` rather
than to revisit this resolution. The public-API decision (no
fourth primitive) stays.

### Documentation impact

- [decisions.md](decisions.md) updated to record that
  Phase 3 ships three interpreter primitives, not four.
- [plan.md](plan.md)'s phasing list dropped step 5;
  steps 6, 7, 8, 9 renumbered to 5, 6, 7, 8.
- [prompt.md](prompt.md)'s "Step 5 implementation pattern"
  subsection replaced with a "Deferred: combined pipeline +
  MonadRec" note pointing here.

### Cross-references

- `Step 3 `interpret_with``:
the row-narrowing primitive; structural recursion in
`interpret_with_shared`'s matched arm is the precedent.
- `Step 4 `interpret_rec``:
  the MonadRec-target primitive; loop state is un-narrowed
  program, no structural recursion needed.
- [`tail_rec_m`](../../../fp-library/src/classes/monad_rec.rs):
  the stack-safe loop driver; step closure shape is
  `A -> M(ControlFlow<B, A>)`.

## Resolved (2026-05-04): Phase 3 step 6a downstream blocker; `ArcCoyoneda`'s algebra migrated to `SendFunctor` (option (a))

The
[2026-05-03 SendFunctor option-(c) resolution](#resolved-2026-05-03-phase-3-step-6a-sendfunctor-reopened-after-option-b-unimplementable-option-c-parallel-sendstatebrand-ratified)
ratified a parallel
[`SendStateBrand<P, S>`](../../../fp-library/src/brands.rs)
/
`SendState<'a, P, S, A>`
type for the Arc family State effect. The 6a.4 + 6a.6 smart
constructors compiled under that resolution, but attempting to
ship `run_state.rs` integration tests surfaced a downstream
gap: the
[`ArcCoyoneda`](../../../fp-library/src/types/arc_coyoneda.rs)
dispatch path required `EBrand: Functor + SendFunctor`
(`interpreter.rs:337`),
but `SendStateBrand` cannot honestly implement `Functor`
because [`Functor::map`](../../../fp-library/src/classes/functor.rs)'s
signature only requires `f: impl Fn` (no `Send + Sync` bound)
while `SendState`'s variants store
`<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> A + Send + Sync>`
(closures must be `Send + Sync` at storage time).

### Three flavours of "Send-aware"

[`ArcCoyoneda`](../../../fp-library/src/types/arc_coyoneda.rs)
satisfied two of three Send-awareness properties but not the
third:

1. **Storage Send-aware** (yes): the inner
   `ArcCoyonedaLowerRef` trait has `: Send + Sync + 'a` as a
   supertrait; layer fields are `Send + Sync`; the whole value
   can cross thread boundaries.
2. **Operations Send-aware** (yes): `lower_ref` is callable
   from a spawned thread.
3. **Algebra Send-aware** (no, pre-migration): `lower_ref`'s
   body invoked `F::map` (no `Send + Sync` bound on the
   closure parameter).

For brands implementing both `Functor` and `SendFunctor`,
`lower_ref` worked because `Functor::map` happened to produce
a `Send + Sync` result when inputs were. The system implicitly
relied on every `Functor` impl in the project producing
`Send + Sync` results; a coincidence rather than a guarantee.
For brands like `SendStateBrand` whose structure pins
`Send + Sync` on the closure, the gap surfaced as a hard
compile error: the dispatch impl's `Functor` bound was
unsatisfiable.

### Phase 2 step 9d precedent

[Phase 2 step 9d](#resolved-2026-04-28-implementation-expansion-step-9-sendfunctor-cascade-prerequisites-for-arc-family)
explicitly migrated
[`ArcFree`](../../../fp-library/src/types/arc_free.rs) and
[`ArcFreeExplicit`](../../../fp-library/src/types/arc_free_explicit.rs)
from `F: Functor` to `F: SendFunctor` for the same reason: a
Send-aware substrate's algebra should propagate Send-aware
bounds. `ArcCoyoneda` was not part of that migration because
no Send-only brand existed inside ArcCoyoneda at the time.
`SendStateBrand` was the first.

### Options surveyed

**(a) Migrate `ArcCoyoneda`'s algebra to `SendFunctor`.**
Replace `F: Functor` with `F: SendFunctor` on the inner
`ArcCoyonedaLowerRef` trait method, the three layer impls
(Base, MapLayer, NewLayer), and the public
`ArcCoyoneda::lower_ref` plus all derived methods. Bodies use
[`F::send_map`](../../../fp-library/src/classes/send_functor.rs)
instead of `F::map`. Drop `+ Functor` from the dispatch impl.

**(b) Add a parallel `send_lower_ref` method.** Keep
`lower_ref` bound on `F: Functor`; add a new `send_lower_ref`
method bound on `F: SendFunctor`. Layer impls implement both.
Discovered structurally harder than it sounded:
`MapLayer::send_lower_ref`'s recursion needs `B: Send + Sync`
on the intermediate type, but Rust forbids tightening
trait-method where-clauses on impls beyond what the trait
declares, so the bound has to live at the struct level
(`ArcCoyonedaMapLayer<F, B: Send + Sync, A>`), which
propagates back to `ArcCoyoneda::map<B>` and breaks callers
using non-Send + Sync intermediates.

**(c) Defer 6a.4 + 6a.6 indefinitely.** Document the gap;
leave `SendStateBrand` and the Arc-family smart constructors
in place but un-dispatch-able. Rejected because it abandons
the locked-in "six variants per effect" design from the
[2026-05-03 wrapper-parameterization resolution](#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization).

**(c'') Define a parallel `SendArcCoyoneda`.** Additive (no
breaking change), but doubles the Coyoneda type surface and
complicates the per-wrapper Coyoneda variant pairing rule
from Phase 2 step 9h.

### Resolution: option (a)

Locked in. The migration is bounded; the Phase 2 step 9d
precedent is exactly this; the existing implicit "every
Functor produces Send + Sync" reliance is fragile and would
break the next time a Functor-only-but-not-Send-friendly
brand surfaces.

### Bound placement: method-level + impl-block-level

Matching ArcFree's precedent:

- `A: Send + Sync + 'a` lives at the main impl block.
- `F: SendFunctor` lives at method-level where-clauses.
- The struct definition stays minimal.

### Brand-level `Foldable` on `ArcCoyonedaBrand` dropped (deferred to a follow-up)

The brand-level
[`Foldable`](../../../fp-library/src/classes/foldable.rs)
impl on `ArcCoyonedaBrand` was dropped because
`Foldable::fold_map`'s trait method declares `A: Clone` only,
but the post-migration body needs `A: Send + Sync`. Rust
forbids tightening trait method bounds in impls, so the impl
cannot be salvaged without a parallel `SendFoldable` trait.
That trait is a separate follow-up; the inherent
`ArcCoyoneda::fold_map` method (now `F: SendFunctor`-bound)
covers the user-facing surface in the meantime.

### Migration scope

- [`fp-library/src/types/vec.rs`](../../../fp-library/src/types/vec.rs):
  added `SendFunctor` impl for `VecBrand` (byte-identical
  body to `Functor::map`'s, with tighter `Send + Sync` bounds).
- [`fp-library/src/types/arc_coyoneda.rs`](../../../fp-library/src/types/arc_coyoneda.rs):
  inner trait, three layer impls (Base, MapLayer, NewLayer),
  and public methods (`lower_ref`, `collapse`, `hoist`,
  `fold_map`, `bind`, `apply`, `lift2`) migrated;
  `From<ArcCoyoneda> for Coyoneda` bounds tightened to
  `F: SendFunctor` and `A: Send + Sync`; brand-level
  `Foldable` impl on `ArcCoyonedaBrand` dropped.
- `fp-library/src/types/effects/interpreter.rs`:
  dropped `+ Functor` from the ArcCoyoneda dispatch impl's
  `EBrand` bound (now just
  `EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static`).
- `fp-library/src/types/effects/arc_run.rs`:
  added `A: Send + Sync` to the `lift_node` helper's
  where-clause.
- [`fp-library/tests/ui/arc_coyoneda_requires_send.stderr`](../../../fp-library/tests/ui/arc_coyoneda_requires_send.stderr):
  re-blessed because the `Send + Sync` rejection now points
  at the impl-block-level bound rather than the inner
  `Apply!` clone-bound.

### Cross-references

- [Original (b) ratification](#resolved-2026-05-03-phase-3-step-6a-sendfunctor-impl-on-statebrand-for-the-arc-family-option-b-per-method-bounds):
  the first SendFunctor blocker resolution.
- [Option (c) re-ratification](#resolved-2026-05-03-phase-3-step-6a-sendfunctor-reopened-after-option-b-unimplementable-option-c-parallel-sendstatebrand-ratified):
  the parallel `SendStateBrand` design that surfaced this
  downstream issue.
- [Phase 2 step 9d resolution](#resolved-2026-04-28-implementation-expansion-step-9-sendfunctor-cascade-prerequisites-for-arc-family):
  the `ArcFree` migration precedent.
- [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md):
  the project-wide pattern table for Send-awareness gaps.

## Resolved (2026-05-03): Phase 3 step 6a SendFunctor reopened after option (b) unimplementable; option (c) parallel `SendStateBrand` ratified

The
[earlier 2026-05-03 ratification of option (b)](#resolved-2026-05-03-phase-3-step-6a-sendfunctor-impl-on-statebrand-for-the-arc-family-option-b-per-method-bounds)
(commit `4bd1636`) locked in per-method `Send + Sync` bounds
at smart-constructor sites. Implementation of `ArcRun::get` /
`ArcRun::put` under that lock-in failed at compile time; the
blocker was reopened and re-ratified with option (c).

### Why option (b) was unimplementable

`State<'a, P, S, A>`
holds:

- `Get(<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(S) -> A>)`
- `Put(S, <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>)`

For `P = ArcBrand`, the projection is
`Arc<dyn 'a + Fn(...) -> A>`. The trait object's bounds are
`'a + Fn(...) -> A` only , no `+ Send + Sync` baked in. Since
`Arc<T>: Send + Sync` requires `T: Send + Sync` structurally,
and `dyn Fn(...)` (without `+ Send + Sync`) is structurally
`!Send + !Sync`, the projection
`Arc<dyn Fn(...)>: Send + Sync` is provably false at the type
level.

Concrete rustc error from the attempted implementation:

```
error[E0277]: `(dyn Fn(()) + 'static)` cannot be shared between threads safely
   = help: the trait `Sync` is not implemented for `(dyn Fn(()) + 'static)`
   = note: required for `Arc<(dyn Fn(()) + 'static)>` to implement `Sync`
```

Adding the bound
`<ArcBrand as RefCountedPointer>::Of<'static, dyn 'static + Fn(()) -> ()>: Send + Sync`
to the smart-constructor's where-clause does not satisfy
this: rustc rejects the bound because the underlying type is
structurally `!Send + !Sync`, and use-site bounds cannot
refine a structural type-level fact.

The (b) analysis conflated two superficially-similar cases:
the existing per-method `Send + Sync` bounds on Arc-family
`interpret_with` / `interpret_rec` work because they apply to
projections of _concrete generic structs_ like
`ArcFree<NodeBrand<R, S>, ...>`, where `Send + Sync` _can_ be
true for specific instantiations. `Arc<dyn Fn(...)>` is a
different beast: the dyn trait object's marker-trait bounds
are part of its type identity, so `dyn Fn(...)` and
`dyn Fn(...) + Send + Sync` are different types.

### Decision: option (c) parallel `SendStateBrand` / `SendState`

Add a parallel
[`SendStateBrand<P, S>`](../../../fp-library/src/brands.rs)
brand registration alongside `StateBrand<P, S>`, and a
parallel
`SendState<'a, P, S, A>`
enum whose variants use the Send-aware projection:

- `Get(<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(S) -> A + Send + Sync>)`
- `Put(S, <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>)`

For `P = ArcBrand`, the projection is
`Arc<dyn 'a + Fn(...) + Send + Sync>` , a different type from
the non-Send `Arc<dyn Fn(...)>` in `State`, and one that IS
`Send + Sync` because the trait object's bounds now include
the marker traits.

Brand-level `SendFunctor` impl on `SendStateBrand<P, S>` is
implementable because `SendRefCountedPointer::Of<'a, T>`'s
trait bound carries `T: ?Sized + Send + Sync + 'a`,
guaranteeing the projection is structurally `Send + Sync` for
any valid `T`. No HRTB-over-types needed.

The non-Arc smart constructors (`Run::get/put` /
`RunExplicit::get/put` / `RcRun::get/put` /
`RcRunExplicit::get/put`) keep using `StateBrand<P, S>` in
their rows; they cannot use `SendStateBrand` because
`RcBrand` does not implement `SendRefCountedPointer` (`Rc` is
`!Send`). The Arc smart constructors (`ArcRun::get/put` /
`ArcRunExplicit::get/put`) use `SendStateBrand<ArcBrand, S>`
in their rows.

### Why other alternatives were rejected (again)

**(a) HRTB-over-types**: still unimplementable on stable Rust.

**(b) Per-method bounds**: discovered unimplementable, see
above. The original (b) ratification stays in
[resolutions.md](#resolved-2026-05-03-phase-3-step-6a-sendfunctor-impl-on-statebrand-for-the-arc-family-option-b-per-method-bounds)
as historical record.

**(d.1) Polymorphic-projection State**: parameterize
`State<'a, P, S, A>` over a "function-pointer-shape" trait
that returns `RefCountedPointer::Of<dyn Fn>` for non-Send
brands and `SendRefCountedPointer::Of<dyn Fn + Send + Sync>`
for Send brands. The "function-pointer-shape" trait would
need a per-`A`-and-`S` associated type to express the
parametric projection, which collapses back into the
HRTB-over-types wall.

**(d.2) Unconditional Send-aware representation**: change
`State<'a, P, S, A>` to use the Send-aware projection
unconditionally. Forces RcBrand users' closures to be
`Send + Sync`, breaking the canonical `Rc<RefCell<S>>`
capture pattern for state threading (`Rc` is `!Send`).
Rejected.

**(d.3) Two struct-level variants `State` and `SendState`**:
effectively option (c) under a different name. The chosen
implementation IS this; calling it (c) keeps continuity with
the original blocker analysis.

**(e) Defer-and-document**: viable, but the implementation
cost of (c) is small (~1 new brand, ~1 new enum, ~4 new impls
plus 4 smart-constructor methods, all parallel to existing
`State` shape), so the cost-benefit favors shipping over
deferring.

### Implementation phasing

1. New
   [`SendStateBrand<P, S>`](../../../fp-library/src/brands.rs)
   registration.
2. New `SendState<'a, P, S, A>` enum in
   `state.rs`
   alongside `State<'a, P, S, A>`. Or in a new
   `send_state.rs` module if `state.rs` grows too large.
3. `impl_kind!` for `SendStateBrand`.
4. `Functor` impl for `SendStateBrand`.
5. Manual `Clone` impl for `SendState` (gated on `S: Clone`,
   like the existing `State::Clone` impl from 5a.3).
6. `SendFunctor` impl for `SendStateBrand` (the whole point
   of (c) , this works because the projection is structurally
   `Send + Sync`).
7. `ArcRun::get<Idx>()` and `ArcRun::put<StateType, Idx>(s)`
   smart constructors using `SendStateBrand<ArcBrand, A>` in
   the row, plus `<ArcBrand as ToDynSendFn>::new(closure)`
   for continuation construction.
8. `ArcRunExplicit::get/put` same pattern.

The user-facing API surface for State is now:

- Single-thread programs: use `StateBrand<RcBrand, S>` (or
  `StateBrand<ArcBrand, S>` if the substrate happens to be
  Arc but you don't need thread-safety).
- Thread-safe programs that lift state-effect closures into
  Arc-substrate Run programs: use
  `SendStateBrand<ArcBrand, S>`.

Users with mixed programs face two distinct row brands they
must use depending on the substrate. The
[`define_effect!`](../../../fp-macros/src/effects/) macro
(Phase 3 step 7) can generate the per-wrapper smart
constructors that hide this distinction by selecting the
right brand per wrapper.

### Cross-references

- [Original (b) ratification](#resolved-2026-05-03-phase-3-step-6a-sendfunctor-impl-on-statebrand-for-the-arc-family-option-b-per-method-bounds):
  the superseded resolution.
- [Phase 3 step 5a.1 deviations entry](deviations.md): the
  original `State` design with `Functor`-only impl.
- [Phase 3 step 5a.3 deviations entry](deviations.md): the
  `State::Clone` impl that 6a.4 / 6a.6 cascade requires.
- [`SendRefCountedPointer`](../../../fp-library/src/classes/send_ref_counted_pointer.rs):
  the trait powering `SendState`'s projection.
- [`ToDynSendFn`](../../../fp-library/src/classes/to_dyn_send_fn.rs):
  parallel to `ToDynCloneFn`, used to construct Send-aware
  `dyn Fn + Send + Sync` continuations.

## Resolved (2026-05-03): Phase 3 step 6a `SendFunctor` impl on `StateBrand` for the Arc family (option (b) per-method bounds)

`StateBrand<P, S>`
shipped in step 6a.1 with the `Functor` impl only; the
`SendFunctor` impl was deferred. Without `SendFunctor`, the
Arc family smart constructors (`ArcRun::get/put` /
`ArcRunExplicit::get/put`, step 6a.4 and 6a.6) could not ship
via the same path that 6a.3 / 6a.5 use, because
`ArcCoyoneda`'s `Member::project` and `lower_ref` paths
require the inner projection to be `Send + Sync` per-`A`. The
bound
`<P as RefCountedPointer>::Of<'_, dyn 'a + Fn(S) -> A>: Send + Sync`
must be expressed for each `A` the smart constructor
produces, which hits stable Rust's HRTB-over-types limit (the
same constraint family that drove the brand-level
`SendFunctor` cascade gaps in Phase 2 step 9d / 9g / 9i).

### Background

[`SendFunctor`](../../../fp-library/src/classes/send_functor.rs)
adds `Send + Sync` bounds on the input/output types and the
closure to the `Functor::map` contract. The
[`ArcCoyoneda`](../../../fp-library/src/types/arc_coyoneda.rs)
dispatch impl in
`interpreter.rs`
requires `EBrand: SendFunctor` and
`<EBrand as Kind>::Of<'a, NextProgram>: Send + Sync + 'a`.

For `StateBrand<P, S>` with `P = ArcBrand`, the projection is
`State<'a, ArcBrand, S, A>`. The Get and Put variants hold
`<ArcBrand as RefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> A>`
= `Arc<dyn 'a + Fn(...) -> A>`. **This `Arc<dyn Fn>` is NOT
`Send + Sync` by default**: `RefCountedPointer::Of` provides
`Clone + Deref + 'a` but no `Send + Sync` guarantee. For the
`Arc<dyn Fn>` to be thread-safe, the inner `dyn Fn` must
carry `+ Send + Sync`.

### Decision: option (b) per-method `Send + Sync` bounds at smart-constructor sites

No brand-level `SendFunctor` impl on `StateBrand<P, S>`.
Instead, `ArcRun::get/put` and `ArcRunExplicit::get/put`
explicitly require
`<ArcBrand as RefCountedPointer>::Of<'_, dyn Fn(...)>: Send + Sync`
in their where-clauses for the specific `A` the constructor
produces. Callers see the bound at use sites; the
`SendFunctor::send_map` cascade through `ArcCoyoneda`
resolves because each `A` instantiation gets its own concrete
bound discharged.

This mirrors `ArcRunExplicit`'s precedent (every Arc-family
`interpret_with` / `interpret_rec` method already carries
~12 per-method `Send + Sync` clauses rather than struct-level
ones). Uniform precedent across the codebase.

### Alternatives considered and rejected

**(a) Brand-level `SendFunctor` impl with HRTB.** Would need

```rust
impl<S> SendFunctor for StateBrand<ArcBrand, S>
where
    S: Send + Sync + 'static,
    for<'a, A: Send + Sync + 'a> <ArcBrand as RefCountedPointer>::Of<
        'a,
        dyn 'a + Fn(S) -> A,
    >: Send + Sync,
{
    ...
}
```

The `for<'a, A>` HRTB-over-types is unsupported on stable
Rust. Same wall as Phase 2 step 9d / 9g / 9i. Unimplementable.

**(c) Parallel `SendStateBrand<P, S>` separate from
`StateBrand<P, S>`.** Define a second brand whose `Of<'a, A>`
is `SendState<'a, P, S, A>` with the inner pointer projection
typed via
[`SendRefCountedPointer::Of`](../../../fp-library/src/classes/ref_counted_pointer.rs)
(which carries `T: Send + Sync` in its bound). The Arc smart
constructors use `SendStateBrand` in the row; non-Arc
constructors use `StateBrand`. Two state types, two row-brand
entries.

Rejected because:

- Doubles the type surface; users with mixed
  single-thread / thread-safe code in the same program face
  two state types they must convert between.
- The current `StateBrand<P, S>` parameterization was chosen
  specifically to give one State type across all six
  wrappers; splitting it would partially undo that design.

**(d) Use `SendRefCountedPointer::Of` directly in `State`'s
representation.** Bake `+ Send + Sync` into the `dyn Fn(...)`
trait object in the State enum's variants. Rejected because:

- Forces RcBrand single-thread users to provide
  `Send + Sync` closures for State, which **breaks the
  canonical `Rc<RefCell<S>>` capture pattern** for state
  threading (`Rc` is `!Send`).
- The workaround is parallel `ToDynCloneFn::new_send`
  machinery, which collapses into option (c) with extra
  steps.

**(e) Defer-and-document.** Don't ship 6a.4 / 6a.6; document
the closure-capture state pattern (the same pattern F1D
codified for `interpret`) as the canonical way to thread
state through `ArcRun` / `ArcRunExplicit`.

Rejected because:

- The locked-in design from the
  [2026-05-03 wrapper-parameterization resolution](#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization)
  signaled "six variants per effect" intent; deferring would
  partially unfulfill that lock-in.
- The implementation cost of (b) is small (~80 lines per
  wrapper plus where-clause noise), so deferring buys little
  while delaying a complete State surface.

### Implementation phasing

Under (b):

- `state.rs` keeps the `Functor` impl on `StateBrand<P, S>`;
  no `SendFunctor` impl is added.
- `ArcRun::get/put` and `ArcRunExplicit::get/put` add
  per-method bounds:
  - `<ArcBrand as RefCountedPointer>::Of<'_, dyn 'a + Fn(S) -> A>: Send + Sync`
  - `<ArcBrand as RefCountedPointer>::Of<'_, dyn 'a + Fn(()) -> A>: Send + Sync`
  - `S: Send + Sync` (already required for `ArcCoyoneda::lift`).
- `ArcRun::interpret_with::<StateBrand<ArcBrand, S>>` users
  get these bounds propagated into their where-clauses.
  Documented as a known cost in deviations.md and the `Arc
family` per-wrapper notes.

### Cross-references

- [Per-`A` HRTB-over-types blocks brand-level type-class
  delegation](prompt.md#per-a-hrtb-over-types-blocks-brand-level-type-class-delegation):
  the broader pattern this blocker instantiates.
- [Phase 2 step 9d resolution](#resolved-2026-04-28-implementation-expansion-step-9-sendfunctor-cascade-prerequisites-for-arc-family):
  the per-method workaround precedent.
- [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md):
  the project-wide table of HRTB-over-types-blocked
  brand-level cascades.
- [Phase 3 step 5 wrapper-parameterization
  resolution](#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization):
  the locked-in "six variants per effect" design that this
  resolution implements for the Arc family.

## Resolved (2026-05-03): Adversarial review reversals (delete `run_accum`, ship `interpret_with_rec`, parameterise `interpret_with` over `RefCountedPointer`)

An adversarial review of the WIP effects implementation
(commissioned 2026-05-03;
[review_effects_rs.md](review/0_first_order_effects_implementation/review_effects_rs.md))
filed 5 fundamental, 8 major, and 9 minor findings. The
remediation analysis
([remediation_proposals.md](review/0_first_order_effects_implementation/remediation_proposals.md))
recommended three changes that overturn prior locked-in
decisions in this file. This entry ratifies those three
reversals and pre-records the remaining recommendations as a
Phase 3 cleanup step.

### Reversal 1: delete `run_accum` and `run_accum_rec` entirely (review F1D)

**Prior decision:**
[Q3 (2026-05-02): closure-capture state threading with `init`
parameter](#resolved-2026-05-02-phase-3-step-4-interpreter-design-handler-shape-dispatch-trait-reuse-state-threading)
locked the `run_accum` family at "accept `init`, thread state
via user-side closure captures, return `A`". The implementation
honours that lock by writing `let _ = init; self.interpret(handlers)`
across all six wrappers.

**Review finding:** F1 (fundamental). With state threading
delegated to user-side captures, the `init` parameter is
vestigial; the function body is byte-equivalent to `interpret`'s.
The signature accepts `init` but does nothing with it. The
function name advertises state threading the implementation
cannot deliver in the mono-in-A return-type encoding.

**Reversal:** Delete `run_accum` and `run_accum_rec` from all six
wrappers. Document the closure-capture state pattern on
`interpret`'s
rustdoc with the keyword "state" so rustdoc-search picks it up.
The closure-capture state pattern itself is unchanged; only the
vestigial second method name is removed.

**Why the prior reasoning no longer holds:** The Q3 lock-in
recorded "the latter is documented as the convention for
state-threading uses". That role was a documentation hook, not a
semantic distinction. The same hook lives equally well as a
rustdoc paragraph on `interpret`. The PureScript Run parity
argument (literal `runAccum` naming) is a convention rather than
a constraint, and the project has departed from PureScript naming
elsewhere when the Rust shape diverges (e.g., `im_do!` for
inherent monadic do, no PureScript analogue).

**Forward compatibility:** When StateT lands in Phase 6+ per
[plan.md decisions row 1207](plan.md#L1207), the natural entry
point is `interpret_rec::<StateT<S, IdBrand>>`, not a re-purposed
`run_accum` slot. Keeping the slot would invite future confusion;
deleting it forecloses that.

### Reversal 2: ship `interpret_with_rec::<MBrand, EBrand>` (review M2A)

**Prior decision:**
[Decision 4 (2026-04-29): Phase 6+ deferred entry for
`interpret_with<M: Monad>`](#resolved-2026-04-29-phase-3-step-23-interpreter-family-shape)
deferred the pipeline-plus-MonadRec combination on the grounds of
"no current user demand" and "axis 3 alternative branch".

**Review finding:** M2 (major). `interpret_with`'s recursion is
host-stack-frame per peeled layer
(`run.rs:1032-1054`),
so users with deep eager-recursing effect chains who also need
row narrowing have no stack-safe option. The current escape
hatch (flatten into `interpret_rec`) forfeits the row-narrowing
benefit; the two shapes are not interchangeable.

**Reversal:** Ship the pipeline-plus-MonadRec combination as
Phase 3 step 5, sitting between the simple/pipeline/rec
interpreter families (steps 2-4) and the standard first-order
effects step (step 6). Per-wrapper inherent method
`interpret_with_rec::<MBrand, EBrand, Idx, RMinusE>` returning
`M::Of<'_, Run<RMinusE, CNilBrand, A>>`, internally driven by
[`tail_rec_m`](../../../fp-library/src/classes/monad_rec.rs).
Closes the orthogonality grid: simple, pipeline, MonadRec,
pipeline+MonadRec. The renumbering shifts the in-flight
"standard first-order effects" step from 5 to 6 and consequent
sub-step labels from 5a to 6a (per the project's preference for
clean numerical ordering over append-only step preservation).

**Why the prior reasoning no longer holds:** The "no current
user demand" reasoning was driven by the absence of standard
first-order effects (which only Phase 3 step 5 provides). With
Phase 3 step 5 in progress and the standard effects (`State`,
`Reader`, `Except`, `Writer`, `Choose`) about to land, the
demand-floor is no longer "no users"; it is "every user with a
deep program who wants pipeline narrowing". The original
deferral was precautionary; the review made the trade-off
visible.

### Reversal 3: parameterise `interpret_with` over `P: RefCountedPointer` (review M3C)

**Prior decision:** Implicit. The current `interpret_with`
implementation
(`run.rs:1003-1063`)
requires the handler closure to be `Fn + Clone + 'static` (plus
`Send + Sync` on Arc wrappers); the per-recursion clone is the
mechanism for sharing the handler across sub-program narrowings.
No prior resolution covered this; the choice landed in the
[Phase 3 step 3 deviations entry](deviations.md#L2059-L2061)
without an alternative survey.

**Review finding:** M3 (major). The `Clone` bound rules out
handler closures that capture unique resources (e.g., a
`BufWriter` acquired in scope). Users hit the bound at
`interpret_with` call sites and must wrap their captures in
`Rc<RefCell<_>>` themselves.

**Reversal:** Parameterise `interpret_with` over a pointer brand
`P: RefCountedPointer` (the existing trait at
[`ref_counted_pointer.rs`](../../../fp-library/src/classes/ref_counted_pointer.rs)).
Wrap the handler in `P::Of<F>` once at entry; clone the pointer
(cheap refcount bump) on each recursion. The four non-Arc
wrappers thread `RcBrand`; the two Arc wrappers thread
`ArcBrand`. The wrapper-level public method fixes `P` so users
see no extra parameter at the call site. Drops the user-facing
`Clone` bound; the bound becomes `Fn + 'static` (plus
`Send + Sync` on Arc wrappers).

**Why the prior reasoning no longer holds:** The implicit choice
predated the
[`RefCountedPointer`](../../../fp-library/src/classes/ref_counted_pointer.rs)
trait's load-bearing role in
[Phase 3 step 5a's State effect parameterisation](#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization),
which set the convention "abstract per-effect machinery over
`P: RefCountedPointer` so one definition serves both refcount
families". `interpret_with` is currently the only Phase 3
machinery that does not follow that convention; aligning it
removes a per-wrapper hard-code and a user-facing bound in one
move.

### Locked-in resolution set: F1D + F3A + M3C cleanup, then M2A

The three reversals fold into the plan's Phase 3 step list
through in-place revisions to existing steps (F1D, F3A, M3C
become "what step 2/3/4 say going forward") plus one new step
(M2A becomes step 5). Land order:

1. **Reversal cleanup** (small): F1D + F3A + M3C land together
   as one commit. F1D deletes `run_accum` / `run_accum_rec`
   from steps 2 and 4 (12 method signatures + 12 doctests).
   F3A tightens the `S` bound to `CNilBrand` on the
   `interpret`, `interpret_with`, and `interpret_rec` families
   (18 wrapper-method bodies; removes the
   `clippy::unreachable`-suppressed panic in the
   `Node::Scoped(_)` arms). **Phase 4 addendum (2026-05-05):**
   the `S = CNilBrand` tightening was correct for Phase 3
   closure but assumed Phase 4 would specify a parallel
   dispatch mechanism. Per [decisions.md section 4.5's
   "Dispatcher trait shape and substrate-level interpose
   primitive" sub-decision](decisions.md), Phase 4 lifts the
   `S = CNilBrand` bound on the interpret family and routes
   `Node::Scoped` through the new `DispatchScopedHandlers`
   trait; the F3A invariant is therefore Phase-3-only. M3C parameterises step 3's
   `interpret_with` over `P: RefCountedPointer` (drops the
   user-facing `Clone` bound; pairs naturally with [m9
   interpreter dispatch impl deduplication](review/0_first_order_effects_implementation/remediation_proposals.md#minor-findings)).
   Lands before resuming step 6a.3 (next pending sub-step of
   the standard first-order effects work) so step 6 does not
   inherit the issues.
2. **M2A: new step 5** (medium): six new method bodies
   `interpret_with_rec` plus integration tests. Sequence after
   step 6 completes so the standard first-order effects can
   drive the integration tests.

### Remaining review recommendations: tracked as Phase 3 step 9

The review's other recommendations
([remediation_proposals.md](review/0_first_order_effects_implementation/remediation_proposals.md))
are non-reversals and do not require ratification here. They
bundle into the new Phase 3 step 9 (review-remediation
documentation pass): F2A, F4A, F5A, M4 audit, M6A
async-via-`spawn_blocking` doc, M7A bind/handler asymmetry
note, and all minor m1-m9 items. Lands as one commit before
public release. The
[`SendFunctorAt`](review/0_first_order_effects_implementation/remediation_proposals.md#m5-sendfunctor-for-statebrand-is-deferred-multi-thread-state-is-unimplemented)
spike on State for the Arc family is a sub-task of step 6
(standard first-order effects), not step 9, since it gates the
ArcRun State Success criterion.

### Cross-references

- [`review_effects_rs.md`](review/0_first_order_effects_implementation/review_effects_rs.md): the
  adversarial review report ($9fd2bb8$) that surfaced the
  findings.
- [`remediation_proposals.md`](review/0_first_order_effects_implementation/remediation_proposals.md):
  the per-finding options + recommendations ($1bfb9f0$) that
  proposed these reversals.
- [Q3 (2026-05-02)](#resolved-2026-05-02-phase-3-step-4-interpreter-design-handler-shape-dispatch-trait-reuse-state-threading):
  the closure-capture-state lock-in that F1D updates.
- [Decision 4 (2026-04-29)](#resolved-2026-04-29-phase-3-step-23-interpreter-family-shape):
  the Phase 6+ deferral that M2A overturns.
- [Phase 3 step 5 (2026-05-03)](#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization):
  the
  [`RefCountedPointer`](../../../fp-library/src/classes/ref_counted_pointer.rs)
  parameterisation convention that M3C aligns with.
- [Phase 3 step 3 deviations
  entry](deviations.md#L2059-L2061): the implicit `Clone`-bound
  decision that M3C overturns.

## Resolved (2026-05-03): Phase 3 step 5 smart-constructor wrapper parameterization

Phase 3 step 4 shipped as `bd540d5` + `fafcfde`. Step 5
(standard first-order effect types and smart constructors:
`State<S>`, `Reader<E>`, `Except<E>`, `Writer<W>`, `Choose`)
is the next work. Five sub-decisions are entangled with the
top-level wrapper-parameterization question; all five shape
the public API and the Phase 3 step 6 `define_effect!` macro
emit shape, and were answered before implementation begins.

### Background

PureScript Run ships `ask :: Run (READER e r) e`, `get :: Run (STATE s r) s`,
etc. , single entry-points, no wrapper choice. PureScript has
one `Run` type, so the question doesn't arise.

fp-library has six Run wrappers
(`Run`,
`RcRun`,
`ArcRun`,
`RunExplicit`,
`RcRunExplicit`,
`ArcRunExplicit`)
because Rust requires substrate choices around continuation
function-pointer kind (`Box<dyn FnOnce>` vs `Rc<dyn Fn>` vs
`Arc<dyn Fn + Send + Sync>`), `'static` vs `'a` payload, and
type-erasure vs concrete-recursive-enum representation. The
[Phase 2 step 9h per-wrapper Coyoneda variant pairing
rule](plan.md#earlier-completed-steps-commit-log) locks each
wrapper to one Coyoneda variant. So smart constructors that
produce `Wrapper<R, S, A>` cannot be wrapper-polymorphic
without abstracting over the substrate cascade.

A second consideration: effects must compose. If a user wants
both `Choose` (multi-shot) and `State` (any wrapper) in the
same program, they must pick a multi-shot wrapper for the whole
program. So `State` has to be available in _whichever wrapper
supports `Choose`_ (i.e., `RcRun` / `ArcRun`), not just on a
canonical default like `Run`. This rules out a simple "ship each
effect on one canonical wrapper" approach.

A third consideration: Phase 3 step 6's `define_effect!` macro
generates effect types + smart constructors at user request.
Whatever shape step 5 picks for the hand-rolled standard
effects, step 6's macro must emit the same shape. So the
decision cascades.

### The five questions and their resolutions

**Q1 (top-level wrapper parameterization shape, confirmed (b) six variants per effect).**
Each effect ships as inherent methods on each wrapper (or in
per-wrapper modules), e.g., `Run::ask`, `RcRun::ask`,
`ArcRun::ask`, `RunExplicit::ask`, `RcRunExplicit::ask`,
`ArcRunExplicit::ask`. Verbose at face value, but the
`define_effect!` macro (Phase 3 step 6) mechanically generates
the six variants from one user declaration, hiding the
verbosity at user-code level.

Alternatives considered:

- **(a) Wrapper-parameterized.** One `ask::<W, R, ..., E, Idx>(...)`
  generic over `W: RunWrapper` (a new trait abstracting the
  lift behaviour). All wrappers implement the trait. Pros: one
  entry-point per effect; symmetric across wrappers; mirrors
  PureScript's single-Run shape most closely. Cons: requires
  defining a `RunWrapper` trait that captures the substrate
  cascade (Coyoneda variant choice, `lift` signature,
  where-clause cascade); every effect type must be parameterised
  by the wrapper's function-pointer kind via
  [`FnBrand`](../../../fp-library/src/types/fn_brand.rs);
  user-facing turbofish at call sites in non-inferrable cases;
  trait machinery is non-trivial because the six wrappers have
  meaningfully different bound cascades (`A: Clone` for Rc
  family; `A: Send + Sync` for Arc family; etc.).
- **(c) Canonical wrapper per effect family.** Pick one per
  effect: e.g., State/Reader/Except/Writer ship on `Run`;
  Choose ships on `RcRun`. Users convert between wrappers via
  existing `From` impls when needed. Pros: minimal API surface;
  one entry-point per effect. Cons: broken by the composition
  argument: if a user wants Choose + State, they need
  `RcRun`-shaped State, not `Run`-shaped. So the user would
  have to convert `Run::get()` into an `RcRun` program, but the
  row brand is fixed at the wrapper, so there's no clean
  conversion. (c) requires either ad-hoc conversion machinery
  or just falls back to (b) under the hood for the multi-shot
  case. Not a coherent design as stated.

The principled argument: the precedent of every other Phase 3
step shipping six wrapper-specific inherent methods is strong
(step 2's `interpret`/`run`/`run_accum`, step 3's
`interpret_with`/`extract`, step 4's
`interpret_rec`/`run_rec`/`run_accum_rec`). (a) would be the
first time we introduce a wrapper-abstracting trait; the cost
(substrate-cascade abstraction; complex bounds; turbofish
proliferation) outweighs the cosmetic benefit of one entry-
point. (c) is fundamentally incoherent because of effect
composition.

**Q2 (effect-type representation, confirmed (a) per-effect Functor instance).**
Each effect type holds its continuation directly:

```rust
enum State<FnP: FnBrand, S, A> {
    Get(FnP::Of<dyn FnOnce(S) -> A>),
    Put(S, FnP::Of<dyn FnOnce(()) -> A>),
}
```

Requires per-effect `Functor` impl (and `SendFunctor` for the
Arc family). The function-pointer kind is wrapper-specific
(`Box` for Run/RunExplicit; `Rc` for Rc family; `Arc` for Arc
family) and threaded via `FnBrand` (see Q3).

Alternatives considered:

- **(b) Continuation-free effect type, Coyoneda holds the
  continuation.** Effect types carry only the operation
  discriminator: `enum State<S, A> { Get(PhantomData<A>), Put(S, PhantomData<A>) }`.
  Coyoneda's `lift(fa)` stores `fa: F::Of<'_, A>` plus an
  identity function. After `map`, Coyoneda accumulates
  continuations. The catch: in (b), the "underlying value"
  inside Coyoneda is not actually `A`; it's `S` for Get and
  `()` for Put. Coyoneda's type signature is `Coyoneda<F, A>`
  where `F::Of<A>` is the inner value. With (b),
  `F::Of<A> = State<S, A>` which is just a tag , but Coyoneda's
  stored `f: B -> A` would then be `B = A`, so the function is
  identity, and the handler must manually invoke its own
  continuation by inspecting the variant. Workable but loses
  the Functor abstraction.

The principled argument: matches PureScript's structure
directly. The `Choose<A> = Choose Boolean a` shape inherently
needs the continuation in the type because the handler runs
both branches; (b) would force handlers to track the
continuation outside the effect type, breaking the abstraction
asymmetrically across effects.

**Q3 (function-pointer-kind threading, confirmed (a-1) `FnBrand`-parameterised).**
Single effect type per effect, parameterised by `FnBrand`:

```rust
enum State<FnP: FnBrand, S, A> { ... }
```

Smart constructors thread `FnP` per-wrapper. Matches existing
`FnBrand`-based code (e.g.,
[`RcFree`/`ArcFree`](../../../fp-library/src/types/rc_free.rs)
already use `FnBrand`-shaped continuations).

Alternatives considered:

- **(a-2) Per-wrapper effect types.** `RunState<S, A>`,
  `RcState<S, A>`, `ArcState<S, A>`, etc., each hard-coded to
  its substrate's function-pointer. ~30 named effect types per
  effect = 30+ types per effect. Massive duplication; rejected.
- **(a-3) Single effect type with `Box<dyn FnOnce>` always;
  convert at lift site.** The effect type stores `Box`; for
  Rc/Arc lifts, convert to `Rc`/`Arc` wrapping at lift time.
  Requires conversion machinery; loses the `FnBrand` precedent.
  Rejected as ad-hoc.

**Q4 (Choose's wrapper coverage, confirmed (ii) all four multi-shot wrappers).**
Choose is intrinsically multi-shot (handler runs both branches
of the choice and combines results). Continuation must be
cloneable. So Choose ships on `RcRun`, `RcRunExplicit`,
`ArcRun`, `ArcRunExplicit`; not on `Run` or `RunExplicit`.

Plan text said "Choose (multi-shot, `RcRun`-only)" , overly
narrow. Plan text needs updating to reflect all four
multi-shot wrappers.

**Q5 (row-brand notation, confirmed (b) `effects!` only initially).**
Users compose row brands via the
[`effects!`](../../../fp-macros/src/effects/effects_macro.rs)
macro: `effects!(ReaderBrand<FnP, E>, StateBrand<FnP, S>)`. No
per-effect type aliases (e.g., `type ReaderRow<E, R> = ...`)
ship in step 5; can be added later if users complain.

Alternatives considered:

- **(a) Ship row-alias type aliases per effect.** E.g.,
  `type ReaderRow<'a, E, R> = CoproductBrand<CoyonedaBrand<ReaderBrand<'a, E>>, R>;`
  Multiplies the API surface and locks in naming choices.
- **(c) Both.** Ship aliases for ergonomics; users can also
  use `effects!`. Reasonable but adds maintenance overhead;
  defer until users complain.

The principled argument: the `effects!` macro is the canonical
composition path; per-effect aliases add API surface and
naming-debate exposure for marginal gain.

### Locked-in resolution set: (1.b) + (2.a) + (3.a-1) + (4.ii) + (5.b)

The combination ships step 5 as wrapper-symmetric inherent-
method variants on per-wrapper modules (precedent-matching),
with `FnBrand`-parameterised effect types (substrate-agnostic),
and row-brand composition via the existing `effects!` macro.

### Sub-decisions summary

| #   | Question                        | Resolution                                                                                        |
| --- | ------------------------------- | ------------------------------------------------------------------------------------------------- |
| 1   | Wrapper parameterization        | (b) six variants per effect; precedent-matching; `define_effect!` macro hides verbosity user-side |
| 2   | Effect type representation      | (a) per-effect Functor; matches PureScript; needed for `Choose`'s multi-branch continuation       |
| 3   | Function-pointer-kind threading | (a-1) `FnBrand`-parameterised; matches `RcFree`/`ArcFree`'s precedent; single effect type per     |
| 4   | Choose's wrapper coverage       | (ii) all four multi-shot wrappers; plan text updated                                              |
| 5   | Row-brand notation              | (b) `effects!` only; per-effect aliases deferred until user demand                                |

### Implementation phasing under the locked-in set

- **Step 5 ships 5 effect types** (`State<FnP, S, A>`,
  `Reader<FnP, E, A>`, `Except<E, A>`, `Writer<FnP, W, A>`,
  `Choose<FnP, A>`) parameterised by `FnBrand` where the
  effect carries continuations.
- **Step 5 ships ~34 smart constructors:**
  - State: `get` and `put` on each wrapper. 2 \* 6 = 12.
  - Reader: `ask` on each wrapper. 1 \* 6 = 6.
  - Except: `throw` on each wrapper. 1 \* 6 = 6.
  - Writer: `tell` on each wrapper. 1 \* 6 = 6.
  - Choose: `choose` on each multi-shot wrapper. 1 \* 4 = 4.
  - Total: ~34 named smart constructors, distributed across
    six per-wrapper modules.
- **Step 5 may be split into sub-steps** per the
  [implementation protocol](plan.md#implementation-protocol)'s
  oversized-step rule; one effect per sub-step is the natural
  cut (5a State, 5b Reader, 5c Except, 5d Writer, 5e Choose).
  Surface the split decision to the user before starting.
- **Step 6's `define_effect!` macro** mechanically generates
  the six per-wrapper variants from a single user declaration:
  ```rust
  define_effect! {
      Reader<E> {
          fn ask() -> E,
      }
  }
  ```
  expands to per-effect type + 6 per-wrapper smart-constructor
  bodies.
- **`Choose` opts out of the single-shot wrappers** at the
  `define_effect!` macro level via a `multi_shot` attribute or
  similar. Step 6's design needs to account for this.

### Cross-references

- [Phase 2 step 9h pairing rule](plan.md#earlier-completed-steps-commit-log):
  locks each wrapper to one Coyoneda variant; root cause of
  wrapper proliferation.
- [`FnBrand`](../../../fp-library/src/types/fn_brand.rs):
  precedent for substrate-abstracted function-pointer kinds.
- [decisions.md](decisions.md) section 4.6: covers natural-
  transformation handler shape but does not pre-decide the
  wrapper-parameterization question.
- The lessons-learned section in [prompt.md](prompt.md)'s
  "Coyoneda variant pairing rule" subsection explicitly flags
  this question.

## Resolved (2026-05-02): Phase 3 step 4 interpreter design (handler shape, dispatch-trait reuse, state threading)

Phase 3 step 3 shipped as `ff84f20`. Step 4 (the
`MonadRec`-target interpreter family `interpret_rec` / `run_rec`
/ `run_accum_rec`) is the next work. Three load-bearing design
questions surfaced during step-4 scoping; all three shape the
public API and the dispatch-trait reuse and were answered
before implementation began.

### The three questions and their resolutions

**Q1 (handler input/output shape, confirmed (A) mirror PureScript).**
Handler is
`Fn(<EBrand as Kind>::Of<'_, M::Of<'_, Run<R, S, A>>>) -> M::Of<'_, Run<R, S, A>>`.
The interpreter does
`<R as Functor>::map(M::pure, peel_layer)` to lift the peeled
layer's `Run<R, S, A>`-continuations to
`M::Of<Run<R, S, A>>`-continuations before dispatch. Reuses
step 2's
`DispatchHandlers<'a, Layer, NextProgram>`
trait unchanged, instantiated with
`NextProgram = M::Of<Run<R, S, A>>`. Handlers can do real
`M::bind`-monadic work between effect dispatches (e.g., short-
circuit on `M = ResultBrand` via `Result::bind`).

Alternatives considered:

- **(B) Rust-flavoured simplification.** Handler input matches
  step 2's raw shape
  `Fn(<EBrand as Kind>::Of<'_, Run<R, S, A>>) -> M::Of<'_, Run<R, S, A>>`.
  Output is M-wrapped only. Requires a new
  `DispatchHandlersRec<'a, Layer, NextProgram, MBrand>` trait
  parallel to `DispatchHandlers`. The user-facing handler shape
  is simpler (no `M::Of<...>` ceremony in the input type), but
  the handler cannot perform M-monadic operations on the
  continuation before dispatch.
- **(C) Hybrid: M-wrapped input, raw Run output.** Handler is
  `Fn(<EBrand as Kind>::Of<'_, M::Of<'_, Run<R, S, A>>>) -> Run<R, S, A>`.
  Loses access to `M::bind` inside handlers, defeating most of
  the reason to ship a MonadRec-target form.

The principled argument: M-monadic work in handlers IS the
load-bearing capability that distinguishes step 4 from step 2.
(B) and (C) lose that capability for a small cosmetic
simplification. (A) preserves it and reuses the existing
dispatch trait without ceremony.

**Q2 (`DispatchHandlers::dispatch` `&mut self` vs `&self`, confirmed (A1) relax to `&self`).**
Step 4's body invokes dispatch inside a `tail_rec_m` step
function whose closure is `Fn` (not `FnMut`); the closure
captures the handler list and calls `dispatch` on each
iteration. With `&mut self`, the captured list cannot be
borrowed mutably across iterations from inside a `Fn`-bound
closure. The relaxation lands as the first commit in step 4
implementation.

Alternatives considered:

- **(A2) Clone handlers per iteration.**
  `HandlersCons<H, T>: Clone` already holds for `H, T: Clone`,
  which `Handler<E, F>: Clone` satisfies for `F: Clone`. The
  step closure clones the handler list each iteration. Per-
  iteration overhead proportional to chain depth (each clone
  is O(N) in handler-list length, so O(N x chain depth)
  total). Acceptable for short handler lists but unbounded.
- **(A3) Wrap in `RefCell` inside the step closure.** The step
  closure captures
  `RefCell::new(handlers)`; each iteration calls
  `handlers.borrow_mut().dispatch(layer)`. Hides the issue at
  the call site rather than fixing it; ergonomically bad and
  not a clean shape.

The principled argument: all current handler closures across
the codebase use interior mutability for state (e.g.,
`run_handle.rs`'s
`run_accum`-via-`Rc<RefCell>` tests use closures that Rust
infers as `Fn` because the mutation goes through
`RefCell::borrow_mut(&self)`). Step 2's
`Handler<E, F>`
carrier holds `F` opaquely; the `Fn` constraint is added at
the dispatch trait's impl bounds, not at the carrier. Relaxing
to `&self` matches actual usage and adds zero per-iteration
overhead. The risk (relaxation breaks an existing handler) is
bounded and easily verified via `just verify`. (A2) and (A3)
are workarounds for a constraint that doesn't actually bind.

**Q3 (state threading in `run_accum_rec`, confirmed (A) closure-capture).**
State threading uses
[`Rc<RefCell<S>>`](https://doc.rust-lang.org/std/cell/struct.RefCell.html)
or
[`Arc<Mutex<S>>`](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
closure captures at the user level, matching step 2's
`run_accum`
shape. The `init` parameter is moved into the user's chosen
state cell internally and is otherwise ignored. The only
difference between
`interpret_rec` and `run_accum_rec` is that the latter is
documented as the convention for state-threading uses; no
separate stateful trait machinery.

Alternatives considered:

- **(B) State-via-M (PureScript-mirroring).** Add a separate
  stateful trait or require `M = StateT<S, MInner>` for some
  inner monad. Mirrors PureScript more directly. Requires
  fp-library to ship `StateT` (currently absent) or a similar
  state-monad transformer. Doubles the trait machinery.
  Deferred to Phase 6+; revisit when the demand surfaces.
- **(C) Punt to Phase 6+.** Drop `run_accum_rec` from step 4
  entirely; document it as deferred. Loses per-method API
  parity with step 2.

The principled argument: parity with step 2 minimises
cognitive load and keeps the trait machinery thin. (B) is
genuinely useful for some `MonadRec` instances (especially
when the target M wants to maintain a state that's accessible
from inside `M::bind`-monadic handler code, not just from
between effect dispatches), but requires `StateT` which is a
separate Phase 6+ addition.

### Locked-in resolution set: (A) + (A1) + (A)

The combination ships step 4 with maximum ergonomic and
capability symmetry to step 2, mirrors PureScript Run as the
upstream design intent, and introduces no new trait ceremony.

### Implementation order under the locked-in set

1. (Q2 = A1.) Refactor
   `DispatchHandlers::dispatch`
   from `&mut self` to `&self` and confirm `Handler::F: Fn`
   bound suffices for the existing dispatch impls. Land as
   the first commit in step 4 (mechanical refactor; should
   not break any existing tests).
2. Add `interpret_rec` / `run_rec` / `run_accum_rec` per-
   wrapper inherent methods (parallel to step 2's
   `interpret`/`run`/`run_accum`). Each wrapper's body uses
   `tail_rec_m::<MBrand, _, _>(step_fn, self.into())`. Step
   function: peel current program, dispatch on `Node::First`
   via the (now `&self`)
   `DispatchHandlers`
   trait (with `NextProgram = M::Of<Run<R, S, A>>` after
   pre-dispatch lifting via
   `<R as Functor>::map(M::pure, peel_layer)`), fmap
   `M::Of<NextProgram>` to `M::Of<ControlFlow<Continue, Break>>`.
   ArcRun reuses
   `unwrap_first`
   for the HRTB-poisoning workaround. The
   `make_node_first`
   /
   `wrap_first_arc`
   helpers from step 3 are not needed for step 4 (no narrowed
   Run construction in scope; the rec form returns `M::Of<A>`
   directly).
3. Per-wrapper bounds cascade: `MBrand: MonadRec`; for the Arc
   wrappers, also `M::Of<'_, Run<...>>: Send + Sync` and the
   per-projection cascade.
4. Integration tests in
   `fp-library/tests/run_handle_rec.rs` covering each
   wrapper x several `M` choices (`ThunkBrand`, `OptionBrand`,
   `ResultBrand`); doctests on each method.
5. Update plan.md's `Current progress` (rolling-detail entry
   for step 4; demote step 1 to commit log per the rolling-
   detail trim window of ~3 narratives).
6. Append deviations.md entry for step 4.

### Cross-references

- [decisions.md](decisions.md) section 4.3 ("Ship both
  interpreter families"): the original commitment that frames
  step 4 as a `MonadRec`-target sibling of step 2.
- [`fp-library/src/classes/monad_rec.rs`](../../../fp-library/src/classes/monad_rec.rs):
  fp-library's `MonadRec` trait + `tail_rec_m` free function.
- `fp-library/src/types/effects/interpreter.rs`:
  `DispatchHandlers` trait that step 4 reuses (after Q2 = A1
  relaxation).
- `fp-library/src/types/effects/handlers.rs`:
  `Handler<E, F>` carrier; `F: Fn` constraint moves from impl
  bounds to the relaxed dispatch signature.
- Phase 3 step 3 commit `ff84f20` (pipeline row-narrowing +
  empty-row extract).
- Phase 3 step 2 commit `d5efe2a` (`interpret` / `run` /
  `run_accum` family) -- the API whose shape step 4 mirrors.
- [Resolved (2026-04-29): Phase 3 step 2/3 interpreter family shape](#resolved-2026-04-29-phase-3-step-23-interpreter-family-shape)
  -- the prior resolution that locked in step 4's role as the
  third orthogonal interpreter primitive (M-target via
  `tail_rec_m`).
- [PureScript Run](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
  source.
- [PureScript MonadRec](https://github.com/purescript/purescript-tailrec/blob/master/src/Control/Monad/Rec/Class.purs)
  source.

## Resolved (2026-04-29): Phase 3 step 2/3 interpreter family shape

Phase 3 step 2 (`d5efe2a`) shipped `interpret` / `run` /
`run_accum` on the six Run wrappers with the target monad
implicit (`M = self Run wrapper`, returns `A` directly via a
while-loop). [decisions.md](decisions.md) section 4.3 frames
both step 2 (`Monad m`) and step 3 (`MonadRec m`) as exposing
the target monad as a parameter, mirroring PureScript Run's
`run` / `runRec`. The blocker question: should step 2 be
reshaped to match decisions.md 4.3, and what shape should
step 3 take?

A subsequent close read of
[PureScript Run](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
plus [heftia](https://github.com/sayo-hs/heftia)'s
interpreter machinery widened the question into three
orthogonal axes: which interpreter functions ship (axis 1),
handler shape algebraic vs return-next-program (axis 2), and
rec/non-rec for the externally-targeted M family (axis 3).
The original blocker was just axis 3; the widened scope made
axes 1 and 2 explicit.

Five decisions ultimately gated the resolution. The full
analysis lives in plan.md across commits `9f9e07b` (initial
blocker), `8e59bb5` (widened-scope analysis), `05539af`
(per-decision approaches and trade-offs), `f3148b2` (clean
rewrite for readability), and `35ceeee` / `ccc66c9` /
`5757796` / `86a544f` / `855f85c` (per-decision lock-ins).

### The five decisions and their resolutions

**Decision 1 (axis 1 widening, confirmed (1.A) Full widen).**
Schedule `extract(self) -> A` (empty-row pure extract) and
`interpret_with::<EBrand>(handler) -> Run<R_minus_E, S, A>`
(single-effect row-narrowing pipeline) as a new Phase 3 step.
Pipeline + extract uniquely enables three capabilities that
neither step 2's all-handlers-at-once form nor the future
MonadRec form can provide:

1. **Partial interpretation.** Pipeline keeps the program in
   Run-land while peeling effects (returns
   `Run<R_minus_E, S, A>`). Step 2 returns `A`; the future
   MonadRec form returns `M::Of<A>` (extracted). Pipeline is
   the only shape that supports "interpret one effect, store
   the result, interpret the rest later".
2. **User-controlled handler ordering for non-commuting
   effects.** Combinations like `NonDet * Except` produce
   different semantics depending on which handler runs
   "outside" which. Pipeline lets users explicitly chain
   `.interpret_with::<Except>(...).interpret_with::<NonDet>(...)`.
3. **Compositional handler libraries.** Library authors can
   ship reusable handlers as
   `fn run_state<R, A>(...) -> Run<R_minus_State, S, A>`.
   Without pipeline, they can only ship handler closures
   meant for inclusion in a user-built `handlers!{}` block.

These three capabilities are real ecosystem needs, not just
ecosystem precedent. Heftia and PureScript ship pipeline as
their primary interpretation mode for these reasons; the
load-bearing argument is the capability set, not the
convention.

**Decision 2 (axis 3 rec/non-rec, confirmed (2.C) Asymmetric).**
Step 2 stays as-shipped (M = self Run, returns A). The
MonadRec step (renumbered to step 4) adds an
externally-targeted `<MBrand: MonadRec>` family alongside.
Three orthogonal cognitive models map to three primitives:

- Simple (step 2): M-free, "give me a value", no engagement
  with MonadRec abstraction.
- Pipeline (step 3): row-narrowing for compositional handler
  chains.
- MonadRec (step 4): external target (`Thunk` / `Option` /
  `Result` etc.) with stack-safety guarantees via
  `tail_rec_m`.

The key principled argument: step 2's M-free shape uniquely
enables value extraction without engaging MonadRec
abstraction. Its `while`-loop is structurally stack-safe by
construction (no `M::bind` or `M::tail_rec_m` in the body),
so there's no need for a `MonadRec` constraint. Under any
alternative , (2.A) symmetric Monad/MonadRec, (2.B) MonadRec
uniform, or (2.D) drop-simple-form , value extraction would
route through `M = IdentityBrand` with turbofish + `.0`
unwrap, forcing users to encounter MonadRec machinery they
don't conceptually need.

The original recommendation reasoning ("preserve `d5efe2a`'s
API") was partly conventional. The principled reasoning that
landed during the discussion is the M-free unique-value
framing.

**Decision 3 (Phase 3 step ordering, confirmed (3.A) Insert + renumber).**
Inserts the new pipeline step at position 3; renumbers
former steps 3-6 to 4-7. The principled reason: atomic
commits + linear readability is a software-engineering
practice with concrete benefits (bisectability, reviewability,
navigability). Reference-sweep cost is bounded (plan.md
"Implementation phasing" + "Current progress" only; existing
deviations.md entries don't need editing because their step
numbers don't change).

**Decision 4 (Phase 6+ deferred entries, confirmed (4.A) Defer all three).**
Three new Phase 6+ deferred-items entries in plan.md:

- `interpret_with<M: Monad>` (Monad-bound externally-targeted
  family, axis 3 alternative branch).
- `run_cont` / `run_accum_cont` family (axis 1
  continuation-passing handlers).
- `interpose` family (heftia hook-without-removing).
- Algebraic-shape FO handlers (axis 2).

The Phase 6+ pattern serves a real institutional-memory
purpose: each entry records what the item is, why it's
deferred, and a trigger for revisitation. Without entries,
deferred items get lost or re-litigated by future agents.

**Decision 5 (decisions.md update, confirmed (5.A) Keep frozen).**
The doc system has separate roles for separate kinds of
content: decisions.md (design-time frozen rationale), plan.md
"Key decisions" (implementation-time choices), resolutions.md
(blocker analyses), deviations.md (per-step deviations).
Editing decisions.md to record implementation-time choices
would merge two roles inappropriately. (5.B) "refine 4.3"
and (5.C) "add 4.7" both violate role separation.

### Rust constraints that shaped the analysis

The Rust port differs from PureScript in two structural ways
that affect the interpreter design:

1. **Step 2's loop is structurally stack-safe regardless of
   M.** PureScript Run's `run` body
   `loop = resume (\a -> loop =<< k a) pure` recurses through
   `m`'s bind, building m-bind frames in the host stack. The
   `runRec` sibling swaps for `tailRecM` to keep host stack
   constant. fp-library step 2's body is
   `while { match peel { Ok(a) => return a, Err(node) => prog = handlers.dispatch(layer) }}`
   , assignment-driven, no `m`, no `bind`, no `tail_rec_m`.
   The PureScript rec/non-rec distinction does not apply to
   step 2's shape; stack-safety is by construction.
2. **Bind-driven recursion with borrowed handler state is
   awkward in Rust.** PureScript's `loop =<< k a` recurses
   inside the bind continuation, which captures `loop`. In
   Rust, the closure passed to `M::bind` must be `Fn`
   (multi-callable for non-deterministic m), capture the
   handler list, and avoid mutably aliasing it. The
   workarounds are real but cheap (`Fn` closures are
   natural; `HandlersCons<H, T>` and `Handler<E, F>` already
   derive `Clone`; `'static` is mostly already required).
   Without the workarounds, only `tail_rec_m`-driven loops
   are tractable.

These constraints meant the original decisions.md 4.3
"ship both families" reasoning ("implementation cost is
mostly mechanical") was weaker in Rust than the
decision-author anticipated. (2.C) accepts this and ships
the rec form alongside step 2's M-free form rather than
mirroring PureScript's bind-driven `run` shape.

### Heftia row architecture clarification

Decisions.md section 4.5 commits to "heftia's dual-row
architecture" for scoped effects. Reading
[heftia source](https://github.com/sayo-hs/heftia) directly:
heftia uses **one** effect list (`es`) where each element
has a `KnownOrder` (FO or HO), with a `FOEs es` constraint
when all members must be first-order. fp-library takes the
**idea** (separate FO vs HO dispatch) and ships a
value-level dual row (`Run<R, S, A>` with
`Node<R, S> = First | Scoped`). So fp-library's
"heftia-inspired" framing is inspiration, not direct port;
the row encoding diverges. This gives latitude on the
interpreter API: we don't need heftia's mixed-order
constraint machinery (`FOEs`, `KnownOrder`); we just
dispatch on `Node::First` vs `Node::Scoped`.

### What ships under the resolution

Phase 3 re-scheduled per (3.A):

1. Step 1 (`82dd7bb`): `handlers!{...}` macro + `nt()` builder.
2. Step 2 (`d5efe2a`): `interpret` / `run` / `run_accum`
   M-free family.
3. Step 3 (NEW, pending): pipeline `interpret_with::<E>` +
   `extract` per wrapper. New `DispatchOneHandler` trait
   variant.
4. Step 4 (was step 3, pending): MonadRec-target
   `interpret_rec` / `run_rec` / `run_accum_rec`.
5. Step 5 (was step 4): standard first-order effects.
6. Step 6 (was step 5): `define_effect!` macro.
7. Step 7 (was step 6): `compile_fail` UI tests.

Phase 6+ deferred-items section gains four new entries
(`interpret_with<M: Monad>`, `run_cont` / `run_accum_cont`,
`interpose` family, algebraic-shape FO handlers) with full
trigger conditions.

decisions.md stays frozen per (5.A); per-step deviations.md
entries land when each Phase 3 step ships, cross-referencing
this resolutions.md entry.

### Cross-references

- [decisions.md](decisions.md) section 4.3 ("Ship both
  interpreter families"): the original commitment that
  shaped the question.
- [decisions.md](decisions.md) section 4.5: heftia-inspired
  dual-row scoped effects (where the heftia inspiration
  framing originated).
- [`fp-library/src/types/free.rs`](../../../fp-library/src/types/free.rs)'s
  `Free::fold_free<G: MonadRec>`: existing Rust precedent
  for the externally-targeted MonadRec pattern.
- [`fp-library/src/classes/natural_transformation.rs`](../../../fp-library/src/classes/natural_transformation.rs):
  rank-2 polymorphic abstraction; complementary to the
  handler-list path. Future `interpret_nt` deferred entry
  references this.
- [`fp-library/src/classes/monad_rec.rs`](../../../fp-library/src/classes/monad_rec.rs):
  fp-library's MonadRec, mirror of PureScript's.
- Phase 3 step 1 commit `82dd7bb` (handlers! macro).
- Phase 3 step 2 commit `d5efe2a` (interpret family) , the
  API `(2.C)` preserves and `(2.A)` / `(2.B)` / `(2.D)` would
  have broken.
- [PureScript Run](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
  source.
- [PureScript MonadRec](https://github.com/purescript/purescript-tailrec/blob/master/src/Control/Monad/Rec/Class.purs)
  source.
- [Heftia](https://github.com/sayo-hs/heftia) interpreter
  machinery.

## Resolved (2026-04-28): Phase 2 step 9 scope is under-specified

Phase 2 step 9's plan-text originally read in full:

> 9. Coyoneda-wrapping smart constructors (`lift_f` analogues for each effect type).

Two plausible interpretations of "smart constructors for each
effect type" existed, and they differed substantially in scope:
the **generic combinator** interpretation (one helper that
takes any effect value plus a `Member` witness, lifts it
through Coyoneda, injects into the row, wraps in `Node::First`,
and `send`s) and the **per-effect helpers** interpretation
(concrete `State<S>` / `Reader<E>` / `Except<E>` / `Writer<W>` /
`Choose` types plus `ask`, `get`, `put`, `modify`, `tell`,
`throw`).

Reading the rest of the plan, the generic-combinator interpretation
was the intended one: Phase 3 step 4 explicitly schedules
_"Standard first-order effect types and their smart
constructors: `State<S>`, `Reader<E>`, `Except<E>`, `Writer<W>`,
`Choose`"_ as a separate Phase 3 deliverable, so doing per-effect
work in Phase 2 step 9 would duplicate it.
[decisions.md](decisions.md) section 6 likewise describes
per-effect smart constructors as **thin wrappers over** the
`inj + liftF`/`send` infrastructure, implying the row-aware
lift combinator is prerequisite infrastructure that ships first
(which is what step 9 lands).

Three sub-questions remained open under the generic-combinator
interpretation:

- **Free function vs per-wrapper inherent method.** The
  established Phase 2 pattern (steps 5, 7a-c) puts user-facing
  Run-program operations on the wrappers as inherent methods
  (`Run::pure`, `RcRun::bind`, etc.), but the combinator's key
  argument is the effect value, not `self`, so a free function
  would also be natural.
- **Exact signature.** The `Member` bounds, the `Coyoneda`
  decode closure (does the user supply it?), the alignment across
  the six wrappers (whose bounds differ: Erased Rc family wants
  `A: 'static`, Explicit family wants `A: 'a`, ArcRunExplicit
  wants `A: 'a + Send + Sync`), and whether `Idx` is turbofished
  or inferred.
- **HRTB-poisoning under `ArcFree`.** Per the prior 2026-04-27
  resolution, constructing a `Node`-projection literal inside
  an HRTB-bearing scope (which `ArcFree`'s struct propagates
  into every `ArcRun`-method context) fails GAT normalization.
  `ArcRun`'s row-aware lift combinator would need to thread
  the same workaround.

- **Naming: `lift` vs `lift_f`.** The combinator does the full
  chain (`Coyoneda::lift` + Member inject + `Node::First` +
  `*Run::send`); functionally it is the direct analog of
  PureScript Run's
  [`lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
  (signature `Proxy sym -> f a -> Run r a`, body
  `Run <<< liftF <<< inj p`), not of
  [`Free.liftF`](https://github.com/purescript/purescript-free/blob/main/src/Control/Monad/Free.purs).
  PureScript explicitly distinguishes the two: `Free.liftF`
  is the Free-only operation; `Run.lift` is the row-aware
  Run-level operation that consumes a row label and runs the
  full inject + liftF chain. fp-library already mirrors the
  Free side as
  [`Free::lift_f`](../../../fp-library/src/types/free.rs)
  (snake_case translation of `liftF`); the Run-level operation
  takes the bare name `lift`. Phase 3's per-effect smart
  constructors (`ask = lift ReaderBrand Reader::Ask`, etc.)
  read consistently with PureScript's
  `liftEffect = lift (Proxy :: "effect")` pattern under this
  naming.

### Resolution

**Generic combinator interpretation, named `lift` (matching
PureScript Run's
[`lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)).
Inherent associated function on each of the six Run wrappers,
mirroring `*Run::send`'s shape. Take the raw effect (an
`EBrand::Of<'a, A>` value) and do the full chain (`Coyoneda::lift`
-> row inject -> `Node::First` -> `*Run::send`) inside the body.
Type-infer `Idx` at call sites where the row is unambiguous;
turbofish only when duplicate effect types make `Idx` ambiguous.
Try the simple inline body first; fall back to a free
`lift_node<R, S, EBrand, Idx, A>(effect)` helper for `ArcRun::lift`
if HRTB-poisoning recurs.**

The signature for `Run` is:

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

The `Kind!()` macro can't appear in trait-bound position (per
its doc-comment limitation: invalid in supertrait bounds, type
aliases, and trait aliases on stable Rust); the bound on
`EBrand` uses the generated hash name `Kind_cdc7cd43dac7585f`
directly. The hash is deterministic from the signature
`type Of<'a, T: 'a>: 'a;` and is in scope via fp-library's
existing `kinds::*` re-export.

Per-wrapper deltas (the body shape is identical; only the bounds
change):

| Wrapper          | `'a`         | Extra `A` bound            | Extra row/node bound                                          |
| :--------------- | :----------- | :------------------------- | :------------------------------------------------------------ |
| `Run`            | `'static`    | `A: 'static`               | (none)                                                        |
| `RcRun`          | `'static`    | `A: 'static`               | the `Apply<...>: Clone` bound `RcRun::send` carries           |
| `ArcRun`         | `'static`    | `A: Send + Sync + 'static` | `NodeBrand<R, S>: Functor` plus the `Apply<...>: Clone` bound |
| `RunExplicit`    | `'a` (param) | `A: 'a`                    | (none)                                                        |
| `RcRunExplicit`  | `'a` (param) | `A: 'a`                    | (none)                                                        |
| `ArcRunExplicit` | `'a` (param) | `A: 'a + Send + Sync`      | (none)                                                        |

The Coyoneda decode closure is implicit: `Coyoneda::lift` defaults
to the trivial decode, which is what every smart-constructor case
wants. Users who want a non-trivial decode construct `Coyoneda`
themselves and use `*Run::send` directly.

### Why not a single polymorphic free function

The six wrappers use six different inner constructors with six
different bound shapes (`'static` vs `'a`, `Clone` on the Apply
node-projection, `Send + Sync` for Arc, etc.) and there is no
common trait abstracting "construct from a node-projection".
Inventing one to make `lift` polymorphic would be more code
than just writing six near-identical inherent methods, which is
the same trade-off `*Run::send` already settled on its 2026-04-27
resolution.

### Why raw effect input (not pre-lifted Coyoneda)

Matches PureScript Run's
[`lift :: Row.Cons sym f r1 r2 => Proxy sym -> f a -> Run r2 a`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs),
which takes the raw effect (`f a`) and does the inject + liftF
chain internally. Phase 3's smart constructors then become
one-liners
(`pub fn ask<R, S, Idx>() -> Run<R, S, Env> { Run::lift::<ReaderBrand, _>(Reader::Ask) }`),
mirroring PureScript's
`liftEffect = lift (Proxy :: "effect")` pattern. The Coyoneda
detail stays an implementation concern of the helper rather than a
user-visible step. Users who want to construct a non-trivial
Coyoneda decode bypass `lift` and call `Coyoneda::new` plus
`*Run::send` directly.

### Why "try inline; fall back if needed" for the HRTB workaround

The 2026-04-27 GAT-normalization issue specifically hit
`Apply!(<NodeBrand<R, S> as Kind>::Of<'static, A>)` _normalization_
inside `ArcFree`'s HRTB-bearing scope. The `lift` body builds the
Node-projection by _literal construction_ (`Node::First(Member::inject(coyo))`);
no `Apply!` normalization is required on the result type, only on
the `effect` parameter (which is fine, since it's already
pre-resolved at the function boundary). Plausibly clean for
`ArcRun::lift`. If it does fail, the workaround is mechanical:
factor `Node::First(<_ as Member<_, Idx>>::inject(Coyoneda::lift(effect)))`
into a free helper outside the HRTB scope and have
`ArcRun::lift` call `Self::send(lift_node::<R, S, EBrand, Idx, A>(effect))`.
Pre-baking the `lift_node` helper in all six wrappers
prophylactically would be wasted code if the simple form works
for everything but `ArcRun`.

## Resolved (2026-04-28 implementation expansion): step 9 SendFunctor cascade prerequisites for Arc family

While implementing the original 2026-04-28 resolution above,
`Run::lift`
landed cleanly at commit `34b6a97`. Extending the same body to
`RunExplicit`, `RcRun`, `RcRunExplicit` worked. But `ArcRun::lift`
and `ArcRunExplicit::lift` hit a structural conflict the original
resolution didn't anticipate.

### Problem

`ArcRun`'s struct-level HRTB
(`Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync`)
forces every variant of the row's projection to be `Send + Sync`.
The bare
[`Coyoneda`](../../../fp-library/src/types/coyoneda.rs) stores
its accumulated continuation in `Box<dyn FnOnce>` (no
`Send + Sync`), so `Coyoneda<'_, EBrand, A>` is not
`Send + Sync` and `ArcRun` rejects `CoyonedaBrand`-headed rows.

The Send-aware companion
[`ArcCoyoneda`](../../../fp-library/src/types/arc_coyoneda.rs)
exists and is `Send + Sync`. But
[`ArcCoyonedaBrand`](../../../fp-library/src/brands.rs)
deliberately doesn't implement
[`Functor`](../../../fp-library/src/classes/functor.rs) (it
only implements [`SendFunctor`](../../../fp-library/src/classes/send_functor.rs)
and [`Foldable`](../../../fp-library/src/classes/foldable.rs)),
because the
[`Functor::map`](../../../fp-library/src/classes/functor.rs)
trait method's signature lacks `Send + Sync` bounds on its
closure parameter; closures stored in Arc-wrapped layers must
be `Send + Sync`. This is a deliberate fp-library design
choice: the Send-aware parallel trait family
([`SendFunctor`](../../../fp-library/src/classes/send_functor.rs),
[`SendPointed`](../../../fp-library/src/classes/send_pointed.rs),
[`SendSemimonad`](../../../fp-library/src/classes/send_semimonad.rs),
[`SendApplicative`](../../../fp-library/src/classes/send_applicative.rs),
etc., plus the
[`SendRef`](../../../fp-library/src/classes/send_ref_functor.rs)
prefix tree) exists to handle Arc-substrate brands; plain
`Functor` deliberately does not impose Send bounds for the
common-case non-thread-crossing brands.

`ArcRun`'s existing `peel` / `send` / `bind` / `map`
implementations route through `<NodeBrand<R, S> as Functor>::map`
on the row brand. `NodeBrand: Functor` cascades to `R: Functor`,
which `ArcCoyonedaBrand` cannot satisfy. So the universal
`Run.lift` shape (Coyoneda lift -> row inject -> `Node::First` ->
`*Run::send`) cannot work for the Arc family without a Send-aware
substrate path.

The substrate
[`ArcFree`](../../../fp-library/src/types/arc_free.rs) compounds
the issue: its internal machinery (`lift_f`, `wrap`, `bind`,
`evaluate`, `fold_free`, `hoist_free`) all bound `F: Functor` and
call `F::map` directly. Switching the Run wrappers to the
Send-aware tree requires switching the substrate too.

### Resolution

**Expand step 9 with a `SendFunctor` cascade as prerequisite
sub-steps before the universal `lift` work. Replace
`F: Functor` bounds with `F: SendFunctor` on the Arc-substrate
machinery (`ArcFree`, `ArcFreeExplicit`); land the missing
`SendFunctor` impls on the row-cascade brands; expand the
brand-level type-class surface on `ArcFreeExplicitBrand` and
`ArcRunExplicitBrand` to absorb newly-reachable Send-aware
impls; then complete `*Run::lift` for all six wrappers under
the now-supported cascade.** Implement `SendRefFunctor` on
`ArcRunExplicitBrand` via inherent-method delegation (calling
the wrapper's `ref_map` / `ref_bind` / `ref_pure` directly,
bypassing the brand-level cascade via the clone-trick).

The expanded sub-step structure lives in plan.md step 9 (9a
through 9i); each lands as a separate commit. The `Run::lift`
implementation already shipped at commit `34b6a97` stays as the
reference design; sub-step 9h fills in the remaining five
wrappers.

### Why "replace Functor with SendFunctor" instead of adding sibling methods

Two paths considered:

- **Replace `F: Functor` with `F: SendFunctor`** on `ArcFree` /
  `ArcFreeExplicit`'s methods (signatures change, internal
  `F::map` calls become `F::send_map`). Breaking change for any
  pre-existing caller passing a non-Send `Functor`-only row
  brand. Cleaner long-term: one method per operation; semantic
  alignment between the substrate's thread-safety bounds and
  the trait surface.
- **Add Send-aware sibling methods** (`ArcFree::send_lift_f`
  alongside `ArcFree::lift_f`). Backwards-compatible but doubles
  the API surface; users have to pick the right method. The cost
  compounds across `ArcFreeExplicit`'s siblings.

Replacement chosen because `ArcFree`'s struct-level Send+Sync
HRTB already restricts concrete callers to row brands that
satisfy `Send + Sync`; adding `SendFunctor` impls to the row-
cascade brands (sub-step 9a) keeps existing concrete callers
working without method-surface duplication.

### Why `SendRefFunctor` via inherent-method delegation

The 2026-04-27
"[brand-level type-class coverage gap on the Explicit Run brands](#resolved-2026-04-27-brand-level-type-class-coverage-gap-on-the-explicit-run-brands)"
resolution documented `SendRef`-family hierarchy as unreachable
through brand-level delegation: `ArcFreeExplicitBrand` can't
implement `SendRefFunctor` because the auto-derive of
`Send + Sync` on the closure return type requires a per-`A`
HRTB on the `Kind` projection that stable Rust's trait method
signatures cannot carry.

The unreachability is at the substrate-brand level. The
`ArcRunExplicit` _wrapper_ has inherent
`ref_map`
/ `ref_bind` / `ref_pure` methods that work via the clone-trick
(`self.clone().send_map(move |a| f(&a))`); the `O(1)`
`Arc::clone` makes this cheap, and the per-`A` HRTB doesn't
appear at the wrapper-method signature because the closure
constraints are checked against the inherent method's bound
list rather than against the brand-level trait method's. So
`ArcRunExplicitBrand: SendRefFunctor` is reachable if the impl
delegates to the wrapper's inherent `ref_map`, sidestepping
`ArcFreeExplicitBrand` entirely.

This is a different delegation strategy than what step 4b's
resolution considered (substrate-brand delegation). The
inherent-method delegation pattern produces a working
brand-level `SendRefFunctor` impl with the same observable
behavior at the cost of an `O(1)` clone per call. The clone is
acceptable: brand-level dispatch is the path the user opted into
when they wrote `<ArcRunExplicitBrand as SendRefFunctor>::send_ref_map`,
and the alternative is no brand-level coverage at all.

### Why not defer the SendFunctor cascade to a later phase

Three plausible structures considered:

- **Defer to Phase 1.5 follow-up.** The SendFunctor cascade on
  the row-brand types is genuinely substrate-level
  infrastructure, and Phase 1's WrapDrop migration set a
  precedent for landing prerequisite trait-cascade work as a
  follow-up between phases. But Phase 1 has long completed; a
  retroactive "Phase 1.5" is structurally awkward and signals
  bigger drift than the work warrants.
- **Defer to Phase 3.** Phase 3 step 4 lands per-effect smart
  constructors that build on `*Run::lift`. Deferring the
  cascade would push `ArcRun::ask` / `ArcRun::get` etc. behind
  a structural prerequisite, breaking Phase 3's promise of
  thin one-liners over `*Run::lift`.
- **Expand step 9's scope.** Most coherent: the cascade is
  required by step 9's universal-`lift` promise; landing it as
  step 9 sub-steps keeps the prerequisite-and-payoff together,
  visible in one place, and verifiable as a unit. The smaller
  sub-step granularity ensures each is independently
  reviewable.

The third option chosen.

### Reference: scope inventory at start of expansion

Confirmed by code inspection at the time the blocker surfaced:

- `ArcCoyonedaBrand`: has [`SendFunctor`](../../../fp-library/src/types/arc_coyoneda.rs);
  needs [`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs).
- `IdentityBrand`: has `Functor` and `WrapDrop`; needs
  `SendFunctor` (mechanical; `Identity<A>` has no closures, so
  the closure `Send + Sync` requirement is vacuous).
- `NodeBrand`, `CoproductBrand<H, T>`, `CNilBrand`: have
  `Functor` and `WrapDrop`; need `SendFunctor` (recursive
  cascade for the inductive cases; uninhabited base case for
  `CNilBrand`).
- `ArcFree`: bounds `lift_f` / `wrap` / `bind` / `evaluate` /
  `fold_free` / `hoist_free` on `F: Functor`; calls `F::map` at
  three sites; switch to `F: SendFunctor` and `F::send_map`.
- `ArcFreeExplicit`: same shape as `ArcFree`; same migration.
- `ArcRun`: methods route through `<NodeBrand<R, S> as Functor>::map`;
  switch to `<NodeBrand<R, S> as SendFunctor>::send_map` after
  the cascade lands.
- `ArcRunExplicit`: same as `ArcRun`.
- `ArcFreeExplicitBrand`: brand-level coverage limited to
  `SendPointed` per step 4b; expand to `SendFunctor` and
  cascade dependents under the Send-aware machinery.
- `ArcRunExplicitBrand`: same expansion path; plus the
  `SendRefFunctor`-via-inherent-method-delegation impl.

## Resolved (2026-04-27): `*Run::send` takes a `Node`-projection value to sidestep GAT-normalization poisoning under `ArcFree`'s HRTB

Step 5's `send` method on each of the six Run wrappers takes the
[`NodeBrand<R, S>`](../../../fp-library/src/brands.rs)
`Of<'_, A>` projection (already-constructed) rather than the
first-order row variant (constructed internally via
`Node::First(layer)`). This deviates from the natural shape that
mirrors PureScript Run's `send`, but is required because of a
stable-Rust GAT-normalization limit that surfaces in `ArcRun`'s
impl-block context.

### Problem

While implementing `ArcRun::send` with the natural shape (take
the row variant `R::Of<'static, A>`, construct
`Node::First(layer)` internally, pass to `ArcFree::lift_f`),
the compiler refused to unify `Node<'static, R, S, A>` (the
literal value) with
`<NodeBrand<R, S> as Kind_cdc7cd43dac7585f>::Of<'static, A>`
(the projection that `ArcFree::lift_f` expects), even though
`impl_kind!` declares them equal:

```
expected associated type `<NodeBrand<R, S> as kinds::Kind_cdc7cd43dac7585f>::Of<'static, A>`
                  found enum `node::inner::Node<'static, R, S, A>`
```

The same construction succeeds for `Run::send` (over
[`Free`](../../../fp-library/src/types/free.rs)) and
`RcRun::send` (over
[`RcFree`](../../../fp-library/src/types/rc_free.rs)). The
difference is that
[`ArcFree`](../../../fp-library/src/types/arc_free.rs)'s struct
carries a per-`A`-instantiation HRTB
`F: Kind<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>`
(needed so the compiler can auto-derive `Send + Sync` on
`ArcFree<F, A>` when `F`'s `Of` projection is `Send + Sync`).
This HRTB propagates to `ArcRun`'s impl block, and inside that
block stable Rust's normalizer refuses to fire for any other
instantiation of the same `Of` projection.

### Investigation

Eleven experiments at
`fp-library/tests/arc_run_normalization_probe.rs`
(see history; trimmed in the final commit to the four passing
patterns) isolated the trigger:

- The HRTB itself, not the `ArcFree` field, is the trigger
  (PhantomData-only struct + HRTB still fails).
- The trigger is not impl-block-specific: a free function
  carrying the HRTB also fails.
- The trigger poisons cross-substrate calls: a `RcFree::lift_f`
  call from inside an `ArcFree`-HRTB-bearing impl also fails.
- Workarounds tried that all fail: explicit `Apply!()`-typed
  local; turbofish `Node::<'static, R, S, A>::First(layer)`;
  using `<...as Kind_cdc7cd43dac7585f>::Of` directly bypassing
  `Apply!`; routing through `Functor::map(identity, Node::First(layer))`
  (whose input is also at the projection); restructuring the
  impl block to use direct `R: ... + 'static, S: ... + 'static`
  bounds plus the HRTB.
- The workaround that succeeds: pass an already-projection-typed
  value into the HRTB-scope function, never construct a Node
  literal there. The caller (typically code without HRTB in
  scope, e.g., test code, smart-constructor macro output) builds
  `Node::First(layer)` and passes the result.

The probe file at
`fp-library/tests/arc_run_normalization_probe.rs`
is the trimmed regression-test version that documents the four
patterns confirmed to work despite the limit.

### Resolution

`*Run::send` on all six wrappers takes the
`Node`-projection value as a parameter, uniform signature:

```rust
pub fn send(
    node: Apply!(<NodeBrand<R, S> as Kind!(...)>::Of<'_, A>),
) -> Self;
```

Smart constructors (Phase 2 step 9) will emit
`Node::First(<R as Member<...>>::inject(coyo))` in their bodies
and pass the result to `send`. User test code does the same.

### Why not work around at a different layer

- **Re-architect `ArcFree` to remove the struct-level HRTB**:
  out of scope for step 5 (would require a Phase 1 follow-up
  commit). The HRTB is load-bearing for `Send + Sync` auto-derive
  on `ArcFree`, which dozens of other code paths depend on.
- **Provide `unsafe impl Send` / `unsafe impl Sync` for
  `ArcRun`** with bounds that don't include the HRTB: the unsafe
  impl's `where` clause would still need to express the
  Send/Sync condition somehow, and any expression of "the
  projection at this instantiation is Send + Sync" is itself an
  HRTB-shaped constraint that re-triggers the issue.
- **Accept the asymmetry between `Run`/`RcRun` (take row
  variant) and `ArcRun` (take Node projection)**: the symmetric
  approach was chosen for design consistency (the two patterns
  diverging across the six wrappers would surface as confusion
  in users of step 7's macros and step 9's smart constructors).

## Resolved (2026-04-27): brand-level type-class coverage gap on the Explicit Run brands

The plan's Phase 2 step 4 specification named a full
`Functor / Pointed / Semimonad / Monad` hierarchy plus a
`RefFunctor / RefPointed / RefSemimonad / RefMonad` hierarchy
for `RunExplicitBrand`, with analogous coverage for
`RcRunExplicitBrand` and `ArcRunExplicitBrand`. Step 4b
landed the achievable subset: `Functor / Pointed / Semimonad`
plus the by-reference equivalents for `RunExplicitBrand`,
`Pointed` plus by-reference equivalents for
`RcRunExplicitBrand`, and `SendPointed` only for
`ArcRunExplicitBrand`.
[`Monad`](../../../fp-library/src/classes/monad.rs) /
[`RefMonad`](../../../fp-library/src/classes/ref_monad.rs) /
[`SendMonad`](../../../fp-library/src/classes/send_monad.rs) and
the [`SendRef`](../../../fp-library/src/classes/send_ref_functor.rs)-family
hierarchy are not reachable through brand-level delegation;
inherent `bind` and `map` methods on `RcRunExplicit` and
`ArcRunExplicit` (mirroring
[`RcFreeExplicit`](../../../fp-library/src/types/rc_free_explicit.rs)'s
inherent surface) cover the by-value monadic surface for
concrete-type call sites.

### Problem

Three independent gaps share the same root cause: stable Rust's
trait method signatures cannot carry per-`A` bounds (no HRTB
over types), and the `*FreeExplicitBrand`s the Run-Explicit
brands delegate to deliberately do not implement the missing
classes for the same reason.

1. **`Monad` blanket impl requires `Applicative`.** The
   project's [`Monad`](../../../fp-library/src/classes/monad.rs)
   trait at line 214 is
   `pub trait Monad: Applicative + Semimonad {}` with a blanket
   `impl<Brand> Monad for Brand where Brand: Applicative + Semimonad {}`
   at line 218. Same shape for
   [`RefMonad`](../../../fp-library/src/classes/ref_monad.rs)
   over `RefApplicative + RefSemimonad`. So a brand cannot be
   `Monad` without first being `Applicative`.
   [`FreeExplicitBrand`](../../../fp-library/src/brands.rs)
   deliberately does not implement
   [`Applicative`](../../../fp-library/src/classes/applicative.rs)
   (its [`Lift`](../../../fp-library/src/classes/lift.rs)
   supertrait's natural definition pattern
   `lift2 = bind(fa, |a| map(fb, |b| f(a, b)))` requires `fb` to
   be reusable across closure invocations, and
   [`FreeExplicit`](../../../fp-library/src/types/free_explicit.rs)
   is not `Clone` per [`free_explicit.rs`](../../../fp-library/src/types/free_explicit.rs)
   lines 369-388). The Run wrapper brands inherit this gap
   through delegation.
2. **`SendRef` hierarchy unreachable on `ArcRunExplicitBrand`.**
   The [`ArcFreeExplicit`](../../../fp-library/src/types/arc_free_explicit.rs)
   substrate auto-derives `Send + Sync` only when its struct
   carries a per-`A` `Kind` HRTB
   (`Of<'a, ArcFreeExplicit<'a, F, A>>: Send + Sync`).
   That bound's `'a` and `A` are the trait method's per-method
   generics; stable Rust does not support `for<'a, T>` HRTB at
   the impl-block level. So
   [`ArcFreeExplicitBrand`](../../../fp-library/src/brands.rs)
   does not implement
   [`SendRefFunctor`](../../../fp-library/src/classes/send_ref_functor.rs)
   /
   [`SendRefPointed`](../../../fp-library/src/classes/send_ref_pointed.rs)
   /
   [`SendRefSemimonad`](../../../fp-library/src/classes/send_ref_semimonad.rs)
   (see [`arc_free_explicit.rs`](../../../fp-library/src/types/arc_free_explicit.rs)
   lines 730-745). `ArcRunExplicitBrand`'s would-be Send-Ref
   delegation has no target.
3. **Ref hierarchy is bounded by `R: RefFunctor`.** The Ref
   impls on `RunExplicitBrand` and `RcRunExplicitBrand` delegate
   to the corresponding `*FreeExplicitBrand`'s Ref impls, which
   carry `F: WrapDrop + Functor + RefFunctor + 'static`.
   For `Run`, `F = NodeBrand<R, S>`; the cascade requires
   `R: RefFunctor` and `S: RefFunctor`. Step 4b adds
   [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs)
   impls on `CNilBrand`, `CoproductBrand<H, T>`, and
   `NodeBrand<R, S>`, but
   [`CoyonedaBrand`](../../../fp-library/src/brands.rs) does not
   implement
   [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs).
   Canonical Run rows (`CoproductBrand<CoyonedaBrand<E_i>, ...>`)
   do not satisfy the cascade. The Ref impls are present at the
   brand level but reachable only for synthetic rows whose
   brands carry their own `RefFunctor` impls (e.g.,
   `CoproductBrand<IdentityBrand, CNilBrand>`).

### Resolution

Ship the achievable subset; document gaps as deviations. Future
work that needs the missing coverage either reaches for the
inherent methods on the concrete Run wrapper types or, for
`Coyoneda`-wrapped effect rows, adds `RefFunctor` to
[`CoyonedaBrand`](../../../fp-library/src/types/coyoneda.rs)
(scope-creep beyond step 4b; tracked separately).

### Why not work around

- **Restructuring `Monad`'s supertrait chain:** would require
  editing [`monad.rs`](../../../fp-library/src/classes/monad.rs)
  and similar; out of scope for the effects port and would break
  every existing brand impl.
- **Adding `Applicative` impls with `Clone` bounds at the trait
  signature level:** stable Rust's
  [`Applicative::lift2`](../../../fp-library/src/classes/lift.rs)
  signature can't be augmented; per-method `where` clauses on
  trait impls are restricted to what the trait allows.
- **Adding the SendRef hierarchy directly on
  `ArcRunExplicitBrand`** (bypassing
  `ArcFreeExplicitBrand`): would have the same per-`A` HRTB
  obstacle the underlying brand has.

## Resolved (2026-04-27): row-brand `RefFunctor` and `Extract` cascade impls land in step 4b

Phase 2 step 4a left
[`CNilBrand`](../../../fp-library/src/types/effects/variant_f.rs),
[`CoproductBrand<H, T>`](../../../fp-library/src/types/effects/variant_f.rs),
and
`NodeBrand<R, S>`
with [`Functor`](../../../fp-library/src/classes/functor.rs)
and [`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs)
impls only. Step 4b added
[`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs)
and [`Extract`](../../../fp-library/src/classes/extract.rs)
cascade impls on each of the three brands, plus a
[`Clone`] impl for the
`Node` enum.

### Problem

Three trait gaps surfaced as step 4b's Explicit family was
landed:

1. **`RefFunctor` needed for Ref-hierarchy delegation.**
   `RunExplicitBrand`'s
   [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs)
   impl delegates to
   [`FreeExplicitBrand`](../../../fp-library/src/brands.rs)'s,
   which carries `F: WrapDrop + Functor + RefFunctor + 'static`.
   For `Run`, `F = NodeBrand<R, S>`; the cascade requires
   `R: RefFunctor` and `S: RefFunctor`, so the row brand chain
   must support it.
2. **`Extract` needed for `evaluate()` on canonical Run
   programs.**
   [`FreeExplicit::evaluate`](../../../fp-library/src/types/free_explicit.rs)
   requires `F: Extract`. For `Run`, `F = NodeBrand<R, S>`; the
   cascade requires `R: Extract` and `S: Extract`.
   [`IdentityBrand`](../../../fp-library/src/types/identity.rs)
   has [`Extract`](../../../fp-library/src/classes/extract.rs);
   the row chain (Coproduct / CNil / Node) did not.
   Without it, brand-level test programs and doctests over
   synthetic rows could not assert evaluation results.
3. **`Clone` needed by Rc/Arc Free's evaluate fallback.**
   [`RcFreeExplicit::evaluate`](../../../fp-library/src/types/rc_free_explicit.rs)
   and
   [`ArcFreeExplicit::evaluate`](../../../fp-library/src/types/arc_free_explicit.rs)
   carry the per-`A` bound
   `Apply!(<F as Kind!(...)>::Of<'a, *FreeExplicit<'a, F, A>>): Clone`.
   For `F = NodeBrand<R, S>`, this expands to
   `Node<'a, R, S, *FreeExplicit<'a, NodeBrand<R, S>, A>>: Clone`.
   `Node` did not implement
   [`Clone`].

### Resolution

Land mechanical cascade impls on the row brands following the
same shape as the existing
[`Functor`](../../../fp-library/src/classes/functor.rs) /
[`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs)
impls:

- [`CNilBrand`](../../../fp-library/src/types/effects/variant_f.rs):
  uninhabited base case for both
  [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs)
  and
  [`Extract`](../../../fp-library/src/classes/extract.rs).
- [`CoproductBrand<H, T>`](../../../fp-library/src/types/effects/variant_f.rs):
  dispatches by `Inl` / `Inr` recursing into the active brand;
  bounded `H: RefFunctor + 'static, T: RefFunctor + 'static`
  for [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs);
  same shape with [`Extract`](../../../fp-library/src/classes/extract.rs)
  for the Extract impl.
- `NodeBrand<R, S>`:
  dispatches by `First` / `Scoped`; bounded
  `R: RefFunctor + 'static, S: RefFunctor + 'static` for
  [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs);
  same shape for [`Extract`](../../../fp-library/src/classes/extract.rs).
- `Node<'a, R, S, A>`:
  manual [`Clone`] impl bounded on `Apply!(<R as Kind!(...)>::Of<'a, A>): Clone`
  and the `S` projection; clones the active variant's payload.

`SendRefFunctor` cascade is _not_ added because
[`ArcRunExplicitBrand`](../../../fp-library/src/brands.rs)
cannot have a SendRef hierarchy in the first place (see the
adjacent resolution about brand-level coverage gaps).

## Resolved (2026-04-27): re-export pattern for the effects subsystem types follows the optics A+B hybrid

Step 4b adopts the
[`optics`](../../../fp-library/src/types/optics.rs) precedent:
selective top-level re-exports of headline types in
[`crate::types::*`](../../../fp-library/src/types.rs), plus
comprehensive subsystem-scoped re-exports at
[`crate::types::effects::*`](../../../fp-library/src/types/effects.rs).

### Problem

Phase 2 step 4 left re-exports undecided. Three options were
considered:

- **A. Top-level only** (`crate::types::*`): matches the rest
  of the [`types/`](../../../fp-library/src/types/) directory;
  ergonomic; but ~12 names land in the top-level block and the
  effects subsystem stops being visually distinguished.
- **B. Subsystem-scoped only** (`crate::types::effects::*`):
  preserves the top-level namespace shape; matches what
  [`optics`](../../../fp-library/src/types/optics.rs) does for
  non-headline types; but deviates from the Free family's
  surface.
- **C. No re-exports**: zero maintenance, but friction at
  every import site and matches no existing pattern.

The existing
[`optics`](../../../fp-library/src/types/optics.rs) precedent
is neither pure A nor pure B: it re-exports every submodule
symbol via
`pub use submodule::*` at
[`crate::types::optics::*`](../../../fp-library/src/types/optics.rs)
(comprehensive, B), AND surfaces only the three headline types
[`Composed`](../../../fp-library/src/types/optics.rs),
[`Lens`](../../../fp-library/src/types/optics.rs),
[`LensPrime`](../../../fp-library/src/types/optics.rs) at the
top-level (selective, A).

### Resolution

Adopt the optics precedent literally: the six Run wrapper
headline types
(`Run`, `RcRun`, `ArcRun`, `RunExplicit`, `RcRunExplicit`,
`ArcRunExplicit`) are headline-class and ship at the top level
([`crate::types::*`](../../../fp-library/src/types.rs)) because
they are the user-facing types most callers will import; the
brands and row machinery (`Node`, `VariantF`,
`*RunExplicitBrand`) are subsystem-scoped and ship at
[`crate::types::effects::*`](../../../fp-library/src/types/effects.rs)
only. Brand types stay in
[`crate::brands::*`](../../../fp-library/src/brands.rs) per the
existing precedent for all brand types in the library.

## Resolved (2026-04-27): introduce `WrapDrop` trait for Free's struct-level Drop concern

A new trait `WrapDrop` lands at the struct level of the Free
family, replacing `Extract` for `Drop`'s iterative-dismantling
purposes while preserving `Extract` as a separate trait for
`evaluate` / `fold_free` / etc. Migration ships as two Phase 1
follow-up commits before Phase 2 step 4 resumes; the actual
step-by-step migration spec lives in
[plan.md](plan.md)'s "Phase 1 follow-up: WrapDrop migration"
section.

### Problem

Phase 2 step 4 (the six concrete `Run` types) commits to
`Run<R, S, A> = Free<NodeBrand<R, S>, A>` per
[decisions.md](decisions.md) section 5.2 and [plan.md](plan.md)'s
"Will change" table entry for
[`fp-library/src/types/effects.rs`](../../../fp-library/src/types/effects.rs).
This requires `Free<NodeBrand<R, S>, A>` to compile for typical
effect rows. It does not, because of a transitively-poisoning
trait bound:

1. [`Free<F, A>`](../../../fp-library/src/types/free.rs) (and
   the other five Free variants) declares its struct with
   `where F: Extract + Functor + 'static`. The `Extract` bound
   is enforced at the type-declaration site, not just on
   inherent methods, so a `Free<NodeBrand<R, S>, A>` instance
   fails to compile when `NodeBrand<R, S>` does not implement
   `Extract`.
2. [`Free::drop`](../../../fp-library/src/types/free.rs) calls
   `<F as Extract>::extract(fa)` to walk deep `Wrap` chains
   iteratively. This is what keeps a 100 000-deep `Wrap` chain
   from stack-overflowing during cleanup; the `Extract` bound
   is load-bearing for the existing `Drop` strategy, which is
   why the bound is on the struct rather than on individual
   methods (Rust requires `Drop` impl bounds to match struct
   bounds exactly).
3. To satisfy `NodeBrand<R, S>: Extract` for typical Run usage,
   the bound recurses into the row brands. For the first-order
   row, `R = CoproductBrand<CoyonedaBrand<E1>, CoproductBrand<...>>`,
   and the recursive bound bottoms out at
   `CoyonedaBrand<E>: Extract`.
4. `CoyonedaBrand<E>::extract` would need to recover an `A` from
   `Coyoneda<E, A>`. The natural implementation lowers the
   Coyoneda (`coyo.lower()` returns `E::Of<A>`, requires
   `E: Functor`) and then calls `<E as Extract>::extract(...)`.
   So the bound transitively requires `E: Extract` for every
   effect type in the row.
5. Effect types (`Reader<E>`, `State<S>`, `Choose`, `Except<E>`,
   `Writer<W>`, etc.) are pure data with no canonical
   "evaluate" semantics: they need a handler to interpret. So
   `Reader<E>: Extract` (and the same for every other effect)
   cannot hold without baking arbitrary semantics into each
   effect type.

The bound is correct for the Free family's general use cases
(`Free<IdentityBrand>` evaluates by unwrapping; `Free<ThunkBrand>`
evaluates by running the thunk). It is over-conservative for the
effects-as-data use case Run needs.

### Investigation: Wrap-depth probe

A probe at
[`fp-library/tests/run_wrap_depth_probe.rs`](../../../fp-library/tests/run_wrap_depth_probe.rs)
(commit `09d676b`) measures `Wrap`-arm depth in Run-shaped
programs over `Free<ThunkBrand, _>` (using `ThunkBrand` because
`Free<IdentityBrand, _>` is layout-cyclic per the Phase 1 step 8
deviation, but the structural behaviour the probe measures is
brand-independent). The probe distinguishes two metrics:

- **Evaluation depth:** how many `Wrap` layers materialise when
  `to_view` applies pending continuations and follows the
  resulting `Wrap` chain via `Extract`. This is what an
  interpreter sees when walking the program.
- **Structural depth:** how many `Wrap` layers exist in the
  original view BEFORE `to_view` applies any continuation.
  This is what `Drop` traverses, because `Drop` dismantles
  the view and continuations in place without applying the
  closures.

Seven tests and their findings:

| Pattern                                                                | Evaluation depth | Structural depth                |
| ---------------------------------------------------------------------- | ---------------- | ------------------------------- |
| `Free::pure(0)`                                                        | 0                | 0                               |
| `pure(0).bind(\|x\| pure(x+1))` chained 1000 times                     | 0                | 0                               |
| `lift_f(eff)` alone                                                    | 1                | 1                               |
| `lift_f(eff).bind(\|x\| pure(x+1))` chained 1000 times                 | 1                | 1                               |
| `pure(0).bind(\|x\| lift_f(eff))` chained 100 times                    | 100              | 0                               |
| `lift_f(eff).bind(\|x\| pure(x+1))` chained 100 000 times, then `drop` | n/a              | succeeds without stack overflow |
| Explicit `Free::wrap(...)` chained 100 times                           | 100              | 100                             |

Bottom-line finding: Run-typical programs (built via `lift_f`
plus a flat `bind` chain) have structural `Wrap` depth at most
1, regardless of bind-chain length. The depth that grows with
sequencing lives in the `CatList` of continuations, which the
existing iterative `Drop` already dismantles without calling
`Extract`. The 100 000-bind drop test passes without stack
overflow even though `Drop` only walks one `Wrap` layer (the
original `lift_f`'s `Wrap`) recursively.

The artificial 100-deep `Free::wrap` chain pattern (last row) is
the case that motivated the existing `Extract`-based iterative
`Drop`. Run-typical usage does not produce this pattern; users
inject effects via `lift_f` (one `Wrap` per call) and chain via
`bind` (no new `Wrap`s). The probe also covers
`nested_lift_f_via_bind_materializes_wraps_at_evaluation_time`,
showing that `bind` closures returning `lift_f` build their
`Wrap`s at _evaluation_ time, not construction time, so they
live in the `CatList` rather than the structural `Wrap` chain.

### Resolution: introduce the `WrapDrop` trait

A new trait `WrapDrop` separates the structural-cleanup question
(what `Drop` needs) from the semantic-interpretation question
(what `Extract` answers). `Extract` continues to mean "given
`F::Of<X>`, give me the `X`" and is used by `evaluate`,
`fold_free`, `resume`, etc. `WrapDrop` instead asks "given
`F::Of<X>`, can you yield the inner `X` without running user
code?", returning `Option<X>`.

#### Trait definition

```rust
pub trait WrapDrop: Kind {
    /// Drop-time decomposition. `Some(x)` means F materially
    /// stores X and the caller can iterate on it. `None` means
    /// F doesn't store X (or storing is closure-captured), so
    /// the caller should let `fa` drop normally.
    fn drop<'a, X: 'a>(fa: Self::Of<'a, X>) -> Option<X>;
}
```

#### Naming rationale

The trait's name reflects that it is the operation `Free`'s
`Wrap` variant performs at drop time. The method name `drop`
does not clash with `std::ops::Drop::drop` because they are
different traits with different receiver shapes
(`std::ops::Drop::drop(&mut self)` is a method;
`WrapDrop::drop(fa: F::Of<'_, X>)` is an associated function).
Call sites use fully-qualified syntax:
`<F as WrapDrop>::drop(fa)`.

#### Free's Drop dispatch

Free's `Drop` impl is rewritten to dispatch on the `Option`:

```rust
match F::drop(layer) {
    Some(inner) => worklist.push(inner.view); // existing iterative path
    None => { /* layer already dropped recursively by the match arm */ }
}
```

#### Per-F policy choices

- **F materially stores the inner X** (e.g., `IdentityBrand`):
  `WrapDrop::drop` returns `Some(<F as Extract>::extract(fa))`,
  preserving the existing iterative path.
- **F's storage runs user code to materialise X but the
  existing test suite relies on iterative dismantling** (e.g.,
  `ThunkBrand`): `WrapDrop::drop` returns
  `Some(<F as Extract>::extract(fa))`. This preserves
  side-effect-on-Drop semantics and the Phase 1
  `deep_drop_does_not_overflow` test. The alternative (return
  `None` to skip closures) was rejected because the closure's
  captures hold inner Frees that would drop recursively for
  100k-deep chains.
- **F does not materially store X at all** (e.g.,
  `CoyonedaBrand<E>`, `CoproductBrand<H, T>`, `CNilBrand`,
  `NodeBrand<R, S>`): `WrapDrop::drop` returns `None`. Drop
  falls through to recursive drop on `fa`; the probe validates
  this is sound for Run-typical patterns because the `F::Of<X>`
  storage doesn't materially recurse on inner Frees (Coyoneda's
  closure would construct a Free if called, but doesn't store
  one; the Coproduct's variants hold Coyonedas which have the
  same property).

#### Documented limitation

Artificial deep `wrap(...)` chains over F's whose
`WrapDrop::drop` returns `None` (e.g., a hand-built 100k-deep
`wrap(Coyoneda(...))` chain) overflow the stack on `Drop`.
Run-typical usage does not generate this pattern, and no
existing test exercises it. The trait's docs warn future
F-authors of the constraint.

### Alternatives considered and rejected

Four resolution paths were evaluated; the chosen path is the
`WrapDrop` introduction described above. The other three are
recorded for design-history transparency:

- **Build a parallel `RunFree`-like substrate without the
  `Extract` bound.** Define six new types in `types/effects/`
  paralleling the six existing Free variants, with relaxed
  bounds and recursive `Wrap` drop. Same insight as the chosen
  path but isolated to Run; Phase 1's Free family would stay
  untouched. Probe-validated as sound for Run usage. Rejected
  because it duplicates the entire substrate (CatList for
  Erased, naive recursive enum for Explicit, custom `Drop`)
  for one architectural concern. `WrapDrop` achieves the same
  expressivity with a single new trait and mechanical-but-
  unified migration.
- **Make Run a newtype struct that internally holds something
  other than a raw `Free<NodeBrand, A>`** (e.g., a
  `Box<dyn ...>` trait object, a custom enum, or a Free over a
  placeholder brand that does implement Extract trivially while
  effect data lives elsewhere). Rejected because it diverges
  from the plan's literal "Run is a Free" model
  ([decisions.md](decisions.md) section 5.2,
  [README of `purescript-run`](https://github.com/natefaubion/purescript-run))
  and the other paths achieve the goal without redesigning
  the relationship.
- **Implement `Extract` for `CoyonedaBrand<E>` /
  `CoproductBrand<H, T>` / `NodeBrand<R, S>` with panic
  semantics** (extract panics with a clear "handler required"
  message; Drop falls back to recursive drop when extract
  panics). Rejected as a footgun: programs that drop unhandled
  Run values panic in legitimate scenarios (program panics in
  user code mid-evaluation, deliberate program discarding,
  test fixtures asserting on Run structure without running it).

## Resolved (2026-04-26): brand-level dispatch for the multi-shot Explicit Free family lands on the by-reference hierarchy

`RcFreeExplicit::bind` requires `A: Clone` (because shared inner
state must clone to recover an owned `A`), and stable Rust does
not admit per-method `where A: Clone` on a `Functor::map` impl.
This is the same constraint that
[fp-library/docs/limitations-and-workarounds.md](../../../fp-library/docs/limitations-and-workarounds.md)
documents under "Unexpressible Bounds in Trait Method Signatures"
for `RcCoyoneda`/`ArcCoyoneda` and addresses under "Memoized Types
Cannot Implement `Functor`" via the by-reference hierarchy
(`RefFunctor`, `RefSemimonad`, `RefMonad` and `SendRef*`
parallels) that `Lazy` already uses. The decision is to follow
`Lazy`'s precedent.

### Brand-level coverage

- `FreeExplicitBrand`: full by-value (`Functor` / `Pointed` /
  `Semimonad` / `Monad`) + full Ref hierarchy.
- `RcFreeExplicitBrand`: `Pointed` on the by-value side; full
  Ref hierarchy (`RefFunctor` / `RefSemimonad` / `RefMonad`,
  plus `RefPointed` and the supporting Ref traits per
  [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md)).
- `ArcFreeExplicitBrand`: `SendPointed` on the by-value side
  (added by step 6 alongside `SendFunctor` etc.); full SendRef
  hierarchy (`SendRefFunctor` / `SendRefSemimonad` /
  `SendRefMonad`, plus the supporting `SendRef*` traits).

### Inherent-method fallback

The remaining by-value operations (`bind`, `map`, etc.) on
`RcFreeExplicit` / `ArcFreeExplicit` ship as inherent methods
with their natural `Clone` bounds, mirroring the
`RcCoyoneda`/`ArcCoyoneda` precedent.

### Alternatives considered and rejected

- Modifying the existing by-value hierarchy to add `Clone`
  bounds taxes the entire ecosystem (`Option`, `Vec`,
  `Identity`, etc.) for one wrapper's storage strategy.
- Adding a parallel `CloneFunctor` / `CloneSemimonad` /
  `CloneMonad` family duplicates the Ref hierarchy's dispatch
  story and adds a third orthogonal trait-and-dispatch axis
  (closure shape, send-ness, Clone-ness). The Ref path is the
  documented library convention and exists today; revisit
  `CloneFunctor` only if Phase 5+ user feedback indicates
  Ref-only brand UX is insufficient for the multi-shot Explicit
  family.

### Plan-level consequences

The decision is reflected in Phase 1 step 7, Phase 2 step 4, the
Motivation section's multi-shot example, and the "Will change"
table's `*RunExplicitBrand` row. Step 7 also schedules an update
to
[fp-library/docs/limitations-and-workarounds.md](../../../fp-library/docs/limitations-and-workarounds.md)'s
"Unexpressible Bounds" classification table to add rows for the
three Explicit Free variants once their impls land.

## Resolved earlier: Erased / Explicit dispatch split for the Free family

The earlier `RcFreeBrand` / `ArcFreeBrand` blocker is resolved by
adopting the Erased/Explicit dispatch split documented in
[decisions.md](decisions.md) section 4.4: the Erased family
(`Free`, `RcFree`, `ArcFree`) is inherent-method only and is not
Brand-dispatched, while the Explicit family (`FreeExplicit`,
`RcFreeExplicit`, `ArcFreeExplicit`) carries the full Brand
hierarchy. Phase 1 grows by three steps to add the two new
Explicit Rc/Arc siblings and the `SendFunctor` trait family;
Phase 2 grows the Run surface to six concrete types (one per Free
variant) plus an `into_explicit` / `from_explicit` conversion
API. See plan.md's resequenced phasing.

## Design-phase blockers (resolved in decisions.md)

All blockers from the design phase are resolved in
[decisions.md](decisions.md):

- Section 4 (six DECISIONs): row encoding, Functor dictionary,
  stack-safety, six-variant Free family with Erased/Explicit
  dispatch split, scoped-effect representation (heftia dual row),
  natural transformations as values.
- Section 9 (nine pre-implementation decisions): target audience,
  partial interpretation, async, IO/Effect story, higher-order
  effects, performance, lifetime constraints, macro
  infrastructure, testing strategy.
