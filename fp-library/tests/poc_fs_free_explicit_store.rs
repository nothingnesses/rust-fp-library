//! Proof-of-concept for the flat `Store`-parameterised concrete `FreeExplicit`.
//!
//! Validates the unification mechanism for the concrete (non-`'static`,
//! O(N)-bind) free-monad family: ONE flat struct
//! `FreeExplicit<'a, A, Store>` whose `Wrap` variant holds the recursion
//! indirection pointer `Store::SelfPtr<Self>`, where the per-store axis is
//! supplied by a small pointer-storage trait `ExplicitStore`
//! (`Box<T>`/`Rc<T>`/`Arc<T>`). This is the sibling of the closure-storage and
//! Coyoneda pointer-storage traits, applied to the concrete free's self
//! pointer rather than to a closure or an existential cell.
//!
//! The novel parts this isolates (the then-standalone Rc/Arc explicit types
//! had already proven the per-arm parts on a generic functor, so
//! the POC uses a concrete `Identity` functor to keep the focus on the flat
//! `Store`-axis shape):
//!
//! 1. The flat struct with `Store::SelfPtr<Self>` inside `Wrap` compiles (the
//!    GAT self-reference plus the lifetime threading).
//! 2. A SINGLE conditional `Clone` impl makes the `Rc`/`Arc` arms multi-shot
//!    while leaving the `Box` arm not-`Clone` (because `Box<Self>: Clone`
//!    requires `Self: Clone`, which is never proven, so the impl does not apply
//!    to the `Box` arm).
//! 3. A SINGLE iterative `Drop`, generic over `Store`, dismantles deep `Wrap`
//!    chains without overflowing, via one `ExplicitStore::try_into_inner`
//!    helper (`Box`: move out; `Rc`/`Arc`: `try_unwrap`, stop when shared).
//! 4. The `Arc` arm is statically `Send + Sync` (auto-derived through the
//!    concrete `Identity<Arc<Self>>`; the real generic-`F` buildout recovers
//!    this through the associated-type-bound trick the then-standalone Arc
//!    explicit type shipped).
//! 5. The `Box` arm carries a non-`'static` borrowed payload (the
//!    borrowed-span parity property) and has no `Clone` bound on
//!    `bind`/`evaluate`.

#![expect(
	clippy::expect_used,
	clippy::wrong_self_convention,
	reason = "POC test crate: `to_view` consumes `self` to mirror the real API name, and the linear-consumption guard uses `expect`"
)]

use std::{
	rc::Rc,
	sync::Arc,
};

// -- Minimal store brands (stand-ins for the crate's BoxBrand/RcBrand/ArcBrand) --

struct BoxBrand;
struct RcBrand;
struct ArcBrand;

/// The per-store recursion-indirection pointer for the concrete free monad.
///
/// `SelfPtr<'a, T>` is the sized pointer that `FreeExplicit`'s `Wrap` variant
/// stores so the recursive type has heap indirection: `Box<T>` for the
/// single-shot default, `Rc<T>`/`Arc<T>` for the multi-shot arms.
/// `try_into_inner` recovers ownership of the pointee (always for `Box`; for
/// `Rc`/`Arc` only when the pointer is unique), which is what makes the
/// iterative `Drop` generic over the store.
trait ExplicitStore: 'static {
	type SelfPtr<'a, T: 'a>: 'a;

	fn try_into_inner<'a, T: 'a>(ptr: Self::SelfPtr<'a, T>) -> Option<T>;
}

impl ExplicitStore for BoxBrand {
	type SelfPtr<'a, T: 'a> = Box<T>;

	fn try_into_inner<'a, T: 'a>(ptr: Self::SelfPtr<'a, T>) -> Option<T> {
		Some(*ptr)
	}
}

impl ExplicitStore for RcBrand {
	type SelfPtr<'a, T: 'a> = Rc<T>;

	fn try_into_inner<'a, T: 'a>(ptr: Self::SelfPtr<'a, T>) -> Option<T> {
		Rc::try_unwrap(ptr).ok()
	}
}

impl ExplicitStore for ArcBrand {
	type SelfPtr<'a, T: 'a> = Arc<T>;

	fn try_into_inner<'a, T: 'a>(ptr: Self::SelfPtr<'a, T>) -> Option<T> {
		Arc::try_unwrap(ptr).ok()
	}
}

// -- A concrete functor for the POC --

/// The identity functor; `map` applies the function to the single payload.
struct Identity<T>(T);

// -- The flat Store-parameterised concrete free monad --

/// The internal view: a pure value or a suspended `Identity` layer whose
/// payload is the store's self-pointer to the next step.
enum View<'a, A: 'a, Store: ExplicitStore> {
	Pure(A),
	Wrap(Identity<<Store as ExplicitStore>::SelfPtr<'a, FreeExplicit<'a, A, Store>>>),
}

