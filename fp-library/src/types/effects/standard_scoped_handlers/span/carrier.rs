#[fp_macros::document_module]
pub(crate) mod inner {
	use super::super::{
		super::prelude::*,
		inner::SpanHandler,
	};

	/// Carrier-aware Span dispatch for `RunExplicitBoundary`.
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
	#[document_parameters("The Span handler receiver.")]
	impl<'a, R, S, Action, Final, K, Tag, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			BoxSpan<'a, BoxBrand, Tag, RunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RunExplicit<'a, R, S, Final>,
			RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for SpanHandler
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: 'a,
		Final: 'a,
		K: Fn(Action) -> RunExplicit<'a, R, S, Final> + 'a,
		Tag: 'a,
		FirstLayer: 'a,
		RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>:
			ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = RunExplicit<'a, R, S, Action>,
					OperationValue = Action,
					OperationProgram = RunExplicit<'a, R, S, Action>,
				> + ExplicitActionSuppliedScopedResume<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
	{
		/// Resume the Span action unchanged and then run the boundary's
		/// outer continuation.
		#[document_signature]
		#[document_parameters(
			"The Span scoped layer carrying the selected action program.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the Span boundary.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped-carrier protocol hook receives a crate-private ScopedContinuation produced by the interpreter; external examples cannot construct that continuation directly, so the example documents the Span action and outer-continuation semantics."
		)]
		///
		/// ```
		/// let action_value = 41;
		/// let outer = |value| value + 1;
		/// assert_eq!(outer(action_value), 42);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: BoxSpan<'a, BoxBrand, Tag, RunExplicit<'a, R, S, Action>>,
			continuation: ScopedContinuation<
				RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
		) -> RunExplicit<'a, R, S, Final> {
			match layer {
				BoxSpan::Span {
					tag,
					action,
				} => continuation.resume_explicit_with_supplied_action(fo_handlers, move || {
					action(()).bind(move |action_value| {
						let _ = &tag;
						RunExplicit::pure(action_value)
					})
				}),
			}
		}
	}

	#[document_parameters("The Span handler receiver.")]
	#[cfg_attr(
		not(test),
		expect(
			dead_code,
			reason = "The focused RunExplicit Span carrier-cell proof is exercised by tests before the full wrapper interpreter route consumes it."
		)
	)]
	impl SpanHandler {
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
		#[document_examples(
			skip_call_check,
			reason = "This private RunExplicit Span carrier helper consumes a crate-private carrier layer and continuation cell constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the tag-aware post-action behaviour."
		)]
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
					OperationValue = Action,
					OperationProgram = RunExplicit<'a, R, S, Action>,
				> + ExplicitScopedResume<'a, FirstLayer, RunExplicit<'a, R, S, Final>>, {
			let (tag, continuation) = layer.into_parts();

			continuation.resume_explicit_with_post_action(fo_handlers, move |action_value| {
				post_action(&tag, action_value)
			})
		}

		/// Dispatch a `RunExplicit` indexed Span boundary.
		///
		/// The boundary stores the selected action in the scoped row layer
		/// and keeps the final-result continuation separately. This method
		/// projects the Span layer, observes the tag, runs the selected
		/// action with result-preserving post-action work, and resumes the
		/// outer continuation to produce the final `RunExplicit` program.
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
			"The row index witnessing the Span operation.",
			"The first-order handler layer type."
		)]
		///
		#[document_parameters(
			"The indexed Span boundary produced by `RunExplicit::span`.",
			"The first-order handler list available while resuming the selected action.",
			"The result-preserving callback to run while the Span tag is in scope."
		)]
		///
		#[document_returns("The final `RunExplicit` program produced by the Span boundary.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::effects::{
		/// 		run_explicit::RunExplicit,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
		///
		/// let action: RunExplicit<'static, FirstRow, ScopedRow, i32> = RunExplicit::pure(42);
		/// let boundary = RunExplicit::span::<&'static str, _>("request", action).map(|value| value + 1);
		/// let program = span_handler().dispatch_run_explicit_span_boundary_with_post_action(
		/// 	boundary,
		/// 	&handlers! {},
		/// 	|tag, value| {
		/// 		assert_eq!(*tag, "request");
		/// 		RunExplicit::pure(value)
		/// 	},
		/// );
		/// assert!(matches!(program.peel(), Ok(43)));
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "RunExplicit::span constructs this boundary by injecting a Span layer; reaching the non-Span projection branch means a crate-private constructor violated the boundary invariant."
		)]
		pub fn dispatch_run_explicit_span_boundary_with_post_action<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			Tag,
			Idx,
			FirstLayer,
		>(
			&self,
			boundary: RunExplicitBoundary<
				'a,
				R,
				S,
				BoxSpanBrand<BoxBrand, Tag>,
				Idx,
				Action,
				Final,
				K,
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
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RunExplicit<'a, R, S, Action>,
			>): Member<BoxSpan<'a, BoxBrand, Tag, RunExplicit<'a, R, S, Action>>, Idx>, {
			let (layer, continuation) = boundary.into_parts();
			let span =
				match <Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RunExplicit<'a, R, S, Action>,
				>) as Member<BoxSpan<'a, BoxBrand, Tag, RunExplicit<'a, R, S, Action>>, Idx>>::project(
					layer
				) {
					Ok(span) => span,
					Err(_) =>
						unreachable!("RunExplicit Span boundary contained a non-Span scoped layer"),
				};

			match span {
				BoxSpan::Span {
					tag,
					action,
				} => continuation.resume_explicit_with_supplied_action(fo_handlers, move || {
					action(()).bind(move |action_value| post_action(&tag, action_value))
				}),
			}
		}

		/// Dispatch an indexed `RcRunExplicit` Span boundary.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of values carried by the Rc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Span action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Span tag type.",
			"The scoped-row Member witness for the Span layer.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The indexed Span boundary returned by `RcRunExplicit::span`.",
			"The first-order handler list available while resuming the selected action.",
			"The result-preserving action callback to run before the outer continuation."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the Span boundary.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::effects::{
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, String>, CNilBrand>;
		///
		/// let action: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(41);
		/// let boundary = RcRunExplicit::span::<String, _>("request".to_owned(), action);
		/// let prog: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = span_handler()
		/// 	.dispatch_rc_run_explicit_span_boundary_with_post_action(
		/// 		boundary,
		/// 		&handlers! {},
		/// 		|tag, value| {
		/// 			assert_eq!(tag.as_str(), "request");
		/// 			RcRunExplicit::pure(value + 1)
		/// 		},
		/// 	);
		/// assert!(matches!(prog.peel(), Ok(42)));
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "RcRunExplicit::span constructs this boundary by injecting a Span layer; reaching the non-Span projection branch means a crate-private constructor violated the boundary invariant."
		)]
		pub fn dispatch_rc_run_explicit_span_boundary_with_post_action<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			Tag,
			Idx,
			FirstLayer,
		>(
			&self,
			boundary: RcRunExplicitBoundary<
				'a,
				R,
				S,
				SpanBrand<RcBrand, Tag>,
				Idx,
				Action,
				Final,
				K,
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
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, Action>,
			>): Member<Span<'a, RcBrand, Tag, RcRunExplicit<'a, R, S, Action>>, Idx>,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone, {
			let (layer, continuation) = boundary.into_parts();
			let span = match <Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcRunExplicit<'a, R, S, Action>,
				>) as Member<Span<'a, RcBrand, Tag, RcRunExplicit<'a, R, S, Action>>, Idx>>::project(
				layer
			) {
				Ok(span) => span,
				Err(_) =>
					unreachable!("RcRunExplicit Span boundary contained a non-Span scoped layer"),
			};

			match span {
				Span::Span {
					tag,
					action,
				} => continuation.resume_rc_with_supplied_action(fo_handlers, move || {
					action(()).bind(move |action_value| post_action(&tag, action_value))
				}),
			}
		}

		/// Dispatch an indexed `ArcRunExplicit` Span boundary.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of values carried by the Arc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The selected Span action result type.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The Span tag type.",
			"The scoped-row Member witness for the Span layer.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The indexed Span boundary returned by `ArcRunExplicit::span`.",
			"The first-order handler list available while resuming the selected action.",
			"The result-preserving action callback to run before the outer continuation."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the Span boundary.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, String>, CNilBrand>;
		///
		/// let action: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(41);
		/// let boundary = ArcRunExplicit::span::<String, _>("request".to_owned(), action);
		/// let prog: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = span_handler()
		/// 	.dispatch_arc_run_explicit_span_boundary_with_post_action(
		/// 		boundary,
		/// 		&handlers! {},
		/// 		|tag, value| {
		/// 			assert_eq!(tag.as_str(), "request");
		/// 			ArcRunExplicit::pure(value + 1)
		/// 		},
		/// 	);
		/// assert!(matches!(prog.peel(), Ok(42)));
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "ArcRunExplicit::span constructs this boundary by injecting a Span layer; reaching the non-Span projection branch means a crate-private constructor violated the boundary invariant."
		)]
		pub fn dispatch_arc_run_explicit_span_boundary_with_post_action<
			'a,
			R,
			S,
			Action,
			Final,
			K,
			Tag,
			Idx,
			FirstLayer,
		>(
			&self,
			boundary: ArcRunExplicitBoundary<
				'a,
				R,
				S,
				SendSpanBrand<ArcBrand, Tag>,
				Idx,
				Action,
				Final,
				K,
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
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Action>,
			>): Member<SendSpan<'a, ArcBrand, Tag, ArcRunExplicit<'a, R, S, Action>>, Idx>
				+ Send
				+ Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone + Send + Sync, {
			let (layer, continuation) = boundary.into_parts();
			let span = match <Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcRunExplicit<'a, R, S, Action>,
				>) as Member<
				SendSpan<'a, ArcBrand, Tag, ArcRunExplicit<'a, R, S, Action>>,
				Idx,
			>>::project(layer)
			{
				Ok(span) => span,
				Err(_) =>
					unreachable!("ArcRunExplicit Span boundary contained a non-Span scoped layer"),
			};

			match span {
				SendSpan::Span {
					tag,
					action,
				} => continuation.resume_arc_with_supplied_action(fo_handlers, move || {
					action(()).bind(move |action_value| post_action(&tag, action_value))
				}),
			}
		}

		/// Dispatch a private `RcRunExplicit` Span carrier-cell layer.
		///
		/// This is the shared-Rc counterpart to
		/// [`dispatch_run_explicit_span_carrier_with_post_action`](SpanHandler::dispatch_run_explicit_span_carrier_with_post_action).
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
		#[document_examples(
			skip_call_check,
			reason = "This private RcRunExplicit Span carrier helper consumes a crate-private carrier layer and continuation cell constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the Rc tag-aware post-action behaviour."
		)]
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
					OperationValue = Action,
					OperationProgram = RcRunExplicit<'a, R, S, Action>,
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
		#[document_examples(
			skip_call_check,
			reason = "This private ArcRunExplicit Span carrier helper consumes a crate-private carrier layer and continuation cell constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the thread-safe tag-aware post-action behaviour."
		)]
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
					OperationValue = Action,
					OperationProgram = ArcRunExplicit<'a, R, S, Action>,
				> + ArcScopedResume<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>, {
			let (tag, continuation) = layer.into_parts();

			continuation.resume_arc_with_post_action(fo_handlers, move |action_value| {
				post_action(&tag, action_value)
			})
		}
	}

	// Carrier-aware public-boundary facade impls for the shared Explicit
	// wrappers. These consume the original scoped layer plus the typed
	// action-supplied continuation produced by `RcRunExplicitBoundary` /
	// `ArcRunExplicitBoundary`.

	#[document_type_parameters(
		"The lifetime of values carried by the shared Explicit boundary.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The Span tag type.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Span handler receiver.")]
	impl<'a, R, S, Action, Final, K, Tag, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			Span<'a, RcBrand, Tag, RcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RcRunExplicit<'a, R, S, Final>,
			RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for SpanHandler
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Final: 'a,
		K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a,
		Tag: 'a,
		FirstLayer: 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone,
		RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>:
			ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = RcRunExplicit<'a, R, S, Action>,
					OperationValue = Action,
					OperationProgram = RcRunExplicit<'a, R, S, Action>,
				> + RcActionSuppliedScopedResume<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
	{
		/// Resume the Rc Span action and then apply the stored outer continuation.
		#[document_signature]
		#[document_parameters(
			"The Span layer carrying the selected action program.",
			"The wrapper-owned continuation carrier.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the boundary handler.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped-carrier protocol hook receives a crate-private RcRunExplicit ScopedContinuation produced by the interpreter; external examples cannot construct that continuation directly, so the example documents the Span action and outer-continuation semantics."
		)]
		///
		/// ```
		/// let action_value = 41;
		/// let outer = |value| value + 1;
		/// assert_eq!(outer(action_value), 42);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: Span<'a, RcBrand, Tag, RcRunExplicit<'a, R, S, Action>>,
			continuation: ScopedContinuation<
				RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
		) -> RcRunExplicit<'a, R, S, Final> {
			match layer {
				Span::Span {
					tag,
					action,
				} => continuation.resume_rc_with_supplied_action(fo_handlers, move || {
					action(()).bind(move |action_value| {
						let _ = &tag;
						RcRunExplicit::pure(action_value)
					})
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
		"The Span tag type.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Span handler receiver.")]
	impl<'a, R, S, Action, Final, K, Tag, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			SendSpan<'a, ArcBrand, Tag, ArcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			ArcRunExplicit<'a, R, S, Final>,
			ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for SpanHandler
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Action: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Action) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
		Tag: Send + Sync + 'a,
		FirstLayer: 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone + Send + Sync,
		ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>:
			ScopedResumeTypes<
					'a,
					ActionValue = Action,
					ActionProgram = ArcRunExplicit<'a, R, S, Action>,
					OperationValue = Action,
					OperationProgram = ArcRunExplicit<'a, R, S, Action>,
				> + ArcActionSuppliedScopedResume<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
	{
		/// Resume the Arc Span action and then apply the stored outer continuation.
		#[document_signature]
		#[document_parameters(
			"The Span layer carrying the selected action program.",
			"The wrapper-owned continuation carrier.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the boundary handler.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped-carrier protocol hook receives a crate-private ArcRunExplicit ScopedContinuation produced by the interpreter; external examples cannot construct that continuation directly, so the example documents the thread-safe Span action and outer-continuation semantics."
		)]
		///
		/// ```
		/// let action_value = 41;
		/// let outer = |value| value + 1;
		/// assert_eq!(outer(action_value), 42);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: SendSpan<'a, ArcBrand, Tag, ArcRunExplicit<'a, R, S, Action>>,
			continuation: ScopedContinuation<
				ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
		) -> ArcRunExplicit<'a, R, S, Final> {
			match layer {
				SendSpan::Span {
					tag,
					action,
				} => continuation.resume_arc_with_supplied_action(fo_handlers, move || {
					action(()).bind(move |action_value| {
						let _ = &tag;
						ArcRunExplicit::pure(action_value)
					})
				}),
			}
		}
	}
}
