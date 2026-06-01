#![cfg(feature = "effects")]

//! Integration tests for the named Log helpers.
//!
//! `log` emits one message without changing the program result. The
//! named runners must preserve emission order whether they collect
//! messages directly into a vector or map each message into a monoidal
//! accumulator. The effect uses `LogBrand`, so it remains a distinct
//! row member from Output and Writer even though all three carry direct
//! payload-like values.

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

type RunLogRow = CoproductBrand<CoyonedaBrand<LogBrand<&'static str>>, CNilBrand>;
type RcRunLogRow = CoproductBrand<RcCoyonedaBrand<LogBrand<&'static str>>, CNilBrand>;
type ArcRunLogRow = CoproductBrand<ArcCoyonedaBrand<LogBrand<&'static str>>, CNilBrand>;

fn run_program() -> Run<RunLogRow, CNilBrand, i32> {
	Run::<RunLogRow, CNilBrand, ()>::log::<&'static str, _>("first")
		.bind(|()| Run::<RunLogRow, CNilBrand, ()>::log::<&'static str, _>("second"))
		.bind(|()| Run::<RunLogRow, CNilBrand, i32>::pure(7))
}

fn rc_run_program() -> RcRun<RcRunLogRow, CNilBrand, i32> {
	RcRun::<RcRunLogRow, CNilBrand, ()>::log::<&'static str, _>("first")
		.bind(|()| RcRun::<RcRunLogRow, CNilBrand, ()>::log::<&'static str, _>("second"))
		.bind(|()| RcRun::<RcRunLogRow, CNilBrand, i32>::pure(7))
}

fn arc_run_program() -> ArcRun<ArcRunLogRow, CNilBrand, i32> {
	ArcRun::<ArcRunLogRow, CNilBrand, ()>::log::<&'static str, _>("first")
		.bind(|()| ArcRun::<ArcRunLogRow, CNilBrand, ()>::log::<&'static str, _>("second"))
		.bind(|()| ArcRun::<ArcRunLogRow, CNilBrand, i32>::pure(7))
}

fn run_explicit_program() -> RunExplicit<'static, RunLogRow, CNilBrand, i32> {
	RunExplicit::<'static, RunLogRow, CNilBrand, ()>::log::<&'static str, _>("first")
		.bind(|()| {
			RunExplicit::<'static, RunLogRow, CNilBrand, ()>::log::<&'static str, _>("second")
		})
		.bind(|()| RunExplicit::<'static, RunLogRow, CNilBrand, i32>::pure(7))
}

fn rc_run_explicit_program() -> RcRunExplicit<'static, RcRunLogRow, CNilBrand, i32> {
	RcRunExplicit::<'static, RcRunLogRow, CNilBrand, ()>::log::<&'static str, _>("first")
		.bind(|()| {
			RcRunExplicit::<'static, RcRunLogRow, CNilBrand, ()>::log::<&'static str, _>("second")
		})
		.bind(|()| RcRunExplicit::<'static, RcRunLogRow, CNilBrand, i32>::pure(7))
}

fn arc_run_explicit_program() -> ArcRunExplicit<'static, ArcRunLogRow, CNilBrand, i32> {
	ArcRunExplicit::<'static, ArcRunLogRow, CNilBrand, ()>::log::<&'static str, _>("first")
		.bind(|()| {
			ArcRunExplicit::<'static, ArcRunLogRow, CNilBrand, ()>::log::<&'static str, _>("second")
		})
		.bind(|()| ArcRunExplicit::<'static, ArcRunLogRow, CNilBrand, i32>::pure(7))
}

#[test]
fn run_log_helpers_preserve_order() {
	let vector: Run<CNilBrand, CNilBrand, (i32, Vec<&'static str>)> =
		run_program().run_log_vec::<&'static str, _, CNilBrand>();
	assert_eq!(vector.extract(), (7, vec!["first", "second"]));

	let monoid: Run<CNilBrand, CNilBrand, (i32, String)> =
		run_program().run_log_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
	assert_eq!(monoid.extract(), (7, "firstsecond".to_string()));
}

#[test]
fn rc_run_log_helpers_preserve_order() {
	let vector: RcRun<CNilBrand, CNilBrand, (i32, Vec<&'static str>)> =
		rc_run_program().run_log_vec::<&'static str, _, CNilBrand>();
	assert_eq!(vector.extract(), (7, vec!["first", "second"]));

	let monoid: RcRun<CNilBrand, CNilBrand, (i32, String)> =
		rc_run_program().run_log_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
	assert_eq!(monoid.extract(), (7, "firstsecond".to_string()));
}

#[test]
fn arc_run_log_helpers_preserve_order() {
	let vector: ArcRun<CNilBrand, CNilBrand, (i32, Vec<&'static str>)> =
		arc_run_program().run_log_vec::<&'static str, _, CNilBrand>();
	assert_eq!(vector.extract(), (7, vec!["first", "second"]));

	let monoid: ArcRun<CNilBrand, CNilBrand, (i32, String)> =
		arc_run_program().run_log_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
	assert_eq!(monoid.extract(), (7, "firstsecond".to_string()));
}

#[test]
fn run_explicit_log_helpers_preserve_order() {
	let vector: RunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<&'static str>)> =
		run_explicit_program().run_log_vec::<&'static str, _, CNilBrand>();
	assert_eq!(vector.extract(), (7, vec!["first", "second"]));

	let monoid: RunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		run_explicit_program().run_log_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
	assert_eq!(monoid.extract(), (7, "firstsecond".to_string()));
}

#[test]
fn rc_run_explicit_log_helpers_preserve_order() {
	let vector: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<&'static str>)> =
		rc_run_explicit_program().run_log_vec::<&'static str, _, CNilBrand>();
	assert_eq!(vector.extract(), (7, vec!["first", "second"]));

	let monoid: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		rc_run_explicit_program()
			.run_log_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
	assert_eq!(monoid.extract(), (7, "firstsecond".to_string()));
}

#[test]
fn arc_run_explicit_log_helpers_preserve_order() {
	let vector: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<&'static str>)> =
		arc_run_explicit_program().run_log_vec::<&'static str, _, CNilBrand>();
	assert_eq!(vector.extract(), (7, vec!["first", "second"]));

	let monoid: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		arc_run_explicit_program()
			.run_log_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
	assert_eq!(monoid.extract(), (7, "firstsecond".to_string()));
}

#[test]
fn log_output_and_writer_keep_distinct_row_identities() {
	type RunOutputRow = CoproductBrand<CoyonedaBrand<OutputBrand<&'static str>>, CNilBrand>;
	type RunWriterRow = CoproductBrand<CoyonedaBrand<WriterBrand<&'static str>>, CNilBrand>;

	let _log: Run<RunLogRow, CNilBrand, ()> =
		Run::<RunLogRow, CNilBrand, ()>::log::<&'static str, _>("message");
	let _output: Run<RunOutputRow, CNilBrand, ()> =
		Run::<RunOutputRow, CNilBrand, ()>::output::<&'static str, _>("message");
	let _writer: Run<RunWriterRow, CNilBrand, ()> =
		Run::<RunWriterRow, CNilBrand, ()>::tell::<&'static str, _>("message");
}
