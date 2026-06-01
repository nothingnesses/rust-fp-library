#![cfg(feature = "effects")]
#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]

//! Coroutine helper coverage for the multi-shot Run wrappers.
//!
//! A Coroutine program yields an output value and resumes when the runner
//! supplies an input value. `run_coroutine` removes one Coroutine row cell
//! and returns a status in the residual row; resume continuations return
//! the next status rather than the final result directly.

use fp_library::{
	brands::*,
	types::effects::{
		arc_run::ArcRun,
		arc_run_explicit::ArcRunExplicit,
		coroutine::{
			ArcRunCoroutineStatus,
			ArcRunExplicitCoroutineStatus,
			RcRunCoroutineStatus,
			RcRunExplicitCoroutineStatus,
		},
		rc_run::RcRun,
		rc_run_explicit::RcRunExplicit,
	},
};

type RcCoroutineRow =
	CoproductBrand<RcCoyonedaBrand<CoroutineBrand<RcBrand, &'static str, i32>>, CNilBrand>;
type ArcCoroutineRow =
	CoproductBrand<ArcCoyonedaBrand<SendCoroutineBrand<ArcBrand, &'static str, i32>>, CNilBrand>;

#[test]
fn rcrun_coroutine_returns_done_for_pure_program() {
	let program: RcRun<RcCoroutineRow, CNilBrand, i32> = RcRun::pure(7);

	let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();

	assert!(matches!(status, RcRunCoroutineStatus::Done(7)));
}

#[test]
fn rcrun_coroutine_resumes_to_next_status_and_is_multi_shot() {
	let program: RcRun<RcCoroutineRow, CNilBrand, i32> =
		RcRun::<RcCoroutineRow, CNilBrand, i32>::yield_value::<&'static str, _>("first")
			.bind(|input| RcRun::pure(input + 1));

	let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
	let RcRunCoroutineStatus::Continue(out, resume) = status else {
		panic!("expected Coroutine yield status");
	};

	assert_eq!(out, "first");
	assert!(matches!(resume(41).extract(), RcRunCoroutineStatus::Done(42)));
	assert!(matches!(resume(1).extract(), RcRunCoroutineStatus::Done(2)));
}

#[test]
fn rcrun_coroutine_can_resume_to_another_continue() {
	let program: RcRun<RcCoroutineRow, CNilBrand, i32> =
		RcRun::<RcCoroutineRow, CNilBrand, i32>::yield_value::<&'static str, _>("first").bind(
			|first_input| {
				RcRun::<RcCoroutineRow, CNilBrand, i32>::yield_value::<&'static str, _>("second")
					.map(move |second_input| first_input + second_input)
			},
		);

	let first_status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
	let RcRunCoroutineStatus::Continue(first_out, first_resume) = first_status else {
		panic!("expected first Coroutine yield status");
	};
	assert_eq!(first_out, "first");

	let second_status = first_resume(10).extract();
	let RcRunCoroutineStatus::Continue(second_out, second_resume) = second_status else {
		panic!("expected second Coroutine yield status");
	};
	assert_eq!(second_out, "second");
	assert!(matches!(second_resume(5).extract(), RcRunCoroutineStatus::Done(15)));
}

#[test]
fn arcrun_coroutine_resumes_to_next_status_and_is_multi_shot() {
	let program: ArcRun<ArcCoroutineRow, CNilBrand, i32> =
		ArcRun::<ArcCoroutineRow, CNilBrand, i32>::yield_value::<&'static str, _>("first")
			.bind(|input| ArcRun::pure(input + 1));

	let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
	let ArcRunCoroutineStatus::Continue(out, resume) = status else {
		panic!("expected Coroutine yield status");
	};

	assert_eq!(out, "first");
	assert!(matches!(resume(41).extract(), ArcRunCoroutineStatus::Done(42)));
	assert!(matches!(resume(1).extract(), ArcRunCoroutineStatus::Done(2)));
}

#[test]
fn rcrun_explicit_coroutine_resumes_to_next_status() {
	let program: RcRunExplicit<'static, RcCoroutineRow, CNilBrand, i32> =
		RcRunExplicit::<'static, RcCoroutineRow, CNilBrand, i32>::yield_value::<&'static str, _>(
			"first",
		)
		.bind(|input| RcRunExplicit::pure(input + 1));

	let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
	let RcRunExplicitCoroutineStatus::Continue(out, resume) = status else {
		panic!("expected Coroutine yield status");
	};

	assert_eq!(out, "first");
	assert!(matches!(resume(41).extract(), RcRunExplicitCoroutineStatus::Done(42)));
}

#[test]
fn arcrun_explicit_coroutine_resumes_to_next_status() {
	let program: ArcRunExplicit<'static, ArcCoroutineRow, CNilBrand, i32> =
		ArcRunExplicit::<'static, ArcCoroutineRow, CNilBrand, i32>::yield_value::<&'static str, _>(
			"first",
		)
		.bind(|input| ArcRunExplicit::pure(input + 1));

	let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
	let ArcRunExplicitCoroutineStatus::Continue(out, resume) = status else {
		panic!("expected Coroutine yield status");
	};

	assert_eq!(out, "first");
	assert!(matches!(resume(41).extract(), ArcRunExplicitCoroutineStatus::Done(42)));
}
