//! Proof of concept for the handler surface's composition mechanism: the
//! trait-mediated per-effect pieces `define_effect!` will emit and the
//! type-level composition `define_row!` will assemble from them.
//!
//! The row macro cannot see the effects' operation inventories (macro
//! invocations expand independently), so per-effect knowledge crosses the
//! seam as associated items on the brand: an arms struct over the effect's
//! own payload and resume-value types (boxed closure fields over borrowed
//! handler state), a per-effect abort type (uninhabited when nothing
//! aborts), and a generic dispatch function. The row side composes them
//! with no operation knowledge at all: a handler struct with one field per
//! cell at the brand's `Arms` projection, a row abort enum built from the
//! cells' abort types, and a loop that is an `uninject` chain over the
//! cells' dispatch functions. Higher-order re-entry crosses the seam
//! through a runner trait with a generic method (the handler struct itself
//! is the runner), so one handler value drives programs at every result
//! type.
#![cfg(feature = "effects")]

use {
	fp_library::{
		classes::WrapDrop,
		define_effect,
		define_row,
		kinds::LifetimeUnaryKind,
		types::{
			Coyoneda,
			Free,
			effects::{
				choose::{
					ChooseBrand,
					ChooseF,
					choose,
					empty,
				},
				state::{
					StateBrand,
					StateF,
					get,
					put,
				},
			},
		},
	},
	std::cell::Cell,
};

define_effect! {
	/// Abort the whole computation with a reason.
	#[handler_state(none)]
	pub effect Fail {
		/// Abort with `reason`; never resumes.
		fn fail(reason: &'static str) -> !;
	}
}

define_row! {
	/// Integer state, a payload-carrying abort, and integer scoped choice.
	pub row PocRow {
		StateBrand<i32>,
		FailBrand,
		ChooseBrand<PocRow, i32>,
	}
}

// -- The library-side seam (what fp-library would provide) --

/// The re-entry seam: a value that can run any program over the row. The
/// method is generic, so the trait is not object-safe; dispatch takes the
/// runner as a generic parameter instead, and the row's handler struct is
/// itself the runner.
trait RowRunner<Row: WrapDrop + 'static, RowAbort> {
	/// Runs a program over the row to its outcome.
	fn run<T: 'static>(
		&self,
		program: Free<Row, T>,
	) -> Result<T, RowAbort>;
}

/// The per-effect pieces the effect macro would emit, exposed on the brand
/// so the row side reaches them by ordinary path-resolved projection.
trait HandlerPieces<Row: WrapDrop + 'static, RowAbort>: LifetimeUnaryKind + Sized {
	/// The effect's arm bundle over borrowed handler state.
	type Arms<'h>;

	/// Interprets one lowered operation against the arms, returning the
	/// continuation program or the abort to propagate; higher-order cells
	/// re-enter through the runner.
	fn dispatch<T: 'static, R: RowRunner<Row, RowAbort>>(
		op: <Self as LifetimeUnaryKind>::Of<'static, Free<Row, T>>,
		arms: &Self::Arms<'_>,
		runner: &R,
	) -> Result<Free<Row, T>, RowAbort>;
}

// -- The per-effect pieces (what define_effect! would emit, per effect) --

/// `State` has no aborting operation.
enum StateAbort {}

/// The `State` arm bundle: payloads-to-resume-value closures.
struct StateArms<'h, S> {
	/// The `get` arm: resume with the current state.
	get: Box<dyn Fn() -> S + 'h>,
	/// The `put` arm: accept the written state.
	put: Box<dyn Fn(S) + 'h>,
}

impl<Row: WrapDrop + 'static, RowAbort, S: 'static> HandlerPieces<Row, RowAbort> for StateBrand<S> {
	type Arms<'h> = StateArms<'h, S>;

	fn dispatch<T: 'static, R: RowRunner<Row, RowAbort>>(
		op: <Self as LifetimeUnaryKind>::Of<'static, Free<Row, T>>,
		arms: &Self::Arms<'_>,
		_runner: &R,
	) -> Result<Free<Row, T>, RowAbort> {
		Ok(match op {
			StateF::Get(resume) => resume((arms.get)()),
			StateF::Put(value, resume) => {
				(arms.put)(value);
				resume(())
			}
		})
	}
}

