// Integration tests for the substrate-level
// `interpret_with_either<EBrand, Idx, RMinusE>(self, fo_handlers) -> Result<A, EBrand::Op>`
// primitive across the Run-wrapper family. Walks the program tree,
// dispatches non-matched first-order effects through `fo_handlers`,
// and short-circuits on the matched effect (`EBrand`), returning
// the matched effect's lowered payload.
//
// Coverage (parallel across wrappers):
//   T1: Pure program returns Ok(value) (no matched-effect dispatch
//       fires). Confirms the Pure arm wraps the result in Ok.
//   T2: Single matched-effect dispatch short-circuits to Err(op)
//       carrying the lowered payload. Confirms the matched arm
//       short-circuits without invoking any handler.
//   T3: FO-then-matched chain dispatches the FO step before
//       short-circuiting. Validates the FO-dispatch arm runs
//       before the matched arm fires.
//   T4: FO-only program runs to completion via the FO handler and
//       returns Ok with the final value. Validates the happy path
//       when the matched effect never fires.

use fp_library::{
	brands::{
		ArcCoyonedaBrand,
		CNilBrand,
		CoproductBrand,
		CoyonedaBrand,
		ExceptBrand,
		IdentityBrand,
		RcCoyonedaBrand,
	},
	handlers,
	types::{
		Identity,
		effects::{
			arc_run::ArcRun,
			arc_run_explicit::ArcRunExplicit,
			except::Except,
			rc_run::RcRun,
			rc_run_explicit::RcRunExplicit,
			run::Run,
			run_explicit::RunExplicit,
		},
	},
};

// -- RcRun --
//
// Two-effect row: Identity is FO-dispatched normally; Except<String>
// is matched and short-circuited. Row variants follow the handlers!
// macro's lexical-sort canonical ordering: ExceptBrand < IdentityBrand.

type RcRow = CoproductBrand<
	RcCoyonedaBrand<ExceptBrand<String>>,
	CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>,
>;
type RcRowMinusExcept = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type RcProg = RcRun<RcRow, CNilBrand, i32>;

#[test]
fn rc_run_t1_pure_program_returns_ok_with_value() {
	let prog: RcProg = RcRun::pure(42);
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RcRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RcProg>| op.0,
		});
	assert!(matches!(result, Ok(42)));
}

#[test]
fn rc_run_t2_single_throw_short_circuits_to_err() {
	let prog: RcProg = RcRun::throw::<String, _>("oops".to_string());
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RcRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RcProg>| op.0,
		});
	assert!(matches!(&result, Err(Except::Throw(e, _)) if e == "oops"));
}

#[test]
fn rc_run_t3_fo_then_matched_short_circuits_after_dispatch() {
	let identity_step: RcProg = RcRun::lift::<IdentityBrand, _>(Identity(7));
	let prog: RcProg =
		identity_step.bind(|_v: i32| RcRun::throw::<String, _>("after-identity".to_string()));
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RcRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RcProg>| op.0,
		});
	assert!(matches!(&result, Err(Except::Throw(e, _)) if e == "after-identity"));
}

#[test]
fn rc_run_t4_fo_only_program_runs_to_completion() {
	let step1: RcProg = RcRun::lift::<IdentityBrand, _>(Identity(10));
	let step2 = step1.bind(|v: i32| RcRun::lift::<IdentityBrand, _>(Identity(v + 5)));
	let prog: RcProg = step2.bind(|v: i32| RcRun::pure(v * 2));
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RcRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RcProg>| op.0,
		});
	assert!(matches!(result, Ok(30)));
}

// -- Run --

type RunRow = CoproductBrand<
	CoyonedaBrand<ExceptBrand<String>>,
	CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>,
>;
type RunRowMinusExcept = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type RunProg = Run<RunRow, CNilBrand, i32>;

#[test]
fn run_t1_pure_program_returns_ok_with_value() {
	let prog: RunProg = Run::pure(42);
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RunRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RunProg>| op.0,
		});
	assert!(matches!(result, Ok(42)));
}

#[test]
fn run_t2_single_throw_short_circuits_to_err() {
	let prog: RunProg = Run::throw::<String, _>("oops".to_string());
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RunRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RunProg>| op.0,
		});
	assert!(matches!(&result, Err(Except::Throw(e, _)) if e == "oops"));
}

