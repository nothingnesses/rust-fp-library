# Type-level sorting research directory

Supporting research for the question: do any approaches to type-level
sorting in Rust actually work, and well enough to inform
[../../effects/port-plan.md](../../effects/port-plan.md) section 4.1's
"ordering mitigations" subsection (specifically workaround 2: tag-based
type-level sorting)? Each file in this directory records findings from
reading one codebase or pursuing one question. Files are descriptive
(what is true about the source) rather than prescriptive (what the
project will do); the port-plan remains the authoritative decision
document for the effects port.

## Why this directory exists

A focused survey of 11 type-level-programming codebases cloned at
`/home/jessea/Documents/projects/type-level/`. The research is split
into small per-file tasks so it can be paused and resumed across
sessions without losing state. See [`_status.md`](_status.md) for the
task queue.

This directory was spawned by the effects research; the originating
question is workaround 2 in port-plan section 4.1, currently rejected
on speculative complexity and compile-time grounds. The dive aims to
catalogue the actual state of type-level sorting in Rust, identify
which approaches work and at what cost, and surface anything novel.

## File layout

- `_status.md`: task tracker with checkboxes. Read this first to resume.
- `_classification.md`: aggregated Stage 1 findings; populated only
  after every per-codebase file is complete.
- `<codebase>.md`: one per codebase, Stage 1 classification.
- `deep-dive-<topic>.md`: Stage 2, added only for approaches flagged as
  genuinely promising or surprising in Stage 1.

Underscore-prefixed files are meta-files (tracking, indices, synthesis);
files without an underscore are content.

## The candidate approaches under consideration

The first eight approaches are about ordering / sorting. Approaches 9
and 10, added in a later expansion, are alternative canonicalisation
routes that do not require sorting (they deduplicate by type identity
rather than ordering).

1. **Peano + typenum comparison.** Recursive insertion sort via traits.
   Each effect implements a typenum tag; a recursive trait performs
   insertion sort at type-resolution time.
2. **Proc-macro textual canonicalisation.** Sort effect names lexically
   at macro-expansion time and emit a canonical coproduct order.
3. **Hash-based type tags.** Compile-time hash of `type_name::<T>()` or
   similar, used as a numeric tag drivable by typenum or const generics.
4. **`feature(adt_const_params)` with string const parameters.** Use
   `&'static str` or `[char; N]` as a const generic and compare directly.
5. **`feature(specialization)` / `min_specialization`.** Encode "smaller
   than" via overlapping impls; nightly-only, brittle.
6. **`std::any::TypeId` runtime comparison.** Not type-level proper;
   compare at runtime via stable type identity. Falls back to Option 3
   (TypeId dispatch) in port-plan terminology.
7. **Marker-trait inequality via orphan rules.** Exploit coherence rules
   to prove "two types are different".
8. **Const generics + `const fn` comparison.** Use stable const generics
   and `const fn` to compute order at compile time.
9. **Type-level hashing with type-level result.** Compile-time hash that
   produces a TYPE (not just a runtime const). Two types with the same
   hash collapse to identical type-level identity, enabling
   canonicalisation without ordering. Distinguished from approach 3,
   where the hash is a runtime constant unable to drive trait dispatch
   on stable Rust.
10. **Type-level hash-map / hash-set.** Data structures keyed by type
    identity that deduplicate effects in a row (set) or look up
    per-type values (map), achieving canonicalisation without ordering.
    On stable Rust this typically reduces to runtime `TypeId`-keyed
    storage, which is approach 6 / port-plan Option 3 in disguise; a
    truly type-level realisation would require nightly features.

The classification asks: which approach (or combination) does this
codebase use, and does it actually solve the row-canonicalisation
problem (`Coproduct<A, Coproduct<B, Void>>` and `Coproduct<B,
Coproduct<A, Void>>` resolve to the same type)?

## Agent contract

Every file in this directory must:

1. Begin with a header block containing at minimum:
   - `Status:` one of `pending`, `in-progress`, `complete`.
   - `Last updated:` a date or descriptive marker.
2. Fill every required subsection of the template. If a section does
   not apply, write "not applicable" or "not documented in source"
   explicitly; do not leave blank headers.
3. Ground every non-trivial claim in a source reference: file path
   plus line number into the relevant codebase at
   `/home/jessea/Documents/projects/type-level/<name>/`.
4. Respect the per-file word budget noted in the template (~1200 words
   excluding template boilerplate).
5. When completing a file, update its `Status:` to `complete` AND tick
   the corresponding checkbox in [`_status.md`](_status.md).

Agents writing here should not modify
[../../effects/port-plan.md](../../effects/port-plan.md) directly. If a
finding recommends a plan change, note it under "Relevance to
port-plan" within the research file; the plan edit happens as a
separate human-driven step after the research is reviewed.

## Resume protocol

1. Open [`_status.md`](_status.md).
2. Find the first unchecked `[ ]` entry under Stage 1.
3. Launch one agent per pending file, typically 1-3 at a time.
4. Each agent reads its assigned codebase and fills the corresponding
   template, updating both the file's `Status:` and the checkbox in
   `_status.md`.
5. Session can end at any point. Next session resumes at step 1.
6. When every Stage 1 entry is ticked, an aggregation agent writes
   `_classification.md`. Stage 2 deep dives are scheduled from there.
