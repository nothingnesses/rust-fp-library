#[allow(
	unused_imports,
	reason = "Each scoped-dispatcher child module consumes a different subset of the shared parent prelude."
)]
use super::prelude::*;

mod carrier;

#[fp_macros::document_module]
mod inner {
	use super::*;

	/// Dispatcher for the standard `Span` scoped effect.
	///
	/// The dispatcher consumes the by-value tag and resumes the stored
	/// action program unchanged.
	#[derive(Clone, Copy, Debug, Default)]
	pub struct SpanHandler;

	/// Constructs a [`SpanHandler`].
	#[document_examples(skip_call_check)]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		BoxBrand,
	/// 		BoxSpanBrand,
	/// 		CNilBrand,
	/// 		CoproductBrand,
	/// 	},
	/// 	handlers,
	/// 	scoped_handlers,
	/// 	types::effects::{
	/// 		run::Run,
	/// 		standard_scoped_handlers::span_handler,
	/// 	},
	/// };
	///
	/// type FirstRow = CNilBrand;
	/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
	/// type Prog = Run<FirstRow, ScopedRow, i32>;
	///
	/// let program: Prog = Run::span::<&'static str, _>("request", Run::pure(42));
	///
	/// let result = program.handle(
	/// 	handlers! {},
	/// 	scoped_handlers! {
	/// 		BoxSpanBrand<BoxBrand, &'static str>: span_handler(),
	/// 	},
	/// );
	///
	/// assert_eq!(result, 42);
	/// ```
	pub const fn span_handler() -> SpanHandler {
		SpanHandler
	}

