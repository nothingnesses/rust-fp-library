# POC-9 Findings: integration slice over the unified row

Tier F, foundation sweep. Charter question: does the surviving FS-1 design carry a real slice (State + Reader + Catch + Writer-censor) end-to-end on the unified row, reproducing the heftia semantics cases?

Result: PASS, via Approach (isolated multi-HOE). The slice composes four first-order effects and two higher-order effects in one row on the existing public `Free`, under one interpret pass, reproducing the targeted heftia cases.

POC code (on the `spike/foundation-sweep` branch): `fp-library/tests/poc_fs_integration_slice.rs`. Three tests pass. Run via `just filtered test`.

## What was built

- Four first-order effects, each `Coyoneda`-wrapped in the row: State (`Get`/`Put` over a bool), Reader (`Ask` over an i32 environment), Throw (unit error), Writer (`Tell` over a String).
- Two higher-order effects as in-row cells (same-result, action result a brand type parameter as in POC-5): Catch (`{ action, recover, k }`) and Censor (`{ f, action, k }`), both fixed at action result `()` for this slice.
- One unified row holding all six, and one interpret pass (`run`) over the existing public `Free` (fixed `Box`): State reads/writes a shared cell; Reader returns the environment; Throw aborts (`Err`); Writer appends to the log; Catch runs the action and, on a throw, runs `recover`; Censor runs the action against a fresh local log, then emits `f(total)` to the outer log.

Three tests pass: the State-with-Catch ordering pair `(true, true)`; the Writer post-censor result `"Hello world!!"`; and a Reader + State + Catch composition (the environment selects the state write, which survives a caught throw).

## Findings

1. The unified row composes multiple higher-order effects with first-order effects under one interpret pass. This is POC-9's new evidence: POC-4 and POC-5 each handled a single higher-order effect; POC-9 holds Catch and Censor in the same row alongside State, Reader, Throw, and Writer, and one `run` loop peels and handles all six. No boundary frames, no result-polymorphic protocol traits, no scoped row.
2. Higher-order semantics fall out of how the interpreter shares or scopes its state at the recursive call, not from a carrier/boundary protocol. The State-with-Catch write survives because the recursive `run(action)` shares the one `state` cell, so the mutation persists through the `Err` the catch then recovers (the algebraic semantics the dual row uses boundary frames to achieve). Censor scopes a fresh local log for the action, rewrites the total with `f`, and emits only that to the outer log. Both higher-order behaviours are a few lines in the interpreter's arm for the cell.
3. The slice reproduces the heftia cases exactly. The State-with-Catch ordering pair `(true, true)` (from `run_heftia_semantics.rs`) and the Writer post-censor `"Hello world!!"` match; the Reader composition confirms a first-order reader threads through the same pass.

## Limitations and scope

- First-order handling uses direct coproduct matching (the documented fallback), not brand-keyed dispatch. Brand-keyed dispatch was proven standalone in POC-2 and is an orthogonal layer; integrating it over the real `Free` resume loop is left as wiring, since it would add no evidence for POC-9's composition question.
- The substrate is the existing public `Free` (fixed `Box`), deliberately not POC-8's parameterised `Run<Store, A>`; the substrate unification is validated separately (POC-8), and compounding it here would entangle a proven-separately concern.
- Catch and censor are fixed at action result `()` (the slice's actions return unit); generalising the action result is the per-effect brand parameter already exercised in POC-5/POC-6.
- The censor pre case (`"Goodbye world!"`, per-tell `f`) is not built; the post case (total `f`) is the one reproduced. Pre is a per-tell map variant of the same interpose.

## Bearing on gate G4

POC-9 shows the FS-1 design carries a real multi-effect, multi-higher-order slice end-to-end on the unified row, reproducing the heftia semantics, with the higher-order behaviour localised in the interpreter. Together with the earlier tiers (unified row, elaboration of same-result and result-shape-changing higher-order effects, row widening, brand-keyed dispatch, substrate unification), the design is validated across all four axes. The remaining Tier F item is POC-10 (the per-layer dispatch and allocation micro-benchmark against FS-0, with the perf baseline captured first per S6), after which gate G4 folds the rubric scores and the chosen design back into remediation items 8, 12, 13, and 14.
