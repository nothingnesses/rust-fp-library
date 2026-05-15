//! Focused tests for same-row first-order Writer accumulation.
//!
//! These tests exercise the wrapper-level accumulation substrate
//! directly, before the standard Writer post-handler is layered on top.
//! The accumulator consumes each lowered `Writer::Tell(w, next)` inside
//! a selected action, recursively resumes `next`, and returns the
//! action result paired with the collected log values. A fallback
//! Writer handler is still installed when interpreting the accumulated
//! program; if the selected `Tell`s were re-emitted instead of consumed,
//! the fallback handler would observe them and the assertions would
//! fail.

use {
	fp_library::{
		brands::*,
		handlers,
		types::effects::{
			arc_run::{
				ArcRun,
				ArcRunFirstOrderAccumulator,
			},
			rc_run::{
				RcRun,
				RcRunFirstOrderAccumulator,
			},
			run::{
				Run,
				RunFirstOrderAccumulator,
			},
			scoped_nt,
			writer::Writer,
		},
	},
	std::{
		cell::RefCell,
		rc::Rc,
		sync::{
			Arc,
			Mutex,
		},
	},
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct AccumulatedLog(&'static str);

fn prepend_log(
	log: AccumulatedLog,
	mut suffix: Vec<AccumulatedLog>,
) -> Vec<AccumulatedLog> {
	let mut accumulated = vec![log];
	accumulated.append(&mut suffix);
	accumulated
}

fn push_arc_log(
	log: &Mutex<Vec<AccumulatedLog>>,
	entry: AccumulatedLog,
) {
	match log.lock() {
		Ok(mut guard) => guard.push(entry),
		Err(poisoned) => poisoned.into_inner().push(entry),
	}
}

fn clone_arc_log(log: &Mutex<Vec<AccumulatedLog>>) -> Vec<AccumulatedLog> {
	match log.lock() {
		Ok(guard) => guard.clone(),
		Err(poisoned) => poisoned.into_inner().clone(),
	}
}

type RunWriterAccumulationRow =
	CoproductBrand<CoyonedaBrand<WriterBrand<AccumulatedLog>>, CNilBrand>;
type RcRunWriterAccumulationRow =
	CoproductBrand<RcCoyonedaBrand<WriterBrand<AccumulatedLog>>, CNilBrand>;
type ArcRunWriterAccumulationRow =
	CoproductBrand<ArcCoyonedaBrand<WriterBrand<AccumulatedLog>>, CNilBrand>;
type AccumulatedResult = (i32, Vec<AccumulatedLog>);
type RunAccumulatedWriterOp<'a> =
	Writer<'a, AccumulatedLog, Run<RunWriterAccumulationRow, CNilBrand, AccumulatedResult>>;
type RcRunAccumulatedWriterOp<'a> =
	Writer<'a, AccumulatedLog, RcRun<RcRunWriterAccumulationRow, CNilBrand, AccumulatedResult>>;
type ArcRunAccumulatedWriterOp<'a> =
	Writer<'a, AccumulatedLog, ArcRun<ArcRunWriterAccumulationRow, CNilBrand, AccumulatedResult>>;

struct RunTellAccumulator;

impl
	RunFirstOrderAccumulator<
		WriterBrand<AccumulatedLog>,
		RunWriterAccumulationRow,
		CNilBrand,
		Vec<AccumulatedLog>,
	> for RunTellAccumulator
{
	fn empty(&self) -> Vec<AccumulatedLog> {
		Vec::new()
	}

	fn accumulate<T: 'static>(
		&self,
		effect: Writer<
			'static,
			AccumulatedLog,
			Run<RunWriterAccumulationRow, CNilBrand, (T, Vec<AccumulatedLog>)>,
		>,
	) -> Run<RunWriterAccumulationRow, CNilBrand, (T, Vec<AccumulatedLog>)> {
		match effect {
			Writer::Tell(log, next, _) =>
				next.map(move |(value, suffix)| (value, prepend_log(log, suffix))),
		}
	}
}

struct RcRunTellAccumulator;

impl
	RcRunFirstOrderAccumulator<
		WriterBrand<AccumulatedLog>,
		RcRunWriterAccumulationRow,
		CNilBrand,
		Vec<AccumulatedLog>,
	> for RcRunTellAccumulator
{
	fn empty(&self) -> Vec<AccumulatedLog> {
		Vec::new()
	}

	fn accumulate<T: Clone + 'static>(
		&self,
		effect: Writer<
			'static,
			AccumulatedLog,
			RcRun<RcRunWriterAccumulationRow, CNilBrand, (T, Vec<AccumulatedLog>)>,
		>,
	) -> RcRun<RcRunWriterAccumulationRow, CNilBrand, (T, Vec<AccumulatedLog>)> {
		match effect {
			Writer::Tell(log, next, _) =>
				next.map(move |(value, suffix)| (value, prepend_log(log.clone(), suffix))),
		}
	}
}

