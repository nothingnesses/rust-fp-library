# Deep dive: coroutines as a substrate for first-class programs

**Status:** complete
**Last updated:** 2026-04-24

## 1. Purpose and scope

The port-plan's section 4.4 couples first-class programs with the Free monad
family as a single architectural commitment. This dive asks: which properties
of the Free-family actually require an AST data structure, and which survive
under a coroutine-only substrate?

Answering this question is critical: if every property survives, the plan's
four-variant Free family may shrink or become one of two alternate choices
(Free vs. native coroutine). If any property requires AST inspection, Free
remains mandatory.

Scope: test five concrete Free-family properties against three Rust
coroutine-based effect libraries: corophage (stable, fauxgen-based async),
reffect (nightly, native Coroutine trait), and effing-mad (nightly, native
Coroutine trait). For each property, define it in PureScript-Run terms,
examine the Rust crate evidence, and deliver a verdict. No prototype; only
analysis of existing code.

What is NOT evaluated: control flow expressivity (multi-prompt delimited
continuations); compile-time row optimizations; runtime performance
trade-offs; error handling ergonomics.

## 2. Detailed mechanism

### 2.1 Coroutine-plus-wrapper pattern across three Rust crates

All three Rust crates wrap a native or simulated coroutine in a handler
pipeline, deferring execution until all handlers are attached. The pattern is:

1. **Suspend/resume state machine:** The computation yields an effect (a value
   of the effect enum) and pauses. The caller (handler layer) inspects the
   effect, runs application code, and resumes the computation with a result.
2. **Single-shot FnOnce semantics:** The handler consumes the resume value in
   a closure (`impl FnMut` or `impl FnOnce`). Once the handler fires, the
   continuation is consumed; re-invocation is not possible without re-running
   the entire coroutine from the start.
3. **Composition via nesting:** Multiple handlers are stacked as nested
   coroutines, each wrapping the previous. A yield from the inner coroutine
   is caught by the outermost unhandled effect layer.

**Corophage** (stable Rust via fauxgen):

- Core: `GenericCo<'a, Effs, Return, L>` wrapping a `SyncGenerator` (fauxgen
  abstraction; see `src/coroutine.rs:52-58`). The generator is a pinned async
  closure that calls `Yielder::yield_` to suspend.
- Handlers: `Program::handle` (src/program.rs:133-150) chains FnMut closures.
  Each handler wraps the prior coroutine and intercepts one effect type. No
  composition beyond chaining; once attached, handlers execute in order.
- README explicitly: "handlers are `FnOnce`" (line 62): "each handler can
  resume the computation at most once." This is enforced by the ownership
  model: the handler owns the resume value, making it impossible to invoke
  twice.

**Reffect** (nightly, native Coroutine trait):

- Core: `Coro: Coroutine<Sum<...>>` trait bound. Effectful computations are
  types implementing the standard `core::ops::Coroutine` trait (src/adapter.rs,
  line 27, invokes `pin!(coro).resume(state)`).
- Handlers: `catch0`, `catch1` (src/adapter.rs:59-86) follow the pattern of
  wrapping the coroutine, running a loop that resumes and matches the yielded
  effect, then re-injects the handler's response. Single-shot semantics
  enforced by the coroutine-state machine: once `Complete(ret)` is returned,
  the coroutine is finished.
- No explicit handler composition beyond nesting; handlers are applied
  sequentially in the trait bound.

**Effing-mad** (nightly, native Coroutine trait):

- Core: `#[coroutine]` attribute on generator functions, producing types
  implementing `Coroutine<Resume, Yield, Return>` (src/lib.rs, lines 136-169,
  212-225). The handler function matches on yielded effects and returns
  `ControlFlow::Continue(resume_value)` or `ControlFlow::Break(return_value)`.
- Handlers: `handle` and `handle_group` (src/lib.rs:136-225) are generic
  wrappers that loop over `coro.resume(injection)` / `match yielded`, invoke
  the handler, and either re-inject (Continue) or break (Break). Single-shot:
  each time the handler runs, it consumes its input (the effect instance) and
  returns a ControlFlow; the coroutine does not resume twice with different
  resume values.

### 2.2 Free monad structure in fp-library for comparison

The Free monad (`fp-library/src/types/free.rs`) is a data structure AST with
O(1) bind via a `CatList` tail:

- **Structure** (line 231+): Each `Free<F, A>` is either `Return(TypeErasedValue)`
  or `Suspend(F<Free<F, A>>)` (via `to_view()`, line 613). The suspended
  functor layer holds another Free computation.
