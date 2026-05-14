#[allow(
	unused_imports,
	reason = "Each scoped-dispatcher child module consumes a different subset of the shared parent prelude."
)]
use super::prelude::*;

#[fp_macros::document_module]
mod inner {
	use super::*;

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
	#[document_parameters("The Span dispatcher receiver.")]
	impl<'a, R, S, Action, Final, K, Tag, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			BoxSpan<'a, BoxBrand, Tag, RunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RunExplicit<'a, R, S, Final>,
			RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for SpanDispatcher
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
		#[document_examples]
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
		/// struct Boundary<Tag, Action, Outer> {
		/// 	tag: Tag,
		/// 	action: Action,
		/// 	outer: Outer,
		/// }
		///
		/// impl<Tag, Action, Outer> Boundary<Tag, Action, Outer> {
		/// 	fn dispatch<Final>(
		/// 		self,
		/// 		post_action: impl Fn(&Tag, Action) -> Action,
		/// 	) -> Final
		/// 	where
		/// 		Outer: Fn(Action) -> Final, {
		/// 		(self.outer)(post_action(&self.tag, self.action))
		/// 	}
		/// }
		///
		/// let result = Boundary {
		/// 	tag: "request",
		/// 	action: 40,
		/// 	outer: |value| value + 1,
		/// }
		/// .dispatch(|tag, value| {
		/// 	assert_eq!(*tag, "request");
		/// 	value + 1
		/// });
		/// assert_eq!(result, 42);
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
			boundary: RunExplicitBoundary<'a, R, S, Action, Final, K>,
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
		/// 		scoped_dispatchers::span_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, String>, CNilBrand>;
		///
		/// let action: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(41);
		/// let boundary = RcRunExplicit::span::<String, _>("request".to_owned(), action);
		/// let prog: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = span_dispatcher()
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
			boundary: RcRunExplicitBoundary<'a, R, S, Action, Final, K>,
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
		/// 		scoped_dispatchers::span_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, String>, CNilBrand>;
		///
		/// let action: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(41);
		/// let boundary = ArcRunExplicit::span::<String, _>("request".to_owned(), action);
		/// let prog: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = span_dispatcher()
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
			boundary: ArcRunExplicitBoundary<'a, R, S, Action, Final, K>,
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
		DispatchRcRunRawScopedHandler<R, S, A, SpanBrand<RcBrand, Tag>, FirstLayer> for SpanDispatcher
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		rc_run::RcRun,
		/// 		scoped_dispatchers::span_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, i32>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = RcRun::span::<i32, _>(7, RcRun::pure(42));
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SpanBrand<RcBrand, i32>: span_dispatcher(),
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
		DispatchArcRunRawScopedHandler<R, S, A, SendSpanBrand<ArcBrand, Tag>, FirstLayer>
		for SpanDispatcher
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
			"The first-order handler list available to the scoped dispatcher."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		scoped_dispatchers::span_dispatcher,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, i32>, CNilBrand>;
		/// type Prog = ArcRun<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = ArcRun::span::<i32, _>(7, ArcRun::pure(42));
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SendSpanBrand<ArcBrand, i32>: span_dispatcher(),
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
	#[document_parameters("The Span dispatcher receiver.")]
	impl<'a, R, S, Action, Final, K, Tag, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			Span<'a, RcBrand, Tag, RcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			RcRunExplicit<'a, R, S, Final>,
			RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for SpanDispatcher
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
				> + RcActionSuppliedScopedResume<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
	{
		/// Resume the Rc Span action and then apply the stored outer continuation.
		#[document_signature]
		#[document_parameters(
			"The Span layer carrying the selected action program.",
			"The wrapper-owned continuation carrier.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns(
			"The final `RcRunExplicit` program produced by the boundary dispatcher."
		)]
		#[document_examples]
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
	#[document_parameters("The Span dispatcher receiver.")]
	impl<'a, R, S, Action, Final, K, Tag, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			SendSpan<'a, ArcBrand, Tag, ArcRunExplicit<'a, R, S, Action>>,
			FirstLayer,
			ArcRunExplicit<'a, R, S, Final>,
			ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K>,
		> for SpanDispatcher
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
				> + ArcActionSuppliedScopedResume<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
	{
		/// Resume the Arc Span action and then apply the stored outer continuation.
		#[document_signature]
		#[document_parameters(
			"The Span layer carrying the selected action program.",
			"The wrapper-owned continuation carrier.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns(
			"The final `ArcRunExplicit` program produced by the boundary dispatcher."
		)]
		#[document_examples]
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

pub use inner::*;
