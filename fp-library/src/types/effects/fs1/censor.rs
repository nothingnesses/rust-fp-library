//! FS-1 slice: the `Censor` higher-order effect.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition, its smart constructor, and its bucket A parity test. `Censor`
//! owns a sub-program and a log transform; the parent interpreter elaborates it
//! by giving the action a fresh local log, applying the transform, and emitting
//! the result to the outer log (no boundary frame). The cell fields are
//! `pub(super)` because the parent interpreter destructures them.

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
	std::rc::Rc,
};

/// Censor is a higher-order effect: it owns an action sub-program and a
/// transform `f` applied to the log the action produces. The interpreter
/// elaborates it by giving the action a fresh local log, applying `f` to the
/// total, and emitting the result to the outer log, so the censor scopes the
/// accumulation (no boundary frame).
pub(crate) struct CensorBrand;
pub(crate) struct CensorCell<'a, Next> {
	pub(super) f: Rc<dyn Fn(String) -> String + 'a>,
	pub(super) action: Free<Row, ()>,
	pub(super) k: Box<dyn FnOnce(()) -> Next + 'a>,
}
impl_kind! {
	impl for CensorBrand {
		type Of<'a, Next: 'a>: 'a = CensorCell<'a, Next>;
	}
}
impl Functor for CensorBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		let CensorCell {
			f: transform,
			action,
			k,
		} = fa;
		CensorCell {
			f: transform,
			action,
			k: Box::new(move |u| f(k(u))),
		}
	}
}
impl OrderOf for CensorBrand {
	type Order = HigherOrder;
}

pub(crate) fn censor(
	f: impl Fn(String) -> String + 'static,
	action: Free<Row, ()>,
) -> Free<Row, ()> {
	let cell: CensorCell<'static, ()> = CensorCell {
		f: Rc::new(f),
		action,
		k: Box::new(|u| u),
	};
	let coyo: Coyoneda<'static, CensorBrand, ()> = Coyoneda::lift(cell);
	let node: Node<()> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[cfg(test)]
mod tests {
	use crate::types::{
		Free,
		effects::fs1::{
			Fixture,
			Row,
			censor,
			run,
			tell,
		},
	};

	// Behaviour-parity oracle bucket A: Writer post-censor.
	// `censor(f, tell("Hello") >> tell(" world!"))` with `f(total) = total + "!"`
	// yields the log `"Hello world!!"`: the action's tells accumulate in the
	// censor's local log, then `f` is applied to the total and emitted.
	#[test]
	fn censor_transforms_the_accumulated_log() {
		let program: Free<Row, ()> = censor(
			|total| format!("{total}!"),
			tell("Hello".to_string()).bind(|()| tell(" world!".to_string())),
		);

		let fx = Fixture::new();
		let result = run(program, &fx.handlers());

		assert_eq!(result, Ok(()));
		assert_eq!(*fx.log.borrow(), "Hello world!!");
	}
}