- **`resume`** (line 713-720): Returns `Result<A, F<Free<F, A>>>`. If Pure,
  returns the value. If Suspended, returns the effect functor and the
  continuation as a data value. Continuations can be inspected, stored, and
  re-invoked with different results.
- **`fold_free`** (line 774-798): Takes a `NaturalTransformation<F, G>` and
  iteratively walks the Free AST, applying the transformation to each layer.
  The iteration is driven by `G::tail_rec_m`, not recursion.
- **`hoist_free`** (line 847-866): Transforms the functor at each layer using
  a natural transformation, without running the effect. Returns a new Free
  structure with the same shape but functor `G` instead of `F`. Key: the
  hoisted computation is still an AST; it has not been executed.

The critical difference: Free stores the rest-of-program as **data**. A
coroutine stores it as **suspended execution state**. This has five
measurable consequences.

## 3. Property-by-property evaluation

### 3.1 Multi-shot interpretation

**Definition (PureScript-Run context):**

PureScript's `Choose` effect (purescript-run/src/Run/Internal.purs) takes a
single continuation (the rest-of-program) and invokes it twice: once for the
true branch, once for the false branch. A multi-shot handler can run the same
program under different condition branches or with different starting values
(e.g., `amb` in logic programming, or non-deterministic search). The
continuation is a **first-class value** (a Run program); running it does not
consume it.

**Test case:**

```rust
// Pseudo-code, adapted from Choose semantics
type Effects = Effects![Choose, ...];

let program: Program<Effects, u32> = Program::new(|y| async {
    let success = y.yield_(Choose).await;
    if success { y.yield_(Ask("Yes?")).await }
    else { y.yield_(Ask("No?")).await }
});

// Run the same program twice with different Choose outcomes
let result1 = program.handle(|_: Choose| Control::resume(true)).run();
let result2 = program.handle(|_: Choose| Control::resume(false)).run();
// Both succeed; program was executed twice with different inputs.
```

**Evidence from Rust crates:**

_Corophage_: README line 62 states: "each handler can resume the computation
at most once." The coroutine's state machine is a struct with pinned state;
once `resume(value)` returns `GeneratorState::Complete(ret)`, the coroutine
is exhausted. Calling `resume` again on an exhausted coroutine is undefined
(or panics). There is no way to clone or duplicate the continuation.

_Reffect_: `Coroutine` trait (core::ops) has a single `resume` method. Once
the trait returns `CoroutineState::Complete(ret)`, the coroutine is done.
Native Rust coroutines (nightly) follow the same contract. No duplication.

_Effing-mad_: Same as reffect. The `#[coroutine]` macro produces a type
implementing `Coroutine<...>`. Once it yields `Complete(ret)`, re-calling
`resume` is a type error (the method consumes `self: Pin<&mut Self>`; if
`self` is pinned, further invocations have nowhere to resume from).

_Free monad_: The continuation is stored as `Free<F, A>` (a clone-able data
structure if `F` and `A` are clone-able). The program can be cloned, stored
in a vector, and invoked multiple times with different effects injected:

```rust
let program: Free<ChooseFunctor, u32> = Free::pure(42);
let prog_clone = program.clone();
let result1 = program.fold_free(handler1);
let result2 = prog_clone.fold_free(handler2);
```

**Verdict: UNSUPPORTED by coroutine substrate.**

Coroutines enforce single-shot semantics by design. The Rust ownership model
makes it impossible to re-invoke a consumed continuation. Free, by contrast,
stores the continuation as data and can clone it arbitrarily.

---

### 3.2 Handler composition via data transformation (peel / send)

**Definition (PureScript-Run context):**

PureScript's `peel` (Run.purs:128-132) peels one layer of effect off a Run
program, returning either a VariantF with the rest-of-program as a data
value, or the final result. Once peeled, the rest-of-program is a
**first-class value** that can be stored, composed, or transformed. The `send`
combinator (Run.purs:144-148) injects an effect into the VariantF, allowing
handler composition to build up effects layer by layer without executing them.

This pattern enables **handler composition via data transformation**: one
handler removes effect A and transforms the remaining row R into a smaller
row R'; another handler removes B from R' and shrinks to R''; and so on. Each
step is a pure transformation of the program data structure, not execution.

**Test case:**

