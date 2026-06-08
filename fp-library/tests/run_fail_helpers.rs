#![cfg(feature = "effects")]

//! Integration tests for the named Fail helpers.
//!
//! `FailBrand` represents a fixed-message abort channel. It returns
//! `Result<A, String>` when interpreted and remains distinct from
//! `ExceptBrand<String>`, which is the typed exception channel.

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

type RunFailRow = CoproductBrand<CoyonedaBrand<FailBrand>, CNilBrand>;
type RcRunFailRow = CoproductBrand<RcCoyonedaBrand<FailBrand>, CNilBrand>;
type ArcRunFailRow = CoproductBrand<ArcCoyonedaBrand<FailBrand>, CNilBrand>;

#[test]
fn run_fail_returns_string_error() {
	let program: Run<RunFailRow, CNilBrand, i32> =
		Run::<RunFailRow, CNilBrand, ()>::fail::<_>("missing")
			.bind(|()| Run::<RunFailRow, CNilBrand, i32>::pure(99));
	let handled: Run<CNilBrand, CNilBrand, Result<i32, String>> =
		program.run_fail::<_, CNilBrand>();
	assert_eq!(handled.extract(), Err(String::from("missing")));

	let success: Run<CNilBrand, CNilBrand, Result<i32, String>> =
		Run::<RunFailRow, CNilBrand, i32>::pure(7).run_fail::<_, CNilBrand>();
	assert_eq!(success.extract(), Ok(7));
}

#[test]
fn rc_run_fail_returns_string_error() {
	let program: RcRun<RcRunFailRow, CNilBrand, i32> =
		RcRun::<RcRunFailRow, CNilBrand, ()>::fail::<_>(String::from("missing"))
			.bind(|()| RcRun::<RcRunFailRow, CNilBrand, i32>::pure(99));
	let handled: RcRun<CNilBrand, CNilBrand, Result<i32, String>> =
		program.run_fail::<_, CNilBrand>();
	assert_eq!(handled.extract(), Err(String::from("missing")));

	let success: RcRun<CNilBrand, CNilBrand, Result<i32, String>> =
		RcRun::<RcRunFailRow, CNilBrand, i32>::pure(7).run_fail::<_, CNilBrand>();
	assert_eq!(success.extract(), Ok(7));
}

#[test]
fn arc_run_fail_returns_string_error() {
	let program: ArcRun<ArcRunFailRow, CNilBrand, i32> =
		ArcRun::<ArcRunFailRow, CNilBrand, ()>::fail::<_>("missing")
			.bind(|()| ArcRun::<ArcRunFailRow, CNilBrand, i32>::pure(99));
	let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, String>> =
		program.run_fail::<_, CNilBrand>();
	assert_eq!(handled.extract(), Err(String::from("missing")));

	let success: ArcRun<CNilBrand, CNilBrand, Result<i32, String>> =
		ArcRun::<ArcRunFailRow, CNilBrand, i32>::pure(7).run_fail::<_, CNilBrand>();
	assert_eq!(success.extract(), Ok(7));
}

#[test]
fn run_explicit_fail_returns_string_error() {
	let program: RunExplicit<'static, RunFailRow, CNilBrand, i32> =
		RunExplicit::<'static, RunFailRow, CNilBrand, ()>::fail::<_>("missing")
			.bind(|()| RunExplicit::<'static, RunFailRow, CNilBrand, i32>::pure(99));
	let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, String>> =
		program.run_fail::<_, CNilBrand>();
	assert_eq!(handled.extract(), Err(String::from("missing")));

	let success: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, String>> =
		RunExplicit::<'static, RunFailRow, CNilBrand, i32>::pure(7).run_fail::<_, CNilBrand>();
	assert_eq!(success.extract(), Ok(7));
}

#[test]
fn rc_run_explicit_fail_returns_string_error() {
	let program: RcRunExplicit<'static, RcRunFailRow, CNilBrand, i32> =
		RcRunExplicit::<'static, RcRunFailRow, CNilBrand, ()>::fail::<_>("missing")
			.bind(|()| RcRunExplicit::<'static, RcRunFailRow, CNilBrand, i32>::pure(99));
	let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, String>> =
		program.run_fail::<_, CNilBrand>();
	assert_eq!(handled.extract(), Err(String::from("missing")));

	let success: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, String>> =
		RcRunExplicit::<'static, RcRunFailRow, CNilBrand, i32>::pure(7).run_fail::<_, CNilBrand>();
	assert_eq!(success.extract(), Ok(7));
}

#[test]
fn arc_run_explicit_fail_returns_string_error() {
	let program: ArcRunExplicit<'static, ArcRunFailRow, CNilBrand, i32> =
		ArcRunExplicit::<'static, ArcRunFailRow, CNilBrand, ()>::fail::<_>("missing")
			.bind(|()| ArcRunExplicit::<'static, ArcRunFailRow, CNilBrand, i32>::pure(99));
	let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, String>> =
		program.run_fail::<_, CNilBrand>();
	assert_eq!(handled.extract(), Err(String::from("missing")));

	let success: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, String>> =
		ArcRunExplicit::<'static, ArcRunFailRow, CNilBrand, i32>::pure(7)
			.run_fail::<_, CNilBrand>();
	assert_eq!(success.extract(), Ok(7));
}

#[test]
fn fail_and_string_except_keep_distinct_row_identities() {
	type RunExceptStringRow = CoproductBrand<CoyonedaBrand<ExceptBrand<String>>, CNilBrand>;

	let _fail: Run<RunFailRow, CNilBrand, i32> =
		Run::<RunFailRow, CNilBrand, i32>::fail::<_>("message");
	let _except: Run<RunExceptStringRow, CNilBrand, i32> =
		Run::<RunExceptStringRow, CNilBrand, i32>::throw::<String, _>(String::from("message"));
}
