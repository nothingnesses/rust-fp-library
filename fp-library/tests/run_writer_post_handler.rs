#![cfg(feature = "effects")]
#![expect(clippy::unwrap_used, reason = "Tests use panicking operations for brevity and clarity.")]

//! End-to-end tests for the standard Writer post-censor handler.
//!
//! A post-censor handler consumes the `Tell`s emitted inside the
//! selected action, combines those logs with the `Monoid` instance, and
//! emits one transformed aggregate log before the outer continuation is
//! reattached. The tests use string concatenation so per-`Tell`
//! rewriting would produce different observable logs.

use {
	fp_library::{
		brands::*,
		handlers,
		scoped_handlers,
		types::effects::{
			arc_run::ArcRun,
			arc_run_explicit::ArcRunExplicit,
			rc_run::RcRun,
			rc_run_explicit::RcRunExplicit,
			run::Run,
			run_explicit::RunExplicit,
			standard_scoped_handlers::writer_post_handler,
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

fn post_censor_log(log: String) -> String {
	format!("[{log}]")
}

type RunWriterRow = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
type RunWriterScopedRow = CoproductBrand<BoxWriterCensorBrand<BoxBrand, String>, CNilBrand>;
type RcRunWriterRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
type RcRunWriterScopedRow = CoproductBrand<WriterCensorBrand<RcBrand, String>, CNilBrand>;
type ArcRunWriterRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
type ArcRunWriterScopedRow = CoproductBrand<SendWriterCensorBrand<ArcBrand, String>, CNilBrand>;

#[test]
fn run_writer_post_handler_censors_aggregate_before_outer_continuation() {
	type Prog = Run<RunWriterRow, RunWriterScopedRow, i32>;

	let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let action: Prog =
		Run::<RunWriterRow, RunWriterScopedRow, ()>::tell::<String, _>("first".to_string())
			.bind(|()| {
				Run::<RunWriterRow, RunWriterScopedRow, ()>::tell::<String, _>("second".to_string())
			})
			.bind(|()| Run::pure(40));
	let program: Prog = Run::censor::<String, _>(post_censor_log, action).bind(|value| {
		Run::<RunWriterRow, RunWriterScopedRow, ()>::tell::<String, _>("outer".to_string())
			.bind(move |()| Run::pure(value + 2))
	});

	let result = program.handle(
		handlers! {
			WriterBrand<String>: move |op: Writer<'_, String, Prog>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.borrow_mut().push(log);
					next
				}
			},
		},
		scoped_handlers! {
			BoxWriterCensorBrand<BoxBrand, String>: writer_post_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, 42);
	assert_eq!(*log.borrow(), vec!["[firstsecond]".to_string(), "outer".to_string()]);
}

#[test]
fn rc_run_writer_post_handler_censors_aggregate_before_outer_continuation() {
	type Prog = RcRun<RcRunWriterRow, RcRunWriterScopedRow, i32>;

	let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let action: Prog =
		RcRun::<RcRunWriterRow, RcRunWriterScopedRow, ()>::tell::<String, _>("first".to_string())
			.bind(|()| {
				RcRun::<RcRunWriterRow, RcRunWriterScopedRow, ()>::tell::<String, _>(
					"second".to_string(),
				)
			})
			.bind(|()| RcRun::pure(40));
	let program: Prog = RcRun::censor::<String, _>(post_censor_log, action).bind(|value| {
		RcRun::<RcRunWriterRow, RcRunWriterScopedRow, ()>::tell::<String, _>("outer".to_string())
			.bind(move |()| RcRun::pure(value + 2))
	});

	let result = program.handle(
		handlers! {
			WriterBrand<String>: move |op: Writer<'_, String, Prog>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.borrow_mut().push(log);
					next
				}
			},
		},
		scoped_handlers! {
			WriterCensorBrand<RcBrand, String>: writer_post_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, 42);
	assert_eq!(*log.borrow(), vec!["[firstsecond]".to_string(), "outer".to_string()]);
}

#[test]
fn arc_run_writer_post_handler_censors_aggregate_before_outer_continuation() {
	type Prog = ArcRun<ArcRunWriterRow, ArcRunWriterScopedRow, i32>;

	let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
	let log_for_handler = Arc::clone(&log);
	let action: Prog = ArcRun::<ArcRunWriterRow, ArcRunWriterScopedRow, ()>::tell::<String, _>(
		"first".to_string(),
	)
	.bind(|()| {
		ArcRun::<ArcRunWriterRow, ArcRunWriterScopedRow, ()>::tell::<String, _>(
			"second".to_string(),
		)
	})
	.bind(|()| ArcRun::pure(40));
	let program: Prog = ArcRun::censor::<String, _>(post_censor_log, action).bind(|value| {
		ArcRun::<ArcRunWriterRow, ArcRunWriterScopedRow, ()>::tell::<String, _>("outer".to_string())
			.bind(move |()| ArcRun::pure(value + 2))
	});

	let result = program.handle(
		handlers! {
			WriterBrand<String>: move |op: Writer<'_, String, Prog>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.lock().unwrap().push(log);
					next
				}
			},
		},
		scoped_handlers! {
			SendWriterCensorBrand<ArcBrand, String>: writer_post_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, 42);
	assert_eq!(*log.lock().unwrap(), vec!["[firstsecond]".to_string(), "outer".to_string()]);
}

