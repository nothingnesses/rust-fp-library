//! Build-and-run proof that `fp_macros::define_effect!` emits working effect
//! definitions.
//!
//! Three macro invocations cover every current emission path: `State` (two
//! first-order operations with distinct resume types and an effect type
//! parameter), `Throw` (a no-resume operation storing `PhantomData` instead
//! of a continuation), and `Catch` (a higher-order operation with a
//! sub-program payload, a program-returning callable, and a value-threading
//! continuation, so its brand carries the row type parameter).
//!
//! The row over the emitted effects is a nominal brand (`PocRow`), not a
//! type alias: `Catch`'s cell stores `Free<PocRow, _>` sub-programs, so the
//! row must name itself, and a self-referencing type alias is a definition
//! cycle while the same self-reference through a nominal brand's kind
//! projection is lazy and legal. The nominal brand delegates `Functor` and
//! `WrapDrop` to the coproduct chain it projects to; that delegation is what
//! a row-assembly macro automates for the public surface.
//!
//! The interpreter is hand-written against the emitted operations enums,
//! exactly as row interpreters are: brand-keyed `uninject` dispatch with
//! total matches over each effect's operations, and elaboration of the
//! higher-order `Catch` by recursive interpretation sharing the state cell
//! (a write before a caught throw survives).

use {
	crate::{
		Apply,
		brands::{
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
		},
		classes::{
			Functor,
			WrapDrop,
		},
		impl_kind,
		kinds::*,
		types::{
			Coyoneda,
			Free,
		},
	},
	std::cell::Cell,
};

fp_macros::define_effect! {
	/// State over a cell of `S`: `get` reads it, `put` writes it.
	#[handler_state(shared_by_reference)]
	#[crate_path(crate)]
	pub(crate) effect State<S: 'static> {
		/// Read the current state.
		fn get() -> S;
		/// Write the state.
		fn put(value: S) -> ();
	}
}

fp_macros::define_effect! {
	/// A bare abort: the program stops with no payload and no continuation.
	#[handler_state(none)]
	#[crate_path(crate)]
	pub(crate) effect Throw {
		/// Abort the current program.
		fn throw() -> !;
	}
}

fp_macros::define_effect! {
	/// Run `action`; if it aborts with a bare throw, run `recover()` instead.
	/// The action's value (or the recovery's) threads to the continuation.
	#[handler_state(none)]
	#[crate_path(crate)]
	pub(crate) effect Catch<RAction: 'static> {
		/// Run `action`, recovering a bare throw with `recover`.
		fn catch(action: Program<RAction>, recover: impl FnOnce() -> Program<RAction>) -> RAction;
	}
}

/// The nominal row over the three emitted effects. A brand struct whose kind
/// projection maps to the coproduct chain: the self-reference inside
/// `CatchBrand<PocRow, ()>` sits behind the projection, so it is lazy where
/// a type alias would be a cycle.
pub(crate) struct PocRow;

/// The row's commitment of `Catch`'s parameters to concrete types (this row
/// and a unit action result), the parameterise-and-pin convention: stated
/// once and reused by the chain and the dispatch arm.
type CatchPinned = CatchBrand<PocRow, ()>;

/// The projected chain, spelled once.
type PocChain = CoproductBrand<
	CoyonedaBrand<StateBrand<bool>>,
	CoproductBrand<
		CoyonedaBrand<ThrowBrand>,
		CoproductBrand<CoyonedaBrand<CatchPinned>, CNilBrand>,
	>,
>;

impl_kind! {
	impl for PocRow {
		type Of<'a, A: 'a>: 'a = Apply!(<PocChain as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl Functor for PocRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<PocChain as Functor>::map(f, fa)
	}
}

impl WrapDrop for PocRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<PocChain as WrapDrop>::drop(fa)
	}
}

/// The interpreter's abort channel: the only aborting effect in this row is
/// the bare `Throw`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PocAbort {
	/// A bare `Throw`, recoverable by the `Catch` elaboration.
	Throw,
}

/// Interpret a program over [`PocRow`], with the `State` cell as the only
/// handler state. `Catch` is elaborated by recursive interpretation under
/// the same state cell, recovering a bare throw only.
pub(crate) fn run<A: 'static>(
	program: Free<PocRow, A>,
	state: &Cell<bool>,
) -> Result<A, PocAbort> {
	let mut program = program;
	loop {
		let layer = match program.resume() {
			Ok(value) => return Ok(value),
			Err(layer) => layer,
		};
		let selected: Result<Coyoneda<'static, StateBrand<bool>, Free<PocRow, A>>, _> =
			layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				program = match coyo.lower() {
					StateF::Get(k) => k(state.get()),
					StateF::Put(value, k) => {
						state.set(value);
						k(())
					}
				};
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, ThrowBrand, Free<PocRow, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(_throw) => return Err(PocAbort::Throw),
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, CatchPinned, Free<PocRow, A>>, _> = layer.uninject();
		let remainder = match selected {
			Ok(coyo) => {
				let CatchF::Catch {
					action,
					recover,
					k,
				} = coyo.lower();
				// The action's value (or the recovery's) threads to the
				// continuation; this row pins the catch result to unit, but
				// the value-threading shape is the general elaboration.
				#[expect(
					clippy::unit_arg,
					reason = "The continuation input is the action's (or recovery's) value; this row pins the catch result type to unit, but the value-threading shape is the general elaboration."
				)]
				{
					program = k(match run(action, state) {
						Ok(value) => value,
						Err(PocAbort::Throw) => run(recover(), state)?,
					});
				}
				continue;
			}
			Err(rest) => rest,
		};
		// Every brand in the row has been peeled, so the remainder is the
		// uninhabited terminal row.
		match remainder {}
	}
}

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::types::effects::fs1::{
			FirstOrder,
			HigherOrder,
			OrderOf,
		},
	};

	// Type-level probes: each instantiates only when the brand's computed
	// order marker matches, so the test body is the assertion.
	fn classify_first_order<E: OrderOf<Order = FirstOrder>>() {}
	fn classify_higher_order<E: OrderOf<Order = HigherOrder>>() {}

	// A write before a caught throw survives, and the catch's recovery value
	// threads onward: the emitted `State`, `Throw`, and `Catch` definitions
	// compose into a row and interpret with the shared-cell elaboration
	// semantics.
	#[test]
	fn state_write_survives_a_caught_throw_on_emitted_effects() {
		let program: Free<PocRow, bool> =
			catch(put(true).bind(|()| throw()), || Free::pure(())).bind(|()| get());

		let state = Cell::new(false);
		let result = run(program, &state);

		assert_eq!(result, Ok(true));
		assert!(state.get());
	}

	// A throw with no enclosing catch aborts the whole program.
	#[test]
	fn an_uncaught_throw_aborts() {
		let program: Free<PocRow, bool> = put(true).bind(|()| throw());

		let state = Cell::new(false);
		assert_eq!(run(program, &state), Err(PocAbort::Throw));
		assert!(state.get());
	}

	// A catch whose action succeeds never runs the recovery.
	#[test]
	fn a_successful_action_skips_the_recovery() {
		let program: Free<PocRow, bool> = catch(put(true), || put(false)).bind(|()| get());

		let state = Cell::new(false);
		assert_eq!(run(program, &state), Ok(true));
	}

	// The order markers are computed from the operation shapes: `State` (no
	// sub-program payload) is first-order, `Catch` (a `Program` payload) is
	// higher-order.
	#[test]
	fn order_markers_derive_from_the_operation_shapes() {
		classify_first_order::<StateBrand<bool>>();
		classify_higher_order::<CatchBrand<PocRow, ()>>();
	}
}
