# Effects System Review (review-1)

A review of the current effects subsystem: architecture, structure,
naming, gaps, inconsistencies, and a survey of additional effects worth
porting from `heftia-effects` (Control.Monad.Hefty) and `purescript-run`.

## 1. Scope and method

Files and submodules reviewed:

- `fp-library/src/brands/effects.rs` (the effect / Run brand catalog).
- `fp-library/src/types/effects.rs` and all submodules (core machinery,
  effect definitions, the six Run wrappers, handlers, interpreter,
  standard scoped handlers, named helpers).
- `fp-macros/src/effects.rs` and all submodules (`effects_macro`,
  `handlers`, `row_sort`, `scoped_row`, `row_aliases`, `im_do`).

Method:

- Generated an item inventory with `just item-inventory` to map where
  complexity concentrates before reading. Counts in this document are
  approximate and indicative, not authoritative: the run captured the
  `fp-library` effects modules (on the order of 1,500 items across the
  inventoried files, itself a subset of the roughly 75 Rust files in
  scope), did not include the `fp-macros` effects crate, and the tool's
  totals vary with how paths are grouped. The qualitative conclusions do
  not depend on exact counts.
- Read the core machinery in full (`node`, `coproduct`, `member`,
  `scoped`, `handlers`, `interpreter`, `interpreter/first_order`) plus a
  representative first-order effect (`reader`) and the brand catalog.
- Confirmed the duplication hypothesis by comparing the public surface
  of all six wrappers and checking for any wrapper-generating macro.
- Surveyed `heftia-effects/src/Control/Monad/Hefty/*` and
  `purescript-run/src/Run/*` for portable effects and handlers, and
  compared the combinator surface (`expand`, `interpose`, `handle_rec`,
  etc.).

## 2. What the system is (one page)

The subsystem is a single idea applied at two levels: a Free monad over
a dual-row dispatch node.

```text
Run<R, S, A> = FreeFamily<NodeBrand<R, S>, A>
```

- `Node<'a, R, S, A>` (in `node.rs`) is a two-variant enum: `First`
  carries a first-order effect drawn from row `R`; `Scoped` carries a
  higher-order effect drawn from row `S`. It is the value stored in the
  Free wrapper's `Wrap` arm.
- The first-order row `R` is a right-nested `CoproductBrand` chain of
  `CoyonedaBrand`-wrapped effect functors terminated by `CNilBrand`.
  Coyoneda gives any effect type a free `Functor`, so first-order
  effects are dispatched by `Functor::map`. This is the Rust encoding of
  PureScript's `VariantF`.
- The scoped row `S` is the same `CoproductBrand` structure but holds
  raw scoped-effect brands (no Coyoneda), dispatched by hand-written
  case analysis. Programs with no scoped effects set `S = ScopedNil`
  (alias for `CNilBrand`).
- Row encoding reuses `frunk_core::coproduct` (`coproduct.rs`), and
  `Member<E, Idx>` (`member.rs`) is a thin facade over frunk's
  `CoprodInjector` / `CoprodUninjector` for inject / project. This
  mirrors the `Member` row-membership bound used by PureScript Run.
- The `effects!` / `handlers!` macros build the row and the parallel
  handler cons-list, both lexically sorted by a shared structural key so
  they align cell-for-cell.
- Interpretation (`interpreter.rs`, `interpreter/first_order.rs`) walks
  the row's value-level `Coproduct` against the handler cons-list in
  lock-step. `DispatchHandlers` handles the first-order row; a family of
  scoped-dispatch traits handles the scoped row.

This is faithful to heftia's "Hefty" split between algebraic
(first-order) and higher-order (scoped) effects, layered onto
fp-library's Brand / Kind HKT encoding.

### The six substrate wrappers

The same dual-row program is offered over six wrappers, the cross
product of two axes:

- Substrate family: Erased (`Run`, over `Free`) vs Explicit
  (`RunExplicit`, over `FreeExplicit`).
- Sharing / shot model: single-shot Box (`Run` / `RunExplicit`),
  multi-shot Rc (`RcRun` / `RcRunExplicit`), thread-safe Arc (`ArcRun` /
  `ArcRunExplicit`).

