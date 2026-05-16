//! Interpreter family for Run programs.
//!
//! Provides the [`DispatchHandlers`] trait that walks a row's
//! value-level [`Coproduct`](crate::types::effects::coproduct::Coproduct)
//! variants against a
//! [`HandlersCons`](crate::types::effects::handlers::HandlersCons) /
//! [`HandlersNil`](crate::types::effects::handlers::HandlersNil) handler
//! list in lock-step, dispatching the active variant to its matching
//! [`Handler`](crate::types::effects::handlers::Handler) closure. Each Run wrapper
//! exposes inherent `handle` / `run` methods that
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
//! Rust's non-generic-closure constraint. Each [`Handler`](crate::types::effects::handlers::Handler) cell
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
//! The handler list cons cells are positional:
//! `HandlersCons<Handler<EBrand, F>, T>` aligns with the row brand
//! chain `CoproductBrand<CoyonedaBrand<EBrand>, RestBrand>`. The
//! [`DispatchHandlers`] trait recurses through both in lock-step:
//! `Coproduct::Inl` dispatches to `HandlersCons::head`,
//! `Coproduct::Inr` recurses on `HandlersCons::tail`. `CNil` matches
//! [`HandlersNil`](crate::types::effects::handlers::HandlersNil) and is uninhabited, so the recursion terminates
//! safely.
//!
//! ## Async / IO workaround: `spawn_blocking`
//!
//! The interpreter family is synchronous: handler closures take a
//! row layer and return the next program directly, not a `Future`.
//! No `async fn` interpreter variant ships, because
//! [`MonadRec`](crate::classes::MonadRec) (the trait whose
//! `tail_rec_m` drives `handle_rec`'s stack-safe loop) has no
//! impl for `Future`-shaped target monads in this library; without
//! that, an async interpreter cannot satisfy the same stack-safety
//! contract the sync family does.
//!
//! For programs that need to interleave async work with effect
//! interpretation today, the supported workaround is
//! [`tokio::task::spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
//! (or the equivalent on other runtimes): wrap the synchronous
//! `handle` call inside a blocking task, await the join handle
//! from async code. Handler closures may themselves block on
//! [`tokio::runtime::Handle::block_on`](https://docs.rs/tokio/latest/tokio/runtime/struct.Handle.html#method.block_on)
//! to call out to async APIs from inside the interpreter, at the
//! cost of one blocking-thread-pool slot per concurrent program.
//!
//! This is a runtime-level workaround, not a library feature. A
//! native `Future`-based interpreter would compose better but
//! requires a `MonadRec` impl over `Future` first.

mod first_order;
mod scoped_resume;

#[fp_macros::document_module]
mod inner {
	pub use super::first_order::DispatchHandlers;
	pub(crate) use super::scoped_resume::*;
	use crate::{
		kinds::Kind_cdc7cd43dac7585f,
		types::effects::{
			coproduct::{
				CNil,
				Coproduct,
				Here,
				There,
			},
			handlers::{
				ScopedHandler,
				ScopedHandlersCons,
				ScopedHandlersNil,
			},
		},
	};

