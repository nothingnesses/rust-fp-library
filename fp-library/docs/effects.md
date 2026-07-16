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

### Tagged (labelled) effects

Brand-keyed membership means one row cannot usefully hold the same effect
twice, since injection and selection both go by type. Tagging lifts this:
`TaggedBrand<Label, EBrand>` (`types::effects::tagged`) wraps an effect
brand under a label (any user-defined zero-sized type), and the wrapper's
identity is a new dispatch key while its kind projection reuses the
effect's own operations enum, so a tagged cell carries the same operations,
arms, and abort as its bare effect. The design follows purescript-run's
`*At` convention (`askAt`, `tellAt`) and heftia's label-resolved
membership: a tag is a wrapper brand that changes the dispatch key,
composing with brand-keyed dispatch rather than relying on positional
disambiguation.

The surface is emitted. `define_effect!` emits a labelled
`<name>_at<Label, ...>` constructor alongside each smart constructor,
injecting at the tagged brand; `define_row!`'s `#[handlers]` derives a
tagged cell's handler field from the label joined to the effect's stem
(`TaggedBrand<Fst, StateBrand<i32>>` derives `fst_state`), duplicate
derived names remaining expansion errors; and a step written for the bare
effect serves any labelled cell through the `tag_step` adapter, a nominal
wrapper, so a bare step passed to a runner keeps inferring the bare brand.
Two same-type states, end to end:

```rust
use {
	fp_library::{
		define_row,
		types::{
			Free,
			effects::{
				handle::RowHandler,
				state::{
					StateArms,
					StateBrand,
					get_at,
					put_at,
				},
				tagged::TaggedBrand,
			},
		},
	},
	std::cell::Cell,
};

/// The first label.
pub struct Fst;

/// The second label.
pub struct Snd;

define_row! {
	/// Two integer states, distinguished by label alone.
	#[handlers]
	pub row TwoStateRow {
		TaggedBrand<Fst, StateBrand<i32>>,
		TaggedBrand<Snd, StateBrand<i32>>,
	}
}

fn main() {
	// Each labelled constructor targets its own cell.
	let program: Free<TwoStateRow, i32> = put_at::<Fst, i32, _, _, _>(10).bind(|()| {
		get_at::<Fst, i32, _, _, _>()
			.bind(|x: i32| get_at::<Snd, i32, _, _, _>().bind(move |y: i32| Free::pure(x * 100 + y)))
	});

	// The handler fields derive from the labels: `fst_state`, `snd_state`.
	let fst = Cell::new(1);
	let snd = Cell::new(2);
	let handlers = TwoStateRowHandlers {
		fst_state: StateArms {
			get: Box::new(|| fst.get()),
			put: Box::new(|value| fst.set(value)),
		},
		snd_state: StateArms {
			get: Box::new(|| snd.get()),
			put: Box::new(|value| snd.set(value)),
		},
	};
	assert_eq!(handlers.handle(program).ok(), Some(1002));
	assert_eq!(fst.get(), 10);
	assert_eq!(snd.get(), 2);
}
```

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
from silently swallowing failure kinds it was never meant to handle. The
`#[handlers]` handler surface realises the same shape per row: `define_row!`
emits an abort union with one variant per cell, each carrying its effect's
no-resume payloads, so a row's failure kinds stay distinct data there too.

### The store axis and the single-shot guard

The `Free` substrate carries a `Store` parameter selecting how continuations
are stored: the default `Box` store holds `FnOnce` continuations, so each
continuation can be called at most once (single-shot), while the `Rc`/`Arc`
stores hold re-callable `Fn` continuations for multi-shot interpretation.
Scoped nondeterministic choice is expressible under the single-shot guard:
the built-in `Choose` owns its two branch sub-programs and resumes exactly
once, with the values of the branches that survived, so per-branch
distribution of the continuation is expressed program-side, by placing it
inside the owned branches. What the guard rules out is re-entering one
continuation once per branch (the first-order `alt` style), which the
multi-shot tier below exists to serve. The custom-effects guide documents
the same guard for hand-written interpreters.

### The multi-shot tier

