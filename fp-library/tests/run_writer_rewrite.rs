#![cfg(feature = "effects")]

//! Focused tests for same-row first-order Writer rewrites.
//!
//! These tests exercise the wrapper-level rewrite substrate directly,
//! before the standard Writer pre-handler is layered on top. The
//! rewriter transforms each lowered `Writer::Tell(w, next)` into
//! `Writer::Tell(censor(w), next)` while preserving the operation
//! constructor, the first-order row, and the pending continuation. The
//! log type intentionally has no `Semigroup` or `Monoid` implementation;
//! pre-applying a censor function to individual `Tell` operations should
//! not need log accumulation.

use {
	fp_library::{
		brands::*,
		handlers,
		types::effects::{
			arc_run::{
				ArcRun,
				ArcRunFirstOrderRewriter,
			},
			arc_run_explicit::{
				ArcRunExplicit,
				ArcRunExplicitFirstOrderRewriter,
			},
			rc_run::{
				RcRun,
				RcRunFirstOrderRewriter,
			},
			rc_run_explicit::{
				RcRunExplicit,
				RcRunExplicitFirstOrderRewriter,
			},
			run::{
				Run,
				RunFirstOrderRewriter,
			},
			run_explicit::{
				RunExplicit,
				RunExplicitFirstOrderRewriter,
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
struct NoMonoidLog(&'static str);

fn censor_log(log: NoMonoidLog) -> NoMonoidLog {
	match log.0 {
		"first" => NoMonoidLog("censored:first"),
		"second" => NoMonoidLog("censored:second"),
		other => NoMonoidLog(other),
	}
}

fn push_arc_log(
	log: &Mutex<Vec<NoMonoidLog>>,
	entry: NoMonoidLog,
) {
	match log.lock() {
		Ok(mut guard) => guard.push(entry),
		Err(poisoned) => poisoned.into_inner().push(entry),
	}
}

fn clone_arc_log(log: &Mutex<Vec<NoMonoidLog>>) -> Vec<NoMonoidLog> {
	match log.lock() {
		Ok(guard) => guard.clone(),
		Err(poisoned) => poisoned.into_inner().clone(),
	}
}

type RunWriterRewriteRow = CoproductBrand<CoyonedaBrand<WriterBrand<NoMonoidLog>>, CNilBrand>;
type RcRunWriterRewriteRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<NoMonoidLog>>, CNilBrand>;
type ArcRunWriterRewriteRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<NoMonoidLog>>, CNilBrand>;
type RunExplicitWriterRewriteRow =
	CoproductBrand<CoyonedaBrand<WriterBrand<NoMonoidLog>>, CNilBrand>;
type RcRunExplicitWriterRewriteRow =
	CoproductBrand<RcCoyonedaBrand<WriterBrand<NoMonoidLog>>, CNilBrand>;
type ArcRunExplicitWriterRewriteRow =
	CoproductBrand<ArcCoyonedaBrand<WriterBrand<NoMonoidLog>>, CNilBrand>;

struct RunTellCensor;

impl RunFirstOrderRewriter<WriterBrand<NoMonoidLog>, RunWriterRewriteRow, CNilBrand>
	for RunTellCensor
{
	fn rewrite<T: 'static>(
		&self,
		effect: Writer<'static, NoMonoidLog, Run<RunWriterRewriteRow, CNilBrand, T>>,
	) -> Writer<'static, NoMonoidLog, Run<RunWriterRewriteRow, CNilBrand, T>> {
		match effect {
			Writer::Tell(log, next, marker) => Writer::Tell(censor_log(log), next, marker),
		}
	}
}

struct RcRunTellCensor;

impl RcRunFirstOrderRewriter<WriterBrand<NoMonoidLog>, RcRunWriterRewriteRow, CNilBrand>
	for RcRunTellCensor
{
	fn rewrite<T: Clone + 'static>(
		&self,
		effect: Writer<'static, NoMonoidLog, RcRun<RcRunWriterRewriteRow, CNilBrand, T>>,
	) -> Writer<'static, NoMonoidLog, RcRun<RcRunWriterRewriteRow, CNilBrand, T>> {
		match effect {
			Writer::Tell(log, next, marker) => Writer::Tell(censor_log(log), next, marker),
		}
	}
}

struct ArcRunTellCensor;

impl ArcRunFirstOrderRewriter<WriterBrand<NoMonoidLog>, ArcRunWriterRewriteRow, CNilBrand>
	for ArcRunTellCensor
{
	fn rewrite<T: Clone + Send + Sync + 'static>(
		&self,
		effect: Writer<'static, NoMonoidLog, ArcRun<ArcRunWriterRewriteRow, CNilBrand, T>>,
	) -> Writer<'static, NoMonoidLog, ArcRun<ArcRunWriterRewriteRow, CNilBrand, T>> {
		match effect {
			Writer::Tell(log, next, marker) => Writer::Tell(censor_log(log), next, marker),
		}
	}
}