```rust
// Ground truth: PureScript-Run
let program: Run<State<Int> + Reader<String> + ()> = ...;

// Peel off State, leaving Reader
let program2: Run<Reader<String> + ()> = match program.peel() {
  Left(state_with_cont) => {
    // Transform the state effect
    let new_cont = state_with_cont.map_cont(|cont| cont.handled_state(...));
    // cont is a Re-invokable first-class value
    ...
  },
  Right(result) => Run::pure(result)
};
```

**Evidence from Rust crates:**

This property has two distinguishable sub-properties that the Rust crates
separate cleanly:

- _Type-level handler composition_: narrowing the effect row in the type
  system as handlers are attached, yielding a typed intermediate value that
  records which effects are still unhandled.
- _Data-level peel_: returning the effect together with the rest-of-program
  as a first-class Run-like value that the caller can store, inspect, or
  route to different handlers before execution.

_Corophage_: _Type-level composition supported._ `Program::handle`
(src/program.rs:133-156) returns a new `Program<'a, Effs, R, L, NewRemaining,
NewHandlers>` where `NewRemaining` is computed by
`<Remaining as CoproductSubsetter<H::Effects, SubsetIdx>>::Remainder` and
`NewHandlers` is the HList extended with the new handler. The effect row
narrows in the type system with each `.handle()` call, so handlers can be
attached in any order and the type system tracks which effects remain. This
is a genuine type-level peel: each step removes one effect from the
`Remaining` phantom. _Data-level peel unsupported._ The underlying
`co: GenericCo` stays opaque; the continuation is inside the pinned coroutine
state and is never surfaced as an extractable Run value the caller could
store or route to alternative handlers before execution.

_Reffect_: Same split. `catch0` and `catch1` (src/adapter.rs:59-86) return a
new `Effectful<F>` where `F` is the narrowed effect row (type-level peel),
but the resume loop runs the handler eagerly when an effect is yielded; the
continuation is never exposed as data.

_Effing-mad_: Same split. `handle` and `handle_group` (src/lib.rs:136-225)
produce a new `#[coroutine]` over a narrowed effect set (type-level peel),
but the inner coroutine runs to completion inside the resume loop and no
data-level continuation escapes.

_Free monad_: `resume` (free.rs:713-720) returns `Result<A, F<Free<F, A>>>`.
The `Err` case holds the effect AND the rest-of-program as a data value. This
continuation can be stored in a variable, transformed (e.g., by mapping over
the effect functor), and re-invoked later:

```rust
let program: Free<StateF, u32> = ...;
let peeled: Result<u32, StateF<Free<StateF, u32>>> = program.resume();
match peeled {
  Ok(result) => result,
  Err(state_effect) => {
    // The rest-of-program is now data: Free<StateF, u32>
    let cont: Free<StateF, u32> = state_effect.continuation;
    // Pass it to another handler, store it, etc.
  }
}
```

**Verdict: PARTIALLY SUPPORTED by coroutine substrate.**

_Type-level composition is supported_ and is not trivial: corophage, reffect,
and effing-mad each provide handler-attachment APIs that narrow the remaining
effect row in the type system, letting callers attach handlers in any order
and receive a type-checked intermediate `Program` / `Effectful` value at each
step. For many real-world uses of peel/send this is enough: the user wants
"attach these handlers, then run," not "extract the continuation and route
it elsewhere."

_Data-level peel is unsupported._ None of the three crates exposes the
rest-of-program as a first-class value the caller can store, hold across
branches, or pass to different handlers before execution. The coroutine
continuation stays inside the pinned state machine until a handler consumes
it. Free, by contrast, returns `Result<A, F<Free<F, A>>>` from `resume`,
where the `Err` branch carries the continuation as data. Any port use case
that needs to hold a continuation as a value (for backtracking, for
routing, for multi-interpretation) requires Free.

---

### 3.3 Pure-data AST inspection (runPure-style)

**Definition (PureScript-Run context):**

PureScript's `runPure` (Run.purs:279-292) walks a Free AST using `peel` to
extract one layer at a time, applies a **pure** handler to each layer
(returning a Step<a | Done>, not a monad), and reconstructs the program
without running side effects. The handler inspects the effect _as data_ and
decides whether to absorb it, replace it, or loop.

The key insight: the program is traversable as a **data structure**. You can
walk it without executing it, emit a result per node, and compose multiple
walkers.

**Test case:**

