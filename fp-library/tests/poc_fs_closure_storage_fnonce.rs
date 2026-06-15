//! POC-8b (foundation sweep, Tier E): the `FnOnce`-versus-`Fn` reconciliation
//! and `bind`/`map` over composed stored closures.
//!
//! POC-8 proved the substrate type, interpreter, and `Clone` collapse to one
//! `Run<Store: ClosureStorage, A>` on the `Fn`-only layer, and deliberately
//! skipped two things, which this POC builds:
//!
//! 1. The `FnOnce`-versus-`Fn` reconciliation. The real `Free` spine stores
//!    `Box<dyn FnOnce>` one-shot continuations, while the multi-shot Rc/Arc
//!    wrappers store reusable `Fn`. POC-8's `Fn`-only layer forced Box onto
//!    `Fn` too, losing the one-shot spine. The load-bearing question for
//!    plan item 6's primary outcome (one brand per effect, no prefix axis) was
//!    whether ONE `ClosureStorage` associated type can carry the callable KIND
//!    so Box keeps `FnOnce` while Rc/Arc keep `Fn`, all under one substrate.
//!
//! 2. `bind`/`map`. POC-8 exercised only `pure`/`handle`; `bind`/`map`
//!    construct new stored closures by composition, which is where the
//!    `FnOnce`-versus-`Fn` asymmetry actually bites (composing onto a `FnOnce`
//!    moves it; composing onto an `Fn` must not).
//!
//! The unification mechanism proved here:
//! - `ClosureStorage::Stored<'a, I, O>` is `Box<dyn FnOnce>` for Box and
//!   `Rc`/`Arc<dyn Fn (+ Send + Sync)>` for Rc/Arc. The associated type carries
//!   the callable kind, not just the pointer.
//! - `call_once(s: Stored, i) -> O` takes the stored closure BY VALUE, which
//!   bridges both kinds: a `FnOnce` is consumed and run once; an `Fn` is
//!   borrowed through the owned pointer for the call, then the pointer drops.
//!   One method signature, one `handle`, for both kinds.
//! - Multi-shot (calling a continuation more than once) is the conditional
//!   `Clone` from POC-8: clone the stored closure, then `call_once` each clone.
//!   Rc/Arc are `Clone`, Box is not, so Box is one-shot by construction, which
//!   is the correct spine semantics.
//! - `bind`/`map` are written per `Store` (construction is per-`Store`, POC-8
//!   finding 2): the Box bodies move the captured continuation (a `FnOnce`
//!   closure consumes its captures once); the Rc/Arc bodies clone it (an `Fn`
//!   closure cannot move a capture out on each call). Same `Run`, same
//!   `call_once`, same `handle`, three construction sites.
//!
//! Throwaway spike code (S2 finding: bespoke pointer-parameterised substrate;
//! the public `Free` is `Box`-spine only).

#![cfg(feature = "effects")]
#![allow(
	dead_code,
	reason = "some per-Store constructors/runners are present for symmetry but not every one is exercised by a test"
)]

use {
	fp_library::brands::{
		ArcBrand,
		BoxBrand,
		RcBrand,
	},
	std::{
		rc::Rc,
		sync::Arc,
	},
};

// One class. `Stored` now carries the callable KIND per `Store`: `FnOnce` for
// Box, `Fn` for Rc/Arc. `call_once` consumes the stored closure by value, which
// is what lets one signature serve both kinds.
trait ClosureStorage: 'static {
	type Stored<'a, I: 'a, O: 'a>: 'a;
	fn call_once<'a, I: 'a, O: 'a>(
		s: Self::Stored<'a, I, O>,
		i: I,
	) -> O;
}

impl ClosureStorage for BoxBrand {
	// The one-shot spine: a genuine `FnOnce`, which an `Fn`-only GAT could not
	// hold (see `box_continuation_is_genuinely_fnonce`).
	type Stored<'a, I: 'a, O: 'a> = Box<dyn FnOnce(I) -> O + 'a>;

