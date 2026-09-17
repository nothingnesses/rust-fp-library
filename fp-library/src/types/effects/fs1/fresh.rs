//! FS-1 slice: the `Fresh` effect (a monotonic counter).
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition and its smart constructor are emitted by
//! `fp_macros::define_effect!` from the operation signature below, and the
//! module keeps its bucket A parity test.

fp_macros::define_effect! {
	/// Fresh generates a monotonically increasing counter value. `fresh`
	/// requests the next value and resumes with it; the interpreter threads the
	/// counter.
	#[handler_state(shared_by_reference)]
	#[crate_path(crate)]
	pub(crate) effect Fresh {
		/// Request the next counter value.
		fn fresh() -> usize;
	}
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
