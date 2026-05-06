// Integration tests for the substrate-level `interpose<EBrand, Idx>`
// primitive across the Run-wrapper family. Each wrapper's section walks
// the program tree, finds dispatches against EBrand, applies the
// user-supplied replacement, and re-emits in the same row (no row
// narrowing).
//
// Coverage (parallel across wrappers):
//   T1: single-effect row, no-op replacement (identity at the matched
//       arm); program is unchanged after interpose.
//   T2: single-effect row, constant replacement (substitute every
//       matched dispatch with a fresh program); demonstrates the
//       matched arm fires.
//   T3: two-effect row, replacement on the head effect only; the tail
//       effect's dispatches walk through unchanged via the embed path.
//   T4: two-effect row, replacement on the tail effect (deep position
//       at There<Here>); exercises the Member<EBrand, There<Here>>
//       projection plus the embed-back into the original row.

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
			except::Except,
			rc_run::RcRun,
			run::Run,
			run_explicit::RunExplicit,
		},
	},
};

// -- RcRun --

// Single-effect row used in T1, T2.
type RcSingleRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type RcSingleProg = RcRun<RcSingleRow, CNilBrand, i32>;

// Single-effect row remainder (after Identity is removed).
type RcSingleRowMinus = CNilBrand;

// Two-effect row used in T3, T4. Row variants follow the handlers!
// macro's lexical-sort canonical ordering: ExceptBrand < IdentityBrand.
type RcDualRow = CoproductBrand<
	RcCoyonedaBrand<ExceptBrand<String>>,
	CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>,
>;
type RcDualProg = RcRun<RcDualRow, CNilBrand, i32>;

// Dual-row remainders for each interpose target.
type RcDualRowMinusExcept = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type RcDualRowMinusIdentity = CoproductBrand<RcCoyonedaBrand<ExceptBrand<String>>, CNilBrand>;

#[test]
fn rc_run_t1_single_effect_no_op_interpose() {
	let prog: RcSingleProg = RcRun::lift::<IdentityBrand, _>(Identity(7));
	// No-op: replacement returns the matched-effect's continuation as
	// the new program (the same shape would result from no interpose
	// at all).
	let interposed =
		prog.interpose::<IdentityBrand, _, RcSingleRowMinus, _>(|op: Identity<RcSingleProg>| op.0);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<RcSingleProg>| op.0,
	});
	assert_eq!(result, 7);
}

#[test]
fn rc_run_t2_single_effect_constant_replacement() {
	let prog: RcSingleProg = RcRun::lift::<IdentityBrand, _>(Identity(7));
	let interposed =
		prog.interpose::<IdentityBrand, _, RcSingleRowMinus, _>(|_op: Identity<RcSingleProg>| {
			RcRun::pure(99)
		});
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<RcSingleProg>| op.0,
	});
	assert_eq!(result, 99);
}

#[test]
fn rc_run_t3_dual_row_unmatched_walks_through_embed_path() {
	// Program emits only Throw (tail-position effect, There<Here>).
	// interpose target is Identity (head-position, Here); the matched
	// arm never fires because the program has no Identity dispatch.
	// Every dispatch the walker visits is unmatched, so the embed-back
	// path runs unconditionally: project Identity from the layer fails,
	// the layer's RMinusE remainder is recursively interposed, and the
	// embed step puts the rebuilt RMinusE-typed value back into the
	// original DualRow shape so the substrate can rewrap as a DualProg.
	let prog: RcDualProg = RcRun::throw::<String, _>("from_t3".to_string());
	let interposed = prog.interpose::<IdentityBrand, _, RcDualRowMinusIdentity, _>(
		|_op: Identity<RcDualProg>| RcRun::pure(0),
	);
	// Interpose's embed-back path preserves the Throw dispatch in the
	// rebuilt program; interpret then fires the Throw handler.
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<RcDualProg>| op.0,
		ExceptBrand<String>: |op: Except<'_, String, RcDualProg>| match op {
			Except::Throw(_, _) => RcRun::pure(42),
		},
	});
	assert_eq!(result, 42);
}

#[test]
fn rc_run_t4_dual_row_unmatched_at_head_walks_through_embed_path() {
	// Mirror of T3 with the target-position swapped: program emits only
	// Identity (head-position), interpose target is Except (tail-position,
	// There<Here>). Every dispatch is unmatched; the embed-back path
	// rebuilds the Identity dispatch in the original DualRow shape so
	// interpret dispatches it normally.
	let prog: RcDualProg = RcRun::lift::<IdentityBrand, _>(Identity(99));
	let interposed = prog.interpose::<ExceptBrand<String>, _, RcDualRowMinusExcept, _>(
		|_op: Except<'_, String, RcDualProg>| RcRun::pure(0),
	);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<RcDualProg>| op.0,
		ExceptBrand<String>: |op: Except<'_, String, RcDualProg>| match op {
			Except::Throw(_, _) => RcRun::pure(0),
		},
	});
	assert_eq!(result, 99);
}

