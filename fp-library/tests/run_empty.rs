//! End-to-end tests for the first-order `Empty` effect.
//!
//! `Empty` aborts the current branch without producing a value. The
//! handler decides what that means in the target interpretation. The
//! single-shot wrapper tests interpret `Empty` as a fallback value; the
//! multi-shot tests combine `Choose` with `Empty` and interpret the
//! empty branch as an empty result list.

use fp_library::{
	brands::*,
	handlers,
	types::effects::{
		arc_run::ArcRun,
		arc_run_explicit::ArcRunExplicit,
		choose::{
			Choose,
			SendChoose,
		},
		empty::Empty,
		rc_run::RcRun,
		rc_run_explicit::RcRunExplicit,
		run::Run,
		run_explicit::RunExplicit,
		scoped_nt,
	},
};

type RunEmptyRow = CoproductBrand<CoyonedaBrand<EmptyBrand>, CNilBrand>;
type RunEmptyProgram<A> = Run<RunEmptyRow, CNilBrand, A>;
type RunExplicitEmptyProgram<'a, A> = RunExplicit<'a, RunEmptyRow, CNilBrand, A>;
type RcEmptyRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
type ArcEmptyRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;

type RcNonDetRow = CoproductBrand<
	RcCoyonedaBrand<ChooseBrand<RcBrand>>,
	CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>,
>;
type RcNonDetProgram<A> = RcRun<RcNonDetRow, CNilBrand, A>;
type RcExplicitNonDetProgram<'a, A> = RcRunExplicit<'a, RcNonDetRow, CNilBrand, A>;

type ArcNonDetRow = CoproductBrand<
	ArcCoyonedaBrand<EmptyBrand>,
	CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, CNilBrand>,
>;
type ArcNonDetProgram<A> = ArcRun<ArcNonDetRow, CNilBrand, A>;
type ArcExplicitNonDetProgram<'a, A> = ArcRunExplicit<'a, ArcNonDetRow, CNilBrand, A>;

#[test]
fn run_empty_handler_returns_fallback_value() {
	let program: RunEmptyProgram<i32> = Run::empty();

	let result = program.handle(
		handlers! {
			EmptyBrand: |_op: Empty<'_, RunEmptyProgram<i32>>| Run::pure(0),
		},
		scoped_nt(),
	);

	assert_eq!(result, 0);
}

#[test]
fn run_explicit_empty_handler_returns_fallback_value() {
	let program: RunExplicitEmptyProgram<'static, i32> = RunExplicit::empty();

	let result = program.handle(
		handlers! {
			EmptyBrand: |_op: Empty<'_, RunExplicitEmptyProgram<'static, i32>>| RunExplicit::pure(0),
		},
		scoped_nt(),
	);

	assert_eq!(result, 0);
}

#[test]
fn named_run_empty_returns_none_for_empty_across_wrappers() {
	let run_program: Run<RunEmptyRow, CNilBrand, i32> = Run::empty();
	let run_handled: Run<CNilBrand, CNilBrand, Option<i32>> =
		run_program.run_empty::<_, CNilBrand>();
	assert_eq!(run_handled.extract(), None);

	let run_explicit_program: RunExplicit<'static, RunEmptyRow, CNilBrand, i32> =
		RunExplicit::empty();
	let run_explicit_handled: RunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		run_explicit_program.run_empty::<_, CNilBrand>();
	assert_eq!(run_explicit_handled.extract(), None);

	let rc_program: RcRun<RcEmptyRow, CNilBrand, i32> = RcRun::empty();
	let rc_handled: RcRun<CNilBrand, CNilBrand, Option<i32>> =
		rc_program.run_empty::<_, CNilBrand>();
	assert_eq!(rc_handled.extract(), None);

	let rc_explicit_program: RcRunExplicit<'static, RcEmptyRow, CNilBrand, i32> =
		RcRunExplicit::empty();
	let rc_explicit_handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		rc_explicit_program.run_empty::<_, CNilBrand>();
	assert_eq!(rc_explicit_handled.extract(), None);

	let arc_program: ArcRun<ArcEmptyRow, CNilBrand, i32> = ArcRun::empty();
	let arc_handled: ArcRun<CNilBrand, CNilBrand, Option<i32>> =
		arc_program.run_empty::<_, CNilBrand>();
	assert_eq!(arc_handled.extract(), None);

	let arc_explicit_program: ArcRunExplicit<'static, ArcEmptyRow, CNilBrand, i32> =
		ArcRunExplicit::empty();
	let arc_explicit_handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		arc_explicit_program.run_empty::<_, CNilBrand>();
	assert_eq!(arc_explicit_handled.extract(), None);
}

fn rc_choose_empty_program() -> RcNonDetProgram<Vec<i32>> {
	RcRun::<RcNonDetRow, CNilBrand, bool>::choose()
		.bind(|branch| if branch { RcRun::pure(vec![1]) } else { RcRun::empty() })
}

