#![cfg(feature = "effects")]

// Integration tests for the Reader effect smart constructors on
// all six Run wrappers.
//
// Each wrapper is exercised end-to-end with:
//   - ask_returns_environment: a single Ask effect dispatched
//     through a handler that injects a captured environment value.
//   - ask_bind_chain: a bind-chained program (`ask >>= |e1| ask
//     >>= |e2| pure(e1 + e2)`) verifying the same environment is
//     delivered on each successive Ask within one program.
//
// The two default single-shot wrappers (Run, RunExplicit) thread
// `BoxBrand` and use `BoxReaderBrand<BoxBrand, E>` whose closure
// projection is `Box<dyn FnOnce>`. The two non-Arc multi-shot
// wrappers (RcRun, RcRunExplicit) thread `RcBrand` and use
// `ReaderBrand<RcBrand, E>` whose closure projection is
// `Rc<dyn Fn>`. The two Arc wrappers (ArcRun, ArcRunExplicit)
// thread `ArcBrand` and use `SendReaderBrand<ArcBrand, E>` whose
// closure projection bakes in `Send + Sync`.

use fp_library::{
	brands::*,
	handlers,
	types::effects::{
		arc_run::ArcRun,
		arc_run_explicit::ArcRunExplicit,
		rc_run::RcRun,
		rc_run_explicit::RcRunExplicit,
		reader::{
			BoxReader,
			Reader,
			SendReader,
		},
		run::Run,
		run_explicit::RunExplicit,
	},
};

// -- Run --

type RunReaderRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;

