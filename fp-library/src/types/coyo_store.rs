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
//! the refcounted stores), and bridges lowering by value: the Box arm consumes its
//! box, the refcounted arms borrow through the owned pointer (cloning the stored
//! value). The map closure inside a cell is an ordinary `Fn` in every store; only
//! the outer pointer and the lowering discipline vary, which is the per-`Store`
//! axis this trait names.
//!
//! Only the Box impl is present so far (reusing the existing core
//! `CoyonedaInner`); the refcounted impls and the threading of `Store` onto
//! `Coyoneda` itself land in later sub-steps.
//!
//! Documentation status: like the other in-progress FS-1 substrate modules, this
//! module intentionally does NOT yet use the `#[fp_macros::document_module]`
//! wrapper. The wrapper and full per-item documentation are added once the
//! substrate is settled (remediation item 11), which clears this tracked,
//! temporary exception.

#![allow(
	dead_code,
	reason = "FS-1 rebuild in progress (item 4 step 5.4): this pointer-storage interface is consumed by the Store-parameterised unified Coyoneda built in later sub-steps; item 20 sweeps any residual allowances at the end of the rebuild."
)]

use crate::{
	Apply,
	brands::BoxBrand,
	classes::Functor,
	kinds::*,
	types::coyoneda::CoyonedaInner,
};

/// One pointer-storage interface over the per-`Store` outer pointer that holds a
/// `Coyoneda`'s inner existential cell. The associated `Ptr` GAT is the store's
/// outer pointer to the cell, and `lower` bridges lowering by value (consuming for
/// the Box store, borrowing through the owned pointer for the refcounted stores).
///
/// `pub(crate)` for now: it is promoted to `pub` (mirroring `ClosureStorage` per
/// the OQ-6D resolution) only when it bounds the public `Coyoneda`, which is a
/// later sub-step; at that point `CoyonedaInner` is promoted alongside it so the
/// `Ptr` associated type does not leak a crate-private trait.
pub(crate) trait CoyoStore: 'static {
	/// The per-`Store` outer pointer to the inner cell (a `Box`/`Rc`/`Arc` of the
	/// store's `dyn ...Inner` existential).
	type Ptr<'a, F, A>: 'a
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		A: 'a;

	/// Lower the stored cell to the underlying functor value, consuming the
	/// pointer by value. Requires `F: Functor`, applied at lowering time.
	fn lower<'a, F, A>(
		ptr: Self::Ptr<'a, F, A>
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		F: Functor + 'a,
		A: 'a;
}

impl CoyoStore for BoxBrand {
	type Ptr<'a, F, A>
		= Box<dyn CoyonedaInner<'a, F, A> + 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		A: 'a;

	fn lower<'a, F, A>(
		ptr: Self::Ptr<'a, F, A>
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		F: Functor + 'a,
		A: 'a, {
		ptr.lower()
	}
}

#[cfg(test)]
mod tests {
	use {
		super::CoyoStore,
		crate::{
			Apply,
			brands::{
				BoxBrand,
				OptionBrand,
			},
			kinds::*,
			types::coyoneda::CoyonedaInner,
		},
	};

	// A minimal inner cell standing in for a real `Coyoneda` base layer: it holds
	// an `Option<i32>` and lowers to it directly. The `Box` store's `lower`
	// consumes this through the `CoyonedaInner` trait object.
	struct TestCell(Option<i32>);

	impl<'a> CoyonedaInner<'a, OptionBrand, i32> for TestCell {
		fn lower(
			self: Box<Self>
		) -> Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, i32>) {
			self.0
		}
	}

	// The Box `CoyoStore` lowers a `Box<dyn CoyonedaInner>` to the underlying
	// `Option`, end-to-end through the by-value bridge and the real functor value.
	#[test]
	fn box_coyo_store_lowers_through_the_inner_cell() {
		let ptr: Box<dyn CoyonedaInner<'static, OptionBrand, i32>> = Box::new(TestCell(Some(7)));
		assert_eq!(<BoxBrand as CoyoStore>::lower::<OptionBrand, i32>(ptr), Some(7));
	}
}
