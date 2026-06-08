# Research progress tracker

Tracks which research documents are complete and which are still
pending. Update this whenever you finish (or explicitly defer) a
research doc. See [README.md](README.md) for the full protocol.

**Status: research phase REOPENED as of 2026-04-25.** The original 11-codebase Stage 1 plus its synthesis closed cleanly on 2026-04-24, but the user has expanded scope to cover three additional topics: tyrade (a type-level DSL by Will Crichton), type-level hashing producing a type-level result (a deduplication-based alternative to sorting), and type-level hash-map / hash-set (also deduplication-based). Five new Stage 1 entries scheduled below. The synthesis is unticked and will be re-done once the new classifications complete.

## Stage 1: per-codebase classification

Each file below is a brief classification doc (~1200 words max)
against the eight approaches catalogued in [README.md](README.md).

### Direct sorting implementations

- [x] [type-level-sort.md](type-level-sort.md)
- [x] [typelist.md](typelist.md)

### Foundational primitives

- [x] [typenum.md](typenum.md)
- [x] [frunk.md](frunk.md)
- [x] [static-assertions-rs.md](static-assertions-rs.md)

### Hash-based type identity

- [x] [stabby.md](stabby.md)
- [x] [type-uuid.md](type-uuid.md)

### Type-level DSLs and operators

- [x] [rust-type-freak.md](rust-type-freak.md)
- [x] [typ.md](typ.md)
- [x] [tstr-crates.md](tstr-crates.md)

### Marker-trait approaches

- [x] [spidermeme.md](spidermeme.md)

### Type-level DSL (added in expansion)

- [x] [tyrade.md](tyrade.md)

### Type-level hashing (added in expansion)

- [x] [fixed-type-id.md](fixed-type-id.md)

### Type-level hash-map / hash-set (added in expansion)

- [x] [anymap.md](anymap.md)
- [x] [typemap-meta.md](typemap-meta.md)
- [x] [rust-typemap.md](rust-typemap.md)

## Stage 1 synthesis

Populate only after every entry above is ticked. Reopened on
2026-04-25; rewrite folded in the five new classifications and
restated the verdict (no decisions edits, two low-priority optional
notes).

- [x] [\_classification.md](_classification.md)

## Stage 2: deep dives

Populated after Stage 1 synthesis identifies which approaches (if
any) warrant deeper investigation. Each entry becomes a new
`deep-dive-<topic>.md` file in this directory.

_(None scheduled yet.)_
