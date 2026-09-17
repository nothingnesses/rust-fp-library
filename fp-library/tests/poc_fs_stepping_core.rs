//! POC (item 4 step 5.2 / OQ-6C): how much of the erased free-monad stepping
//! core (`to_view`) can stay a single `Store`-generic implementation?
//!
//! `to_view` is not pure invocation. On a suspended layer it does two things
//! that touch the store: (a) it constructs a downcast/unbox continuation inline,
//! and (b) it threads the pending continuation queue into the suspended functor
//! layer via the functor's `map`, which is `Fn`-bounded (verified against the
//! real `Functor::map`, whose closure bound is `impl Fn(A) -> B`). A real lazy
//! functor (such as `Thunk`) stores that `Fn` threading closure, so for the Arc
//! store the closure (and therefore its captures) must be `Send + Sync`.
//!
//! This spike isolates the two concerns and lets the compiler decide which can
//! be uniform:
//!   Test 1 validates that the capture-free downcast continuation (a) is built
//!   for every store by one `ClosureStorage::from_fn(impl Fn + Send + Sync)`
//!   bridge, and that the Arc result is `Send + Sync`.
//!   Test 2 validates the decisive fact behind (b): the per-store continuation
//!   queue, which the threading closure must capture, is `Send + Sync` for the
//!   Arc store but is intentionally not `Send` for the Box store. A single
//!   `Fn + Send + Sync`-bounded storage method therefore cannot store the
//!   threading closure for both stores, so layer-threading is per-`Store`.
//!
//! Conclusion the spike supports: OQ-6C approach A holds for the capture-free
//! continuations (one generic `from_fn`), but the queue-threading half of
//! `to_view` is per-`Store` (approach B for stepping), because the threading
//! closure's `Send`-ness is fixed by the store's queue and cannot be uniformly
//! bounded.

use std::{
	any::Any,
	rc::Rc,
	sync::Arc,
};

type Erased = Box<dyn Any>;

trait ClosureStorage: 'static {
	type Stored<'a, I: 'a, O: 'a>: 'a;

	fn call_once<'a, I: 'a, O: 'a>(
		stored: Self::Stored<'a, I, O>,
		input: I,
	) -> O;

	/// Build a stored closure from a capture-free (`Fn + Send + Sync`) body.
	/// Used for the downcast/unbox continuations the core constructs inline,
	/// which capture nothing, so the strict bound is harmless.
	fn from_fn<'a, I: 'a, O: 'a>(f: impl Fn(I) -> O + Send + Sync + 'a) -> Self::Stored<'a, I, O>;
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

	fn from_fn<'a, I: 'a, O: 'a>(f: impl Fn(I) -> O + Send + Sync + 'a) -> Self::Stored<'a, I, O> {
		Box::new(f)
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

	fn from_fn<'a, I: 'a, O: 'a>(f: impl Fn(I) -> O + Send + Sync + 'a) -> Self::Stored<'a, I, O> {
		Rc::new(f)
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

	fn from_fn<'a, I: 'a, O: 'a>(f: impl Fn(I) -> O + Send + Sync + 'a) -> Self::Stored<'a, I, O> {
		Arc::new(f)
	}
}

fn assert_send_sync<T: Send + Sync>() {}

// Recover a concrete value from an erased box. The type is maintained by
// construction in this spike; the `expect` is acknowledged via `#[expect]`
// (the house lint style, mirroring `free.rs`).
fn dc<A: 'static>(val: Erased) -> A {
	#[expect(clippy::expect_used, reason = "POC: type is maintained by construction")]
	let a = *val.downcast::<A>().expect("type maintained by invariant");
	a
}

// -- Test 1: the capture-free downcast continuation is generic across stores. --
// `make_downcast` is one generic body (no per-store code). It mirrors the
// downcast continuation `to_view` builds: take an erased value, recover the
// concrete `A`, re-erase it. It captures nothing, so `from_fn`'s strict bound
// is satisfied for every store.

fn make_downcast<A: 'static, Store: ClosureStorage>() -> Store::Stored<'static, Erased, Erased> {
	Store::from_fn(|val: Erased| Box::new(dc::<A>(val)) as Erased)
}

#[test]
fn downcast_continuation_is_generic_across_stores() {
	// Box.
	let kb = make_downcast::<i32, BoxBrand>();
	let out = BoxBrand::call_once(kb, Box::new(7i32) as Erased);
	assert_eq!(dc::<i32>(out), 7);

	// Rc.
	let kr = make_downcast::<i32, RcBrand>();
	let out = RcBrand::call_once(kr, Box::new(8i32) as Erased);
	assert_eq!(dc::<i32>(out), 8);

	// Arc, and the Arc stored continuation is statically Send + Sync.
	let ka = make_downcast::<i32, ArcBrand>();
	let out = ArcBrand::call_once(ka, Box::new(9i32) as Erased);
	assert_eq!(dc::<i32>(out), 9);
	assert_send_sync::<<ArcBrand as ClosureStorage>::Stored<'static, Erased, Erased>>();
}

// -- Test 2: the queue the threading closure must capture is per-store. --
// The pending continuation queue is `Vec<Store::Stored<...>>`. The `Fn` closure
// `to_view` passes to the layer's `map` captures this queue, so the closure is
// `Send + Sync` only if the queue is. For Arc it is; for Box it is not. A single
// `Fn + Send + Sync`-bounded storage method therefore cannot store the threading
// closure for both stores, so the layer-threading half of `to_view` is
// per-`Store`.

type Queue<Store> = Vec<<Store as ClosureStorage>::Stored<'static, Erased, Erased>>;

#[test]
fn arc_queue_is_send_sync_so_per_store_arc_stepping_can_be_send_sync() {
	assert_send_sync::<Queue<ArcBrand>>();
}

// The Box queue is intentionally NOT asserted `Send`: `Box<dyn FnOnce>` is not
// `Send`, so a closure capturing `Queue<BoxBrand>` is not `Send`, which is why
// the uniform `from_fn` (a `Send + Sync` bound) cannot store the Box store's
// threading closure. Uncommenting the next line must fail to compile, the
// recorded evidence that layer-threading is per-`Store` (OQ-6C approach B for
// stepping), not generic:
//
//   assert_send_sync::<Queue<BoxBrand>>();
