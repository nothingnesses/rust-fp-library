//! POC (remediation item 4 step 5.4): unify the `Coyoneda` path-syntax
//! constructor `lift` across the Box and Arc stores under ONE generic definition.
//!
//! Prior work split here. The earlier Coyoneda-under-`Store` spike unified the
//! `Coyoneda` TYPE and its `lower` bridge but kept construction per-store with two
//! separate functions (`make_box`/`make_arc`), because the bound sets differ: the
//! Box arm stores a `Box<dyn Inner>` and lowers by consuming it (no `Send`
//! needed), while the Arc arm stores an `Arc<dyn Inner + Send + Sync>` and lowers
//! by borrowing (cloning the stored value), so it requires the FUNCTOR VALUE
//! itself to be `Clone + Send + Sync`. A single `fn lift<A>(fa) -> ...` cannot name
//! both bound sets in one signature, exactly as a path-syntax `Free::pure` with
//! two definitions is crate-wide ambiguous.
//!
//! Resolution under test (the `Free::pure` resolution generalised): a per-store
//! construction trait `CoyoLift<'a, F, Store>`, implemented FOR the value type,
//! carries the per-store bound in its impl `where`-clause and does the cell
//! construction in its impl body. The single generic `Coyoneda::lift` is bounded
//! only by `A: CoyoLift<'a, F, Store>` and delegates, so its body never names
//! `Send`/`Sync`/`Clone`. The novel part the `Free::pure` `ValueFor` does NOT
//! cover: here the varying bound is on the functor value `F::Of<A>` (a GAT
//! projection), not just on `A`, so this POC runs on the real `Kind`/`Functor`/
//! `SendFunctor` machinery and a real brand (`OptionBrand`) to exercise the
//! projection-bound discharge directly rather than reduce it away with a concrete
//! stand-in functor. Confirms: one generic `lift` type-checks at both stores, the
//! Box arm accepts any value, the Arc arm is statically `Send + Sync`, and both
//! lower to the original functor value.

use {
	fp_library::{
		Apply,
		brands::{
			ArcBrand,
			BoxBrand,
			OptionBrand,
		},
		classes::{
			Functor,
			SendFunctor,
		},
		kinds::*,
	},
	std::sync::Arc,
};

// -- The Box arm's inner cell: lowers by consuming `Box<Self>`; algebra is
//    `Functor`-bound; no `Send`/`Sync` needed. Mirrors `CoyonedaInner`. --

trait BoxInner<'a, F, A: 'a>: 'a
where
	F: Kind_cdc7cd43dac7585f + 'a, {
	fn lower(self: Box<Self>) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		F: Functor;
}

struct BoxBase<'a, F, A: 'a>
where
	F: Kind_cdc7cd43dac7585f + 'a, {
	fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
}

impl<'a, F, A: 'a> BoxInner<'a, F, A> for BoxBase<'a, F, A>
where
	F: Kind_cdc7cd43dac7585f + 'a,
{
	fn lower(self: Box<Self>) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		F: Functor, {
		self.fa
	}
}

// -- The Arc arm's inner cell: lowers by borrowing `&self` (clones the value);
//    `Send + Sync` supertrait; algebra is `SendFunctor`-bound. Mirrors
//    `ArcCoyonedaLowerRef`. --

trait ArcInner<'a, F, A: 'a>: Send + Sync + 'a
where
	F: Kind_cdc7cd43dac7585f + 'a, {
	fn lower_ref(&self) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		F: SendFunctor,
		A: Send + Sync;
}

struct ArcBase<'a, F, A: 'a>
where
	F: Kind_cdc7cd43dac7585f<Of<'a, A>: Send + Sync> + 'a, {
	fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
}

impl<'a, F, A: 'a> ArcInner<'a, F, A> for ArcBase<'a, F, A>
where
	F: Kind_cdc7cd43dac7585f + 'a,
	Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone + Send + Sync,
{
	fn lower_ref(&self) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		F: SendFunctor,
		A: Send + Sync, {
		self.fa.clone()
	}
}

// -- The pointer-store trait: one GAT for the per-store outer pointer. Mirrors
//    the shipped core `CoyoStore`. --

trait CoyoStore: 'static {
	type Ptr<'a, F, A>: 'a
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		A: 'a;
}

impl CoyoStore for BoxBrand {
	type Ptr<'a, F, A>
		= Box<dyn BoxInner<'a, F, A> + 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		A: 'a;
}

impl CoyoStore for ArcBrand {
	// `Send + Sync` lives on the `ArcInner` supertrait, so the trait object and
	// the `Arc` holding it are `Send + Sync` automatically.
	type Ptr<'a, F, A>
		= Arc<dyn ArcInner<'a, F, A> + 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		A: 'a;
}

