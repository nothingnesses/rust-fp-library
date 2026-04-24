# Effects research directory

Supporting research for the extensible-algebraic-effects port at
[../port-plan.md](../port-plan.md). Each file in this directory records
findings from reading one codebase or pursuing one question. Files are
descriptive (what is true about the source) rather than prescriptive (what
the project will do); the plan remains the authoritative decision document.

## Why this directory exists

A focused survey of 13 effect-system codebases cloned at
`/home/jessea/Documents/projects/effects/`. The research is split into
small per-file tasks so it can be paused and resumed across sessions
without losing state. See [`_status.md`](_status.md) for the task queue.

## File layout

- `_status.md` — task tracker with checkboxes. Read this first to resume.
- `_classification.md` — aggregated Stage 1 findings; populated only after
  every per-codebase file is complete.
- `<codebase>.md` — one per codebase, Stage 1 classification.
- `deep-dive-<topic>.md` — Stage 2, added only for codebases flagged as
  genuinely novel in Stage 1. Shape defined ad-hoc per topic.

Underscore-prefixed files are meta-files (tracking, indices, synthesis);
files without an underscore are content.

## Agent contract

Every file in this directory must:

1. Begin with a YAML-ish header block containing at minimum:
   - `Status:` one of `pending`, `in-progress`, `complete`.
   - `Last updated:` a date or descriptive marker.
2. Fill every required subsection of the template. If a section does not
   apply, write "not applicable" or "not documented in source" explicitly;
   do not leave blank headers.
3. Ground every non-trivial claim in a source reference: file path plus
   line number into the relevant codebase at
   `/home/jessea/Documents/projects/effects/<name>/`.
4. Respect the per-file word budget noted in the template (typically 1500
   words excluding the template boilerplate).
5. When completing a file, update its `Status:` to `complete` AND tick the
   corresponding checkbox in [`_status.md`](_status.md).

Agents writing here should not modify `../port-plan.md` directly. If a
finding recommends a plan change, note it under "Relevance to port-plan"
within the research file; the plan edit happens as a separate human-driven
step after the research is reviewed.

## Resume protocol

1. Open [`_status.md`](_status.md).
2. Find the first unchecked `[ ]` entry under Stage 1.
3. Launch one agent per pending file, typically 1-3 at a time based on
   rate-limit budget.
4. Each agent reads its assigned codebase and fills the corresponding
   template, updating both the file's `Status:` and the checkbox in
   `_status.md`.
5. Session can end at any point. Next session resumes at step 1.
6. When every Stage 1 entry is ticked, an aggregation agent writes
   `_classification.md`. Stage 2 deep dives are scheduled from there.
