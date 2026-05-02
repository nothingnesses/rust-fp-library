//! Interpreter family for Run programs (Phase 3 step 2).
//!
//! Provides the [`DispatchHandlers`] trait that walks a row's
//! value-level [`Coproduct`] variants against a [`HandlersCons`] /
//! [`HandlersNil`] handler list in lock-step, dispatching the active
//! variant to its matching [`Handler`] closure. Each Run wrapper
//! exposes inherent `interpret` / `run` / `run_accum` methods that
//! loop over `peel` and invoke `DispatchHandlers` once per
//! `Node::First` layer.
//!
//! ## Mono-in-`A` dispatch model
//!
//! PureScript Run's
//! [`interpret`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
//! has the signature
//! `(VariantF r ~> m) -> Run r a -> m a` (a true rank-2 natural
//! transformation), but its implementation literally aliases `run`
//! whose signature is `(VariantF r (Run r a) -> m (Run r a)) -> Run r a
//! -> m a` -- a step function whose handler is mono-in-`a`. The Rust
//! port adopts the mono-in-`a` form directly so handler closures fit
//! Rust's non-generic-closure constraint. Each [`Handler`] cell
//! carries a closure of shape
//! `FnOnce(<EBrand as Kind>::Of<'_, NextProgram>) -> NextProgram`
//! where `NextProgram` is the Run wrapper specialized to the
//! program's result type `A`.
//!
//! Users who genuinely need rank-2 polymorphism over `A` (e.g., a
//! transformation that doesn't depend on the program's result type at
//! all) reach for [`crate::classes::NaturalTransformation`] directly,
//! consumed by [`Free::fold_free`](crate::types::Free::fold_free) or
//! similar; that path bypasses the per-effect handler-list pattern.
//!
//! ## Shape: handler list mirrors row brand chain
//!
//! Per [Phase 3 step 1](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/plan.md),
//! the handler list cons cells are positional:
//! `HandlersCons<Handler<EBrand, F>, T>` aligns with the row brand
//! chain `CoproductBrand<CoyonedaBrand<EBrand>, RestBrand>`. The
//! [`DispatchHandlers`] trait recurses through both in lock-step:
//! `Coproduct::Inl` dispatches to `HandlersCons::head`,
//! `Coproduct::Inr` recurses on `HandlersCons::tail`. `CNil` matches
//! [`HandlersNil`] and is uninhabited, so the recursion terminates
//! safely.

use crate::{
	classes::{
		Functor,
		SendFunctor,
	},
	kinds::Kind_cdc7cd43dac7585f,
	types::{
		ArcCoyoneda,
		Coyoneda,
		RcCoyoneda,
		effects::{
			coproduct::{
				CNil,
				Coproduct,
			},
			handlers::{
				Handler,
				HandlersCons,
				HandlersNil,
			},
		},
	},
};

/// Walks a handler list against a row's value-level `Coproduct` chain
/// in lock-step, dispatching to the matching handler.
///
/// Implemented recursively:
///
/// - [`HandlersNil`] paired with [`CNil`] is the base case; the body
///   matches the uninhabited `CNil` exhaustively.
/// - [`HandlersCons<Handler<E, F>, T>`] paired with
///   `Coproduct<Coyoneda<'a, E, NextProgram>, Rest>` dispatches `Inl`
///   to the head handler (after lowering the `Coyoneda` via `E`'s
///   `Functor`) and recurses `Inr` into the tail.
///
/// `Layer` is the row's value-level shape at the active `'a` /
/// `NextProgram` instantiation;
/// `NextProgram` is the Run wrapper specialized to the program's
/// result type. The trait method takes ownership of the layer
/// (single-shot semantics; multi-shot wrappers' callers can
/// [`Clone`] the layer before invoking dispatch).
///
/// `dispatch` takes `&self` (not `&mut self`) so it can be called
/// from inside a [`Fn`] closure (e.g., the step closure passed to
/// [`MonadRec::tail_rec_m`](crate::classes::MonadRec) by
/// `interpret_rec`). Handler closures stored in [`Handler<E, F>`]
/// are bound `F: Fn`; mutation flows through interior mutability
/// at the user level (`Rc<RefCell<_>>` or `Arc<Mutex<_>>` captures),
/// matching the `Fn`-callable contract.
pub trait DispatchHandlers<'a, Layer, NextProgram>
where
	Layer: 'a,
	NextProgram: 'a, {
	/// Dispatches the row's active variant to the matching handler
	/// closure, producing the next program.
	fn dispatch(
		&self,
		layer: Layer,
	) -> NextProgram;
}

impl<'a, NextProgram> DispatchHandlers<'a, CNil, NextProgram> for HandlersNil
where
	NextProgram: 'a,
{
	#[inline]
	fn dispatch(
		&self,
		layer: CNil,
	) -> NextProgram {
		match layer {}
	}
}

impl<'a, EBrand, F, T, Rest, NextProgram>
	DispatchHandlers<'a, Coproduct<Coyoneda<'a, EBrand, NextProgram>, Rest>, NextProgram>
	for HandlersCons<Handler<EBrand, F>, T>
where
	EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
	F: Fn(<EBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>) -> NextProgram,
	T: DispatchHandlers<'a, Rest, NextProgram>,
	NextProgram: 'a,
	Rest: 'a,
{
	#[inline]
	fn dispatch(
		&self,
		layer: Coproduct<Coyoneda<'a, EBrand, NextProgram>, Rest>,
	) -> NextProgram {
		match layer {
			Coproduct::Inl(coyo) => (self.head.run)(coyo.lower()),
			Coproduct::Inr(rest) => self.tail.dispatch(rest),
		}
	}
}

impl<'a, EBrand, F, T, Rest, NextProgram>
	DispatchHandlers<'a, Coproduct<RcCoyoneda<'a, EBrand, NextProgram>, Rest>, NextProgram>
	for HandlersCons<Handler<EBrand, F>, T>
where
	EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
	F: Fn(<EBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>) -> NextProgram,
	T: DispatchHandlers<'a, Rest, NextProgram>,
	NextProgram: 'a,
	Rest: 'a,
	<EBrand as Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>: 'a,
{
	#[inline]
	fn dispatch(
		&self,
		layer: Coproduct<RcCoyoneda<'a, EBrand, NextProgram>, Rest>,
	) -> NextProgram {
		match layer {
			Coproduct::Inl(coyo) => (self.head.run)(coyo.lower_ref()),
			Coproduct::Inr(rest) => self.tail.dispatch(rest),
		}
	}
}

impl<'a, EBrand, F, T, Rest, NextProgram>
	DispatchHandlers<'a, Coproduct<ArcCoyoneda<'a, EBrand, NextProgram>, Rest>, NextProgram>
	for HandlersCons<Handler<EBrand, F>, T>
where
	EBrand: Kind_cdc7cd43dac7585f + Functor + SendFunctor + 'static,
	F: Fn(<EBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>) -> NextProgram,
	T: DispatchHandlers<'a, Rest, NextProgram>,
	NextProgram: Send + Sync + 'a,
	Rest: 'a,
	<EBrand as Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>: Send + Sync + 'a,
{
	#[inline]
	fn dispatch(
		&self,
		layer: Coproduct<ArcCoyoneda<'a, EBrand, NextProgram>, Rest>,
	) -> NextProgram {
		match layer {
			Coproduct::Inl(coyo) => (self.head.run)(coyo.lower_ref()),
			Coproduct::Inr(rest) => self.tail.dispatch(rest),
		}
	}
}
