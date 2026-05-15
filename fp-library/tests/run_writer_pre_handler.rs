#![expect(clippy::unwrap_used, reason = "Tests use panicking operations for brevity and clarity.")]

//! End-to-end tests for the standard Writer pre-censor handler.
//!
//! A pre-censor handler applies the stored `censor` function to each
//! `Tell` inside the selected action before the surrounding first-order
//! Writer handler observes the log. The outer continuation is reattached
//! after the selected action, so Writer operations produced by that
//! continuation remain uncensored and run afterward.

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
			standard_scoped_handlers::writer_pre_handler,
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

fn censor_log(log: &'static str) -> &'static str {
	match log {
		"first" => "censored:first",
		"second" => "censored:second",
		other => other,
	}
}

type RunWriterRow = CoproductBrand<CoyonedaBrand<WriterBrand<&'static str>>, CNilBrand>;
type RunWriterScopedRow = CoproductBrand<BoxWriterCensorBrand<BoxBrand, &'static str>, CNilBrand>;
type RcRunWriterRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<&'static str>>, CNilBrand>;
type RcRunWriterScopedRow = CoproductBrand<WriterCensorBrand<RcBrand, &'static str>, CNilBrand>;
type ArcRunWriterRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<&'static str>>, CNilBrand>;
type ArcRunWriterScopedRow =
	CoproductBrand<SendWriterCensorBrand<ArcBrand, &'static str>, CNilBrand>;

#[test]
fn run_writer_pre_handler_censors_selected_tells_before_outer_continuation() {
	type Prog = Run<RunWriterRow, RunWriterScopedRow, i32>;

	let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let action: Prog =
		Run::<RunWriterRow, RunWriterScopedRow, ()>::tell::<&'static str, _>("first")
			.bind(|()| {
				Run::<RunWriterRow, RunWriterScopedRow, ()>::tell::<&'static str, _>("second")
			})
			.bind(|()| Run::pure(40));
	let program: Prog = Run::censor::<&'static str, _>(censor_log, action).bind(|value| {
		Run::<RunWriterRow, RunWriterScopedRow, ()>::tell::<&'static str, _>("outer")
			.bind(move |()| Run::pure(value + 2))
	});

	let result = program.handle(
		handlers! {
			WriterBrand<&'static str>: move |op: Writer<'_, &'static str, Prog>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.borrow_mut().push(log);
					next
				}
			},
		},
		scoped_handlers! {
			BoxWriterCensorBrand<BoxBrand, &'static str>: writer_pre_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, 42);
	assert_eq!(*log.borrow(), vec!["censored:first", "censored:second", "outer"]);
}

#[test]
fn rc_run_writer_pre_handler_censors_selected_tells_before_outer_continuation() {
	type Prog = RcRun<RcRunWriterRow, RcRunWriterScopedRow, i32>;

	let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let action: Prog =
		RcRun::<RcRunWriterRow, RcRunWriterScopedRow, ()>::tell::<&'static str, _>("first")
			.bind(|()| {
				RcRun::<RcRunWriterRow, RcRunWriterScopedRow, ()>::tell::<&'static str, _>("second")
			})
			.bind(|()| RcRun::pure(40));
	let program: Prog = RcRun::censor::<&'static str, _>(censor_log, action).bind(|value| {
		RcRun::<RcRunWriterRow, RcRunWriterScopedRow, ()>::tell::<&'static str, _>("outer")
			.bind(move |()| RcRun::pure(value + 2))
	});

	let result = program.handle(
		handlers! {
			WriterBrand<&'static str>: move |op: Writer<'_, &'static str, Prog>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.borrow_mut().push(log);
					next
				}
			},
		},
		scoped_handlers! {
			WriterCensorBrand<RcBrand, &'static str>: writer_pre_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, 42);
	assert_eq!(*log.borrow(), vec!["censored:first", "censored:second", "outer"]);
}

#[test]
fn arc_run_writer_pre_handler_censors_selected_tells_before_outer_continuation() {
	type Prog = ArcRun<ArcRunWriterRow, ArcRunWriterScopedRow, i32>;

	let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
	let log_for_handler = Arc::clone(&log);
	let action: Prog =
		ArcRun::<ArcRunWriterRow, ArcRunWriterScopedRow, ()>::tell::<&'static str, _>("first")
			.bind(|()| {
				ArcRun::<ArcRunWriterRow, ArcRunWriterScopedRow, ()>::tell::<&'static str, _>(
					"second",
				)
			})
			.bind(|()| ArcRun::pure(40));
	let program: Prog = ArcRun::censor::<&'static str, _>(censor_log, action).bind(|value| {
		ArcRun::<ArcRunWriterRow, ArcRunWriterScopedRow, ()>::tell::<&'static str, _>("outer")
			.bind(move |()| ArcRun::pure(value + 2))
	});

	let result = program.handle(
		handlers! {
			WriterBrand<&'static str>: move |op: Writer<'_, &'static str, Prog>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.lock().unwrap().push(log);
					next
				}
			},
		},
		scoped_handlers! {
			SendWriterCensorBrand<ArcBrand, &'static str>: writer_pre_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, 42);
	assert_eq!(*log.lock().unwrap(), vec!["censored:first", "censored:second", "outer"]);
}

