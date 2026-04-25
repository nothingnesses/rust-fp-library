# tyrade

**Status:** complete
**Last updated:** 2026-04-25
**Codebase location:** `/home/jessea/Documents/projects/type-level/tyrade/`

## Purpose

Stage 1 research document: classify `tyrade` against the ten
type-level approaches catalogued in [README.md](README.md). Tyrade is
Will Crichton's type-level DSL for Rust; the survey already classified
the conceptually-similar `typ` (jerry73204), so the focus here is
whether tyrade differs materially in expressiveness, scope, or
applicability to coproduct row canonicalisation.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

Tyrade is a procedural-macro DSL enabling type-level computation via
a pure functional syntax. Users write recursive functions and algebraic
data types within a `tyrade! { }` macro, using `fn`, `match`, and
function calls that mimic ordinary Rust. The macro translates each
function into a trait definition plus concrete trait impls, one per
match branch (README.md:13-33, trans.rs:104+). The DSL generates
correct where-clauses and trait bounds automatically during translation.
Tyrade ships a standard library (tcore, tnum, tbool, tlist) providing
type-level operations on Peano numerals (`Z`, `S(n)`), booleans, and
heterogeneous lists. Unlike `typ`, tyrade does not integrate `typenum`;
instead it defines its own numeric primitives via enums and trait
recursion (tnum.rs).

### Type-level sorting capability

No built-in sort exists; however, the DSL is Turing-complete for
type-level computation. Users can express recursive list operations
including `TListNth` (index access) and `TListMap` (homogeneous
transformation) via match patterns (tlist.rs:11-43). Comparison
primitives exist only for Peano numerals: `TLessThanEqual` (tnum.rs:28-35)
demonstrates comparison recursion. A full sort would require a
comparison operator applicable to arbitrary user types, which the DSL
does not provide. Thus sorting on custom type tags is inexpressible
without extending the DSL to support type-level comparison of
non-numeric types (e.g., string or hash-based tags). Deduplication via
pattern matching is expressible, but ordering by semantic tags requires
primitives beyond the current standard library.

### Approach used (or enabled)

Approach 2 (proc-macro textual canonicalisation), but incomplete for
sorting. Tyrade excels at approach 2 for DSL definitions: the macro
accepts textual input and emits trait code, enabling canonicalisation
of type definitions _via_ lexical ordering of variant names or clauses
in the DSL itself. However, the coproduct row problem requires
_runtime_ sorting of effect labels, which would demand type-level
comparison (approach 1 or 5) as an additional layer. Tyrade provides
no turnkey mechanism for this; a user would need to manually encode
effect comparison as a type-level predicate outside the DSL, then
compose it with Tyrade-generated operations.

### Stable or nightly

Nightly-only via `feature(specialization)` (lib.rs:1). The macro
itself (2021 edition, tyrade-macro/Cargo.toml) poses no additional
feature gate, but the standard library and most examples require
specialization to compile. The comment at tnum.rs:45 flags that
`TDivide` causes stack overflow when specialization is enabled,
suggesting reliability issues under nightly assumptions. Edition 2021
(Cargo.toml) represents recent Rust, but specialization adoption
blocks stable deployment.

### Ergonomics and compile-time profile

Tyrade is significantly more ergonomic than `typ`. Users write
straightforward `fn` syntax with pattern matching, closely mirroring
domain logic. Example: `TAdd` (README.md:21-26) is visually identical
to a runtime recursive addition function. No manual trait definition or
impl boilerplate is needed. Compile-time profile is not explicitly
documented. The codebase is small (< 1000 LOC including tests), and no
benchmarks or compile-time measurements are provided. The presence of
stack overflow bugs under specialization (tnum.rs:45-51) suggests
potential for expensive trait expansion in recursive cases.

### Production status

Experimental proof-of-concept, last active around 2018. The GitHub
repository (cited in README.md:35) carries a "proof-of-concept"
label. No version tags, CHANGELOG, or release notes are present;
codebase appears unmaintained. The blog post reference
(README.md:35, willcrichton.net/notes/type-level-programming/)
is the authoritative external documentation.

### Differences from `typ`

Both are proc-macro DSLs enabling type-level computation. Tyrade is
more compact and readable: it avoids `typ`'s requirement to annotate
types with typenum bounds, instead defining its own numeric type
system from scratch. Tyrade integrates first-class list and boolean
primitives in the standard library; `typ` relies on users composing
`typenum` and custom list structures. Conversely, `typ` benefits from
typenum's richer numeric operations; Tyrade's Peano system is simpler
but less expressive for non-trivial arithmetic. Tyrade uses
specialization, while `typ` avoids nightly; this is a fundamental
design trade-off. Neither is a successor to the other; they are
independent experiments addressing the same ergonomic problem via
different feature-gate trade-offs.

### Applicability to coproduct row canonicalisation

Poor fit. Tyrade excels at describing type-level transformations
_within_ a DSL, but does not address comparison of arbitrary types
outside the DSL. To canonicalise a coproduct row, a user would need to
write a custom type-level comparison function for effect identifiers
(outside Tyrade), then compose it with a Tyrade-generated sort. This
is possible but adds a layer of manual trait work that defeats the
DSL's ergonomic benefit. Simpler approaches (lexical canonicalisation
at macro time, or hash-based identity) avoid Tyrade entirely.

### References

- `/home/jessea/Documents/projects/type-level/tyrade/README.md`: overview, examples, design
- `/home/jessea/Documents/projects/type-level/tyrade/src/lib.rs:1`: specialization gate
- `/home/jessea/Documents/projects/type-level/tyrade/src/tnum.rs:28-35, 45-51`: comparison ops, known bugs
- `/home/jessea/Documents/projects/type-level/tyrade/src/tlist.rs:11-43`: list operations
- `/home/jessea/Documents/projects/type-level/tyrade/tyrade-macro/src/trans.rs:104+`: macro translation logic
- `/home/jessea/Documents/projects/type-level/tyrade/tyrade-macro/Cargo.toml`: edition 2018
- `/home/jessea/Documents/projects/type-level/tyrade/Cargo.toml`: edition 2021

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1200 (excluding this template boilerplate)
