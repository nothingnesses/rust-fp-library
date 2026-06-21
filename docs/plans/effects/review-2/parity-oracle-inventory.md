# FS-1 Rebuild: Behaviour-Parity Oracle Inventory

This is item 4 step 1 of [remediation-plan.md](remediation-plan.md): the pre-destructive tagging of the effects test corpus. It defines the rebuild's acceptance oracle, the behaviours that must survive the FS-1 rebuild regardless of the API break, so that no guarantee is deleted (item 4 step 4) before it is re-pinned on FS-1.

Method: each effects-related test file under `fp-library/tests/` is tagged into one bucket. The porting itself happens as the FS-1 substrate and vertical slice come up (item 4 steps 3 to 5); this document is the checklist that porting works against. The non-effects and inference-only POC files are listed under bucket D so the boundary of the rebuild is explicit.

Principle applied: the oracle pins _observable_ behaviour. The plan's intentional changes do not conflict with it, the FS-1 mechanism change (elaboration replacing boundary frames) was shown by POC-4/POC-5/POC-9 to _reproduce_ the observable results, and item 14 _adds_ threaded-accumulator runners while keeping the existing cell-based behaviour. So "preserve all current observable semantics" is correct; new behaviour is owned by its own item's tests.

## Bucket A: semantics to preserve (the acceptance oracle)

These encode observable effect semantics that must hold on FS-1 with equivalent assertions (the API calls are rewritten, the asserted results are not). Port these first and keep them green through the rebuild.

- `run_heftia_semantics.rs`, the primary parity suite (heftia current-effect semantics: State-with-Catch ordering, Choose-with-Catch, etc.). Highest-priority oracle.
- `run_catch.rs`, Catch recovery and state-survives-catch ordering.
- `run_state.rs`, State get/put threading.
- `run_reader.rs`, Reader ask/environment.
- `run_writer.rs`, `run_writer_accumulation.rs`, `run_writer_rewrite.rs`, `run_writer_listen_handler.rs`, `run_writer_post_handler.rs`, `run_writer_pre_handler.rs`, `run_writer_scoped.rs`, `run_writer_listen_boundary_split.rs`, Writer/Listen/Censor accumulation and ordering (the boundary-split variant becomes an elaboration case; the asserted logs are preserved).
- `run_choose.rs`, NonDet/Choose branching (and the shared-across-branches accumulator behaviour item 14 keeps).
- `run_bracket.rs`, `run_ref_bracket.rs`, Bracket resource acquire/release ordering.
- `run_local.rs`, `run_ref_local.rs`, Local environment modification scope.
- `run_empty.rs`, Empty short-circuit.
- `run_except.rs`, Except throw/catch narrowing.
- `run_span.rs`, Span boundary semantics, including the borrowed (non-`'static`) payload case that fixes the concrete substrate as a requirement (POC-11).
- `run_coroutine_helpers.rs`, Coroutine/Yield suspension and resumption.
- `run_interpose.rs`, interposition (re-handling an effect mid-program).
- `run_effect_composition_matrix.rs`, cross-effect composition results.
- `run_talkf_dinnerf_integration.rs`, the canonical multi-effect end-to-end example.
- `thread_safety.rs`, `Send + Sync` guarantees for the Arc-store substrate (a property the `Store`-parameterised Arc instantiation must keep, POC-8b).

Effect-behaviour content inside the `_helpers` files (`run_except_helpers.rs`, `run_fail_helpers.rs`, `run_log_helpers.rs`, `run_output_helpers.rs`, `run_reader_helpers.rs`, `run_state_helpers.rs`, `run_low_risk_effect_helpers.rs` for Fresh/Input/KVStore) is bucket A for its asserted results; the helper-naming surface is bucket B.

## Bucket B: API-shape (rewrite onto the FS-1 surface, preserve inner assertions)

These test the dual-row API surface that the rebuild re-authors. Rewrite the calls onto FS-1; keep any behavioural assertions (which are bucket A in spirit).

- `run_handle.rs`, `run_handle_rec.rs`, `run_handle_with.rs`, `run_handle_with_either.rs`, the `handle`/`run` method surface (items 12, 13 rename; item 4 step 7 re-authors on `Run<Store, A>`).
- `*_helpers.rs` (the naming surface only), regenerated per-effect against the FS-1 smart constructors (item 7).
- `run_lift.rs`, `lift` onto the unified row.
- `run_row_canonicalisation.rs`, positional row sort, made moot by brand-keyed dispatch (item 8); the canonicalisation assertions are dropped, not ported.
- `multi_brand_integration.rs`, `non_regression_single_brand.rs`, row membership and single-brand integration, rewritten onto the unified row.
- `handlers_macro.rs`, `effects_macro.rs`, the `handlers!`/`effects!` macros, superseded by `effect_spec!` (item 8); rewrite against the new macro surface.
- `im_do.rs`, the `im_do` do-notation over `Run`, re-targeted to the FS-1 substrate.

## Bucket B-scoped: scoped-row API tests (semantics re-pinned via elaboration)

These test the scoped-row / boundary-frame machinery that FS-1 _deletes_ (item 4 step 4). Their API is obsolete, but the observable semantics they assert must be re-pinned as in-row elaboration cases (item 4 step 2 elaboration; item 17 for scoped choice).

- `run_standard_scoped_handlers.rs`, `run_scoped_row_primitives.rs`, `define_scoped_row_macro.rs`, scoped handlers / scoped-row primitives / the `define_scoped_row!` macro. The handlers' results move to bucket A (elaborated); the scoped-row API itself is deleted.

## Bucket C: substrate / foundation tests (reused, keep)

The FS-1 substrate decision (POC-11) reuses the existing `Free` (erased) and `FreeExplicit` (concrete) spines, `Store`-parameterised. These substrate-level tests stay relevant.

- `free_explicit_poc.rs`, the concrete non-`'static` substrate (becomes the concrete `Store`-parameterised form's base).
- `stack_safety.rs`, stack-safe `evaluate`/`Drop` for `Trampoline`/`Thunk`/`TryTrampoline` (the shared `Free` infrastructure the rebuild leaves intact). Item 2 adds the missing deep-`Run`-program tests on top.
- `run_wrap_depth_probe.rs`, `arc_run_normalization_probe.rs`, substrate-internal probes (wrap-depth drop, Arc GAT normalisation); re-evaluate against the `Store`-parameterised substrate, keep whichever still pin a real property.

## Bucket D: out of scope for the effects-rebuild oracle

- `slot_*.rs` (12 files), `InferableBrand`/slot inference POCs for the `Kind` machinery, not effects-substrate behaviour.
- `async_interpreter_feasibility.rs`, `async_interpreter_increments.rs`, `async_interpreter_remainders.rs`, throwaway async feasibility POCs.
- `async_interpreter_public_api.rs`, the public async surface, owned by item 4 step 6 (carry forward or reintroduce) and item 18 (exponential redesign), not the polynomial parity oracle.
- `compile_fail.rs` and `tests/ui/*`, the trybuild error-quality suite; the cases that pin effect-API error quality are re-authored with the FS-1 surface, but they are not behavioural parity.

## Use during the rebuild

Item 4 step 4 (the destructive deletion) may not proceed for a given behaviour until its bucket A (or B-scoped re-pinned) test is green on FS-1. Bucket A is the gate; buckets B and B-scoped are rewrite work tracked by their owning items (8, 12, 13, 17); bucket C is carried; bucket D is untouched by the rebuild.