```rust
// Ground truth: PureScript-Run
let program: Run<Logger + Reader<String> + ()> = log("hello") >> ask();

// Walk the AST without executing
let walked = runPure(
  |logger_effect| {
    // logger_effect is not a side effect; it is data
    // Inspect it, log the fact that a log was requested, don't execute it
    Loop(program_with_logger_removed)  // or Done(value)
  },
  program
);

// walked is a new program with Logger effects removed, but no side effects occurred.
```

**Evidence from Rust crates:**

_Corophage_: Handlers are closures (`|effect| { ... control.resume(value)
}`). They run eagerly when an effect is yielded. There is no inspection-only
mode; a handler either resumes (executing the closure) or cancels. You cannot
"inspect and log" without side effects. The computation runs immediately once
the handler closure is invoked.

_Reffect_: Handlers run in the resume loop (src/adapter.rs:59-86). The
coroutine yields an effect, the handler is called with `pin!(handler).handle(...)`,
and the result is injected back. No separate inspection phase.

_Effing-mad_: Handlers are `impl FnMut(E) -> ControlFlow<R, E::Injection>`.
They run immediately when an effect is yielded (src/lib.rs:162-169). No pure
inspection mode.

_Free monad_: Folding over the AST is a _pure_ operation that does not require
executing effects. The `fold_free` method (free.rs:774-798) takes a
`NaturalTransformation<F, G>` where `G: MonadRec + 'static`. The
transformation is applied to each layer _as data_, and the result is built up
using the monad's `bind` and `pure`. This allows building a new program, or
emitting results, without running the original effects:

```rust
let inspect_free = |effect: StateF<Free<...>>| {
  // Inspect effect as data, e.g., print its discriminant
  println!("Effect: {:?}", effect);
  // Return the effect lifted into Option (pure data, no execution)
  Some(effect)
};

let new_program: Option<u32> = program.fold_free(inspect_free);
// No side effects occurred; we walked the AST and built a new value.
```

**Verdict: UNSUPPORTED by coroutine substrate.**

Coroutines run handlers eagerly and do not provide a pure inspection phase.
You cannot walk a coroutine-based program as data without executing it. Free,
by contrast, stores the program as an AST and allows pure traversal.

**Note:** You CAN drive a coroutine with a pure handler that accumulates
inspection results into a Vec, or logs them via a RefCell. But this is not
equivalent to a pure walk; it involves side effects (interior mutability).
The code is not pure; it is imperative wrapped in a closure.

---

### 3.4 Hoisting (natural-transformation lift)

**Definition (PureScript-Run context):**

PureScript's `hoistFree` (purescript-free, referenced in port-plan section
3.2) transforms every effect in a program via a natural transformation `F ~>
G` without running the program. Example: convert all `State` effects to
`(s -> Tuple s a)` via a natural transformation. The result is a new program
with the same structure but a different effect functor.

The key: hoisting is a **pure data transformation**. The original program is
not executed; a new AST is constructed with transformed layers.

**Test case:**

```rust
// Ground truth: conceptually
let program: Free<StateF, u32> = state_get >> state_put;

// Define a natural transformation: StateF -> IOF
let nt = |state_effect| IO::run_state(state_effect);

// Hoist the program without executing
let hoisted: Free<IOF, u32> = program.hoist_free(nt);

// hoisted is still a program (AST); it has not been executed.
// Now run it with an IO handler.
```

**Evidence from Rust crates:**

_Corophage_: Handlers are attached via `Program::handle(...)`, which wraps
the coroutine. There is no way to transform all effects in the program using
a natural transformation without attaching handlers. The coroutine is
black-box; you can only feed it handlers, not inspect or hoist its internal
functor.

_Reffect_: Same. Handlers are attached via `catch0`/`catch1`, which run the
coroutine. No hoisting mechanism.

_Effing-mad_: Same. The `#[coroutine]` macro produces sealed code; you cannot
hoist the effect functors.

_Free monad_: `hoist_free` (free.rs:847-866) takes a `NaturalTransformation<F,
G>` and returns a new `Free<G, A>`. It does not execute the program; it
recursively applies the natural transformation to each suspended layer:

```rust
pub fn hoist_free<G: Extract + Functor + 'static>(
    self,
    nt: impl NaturalTransformation<F, G> + Clone + 'static,
) -> Free<G, A> {
    match self.resume() {
        Ok(a) => Free::pure(a),
        Err(fa) => {
            let ga = nt.transform(fa);
            Free::<G, Free<F, A>>::lift_f(ga)
                .bind(move |inner| inner.hoist_free(nt.clone()))
        }
    }
}
```

The result is a new Free program; no side effects occur.

