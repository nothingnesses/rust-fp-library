# Effects System Review, 2026-05-15

## Scope

This review covers the current effects implementation in:

- [`fp-library/src/brands/effects.rs`](../../../../../fp-library/src/brands/effects.rs).
- [`fp-library/src/types/effects.rs`](../../../../../fp-library/src/types/effects.rs) and its submodules.
- [`fp-macros/src/effects.rs`](../../../../../fp-macros/src/effects.rs) and its submodules.

It also compares the current surface against the local reference copies of:

- `/home/jessea/Documents/projects/purescript-run/src/Run`.
- `/home/jessea/Documents/projects/effects/heftia/heftia-effects/src/Control/Monad/Hefty`.

## Executive Summary

The effects system is no longer a proof-of-concept. It now has a serious
semantic core:

- A dual-row `Run<R, S, A>` model where `R` is the first-order row and `S`
  is the scoped-effect row.
- Six wrapper families: `Run`, `RcRun`, `ArcRun`, `RunExplicit`,
  `RcRunExplicit`, and `ArcRunExplicit`.
- First-order effects for State, Reader, Except, Writer, and Choose.
- Scoped effects for Catch, Local, RefLocal, Bracket, RefBracket, and Span.
- Standard scoped handler values that run those scoped effects through the
  interpreter machinery.
- Proc macros for row construction, handler-list construction, named scoped
  rows, and inherent-method do notation.
- Regression coverage for reference examples from PureScript Run and Heftia,
  plus targeted regressions for continuation ordering, repeated shared use,
  typed boundaries, and scoped lifecycle behavior.

The current architecture is directionally sound. The H2-style internal
continuation boundary, private raw-step extraction, and result-polymorphic
handler/replacer protocols are the right kind of architecture for Rust: they
make the selected scoped action and the outer continuation explicit instead of
hoping ordinary `Functor::map` can preserve around-action semantics.

The main issue is that the public surface has not caught up with the internal
architecture. Naming still says "dispatcher" in places where the public concept
is now "handler"; top-level re-exports are inconsistent; row and witness
spelling is still too noisy; missing-handler diagnostics are raw trait errors;
and the large wrapper/handler modules are hard to review because they mix public
API, private continuation machinery, and tests.

## What Has Been Achieved

### Dual-row Run substrate

The central substrate is `Node<'a, R, S, A>`:

- `Node::First(...)` stores one first-order effect layer from row `R`.
- `Node::Scoped(...)` stores one scoped-effect layer from row `S`.
- `NodeBrand<R, S>` gives the Free-family substrates a single brand wrapping
  both rows.

This is the key architectural move. It separates ordinary first-order effects
from around-action scoped effects, while still letting all six Run wrappers use
the existing Free / FreeExplicit / RcFree / ArcFree substrate families.

### Run wrapper family

The implementation currently has six public program wrappers:

