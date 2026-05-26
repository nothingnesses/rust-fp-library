# Plan: Audit `document_examples(skip_call_check)`

## Status

Step 1 is complete. Step 2 is next.

Chosen approaches are represented directly in the implementation steps,
acceptance criteria, and verification commands below. This plan intentionally
does not keep an adopted or resolved decisions archive.

Created 2026-05-26.

## Goal

Make `#[document_examples(skip_call_check, reason = "...")]` precise,
auditable, and hard to leave stale. The end state should have:

- no missing `reason` values;
- no stale placeholder reasons;
- a script-level repo audit for invalid or suspicious skip reasons;
- an expect-like macro check that reports an unnecessary `skip_call_check`;
- repo-wide cleanup of existing stale reasons in focused batches;
- verification commands documented for every implementation step.

## Current Findings

### Effects subtree

`fp-library/src/types/effects` currently has `306` `document_examples`
attributes with `skip_call_check`.

The stale placeholder reason:

```text
Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical.
```

has `0` matches under `fp-library/src/types/effects`.

### Repo-wide surface

The repo currently has `1089` `document_examples` attributes with
`skip_call_check`.

The stale placeholder reason still appears `781` times across `117`
Rust files outside the completed effects cleanup.

The main remaining areas are:

- `fp-library/src/classes/`
- `fp-library/src/dispatch/`
- `fp-library/src/types/` outside `types/effects`
- `fp-library/src/types/optics/`

### Macro test baseline

The focused trybuild command:

```bash
just filtered test '^(test .*compile_fail_tests|test result:|failures:|error|warning|[[:space:]]*-->|\\[.*\\] tests/compile-pass/document_examples_call_check\\.rs)' -p fp-macros compile_fail_tests
```

now passes.

Step 1 repaired the previous failures by:

- adding concrete reasons to the two compile-pass
  `#[document_examples(skip_call_check)]` fixtures;
- updating the stale direct-call diagnostic expectation so it suggests
  `skip_call_check, reason = "..."`;
- adding a trybuild compile-fail fixture for bare `skip_call_check`.

## Current Macro Behavior Investigation

### `document_examples`

`fp-macros/src/documentation/document_examples.rs` parses these options:

- no options;
- `skip_call_check, reason = "..."`.

Current hard errors:

- duplicate `skip_call_check`;
- `skip_call_check = ...`;
- duplicate `reason`;
- empty `reason`;
- `skip_call_check` without `reason`;
- `reason` without `skip_call_check`;
- unsupported options;
- missing Rust code blocks;
- code blocks without assertion macros;
- literal-only assertions;
- wildcard-only variant assertions;
- missing direct call to the documented function or method when
  `skip_call_check` is absent.

When `skip_call_check` is present, the macro currently skips direct-call
validation entirely. It does not check whether the skip was actually needed.

### `document_module`

`fp-macros/src/documentation/document_module.rs` uses `WarningEmitter` for
documentation validation and impl-trait lint diagnostics. The emitter lives at
`fp-macros/src/core/warning_emitter.rs` and produces non-blocking warnings via
`proc-macro-warning` deprecated marker tokens.

This means the macro system has both styles:

- hard errors via `syn::Error::to_compile_error()` for invalid macro input and
  invalid `document_examples` code blocks;
- non-blocking warnings for `document_module` validation lint findings.

### `scripts/document_examples.rs`

`scripts/document_examples.rs` currently counts, lists, and extracts examples.
It does not validate reason text.

The script has no `just` recipe wrapper today. The project command policy says
commands should run through `just`, so the plan should add an argv-safe recipe
before relying on the script as a regular verification command.

## Open questions, decisions, issues and blockers

