//! POC-9 (foundation sweep, Tier F): integration slice composing several
//! effects and several higher-order effects over one unified row.
//!
//! Charter question: does the surviving FS-1 design carry a real slice
//! (State + Reader + Catch + Writer-censor) end-to-end on the unified row,
//! reproducing the heftia semantics cases? This is the capstone composition
//! test; the new evidence is composing multiple higher-order effects with
//! first-order effects in one row (no earlier POC did that). Adopted Approach
//! (isolated multi-HOE): the existing public `Free` (fixed `Box`) is the
//! substrate (POC-8's parameterised substrate is validated separately), and
//! both higher-order effects are in-row cells elaborated by one interpret pass.
//!
//! Target properties (setup step S5):
//! - State-with-Catch ordering: `catch(put(true) >> throw, recover = pure(()))`
//!   then `get` yields value `true` and final state `true` (the write before a
//!   caught throw survives). From `run_heftia_semantics.rs`.
//! - Writer post-censor: `censor(f, tell("Hello") >> tell(" world!"))` with
//!   `f("Hello world!") = "Hello world!!"` yields log "Hello world!!".
//! - Reader composes: `ask()` supplies the environment into a State+Catch
//!   program.
//!
//! Dispatch note: the first-order handling here is direct coproduct matching
//! (the documented fallback); brand-keyed dispatch was proven standalone in
//! POC-2 and is an orthogonal layer. Throwaway spike code.

#![cfg(feature = "effects")]
#![allow(
	dead_code,
	reason = "some constructors/arms are present for the slice but not all are exercised"
)]

use {
	fp_library::{
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
			effects::coproduct::Coproduct,
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

// State over a bool cell.
struct StateBrand;
enum StateF<'a, A> {
	Get(Box<dyn FnOnce(bool) -> A + 'a>),
	Put(bool, Box<dyn FnOnce(()) -> A + 'a>),
}
impl_kind! { impl for StateBrand { type Of<'a, A: 'a>: 'a = StateF<'a, A>; } }
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

// Reader over an i32 environment.
struct ReaderBrand;
enum ReaderF<'a, A> {
	Ask(Box<dyn FnOnce(i32) -> A + 'a>),
}
impl_kind! { impl for ReaderBrand { type Of<'a, A: 'a>: 'a = ReaderF<'a, A>; } }
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

// Throw with a unit error (phantom result).
struct ThrowBrand;
struct ThrowF<A>(PhantomData<A>);
impl_kind! { impl for ThrowBrand { type Of<'a, A: 'a>: 'a = ThrowF<A>; } }
impl Functor for ThrowBrand {
	fn map<'a, A: 'a, B: 'a>(
		_f: impl Fn(A) -> B + 'a,
		_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		ThrowF(PhantomData)
	}
}

// Writer over a String log.
struct WriterBrand;
enum WriterF<'a, A> {
	Tell(String, Box<dyn FnOnce(()) -> A + 'a>),
}
impl_kind! { impl for WriterBrand { type Of<'a, A: 'a>: 'a = WriterF<'a, A>; } }
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

// -- Higher-order effects as in-row cells (same-result) --

// Catch: run the action; if it throws, run recover. Result equals the action
// result `RAction`.
struct CatchBrand<RAction>(PhantomData<RAction>);
struct CatchCell<'a, RAction: 'static, Next> {
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

// Censor: run the action accumulating its log locally, then emit `f(total)` to
// the outer writer. Result equals the action result `RAction`.
struct CensorBrand<RAction>(PhantomData<RAction>);
struct CensorCell<'a, RAction: 'static, Next> {
	f: Rc<dyn Fn(String) -> String + 'a>,
	action: Free<Row, RAction>,
	k: Box<dyn FnOnce(RAction) -> Next + 'a>,
}
impl_kind! {
	impl<RAction: 'static> for CensorBrand<RAction> {
		type Of<'a, Next: 'a>: 'a = CensorCell<'a, RAction, Next>;
	}
}
impl<RAction: 'static> Functor for CensorBrand<RAction> {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		let CensorCell {
			f: cf,
			action,
			k,
		} = fa;
		CensorCell {
			f: cf,
			action,
			k: Box::new(move |a| f(k(a))),
		}
	}
}

// The unified row: four first-order effects plus two higher-order cells (both
// fixed at action result `()` for this slice).
type Row = CoproductBrand<
	CoyonedaBrand<StateBrand>,
	CoproductBrand<
		CoyonedaBrand<ReaderBrand>,
		CoproductBrand<
			CoyonedaBrand<ThrowBrand>,
			CoproductBrand<
				CoyonedaBrand<WriterBrand>,
				CoproductBrand<
					CoyonedaBrand<CatchBrand<()>>,
					CoproductBrand<CoyonedaBrand<CensorBrand<()>>, CNilBrand>,
				>,
			>,
		>,
	>,
>;

type Node<A> = Apply!(<Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>);

// -- Smart constructors (inject at the right coproduct position) --

fn get() -> Free<Row, bool> {
	let coyo: Coyoneda<'static, StateBrand, bool> = Coyoneda::lift(StateF::Get(Box::new(|s| s)));
	Free::lift_f(Coproduct::Inl(coyo) as Node<bool>)
}
fn put(value: bool) -> Free<Row, ()> {
	let coyo: Coyoneda<'static, StateBrand, ()> =
		Coyoneda::lift(StateF::Put(value, Box::new(|u| u)));
	Free::lift_f(Coproduct::Inl(coyo) as Node<()>)
}
fn ask() -> Free<Row, i32> {
	let coyo: Coyoneda<'static, ReaderBrand, i32> = Coyoneda::lift(ReaderF::Ask(Box::new(|e| e)));
	Free::lift_f(Coproduct::Inr(Coproduct::Inl(coyo)) as Node<i32>)
}
fn throw<A: 'static>() -> Free<Row, A> {
	let coyo: Coyoneda<'static, ThrowBrand, A> = Coyoneda::lift(ThrowF(PhantomData));
	Free::lift_f(Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(coyo))) as Node<A>)
}
fn tell(w: String) -> Free<Row, ()> {
	let coyo: Coyoneda<'static, WriterBrand, ()> =
		Coyoneda::lift(WriterF::Tell(w, Box::new(|u| u)));
	Free::lift_f(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(coyo)))) as Node<()>)
}
fn catch(
	action: Free<Row, ()>,
	recover: impl Fn() -> Free<Row, ()> + 'static,
) -> Free<Row, ()> {
	let cell: CatchCell<'static, (), ()> = CatchCell {
		action,
		recover: Rc::new(recover),
		k: Box::new(|a| a),
	};
	let coyo: Coyoneda<'static, CatchBrand<()>, ()> = Coyoneda::<CatchBrand<()>, _>::lift(cell);
	Free::lift_f(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(
		coyo,
	))))) as Node<()>)
}
fn censor(
	f: impl Fn(String) -> String + 'static,
	action: Free<Row, ()>,
) -> Free<Row, ()> {
	let cell: CensorCell<'static, (), ()> = CensorCell {
		f: Rc::new(f),
		action,
		k: Box::new(|a| a),
	};
	let coyo: Coyoneda<'static, CensorBrand<()>, ()> = Coyoneda::<CensorBrand<()>, _>::lift(cell);
	Free::lift_f(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(
		Coproduct::Inl(coyo),
	))))) as Node<()>)
}

