#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		super::super::inner::RcRunExplicit,
		crate::{
			Apply,
			brands::{
				NodeBrand,
				RcBrand,
			},
			classes::{
				Functor,
				RefCountedPointer,
				WrapDrop,
			},
			kinds::*,
			types::{
				RcFreeExplicit,
				effects::{
					interpreter::{
						DispatchHandlers,
						DispatchResidualScopedHandlers,
						DispatchScopedBoundaryHandlers,
						IntoScopedBoundaryParts,
						RcActionSuppliedScopedResume,
						RcScopedResume,
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
	/// Rc-backed Explicit carrier for a selected scoped action.
	///
	/// The carrier keeps the action and the action's outer continuation
	/// separate while preserving the `RcFreeExplicit` multi-shot contract:
	/// cloning the carrier is O(1), and post-action work is a reusable `Fn`
	/// continuation over the selected action value.
	#[derive(Clone)]
	pub(crate) struct RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Final: 'a,
		K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a, {
		/// The selected scoped action before its outer continuation has
		/// been reattached.
		pub(crate) action: RcRunExplicit<'a, R, S, Action>,
		/// The action's outer continuation, still outside the selected action.
		pub(crate) outer: <RcBrand as RefCountedPointer>::Of<'a, K>,
		/// Carries the final result type without owning a value of that type.
		pub(crate) result: PhantomData<fn() -> Final>,
	}

	#[doc(hidden)]
	/// Rc-backed Explicit carrier for an action supplied by a dispatcher.
	///
	/// The carrier stores the shared outer continuation without storing the
	/// selected action itself. Indexed around-action boundaries provide that
	/// action from the scoped row; Bracket-style dispatchers provide it after
	/// acquire/body lifecycle work has determined the value that should flow
	/// into the outer continuation.
	#[allow(
		dead_code,
		reason = "Bracket carrier wiring consumes the Rc action-supplied carrier in the next implementation step; focused tests exercise the private shape until then."
	)]
	pub(crate) struct RcRunExplicitActionSuppliedScopedContinuation<
		'a,
		R,
		S,
		Action,
		Final,
		K,
		Operation = Action,
	>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Operation: Clone + 'a,
		Final: 'a,
		K: Fn(Operation) -> RcRunExplicit<'a, R, S, Final> + 'a, {
		/// The operation-result outer continuation, still outside the selected
		/// action.
		pub(crate) outer: <RcBrand as RefCountedPointer>::Of<'a, K>,
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
	#[document_parameters("The action-supplied RcRunExplicit scoped-continuation carrier.")]
	impl<'a, R, S, Action, Final, K, Operation> Clone
		for RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Operation: Clone + 'a,
		Final: 'a,
		K: Fn(Operation) -> RcRunExplicit<'a, R, S, Final> + 'a,
	{
		/// Clone the carrier by refcount-bumping the shared outer
		/// continuation.
		#[document_signature]
		#[document_returns("A carrier sharing the same outer continuation.")]
		#[document_examples]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// let outer = Rc::new(|value: i32| value + 1);
		/// let cloned = Rc::clone(&outer);
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

	/// Production indexed boundary for `RcRunExplicit` around-action scoped
	/// operations.
	///
	/// The boundary keeps the selected action in the scoped row projection
	/// and stores the shared outer continuation separately. Boundary `map`
	/// and `bind` compose only that outer continuation, preserving the
	/// multi-shot `RcRunExplicit` selected-action slot until a scoped
	/// dispatcher resumes it.
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
	pub struct RcRunExplicitBoundary<'a, R, S, SBrand, Idx, Action, Final, K, Operation = Action>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		SBrand: 'static,
		Idx: 'a,
		Action: Clone + 'a,
		Operation: Clone + 'a,
		Final: 'a,
		K: Fn(Operation) -> RcRunExplicit<'a, R, S, Final> + 'a, {
		/// The scoped row layer carrying the selected action program.
		layer: Apply!(
			<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, Action>>
		),
		/// The wrapper-owned continuation from operation result to final result.
		continuation: ScopedContinuation<
			RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>,
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
	#[document_parameters("The `RcRunExplicit` indexed scoped boundary.")]
	impl<'a, R, S, SBrand, Idx, Action, Final, K, Operation>
		RcRunExplicitBoundary<'a, R, S, SBrand, Idx, Action, Final, K, Operation>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		SBrand: 'a,
		Idx: 'a,
		Action: Clone + 'a,
		Operation: Clone + 'a,
		Final: 'a,
		K: Fn(Operation) -> RcRunExplicit<'a, R, S, Final> + 'a,
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
		#[document_examples]
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
				<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, Action>>
			),
			outer: K,
		) -> Self {
			Self {
				layer,
				continuation: ScopedContinuation::new(
					RcRunExplicitActionSuppliedScopedContinuation {
						outer: <RcBrand as RefCountedPointer>::new(outer),
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
		#[document_examples]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// let outer = Rc::new(|value: i32| value + 1);
		/// let f = Rc::new(|value: i32| value * 2);
		/// let composed = {
		/// 	let outer = Rc::clone(&outer);
		/// 	let f = Rc::clone(&f);
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
			f: impl Fn(Final) -> RcRunExplicit<'a, R, S, Next> + 'a,
		) -> RcRunExplicitBoundary<
			'a,
			R,
			S,
			SBrand,
			Idx,
			Action,
			Next,
			impl Fn(Operation) -> RcRunExplicit<'a, R, S, Next> + 'a,
			Operation,
		>
		where
			Final: Clone,
			Next: 'a,
			K: 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Next>,
			>): Clone, {
			let Self {
				layer,
				continuation,
				member: _,
			} = self;
			let carrier = continuation.into_inner();
			let outer = carrier.outer.clone();
			let f = <RcBrand as RefCountedPointer>::new(f);
			let composed = move |operation_value: Operation| {
				let f = f.clone();
				outer(operation_value).bind(move |final_value| f(final_value))
			};

			RcRunExplicitBoundary::new(layer, composed)
		}

		/// Map over the final result while leaving the selected action
		/// layer unchanged.
		#[document_signature]
		#[document_type_parameters("The mapped final result type.")]
		#[document_parameters("The function to apply after the outer continuation completes.")]
		#[document_returns("A boundary with the same action layer and mapped final continuation.")]
		#[document_examples]
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
			f: impl Fn(Final) -> Next + 'a,
		) -> RcRunExplicitBoundary<
			'a,
			R,
			S,
			SBrand,
			Idx,
			Action,
			Next,
			impl Fn(Operation) -> RcRunExplicit<'a, R, S, Next> + 'a,
			Operation,
		>
		where
			Final: Clone,
			Next: 'a,
			K: 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Next>,
			>): Clone, {
			let f = <RcBrand as RefCountedPointer>::new(f);

			self.bind(move |final_value| {
				let f = f.clone();
				RcRunExplicit::pure(f(final_value))
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
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
		/// type Prog = RcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = RcRunExplicit::pure(41);
		/// let boundary = RcRunExplicit::span::<&'static str, _>("request", action).map(|value| value + 1);
		///
		/// let result = boundary.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SpanBrand<RcBrand, &'static str>: span_handler(),
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
					RcRunExplicit<'a, R, S, Final>,
				>),
				RcRunExplicit<'a, R, S, Final>,
			>,
			scoped_handlers: impl DispatchScopedBoundaryHandlers<
				'a,
				Self,
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcRunExplicit<'a, R, S, Final>,
				>),
				RcRunExplicit<'a, R, S, Final>,
			> + DispatchResidualScopedHandlers<
				'a,
				SBrand,
				Idx,
				Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcRunExplicit<'a, R, S, Final>,
				>),
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcRunExplicit<'a, R, S, Final>,
				>),
				RcRunExplicit<'a, R, S, Final>,
			>,
		) -> Final
		where
			Final: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone, {
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

		/// Alias for [`handle`](RcRunExplicitBoundary::handle).
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
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
		/// type Prog = RcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = RcRunExplicit::pure(41);
		/// let boundary = RcRunExplicit::span::<&'static str, _>("request", action).map(|value| value + 1);
		///
		/// let result = boundary.run(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SpanBrand<RcBrand, &'static str>: span_handler(),
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
					RcRunExplicit<'a, R, S, Final>,
				>),
				RcRunExplicit<'a, R, S, Final>,
			>,
			scoped_handlers: impl DispatchScopedBoundaryHandlers<
				'a,
				Self,
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcRunExplicit<'a, R, S, Final>,
				>),
				RcRunExplicit<'a, R, S, Final>,
			> + DispatchResidualScopedHandlers<
				'a,
				SBrand,
				Idx,
				Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcRunExplicit<'a, R, S, Final>,
				>),
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcRunExplicit<'a, R, S, Final>,
				>),
				RcRunExplicit<'a, R, S, Final>,
			>,
		) -> Final
		where
			Final: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone, {
			self.handle(handlers, scoped_handlers)
		}

		/// Split the boundary into its action layer and scoped
		/// continuation carrier.
		#[document_signature]
		#[document_returns(
			"The scoped row layer and wrapper-owned continuation carrier stored by the boundary."
		)]
		#[document_examples]
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
				<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, Action>>
			),
			ScopedContinuation<
				RcRunExplicitActionSuppliedScopedContinuation<
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
	#[document_parameters("The `RcRunExplicit` indexed scoped boundary.")]
	impl<'a, R, S, SBrand, Idx, Action, Final, K, Operation> IntoScopedBoundaryParts<'a>
		for RcRunExplicitBoundary<'a, R, S, SBrand, Idx, Action, Final, K, Operation>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		SBrand: 'a,
		Idx: 'a,
		Action: Clone + 'a,
		Operation: Clone + 'a,
		Final: 'a,
		K: Fn(Operation) -> RcRunExplicit<'a, R, S, Final> + 'a,
	{
		type Carrier =
			RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>;
		type ConsumedBrand = SBrand;
		type ConsumedIdx = Idx;
		type ScopedLayer = Apply!(
			<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, Action>>
		);

		/// Split this boundary for the private carrier-aware dispatcher route.
		#[document_signature]
		#[document_returns(
			"The scoped row layer and wrapper-owned continuation carrier stored by the boundary."
		)]
		#[document_examples]
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
				RcRunExplicitActionSuppliedScopedContinuation<
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
		for RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Final: 'a,
		K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a,
	{
		type ActionProgram = RcRunExplicit<'a, R, S, Action>;
		type ActionValue = Action;
		type OperationProgram = RcRunExplicit<'a, R, S, Action>;
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
		for RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Operation: Clone + 'a,
		Final: 'a,
		K: Fn(Operation) -> RcRunExplicit<'a, R, S, Final> + 'a,
	{
		type ActionProgram = RcRunExplicit<'a, R, S, Action>;
		type ActionValue = Action;
		type OperationProgram = RcRunExplicit<'a, R, S, Operation>;
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
	#[document_parameters("The RcRunExplicit scoped-continuation carrier.")]
	impl<'a, R, S, Action, Final, K, FirstLayer>
		RcScopedResume<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>
		for RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Final: 'a,
		K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a,
		FirstLayer: 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone,
	{
		/// Resume the selected action by reattaching its outer continuation.
		#[document_signature]
		///
		#[document_parameters("The first-order handler list retained by the carrier contract.")]
		#[document_returns("The resumed `RcRunExplicit` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// let run: RcRunExplicit<'_, CNilBrand, CNilBrand, i32> = RcRunExplicit::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_rc(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
		) -> RcRunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();
			self.action.bind(move |action_value: Action| -> RcRunExplicit<'a, R, S, Final> {
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
		#[document_returns("The resumed `RcRunExplicit` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// let run: RcRunExplicit<'_, CNilBrand, CNilBrand, i32> = RcRunExplicit::pure(41);
		/// let incremented = run.bind(|value| RcRunExplicit::pure(value + 1));
		/// assert_eq!(incremented.extract(), 42);
		/// ```
		fn resume_rc_with_post_action(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
			post_action: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionValue,
			) -> <Self as ScopedResumeTypes<'a>>::OperationProgram
			+ 'a,
		) -> RcRunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();

			self.action.bind(move |action_value: Action| -> RcRunExplicit<'a, R, S, Final> {
				let outer = outer.clone();
				let post_program: RcRunExplicit<'a, R, S, Action> = post_action(action_value);
				post_program.bind(move |post_value: Action| -> RcRunExplicit<'a, R, S, Final> {
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
		#[document_returns("The resumed `RcRunExplicit` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// let run: RcRunExplicit<'_, CNilBrand, CNilBrand, i32> = RcRunExplicit::pure(41);
		/// let incremented = run.bind(|value| RcRunExplicit::pure(value + 1));
		/// assert_eq!(incremented.extract(), 42);
		/// ```
		fn resume_rc_with_action_transform(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
			transform: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionProgram,
			) -> <Self as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> RcRunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();

			transform(self.action).bind(
				move |action_value: Action| -> RcRunExplicit<'a, R, S, Final> {
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
	#[document_parameters("The RcRunExplicit action-supplied scoped-continuation carrier.")]
	impl<'a, R, S, Action, Final, K, Operation, FirstLayer>
		RcActionSuppliedScopedResume<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>
		for RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Operation: Clone + 'a,
		Final: 'a,
		K: Fn(Operation) -> RcRunExplicit<'a, R, S, Final> + 'a,
		FirstLayer: 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Operation>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone,
	{
		/// Resume a supplied action before reattaching its outer continuation.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The factory that supplies the selected action program."
		)]
		#[document_returns("The resumed `RcRunExplicit` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// let action: RcRunExplicit<'_, CNilBrand, CNilBrand, i32> = RcRunExplicit::pure(41);
		/// let resumed = action.bind(|value| RcRunExplicit::pure(value + 1));
		/// assert_eq!(resumed.extract(), 42);
		/// ```
		fn resume_rc_with_supplied_action(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
			supplied_action: impl FnOnce() -> <Self as ScopedResumeTypes<'a>>::OperationProgram + 'a,
		) -> RcRunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();

			supplied_action().bind(
				move |operation_value: Operation| -> RcRunExplicit<'a, R, S, Final> {
					outer(operation_value)
				},
			)
		}
	}
}

pub use inner::*;
