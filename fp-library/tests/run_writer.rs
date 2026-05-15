#![expect(clippy::unwrap_used, reason = "Tests use panicking operations for brevity and clarity.")]

// Integration tests for the Writer effect smart constructors on
// all six Run wrappers.
//
// Each wrapper is exercised end-to-end with:
//   - tell_emits_log: a single Tell effect dispatched through a
//     handler that captures the log value into a shared cell.
//   - tell_bind_chain: a `tell(a) >>= |_| tell(b)` program
//     verifying that both logs are captured in order.
//
// All six wrappers use the same `WriterBrand<W>` in the row; there
// is no Arc-family parallel because `Writer` has no `dyn Fn`
// continuation (its variant carries the log value and the next
// program's value directly, not a closure).

use {
	fp_library::{
		brands::*,
		handlers,
		types::effects::{
			arc_run::ArcRun,
			arc_run_explicit::ArcRunExplicit,
			rc_run::RcRun,
			rc_run_explicit::RcRunExplicit,
			run::Run,
			run_explicit::RunExplicit,
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

// -- Run --

type RunWriterRow = CoproductBrand<CoyonedaBrand<WriterBrand<&'static str>>, CNilBrand>;

#[test]
fn run_tell_emits_log() {
	let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let prog: Run<RunWriterRow, CNilBrand, ()> = Run::tell::<&'static str, _>("hello");
	prog.handle(handlers! {
		WriterBrand<&'static str>: move |op: Writer<'_, &'static str, Run<RunWriterRow, CNilBrand, ()>>| {
			match op {
				Writer::Tell(w, next, _) => {
					log_for_handler.borrow_mut().push(w);
					next
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*log.borrow(), vec!["hello"]);
}

#[test]
fn run_tell_bind_chain() {
	let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let prog: Run<RunWriterRow, CNilBrand, ()> =
		Run::<RunWriterRow, CNilBrand, ()>::tell::<&'static str, _>("first")
			.bind(|()| Run::<RunWriterRow, CNilBrand, ()>::tell::<&'static str, _>("second"));
	prog.handle(handlers! {
		WriterBrand<&'static str>: move |op: Writer<'_, &'static str, Run<RunWriterRow, CNilBrand, ()>>| {
			match op {
				Writer::Tell(w, next, _) => {
					log_for_handler.borrow_mut().push(w);
					next
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*log.borrow(), vec!["first", "second"]);
}

// -- RcRun --

type RcRunWriterRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<&'static str>>, CNilBrand>;

#[test]
fn rc_run_tell_emits_log() {
	let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let prog: RcRun<RcRunWriterRow, CNilBrand, ()> = RcRun::tell::<&'static str, _>("hello");
	prog.handle(handlers! {
		WriterBrand<&'static str>: move |op: Writer<'_, &'static str, RcRun<RcRunWriterRow, CNilBrand, ()>>| {
			match op {
				Writer::Tell(w, next, _) => {
					log_for_handler.borrow_mut().push(w);
					next
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*log.borrow(), vec!["hello"]);
}

#[test]
fn rc_run_tell_bind_chain() {
	let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let prog: RcRun<RcRunWriterRow, CNilBrand, ()> =
		RcRun::<RcRunWriterRow, CNilBrand, ()>::tell::<&'static str, _>("first")
			.bind(|()| RcRun::<RcRunWriterRow, CNilBrand, ()>::tell::<&'static str, _>("second"));
	prog.handle(handlers! {
		WriterBrand<&'static str>: move |op: Writer<'_, &'static str, RcRun<RcRunWriterRow, CNilBrand, ()>>| {
			match op {
				Writer::Tell(w, next, _) => {
					log_for_handler.borrow_mut().push(w);
					next
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*log.borrow(), vec!["first", "second"]);
}

// -- RunExplicit --

#[test]
fn run_explicit_tell_emits_log() {
	let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let prog: RunExplicit<'static, RunWriterRow, CNilBrand, ()> =
		RunExplicit::tell::<&'static str, _>("hello");
	prog.handle(handlers! {
		WriterBrand<&'static str>: move |op: Writer<'_, &'static str, RunExplicit<'static, RunWriterRow, CNilBrand, ()>>| {
			match op {
				Writer::Tell(w, next, _) => {
					log_for_handler.borrow_mut().push(w);
					next
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*log.borrow(), vec!["hello"]);
}

#[test]
fn run_explicit_tell_bind_chain() {
	let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let prog: RunExplicit<'static, RunWriterRow, CNilBrand, ()> =
		RunExplicit::<'static, RunWriterRow, CNilBrand, ()>::tell::<&'static str, _>("first").bind(
			|()| {
				RunExplicit::<'static, RunWriterRow, CNilBrand, ()>::tell::<&'static str, _>(
					"second",
				)
			},
		);
	prog.handle(handlers! {
		WriterBrand<&'static str>: move |op: Writer<'_, &'static str, RunExplicit<'static, RunWriterRow, CNilBrand, ()>>| {
			match op {
				Writer::Tell(w, next, _) => {
					log_for_handler.borrow_mut().push(w);
					next
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*log.borrow(), vec!["first", "second"]);
}

// -- RcRunExplicit --

#[test]
fn rc_run_explicit_tell_emits_log() {
	let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let prog: RcRunExplicit<'static, RcRunWriterRow, CNilBrand, ()> =
		RcRunExplicit::tell::<&'static str, _>("hello");
	prog.handle(handlers! {
		WriterBrand<&'static str>: move |op: Writer<'_, &'static str, RcRunExplicit<'static, RcRunWriterRow, CNilBrand, ()>>| {
			match op {
				Writer::Tell(w, next, _) => {
					log_for_handler.borrow_mut().push(w);
					next
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*log.borrow(), vec!["hello"]);
}

#[test]
fn rc_run_explicit_tell_bind_chain() {
	let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let prog: RcRunExplicit<'static, RcRunWriterRow, CNilBrand, ()> =
		RcRunExplicit::<'static, RcRunWriterRow, CNilBrand, ()>::tell::<&'static str, _>("first")
			.bind(|()| {
				RcRunExplicit::<'static, RcRunWriterRow, CNilBrand, ()>::tell::<&'static str, _>(
					"second",
				)
			});
	prog.handle(handlers! {
		WriterBrand<&'static str>: move |op: Writer<'_, &'static str, RcRunExplicit<'static, RcRunWriterRow, CNilBrand, ()>>| {
			match op {
				Writer::Tell(w, next, _) => {
					log_for_handler.borrow_mut().push(w);
					next
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*log.borrow(), vec!["first", "second"]);
}

// -- ArcRun --

type ArcRunWriterRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<&'static str>>, CNilBrand>;

#[test]
fn arc_run_tell_emits_log() {
	let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
	let log_for_handler = Arc::clone(&log);
	let prog: ArcRun<ArcRunWriterRow, CNilBrand, ()> = ArcRun::tell::<&'static str, _>("hello");
	prog.handle(handlers! {
		WriterBrand<&'static str>: move |op: Writer<'_, &'static str, ArcRun<ArcRunWriterRow, CNilBrand, ()>>| {
			match op {
				Writer::Tell(w, next, _) => {
					log_for_handler.lock().unwrap().push(w);
					next
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*log.lock().unwrap(), vec!["hello"]);
}

#[test]
fn arc_run_tell_bind_chain() {
	let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
	let log_for_handler = Arc::clone(&log);
	let prog: ArcRun<ArcRunWriterRow, CNilBrand, ()> =
		ArcRun::<ArcRunWriterRow, CNilBrand, ()>::tell::<&'static str, _>("first")
			.bind(|()| ArcRun::<ArcRunWriterRow, CNilBrand, ()>::tell::<&'static str, _>("second"));
	prog.handle(handlers! {
		WriterBrand<&'static str>: move |op: Writer<'_, &'static str, ArcRun<ArcRunWriterRow, CNilBrand, ()>>| {
			match op {
				Writer::Tell(w, next, _) => {
					log_for_handler.lock().unwrap().push(w);
					next
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*log.lock().unwrap(), vec!["first", "second"]);
}

// -- ArcRunExplicit --

#[test]
fn arc_run_explicit_tell_emits_log() {
	let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
	let log_for_handler = Arc::clone(&log);
	let prog: ArcRunExplicit<'static, ArcRunWriterRow, CNilBrand, ()> =
		ArcRunExplicit::tell::<&'static str, _>("hello");
	prog.handle(handlers! {
		WriterBrand<&'static str>: move |op: Writer<'_, &'static str, ArcRunExplicit<'static, ArcRunWriterRow, CNilBrand, ()>>| {
			match op {
				Writer::Tell(w, next, _) => {
					log_for_handler.lock().unwrap().push(w);
					next
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*log.lock().unwrap(), vec!["hello"]);
}

#[test]
fn arc_run_explicit_tell_bind_chain() {
	let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
	let log_for_handler = Arc::clone(&log);
	let prog: ArcRunExplicit<'static, ArcRunWriterRow, CNilBrand, ()> =
		ArcRunExplicit::<'static, ArcRunWriterRow, CNilBrand, ()>::tell::<&'static str, _>("first")
			.bind(|()| {
				ArcRunExplicit::<'static, ArcRunWriterRow, CNilBrand, ()>::tell::<&'static str, _>(
					"second",
				)
			});
	prog.handle(handlers! {
		WriterBrand<&'static str>: move |op: Writer<'_, &'static str, ArcRunExplicit<'static, ArcRunWriterRow, CNilBrand, ()>>| {
			match op {
				Writer::Tell(w, next, _) => {
					log_for_handler.lock().unwrap().push(w);
					next
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*log.lock().unwrap(), vec!["first", "second"]);
}
