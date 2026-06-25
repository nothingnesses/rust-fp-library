//! Per-`Store` recursion-indirection pointer for the concrete `FreeExplicit`
//! free monad, crate-internal and work in progress.
//!
//! `ExplicitStore` is the sibling of the closure-storage and `Coyoneda`
//! pointer-storage interfaces, for the concrete (non-`'static`, O(N)-bind) free
//! monad's self pointer. Where the closure-storage interface abstracts a stored
//! closure and the `Coyoneda` interface abstracts the outer pointer to an
//! existential cell, `ExplicitStore` abstracts the sized self-pointer that the
//! `Wrap` variant stores so the recursive type has heap indirection: `Box<T>`
//! for the single-shot by-value form, `Rc<T>`/`Arc<T>` for the multi-shot,
//! cheaply-clonable arms. `try_into_inner` recovers ownership of the pointee
//! (always for `Box`; for the refcounted pointers only when the pointer is
//! unique), which is what lets one iterative `Drop` be generic over the store.
//!
//! It is `pub` (mirroring the closure-storage interface) because it bounds the
//! public `FreeExplicit`, so its associated pointer type must not leak a
//! crate-private trait.
//!
//! Documentation status: like the other in-progress substrate modules, this
//! module intentionally does not yet use the `#[fp_macros::document_module]`
//! wrapper; the wrapper and full per-item documentation are added once the
//! substrate is settled.

#![allow(
	dead_code,
	reason = "FS-1 rebuild work in progress: this pointer-storage interface is consumed by the Store-parameterised concrete FreeExplicit built in later sub-steps; residual allowances are swept at the end of the rebuild."
)]

use {
	crate::brands::{
		ArcBrand,
		BoxBrand,
		RcBrand,
	},
	std::{
		rc::Rc,
		sync::Arc,
	},
};

/// The per-`Store` recursion-indirection pointer interface for `FreeExplicit`.
///
/// `: 'static` so that `SelfPtr<'a, T>` (which holds the store) satisfies its
/// own `: 'a` bound for every `'a` without each user restating `Store: 'a`.
pub trait ExplicitStore: 'static {
	/// The sized self-pointer the `Wrap` variant stores: a `Box`/`Rc`/`Arc` of
	/// the next step in the computation.
	type SelfPtr<'a, T: 'a>: 'a;

	/// Recover ownership of the pointee: always for `Box`; for `Rc`/`Arc` only
	/// when the pointer is unique (otherwise `None`, leaving other holders to
	/// dismantle their own share).
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
