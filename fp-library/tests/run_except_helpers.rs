#![cfg(feature = "effects")]

//! Integration tests for the named Except helpers on the default `Run` wrapper.
//!
//! These tests exercise Rust-shaped conversions around the existing Except
//! substrate: `Result` values can be rethrown, `Option` values can be converted
//! into thrown errors, and `run_except` turns the selected effect into a
//! `Result` without introducing new core machinery.

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

type StrExceptRow = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type UnitExceptRow = CoproductBrand<CoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
type ArcStrExceptRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type ArcUnitExceptRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
type RcStrExceptRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type RcUnitExceptRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;

#[test]
fn run_throw_unit_returns_unit_error() {
	let program: Run<UnitExceptRow, CNilBrand, i32> = Run::throw_unit();
	let handled: Run<CNilBrand, CNilBrand, Result<i32, ()>> =
		program.run_except::<(), _, CNilBrand>();
	assert_eq!(handled.extract(), Err(()));
}

#[test]
fn run_rethrow_preserves_ok_value() {
	let program: Run<StrExceptRow, CNilBrand, i32> = Run::rethrow::<&'static str, _>(Ok(7));
	let handled: Run<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Ok(7));
}

#[test]
fn run_rethrow_turns_err_into_except() {
	let program: Run<StrExceptRow, CNilBrand, i32> =
		Run::rethrow::<&'static str, _>(Err("missing"));
	let handled: Run<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Err("missing"));
}

#[test]
fn run_note_preserves_some_value() {
	let program: Run<StrExceptRow, CNilBrand, i32> =
		Run::note::<&'static str, _>("missing", Some(7));
	let handled: Run<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Ok(7));
}

#[test]
fn run_note_turns_none_into_except() {
	let program: Run<StrExceptRow, CNilBrand, i32> = Run::note::<&'static str, _>("missing", None);
	let handled: Run<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Err("missing"));
}

#[test]
fn run_from_option_uses_unit_error() {
	let program: Run<UnitExceptRow, CNilBrand, i32> = Run::from_option(None);
	let handled: Run<CNilBrand, CNilBrand, Result<i32, ()>> =
		program.run_except::<(), _, CNilBrand>();
	assert_eq!(handled.extract(), Err(()));
}

#[test]
fn run_explicit_throw_unit_returns_unit_error() {
	let program: RunExplicit<'static, UnitExceptRow, CNilBrand, i32> = RunExplicit::throw_unit();
	let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		program.run_except::<(), _, CNilBrand>();
	assert_eq!(handled.extract(), Err(()));
}

#[test]
fn run_explicit_rethrow_preserves_ok_value() {
	let program: RunExplicit<'static, StrExceptRow, CNilBrand, i32> =
		RunExplicit::rethrow::<&'static str, _>(Ok(7));
	let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Ok(7));
}

#[test]
fn run_explicit_rethrow_turns_err_into_except() {
	let program: RunExplicit<'static, StrExceptRow, CNilBrand, i32> =
		RunExplicit::rethrow::<&'static str, _>(Err("missing"));
	let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Err("missing"));
}

#[test]
fn run_explicit_note_preserves_some_value() {
	let program: RunExplicit<'static, StrExceptRow, CNilBrand, i32> =
		RunExplicit::note::<&'static str, _>("missing", Some(7));
	let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Ok(7));
}

#[test]
fn run_explicit_note_turns_none_into_except() {
	let program: RunExplicit<'static, StrExceptRow, CNilBrand, i32> =
		RunExplicit::note::<&'static str, _>("missing", None);
	let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Err("missing"));
}

#[test]
fn run_explicit_from_option_uses_unit_error() {
	let program: RunExplicit<'static, UnitExceptRow, CNilBrand, i32> =
		RunExplicit::from_option(None);
	let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		program.run_except::<(), _, CNilBrand>();
	assert_eq!(handled.extract(), Err(()));
}

#[test]
fn rc_run_explicit_throw_unit_returns_unit_error() {
	let program: RcRunExplicit<'static, RcUnitExceptRow, CNilBrand, i32> =
		RcRunExplicit::throw_unit();
	let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		program.run_except::<(), _, CNilBrand>();
	assert_eq!(handled.extract(), Err(()));
}

#[test]
fn rc_run_explicit_rethrow_preserves_ok_value() {
	let program: RcRunExplicit<'static, RcStrExceptRow, CNilBrand, i32> =
		RcRunExplicit::rethrow::<&'static str, _>(Ok(7));
	let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Ok(7));
}

#[test]
fn rc_run_explicit_rethrow_turns_err_into_except() {
	let program: RcRunExplicit<'static, RcStrExceptRow, CNilBrand, i32> =
		RcRunExplicit::rethrow::<&'static str, _>(Err("missing"));
	let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Err("missing"));
}

#[test]
fn rc_run_explicit_note_preserves_some_value() {
	let program: RcRunExplicit<'static, RcStrExceptRow, CNilBrand, i32> =
		RcRunExplicit::note::<&'static str, _>("missing", Some(7));
	let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Ok(7));
}

#[test]
fn rc_run_explicit_note_turns_none_into_except() {
	let program: RcRunExplicit<'static, RcStrExceptRow, CNilBrand, i32> =
		RcRunExplicit::note::<&'static str, _>("missing", None);
	let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Err("missing"));
}

