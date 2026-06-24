//! FS-1 effects rebuild, crate-internal work in progress.
//!
//! This module is the in-tree vertical slice of the unified-row effects
//! rebuild (remediation-plan review-2, item 4). It is deliberately
//! `pub(crate)` and not part of the public surface: per the adopted hybrid
//! method, the new substrate is built here to a compiling, test-backed state
//! before the dual-row subsystem it replaces is deleted, so a half-built
//! rewrite never ships.
//!
//! FS-1 replaces the dual rows (`Run<R, S, A>`) with one unified row of effect
//! brands and elaborates higher-order effects into first-order ones over that
//! row, rather than using boundary frames. This slice carries four first-order
//! effects (`State`, `Throw`, `Reader`, `Writer`) and two higher-order effects
//! (`Catch`, `Censor`) as in-row cells in one `Coyoneda`-wrapped
//! `CoproductBrand` row, interpreted by one pass that elaborates the
//! higher-order cells. It reproduces the behaviour-parity oracle's bucket-A
//! cases: State-with-Catch ordering (a write before a caught throw survives),
//! Writer post-censor (`"Hello world!!"`), and a Reader + State + Catch
//! composition.
//!
//! Higher-order semantics fall out of how the interpreter shares or scopes its
//! accumulators at the recursive call: `Catch` shares the `State` cell (so the
//! write survives), while `Censor` gives its action a fresh local log (so the
//! censor scopes the accumulation), with no boundary frames.
//!
//! Scope of this slice: the substrate is the existing public `Free` (the
//! `Store = Box`, erased, `'static` form, reused per the POC-11 substrate
//! decision); the `Store`-parameterised Rc/Arc forms and the concrete
//! (non-`'static`) form are folded in later. The interpreter dispatches each
//! active row arm by its effect brand (type-directed selection over the
//! coproduct), not by the arm's position in the row, so the dispatch arms may
//! be written in any order and need not track the row's declared order; this is
//! the brand-keyed dispatch that removes the positional-sort footgun (item 8's
//! mechanism). Per-brand order markers and the order-directed peel classify
//! whether an active arm is first-order or higher-order; they are exercised by
//! the order-classification test and become the routing layer when elaboration
//! is generalised over the row.
//!
//! Documentation status: this module intentionally does NOT yet use the
//! `#[fp_macros::document_module]` wrapper that the rest of `fp-library/src/`
//! uses. The effects here are hand-written placeholders that the FS-1
//! `define_effect!` macro (remediation item 11) will regenerate (the way
//! `state.rs` and the other shipped effects are already generated), so
//! hand-documenting them now would be throwaway: `document_module` requires
//! signature/type-parameter/parameter/return/example attributes with runnable
//! doctests on every method. The wrapper and full per-item documentation are
//! added once the FS-1 macro and the remaining prerequisites (the code the
//! production tests need) exist; until then this is a tracked, temporary
//! exception, not an oversight.

#![allow(
	dead_code,
	reason = "FS-1 rebuild in progress (item 4): these items form the vertical slice and are currently exercised only by this module's tests; the public surface that consumes them is added in later steps, and item 20 sweeps any residual allowances at the end of the rebuild."
)]

use {
	crate::{
		Apply,
		brands::{
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
		},
		classes::Functor,
		impl_kind,
		kinds::*,
		types::{
			Coyoneda,
			Free,
			effects::coproduct::{
				CNil,
				Coproduct,
			},
		},
	},
	std::{
		cell::{
			Cell,
			RefCell,
		},
		marker::PhantomData,
		rc::Rc,
	},
};

// -- First-order effects --

/// State over a `bool` cell. `Get` reads the current state; `Put` writes it.
pub(crate) struct StateBrand;
pub(crate) enum StateF<'a, A> {
	Get(Box<dyn FnOnce(bool) -> A + 'a>),
	Put(bool, Box<dyn FnOnce(()) -> A + 'a>),
}
impl_kind! {
	impl for StateBrand {
		type Of<'a, A: 'a>: 'a = StateF<'a, A>;
	}
}
impl Functor for StateBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			StateF::Get(k) => StateF::Get(Box::new(move |s| f(k(s)))),
			StateF::Put(s, k) => StateF::Put(s, Box::new(move |u| f(k(u)))),
		}
	}
}

