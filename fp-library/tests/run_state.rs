#![expect(clippy::unwrap_used, reason = "Tests use panicking operations for brevity and clarity.")]

// Integration tests for the State effect smart constructors on
// all six Run wrappers.
//
// Each wrapper is exercised end-to-end with:
//   - get_returns_current_state: a single Get effect dispatched
//     through a handler that reads from a captured cell.
//   - put_writes_state: a single Put effect dispatched through a
//     handler that writes to a captured cell.
//   - get_put_get_bind_chain: a bind-chained program
//     (`get >>= |s| put(s + 1) >>= |_| get`) verifying state
//     threads through the bind continuation.
//
// State threading is via user-side closure captures
// (`Rc<RefCell<S>>` for non-Arc, `Arc<Mutex<S>>` for Arc).
//
// The two default single-shot wrappers (Run, RunExplicit) thread
// `BoxBrand` and use `BoxStateBrand<BoxBrand, S>` whose closure
// projection is `Box<dyn FnOnce>`. The two non-Arc multi-shot
// wrappers (RcRun, RcRunExplicit) thread `RcBrand` and use
// `StateBrand<RcBrand, S>` whose closure projection is
// `Rc<dyn Fn>`. The two Arc wrappers (ArcRun, ArcRunExplicit)
// thread `ArcBrand` and use `SendStateBrand<ArcBrand, S>` whose
// closure projection bakes in `Send + Sync`.

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
			state::{
				BoxState,
				SendState,
				State,
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

type RunStateRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;

#[test]
fn run_get_returns_current_state() {
	let cell: Rc<RefCell<i32>> = Rc::new(RefCell::new(42));
	let cell_for_handler = Rc::clone(&cell);
	let prog: Run<RunStateRow, CNilBrand, i32> = Run::get();
	let result = prog.interpret(handlers! {
		BoxStateBrand<BoxBrand, i32>: move |op: BoxState<'_, BoxBrand, i32, Run<RunStateRow, CNilBrand, i32>>| {
			match op {
				BoxState::Get(k) => {
					let s = *cell_for_handler.borrow();
					k(s)
				}
				BoxState::Put(s, k) => {
					*cell_for_handler.borrow_mut() = s;
					k(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 42);
	assert_eq!(*cell.borrow(), 42);
}

#[test]
fn run_put_writes_state() {
	let cell: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let cell_for_handler = Rc::clone(&cell);
	let prog: Run<RunStateRow, CNilBrand, ()> = Run::put::<i32, _>(99);
	prog.interpret(handlers! {
		BoxStateBrand<BoxBrand, i32>: move |op: BoxState<'_, BoxBrand, i32, Run<RunStateRow, CNilBrand, ()>>| {
			match op {
				BoxState::Get(k) => {
					let s = *cell_for_handler.borrow();
					k(s)
				}
				BoxState::Put(s, k) => {
					*cell_for_handler.borrow_mut() = s;
					k(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*cell.borrow(), 99);
}

#[test]
fn run_get_put_get_bind_chain() {
	let cell: Rc<RefCell<i32>> = Rc::new(RefCell::new(10));
	let cell_for_handler = Rc::clone(&cell);
	let prog: Run<RunStateRow, CNilBrand, i32> = Run::<RunStateRow, CNilBrand, i32>::get()
		.bind(|s: i32| Run::<RunStateRow, CNilBrand, ()>::put::<i32, _>(s + 1))
		.bind(|()| Run::<RunStateRow, CNilBrand, i32>::get());
	let result = prog.interpret(handlers! {
		BoxStateBrand<BoxBrand, i32>: move |op: BoxState<'_, BoxBrand, i32, Run<RunStateRow, CNilBrand, i32>>| {
			match op {
				BoxState::Get(k) => {
					let s = *cell_for_handler.borrow();
					k(s)
				}
				BoxState::Put(s, k) => {
					*cell_for_handler.borrow_mut() = s;
					k(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 11);
	assert_eq!(*cell.borrow(), 11);
}

// -- RcRun --

type RcRunStateRow = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;

#[test]
fn rc_run_get_returns_current_state() {
	let cell: Rc<RefCell<i32>> = Rc::new(RefCell::new(42));
	let cell_for_handler = Rc::clone(&cell);
	let prog: RcRun<RcRunStateRow, CNilBrand, i32> = RcRun::get();
	let result = prog.interpret(handlers! {
		StateBrand<RcBrand, i32>: move |op: State<'_, RcBrand, i32, RcRun<RcRunStateRow, CNilBrand, i32>>| {
			match op {
				State::Get(k) => {
					let s = *cell_for_handler.borrow();
					(*k)(s)
				}
				State::Put(s, k) => {
					*cell_for_handler.borrow_mut() = s;
					(*k)(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 42);
	assert_eq!(*cell.borrow(), 42);
}

#[test]
fn rc_run_put_writes_state() {
	let cell: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let cell_for_handler = Rc::clone(&cell);
	let prog: RcRun<RcRunStateRow, CNilBrand, ()> = RcRun::put::<i32, _>(99);
	prog.interpret(handlers! {
		StateBrand<RcBrand, i32>: move |op: State<'_, RcBrand, i32, RcRun<RcRunStateRow, CNilBrand, ()>>| {
			match op {
				State::Get(k) => {
					let s = *cell_for_handler.borrow();
					(*k)(s)
				}
				State::Put(s, k) => {
					*cell_for_handler.borrow_mut() = s;
					(*k)(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*cell.borrow(), 99);
}

#[test]
fn rc_run_get_put_get_bind_chain() {
	let cell: Rc<RefCell<i32>> = Rc::new(RefCell::new(10));
	let cell_for_handler = Rc::clone(&cell);
	let prog: RcRun<RcRunStateRow, CNilBrand, i32> = RcRun::<RcRunStateRow, CNilBrand, i32>::get()
		.bind(|s: i32| RcRun::<RcRunStateRow, CNilBrand, ()>::put::<i32, _>(s + 1))
		.bind(|()| RcRun::<RcRunStateRow, CNilBrand, i32>::get());
	let result = prog.interpret(handlers! {
		StateBrand<RcBrand, i32>: move |op: State<'_, RcBrand, i32, RcRun<RcRunStateRow, CNilBrand, i32>>| {
			match op {
				State::Get(k) => {
					let s = *cell_for_handler.borrow();
					(*k)(s)
				}
				State::Put(s, k) => {
					*cell_for_handler.borrow_mut() = s;
					(*k)(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 11);
	assert_eq!(*cell.borrow(), 11);
}

// -- RunExplicit --

#[test]
fn run_explicit_get_returns_current_state() {
	let cell: Rc<RefCell<i32>> = Rc::new(RefCell::new(42));
	let cell_for_handler = Rc::clone(&cell);
	let prog: RunExplicit<'static, RunStateRow, CNilBrand, i32> = RunExplicit::get();
	let result = prog.interpret(handlers! {
		BoxStateBrand<BoxBrand, i32>: move |op: BoxState<'_, BoxBrand, i32, RunExplicit<'static, RunStateRow, CNilBrand, i32>>| {
			match op {
				BoxState::Get(k) => {
					let s = *cell_for_handler.borrow();
					k(s)
				}
				BoxState::Put(s, k) => {
					*cell_for_handler.borrow_mut() = s;
					k(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 42);
	assert_eq!(*cell.borrow(), 42);
}

#[test]
fn run_explicit_put_writes_state() {
	let cell: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let cell_for_handler = Rc::clone(&cell);
	let prog: RunExplicit<'static, RunStateRow, CNilBrand, ()> = RunExplicit::put::<i32, _>(99);
	prog.interpret(handlers! {
		BoxStateBrand<BoxBrand, i32>: move |op: BoxState<'_, BoxBrand, i32, RunExplicit<'static, RunStateRow, CNilBrand, ()>>| {
			match op {
				BoxState::Get(k) => {
					let s = *cell_for_handler.borrow();
					k(s)
				}
				BoxState::Put(s, k) => {
					*cell_for_handler.borrow_mut() = s;
					k(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*cell.borrow(), 99);
}

#[test]
fn run_explicit_get_put_get_bind_chain() {
	let cell: Rc<RefCell<i32>> = Rc::new(RefCell::new(10));
	let cell_for_handler = Rc::clone(&cell);
	let prog: RunExplicit<'static, RunStateRow, CNilBrand, i32> =
		RunExplicit::<'static, RunStateRow, CNilBrand, i32>::get()
			.bind(|s: i32| RunExplicit::<'static, RunStateRow, CNilBrand, ()>::put::<i32, _>(s + 1))
			.bind(|()| RunExplicit::<'static, RunStateRow, CNilBrand, i32>::get());
	let result = prog.interpret(handlers! {
		BoxStateBrand<BoxBrand, i32>: move |op: BoxState<'_, BoxBrand, i32, RunExplicit<'static, RunStateRow, CNilBrand, i32>>| {
			match op {
				BoxState::Get(k) => {
					let s = *cell_for_handler.borrow();
					k(s)
				}
				BoxState::Put(s, k) => {
					*cell_for_handler.borrow_mut() = s;
					k(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 11);
	assert_eq!(*cell.borrow(), 11);
}

// -- RcRunExplicit --

#[test]
fn rc_run_explicit_get_returns_current_state() {
	let cell: Rc<RefCell<i32>> = Rc::new(RefCell::new(42));
	let cell_for_handler = Rc::clone(&cell);
	let prog: RcRunExplicit<'static, RcRunStateRow, CNilBrand, i32> = RcRunExplicit::get();
	let result = prog.interpret(handlers! {
		StateBrand<RcBrand, i32>: move |op: State<'_, RcBrand, i32, RcRunExplicit<'static, RcRunStateRow, CNilBrand, i32>>| {
			match op {
				State::Get(k) => {
					let s = *cell_for_handler.borrow();
					(*k)(s)
				}
				State::Put(s, k) => {
					*cell_for_handler.borrow_mut() = s;
					(*k)(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 42);
	assert_eq!(*cell.borrow(), 42);
}

#[test]
fn rc_run_explicit_put_writes_state() {
	let cell: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
	let cell_for_handler = Rc::clone(&cell);
	let prog: RcRunExplicit<'static, RcRunStateRow, CNilBrand, ()> =
		RcRunExplicit::put::<i32, _>(99);
	prog.interpret(handlers! {
		StateBrand<RcBrand, i32>: move |op: State<'_, RcBrand, i32, RcRunExplicit<'static, RcRunStateRow, CNilBrand, ()>>| {
			match op {
				State::Get(k) => {
					let s = *cell_for_handler.borrow();
					(*k)(s)
				}
				State::Put(s, k) => {
					*cell_for_handler.borrow_mut() = s;
					(*k)(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*cell.borrow(), 99);
}

#[test]
fn rc_run_explicit_get_put_get_bind_chain() {
	let cell: Rc<RefCell<i32>> = Rc::new(RefCell::new(10));
	let cell_for_handler = Rc::clone(&cell);
	let prog: RcRunExplicit<'static, RcRunStateRow, CNilBrand, i32> =
		RcRunExplicit::<'static, RcRunStateRow, CNilBrand, i32>::get()
			.bind(|s: i32| {
				RcRunExplicit::<'static, RcRunStateRow, CNilBrand, ()>::put::<i32, _>(s + 1)
			})
			.bind(|()| RcRunExplicit::<'static, RcRunStateRow, CNilBrand, i32>::get());
	let result = prog.interpret(handlers! {
		StateBrand<RcBrand, i32>: move |op: State<'_, RcBrand, i32, RcRunExplicit<'static, RcRunStateRow, CNilBrand, i32>>| {
			match op {
				State::Get(k) => {
					let s = *cell_for_handler.borrow();
					(*k)(s)
				}
				State::Put(s, k) => {
					*cell_for_handler.borrow_mut() = s;
					(*k)(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 11);
	assert_eq!(*cell.borrow(), 11);
}

// -- ArcRun --

type ArcRunStateRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;

#[test]
fn arc_run_get_returns_current_state() {
	let cell: Arc<Mutex<i32>> = Arc::new(Mutex::new(42));
	let cell_for_handler = Arc::clone(&cell);
	let prog: ArcRun<ArcRunStateRow, CNilBrand, i32> = ArcRun::get();
	let result = prog.interpret(handlers! {
		SendStateBrand<ArcBrand, i32>: move |op: SendState<'_, ArcBrand, i32, ArcRun<ArcRunStateRow, CNilBrand, i32>>| {
			match op {
				SendState::Get(k) => {
					let s = *cell_for_handler.lock().unwrap();
					(*k)(s)
				}
				SendState::Put(s, k) => {
					*cell_for_handler.lock().unwrap() = s;
					(*k)(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 42);
	assert_eq!(*cell.lock().unwrap(), 42);
}

#[test]
fn arc_run_put_writes_state() {
	let cell: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
	let cell_for_handler = Arc::clone(&cell);
	let prog: ArcRun<ArcRunStateRow, CNilBrand, ()> = ArcRun::put::<i32, _>(99);
	prog.interpret(handlers! {
		SendStateBrand<ArcBrand, i32>: move |op: SendState<'_, ArcBrand, i32, ArcRun<ArcRunStateRow, CNilBrand, ()>>| {
			match op {
				SendState::Get(k) => {
					let s = *cell_for_handler.lock().unwrap();
					(*k)(s)
				}
				SendState::Put(s, k) => {
					*cell_for_handler.lock().unwrap() = s;
					(*k)(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*cell.lock().unwrap(), 99);
}

#[test]
fn arc_run_get_put_get_bind_chain() {
	let cell: Arc<Mutex<i32>> = Arc::new(Mutex::new(10));
	let cell_for_handler = Arc::clone(&cell);
	let prog: ArcRun<ArcRunStateRow, CNilBrand, i32> =
		ArcRun::<ArcRunStateRow, CNilBrand, i32>::get()
			.bind(|s: i32| ArcRun::<ArcRunStateRow, CNilBrand, ()>::put::<i32, _>(s + 1))
			.bind(|()| ArcRun::<ArcRunStateRow, CNilBrand, i32>::get());
	let result = prog.interpret(handlers! {
		SendStateBrand<ArcBrand, i32>: move |op: SendState<'_, ArcBrand, i32, ArcRun<ArcRunStateRow, CNilBrand, i32>>| {
			match op {
				SendState::Get(k) => {
					let s = *cell_for_handler.lock().unwrap();
					(*k)(s)
				}
				SendState::Put(s, k) => {
					*cell_for_handler.lock().unwrap() = s;
					(*k)(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 11);
	assert_eq!(*cell.lock().unwrap(), 11);
}

// -- ArcRunExplicit --

#[test]
fn arc_run_explicit_get_returns_current_state() {
	let cell: Arc<Mutex<i32>> = Arc::new(Mutex::new(42));
	let cell_for_handler = Arc::clone(&cell);
	let prog: ArcRunExplicit<'static, ArcRunStateRow, CNilBrand, i32> = ArcRunExplicit::get();
	let result = prog.interpret(handlers! {
		SendStateBrand<ArcBrand, i32>: move |op: SendState<'_, ArcBrand, i32, ArcRunExplicit<'static, ArcRunStateRow, CNilBrand, i32>>| {
			match op {
				SendState::Get(k) => {
					let s = *cell_for_handler.lock().unwrap();
					(*k)(s)
				}
				SendState::Put(s, k) => {
					*cell_for_handler.lock().unwrap() = s;
					(*k)(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 42);
	assert_eq!(*cell.lock().unwrap(), 42);
}

#[test]
fn arc_run_explicit_put_writes_state() {
	let cell: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
	let cell_for_handler = Arc::clone(&cell);
	let prog: ArcRunExplicit<'static, ArcRunStateRow, CNilBrand, ()> =
		ArcRunExplicit::put::<i32, _>(99);
	prog.interpret(handlers! {
		SendStateBrand<ArcBrand, i32>: move |op: SendState<'_, ArcBrand, i32, ArcRunExplicit<'static, ArcRunStateRow, CNilBrand, ()>>| {
			match op {
				SendState::Get(k) => {
					let s = *cell_for_handler.lock().unwrap();
					(*k)(s)
				}
				SendState::Put(s, k) => {
					*cell_for_handler.lock().unwrap() = s;
					(*k)(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(*cell.lock().unwrap(), 99);
}

#[test]
fn arc_run_explicit_get_put_get_bind_chain() {
	let cell: Arc<Mutex<i32>> = Arc::new(Mutex::new(10));
	let cell_for_handler = Arc::clone(&cell);
	let prog: ArcRunExplicit<'static, ArcRunStateRow, CNilBrand, i32> =
		ArcRunExplicit::<'static, ArcRunStateRow, CNilBrand, i32>::get()
			.bind(|s: i32| {
				ArcRunExplicit::<'static, ArcRunStateRow, CNilBrand, ()>::put::<i32, _>(s + 1)
			})
			.bind(|()| ArcRunExplicit::<'static, ArcRunStateRow, CNilBrand, i32>::get());
	let result = prog.interpret(handlers! {
		SendStateBrand<ArcBrand, i32>: move |op: SendState<'_, ArcBrand, i32, ArcRunExplicit<'static, ArcRunStateRow, CNilBrand, i32>>| {
			match op {
				SendState::Get(k) => {
					let s = *cell_for_handler.lock().unwrap();
					(*k)(s)
				}
				SendState::Put(s, k) => {
					*cell_for_handler.lock().unwrap() = s;
					(*k)(())
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 11);
	assert_eq!(*cell.lock().unwrap(), 11);
}
