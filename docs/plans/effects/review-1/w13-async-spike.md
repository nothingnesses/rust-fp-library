# W13 Async-Interpreter Feasibility Spike

This records the bounded feasibility spike the W13 research recommended. It
settles the central empirical unknown: whether a direct async interpreter
driver loop can run an fp-library effect program on stable Rust, holding
the program as data across `.await` points, without a
`MonadRec`-over-`Future` impl and without naming a recursive async type.
The gate permits such a spike folded into the decision record.

## Method

The spike is [`fp-library/tests/w13_async_spike.rs`](../../../../fp-library/tests/w13_async_spike.rs).
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

## What it does not yet cover

Honest scope limits; these are the next increments, not blockers:

- Async handlers that `.await` real IO to produce the next program. The
  spike's `.await` is a suspension point; the effect advance itself uses
  the synchronous handler. The next increment is a handler that returns a
  `Future<Output = next program>`, which would also test the async-closure
  lending ergonomic (a handler whose returned future borrows captured
  handler state).
- The scoped (dual-row) async path: boundary / carrier / residual dispatch
  under async. The spike covers the first-order row only. The scoped path
  shares this core loop but adds the scoped dispatch roles documented in
  [`w8-scoped-dispatch-design-note.md`](w8-scoped-dispatch-design-note.md).
- The `Send` / Arc family: this spike is local (non-`Send`) on default
  `Run`. The Arc family would carry `Send + Sync` bounds end to end.

## Implication for W13

The substrate-feasibility risk for an async interpreter is retired for the
first-order core: the shape compiles and runs on stable over the real
substrate. The remaining work (async-producing handlers, the scoped path,
the `Send` family) is incremental rather than a fundamental blocker, which
supports proceeding with the executor-neutral, direct-async-loop direction
when the W13 runtime policy is adopted.
