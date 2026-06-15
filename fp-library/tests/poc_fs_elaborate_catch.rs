//! POC-4 (foundation sweep, Tier D): elaborate a same-result higher-order
//! effect (`Catch`) into first-order `Throw` plus interpose.
//!
//! Charter question: can a same-result higher-order effect (`Catch`, whose
//! action result equals its operation result) be elaborated into first-order
//! `Throw` plus interpose, reproducing the boundary-frame semantics? The target
//! property, restated from `run_heftia_semantics.rs`
//! (`state_write_before_caught_throw_survives_handler_orders`, setup step S5):
//! `put(true) >> throw`, protected by `catch(_, recover = pure(()))`, then
//! `get`, yields value `true` and final state `true` (the state write before a
//! caught throw survives the catch).
//!
//! Mechanism: `catch(action, recover)` is elaborated into an interpose over
//! `Throw` (`elaborate_catch`): walk the action, replace each `Throw` with
//! `recover`, and pass every other first-order effect (`State`) through
//! unchanged. The result is a first-order `State`+`Throw` program, which a plain
//! peel-loop interpreter runs. This is heftia's `runCatch` semantics
//! (`action & interposeWith (\Throw _ -> recover)`); here the elaboration is
//! applied at construction for spike simplicity, which is semantically
//! equivalent to an interpret-pass elaboration for this same-result case.
//!
//! Harness (setup steps S2, S3): public-API integration test over the real
//! `Free` substrate with a uniformly `Coyoneda`-wrapped `State`+`Throw` row.
//! Throwaway spike code.

#![cfg(feature = "effects")]

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
		cell::Cell,
		marker::PhantomData,
		rc::Rc,
	},
};

// First-order State effect over a `bool` cell.
struct StateBrand;
enum StateF<'a, A> {
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

// First-order Throw effect with a unit error; the result type is phantom.
struct ThrowBrand;
struct ThrowF<A>(PhantomData<A>);
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

// The first-order row that `catch` elaborates into: State plus Throw, each
// Coyoneda-wrapped.
type Row =
	CoproductBrand<CoyonedaBrand<StateBrand>, CoproductBrand<CoyonedaBrand<ThrowBrand>, CNilBrand>>;

type Node<A> = Apply!(<Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>);

fn get() -> Free<Row, bool> {
	let coyo: Coyoneda<'static, StateBrand, bool> = Coyoneda::lift(StateF::Get(Box::new(|s| s)));
	let node: Node<bool> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

fn put(value: bool) -> Free<Row, ()> {
	let coyo: Coyoneda<'static, StateBrand, ()> =
		Coyoneda::lift(StateF::Put(value, Box::new(|unit| unit)));
	let node: Node<()> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

fn throw<R: 'static>() -> Free<Row, R> {
	let coyo: Coyoneda<'static, ThrowBrand, R> = Coyoneda::lift(ThrowF(PhantomData));
	let node: Node<R> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

// Elaboration: rewrite the action so each `Throw` becomes `recover`, passing
// `State` through. This is the interpose-over-Throw that `catch` elaborates to.
fn elaborate_catch<R: 'static>(
	action: Free<Row, R>,
	recover: Rc<dyn Fn() -> Free<Row, R>>,
) -> Free<Row, R> {
	match action.resume() {
		Ok(value) => Free::pure(value),
		Err(layer) => match layer {
			// State: rebuild the same operation with each continuation
			// recursively elaborated.
			Coproduct::Inl(state_coyo) => {
				let op = state_coyo.lower();
				let rebuilt: Node<Free<Row, R>> =
					Coproduct::inject(Coyoneda::<StateBrand, _>::lift(match op {
						StateF::Get(k) => {
							let recover = Rc::clone(&recover);
							StateF::Get(Box::new(move |s| {
								elaborate_catch(k(s), Rc::clone(&recover))
							}))
						}
						StateF::Put(s, k) => {
							let recover = Rc::clone(&recover);
							StateF::Put(
								s,
								Box::new(move |u| elaborate_catch(k(u), Rc::clone(&recover))),
							)
						}
					}));
				Free::wrap(rebuilt)
			}
			// Throw: caught, run the recovery program.
			Coproduct::Inr(Coproduct::Inl(_throw_coyo)) => recover(),
			Coproduct::Inr(Coproduct::Inr(cnil)) => match cnil {},
		},
	}
}

// `catch` is its elaboration.
fn catch<R: 'static>(
	action: Free<Row, R>,
	recover: impl Fn() -> Free<Row, R> + 'static,
) -> Free<Row, R> {
	elaborate_catch(action, Rc::new(recover))
}

// A plain peel-loop interpreter for the first-order State+Throw row: State reads
// and writes the cell; Throw aborts.
fn run<R: 'static>(
	program: Free<Row, R>,
	state: &Cell<bool>,
) -> Result<R, ()> {
	let mut program = program;
	loop {
		match program.resume() {
			Ok(value) => return Ok(value),
			Err(layer) => match layer {
				Coproduct::Inl(state_coyo) => match state_coyo.lower() {
					StateF::Get(k) => program = k(state.get()),
					StateF::Put(s, k) => {
						state.set(s);
						program = k(());
					}
				},
				Coproduct::Inr(Coproduct::Inl(_throw_coyo)) => return Err(()),
				Coproduct::Inr(Coproduct::Inr(cnil)) => match cnil {},
			},
		}
	}
}

#[test]
fn state_write_before_caught_throw_survives() {
	// catch(put(true) >> throw, recover = pure(())) >> get
	let protected: Free<Row, ()> = put(true).bind(|()| throw::<()>());
	let program: Free<Row, bool> = catch(protected, || Free::pure(())).bind(|()| get());

	let state = Cell::new(false);
	let result = run(program, &state);

	// The throw inside the catch is recovered, the state write survives, and get
	// observes it: value true, final state true. Matches the heftia case.
	assert_eq!(result, Ok(true));
	assert!(state.get());
}

#[test]
fn an_uncaught_throw_aborts() {
	// Without a surrounding catch, a throw aborts the interpreter.
	let program: Free<Row, bool> = put(true).bind(|()| throw::<bool>());
	let state = Cell::new(false);
	let result = run(program, &state);
	assert_eq!(result, Err(()));
	// The write before the throw still happened.
	assert!(state.get());
}
