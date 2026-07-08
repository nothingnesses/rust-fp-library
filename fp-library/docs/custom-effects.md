# Custom effects

The effects subsystem lets you describe a program as data, a value of type
`Free<Row, A>`, and interpret it however you like. This guide shows how to
define your own effects with the `define_effect!` and `define_row!` macros,
build programs against them, and interpret those programs with a small
hand-written loop. Everything here uses only `fp_library`'s public API, so a
custom effect you define in your own crate works exactly like the examples
below.

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
  There is no built-in generic interpreter yet: you write a small dispatch
  loop, shown below.

## Quick start

Here is a one-effect program end to end: a `Console` effect with a single
`print_line` operation, a row containing just that effect, a two-line program,
and an interpreter that collects the output into a `Vec<String>`.

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
	#[handler_state(none)]
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

// A program is an ordinary value. The smart constructors are row-generic, so
// the target row is inferred from the annotation; `bind` sequences operations.
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

The interpreter is the only hand-written piece, and it is small: `resume` steps
the program, `uninject` selects the effect, `lower` unpacks the operation, and
`resume(value)` continues. Adding an operation to `Console` adds a variant to
`ConsoleF`, and the `match` stops compiling until you handle it, so a missing
case is a compile error, not a silent bug.

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
generic parameter names `R`, `I`, `A`, `B` and the payload name `k` are reserved
by the emission, and derived-name collisions are compile errors, never silently
renamed.

## What `define_effect!` emits

For an effect `Name`, the macro emits: a brand `NameBrand`; an operations enum
`NameF` with one variant per operation (first-order operations are tuple
variants ending in the continuation or `PhantomData`, higher-order operations
are named-field cells including the continuation field `k`); the kind projection
that lets `NameBrand` sit in a row; a `Functor` instance that threads a mapped
function through each continuation; an `OrderOf` marker computed from the
operations (first-order unless some operation owns a sub-program); and one
row-generic smart constructor per operation.

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
program length.

## Interpreting a program

The quick-start interpreter is the general shape: loop, `resume` to the next
operation, and for each effect in the row `uninject` its cell, `lower` it, do
the work, and resume. For a row with several effects, chain the `uninject`
calls, each peeling one brand and passing the remainder on, until the terminal
row (`match remainder {}`) proves every case is handled. Dispatch is by brand,
not by position, so the arms may be written in any order.

This slice interprets on the single-shot `Box` store: each continuation is
called at most once. Effects whose handlers re-enter a continuation more than
once (nondeterministic choice, for example) need the multi-shot stores and are
not expressible on this interpreter yet. There is no generic reusable
interpreter today; the hand-written loop is the interpretation model, and a
generic runner surface is planned (it will not change how effects are defined,
only how they are run).

## Appendix: the manual pattern

`define_effect!` writes the effect definition so you do not have to, but the
generated shape is ordinary code. Here is the `Console` effect hand-written,
which is what the macro emits (the row still uses `define_row!`): a brand, an
operations enum, the kind projection via `impl_kind!`, a `Functor` instance, an
`OrderOf` marker, and a smart constructor (pinned to the row here for brevity,
where the macro emits the row-generic form).

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
operation signatures alone, with the constructor made row-generic and the
collision and naming rules enforced.
