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

// -- Named helpers --

type RunWriterStringRow = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
type RcRunWriterStringRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
type ArcRunWriterStringRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;

#[test]
fn run_writer_helpers_accumulate_logs() {
	let run_program: Run<RunWriterStringRow, CNilBrand, i32> =
		Run::<RunWriterStringRow, CNilBrand, ()>::tell::<String, _>("first".to_string())
			.bind(|()| {
				Run::<RunWriterStringRow, CNilBrand, ()>::tell::<String, _>("second".to_string())
			})
			.bind(|()| Run::<RunWriterStringRow, CNilBrand, i32>::pure(7));
	let run_handled: Run<CNilBrand, CNilBrand, (i32, String)> =
		run_program.run_writer::<String, _, CNilBrand>();
	assert_eq!(run_handled.extract(), (7, "firstsecond".to_string()));

	let fold_program: Run<RunWriterStringRow, CNilBrand, i32> =
		Run::<RunWriterStringRow, CNilBrand, ()>::tell::<String, _>("first".to_string())
			.bind(|()| {
				Run::<RunWriterStringRow, CNilBrand, ()>::tell::<String, _>("second".to_string())
			})
			.bind(|()| Run::<RunWriterStringRow, CNilBrand, i32>::pure(7));
	let fold_handled: Run<CNilBrand, CNilBrand, (i32, Vec<String>)> = fold_program
		.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
			logs.push(log);
			logs
		});
	assert_eq!(fold_handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
}

#[test]
fn rc_run_writer_helpers_accumulate_logs() {
	let run_program: RcRun<RcRunWriterStringRow, CNilBrand, i32> =
		RcRun::<RcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>("first".to_string())
			.bind(|()| {
				RcRun::<RcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
					"second".to_string(),
				)
			})
			.bind(|()| RcRun::<RcRunWriterStringRow, CNilBrand, i32>::pure(7));
	let run_handled: RcRun<CNilBrand, CNilBrand, (i32, String)> =
		run_program.run_writer::<String, _, CNilBrand>();
	assert_eq!(run_handled.extract(), (7, "firstsecond".to_string()));

	let fold_program: RcRun<RcRunWriterStringRow, CNilBrand, i32> =
		RcRun::<RcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>("first".to_string())
			.bind(|()| {
				RcRun::<RcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
					"second".to_string(),
				)
			})
			.bind(|()| RcRun::<RcRunWriterStringRow, CNilBrand, i32>::pure(7));
	let fold_handled: RcRun<CNilBrand, CNilBrand, (i32, Vec<String>)> = fold_program
		.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
			logs.push(log);
			logs
		});
	assert_eq!(fold_handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
}

#[test]
fn arc_run_writer_helpers_accumulate_logs() {
	let run_program: ArcRun<ArcRunWriterStringRow, CNilBrand, i32> =
		ArcRun::<ArcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>("first".to_string())
			.bind(|()| {
				ArcRun::<ArcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
					"second".to_string(),
				)
			})
			.bind(|()| ArcRun::<ArcRunWriterStringRow, CNilBrand, i32>::pure(7));
	let run_handled: ArcRun<CNilBrand, CNilBrand, (i32, String)> =
		run_program.run_writer::<String, _, CNilBrand>();
	assert_eq!(run_handled.extract(), (7, "firstsecond".to_string()));

	let fold_program: ArcRun<ArcRunWriterStringRow, CNilBrand, i32> =
		ArcRun::<ArcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>("first".to_string())
			.bind(|()| {
				ArcRun::<ArcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
					"second".to_string(),
				)
			})
			.bind(|()| ArcRun::<ArcRunWriterStringRow, CNilBrand, i32>::pure(7));
	let fold_handled: ArcRun<CNilBrand, CNilBrand, (i32, Vec<String>)> = fold_program
		.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
			logs.push(log);
			logs
		});
	assert_eq!(fold_handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
}

