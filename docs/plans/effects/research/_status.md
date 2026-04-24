# Research progress tracker

Tracks which research documents are complete and which are still pending.
Update this whenever you finish (or explicitly defer) a research doc. See
[README.md](README.md) for the full protocol.

**Status: research phase complete as of 2026-04-24.** All Stage 1 per-codebase classifications, the Stage 1 synthesis, and the three priority Stage 2 deep dives are ticked. The one remaining Stage 2 entry (compile-time indexing) has been reclassified as an implementation-phase investigation; see the note under "Stage 2: deep dives" below. Next phase: apply research findings to implementation per the port-plan's section 6 roadmap.

## Stage 1: per-codebase classification

Each file below is a brief classification doc (~1500 words max) against
the five encodings catalogued in [../port-plan.md](../port-plan.md)
section 4.1.

### Haskell effect libraries

- [x] [freer-simple.md](freer-simple.md)
- [x] [polysemy.md](polysemy.md)
- [x] [fused-effects.md](fused-effects.md)
- [x] [eveff.md](eveff.md)
- [x] [mpeff.md](mpeff.md)
- [x] [heftia.md](heftia.md)
- [x] [in-other-words.md](in-other-words.md)

### Non-Rust languages

- [x] [effekt.md](effekt.md)
- [x] [koka.md](koka.md)

### Rust effect crates

- [x] [corophage.md](corophage.md)
- [x] [effing-mad.md](effing-mad.md)
- [x] [reffect.md](reffect.md)
- [x] [fx-rs.md](fx-rs.md)

## Stage 1 synthesis

Populate only after every entry above is ticked.

- [x] [\_classification.md](_classification.md)

## Stage 2: deep dives

Populated after Stage 1 synthesis identifies which codebases (if any)
warrant deeper investigation. Each entry becomes a new
`deep-dive-<topic>.md` file in this directory.

- [x] [deep-dive-evidence-passing.md](deep-dive-evidence-passing.md): can Rust host typed handler-vector dispatch (EvEff / Koka indexing) without delimited continuations? Priority 1.
- [x] [deep-dive-coroutine-vs-free.md](deep-dive-coroutine-vs-free.md): do coroutines alone preserve the first-class-program properties section 4.4 requires, or is a Free wrapper still needed? Priority 2.
- [x] [deep-dive-scoped-effects.md](deep-dive-scoped-effects.md): which scoped-effect pattern (heftia dual row, in-other-words Effly, polysemy Tactical, freer-simple interposition) ports most cleanly to Rust? Priority 3.

Deferred to Option 4 implementation (no longer classified as Stage 2 research):

- `deep-dive-compile-time-indexing.md` (original title). Prototype a proc-macro that emits a const `[usize; N]` index table alongside the coproduct expansion, extending corophage's `Effects![...]`, and measure compile-time and dispatch cost vs a coproduct pattern-match baseline. This is not research: it cannot be answered without building an Option 4 prototype to A/B against. Reclassified as an implementation-phase investigation to run during Option 4 build-out. The evidence-passing deep dive (section 5.2 of research/deep-dive-evidence-passing.md) is the motivating context.
