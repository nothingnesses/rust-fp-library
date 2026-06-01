#![cfg(feature = "effects")]
#![expect(
	clippy::unwrap_used,
	clippy::panic,
	reason = "Tests use panicking operations for brevity and clarity."
)]

// Integration tests for the Choose effect smart constructors on
// the four multi-shot Run wrappers. The single-shot wrappers
// (Run, RunExplicit) cannot host Choose because the handler must
// invoke the continuation twice (once per branch).
//
// Each wrapper is exercised end-to-end with:
//   - choose_branches_capture_both_paths: a single Alt effect
//     dispatched through a handler that captures both branches
//     into a Vec, verifying the continuation runs twice (once
//     with true, once with false) and the captured branch values
//     reach the handler.
//
// The two non-Arc multi-shot wrappers (RcRun, RcRunExplicit)
// thread `RcBrand` and use `ChooseBrand<RcBrand>` in the row.
// The two Arc wrappers (ArcRun, ArcRunExplicit) thread `ArcBrand`
// and use `SendChooseBrand<ArcBrand>` whose closure projection
// bakes in `Send + Sync`.

use {
	fp_library::{
		brands::*,
		handlers,
		types::effects::{
			arc_run::ArcRun,
			arc_run_explicit::ArcRunExplicit,
			choose::{
				Choose,
				SendChoose,
			},
			rc_run::RcRun,
			rc_run_explicit::RcRunExplicit,
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

// -- RcRun --

type RcRunChooseRow = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;

#[test]
fn rc_run_choose_branches_capture_both_paths() {
	let captured: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));
	let captured_for_handler = Rc::clone(&captured);
	let prog: RcRun<RcRunChooseRow, CNilBrand, i32> =
		RcRun::<RcRunChooseRow, CNilBrand, bool>::choose()
			.bind(|b: bool| RcRun::<RcRunChooseRow, CNilBrand, i32>::pure(if b { 1 } else { 0 }));
	let result = prog.handle(
		handlers! {
			ChooseBrand<RcBrand>: move |op: Choose<'_, RcBrand, RcRun<RcRunChooseRow, CNilBrand, i32>>| {
				match op {
					Choose::Alt(k) => {
						let true_branch = (*k)(true);
						let false_branch = (*k)(false);
						let true_value = match true_branch.peel() {
							Ok(v) => v,
							Err(_) => panic!("expected pure value after choose continuation"),
						};
						let false_value = match false_branch.peel() {
							Ok(v) => v,
							Err(_) => panic!("expected pure value after choose continuation"),
						};
						captured_for_handler.borrow_mut().push(true_value);
						captured_for_handler.borrow_mut().push(false_value);
						RcRun::pure(true_value + false_value)
					}
				}
			},
		},
		fp_library::types::effects::scoped_nt(),
	);
	assert_eq!(result, 1);
	assert_eq!(*captured.borrow(), vec![1, 0]);
}

#[test]
fn rc_run_named_run_choose_collects_branches() {
	let program: RcRun<RcRunChooseRow, CNilBrand, i32> =
		RcRun::<RcRunChooseRow, CNilBrand, bool>::choose()
			.bind(|branch| RcRun::pure(if branch { 1 } else { 0 }));
	let handled: RcRun<CNilBrand, CNilBrand, Vec<i32>> = program.run_choose::<_, CNilBrand>();
	assert_eq!(handled.extract(), vec![1, 0]);
}

// -- RcRunExplicit --

