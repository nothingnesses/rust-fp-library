//! FS-1 slice: the `Throw` effect (abort with a unit error).
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

/// Throw with a unit error. The result type is phantom: a throw never returns,
/// so it can stand in any result position.
pub(crate) struct ThrowBrand;
pub(crate) enum ThrowF<A> {
	Throw(PhantomData<A>),
}
impl_kind! {
	impl for ThrowBrand {
		type Of<'a, A: 'a>: 'a = ThrowF<A>;
	}
}
impl Functor for ThrowBrand {
	fn map<'a, A: 'a, B: 'a>(
		_f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			ThrowF::Throw(_) => ThrowF::Throw(PhantomData),
		}
	}
}
impl OrderOf for ThrowBrand {
	type Order = FirstOrder;
}

pub(crate) fn throw<A: 'static>() -> Free<Row, A> {
	let coyo: Coyoneda<'static, ThrowBrand, A> = Coyoneda::lift(ThrowF::Throw(PhantomData));
	let node: Node<A> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[cfg(test)]
mod tests {
	use crate::types::effects::fs1::{
		Abort,
		Fixture,
		run,
		throw,
	};

	// Behaviour-parity oracle bucket A (single-effect): a `throw` aborts the
	// program to `Err(Abort::Throw)` regardless of the result position it
	// stands in.
	#[test]
	fn throw_aborts_to_err() {
		let fx = Fixture::new();
		assert_eq!(run(throw::<i32>(), &fx.handlers()), Err(Abort::Throw));
	}
}
