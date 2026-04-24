# Research progress tracker

Tracks which research documents are complete and which are still pending.
Update this whenever you finish (or explicitly defer) a research doc. See
[README.md](README.md) for the full protocol.

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
- [ ] [koka.md](koka.md)

### Rust effect crates

- [ ] [corophage.md](corophage.md)
- [ ] [effing-mad.md](effing-mad.md)
- [ ] [reffect.md](reffect.md)
- [ ] [fx-rs.md](fx-rs.md)

## Stage 1 synthesis

Populate only after every entry above is ticked.

- [ ] [\_classification.md](_classification.md)

## Stage 2: deep dives

Populated after Stage 1 synthesis identifies which codebases (if any)
warrant deeper investigation. Each entry becomes a new
`deep-dive-<topic>.md` file in this directory.

_(None scheduled yet.)_
