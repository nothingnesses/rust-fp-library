//! FS-1 slice: the `Input` effect (drain a supplied queue of values).
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition and its smart constructor are emitted by
//! `fp_macros::define_effect!` from the operation signature below, and the
//! module keeps its bucket A parity test.

fp_macros::define_effect! {
	/// Input over a queue of `&'static str` values. `input` reads the next
	/// value, resuming with `Some(value)` while values remain and `None` after
	/// the queue is drained.
	#[handler_state(shared_by_reference)]
	#[crate_path(crate)]
	pub(crate) effect Input {
		/// Read the next queued value.
		fn input() -> Option<&'static str>;
	}
}

#[cfg(test)]
mod tests {
	use crate::types::effects::fs1::{
		Fixture,
		input,
		run,
	};

	// Behaviour-parity oracle bucket A: `input()` drains the handler's supplied
	// queue, yielding `Some(value)` per value then `None` once exhausted. Three
	// `input()` calls over `["red", "blue"]` yield `(Some("red"), Some("blue"),
	// None)`, matching the dual-row sequence runner.
	#[test]
	fn input_drains_the_queue_then_yields_none() {
		let program = input().bind(|first| {
			input().bind(move |second| input().map(move |third| (first, second, third)))
		});
		let fx = Fixture::with_input(["red", "blue"]);
		assert_eq!(run(program, &fx.handlers()), Ok((Some("red"), Some("blue"), None)));
	}
}
