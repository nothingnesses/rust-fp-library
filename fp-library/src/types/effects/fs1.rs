//! FS-1 effects rebuild, crate-internal work in progress.
//!
//! This module is the in-tree vertical slice of the unified-row effects
//! rebuild (remediation-plan review-2, item 4). It is deliberately
//! `pub(crate)` and not part of the public surface: per the adopted hybrid
//! method, the new substrate is built here to a compiling, test-backed state
//! before the dual-row subsystem it replaces is deleted, so a half-built
//! rewrite never ships.
//!
//! FS-1 replaces the dual rows (`Run<R, S, A>`) with one unified row of effect
//! brands and elaborates higher-order effects into first-order ones over that
//! row, rather than using boundary frames. This slice demonstrates the core of
//! that: a single `Coyoneda`-wrapped `CoproductBrand` row carrying first-order
//! effects (`State`, `Throw`) and one higher-order effect (`Catch`) as an
//! in-row cell, interpreted by one pass that elaborates `Catch` into a
//! sub-interpretation over `Throw`. It reproduces the heftia State-with-Catch
//! ordering case from the behaviour-parity oracle (a state write before a
//! caught throw survives).
//!
//! Scope of this slice: the substrate is the existing public `Free` (the
//! `Store = Box`, erased, `'static` form, reused per the POC-11 substrate
//! decision); the `Store`-parameterised Rc/Arc forms and the concrete
//! (non-`'static`) form are folded in later. First-order handling uses direct
//! coproduct matching; brand-keyed dispatch (item 8) is an orthogonal layer
//! added later. Per-brand order markers (item 4 step 3) arrive with that
//! dispatch layer, which is the first place they are used.

#![allow(
	dead_code,
	reason = "FS-1 rebuild in progress (item 4): these items form the vertical slice and are currently exercised only by this module's tests; the public surface that consumes them is added in later steps, and item 20 sweeps any residual allowances at the end of the rebuild."
)]

use {
	crate::{
		Apply,
		brands::{
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
		},
		classes::Functor,
		impl_kind,
		kinds::*,
		types::{
			Coyoneda,
			Free,
			effects::coproduct::Coproduct,
		},
	},
	std::{
		cell::Cell,
		marker::PhantomData,
		rc::Rc,
	},
};

// -- First-order effects --

/// State over a `bool` cell. `Get` reads the current state; `Put` writes it.
/// Each arm carries its continuation, so the effect is a `Functor` over the
/// next program node.
pub(crate) struct StateBrand;
pub(crate) enum StateF<'a, A> {
	Get(Box<dyn FnOnce(bool) -> A + 'a>),
	Put(bool, Box<dyn FnOnce(()) -> A + 'a>),
}
impl_kind! {
	impl for StateBrand {
		type Of<'a, A: 'a>: 'a = StateF<'a, A>;
	}
}
impl Functor for StateBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			StateF::Get(k) => StateF::Get(Box::new(move |s| f(k(s)))),
			StateF::Put(s, k) => StateF::Put(s, Box::new(move |u| f(k(u)))),
		}
	}
}

/// Throw with a unit error. The result type is phantom: a throw never returns,
/// so it can stand in any result position.
pub(crate) struct ThrowBrand;
pub(crate) struct ThrowF<A>(PhantomData<A>);
impl_kind! {
	impl for ThrowBrand {
		type Of<'a, A: 'a>: 'a = ThrowF<A>;
	}
}
impl Functor for ThrowBrand {
	fn map<'a, A: 'a, B: 'a>(
		_f: impl Fn(A) -> B + 'a,
		_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		ThrowF(PhantomData)
	}
}

// -- Higher-order effect as an in-row cell --

/// Catch is a higher-order effect: it owns an action sub-program and a recovery
/// thunk. Its result equals the action result `RAction`. It is stored in the
/// row as a cell (the carrier `R` is fixed to the slice row here) and
/// elaborated by the interpreter into a sub-interpretation over `Throw`, rather
/// than handled by a boundary frame.
pub(crate) struct CatchBrand<RAction>(PhantomData<RAction>);
pub(crate) struct CatchCell<'a, RAction: 'static, Next> {
	action: Free<Row, RAction>,
	recover: Rc<dyn Fn() -> Free<Row, RAction> + 'a>,
	k: Box<dyn FnOnce(RAction) -> Next + 'a>,
}
impl_kind! {
	impl<RAction: 'static> for CatchBrand<RAction> {
		type Of<'a, Next: 'a>: 'a = CatchCell<'a, RAction, Next>;
	}
}
impl<RAction: 'static> Functor for CatchBrand<RAction> {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		let CatchCell {
			action,
			recover,
			k,
		} = fa;
		CatchCell {
			action,
			recover,
			k: Box::new(move |a| f(k(a))),
		}
	}
}

