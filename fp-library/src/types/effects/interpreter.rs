//! Interpreter family for Run programs.
//!
//! Provides the [`DispatchHandlers`] trait that walks a row's
//! value-level [`Coproduct`](crate::types::effects::coproduct::Coproduct)
//! variants against a
//! [`HandlersCons`](crate::types::effects::handlers::HandlersCons) /
//! [`HandlersNil`](crate::types::effects::handlers::HandlersNil) handler
//! list in lock-step, dispatching the active variant to its matching
//! [`Handler`](crate::types::effects::handlers::Handler) closure. Each Run wrapper
//! exposes inherent `interpret` / `run` methods that
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
//! `tail_rec_m` drives `interpret_rec`'s stack-safe loop) has no
//! impl for `Future`-shaped target monads in this library; without
//! that, an async interpreter cannot satisfy the same stack-safety
//! contract the sync family does.
//!
//! For programs that need to interleave async work with effect
//! interpretation today, the supported workaround is
//! [`tokio::task::spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
//! (or the equivalent on other runtimes): wrap the synchronous
//! `interpret` call inside a blocking task, await the join handle
//! from async code. Handler closures may themselves block on
//! [`tokio::runtime::Handle::block_on`](https://docs.rs/tokio/latest/tokio/runtime/struct.Handle.html#method.block_on)
//! to call out to async APIs from inside the interpreter, at the
//! cost of one blocking-thread-pool slot per concurrent program.
//!
//! This is a runtime-level workaround, not a library feature. A
//! native `Future`-based interpreter would compose better but
//! requires a `MonadRec` impl over `Future` first.

