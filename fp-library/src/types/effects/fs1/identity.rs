//! FS-1 slice: the `Identity` effect (the trivial functor as an effect).
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition, its smart constructor, and its bucket A parity test. `Identity`
//! is the trivial functor `Identity(A)`: as an effect it carries its
//! continuation directly, so the interpreter just continues with it. It is the
//! no-op/constant target the `Interpose` bucket A oracle rewrites, and its
//! payload field is read by a no-op interpose replacement (`|op| op.0`).

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

/// The trivial functor as an effect: `IdentityF(next)` carries its continuation
/// directly, so the interpreter continues with it. `map f (IdentityF(a)) =
/// IdentityF(f(a))`. The field is `pub(super)` so the parent interpreter and the
/// sibling interpose walker can read it.
pub(crate) struct IdentityBrand;
pub(crate) struct IdentityF<A>(pub(super) A);
impl_kind! {
	impl for IdentityBrand {
		type Of<'a, A: 'a>: 'a = IdentityF<A>;
	}
}
impl Functor for IdentityBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		IdentityF(f(fa.0))
	}
}
impl OrderOf for IdentityBrand {
	type Order = FirstOrder;
}

/// Yield `value` through the trivial `Identity` effect; after interpretation the
/// program continues with `value`.
pub(crate) fn identity_op(value: i32) -> Free<Row, i32> {
	let coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(IdentityF(value));
	let node: Node<i32> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[cfg(test)]
mod tests {
	use crate::types::effects::fs1::{
		Fixture,
		identity_op,
		run,
	};

	// Behaviour-parity oracle bucket A (single-effect): the trivial `Identity`
	// effect yields its carried value and continues, so the program reduces to it.
	#[test]
	fn identity_yields_its_value() {
		let fx = Fixture::new();
		assert_eq!(run(identity_op(7), &fx.handlers()), Ok(7));
	}
}
