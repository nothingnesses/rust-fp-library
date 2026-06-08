#![cfg(feature = "effects")]

//! Semantic regression tests ported from Heftia's current-effect test
//! suite.
//!
//! The source tests are
//! [`heftia-effects/test/Test/Semantics.hs`](https://github.com/sayo-hs/heftia/blob/542963d4449d31a0c17a41a1acf56c74ed79ac0d/heftia-effects/test/Test/Semantics.hs#L30-L88)
//! and
//! [`heftia-effects/test/Test/Pyth.hs`](https://github.com/sayo-hs/heftia/blob/542963d4449d31a0c17a41a1acf56c74ed79ac0d/heftia-effects/test/Test/Pyth.hs#L23-L30).
//! The Writer pre/post `censor` cases are ported from
//! [`heftia-effects/test/Test/Writer.hs`](https://github.com/sayo-hs/heftia/blob/542963d4449d31a0c17a41a1acf56c74ed79ac0d/heftia-effects/test/Test/Writer.hs#L29-L36).
//!
//! These tests keep the Heftia cases whose effect surfaces already
//! exist in this library: State, Catch, Except/Throw, Choose, Empty,
//! custom first-order effects, scoped Writer `censor`, and scoped
//! Writer `listen`.

use {
	fp_library::{
		Apply,
		brands::{
			BoxBrand,
			BoxCatchBrand,
			BoxStateBrand,
			BoxWriterCensorBrand,
			CNilBrand,
			CatchBrand,
			ChooseBrand,
			CoproductBrand,
			CoyonedaBrand,
			EmptyBrand,
			ExceptBrand,
			RcBrand,
			RcCoyonedaBrand,
			WriterBrand,
			WriterListenBrand,
		},
		classes::{
			Functor,
			WrapDrop,
		},
		handlers,
		impl_kind,
		kinds::*,
		scoped_handlers,
		types::{
			Additive,
			effects::{
				choose::Choose,
				empty::Empty,
				except::Except,
				rc_run::RcRun,
				run::{
					Run,
					RunFirstOrderHandler,
				},
				scoped_nt,
				standard_scoped_handlers::{
					catch_handler,
					writer_post_handler,
					writer_pre_handler,
				},
				state::BoxState,
				writer::Writer,
			},
		},
	},
	std::{
		cell::RefCell,
		rc::Rc,
	},
};

type UnitError = ();
type BoolResult = Result<bool, UnitError>;

type StateExceptRow = CoproductBrand<
	CoyonedaBrand<BoxStateBrand<BoxBrand, bool>>,
	CoproductBrand<CoyonedaBrand<ExceptBrand<UnitError>>, CNilBrand>,
>;
type StateOnlyRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, bool>>, CNilBrand>;
type UnitExceptOnlyRow = CoproductBrand<CoyonedaBrand<ExceptBrand<UnitError>>, CNilBrand>;
type UnitCatchRow = CoproductBrand<BoxCatchBrand<BoxBrand, UnitError>, CNilBrand>;

type StateCatchProgram<A> = Run<StateExceptRow, UnitCatchRow, A>;
type StateEliminatedProgram<A> = Run<UnitExceptOnlyRow, UnitCatchRow, A>;

type WriterRow = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
type WriterCensorRow = CoproductBrand<BoxWriterCensorBrand<BoxBrand, String>, CNilBrand>;
type WriterProgram<A> = Run<WriterRow, WriterCensorRow, A>;

fn state_then_throw_inside_catch() -> StateCatchProgram<bool> {
	let protected: StateCatchProgram<()> =
		Run::<StateExceptRow, UnitCatchRow, ()>::put::<bool, _>(true)
			.bind(|()| Run::<StateExceptRow, UnitCatchRow, ()>::throw::<UnitError, _>(()));

	Run::catch::<UnitError, _>(protected, |()| Run::pure(()))
		.bind(|()| Run::<StateExceptRow, UnitCatchRow, bool>::get())
}

fn writer_hello() -> WriterProgram<()> {
	Run::<WriterRow, WriterCensorRow, ()>::tell::<String, _>("Hello".to_string())
		.bind(|()| Run::<WriterRow, WriterCensorRow, ()>::tell::<String, _>(" world!".to_string()))
}

fn heftia_writer_censor(log: String) -> String {
	match log.as_str() {
		"Hello" => "Goodbye".to_string(),
		"Hello world!" => "Hello world!!".to_string(),
		_ => log,
	}
}

