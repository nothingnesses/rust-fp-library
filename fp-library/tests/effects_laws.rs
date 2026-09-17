//! Law property tests for the public program type over a two-effect row.
//!
//! Program equality is observational: two programs are equal when
//! interpreting them through the same hand-written dispatch loop (the shape
//! the custom-effects guide teaches) yields the same result and the same
//! handler-state trace (tick count and supply position). On that equality,
//! the tests check the functor laws (identity, composition), the monad laws
//! (left identity, right identity, associativity), and `embed` naturality
//! (interpreting a widened program equals interpreting the original), each
//! over a small seeded set of effectful programs. A failure here is a
//! substrate defect, never something to accommodate in the test.
#![cfg(feature = "effects")]

use {
	fp_library::{
		Apply,
		classes::Functor,
		define_effect,
		define_row,
		// The `Apply!`/`Kind!` type annotations resolve the macro-generated
		// kind traits by their internal names, so the `kinds` glob must be in
		// scope (the effect macros wrap this glob for their own emissions).
		kinds::*,
		types::{
			Coyoneda,
			Free,
		},
	},
	std::cell::Cell,
};

define_effect! {
	/// A unit step: each `tick` advances a counter the interpreter owns.
	#[handler_state(shared_by_reference)]
	pub effect Tick {
		/// Advance the counter, resuming with unit.
		fn tick() -> ();
	}
}

define_effect! {
	/// A supply of increasing values: each `next` yields the current supply
	/// position and advances it, so a program's result can observe how many
	/// supply values were consumed before it.
	#[handler_state(shared_by_reference)]
	pub effect Supply {
		/// Yield the next supply value.
		fn next() -> i32;
	}
}

define_row! {
	/// The two-effect row the laws are stated over.
	pub row LawRow {
		TickBrand,
		SupplyBrand,
	}
}

define_row! {
	/// A narrow row (`Tick` alone) for the `embed` naturality law.
	pub row NarrowLawRow {
		TickBrand,
	}
}

/// The observation an interpretation produces: the program's result, the
/// tick count, and the number of supply values consumed. Two programs are
/// observationally equal when these agree.
type Observation = (i32, u64, i32);

/// The guide-shaped interpreter over the two-effect row.
fn observe(program: Free<LawRow, i32>) -> Observation {
	let ticks = Cell::new(0);
	let supply = Cell::new(0);
	let mut program = program;
	let value = loop {
		let layer = match program.resume() {
			Ok(value) => break value,
			Err(layer) => layer,
		};
		let layer = {
			let selected: Result<Coyoneda<'static, TickBrand, Free<LawRow, i32>>, _> =
				layer.uninject();
			match selected {
				Ok(op) => {
					match op.lower() {
						TickF::Tick(resume) => {
							ticks.set(ticks.get() + 1);
							program = resume(());
						}
					}
					continue;
				}
				Err(rest) => rest,
			}
		};
		let selected: Result<Coyoneda<'static, SupplyBrand, Free<LawRow, i32>>, _> =
			layer.uninject();
		match selected {
			Ok(op) => match op.lower() {
				SupplyF::Next(resume) => {
					let value = supply.get();
					supply.set(value + 1);
					program = resume(value);
				}
			},
			Err(terminal) => match terminal {},
		}
	};
	(value, ticks.get(), supply.get())
}

/// The narrow-row interpreter for the naturality law's left-hand side.
fn observe_narrow(program: Free<NarrowLawRow, i32>) -> (i32, u64) {
	let ticks = Cell::new(0);
	let mut program = program;
	let value = loop {
		let layer = match program.resume() {
			Ok(value) => break value,
			Err(layer) => layer,
		};
		let selected: Result<Coyoneda<'static, TickBrand, Free<NarrowLawRow, i32>>, _> =
			layer.uninject();
		match selected {
			Ok(op) => match op.lower() {
				TickF::Tick(resume) => {
					ticks.set(ticks.get() + 1);
					program = resume(());
				}
			},
			Err(terminal) => match terminal {},
		}
	};
	(value, ticks.get())
}

/// Widen a narrow-row program into the full row, lazily (the widening of the
/// continuation is deferred into each layer's `Coyoneda` map).
fn widen(program: Free<NarrowLawRow, i32>) -> Free<LawRow, i32> {
	match program.resume() {
		Ok(value) => Free::pure(value),
		Err(layer) => {
			let mapped = <NarrowLawRow as Functor>::map(widen, layer);
			let widened: Apply!(<LawRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<LawRow, i32>>) =
				mapped.embed();
			Free::wrap(widened)
		}
	}
}

/// The seeded program set the laws are checked over: a pure program, a
/// two-effect chain, and a longer mixed chain, so structural differences a
/// law violation would introduce show up in the observation.
fn seeds() -> Vec<fn() -> Free<LawRow, i32>> {
	vec![|| Free::pure(5), || tick().bind(|()| next()), || {
		next()
			.bind(|n| tick().bind(move |()| next().bind(move |m| Free::pure(n * 10 + m))))
			.bind(|x| tick().bind(move |()| Free::pure(x + 1)))
	}]
}

/// An effectful Kleisli arrow: a tick, then an offset.
fn kf(x: i32) -> Free<LawRow, i32> {
	tick().bind(move |()| Free::pure(x + 10))
}

/// A second effectful Kleisli arrow: consumes a supply value.
fn kg(x: i32) -> Free<LawRow, i32> {
	next().bind(move |n| Free::pure(x * 2 + n))
}

#[test]
fn functor_identity() {
	for seed in seeds() {
		assert_eq!(observe(seed().map(|x| x)), observe(seed()));
	}
}

#[test]
fn functor_composition() {
	let f = |x: i32| x * 2 + 1;
	let g = |x: i32| x - 3;
	for seed in seeds() {
		assert_eq!(observe(seed().map(f).map(g)), observe(seed().map(move |x| g(f(x)))));
	}
}

#[test]
fn monad_left_identity() {
	for a in [0, 7, -3] {
		let lhs: Free<LawRow, i32> = Free::pure(a);
		assert_eq!(observe(lhs.bind(kf)), observe(kf(a)));
	}
}

#[test]
fn monad_right_identity() {
	for seed in seeds() {
		assert_eq!(observe(seed().bind(Free::pure)), observe(seed()));
	}
}

#[test]
fn monad_associativity() {
	for seed in seeds() {
		assert_eq!(observe(seed().bind(kf).bind(kg)), observe(seed().bind(|x| kf(x).bind(kg))),);
	}
}

#[test]
fn embed_naturality() {
	// Interpreting a widened program equals interpreting the original: the
	// same result and the same tick trace (the wide row's extra effect is
	// never consumed).
	let narrow_seeds: Vec<fn() -> Free<NarrowLawRow, i32>> =
		vec![|| Free::pure(5), || tick().bind(|()| Free::pure(1)), || {
			tick().bind(|()| tick()).bind(|()| tick().bind(|()| Free::pure(9)))
		}];
	for seed in narrow_seeds {
		let (value, ticks) = observe_narrow(seed());
		assert_eq!(observe(widen(seed())), (value, ticks, 0));
	}
}