/// `Fail`'s abort: the `fail` operation's payload.
#[derive(Debug, PartialEq, Eq)]
struct FailAbort(&'static str);

/// `Fail` has no resumptive operation, so its arm bundle is empty.
struct FailArms;

impl<Row: WrapDrop + 'static, RowAbort: From<FailAbort>> HandlerPieces<Row, RowAbort>
	for FailBrand
{
	type Arms<'h> = FailArms;

	fn dispatch<T: 'static, R: RowRunner<Row, RowAbort>>(
		op: <Self as LifetimeUnaryKind>::Of<'static, Free<Row, T>>,
		_arms: &Self::Arms<'_>,
		_runner: &R,
	) -> Result<Free<Row, T>, RowAbort> {
		match op {
			FailF::Fail(reason, _) => Err(FailAbort(reason).into()),
		}
	}
}

/// `Choose`'s abort: the branch-killing `empty`.
#[derive(Debug, PartialEq, Eq)]
struct ChooseEmptyAbort;

/// The `Choose` arm bundle: the elaboration arm receives the owned branch
/// sub-programs and a re-entry handle at the cell's pin, and returns the
/// resume value or the abort to propagate.
struct ChooseArms<'h, Row: WrapDrop + 'static, RAction: 'static, RowAbort> {
	/// The `choose` elaboration arm.
	#[expect(
		clippy::type_complexity,
		reason = "This is the hand-written form of a field type the macro will emit; the emission synthesises it from the operation signature, so there is no reusable alias to factor it into."
	)]
	choose: Box<
		dyn Fn(
				Free<Row, RAction>,
				Free<Row, RAction>,
				&dyn Fn(Free<Row, RAction>) -> Result<RAction, RowAbort>,
			) -> Result<Vec<RAction>, RowAbort>
			+ 'h,
	>,
}

impl<Row: WrapDrop + 'static, RAction: 'static, RowAbort: From<ChooseEmptyAbort>>
	HandlerPieces<Row, RowAbort> for ChooseBrand<Row, RAction>
{
	type Arms<'h> = ChooseArms<'h, Row, RAction, RowAbort>;

	fn dispatch<T: 'static, R: RowRunner<Row, RowAbort>>(
		op: <Self as LifetimeUnaryKind>::Of<'static, Free<Row, T>>,
		arms: &Self::Arms<'_>,
		runner: &R,
	) -> Result<Free<Row, T>, RowAbort> {
		match op {
			ChooseF::Choose {
				left,
				right,
				k,
			} => {
				let retry = |sub: Free<Row, RAction>| runner.run(sub);
				let survivors = (arms.choose)(left, right, &retry)?;
				Ok(k(survivors))
			}
			ChooseF::Empty(_) => Err(ChooseEmptyAbort.into()),
		}
	}
}

// -- The row-side composition (what the define_row! extension would emit) --

/// The row abort: one variant per aborting cell, converted from the cells'
/// own abort types. `State` contributes no variant because its abort type
/// is uninhabited and its pieces impl carries no conversion bound.
#[derive(Debug, PartialEq, Eq)]
enum PocAbort {
	/// The `Fail` cell's abort.
	Fail(FailAbort),
	/// The choice cell's branch death.
	ChooseEmpty(ChooseEmptyAbort),
}

impl From<FailAbort> for PocAbort {
	fn from(abort: FailAbort) -> Self {
		PocAbort::Fail(abort)
	}
}

impl From<ChooseEmptyAbort> for PocAbort {
	fn from(abort: ChooseEmptyAbort) -> Self {
		PocAbort::ChooseEmpty(abort)
	}
}

/// The row's handler list: one field per cell at the brand's arms
/// projection. Construction is a named-field struct literal, so it is
/// order-insensitive and a misspelled cell name is a compile error.
struct PocHandlers<'h> {
	state: <StateBrand<i32> as HandlerPieces<PocRow, PocAbort>>::Arms<'h>,
	fail: <FailBrand as HandlerPieces<PocRow, PocAbort>>::Arms<'h>,
	choose: <ChooseBrand<PocRow, i32> as HandlerPieces<PocRow, PocAbort>>::Arms<'h>,
}