// -- Run --

// Single-effect row used in T1, T2 for the Erased Run wrapper.
type RunSingleRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type RunSingleProg = Run<RunSingleRow, CNilBrand, i32>;

// Single-effect row remainder (after Identity is removed).
type RunSingleRowMinus = CNilBrand;

// Two-effect row used in T3, T4. Row variants follow the handlers!
// macro's lexical-sort canonical ordering: ExceptBrand < IdentityBrand.
type RunDualRow = CoproductBrand<
	CoyonedaBrand<ExceptBrand<String>>,
	CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>,
>;
type RunDualProg = Run<RunDualRow, CNilBrand, i32>;

// Dual-row remainders for each interpose target.
type RunDualRowMinusExcept = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type RunDualRowMinusIdentity = CoproductBrand<CoyonedaBrand<ExceptBrand<String>>, CNilBrand>;

#[test]
fn run_t1_single_effect_no_op_interpose() {
	let prog: RunSingleProg = Run::lift::<IdentityBrand, _>(Identity(7));
	let interposed = prog
		.interpose::<IdentityBrand, _, RunSingleRowMinus, _>(|op: Identity<RunSingleProg>| op.0);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<RunSingleProg>| op.0,
	});
	assert_eq!(result, 7);
}

#[test]
fn run_t2_single_effect_constant_replacement() {
	let prog: RunSingleProg = Run::lift::<IdentityBrand, _>(Identity(7));
	let interposed =
		prog.interpose::<IdentityBrand, _, RunSingleRowMinus, _>(|_op: Identity<RunSingleProg>| {
			Run::pure(99)
		});
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<RunSingleProg>| op.0,
	});
	assert_eq!(result, 99);
}

#[test]
fn run_t3_dual_row_unmatched_walks_through_embed_path() {
	let prog: RunDualProg = Run::throw::<String, _>("from_t3".to_string());
	let interposed = prog.interpose::<IdentityBrand, _, RunDualRowMinusIdentity, _>(
		|_op: Identity<RunDualProg>| Run::pure(0),
	);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<RunDualProg>| op.0,
		ExceptBrand<String>: |op: Except<'_, String, RunDualProg>| match op {
			Except::Throw(_, _) => Run::pure(42),
		},
	});
	assert_eq!(result, 42);
}

#[test]
fn run_t4_dual_row_unmatched_at_head_walks_through_embed_path() {
	let prog: RunDualProg = Run::lift::<IdentityBrand, _>(Identity(99));
	let interposed = prog.interpose::<ExceptBrand<String>, _, RunDualRowMinusExcept, _>(
		|_op: Except<'_, String, RunDualProg>| Run::pure(0),
	);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<RunDualProg>| op.0,
		ExceptBrand<String>: |op: Except<'_, String, RunDualProg>| match op {
			Except::Throw(_, _) => Run::pure(0),
		},
	});
	assert_eq!(result, 99);
}

// -- ArcRun --

// Single-effect row used in T1, T2 for the thread-safe ArcRun wrapper.
type ArcSingleRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type ArcSingleProg = ArcRun<ArcSingleRow, CNilBrand, i32>;

// Single-effect row remainder (after Identity is removed).
type ArcSingleRowMinus = CNilBrand;

// Two-effect row used in T3, T4. Row variants follow the handlers!
// macro's lexical-sort canonical ordering: ExceptBrand < IdentityBrand.
type ArcDualRow = CoproductBrand<
	ArcCoyonedaBrand<ExceptBrand<String>>,
	CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>,
>;
type ArcDualProg = ArcRun<ArcDualRow, CNilBrand, i32>;

// Dual-row remainders for each interpose target.
type ArcDualRowMinusExcept = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type ArcDualRowMinusIdentity = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<String>>, CNilBrand>;

#[test]
fn arc_run_t1_single_effect_no_op_interpose() {
	let prog: ArcSingleProg = ArcRun::lift::<IdentityBrand, _>(Identity(7));
	let interposed = prog
		.interpose::<IdentityBrand, _, ArcSingleRowMinus, _>(|op: Identity<ArcSingleProg>| op.0);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<ArcSingleProg>| op.0,
	});
	assert_eq!(result, 7);
}