> **Maintenance template.** Tracks all active load-bearing questions,
> decisions, issues, and blockers that affect upcoming work. Each active item
> must include the blocked work, context, options or approaches, trade-offs,
> recommendation, and reasoning for the recommendation. Do not add an
> `Adopted Decisions`, `Resolved decisions`, or equivalent archive section.
> Once an item resolves, fold the chosen path cleanly into the relevant
> concrete implementation steps and remove the active item. Preserve any needed
> historical detail in the commit message rather than keeping a parallel
> decision archive in this plan.

### Active items

No active items.

### Procedure for new active items

If a load-bearing question or blocker surfaces during implementation:

1. Add an `#### <id>. <summary>` subsection under `### Active items` above and
   pause work if the item blocks the next implementation step.
2. Include the blocked work, context, options or approaches, trade-offs,
   recommendation, and reasoning for the recommendation.
3. When the item resolves, fold the chosen path into the relevant concrete
   implementation step, acceptance criteria, or verification command.
4. Remove the active item if it no longer affects upcoming work. Do not retain
   adopted-decision or resolved-decision prose in this plan; use commit
   messages for historical detail.

## Implementation Steps

### Step 1: Repair current macro test drift

Update:

- `fp-macros/tests/compile-pass/document_examples_call_check.rs`
- `fp-macros/tests/ui/document_examples_requires_annotated_call.stderr`

Work:

- Add concrete `reason = "..."` values to the two compile-pass fixture skips.
- Update the UI stderr expected message to mention
  `skip_call_check, reason = "..."`.
- Add or adjust tests so the current reason requirement is explicitly covered
  in both unit tests and trybuild fixtures.
- Complete this step before script audit work or expect-like macro behavior
  changes so later macro failures are not mixed with existing fixture drift.

Acceptance criteria:

- no bare `#[document_examples(skip_call_check)]` remains except in an
  intentional compile-fail fixture;
- the focused trybuild test passes.

Verification:

```bash
just fmt
just filtered test '^(test .*compile_fail_tests|test result:|failures:|error|warning|[[:space:]]*-->)' -p fp-macros compile_fail_tests
just filtered check '^(error|warning|[[:space:]]*-->)' -p fp-macros
```

### Step 2: Add a `just` wrapper for the audit script

Update:

- `justfile`

Work:

- Add an argv-safe recipe for `scripts/document_examples.rs`.
- Use `[positional-arguments]`.
- Forward arguments with `"$@"`.
- Do not interpolate unquoted variadic args.

Proposed command shape:

```bash
just document-examples --path fp-library/src/types/effects --json
```

Acceptance criteria:

- the script can be run through `just`;
- direct `rust-script` invocation is no longer needed in normal workflow.

Verification:

```bash
just fmt
just document-examples --path fp-library/src/types/effects --json
just document-examples --path fp-library/src/types/effects --list
```

### Step 3: Add objective reason-audit support to the script

Update:

- `scripts/document_examples.rs`

Work:

- Preserve existing count, list, and extract modes.
- Add an audit mode such as `--invalid-reasons`.
- Collect the full attribute text, path, line, kind, and parsed reason.
- Report objective invalid entries:
  - missing reason;
  - empty reason;
  - stale placeholder reason;
  - reason without skip;
  - skip on item kind where direct-call validation does not apply;
  - unnecessary skip when every Rust code block calls the documented function or
    method.
- Add `--json` support for the new mode.
- Add a separate report-only mode for subjective cleanup signals such as
  repeated reason strings, very short reasons, `TODO`-style wording, and weak
  assertion patterns that are not macro errors.
- Use this script audit as the temporary enforcement mechanism while repo-wide
  cleanup is still in progress; do not add macro hard errors for unnecessary
  skips until Steps 3-9 are complete.

Acceptance criteria:

- effects subtree reports zero stale placeholder reasons;
- repo-wide audit reports the existing invalid/stale entries before cleanup;
- output is stable enough to use in cleanup batches.

Verification:

```bash
just fmt
just document-examples --path fp-library/src/types/effects --invalid-reasons
just document-examples --invalid-reasons --json
just filtered check '^(error|warning|[[:space:]]*-->)' -p fp-macros
```

