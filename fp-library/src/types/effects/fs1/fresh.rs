//! FS-1 slice: the `Fresh` effect (a monotonic counter).
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition, its smart constructor, and its bucket A parity test.

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

/// Fresh generates a monotonically increasing counter value. `Fresh` requests
/// the next value and continues with it; the interpreter threads the counter.
pub(crate) struct FreshBrand;
pub(crate) enum FreshF<'a, A> {
	Fresh(Box<dyn FnOnce(usize) -> A + 'a>),
}
impl_kind! {
	impl for FreshBrand {
		type Of<'a, A: 'a>: 'a = FreshF<'a, A>;
	}
}
impl Functor for FreshBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			FreshF::Fresh(k) => FreshF::Fresh(Box::new(move |n| f(k(n)))),
		}
	}
}
impl OrderOf for FreshBrand {
	type Order = FirstOrder;
}

pub(crate) fn fresh() -> Free<Row, usize> {
	let coyo: Coyoneda<'static, FreshBrand, usize> = Coyoneda::lift(FreshF::Fresh(Box::new(|n| n)));
	let node: Node<usize> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[cfg(test)]
mod tests {
	use crate::types::effects::fs1::{
		Fixture,
		fresh,
		run,
	};

	// Behaviour-parity oracle bucket A: `fresh()` yields a monotonically
	// increasing counter. Two `fresh()` calls in sequence yield `(0, 1)` and
	// leave the counter at `2`, matching the standard Fresh runner result
	// `((0, 1), 2)`.
	#[test]
	fn fresh_yields_a_monotonic_counter() {
		let program = fresh().bind(|first| fresh().map(move |second| (first, second)));
		let fx = Fixture::new();
		assert_eq!(run(program, &fx.handlers()), Ok((0, 1)));
		assert_eq!(fx.fresh.get(), 2);
	}
}
