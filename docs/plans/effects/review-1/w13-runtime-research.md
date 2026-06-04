# W13 Runtime Policy: Reference Research and Refined Options

This is research input for the W13 Runtime Policy Gate in
[`remediation-plan.md`](remediation-plan.md). It surveys how seven effect
libraries handle async, executors, multi-shot continuations, thread
safety, and cancellation, and it refines the gate's options with that
evidence. It does not resolve the gate; the runtime-policy choice remains
the user's.

This is a new, dated artifact. The older per-library docs under
`docs/plans/effects/research/` are line-cited from `decisions.md` and the
reviews and are left untouched; where their conclusions still hold they
are superseded here only for the async-specific lens, read against
current code.

## Reference projects examined

Findings come from reading current source (read-only), commit recorded:

| Project        | Commit  | Lang       | One-line characterization                                                                                   |
| -------------- | ------- | ---------- | ----------------------------------------------------------------------------------------------------------- |
| corophage      | 2276bd8 | Rust       | Coroutine-based (fauxgen on stable), closest lineage to fp-library; has a working stable async interpreter. |
| heftia         | 542963d | Haskell    | The porting target; synchronous Eff interpreter, IO at the boundary, FTCQueue multi-shot.                   |
| purescript-run | abec7c3 | PureScript | The substrate inspiration; Run is sync, async is the target monad (Aff).                                    |
| effing-mad     | e7c1a5f | Rust       | Nightly coroutine-based; non-recursive async runner loop.                                                   |
| reffect        | b1886b3 | Rust       | Nightly coroutine-based; non-recursive async runner via poll_fn.                                            |
| rust-effects   | 702b683 | Rust       | Monad-typeclass library; concrete stable Monad instance for a Future newtype.                               |
| fx-rs          | c2d51c7 | Rust       | Reader/ability effects, do-notation macro; fully synchronous, no async.                                     |

Not examined (optional local sources flagged earlier, not in this pass):
the local `eff` (Haskell delimited-continuation lib) and `switch-resume`
(small Rust experiment).

## The central finding: the blocker is narrower than stated

`findings.md` section 6 frames the async gap as blocked on a
`MonadRec`-over-`Future` strategy, because the interpreter's stack safety
runs through `MonadRec::tail_rec_m` over the target monad, and stable Rust
cannot name a recursive `Future` type. The survey shows that framing is
avoidable: an async interpreter does not have to go through the generic
`tail_rec_m` machinery at all.

Every library that supports async uses the same shape: a single,
non-recursive `async fn` whose body is a `loop` that drives the program's
step function and `.await`s the handler at each effect, rather than a
recursive `async fn interpret(...)`.

- corophage (stable Rust): `asynk::run` is a non-recursive `async fn`
  running the `run!` loop, `.await`ing each handler future
  (lib.rs:109-118). Its own transferability note states fp-library could
  add an async interpreter today by driving the `Free` tree in an
  `async fn` loop that `.await`s async handlers; the coroutine encoding is
  what removes the recursive-type problem for user code, not a
  prerequisite for the async interpreter pattern.
- effing-mad and reffect (nightly): `handle_async` and `future::run` are
  the same non-recursive loop-over-step shape; the agents note this
  "sidesteps the cannot-name-recursive-async-type blocker entirely."
- rust-effects (stable): a `Future` newtype `CFuture<A>` =
  `Shared<BoxFuture<'static, A>>` gets a concrete `Monad` instance whose
  `bind` uses future combinators (`then`), so the type is always
  nameable. This solves the recursive-type problem for `bind`, though it
  ships no `MonadRec`, so it would stack-overflow on deep recursion
  without a trampoline.