| Wrapper                       | Substrate                                                         | Multiplicity / thread model                  | Role                                                                                                                                                 |
| ----------------------------- | ----------------------------------------------------------------- | -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Run<R, S, A>`                | `Free<NodeBrand<R, S>, A>` plus a private boundary representation | single-shot, default erased                  | Main ergonomic default. Supports Box-backed first-order and scoped effects, plus representation-native boundary frames for around-action operations. |
| `RcRun<R, S, A>`              | `RcFree<NodeBrand<R, S>, A>`                                      | multi-shot, single-threaded                  | Shared/reusable programs with `Rc` closure cells.                                                                                                    |
| `ArcRun<R, S, A>`             | `ArcFree<NodeBrand<R, S>, A>`                                     | multi-shot, `Send + Sync`                    | Shared/reusable programs across threads with `Arc` closure cells and Send-aware brands.                                                              |
| `RunExplicit<'a, R, S, A>`    | `FreeExplicit<'a, NodeBrand<R, S>, A>`                            | single-shot, explicit lifetime               | Brand-dispatched and explicit-lifetime sibling. Uses typed boundary values for around-action scoped constructors.                                    |
| `RcRunExplicit<'a, R, S, A>`  | `RcFreeExplicit<'a, NodeBrand<R, S>, A>`                          | multi-shot, explicit lifetime                | Rc-backed explicit sibling.                                                                                                                          |
| `ArcRunExplicit<'a, R, S, A>` | `ArcFreeExplicit<'a, NodeBrand<R, S>, A>`                         | multi-shot, explicit lifetime, `Send + Sync` | Arc-backed explicit sibling.                                                                                                                         |

The wrappers now expose a broad common surface:

- Core constructors and destructors: `pure`, `send`, `lift`, `peel`,
  `extract`, `from_*_free`, `into_*_free`.
- Composition: `map`, `bind`, and by-reference variants on Rc/Arc and Explicit
  where supported.
- First-order interpretation: `interpret`, `run`, `interpret_rec`, `run_rec`,
  `interpret_with`, `interpose`, and `interpret_with_either`.
- Scoped interpretation: `interpret_scoped_with` and full `interpret` /
  `run` paths that accept both first-order and scoped handler lists.
- Default/shared wrapper refinements: `RunFirstOrderHandler`,
  `RunFirstOrderReplacer`, `RcRunFirstOrderReplacer`, and
  `ArcRunFirstOrderReplacer` for result-polymorphic first-order rewrites under
  boundary-backed scoped operations.
- Smart constructors: `get`, `put`, `ask`, `throw`, `tell`, `choose` where
  supported, plus scoped constructors `catch`, `local`, `ref_local`, `span`,
  `bracket`, and `ref_bracket` where semantically valid for the wrapper.

### Row machinery

The row substrate is built from:

- `CNilBrand`, the empty row.
- `CoproductBrand<H, T>`, the recursive row brand.
- `VariantF<H, T>`, a first-order row alias over the coproduct brand.
- `ScopedCoproduct<H, T>` and `ScopedNil`, scoped-row aliases over the same
  row encoding.
- `Member<E, Idx>`, the row-membership trait used by `lift` and smart
  constructors.
- `CoproductEmbedder`, row-removal / embedding machinery used by `interpose`
  and standard scoped handlers that remove an effect from the first-order row.

Rows are sorted lexically by the macros, so `effects![A, B]`,
`scoped_effects![A, B]`, `handlers! { ... }`, and `scoped_handlers! { ... }`
produce aligned type-level and value-level lists.

### First-order effects

The first-order effect set currently includes:

| Effect | Main type families                  | Smart constructor surface       | Notes                                                                                                                                                               |
| ------ | ----------------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| State  | `State`, `SendState`, `BoxState`    | `get`, `put`                    | Mirrors PureScript Run's `State`. Default wrappers use `BoxStateBrand`; Rc wrappers use `StateBrand`; Arc wrappers use `SendStateBrand`.                            |
| Reader | `Reader`, `SendReader`, `BoxReader` | `ask`                           | Mirrors PureScript Run's `Reader`. Local is modeled as a scoped effect rather than as a pure first-order rewrite.                                                   |
| Except | `Except`                            | `throw`                         | First-order throw effect. Catch is a scoped effect over action/recovery programs.                                                                                   |
| Writer | `Writer`                            | `tell`                          | First-order tell exists, but `listen` / `censor` scoped Writer semantics are not implemented yet.                                                                   |
| Choose | `Choose`, `SendChoose`, `BoxChoose` | `choose` on multi-shot wrappers | The shipped operation is a Boolean branch continuation. It intentionally ships only on multi-shot wrappers because handlers invoke the continuation more than once. |

The Box/Rc/Arc sibling split is an important implementation theme:

- Box-backed default wrappers can store `FnOnce` continuations.
- Rc-backed wrappers store reusable `Fn` closures.
- Arc-backed wrappers store reusable `Fn + Send + Sync` closures and use
  Send-aware brands where ordinary `Functor` cannot express the bounds.

### Scoped effects

The scoped effect set currently includes:

| Effect     | Main type families                                                    | Semantics                                                                                                            |
| ---------- | --------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Catch      | `BoxCatch`, `Catch`, `SendCatch`                                      | Runs an action; if a matching `Except` throw occurs inside it, run the recovery handler.                             |
| Local      | `BoxLocal`, `Local`, `SendLocal`                                      | Runs an action under an environment transformed by value, `E -> E`.                                                  |
| RefLocal   | `BoxRefLocal`, `RefLocal`, `SendRefLocal`                             | Runs an action under an environment transformed from a borrow, `&E -> E`.                                            |
| Bracket    | `BoxBracket`, `Bracket`, `SendBracket`, plus Explicit-family siblings | Runs acquire, body, release, and preserves release-before-outer-continuation ordering.                               |
| RefBracket | `RefBracket`, `SendRefBracket`, plus Explicit-family siblings         | Rc/Arc-only resource lifecycle where body/release receive pointer clones of the resource.                            |
| Span       | `BoxSpan`, `Span`, `SendSpan`                                         | Runs an action under a tag. Current standard handler resumes the action; it is a hook for instrumentation semantics. |

The architecture distinguishes ordinary scoped rows from around-action scoped
rows. Around-action operations need to keep two things separate:

- The selected action's result type.
- The outer program continuation's final result type.

That separation is what drove the H2 internal continuation-carrier design and
the private boundary types in the Explicit wrappers.

### Handler machinery

First-order handlers are runtime cons lists:

- `Handler<E, F>` tags one closure with effect brand `E`.
- `HandlersNil` and `HandlersCons<H, T>` form a list aligned with the row.
- `nt()` starts a builder chain.
- `DispatchHandlers<'a, Layer, NextProgram>` walks the value-level coproduct
  and handler list in lock-step.

Scoped handlers mirror the same idea:

- `ScopedHandler<S, F>` tags one scoped handler value with scoped brand `S`.
- `ScopedHandlersNil` and `ScopedHandlersCons<H, T>` form a scoped-handler
  list.
- `scoped_nt()` starts a scoped builder chain.
- `DispatchScopedHandler`, `DispatchScopedHandlers`, and
  `DispatchScopedBoundaryHandlers` dispatch ordinary scoped layers and typed
  boundary values.

The important difference is that scoped handler cells are not ordinary closure
cells in the general case. They need a method generic over the concrete
first-order handler list so nested actions can be interpreted without erasing
the first-order handler list behind `dyn`.

### Standard scoped handlers

At review time, the module named `scoped_dispatchers` contained standard
handler values for built-in scoped effects:

- `CatchDispatcher` / `catch_dispatcher`.
- `LocalDispatcher` / `local_dispatcher`.
- `RefLocalDispatcher` / `ref_local_dispatcher`.
- `BracketDispatcher` / `bracket_dispatcher`.
- `RefBracketDispatcher` / `ref_bracket_dispatcher`.
- `SpanDispatcher` / `span_dispatcher`.

