# Run Effects

This subsystem is optional and experimental; its API may change between
releases. Enable the `effects` crate feature to use the `Run` wrappers, effect
row macros, and handler macros through `fp-library`.
Invoke effect macros through `fp_library`, not directly through `fp_macros`;
the proc-macro crate cannot observe whether `fp-library` enabled the optional
feature. The low-level `raw_effects!` helper is intentionally internal-only at
`fp_library::__internal::raw_effects!`; public code should use `effects!`,
`scoped_effects!`, `define_effect_row_aliases!`, `define_scoped_row!`,
`handlers!`, and `scoped_handlers!`.

The `Run` subsystem provides row-polymorphic effect programs. A program has two
independent rows:

- `R`, the first-order effect row, stores ordinary operation functors such as
  `State`, `Reader`, `Except`, `Writer`, `Choose`, and `Empty`.
- `S`, the scoped-effect row, stores action-scoped operations such as `Catch`,
  `Local`, `Bracket`, `Span`, and Writer `listen` / `censor`.

First-order effects describe one suspended operation and its continuation.
Scoped effects describe an operation that owns an action program and controls
how that action is handled before the outer continuation resumes. In Heftia
terms, the current scoped row covers the action-scoped subset of higher-order
effects. It is not a catch-all representation for every possible higher-order
effect; higher-order effects that need IO, public resumption, or target-monad
semantics, or async around a scoped action, need a separate design before they
are added. First-order async is available through the `Await` future base-lift
effect (see [Async Interpretation](#async-interpretation)).

The user-facing design is intentionally explicit about rows and handlers. Row
aliases make program types readable, while handler lists make the meaning of
each effect visible at the call site. The dual-row representation exists because
ordinary first-order operations and action-scoped operations have different
continuation shapes; keeping them separate prevents handler APIs from mixing
single-operation interpretation with around-action control flow.

## Wrapper Families

There are six concrete `Run` wrappers:

| Wrapper                       | Substrate               | Reusable | Thread-safe | Typical use                                    |
| ----------------------------- | ----------------------- | -------- | ----------- | ---------------------------------------------- |
| `Run<R, S, A>`                | erased `Free`           | No       | No          | single-shot local programs                     |
| `RcRun<R, S, A>`              | erased `RcFree`         | Yes      | No          | reusable local programs                        |
| `ArcRun<R, S, A>`             | erased `ArcFree`        | Yes      | Yes         | reusable `Send + Sync` programs                |
| `RunExplicit<'a, R, S, A>`    | typed `FreeExplicit`    | No       | No          | lifetime-aware single-shot programs            |
| `RcRunExplicit<'a, R, S, A>`  | typed `RcFreeExplicit`  | Yes      | No          | reusable lifetime-aware programs               |
| `ArcRunExplicit<'a, R, S, A>` | typed `ArcFreeExplicit` | Yes      | Yes         | reusable lifetime-aware `Send + Sync` programs |

Use the default `Run` wrapper first when a program is single-shot and does not
need borrowed payloads. Use `RcRun` or `ArcRun` when the same program must be
handled more than once. Use an Explicit wrapper when the program must carry
non-`'static` action payloads or when the typed substrate gives clearer
boundaries for a composition.

The Erased Free substrate needs a real suspension functor. A recursively
self-containing functor such as `IdentityBrand` is layout-cyclic under
`Free<IdentityBrand, A>` because the erased Free node would contain another
Free node directly. Use an effect functor whose payload is a continuation
position, or use an Explicit substrate when a typed recursive structure is the
actual goal.

The wrapper choice determines which built-in effect brand appears in a row. For
example, default `Run` stores default boxed state operations under
`BoxStateBrand<BoxBrand, S>`, `RcRun` stores reusable local state operations
under `StateBrand<RcBrand, S>`, and `ArcRun` stores thread-safe state operations
under `SendStateBrand<ArcBrand, S>`.

## Row Aliases

Rows are type-level coproducts. Writing them by hand is possible, but most code
should use `define_effect_row_aliases!` for named aliases:

```rust
use fp_library::{
	brands::{
		BoxBrand,
		BoxReaderBrand,
		BoxStateBrand,
		ExceptBrand,
	},
	define_effect_row_aliases,
};

define_effect_row_aliases! {
	type AppEffects = first_order [
		BoxReaderBrand<BoxBrand, i32>,
		BoxStateBrand<BoxBrand, i32>,
		ExceptBrand<&'static str>,
	];
	type ReaderRemoved = first_order [
		BoxStateBrand<BoxBrand, i32>,
		ExceptBrand<&'static str>,
	];
	type NoScopedEffects = scoped [];
}
```

The macro emits ordinary type aliases. It does not create hidden effect stacks,
programs, or handlers. This keeps row types visible in compiler diagnostics
while removing repeated `CoproductBrand` nesting from user code.

## First-Order Effects

First-order effects are handled by `handlers! { ... }`. A handler receives the
operation with its continuation already rewritten so that resuming the operation
returns another program in the remaining row.

This example handles a small `State` program. The handler stores the state in an
`Rc<RefCell<_>>`, and the assertions check both the returned value and the final
state cell.

```rust
use fp_library::{
	brands::{
		BoxBrand,
		BoxStateBrand,
		CNilBrand,
		CoproductBrand,
		CoyonedaBrand,
	},
	handlers,
	types::effects::{
		run::Run,
		scoped_nt,
		state::BoxState,
	},
};
use std::{
	cell::RefCell,
	rc::Rc,
};

type StateRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
type NoScoped = CNilBrand;
type Program<A> = Run<StateRow, NoScoped, A>;

let state = Rc::new(RefCell::new(10));
let state_for_handler = Rc::clone(&state);

let program: Program<i32> = Run::<StateRow, NoScoped, i32>::get()
	.bind(|current| Run::<StateRow, NoScoped, ()>::put::<i32, _>(current + 5))
	.bind(|()| Run::<StateRow, NoScoped, i32>::get());

let result = program.handle(
	handlers! {
		BoxStateBrand<BoxBrand, i32>: move |op: BoxState<'_, BoxBrand, i32, Program<i32>>| {
			match op {
				BoxState::Get(k) => k(*state_for_handler.borrow()),
				BoxState::Put(next_state, k) => {
					*state_for_handler.borrow_mut() = next_state;
					k(())
				}
			}
		},
	},
	scoped_nt(),
);

assert_eq!(result, 15);
assert_eq!(*state.borrow(), 15);
```

Built-in first-order effects currently include:

- `State`: `get` and `put`.
- `Reader`: `ask`.
- `Except`: `throw`.
- `Writer`: `tell`.
- `Choose`: binary nondeterministic choice for reusable wrappers.
- `Empty`: abortive nondeterministic branch.

## Scoped Effects

Scoped effects are handled by `scoped_handlers! { ... }`. A scoped handler value
does not handle an ordinary operation functor directly; it receives a selected
action and a wrapper-owned continuation boundary, then decides how the action is
run before resuming the outer continuation.

This example uses `Local` to modify the Reader environment only for the selected
action. The outer `ask` still sees the unmodified environment.

```rust
use fp_library::{
	brands::{
		BoxBrand,
		BoxLocalBrand,
		BoxReaderBrand,
	},
	define_effect_row_aliases,
	handlers,
	scoped_handlers,
	types::effects::{
		reader::BoxReader,
		run::Run,
		standard_scoped_handlers::local_handler,
	},
};

define_effect_row_aliases! {
	type ReaderRow = first_order [BoxReaderBrand<BoxBrand, i32>];
	type ReaderRemoved = first_order [];
	type LocalRow = scoped [BoxLocalBrand<BoxBrand, i32>];
}
type Program<A> = Run<ReaderRow, LocalRow, A>;

let action: Program<i32> = Run::<ReaderRow, LocalRow, i32>::ask();
let program: Program<i32> = Run::local::<i32, _>(|env| env + 2, action)
	.bind(|local_env| {
		Run::<ReaderRow, LocalRow, i32>::ask()
			.map(move |outer_env| local_env + outer_env)
	});

let result = program.handle(
	handlers! {
		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Program<i32>>| {
			match op {
				BoxReader::Ask(k) => k(10),
			}
		},
	},
	scoped_handlers! {
		BoxLocalBrand<BoxBrand, i32>: local_handler::<_, ReaderRemoved, _>(),
	},
);

assert_eq!(result, 22);
```

Built-in scoped effects currently include:

- `Catch`: handle `Except` failures inside a selected action.
- `Local` and `RefLocal`: run a selected action under a modified Reader
  environment.
- `Bracket` and `RefBracket`: acquire a resource, run a selected action, and
  release the resource on the normal path while Rust `Drop` preserves unwind
  cleanup.
- `Span`: bracket an action with a tag for tracing-style handlers.
- Writer `censor`: transform selected Writer output before or after collection,
  depending on the chosen standard handler.
- Writer `listen`: observe selected Writer output while preserving the original
  `Tell`s for outer Writer handlers.

## Handler Order

`handle(handlers, scoped_handlers)` closes a program when both rows are empty
after every handler has run. For incremental handling, use
`handle_with::<EffectBrand, _, RemainingRow>(...)` or
`handle_with_handler::<EffectBrand, _, RemainingRow>(...)` for one first-order
effect at a time. Use the scoped-handler list when closing or handling programs
that still contain scoped operations.

Handler order is semantic. For example, handling Writer inside NonDet gives
each nondeterministic branch its own observed log, while handling Writer outside
NonDet accumulates output globally. The library keeps these orderings explicit
instead of hiding them behind a single global runtime.

## Standard Handler Values

Standard scoped handlers live under
`fp_library::types::effects::standard_scoped_handlers`. Constructors are named
as handler values: `catch_handler`, `local_handler`, `ref_local_handler`,
`bracket_handler`, `ref_bracket_handler`, `span_handler`,
`writer_pre_handler`, and `writer_post_handler`.

Some constructors require an explicit row-minus alias:

```text
BoxLocalBrand<BoxBrand, i32>: local_handler::<_, ReaderRemoved, _>(),
```

The explicit row-minus type is intentional for now. Stable Rust can infer the
scoped-row position for representative handlers, but it cannot infer the
remaining first-order row from trait-selection context alone without making the
handler surface less transparent.

## Async Interpretation

The default `Run` family can interpret programs asynchronously through the
`Await` future base-lift effect. `Await` (brand `AwaitBrand`) is a first-order
effect that carries a boxed local `Future`; because its brand is a `Functor`
over that future, the interpreter can lower it directly to a future of the next
program and await it.

Two public pieces drive this:

- `Run::await_future(future)` embeds a `Future` into a program's first-order
  row as an `Await` effect; the awaited output feeds the continuation. The row
  must contain `AwaitBrand` at some position (any position works).
- `Run::run_async(handlers)` drives such a program to completion, returning a
  runtime-agnostic future. At each layer it projects the `Await` effect out of
  the row and awaits it; every other first-order effect is dispatched to
  `handlers`. The returned future is `Ready` only once every embedded future
  has completed, so it can be driven by any executor (a hand-written poll loop
  or a runtime such as Tokio); the core takes no runtime dependency.

```rust
use fp_library::{
	brands::{
		AwaitBrand,
		CNilBrand,
		IdentityBrand,
	},
	effects,
	handlers,
	types::{
		Identity,
		effects::run::Run,
	},
};
use std::{
	future::Future,
	pin::pin,
	task::{
		Context,
		Poll,
		Waker,
	},
};

// Minimal std-only executor; here the embedded futures are immediately ready.
fn block_on<F: Future>(future: F) -> F::Output {
	let mut future = pin!(future);
	let mut context = Context::from_waker(Waker::noop());
	loop {
		if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
			return value;
		}
	}
}

type Row = effects![AwaitBrand, IdentityBrand];
type Prog<A> = Run<Row, CNilBrand, A>;

// Await a future, hand its value to an Identity handler, then await another
// future computed from the result: 20 -> 21 -> 42.
let program: Prog<i32> = Run::await_future(async { 20 })
	.bind(|first| Run::lift::<IdentityBrand, _>(Identity(first + 1)))
	.bind(|second| Run::await_future(async move { second * 2 }));

// `run_async` returns a runtime-agnostic future; drive it on any executor.
let result = block_on(program.run_async(handlers! {
	IdentityBrand: |operation: Identity<Prog<i32>>| operation.0,
}));
assert_eq!(result, 42);
```

The embedded future is local (non-`Send`) and single-shot, so async lives on
the single-shot `Box`-backed default `Run` family. Async on the multi-shot
`Rc` / `Arc` families (which would need clone-able cached futures) and async
around scoped actions are not yet supported; see Current Limits.

## Current Limits

The current standard effect set is intentionally bounded. First-order async is
available through the `Await` base-lift effect (see
[Async Interpretation](#async-interpretation)). Runtime-sensitive Heftia-style
effects such as concurrent, shift, provider, unlift, stream, timer, subprocess,
parallel, and database-provider effects remain deferred until the library has a
precise design for them; the still-open policies are higher-order continuation
capture, IO, target-monad semantics, and async on the multi-shot `Rc` / `Arc`
wrapper families and around scoped actions.

Custom first-order effects are supported manually; see
[Custom First-Order Effects](./custom-effects.md). A future `define_effect!`
macro should remove stable boilerplate only after more custom examples prove the
generated shape. Handler bodies should remain explicit because they define the
meaning of each effect.