/// Naive recursive free monad over `Identity`, flat (`{ view }`) for every
/// store; the per-store axis is the `Wrap` indirection pointer.
struct FreeExplicit<'a, A: 'a, Store: ExplicitStore> {
	view: Option<View<'a, A, Store>>,
}

// Path-syntax constructors: ONE definition over all stores (no `Self`-receiver,
// so a second per-store definition would be E0034-ambiguous crate-wide).
impl<'a, A: 'a, Store: ExplicitStore> FreeExplicit<'a, A, Store> {
	fn pure(a: A) -> Self {
		FreeExplicit {
			view: Some(View::Pure(a)),
		}
	}

	fn wrap(
		layer: Identity<<Store as ExplicitStore>::SelfPtr<'a, FreeExplicit<'a, A, Store>>>
	) -> Self {
		FreeExplicit {
			view: Some(View::Wrap(layer)),
		}
	}

	/// Decompose into the view. Uniform and clone-free for every store, because
	/// the struct is owned by value (the sharing pointer is inside `Wrap`, not
	/// around the struct).
	fn to_view(mut self) -> View<'a, A, Store> {
		self.view.take().expect("FreeExplicit consumed exactly once")
	}
}

// ONE conditional `Clone`: applies only where the self-pointer is `Clone`
// (`Rc<Self>`/`Arc<Self>` always are; `Box<Self>: Clone` needs `Self: Clone`,
// never proven, so the `Box` arm stays not-`Clone`).
impl<'a, A, Store> Clone for FreeExplicit<'a, A, Store>
where
	A: Clone + 'a,
	Store: ExplicitStore,
	<Store as ExplicitStore>::SelfPtr<'a, FreeExplicit<'a, A, Store>>: Clone,
{
	fn clone(&self) -> Self {
		let view = match self.view.as_ref() {
			None => None,
			Some(View::Pure(a)) => Some(View::Pure(a.clone())),
			Some(View::Wrap(Identity(ptr))) => Some(View::Wrap(Identity(ptr.clone()))),
		};
		FreeExplicit {
			view,
		}
	}
}

// ONE iterative `Drop`, generic over `Store` (no bounds beyond the struct's),
// draining a deep `Wrap` chain via `try_into_inner`.
impl<'a, A: 'a, Store: ExplicitStore> Drop for FreeExplicit<'a, A, Store> {
	fn drop(&mut self) {
		let mut current = self.view.take();
		while let Some(view) = current {
			match view {
				View::Pure(_) => current = None,
				View::Wrap(Identity(ptr)) => {
					current = match Store::try_into_inner(ptr) {
						Some(mut inner) => inner.view.take(),
						None => None,
					};
				}
			}
		}
	}
}

// -- Box arm: single-shot, no `Clone` bound --

impl<'a, A: 'a> FreeExplicit<'a, A, BoxBrand> {
	fn bind<B: 'a>(
		self,
		f: impl Fn(A) -> FreeExplicit<'a, B, BoxBrand> + 'a,
	) -> FreeExplicit<'a, B, BoxBrand> {
		let boxed: Rc<dyn Fn(A) -> FreeExplicit<'a, B, BoxBrand> + 'a> = Rc::new(f);
		self.bind_boxed(boxed)
	}

	fn bind_boxed<B: 'a>(
		self,
		f: Rc<dyn Fn(A) -> FreeExplicit<'a, B, BoxBrand> + 'a>,
	) -> FreeExplicit<'a, B, BoxBrand> {
		match self.to_view() {
			View::Pure(a) => f(a),
			View::Wrap(Identity(boxed_self)) => {
				let next = Rc::clone(&f);
				FreeExplicit::wrap(Identity(Box::new((*boxed_self).bind_boxed(next))))
			}
		}
	}

	fn evaluate(self) -> A {
		let mut current = self;
		loop {
			match current.to_view() {
				View::Pure(a) => return a,
				View::Wrap(Identity(boxed)) => current = *boxed,
			}
		}
	}
}

// -- Rc arm: multi-shot, `Clone`-bounded methods --