	fn call_once<'a, I: 'a, O: 'a>(
		s: Self::Stored<'a, I, O>,
		i: I,
	) -> O {
		// `Box<dyn FnOnce>: FnOnce`, so calling the owned box consumes it.
		s(i)
	}
}
impl ClosureStorage for RcBrand {
	type Stored<'a, I: 'a, O: 'a> = Rc<dyn Fn(I) -> O + 'a>;

	fn call_once<'a, I: 'a, O: 'a>(
		s: Self::Stored<'a, I, O>,
		i: I,
	) -> O {
		// Borrow the `Fn` through the owned `Rc` for the call; `s` drops after.
		// `&dyn Fn: Fn`, so this is a normal call. The `Rc` could equally have
		// been cloned first and retained (multi-shot); consuming it here is the
		// single-use path.
		(&*s)(i)
	}
}
impl ClosureStorage for ArcBrand {
	type Stored<'a, I: 'a, O: 'a> = Arc<dyn Fn(I) -> O + Send + Sync + 'a>;

	fn call_once<'a, I: 'a, O: 'a>(
		s: Self::Stored<'a, I, O>,
		i: I,
	) -> O {
		(&*s)(i)
	}
}

// ONE substrate type over the associated stored-closure type, exactly as in
// POC-8. `Ask`'s continuation is `i32 -> Run`, stored via `Store`.
enum Run<Store: ClosureStorage, A: 'static> {
	Pure(A),
	Ask(Store::Stored<'static, i32, Run<Store, A>>),
}

impl<Store: ClosureStorage, A: 'static> Run<Store, A> {
	fn pure(a: A) -> Self {
		Run::Pure(a)
	}
}

// ONE generic interpreter. `program` is owned, so the `Ask` continuation is
// owned and `call_once` consumes it; the recursion is the same for all three
// stores regardless of the stored callable kind.
fn handle<Store: ClosureStorage, A: 'static>(
	program: Run<Store, A>,
	env: i32,
) -> A {
	match program {
		Run::Pure(a) => a,
		Run::Ask(k) => handle(Store::call_once(k, env), env),
	}
}

// ONE conditional `Clone`, identical to POC-8: Rc/Arc stored closures are
// `Clone` so `Run` is `Clone` there; Box is not. This is what gates multi-shot.
impl<Store: ClosureStorage, A: Clone + 'static> Clone for Run<Store, A>
where
	for<'a> Store::Stored<'a, i32, Run<Store, A>>: Clone,
{
	fn clone(&self) -> Self {
		match self {
			Run::Pure(a) => Run::Pure(a.clone()),
			Run::Ask(k) => Run::Ask(k.clone()),
		}
	}
}

// -- Per-Store construction (POC-8 finding 2: construction stays per-Store) --

fn ask_box() -> Run<BoxBrand, i32> {
	let k: Box<dyn FnOnce(i32) -> Run<BoxBrand, i32>> = Box::new(|i: i32| Run::Pure(i));
	Run::Ask(k)
}
fn ask_rc() -> Run<RcBrand, i32> {
	let k: Rc<dyn Fn(i32) -> Run<RcBrand, i32>> = Rc::new(|i: i32| Run::Pure(i));
	Run::Ask(k)
}
fn ask_arc() -> Run<ArcBrand, i32> {
	let k: Arc<dyn Fn(i32) -> Run<ArcBrand, i32> + Send + Sync> = Arc::new(|i: i32| Run::Pure(i));
	Run::Ask(k)
}

// -- `map`, per Store. The Box body MOVES the captured continuation (its
// composed closure is a `FnOnce`, run once); the Rc/Arc bodies CLONE it (their
// composed closures are `Fn`, callable repeatedly, so they cannot move a
// capture out per call). This per-Store body difference is precisely why
// construction does not unify (POC-8 finding 2), even though the substrate
// type/interpreter/Clone do. --

