//! Integration tests for the named Except helpers on the default `Run` wrapper.
//!
//! These tests exercise Rust-shaped conversions around the existing Except
//! substrate: `Result` values can be rethrown, `Option` values can be converted
//! into thrown errors, and `run_except` turns the selected effect into a
//! `Result` without introducing new core machinery.

use fp_library::{
	brands::*,
	types::effects::run::Run,
};

type StrExceptRow = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type UnitExceptRow = CoproductBrand<CoyonedaBrand<ExceptBrand<()>>, CNilBrand>;

#[test]
fn run_fail_returns_unit_error() {
	let program: Run<UnitExceptRow, CNilBrand, i32> = Run::fail();
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
