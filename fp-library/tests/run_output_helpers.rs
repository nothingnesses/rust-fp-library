#![cfg(feature = "effects")]

//! Integration tests for the named Output helpers.
//!
//! `output` emits one value without changing the program result. The
//! named runners must preserve emission order whether they collect
//! values directly into a vector or map each value into a monoidal
//! accumulator.

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

type RunOutputRow = CoproductBrand<CoyonedaBrand<OutputBrand<&'static str>>, CNilBrand>;
type RcRunOutputRow = CoproductBrand<RcCoyonedaBrand<OutputBrand<&'static str>>, CNilBrand>;
type ArcRunOutputRow = CoproductBrand<ArcCoyonedaBrand<OutputBrand<&'static str>>, CNilBrand>;

fn run_program() -> Run<RunOutputRow, CNilBrand, i32> {
	Run::<RunOutputRow, CNilBrand, ()>::output::<&'static str, _>("first")
		.bind(|()| Run::<RunOutputRow, CNilBrand, ()>::output::<&'static str, _>("second"))
		.bind(|()| Run::<RunOutputRow, CNilBrand, i32>::pure(7))
}

fn rc_run_program() -> RcRun<RcRunOutputRow, CNilBrand, i32> {
	RcRun::<RcRunOutputRow, CNilBrand, ()>::output::<&'static str, _>("first")
		.bind(|()| RcRun::<RcRunOutputRow, CNilBrand, ()>::output::<&'static str, _>("second"))
		.bind(|()| RcRun::<RcRunOutputRow, CNilBrand, i32>::pure(7))
}

fn arc_run_program() -> ArcRun<ArcRunOutputRow, CNilBrand, i32> {
	ArcRun::<ArcRunOutputRow, CNilBrand, ()>::output::<&'static str, _>("first")
		.bind(|()| ArcRun::<ArcRunOutputRow, CNilBrand, ()>::output::<&'static str, _>("second"))
		.bind(|()| ArcRun::<ArcRunOutputRow, CNilBrand, i32>::pure(7))
}

fn run_explicit_program() -> RunExplicit<'static, RunOutputRow, CNilBrand, i32> {
	RunExplicit::<'static, RunOutputRow, CNilBrand, ()>::output::<&'static str, _>("first")
		.bind(|()| {
			RunExplicit::<'static, RunOutputRow, CNilBrand, ()>::output::<&'static str, _>("second")
		})
		.bind(|()| RunExplicit::<'static, RunOutputRow, CNilBrand, i32>::pure(7))
}

fn rc_run_explicit_program() -> RcRunExplicit<'static, RcRunOutputRow, CNilBrand, i32> {
	RcRunExplicit::<'static, RcRunOutputRow, CNilBrand, ()>::output::<&'static str, _>("first")
		.bind(|()| {
			RcRunExplicit::<'static, RcRunOutputRow, CNilBrand, ()>::output::<&'static str, _>(
				"second",
			)
		})
		.bind(|()| RcRunExplicit::<'static, RcRunOutputRow, CNilBrand, i32>::pure(7))
}

fn arc_run_explicit_program() -> ArcRunExplicit<'static, ArcRunOutputRow, CNilBrand, i32> {
	ArcRunExplicit::<'static, ArcRunOutputRow, CNilBrand, ()>::output::<&'static str, _>("first")
		.bind(|()| {
			ArcRunExplicit::<'static, ArcRunOutputRow, CNilBrand, ()>::output::<&'static str, _>(
				"second",
			)
		})
		.bind(|()| ArcRunExplicit::<'static, ArcRunOutputRow, CNilBrand, i32>::pure(7))
}

#[test]
fn run_output_helpers_preserve_order() {
	let vector: Run<CNilBrand, CNilBrand, (i32, Vec<&'static str>)> =
		run_program().run_output_vec::<&'static str, _, CNilBrand>();
	assert_eq!(vector.extract(), (7, vec!["first", "second"]));

	let monoid: Run<CNilBrand, CNilBrand, (i32, String)> =
		run_program().run_output_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
	assert_eq!(monoid.extract(), (7, "firstsecond".to_string()));
}

#[test]
fn rc_run_output_helpers_preserve_order() {
	let vector: RcRun<CNilBrand, CNilBrand, (i32, Vec<&'static str>)> =
		rc_run_program().run_output_vec::<&'static str, _, CNilBrand>();
	assert_eq!(vector.extract(), (7, vec!["first", "second"]));

	let monoid: RcRun<CNilBrand, CNilBrand, (i32, String)> =
		rc_run_program().run_output_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
	assert_eq!(monoid.extract(), (7, "firstsecond".to_string()));
}

#[test]
fn arc_run_output_helpers_preserve_order() {
	let vector: ArcRun<CNilBrand, CNilBrand, (i32, Vec<&'static str>)> =
		arc_run_program().run_output_vec::<&'static str, _, CNilBrand>();
	assert_eq!(vector.extract(), (7, vec!["first", "second"]));

	let monoid: ArcRun<CNilBrand, CNilBrand, (i32, String)> =
		arc_run_program().run_output_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
	assert_eq!(monoid.extract(), (7, "firstsecond".to_string()));
}

#[test]
fn run_explicit_output_helpers_preserve_order() {
	let vector: RunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<&'static str>)> =
		run_explicit_program().run_output_vec::<&'static str, _, CNilBrand>();
	assert_eq!(vector.extract(), (7, vec!["first", "second"]));

	let monoid: RunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		run_explicit_program()
			.run_output_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
	assert_eq!(monoid.extract(), (7, "firstsecond".to_string()));
}

#[test]
fn rc_run_explicit_output_helpers_preserve_order() {
	let vector: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<&'static str>)> =
		rc_run_explicit_program().run_output_vec::<&'static str, _, CNilBrand>();
	assert_eq!(vector.extract(), (7, vec!["first", "second"]));

	let monoid: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		rc_run_explicit_program()
			.run_output_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
	assert_eq!(monoid.extract(), (7, "firstsecond".to_string()));
}

#[test]
fn arc_run_explicit_output_helpers_preserve_order() {
	let vector: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<&'static str>)> =
		arc_run_explicit_program().run_output_vec::<&'static str, _, CNilBrand>();
	assert_eq!(vector.extract(), (7, vec!["first", "second"]));

	let monoid: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		arc_run_explicit_program()
			.run_output_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
	assert_eq!(monoid.extract(), (7, "firstsecond".to_string()));
}
