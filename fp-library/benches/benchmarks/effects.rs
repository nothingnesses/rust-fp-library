// Criterion benches for the effects public surface: programs over
// `define_effect!`/`define_row!` rows, interpreted with the hand-written
// dispatch-loop shape the custom-effects guide teaches (an iterative
// `Free::resume` loop, recursion only to elaborate a higher-order cell).
// Four families:
//
// - dispatch-position: the same unit operation dispatched from each cell of a
//   five-cell row. The interpreter peels cells in row order, so an operation
//   in the tail cell pays four failed `uninject`s per step while the head
//   cell pays none; the sweep makes the per-position cost visible.
// - deep-bind: an effect-step chain over a one-cell row versus the same chain
//   over plain `Free<ThunkBrand>` (one `Thunk` wrap per step, no `Coyoneda`
//   cell, no row coproduct), isolating the row encoding's per-step cost.
// - row-embed: a one-cell-row program widened lazily into the five-cell row
//   and interpreted there, versus interpreted directly on its own row,
//   isolating the per-step `embed` and deferral cost.
// - elaboration: the same tick chain bare versus under one `catch` wrapper
//   (plus an aborting-action variant), isolating the higher-order
//   elaboration overhead over first-order dispatch.
//
// The module body is gated on the `effects` feature (the effect macros and
// row machinery do not exist without it); with the feature off the
// registered bench function is an empty stub, so the harness builds either
// way.

