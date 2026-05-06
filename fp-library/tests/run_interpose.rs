// Integration tests for the substrate-level `interpose<EBrand, Idx>`
// primitive on RcRun. Walks the program tree, finds dispatches against
// EBrand, applies the user-supplied replacement, and re-emits in the
// same row (no row narrowing).
//
// Coverage:
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
		CNilBrand,
		CoproductBrand,
		ExceptBrand,
		IdentityBrand,
		RcCoyonedaBrand,
	},
	handlers,
	types::{
		Identity,
		effects::{
			except::Except,
			rc_run::RcRun,
		},
	},
};

// Single-effect row used in T1, T2.
type SingleRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type SingleProg = RcRun<SingleRow, CNilBrand, i32>;

// Single-effect row remainder (after Identity is removed).
type SingleRowMinus = CNilBrand;

// Two-effect row used in T3, T4. Row variants follow the handlers!
// macro's lexical-sort canonical ordering: ExceptBrand < IdentityBrand.
type DualRow = CoproductBrand<
	RcCoyonedaBrand<ExceptBrand<String>>,
	CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>,
>;
type DualProg = RcRun<DualRow, CNilBrand, i32>;

// Dual-row remainders for each interpose target.
type DualRowMinusExcept = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type DualRowMinusIdentity = CoproductBrand<RcCoyonedaBrand<ExceptBrand<String>>, CNilBrand>;

#[test]
fn t1_single_effect_no_op_interpose() {
	let prog: SingleProg = RcRun::lift::<IdentityBrand, _>(Identity(7));
	// No-op: replacement returns the matched-effect's continuation as
	// the new program (the same shape would result from no interpose
	// at all).
	let interposed =
		prog.interpose::<IdentityBrand, _, SingleRowMinus, _>(|op: Identity<SingleProg>| op.0);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<SingleProg>| op.0,
	});
	assert_eq!(result, 7);
}

#[test]
fn t2_single_effect_constant_replacement() {
	let prog: SingleProg = RcRun::lift::<IdentityBrand, _>(Identity(7));
	let interposed =
		prog.interpose::<IdentityBrand, _, SingleRowMinus, _>(|_op: Identity<SingleProg>| {
			RcRun::pure(99)
		});
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<SingleProg>| op.0,
	});
	assert_eq!(result, 99);
}

#[test]
fn t3_dual_row_unmatched_walks_through_embed_path() {
	// Program emits only Throw (tail-position effect, There<Here>).
	// interpose target is Identity (head-position, Here); the matched
	// arm never fires because the program has no Identity dispatch.
	// Every dispatch the walker visits is unmatched, so the embed-back
	// path runs unconditionally: project Identity from the layer fails,
	// the layer's RMinusE remainder is recursively interposed, and the
	// embed step puts the rebuilt RMinusE-typed value back into the
	// original DualRow shape so the substrate can rewrap as a DualProg.
	let prog: DualProg = RcRun::throw::<String, _>("from_t3".to_string());
	let interposed =
		prog.interpose::<IdentityBrand, _, DualRowMinusIdentity, _>(|_op: Identity<DualProg>| {
			RcRun::pure(0)
		});
	// Interpose's embed-back path preserves the Throw dispatch in the
	// rebuilt program; interpret then fires the Throw handler.
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<DualProg>| op.0,
		ExceptBrand<String>: |op: Except<'_, String, DualProg>| match op {
			Except::Throw(_, _) => RcRun::pure(42),
		},
	});
	assert_eq!(result, 42);
}

#[test]
fn t4_dual_row_unmatched_at_head_walks_through_embed_path() {
	// Mirror of T3 with the target-position swapped: program emits only
	// Identity (head-position), interpose target is Except (tail-position,
	// There<Here>). Every dispatch is unmatched; the embed-back path
	// rebuilds the Identity dispatch in the original DualRow shape so
	// interpret dispatches it normally.
	let prog: DualProg = RcRun::lift::<IdentityBrand, _>(Identity(99));
	let interposed = prog.interpose::<ExceptBrand<String>, _, DualRowMinusExcept, _>(
		|_op: Except<'_, String, DualProg>| RcRun::pure(0),
	);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<DualProg>| op.0,
		ExceptBrand<String>: |op: Except<'_, String, DualProg>| match op {
			Except::Throw(_, _) => RcRun::pure(0),
		},
	});
	assert_eq!(result, 99);
}