	/// Dispatch implementation for a standard span dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The span tag type."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, Tag>
		DispatchScopedHandler<
			'static,
			BoxSpan<'static, BoxBrand, Tag, Run<R, S, A>>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			Run<R, S, A>,
		> for SpanHandler
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
		Tag: 'static,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped operation layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxSpanBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: BoxSpan<'static, BoxBrand, Tag, Run<R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				Run<R, S, A>,
			>,
		) -> Run<R, S, A> {
			match layer {
				BoxSpan::Span {
					tag: _tag,
					action,
				} => action(()),
			}
		}
	}

	/// Dispatch implementation for a standard span dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The span tag type.",
		"The first first-order handler layer type."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, Tag, FirstLayer>
		DispatchRunRawScopedHandler<R, S, A, BoxSpanBrand<BoxBrand, Tag>, FirstLayer> for SpanHandler
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
		Tag: 'static,
		FirstLayer: 'static,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The raw scoped operation layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxSpanBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_run_raw_scoped_head(
			&self,
			layer: BoxSpan<'static, BoxBrand, Tag, RawRunFree<R, S>>,
			continuations: RunContinuations<R, S>,
			fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
		) -> Run<R, S, A> {
			match layer {
				BoxSpan::Span {
					tag,
					action,
				} => ScopedContinuation::new(RunScopedContinuation {
					action: action(()),
					continuations,
					result: PhantomData,
				})
				.resume_default_with_post_action(fo_handlers, move |action_value| {
					let _ = &tag;
					Free::<NodeBrand<R, S>, crate::types::free::TypeErasedValue>::from_erased_value(
						action_value,
					)
				}),
			}
		}
	}

	/// Raw scoped dispatch implementation for the Rc-backed Span dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The span tag type.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, Tag, FirstLayer>
		DispatchRcRunRawScopedHandler<R, S, A, SpanBrand<RcBrand, Tag>, FirstLayer> for SpanHandler
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		Tag: 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
	{
		#[document_signature]
		#[document_parameters(
			"The raw scoped operation layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		rc_run::RcRun,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, i32>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = RcRun::span::<i32, _>(7, RcRun::pure(42));
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SpanBrand<RcBrand, i32>: span_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_rc_run_raw_scoped_head(
			&self,
			layer: Span<'static, RcBrand, Tag, RawRcRunFree<R, S>>,
			continuations: RcRunContinuations<R, S>,
			fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
		) -> RcRun<R, S, A> {
			match layer {
				Span::Span {
					tag,
					action,
				} => ScopedContinuation::new(RcRunRawScopedContinuation {
					action: action(()),
					continuations,
					result: PhantomData,
				})
				.resume_rc_with_post_action(fo_handlers, move |action_value| {
					let _ = &tag;
					RcFree::<NodeBrand<R, S>, RcTypeErasedValue>::from_erased_value(action_value)
				}),
			}
		}
	}

	/// Raw scoped dispatch implementation for the Arc-backed Span dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The span tag type.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, Tag, FirstLayer>
		DispatchArcRunRawScopedHandler<R, S, A, SendSpanBrand<ArcBrand, Tag>, FirstLayer> for SpanHandler
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		Tag: Clone + Send + Sync + 'static,
		FirstLayer: 'static,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone + Send + Sync,
	{
		#[document_signature]
		#[document_parameters(
			"The raw scoped operation layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, i32>, CNilBrand>;
		/// type Prog = ArcRun<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = ArcRun::span::<i32, _>(7, ArcRun::pure(42));
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SendSpanBrand<ArcBrand, i32>: span_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_arc_run_raw_scoped_head(
			&self,
			layer: SendSpan<'static, ArcBrand, Tag, RawArcRunFree<R, S>>,
			continuations: ArcRunContinuations<R, S>,
			fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
		) -> ArcRun<R, S, A> {
			match layer {
				SendSpan::Span {
					tag,
					action,
				} => ScopedContinuation::new(ArcRunRawScopedContinuation {
					action: action(()),
					continuations,
					result: PhantomData,
				})
				.resume_arc_with_post_action(fo_handlers, move |action_value| {
					let _ = &tag;
					ArcFree::<NodeBrand<R, S>, ArcTypeErasedValue>::from_erased_value(action_value)
				}),
			}
		}
	}

	/// Dispatch implementation for a standard span dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The span tag type."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, Tag>
		DispatchScopedHandler<
			'static,
			Span<'static, RcBrand, Tag, RcRun<R, S, A>>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
			RcRun<R, S, A>,
		> for SpanHandler
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
		Tag: 'static,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped operation layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxSpanBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: Span<'static, RcBrand, Tag, RcRun<R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
				RcRun<R, S, A>,
			>,
		) -> RcRun<R, S, A> {
			match layer {
				Span::Span {
					tag: _tag,
					action,
				} => action(()),
			}
		}
	}

	/// Dispatch implementation for a standard span dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The span tag type."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, Tag>
		DispatchScopedHandler<
			'static,
			SendSpan<'static, ArcBrand, Tag, ArcRun<R, S, A>>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			ArcRun<R, S, A>,
		> for SpanHandler
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		A: Send + Sync + 'static,
		Tag: Send + Sync + 'static,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped operation layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxSpanBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendSpan<'static, ArcBrand, Tag, ArcRun<R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
				ArcRun<R, S, A>,
			>,
		) -> ArcRun<R, S, A> {
			match layer {
				SendSpan::Span {
					tag: _tag,
					action,
				} => action(()),
			}
		}
	}

	/// Dispatch implementation for a standard span dispatcher.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The span tag type."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, A, Tag>
		DispatchScopedHandler<
			'a,
			BoxSpan<'a, BoxBrand, Tag, RunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>),
			RunExplicit<'a, R, S, A>,
		> for SpanHandler
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a,
		Tag: 'a + 'static,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped operation layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxSpanBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: BoxSpan<'a, BoxBrand, Tag, RunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>
				),
				RunExplicit<'a, R, S, A>,
			>,
		) -> RunExplicit<'a, R, S, A> {
			match layer {
				BoxSpan::Span {
					tag: _tag,
					action,
				} => action(()),
			}
		}
	}

	/// Dispatch implementation for a standard span dispatcher.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The span tag type."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, A, Tag>
		DispatchScopedHandler<
			'a,
			Span<'a, RcBrand, Tag, RcRunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
			RcRunExplicit<'a, R, S, A>,
		> for SpanHandler
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a,
		Tag: 'a + 'static,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped operation layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxSpanBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: Span<'a, RcBrand, Tag, RcRunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
				),
				RcRunExplicit<'a, R, S, A>,
			>,
		) -> RcRunExplicit<'a, R, S, A> {
			match layer {
				Span::Span {
					tag: _tag,
					action,
				} => action(()),
			}
		}
	}

	/// Dispatch implementation for a standard span dispatcher.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The span tag type."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, A, Tag>
		DispatchScopedHandler<
			'a,
			SendSpan<'a, ArcBrand, Tag, ArcRunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
			ArcRunExplicit<'a, R, S, A>,
		> for SpanHandler
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Send + Sync + 'a,
		Tag: Send + Sync + 'a + 'static,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped operation layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxSpanBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendSpan<'a, ArcBrand, Tag, ArcRunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
				),
				ArcRunExplicit<'a, R, S, A>,
			>,
		) -> ArcRunExplicit<'a, R, S, A> {
			match layer {
				SendSpan::Span {
					tag: _tag,
					action,
				} => action(()),
			}
		}
	}
}

pub use inner::*;
