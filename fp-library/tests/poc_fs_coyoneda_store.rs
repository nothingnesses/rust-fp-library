//! POC: a single `Store`-parameterised `Coyoneda` (remediation item 4 step 5,
//! the substrate build; the Coyoneda-under-`Store` question).
//!
//! Question: can ONE `Coyoneda` type, generic over a pointer-store, express both
//! the Box case (lowers by consuming, need not be `Send`) and the Arc case
//! (statically `Send + Sync`, lowers by borrowing) under a single definition,
//! while preserving the free-functor shape (store the effect value plus a
//! deferred map, and apply the map only at `lower`)? Today the project ships
//! three separate Coyoneda types (`Coyoneda`, `RcCoyoneda`, `ArcCoyoneda`) for
//! exactly this per-store reason; the substrate decision wants them collapsed to
//! one. No prior POC exercised this: POC-8b reconciled the `FnOnce`/`Fn` split
//! for CONTINUATIONS via a by-value bridge but explicitly excluded Coyoneda.
//!
//! Design under test: the per-store axis is the OUTER pointer holding the inner
//! trait object (`Box<dyn Inner>` for Box, `Arc<dyn Inner + Send + Sync>` for
//! Arc), not the map closure (which is an ordinary `Fn` in both). A pointer-store
//! trait (`CoyoStore`) abstracts that pointer behind a GAT and bridges `lower` by
//! value, the same by-value trick POC-8b used for `call_once`. The Box arm's
//! `lower` consumes its box; the Arc arm's `lower` borrows through the owned arc
//! (so the stored value is cloned), matching the shipped `Coyoneda`/`ArcCoyoneda`
//! lowering. Construction stays per-store (`make_box`/`make_arc`), as POC-8b found
//! for closure construction; only the Coyoneda TYPE and its `lower` unify.

use std::sync::Arc;

// A concrete functor stands in for an effect's `F`. Coyoneda only needs
// `F: Functor` at `lower` time; `FVal` is a one-slot functor with a `map`.
#[derive(Debug, PartialEq, Eq)]
struct FVal<X>(X);
impl<X> FVal<X> {
	fn map<Y>(
		self,
		f: impl FnOnce(X) -> Y,
	) -> FVal<Y> {
		FVal(f(self.0))
	}
}

// -- The Box arm: lowers by consuming; the map is `FnOnce`; no `Send` needed. --

trait BoxInner<'a, A: 'a>: 'a {
	fn lower_boxed(self: Box<Self>) -> FVal<A>;
}
struct BoxCell<'a, B: 'a, A: 'a> {
	fb: FVal<B>,
	f: Box<dyn FnOnce(B) -> A + 'a>,
}
impl<'a, B: 'a, A: 'a> BoxInner<'a, A> for BoxCell<'a, B, A> {
	fn lower_boxed(self: Box<Self>) -> FVal<A> {
		let this = *self;
		// `map` takes `FnOnce`; the boxed `FnOnce` map is moved in and run once.
		this.fb.map(this.f)
	}
}

// -- The Arc arm: lowers by borrowing (clones the value); `Send + Sync`. --

trait ArcInner<'a, A: 'a>: Send + Sync + 'a {
	fn lower_ref(&self) -> FVal<A>;
}
struct ArcCell<'a, B: Clone + Send + Sync + 'a, A: 'a> {
	value: B,
	f: Arc<dyn Fn(B) -> A + Send + Sync + 'a>,
}
impl<'a, B: Clone + Send + Sync + 'a, A: 'a> ArcInner<'a, A> for ArcCell<'a, B, A> {
	fn lower_ref(&self) -> FVal<A> {
		// Cannot move the value out of `&self`, so clone it (the shipped
		// `ArcCoyoneda` does the same); the map is a reusable `Fn`.
		FVal((self.f)(self.value.clone()))
	}
}

// -- The pointer-store trait: one GAT for the per-store pointer, one by-value
//    `lower` bridge. This is the sibling of POC-8b's `ClosureStorage`. --

trait CoyoStore: 'static {
	type Ptr<'a, A: 'a>: 'a;
	fn lower<'a, A: 'a>(ptr: Self::Ptr<'a, A>) -> FVal<A>;
}

struct BoxStore;
impl CoyoStore for BoxStore {
	type Ptr<'a, A: 'a> = Box<dyn BoxInner<'a, A> + 'a>;