Phase 5 step 5.2 renamed that public surface to
`standard_scoped_handlers` with `*Handler` / `*_handler` names. These values
implement the ordinary scoped-handler and boundary/raw scoped-handler protocols
needed by the wrapper families.

### Macro surface

The effects macro module currently contains:

| Macro                | Worker module      | Purpose                                                                                   |
| -------------------- | ------------------ | ----------------------------------------------------------------------------------------- |
| `effects!`           | `effects_macro.rs` | Builds a lexically sorted first-order row where effects are Coyoneda-wrapped.             |
| `raw_effects!`       | `effects_macro.rs` | Builds a lexically sorted raw row without Coyoneda wrapping. Internal/supporting surface. |
| `scoped_effects!`    | `effects_macro.rs` | Builds a lexically sorted scoped row.                                                     |
| `define_scoped_row!` | `scoped_row.rs`    | Generates a named marker row item for recursive scoped rows.                              |
| `handlers!`          | `handlers.rs`      | Builds a lexically sorted first-order handler list.                                       |
| `scoped_handlers!`   | `handlers.rs`      | Builds a lexically sorted scoped-handler list.                                            |
| `im_do!`             | `im_do/`           | Inherent-method monadic do notation for Run wrappers.                                     |

The macro direction is pragmatic: macros construct rows and handler lists, but
do not attempt to type-check whether a specific handler list covers a specific
program row. Coverage is still enforced by trait bounds during compilation.

## Design and Architecture Assessment

### Strengths

1. The dual-row model is the right core model.

   First-order and scoped effects have different operational meaning. Keeping
   them in distinct rows makes the type-level model honest and avoids treating
   around-action effects as just another functor layer.

2. The pointer-brand split is justified.

   The Box/Rc/Arc split looks verbose, but it maps to real semantic differences:
   single-shot `FnOnce`, repeated single-threaded `Fn`, and repeated
   thread-safe `Fn + Send + Sync`. Trying to collapse these into one effect
   family would likely reintroduce the same bound and HRTB walls that the plan
   has already surfaced.

3. The H2 continuation-boundary approach is the right long-term direction.

   Around-action operations cannot be modeled correctly by simply mapping over a
   suspended scoped cell once the selected action result and final result differ.
   The current design exposes that continuation boundary internally and keeps it
   private from the public API, which is exactly where it should live until the
   custom scoped-handler API has real downstream requirements.

4. The standard-handler implementation is semantically stronger than it was.

   Catch, Local, RefLocal, Bracket, RefBracket, and Span now have tests covering
   continuation order, repeated Rc/Arc use, Explicit typed boundaries, and
   default `Run` raw boundary paths. The recent Heftia and composition tests
   provide useful acceptance coverage beyond isolated shape tests.

5. The macro surface is useful without being too magical.

   `effects!`, `scoped_effects!`, `handlers!`, and `scoped_handlers!` remove
   mechanical row/list construction while still leaving the effect row model
   visible. `define_scoped_row!` solves the item-position need for recursive
   scoped rows without turning everything into one broad "effect stack" macro.

### Structural and organizational issues

1. Public naming was behind the design before Phase 5 step 5.2.

   User-facing standard scoped values were called dispatchers, but they are
   conceptually handlers. Phase 5 step 5.2 renamed
   `scoped_dispatchers` to `standard_scoped_handlers` and renamed the public
   values to `CatchHandler` / `catch_handler`, with the same treatment for
   Local, RefLocal, Bracket, RefBracket, and Span. Internal `Dispatch*` traits
   can keep dispatch names because they describe the implementation protocol.

2. Top-level re-exports were inconsistent before Phase 5 step 5.2.

   [`types/effects.rs`](../../../../../fp-library/src/types/effects.rs) previously
   re-exported only Catch and Span handler values from the standard-handler
   module, even though the module also exposed Local, RefLocal, Bracket, and
   RefBracket handlers. Phase 5 step 5.2 resolved this by keeping all standard
   handler constructors and types under
   `types::effects::standard_scoped_handlers` and removing the partial
   top-level re-export set.

3. The module documentation in `types/effects.rs` was stale before Phase 5
   step 5.2.

   It said `scoped_dispatchers` contained standard dispatcher values "such as
   Catch and Span." Phase 5 step 5.2 updated the module text to describe
   `standard_scoped_handlers` as standard handler values for built-in scoped
   effects such as Catch, Local, Bracket, and Span.

4. Several files are too large for routine review.

   Current line counts are high:
   - `run.rs`: about 5,250 lines.
   - `run_explicit.rs`: about 6,149 lines.
   - `arc_run_explicit.rs`: about 5,363 lines.
   - `arc_run.rs`: about 4,776 lines.
   - `rc_run_explicit.rs`: about 4,734 lines.
   - `rc_run.rs`: about 3,961 lines.
   - `scoped_dispatchers/local.rs`: about 2,596 lines.
   - `scoped_dispatchers/ref_local.rs`: about 2,594 lines.
   - `scoped_dispatchers/catch.rs`: about 2,280 lines.
   - `scoped_dispatchers/bracket.rs`: about 2,247 lines.

   The current new-style module split is better than the earlier single-file
   shape, but the large wrapper modules still mix public API, private raw-step
   extraction, boundary protocols, standard smart constructors, and unit tests.
   This is manageable for now, but review cost is rising.

