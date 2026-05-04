// Integration tests for the Except effect smart constructors on
// all six Run wrappers.
//
// Each wrapper is exercised end-to-end with:
//   - throw_carries_error: a single Throw effect dispatched
//     through a handler that converts the error to a sentinel
//     result value, verifying the error value is delivered to the
//     handler.
//   - throw_in_bind_chain: a `pure(x).bind(|_| throw(e))` program
//     verifying that throw can appear after a successful bind step
//     and that the handler still receives the error.
//
// All six wrappers use the same `ExceptBrand<E>` in the row; there
// is no Arc-family parallel because `Except` has no `dyn Fn`
// continuation (its variant carries only the error value, not a
// closure-shaped continuation).

use fp_library::{
	brands::*,
	handlers,
	types::effects::{
		arc_run::ArcRun,
		arc_run_explicit::ArcRunExplicit,
		except::Except,
		rc_run::RcRun,
		rc_run_explicit::RcRunExplicit,
		run::Run,
		run_explicit::RunExplicit,
	},
};

// -- Run --

type RunExceptRow = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;

#[test]
fn run_throw_carries_error() {
	let prog: Run<RunExceptRow, CNilBrand, i32> = Run::throw::<&'static str, _>("oops");
	let result = prog.interpret(handlers! {
		ExceptBrand<&'static str>: |op: Except<'_, &'static str, Run<RunExceptRow, CNilBrand, i32>>| {
			match op {
				Except::Throw(e, _) => {
					assert_eq!(e, "oops");
					Run::pure(-1)
				}
			}
		},
	});
	assert_eq!(result, -1);
}

#[test]
fn run_throw_in_bind_chain() {
	let prog: Run<RunExceptRow, CNilBrand, i32> = Run::<RunExceptRow, CNilBrand, i32>::pure(7)
		.bind(|_v| Run::<RunExceptRow, CNilBrand, i32>::throw::<&'static str, _>("after-bind"));
	let result = prog.interpret(handlers! {
		ExceptBrand<&'static str>: |op: Except<'_, &'static str, Run<RunExceptRow, CNilBrand, i32>>| {
			match op {
				Except::Throw(e, _) => {
					assert_eq!(e, "after-bind");
					Run::pure(-1)
				}
			}
		},
	});
	assert_eq!(result, -1);
}

// -- RcRun --

type RcRunExceptRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;

#[test]
fn rc_run_throw_carries_error() {
	let prog: RcRun<RcRunExceptRow, CNilBrand, i32> = RcRun::throw::<&'static str, _>("oops");
	let result = prog.interpret(handlers! {
		ExceptBrand<&'static str>: |op: Except<'_, &'static str, RcRun<RcRunExceptRow, CNilBrand, i32>>| {
			match op {
				Except::Throw(e, _) => {
					assert_eq!(e, "oops");
					RcRun::pure(-1)
				}
			}
		},
	});
	assert_eq!(result, -1);
}

#[test]
fn rc_run_throw_in_bind_chain() {
	let prog: RcRun<RcRunExceptRow, CNilBrand, i32> =
		RcRun::<RcRunExceptRow, CNilBrand, i32>::pure(7).bind(|_v| {
			RcRun::<RcRunExceptRow, CNilBrand, i32>::throw::<&'static str, _>("after-bind")
		});
	let result = prog.interpret(handlers! {
		ExceptBrand<&'static str>: |op: Except<'_, &'static str, RcRun<RcRunExceptRow, CNilBrand, i32>>| {
			match op {
				Except::Throw(e, _) => {
					assert_eq!(e, "after-bind");
					RcRun::pure(-1)
				}
			}
		},
	});
	assert_eq!(result, -1);
}

// -- RunExplicit --

#[test]
fn run_explicit_throw_carries_error() {
	let prog: RunExplicit<'static, RunExceptRow, CNilBrand, i32> =
		RunExplicit::throw::<&'static str, _>("oops");
	let result = prog.interpret(handlers! {
		ExceptBrand<&'static str>: |op: Except<'_, &'static str, RunExplicit<'static, RunExceptRow, CNilBrand, i32>>| {
			match op {
				Except::Throw(e, _) => {
					assert_eq!(e, "oops");
					RunExplicit::pure(-1)
				}
			}
		},
	});
	assert_eq!(result, -1);
}

#[test]
fn run_explicit_throw_in_bind_chain() {
	let prog: RunExplicit<'static, RunExceptRow, CNilBrand, i32> =
		RunExplicit::<'static, RunExceptRow, CNilBrand, i32>::pure(7).bind(|_v| {
			RunExplicit::<'static, RunExceptRow, CNilBrand, i32>::throw::<&'static str, _>(
				"after-bind",
			)
		});
	let result = prog.interpret(handlers! {
		ExceptBrand<&'static str>: |op: Except<'_, &'static str, RunExplicit<'static, RunExceptRow, CNilBrand, i32>>| {
			match op {
				Except::Throw(e, _) => {
					assert_eq!(e, "after-bind");
					RunExplicit::pure(-1)
				}
			}
		},
	});
	assert_eq!(result, -1);
}

