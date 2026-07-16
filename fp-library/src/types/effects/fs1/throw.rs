//! FS-1 slice: the `Throw` effect (abort with a unit error).
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition and its smart constructor are emitted by
//! `fp_macros::define_effect!` from the operation signature below, and the
//! module keeps its bucket A parity test. `throw` is a no-resume (`-> !`)
//! operation, so the emitted variant stores `PhantomData` instead of a
//! continuation: a throw never returns, so it stands in any result position.

fp_macros::define_effect! {
	/// Throw with a unit error: the program aborts and carries no continuation.
	#[handler_state(none)]
	#[crate_path(crate)]
	pub(crate) effect Throw {
		/// Abort the current program with a bare throw.
		fn throw() -> !;
	}
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
		assert_eq!(run(throw::<i32, _, _, _>(), &fx.handlers()), Err(Abort::Throw));
	}
}