fn censor_hello() -> WriterProgram<()> {
	Run::censor::<String, _>(heftia_writer_censor, writer_hello())
}

fn run_writer_pre(program: WriterProgram<()>) -> String {
	let output = Rc::new(RefCell::new(String::new()));
	let output_for_handler = Rc::clone(&output);

	program.handle(
		handlers! {
			WriterBrand<String>: move |op: Writer<'_, String, WriterProgram<()>>| match op {
				Writer::Tell(log, next, _) => {
					output_for_handler.borrow_mut().push_str(&log);
					next
				}
			},
		},
		scoped_handlers! {
			BoxWriterCensorBrand<BoxBrand, String>: writer_pre_handler::<_, CNilBrand, _>(),
		},
	);

	output.borrow().clone()
}

fn run_writer_post(program: WriterProgram<()>) -> String {
	let output = Rc::new(RefCell::new(String::new()));
	let output_for_handler = Rc::clone(&output);

	program.handle(
		handlers! {
			WriterBrand<String>: move |op: Writer<'_, String, WriterProgram<()>>| match op {
				Writer::Tell(log, next, _) => {
					output_for_handler.borrow_mut().push_str(&log);
					next
				}
			},
		},
		scoped_handlers! {
			BoxWriterCensorBrand<BoxBrand, String>: writer_post_handler::<_, CNilBrand, _>(),
		},
	);

	output.borrow().clone()
}

#[test]
fn writer_pre_censor_matches_heftia_goodbye_world_case() {
	assert_eq!(run_writer_pre(censor_hello()), "Goodbye world!");
}

#[test]
fn writer_post_censor_matches_heftia_hello_world_bang_bang_case() {
	assert_eq!(run_writer_post(censor_hello()), "Hello world!!");
}

fn state_handler_for_full_row(
	cell: Rc<RefCell<bool>>
) -> impl Fn(BoxState<'_, BoxBrand, bool, StateCatchProgram<bool>>) -> StateCatchProgram<bool> {
	move |op| match op {
		BoxState::Get(k) => k(*cell.borrow()),
		BoxState::Put(next_state, k) => {
			*cell.borrow_mut() = next_state;
			k(())
		}
	}
}

struct BoolStateHandler {
	cell: Rc<RefCell<bool>>,
}

impl RunFirstOrderHandler<BoxStateBrand<BoxBrand, bool>, UnitExceptOnlyRow, UnitCatchRow>
	for BoolStateHandler
{
	fn handle<T: 'static>(
		&self,
		op: BoxState<'static, BoxBrand, bool, StateEliminatedProgram<T>>,
	) -> StateEliminatedProgram<T> {
		match op {
			BoxState::Get(k) => k(*self.cell.borrow()),
			BoxState::Put(next_state, k) => {
				*self.cell.borrow_mut() = next_state;
				k(())
			}
		}
	}
}

fn run_catch_before_state(program: StateCatchProgram<bool>) -> (bool, bool) {
	let state = Rc::new(RefCell::new(false));
	let state_for_handler = Rc::clone(&state);

	let result = program.handle(
		handlers! {
			BoxStateBrand<BoxBrand, bool>: state_handler_for_full_row(state_for_handler),
			ExceptBrand<UnitError>: |_op: Except<'_, UnitError, StateCatchProgram<bool>>| {
				Run::pure(false)
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, UnitError>: catch_handler::<_, StateOnlyRow, _>(),
		},
	);

	(result, *state.borrow())
}

fn run_state_before_catch(program: StateCatchProgram<bool>) -> (bool, bool) {
	let state = Rc::new(RefCell::new(false));
	let state_for_handler = Rc::clone(&state);

	let state_eliminated: StateEliminatedProgram<bool> = program
		.handle_with_handler::<BoxStateBrand<BoxBrand, bool>, _, UnitExceptOnlyRow>(
			BoolStateHandler {
				cell: state_for_handler,
			},
		);
	let result = state_eliminated.handle(
		handlers! {
			ExceptBrand<UnitError>: |_op: Except<'_, UnitError, StateEliminatedProgram<bool>>| {
				Run::pure(false)
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, UnitError>: catch_handler::<_, CNilBrand, _>(),
		},
	);

	(result, *state.borrow())
}

