# Multi-shot interpretation design note

This note scopes the multi-shot interpretation round (item 22): the
interpretation core and runner tier for the `Rc`/`Arc` store family, the
semantic decisions re-entry makes observable, and the staged plan. Its
adoption is a user gate because the round changes the semantic model: a
captured continuation that can be resumed more than once makes handler-state
duplication, side-effect replay, and finalization order observable in ways
the single-shot `Box` tier structurally rules out. Implementation beyond
item 22's proof-of-concept step stays out of scope until this note is
adopted.

## What exists and what is missing

The substrate's multi-shot half is built and tested but has no interpreter
consuming it:

- `MultiShotStore` (implemented by `RcBrand` and `ArcBrand`) with re-callable
  `Fn` stored closures, shareable erased values, and the O(1)-`Clone`
  catenable queues.
- A multi-shot `Free::to_view` arm already exists with one body generic over
  `S: MultiShotStore`, cloning the continuation queue per branch; `evaluate`
  has the same two-arm split. These are the round's stepping precedent: one
  generic body for the multi-shot family, the `Box` arm separate.
- The three unified constructors (`pure`, `wrap`, `lift_f`) and
  `from_raw_parts` are already store-generic.

What is missing, in dependency order:

- A runner tier off the `Box` pin. Everything in `handle.rs` (the `handle`
  vocabulary, `handle_accum`, `extract`, `run_cont`, and the `#[handlers]`
  seam traits) is `Box`-pinned three ways: the `Free` `Store` default, the
  `.resume()` method (defined only on the `Box` arm), and `bind` (defined
  per store because the stored closure kind differs).
- The raw-stepping API's consumer. The eight kept `free.rs` items
  (`into_raw_step`, `transform_raw`, `cast_erased`, the erased-continuation
  constructors) are `Box`-pinned and were retained for this round; whether
  the multi-shot core needs them generalised, needs multi-shot siblings, or
  needs neither is an open question the proof of concept settles.
- A multi-shot consumer. Nothing in the public surface can yet observe
  re-entry; item 17's `alt` and item 19's `sub_shift_fork` are the two
  probes.

## The stepping core

The multi-shot `to_view` arm is the stepping primitive; the core builds on
it rather than replacing it. Two candidate structures for the core:

1. One generic interpretation body over all three stores, behind a stepping
   trait unifying the `Box` and multi-shot `to_view` arms and a bind seam
   trait carrying the per-store closure bound. Rejected for this round: the
   one-shot and multi-shot runners genuinely differ in semantics (only the
   multi-shot tier forks, memoizes, or replays), so forcing one body hides
   the store axis exactly where it is semantically load-bearing, and the
   unified bind seam is heavy machinery serving no consumer the round needs.
2. A multi-shot tier with one body generic over `S: MultiShotStore`,
   mirroring the existing `to_view`/`evaluate` two-arm precedent; the `Box`
   tier stays untouched. Recommended: the `Rc`/`Arc` pair is where
   duplication would actually hurt (two near-identical bodies differing only
   in `Send` bounds), and the two-arm split keeps the single-shot guard
   (`Free::to_view map called more than once`, the `Cell::take` in the `Box`
   arm) out of the multi-shot path entirely.

The per-store `bind` problem inside a generic multi-shot body has an
established in-crate answer: a per-store construction trait carrying the
closure bound on the impl rather than in the trait signature, the
`ValueFor`/`CoyoLift` pattern (the Rc impl carries `Fn`, the Arc impl
`Fn + Send + Sync`). The proof of concept validates whether the multi-shot
runner body can route its continuations through such a seam (or through the
existing capture-free `ClosureStorage::from_fn` where the continuation
captures nothing); the fallback is per-store runner bodies (`Rc` and `Arc`
copies), documented as the limitation per principle 3.

## The runner tier and the store seam