Interpretation on the multi-shot stores mirrors the `Box` tier's narrowing
vocabulary under `types::effects::handle::multi_shot`: its `handle_accum`
eliminates one cell from a row and re-emits every unmatched layer into the
residual row, runners stack, and `extract` closes a fully narrowed
pipeline; one body serves `Rc` and `Arc` generically over `MultiShotStore`.
The module path is the store-axis marker, and the same marking carries to
methods: program construction on a multi-shot store chains with
`bind_multi_shot`/`map_multi_shot`, keeping `bind`/`map` unique to the
`Box` tier so that a program whose store is still an inference variable
resolves to the single-shot default without annotation.

Effects opt into re-callable continuations per operation: a `#[multi_shot]`
operation's cell stores its continuation as `Rc<dyn Fn>` instead of
`Box<dyn FnOnce>`, so a forking interpreter may lower the cell once and
call the captured continuation once per branch; the `Free` spine cooperates
because the multi-shot `to_view` clones the continuation queue into each
re-entry. Such effects emit no handler pieces (the one-pass `#[handlers]`
surface is single-shot by construction, so placing one in a `#[handlers]`
row is a compile error) and are interpreted by the narrowing runners and a
forking fold instead. Two semantics are pinned by the round that shipped
the tier: a threaded accumulator re-emitted into a forking cell is cloned
into each branch at the fork, so every branch continues from the
accumulator as of capture (fork-the-accumulator, not shared mutation), and
`Bracket` stays excluded from multi-shot rows (release-on-abort and
re-entry have no agreed composition, so the combination is not offered
rather than guarded at run time). The public `Alt` effect and its
`handle_alt`/`handle_alt_first` runners are this tier's first consumer;
the forking `shift` primitive is the planned next one.

The scoped `Choose` cell and the first-order `Alt` cell are deliberately
unbridged: no elaboration rewrites one into the other. The scoped cell
pins `Box`-store branch programs in its operations enum and resumes
exactly once with the survivor collection, so rewriting it into an
emitted `alt` cannot typecheck (a forking runner distributes the
downstream continuation per branch, while the scoped continuation
consumes the whole collection at once), and its `Box`-store branches
cannot ride or convert to the `Rc` spine a forking fold needs (a stored
`FnOnce` continuation cannot become a re-callable `Fn`). The delimited
interpretation that remains, running both owned branches once and
resuming once with the survivors, is exactly `handle_choose`. Choose by
store and semantics: owned-branch, resume-once choice on the single-shot
store interprets through the `handle_choose` family; re-entrant
per-branch choice on the multi-shot store is the `Alt` effect.

## The built-in reference catalog

The library carries a catalog of nineteen built-in effects, and every one
but `Shift` is a `define_effect!` invocation, which makes the catalog the
macro's permanent conformance suite; `Shift` is hand-written (its capture
body receives the reified continuation, a payload outside the macro's
operation grammar), making it the catalog's exemplar of the hand-written
path the custom-effects guide documents. Six are public and
payload-generalised, shipping with their narrowing runners: `State<S>`
(`types::effects::state`), `Writer<W>` (`types::effects::writer`), the
scoped choice `Choose<RAction>` (`types::effects::choose`), the first-order
choice `Alt` (`types::effects::alt`, whose `#[multi_shot]` `alt` forks
under the `Rc`-store runners `handle_alt` and `handle_alt_first`), the
yielding `Coroutine<Out, In>` (`types::effects::coroutine`, with the
streaming vocabulary over it in `types::effects::streaming`), and the
one-shot delimited continuation `Shift<Narrow, Ans, V>`
(`types::effects::shift`). The rest are
crate-internal reference fixtures, their payloads or row pins held at
concrete types (an `i32` environment, a `String` log) that keep the reference
interpreter's test oracle simple; each goes public as its runner story lands.
Until then, the catalog documents the reference semantics, and a program that
wants an internal built-in's behaviour defines the same shape in its own
crate, exactly as the custom-effects guide shows, since the definitions below
compile against the public macros unchanged.