#[test]
fn state_write_before_caught_throw_survives_handler_orders() {
	assert_eq!(run_catch_before_state(state_then_throw_inside_catch()), (true, true));
	assert_eq!(run_state_before_catch(state_then_throw_inside_catch()), (true, true));
}

type RcExceptChooseRow = CoproductBrand<
	RcCoyonedaBrand<ChooseBrand<RcBrand>>,
	CoproductBrand<RcCoyonedaBrand<ExceptBrand<UnitError>>, CNilBrand>,
>;
type RcChooseOnlyRow = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
type RcCatchRow = CoproductBrand<CatchBrand<RcBrand, UnitError>, CNilBrand>;
type RcChoiceCatchProgram<A> = RcRun<RcExceptChooseRow, RcCatchRow, A>;

fn choose_or_throw_branch_results(
	pure_on_true_branch: bool
) -> RcChoiceCatchProgram<Vec<BoolResult>> {
	let action = RcRun::<RcExceptChooseRow, RcCatchRow, bool>::choose().bind(move |branch| {
		if branch == pure_on_true_branch {
			RcRun::pure(vec![Ok(true)])
		} else {
			RcRun::throw::<UnitError, _>(())
		}
	});

	RcRun::catch::<UnitError, _>(action, |()| RcRun::pure(vec![Ok(false)]))
}

fn choose_or_throw_whole_result(
	pure_on_true_branch: bool
) -> RcChoiceCatchProgram<Result<Vec<bool>, UnitError>> {
	let action = RcRun::<RcExceptChooseRow, RcCatchRow, bool>::choose().bind(move |branch| {
		if branch == pure_on_true_branch {
			RcRun::pure(Ok(vec![true]))
		} else {
			RcRun::throw::<UnitError, _>(())
		}
	});

	RcRun::catch::<UnitError, _>(action, |()| RcRun::pure(Ok(vec![false])))
}

fn run_choose_then_throw_vec(program: RcChoiceCatchProgram<Vec<BoolResult>>) -> Vec<BoolResult> {
	program.handle(
		handlers! {
			ChooseBrand<RcBrand>: |op: Choose<'_, RcBrand, RcChoiceCatchProgram<Vec<BoolResult>>>| {
				match op {
					Choose::Alt(k) => {
						let mut results = run_choose_then_throw_vec((*k)(true));
						results.extend(run_choose_then_throw_vec((*k)(false)));
						RcRun::pure(results)
					}
				}
			},
			ExceptBrand<UnitError>: |_op: Except<'_, UnitError, RcChoiceCatchProgram<Vec<BoolResult>>>| {
				RcRun::pure(vec![Err(())])
			},
		},
		scoped_handlers! {
			CatchBrand<RcBrand, UnitError>: catch_handler::<_, RcChooseOnlyRow, _>(),
		},
	)
}

fn run_throw_after_choose_result(
	program: RcChoiceCatchProgram<Result<Vec<bool>, UnitError>>
) -> Result<Vec<bool>, UnitError> {
	program.handle(
		handlers! {
			ChooseBrand<RcBrand>: |op: Choose<'_, RcBrand, RcChoiceCatchProgram<Result<Vec<bool>, UnitError>>>| {
				match op {
					Choose::Alt(k) => {
						let left = run_throw_after_choose_result((*k)(true));
						let right = run_throw_after_choose_result((*k)(false));
						let combined = match (left, right) {
							(Ok(mut left_values), Ok(right_values)) => {
								left_values.extend(right_values);
								Ok(left_values)
							}
							(Err(err), _) | (_, Err(err)) => Err(err),
						};
						RcRun::pure(combined)
					}
				}
			},
			ExceptBrand<UnitError>: |_op: Except<'_, UnitError, RcChoiceCatchProgram<Result<Vec<bool>, UnitError>>>| {
				RcRun::pure(Err(()))
			},
		},
		scoped_handlers! {
			CatchBrand<RcBrand, UnitError>: catch_handler::<_, RcChooseOnlyRow, _>(),
		},
	)
}

#[test]
fn choose_and_catch_ordering_matches_heftia_semantics() {
	assert_eq!(
		run_choose_then_throw_vec(choose_or_throw_branch_results(true)),
		vec![Ok(true), Ok(false)]
	);
	assert_eq!(
		run_throw_after_choose_result(choose_or_throw_whole_result(true)),
		Ok(vec![true, false])
	);
	assert_eq!(
		run_choose_then_throw_vec(choose_or_throw_branch_results(false)),
		vec![Ok(false), Ok(true)]
	);
	assert_eq!(
		run_throw_after_choose_result(choose_or_throw_whole_result(false)),
		Ok(vec![false, true])
	);
}