/// Throw with a unit error. The result type is phantom: a throw never returns,
/// so it can stand in any result position.
pub(crate) struct ThrowBrand;
pub(crate) struct ThrowF<A>(PhantomData<A>);
impl_kind! {
	impl for ThrowBrand {
		type Of<'a, A: 'a>: 'a = ThrowF<A>;
	}
}
impl Functor for ThrowBrand {
	fn map<'a, A: 'a, B: 'a>(
		_f: impl Fn(A) -> B + 'a,
		_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		ThrowF(PhantomData)
	}
}

/// Reader over an `i32` environment. `Ask` reads the environment.
pub(crate) struct ReaderBrand;
pub(crate) enum ReaderF<'a, A> {
	Ask(Box<dyn FnOnce(i32) -> A + 'a>),
}
impl_kind! {
	impl for ReaderBrand {
		type Of<'a, A: 'a>: 'a = ReaderF<'a, A>;
	}
}
impl Functor for ReaderBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			ReaderF::Ask(k) => ReaderF::Ask(Box::new(move |e| f(k(e)))),
		}
	}
}

/// Writer over a `String` log. `Tell` appends to the log.
pub(crate) struct WriterBrand;
pub(crate) enum WriterF<'a, A> {
	Tell(String, Box<dyn FnOnce(()) -> A + 'a>),
}
impl_kind! {
	impl for WriterBrand {
		type Of<'a, A: 'a>: 'a = WriterF<'a, A>;
	}
}
impl Functor for WriterBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			WriterF::Tell(w, k) => WriterF::Tell(w, Box::new(move |u| f(k(u)))),
		}
	}
}

// -- Higher-order effects as in-row cells --

/// Catch is a higher-order effect: it owns an action sub-program and a recovery
/// thunk, its result equal to the action result `RAction`. The interpreter
/// elaborates it into a sub-interpretation over `Throw`, sharing the `State`
/// cell (so writes before a caught throw survive), rather than using a boundary
/// frame.
pub(crate) struct CatchBrand<RAction>(PhantomData<RAction>);
pub(crate) struct CatchCell<'a, RAction: 'static, Next> {
	action: Free<Row, RAction>,
	recover: Rc<dyn Fn() -> Free<Row, RAction> + 'a>,
	k: Box<dyn FnOnce(RAction) -> Next + 'a>,
}
impl_kind! {
	impl<RAction: 'static> for CatchBrand<RAction> {
		type Of<'a, Next: 'a>: 'a = CatchCell<'a, RAction, Next>;
	}
}
impl<RAction: 'static> Functor for CatchBrand<RAction> {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		let CatchCell {
			action,
			recover,
			k,
		} = fa;
		CatchCell {
			action,
			recover,
			k: Box::new(move |a| f(k(a))),
		}
	}
}

/// Censor is a higher-order effect: it owns an action sub-program and a
/// transform `f` applied to the log the action produces. The interpreter
/// elaborates it by giving the action a fresh local log, applying `f` to the
/// total, and emitting the result to the outer log, so the censor scopes the
/// accumulation (no boundary frame).
pub(crate) struct CensorBrand;
pub(crate) struct CensorCell<'a, Next> {
	f: Rc<dyn Fn(String) -> String + 'a>,
	action: Free<Row, ()>,
	k: Box<dyn FnOnce(()) -> Next + 'a>,
}
impl_kind! {
	impl for CensorBrand {
		type Of<'a, Next: 'a>: 'a = CensorCell<'a, Next>;
	}
}
impl Functor for CensorBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		let CensorCell {
			f: transform,
			action,
			k,
		} = fa;
		CensorCell {
			f: transform,
			action,
			k: Box::new(move |u| f(k(u))),
		}
	}
}

// -- Per-brand order markers and the order-directed peel --

/// First-order order marker: the effect's representation does not depend on the
/// carrier (no sub-program in a negative position).
pub(crate) struct FirstOrder;
/// Higher-order order marker: the effect owns a sub-program (it is elaborated).
pub(crate) struct HigherOrder;

