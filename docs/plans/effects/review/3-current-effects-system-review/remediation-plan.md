# Effects System Review Remediation Plan, 2026-05-18

## Purpose

This plan converts the findings from
[`effects-system-review.md`](effects-system-review.md) into concrete work. The
goal is to improve the current effects system without slipping back into
status-quo-preserving patches that accumulate architectural debt.

The plan prioritizes:

1. Correct user-facing documentation and naming.
2. Better macro diagnostics and row contracts.
3. Ergonomic helper APIs that exercise the existing semantics.
4. Deferred, explicit decisions for work that would otherwise create hidden
   runtime or type-system commitments.
5. Keeping the review document current as a living architecture assessment,
   not as a status log.

## API Stability Stance

`fp-library` is pre-1.0. API-breaking changes are acceptable when they lead to
a better end state. This remediation plan prioritises design correctness,
internal coherence, and long-term architecture over technical-debt-accruing
compatibility with the in-progress effects API.

When a local compatibility-preserving fix conflicts with a cleaner architecture,
choose the cleaner architecture unless a concrete Rust type-system, safety, or
proc-macro limitation prevents it. If that happens, document the limitation,
trade-offs, and fallback before adopting the fallback.

## Current Blocker Status

There are no blockers that prevent beginning remediation. The remaining
fallbacks and policy gates are summarized in
[Fallbacks And Policy Gates](#fallbacks-and-policy-gates). If an adopted
direction hits a concrete Rust type-system or proc-macro limitation, record the
limitation in the review before using the documented fallback.

## Document Maintenance Rules

Plan updates should keep this file short and action-oriented. When a decision,
analysis, or option comparison has been converted into concrete implementation
steps, remove the duplicated analysis from this plan and keep only the
actionable step, fallback, or policy gate that still matters. Do not preserve
old option catalogs as historical status.

If a plan edit makes a section irrelevant, delete that section in the same
edit. If a section remains necessary, rewrite it around the current state
rather than adding "resolved" notes below stale prose.

[`effects-system-review.md`](effects-system-review.md) must be kept current
throughout this remediation work. Each implementation step should update the
review at the same time as the code or plan change that affects it.

The review is not a changelog. Do not append progress notes that leave stale
analysis in place. When a finding is fixed, remove the now-irrelevant analysis
or replace it with an updated assessment of the current architecture. When an
issue changes shape, rewrite that section so it describes the present state,
remaining limitations, and any new recommendations.

Done criteria for every step:

- The review document accurately describes the current implementation after
  the step lands.
- Obsolete findings, examples, and recommendations are removed rather than
  preserved as historical status updates.
- Any still-open issue keeps current approaches, trade-offs, recommendation,
  and reasoning.
- Historical trace belongs in commit messages, plan progress notes, or git log;
  the review document should remain an up-to-date architecture review.

## Concrete Work Plan

### Step 1. Audit Documentation Examples Using The Inventory

Review trace:
[`effects-system-review.md`](effects-system-review.md#limitations-and-inconsistencies),
recommendation 4.

Tasks:

- Use [`item-inventory.md`](item-inventory.md) to find rows with generic
  fallback descriptions or descriptions that had to skip doctest snippets.
- Prioritize public modules, public functions, and smart constructors.
- Replace shape-only examples with usage examples that exercise actual
  semantics and contain assertions over expected output.
- Keep examples self-contained.

Done criteria:

- Public-facing entries no longer depend on generic fallback descriptions in
  the generated inventory.
- `just doc` passes.

### Step 2. Add Natural-Order Handler Builders And Explicit Prepend APIs

Review trace:
[`effects-system-review.md`](effects-system-review.md#limitations-and-inconsistencies);
recommendation 2; decision [D2](#d2-handler-builder-ordering).

Tasks:

- Add natural-order first-order and scoped handler builders for manual use.
  The concrete public shape should be:
  - `handlers_ordered().on::<Brand, _>(handler).finish()`
  - `scoped_handlers_ordered().on::<ScopedBrand, _>(handler).finish()`
- The natural-order builders must preserve written order in the resulting
  handler-list shape, so `A` then `B` produces `A` at the head and `B` in the
  tail, with the scoped builder following the same rule.
- Keep low-level cons-list construction available, but expose it with explicit
  prepend vocabulary:
  - `nt().prepend::<Brand, _>(handler)`
  - `scoped_nt().prepend::<ScopedBrand, _>(handler)`
- Migrate docs and tests away from prepend `.on(...)` so `.on(...)` means
  natural-order builder composition wherever it is public. Because the effects
  API is pre-1.0, do not add compatibility aliases unless a concrete migration
  issue requires a short-lived internal shim.
- Document `handlers!` and `scoped_handlers!` as the primary path for normal
  users, `handlers_ordered()` / `scoped_handlers_ordered()` as the explicit
  manual fallback, and `nt().prepend(...)` / `scoped_nt().prepend(...)` as the
  low-level representation path.
- Add focused type-shape tests proving natural-order builders and prepend
  builders produce the expected head/tail order.

Done criteria:

- Manual builder composition has an explicit, documented order model.
- Users are steered toward `handlers!` / `scoped_handlers!` for the common
  path.
- Public `.on(...)` examples no longer demonstrate prepend semantics.

### Step 3. Schedule Generic Scoped Row Support As A Separate Macro

Review trace:
[`effects-system-review.md`](effects-system-review.md#missing-or-incomplete-areas);
decision [D3](#d3-generic-scoped-row-support).

Tasks:

- Keep `define_scoped_row!` concrete-only for the immediate remediation pass.
- Add a later implementation step for a separate generic scoped-row item macro,
  using [D3](#d3-generic-scoped-row-support) as the target direction.
- The later step must design syntax, lifetime/type/where-clause handling, and
  recursive `Self` replacement tests before implementation.

Done criteria:

- The project has an explicit decision instead of an implicit macro error being
  the whole policy.

### Step 4. Keep Runtime-Heavy Ports Deferred Behind Policy

Review trace:
[`effects-system-review.md`](effects-system-review.md#upstream-port-candidates),
recommendation 3; decision [D5](#d5-runtime-heavy-upstream-ports).

Tasks:

- Do not port `CC`, `Shift`, `Parallel`, `Timer`, `Stream`, `Subprocess`, or
  `Unlift` until runtime policy exists.
- Write the policy before any of these are scheduled:
  - async executor or blocking-thread model;
  - cancellation semantics;
  - IO embedding;
  - process lifecycle ownership;
  - target-monad or continuation-exposure policy;
  - `Send + Sync` requirements.

Done criteria:

- Runtime-sensitive effects remain explicitly deferred.
- Any future plan step that introduces them links to the runtime policy.

## Fallbacks And Policy Gates

This section keeps only decisions that still affect implementation after the
option analysis has been folded into concrete steps.

### D2. Handler Builder Ordering

Step 2 adopts both parts of the builder decision: natural-order manual builders
for user-written `.on(...)` chains, plus explicit `prepend` vocabulary for the
low-level cons-list path.

### D3. Generic Scoped Row Support

Step 3 keeps `define_scoped_row!` concrete-only for now and schedules generic
scoped rows as a separate item macro with its own syntax and tests.

### D5. Runtime-Heavy Upstream Ports

Step 4 defers `CC`, `Shift`, `Parallel`, `Timer`, `Stream`, `Subprocess`,
`Unlift`, and similar effects until async, IO, cancellation, process lifecycle,
continuation-exposure, target-monad, and `Send + Sync` policy exists.

### D6. Helper Family Implementation Pattern

`#[document_module]` now owns a narrow `documented_helper_impls!` item marker
for documented helper impl blocks. The marker is deliberately constrained: it
only accepts Rust impl blocks, removes the marker before output, and runs the
expanded methods through the normal documentation validation and generation
passes.

Use it for helper families when it reduces wrapper drift without hiding the
public API shape. The Except helper rollout stayed explicit across the wrapper
families because the marker would only have wrapped hand-written impls with
different bound surfaces. Use private support helpers only when they remove real
handler-body or conversion duplication. Do not use plain `macro_rules!` for
public helper methods unless generated methods become visible to
`#[document_module]` validation.

## Suggested Implementation Order

1. Step 1: inventory-driven documentation example audit.
2. Step 2: natural-order builders plus explicit prepend APIs.
3. Step 3: schedule generic scoped row support as a separate macro.
4. Step 4: keep runtime-heavy ports deferred until policy work is scheduled.

## Verification Expectations

For each implementation step:

- Run focused tests for the touched macro/effect/wrapper first.
- Update [`effects-system-review.md`](effects-system-review.md) so it replaces
  stale analysis with the current assessment.
- Run `just fmt`.
- Run `just filtered doc '^(error|warning|[[:space:]]*-->|Documenting|Finished|Generated|could not|broken|invalid|unresolved)'`.
- Run `just verify` before merging larger API or macro changes.

## Traceability

This plan is derived from:

- [`effects-system-review.md`](effects-system-review.md)
- [`item-inventory.md`](item-inventory.md)

Future implementation commits should cite the relevant step and decision ID in
commit bodies or plan updates when they close a decision.