#[test]
fn run_explicit_writer_post_handler_censors_aggregate_before_outer_continuation() {
	type Prog = RunExplicit<'static, RunWriterRow, RunWriterScopedRow, i32>;

	let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let action: Prog = RunExplicit::<'static, RunWriterRow, RunWriterScopedRow, ()>::tell::<
		String,
		_,
	>("first".to_string())
	.bind(|()| {
		RunExplicit::<'static, RunWriterRow, RunWriterScopedRow, ()>::tell::<String, _>(
			"second".to_string(),
		)
	})
	.bind(|()| RunExplicit::pure(40));
	let boundary = RunExplicit::censor::<String, _>(post_censor_log, action).bind(|value| {
		RunExplicit::<'static, RunWriterRow, RunWriterScopedRow, ()>::tell::<String, _>(
			"outer".to_string(),
		)
		.bind(move |()| RunExplicit::pure(value + 2))
	});

	let result = boundary.handle(
		handlers! {
			WriterBrand<String>: move |op: Writer<'_, String, Prog>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.borrow_mut().push(log);
					next
				}
			},
		},
		scoped_handlers! {
			BoxWriterCensorBrand<BoxBrand, String>: writer_post_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, 42);
	assert_eq!(*log.borrow(), vec!["[firstsecond]".to_string(), "outer".to_string()]);
}

#[test]
fn rc_run_explicit_writer_post_handler_censors_aggregate_before_outer_continuation() {
	type Prog = RcRunExplicit<'static, RcRunWriterRow, RcRunWriterScopedRow, i32>;

	let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let action = RcRunExplicit::<'static, RcRunWriterRow, RcRunWriterScopedRow, ()>::tell::<
		String,
		_,
	>("first".to_string())
	.bind(|()| {
		RcRunExplicit::<'static, RcRunWriterRow, RcRunWriterScopedRow, ()>::tell::<String, _>(
			"second".to_string(),
		)
	})
	.bind(|()| RcRunExplicit::pure(40));
	let boundary = RcRunExplicit::censor::<String, _>(post_censor_log, action).bind(|value| {
		RcRunExplicit::<'static, RcRunWriterRow, RcRunWriterScopedRow, ()>::tell::<String, _>(
			"outer".to_string(),
		)
		.bind(move |()| RcRunExplicit::pure(value + 2))
	});

	let result = boundary.handle(
		handlers! {
			WriterBrand<String>: move |op: Writer<'_, String, Prog>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.borrow_mut().push(log);
					next
				}
			},
		},
		scoped_handlers! {
			WriterCensorBrand<RcBrand, String>: writer_post_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, 42);
	assert_eq!(*log.borrow(), vec!["[firstsecond]".to_string(), "outer".to_string()]);
}

#[test]
fn arc_run_explicit_writer_post_handler_censors_aggregate_before_outer_continuation() {
	type Prog = ArcRunExplicit<'static, ArcRunWriterRow, ArcRunWriterScopedRow, i32>;

	let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
	let log_for_handler = Arc::clone(&log);
	let action = ArcRunExplicit::<'static, ArcRunWriterRow, ArcRunWriterScopedRow, ()>::tell::<
		String,
		_,
	>("first".to_string())
	.bind(|()| {
		ArcRunExplicit::<'static, ArcRunWriterRow, ArcRunWriterScopedRow, ()>::tell::<String, _>(
			"second".to_string(),
		)
	})
	.bind(|()| ArcRunExplicit::pure(40));
	let boundary = ArcRunExplicit::censor::<String, _>(post_censor_log, action).bind(|value| {
		ArcRunExplicit::<'static, ArcRunWriterRow, ArcRunWriterScopedRow, ()>::tell::<String, _>(
			"outer".to_string(),
		)
		.bind(move |()| ArcRunExplicit::pure(value + 2))
	});

	let result = boundary.handle(
		handlers! {
			WriterBrand<String>: move |op: Writer<'_, String, Prog>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.lock().unwrap().push(log);
					next
				}
			},
		},
		scoped_handlers! {
			SendWriterCensorBrand<ArcBrand, String>: writer_post_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, 42);
	assert_eq!(*log.lock().unwrap(), vec!["[firstsecond]".to_string(), "outer".to_string()]);
}
