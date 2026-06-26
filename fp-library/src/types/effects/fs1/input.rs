//! FS-1 slice: the `Input` effect (drain a supplied queue of values).
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition, its smart constructor, and its bucket A parity test. `input()`
//! yields `Some(value)` while the handler's queue has values and `None` once it
//! is exhausted. This was the solo template port that validated the catalog-port
//! fan-out mechanism (the marked anchors in the parent `fs1.rs`).

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

/// Input over a queue of `&'static str` values. `Input` reads the next value:
/// `Some(value)` while values remain, `None` after the queue is drained.
pub(crate) struct InputBrand;
pub(crate) enum InputF<'a, A> {
	Input(Box<dyn FnOnce(Option<&'static str>) -> A + 'a>),
}
impl_kind! {
	impl for InputBrand {
		type Of<'a, A: 'a>: 'a = InputF<'a, A>;
	}
}
impl Functor for InputBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			InputF::Input(k) => InputF::Input(Box::new(move |v| f(k(v)))),
		}
	}
}
impl OrderOf for InputBrand {
	type Order = FirstOrder;
}

pub(crate) fn input() -> Free<Row, Option<&'static str>> {
	let coyo: Coyoneda<'static, InputBrand, Option<&'static str>> =
		Coyoneda::lift(InputF::Input(Box::new(|v| v)));
	let node: Node<Option<&'static str>> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[cfg(test)]
mod tests {
	use crate::types::effects::fs1::{
		Fixture,
		input,
		run,
	};

	// Behaviour-parity oracle bucket A: `input()` drains the handler's supplied
	// queue, yielding `Some(value)` per value then `None` once exhausted. Three
	// `input()` calls over `["red", "blue"]` yield `(Some("red"), Some("blue"),
	// None)`, matching the dual-row sequence runner.
	#[test]
	fn input_drains_the_queue_then_yields_none() {
		let program = input().bind(|first| {
			input().bind(move |second| input().map(move |third| (first, second, third)))
		});
		let fx = Fixture::with_input(["red", "blue"]);
		assert_eq!(run(program, &fx.handlers()), Ok((Some("red"), Some("blue"), None)));
	}
}
