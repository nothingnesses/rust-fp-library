//! Handler-list runtime values for the
//! [`handlers!`](https://docs.rs/fp-macros/latest/fp_macros/macro.handlers.html)
//! macro and the `nt()` builder fallback.
//!
//! Per [decisions.md](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/decisions.md)
//! section 4.6, a natural transformation `VariantF<R> ~> M` is assembled
//! at the user level either via the macro
//! `handlers!{ EBrand1: |op| ..., EBrand2: |op| ... }` (the primary
//! surface) or via the chained-builder fallback
//! `nt().on::<EBrand1, _>(|op| ...).on::<EBrand2, _>(|op| ...)`. Both
//! expressions evaluate to the same runtime shape: a type-level
//! cons-list whose structure mirrors the row's
//! [`CoproductBrand`](crate::brands::CoproductBrand) /
//! [`CNilBrand`](crate::brands::CNilBrand) chain cell-for-cell.
//!
//! Phase 3 step 1 (this module) only ships the runtime carrier; the
//! Phase 3 step 2 interpreter family
//! (`interpret` / `run` / `runAccum` and their `MonadRec` siblings) is
//! the consumer that recurses through the row and the handler list in
//! lock-step, dispatching each [`Coproduct::Inl`](crate::types::effects::coproduct::Coproduct::Inl)
//! variant to the matching [`HandlersCons::head`] and recursing into
//! [`HandlersCons::tail`] on [`Coproduct::Inr`](crate::types::effects::coproduct::Coproduct::Inr).
//! The closure shape carried inside each [`Handler`] is left fully
//! generic at this step; step 2 will pin it via an interpreter-side
//! trait bound.
//!
//! ## Why a dedicated cons-list rather than reusing `frunk_core`'s `HList`
//!
//! `frunk_core::hlist::{HNil, HCons}` are already re-exported under
//! [`crate::types::effects::coproduct`] for the row-encoding indexing
//! machinery (`Here` / `There`, `CoprodInjector`, etc.). Reusing the
//! same types here would technically work but conflate two distinct
//! roles: the coproduct module's `HCons` / `HNil` mark type-level
//! positions for row-membership proofs, while this module's
//! [`HandlersCons`] / [`HandlersNil`] carry runtime handler closures
//! aligned with the row's value-level shape. Distinct types keep the
//! intent visible at call sites and let inherent methods (the `.on()`
//! builder method) live on the handler-list types directly without an
//! extension-trait dance.
//!
//! ## Builder ordering
//!
//! [`nt()`] returns [`HandlersNil`]; [`HandlersNil::on`] and
//! [`HandlersCons::on`] both **prepend** a new handler at the head.
//! Chained calls therefore produce a list whose head is the
//! most-recently-added handler:
//!
//! ```ignore
//! nt().on::<A, _>(ha).on::<B, _>(hb)
//! //  yields HandlersCons { head: Handler<B>, tail: HandlersCons { head: Handler<A>, tail: HandlersNil } }
//! ```
//!
//! Users assembling a list to match a row built by
//! [`effects!`](https://docs.rs/fp-macros/latest/fp_macros/macro.effects.html)
//! (which sorts brands lexically) should call `.on()` in
//! reverse-lexical order so the resulting list's head aligns with the
//! row's lexically-smallest brand. The
//! [`handlers!`](https://docs.rs/fp-macros/latest/fp_macros/macro.handlers.html)
//! macro shares the lexical sort with `effects!` and emits the cons
//! chain in canonical order automatically; users who want
//! macro-equivalent ordering should prefer the macro.

use core::marker::PhantomData;

/// Newtype tagging a handler closure with the brand `E` it handles.
///
/// `Handler<E, F>` pins the brand identity at the type level so the
/// Phase 3 step 2 interpreter can match each handler against the row's
/// head brand without the closure's type signature having to encode
/// the brand explicitly. The closure value `F` stays opaque at this
/// step; step 2 will introduce an interpreter trait that adds the
/// concrete `F: FnMut(...) -> ...` bound.
#[derive(Clone, Copy)]
pub struct Handler<E, F> {
	/// The handler closure for effect brand `E`.
	pub run: F,
	#[doc(hidden)]
	pub _brand: PhantomData<fn() -> E>,
}

impl<E, F> Handler<E, F> {
	/// Wraps a closure as a [`Handler`] for effect brand `E`. Zero-cost.
	#[inline]
	pub const fn new(run: F) -> Self {
		Handler {
			run,
			_brand: PhantomData,
		}
	}
}

/// Empty handler list, mirrors [`CNilBrand`](crate::brands::CNilBrand)
/// at the row-shape level.
///
/// Returned by [`nt()`] as the seed of a builder chain. The
/// [`handlers!`](https://docs.rs/fp-macros/latest/fp_macros/macro.handlers.html)
/// macro emits this as the terminator of its cons chain.
#[derive(Clone, Copy, Debug, Default)]
pub struct HandlersNil;