5. Wrapper duplication is high but partly structural.

   The six wrappers repeat many method families. Some duplication is unavoidable
   because the closure storage and Send/Sync obligations genuinely differ.
   Refactoring should focus on private helper functions and shared tests, not
   on forcing a premature generic abstraction over all wrappers.

6. The public API has two naming eras.

   The core methods still use `interpret`, `interpret_with`, and
   `interpret_scoped_with`, while the desired user model is `handle` and
   handlers. It is reasonable to defer the method rename until after standard
   handler naming settles, but this is a real public-language inconsistency.

7. Row witness ergonomics remain rough.

   Tests still define many aliases like `FirstRow`, `ScopedRow`,
   `FirstRowMinusReader`, and `FirstRowMinusExcept`, then pass those witnesses
   into standard handler constructors. This is correct but too noisy for the
   eventual guide-level API.

8. Missing-handler diagnostics are technically correct but not friendly.

   The existing compile-fail tests prove missing first-order and scoped handlers
   are rejected. The current diagnostics are raw trait-bound failures over
   `HandlersNil` / `ScopedHandlersNil` and value-level coproduct projections.
   A user can eventually learn from them, but they do not directly say "add a
   handler for this remaining row cell."

## Specific Concern: Ref Counterparts for Local and Bracket Only

The current shape does make sense semantically.

### Local vs RefLocal

`Local` and `RefLocal` are meaningfully different operations:

- `Local` transforms an owned environment value with `E -> E`.
- `RefLocal` transforms from a borrow with `&E -> E`.

The Ref variant removes unnecessary `E: Clone` pressure for large or
non-cheap-to-clone environments. This is a clear semantic and ergonomic
distinction, so the pair is justified.

### Bracket vs RefBracket

`Bracket` and `RefBracket` are also meaningfully different:

- `Bracket` passes owned resource values through the lifecycle shape.
- `RefBracket` shares the acquired resource through refcounted pointer clones so
  body and release can both observe the same acquired resource without moving it
  twice.

That difference is fundamental to resource lifecycle semantics in Rust, so the
pair is justified.

### Catch without RefCatch

The absence of `RefCatch` is not obviously wrong. Catch's primary parameter is
an error payload, not a surrounding environment or acquired resource. The
current split already covers the main semantics:

- Default Box-backed Catch supports non-Clone errors through single-shot
  `FnOnce` recovery.
- Rc/Arc Catch require clone/send bounds where repeated use or thread-safety
  demands them.

A `RefCatch` could be imagined as a recovery handler receiving `&E` instead of
`E`, mainly to reduce clone pressure for shared wrappers. That is a niche
extension, not a missing core operation. It should not be added unless a real
use case needs repeated shared recovery over expensive or non-owned errors.

### Span without RefSpan

The absence of `RefSpan` is also acceptable for now. Span's tag is metadata,
not a scoped environment or lifecycle resource:

- Default `BoxSpan` already supports non-Clone tags because it is single-shot.
- Rc/Arc Span require `Tag: Clone` because a multi-shot program may be resumed
  more than once.

A `RefSpan` or pointer-backed tag variant could be useful if users need Rc/Arc
multi-shot spans with expensive or non-Clone tags. That should be considered a
future ergonomics/performance extension, not a semantic hole in the current
system.

## Missing or Incomplete Surfaces

### From PureScript Run

The closest PureScript Run features not fully represented yet are:

1. Continuation-passing interpreters.

   PureScript Run exposes `runCont` and `runAccumCont`. The Rust plan already
   tracks these as deferred. They would let a handler receive continuations more
   explicitly, which may align with the current H2 boundary architecture later.

2. Accumulator interpreters.

   PureScript Run exposes `runAccum`, `runAccumRec`, and `runAccumPure`. The
   current Rust implementation has `interpret_rec` / `run_rec`, but not the full
   accumulator interpreter family.

3. Pure row expansion / base effect lifting.

   PureScript Run has `expand`, `liftEffect`, `liftAff`, `runBaseEffect`, and
   `runBaseAff`. Rust does not yet have a direct equivalent for widening rows or
   base IO/async effect lifting. This should wait until the project has a clear
   story for async / IO target monads.

4. Writer `censor` and writer folds.

   Rust currently has first-order `tell`, but not PureScript Run's `censor`,
   `foldWriter`, or `runWriter` equivalent as a polished standard handler
   family. This is the most natural next effect-family gap after naming and
   ergonomics cleanup.

5. Except helper functions.

   PureScript Run has helpers such as `fail`, `rethrow`, `note`, and `fromJust`.
   Rust currently has `throw` and Catch. These helpers are useful but secondary;
   they should be added only after the naming and handler ergonomics pass.

6. Choose Empty.

   PureScript Run's `Choose` has `Empty` and `Alt`. Rust currently exposes the
   Boolean branch operation, but no separate `Empty` effect/smart constructor.
   The Heftia NonDet surface also has `Choose` and `Empty`. This is a real
   semantic gap for full nondeterminism.

### From Heftia

The Heftia modules suggest these future effect families:

