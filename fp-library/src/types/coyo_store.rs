//! FS-1 substrate: the per-`Store` pointer-storage interface for a unified
//! `Coyoneda`, crate-internal work in progress.
//!
//! This module is part of the FS-1 substrate build (remediation-plan review-2,
//! item 4 step 5.4). It lives in core `crate::types` (not under the
//! `effects`-feature-gated subsystem) because the `Coyoneda` it parameterises is
//! itself core and compiles with `effects` off, so a core type's store bound must
//! also be core (the same constraint that relocated `ClosureStorage` to core).
//!
//! `CoyoStore` is the sibling of `ClosureStorage` for the `Coyoneda` axis: where
//! `ClosureStorage` abstracts the per-`Store` stored CLOSURE, `CoyoStore` abstracts
//! the per-`Store` OUTER POINTER that holds a `Coyoneda`'s inner existential cell
//! (`Box<dyn ...Inner>` for the one-shot Box store, `Rc`/`Arc<dyn ...Inner>` for
//! the refcounted stores). The map closure inside a cell is an ordinary `Fn` in
//! every store; only the outer pointer, and how a cell is built and lowered, vary,
//! which is the per-`Store` axis these traits name.
//!
//! Two traits split the axis along the path-syntax/method-syntax line:
//!
//! - `CoyoStore` carries the outer-pointer GAT `Ptr`. It hosts all three stores
//!   uniformly (the GAT is the only thing every store shares without a bound
//!   difference).
//! - `CoyoLift` is the per-`Store` CONSTRUCTION abstraction. Implemented FOR the
//!   value type, each impl carries its store's bound in its `where`-clause and
//!   builds the base-layer `Ptr` from a functor value. This lets ONE generic,
//!   path-syntax `Coyoneda::lift` serve every store by delegating here (a second
//!   per-store definition would be crate-wide ambiguous), mirroring how `ValueFor`
//!   unifies `Free::pure`; the novel part is that the Arc bound here falls on the
//!   functor value `F::Of<A>`, not just on `A`.
//!
//! Lowering and mapping are deliberately NOT on these traits: they are
//! method-syntax operations whose receiver pins the store, so they stay per-arm on
//! the `Coyoneda` type (the Box and Rc arms lower under `F: Functor`, the Arc arm
//! under `F: SendFunctor` with `A: Send + Sync`, a bound a single shared trait
//! method could not carry).
//!
//! Documentation status: like the other in-progress FS-1 substrate modules, this
//! module intentionally does NOT yet use the `#[fp_macros::document_module]`
//! wrapper. The wrapper and full per-item documentation are added once the
//! substrate is settled (remediation item 11), which clears this tracked,
//! temporary exception.

#![allow(
	dead_code,
	reason = "FS-1 rebuild in progress (item 4 step 5.4): this pointer-storage interface and its construction abstraction are consumed by the Store-parameterised unified Coyoneda built in later sub-steps; item 20 sweeps any residual allowances at the end of the rebuild."
)]

use {
	crate::{
		Apply,
		brands::{
			ArcBrand,
			BoxBrand,
			RcBrand,
		},
		kinds::*,
		types::{
			arc_coyoneda::{
				ArcCoyonedaBase,
				ArcCoyonedaLowerRef,
			},
			coyoneda::{
				CoyonedaBase,
				CoyonedaInner,
			},
			rc_coyoneda::{
				RcCoyonedaBase,
				RcCoyonedaLowerRef,
			},
		},
	},
	std::{
		rc::Rc,
		sync::Arc,
	},
};

/// One pointer-storage interface over the per-`Store` outer pointer that holds a
/// `Coyoneda`'s inner existential cell. The associated `Ptr` GAT is the store's
/// outer pointer to the cell (`Box`/`Rc`/`Arc` of the store's `dyn ...Inner`).
///
/// `pub` (mirroring `ClosureStorage` per the OQ-6D resolution) because it bounds
/// the public `Coyoneda`; the per-store inner traits are `pub` alongside it so the
/// `Ptr` associated type does not leak a crate-private trait.
pub trait CoyoStore: 'static {
	/// The per-`Store` outer pointer to the inner cell (a `Box`/`Rc`/`Arc` of the
	/// store's `dyn ...Inner` existential).
	type Ptr<'a, F, A>: 'a
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		A: 'a;
}

