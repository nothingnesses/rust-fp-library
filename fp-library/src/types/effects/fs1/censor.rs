//! FS-1 slice: the `Censor` higher-order effect.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition, its smart constructor, and its bucket A parity test. `Censor`
//! owns a sub-program and a log transform; the parent interpreter elaborates it
//! by giving the action a fresh local log, applying the transform, and emitting
//! the result to the outer log (no boundary frame). The censor is transactional
//! on abort: if the action aborts, the local log is dropped and nothing reaches
//! the outer log, because a censor is a listen-shaped boundary whose
//! transform-and-tell never happens for an aborted action; writes outside a
//! censor survive an abort. The cell fields are `pub(super)` because the parent
//! interpreter destructures them.

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
	std::marker::PhantomData,
};

/// Censor is a higher-order effect: it owns an action sub-program and a
/// transform `f` applied to the log the action produces. The interpreter
/// elaborates it by giving the action a fresh local log, applying `f` to the
/// total, and emitting the result to the outer log, so the censor scopes the
/// accumulation (no boundary frame). The cell is generic in the log type `W`
/// and the action result `RAction`; this slice's `Row` pins them to `String`
/// and `()`.
pub(crate) struct CensorBrand<W, RAction>(PhantomData<(W, RAction)>);
pub(crate) struct CensorCell<'a, W: 'static, RAction: 'static, Next> {
	pub(super) f: Box<dyn FnOnce(W) -> W + 'a>,
	pub(super) action: Free<Row, RAction>,
	pub(super) k: Box<dyn FnOnce(RAction) -> Next + 'a>,
}
impl_kind! {
	impl<W: 'static, RAction: 'static> for CensorBrand<W, RAction> {
		type Of<'a, Next: 'a>: 'a = CensorCell<'a, W, RAction, Next>;
	}
}
impl<W: 'static, RAction: 'static> Functor for CensorBrand<W, RAction> {
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
impl<W: 'static, RAction: 'static> OrderOf for CensorBrand<W, RAction> {
	type Order = HigherOrder;
}

pub(crate) fn censor(
	f: impl FnOnce(String) -> String + 'static,
	action: Free<Row, ()>,
) -> Free<Row, ()> {
	let cell: CensorCell<'static, String, (), ()> = CensorCell {
		f: Box::new(f),
		action,
		k: Box::new(|u| u),
	};
	let coyo: Coyoneda<'static, CensorBrand<String, ()>, ()> = Coyoneda::lift(cell);
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
			censor,
			run,
			tell,
			throw,
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

	// Transactional on abort: writes made inside a censored action before an
	// abort are dropped with the local log (the transform-and-tell never
	// happens), while a write made outside the censor survives. Pins the
	// listen-shaped boundary semantics.
	#[test]
	fn censor_drops_the_local_log_when_the_action_aborts() {
		let program: Free<Row, ()> = tell("outer".to_string()).bind(|()| {
			censor(|total| format!("{total}!"), tell("inner".to_string()).bind(|()| throw::<()>()))
		});

		let fx = Fixture::new();
		let result = run(program, &fx.handlers());

		assert_eq!(result, Err(Abort::Throw));
		assert_eq!(*fx.log.borrow(), "outer");
	}
}