#[test]
fn run_t3_fo_then_matched_short_circuits_after_dispatch() {
	let identity_step: RunProg = Run::lift::<IdentityBrand, _>(Identity(7));
	let prog: RunProg =
		identity_step.bind(|_v: i32| Run::throw::<String, _>("after-identity".to_string()));
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RunRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RunProg>| op.0,
		});
	assert!(matches!(&result, Err(Except::Throw(e, _)) if e == "after-identity"));
}

#[test]
fn run_t4_fo_only_program_runs_to_completion() {
	let step1: RunProg = Run::lift::<IdentityBrand, _>(Identity(10));
	let step2 = step1.bind(|v: i32| Run::lift::<IdentityBrand, _>(Identity(v + 5)));
	let prog: RunProg = step2.bind(|v: i32| Run::pure(v * 2));
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RunRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RunProg>| op.0,
		});
	assert!(matches!(result, Ok(30)));
}

// -- ArcRun --

type ArcRow = CoproductBrand<
	ArcCoyonedaBrand<ExceptBrand<String>>,
	CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>,
>;
type ArcRowMinusExcept = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type ArcProg = ArcRun<ArcRow, CNilBrand, i32>;

#[test]
fn arc_run_t1_pure_program_returns_ok_with_value() {
	let prog: ArcProg = ArcRun::pure(42);
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, ArcRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<ArcProg>| op.0,
		});
	assert!(matches!(result, Ok(42)));
}

#[test]
fn arc_run_t2_single_throw_short_circuits_to_err() {
	let prog: ArcProg = ArcRun::throw::<String, _>("oops".to_string());
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, ArcRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<ArcProg>| op.0,
		});
	assert!(matches!(&result, Err(Except::Throw(e, _)) if e == "oops"));
}

#[test]
fn arc_run_t3_fo_then_matched_short_circuits_after_dispatch() {
	let identity_step: ArcProg = ArcRun::lift::<IdentityBrand, _>(Identity(7));
	let prog: ArcProg =
		identity_step.bind(|_v: i32| ArcRun::throw::<String, _>("after-identity".to_string()));
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, ArcRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<ArcProg>| op.0,
		});
	assert!(matches!(&result, Err(Except::Throw(e, _)) if e == "after-identity"));
}

#[test]
fn arc_run_t4_fo_only_program_runs_to_completion() {
	let step1: ArcProg = ArcRun::lift::<IdentityBrand, _>(Identity(10));
	let step2 = step1.bind(|v: i32| ArcRun::lift::<IdentityBrand, _>(Identity(v + 5)));
	let prog: ArcProg = step2.bind(|v: i32| ArcRun::pure(v * 2));
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, ArcRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<ArcProg>| op.0,
		});
	assert!(matches!(result, Ok(30)));
}

// -- RunExplicit --

type RxRow = CoproductBrand<
	CoyonedaBrand<ExceptBrand<String>>,
	CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>,
>;
type RxRowMinusExcept = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type RxProg = RunExplicit<'static, RxRow, CNilBrand, i32>;

#[test]
fn run_explicit_t1_pure_program_returns_ok_with_value() {
	let prog: RxProg = RunExplicit::pure(42);
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RxRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RxProg>| op.0,
		});
	assert!(matches!(result, Ok(42)));
}

#[test]
fn run_explicit_t2_single_throw_short_circuits_to_err() {
	let prog: RxProg = RunExplicit::throw::<String, _>("oops".to_string());
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RxRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RxProg>| op.0,
		});
	assert!(matches!(&result, Err(Except::Throw(e, _)) if e == "oops"));
}

#[test]
fn run_explicit_t3_fo_then_matched_short_circuits_after_dispatch() {
	let identity_step: RxProg = RunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let prog: RxProg =
		identity_step.bind(|_v: i32| RunExplicit::throw::<String, _>("after-identity".to_string()));
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RxRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RxProg>| op.0,
		});
	assert!(matches!(&result, Err(Except::Throw(e, _)) if e == "after-identity"));
}