### Step 4: Produce a repo-wide cleanup audit

Update:

- `docs/plans/document-examples-skip-call-check/audit.md`

Work:

- Run the enhanced script repo-wide.
- Record counts by directory and file.
- Classify cleanup type:
  - remove skip because direct call is practical;
  - keep skip and replace stale placeholder reason;
  - convert weak example into a direct meaningful example;
  - leave for explicit design decision because the reason exposes a real API
    issue.

Acceptance criteria:

- the audit gives a bounded checklist for every objective invalid entry:
  missing reasons, empty reasons, stale placeholder reasons, reason-without-skip
  cases, non-applicable skips, and unnecessary skips;
- effects are recorded as already clean for stale placeholder reasons and
  either clean or explicitly listed for any newly detected objective issue.

Verification:

```bash
just document-examples --invalid-reasons --json
git diff --check
```

### Step 5: Close effects audit findings

Work:

- Run the enhanced objective audit against `fp-library/src/types/effects`.
- Fix any objective invalid entries the new audit mode reports.
- If the effects subtree is already clean, record the zero-result command in
  the commit body for this step or the repo-wide audit step.

Acceptance criteria:

- `fp-library/src/types/effects` has zero objective invalid entries.
- focused effects doctests pass if any effects files changed.

Verification:

```bash
just fmt
just document-examples --path fp-library/src/types/effects --invalid-reasons
just filtered test '^(test .*types::effects|test .*effects/|test result:|failures:|error|warning|[[:space:]]*-->)' --doc -p fp-library
just filtered check '^(error|warning|[[:space:]]*-->)' -p fp-library --lib
git diff --check
```

### Step 6: Clean up `fp-library/src/classes`

Work:

- Remove `skip_call_check` where examples can call the documented item.
- Replace stale placeholder reasons where the skip remains justified.
- Prefer direct public examples over indirect stand-ins.

Acceptance criteria:

- `fp-library/src/classes` has zero objective invalid entries;
- focused doctests and macro checks pass.

Verification:

```bash
just fmt
just document-examples --path fp-library/src/classes --invalid-reasons
just filtered test '^(test .*classes|test result:|failures:|error|warning|[[:space:]]*-->)' --doc -p fp-library
just filtered check '^(error|warning|[[:space:]]*-->)' -p fp-library --lib
git diff --check
```

### Step 7: Clean up `fp-library/src/dispatch`

Work:

- Apply the same cleanup rules to dispatch wrappers and explicit variants.
- Keep reasons concrete when inference wrappers are intentionally documented
  through public facade behavior.

Acceptance criteria:

- `fp-library/src/dispatch` has zero objective invalid entries;
- dispatch doctests pass.

Verification:

```bash
just fmt
just document-examples --path fp-library/src/dispatch --invalid-reasons
just filtered test '^(test .*dispatch|test result:|failures:|error|warning|[[:space:]]*-->)' --doc -p fp-library
just filtered check '^(error|warning|[[:space:]]*-->)' -p fp-library --lib
git diff --check
```

### Step 8: Clean up core types outside effects and optics

Work:

- Cover `fp-library/src/types/*.rs` excluding `types/effects` and
  `types/optics`.
- Split into smaller commits if a type family is large:
  - free family;
  - lazy and thunk family;
  - control-flow and newtype wrappers;
  - collection and tuple wrappers.

Acceptance criteria:

- the selected core type batch has zero objective invalid entries;
- doctests for touched files pass.

Verification:

```bash
just fmt
just document-examples --path fp-library/src/types --invalid-reasons
just filtered test '^(test .*types::|test result:|failures:|error|warning|[[:space:]]*-->)' --doc -p fp-library
just filtered check '^(error|warning|[[:space:]]*-->)' -p fp-library --lib
git diff --check
```

