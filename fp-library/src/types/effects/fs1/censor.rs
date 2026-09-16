//! FS-1 slice: the `Censor` higher-order effect.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition and its smart constructor are emitted by
//! `fp_macros::define_effect!` from the operation signature below, and the
//! module keeps its bucket A parity tests. `Censor` owns a sub-program and a
//! log transform; the parent interpreter elaborates it by giving the action a
//! fresh local log, applying the transform, and emitting the result to the
//! outer log (no boundary frame). The censor is transactional on abort: if the
//! action aborts, the local log is dropped and nothing reaches the outer log,
//! because a censor is a listen-shaped boundary whose transform-and-tell never
//! happens for an aborted action; writes outside a censor survive an abort. The
//! transform payload is named `f`: it is the built-in that proves the macro's
//! higher-order `Functor` arm rebinds payloads positionally, so a payload named
//! `f` does not shadow the emission's map-function parameter.

fp_macros::define_effect! {
	/// Censor is a higher-order effect: it owns an action sub-program and a
	/// transform `f` applied to the log the action produces. The interpreter
	/// elaborates it by giving the action a fresh local log, applying `f` to the
	/// total, and emitting the result to the outer log, so the censor scopes the
	/// accumulation (no boundary frame). The cell is generic in the log type `W`
	/// and the action result `RAction`; this slice's `Row` pins them to `String`
	/// and `()`.
	#[handler_state(none)]
	#[crate_path(crate)]
	pub(crate) effect Censor<W: 'static, RAction: 'static> {
		/// Run `action`, transform the log it produced with `f`, and emit the
		/// result; the action's value threads to the continuation.
		fn censor(f: impl FnOnce(W) -> W, action: Program<RAction>) -> RAction;
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
			censor,
			run,
			tell,
			throw,
		},
	};

	// Behaviour-parity oracle bucket A: Writer post-censor.
	// `censor(f, tell("Hello") >> tell(" world!"))` with `f(total) = total + "!"`
	// yields the log `"Hello world!!"`: the action's tells accumulate in the
	// censor's local log, then `f` is applied to the total and emitted.
	#[test]
	fn censor_transforms_the_accumulated_log() {
		let program: Free<Row, ()> = censor(
			|total| format!("{total}!"),
			tell("Hello".to_string()).bind(|()| tell(" world!".to_string())),
		);

		let fx = Fixture::new();
		let result = run(program, &fx.handlers());

		assert_eq!(result, Ok(()));
		assert_eq!(*fx.log.borrow(), "Hello world!!");
	}

	// Transactional on abort: writes made inside a censored action before an
	// abort are dropped with the local log (the transform-and-tell never
	// happens), while a write made outside the censor survives. Pins the
	// listen-shaped boundary semantics.
	#[test]
	fn censor_drops_the_local_log_when_the_action_aborts() {
		let program: Free<Row, ()> = tell("outer".to_string()).bind(|()| {
			censor(
				|total| format!("{total}!"),
				tell("inner".to_string()).bind(|()| throw::<(), _, _, _>()),
			)
		});

		let fx = Fixture::new();
		let result = run(program, &fx.handlers());

		assert_eq!(result, Err(Abort::Throw));
		assert_eq!(*fx.log.borrow(), "outer");
	}
}
