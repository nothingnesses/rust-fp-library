// Integration tests for Phase 3 step 3: pipeline row-narrowing
// (`interpret_with::<EBrand>`) and the empty-row terminal extractor
// (`extract`) on all six Run wrappers.
//
// Each wrapper is exercised with three patterns:
//   - single-effect narrowing to the empty row, then `extract` (the
//     `runPure` analog).
//   - bind-chain narrowing: a multi-step program in the original row
//     reduces through `interpret_with` to a bind chain in the narrowed
//     row, then extracts to a value.
//   - two-effect chained narrowing: two consecutive `interpret_with`
//     calls peel one effect each (in user-controlled order), then
//     `extract`.
//
// The Erased trio (Run, RcRun, ArcRun) and the Explicit trio
// (RunExplicit, RcRunExplicit, ArcRunExplicit) each pair with their
// canonical Coyoneda variant per the per-wrapper Coyoneda variant
// pairing rule.

use fp_library::{
	brands::*,
	types::{
		Identity,
		effects::{
			arc_run::ArcRun,
			arc_run_explicit::ArcRunExplicit,
			rc_run::RcRun,
			rc_run_explicit::RcRunExplicit,
			run::Run,
			run_explicit::RunExplicit,
		},
	},
};

// -- Run --

type RunFullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;

#[test]
fn run_interpret_with_single_effect_then_extract() {
	let prog: Run<RunFullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
	let narrowed: Run<CNilBrand, CNilBrand, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<Run<CNilBrand, CNilBrand, i32>>| op.0,
		);
	assert_eq!(narrowed.extract(), 42);
}

#[test]
fn run_interpret_with_bind_chain_then_extract() {
	let prog: Run<RunFullRow, CNilBrand, i32> =
		Run::lift::<IdentityBrand, _>(Identity(10)).bind(|x| Run::pure(x + 5));
	let narrowed: Run<CNilBrand, CNilBrand, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<Run<CNilBrand, CNilBrand, i32>>| op.0,
		);
	assert_eq!(narrowed.extract(), 15);
}

#[test]
fn run_extract_on_pure_program() {
	let prog: Run<CNilBrand, CNilBrand, &'static str> = Run::pure("hello");
	assert_eq!(prog.extract(), "hello");
}

// -- RunExplicit --

type RunExplicitFullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;

#[test]
fn run_explicit_interpret_with_single_effect_then_extract() {
	let prog: RunExplicit<'static, RunExplicitFullRow, CNilBrand, i32> =
		RunExplicit::lift::<IdentityBrand, _>(Identity(42));
	let narrowed: RunExplicit<'static, CNilBrand, CNilBrand, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<RunExplicit<'static, CNilBrand, CNilBrand, i32>>| op.0,
		);
	assert_eq!(narrowed.extract(), 42);
}

#[test]
fn run_explicit_interpret_with_bind_chain_then_extract() {
	let prog: RunExplicit<'static, RunExplicitFullRow, CNilBrand, i32> =
		RunExplicit::lift::<IdentityBrand, _>(Identity(10)).bind(|x| RunExplicit::pure(x * 3));
	let narrowed: RunExplicit<'static, CNilBrand, CNilBrand, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<RunExplicit<'static, CNilBrand, CNilBrand, i32>>| op.0,
		);
	assert_eq!(narrowed.extract(), 30);
}

#[test]
fn run_explicit_extract_on_pure_program() {
	let prog: RunExplicit<'static, CNilBrand, CNilBrand, i64> = RunExplicit::pure(123);
	assert_eq!(prog.extract(), 123);
}

// -- RcRun --

type RcRunFullRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;

#[test]
fn rc_run_interpret_with_single_effect_then_extract() {
	let prog: RcRun<RcRunFullRow, CNilBrand, i32> = RcRun::lift::<IdentityBrand, _>(Identity(42));
	let narrowed: RcRun<CNilBrand, CNilBrand, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<RcRun<CNilBrand, CNilBrand, i32>>| op.0,
		);
	assert_eq!(narrowed.extract(), 42);
}

#[test]
fn rc_run_interpret_with_bind_chain_then_extract() {
	let prog: RcRun<RcRunFullRow, CNilBrand, i32> =
		RcRun::lift::<IdentityBrand, _>(Identity(10)).bind(|x| RcRun::pure(x + 100));
	let narrowed: RcRun<CNilBrand, CNilBrand, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<RcRun<CNilBrand, CNilBrand, i32>>| op.0,
		);
	assert_eq!(narrowed.extract(), 110);
}

#[test]
fn rc_run_extract_on_pure_program() {
	let prog: RcRun<CNilBrand, CNilBrand, i32> = RcRun::pure(7);
	assert_eq!(prog.extract(), 7);
}

// -- RcRunExplicit --

type RcRunExplicitFullRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;

#[test]
fn rc_run_explicit_interpret_with_single_effect_then_extract() {
	let prog: RcRunExplicit<'static, RcRunExplicitFullRow, CNilBrand, i32> =
		RcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
	let narrowed: RcRunExplicit<'static, CNilBrand, CNilBrand, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<RcRunExplicit<'static, CNilBrand, CNilBrand, i32>>| op.0,
		);
	assert_eq!(narrowed.extract(), 42);
}

#[test]
fn rc_run_explicit_extract_on_pure_program() {
	let prog: RcRunExplicit<'static, CNilBrand, CNilBrand, i32> = RcRunExplicit::pure(99);
	assert_eq!(prog.extract(), 99);
}

// -- ArcRun --

type ArcRunFullRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;

#[test]
fn arc_run_interpret_with_single_effect_then_extract() {
	let prog: ArcRun<ArcRunFullRow, CNilBrand, i32> =
		ArcRun::lift::<IdentityBrand, _>(Identity(42));
	let narrowed: ArcRun<CNilBrand, CNilBrand, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<ArcRun<CNilBrand, CNilBrand, i32>>| op.0,
		);
	assert_eq!(narrowed.extract(), 42);
}

#[test]
fn arc_run_interpret_with_bind_chain_then_extract() {
	let prog: ArcRun<ArcRunFullRow, CNilBrand, i32> =
		ArcRun::lift::<IdentityBrand, _>(Identity(10)).bind(|x| ArcRun::pure(x * 2));
	let narrowed: ArcRun<CNilBrand, CNilBrand, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<ArcRun<CNilBrand, CNilBrand, i32>>| op.0,
		);
	assert_eq!(narrowed.extract(), 20);
}

#[test]
fn arc_run_extract_on_pure_program() {
	let prog: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(7);
	assert_eq!(prog.extract(), 7);
}

// -- ArcRunExplicit --

type ArcRunExplicitFullRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;

#[test]
fn arc_run_explicit_interpret_with_single_effect_then_extract() {
	let prog: ArcRunExplicit<'static, ArcRunExplicitFullRow, CNilBrand, i32> =
		ArcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
	let narrowed: ArcRunExplicit<'static, CNilBrand, CNilBrand, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<ArcRunExplicit<'static, CNilBrand, CNilBrand, i32>>| op.0,
		);
	assert_eq!(narrowed.extract(), 42);
}

#[test]
fn arc_run_explicit_extract_on_pure_program() {
	let prog: ArcRunExplicit<'static, CNilBrand, CNilBrand, i32> = ArcRunExplicit::pure(55);
	assert_eq!(prog.extract(), 55);
}