type SumLog = Additive<i32>;
type ListenValue = (bool, SumLog);
type BranchLoggedListenValue = (SumLog, ListenValue);

type RcNonDetWriterRow = CoproductBrand<
	RcCoyonedaBrand<ChooseBrand<RcBrand>>,
	CoproductBrand<
		RcCoyonedaBrand<EmptyBrand>,
		CoproductBrand<RcCoyonedaBrand<WriterBrand<SumLog>>, CNilBrand>,
	>,
>;
type RcNonDetOnlyRow = CoproductBrand<
	RcCoyonedaBrand<ChooseBrand<RcBrand>>,
	CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>,
>;
type RcWriterListenRow = CoproductBrand<WriterListenBrand<RcBrand, SumLog, bool>, CNilBrand>;
type RcNonDetWriterProgram<A> = RcRun<RcNonDetWriterRow, RcWriterListenRow, A>;

fn add_sum(
	left: SumLog,
	right: SumLog,
) -> SumLog {
	Additive(left.0 + right.0)
}

fn tell_sum(amount: i32) -> RcNonDetWriterProgram<()> {
	RcRun::<RcNonDetWriterRow, RcWriterListenRow, ()>::tell::<SumLog, _>(Additive(amount))
}

fn nondet_writer_listen_action() -> RcNonDetWriterProgram<ListenValue> {
	let selected_action = tell_sum(1).bind(|()| {
		RcRun::<RcNonDetWriterRow, RcWriterListenRow, bool>::choose().bind(|branch| {
			if branch {
				tell_sum(2).bind(|()| RcRun::pure(true))
			} else {
				tell_sum(3).bind(|()| RcRun::pure(false))
			}
		})
	});

	RcRun::listen::<SumLog, _>(selected_action)
}

fn run_nondet_after_tell(
	program: RcNonDetWriterProgram<ListenValue>
) -> Vec<BranchLoggedListenValue> {
	program
		.map(|value| vec![(Additive(0), value)])
		.handle(
			handlers! {
				ChooseBrand<RcBrand>: |op: Choose<'_, RcBrand, RcNonDetWriterProgram<Vec<BranchLoggedListenValue>>>| match op {
					Choose::Alt(k) => {
						let mut values = run_nondet_after_tell_mapped((*k)(true));
						values.extend(run_nondet_after_tell_mapped((*k)(false)));
						RcRun::pure(values)
					}
				},
				EmptyBrand: |_op: Empty<'_, RcNonDetWriterProgram<Vec<BranchLoggedListenValue>>>| {
					RcRun::pure(Vec::new())
				},
				WriterBrand<SumLog>: |op: Writer<'_, SumLog, RcNonDetWriterProgram<Vec<BranchLoggedListenValue>>>| match op {
					Writer::Tell(log, next, _) => next.map(move |values| {
						values
							.into_iter()
							.map(|(tail_log, value)| (add_sum(log, tail_log), value))
							.collect()
					}),
				},
			},
			scoped_handlers! {
				WriterListenBrand<RcBrand, SumLog, bool>: writer_post_handler::<_, RcNonDetOnlyRow, _>(),
			},
		)
}

fn run_nondet_after_tell_mapped(
	program: RcNonDetWriterProgram<Vec<BranchLoggedListenValue>>
) -> Vec<BranchLoggedListenValue> {
	program.handle(
		handlers! {
			ChooseBrand<RcBrand>: |op: Choose<'_, RcBrand, RcNonDetWriterProgram<Vec<BranchLoggedListenValue>>>| match op {
				Choose::Alt(k) => {
					let mut values = run_nondet_after_tell_mapped((*k)(true));
					values.extend(run_nondet_after_tell_mapped((*k)(false)));
					RcRun::pure(values)
				}
			},
			EmptyBrand: |_op: Empty<'_, RcNonDetWriterProgram<Vec<BranchLoggedListenValue>>>| {
				RcRun::pure(Vec::new())
			},
			WriterBrand<SumLog>: |op: Writer<'_, SumLog, RcNonDetWriterProgram<Vec<BranchLoggedListenValue>>>| match op {
				Writer::Tell(log, next, _) => next.map(move |values| {
					values
						.into_iter()
						.map(|(tail_log, value)| (add_sum(log, tail_log), value))
						.collect()
				}),
			},
		},
		scoped_handlers! {
			WriterListenBrand<RcBrand, SumLog, bool>: writer_post_handler::<_, RcNonDetOnlyRow, _>(),
		},
	)
}

