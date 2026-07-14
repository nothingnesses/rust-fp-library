# Custom effects

The effects subsystem lets you describe a program as data, a value of type
`Free<Row, A>`, and interpret it however you like. This guide shows how to
define your own effects with the `define_effect!` and `define_row!` macros,
build programs against them, and interpret those programs, from the emitted
handler surface down to a hand-written dispatch loop. Everything here uses
only `fp_library`'s public API, so a custom effect you define in your own
crate works exactly like the examples below.

The subsystem is behind the optional `effects` feature; enable it in your
`Cargo.toml`:

```toml
[dependencies]
fp-library = { version = "*", features = ["effects"] }
```

## The model

An effectful program is built from three pieces:

- **Effects** declare the operations a program may perform. `define_effect!`
  turns a block of operation signatures into a _brand_ (a zero-sized type that
  names the effect), an operations enum, and one smart constructor per
  operation. A constructor call is a one-operation program.
- **A row** is the set of effects a program may use, encoded as one
  `Coyoneda`-wrapped coproduct. `define_row!` turns a list of effect brands
  into a nominal row type. Programs are values of `Free<Row, A>`.
- **An interpreter** gives the operations meaning. It steps the program one
  operation at a time, and for each operation does the real work (print, read
  a cell, push to a log) and resumes the rest of the program with the result.
  The primary form is emitted: marking the row `#[handlers]` makes
  `define_row!` emit a handler struct, one closure arm per operation, whose
  `handle` method runs any program over the row (the quick start below).
  Narrowing runners eliminate one effect at a time, and a small hand-written
  dispatch loop is the general fallback; all three are shown in this guide.

## Quick start

Here is a one-effect program end to end: a `Console` effect with a single
`print_line` operation, a `#[handlers]` row containing just that effect, a
two-line program, and an emitted handler that collects the output into a
`Vec<String>`.

```rust
use fp_library::{
	define_effect,
	define_row,
	types::{
		Free,
		effects::handle::RowHandler,
	},
};

define_effect! {
	/// A tiny console output effect.
	#[handler_state(shared_by_reference)]
	pub effect Console {
		/// Emit a line of output, then continue.
		fn print_line(text: String) -> ();
	}
}

define_row! {
	/// The row this example's programs are written against; `#[handlers]`
	/// additionally emits the row's handler surface.
	#[handlers]
	pub row AppRow {
		ConsoleBrand,
	}
}

// A program is an ordinary value. The smart constructors are row-generic, so
// the target row is inferred from the annotation; `bind` sequences operations.
fn greeting() -> Free<AppRow, ()> {
	print_line("hello".to_string()).bind(|()| print_line("world".to_string()))
}

fn main() {
	// One closure arm per operation: the arm receives the operation's
	// payloads and returns its resume value; the emitted loop steps the
	// program, applies the arm, and resumes the continuation itself.
	let output = std::cell::RefCell::new(Vec::new());
	let handlers = AppRowHandlers {
		console: ConsoleArms {
			print_line: Box::new(|text| output.borrow_mut().push(text)),
		},
	};
	assert_eq!(handlers.handle(greeting()).ok(), Some(()));
	assert_eq!(*output.borrow(), vec!["hello".to_string(), "world".to_string()]);
}
```

The handler is the only piece you write, and it is one closure per operation:
`handle` steps the program to each operation, selects that operation's effect
by brand, applies your arm to its payloads, and resumes the continuation with
the arm's result, returning the program's value or the row's abort (explained
in [The handler surface](#the-handler-surface); this row has no aborting
operations, so the error case is unreachable). Adding an operation to
`Console` adds a field to `ConsoleArms`, and the struct literal stops
compiling until you supply it, so a missing case is a compile error, not a
silent bug.

## The operation grammar

Each `fn` line inside `define_effect!` is one operation. Everything the macro
emits is derived from these signatures, so the signature is the whole spec.

- **The return type is the resume value**, the value the interpreter hands back
  to continue the program. `fn print_line(text: String) -> ()` resumes with
  `()`; `fn read_line() -> String` resumes with a `String`. The emitted variant
  stores that continuation as `Box<dyn FnOnce(Resume) -> A + 'a>`.
