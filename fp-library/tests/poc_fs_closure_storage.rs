//! POC-8 (foundation sweep, Tier E): substrate unification via one
//! `ClosureStorage` associated stored-closure type (merged POC-7 + POC-8).
//!
//! Charter question: can one `ClosureStorage` class with an associated
//! stored-closure type unify the per-`Store` closure storage so that ONE
//! `Run<Store: ClosureStorage, A>` substrate carries the per-`Store`
//! `Clone`/`Send + Sync` bounds, collapsing the six wrappers? Tested on the
//! `Fn`-only layer first (it avoids the `FnOnce`-versus-`Fn` consumption
//! mismatch).
//!
//! What this probes, spelled concretely:
//! - `ClosureStorage::Stored<'a, I, O>` is the per-`Store` stored closure:
//!   `Box<dyn Fn>` / `Rc<dyn Fn>` / `Arc<dyn Fn + Send + Sync>`. The `dyn`
//!   bound varies per `Store` (only Arc adds `Send + Sync`), which is exactly
//!   why one uniform field cannot express it without the associated type.
//! - `Run<Store: ClosureStorage, A>` is ONE substrate type over that associated
//!   type; `handle` (the interpreter) is one generic function.
//! - Construction of a stored closure carries the per-`Store` input bound
//!   (Arc requires the closure `Send + Sync`). Construction is per-`Store`
//!   here; whether it can be written once generically over `Store` is the
//!   load-bearing question, and it cannot (see the construction comment below).
//!
//! Throwaway spike code (S2 finding: this is a bespoke pointer-parameterised
//! substrate because the public `Free` is `Box`-spine only).

#![cfg(feature = "effects")]
#![allow(
	dead_code,
	reason = "pure demonstrates the generic constructor unifies but is not exercised by the Ask tests"
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

// One class: the associated stored-closure type plus a way to call it. `call`
// has a uniform signature; only the body and the associated type vary per
// `Store`, so this part unifies cleanly.
trait ClosureStorage: 'static {
	type Stored<'a, I: 'a, O: 'a>: 'a;
	fn call<'a, I: 'a, O: 'a>(
		s: &Self::Stored<'a, I, O>,
		i: I,
	) -> O;
}

impl ClosureStorage for BoxBrand {
	type Stored<'a, I: 'a, O: 'a> = Box<dyn Fn(I) -> O + 'a>;

	fn call<'a, I: 'a, O: 'a>(
		s: &Self::Stored<'a, I, O>,
		i: I,
	) -> O {
		s(i)
	}
}
impl ClosureStorage for RcBrand {
	type Stored<'a, I: 'a, O: 'a> = Rc<dyn Fn(I) -> O + 'a>;

	fn call<'a, I: 'a, O: 'a>(
		s: &Self::Stored<'a, I, O>,
		i: I,
	) -> O {
		(&**s)(i)
	}
}
impl ClosureStorage for ArcBrand {
	type Stored<'a, I: 'a, O: 'a> = Arc<dyn Fn(I) -> O + Send + Sync + 'a>;

	fn call<'a, I: 'a, O: 'a>(
		s: &Self::Stored<'a, I, O>,
		i: I,
	) -> O {
		(&**s)(i)
	}
}

// Construction is per-`Store`: the input-closure bound varies (Arc requires the
// closure `Send + Sync`), which a single generic constructor signature cannot
// carry, the same reason the three `ToDyn*` classes exist separately. A helper
// trait `IntoStored` with per-`Store` impls (the Arc impl adding `Send + Sync`)
// can carry the bound, but the associated-type projection `Store::Stored` is not
// injective, so the compiler cannot infer `Store` from the constructed type and
// generic construction needs an explicit `Store`. Direct per-`Store`
// construction below (`Box::new`/`Rc::new`/`Arc::new`) keeps the per-`Store`
// bound enforced by the `Ask` field's `dyn` type (the Arc field is
// `dyn Fn + Send + Sync`, so its closure must be `Send + Sync`).

// ONE substrate type over the associated stored-closure type. The `Ask` effect
// stores its continuation (i32 -> Run) via `Store`.
enum Run<Store: ClosureStorage, A: 'static> {
	Pure(A),
	Ask(Store::Stored<'static, i32, Run<Store, A>>),
}

// `pure` and `handle` are generic over `Store` (no closure construction in
// `handle`, only `call`), so they unify cleanly.
impl<Store: ClosureStorage, A: 'static> Run<Store, A> {
	fn pure(a: A) -> Self {
		Run::Pure(a)
	}
}

// The interpreter: one generic function over `Store`. Supplies `env` to each
// `Ask` and returns the final value.
fn handle<Store: ClosureStorage, A: 'static>(
	program: Run<Store, A>,
	env: i32,
) -> A {
	match program {
		Run::Pure(a) => a,
		Run::Ask(k) => handle(Store::call(&k, env), env),
	}
}

// `ask` constructed per `Store` by direct pointer construction (the continuation
// closure captures nothing, so it satisfies the Arc `Send + Sync` bound too).
// Generic-over-`Store` construction is not expressible (see the construction
// comment above); here it is monomorphic, which is enough to run the effect for
// all three stores.
fn ask_box() -> Run<BoxBrand, i32> {
	// The typed binding drives the unsize coercion to `dyn Fn` and infers the
	// closure's `Store` from the annotation.
	let k: Box<dyn Fn(i32) -> Run<BoxBrand, i32>> = Box::new(|i: i32| Run::Pure(i));
	Run::Ask(k)
}
fn ask_rc() -> Run<RcBrand, i32> {
	let k: Rc<dyn Fn(i32) -> Run<RcBrand, i32>> = Rc::new(|i: i32| Run::Pure(i));
	Run::Ask(k)
}
fn ask_arc() -> Run<ArcBrand, i32> {
	// The closure must be `Send + Sync` to coerce to the Arc field's
	// `dyn Fn + Send + Sync`; this captures nothing, so it is.
	let k: Arc<dyn Fn(i32) -> Run<ArcBrand, i32> + Send + Sync> = Arc::new(|i: i32| Run::Pure(i));
	Run::Ask(k)
}

// Conditional `Clone`: one impl block whose `where`-clause resolves per `Store`.
// `Rc`/`Arc` stored closures are `Clone`, so `Run` is `Clone` there; `Box` is
// not, so `Run<BoxBrand, _>` is not `Clone`. This is the per-`Store` `Clone`
// asymmetry expressed on one impl.
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

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn one_substrate_type_runs_ask_for_all_three_stores() {
	// `ask()` returns the supplied environment; the substrate is one generic
	// type and `handle` is one generic function.
	assert_eq!(handle(ask_box(), 42), 42);
	assert_eq!(handle(ask_rc(), 42), 42);
	assert_eq!(handle(ask_arc(), 42), 42);
}

#[test]
fn arc_instantiation_is_send_sync() {
	// The Arc instantiation is `Send + Sync` (static assertion): this is the
	// per-`Store` bound the associated stored-closure type carries.
	assert_send_sync::<Run<ArcBrand, i32>>();
}

#[test]
fn rc_substrate_is_clone_and_box_is_not() {
	// The Rc instantiation is `Clone` via the conditional impl; cloning then
	// running both copies gives the same result. (Box would not compile under a
	// `Clone` bound, which is the asymmetry; not exercised here to keep the test
	// compiling for all stores.)
	let program = ask_rc();
	let copy = program.clone();
	assert_eq!(handle(program, 7), 7);
	assert_eq!(handle(copy, 7), 7);
}
