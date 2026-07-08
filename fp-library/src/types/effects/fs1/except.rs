//! FS-1 slice: the `Except` effect (a typed throw carrying an error payload).
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition and its smart constructor are emitted by
//! `fp_macros::define_effect!` from the operation signature below, and the
//! module keeps its bucket A parity tests. Unlike the unit-error `Throw`,
//! `Except` carries a typed error that survives the abort in the interpreter's
//! return channel (`Err(Abort::Except(e))`) and reaches a recovery via
//! [`run_except`](super::run_except), the same shape as heftia's `runThrow`
//! reifying a throw into `Either e a`. The error type is the effect generic
//! `E`; the slice's `Row` pins it to `&'static str` (matching the `Except`
//! bucket A oracle), so the row-generic constructor infers `E` from the row
//! and the parent re-exports it flat as `throw_e`.

fp_macros::define_effect! {
	/// A typed throw carrying an error payload `E`: it never returns, so it
	/// stands in any result position. The interpreter aborts to
	/// `Err(Abort::Except(e))`, carrying `e` in the return channel for
	/// [`run_except`](super::run_except) to recover; the abort propagates
	/// through a `catch` with its payload intact.
	#[handler_state(none)]
	#[crate_path(crate)]
	pub(crate) effect Except<E: 'static> {
		/// Throw a typed error `error`, aborting with the payload carried.
		fn throw(error: E) -> !;
	}
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
	// `Ok(-1)` with the recovery seeing `"oops"`. The result type is inferred
	// from the recovery (the constructor's turbofish would bind the error type
	// `E`, which the row already forces to `&'static str`).
	#[test]
	fn except_throw_delivers_error_to_recovery() {
		let fx = Fixture::new();
		let program: Free<Row, i32> = throw_e("oops");
		let result = run_except(program, &fx.handlers(), |e| {
			assert_eq!(e, "oops");
			Free::pure(-1)
		});
		assert_eq!(result, Ok(-1));
	}

	// A throw after a successful bind step still reaches the recovery with its
	// error: `pure(7).bind(|_| throw_e("after-bind"))` recovers to `-1`.
	#[test]
	fn except_throw_after_bind_delivers_error() {
		let program: Free<Row, i32> = Free::<Row, i32>::pure(7).bind(|_v| throw_e("after-bind"));
		let fx = Fixture::new();
		let result = run_except(program, &fx.handlers(), |e| {
			assert_eq!(e, "after-bind");
			Free::pure(-1)
		});
		assert_eq!(result, Ok(-1));
	}
}