fn run_tell_after_nondet(
	program: RcNonDetWriterProgram<ListenValue>
) -> (SumLog, Vec<ListenValue>) {
	// Heftia's `runTell` updates its Writer accumulator before invoking
	// the continuation. Modelling it as `next.map(|tail| log + tail)`
	// would add a pre-choice `Tell` once per `Choose` branch, which is
	// not the semantics of `runTell . runNonDet`.
	let accumulated = Rc::new(RefCell::new(Additive(0)));
	let values =
		run_tell_after_nondet_mapped(program.map(|value| vec![value]), Rc::clone(&accumulated));
	(*accumulated.borrow(), values)
}

fn run_tell_after_nondet_mapped(
	program: RcNonDetWriterProgram<Vec<ListenValue>>,
	accumulated: Rc<RefCell<SumLog>>,
) -> Vec<ListenValue> {
	program.handle(
		handlers! {
			ChooseBrand<RcBrand>: |op: Choose<'_, RcBrand, RcNonDetWriterProgram<Vec<ListenValue>>>| match op {
				Choose::Alt(k) => {
					let mut left_values =
						run_tell_after_nondet_mapped((*k)(true), Rc::clone(&accumulated));
					let right_values =
						run_tell_after_nondet_mapped((*k)(false), Rc::clone(&accumulated));
					left_values.extend(right_values);
					RcRun::pure(left_values)
				}
			},
			EmptyBrand: |_op: Empty<'_, RcNonDetWriterProgram<Vec<ListenValue>>>| {
				RcRun::pure(Vec::new())
			},
			WriterBrand<SumLog>: |op: Writer<'_, SumLog, RcNonDetWriterProgram<Vec<ListenValue>>>| match op {
				Writer::Tell(log, next, _) => {
					let mut current = accumulated.borrow_mut();
					*current = add_sum(*current, log);
					drop(current);
					next
				},
			},
		},
		scoped_handlers! {
			WriterListenBrand<RcBrand, SumLog, bool>: writer_post_handler::<_, RcNonDetOnlyRow, _>(),
		},
	)
}

#[test]
fn nondet_outside_writer_duplicates_shared_prefix_per_branch() {
	assert_eq!(
		run_nondet_after_tell(nondet_writer_listen_action()),
		vec![(Additive(3), (true, Additive(3))), (Additive(4), (false, Additive(4))),]
	);
}

#[test]
fn writer_outside_nondet_counts_shared_prefix_once_globally() {
	assert_eq!(
		run_tell_after_nondet(nondet_writer_listen_action()),
		(Additive(6), vec![(true, Additive(3)), (false, Additive(4))])
	);
}

struct SomeActionBrand;

enum SomeActionF<'a, A> {
	SomeAction(Box<dyn FnOnce(&'static str) -> A + 'a>),
}

impl_kind! {
	impl for SomeActionBrand {
		type Of<'a, A: 'a>: 'a = SomeActionF<'a, A>;
	}
}

impl Functor for SomeActionBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			SomeActionF::SomeAction(reply) =>
				SomeActionF::SomeAction(Box::new(move |value| f(reply(value)))),
		}
	}
}

impl WrapDrop for SomeActionBrand {
	fn drop<'a, A: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> Option<A> {
		match fa {
			SomeActionF::SomeAction(_) => None,
		}
	}
}

struct SomeActionToThrow;

impl RunFirstOrderHandler<SomeActionBrand, StrExceptOnlyRow, StrCatchRow> for SomeActionToThrow {
	fn handle<T: 'static>(
		&self,
		op: SomeActionF<'static, SomeLoweredProgram<T>>,
	) -> SomeLoweredProgram<T> {
		match op {
			SomeActionF::SomeAction(_) => Run::throw::<&'static str, _>("not caught"),
		}
	}
}

type SomeExceptRow = CoproductBrand<
	CoyonedaBrand<ExceptBrand<&'static str>>,
	CoproductBrand<CoyonedaBrand<SomeActionBrand>, CNilBrand>,
>;
type SomeOnlyRow = CoproductBrand<CoyonedaBrand<SomeActionBrand>, CNilBrand>;
type StrExceptOnlyRow = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type StrCatchRow = CoproductBrand<BoxCatchBrand<BoxBrand, &'static str>, CNilBrand>;