impl RowRunner<PocRow, PocAbort> for PocHandlers<'_> {
	fn run<T: 'static>(
		&self,
		mut program: Free<PocRow, T>,
	) -> Result<T, PocAbort> {
		loop {
			let layer = match program.resume() {
				Ok(value) => return Ok(value),
				Err(layer) => layer,
			};
			let selected: Result<Coyoneda<'static, StateBrand<i32>, Free<PocRow, T>>, _> =
				layer.uninject();
			let layer = match selected {
				Ok(coyo) => {
					program = <StateBrand<i32> as HandlerPieces<PocRow, PocAbort>>::dispatch(
						coyo.lower(),
						&self.state,
						self,
					)?;
					continue;
				}
				Err(rest) => rest,
			};
			let selected: Result<Coyoneda<'static, FailBrand, Free<PocRow, T>>, _> =
				layer.uninject();
			let layer = match selected {
				Ok(coyo) => {
					program = <FailBrand as HandlerPieces<PocRow, PocAbort>>::dispatch(
						coyo.lower(),
						&self.fail,
						self,
					)?;
					continue;
				}
				Err(rest) => rest,
			};
			let selected: Result<PocChooseCell<T>, _> = layer.uninject();
			match selected {
				Ok(coyo) => {
					program =
						<ChooseBrand<PocRow, i32> as HandlerPieces<PocRow, PocAbort>>::dispatch(
							coyo.lower(),
							&self.choose,
							self,
						)?;
				}
				Err(terminal) => match terminal {},
			}
		}
	}
}

/// The row's choice cell over programs yielding `T`, as selected by the
/// loop's brand-keyed `uninject`.
type PocChooseCell<T> = Coyoneda<'static, ChooseBrand<PocRow, i32>, Free<PocRow, T>>;

/// Builds the collecting handler set over a borrowed state cell: dead
/// branches are recovered by the choice arm, every other abort propagates.
fn collecting_handlers(state: &Cell<i32>) -> PocHandlers<'_> {
	PocHandlers {
		state: StateArms {
			get: Box::new(|| state.get()),
			put: Box::new(|value| state.set(value)),
		},
		fail: FailArms,
		choose: ChooseArms {
			choose: Box::new(|left, right, retry| {
				let mut survivors = Vec::new();
				for branch in [left, right] {
					match retry(branch) {
						Ok(value) => survivors.push(value),
						Err(PocAbort::ChooseEmpty(_)) => {}
						Err(other) => return Err(other),
					}
				}
				Ok(survivors)
			}),
		},
	}
}

#[test]
fn the_composed_loop_drives_a_program_at_a_result_type_other_than_the_pin() {
	// The program yields a String while the choice pin is i32; the state
	// arms are shared with the re-entry, so branch writes are global.
	let state = Cell::new(0);
	let handlers = collecting_handlers(&state);
	let program: Free<PocRow, String> = put(1)
		.bind(|()| choose(put(10).bind(|()| get()), get()))
		.bind(|survivors: Vec<i32>| Free::pure(format!("{survivors:?}")));
	assert_eq!(handlers.run(program), Ok("[10, 10]".to_string()));
	assert_eq!(state.get(), 10);
}

#[test]
fn a_no_resume_operation_reifies_through_its_cell_conversion() {
	let state = Cell::new(0);
	let handlers = collecting_handlers(&state);
	let program: Free<PocRow, i32> =
		put(7).bind(|()| fail::<i32, _, _>("boom")).bind(|value: i32| Free::pure(value));
	assert_eq!(handlers.run(program), Err(PocAbort::Fail(FailAbort("boom"))));
	assert_eq!(state.get(), 7);
}

#[test]
fn the_elaboration_arm_recovers_selectively_through_the_re_entry_result() {
	let state = Cell::new(0);
	let handlers = collecting_handlers(&state);
	let recovered: Free<PocRow, i32> = choose(empty::<_, i32, _, _>(), Free::pure(2))
		.bind(|survivors: Vec<i32>| Free::pure(survivors.iter().sum()));
	assert_eq!(handlers.run(recovered), Ok(2));

	let propagated: Free<PocRow, i32> = choose(fail::<i32, _, _>("branch"), Free::pure(2))
		.bind(|survivors: Vec<i32>| Free::pure(survivors.iter().sum()));
	assert_eq!(handlers.run(propagated), Err(PocAbort::Fail(FailAbort("branch"))));
}

#[test]
fn deep_first_order_chains_drive_the_composed_loop_iteratively() {
	// 100k state operations drive the loop's iterative matched arms only.
	const DEPTH: usize = 100_000;
	let state = Cell::new(0);
	let handlers = collecting_handlers(&state);
	let mut program: Free<PocRow, i32> = put(0).bind(|()| get());
	for value in 1 .. DEPTH {
		program = put(value as i32).bind(move |()| program);
	}
	assert_eq!(handlers.run(program), Ok(0));
	assert_eq!(state.get(), 0);
}

/// The uninhabited per-effect abort stays out of the row abort entirely; a
/// conversion from it exists trivially for completeness of the pattern.
impl From<StateAbort> for PocAbort {
	fn from(abort: StateAbort) -> Self {
		match abort {}
	}
}
