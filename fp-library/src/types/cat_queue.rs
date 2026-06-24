//! FS-1 substrate: the catenable continuation-queue interface, crate-internal
//! work in progress.
//!
//! This module is part of the FS-1 substrate build (remediation-plan review-2,
//! item 4 step 5). It lives in core `crate::types` (not under the
//! `effects`-feature-gated subsystem) because the core `Free<F, A, Store>` carries
//! its queue as `<Store as ClosureStorage>::Queue<_>` and the multi-shot stepping
//! is written against this trait.
//!
//! [`CatQueue`] is the small interface the unified multi-shot stepping needs from
//! a continuation queue: an empty queue (the [`Default`] supertrait), `snoc`,
//! `append`, and `uncons`. It is implemented by the three catenable lists that
//! back the per-`Store` `Queue` associated type on
//! [`ClosureStorage`](crate::types::closure_storage::ClosureStorage): the by-value
//! [`CatList`] for the one-shot Box store (implemented at every element type), and
//! the refcounted [`RcCatList`] and [`ArcCatList`] for the multi-shot Rc/Arc
//! stores (implemented only at `C: Clone`, since their `link` clones the shared
//! sublist deque through `Rc`/`Arc::make_mut`).
//!
//! The trait deliberately has NO `Clone` supertrait: the Box queue holds
//! non-`Clone` `FnOnce` continuations, so a blanket `Clone` bound would exclude
//! it. The multi-shot stepping that clones the queue per branch adds the `Clone`
//! bound at its own use site, where the element is always a `Clone` `Rc`/`Arc`
//! continuation. Likewise the queue is bounded only by `Default` on
//! `ClosureStorage`, which is all the construction-free accessors and the iterative
//! `Drop` require; `uncons` and the rest are required of the queue only where the
//! multi-shot stepping names this trait.
//!
//! Documentation status: like the other in-progress FS-1 substrate modules, this
//! module intentionally does NOT yet use the `#[fp_macros::document_module]`
//! wrapper. The wrapper and full per-item documentation are added once the
//! substrate is settled (remediation item 11), which clears this tracked,
//! temporary exception.

#![allow(
	dead_code,
	reason = "FS-1 rebuild in progress (item 4 step 5): this queue interface is consumed by the Store-parameterised multi-shot substrate built in later sub-steps; item 20 sweeps any residual allowances at the end of the rebuild."
)]

use crate::types::{
	arc_cat_list::ArcCatList,
	cat_list::CatList,
	rc_cat_list::RcCatList,
};

/// One catenable continuation-queue interface over the per-`Store` queues. The
/// empty queue is the [`Default`] supertrait; `snoc` appends one element, `append`
/// concatenates two queues, and `uncons` removes the head. No `Clone` supertrait:
/// the one-shot Box queue holds non-`Clone` `FnOnce` elements, so cloning is bound
/// at the multi-shot use site instead.
pub trait CatQueue<C>: Default + Sized {
	/// Append one element to the back of the queue.
	fn snoc(
		self,
		element: C,
	) -> Self;

	/// Concatenate two queues.
	fn append(
		self,
		other: Self,
	) -> Self;

	/// Remove and return the head element and the rest of the queue, or `None`
	/// when empty.
	fn uncons(self) -> Option<(C, Self)>;
}

impl<C> CatQueue<C> for CatList<C> {
	fn snoc(
		self,
		element: C,
	) -> Self {
		CatList::snoc(self, element)
	}

	fn append(
		self,
		other: Self,
	) -> Self {
		CatList::append(self, other)
	}

	fn uncons(self) -> Option<(C, Self)> {
		CatList::uncons(self)
	}
}

impl<C: Clone> CatQueue<C> for RcCatList<C> {
	fn snoc(
		self,
		element: C,
	) -> Self {
		RcCatList::snoc(self, element)
	}

	fn append(
		self,
		other: Self,
	) -> Self {
		RcCatList::append(self, other)
	}

	fn uncons(self) -> Option<(C, Self)> {
		RcCatList::uncons(self)
	}
}

impl<C: Clone> CatQueue<C> for ArcCatList<C> {
	fn snoc(
		self,
		element: C,
	) -> Self {
		ArcCatList::snoc(self, element)
	}

	fn append(
		self,
		other: Self,
	) -> Self {
		ArcCatList::append(self, other)
	}

	fn uncons(self) -> Option<(C, Self)> {
		ArcCatList::uncons(self)
	}
}