#[test]
fn run_explicit_t4_fo_only_program_runs_to_completion() {
	let step1: RxProg = RunExplicit::lift::<IdentityBrand, _>(Identity(10));
	let step2 = step1.bind(|v: i32| RunExplicit::lift::<IdentityBrand, _>(Identity(v + 5)));
	let prog: RxProg = step2.bind(|v: i32| RunExplicit::pure(v * 2));
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RxRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RxProg>| op.0,
		});
	assert!(matches!(result, Ok(30)));
}

// -- RcRunExplicit --

type RcxRow = CoproductBrand<
	RcCoyonedaBrand<ExceptBrand<String>>,
	CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>,
>;
type RcxRowMinusExcept = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type RcxProg = RcRunExplicit<'static, RcxRow, CNilBrand, i32>;

#[test]
fn rc_run_explicit_t1_pure_program_returns_ok_with_value() {
	let prog: RcxProg = RcRunExplicit::pure(42);
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RcxRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RcxProg>| op.0,
		});
	assert!(matches!(result, Ok(42)));
}

#[test]
fn rc_run_explicit_t2_single_throw_short_circuits_to_err() {
	let prog: RcxProg = RcRunExplicit::throw::<String, _>("oops".to_string());
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RcxRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RcxProg>| op.0,
		});
	assert!(matches!(&result, Err(Except::Throw(e, _)) if e == "oops"));
}

#[test]
fn rc_run_explicit_t3_fo_then_matched_short_circuits_after_dispatch() {
	let identity_step: RcxProg = RcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let prog: RcxProg = identity_step
		.bind(|_v: i32| RcRunExplicit::throw::<String, _>("after-identity".to_string()));
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RcxRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RcxProg>| op.0,
		});
	assert!(matches!(&result, Err(Except::Throw(e, _)) if e == "after-identity"));
}

#[test]
fn rc_run_explicit_t4_fo_only_program_runs_to_completion() {
	let step1: RcxProg = RcRunExplicit::lift::<IdentityBrand, _>(Identity(10));
	let step2 = step1.bind(|v: i32| RcRunExplicit::lift::<IdentityBrand, _>(Identity(v + 5)));
	let prog: RcxProg = step2.bind(|v: i32| RcRunExplicit::pure(v * 2));
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, RcxRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<RcxProg>| op.0,
		});
	assert!(matches!(result, Ok(30)));
}

// -- ArcRunExplicit --

type AcxRow = CoproductBrand<
	ArcCoyonedaBrand<ExceptBrand<String>>,
	CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>,
>;
type AcxRowMinusExcept = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type AcxProg = ArcRunExplicit<'static, AcxRow, CNilBrand, i32>;

#[test]
fn arc_run_explicit_t1_pure_program_returns_ok_with_value() {
	let prog: AcxProg = ArcRunExplicit::pure(42);
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, AcxRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<AcxProg>| op.0,
		});
	assert!(matches!(result, Ok(42)));
}

#[test]
fn arc_run_explicit_t2_single_throw_short_circuits_to_err() {
	let prog: AcxProg = ArcRunExplicit::throw::<String, _>("oops".to_string());
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, AcxRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<AcxProg>| op.0,
		});
	assert!(matches!(&result, Err(Except::Throw(e, _)) if e == "oops"));
}

#[test]
fn arc_run_explicit_t3_fo_then_matched_short_circuits_after_dispatch() {
	let identity_step: AcxProg = ArcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let prog: AcxProg = identity_step
		.bind(|_v: i32| ArcRunExplicit::throw::<String, _>("after-identity".to_string()));
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, AcxRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<AcxProg>| op.0,
		});
	assert!(matches!(&result, Err(Except::Throw(e, _)) if e == "after-identity"));
}

#[test]
fn arc_run_explicit_t4_fo_only_program_runs_to_completion() {
	let step1: AcxProg = ArcRunExplicit::lift::<IdentityBrand, _>(Identity(10));
	let step2 = step1.bind(|v: i32| ArcRunExplicit::lift::<IdentityBrand, _>(Identity(v + 5)));
	let prog: AcxProg = step2.bind(|v: i32| ArcRunExplicit::pure(v * 2));
	let result =
		prog.interpret_with_either::<ExceptBrand<String>, _, AcxRowMinusExcept>(handlers! {
			IdentityBrand: |op: Identity<AcxProg>| op.0,
		});
	assert!(matches!(result, Ok(30)));
}
