//! FS-1 slice: the `State` effect over a `bool` cell.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition (brand, functor, order marker), its smart constructors, and its
//! bucket A parity test. The only shared surfaces it touches are the parent's
//! `Row` (one tail-appended cell) and interpreter (one dispatch arm plus a
//! `Handlers` field), both append-only.

use {
	super::{
		FirstOrder,
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

/// State over a `bool` cell. `Get` reads the current state; `Put` writes it.
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
impl OrderOf for StateBrand {
	type Order = FirstOrder;
}

pub(crate) fn get() -> Free<Row, bool> {
	let coyo: Coyoneda<'static, StateBrand, bool> = Coyoneda::lift(StateF::Get(Box::new(|s| s)));
	let node: Node<bool> = Coproduct::inject(coyo);
	Free::lift_f(node)
}
pub(crate) fn put(value: bool) -> Free<Row, ()> {
	let coyo: Coyoneda<'static, StateBrand, ()> =
		Coyoneda::lift(StateF::Put(value, Box::new(|u| u)));
	let node: Node<()> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[cfg(test)]
mod tests {
	use crate::types::effects::fs1::{
		Fixture,
		get,
		put,
		run,
	};

	// Behaviour-parity oracle bucket A (single-effect): a `put` then `get` reads
	// back the written value and leaves the state cell holding it.
	#[test]
	fn put_then_get_reads_the_written_state() {
		let program = put(true).bind(|()| get());
		let fx = Fixture::new();
		assert_eq!(run(program, &fx.handlers()), Ok(true));
		assert!(fx.state.get());
	}
}