#[test]
fn rc_run_explicit_choose_branches_capture_both_paths() {
	let captured: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));
	let captured_for_handler = Rc::clone(&captured);
	let prog: RcRunExplicit<'static, RcRunChooseRow, CNilBrand, i32> =
		RcRunExplicit::<'static, RcRunChooseRow, CNilBrand, bool>::choose().bind(|b: bool| {
			RcRunExplicit::<'static, RcRunChooseRow, CNilBrand, i32>::pure(if b { 1 } else { 0 })
		});
	let result = prog.handle(handlers! {
		ChooseBrand<RcBrand>: move |op: Choose<'_, RcBrand, RcRunExplicit<'static, RcRunChooseRow, CNilBrand, i32>>| {
			match op {
				Choose::Alt(k) => {
					let true_branch = (*k)(true);
					let false_branch = (*k)(false);
					let true_value = match true_branch.peel() {
						Ok(v) => v,
						Err(_) => panic!("expected pure value after choose continuation"),
					};
					let false_value = match false_branch.peel() {
						Ok(v) => v,
						Err(_) => panic!("expected pure value after choose continuation"),
					};
					captured_for_handler.borrow_mut().push(true_value);
					captured_for_handler.borrow_mut().push(false_value);
					RcRunExplicit::pure(true_value + false_value)
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 1);
	assert_eq!(*captured.borrow(), vec![1, 0]);
}

#[test]
fn rc_run_explicit_named_run_choose_collects_branches() {
	let program: RcRunExplicit<'static, RcRunChooseRow, CNilBrand, i32> =
		RcRunExplicit::<'static, RcRunChooseRow, CNilBrand, bool>::choose()
			.bind(|branch| RcRunExplicit::pure(if branch { 1 } else { 0 }));
	let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		program.run_choose::<_, CNilBrand>();
	assert_eq!(handled.extract(), vec![1, 0]);
}

// -- ArcRun --

type ArcRunChooseRow = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, CNilBrand>;

#[test]
fn arc_run_choose_branches_capture_both_paths() {
	let captured: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));
	let captured_for_handler = Arc::clone(&captured);
	let prog: ArcRun<ArcRunChooseRow, CNilBrand, i32> =
		ArcRun::<ArcRunChooseRow, CNilBrand, bool>::choose()
			.bind(|b: bool| ArcRun::<ArcRunChooseRow, CNilBrand, i32>::pure(if b { 1 } else { 0 }));
	let result = prog.handle(handlers! {
		SendChooseBrand<ArcBrand>: move |op: SendChoose<'_, ArcBrand, ArcRun<ArcRunChooseRow, CNilBrand, i32>>| {
			match op {
				SendChoose::Alt(k) => {
					let true_branch = (*k)(true);
					let false_branch = (*k)(false);
					let true_value = match true_branch.peel() {
						Ok(v) => v,
						Err(_) => panic!("expected pure value after choose continuation"),
					};
					let false_value = match false_branch.peel() {
						Ok(v) => v,
						Err(_) => panic!("expected pure value after choose continuation"),
					};
					captured_for_handler.lock().unwrap().push(true_value);
					captured_for_handler.lock().unwrap().push(false_value);
					ArcRun::pure(true_value + false_value)
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 1);
	assert_eq!(*captured.lock().unwrap(), vec![1, 0]);
}

#[test]
fn arc_run_named_run_choose_collects_branches() {
	let program: ArcRun<ArcRunChooseRow, CNilBrand, i32> =
		ArcRun::<ArcRunChooseRow, CNilBrand, bool>::choose()
			.bind(|branch| ArcRun::pure(if branch { 1 } else { 0 }));
	let handled: ArcRun<CNilBrand, CNilBrand, Vec<i32>> = program.run_choose::<_, CNilBrand>();
	assert_eq!(handled.extract(), vec![1, 0]);
}

// -- ArcRunExplicit --

#[test]
fn arc_run_explicit_choose_branches_capture_both_paths() {
	let captured: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));
	let captured_for_handler = Arc::clone(&captured);
	let prog: ArcRunExplicit<'static, ArcRunChooseRow, CNilBrand, i32> =
		ArcRunExplicit::<'static, ArcRunChooseRow, CNilBrand, bool>::choose().bind(|b: bool| {
			ArcRunExplicit::<'static, ArcRunChooseRow, CNilBrand, i32>::pure(if b { 1 } else { 0 })
		});
	let result = prog.handle(handlers! {
		SendChooseBrand<ArcBrand>: move |op: SendChoose<'_, ArcBrand, ArcRunExplicit<'static, ArcRunChooseRow, CNilBrand, i32>>| {
			match op {
				SendChoose::Alt(k) => {
					let true_branch = (*k)(true);
					let false_branch = (*k)(false);
					let true_value = match true_branch.peel() {
						Ok(v) => v,
						Err(_) => panic!("expected pure value after choose continuation"),
					};
					let false_value = match false_branch.peel() {
						Ok(v) => v,
						Err(_) => panic!("expected pure value after choose continuation"),
					};
					captured_for_handler.lock().unwrap().push(true_value);
					captured_for_handler.lock().unwrap().push(false_value);
					ArcRunExplicit::pure(true_value + false_value)
				}
			}
		},
	}, fp_library::types::effects::scoped_nt());
	assert_eq!(result, 1);
	assert_eq!(*captured.lock().unwrap(), vec![1, 0]);
}

#[test]
fn arc_run_explicit_named_run_choose_collects_branches() {
	let program: ArcRunExplicit<'static, ArcRunChooseRow, CNilBrand, i32> =
		ArcRunExplicit::<'static, ArcRunChooseRow, CNilBrand, bool>::choose()
			.bind(|branch| ArcRunExplicit::pure(if branch { 1 } else { 0 }));
	let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		program.run_choose::<_, CNilBrand>();
	assert_eq!(handled.extract(), vec![1, 0]);
}