// -- The per-store construction abstraction (the novel bit). Implemented FOR the
//    value type `A`; each impl carries its store's bound in the `where`-clause and
//    builds the base-layer `Ptr` in the body. This is the `Free::pure` `ValueFor`
//    pattern, generalised: the Arc bound is on the functor value `F::Of<A>`, not
//    just on `A`. --

trait CoyoLift<'a, F, Store: CoyoStore>: 'a
where
	F: Kind_cdc7cd43dac7585f + 'a, {
	fn lift_ptr(
		fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Self>)
	) -> Store::Ptr<'a, F, Self>
	where
		Self: Sized;
}

impl<'a, F, A: 'a> CoyoLift<'a, F, BoxBrand> for A
where
	F: Kind_cdc7cd43dac7585f + 'a,
{
	fn lift_ptr(
		fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> <BoxBrand as CoyoStore>::Ptr<'a, F, A> {
		// The explicitly-typed local triggers the unsizing coercion to the trait
		// object before it is returned as the `Store::Ptr` projection.
		let cell: Box<dyn BoxInner<'a, F, A> + 'a> = Box::new(BoxBase {
			fa,
		});
		cell
	}
}

impl<'a, F, A: Send + Sync + 'a> CoyoLift<'a, F, ArcBrand> for A
where
	F: Kind_cdc7cd43dac7585f + 'a,
	Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone + Send + Sync,
{
	fn lift_ptr(
		fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> <ArcBrand as CoyoStore>::Ptr<'a, F, A> {
		let cell: Arc<dyn ArcInner<'a, F, A> + 'a> = Arc::new(ArcBase {
			fa,
		});
		cell
	}
}

// -- ONE Coyoneda type over the store, with the trailing-defaulted `Store`. --

struct Coyoneda<'a, F, A: 'a, Store: CoyoStore = BoxBrand>(Store::Ptr<'a, F, A>)
where
	F: Kind_cdc7cd43dac7585f + 'a;

// ONE generic, path-syntax `lift` over ALL stores: bounded only by the
// construction abstraction, the body names no `Send`/`Sync`/`Clone`.
impl<'a, F, A: 'a, Store: CoyoStore> Coyoneda<'a, F, A, Store>
where
	F: Kind_cdc7cd43dac7585f + 'a,
{
	fn lift(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> Self
	where
		A: CoyoLift<'a, F, Store>, {
		Coyoneda(<A as CoyoLift<'a, F, Store>>::lift_ptr(fa))
	}
}

// Per-arm, method-syntax `lower` (the receiver pins the store, so these stay
// per-store and each carries its own algebra bound).
impl<'a, F, A: 'a> Coyoneda<'a, F, A, BoxBrand>
where
	F: Kind_cdc7cd43dac7585f + 'a,
{
	fn lower(self) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		F: Functor, {
		self.0.lower()
	}
}

impl<'a, F, A: Send + Sync + 'a> Coyoneda<'a, F, A, ArcBrand>
where
	F: Kind_cdc7cd43dac7585f + 'a,
{
	fn lower_ref(&self) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		F: SendFunctor, {
		self.0.lower_ref()
	}
}

fn assert_send_sync<T: Send + Sync>() {}

// -- Validations --

// Box arm: the unified `lift` builds a `Coyoneda<_, _, BoxBrand>` and lowers it by
// consuming. The Box `CoyoLift` blanket impl accepts any value type.
#[test]
fn box_lift_lowers() {
	let coyo: Coyoneda<OptionBrand, i32, BoxBrand> = Coyoneda::lift(Some(42));
	assert_eq!(coyo.lower(), Some(42));
}

// Arc arm: the SAME unified `lift` builds a `Coyoneda<_, _, ArcBrand>` that is
// statically `Send + Sync` (the Arc `CoyoLift` impl required `Option<i32>: Clone +
// Send + Sync`, discharged transitively through `A: CoyoLift<_, _, ArcBrand>`
// without the generic `lift` naming it) and lowers by borrowing.
#[test]
fn arc_lift_is_send_sync_and_lowers() {
	assert_send_sync::<Coyoneda<'static, OptionBrand, i32, ArcBrand>>();
	let coyo: Coyoneda<OptionBrand, i32, ArcBrand> = Coyoneda::lift(Some(7));
	assert_eq!(coyo.lower_ref(), Some(7));
}

// The unified `lift` also resolves at the DEFAULTED store (Box) with the store
// unpinned at the call, the configuration that makes a second per-store
// definition crate-wide ambiguous; with one definition it is unambiguous.
#[test]
fn default_store_lift_lowers() {
	let coyo = Coyoneda::<OptionBrand, i32>::lift(Some(1));
	assert_eq!(coyo.lower(), Some(1));
}
