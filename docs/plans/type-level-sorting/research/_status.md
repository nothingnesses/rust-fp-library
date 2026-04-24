# Research progress tracker

Tracks which research documents are complete and which are still
pending. Update this whenever you finish (or explicitly defer) a
research doc. See [README.md](README.md) for the full protocol.

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

## Stage 1 synthesis

Populate only after every entry above is ticked.

- [ ] [\_classification.md](_classification.md)

## Stage 2: deep dives

Populated after Stage 1 synthesis identifies which approaches (if
any) warrant deeper investigation. Each entry becomes a new
`deep-dive-<topic>.md` file in this directory.

_(None scheduled yet.)_