Terminology note: this review uses "scoped effect" for the Rust library's
current subset of Heftia-style higher-order effects whose operation controls the
dynamic extent of an action in the scoped row `S`. It should not be read as a
synonym for every higher-order effect. Before porting a Heftia higher-order
effect beyond action-scoped cases, decide whether it belongs in `S` or needs a
separate continuation, resumption, async, IO, or target-monad protocol.

1. Writer higher-order operations.

   `WriterH` with `Listen` and `Censor` is the most relevant scoped-effect
   candidate. It directly exercises around-action handler semantics and would
   build on existing `Writer` / `Tell`.

2. NonDet `Empty` plus richer choose handlers.

   Heftia separates `Choose` and `Empty` and provides Alternative/Monoid
   interpreters. Rust has `Choose` but not `Empty` and not a standard run
   handler producing collections/monoids.

3. Input, Output, Fresh, Log, and KVStore.

   These are mostly first-order effects or effects implemented via existing
   State/Writer-like handlers. They are good candidates after the API guide and
   custom-effect boilerplate story are clearer.

4. Coroutine.

   Heftia's `Yield` interpreter returns a status containing either completion
   or a continuation. This would be valuable because it tests explicit
   continuation return surfaces, but it should come after the handler naming and
   continuation-facing API is stable.

5. CC and Shift.

   These are control effects and are more architecture-sensitive. They should be
   deferred until the H2 boundary design has survived user-facing handler docs
   and at least one more higher-order scoped effect.

6. Provider, Unlift, Stream, Subprocess, Timer, and Parallel.

   These rely on runtime/IO/concurrency design that the Rust library has not
   settled yet. They should not be near-term Phase 5 work.

## Inconsistencies and Findings

### Finding 1: standard scoped handlers were named as dispatchers

Severity: high for API polish, low for semantic correctness.

The implementation has settled on public handler lists (`handlers!`,
`scoped_handlers!`) and standard values that users pass as handler-list cells.
Calling those values dispatchers leaks the implementation protocol into the
user-facing vocabulary.

Recommendation: complete Phase 5 step 5.2 before adding more public helpers.
Rename the module and public values to handler vocabulary. Keep `Dispatch*`
names for traits that are truly internal protocols.

Status: shipped by Phase 5 step 5.2.

### Finding 2: top-level standard-handler exports were incomplete

Severity: medium.

At review time, `types/effects.rs` re-exported only Catch and Span dispatcher
names from the standard scoped dispatcher module. Local, RefLocal, Bracket, and
RefBracket were available through the submodule but not through the same
top-level path.

Recommendation: during the rename pass, decide a consistent export policy. If
top-level `types::effects::*` exports standard handler constructors, export all
standard handler constructors. If not, export none and require the named module
path.

Status: shipped by Phase 5 step 5.2. Standard handler constructors and types now
live under `types::effects::standard_scoped_handlers`; the partial top-level
Catch / Span re-export set was removed.

### Finding 3: row/witness spelling is still too noisy

Severity: medium.

Recent tests still require repeated type aliases for first-order rows, scoped
rows, and row-minus witnesses. This is especially visible around Local/Reader,
RefLocal/Reader, Catch/Except, and mixed composition rows.

Recommendation: first try constructor inference after handler naming is cleaned
up. If inference cannot hide enough of the witness spelling, add a narrow
item-position row-alias helper that defines named rows and row-minus aliases.
Do not jump straight to a broad effect-stack macro.

### Finding 4: missing-handler errors are not domain-guided

Severity: medium.

The compile-fail tests prove missing handlers are rejected, but the error text
is a long trait-bound mismatch. It exposes `HandlersNil`,
`ScopedHandlersNil`, and nested `Coproduct` projections rather than explaining
what users should add.

Recommendation: add examples and trait-level documentation first. Then explore
whether marker traits or custom helper methods can create better error anchors
without changing the core dispatch protocol. The proc macros alone cannot solve
coverage diagnostics because they do not know the interpreted program row.

### Finding 5: Writer is only half complete

Severity: medium.

The library has `Writer::Tell` and `tell` smart constructors, but the reference
systems both expose richer Writer handling:

- PureScript Run: `censor`, `foldWriter`, `runWriter`.
- Heftia: `WriterH` with `Listen` and `Censor`, plus pre/post semantics.

Recommendation: after Phase 5 naming/ergonomics cleanup, Writer higher-order
semantics are the best next scoped-effect family to implement. They exercise
real around-action behavior without introducing external IO/runtime concerns.

### Finding 6: NonDet is incomplete without Empty

Severity: medium.

Rust has `Choose::Alt`, but PureScript Run and Heftia model failure/empty as
part of nondeterminism. Without `Empty`, the library cannot fully express the
standard Alternative semantics for nondeterministic programs.

Recommendation: add an `Empty` first-order effect or extend the Choose family
only after deciding whether `Empty` is conceptually part of `Choose` or a
separate effect. Heftia's split suggests a separate `Empty` effect is cleaner.

### Finding 7: module size is now a maintainability concern

Severity: medium.

The large wrapper files are difficult to review and easy to regress. The
standard scoped handler files are also large because each effect carries
ordinary, boundary, raw, default, Rc, Arc, and Explicit variants in one file.

Recommendation: do not refactor file layout in the middle of a semantic change.
After the handler rename, split by stable concerns:

- Public wrapper methods.
- Private representation/raw-step helpers.
- Boundary/carrier protocols.
- Smart constructors.
- Tests.

Use new-style modules only.

