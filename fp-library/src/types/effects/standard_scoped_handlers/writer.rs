#[allow(
	unused_imports,
	reason = "Each scoped-handler child module consumes a different subset of the shared parent prelude."
)]
use super::prelude::*;

mod carrier;

mod raw_accumulators;

mod raw_rewriters;

#[fp_macros::document_module]
mod inner {
	use super::{
		raw_accumulators::*,
		raw_rewriters::*,
		*,
	};

	/// Standard Writer handler that transforms each selected action log before accumulation.
	///
	/// `WriterPreHandler` carries only type-level row witnesses. The handler
	/// semantics are intentionally named: it applies `censor` before the
	/// selected action's `Tell` values reach the surrounding Writer handler.
	#[derive(Clone, Copy, Debug, Default)]
	#[expect(
		clippy::type_complexity,
		reason = "The fn marker carries row-witness type parameters without making auto-traits depend on them."
	)]
	pub struct WriterPreHandler<Idx, RMinusWriter, EmbedIndices>(
		PhantomData<fn() -> (Idx, RMinusWriter, EmbedIndices)>,
	);

	/// Standard Writer handler that accumulates a selected action log before transforming it.
	///
	/// `WriterPostHandler` carries only type-level row witnesses. The handler
	/// semantics are intentionally named: it observes the selected action's
	/// `Tell` values, accumulates them, and then applies `censor` to the
	/// aggregate before re-emitting it.
	#[derive(Clone, Copy, Debug, Default)]
	#[expect(
		clippy::type_complexity,
		reason = "The fn marker carries row-witness type parameters without making auto-traits depend on them."
	)]
	pub struct WriterPostHandler<Idx, RMinusWriter, EmbedIndices>(
		PhantomData<fn() -> (Idx, RMinusWriter, EmbedIndices)>,
	);

	/// Constructs a [`WriterPreHandler`] without naming its private field.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	types::effects::standard_scoped_handlers::{
	/// 		WriterPreHandler,
	/// 		writer_pre_handler,
	/// 	},
	/// };
	///
	/// fn accepts_pre_handler<Idx, RMinusWriter, EmbedIndices>(
	/// 	_handler: WriterPreHandler<Idx, RMinusWriter, EmbedIndices>
	/// ) -> &'static str {
	/// 	"writer pre handler"
	/// }
	///
	/// let handler = writer_pre_handler::<(), CNilBrand, ()>();
	/// assert_eq!(accepts_pre_handler(handler), "writer pre handler");
	/// ```
	pub const fn writer_pre_handler<Idx, RMinusWriter, EmbedIndices>()
	-> WriterPreHandler<Idx, RMinusWriter, EmbedIndices> {
		WriterPreHandler(PhantomData)
	}

	/// Constructs a [`WriterPostHandler`] without naming its private field.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	types::effects::standard_scoped_handlers::{
	/// 		WriterPostHandler,
	/// 		writer_post_handler,
	/// 	},
	/// };
	///
	/// fn accepts_post_handler<Idx, RMinusWriter, EmbedIndices>(
	/// 	_handler: WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	/// ) -> &'static str {
	/// 	"writer post handler"
	/// }
	///
	/// let handler = writer_post_handler::<(), CNilBrand, ()>();
	/// assert_eq!(accepts_post_handler(handler), "writer post handler");
	/// ```
	pub const fn writer_post_handler<Idx, RMinusWriter, EmbedIndices>()
	-> WriterPostHandler<Idx, RMinusWriter, EmbedIndices> {
		WriterPostHandler(PhantomData)
	}

	/// Raw scoped dispatch implementation for default `Run` Writer pre-censor.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The Writer log type.",
		"The row index witnessing the target Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer pre-handler receiver.")]
	impl<R, S, A, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchRunRawScopedHandler<R, S, A, BoxWriterCensorBrand<BoxBrand, W>, FirstLayer>
		for WriterPreHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
		W: 'static,
		FirstLayer: 'static,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			Run<R, S, crate::types::free::TypeErasedValue>,
		>): Member<
				Coyoneda<'static, WriterBrand<W>, Run<R, S, crate::types::free::TypeErasedValue>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
									'static,
									Run<R, S, crate::types::free::TypeErasedValue>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			Free<NodeBrand<R, S>, crate::types::free::TypeErasedValue>,
		>): Member<
				Coyoneda<
					'static,
					WriterBrand<W>,
					Free<NodeBrand<R, S>, crate::types::free::TypeErasedValue>,
				>,
				Idx,
			>,
		Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			Free<NodeBrand<R, S>, crate::types::free::TypeErasedValue>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Free<NodeBrand<R, S>, crate::types::free::TypeErasedValue>,
				>),
				EmbedIndices,
			>,
	{
		/// Transform every selected-action `Tell` before the action resumes.
		#[document_signature]
		#[document_parameters(
			"The raw Writer censor layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		fn dispatch_run_raw_scoped_head(
			&self,
			layer: BoxWriterCensor<'static, BoxBrand, W, RawRunFree<R, S>>,
			continuations: RunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
		) -> Run<R, S, A> {
			match layer {
				BoxWriterCensor::Censor {
					censor,
					action,
				} => {
					let rewritten = Run::<R, S, crate::types::free::TypeErasedValue>::from_free(
						action(()).erase_type(),
					)
					.interpose_with_rewriter::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices>(
						BoxWriterPreRewriter {
							censor,
						},
					);
					Run::from_free(Free::continue_from_reboxed_erased(
						rewritten.into_free(),
						continuations,
					))
				}
			}
		}
	}

	/// Raw scoped dispatch implementation for `RcRun` Writer pre-censor.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The Writer log type.",
		"The row index witnessing the target Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer pre-handler receiver.")]
	impl<R, S, A, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchRcRunRawScopedHandler<R, S, A, WriterCensorBrand<RcBrand, W>, FirstLayer>
		for WriterPreHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		W: Clone + 'static,
		FirstLayer: 'static,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcRun<R, S, RcTypeErasedValue>,
		>): Member<
				RcCoyoneda<'static, WriterBrand<W>, RcRun<R, S, RcTypeErasedValue>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
									'static,
									RcRun<R, S, RcTypeErasedValue>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Member<
				RcCoyoneda<'static, WriterBrand<W>, RcFree<NodeBrand<R, S>, RcTypeErasedValue>>,
				Idx,
			>,
		Apply!(<WriterBrand<W> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
		Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
				>),
				EmbedIndices,
			>,
	{
		/// Transform every selected-action `Tell` before the action resumes.
		#[document_signature]
		#[document_parameters(
			"The raw Writer censor layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		fn dispatch_rc_run_raw_scoped_head(
			&self,
			layer: WriterCensor<'static, RcBrand, W, RawRcRunFree<R, S>>,
			continuations: RcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
		) -> RcRun<R, S, A> {
			match layer {
				WriterCensor::Censor {
					censor,
					action,
				} => {
					let rewritten =
						RcRun::<R, S, RcTypeErasedValue>::from_rc_free(action(()).erase_type())
							.interpose_with_rewriter::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices>(
							RcWriterPreRewriter {
								censor,
							},
						);
					RcRun::from_rc_free(RcFree::continue_from_reboxed_erased(
						rewritten.into_rc_free(),
						continuations,
					))
				}
			}
		}
	}

	/// Raw scoped dispatch implementation for `ArcRun` Writer pre-censor.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The Writer log type.",
		"The row index witnessing the target Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer pre-handler receiver.")]
	impl<R, S, A, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchArcRunRawScopedHandler<R, S, A, SendWriterCensorBrand<ArcBrand, W>, FirstLayer>
		for WriterPreHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		W: Clone + Send + Sync + 'static,
		FirstLayer: 'static,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcRun<R, S, ArcTypeErasedValue>,
		>): Member<
				ArcCoyoneda<'static, WriterBrand<W>, ArcRun<R, S, ArcTypeErasedValue>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
									'static,
									ArcRun<R, S, ArcTypeErasedValue>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Member<
				ArcCoyoneda<'static, WriterBrand<W>, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>,
				Idx,
			>,
		Apply!(<WriterBrand<W> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone + Send + Sync,
		ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>: Send + Sync,
		Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
				>),
				EmbedIndices,
			>,
	{
		/// Transform every selected-action `Tell` before the action resumes.
		#[document_signature]
		#[document_parameters(
			"The raw Writer censor layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		fn dispatch_arc_run_raw_scoped_head(
			&self,
			layer: SendWriterCensor<'static, ArcBrand, W, RawArcRunFree<R, S>>,
			continuations: ArcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
		) -> ArcRun<R, S, A> {
			match layer {
				SendWriterCensor::Censor {
					censor,
					action,
				} => {
					let rewritten =
						ArcRun::<R, S, ArcTypeErasedValue>::from_arc_free(action(()).erase_type())
							.interpose_with_rewriter::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices>(
							ArcWriterPreRewriter {
								censor,
							},
						);
					ArcRun::from_arc_free(ArcFree::continue_from_reboxed_erased(
						rewritten.into_arc_free(),
						continuations,
					))
				}
			}
		}
	}

	/// Raw scoped dispatch implementation for default `Run` Writer post-censor.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The Writer log type.",
		"The row index witnessing the target Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<R, S, A, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchRunRawScopedHandler<R, S, A, BoxWriterCensorBrand<BoxBrand, W>, FirstLayer>
		for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
		W: Monoid + Clone + 'static,
		FirstLayer: 'static,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			Run<R, S, crate::types::free::TypeErasedValue>,
		>): Member<
				Coyoneda<'static, WriterBrand<W>, Run<R, S, crate::types::free::TypeErasedValue>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
									'static,
									Run<R, S, crate::types::free::TypeErasedValue>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			crate::types::free::TypeErasedValue,
		>): Member<Coyoneda<'static, WriterBrand<W>, crate::types::free::TypeErasedValue>, Idx>,
		Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			Free<NodeBrand<R, S>, (crate::types::free::TypeErasedValue, W)>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Free<NodeBrand<R, S>, (crate::types::free::TypeErasedValue, W)>,
				>),
				EmbedIndices,
			>,
	{
		/// Accumulate selected-action `Tell`s, censor the aggregate, and re-emit it before resuming.
		#[document_signature]
		#[document_parameters(
			"The raw Writer censor layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		fn dispatch_run_raw_scoped_head(
			&self,
			layer: BoxWriterCensor<'static, BoxBrand, W, RawRunFree<R, S>>,
			continuations: RunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
		) -> Run<R, S, A> {
			match layer {
				BoxWriterCensor::Censor {
					censor,
					action,
				} => {
					let accumulated = Run::<R, S, crate::types::free::TypeErasedValue>::from_free(
						action(()).erase_type(),
					)
					.accumulate_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
						BoxWriterAccumulator(PhantomData),
					);
					let post_applied: Run<R, S, crate::types::free::TypeErasedValue> = accumulated
						.bind(move |(value, log)| {
							Run::<R, S, crate::types::free::TypeErasedValue>::lift::<
								WriterBrand<W>,
								Idx,
							>(Writer::Tell((censor)(log), value, PhantomData))
						});
					Run::from_free(Free::continue_from_reboxed_erased(
						post_applied.into_free(),
						continuations,
					))
				}
			}
		}
	}

	/// Raw scoped dispatch implementation for `RcRun` Writer post-censor.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The Writer log type.",
		"The row index witnessing the target Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<R, S, A, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchRcRunRawScopedHandler<R, S, A, WriterCensorBrand<RcBrand, W>, FirstLayer>
		for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		W: Monoid + Clone + 'static,
		FirstLayer: 'static,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcRun<R, S, RcTypeErasedValue>,
		>): Member<
				RcCoyoneda<'static, WriterBrand<W>, RcRun<R, S, RcTypeErasedValue>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
									'static,
									RcRun<R, S, RcTypeErasedValue>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcTypeErasedValue,
		>): Member<RcCoyoneda<'static, WriterBrand<W>, RcTypeErasedValue>, Idx>,
		Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, (RcTypeErasedValue, W)>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RcFree<NodeBrand<R, S>, (RcTypeErasedValue, W)>,
				>),
				EmbedIndices,
			>,
	{
		/// Accumulate selected-action `Tell`s, censor the aggregate, and re-emit it before resuming.
		#[document_signature]
		#[document_parameters(
			"The raw Writer censor layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		fn dispatch_rc_run_raw_scoped_head(
			&self,
			layer: WriterCensor<'static, RcBrand, W, RawRcRunFree<R, S>>,
			continuations: RcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
		) -> RcRun<R, S, A> {
			match layer {
				WriterCensor::Censor {
					censor,
					action,
				} => {
					let accumulated =
						RcRun::<R, S, RcTypeErasedValue>::from_rc_free(action(()).erase_type())
							.accumulate_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
							RcWriterAccumulator(PhantomData),
						);
					let post_applied: RcRun<R, S, RcTypeErasedValue> =
						accumulated.bind(move |(value, log)| {
							RcRun::<R, S, RcTypeErasedValue>::lift::<WriterBrand<W>, Idx>(
								Writer::Tell((censor)(log), value, PhantomData),
							)
						});
					RcRun::from_rc_free(RcFree::continue_from_reboxed_erased(
						post_applied.into_rc_free(),
						continuations,
					))
				}
			}
		}
	}

	/// Raw scoped dispatch implementation for `ArcRun` Writer post-censor.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The Writer log type.",
		"The row index witnessing the target Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<R, S, A, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchArcRunRawScopedHandler<R, S, A, SendWriterCensorBrand<ArcBrand, W>, FirstLayer>
		for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		W: Monoid + Clone + Send + Sync + 'static,
		FirstLayer: 'static,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone,
		ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>: Send + Sync,
		ArcFree<NodeBrand<R, S>, (ArcTypeErasedValue, W)>: Send + Sync,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcRun<R, S, ArcTypeErasedValue>,
		>): Member<
				ArcCoyoneda<'static, WriterBrand<W>, ArcRun<R, S, ArcTypeErasedValue>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
									'static,
									ArcRun<R, S, ArcTypeErasedValue>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcTypeErasedValue,
		>): Send + Sync + Member<ArcCoyoneda<'static, WriterBrand<W>, ArcTypeErasedValue>, Idx>,
		Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, (ArcTypeErasedValue, W)>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<R, S>, (ArcTypeErasedValue, W)>,
				>),
				EmbedIndices,
			>,
	{
		/// Accumulate selected-action `Tell`s, censor the aggregate, and re-emit it before resuming.
		#[document_signature]
		#[document_parameters(
			"The raw Writer censor layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		fn dispatch_arc_run_raw_scoped_head(
			&self,
			layer: SendWriterCensor<'static, ArcBrand, W, RawArcRunFree<R, S>>,
			continuations: ArcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
		) -> ArcRun<R, S, A> {
			match layer {
				SendWriterCensor::Censor {
					censor,
					action,
				} => {
					let accumulated =
						ArcRun::<R, S, ArcTypeErasedValue>::from_arc_free(action(()).erase_type())
							.accumulate_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
							ArcWriterAccumulator(PhantomData),
						);
					let post_applied: ArcRun<R, S, ArcTypeErasedValue> =
						accumulated.bind(move |(value, log)| {
							ArcRun::<R, S, ArcTypeErasedValue>::lift::<WriterBrand<W>, Idx>(
								Writer::Tell((censor)(log), value, PhantomData),
							)
						});
					ArcRun::from_arc_free(ArcFree::continue_from_reboxed_erased(
						post_applied.into_arc_free(),
						continuations,
					))
				}
			}
		}
	}

	/// Raw scoped dispatch implementation for default `Run` Writer listen.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type after outer continuations run.",
		"The selected action result type.",
		"The Writer log type.",
		"The row index witnessing the target Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<R, S, A, Action, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchRunRawScopedHandler<R, S, A, BoxWriterListenBrand<BoxBrand, W, Action>, FirstLayer>
		for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
		Action: 'static,
		W: Monoid + Clone + 'static,
		FirstLayer: 'static,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			Run<R, S, crate::types::free::TypeErasedValue>,
		>): Member<
				Coyoneda<'static, WriterBrand<W>, Run<R, S, crate::types::free::TypeErasedValue>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
									'static,
									Run<R, S, crate::types::free::TypeErasedValue>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			Free<NodeBrand<R, S>, (crate::types::free::TypeErasedValue, W)>,
		>): Member<
				Coyoneda<
					'static,
					WriterBrand<W>,
					Free<NodeBrand<R, S>, (crate::types::free::TypeErasedValue, W)>,
				>,
				Idx,
			>,
		Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			Free<NodeBrand<R, S>, (crate::types::free::TypeErasedValue, W)>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Free<NodeBrand<R, S>, (crate::types::free::TypeErasedValue, W)>,
				>),
				EmbedIndices,
			>,
	{
		/// Observe selected-action `Tell`s, preserve them, and resume with the observed log.
		#[document_signature]
		#[document_parameters(
			"The raw Writer listen layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// let action_value = 7;
		/// let observed_log = "inner".to_string();
		/// let emitted_log = observed_log.clone();
		/// let listen_result = (action_value, observed_log);
		/// assert_eq!(listen_result, (7, "inner".to_string()));
		/// assert_eq!(emitted_log, "inner");
		/// ```
		fn dispatch_run_raw_scoped_head(
			&self,
			layer: BoxWriterListen<'static, BoxBrand, W, Action, RawRunFree<R, S>>,
			continuations: RunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
		) -> Run<R, S, A> {
			match layer {
				BoxWriterListen::Listen {
					action,
					result: _,
				} => {
					let listened = Run::<R, S, crate::types::free::TypeErasedValue>::from_free(
						action(()).erase_type(),
					)
					.accumulate_preserving_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
						BoxWriterAccumulator(PhantomData),
					)
					.map(|(value, log)| {
						#[expect(clippy::expect_used, reason = "Type maintained by Writer listen")]
						let raw_value: crate::types::free::TypeErasedValue = *value
							.downcast()
							.expect("Type mismatch in Writer listen erased action wrapper");
						#[expect(clippy::expect_used, reason = "Type maintained by Writer listen")]
						let action_value: Action = *raw_value
							.downcast()
							.expect("Type mismatch in Writer listen selected action result");
						Box::new((action_value, log)) as crate::types::free::TypeErasedValue
					});
					Run::from_free(Free::continue_from_reboxed_erased(
						listened.into_free().erase_type(),
						continuations,
					))
				}
			}
		}
	}

	/// Raw scoped dispatch implementation for `RcRun` Writer listen.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type after outer continuations run.",
		"The selected action result type.",
		"The Writer log type.",
		"The row index witnessing the target Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<R, S, A, Action, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchRcRunRawScopedHandler<R, S, A, WriterListenBrand<RcBrand, W, Action>, FirstLayer>
		for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		Action: Clone + 'static,
		W: Monoid + Clone + 'static,
		FirstLayer: 'static,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcRun<R, S, RcTypeErasedValue>,
		>): Member<
				RcCoyoneda<'static, WriterBrand<W>, RcRun<R, S, RcTypeErasedValue>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
									'static,
									RcRun<R, S, RcTypeErasedValue>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, (RcTypeErasedValue, W)>,
		>): Member<
				RcCoyoneda<
					'static,
					WriterBrand<W>,
					RcFree<NodeBrand<R, S>, (RcTypeErasedValue, W)>,
				>,
				Idx,
			>,
		Apply!(<WriterBrand<W> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, (RcTypeErasedValue, W)>,
		>): Clone,
		Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, (RcTypeErasedValue, W)>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RcFree<NodeBrand<R, S>, (RcTypeErasedValue, W)>,
				>),
				EmbedIndices,
			>,
	{
		/// Observe selected-action `Tell`s, preserve them, and resume with the observed log.
		#[document_signature]
		#[document_parameters(
			"The raw Writer listen layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// let action_value = 7;
		/// let observed_log = "inner".to_string();
		/// let emitted_log = observed_log.clone();
		/// let listen_result = (action_value, observed_log);
		/// assert_eq!(listen_result, (7, "inner".to_string()));
		/// assert_eq!(emitted_log, "inner");
		/// ```
		fn dispatch_rc_run_raw_scoped_head(
			&self,
			layer: WriterListen<'static, RcBrand, W, Action, RawRcRunFree<R, S>>,
			continuations: RcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
		) -> RcRun<R, S, A> {
			match layer {
				WriterListen::Listen {
					action,
					result: _,
				} => {
					let listened = RcRun::<R, S, RcTypeErasedValue>::from_rc_free(
						action(()).erase_type(),
					)
					.accumulate_preserving_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
						RcWriterAccumulator(PhantomData),
					)
					.map(|(value, log)| {
						#[expect(clippy::expect_used, reason = "Type maintained by Writer listen")]
						let raw_value: Rc<RcTypeErasedValue> = value
							.downcast()
							.expect("Type mismatch in Writer listen erased action wrapper");
						let erased_value =
							Rc::try_unwrap(raw_value).unwrap_or_else(|shared| (*shared).clone());
						#[expect(clippy::expect_used, reason = "Type maintained by Writer listen")]
						let action_value: Rc<Action> = erased_value
							.downcast()
							.expect("Type mismatch in Writer listen selected action result");
						let action_value =
							Rc::try_unwrap(action_value).unwrap_or_else(|shared| (*shared).clone());
						Rc::new((action_value, log)) as RcTypeErasedValue
					});
					RcRun::from_rc_free(RcFree::continue_from_reboxed_erased(
						listened.into_rc_free().erase_type(),
						continuations,
					))
				}
			}
		}
	}

	/// Raw scoped dispatch implementation for `ArcRun` Writer listen.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type after outer continuations run.",
		"The selected action result type.",
		"The Writer log type.",
		"The row index witnessing the target Writer operation.",
		"The first-order row brand with the Writer operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Writer post-handler receiver.")]
	impl<R, S, A, Action, W, Idx, RMinusWriter, EmbedIndices, FirstLayer>
		DispatchArcRunRawScopedHandler<
			R,
			S,
			A,
			SendWriterListenBrand<ArcBrand, W, Action>,
			FirstLayer,
		> for WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		Action: Clone + Send + Sync + 'static,
		W: Monoid + Clone + Send + Sync + 'static,
		FirstLayer: 'static,
		RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone,
		ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>: Send + Sync,
		ArcFree<NodeBrand<R, S>, (ArcTypeErasedValue, W)>: Send + Sync,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcRun<R, S, ArcTypeErasedValue>,
		>): Member<
				ArcCoyoneda<'static, WriterBrand<W>, ArcRun<R, S, ArcTypeErasedValue>>,
				Idx,
				Remainder = Apply!(
								<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
									'static,
									ArcRun<R, S, ArcTypeErasedValue>,
								>
							),
			>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, (ArcTypeErasedValue, W)>,
		>): Member<
				ArcCoyoneda<
					'static,
					WriterBrand<W>,
					ArcFree<NodeBrand<R, S>, (ArcTypeErasedValue, W)>,
				>,
				Idx,
			>,
		Apply!(<WriterBrand<W> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, (ArcTypeErasedValue, W)>,
		>): Clone + Send + Sync,
		Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, (ArcTypeErasedValue, W)>,
		>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<R, S>, (ArcTypeErasedValue, W)>,
				>),
				EmbedIndices,
			>,
	{
		/// Observe selected-action `Tell`s, preserve them, and resume with the observed log.
		#[document_signature]
		#[document_parameters(
			"The raw Writer listen layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// let action_value = 7;
		/// let observed_log = "inner".to_string();
		/// let emitted_log = observed_log.clone();
		/// let listen_result = (action_value, observed_log);
		/// assert_eq!(listen_result, (7, "inner".to_string()));
		/// assert_eq!(emitted_log, "inner");
		/// ```
		fn dispatch_arc_run_raw_scoped_head(
			&self,
			layer: SendWriterListen<'static, ArcBrand, W, Action, RawArcRunFree<R, S>>,
			continuations: ArcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
		) -> ArcRun<R, S, A> {
			match layer {
				SendWriterListen::Listen {
					action,
					result: _,
				} => {
					let listened = ArcRun::<R, S, ArcTypeErasedValue>::from_arc_free(
						action(()).erase_type(),
					)
					.accumulate_preserving_with_first_order::<WriterBrand<W>, Idx, RMinusWriter, EmbedIndices, W>(
						ArcWriterAccumulator(PhantomData),
					)
					.map(|(value, log)| {
						#[expect(clippy::expect_used, reason = "Type maintained by Writer listen")]
						let raw_value: Arc<ArcTypeErasedValue> = value
							.downcast()
							.expect("Type mismatch in Writer listen erased action wrapper");
						let erased_value =
							Arc::try_unwrap(raw_value).unwrap_or_else(|shared| (*shared).clone());
						#[expect(clippy::expect_used, reason = "Type maintained by Writer listen")]
						let action_value: Arc<Action> = erased_value
							.downcast()
							.expect("Type mismatch in Writer listen selected action result");
						let action_value = Arc::try_unwrap(action_value)
							.unwrap_or_else(|shared| (*shared).clone());
						Arc::new((action_value, log)) as ArcTypeErasedValue
					});
					ArcRun::from_arc_free(ArcFree::continue_from_reboxed_erased(
						listened.into_arc_free().erase_type(),
						continuations,
					))
				}
			}
		}
	}
}

pub use inner::*;
