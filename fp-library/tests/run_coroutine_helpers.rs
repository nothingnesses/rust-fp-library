#![cfg(feature = "effects")]
#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]

//! Coroutine helper coverage for Run wrapper variants.
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
			RunCoroutineStatus,
			RunExplicitCoroutineStatus,
		},
		rc_run::RcRun,
		rc_run_explicit::RcRunExplicit,
		run::Run,
		run_explicit::RunExplicit,
	},
};

struct NonClone(i32);

type BoxCoroutineRow =
	CoproductBrand<CoyonedaBrand<BoxCoroutineBrand<BoxBrand, &'static str, i32>>, CNilBrand>;
type BoxReaderRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, u32>>, CNilBrand>;
type BoxCoroutineThenReaderRow =
	CoproductBrand<CoyonedaBrand<BoxCoroutineBrand<BoxBrand, &'static str, i32>>, BoxReaderRow>;
type RcCoroutineRow =
	CoproductBrand<RcCoyonedaBrand<CoroutineBrand<RcBrand, &'static str, i32>>, CNilBrand>;
type ArcCoroutineRow =
	CoproductBrand<ArcCoyonedaBrand<SendCoroutineBrand<ArcBrand, &'static str, i32>>, CNilBrand>;

#[test]
fn run_coroutine_returns_done_for_pure_program() {
	let program: Run<BoxCoroutineRow, CNilBrand, i32> = Run::pure(7);

	let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();

	assert!(matches!(status, RunCoroutineStatus::Done(7)));
}

#[test]
fn run_coroutine_preserves_residual_rows_and_one_shot_resume() {
	let program: Run<BoxCoroutineThenReaderRow, CNilBrand, i32> =
		Run::<BoxCoroutineThenReaderRow, CNilBrand, i32>::yield_value::<&'static str, _>("first")
			.bind(|input| {
				Run::<BoxCoroutineThenReaderRow, CNilBrand, u32>::ask::<_>()
					.map(move |env| input + env as i32)
			});

	let status_program: Run<
		BoxReaderRow,
		CNilBrand,
		RunCoroutineStatus<BoxReaderRow, CNilBrand, &'static str, i32, i32>,
	> = program.run_coroutine::<&'static str, i32, _, BoxReaderRow>();
	let status = status_program.run_reader::<u32, _, CNilBrand>(10).extract();
	let RunCoroutineStatus::Continue(out, resume) = status else {
		panic!("expected Coroutine yield status");
	};

	assert_eq!(out, "first");
	assert!(matches!(
		resume(5).run_reader::<u32, _, CNilBrand>(10).extract(),
		RunCoroutineStatus::Done(15)
	));
}

#[test]
fn run_coroutine_does_not_require_clone_for_result_or_continuation_capture() {
	let captured = NonClone(1);
	let program: Run<BoxCoroutineRow, CNilBrand, NonClone> =
		Run::<BoxCoroutineRow, CNilBrand, i32>::yield_value::<&'static str, _>("first")
			.map(move |input| NonClone(input + captured.0));

	let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
	let RunCoroutineStatus::Continue(out, resume) = status else {
		panic!("expected Coroutine yield status");
	};
	let RunCoroutineStatus::Done(result) = resume(41).extract() else {
		panic!("expected completed Coroutine status");
	};

	assert_eq!(out, "first");
	assert_eq!(result.0, 42);
}

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
fn run_explicit_coroutine_preserves_residual_rows_and_one_shot_resume() {
	let program: RunExplicit<'static, BoxCoroutineThenReaderRow, CNilBrand, i32> =
		RunExplicit::<'static, BoxCoroutineThenReaderRow, CNilBrand, i32>::yield_value::<
			&'static str,
			_,
		>("first")
		.bind(|input| {
			RunExplicit::<'static, BoxCoroutineThenReaderRow, CNilBrand, u32>::ask::<_>()
				.map(move |env| input + env as i32)
		});

	let status_program: RunExplicit<
		'static,
		BoxReaderRow,
		CNilBrand,
		RunExplicitCoroutineStatus<'static, BoxReaderRow, CNilBrand, &'static str, i32, i32>,
	> = program.run_coroutine::<&'static str, i32, _, BoxReaderRow>();
	let status = status_program.run_reader::<u32, _, CNilBrand>(10).extract();
	let RunExplicitCoroutineStatus::Continue(out, resume) = status else {
		panic!("expected Coroutine yield status");
	};

	assert_eq!(out, "first");
	assert!(matches!(
		resume(5).run_reader::<u32, _, CNilBrand>(10).extract(),
		RunExplicitCoroutineStatus::Done(15)
	));
}

#[test]
fn run_explicit_coroutine_does_not_require_clone_for_result_or_capture() {
	let captured = NonClone(1);
	let program: RunExplicit<'static, BoxCoroutineRow, CNilBrand, NonClone> =
		RunExplicit::<'static, BoxCoroutineRow, CNilBrand, i32>::yield_value::<&'static str, _>(
			"first",
		)
		.map(move |input| NonClone(input + captured.0));

	let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
	let RunExplicitCoroutineStatus::Continue(out, resume) = status else {
		panic!("expected Coroutine yield status");
	};
	let RunExplicitCoroutineStatus::Done(result) = resume(41).extract() else {
		panic!("expected completed Coroutine status");
	};

	assert_eq!(out, "first");
	assert_eq!(result.0, 42);
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
