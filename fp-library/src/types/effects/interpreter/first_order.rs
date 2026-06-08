#[fp_macros::document_module]
pub(crate) mod inner {
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
	/// `handle_rec`). Handler closures stored in [`Handler<E, F>`]
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
	///
	/// Missing first-order handlers surface as ordinary Rust trait
	/// errors against this trait. If an `handle` call reports that
	/// `DispatchHandlers<..., Coproduct<...>>` is not implemented for
	/// the supplied handler list, inspect the remaining `Coproduct`
	/// head. For `effects!` rows that head is usually
	/// `Coyoneda<'_, MissingBrand, NextProgram>`; add a matching
	/// `MissingBrand: ...` entry to the `handlers!` list.
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
		/// 	brands::IdentityBrand,
		/// 	types::{
		/// 		Coyoneda,
		/// 		Identity,
		/// 		effects::{
		/// 			DispatchHandlers,
		/// 			coproduct::{
		/// 				CNil,
		/// 				Coproduct,
		/// 			},
		/// 			handlers::handlers_ordered,
		/// 		},
		/// 	},
		/// };
		///
		/// let handlers = handlers_ordered().on::<IdentityBrand, _>(|op: Identity<i32>| op.0 + 1).finish();
		/// let layer: Coproduct<Coyoneda<'static, IdentityBrand, i32>, CNil> =
		/// 	Coproduct::Inl(Coyoneda::lift(Identity(41)));
		///
		/// let result = handlers.dispatch(layer);
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch(
			&self,
			layer: Layer,
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "The layer type is CNil, so a real value cannot be constructed for an executable direct call; the example documents the callable shape through an uncalled helper and asserts that no layer exists."
		)]
		///
		/// ```
		/// use fp_library::types::effects::{
		/// 	DispatchHandlers,
		/// 	HandlersNil,
		/// 	coproduct::CNil,
		/// };
		///
		/// fn dispatch_empty(layer: CNil) -> i32 {
		/// 	HandlersNil.dispatch(layer)
		/// }
		///
		/// // The `HandlersNil` / `CNil` base case is the recursion
		/// // terminator. No value of CNil can be constructed, so the
		/// // real method body is an exhaustive match over an impossible layer.
		/// let _call_shape: fn(CNil) -> i32 = dispatch_empty;
		/// let absent_layer: Option<CNil> = None;
		/// assert!(absent_layer.is_none());
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
		/// 	brands::IdentityBrand,
		/// 	types::{
		/// 		Coyoneda,
		/// 		Identity,
		/// 		effects::{
		/// 			DispatchHandlers,
		/// 			coproduct::{
		/// 				CNil,
		/// 				Coproduct,
		/// 			},
		/// 			handlers::handlers_ordered,
		/// 		},
		/// 	},
		/// };
		///
		/// let handlers = handlers_ordered().on::<IdentityBrand, _>(|op: Identity<i32>| op.0).finish();
		/// let layer: Coproduct<Coyoneda<'static, IdentityBrand, i32>, CNil> =
		/// 	Coproduct::Inl(Coyoneda::lift(Identity(99)));
		///
		/// let result = handlers.dispatch(layer);
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
		/// 	brands::IdentityBrand,
		/// 	types::{
		/// 		Identity,
		/// 		RcCoyoneda,
		/// 		effects::{
		/// 			DispatchHandlers,
		/// 			coproduct::{
		/// 				CNil,
		/// 				Coproduct,
		/// 			},
		/// 			handlers::handlers_ordered,
		/// 		},
		/// 	},
		/// };
		///
		/// let handlers = handlers_ordered().on::<IdentityBrand, _>(|op: Identity<i32>| op.0).finish();
		/// let layer: Coproduct<RcCoyoneda<'static, IdentityBrand, i32>, CNil> =
		/// 	Coproduct::Inl(RcCoyoneda::lift(Identity(11)));
		///
		/// let result = handlers.dispatch(layer);
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
		/// 	brands::IdentityBrand,
		/// 	types::{
		/// 		ArcCoyoneda,
		/// 		Identity,
		/// 		effects::{
		/// 			DispatchHandlers,
		/// 			coproduct::{
		/// 				CNil,
		/// 				Coproduct,
		/// 			},
		/// 			handlers::handlers_ordered,
		/// 		},
		/// 	},
		/// };
		///
		/// let handlers = handlers_ordered().on::<IdentityBrand, _>(|op: Identity<i32>| op.0).finish();
		/// let layer: Coproduct<ArcCoyoneda<'static, IdentityBrand, i32>, CNil> =
		/// 	Coproduct::Inl(ArcCoyoneda::lift(Identity(13)));
		///
		/// let result = handlers.dispatch(layer);
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
}

pub use inner::*;