struct RunExplicitTellCensor;

impl<'a>
	RunExplicitFirstOrderRewriter<
		'a,
		WriterBrand<NoMonoidLog>,
		RunExplicitWriterRewriteRow,
		CNilBrand,
	> for RunExplicitTellCensor
{
	fn rewrite<T: 'a>(
		&self,
		effect: Writer<'a, NoMonoidLog, RunExplicit<'a, RunExplicitWriterRewriteRow, CNilBrand, T>>,
	) -> Writer<'a, NoMonoidLog, RunExplicit<'a, RunExplicitWriterRewriteRow, CNilBrand, T>> {
		match effect {
			Writer::Tell(log, next, marker) => Writer::Tell(censor_log(log), next, marker),
		}
	}
}

struct RcRunExplicitTellCensor;

impl<'a>
	RcRunExplicitFirstOrderRewriter<
		'a,
		WriterBrand<NoMonoidLog>,
		RcRunExplicitWriterRewriteRow,
		CNilBrand,
	> for RcRunExplicitTellCensor
{
	fn rewrite<T: Clone + 'a>(
		&self,
		effect: Writer<
			'a,
			NoMonoidLog,
			RcRunExplicit<'a, RcRunExplicitWriterRewriteRow, CNilBrand, T>,
		>,
	) -> Writer<'a, NoMonoidLog, RcRunExplicit<'a, RcRunExplicitWriterRewriteRow, CNilBrand, T>> {
		match effect {
			Writer::Tell(log, next, marker) => Writer::Tell(censor_log(log), next, marker),
		}
	}
}

struct ArcRunExplicitTellCensor;

impl<'a>
	ArcRunExplicitFirstOrderRewriter<
		'a,
		WriterBrand<NoMonoidLog>,
		ArcRunExplicitWriterRewriteRow,
		CNilBrand,
	> for ArcRunExplicitTellCensor
{
	fn rewrite<T: Clone + Send + Sync + 'a>(
		&self,
		effect: Writer<
			'a,
			NoMonoidLog,
			ArcRunExplicit<'a, ArcRunExplicitWriterRewriteRow, CNilBrand, T>,
		>,
	) -> Writer<'a, NoMonoidLog, ArcRunExplicit<'a, ArcRunExplicitWriterRewriteRow, CNilBrand, T>>
	{
		match effect {
			Writer::Tell(log, next, marker) => Writer::Tell(censor_log(log), next, marker),
		}
	}
}

#[test]
fn run_rewriter_preserves_tell_operations_and_continuations() {
	let log: Rc<RefCell<Vec<NoMonoidLog>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let prog: Run<RunWriterRewriteRow, CNilBrand, i32> =
		Run::<RunWriterRewriteRow, CNilBrand, ()>::tell::<NoMonoidLog, _>(NoMonoidLog("first"))
			.bind(|()| {
				Run::<RunWriterRewriteRow, CNilBrand, ()>::tell::<NoMonoidLog, _>(NoMonoidLog(
					"second",
				))
			})
			.map(|()| 20)
			.bind(|value| Run::pure(value + 22));
	let rewritten: Run<RunWriterRewriteRow, CNilBrand, i32> =
		prog.interpose_with_rewriter::<WriterBrand<NoMonoidLog>, _, CNilBrand, _>(RunTellCensor);
	let result = rewritten.handle(
		handlers! {
			WriterBrand<NoMonoidLog>: move |op: Writer<'_, NoMonoidLog, Run<RunWriterRewriteRow, CNilBrand, i32>>| {
				match op {
					Writer::Tell(w, next, _) => {
						log_for_handler.borrow_mut().push(w);
						next
					}
				}
			},
		},
		scoped_nt(),
	);

	assert_eq!(result, 42);
	assert_eq!(*log.borrow(), vec![NoMonoidLog("censored:first"), NoMonoidLog("censored:second")]);
}

