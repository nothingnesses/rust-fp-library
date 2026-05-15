# Custom First-Order Effects

This guide shows the current manual pattern for writing a custom first-order
effect for the `Run` family. The point of documenting the manual path first is
to make the effect shape explicit before a future `define_effect!` macro hides
the boilerplate.

A first-order effect is an operation functor. Each operation stores either the
next value directly or a continuation that receives the operation result. A
`Run<R, S, A>` program stores those operations in the first-order row `R`; the
scoped row `S` is separate and is unused in the example below.

## Manual Pattern

The pieces are:

1. A zero-sized brand type.
2. An operation enum parameterized by the continuation result `A`.
3. An `impl_kind!` block mapping the brand to the operation enum.
4. A `Functor` implementation that maps over the continuation result.
5. A `WrapDrop` implementation describing whether a suspended operation can be
   collapsed when the surrounding program is dropped.
6. Named row aliases, usually with `define_effect_row_aliases!`.
7. Smart constructors that lift operation values into a `Run` program.
8. Handler-list entries that lower the custom operation into the remaining row.

For operations with an ordinary stored next value, `WrapDrop::drop` can return
`Some(next)`. For operations that store a closure continuation, it usually
returns `None`, because there is no result value without running the operation.

## Complete Example

This example defines a custom `Config` effect with one operation, `ask_config`.
The handler supplies a concrete configuration value, and the final assertion
checks the handled program output.

```rust
use fp_library::{
	Apply,
	Kind,
	brands::CNilBrand,
	classes::{
		Functor,
		WrapDrop,
	},
	define_effect_row_aliases,
	handlers,
	impl_kind,
	kinds::*,
	types::effects::{
		Run,
		scoped_nt,
	},
};

struct ConfigBrand;

enum ConfigF<'a, A> {
	Ask(Box<dyn FnOnce(&'static str) -> A + 'a>),
}

impl_kind! {
	impl for ConfigBrand {
		type Of<'a, A: 'a>: 'a = ConfigF<'a, A>;
	}
}

impl Functor for ConfigBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			ConfigF::Ask(reply) => ConfigF::Ask(Box::new(move |value| f(reply(value)))),
		}
	}
}

impl WrapDrop for ConfigBrand {
	fn drop<'a, A: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> Option<A> {
		match fa {
			ConfigF::Ask(_) => None,
		}
	}
}

define_effect_row_aliases! {
	type AppEffects = first_order [ConfigBrand];
	type NoEffects = first_order [];
}

type NoScopedEffects = CNilBrand;
type Program<A> = Run<AppEffects, NoScopedEffects, A>;

fn ask_config() -> Program<&'static str> {
	Run::lift::<ConfigBrand, _>(ConfigF::Ask(Box::new(|value| value)))
}

fn greeting() -> Program<String> {
	ask_config().bind(|name| {
		Run::<AppEffects, NoScopedEffects, String>::pure(format!("Hello, {name}"))
	})
}

let result = greeting()
	.interpret_with::<ConfigBrand, _, NoEffects>(
		|op: ConfigF<'_, Run<NoEffects, NoScopedEffects, String>>| {
			match op {
				ConfigF::Ask(reply) => reply("Ada"),
			}
		},
	)
	.interpret(handlers! {}, scoped_nt());

assert_eq!(result, "Hello, Ada");
```

## What The Handler Type Means

The `ConfigBrand` handler receives `ConfigF<'_, Run<NoEffects, NoScopedEffects,
String>>`, not `ConfigF<'_, String>`. The operation's continuation has already
been rewritten so that continuing the custom operation produces a program in the
remaining row. In the example, `reply("Ada")` resumes the suspended program
after `ask_config` with the custom effect removed from the row.

Handle one custom effect at a time with
`interpret_with::<EffectBrand, _, RemainingRow>(...)`. When the first-order row
is empty, close the program with `interpret(handlers! {}, scoped_nt())`. For
programs with several effects still in the row, keep handling one effect at a
time or close the whole row with `interpret(handlers! { ... }, scoped_nt())`
once every remaining first-order and scoped effect has a handler.

## What A Future Macro Can Remove

The stable repetition in the manual pattern is:

- brand declaration;
- operation enum declaration;
- `impl_kind!`;
- the mechanical parts of `Functor`;
- the mechanical parts of `WrapDrop`;
- simple smart constructors;
- row-alias declarations.

The handler body is not mechanical: it defines the meaning of the effect. A
future `define_effect!` macro should not hide handler semantics. It should also
avoid hiding row types entirely, because row aliases are useful in diagnostics
when a handler is missing.

The current recommendation is to keep writing custom effects manually until at
least two or three documented examples expose the same generated shape. That
keeps the macro target aligned with real code instead of with a test-only
abbreviation.
