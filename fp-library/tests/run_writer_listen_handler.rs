#![cfg(feature = "effects")]
#![expect(clippy::unwrap_used, reason = "Tests use panicking operations for brevity and clarity.")]

//! End-to-end tests for the standard Writer listen handler.
//!
//! A Writer listen handler observes the logs emitted by a selected
//! action and returns them alongside the selected action value. Unlike
//! post-censor, listen must preserve the selected action's original
//! `Tell` operations so the surrounding Writer handler still observes
//! the same log sequence before any outer continuation logs.

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

type RunWriterRow = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
type RunWriterListenScopedRow =
	CoproductBrand<BoxWriterListenBrand<BoxBrand, String, i32>, CNilBrand>;
type RcRunWriterRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
type RcRunWriterListenScopedRow =
	CoproductBrand<WriterListenBrand<RcBrand, String, i32>, CNilBrand>;
type ArcRunWriterRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
type ArcRunWriterListenScopedRow =
	CoproductBrand<SendWriterListenBrand<ArcBrand, String, i32>, CNilBrand>;
type ListenResult = (i32, String);

#[test]
fn run_writer_listen_observes_and_preserves_selected_logs_before_outer_continuation() {
	type Prog = Run<RunWriterRow, RunWriterListenScopedRow, ListenResult>;

	let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let action =
		Run::<RunWriterRow, RunWriterListenScopedRow, ()>::tell::<String, _>("first".to_string())
			.bind(|()| {
				Run::<RunWriterRow, RunWriterListenScopedRow, ()>::tell::<String, _>(
					"second".to_string(),
				)
			})
			.bind(|()| Run::pure(40));
	let program: Prog = Run::listen::<String, _>(action).bind(|(value, observed)| {
		Run::<RunWriterRow, RunWriterListenScopedRow, ()>::tell::<String, _>("outer".to_string())
			.bind(move |()| Run::pure((value + 2, observed)))
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
			BoxWriterListenBrand<BoxBrand, String, i32>: writer_post_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, (42, "firstsecond".to_string()));
	assert_eq!(*log.borrow(), vec!["first".to_string(), "second".to_string(), "outer".to_string()]);
}

#[test]
fn rc_run_writer_listen_observes_and_preserves_selected_logs_before_outer_continuation() {
	type Prog = RcRun<RcRunWriterRow, RcRunWriterListenScopedRow, ListenResult>;

	let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let action = RcRun::<RcRunWriterRow, RcRunWriterListenScopedRow, ()>::tell::<String, _>(
		"first".to_string(),
	)
	.bind(|()| {
		RcRun::<RcRunWriterRow, RcRunWriterListenScopedRow, ()>::tell::<String, _>(
			"second".to_string(),
		)
	})
	.bind(|()| RcRun::pure(40));
	let program: Prog = RcRun::listen::<String, _>(action).bind(|(value, observed)| {
		RcRun::<RcRunWriterRow, RcRunWriterListenScopedRow, ()>::tell::<String, _>(
			"outer".to_string(),
		)
		.bind(move |()| RcRun::pure((value + 2, observed.clone())))
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
			WriterListenBrand<RcBrand, String, i32>: writer_post_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, (42, "firstsecond".to_string()));
	assert_eq!(*log.borrow(), vec!["first".to_string(), "second".to_string(), "outer".to_string()]);
}

#[test]
fn arc_run_writer_listen_observes_and_preserves_selected_logs_before_outer_continuation() {
	type Prog = ArcRun<ArcRunWriterRow, ArcRunWriterListenScopedRow, ListenResult>;

	let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
	let log_for_handler = Arc::clone(&log);
	let action = ArcRun::<ArcRunWriterRow, ArcRunWriterListenScopedRow, ()>::tell::<String, _>(
		"first".to_string(),
	)
	.bind(|()| {
		ArcRun::<ArcRunWriterRow, ArcRunWriterListenScopedRow, ()>::tell::<String, _>(
			"second".to_string(),
		)
	})
	.bind(|()| ArcRun::pure(40));
	let program: Prog = ArcRun::listen::<String, _>(action).bind(|(value, observed)| {
		ArcRun::<ArcRunWriterRow, ArcRunWriterListenScopedRow, ()>::tell::<String, _>(
			"outer".to_string(),
		)
		.bind(move |()| ArcRun::pure((value + 2, observed.clone())))
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
			SendWriterListenBrand<ArcBrand, String, i32>: writer_post_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, (42, "firstsecond".to_string()));
	assert_eq!(
		*log.lock().unwrap(),
		vec!["first".to_string(), "second".to_string(), "outer".to_string()]
	);
}

#[test]
fn run_explicit_writer_listen_observes_and_preserves_selected_logs_before_outer_continuation() {
	type Prog = RunExplicit<'static, RunWriterRow, RunWriterListenScopedRow, ListenResult>;

	let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let action = RunExplicit::<'static, RunWriterRow, RunWriterListenScopedRow, ()>::tell::<
		String,
		_,
	>("first".to_string())
	.bind(|()| {
		RunExplicit::<'static, RunWriterRow, RunWriterListenScopedRow, ()>::tell::<String, _>(
			"second".to_string(),
		)
	})
	.bind(|()| RunExplicit::pure(40));
	let boundary = RunExplicit::listen::<String, _>(action).bind(|(value, observed)| {
		RunExplicit::<'static, RunWriterRow, RunWriterListenScopedRow, ()>::tell::<String, _>(
			"outer".to_string(),
		)
		.bind(move |()| RunExplicit::pure((value + 2, observed.clone())))
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
			BoxWriterListenBrand<BoxBrand, String, i32>: writer_post_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, (42, "firstsecond".to_string()));
	assert_eq!(*log.borrow(), vec!["first".to_string(), "second".to_string(), "outer".to_string()]);
}

#[test]
fn rc_run_explicit_writer_listen_observes_and_preserves_selected_logs_before_outer_continuation() {
	type Prog = RcRunExplicit<'static, RcRunWriterRow, RcRunWriterListenScopedRow, ListenResult>;

	let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let action = RcRunExplicit::<'static, RcRunWriterRow, RcRunWriterListenScopedRow, ()>::tell::<
		String,
		_,
	>("first".to_string())
	.bind(|()| {
		RcRunExplicit::<'static, RcRunWriterRow, RcRunWriterListenScopedRow, ()>::tell::<String, _>(
			"second".to_string(),
		)
	})
	.bind(|()| RcRunExplicit::pure(40));
	let boundary = RcRunExplicit::listen::<String, _>(action).bind(|(value, observed)| {
		RcRunExplicit::<'static, RcRunWriterRow, RcRunWriterListenScopedRow, ()>::tell::<String, _>(
			"outer".to_string(),
		)
		.bind(move |()| RcRunExplicit::pure((value + 2, observed.clone())))
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
			WriterListenBrand<RcBrand, String, i32>: writer_post_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, (42, "firstsecond".to_string()));
	assert_eq!(*log.borrow(), vec!["first".to_string(), "second".to_string(), "outer".to_string()]);
}

#[test]
fn arc_run_explicit_writer_listen_observes_and_preserves_selected_logs_before_outer_continuation() {
	type Prog = ArcRunExplicit<'static, ArcRunWriterRow, ArcRunWriterListenScopedRow, ListenResult>;

	let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
	let log_for_handler = Arc::clone(&log);
	let action =
		ArcRunExplicit::<'static, ArcRunWriterRow, ArcRunWriterListenScopedRow, ()>::tell::<
			String,
			_,
		>("first".to_string())
		.bind(|()| {
			ArcRunExplicit::<'static, ArcRunWriterRow, ArcRunWriterListenScopedRow, ()>::tell::<
				String,
				_,
			>("second".to_string())
		})
		.bind(|()| ArcRunExplicit::pure(40));
	let boundary = ArcRunExplicit::listen::<String, _>(action).bind(|(value, observed)| {
		ArcRunExplicit::<'static, ArcRunWriterRow, ArcRunWriterListenScopedRow, ()>::tell::<
			String,
			_,
		>("outer".to_string())
		.bind(move |()| ArcRunExplicit::pure((value + 2, observed.clone())))
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
			SendWriterListenBrand<ArcBrand, String, i32>: writer_post_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, (42, "firstsecond".to_string()));
	assert_eq!(
		*log.lock().unwrap(),
		vec!["first".to_string(), "second".to_string(), "outer".to_string()]
	);
}