These six wrappers and their submodules (`representation`, `boundary`,
`smart_constructors`, `raw_scoped`) are the dominant share of the
inventoried `fp-library` effects items, on the order of 40 to 45 percent
by item count (approximate; see section 1).

## 3. Strengths

- Faithful, ambitious port. It reproduces a sophisticated effect model
  (heftia + purescript-run) in stable Rust without HKTs, and goes beyond
  purescript-run by adding higher-order scoped effects (Catch, Local,
  Bracket, Span, plus Ref variants).
- Good reuse rather than reinvention. The row machinery is built on
  `frunk_core::coproduct`; `Member` is a small facade, not a
  reimplementation. The orphan rules are satisfied cleanly via the local
  `CoproductBrand` (documented in `coproduct.rs`).
- Stack safety is available. `handle_rec` / `run_rec` (6 each) drive a
  `MonadRec::tail_rec_m` loop, so deep programs do not overflow the
  stack. This matches purescript-run's `interpretRec` / `runRec`.
- Reinterpretation is present. `interpose` (15 occurrences) lets a
  handler rewrite an effect in terms of others, which is a capability
  purescript-run's base API does not surface directly.
- Strong documentation and tests. 209 `#[test]` functions in the
  subsystem plus pervasive doctests; every brand and trait carries
  detailed rationale.
- Ergonomic helpers. `named_helpers` adds `eval_state` / `exec_state` /
  `gets` / `modify`, `note` / `rethrow` / `from_option`, `asks`,
  `fold_writer`, etc., which are the small conveniences users expect.
- Clean macro-built user surface. `effects!` / `handlers!` /
  `scoped_effects!` / `scoped_handlers!` sort by a shared key so rows and
  handler lists stay aligned, and duplicate entries are rejected at
  expansion.

## 4. Principal concern: the substrate cross-product explosion

This is the dominant cost of the current design and the thing most worth
addressing.

Evidence:

- The brand catalog `brands/effects.rs` defines about 45 brands. Most
  effects fan out into three or more siblings keyed by closure storage,
  for example Reader -> `ReaderBrand` / `SendReaderBrand` /
  `BoxReaderBrand`, and Bracket fans out into six
  (`BracketBrand` / `BoxBracketBrand` / `SendBracketBrand`, each with an
  `*ExplicitBrand` sibling) plus four more for RefBracket.
- Each first-order effect file defines three parallel types. `reader.rs`
  holds `Reader` (Rc, multi-shot `Fn`), `SendReader` (Arc, `Fn + Send +
Sync`), and `BoxReader` (Box, single-shot `FnOnce`), each with its own
  `Clone` / `Functor` / `SendFunctor` impls.
- Even the first-order dispatcher duplicates: `interpreter/first_order.rs`
  has three near-identical `DispatchHandlers` cons-cell impls, one each
  for `Coyoneda`, `RcCoyoneda`, and `ArcCoyoneda`.
- The six wrappers each re-declare the same smart-constructor surface
  (`ask` / `get` / `put` / `throw` / `tell` / `catch` / `local` /
  `bracket` / `span` / `listen` / `censor`, plus `choose` / `ref_bracket`
  on the multi-shot ones).

Root cause: this is a Rust limitation, not a flaw in the model.
PureScript and Haskell derive all six wrappers from one definition
because they have HKTs and GC. In Rust, closures are not generic, and
trait objects must bake in `Send` / `Sync` and `FnOnce` vs `Fn`, so each
combination becomes a distinct type.

The real problem is that none of this is generated. There is no
`macro_rules!` or proc-macro that emits the six wrappers or the
per-effect brand siblings; only `im_do!` (do-notation) is generated. The
wrappers are hand-maintained parallel code (run.rs alone is 128 KB), so:

- Every change must be mirrored six times by hand.
- Drift is already visible: `choose` and `ref_bracket` exist only on the
  multi-shot wrappers (which is correct, since a `Choose` handler invokes
  the continuation more than once), but this asymmetry lives only in
  prose, not in any single capability matrix or generated surface.

