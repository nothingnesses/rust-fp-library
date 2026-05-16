#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		super::super::inner::ArcRunExplicit,
		crate::{
			Apply,
			brands::{
				ArcBrand,
				NodeBrand,
			},
			classes::{
				RefCountedPointer,
				SendFunctor,
				WrapDrop,
			},
			kinds::*,
			types::{
				ArcFreeExplicit,
				effects::{
					interpreter::{
						ArcActionSuppliedScopedResume,
						ArcScopedResume,
						DispatchHandlers,
						DispatchResidualScopedHandlers,
						DispatchScopedBoundaryHandlers,
						IntoScopedBoundaryParts,
						ScopedContinuation,
						ScopedResumeTypes,
					},
					node::Node,
				},
			},
		},
		core::marker::PhantomData,
		fp_macros::*,
	};
	#[doc(hidden)]
	/// Arc-backed Explicit carrier for a selected scoped action.
	///
	/// The carrier keeps the action and the action's outer continuation
	/// separate while preserving the `ArcFreeExplicit` multi-shot contract:
	/// cloning the carrier is O(1), and post-action work is a reusable
	/// `Send + Sync` `Fn` continuation over the selected action value.
	#[derive(Clone)]
	pub(crate) struct ArcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Action: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Action) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a, {
		/// The selected scoped action before its outer continuation has
		/// been reattached.
		pub(crate) action: ArcRunExplicit<'a, R, S, Action>,
		/// The action's outer continuation, still outside the selected action.
		pub(crate) outer: <ArcBrand as RefCountedPointer>::Of<'a, K>,
		/// Carries the final result type without owning a value of that type.
		pub(crate) result: PhantomData<fn() -> Final>,
	}

	#[doc(hidden)]
	/// Arc-backed Explicit carrier for an action supplied by a dispatcher.
	///
	/// The carrier stores the thread-safe shared outer continuation without
	/// storing the selected action itself. Indexed around-action boundaries
	/// provide that action from the scoped row; Bracket-style dispatchers
	/// provide it after lifecycle work has determined the value that should
	/// flow into the outer continuation.
	#[allow(
		dead_code,
		reason = "Bracket carrier wiring consumes the Arc action-supplied carrier in the next implementation step; focused tests exercise the private shape until then."
	)]
	pub(crate) struct ArcRunExplicitActionSuppliedScopedContinuation<
		'a,
		R,
		S,
		Action,
		Final,
		K,
		Operation = Action,
	>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Action: Clone + Send + Sync + 'a,
		Operation: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Operation) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a, {
		/// The operation-result outer continuation, still outside the selected
		/// action.
		pub(crate) outer: <ArcBrand as RefCountedPointer>::Of<'a, K>,
		/// Carries the selected action, operation result, and final result types
		/// without owning values of those types.
		pub(crate) result: PhantomData<fn(Action) -> (Operation, Final)>,
	}

	#[document_type_parameters(
		"The lifetime that bounds the carrier payload.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The operation result type passed from the scoped operation to the outer continuation."
	)]
	#[document_parameters("The action-supplied ArcRunExplicit scoped-continuation carrier.")]
	impl<'a, R, S, Action, Final, K, Operation> Clone
		for ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Action: Clone + Send + Sync + 'a,
		Operation: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Operation) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
	{
		/// Clone the carrier by refcount-bumping the shared outer
		/// continuation.
		#[document_signature]
		#[document_returns("A carrier sharing the same outer continuation.")]
		#[document_examples]
		///
		/// ```
		/// use std::sync::Arc;
		///
		/// let outer = Arc::new(|value: i32| value + 1);
		/// let cloned = Arc::clone(&outer);
		/// assert_eq!(outer(41), 42);
		/// assert_eq!(cloned(41), 42);
		/// ```
		fn clone(&self) -> Self {
			Self {
				outer: self.outer.clone(),
				result: PhantomData,
			}
		}
	}

	/// Production indexed boundary for `ArcRunExplicit` around-action
	/// scoped operations.
	///
	/// The boundary keeps the selected action in the scoped row projection
	/// and stores the thread-safe shared outer continuation separately.
	/// Boundary `map` and `bind` compose only that outer continuation,
	/// preserving the multi-shot `ArcRunExplicit` selected-action slot
	/// until a scoped handler resumes it.
	#[document_type_parameters(
		"The lifetime that bounds the boundary payload.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The scoped-effect brand consumed by this boundary.",
		"The scoped-row member index consumed by this boundary.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The operation result type passed from the scoped operation to the outer continuation."
	)]
	pub struct ArcRunExplicitBoundary<'a, R, S, SBrand, Idx, Action, Final, K, Operation = Action>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		SBrand: 'static,
		Idx: 'a,
		Action: Clone + Send + Sync + 'a,
		Operation: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Operation) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a, {
		/// The scoped row layer carrying the selected action program.
		layer: Apply!(
			<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, Action>>
		),
		/// The wrapper-owned continuation from operation result to final result.
		continuation: ScopedContinuation<
			ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>,
		>,
		/// Type-level evidence for the scoped-row member consumed as this
		/// boundary's head operation.
		member: PhantomData<fn() -> (SBrand, Idx)>,
	}

	#[document_type_parameters(
		"The lifetime that bounds the boundary payload.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The scoped-effect brand consumed by this boundary.",
		"The scoped-row member index consumed by this boundary.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The operation result type passed from the scoped operation to the outer continuation."
	)]
	#[document_parameters("The `ArcRunExplicit` indexed scoped boundary.")]
	impl<'a, R, S, SBrand, Idx, Action, Final, K, Operation>
		ArcRunExplicitBoundary<'a, R, S, SBrand, Idx, Action, Final, K, Operation>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		SBrand: 'a,
		Idx: 'a,
		Action: Clone + Send + Sync + 'a,
		Operation: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Operation) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
	{
		/// Construct an indexed boundary from a scoped layer and an outer
		/// continuation.
		#[document_signature]
		#[document_parameters(
			"The scoped row layer carrying the selected action program.",
			"The outer continuation from selected action result to final program."
		)]
		#[document_returns(
			"A boundary that stores the action layer and outer continuation separately."
		)]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// struct Boundary<Layer, Outer> {
		/// 	layer: Layer,
		/// 	outer: Outer,
		/// }
		///
		/// let boundary = Boundary {
		/// 	layer: "selected action",
		/// 	outer: |value: i32| value + 1,
		/// };
		/// assert_eq!(boundary.layer, "selected action");
		/// assert_eq!((boundary.outer)(41), 42);
		/// ```
		pub(crate) fn new(
			layer: Apply!(
				<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, Action>>
			),
			outer: K,
		) -> Self {
			Self {
				layer,
				continuation: ScopedContinuation::new(
					ArcRunExplicitActionSuppliedScopedContinuation {
						outer: <ArcBrand as RefCountedPointer>::new(outer),
						result: PhantomData,
					},
				),
				member: PhantomData,
			}
		}

		/// Compose a final-result continuation onto this boundary.
		#[document_signature]
		#[document_type_parameters("The result type produced after the additional continuation.")]
		#[document_parameters(
			"The continuation to run after the existing outer continuation completes."
		)]
		#[document_returns(
			"A boundary with the same action layer and a composed outer continuation."
		)]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use std::sync::Arc;
		///
		/// let outer = Arc::new(|value: i32| value + 1);
		/// let f = Arc::new(|value: i32| value * 2);
		/// let composed = {
		/// 	let outer = Arc::clone(&outer);
		/// 	let f = Arc::clone(&f);
		/// 	move |value| f(outer(value))
		/// };
		/// assert_eq!(composed(20), 42);
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "The boundary return type exposes the private action/operation/final carrier shape that scoped handlers consume."
		)]
		pub fn bind<Next>(
			self,
			f: impl Fn(Final) -> ArcRunExplicit<'a, R, S, Next> + Send + Sync + 'a,
		) -> ArcRunExplicitBoundary<
			'a,
			R,
			S,
			SBrand,
			Idx,
			Action,
			Next,
			impl Fn(Operation) -> ArcRunExplicit<'a, R, S, Next> + Send + Sync + 'a,
			Operation,
		>
		where
			Final: Clone,
			Next: Send + Sync + 'a,
			K: 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Next>,
			>): Clone + Send + Sync, {
			let Self {
				layer,
				continuation,
				member: _,
			} = self;
			let carrier = continuation.into_inner();
			let outer = carrier.outer.clone();
			let f = <ArcBrand as RefCountedPointer>::new(f);
			let composed = move |operation_value: Operation| {
				let f = f.clone();
				outer(operation_value).bind(move |final_value| f(final_value))
			};

			ArcRunExplicitBoundary::new(layer, composed)
		}

		/// Map over the final result while leaving the selected action
		/// layer unchanged.
		#[document_signature]
		#[document_type_parameters("The mapped final result type.")]
		#[document_parameters("The function to apply after the outer continuation completes.")]
		#[document_returns("A boundary with the same action layer and mapped final continuation.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// let mapped = |value: i32| (value + 1) * 2;
		/// assert_eq!(mapped(20), 42);
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "The boundary return type preserves the private action/operation/final carrier shape across final-result mapping."
		)]
		pub fn map<Next>(
			self,
			f: impl Fn(Final) -> Next + Send + Sync + 'a,
		) -> ArcRunExplicitBoundary<
			'a,
			R,
			S,
			SBrand,
			Idx,
			Action,
			Next,
			impl Fn(Operation) -> ArcRunExplicit<'a, R, S, Next> + Send + Sync + 'a,
			Operation,
		>
		where
			Final: Clone,
			Next: Send + Sync + 'a,
			K: 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Next>,
			>): Clone + Send + Sync, {
			let f = <ArcBrand as RefCountedPointer>::new(f);

			self.bind(move |final_value| {
				let f = f.clone();
				ArcRunExplicit::pure(f(final_value))
			})
		}

		/// Interpret this indexed boundary through a public scoped-handler
		/// facade.
		#[document_signature]
		#[document_parameters(
			"The first-order handler list used while resuming the selected action and later first-order layers.",
			"The scoped-handler list that can consume this boundary and later ordinary scoped layers."
		)]
		#[document_returns(
			"The final result value after the boundary and produced program finish."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, &'static str>, CNilBrand>;
		/// type Prog = ArcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = ArcRunExplicit::pure(41);
		/// let boundary =
		/// 	ArcRunExplicit::span::<&'static str, _>("request", action).map(|value| value + 1);
		///
		/// let result = boundary.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SendSpanBrand<ArcBrand, &'static str>: span_handler(),
		/// 	},
		/// );
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		pub fn handle(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'h,
					ArcRunExplicit<'a, R, S, Final>,
				>),
				ArcRunExplicit<'a, R, S, Final>,
			>,
			scoped_handlers: impl DispatchScopedBoundaryHandlers<
				'a,
				Self,
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcRunExplicit<'a, R, S, Final>,
				>),
				ArcRunExplicit<'a, R, S, Final>,
			> + DispatchResidualScopedHandlers<
				'a,
				SBrand,
				Idx,
				Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcRunExplicit<'a, R, S, Final>,
				>),
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcRunExplicit<'a, R, S, Final>,
				>),
				ArcRunExplicit<'a, R, S, Final>,
			>,
		) -> Final
		where
			Final: Clone,
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
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Final>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Final>,
			>): Send + Sync, {
			let mut prog = scoped_handlers.dispatch_scoped_boundary(self, &handlers);
			loop {
				match prog.peel() {
					Ok(final_value) => return final_value,
					Err(Node::First(layer)) => prog = handlers.dispatch(layer),
					Err(Node::Scoped(layer)) =>
						prog = scoped_handlers.dispatch_residual_scoped(layer, &handlers),
				}
			}
		}

		/// Alias for [`handle`](ArcRunExplicitBoundary::handle).
		#[document_signature]
		#[document_parameters(
			"The first-order handler list used while interpreting the selected action and later first-order layers.",
			"The scoped-handler list that can consume this boundary and later ordinary scoped layers."
		)]
		#[document_returns(
			"The final result value after the boundary and produced program finish."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, &'static str>, CNilBrand>;
		/// type Prog = ArcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = ArcRunExplicit::pure(41);
		/// let boundary =
		/// 	ArcRunExplicit::span::<&'static str, _>("request", action).map(|value| value + 1);
		///
		/// let result = boundary.run(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SendSpanBrand<ArcBrand, &'static str>: span_handler(),
		/// 	},
		/// );
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		pub fn run(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'h,
					ArcRunExplicit<'a, R, S, Final>,
				>),
				ArcRunExplicit<'a, R, S, Final>,
			>,
			scoped_handlers: impl DispatchScopedBoundaryHandlers<
				'a,
				Self,
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcRunExplicit<'a, R, S, Final>,
				>),
				ArcRunExplicit<'a, R, S, Final>,
			> + DispatchResidualScopedHandlers<
				'a,
				SBrand,
				Idx,
				Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcRunExplicit<'a, R, S, Final>,
				>),
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcRunExplicit<'a, R, S, Final>,
				>),
				ArcRunExplicit<'a, R, S, Final>,
			>,
		) -> Final
		where
			Final: Clone,
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
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Final>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Final>,
			>): Send + Sync, {
			self.handle(handlers, scoped_handlers)
		}

		/// Split the boundary into its action layer and scoped
		/// continuation carrier.
		#[document_signature]
		#[document_returns(
			"The scoped row layer and wrapper-owned continuation carrier stored by the boundary."
		)]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// let layer = "selected action";
		/// let continuation = "outer continuation";
		/// let parts = (layer, continuation);
		/// assert_eq!(parts.0, "selected action");
		/// assert_eq!(parts.1, "outer continuation");
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "The split returns the explicit S::Of projection and continuation carrier that downstream scoped dispatch consumes."
		)]
		pub(crate) fn into_parts(
			self
		) -> (
			Apply!(
				<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, Action>>
			),
			ScopedContinuation<
				ArcRunExplicitActionSuppliedScopedContinuation<
					'a,
					R,
					S,
					Action,
					Final,
					K,
					Operation,
				>,
			>,
		) {
			(self.layer, self.continuation)
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the boundary payload.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The scoped-effect brand consumed by this boundary.",
		"The scoped-row member index consumed by this boundary.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The operation result type passed from the scoped operation to the outer continuation."
	)]
	#[document_parameters("The `ArcRunExplicit` indexed scoped boundary.")]
	impl<'a, R, S, SBrand, Idx, Action, Final, K, Operation> IntoScopedBoundaryParts<'a>
		for ArcRunExplicitBoundary<'a, R, S, SBrand, Idx, Action, Final, K, Operation>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		SBrand: 'a,
		Idx: 'a,
		Action: Clone + Send + Sync + 'a,
		Operation: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Operation) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
	{
		type Carrier =
			ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>;
		type ConsumedBrand = SBrand;
		type ConsumedIdx = Idx;
		type ScopedLayer = Apply!(
			<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, Action>>
		);

		/// Split this boundary for the private carrier-aware dispatcher route.
		#[document_signature]
		#[document_returns(
			"The scoped row layer and wrapper-owned continuation carrier stored by the boundary."
		)]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// let layer = "selected action";
		/// let continuation = "outer continuation";
		/// let parts = (layer, continuation);
		/// assert_eq!(parts.0, "selected action");
		/// assert_eq!(parts.1, "outer continuation");
		/// ```
		#[inline]
		fn into_scoped_boundary_parts(
			self
		) -> (
			Self::ScopedLayer,
			ScopedContinuation<
				ArcRunExplicitActionSuppliedScopedContinuation<
					'a,
					R,
					S,
					Action,
					Final,
					K,
					Operation,
				>,
			>,
		) {
			self.into_parts()
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type."
	)]
	impl<'a, R, S, Action, Final, K> ScopedResumeTypes<'a>
		for ArcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Action: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Action) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
	{
		type ActionProgram = ArcRunExplicit<'a, R, S, Action>;
		type ActionValue = Action;
		type OperationProgram = ArcRunExplicit<'a, R, S, Action>;
		type OperationValue = Action;
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The generated action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The operation result type passed from the scoped operation to the outer continuation."
	)]
	impl<'a, R, S, Action, Final, K, Operation> ScopedResumeTypes<'a>
		for ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Action: Clone + Send + Sync + 'a,
		Operation: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Operation) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
	{
		type ActionProgram = ArcRunExplicit<'a, R, S, Action>;
		type ActionValue = Action;
		type OperationProgram = ArcRunExplicit<'a, R, S, Operation>;
		type OperationValue = Operation;
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The ArcRunExplicit scoped-continuation carrier.")]
	impl<'a, R, S, Action, Final, K, FirstLayer>
		ArcScopedResume<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>
		for ArcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Action: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Action) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
		FirstLayer: 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone + Send + Sync,
	{
		/// Resume the selected action by reattaching its outer continuation.
		#[document_signature]
		///
		#[document_parameters("The first-order handler list retained by the carrier contract.")]
		#[document_returns("The resumed `ArcRunExplicit` program.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// let run: ArcRunExplicit<'_, CNilBrand, CNilBrand, i32> = ArcRunExplicit::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_arc(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
		) -> ArcRunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();
			self.action.bind(move |action_value: Action| -> ArcRunExplicit<'a, R, S, Final> {
				outer(action_value)
			})
		}

		/// Insert a result-preserving action program before reattaching
		/// the selected action's outer continuation.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The result-preserving action program to apply before the outer continuation."
		)]
		#[document_returns("The resumed `ArcRunExplicit` program.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// let run: ArcRunExplicit<'_, CNilBrand, CNilBrand, i32> = ArcRunExplicit::pure(41);
		/// let incremented = run.bind(|value| ArcRunExplicit::pure(value + 1));
		/// assert_eq!(incremented.extract(), 42);
		/// ```
		fn resume_arc_with_post_action(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
			post_action: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionValue,
			) -> <Self as ScopedResumeTypes<'a>>::OperationProgram
			+ Send
			+ Sync
			+ 'a,
		) -> ArcRunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();

			self.action.bind(move |action_value: Action| -> ArcRunExplicit<'a, R, S, Final> {
				let outer = outer.clone();
				let post_program: ArcRunExplicit<'a, R, S, Action> = post_action(action_value);
				post_program.bind(move |post_value: Action| -> ArcRunExplicit<'a, R, S, Final> {
					outer(post_value)
				})
			})
		}

		/// Transform the selected action before reattaching its outer
		/// continuation.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The selected action transform to apply before outer continuation resume."
		)]
		#[document_returns("The resumed `ArcRunExplicit` program.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// let run: ArcRunExplicit<'_, CNilBrand, CNilBrand, i32> = ArcRunExplicit::pure(41);
		/// let incremented = run.bind(|value| ArcRunExplicit::pure(value + 1));
		/// assert_eq!(incremented.extract(), 42);
		/// ```
		fn resume_arc_with_action_transform(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
			transform: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionProgram,
			) -> <Self as ScopedResumeTypes<'a>>::ActionProgram
			+ Send
			+ Sync
			+ 'a,
		) -> ArcRunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();

			transform(self.action).bind(
				move |action_value: Action| -> ArcRunExplicit<'a, R, S, Final> {
					outer(action_value)
				},
			)
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The generated action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The operation result type passed from the scoped operation to the outer continuation.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The ArcRunExplicit action-supplied scoped-continuation carrier.")]
	impl<'a, R, S, Action, Final, K, Operation, FirstLayer>
		ArcActionSuppliedScopedResume<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>
		for ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Action: Clone + Send + Sync + 'a,
		Operation: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(Operation) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
		FirstLayer: 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Operation>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone + Send + Sync,
	{
		/// Resume a supplied action before reattaching its outer continuation.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The thread-safe factory that supplies the selected action program."
		)]
		#[document_returns("The resumed `ArcRunExplicit` program.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// let action: ArcRunExplicit<'_, CNilBrand, CNilBrand, i32> = ArcRunExplicit::pure(41);
		/// let resumed = action.bind(|value| ArcRunExplicit::pure(value + 1));
		/// assert_eq!(resumed.extract(), 42);
		/// ```
		fn resume_arc_with_supplied_action(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
			supplied_action: impl FnOnce() -> <Self as ScopedResumeTypes<'a>>::OperationProgram
			+ Send
			+ Sync
			+ 'a,
		) -> ArcRunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();

			supplied_action().bind(
				move |operation_value: Operation| -> ArcRunExplicit<'a, R, S, Final> {
					outer(operation_value)
				},
			)
		}
	}
}

pub use inner::*;