/// Cons cell of the handler list, mirrors [`CoproductBrand`](crate::brands::CoproductBrand)
/// at the row-shape level.
///
/// `HandlersCons<H, T>` carries a head handler `H` (typically a
/// [`Handler<EBrand, F>`](Handler)) and a tail `T` that is itself
/// either another `HandlersCons` or [`HandlersNil`]. The shape mirrors
/// the row brand `CoproductBrand<EBrand, Tail>` cell-for-cell so the
/// Phase 3 step 2 interpreter can recurse through both in lock-step.
#[derive(Clone, Copy, Debug, Default)]
pub struct HandlersCons<H, T> {
	/// The handler at this row position.
	pub head: H,
	/// The remaining handlers, aligned with the tail of the row.
	pub tail: T,
}

impl HandlersNil {
	/// Prepends a new handler for effect brand `E` at the head of the
	/// list, transitioning [`HandlersNil`] to a single-cell
	/// [`HandlersCons<Handler<E, F>, HandlersNil>`](HandlersCons).
	///
	/// `E` is the brand identity (usually turbofished;
	/// `nt().on::<StateBrand, _>(...)`); `F` is inferred from the
	/// closure literal.
	#[inline]
	pub fn on<E, F>(
		self,
		handler: F,
	) -> HandlersCons<Handler<E, F>, Self> {
		HandlersCons {
			head: Handler::new(handler),
			tail: self,
		}
	}
}

impl<H, T> HandlersCons<H, T> {
	/// Prepends a new handler for effect brand `E` at the head of the
	/// list. The previous list becomes the tail.
	///
	/// Builder semantics are prepend; chained calls produce a list
	/// whose head is the most-recently-added handler. See the
	/// module-level "Builder ordering" note for alignment with rows
	/// built by [`effects!`](https://docs.rs/fp-macros/latest/fp_macros/macro.effects.html).
	#[inline]
	pub fn on<E, F>(
		self,
		handler: F,
	) -> HandlersCons<Handler<E, F>, Self> {
		HandlersCons {
			head: Handler::new(handler),
			tail: self,
		}
	}
}

/// Entry point for the chained-builder fallback for assembling a
/// handler list per [decisions.md](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/decisions.md)
/// section 4.6.
///
/// Returns [`HandlersNil`]; chain `.on::<EBrand, _>(handler)` calls to
/// prepend handlers. The
/// [`handlers!`](https://docs.rs/fp-macros/latest/fp_macros/macro.handlers.html)
/// macro is the primary surface and produces equivalent shapes via the
/// macro DSL.
#[inline]
#[must_use]
pub const fn nt() -> HandlersNil {
	HandlersNil
}

#[cfg(test)]
mod tests {
	use super::*;

	struct StateBrand;
	struct ReaderBrand;
	struct ExceptBrand;

	#[test]
	fn nt_returns_empty_list() {
		let h = nt();
		let _: HandlersNil = h;
	}

	#[test]
	fn on_at_nil_produces_single_cell() {
		let h = nt().on::<StateBrand, _>(|x: i32| x + 1);
		let _: HandlersCons<Handler<StateBrand, _>, HandlersNil> = h;
		let result = (h.head.run)(7);
		assert_eq!(result, 8);
	}

	#[test]
	fn on_at_cons_prepends_new_head() {
		let h = nt().on::<StateBrand, _>(|x: i32| x).on::<ReaderBrand, _>(|x: i32| x * 2);
		let _: HandlersCons<
			Handler<ReaderBrand, _>,
			HandlersCons<Handler<StateBrand, _>, HandlersNil>,
		> = h;
		assert_eq!((h.head.run)(5), 10);
		assert_eq!((h.tail.head.run)(5), 5);
	}

	#[test]
	fn three_handler_chain() {
		let h = nt()
			.on::<StateBrand, _>(|x: i32| x)
			.on::<ReaderBrand, _>(|x: i32| x + 1)
			.on::<ExceptBrand, _>(|x: i32| x + 2);
		assert_eq!((h.head.run)(0), 2);
		assert_eq!((h.tail.head.run)(0), 1);
		assert_eq!((h.tail.tail.head.run)(0), 0);
	}

	#[test]
	fn handler_new_round_trips_closure() {
		let handler = Handler::<StateBrand, _>::new(|x: i32| x * 3);
		assert_eq!((handler.run)(4), 12);
	}

	#[test]
	fn handlers_cons_struct_literal_works() {
		// The macro emits struct-literal HandlersCons values; this
		// regression-checks the field-name shape stays compatible.
		type SingleStateCell = HandlersCons<Handler<StateBrand, fn(i32) -> i32>, HandlersNil>;
		let h: SingleStateCell = HandlersCons {
			head: Handler::new(|x: i32| x + 100),
			tail: HandlersNil,
		};
		assert_eq!((h.head.run)(1), 101);
	}
}