/// Each effect brand carries its order as an associated marker. This is the
/// unified row's classification: first-order and higher-order effects live in
/// the same row and are told apart by this marker, not by a separate row.
pub(crate) trait OrderOf {
	type Order;
}
impl OrderOf for StateBrand {
	type Order = FirstOrder;
}
impl OrderOf for ThrowBrand {
	type Order = FirstOrder;
}
impl OrderOf for ReaderBrand {
	type Order = FirstOrder;
}
impl OrderOf for WriterBrand {
	type Order = FirstOrder;
}
impl<RAction> OrderOf for CatchBrand<RAction> {
	type Order = HigherOrder;
}
impl OrderOf for CensorBrand {
	type Order = HigherOrder;
}

/// The runtime reflection of an order marker, so an interpreter can branch on
/// the active arm's order (the order-directed peel).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum OrderTag {
	First,
	Higher,
}
trait OrderTagged {
	fn tag() -> OrderTag;
}
impl OrderTagged for FirstOrder {
	fn tag() -> OrderTag {
		OrderTag::First
	}
}
impl OrderTagged for HigherOrder {
	fn tag() -> OrderTag {
		OrderTag::Higher
	}
}

/// A row cell exposes the order of its effect. For a `Coyoneda`-wrapped cell the
/// order is the wrapped brand's [`OrderOf::Order`].
trait CellOrder {
	type Order;
}
// `Kind_cdc7cd43dac7585f` is the macro-generated `Kind` trait for the
// `type Of<'a, T: 'a>: 'a` shape (from the `kinds` module); naming
// `Coyoneda<'a, E, _>` requires its brand `E` to satisfy it. This matches how
// the library's own generated impls reference the trait.
impl<'a, E: OrderOf + Kind_cdc7cd43dac7585f, A> CellOrder for Coyoneda<'a, E, A> {
	type Order = E::Order;
}

/// Classify the active arm of a suspended row layer by order, walking the
/// coproduct to the live cell. This is the order-directed peel (POC-3): one row
/// holds both kinds, and the interpreter reads the order off the active cell.
trait ClassifyActive {
	fn classify(&self) -> OrderTag;
}
impl ClassifyActive for CNil {
	fn classify(&self) -> OrderTag {
		match *self {}
	}
}
impl<Cell: CellOrder, Rest: ClassifyActive> ClassifyActive for Coproduct<Cell, Rest>
where
	Cell::Order: OrderTagged,
{
	fn classify(&self) -> OrderTag {
		match self {
			Coproduct::Inl(_) => <Cell::Order as OrderTagged>::tag(),
			Coproduct::Inr(rest) => rest.classify(),
		}
	}
}

// -- The unified row --

/// The unified effect row for this slice: one `Coyoneda`-wrapped cell per
/// effect, terminated by `CNilBrand`. All effects, first-order and higher-order
/// alike, live in this single row (the defining FS-1 property; the dual scoped
/// row is gone).
pub(crate) type Row = CoproductBrand<
	CoyonedaBrand<StateBrand>,
	CoproductBrand<
		CoyonedaBrand<ThrowBrand>,
		CoproductBrand<
			CoyonedaBrand<CatchBrand<()>>,
			CoproductBrand<
				CoyonedaBrand<ReaderBrand>,
				CoproductBrand<
					CoyonedaBrand<WriterBrand>,
					CoproductBrand<CoyonedaBrand<CensorBrand>, CNilBrand>,
				>,
			>,
		>,
	>,
>;

/// The row cell over a result `A`, as handed to [`Free::lift_f`] by the smart
/// constructors: one `Coyoneda`-wrapped operation at its coproduct position
/// whose hole is the operation's result `A`. (The interpreter's [`Free::resume`]
/// hands back the same row shape but with the hole instantiated to the
/// continuation `Free<Row, A>`; that shape is inferred in `run`, not named.)
type Node<A> = Apply!(<Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>);

// -- Smart constructors (inject at the right coproduct position) --