- **`-> !` is a no-resume operation.** An operation that never returns (an
  abort) is written `fn fail() -> !;`. Its variant stores `PhantomData` instead
  of a continuation, and its constructor is polymorphic in the result type
  (the program never continues, so the operation stands in any position). The
  built-in `Throw` is exactly this shape.
- **A `Program<T>` payload makes the effect higher-order.** An operation can own
  a sub-program, written `Program<T>` (a `Free<Row, T>` over the same row). Its
  presence adds a row type parameter to the brand, and the interpreter runs the
  sub-program itself (see [Higher-order effects](#higher-order-effects)).
- **An `impl FnOnce(Args...) -> Ret` payload is a stored callable**, boxed as
  `Box<dyn FnOnce(Args...) -> Ret + 'a>`. `Ret` may itself be `Program<T>` (a
  callable that returns a sub-program, like a recovery handler).
- **`#[handler_state(...)]` is required** and declares, as documentation, how a
  handler holds state for the effect: `none`, `scoped_by_value`,
  `shared_by_reference`, or `threaded_by_value`. It is emitted into the brand's
  docs so every effect states its state discipline; it does not change the
  emitted types.
- **`#[crate_path(...)]` is optional** and overrides the crate the emitted code
  refers to (default `::fp_library`). You need it only inside `fp_library`
  itself.

Names are used verbatim: an operation `fn foo_bar` keeps the constructor name
`foo_bar` and derives the variant `FooBar` (snake_case to UpperCamelCase). The
generic parameter names `R`, `I`, `A`, `B`, `Label` and the payload name `k` are
reserved by the emission, and derived-name collisions are compile errors, never
silently renamed (an operation named `foo_bar_at` next to an operation named
`foo_bar` is rejected, since the latter's labelled constructor takes that name).

## What `define_effect!` emits

For an effect `Name`, the macro emits: a brand `NameBrand`; an operations enum
`NameF` with one variant per operation (first-order operations are tuple
variants ending in the continuation or `PhantomData`, higher-order operations
are named-field cells including the continuation field `k`); the kind projection
that lets `NameBrand` sit in a row; a `Functor` instance that threads a mapped
function through each continuation; an `OrderOf` marker computed from the
operations (first-order unless some operation owns a sub-program); one
row-generic smart constructor per operation, each alongside its labelled
`<name>_at<Label, ...>` variant injecting at `TaggedBrand<Label, ...>` so a
row can hold the effect once per label (tagged effects; the effects guide's
design section has the worked example); and the effect's handler pieces,
an arm bundle `NameArms` (one boxed closure field per resumptive operation; a
unit struct when there are none) and an abort type `NameAbort` (one variant
per no-resume operation, carrying its payloads; uninhabited when every
operation resumes), which a `#[handlers]` row composes into its handler
surface (see [The handler surface](#the-handler-surface)).

The constructors are **row-generic**: `fn print_line<R, I>(text: String) ->
Free<R, ()>`, bounded so the operation injects into any row `R` that contains
it. The row is normally inferred from context (a type annotation, or the
function the program is passed to). Because the extra parameters `R` and `I` are
inferred, a partial turbofish does not compile: write `print_line("x".into())`
and let inference pick the row, or, when you must name the result type of a
no-resume operation, give all the parameters (`fail::<i32, _, _>()`), since
Rust's turbofish is all-or-nothing.

## Higher-order effects

An operation whose payload is a `Program<T>` owns a sub-program, which makes the
effect _higher-order_: the interpreter does not just resume a value, it runs the
owned sub-program (possibly under modified handler state) and threads its result
onward. The built-in `Catch` is the canonical example; its operation is

```text
fn catch(action: Program<T>, recover: impl FnOnce() -> Program<T>) -> T;
```

so its cell owns the `action` sub-program and a `recover` callable that returns
a sub-program. Because the cell stores `Free<Row, _>` over the very row being
defined, a row that contains a higher-order effect must be a **nominal** row
(a named type from `define_row!`, not a type alias): the row refers to itself
through the brand's kind projection, which is well-founded, whereas a
self-referential type alias is a definition cycle. `define_row!` always emits a
nominal row, so this is handled for you. Interpreting a higher-order effect
means calling your interpreter recursively on the owned sub-program; native
stack use grows with the nesting depth of higher-order operations, not with
program length. In the handler surface, a higher-order operation's arm
receives the owned sub-programs and a re-entry handle instead of plain
payloads (see [The handler surface](#the-handler-surface)).

## The handler surface

Marking a row `#[handlers]` makes `define_row!` emit three things alongside
the row: a handler struct `<Row>Handlers<'h>` with one field per cell, named
after the effect in snake_case and typed at that effect's arm bundle; an
abort union `<Row>Abort` with one variant per cell, named after the effect in
PascalCase and carrying that effect's abort type; and a `RowHandler`
implementation whose `handle` method is the whole interpretation loop:
`handle(program: Free<Row, T>) -> Result<T, RowAbort>`. A tagged cell's
field and variant names derive from the label joined to the effect's stem
(`TaggedBrand<Fst, StateBrand<i32>>` derives the field `fst_state`), so a
row holding one effect under several labels gets one distinctly named arm
bundle per label; duplicate derived names are expansion errors.

The arm grammar follows the operation grammar:

- **A resumptive operation** (an ordinary `-> Type` return) gets a closure arm
  from its payloads to its resume value. The loop applies the continuation
  itself, so the arm never sees it.
- **A no-resume operation** (`-> !`) gets no arm at all. Reaching it ends
  interpretation: the operation's payloads are reified into the effect's
  abort variant and returned as the row abort, so `handle`'s `Err` carries
  exactly which operation aborted and with what. An effect whose operations
  all resume contributes an uninhabited abort variant, and an effect with no
  resumptive operations has a unit arm bundle.
- **A higher-order operation** (one owning `Program<T>` payloads) gets an
  elaboration arm: it receives the owned sub-programs, any ordinary payloads,
  and a re-entry handle, and returns `Result<ResumeValue, RowAbort>`. The
  re-entry handle runs a sub-program under the same handler value and returns
  `Result<T, RowAbort>`, so the arm decides what to do with each
  sub-program's outcome: recover some aborts selectively, propagate others.

No arm mentions the program's result type, so one handler value drives
programs at every result type; that is also what lets an elaboration arm
re-enter interpretation on its owned sub-programs. The arms are `Fn` closures
(the loop may apply an arm many times), so handler state lives in shared
cells the arms borrow, as `state` does below. Construction is a plain struct
literal, order-insensitive and checked: a misspelled or missing cell name is
a compile error naming the available fields.

Here is a two-effect row exercising the first two arm kinds, a resumptive
counter and a payload-carrying abort:

```rust
use fp_library::{
	define_effect,
	define_row,
	types::{
		Free,
		effects::handle::RowHandler,
	},
};

define_effect! {
	/// A counter the program can advance.
	#[handler_state(shared_by_reference)]
	pub effect Counter {
		/// Advance the count by `amount`, resuming with the new count.
		fn advance(amount: i32) -> i32;
	}
}

define_effect! {
	/// Abort the whole computation with a reason.
	#[handler_state(none)]
	pub effect Fail {
		/// Abort with `reason`; never resumes.
		fn fail(reason: &'static str) -> !;
	}
}

define_row! {
	/// The counter alongside the abort.
	#[handlers]
	pub row AppRow {
		CounterBrand,
		FailBrand,
	}
}

fn main() {
	let count = std::cell::Cell::new(0);
	// `Fail` has no resumptive operations, so its arm bundle is a unit
	// struct: there is nothing to write for an operation that never resumes.
	let handlers = AppRowHandlers {
		counter: CounterArms {
			advance: Box::new(|amount| {
				count.set(count.get() + amount);
				count.get()
			}),
		},
		fail: FailArms,
	};

	// A successful program yields its value.
	let program: Free<AppRow, i32> = advance(2).bind(|_| advance(3));
	assert_eq!(handlers.handle(program).ok(), Some(5));

	// A no-resume operation ends interpretation with the row abort, which
	// carries the operation's payloads; writes made before it survive in
	// the handler state.
	let aborting: Free<AppRow, i32> = advance(1).bind(|_| fail::<i32, _, _>("boom"));
	assert!(matches!(
		handlers.handle(aborting),
		Err(AppRowAbort::Fail(FailAbort::Fail("boom")))
	));
	assert_eq!(count.get(), 6);
}
```

For the third arm kind, the built-in scoped choice `Choose` is the canonical
example: its `choose` operation owns two branch sub-programs, so its arm
receives them with a re-entry handle and resumes with the values of the
branches that survived, recovering the cell's own branch-death abort while
propagating every other abort untouched:

```text
choose: Box::new(|left, right, reenter| {
	let mut survivors = Vec::new();
	for branch in [left, right] {
		match reenter(branch) {
			Ok(value) => survivors.push(value),
			Err(AppRowAbort::Choose(ChooseAbort::Empty)) => {}
			Err(other) => return Err(other),
		}
	}
	Ok(survivors)
})
```

The surface is opt-in because it composes from the pieces `define_effect!`
emits: a row holding a hand-written effect cell (the appendix pattern) lacks
those pieces, so it uses the plain `define_row!` form and one of the
interpretation styles below. The emitted abort and arm types carry no derives
(payloads may be callables), so tests assert on them with `.ok()` and
`matches!` rather than `assert_eq!` on the `Result`. Each arm is one boxed
closure, so dispatch costs one indirect call per operation; the hand-written
loop below is the direct-dispatch alternative when that matters.

## Narrowing runners

The second interpretation style eliminates one effect at a time. A narrowing
runner consumes a program over a row and returns the residual program over
the row without the eliminated cell, so runners stack, and `extract` finishes
a program whose row has been narrowed to empty. The generic core is
`handle_accum`, which threads an accumulator through the interpretation of
one effect; its step sees each lowered operation of that effect and returns
the new accumulator with the continuation:

```rust
use fp_library::{
	brands::CNilBrand,
	define_effect,
	define_row,
	types::{
		Free,
		effects::handle::{
			extract,
			handle_accum,
		},
	},
};

define_effect! {
	/// A running total.
	#[handler_state(threaded_by_value)]
	pub effect Counter {
		/// Add `amount` to the total, resuming with the new total.
		fn add(amount: i32) -> i32;
	}
}

define_row! {
	/// The one-cell row.
	pub row CounterRow {
		CounterBrand,
	}
}

fn main() {
	let program: Free<CounterRow, i32> = add(2).bind(|_| add(3));
	// The accumulator is threaded by value, no shared cell: the step
	// returns the new total alongside the resumed continuation. The
	// eliminated brand is named in the turbofish; everything else is
	// inferred.
	let narrowed: Free<CNilBrand, (i32, i32)> =
		handle_accum::<CounterBrand, _, _, _, _, _, _>(0, program, |s, op| match op {
			CounterF::Add(amount, resume) => (s + amount, resume(s + amount)),
		});
	assert_eq!(extract(narrowed), (5, 5));
}
```

This style works for any effect with a `Functor` and a kind projection,
including hand-written ones, since it never touches the handler pieces. The
public built-ins ship their per-effect forms (`handle_state`, `fold_writer`
and `handle_writer`, and the `handle_choose` family); the effects overview
(`docs/effects.md`) catalogs them and explains why runner stacking order is a
semantics choice.

## The hand-written loop

The general fallback, and the style rows holding hand-written effect cells
use, is a small dispatch loop over the primitives: `resume` steps the program
to its next operation, `uninject` selects an effect's cell out of the row
layer, `lower` unpacks the operation enum, and the loop does the work and
resumes. Here is the quick-start program interpreted by hand:

```rust
use fp_library::{
	define_effect,
	define_row,
	types::{
		Coyoneda,
		Free,
	},
};

define_effect! {
	/// A tiny console output effect.
	#[handler_state(shared_by_reference)]
	pub effect Console {
		/// Emit a line of output, then continue.
		fn print_line(text: String) -> ();
	}
}

define_row! {
	/// The row this example's programs are written against.
	pub row AppRow {
		ConsoleBrand,
	}
}

fn greeting() -> Free<AppRow, ()> {
	print_line("hello".to_string()).bind(|()| print_line("world".to_string()))
}

// A hand-written interpreter: step the program, dispatch each Console
// operation, resume the continuation with the operation's result.
fn run(program: Free<AppRow, ()>) -> Vec<String> {
	let mut output = Vec::new();
	let mut program = program;
	loop {
		// `resume` runs the program to its next operation. `Ok` is the final
		// value; `Err` is one suspended operation (a row layer).
		let layer = match program.resume() {
			Ok(()) => return output,
			Err(layer) => layer,
		};
		// Select the `Console` operation out of the row. This row holds only
		// `Console`, so the remainder is the uninhabited terminal row.
		let selected: Result<Coyoneda<'static, ConsoleBrand, Free<AppRow, ()>>, _> =
			layer.uninject();
		let operation = match selected {
			Ok(operation) => operation,
			Err(terminal) => match terminal {},
		};
		// `lower` recovers the operation enum. Do the work, then resume.
		match operation.lower() {
			ConsoleF::PrintLine(text, resume) => {
				output.push(text);
				program = resume(());
			}
		}
	}
}

fn main() {
	assert_eq!(run(greeting()), vec!["hello".to_string(), "world".to_string()]);
}
```

For a row with several effects, chain the `uninject` calls, each peeling one
brand and passing the remainder on, until the terminal row
(`match remainder {}`) proves every case is handled. Dispatch is by brand,
not by position, so the arms may be written in any order. Adding an operation
adds a variant to the operations enum, and the `match` stops compiling until
you handle it, the same totality the handler struct gives you.

All three styles interpret on the single-shot `Box` store: each continuation
is called at most once. Scoped nondeterministic choice is expressible under
that guard, because the built-in `Choose` owns its branch sub-programs and
resumes exactly once with the surviving values; what the guard rules out is
re-entering one continuation once per branch, which waits for the multi-shot
stores. That round changes how such effects are run, not how effects are
defined.

## Appendix: the manual pattern

`define_effect!` writes the effect definition so you do not have to, but the
generated shape is ordinary code. Here is the `Console` effect hand-written,
which is what the macro emits (the row still uses `define_row!`): a brand, an
operations enum, the kind projection via `impl_kind!`, a `Functor` instance, an
`OrderOf` marker, and a smart constructor (pinned to the row here for brevity,
where the macro emits the row-generic form). A hand-written cell carries no
handler pieces, so its row stays in the plain `define_row!` form (no
`#[handlers]`) and is interpreted with a narrowing runner or the hand-written
loop.

```rust
use fp_library::{
	Apply,
	Kind,
	define_row,
	impl_kind,
	classes::Functor,
	types::{
		Coyoneda,
		Free,
		effects::{
			coproduct::Coproduct,
			order::{
				FirstOrder,
				OrderOf,
			},
		},
	},
};
// `impl_kind!` names the generated kind trait by its internal (hash) name, so
// the hand-written kind projection needs the `kinds` glob in scope. The macro
// wraps its own emission in this glob for you; this is one of the chores it
// removes.
use fp_library::kinds::*;

// The brand names the effect; the operations enum has one variant per
// operation, ending in the continuation.
pub struct ConsoleBrand;
pub enum ConsoleF<'a, A> {
	PrintLine(String, Box<dyn FnOnce(()) -> A + 'a>),
}

// The kind projection lets the brand sit in a row cell.
impl_kind! {
	impl for ConsoleBrand {
		type Of<'a, A: 'a>: 'a = ConsoleF<'a, A>;
	}
}

// The Functor instance threads a mapped function through the continuation.
impl Functor for ConsoleBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			ConsoleF::PrintLine(text, k) => ConsoleF::PrintLine(text, Box::new(move |u| f(k(u)))),
		}
	}
}

// No operation owns a sub-program, so the effect is first-order.
impl OrderOf for ConsoleBrand {
	type Order = FirstOrder;
}

define_row! {
	/// The row over the hand-written effect.
	pub row AppRow {
		ConsoleBrand,
	}
}

// The smart constructor builds the one-operation program: lift the operation
// into `Coyoneda`, inject it into the row cell, and wrap it as a `Free` layer.
pub fn print_line(text: String) -> Free<AppRow, ()> {
	let coyo: Coyoneda<'static, ConsoleBrand, ()> =
		Coyoneda::lift(ConsoleF::PrintLine(text, Box::new(|u| u)));
	let node: Apply!(<AppRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>) =
		Coproduct::inject(coyo);
	Free::lift_f(node)
}

fn main() {
	// The hand-written constructor builds the same program the macro's would.
	let program: Free<AppRow, ()> = print_line("hello".to_string());
	assert!(program.resume().is_err());
}
```

Reading the two side by side shows what the macro buys: the same brand, enum,
kind projection, `Functor`, `OrderOf`, and constructor, derived from the
operation signatures alone, with the constructor made row-generic, the
collision and naming rules enforced, and the handler pieces emitted so the
effect can join a `#[handlers]` row.