#[test]
fn run_ask_returns_environment() {
	let env: i32 = 42;
	let prog: Run<RunReaderRow, CNilBrand, i32> = Run::ask();
	let result = prog.handle(handlers! {
		BoxReaderBrand<BoxBrand, i32>: move |op: BoxReader<'_, BoxBrand, i32, Run<RunReaderRow, CNilBrand, i32>>| {
			match op {
				BoxReader::Ask(k) => k(env),
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 42);
}

#[test]
fn run_ask_bind_chain() {
	let env: i32 = 10;
	let prog: Run<RunReaderRow, CNilBrand, i32> =
		Run::<RunReaderRow, CNilBrand, i32>::ask().bind(|e1: i32| {
			Run::<RunReaderRow, CNilBrand, i32>::ask().bind(move |e2: i32| Run::pure(e1 + e2))
		});
	let result = prog.handle(handlers! {
		BoxReaderBrand<BoxBrand, i32>: move |op: BoxReader<'_, BoxBrand, i32, Run<RunReaderRow, CNilBrand, i32>>| {
			match op {
				BoxReader::Ask(k) => k(env),
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 20);
}

// -- RcRun --

type RcRunReaderRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;

#[test]
fn rc_run_ask_returns_environment() {
	let env: i32 = 42;
	let prog: RcRun<RcRunReaderRow, CNilBrand, i32> = RcRun::ask();
	let result = prog.handle(handlers! {
		ReaderBrand<RcBrand, i32>: move |op: Reader<'_, RcBrand, i32, RcRun<RcRunReaderRow, CNilBrand, i32>>| {
			match op {
				Reader::Ask(k) => (*k)(env),
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 42);
}

#[test]
fn rc_run_ask_bind_chain() {
	let env: i32 = 10;
	let prog: RcRun<RcRunReaderRow, CNilBrand, i32> =
		RcRun::<RcRunReaderRow, CNilBrand, i32>::ask().bind(|e1: i32| {
			RcRun::<RcRunReaderRow, CNilBrand, i32>::ask().bind(move |e2: i32| RcRun::pure(e1 + e2))
		});
	let result = prog.handle(handlers! {
		ReaderBrand<RcBrand, i32>: move |op: Reader<'_, RcBrand, i32, RcRun<RcRunReaderRow, CNilBrand, i32>>| {
			match op {
				Reader::Ask(k) => (*k)(env),
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 20);
}

// -- RunExplicit --

#[test]
fn run_explicit_ask_returns_environment() {
	let env: i32 = 42;
	let prog: RunExplicit<'static, RunReaderRow, CNilBrand, i32> = RunExplicit::ask();
	let result = prog.handle(handlers! {
		BoxReaderBrand<BoxBrand, i32>: move |op: BoxReader<'_, BoxBrand, i32, RunExplicit<'static, RunReaderRow, CNilBrand, i32>>| {
			match op {
				BoxReader::Ask(k) => k(env),
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 42);
}

#[test]
fn run_explicit_ask_bind_chain() {
	let env: i32 = 10;
	let prog: RunExplicit<'static, RunReaderRow, CNilBrand, i32> =
		RunExplicit::<'static, RunReaderRow, CNilBrand, i32>::ask().bind(|e1: i32| {
			RunExplicit::<'static, RunReaderRow, CNilBrand, i32>::ask()
				.bind(move |e2: i32| RunExplicit::pure(e1 + e2))
		});
	let result = prog.handle(handlers! {
		BoxReaderBrand<BoxBrand, i32>: move |op: BoxReader<'_, BoxBrand, i32, RunExplicit<'static, RunReaderRow, CNilBrand, i32>>| {
			match op {
				BoxReader::Ask(k) => k(env),
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 20);
}

// -- RcRunExplicit --

#[test]
fn rc_run_explicit_ask_returns_environment() {
	let env: i32 = 42;
	let prog: RcRunExplicit<'static, RcRunReaderRow, CNilBrand, i32> = RcRunExplicit::ask();
	let result = prog.handle(handlers! {
		ReaderBrand<RcBrand, i32>: move |op: Reader<'_, RcBrand, i32, RcRunExplicit<'static, RcRunReaderRow, CNilBrand, i32>>| {
			match op {
				Reader::Ask(k) => (*k)(env),
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 42);
}

#[test]
fn rc_run_explicit_ask_bind_chain() {
	let env: i32 = 10;
	let prog: RcRunExplicit<'static, RcRunReaderRow, CNilBrand, i32> =
		RcRunExplicit::<'static, RcRunReaderRow, CNilBrand, i32>::ask().bind(|e1: i32| {
			RcRunExplicit::<'static, RcRunReaderRow, CNilBrand, i32>::ask()
				.bind(move |e2: i32| RcRunExplicit::pure(e1 + e2))
		});
	let result = prog.handle(handlers! {
		ReaderBrand<RcBrand, i32>: move |op: Reader<'_, RcBrand, i32, RcRunExplicit<'static, RcRunReaderRow, CNilBrand, i32>>| {
			match op {
				Reader::Ask(k) => (*k)(env),
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 20);
}

// -- ArcRun --

type ArcRunReaderRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;

#[test]
fn arc_run_ask_returns_environment() {
	let env: i32 = 42;
	let prog: ArcRun<ArcRunReaderRow, CNilBrand, i32> = ArcRun::ask();
	let result = prog.handle(handlers! {
		SendReaderBrand<ArcBrand, i32>: move |op: SendReader<'_, ArcBrand, i32, ArcRun<ArcRunReaderRow, CNilBrand, i32>>| {
			match op {
				SendReader::Ask(k) => (*k)(env),
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 42);
}

#[test]
fn arc_run_ask_bind_chain() {
	let env: i32 = 10;
	let prog: ArcRun<ArcRunReaderRow, CNilBrand, i32> =
		ArcRun::<ArcRunReaderRow, CNilBrand, i32>::ask().bind(|e1: i32| {
			ArcRun::<ArcRunReaderRow, CNilBrand, i32>::ask()
				.bind(move |e2: i32| ArcRun::pure(e1 + e2))
		});
	let result = prog.handle(handlers! {
		SendReaderBrand<ArcBrand, i32>: move |op: SendReader<'_, ArcBrand, i32, ArcRun<ArcRunReaderRow, CNilBrand, i32>>| {
			match op {
				SendReader::Ask(k) => (*k)(env),
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 20);
}

// -- ArcRunExplicit --

#[test]
fn arc_run_explicit_ask_returns_environment() {
	let env: i32 = 42;
	let prog: ArcRunExplicit<'static, ArcRunReaderRow, CNilBrand, i32> = ArcRunExplicit::ask();
	let result = prog.handle(handlers! {
		SendReaderBrand<ArcBrand, i32>: move |op: SendReader<'_, ArcBrand, i32, ArcRunExplicit<'static, ArcRunReaderRow, CNilBrand, i32>>| {
			match op {
				SendReader::Ask(k) => (*k)(env),
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 42);
}

#[test]
fn arc_run_explicit_ask_bind_chain() {
	let env: i32 = 10;
	let prog: ArcRunExplicit<'static, ArcRunReaderRow, CNilBrand, i32> =
		ArcRunExplicit::<'static, ArcRunReaderRow, CNilBrand, i32>::ask().bind(|e1: i32| {
			ArcRunExplicit::<'static, ArcRunReaderRow, CNilBrand, i32>::ask()
				.bind(move |e2: i32| ArcRunExplicit::pure(e1 + e2))
		});
	let result = prog.handle(handlers! {
		SendReaderBrand<ArcBrand, i32>: move |op: SendReader<'_, ArcBrand, i32, ArcRunExplicit<'static, ArcRunReaderRow, CNilBrand, i32>>| {
			match op {
				SendReader::Ask(k) => (*k)(env),
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 20);
}
