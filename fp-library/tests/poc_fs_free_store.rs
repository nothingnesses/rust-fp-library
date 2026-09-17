//! POC: parameterise the erased free-monad spine by a closure store WITHOUT
//! churning existing consumers, via a TRAILING DEFAULTED `Store` type parameter
//! (remediation item 4 step 5.2 / OQ-5B).
//!
//! OQ-5B's recommendation phrased the unification as "parameterise `Free` into
//! `Free<Store, F, A>` and keep the bare name via `type Free<F, A> = Free<Box, F,
//! A>`." That literal form is impossible: a struct and a type alias cannot share
//! the name `Free`. The implementable form is a trailing defaulted parameter,
//! `Free<F, A, Store = BoxBrand>`: every existing `Free<F, A>` use then means
//! `Free<F, A, BoxBrand>` unchanged (the default absorbs the churn the alias was
//! meant to absorb), while effects pick `Free<F, A, RcBrand>` / `...ArcBrand>`.
//!
//! This POC validates that mechanism on a mini free monad before the production
//! `Free` (about 2600 lines) and its non-effects consumers (`Trampoline`,
//! `TryTrampoline`, `Thunk`, `Identity`) are parameterised. It is self-contained
//! because the real `ClosureStorage` is `pub(crate)` and invisible here, as with
//! the other `poc_fs_*` spikes.

use std::{
	rc::Rc,
	sync::Arc,
};

// Minimal `ClosureStorage` (the proven POC-8b shape), reproduced self-contained.
trait ClosureStorage: 'static {
	type Stored<'a, I: 'a, O: 'a>: 'a;
	fn call_once<'a, I: 'a, O: 'a>(
		stored: Self::Stored<'a, I, O>,
		input: I,
	) -> O;
}

struct BoxBrand;
impl ClosureStorage for BoxBrand {
	type Stored<'a, I: 'a, O: 'a> = Box<dyn FnOnce(I) -> O + 'a>;

	fn call_once<'a, I: 'a, O: 'a>(
		stored: Self::Stored<'a, I, O>,
		input: I,
	) -> O {
		stored(input)
	}
}

struct RcBrand;
impl ClosureStorage for RcBrand {
	type Stored<'a, I: 'a, O: 'a> = Rc<dyn Fn(I) -> O + 'a>;

	fn call_once<'a, I: 'a, O: 'a>(
		stored: Self::Stored<'a, I, O>,
		input: I,
	) -> O {
		(*stored)(input)
	}
}

struct ArcBrand;
impl ClosureStorage for ArcBrand {
	type Stored<'a, I: 'a, O: 'a> = Arc<dyn Fn(I) -> O + Send + Sync + 'a>;

	fn call_once<'a, I: 'a, O: 'a>(
		stored: Self::Stored<'a, I, O>,
		input: I,
	) -> O {
		(*stored)(input)
	}
}

// The mini erased spine. The `Store` parameter is LAST and DEFAULTS to `BoxBrand`,
// so `MiniFree<A>` == `MiniFree<A, BoxBrand>`. The suspended step stores its
// continuation via the store, exactly where the production `Free`'s `CatList`
// continuation element will be `Store`-parameterised.
enum MiniFree<A: 'static, Store: ClosureStorage = BoxBrand> {
	Pure(A),
	Ask(Store::Stored<'static, i32, MiniFree<A, Store>>),
}

fn run<A: 'static, Store: ClosureStorage>(
	mut program: MiniFree<A, Store>,
	env: i32,
) -> A {
	loop {
		match program {
			MiniFree::Pure(a) => return a,
			MiniFree::Ask(k) => program = Store::call_once(k, env),
		}
	}
}

fn assert_send_sync<T: Send + Sync>() {}

// (a) The churn-absorbing claim: a consumer written in the existing style, with no
// `Store` argument, compiles unchanged and runs. `MiniFree<i32>` is the Box spine.
#[test]
fn bare_default_is_box_and_runs() {
	let k: Box<dyn FnOnce(i32) -> MiniFree<i32> + 'static> =
		Box::new(|env| MiniFree::Pure(env + 1));
	let program: MiniFree<i32> = MiniFree::Ask(k);
	assert_eq!(run(program, 41), 42);
}

// (b) The Rc and Arc instantiations of the SAME type run.
#[test]
fn rc_and_arc_instantiations_run() {
	let kr: Rc<dyn Fn(i32) -> MiniFree<i32, RcBrand>> = Rc::new(|env| MiniFree::Pure(env + 1));
	let pr: MiniFree<i32, RcBrand> = MiniFree::Ask(kr);
	assert_eq!(run(pr, 41), 42);

	let ka: Arc<dyn Fn(i32) -> MiniFree<i32, ArcBrand> + Send + Sync> =
		Arc::new(|env| MiniFree::Pure(env + 1));
	let pa: MiniFree<i32, ArcBrand> = MiniFree::Ask(ka);
	assert_eq!(run(pa, 41), 42);
}

// (b cont.) The Arc instantiation is statically `Send + Sync`.
#[test]
fn arc_instantiation_is_send_sync() {
	assert_send_sync::<MiniFree<i32, ArcBrand>>();
}
