#![cfg(feature = "effects")]
// POC: substrate-level `handle_with_either` primitive on RcRun.
//
// Question being answered: can `RcRun` host an
// `handle_with_either<EBrand, Idx>(self, fo_handlers) -> Result<A, EBrand::Op>`
// primitive that walks the program tree, dispatches non-matched
// FO effects via the supplied handler list, and short-circuits
// when the matched effect (`EBrand`) is encountered, returning
// the matched effect's payload? This is the structural building
// block the `Catch` cons-cell impl uses to distinguish "action
// completed normally" from "action threw the matched error" without
// resorting to interior mutability or a placeholder-program sentinel.
//
// Hypothesis: yes. The substrate primitives `peel`, `Coproduct`
// pattern matching, `RcCoyoneda::lower_ref`, and the user
// handler's existing `Identity<Prog> -> Prog` shape suffice. The
// only structural difference from `handle` is the matched-
// effect arm short-circuits with `Err(payload)` instead of
// dispatching through the handler list, and the Pure arm wraps
// the result in `Ok` instead of returning it directly.
//
// Scope of this POC: a concrete two-effect row with `Identity`
// (FO-dispatched normally) and `Except<String>` (matched-and-
// short-circuited). The generic shape across all six Run
// wrappers (parameterised by `EBrand`, `Idx`, and the FO handler
// list type per `DispatchHandlers`) is mechanical from this
// concrete case: replace the literal `Coproduct::Inr(Inl(except_coyo))`
// arm with a `Member::project::<RcCoyoneda<EBrand, _>, Idx>`
// call, and replace the literal `Coproduct::Inl(identity_coyo)`
// branch with a generic `fo_handlers.dispatch(rest_layer)` that
// returns the next program.
//
// Production shape (sketch, not executed in this POC):
//
// ```ignore
// pub fn handle_with_either<EBrand, Idx>(
//     self,
//     fo_handlers: impl for<'h> DispatchHandlers<...>,
// ) -> Result<A, <EBrand as Kind>::Of<'static, Self>>
// where
//     // Member witness for EBrand in the row, plus the usual
//     // WrapDrop / Functor / Clone bounds inherited from
//     // handle_with_shared at rc_run.rs:992-1023.
// {
//     let mut prog = self;
//     loop {
//         match prog.peel() {
//             Ok(a) => return Ok(a),
//             Err(Node::First(layer)) => match Member::project(layer) {
//                 Ok(matched_coyo) => return Err(matched_coyo.lower_ref()),
//                 Err(rest) => prog = fo_handlers.dispatch(rest),
//             },
//             Err(Node::Scoped(cnil)) => match cnil {},
//         }
//     }
// }
// ```
#![allow(dead_code)]

use {
	core::marker::PhantomData,
	fp_library::{
		brands::{
			CNilBrand,
			CoproductBrand,
			ExceptBrand,
			IdentityBrand,
			RcCoyonedaBrand,
		},
		types::{
			Identity,
			effects::{
				coproduct::Coproduct,
				except::Except,
				node::Node,
				rc_run::RcRun,
			},
		},
	},
};

// Two-effect row: Identity is FO-dispatched normally; Except<String>
// is matched and short-circuited.
type FirstRow = CoproductBrand<
	RcCoyonedaBrand<IdentityBrand>,
	CoproductBrand<RcCoyonedaBrand<ExceptBrand<String>>, CNilBrand>,
>;
type Scoped = CNilBrand;
type Prog = RcRun<FirstRow, Scoped, i32>;

// ----------------------------------------------------------------
// `handle_with_either` primitive (concrete two-effect row).
//
// Walks `prog`. On each layer:
// - `Pure(a)` returns `Ok(a)`.
// - `Coproduct::Inl(Identity)` dispatches via `identity_handler`
//   (which returns the next program in the same row) and recurses.
// - `Coproduct::Inr(Inl(Except))` short-circuits with `Err(error)`.
// - `Coproduct::Inr(Inr(cnil))` is uninhabited.
//
// The Identity branch mirrors `handle`'s loop body
// at `rc_run.rs:669-676`; the Except branch is the new short-
// circuit logic.
// ----------------------------------------------------------------

