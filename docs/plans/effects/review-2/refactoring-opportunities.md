# Refactoring Opportunities

Ordered by leverage (expected payoff relative to effort and risk). Each item states the problem, the proposed end state, and the main risk. Per the project principles, breaking changes are assumed acceptable where they buy a better end state; where a type-system limitation may force a fallback, the item says what must be documented.

Foundation sweep outcome. The [foundation-sweep/](foundation-sweep/) investigation adopted FS-1 (unified row + elaboration + brand-keyed dispatch + a single closure-storage-parameterised substrate), which resolves several of these opportunities together: R1 (positional-sort footgun) is eliminated by brand-keyed dispatch (POC-2); R2 (wrapper duplication) collapses, the substrate type/interpreter/`Clone` unify (POC-8), with per-`Store` construction the residual generation; R5 (branch-local versus global semantics) is reframed by elaboration/weave (gate G2); R12 (dual rows) is superseded by the unified row. The end states proposed here are subsumed by the FS-1 rebuild tracked in [remediation-plan.md](remediation-plan.md) item 4.

## R1. One effect-spec surface: kill the spelling footgun class

Problem: rows and handler lists are independently sorted by a syntactic structural key; spelling divergence (paths, aliases) silently misaligns them, producing trait-resolution errors that need a reading guide (`handlers.rs` module doc). The doc itself names the durable fix.

End state, two compatible parts:

- A single macro (working name `effect_spec!`) that takes one list of effect brands and emits the row alias(es) (default/Rc/Arc flavours), a handler-list type alias, and a constructor function whose argument order is the user's writing order (the macro performs the canonical sort once, in one place). `effects!`/`handlers!` remain as low-level escape hatches.
- Brand-keyed dispatch as a robustness layer: add a `DispatchHandlersFor<EBrand>`-style type-level search so a handler list in any order can serve a row (positional lock-step stays as the fast path the macros guarantee). This makes hand-written handler lists order-insensitive and turns the misalignment failure mode into "missing handler for brand X" errors, which are self-explanatory.

Risk: type-level search worsens inference and error messages if done naively; prototype on a two-effect row first. The `effect_spec!` part has essentially no risk and can land alone.

## R2. Generate the six wrapper families from one template

Problem: six hand-maintained module families, 60 to 75 percent mutually identical (measured; see architecture 3.6), roughly 35k lines including submodules. Every interpreter improvement, protocol-trait change, and boundary fix must be applied six times; drift between families is a matter of time.

End state: one declarative template (a proc macro in the spirit of the existing `define_run_wrapper!`, or a well-structured `macro_rules!` with substitution tables) that takes the per-family parameters, substrate types (`Free`/`RcFree`/...), Coyoneda flavour, continuation storage (`Box<dyn FnOnce>`/`Rc<dyn Fn>`/`Arc<dyn Fn + Send + Sync>`), bound sets (none/`Clone`/`Send + Sync`), and pointer brands, and emits the wrapper struct, the protocol traits, the interpreter methods, and the representation/boundary modules. Hand-written code remains only where families genuinely differ (the documented Brand-matrix gaps, the Arc GAT-normalization workarounds).

Where blocked: if a specific method cannot be templated because the Arc family needs a structurally different body (not just different names and bounds), keep that method hand-written per family and record the reason inline; the template should cover the measured 60-75 percent overlap at minimum.

This is the highest-effort item but it converts the dominant ongoing cost (six-fold maintenance) into a one-time cost. It should precede big feature ports (streaming, CC/Shift), because each port done before it multiplies by six.

Alternative worth evaluating first: shrink the matrix. If the Explicit family subsumes the erased family's use cases at acceptable ergonomic cost (or vice versa), six becomes three before any templating.

## R3. Collapse the per-effect brand siblings and fix the prefix scheme

Problem: `BoxStateBrand<P, S>` (with `P` only ever `BoxBrand`), `StateBrand<P, S>` (Rc flavour, unprefixed), `SendStateBrand<P, S>` (Arc flavour); the unprefixed name does not match the default wrapper; `Send` names a bound while `Box` names a pointer; Bracket needs six brands. Details in organisation section 2.1.

End state: one brand per effect, `StateBrand<PB, S>`, where `PB` is a closure-storage brand (`BoxBrand`/`RcBrand`/`ArcBrand`) and a single class (working name `ClosureStorage`) with associated dyn-closure types unifies today's `ToDynFnOnce`/`ToDynCloneFn`/`ToDynSendFn`. The effect enum is then defined once over `PB`.

Likely blocker: a single associated type cannot express both `dyn FnOnce(Args) -> A` and `dyn Fn(Args) -> A` uniformly while keeping the Functor impls' composition bodies valid for both (composing onto a `FnOnce` consumes it; composing onto an `Fn` must not). If a GAT-based attempt fails, adopt the fallback naming scheme (`BoxStateBrand<S>` without the redundant `P`, `RcStateBrand<S>`, `ArcStateBrand<S>`) and document the unification attempt and the exact failure in the pointer-abstraction doc, per the project rule.

## R4. Ship the user-facing `define_effect!` and de-collide the internal one

Problem: custom effects require an eight-step manual pattern (`custom-effects.md`), while built-ins enjoy a registry-keyed internal `define_effect!` that proves the generated shape works; the two share a name. The manual pattern is also missing several requirements in its documentation (single continuation hole on the Box family, `SendFunctor`/`RefFunctor`/`Extract` needs for shared wrappers).