struct ArcRunTellAccumulator;

impl
	ArcRunFirstOrderAccumulator<
		WriterBrand<AccumulatedLog>,
		ArcRunWriterAccumulationRow,
		CNilBrand,
		Vec<AccumulatedLog>,
	> for ArcRunTellAccumulator
{
	fn empty(&self) -> Vec<AccumulatedLog> {
		Vec::new()
	}

	fn accumulate<T: Clone + Send + Sync + 'static>(
		&self,
		effect: Writer<
			'static,
			AccumulatedLog,
			ArcRun<ArcRunWriterAccumulationRow, CNilBrand, (T, Vec<AccumulatedLog>)>,
		>,
	) -> ArcRun<ArcRunWriterAccumulationRow, CNilBrand, (T, Vec<AccumulatedLog>)> {
		match effect {
			Writer::Tell(log, next, _) =>
				next.map(move |(value, suffix)| (value, prepend_log(log.clone(), suffix))),
		}
	}
}

#[test]
fn run_accumulator_removes_tells_and_returns_value_with_logs() {
	let observed_tells: Rc<RefCell<Vec<AccumulatedLog>>> = Rc::new(RefCell::new(Vec::new()));
	let observed_tells_for_handler = Rc::clone(&observed_tells);
	let prog: Run<RunWriterAccumulationRow, CNilBrand, i32> =
		Run::<RunWriterAccumulationRow, CNilBrand, ()>::tell::<AccumulatedLog, _>(AccumulatedLog(
			"first",
		))
		.bind(|()| {
			Run::<RunWriterAccumulationRow, CNilBrand, ()>::tell::<AccumulatedLog, _>(
				AccumulatedLog("second"),
			)
		})
		.map(|()| 20)
		.bind(|value| Run::pure(value + 22));
	let accumulated = prog
		.accumulate_with_first_order::<WriterBrand<AccumulatedLog>, _, CNilBrand, _, Vec<AccumulatedLog>>(
			RunTellAccumulator,
		);
	let result = accumulated.handle(
		handlers! {
			WriterBrand<AccumulatedLog>: move |op: RunAccumulatedWriterOp<'_>| {
				match op {
					Writer::Tell(log, next, _) => {
						observed_tells_for_handler.borrow_mut().push(log);
						next
					}
				}
			},
		},
		scoped_nt(),
	);

	assert_eq!(result, (42, vec![AccumulatedLog("first"), AccumulatedLog("second")]));
	assert_eq!(*observed_tells.borrow(), Vec::<AccumulatedLog>::new());
}