#[fp_macros::document_module]
mod inner {
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
					ScopedHandler,
					ScopedHandlersCons,
					ScopedHandlersNil,
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
	///
	/// **Note: `Fn` vs `FnOnce` asymmetry.** Each Run wrapper's
	/// inherent `bind` method
	/// ([`Run::bind`](crate::types::effects::run::Run::bind) and
	/// siblings) takes `f: FnOnce(A) -> ...` (single-shot, matching
	/// the Free continuation queue's storage). Handler closures
	/// here are `Fn` (multi-shot, callable inside `tail_rec_m`'s
	/// step closure). Converting one shape to the other requires
	/// either an interior-mutability capture or wrapping in
	/// `Rc`/`Arc`; the asymmetry is structural, driven by the two
	/// call sites' differing reentry needs.
	#[fp_macros::document_type_parameters(
		"The lifetime of the layer and the produced next program.",
		"The row's value-level shape (typically a `Coproduct` chain).",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The handler-list instance.")]
	pub trait DispatchHandlers<'a, Layer, NextProgram>
	where
		Layer: 'a,
		NextProgram: 'a, {
		/// Dispatches the row's active variant to the matching handler
		/// closure, producing the next program.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The row layer carrying the active effect variant.")]
		///
		#[fp_macros::document_returns("The next program produced by the matching handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// // `dispatch` is invoked internally by `Run::interpret` once per
		/// // peeled `Node::First` layer. The handler list passed to
		/// // `interpret` becomes the `&self` receiver of `dispatch`.
		/// let prog: Run<FirstRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let result = prog.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Run<FirstRow, CNilBrand, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch(
			&self,
			layer: Layer,
		) -> NextProgram;
	}

	/// Dispatch contract for a single scoped-handler cell.
	///
	/// Unlike first-order [`Handler`] values, scoped handlers cannot be
	/// plain `Fn` closures in the general case: they receive the
	/// first-order handler list, and that list's concrete type remains
	/// generic at the method level. Standard scoped dispatchers and
	/// user-defined scoped dispatcher values implement this trait, then
	/// [`DispatchScopedHandlers`] lifts them into a recursive handler
	/// list.
	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, and produced next program.",
		"The active scoped-effect layer handled by this cell.",
		"The first-order row's value-level layer shape.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The scoped-handler dispatcher value.")]
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

	#[fp_macros::document_type_parameters(
		"The lifetime of the layer.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The empty handler list (unused; the layer is uninhabited).")]
	impl<'a, NextProgram> DispatchHandlers<'a, CNil, NextProgram> for HandlersNil
	where
		NextProgram: 'a,
	{
		/// Base case: an empty row carries no effects, so the layer is
		/// uninhabited and the body diverges via exhaustive match.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The uninhabited row layer.")]
		///
		#[fp_macros::document_returns("Diverges; never returns.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// // The `HandlersNil` / `CNil` base case is the recursion
		/// // terminator: when `interpret` walks past every cons-cell
		/// // dispatch impl, it eventually lands here on the `CNil`
		/// // tail, which is uninhabited and matches exhaustively.
		/// let prog: Run<FirstRow, CNilBrand, i32> = Run::pure(7);
		/// let result = prog.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Run<FirstRow, CNilBrand, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 7);
		/// ```
		#[inline]
		fn dispatch(
			&self,
			layer: CNil,
		) -> NextProgram {
			match layer {}
		}
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the layer and the next program.",
		"The effect brand at this row position.",
		"The handler closure stored in the head cell.",
		"The tail handler list type.",
		"The remaining row brands after this position.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The cons cell of the handler list.")]
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
		/// Cons-cell case for bare [`Coyoneda`]: dispatches `Inl` to
		/// the head handler (after lowering the Coyoneda via
		/// `EBrand`'s [`Functor`]) and recurses `Inr` into the tail.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The row layer carrying the active effect variant.")]
		///
		#[fp_macros::document_returns("The next program produced by the matching handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// // Bare-Coyoneda dispatch impl is invoked by `Run::interpret` /
		/// // `RunExplicit::interpret` per peeled `Node::First` layer.
		/// let prog: Run<FirstRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(99));
		/// let result = prog.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Run<FirstRow, CNilBrand, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 99);
		/// ```
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

	#[fp_macros::document_type_parameters(
		"The lifetime of the layer and the next program.",
		"The effect brand at this row position.",
		"The handler closure stored in the head cell.",
		"The tail handler list type.",
		"The remaining row brands after this position.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The cons cell of the handler list.")]
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
		/// Cons-cell case for [`RcCoyoneda`]: like the bare-Coyoneda
		/// case but uses [`RcCoyoneda::lower_ref`] for the multi-shot
		/// shared-pointer substrate.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The row layer carrying the active effect variant.")]
		///
		#[fp_macros::document_returns("The next program produced by the matching handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// // The `RcCoyoneda` dispatch impl is invoked by
		/// // `RcRun::interpret` / `RcRunExplicit::interpret` per peeled
		/// // layer; `lower_ref` preserves the underlying `Rc`-shared
		/// // continuation for multi-shot use.
		/// let prog: RcRun<FirstRow, CNilBrand, i32> = RcRun::lift::<IdentityBrand, _>(Identity(11));
		/// let result = prog.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<RcRun<FirstRow, CNilBrand, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 11);
		/// ```
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

	#[fp_macros::document_type_parameters(
		"The lifetime of the layer and the next program.",
		"The effect brand at this row position (must satisfy [`SendFunctor`]).",
		"The handler closure stored in the head cell.",
		"The tail handler list type.",
		"The remaining row brands after this position.",
		"The Run wrapper specialized to the program's result type ([`Send`] + [`Sync`])."
	)]
	#[fp_macros::document_parameters("The cons cell of the handler list.")]
	impl<'a, EBrand, F, T, Rest, NextProgram>
		DispatchHandlers<'a, Coproduct<ArcCoyoneda<'a, EBrand, NextProgram>, Rest>, NextProgram>
		for HandlersCons<Handler<EBrand, F>, T>
	where
		EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
		F: Fn(<EBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>) -> NextProgram,
		T: DispatchHandlers<'a, Rest, NextProgram>,
		NextProgram: Send + Sync + 'a,
		Rest: 'a,
		<EBrand as Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>: Send + Sync + 'a,
	{
		/// Cons-cell case for [`ArcCoyoneda`]: like the [`RcCoyoneda`]
		/// case but adds [`Send`] + [`Sync`] bounds for the thread-safe
		/// substrate.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The row layer carrying the active effect variant.")]
		///
		#[fp_macros::document_returns("The next program produced by the matching handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::ArcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// // The `ArcCoyoneda` dispatch impl is invoked by
		/// // `ArcRun::interpret` / `ArcRunExplicit::interpret`. The
		/// // `Send + Sync` bounds on `NextProgram` and the inner
		/// // projection let the dispatched continuation cross thread
		/// // boundaries.
		/// let prog: ArcRun<FirstRow, CNilBrand, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(13));
		/// let result = prog.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<ArcRun<FirstRow, CNilBrand, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 13);
		/// ```
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
		/// assert!(core::mem::size_of::<CNil>() == 0);
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
		"The dispatcher value stored in the head cell.",
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
		/// scoped dispatcher and recurses `Inr` into the tail.
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
}

pub use inner::*;