#[test]
fn rc_run_rewriter_preserves_tell_operations_and_continuations() {
	let log: Rc<RefCell<Vec<NoMonoidLog>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let prog: RcRun<RcRunWriterRewriteRow, CNilBrand, i32> =
		RcRun::<RcRunWriterRewriteRow, CNilBrand, ()>::tell::<NoMonoidLog, _>(NoMonoidLog("first"))
			.bind(|()| {
				RcRun::<RcRunWriterRewriteRow, CNilBrand, ()>::tell::<NoMonoidLog, _>(NoMonoidLog(
					"second",
				))
			})
			.map(|()| 20)
			.bind(|value| RcRun::pure(value + 22));
	let rewritten: RcRun<RcRunWriterRewriteRow, CNilBrand, i32> =
		prog.interpose_with_rewriter::<WriterBrand<NoMonoidLog>, _, CNilBrand, _>(RcRunTellCensor);
	let result = rewritten.handle(
		handlers! {
			WriterBrand<NoMonoidLog>: move |op: Writer<'_, NoMonoidLog, RcRun<RcRunWriterRewriteRow, CNilBrand, i32>>| {
				match op {
					Writer::Tell(w, next, _) => {
						log_for_handler.borrow_mut().push(w);
						next
					}
				}
			},
		},
		scoped_nt(),
	);

	assert_eq!(result, 42);
	assert_eq!(*log.borrow(), vec![NoMonoidLog("censored:first"), NoMonoidLog("censored:second")]);
}

#[test]
fn arc_run_rewriter_preserves_tell_operations_and_continuations() {
	let log: Arc<Mutex<Vec<NoMonoidLog>>> = Arc::new(Mutex::new(Vec::new()));
	let log_for_handler = Arc::clone(&log);
	let prog: ArcRun<ArcRunWriterRewriteRow, CNilBrand, i32> =
		ArcRun::<ArcRunWriterRewriteRow, CNilBrand, ()>::tell::<NoMonoidLog, _>(NoMonoidLog(
			"first",
		))
		.bind(|()| {
			ArcRun::<ArcRunWriterRewriteRow, CNilBrand, ()>::tell::<NoMonoidLog, _>(NoMonoidLog(
				"second",
			))
		})
		.map(|()| 20)
		.bind(|value| ArcRun::pure(value + 22));
	let rewritten: ArcRun<ArcRunWriterRewriteRow, CNilBrand, i32> =
		prog.interpose_with_rewriter::<WriterBrand<NoMonoidLog>, _, CNilBrand, _>(ArcRunTellCensor);
	let result = rewritten.handle(
		handlers! {
			WriterBrand<NoMonoidLog>: move |op: Writer<'_, NoMonoidLog, ArcRun<ArcRunWriterRewriteRow, CNilBrand, i32>>| {
				match op {
					Writer::Tell(w, next, _) => {
						push_arc_log(&log_for_handler, w);
						next
					}
				}
			},
		},
		scoped_nt(),
	);

	assert_eq!(result, 42);
	assert_eq!(
		clone_arc_log(&log),
		vec![NoMonoidLog("censored:first"), NoMonoidLog("censored:second")]
	);
}

#[test]
fn run_explicit_rewriter_preserves_tell_operations_and_continuations() {
	let log: Rc<RefCell<Vec<NoMonoidLog>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let prog: RunExplicit<'static, RunExplicitWriterRewriteRow, CNilBrand, i32> =
		RunExplicit::<'static, RunExplicitWriterRewriteRow, CNilBrand, ()>::tell::<NoMonoidLog, _>(
			NoMonoidLog("first"),
		)
		.bind(|()| {
			RunExplicit::<'static, RunExplicitWriterRewriteRow, CNilBrand, ()>::tell::<NoMonoidLog, _>(
				NoMonoidLog("second"),
			)
		})
		.map(|()| 20)
		.bind(|value| RunExplicit::pure(value + 22));
	let rewritten: RunExplicit<'static, RunExplicitWriterRewriteRow, CNilBrand, i32> =
		prog.interpose_with_rewriter::<WriterBrand<NoMonoidLog>, _, CNilBrand, _>(
			RunExplicitTellCensor,
		);
	let result = rewritten.handle(
		handlers! {
			WriterBrand<NoMonoidLog>: move |op: Writer<'_, NoMonoidLog, RunExplicit<'static, RunExplicitWriterRewriteRow, CNilBrand, i32>>| {
				match op {
					Writer::Tell(w, next, _) => {
						log_for_handler.borrow_mut().push(w);
						next
					}
				}
			},
		},
		scoped_nt(),
	);

	assert_eq!(result, 42);
	assert_eq!(*log.borrow(), vec![NoMonoidLog("censored:first"), NoMonoidLog("censored:second")]);
}