#[test]
fn rc_run_explicit_from_option_uses_unit_error() {
	let program: RcRunExplicit<'static, RcUnitExceptRow, CNilBrand, i32> =
		RcRunExplicit::from_option(None);
	let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		program.run_except::<(), _, CNilBrand>();
	assert_eq!(handled.extract(), Err(()));
}

#[test]
fn arc_run_throw_unit_returns_unit_error() {
	let program: ArcRun<ArcUnitExceptRow, CNilBrand, i32> = ArcRun::throw_unit();
	let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, ()>> =
		program.run_except::<(), _, CNilBrand>();
	assert_eq!(handled.extract(), Err(()));
}

#[test]
fn arc_run_rethrow_preserves_ok_value() {
	let program: ArcRun<ArcStrExceptRow, CNilBrand, i32> =
		ArcRun::rethrow::<&'static str, _>(Ok(7));
	let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Ok(7));
}

#[test]
fn arc_run_rethrow_turns_err_into_except() {
	let program: ArcRun<ArcStrExceptRow, CNilBrand, i32> =
		ArcRun::rethrow::<&'static str, _>(Err("missing"));
	let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Err("missing"));
}

#[test]
fn arc_run_note_preserves_some_value() {
	let program: ArcRun<ArcStrExceptRow, CNilBrand, i32> =
		ArcRun::note::<&'static str, _>("missing", Some(7));
	let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Ok(7));
}

#[test]
fn arc_run_note_turns_none_into_except() {
	let program: ArcRun<ArcStrExceptRow, CNilBrand, i32> =
		ArcRun::note::<&'static str, _>("missing", None);
	let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Err("missing"));
}

#[test]
fn arc_run_from_option_uses_unit_error() {
	let program: ArcRun<ArcUnitExceptRow, CNilBrand, i32> = ArcRun::from_option(None);
	let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, ()>> =
		program.run_except::<(), _, CNilBrand>();
	assert_eq!(handled.extract(), Err(()));
}

#[test]
fn arc_run_explicit_throw_unit_returns_unit_error() {
	let program: ArcRunExplicit<'static, ArcUnitExceptRow, CNilBrand, i32> =
		ArcRunExplicit::throw_unit();
	let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		program.run_except::<(), _, CNilBrand>();
	assert_eq!(handled.extract(), Err(()));
}

#[test]
fn arc_run_explicit_rethrow_preserves_ok_value() {
	let program: ArcRunExplicit<'static, ArcStrExceptRow, CNilBrand, i32> =
		ArcRunExplicit::rethrow::<&'static str, _>(Ok(7));
	let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Ok(7));
}

#[test]
fn arc_run_explicit_rethrow_turns_err_into_except() {
	let program: ArcRunExplicit<'static, ArcStrExceptRow, CNilBrand, i32> =
		ArcRunExplicit::rethrow::<&'static str, _>(Err("missing"));
	let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Err("missing"));
}

#[test]
fn arc_run_explicit_note_preserves_some_value() {
	let program: ArcRunExplicit<'static, ArcStrExceptRow, CNilBrand, i32> =
		ArcRunExplicit::note::<&'static str, _>("missing", Some(7));
	let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Ok(7));
}

#[test]
fn arc_run_explicit_note_turns_none_into_except() {
	let program: ArcRunExplicit<'static, ArcStrExceptRow, CNilBrand, i32> =
		ArcRunExplicit::note::<&'static str, _>("missing", None);
	let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Err("missing"));
}

#[test]
fn arc_run_explicit_from_option_uses_unit_error() {
	let program: ArcRunExplicit<'static, ArcUnitExceptRow, CNilBrand, i32> =
		ArcRunExplicit::from_option(None);
	let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		program.run_except::<(), _, CNilBrand>();
	assert_eq!(handled.extract(), Err(()));
}

#[test]
fn rc_run_throw_unit_returns_unit_error() {
	let program: RcRun<RcUnitExceptRow, CNilBrand, i32> = RcRun::throw_unit();
	let handled: RcRun<CNilBrand, CNilBrand, Result<i32, ()>> =
		program.run_except::<(), _, CNilBrand>();
	assert_eq!(handled.extract(), Err(()));
}

#[test]
fn rc_run_rethrow_preserves_ok_value() {
	let program: RcRun<RcStrExceptRow, CNilBrand, i32> = RcRun::rethrow::<&'static str, _>(Ok(7));
	let handled: RcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Ok(7));
}

#[test]
fn rc_run_rethrow_turns_err_into_except() {
	let program: RcRun<RcStrExceptRow, CNilBrand, i32> =
		RcRun::rethrow::<&'static str, _>(Err("missing"));
	let handled: RcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Err("missing"));
}

#[test]
fn rc_run_note_preserves_some_value() {
	let program: RcRun<RcStrExceptRow, CNilBrand, i32> =
		RcRun::note::<&'static str, _>("missing", Some(7));
	let handled: RcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Ok(7));
}

#[test]
fn rc_run_note_turns_none_into_except() {
	let program: RcRun<RcStrExceptRow, CNilBrand, i32> =
		RcRun::note::<&'static str, _>("missing", None);
	let handled: RcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		program.run_except::<&'static str, _, CNilBrand>();
	assert_eq!(handled.extract(), Err("missing"));
}

#[test]
fn rc_run_from_option_uses_unit_error() {
	let program: RcRun<RcUnitExceptRow, CNilBrand, i32> = RcRun::from_option(None);
	let handled: RcRun<CNilBrand, CNilBrand, Result<i32, ()>> =
		program.run_except::<(), _, CNilBrand>();
	assert_eq!(handled.extract(), Err(()));
}