The reframing: fp-library's interpreter is already a step-function loop
(peel a `Node`, dispatch, get the next program). The async version is a
concrete `async fn` whose loop peels and dispatches but `.await`s async
handlers. The `.await` suspension points bound the stack, so a generic
`MonadRec`-over-`Future` impl is not required for the interpreter loop
itself. The remaining cost is per-step heap allocation only where a
recursive async tail call must be boxed (`Pin<Box<dyn Future>>`), which
the substrate already accepts elsewhere for stack safety. Whether that
direct async driver loop is fully expressible on the dual-row substrate
is the empirical question a bounded spike should settle, but the survey
removes "needs `MonadRec` over `Future`" as a hard precondition.

## Refined options per W13 sub-question

### Async interpreter substrate

The gate's three approaches stand, but the evidence reweights them:

- Direct async interpreter loop (refinement of the gate's "Future-capable
  target-monad path"): write a concrete `async fn` driver that walks the
  `Free`/`Node` tree and `.await`s async handlers, boxing recursive tail
  calls as `Pin<Box<dyn Future>>` where needed. Demonstrated on stable
  Rust by corophage; structurally confirmed by the nightly libraries.
  Smallest footprint, reuses the existing loop, keeps the sync wrappers
  intact. Cost: per-step boxing at the async boundary.
- Future-monad target newtype: a `CFuture`-style newtype over
  `Pin<Box<dyn Future<Output = A> + Send + 'static>>` with `bind` via
  combinators (rust-effects pattern). Gives a nameable async target
  monad, but still needs a stack-safe driver (it has no `MonadRec`), so
  it folds back into the direct-loop approach for recursion.
- Dedicated async substrate / wrapper family: still the fallback only if
  the direct loop hits a concrete dual-row or lifetime wall, because it
  risks the cross-product explosion W2 controls.

Recommendation: a bounded stable-Rust spike of the direct async
interpreter loop over the dual-row substrate, boxing recursive tail
calls. Adopt a dedicated substrate only if the spike documents a concrete
limitation. This is the gate's recommendation, sharpened: the spike is
now known to be feasible in principle (corophage proves the shape on
stable), so the spike is about the dual-row specifics, not about whether
the idea works at all.

### Executor and blocking model

Strong convergence on executor neutrality. corophage, effing-mad,
reffect, and fx-rs all return runtime-agnostic futures with no runtime
dependency in the library (tokio appears only in their dev-dependencies).
heftia keeps its core runtime-agnostic (the base monad is a type
parameter) and confines runtime-specific interpreters to a separate
layer. Only rust-effects pulls in tokio, and even there the `CFuture`
type itself uses `futures`, not tokio.

Recommendation (validated): executor-neutral. The core returns
runtime-agnostic futures; any runtime-specific conveniences live in
optional adapter crates or feature flags. heftia's own two-interpreter
split (a runtime-agnostic algebraic `Parallel` plus a separate
IO-thread `runMachineryIO`) is the model: keep the algebraic machinery
neutral, provide a runtime adapter behind a feature flag.

### Send, Sync, and local futures

corophage offers a clean precedent fp-library already half-has: a
type-level locality marker with two aliases, `Co` (local, future is
`Pin<Box<dyn Future + 'a>>`) and `CoSend` (future is `+ Send`), chosen at
construction. fp-library's existing Box/Rc/Arc wrapper axis already
encodes exactly this distinction (Arc-family for `Send + Sync`).

Recommendation: provide both a local (non-`Send`) and a `Send` async
path, selected by the existing wrapper family (Rc-family local, Arc-family
`Send`), rather than committing the whole library to `Send`. This matches
the substrate fp-library already ships and avoids forcing `Send` onto
synchronous users.

### Multi-shot continuations and Shift / CC

A clear contrast emerged. The nightly coroutine libraries (effing-mad,
reffect) are one-shot: a compiler-generated state machine cannot be
resumed twice (effing-mad simulates branching only by `Clone`-ing the
whole coroutine before each resume; reffect has no replay). corophage is
also unconditionally one-shot (its `Pin<&mut>` coroutine cannot be
cloned). heftia, by contrast, is multi-shot by default: the continuation
is an FTCQueue, a persistent tree of plain closures that `qApp` never
mutates, so it can be applied any number of times.

The decisive point: fp-library's free-monad-of-values is structurally on
heftia's side, not the coroutine libraries' side. A continuation that is
data (a `Free` tail) is replayable whenever the functor and value are
`Clone`, which the Rc/Arc families already provide. The heftia agent's
translation: the pure algebraic portion (FTCQueue, Coroutine, NonDet,
Shift/CC) ports to Rust with `Arc<dyn Fn>` closures and the `ArcFree`
family, which fp-library already heads toward; the two-shot Shift/CC
pattern needs `Arc<dyn Fn>`, not `Box<dyn FnOnce>`.

Recommendation: keep Shift/CC deferred (as the gate says), but record
that the substrate path is concrete and stable-viable: an FTCQueue-style
`Arc<dyn Fn>` continuation over the Arc-family. The genuinely open design
question is answer-type-polymorphic continuation capture (a continuation
whose answer type differs from the program's result type), which neither
the survey nor the current mono-in-`A` interpreter settles; that is the
real content of the deferred Shift/CC spike.

### Cancellation and resource lifecycle

Universal across the Rust libraries: cleanup is RAII/`Drop`-based, with no
async-drop or finalizer concept. corophage, effing-mad, and reffect all
rely on dropping the suspended computation to release resources; cancel is
"return early, drop, unwind." heftia surfaces a semantic tradeoff worth
recording: prompt `bracket` finalization is incompatible with multi-shot
pure machinery, because the release would fire before a later resumption,
so heftia offers prompt finalization only in its IO-thread interpreter,
not its pure resumable one.

Recommendation: adopt `Drop`-based cleanup as the cancellation model
(consistent with the existing Bracket-on-unwind behavior), and document
the multi-shot-vs-prompt-finalization tradeoff explicitly: a program that
both resumes a continuation multiple times and relies on prompt Bracket
release cannot have both; prompt release is available only on
single-shot/IO-adapter paths.

### IO embedding, target-monad lifting, and Unlift

Unlift is the least-transferable capability. heftia's `MonadUnliftIO`
relies on GHC higher-rank closures and the absence of ownership: the
"run" function can be a closure over the effect environment with no
`Send`/lifetime obligation. In Rust, an equivalent unblock function that
crosses task boundaries needs `Send + 'static`, which conflicts with
non-`Send` effects, and the higher-rank closure is hard to express
without `unsafe`.

Recommendation: treat Unlift as the hard, last port; restrict it to the
Arc/`Send` family if attempted, and gate it behind the same runtime
adapter as Concurrent. Do not block the rest of W13 on it.

## Recommendation summary

1. Async substrate: bounded stable-Rust spike of a direct async
   interpreter loop over the dual row (not a `MonadRec`-over-`Future`
   impl). Feasibility of the shape is established; the spike tests the
   dual-row specifics.
2. Executor: executor-neutral; runtime adapters behind feature flags.
3. Send/local: two paths via the existing Rc-family (local) and
   Arc-family (`Send`) wrappers.
4. Multi-shot / Shift/CC: keep deferred; substrate path is `Arc<dyn Fn>`
   FTCQueue continuations over the Arc family; the open design question is
   answer-type-polymorphic capture.
5. Cancellation: `Drop`-based; document the multi-shot vs prompt-Bracket
   tradeoff.
6. Unlift: hardest port; Arc/`Send`-only, behind the runtime adapter;
   non-blocking for the rest.

## What this does not settle

- The empirical feasibility of the direct async loop on the specific
  dual-row substrate (the spike).
- Answer-type-polymorphic continuation capture for Shift/CC.
- The exact runtime-adapter surface (which runtimes, what the feature
  flags expose).
- The local `eff` and `switch-resume` projects were not examined; if
  `switch-resume` is a prior in-house probe at this exact problem it is
  worth a focused read before the spike.