#[test]
fn run_explicit_writer_pre_handler_censors_selected_tells_before_outer_continuation() {
	type Prog = RunExplicit<'static, RunWriterRow, RunWriterScopedRow, i32>;

	let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let action: Prog = RunExplicit::<'static, RunWriterRow, RunWriterScopedRow, ()>::tell::<
		&'static str,
		_,
	>("first")
	.bind(|()| {
		RunExplicit::<'static, RunWriterRow, RunWriterScopedRow, ()>::tell::<&'static str, _>(
			"second",
		)
	})
	.bind(|()| RunExplicit::pure(40));
	let boundary = RunExplicit::censor::<&'static str, _>(censor_log, action).bind(|value| {
		RunExplicit::<'static, RunWriterRow, RunWriterScopedRow, ()>::tell::<&'static str, _>(
			"outer",
		)
		.bind(move |()| RunExplicit::pure(value + 2))
	});

	let result = boundary.handle(
		handlers! {
			WriterBrand<&'static str>: move |op: Writer<'_, &'static str, Prog>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.borrow_mut().push(log);
					next
				}
			},
		},
		scoped_handlers! {
			BoxWriterCensorBrand<BoxBrand, &'static str>: writer_pre_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, 42);
	assert_eq!(*log.borrow(), vec!["censored:first", "censored:second", "outer"]);
}

#[test]
fn rc_run_explicit_writer_pre_handler_censors_selected_tells_before_outer_continuation() {
	type Prog = RcRunExplicit<'static, RcRunWriterRow, RcRunWriterScopedRow, i32>;

	let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let action = RcRunExplicit::<'static, RcRunWriterRow, RcRunWriterScopedRow, ()>::tell::<
		&'static str,
		_,
	>("first")
	.bind(|()| {
		RcRunExplicit::<'static, RcRunWriterRow, RcRunWriterScopedRow, ()>::tell::<&'static str, _>(
			"second",
		)
	})
	.bind(|()| RcRunExplicit::pure(40));
	let boundary = RcRunExplicit::censor::<&'static str, _>(censor_log, action).bind(|value| {
		RcRunExplicit::<'static, RcRunWriterRow, RcRunWriterScopedRow, ()>::tell::<&'static str, _>(
			"outer",
		)
		.bind(move |()| RcRunExplicit::pure(value + 2))
	});

	let result = boundary.handle(
		handlers! {
			WriterBrand<&'static str>: move |op: Writer<'_, &'static str, Prog>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.borrow_mut().push(log);
					next
				}
			},
		},
		scoped_handlers! {
			WriterCensorBrand<RcBrand, &'static str>: writer_pre_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, 42);
	assert_eq!(*log.borrow(), vec!["censored:first", "censored:second", "outer"]);
}

#[test]
fn arc_run_explicit_writer_pre_handler_censors_selected_tells_before_outer_continuation() {
	type Prog = ArcRunExplicit<'static, ArcRunWriterRow, ArcRunWriterScopedRow, i32>;

	let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
	let log_for_handler = Arc::clone(&log);
	let action = ArcRunExplicit::<'static, ArcRunWriterRow, ArcRunWriterScopedRow, ()>::tell::<
		&'static str,
		_,
	>("first")
	.bind(|()| {
		ArcRunExplicit::<'static, ArcRunWriterRow, ArcRunWriterScopedRow, ()>::tell::<&'static str, _>(
			"second",
		)
	})
	.bind(|()| ArcRunExplicit::pure(40));
	let boundary = ArcRunExplicit::censor::<&'static str, _>(censor_log, action).bind(|value| {
		ArcRunExplicit::<'static, ArcRunWriterRow, ArcRunWriterScopedRow, ()>::tell::<
			&'static str,
			_,
		>("outer")
		.bind(move |()| ArcRunExplicit::pure(value + 2))
	});

	let result = boundary.handle(
		handlers! {
			WriterBrand<&'static str>: move |op: Writer<'_, &'static str, Prog>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.lock().unwrap().push(log);
					next
				}
			},
		},
		scoped_handlers! {
			SendWriterCensorBrand<ArcBrand, &'static str>: writer_pre_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, 42);
	assert_eq!(*log.lock().unwrap(), vec!["censored:first", "censored:second", "outer"]);
}