Recommendation: introduce code generation. A proc-macro (for example
`define_effect!` for the brand siblings + effect types + smart
constructors, and a `define_run_wrapper!` for the six wrappers) would
collapse the parallel files to a single spec per effect and per wrapper,
eliminate drift, and make adding a new effect cheap. The module doc in
`fp-macros/src/effects.rs` already reserves `define_effect!` and
`define_scoped_effect!` as future work; this should be prioritized. Note
that a narrow precedent already exists: `#[fp_macros::document_module]`
supports a `documented_helper_impls!` item marker (see
`fp-macros/src/documentation/item_generators.rs`) that expands documented
helper impl blocks before validation, which is a partial mitigation for
helper-method drift. A wrapper / effect generator would extend that idea
to the brand and smart-constructor surface.

## 5. Scoped-effect dispatch is conceptually heavy

The scoped path in `interpreter.rs` is the most demanding area of the
codebase. It defines a family of dispatch traits:

- `DispatchScopedHandler` / `DispatchScopedHandlers` (ordinary scoped
  dispatch, public).
- `DispatchScopedBoundaryHandlers` (public facade for typed boundaries).
- `DispatchScopedCarrierHandler` / `DispatchScopedCarrierHandlers`
  (carrier-aware around-action route, `pub(crate)`).
- `DispatchScopedBoundaryHeadHandlers` (indexed boundary-head
  projection, `pub(crate)`).
- `DispatchResidualScopedHandlers` (residual ordinary dispatch after a
  boundary head is consumed, `doc(hidden)` but public).

This machinery exists to support around-action effects, principally
Writer `listen` / `censor` and `Span`, which must observe a selected
action's result before the wrapper resumes the outer continuation. The
boundary / carrier / residual split is the mechanism that keeps
`NextProgram` independent from the selected `ActionProgram` while routing
the consumed member to a carrier-aware handler and every other member to
ordinary dispatch.

It is correct and powerful, but it is a lot of surface for a reader to
hold in their head, and several traits are `pub(crate)` with
`skip_call_check` doctests because the real call path cannot be
constructed externally. Recommendations:

- Write a dedicated design note (in this plans area) that explains the
  boundary / carrier / residual split with one worked example
  (Writer `listen`), so the rationale is not spread across trait docs.
- Evaluate whether the three boundary-related traits can be unified or
  reduced once the design stabilizes. This is the area most likely to
  benefit from consolidation.

## 6. The interpreter is mono-in-`A` and synchronous

Two deliberate, documented design points worth restating as known
limitations:

- Mono-in-`A` dispatch. The interpreter is a step function whose handler
  is monomorphic in the program's result type `A`, not a true rank-2
  natural transformation. This is the same shortcut PureScript Run's
  `run` takes internally, and it is the only shape Rust's non-generic
  closures permit. The escape hatch for genuine rank-2 work is
  `NaturalTransformation` + `Free::fold_free`. This is fine, but it
  should be surfaced in a top-level "limitations" list rather than only
  in the `interpreter.rs` module doc.
- No async interpreter. Handler closures return the next program
  synchronously, not a `Future`. There is no async interpreter because
  `MonadRec` has no `Future` impl, and the documented workaround is
  `tokio::task::spawn_blocking`. This is the single biggest functional
  gap for real IO workloads, and it is also what blocks porting heftia's
  `Unlift` and `Concurrent` effects. A `MonadRec`-over-`Future` impl (or
  a dedicated async substrate) is the unlock.

## 7. Structure and organization

