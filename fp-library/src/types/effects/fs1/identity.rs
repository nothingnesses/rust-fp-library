//! FS-1 slice: the `Identity` effect (a value-echoing no-op).
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition and its smart constructor are emitted by
//! `fp_macros::define_effect!` from the operation signature below, and the
//! module keeps its bucket A parity test. `Identity` is the no-op first-order
//! target the `Interpose` bucket A oracle rewrites: it carries a value and
//! echoes it to the continuation, so the interpreter just resumes with the
//! carried value, and the sibling interpose walker rewrites its dispatch. It is
//! an echo operation rather than the trivial functor because the macro's
//! emission model has no bare-hole cell shape: every operation variant ends in
//! a continuation or `PhantomData`, so the no-op target is expressed as
//! `identity_op(value) -> value`.

fp_macros::define_effect! {
	/// The value-echoing no-op effect: `identity_op(value)` yields `value` and
	/// resumes the continuation with it unchanged, so after interpretation the
	/// program continues with `value`. It is the target the `Interpose` oracle
	/// rewrites.
	#[handler_state(none)]
	#[crate_path(crate)]
	pub(crate) effect Identity {
		/// Yield `value` and resume with it unchanged.
		fn identity_op(value: i32) -> i32;
	}
}

#[cfg(test)]
mod tests {
	use crate::types::effects::fs1::{
		Fixture,
		identity_op,
		run,
	};

	// Behaviour-parity oracle bucket A (single-effect): the echoing `Identity`
	// effect yields its carried value and resumes with it, so the program
	// reduces to that value.
	#[test]
	fn identity_yields_its_value() {
		let fx = Fixture::new();
		assert_eq!(run(identity_op(7), &fx.handlers()), Ok(7));
	}
}