**Verdict: UNSUPPORTED by coroutine substrate.**

Coroutines have no mechanism to transform their internal functors without
executing them. Free, by contrast, is a tree of functors that can be walked
and hoisted via natural transformations.

---

### 3.5 Stack safety

**Definition (PureScript-Run context):**

Stack safety means that deeply nested monadic chains (e.g., `bind` chained
1,000,000 times) do not overflow the call stack. Free achieves this via
trampolining: each `bind` is stored as a continuation in a `CatList` tail,
and evaluation is driven iteratively by `tail_rec_m`, not recursion.

**Evidence from Rust crates:**

_Corophage_: The async coroutine is driven by a futures runtime, which is
inherently stack-safe. Yields and resumes do not grow the call stack; they are
state machine transitions. Confirmed by benchmark: 1000 yields take ~9.5 µs
(src/README.md, "Yield Scaling" table), which is linear, not exponential.

_Reffect_: Native Rust coroutines (nightly) are stack-safe by design. A
`Coroutine` implementation is a state machine; each `resume` call advances the
state without recursion. No stack growth.

_Effing-mad_: Same as reffect. The `#[coroutine]` macro produces a state
machine.

_Free monad_: Stack-safe by design via trampolining (`CatList` tail and
`tail_rec_m` iteration; free.rs:1-7). No stack growth during bind.

**Verdict: SUPPORTED by coroutine substrate.**

Both Free and coroutines achieve stack safety via different mechanisms
(trampolining vs. state machines), but both are stack-safe. This property is
not a differentiator.

---

### 3.6 Summary matrix: Free vs. coroutine substrate

| Property                       | Coroutine substrate | Free monad | Required for first-class?        |
| ------------------------------ | ------------------- | ---------- | -------------------------------- |
| Multi-shot                     | No                  | Yes        | Yes                              |
| Handler composition (type-lvl) | Yes                 | Yes        | Partial (subsumed by data-level) |
| Handler composition (data-lvl) | No                  | Yes        | Yes                              |
| Pure-data inspection           | No                  | Yes        | Yes                              |
| Hoisting (natural-tx)          | No                  | Yes        | Yes                              |
| Stack safety                   | Yes                 | Yes        | No (both support it)             |

**Key finding:** Coroutines support two of the six cells (stack safety and
type-level handler composition). The other four all require an AST data
structure. Since the port-plan commits to multi-shot interpretation,
data-level handler composition (peel/send routing), pure-data inspection,
and hoisting as core features (section 4.4), Free is **mandatory**, not
optional. The type-level composition coroutines do provide is a convenience
the Free-based port can adopt at its handler-API layer without needing a
coroutine substrate to get it; it is orthogonal to the substrate choice.

---

## 4. Rust portability and implementation assessment

### 4.1 What a coroutine-only effect system looks like

If the port rejected Free and committed to coroutines (e.g., via `#[coroutine]`
on stable Rust or via fauxgen async), the substrate would be:

1. **Computation:** A `Coroutine` or async closure that yields effects and
   returns a result.
2. **Handlers:** Closures attached via nesting; each wraps the previous
   coroutine and runs a resume loop.
3. **Composition:** Sequential chaining of handlers, not data-driven
   composition.
4. **Interpretation:** Execute-only (no peel/send, no runPure-style walkers,
   no hoisting).

This is what corophage, reffect, and effing-mad provide. It is ergonomic for
simple use cases (effect handlers run and produce results); handler
attachment order is type-checked via the row-narrowing pattern described in
section 3.2. But it cannot support:

- Multi-shot interpretation (Choose / Amb).
- Data-level peel/send where the continuation is a first-class value the
  caller can hold, branch on, or route to multiple handlers before
  execution. Type-level peel (row narrowing with each `.handle()`) is
  available and useful, but does not substitute for data-level peel in the
  cases that motivate it.
- Pure program walkers (runPure, logging inspectors, etc).
- Hoisting and effect-functor transformations.

### 4.2 Hybrid option: Free over a coroutine base functor

The port-plan already contemplates shipping four Free variants (`Free`, `RcFree`,
`ArcFree`, `FreeExplicit`). A fifth option would be **Free over a coroutine
base functor**: each suspended layer is a coroutine, not a plain functor. This
would combine the AST properties of Free (peel, send, runPure, hoisting) with
the stack-safe and ergonomic control flow of coroutines.

Feasibility: Coroutines would need to be wrapped in a functor trait. Rust's
nightly `Coroutine` trait is not a functor; wrapping would require additional
machinery. This is a legitimate design space but is not explored here (Stage 3
candidate).

