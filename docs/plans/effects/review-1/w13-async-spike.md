# W13 Async-Interpreter Feasibility Spike

This records the bounded feasibility spike the W13 research recommended. It
settles the central empirical unknown: whether a direct async interpreter
driver loop can run an fp-library effect program on stable Rust, holding
the program as data across `.await` points, without a
`MonadRec`-over-`Future` impl and without naming a recursive async type.
The gate permits such a spike folded into the decision record.

## Method

The spike is [`fp-library/tests/async_interpreter_feasibility.rs`](../../../../fp-library/tests/async_interpreter_feasibility.rs).
It is an `async` block whose `loop` peels a `Run` program, `.await`s a
genuine suspension (`yield_once()`) at each first-order layer, and advances
via the real `DispatchHandlers::dispatch`. The continuation stays a `Run`
value (data); only the driver is async.

Scope, deliberately minimal: default `Run`, first-order only (the Identity
effect over `CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>`),
single-shot, non-`Send`, scoped row `CNilBrand`. A std-only `block_on`
(via `Waker::noop()`, stable since Rust 1.85; toolchain is 1.94) runs it,
with no `tokio` and no `futures` dependency. The test program chains 2000
Identity effects to stress depth.

## Result

Passes, and passes under full `just verify` (so it is clippy-clean across
`--all-targets`). The program threads to its depth (result 2000), the
driver suspends exactly once per effect layer (2000 suspensions), and a
std-only executor drives it to completion.

## What it proves

- A direct async driver loop holds the `Run` program (data) across
  `.await` on stable Rust.
- It needs no `MonadRec`-over-`Future` impl, no named recursive async
  type, and not even a `Pin<Box<dyn Future>>` around a recursive tail:
  the driver is a flat `loop` and the continuation is data, so there is no
  recursion to box. This is the switch-resume lesson (keep the
  continuation as data, await handlers, do not capture the async stack)
  confirmed empirically.
- It reuses the real substrate (`peel` plus `DispatchHandlers::dispatch`),
  so it drives an actual fp-library program, not a toy model.
- It is stack-safe at depth (the loop is iterative; peel and dispatch are
  O(1) per step).
- It is runtime-neutral: a std-only executor suffices, supporting the
  executor-neutral recommendation.

This confirms the research's "direct async interpreter loop" recommendation
over the `MonadRec`-over-`Future` framing in `findings.md` section 6.

## Part 2: the three increments, all feasible

A second POC,
[`fp-library/tests/async_interpreter_increments.rs`](../../../../fp-library/tests/async_interpreter_increments.rs),
settles the three increments the first spike left open. All three pass on
stable Rust over the real substrate, and the whole suite is green under
`just verify`.

- Async-producing handlers. A Reader program
  (`ask().bind(|x| ask().bind(|y| pure(x + y)))`) is driven so each `ask`
  is answered by a value obtained from an awaited `fetch(..)`. The awaited
  values (10, then 20) thread into the result (30). Design note: the await
  happens in the driver, which then advances with an ordinary synchronous
  handler (`BoxReader::Ask(k) => k(env)`) built with the awaited value.
  This keeps handlers synchronous and confines async to the driver, which
  avoids the async-closure lending problem (a handler returning a future
  that borrows captured state) and preserves the existing handler
  ecosystem. So the feasible and preferable shape is "await in the driver,
  dispatch synchronously," not "handlers return futures."
- Scoped / dual-row path. A scoped `Span` program is driven under the async
  loop via the public `peel` plus `DispatchScopedHandlers::dispatch_scoped`
  path (the same path `handle_rec` uses), awaiting at the scoped layer. The
  scoped layer is held across `.await` and dispatched; the result (42) is
  correct.
- Send / Arc family. The `ArcRun` async driver runs on a spawned
  `std::thread`, which compiles only if the program, handlers, and driver
  future are all `Send + 'static`. A 100-deep program threads to 100 across
  the thread boundary.

## Still open (finer-grained, not blockers)

- Carrier-based scoped effects under async. The Span case uses the ordinary
  one-slot `dispatch_scoped` path. Writer `listen` / `censor`, Catch,
  Bracket, Local, and RefLocal use the around-action carrier path
  (see [`w8-scoped-dispatch-design-note.md`](w8-scoped-dispatch-design-note.md)),
  whose async behavior is not yet exercised. Default `Run`'s production
  `handle` also uses the raw scoped path rather than `dispatch_scoped`; the
  async driver here used `dispatch_scoped`, which works for Span, so which
  scoped path the real async interpreter standardizes on is a design choice
  to settle.
- Combined scoped + Arc, and a single program mixing first-order, scoped,
  and async.
- Real runtime integration (a concrete async runtime and IO), as opposed
  to the std-only `block_on` plus yield used here.

## Implication for W13

The substrate-feasibility risk for an async interpreter is retired across
the first-order core, async-producing handling, the basic scoped path, and
the Send / Arc family: each compiles and runs on stable over the real
substrate, with a std-only executor (no runtime dependency), confirming
both the direct-async-loop and executor-neutral recommendations. The
remaining items are finer-grained increments and design choices, not
fundamental blockers. This clears the way to adopt the W13 policy
recommendations and, when explicitly requested, build the real async
interpreter.