The multi-shot runner tier reuses the shipped vocabulary, not a new one:
`handle`-family names, narrowing by uninject/embed, threaded accumulators
via the rank-2 `AccumStep` (already `Clone` exactly so forking runners can
give each branch a copy, the `handle_choose_accum` precedent), and the
`#[handlers]` seam traits (`HandlerPieces`, `RowHandler`) extended or
paralleled for the multi-shot stores. The tier's defining new capability is
the forking step: a runner hands its handler a resumable continuation value
(the store's `Fn` form) instead of consuming a `FnOnce`, and the handler may
call it zero, one, or many times, collecting the results.

The raw-stepping API is consumed on demand: the proof of concept and the
`alt` probe decide which of the eight retained items the multi-shot core
actually needs (generalised over the store, or as multi-shot siblings); the
items the round ends without consuming lose their retention reason and are
deleted in the round's hygiene pass rather than kept on a new promise.

## Semantic decisions

### Handler state under re-entry

Invocation-time threading is the default, matching heftia: the state at the
moment a continuation is re-invoked flows forward, with no automatic
rollback. `transact_state` (shipped) is the opt-in rollback; threaded
accumulators fork per re-entry exactly as `handle_choose_accum` forks per
branch; shared-by-reference cells are global across re-entries unless the
handler adopts the MpEff snapshot-and-restore discipline, which this round
documents as the pattern for cell-backed handlers needing per-entry
isolation. No new mechanism is built for this: the decision is that the
existing opt-ins are the model, stated in the guides.

### Async under multi-shot (the OQ-18B revisit)

The folded OQ-18B decision named two mechanisms; the round chooses replay
consistency:

- Thunked re-await is the default shape: the multi-shot `Await` cell stores
  a re-callable future thunk, and each re-entry re-executes the async
  operation. This matches the round's semantics everywhere else (state
  threads per invocation, accumulators fork per branch, continuations
  replay), so re-running the awaited effect is the unsurprising reading.
- Memoize-once (`Shared`-style: one await, the cached value cloned across
  re-entries) is the opt-in caching wrapper, the async analogue of
  `transact_state`'s relationship to threading.

Implementation of either stays deferred to a concrete consumer per the
runtime policy's eligible-on-consumer pattern: the round's probes (`alt`,
`sub_shift_fork`) are synchronous, so building multi-shot async now would
again be unvalidatable machinery. The decision recorded here is the default
mechanism and the opt-in's shape, so the consumer that arrives does not
reopen the semantics.

### Bracket

The CC/Shift note's exclusion is restated as binding for the whole round:
prompt `Bracket` finalization and multi-shot resumption cannot both hold (a
release that ran once cannot be un-run when a continuation re-enters the
bracketed region), so multi-shot `shift`/`alt` and `Bracket` do not share a
row in this round. The exclusion is documentation plus whatever type-level
material the `OrderOf` markers already give; a checked boundary is not
invented for it, and the exclusion is revisited only on a concrete consumer
with a concrete finalization semantics.

## Probe consumers

Two consumers validate the core, in order:

1. Item 17's `alt`: heftia derives NonDet on `Shift` by calling the captured
   continuation once per branch, so the first-order `alt() -> bool` effect
   with a collecting runner is the smallest observable re-entry. Its oracle
   is the item 14 ordering zoo, which pins the branch orderings the
   single-shot tier already exhibits.
2. Item 19's `sub_shift_fork`: the multi-shot `shift` stage, the reified
   continuation as the store's `Fn` form, the fork handler calling the exit
   per branch. Its oracle extends the shipped one-shot `shift` suite.

## What the design gives up

- No single interpretation body over all three stores: the two-arm split is
  chosen deliberately (the store axis is semantic, not incidental), so a
  future store joins by implementing the seam traits, not by inheriting a
  universal body.
- The multi-shot tier requires `Clone` values (`ValueFor` on the multi-shot
  stores already demands it) and re-callable continuations, so programs
  moving non-`Clone` captures stay `Box`-tier-only by type, as today.
- Per-cell effects whose payloads are one-shot by nature (the async
  `Pin<Box<dyn Future>>` cell) do not join the multi-shot row until their
  recorded mechanism (thunked re-await) is built on a consumer.

## One-shot compatibility

The `Box` tier is untouched: no public signature changes, the single-shot
guard stays in the `Box` `to_view` arm only, the shipped runner vocabulary
keeps its meaning, and every existing test and doctest remains the oracle
for it. The multi-shot tier is additive surface.

## Staged implementation plan (on adoption)

1. The stepping proof of concept (item 22's second step): a captured
   `Rc`-store continuation resumed once per branch from one suspension,
   driving a two-branch collection oracle through the multi-shot `to_view`
   arm; it decides the continuation seam (the `ValueFor`/`CoyoLift`-pattern
   bind seam versus per-store bodies) and which raw-stepping items the core
   consumes.
2. The stepping core and the store-generalised runner tier (item 22's third
   step), with the item 14 zoo as the ordering oracle.
3. The probes: item 17's `alt` re-expression, then item 19's multi-shot
   `shift` and fork primitive, each on its own item's steps.
4. Documentation and hygiene (item 22's fourth step): the guides' multi-shot
   model, the OQ-18B fold into item 18's record, and the raw-stepping
   retention pass (consumed items lose their allowances by gaining their
   consumer; unconsumed items are deleted).
