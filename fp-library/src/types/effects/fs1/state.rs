//! FS-1 slice: the `State` effect (pinned to a `bool` cell at this slice's
//! `Row`).
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition (brand, functor, order marker) and its smart constructors are
//! emitted by `fp_macros::define_effect!` from the operation signatures
//! below, and the module keeps its bucket A parity test. The only shared
//! surfaces it touches are the parent's `Row` (one cell) and interpreter
//! (one dispatch arm plus a `Handlers` field), both append-only.

fp_macros::define_effect! {
	/// State over a cell of `S`. `Get` reads the current state; `Put` writes it.
	#[handler_state(shared_by_reference)]
	#[crate_path(crate)]
	pub(crate) effect State<S: 'static> {
		/// Read the current state.
		fn get() -> S;
		/// Write the state.
		fn put(value: S) -> ();
	}
}

#[cfg(test)]
mod tests {
	use crate::types::effects::fs1::{
		Fixture,
		get,
		put,
		run,
	};

	// Behaviour-parity oracle bucket A (single-effect): a `put` then `get` reads
	// back the written value and leaves the state cell holding it.
	#[test]
	fn put_then_get_reads_the_written_state() {
		let program = put(true).bind(|()| get());
		let fx = Fixture::new();
		assert_eq!(run(program, &fx.handlers()), Ok(true));
		assert!(fx.state.get());
	}
}