type SomeProgram<A> = Run<SomeExceptRow, StrCatchRow, A>;
type SomeLoweredProgram<A> = Run<StrExceptOnlyRow, StrCatchRow, A>;
type StrResult = Result<&'static str, &'static str>;

fn some_action_result() -> SomeProgram<StrResult> {
	Run::lift::<SomeActionBrand, _>(SomeActionF::SomeAction(Box::new(Ok)))
}

fn some_action_under_catch() -> SomeProgram<StrResult> {
	Run::catch::<&'static str, _>(some_action_result(), |_err| Run::pure(Ok("caught")))
}

fn lower_some_action_first(program: SomeProgram<StrResult>) -> SomeLoweredProgram<StrResult> {
	program.handle_with_handler::<SomeActionBrand, _, StrExceptOnlyRow>(SomeActionToThrow)
}

fn close_lowered_some_program(program: SomeLoweredProgram<StrResult>) -> StrResult {
	program.handle(
		handlers! {
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, SomeLoweredProgram<StrResult>>| {
				match op {
					Except::Throw(err, _) => Run::pure(Err(err)),
				}
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, CNilBrand, _>(),
		},
	)
}

fn close_catch_before_some(program: SomeProgram<StrResult>) -> StrResult {
	program.handle(
		handlers! {
			SomeActionBrand: |op: SomeActionF<'_, SomeProgram<StrResult>>| {
				match op {
					SomeActionF::SomeAction(_) =>
						Run::throw::<&'static str, _>("not caught"),
				}
			},
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, SomeProgram<StrResult>>| {
				match op {
					Except::Throw(err, _) => Run::pure(Err(err)),
				}
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, SomeOnlyRow, _>(),
		},
	)
}

#[test]
fn custom_effect_throw_is_caught_only_when_lowered_inside_catch() {
	assert_eq!(
		close_lowered_some_program(lower_some_action_first(some_action_under_catch())),
		Ok("caught")
	);
	assert_eq!(close_catch_before_some(some_action_under_catch()), Err("not caught"));
}

type RcChooseProgram<A> = RcRun<RcChooseOnlyRow, CNilBrand, A>;
type Triple = (i32, i32, i32);

fn choose_number(upbound: i32) -> RcChooseProgram<Option<i32>> {
	if upbound == 0 {
		RcRun::pure(None)
	} else {
		RcRun::<RcChooseOnlyRow, CNilBrand, bool>::choose().bind(move |take_lower| {
			if take_lower { choose_number(upbound - 1) } else { RcRun::pure(Some(upbound)) }
		})
	}
}

fn pythagorean_search(upbound: i32) -> RcChooseProgram<Option<Triple>> {
	choose_number(upbound).bind(move |x| match x {
		Some(x) => choose_number(upbound).bind(move |y| match y {
			Some(y) => choose_number(upbound).bind(move |z| match z {
				Some(z) if x * x + y * y == z * z => RcRun::pure(Some((x, y, z))),
				Some(_) | None => RcRun::pure(None),
			}),
			None => RcRun::pure(None),
		}),
		None => RcRun::pure(None),
	})
}

fn run_choose_values<T: Clone + 'static>(program: RcChooseProgram<T>) -> Vec<T> {
	run_choose_vec(program.map(|value| vec![value]))
}

fn run_choose_vec<T: Clone + 'static>(program: RcChooseProgram<Vec<T>>) -> Vec<T> {
	program.handle(
		handlers! {
			ChooseBrand<RcBrand>: |op: Choose<'_, RcBrand, RcChooseProgram<Vec<T>>>| {
				match op {
					Choose::Alt(k) => {
						let mut values = run_choose_vec((*k)(true));
						values.extend(run_choose_vec((*k)(false)));
						RcRun::pure(values)
					}
				}
			},
		},
		scoped_nt(),
	)
}

#[test]
fn pythagorean_search_matches_heftia_order() {
	let triples: Vec<Triple> =
		run_choose_values(pythagorean_search(16)).into_iter().flatten().collect();

	assert_eq!(
		triples,
		vec![
			(3, 4, 5),
			(4, 3, 5),
			(5, 12, 13),
			(6, 8, 10),
			(8, 6, 10),
			(9, 12, 15),
			(12, 5, 13),
			(12, 9, 15),
		]
	);
}