fn handle_with_either_except<F>(
	prog: Prog,
	identity_handler: F,
) -> Result<i32, String>
where
	F: Fn(Identity<Prog>) -> Prog + 'static, {
	let mut current = prog;
	loop {
		match current.peel() {
			Ok(a) => return Ok(a),
			Err(Node::First(layer)) => match layer {
				Coproduct::Inl(identity_coyo) => {
					let lowered: Identity<Prog> = identity_coyo.lower_ref();
					current = identity_handler(lowered);
				}
				Coproduct::Inr(rest) => match rest {
					Coproduct::Inl(except_coyo) => {
						let lowered: Except<'static, String, Prog> = except_coyo.lower_ref();
						match lowered {
							Except::Throw(e, _) => return Err(e),
						}
					}
					Coproduct::Inr(cnil) => match cnil {},
				},
			},
			Err(Node::Scoped(cnil)) => match cnil {},
		}
	}
}

// ----------------------------------------------------------------
// T1. Pure program returns `Ok` with the lifted value.
// Confirms the Pure arm wraps the result in `Ok` rather than
// returning it directly (the difference from `handle`).
// ----------------------------------------------------------------

#[test]
fn t1_pure_program_returns_ok_with_value() {
	let prog: Prog = RcRun::pure(42);
	let result = handle_with_either_except(prog, |op: Identity<Prog>| op.0);
	assert_eq!(result, Ok(42));
}

// ----------------------------------------------------------------
// T2. Single throw short-circuits to `Err` carrying the error.
// Confirms the matched-effect arm short-circuits without invoking
// any user handler for the matched effect.
// ----------------------------------------------------------------

#[test]
fn t2_single_throw_short_circuits_to_err() {
	let prog: Prog = RcRun::throw::<String, _>("oops".to_string());
	let result = handle_with_either_except(prog, |op: Identity<Prog>| op.0);
	assert_eq!(result, Err("oops".to_string()));
}

// ----------------------------------------------------------------
// T3. Identity-then-Throw chain short-circuits AFTER the Identity
// is dispatched. Validates the FO-dispatch arm runs before the
// matched-effect arm fires (mirroring how Catch dispatch runs
// first-order effects up until the Throw point).
// ----------------------------------------------------------------

#[test]
fn t3_identity_then_throw_short_circuits_after_dispatch() {
	let identity_step: Prog = RcRun::lift::<IdentityBrand, _>(Identity(7));
	let prog: Prog =
		identity_step.bind(|_v: i32| RcRun::throw::<String, _>("after-identity".to_string()));
	let result = handle_with_either_except(prog, |op: Identity<Prog>| op.0);
	assert_eq!(result, Err("after-identity".to_string()));
}

// ----------------------------------------------------------------
// T4. Identity-only program (no throw) runs to completion via the
// FO handler and returns `Ok` with the final value. Validates the
// happy path: the FO dispatch loop terminates normally when the
// matched effect never fires.
// ----------------------------------------------------------------

#[test]
fn t4_identity_only_program_runs_to_completion() {
	let step1: Prog = RcRun::lift::<IdentityBrand, _>(Identity(10));
	let step2 = step1.bind(|v: i32| RcRun::lift::<IdentityBrand, _>(Identity(v + 5)));
	let prog: Prog = step2.bind(|v: i32| RcRun::pure(v * 2));
	let result = handle_with_either_except(prog, |op: Identity<Prog>| op.0);
	// 10 -> bind -> 15 -> bind -> 30.
	assert_eq!(result, Ok(30));
}

// ----------------------------------------------------------------
// T5. Construct the matched-effect short-circuit value at a
// type-level position that mimics the Catch dispatcher's call
// site, demonstrating the `Result<A, EffectOp>`-typed return is
// usable for downstream pattern-matching against `Except::Throw`.
// ----------------------------------------------------------------

#[test]
fn t5_short_circuited_payload_is_usable_for_catch_recovery() {
	let prog: Prog = RcRun::throw::<String, _>("recoverable".to_string());
	let result = handle_with_either_except(prog, |op: Identity<Prog>| op.0);
	let recovered: i32 = match result {
		Ok(a) => a,
		Err(e) => {
			// Catch dispatcher would call `(catch.handler)(thrown_e)` here;
			// for the POC we simulate by mapping the error to a recovery value.
			let _phantom: PhantomData<&'static i32> = PhantomData;
			if e == "recoverable" { -1 } else { -2 }
		}
	};
	assert_eq!(recovered, -1);
}
