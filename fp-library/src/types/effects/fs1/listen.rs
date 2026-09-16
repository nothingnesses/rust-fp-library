//! FS-1 slice: the `Listen` higher-order effect.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition and its smart constructor are emitted by
//! `fp_macros::define_effect!` from the operation signature below, and the
//! module keeps its bucket A parity test. `Listen` owns a sub-program and is
//! elaborated by the parent interpreter, which runs the action under the SAME
//! `Handlers` (so the action's writes land in the same `log` and are
//! PRESERVED), capturing the log delta the action produced and pairing it with
//! the action's value, rather than using a boundary frame. The emitted brand
//! carries the row type parameter and the parent's `Row` pins it.

fp_macros::define_effect! {
	/// Listen is a higher-order effect: it owns an action sub-program and
	/// observes the `Writer` log the action produces. The interpreter elaborates
	/// it by running the action under the same `Handlers` (so the action's
	/// writes are preserved into the outer log), capturing the log delta and
	/// resuming with `(value, observed)`, no boundary frame, in contrast with
	/// `Censor`, which replaces the log with a fresh local one. The cell is
	/// generic in the action result `RAction` and the log type `W`; this slice's
	/// `Row` pins them to `i32` and `String`.
	#[handler_state(none)]
	#[crate_path(crate)]
	pub(crate) effect Listen<RAction: 'static, W: 'static> {
		/// Run `action`, resuming with its value paired with the log it produced.
		fn listen(action: Program<RAction>) -> (RAction, W);
	}
}

#[cfg(test)]
mod tests {
	use crate::types::{
		Free,
		effects::fs1::{
			Fixture,
			Row,
			listen,
			run,
			tell,
		},
	};

	// Behaviour-parity oracle bucket A: Writer listen observes and preserves.
	// `listen(tell("first") >> tell("second") >> pure(40))` observes the log the
	// action emitted (`"firstsecond"`) and pairs it with the value, while
	// preserving those writes in the outer log; the outer `tell("outer")` then
	// appends, so the final log is `"firstsecondouter"`. The result is
	// `(40 + 2, "firstsecond")`.
	#[test]
	fn listen_observes_and_preserves_the_action_log() {
		let action = tell("first".to_string()).bind(|()| tell("second".to_string())).map(|()| 40);
		let program: Free<Row, (i32, String)> = listen(action).bind(|(value, observed)| {
			tell("outer".to_string()).map(move |()| (value + 2, observed))
		});

		let fx = Fixture::new();
		let result = run(program, &fx.handlers());

		assert_eq!(result, Ok((42, "firstsecond".to_string())));
		assert_eq!(*fx.log.borrow(), "firstsecondouter");
	}
}