The definitions are ordinary `define_effect!` invocations. A representative
first-order slice of the catalog, exactly as the built-ins define it (with
`State`'s and `Writer`'s payloads pinned at the row, the parameterise-and-pin
convention the built-ins use for every generic effect):

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
	/// Writer over a log of `W`. `tell` appends to the log.
	#[handler_state(shared_by_reference)]
	pub effect Writer<W: 'static> {
		/// Append `value` to the log.
		fn tell(value: W) -> ();
	}
}

define_row! {
	/// A row of the three effects, `State`'s cell pinned to `bool` and
	/// `Writer`'s to `String`.
	pub row AppRow {
		StateBrand<bool>,
		ThrowBrand,
		WriterBrand<String>,
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
	// all-or-nothing, so the inferred row and store parameters are written `_`.
	let aborting: Free<AppRow, i32> = throw::<i32, _, _, _>();
	let program: Free<AppRow, i32> = catch(aborting, || Free::pure(42));
	assert!(program.resume().is_err());
}
```

The full catalog, with each effect's operations, its declared
`#[handler_state(...)]` class, and its reference semantics:

| Effect                  | Operations                                                            | Handler state         | Reference semantics                                                                                                                                                                                                |
| ----------------------- | --------------------------------------------------------------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `State<S>`              | `get() -> S`, `put(value: S) -> ()`                                   | `shared_by_reference` | Reads and writes the shared state cell.                                                                                                                                                                            |
| `Reader`                | `ask() -> i32`                                                        | `scoped_by_value`     | Reads the environment; `Local` scopes it.                                                                                                                                                                          |
| `Writer<W>`             | `tell(value: W) -> ()`                                                | `shared_by_reference` | Appends to the log; the log is append-only.                                                                                                                                                                        |
| `Fresh`                 | `fresh() -> usize`                                                    | `shared_by_reference` | Yields the next counter value; the successor policy is handler state.                                                                                                                                              |
| `Input`                 | `input() -> Option<&'static str>`                                     | `shared_by_reference` | Drains a queue; `None` once empty.                                                                                                                                                                                 |
| `KVStore`               | `lookup(key) -> Option<i32>`, `update(key, value: Option<i32>) -> ()` | `shared_by_reference` | Map read; `Some` inserts or overwrites, `None` deletes.                                                                                                                                                            |
| `Throw`                 | `throw() -> !`                                                        | `none`                | Bare abort; the one case `Catch` recovers.                                                                                                                                                                         |
| `Empty`                 | `empty() -> !`                                                        | `none`                | Dead branch; propagates through `Catch`. Scoped pruning lives in `Choose`'s own `empty`; the first-order branch-pruning form is the public `Alt` effect's `empty` under the forking runners.                       |
| `Except<E>`             | `throw(error: E) -> !`                                                | `none`                | Typed abort carried in the return channel; recovered at its own boundary, propagates through `Catch`.                                                                                                              |
| `Identity`              | `identity_op(value: i32) -> i32`                                      | `none`                | Value echo; the no-op target interposition rewrites.                                                                                                                                                               |
| `Catch<RAction>`        | `catch(action, recover) -> RAction`                                   | `none`                | Recovers a bare `Throw` only; state written before a caught throw survives.                                                                                                                                        |
| `Local<Env, RAction>`   | `local(modify, action) -> RAction`                                    | `none`                | Runs the action under `modify(env)`; the scope ends with the action.                                                                                                                                               |
| `Listen<RAction, W>`    | `listen(action) -> (RAction, W)`                                      | `none`                | Runs under the same log, observing the delta; the action's writes are preserved.                                                                                                                                   |
| `Censor<W, RAction>`    | `censor(f, action) -> RAction`                                        | `none`                | Fresh local log, then `f(total)` emitted to the outer log; transactional on abort.                                                                                                                                 |
| `Bracket<Res, RBody>`   | `bracket(acquire, body, release) -> RBody`                            | `none`                | Acquire, use, release in order; a body abort still releases.                                                                                                                                                       |
| `Choose<RAction>`       | `choose(left, right) -> Vec<RAction>`, `empty() -> !`                 | `none`                | Runs both owned branches once each; resumes once with the surviving values in branch order; `empty` kills the branch.                                                                                              |
| `Coroutine<Out, In>`    | `yield_value(output: Out) -> In`                                      | `none`                | Yields an `Out` to the runner, resumes with an `In`; the streaming vocabulary's producer and consumer are its pins.                                                                                                |
| `Shift<Narrow, Ans, V>` | `shift(body) -> V`                                                    | `none`                | One-shot delimited continuation: the body receives the captured continuation and produces the answer program over the residual row; dropping the continuation aborts the unresumed tail. Hand-written (see above). |

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
on). It is crate-internal today, in the same status as the internal catalog.

