#![expect(clippy::unwrap_used, reason = "Tests use panicking operations for brevity and clarity.")]

// Integration tests for Phase 3 step 2: the recursive-target
// interpreter family (`interpret` / `run` / `run_accum`) on all six
// Run wrappers. Each wrapper is exercised with:
//   - a single-effect program that interprets to its result.
//   - a binded program (effect chain) that interprets through the
//     handler list and the bind continuation.
//   - the `run` alias producing the same result as `interpret`.
//   - `run_accum` with state threaded via closure captures, asserting
//     the final result and the post-loop state.
//
// The Erased-trio (Run, RcRun, ArcRun) uses Coyoneda-headed rows
// (CoyonedaBrand for Run/RcRun, ArcCoyonedaBrand for ArcRun); the
// Explicit-trio uses the same. Effect bodies are Identity for
// simplicity since Phase 3 step 4 hasn't shipped State / Reader /
// Except / Writer / Choose smart constructors yet.

use {
	fp_library::{
		brands::*,
		handlers,
		types::{
			Identity,
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

#[test]
fn run_interpret_single_effect() {
	let prog: Run<RunRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
	let result = prog.interpret(handlers! {
		IdentityBrand: |op: Identity<Run<RunRow, CNilBrand, i32>>| op.0,
	});
	assert_eq!(result, 42);
}

#[test]
fn run_interpret_bind_chain() {
	let prog: Run<RunRow, CNilBrand, i32> =
		Run::lift::<IdentityBrand, _>(Identity(10)).bind(|x| Run::pure(x + 5));
	let result = prog.interpret(handlers! {
		IdentityBrand: |op: Identity<Run<RunRow, CNilBrand, i32>>| op.0,
	});
	assert_eq!(result, 15);
}

#[test]
fn run_run_alias_matches_interpret() {
	let prog: Run<RunRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(7));
	let result = prog.run(handlers! {
		IdentityBrand: |op: Identity<Run<RunRow, CNilBrand, i32>>| op.0,
	});
	assert_eq!(result, 7);
}

#[test]
fn run_run_accum_threads_state_via_closure_capture() {
	let counter: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let counter_for_handler = Rc::clone(&counter);
	let prog: Run<RunRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(100));
	let result = prog.run_accum(
		handlers! {
			IdentityBrand: move |op: Identity<Run<RunRow, CNilBrand, i32>>| {
				*counter_for_handler.borrow_mut() += 1;
				op.0
			},
		},
		0_i32,
	);
	assert_eq!(result, 100);
	assert_eq!(*counter.borrow(), 1);
}

// -- RcRun --

#[test]
fn rc_run_interpret_single_effect() {
	let prog: RcRun<RcRunRow, CNilBrand, i32> = RcRun::lift::<IdentityBrand, _>(Identity(42));
	let result = prog.interpret(handlers! {
		IdentityBrand: |op: Identity<RcRun<RcRunRow, CNilBrand, i32>>| op.0,
	});
	assert_eq!(result, 42);
}

#[test]
fn rc_run_run_accum_threads_state() {
	let counter: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let counter_for_handler = Rc::clone(&counter);
	let prog: RcRun<RcRunRow, CNilBrand, i32> = RcRun::lift::<IdentityBrand, _>(Identity(7));
	let result = prog.run_accum(
		handlers! {
			IdentityBrand: move |op: Identity<RcRun<RcRunRow, CNilBrand, i32>>| {
				*counter_for_handler.borrow_mut() += 1;
				op.0
			},
		},
		0_i32,
	);
	assert_eq!(result, 7);
	assert_eq!(*counter.borrow(), 1);
}

// -- ArcRun --

type ArcRunRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;

#[test]
fn arc_run_interpret_single_effect() {
	let prog: ArcRun<ArcRunRow, CNilBrand, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(42));
	let result = prog.interpret(handlers! {
		IdentityBrand: |op: Identity<ArcRun<ArcRunRow, CNilBrand, i32>>| op.0,
	});
	assert_eq!(result, 42);
}

#[test]
fn arc_run_run_accum_threads_state_via_mutex() {
	let counter: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
	let counter_for_handler = Arc::clone(&counter);
	let prog: ArcRun<ArcRunRow, CNilBrand, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(7));
	let result = prog.run_accum(
		handlers! {
			IdentityBrand: move |op: Identity<ArcRun<ArcRunRow, CNilBrand, i32>>| {
				*counter_for_handler.lock().unwrap() += 1;
				op.0
			},
		},
		0_i32,
	);
	assert_eq!(result, 7);
	assert_eq!(*counter.lock().unwrap(), 1);
}

// -- RunExplicit --

#[test]
fn run_explicit_interpret_single_effect() {
	let prog: RunExplicit<'static, RunRow, CNilBrand, i32> =
		RunExplicit::lift::<IdentityBrand, _>(Identity(42));
	let result = prog.interpret(handlers! {
		IdentityBrand: |op: Identity<RunExplicit<'static, RunRow, CNilBrand, i32>>| op.0,
	});
	assert_eq!(result, 42);
}

#[test]
fn run_explicit_run_accum_threads_state() {
	let counter: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let counter_for_handler = Rc::clone(&counter);
	let prog: RunExplicit<'static, RunRow, CNilBrand, i32> =
		RunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let result = prog.run_accum(
		handlers! {
			IdentityBrand: move |op: Identity<RunExplicit<'static, RunRow, CNilBrand, i32>>| {
				*counter_for_handler.borrow_mut() += 1;
				op.0
			},
		},
		0_i32,
	);
	assert_eq!(result, 7);
	assert_eq!(*counter.borrow(), 1);
}

// -- RcRunExplicit --

#[test]
fn rc_run_explicit_interpret_single_effect() {
	let prog: RcRunExplicit<'static, RcRunRow, CNilBrand, i32> =
		RcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
	let result = prog.interpret(handlers! {
		IdentityBrand: |op: Identity<RcRunExplicit<'static, RcRunRow, CNilBrand, i32>>| op.0,
	});
	assert_eq!(result, 42);
}

// -- ArcRunExplicit --

#[test]
fn arc_run_explicit_interpret_single_effect() {
	let prog: ArcRunExplicit<'static, ArcRunRow, CNilBrand, i32> =
		ArcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
	let result = prog.interpret(handlers! {
		IdentityBrand: |op: Identity<ArcRunExplicit<'static, ArcRunRow, CNilBrand, i32>>| op.0,
	});
	assert_eq!(result, 42);
}
