//! FS-1 slice: the `Writer` effect over a `String` log.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition and its smart constructor are emitted by
//! `fp_macros::define_effect!` from the operation signature below, and the
//! module keeps its bucket A parity test.

fp_macros::define_effect! {
	/// Writer over a `String` log. `tell` appends to the log. The accumulator
	/// is append-only: a handler never rewrites or truncates earlier writes,
	/// which is what makes `Listen`'s observed-delta slicing (the tail the
	/// action appended) sound.
	#[handler_state(shared_by_reference)]
	#[crate_path(crate)]
	pub(crate) effect Writer {
		/// Append `value` to the log.
		fn tell(value: String) -> ();
	}
}

#[cfg(test)]
mod tests {
	use crate::types::effects::fs1::{
		Fixture,
		run,
		tell,
	};

	// Behaviour-parity oracle bucket A (single-effect): successive `tell`s
	// accumulate into the `Writer` log in order.
	#[test]
	fn tell_accumulates_the_log_in_order() {
		let program = tell("Hello".to_string()).bind(|()| tell(" world!".to_string()));
		let fx = Fixture::new();
		assert_eq!(run(program, &fx.handlers()), Ok(()));
		assert_eq!(*fx.log.borrow(), "Hello world!");
	}
}
