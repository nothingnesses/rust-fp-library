//! Standard scoped-effect dispatcher values.
//!
//! These dispatchers are runtime handler-list cells for built-in scoped
//! effects. They implement
//! [`DispatchScopedHandler`](crate::types::effects::interpreter::DispatchScopedHandler)
//! so callers can pass them to `scoped_handlers!` or the scoped handler
//! builder API. Dispatchers that rewrite first-order operations carry
//! the row evidence needed by the underlying `interpose` operation;
//! simple around-action dispatchers are witness-free.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				BoxBracketBrand,
				BoxBrand,
				BoxCatchBrand,
				BoxLocalBrand,
				BoxReaderBrand,
				BoxRefLocalBrand,
				BoxSpanBrand,
				ExceptBrand,
				NodeBrand,
				RcBrand,
				ReaderBrand,
				SendReaderBrand,
			},
			classes::{
				Functor,
				SendFunctor,
				WrapDrop,
			},
			kinds::*,
			types::{
				ArcCoyoneda,
				ArcFree,
				ArcFreeExplicit,
				Coyoneda,
				Free,
				FreeExplicit,
				RcCoyoneda,
				RcFree,
				RcFreeExplicit,
				arc_free::ArcTypeErasedValue,
				effects::{
					arc_run::ArcRun,
					arc_run_explicit::{
						ArcRunExplicit,
						ArcRunExplicitLifecycleScopedContinuation,
						ArcRunExplicitScopedContinuation,
					},
					bracket::{
						BoxBracket,
						BoxBracketExplicit,
						Bracket,
						BracketExplicit,
						SendBracket,
						SendBracketExplicit,
					},
					catch::{
						BoxCatch,
						Catch,
						SendCatch,
					},
					coproduct::CoproductEmbedder,
					except::Except,
					interpreter::{
						ArcLifecycleScopedResume,
						ArcScopedResume,
						DispatchHandlers,
						DispatchScopedHandler,
						ExplicitLifecycleScopedResume,
						ExplicitScopedResume,
						RcLifecycleScopedResume,
						RcScopedResume,
						ScopedResumeTypes,
					},
					local::{
						BoxLocal,
						Local,
						SendLocal,
					},
					member::Member,
					rc_run::RcRun,
					rc_run_explicit::{
						RcRunExplicit,
						RcRunExplicitLifecycleScopedContinuation,
						RcRunExplicitScopedContinuation,
					},
					reader::{
						BoxReader,
						Reader,
						SendReader,
					},
					ref_bracket::{
						RefBracket,
						RefBracketExplicit,
						SendRefBracket,
						SendRefBracketExplicit,
					},
					ref_local::{
						BoxRefLocal,
						RefLocal,
						SendRefLocal,
					},
					run::{
						DispatchRunRawScopedHandler,
						RawRunFree,
						Run,
						RunContinuations,
					},
					run_explicit::{
						RunExplicit,
						RunExplicitBracketCarrierLayer,
						RunExplicitCatchCarrierLayer,
						RunExplicitLifecycleScopedContinuation,
						RunExplicitLocalCarrierLayer,
						RunExplicitRefBracketCarrierLayer,
						RunExplicitRefLocalCarrierLayer,
						RunExplicitScopedContinuation,
						RunExplicitSpanCarrierLayer,
					},
					span::{
						BoxSpan,
						SendSpan,
						Span,
					},
				},
				rc_free::RcTypeErasedValue,
			},
		},
		fp_macros::*,
		std::{
			marker::PhantomData,
			rc::Rc,
			sync::Arc,
		},
	};

	/// Dispatcher for the standard `Catch` scoped effect.
	///
	/// `Idx`, `RMinusE`, and `EmbedIndices` are the same row witnesses
	/// consumed by each wrapper's `interpose` method: the position of
	/// `ExceptBrand<E>` in the first-order row, the row with that effect
	/// removed, and the witness for embedding the narrowed row back into
	/// the original row while preserving surrounding scoped operations.
	///
	/// This dispatcher is implemented for Rc-backed and Arc-backed
	/// wrappers. Box-backed `Run` / `RunExplicit` need a separate design:
	/// after `Run::peel` maps a suspended `BoxCatch` layer, both the
	/// protected action and the recovery handler can need the same
	/// single-shot `Free` continuation.
	#[derive(Clone, Copy, Debug, Default)]
	#[expect(
		clippy::type_complexity,
		reason = "The fn marker carries row-witness type parameters without making auto-traits depend on them."
	)]
	pub struct CatchDispatcher<Idx, RMinusE, EmbedIndices>(
		PhantomData<fn() -> (Idx, RMinusE, EmbedIndices)>,
	);

	/// Constructs a [`CatchDispatcher`] without naming its private field.
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
	/// 		scoped_dispatchers::{
	/// 			catch_dispatcher,
	/// 			span_dispatcher,
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
	///
	/// let result = program.interpret(
	/// 	handlers! {
	/// 		ExceptBrand<&'static str>: |_op: Except<'_, &'static str, Prog>| Run::pure(0),
	/// 	},
	/// 	scoped_handlers! {
	/// 		BoxCatchBrand<BoxBrand, &'static str>: catch_dispatcher::<_, FirstRowMinusExcept, _>(),
	/// 		BoxSpanBrand<BoxBrand, &'static str>: span_dispatcher(),
	/// 	},
	/// );
	///
	/// assert_eq!(result, 42);
	/// ```
	pub const fn catch_dispatcher<Idx, RMinusE, EmbedIndices>()
	-> CatchDispatcher<Idx, RMinusE, EmbedIndices> {
		CatchDispatcher(PhantomData)
	}

	#[document_type_parameters(
		"The row index witnessing the target Except operation.",
		"The first-order row brand with the Except operation removed.",
		"The row embedding witness used to rebuild the original row."
	)]
	#[document_parameters("The Catch dispatcher receiver.")]
	#[allow(
		dead_code,
		reason = "Focused Catch carrier methods are introduced before the wrapper interpreter route constructs these private layers."
	)]
	impl<Idx, RMinusE, EmbedIndices> CatchDispatcher<Idx, RMinusE, EmbedIndices> {
		/// Dispatch a private `RunExplicit` Catch carrier-cell layer.
		///
		/// The dispatcher transforms the selected action by interposing
		/// the target Except operation. Thrown errors run the stored
		/// recovery handler; the outer continuation resumes only if the
		/// action or recovery produces an action value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Catch action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The recovered error type.",
			"The recovery handler type.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The private Catch layer carrying the handler and `RunExplicit` carrier cell.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// let recover = |err: &'static str| err.len() as i32;
		/// let recovered = recover("boom") + 1;
		/// assert_eq!(recovered, 5);
		/// ```
		#[inline]
		pub(crate) fn dispatch_run_explicit_catch_carrier<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			E,
			Handler,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitCatchCarrierLayer<
				'a,
				E,
				Handler,
				RunExplicitScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &'a (impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>> + 'a),
		) -> RunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Action: 'a,
			Final: 'a,
			K: Fn(Action) -> RunExplicit<'a, R, S, Final> + 'a,
			E: 'a + 'static,
			Handler: FnOnce(E) -> RunExplicit<'a, R, S, Action> + 'a,
			FirstLayer: 'a,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RunExplicit<'a, R, S, Action>,
			>): Member<
					Coyoneda<'a, ExceptBrand<E>, RunExplicit<'a, R, S, Action>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
										'a,
										RunExplicit<'a, R, S, Action>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				Box<FreeExplicit<'a, NodeBrand<R, S>, Action>>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						Box<FreeExplicit<'a, NodeBrand<R, S>, Action>>,
					>),
					EmbedIndices,
				>,
			RunExplicitScopedContinuation<'a, R, S, Action, Final, K>: ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = RunExplicit<'a, R, S, Action>,
				> + ExplicitScopedResume<'a, FirstLayer, RunExplicit<'a, R, S, Final>>, {
			let (handler, continuation) = layer.into_parts();
			let handler = Rc::new(std::cell::RefCell::new(Some(handler)));

			continuation.resume_explicit_with_action_transform(fo_handlers, move |action| {
				let handler = Rc::clone(&handler);
				action.interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
					match op {
						Except::Throw(e, _) => {
							#[expect(
								clippy::expect_used,
								reason = "Box-backed Catch carrier handlers are single-shot and the protected action can throw at most once"
							)]
							let handler = handler
								.borrow_mut()
								.take()
								.expect("RunExplicit Catch carrier handler invoked more than once");
							handler(e)
						}
					}
				})
			})
		}

		/// Dispatch a private `RcRunExplicit` Catch carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Rc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Catch action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The recovered error type.",
			"The recovery handler type.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The private Catch layer carrying the handler and `RcRunExplicit` carrier cell.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// let recover = |err: &'static str| err.len() as i32;
		/// assert_eq!(recover("boom") + 1, 5);
		/// assert_eq!(recover("fail") + 1, 5);
		/// ```
		#[inline]
		pub(crate) fn dispatch_rc_run_explicit_catch_carrier<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			E,
			Handler,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitCatchCarrierLayer<
				'a,
				E,
				Handler,
				RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &'a (
			        impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>> + 'a
			    ),
		) -> RcRunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Action: Clone + 'a,
			Final: 'a,
			K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a,
			E: 'a + 'static,
			Handler: Fn(E) -> RcRunExplicit<'a, R, S, Action> + Clone + 'a,
			FirstLayer: 'a,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
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
					RcCoyoneda<'a, ExceptBrand<E>, RcRunExplicit<'a, R, S, Action>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
										'a,
										RcRunExplicit<'a, R, S, Action>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
					>),
					EmbedIndices,
				>,
			RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>: Clone
				+ ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = RcRunExplicit<'a, R, S, Action>,
				> + RcScopedResume<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>, {
			let (handler, continuation) = layer.into_parts();

			continuation.resume_rc_with_action_transform(fo_handlers, move |action| {
				let handler = handler.clone();
				action.interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| match op {
					Except::Throw(e, _) => handler(e),
				})
			})
		}

		/// Dispatch a private `ArcRunExplicit` Catch carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Arc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Catch action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The recovered error type.",
			"The recovery handler type.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The private Catch layer carrying the handler and `ArcRunExplicit` carrier cell.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// let recover = |err: &'static str| err.len() as i32;
		/// let first = recover("boom") + 1;
		/// let second = recover("fail") + 1;
		/// assert_eq!((first, second), (5, 5));
		/// ```
		#[inline]
		pub(crate) fn dispatch_arc_run_explicit_catch_carrier<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			E,
			Handler,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitCatchCarrierLayer<
				'a,
				E,
				Handler,
				ArcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &'a (
			        impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>
			        + Send
			        + Sync
			        + 'a
			    ),
		) -> ArcRunExplicit<'a, R, S, Final>
		where
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			Action: Clone + Send + Sync + 'a,
			Final: Send + Sync + 'a,
			K: Fn(Action) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
			E: Send + Sync + 'a + 'static,
			Handler: Fn(E) -> ArcRunExplicit<'a, R, S, Action> + Clone + Send + Sync + 'a,
			FirstLayer: 'a,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			ExceptBrand<E>: Functor
				+ SendFunctor
				+ Kind_cdc7cd43dac7585f<
					Of<'a, ArcRunExplicit<'a, R, S, Action>> = Except<
						'a,
						E,
						ArcRunExplicit<'a, R, S, Action>,
					>,
				>,
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
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Action>,
			>): Send
				+ Sync
				+ Member<
					ArcCoyoneda<'a, ExceptBrand<E>, ArcRunExplicit<'a, R, S, Action>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
										'a,
										ArcRunExplicit<'a, R, S, Action>,
									>
								),
				>,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Action>,
			>): Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Action>,
			>): Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
					>),
					EmbedIndices,
				>,
			ArcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>: Clone
				+ ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = ArcRunExplicit<'a, R, S, Action>,
				> + ArcScopedResume<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>, {
			let (handler, continuation) = layer.into_parts();

			continuation.resume_arc_with_action_transform(fo_handlers, move |action| {
				let handler = handler.clone();
				action.interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| match op {
					Except::Throw(e, _) => handler(e),
				})
			})
		}
	}

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
	pub struct LocalDispatcher<Idx, RMinusE, EmbedIndices>(
		PhantomData<fn() -> (Idx, RMinusE, EmbedIndices)>,
	);

	/// Constructs a [`LocalDispatcher`] without naming its private field.
	#[document_examples]
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
	/// 		scoped_dispatchers::{
	/// 			local_dispatcher,
	/// 			ref_local_dispatcher,
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
	/// let result = program.interpret(
	/// 	handlers! {
	/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
	/// 			BoxReader::Ask(k) => k(10),
	/// 		},
	/// 	},
	/// 	scoped_handlers! {
	/// 		BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, FirstRowMinusReader, _>(),
	/// 		BoxRefLocalBrand<BoxBrand, i32>: ref_local_dispatcher::<_, FirstRowMinusReader, _>(),
	/// 	},
	/// );
	///
	/// assert_eq!(result, 22);
	/// ```
	pub const fn local_dispatcher<Idx, RMinusE, EmbedIndices>()
	-> LocalDispatcher<Idx, RMinusE, EmbedIndices> {
		LocalDispatcher(PhantomData)
	}

	/// Dispatcher for the standard `RefLocal` scoped effect.
	///
	/// The dispatcher asks the inherited Reader environment once, applies
	/// the stored by-reference environment transform, then answers Reader
	/// asks inside the action with clones of the modified environment.
	#[derive(Clone, Copy, Debug, Default)]
	#[expect(
		clippy::type_complexity,
		reason = "The fn marker carries row-witness type parameters without making auto-traits depend on them."
	)]
	pub struct RefLocalDispatcher<Idx, RMinusE, EmbedIndices>(
		PhantomData<fn() -> (Idx, RMinusE, EmbedIndices)>,
	);

	/// Constructs a [`RefLocalDispatcher`] without naming its private field.
	#[document_examples]
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
	/// 		scoped_dispatchers::{
	/// 			local_dispatcher,
	/// 			ref_local_dispatcher,
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
	/// let program: Prog = Run::ref_local::<i32, _>(|env| *env + 5, action);
	///
	/// let result = program.interpret(
	/// 	handlers! {
	/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
	/// 			BoxReader::Ask(k) => k(10),
	/// 		},
	/// 	},
	/// 	scoped_handlers! {
	/// 		BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, FirstRowMinusReader, _>(),
	/// 		BoxRefLocalBrand<BoxBrand, i32>: ref_local_dispatcher::<_, FirstRowMinusReader, _>(),
	/// 	},
	/// );
	///
	/// assert_eq!(result, 30);
	/// ```
	pub const fn ref_local_dispatcher<Idx, RMinusE, EmbedIndices>()
	-> RefLocalDispatcher<Idx, RMinusE, EmbedIndices> {
		RefLocalDispatcher(PhantomData)
	}

	#[document_type_parameters(
		"The row index witnessing the target Reader operation.",
		"The first-order row brand with the Reader operation removed.",
		"The row embedding witness used to rebuild the original row."
	)]
	#[document_parameters("The Local dispatcher receiver.")]
	#[allow(
		dead_code,
		reason = "Focused Local carrier methods are introduced before the wrapper interpreter route constructs these private layers."
	)]
	impl<Idx, RMinusE, EmbedIndices> LocalDispatcher<Idx, RMinusE, EmbedIndices> {
		/// Dispatch a private `RunExplicit` Local carrier-cell layer.
		///
		/// The dispatcher asks the inherited Reader environment, applies
		/// the stored by-value modifier, transforms the selected action so
		/// Reader asks inside that action see the local environment, then
		/// resumes the outer continuation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Local action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Reader environment type.",
			"The by-value environment modifier type.",
			"The first-order handler layer type."
		)]
		///
		#[document_parameters(
			"The private Local layer carrying the modifier and `RunExplicit` carrier cell.",
			"The first-order handler list available while resuming the selected action."
		)]
		///
		#[document_returns("The final `RunExplicit` program produced by the carrier.")]
		///
		#[document_examples]
		///
		/// ```
		/// let inherited_env = 10;
		/// let local_env = (|env| env + 1)(inherited_env);
		/// let action_result = local_env * 2;
		/// assert_eq!(action_result, 22);
		/// ```
		#[inline]
		pub(crate) fn dispatch_run_explicit_local_carrier<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			E,
			Modify,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitLocalCarrierLayer<
				'a,
				E,
				Modify,
				RunExplicitScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &'a (impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>> + 'a),
		) -> RunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Action: 'a,
			Final: 'a,
			K: Fn(Action) -> RunExplicit<'a, R, S, Final> + 'a,
			E: Clone + 'a + 'static,
			Modify: FnOnce(E) -> E + 'a,
			FirstLayer: 'a,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, E>):
				Member<Coyoneda<'a, BoxReaderBrand<BoxBrand, E>, E>, Idx>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RunExplicit<'a, R, S, Action>,
			>): Member<
					Coyoneda<'a, BoxReaderBrand<BoxBrand, E>, RunExplicit<'a, R, S, Action>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
										'a,
										RunExplicit<'a, R, S, Action>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				Box<FreeExplicit<'a, NodeBrand<R, S>, Action>>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						Box<FreeExplicit<'a, NodeBrand<R, S>, Action>>,
					>),
					EmbedIndices,
				>,
			RunExplicitScopedContinuation<'a, R, S, Action, Final, K>: ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = RunExplicit<'a, R, S, Action>,
				> + ExplicitScopedResume<'a, FirstLayer, RunExplicit<'a, R, S, Final>>, {
			let (modify, continuation) = layer.into_parts();
			let modify = std::cell::RefCell::new(Some(modify));
			let continuation = std::cell::RefCell::new(Some(continuation));

			RunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
				#[expect(
					clippy::expect_used,
					reason = "Box-backed Local carrier dispatch is single-shot; RunExplicit invokes this continuation once"
				)]
				let modify = modify
					.borrow_mut()
					.take()
					.expect("RunExplicit Local carrier modify invoked more than once");
				#[expect(
					clippy::expect_used,
					reason = "Box-backed Local carrier dispatch is single-shot; RunExplicit invokes this continuation once"
				)]
				let continuation = continuation
					.borrow_mut()
					.take()
					.expect("RunExplicit Local carrier continuation invoked more than once");
				let local_env = modify(env);

				continuation.resume_explicit_with_action_transform(fo_handlers, move |action| {
					let local_env = local_env.clone();
					action.interpose::<BoxReaderBrand<BoxBrand, E>, Idx, RMinusE, EmbedIndices>(
						move |op| match op {
							BoxReader::Ask(k) => k(local_env.clone()),
						},
					)
				})
			})
		}

		/// Dispatch a private `RcRunExplicit` Local carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Rc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Local action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Reader environment type.",
			"The by-value environment modifier type.",
			"The first-order handler layer type."
		)]
		///
		#[document_parameters(
			"The private Local layer carrying the modifier and `RcRunExplicit` carrier cell.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// let modify = |env| env + 1;
		/// assert_eq!(modify(10) * 2, 22);
		/// assert_eq!(modify(20) * 2, 42);
		/// ```
		#[inline]
		pub(crate) fn dispatch_rc_run_explicit_local_carrier<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			E,
			Modify,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitLocalCarrierLayer<
				'a,
				E,
				Modify,
				RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &'a (
			        impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>> + 'a
			    ),
		) -> RcRunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Action: Clone + 'a,
			Final: 'a,
			K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a,
			E: Clone + 'a + 'static,
			Modify: Fn(E) -> E + Clone + 'a,
			FirstLayer: 'a,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, E>):
				Member<RcCoyoneda<'a, ReaderBrand<RcBrand, E>, E>, Idx>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, Action>,
			>): Member<
					RcCoyoneda<'a, ReaderBrand<RcBrand, E>, RcRunExplicit<'a, R, S, Action>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
										'a,
										RcRunExplicit<'a, R, S, Action>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
					>),
					EmbedIndices,
				>,
			RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>: Clone
				+ ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = RcRunExplicit<'a, R, S, Action>,
				> + RcScopedResume<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>, {
			let (modify, continuation) = layer.into_parts();

			RcRunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
				let local_env = modify(env);
				let continuation = continuation.clone();

				continuation.resume_rc_with_action_transform(fo_handlers, move |action| {
					let local_env = local_env.clone();
					action.interpose::<ReaderBrand<RcBrand, E>, Idx, RMinusE, EmbedIndices>(
						move |op| match op {
							Reader::Ask(k) => k(local_env.clone()),
						},
					)
				})
			})
		}

		/// Dispatch a private `ArcRunExplicit` Local carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Arc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Local action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Reader environment type.",
			"The by-value environment modifier type.",
			"The first-order handler layer type."
		)]
		///
		#[document_parameters(
			"The private Local layer carrying the modifier and `ArcRunExplicit` carrier cell.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// let modify = |env| env + 1;
		/// let first = modify(10) * 2;
		/// let second = modify(20) * 2;
		/// assert_eq!((first, second), (22, 42));
		/// ```
		#[inline]
		pub(crate) fn dispatch_arc_run_explicit_local_carrier<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			E,
			Modify,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitLocalCarrierLayer<
				'a,
				E,
				Modify,
				ArcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &'a (
			        impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>
			        + Send
			        + Sync
			        + 'a
			    ),
		) -> ArcRunExplicit<'a, R, S, Final>
		where
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			Action: Clone + Send + Sync + 'a,
			Final: Send + Sync + 'a,
			K: Fn(Action) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
			E: Clone + Send + Sync + 'a + 'static,
			Modify: Fn(E) -> E + Clone + Send + Sync + 'a,
			FirstLayer: 'a,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			SendReaderBrand<ArcBrand, E>: SendFunctor
				+ Kind_cdc7cd43dac7585f<
					Of<'a, ArcRunExplicit<'a, R, S, Action>> = SendReader<
						'a,
						ArcBrand,
						E,
						ArcRunExplicit<'a, R, S, Action>,
					>,
				>,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Clone + Send + Sync,
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
				ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Action>,
			>): Send
				+ Sync
				+ Member<
					ArcCoyoneda<'a, SendReaderBrand<ArcBrand, E>, ArcRunExplicit<'a, R, S, Action>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
										'a,
										ArcRunExplicit<'a, R, S, Action>,
									>
								),
				>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, E>):
				Member<ArcCoyoneda<'a, SendReaderBrand<ArcBrand, E>, E>, Idx>,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Action>,
			>): Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Action>,
			>): Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
					>),
					EmbedIndices,
				>,
			ArcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>: Clone
				+ ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = ArcRunExplicit<'a, R, S, Action>,
				> + ArcScopedResume<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>, {
			let (modify, continuation) = layer.into_parts();

			ArcRunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
				let local_env = modify(env);
				let continuation = continuation.clone();

				continuation.resume_arc_with_action_transform(fo_handlers, move |action| {
					let local_env = local_env.clone();
					action.interpose::<SendReaderBrand<ArcBrand, E>, Idx, RMinusE, EmbedIndices>(
						move |op| match op {
							SendReader::Ask(k) => k(local_env.clone()),
						},
					)
				})
			})
		}
	}

	#[document_type_parameters(
		"The row index witnessing the target Reader operation.",
		"The first-order row brand with the Reader operation removed.",
		"The row embedding witness used to rebuild the original row."
	)]
	#[document_parameters("The RefLocal dispatcher receiver.")]
	#[allow(
		dead_code,
		reason = "Focused RefLocal carrier methods are introduced before the wrapper interpreter route constructs these private layers."
	)]
	impl<Idx, RMinusE, EmbedIndices> RefLocalDispatcher<Idx, RMinusE, EmbedIndices> {
		/// Dispatch a private `RunExplicit` RefLocal carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected RefLocal action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Reader environment type.",
			"The borrow-based environment modifier type.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The private RefLocal layer carrying the modifier and `RunExplicit` carrier cell.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// let inherited_env = 10;
		/// let local_env = (|env: &i32| *env + 5)(&inherited_env);
		/// assert_eq!(local_env * 2, 30);
		/// ```
		#[inline]
		pub(crate) fn dispatch_run_explicit_ref_local_carrier<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			E,
			Modify,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitRefLocalCarrierLayer<
				'a,
				E,
				Modify,
				RunExplicitScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &'a (impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>> + 'a),
		) -> RunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Action: 'a,
			Final: 'a,
			K: Fn(Action) -> RunExplicit<'a, R, S, Final> + 'a,
			E: Clone + 'a + 'static,
			Modify: FnOnce(&E) -> E + 'a,
			FirstLayer: 'a,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, E>):
				Member<Coyoneda<'a, BoxReaderBrand<BoxBrand, E>, E>, Idx>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RunExplicit<'a, R, S, Action>,
			>): Member<
					Coyoneda<'a, BoxReaderBrand<BoxBrand, E>, RunExplicit<'a, R, S, Action>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
										'a,
										RunExplicit<'a, R, S, Action>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				Box<FreeExplicit<'a, NodeBrand<R, S>, Action>>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						Box<FreeExplicit<'a, NodeBrand<R, S>, Action>>,
					>),
					EmbedIndices,
				>,
			RunExplicitScopedContinuation<'a, R, S, Action, Final, K>: ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = RunExplicit<'a, R, S, Action>,
				> + ExplicitScopedResume<'a, FirstLayer, RunExplicit<'a, R, S, Final>>, {
			let (modify, continuation) = layer.into_parts();
			let modify = std::cell::RefCell::new(Some(modify));
			let continuation = std::cell::RefCell::new(Some(continuation));

			RunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
				#[expect(
					clippy::expect_used,
					reason = "Box-backed RefLocal carrier dispatch is single-shot; RunExplicit invokes this continuation once"
				)]
				let modify = modify
					.borrow_mut()
					.take()
					.expect("RunExplicit RefLocal carrier modify invoked more than once");
				#[expect(
					clippy::expect_used,
					reason = "Box-backed RefLocal carrier dispatch is single-shot; RunExplicit invokes this continuation once"
				)]
				let continuation = continuation
					.borrow_mut()
					.take()
					.expect("RunExplicit RefLocal carrier continuation invoked more than once");
				let local_env = modify(&env);

				continuation.resume_explicit_with_action_transform(fo_handlers, move |action| {
					let local_env = local_env.clone();
					action.interpose::<BoxReaderBrand<BoxBrand, E>, Idx, RMinusE, EmbedIndices>(
						move |op| match op {
							BoxReader::Ask(k) => k(local_env.clone()),
						},
					)
				})
			})
		}

		/// Dispatch a private `RcRunExplicit` RefLocal carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Rc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected RefLocal action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Reader environment type.",
			"The borrow-based environment modifier type.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The private RefLocal layer carrying the modifier and `RcRunExplicit` carrier cell.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// let modify = |env: &i32| *env + 5;
		/// assert_eq!(modify(&10) * 2, 30);
		/// assert_eq!(modify(&20) * 2, 50);
		/// ```
		#[inline]
		pub(crate) fn dispatch_rc_run_explicit_ref_local_carrier<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			E,
			Modify,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitRefLocalCarrierLayer<
				'a,
				E,
				Modify,
				RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &'a (
			        impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>> + 'a
			    ),
		) -> RcRunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Action: Clone + 'a,
			Final: 'a,
			K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a,
			E: Clone + 'a + 'static,
			Modify: Fn(&E) -> E + Clone + 'a,
			FirstLayer: 'a,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, E>):
				Member<RcCoyoneda<'a, ReaderBrand<RcBrand, E>, E>, Idx>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, Action>,
			>): Member<
					RcCoyoneda<'a, ReaderBrand<RcBrand, E>, RcRunExplicit<'a, R, S, Action>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
										'a,
										RcRunExplicit<'a, R, S, Action>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
					>),
					EmbedIndices,
				>,
			RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>: Clone
				+ ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = RcRunExplicit<'a, R, S, Action>,
				> + RcScopedResume<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>, {
			let (modify, continuation) = layer.into_parts();

			RcRunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
				let local_env = modify(&env);
				let continuation = continuation.clone();

				continuation.resume_rc_with_action_transform(fo_handlers, move |action| {
					let local_env = local_env.clone();
					action.interpose::<ReaderBrand<RcBrand, E>, Idx, RMinusE, EmbedIndices>(
						move |op| match op {
							Reader::Ask(k) => k(local_env.clone()),
						},
					)
				})
			})
		}

		/// Dispatch a private `ArcRunExplicit` RefLocal carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Arc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected RefLocal action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Reader environment type.",
			"The borrow-based environment modifier type.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The private RefLocal layer carrying the modifier and `ArcRunExplicit` carrier cell.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// let modify = |env: &i32| *env + 5;
		/// let first = modify(&10) * 2;
		/// let second = modify(&20) * 2;
		/// assert_eq!((first, second), (30, 50));
		/// ```
		#[inline]
		pub(crate) fn dispatch_arc_run_explicit_ref_local_carrier<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			E,
			Modify,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitRefLocalCarrierLayer<
				'a,
				E,
				Modify,
				ArcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &'a (
			        impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>
			        + Send
			        + Sync
			        + 'a
			    ),
		) -> ArcRunExplicit<'a, R, S, Final>
		where
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			Action: Clone + Send + Sync + 'a,
			Final: Send + Sync + 'a,
			K: Fn(Action) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
			E: Clone + Send + Sync + 'a + 'static,
			Modify: Fn(&E) -> E + Clone + Send + Sync + 'a,
			FirstLayer: 'a,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			SendReaderBrand<ArcBrand, E>: SendFunctor
				+ Kind_cdc7cd43dac7585f<
					Of<'a, ArcRunExplicit<'a, R, S, Action>> = SendReader<
						'a,
						ArcBrand,
						E,
						ArcRunExplicit<'a, R, S, Action>,
					>,
				>,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Clone + Send + Sync,
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
				ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Action>,
			>): Send
				+ Sync
				+ Member<
					ArcCoyoneda<'a, SendReaderBrand<ArcBrand, E>, ArcRunExplicit<'a, R, S, Action>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
										'a,
										ArcRunExplicit<'a, R, S, Action>,
									>
								),
				>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, E>):
				Member<ArcCoyoneda<'a, SendReaderBrand<ArcBrand, E>, E>, Idx>,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Action>,
			>): Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Action>,
			>): Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
					>),
					EmbedIndices,
				>,
			ArcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>: Clone
				+ ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = ArcRunExplicit<'a, R, S, Action>,
				> + ArcScopedResume<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>, {
			let (modify, continuation) = layer.into_parts();

			ArcRunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
				let local_env = modify(&env);
				let continuation = continuation.clone();

				continuation.resume_arc_with_action_transform(fo_handlers, move |action| {
					let local_env = local_env.clone();
					action.interpose::<SendReaderBrand<ArcBrand, E>, Idx, RMinusE, EmbedIndices>(
						move |op| match op {
							SendReader::Ask(k) => k(local_env.clone()),
						},
					)
				})
			})
		}
	}

	/// Dispatcher for the standard `Span` scoped effect.
	///
	/// The dispatcher consumes the by-value tag and resumes the stored
	/// action program unchanged.
	#[derive(Clone, Copy, Debug, Default)]
	pub struct SpanDispatcher;

	/// Constructs a [`SpanDispatcher`].
	#[document_examples]
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
	/// 		scoped_dispatchers::span_dispatcher,
	/// 	},
	/// };
	///
	/// type FirstRow = CNilBrand;
	/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
	/// type Prog = Run<FirstRow, ScopedRow, i32>;
	///
	/// let program: Prog = Run::span::<&'static str, _>("request", Run::pure(42));
	///
	/// let result = program.interpret(
	/// 	handlers! {},
	/// 	scoped_handlers! {
	/// 		BoxSpanBrand<BoxBrand, &'static str>: span_dispatcher(),
	/// 	},
	/// );
	///
	/// assert_eq!(result, 42);
	/// ```
	pub const fn span_dispatcher() -> SpanDispatcher {
		SpanDispatcher
	}

	#[document_parameters("The Span dispatcher receiver.")]
	#[cfg_attr(
		not(test),
		expect(
			dead_code,
			reason = "The focused RunExplicit Span carrier-cell proof is exercised by tests before the full wrapper interpreter route consumes it in step 7.4.4c."
		)
	)]
	impl SpanDispatcher {
		/// Dispatch a private `RunExplicit` Span carrier-cell layer.
		///
		/// This focused proof path consumes the Span tag together with
		/// the wrapper-owned carrier cell. The `post_action` callback
		/// receives the tag and selected action value, returns the
		/// result-preserving action program, and runs before the carrier
		/// resumes the selected action's outer continuation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Span action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Span tag type.",
			"The first-order handler layer type."
		)]
		///
		#[document_parameters(
			"The private Span layer carrying the tag and `RunExplicit` carrier cell.",
			"The first-order handler list available while resuming the selected action.",
			"The result-preserving action callback to run before the outer continuation."
		)]
		///
		#[document_returns("The final `RunExplicit` program produced by the carrier.")]
		///
		#[document_examples]
		///
		/// ```
		/// struct LocalSpanLayer<Tag, Carrier> {
		/// 	tag: Tag,
		/// 	carrier: Carrier,
		/// }
		///
		/// impl<Tag, Carrier> LocalSpanLayer<Tag, Carrier> {
		/// 	fn dispatch(
		/// 		self,
		/// 		post_action: impl Fn(&Tag, Carrier) -> Carrier,
		/// 	) -> Carrier {
		/// 		post_action(&self.tag, self.carrier)
		/// 	}
		/// }
		///
		/// let result = LocalSpanLayer {
		/// 	tag: "request",
		/// 	carrier: 41,
		/// }
		/// .dispatch(|tag, value| {
		/// 	assert_eq!(*tag, "request");
		/// 	value + 1
		/// });
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		pub(crate) fn dispatch_run_explicit_span_carrier_with_post_action<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			Tag,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitSpanCarrierLayer<
				'a,
				Tag,
				RunExplicitScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
			post_action: impl Fn(&Tag, Action) -> RunExplicit<'a, R, S, Action> + 'a,
		) -> RunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Action: 'a,
			Final: 'a,
			K: Fn(Action) -> RunExplicit<'a, R, S, Final> + 'a,
			Tag: 'a,
			FirstLayer: 'a,
			RunExplicitScopedContinuation<'a, R, S, Action, Final, K>: ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = RunExplicit<'a, R, S, Action>,
				> + ExplicitScopedResume<'a, FirstLayer, RunExplicit<'a, R, S, Final>>, {
			let (tag, continuation) = layer.into_parts();

			continuation.resume_explicit_with_post_action(fo_handlers, move |action_value| {
				post_action(&tag, action_value)
			})
		}

		/// Dispatch a private `RcRunExplicit` Span carrier-cell layer.
		///
		/// This is the shared-Rc counterpart to
		/// [`dispatch_run_explicit_span_carrier_with_post_action`](SpanDispatcher::dispatch_run_explicit_span_carrier_with_post_action).
		/// It consumes one carrier layer, observes the tag, inserts
		/// result-preserving post-action work, and then resumes the
		/// `RcRunExplicit` outer continuation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Rc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Span action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Span tag type.",
			"The first-order handler layer type."
		)]
		///
		#[document_parameters(
			"The private Span layer carrying the tag and `RcRunExplicit` carrier cell.",
			"The first-order handler list available while resuming the selected action.",
			"The result-preserving action callback to run before the outer continuation."
		)]
		///
		#[document_returns("The final `RcRunExplicit` program produced by the carrier.")]
		///
		#[document_examples]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// let tag = Rc::new("request");
		/// let post = {
		/// 	let tag = Rc::clone(&tag);
		/// 	move |value: i32| {
		/// 		assert_eq!(*tag, "request");
		/// 		value + 1
		/// 	}
		/// };
		///
		/// assert_eq!(post(41), 42);
		/// ```
		#[inline]
		pub(crate) fn dispatch_rc_run_explicit_span_carrier_with_post_action<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			Tag,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitSpanCarrierLayer<
				'a,
				Tag,
				RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
			post_action: impl Fn(&Tag, Action) -> RcRunExplicit<'a, R, S, Action> + 'a,
		) -> RcRunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Action: Clone + 'a,
			Final: 'a,
			K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a,
			Tag: 'a,
			FirstLayer: 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone,
			RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>: ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = RcRunExplicit<'a, R, S, Action>,
				> + RcScopedResume<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>, {
			let (tag, continuation) = layer.into_parts();

			continuation.resume_rc_with_post_action(fo_handlers, move |action_value| {
				post_action(&tag, action_value)
			})
		}

		/// Dispatch a private `ArcRunExplicit` Span carrier-cell layer.
		///
		/// This is the thread-safe shared counterpart to the single-shot
		/// `RunExplicit` proof. The post-action callback must be
		/// `Send + Sync`, matching the `ArcRunExplicit` carrier contract.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Arc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Span action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Span tag type.",
			"The first-order handler layer type."
		)]
		///
		#[document_parameters(
			"The private Span layer carrying the tag and `ArcRunExplicit` carrier cell.",
			"The first-order handler list available while resuming the selected action.",
			"The result-preserving action callback to run before the outer continuation."
		)]
		///
		#[document_returns("The final `ArcRunExplicit` program produced by the carrier.")]
		///
		#[document_examples]
		///
		/// ```
		/// use std::sync::Arc;
		///
		/// let tag = Arc::new("request");
		/// let post = {
		/// 	let tag = Arc::clone(&tag);
		/// 	move |value: i32| {
		/// 		assert_eq!(*tag, "request");
		/// 		value + 1
		/// 	}
		/// };
		///
		/// assert_eq!(post(41), 42);
		/// ```
		#[inline]
		pub(crate) fn dispatch_arc_run_explicit_span_carrier_with_post_action<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			Tag,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitSpanCarrierLayer<
				'a,
				Tag,
				ArcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
			post_action: impl Fn(&Tag, Action) -> ArcRunExplicit<'a, R, S, Action> + Send + Sync + 'a,
		) -> ArcRunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + SendFunctor + 'static,
			S: WrapDrop + SendFunctor + 'static,
			Action: Clone + Send + Sync + 'a,
			Final: Send + Sync + 'a,
			K: Fn(Action) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
			Tag: Send + Sync + 'a,
			FirstLayer: 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone + Send + Sync,
			ArcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>: ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = ArcRunExplicit<'a, R, S, Action>,
				> + ArcScopedResume<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>, {
			let (tag, continuation) = layer.into_parts();

			continuation.resume_arc_with_post_action(fo_handlers, move |action_value| {
				post_action(&tag, action_value)
			})
		}
	}

	/// Dispatcher for the standard `Bracket` scoped effect.
	///
	/// The dispatcher runs acquire, passes the acquired resource to the
	/// body, runs the effectful release program on the normal path, and
	/// returns the body result after release completes. During unwinding it
	/// relies only on ordinary Rust `Drop` for the resource; the effectful
	/// release program is not interpreted from `Drop`.
	#[derive(Clone, Copy, Debug, Default)]
	pub struct BracketDispatcher;

	/// Constructs a [`BracketDispatcher`].
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	fp_library::{
	/// 		Apply,
	/// 		brands::{
	/// 			BracketBrand,
	/// 			CNilBrand,
	/// 			CoproductBrand,
	/// 			NodeBrand,
	/// 			RcBrand,
	/// 		},
	/// 		classes::{
	/// 			Functor,
	/// 			WrapDrop,
	/// 		},
	/// 		handlers,
	/// 		impl_kind,
	/// 		kinds::*,
	/// 		scoped_handlers,
	/// 		types::effects::{
	/// 			rc_run::RcRun,
	/// 			scoped_dispatchers::bracket_dispatcher,
	/// 		},
	/// 	},
	/// 	std::{
	/// 		cell::Cell,
	/// 		rc::Rc,
	/// 	},
	/// };
	///
	/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	/// struct ScopedRow;
	///
	/// type FirstRow = CNilBrand;
	/// type UnderlyingRow =
	/// 	CoproductBrand<BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>, CNilBrand>;
	/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
	///
	/// impl_kind! {
	/// 	impl for ScopedRow {
	/// 		type Of<'a, A: 'a>: 'a =
	/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	/// 	}
	/// }
	///
	/// impl WrapDrop for ScopedRow {
	/// 	fn drop<'a, X: 'a>(
	/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	/// 	) -> Option<X> {
	/// 		<UnderlyingRow as WrapDrop>::drop(fa)
	/// 	}
	/// }
	///
	/// impl Functor for ScopedRow {
	/// 	fn map<'a, A: 'a, B: 'a>(
	/// 		f: impl Fn(A) -> B + 'a,
	/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
	/// 		<UnderlyingRow as Functor>::map(f, fa)
	/// 	}
	/// }
	///
	/// let released = Rc::new(Cell::new(false));
	/// let released_in_cleanup = Rc::clone(&released);
	/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::bracket::<i32, _>(
	/// 	RcRun::pure(7),
	/// 	|resource: Rc<i32>| RcRun::pure((*resource, *resource + 35)),
	/// 	move |_resource: Rc<i32>| {
	/// 		released_in_cleanup.set(true);
	/// 		RcRun::pure(())
	/// 	},
	/// );
	///
	/// let result = program.interpret(
	/// 	handlers! {},
	/// 	scoped_handlers! {
	/// 		BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: bracket_dispatcher(),
	/// 	},
	/// );
	///
	/// assert_eq!(result, 42);
	/// assert!(released.get());
	/// ```
	pub const fn bracket_dispatcher() -> BracketDispatcher {
		BracketDispatcher
	}

	/// Dispatcher for the standard `RefBracket` scoped effect.
	///
	/// The dispatcher runs acquire, stores the resource in a refcounted
	/// pointer, passes pointer clones to body and release, runs the
	/// effectful release program on the normal path, and returns the body
	/// result after release completes. During unwinding it relies only on
	/// ordinary Rust `Drop` for the refcounted resource.
	#[derive(Clone, Copy, Debug, Default)]
	pub struct RefBracketDispatcher;

	/// Constructs a [`RefBracketDispatcher`].
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	fp_library::{
	/// 		Apply,
	/// 		brands::{
	/// 			CNilBrand,
	/// 			CoproductBrand,
	/// 			NodeBrand,
	/// 			RcBrand,
	/// 			RefBracketBrand,
	/// 		},
	/// 		classes::{
	/// 			Functor,
	/// 			WrapDrop,
	/// 		},
	/// 		handlers,
	/// 		impl_kind,
	/// 		kinds::*,
	/// 		scoped_handlers,
	/// 		types::effects::{
	/// 			rc_run::RcRun,
	/// 			scoped_dispatchers::ref_bracket_dispatcher,
	/// 		},
	/// 	},
	/// 	std::{
	/// 		cell::Cell,
	/// 		rc::Rc,
	/// 	},
	/// };
	///
	/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	/// struct ScopedRow;
	///
	/// type FirstRow = CNilBrand;
	/// type UnderlyingRow = CoproductBrand<
	/// 	RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
	/// 	CNilBrand,
	/// >;
	/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
	///
	/// impl_kind! {
	/// 	impl for ScopedRow {
	/// 		type Of<'a, A: 'a>: 'a =
	/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	/// 	}
	/// }
	///
	/// impl WrapDrop for ScopedRow {
	/// 	fn drop<'a, X: 'a>(
	/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	/// 	) -> Option<X> {
	/// 		<UnderlyingRow as WrapDrop>::drop(fa)
	/// 	}
	/// }
	///
	/// impl Functor for ScopedRow {
	/// 	fn map<'a, A: 'a, B: 'a>(
	/// 		f: impl Fn(A) -> B + 'a,
	/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
	/// 		<UnderlyingRow as Functor>::map(f, fa)
	/// 	}
	/// }
	///
	/// let observed = Rc::new(Cell::new(0));
	/// let released = Rc::new(Cell::new(false));
	/// let observed_in_body = Rc::clone(&observed);
	/// let released_in_cleanup = Rc::clone(&released);
	/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
	/// 	RcRun::pure(7),
	/// 	move |resource: Rc<i32>| {
	/// 		observed_in_body.set(*resource);
	/// 		RcRun::pure(*resource + 35)
	/// 	},
	/// 	move |resource: Rc<i32>| {
	/// 		released_in_cleanup.set(*resource == 7);
	/// 		RcRun::pure(())
	/// 	},
	/// );
	///
	/// let result = program.interpret(
	/// 	handlers! {},
	/// 	scoped_handlers! {
	/// 		RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: ref_bracket_dispatcher(),
	/// 	},
	/// );
	///
	/// assert_eq!(result, 42);
	/// assert_eq!(observed.get(), 7);
	/// assert!(released.get());
	/// ```
	pub const fn ref_bracket_dispatcher() -> RefBracketDispatcher {
		RefBracketDispatcher
	}

	#[document_parameters("The Bracket dispatcher receiver.")]
	#[allow(
		dead_code,
		reason = "Focused Bracket carrier methods are introduced before the wrapper interpreter route constructs these private layers."
	)]
	impl BracketDispatcher {
		/// Dispatch a private `RunExplicit` Bracket carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The acquire lifecycle cell type.",
			"The body lifecycle cell type.",
			"The release lifecycle cell type.",
			"The first-order handler layer type."
		)]
		///
		#[document_parameters(
			"The private Bracket layer carrying acquire, body, release, and the `RunExplicit` lifecycle carrier cell.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// let acquire = || 7;
		/// let body = |resource: Box<i32>| (*resource, *resource + 35);
		/// let release = |resource: Box<i32>| *resource == 7;
		/// let resource = acquire();
		/// let (resource, body_result) = body(Box::new(resource));
		/// assert!(release(Box::new(resource)));
		/// assert_eq!(body_result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "The private Bracket carrier signature must keep the wrapper, lifecycle, resource, and continuation types explicit."
		)]
		pub(crate) fn dispatch_run_explicit_bracket_carrier<
			'a,
			R,
			S,
			Resource,
			BodyResult,
			Final,
			K,
			Acquire,
			BodyFn,
			Release,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitBracketCarrierLayer<
				'a,
				BoxBrand,
				Resource,
				BodyResult,
				Acquire,
				BodyFn,
				Release,
				RunExplicitLifecycleScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
		) -> RunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Resource: 'a,
			BodyResult: 'a,
			Final: 'a,
			K: Fn(BodyResult) -> RunExplicit<'a, R, S, Final> + 'a,
			Acquire: FnOnce() -> RunExplicit<'a, R, S, Resource> + 'a,
			BodyFn: FnOnce(Box<Resource>) -> RunExplicit<'a, R, S, (Resource, BodyResult)> + 'a,
			Release: FnOnce(Box<Resource>) -> RunExplicit<'a, R, S, ()> + 'a,
			FirstLayer: 'a,
			RunExplicitLifecycleScopedContinuation<'a, R, S, BodyResult, Final, K>:
				ScopedResumeTypes<
						'a,
						ActionValue = BodyResult,
						ActionProgram = RunExplicit<'a, R, S, BodyResult>,
					> + ExplicitLifecycleScopedResume<'a, FirstLayer, RunExplicit<'a, R, S, Final>>, {
			let (acquire, body, release, continuation) = layer.into_parts();
			let body = std::cell::RefCell::new(Some(body));
			let release = Rc::new(std::cell::RefCell::new(Some(release)));

			continuation.resume_explicit_with_lifecycle_action(fo_handlers, move || {
				acquire().bind(move |resource| {
					#[expect(
						clippy::expect_used,
						reason = "Box-backed Bracket carrier body is single-shot; RunExplicit invokes this continuation once"
					)]
					let body = body
						.borrow_mut()
						.take()
						.expect("RunExplicit Bracket carrier body invoked more than once");
					let release = Rc::clone(&release);
					body(Box::new(resource)).bind(move |(resource, body_result)| {
						#[expect(
							clippy::expect_used,
							reason = "Box-backed Bracket carrier release is single-shot; RunExplicit invokes this continuation once"
						)]
						let release = release
							.borrow_mut()
							.take()
							.expect("RunExplicit Bracket carrier release invoked more than once");
						let body_result = std::cell::RefCell::new(Some(body_result));
						release(Box::new(resource)).bind(move |()| {
							#[expect(
								clippy::expect_used,
								reason = "Box-backed Bracket carrier result is single-shot; RunExplicit invokes this continuation once"
							)]
							RunExplicit::pure(body_result.borrow_mut().take().expect(
								"RunExplicit Bracket carrier result returned more than once",
							))
						})
					})
				})
			})
		}

		/// Dispatch a private `RcRunExplicit` Bracket carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Rc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The acquire lifecycle cell type.",
			"The body lifecycle cell type.",
			"The release lifecycle cell type.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The private Bracket layer carrying acquire, body, release, and the `RcRunExplicit` lifecycle carrier cell.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// let resource = 7;
		/// let (resource, body_result) =
		/// 	(|resource: Rc<i32>| (*resource, *resource + 35))(Rc::new(resource));
		/// assert!((|resource: Rc<i32>| *resource == 7)(Rc::new(resource)));
		/// assert_eq!(body_result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "The private Bracket carrier signature must keep the wrapper, lifecycle, resource, and continuation types explicit."
		)]
		pub(crate) fn dispatch_rc_run_explicit_bracket_carrier<
			'a,
			R,
			S,
			Resource,
			BodyResult,
			Final,
			K,
			Acquire,
			BodyFn,
			Release,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitBracketCarrierLayer<
				'a,
				RcBrand,
				Resource,
				BodyResult,
				Acquire,
				BodyFn,
				Release,
				RcRunExplicitLifecycleScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
		) -> RcRunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Resource: Clone + 'a,
			BodyResult: Clone + 'a,
			Final: 'a,
			K: Fn(BodyResult) -> RcRunExplicit<'a, R, S, Final> + 'a,
			Acquire: Fn() -> RcRunExplicit<'a, R, S, Resource> + 'a,
			BodyFn:
				Fn(Rc<Resource>) -> RcRunExplicit<'a, R, S, (Resource, BodyResult)> + Clone + 'a,
			Release: Fn(Rc<Resource>) -> RcRunExplicit<'a, R, S, ()> + Clone + 'a,
			FirstLayer: 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (Resource, BodyResult)>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, ()>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone,
			RcRunExplicitLifecycleScopedContinuation<'a, R, S, BodyResult, Final, K>:
				ScopedResumeTypes<
						'a,
						ActionValue = BodyResult,
						ActionProgram = RcRunExplicit<'a, R, S, BodyResult>,
					> + RcLifecycleScopedResume<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>, {
			let (acquire, body, release, continuation) = layer.into_parts();

			continuation.resume_rc_with_lifecycle_action(fo_handlers, move || {
				acquire().bind(move |resource| {
					let body = body.clone();
					let release = release.clone();
					body(Rc::new(resource)).bind(move |(resource, body_result)| {
						release(Rc::new(resource)).map(move |()| body_result.clone())
					})
				})
			})
		}

		/// Dispatch a private `ArcRunExplicit` Bracket carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Arc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The acquire lifecycle cell type.",
			"The body lifecycle cell type.",
			"The release lifecycle cell type.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The private Bracket layer carrying acquire, body, release, and the `ArcRunExplicit` lifecycle carrier cell.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// use std::sync::Arc;
		///
		/// let resource = 7;
		/// let (resource, body_result) =
		/// 	(|resource: Arc<i32>| (*resource, *resource + 35))(Arc::new(resource));
		/// assert!((|resource: Arc<i32>| *resource == 7)(Arc::new(resource)));
		/// assert_eq!(body_result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "The private Bracket carrier signature must keep the wrapper, lifecycle, resource, and continuation types explicit."
		)]
		pub(crate) fn dispatch_arc_run_explicit_bracket_carrier<
			'a,
			R,
			S,
			Resource,
			BodyResult,
			Final,
			K,
			Acquire,
			BodyFn,
			Release,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitBracketCarrierLayer<
				'a,
				ArcBrand,
				Resource,
				BodyResult,
				Acquire,
				BodyFn,
				Release,
				ArcRunExplicitLifecycleScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
		) -> ArcRunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + SendFunctor + 'static,
			S: WrapDrop + SendFunctor + 'static,
			Resource: Clone + Send + Sync + 'a,
			BodyResult: Clone + Send + Sync + 'a,
			Final: Send + Sync + 'a,
			K: Fn(BodyResult) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
			Acquire: Fn() -> ArcRunExplicit<'a, R, S, Resource> + Send + Sync + 'a,
			BodyFn: Fn(Arc<Resource>) -> ArcRunExplicit<'a, R, S, (Resource, BodyResult)>
				+ Clone
				+ Send
				+ Sync
				+ 'a,
			Release: Fn(Arc<Resource>) -> ArcRunExplicit<'a, R, S, ()> + Clone + Send + Sync + 'a,
			FirstLayer: 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, (Resource, BodyResult)>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, ()>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone + Send + Sync,
			ArcRunExplicitLifecycleScopedContinuation<'a, R, S, BodyResult, Final, K>:
				ScopedResumeTypes<
						'a,
						ActionValue = BodyResult,
						ActionProgram = ArcRunExplicit<'a, R, S, BodyResult>,
					> + ArcLifecycleScopedResume<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>, {
			let (acquire, body, release, continuation) = layer.into_parts();

			continuation.resume_arc_with_lifecycle_action(fo_handlers, move || {
				acquire().bind(move |resource| {
					let body = body.clone();
					let release = release.clone();
					body(Arc::new(resource)).bind(move |(resource, body_result)| {
						release(Arc::new(resource)).map(move |()| body_result.clone())
					})
				})
			})
		}
	}

	#[document_parameters("The RefBracket dispatcher receiver.")]
	#[allow(
		dead_code,
		reason = "Focused RefBracket carrier methods are introduced before the wrapper interpreter route constructs these private layers."
	)]
	impl RefBracketDispatcher {
		/// Dispatch a private `RunExplicit` RefBracket carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The acquire lifecycle cell type.",
			"The body lifecycle cell type.",
			"The release lifecycle cell type.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The private RefBracket layer carrying acquire, body, release, and the `RunExplicit` lifecycle carrier cell.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// let acquire = || 7;
		/// let body = |resource: Rc<i32>| {
		/// 	assert_eq!(Rc::strong_count(&resource), 2);
		/// 	*resource + 35
		/// };
		/// let release = |resource: Rc<i32>| {
		/// 	assert_eq!(Rc::strong_count(&resource), 1);
		/// 	*resource == 7
		/// };
		/// let resource = Rc::new(acquire());
		/// let release_resource = Rc::clone(&resource);
		/// let body_result = body(resource);
		/// assert!(release(release_resource));
		/// assert_eq!(body_result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "The private RefBracket carrier signature must keep the wrapper, lifecycle, pointer, resource, and continuation types explicit."
		)]
		pub(crate) fn dispatch_run_explicit_ref_bracket_carrier<
			'a,
			R,
			S,
			Resource,
			BodyResult,
			Final,
			K,
			Acquire,
			BodyFn,
			Release,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitRefBracketCarrierLayer<
				'a,
				RcBrand,
				Resource,
				BodyResult,
				Acquire,
				BodyFn,
				Release,
				RunExplicitLifecycleScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
		) -> RunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Resource: 'a,
			BodyResult: 'a,
			Final: 'a,
			K: Fn(BodyResult) -> RunExplicit<'a, R, S, Final> + 'a,
			Acquire: FnOnce() -> RunExplicit<'a, R, S, Resource> + 'a,
			BodyFn: FnOnce(Rc<Resource>) -> RunExplicit<'a, R, S, BodyResult> + 'a,
			Release: FnOnce(Rc<Resource>) -> RunExplicit<'a, R, S, ()> + 'a,
			FirstLayer: 'a,
			RunExplicitLifecycleScopedContinuation<'a, R, S, BodyResult, Final, K>:
				ScopedResumeTypes<
						'a,
						ActionValue = BodyResult,
						ActionProgram = RunExplicit<'a, R, S, BodyResult>,
					> + ExplicitLifecycleScopedResume<'a, FirstLayer, RunExplicit<'a, R, S, Final>>, {
			let (acquire, body, release, continuation) = layer.into_parts();
			let body = std::cell::RefCell::new(Some(body));
			let release = Rc::new(std::cell::RefCell::new(Some(release)));

			continuation.resume_explicit_with_lifecycle_action(fo_handlers, move || {
				acquire().bind(move |resource| {
					let resource = Rc::new(resource);
					let release_resource = Rc::clone(&resource);
					let release_resource = std::cell::RefCell::new(Some(release_resource));
					#[expect(
						clippy::expect_used,
						reason = "Box-backed RefBracket carrier body is single-shot; RunExplicit invokes this continuation once"
					)]
					let body = body
						.borrow_mut()
						.take()
						.expect("RunExplicit RefBracket carrier body invoked more than once");
					let release = Rc::clone(&release);
					body(resource).bind(move |body_result| {
						#[expect(
							clippy::expect_used,
							reason = "Box-backed RefBracket carrier release is single-shot; RunExplicit invokes this continuation once"
						)]
						let release = release
							.borrow_mut()
							.take()
							.expect("RunExplicit RefBracket carrier release invoked more than once");
						#[expect(
							clippy::expect_used,
							reason = "Box-backed RefBracket carrier release pointer is single-shot; RunExplicit invokes this continuation once"
						)]
						let release_resource = release_resource
							.borrow_mut()
							.take()
							.expect("RunExplicit RefBracket carrier release pointer used more than once");
						let body_result = std::cell::RefCell::new(Some(body_result));
						release(release_resource).bind(move |()| {
							#[expect(
								clippy::expect_used,
								reason = "Box-backed RefBracket carrier result is single-shot; RunExplicit invokes this continuation once"
							)]
							RunExplicit::pure(body_result.borrow_mut().take().expect(
								"RunExplicit RefBracket carrier result returned more than once",
							))
						})
					})
				})
			})
		}

		/// Dispatch a private `RcRunExplicit` RefBracket carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Rc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The acquire lifecycle cell type.",
			"The body lifecycle cell type.",
			"The release lifecycle cell type.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The private RefBracket layer carrying acquire, body, release, and the `RcRunExplicit` lifecycle carrier cell.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// let resource = Rc::new(7);
		/// let release_resource = Rc::clone(&resource);
		/// assert_eq!(Rc::strong_count(&resource), 2);
		/// let body_result = (|resource: Rc<i32>| *resource + 35)(resource);
		/// assert!((|resource: Rc<i32>| *resource == 7)(release_resource));
		/// assert_eq!(body_result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "The private RefBracket carrier signature must keep the wrapper, lifecycle, pointer, resource, and continuation types explicit."
		)]
		pub(crate) fn dispatch_rc_run_explicit_ref_bracket_carrier<
			'a,
			R,
			S,
			Resource,
			BodyResult,
			Final,
			K,
			Acquire,
			BodyFn,
			Release,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitRefBracketCarrierLayer<
				'a,
				RcBrand,
				Resource,
				BodyResult,
				Acquire,
				BodyFn,
				Release,
				RcRunExplicitLifecycleScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
		) -> RcRunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Resource: Clone + 'a,
			BodyResult: Clone + 'a,
			Final: 'a,
			K: Fn(BodyResult) -> RcRunExplicit<'a, R, S, Final> + 'a,
			Acquire: Fn() -> RcRunExplicit<'a, R, S, Resource> + 'a,
			BodyFn: Fn(Rc<Resource>) -> RcRunExplicit<'a, R, S, BodyResult> + Clone + 'a,
			Release: Fn(Rc<Resource>) -> RcRunExplicit<'a, R, S, ()> + Clone + 'a,
			FirstLayer: 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, ()>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone,
			RcRunExplicitLifecycleScopedContinuation<'a, R, S, BodyResult, Final, K>:
				ScopedResumeTypes<
						'a,
						ActionValue = BodyResult,
						ActionProgram = RcRunExplicit<'a, R, S, BodyResult>,
					> + RcLifecycleScopedResume<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>, {
			let (acquire, body, release, continuation) = layer.into_parts();

			continuation.resume_rc_with_lifecycle_action(fo_handlers, move || {
				acquire().bind(move |resource| {
					let resource = Rc::new(resource);
					let release_resource = Rc::clone(&resource);
					let body = body.clone();
					let release = release.clone();
					body(resource).bind(move |body_result| {
						release(Rc::clone(&release_resource)).map(move |()| body_result.clone())
					})
				})
			})
		}

		/// Dispatch a private `ArcRunExplicit` RefBracket carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Arc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The acquire lifecycle cell type.",
			"The body lifecycle cell type.",
			"The release lifecycle cell type.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The private RefBracket layer carrying acquire, body, release, and the `ArcRunExplicit` lifecycle carrier cell.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the carrier.")]
		#[document_examples]
		///
		/// ```
		/// use std::sync::Arc;
		///
		/// let resource = Arc::new(7);
		/// let release_resource = Arc::clone(&resource);
		/// assert_eq!(Arc::strong_count(&resource), 2);
		/// let body_result = (|resource: Arc<i32>| *resource + 35)(resource);
		/// assert!((|resource: Arc<i32>| *resource == 7)(release_resource));
		/// assert_eq!(body_result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "The private RefBracket carrier signature must keep the wrapper, lifecycle, pointer, resource, and continuation types explicit."
		)]
		pub(crate) fn dispatch_arc_run_explicit_ref_bracket_carrier<
			'a,
			R,
			S,
			Resource,
			BodyResult,
			Final,
			K,
			Acquire,
			BodyFn,
			Release,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitRefBracketCarrierLayer<
				'a,
				ArcBrand,
				Resource,
				BodyResult,
				Acquire,
				BodyFn,
				Release,
				ArcRunExplicitLifecycleScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
		) -> ArcRunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + SendFunctor + 'static,
			S: WrapDrop + SendFunctor + 'static,
			Resource: Clone + Send + Sync + 'a,
			BodyResult: Clone + Send + Sync + 'a,
			Final: Send + Sync + 'a,
			K: Fn(BodyResult) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
			Acquire: Fn() -> ArcRunExplicit<'a, R, S, Resource> + Send + Sync + 'a,
			BodyFn: Fn(Arc<Resource>) -> ArcRunExplicit<'a, R, S, BodyResult>
				+ Clone
				+ Send
				+ Sync
				+ 'a,
			Release: Fn(Arc<Resource>) -> ArcRunExplicit<'a, R, S, ()> + Clone + Send + Sync + 'a,
			FirstLayer: 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, ()>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone + Send + Sync,
			ArcRunExplicitLifecycleScopedContinuation<'a, R, S, BodyResult, Final, K>:
				ScopedResumeTypes<
						'a,
						ActionValue = BodyResult,
						ActionProgram = ArcRunExplicit<'a, R, S, BodyResult>,
					> + ArcLifecycleScopedResume<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>, {
			let (acquire, body, release, continuation) = layer.into_parts();

			continuation.resume_arc_with_lifecycle_action(fo_handlers, move || {
				acquire().bind(move |resource| {
					let resource = Arc::new(resource);
					let release_resource = Arc::clone(&resource);
					let body = body.clone();
					let release = release.clone();
					body(resource).bind(move |body_result| {
						release(Arc::clone(&release_resource)).map(move |()| body_result.clone())
					})
				})
			})
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
		"The row embedding witness used to rebuild the original row.",
		"The first first-order handler layer type."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, E, Idx, RMinusE, EmbedIndices, FirstLayer>
		DispatchRunRawScopedHandler<R, S, A, BoxLocalBrand<BoxBrand, E>, FirstLayer>
		for LocalDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::local_dispatcher,
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
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, FirstRowMinusReader, _>(),
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
					.interpose::<BoxReaderBrand<BoxBrand, E>, Idx, RMinusE, EmbedIndices>(
						move |op| match op {
							BoxReader::Ask(k) => k(local_env.clone()),
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
		DispatchRunRawScopedHandler<R, S, A, BoxRefLocalBrand<BoxBrand, E>, FirstLayer>
		for RefLocalDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
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
		/// 		scoped_dispatchers::ref_local_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env: i32| Run::pure(env * 2));
		/// let program: Prog = Run::ref_local::<i32, _>(|env| *env + 5, action);
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxRefLocalBrand<BoxBrand, i32>: ref_local_dispatcher::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 30);
		/// ```
		fn dispatch_run_raw_scoped_head(
			&self,
			layer: BoxRefLocal<'static, BoxBrand, E, RawRunFree<R, S>>,
			continuations: RunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
		) -> Run<R, S, A> {
			match layer {
				BoxRefLocal::Local {
					modify,
					action,
				} => Run::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(&env);
					let interposed = Run::<R, S, crate::types::free::TypeErasedValue>::from_free(
						action(()).erase_type(),
					)
					.interpose::<BoxReaderBrand<BoxBrand, E>, Idx, RMinusE, EmbedIndices>(
						move |op| match op {
							BoxReader::Ask(k) => k(local_env.clone()),
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
		DispatchRunRawScopedHandler<R, S, A, BoxCatchBrand<BoxBrand, E>, FirstLayer>
		for CatchDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::catch_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<i32>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::catch::<i32, _>(Run::throw::<i32, _>(7), |err| Run::pure(err + 35));
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		ExceptBrand<i32>: |_op: Except<'_, i32, Prog>| Run::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxCatchBrand<BoxBrand, i32>: catch_dispatcher::<_, FirstRowMinusExcept, _>(),
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
					let handler = std::cell::RefCell::new(Some(handler));
					let interposed = Run::<R, S, crate::types::free::TypeErasedValue>::from_free(
					action(()).erase_type(),
				)
				.interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| match op {
					Except::Throw(e, _) => {
						#[expect(
							clippy::expect_used,
							reason = "Box-backed Catch handlers are single-shot and the protected action can throw at most once"
						)]
						let handler = handler
							.borrow_mut()
							.take()
							.expect("BoxCatch handler invoked more than once");
						Run::from_free(handler(e).erase_type())
					}
				});
					Run::from_free(Free::continue_from_reboxed_erased(
						interposed.into_free(),
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
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'static,
			Catch<'static, RcBrand, E, RcRun<R, S, A>>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
			RcRun<R, S, A>,
		> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::catch_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<i32>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::catch::<i32, _>(Run::throw::<i32, _>(7), |err| Run::pure(err + 35));
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		ExceptBrand<i32>: |_op: Except<'_, i32, Prog>| Run::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxCatchBrand<BoxBrand, i32>: catch_dispatcher::<_, FirstRowMinusExcept, _>(),
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
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'static,
			SendCatch<'static, ArcBrand, E, ArcRun<R, S, A>>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			ArcRun<R, S, A>,
		> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::catch_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<i32>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::catch::<i32, _>(Run::throw::<i32, _>(7), |err| Run::pure(err + 35));
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		ExceptBrand<i32>: |_op: Except<'_, i32, Prog>| Run::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxCatchBrand<BoxBrand, i32>: catch_dispatcher::<_, FirstRowMinusExcept, _>(),
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
		> for LocalDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::local_dispatcher,
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
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, FirstRowMinusReader, _>(),
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
			RefLocal<'static, RcBrand, E, RcRun<R, S, A>>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
			RcRun<R, S, A>,
		> for RefLocalDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
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
		/// 		scoped_dispatchers::ref_local_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env: i32| Run::pure(env * 2));
		/// let program: Prog = Run::ref_local::<i32, _>(|env| *env + 5, action);
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxRefLocalBrand<BoxBrand, i32>: ref_local_dispatcher::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 30);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: RefLocal<'static, RcBrand, E, RcRun<R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
				RcRun<R, S, A>,
			>,
		) -> RcRun<R, S, A> {
			match layer {
				RefLocal::Local {
					modify,
					action,
				} => RcRun::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(&env);
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
		> for LocalDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::local_dispatcher,
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
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, FirstRowMinusReader, _>(),
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
			SendRefLocal<'static, ArcBrand, E, ArcRun<R, S, A>>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			ArcRun<R, S, A>,
		> for RefLocalDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
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
		/// 		scoped_dispatchers::ref_local_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env: i32| Run::pure(env * 2));
		/// let program: Prog = Run::ref_local::<i32, _>(|env| *env + 5, action);
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxRefLocalBrand<BoxBrand, i32>: ref_local_dispatcher::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 30);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendRefLocal<'static, ArcBrand, E, ArcRun<R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
				ArcRun<R, S, A>,
			>,
		) -> ArcRun<R, S, A> {
			match layer {
				SendRefLocal::Local {
					modify,
					action,
				} => ArcRun::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(&env);
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
		> for LocalDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::local_dispatcher,
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
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, FirstRowMinusReader, _>(),
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
			BoxRefLocal<'a, BoxBrand, E, RunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>),
			RunExplicit<'a, R, S, A>,
		> for RefLocalDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
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
		/// 		scoped_dispatchers::ref_local_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env: i32| Run::pure(env * 2));
		/// let program: Prog = Run::ref_local::<i32, _>(|env| *env + 5, action);
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxRefLocalBrand<BoxBrand, i32>: ref_local_dispatcher::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 30);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: BoxRefLocal<'a, BoxBrand, E, RunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>
				),
				RunExplicit<'a, R, S, A>,
			>,
		) -> RunExplicit<'a, R, S, A> {
			match layer {
				BoxRefLocal::Local {
					modify,
					action,
				} => {
					let modify = std::cell::RefCell::new(Some(modify));
					let action = std::cell::RefCell::new(Some(action));
					RunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
						#[expect(
							clippy::expect_used,
							reason = "Box-backed RefLocal is single-shot; RunExplicit invokes this continuation once"
						)]
						let modify = modify
							.borrow_mut()
							.take()
							.expect("BoxRefLocal modify invoked more than once");
						#[expect(
							clippy::expect_used,
							reason = "Box-backed RefLocal is single-shot; RunExplicit invokes this continuation once"
						)]
						let action = action
							.borrow_mut()
							.take()
							.expect("BoxRefLocal action invoked more than once");
						let local_env = modify(&env);
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
		> for LocalDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::local_dispatcher,
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
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, FirstRowMinusReader, _>(),
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
			RefLocal<'a, RcBrand, E, RcRunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
			RcRunExplicit<'a, R, S, A>,
		> for RefLocalDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
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
		/// 		scoped_dispatchers::ref_local_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env: i32| Run::pure(env * 2));
		/// let program: Prog = Run::ref_local::<i32, _>(|env| *env + 5, action);
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxRefLocalBrand<BoxBrand, i32>: ref_local_dispatcher::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 30);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: RefLocal<'a, RcBrand, E, RcRunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
				),
				RcRunExplicit<'a, R, S, A>,
			>,
		) -> RcRunExplicit<'a, R, S, A> {
			match layer {
				RefLocal::Local {
					modify,
					action,
				} => RcRunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(&env);
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
		> for LocalDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::local_dispatcher,
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
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, FirstRowMinusReader, _>(),
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
			SendRefLocal<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
			ArcRunExplicit<'a, R, S, A>,
		> for RefLocalDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
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
		/// 		scoped_dispatchers::ref_local_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env: i32| Run::pure(env * 2));
		/// let program: Prog = Run::ref_local::<i32, _>(|env| *env + 5, action);
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxRefLocalBrand<BoxBrand, i32>: ref_local_dispatcher::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 30);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendRefLocal<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, A>>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
				),
				ArcRunExplicit<'a, R, S, A>,
			>,
		) -> ArcRunExplicit<'a, R, S, A> {
			match layer {
				SendRefLocal::Local {
					modify,
					action,
				} => ArcRunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(&env);
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
			BoxCatch<'a, BoxBrand, E, RunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>),
			RunExplicit<'a, R, S, A>,
		> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::catch_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<i32>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::catch::<i32, _>(Run::throw::<i32, _>(7), |err| Run::pure(err + 35));
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		ExceptBrand<i32>: |_op: Except<'_, i32, Prog>| Run::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxCatchBrand<BoxBrand, i32>: catch_dispatcher::<_, FirstRowMinusExcept, _>(),
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
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'a,
			Catch<'a, RcBrand, E, RcRunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
			RcRunExplicit<'a, R, S, A>,
		> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::catch_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<i32>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::catch::<i32, _>(Run::throw::<i32, _>(7), |err| Run::pure(err + 35));
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		ExceptBrand<i32>: |_op: Except<'_, i32, Prog>| Run::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxCatchBrand<BoxBrand, i32>: catch_dispatcher::<_, FirstRowMinusExcept, _>(),
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
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, A, E, Idx, RMinusE, EmbedIndices>
		DispatchScopedHandler<
			'a,
			SendCatch<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, A>>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
			ArcRunExplicit<'a, R, S, A>,
		> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::catch_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<i32>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::catch::<i32, _>(Run::throw::<i32, _>(7), |err| Run::pure(err + 35));
		/// let result = program.interpret(
		/// 	handlers! {
		/// 		ExceptBrand<i32>: |_op: Except<'_, i32, Prog>| Run::pure(0),
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxCatchBrand<BoxBrand, i32>: catch_dispatcher::<_, FirstRowMinusExcept, _>(),
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
		> for SpanDispatcher
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::span_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_dispatcher(),
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
		DispatchRunRawScopedHandler<R, S, A, BoxSpanBrand<BoxBrand, Tag>, FirstLayer> for SpanDispatcher
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::span_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_dispatcher(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_run_raw_scoped_head(
			&self,
			layer: BoxSpan<'static, BoxBrand, Tag, RawRunFree<R, S>>,
			continuations: RunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
		) -> Run<R, S, A> {
			match layer {
				BoxSpan::Span {
					tag: _tag,
					action,
				} => Run::from_free(Free::continue_from_erased(action(()), continuations)),
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
		> for SpanDispatcher
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::span_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_dispatcher(),
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
		> for SpanDispatcher
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::span_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_dispatcher(),
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
		> for SpanDispatcher
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::span_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_dispatcher(),
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
		> for SpanDispatcher
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::span_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_dispatcher(),
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
		> for SpanDispatcher
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
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
		/// 		scoped_dispatchers::span_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = Run::span::<i32, _>(7, Run::pure(42));
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, i32>: span_dispatcher(),
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

	/// Dispatch implementation for the default `Run` Bracket dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release.",
		"The first first-order handler layer type."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, Resource, Body, FirstLayer>
		DispatchRunRawScopedHandler<
			R,
			S,
			Body,
			BoxBracketBrand<BoxBrand, NodeBrand<R, S>, Resource, Body>,
			FirstLayer,
		> for BracketDispatcher
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Resource: 'static,
		Body: 'static,
		FirstLayer: 'static,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The raw scoped Bracket layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			scoped_dispatchers::bracket_dispatcher,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow =
		/// 	CoproductBrand<BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl Functor for ScopedRow {
		/// 	fn map<'a, A: 'a, B: 'a>(
		/// 		f: impl Fn(A) -> B + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as Functor>::map(f, fa)
		/// 	}
		/// }
		///
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure((*resource, *resource + 35)),
		/// 	move |_resource: Rc<i32>| {
		/// 		released_in_cleanup.set(true);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: bracket_dispatcher(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_run_raw_scoped_head(
			&self,
			layer: BoxBracket<'static, BoxBrand, NodeBrand<R, S>, Resource, Body>,
			continuations: RunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, Body>>,
		) -> Run<R, S, Body> {
			match layer {
				BoxBracket::Bracket {
					acquire,
					body,
					release,
				} => {
					let bracket =
						Run::<R, S, Resource>::from_free(acquire(())).bind(move |resource| {
							Run::<R, S, (Resource, Body)>::from_free(body(Box::new(resource))).bind(
								move |(resource, body_result)| {
									Run::<R, S, ()>::from_free(release(Box::new(resource)))
										.map(move |()| body_result)
								},
							)
						});
					Run::from_free(Free::continue_from_erased(
						bracket.into_free().cast_erased(),
						continuations,
					))
				}
			}
		}
	}

	/// Dispatch implementation for the explicit `Run` Bracket dispatcher.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, Resource, Body>
		DispatchScopedHandler<
			'a,
			BoxBracketExplicit<'a, BoxBrand, NodeBrand<R, S>, Resource, Body>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RunExplicit<'a, R, S, Body>,
			>),
			RunExplicit<'a, R, S, Body>,
		> for BracketDispatcher
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Resource: 'a,
		Body: 'a,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped Bracket layer to interpret.",
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			scoped_dispatchers::bracket_dispatcher,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow =
		/// 	CoproductBrand<BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl Functor for ScopedRow {
		/// 	fn map<'a, A: 'a, B: 'a>(
		/// 		f: impl Fn(A) -> B + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as Functor>::map(f, fa)
		/// 	}
		/// }
		///
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure((*resource, *resource + 35)),
		/// 	move |_resource: Rc<i32>| {
		/// 		released_in_cleanup.set(true);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: bracket_dispatcher(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: BoxBracketExplicit<'a, BoxBrand, NodeBrand<R, S>, Resource, Body>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						RunExplicit<'a, R, S, Body>,
					>
				),
				RunExplicit<'a, R, S, Body>,
			>,
		) -> RunExplicit<'a, R, S, Body> {
			match layer {
				BoxBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => {
					let body = std::cell::RefCell::new(Some(body));
					let release = Rc::new(std::cell::RefCell::new(Some(release)));
					RunExplicit::<R, S, Resource>::from_free_explicit(*acquire(())).bind(
						move |resource| {
							#[expect(
								clippy::expect_used,
								reason = "Box-backed Bracket is single-shot; RunExplicit invokes this continuation once"
							)]
							let body = body
								.borrow_mut()
								.take()
								.expect("BoxBracketExplicit body invoked more than once");
							let release = Rc::clone(&release);
							RunExplicit::<R, S, (Resource, Body)>::from_free_explicit(*body(
								Box::new(resource),
							))
							.bind(move |(resource, body_result)| {
								#[expect(
									clippy::expect_used,
									reason = "Box-backed Bracket is single-shot; RunExplicit invokes this continuation once"
								)]
								let release = release
									.borrow_mut()
									.take()
									.expect("BoxBracketExplicit release invoked more than once");
								let body_result = std::cell::RefCell::new(Some(body_result));
								RunExplicit::<R, S, ()>::from_free_explicit(*release(Box::new(
									resource,
								)))
								.bind(move |()| {
									#[expect(
										clippy::expect_used,
										reason = "Box-backed Bracket is single-shot; RunExplicit invokes this continuation once"
									)]
									RunExplicit::pure(body_result.borrow_mut().take().expect(
										"BoxBracketExplicit result returned more than once",
									))
								})
							})
						},
					)
				}
			}
		}
	}

	/// Dispatch implementation for the Rc-backed Bracket dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, Resource, Body>
		DispatchScopedHandler<
			'static,
			Bracket<'static, RcBrand, NodeBrand<R, S>, Resource, Body>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, Body>>),
			RcRun<R, S, Body>,
		> for BracketDispatcher
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Resource: Clone + 'static,
		Body: Clone + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped Bracket layer to interpret.",
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			scoped_dispatchers::bracket_dispatcher,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow =
		/// 	CoproductBrand<BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl Functor for ScopedRow {
		/// 	fn map<'a, A: 'a, B: 'a>(
		/// 		f: impl Fn(A) -> B + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as Functor>::map(f, fa)
		/// 	}
		/// }
		///
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure((*resource, *resource + 35)),
		/// 	move |_resource: Rc<i32>| {
		/// 		released_in_cleanup.set(true);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: bracket_dispatcher(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: Bracket<'static, RcBrand, NodeBrand<R, S>, Resource, Body>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, Body>>),
				RcRun<R, S, Body>,
			>,
		) -> RcRun<R, S, Body> {
			match layer {
				Bracket::Bracket {
					acquire,
					body,
					release,
				} => RcRun::<R, S, Resource>::from_rc_free(acquire(())).bind(move |resource| {
					let body = Rc::clone(&body);
					let release = Rc::clone(&release);
					RcRun::<R, S, (Resource, Body)>::from_rc_free(body(Rc::new(resource))).bind(
						move |(resource, body_result)| {
							RcRun::<R, S, ()>::from_rc_free(release(Rc::new(resource)))
								.map(move |()| body_result.clone())
						},
					)
				}),
			}
		}
	}

	/// Dispatch implementation for the Arc-backed Bracket dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, Resource, Body>
		DispatchScopedHandler<
			'static,
			SendBracket<'static, ArcBrand, NodeBrand<R, S>, Resource, Body>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, Body>>),
			ArcRun<R, S, Body>,
		> for BracketDispatcher
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		Resource: Clone + Send + Sync + 'static,
		Body: Clone + Send + Sync + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone + Send + Sync,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped Bracket layer to interpret.",
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			scoped_dispatchers::bracket_dispatcher,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow =
		/// 	CoproductBrand<BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl Functor for ScopedRow {
		/// 	fn map<'a, A: 'a, B: 'a>(
		/// 		f: impl Fn(A) -> B + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as Functor>::map(f, fa)
		/// 	}
		/// }
		///
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure((*resource, *resource + 35)),
		/// 	move |_resource: Rc<i32>| {
		/// 		released_in_cleanup.set(true);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: bracket_dispatcher(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendBracket<'static, ArcBrand, NodeBrand<R, S>, Resource, Body>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, Body>>),
				ArcRun<R, S, Body>,
			>,
		) -> ArcRun<R, S, Body> {
			match layer {
				SendBracket::Bracket {
					acquire,
					body,
					release,
				} => ArcRun::<R, S, Resource>::from_arc_free(acquire(())).bind(move |resource| {
					let body = Arc::clone(&body);
					let release = Arc::clone(&release);
					ArcRun::<R, S, (Resource, Body)>::from_arc_free(body(Arc::new(resource))).bind(
						move |(resource, body_result)| {
							ArcRun::<R, S, ()>::from_arc_free(release(Arc::new(resource)))
								.map(move |()| body_result.clone())
						},
					)
				}),
			}
		}
	}

	/// Dispatch implementation for the Rc explicit Bracket dispatcher.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, Resource, Body>
		DispatchScopedHandler<
			'a,
			BracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, Body>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, Body>,
			>),
			RcRunExplicit<'a, R, S, Body>,
		> for BracketDispatcher
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Resource: Clone + 'a,
		Body: Clone + 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (Resource, Body)>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, ()>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Body>,
		>): Clone,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped Bracket layer to interpret.",
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			scoped_dispatchers::bracket_dispatcher,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow =
		/// 	CoproductBrand<BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl Functor for ScopedRow {
		/// 	fn map<'a, A: 'a, B: 'a>(
		/// 		f: impl Fn(A) -> B + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as Functor>::map(f, fa)
		/// 	}
		/// }
		///
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure((*resource, *resource + 35)),
		/// 	move |_resource: Rc<i32>| {
		/// 		released_in_cleanup.set(true);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: bracket_dispatcher(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: BracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, Body>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						RcRunExplicit<'a, R, S, Body>,
					>
				),
				RcRunExplicit<'a, R, S, Body>,
			>,
		) -> RcRunExplicit<'a, R, S, Body> {
			match layer {
				BracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => RcRunExplicit::<R, S, Resource>::from_rc_free_explicit(acquire(())).bind(
					move |resource| {
						let body = Rc::clone(&body);
						let release = Rc::clone(&release);
						RcRunExplicit::<R, S, (Resource, Body)>::from_rc_free_explicit(body(
							Rc::new(resource),
						))
						.bind(move |(resource, body_result)| {
							RcRunExplicit::<R, S, ()>::from_rc_free_explicit(release(Rc::new(
								resource,
							)))
							.map(move |()| body_result.clone())
						})
					},
				),
			}
		}
	}

	/// Dispatch implementation for the Arc explicit Bracket dispatcher.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, Resource, Body>
		DispatchScopedHandler<
			'a,
			SendBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, Body>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Body>,
			>),
			ArcRunExplicit<'a, R, S, Body>,
		> for BracketDispatcher
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Resource: Clone + Send + Sync + 'a,
		Body: Clone + Send + Sync + 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (Resource, Body)>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, ()>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Body>,
		>): Clone + Send + Sync,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped Bracket layer to interpret.",
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			scoped_dispatchers::bracket_dispatcher,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow =
		/// 	CoproductBrand<BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl Functor for ScopedRow {
		/// 	fn map<'a, A: 'a, B: 'a>(
		/// 		f: impl Fn(A) -> B + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as Functor>::map(f, fa)
		/// 	}
		/// }
		///
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure((*resource, *resource + 35)),
		/// 	move |_resource: Rc<i32>| {
		/// 		released_in_cleanup.set(true);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: bracket_dispatcher(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, Body>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						ArcRunExplicit<'a, R, S, Body>,
					>
				),
				ArcRunExplicit<'a, R, S, Body>,
			>,
		) -> ArcRunExplicit<'a, R, S, Body> {
			match layer {
				SendBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => ArcRunExplicit::<R, S, Resource>::from_arc_free_explicit(acquire(())).bind(
					move |resource| {
						let body = Arc::clone(&body);
						let release = Arc::clone(&release);
						ArcRunExplicit::<R, S, (Resource, Body)>::from_arc_free_explicit(body(
							Arc::new(resource),
						))
						.bind(move |(resource, body_result)| {
							ArcRunExplicit::<R, S, ()>::from_arc_free_explicit(release(Arc::new(
								resource,
							)))
							.map(move |()| body_result.clone())
						})
					},
				),
			}
		}
	}

	/// Dispatch implementation for the Rc-backed RefBracket dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, Resource, Body>
		DispatchScopedHandler<
			'static,
			RefBracket<'static, RcBrand, NodeBrand<R, S>, Resource, Body>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, Body>>),
			RcRun<R, S, Body>,
		> for RefBracketDispatcher
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Resource: Clone + 'static,
		Body: Clone + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped RefBracket layer to interpret.",
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			scoped_dispatchers::ref_bracket_dispatcher,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow = CoproductBrand<
		/// 	RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
		/// 	CNilBrand,
		/// >;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl Functor for ScopedRow {
		/// 	fn map<'a, A: 'a, B: 'a>(
		/// 		f: impl Fn(A) -> B + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as Functor>::map(f, fa)
		/// 	}
		/// }
		///
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure(*resource + 35),
		/// 	move |resource: Rc<i32>| {
		/// 		released_in_cleanup.set(*resource == 7);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: ref_bracket_dispatcher(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: RefBracket<'static, RcBrand, NodeBrand<R, S>, Resource, Body>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, Body>>),
				RcRun<R, S, Body>,
			>,
		) -> RcRun<R, S, Body> {
			match layer {
				RefBracket::Bracket {
					acquire,
					body,
					release,
				} => RcRun::<R, S, Resource>::from_rc_free(acquire(())).bind(move |resource| {
					let resource = Rc::new(resource);
					let release_resource = Rc::clone(&resource);
					let body = Rc::clone(&body);
					let release = Rc::clone(&release);
					RcRun::<R, S, Body>::from_rc_free(body(resource)).bind(move |body_result| {
						RcRun::<R, S, ()>::from_rc_free(release(Rc::clone(&release_resource)))
							.map(move |()| body_result.clone())
					})
				}),
			}
		}
	}

	/// Dispatch implementation for the Arc-backed RefBracket dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, Resource, Body>
		DispatchScopedHandler<
			'static,
			SendRefBracket<'static, ArcBrand, NodeBrand<R, S>, Resource, Body>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, Body>>),
			ArcRun<R, S, Body>,
		> for RefBracketDispatcher
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		Resource: Clone + Send + Sync + 'static,
		Body: Clone + Send + Sync + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone + Send + Sync,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped RefBracket layer to interpret.",
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			scoped_dispatchers::ref_bracket_dispatcher,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow = CoproductBrand<
		/// 	RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
		/// 	CNilBrand,
		/// >;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl Functor for ScopedRow {
		/// 	fn map<'a, A: 'a, B: 'a>(
		/// 		f: impl Fn(A) -> B + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as Functor>::map(f, fa)
		/// 	}
		/// }
		///
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure(*resource + 35),
		/// 	move |resource: Rc<i32>| {
		/// 		released_in_cleanup.set(*resource == 7);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: ref_bracket_dispatcher(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendRefBracket<'static, ArcBrand, NodeBrand<R, S>, Resource, Body>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, Body>>),
				ArcRun<R, S, Body>,
			>,
		) -> ArcRun<R, S, Body> {
			match layer {
				SendRefBracket::Bracket {
					acquire,
					body,
					release,
				} => ArcRun::<R, S, Resource>::from_arc_free(acquire(())).bind(move |resource| {
					let resource = Arc::new(resource);
					let release_resource = Arc::clone(&resource);
					let body = Arc::clone(&body);
					let release = Arc::clone(&release);
					ArcRun::<R, S, Body>::from_arc_free(body(resource)).bind(move |body_result| {
						ArcRun::<R, S, ()>::from_arc_free(release(Arc::clone(&release_resource)))
							.map(move |()| body_result.clone())
					})
				}),
			}
		}
	}

	/// Dispatch implementation for the Rc explicit RefBracket dispatcher.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, Resource, Body>
		DispatchScopedHandler<
			'a,
			RefBracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, Body>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, Body>,
			>),
			RcRunExplicit<'a, R, S, Body>,
		> for RefBracketDispatcher
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Resource: Clone + 'a,
		Body: Clone + 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Body>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, ()>,
		>): Clone,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped RefBracket layer to interpret.",
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			scoped_dispatchers::ref_bracket_dispatcher,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow = CoproductBrand<
		/// 	RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
		/// 	CNilBrand,
		/// >;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl Functor for ScopedRow {
		/// 	fn map<'a, A: 'a, B: 'a>(
		/// 		f: impl Fn(A) -> B + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as Functor>::map(f, fa)
		/// 	}
		/// }
		///
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure(*resource + 35),
		/// 	move |resource: Rc<i32>| {
		/// 		released_in_cleanup.set(*resource == 7);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: ref_bracket_dispatcher(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: RefBracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, Body>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						RcRunExplicit<'a, R, S, Body>,
					>
				),
				RcRunExplicit<'a, R, S, Body>,
			>,
		) -> RcRunExplicit<'a, R, S, Body> {
			match layer {
				RefBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => RcRunExplicit::<R, S, Resource>::from_rc_free_explicit(acquire(())).bind(
					move |resource| {
						let resource = Rc::new(resource);
						let release_resource = Rc::clone(&resource);
						let body = Rc::clone(&body);
						let release = Rc::clone(&release);
						RcRunExplicit::<R, S, Body>::from_rc_free_explicit(body(resource)).bind(
							move |body_result| {
								RcRunExplicit::<R, S, ()>::from_rc_free_explicit(release(
									Rc::clone(&release_resource),
								))
								.map(move |()| body_result.clone())
							},
						)
					},
				),
			}
		}
	}

	/// Dispatch implementation for the Arc explicit RefBracket dispatcher.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, Resource, Body>
		DispatchScopedHandler<
			'a,
			SendRefBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, Body>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Body>,
			>),
			ArcRunExplicit<'a, R, S, Body>,
		> for RefBracketDispatcher
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Resource: Clone + Send + Sync + 'a,
		Body: Clone + Send + Sync + 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Body>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, ()>,
		>): Clone + Send + Sync,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped RefBracket layer to interpret.",
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			scoped_dispatchers::ref_bracket_dispatcher,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow = CoproductBrand<
		/// 	RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
		/// 	CNilBrand,
		/// >;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl Functor for ScopedRow {
		/// 	fn map<'a, A: 'a, B: 'a>(
		/// 		f: impl Fn(A) -> B + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as Functor>::map(f, fa)
		/// 	}
		/// }
		///
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure(*resource + 35),
		/// 	move |resource: Rc<i32>| {
		/// 		released_in_cleanup.set(*resource == 7);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: ref_bracket_dispatcher(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendRefBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, Body>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						ArcRunExplicit<'a, R, S, Body>,
					>
				),
				ArcRunExplicit<'a, R, S, Body>,
			>,
		) -> ArcRunExplicit<'a, R, S, Body> {
			match layer {
				SendRefBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => ArcRunExplicit::<R, S, Resource>::from_arc_free_explicit(acquire(())).bind(
					move |resource| {
						let resource = Arc::new(resource);
						let release_resource = Arc::clone(&resource);
						let body = Arc::clone(&body);
						let release = Arc::clone(&release);
						ArcRunExplicit::<R, S, Body>::from_arc_free_explicit(body(resource)).bind(
							move |body_result| {
								ArcRunExplicit::<R, S, ()>::from_arc_free_explicit(release(
									Arc::clone(&release_resource),
								))
								.map(move |()| body_result.clone())
							},
						)
					},
				),
			}
		}
	}
}

pub use inner::*;