Status: Phase 5 step 5.3 started with the least risky split: the large inline
test modules for the main Run wrappers and interpreter substrate now live in
new-style child test modules. The first production-code split followed the
same rule by moving default `Run`'s private representation, raw boundary frame,
raw selected-action continuation carrier, and raw scoped-dispatch protocols
into `run/representation.rs`. The next production split moved
`RunExplicit`'s typed boundary wrapper, action-supplied continuation carriers,
per-effect carrier layers, and Explicit resume impls into
`run_explicit/boundary.rs`. The following production split mirrored that stable
concern boundary for `ArcRunExplicit` by moving its typed boundary wrapper,
action-supplied continuation carriers, and Arc Explicit resume impls into
`arc_run_explicit/boundary.rs`. The next production split moved `ArcRun`'s raw
scoped handler protocol, raw selected-action continuation carrier, and Arc
scoped continuation carrier into `arc_run/raw_scoped.rs`. The following
production split moved `RcRunExplicit`'s typed boundary wrapper,
action-supplied continuation carrier, and Rc Explicit resume impls into
`rc_run_explicit/boundary.rs`. The next production split mirrored the
raw-scoped concern for `RcRun` by moving its raw scoped handler protocol, raw
selected-action continuation carrier, and Rc scoped continuation carrier into
`rc_run/raw_scoped.rs`. The following production split moved
`ArcRunExplicit`'s public first-order and scoped smart constructors into
`arc_run_explicit/smart_constructors.rs`. The next production split mirrored
that public smart-constructor concern for `ArcRun` in
`arc_run/smart_constructors.rs`. The following production split moved default
`Run`'s public first-order and scoped smart constructors into
`run/smart_constructors.rs`. The next production split moved `RunExplicit`'s
public first-order and scoped smart constructors into
`run_explicit/smart_constructors.rs`. The following production split moved
`RcRun`'s public first-order and scoped smart constructors into
`rc_run/smart_constructors.rs`. The next production split moved
`RcRunExplicit`'s public first-order and scoped smart constructors into
`rc_run_explicit/smart_constructors.rs`. The next production split moved
first-order handler dispatch (`DispatchHandlers` plus the CNil / Coyoneda /
RcCoyoneda / ArcCoyoneda impls) into `interpreter/first_order.rs`. The next
production split moved the interpreter's private scoped-resume protocol
vocabulary, boundary projection aliases, family-specific resume traits,
`ScopedContinuation`, and `IntoScopedBoundaryParts` into
`interpreter/scoped_resume.rs`. The following production split moved
`Bracket`'s Explicit-family cells and trait impls into `bracket/explicit.rs`.
The standard-handler pilot split moved `Span`'s Explicit carrier-aware boundary
and carrier-cell support into `standard_scoped_handlers/span/carrier.rs`. The
next standard-handler split moved `Local`'s Explicit carrier-aware boundary,
focused carrier helpers, and Rc/Arc Explicit carrier facades into
`standard_scoped_handlers/local/carrier.rs`. The following standard-handler
split moved `RefLocal`'s Explicit carrier-aware boundary, focused carrier
helpers, and Rc/Arc Explicit carrier facades into
`standard_scoped_handlers/ref_local/carrier.rs`. The next standard-handler
split moved `Catch`'s Explicit carrier-aware boundary, focused carrier helpers,
and Rc/Arc Explicit carrier facades into
`standard_scoped_handlers/catch/carrier.rs`. The
remaining 5.3 module-split scope is finite rather than open-ended: apply the
same Explicit carrier-aware split to `Bracket` and `RefBracket`; then run one
raw first-order-replacer checkpoint for `Local`, `RefLocal`, and `Catch`. Raw
replacers should move only if the checkpoint shows they still obscure
reviewability after the carrier splits and can move as complete named concerns
without API or semantic changes. Otherwise step 5.4 should proceed.

### Finding 8: custom-effect authoring is still verbose

Severity: low to medium.

The TalkF/DinnerF and Heftia custom-effect tests show the current boilerplate:
brand, enum, `impl_kind!`, `Functor`, `WrapDrop`, smart constructors, row
aliases, and handler closures. That is acceptable for tests but too verbose for
eventual user docs.

Recommendation: defer `define_effect!` until after the handler surface is
renamed and documented. Then use the guide-writing process to decide whether
the repeated pattern is stable enough for a macro.

## Approaches to Address the Findings

### Handler vocabulary and exports

Problem addressed: Findings 1 and 2.

Options:

1. Keep the current dispatcher names until Phase 5 is otherwise complete.

   Trade-offs:
   - Lowest immediate churn.
   - Avoids touching many tests and doctests before more semantic work.
   - Preserves the public mismatch between `scoped_handlers!` and
     `scoped_dispatchers`.
   - New helpers would have to use either old names or introduce a second
     vocabulary later.

2. Add handler aliases while keeping dispatcher names.

   Trade-offs:
   - Gives users the better names immediately.
   - Reduces migration pressure.
   - Leaves two public names for every standard scoped handler, which makes docs
     and examples noisier.
   - Conflicts with the project's stance against preserving in-progress API
     compatibility when it produces debt.

3. Do a breaking rename now, without compatibility aliases.

   Trade-offs:
   - Largest immediate edit because tests, doctests, imports, and examples must
     move together.
   - Gives the public API one vocabulary: programs are handled by handlers;
     dispatch is the implementation protocol.
   - Prevents new row/witness helpers from baking in old names.
   - Forces the export policy to be cleaned up at the same time.

