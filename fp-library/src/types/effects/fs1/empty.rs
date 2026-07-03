//! FS-1 slice: the `Empty` effect (abort the current branch without a value).
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
	std::marker::PhantomData,
};

/// Empty aborts the current branch without producing a value. The result type is
/// phantom: an empty branch never returns, so it can stand in any result
/// position. In this single-shot slice the abort surfaces as `Err(Abort::Empty)`
/// from the interpreter, which a caller reads as `None` or replaces with a
/// fallback; it propagates through a `catch` (which recovers `Throw` only).
/// Empty's distinctive pruning of nondeterministic branches needs the multi-shot
/// substrate and is interpreted there.
pub(crate) struct EmptyBrand;
pub(crate) enum EmptyF<A> {
	Empty(PhantomData<A>),
}
impl_kind! {
	impl for EmptyBrand {
		type Of<'a, A: 'a>: 'a = EmptyF<A>;
	}
}
impl Functor for EmptyBrand {
	fn map<'a, A: 'a, B: 'a>(
		_f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			EmptyF::Empty(_) => EmptyF::Empty(PhantomData),
		}
	}
}
impl OrderOf for EmptyBrand {
	type Order = FirstOrder;
}

pub(crate) fn empty<A: 'static>() -> Free<Row, A> {
	let coyo: Coyoneda<'static, EmptyBrand, A> = Coyoneda::lift(EmptyF::Empty(PhantomData));
	let node: Node<A> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[cfg(test)]
mod tests {
	use crate::types::effects::fs1::{
		Fixture,
		empty,
		run,
	};

	// Behaviour-parity oracle bucket A (single-shot): an `empty()` aborts the
	// branch with no value. The interpreter surfaces the abort as
	// `Err(Abort::Empty)`, so a caller reads it as `None` (the `run_empty`
	// Option semantics) or substitutes a fallback value via `unwrap_or`.
	// (Empty's nondeterministic-pruning cases are multi-shot and interpreted on
	// that substrate.)
	#[test]
	fn empty_aborts_to_none_or_a_fallback() {
		let none_fx = Fixture::new();
		assert_eq!(run(empty::<i32>(), &none_fx.handlers()).ok(), None);

		let fallback_fx = Fixture::new();
		assert_eq!(run(empty::<i32>(), &fallback_fx.handlers()).unwrap_or(0), 0);
	}
}