// -- RcRunExplicit --

#[test]
fn rc_run_explicit_throw_carries_error() {
	let prog: RcRunExplicit<'static, RcRunExceptRow, CNilBrand, i32> =
		RcRunExplicit::throw::<&'static str, _>("oops");
	let result = prog.interpret(handlers! {
		ExceptBrand<&'static str>: |op: Except<'_, &'static str, RcRunExplicit<'static, RcRunExceptRow, CNilBrand, i32>>| {
			match op {
				Except::Throw(e, _) => {
					assert_eq!(e, "oops");
					RcRunExplicit::pure(-1)
				}
			}
		},
	});
	assert_eq!(result, -1);
}

#[test]
fn rc_run_explicit_throw_in_bind_chain() {
	let prog: RcRunExplicit<'static, RcRunExceptRow, CNilBrand, i32> =
		RcRunExplicit::<'static, RcRunExceptRow, CNilBrand, i32>::pure(7).bind(|_v| {
			RcRunExplicit::<'static, RcRunExceptRow, CNilBrand, i32>::throw::<&'static str, _>(
				"after-bind",
			)
		});
	let result = prog.interpret(handlers! {
		ExceptBrand<&'static str>: |op: Except<'_, &'static str, RcRunExplicit<'static, RcRunExceptRow, CNilBrand, i32>>| {
			match op {
				Except::Throw(e, _) => {
					assert_eq!(e, "after-bind");
					RcRunExplicit::pure(-1)
				}
			}
		},
	});
	assert_eq!(result, -1);
}

// -- ArcRun --

type ArcRunExceptRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;

#[test]
fn arc_run_throw_carries_error() {
	let prog: ArcRun<ArcRunExceptRow, CNilBrand, i32> = ArcRun::throw::<&'static str, _>("oops");
	let result = prog.interpret(handlers! {
		ExceptBrand<&'static str>: |op: Except<'_, &'static str, ArcRun<ArcRunExceptRow, CNilBrand, i32>>| {
			match op {
				Except::Throw(e, _) => {
					assert_eq!(e, "oops");
					ArcRun::pure(-1)
				}
			}
		},
	});
	assert_eq!(result, -1);
}

#[test]
fn arc_run_throw_in_bind_chain() {
	let prog: ArcRun<ArcRunExceptRow, CNilBrand, i32> =
		ArcRun::<ArcRunExceptRow, CNilBrand, i32>::pure(7).bind(|_v| {
			ArcRun::<ArcRunExceptRow, CNilBrand, i32>::throw::<&'static str, _>("after-bind")
		});
	let result = prog.interpret(handlers! {
		ExceptBrand<&'static str>: |op: Except<'_, &'static str, ArcRun<ArcRunExceptRow, CNilBrand, i32>>| {
			match op {
				Except::Throw(e, _) => {
					assert_eq!(e, "after-bind");
					ArcRun::pure(-1)
				}
			}
		},
	});
	assert_eq!(result, -1);
}

// -- ArcRunExplicit --

#[test]
fn arc_run_explicit_throw_carries_error() {
	let prog: ArcRunExplicit<'static, ArcRunExceptRow, CNilBrand, i32> =
		ArcRunExplicit::throw::<&'static str, _>("oops");
	let result = prog.interpret(handlers! {
		ExceptBrand<&'static str>: |op: Except<'_, &'static str, ArcRunExplicit<'static, ArcRunExceptRow, CNilBrand, i32>>| {
			match op {
				Except::Throw(e, _) => {
					assert_eq!(e, "oops");
					ArcRunExplicit::pure(-1)
				}
			}
		},
	});
	assert_eq!(result, -1);
}

#[test]
fn arc_run_explicit_throw_in_bind_chain() {
	let prog: ArcRunExplicit<'static, ArcRunExceptRow, CNilBrand, i32> =
		ArcRunExplicit::<'static, ArcRunExceptRow, CNilBrand, i32>::pure(7).bind(|_v| {
			ArcRunExplicit::<'static, ArcRunExceptRow, CNilBrand, i32>::throw::<&'static str, _>(
				"after-bind",
			)
		});
	let result = prog.interpret(handlers! {
		ExceptBrand<&'static str>: |op: Except<'_, &'static str, ArcRunExplicit<'static, ArcRunExceptRow, CNilBrand, i32>>| {
			match op {
				Except::Throw(e, _) => {
					assert_eq!(e, "after-bind");
					ArcRunExplicit::pure(-1)
				}
			}
		},
	});
	assert_eq!(result, -1);
}
