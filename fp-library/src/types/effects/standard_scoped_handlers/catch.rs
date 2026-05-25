#[allow(
	unused_imports,
	reason = "Each scoped-handler child module consumes a different subset of the shared parent prelude."
)]
use super::prelude::*;

mod carrier;

mod raw_replacers;

#[fp_macros::document_module]
mod inner {
	use super::{
		raw_replacers::*,
		*,
	};

	/// Handler for the standard `Catch` scoped effect.
	///
	/// `Idx`, `RMinusE`, and `EmbedIndices` are the same row witnesses
	/// consumed by each wrapper's `interpose` method: the position of
	/// `ExceptBrand<E>` in the first-order row, the row with that effect
	/// removed, and the witness for embedding the narrowed row back into
	/// the original row while preserving surrounding scoped operations.
	///
	/// This handler is implemented for Rc-backed and Arc-backed
	/// wrappers. Box-backed `Run` / `RunExplicit` need a separate design:
	/// after `Run::peel` maps a suspended `BoxCatch` layer, both the
	/// protected action and the recovery handler can need the same
	/// single-shot `Free` continuation.
	#[derive(Clone, Copy, Debug, Default)]
	#[expect(
		clippy::type_complexity,
		reason = "The fn marker carries row-witness type parameters without making auto-traits depend on them."
	)]
	pub struct CatchHandler<Idx, RMinusE, EmbedIndices>(
		PhantomData<fn() -> (Idx, RMinusE, EmbedIndices)>,
	);

	/// Constructs a [`CatchHandler`] without naming its private field.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		BoxBrand,
	/// 		BoxCatchBrand,
	/// 		BoxSpanBrand,
	/// 		CNilBrand,
	/// 		CoproductBrand,
	/// 		CoyonedaBrand,
	/// 		ExceptBrand,
	/// 	},
	/// 	handlers,
	/// 	scoped_handlers,
	/// 	types::effects::{
	/// 		except::Except,
	/// 		run::Run,
	/// 		standard_scoped_handlers::{
	/// 			catch_handler,
	/// 			span_handler,
	/// 		},
	/// 	},
	/// };
	///
	/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
	/// type FirstRowMinusExcept = CNilBrand;
	/// type ScopedRow = CoproductBrand<
	/// 	BoxCatchBrand<BoxBrand, &'static str>,
	/// 	CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>,
	/// >;
	/// type Prog = Run<FirstRow, ScopedRow, i32>;
	///
	/// let action: Prog = Run::span::<&'static str, _>("inner", Run::throw::<&'static str, _>("boom"));
	/// let program: Prog = Run::catch::<&'static str, _>(action, |_err| Run::pure(42));
	/// let catch = catch_handler::<_, FirstRowMinusExcept, _>();
	///
	/// let result = program.handle(
	/// 	handlers! {
	/// 		ExceptBrand<&'static str>: |_op: Except<'_, &'static str, Prog>| Run::pure(0),
	/// 	},
	/// 	scoped_handlers! {
	/// 		BoxCatchBrand<BoxBrand, &'static str>: catch,
	/// 		BoxSpanBrand<BoxBrand, &'static str>: span_handler(),
	/// 	},
	/// );
	///
	/// assert_eq!(result, 42);
	/// ```
	pub const fn catch_handler<Idx, RMinusE, EmbedIndices>()
	-> CatchHandler<Idx, RMinusE, EmbedIndices> {
		CatchHandler(PhantomData)
	}

	/// Dispatch implementation for a standard scoped-effect wrapper.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The scoped effect payload or environment type.",
		"The row index witnessing the target first-order operation.",
		"The first-order row brand with the handled operation removed.",
		"The row embedding witness used to rebuild the original row.",
		"The first first-order handler layer type."
	)]
	#[document_parameters("The handler receiver.")]
	impl<R, S, A, E, Idx, RMinusE, EmbedIndices, FirstLayer>
		DispatchRunRawScopedHandler<R, S, A, BoxCatchBrand<BoxBrand, E>, FirstLayer>
		for CatchHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
		E: 'static,
		FirstLayer: 'static,
		RMinusE: WrapDrop + Functor + 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		Run<R, S, crate::types::free::TypeErasedValue>,
	>): Member<
				Coyoneda<'static, ExceptBrand<E>, Run<R, S, crate::types::free::TypeErasedValue>>,
				Idx,
				Remainder = Apply!(
								<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
									'static,
									Run<R, S, crate::types::free::TypeErasedValue>,
								>
							),
			>,
		Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
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
		#[document_signature]
		///
		#[document_parameters(
			"The raw scoped operation layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(
			skip_call_check,
			reason = "This raw scoped-handler protocol hook receives type-erased Run action carriers and continuation stacks constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the supported public handler path."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxCatchBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		CoyonedaBrand,
		/// 		ExceptBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		except::Except,
		/// 		run::Run,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<i32>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::catch::<i32, _>(Run::throw::<i32, _>(7), |err| Run::pure(err + 35));
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ExceptBrand<i32>: |_op: Except<'_, i32, Prog>| Run::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxCatchBrand<BoxBrand, i32>: catch_handler::<_, FirstRowMinusExcept, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_run_raw_scoped_head(
			&self,
			layer: BoxCatch<'static, BoxBrand, E, RawRunFree<R, S>>,
			continuations: RunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
		) -> Run<R, S, A> {
			match layer {
				BoxCatch::Catch {
					action,
					handler,
				} => {
					let interposed = Run::<R, S, crate::types::free::TypeErasedValue>::from_free(
						action(()).erase_type(),
					)
					.interpose_with_replacer::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(
						BoxCatchRawRunReplacer {
							handler: std::cell::RefCell::new(Some(handler)),
							_row: PhantomData,
							_scoped: PhantomData,
							_error: PhantomData,
						},
					);
					Run::from_free(Free::continue_from_reboxed_erased(
						interposed.into_free(),
						continuations,
					))
				}
			}
		}
	}

	/// Raw scoped dispatch implementation for the Rc-backed Catch handler.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The handled error type.",
		"The first-order row index witnessing the handled Except operation.",
		"The first-order row brand with the handled Except operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Catch handler receiver.")]
	impl<R, S, A, E, Idx, RMinusE, EmbedIndices, FirstLayer>
		DispatchRcRunRawScopedHandler<R, S, A, CatchBrand<RcBrand, E>, FirstLayer>
		for CatchHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		E: 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcRun<R, S, RcTypeErasedValue>,
		>): Member<
				RcCoyoneda<'static, ExceptBrand<E>, RcRun<R, S, RcTypeErasedValue>>,
				Idx,
				Remainder = Apply!(
								<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
									'static,
									RcRun<R, S, RcTypeErasedValue>,
								>
							),
			>,
		Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
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
		#[document_signature]
		#[document_parameters(
			"The raw Catch layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(
			skip_call_check,
			reason = "This raw scoped-handler protocol hook receives type-erased RcRun action carriers and continuation stacks constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the supported public handler path."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		except::Except,
		/// 		rc_run::RcRun,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type ScopedRow = CoproductBrand<CatchBrand<RcBrand, &'static str>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog =
		/// 	RcRun::catch::<&'static str, _>(RcRun::throw::<&'static str, _>("err"), |_| {
		/// 		RcRun::pure(42)
		/// 	});
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ExceptBrand<&'static str>: |_op: Except<'_, &'static str, Prog>| RcRun::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		CatchBrand<RcBrand, &'static str>: catch_handler::<_, CNilBrand, _>(),
		/// 	},
		/// );
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_rc_run_raw_scoped_head(
			&self,
			layer: Catch<'static, RcBrand, E, RawRcRunFree<R, S>>,
			continuations: RcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
		) -> RcRun<R, S, A> {
			match layer {
				Catch::Catch {
					action,
					handler,
				} => {
					let interposed =
						RcRun::<R, S, RcTypeErasedValue>::from_rc_free(action(()).erase_type())
							.interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(
							move |op| match op {
								Except::Throw(e, _) => RcRun::from_rc_free(handler(e).erase_type()),
							},
						);
					RcRun::from_rc_free(RcFree::continue_from_reboxed_erased(
						interposed.into_rc_free(),
						continuations,
					))
				}
			}
		}
	}

	/// Raw scoped dispatch implementation for the Arc-backed Catch handler.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The handled error type.",
		"The first-order row index witnessing the handled Except operation.",
		"The first-order row brand with the handled Except operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Catch handler receiver.")]
	impl<R, S, A, E, Idx, RMinusE, EmbedIndices, FirstLayer>
		DispatchArcRunRawScopedHandler<R, S, A, SendCatchBrand<ArcBrand, E>, FirstLayer>
		for CatchHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		E: Send + Sync + 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		FirstLayer: 'static,
		ExceptBrand<E>: SendFunctor
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcRun<R, S, ArcTypeErasedValue>> = Except<
					'static,
					E,
					ArcRun<R, S, ArcTypeErasedValue>,
				>,
			>,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone + Send + Sync,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcRun<R, S, ArcTypeErasedValue>,
		>): Member<
				ArcCoyoneda<'static, ExceptBrand<E>, ArcRun<R, S, ArcTypeErasedValue>>,
				Idx,
				Remainder = Apply!(
								<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
									'static,
									ArcRun<R, S, ArcTypeErasedValue>,
								>
							),
			>,
		Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
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
		#[document_signature]
		#[document_parameters(
			"The raw Catch layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(
			skip_call_check,
			reason = "This raw scoped-handler protocol hook receives type-erased ArcRun action carriers and continuation stacks constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the supported public handler path."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		except::Except,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type ScopedRow = CoproductBrand<SendCatchBrand<ArcBrand, &'static str>, CNilBrand>;
		/// type Prog = ArcRun<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog =
		/// 	ArcRun::catch::<&'static str, _>(ArcRun::throw::<&'static str, _>("err"), |_| {
		/// 		ArcRun::pure(42)
		/// 	});
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ExceptBrand<&'static str>: |_op: Except<'_, &'static str, Prog>| ArcRun::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		SendCatchBrand<ArcBrand, &'static str>: catch_handler::<_, CNilBrand, _>(),
		/// 	},
		/// );
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_arc_run_raw_scoped_head(
			&self,
			layer: SendCatch<'static, ArcBrand, E, RawArcRunFree<R, S>>,
			continuations: ArcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
		) -> ArcRun<R, S, A> {
			match layer {
				SendCatch::Catch {
					action,
					handler,
				} => {
					let interposed =
						ArcRun::<R, S, ArcTypeErasedValue>::from_arc_free(action(()).erase_type())
							.interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(
							move |op| match op {
								Except::Throw(e, _) =>
									ArcRun::from_arc_free(handler(e).erase_type()),
							},
						);
					ArcRun::from_arc_free(ArcFree::continue_from_reboxed_erased(
						interposed.into_arc_free(),
						continuations,
					))
				}
			}
		}
	}

	/// Dispatch implementation for a standard scoped-effect wrapper.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The scoped effect payload or environment type.",
		"The row index witnessing the target first-order operation.",
		"The first-order row brand with the handled operation removed.",
		"The row embedding witness used to rebuild the original row."
	)]
	#[document_parameters("The handler receiver.")]
	impl<R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'static,
			Catch<'static, RcBrand, E, RcRun<R, S, A>>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
			RcRun<R, S, A>,
		> for CatchHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		E: 'static,
		RMinusE: WrapDrop + Functor + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
	>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>): Member<
				RcCoyoneda<'static, ExceptBrand<E>, RcRun<R, S, A>>,
				Idx,
				Remainder = Apply!(
								<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>
							),
			>,
		Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<R, S>, A>,
	>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, A>,
			>),
				EmbedIndices,
			>,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped operation layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(
			skip_call_check,
			reason = "This trait impl method is a scoped-handler protocol hook; the public API is installing the handler with scoped_handlers! and running handle, so the example documents the supported handler path instead of direct protocol invocation."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		except::Except,
		/// 		rc_run::RcRun,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<i32>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<CatchBrand<RcBrand, i32>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog =
		/// 	RcRun::catch::<i32, _>(RcRun::throw::<i32, _>(7), |err| RcRun::pure(err + 35));
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ExceptBrand<i32>: |_op: Except<'_, i32, Prog>| RcRun::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		CatchBrand<RcBrand, i32>: catch_handler::<_, FirstRowMinusExcept, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: Catch<'static, RcBrand, E, RcRun<R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
				RcRun<R, S, A>,
			>,
		) -> RcRun<R, S, A> {
			match layer {
				Catch::Catch {
					action,
					handler,
				} => action(()).interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
					match op {
						Except::Throw(e, _) => (*handler)(e),
					}
				}),
			}
		}
	}

	/// Dispatch implementation for a standard scoped-effect wrapper.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The scoped effect payload or environment type.",
		"The row index witnessing the target first-order operation.",
		"The first-order row brand with the handled operation removed.",
		"The row embedding witness used to rebuild the original row."
	)]
	#[document_parameters("The handler receiver.")]
	impl<R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'static,
			SendCatch<'static, ArcBrand, E, ArcRun<R, S, A>>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			ArcRun<R, S, A>,
		> for CatchHandler<Idx, RMinusE, EmbedIndices>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		E: Send + Sync + 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		ExceptBrand<E>: Functor
			+ SendFunctor
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcRun<R, S, A>> = Except<'static, E, ArcRun<R, S, A>>,
			>,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
	>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
				ArcCoyoneda<'static, ExceptBrand<E>, ArcRun<R, S, A>>,
				Idx,
				Remainder = Apply!(
								<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
							),
			>,
		Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		ArcFree<NodeBrand<R, S>, A>,
	>): CoproductEmbedder<
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, A>,
			>),
				EmbedIndices,
			>,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped operation layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(
			skip_call_check,
			reason = "This trait impl method is a scoped-handler protocol hook; the public API is installing the handler with scoped_handlers! and running handle, so the example documents the supported handler path instead of direct protocol invocation."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		except::Except,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<i32>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendCatchBrand<ArcBrand, i32>, CNilBrand>;
		/// type Prog = ArcRun<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog =
		/// 	ArcRun::catch::<i32, _>(ArcRun::throw::<i32, _>(7), |err| ArcRun::pure(err + 35));
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ExceptBrand<i32>: |_op: Except<'_, i32, Prog>| ArcRun::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		SendCatchBrand<ArcBrand, i32>: catch_handler::<_, FirstRowMinusExcept, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendCatch<'static, ArcBrand, E, ArcRun<R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
				ArcRun<R, S, A>,
			>,
		) -> ArcRun<R, S, A> {
			match layer {
				SendCatch::Catch {
					action,
					handler,
				} => action(()).interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
					match op {
						Except::Throw(e, _) => (*handler)(e),
					}
				}),
			}
		}
	}

	/// Dispatch implementation for a standard scoped-effect wrapper.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The scoped effect payload or environment type.",
		"The row index witnessing the target first-order operation.",
		"The first-order row brand with the handled operation removed.",
		"The row embedding witness used to rebuild the original row."
	)]
	#[document_parameters("The handler receiver.")]
	impl<'a, R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'a,
			BoxCatch<'a, BoxBrand, E, RunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>),
			RunExplicit<'a, R, S, A>,
		> for CatchHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a,
		E: 'a + 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>): Member<
				Coyoneda<'a, ExceptBrand<E>, RunExplicit<'a, R, S, A>>,
				Idx,
				Remainder = Apply!(
								<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>
							),
			>,
		Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
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
		#[document_signature]
		///
		#[document_parameters(
			"The scoped operation layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(
			skip_call_check,
			reason = "This trait impl method is a scoped-handler protocol hook; the public API is installing the handler with scoped_handlers! and running handle, so the example documents the supported handler path instead of direct protocol invocation."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxCatchBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		CoyonedaBrand,
		/// 		ExceptBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		except::Except,
		/// 		run_explicit::RunExplicit,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<i32>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = RunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = RunExplicit::throw::<i32, _>(7);
		/// let boundary = RunExplicit::catch::<i32, _>(action, |err| RunExplicit::pure(err + 35));
		/// let catch = catch_handler::<_, FirstRowMinusExcept, _>();
		/// let program: Prog = catch.dispatch_run_explicit_catch_boundary(boundary, &handlers! {});
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ExceptBrand<i32>: |_op: Except<'_, i32, Prog>| RunExplicit::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxCatchBrand<BoxBrand, i32>: catch,
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: BoxCatch<'a, BoxBrand, E, RunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>
				),
				RunExplicit<'a, R, S, A>,
			>,
		) -> RunExplicit<'a, R, S, A> {
			match layer {
				BoxCatch::Catch {
					action,
					handler,
				} => {
					let handler = std::cell::RefCell::new(Some(handler));
					action(()).interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
					match op {
						Except::Throw(e, _) => {
							#[expect(
								clippy::expect_used,
								reason = "Box-backed Catch handlers are single-shot and the protected action can throw at most once"
							)]
							let handler = handler
								.borrow_mut()
								.take()
								.expect("BoxCatch handler invoked more than once");
							handler(e)
						}
					}
				})
				}
			}
		}
	}

	/// Dispatch implementation for a standard scoped-effect wrapper.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The scoped effect payload or environment type.",
		"The row index witnessing the target first-order operation.",
		"The first-order row brand with the handled operation removed.",
		"The row embedding witness used to rebuild the original row."
	)]
	#[document_parameters("The handler receiver.")]
	impl<'a, R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'a,
			Catch<'a, RcBrand, E, RcRunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
			RcRunExplicit<'a, R, S, A>,
		> for CatchHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'a,
		E: 'a + 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		RcFreeExplicit<'a, NodeBrand<R, S>, A>,
	>): Clone,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>): Member<
				RcCoyoneda<'a, ExceptBrand<E>, RcRunExplicit<'a, R, S, A>>,
				Idx,
				Remainder = Apply!(
								<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
							),
			>,
		Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
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
		#[document_signature]
		///
		#[document_parameters(
			"The scoped operation layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(
			skip_call_check,
			reason = "This trait impl method is a scoped-handler protocol hook; the public API is installing the handler with scoped_handlers! and running handle, so the example documents the supported handler path instead of direct protocol invocation."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		except::Except,
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<i32>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<CatchBrand<RcBrand, i32>, CNilBrand>;
		/// type Prog = RcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = RcRunExplicit::throw::<i32, _>(7);
		/// let boundary = RcRunExplicit::catch::<i32, _>(action, |err| RcRunExplicit::pure(err + 35));
		/// let catch = catch_handler::<_, FirstRowMinusExcept, _>();
		/// let program: Prog = catch.dispatch_rc_run_explicit_catch_boundary(boundary, &handlers! {});
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ExceptBrand<i32>: |_op: Except<'_, i32, Prog>| RcRunExplicit::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		CatchBrand<RcBrand, i32>: catch,
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: Catch<'a, RcBrand, E, RcRunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
				),
				RcRunExplicit<'a, R, S, A>,
			>,
		) -> RcRunExplicit<'a, R, S, A> {
			match layer {
				Catch::Catch {
					action,
					handler,
				} => action(()).interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
					match op {
						Except::Throw(e, _) => (*handler)(e),
					}
				}),
			}
		}
	}

	/// Dispatch implementation for a standard scoped-effect wrapper.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The scoped effect payload or environment type.",
		"The row index witnessing the target first-order operation.",
		"The first-order row brand with the handled operation removed.",
		"The row embedding witness used to rebuild the original row."
	)]
	#[document_parameters("The handler receiver.")]
	impl<'a, R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'a,
			SendCatch<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
			ArcRunExplicit<'a, R, S, A>,
		> for CatchHandler<Idx, RMinusE, EmbedIndices>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'a,
		E: Send + Sync + 'a + 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		ExceptBrand<E>: Functor
			+ SendFunctor
			+ Kind_cdc7cd43dac7585f<
				Of<'a, ArcRunExplicit<'a, R, S, A>> = Except<'a, E, ArcRunExplicit<'a, R, S, A>>,
			>,
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
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>):
			Send
				+ Sync
				+ Member<
					ArcCoyoneda<'a, ExceptBrand<E>, ArcRunExplicit<'a, R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
								),
				>,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>):
			Send + Sync,
		Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>):
			Send + Sync,
		Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
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
		#[document_signature]
		///
		#[document_parameters(
			"The scoped operation layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(
			skip_call_check,
			reason = "This trait impl method is a scoped-handler protocol hook; the public API is installing the handler with scoped_handlers! and running handle, so the example documents the supported handler path instead of direct protocol invocation."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		except::Except,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<i32>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendCatchBrand<ArcBrand, i32>, CNilBrand>;
		/// type Prog = ArcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = ArcRunExplicit::throw::<i32, _>(7);
		/// let boundary = ArcRunExplicit::catch::<i32, _>(action, |err| ArcRunExplicit::pure(err + 35));
		/// let catch = catch_handler::<_, FirstRowMinusExcept, _>();
		/// let program: Prog = catch.dispatch_arc_run_explicit_catch_boundary(boundary, &handlers! {});
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ExceptBrand<i32>: |_op: Except<'_, i32, Prog>| ArcRunExplicit::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		SendCatchBrand<ArcBrand, i32>: catch,
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendCatch<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
				),
				ArcRunExplicit<'a, R, S, A>,
			>,
		) -> ArcRunExplicit<'a, R, S, A> {
			match layer {
				SendCatch::Catch {
					action,
					handler,
				} => action(()).interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
					match op {
						Except::Throw(e, _) => (*handler)(e),
					}
				}),
			}
		}
	}
}

pub use inner::*;
