#[allow(
	unused_imports,
	reason = "Each scoped-dispatcher child module consumes a different subset of the shared parent prelude."
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

	/// Dispatcher for the standard `Local` scoped effect.
	///
	/// The dispatcher asks the inherited Reader environment once, applies
	/// the stored by-value environment transform, then answers Reader asks
	/// inside the action with clones of the modified environment.
	#[derive(Clone, Copy, Debug, Default)]
	#[expect(
		clippy::type_complexity,
		reason = "The fn marker carries row-witness type parameters without making auto-traits depend on them."
	)]
	pub struct LocalHandler<Idx, RMinusE, EmbedIndices>(
		PhantomData<fn() -> (Idx, RMinusE, EmbedIndices)>,
	);

	/// Constructs a [`LocalHandler`] without naming its private field.
	#[document_examples(skip_call_check)]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		BoxBrand,
	/// 		BoxLocalBrand,
	/// 		BoxReaderBrand,
	/// 		BoxRefLocalBrand,
	/// 		CNilBrand,
	/// 		CoproductBrand,
	/// 		CoyonedaBrand,
	/// 	},
	/// 	handlers,
	/// 	scoped_handlers,
	/// 	types::effects::{
	/// 		reader::BoxReader,
	/// 		run::Run,
	/// 		standard_scoped_handlers::{
	/// 			local_handler,
	/// 			ref_local_handler,
	/// 		},
	/// 	},
	/// };
	///
	/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
	/// type FirstRowMinusReader = CNilBrand;
	/// type ScopedRow = CoproductBrand<
	/// 	BoxLocalBrand<BoxBrand, i32>,
	/// 	CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>,
	/// >;
	/// type Prog = Run<FirstRow, ScopedRow, i32>;
	///
	/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env: i32| Run::pure(env * 2));
	/// let program: Prog = Run::local::<i32, _>(|env| env + 1, action);
	///
	/// let result = program.handle(
	/// 	handlers! {
	/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
	/// 			BoxReader::Ask(k) => k(10),
	/// 		},
	/// 	},
	/// 	scoped_handlers! {
	/// 		BoxLocalBrand<BoxBrand, i32>: local_handler::<_, FirstRowMinusReader, _>(),
	/// 		BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, FirstRowMinusReader, _>(),
	/// 	},
	/// );
	///
	/// assert_eq!(result, 22);
	/// ```
	pub const fn local_handler<Idx, RMinusE, EmbedIndices>()
	-> LocalHandler<Idx, RMinusE, EmbedIndices> {
		LocalHandler(PhantomData)
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
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, E, Idx, RMinusE, EmbedIndices, FirstLayer>
		DispatchRunRawScopedHandler<R, S, A, BoxLocalBrand<BoxBrand, E>, FirstLayer>
		for LocalHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
		E: Clone + 'static,
		FirstLayer: 'static,
		RMinusE: WrapDrop + Functor + 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, E>):
			Member<Coyoneda<'static, BoxReaderBrand<BoxBrand, E>, E>, Idx>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		Run<R, S, crate::types::free::TypeErasedValue>,
	>): Member<
				Coyoneda<
					'static,
					BoxReaderBrand<BoxBrand, E>,
					Run<R, S, crate::types::free::TypeErasedValue>,
				>,
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
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxLocalBrand,
		/// 		BoxReaderBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		CoyonedaBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		reader::BoxReader,
		/// 		run::Run,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env: i32| Run::pure(env * 2));
		/// let program: Prog = Run::local::<i32, _>(|env| env + 1, action);
		/// let result = program.handle(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 22);
		/// ```
		fn dispatch_run_raw_scoped_head(
			&self,
			layer: BoxLocal<'static, BoxBrand, E, RawRunFree<R, S>>,
			continuations: RunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
		) -> Run<R, S, A> {
			match layer {
				BoxLocal::Local {
					modify,
					action,
				} => Run::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(env);
					let interposed = Run::<R, S, crate::types::free::TypeErasedValue>::from_free(
						action(()).erase_type(),
					)
					.interpose_with_replacer::<BoxReaderBrand<BoxBrand, E>, Idx, RMinusE, EmbedIndices>(
						BoxLocalRawRunReplacer {
							local_env,
						},
					);
					Run::from_free(Free::continue_from_reboxed_erased(
						interposed.into_free(),
						continuations,
					))
				}),
			}
		}
	}

	/// Raw scoped dispatch implementation for the Rc-backed Local dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The scoped environment type.",
		"The row index witnessing the target Reader operation.",
		"The first-order row brand with the handled Reader removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Local dispatcher receiver.")]
	impl<R, S, A, E, Idx, RMinusE, EmbedIndices, FirstLayer>
		DispatchRcRunRawScopedHandler<R, S, A, LocalBrand<RcBrand, E>, FirstLayer>
		for LocalHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		E: Clone + 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, E>):
			Member<RcCoyoneda<'static, ReaderBrand<RcBrand, E>, E>, Idx>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcRun<R, S, RcTypeErasedValue>,
		>): Member<
				RcCoyoneda<'static, ReaderBrand<RcBrand, E>, RcRun<R, S, RcTypeErasedValue>>,
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
			"The raw Local layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the dispatcher contract."
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
		/// 		reader::Reader,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		/// type ScopedRow = CoproductBrand<LocalBrand<RcBrand, i32>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// let action = Prog::ask().bind(|env| RcRun::pure(env + 1));
		/// let result = RcRun::local::<i32, _>(|env| env + 1, action).handle(
		/// 	handlers! {
		/// 		ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, Prog>| match op {
		/// 			Reader::Ask(k) => k(40),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		LocalBrand<RcBrand, i32>: local_handler::<_, CNilBrand, _>(),
		/// 	},
		/// );
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_rc_run_raw_scoped_head(
			&self,
			layer: Local<'static, RcBrand, E, RawRcRunFree<R, S>>,
			continuations: RcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
		) -> RcRun<R, S, A> {
			match layer {
				Local::Local {
					modify,
					action,
				} => RcRun::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(env);
					let interposed =
						RcRun::<R, S, RcTypeErasedValue>::from_rc_free(action(()).erase_type())
							.interpose_with_replacer::<ReaderBrand<RcBrand, E>, Idx, RMinusE, EmbedIndices>(
							RcLocalRawRunReplacer {
								local_env,
							},
						);
					RcRun::from_rc_free(RcFree::continue_from_reboxed_erased(
						interposed.into_rc_free(),
						continuations.clone(),
					))
				}),
			}
		}
	}

	/// Raw scoped dispatch implementation for the Arc-backed Local dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final program result type.",
		"The scoped environment type.",
		"The row index witnessing the target Reader operation.",
		"The first-order row brand with the handled Reader removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Local dispatcher receiver.")]
	impl<R, S, A, E, Idx, RMinusE, EmbedIndices, FirstLayer>
		DispatchArcRunRawScopedHandler<R, S, A, SendLocalBrand<ArcBrand, E>, FirstLayer>
		for LocalHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		E: Clone + Send + Sync + 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		FirstLayer: 'static,
		SendReaderBrand<ArcBrand, E>: SendFunctor
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcRun<R, S, ArcTypeErasedValue>> = SendReader<
					'static,
					ArcBrand,
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
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, E>):
			Member<ArcCoyoneda<'static, SendReaderBrand<ArcBrand, E>, E>, Idx>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcRun<R, S, ArcTypeErasedValue>,
		>): Member<
				ArcCoyoneda<
					'static,
					SendReaderBrand<ArcBrand, E>,
					ArcRun<R, S, ArcTypeErasedValue>,
				>,
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
			"The raw Local layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the dispatcher contract."
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
		/// 		reader::SendReader,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		/// type ScopedRow = CoproductBrand<SendLocalBrand<ArcBrand, i32>, CNilBrand>;
		/// type Prog = ArcRun<FirstRow, ScopedRow, i32>;
		///
		/// let action = Prog::ask().bind(|env| ArcRun::pure(env + 1));
		/// let result = ArcRun::local::<i32, _>(|env| env + 1, action).handle(
		/// 	handlers! {
		/// 		SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, Prog>| match op {
		/// 			SendReader::Ask(k) => k(40),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		SendLocalBrand<ArcBrand, i32>: local_handler::<_, CNilBrand, _>(),
		/// 	},
		/// );
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_arc_run_raw_scoped_head(
			&self,
			layer: SendLocal<'static, ArcBrand, E, RawArcRunFree<R, S>>,
			continuations: ArcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
		) -> ArcRun<R, S, A> {
			match layer {
				SendLocal::Local {
					modify,
					action,
				} => ArcRun::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(env);
					let interposed =
						ArcRun::<R, S, ArcTypeErasedValue>::from_arc_free(action(()).erase_type())
							.interpose_with_replacer::<SendReaderBrand<ArcBrand, E>, Idx, RMinusE, EmbedIndices>(
							ArcLocalRawRunReplacer {
								local_env,
							},
						);
					ArcRun::from_arc_free(ArcFree::continue_from_reboxed_erased(
						interposed.into_arc_free(),
						continuations.clone(),
					))
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
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'static,
			Local<'static, RcBrand, E, RcRun<R, S, A>>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
			RcRun<R, S, A>,
		> for LocalHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		E: Clone + 'static,
		RMinusE: WrapDrop + Functor + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
	>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, E>):
			Member<RcCoyoneda<'static, ReaderBrand<RcBrand, E>, E>, Idx>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>): Member<
				RcCoyoneda<'static, ReaderBrand<RcBrand, E>, RcRun<R, S, A>>,
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
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxLocalBrand,
		/// 		BoxReaderBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		CoyonedaBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		reader::BoxReader,
		/// 		run::Run,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env: i32| Run::pure(env * 2));
		/// let program: Prog = Run::local::<i32, _>(|env| env + 1, action);
		/// let result = program.handle(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 22);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: Local<'static, RcBrand, E, RcRun<R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
				RcRun<R, S, A>,
			>,
		) -> RcRun<R, S, A> {
			match layer {
				Local::Local {
					modify,
					action,
				} => RcRun::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(env);
					action(()).interpose::<ReaderBrand<RcBrand, E>, Idx, RMinusE, EmbedIndices>(
						move |op| match op {
							Reader::Ask(k) => k(local_env.clone()),
						},
					)
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
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'static,
			SendLocal<'static, ArcBrand, E, ArcRun<R, S, A>>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			ArcRun<R, S, A>,
		> for LocalHandler<Idx, RMinusE, EmbedIndices>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		E: Clone + Send + Sync + 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		SendReaderBrand<ArcBrand, E>: SendFunctor
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcRun<R, S, A>> = SendReader<'static, ArcBrand, E, ArcRun<R, S, A>>,
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
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, E>):
			Member<ArcCoyoneda<'static, SendReaderBrand<ArcBrand, E>, E>, Idx>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
				ArcCoyoneda<'static, SendReaderBrand<ArcBrand, E>, ArcRun<R, S, A>>,
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
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxLocalBrand,
		/// 		BoxReaderBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		CoyonedaBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		reader::BoxReader,
		/// 		run::Run,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env: i32| Run::pure(env * 2));
		/// let program: Prog = Run::local::<i32, _>(|env| env + 1, action);
		/// let result = program.handle(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 22);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendLocal<'static, ArcBrand, E, ArcRun<R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
				ArcRun<R, S, A>,
			>,
		) -> ArcRun<R, S, A> {
			match layer {
				SendLocal::Local {
					modify,
					action,
				} => ArcRun::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(env);
					action(())
						.interpose::<SendReaderBrand<ArcBrand, E>, Idx, RMinusE, EmbedIndices>(
							move |op| match op {
								SendReader::Ask(k) => k(local_env.clone()),
							},
						)
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
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'a,
			BoxLocal<'a, BoxBrand, E, RunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>),
			RunExplicit<'a, R, S, A>,
		> for LocalHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a,
		E: Clone + 'a + 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, E>):
			Member<Coyoneda<'a, BoxReaderBrand<BoxBrand, E>, E>, Idx>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>): Member<
				Coyoneda<'a, BoxReaderBrand<BoxBrand, E>, RunExplicit<'a, R, S, A>>,
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
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxLocalBrand,
		/// 		BoxReaderBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		CoyonedaBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		reader::BoxReader,
		/// 		run::Run,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env: i32| Run::pure(env * 2));
		/// let program: Prog = Run::local::<i32, _>(|env| env + 1, action);
		/// let result = program.handle(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 22);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: BoxLocal<'a, BoxBrand, E, RunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>
				),
				RunExplicit<'a, R, S, A>,
			>,
		) -> RunExplicit<'a, R, S, A> {
			match layer {
				BoxLocal::Local {
					modify,
					action,
				} => {
					let modify = std::cell::RefCell::new(Some(modify));
					let action = std::cell::RefCell::new(Some(action));
					RunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
						#[expect(
							clippy::expect_used,
							reason = "Box-backed Local is single-shot; RunExplicit invokes this continuation once"
						)]
						let modify = modify
							.borrow_mut()
							.take()
							.expect("BoxLocal modify invoked more than once");
						#[expect(
							clippy::expect_used,
							reason = "Box-backed Local is single-shot; RunExplicit invokes this continuation once"
						)]
						let action = action
							.borrow_mut()
							.take()
							.expect("BoxLocal action invoked more than once");
						let local_env = modify(env);
						action(())
							.interpose::<BoxReaderBrand<BoxBrand, E>, Idx, RMinusE, EmbedIndices>(
								move |op| match op {
									BoxReader::Ask(k) => k(local_env.clone()),
								},
							)
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
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'a,
			Local<'a, RcBrand, E, RcRunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
			RcRunExplicit<'a, R, S, A>,
		> for LocalHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'a,
		E: Clone + 'a + 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		RcFreeExplicit<'a, NodeBrand<R, S>, E>,
	>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		RcFreeExplicit<'a, NodeBrand<R, S>, A>,
	>): Clone,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, E>):
			Member<RcCoyoneda<'a, ReaderBrand<RcBrand, E>, E>, Idx>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>): Member<
				RcCoyoneda<'a, ReaderBrand<RcBrand, E>, RcRunExplicit<'a, R, S, A>>,
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
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxLocalBrand,
		/// 		BoxReaderBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		CoyonedaBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		reader::BoxReader,
		/// 		run::Run,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env: i32| Run::pure(env * 2));
		/// let program: Prog = Run::local::<i32, _>(|env| env + 1, action);
		/// let result = program.handle(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 22);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: Local<'a, RcBrand, E, RcRunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
				),
				RcRunExplicit<'a, R, S, A>,
			>,
		) -> RcRunExplicit<'a, R, S, A> {
			match layer {
				Local::Local {
					modify,
					action,
				} => RcRunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(env);
					action(()).interpose::<ReaderBrand<RcBrand, E>, Idx, RMinusE, EmbedIndices>(
						move |op| match op {
							Reader::Ask(k) => k(local_env.clone()),
						},
					)
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
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'a,
			SendLocal<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
			ArcRunExplicit<'a, R, S, A>,
		> for LocalHandler<Idx, RMinusE, EmbedIndices>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'a,
		E: Clone + Send + Sync + 'a + 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		SendReaderBrand<ArcBrand, E>: SendFunctor
			+ Kind_cdc7cd43dac7585f<
				Of<'a, ArcRunExplicit<'a, R, S, A>> = SendReader<
					'a,
					ArcBrand,
					E,
					ArcRunExplicit<'a, R, S, A>,
				>,
			>,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
	>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
	>): Clone + Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
	>): Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
	>): Send + Sync,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
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
				ArcCoyoneda<'a, SendReaderBrand<ArcBrand, E>, ArcRunExplicit<'a, R, S, A>>,
				Idx,
				Remainder = Apply!(
								<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
							),
			>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, E>):
			Member<ArcCoyoneda<'a, SendReaderBrand<ArcBrand, E>, E>, Idx>,
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
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxLocalBrand,
		/// 		BoxReaderBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		CoyonedaBrand,
		/// 	},
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		reader::BoxReader,
		/// 		run::Run,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env: i32| Run::pure(env * 2));
		/// let program: Prog = Run::local::<i32, _>(|env| env + 1, action);
		/// let result = program.handle(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 22);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendLocal<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
				),
				ArcRunExplicit<'a, R, S, A>,
			>,
		) -> ArcRunExplicit<'a, R, S, A> {
			match layer {
				SendLocal::Local {
					modify,
					action,
				} => ArcRunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(env);
					action(())
						.interpose::<SendReaderBrand<ArcBrand, E>, Idx, RMinusE, EmbedIndices>(
							move |op| match op {
								SendReader::Ask(k) => k(local_env.clone()),
							},
						)
				}),
			}
		}
	}
}

pub use inner::*;
