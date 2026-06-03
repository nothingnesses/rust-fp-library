# Effects System Remediation Plan (review-1)

An implementation plan derived from [`findings.md`](findings.md). Each
work item states its adopted goal and the concrete steps to reach it,
with the justification for each step carried inline. The
[Open Questions, Decisions, Issues and Blockers](#open-questions-decisions-issues-and-blockers)
section records whether any decisions still block implementation.

This is still a working plan. Adopted decisions are folded into concrete
work-item steps; unresolved decisions remain in the Open Questions,
Decisions, Issues and Blockers section until they are adopted.

## Guiding principles

- API-breaking changes are acceptable when they lead to a better end
  state. The design, plan, and implementation prioritise correctness,
  internal coherence, and long-term architecture over compatibility
  shims for existing implementation details. `fp-library` is pre-1.0;
  when a local compatibility-preserving fix conflicts with a cleaner
  architecture, choose the cleaner architecture unless a concrete Rust
  type-system, safety, or proc-macro limitation prevents it. If such a
  limitation forces a fallback, document the limitation, the trade-offs,
  and the fallback before adopting it.
- Optimise for the smallest coherent end state, not the smallest diff. A
  finding is "addressed" when the architecture is right, not when the
  symptom is patched.

## Document conventions

- This plan intentionally does not use a separate "Resolved Decisions"
  section. Resolved or adopted decisions are folded into the
  implementation plan as concrete steps.
- The Open Questions, Decisions, Issues and Blockers section lists only
  unresolved items. If an item has a recommended approach that has been
  adopted, fold the recommendation and reasoning into the owning work
  item's concrete steps and remove the item from the list.
- Work items carry that justification inline and refer to any still-open
  decision by its title, never by list position. The Open Questions list
  can therefore be reordered or renumbered freely without breaking any
  reference.
- The plan is action-oriented. When analysis is converted into steps, the
  analysis is trimmed to the residual reasoning that keeps a rejected
  option from being reintroduced. Historical trace belongs in commit
  messages and git log.
- Each implementation commit should update this plan's status lines for
  affected work items. Status values are `Not started`, `Partial`,
  `Complete`, and `Blocked`. A `Partial` or `Blocked` status records what
  landed and what remains, so the next session can resume without
  reconstructing progress from git history.

## Root-cause framing

Most findings trace back to a single root: the substrate cross-product
(six wrappers, per-effect closure-storage siblings) is materialized by
hand rather than generated. Row-subsumption duplication, brand and impl
consistency, helper drift, and the syntactic row-sort footgun all shrink
or disappear once the cross-product is generated from one spec per effect
and per wrapper. (Generation makes the brand and class matrix consistent
and declared, but it cannot create class impls that Rust's type system
cannot express; those remain a separate capability question, W3.)

This is why the plan is generator-first: W2 (code generation) is the
central lever; wrapper-wide public work such as `expand` rides the
generated surface rather than being hand-built across six wrappers; and
only feasibility spikes run ahead of it.

## Open Questions, Decisions, Issues and Blockers

### W13 Runtime Policy Gate

Status: unresolved. This blocks implementation of the async interpreter
and the runtime-sensitive effect ports: Unlift, Concurrent, Shift / CC,
Provider when it owns runtime resources, timers, subprocesses, streams,
and any IO embedding beyond the current synchronous workaround. It does
not block documentation edits or throwaway feasibility spikes that are
explicitly discarded or folded into the decision record.

#### Async interpreter substrate

Question: should the async interpreter be expressed as a
`Future`-capable target-monad layer over the existing Run architecture,
or as a dedicated async Run substrate?

Approaches:

- Add a `Future`-capable target-monad path, using boxed/pinned futures at
  the async boundary where stable Rust cannot name recursive future
  types. Keep the existing synchronous wrappers unchanged.
- Add a dedicated async Run substrate and async wrapper family.
- Defer true async interpretation and keep only the current
  `spawn_blocking` workaround.

Trade-offs:

- The `Future`-capable target-monad path has the smallest coherent
  architectural footprint and keeps the existing sync substrate intact,
  but it may require boxed futures or helper carrier types where
  recursive async types cannot be named. That is a runtime-boundary cost,
  not a cost imposed on current pure / synchronous programs.
- A dedicated async substrate could make async behavior explicit in
  types, but it risks recreating the same cross-product explosion W2 is
  trying to control. It should be a fallback only if stable Rust cannot
  express the Future target-monad route.
- Deferring async avoids design risk now, but it leaves the major P3
  gap unresolved and keeps Unlift / Concurrent / runtime IO ports
  blocked.

Recommendation: start with a runtime-agnostic `Future`-capable
target-monad spike. Allow boxed futures at the runtime boundary, preserve
the synchronous wrappers, and adopt a dedicated async substrate only if
the spike documents a concrete Rust type-system, lifetime, safety, or
proc-macro limitation that prevents the cleaner path.

Reasoning: this best matches the guiding principles. It addresses the
architecture at the boundary that needs async behavior without spreading
async-specific wrappers across the existing surface, and it keeps the
fallback threshold concrete rather than compatibility-driven.

#### Executor and blocking model

Question: should fp-library own an async executor, depend on a specific
runtime, or return runtime-agnostic futures for callers to drive?

Approaches:

- Return runtime-agnostic futures and let callers choose the executor.
- Add a Tokio-specific runtime layer.
- Treat blocking execution as the primary policy and continue exposing
  only `spawn_blocking` guidance.

Trade-offs:

- Runtime-agnostic futures keep the core library independent and usable
  with Tokio, async-std, smol, custom executors, and no-std-adjacent
  environments if future feature work permits. The trade-off is that
  runtime-specific conveniences must live in optional adapters.
- A Tokio-specific layer is ergonomic for many users but would make a
  runtime choice for the whole library and complicate alternative
  executors.
- A blocking-only model is simplest but does not unlock the deferred
  runtime-sensitive effects and does not solve real async IO workloads.

Recommendation: the core policy should be executor-neutral: return
futures and do not own a runtime. Runtime-specific adapters may be
added later behind explicit features after the core semantics are
stable.

Reasoning: fp-library is a foundational library, not an application
runtime. Owning executor choice in core would be a long-term dependency
and semantic commitment that is not required to interpret effects
asynchronously.

#### Cancellation and resource lifecycle

Question: what cancellation guarantee should async interpretation and
runtime-owned resources provide?

Approaches:

- Treat Rust future drop as cancellation, document that dropping a
  future stops polling, and require scoped resource effects to own any
  cleanup guarantee they expose.
- Add explicit cancellation tokens and structured task scopes to the
  core async interpreter.
- Avoid cancellation semantics initially.

Trade-offs:

- Drop-as-cancellation matches Rust's base async model and avoids
  inventing an executor policy, but it does not by itself guarantee async
  finalizers run after a future is dropped. Bracket-like async cleanup
  must be designed explicitly.
- Core cancellation tokens can provide stronger structured semantics,
  but they pull the library toward a runtime and task model before the
  Future interpreter is proven.
- Avoiding cancellation semantics is fastest, but it is unsafe as a
  policy for subprocesses, streams, timers, and concurrent fibers because
  resource ownership becomes implicit.

Recommendation: adopt drop-as-cancellation as the baseline async
semantics, and require any effect that starts background work or owns
external resources to specify explicit cleanup / join / cancellation
behavior before it is ported. Do not add core cancellation tokens until
a concrete Concurrent or subprocess design proves they are needed.

Reasoning: this follows Rust's async model while keeping the resource
policy explicit. It avoids pretending that ordinary future cancellation
is enough for process or task lifecycle management.

#### IO embedding, target-monad lifting, and Unlift

Question: how should user IO futures enter and leave effect programs?

Approaches:

- Add a minimal `lift_future` / base-lift capability first, then design
  Unlift after the async interpreter and cancellation policy are proven.
- Port Unlift immediately as the central async interface.
- Keep IO embedding outside the effect system.

Trade-offs:

- A minimal lift-first path gives users a direct way to embed async IO
  without committing to the full Unlift surface. It still requires a
  clear Send/local policy and a clear cancellation story.
- Immediate Unlift is more powerful, but it requires a stable target
  monad, runtime policy, and continuation exposure story up front.
- Keeping IO external preserves purity but fails to address the review's
  largest functional async gap.

Recommendation: design and implement a minimal base-lift / `Future`
embedding capability before Unlift. Treat Unlift as a follow-on effect
that must reuse the adopted async interpreter, executor, cancellation,
and Send/local policies.

Reasoning: this unlocks practical async IO incrementally while avoiding a
large compatibility surface before the runtime boundary is stable.

#### Send, Sync, and local futures

Question: should async interpretation require `Send + Sync`, support
local futures, or expose both surfaces?

Approaches:

- Preserve the existing wrapper distinction: local futures for `Run` /
  `RunExplicit` / `RcRun` / `RcRunExplicit`, and `Send` futures for
  `ArcRun` / `ArcRunExplicit`.
- Require `Send` futures everywhere.
- Support only local futures initially.

Trade-offs:

- Preserving the wrapper distinction matches the current Box / Rc / Arc
  architecture and avoids imposing thread-safe captures on local
  programs. It is more generator work because bounds differ by wrapper.
- Requiring `Send` everywhere is simpler for threaded executors but
  excludes valid single-threaded async programs and conflicts with the
  existing Rc/local design.
- Supporting only local futures is the smallest initial implementation,
  but it would not unlock thread-safe Concurrent effects.

Recommendation: preserve the local versus `Send` distinction in the
async policy. The default local wrappers may accept local futures; Arc
wrappers and any threaded Concurrent family must require `Send` where
the executor may move work between threads.

Reasoning: this keeps the async design aligned with the existing
substrate semantics instead of collapsing them into a one-size-fits-all
runtime constraint.

#### Continuation exposure for Shift / CC

Question: should Shift / CC be ported with a restricted mono-in-`A`
model, or deferred until answer-type-polymorphic continuation capture is
designed?

Approaches:

- Defer Shift / CC until after the async base and a dedicated
  continuation-capture spike.
- Implement a restricted mono-in-`A` Shift / CC variant.
- Omit Shift / CC from the planned ports.

Trade-offs:

- Deferral keeps the current model honest and allows the design to
  account for answer-type polymorphism, one-shot versus multi-shot
  continuations, and runtime cancellation before public API is exposed.
- A restricted mono-in-`A` variant may be easier to ship, but it risks
  encoding a compatibility shim that does not match the intended Heftia
  semantics.
- Omitting Shift / CC avoids a hard design area, but it leaves one of the
  distinctive higher-order effect families unexplored.

Recommendation: defer Shift / CC implementation until a dedicated spike
proves an answer-type-polymorphic or explicitly restricted design. Do
not ship a mono-in-`A` compatibility surface unless the spike documents
the exact limitation and why the restricted form is still worth
exposing.

Reasoning: the guiding principles favor the correct architecture over a
local patch. Shift / CC changes continuation semantics deeply enough that
it should not be squeezed into the current interpreter shape without
proof.

## Baseline status

Runtime benchmark baselines were captured on 2026-05-31 while the
machine was idle. Use these as the comparison baseline for W1 / W2
changes, and rerun on an idle machine before accepting any
performance-sensitive regression or improvement claim. Do not use the
earlier partial W2-prep Criterion run as evidence, because unrelated
local processes made those numbers noisy.

Commands:

- `just check`
- `XDG_RUNTIME_DIR=/tmp just bench -p fp-library --bench benchmarks -- "Effect Rows" --sample-size 10`
- `XDG_RUNTIME_DIR=/tmp just bench -p fp-library --bench benchmarks -- "Scoped Operations" --sample-size 10`

`XDG_RUNTIME_DIR=/tmp` is only needed when the sandbox cannot write
`just` temporary files under `/run/user/1000`; it does not change the
benchmark binary. `just check` was already warm and completed in 0.21s.
Criterion reports are in `target/criterion/report/index.html`. The
following times are Criterion's `[low point high]` intervals:

| Group                              | Case                             | Baseline                          |
| ---------------------------------- | -------------------------------- | --------------------------------- |
| Effect Rows Row Canonicalisation   | direct canonical / 3 effects     | `[1.7806 ns 1.7858 ns 1.7953 ns]` |
| Effect Rows Row Canonicalisation   | subset fallback / 3 effects      | `[14.793 ns 15.867 ns 16.275 ns]` |
| Effect Rows Row Canonicalisation   | direct canonical / 5 effects     | `[2.2963 ns 2.3117 ns 2.3273 ns]` |
| Effect Rows Row Canonicalisation   | subset fallback / 5 effects      | `[16.384 ns 17.669 ns 18.395 ns]` |
| Effect Rows Handler Composition    | handlers macro / 3 handlers      | `[1.2665 ns 1.2683 ns 1.2703 ns]` |
| Effect Rows Handler Composition    | builder chain / 3 handlers       | `[1.2720 ns 1.2795 ns 1.2858 ns]` |
| Effect Rows Handler Composition    | handlers macro / 5 handlers      | `[1.7963 ns 1.8003 ns 1.8049 ns]` |
| Effect Rows Handler Composition    | builder chain / 5 handlers       | `[1.8128 ns 1.8241 ns 1.8384 ns]` |
| Scoped Operations Run Bracket      | construct scoped / BoxBracket    | `[68.950 ns 69.585 ns 70.268 ns]` |
| Scoped Operations Run Bracket      | construct manual / bind-chain    | `[56.564 ns 57.058 ns 57.545 ns]` |
| Scoped Operations Run Bracket      | dispatch scoped / BoxBracket     | `[632.58 ns 646.24 ns 655.78 ns]` |
| Scoped Operations Run Bracket      | extract manual / bind-chain      | `[164.22 ns 165.28 ns 165.99 ns]` |
| Scoped Operations RcRun Bracket    | construct scoped / Bracket       | `[96.155 ns 96.805 ns 97.473 ns]` |
| Scoped Operations RcRun Bracket    | construct manual / bind-chain    | `[93.027 ns 93.761 ns 94.738 ns]` |
| Scoped Operations RcRun Bracket    | dispatch scoped / Bracket        | `[892.18 ns 917.37 ns 929.38 ns]` |
| Scoped Operations RcRun Bracket    | extract manual / bind-chain      | `[290.84 ns 298.65 ns 303.21 ns]` |
| Scoped Operations RcRun RefBracket | construct scoped / RefBracket    | `[94.904 ns 95.944 ns 97.489 ns]` |
| Scoped Operations RcRun RefBracket | construct manual / Rc bind-chain | `[93.665 ns 94.255 ns 95.442 ns]` |
| Scoped Operations RcRun RefBracket | dispatch scoped / RefBracket     | `[849.64 ns 858.07 ns 863.72 ns]` |
| Scoped Operations RcRun RefBracket | extract manual / Rc bind-chain   | `[291.74 ns 298.24 ns 301.91 ns]` |

## Work items

Each item links to the originating finding section. The "Sequencing" note
ties the item to the generator (W2); the milestone view is in
[Suggested implementation order](#suggested-implementation-order).

### W1. Row subsumption: `expand` / `weaken`

Status: Complete. The row-embed feasibility spike is documented in
[`w1-row-embed-spike.md`](w1-row-embed-spike.md). It confirms that
all-six-wrapper support remains feasible. Adopted decision: reject
unsafe coercion and the direct `NaturalTransformation` / `hoist_free`
route; implement method-local row-embed evidence plus default `Run`
raw-step / boundary-frame traversal. The alternatives, trade-offs,
recommendation, and reasoning are recorded in the spike note. Adopted
generator-surface decision: add a separate internal
`define_run_wrapper_method!` item generator for wrapper-wide row methods,
backed by typed wrapper-method descriptors for `expand` and `weaken`.
Do not overload the effect-specific `define_run_wrapper!` marker, and do
not hand-write six wrapper copies as a migration bridge. Adopted
`weaken` scope decision: keep `expand` as the general dual-row widening
operation matching PureScript Run's row-subsumption role, and implement
`weaken` as a Heftia/OpenUnion-inspired convenience that prepends one
unused first-order effect row cell while leaving the scoped row unchanged.
Do not describe `weaken` as a PureScript Run primitive. Adopted
non-default traversal decision: broaden `expand` through crate-private
raw-transform primitives for `RcFree`, `ArcFree`, and the Explicit
substrates rather than shipping view-recursive wrapper-specific row
embedding. This best matches the long-term architecture priority because
all six wrappers keep the same structural traversal model, and the
substrate APIs own their continuation and view invariants. A narrow
view-recursion spike may be used only as a throwaway diagnostic if the
raw-transform path exposes an unexpected type-system issue; do not merge
that shape as production unless the raw-transform limitation is
documented first. Progress:
`define_run_wrapper_method!` now exists as an impl-item marker, with
typed descriptors for `expand` / `weaken`, all six wrappers, wrapper
substrate / pointer / lifetime / sendability metadata, row-embed
evidence, doc/example intent, parser diagnostics, and focused generator
tests. The shared Free / `Node` row-embed substrate now exists as
crate-private `row_embed::inner::embed_free_node`, backed by
`Free::transform_raw` so raw continuation queues can be preserved without
downcasting phantom-erased branch results. Focused tests cover widening
first-order and scoped rows while preserving pending continuations.
Default `Run::expand` now emits through the wrapper-method generator and
uses `RunRepresentation::expand` / `RunScopedBoundaryFrame::expand` so
free-backed programs and raw scoped-boundary frames both widen without
lowering through the public Free view. Focused tests cover free-backed
first-order widening and boundary-backed scoped-row widening. `RcFree`
and `ArcFree` now have crate-private `transform_raw` primitives with
continuation constructor / invocation helpers, and focused tests cover
transforming a suspended layer while preserving a pending map
continuation. `FreeExplicit`, `RcFreeExplicit`, and `ArcFreeExplicit`
now have crate-private `transform_raw` primitives that consume one
concrete view and rebuild either a pure target value or one transformed
suspended layer; focused tests cover preserving inline bind
continuations. `row_embed` now has substrate-specific helpers for
`RcFree`, `ArcFree`, `FreeExplicit`, `RcFreeExplicit`, and
`ArcFreeExplicit`, with focused tests covering first-order widening and
continuation / inline-continuation preservation across the non-default
substrates. Generated `expand` now covers `RcRun`, `ArcRun`,
`RunExplicit`, `RcRunExplicit`, and `ArcRunExplicit` through the same
`define_run_wrapper_method!` path as default `Run`; focused tests cover
first-order row widening while preserving continuations across all five
non-default wrappers, and representative `just cargo expand` checks
cover `RcRun` and `ArcRunExplicit` method shape. `row_embed` now also
has first-order-only helpers for all six substrates, and generated
`weaken` covers default `Run`, `RcRun`, `ArcRun`, `RunExplicit`,
`RcRunExplicit`, and `ArcRunExplicit` through the same wrapper-method
descriptor path. Focused tests cover no-turbofish `weaken()` calls for
all six wrappers while preserving continuations, and default `Run` also
covers weakening a scoped-boundary frame without changing its scoped row.
Representative `just cargo expand` checks cover default `Run` and
`ArcRunExplicit` `weaken` method shape. Cross-wrapper integration
coverage now composes independently-rowed Reader and State programs into
shared rows with no-turbofish `expand()` calls, then handles Reader and
State across default `Run`, `RcRun`, `ArcRun`, `RunExplicit`,
`RcRunExplicit`, and `ArcRunExplicit`.

Finding: section 9, section 11 (P0).

Goal: add `expand` as the general sound O(n) structural re-embed that
widens a program's first-order row `R` and scoped row `S` to compatible
superset rows, and add `weaken` as a narrower Heftia/OpenUnion-inspired
convenience for adding one unused first-order row cell. `expand` walks
the program and lifts each `Node` layer's first-order row `R` and scoped
row `S` (through `NodeBrand`) into the larger rows via
`CoproductEmbedder`. `weaken<E>` should have the logical shape
`Run<R, S, A> -> Run<CoproductBrand<E, R>, S, A>` for default `Run`, with
parallel shapes for the other five wrappers. PureScript's `unsafeCoerce`
`expand` has no sound analog here, because `Coproduct`s of different
arity differ in size and layout.

Steps:

- The row-embed spike has been run. It rejects an O(1)
  representation-cast `expand` because different `Coproduct` row shapes
  have different layouts, sizes, and drop behavior.
- Do not implement W1 as a generic
  `NaturalTransformation<NodeBrand<R, S>, NodeBrand<R2, S2>>` plus
  `hoist_free`. The required `CoproductEmbedder` evidence is specific to
  each `transform<'a, A>` payload type, and the existing
  `NaturalTransformation` method cannot add those per-call bounds.
- Complete for the Free / `Node` substrate. Build the row-embed as shared
  machinery, not per-wrapper, widening both the first-order and scoped
  rows symmetrically; asymmetric widening would leave one row unable to
  compose, so symmetry is the natural contract. The shared machinery is a
  crate-private row-embed helper whose method carries the concrete
  payload's embedding evidence, so Rust checks the `CoproductEmbedder`
  bounds at the call site. It uses `Free::transform_raw` rather than
  `Free::into_raw_step` because raw `Free<F, TypeErasedValue>` branches
  can store phantom-erased concrete values, not an extra `Box<dyn Any>`.
- Implement default `Run` through `RunRepresentation` /
  `RunScopedBoundaryFrame` raw-step traversal, not through public
  `Free::resume` / `Free::to_view` lowering. Boundary frames keep
  single-shot continuations outside selected scoped branches; pushing a
  `FnOnce` continuation through row `Functor::map` before branch
  selection would reintroduce the behavior the boundary representation
  exists to avoid.
- Complete for default `Run`; adopted for the remaining wrappers. Use
  raw-transform primitives as the production implementation path for
  non-default `expand`, not view-recursive helper functions built around
  public `resume` / `to_view` APIs. This deliberately accepts more
  substrate plumbing because it keeps the architecture uniform and keeps
  continuation/view invariants at the substrate boundary.
- Complete for `RcFree` and `ArcFree`. Their crate-private
  `transform_raw` methods mirror `Free::transform_raw`: transform one
  suspended raw layer from `NodeBrand<R, S>` to `NodeBrand<R2, S2>`,
  transform the pending continuation queue, preserve pure results
  without extra wrapping, and carry the wrapper-specific `Clone` /
  `Send + Sync` bounds at the method boundary. The slice also adds
  continuation constructor / invocation helpers so the raw transform owns
  erased-continuation construction consistently with existing bind and
  view traversal code.
- Complete for `FreeExplicit`, `RcFreeExplicit`, and
  `ArcFreeExplicit`. These substrates do not have erased continuation
  queues like `Free`, so their `transform_raw` methods own the
  consume-and-rebuild view traversal for `Pure` / `Wrap` while
  preserving each substrate's linear, clone, and thread-safety
  invariants.
- Complete. Extend `row_embed` with substrate-specific helpers for
  `RcFree`, `ArcFree`, `FreeExplicit`, `RcFreeExplicit`, and
  `ArcFreeExplicit` using the new raw-transform primitives. Keep the
  public method-local `CoproductEmbedder` evidence on generated wrapper
  methods, and keep the target-row payload type tied to each substrate's
  raw branch type.
- Complete. Generate `expand` for `RcRun`, `ArcRun`, `RunExplicit`,
  `RcRunExplicit`, and `ArcRunExplicit` through the same
  `define_run_wrapper_method!` descriptor path as default `Run`, with
  wrapper-specific storage, lifetime, clone, and `Send + Sync` bounds
  emitted from the typed wrapper descriptors. `RcRun` uses a dedicated
  wrapper-method impl so `expand` does not inherit unrelated
  smart-constructor `A: Clone` bounds.
- If a narrow view-recursion spike is useful for comparison, keep it
  outside the production diff and preserve it in a named stash if it is
  backed out. Only adopt view recursion as a fallback after documenting
  the exact raw-transform type-system or safety blocker and its
  consequences.
- Complete. Add `define_run_wrapper_method!` as a separate internal
  `#[document_module]` item-generator marker for wrapper-wide methods,
  using syntax such as
  `define_run_wrapper_method! { wrapper Run; method expand; }`. Keep
  `define_run_wrapper!` effect-specific so Reader / State helper
  diagnostics and descriptors do not need a second effect-optional
  grammar.
- Complete for the descriptor surface. Add typed wrapper-method
  descriptors for `expand` and `weaken` that record the wrapper,
  substrate, explicit lifetime mode, pointer mode, `Send + Sync`
  requirements, row-embed evidence, doc/example intent, and capability
  constraints. This makes the row-embed bounds and wrapper differences
  reviewable in one place before broadening to all six wrappers. Thread
  the descriptor docs and examples into emitted methods when method-body
  generation lands, and update the `weaken` descriptor text so it is
  explicitly Heftia/OpenUnion-inspired rather than PureScript Run-derived.
- Complete. Add parser and generator tests for the new marker: supported wrappers,
  supported methods, unknown method diagnostics, missing wrapper
  diagnostics, and rejection of effect-family helper methods on the
  wrapper-wide marker.
- Expose `expand` and `weaken` only through generated wrapper methods
  across default `Run`, `RcRun`, `ArcRun`, `RunExplicit`,
  `RcRunExplicit`, and `ArcRunExplicit`; do not introduce hand-written
  public copies while waiting for generation.
- Complete. Implement a shared
  crate-private row-embed helper whose method carries the concrete
  payload's `CoproductEmbedder` evidence for both the first-order row and
  the scoped row. The generated public wrapper evidence is inferable for
  the common no-turbofish call shapes covered by focused wrapper tests
  and cross-wrapper composition tests.
- Complete for default `Run`. Implement `expand` first. Add generated
  method bodies for the default `Run` wrapper that call the shared
  row-embed machinery for free-backed programs and the default
  boundary-frame path for scoped-boundary programs, widening both
  `R -> R2` and `S -> S2`.
- Complete. Implement the non-default `expand` siblings after the
  substrate raw-transform primitives and `row_embed` helpers exist, then
  wire them through the same generated descriptor path with their
  wrapper-specific storage and bound differences.
- Complete. Implement `weaken` after `expand`. Generate it as the first-order
  convenience `Wrapper<R, S, A> -> Wrapper<CoproductBrand<E, R>, S, A>`
  for all six wrappers. Use the same traversal substrate as `expand`,
  but keep the scoped row unchanged. Prefer a small first-order-only
  helper over requiring identity `CoproductEmbedder` evidence for `S` if
  the identity evidence is not naturally inferable.
- Complete. Test composition of two
  independently-rowed programs into a shared row, round-trip `expand`
  then `handle`, first-order row widening, scoped row widening, default
  `Run` boundary-frame traversal, `weaken` adding one first-order row
  cell without changing the scoped row, and inference for the common
  no-turbofish call shapes. Current focused coverage includes default
  `Run::expand` on a free-backed first-order program and on a
  boundary-backed scoped program without lowering the boundary frame,
  plus non-default `expand` first-order widening tests for `RcRun`,
  `ArcRun`, `RunExplicit`, `RcRunExplicit`, and `ArcRunExplicit` that
  assert pending continuations still run after widening. Current focused
  `weaken` coverage includes no-turbofish first-order weakening tests
  for all six wrappers and a default `Run` scoped-boundary test proving
  the scoped row remains unchanged. Cross-wrapper integration coverage
  now composes Reader-only and State-only programs into shared rows with
  no-turbofish `expand()` calls, then handles both effects across all six
  wrappers.
- Complete. Verify the generated surface with focused macro tests and
  `just cargo expand ...` checks for representative wrapper modules.
  Because there is no prior hand-written `expand` / `weaken` baseline to
  match exactly, use expansion review to confirm the six generated method
  shapes are parallel and descriptor-driven rather than to require a
  byte-for-byte old-code diff. Current coverage includes macro emission
  tests for all six `expand` and `weaken` methods, representative
  `expand` expansion review for `RcRun` and `ArcRunExplicit`, and
  representative `weaken` expansion review for default `Run` and
  `ArcRunExplicit`.

Sequencing: run the feasibility spike early; ship the public surface after
the W2 vertical slice.

### W2. Code generation for the wrapper x effect cross-product

Status: Partial. The reduction spike is documented in
[`w2-reduction-spike.md`](w2-reduction-spike.md). It rejects a single
generic wrapper as a worse version of generation because Box / Rc / Arc
continuation storage, explicit lifetimes, erased boundary frames, Arc
`Send + Sync` projection bounds, and Brand class coverage remain
mode-specific. The macro-expansion tool decision has been adopted:
`cargo-expand` is part of the Nix development environment and W2 should
use `just cargo expand ...` for vertical-slice diffs. The initial
generator/spec surface is documented in
[`w2-generator-spec-design.md`](w2-generator-spec-design.md): generate
Reader effect cells first, then the default `Run` Reader helper slice,
before expanding to Rc, Arc, and explicit wrappers. Progress:
`define_effect! { effect Reader; }` now expands, inside the
`#[document_module]` item-generator pipeline, to the three Reader cell
families (`Reader`, `SendReader`, and `BoxReader`), their `impl_kind!`
brand projections, and their documented `Clone` / `Functor` /
`SendFunctor` impls. The hand-written Reader cell block has been
replaced by the generator invocation, and `just cargo expand -p
fp-library --lib types::effects::reader` matches the pre-replacement
expansion exactly. The default `Run` Reader helper slice now has a
method-level `define_run_wrapper!` generator for `ask`, `asks`, and
`run_reader`. The hand-written default `Run` Reader helper methods have
been replaced with generator invocations. `just cargo expand -p
fp-library --lib types::effects::named_helpers::reader` matches the
pre-replacement expansion exactly. `just cargo expand -p fp-library
--lib types::effects::run::smart_constructors` differs only because
rustfmt's `reorder_impl_items` places the associated macro invocation
before ordinary methods, so generated `ask` appears before hand-written
`get`; the generated `ask` body and documentation match the original
method. Do not add rustfmt skip attributes for this ordering artifact.
The trybuild snapshot for a mismatched `Run::ask` row was updated: the
type error still names `Run::ask` and the missing `Member` bound, but
the bound-location note now points through `#[document_module]` because
the method is generated. This diagnostic-span change is accepted for the
generated slice; if later generated helpers lose the method name or the
actual trait obligation, add span-preservation work before broadening the
generator. The `RcRun` Reader helper slice is now generated for `ask`,
`asks`, and `run_reader` as well. `named_helpers::reader` matches the
pre-replacement expansion exactly for RcRun; `rc_run::smart_constructors`
has the same rustfmt order-only `ask` / `get` movement as default `Run`.
The `ArcRun` Reader helper slice is now generated for `ask`, `asks`, and
`run_reader` as well. `arc_run::smart_constructors` has the same rustfmt
order-only `ask` / `get` movement; `named_helpers::reader` differs only by
rustfmt reducing the generated `run_reader` closure body from
`{ match ... }` to `match ...`. The `RunExplicit` Reader helper slice is
now generated for `ask`, `asks`, and `run_reader` as well, with the same
smart-constructor ordering artifact and named-helper closure-body
simplification. The `RcRunExplicit` Reader helper slice is now generated
for `ask`, `asks`, and `run_reader` as well, with the same expansion
artifacts. The `ArcRunExplicit` Reader helper slice is now generated for
`ask`, `asks`, and `run_reader` as well; `named_helpers::reader` matches
the pre-replacement expansion exactly for ArcRunExplicit, and
`arc_run_explicit::smart_constructors` has the same rustfmt order-only
`ask` / `get` movement. The Reader effect-cell and six-wrapper Reader
helper vertical slice is complete. Adopted post-Reader strategy: migrate
exactly one second effect family with the current vertical-slice
discipline, then refactor the generator around typed effect and wrapper
descriptors before broadening to the remaining first-order effects. Use
`State` as the second family because it is close enough to Reader to keep
the slice mechanical while still exercising `get`, `put`, `modify`,
`run_state`, the plain / `Send` / `Box` State cells, and wrapper-specific
clone and thread-safety bounds. Do not migrate `Except`, `Writer`,
`NonDet`, `Fresh`, or other first-order families until the descriptor
refactor has landed. The State pre-replacement baselines for the effect
cell module, six smart-constructor modules, and `named_helpers::state`
were captured before the State cell replacement. `define_effect! { effect
State; }` now expands to the three State cell families (`State`,
`SendState`, and `BoxState`), their `impl_kind!` brand projections, and
their documented `Clone` / `Functor` / `SendFunctor` impls. The
hand-written State cell block has been replaced by the generator
invocation, and `just cargo expand -p fp-library --lib
types::effects::state` matches the pre-replacement expansion exactly.
State helper methods are now generated for `get`, `put`, `modify`, and
`run_state` across all six wrappers. `run::smart_constructors`,
`rc_run::smart_constructors`, `arc_run::smart_constructors`,
`run_explicit::smart_constructors`, `rc_run_explicit::smart_constructors`,
and `arc_run_explicit::smart_constructors` match the pre-replacement
expansions exactly. The `named_helpers::state` expansion differs only by
rustfmt reducing the generated `Run::run_state` and `RcRun::run_state`
closure bodies from `{ match ... }` to `match ...` and reducing the
generated `RunExplicit::modify`, `RcRunExplicit::modify`, and
`ArcRunExplicit::modify` closure bodies to single-expression closures.
The State effect-cell and six-wrapper State helper vertical slice is
complete. Adopted descriptor-refactor decision: use typed descriptors
and AST/token builders inside the existing `#[document_module]`
item-generator path. Reject a table-only template registry because it
would still leave one source file per generated item, and reject external
text templates because they would move this macro work toward string
substitution instead of structural Rust generation. The initial
descriptor model now exists in `fp-macros/src/documentation/` for the
already-proven Reader / State and six-wrapper surfaces, and
`item_generators.rs` parses supported effects, wrappers, and methods into
typed enums before selecting the current templates. Emission still uses
the existing templates for concrete Reader / State bodies, but generated
source parsing and descriptor-backed marker reconstruction now go
through token-builder helpers. `define_effect!` emission for Reader and
State now routes through `effect_items_from_descriptor`, validates the
typed effect spec has the required plain / send / box brand siblings, and
emits the existing Reader / State bodies from descriptor-builder modules
rather than fixed whole-effect template files. The Reader and State
`just cargo expand -p fp-library --lib types::effects::{reader,state}`
outputs match the pre-migration generated baselines exactly. Reader
`define_run_wrapper!` emission now routes through descriptor-backed
impl-item builders for `ask`, `asks`, and `run_reader` across all six
wrappers, and the obsolete Reader wrapper impl-item template files have
been removed. The six Reader smart-constructor module expansions and
`named_helpers::reader` match their pre-migration generated baselines
exactly. State `define_run_wrapper!` emission now routes through
descriptor-backed impl-item builders for `get`, `put`, `modify`, and
`run_state` across all six wrappers, and the obsolete State wrapper
impl-item template files have been removed. The six State-touched
smart-constructor module expansions and `named_helpers::state` match
their current generated baselines exactly. With Reader and State helper
templates removed, `item_generators.rs` now delegates supported
`define_run_wrapper!` combinations through descriptor builders and keeps
only targeted unsupported-combination diagnostics in the fallback match.
The verified W3 wrapper capability matrix is now encoded in
`WrapperSpec` through owned / ref / send brand capability fields, and
descriptor validation now checks row-bound support, wrapper capability
rules, and owned / ref / send brand capability requirements before
helper generation. Tests cover supported and unsupported matrix
combinations. The `Except` pre-migration expansion baselines and
first-order surface inventory were captured in
[`w2-except-baseline-inventory.md`](w2-except-baseline-inventory.md)
before descriptor code changes. The `Except` effect cell is now emitted
through `define_effect! { effect Except; }` via a distinct typed-abort
descriptor shape, and the generated `types::effects::except` expansion
matches the captured baseline exactly. The six `Except::throw`
constructors are now descriptor-backed through `define_run_wrapper!`
markers for `Run`, `RcRun`, `ArcRun`, `RunExplicit`, `RcRunExplicit`,
and `ArcRunExplicit`; each touched smart-constructor expansion matches
the captured baseline exactly. The six `run_except` runners are now
descriptor-backed through `define_run_wrapper!` markers in
`named_helpers::except`; the regenerated expansion matches the captured
baseline exactly. The `throw_unit`, `rethrow`, `note`, and
`from_option` convenience helpers are now descriptor-backed across all
six wrappers, and the complete `Except` vertical slice is
expansion-equivalent to the captured W2 baselines.

Finding: section 4, section 11 (P0).

Goal: generate the six wrappers, the per-effect `Box` / `Send` / plain
brand siblings, their class impls, and the smart constructors from one
spec per effect and one per wrapper, replacing the hand-maintained
parallel code and declaring capability rules (such as multi-shot-only
`choose` / `ref_bracket`) instead of letting them drift.

Steps:

- The reduction spike has been run. It records why one wrapper generic
  over closure storage, pointer brand, or Free substrate does not remove
  the real mode-specific behavior: `FnOnce::call_once` consumes `self`
  out of a shared pointer, `Send + Sync` is baked into Arc trait objects
  and row projections, explicit wrappers carry lifetimes and different
  Brand class coverage, and stable Rust still lacks the per-`A`
  quantification needed for one clean Arc explicit class matrix. Proceed
  with generation.
- Specify each effect and wrapper via a co-located `define_effect!` /
  `define_run_wrapper!` invocation in its module, matching the
  file-per-effect layout and the `#[fp_macros::document_module] mod inner`
  convention, rather than a central manifest or a `build.rs` step, so
  rust-analyzer support and the doc audit keep working and each spec is
  reviewable in isolation.
- Emit `#[document_module]`-compatible doc attributes by extending the
  `documented_helper_impls!` precedent
  (`fp-macros/src/documentation/item_generators.rs`), and carry
  user-facing doctests in the spec (templates only for mechanical items)
  so generated items still pass the `document_examples` audit
  (`scripts/document_examples.rs`); do not exempt generated items, which
  would quietly erode a documentation guarantee the project spends real
  effort to keep. Rule out `macro_rules!` for the public surface, since
  its generated methods are invisible to `#[document_module]` validation
  (prior decision D6).
- Scope the first iteration to the first-order effects, their brand
  siblings, and the six wrappers; defer scoped-effect generation until
  after W8 stabilizes the boundary / carrier / residual model, so the
  headline de-duplication win is not coupled to the riskiest subsystem.
- Migrate incrementally and behavior-preservingly: keep the existing
  effects test suite green against generated wrappers (which requires the
  generated public API to stay path-compatible) and golden-diff macro
  expansion during the vertical slice, accepting brief coexistence rather
  than a big-bang cutover, because the existing tests are the cheapest
  high-fidelity behavioral oracle. Use `just cargo expand ...`; the
  required `cargo-expand` binary is provided by the Nix development
  environment.
- Vertical slice: migrate Reader and one wrapper end-to-end, diffing the
  generated output against the current code, then migrate the rest,
  encoding the capability rules in the spec so the `choose` /
  `ref_bracket` asymmetry is declared, not drifted. Adopt the
  `w2-generator-spec-design.md` order: Reader effect cells first, default
  `Run` Reader helpers second, then Rc, Arc, and explicit siblings.
- Complete for State cells and captured for wrapper follow-up. Before
  the State cell replacement, the pre-replacement `just cargo expand`
  baselines were captured for
  `types::effects::state`, `types::effects::run::smart_constructors`,
  `types::effects::rc_run::smart_constructors`,
  `types::effects::arc_run::smart_constructors`,
  `types::effects::run_explicit::smart_constructors`,
  `types::effects::rc_run_explicit::smart_constructors`,
  `types::effects::arc_run_explicit::smart_constructors`, and
  `types::effects::named_helpers::state`.
- Complete. Extend `define_effect!` only far enough to generate the State
  effect cell families and brands (`State`, `SendState`, and `BoxState`)
  plus their class impls, then replace the hand-written State cell block
  with a co-located `define_effect! { effect State; }` invocation.
- Extend `define_run_wrapper!` only far enough to generate State helper
  methods across all six wrappers: `get`, `put`, `modify`, and
  `run_state`, preserving each wrapper's existing clone, lifetime, and
  `Send + Sync` bounds. Complete for all six wrappers: default `Run`,
  `RcRun`, `ArcRun`, `RunExplicit`, `RcRunExplicit`, and
  `ArcRunExplicit`. Each generated slice has been compared against the
  captured expansion, and the rustfmt-only formatting artifacts are
  documented in this W2 status line.
- Adopted. Refactor the generator around typed effect and wrapper
  descriptors, using AST/token builders rather than external text
  templates. Keep the public macro syntax (`define_effect!` and
  `define_run_wrapper!`) unchanged, and keep the current Reader and
  State generated expansions as the acceptance baseline during the
  refactor.
- Complete. Add a descriptor module under `fp-macros/src/documentation/`
  with typed Rust data for `EffectSpec`, effect cell variants, method
  families, `WrapperSpec`, pointer mode, wrapper substrate, explicit
  lifetime mode, sendability, required brand siblings, required row
  bounds, handler names, and capability rules such as multi-shot-only
  operations. Start with only the already-proven Reader and State
  surfaces, and route supported effect / wrapper / method parsing through
  descriptor enums while emission still uses the current templates.
- Complete. Add token-builder helpers that turn descriptors into `syn` /
  `quote` output inside the `#[document_module]` item-generator pipeline.
  Match identifiers and paths structurally in Rust code; do not introduce
  string substitution templates or generated source files outside the
  macro crate. The first helper slice centralizes token-to-AST parsing
  and descriptor-backed `define_effect!` / `define_run_wrapper!` marker
  reconstruction; concrete Reader / State bodies still migrate in the
  following steps.
- Complete. Migrate `define_effect!` for Reader and State from fixed
  whole-effect template files to descriptor builders. Reader and State
  effect emission now goes through `effect_items_from_descriptor`, and
  the generated `types::effects::reader` and `types::effects::state`
  expansions match the current generated baselines exactly.
- Complete. Migrate `define_run_wrapper!` for Reader helpers (`ask`,
  `asks`, `run_reader`) across all six wrappers from per-item templates
  to descriptor builders. Each touched smart-constructor module and
  `named_helpers::reader` matches the current generated expansion
  exactly.
- Complete. Migrate `define_run_wrapper!` for State helpers (`get`,
  `put`, `modify`, `run_state`) across all six wrappers from per-item
  templates to descriptor builders. Each touched smart-constructor
  module and `named_helpers::state` matches the current generated
  expansion exactly.
- Complete. Once Reader and State are descriptor-backed and
  expansion-equivalent, remove the obsolete per-item Reader / State
  template files and replace the large wrapper/effect/method match in
  `item_generators.rs` with descriptor lookup plus targeted
  unsupported-combination diagnostics.
- Adopted post-W12 sequencing. Return to W2/W3 before W13; W13 remains
  last and blocked on the runtime-policy gate until the generator and
  wrapper-capability architecture is no longer partial.
- Complete. Fold the W3 capability matrix into the descriptor model
  before the next effect migration. Wrapper descriptors now include owned
  / ref / send brand capability fields alongside explicit-lifetime
  support, pointer mode, substrate, row-bound requirements, and
  multi-shot-only helper constraints.
- Complete. Add descriptor validation that rejects unsupported wrapper /
  effect / helper combinations with targeted diagnostics. The validation
  covers row-bound support, wrapper capability rules, and owned / ref /
  send brand capability requirements, with tests for supported and
  unsupported combinations in each class.
- Inventory the remaining hand-written first-order effect and helper
  families before editing them. Treat `Except`, `Empty`, `Choose` /
  `NonDet`, and the first-order `Writer` helper surface as W2 migration
  targets. Keep scoped operations such as Writer `listen` / `censor`,
  Catch, Local, Bracket, RefBracket, and Span out of this W2 migration
  until W8 evaluates the scoped-dispatch boundary.
- Adopted. Model `Except` with a distinct typed-abort descriptor shape
  rather than reusing or over-generalizing the fixed-message `Fail`
  shape. The descriptor must model `ExceptBrand<E>`, preserve the
  distinction between typed exception handling and `FailBrand`, and keep
  scoped `Catch` out of W2 until W8 evaluates scoped-dispatch
  consolidation.
- Complete. Capture `Except` expansion baselines before editing code.
  The capture used the `just cargo expand` recipe with the `effects`
  feature for `types::effects::except`, all six smart-constructor
  modules containing `throw`, and
  `types::effects::named_helpers::except`. The inventory records the
  current first-order surface explicitly: the `Except` cell and brand,
  `Functor` / `SendFunctor` impls, six `throw` constructors, six
  `run_except` runners, and the `throw_unit`, `rethrow`, `note`, and
  `from_option` convenience helpers across all six wrappers. See
  [`w2-except-baseline-inventory.md`](w2-except-baseline-inventory.md)
  for commands, line counts, SHA-256 hashes, and surface details.
- Complete for the effect-cell slice. Add `Except` to the descriptor
  model as a distinct typed-abort operation shape, register the
  parameterized `ExceptBrand<E>` effect family, and register the planned
  first-order helper methods: `throw`, `throw_unit`, `rethrow`, `note`,
  `from_option`, and `run_except`. Wrapper-specific lifetime,
  per-wrapper `Clone` / `Send + Sync`, and `Result` runner emission have
  been fulfilled by the `throw` and `run_except` steps below.
- Complete. Generate `define_effect! { effect Except; }` through
  descriptor builders first. The hand-written `Except` cell block has
  been replaced with the co-located marker, and the regenerated
  `types::effects::except` expansion matches the captured baseline
  exactly: 237 lines and SHA-256
  `db2688aef4664ffd4018bfbdddd550c911798cde0394809014eaffdf97315407`.
- Complete. Generate the six `throw` constructors. The generated
  descriptor-backed methods preserve each wrapper's current error bounds
  exactly: single-shot local wrappers do not add `Clone`; multi-shot
  local wrappers require `Clone`; Arc wrappers require
  `Clone + Send + Sync`; explicit wrappers use their explicit lifetime
  in row membership and effect payloads. The regenerated expansions
  match the captured W2 baselines exactly: `run::smart_constructors`
  2041 lines, SHA-256
  `1c8a585f0e41931c9c9c0a6a28a6a517bef01dd64c736a605a1768bd7b1fd64d`;
  `rc_run::smart_constructors` 2458 lines, SHA-256
  `68debfe03c8f5f4783c2fdf639f28ba6e19bf54923e59d58be4d03c6077516b5`;
  `arc_run::smart_constructors` 2768 lines, SHA-256
  `debb3e6df6562611f801125fbb9dbd986019210604c594b01ef1c02c38567836`;
  `run_explicit::smart_constructors` 1962 lines, SHA-256
  `c2201763868d3ae46da9e313e70227aea6ea78b766e6f740713037af0d0606f2`;
  `rc_run_explicit::smart_constructors` 2216 lines, SHA-256
  `40831e83b9640c979594579280a85188fdcf04693cfc8d4122524cf085dca909`;
  and `arc_run_explicit::smart_constructors` 2662 lines, SHA-256
  `a071c9850da431e2edb84d872e501d630056f3d39cd3e4d5392a26fe31ce45e3`.
- Complete. Generate the six `run_except` runners after `throw`. The
  generated descriptor-backed methods preserve the current
  `Result<A, ErrorType>` output, row-remainder bounds, `NodeBrand`
  bounds, explicit-lifetime behavior, and Arc `Send + Sync` projection
  requirements. The regenerated `types::effects::named_helpers::except`
  expansion matches the captured W2 baseline exactly: 1950 lines and
  SHA-256
  `fde33e15dd651c127e580ccf5ad06776dec1233884eab3438e807da79fc98ff5`.
- Complete. Generate the convenience helpers after the core constructor
  and runner are descriptor-backed: `throw_unit`, `rethrow`, `note`, and
  `from_option`. The generated helpers delegate to generated `throw`,
  generated `note`, and the existing `pure` surfaces rather than
  duplicating row-construction logic. The regenerated
  `types::effects::named_helpers::except` expansion still matches the
  captured W2 baseline exactly: 1950 lines and SHA-256
  `fde33e15dd651c127e580ccf5ad06776dec1233884eab3438e807da79fc98ff5`.
- Complete. The `Except` slice is descriptor-backed and
  expansion-equivalent to the captured baselines for the effect module,
  all touched smart-constructor modules, and `named_helpers::except`.
- Complete. Capture `Empty` / `Choose` and `NonDet` expansion baselines
  before editing code. The capture used the `just cargo expand` recipe
  with the `effects` feature for `types::effects::empty`,
  `types::effects::choose`, all six smart-constructor modules containing
  `empty`, and `types::effects::named_helpers::nondet`. The inventory
  records the current first-order surface explicitly: the `Empty`,
  `Choose`, `SendChoose`, and `BoxChoose` cells, the six `empty`
  constructors, the four multi-shot-only `choose` constructors, six
  `run_empty` runners, and the multi-shot-only `run_choose`,
  `run_nondet`, and `run_first_success` helpers. See
  [`w2-nondet-baseline-inventory.md`](w2-nondet-baseline-inventory.md)
  for commands, line counts, SHA-256 hashes, and surface details.
- Complete for the effect-cell slice. Generate `Empty` through
  descriptors first, preserving `Clone`, `Copy`, `Default`, `Functor`,
  and `SendFunctor` expansion shape before touching wrapper helpers. The
  hand-written `Empty` cell block has been replaced with the co-located
  marker, backed by the distinct `PhantomAbort` descriptor shape, and
  the regenerated `types::effects::empty` expansion matches the captured
  W2 baseline exactly: 301 lines and SHA-256
  `22f5be128265a08b75aef449ee44582ea80d51a927746f10b375c23e37502ef9`.
- Complete. Generate the six `empty` constructors through wrapper
  descriptors and compare the affected smart-constructor module
  expansions against the captured W2 NonDet baselines. The generated
  methods preserve each wrapper's current row-member type, explicit
  lifetime behavior, clone projection, and Arc `Send + Sync`
  requirements. The regenerated expansions match the captured W2
  baselines exactly: `run::smart_constructors` 2041 lines, SHA-256
  `1c8a585f0e41931c9c9c0a6a28a6a517bef01dd64c736a605a1768bd7b1fd64d`;
  `rc_run::smart_constructors` 2458 lines, SHA-256
  `68debfe03c8f5f4783c2fdf639f28ba6e19bf54923e59d58be4d03c6077516b5`;
  `arc_run::smart_constructors` 2768 lines, SHA-256
  `debb3e6df6562611f801125fbb9dbd986019210604c594b01ef1c02c38567836`;
  `run_explicit::smart_constructors` 1962 lines, SHA-256
  `c2201763868d3ae46da9e313e70227aea6ea78b766e6f740713037af0d0606f2`;
  `rc_run_explicit::smart_constructors` 2216 lines, SHA-256
  `40831e83b9640c979594579280a85188fdcf04693cfc8d4122524cf085dca909`;
  and `arc_run_explicit::smart_constructors` 2662 lines, SHA-256
  `a071c9850da431e2edb84d872e501d630056f3d39cd3e4d5392a26fe31ce45e3`.
- Complete. Generate the six `run_empty` helpers through named-helper
  descriptors, preserving first-order row removal and unreachable-branch
  handling. The generated methods preserve the current `Option<A>`
  result, `map(Some)` runner shape, row-remainder bounds, `NodeBrand`
  projection bounds, explicit-lifetime behavior, and Arc `Send + Sync`
  requirements. The regenerated
  `types::effects::named_helpers::nondet` expansion matches the captured
  W2 baseline exactly: 2619 lines and SHA-256
  `286c53148fd7e95ac8d2c2137076af3baced19bd4bb50685cd89201d8d7e726e`.
- Complete for the effect-cell slice. Generate `Choose`, `SendChoose`,
  and `BoxChoose` through descriptors, preserving the current owned,
  clone, boxed single-shot, and thread-safe class capability
  distinctions. The hand-written cell block has been replaced with the
  co-located marker, backed by the distinct `BooleanChoiceContinuation`
  descriptor shape. The regenerated `types::effects::choose` expansion
  is 492 lines with SHA-256
  `ea3620d81adef92f567696649390d43a22a864ef7fc21a1d715850208ee15ee0`;
  this is intentionally not hash-identical to the captured baseline
  `9c51dae5724fdee8ec1133dbf2e457b9520a3d0d548ab8480c348d25d90fddfa`
  because generated-item attribute normalization reorders the generated
  `Parameters`, `Returns`, and `Examples` documentation section headers
  around the existing example body. The diff is limited to those
  documentation-section placements; the generated enums, kind impls,
  class impls, method signatures, and method bodies are unchanged.
- Complete. Generate the multi-shot-only `choose` smart constructors,
  `run_choose`, `run_nondet`, and `run_first_success` named helpers for
  `RcRun`, `ArcRun`, `RcRunExplicit`, and `ArcRunExplicit`, encoding the
  `MultiShot` capability rule in the `Choose` method descriptors so
  single-shot `Run` / `RunExplicit` wrappers are rejected by descriptor
  validation rather than wrapper-specific exceptions. The regenerated
  smart-constructor expansions match the captured W2 baselines exactly:
  `rc_run::smart_constructors` 2458 lines, SHA-256
  `68debfe03c8f5f4783c2fdf639f28ba6e19bf54923e59d58be4d03c6077516b5`;
  `arc_run::smart_constructors` 2768 lines, SHA-256
  `debb3e6df6562611f801125fbb9dbd986019210604c594b01ef1c02c38567836`;
  `rc_run_explicit::smart_constructors` 2216 lines, SHA-256
  `40831e83b9640c979594579280a85188fdcf04693cfc8d4122524cf085dca909`;
  and `arc_run_explicit::smart_constructors` 2662 lines, SHA-256
  `a071c9850da431e2edb84d872e501d630056f3d39cd3e4d5392a26fe31ce45e3`.
  The regenerated `types::effects::named_helpers::nondet` expansion also
  still matches the captured W2 baseline exactly after the remaining
  NonDet runner migration: 2619 lines and SHA-256
  `286c53148fd7e95ac8d2c2137076af3baced19bd4bb50685cd89201d8d7e726e`.
- Complete. Mark the `NonDet` slice complete: `types::effects::empty`,
  `types::effects::choose`, all touched smart-constructor modules, and
  `types::effects::named_helpers::nondet` are descriptor-backed and
  expansion-equivalent or intentionally documented where generated
  documentation attributes change shape.
- Migrate the first-order `Writer` helper surface after `NonDet`,
  preserving existing scoped Writer semantics by leaving `listen` /
  `censor` and their carriers in the W8-scoped bucket.
- Mark W2 complete only after the remaining first-order generated
  surfaces are descriptor-backed, expansion-equivalent or intentionally
  documented where rustfmt changes shape, and covered by the matrix
  validation from W3. If a concrete Rust type-system, lifetime, safety,
  or proc-macro limitation prevents descriptor-backed migration, stop
  and document the blocker, alternatives, recommendation, and reasoning
  before adding any temporary template-per-item copy.

### W3. Brand and class capability audit, then decide the gaps

Status: Complete. The verified wrapper capability matrix has been folded
into the W6 effects guide, the stale `ArcRunExplicitBrand` docs now
include `SendRefPointed`, and the current gaps are documented as
intentional Rust-bound limitations. The matrix is also encoded in W2's
wrapper descriptors, descriptor validation, and generator tests before
broadening the remaining first-order migrations.

Finding: section 7, section 11 (P0).

Goal: replace the (incorrect) assumption that the Erased wrappers can
delegate to existing Free brands with a verified capability matrix and a
per-gap decision. Verified state: `RunExplicitBrand` has Functor /
Pointed / Semimonad plus the Ref trio; `RcRunExplicitBrand` has Pointed
plus the Ref trio (no owned Functor or Semimonad); `ArcRunExplicitBrand`
has SendPointed and SendRefPointed; the Erased trio and the non-explicit
Free family are unbranded (no `FreeBrand` / `RcFreeBrand` /
`ArcFreeBrand`).

Steps:

- Produce the verified per-brand, per-class matrix and fold it into the
  W6 doc.
- Fix the stale `ArcRunExplicitBrand` docs, which claim "limited to
  `SendPointed`" but `SendRefPointed` is implemented.
- Decide each gap, close or document: branding the Erased trio first
  requires branding `Free` / `RcFree` / `ArcFree` and proving the erased
  `Box<dyn Any>` `Run` admits a clean `Kind` projection, so do it only if
  a concrete generic use case needs it; document gaps forced by Rust
  limits (for example `ArcRunExplicitBrand` `SendFunctor` /
  `SendSemimonad`) rather than chasing them.
- Complete. Feed the decided matrix into W2's wrapper and effect specs,
  including owned / ref / send capability fields, explicit-lifetime
  support, pointer mode, substrate, row-bound requirements, and
  multi-shot-only helper constraints.
- Complete. Add matrix-backed generator validation and tests so
  unsupported wrapper / effect / helper combinations fail with targeted
  diagnostics instead of silently generating inconsistent APIs.

Sequencing: before or alongside W2 so the generator emits a consistent,
intentional matrix.

### W4. Feature-gate the subsystem

Status: Complete for the feature-gating implementation. The default-off
`effects` feature, gated modules and flat re-exports, explicit macro
surface, internal-only `raw_effects!` path, feature-off macro diagnostic
fixture, local `just` verification recipe, and first hosted CI workflow
are implemented.

Finding: section 7, section 11 (P1).

Goal: add a single default-off `effects` cargo feature gating the effects
modules and their public re-exports, with granular sub-features
(`effects-arc`, `effects-explicit`) considered only if compile-cost data
later justifies the `cfg` and CI-matrix complexity. Default-off is an
API-breaking change for users who currently get effects without features,
but it is the coherent end state for an optional heavy subsystem in a
pre-1.0 crate. The gating must account for two macro path families: row
macros (`effects!`, `scoped_effects!`, `define_scoped_row!`,
`define_effect_row_aliases!`) emit `::fp_library::brands::` paths
(`CoproductBrand` / `CNilBrand` in `brands/effects.rs`; the
`CoyonedaBrand` family in the general `brands.rs`), while handler macros
(`handlers!`, `scoped_handlers!`) emit
`::fp_library::types::effects::handlers::` paths, so the two fail at
different gated locations when the feature is off. Feature-off use of
the public effect macros must produce a clear "enable the `effects`
feature" diagnostic at the `fp-library` public macro surface. The
low-level `raw_effects!` macro is not part of the crate-root public
surface; keep it only under `fp_library::__internal` for generated and
expert code. Direct `fp_macros::...` use remains an expert escape hatch
with documented limitations.

Implemented steps:

- Complete. Gate the effects modules and their public re-exports, not only the
  module declarations: `pub mod effects;` and `pub use effects::*;` in
  `brands.rs`, and `pub mod effects;` and the flat `effects::{...}`
  re-export in `types.rs`.
- Complete. Add `effects = []` to `fp-library/Cargo.toml` and leave
  `default = []` unchanged, so users opt into the effects subsystem
  explicitly.
- Complete. Replace the broad `pub use fp_macros::*` public macro
  re-export with an explicit re-export list: keep non-effects macros
  always exported, gate public effect macros behind `feature = "effects"`,
  and provide feature-off shim macros with the same names that expand to a
  clear `compile_error!("enable the `effects` feature")` style diagnostic.
- Complete. Do not re-export `raw_effects!` at the crate root. Keep the low-level
  macro under `fp_library::__internal` only, gate that hidden re-export
  with `feature = "effects"`, and provide a hidden feature-off shim there
  if generated or expert code reaches the internal path with the feature
  disabled.
- Complete. Document the intentional root-level removal of `raw_effects!` as
  cleanup of an accidental macro surface from the old broad
  `fp_macros::*` re-export. Public examples should use `effects!`,
  `scoped_effects!`, row aliases, or handlers instead.
- Complete. Document that invoking the effect macros directly through `fp_macros`
  while `fp-library/effects` is disabled is unsupported because the
  proc-macro crate cannot observe `fp-library`'s active features.
- Complete. Confirm the rest of the crate builds with the feature off (no
  non-effects code depends on effects).
- Complete. Add local `just` verification coverage for feature-off builds and
  feature-off macro diagnostics. The feature-off macro check should use a
  dedicated fixture crate or compile-test harness with `fp-library`
  default features disabled, invoke at least one public effect macro, and
  assert that the diagnostic tells the user to enable the `effects`
  feature.
- Complete. Add hosted CI, preferably a first GitHub Actions workflow unless the
  repository adopts another hosted CI platform before W4 implementation.
  CI must run the existing full feature-on verification and the new
  feature-off checks, including the feature-off macro diagnostic test.
- Complete. Add feature-on examples/docs for effects imports and update any
  crate-level docs that currently imply effects are always available.

### W4a. Cfg-gate effects-only tests

Status: Complete. Effects-only integration tests now carry file-level
`#![cfg(feature = "effects")]`; effects-specific docs modules and the
effects-only `Free` unit-test support are feature-gated; and
`just effects-feature-off` now runs the widened no-default-features
package test surface after the fast library check.

Finding: follow-up to W4.

Goal: make the default-off test surface structurally match the optional
subsystem boundary. Effects-specific integration tests should be present
and exercised under `--all-features`, but should not make ordinary
`--no-default-features` test commands fail just because the optional
subsystem is disabled. Keep the explicit feature-off macro diagnostic
fixture from W4 as the acceptance test for the public macro shims; cfg
gating effects tests improves test-suite hygiene but does not replace
that diagnostic contract.

Implemented steps:

- Complete. Inventory `fp-library/tests/*.rs` and identify files whose entire
  purpose is the effects subsystem: files importing `fp_library::types::effects`,
  using effect row or handler macros, or constructing `Run` / `RcRun` /
  `ArcRun` wrappers. Add file-level `#![cfg(feature = "effects")]` to
  those effects-only integration tests.
- Complete. Do not blanket-gate mixed-purpose tests. If a test file covers both
  core library behavior and effects behavior, split the effects cases
  into an effects-only file first, then gate only that file.
- Complete. Keep `fp-library/tests/compile_fail.rs` dual-mode: under
  `feature = "effects"` it should continue to run the normal UI suite;
  under `not(feature = "effects")` it should run the feature-off macro
  diagnostic fixture.
- Complete. After the effects-only integration tests are cfg-gated, broaden
  `just effects-feature-off` from the narrow compile-fail test target to
  `just --one test -p fp-library --no-default-features`, while keeping
  the current `check -p fp-library --no-default-features --lib` fast
  smoke check first.
- Complete. Gate the effects-specific docs modules and effects-only
  internal unit-test support that are compiled by the widened
  no-default-features test surface.
- Complete. If benches or examples compile effects code under no-default-features
  checks, gate or split them using the same rule: effects-only targets
  get `cfg(feature = "effects")`; mixed targets are split before gating.
- Complete. Keep the CI feature-off job pointed at `just effects-feature-off`, so
  the workflow follows the local verification contract instead of
  duplicating cargo arguments in YAML.
- Complete. Verify with `just fmt`, `just filtered check`, `just filtered clippy`,
  `just filtered test`, and `just effects-feature-off`.

### W5. Row-macro Rc / Arc symmetry

Status: Not started.

Finding: section 9.

Goal: close the `rc_effects!` / `arc_effects!` gap as part of the W2 macro
redesign rather than adding throwaway macros now. By default, defer and
decide the row-macro surface inside W2; add the two macros early only if
their names are committed to survive W2, or a concrete near-term Rc / Arc
need predates it. When the work does happen, thin sibling macros sharing
the worker beat changing `effects!`'s syntax or leaving the asymmetry a
documentation burden.

Steps (only if done early, ahead of W2):

- Add `rc_effects!` / `arc_effects!` entry points and workers reusing
  `build_coproduct_row` with `RcCoyoneda` / `ArcCoyoneda`.
- Add tests mirroring the `effects!` tests; add docs.

Sequencing: deferred into W2 by default.

### W6. Documentation consolidation

Status: Complete for the current consolidation pass. The root effects
guide, wrapper capability matrix, and known-limitations list have landed
in `types/effects.rs`; wrapper and effect modules that restate wrapper,
pointer-brand, or capability conventions now link back to that guide.

Finding: sections 6, 8, 9.

Goal: one authoritative effects-guide section (module-level in
`types/effects.rs`) covering the substrate axis (Erased vs Explicit), the
shot / sharing axis (Box / Rc / Arc), the prefix legend (Box / Send /
plain), the brand and class capability matrix (from W3), and a
consolidated known-limitations list (mono-in-`A`, `Fn` vs `FnOnce`,
`ArcRunExplicit` coverage, Erased downcasts). Per-module docs link to it
rather than restating it.

Steps:

- Write the guide and the capability matrix.
- Link existing module docs to it.
- Ensure ASCII-only and lychee link checks pass.

### W7. Syntactic row-sort footgun

Status: Complete for the current documentation hardening. The row-sort
limitation is documented, handler-list diagnostics now include a worked
spelling-mismatch example and `HandlersNil` guidance, and the durable
elimination remains assigned to W2 generation.

Finding: section 9.

Goal: sharpen the docs and the downstream diagnostic now; the durable fix
is W2 (generating the row and the handlers from one spec removes the
mismatch entirely). Macro-time detection is impossible because
proc-macros cannot resolve aliases or imports, so a spelling mismatch is
not rejected at expansion and instead surfaces as a trait/type error at
the `handle` call site.

Steps:

- Strengthen the `row_sort` / `handlers` docs with a worked mismatch
  example.
- Improve the "not implemented for `HandlersNil`" guidance.
- Note generation (W2) as the eventual elimination.

### W8. Scoped-dispatch design note and consolidation evaluation

Status: Partial. The scoped-dispatch split and the
`NextProgram` / `ActionProgram` invariant are documented in
`types/effects/interpreter.rs`. Remaining: evaluate consolidation after
W2 and only if Writer `listen` / `censor` and Span semantics remain
preserved.

Finding: section 5.

Goal: document the boundary / carrier / residual scoped-dispatch split
with a Writer-`listen` worked example and the invariant each trait
protects, so the rationale is not spread across `pub(crate)` trait docs.

Steps:

- Write the design note; enumerate the invariant each trait protects,
  chiefly keeping `NextProgram` independent from the selected
  `ActionProgram`.
- Evaluate consolidation only after W2, and gate it on a
  semantics-preservation proof that Writer `listen` / `censor` and `Span`
  still behave; the split encodes a real need, so unifying it prematurely
  risks regressions against working code.

### W9. Generic scoped rows

Status: Not started.

Finding: section 9.

Goal: add a separate generic scoped-row item macro (matches prior decision
D3), leaving `define_scoped_row!` concrete-only.

Steps:

- Design the macro syntax, including lifetime / type / where-clause
  handling and recursive `Self` replacement under generics, before
  implementing; these are the hard parts that distinguish it from the
  concrete `define_scoped_row!`.
- Implement; add tests covering lifetimes, type parameters, and where
  clauses.

Sequencing: when a concrete generic scoped-row need arrives.

### W10. Erased `Run` downcast soundness

Status: Complete for the current safe-API invariant. The
`RunRepresentation` downcast invariant is documented, the W6 known
limitations cross-link to it, and a regression test covers boundary
continuation typing across `map` and `bind`.

Finding: section 9.

Goal: keep the erased default (its `Box<dyn Any>` downcasts are a
deliberate ergonomics choice) and harden it with tests and docs, rather
than removing it (which would mean merging `Run` into `RunExplicit` and
losing the erased default).

Steps:

- Add boundary downcast soundness tests showing a type mismatch cannot
  arise through the safe API.
- Document the invariant near `RunRepresentation`.
- Cross-link from the W6 known-limitations list.

### W11. Port low-risk first-order effects and NonDet aggregation

Status: Complete. The residual-row-aware one-pass NonDet helper slice is
complete for `RcRun`, `ArcRun`, `RcRunExplicit`, and `ArcRunExplicit`.
`run_nondet` and `run_first_success` now interpret `Choose` and `Empty`
in one traversal without adding single-shot `Run` / `RunExplicit`
variants. The Fresh, Input, KVStore, and Output surface decisions have
now been adopted and folded into the concrete steps below. Fresh, Input,
KVStore, and Output core effect modules, brands, public exports, and
named-helper module wiring now exist through the W2 generator. Fresh
constructors, the generic `run_fresh_with(initial, next)` runner, and
the zero-based `usize` `run_fresh()` convenience runner now exist across
all six wrappers. Input constructors and `run_input_seq` now exist across
all six wrappers. KVStore constructors and `run_kv_store` now exist
across all six wrappers. Output constructors, `run_output_vec`, and
`run_output_monoid` now exist across all six wrappers, and the Output
helper slice has focused integration coverage for vector ordering and
monoid accumulation. Fresh, Input, and KVStore now have focused
integration coverage across all six wrappers. Generator tests now cover
descriptor registration, marker expansion, unsupported-method
diagnostics for the new effect families, and representative generated
item presence. Representative expansion checks have covered both the
direct-payload Output helper path and pointer-sibling Fresh / Input /
KVStore helper paths.

Finding: section 10, section 11 (P1).

Goal: add thin dedicated effects Fresh, Input, Output, and KVStore, each
with a named runner that follows the State / Writer interpretation
conventions already used by this crate while preserving the source
semantics from heftia where they are load-bearing. Fresh follows
heftia's `runFreshNatural` / `runFreshNaturalAsState` model: a generated
fresh value is read from a counter and the counter is advanced after each
operation. The Rust surface keeps the generated value type in the effect
brand and provides both a generic successor runner and an ergonomic
zero-based `usize` runner. Input follows heftia's `runInputList`: the
standard sequence runner targets `Option<Item>`, returns `Some(item)` for
available input, and returns `None` indefinitely after exhaustion. Output
follows heftia's split between list and monoid runners, but keeps this
crate's result-first tuple convention (`(result, accumulator)`) and
preserves output order. KVStore follows heftia's `runKVStoreCC` model of
interpreting the store as map-backed State, using
`std::collections::BTreeMap` with `K: Ord` for deterministic examples and
deferring a storage abstraction until there is concrete demand. For
nondeterminism, add only the genuinely-missing pieces:
residual-row-aware one-pass `Choose` + `Empty` helpers on the multi-shot
wrappers. The collection helper should be named `run_nondet` and return
`Vec<A>` in the existing true-then-false branch order. The first-success
helper should be named `run_first_success`, return `Option<A>`, try the
true branch first, and resume the false branch only when the true branch
aborts with `Empty`. The per-effect `run_empty` (into `Option`) and
`run_choose` (into `Vec`) already exist in `named_helpers/nondet.rs`.

Steps:

- Complete. Implement the NonDet helper slice first because it uses existing
  `Choose` and `Empty` effects and does not require adding a new effect
  spec. Add `run_nondet` and `run_first_success` to
  `named_helpers/nondet.rs` for `RcRun`, `ArcRun`, `RcRunExplicit`, and
  `ArcRunExplicit` only. Do not add these helpers to `Run` or
  `RunExplicit`, because `Choose` is intentionally multi-shot.
- Complete. Make both NonDet helpers residual-row-aware: remove both `Choose` and
  `Empty` from the first-order row in one traversal while preserving a
  residual `RMinusNonDet` row. Carry the wrapper-specific `Functor` /
  `SendFunctor`, projection `Clone`, and `Send + Sync` bounds at the
  method boundary, following the existing `run_choose` and `run_empty`
  helper style.
- Complete. Implement `run_nondet` as the one-pass collection interpreter. Pure
  results become singleton vectors, `Empty` becomes an empty vector, and
  `Choose` evaluates the true branch before the false branch and
  concatenates the results in that order.
- Complete. Implement `run_first_success` as the one-pass short-circuit
  interpreter. Pure results become `Some(value)`, `Empty` becomes
  `None`, and `Choose` evaluates the true branch first; evaluate the
  false branch only if the true branch returns `None`.
- Complete. Add focused tests for `run_nondet` and `run_first_success` across
  `RcRun`, `ArcRun`, `RcRunExplicit`, and `ArcRunExplicit`. Cover branch
  order, empty-branch pruning, residual-row preservation, and
  first-success short-circuiting with a program whose false branch would
  be observable if it ran.
- Complete. If the residual-row implementation hits a concrete Rust type-system
  blocker, document the exact limitation before falling back to a closed
  exact-row helper. Do not replace the adopted surface with a composition
  of `run_empty` and `run_choose`; that would fail the short-circuit
  contract. No fallback was needed; the Arc wrappers use the same
  projection-normalization pattern already established by `ArcRun`.
- Complete. Extend the W2 descriptor model before adding the new effect
  specs. The generator must support the existing continuation-effect
  sibling shape (`Box*Brand`, local refcounted `*Brand`, and
  `Send*Brand`) and the direct-payload shape used by Writer-style
  effects that do not need a pointer-brand parameter. This keeps Fresh,
  Input, and KVStore aligned with Reader / State closure-storage
  semantics while letting Output stay as small as Writer. The descriptor
  now records whether a spec uses pointer-brand siblings, and validation
  enforces either the complete three-sibling shape or an empty
  direct-payload sibling list.
- Complete. Add generated effect modules and brand declarations for the four
  new families, plus public module exports and `named_helpers` wiring.
  Use current new-style module layout. Keep all effect API modules under
  `#[fp_macros::document_module]` and fix generated documentation
  validation rather than suppressing it. Fresh, Input, and KVStore now
  generate local refcounted, sendable refcounted, and boxed single-shot
  continuation cells with their corresponding brands. Output now
  generates the direct-payload cell and `OutputBrand` without pointer
  siblings.
- Complete. Implement Fresh through the W2 generator as a first-order
  continuation effect with pointer-sibling brands (`BoxFreshBrand`,
  `FreshBrand`, and `SendFreshBrand`) parameterized by the generated ID
  type. Generated `fresh` constructors now exist across all six wrappers,
  and the generated `run_fresh_with(initial, next)` runner handles
  `Fresh<Id>` by returning the current ID and storing `next(current)`.
  The standard `run_fresh` runner is generated only for `usize`, using
  `0usize` and `|n| n + 1`, mirroring heftia's zero-based
  `Fresh Natural` runner without freezing the whole effect family to one
  counter type. Both runners return `(result, final_counter)` to match
  this crate's `run_state` convention.
- Complete. Implement Input through the W2 generator as a first-order continuation
  effect with pointer-sibling brands (`BoxInputBrand`, `InputBrand`, and
  `SendInputBrand`) parameterized by the value returned by `input`.
  Generated `input` constructors now exist across all six wrappers.
  Generated `run_input_seq` handles rows containing
  `Input<Option<Item>>`; the runner accepts an
  `impl IntoIterator<Item = Item>`, stores it as a `VecDeque<Item>`, pops
  from the front on each operation, and returns `None` after exhaustion.
  The non-Arc runners pop into a local before invoking the continuation
  so nested input effects do not keep a `RefCell` borrow live across
  resumption. Do not add a mandatory-input runner in the first slice;
  users who need a different exhaustion policy can reinterpret manually
  until a concrete error surface is justified.
- Complete. Implement KVStore through the W2 generator as a first-order
  continuation effect with pointer-sibling brands (`BoxKVStoreBrand`,
  `KVStoreBrand`, and `SendKVStoreBrand`) parameterized by key and value
  types. Generate primitive `lookup(key) -> Option<V>` and
  `update(key, Option<V>) -> ()` constructors across all six wrappers.
  Treat `None` in `update` as deletion and `Some(value)` as
  insert/replace. Generate `run_kv_store(initial_map)` using
  `BTreeMap<K, V>` and `K: Ord`; it returns `(result, final_map)` to
  match `run_state`. The generated constructors live on specialized
  wrapper impls (`Option<V>` for `lookup`, `()` for `update`) so callers
  do not need to specify an unconstrained result type. The runner stores
  the map behind `RefCell` or `Mutex` according to the wrapper substrate,
  clones lookup results, and drops each borrow or lock before invoking
  the continuation. Leave `insert`, `delete`, and `modify` as later thin
  helpers rather than primitive operations unless real usage shows they
  should be part of the generated core.
- Complete. Implement Output through the W2 generator as a first-order
  direct-payload effect with `OutputBrand<Out>`, an `output(out)`
  constructor across all six wrappers, and no pointer-brand siblings.
  Generate `run_output_vec` to return `(result, Vec<Out>)` with outputs
  in program order. Generate `run_output_monoid` to fold outputs through
  a user-supplied `Out -> Acc` mapping and `Acc: Monoid`, also returning
  `(result, Acc)`. Reuse the existing Writer fold-order strategy where
  needed so handler traversal order does not reverse user-visible output.
  The generated Output surface uses the direct-payload `Output` cell and
  `OutputBrand<Out>` for all wrapper substrates. `run_output_vec`
  delegates through the monoidal runner with `Vec<Out>` chunks, and
  `run_output_monoid` uses the same deferred fold-chain strategy as
  Writer to preserve observable output order.
- Complete. Add focused integration tests for each new effect across
  representative wrappers first (`Run`, `RcRun`, and `ArcRun`), then
  broaden to the explicit wrappers once the generator shape is stable.
  Cover Fresh counter progression and final counter, Input exhaustion
  after the sequence ends, KVStore lookup / insert / delete behavior and
  final map, Output vector order, and Output monoid accumulation. Output
  now has integration coverage across all six wrappers for vector order
  and monoid accumulation. Fresh, Input, and KVStore now have integration
  coverage across all six wrappers for their standard runner semantics.
- Complete. Add macro-generator tests for descriptor registration, marker parsing,
  unsupported-combination diagnostics, and representative generated item
  presence. Add representative `just cargo expand` checks for one
  pointer-sibling effect and Output's direct-payload effect before
  marking W11 complete. Descriptor registration now covers Output's
  direct-payload shape, marker parsing accepts Output methods, unsupported
  Output methods report the supported method set, and generated item
  presence is covered for `output`, `run_output_vec`, and
  `run_output_monoid`. `cargo expand` checks for
  `types::effects::named_helpers` verified the named Output runners, and
  the named Fresh / Input / KVStore runners from pointer-sibling effect
  specs. `cargo expand` checks for `types::effects::run` verified the
  `Run::output`, `Run::fresh`, `Run::input`, `Run::lookup`, and
  `Run::update` constructors.

Sequencing: after W2 so each effect is a single spec; if done earlier,
implement on the multi-shot wrappers first.

### W12. Port moderate effects: Coroutine, Log, Fail

Status: Complete. The W12 semantic choices and generator-shape decision
have been adopted into the concrete steps below. The generator now has a
minimal typed operation-shape layer for the current generated effects and
the reserved W12 shapes, descriptor validation checks the shape's
pointer-sibling rules, and existing effect item / wrapper builder routing
runs through the shape metadata. The first W12 effect-cell slice now
registers Coroutine, Log, and Fail as distinct generated effects,
including public brands, `types::effects` modules, `define_effect!`
expansion, and focused macro-generator coverage. Coroutine uses a
substrate-specific status family grounded in Heftia's `runCoroutine`
shape, Log is a distinct direct-payload effect that reuses Output's
runner pattern without aliasing row identity to Output or Writer, and
Fail is a distinct fixed-message `String` effect, not a generic `Except`
alias. The Coroutine status type matrix now exists for all six wrappers
with one-shot, multi-shot, and thread-safe resume continuation storage.
The multi-shot status cloneability decision has been adopted: generate
`Clone` only for the Rc/Arc status types and keep one-shot statuses
non-`Clone`; the generated impls and focused generator coverage are
complete. The Coroutine resume-result decision has been adopted: status
resume continuations return the next interpreted status in the residual
wrapper, not the final `A` directly; the status enum matrix, Clone impls,
and focused generator coverage now use that recursive status shape.
The Coroutine vertical slice is implemented for all six wrappers. The
multi-shot `RcRun`, `RcRunExplicit`, `ArcRun`, and `ArcRunExplicit`
helpers cover cloneable and thread-safe resume semantics; the one-shot
`Run` and `RunExplicit` helpers cover `FnOnce` resume semantics without
adding `Clone` bounds. Focused generator and marker-expansion coverage
proves the generated `yield_value` / recursive-status `run_coroutine`
methods are present, and integration coverage exercises `Done`,
`Continue`, input-fed resume, residual-row typing, multi-shot resume
reuse on Rc/Arc wrappers, and non-`Clone` one-shot result/capture
programs. Representative `cargo expand` checks cover the generated
Coroutine status matrix and `run_coroutine` runner shape. Remaining W12
work has advanced through the shared direct-payload wrapper builder:
Output and Log constructors/runners now come from the same token
templates while preserving separate `OutputBrand` and `LogBrand` row
identities. Focused integration tests cover Log vector order, monoid
accumulation, and positive row-identity separation from Output and
Writer. Representative `cargo expand` checks cover Output and Log
named-helper runners plus the default `Run` constructors. The Fail
constructor and runner layer is implemented across all six wrappers after
renaming the unit-Except helper to `throw_unit()`. Focused integration
coverage exercises pure success, failure short-circuiting through a bind,
`Result<A, String>` interpretation, and row identity separation from
`ExceptBrand<String>`. Representative `cargo expand` output for
`types::effects::named_helpers::fail` shows each generated `run_fail`
runner narrows the row with `map(Ok).handle_with::<FailBrand, ...>` and
maps `Fail::Fail(message, _)` to `Err(message)`. Full `just verify`
passes for the completed W12 implementation.

Finding: section 10.

Goal: port Coroutine, Log, and Fail through the generator as distinct
row capabilities. Coroutine follows the Heftia reference in
`Control.Monad.Hefty.Coroutine`: the runner removes one `Yield` row cell
and returns a status in the residual effect context. In this Rust
implementation, the status type must vary by wrapper substrate so the
continuation carries the same one-shot, cloneable, or thread-safe
semantics as the wrapper. Log follows Heftia's co-log-style identity as
a direct log-message capability; it should share Output's vector and
monoid runner machinery only as an implementation pattern. Fail follows
the separate failure capability exposed by `Control.Monad.Hefty.Fail`
and keeps its own `FailBrand` even though its standard runner returns
`Result<A, String>`.

Steps:

- Complete. Adopt the W12 decisions into this work item and remove them
  from the Open Questions, Decisions, Issues and Blockers section. The
  chosen architecture is the long-term shape: distinct row identities for
  Coroutine, Log, and Fail, with helper code shared through generator
  descriptors rather than public type aliases that collapse semantics.
- Complete. Adopt the minimal typed operation-shape layer as the W12
  generator approach. Do not add standalone one-off builders for
  Coroutine, Log, and Fail when a shared shape captures the semantics,
  and do not replace the current generator with a broad generic
  operation-description DSL before the extra abstraction is proven. This
  is the smallest coherent architecture: it removes known W12 duplication
  without reopening completed W2 / W11 work.
- Complete for the current generated effects and reserved W12 shapes. Add
  an `EffectOperationShape`-style descriptor field, or an equivalent
  typed metadata layer, to the generator descriptors before adding the
  new W12 effects. Cover the existing and W12 shapes explicitly:
  `ReaderEnvironment`, `StateCell`, request-value continuation for
  Fresh / Input, direct payload for Output / Log, fixed-message abort for
  Fail, coroutine yield/status for Coroutine, and a named dedicated
  multi-operation shape for KVStore. Keeping KVStore named in the shape
  layer preserves the current generator truth: it is not the same as the
  one-operation W12 effects, and pretending otherwise would just move the
  special case out of sight.
- Complete for pointer-sibling shape validation. Route descriptor
  validation through the operation-shape metadata. Check that
  pointer-brand siblings, sendability, wrapper capability rules, and
  supported method sets agree with the selected shape before token
  emission. The validation should reject impossible combinations early,
  such as a direct-payload effect with pointer-brand siblings, a
  fixed-message abort effect parameterized like `Except<E>`, or a
  coroutine status runner without wrapper-specific continuation
  semantics.
- Complete. Route builder selection through the operation-shape metadata.
  Complete for all currently generated effect cells, including W12 cell
  generation, and complete for the direct-payload wrapper constructor /
  runner layer shared by Output and Log. The direct-payload builder now
  emits both Output and Log cells where the code is genuinely
  shape-identical; the aborting-effect builder emits Fail without
  aliasing `FailBrand` to `ExceptBrand<String>`; and Coroutine has a
  named yield/resume cell builder because Heftia's `runCoroutine` shape
  exposes `Done(result)` and `Continue(output, resume)` as public status
  values rather than merely asking a handler for one value. The
  fixed-message abort wrapper builder now routes
  `EffectOperationShape::FixedMessageAbort` / `EffectName::Fail` and
  emits the Fail constructor and runner methods across all six wrappers.
- Complete. Refactor the existing Output wrapper-method builder into a
  generic direct-payload wrapper builder parameterized by constructor
  name, vector runner name, monoid runner name, cell type, brand type,
  operation variant, row functor, wrapper type, and wrapper module. Route
  both Output and Log through this builder. This deliberately accepts a
  larger internal generator refactor, and careful Output regression
  coverage, because it removes the hardcoded direct-payload wrapper
  cross-product before Log expands it. Do not copy the Output builder
  into a separate Log builder: that is the fastest short-term path, but
  it makes every future direct-payload semantic fix a two-site edit and
  accrues technical debt at the generator boundary. Do not hand-write Log
  methods in the six wrapper modules: that bypasses descriptor
  validation and risks public API drift between generated effects. The
  recommended refactor best matches the guiding principles because the
  architecture becomes right at the source of duplication rather than
  patching only the Log symptom. Implementation notes: the old
  `output_wrapper_impl_items` module was replaced by
  `direct_payload_wrapper_impl_items`; the shared builder uses
  compile-time token templates rather than a runtime metadata struct
  because `quote!` method identifiers must remain Rust tokens, while the
  per-effect/per-wrapper strings still drive generated examples and row
  brand paths. `run_wrapper_impl_items_from_descriptor` dispatches
  `EffectOperationShape::DirectPayload` for both Output and Log through
  the shared builder. Focused generator tests prove Output still emits
  `run_output_vec` and Log emits `log` / `run_log_monoid`; integration
  regression tests prove Output helper behavior still preserves order;
  and representative `just cargo expand` checks cover both Output and
  Log named-helper runners.
- Partial. Add focused macro-generator tests for the shape layer before
  adding the W12 effect ports. Current coverage records descriptor
  registration for the existing and W12 shapes, validates pointer-sibling
  consistency, validates direct-payload sibling rejection, validates
  missing pointer-brand siblings, proves all current effect item routing
  goes through operation-shape builders, verifies W12 generated item
  presence, verifies Output / Log direct-payload sibling behavior,
  verifies Fail's distinct abort shape, verifies Coroutine's yield/resume
  sibling shape, verifies that Coroutine generates `Clone` impls for the
  four multi-shot statuses and not the two one-shot statuses, verifies
  recursive status-continuation resume result presence, verifies
  multi-shot and one-shot Coroutine wrapper-method generated item
  presence, verifies one-shot Coroutine marker expansion for valid
  constructors/runners, verifies direct-payload wrapper-method generated
  item presence for Output and Log, and derives unsupported method
  diagnostics from descriptor method sets for every current effect
  family. Representative `just cargo expand` checks now cover the
  generated Coroutine status matrix and runner shape, Output
  direct-payload named-helper runners, Log direct-payload named-helper
  runners, and the default `Run` Output / Log constructors. Remaining
  macro coverage should land with Fail's fixed-message runner.
- Complete. Add generator descriptors for a Coroutine
  `yield_value(output) -> In` primitive. Use `yield_value` rather than
  raw `yield` so examples avoid Rust keyword escaping. Model the
  operation as a first-order continuation effect with pointer-sibling brands
  (`BoxCoroutineBrand<P, Out, In>`, `CoroutineBrand<P, Out, In>`, and
  `SendCoroutineBrand<P, Out, In>`) and wrapper-specific continuation
  storage. The effect-cell descriptor, public brands, generated
  `Coroutine` / `SendCoroutine` / `BoxCoroutine` cells, and `Functor` /
  `SendFunctor` impls are complete, and the status enum naming matrix
  exists with recursive status-continuation resume results. The
  multi-shot `yield_value` and `run_coroutine` methods are generated for
  `RcRun`, `RcRunExplicit`, `ArcRun`, and `ArcRunExplicit`; the one-shot
  `yield_value` and `run_coroutine` methods are generated for `Run` and
  `RunExplicit`. Do not use a single erased `dyn Fn` status callback for
  all wrappers; it would hide the semantic difference between one-shot,
  cloneable, and thread-safe substrates and would accrue technical debt
  at every runner boundary.
- Complete. Adopt the recursive status-continuation architecture for
  Coroutine runner statuses. A prior W12 implementation spike showed
  that the coherent generated runner path is
  `self.map(Done).handle_with(...)`: after mapping pure results to
  `Done`, the Coroutine operation's resume continuation returns the next
  interpreted status in the residual row, not the final `A` directly.
  Keep that generated narrowing path and do not implement a bespoke
  Coroutine-only traversal that duplicates row projection and
  continuation rebuilding across the wrapper matrix. Also do not expose a
  full-row resume continuation that asks callers to run
  `run_coroutine` again manually; that leaks the handled row and weakens
  the residual-row guarantee.
- Complete. Update Coroutine status types before exposing the runner
  methods. The shared descriptor-driven naming matrix exposes the same
  conceptual variants for all six wrappers while preserving substrate
  semantics: `RunCoroutineStatus`, `RcRunCoroutineStatus`,
  `ArcRunCoroutineStatus`, `RunExplicitCoroutineStatus`,
  `RcRunExplicitCoroutineStatus`, and `ArcRunExplicitCoroutineStatus`.
  Each status has `Done(A)` and `Continue(Out, resume)` variants. The
  default and explicit `Run` statuses carry a one-shot resume
  continuation; `RcRun` and `RcRunExplicit` carry a cloneable resume
  continuation; `ArcRun` and `ArcRunExplicit` carry a `Send + Sync`
  cloneable resume continuation. Each resume continuation returns the
  same residual wrapper whose result is the corresponding status type,
  for example
  `RcRun<RMinusCoroutine, CNilBrand, RcRunCoroutineStatus<...>>` rather
  than `RcRun<RMinusCoroutine, CNilBrand, A>`. This is an API-breaking
  adjustment, but it preserves the long-term architecture: the public
  status represents the next state of the interpreted coroutine and the
  Coroutine row stays removed from every resume result.
- Complete. Update the generated `Clone` impls for the multi-shot
  Coroutine statuses after changing the recursive status shape and before
  implementing the multi-shot runner methods. `Clone` is implemented for
  `RcRunCoroutineStatus`, `RcRunExplicitCoroutineStatus`,
  `ArcRunCoroutineStatus`, and `ArcRunExplicitCoroutineStatus`; it
  requires `A: Clone` and `Out: Clone`, and carries the existing
  `Send + Sync` bounds on the Arc variants. It does not require
  `In: Clone`, because `In` is accepted by the resume function and is not
  stored as a cloneable payload. It does not implement `Clone` for
  `RunCoroutineStatus` or `RunExplicitCoroutineStatus`, because their
  resume continuations are `FnOnce` and the public status should preserve
  one-shot semantics. The generated impls clone the recursive resume
  continuations only on the multi-shot status types and keep the same
  one-shot / multi-shot distinction. Macro-generator coverage verifies
  that the four multi-shot status types have `Clone` impls, the one-shot
  status types do not, and every status resume field returns the
  recursive status shape.
- Complete. Implement Coroutine as a phased vertical slice: first
  generate `yield_value` and recursive-status `run_coroutine` for
  `RcRun`, `RcRunExplicit`, `ArcRun`, and `ArcRunExplicit`, because those
  wrappers exercise the hardest multi-shot continuation semantics. The
  runner uses `map(Done).handle_with(...)` so the implementation stays on
  the same generated row-narrowing architecture as the other first-order
  runners. Integration tests cover zero-yield `Done`, one-yield
  `Continue`, resuming with input to reach `Done`, resuming with input to
  reach another `Continue`, and calling the multi-shot resume
  continuation more than once on Rc/Arc wrappers.
- Complete. Implement the one-shot Coroutine slice for `Run` and
  `RunExplicit`. It reuses the same effect descriptor and status naming
  matrix and recursive status-continuation result, but keeps the
  continuation one-shot through `BoxCoroutineBrand<BoxBrand, Out, In>`
  and `FnOnce` status resumes rather than adding cloneable indirection.
  Integration tests prove that the runner removes the Coroutine row,
  preserves remaining rows, and does not impose `Clone` on the resumed
  program result or captured continuation.
- Complete. Add generator descriptors for Log as a direct-payload first-order
  effect with `LogBrand<Message>` and `Log<'a, Message, A>`. Generate a
  `log(message) -> ()` constructor across all six wrappers, plus
  `run_log_vec` and `run_log_monoid` runners following the W11 Output
  runner convention and preserving program order. Share Output runner
  builder code where it removes duplication, but keep the public cell,
  brand, helper names, examples, and row membership distinct from
  `OutputBrand<Message>` and `WriterBrand<Message>`. The effect-cell
  descriptor, public brand, generated `Log` cell, and `Functor` /
  `SendFunctor` impls are complete. The shared direct-payload wrapper
  builder generates `log`, `run_log_vec`, and `run_log_monoid` across
  all six wrappers; the constructor macros are wired into each wrapper's
  smart constructors; `named_helpers::log` mirrors the Output helper
  registration pattern; and focused integration tests cover Log vector
  order, monoid accumulation, and positive row identity separation from
  Output and Writer.
- Complete. Adopt the Fail constructor name-collision decision by reserving
  `fail(message)` for the generated `FailBrand` constructor and renaming
  the existing unit-Except helper from `fail()` to `throw_unit()` across
  all six wrappers. Rust inherent methods cannot be overloaded by arity,
  so keeping both `fail()` and `fail(message)` is not available. Do not
  rename the new fixed-message constructor to `fail_message`,
  `fail_with`, or `abort`: that compatibility-preserving route would
  attach the natural `fail` spelling to `ExceptBrand<()>`, even though
  W12 adds `FailBrand` as the distinct source-level failure capability.
  Do not remove the unit-Except helper outright: `throw_unit()` keeps the
  existing convenience while making its semantics explicit. Concrete
  implementation steps: rename the `named_helpers::except` `fail<Idx>()`
  methods to `throw_unit<Idx>()` for `Run`, `RcRun`, `ArcRun`,
  `RunExplicit`, `RcRunExplicit`, and `ArcRunExplicit`; update
  `run_except_helpers`, doctests, and UI stderr snapshots that mention
  `fail()`; verify no `Run::fail()` unit-Except references remain; then
  implement generated `fail(message)` for `FailBrand`.
- Complete. Add generator descriptors for Fail as a fixed-message aborting effect
  with `FailBrand` and `Fail<'a, A>` carrying a `String` message and no
  continuation. Generate `fail(message) -> A`, accept any `message` value
  implementing `Into<String>`, and generate
  `run_fail() -> Result<A, String>` across all six wrappers. Model the
  generated runner after `Except`'s
  short-circuiting shape, but do not parameterize `FailBrand` by an
  arbitrary error type and do not implement it as a type alias for
  `ExceptBrand<String>`; the source-level capability should remain
  visible in row types. The effect-cell descriptor, public brand,
  generated `Fail` cell, and `Functor` / `SendFunctor` impls are
  complete. The fixed-message abort wrapper builder now routes
  `EffectOperationShape::FixedMessageAbort` / `EffectName::Fail`,
  generates `fail(message)` constructors for all six wrappers, and
  generates `run_fail` through `named_helpers::fail` for all six wrappers.
  Focused tests cover pure success, failure short-circuiting through a
  bind, conversion to `Result<A, String>`, and row identity distinct from
  `ExceptBrand<String>`.
- Complete. Add macro-generator coverage for the three new descriptor
  families: descriptor registration, marker parsing, unsupported method
  diagnostics, representative generated item presence, and at least one
  `just cargo expand` comparison for Coroutine status / runner shape,
  Log's direct-payload runners, and Fail's fixed-message runner.
  Coroutine descriptor registration, marker parsing, unsupported method
  diagnostics, generated item presence, and focused status-shape coverage
  are complete. Representative `cargo expand` comparison covers
  `types::effects::coroutine`, where the one-shot statuses resume through
  `FnOnce` into the residual wrapper/status, and
  `types::effects::named_helpers::coroutine`, where `run_coroutine`
  narrows the row through `map(Done).handle_with(...)` and returns
  `Continue(out, resume)` without reintroducing the handled Coroutine
  row. Representative `cargo expand` checks also cover Output and Log
  named-helper runners plus default `Run` constructors, and
  `types::effects::named_helpers::fail`, where `run_fail` narrows the row
  with `map(Ok).handle_with::<FailBrand, ...>` and returns `Err(message)`
  for `Fail::Fail`.
- Complete. Add integration coverage for all six wrappers. Coroutine
  coverage now includes status shape, residual-row typing, input-fed
  resume, multi-shot resume on Rc / Arc wrappers, one-shot resume on
  default / explicit `Run`, and non-`Clone` one-shot result/capture
  programs. Log coverage now includes vector order and monoid
  accumulation across all six wrappers, plus positive row identity
  separation from Output and Writer. Fail coverage now includes pure
  success, failure short-circuiting through a bind across all six
  wrappers, conversion to `Result<A, String>`, and row identity distinct
  from `ExceptBrand<String>`.
- Complete. Run `just fmt`, `just check`, `just clippy`, `just deny`,
  `just doc`, and `just test` before marking W12 complete. Full
  `just verify` passes for the completed W12 implementation. No
  benchmark-sensitive Coroutine runner change was made in the final Fail
  constructor / runner layer.

Sequencing: after W11.

### W13. Runtime policy, then async interpreter, then deferred ports

Status: Blocked on the unresolved W13 Runtime Policy Gate in the Open
Questions, Decisions, Issues and Blockers section. Do not implement the
async interpreter, Unlift, Concurrent, Shift / CC, Provider with
runtime-owned resources, timers, subprocesses, streams, or new IO
embedding until those decisions are adopted and folded into this work
item.

Finding: sections 6 and 10, section 11 (P3).

Goal: write a runtime policy first, then build the async interpreter the
policy allows, then schedule the runtime-sensitive ports (Shift / CC,
Provider, Unlift, the Concurrent family) against it.

Steps:

- Adopt or revise the recommendations in the W13 Runtime Policy Gate,
  then fold the chosen decisions into this work item and remove the
  resolved open questions. The policy must cover the async interpreter
  substrate, executor ownership, blocking model, cancellation, IO
  embedding, process lifecycle ownership, target-monad lifting,
  continuation exposure, and `Send + Sync`; these ports encode runtime
  commitments that must be decided once and centrally rather than
  per-effect.
- After the runtime policy is adopted, implement the async approach. The
  current recommendation is a runtime-agnostic `Future`-capable
  target-monad path, with boxed futures allowed at the async boundary,
  and a dedicated async substrate only as a documented fallback if the
  spike proves a concrete Rust type-system, lifetime, safety, or
  proc-macro limitation.
- Schedule Shift / CC, Provider, Unlift, and the Concurrent family
  against the policy. Shift / CC additionally needs
  answer-type-polymorphic continuation capture, which is beyond the
  current mono-in-`A` model.

Sequencing: last; everything here is policy-gated.

## Suggested implementation order

This order follows the generator-first thesis in the
[root-cause framing](#root-cause-framing) and reflects the current state
after W12. Completed items remain documented in their work sections; the
next actionable implementation is to finish W2/W3 before revisiting
runtime-sensitive W13 work.

Current adopted order after W12:

1. Complete W2/W3 generator and capability work. Encode the verified W3
   matrix in the W2 descriptor model, add descriptor validation for
   unsupported wrapper / effect / helper combinations, and cover the
   matrix with targeted generator tests before broadening migrations.
2. Continue W2 migrations through descriptors only. Inventory the
   remaining hand-written first-order surfaces, then migrate `Except`,
   `Empty` / `Choose` plus `NonDet` named helpers, and the first-order
   `Writer` helper surface with the same `just cargo expand ...`
   baseline and comparison discipline used for Reader and State.
3. Close W2/W3 together. Mark them complete only when the remaining
   first-order generated surfaces are descriptor-backed,
   expansion-equivalent or intentionally documented, and constrained by
   the W3 capability matrix. If a concrete Rust or proc-macro limitation
   blocks this, document the blocker and alternatives before adding any
   temporary template-per-item copy.
4. Evaluate W8 consolidation after W2. Preserve Writer `listen` /
   `censor`, Span, Catch, Local, Bracket, and RefBracket semantics unless
   the consolidation proof shows an equivalent boundary / carrier /
   residual model.
5. Fold W5 row macros and W9 generic scoped rows into the macro redesign
   only if W2 or W8 exposes a concrete need. Otherwise leave them
   deferred rather than adding standalone macro surface.
6. Resolve the W13 Runtime Policy Gate before starting runtime-sensitive
   implementation. After the policy is adopted, implement the async
   interpreter approach and schedule Shift / CC, Provider, Unlift, and
   the Concurrent family against that policy.

## Traceability

This plan is derived from [`findings.md`](findings.md). Future
implementation commits should cite the relevant work item (W-number) and
any adopted decision materially exercised by the change.
