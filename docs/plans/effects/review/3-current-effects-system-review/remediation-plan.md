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

There are no blockers that prevent beginning remediation. The decision records
in [Adopted Decisions And Fallbacks](#adopted-decisions-and-fallbacks) are
adopted implementation direction, not unresolved blockers. If an adopted
direction hits a concrete Rust type-system or proc-macro limitation, record the
limitation in the review before using the documented fallback.

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

### Step 3. Prototype And Adopt Structural Row Canonicalization

Review trace:
[`effects-system-review.md`](effects-system-review.md#limitations-and-inconsistencies),
recommendation 4; decision [D1](#d1-row-canonicalization-strategy).

Tasks:

- Replace the current `quote!(#ty).to_string()` ordering key with a shared
  structural row-key helper over `syn::Type`.
- Keep the helper centralized in `fp-macros/src/effects/row_sort.rs` or a
  child module owned by row sorting so effect rows, scoped rows, handler lists,
  scoped-handler lists, and row-alias generation cannot drift.
- Normalize the AST shapes that can be normalized without Rust name resolution:
  path segments, generic arguments, qualified-self syntax where representable,
  parenthesized/grouped types, references, tuples, and punctuation-independent
  spacing.
- Do not claim alias resolution, import resolution, or semantic Rust type
  identity. Proc macros do not have that information.
- Add focused unit tests for structurally equivalent spellings that should sort
  together and for semantically equivalent aliases that are intentionally not
  promised.
- If the structural key cannot be made deterministic and trustworthy for the
  supported macro input grammar, stop and document the limitation before using
  D1's token-spelling fallback.

Done criteria:

- Row and handler macros use the same structural row-key helper.
- The review document replaces the old lexical-sort analysis with the current
  structural-key assessment or documents the fallback limitation.
- Focused macro tests cover supported normalization and the no-name-resolution
  boundary.

### Step 4. Add Duplicate-Entry Diagnostics To Row And Handler Macros

Review trace:
[`effects-system-review.md`](effects-system-review.md#limitations-and-inconsistencies),
recommendations 3 and 4; decision [D1](#d1-row-canonicalization-strategy).

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

### Step 5. Document And Enforce The Row Canonicalization Contract

Review trace:
[`effects-system-review.md`](effects-system-review.md#limitations-and-inconsistencies),
recommendation 4; decision [D1](#d1-row-canonicalization-strategy).

Tasks:

- Document the adopted structural row-key contract: macro row ordering is based
  on the supported `syn::Type` structure that the macro can observe, not full
  semantic Rust type identity.
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

### Step 6. Add Named Runners And Thin Ergonomic Helpers

Review trace:
[`effects-system-review.md`](effects-system-review.md#missing-or-incomplete-areas),
recommendation 5; decision [D4](#d4-named-helper-and-runner-scope).

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

### Step 7. Audit Documentation Examples Using The Inventory

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

### Step 8. Add Natural-Order Handler Builders And Explicit Prepend APIs

Review trace:
[`effects-system-review.md`](effects-system-review.md#limitations-and-inconsistencies).

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

### Step 9. Schedule Generic Scoped Row Support As A Separate Macro

Review trace:
[`effects-system-review.md`](effects-system-review.md#missing-or-incomplete-areas).

Tasks:

- Keep `define_scoped_row!` concrete-only for the immediate remediation pass.
- Add a later implementation step for a separate generic scoped-row item macro,
  using [D3](#d3-generic-scoped-row-support) Option C as the target direction.
- The later step must design syntax, lifetime/type/where-clause handling, and
  recursive `Self` replacement tests before implementation.

Done criteria:

- The project has an explicit decision instead of an implicit macro error being
  the whole policy.

### Step 10. Keep Runtime-Heavy Ports Deferred Behind Policy

Review trace:
[`effects-system-review.md`](effects-system-review.md#upstream-port-candidates),
recommendation 6; decision [D5](#d5-runtime-heavy-upstream-ports).

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

## Adopted Decisions And Fallbacks

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
Adopt Option B as the target. Implement a structural row-key helper over
`syn::Type`, share it across row and handler macros, and use it for sorting and
duplicate detection. Keep Option A as the fallback only if a focused prototype
shows that the structural key cannot be made deterministic and trustworthy for
the supported macro input grammar.

Reasoning:
The API stability stance favors the cleaner long-term macro contract over
preserving a quote-string implementation detail. Option B can improve
internal coherence and reduce spacing/punctuation artifacts while still being
honest about the proc-macro boundary: it cannot resolve Rust aliases or
imports. If the prototype shows that structural keys would be more misleading
than useful, Option A remains the documented fallback.

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
Adopt Option C together with Option D's naming discipline. Add a natural-order
builder for manual handler composition and keep the low-level cons-list path
available under explicit `prepend` vocabulary. In other words, `.on(...)`
should mean natural-order composition on the new builder, while
`nt().prepend(...)` and `scoped_nt().prepend(...)` expose representation-level
prepend construction.

Reasoning:
Adopting both C and D makes sense because they solve different parts of the
problem. Option C gives users an intuitive manual API. Option D prevents the
existing cons-list operation from pretending to be intuitive source-order
composition. This is more aligned with the pre-1.0 API stance than keeping a
surprising `.on(...)` behavior for compatibility.

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
Defer implementation until after named runners/helpers land, and record Option
C as the adopted future direction for generic scoped rows.

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
3. Step 3: structural row-key helper and canonicalization prototype.
4. Step 4: duplicate-entry macro diagnostics using the adopted row key.
5. Step 5: canonicalization contract docs/tests.
6. Step 6: named helpers/runners, effect family by effect family.
7. Step 7: inventory-driven documentation example audit.
8. Step 8: natural-order builders plus explicit prepend APIs.
9. Step 9: schedule generic scoped row support as a separate macro.
10. Step 10: keep runtime-heavy ports deferred until policy work is scheduled.

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