#[cfg(feature = "effects")]
mod gated {
	use {
		criterion::{
			BatchSize,
			BenchmarkId,
			Criterion,
		},
		fp_library::{
			Apply,
			brands::ThunkBrand,
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
				Thunk,
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
		/// A unit step identical in shape to `Tick`; the five-cell row places
		/// it in its second cell.
		#[handler_state(shared_by_reference)]
		pub effect OpB {
			/// Advance the counter, resuming with unit.
			fn op_b() -> ();
		}
	}

	define_effect! {
		/// A unit step identical in shape to `Tick`; the five-cell row places
		/// it in its middle cell.
		#[handler_state(shared_by_reference)]
		pub effect OpC {
			/// Advance the counter, resuming with unit.
			fn op_c() -> ();
		}
	}

	define_effect! {
		/// A unit step identical in shape to `Tick`; the five-cell row places
		/// it in its fourth cell.
		#[handler_state(shared_by_reference)]
		pub effect OpD {
			/// Advance the counter, resuming with unit.
			fn op_d() -> ();
		}
	}

	define_effect! {
		/// A unit step identical in shape to `Tick`; the five-cell row places
		/// it in its tail cell.
		#[handler_state(shared_by_reference)]
		pub effect OpE {
			/// Advance the counter, resuming with unit.
			fn op_e() -> ();
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
		/// Catch owns an action sub-program and a recovery thunk; the
		/// interpreter elaborates it by interpreting the action recursively.
		#[handler_state(none)]
		pub effect Catch<RAction: 'static> {
			/// Run `action`, recovering a bare throw with `recover`.
			fn catch(action: Program<RAction>, recover: impl FnOnce() -> Program<RAction>) -> RAction;
		}
	}

	define_row! {
		/// The five-cell row for the dispatch-position and row-embed families:
		/// `Tick` at the head, the four identical-shape operations behind it.
		pub row PosRow {
			TickBrand,
			OpBBrand,
			OpCBrand,
			OpDBrand,
			OpEBrand,
		}
	}

	define_row! {
		/// The one-cell row for the deep-bind family and the embed source.
		pub row TickRow {
			TickBrand,
		}
	}

	define_row! {
		/// The elaboration row: first-order steps, an abort, and the
		/// higher-order catch cell storing `Free<CatchRow, i32>` sub-programs.
		pub row CatchRow {
			TickBrand,
			ThrowBrand,
			CatchBrand<CatchRow, i32>,
		}
	}

	/// The catch cell pinned at the row; the alias keeps the interpreter's
	/// `uninject` annotation under the type-complexity lint's threshold.
	type CatchPinned = CatchBrand<CatchRow, i32>;

	/// Build an `n`-step chain of one operation ending in `pure(7)`; the
	/// operation constructors are row-generic, so the row is pinned here.
	macro_rules! op_chain {
		($row:ty, $op:expr, $n:expr) => {{
			let mut program: Free<$row, ()> = $op;
			for _ in 1 .. $n {
				program = program.bind(|()| $op);
			}
			program.bind(|()| Free::pure(7))
		}};
	}

	/// One non-terminal cell of a dispatch loop: try to select the brand's
	/// operation; on a match, count it, resume, and continue the loop;
	/// otherwise pass the remaining layer to the next cell.
	macro_rules! peel_unit {
		($layer:expr, $program:ident, $count:expr, $row:ty, $brand:ty, $enum:ident :: $variant:ident) => {{
			let selected: Result<Coyoneda<'static, $brand, Free<$row, i32>>, _> = $layer.uninject();
			match selected {
				Ok(op) => {
					match op.lower() {
						$enum::$variant(resume) => {
							$count.set($count.get() + 1);
							$program = resume(());
						}
					}
					continue;
				}
				Err(rest) => rest,
			}
		}};
	}

	/// The guide-shaped interpreter over the five-cell row, peeling cells in
	/// row order, so an operation's dispatch cost grows with its cell position.
	fn run_pos(
		mut program: Free<PosRow, i32>,
		ops: &Cell<u64>,
	) -> i32 {
		loop {
			let layer = match program.resume() {
				Ok(value) => return value,
				Err(layer) => layer,
			};
			let layer = peel_unit!(layer, program, ops, PosRow, TickBrand, TickF::Tick);
			let layer = peel_unit!(layer, program, ops, PosRow, OpBBrand, OpBF::OpB);
			let layer = peel_unit!(layer, program, ops, PosRow, OpCBrand, OpCF::OpC);
			let layer = peel_unit!(layer, program, ops, PosRow, OpDBrand, OpDF::OpD);
			let selected: Result<Coyoneda<'static, OpEBrand, Free<PosRow, i32>>, _> =
				layer.uninject();
			match selected {
				Ok(op) => match op.lower() {
					OpEF::OpE(resume) => {
						ops.set(ops.get() + 1);
						program = resume(());
					}
				},
				Err(terminal) => match terminal {},
			}
		}
	}

	/// The one-cell-row interpreter (the deep-bind and embed-reference loop).
	fn run_tick(
		mut program: Free<TickRow, i32>,
		ticks: &Cell<u64>,
	) -> i32 {
		loop {
			let layer = match program.resume() {
				Ok(value) => return value,
				Err(layer) => layer,
			};
			let selected: Result<Coyoneda<'static, TickBrand, Free<TickRow, i32>>, _> =
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
		}
	}

	/// The elaboration-row interpreter: iterative dispatch for the first-order
	/// cells, recursion only to elaborate `catch` (run the action; on a bare
	/// throw, run the recovery; thread the value to the continuation).
	fn run_catch(
		mut program: Free<CatchRow, i32>,
		ticks: &Cell<u64>,
	) -> Result<i32, ()> {
		loop {
			let layer = match program.resume() {
				Ok(value) => return Ok(value),
				Err(layer) => layer,
			};
			let layer = peel_unit!(layer, program, ticks, CatchRow, TickBrand, TickF::Tick);
			let layer = {
				let selected: Result<Coyoneda<'static, ThrowBrand, Free<CatchRow, i32>>, _> =
					layer.uninject();
				match selected {
					Ok(op) => match op.lower() {
						ThrowF::Throw(_) => return Err(()),
					},
					Err(rest) => rest,
				}
			};
			let selected: Result<Coyoneda<'static, CatchPinned, Free<CatchRow, i32>>, _> =
				layer.uninject();
			match selected {
				Ok(op) => match op.lower() {
					CatchF::Catch {
						action,
						recover,
						k,
					} => {
						let value = match run_catch(action, ticks) {
							Ok(value) => value,
							Err(()) => run_catch(recover(), ticks)?,
						};
						program = k(value);
					}
				},
				Err(terminal) => match terminal {},
			}
		}
	}

	/// Widen a one-cell-row program into the five-cell row, lazily (the
	/// widening of the continuation is deferred into each layer's `Coyoneda`
	/// map), so the measured cost is the per-step `embed` and deferral
	/// machinery, not an eager pre-pass.
	fn widen(program: Free<TickRow, i32>) -> Free<PosRow, i32> {
		match program.resume() {
			Ok(value) => Free::pure(value),
			Err(layer) => {
				let mapped = <TickRow as Functor>::map(widen, layer);
				let widened: Apply!(<PosRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<PosRow, i32>>) =
					mapped.embed();
				Free::wrap(widened)
			}
		}
	}

	/// The plain-`Free` analog of an `n`-step effect chain: each step is one
	/// `Thunk` wrap, with no `Coyoneda` cell and no row coproduct around it.
	fn plain_chain(n: usize) -> Free<ThunkBrand, i32> {
		let mut program: Free<ThunkBrand, ()> = Free::wrap(Thunk::new(|| Free::pure(())));
		for _ in 1 .. n {
			program = program.bind(|()| Free::wrap(Thunk::new(|| Free::pure(()))));
		}
		program.bind(|()| Free::pure(7))
	}

	pub fn bench_effects(c: &mut Criterion) {
		let depths: &[usize] = &[10, 100, 1_000, 10_000];

		// -- dispatch position --

		let mut group = c.benchmark_group("Effects/dispatch-position");

		for &depth in depths {
			group.bench_with_input(BenchmarkId::new("cell 1 (head)", depth), &depth, |b, &k| {
				b.iter_batched(
					|| op_chain!(PosRow, tick(), k),
					|program| run_pos(program, &Cell::new(0)),
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("cell 2", depth), &depth, |b, &k| {
				b.iter_batched(
					|| op_chain!(PosRow, op_b(), k),
					|program| run_pos(program, &Cell::new(0)),
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("cell 3 (middle)", depth), &depth, |b, &k| {
				b.iter_batched(
					|| op_chain!(PosRow, op_c(), k),
					|program| run_pos(program, &Cell::new(0)),
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("cell 4", depth), &depth, |b, &k| {
				b.iter_batched(
					|| op_chain!(PosRow, op_d(), k),
					|program| run_pos(program, &Cell::new(0)),
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("cell 5 (tail)", depth), &depth, |b, &k| {
				b.iter_batched(
					|| op_chain!(PosRow, op_e(), k),
					|program| run_pos(program, &Cell::new(0)),
					BatchSize::SmallInput,
				)
			});
		}

		group.finish();

		// -- deep-bind throughput versus plain Free --

		let mut group = c.benchmark_group("Effects/deep-bind vs plain Free");

		for &depth in depths {
			group.bench_with_input(
				BenchmarkId::new("row: build+interpret", depth),
				&depth,
				|b, &k| {
					b.iter(|| {
						let program = op_chain!(TickRow, tick(), k);
						run_tick(program, &Cell::new(0))
					})
				},
			);
			group.bench_with_input(
				BenchmarkId::new("row: interpret only", depth),
				&depth,
				|b, &k| {
					b.iter_batched(
						|| op_chain!(TickRow, tick(), k),
						|program| run_tick(program, &Cell::new(0)),
						BatchSize::SmallInput,
					)
				},
			);
			group.bench_with_input(
				BenchmarkId::new("plain Free: build+evaluate", depth),
				&depth,
				|b, &k| b.iter(|| plain_chain(k).evaluate()),
			);
			group.bench_with_input(
				BenchmarkId::new("plain Free: evaluate only", depth),
				&depth,
				|b, &k| {
					b.iter_batched(
						|| plain_chain(k),
						Free::<ThunkBrand, i32>::evaluate,
						BatchSize::SmallInput,
					)
				},
			);
		}

		group.finish();

		// -- row-embed cost versus program size --

		let mut group = c.benchmark_group("Effects/row-embed");

		for &depth in depths {
			group.bench_with_input(
				BenchmarkId::new("widen into five-cell row + interpret", depth),
				&depth,
				|b, &k| {
					b.iter_batched(
						|| op_chain!(TickRow, tick(), k),
						|program| run_pos(widen(program), &Cell::new(0)),
						BatchSize::SmallInput,
					)
				},
			);
			group.bench_with_input(
				BenchmarkId::new("narrow interpret (reference)", depth),
				&depth,
				|b, &k| {
					b.iter_batched(
						|| op_chain!(TickRow, tick(), k),
						|program| run_tick(program, &Cell::new(0)),
						BatchSize::SmallInput,
					)
				},
			);
		}

		group.finish();

		// -- elaboration overhead versus first-order dispatch --

		let mut group = c.benchmark_group("Effects/elaboration");

		for &depth in depths {
			group.bench_with_input(
				BenchmarkId::new("no catch (first-order)", depth),
				&depth,
				|b, &k| {
					b.iter_batched(
						|| op_chain!(CatchRow, tick(), k),
						|program| run_catch(program, &Cell::new(0)),
						BatchSize::SmallInput,
					)
				},
			);
			group.bench_with_input(BenchmarkId::new("under one catch", depth), &depth, |b, &k| {
				b.iter_batched(
					|| catch(op_chain!(CatchRow, tick(), k), || Free::pure(-1)),
					|program| run_catch(program, &Cell::new(0)),
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(
				BenchmarkId::new("under one catch, aborting action", depth),
				&depth,
				|b, &k| {
					b.iter_batched(
						|| {
							let action =
								op_chain!(CatchRow, tick(), k).bind(|_| throw::<i32, _, _, _>());
							catch(action, || Free::pure(-1))
						},
						|program| run_catch(program, &Cell::new(0)),
						BatchSize::SmallInput,
					)
				},
			);
		}

		group.finish();
	}
}

#[cfg(feature = "effects")]
pub use gated::bench_effects;

/// With the effects feature off the row machinery does not exist, so the
/// registered bench function is an empty stub.
#[cfg(not(feature = "effects"))]
pub fn bench_effects(_c: &mut criterion::Criterion) {}