#[test]
fn arc_run_t2_single_effect_constant_replacement() {
	let prog: ArcSingleProg = ArcRun::lift::<IdentityBrand, _>(Identity(7));
	let interposed =
		prog.interpose::<IdentityBrand, _, ArcSingleRowMinus, _>(|_op: Identity<ArcSingleProg>| {
			ArcRun::pure(99)
		});
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<ArcSingleProg>| op.0,
	});
	assert_eq!(result, 99);
}

#[test]
fn arc_run_t3_dual_row_unmatched_walks_through_embed_path() {
	let prog: ArcDualProg = ArcRun::throw::<String, _>("from_t3".to_string());
	let interposed = prog.interpose::<IdentityBrand, _, ArcDualRowMinusIdentity, _>(
		|_op: Identity<ArcDualProg>| ArcRun::pure(0),
	);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<ArcDualProg>| op.0,
		ExceptBrand<String>: |op: Except<'_, String, ArcDualProg>| match op {
			Except::Throw(_, _) => ArcRun::pure(42),
		},
	});
	assert_eq!(result, 42);
}

#[test]
fn arc_run_t4_dual_row_unmatched_at_head_walks_through_embed_path() {
	let prog: ArcDualProg = ArcRun::lift::<IdentityBrand, _>(Identity(99));
	let interposed = prog.interpose::<ExceptBrand<String>, _, ArcDualRowMinusExcept, _>(
		|_op: Except<'_, String, ArcDualProg>| ArcRun::pure(0),
	);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<ArcDualProg>| op.0,
		ExceptBrand<String>: |op: Except<'_, String, ArcDualProg>| match op {
			Except::Throw(_, _) => ArcRun::pure(0),
		},
	});
	assert_eq!(result, 99);
}

// -- RunExplicit --

// Single-effect row used in T1, T2 for the explicit-lifetime Run wrapper.
type RxSingleRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type RxSingleProg = RunExplicit<'static, RxSingleRow, CNilBrand, i32>;

// Single-effect row remainder (after Identity is removed).
type RxSingleRowMinus = CNilBrand;

// Two-effect row used in T3, T4. Row variants follow the handlers!
// macro's lexical-sort canonical ordering: ExceptBrand < IdentityBrand.
type RxDualRow = CoproductBrand<
	CoyonedaBrand<ExceptBrand<String>>,
	CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>,
>;
type RxDualProg = RunExplicit<'static, RxDualRow, CNilBrand, i32>;

// Dual-row remainders for each interpose target.
type RxDualRowMinusExcept = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type RxDualRowMinusIdentity = CoproductBrand<CoyonedaBrand<ExceptBrand<String>>, CNilBrand>;

#[test]
fn run_explicit_t1_single_effect_no_op_interpose() {
	let prog: RxSingleProg = RunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let interposed =
		prog.interpose::<IdentityBrand, _, RxSingleRowMinus, _>(|op: Identity<RxSingleProg>| op.0);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<RxSingleProg>| op.0,
	});
	assert_eq!(result, 7);
}

#[test]
fn run_explicit_t2_single_effect_constant_replacement() {
	let prog: RxSingleProg = RunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let interposed =
		prog.interpose::<IdentityBrand, _, RxSingleRowMinus, _>(|_op: Identity<RxSingleProg>| {
			RunExplicit::pure(99)
		});
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<RxSingleProg>| op.0,
	});
	assert_eq!(result, 99);
}

#[test]
fn run_explicit_t3_dual_row_unmatched_walks_through_embed_path() {
	let prog: RxDualProg = RunExplicit::throw::<String, _>("from_t3".to_string());
	let interposed = prog.interpose::<IdentityBrand, _, RxDualRowMinusIdentity, _>(
		|_op: Identity<RxDualProg>| RunExplicit::pure(0),
	);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<RxDualProg>| op.0,
		ExceptBrand<String>: |op: Except<'_, String, RxDualProg>| match op {
			Except::Throw(_, _) => RunExplicit::pure(42),
		},
	});
	assert_eq!(result, 42);
}

#[test]
fn run_explicit_t4_dual_row_unmatched_at_head_walks_through_embed_path() {
	let prog: RxDualProg = RunExplicit::lift::<IdentityBrand, _>(Identity(99));
	let interposed = prog.interpose::<ExceptBrand<String>, _, RxDualRowMinusExcept, _>(
		|_op: Except<'_, String, RxDualProg>| RunExplicit::pure(0),
	);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<RxDualProg>| op.0,
		ExceptBrand<String>: |op: Except<'_, String, RxDualProg>| match op {
			Except::Throw(_, _) => RunExplicit::pure(0),
		},
	});
	assert_eq!(result, 99);
}
