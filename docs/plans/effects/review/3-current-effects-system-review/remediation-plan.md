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

## Current Blocker Status

There are no blockers that prevent beginning remediation. There are, however,
several decisions that should be made before implementing the relevant steps.
Those decisions are listed in [Open Decisions](#open-decisions) with options,
trade-offs, recommendations, and rationale.

## Review Maintenance Rule

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

### Step 1. Fix Stale Public Documentation

Review trace:
[`effects-system-review.md`](effects-system-review.md#limitations-and-inconsistencies),
recommendation 1.

Tasks:

- Update `fp-library/src/types/effects/run.rs` so it describes scoped rows as
  current functionality rather than future work.
- Replace stale `run_do!` wording with `im_do!`.
- Add a short "default erased vs explicit" note explaining when to choose
  `Run` versus `RunExplicit`.
- Ensure examples and module prose avoid plan-phase language and remain
  self-contained.

Done criteria:

- `just doc` passes.
- `rg -n "run_do|future scoped|future work populates" fp-library/src/types/effects/run.rs`
  has no stale hits.

### Step 2. Normalize Public Handler Naming

Review trace:
[`effects-system-review.md`](effects-system-review.md#limitations-and-inconsistencies),
recommendation 2.

Tasks:

- Keep `Dispatch*` names for internal traits that walk handler lists or resume
  private boundaries.
- Use "handler" for public standard scoped-handler values, constructors,
  module docs, and examples.
- Audit standard scoped handler docs for "dispatcher receiver" and
  "scoped-dispatcher child module" wording.
- Do not rename internal tests unless they appear in user-facing docs or create
  confusion in generated documentation.

Done criteria:

- Public docs in `standard_scoped_handlers` consistently describe
  `catch_handler`, `local_handler`, `bracket_handler`, `span_handler`,
  `writer_pre_handler`, and `writer_post_handler` as handlers.
- Internal protocol docs still explain why `Dispatch*` is the correct protocol
  vocabulary.

### Step 3. Add Duplicate-Entry Diagnostics To Row And Handler Macros

Review trace:
[`effects-system-review.md`](effects-system-review.md#limitations-and-inconsistencies),
recommendation 3.

Tasks:

- Detect duplicate normalized keys in:
  - `effects!`
  - `raw_effects!`
  - `scoped_effects!`
  - `handlers!`
  - `scoped_handlers!`
  - `define_effect_row_aliases!`
  - `define_scoped_row!`
- Emit direct `syn::Error` messages pointing at the duplicate entry.
- Add UI or unit tests for duplicate first-order rows, scoped rows,
  first-order handler lists, scoped handler lists, and named row aliases.

Done criteria:

- Duplicate macro inputs fail during macro expansion with clear messages.
- Existing valid row-ordering tests still pass.

### Step 4. Document And Enforce The Row Canonicalization Contract

Review trace:
[`effects-system-review.md`](effects-system-review.md#limitations-and-inconsistencies),
recommendation 4.

Tasks:

- Document that macro row ordering is lexical over the macro input's normalized
  token spelling, not semantic Rust type identity.
- State the caveat that aliases and fully-qualified/imported spellings may sort
  differently because proc macros cannot resolve Rust names.
- Keep sorting and duplicate detection in one shared helper so
  `effects!`, `scoped_effects!`, handler macros, and row-alias macros do not
  drift.
- Add tests that demonstrate the supported contract:
  - whitespace-insensitive spelling is stable;
  - same spelling duplicates are rejected;
  - semantically equivalent aliases are not promised to canonicalize together.

Done criteria:

- Macro docs state the ordering contract and its limits.
- Tests cover the contract rather than implying semantic alias resolution.

### Step 5. Add Named Runners And Thin Ergonomic Helpers

Review trace:
[`effects-system-review.md`](effects-system-review.md#missing-or-incomplete-areas),
recommendation 5.

Scope for this pass:

- Reader helpers:
  - `asks`
  - `run_reader`
- State helpers:
  - `gets`
  - `modify`
  - `run_state`
  - `eval_state`
  - `exec_state`
- Except helpers:
  - `fail` or a Rust-appropriate name if `fail` conflicts with local naming
    conventions;
  - `rethrow`
  - `note`
  - `from_just` or a Rust-appropriate `Option`-to-Except helper name;
  - `run_except`
- Writer helpers:
  - named Writer runners/folders over current `tell`, `listen`, and `censor`
    semantics.
- Choose/Empty helpers:
  - `run_empty`
  - `run_choose` for supported multi-shot wrappers.

Boundaries:

- Do not add new core effect machinery in this step.
- Do not port runtime-sensitive Heftia effects in this step.
- Use the existing handler machinery as the implementation substrate.

Done criteria:

- Each helper has semantic examples with assertions over real handled results.
- Helpers are tested across the wrapper families where the semantics are valid.
- Any helper name that conflicts with Rust expectations is documented with the
  chosen alternative.

### Step 6. Audit Documentation Examples Using The Inventory

Review trace:
[`effects-system-review.md`](effects-system-review.md#limitations-and-inconsistencies),
recommendation 7.

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

### Step 7. Decide The Builder Fallback Story

Review trace:
[`effects-system-review.md`](effects-system-review.md#limitations-and-inconsistencies).

Tasks:

- Resolve [D2](#d2-handler-builder-ordering) before changing builder APIs.
- Update docs and tests according to the chosen option.

Done criteria:

- Manual builder composition has an explicit, documented order model.
- Users are steered toward `handlers!` / `scoped_handlers!` for the common
  path.

### Step 8. Decide Generic Scoped Row Support

Review trace:
[`effects-system-review.md`](effects-system-review.md#missing-or-incomplete-areas).

Tasks:

- Resolve [D3](#d3-generic-scoped-row-support).
- If adopted now, design syntax and tests before implementation.
- If deferred, add the revisit trigger to the long-term effects plan.

Done criteria:

- The project has an explicit decision instead of an implicit macro error being
  the whole policy.

### Step 9. Keep Runtime-Heavy Ports Deferred Behind Policy

Review trace:
[`effects-system-review.md`](effects-system-review.md#upstream-port-candidates),
recommendation 6.

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

## Open Decisions

### D1. Row Canonicalization Strategy

Issue:
The macros currently sort rows and handler lists by a string key derived from
the parsed `syn::Type` tokens. This is deterministic and shared, but it is not
semantic Rust type identity.

Options:

- **A. Keep lexical token-spelling canonicalization and document it.**
  - Pros: matches current implementation; cheap; predictable; keeps rows and
    handlers aligned; proc macros can implement it without name resolution.
  - Cons: aliases and fully-qualified/imported spellings can differ; users may
    assume semantic canonicalization unless docs are explicit.
- **B. Attempt structural canonicalization over `syn::Type`.**
  - Pros: better aligned with the repo preference for AST-aware macro code;
    could normalize whitespace and some path syntax without relying directly on
    raw strings.
  - Cons: still cannot resolve aliases or imports; likely more complex without
    solving the hardest user-visible cases; risks a false sense of semantic
    equality.
- **C. Remove sorting and make input order authoritative.**
  - Pros: most explicit; no hidden ordering rule; no alias canonicalization
    ambiguity.
  - Cons: handlers must be written in exactly the same order as rows; row alias
    reuse becomes more fragile; loses the current ergonomic alignment between
    row and handler macros.

Recommendation:
Adopt Option A for now, with one refinement: keep parsing structurally with
`syn`, but make the final ordering key an explicit documented token-spelling
contract. Add duplicate detection for identical normalized keys. Do not attempt
semantic alias resolution in a proc macro.

Reasoning:
Option B is attractive in principle, but proc macros do not have Rust name
resolution, so it cannot deliver true semantic canonicalization. Option C makes
the common case worse. Option A is the honest contract and can be made safe
with good diagnostics.

### D2. Handler Builder Ordering

Issue:
The `nt().on(...)` and scoped builder fallback currently prepends cells. That
matches cons-list construction, but it is easy for users to get wrong because
the macro path sorts handlers for them.

Options:

- **A. Keep `.on(...)` as prepend and document it as low-level.**
  - Pros: no implementation churn; preserves current type-level shape; honest
    about cons-list mechanics.
  - Cons: continues to be a footgun for manual users.
- **B. Change `.on(...)` to append in user-written order.**
  - Pros: intuitive builder order; less surprising for users.
  - Cons: likely requires type-level append machinery; breaks current code;
    may complicate handler-list types and inference.
- **C. Add an explicit append-style builder while keeping prepend available.**
  - Pros: gives users an intuitive path without losing low-level cons-list
    construction; transition can be documented clearly.
  - Cons: adds API surface; still needs append machinery if implemented as
    true append.
- **D. Rename or supplement prepend semantics with an explicit `prepend`
  vocabulary and steer common users to macros.**
  - Pros: low risk; makes the footgun visible; avoids append complexity until
    a real need appears.
  - Cons: does not provide natural-order manual builder composition.

Recommendation:
Adopt Option D now. Document `.on(...)` as cons-list/prepend-oriented or add a
more explicit prepend-named alias, and strongly recommend `handlers!` /
`scoped_handlers!` for normal use. Revisit Option C if users need manual
natural-order builders.

Reasoning:
The macro path already solves the common case. Changing builder semantics now
would add complexity to a fallback API while the larger effects surface is still
stabilizing.

### D3. Generic Scoped Row Support

Issue:
`define_scoped_row!` rejects generic marker rows. Concrete marker rows are
sufficient today, but reusable environment/error/log-parameterized scoped rows
will eventually need a better syntax.

Options:

- **A. Keep concrete-only scoped rows.**
  - Pros: simplest; matches current use; avoids premature macro design.
  - Cons: reusable generic scoped stacks remain awkward or impossible through
    the macro.
- **B. Add generic parameters directly to `define_scoped_row!`.**
  - Pros: most direct user-facing syntax; improves reusable row definitions.
  - Cons: recursive marker impl generation becomes more complex; must handle
    lifetimes, type params, and where-clauses carefully.
- **C. Add a separate generic row item macro.**
  - Pros: keeps the current concrete macro simple; lets generic support have a
    purpose-built syntax and tests.
  - Cons: more macro surface and documentation.

Recommendation:
Defer implementation until after named runners/helpers land, but record Option
C as the preferred direction if generic scoped rows become necessary.

Reasoning:
Generic scoped rows are not blocking the immediate review remediation. A
separate macro is more maintainable than overloading the concrete marker macro
before the generic use cases are known.

### D4. Named Helper And Runner Scope

Issue:
The review identified many possible helper APIs. Adding all of them at once
risks broad churn and uneven semantics across wrappers.

Options:

- **A. Add only thin first-order helpers first.**
  - Pros: low semantic risk; directly improves ergonomics; uses existing
    handlers.
  - Cons: scoped helper coverage remains incomplete for a while.
- **B. Add named runners for every implemented effect in one pass.**
  - Pros: broad discoverability improvement.
  - Cons: large surface; more chances for wrapper drift; harder to review.
- **C. Add one effect-family at a time, starting with State/Reader/Except.**
  - Pros: bounded commits; tests can establish patterns before Writer/Choose.
  - Cons: takes longer to reach full parity.

Recommendation:
Adopt Option C. Start with State, Reader, and Except, then Writer, then
Choose/Empty. Keep each effect family in a separate commit series with
wrapper-validity tests.

Reasoning:
This gives users visible ergonomic wins while limiting blast radius and
exposing pattern problems early.

### D5. Runtime-Heavy Upstream Ports

Issue:
Heftia includes effects whose semantics depend on async, IO, process
lifecycle, cancellation, or continuation capture.

Options:

- **A. Port the type shapes now and leave handlers minimal.**
  - Pros: broad apparent coverage.
  - Cons: creates misleading APIs; semantics are underspecified.
- **B. Defer all runtime-heavy ports until policy exists.**
  - Pros: avoids unsound or misleading abstractions; aligns with long-term
    architecture priority.
  - Cons: delays feature coverage.
- **C. Prototype one runtime-heavy effect privately.**
  - Pros: can reveal needed substrate changes.
  - Cons: should not become public without policy; risks another technical debt
    loop if not time-boxed.

Recommendation:
Adopt Option B. Use private prototypes only when a later policy question needs
evidence.

Reasoning:
The library currently has no async/IO/cancellation policy. Public runtime-heavy
effects before that policy would be architecture debt, not progress.

## Suggested Implementation Order

1. Step 1: stale docs.
2. Step 2: handler naming cleanup.
3. Step 3: duplicate-entry macro diagnostics.
4. Step 4: canonicalization contract docs/tests.
5. Step 5: named helpers/runners, effect family by effect family.
6. Step 6: inventory-driven documentation example audit.
7. Step 7 and Step 8: builder/generic-row decisions when implementation reaches
   those surfaces.
8. Step 9: keep runtime-heavy ports deferred until policy work is scheduled.

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