Recommendation: choose option 3.

Reasoning: this is the most aligned with the current API stability stance. The
effects API is still in progress, and the old names are not just cosmetic: they
teach the wrong conceptual model. Rename public standard scoped handler modules,
types, and constructors first; keep `Dispatch*` names only for implementation
traits and methods that actually perform protocol dispatch. During the same
step, decide whether `types::effects` re-exports all standard handler
constructors or none. Partial re-export is the worst outcome because it makes
some standard handlers look more public than others.

### Row and witness ergonomics

Problem addressed: Finding 3.

Options:

1. Do nothing and rely on explicit row aliases in user code.

   Trade-offs:
   - Keeps every type-level witness visible and debuggable.
   - Avoids macro/API work.
   - Leaves guide examples and real programs with a lot of repetitive
     `FirstRow`, `ScopedRow`, and `FirstRowMinus*` boilerplate.
   - Makes the library feel more like a substrate than a usable effect system.

2. Try function/type inference on standard handler constructors first.

   Trade-offs:
   - Keeps the surface as ordinary Rust items.
   - Improves ergonomics without adding macro expansion indirection.
   - May hit stable Rust inference limits because the row-minus and embedding
     witnesses appear only in trait-selection context.
   - If it fails, the failure gives precise evidence for any macro/helper
     fallback.

3. Add a narrow item-position row-alias helper.

   Trade-offs:
   - Directly targets repeated named-row boilerplate.
   - Preserves visible row aliases and row-minus aliases for diagnostics.
   - Adds a macro surface, but one with a narrow job.
   - Does not hide handler construction or program construction.

4. Add a broad effect-stack macro that defines rows, row-minus aliases,
   handlers, and perhaps program aliases together.

   Trade-offs:
   - Could make examples very short.
   - Hides too much of the type model at once.
   - Makes diagnostics harder to relate to source code.
   - Risks becoming the central public API before the lower-level handler
     surface is stable.

Recommendation: choose option 2 first, option 3 only if repetition remains.

Reasoning: inference-first preserves the ordinary Rust API and avoids creating
macro debt to cover a problem that better constructor signatures might solve.
If inference cannot make `catch_handler`, `local_handler`, and
`ref_local_handler` pleasant enough, add a narrow row-alias helper. Do not add a
broad effect-stack macro in Phase 5; it would freeze too much of the surface
before Writer, NonDet Empty, and the public guide clarify the recurring shapes.

### Missing-handler diagnostics

Problem addressed: Finding 4.

Options:

1. Keep only the current compile-fail tests.

   Trade-offs:
   - No implementation cost.
   - Keeps the core dispatch traits untouched.
   - Leaves users with raw trait-bound errors over nested coproduct types.

2. Improve documentation and examples around missing first-order/scoped handler
   failures.

   Trade-offs:
   - Low-risk and immediately useful.
   - Does not improve compiler output directly.
   - Gives users a reference for decoding the current error shapes.

3. Add diagnostic anchor traits or helper methods.

   Trade-offs:
   - May let compiler errors mention domain concepts like missing first-order
     handler or missing scoped handler.
   - Could improve trybuild coverage with more intentional error text.
   - Risks adding traits solely for diagnostics, which can complicate the
     dispatch model if not kept separate.

4. Make `handlers!` / `scoped_handlers!` validate row coverage.

   Trade-offs:
   - The desired user experience is attractive.
   - Not feasible as the primary solution because the proc macros see only the
     handler-list entries, not the type of the program being interpreted.
   - Would require a broader macro that owns both program and handler context,
     which is the wrong direction right now.

Recommendation: choose option 2 now, investigate option 3 only after the rename.

Reasoning: examples can be improved without changing semantics. Diagnostic
anchors are worth exploring, but only if they remain a thin layer over the
existing dispatch traits. Macro-level coverage validation is not a good fit for
the current API because coverage is a relationship between a program row and a
handler list, and the standalone handler-list macros do not have both sides.

### Writer and NonDet gaps

Problem addressed: Findings 5 and 6.

Options:

1. Implement Writer higher-order semantics next.

   Trade-offs:
   - Builds directly on the existing `Writer::Tell` first-order effect.
   - Exercises scoped around-action semantics with `listen` and `censor`.
   - Has strong reference coverage in both PureScript Run and Heftia.
   - Requires a design decision around pre- vs post-applying censor semantics,
     because Heftia exposes both.

2. Implement NonDet Empty and richer Choose handlers next.

   Trade-offs:
   - Completes the Alternative-style nondeterminism story.
   - Aligns with PureScript Run's `Empty` / `Alt` and Heftia's `Choose` /
     `Empty` split.
   - Mostly first-order, so it is likely less architectural risk than Writer
     higher-order semantics.
   - Does not exercise the scoped architecture that still needs public API
     polish.

3. Implement Input, Output, Fresh, Log, or KVStore next.

   Trade-offs:
   - Useful examples for custom first-order effect authoring.
   - Many can lower into existing State/Writer-like machinery.
   - Less urgent than Writer and NonDet because they do not expose a major
     current semantic gap in the shipped standard effects.

