//! FS-1 slice: the `Except` effect (a typed throw carrying an error payload).
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition, its smart constructor, and its bucket A parity test. Unlike the
//! unit-error `Throw`, `Except` carries a typed error `e` that survives the
//! abort in the interpreter's return channel (`Err(Some(e))`) and reaches a
//! recovery via [`run_except`](super::run_except), the same shape as heftia's
//! `runThrow` reifying a throw into `Either e a`. The error is monomorphic at
//! this slice (`&'static str`, matching the `Except` bucket A oracle).

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

/// A typed throw carrying an error payload `E`. The result type is phantom: a
/// throw never returns, so it can stand in any result position. The `Functor`
/// instance preserves the error and only retypes the phantom, mirroring the
/// dual row's `map f (Throw e) = Throw e`.
pub(crate) struct ExceptBrand<E>(PhantomData<E>);
pub(crate) enum ExceptF<E, A> {
	Throw(E, PhantomData<A>),
}
impl_kind! {
	impl<E: 'static> for ExceptBrand<E> {
		type Of<'a, A: 'a>: 'a = ExceptF<E, A>;
	}
}
impl<E: 'static> Functor for ExceptBrand<E> {
	fn map<'a, A: 'a, B: 'a>(
		_f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			ExceptF::Throw(e, _) => ExceptF::Throw(e, PhantomData),
		}
	}
}
impl<E> OrderOf for ExceptBrand<E> {
	type Order = FirstOrder;
}

/// Throw a typed error `e`. The interpreter aborts to `Err(Some(e))`, carrying
/// `e` in the return channel for [`run_except`](super::run_except) to recover.
pub(crate) fn throw_e<A: 'static>(e: &'static str) -> Free<Row, A> {
	let coyo: Coyoneda<'static, ExceptBrand<&'static str>, A> =
		Coyoneda::lift(ExceptF::Throw(e, PhantomData));
	let node: Node<A> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[cfg(test)]
mod tests {
	use crate::types::{
		Free,
		effects::fs1::{
			Fixture,
			Row,
			run_except,
			throw_e,
		},
	};

	// Behaviour-parity oracle bucket A (single-effect): a typed `throw_e(e)`
	// delivers its error `e` to the recovery, whose value becomes the program
	// result. `run_except(throw_e("oops"), |e| { assert e; pure(-1) })` yields
	// `Ok(-1)` with the recovery seeing `"oops"`.
	#[test]
	fn except_throw_delivers_error_to_recovery() {
		let fx = Fixture::new();
		let result = run_except(throw_e::<i32>("oops"), &fx.handlers(), |e| {
			assert_eq!(e, "oops");
			Free::pure(-1)
		});
		assert_eq!(result, Ok(-1));
	}

	// A throw after a successful bind step still reaches the recovery with its
	// error: `pure(7).bind(|_| throw_e("after-bind"))` recovers to `-1`.
	#[test]
	fn except_throw_after_bind_delivers_error() {
		let program: Free<Row, i32> =
			Free::<Row, i32>::pure(7).bind(|_v| throw_e::<i32>("after-bind"));
		let fx = Fixture::new();
		let result = run_except(program, &fx.handlers(), |e| {
			assert_eq!(e, "after-bind");
			Free::pure(-1)
		});
		assert_eq!(result, Ok(-1));
	}
}