fn map_box<A: 'static, B: 'static>(
	program: Run<BoxBrand, A>,
	f: Box<dyn FnOnce(A) -> B>,
) -> Run<BoxBrand, B> {
	match program {
		Run::Pure(a) => Run::Pure(f(a)),
		Run::Ask(k) => {
			// FnOnce: moves `k` and `f` into a closure run at most once.
			let k2: Box<dyn FnOnce(i32) -> Run<BoxBrand, B>> =
				Box::new(move |i| map_box(BoxBrand::call_once(k, i), f));
			Run::Ask(k2)
		}
	}
}
fn map_rc<A: 'static, B: 'static>(
	program: Run<RcBrand, A>,
	f: Rc<dyn Fn(A) -> B>,
) -> Run<RcBrand, B> {
	match program {
		Run::Pure(a) => Run::Pure((&*f)(a)),
		Run::Ask(k) => {
			// Fn: must clone the captures on each call rather than move them.
			let k2: Rc<dyn Fn(i32) -> Run<RcBrand, B>> =
				Rc::new(move |i| map_rc(RcBrand::call_once(k.clone(), i), f.clone()));
			Run::Ask(k2)
		}
	}
}
fn map_arc<A: 'static, B: 'static>(
	program: Run<ArcBrand, A>,
	f: Arc<dyn Fn(A) -> B + Send + Sync>,
) -> Run<ArcBrand, B> {
	match program {
		Run::Pure(a) => Run::Pure((&*f)(a)),
		Run::Ask(k) => {
			let k2: Arc<dyn Fn(i32) -> Run<ArcBrand, B> + Send + Sync> =
				Arc::new(move |i| map_arc(ArcBrand::call_once(k.clone(), i), f.clone()));
			Run::Ask(k2)
		}
	}
}

// -- `bind`, per Store. Same move-versus-clone asymmetry; the Pure case applies
// the continuation directly (consuming for Box, borrowing for Rc/Arc). --

fn bind_box<A: 'static, B: 'static>(
	program: Run<BoxBrand, A>,
	k: Box<dyn FnOnce(A) -> Run<BoxBrand, B>>,
) -> Run<BoxBrand, B> {
	match program {
		Run::Pure(a) => k(a),
		Run::Ask(cont) => {
			let cont2: Box<dyn FnOnce(i32) -> Run<BoxBrand, B>> =
				Box::new(move |i| bind_box(BoxBrand::call_once(cont, i), k));
			Run::Ask(cont2)
		}
	}
}
fn bind_rc<A: 'static, B: 'static>(
	program: Run<RcBrand, A>,
	k: Rc<dyn Fn(A) -> Run<RcBrand, B>>,
) -> Run<RcBrand, B> {
	match program {
		Run::Pure(a) => (&*k)(a),
		Run::Ask(cont) => {
			let cont2: Rc<dyn Fn(i32) -> Run<RcBrand, B>> =
				Rc::new(move |i| bind_rc(RcBrand::call_once(cont.clone(), i), k.clone()));
			Run::Ask(cont2)
		}
	}
}
fn bind_arc<A: 'static, B: 'static>(
	program: Run<ArcBrand, A>,
	k: Arc<dyn Fn(A) -> Run<ArcBrand, B> + Send + Sync>,
) -> Run<ArcBrand, B> {
	match program {
		Run::Pure(a) => (&*k)(a),
		Run::Ask(cont) => {
			let cont2: Arc<dyn Fn(i32) -> Run<ArcBrand, B> + Send + Sync> =
				Arc::new(move |i| bind_arc(ArcBrand::call_once(cont.clone(), i), k.clone()));
			Run::Ask(cont2)
		}
	}
}

fn assert_send_sync<T: Send + Sync>() {}

// -- Gap 2: the `FnOnce`-versus-`Fn` reconciliation. --

#[test]
fn one_substrate_holds_fnonce_box_and_fn_rc_arc() {
	// The same `Run`/`handle` runs a Box program whose continuation is `FnOnce`
	// and Rc/Arc programs whose continuations are `Fn`.
	assert_eq!(handle(ask_box(), 42), 42);
	assert_eq!(handle(ask_rc(), 42), 42);
	assert_eq!(handle(ask_arc(), 42), 42);
}