#[test]
fn rc_run_accumulator_restarts_with_fresh_accumulated_state() {
	let observed_tells: Rc<RefCell<Vec<AccumulatedLog>>> = Rc::new(RefCell::new(Vec::new()));
	let observed_tells_for_first_handler = Rc::clone(&observed_tells);
	let observed_tells_for_second_handler = Rc::clone(&observed_tells);
	let prog: RcRun<RcRunWriterAccumulationRow, CNilBrand, i32> =
		RcRun::<RcRunWriterAccumulationRow, CNilBrand, ()>::tell::<AccumulatedLog, _>(
			AccumulatedLog("first"),
		)
		.bind(|()| {
			RcRun::<RcRunWriterAccumulationRow, CNilBrand, ()>::tell::<AccumulatedLog, _>(
				AccumulatedLog("second"),
			)
		})
		.map(|()| 20)
		.bind(|value| RcRun::pure(value + 22));
	let accumulated = prog
		.accumulate_with_first_order::<WriterBrand<AccumulatedLog>, _, CNilBrand, _, Vec<AccumulatedLog>>(
			RcRunTellAccumulator,
		);
	let first_result = accumulated.clone().handle(
		handlers! {
			WriterBrand<AccumulatedLog>: move |op: RcRunAccumulatedWriterOp<'_>| {
				match op {
					Writer::Tell(log, next, _) => {
						observed_tells_for_first_handler.borrow_mut().push(log);
						next
					}
				}
			},
		},
		scoped_nt(),
	);
	let second_result = accumulated.handle(
		handlers! {
			WriterBrand<AccumulatedLog>: move |op: RcRunAccumulatedWriterOp<'_>| {
				match op {
					Writer::Tell(log, next, _) => {
						observed_tells_for_second_handler.borrow_mut().push(log);
						next
					}
				}
			},
		},
		scoped_nt(),
	);

	assert_eq!(first_result, (42, vec![AccumulatedLog("first"), AccumulatedLog("second")]));
	assert_eq!(second_result, first_result);
	assert_eq!(*observed_tells.borrow(), Vec::<AccumulatedLog>::new());
}

#[test]
fn arc_run_accumulator_restarts_with_fresh_accumulated_state() {
	let observed_tells: Arc<Mutex<Vec<AccumulatedLog>>> = Arc::new(Mutex::new(Vec::new()));
	let observed_tells_for_first_handler = Arc::clone(&observed_tells);
	let observed_tells_for_second_handler = Arc::clone(&observed_tells);
	let prog: ArcRun<ArcRunWriterAccumulationRow, CNilBrand, i32> =
		ArcRun::<ArcRunWriterAccumulationRow, CNilBrand, ()>::tell::<AccumulatedLog, _>(
			AccumulatedLog("first"),
		)
		.bind(|()| {
			ArcRun::<ArcRunWriterAccumulationRow, CNilBrand, ()>::tell::<AccumulatedLog, _>(
				AccumulatedLog("second"),
			)
		})
		.map(|()| 20)
		.bind(|value| ArcRun::pure(value + 22));
	let accumulated = prog
		.accumulate_with_first_order::<WriterBrand<AccumulatedLog>, _, CNilBrand, _, Vec<AccumulatedLog>>(
			ArcRunTellAccumulator,
		);
	let first_result = accumulated.clone().handle(
		handlers! {
			WriterBrand<AccumulatedLog>: move |op: ArcRunAccumulatedWriterOp<'_>| {
				match op {
					Writer::Tell(log, next, _) => {
						push_arc_log(&observed_tells_for_first_handler, log);
						next
					}
				}
			},
		},
		scoped_nt(),
	);
	let second_result = accumulated.handle(
		handlers! {
			WriterBrand<AccumulatedLog>: move |op: ArcRunAccumulatedWriterOp<'_>| {
				match op {
					Writer::Tell(log, next, _) => {
						push_arc_log(&observed_tells_for_second_handler, log);
						next
					}
				}
			},
		},
		scoped_nt(),
	);

	assert_eq!(first_result, (42, vec![AccumulatedLog("first"), AccumulatedLog("second")]));
	assert_eq!(second_result, first_result);
	assert_eq!(clone_arc_log(&observed_tells), Vec::<AccumulatedLog>::new());
}