// -- One interpret pass over the unified row --

fn run<A: 'static>(
	program: Free<Row, A>,
	state: &Cell<bool>,
	env: i32,
	log: &RefCell<String>,
) -> Result<A, ()> {
	let mut program = program;
	loop {
		match program.resume() {
			Ok(value) => return Ok(value),
			Err(layer) => match layer {
				// State
				Coproduct::Inl(coyo) => match coyo.lower() {
					StateF::Get(k) => program = k(state.get()),
					StateF::Put(s, k) => {
						state.set(s);
						program = k(());
					}
				},
				// Reader
				Coproduct::Inr(Coproduct::Inl(coyo)) => match coyo.lower() {
					ReaderF::Ask(k) => program = k(env),
				},
				// Throw
				Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(_coyo))) => return Err(()),
				// Writer
				Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(coyo)))) => {
					match coyo.lower() {
						WriterF::Tell(w, k) => {
							log.borrow_mut().push_str(&w);
							program = k(());
						}
					}
				}
				// Catch (higher-order): run the action; on throw, run recover. State
				// writes inside the action persist (shared cell), so a write before a
				// caught throw survives.
				Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(
					coyo,
				))))) => {
					let CatchCell {
						action,
						recover,
						k,
					} = coyo.lower();
					match run(action, state, env, log) {
						Ok(()) => {}
						Err(()) => run(recover(), state, env, log)?,
					}
					program = k(());
				}
				// Censor (higher-order): run the action with a local log; emit
				// f(total) to the outer log; continue.
				Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(
					Coproduct::Inl(coyo),
				))))) => {
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
				}
				// CNil
				Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(
					Coproduct::Inr(cnil),
				))))) => match cnil {},
			},
		}
	}
}

fn heftia_writer_censor(log: String) -> String {
	match log.as_str() {
		"Hello" => "Goodbye".to_string(),
		"Hello world!" => "Hello world!!".to_string(),
		_ => log,
	}
}

#[test]
fn state_with_catch_ordering_matches_heftia() {
	// catch(put(true) >> throw, recover = pure(())) >> get
	let protected: Free<Row, ()> = put(true).bind(|()| throw::<()>());
	let program: Free<Row, bool> = catch(protected, || Free::pure(())).bind(|()| get());

	let state = Cell::new(false);
	let log = RefCell::new(String::new());
	let result = run(program, &state, 0, &log);

	assert_eq!(result, Ok(true));
	assert!(state.get());
}

#[test]
fn writer_post_censor_matches_heftia() {
	// censor(f, tell("Hello") >> tell(" world!"))
	let action: Free<Row, ()> = tell("Hello".to_string()).bind(|()| tell(" world!".to_string()));
	let program: Free<Row, ()> = censor(heftia_writer_censor, action);

	let state = Cell::new(false);
	let log = RefCell::new(String::new());
	let result = run(program, &state, 0, &log);

	assert_eq!(result, Ok(()));
	assert_eq!(*log.borrow(), "Hello world!!".to_string());
}

#[test]
fn reader_composes_with_state_and_catch() {
	// ask() supplies the environment; if positive, put(true) then a caught throw;
	// get observes the surviving write.
	let program: Free<Row, bool> = ask().bind(|env| {
		catch(put(env > 5).bind(|()| throw::<()>()), || Free::pure(())).bind(|()| get())
	});

	let state = Cell::new(false);
	let log = RefCell::new(String::new());
	let result = run(program, &state, 10, &log);

	assert_eq!(result, Ok(true));
	assert!(state.get());
}
