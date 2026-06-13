# Architecture Assessment

Snapshot: commit `2e0a7417`, branch `feat/effects`. Line numbers cited below refer to that snapshot.

## 1. What exists (system summary)

The subsystem layers as follows, bottom to top:

1. Substrate: the `Free` family (`Free`, `RcFree`, `ArcFree` erased; `FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit` typed) with `CatList`-queued continuations (the role freer-simple's FTCQueue plays), `TypeErasedValue` for the erased families, and the `WrapDrop` class for safe iterative dropping of suspended programs.
2. Rows: frunk_core `Coproduct` re-exported through `types/effects/coproduct.rs`; `CoproductBrand`/`CNilBrand` give the brand-level row; `variant_f.rs` provides `Functor`/`SendFunctor`/`RefFunctor`/`WrapDrop`/`Extract` recursion over rows; `member.rs` wraps frunk's `CoprodInjector`/`CoprodUninjector` as `Member<E, Idx>` with a `Remainder` associated type.
3. Dual-row dispatch: `Node<'a, R, S, A>` (`node.rs`) is `First(R::Of<A>) | Scoped(S::Of<A>)`; `NodeBrand<R, S>` is the functor the Free families wrap. First-order row entries are `CoyonedaBrand`-wrapped (freer encoding: any effect type becomes a Functor); scoped row entries are raw brands dispatched by case analysis.
4. Wrappers: six public program types over the six substrates, `Run`, `RcRun`, `ArcRun`, `RunExplicit`, `RcRunExplicit`, `ArcRunExplicit`, each with `pure`/`lift`/`send`/`peel`/`bind`/`map`/`expand`/`weaken`, full-row interpreters (`handle`/`run`, `handle_rec`/`run_rec`), single-effect narrowing (`handle_with`, `handle_with_handler`, `handle_scoped_with`), interposition (`interpose`, `interpose_with_replacer`, `interpose_with_rewriter`), and accumulating traversals (`accumulate_with_first_order`, `accumulate_preserving_with_first_order`).
5. Handler runtime: positional cons-lists (`Handler`/`HandlersCons`/`HandlersNil` and the scoped parallels) walked in lock-step with the row by `DispatchHandlers` (`interpreter/first_order.rs`), with three impl families for `Coyoneda`/`RcCoyoneda`/`ArcCoyoneda` heads; scoped dispatch splits into `DispatchScopedHandler`, `DispatchScopedHandlers`, `DispatchScopedBoundaryHandlers` plus per-wrapper raw and carrier protocols (`interpreter/scoped_resume.rs`, per-wrapper `raw_scoped.rs`/`boundary.rs`).
6. Scoped effects: `Catch`, `Local`, `RefLocal`, `Bracket`, `RefBracket`, `Span`, Writer `censor` and `listen`, each as an effect type with Box/Rc/Arc siblings, plus standard handler values under `standard_scoped_handlers/` implemented against the carrier protocols for all six wrappers.
7. First-order effects: `State`, `Reader`, `Except`, `Writer` (tell), `Choose`, `Empty`, `Coroutine`, `Fresh`, `Fail`, `Input`, `KVStore`, `Log`, `Output`, `Await`, with per-wrapper smart constructors and standard runners (`run_state`, `eval_state`, `exec_state`, `gets`, `run_reader`, `asks`, `run_writer`, `fold_writer`, `run_except`, `rethrow`, `note`, `run_fail`, `run_empty`, `run_choose`, `run_nondet`, `run_first_success`, `run_coroutine`, `run_fresh`, `run_input_seq`, `run_kv_store`, `run_log_vec`, `run_log_monoid`, `run_output_vec`, `run_output_monoid`, `handle_with_either`, and more), generated per wrapper by `define_run_wrapper!`.
8. Async: the `Await` base-lift effect (`await_future.rs`) embeds a boxed local `Future`; `Run::run_async` (driver in `async_interpreter.rs`) projects `AwaitBrand` out of the row at any position, awaits it, and dispatches everything else synchronously. Box-family, first-order-only, local futures.
9. Macros: `effects!`/`scoped_effects!` (rows, canonical-sorted), `handlers!`/`scoped_handlers!` (handler lists, same sort), `define_effect_row_aliases!` (named row aliases for default/Rc/Arc/scoped flavours), `define_scoped_row!` (recursive scoped row markers with `Self` substitution), `im_do!` (inherent-method do-notation), and the internal registry-driven `define_effect!`/`define_run_wrapper!` expanded by `document_module`.

## 2. Strengths

- Faithfulness with judgment. The purescript-run core (Free over VariantF, Coyoneda lifting, `lift`/`peel`/`send`, mono-in-A interpreters, `interpret`/`interpretRec` naming) is ported accurately, and where Rust forces a divergence the divergence is usually documented with the reason (mono-in-A rationale in `interpreter.rs`, single-shot versus multi-shot wrapper split, the `Node`-projection signature of `send` justified by a named GAT-normalization probe test).
- The boundary-frame mechanism (`RunRepresentation::ScopedBoundary`, `run/representation.rs`) is a genuine design contribution. Single-shot scoped operations (Box-backed `Catch`) cannot copy one `FnOnce` continuation into both the action and recovery branches; keeping the raw scoped layer and the pending continuation queue separate until branch selection solves a problem that none of the surveyed Rust effect libraries solve, and that garbage-collected reference implementations never face.
- Drop safety is treated as a first-class concern (`WrapDrop` across rows, `Node`, and effect types; wrap-depth probe tests), which matters in Rust where naive recursive `Drop` on a deep `Free` chain overflows the stack.
- Type safety throughout: typed rows, no string keys, no `unsafe` in the effects sources; the erased substrate's downcast invariant is confined behind safe constructors and documented (`run/representation.rs` lines 66 to 75).
- The macro layer is defensive: canonical ordering with duplicate rejection, a documented missing-handler-error reading guide, and a `cfg(not(feature = "effects"))` diagnostic stub so macro misuse without the feature fails comprehensibly (checked by the `effects-feature-off` recipe).
- Test breadth: forty-plus integration files including a ported heftia semantics suite (`run_heftia_semantics.rs`: State/Catch ordering, Choose/Empty, Writer censor pre/post), a composition matrix, row-canonicalisation tests, boundary-split Writer listen tests, and per-feature suites. Benchmarks exist for effect rows and scoped operations (`benches/benchmarks/effect_rows.rs`, `scoped_operations.rs`).
- The async design is runtime-agnostic by construction (a plain `Future` driven by any executor) rather than tied to tokio, which fits the library's dependency posture.

## 3. Design decisions and their costs

### 3.1 Dual rows versus heftia's current unified row

The subsystem keeps two rows: `R` for first-order operation functors and `S` for around-action scoped constructors, with `Node` dispatching between them. The docs (`node.rs` module doc, `run.md`) call this "heftia's pattern" and "the dual-row architecture (heftia's pattern)".

The heftia checkout this project references does not use dual rows. Its `Eff` is a `Freer` over a single effect list (`heftia/src/Control/Monad/Hefty/Types.hs`: `type Eff = D.Eff Freer`), where every effect has kind `(Type -> Type) -> Type -> Type`; first-order and higher-order effects coexist in one row, distinguished by the `KnownOrder` class, and algebraic handlers require `FOEs es` on the remainder (`Interpret.hs` line 75). The dual-row `Eff eh ef` design is heftia's older architecture.

Consequences of the local dual-row choice:

- Every program type carries two row parameters, and first-order-only programs (the common case) drag a `CNilBrand`/`ScopedNil` parameter through every signature, alias, and error message.
- Row-level machinery exists twice. `expand` must widen both rows with two embedding witnesses; `weaken` exists for the first-order row only; interpose/rewrite traversals are first-order-only; scoped narrowing is a separate `handle_scoped_with` with its own witness conventions.
- The benefit claimed in `run.md` is real: the two rows have statically different continuation shapes, so first-order handler lists stay simple closures while scoped handlers get the around-action protocol, and the type system prevents putting a scoped constructor in a first-order row position by construction rather than by a `KnownOrder`-style constraint.

This is a defensible Rust-specific trade (Rust lacks the lightweight constraint kinds heftia uses to make a unified row pleasant), but it is currently presented as fidelity to heftia when it is actually a divergence from the referenced heftia. It deserves a decision record stating the trade-offs against the unified-row alternative (one row of brands plus an `OrderOf`-style associated marker), so the choice is re-examinable rather than inherited. See refactoring item R12 in [refactoring-opportunities.md](refactoring-opportunities.md).

### 3.2 Mono-in-A dispatch and where the rank-2 problem resurfaces

The interpreter family adopts purescript-run's mono-in-`a` step-function shape (handler sees the operation with continuations already pointing at `NextProgram`, returns `NextProgram`), because Rust closures cannot be polymorphic in the result type. `interpreter.rs` documents this accurately and points genuinely rank-2 use cases at `NaturalTransformation` over `fold_free`.

The cost shows up exactly where purescript never goes: boundary-backed scoped programs contain selected action or recovery programs whose result type differs from the outer program's result type. Interpreting a first-order effect inside those branches requires result polymorphism after all, so `run.rs` introduces five result-polymorphic protocol traits per wrapper family (`RunFirstOrderHandler`, `RunFirstOrderReplacer`, `RunFirstOrderRewriter`, `RunFirstOrderAccumulator`, `RunFirstOrderPreservingAccumulator`, lines 125 to 553), and the convenient closure-based methods (`handle_with`, `interpose`, `handle_with_either`) are only available on `Run<R, CNilBrand, A>` (the `impl` block at line 3351). Users of scoped rows must define a struct and implement a trait to handle one first-order effect.

This is a coherent design, but the ergonomics cliff between `S = CNilBrand` and any non-empty scoped row is steep and is the kind of thing users hit immediately (the first program that combines `Catch` with a custom effect). Two mitigations are available without abandoning mono-in-A: generate the handler-struct boilerplate with a macro (a natural extension of the planned `define_effect!`), or provide blanket impls of the protocol traits for small generic-closure-shaped adapters where the effect's continuation positions make the polymorphism mechanical (State, Reader, and most built-ins qualify; the standard raw_replacers under `standard_scoped_handlers/` already are exactly this, hand-written).

### 3.3 Positional handler dispatch and syntactic row sorting

`DispatchHandlers` walks the handler cons-list and the row coproduct in positional lock-step. Order therefore matters, and the macros make order canonical by sorting both rows and handler lists with the same structural key over the parsed type syntax (`fp-macros/src/effects/row_sort.rs`).

The design has three sharp edges, all acknowledged in `handlers.rs`'s module doc but worth elevating:

- The key is syntactic. `BoxReaderBrand<BoxBrand, Env>` and `crate::brands::BoxReaderBrand<BoxBrand, Env>` sort differently; type aliases sort by their spelling, not their target. A row and a handler list that disagree in spelling produce a trait-resolution error pointing at a handler-list tail, which is decipherable only with the doc's reading guide.
- Sorting is global over the entry list, so two same-type effects cannot be distinguished at all at the macro level (duplicates are rejected), and the only path to duplicate effects is hand-written rows plus turbofished `Member` indices. There is no labeled/tagged effect mechanism (purescript-run's `*At` proxies, heftia's `Tagged`/key memberships); see [coverage-gaps.md](coverage-gaps.md).
- The canonical order is an implementation detail users must not depend on but can observe (handler execution order per layer is fixed by row position).

The handler doc itself states the durable fix: generate the row and the handlers from one effect spec. An alternative or complement is brand-keyed dispatch (search the handler list by `EBrand` at the type level instead of positional alignment), which makes handler-list order irrelevant and removes the need for the handler-side sort entirely; the row-side sort can stay as a normalization convenience. Both options are elaborated as R1 in [refactoring-opportunities.md](refactoring-opportunities.md).

### 3.4 The erased substrate and its dynamic invariants

The default family stores selected values behind `TypeErasedValue` so `bind` is O(1) on left-associated chains, with safe constructors maintaining the pairing between erased values and the continuations expecting them. Two dynamic invariants result:

- The downcast-pairing invariant (`run/representation.rs` lines 66 to 75): sound by construction for safe API users, but it is the one place where the system is correct by convention rather than by types. The invariant is well documented; keeping all constructors that touch it in one module (it currently spans `representation.rs`, the boundary code, and `row_embed.rs`) would shrink the audit surface.
- The single-shot continuation guard in `Run::handle` (`run.rs` lines 1117 to 1131): attaching the pending continuation queue is done inside a `Functor::map` over the layer with a `Cell<Option<...>>` and an `expect` that fires if a first-order effect maps its continuation hole more than once. This encodes "first-order effects on the Box family have exactly one continuation hole" as a runtime panic rather than a type-level property. The property is real (it is why `Choose` only ships on multi-shot wrappers), but nothing prevents a user-defined effect with two continuation holes from being lifted into a Box-family row and panicking at interpretation time. Worth documenting in `custom-effects.md` (which currently does not mention the single-hole requirement) and, longer-term, worth a marker trait (for example `SingleShotOp`) bounding Box-family `lift`.

### 3.5 expand and weaken are deep traversals

`expand`/`weaken` rebuild the entire program tree (`row_embed.rs`, `embed_free_node` via `Free::transform_raw`), re-embedding every layer with `CoproductEmbedder` evidence and rebuilding continuation queues, including inside boundary frames. purescript-run's `expand` is `unsafeCoerce` (O(1)); heftia's `raise`/`weakens` is per-operation at dispatch time.

In this representation the cost is structural: widening the row changes the concrete `Coproduct` type of every node, so a coercion-based `expand` would require layout guarantees Rust does not give for nested enums. Two consequences:

- The cost (O(program size) per call, with allocation) is nowhere documented; `run.md` does not mention `expand`/`weaken` at all.
- The idiomatic way to avoid it already exists and should be the documented default: write reusable program fragments generically over the row with `Member` bounds (which `Run::lift` and the smart constructors already support), so programs are born at their final row and `expand` is reserved for composing separately compiled concrete-row values. The composition-matrix test (`row_subsumption_composes_independent_reader_and_state_rows`) demonstrates the expand style; the generic style deserves equal billing in `run.md`.

A dispatch-time alternative (storing membership evidence at lift time, purescript-style open variants with runtime tags) would change the whole representation and is not worth it; documenting the cost and the generic-authoring pattern is.

### 3.6 The six-wrapper matrix and duplication

Measured similarity after normalizing the Rc/Arc/Box/Send/rc*/arc* name axes: `run.rs` versus `rc_run.rs` differ in roughly 2850 of 6712 combined lines; `rc_run.rs` versus `arc_run.rs` in roughly 2341 of 6485; `rc_run_explicit.rs` versus `arc_run_explicit.rs` in roughly 2015 of 7335. The six wrapper modules plus their `smart_constructors`/`representation`/`raw_scoped`/`boundary` submodules total roughly 35k lines, before counting the per-wrapper carrier impls inside `standard_scoped_handlers/` and the six-fold runner instantiations in `named_helpers/`.

The duplication is partially mitigated (smart constructors and runners are generated per wrapper by `define_run_wrapper!`; effect types are generated by `define_effect!`), but the interpreter methods, the five protocol traits, the representation/boundary machinery, and the scoped carriers are hand-maintained six times. The wrapper-matrix table in `types/effects.rs` (lines 45 to 50) honestly records that full Brand-level unification is blocked by real type-system limits (per-result `Clone` bounds for Rc, per-result `Send + Sync` projection bounds for Arc, GAT normalization failures probed by `tests/arc_run_normalization_probe.rs`). Those blocks are about the public type-class surface, though; they do not force the module bodies to be written six times. Template-generating the wrapper families from one source (the same approach `define_run_wrapper!` already takes for methods) is the highest-leverage refactoring available; see R2.

It is also worth asking whether all six wrappers earn their keep. `RcRunExplicit` and `ArcRunExplicit` exist for completeness of the matrix; if usage shows the Explicit family is only needed for non-`'static` payloads on the default substrate, the matrix could shrink to four (or the Explicit axis could become the only axis, with erasure as an internal optimization). The README principle (breaking changes acceptable for a better end state) applies squarely here.

### 3.7 Cell-based runners and multi-shot semantics

The generated standard runners thread state through shared interior-mutability cells: `run_state` captures `Rc<RefCell<S>>` (or `Arc<Mutex<S>>` on the Arc family) and reads the final state after interpretation (`fp-macros/src/documentation/generator_builders/state_wrapper_impl_items.rs` lines 273 to 289 and siblings); `fold_writer` accumulates by composing closures inside an `Rc<RefCell<...>>` and applying the composition at the end.

On the single-shot Box family this is unobservable. On the multi-shot wrappers it fixes one of the two classic semantics: state and logs are global across nondeterministic branches, always. heftia and purescript-run can express both semantics by handler order, because their `runState`/`runTell` thread the accumulator purely through the continuation, so re-running a captured continuation re-runs the state threading per branch:

- heftia: `runNonDet . runTell $ ...` gives each branch its own log; `runTell . runNonDet $ ...` gives one shared log (documented in the heftia semantics zoo).
- This library: with cell-based runners, both orders share one cell; branch-local accumulation has no mechanism.

`run.md`'s Handler Order section claims "handling Writer inside NonDet gives each nondeterministic branch its own observed log, while handling Writer outside NonDet accumulates output globally". No shipped runner implements the first behaviour, and no test exercises a Writer/NonDet order swap (the heftia semantics port covers State-with-Catch ordering and Choose-with-Catch, not NonDet-with-Writer). Either the claim should be corrected, or (better) pure threaded-accumulator runners should be added for the multi-shot wrappers (purescript-run's `runAccum` family is the design to port: the interpreter loop threads `s` explicitly and a multi-shot `Choose` handler that re-enters the loop per branch forks the accumulator naturally). This is the most significant semantic gap found in the review; it is also a prerequisite for porting heftia's NonDet zoo cases as tests. See R5.

