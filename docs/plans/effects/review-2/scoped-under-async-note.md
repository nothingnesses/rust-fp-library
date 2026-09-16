# Scoped-under-async design note

Written for item 18 of the remediation plan, on the shipped async surface
(the OQ-18A terminal composition: the row-generic `await_future`
constructor and the `run_async` terminal driver, with mixed rows narrowed
by the sync runner tier first). It extends the W13 continuation-as-data
policy record with the scoped-semantics half that record left open.

## The question

The prior-generation (dual-row) async design had to specify how scoped
effects interact with a driver that awaits through scoped boundary frames.
FS-1 has no boundary frames: scoping is elaboration inside a synchronous
fold, and the driver is terminal. What remains to specify is how scoping
constructs behave when `Await` cells pass through them, on each of the two
interpretation tiers.

## The one-pass surface: excluded by construction

`AwaitBrand` carries no `HandlerPieces` implementation, deliberately:
interpreting `Await` is a suspension an async driver awaits, not an arm a
synchronous `Result`-returning loop can apply. A `#[handlers]` row
containing `AwaitBrand` therefore fails to compile with the clear
missing-`HandlerPieces` bound error, pinned by
`fp-library/tests/ui/handlers_row_rejects_await.rs`. Consequently a
higher-order arm's re-entry (`RowHandler::handle` on an owned sub-program)
can never encounter a suspension, and no scoped-under-async semantics
arise on this tier at all. This is the honest boundary rather than a
limitation to engineer around; an async twin of the handler surface was
considered and rejected under OQ-18A (it would duplicate the dispatch
loop and thread the row abort through a second vocabulary).

## The narrowing tier: scopes span suspensions unchanged

On the narrowing tier a scope is part of a synchronous fold:
`transact_state`'s local accumulator, the scoped `Choose` cell's owned
branches, a per-branch accumulator under `handle_choose_accum`. To such a
fold an `Await` cell inside the scoped region is simply an unmatched
cell: the runner re-emits it into the residual row with the rest of the
scoped fold riding inside the re-emitted continuation. The scope
therefore spans the suspension with its semantics unchanged; nothing in a
scope's definition refers to suspension at all.

Evidence, in `fp-library/tests/effects_async.rs`:

- The state fold resumes across suspensions: an interleaved
  `Await`-plus-`State` program narrows through `handle_state` and drives
  to the oracle on both a busy-poll executor and the Tokio scheduler.
- A transactional scope spans a suspension: the transaction opens before
  the embedded future, its local write is read back on the far side of
  the await, and the commit lands only once the driver resumes past the
  suspension. Rollback remains structural: an abort discards the
  continuation carrying the commit, whether or not suspensions sit inside
  the region.

## Ordering: the driver is outermost by construction

`run_async` consumes the `Await`-only residual and returns a value, so no
runner can stack outside it. Every ordering decision (which scope
contains which, which runner sees which effects) is a sync-tier decision
made before the driver, identical to the fully synchronous stacks; the
async surface adds no new ordering axis to document or misuse.

## Scoped choice: branches sequence; racing is `Parallel`'s

Branches of the scoped `Choose` cell that contain `Await` cells re-emit
them in branch order, so the driver awaits them sequentially. Running
branches concurrently (racing or joining their futures) is not a
scoped-choice property but the deferred `Parallel` runtime-sensitive
effect, owned by item 18's runtime-policy step; conflating the two would
put a scheduling decision inside a semantics construct.

## Single-shot interaction

Per the adopted multi-shot decision (OQ-18B), async is single-shot-only:
no scope re-enters a continuation across an `Await` more than once, so
the memoize-versus-replay question does not arise inside scopes today. It
returns, for scopes as for everything else, at the multi-shot
interpretation round.

## What dissolved from the prior design

The dual-row design's open item, awaiting through scoped boundary frames,
has no referent under FS-1: there are no frames, scoping is elaboration
in a fold, and folds commute with suspension by lazy re-emission. No
scoped construct needed modification to coexist with the async driver,
and none needs awareness of it.