	/// Public facade for scoped-handler lists that consume typed boundaries.
	///
	/// Indexed around-action constructors such as Explicit scoped effects return
	/// boundary values rather than ordinary programs: the selected action program
	/// and the outer continuation must stay typed separately until a scoped
	/// handler consumes the boundary. This facade is the public handler-list
	/// entrypoint for that operation. It names only stable public concepts:
	/// the boundary value, the first-order handler layer used while the selected
	/// action runs, and the next program produced after the boundary resumes.
	///
	/// Implementations may delegate to private continuation-carrier machinery,
	/// but public `handle` / `run` methods should depend on this trait rather
	/// than on the private carrier traits directly.
	#[fp_macros::document_type_parameters(
		"The lifetime of the boundary, first-order layer, and produced next program.",
		"The typed scoped boundary value produced by an around-action constructor.",
		"The first-order row's value-level layer shape used by nested interpretation.",
		"The next program produced after the boundary resumes."
	)]
	#[fp_macros::document_parameters("The scoped-handler-list instance.")]
	pub trait DispatchScopedBoundaryHandlers<'a, Boundary, FirstLayer, NextProgram>
	where
		Boundary: 'a,
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Dispatch a typed scoped boundary through this scoped-handler list.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The typed scoped boundary carrying the selected action and outer continuation.",
			"The first-order handler list used while resuming the selected action."
		)]
		///
		#[fp_macros::document_returns(
			"The next program produced after the matching scoped handler consumes the boundary."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::{
		/// 	coproduct::CNil,
		/// 	handlers::HandlersNil,
		/// 	interpreter::{
		/// 		DispatchHandlers,
		/// 		DispatchScopedBoundaryHandlers,
		/// 	},
		/// };
		///
		/// struct Boundary {
		/// 	action_value: i32,
		/// 	outer: fn(i32) -> i32,
		/// }
		///
		/// struct AddBeforeOuter;
		///
		/// impl<'a> DispatchScopedBoundaryHandlers<'a, Boundary, CNil, i32> for AddBeforeOuter {
		/// 	fn dispatch_scoped_boundary(
		/// 		&self,
		/// 		boundary: Boundary,
		/// 		_fo_handlers: &impl DispatchHandlers<'a, CNil, i32>,
		/// 	) -> i32 {
		/// 		(boundary.outer)(boundary.action_value + 1)
		/// 	}
		/// }
		///
		/// let boundary = Boundary {
		/// 	action_value: 40,
		/// 	outer: |value| value + 1,
		/// };
		///
		/// let result = AddBeforeOuter.dispatch_scoped_boundary(boundary, &HandlersNil);
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_boundary(
			&self,
			boundary: Boundary,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;
	}

	/// Dispatch contract for a single scoped-handler cell.
	///
	/// Unlike first-order [`Handler`] values, scoped handlers cannot be
	/// plain `Fn` closures in the general case: they receive the
	/// first-order handler list, and that list's concrete type remains
	/// generic at the method level. Standard scoped handlers and
	/// user-defined scoped handler values implement this trait, then
	/// [`DispatchScopedHandlers`] lifts them into a recursive handler
	/// list.
	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, and produced next program.",
		"The active scoped-effect layer handled by this cell.",
		"The first-order row's value-level layer shape.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The scoped-handler handler value.")]
	pub trait DispatchScopedHandler<'a, ScopedLayer, FirstLayer, NextProgram>
	where
		ScopedLayer: 'a,
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Dispatches one scoped-effect layer, with access to the
		/// inherited first-order handler list.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The scoped-effect layer carrying the active scoped operation.",
			"The first-order handler list used by nested interpretation."
		)]
		///
		#[fp_macros::document_returns("The next program produced by the scoped handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::{
		/// 	DispatchHandlers,
		/// 	DispatchScopedHandler,
		/// 	HandlersNil,
		/// 	coproduct::CNil,
		/// };
		///
		/// struct AddOne;
		///
		/// impl<'a> DispatchScopedHandler<'a, i32, CNil, i32> for AddOne {
		/// 	fn dispatch_scoped_head(
		/// 		&self,
		/// 		layer: i32,
		/// 		_fo_handlers: &impl DispatchHandlers<'a, CNil, i32>,
		/// 	) -> i32 {
		/// 		layer + 1
		/// 	}
		/// }
		///
		/// let result = AddOne.dispatch_scoped_head(41, &HandlersNil);
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: ScopedLayer,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;
	}

	/// Walks a scoped-handler list against a scoped row's value-level
	/// `Coproduct` chain in lock-step, dispatching to the matching scoped
	/// handler.
	///
	/// `DispatchScopedHandlers` is the scoped-row parallel to
	/// [`DispatchHandlers`]. Each cons-cell method is generic over the
	/// concrete first-order handler-list type so scoped handlers can
	/// recursively interpret nested first-order operations without
	/// erasing the first-order handler list behind dynamic dispatch.
	///
	/// Missing scoped handlers surface as trait errors against this
	/// list-walking contract or the wrapper-specific raw scoped-dispatch
	/// companion used by default `Run`. If an `handle` call reports
	/// that a scoped-dispatch trait is not implemented for
	/// [`ScopedHandlersNil`] or another scoped-handler-list tail, inspect
	/// the remaining scoped-row `Coproduct` head in the error and add a
	/// matching `ScopedBrand: handler_value` entry to
	/// `scoped_handlers!`.
	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, and produced next program.",
		"The scoped row's value-level shape.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The scoped-handler-list instance.")]
	pub trait DispatchScopedHandlers<'a, ScopedLayer, FirstLayer, NextProgram>
	where
		ScopedLayer: 'a,
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Dispatches the scoped row's active variant to the matching
		/// scoped handler.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The scoped row layer carrying the active scoped effect variant.",
			"The first-order handler list used by nested interpretation."
		)]
		///
		#[fp_macros::document_returns("The next program produced by the matching scoped handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::IdentityBrand,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			DispatchHandlers,
		/// 			DispatchScopedHandler,
		/// 			DispatchScopedHandlers,
		/// 			HandlersNil,
		/// 			coproduct::{
		/// 				CNil,
		/// 				Coproduct,
		/// 			},
		/// 			scoped_nt,
		/// 		},
		/// 	},
		/// };
		///
		/// struct IdentityScoped;
		///
		/// impl<'a> DispatchScopedHandler<'a, Identity<i32>, CNil, i32> for IdentityScoped {
		/// 	fn dispatch_scoped_head(
		/// 		&self,
		/// 		layer: Identity<i32>,
		/// 		_fo_handlers: &impl DispatchHandlers<'a, CNil, i32>,
		/// 	) -> i32 {
		/// 		layer.0
		/// 	}
		/// }
		///
		/// let scoped_handlers =
		/// 	fp_library::types::effects::scoped_nt().on::<IdentityBrand, _>(IdentityScoped);
		/// let layer = Coproduct::Inl(Identity(42));
		/// let result = scoped_handlers.dispatch_scoped(layer, &HandlersNil);
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped(
			&self,
			layer: ScopedLayer,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;
	}

	/// Carrier-aware dispatch contract for a single around-action scoped
	/// handler.
	///
	/// This private route is the H2 companion to
	/// [`DispatchScopedHandler`]. Ordinary scoped handlers receive a scoped
	/// layer whose action has already been mapped to `NextProgram`.
	/// Around-action handlers instead receive the scoped layer mapped to the
	/// carrier's `ActionProgram`, plus the wrapper-owned
	/// [`ScopedContinuation`] that can resume the outer continuation after
	/// inserting result-preserving post-action work. The scoped-effect row brand
	/// remains static; the selected action program/value travels through the
	/// carrier's lifetime-indexed associated types, and `NextProgram` stays the
	/// final mapped result slot.
	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, produced next program, and carrier.",
		"The active scoped-effect layer handled by this cell.",
		"The first-order row's value-level layer shape.",
		"The Run wrapper specialized to the program's result type.",
		"The wrapper-owned continuation carrier type."
	)]
	#[fp_macros::document_parameters("The scoped-handler handler value.")]
	#[allow(
		dead_code,
		reason = "The documentation macro expansion makes expect(dead_code) report unfulfilled here even though the non-test library target warns without an allowance; wrapper interpreter wiring uses this private trait in the next step."
	)]
	pub(crate) trait DispatchScopedCarrierHandler<'a, ScopedLayer, FirstLayer, NextProgram, Carrier>
	where
		ScopedLayer: 'a,
		FirstLayer: 'a,
		NextProgram: 'a,
		Carrier: ScopedResumeTypes<'a>, {
		/// Dispatches one around-action scoped-effect layer with access
		/// to the selected action's continuation carrier.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The scoped-effect layer carrying the active around-action scoped operation.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list used by nested interpretation."
		)]
		///
		#[fp_macros::document_returns("The next program produced by the scoped handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct Continuation(i32);
		///
		/// impl Continuation {
		/// 	fn resume_with_post_action(
		/// 		self,
		/// 		post_action: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		post_action(self.0)
		/// 	}
		/// }
		///
		/// struct AddAfterAction;
		///
		/// impl AddAfterAction {
		/// 	fn dispatch(
		/// 		&self,
		/// 		amount: i32,
		/// 		continuation: Continuation,
		/// 	) -> i32 {
		/// 		continuation.resume_with_post_action(|action_value| action_value + amount)
		/// 	}
		/// }
		///
		/// assert_eq!(AddAfterAction.dispatch(1, Continuation(41)), 42);
		/// ```
		fn dispatch_scoped_carrier_head(
			&self,
			layer: ScopedLayer,
			continuation: ScopedContinuation<Carrier>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;
	}

	/// Walks a scoped-handler list through the carrier-aware around-action
	/// route.
	///
	/// This is the private list-level parallel to
	/// [`DispatchScopedHandlers`]. It keeps the ordinary scoped-dispatch path
	/// intact for non-around-action handlers while giving Span-like handlers a
	/// route that receives `SBrand::Of<ActionProgram>` and a typed
	/// continuation carrier. `NextProgram` stays independent from
	/// `ActionProgram`, which lets Span-like handlers observe a selected
	/// borrowed action before the wrapper resumes the final continuation.
	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, produced next program, and carrier.",
		"The scoped row's value-level shape.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type.",
		"The wrapper-owned continuation carrier type."
	)]
	#[fp_macros::document_parameters("The scoped-handler-list instance.")]
	#[allow(
		dead_code,
		reason = "The documentation macro expansion makes expect(dead_code) report unfulfilled here even though the non-test library target warns without an allowance; wrapper interpreter wiring uses this private trait in the next step."
	)]
	pub(crate) trait DispatchScopedCarrierHandlers<
		'a,
		ScopedLayer,
		FirstLayer,
		NextProgram,
		Carrier,
	>
	where
		ScopedLayer: 'a,
		FirstLayer: 'a,
		NextProgram: 'a,
		Carrier: ScopedResumeTypes<'a>, {
		/// Dispatches the scoped row's active variant through the
		/// carrier-aware around-action route.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The scoped row layer carrying the active around-action scoped effect variant.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list used by nested interpretation."
		)]
		///
		#[fp_macros::document_returns("The next program produced by the matching scoped handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// enum Row<A, Rest> {
		/// 	Head(A),
		/// 	Tail(Rest),
		/// }
		///
		/// struct Continuation(i32);
		///
		/// impl Continuation {
		/// 	fn resume_with_post_action(
		/// 		self,
		/// 		post_action: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		post_action(self.0)
		/// 	}
		/// }
		///
		/// let layer: Row<i32, core::convert::Infallible> = Row::Head(1);
		/// let result = match layer {
		/// 	Row::Head(amount) =>
		/// 		Continuation(41).resume_with_post_action(|action_value| action_value + amount),
		/// 	Row::Tail(rest) => match rest {},
		/// };
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_carrier(
			&self,
			layer: ScopedLayer,
			continuation: ScopedContinuation<Carrier>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;
	}

	/// Dispatches only the scoped-handler cell selected by a boundary index.
	///
	/// Boundary dispatch receives a full scoped row carrying the selected
	/// action program, but only the row member that constructed the boundary
	/// should prove carrier-aware semantics. This list-level projection uses the
	/// consumed scoped brand and member index to walk directly to that handler
	/// cell. Non-consumed heads are skipped without requiring them to implement
	/// [`DispatchScopedCarrierHandler`] for the selected boundary carrier.
	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, produced next program, and carrier.",
		"The scoped-effect brand consumed by the boundary head.",
		"The type-level position of the consumed scoped-effect brand in the scoped row.",
		"The full scoped row's value-level shape for the selected action program.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type.",
		"The wrapper-owned continuation carrier type."
	)]
	#[fp_macros::document_parameters("The scoped-handler-list instance.")]
	#[allow(
		dead_code,
		reason = "Introduced before the boundary facade is rewired to use the indexed head projection in the following implementation step."
	)]
	pub(crate) trait DispatchScopedBoundaryHeadHandlers<
		'a,
		ConsumedBrand,
		ConsumedIdx,
		ScopedLayer,
		FirstLayer,
		NextProgram,
		Carrier,
	>
	where
		ConsumedBrand: 'static,
		ScopedLayer: 'a,
		FirstLayer: 'a,
		NextProgram: 'a,
		Carrier: ScopedResumeTypes<'a>, {
		/// Dispatch the boundary-selected scoped row member through its
		/// carrier-aware handler.
		#[fp_macros::document_signature]
		#[fp_macros::document_parameters(
			"The full scoped row layer carrying the selected boundary operation.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list used by nested interpretation."
		)]
		#[fp_macros::document_returns(
			"The next program produced by the selected boundary handler."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// enum Row<Prefix, Selected> {
		/// 	Prefix(Prefix),
		/// 	Selected(Selected),
		/// }
		///
		/// let layer: Row<&'static str, i32> = Row::Selected(42);
		/// let result = match layer {
		/// 	Row::Prefix(_) => unreachable!("the boundary selected the tail member"),
		/// 	Row::Selected(value) => value,
		/// };
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_boundary_head(
			&self,
			layer: ScopedLayer,
			continuation: ScopedContinuation<Carrier>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;
	}

	/// Residual ordinary scoped-dispatch route after a boundary head is consumed.
	///
	/// Around-action boundary constructors can require a handler cell whose
	/// ordinary `DispatchScopedHandler` shape is intentionally not implemented
	/// for the final program slot. Writer `listen` is the motivating example:
	/// the boundary handler resumes the outer continuation with
	/// `(action_value, observed_log)`, but an ordinary
	/// `WriterListen<NextProgram>` handler would have no sound way to produce
	/// that observed log. This private trait walks the same scoped-handler list
	/// and scoped row after the boundary is consumed, skipping exactly the
	/// consumed member position while preserving ordinary dispatch for every
	/// other position.
	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, and produced next program.",
		"The scoped-effect brand consumed by the boundary head.",
		"The type-level position of the consumed scoped-effect brand in the scoped row.",
		"The full scoped row's value-level shape for the final program.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The scoped-handler-list instance.")]
	#[doc(hidden)]
	pub trait DispatchResidualScopedHandlers<
		'a,
		ConsumedBrand,
		ConsumedIdx,
		ScopedLayer,
		FirstLayer,
		NextProgram,
	>
	where
		ConsumedBrand: 'static,
		ScopedLayer: 'a,
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Dispatch a post-boundary ordinary scoped layer after removing
		/// the boundary-only member position.
		#[fp_macros::document_signature]
		#[fp_macros::document_parameters(
			"The full scoped row layer produced after the boundary resumes.",
			"The first-order handler list used by nested interpretation."
		)]
		#[fp_macros::document_returns(
			"The next program produced by the residual ordinary scoped handler."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// enum Row<Consumed, Rest> {
		/// 	Consumed(Consumed),
		/// 	Rest(Rest),
		/// }
		///
		/// let layer: Row<&'static str, i32> = Row::Rest(42);
		/// let result = match layer {
		/// 	Row::Consumed(_) => unreachable!("boundary-only operation reappeared"),
		/// 	Row::Rest(value) => value,
		/// };
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_residual_scoped(
			&self,
			layer: ScopedLayer,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the boundary, first-order layer, produced next program, and private carrier.",
		"The typed scoped boundary value.",
		"The scoped row layer produced by splitting the boundary.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type.",
		"The wrapper-owned continuation carrier type."
	)]
	#[fp_macros::document_parameters("The scoped-handler-list instance.")]
	impl<'a, Boundary, ScopedLayer, FirstLayer, NextProgram, Carrier, Handlers>
		DispatchScopedBoundaryHandlers<'a, Boundary, FirstLayer, NextProgram> for Handlers
	where
		Boundary: IntoScopedBoundaryParts<'a, ScopedLayer = ScopedLayer, Carrier = Carrier> + 'a,
		ScopedLayer: 'a,
		FirstLayer: 'a,
		NextProgram: 'a,
		Carrier: ScopedResumeTypes<'a>,
		<Boundary as IntoScopedBoundaryParts<'a>>::ConsumedBrand: 'static,
		Handlers: DispatchScopedBoundaryHeadHandlers<
				'a,
				<Boundary as IntoScopedBoundaryParts<'a>>::ConsumedBrand,
				<Boundary as IntoScopedBoundaryParts<'a>>::ConsumedIdx,
				ScopedLayer,
				FirstLayer,
				NextProgram,
				Carrier,
			>,
	{
		/// Split a public boundary and dispatch its selected scoped member
		/// through the private indexed boundary-head projection.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The typed scoped boundary carrying the selected action and outer continuation.",
			"The first-order handler list used while resuming the selected action."
		)]
		///
		#[fp_macros::document_returns(
			"The next program produced after the matching scoped handler consumes the boundary."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct Boundary {
		/// 	action_value: i32,
		/// }
		///
		/// impl Boundary {
		/// 	fn dispatch(self) -> i32 {
		/// 		self.action_value + 1
		/// 	}
		/// }
		///
		/// let result = Boundary {
		/// 	action_value: 41,
		/// }
		/// .dispatch();
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		fn dispatch_scoped_boundary(
			&self,
			boundary: Boundary,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			let (layer, continuation) = boundary.into_scoped_boundary_parts();
			self.dispatch_scoped_boundary_head(layer, continuation, fo_handlers)
		}
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, and produced next program.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters(
		"The empty scoped-handler list (unused; the scoped layer is uninhabited)."
	)]
	impl<'a, FirstLayer, NextProgram> DispatchScopedHandlers<'a, CNil, FirstLayer, NextProgram>
		for ScopedHandlersNil
	where
		FirstLayer: 'a,
		NextProgram: 'a,
	{
		/// Base case: an empty scoped row carries no scoped effects, so
		/// the scoped layer is uninhabited and the body diverges via
		/// exhaustive match.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The uninhabited scoped row layer.",
			"The first-order handler list (unused)."
		)]
		///
		#[fp_macros::document_returns("Diverges; never returns.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::{
		/// 	DispatchScopedHandlers,
		/// 	HandlersNil,
		/// 	ScopedHandlersNil,
		/// 	coproduct::CNil,
		/// };
		///
		/// fn dispatch_empty(layer: CNil) -> i32 {
		/// 	ScopedHandlersNil.dispatch_scoped(layer, &HandlersNil)
		/// }
		///
		/// let _call_shape: fn(CNil) -> i32 = dispatch_empty;
		/// let absent_layer: Option<CNil> = None;
		/// assert!(absent_layer.is_none());
		/// ```
		#[inline]
		fn dispatch_scoped(
			&self,
			layer: CNil,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			match layer {}
		}
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, and produced next program.",
		"The scoped-effect brand at this row position.",
		"The handler value stored in the head cell.",
		"The tail scoped-handler list type.",
		"The remaining scoped row brands after this position.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The cons cell of the scoped-handler list.")]
	impl<'a, SBrand, F, T, Rest, FirstLayer, NextProgram>
		DispatchScopedHandlers<
			'a,
			Coproduct<<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>, Rest>,
			FirstLayer,
			NextProgram,
		> for ScopedHandlersCons<ScopedHandler<SBrand, F>, T>
	where
		SBrand: Kind_cdc7cd43dac7585f + 'static,
		F: DispatchScopedHandler<
				'a,
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>,
				FirstLayer,
				NextProgram,
			>,
		T: DispatchScopedHandlers<'a, Rest, FirstLayer, NextProgram>,
		FirstLayer: 'a,
		NextProgram: 'a,
		Rest: 'a,
		<SBrand as Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>: 'a,
	{
		/// Cons-cell case for scoped rows: dispatches `Inl` to the head
		/// scoped handler and recurses `Inr` into the tail.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The scoped row layer carrying the active scoped effect variant.",
			"The first-order handler list used by nested interpretation."
		)]
		///
		#[fp_macros::document_returns("The next program produced by the matching scoped handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::IdentityBrand,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			DispatchHandlers,
		/// 			DispatchScopedHandler,
		/// 			DispatchScopedHandlers,
		/// 			HandlersNil,
		/// 			coproduct::{
		/// 				CNil,
		/// 				Coproduct,
		/// 			},
		/// 			scoped_nt,
		/// 		},
		/// 	},
		/// };
		///
		/// struct IdentityScoped;
		///
		/// impl<'a> DispatchScopedHandler<'a, Identity<i32>, CNil, i32> for IdentityScoped {
		/// 	fn dispatch_scoped_head(
		/// 		&self,
		/// 		layer: Identity<i32>,
		/// 		_fo_handlers: &impl DispatchHandlers<'a, CNil, i32>,
		/// 	) -> i32 {
		/// 		layer.0
		/// 	}
		/// }
		///
		/// let scoped_handlers =
		/// 	fp_library::types::effects::scoped_nt().on::<IdentityBrand, _>(IdentityScoped);
		/// let layer = Coproduct::Inl(Identity(42));
		/// let result = scoped_handlers.dispatch_scoped(layer, &HandlersNil);
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		fn dispatch_scoped(
			&self,
			layer: Coproduct<
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>,
				Rest,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			match layer {
				Coproduct::Inl(scoped) => self.head.run.dispatch_scoped_head(scoped, fo_handlers),
				Coproduct::Inr(rest) => self.tail.dispatch_scoped(rest, fo_handlers),
			}
		}
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, produced next program, and carrier.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type.",
		"The wrapper-owned continuation carrier type."
	)]
	#[fp_macros::document_parameters(
		"The empty scoped-handler list (unused; the scoped layer is uninhabited)."
	)]
	impl<'a, FirstLayer, NextProgram, Carrier>
		DispatchScopedCarrierHandlers<'a, CNil, FirstLayer, NextProgram, Carrier> for ScopedHandlersNil
	where
		FirstLayer: 'a,
		NextProgram: 'a,
		Carrier: ScopedResumeTypes<'a>,
	{
		/// Base case: an empty scoped row carries no scoped effects, so
		/// the carrier-aware scoped layer is uninhabited and the body
		/// diverges via exhaustive match.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The uninhabited scoped row layer.",
			"The wrapper-owned continuation carrier (unused).",
			"The first-order handler list (unused)."
		)]
		///
		#[fp_macros::document_returns("Diverges; never returns.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// enum Never {}
		///
		/// fn dispatch_empty(layer: Never) -> i32 {
		/// 	match layer {}
		/// }
		///
		/// let _call_shape: fn(Never) -> i32 = dispatch_empty;
		/// let absent_layer: Option<Never> = None;
		/// assert!(absent_layer.is_none());
		/// ```
		#[inline]
		fn dispatch_scoped_carrier(
			&self,
			layer: CNil,
			_continuation: ScopedContinuation<Carrier>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			match layer {}
		}
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, produced next program, and carrier.",
		"The scoped-effect brand at this row position.",
		"The handler value stored in the head cell.",
		"The tail scoped-handler list type.",
		"The remaining scoped row brands after this position.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type.",
		"The wrapper-owned continuation carrier type."
	)]
	#[fp_macros::document_parameters("The cons cell of the scoped-handler list.")]
	impl<'a, SBrand, F, T, Rest, FirstLayer, NextProgram, Carrier>
		DispatchScopedCarrierHandlers<
			'a,
			Coproduct<
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<
					'a,
					<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
				>,
				Rest,
			>,
			FirstLayer,
			NextProgram,
			Carrier,
		> for ScopedHandlersCons<ScopedHandler<SBrand, F>, T>
	where
		SBrand: Kind_cdc7cd43dac7585f + 'static,
		F: DispatchScopedCarrierHandler<
				'a,
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<
					'a,
					<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
				>,
				FirstLayer,
				NextProgram,
				Carrier,
			>,
		T: DispatchScopedCarrierHandlers<'a, Rest, FirstLayer, NextProgram, Carrier>,
		FirstLayer: 'a,
		NextProgram: 'a,
		Carrier: ScopedResumeTypes<'a>,
		Rest: 'a,
		<SBrand as Kind_cdc7cd43dac7585f>::Of<
			'a,
			<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
		>: 'a,
	{
		/// Cons-cell case for carrier-aware scoped rows: dispatches
		/// `Inl` to the head scoped handler and recurses `Inr` into
		/// the tail, preserving the same continuation carrier.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The scoped row layer carrying the active around-action scoped effect variant.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list used by nested interpretation."
		)]
		///
		#[fp_macros::document_returns("The next program produced by the matching scoped handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// enum Row<A, Rest> {
		/// 	Head(A),
		/// 	Tail(Rest),
		/// }
		///
		/// struct Continuation(i32);
		///
		/// impl Continuation {
		/// 	fn resume_with_post_action(
		/// 		self,
		/// 		post_action: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		post_action(self.0)
		/// 	}
		/// }
		///
		/// let layer: Row<i32, core::convert::Infallible> = Row::Head(1);
		/// let result = match layer {
		/// 	Row::Head(amount) =>
		/// 		Continuation(41).resume_with_post_action(|action_value| action_value + amount),
		/// 	Row::Tail(rest) => match rest {},
		/// };
		///
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier(
			&self,
			layer: Coproduct<
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<
					'a,
					<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
				>,
				Rest,
			>,
			continuation: ScopedContinuation<Carrier>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			match layer {
				Coproduct::Inl(scoped) =>
					self.head.run.dispatch_scoped_carrier_head(scoped, continuation, fo_handlers),
				Coproduct::Inr(rest) =>
					self.tail.dispatch_scoped_carrier(rest, continuation, fo_handlers),
			}
		}
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, produced next program, and carrier.",
		"The consumed scoped-effect brand at this row position.",
		"The handler value stored in the consumed head cell.",
		"The tail scoped-handler list type.",
		"The remaining scoped row brands after the consumed position.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type.",
		"The wrapper-owned continuation carrier type."
	)]
	#[fp_macros::document_parameters("The scoped-handler cons cell at the consumed position.")]
	impl<'a, ConsumedBrand, F, T, Rest, FirstLayer, NextProgram, Carrier>
		DispatchScopedBoundaryHeadHandlers<
			'a,
			ConsumedBrand,
			Here,
			Coproduct<
				<ConsumedBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<
					'a,
					<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
				>,
				Rest,
			>,
			FirstLayer,
			NextProgram,
			Carrier,
		> for ScopedHandlersCons<ScopedHandler<ConsumedBrand, F>, T>
	where
		ConsumedBrand: Kind_cdc7cd43dac7585f + 'static,
		F: DispatchScopedCarrierHandler<
				'a,
				<ConsumedBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<
					'a,
					<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
				>,
				FirstLayer,
				NextProgram,
				Carrier,
			>,
		FirstLayer: 'a,
		NextProgram: 'a,
		Carrier: ScopedResumeTypes<'a>,
		Rest: 'a,
		<ConsumedBrand as Kind_cdc7cd43dac7585f>::Of<
			'a,
			<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
		>: 'a,
	{
		/// Dispatch the consumed boundary head through its carrier-aware handler.
		#[fp_macros::document_signature]
		#[fp_macros::document_parameters(
			"The full scoped row layer carrying the selected boundary operation.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list used by nested interpretation."
		)]
		#[fp_macros::document_returns(
			"The next program produced by the selected boundary handler."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// enum Row<Selected, Tail> {
		/// 	Selected(Selected),
		/// 	Tail(Tail),
		/// }
		///
		/// let layer: Row<i32, &'static str> = Row::Selected(42);
		/// let result = match layer {
		/// 	Row::Selected(value) => value,
		/// 	Row::Tail(_) => unreachable!("the boundary selected the head member"),
		/// };
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "A boundary layer at the tail while the consumed index is Here means the boundary member evidence and runtime layer diverged."
		)]
		fn dispatch_scoped_boundary_head(
			&self,
			layer: Coproduct<
				<ConsumedBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<
					'a,
					<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
				>,
				Rest,
			>,
			continuation: ScopedContinuation<Carrier>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			match layer {
				Coproduct::Inl(scoped) =>
					self.head.run.dispatch_scoped_carrier_head(scoped, continuation, fo_handlers),
				Coproduct::Inr(_) => unreachable!(
					"boundary-selected scoped layer appeared after the consumed head position"
				),
			}
		}
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, produced next program, and carrier.",
		"The non-consumed scoped-effect brand at this row position.",
		"The handler value stored in the non-consumed head cell.",
		"The tail scoped-handler list type.",
		"The consumed scoped-effect brand deeper in the scoped row.",
		"The type-level position of the consumed scoped-effect brand in the tail row.",
		"The remaining scoped row brands after this position.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type.",
		"The wrapper-owned continuation carrier type."
	)]
	#[fp_macros::document_parameters("The scoped-handler cons cell before the consumed position.")]
	impl<'a, SBrand, F, T, ConsumedBrand, ConsumedTailIdx, Rest, FirstLayer, NextProgram, Carrier>
		DispatchScopedBoundaryHeadHandlers<
			'a,
			ConsumedBrand,
			There<ConsumedTailIdx>,
			Coproduct<
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<
					'a,
					<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
				>,
				Rest,
			>,
			FirstLayer,
			NextProgram,
			Carrier,
		> for ScopedHandlersCons<ScopedHandler<SBrand, F>, T>
	where
		SBrand: Kind_cdc7cd43dac7585f + 'static,
		ConsumedBrand: Kind_cdc7cd43dac7585f + 'static,
		T: DispatchScopedBoundaryHeadHandlers<
				'a,
				ConsumedBrand,
				ConsumedTailIdx,
				Rest,
				FirstLayer,
				NextProgram,
				Carrier,
			>,
		FirstLayer: 'a,
		NextProgram: 'a,
		Carrier: ScopedResumeTypes<'a>,
		Rest: 'a,
		<SBrand as Kind_cdc7cd43dac7585f>::Of<
			'a,
			<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
		>: 'a,
	{
		/// Skip a non-consumed head and recurse toward the consumed boundary member.
		#[fp_macros::document_signature]
		#[fp_macros::document_parameters(
			"The full scoped row layer carrying the selected boundary operation.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list used by nested interpretation."
		)]
		#[fp_macros::document_returns(
			"The next program produced by the selected boundary handler."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// enum Row<Prefix, Selected> {
		/// 	Prefix(Prefix),
		/// 	Selected(Selected),
		/// }
		///
		/// let layer: Row<&'static str, i32> = Row::Selected(42);
		/// let result = match layer {
		/// 	Row::Prefix(_) => unreachable!("the boundary selected the tail member"),
		/// 	Row::Selected(value) => value,
		/// };
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "A boundary layer at a non-consumed prefix means the boundary member evidence and runtime layer diverged."
		)]
		fn dispatch_scoped_boundary_head(
			&self,
			layer: Coproduct<
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<
					'a,
					<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
				>,
				Rest,
			>,
			continuation: ScopedContinuation<Carrier>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			match layer {
				Coproduct::Inl(_) => unreachable!(
					"boundary-selected scoped layer appeared before the consumed member position"
				),
				Coproduct::Inr(rest) =>
					self.tail.dispatch_scoped_boundary_head(rest, continuation, fo_handlers),
			}
		}
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, and produced next program.",
		"The consumed scoped-effect brand at this row position.",
		"The handler value stored in the consumed head cell.",
		"The tail scoped-handler list type.",
		"The remaining scoped row brands after the consumed position.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The scoped-handler cons cell at the consumed position.")]
	impl<'a, ConsumedBrand, F, T, Rest, FirstLayer, NextProgram>
		DispatchResidualScopedHandlers<
			'a,
			ConsumedBrand,
			Here,
			Coproduct<
				<ConsumedBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>,
				Rest,
			>,
			FirstLayer,
			NextProgram,
		> for ScopedHandlersCons<ScopedHandler<ConsumedBrand, F>, T>
	where
		ConsumedBrand: Kind_cdc7cd43dac7585f + 'static,
		T: DispatchScopedHandlers<'a, Rest, FirstLayer, NextProgram>,
		FirstLayer: 'a,
		NextProgram: 'a,
		Rest: 'a,
		<ConsumedBrand as Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>: 'a,
	{
		/// Skip the consumed boundary head and dispatch only the residual tail.
		#[fp_macros::document_signature]
		#[fp_macros::document_parameters(
			"The full scoped row layer produced after the boundary resumes.",
			"The first-order handler list used by nested interpretation."
		)]
		#[fp_macros::document_returns(
			"The next program produced by the residual ordinary scoped handler."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// enum Row<Consumed, Rest> {
		/// 	Consumed(Consumed),
		/// 	Rest(Rest),
		/// }
		///
		/// let layer: Row<&'static str, i32> = Row::Rest(42);
		/// let result = match layer {
		/// 	Row::Consumed(_) => unreachable!("boundary-only operation reappeared"),
		/// 	Row::Rest(value) => value,
		/// };
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "A post-boundary ordinary layer at the consumed boundary-only position means the private boundary/residual split was violated."
		)]
		fn dispatch_residual_scoped(
			&self,
			layer: Coproduct<
				<ConsumedBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>,
				Rest,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			match layer {
				Coproduct::Inl(_) => unreachable!(
					"post-boundary scoped layer reintroduced the consumed boundary-only member"
				),
				Coproduct::Inr(rest) => self.tail.dispatch_scoped(rest, fo_handlers),
			}
		}
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, and produced next program.",
		"The non-consumed scoped-effect brand at this row position.",
		"The handler value stored in the non-consumed head cell.",
		"The tail scoped-handler list type.",
		"The consumed scoped-effect brand deeper in the scoped row.",
		"The type-level position of the consumed scoped-effect brand in the tail row.",
		"The remaining scoped row brands after this position.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The scoped-handler cons cell before the consumed position.")]
	impl<'a, SBrand, F, T, ConsumedBrand, ConsumedTailIdx, Rest, FirstLayer, NextProgram>
		DispatchResidualScopedHandlers<
			'a,
			ConsumedBrand,
			There<ConsumedTailIdx>,
			Coproduct<<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>, Rest>,
			FirstLayer,
			NextProgram,
		> for ScopedHandlersCons<ScopedHandler<SBrand, F>, T>
	where
		SBrand: Kind_cdc7cd43dac7585f + 'static,
		ConsumedBrand: Kind_cdc7cd43dac7585f + 'static,
		F: DispatchScopedHandler<
				'a,
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>,
				FirstLayer,
				NextProgram,
			>,
		T: DispatchResidualScopedHandlers<
				'a,
				ConsumedBrand,
				ConsumedTailIdx,
				Rest,
				FirstLayer,
				NextProgram,
			>,
		FirstLayer: 'a,
		NextProgram: 'a,
		Rest: 'a,
		<SBrand as Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>: 'a,
	{
		/// Dispatch non-consumed heads normally and recurse into the tail
		/// until the consumed position is found.
		#[fp_macros::document_signature]
		#[fp_macros::document_parameters(
			"The full scoped row layer produced after the boundary resumes.",
			"The first-order handler list used by nested interpretation."
		)]
		#[fp_macros::document_returns(
			"The next program produced by the residual ordinary scoped handler."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// enum Row<Head, Tail> {
		/// 	Head(Head),
		/// 	Tail(Tail),
		/// }
		///
		/// let layer: Row<i32, &'static str> = Row::Head(42);
		/// let result = match layer {
		/// 	Row::Head(value) => value,
		/// 	Row::Tail(_) => 0,
		/// };
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		fn dispatch_residual_scoped(
			&self,
			layer: Coproduct<
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>,
				Rest,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			match layer {
				Coproduct::Inl(scoped) => self.head.run.dispatch_scoped_head(scoped, fo_handlers),
				Coproduct::Inr(rest) => self.tail.dispatch_residual_scoped(rest, fo_handlers),
			}
		}
	}
}

pub use inner::*;

#[cfg(test)]
mod scoped_continuation_tests;
