// Integration tests for the named Reader helpers on all Run wrappers.
//
// `asks` must behave as `ask().map(f)`, and `run_reader` must remove
// the Reader effect by supplying a clone of the environment to each Ask.
// The tests use the wrapper-specific Reader brands because the boxed,
// Rc, and Arc substrates store different continuation pointer shapes.

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

type RunReaderRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
type RcRunReaderRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
type ArcRunReaderRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;

#[test]
fn run_asks_and_run_reader_project_environment() {
	let program: Run<RunReaderRow, CNilBrand, String> =
		Run::asks::<i32, _>(|env| format!("env={env}"));
	let handled: Run<CNilBrand, CNilBrand, String> = program.run_reader::<i32, _, CNilBrand>(7);

	assert_eq!(handled.extract(), "env=7");
}

#[test]
fn rc_run_asks_and_run_reader_project_environment() {
	let program: RcRun<RcRunReaderRow, CNilBrand, String> =
		RcRun::asks::<i32, _>(|env| format!("env={env}"));
	let handled: RcRun<CNilBrand, CNilBrand, String> = program.run_reader::<i32, _, CNilBrand>(7);

	assert_eq!(handled.extract(), "env=7");
}

#[test]
fn arc_run_asks_and_run_reader_project_environment() {
	let program: ArcRun<ArcRunReaderRow, CNilBrand, String> =
		ArcRun::asks::<i32, _>(|env| format!("env={env}"));
	let handled: ArcRun<CNilBrand, CNilBrand, String> = program.run_reader::<i32, _, CNilBrand>(7);

	assert_eq!(handled.extract(), "env=7");
}

#[test]
fn run_explicit_asks_and_run_reader_project_environment() {
	let program: RunExplicit<'static, RunReaderRow, CNilBrand, String> =
		RunExplicit::asks::<i32, _>(|env| format!("env={env}"));
	let handled: RunExplicit<'static, CNilBrand, CNilBrand, String> =
		program.run_reader::<i32, _, CNilBrand>(7);

	assert_eq!(handled.extract(), "env=7");
}

#[test]
fn rc_run_explicit_asks_and_run_reader_project_environment() {
	let program: RcRunExplicit<'static, RcRunReaderRow, CNilBrand, String> =
		RcRunExplicit::asks::<i32, _>(|env| format!("env={env}"));
	let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, String> =
		program.run_reader::<i32, _, CNilBrand>(7);

	assert_eq!(handled.extract(), "env=7");
}

#[test]
fn arc_run_explicit_asks_and_run_reader_project_environment() {
	let program: ArcRunExplicit<'static, ArcRunReaderRow, CNilBrand, String> =
		ArcRunExplicit::asks::<i32, _>(|env| format!("env={env}"));
	let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, String> =
		program.run_reader::<i32, _, CNilBrand>(7);

	assert_eq!(handled.extract(), "env=7");
}
