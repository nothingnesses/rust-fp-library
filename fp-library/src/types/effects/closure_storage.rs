//! FS-1 substrate: the unified closure storage, crate-internal work in progress.
//!
//! This module is part of the FS-1 substrate build (remediation-plan review-2,
//! item 4 step 5). It is `pub(crate)` and not part of the public surface: per the
//! adopted hybrid method, the new substrate is built to a compiling, test-backed
//! state before the dual-row subsystem it replaces is deleted.
//!
//! `ClosureStorage` unifies the per-pointer closure storage that the dual-row
//! system spreads across separate `ToDynFnOnce` / `ToDynCloneFn` / `ToDynSendFn`
//! conversions and the `FnBrand` family. One associated `Stored` type carries the
//! callable *kind* per store, a one-shot `Box<dyn FnOnce>` for [`BoxBrand`], a
//! reusable `Rc<dyn Fn>` for [`RcBrand`], and a thread-safe
//! `Arc<dyn Fn + Send + Sync>` for [`ArcBrand`], and a single by-value
//! [`call_once`](ClosureStorage::call_once) bridges all three: it consumes the
//! `FnOnce` for Box (running it once) and borrows the `Fn` through the owned
//! pointer for Rc/Arc (which the multi-shot path clones first). This is the
//! substrate's continuation-storage axis; the row-cell pointer axis (a sibling
//! storage for `Coyoneda`) is built alongside it.
//!
//! Documentation status: like the FS-1 vertical-slice module, this module
//! intentionally does NOT yet use the `#[fp_macros::document_module]` wrapper that
//! the rest of `fp-library/src/` uses. The substrate is still being shaped and the
//! prerequisites for writing the runnable per-method doctests `document_module`
//! requires are not yet in place; documenting it now would be throwaway. The
//! wrapper and full per-item documentation are added once the substrate is settled
//! and the FS-1 `define_effect!` macro lands (remediation item 11), which clears
//! this tracked, temporary exception.

#![allow(
	dead_code,
	reason = "FS-1 rebuild in progress (item 4 step 5): the storage trait and its impls are consumed by the Store-parameterised substrate built in later sub-steps, and are currently exercised only by this module's tests; item 20 sweeps any residual allowances at the end of the rebuild."
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

/// One closure-storage class over the pointer stores. `Stored<'a, I, O>` is the
/// stored callable for a store, carrying its callable kind (`FnOnce` for Box,
/// `Fn` for Rc/Arc), and [`call_once`](ClosureStorage::call_once) invokes it by
/// value so one signature serves both kinds.
pub(crate) trait ClosureStorage: 'static {
	/// The stored callable from `I` to `O` for this store.
	type Stored<'a, I: 'a, O: 'a>: 'a;

	/// Invoke the stored callable once, consuming it. For Box this consumes the
	/// owned `FnOnce`; for Rc/Arc it borrows the `Fn` through the owned pointer,
	/// which then drops (the multi-shot path clones the pointer beforehand).
	fn call_once<'a, I: 'a, O: 'a>(
		stored: Self::Stored<'a, I, O>,
		input: I,
	) -> O;
}

impl ClosureStorage for BoxBrand {
	type Stored<'a, I: 'a, O: 'a> = Box<dyn FnOnce(I) -> O + 'a>;

	fn call_once<'a, I: 'a, O: 'a>(
		stored: Self::Stored<'a, I, O>,
		input: I,
	) -> O {
		stored(input)
	}
}

impl ClosureStorage for RcBrand {
	type Stored<'a, I: 'a, O: 'a> = Rc<dyn Fn(I) -> O + 'a>;

	fn call_once<'a, I: 'a, O: 'a>(
		stored: Self::Stored<'a, I, O>,
		input: I,
	) -> O {
		(*stored)(input)
	}
}

impl ClosureStorage for ArcBrand {
	type Stored<'a, I: 'a, O: 'a> = Arc<dyn Fn(I) -> O + Send + Sync + 'a>;

	fn call_once<'a, I: 'a, O: 'a>(
		stored: Self::Stored<'a, I, O>,
		input: I,
	) -> O {
		(*stored)(input)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn assert_send_sync<T: Send + Sync>() {}

	// The Box store holds a genuine `FnOnce`: the closure moves a non-`Copy`
	// capture out when it runs, which an `Fn`-only store could not hold.
	#[test]
	fn box_store_holds_a_genuine_fnonce() {
		let captured = String::from("hi");
		let stored: <BoxBrand as ClosureStorage>::Stored<'static, (), usize> =
			Box::new(move |()| captured.into_bytes().len());
		assert_eq!(<BoxBrand as ClosureStorage>::call_once(stored, ()), 2);
	}

	// The Rc store is multi-shot: clone the stored `Fn`, then `call_once` each
	// clone independently.
	#[test]
	fn rc_store_is_multi_shot_via_clone() {
		let stored: <RcBrand as ClosureStorage>::Stored<'static, i32, i32> = Rc::new(|x| x + 1);
		let first = <RcBrand as ClosureStorage>::call_once(Rc::clone(&stored), 10);
		let second = <RcBrand as ClosureStorage>::call_once(stored, 20);
		assert_eq!((first, second), (11, 21));
	}

	// The Arc store is statically `Send + Sync` and invokes correctly.
	#[test]
	fn arc_store_is_send_sync_and_invokes() {
		assert_send_sync::<<ArcBrand as ClosureStorage>::Stored<'static, i32, i32>>();
		let stored: <ArcBrand as ClosureStorage>::Stored<'static, i32, i32> = Arc::new(|x| x * 2);
		assert_eq!(<ArcBrand as ClosureStorage>::call_once(stored, 21), 42);
	}
}
