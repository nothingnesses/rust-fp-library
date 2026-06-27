//! FS-1 slice: the `Listen` higher-order effect.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition, its smart constructor, and its bucket A parity test. `Listen`
//! owns a sub-program and is elaborated by the parent interpreter, which runs
//! the action under the SAME `Handlers` (so the action's writes land in the
//! same `log` and are PRESERVED), capturing the log delta the action produced
//! and pairing it with the action's value, rather than using a boundary frame.
//! The cell fields are `pub(super)` because the parent interpreter destructures
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
};

/// Listen is a higher-order effect: it owns an action sub-program whose result
/// is pinned to `i32`, and observes the `Writer` log the action produces. The
/// interpreter elaborates it by running the action under the same `Handlers` (so
/// the action's writes are preserved into the outer log), capturing the log
/// delta and resuming with `(value, observed)` — no boundary frame, in contrast
/// with `Censor`, which replaces the log with a fresh local one.
pub(crate) struct ListenBrand;
pub(crate) struct ListenCell<'a, Next> {
	pub(super) action: Free<Row, i32>,
	pub(super) k: Box<dyn FnOnce((i32, String)) -> Next + 'a>,
}
impl_kind! {
	impl for ListenBrand {
		type Of<'a, Next: 'a>: 'a = ListenCell<'a, Next>;
	}
}
impl Functor for ListenBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		let ListenCell {
			action,
			k,
		} = fa;
		ListenCell {
			action,
			k: Box::new(move |pair| f(k(pair))),
		}
	}
}
impl OrderOf for ListenBrand {
	type Order = HigherOrder;
}

pub(crate) fn listen(action: Free<Row, i32>) -> Free<Row, (i32, String)> {
	let cell: ListenCell<'static, (i32, String)> = ListenCell {
		action,
		k: Box::new(|pair| pair),
	};
	let coyo: Coyoneda<'static, ListenBrand, (i32, String)> = Coyoneda::lift(cell);
	let node: Node<(i32, String)> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[cfg(test)]
mod tests {
	use crate::types::{
		Free,
		effects::fs1::{
			Fixture,
			Row,
			listen,
			run,
			tell,
		},
	};

	// Behaviour-parity oracle bucket A: Writer listen observes and preserves.
	// `listen(tell("first") >> tell("second") >> pure(40))` observes the log the
	// action emitted (`"firstsecond"`) and pairs it with the value, while
	// preserving those writes in the outer log; the outer `tell("outer")` then
	// appends, so the final log is `"firstsecondouter"`. The result is
	// `(40 + 2, "firstsecond")`.
	#[test]
	fn listen_observes_and_preserves_the_action_log() {
		let action = tell("first".to_string()).bind(|()| tell("second".to_string())).map(|()| 40);
		let program: Free<Row, (i32, String)> = listen(action).bind(|(value, observed)| {
			tell("outer".to_string()).map(move |()| (value + 2, observed))
		});

		let fx = Fixture::new();
		let result = run(program, &fx.handlers());

		assert_eq!(result, Ok((42, "firstsecond".to_string())));
		assert_eq!(*fx.log.borrow(), "firstsecondouter");
	}
}
