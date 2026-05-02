#![expect(clippy::unwrap_used, reason = "Tests use panicking operations for brevity and clarity.")]

// Integration tests for the MonadRec-target interpreter family
// (`interpret_rec` / `run_rec` / `run_accum_rec`) on all six Run
// wrappers. Each wrapper is exercised against several `M` brand
// targets; the available choice depends on the wrapper's substrate:
//
//   - Erased non-Arc (Run, RcRun, RunExplicit, RcRunExplicit) +
//     ThunkBrand: stack-safety target. Thunk's `Box<dyn FnOnce>`
//     payload is not `Send + Sync`, so it cannot be combined with
//     ArcRun / ArcRunExplicit (those wrappers' substrate-level
//     `Send + Sync` bound on the M-wrapped continuation rules out
//     ThunkBrand).
//   - All six wrappers + OptionBrand: short-circuit semantics.
//   - All six wrappers + ResultBrand: error-channel semantics.
//
// State threading uses closure captures (Rc/RefCell for non-Arc;
// Arc/Mutex for the Arc family) per the Phase 3 step 4 design
// (Q3 = A, parallel to step 2's `run_accum`).
//
// Each test verifies the final value (and post-loop state where
// applicable). Stack-safety is verified by `prop_monad_rec_*` tests
// over `MonadRec` itself; the per-wrapper rec interpreter inherits
// that property by construction since the loop body emits one
// `ControlFlow::Continue` per peeled layer.

use {
	fp_library::{
		brands::*,
		handlers,
		types::{
			Identity,
			Thunk,
			effects::{
				arc_run::ArcRun,
				arc_run_explicit::ArcRunExplicit,
				rc_run::RcRun,
				rc_run_explicit::RcRunExplicit,
				run::Run,
				run_explicit::RunExplicit,
			},
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

type RunRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type RcRunRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type ArcRunRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;

#[test]
fn run_interpret_rec_thunk() {
	let prog: Run<RunRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
	let result: Thunk<'static, i32> = prog.interpret_rec::<ThunkBrand>(handlers! {
		IdentityBrand: |op: Identity<Thunk<'static, Run<RunRow, CNilBrand, i32>>>| op.0,
	});
	assert_eq!(result.evaluate(), 42);
}

#[test]
fn run_interpret_rec_option() {
	let prog: Run<RunRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(7));
	let result: Option<i32> = prog.interpret_rec::<OptionBrand>(handlers! {
		IdentityBrand: |op: Identity<Option<Run<RunRow, CNilBrand, i32>>>| op.0,
	});
	assert_eq!(result, Some(7));
}

#[test]
fn run_run_rec_alias_matches() {
	let prog: Run<RunRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(99));
	let result: Thunk<'static, i32> = prog.run_rec::<ThunkBrand>(handlers! {
		IdentityBrand: |op: Identity<Thunk<'static, Run<RunRow, CNilBrand, i32>>>| op.0,
	});
	assert_eq!(result.evaluate(), 99);
}

#[test]
fn run_run_accum_rec_threads_state() {
	let counter: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let counter_for_handler = Rc::clone(&counter);
	let prog: Run<RunRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(11));
	let result: Thunk<'static, i32> = prog.run_accum_rec::<ThunkBrand, _>(
		handlers! {
			IdentityBrand: move |op: Identity<Thunk<'static, Run<RunRow, CNilBrand, i32>>>| {
				*counter_for_handler.borrow_mut() += 1;
				op.0
			},
		},
		0_i32,
	);
	assert_eq!(result.evaluate(), 11);
	assert_eq!(*counter.borrow(), 1);
}

#[test]
fn run_interpret_rec_bind_chain() {
	let prog: Run<RunRow, CNilBrand, i32> =
		Run::lift::<IdentityBrand, _>(Identity(10)).bind(|x| Run::pure(x * 3));
	let result: Thunk<'static, i32> = prog.interpret_rec::<ThunkBrand>(handlers! {
		IdentityBrand: |op: Identity<Thunk<'static, Run<RunRow, CNilBrand, i32>>>| op.0,
	});
	assert_eq!(result.evaluate(), 30);
}

// -- RcRun --

#[test]
fn rc_run_interpret_rec_thunk() {
	let prog: RcRun<RcRunRow, CNilBrand, i32> = RcRun::lift::<IdentityBrand, _>(Identity(42));
	let result: Thunk<'static, i32> = prog.interpret_rec::<ThunkBrand>(handlers! {
		IdentityBrand: |op: Identity<Thunk<'static, RcRun<RcRunRow, CNilBrand, i32>>>| op.0,
	});
	assert_eq!(result.evaluate(), 42);
}

#[test]
fn rc_run_interpret_rec_option() {
	let prog: RcRun<RcRunRow, CNilBrand, i32> = RcRun::lift::<IdentityBrand, _>(Identity(7));
	let result: Option<i32> = prog.interpret_rec::<OptionBrand>(handlers! {
		IdentityBrand: |op: Identity<Option<RcRun<RcRunRow, CNilBrand, i32>>>| op.0,
	});
	assert_eq!(result, Some(7));
}

#[test]
fn rc_run_run_accum_rec_threads_state() {
	let counter: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let counter_for_handler = Rc::clone(&counter);
	let prog: RcRun<RcRunRow, CNilBrand, i32> = RcRun::lift::<IdentityBrand, _>(Identity(13));
	let result: Thunk<'static, i32> = prog.run_accum_rec::<ThunkBrand, _>(
		handlers! {
			IdentityBrand: move |op: Identity<Thunk<'static, RcRun<RcRunRow, CNilBrand, i32>>>| {
				*counter_for_handler.borrow_mut() += 1;
				op.0
			},
		},
		0_i32,
	);
	assert_eq!(result.evaluate(), 13);
	assert_eq!(*counter.borrow(), 1);
}