impl CoyoStore for BoxBrand {
	type Ptr<'a, F, A>
		= Box<dyn CoyonedaInner<'a, F, A> + 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		A: 'a;
}

impl CoyoStore for RcBrand {
	type Ptr<'a, F, A>
		= Rc<dyn RcCoyonedaLowerRef<'a, F, A> + 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		A: 'a;
}

impl CoyoStore for ArcBrand {
	// `Send + Sync` lives on the `ArcCoyonedaLowerRef` supertrait, so the trait
	// object and the `Arc` holding it are `Send + Sync` automatically.
	type Ptr<'a, F, A>
		= Arc<dyn ArcCoyonedaLowerRef<'a, F, A> + 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		A: 'a;
}

/// The per-`Store` construction abstraction for a unified `Coyoneda`. Implemented
/// FOR the value type, each impl carries its store's per-`Store` bound in its
/// `where`-clause and builds the base-layer `Ptr` from a functor value, so a
/// single generic, path-syntax `Coyoneda::lift` can delegate here and serve every
/// store without naming any store-specific bound in its own signature.
///
/// This is the `ValueFor`-for-`Free::pure` pattern, generalised: there the bound
/// varies on the value `A`; here it varies on the functor value `F::Of<A>` (the Rc
/// arm needs `F::Of<A>: Clone`, the Arc arm `F::Of<A>: Clone + Send + Sync`), which
/// a fixed trait-method signature cannot carry, so each impl's `where`-clause does.
pub trait CoyoLift<'a, F, Store: CoyoStore>: 'a
where
	F: Kind_cdc7cd43dac7585f + 'a, {
	/// Build the base-layer `Ptr` (identity mapping) wrapping the functor value.
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
		let cell: Box<dyn CoyonedaInner<'a, F, A> + 'a> = Box::new(CoyonedaBase {
			fa,
		});
		cell
	}
}

impl<'a, F, A: 'a> CoyoLift<'a, F, RcBrand> for A
where
	F: Kind_cdc7cd43dac7585f + 'a,
	Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
{
	fn lift_ptr(
		fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> <RcBrand as CoyoStore>::Ptr<'a, F, A> {
		let cell: Rc<dyn RcCoyonedaLowerRef<'a, F, A> + 'a> = Rc::new(RcCoyonedaBase {
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
		let cell: Arc<dyn ArcCoyonedaLowerRef<'a, F, A> + 'a> = Arc::new(ArcCoyonedaBase {
			fa,
		});
		cell
	}
}

#[cfg(test)]
mod tests {
	use {
		super::{
			CoyoLift,
			CoyoStore,
		},
		crate::brands::{
			ArcBrand,
			BoxBrand,
			OptionBrand,
			RcBrand,
		},
	};

	fn assert_send_sync<T: Send + Sync>() {}

	// The Box `CoyoLift` builds a `Box<dyn CoyonedaInner>` that lowers by consuming
	// the box, end-to-end through the real base cell and the real functor value.
	#[test]
	fn box_coyo_lift_builds_a_lowerable_cell() {
		let ptr = <i32 as CoyoLift<'static, OptionBrand, BoxBrand>>::lift_ptr(Some(7));
		assert_eq!(ptr.lower(), Some(7));
	}

	// The Rc `CoyoLift` builds an `Rc<dyn RcCoyonedaLowerRef>` that lowers by
	// borrowing (cloning the stored functor value).
	#[test]
	fn rc_coyo_lift_builds_a_lowerable_cell() {
		let ptr = <i32 as CoyoLift<'static, OptionBrand, RcBrand>>::lift_ptr(Some(7));
		assert_eq!(ptr.lower_ref(), Some(7));
	}

	// The Arc `CoyoLift` builds an `Arc<dyn ArcCoyonedaLowerRef + Send + Sync>` that
	// is statically `Send + Sync` (the Arc impl required `Option<i32>: Clone + Send +
	// Sync`, discharged through `i32: CoyoLift<_, _, ArcBrand>`) and lowers by
	// borrowing.
	#[test]
	fn arc_coyo_lift_builds_a_send_sync_cell() {
		assert_send_sync::<<ArcBrand as CoyoStore>::Ptr<'static, OptionBrand, i32>>();
		let ptr = <i32 as CoyoLift<'static, OptionBrand, ArcBrand>>::lift_ptr(Some(7));
		assert_eq!(ptr.lower_ref(), Some(7));
	}
}
