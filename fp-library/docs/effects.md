# The effects system

The effects subsystem represents effectful programs as data: a program is a
value of type `Free<Row, A>`, built from effect operations and interpreted by
a dispatch loop that gives each operation meaning. This document is the
design story: why the system is shaped the way it is, the catalog of built-in
reference effects and their pinned semantics, how handler state behaves, and
how the vocabulary corresponds to purescript-run's. The hands-on companion is
the custom-effects guide (`docs/custom-effects.md`), which walks through
defining an effect with `define_effect!`, declaring a row with `define_row!`,
and interpreting a program end to end; everything there applies here.

The subsystem is behind the optional `effects` feature and is experimental:
the API may change between releases.

## Design

### One unified row

A program's effect set is one row: a coproduct of effect brands, each cell
wrapped in `Coyoneda` so every effect gets its `Functor` for free (the functor
an effect needs is exactly a mapping over its continuation, which `Coyoneda`
defers structurally). First-order and higher-order effects live in the same
row; there is no second row for scoped effects and no boundary-frame protocol
between rows. The design follows purescript-run's single open row of effect
functors, with the higher-order treatment taken from heftia (see below).

A row is a nominal brand, not a type alias, because a higher-order effect's
cell stores sub-programs over the very row that contains it: the row names
itself, which is a definition cycle for an alias but lazy and legal through a
nominal brand's kind projection. `define_row!` always emits the nominal form.

### Brand-keyed dispatch

An interpreter selects the suspended operation by its effect brand, via
type-directed `uninject` over the coproduct, not by the cell's position in
the row. Dispatch arms may therefore be written in any order, and adding an
effect to a row never re-indexes the others: constructors inject by type
(`Coproduct::inject`) and handlers select by type, so nothing in the system
depends on a row's declared order.

### Higher-order effects by elaboration