4. Implement Coroutine, CC, Shift, Provider, Unlift, Stream, Subprocess, Timer,
   or Parallel next.

   Trade-offs:
   - Valuable long term.
   - Much more architecture-sensitive, especially around continuation exposure,
     async/IO, and runtime integration.
   - Premature before the public handler surface and guide are settled.

Recommendation: do Writer higher-order semantics first after Phase 5 cleanup,
then NonDet Empty, then smaller first-order ports.

Reasoning: Writer is the best next architecture test because it depends on
scoped action handling and already has a first-order base effect in the library.
It will reveal whether the H2 boundary and standard handler APIs generalize
beyond Catch/Local/Bracket/Span. NonDet Empty should follow because it completes
the shipped Choose story. Runtime-heavy effects should wait until the library has
a clear async/IO target-monad policy.

### Module organization

Problem addressed: Finding 7.

Options:

1. Leave the current file structure alone.

   Trade-offs:
   - Avoids churn.
   - Keeps each wrapper/effect in one obvious file.
   - Review cost remains high, and future semantic changes will continue to
     touch very large files.

2. Split modules by stable concern after the handler rename.

   Trade-offs:
   - Gives each wrapper a clearer internal structure.
   - Can separate public methods, private raw-step helpers, boundary/carrier
     protocols, smart constructors, and tests.
   - Requires careful re-export boundaries.
   - Should be done after naming stabilizes to avoid compounding move churn with
     API rename churn.

3. Abstract aggressively across all six wrappers.

   Trade-offs:
   - Could reduce duplicated method bodies.
   - Likely to reintroduce HRTB, Send/Sync, and associated-type projection
     complexity.
   - Risks hiding real semantic differences between Box, Rc, Arc, Erased, and
     Explicit families.

Recommendation: choose option 2 after Phase 5 step 5.2, and avoid option 3
unless a repeated helper can be proved locally. Phase 5 step 5.3 has started
with child test modules plus default `Run`'s private representation/raw-dispatch
module, `RunExplicit` and `ArcRunExplicit` typed boundary/carrier modules, and
`ArcRun`'s raw scoped handler/continuation module, and `RcRunExplicit`'s typed
boundary/carrier module, plus `RcRun`'s raw scoped handler/continuation module,
the six wrapper smart-constructor modules, `interpreter/first_order.rs`, and
`interpreter/scoped_resume.rs`, `bracket/explicit.rs`, and
`standard_scoped_handlers/span/carrier.rs`, and
`standard_scoped_handlers/local/carrier.rs`, and
`standard_scoped_handlers/ref_local/carrier.rs`, and
`standard_scoped_handlers/catch/carrier.rs`;
continue option 2 through the finite standard-handler carrier split sequence,
then stop after the raw-replacer checkpoint unless it identifies complete named
raw-replacer concerns worth extracting.

Reasoning: the large files are a real maintainability problem, but the right
split is organizational, not a cross-wrapper abstraction push. The wrapper
families differ for real reasons. New-style child modules can reduce review
size while preserving those differences explicitly.

### Custom-effect authoring

Problem addressed: Finding 8.

Options:

1. Keep custom effects fully manual.

   Trade-offs:
   - Maximum transparency.
   - No macro design risk.
   - Too verbose for eventual user-facing documentation.

2. Revive `define_effect!` immediately.

   Trade-offs:
   - Reduces boilerplate quickly.
   - Risks designing the macro around test-only examples before the public
     handler vocabulary and guide are stable.
   - Could lock in a shape that does not fit Writer, Empty, or later custom
     scoped effects.

3. Write the public guide/manual pattern first, then design `define_effect!`
   from repeated guide code.

   Trade-offs:
   - Slower than immediate macro work.
   - Produces a macro from real documented repetition.
   - Keeps the manual path understandable and testable.

Recommendation: choose option 3.

Reasoning: macro design should follow a stable manual pattern. The current
custom-effect tests are enough to show the problem exists, but not enough to
prove the best macro input syntax or generated surface. Rename and document the
handler API first, then use the guide-writing process to decide what
`define_effect!` should remove.

## Recommended Near-term Order

1. Phase 5 step 5.2 shipped: public standard scoped handler vocabulary now uses
   `standard_scoped_handlers` and `*Handler` / `*_handler` names.

2. Phase 5 step 5.2 shipped: exports and module docs now use the named
   `standard_scoped_handlers` module rather than a partial top-level re-export
   set.

3. Attempt standard handler constructor inference polish.

4. Improve missing-handler docs/examples and add diagnostics experiments only
   if they do not distort the dispatch architecture.

5. Decide whether row-alias helper macros are still needed after the inference
   pass.

6. Revisit Writer `listen` / `censor` as the next scoped-effect family.

7. Revisit NonDet `Empty` and richer Choose handlers.

8. Only then revisit broad custom-effect macro generation and the
   `interpret` -> `handle` method rename.

## Bottom Line

The current effects system is a strong foundation, not a dead end. The recent
complexity is not primarily from choosing the wrong core architecture; it came
from discovering that around-action scoped effects need explicit continuation
boundaries in Rust. The current H2-style private boundary architecture addresses
that correctly.

The next work should make the public API match that foundation. In particular:
rename dispatchers to handlers, make exports consistent, reduce witness
boilerplate carefully, improve diagnostics, and then expand to the next effect
families. Writer higher-order semantics and NonDet Empty are the most valuable
next semantic gaps once the API cleanup lands.
