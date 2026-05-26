# Plan: Audit `document_examples(skip_call_check)`

## Status

Policy decisions adopted. Implementation has not started.

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

### Live failing macro tests

The focused trybuild command:

```bash
just filtered test '^(test .*compile_fail_tests|test result:|failures:|error|warning|[[:space:]]*-->|\\[.*\\] tests/compile-pass/document_examples_call_check\\.rs)' -p fp-macros compile_fail_tests
```

currently fails.

Known causes:

- `fp-macros/tests/compile-pass/document_examples_call_check.rs` still has two
  bare `#[document_examples(skip_call_check)]` attributes without `reason`.
- `fp-macros/tests/ui/document_examples_requires_annotated_call.stderr` still
  expects an older diagnostic that suggests bare `skip_call_check`.

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

## Policy Decisions

### Macro enforcement

Use hard errors for objective `document_examples` misuse.

Hard-error cases:

- `skip_call_check` without `reason`;
- empty `reason`;
- `reason` without `skip_call_check`;
- `skip_call_check` on a non-function item where direct-call validation does
  not apply;
- unnecessary `skip_call_check` when every Rust code block already calls the
  documented function or method.

Rationale:

- `document_examples` already uses hard errors for invalid examples and invalid
  options.
- The current warning path is tied to `document_module` lint-style validation,
  not item-local `document_examples` contract failures.
- A warning would let stale suppressions survive normal builds unless the
  caller denies that warning. A hard error makes the expectation self-cleaning.

Do not make subjective reason quality a macro hard error beyond the existing
empty-reason check. Reasons such as "too generic" are better handled by an audit
script and human review.

### Expect-like skip semantics

Treat `skip_call_check` as an item-level expectation:

- Without `skip_call_check`, every Rust code block for a function or method must
  call the documented item.
- With `skip_call_check`, at least one Rust code block must fail direct-call
  validation. If every Rust code block already calls the documented item, the
  skip is unnecessary and should be reported.

This avoids rejecting mixed examples where one block calls the item directly
and another block intentionally documents an indirect public facade.

The stricter per-block rule is not selected. The macro should not report
`skip_call_check` merely because one block calls the documented item directly;
the skip is still needed when another block on the same item intentionally
documents an indirect public facade.

### Script audit policy

Enhance `scripts/document_examples.rs` with reason-audit modes that flag only
objective issues by default:

- missing `reason`;
- empty `reason`;
- exact stale placeholder reason;
- `reason` present without `skip_call_check`;
- `skip_call_check` on an item with no direct-call validation target;
- unnecessary `skip_call_check` when every Rust code block calls the documented
  function or method.

Add a separate report mode for non-blocking review data:

- repeated reason strings and counts;
- examples with very short reasons;
- examples with broad words such as `TODO`, `audit`, or `temporary`;
- examples whose code uses weak assertion patterns that the macro does not
  reject.

The default invalid-reason mode must stay objective and suitable for CI.
Subjective quality signals belong in a report-only mode.

## Adopted Decisions

These decisions are adopted for implementation.

1. Diagnostic severity for unnecessary `skip_call_check`.

   Use a hard error.

2. Expect-like semantics.

   Use the item-level rule. Error only when every Rust code block calls the
   documented item.

3. Script command shape.

   Add an argv-safe `just document-examples *args` recipe that runs
   `rust-script scripts/document_examples.rs -- "$@"`.

4. CI integration.

   Add the script audit to `just doc` only after repo-wide stale placeholder
   cleanup is complete. Until then, keep it as an explicit verification command
   for each cleanup batch.

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

### Step 4: Add expect-like macro validation

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
- a mixed direct and indirect example passes.

Verification:

```bash
just fmt
just filtered test '^(test .*document_examples|test .*compile_fail_tests|test result:|failures:|error|warning|[[:space:]]*-->)' -p fp-macros
just filtered check '^(error|warning|[[:space:]]*-->)' -p fp-macros
```

### Step 5: Produce a repo-wide cleanup audit

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

- the audit gives a bounded checklist for the `781` stale placeholder reasons;
- effects are recorded as already clean for stale placeholder reasons.

Verification:

```bash
just document-examples --invalid-reasons --json
git diff --check
```

### Step 6: Clean up `fp-library/src/classes`

Work:

- Remove `skip_call_check` where examples can call the documented item.
- Replace stale placeholder reasons where the skip remains justified.
- Prefer direct public examples over indirect stand-ins.

Acceptance criteria:

- no stale placeholder reason remains under `fp-library/src/classes`;
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

- no stale placeholder reason remains under `fp-library/src/dispatch`;
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

- no stale placeholder reason remains in the selected core type batch;
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

- no stale placeholder reason remains under `fp-library/src/types/optics`;
- optics doctests pass.

Verification:

```bash
just fmt
just document-examples --path fp-library/src/types/optics --invalid-reasons
just filtered test '^(test .*optics|test result:|failures:|error|warning|[[:space:]]*-->)' --doc -p fp-library
just filtered check '^(error|warning|[[:space:]]*-->)' -p fp-library --lib
git diff --check
```

### Step 10: Enable enforcement in the standard docs gate

Update:

- `justfile`
- possibly `scripts/document_examples.rs`

Work:

- After all stale placeholder reasons are gone, add the reason audit to the
  standard documentation gate.
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
4. `fix(macros): reject unnecessary skip_call_check`
5. `docs(plan): record document_examples cleanup audit`
6. `docs(classes): audit document_examples skip reasons`
7. `docs(dispatch): audit document_examples skip reasons`
8. `docs(types): audit document_examples skip reasons`
9. `docs(optics): audit document_examples skip reasons`
10. `chore(docs): enforce document_examples reason audit`

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
