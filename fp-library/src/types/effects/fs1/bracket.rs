//! FS-1 slice: the `Bracket` higher-order effect.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition and its smart constructor are emitted by
//! `fp_macros::define_effect!` from the operation signature below, and the
//! module keeps its bucket A parity tests. `Bracket` owns three sub-programs
//! (`acquire`, `body`, `release`) and is elaborated by the parent interpreter,
//! which runs them in order under the same `Handlers`, threading the acquired
//! resource as a plain value (no boundary frame). A body abort still releases
//! and then propagates (the body's abort taking priority over one raised by
//! `release` during that unwind); an acquire abort skips both body and release,
//! since no resource exists yet. The emitted brand carries the row type
//! parameter and the parent's `Row` pins it.

fp_macros::define_effect! {
	/// Bracket is a higher-order effect: it owns an `acquire` sub-program
	/// yielding a resource, a `body` callable that uses the resource, and a
	/// `release` callable that cleans it up. The interpreter elaborates it by
	/// running the three under the same `Handlers` in order (`acquire` then
	/// `body` then `release`), threading the resource as a plain value, with the
	/// cell's result equal to the body result. The resource generic is `Res`
	/// (the name `R` is reserved for the emitted row parameter). A resource that
	/// flows into more than one callable must be duplicable by the elaborator;
	/// this slice pins `Res` to `i32` (`Copy`), the trivial case of the
	/// `Res: Clone` ownership rule, with `release` receiving the copy. The cell
	/// is generic in the resource type `Res` and the body result `RBody`; this
	/// slice's `Row` pins both to `i32`.
	#[handler_state(none)]
	#[crate_path(crate)]
	pub(crate) effect Bracket<Res: 'static, RBody: 'static> {
		/// Acquire a resource, use it with `body`, then clean up with `release`;
		/// the body's value threads to the continuation.
		fn bracket(
			acquire: Program<Res>,
			body: impl FnOnce(Res) -> Program<RBody>,
			release: impl FnOnce(Res) -> Program<()>,
		) -> RBody;
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
			bracket,
			run,
			tell,
			throw,
		},
	};

	// Behaviour-parity oracle bucket A: Bracket acquire/body/release ordering.
	// `acquire` tells "acquire" and yields the resource `7`; `body` tells "body"
	// and yields `resource + 35`; `release` tells "release". The three run in
	// order under the same handlers, so the result is `42` (7 + 35) and the final
	// `Writer` log is `"acquirebodyrelease"`.
	#[test]
	fn bracket_runs_acquire_body_release_in_order() {
		let program: Free<Row, i32> = bracket(
			tell("acquire".to_string()).map(|()| 7),
			|resource| tell("body".to_string()).map(move |()| resource + 35),
			|_resource| tell("release".to_string()),
		);

		let fx = Fixture::new();
		let result = run(program, &fx.handlers());

		assert_eq!(result, Ok(42));
		assert_eq!(*fx.log.borrow(), "acquirebodyrelease");
	}

	// Release-on-abort: a body that aborts still releases the resource, and the
	// body's abort propagates. The log shows acquire and release around the
	// aborted body's write.
	#[test]
	fn bracket_releases_when_the_body_aborts() {
		let program: Free<Row, i32> = bracket(
			tell("acquire".to_string()).map(|()| 7),
			|_resource| tell("body".to_string()).bind(|()| throw::<i32, _, _, _>()),
			|_resource| tell("release".to_string()),
		);

		let fx = Fixture::new();
		let result = run(program, &fx.handlers());

		assert_eq!(result, Err(Abort::Throw));
		assert_eq!(*fx.log.borrow(), "acquirebodyrelease");
	}
}