End state: a public `define_effect!` (or derive) taking an operation-enum-like spec and emitting the brand(s), `impl_kind!`, `Functor`/`SendFunctor`/`RefFunctor`/`WrapDrop`/`Extract` impls, and Member-generic smart constructors, modeled on data-effects' `makeEffectF` and the internal generators. The internal registry becomes invocations of the public macro (or is renamed `define_builtin_effect!` if the public one needs a different shape). `custom-effects.md` then teaches the macro first and keeps the manual pattern as the explanatory appendix.

This is the single biggest ergonomic win available; the run.md position ("only after more custom examples prove the generated shape") is now satisfiable by pointing at fourteen built-in effects generated from the same machinery.

## R5. Pure threaded-accumulator runners for the multi-shot wrappers

Problem: cell-based `run_state`/`fold_writer` hard-code shared-across-branches semantics; branch-local semantics under `Choose` is inexpressible; `run.md` claims an ordering distinction the implementation cannot deliver (architecture 3.7).

End state: `handle_accum`-style interpreters on `RcRun`/`ArcRun` (and Explicit siblings) that thread `s` through the interpretation loop, with the Choose handler re-entering per branch so each branch forks the accumulator; `run_state_threaded`/`run_writer_threaded` (final names to taste) built on them; heftia's NonDet zoo cases added as tests; `run.md`'s Handler Order section rewritten to describe what each runner family actually does.

Risk: the multi-shot scoped-dispatch path must agree on where the accumulator lives when a scoped boundary interleaves; prototype with first-order rows first.

## R6. Documentation-drift sweep

Mechanical, zero-risk, should land immediately: the eight drift items and five self-containedness violations listed in [organisation-naming-documentation.md](organisation-naming-documentation.md) sections 3 and 4 (stale node.rs/interpreter.rs/catch-handler claims, W-references, future-tense runner docs, the `run.md` catalog, the `handle` recursion claim, custom-effects.md's missing requirements).

## R7. Move effect codegen out of the documentation tree

Problem: the effect generators live under `fp-macros/src/documentation/generator_builders/`; discoverability and layering (organisation 1.2).

End state: `fp-macros/src/effects/codegen/` owns the generators and an explicit descriptor table; `document_module` keeps only the hook that asks the effects codegen to expand markers before documentation validation. Pure code motion plus one interface; no behaviour change.

## R8. Drop the alias method families

`run` = `handle` and `run_rec` = `handle_rec` on six wrappers. Keep the `handle` family (library-native vocabulary), delete the aliases, and put the purescript-run correspondence in a table in `run.md` instead of in the API. Breaking but trivial for users to follow.

## R9. Rename or reshape `handle_with_either`

It drives the entire program and returns `Result<A, matched-op>`; the name suggests purescript-run's `runExcept`, which instead narrows to a program. Either rename (for example `drive_or_intercept`) or change it to return `Run<RMinusE, CNilBrand, Result<...>>` using the accumulating traversal machinery, and let the generated `run_except` be the only either-shaped story. Decide once, before external users exist.

## R10. Document `expand`/`weaken` cost and bless row-polymorphic authoring

Add to `run.md`: `expand`/`weaken` rebuild the program (O(size), allocating); the preferred pattern for reusable fragments is Member-generic functions (already supported by `lift` and the smart constructors), which fix the row at the use site for free. Include one worked example of each style. No code change; prevents a predictable performance surprise. If a cheaper expand is ever wanted, the investigation notes belong here (nested-coproduct layout differences make a safe O(1) coercion implausible; record that conclusion).

## R11. Investigate a tail-resumptive fast path (benchmark-driven)

EvEff and koka demonstrate that operations which always resume exactly once, immediately (Ask, Get, Tell, the majority of real programs), can skip continuation-queue traffic entirely. In this architecture the analogous win would be a fused `handle_with`-style loop that pattern-matches the projected op and applies the continuation inline (which the runners already do) versus the generic Coyoneda-lower-then-dispatch path. Before designing anything: extend the benchmarks (coverage-gaps section 5) to measure where time actually goes (Coyoneda lowering, positional dispatch depth, queue churn). Only act if the numbers say so.

## R12. Write the dual-row decision record

Not a refactoring yet: a decision document comparing the current dual-row design against the unified-row-with-order-marker design the referenced heftia now uses (single brand row; per-brand associated `Order` marker; `FOEs`-style bound on algebraic handling), covering signature noise (`CNilBrand` everywhere), duplicated row machinery, diagnostics, and what each design makes impossible. If dual-row wins (plausible in Rust), the docs stop calling it "heftia's pattern" and start calling it this library's design with stated reasons; if unified wins, it becomes the headline of the next major rework and should precede R2's templating.

## R13. Housekeeping

- Sweep the 43 `dead_code` expectations once boundary-carrier wiring completes; remove the `ExplicitBoundaryOf` compatibility alias per the no-shims principle.
- Add the missing `rc_effects!`/`arc_effects!` row macros or document the alias-macro path as canonical for shared wrappers (organisation 2.3).
- Consider a `SingleShotOp` marker bound on Box-family `lift` to turn the runtime single-shot guard into a compile-time obligation (architecture 3.4); if the bound proves too infectious, document the runtime panic in `custom-effects.md` instead.
- Merge the by-wrapper `smart_constructors.rs` shells into the by-effect modules so each effect has one home per crate (organisation 1.1).
