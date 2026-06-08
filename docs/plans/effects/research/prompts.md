# Prompts for resuming paused effects research

Paste one of the prompts below into a new session to continue research
where the last session stopped. Every prompt is self-contained; agents
receiving them do not need the originating conversation history.

Each prompt is a copy-paste block. Placeholders marked `<LIKE_THIS>`
should be substituted before pasting.

---

## Prompt 1: Continue the Stage 1 queue (most common)

Use this when you want to make progress on whatever is pending, without
caring which specific codebases. Replace `<N>` with how many codebases
you're budgeting for this session (typically 1 to 3 depending on your
rate-limit budget).

```
Continue the paused effects-research work. Open
`docs/plans/effects/research/_status.md`. Find the first <N> unchecked
`[ ]` items under "Stage 1: per-codebase classification". For each
pending item, in parallel:

1. Read the template at
   `docs/plans/effects/research/<codebase>.md` where `<codebase>` is
   the filename corresponding to the unchecked item.
2. Read the codebase source at the path listed in that file's header
   (respecting case sensitivity: `EvEff` and `MpEff` use their
   capitalised directory names).
3. Fill every required subsection of the template with findings
   grounded in source references (file paths and line numbers into
   the codebase).
4. Update the file's `Status:` field to `complete` and set
   `Last updated:` to a date or descriptive marker.
5. Tick the corresponding checkbox in `_status.md`.

Agents must respect the per-file word budget (~1500 words excluding
template boilerplate) and the contract in
`docs/plans/effects/research/README.md`. Do NOT modify
`../decisions.md` directly; note any plan-relevant findings under the
"Relevance to decisions" subsection of the research file.

After all <N> agents complete, report a brief summary of what each
found: the classification against options 1-5 and whether the codebase
was novel or a variant. I will then decide whether to continue or end
the session.
```

---

## Prompt 2: Research a specific codebase

Use this if you want to target a particular codebase out of order (for
example, to prioritise a novelty you suspect is lurking). Replace
`<CODEBASE>` with the filename stem, such as `eveff` or `heftia`.

```
Research the `<CODEBASE>` codebase for the effects port. Open
`docs/plans/effects/research/<CODEBASE>.md` for the template, codebase
path, and required subsections. Read the codebase source at the path
listed in the file's header.

Fill every required subsection with findings grounded in source
references (file paths and line numbers). Respect the ~1500-word
budget and the contract in
`docs/plans/effects/research/README.md`. Do NOT modify
`../decisions.md` directly; note any plan-relevant findings under the
"Relevance to decisions" subsection of the research file.

When complete, update the file's `Status:` to `complete`, set
`Last updated:`, and tick the corresponding checkbox in
`docs/plans/effects/research/_status.md`.
```

---

## Prompt 3: Synthesise Stage 1

Use this after every per-codebase file has been ticked in `_status.md`.
It produces `_classification.md`, the aggregated Stage 1 findings that
drive the Stage 2 decision.

```
Stage 1 per-codebase research is complete: every entry under
`docs/plans/effects/research/_status.md`'s "Stage 1: per-codebase
classification" should now be ticked. Verify that by opening the
status file before starting.

Write `docs/plans/effects/research/_classification.md` synthesising
findings across all 13 per-codebase files in the same directory.

Required structure:

1. Brief intro: what Stage 1 set out to answer.
2. Classification table: one row per codebase, columns for core
   substrate, classification against decisions section 4.1's options
   1-5, and a one-line novelty verdict.
3. "Novel encodings" section: for each codebase flagged genuinely
   novel, briefly sketch what makes it novel and why it deserves a
   Stage 2 deep dive.
4. "Variants of known options" section: one paragraph per option
   family summarising which codebases cluster there.
5. "Recommendations for Stage 2" section: name which (if any) deep
   dives are worth scheduling and what specific questions each should
   answer.
6. "Relevance to decisions" section: which sections of
   `../decisions.md` would change based on Stage 1 findings, or "no
   changes recommended" explicitly if that is the conclusion.

Word budget: ~2500 words. Do NOT modify `../decisions.md` directly.

When complete, update `docs/plans/effects/research/_status.md` to
tick the `_classification.md` checkbox and add any Stage 2 deep dives
as new unchecked entries under "Stage 2: deep dives".
```

---

## Prompt 4: Stage 2 deep dive

Use this for each deep dive identified by Stage 1's synthesis. Replace
`<TOPIC>` with a kebab-case name, such as `eveff-evidence-passing`;
replace `<CODEBASE>` with the directory name at
`/home/jessea/Documents/projects/effects/<CODEBASE>/`.

Before running this prompt, create the target file
(`docs/plans/effects/research/deep-dive-<TOPIC>.md`) with at least a
`Status: pending` header; the agent fills in the rest.

```
Stage 2 deep dive on `<TOPIC>`. Read
`docs/plans/effects/research/_classification.md` for context on why
this topic was flagged novel, and any upstream per-codebase file such
as `docs/plans/effects/research/<CODEBASE>.md` for the Stage 1
observations that motivated this dive. Read the relevant source at
`/home/jessea/Documents/projects/effects/<CODEBASE>/` deeply enough
to answer the questions below.

Fill `docs/plans/effects/research/deep-dive-<TOPIC>.md` with:

1. Purpose and scope: the specific question this deep dive answers.
2. Detailed mechanism: how the encoding actually works. Annotated
   code excerpts with line numbers are welcome.
3. Rust portability assessment: could this encoding be expressed in
   Rust given the `fp-library` Brand/Kind machinery? What would
   block it? What would it cost in complexity, unsafe code, or
   performance?
4. Comparison against the five options in `../decisions.md` section
   4.1. Does this warrant a sixth option, or does it refine an
   existing one?
5. Concrete plan-edit recommendations, if any.

Word budget: ~2000 words. Do NOT modify `../decisions.md` directly;
recommend plan edits in section 5 of the deep-dive doc instead.

When complete, set `Status: complete` in the deep-dive file, tick
its checkbox in `_status.md`, and add any follow-up research ideas
as new unchecked entries under "Stage 2: deep dives".
```

---

## Prompt 5: Ad-hoc research agent

Use this when the work does not match one of the prompts above but
should still follow the research directory's conventions (for example,
a cross-cutting question that spans several codebases, or a re-check
of a finding).

```
Research task: <DESCRIBE THE QUESTION IN ONE OR TWO SENTENCES>.

Work within `docs/plans/effects/research/`. Follow the agent contract
in `docs/plans/effects/research/README.md`: files descriptive, not
prescriptive; claims grounded in source references; plan edits
recommended via "Relevance to decisions" subsections, not executed.

If this work produces a new research artefact, name the file
`docs/plans/effects/research/<sensible-name>.md` and add an
appropriate entry to `_status.md`. Word budget: ~1500 words unless
the question clearly needs more.
```
