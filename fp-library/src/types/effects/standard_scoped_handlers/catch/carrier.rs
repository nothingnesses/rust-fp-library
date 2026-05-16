#[fp_macros::document_module]
pub(crate) mod inner {
	use super::super::{
		super::prelude::*,
		inner::CatchHandler,
	};

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
		> for CatchHandler<Idx, RMinusE, EmbedIndices>
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
				OperationValue = Action,
				OperationProgram = RunExplicit<'a, R, S, Action>,
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
		#[document_examples(skip_call_check)]
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
	impl<Idx, RMinusE, EmbedIndices> CatchHandler<Idx, RMinusE, EmbedIndices> {
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
		#[document_examples(skip_call_check)]
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
			boundary: RunExplicitBoundary<
				'a,
				R,
				S,
				BoxCatchBrand<BoxBrand, E>,
				ScopedIdx,
				Action,
				Final,
				K,
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
		#[document_examples(skip_call_check)]
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
					OperationValue = Action,
					OperationProgram = RunExplicit<'a, R, S, Action>,
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
		#[document_examples(skip_call_check)]
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
			boundary: RcRunExplicitBoundary<
				'a,
				R,
				S,
				CatchBrand<RcBrand, E>,
				ScopedIdx,
				Action,
				Final,
				K,
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
		#[document_examples(skip_call_check)]
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
			boundary: ArcRunExplicitBoundary<
				'a,
				R,
				S,
				SendCatchBrand<ArcBrand, E>,
				ScopedIdx,
				Action,
				Final,
				K,
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
		#[document_examples(skip_call_check)]
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
					OperationValue = Action,
					OperationProgram = RcRunExplicit<'a, R, S, Action>,
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
		#[document_examples(skip_call_check)]
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
					OperationValue = Action,
					OperationProgram = ArcRunExplicit<'a, R, S, Action>,
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
		> for CatchHandler<Idx, RMinusE, EmbedIndices>
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
		#[document_examples(skip_call_check)]
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
		> for CatchHandler<Idx, RMinusE, EmbedIndices>
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
		#[document_examples(skip_call_check)]
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