pub(crate) fn get() -> Free<Row, bool> {
	let coyo: Coyoneda<'static, StateBrand, bool> = Coyoneda::lift(StateF::Get(Box::new(|s| s)));
	Free::lift_f(Coproduct::Inl(coyo) as Node<bool>)
}
pub(crate) fn put(value: bool) -> Free<Row, ()> {
	let coyo: Coyoneda<'static, StateBrand, ()> =
		Coyoneda::lift(StateF::Put(value, Box::new(|u| u)));
	Free::lift_f(Coproduct::Inl(coyo) as Node<()>)
}
pub(crate) fn throw<A: 'static>() -> Free<Row, A> {
	let coyo: Coyoneda<'static, ThrowBrand, A> = Coyoneda::lift(ThrowF(PhantomData));
	Free::lift_f(Coproduct::Inr(Coproduct::Inl(coyo)) as Node<A>)
}
pub(crate) fn catch(
	action: Free<Row, ()>,
	recover: impl Fn() -> Free<Row, ()> + 'static,
) -> Free<Row, ()> {
	let cell: CatchCell<'static, (), ()> = CatchCell {
		action,
		recover: Rc::new(recover),
		k: Box::new(|a| a),
	};
	let coyo: Coyoneda<'static, CatchBrand<()>, ()> = Coyoneda::<CatchBrand<()>, _>::lift(cell);
	Free::lift_f(Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(coyo))) as Node<()>)
}
pub(crate) fn ask() -> Free<Row, i32> {
	let coyo: Coyoneda<'static, ReaderBrand, i32> = Coyoneda::lift(ReaderF::Ask(Box::new(|e| e)));
	Free::lift_f(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(coyo)))) as Node<i32>)
}
pub(crate) fn tell(w: String) -> Free<Row, ()> {
	let coyo: Coyoneda<'static, WriterBrand, ()> =
		Coyoneda::lift(WriterF::Tell(w, Box::new(|u| u)));
	Free::lift_f(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(
		coyo,
	))))) as Node<()>)
}
pub(crate) fn censor(
	f: impl Fn(String) -> String + 'static,
	action: Free<Row, ()>,
) -> Free<Row, ()> {
	let cell: CensorCell<'static, ()> = CensorCell {
		f: Rc::new(f),
		action,
		k: Box::new(|u| u),
	};
	let coyo: Coyoneda<'static, CensorBrand, ()> = Coyoneda::lift(cell);
	Free::lift_f(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(
		Coproduct::Inl(coyo),
	))))) as Node<()>)
}

// -- The interpreter: one pass, elaborating the higher-order cells --

