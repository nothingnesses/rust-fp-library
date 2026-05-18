// Integration tests for the named State helpers on non-explicit Run wrappers.
//
// `gets` must behave as `get().map(f)`, `modify` must behave as
// `get().bind(|s| put(f(s)))`, and the state runners must thread state
// through the existing first-order handler machinery.

use fp_library::{
	brands::*,
	types::effects::{
		arc_run::ArcRun,
		arc_run_explicit::ArcRunExplicit,
		rc_run::RcRun,
		rc_run_explicit::RcRunExplicit,
		run::Run,
		run_explicit::RunExplicit,
	},
};

type RunStateRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
type RunExplicitStateRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
type RcRunStateRow = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
type RcRunExplicitStateRow = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
type ArcRunStateRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
type ArcRunExplicitStateRow =
	CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;

#[test]
fn run_state_helpers_thread_state() {
	let program: Run<RunStateRow, CNilBrand, i32> = Run::<RunStateRow, CNilBrand, i32>::get()
		.bind(|state| Run::<RunStateRow, CNilBrand, ()>::put::<i32, _>(state + 1))
		.bind(|()| Run::<RunStateRow, CNilBrand, i32>::get());
	let handled: Run<CNilBrand, CNilBrand, (i32, i32)> = program.run_state::<i32, _, CNilBrand>(41);
	assert_eq!(handled.extract(), (42, 42));

	let projected: Run<RunStateRow, CNilBrand, String> =
		Run::gets::<i32, _>(|state| format!("state={state}"));
	let evaluated: Run<CNilBrand, CNilBrand, String> = projected.eval_state::<i32, _, CNilBrand>(7);
	assert_eq!(evaluated.extract(), "state=7");

	let update: Run<RunStateRow, CNilBrand, ()> = Run::modify::<i32, _>(|state| state + 5);
	let final_state: Run<CNilBrand, CNilBrand, i32> = update.exec_state::<i32, _, CNilBrand>(7);
	assert_eq!(final_state.extract(), 12);
}

#[test]
fn run_explicit_state_helpers_thread_state() {
	let program: RunExplicit<'static, RunExplicitStateRow, CNilBrand, i32> =
		RunExplicit::<'static, RunExplicitStateRow, CNilBrand, i32>::get()
			.bind(|state| {
				RunExplicit::<'static, RunExplicitStateRow, CNilBrand, ()>::put::<i32, _>(state + 1)
			})
			.bind(|()| RunExplicit::<'static, RunExplicitStateRow, CNilBrand, i32>::get());
	let handled: RunExplicit<'static, CNilBrand, CNilBrand, (i32, i32)> =
		program.run_state::<i32, _, CNilBrand>(41);
	assert_eq!(handled.extract(), (42, 42));

	let projected: RunExplicit<'static, RunExplicitStateRow, CNilBrand, String> =
		RunExplicit::gets::<i32, _>(|state| format!("state={state}"));
	let evaluated: RunExplicit<'static, CNilBrand, CNilBrand, String> =
		projected.eval_state::<i32, _, CNilBrand>(7);
	assert_eq!(evaluated.extract(), "state=7");

	let update: RunExplicit<'static, RunExplicitStateRow, CNilBrand, ()> =
		RunExplicit::modify::<i32, _>(|state| state + 5);
	let final_state: RunExplicit<'static, CNilBrand, CNilBrand, i32> =
		update.exec_state::<i32, _, CNilBrand>(7);
	assert_eq!(final_state.extract(), 12);
}

#[test]
fn rc_run_state_helpers_thread_state() {
	let program: RcRun<RcRunStateRow, CNilBrand, i32> =
		RcRun::<RcRunStateRow, CNilBrand, i32>::get()
			.bind(|state| RcRun::<RcRunStateRow, CNilBrand, ()>::put::<i32, _>(state + 1))
			.bind(|()| RcRun::<RcRunStateRow, CNilBrand, i32>::get());
	let handled: RcRun<CNilBrand, CNilBrand, (i32, i32)> =
		program.run_state::<i32, _, CNilBrand>(41);
	assert_eq!(handled.extract(), (42, 42));

	let projected: RcRun<RcRunStateRow, CNilBrand, String> =
		RcRun::gets::<i32, _>(|state| format!("state={state}"));
	let evaluated: RcRun<CNilBrand, CNilBrand, String> =
		projected.eval_state::<i32, _, CNilBrand>(7);
	assert_eq!(evaluated.extract(), "state=7");

	let update: RcRun<RcRunStateRow, CNilBrand, ()> = RcRun::modify::<i32, _>(|state| state + 5);
	let final_state: RcRun<CNilBrand, CNilBrand, i32> = update.exec_state::<i32, _, CNilBrand>(7);
	assert_eq!(final_state.extract(), 12);
}

#[test]
fn rc_run_explicit_state_helpers_thread_state() {
	let program: RcRunExplicit<'static, RcRunExplicitStateRow, CNilBrand, i32> =
		RcRunExplicit::<'static, RcRunExplicitStateRow, CNilBrand, i32>::get()
			.bind(|state| {
				RcRunExplicit::<'static, RcRunExplicitStateRow, CNilBrand, ()>::put::<i32, _>(
					state + 1,
				)
			})
			.bind(|()| RcRunExplicit::<'static, RcRunExplicitStateRow, CNilBrand, i32>::get());
	let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, i32)> =
		program.run_state::<i32, _, CNilBrand>(41);
	assert_eq!(handled.extract(), (42, 42));

	let projected: RcRunExplicit<'static, RcRunExplicitStateRow, CNilBrand, String> =
		RcRunExplicit::gets::<i32, _>(|state| format!("state={state}"));
	let evaluated: RcRunExplicit<'static, CNilBrand, CNilBrand, String> =
		projected.eval_state::<i32, _, CNilBrand>(7);
	assert_eq!(evaluated.extract(), "state=7");

	let update: RcRunExplicit<'static, RcRunExplicitStateRow, CNilBrand, ()> =
		RcRunExplicit::modify::<i32, _>(|state| state + 5);
	let final_state: RcRunExplicit<'static, CNilBrand, CNilBrand, i32> =
		update.exec_state::<i32, _, CNilBrand>(7);
	assert_eq!(final_state.extract(), 12);
}