#[test]
fn run_explicit_writer_helpers_accumulate_logs() {
	let run_program: RunExplicit<'static, RunWriterStringRow, CNilBrand, i32> =
		RunExplicit::<'static, RunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
			"first".to_string(),
		)
		.bind(|()| {
			RunExplicit::<'static, RunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
				"second".to_string(),
			)
		})
		.bind(|()| RunExplicit::<'static, RunWriterStringRow, CNilBrand, i32>::pure(7));
	let run_handled: RunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		run_program.run_writer::<String, _, CNilBrand>();
	assert_eq!(run_handled.extract(), (7, "firstsecond".to_string()));

	let fold_program: RunExplicit<'static, RunWriterStringRow, CNilBrand, i32> =
		RunExplicit::<'static, RunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
			"first".to_string(),
		)
		.bind(|()| {
			RunExplicit::<'static, RunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
				"second".to_string(),
			)
		})
		.bind(|()| RunExplicit::<'static, RunWriterStringRow, CNilBrand, i32>::pure(7));
	let fold_handled: RunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<String>)> = fold_program
		.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
			logs.push(log);
			logs
		});
	assert_eq!(fold_handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
}

#[test]
fn rc_run_explicit_writer_helpers_accumulate_logs() {
	let run_program: RcRunExplicit<'static, RcRunWriterStringRow, CNilBrand, i32> =
		RcRunExplicit::<'static, RcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
			"first".to_string(),
		)
		.bind(|()| {
			RcRunExplicit::<'static, RcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
				"second".to_string(),
			)
		})
		.bind(|()| RcRunExplicit::<'static, RcRunWriterStringRow, CNilBrand, i32>::pure(7));
	let run_handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		run_program.run_writer::<String, _, CNilBrand>();
	assert_eq!(run_handled.extract(), (7, "firstsecond".to_string()));

	let fold_program: RcRunExplicit<'static, RcRunWriterStringRow, CNilBrand, i32> =
		RcRunExplicit::<'static, RcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
			"first".to_string(),
		)
		.bind(|()| {
			RcRunExplicit::<'static, RcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
				"second".to_string(),
			)
		})
		.bind(|()| RcRunExplicit::<'static, RcRunWriterStringRow, CNilBrand, i32>::pure(7));
	let fold_handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<String>)> =
		fold_program.fold_writer::<String, Vec<String>, _, CNilBrand>(
			Vec::new(),
			|mut logs, log| {
				logs.push(log);
				logs
			},
		);
	assert_eq!(fold_handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
}

#[test]
fn arc_run_explicit_writer_helpers_accumulate_logs() {
	let run_program: ArcRunExplicit<'static, ArcRunWriterStringRow, CNilBrand, i32> =
		ArcRunExplicit::<'static, ArcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
			"first".to_string(),
		)
		.bind(|()| {
			ArcRunExplicit::<'static, ArcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
				"second".to_string(),
			)
		})
		.bind(|()| ArcRunExplicit::<'static, ArcRunWriterStringRow, CNilBrand, i32>::pure(7));
	let run_handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		run_program.run_writer::<String, _, CNilBrand>();
	assert_eq!(run_handled.extract(), (7, "firstsecond".to_string()));

	let fold_program: ArcRunExplicit<'static, ArcRunWriterStringRow, CNilBrand, i32> =
		ArcRunExplicit::<'static, ArcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
			"first".to_string(),
		)
		.bind(|()| {
			ArcRunExplicit::<'static, ArcRunWriterStringRow, CNilBrand, ()>::tell::<String, _>(
				"second".to_string(),
			)
		})
		.bind(|()| ArcRunExplicit::<'static, ArcRunWriterStringRow, CNilBrand, i32>::pure(7));
	let fold_handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<String>)> =
		fold_program.fold_writer::<String, Vec<String>, _, CNilBrand>(
			Vec::new(),
			|mut logs, log| {
				logs.push(log);
				logs
			},
		);
	assert_eq!(fold_handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
}
