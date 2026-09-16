# POC-11 Findings: substrate-identity integration spike

Follow-up spike for the production rebuild (remediation-plan item 4 step 2). It resolves the two substrate-identity questions the earlier POCs left open: the erasure axis (C-versus-B) and the Store-parameterisation of the substrate over the library's real row.

Result: PASS. POC code on the `spike/foundation-sweep` branch: `fp-library/tests/poc_fs_substrate_integration.rs`, four tests, clippy-clean. The concrete-form + real-row + non-`'static` combination is not re-proven here because it already ships as `RunExplicit` (`FreeExplicit` over `NodeBrand`) and is exercised with a borrowed payload by `run_span`'s `run_explicit_t5_span_boundary_preserves_borrowed_payload`; the spike cites that.

## Findings

1. Erasure axis: B (two `Store`-parameterised forms), not C2 (one form). The erased O(1) representation stores values and continuations behind `dyn Any` (the production `Free` uses `TypeErasedValue = Box<dyn Any>` and a `CatList` of `Box<dyn FnOnce(Box<dyn Any>) -> ...>`). `Box<dyn Any>` is `Box<dyn Any + 'static>`, so the O(1) path forces `A: 'static` (Part 1's `erase_value` compiles only with the bound; dropping it is E0310). Stable Rust cannot make one type select the erased queue only when `A: 'static` (no specialisation, no negative bounds), so the erased (`'static`, O(1) `bind`) and concrete (non-`'static`, O(N) `bind`) substrates are necessarily distinct types. C2 (one substrate, internal erasure for the `'static` case) is ruled out; the erasure axis collapses the six wrappers to two parameterised forms, not one. Approach A (drop non-`'static`) was already eliminated by the catalog audit.

2. Pointer axis, erased form: the real `Coyoneda`-wrapped `CoproductBrand` row composes on the existing public `Free` (Box-spine) directly (Part 2), so the erased form's `Store = Box` case reuses the existing `Free`, no second Box spine, satisfying the single-spine invariant. The Rc/Arc pointer cases are the `ClosureStorage` unification already proven in POC-8/POC-8b. So the erased form's pointer axis is the full {Box, Rc, Arc} via `Store`.

3. Pointer axis, concrete form: a `ClosureStorage`-parameterised concrete spine binds over the real `Coyoneda` row (Part 4), so the concrete form Store-parameterises over the real encoding too. One constraint is recorded: the concrete recursive `bind` clones the continuation into its per-layer closure (the pattern `FreeExplicit::bind_boxed` uses), so `Store::Stored` must be `Clone`. Box's `FnOnce` is not `Clone`, so the concrete form's pointer axis is {Rc, Arc}; a Box-flavoured concrete substrate reuses `Rc` internally, exactly as the shipped `FreeExplicit` already does. This is the one place the two forms' `Store` axes differ (erased: {Box, Rc, Arc}; concrete: {Rc, Arc}).

4. Net substrate identity for FS-1: `Store`-parameterise the two existing spine families rather than introduce new ones. The erased family (`Free`/`RcFree`/`ArcFree`) collapses to one `Store`-parameterised erased substrate (O(1) `CatList` `bind`, `'static`), reusing the existing `Free` for the Box case; the concrete family (`FreeExplicit`/`RcFreeExplicit`/`ArcFreeExplicit`) collapses to one `Store`-parameterised concrete substrate (O(N) `bind`, non-`'static`). Six wrappers become two parameterised substrates, with no new or duplicated spine.

## Limitations and scope

- Part 4's concrete spine is a minimal `Pure`/`Wrap` mirror of `FreeExplicit` with a `Store`-parameterised `bind`; it demonstrates the parameterised recursive bind over the real row, not a full interpreter or `WrapDrop`-iterative `Drop` (the production substrate keeps `FreeExplicit`'s existing iterative `Drop`). The full `Store`-parameterisation of each existing spine family (including the `CatList` continuation queue for the erased form) is mechanical production work for item 4 step 5, not a feasibility unknown: the closure-storage unification is POC-8/8b, and this spike shows it composes with the real row.
- The C2 ruling rests on stable Rust having no specialisation; if that ever changes, a single-form unification could be revisited, but it is not assumed.

## Bearing on item 4 step 2 and item 5

The substrate-identity decision is now fully grounded. Erasure axis: B, two `Store`-parameterised forms (item 5's "one parameterised substrate" holds per form, with two forms total). Pointer axis: `Store` collapses {Box, Rc, Arc} for the erased form (Box reuses `Free`) and {Rc, Arc} for the concrete form, on the real row. The single-spine invariant is met: each form reuses its existing spine, and no duplicate Box spine is introduced. The destructive steps (item 4 step 4 onward) are unblocked on the substrate-identity question.