### Step 9: Clean up optics

Work:

- Cover `fp-library/src/types/optics`.
- Preserve the optics pointer-brand and profunctor abstractions in examples.
- Prefer examples that run the public optic operation and assert a visible
  source or target value.

Acceptance criteria:

- `fp-library/src/types/optics` has zero objective invalid entries;
- optics doctests pass.

Verification:

```bash
just fmt
just document-examples --path fp-library/src/types/optics --invalid-reasons
just filtered test '^(test .*optics|test result:|failures:|error|warning|[[:space:]]*-->)' --doc -p fp-library
just filtered check '^(error|warning|[[:space:]]*-->)' -p fp-library --lib
git diff --check
```

### Step 10: Add expect-like macro validation

Precondition:

- Steps 3-9 are complete, and the repo-wide objective audit reports zero
  invalid entries.

Update:

- `fp-macros/src/documentation/document_examples.rs`
- `fp-macros/tests/ui/`
- `fp-macros/tests/compile-pass/`

Work:

- Reuse the existing direct-call detector when `skip_call_check` is present.
- Implement the item-level unnecessary-skip rule: if every Rust code block calls
  the documented function or method, `skip_call_check` is stale and must hard
  error.
- Reject `skip_call_check` on non-function items if direct-call validation has
  no target.
- Add compile-fail tests for unnecessary skip.
- Add compile-pass tests for justified skip.
- Add compile-pass tests for mixed examples where at least one block calls the
  documented item and at least one block intentionally documents indirect
  behavior.

Acceptance criteria:

- a stale skip on an example that directly calls the documented item fails;
- a justified indirect example with a concrete reason passes;
- a mixed direct and indirect example passes;
- `fp-library` still checks after the macro hard error is enabled.

Verification:

```bash
just fmt
just filtered test '^(test .*document_examples|test .*compile_fail_tests|test result:|failures:|error|warning|[[:space:]]*-->)' -p fp-macros
just filtered check '^(error|warning|[[:space:]]*-->)' -p fp-macros
just filtered check '^(error|warning|[[:space:]]*-->)' -p fp-library --lib
```

### Step 11: Enable enforcement in the standard docs gate

Update:

- `justfile`
- possibly `scripts/document_examples.rs`

Work:

- After the repo-wide objective audit is clean and Step 10 has enabled macro
  hard errors, add the reason audit to the standard documentation gate.
- Prefer adding it to `just doc`, because `just verify` already runs `doc`.
- Keep output bounded and actionable.

Acceptance criteria:

- `just doc` fails on missing reasons, stale placeholder reasons, and
  unnecessary skips;
- `just verify` includes the audit transitively.

Verification:

```bash
just fmt
just doc
just verify
```

## Commit Strategy

Use one commit per coherent step or cleanup batch. Suggested commits:

1. `test(macros): align document_examples skip fixtures`
2. `chore(docs): wrap document example audit script`
3. `chore(docs): audit document_examples skip reasons`
4. `docs(plan): record document_examples cleanup audit`
5. `docs(effects): audit document_examples skip reasons`
6. `docs(classes): audit document_examples skip reasons`
7. `docs(dispatch): audit document_examples skip reasons`
8. `docs(types): audit document_examples skip reasons`
9. `docs(optics): audit document_examples skip reasons`
10. `fix(macros): reject unnecessary skip_call_check`
11. `chore(docs): enforce document_examples reason audit`

Each commit should include the verification performed in its body.

## Stop Conditions

Pause and ask for a decision if:

- implementing hard errors for unnecessary skips exposes a concrete conflict
  with existing project lint behavior that is not covered by the macro behavior
  investigation above;
- the script cannot detect unnecessary skips without duplicating too much macro
  parsing logic;
- a large group of stale reasons exposes a real API documentation problem
  rather than simple stale suppression text;
- a cleanup batch needs API changes instead of documentation-only changes.
