#[allow(
	unused_imports,
	reason = "Each scoped-dispatcher child module consumes a different subset of the shared parent prelude."
)]
use super::prelude::*;

#[fp_macros::document_module]
mod inner {
	use super::*;

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

	/// Carrier-aware Catch dispatch for `RunExplicitBoundary`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected Catch action result type.",
		"The final program result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The error type handled by Catch.",
		"The row index witnessing the target Except operation.",
		"The first-order row brand with the handled operation removed.",
		"The row embedding witness used to rebuild the original row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Catch dispatcher receiver.")]
	impl<'a, R, S, Action, Final, K, E, Idx, RMinusE, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			BoxCatch<'a, BoxBrand, E, RunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RunExplicit<'a, R, S, Final>,
			RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: 'a,
		Final: 'a,
		K: Fn(Action) -> RunExplicit<'a, R, S, Final> + 'a,
		E: 'a + 'static,
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
		RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>: ScopedResumeTypes<
				'a,
				ActionValue = Action,
				ActionProgram = RunExplicit<'a, R, S, Action>,
			>,
	{
		/// Run the protected action with recovery, then resume the outer
		/// continuation.
		#[document_signature]
		#[document_parameters(
			"The Catch scoped layer carrying the selected action program.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the Catch boundary.")]
		#[document_examples]
		///
		/// ```
		/// let recover = |err: &'static str| {
		/// 	assert_eq!(err, "from-action");
		/// 	41
		/// };
		/// let outer = |value| value + 1;
		/// assert_eq!(outer(recover("from-action")), 42);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: BoxCatch<'a, BoxBrand, E, RunExplicit<'a, R, S, Action>>,
			continuation: crate::types::effects::interpreter::ScopedContinuation<
				RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
		) -> RunExplicit<'a, R, S, Final> {
			let outer = continuation.into_inner().outer.clone();
			match layer {
				BoxCatch::Catch {
					action,
					handler,
				} => {
					let handler = Rc::new(std::cell::RefCell::new(Some(handler)));
					action(())
						.interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
							match op {
								Except::Throw(e, _) => {
									#[expect(
										clippy::expect_used,
										reason = "Box-backed Catch boundary handlers are single-shot and the protected action can throw at most once"
									)]
									let handler = handler.borrow_mut().take().expect(
										"RunExplicit Catch boundary handler invoked more than once",
									);
									handler(e)
								}
							}
						})
						.bind(move |action_value| (*outer)(action_value))
				}
			}
		}
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
		/// Dispatch an indexed `RunExplicit` Catch boundary.
		///
		/// The boundary layer owns the selected action program while the
		/// boundary continuation owns only the typed outer resume. The
		/// dispatcher projects the Catch layer, runs recovery inside the
		/// selected action, and resumes the outer continuation only after
		/// the action or recovery has produced a value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Catch action result type.",
			"The final result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The recovered error type.",
			"The type-level Member-position witness for the scoped Catch layer.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The indexed Catch boundary produced around the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the boundary.")]
		#[document_examples]
		///
		/// ```
		/// let recover = |err: &'static str| {
		/// 	assert_eq!(err, "from-action");
		/// 	41
		/// };
		/// let outer = |value| value + 1;
		/// assert_eq!(outer(recover("from-action")), 42);
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "RunExplicit Catch boundaries are constructed by injecting a Catch layer; reaching the non-Catch projection branch means a crate-private constructor violated the boundary invariant."
		)]
		pub fn dispatch_run_explicit_catch_boundary<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			E,
			ScopedIdx,
			FirstLayer,
		>(
			&self,
			boundary: RunExplicitBoundary<'a, R, S, Action, Final, K>,
			fo_handlers: &'a (impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>> + 'a),
		) -> RunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Action: 'a,
			Final: 'a,
			K: Fn(Action) -> RunExplicit<'a, R, S, Final> + 'a,
			E: 'a + 'static,
			FirstLayer: 'a,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RunExplicit<'a, R, S, Action>,
			>): Member<BoxCatch<'a, BoxBrand, E, RunExplicit<'a, R, S, Action>>, ScopedIdx>,
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
				>, {
			let (layer, continuation) = boundary.into_parts();
			let catch = match <Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RunExplicit<'a, R, S, Action>,
				>) as Member<
				BoxCatch<'a, BoxBrand, E, RunExplicit<'a, R, S, Action>>,
				ScopedIdx,
			>>::project(layer)
			{
				Ok(catch) => catch,
				Err(_) =>
					unreachable!("RunExplicit Catch boundary contained a non-Catch scoped layer"),
			};

			match catch {
				BoxCatch::Catch {
					action,
					handler,
				} => {
					let handler = Rc::new(std::cell::RefCell::new(Some(handler)));
					continuation.resume_explicit_with_supplied_action(fo_handlers, move || {
						let handler = Rc::clone(&handler);
						action(()).interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(
							move |op| match op {
								Except::Throw(e, _) => {
									#[expect(
										clippy::expect_used,
										reason = "Box-backed Catch boundary handlers are single-shot and the protected action can throw at most once"
									)]
									let handler = handler.borrow_mut().take().expect(
										"RunExplicit Catch boundary handler invoked more than once",
									);
									handler(e)
								}
							},
						)
					})
				}
			}
		}

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

		/// Dispatch an indexed `RcRunExplicit` Catch boundary.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of values carried by the Rc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Catch action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The recovered error type.",
			"The type-level Member-position witness for the scoped Catch layer.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The indexed Catch boundary produced around the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the boundary.")]
		#[document_examples]
		///
		/// ```
		/// let recover = |err: &'static str| {
		/// 	assert_eq!(err, "from-action");
		/// 	41
		/// };
		/// let outer = |value| value + 1;
		/// assert_eq!(outer(recover("from-action")), 42);
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "RcRunExplicit Catch boundaries are constructed by injecting a Catch layer; reaching the non-Catch projection branch means a crate-private constructor violated the boundary invariant."
		)]
		pub fn dispatch_rc_run_explicit_catch_boundary<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			E,
			ScopedIdx,
			FirstLayer,
		>(
			&self,
			boundary: RcRunExplicitBoundary<'a, R, S, Action, Final, K>,
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
			FirstLayer: 'a,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, Action>,
			>): Member<Catch<'a, RcBrand, E, RcRunExplicit<'a, R, S, Action>>, ScopedIdx>,
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
				>, {
			let (layer, continuation) = boundary.into_parts();
			let catch = match <Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcRunExplicit<'a, R, S, Action>,
				>) as Member<
				Catch<'a, RcBrand, E, RcRunExplicit<'a, R, S, Action>>,
				ScopedIdx,
			>>::project(layer)
			{
				Ok(catch) => catch,
				Err(_) =>
					unreachable!("RcRunExplicit Catch boundary contained a non-Catch scoped layer"),
			};

			match catch {
				Catch::Catch {
					action,
					handler,
				} => continuation.resume_rc_with_supplied_action(fo_handlers, move || {
					action(()).interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
						match op {
							Except::Throw(e, _) => handler(e),
						}
					})
				}),
			}
		}

		/// Dispatch an indexed `ArcRunExplicit` Catch boundary.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of values carried by the Arc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Catch action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The recovered error type.",
			"The type-level Member-position witness for the scoped Catch layer.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The indexed Catch boundary produced around the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the boundary.")]
		#[document_examples]
		///
		/// ```
		/// let recover = |err: &'static str| {
		/// 	assert_eq!(err, "from-action");
		/// 	41
		/// };
		/// let outer = |value| value + 1;
		/// assert_eq!(outer(recover("from-action")), 42);
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "ArcRunExplicit Catch boundaries are constructed by injecting a Catch layer; reaching the non-Catch projection branch means a crate-private constructor violated the boundary invariant."
		)]
		pub fn dispatch_arc_run_explicit_catch_boundary<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			E,
			ScopedIdx,
			FirstLayer,
		>(
			&self,
			boundary: ArcRunExplicitBoundary<'a, R, S, Action, Final, K>,
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
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Action>,
			>): Send
				+ Sync
				+ Member<SendCatch<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, Action>>, ScopedIdx>,
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
				>, {
			let (layer, continuation) = boundary.into_parts();
			let catch = match <Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcRunExplicit<'a, R, S, Action>,
				>) as Member<
				SendCatch<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, Action>>,
				ScopedIdx,
			>>::project(layer)
			{
				Ok(catch) => catch,
				Err(_) =>
					unreachable!("ArcRunExplicit Catch boundary contained a non-Catch scoped layer"),
			};

			match catch {
				SendCatch::Catch {
					action,
					handler,
				} => continuation.resume_arc_with_supplied_action(fo_handlers, move || {
					action(()).interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
						match op {
							Except::Throw(e, _) => handler(e),
						}
					})
				}),
			}
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

	#[document_type_parameters(
		"The lifetime of values carried by the shared Explicit boundary.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The handled error type.",
		"The first-order row index witnessing the handled Except operation.",
		"The first-order row brand with the handled Except operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Catch dispatcher receiver.")]
	impl<'a, R, S, Action, Final, K, E, Idx, RMinusE, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			Catch<'a, RcBrand, E, RcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RcRunExplicit<'a, R, S, Final>,
			RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Final: 'a,
		K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a,
		E: 'a + 'static,
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
	{
		/// Run Rc Catch recovery around the selected action and then apply the outer continuation.
		#[document_signature]
		#[document_parameters(
			"The Catch layer carrying the protected action and recovery handler.",
			"The wrapper-owned continuation carrier.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns(
			"The final `RcRunExplicit` program produced by the boundary dispatcher."
		)]
		#[document_examples]
		///
		/// ```
		/// let recover = |err: &'static str| {
		/// 	assert_eq!(err, "from-action");
		/// 	41
		/// };
		/// let outer = |value| value + 1;
		/// assert_eq!(outer(recover("from-action")), 42);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: Catch<'a, RcBrand, E, RcRunExplicit<'a, R, S, Action>>,
			continuation: crate::types::effects::interpreter::ScopedContinuation<
				RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
		) -> RcRunExplicit<'a, R, S, Final> {
			let outer = continuation.into_inner().outer.clone();
			match layer {
				Catch::Catch {
					action,
					handler,
				} => action(())
					.interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| match op {
						Except::Throw(e, _) => handler(e),
					})
					.bind(move |action_value| outer(action_value)),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of values carried by the shared Explicit boundary.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The handled error type.",
		"The first-order row index witnessing the handled Except operation.",
		"The first-order row brand with the handled Except operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Catch dispatcher receiver.")]
	impl<'a, R, S, Action, Final, K, E, Idx, RMinusE, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			SendCatch<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			ArcRunExplicit<'a, R, S, Final>,
			ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Action: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Action) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
		E: Send + Sync + 'a + 'static,
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
	{
		/// Run Arc Catch recovery around the selected action and then apply the outer continuation.
		#[document_signature]
		#[document_parameters(
			"The Catch layer carrying the protected action and recovery handler.",
			"The wrapper-owned continuation carrier.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns(
			"The final `ArcRunExplicit` program produced by the boundary dispatcher."
		)]
		#[document_examples]
		///
		/// ```
		/// let recover = |err: &'static str| {
		/// 	assert_eq!(err, "from-action");
		/// 	41
		/// };
		/// let outer = |value| value + 1;
		/// assert_eq!(outer(recover("from-action")), 42);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: SendCatch<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, Action>>,
			continuation: crate::types::effects::interpreter::ScopedContinuation<
				ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
		) -> ArcRunExplicit<'a, R, S, Final> {
			let outer = continuation.into_inner().outer.clone();
			match layer {
				SendCatch::Catch {
					action,
					handler,
				} => action(())
					.interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| match op {
						Except::Throw(e, _) => handler(e),
					})
					.bind(move |action_value| outer(action_value)),
			}
		}
	}
}

pub use inner::*;