// -- The unified row --

/// The unified effect row for this slice: one `Coyoneda`-wrapped cell per
/// effect, terminated by `CNilBrand`. All effects, first-order and higher-order
/// alike, live in this single row (the defining FS-1 property; the dual scoped
/// row is gone).
pub(crate) type Row = CoproductBrand<
	CoyonedaBrand<StateBrand>,
	CoproductBrand<
		CoyonedaBrand<ThrowBrand>,
		CoproductBrand<CoyonedaBrand<CatchBrand<()>>, CNilBrand>,
	>,
>;

/// The row cell over a result `A`, as handed to [`Free::lift_f`] by the smart
/// constructors: one `Coyoneda`-wrapped operation at its coproduct position
/// whose hole is the operation's result `A`. (The interpreter's [`Free::resume`]
/// hands back the same row shape but with the hole instantiated to the
/// continuation `Free<Row, A>`; that shape is inferred in `run`, not named.)
type Node<A> = Apply!(<Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>);

// -- Smart constructors (inject at the right coproduct position) --

pub(crate) fn get() -> Free<Row, bool> {
	let coyo: Coyoneda<'static, StateBrand, bool> = Coyoneda::lift(StateF::Get(Box::new(|s| s)));
	Free::lift_f(Coproduct::Inl(coyo) as Node<bool>)
}
pub(crate) fn put(value: bool) -> Free<Row, ()> {
	let coyo: Coyoneda<'static, StateBrand, ()> =
		Coyoneda::lift(StateF::Put(value, Box::new(|u| u)));
	Free::lift_f(Coproduct::Inl(coyo) as Node<()>)
}
pub(crate) fn throw<A: 'static>() -> Free<Row, A> {
	let coyo: Coyoneda<'static, ThrowBrand, A> = Coyoneda::lift(ThrowF(PhantomData));
	Free::lift_f(Coproduct::Inr(Coproduct::Inl(coyo)) as Node<A>)
}
pub(crate) fn catch(
	action: Free<Row, ()>,
	recover: impl Fn() -> Free<Row, ()> + 'static,
) -> Free<Row, ()> {
	let cell: CatchCell<'static, (), ()> = CatchCell {
		action,
		recover: Rc::new(recover),
		k: Box::new(|a| a),
	};
	let coyo: Coyoneda<'static, CatchBrand<()>, ()> = Coyoneda::<CatchBrand<()>, _>::lift(cell);
	Free::lift_f(Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(coyo))) as Node<()>)
}

// -- The interpreter: one pass, elaborating Catch over Throw --

/// Interpret a program over the unified row. `state` is the shared `State`
/// cell. A `Throw` aborts to `Err(())`. `Catch` is elaborated here: its action
/// is interpreted as a sub-program over the same `state`; if it throws, the
/// recovery program runs; either way the state cell is shared, so writes before
/// a caught throw survive (the heftia ordering semantics), with no boundary
/// frame.
pub(crate) fn run<A: 'static>(
	program: Free<Row, A>,
	state: &Cell<bool>,
) -> Result<A, ()> {
	let mut program = program;
	loop {
		match program.resume() {
			Ok(value) => return Ok(value),
			Err(layer) => match layer {
				Coproduct::Inl(coyo) => match coyo.lower() {
					StateF::Get(k) => program = k(state.get()),
					StateF::Put(s, k) => {
						state.set(s);
						program = k(());
					}
				},
				Coproduct::Inr(Coproduct::Inl(_throw)) => return Err(()),
				Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(coyo))) => {
					let CatchCell {
						action,
						recover,
						k,
					} = coyo.lower();
					match run(action, state) {
						Ok(()) => {}
						Err(()) => {
							run(recover(), state)?;
						}
					}
					program = k(());
				}
				Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(cnil))) => match cnil {},
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::types::Free,
	};

	// Heftia State-with-Catch ordering (behaviour-parity oracle bucket A):
	// `catch(put(true) >> throw, recover = pure(()))` then `get` yields value
	// `true` and final state `true`. The write before the caught throw survives
	// because the interpreter shares the state cell across the catch (no
	// boundary frame, no rollback). This reproduces the dual-row system's
	// `run_heftia_semantics` result on the unified row.
	#[test]
	fn state_write_survives_caught_throw() {
		let program: Free<Row, bool> =
			catch(Free::bind(put(true), |()| throw::<()>()), || Free::pure(())).bind(|()| get());

		let state = Cell::new(false);
		let result = run(program, &state);

		assert_eq!(result, Ok(true));
		assert!(state.get());
	}
}
