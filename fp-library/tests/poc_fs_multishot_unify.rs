//! POC (item 4 step 5.2 / OQ-6H): can the Rc and Arc arms share one generic
//! multi-shot body, with only the per-store bound differing (`Arc` requires
//! `Send + Sync`, `Rc` does not)?
//!
//! The crux is carrying a per-store bound into generic code. A trait method
//! `fn erase<A>(a: A) -> Self::Erased` on the store cannot add `Send + Sync`
//! only for the Arc impl, and a bound `A: ValueFor<ArcBrand>` only hands generic
//! code the trait's SUPERTRAITS, not the impl's bounds (a first attempt failed
//! exactly there: `A cannot be sent between threads safely`). The mechanism that
//! works puts the value operations ON the bound-carrying trait `ValueFor<S>`,
//! with per-store impls whose bodies see their own bound: the Arc impl is
//! `impl<A: Clone + Send + Sync + 'static> ValueFor<ArcBrand> for A`, so inside
//! its `erase` the value IS `Send + Sync`. Generic code bounds `A: ValueFor<S>`
//! and calls `a.erase()` / `A::recover(..)`; each store's bound is satisfied
//! inside its impl.
//!
//! This spike validates:
//!   (a) one generic body (`round_trip`) erases-then-recovers and type-checks
//!       for BOTH `Rc` and `Arc`;
//!   (b) the `Arc` store's erased cell is statically `Send + Sync`;
//!   (c) the `Rc` store accepts a NON-`Send` value (`Rc<i32>`), which the Arc
//!       bound rejects, the difference that justifies two stores.

use std::{
	any::Any,
	rc::Rc,
	sync::Arc,
};

// The multi-shot store contributes only its shared erased-cell type; the value
// operations live on `ValueFor<S>` so each store's bound is local to its impl.
trait MultiShotStore: 'static {
	type Erased: Clone + 'static;
}

// The per-(store, value) bound carrier. `erase`/`recover` live here, so the Arc
// impl's body sees `A: Send + Sync` and the Rc impl's does not.
trait ValueFor<S: MultiShotStore>: Clone + 'static {
	fn erase(self) -> S::Erased;
	fn recover(erased: S::Erased) -> Self;
}

struct RcBrand;
impl MultiShotStore for RcBrand {
	type Erased = Rc<dyn Any>;
}
impl<A: Clone + 'static> ValueFor<RcBrand> for A {
	fn erase(self) -> Rc<dyn Any> {
		Rc::new(self)
	}

	fn recover(erased: Rc<dyn Any>) -> A {
		match erased.downcast::<A>() {
			Ok(rc) => Rc::try_unwrap(rc).unwrap_or_else(|shared| (*shared).clone()),
			Err(_) => unreachable_type_mismatch(),
		}
	}
}

struct ArcBrand;
impl MultiShotStore for ArcBrand {
	type Erased = Arc<dyn Any + Send + Sync>;
}
impl<A: Clone + Send + Sync + 'static> ValueFor<ArcBrand> for A {
	fn erase(self) -> Arc<dyn Any + Send + Sync> {
		// `A: Send + Sync` is known here (the impl's bound), so the unsizing
		// coercion to `Arc<dyn Any + Send + Sync>` type-checks.
		Arc::new(self)
	}

	fn recover(erased: Arc<dyn Any + Send + Sync>) -> A {
		match erased.downcast::<A>() {
			Ok(arc) => Arc::try_unwrap(arc).unwrap_or_else(|shared| (*shared).clone()),
			Err(_) => unreachable_type_mismatch(),
		}
	}
}

fn unreachable_type_mismatch<A>() -> A {
	#[expect(clippy::panic, reason = "POC: the erased type is maintained by construction")]
	{
		panic!("type maintained by construction")
	}
}

fn assert_send_sync<T: Send + Sync>() {}

// (a) ONE generic body serves both stores: erase a value then recover it. It
// needs only `A: ValueFor<S>`; the per-store bound is enforced inside the impl.
fn round_trip<S: MultiShotStore, A: ValueFor<S>>(a: A) -> A {
	A::recover(a.erase())
}

#[test]
fn one_generic_body_serves_rc_and_arc() {
	assert_eq!(round_trip::<RcBrand, i32>(7), 7);
	assert_eq!(round_trip::<ArcBrand, i32>(8), 8);
}

// (b) The Arc store's erased cell is statically `Send + Sync`, so a structure
// holding it (the Arc `Free`'s view and queue) is `Send + Sync` by inference.
#[test]
fn arc_erased_is_send_sync() {
	assert_send_sync::<<ArcBrand as MultiShotStore>::Erased>();
}

// (c) The Rc store accepts a NON-`Send` value (`Rc<i32>` is `Clone + 'static`
// but not `Send`); the Arc store's bound rejects it at compile time. This is the
// genuine reason both stores exist, and why the bound must be per-store.
#[test]
fn rc_accepts_non_send_values() {
	let non_send: Rc<i32> = Rc::new(5);
	let out: Rc<i32> = round_trip::<RcBrand, Rc<i32>>(non_send);
	assert_eq!(*out, 5);
	// round_trip::<ArcBrand, Rc<i32>>(...) would NOT compile: `Rc<i32>: Send` is
	// not satisfied, so `Rc<i32>: ValueFor<ArcBrand>` does not hold.
}
