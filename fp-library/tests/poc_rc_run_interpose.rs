// POC: substrate-level `interpose` primitive on RcRun.
//
// Question being answered: can `RcRun` host an `interpose<EBrand>`
// primitive that walks the program tree and substitutes one effect's
// dispatch with a replacement, without narrowing the row? This is the
// Rust analogue of heftia's `interposeInWith`. The R1-Option-B
// remediation in `remediation_proposals_phase_4.md` builds Catch's
// scoped-handler dispatch on top of this primitive.
//
// Hypothesis: yes. The substrate primitives `peel`, `RcCoyoneda::lift`
// / `lower_ref`, `<EBrand as Functor>::map`, `RcFree::wrap`, and the
// Run<->Free conversion sugar (`from_rc_free` / `into_rc_free`) are
// the same set `RcRun::interpret_with_shared` uses internally at
// `fp-library/src/types/effects/rc_run.rs:992-1062`. The only
// structural difference for interpose is the rebuilt layer stays in
// row `R` instead of being threaded through a row-narrowing handler.
//
// Scope of this POC: a concrete one-effect-row interpose for
// `Coproduct<RcCoyoneda<IdentityBrand>, CNil>`. Generalising to
// `<EBrand, Idx, R>` would mirror `interpret_with_shared` line for
// line; the constraint surface is identical, the only delta is `R`
// in place of `RMinusE` everywhere. Validating the concrete case
// proves the substrate primitives suffice.

#![allow(dead_code)]

use {
	fp_library::{
		Apply,
		brands::{
			CNilBrand,
			CoproductBrand,
			IdentityBrand,
			NodeBrand,
			RcCoyonedaBrand,
		},
		classes::Functor,
		handlers,
		kinds::*,
		types::{
			Identity,
			RcCoyoneda,
			RcFree,
			effects::{
				coproduct::Coproduct,
				node::Node,
				rc_run::RcRun,
			},
		},
	},
	std::rc::Rc,
};

type IdentityRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type Scoped = CNilBrand;
type Prog = RcRun<IdentityRow, Scoped, i32>;
type ProgFree = RcFree<NodeBrand<IdentityRow, Scoped>, i32>;

// ----------------------------------------------------------------
// Interpose primitive (concrete one-effect row).
//
// Walks `prog` and, for each Identity-effect dispatch, recurses on
// the inner program through `transform`. The transform is applied
// per-program (not per-effect-payload), mirroring how heftia's
// `interposeInWith` walks the action's `Eff` value re-interpreting
// matched effects.
// ----------------------------------------------------------------

fn interpose_identity(
	prog: Prog,
	transform: Rc<dyn Fn(Prog) -> Prog>,
) -> Prog {
	match prog.peel() {
		Ok(a) => RcRun::pure(a),
		Err(Node::First(layer)) => match layer {
			Coproduct::Inl(coyo) => {
				let lowered: Identity<Prog> = coyo.lower_ref();
				let t = transform.clone();
				// Map: recurse on the inner program, then return its
				// underlying Free for re-wrap. Same map shape as
				// `interpret_with_shared`'s Inr branch at
				// `rc_run.rs:1047-1056`, applied to the matched arm.
				let mapped: Identity<ProgFree> = <IdentityBrand as Functor>::map(
					move |inner: Prog| {
						let recursed = interpose_identity(inner, t.clone());
						let post_transform = (transform.clone())(recursed);
						post_transform.into_rc_free()
					},
					lowered,
				);
				// Re-wrap as a Coyoneda layer in the same row, then
				// `RcFree::wrap` to construct the layered program.
				let coyo_back: RcCoyoneda<'static, IdentityBrand, ProgFree> =
					RcCoyoneda::lift(mapped);
				let layer_back: Apply!(
					<IdentityRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ProgFree>
				) = Coproduct::Inl(coyo_back);
				let node: Apply!(
					<NodeBrand<IdentityRow, Scoped> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						ProgFree,
					>
				) = Node::First(layer_back);
				let free: ProgFree = RcFree::wrap(node);
				RcRun::from_rc_free(free)
			}
			Coproduct::Inr(cnil) => match cnil {},
		},
		Err(Node::Scoped(cnil)) => match cnil {},
	}
}

// ----------------------------------------------------------------
// T1. Round-trip baseline.
// Confirms a freshly-lifted program interprets correctly.
// ----------------------------------------------------------------

#[test]
fn t1_baseline_program_interprets_to_lifted_value() {
	let prog: Prog = RcRun::lift::<IdentityBrand, _>(Identity(7));
	let result = prog.interpret(handlers! {
		IdentityBrand: |op: Identity<Prog>| op.0,
	});
	assert_eq!(result, 7);
}

// ----------------------------------------------------------------
// T2. Interpose transforms the inner program before re-emission.
// Validates the walk-and-rebuild mechanism: the rebuilt program is
// still in row `IdentityRow`, the same handler list discharges it,
// and the transform observably changes the result.
// ----------------------------------------------------------------

#[test]
fn t2_interpose_transforms_inner_program() {
	let prog: Prog = RcRun::lift::<IdentityBrand, _>(Identity(7));
	let transform: Rc<dyn Fn(Prog) -> Prog> = Rc::new(|p: Prog| p.map(|x: i32| x + 100));
	let interposed = interpose_identity(prog, transform);
	let result = interposed.interpret(handlers! {
		IdentityBrand: |op: Identity<Prog>| op.0,
	});
	// Original lifted value 7; transform applied during interpose
	// adds 100; expected 107.
	assert_eq!(result, 107);
}

// ----------------------------------------------------------------
// T3. Same-row property: the rebuilt program can itself be
// interposed again, demonstrating that interpose preserves the row
// (the heftia property `interposeInWith` guarantees).
// ----------------------------------------------------------------

#[test]
fn t3_interpose_preserves_row_for_re_dispatch() {
	let prog: Prog = RcRun::lift::<IdentityBrand, _>(Identity(3));
	let double: Rc<dyn Fn(Prog) -> Prog> = Rc::new(|p: Prog| p.map(|x: i32| x * 2));
	let after_first = interpose_identity(prog, double);

	// Second interpose works because `after_first` is still in row
	// IdentityRow.
	let plus_one: Rc<dyn Fn(Prog) -> Prog> = Rc::new(|p: Prog| p.map(|x: i32| x + 1));
	let after_second = interpose_identity(after_first, plus_one);

	let result = after_second.interpret(handlers! {
		IdentityBrand: |op: Identity<Prog>| op.0,
	});
	// 3 -> 6 (double) -> 7 (plus_one).
	assert_eq!(result, 7);
}