#[test]
fn box_continuation_is_genuinely_fnonce() {
	// The Box continuation MOVES a non-`Copy` capture out of itself when run
	// (`into_bytes` consumes the `String`), so it is a true `FnOnce` that an
	// `Fn`-only GAT could not store. It still lives in the same `Run`/`Store`.
	let owned = String::from("consumed-once");
	let k: Box<dyn FnOnce(i32) -> Run<BoxBrand, usize>> =
		Box::new(move |i| Run::Pure(owned.into_bytes().len() + i as usize));
	let program: Run<BoxBrand, usize> = Run::Ask(k);
	assert_eq!(handle(program, 3), "consumed-once".len() + 3);
}

#[test]
fn rc_continuation_is_reusable_multi_shot() {
	// The Rc continuation is `Fn`, so it can be cloned and `call_once`d several
	// times (multi-shot), which the conditional `Clone` enables. The Box
	// `FnOnce` continuation cannot be cloned, so Box is one-shot by
	// construction; that asymmetry is the correct spine semantics, carried by
	// one `ClosureStorage`.
	let cont: Rc<dyn Fn(i32) -> Run<RcBrand, i32>> = Rc::new(|i| Run::Pure(i * 10));
	let first = RcBrand::call_once(cont.clone(), 1);
	let second = RcBrand::call_once(cont.clone(), 2);
	let third = RcBrand::call_once(cont, 3); // last use consumes the original
	assert_eq!(handle(first, 0), 10);
	assert_eq!(handle(second, 0), 20);
	assert_eq!(handle(third, 0), 30);
}

#[test]
fn arc_instantiation_is_send_sync() {
	assert_send_sync::<Run<ArcBrand, i32>>();
}

// -- Gap 1: `bind`/`map` over composed stored closures, all three stores. --

#[test]
fn map_composes_stored_closures_box() {
	// map over `Ask(k)` builds a new stored continuation `i -> map(k(i), f)`.
	let program = map_box(ask_box(), Box::new(|x: i32| x + 100));
	assert_eq!(handle(program, 5), 105);
}

#[test]
fn map_composes_stored_closures_rc() {
	let program = map_rc(ask_rc(), Rc::new(|x: i32| x + 100));
	assert_eq!(handle(program, 5), 105);
}

#[test]
fn map_composes_stored_closures_arc() {
	let program = map_arc(ask_arc(), Arc::new(|x: i32| x + 100));
	assert_eq!(handle(program, 5), 105);
}

#[test]
fn bind_composes_stored_closures_box() {
	// bind the Ask result into another Ask, then add: reads `env` twice.
	let program =
		bind_box(ask_box(), Box::new(|x: i32| map_box(ask_box(), Box::new(move |y: i32| x + y))));
	assert_eq!(handle(program, 21), 42);
}

#[test]
fn bind_composes_stored_closures_rc() {
	let program =
		bind_rc(ask_rc(), Rc::new(|x: i32| map_rc(ask_rc(), Rc::new(move |y: i32| x + y))));
	assert_eq!(handle(program, 21), 42);
}

#[test]
fn bind_composes_stored_closures_arc() {
	let program =
		bind_arc(ask_arc(), Arc::new(|x: i32| map_arc(ask_arc(), Arc::new(move |y: i32| x + y))));
	assert_eq!(handle(program, 21), 42);
}

#[test]
fn bound_program_is_clone_for_rc_and_runs_both_copies() {
	// A non-trivial bound Rc program is `Clone` via the one conditional impl,
	// and both copies run independently: this is `bind` output flowing through
	// the multi-shot path, which POC-8 only showed for a bare `ask`.
	let program =
		bind_rc(ask_rc(), Rc::new(|x: i32| map_rc(ask_rc(), Rc::new(move |y: i32| x + y))));
	let copy = program.clone();
	assert_eq!(handle(program, 21), 42);
	assert_eq!(handle(copy, 10), 20);
}