### 4.3 What the port actually ships

Based on the evidence, the port should:

1. **Commit to Free for the core effect system**, not coroutines. Every Free
   property tested above (except stack safety, which both support equally) is
   **essential** to the port-plan's vision of first-class programs.
2. **Use the four planned Free variants** (Free, RcFree, ArcFree, FreeExplicit)
   to handle Send/Sync/lifetime constraints.
3. **Consider coroutines as an implementation detail**, not a substrate choice:
   e.g., use coroutines internally for the runner's resume loop, but expose
   the public API via Free.
4. **Revisit hybrid Free-over-coroutine** if ergonomic concerns about Free
   arise during implementation (Stage 3).

---

## 5. Comparison against port-plan section 4.4

Section 4.4 commits to "a Free monad design" and lists four variants. The
stated goal is to preserve: "multi-shot continuations" (handler composition
via peel/send), "pure-data interpreters" (runPure-style), and "hoisting" (nt-driven
transformations).

**Finding:** Coroutines support type-level handler attachment (row
narrowing) but do not support the data-level versions of the four
distinguishing properties: multi-shot interpretation, data-level peel/send,
pure-data inspection, and hoisting. Since the port-plan's stated goals name
the data-level properties specifically (continuations that can be invoked
multiple times, programs that can be walked as data, natural-transformation
hoists), Free (or an equivalent AST) is **required**, not optional.

The plan's four-variant structure (Free, RcFree, ArcFree, FreeExplicit)
stands unchanged. The type-level row-narrowing pattern surfaced by
corophage, reffect, and effing-mad is a handler-API convenience the port
should adopt on top of its Free substrate; it is independent of the
substrate choice.

---

## 6. Concrete plan-edit recommendations

The primary finding is **defensive**: the port-plan's section 4.4 Free-family
commitment is validated, not questioned. The four-variant Free family stays.

One minor edit is warranted from the type-level-vs-data-level distinction
surfaced in section 3.2.

**6.1 Note the type-level row-narrowing pattern in section 4.1.** All three
Rust coroutine crates (corophage's `Program::handle`, reffect's
`catch0`/`catch1`, effing-mad's `handle`) implement a consistent
row-narrowing pattern: the handler-attachment method consumes the effectful
value and returns a new value typed against a reduced remaining-effect row,
with an HList of accumulated handlers. This is a type-level version of
PureScript's peel that does NOT require a coroutine substrate; it is a
handler-API design the port can adopt directly on top of Free. The plan's
section 4.1 Option 4 discussion (where corophage is named as the reference
implementation) should note this pattern as a handler-API target
independently of the row-encoding choice.

If the plan is revised in the future (for example, to relax the
first-class-programs requirement or to explore a Free-over-coroutine
hybrid), revisit this dive's findings. For now, proceed with Free as stated
in section 4.4 and adopt the row-narrowing handler API pattern on top of it.

---

## 7. References

**Coroutine crates:**

- `corophage/src/coroutine.rs:52-58`: GenericCo struct and single-shot note.
- `corophage/README.md:62`: "handlers are FnOnce" / single-shot policy.
- `corophage/src/program.rs:133-150`: Program::handle signature and chaining.
- `reffect/src/adapter.rs:27-86`: Catch0/Catch1 and resume loop.
- `effing-mad/src/lib.rs:136-225`: handle and handle_group coroutine wrappers.

**Free monad (ground truth):**

- `fp-library/src/types/free.rs:231+`: Free struct and to_view.
- `fp-library/src/types/free.rs:713-720`: resume method.
- `fp-library/src/types/free.rs:774-798`: fold_free with NaturalTransformation.
- `fp-library/src/types/free.rs:847-866`: hoist_free implementation.

**PureScript-Run (spec):**

- `purescript-run/src/Run.purs:93`: Run = Free (VariantF r) a.
- `purescript-run/src/Run.purs:128-132`: peel and resume.
- `purescript-run/src/Run.purs:144-148`: send.
- `purescript-run/src/Run.purs:279-292`: runPure.
- `purescript-run/src/Run/Internal.purs`: Choose effect.

**Fused-effects (foil):**

- `fused-effects/src/Control/Algebra.hs:74`: Algebra typeclass (no Free).
- `fused-effects/src/Control/Algebra.hs:133`: Has constraint.
- `fused-effects/docs/overview.md:37-45`: Explicit statement of no Free.