### 3.8 Stack safety

The interpretation loops (`handle`, `handle_with_either`, the async drivers) are iterative; `bind` is O(1) via the continuation queue; deep drops are covered by `WrapDrop` machinery and wrap-depth probes. Two loose ends:

- `Run::handle`'s doc (line 996 to 1000) claims the method "recurses host-stack-frame per peeled layer" and points at `handle_rec` for stack safety. The body is a `loop`; per first-order layer there is no recursion. Host-stack growth can occur through nested scoped boundaries (each level of scoped nesting interprets its selected action) and through whatever the user's handlers do, but not per layer. The doc looks stale relative to the raw-step rewrite; it should be corrected, and the actual recursion budget (scoped nesting depth) stated.
- `stack_safety.rs` exercises Trampoline, Thunk, and the Coyoneda families but contains no deep `Run` program tests. A 100k-bind program through `handle`, through `handle_with` narrowing, through `interpose`, and through `expand` would pin the intended guarantees, especially because `row_embed`'s structural traversals and the interpose/accumulate traversals are the places where accidental recursion over program depth could reappear.

### 3.9 Async scope

The `Await` design is sound and honestly scoped (Box family, first-order rows, local single-shot futures, no runtime dependency). Known gaps are documented in `run.md` and `types/effects.rs` (no Rc/Arc async, no scoped-under-async, no Send-future variant). Two review notes:

