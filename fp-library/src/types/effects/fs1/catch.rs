//! FS-1 slice: the `Catch` higher-order effect.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition and its smart constructor are emitted by
//! `fp_macros::define_effect!` from the operation signature below, and the
//! module keeps its bucket A parity tests. `Catch` owns a sub-program, so
//! the emitted brand carries the row type parameter and the parent's `Row`
//! pins it (the parameterise-and-pin convention); the parent interpreter
//! elaborates the cell, sharing the `State` cell across the recursive call
//! (so a write before a caught throw survives) rather than using a boundary
//! frame.

fp_macros::define_effect! {
	/// Catch is a higher-order effect: it owns an action sub-program and a
	/// recovery thunk, its result equal to the action result `RAction`. The
	/// interpreter elaborates it into a sub-interpretation over `Throw`,
	/// sharing the `State` cell (so writes before a caught throw survive). A
	/// catch recovers the bare `Throw` abort only: an `Empty` dead branch and
	/// a typed `Except` throw are different effects with their own boundaries
	/// and propagate through it.
	#[handler_state(none)]
	#[crate_path(crate)]
	pub(crate) effect Catch<RAction: 'static> {
		/// Run `action`, recovering a bare throw with `recover`; the action's
		/// value (or the recovery's) threads to the continuation.
		fn catch(action: Program<RAction>, recover: impl FnOnce() -> Program<RAction>) -> RAction;
	}
}

#[cfg(test)]
mod tests {
	use crate::types::{
		Free,
		effects::fs1::{
			Abort,
			Fixture,
			Row,
			catch,
			empty,
			get,
			put,
			run,
			run_except,
			throw,
			throw_e,
		},
	};

	// Behaviour-parity oracle bucket A: State-with-Catch ordering.
	// `catch(put(true) >> throw, recover = pure(()))` then `get` yields value
	// `true` and final state `true`. The write before the caught throw survives
	// because the interpreter shares the state cell across the catch (no boundary
	// frame, no rollback).
	#[test]
	fn state_write_survives_caught_throw() {
		let program: Free<Row, bool> =
			catch(put(true).bind(|()| throw::<()>()), || Free::pure(())).bind(|()| get());

		let fx = Fixture::new();
		let result = run(program, &fx.handlers());

		assert_eq!(result, Ok(true));
		assert!(fx.state.get());
	}

	// A catch is selective: a typed `Except` throw inside the caught action is
	// not recovered by the catch; it propagates with its payload intact, so an
	// enclosing `run_except` still receives the error value.
	#[test]
	fn typed_except_propagates_through_catch_to_run_except() {
		let program: Free<Row, i32> =
			catch(throw_e::<()>("boom"), || Free::pure(())).bind(|()| Free::<Row, i32>::pure(1));

		let fx = Fixture::new();
		let recovered = run_except(program, &fx.handlers(), |e| {
			assert_eq!(e, "boom");
			Free::pure(42)
		});

		assert_eq!(recovered, Ok(42));
	}

	// A catch is selective: an `Empty` dead branch inside the caught action is
	// not recovered by the catch; the abort propagates as `Abort::Empty`.
	#[test]
	fn empty_propagates_through_catch() {
		let program: Free<Row, i32> =
			catch(empty::<()>(), || Free::pure(())).bind(|()| Free::<Row, i32>::pure(1));

		let fx = Fixture::new();
		assert_eq!(run(program, &fx.handlers()), Err(Abort::Empty));
	}
}
