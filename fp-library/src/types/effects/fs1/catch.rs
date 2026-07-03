//! FS-1 slice: the `Catch` higher-order effect.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition, its smart constructor, and its bucket A parity test. `Catch`
//! owns a sub-program and is elaborated by the parent interpreter, which shares
//! the `State` cell across the recursive call (so a write before a caught throw
//! survives), rather than using a boundary frame. A catch recovers the bare
//! `Throw` abort only: an `Empty` dead branch and a typed `Except` throw are
//! different effects with their own boundaries and propagate through it. The
//! cell fields are `pub(super)` because the parent interpreter destructures
//! them when it elaborates the cell.

use {
	super::{
		HigherOrder,
		Node,
		OrderOf,
		Row,
	},
	crate::{
		Apply,
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
		marker::PhantomData,
		rc::Rc,
	},
};

/// Catch is a higher-order effect: it owns an action sub-program and a recovery
/// thunk, its result equal to the action result `RAction`. The interpreter
/// elaborates it into a sub-interpretation over `Throw`, sharing the `State`
/// cell (so writes before a caught throw survive), rather than using a boundary
/// frame.
pub(crate) struct CatchBrand<RAction>(PhantomData<RAction>);
pub(crate) struct CatchCell<'a, RAction: 'static, Next> {
	pub(super) action: Free<Row, RAction>,
	pub(super) recover: Rc<dyn Fn() -> Free<Row, RAction> + 'a>,
	pub(super) k: Box<dyn FnOnce(RAction) -> Next + 'a>,
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
impl<RAction> OrderOf for CatchBrand<RAction> {
	type Order = HigherOrder;
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
	let node: Node<()> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[cfg(test)]
mod tests {
	use crate::types::{
		Free,
		effects::fs1::{
			Abort,
			Fixture,
			Row,
			catch,
			empty,
			get,
			put,
			run,
			run_except,
			throw,
			throw_e,
		},
	};

	// Behaviour-parity oracle bucket A: State-with-Catch ordering.
	// `catch(put(true) >> throw, recover = pure(()))` then `get` yields value
	// `true` and final state `true`. The write before the caught throw survives
	// because the interpreter shares the state cell across the catch (no boundary
	// frame, no rollback).
	#[test]
	fn state_write_survives_caught_throw() {
		let program: Free<Row, bool> =
			catch(put(true).bind(|()| throw::<()>()), || Free::pure(())).bind(|()| get());

		let fx = Fixture::new();
		let result = run(program, &fx.handlers());

		assert_eq!(result, Ok(true));
		assert!(fx.state.get());
	}

	// A catch is selective: a typed `Except` throw inside the caught action is
	// not recovered by the catch; it propagates with its payload intact, so an
	// enclosing `run_except` still receives the error value.
	#[test]
	fn typed_except_propagates_through_catch_to_run_except() {
		let program: Free<Row, i32> =
			catch(throw_e::<()>("boom"), || Free::pure(())).bind(|()| Free::<Row, i32>::pure(1));

		let fx = Fixture::new();
		let recovered = run_except(program, &fx.handlers(), |e| {
			assert_eq!(e, "boom");
			Free::pure(42)
		});

		assert_eq!(recovered, Ok(42));
	}

	// A catch is selective: an `Empty` dead branch inside the caught action is
	// not recovered by the catch; the abort propagates as `Abort::Empty`.
	#[test]
	fn empty_propagates_through_catch() {
		let program: Free<Row, i32> =
			catch(empty::<()>(), || Free::pure(())).bind(|()| Free::<Row, i32>::pure(1));

		let fx = Fixture::new();
		assert_eq!(run(program, &fx.handlers()), Err(Abort::Empty));
	}
}