- Brand-level class participation is partial and uneven, and is best
  treated as a capability matrix rather than a simple "branded vs not"
  split. All six wrappers are newtype structs (for example
  `Run<R, S, A>(RunRepresentation<R, S, A>)`,
  `RcRun<R, S, A>(RcFree<NodeBrand<R, S>, A>)`,
  `RunExplicit<'a, R, S, A>(FreeExplicit<'a, NodeBrand<R, S>, A>)`). Only
  the Explicit trio has dedicated brands (`RunExplicitBrand` /
  `RcRunExplicitBrand` / `ArcRunExplicitBrand` in `brands/effects.rs`),
  and their class coverage is not uniform: `RunExplicitBrand` implements
  `Functor`, `Pointed`, `Semimonad`, `RefFunctor`, `RefPointed`, and
  `RefSemimonad`; `RcRunExplicitBrand` implements `Pointed`, `RefFunctor`,
  `RefPointed`, and `RefSemimonad` (no owned `Functor` or `Semimonad`);
  `ArcRunExplicitBrand` implements only `SendPointed` and `SendRefPointed`.
  The Erased trio (`Run` / `RcRun` / `ArcRun`) has no brand at all: there
  is no `RunBrand` / `RcRunBrand` / `ArcRunBrand` anywhere, and the
  non-explicit Free family is itself unbranded (only `FreeExplicitBrand` /
  `RcFreeExplicitBrand` / `ArcFreeExplicitBrand` exist; there is no
  `FreeBrand` / `RcFreeBrand` / `ArcFreeBrand`). So the Erased wrappers
  expose only inherent `bind` / `map` methods and cannot be used
  generically at the brand level, and giving them brands would first
  require branding the non-explicit Free family (which the erased
  `Box<dyn Any>` representation may not admit cleanly). The right next
  step is a capability audit (record the actual per-brand class matrix)
  and a deliberate decision about which gaps to close versus document as
  intentional, not an assumed delegation.
- The subsystem is not feature-gated. `types.rs` exposes it as a plain
  `pub mod effects;`, so the entire heavy subsystem (and its six
  wrappers) always compiles for every downstream user. Recommend an
  `effects` cargo feature, and possibly sub-features (for example
  `effects-arc`, `effects-explicit`) so users who only need single-shot
  `Run` do not pay for the Arc / Explicit families in compile time.
- `named_helpers` is a good split. Keeping the small ergonomic methods
  out of the already-large wrapper files is the right call. Likewise
  `standard_scoped_handlers` with per-effect `carrier` / `raw_replacers`
  submodules is cleanly separated.
- File sizes are dominated by doctests, not logic. run.rs at 128 KB is
  mostly examples; that is acceptable. The cost is the six-fold
  parallelism, not the per-file length.

## 8. Naming

- Generally strong and consistent. `Member`, the `effects!` /
  `handlers!` family, the `Brand` suffix, and the `Box` / `Send` / `Rc` /
  `Arc` prefixes are all coherent. `ScopedCoproduct` / `ScopedNil` as
  transparent aliases for `CoproductBrand` / `CNilBrand` are a nice
  self-documenting cue at user-facing signatures.
- The prefix semantics are not obvious without prose. `SendX` means
  Arc-backed with `Send + Sync` baked into the trait object; `BoxX`
  means single-shot `FnOnce`; the bare brand is Rc-backed multi-shot.
  These are documented per brand but there is no single legend. A short
  table mapping prefix -> closure storage -> which wrappers use it would
  save readers from reconstructing it.
- "Explicit" vs "Erased" substrate naming is opaque. The axis (Explicit
  `FreeExplicit` vs Erased `Free`) is meaningful but never explained in
  one central place. Document the axis once.

## 9. Inconsistencies and limitations (consolidated)

- Row subsumption (`expand` / `weaken`) is missing. There are zero
  occurrences of `expand` / `weaken` / `subsume` on the wrappers, even
  though `coproduct.rs` surfaces `CoproductEmbedder` / `CoproductSubsetter`.
  PureScript Run's `expand` (lift a program over a smaller row into a
  larger row) is a core composability combinator. Without it, programs
  written against different-but-compatible rows cannot be combined
  without manual plumbing. This is the most important missing combinator.
- The `choose` / `ref_bracket` capability asymmetry is not surfaced.
  These exist only on multi-shot wrappers for sound reasons, but there is
  no capability matrix; a user on `Run` discovers the gap via a trait
  error.