// -- ArcRun --

#[test]
fn arc_run_interpret_rec_option() {
	let prog: ArcRun<ArcRunRow, CNilBrand, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(42));
	let result: Option<i32> = prog.interpret_rec::<OptionBrand>(handlers! {
		IdentityBrand: |op: Identity<Option<ArcRun<ArcRunRow, CNilBrand, i32>>>| op.0,
	});
	assert_eq!(result, Some(42));
}

#[test]
fn arc_run_run_rec_alias_matches() {
	let prog: ArcRun<ArcRunRow, CNilBrand, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(99));
	let result: Option<i32> = prog.run_rec::<OptionBrand>(handlers! {
		IdentityBrand: |op: Identity<Option<ArcRun<ArcRunRow, CNilBrand, i32>>>| op.0,
	});
	assert_eq!(result, Some(99));
}

#[test]
fn arc_run_run_accum_rec_threads_state_via_mutex() {
	let counter: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
	let counter_for_handler = Arc::clone(&counter);
	let prog: ArcRun<ArcRunRow, CNilBrand, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(7));
	let result: Option<i32> = prog.run_accum_rec::<OptionBrand, _>(
		handlers! {
			IdentityBrand: move |op: Identity<Option<ArcRun<ArcRunRow, CNilBrand, i32>>>| {
				*counter_for_handler.lock().unwrap() += 1;
				op.0
			},
		},
		0_i32,
	);
	assert_eq!(result, Some(7));
	assert_eq!(*counter.lock().unwrap(), 1);
}

// -- RunExplicit --

#[test]
fn run_explicit_interpret_rec_thunk() {
	let prog: RunExplicit<'static, RunRow, CNilBrand, i32> =
		RunExplicit::lift::<IdentityBrand, _>(Identity(42));
	let result: Thunk<'static, i32> = prog.interpret_rec::<ThunkBrand>(handlers! {
		IdentityBrand: |op: Identity<Thunk<'static, RunExplicit<'static, RunRow, CNilBrand, i32>>>| op.0,
	});
	assert_eq!(result.evaluate(), 42);
}

#[test]
fn run_explicit_interpret_rec_option() {
	let prog: RunExplicit<'static, RunRow, CNilBrand, i32> =
		RunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let result: Option<i32> = prog.interpret_rec::<OptionBrand>(handlers! {
		IdentityBrand: |op: Identity<Option<RunExplicit<'static, RunRow, CNilBrand, i32>>>| op.0,
	});
	assert_eq!(result, Some(7));
}

#[test]
fn run_explicit_run_accum_rec_threads_state() {
	let counter: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let counter_for_handler = Rc::clone(&counter);
	let prog: RunExplicit<'static, RunRow, CNilBrand, i32> =
		RunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let result: Thunk<'static, i32> = prog.run_accum_rec::<ThunkBrand, _>(
		handlers! {
			IdentityBrand: move |op: Identity<Thunk<'static, RunExplicit<'static, RunRow, CNilBrand, i32>>>| {
				*counter_for_handler.borrow_mut() += 1;
				op.0
			},
		},
		0_i32,
	);
	assert_eq!(result.evaluate(), 7);
	assert_eq!(*counter.borrow(), 1);
}

// -- RcRunExplicit --

#[test]
fn rc_run_explicit_interpret_rec_thunk() {
	let prog: RcRunExplicit<'static, RcRunRow, CNilBrand, i32> =
		RcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
	let result: Thunk<'static, i32> = prog.interpret_rec::<ThunkBrand>(handlers! {
		IdentityBrand: |op: Identity<Thunk<'static, RcRunExplicit<'static, RcRunRow, CNilBrand, i32>>>| op.0,
	});
	assert_eq!(result.evaluate(), 42);
}

#[test]
fn rc_run_explicit_interpret_rec_option() {
	let prog: RcRunExplicit<'static, RcRunRow, CNilBrand, i32> =
		RcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let result: Option<i32> = prog.interpret_rec::<OptionBrand>(handlers! {
		IdentityBrand: |op: Identity<Option<RcRunExplicit<'static, RcRunRow, CNilBrand, i32>>>| op.0,
	});
	assert_eq!(result, Some(7));
}

// -- ArcRunExplicit --

#[test]
fn arc_run_explicit_interpret_rec_option() {
	let prog: ArcRunExplicit<'static, ArcRunRow, CNilBrand, i32> =
		ArcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
	let result: Option<i32> = prog.interpret_rec::<OptionBrand>(handlers! {
		IdentityBrand: |op: Identity<Option<ArcRunExplicit<'static, ArcRunRow, CNilBrand, i32>>>| op.0,
	});
	assert_eq!(result, Some(42));
}

#[test]
fn arc_run_explicit_run_accum_rec_threads_state() {
	let counter: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
	let counter_for_handler = Arc::clone(&counter);
	let prog: ArcRunExplicit<'static, ArcRunRow, CNilBrand, i32> =
		ArcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let result: Option<i32> = prog.run_accum_rec::<OptionBrand, _>(
		handlers! {
			IdentityBrand: move |op: Identity<Option<ArcRunExplicit<'static, ArcRunRow, CNilBrand, i32>>>| {
				*counter_for_handler.lock().unwrap() += 1;
				op.0
			},
		},
		0_i32,
	);
	assert_eq!(result, Some(7));
	assert_eq!(*counter.lock().unwrap(), 1);
}