fn handle_rc_nondet(program: RcNonDetProgram<Vec<i32>>) -> Vec<i32> {
	program.handle(
		handlers! {
			ChooseBrand<RcBrand>: |op: Choose<'_, RcBrand, RcNonDetProgram<Vec<i32>>>| match op {
				Choose::Alt(k) => {
					let mut values = handle_rc_nondet((*k)(true));
					values.extend(handle_rc_nondet((*k)(false)));
					RcRun::pure(values)
				},
			},
			EmptyBrand: |_op: Empty<'_, RcNonDetProgram<Vec<i32>>>| RcRun::pure(Vec::new()),
		},
		scoped_nt(),
	)
}

#[test]
fn rc_run_choose_plus_empty_keeps_only_non_empty_branch() {
	assert_eq!(handle_rc_nondet(rc_choose_empty_program()), vec![1]);
}

fn rc_explicit_choose_empty_program() -> RcExplicitNonDetProgram<'static, Vec<i32>> {
	RcRunExplicit::<'static, RcNonDetRow, CNilBrand, bool>::choose()
		.bind(|branch| if branch { RcRunExplicit::pure(vec![1]) } else { RcRunExplicit::empty() })
}

fn handle_rc_explicit_nondet(program: RcExplicitNonDetProgram<'static, Vec<i32>>) -> Vec<i32> {
	program.handle(
		handlers! {
			ChooseBrand<RcBrand>: |op: Choose<'_, RcBrand, RcExplicitNonDetProgram<'static, Vec<i32>>>| match op {
				Choose::Alt(k) => {
					let mut values = handle_rc_explicit_nondet((*k)(true));
					values.extend(handle_rc_explicit_nondet((*k)(false)));
					RcRunExplicit::pure(values)
				},
			},
			EmptyBrand: |_op: Empty<'_, RcExplicitNonDetProgram<'static, Vec<i32>>>| {
				RcRunExplicit::pure(Vec::new())
			},
		},
		scoped_nt(),
	)
}

#[test]
fn rc_run_explicit_choose_plus_empty_keeps_only_non_empty_branch() {
	assert_eq!(handle_rc_explicit_nondet(rc_explicit_choose_empty_program()), vec![1]);
}

fn arc_choose_empty_program() -> ArcNonDetProgram<Vec<i32>> {
	ArcRun::<ArcNonDetRow, CNilBrand, bool>::choose()
		.bind(|branch| if branch { ArcRun::pure(vec![1]) } else { ArcRun::empty() })
}

fn handle_arc_nondet(program: ArcNonDetProgram<Vec<i32>>) -> Vec<i32> {
	program.handle(
		handlers! {
			SendChooseBrand<ArcBrand>: |op: SendChoose<'_, ArcBrand, ArcNonDetProgram<Vec<i32>>>| match op {
				SendChoose::Alt(k) => {
					let mut values = handle_arc_nondet((*k)(true));
					values.extend(handle_arc_nondet((*k)(false)));
					ArcRun::pure(values)
				},
			},
			EmptyBrand: |_op: Empty<'_, ArcNonDetProgram<Vec<i32>>>| ArcRun::pure(Vec::new()),
		},
		scoped_nt(),
	)
}

#[test]
fn arc_run_choose_plus_empty_keeps_only_non_empty_branch() {
	assert_eq!(handle_arc_nondet(arc_choose_empty_program()), vec![1]);
}

fn arc_explicit_choose_empty_program() -> ArcExplicitNonDetProgram<'static, Vec<i32>> {
	ArcRunExplicit::<'static, ArcNonDetRow, CNilBrand, bool>::choose()
		.bind(|branch| if branch { ArcRunExplicit::pure(vec![1]) } else { ArcRunExplicit::empty() })
}

fn handle_arc_explicit_nondet(program: ArcExplicitNonDetProgram<'static, Vec<i32>>) -> Vec<i32> {
	program.handle(
		handlers! {
			SendChooseBrand<ArcBrand>: |op: SendChoose<'_, ArcBrand, ArcExplicitNonDetProgram<'static, Vec<i32>>>| match op {
				SendChoose::Alt(k) => {
					let mut values = handle_arc_explicit_nondet((*k)(true));
					values.extend(handle_arc_explicit_nondet((*k)(false)));
					ArcRunExplicit::pure(values)
				},
			},
			EmptyBrand: |_op: Empty<'_, ArcExplicitNonDetProgram<'static, Vec<i32>>>| {
				ArcRunExplicit::pure(Vec::new())
			},
		},
		scoped_nt(),
	)
}

#[test]
fn arc_run_explicit_choose_plus_empty_keeps_only_non_empty_branch() {
	assert_eq!(handle_arc_explicit_nondet(arc_explicit_choose_empty_program()), vec![1]);
}