#[test]
fn rc_run_explicit_rewriter_preserves_tell_operations_and_continuations() {
	let log: Rc<RefCell<Vec<NoMonoidLog>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let prog: RcRunExplicit<'static, RcRunExplicitWriterRewriteRow, CNilBrand, i32> =
		RcRunExplicit::<'static, RcRunExplicitWriterRewriteRow, CNilBrand, ()>::tell::<
			NoMonoidLog,
			_,
		>(NoMonoidLog("first"))
		.bind(|()| {
			RcRunExplicit::<'static, RcRunExplicitWriterRewriteRow, CNilBrand, ()>::tell::<
				NoMonoidLog,
				_,
			>(NoMonoidLog("second"))
		})
		.map(|()| 20)
		.bind(|value| RcRunExplicit::pure(value + 22));
	let rewritten: RcRunExplicit<'static, RcRunExplicitWriterRewriteRow, CNilBrand, i32> =
		prog.interpose_with_rewriter::<WriterBrand<NoMonoidLog>, _, CNilBrand, _>(
			RcRunExplicitTellCensor,
		);
	let result = rewritten.handle(
		handlers! {
			WriterBrand<NoMonoidLog>: move |op: Writer<'_, NoMonoidLog, RcRunExplicit<'static, RcRunExplicitWriterRewriteRow, CNilBrand, i32>>| {
				match op {
					Writer::Tell(w, next, _) => {
						log_for_handler.borrow_mut().push(w);
						next
					}
				}
			},
		},
		scoped_nt(),
	);

	assert_eq!(result, 42);
	assert_eq!(*log.borrow(), vec![NoMonoidLog("censored:first"), NoMonoidLog("censored:second")]);
}

#[test]
fn arc_run_explicit_rewriter_preserves_tell_operations_and_continuations() {
	let log: Arc<Mutex<Vec<NoMonoidLog>>> = Arc::new(Mutex::new(Vec::new()));
	let log_for_handler = Arc::clone(&log);
	let prog: ArcRunExplicit<'static, ArcRunExplicitWriterRewriteRow, CNilBrand, i32> =
		ArcRunExplicit::<'static, ArcRunExplicitWriterRewriteRow, CNilBrand, ()>::tell::<
			NoMonoidLog,
			_,
		>(NoMonoidLog("first"))
		.bind(|()| {
			ArcRunExplicit::<'static, ArcRunExplicitWriterRewriteRow, CNilBrand, ()>::tell::<
				NoMonoidLog,
				_,
			>(NoMonoidLog("second"))
		})
		.map(|()| 20)
		.bind(|value| ArcRunExplicit::pure(value + 22));
	let rewritten: ArcRunExplicit<'static, ArcRunExplicitWriterRewriteRow, CNilBrand, i32> =
		prog.interpose_with_rewriter::<WriterBrand<NoMonoidLog>, _, CNilBrand, _>(
			ArcRunExplicitTellCensor,
		);
	let result = rewritten.handle(
		handlers! {
			WriterBrand<NoMonoidLog>: move |op: Writer<'_, NoMonoidLog, ArcRunExplicit<'static, ArcRunExplicitWriterRewriteRow, CNilBrand, i32>>| {
				match op {
					Writer::Tell(w, next, _) => {
						push_arc_log(&log_for_handler, w);
						next
					}
				}
			},
		},
		scoped_nt(),
	);

	assert_eq!(result, 42);
	assert_eq!(
		clone_arc_log(&log),
		vec![NoMonoidLog("censored:first"), NoMonoidLog("censored:second")]
	);
}
