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
	// increasing counter, threaded by the interpreter from a configurable start
	// through a configurable successor. The standard runner starts at `0` and
	// advances by one; a custom runner supplies its own start and successor.
	#[test]
	fn fresh_threads_a_counter_with_a_configurable_start_and_successor() {
		// Standard: start `0`, successor `+1`. Two `fresh()` calls yield `(0, 1)`
		// and leave the counter at `2` (the oracle's `((0, 1), 2)`).
		let program = fresh().bind(|first| fresh().map(move |second| (first, second)));
		let fx = Fixture::new();
		assert_eq!(run(program, &fx.handlers()), Ok((0, 1)));
		assert_eq!(fx.fresh.get(), 2);

		// Custom: start `10`, successor `+2`. Two `fresh()` calls yield `(10, 12)`
		// and leave the counter at `14` (the oracle's `run_fresh_with(10, |c| c + 2)`
		// result `((10, 12), 14)`).
		let custom_program = fresh().bind(|first| fresh().map(move |second| (first, second)));
		let custom_fx = Fixture::with_fresh(10, |c| c + 2);
		assert_eq!(run(custom_program, &custom_fx.handlers()), Ok((10, 12)));
		assert_eq!(custom_fx.fresh.get(), 14);
	}
}