	fn lower<'a, A: 'a>(ptr: Self::Ptr<'a, A>) -> FVal<A> {
		ptr.lower_boxed()
	}
}

struct ArcStore;
impl CoyoStore for ArcStore {
	// The `Send + Sync` lives on the `ArcInner` supertrait, so the trait object
	// (and the `Arc` holding it) are `Send + Sync` automatically.
	type Ptr<'a, A: 'a> = Arc<dyn ArcInner<'a, A> + 'a>;

	fn lower<'a, A: 'a>(ptr: Self::Ptr<'a, A>) -> FVal<A> {
		ptr.lower_ref()
	}
}

// ONE Coyoneda type over the store.
struct Coyoneda<'a, Store: CoyoStore, A: 'a>(Store::Ptr<'a, A>);
impl<'a, Store: CoyoStore, A: 'a> Coyoneda<'a, Store, A> {
	fn lower(self) -> FVal<A> {
		Store::lower(self.0)
	}
}

// Construction stays per-store (per POC-8b): the bounds differ (Arc needs
// `Clone + Send + Sync`), so this cannot be one generic constructor.
fn make_box<'a, B: 'a, A: 'a>(
	value: B,
	f: impl FnOnce(B) -> A + 'a,
) -> Coyoneda<'a, BoxStore, A> {
	// The explicitly-typed local triggers the unsizing coercion to the trait
	// object before it is stored in the GAT-typed field.
	let ptr: Box<dyn BoxInner<'a, A> + 'a> = Box::new(BoxCell {
		fb: FVal(value),
		f: Box::new(f),
	});
	Coyoneda(ptr)
}
fn make_arc<'a, B: Clone + Send + Sync + 'a, A: 'a>(
	value: B,
	f: impl Fn(B) -> A + Send + Sync + 'a,
) -> Coyoneda<'a, ArcStore, A> {
	let ptr: Arc<dyn ArcInner<'a, A> + 'a> = Arc::new(ArcCell {
		value,
		f: Arc::new(f),
	});
	Coyoneda(ptr)
}

fn assert_send_sync<T: Send + Sync>() {}

// -- Validations --

// Box arm: one `Coyoneda<BoxStore, _>` stores a deferred map and lowers it,
// applying the map at `lower` (the free-functor shape). A non-`Copy` capture in
// the map is fine for the Box store.
#[test]
fn box_coyoneda_lowers_a_deferred_map() {
	let suffix = String::from("!"); // non-Copy capture, moved into the FnOnce map
	let coyo: Coyoneda<BoxStore, String> = make_box(5_i32, move |b| format!("{b}{suffix}"));
	assert_eq!(coyo.lower(), FVal(String::from("5!")));
}

// Arc arm: the SAME `Coyoneda` type instantiated at `ArcStore` is statically
// `Send + Sync`, and lowers correctly (cloning the stored value through the
// borrowing `lower`).
#[test]
fn arc_coyoneda_is_send_sync_and_lowers() {
	assert_send_sync::<Coyoneda<'static, ArcStore, i32>>();
	let coyo: Coyoneda<ArcStore, i32> = make_arc(5_i32, |b| b + 1);
	assert_eq!(coyo.lower(), FVal(6));
}

// A coproduct row of the unified Coyoneda cells at `ArcStore` is itself
// `Send + Sync`, and the active arm dispatches and lowers, this is the row shape
// the FS-1 substrate needs the Arc store to carry, the combination no POC had
// exercised.
#[test]
fn arc_row_of_coyoneda_cells_is_send_sync_and_dispatches() {
	enum Coproduct<H, T> {
		Inl(H),
		Inr(T),
	}
	enum CNil {}

	type Row<'a> =
		Coproduct<Coyoneda<'a, ArcStore, i32>, Coproduct<Coyoneda<'a, ArcStore, i32>, CNil>>;

	assert_send_sync::<Row<'static>>();

	// Active arm is the second cell (deep position), found and lowered directly.
	let row: Row = Coproduct::Inr(Coproduct::Inl(make_arc(10_i32, |b| b * 2)));
	let lowered = match row {
		Coproduct::Inl(coyo) => coyo.lower(),
		Coproduct::Inr(Coproduct::Inl(coyo)) => coyo.lower(),
		Coproduct::Inr(Coproduct::Inr(cnil)) => match cnil {},
	};
	assert_eq!(lowered, FVal(20));
}
