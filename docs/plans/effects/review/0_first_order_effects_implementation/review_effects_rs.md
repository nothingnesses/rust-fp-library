# Adversarial Review of effects.rs

## Summary

The effects subsystem is a Coyoneda-over-Coproduct freer-monad encoding,
modelled on PureScript Run with a heftia-style "dual row" (first-order
plus scoped) skeleton. The first-order side is largely wired; the
scoped side is reserved at the type level but every interpreter panics
when it sees a `Node::Scoped`. The most important issue is that the
`run_accum` family on all six wrappers is a stub: it accepts an `init`
state, drops it on the floor, and delegates to `interpret`. Counts:
**5 fundamental**, **8 major**, **9 minor** findings. The single most
important issue is the `run_accum` API lie (F1): the function's name
and signature both promise state threading the implementation cannot
deliver in its current encoding.

## Fundamental Flaws

### F1. `run_accum` and `run_accum_rec` ignore the `init` argument across all six wrappers

- **Where:**
  [run.rs:653-666](../../../../../fp-library/src/types/effects/run.rs#L653-L666),
  [run.rs:905-922](../../../../../fp-library/src/types/effects/run.rs#L905-L922),
  [rc_run.rs:810-811](../../../../../fp-library/src/types/effects/rc_run.rs#L810-L811),
  [rc_run.rs:1043](../../../../../fp-library/src/types/effects/rc_run.rs#L1043),
  [arc_run.rs:825-826](../../../../../fp-library/src/types/effects/arc_run.rs#L825-L826),
  [arc_run.rs:1067](../../../../../fp-library/src/types/effects/arc_run.rs#L1067),
  [run_explicit.rs:632-633](../../../../../fp-library/src/types/effects/run_explicit.rs#L632-L633),
  [run_explicit.rs:852](../../../../../fp-library/src/types/effects/run_explicit.rs#L852),
  [rc_run_explicit.rs:885-886](../../../../../fp-library/src/types/effects/rc_run_explicit.rs#L885-L886),
  [rc_run_explicit.rs:1118](../../../../../fp-library/src/types/effects/rc_run_explicit.rs#L1118),
  [arc_run_explicit.rs:975-976](../../../../../fp-library/src/types/effects/arc_run_explicit.rs#L975-L976),
  [arc_run_explicit.rs:1262](../../../../../fp-library/src/types/effects/arc_run_explicit.rs#L1262).
- **What:** Every `run_accum<St>(handlers, init: St)` body is literally
  `let _ = init; self.interpret(handlers)`. Same for `run_accum_rec`.
  The `init` value is moved in and dropped without being read. The
  doctests show users threading state via a captured
  `Rc<RefCell<i32>>` and then asserting on `*counter.borrow()`,
  meaning the documented usage pattern silently bypasses the
  parameter the function accepts.
- **Why fundamental:** The mono-in-A interpreter returns `A`, not
  `(S, A)` or `M<(S, A)>`, so there is no "place" in the result for
  the threaded state to come out. Plan
  [decisions row at line 1207](../../plan.md#L1207)
  defers the principled fix (StateT-as-target-monad) to Phase 6+.
  Until that ships, any honest `run_accum` requires a different
  return shape, which means a different handler-dispatch trait, which
  means redesigning the interpreter family. The current state of the
  API violates rubric Section 6.1 #4 (handlers receive a callable
  continuation that the system threads cleanly): in this design, the
  user must pre-thread state through closures themselves, then the
  function pretends it threaded the state for them. Per the plan's
  own [Success criteria](../../plan.md#L2574),
  "Reader, State, Except, Writer, Choose ship as standard first-order
  effects with smart constructors." The current `run_accum` cannot
  legibly run any of those effects; it can only run programs whose
  effects have already been threaded by hand. This is a
  rubric Section 2 violation as well: "decoupled interpretation"
  fails when the canonical State handler cannot be written without
  side-channels.
- **Worked example:**
  ```rust
  // Suggests state will be threaded; doesn't.
  let prog: Run<StateRow, _, i32> = Run::get::<_>().bind(|n| Run::put::<i32, _>(n + 1).bind(move |_| Run::pure(n + 1)));
  let result = prog.run_accum(state_handlers, 10_i32);
  // The `10` is dropped. No matter what `state_handlers` does, the
  // initial state cannot have been `10` unless `state_handlers`
  // *also* knows about a separate Rc<RefCell> captured in scope.
  ```

### F2. Mono-in-A handler dispatch has no callable continuation

- **Where:**
  [interpreter.rs:106-145](../../../../../fp-library/src/types/effects/interpreter.rs#L106-L145)
  defines `DispatchHandlers::dispatch(&self, layer: Layer) -> NextProgram`.
  Each handler's closure is
  `Fn(<EBrand as Kind>::Of<'a, NextProgram>) -> NextProgram`
  ([interpreter.rs:211](../../../../../fp-library/src/types/effects/interpreter.rs#L211)).
- **What:** A handler clause receives the lowered effect layer (e.g.,
  `State<'_, P, S, NextProgram>` containing a `dyn Fn(S) -> NextProgram`),
  not a continuation `k : x -> Result`. The "continuation" is whatever
  function each effect type embeds in its constructor. There is no
  uniform `k` argument; the system has the lookalike of an algebraic
  effect API but the binding to "the rest of the program" is
  per-effect-type rather than per-handler-clause.
- **Why fundamental:** This is the load-bearing distinction between a
  free-monad library that happens to dispatch on row variants and a
  true algebraic effect handler system. Rubric Section 3 lists
  "continuation" as core vocabulary: "Inside a handler clause, the
  suspended rest of the program, exposed as a callable function
  `k : x -> Result`". This implementation does not expose that. A
  handler that wants to call its continuation twice (multi-shot) has
  to first know what kind of effect type it is dispatching on, then
  reach into the variant's stored function, then call it twice. There
  is no uniform interface for "resume". The plan acknowledges this at
  [interpreter.rs:14-34](../../../../../fp-library/src/types/effects/interpreter.rs#L14-L34):
  "Users who genuinely need rank-2 polymorphism over A ... reach for
  `NaturalTransformation` directly, consumed by `Free::fold_free`."
  But `fold_free` is the underlying free-monad fold; it does not
  expose a `k` to handler clauses either. Fixing this requires
  delimited continuations or a different encoding altogether (rubric
  Section 5.4). On stable Rust without GHC's
  `prompt#`/`control0#`, this is "redesign-only".
- **Worked example:** Implementing `Choose` (multi-shot
  nondeterminism, rubric Section 8.2) requires the handler clause for
  `Choose` to call the continuation once per choice and concat the
  results. In this encoding the per-effect closure looks like
  `dyn Fn(Bool) -> NextProgram`. To run nondet you would have to call
  the inner `dyn Fn` twice (with `true`, `false`), then _combine the
  two `NextProgram`s into one return value_. That combination is the
  responsibility of the handler clause's return type
  (`NextProgram`), but `NextProgram` is just `Run<R, S, A>` --
  there is no `Vec<A>` shape, no place to deliver multiple results.
  The user must pre-arrange the result type to be a list and have
  the handler concat. This is doable, but the structural support for
  it is absent: there is no `resume(x)` primitive of the kind rubric
  Section 8.2's example demands.

### F3. Scoped effects are reserved structurally but unimplemented and panic at runtime

- **Where:** Every `interpret`, `run`, `run_accum`, `interpret_with`,
  `interpret_rec`, `run_rec`, `run_accum_rec`, and `extract` body
  contains a
  ```rust
  Err(Node::Scoped(_)) => unreachable!(
      "Phase 3 first-order interpreter received a scoped layer; scoped effects ship in Phase 4"
  )
  ```
  branch. Examples:
  [run.rs:512-517](../../../../../fp-library/src/types/effects/run.rs#L512-L517),
  [run.rs:1057-1061](../../../../../fp-library/src/types/effects/run.rs#L1057-L1061).
  The scoped row brand `S` is part of every public type signature
  (`Run<R, S, A>`, `RcRun<R, S, A>`, ...), but the only stable
  inhabitant of `S` is `CNilBrand` because no scoped-effect
  constructors have shipped.
- **What:** The dual-row architecture
  ([decisions row at line 1144](../../plan.md#L1144))
  is structurally present (`Node<First, Scoped>` enum,
  `NodeBrand<R, S>` brand, `Functor`/`SendFunctor`/`WrapDrop`/`RefFunctor`
  routing), but the entire
  [`Phase 4: Scoped effects (heftia dual row)`](../../plan.md#L2096)
  section is undelivered, and the runtime falls back to `unreachable!`.
- **Why fundamental:** Rubric Section 6.2 #8 lists "Scoped operations"
  as a higher-order capability the plan claims to support
  ([Success criteria line 2614](../../plan.md#L2614)
  promises `Catch<'a, E>`, `Local<'a, E>`, `Bracket<'a, A, B>`,
  `Span<'a, Tag>`). This rates as fundamental because (i) the plan
  claims to support it; (ii) the constraint that forces the
  `unreachable!` is structural -- the mono-in-A handler list aligned
  cell-for-cell with the row brand chain has no way to recurse on
  scoped operations whose argument is itself a `Run<R, S, A>` (the
  argument is not a "value" in the algebraic-effect sense; it is a
  computation in the same row). Fixing it requires either a
  hefty-algebra elaborator or a separate scoped-handler trait that
  takes a `Run<R, S, A>` argument and yields a `Run<R, S, A>`
  result. The current `DispatchHandlers` trait cannot host that
  shape because its closure is `Fn(EBrand::Of<NextProgram>) -> NextProgram`,
  not `Fn(Run<R, S, A>) -> Run<R, S, A>`. So the design as written
  precludes scoped operations until a parallel infrastructure is
  built. Plan acknowledges Phase 4 is the work; rate this as
  fundamental because the _current_ claim that the encoding "supports"
  scoped effects is unbacked.
- **Worked example:**
  ```rust
  // Hypothetical Phase-4 program calling `catch`:
  let prog: Run<R, ScopedRow, i32> = catch(throwing_subprogram, |_e| recovery);
  prog.interpret(handlers!{ ... });  // -> panics on Node::Scoped
  ```
  No constructor of `ScopedRow` exists today, so the type system
  cannot help; users either avoid the scoped row entirely (forcing
  `S = CNilBrand`) or trip the `unreachable!` at runtime when Phase 4
  delivery starts.

### F4. The "single-shot vs multi-shot" property promised per wrapper is not enforced at the effect-instance level

- **Where:**
  [state.rs:60-73](../../../../../fp-library/src/types/effects/state.rs#L60-L73)
  declares `Get(P::Of<'a, dyn 'a + Fn(S) -> A>)` and
  `Put(S, P::Of<'a, dyn 'a + Fn(()) -> A>)`. The continuation closure
  inside each variant is `dyn Fn` (callable many times), not
  `dyn FnOnce`. This is the same shape on every wrapper (Run,
  RcRun, ArcRun, ...).
  [state.rs:24-29](../../../../../fp-library/src/types/effects/state.rs#L24-L29)
  spells this out: "even single-shot wrappers (Run / RunExplicit) use
  Rc-wrapped continuations rather than Box<dyn FnOnce>".
- **What:** Plan
  [Success criteria line 2598](../../plan.md#L2598)
  claims "Each of the six Free variants supports its promised property
  (single-shot vs. multi-shot, ...)." For State specifically, the
  per-effect continuation is `Fn` regardless of which wrapper holds
  it, so a handler's call to that continuation is multi-shot at the
  effect-operation level even when the surrounding wrapper is
  declared single-shot. The "single-shot" guarantee on `Run` /
  `RunExplicit` only applies to consuming the outer Free spine, not
  to the per-effect closure stored inside `Coyoneda<E, NextProgram>`.
- **Why fundamental:** The marketing of six wrappers is structured
  around a binary single-shot vs multi-shot property. State (and any
  effect with a stored closure continuation) silently deviates from
  that taxonomy. Fixing it requires either (a) parameterising every
  effect's continuation slot by the wrapper's own
  Fn/FnOnce/FnMut policy, which fans out the effect type by 6x; or
  (b) abandoning the property as a global wrapper claim and
  documenting it per-effect. The plan itself
  ([state.rs:24-29](../../../../../fp-library/src/types/effects/state.rs#L24-L29))
  defends this divergence: "The single Rc allocation per Get/Put is
  a small cost compared to the design simplification of one effect
  type per operation across all wrappers." That defence concedes the
  property has been weakened; the success criterion has not been
  weakened to match. This is rubric Section 12 #2 (pragmatic
  decision framework: "Do you need multi-shot continuations?").
  Users who pick `Run` thinking it is single-shot get multi-shot
  semantics for State for free, which is a footgun for resource
  safety (rubric Section 6.2 #9: "What happens to acquired resources
  when a handler ... duplicates a continuation?").
- **Worked example:**
  ```rust
  // On `Run` (declared single-shot), a handler can still call k twice:
  let handler = |state_op: State<'_, RcBrand, i32, NextProgram>| match state_op {
      State::Get(k) => {
          let p1 = (*k)(0);     // call once
          let p2 = (*k)(0);     // call again -- legal, k is `Fn`
          // ...somehow combine p1 and p2...
      }
      State::Put(_, k) => (*k)(()),
  };
  ```

### F5. The natural-transformation handler shape that the rubric demands cannot be expressed by Rust closures, and the chosen workaround removes the property

- **Where:**
  [interpreter.rs:14-34](../../../../../fp-library/src/types/effects/interpreter.rs#L14-L34)
  documents the choice: "PureScript Run's `interpret` has the
  signature `(VariantF r ~> m) -> Run r a -> m a` (a true rank-2
  natural transformation), but its implementation literally aliases
  `run` whose signature is `... -> Run r a -> m a` -- a step
  function whose handler is mono-in-`a`. The Rust port adopts the
  mono-in-`a` form directly so handler closures fit Rust's
  non-generic-closure constraint."
  [run.rs:497-519](../../../../../fp-library/src/types/effects/run.rs#L497-L519)
  shows the resulting signature: handlers are bound by `for<'h>`
  over the lifetime only, never over `A`. The user-level workaround
  is "reach for `NaturalTransformation` directly, consumed by
  `Free::fold_free`" -- i.e., bypass the handler-list entirely.
- **What:** A natural transformation is rank-2 polymorphic over the
  result type `A`: one transformation handles every program
  regardless of `A`. The mono-in-`A` form pins `NextProgram` (and
  therefore `A`) at the call site, so writing a generic library of
  reusable handlers parameterised over arbitrary programs is harder.
  Each handler is bound to one program's `A`.
- **Why fundamental:** Rubric Section 6.1 #3 ("Handlers as ordinary
  values") and #6 ("Effect polymorphism") both want handlers to be
  reusable across programs of different result types. The plan
  acknowledges the gap and routes users to a different API
  (`fold_free` plus a `NaturalTransformation` value), which means
  the headline `interpret` / `run` / `run_accum` family is not a
  natural-transformation API; it is a per-program-type interpreter
  family. A user writing `fn run_state<R, A, ...>(prog: Run<R, _, A>) -> Run<RMinus, _, A>`
  has to repeat the bound for every `A`. The
  `interpret_with::<EBrand, Idx, RMinusE>` form mitigates this for
  _one_ effect at a time but reproduces the same `A`-pinning
  ([run.rs:1003-1063](../../../../../fp-library/src/types/effects/run.rs#L1003-L1063)).
  Fixing it would require GATs over closure types or an entirely
  different handler representation; on stable Rust today this is
  redesign-only.
- **Worked example:** A library author wants to ship
  `fn pure_state_runner() -> impl SomeHandlerTrait`. In this
  encoding, the trait would have to be parameterised over the
  caller's `A`, so it cannot be returned as a single value. The
  PureScript Run shape `runState :: Int -> Run (state :: STATE Int | r) ~> Run r`
  has no clean Rust analogue.

## Major Issues

### M1. `interpret`'s "all handlers at once" form forces handler-list ordering by lexical sort, not by user intent

- **Where:**
  [handlers.rs:50-61](../../../../../fp-library/src/types/effects/handlers.rs#L50-L61):
  "Users assembling a list to match a row built by `effects!` (which
  sorts brands lexically) should call `.on()` in reverse-lexical
  order...". The
  `handlers!{...}` macro internally sorts by brand identifier.
- **What:** For non-commuting effects (e.g., `NonDet x Except`), the
  result depends on which is interpreted first
  ([rubric Section 4.3](../algebraic_effects.md#43-where-they-differ)). The all-at-once
  `interpret` form does not expose ordering -- the macro picks for
  the user. The pipelined `interpret_with::<EBrand>` form is the only
  way to control ordering, but as F5 notes, that form pins `A`.
- **Why major (not fundamental):** Fixable by either (a) requiring
  users to pick `interpret_with` always for non-commuting cases (the
  plan's stance per
  [run.rs:933-935](../../../../../fp-library/src/types/effects/run.rs#L933-L935)),
  or (b) lifting the lexical-sort constraint so users specify the
  order. Documentation already warns about this; not a soundness
  bug.

### M2. Stack safety only via `interpret_rec`/`tail_rec_m`; the bare `interpret` is host-stack recursive

- **Where:**
  [run.rs:507-519](../../../../../fp-library/src/types/effects/run.rs#L507-L519):
  the `interpret` body is a `loop` over `peel`, which is
  iterative _across program layers_ but the per-layer `Functor::map`
  inside `interpret_with` recurses via host stack (see
  [run.rs:1032-1054](../../../../../fp-library/src/types/effects/run.rs#L1032-L1054)
  for the recursive `<R as Functor>::map`).
- **What:** Programs with deep chains of eager-recursing effects can
  blow the host stack on `interpret_with`. Plan acknowledges this at
  [run.rs:947-954](../../../../../fp-library/src/types/effects/run.rs#L947-L954).
  Users must remember to use the `_rec` family (which requires a
  `MonadRec` target).
- **Why major:** Stack-safety pitfall but not a soundness hole; users
  have a documented escape hatch.

### M3. Handler closures must be `Clone + 'static` (and `Send + Sync` on Arc) for `interpret_with`

- **Where:**
  [run.rs:1005-1009](../../../../../fp-library/src/types/effects/run.rs#L1005-L1009),
  same shape across all six wrappers.
- **What:** Each `Functor::map` over the recursive narrowing of a
  layer's content clones the handler. For typical Identity-shaped
  effects that is one clone per layer; for higher-arity effects this
  multiplies. The closure must close over only `Clone` data, which
  rules out captures of unique resources (e.g., a `BufWriter`
  acquired in scope).
- **Why major:** Inconvenient and limits expressivity (rubric Section
  6.3 #15: handler combinators); not unsound.

### M4. State-effect `Functor` allocates a fresh `Rc`/`Arc` per `map` call

- **Where:**
  [state.rs:130-139](../../../../../fp-library/src/types/effects/state.rs#L130-L139):
  ```rust
  fn map<'a, A: 'a, B: 'a>(f: ..., fa: ...) -> ... {
      match fa {
          State::Get(k) => State::Get(<P as ToDynCloneFn>::new(move |s: S| f((*k)(s)))),
          State::Put(s, k) => State::Put(s, <P as ToDynCloneFn>::new(move |u: ()| f((*k)(u)))),
      }
  }
  ```
- **What:** Each `Functor::map` rebuilds the continuation by
  composing `f` over the existing `Rc<dyn Fn>`. The new `Rc` is a
  fresh allocation; the old `Rc` is dropped after composition. So a
  long bind-chain over a State program does N allocations where one
  CatList-style fusion would do one. Per rubric Section 5.1's
  left-bind blowup discussion, this is the unmitigated form.
- **Why major:** The Erased family claims O(1) bind via `dyn Any`
  erasure plus CatList
  ([plan line 1133-1140](../../plan.md#L1133-L1140)).
  But the per-effect map composition above is not amortised by
  CatList; it runs once per `Functor::map` invocation by the
  interpreter on the State layer. Fixable by Coyoneda fusion
  (precisely what `Coyoneda` exists to do), but the present code
  composes inside the State `Functor` directly rather than letting
  Coyoneda fuse externally.

### M5. SendFunctor for `StateBrand` is deferred; multi-thread State is unimplemented

- **Where:**
  [state.rs:142-146](../../../../../fp-library/src/types/effects/state.rs#L142-L146):
  "SendFunctor impl deferred to step 5a.3: the bound
  `<P as RefCountedPointer>::Of<'_, dyn 'a + Fn(S) -> A>: Send + Sync`
  must be expressed per-`A` for `send_map`'s generic `A` parameter,
  which requires HRTB-over-types support not available on stable
  Rust." Plan
  [active blocker line 599+](../../plan.md#L599)
  documents this.
- **What:** `ArcRun::get` and `ArcRun::put` cannot be implemented in
  the current Phase 3 step 5a milestone. The Send-side State + Arc
  family is blocked behind an unsolved type-system limitation.
- **Why major:** Plan-acknowledged; tracked. But the plan
  [Success criteria line 2610](../../plan.md#L2610)
  promises State on every wrapper, so until this resolves the criteria
  cannot be met. Not fundamental because the row-encoding itself is
  fine; the obstruction is in HRTB-over-types for the Send variant.

### M6. No async / `IO` story; `Future`-as-`MonadRec` is deferred to "Phase 6+"

- **Where:**
  [plan line 1308-1312](../../plan.md#L1308-L1312):
  "Section 9.4 commits to `Thunk` (v1) and `Future` (Phase 3) as
  `MonadRec` targets..." but the present code only ships
  `ThunkBrand`, `OptionBrand`, `ResultBrand` as `interpret_rec`
  targets (per [run.rs:892-901](../../../../../fp-library/src/types/effects/run.rs#L892-L901)
  doctest example).
- **What:** Real-world IO inside handlers is not addressed. Async
  cancellation, runtimes, structured concurrency are all absent.
- **Why major:** Rubric Section 6.2 #10 lists this as a higher-order
  capability; rubric Section 6.3 #13 lists "interoperation with the
  host language" as pragmatic. Plan acknowledges the gap.

### M7. `bind` is `FnOnce` while handler closures are `Fn`; conversions across the API are awkward

- **Where:**
  [run.rs:310-315](../../../../../fp-library/src/types/effects/run.rs#L310-L315):
  `pub fn bind<B: 'static>(self, f: impl FnOnce(A) -> Run<R, S, B> + 'static)`.
  vs handler shape `Fn(...) -> ...` at
  [interpreter.rs:211](../../../../../fp-library/src/types/effects/interpreter.rs#L211).
- **What:** Mixed lifetime/`Fn`-trait bounds across the API. A user
  who writes a helper that takes a closure and forwards it to both a
  `bind` and a `dispatch` cannot pick a single trait bound; they
  must write two helpers or a wrapper trait.
- **Why major:** Inconvenience, not unsoundness.

### M8. The "scoped" row in every public type signature is currently always `CNilBrand`; users cannot inhabit it

- **Where:** Every wrapper's type is `Run<R, S, A>` where the only
  ScopedRow with constructors today is `CNilBrand`. The `Node::Scoped`
  arm in the interpreters traps anything else
  ([run.rs:512-516](../../../../../fp-library/src/types/effects/run.rs#L512-L516)).
- **What:** The dual-row API surface is paid for in every type
  parameter, every where-clause, every doctest, but no user can
  populate the second slot until Phase 4. Users today write
  `S = CNilBrand` everywhere; the parameter is currently zero-cost
  noise.
- **Why major:** Cosmetic for now, semantically meaningful when
  Phase 4 lands. The risk is that the structural choice of
  `Node<First, Scoped>` constrains Phase 4's options; if the chosen
  Phase 4 elaboration needs more than a "two-arm enum", the schema
  will need to change.

## Minor Issues

- [run.rs:493-496](../../../../../fp-library/src/types/effects/run.rs#L493-L496):
  the `clippy::unreachable` lint is suppressed with the comment
  "Reaching the Scoped arm would indicate a wrapper-API logic
  error rather than user error", but no static type-system
  obstruction prevents users from constructing a `Node::Scoped`
  via `send` on a non-`CNilBrand` `S` once Phase 4 ships, and the
  panic message is the user-visible failure mode.
- [interpreter.rs:189-194](../../../../../fp-library/src/types/effects/interpreter.rs#L189-L194):
  the `CNil` base-case's `match layer {}` is sound (CNil is
  uninhabited), but the trait method takes `&self` on `HandlersNil`,
  forcing every caller to materialise an unused trivial value.
- [handlers.rs:75-81](../../../../../fp-library/src/types/effects/handlers.rs#L75-L81):
  `Handler<E, F>` derives `Clone, Copy`, but the closures it wraps
  are typically not `Copy`, so the `Copy` derive is mostly cosmetic.
- [handlers.rs:140-180](../../../../../fp-library/src/types/effects/handlers.rs#L140-L180):
  the builder's lexical-order requirement is documented in module
  docs but unenforced. A user calling `.on()` in the wrong order
  produces a list whose head does not match the row chain head; the
  failure surface is a confusing trait-bound error far from the
  builder call.
- [run.rs:824-839](../../../../../fp-library/src/types/effects/run.rs#L824-L839):
  `run_rec` is a literal alias for `interpret_rec`; the parallel
  alias at `run` for `interpret` is also a literal alias. The
  duplication exists for PureScript-cross-reference convenience and
  inflates the documentation surface.
- [node.rs:65-74](../../../../../fp-library/src/types/effects/node.rs#L65-L74):
  `Node<'a, R, S, A>` requires `R: Kind + 'static`, `S: Kind + 'static`
  but holds a `'a` lifetime on the payload; the inner Coproduct
  payload's lifetime can outlive `'static` constraints elsewhere,
  which has caused HRTB-poisoning workarounds (the
  [arc_run.rs unwrap_first / lift_node helpers](../../../../../fp-library/src/types/effects/arc_run.rs)
  documented in
  [plan line 152-157](../../plan.md#L152-L157)).
- [state.rs:1-30](../../../../../fp-library/src/types/effects/state.rs#L1-L30):
  module docs reference "Phase 3 step 5a.3" / "Phase 5 step 4" by
  identifier; the test/comment guidance in CLAUDE.md memory says
  "All test file comments and annotations must be self-contained;
  never reference external documents, plan phases, or finding IDs."
  Several module docs across the effects subsystem lean on plan
  phase names that will rot.
- [member.rs:81-139](../../../../../fp-library/src/types/effects/member.rs#L81-L139):
  `Member<E, Idx>` is a thin facade over `CoprodInjector` and
  `CoprodUninjector` with a blanket impl. The trait adds essentially
  nothing over the underlying frunk traits except the `Remainder`
  associated type rebound; the documented motivation is "PureScript
  parity". The cost is one more name in the namespace.
- [interpreter.rs:329-388](../../../../../fp-library/src/types/effects/interpreter.rs#L329-L388):
  the three `DispatchHandlers` impls (Coyoneda, RcCoyoneda,
  ArcCoyoneda) are near-duplicates differing only in the lower
  call (`lower` vs `lower_ref`) and the `Send + Sync` bounds. Could
  be unified via a small trait but is not.

## Plan vs. Implementation Drift

| Plan claim                                                                                                                                                                                                                                                                                                                   | Implementation reality                                                                                                                                                            |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [Plan line 2611](../../plan.md#L2611): "Reader, State, Except, Writer, Choose ship as standard first-order effects with smart constructors."                                                                                                                                                                                 | Only `State` exists today, and only `Run::get`/`Run::put` are wired. `Reader`, `Except`, `Writer`, `Choose` are unwritten.                                                        |
| [Plan line 2598](../../plan.md#L2598): "Each of the six Free variants supports its promised property (single-shot vs. multi-shot, ...)."                                                                                                                                                                                     | The State effect's continuation is `dyn Fn` on every wrapper, including the single-shot ones (F4).                                                                                |
| Plan run_accum docstring at [run.rs:580-600](../../../../../fp-library/src/types/effects/run.rs#L580-L600): "Each handler receives the current state by mutable reference inside the closures..."                                                                                                                            | Implementation drops `init` and delegates to `interpret`; the comment about "mutable reference inside the closures" describes user code, not library code (F1).                   |
| [Plan line 1131](../../plan.md#L1131): "ship both interpreter families, mirroring PureScript: `interpret`/`run`/`runAccum` (assume target stack-safe) and `interpretRec`/`runRec`/`runAccumRec`".                                                                                                                            | All four `*_accum*` methods are stubs (F1). Half of the promised pair is name-only.                                                                                               |
| [Plan line 1144](../../plan.md#L1144): "Heftia dual-row for scoped effects".                                                                                                                                                                                                                                                 | Dual row is structural; scoped row never populated; interpreters panic on `Node::Scoped` (F3).                                                                                    |
| [Plan line 2614](../../plan.md#L2614): "Catch<'a, E> and Span<'a, Tag> ship as Val-only scoped-effect constructors..."                                                                                                                                                                                                       | None ship; Phase 4 not started. Plan acknowledges this.                                                                                                                           |
| Interpreter docstring at [interpreter.rs:14-34](../../../../../fp-library/src/types/effects/interpreter.rs#L14-L34) describes mono-in-A as a deliberate downgrade from rank-2 NT. The plan never names "rank-2 NT" as a Success criterion, but the rubric (Section 6.1 #3, #4, #6) treats handlers-as-NT as core capability. | Drift is between the rubric and the implementation, not the plan. The plan's choice is internally consistent; the rubric reads it as a real expressiveness loss (F5).             |
| [Plan line 1207](../../plan.md#L1207): "Phase 3 step 4 `run_accum_rec` continues closure-capture state threading (parity with step 2's `run_accum`); state-via-`StateT s m` deferred to Phase 6+."                                                                                                                           | Implementation matches the plan, but the plan's choice means the named function `run_accum` is functionally identical to `interpret` and the `init` parameter is misleading (F1). |
| [Plan line 596 onward (active blockers)](../../plan.md#L596): SendFunctor over State is HRTB-over-types blocked.                                                                                                                                                                                                             | Implementation matches: state.rs has the SendFunctor `// deferred` comment (M5).                                                                                                  |

## Comparison to Existing Designs

This is closest to **PureScript Run** plus **heftia**. The first-order
side mirrors `purescript-run`'s Coyoneda-over-VariantF over Free; the
scoped side reserves a heftia-style second row at the type level
without yet shipping the elaborator. Concretely:

- **Inherits from PureScript Run / freer-simple / polysemy:** the
  per-operation allocation cost (Coyoneda + Coproduct + Free::wrap),
  the left-bind blowup risk in the absence of a CatList queue, the
  open-union row encoding (frunk Coproduct here, type-level lists in
  Haskell), the lexical-sort macro for canonical row construction.
- **Inherits from heftia:** the dual-row structural split, the
  intent of a separate scoped-handler trait. Inherits the _promise_
  of higher-order effects without inheriting the _delivery_.
- **Diverges from PureScript Run:** PureScript's `interpret` is
  rank-2 (`VariantF r ~> m`) at the surface even though its
  implementation aliases the mono-in-`a` `run`; Rust closures cannot
  carry that rank-2 quantification, so the surface is mono-in-`a`
  here too. This loses the "write a handler once, reuse across
  programs of any result type" property at the headline API level
  (F5).
- **Diverges from heftia / hefty algebras:** heftia ships an
  elaborator from higher-order operations to first-order ones; this
  port has no elaborator, just a placeholder enum arm and a panic.
- **Novel issue specific to this port:** F1 (the `run_accum` stub)
  is not a feature of the libraries it copies. PureScript Run's
  `runState s = case _ of Get k -> Tuple s (k s); Put s' k -> Tuple s' (k unit)`
  threads the state internally and returns `Tuple s a`. The Rust
  port's `run_accum` claims the same shape but cannot deliver it
  because the interpreter's return type is `A`, not `(S, A)` or
  `M<(S, A)>`. The port has chosen the function name without the
  capability behind it.
- **Novel issue specific to this port:** F4 (per-effect continuations
  always behind `Rc`/`Arc` even on single-shot wrappers) is a
  consequence of "one effect type for all six wrappers" -- a
  simplification that is not present in PureScript Run (which has
  one wrapper).
- **Strengths inherited and not lost:** open effect rows; effect
  signatures as first-class user-defined types; effects are
  reinterpretable by swapping handler-list cells (within the
  mono-in-A constraint); no `unsafe`; no `mem::transmute`.

The honest summary: this is a faithful Rust port of PureScript Run's
_shape_, with the same expressiveness ceiling and a measurable
expressiveness loss in handler polymorphism. The dual-row scoped-effect
hook is reserved for Phase 4 work that has not started; until it does,
the second row is a tax on every signature with no payoff. The most
load-bearing engineering debt is the `run_accum` family, which
documents and accepts a state argument it cannot use, and the
`unreachable!` panic on every `Node::Scoped` arm, which converts a
type-level promise into a runtime trap.
