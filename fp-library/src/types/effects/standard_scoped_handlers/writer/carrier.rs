#[fp_macros::document_module]
pub(crate) mod inner {
	use super::super::{
		super::prelude::*,
		inner::{
			WriterPostHandler,
			WriterPreHandler,
		},
		raw_accumulators::*,
		raw_rewriters::*,
	};

	/// Carrier-aware Writer pre-censor dispatch for `RunExplicitBoundary`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final program result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer pre-handler receiver.")]
	impl<'a, R, S, Action, Final, K, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			BoxWriterCensor<'a, BoxBrand, W, RunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RunExplicit<'a, R, S, Final>,
			RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for WriterPreHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: 'a,
		Final: 'a,
		K: Fn(Action) -> RunExplicit<'a, R, S, Final> + 'a,
		W: 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RunExplicit<'a, R, S, Action>,
		>): Member<
				Coyoneda<'a, WriterBrand<W>, RunExplicit<'a, R, S, Action>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									RunExplicit<'a, R, S, Action>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			Box<FreeExplicit<'a, NodeBrand<R, S>, Action>>,
		>): Member<Coyoneda<'a, WriterBrand<W>, Box<FreeExplicit<'a, NodeBrand<R, S>, Action>>>, Idx>,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			Box<FreeExplicit<'a, NodeBrand<R, S>, Action>>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					Box<FreeExplicit<'a, NodeBrand<R, S>, Action>>,
				>),
				EmbedIndices,
			>,
	{
		/// Rewrite selected-action `Tell`s before resuming the outer continuation.
		#[document_signature]
		#[document_parameters(
			"The Writer censor layer carrying the selected action.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the Writer boundary.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: BoxWriterCensor<'a, BoxBrand, W, RunExplicit<'a, R, S, Action>>,
			continuation: ScopedContinuation<
				RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
		) -> RunExplicit<'a, R, S, Final> {
			match layer {
				BoxWriterCensor::Censor {
					censor,
					action,
				} => continuation.resume_explicit_with_supplied_action(fo_handlers, move || {
					action(())
						.interpose_with_rewriter::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices>(
							BoxWriterPreRewriter {
								censor,
							},
						)
				}),
			}
		}
	}

	/// Ordinary scoped Writer pre-censor dispatch for `RunExplicit`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The program result type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer pre-handler receiver.")]
	impl<'a, R, S, A, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedHandler<
			'a,
			BoxWriterCensor<'a, BoxBrand, W, RunExplicit<'a, R, S, A>>,
			FirstLayer,
			RunExplicit<'a, R, S, A>,
		> for WriterPreHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a,
		W: 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RunExplicit<'a, R, S, A>,
		>): Member<
				Coyoneda<'a, WriterBrand<W>, RunExplicit<'a, R, S, A>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									RunExplicit<'a, R, S, A>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			Box<FreeExplicit<'a, NodeBrand<R, S>, A>>,
		>): Member<Coyoneda<'a, WriterBrand<W>, Box<FreeExplicit<'a, NodeBrand<R, S>, A>>>, Idx>,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			Box<FreeExplicit<'a, NodeBrand<R, S>, A>>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					Box<FreeExplicit<'a, NodeBrand<R, S>, A>>,
				>),
				EmbedIndices,
			>,
	{
		/// Rewrite selected-action `Tell`s and return the resumed program.
		#[document_signature]
		#[document_parameters(
			"The Writer censor layer carrying the selected action.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns("The `RunExplicit` program with selected-action Writer logs censored.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		#[inline]
		fn dispatch_scoped_head(
			&self,
			layer: BoxWriterCensor<'a, BoxBrand, W, RunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, A>>,
		) -> RunExplicit<'a, R, S, A> {
			match layer {
				BoxWriterCensor::Censor {
					censor,
					action,
				} => action(())
					.interpose_with_rewriter::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices>(
						BoxWriterPreRewriter {
							censor,
						},
					),
			}
		}
	}

	/// Carrier-aware Writer pre-censor dispatch for `RcRunExplicitBoundary`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer pre-handler receiver.")]
	impl<'a, R, S, Action, Final, K, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			WriterCensor<'a, RcBrand, W, RcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RcRunExplicit<'a, R, S, Final>,
			RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for WriterPreHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Final: 'a,
		K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a,
		W: Clone + 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcRunExplicit<'a, R, S, Action>,
		>): Member<
				RcCoyoneda<'a, WriterBrand<W>, RcRunExplicit<'a, R, S, Action>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									RcRunExplicit<'a, R, S, Action>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Member<RcCoyoneda<'a, WriterBrand<W>, RcFreeExplicit<'a, NodeBrand<R, S>, Action>>, Idx>,
		Apply!(<WriterBrand<W> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Clone,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
				>),
				EmbedIndices,
			>,
	{
		/// Rewrite selected-action `Tell`s before resuming the outer continuation.
		#[document_signature]
		#[document_parameters(
			"The Writer censor layer carrying the selected action.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the Writer boundary.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: WriterCensor<'a, RcBrand, W, RcRunExplicit<'a, R, S, Action>>,
			continuation: ScopedContinuation<
				RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
		) -> RcRunExplicit<'a, R, S, Final> {
			match layer {
				WriterCensor::Censor {
					censor,
					action,
				} => continuation.resume_rc_with_supplied_action(fo_handlers, move || {
					action(())
						.interpose_with_rewriter::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices>(
							RcWriterPreRewriter {
								censor,
							},
						)
				}),
			}
		}
	}

	/// Ordinary scoped Writer pre-censor dispatch for `RcRunExplicit`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The program result type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer pre-handler receiver.")]
	impl<'a, R, S, A, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedHandler<
			'a,
			WriterCensor<'a, RcBrand, W, RcRunExplicit<'a, R, S, A>>,
			FirstLayer,
			RcRunExplicit<'a, R, S, A>,
		> for WriterPreHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'a,
		W: Clone + 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcRunExplicit<'a, R, S, A>,
		>): Member<
				RcCoyoneda<'a, WriterBrand<W>, RcRunExplicit<'a, R, S, A>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									RcRunExplicit<'a, R, S, A>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): Member<RcCoyoneda<'a, WriterBrand<W>, RcFreeExplicit<'a, NodeBrand<R, S>, A>>, Idx>,
		Apply!(<WriterBrand<W> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): Clone,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<R, S>, A>,
				>),
				EmbedIndices,
			>,
	{
		/// Rewrite selected-action `Tell`s and return the resumed program.
		#[document_signature]
		#[document_parameters(
			"The Writer censor layer carrying the selected action.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns(
			"The `RcRunExplicit` program with selected-action Writer logs censored."
		)]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		#[inline]
		fn dispatch_scoped_head(
			&self,
			layer: WriterCensor<'a, RcBrand, W, RcRunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, A>>,
		) -> RcRunExplicit<'a, R, S, A> {
			match layer {
				WriterCensor::Censor {
					censor,
					action,
				} => action(())
					.interpose_with_rewriter::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices>(
						RcWriterPreRewriter {
							censor,
						},
					),
			}
		}
	}

	/// Carrier-aware Writer pre-censor dispatch for `ArcRunExplicitBoundary`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer pre-handler receiver.")]
	impl<'a, R, S, Action, Final, K, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			SendWriterCensor<'a, ArcBrand, W, ArcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			ArcRunExplicit<'a, R, S, Final>,
			ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for WriterPreHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Action: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Action) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
		W: Clone + Send + Sync + 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		ArcFreeExplicit<'a, NodeBrand<R, S>, Action>: Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone + Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Send + Sync,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, Action>,
		>): Send
			+ Sync
			+ Member<
				ArcCoyoneda<'a, WriterBrand<W>, ArcRunExplicit<'a, R, S, Action>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									ArcRunExplicit<'a, R, S, Action>,
								>
							),
			>,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, Action>,
		>): Send + Sync,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, Action>,
		>): Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Member<ArcCoyoneda<'a, WriterBrand<W>, ArcFreeExplicit<'a, NodeBrand<R, S>, Action>>, Idx>,
		Apply!(<WriterBrand<W> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Clone + Send + Sync,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
				>),
				EmbedIndices,
			>,
	{
		/// Rewrite selected-action `Tell`s before resuming the outer continuation.
		#[document_signature]
		#[document_parameters(
			"The Writer censor layer carrying the selected action.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the Writer boundary.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: SendWriterCensor<'a, ArcBrand, W, ArcRunExplicit<'a, R, S, Action>>,
			continuation: ScopedContinuation<
				ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
		) -> ArcRunExplicit<'a, R, S, Final> {
			match layer {
				SendWriterCensor::Censor {
					censor,
					action,
				} => continuation.resume_arc_with_supplied_action(fo_handlers, move || {
					action(())
						.interpose_with_rewriter::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices>(
							ArcWriterPreRewriter {
								censor,
							},
						)
				}),
			}
		}
	}

	/// Ordinary scoped Writer pre-censor dispatch for `ArcRunExplicit`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The program result type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer pre-handler receiver.")]
	impl<'a, R, S, A, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedHandler<
			'a,
			SendWriterCensor<'a, ArcBrand, W, ArcRunExplicit<'a, R, S, A>>,
			FirstLayer,
			ArcRunExplicit<'a, R, S, A>,
		> for WriterPreHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'a,
		W: Clone + Send + Sync + 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		ArcFreeExplicit<'a, NodeBrand<R, S>, A>: Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): Clone + Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): Send + Sync,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, A>,
		>): Send
			+ Sync
			+ Member<
				ArcCoyoneda<'a, WriterBrand<W>, ArcRunExplicit<'a, R, S, A>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									ArcRunExplicit<'a, R, S, A>,
								>
							),
			>,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, A>,
		>): Send + Sync,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, A>,
		>): Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): Member<ArcCoyoneda<'a, WriterBrand<W>, ArcFreeExplicit<'a, NodeBrand<R, S>, A>>, Idx>,
		Apply!(<WriterBrand<W> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): Clone + Send + Sync,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
				>),
				EmbedIndices,
			>,
	{
		/// Rewrite selected-action `Tell`s and return the resumed program.
		#[document_signature]
		#[document_parameters(
			"The Writer censor layer carrying the selected action.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns(
			"The `ArcRunExplicit` program with selected-action Writer logs censored."
		)]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		#[inline]
		fn dispatch_scoped_head(
			&self,
			layer: SendWriterCensor<'a, ArcBrand, W, ArcRunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, A>>,
		) -> ArcRunExplicit<'a, R, S, A> {
			match layer {
				SendWriterCensor::Censor {
					censor,
					action,
				} => action(())
					.interpose_with_rewriter::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices>(
						ArcWriterPreRewriter {
							censor,
						},
					),
			}
		}
	}

	/// Carrier-aware Writer post-censor dispatch for `RunExplicitBoundary`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final program result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<'a, R, S, Action, Final, K, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			BoxWriterCensor<'a, BoxBrand, W, RunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RunExplicit<'a, R, S, Final>,
			RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: 'a,
		Final: 'a,
		K: Fn(Action) -> RunExplicit<'a, R, S, Final> + 'a,
		W: Monoid + Clone + 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RunExplicit<'a, R, S, Action>,
		>): Member<
				Coyoneda<'a, WriterBrand<W>, RunExplicit<'a, R, S, Action>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									RunExplicit<'a, R, S, Action>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, Action>):
			Member<Coyoneda<'a, WriterBrand<W>, Action>, Idx>,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			Box<FreeExplicit<'a, NodeBrand<R, S>, (Action, W)>>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					Box<FreeExplicit<'a, NodeBrand<R, S>, (Action, W)>>,
				>),
				EmbedIndices,
			>,
	{
		/// Accumulate selected-action `Tell`s, censor the aggregate, and resume the boundary.
		#[document_signature]
		#[document_parameters(
			"The Writer censor layer carrying the selected action.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the Writer boundary.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let censor = |log: String| format!("[{log}]");
		/// assert_eq!(censor("firstsecond".to_string()), "[firstsecond]");
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: BoxWriterCensor<'a, BoxBrand, W, RunExplicit<'a, R, S, Action>>,
			continuation: ScopedContinuation<
				RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
		) -> RunExplicit<'a, R, S, Final> {
			match layer {
				BoxWriterCensor::Censor {
					censor,
					action,
				} =>
					continuation.resume_explicit_with_supplied_action(fo_handlers, move || {
						action(())
							.accumulate_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
								BoxWriterAccumulator(PhantomData),
							)
							.bind(move |(value, log)| {
								RunExplicit::<R, S, Action>::lift::<WriterBrand<W>, Idx>(
									Writer::Tell((censor)(log), value, PhantomData),
								)
							})
					}),
			}
		}
	}

	/// Ordinary scoped Writer post-censor dispatch for `RunExplicit`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The program result type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<'a, R, S, A, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedHandler<
			'a,
			BoxWriterCensor<'a, BoxBrand, W, RunExplicit<'a, R, S, A>>,
			FirstLayer,
			RunExplicit<'a, R, S, A>,
		> for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a,
		W: Monoid + Clone + 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RunExplicit<'a, R, S, A>,
		>): Member<
				Coyoneda<'a, WriterBrand<W>, RunExplicit<'a, R, S, A>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									RunExplicit<'a, R, S, A>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>):
			Member<Coyoneda<'a, WriterBrand<W>, A>, Idx>,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			Box<FreeExplicit<'a, NodeBrand<R, S>, (A, W)>>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					Box<FreeExplicit<'a, NodeBrand<R, S>, (A, W)>>,
				>),
				EmbedIndices,
			>,
	{
		/// Accumulate selected-action `Tell`s and return the resumed program.
		#[document_signature]
		#[document_parameters(
			"The Writer censor layer carrying the selected action.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns(
			"The `RunExplicit` program with selected-action Writer logs post-censored."
		)]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let censor = |log: String| format!("[{log}]");
		/// assert_eq!(censor("firstsecond".to_string()), "[firstsecond]");
		/// ```
		#[inline]
		fn dispatch_scoped_head(
			&self,
			layer: BoxWriterCensor<'a, BoxBrand, W, RunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, A>>,
		) -> RunExplicit<'a, R, S, A> {
			match layer {
				BoxWriterCensor::Censor {
					censor,
					action,
				} => action(())
					.accumulate_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
						BoxWriterAccumulator(PhantomData),
					)
					.bind(move |(value, log)| {
						RunExplicit::<R, S, A>::lift::<WriterBrand<W>, Idx>(Writer::Tell(
							(censor)(log),
							value,
							PhantomData,
						))
					}),
			}
		}
	}

	/// Carrier-aware Writer post-censor dispatch for `RcRunExplicitBoundary`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<'a, R, S, Action, Final, K, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			WriterCensor<'a, RcBrand, W, RcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RcRunExplicit<'a, R, S, Final>,
			RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Final: 'a,
		K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a,
		W: Monoid + Clone + 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcRunExplicit<'a, R, S, Action>,
		>): Member<
				RcCoyoneda<'a, WriterBrand<W>, RcRunExplicit<'a, R, S, Action>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									RcRunExplicit<'a, R, S, Action>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, Action>):
			Member<RcCoyoneda<'a, WriterBrand<W>, Action>, Idx>,
		Apply!(<WriterBrand<W> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, Action>): Clone,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
				>),
				EmbedIndices,
			>,
	{
		/// Accumulate selected-action `Tell`s, censor the aggregate, and resume the boundary.
		#[document_signature]
		#[document_parameters(
			"The Writer censor layer carrying the selected action.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the Writer boundary.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let censor = |log: String| format!("[{log}]");
		/// assert_eq!(censor("firstsecond".to_string()), "[firstsecond]");
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: WriterCensor<'a, RcBrand, W, RcRunExplicit<'a, R, S, Action>>,
			continuation: ScopedContinuation<
				RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
		) -> RcRunExplicit<'a, R, S, Final> {
			match layer {
				WriterCensor::Censor {
					censor,
					action,
				} => continuation.resume_rc_with_supplied_action(fo_handlers, move || {
					action(())
						.accumulate_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
							RcWriterAccumulator(PhantomData),
						)
						.bind(move |(value, log)| {
							RcRunExplicit::<R, S, Action>::lift::<WriterBrand<W>, Idx>(
								Writer::Tell((censor)(log), value, PhantomData),
							)
						})
				}),
			}
		}
	}

	/// Ordinary scoped Writer post-censor dispatch for `RcRunExplicit`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The program result type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<'a, R, S, A, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedHandler<
			'a,
			WriterCensor<'a, RcBrand, W, RcRunExplicit<'a, R, S, A>>,
			FirstLayer,
			RcRunExplicit<'a, R, S, A>,
		> for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'a,
		W: Monoid + Clone + 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (A, W)>,
		>): Clone,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (A, W)>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (A, W)>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcRunExplicit<'a, R, S, A>,
		>): Member<
				RcCoyoneda<'a, WriterBrand<W>, RcRunExplicit<'a, R, S, A>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									RcRunExplicit<'a, R, S, A>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>):
			Member<RcCoyoneda<'a, WriterBrand<W>, A>, Idx>,
		Apply!(<WriterBrand<W> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>): Clone,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (A, W)>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<R, S>, (A, W)>,
				>),
				EmbedIndices,
			>,
	{
		/// Accumulate selected-action `Tell`s and return the resumed program.
		#[document_signature]
		#[document_parameters(
			"The Writer censor layer carrying the selected action.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns(
			"The `RcRunExplicit` program with selected-action Writer logs post-censored."
		)]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let censor = |log: String| format!("[{log}]");
		/// assert_eq!(censor("firstsecond".to_string()), "[firstsecond]");
		/// ```
		#[inline]
		fn dispatch_scoped_head(
			&self,
			layer: WriterCensor<'a, RcBrand, W, RcRunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, A>>,
		) -> RcRunExplicit<'a, R, S, A> {
			match layer {
				WriterCensor::Censor {
					censor,
					action,
				} => action(())
					.accumulate_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
						RcWriterAccumulator(PhantomData),
					)
					.bind(move |(value, log)| {
						RcRunExplicit::<R, S, A>::lift::<WriterBrand<W>, Idx>(Writer::Tell(
							(censor)(log),
							value,
							PhantomData,
						))
					}),
			}
		}
	}

	/// Carrier-aware Writer post-censor dispatch for `ArcRunExplicitBoundary`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<'a, R, S, Action, Final, K, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			SendWriterCensor<'a, ArcBrand, W, ArcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			ArcRunExplicit<'a, R, S, Final>,
			ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Action: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Action) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
		W: Monoid + Clone + Send + Sync + 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		ArcFreeExplicit<'a, NodeBrand<R, S>, Action>: Send + Sync,
		ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>: Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Clone + Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Send + Sync,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone + Send + Sync,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone + Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, Action>,
		>): Send
			+ Sync
			+ Member<
				ArcCoyoneda<'a, WriterBrand<W>, ArcRunExplicit<'a, R, S, Action>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									ArcRunExplicit<'a, R, S, Action>,
								>
							),
			>,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, Action>,
		>): Send + Sync,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, Action>,
		>): Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, Action>):
			Member<ArcCoyoneda<'a, WriterBrand<W>, Action>, Idx>,
		Apply!(<WriterBrand<W> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, Action>):
			Clone + Send + Sync,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
				>),
				EmbedIndices,
			>,
	{
		/// Accumulate selected-action `Tell`s, censor the aggregate, and resume the boundary.
		#[document_signature]
		#[document_parameters(
			"The Writer censor layer carrying the selected action.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the Writer boundary.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let censor = |log: String| format!("[{log}]");
		/// assert_eq!(censor("firstsecond".to_string()), "[firstsecond]");
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: SendWriterCensor<'a, ArcBrand, W, ArcRunExplicit<'a, R, S, Action>>,
			continuation: ScopedContinuation<
				ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
		) -> ArcRunExplicit<'a, R, S, Final> {
			match layer {
				SendWriterCensor::Censor {
					censor,
					action,
				} => continuation.resume_arc_with_supplied_action(fo_handlers, move || {
					action(())
						.accumulate_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
							ArcWriterAccumulator(PhantomData),
						)
						.bind(move |(value, log)| {
							ArcRunExplicit::<R, S, Action>::lift::<WriterBrand<W>, Idx>(
								Writer::Tell((censor)(log), value, PhantomData),
							)
						})
				}),
			}
		}
	}

	/// Ordinary scoped Writer post-censor dispatch for `ArcRunExplicit`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The program result type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<'a, R, S, A, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedHandler<
			'a,
			SendWriterCensor<'a, ArcBrand, W, ArcRunExplicit<'a, R, S, A>>,
			FirstLayer,
			ArcRunExplicit<'a, R, S, A>,
		> for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'a,
		W: Monoid + Clone + Send + Sync + 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		ArcFreeExplicit<'a, NodeBrand<R, S>, A>: Send + Sync,
		ArcFreeExplicit<'a, NodeBrand<R, S>, (A, W)>: Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): Clone + Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): Send + Sync,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
		>): Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (A, W)>,
		>): Clone + Send + Sync,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (A, W)>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (A, W)>,
		>): Clone + Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, A>,
		>): Send
			+ Sync
			+ Member<
				ArcCoyoneda<'a, WriterBrand<W>, ArcRunExplicit<'a, R, S, A>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									ArcRunExplicit<'a, R, S, A>,
								>
							),
			>,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, A>,
		>): Send + Sync,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, A>,
		>): Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>):
			Member<ArcCoyoneda<'a, WriterBrand<W>, A>, Idx>,
		Apply!(<WriterBrand<W> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>):
			Clone + Send + Sync,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (A, W)>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<R, S>, (A, W)>,
				>),
				EmbedIndices,
			>,
	{
		/// Accumulate selected-action `Tell`s and return the resumed program.
		#[document_signature]
		#[document_parameters(
			"The Writer censor layer carrying the selected action.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns(
			"The `ArcRunExplicit` program with selected-action Writer logs post-censored."
		)]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let censor = |log: String| format!("[{log}]");
		/// assert_eq!(censor("firstsecond".to_string()), "[firstsecond]");
		/// ```
		#[inline]
		fn dispatch_scoped_head(
			&self,
			layer: SendWriterCensor<'a, ArcBrand, W, ArcRunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, A>>,
		) -> ArcRunExplicit<'a, R, S, A> {
			match layer {
				SendWriterCensor::Censor {
					censor,
					action,
				} => action(())
					.accumulate_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
						ArcWriterAccumulator(PhantomData),
					)
					.bind(move |(value, log)| {
						ArcRunExplicit::<R, S, A>::lift::<WriterBrand<W>, Idx>(Writer::Tell(
							(censor)(log),
							value,
							PhantomData,
						))
					}),
			}
		}
	}

	/// Carrier-aware Writer listen dispatch for `RunExplicitBoundary`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final program result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<'a, R, S, Action, Final, K, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			BoxWriterListen<'a, BoxBrand, W, Action, RunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RunExplicit<'a, R, S, Final>,
			RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, (Action, W)>,
		> for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: 'a,
		Final: 'a,
		K: Fn((Action, W)) -> RunExplicit<'a, R, S, Final> + 'a,
		W: Monoid + Clone + 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RunExplicit<'a, R, S, Action>,
		>): Member<
				Coyoneda<'a, WriterBrand<W>, RunExplicit<'a, R, S, Action>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									RunExplicit<'a, R, S, Action>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			Box<FreeExplicit<'a, NodeBrand<R, S>, (Action, W)>>,
		>): Member<
				Coyoneda<'a, WriterBrand<W>, Box<FreeExplicit<'a, NodeBrand<R, S>, (Action, W)>>>,
				Idx,
			>,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			Box<FreeExplicit<'a, NodeBrand<R, S>, (Action, W)>>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					Box<FreeExplicit<'a, NodeBrand<R, S>, (Action, W)>>,
				>),
				EmbedIndices,
			>,
	{
		/// Observe selected-action `Tell`s, preserve them, and resume the boundary.
		#[document_signature]
		#[document_parameters(
			"The Writer listen layer carrying the selected action.",
			"The wrapper-owned continuation carrier for the listen result.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the Writer boundary.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let selected_value = 7;
		/// let observed_log = "inner".to_string();
		/// let operation_result = (selected_value, observed_log);
		/// assert_eq!(operation_result, (7, "inner".to_string()));
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: BoxWriterListen<'a, BoxBrand, W, Action, RunExplicit<'a, R, S, Action>>,
			continuation: ScopedContinuation<
				RunExplicitActionSuppliedScopedContinuation<
					'a,
					R,
					S,
					Action,
					Final,
					K,
					(Action, W),
				>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
		) -> RunExplicit<'a, R, S, Final> {
			match layer {
				BoxWriterListen::Listen {
					action,
					result: _,
				} => continuation.resume_explicit_with_supplied_action(fo_handlers, move || {
					action(())
						.accumulate_preserving_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
							BoxWriterAccumulator(PhantomData),
						)
				}),
			}
		}
	}

	/// Carrier-aware Writer listen dispatch for `RcRunExplicitBoundary`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<'a, R, S, Action, Final, K, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			WriterListen<'a, RcBrand, W, Action, RcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RcRunExplicit<'a, R, S, Final>,
			RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, (Action, W)>,
		> for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Final: 'a,
		K: Fn((Action, W)) -> RcRunExplicit<'a, R, S, Final> + 'a,
		W: Monoid + Clone + 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone
			+ Member<
				RcCoyoneda<'a, WriterBrand<W>, RcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>>,
				Idx,
			>,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcRunExplicit<'a, R, S, Action>,
		>): Member<
				RcCoyoneda<'a, WriterBrand<W>, RcRunExplicit<'a, R, S, Action>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									RcRunExplicit<'a, R, S, Action>,
								>
							),
			>,
		Apply!(<WriterBrand<W> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
				>),
				EmbedIndices,
			>,
	{
		/// Observe selected-action `Tell`s, preserve them, and resume the boundary.
		#[document_signature]
		#[document_parameters(
			"The Writer listen layer carrying the selected action.",
			"The wrapper-owned continuation carrier for the listen result.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the Writer boundary.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let selected_value = 7;
		/// let observed_log = "inner".to_string();
		/// let operation_result = (selected_value, observed_log);
		/// assert_eq!(operation_result, (7, "inner".to_string()));
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: WriterListen<'a, RcBrand, W, Action, RcRunExplicit<'a, R, S, Action>>,
			continuation: ScopedContinuation<
				RcRunExplicitActionSuppliedScopedContinuation<
					'a,
					R,
					S,
					Action,
					Final,
					K,
					(Action, W),
				>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
		) -> RcRunExplicit<'a, R, S, Final> {
			match layer {
				WriterListen::Listen {
					action,
					result: _,
				} => continuation.resume_rc_with_supplied_action(fo_handlers, move || {
					action(())
						.accumulate_preserving_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
							RcWriterAccumulator(PhantomData),
						)
				}),
			}
		}
	}

	/// Carrier-aware Writer listen dispatch for `ArcRunExplicitBoundary`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The Writer log type.",
		"The first-order row index witnessing the Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<'a, R, S, Action, Final, K, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			SendWriterListen<'a, ArcBrand, W, Action, ArcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			ArcRunExplicit<'a, R, S, Final>,
			ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, (Action, W)>,
		> for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Action: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn((Action, W)) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
		W: Monoid + Clone + Send + Sync + 'static,
		FirstLayer: 'a,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		ArcFreeExplicit<'a, NodeBrand<R, S>, Action>: Send + Sync,
		ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>: Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Clone + Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Send + Sync,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone
			+ Send
			+ Sync
			+ Member<
				ArcCoyoneda<'a, WriterBrand<W>, ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>>,
				Idx,
			>,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone + Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, Action>,
		>): Send
			+ Sync
			+ Member<
				ArcCoyoneda<'a, WriterBrand<W>, ArcRunExplicit<'a, R, S, Action>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
									'a,
									ArcRunExplicit<'a, R, S, Action>,
								>
							),
			>,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, Action>,
		>): Send + Sync,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, S, Action>,
		>): Send + Sync,
		Apply!(<WriterBrand<W> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): Clone + Send + Sync,
		Apply!(<RMinusWriter as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<R, S>, (Action, W)>,
				>),
				EmbedIndices,
			>,
	{
		/// Observe selected-action `Tell`s, preserve them, and resume the boundary.
		#[document_signature]
		#[document_parameters(
			"The Writer listen layer carrying the selected action.",
			"The wrapper-owned continuation carrier for the listen result.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the Writer boundary.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped Writer carrier or protocol hook receives interpreter-built continuation state; examples document Writer accumulation semantics instead of constructing private protocol inputs directly."
		)]
		///
		/// ```
		/// let selected_value = 7;
		/// let observed_log = "inner".to_string();
		/// let operation_result = (selected_value, observed_log);
		/// assert_eq!(operation_result, (7, "inner".to_string()));
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: SendWriterListen<'a, ArcBrand, W, Action, ArcRunExplicit<'a, R, S, Action>>,
			continuation: ScopedContinuation<
				ArcRunExplicitActionSuppliedScopedContinuation<
					'a,
					R,
					S,
					Action,
					Final,
					K,
					(Action, W),
				>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
		) -> ArcRunExplicit<'a, R, S, Final> {
			match layer {
				SendWriterListen::Listen {
					action,
					result: _,
				} => continuation.resume_arc_with_supplied_action(fo_handlers, move || {
					action(())
						.accumulate_preserving_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
							ArcWriterAccumulator(PhantomData),
						)
				}),
			}
		}
	}
}