- The row macros have an Rc / Arc surface asymmetry. `RowHeadWrap` in
  `effects_macro.rs` has `RcCoyoneda` / `ArcCoyoneda` variants, but only
  `define_effect_row_aliases!` reaches them; there is no `rc_effects!` /
  `arc_effects!` paralleling `effects!`. Users targeting Rc / Arc
  wrappers must use `define_effect_row_aliases!` or hand-write the row.
  Either add the macros or document the alias macro as the canonical
  Rc / Arc path.
- The structural row-sort key is syntactic, not semantic. `row_sort.rs`
  normalizes whitespace and grouping but does not resolve aliases,
  imports, or fully-qualified paths. Spelling a brand `ReaderBrand` in
  `effects!` and `crate::brands::ReaderBrand` in `handlers!` produces two
  distinct keys, so the row and the handler list are built in different
  orders. The macros cannot catch this at expansion (they cannot resolve
  names), so it is not rejected at macro time; the misalignment instead
  surfaces downstream as a trait/type error at the `handle` call site
  rather than a clear macro-time message. It is documented, but it is a
  sharp footgun; consider emphasizing it more loudly or improving the
  downstream diagnostic.
- `define_scoped_row!` rejects generic rows ("generic scoped rows are
  deferred"). Parameterized scoped rows are not yet supported.
- The `Fn` vs `FnOnce` asymmetry is ergonomically sharp. Handler closures
  are `Fn` (callable inside `tail_rec_m`'s step) while each wrapper's
  `bind` takes `FnOnce`. Bridging requires interior mutability or
  `Rc` / `Arc` wrapping. Documented in `first_order.rs`, but worth a
  prominent mention in user-facing docs.
- `ArcRunExplicit` brand-level coverage is limited but not as limited as
  its docs claim. `ArcRunExplicitBrand` implements `SendPointed` and
  `SendRefPointed`; `SendFunctor`, `SendSemimonad`, `SendRefFunctor`, and
  `SendRefSemimonad` are not implemented (the functor and semimonad
  layers need a per-`A` HRTB or `Send`-closure bound that the trait
  method signatures cannot carry). Concrete-type inherent methods cover
  the by-value monadic surface. The `ArcRunExplicitBrand` source
  documentation is stale: it states coverage is "limited to `SendPointed`"
  and that the `SendRef` hierarchy is unreachable, but `SendRefPointed` is
  in fact implemented (`arc_run_explicit.rs`). Fix the doc as part of the
  capability audit in section 7.
- The default Erased `Run` relies on runtime type erasure and downcasts.
  The Erased substrate stores values as `Box<dyn Any>` and downcasts on
  the way out (for example `value.downcast().expect("Type mismatch in
Run boundary bind")` in `run/representation.rs`). This is an acceptable
  trade-off for the ergonomic default, but it is a runtime-invariant
  surface that should stay clearly separated from the type-directed
  Explicit family and be covered by focused tests; the `expect` paths are
  internal soundness assertions, not user-facing errors.

## 10. Effects and handlers worth porting

### purescript-run

`Run` ships Choose, Except, Reader, State, Writer (plus Internal). All
five are present in fp-library, and the scoped effects exceed it. The
only material gap is the `expand` combinator (see section 9). Note also
that purescript-run offers `liftAff` / `runBaseAff` for async IO interop;
the fp-library analog is the documented `spawn_blocking` workaround.

### heftia-effects (Control.Monad.Hefty)

Low cost, high value (first-order, mostly handlers that reinterpret as
State or Writer over the existing substrate):

- Fresh: generate a fresh value. Interpreted as a State counter
  (`interpret \Fresh -> get <* modify (+1)`). Very common (IDs, gensym).
- Input: consume a stream of inputs. Interpreted as State over a list.
  Reader-like but consuming.
- Output: emit outputs. Interpreted by accumulating to a list or a
  monoid (heftia's `runOutputMonoid` literally reuses Writer's `Tell`).
  Dual of Input. The accumulation convention (list vs monoid) is a
  decision to make; see the remediation plan's open questions.
- KVStore: key-value store (lookup / update). Interpreted as State over a
  `Map`. From polysemy-kvstore.

These four are cheap because each is largely a "reinterpret as State /
Writer" handler over machinery that already exists. They would
meaningfully broaden the standard library of effects.

Moderate (need a small decision before porting):

- Log: co-log-style logging. An Output specialized to log messages, so it
  should follow the Output accumulation-convention decision rather than
  lead it.
- Fail: `MonadFail` as an effect (abort with a message). Essentially
  `Except<String>`; needs a decision on whether a distinct identity from
  `Except` is warranted (see the remediation plan's open questions).
- Coroutine (`Yield a b`): yield an `a`, resume with a `b`. The handler
  produces a `Status` value (`Done ans | Continue a (b -> program)`),
  which needs a `Status` type carrying a continuation. The multi-shot
  wrappers fit this naturally.
- NonDet aggregation already partly exists. `named_helpers/nondet.rs`
  provides `run_empty` (interprets `Empty` into `Option<A>`) on all six
  wrappers and `run_choose` (interprets `Choose` into `Vec<A>`,
  concatenating the `true` then `false` branch results) on the four
  multi-shot wrappers. What is genuinely missing is a combined `Choose` +
  `Empty` runner that interprets both in one pass into a single
  `Alternative`-style collection (heftia's `runNonDet :: -> f a` and
  `runNonDetMonoid`), and a first-success / short-circuit helper. This is
  a narrowing of scope, not a from-scratch addition.

Advanced / higher-order (design first, do not treat as low-hanging):

- Shift / CC: delimited continuations (shift / reset). Powerful and
  distinctive, but it is answer-type-polymorphic continuation capture,
  which is a major semantic addition beyond the current mono-in-`A`
  scoped-handler model (see section 6). Multi-shot Rc / Arc wrappers are
  necessary but not sufficient; this needs a continuation-exposure
  policy first. Treat as a flagship feature to design deliberately, not
  a quick port.
- Provider: provide a resource or sub-interpreter to a scoped
  computation. Higher-order; fits the existing scoped-effect row, but
  the prior review notes it should follow (not lead) the Output / Writer
  and resource-scoping decisions.

Out of scope until an async base exists:

- Unlift (UnliftIO) and Concurrent (Parallel, Stream, Subprocess,
  Timer). These require an IO / async base monad and an async
  interpreter, which is the documented gap in section 6. Defer until a
  `Future`-capable interpreter lands.

## 11. Recommendations (prioritized)

P0 (highest leverage):

- Add Run-level `expand` / `weaken` (row subsumption) on top of the
  already-surfaced `CoproductEmbedder` / `CoproductSubsetter`. Unblocks
  composing programs written against different rows.
- Introduce code generation for the six wrappers, the per-effect brand
  siblings, and the smart constructors. This is the single biggest
  maintainability win and removes the class of drift bugs that
  `choose` / `ref_bracket` exemplifies. The macro module already reserves
  `define_effect!` / `define_scoped_effect!` for this.
- Audit and decide the brand / class capability matrix (see W3 in the
  remediation plan). Brand-level class coverage is partial and uneven
  even across the branded Explicit trio, and the Erased family has no
  brands at all (the non-explicit Free family is unbranded). Decide which
  gaps to close versus document, and fix the stale `ArcRunExplicitBrand`
  docs.

P1:

- Port the cheap first-order effects: Fresh, Input, Output, KVStore.
  Each is a reinterpret-as-State / Writer handler. (Log and Fail are
  close cousins but each needs a small decision first, so they sit in
  P2, see section 10.) For NonDet, add only the genuinely-missing
  pieces: a combined `Choose` + `Empty` runner and a first-success
  helper (per-effect `run_choose` / `run_empty` already exist).
- Feature-gate the subsystem (`effects`, with possible `effects-arc` /
  `effects-explicit` sub-features) to cut compile time for users who do
  not need the full cross product.
- Close the macro-surface asymmetry: add `rc_effects!` / `arc_effects!`
  or document `define_effect_row_aliases!` as the canonical Rc / Arc row
  path.

P2:

- Add a central "substrate and capability matrix" doc: which wrapper
  supports which effects and classes, the closure-flavour prefix legend,
  and the Explicit vs Erased axis. Pull the scattered known-limitations
  notes (mono-in-`A`, `Fn` vs `FnOnce`, `ArcRunExplicit` coverage) into
  one place.
- Write a design note for the scoped boundary / carrier / residual
  dispatch split, and evaluate consolidating those traits.
- Port Log, Fail, and Coroutine once their small decisions are made
  (Output accumulation convention, Fail identity, Coroutine `Status`
  shape); evaluate Shift / CC and Provider as flagship higher-order
  features.

P3 (longer term):

- Decide a runtime policy first, then build an async interpreter. Before
  any runtime-sensitive port (async, IO, cancellation, process
  lifecycle, target-monad lifting, continuation exposure), commit to an
  explicit policy; the engineering (a `MonadRec`-over-`Future` impl or a
  dedicated async substrate) follows that decision. This unlocks Unlift,
  the Concurrent family, and Shift / CC.
- Add a semantic-identity guard, or strengthen the docs, for the
  syntactic row-sort footgun.

## 12. Cross-check against the prior review (2026-05-16)

After writing the above, I restored and read the prior reviews under
`docs/plans/effects/review/`, in particular
`3-current-effects-system-review/effects-system-review.md` (dated
2026-05-16) and its `remediation-plan.md`. The two reviews agree on the
core picture: the dual-row model is the right foundation, the Box / Rc /
Arc split is semantically honest, the six-wrapper surface is large and
drift-prone, row canonicalization is structural and not semantic,
`define_scoped_row!` is concrete-only, and the runtime-heavy heftia
effects must be deferred.

Correct and still-relevant points from the prior review that this review
should fold in:

- Deferral is a policy gate, not just an engineering task. The prior
  review (and its decision D5) frames async / IO / Concurrent / Shift /
  CC / Unlift as blocked on an explicit, written runtime policy (async
  executor or blocking model, cancellation, IO embedding, process
  lifecycle, target-monad lifting, `Send + Sync`, continuation
  exposure). Section 11 P3 above is updated to reflect that policy gate.
- Shift / CC is a larger semantic step than "multi-shot continuations."
  The prior review correctly characterizes it as answer-type-polymorphic
  continuation capture, which is beyond the current mono-in-`A` model.
  Section 10 above is updated accordingly.
- Default Erased `Run` rests on runtime downcasts. The prior review
  flags this as a surface to keep separated from the Explicit family and
  to cover with focused tests. Verified at `run/representation.rs`;
  captured in section 9 above.
- A partial helper-drift mitigation already exists. The
  `documented_helper_impls!` marker and decision D6 predate this review;
  the code-generation recommendation in section 4 now references it.
- Port-candidate tiering is slightly more conservative there. The prior
  review puts `Log` and `Provider` under "worth designing" rather than
  "low-risk," and does not list `Fail`. Treat `Input` / `Output` /
  `Fresh` / `KVStore` as the safe low-risk set; `Log` should follow the
  Output / Writer decision, and `Fail` should be checked against whether
  a distinct identity from `Except` is wanted.

Points this review adds that the prior one did not cover:

- The missing `expand` / `weaken` row-subsumption combinator
  (verified absent crate-wide). The prior review lists
  `CoproductEmbedder` among available row values but does not flag the
  absence of a Run-level subsumption combinator. This is the most
  significant net-new finding here.
- The subsystem is not feature-gated, with the resulting always-on
  compile cost.
- The brand / class capability matrix: the Erased wrapper family has no
  brands at all, and even the branded Explicit trio has partial, uneven
  class coverage (`RunExplicitBrand` fullest; `RcRunExplicitBrand` lacks
  owned `Functor` / `Semimonad`; `ArcRunExplicitBrand` has only
  `SendPointed` + `SendRefPointed`).
- The `rc_effects!` / `arc_effects!` row-macro asymmetry against the
  `RowHeadWrap` Rc / Arc variants.
- An explicit framing of the mono-in-`A` interpreter as a named
  limitation, and its link to why Shift / CC is hard.
