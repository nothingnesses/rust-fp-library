//! FS-1 slice: the `Reader` effect over an `i32` environment.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition and its smart constructor are emitted by
//! `fp_macros::define_effect!` from the operation signature below, and the
//! module keeps its bucket A parity test. The parity case is a higher-order
//! composition (Reader feeding State under a Catch), so the test imports the
//! sibling constructors it composes with.

fp_macros::define_effect! {
	/// Reader over an `i32` environment. `ask` reads the environment, resuming
	/// the continuation with it.
	#[handler_state(scoped_by_value)]
	#[crate_path(crate)]
	pub(crate) effect Reader {
		/// Read the current environment.
		fn ask() -> i32;
	}
}

#[cfg(test)]
mod tests {
	use crate::types::{
		Free,
		effects::fs1::{
			Fixture,
			Row,
			ask,
			catch,
			get,
			put,
			run,
			throw,
		},
	};

	// Behaviour-parity oracle bucket A: Reader composes with State and Catch.
	// `ask()` supplies the environment, which is written into State (as its
	// parity), and a caught throw leaves the write intact.
	#[test]
	fn reader_composes_with_state_and_catch() {
		let program: Free<Row, bool> = ask().bind(|env| {
			let parity = env % 2 == 0;
			catch(put(parity).bind(|()| throw::<(), _, _>()), || Free::pure(())).bind(|()| get())
		});

		// env = 4 is even, so the State write is `true` and survives the catch.
		let fx = Fixture::with_env(4);
		let result = run(program, &fx.handlers());

		assert_eq!(result, Ok(true));
		assert!(fx.state.get());
	}
}