#[test]
fn arc_run_state_helpers_thread_state() {
	let program: ArcRun<ArcRunStateRow, CNilBrand, i32> =
		ArcRun::<ArcRunStateRow, CNilBrand, i32>::get()
			.bind(|state| ArcRun::<ArcRunStateRow, CNilBrand, ()>::put::<i32, _>(state + 1))
			.bind(|()| ArcRun::<ArcRunStateRow, CNilBrand, i32>::get());
	let handled: ArcRun<CNilBrand, CNilBrand, (i32, i32)> =
		program.run_state::<i32, _, CNilBrand>(41);
	assert_eq!(handled.extract(), (42, 42));

	let projected: ArcRun<ArcRunStateRow, CNilBrand, String> =
		ArcRun::gets::<i32, _>(|state| format!("state={state}"));
	let evaluated: ArcRun<CNilBrand, CNilBrand, String> =
		projected.eval_state::<i32, _, CNilBrand>(7);
	assert_eq!(evaluated.extract(), "state=7");

	let update: ArcRun<ArcRunStateRow, CNilBrand, ()> = ArcRun::modify::<i32, _>(|state| state + 5);
	let final_state: ArcRun<CNilBrand, CNilBrand, i32> = update.exec_state::<i32, _, CNilBrand>(7);
	assert_eq!(final_state.extract(), 12);
}

#[test]
fn arc_run_explicit_state_helpers_thread_state() {
	let program: ArcRunExplicit<'static, ArcRunExplicitStateRow, CNilBrand, i32> =
		ArcRunExplicit::<'static, ArcRunExplicitStateRow, CNilBrand, i32>::get()
			.bind(|state| {
				ArcRunExplicit::<'static, ArcRunExplicitStateRow, CNilBrand, ()>::put::<i32, _>(
					state + 1,
				)
			})
			.bind(|()| ArcRunExplicit::<'static, ArcRunExplicitStateRow, CNilBrand, i32>::get());
	let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, i32)> =
		program.run_state::<i32, _, CNilBrand>(41);
	assert_eq!(handled.extract(), (42, 42));

	let projected: ArcRunExplicit<'static, ArcRunExplicitStateRow, CNilBrand, String> =
		ArcRunExplicit::gets::<i32, _>(|state| format!("state={state}"));
	let evaluated: ArcRunExplicit<'static, CNilBrand, CNilBrand, String> =
		projected.eval_state::<i32, _, CNilBrand>(7);
	assert_eq!(evaluated.extract(), "state=7");

	let update: ArcRunExplicit<'static, ArcRunExplicitStateRow, CNilBrand, ()> =
		ArcRunExplicit::modify::<i32, _>(|state| state + 5);
	let final_state: ArcRunExplicit<'static, CNilBrand, CNilBrand, i32> =
		update.exec_state::<i32, _, CNilBrand>(7);
	assert_eq!(final_state.extract(), 12);
}
