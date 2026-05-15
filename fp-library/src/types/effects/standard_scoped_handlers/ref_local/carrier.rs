#[fp_macros::document_module]
pub(crate) mod inner {
	use super::super::{
		super::prelude::*,
		inner::RefLocalHandler,
	};

	/// Carrier-aware RefLocal dispatch for `RunExplicitBoundary`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected RefLocal action result type.",
		"The final program result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The Reader environment type.",
		"The row index witnessing the target Reader operation.",
		"The first-order row brand with the handled operation removed.",
		"The row embedding witness used to rebuild the original row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The RefLocal dispatcher receiver.")]
	impl<'a, R, S, Action, Final, K, E, Idx, RMinusE, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			BoxRefLocal<'a, BoxBrand, E, RunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RunExplicit<'a, R, S, Final>,
			RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for RefLocalHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: 'a,
		Final: 'a,
		K: Fn(Action) -> RunExplicit<'a, R, S, Final> + 'a,
		E: Clone + 'a + 'static,
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
		RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>: ScopedResumeTypes<
				'a,
				ActionValue = Action,
				ActionProgram = RunExplicit<'a, R, S, Action>,
				OperationValue = Action,
				OperationProgram = RunExplicit<'a, R, S, Action>,
			>,
	{
		/// Transform a borrowed Reader environment for the selected action,
		/// then resume the outer continuation.
		#[document_signature]
		#[document_parameters(
			"The RefLocal scoped layer carrying the selected action program.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the RefLocal boundary.")]
		#[document_examples]
		///
		/// ```
		/// let inherited_env = 10;
		/// let local_env = (|env: &i32| *env + 1)(&inherited_env);
		/// let action_result = local_env * 2;
		/// assert_eq!(action_result, 22);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: BoxRefLocal<'a, BoxBrand, E, RunExplicit<'a, R, S, Action>>,
			continuation: crate::types::effects::interpreter::ScopedContinuation<
				RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
		) -> RunExplicit<'a, R, S, Final> {
			match layer {
				BoxRefLocal::Local {
					modify,
					action,
				} => {
					let modify = std::cell::RefCell::new(Some(modify));
					let action = std::cell::RefCell::new(Some(action));
					let continuation = std::cell::RefCell::new(Some(continuation));

					RunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
						#[expect(
							clippy::expect_used,
							reason = "Box-backed RefLocal boundary dispatch is single-shot; RunExplicit invokes this continuation once"
						)]
						let modify = modify
							.borrow_mut()
							.take()
							.expect("RunExplicit RefLocal boundary modify invoked more than once");
						#[expect(
							clippy::expect_used,
							reason = "Box-backed RefLocal boundary dispatch is single-shot; RunExplicit invokes this continuation once"
						)]
						let action = action
							.borrow_mut()
							.take()
							.expect("RunExplicit RefLocal boundary action invoked more than once");
						#[expect(
							clippy::expect_used,
							reason = "Box-backed RefLocal boundary dispatch is single-shot; RunExplicit invokes this continuation once"
						)]
						let outer = continuation
							.borrow_mut()
							.take()
							.expect(
								"RunExplicit RefLocal boundary continuation invoked more than once",
							)
							.into_inner()
							.outer
							.clone();
						let local_env = modify(&env);

						action(())
							.interpose::<BoxReaderBrand<BoxBrand, E>, Idx, RMinusE, EmbedIndices>(
								move |op| match op {
									BoxReader::Ask(k) => k(local_env.clone()),
								},
							)
							.bind(move |action_value| (*outer)(action_value))
					})
				}
			}
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
	impl<Idx, RMinusE, EmbedIndices> RefLocalHandler<Idx, RMinusE, EmbedIndices> {
		/// Dispatch an indexed `RunExplicit` RefLocal boundary.
		///
		/// The boundary layer owns the selected action. The dispatcher
		/// asks the inherited Reader environment, applies the stored
		/// borrow-based modifier, supplies an action transformed so Reader
		/// asks see the local environment, then resumes the outer
		/// continuation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected RefLocal action result type.",
			"The final result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Reader environment type.",
			"The type-level Member-position witness for the scoped RefLocal layer.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The indexed RefLocal boundary produced around the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the boundary.")]
		#[document_examples]
		///
		/// ```
		/// let inherited_env = 10;
		/// let local_env = (|env: &i32| *env + 5)(&inherited_env);
		/// assert_eq!(local_env * 2, 30);
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "RunExplicit RefLocal boundaries are constructed by injecting a RefLocal layer; reaching the non-RefLocal projection branch means a crate-private constructor violated the boundary invariant."
		)]
		pub fn dispatch_run_explicit_ref_local_boundary<
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
			E: Clone + 'a + 'static,
			FirstLayer: 'a,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RunExplicit<'a, R, S, Action>,
			>): Member<BoxRefLocal<'a, BoxBrand, E, RunExplicit<'a, R, S, Action>>, ScopedIdx>,
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
				>, {
			let (layer, continuation) = boundary.into_parts();
			let local = match <Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RunExplicit<'a, R, S, Action>,
				>) as Member<
				BoxRefLocal<'a, BoxBrand, E, RunExplicit<'a, R, S, Action>>,
				ScopedIdx,
			>>::project(layer)
			{
				Ok(local) => local,
				Err(_) => unreachable!(
					"RunExplicit RefLocal boundary contained a non-RefLocal scoped layer"
				),
			};

			match local {
				BoxRefLocal::Local {
					modify,
					action,
				} => {
					let modify = std::cell::RefCell::new(Some(modify));
					let action = std::cell::RefCell::new(Some(action));
					let continuation = std::cell::RefCell::new(Some(continuation));

					RunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
						#[expect(
							clippy::expect_used,
							reason = "Box-backed RefLocal boundary dispatch is single-shot; RunExplicit invokes this continuation once"
						)]
						let modify = modify
							.borrow_mut()
							.take()
							.expect("RunExplicit RefLocal boundary modify invoked more than once");
						#[expect(
							clippy::expect_used,
							reason = "Box-backed RefLocal boundary dispatch is single-shot; RunExplicit invokes this continuation once"
						)]
						let action = action
							.borrow_mut()
							.take()
							.expect("RunExplicit RefLocal boundary action invoked more than once");
						#[expect(
							clippy::expect_used,
							reason = "Box-backed RefLocal boundary dispatch is single-shot; RunExplicit invokes this continuation once"
						)]
						let continuation = continuation.borrow_mut().take().expect(
							"RunExplicit RefLocal boundary continuation invoked more than once",
						);
						let local_env = modify(&env);

						continuation.resume_explicit_with_supplied_action(fo_handlers, move || {
							let local_env = local_env.clone();
							action(()).interpose::<
								BoxReaderBrand<BoxBrand, E>,
								Idx,
								RMinusE,
								EmbedIndices,
							>(move |op| match op {
								BoxReader::Ask(k) => k(local_env.clone()),
							})
						})
					})
				}
			}
		}

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
					OperationValue = Action,
					OperationProgram = RunExplicit<'a, R, S, Action>,
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

		/// Dispatch an indexed `RcRunExplicit` RefLocal boundary.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of values carried by the Rc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected RefLocal action result type.",
			"The final result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Reader environment type.",
			"The type-level Member-position witness for the scoped RefLocal layer.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The indexed RefLocal boundary produced around the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the boundary.")]
		#[document_examples]
		///
		/// ```
		/// let inherited_env = 10;
		/// let local_env = (|env: &i32| *env + 5)(&inherited_env);
		/// assert_eq!(local_env * 2, 30);
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "RcRunExplicit RefLocal boundaries are constructed by injecting a RefLocal layer; reaching the non-RefLocal projection branch means a crate-private constructor violated the boundary invariant."
		)]
		pub fn dispatch_rc_run_explicit_ref_local_boundary<
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
			E: Clone + 'a + 'static,
			FirstLayer: 'a,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, Action>,
			>): Member<RefLocal<'a, RcBrand, E, RcRunExplicit<'a, R, S, Action>>, ScopedIdx>,
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
				>, {
			let (layer, continuation) = boundary.into_parts();
			let local = match <Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcRunExplicit<'a, R, S, Action>,
				>) as Member<
				RefLocal<'a, RcBrand, E, RcRunExplicit<'a, R, S, Action>>,
				ScopedIdx,
			>>::project(layer)
			{
				Ok(local) => local,
				Err(_) => unreachable!(
					"RcRunExplicit RefLocal boundary contained a non-RefLocal scoped layer"
				),
			};

			match local {
				RefLocal::Local {
					modify,
					action,
				} => RcRunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(&env);
					let continuation = continuation.clone();
					let action = action.clone();

					continuation.resume_rc_with_supplied_action(fo_handlers, move || {
						let local_env = local_env.clone();
						action(()).interpose::<ReaderBrand<RcBrand, E>, Idx, RMinusE, EmbedIndices>(
							move |op| match op {
								Reader::Ask(k) => k(local_env.clone()),
							},
						)
					})
				}),
			}
		}

		/// Dispatch an indexed `ArcRunExplicit` RefLocal boundary.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of values carried by the Arc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected RefLocal action result type.",
			"The final result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Reader environment type.",
			"The type-level Member-position witness for the scoped RefLocal layer.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The indexed RefLocal boundary produced around the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the boundary.")]
		#[document_examples]
		///
		/// ```
		/// let inherited_env = 10;
		/// let local_env = (|env: &i32| *env + 5)(&inherited_env);
		/// assert_eq!(local_env * 2, 30);
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "ArcRunExplicit RefLocal boundaries are constructed by injecting a RefLocal layer; reaching the non-RefLocal projection branch means a crate-private constructor violated the boundary invariant."
		)]
		pub fn dispatch_arc_run_explicit_ref_local_boundary<
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
			E: Clone + Send + Sync + 'a + 'static,
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
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Action>,
			>): Send
				+ Sync
				+ Member<SendRefLocal<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, Action>>, ScopedIdx>,
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
				>, {
			let (layer, continuation) = boundary.into_parts();
			let local = match <Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcRunExplicit<'a, R, S, Action>,
				>) as Member<
				SendRefLocal<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, Action>>,
				ScopedIdx,
			>>::project(layer)
			{
				Ok(local) => local,
				Err(_) => unreachable!(
					"ArcRunExplicit RefLocal boundary contained a non-RefLocal scoped layer"
				),
			};

			match local {
				SendRefLocal::Local {
					modify,
					action,
				} => ArcRunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(&env);
					let continuation = continuation.clone();
					let action = action.clone();

					continuation.resume_arc_with_supplied_action(fo_handlers, move || {
						let local_env = local_env.clone();
						action(())
							.interpose::<SendReaderBrand<ArcBrand, E>, Idx, RMinusE, EmbedIndices>(
								move |op| match op {
									SendReader::Ask(k) => k(local_env.clone()),
								},
							)
					})
				}),
			}
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
					OperationValue = Action,
					OperationProgram = RcRunExplicit<'a, R, S, Action>,
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
					OperationValue = Action,
					OperationProgram = ArcRunExplicit<'a, R, S, Action>,
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

	#[document_type_parameters(
		"The lifetime of values carried by the shared Explicit boundary.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The Reader environment type.",
		"The first-order row index witnessing the Reader operation.",
		"The first-order row brand with the Reader operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The RefLocal dispatcher receiver.")]
	impl<'a, R, S, Action, Final, K, E, Idx, RMinusE, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			RefLocal<'a, RcBrand, E, RcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RcRunExplicit<'a, R, S, Final>,
			RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for RefLocalHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Final: 'a,
		K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a,
		E: Clone + 'a + 'static,
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
	{
		/// Run the Rc selected action under a borrowed-environment transform and resume the outer continuation.
		#[document_signature]
		#[document_parameters(
			"The RefLocal layer carrying the borrowed environment transform and selected action.",
			"The wrapper-owned continuation carrier.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns(
			"The final `RcRunExplicit` program produced by the boundary dispatcher."
		)]
		#[document_examples]
		///
		/// ```
		/// let inherited_env = 10;
		/// let local_env = (|env: &i32| *env + 1)(&inherited_env);
		/// let outer = |value| value + 1;
		/// assert_eq!(outer(local_env * 2), 23);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: RefLocal<'a, RcBrand, E, RcRunExplicit<'a, R, S, Action>>,
			continuation: crate::types::effects::interpreter::ScopedContinuation<
				RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
		) -> RcRunExplicit<'a, R, S, Final> {
			let outer = continuation.into_inner().outer.clone();
			match layer {
				RefLocal::Local {
					modify,
					action,
				} => RcRunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(&env);
					let action = action.clone();
					let outer = outer.clone();

					action(())
						.interpose::<ReaderBrand<RcBrand, E>, Idx, RMinusE, EmbedIndices>(
							move |op| match op {
								Reader::Ask(k) => k(local_env.clone()),
							},
						)
						.bind(move |action_value| outer(action_value))
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
		"The Reader environment type.",
		"The first-order row index witnessing the Reader operation.",
		"The first-order row brand with the Reader operation removed.",
		"The embedding witness used to rebuild the original first-order row.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The RefLocal dispatcher receiver.")]
	impl<'a, R, S, Action, Final, K, E, Idx, RMinusE, EmbedIndices, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			SendRefLocal<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			ArcRunExplicit<'a, R, S, Final>,
			ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for RefLocalHandler<Idx, RMinusE, EmbedIndices>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Action: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Action) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
		E: Clone + Send + Sync + 'a + 'static,
		FirstLayer: 'a,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		SendReaderBrand<ArcBrand, E>: SendFunctor
			+ Kind_cdc7cd43dac7585f<
				Of<'a, E> = SendReader<'a, ArcBrand, E, E>,
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
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
		>): Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Send + Sync,
		Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Send + Sync,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, E>):
			Member<ArcCoyoneda<'a, SendReaderBrand<ArcBrand, E>, E>, Idx>,
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
		/// Run the Arc selected action under a borrowed-environment transform and resume the outer continuation.
		#[document_signature]
		#[document_parameters(
			"The RefLocal layer carrying the borrowed environment transform and selected action.",
			"The wrapper-owned continuation carrier.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns(
			"The final `ArcRunExplicit` program produced by the boundary dispatcher."
		)]
		#[document_examples]
		///
		/// ```
		/// let inherited_env = 10;
		/// let local_env = (|env: &i32| *env + 1)(&inherited_env);
		/// let outer = |value| value + 1;
		/// assert_eq!(outer(local_env * 2), 23);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: SendRefLocal<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, Action>>,
			continuation: crate::types::effects::interpreter::ScopedContinuation<
				ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
		) -> ArcRunExplicit<'a, R, S, Final> {
			let outer = continuation.into_inner().outer.clone();
			match layer {
				SendRefLocal::Local {
					modify,
					action,
				} => ArcRunExplicit::<R, S, E>::ask::<Idx>().bind(move |env| {
					let local_env = modify(&env);
					let action = action.clone();
					let outer = outer.clone();

					action(())
						.interpose::<SendReaderBrand<ArcBrand, E>, Idx, RMinusE, EmbedIndices>(
							move |op| match op {
								SendReader::Ask(k) => k(local_env.clone()),
							},
						)
						.bind(move |action_value| outer(action_value))
				}),
			}
		}
	}
}
