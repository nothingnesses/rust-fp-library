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
			rc_run::{
				RcRun,
				RcRunFirstOrderRewriter,
			},
			run::{
				Run,
				RunFirstOrderRewriter,
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
