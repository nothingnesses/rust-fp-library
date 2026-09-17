//! FS-1 slice: the `Empty` effect (abort the current branch without a value).
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition and its smart constructor are emitted by
//! `fp_macros::define_effect!` from the operation signature below, and the
//! module keeps its bucket A parity test. `empty` is a no-resume (`-> !`)
//! operation, so the emitted variant stores `PhantomData` instead of a
//! continuation. In this single-shot slice the abort surfaces as
//! `Err(Abort::Empty)` from the interpreter, which a caller reads as `None`
//! or replaces with a fallback; it propagates through a `catch` (which
//! recovers `Throw` only). Empty's distinctive pruning of nondeterministic
//! branches needs the multi-shot substrate and is interpreted there.

fp_macros::define_effect! {
	/// Empty aborts the current branch without producing a value: it never
	/// returns, so it stands in any result position, and the interpreter
	/// surfaces the abort as `Err(Abort::Empty)`.
	#[handler_state(none)]
	#[crate_path(crate)]
	pub(crate) effect Empty {
		/// Abort the current branch with no value.
		fn empty() -> !;
	}
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
		assert_eq!(run(empty::<i32, _, _, _>(), &none_fx.handlers()).ok(), None);

		let fallback_fx = Fixture::new();
		assert_eq!(run(empty::<i32, _, _, _>(), &fallback_fx.handlers()).unwrap_or(0), 0);
	}
}