/// Interpret a program over the unified row. `state` is the shared `State`
/// cell, `env` the `Reader` environment, `log` the `Writer` accumulator. A
/// `Throw` aborts to `Err(())`. `Catch` is elaborated by interpreting its action
/// over the same `state` (so writes survive a caught throw); `Censor` is
/// elaborated by interpreting its action over a fresh local log, then emitting
/// `f(total)` to the outer log. No boundary frames.
pub(crate) fn run<A: 'static>(
	program: Free<Row, A>,
	state: &Cell<bool>,
	env: i32,
	log: &RefCell<String>,
) -> Result<A, ()> {
	let mut program = program;
	loop {
		let layer = match program.resume() {
			Ok(value) => return Ok(value),
			Err(layer) => layer,
		};
		// Brand-keyed dispatch: select the active arm by its effect brand via
		// type-directed `uninject`, independent of the brand's position in the
		// row. The arms below are deliberately not in the row's declared order
		// (Reader and Writer are handled before Catch even though they sit after
		// it in `Row`), which is exactly the property that removes the
		// positional-sort footgun: dispatch correctness no longer depends on the
		// arm order matching the row order. Each `uninject` peels its brand's cell
		// out wherever it sits, yielding the active cell or the remaining row; the
		// chain bottoms out at the uninhabited terminal row.
		let selected: Result<Coyoneda<'static, StateBrand, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				program = match coyo.lower() {
					StateF::Get(k) => k(state.get()),
					StateF::Put(s, k) => {
						state.set(s);
						k(())
					}
				};
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, ThrowBrand, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(_throw) => return Err(()),
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, ReaderBrand, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				program = match coyo.lower() {
					ReaderF::Ask(k) => k(env),
				};
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, WriterBrand, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				match coyo.lower() {
					WriterF::Tell(w, k) => {
						log.borrow_mut().push_str(&w);
						program = k(());
					}
				}
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, CatchBrand<()>, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				let CatchCell {
					action,
					recover,
					k,
				} = coyo.lower();
				if run(action, state, env, log).is_err() {
					run(recover(), state, env, log)?;
				}
				program = k(());
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, CensorBrand, Free<Row, A>>, _> = layer.uninject();
		let remainder = match selected {
			Ok(coyo) => {
				let CensorCell {
					f,
					action,
					k,
				} = coyo.lower();
				let local = RefCell::new(String::new());
				run(action, state, env, &local)?;
				let censored = f(local.into_inner());
				log.borrow_mut().push_str(&censored);
				program = k(());
				continue;
			}
			Err(rest) => rest,
		};
		// Every brand in the row has been peeled, so the remainder is the
		// uninhabited terminal row: this point is unreachable for any program.
		match remainder {}
	}
}

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::types::{
			Coyoneda,
			Free,
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

		let state = Cell::new(false);
		let log = RefCell::new(String::new());
		let result = run(program, &state, 0, &log);

		assert_eq!(result, Ok(true));
		assert!(state.get());
	}

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

		let state = Cell::new(false);
		let log = RefCell::new(String::new());
		let result = run(program, &state, 0, &log);

		assert_eq!(result, Ok(()));
		assert_eq!(log.into_inner(), "Hello world!!");
	}

	// Behaviour-parity oracle bucket A: Reader composes with State and Catch.
	// `ask()` supplies the environment, which is written into State (as its
	// parity), and a caught throw leaves the write intact.
	#[test]
	fn reader_composes_with_state_and_catch() {
		let program: Free<Row, bool> = ask().bind(|env| {
			let parity = env % 2 == 0;
			catch(put(parity).bind(|()| throw::<()>()), || Free::pure(())).bind(|()| get())
		});

		let state = Cell::new(false);
		let log = RefCell::new(String::new());
		// env = 4 is even, so the State write is `true` and survives the catch.
		let result = run(program, &state, 4, &log);

		assert_eq!(result, Ok(true));
		assert!(state.get());
	}

	// Item 4 step 3: the order-directed peel classifies the active arm of a
	// suspended layer by order, over the one unified row. A `State` operation is
	// first-order; a `Catch` cell is higher-order.
	#[test]
	fn order_directed_peel_classifies_the_active_arm() {
		let state_layer = get().resume();
		assert!(state_layer.is_err());
		if let Err(layer) = state_layer {
			assert_eq!(layer.classify(), OrderTag::First);
		}

		let catch_layer = catch(Free::pure(()), || Free::pure(())).resume();
		assert!(catch_layer.is_err());
		if let Err(layer) = catch_layer {
			assert_eq!(layer.classify(), OrderTag::Higher);
		}
	}

	// Item 4 step 3: brand-keyed dispatch selects the active arm by effect brand,
	// not by its position in the row. `State` is the head arm of `Row` while
	// `Censor` is the tail arm; both are found by a type-directed `uninject` keyed
	// on the brand's cell, with the position inferred. This is the property that
	// makes the interpreter's dispatch-arm order independent of the row's declared
	// order, so the positional-sort footgun is gone.
	#[test]
	#[expect(
		clippy::expect_used,
		reason = "a suspended effect's `resume()` is `Err(layer)` by construction; `expect_err` extracts the layer this test then inspects, and a wrong `Ok` should fail the test loudly."
	)]
	fn brand_keyed_dispatch_selects_by_brand_not_position() {
		// `State` is the head arm of `Row`.
		let state_layer = get().resume().expect_err("a suspended Get is a layer");
		let head: Result<Coyoneda<'static, StateBrand, Free<Row, bool>>, _> =
			state_layer.uninject();
		assert!(head.is_ok(), "the head brand is found by brand-keyed selection");

		// `Censor` is the tail arm of `Row`; brand-keyed selection reaches it the
		// same way, without walking coproduct positions by hand.
		let censor_layer =
			censor(|s| s, Free::pure(())).resume().expect_err("a suspended Censor is a layer");
		let tail: Result<Coyoneda<'static, CensorBrand, Free<Row, ()>>, _> =
			censor_layer.uninject();
		assert!(tail.is_ok(), "the tail brand is found by brand-keyed selection");
	}
}