## Handler state and ordering

Every effect declares how a handler holds its state, via the mandatory
`#[handler_state(...)]` attribute:

- `none`: the handler needs no state (the aborting effects, the pure
  higher-order elaborations).
- `scoped_by_value`: the state is a value copied into a scope, so mutation
  inside the scope cannot leak out (`Reader`'s environment under `Local`).
- `shared_by_reference`: the state is a shared cell the handler reads and
  writes in place (`State`, `Writer`, `Fresh`, `Input`, `KVStore`).
- `threaded_by_value`: the state is an accumulator threaded through
  interpretation by value (the discipline of the accumulator runner family
  below). No built-in declares it: the built-ins declare their shared-cell
  one-pass discipline and gain by-value threading through their runners
  instead.

The shared-cell discipline is a semantic commitment worth stating plainly:
**shared-cell state is global across recovery boundaries**, which is why a
write before a caught throw survives, **and is global across nondeterministic
branches** when a row combines a shared-cell effect with `Choose` under the
one-pass handler surface, since the re-entered branches share the arms'
cells. Branch-local accumulation (each branch forking its own state,
purescript-run's `runAccum` family) is the threaded discipline, shipped as
the accumulator runner family: `handle_accum` threads an accumulator through
one effect's interpretation, `handle_state` and `fold_writer`/`handle_writer`
are its per-effect forms, and `handle_choose_accum` forks the accumulator per
branch. A handler that cannot be expressed in threaded form can fall back to
snapshot-and-restore over a shared cell (capture the cell before a branch,
restore it after).

Handler ordering is user-visible in the runner family: narrowing runners
stack, and which runner sits outside decides the semantics. Eliminating
choice first and stacking an accumulator's runner outside threads one
accumulator through all branches in sequence (the global ordering), while
`handle_choose_accum` forks the accumulator per branch (the branch-local
ordering); both stackings are meaningful, and the choice belongs to the
program author. Under the one-pass handler surface the elaboration policy is
the arms': a higher-order arm decides what state its re-entries share. The
crate-internal reference interpreter makes the choices listed above (state
shared across `catch`, logs scoped by `censor` but not by `listen`, release
before an abort propagates), and those are the defaults the built-in catalog
pins.

## purescript-run correspondence

The subsystem's first-order design follows purescript-run, so most names have
a direct analogue. The correspondence, with the right column naming the
reference catalog's spelling today:

| purescript-run                                        | Here                                                                                                                                                               |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `Run.State`: `get`, `put` (`gets`, `modify` derived)  | `State`: `get`, `put`; the derived forms compose from them.                                                                                                        |
| `Run.State`: `runState`, `evalState`, `execState`     | `handle_state`, yielding the final state paired with the result; `evalState`/`execState` are its projections.                                                      |
| `Run.Reader`: `ask` (`asks` derived)                  | `Reader`: `ask`.                                                                                                                                                   |
| `Run.Reader`: `local`                                 | `Local`: `local`, a separate higher-order effect rather than a runner-level combinator.                                                                            |
| `Run.Writer`: `tell`                                  | `Writer`: `tell`.                                                                                                                                                  |
| `Run.Writer`: `censor`                                | `Censor`: `censor`, a higher-order effect.                                                                                                                         |
| `Run.Writer`: `foldWriter`, `runWriter`               | `fold_writer` and `handle_writer`; `Listen`: `listen` observes in-program.                                                                                         |
| `Run.Except`: `throw` (typed), `rethrow`, `runExcept` | `Except`: a typed `throw` (the catalog re-exports it as `throw_e`); its boundary reifies the abort to a `Result`.                                                  |
| `Run.Except`: `fail` (the unit error), `catch`        | `Throw`: `throw` (the unit-error abort); `Catch`: `catch`, recovering the bare throw only.                                                                         |
| `Run.Choose`: `cempty`                                | `Choose`: `empty` kills a branch; the bare `Empty` effect is the catalog's first-order form.                                                                       |
| `Run.Choose`: `calt`, `runChoose`                     | The scoped `Choose` cell and its `handle_choose` family; the first-order `calt`/`runChoose` pair is the `Alt` effect's `alt` with `handle_alt`/`handle_alt_first`. |
| `Run`: `lift` / `send`                                | The row-generic smart constructors `define_effect!` emits.                                                                                                         |
| `Run`: `peel` / `resume`                              | `Free::resume`, then `Coproduct::uninject` and `Coyoneda::lower` on the layer.                                                                                     |
| `Run`: `interpret`, `run`, `runRec`                   | The `#[handlers]` handler surface (`RowHandler::handle`); the hand-written loop remains the documented fallback.                                                   |
| `Run`: `expand` (row widening)                        | `embed` on the coproduct remainder.                                                                                                                                |
| `Run`: the `runAccum` family                          | `handle_accum` and the per-effect runners built on it (see the handler-state section).                                                                             |
| `Run.*`: the `*At` label variants (`askAt`, `tellAt`) | The emitted `*_at` labelled constructors (`get_at::<Fst, i32, _, _, _>()`) over `TaggedBrand<Label, EBrand>` cells.                                                |

## Interpreting programs

The interpretation surface has two tiers, with a hand-written fallback under
them; the custom-effects guide walks through all three.

The one-pass handler surface is the full-elimination convenience. Marking a
row `#[handlers]` makes `define_row!` emit a handler struct (one closure-arm
bundle per cell, composed from the pieces `define_effect!` emits for every
effect), the row's abort union, and a `RowHandler` implementation whose
`handle` method drives any program over the row to `Result<T, RowAbort>`.
Dispatch is brand-keyed, construction is an order-insensitive struct literal,
and one handler value drives programs at every result type, which is what
lets a higher-order arm re-enter interpretation on its owned sub-programs.

The narrowing runners are the per-effect tier: each eliminates one effect
from the row and returns the residual program over the narrowed row, so
runners stack, and `extract` finishes a fully narrowed program. The generic
core is `handle_accum`, which threads an accumulator through one effect's
interpretation; `handle_state`, `fold_writer` and `handle_writer`, and the
`handle_choose` family (collecting, first-success, and accumulator-forking)
are the shipped per-effect forms. Stacking order is a semantics choice (see
the handler-state section above). A tagged cell eliminates the same way,
one label at a time: `tag_step` adapts a bare effect's step to the labelled
brand, so `handle_accum` at `TaggedBrand<Fst, StateBrand<i32>>` narrows the
`Fst` cell and leaves the `Snd` cell in the residual row.

The hand-written dispatch loop remains the general fallback, and is what rows
holding hand-written effect cells use: `Free::resume` steps the program to
its next operation, `Coproduct::uninject` selects an effect by brand,
`Coyoneda::lower` unpacks the operation, and the handler does the work and
resumes. The built-in catalog's reference interpreter is the same loop at
scale, one brand-keyed arm per effect, with the elaboration choices and the
abort channel described above.

Async follows the same programs-as-data shape: `await_future`
(`types::effects::await_future`) embeds a future into a row as the `Await`
base-lift cell (a boxed future behind a `Functor` brand), and `run_async`
is the terminal driver, `extract`'s async sibling: it drives a program
whose row's only cell is `Await`, lowering each layer to a future of the
next program and awaiting it, with the continuation held as data. Mixed
rows need no async-aware runners: a narrowing runner re-emits unmatched
`Await` cells lazily (the rest of its fold rides inside the re-emitted
continuation), so stacking the sync runners over a mixed row leaves the
`Await`-only residual `run_async` finishes, and handler work interleaves
between suspensions. The returned future is runtime-agnostic; any executor
drives it. The boxed future is local (non-`Send`), targeting single-shot
programs. For multi-shot programs the adopted mechanism is thunked
re-await (a re-entered continuation re-runs its awaited operation, with
memoize-once the opt-in cache when re-running would duplicate work), its
implementation waiting on a consumer; the thread-safe async family remains
the planned async round.