A higher-order effect is one whose operation owns sub-programs (`Catch`'s
action, `Bracket`'s acquire/body/release). Rather than interpreting these
behind scoped boundaries, the interpreter elaborates them: it runs the owned
sub-programs itself, recursively, and threads their results onward. The
semantics of each higher-order effect then fall out of one decision, how the
elaboration shares or scopes the interpreter's handler state at the recursive
call:

- `Catch` interprets its action under the same state cell, so a write made
  before a caught throw survives the recovery.
- `Local` interprets its action under a transformed copy of the environment,
  so the scope ends when the action ends.
- `Listen` interprets its action under the same log, preserving the action's
  writes while observing the delta they added.
- `Censor` interprets its action under a fresh local log, transforming and
  emitting the total afterwards, so the censor scopes the accumulation.

Native stack use grows with the nesting depth of higher-order operations, not
with program length: elaboration recurses per owned sub-program, while the
program's own `bind` chain is driven iteratively.

### The abort channel

Aborting effects are distinct cases of one precise error type in the
interpreter's return channel, not a single catch-all failure: a bare `Throw`,
an `Empty` dead branch, and a typed `Except` throw carrying its error value.
Recovery boundaries are therefore selective by construction: `Catch` recovers
the bare `Throw` only, while `Empty` and `Except` propagate through it
untouched, and a typed error is recovered at its own boundary, which reifies
the abort into a `Result` (the shape of heftia's `runThrow` and
purescript-run's `runExcept`). Making the cases distinct data keeps a `catch`
from silently swallowing failure kinds it was never meant to handle.

### The store axis and the single-shot guard

The `Free` substrate carries a `Store` parameter selecting how continuations
are stored: the default `Box` store holds `FnOnce` continuations, so each
continuation can be called at most once (single-shot), while the `Rc`/`Arc`
stores hold re-callable `Fn` continuations for multi-shot interpretation.
Interpretation today targets the single-shot `Box` store; effects whose
handlers re-enter a continuation more than once (nondeterministic choice) are
not expressible on it and wait for the multi-shot interpretation round. The
custom-effects guide documents the same guard for hand-written interpreters.

## The built-in reference catalog

The library carries a catalog of fifteen built-in effects. Their status today
is deliberate: they are crate-internal conformance fixtures, not public API.
Every built-in is a `define_effect!` invocation, which makes the catalog the
macro's permanent conformance suite, and their payloads are pinned to
concrete types (an `i32` environment, a `String` log) that keep the reference
interpreter's test oracle simple. The public, payload-generalised form of the
catalog ships together with the planned generic runner surface; until then,
the catalog documents the reference semantics, and a program that wants a
built-in's behaviour defines the same shape in its own crate, exactly as the
custom-effects guide shows, since the definitions below compile against the
public macros unchanged.

The definitions are ordinary `define_effect!` invocations. A representative
first-order slice of the catalog, exactly as the built-ins define it (with
`State`'s payload pinned at the row, the parameterise-and-pin convention the
built-ins use for every generic effect):

```rust
use fp_library::{
	define_effect,
	define_row,
	types::Free,
};

define_effect! {
	/// State over a cell of `S`. `Get` reads the current state; `Put` writes it.
	#[handler_state(shared_by_reference)]
	pub effect State<S: 'static> {
		/// Read the current state.
		fn get() -> S;
		/// Write the state.
		fn put(value: S) -> ();
	}
}

define_effect! {
	/// Throw with a unit error: the program aborts and carries no continuation.
	#[handler_state(none)]
	pub effect Throw {
		/// Abort the current program with a bare throw.
		fn throw() -> !;
	}
}

define_effect! {
	/// Writer over a `String` log. `tell` appends to the log.
	#[handler_state(shared_by_reference)]
	pub effect Writer {
		/// Append `value` to the log.
		fn tell(value: String) -> ();
	}
}

define_row! {
	/// A row of the three effects, `State`'s cell pinned to `bool`.
	pub row AppRow {
		StateBrand<bool>,
		ThrowBrand,
		WriterBrand,
	}
}

fn main() {
	// A program is a value; `bind` sequences operations and the row is
	// inferred from the annotation.
	let program: Free<AppRow, bool> = put(true)
		.bind(|()| tell("wrote".to_string()))
		.bind(|()| get());
	// It is data: it suspends at its first operation until interpreted.
	assert!(program.resume().is_err());
}
```

And the higher-order shape, `Catch` exactly as the built-in defines it, whose
row demonstrates the nominal self-reference:

```rust
use fp_library::{
	define_effect,
	define_row,
	types::Free,
};

define_effect! {
	/// Throw with a unit error: the program aborts and carries no continuation.
	#[handler_state(none)]
	pub effect Throw {
		/// Abort the current program with a bare throw.
		fn throw() -> !;
	}
}

define_effect! {
	/// Catch is a higher-order effect: it owns an action sub-program and a
	/// recovery thunk, its result equal to the action result `RAction`.
	#[handler_state(none)]
	pub effect Catch<RAction: 'static> {
		/// Run `action`, recovering a bare throw with `recover`; the action's
		/// value (or the recovery's) threads to the continuation.
		fn catch(action: Program<RAction>, recover: impl FnOnce() -> Program<RAction>) -> RAction;
	}
}

define_row! {
	/// The row names itself inside `CatchBrand`'s cell (the owned
	/// sub-programs are `Free<AppRow, i32>`), which the nominal row brand
	/// makes legal: the self-reference is lazy through its kind projection.
	pub row AppRow {
		ThrowBrand,
		CatchBrand<AppRow, i32>,
	}
}

fn main() {
	// `throw` never resumes, so its result type is free; Rust's turbofish is
	// all-or-nothing, so the inferred row parameters are written `_`.
	let aborting: Free<AppRow, i32> = throw::<i32, _, _>();
	let program: Free<AppRow, i32> = catch(aborting, || Free::pure(42));
	assert!(program.resume().is_err());
}
```

The full catalog, with each effect's operations, its declared
`#[handler_state(...)]` class, and its reference semantics:

| Effect                | Operations                                                            | Handler state         | Reference semantics                                                                                   |
| --------------------- | --------------------------------------------------------------------- | --------------------- | ----------------------------------------------------------------------------------------------------- |
| `State<S>`            | `get() -> S`, `put(value: S) -> ()`                                   | `shared_by_reference` | Reads and writes the shared state cell.                                                               |
| `Reader`              | `ask() -> i32`                                                        | `scoped_by_value`     | Reads the environment; `Local` scopes it.                                                             |
| `Writer`              | `tell(value: String) -> ()`                                           | `shared_by_reference` | Appends to the log; the log is append-only.                                                           |
| `Fresh`               | `fresh() -> usize`                                                    | `shared_by_reference` | Yields the next counter value; the successor policy is handler state.                                 |
| `Input`               | `input() -> Option<&'static str>`                                     | `shared_by_reference` | Drains a queue; `None` once empty.                                                                    |
| `KVStore`             | `lookup(key) -> Option<i32>`, `update(key, value: Option<i32>) -> ()` | `shared_by_reference` | Map read; `Some` inserts or overwrites, `None` deletes.                                               |
| `Throw`               | `throw() -> !`                                                        | `none`                | Bare abort; the one case `Catch` recovers.                                                            |
| `Empty`               | `empty() -> !`                                                        | `none`                | Dead branch; propagates through `Catch`. Branch pruning waits for multi-shot interpretation.          |
| `Except<E>`           | `throw(error: E) -> !`                                                | `none`                | Typed abort carried in the return channel; recovered at its own boundary, propagates through `Catch`. |
| `Identity`            | `identity_op(value: i32) -> i32`                                      | `none`                | Value echo; the no-op target interposition rewrites.                                                  |
| `Catch<RAction>`      | `catch(action, recover) -> RAction`                                   | `none`                | Recovers a bare `Throw` only; state written before a caught throw survives.                           |
| `Local<Env, RAction>` | `local(modify, action) -> RAction`                                    | `none`                | Runs the action under `modify(env)`; the scope ends with the action.                                  |
| `Listen<RAction, W>`  | `listen(action) -> (RAction, W)`                                      | `none`                | Runs under the same log, observing the delta; the action's writes are preserved.                      |
| `Censor<W, RAction>`  | `censor(f, action) -> RAction`                                        | `none`                | Fresh local log, then `f(total)` emitted to the outer log; transactional on abort.                    |
| `Bracket<Res, RBody>` | `bracket(acquire, body, release) -> RBody`                            | `none`                | Acquire, use, release in order; a body abort still releases.                                          |

Four of these carry semantics precise enough to state as contracts, pinned by
the reference interpreter's test suite:

- **`Catch` recovers the bare `Throw` only.** An `Empty` dead branch and a
  typed `Except` throw are different effects with their own boundaries; both
  propagate through a `catch` intact, the typed error keeping its payload.
  And because the elaboration shares the state cell, a `put` executed before
  a caught throw survives into the recovery and beyond.
- **`Bracket` releases on a body abort.** If the body aborts, `release` still
  runs and then the body's abort propagates, taking priority over any abort
  `release` itself raises during that unwind; on success, a release abort
  propagates. An acquire abort skips both body and release, since no resource
  exists yet. A resource that flows into more than one callable must be
  duplicable (`Res: Clone`); the reference catalog pins `Res` to a `Copy`
  type, the trivial case.
- **`Censor` is transactional on abort.** The censored action writes to a
  fresh local log, and the transformed total is emitted only when the action
  completes; if the action aborts, the local log is dropped and nothing
  reaches the outer log. Writes made outside the censor survive the abort;
  the asymmetry is a property of where the censor boundary sits.
- **`Writer`'s log is append-only.** A handler never rewrites or truncates
  earlier writes. This invariant is what makes `Listen`'s observed delta (the
  tail the action appended) well-defined.

Beyond the effects themselves, the catalog exercises interposition: a
rewriting walker that steps a program one layer at a time and either replaces
a matched effect's dispatch with a supplied program or re-embeds the
unmatched layer unchanged (the deeper primitive heftia builds scoped `catch`
on). It is crate-internal today, in the same status as the catalog.

## Handler state and ordering

Every effect declares how a handler holds its state, via the mandatory
`#[handler_state(...)]` attribute:

- `none`: the handler needs no state (the aborting effects, the pure
  higher-order elaborations).
- `scoped_by_value`: the state is a value copied into a scope, so mutation
  inside the scope cannot leak out (`Reader`'s environment under `Local`).
- `shared_by_reference`: the state is a shared cell the handler reads and
  writes in place (`State`, `Writer`, `Fresh`, `Input`, `KVStore`).
- `threaded_by_value`: reserved for the planned threaded-accumulator runners;
  no built-in declares it yet.

The shared-cell discipline is a semantic commitment worth stating plainly:
**shared-cell state is global across recovery boundaries**, which is why a
write before a caught throw survives, **and will be global across
nondeterministic branches** if a row combines a shared-cell effect with
choice once multi-shot interpretation lands. Branch-local accumulation (each
branch forking its own state, purescript-run's `runAccum` family) is a
different discipline, threading the accumulator through interpretation by
value, and is planned as its own runner family; a handler that cannot be
expressed in threaded form can fall back to snapshot-and-restore over a
shared cell (capture the cell before a branch, restore it after).

Interpretation ordering today is fixed: there is one reference interpreter
whose elaboration policy makes the choices listed above (state shared across
`catch`, logs scoped by `censor` but not by `listen`, release before an abort
propagates). When the generic runner surface lands, handler order becomes
user-visible, and the semantics in this document are the defaults the
built-in catalog pins.

## purescript-run correspondence

The subsystem's first-order design follows purescript-run, so most names have
a direct analogue. The correspondence, with the right column naming the
reference catalog's spelling today:

| purescript-run                                        | Here                                                                                                              |
| ----------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `Run.State`: `get`, `put` (`gets`, `modify` derived)  | `State`: `get`, `put`; the derived forms compose from them.                                                       |
| `Run.State`: `runState`, `evalState`, `execState`     | The hand-written dispatch loop today; a generic runner family is planned.                                         |
| `Run.Reader`: `ask` (`asks` derived)                  | `Reader`: `ask`.                                                                                                  |
| `Run.Reader`: `local`                                 | `Local`: `local`, a separate higher-order effect rather than a runner-level combinator.                           |
| `Run.Writer`: `tell`                                  | `Writer`: `tell`.                                                                                                 |
| `Run.Writer`: `censor`                                | `Censor`: `censor`, a higher-order effect.                                                                        |
| `Run.Writer`: `foldWriter`, `runWriter`               | `Listen`: `listen` observes in-program; the fold-shaped runners are planned.                                      |
| `Run.Except`: `throw` (typed), `rethrow`, `runExcept` | `Except`: a typed `throw` (the catalog re-exports it as `throw_e`); its boundary reifies the abort to a `Result`. |
| `Run.Except`: `fail` (the unit error), `catch`        | `Throw`: `throw` (the unit-error abort); `Catch`: `catch`, recovering the bare throw only.                        |
| `Run.Choose`: `cempty`                                | `Empty`: `empty` (the dead-branch half of choice).                                                                |
| `Run.Choose`: `calt`, `runChoose`                     | Nondeterministic choice needs multi-shot interpretation and is planned.                                           |
| `Run`: `lift` / `send`                                | The row-generic smart constructors `define_effect!` emits.                                                        |
| `Run`: `peel` / `resume`                              | `Free::resume`, then `Coproduct::uninject` and `Coyoneda::lower` on the layer.                                    |
| `Run`: `interpret`, `run`, `runRec`                   | The hand-written dispatch loop (see the custom-effects guide); a generic `handle`/`handle_rec` family is planned. |
| `Run`: `expand` (row widening)                        | `embed` on the coproduct remainder.                                                                               |
| `Run`: the `runAccum` family                          | The planned threaded-accumulator runners (see the handler-state section).                                         |
| `Run.*`: the `*At` label variants (`askAt`, `tellAt`) | Planned label-brands (a tag is a wrapper brand that changes the dispatch key).                                    |

## Interpreting programs

The interpretation model today is the hand-written dispatch loop the
custom-effects guide teaches: `Free::resume` steps the program to its next
operation, `Coproduct::uninject` selects an effect by brand, `Coyoneda::lower`
unpacks the operation, and the handler does the work and resumes. The
built-in catalog's reference interpreter is the same loop at scale, one
brand-keyed arm per effect, with the elaboration choices and the abort
channel described above.

A generic, reusable runner surface (a brand-keyed handler list with
order-insensitive construction, plus the threaded-accumulator runners) is
planned; it changes how programs are run, not how effects are defined, so
everything in the catalog and the definition grammar carries forward
unchanged.

Async follows the same programs-as-data shape: a future is lifted into a row
as an effect (a boxed future behind a `Functor` brand), and an async driver
loop awaits it where a synchronous interpreter would dispatch it. The driver
mechanism is proven substrate-side and the full async round is planned.