impl<'a, A: Clone + 'a> FreeExplicit<'a, A, RcBrand> {
	fn bind<B: Clone + 'a>(
		self,
		f: impl Fn(A) -> FreeExplicit<'a, B, RcBrand> + 'a,
	) -> FreeExplicit<'a, B, RcBrand> {
		let boxed: Rc<dyn Fn(A) -> FreeExplicit<'a, B, RcBrand> + 'a> = Rc::new(f);
		self.bind_boxed(boxed)
	}

	fn bind_boxed<B: Clone + 'a>(
		self,
		f: Rc<dyn Fn(A) -> FreeExplicit<'a, B, RcBrand> + 'a>,
	) -> FreeExplicit<'a, B, RcBrand> {
		match self.to_view() {
			View::Pure(a) => f(a),
			View::Wrap(Identity(rc_self)) => {
				let inner = Rc::try_unwrap(rc_self).unwrap_or_else(|shared| (*shared).clone());
				let next = Rc::clone(&f);
				FreeExplicit::wrap(Identity(Rc::new(inner.bind_boxed(next))))
			}
		}
	}

	fn evaluate(self) -> A {
		let mut current = self;
		loop {
			match current.to_view() {
				View::Pure(a) => return a,
				View::Wrap(Identity(rc)) => {
					current = Rc::try_unwrap(rc).unwrap_or_else(|shared| (*shared).clone());
				}
			}
		}
	}

	fn lower_ref(&self) -> A {
		self.clone().evaluate()
	}
}

// -- Arc arm: multi-shot + thread-safe --

impl<'a, A: Clone + 'a> FreeExplicit<'a, A, ArcBrand> {
	fn evaluate(self) -> A {
		let mut current = self;
		loop {
			match current.to_view() {
				View::Pure(a) => return a,
				View::Wrap(Identity(arc)) => {
					current = Arc::try_unwrap(arc).unwrap_or_else(|shared| (*shared).clone());
				}
			}
		}
	}
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn box_arm_pure_wrap_bind_evaluate_and_non_static_payload() {
	// pure / evaluate.
	let free = FreeExplicit::<i32, BoxBrand>::pure(42);
	assert_eq!(free.evaluate(), 42);

	// wrap / evaluate.
	let inner = FreeExplicit::<i32, BoxBrand>::pure(7);
	let wrapped = FreeExplicit::<i32, BoxBrand>::wrap(Identity(Box::new(inner)));
	assert_eq!(wrapped.evaluate(), 7);

	// bind chain.
	let chained = FreeExplicit::<i32, BoxBrand>::pure(2)
		.bind(|x| FreeExplicit::pure(x + 1))
		.bind(|x| FreeExplicit::pure(x * 10));
	assert_eq!(chained.evaluate(), 30);

	// Non-`'static` borrowed payload: the defining property of the Explicit
	// family, and the borrowed-span parity property. The `Box` arm carries
	// it with no `Clone` bound.
	let owned = String::from("borrowed");
	let borrowed: &str = owned.as_str();
	let prog: FreeExplicit<'_, &str, BoxBrand> = FreeExplicit::pure(borrowed);
	assert_eq!(prog.evaluate().len(), 8);
}

#[test]
fn rc_arm_clones_into_independent_multi_shot_branches() {
	let program = FreeExplicit::<i32, RcBrand>::pure(10).bind(|x| FreeExplicit::pure(x + 1));
	let branch = program.clone();
	// The same stored program drives two independent branches.
	assert_eq!(branch.evaluate(), 11);
	assert_eq!(program.evaluate(), 11);

	// Non-consuming lower_ref runs repeatedly.
	let p = FreeExplicit::<i32, RcBrand>::pure(7).bind(|x| FreeExplicit::pure(x * 6));
	assert_eq!(p.lower_ref(), 42);
	assert_eq!(p.lower_ref(), 42);
	assert_eq!(p.evaluate(), 42);
}

#[test]
fn rc_arm_deep_drop_does_not_overflow() {
	const DEPTH: usize = 100_000;
	let mut free = FreeExplicit::<i32, RcBrand>::pure(0);
	for _ in 0 .. DEPTH {
		free = FreeExplicit::wrap(Identity(Rc::new(free)));
	}
	drop(free);
}

#[test]
fn box_arm_deep_evaluate_and_drop_do_not_overflow() {
	const DEPTH: usize = 100_000;
	let mut free = FreeExplicit::<i32, BoxBrand>::pure(0);
	for _ in 0 .. DEPTH {
		free = FreeExplicit::wrap(Identity(Box::new(free)));
	}
	// Build a second deep chain to drop without evaluating.
	let mut to_drop = FreeExplicit::<i32, BoxBrand>::pure(0);
	for _ in 0 .. DEPTH {
		to_drop = FreeExplicit::wrap(Identity(Box::new(to_drop)));
	}
	assert_eq!(free.evaluate(), 0);
	drop(to_drop);
}

#[test]
fn arc_arm_is_send_sync_and_deep_drops() {
	// Static thread-safety assertion (auto-derived through the concrete
	// `Identity<Arc<Self>>`).
	assert_send_sync::<FreeExplicit<'static, i32, ArcBrand>>();

	const DEPTH: usize = 100_000;
	let mut free = FreeExplicit::<i32, ArcBrand>::pure(0);
	for _ in 0 .. DEPTH {
		free = FreeExplicit::wrap(Identity(Arc::new(free)));
	}
	assert_eq!(free.clone().evaluate(), 0);
	drop(free);
}