- `interpreter.rs`'s module doc (lines 76 to 99) still describes the pre-`Await` world: "No `async fn` interpreter variant ships" and recommends `tokio::task::spawn_blocking` as the workaround. This directly contradicts `run_async` and must be rewritten.
- The crate-internal `handle_async` foundation driver is retained behind a `dead_code` expectation whose reason cites plan workstream W13; if it is worth keeping as the synchronous-dispatch foundation it should be documented in self-contained terms (see the documentation findings).

## 4. Comparison table

| Axis          | This library                                                                                                  | purescript-run (checkout)                                   | heftia (checkout)                                                                                 |
| ------------- | ------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Carrier       | `Free` family over `NodeBrand<R, S>`; erased and typed substrates; Box/Rc/Arc continuation storage.           | `Free (VariantF r)` newtype.                                | Local `Freer` over `Union es` with FTCQueue.                                                      |
| Rows          | Two (first-order + scoped), type-level coproduct of brands, Coyoneda-wrapped first-order entries.             | One row of functors, symbol-labeled.                        | One list of effects, order classified by `KnownOrder`; `FOEs` gates algebraic handling.           |
| Membership    | `Member<E, Idx>` over frunk; positional indices; no labels.                                                   | Symbol labels, `Row.Cons` constraints, `*At` variants.      | `In`/label/key memberships plus first-class `Membership` witnesses.                               |
| Handler shape | Mono-in-A closures (first-order); around-action handler values implementing dispatch/carrier traits (scoped). | Mono-in-a step functions; rank-2 only as documentation.     | `AlgHandler e m n ans` (continuation-passing, answer-typed); stateless/stateful/By/With variants. |
| Multi-shot    | Rc/Arc wrappers only (cloneable continuations); Box family guarded by runtime single-shot check.              | Pervasive (GC).                                             | Pervasive; multi-shot plus HOEs soundly via elaboration.                                          |
| Row widening  | `expand`/`weaken`, deep structural rebuild.                                                                   | `expand` via unsafeCoerce, O(1).                            | `raise`/`raises`/`raiseUnder`/`subsume`, per-op at dispatch.                                      |
| Higher-order  | Action-scoped subset (Catch/Local/Bracket/Span/listen/censor) with boundary frames.                           | None (catch/local are handler-level recursion over `peel`). | Full HOEs with elaboration; CC/Shift delimited continuations.                                     |
| Stack safety  | Iterative loops + `handle_rec` over `MonadRec`; queue-based bind.                                             | `runRec`/`runPure` via `MonadRec`/`Step`.                   | FTCQueue + trampolined fold.                                                                      |
| Async         | `Await` base-lift + runtime-agnostic driver (Box family).                                                     | `AFF`/`EFFECT` base effects.                                | `Emb IO` + UnliftIO; concurrency effect suite.                                                    |
