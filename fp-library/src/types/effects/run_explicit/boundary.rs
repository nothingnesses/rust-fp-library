#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		super::super::inner::RunExplicit,
		crate::{
			Apply,
			brands::RcBrand,
			classes::{
				Functor,
				RefCountedPointer,
				WrapDrop,
			},
			kinds::*,
			types::effects::{
				interpreter::{
					DispatchHandlers,
					DispatchResidualScopedHandlers,
					DispatchScopedBoundaryHandlers,
					ExplicitActionSuppliedScopedResume,
					ExplicitScopedResume,
					IntoScopedBoundaryParts,
					ScopedContinuation,
					ScopedResumeTypes,
				},
				node::Node,
			},
		},
		core::marker::PhantomData,
		fp_macros::*,
	};

	#[doc(hidden)]
	/// Explicit-substrate carrier for a selected scoped action.
	///
	/// The carrier keeps the action and the action's outer continuation
	/// as distinct values. Around-action scoped handlers can therefore
	/// insert a result-preserving action program between them without
	/// changing the action result type or erasing the intermediate value.
	#[allow(
		dead_code,
		reason = "Carrier-aware scoped dispatch wiring constructs the Explicit carrier later; focused tests exercise it directly, so expect(dead_code) is target-dependent across lib and test builds."
	)]
	pub(crate) struct RunExplicitScopedContinuation<'a, R, S, Action, Final, K>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: 'a,
		Final: 'a,
		K: Fn(Action) -> RunExplicit<'a, R, S, Final> + 'a, {
		/// The selected scoped action before its outer continuation has
		/// been reattached.
		pub(crate) action: RunExplicit<'a, R, S, Action>,
		/// The action's outer continuation, still outside the selected action.
		pub(crate) outer: <RcBrand as RefCountedPointer>::Of<'a, K>,
		/// Carries the final result type without owning a value of that type.
		pub(crate) result: PhantomData<fn() -> Final>,
	}

	#[doc(hidden)]
	/// Explicit-substrate carrier for an action supplied by a dispatcher.
	///
	/// Indexed around-action boundaries keep the selected action in the
	/// scoped row projection. Bracket-style dispatchers build the selected
	/// action after resource acquisition. Both cases use this carrier for the
	/// typed outer continuation; the dispatcher supplies the selected action
	/// program when it resumes the carrier.
	#[allow(
		dead_code,
		reason = "Bracket carrier wiring consumes the Explicit action-supplied carrier in the next implementation step; focused tests exercise the private shape until then."
	)]
	pub(crate) struct RunExplicitActionSuppliedScopedContinuation<
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
		Action: 'a,
		Operation: 'a,
		Final: 'a,
		K: Fn(Operation) -> RunExplicit<'a, R, S, Final> + 'a, {
		/// The operation-result outer continuation, still outside the selected
		/// action.
		pub(crate) outer: <RcBrand as RefCountedPointer>::Of<'a, K>,
		/// Carries the selected action, operation result, and final result types
		/// without owning values of those types.
		pub(crate) result: PhantomData<fn(Action) -> (Operation, Final)>,
	}

	/// Production indexed boundary for `RunExplicit` around-action scoped
	/// operations.
	///
	/// The boundary keeps the selected action in the scoped row projection and
	/// stores the typed outer continuation separately. Boundary `map` and
	/// `bind` compose only that outer continuation, so the scoped layer remains
	/// typed by the selected action result rather than the final mapped result.
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
	pub struct RunExplicitBoundary<'a, R, S, SBrand, Idx, Action, Final, K, Operation = Action>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		SBrand: 'static,
		Idx: 'a,
		Action: 'a,
		Operation: 'a,
		Final: 'a,
		K: Fn(Operation) -> RunExplicit<'a, R, S, Final> + 'a, {
		/// The scoped row layer carrying the selected action program.
		layer: Apply!(
			<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, Action>>
		),
		/// The wrapper-owned continuation from operation result to final result.
		continuation: ScopedContinuation<
			RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>,
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
	#[document_parameters("The `RunExplicit` indexed scoped boundary.")]
	impl<'a, R, S, SBrand, Idx, Action, Final, K, Operation>
		RunExplicitBoundary<'a, R, S, SBrand, Idx, Action, Final, K, Operation>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		SBrand: 'a,
		Idx: 'a,
		Action: 'a,
		Operation: 'a,
		Final: 'a,
		K: Fn(Operation) -> RunExplicit<'a, R, S, Final> + 'a,
	{
		/// Construct an indexed boundary from a scoped layer and an
		/// outer continuation.
		#[document_signature]
		///
		#[document_parameters(
			"The scoped row layer carrying the selected action program.",
			"The outer continuation from selected action result to final program."
		)]
		///
		#[document_returns(
			"A boundary that stores the action layer and outer continuation separately."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// struct Boundary<Layer, Outer> {
		/// 	layer: Layer,
		/// 	outer: Outer,
		/// }
		///
		/// impl<Layer, Outer> Boundary<Layer, Outer> {
		/// 	fn new(
		/// 		layer: Layer,
		/// 		outer: Outer,
		/// 	) -> Self {
		/// 		Self {
		/// 			layer,
		/// 			outer,
		/// 		}
		/// 	}
		/// }
		///
		/// let boundary = Boundary::new("selected action", |value: i32| value + 1);
		/// assert_eq!(boundary.layer, "selected action");
		/// assert_eq!((boundary.outer)(41), 42);
		/// ```
		pub(crate) fn new(
			layer: Apply!(
				<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, Action>>
			),
			outer: K,
		) -> Self {
			Self {
				layer,
				continuation: ScopedContinuation::new(
					RunExplicitActionSuppliedScopedContinuation {
						outer: <RcBrand as RefCountedPointer>::new(outer),
						result: PhantomData,
					},
				),
				member: PhantomData,
			}
		}

		/// Compose a final-result continuation onto this boundary.
		#[document_signature]
		///
		#[document_type_parameters("The result type produced after the additional continuation.")]
		///
		#[document_parameters(
			"The continuation to run after the existing outer continuation completes."
		)]
		///
		#[document_returns(
			"A boundary with the same action layer and a composed outer continuation."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// struct Boundary<Layer, Outer> {
		/// 	layer: Layer,
		/// 	outer: Rc<Outer>,
		/// }
		///
		/// impl<Layer, Outer> Boundary<Layer, Outer>
		/// where
		/// 	Outer: Fn(i32) -> i32 + 'static,
		/// {
		/// 	fn bind<Next>(
		/// 		self,
		/// 		f: impl Fn(i32) -> Next + 'static,
		/// 	) -> Boundary<Layer, impl Fn(i32) -> Next> {
		/// 		let outer = self.outer.clone();
		/// 		Boundary {
		/// 			layer: self.layer,
		/// 			outer: Rc::new(move |value| f(outer(value))),
		/// 		}
		/// 	}
		/// }
		///
		/// let boundary = Boundary {
		/// 	layer: "selected action",
		/// 	outer: Rc::new(|value| value + 1),
		/// }
		/// .bind(|value| value * 2);
		/// assert_eq!(boundary.layer, "selected action");
		/// assert_eq!((boundary.outer)(20), 42);
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "The boundary return type exposes the private action/operation/final carrier shape that scoped handlers consume."
		)]
		pub fn bind<Next>(
			self,
			f: impl Fn(Final) -> RunExplicit<'a, R, S, Next> + 'a,
		) -> RunExplicitBoundary<
			'a,
			R,
			S,
			SBrand,
			Idx,
			Action,
			Next,
			impl Fn(Operation) -> RunExplicit<'a, R, S, Next> + 'a,
			Operation,
		>
		where
			Next: 'a,
			K: 'a, {
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

			RunExplicitBoundary::new(layer, composed)
		}

		/// Map over the final result while leaving the selected action
		/// layer unchanged.
		#[document_signature]
		///
		#[document_type_parameters("The mapped final result type.")]
		///
		#[document_parameters("The function to apply after the outer continuation completes.")]
		///
		#[document_returns("A boundary with the same action layer and mapped final continuation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// struct Boundary<Layer, Outer> {
		/// 	layer: Layer,
		/// 	outer: Rc<Outer>,
		/// }
		///
		/// impl<Layer, Outer> Boundary<Layer, Outer>
		/// where
		/// 	Outer: Fn(i32) -> i32 + 'static,
		/// {
		/// 	fn map<Next>(
		/// 		self,
		/// 		f: impl Fn(i32) -> Next + 'static,
		/// 	) -> Boundary<Layer, impl Fn(i32) -> Next> {
		/// 		let outer = self.outer.clone();
		/// 		Boundary {
		/// 			layer: self.layer,
		/// 			outer: Rc::new(move |value| f(outer(value))),
		/// 		}
		/// 	}
		/// }
		///
		/// let boundary = Boundary {
		/// 	layer: "selected action",
		/// 	outer: Rc::new(|value| value + 1),
		/// }
		/// .map(|value| value * 2);
		/// assert_eq!(boundary.layer, "selected action");
		/// assert_eq!((boundary.outer)(20), 42);
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "The boundary return type preserves the private action/operation/final carrier shape across final-result mapping."
		)]
		pub fn map<Next>(
			self,
			f: impl Fn(Final) -> Next + 'a,
		) -> RunExplicitBoundary<
			'a,
			R,
			S,
			SBrand,
			Idx,
			Action,
			Next,
			impl Fn(Operation) -> RunExplicit<'a, R, S, Next> + 'a,
			Operation,
		>
		where
			Next: 'a,
			K: 'a, {
			let f = <RcBrand as RefCountedPointer>::new(f);

			self.bind(move |final_value| {
				let f = f.clone();
				RunExplicit::pure(f(final_value))
			})
		}

		/// Interpret this boundary through a scoped-handler list facade.
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
		/// 		run_explicit::RunExplicit,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
		/// type Prog = RunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = RunExplicit::pure(41);
		/// let boundary = RunExplicit::span::<&'static str, _>("request", action).map(|value| value + 1);
		///
		/// let result = boundary.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, &'static str>: span_handler(),
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
					RunExplicit<'a, R, S, Final>,
				>),
				RunExplicit<'a, R, S, Final>,
			>,
			scoped_handlers: impl DispatchScopedBoundaryHandlers<
				'a,
				Self,
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RunExplicit<'a, R, S, Final>,
				>),
				RunExplicit<'a, R, S, Final>,
			> + DispatchResidualScopedHandlers<
				'a,
				SBrand,
				Idx,
				Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RunExplicit<'a, R, S, Final>,
				>),
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RunExplicit<'a, R, S, Final>,
				>),
				RunExplicit<'a, R, S, Final>,
			>,
		) -> Final {
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

		/// Alias for [`handle`](RunExplicitBoundary::handle).
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
		/// 		run_explicit::RunExplicit,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
		/// type Prog = RunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = RunExplicit::pure(41);
		/// let boundary = RunExplicit::span::<&'static str, _>("request", action).map(|value| value + 1);
		///
		/// let result = boundary.run(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, &'static str>: span_handler(),
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
					RunExplicit<'a, R, S, Final>,
				>),
				RunExplicit<'a, R, S, Final>,
			>,
			scoped_handlers: impl DispatchScopedBoundaryHandlers<
				'a,
				Self,
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RunExplicit<'a, R, S, Final>,
				>),
				RunExplicit<'a, R, S, Final>,
			> + DispatchResidualScopedHandlers<
				'a,
				SBrand,
				Idx,
				Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RunExplicit<'a, R, S, Final>,
				>),
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RunExplicit<'a, R, S, Final>,
				>),
				RunExplicit<'a, R, S, Final>,
			>,
		) -> Final {
			self.handle(handlers, scoped_handlers)
		}

		/// Split the boundary into its action layer and scoped
		/// continuation carrier.
		#[document_signature]
		///
		#[document_returns(
			"The scoped row layer and wrapper-owned continuation carrier stored by the boundary."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// struct Boundary<Layer, Continuation> {
		/// 	layer: Layer,
		/// 	continuation: Continuation,
		/// }
		///
		/// impl<Layer, Continuation> Boundary<Layer, Continuation> {
		/// 	fn into_parts(self) -> (Layer, Continuation) {
		/// 		(self.layer, self.continuation)
		/// 	}
		/// }
		///
		/// let (layer, continuation) = Boundary {
		/// 	layer: "selected action",
		/// 	continuation: "outer continuation",
		/// }
		/// .into_parts();
		/// assert_eq!(layer, "selected action");
		/// assert_eq!(continuation, "outer continuation");
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "The split returns the explicit S::Of projection and continuation carrier that downstream scoped dispatch consumes."
		)]
		pub(crate) fn into_parts(
			self
		) -> (
			Apply!(
				<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, Action>>
			),
			ScopedContinuation<
				RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>,
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
	#[document_parameters("The `RunExplicit` indexed scoped boundary.")]
	impl<'a, R, S, SBrand, Idx, Action, Final, K, Operation> IntoScopedBoundaryParts<'a>
		for RunExplicitBoundary<'a, R, S, SBrand, Idx, Action, Final, K, Operation>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		SBrand: 'a,
		Idx: 'a,
		Action: 'a,
		Operation: 'a,
		Final: 'a,
		K: Fn(Operation) -> RunExplicit<'a, R, S, Final> + 'a,
	{
		type Carrier =
			RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>;
		type ConsumedBrand = SBrand;
		type ConsumedIdx = Idx;
		type ScopedLayer = Apply!(
			<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, Action>>
		);

		/// Split this boundary for the private carrier-aware dispatcher route.
		#[document_signature]
		#[document_returns(
			"The scoped row layer and wrapper-owned continuation carrier stored by the boundary."
		)]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// struct Boundary<Layer, Continuation> {
		/// 	layer: Layer,
		/// 	continuation: Continuation,
		/// }
		///
		/// impl<Layer, Continuation> Boundary<Layer, Continuation> {
		/// 	fn into_parts(self) -> (Layer, Continuation) {
		/// 		(self.layer, self.continuation)
		/// 	}
		/// }
		///
		/// let (layer, continuation) = Boundary {
		/// 	layer: "selected action",
		/// 	continuation: "outer continuation",
		/// }
		/// .into_parts();
		/// assert_eq!(layer, "selected action");
		/// assert_eq!(continuation, "outer continuation");
		/// ```
		#[inline]
		fn into_scoped_boundary_parts(
			self
		) -> (
			Self::ScopedLayer,
			ScopedContinuation<
				RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>,
			>,
		) {
			self.into_parts()
		}
	}

	#[doc(hidden)]
	/// Private Span layer shape for Explicit carrier-backed dispatch.
	///
	/// The ordinary `BoxSpan` layer stores a tag and an action thunk.
	/// The Explicit carrier-backed path instead needs the scoped layer
	/// to carry the tag together with the wrapper-owned continuation
	/// carrier that already owns the selected action and typed outer
	/// continuation. Keeping this shape private preserves the public
	/// `Span` operation while giving the next dispatcher step a concrete
	/// carrier cell to consume.
	#[document_type_parameters(
		"The lifetime that bounds the Span carrier cell.",
		"The Span tag type.",
		"The concrete wrapper-owned scoped-continuation carrier."
	)]
	#[derive(Clone)]
	pub(crate) struct RunExplicitSpanCarrierLayer<'a, Tag, Carrier>
	where
		Tag: 'a,
		Carrier: ScopedResumeTypes<'a>, {
		/// The instrumentation tag stored by value.
		pub(crate) tag: Tag,
		/// The wrapper-owned carrier that owns the selected action and
		/// outer continuation.
		pub(crate) continuation: ScopedContinuation<Carrier>,
		/// Carries the layer lifetime independently from the concrete
		/// carrier type.
		pub(crate) lifetime: PhantomData<&'a ()>,
	}

	#[document_type_parameters(
		"The lifetime that bounds the Span carrier cell.",
		"The Span tag type.",
		"The concrete wrapper-owned scoped-continuation carrier."
	)]
	#[document_parameters("The Explicit Span carrier layer.")]
	#[cfg_attr(
		not(test),
		expect(
			dead_code,
			reason = "The Explicit Span carrier layer helpers are introduced before the full wrapper interpreter route consumes them in step 7.4.4c; focused tests and the private dispatcher proof exercise the shape until then."
		)
	)]
	impl<'a, Tag, Carrier> RunExplicitSpanCarrierLayer<'a, Tag, Carrier>
	where
		Tag: 'a,
		Carrier: ScopedResumeTypes<'a>,
	{
		/// Construct a private Explicit Span carrier layer.
		#[document_signature]
		///
		#[document_parameters(
			"The instrumentation tag stored by value.",
			"The wrapper-owned continuation carrier for the selected Span action."
		)]
		///
		#[document_returns("A private Explicit Span carrier layer.")]
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
		/// 	fn new(
		/// 		tag: Tag,
		/// 		carrier: Carrier,
		/// 	) -> Self {
		/// 		Self {
		/// 			tag,
		/// 			carrier,
		/// 		}
		/// 	}
		/// }
		///
		/// let layer = LocalSpanLayer::new("request", 41);
		/// assert_eq!(layer.tag, "request");
		/// assert_eq!(layer.carrier, 41);
		/// ```
		pub(crate) const fn new(
			tag: Tag,
			continuation: ScopedContinuation<Carrier>,
		) -> Self {
			Self {
				tag,
				continuation,
				lifetime: PhantomData,
			}
		}

		/// Borrow the instrumentation tag.
		#[document_signature]
		///
		#[document_returns("A shared reference to the instrumentation tag.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// struct LocalSpanLayer<Tag> {
		/// 	tag: Tag,
		/// }
		///
		/// impl<Tag> LocalSpanLayer<Tag> {
		/// 	fn tag(&self) -> &Tag {
		/// 		&self.tag
		/// 	}
		/// }
		///
		/// let layer = LocalSpanLayer {
		/// 	tag: "request",
		/// };
		/// assert_eq!(layer.tag(), &"request");
		/// ```
		pub(crate) const fn tag(&self) -> &Tag {
			&self.tag
		}

		/// Split the layer into its tag and continuation carrier.
		#[document_signature]
		///
		#[document_returns("The instrumentation tag and wrapper-owned continuation carrier.")]
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
		/// 	fn into_parts(self) -> (Tag, Carrier) {
		/// 		(self.tag, self.carrier)
		/// 	}
		/// }
		///
		/// let (tag, carrier) = LocalSpanLayer {
		/// 	tag: "request",
		/// 	carrier: 42,
		/// }
		/// .into_parts();
		/// assert_eq!(tag, "request");
		/// assert_eq!(carrier, 42);
		/// ```
		pub(crate) fn into_parts(self) -> (Tag, ScopedContinuation<Carrier>) {
			(self.tag, self.continuation)
		}
	}

	#[doc(hidden)]
	/// Private Local layer shape for Explicit carrier-backed dispatch.
	///
	/// The ordinary `BoxLocal` layer stores an environment modifier and
	/// an action thunk. The Explicit carrier-backed path keeps the action
	/// inside a wrapper-owned continuation carrier instead, so the scoped
	/// layer only needs to carry the modifier plus that private carrier
	/// cell. This preserves the public Local operation while giving the
	/// dispatcher a concrete place to store metadata that must be applied
	/// before the selected action resumes.
	#[document_type_parameters(
		"The lifetime that bounds the Local carrier cell.",
		"The environment type transformed by the Local modifier.",
		"The concrete environment-modifier closure or closure cell.",
		"The concrete wrapper-owned scoped-continuation carrier."
	)]
	#[derive(Clone)]
	#[allow(
		dead_code,
		reason = "Carrier-aware Local dispatcher wiring consumes this private metadata layer in the next implementation step; focused tests exercise the shape until then."
	)]
	pub(crate) struct RunExplicitLocalCarrierLayer<'a, E, Modify, Carrier>
	where
		E: 'a,
		Modify: 'a,
		Carrier: ScopedResumeTypes<'a>, {
		/// The environment-transform closure or closure cell. Single-shot
		/// paths may store a `FnOnce` cell; shared paths may store cloneable
		/// `Fn` cells.
		pub(crate) modify: Modify,
		/// The wrapper-owned carrier that owns the selected action and
		/// outer continuation.
		pub(crate) continuation: ScopedContinuation<Carrier>,
		/// Carries the environment and layer lifetime independently from
		/// the concrete modifier type.
		pub(crate) environment: PhantomData<&'a E>,
	}

	#[document_type_parameters(
		"The lifetime that bounds the Local carrier cell.",
		"The environment type transformed by the Local modifier.",
		"The concrete environment-modifier closure or closure cell.",
		"The concrete wrapper-owned scoped-continuation carrier."
	)]
	#[document_parameters("The Explicit Local carrier layer.")]
	#[allow(
		dead_code,
		reason = "Carrier-aware Local dispatcher wiring consumes this private metadata layer in the next implementation step; focused tests exercise the shape until then."
	)]
	impl<'a, E, Modify, Carrier> RunExplicitLocalCarrierLayer<'a, E, Modify, Carrier>
	where
		E: 'a,
		Modify: 'a,
		Carrier: ScopedResumeTypes<'a>,
	{
		/// Construct a private Explicit Local carrier layer.
		#[document_signature]
		///
		#[document_parameters(
			"The environment modifier stored by the Local operation.",
			"The wrapper-owned continuation carrier for the selected Local action."
		)]
		///
		#[document_returns("A private Explicit Local carrier layer.")]
		///
		#[document_examples]
		///
		/// ```
		/// use core::marker::PhantomData;
		///
		/// struct LocalCarrierLayer<E, Modify, Carrier> {
		/// 	modify: Modify,
		/// 	carrier: Carrier,
		/// 	environment: PhantomData<E>,
		/// }
		///
		/// impl<E, Modify, Carrier> LocalCarrierLayer<E, Modify, Carrier> {
		/// 	fn new(
		/// 		modify: Modify,
		/// 		carrier: Carrier,
		/// 	) -> Self {
		/// 		Self {
		/// 			modify,
		/// 			carrier,
		/// 			environment: PhantomData,
		/// 		}
		/// 	}
		/// }
		///
		/// let layer = LocalCarrierLayer::<i32, _, _>::new(|env| env + 1, 41);
		/// assert_eq!((layer.modify)(4), 5);
		/// assert_eq!(layer.carrier, 41);
		/// ```
		pub(crate) const fn new(
			modify: Modify,
			continuation: ScopedContinuation<Carrier>,
		) -> Self {
			Self {
				modify,
				continuation,
				environment: PhantomData,
			}
		}

		/// Split the layer into its modifier and continuation carrier.
		#[document_signature]
		///
		#[document_returns(
			"The Local environment modifier and wrapper-owned continuation carrier."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use core::marker::PhantomData;
		///
		/// struct LocalCarrierLayer<E, Modify, Carrier> {
		/// 	modify: Modify,
		/// 	carrier: Carrier,
		/// 	environment: PhantomData<E>,
		/// }
		///
		/// impl<E, Modify, Carrier> LocalCarrierLayer<E, Modify, Carrier> {
		/// 	fn into_parts(self) -> (Modify, Carrier) {
		/// 		(self.modify, self.carrier)
		/// 	}
		/// }
		///
		/// let (modify, carrier) = LocalCarrierLayer::<i32, _, _> {
		/// 	modify: |env| env + 1,
		/// 	carrier: 41,
		/// 	environment: PhantomData,
		/// }
		/// .into_parts();
		/// assert_eq!(modify(4), 5);
		/// assert_eq!(carrier, 41);
		/// ```
		pub(crate) fn into_parts(self) -> (Modify, ScopedContinuation<Carrier>) {
			(self.modify, self.continuation)
		}
	}

	#[doc(hidden)]
	/// Private RefLocal layer shape for Explicit carrier-backed dispatch.
	///
	/// RefLocal has the same carrier split as Local, but its modifier
	/// borrows the inherited environment and returns the environment
	/// value used by the selected action. The layer stores that modifier
	/// together with the wrapper-owned continuation carrier so the
	/// dispatcher can apply the borrow-based environment transform before
	/// resuming the selected action.
	#[document_type_parameters(
		"The lifetime that bounds the RefLocal carrier cell.",
		"The environment type borrowed and reconstructed by the RefLocal modifier.",
		"The concrete borrow-based environment-modifier closure or closure cell.",
		"The concrete wrapper-owned scoped-continuation carrier."
	)]
	#[derive(Clone)]
	#[allow(
		dead_code,
		reason = "Carrier-aware RefLocal dispatcher wiring consumes this private metadata layer in the next implementation step; focused tests exercise the shape until then."
	)]
	pub(crate) struct RunExplicitRefLocalCarrierLayer<'a, E, Modify, Carrier>
	where
		E: 'a,
		Modify: 'a,
		Carrier: ScopedResumeTypes<'a>, {
		/// The borrow-based environment-transform closure or closure
		/// cell. Single-shot paths may store a `FnOnce` cell; shared
		/// paths may store cloneable `Fn` cells.
		pub(crate) modify: Modify,
		/// The wrapper-owned carrier that owns the selected action and
		/// outer continuation.
		pub(crate) continuation: ScopedContinuation<Carrier>,
		/// Carries the environment and layer lifetime independently from
		/// the concrete modifier type.
		pub(crate) environment: PhantomData<&'a E>,
	}

	#[document_type_parameters(
		"The lifetime that bounds the RefLocal carrier cell.",
		"The environment type borrowed and reconstructed by the RefLocal modifier.",
		"The concrete borrow-based environment-modifier closure or closure cell.",
		"The concrete wrapper-owned scoped-continuation carrier."
	)]
	#[document_parameters("The Explicit RefLocal carrier layer.")]
	#[allow(
		dead_code,
		reason = "Carrier-aware RefLocal dispatcher wiring consumes this private metadata layer in the next implementation step; focused tests exercise the shape until then."
	)]
	impl<'a, E, Modify, Carrier> RunExplicitRefLocalCarrierLayer<'a, E, Modify, Carrier>
	where
		E: 'a,
		Modify: 'a,
		Carrier: ScopedResumeTypes<'a>,
	{
		/// Construct a private Explicit RefLocal carrier layer.
		#[document_signature]
		///
		#[document_parameters(
			"The borrow-based environment modifier stored by the RefLocal operation.",
			"The wrapper-owned continuation carrier for the selected RefLocal action."
		)]
		///
		#[document_returns("A private Explicit RefLocal carrier layer.")]
		///
		#[document_examples]
		///
		/// ```
		/// use core::marker::PhantomData;
		///
		/// struct RefLocalCarrierLayer<E, Modify, Carrier> {
		/// 	modify: Modify,
		/// 	carrier: Carrier,
		/// 	environment: PhantomData<E>,
		/// }
		///
		/// impl<E, Modify, Carrier> RefLocalCarrierLayer<E, Modify, Carrier> {
		/// 	fn new(
		/// 		modify: Modify,
		/// 		carrier: Carrier,
		/// 	) -> Self {
		/// 		Self {
		/// 			modify,
		/// 			carrier,
		/// 			environment: PhantomData,
		/// 		}
		/// 	}
		/// }
		///
		/// let layer = RefLocalCarrierLayer::<i32, _, _>::new(|env: &i32| *env + 1, 41);
		/// assert_eq!((layer.modify)(&4), 5);
		/// assert_eq!(layer.carrier, 41);
		/// ```
		pub(crate) const fn new(
			modify: Modify,
			continuation: ScopedContinuation<Carrier>,
		) -> Self {
			Self {
				modify,
				continuation,
				environment: PhantomData,
			}
		}

		/// Split the layer into its modifier and continuation carrier.
		#[document_signature]
		///
		#[document_returns(
			"The RefLocal environment modifier and wrapper-owned continuation carrier."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use core::marker::PhantomData;
		///
		/// struct RefLocalCarrierLayer<E, Modify, Carrier> {
		/// 	modify: Modify,
		/// 	carrier: Carrier,
		/// 	environment: PhantomData<E>,
		/// }
		///
		/// impl<E, Modify, Carrier> RefLocalCarrierLayer<E, Modify, Carrier> {
		/// 	fn into_parts(self) -> (Modify, Carrier) {
		/// 		(self.modify, self.carrier)
		/// 	}
		/// }
		///
		/// let (modify, carrier) = RefLocalCarrierLayer::<i32, _, _> {
		/// 	modify: |env: &i32| *env + 1,
		/// 	carrier: 41,
		/// 	environment: PhantomData,
		/// }
		/// .into_parts();
		/// assert_eq!(modify(&4), 5);
		/// assert_eq!(carrier, 41);
		/// ```
		pub(crate) fn into_parts(self) -> (Modify, ScopedContinuation<Carrier>) {
			(self.modify, self.continuation)
		}
	}

	#[doc(hidden)]
	/// Private Catch layer shape for Explicit carrier-backed dispatch.
	///
	/// The ordinary `BoxCatch` layer stores an action thunk and a recovery
	/// handler. The carrier-backed path keeps the selected action inside
	/// the wrapper-owned continuation carrier, so the scoped layer stores
	/// only the recovery handler plus that carrier. The dispatcher can
	/// then protect the selected action, run recovery on thrown errors,
	/// and resume the outer continuation only when the action or recovery
	/// produces a value.
	#[document_type_parameters(
		"The lifetime that bounds the Catch carrier cell.",
		"The error type recovered from by the Catch handler.",
		"The concrete recovery-handler closure or closure cell.",
		"The concrete wrapper-owned scoped-continuation carrier."
	)]
	#[derive(Clone)]
	#[allow(
		dead_code,
		reason = "Carrier-aware Catch dispatcher wiring consumes this private metadata layer before the full wrapper interpreter route constructs it."
	)]
	pub(crate) struct RunExplicitCatchCarrierLayer<'a, E, Handler, Carrier>
	where
		E: 'a,
		Handler: 'a,
		Carrier: ScopedResumeTypes<'a>, {
		/// The recovery handler closure or closure cell. Single-shot
		/// paths may store a `FnOnce` cell; shared paths may store
		/// cloneable `Fn` cells.
		pub(crate) handler: Handler,
		/// The wrapper-owned carrier that owns the selected action and
		/// outer continuation.
		pub(crate) continuation: ScopedContinuation<Carrier>,
		/// Carries the error and layer lifetime independently from the
		/// concrete handler type.
		pub(crate) error: PhantomData<&'a E>,
	}

	#[document_type_parameters(
		"The lifetime that bounds the Catch carrier cell.",
		"The error type recovered from by the Catch handler.",
		"The concrete recovery-handler closure or closure cell.",
		"The concrete wrapper-owned scoped-continuation carrier."
	)]
	#[document_parameters("The Explicit Catch carrier layer.")]
	#[allow(
		dead_code,
		reason = "Carrier-aware Catch dispatcher wiring consumes this private metadata layer before the full wrapper interpreter route constructs it."
	)]
	impl<'a, E, Handler, Carrier> RunExplicitCatchCarrierLayer<'a, E, Handler, Carrier>
	where
		E: 'a,
		Handler: 'a,
		Carrier: ScopedResumeTypes<'a>,
	{
		/// Construct a private Explicit Catch carrier layer.
		#[document_signature]
		///
		#[document_parameters(
			"The recovery handler stored by the Catch operation.",
			"The wrapper-owned continuation carrier for the selected Catch action."
		)]
		#[document_returns("A private Explicit Catch carrier layer.")]
		#[document_examples]
		///
		/// ```
		/// use core::marker::PhantomData;
		///
		/// struct CatchCarrierLayer<E, Handler, Carrier> {
		/// 	handler: Handler,
		/// 	carrier: Carrier,
		/// 	error: PhantomData<E>,
		/// }
		///
		/// impl<E, Handler, Carrier> CatchCarrierLayer<E, Handler, Carrier> {
		/// 	fn new(
		/// 		handler: Handler,
		/// 		carrier: Carrier,
		/// 	) -> Self {
		/// 		Self {
		/// 			handler,
		/// 			carrier,
		/// 			error: PhantomData,
		/// 		}
		/// 	}
		/// }
		///
		/// let layer = CatchCarrierLayer::<&'static str, _, _>::new(|err: &'static str| err.len(), 41);
		/// assert_eq!((layer.handler)("boom"), 4);
		/// assert_eq!(layer.carrier, 41);
		/// ```
		pub(crate) const fn new(
			handler: Handler,
			continuation: ScopedContinuation<Carrier>,
		) -> Self {
			Self {
				handler,
				continuation,
				error: PhantomData,
			}
		}

		/// Split the layer into its recovery handler and continuation carrier.
		#[document_signature]
		///
		#[document_returns("The Catch recovery handler and wrapper-owned continuation carrier.")]
		#[document_examples]
		///
		/// ```
		/// use core::marker::PhantomData;
		///
		/// struct CatchCarrierLayer<E, Handler, Carrier> {
		/// 	handler: Handler,
		/// 	carrier: Carrier,
		/// 	error: PhantomData<E>,
		/// }
		///
		/// impl<E, Handler, Carrier> CatchCarrierLayer<E, Handler, Carrier> {
		/// 	fn into_parts(self) -> (Handler, Carrier) {
		/// 		(self.handler, self.carrier)
		/// 	}
		/// }
		///
		/// let (handler, carrier) = CatchCarrierLayer::<&'static str, _, _> {
		/// 	handler: |err: &'static str| err.len(),
		/// 	carrier: 41,
		/// 	error: PhantomData,
		/// }
		/// .into_parts();
		/// assert_eq!(handler("boom"), 4);
		/// assert_eq!(carrier, 41);
		/// ```
		pub(crate) fn into_parts(self) -> (Handler, ScopedContinuation<Carrier>) {
			(self.handler, self.continuation)
		}
	}

	#[doc(hidden)]
	/// Private Bracket layer shape for Explicit carrier-backed dispatch.
	///
	/// The ordinary Bracket layer stores acquire, body, and release cells
	/// together inside the scoped operation. The carrier-backed path still
	/// stores those lifecycle cells by value, but keeps the selected body
	/// action out of the layer until acquire has produced the resource. The
	/// continuation carrier owns only the typed outer resume boundary.
	#[document_type_parameters(
		"The lifetime that bounds the Bracket carrier cell.",
		"The pointer brand used by the body and release resource cells.",
		"The resource type produced by acquire.",
		"The body result type returned after release.",
		"The concrete acquire program factory.",
		"The concrete body-action factory.",
		"The concrete release-action factory.",
		"The concrete wrapper-owned scoped-continuation carrier."
	)]
	#[derive(Clone)]
	#[allow(
		dead_code,
		reason = "Carrier-aware Bracket dispatcher wiring consumes this private metadata layer in the next implementation step; focused tests exercise the shape until then."
	)]
	pub(crate) struct RunExplicitBracketCarrierLayer<
		'a,
		P,
		Resource,
		BodyResult,
		Acquire,
		Body,
		Release,
		Carrier,
	>
	where
		P: 'a,
		Resource: 'a,
		BodyResult: 'a,
		Acquire: 'a,
		Body: 'a,
		Release: 'a,
		Carrier: ScopedResumeTypes<'a>, {
		/// The acquire lifecycle cell.
		pub(crate) acquire: Acquire,
		/// The body lifecycle cell. For Bracket, this consumes the
		/// pointer-wrapped resource and returns the resource together with
		/// the body result so release can receive the resource afterward.
		pub(crate) body: Body,
		/// The release lifecycle cell.
		pub(crate) release: Release,
		/// The wrapper-owned carrier that resumes the outer continuation
		/// after the generated body/release action completes.
		pub(crate) continuation: ScopedContinuation<Carrier>,
		/// Carries the pointer brand and lifecycle result types without
		/// owning values of those types.
		#[expect(
			clippy::type_complexity,
			reason = "The marker intentionally carries the layer lifetime, pointer brand, resource type, and body result type without adding runtime fields."
		)]
		pub(crate) lifecycle: PhantomData<(&'a (), fn(P, Resource) -> BodyResult)>,
	}

	#[document_type_parameters(
		"The lifetime that bounds the Bracket carrier cell.",
		"The pointer brand used by the body and release resource cells.",
		"The resource type produced by acquire.",
		"The body result type returned after release.",
		"The concrete acquire program factory.",
		"The concrete body-action factory.",
		"The concrete release-action factory.",
		"The concrete wrapper-owned scoped-continuation carrier."
	)]
	#[document_parameters("The Explicit Bracket carrier layer.")]
	#[allow(
		dead_code,
		reason = "Carrier-aware Bracket dispatcher wiring consumes this private metadata layer in the next implementation step; focused tests exercise the shape until then."
	)]
	impl<'a, P, Resource, BodyResult, Acquire, Body, Release, Carrier>
		RunExplicitBracketCarrierLayer<'a, P, Resource, BodyResult, Acquire, Body, Release, Carrier>
	where
		P: 'a,
		Resource: 'a,
		BodyResult: 'a,
		Acquire: 'a,
		Body: 'a,
		Release: 'a,
		Carrier: ScopedResumeTypes<'a>,
	{
		/// Construct a private Explicit Bracket carrier layer.
		#[document_signature]
		///
		#[document_parameters(
			"The acquire lifecycle cell.",
			"The body lifecycle cell.",
			"The release lifecycle cell.",
			"The wrapper-owned continuation carrier for the selected Bracket action."
		)]
		///
		#[document_returns("A private Explicit Bracket carrier layer.")]
		///
		#[document_examples]
		///
		/// ```
		/// struct BracketLayer<Acquire, Body, Release, Carrier> {
		/// 	acquire: Acquire,
		/// 	body: Body,
		/// 	release: Release,
		/// 	carrier: Carrier,
		/// }
		///
		/// impl<Acquire, Body, Release, Carrier> BracketLayer<Acquire, Body, Release, Carrier> {
		/// 	fn new(
		/// 		acquire: Acquire,
		/// 		body: Body,
		/// 		release: Release,
		/// 		carrier: Carrier,
		/// 	) -> Self {
		/// 		Self {
		/// 			acquire,
		/// 			body,
		/// 			release,
		/// 			carrier,
		/// 		}
		/// 	}
		/// }
		///
		/// let layer = BracketLayer::new(
		/// 	|| 7,
		/// 	|resource: Box<i32>| (*resource, *resource + 35),
		/// 	|resource: Box<i32>| *resource == 7,
		/// 	"outer",
		/// );
		/// let resource = (layer.acquire)();
		/// let (resource, body_result) = (layer.body)(Box::new(resource));
		/// assert_eq!(body_result, 42);
		/// assert!((layer.release)(Box::new(resource)));
		/// assert_eq!(layer.carrier, "outer");
		/// ```
		pub(crate) const fn new(
			acquire: Acquire,
			body: Body,
			release: Release,
			continuation: ScopedContinuation<Carrier>,
		) -> Self {
			Self {
				acquire,
				body,
				release,
				continuation,
				lifecycle: PhantomData,
			}
		}

		/// Split the layer into lifecycle cells and continuation carrier.
		#[document_signature]
		///
		#[document_returns(
			"The Bracket acquire, body, release cells and wrapper-owned continuation carrier."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// struct BracketLayer<Acquire, Body, Release, Carrier> {
		/// 	acquire: Acquire,
		/// 	body: Body,
		/// 	release: Release,
		/// 	carrier: Carrier,
		/// }
		///
		/// impl<Acquire, Body, Release, Carrier> BracketLayer<Acquire, Body, Release, Carrier> {
		/// 	fn into_parts(self) -> (Acquire, Body, Release, Carrier) {
		/// 		(self.acquire, self.body, self.release, self.carrier)
		/// 	}
		/// }
		///
		/// let (acquire, body, release, carrier) = BracketLayer {
		/// 	acquire: || 7,
		/// 	body: |resource: Box<i32>| (*resource, *resource + 35),
		/// 	release: |resource: Box<i32>| *resource == 7,
		/// 	carrier: "outer",
		/// }
		/// .into_parts();
		/// let resource = acquire();
		/// let (resource, body_result) = body(Box::new(resource));
		/// assert_eq!(body_result, 42);
		/// assert!(release(Box::new(resource)));
		/// assert_eq!(carrier, "outer");
		/// ```
		pub(crate) fn into_parts(self) -> (Acquire, Body, Release, ScopedContinuation<Carrier>) {
			(self.acquire, self.body, self.release, self.continuation)
		}
	}

	#[doc(hidden)]
	/// Private RefBracket layer shape for Explicit carrier-backed dispatch.
	///
	/// RefBracket differs from Bracket by keeping the acquired resource in a
	/// refcounted pointer and passing pointer clones to body and release.
	/// This metadata layer stores acquire, body, release, and the
	/// wrapper-owned action-supplied continuation while keeping the pointer
	/// brand explicit in the type.
	#[document_type_parameters(
		"The lifetime that bounds the RefBracket carrier cell.",
		"The refcounted pointer brand used for body and release resource clones.",
		"The resource type produced by acquire.",
		"The body result type returned after release.",
		"The concrete acquire program factory.",
		"The concrete body-action factory.",
		"The concrete release-action factory.",
		"The concrete wrapper-owned scoped-continuation carrier."
	)]
	#[derive(Clone)]
	#[allow(
		dead_code,
		reason = "Carrier-aware RefBracket dispatcher wiring consumes this private metadata layer in the next implementation step; focused tests exercise the shape until then."
	)]
	pub(crate) struct RunExplicitRefBracketCarrierLayer<
		'a,
		P,
		Resource,
		BodyResult,
		Acquire,
		Body,
		Release,
		Carrier,
	>
	where
		P: 'a,
		Resource: 'a,
		BodyResult: 'a,
		Acquire: 'a,
		Body: 'a,
		Release: 'a,
		Carrier: ScopedResumeTypes<'a>, {
		/// The acquire lifecycle cell.
		pub(crate) acquire: Acquire,
		/// The body lifecycle cell. For RefBracket, this receives a
		/// resource-pointer clone and returns the body result.
		pub(crate) body: Body,
		/// The release lifecycle cell. It receives a separate
		/// resource-pointer clone after the body action completes.
		pub(crate) release: Release,
		/// The wrapper-owned carrier that resumes the outer continuation
		/// after the generated body/release action completes.
		pub(crate) continuation: ScopedContinuation<Carrier>,
		/// Carries the pointer brand and lifecycle result types without
		/// owning values of those types.
		#[expect(
			clippy::type_complexity,
			reason = "The marker intentionally carries the layer lifetime, pointer brand, resource type, and body result type without adding runtime fields."
		)]
		pub(crate) lifecycle: PhantomData<(&'a (), fn(P, Resource) -> BodyResult)>,
	}

	#[document_type_parameters(
		"The lifetime that bounds the RefBracket carrier cell.",
		"The refcounted pointer brand used for body and release resource clones.",
		"The resource type produced by acquire.",
		"The body result type returned after release.",
		"The concrete acquire program factory.",
		"The concrete body-action factory.",
		"The concrete release-action factory.",
		"The concrete wrapper-owned scoped-continuation carrier."
	)]
	#[document_parameters("The Explicit RefBracket carrier layer.")]
	#[allow(
		dead_code,
		reason = "Carrier-aware RefBracket dispatcher wiring consumes this private metadata layer in the next implementation step; focused tests exercise the shape until then."
	)]
	impl<'a, P, Resource, BodyResult, Acquire, Body, Release, Carrier>
		RunExplicitRefBracketCarrierLayer<'a, P, Resource, BodyResult, Acquire, Body, Release, Carrier>
	where
		P: 'a,
		Resource: 'a,
		BodyResult: 'a,
		Acquire: 'a,
		Body: 'a,
		Release: 'a,
		Carrier: ScopedResumeTypes<'a>,
	{
		/// Construct a private Explicit RefBracket carrier layer.
		#[document_signature]
		///
		#[document_parameters(
			"The acquire lifecycle cell.",
			"The body lifecycle cell.",
			"The release lifecycle cell.",
			"The wrapper-owned continuation carrier for the selected RefBracket action."
		)]
		///
		#[document_returns("A private Explicit RefBracket carrier layer.")]
		///
		#[document_examples]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// struct RefBracketLayer<Acquire, Body, Release, Carrier> {
		/// 	acquire: Acquire,
		/// 	body: Body,
		/// 	release: Release,
		/// 	carrier: Carrier,
		/// }
		///
		/// impl<Acquire, Body, Release, Carrier> RefBracketLayer<Acquire, Body, Release, Carrier> {
		/// 	fn new(
		/// 		acquire: Acquire,
		/// 		body: Body,
		/// 		release: Release,
		/// 		carrier: Carrier,
		/// 	) -> Self {
		/// 		Self {
		/// 			acquire,
		/// 			body,
		/// 			release,
		/// 			carrier,
		/// 		}
		/// 	}
		/// }
		///
		/// let layer = RefBracketLayer::new(
		/// 	|| 7,
		/// 	|resource: Rc<i32>| *resource + 35,
		/// 	|resource: Rc<i32>| *resource == 7,
		/// 	"outer",
		/// );
		/// let resource = Rc::new((layer.acquire)());
		/// let release_resource = Rc::clone(&resource);
		/// assert_eq!((layer.body)(resource), 42);
		/// assert!((layer.release)(release_resource));
		/// assert_eq!(layer.carrier, "outer");
		/// ```
		pub(crate) const fn new(
			acquire: Acquire,
			body: Body,
			release: Release,
			continuation: ScopedContinuation<Carrier>,
		) -> Self {
			Self {
				acquire,
				body,
				release,
				continuation,
				lifecycle: PhantomData,
			}
		}

		/// Split the layer into lifecycle cells and continuation carrier.
		#[document_signature]
		///
		#[document_returns(
			"The RefBracket acquire, body, release cells and wrapper-owned continuation carrier."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// struct RefBracketLayer<Acquire, Body, Release, Carrier> {
		/// 	acquire: Acquire,
		/// 	body: Body,
		/// 	release: Release,
		/// 	carrier: Carrier,
		/// }
		///
		/// impl<Acquire, Body, Release, Carrier> RefBracketLayer<Acquire, Body, Release, Carrier> {
		/// 	fn into_parts(self) -> (Acquire, Body, Release, Carrier) {
		/// 		(self.acquire, self.body, self.release, self.carrier)
		/// 	}
		/// }
		///
		/// let (acquire, body, release, carrier) = RefBracketLayer {
		/// 	acquire: || 7,
		/// 	body: |resource: Rc<i32>| *resource + 35,
		/// 	release: |resource: Rc<i32>| *resource == 7,
		/// 	carrier: "outer",
		/// }
		/// .into_parts();
		/// let resource = Rc::new(acquire());
		/// let release_resource = Rc::clone(&resource);
		/// assert_eq!(body(resource), 42);
		/// assert!(release(release_resource));
		/// assert_eq!(carrier, "outer");
		/// ```
		pub(crate) fn into_parts(self) -> (Acquire, Body, Release, ScopedContinuation<Carrier>) {
			(self.acquire, self.body, self.release, self.continuation)
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
		for RunExplicitScopedContinuation<'a, R, S, Action, Final, K>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: 'a,
		Final: 'a,
		K: Fn(Action) -> RunExplicit<'a, R, S, Final> + 'a,
	{
		type ActionProgram = RunExplicit<'a, R, S, Action>;
		type ActionValue = Action;
		type OperationProgram = RunExplicit<'a, R, S, Action>;
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
		for RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: 'a,
		Operation: 'a,
		Final: 'a,
		K: Fn(Operation) -> RunExplicit<'a, R, S, Final> + 'a,
	{
		type ActionProgram = RunExplicit<'a, R, S, Action>;
		type ActionValue = Action;
		type OperationProgram = RunExplicit<'a, R, S, Operation>;
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
	#[document_parameters("The Explicit scoped-continuation carrier.")]
	impl<'a, R, S, Action, Final, K, FirstLayer>
		ExplicitScopedResume<'a, FirstLayer, RunExplicit<'a, R, S, Final>>
		for RunExplicitScopedContinuation<'a, R, S, Action, Final, K>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: 'a,
		Final: 'a,
		K: Fn(Action) -> RunExplicit<'a, R, S, Final> + 'a,
		FirstLayer: 'a,
	{
		/// Resume the selected action by reattaching its outer continuation.
		#[document_signature]
		///
		#[document_parameters("The first-order handler list retained by the carrier contract.")]
		#[document_returns("The resumed `RunExplicit` program.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// let run: RunExplicit<'_, CNilBrand, CNilBrand, i32> = RunExplicit::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_explicit(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
		) -> RunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();
			self.action.bind(move |action_value| outer(action_value))
		}

		/// Insert a result-preserving action program before reattaching
		/// the selected action's outer continuation.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The result-preserving action program to apply before the outer continuation."
		)]
		#[document_returns("The resumed `RunExplicit` program.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// let run: RunExplicit<'_, CNilBrand, CNilBrand, i32> = RunExplicit::pure(41);
		/// let incremented = run.bind(|value| RunExplicit::pure(value + 1));
		/// assert_eq!(incremented.extract(), 42);
		/// ```
		fn resume_explicit_with_post_action(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
			post_action: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionValue,
			) -> <Self as ScopedResumeTypes<'a>>::OperationProgram
			+ 'a,
		) -> RunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();

			self.action.bind(move |action_value| {
				let outer = outer.clone();
				post_action(action_value).bind(move |post_value| outer(post_value))
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
		#[document_returns("The resumed `RunExplicit` program.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// let run: RunExplicit<'_, CNilBrand, CNilBrand, i32> = RunExplicit::pure(41);
		/// let incremented = run.bind(|value| RunExplicit::pure(value + 1));
		/// assert_eq!(incremented.extract(), 42);
		/// ```
		fn resume_explicit_with_action_transform(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
			transform: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionProgram,
			) -> <Self as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> RunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();

			transform(self.action).bind(move |action_value| outer(action_value))
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
	#[document_parameters("The Explicit action-supplied scoped-continuation carrier.")]
	impl<'a, R, S, Action, Final, K, Operation, FirstLayer>
		ExplicitActionSuppliedScopedResume<'a, FirstLayer, RunExplicit<'a, R, S, Final>>
		for RunExplicitActionSuppliedScopedContinuation<'a, R, S, Action, Final, K, Operation>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: 'a,
		Operation: 'a,
		Final: 'a,
		K: Fn(Operation) -> RunExplicit<'a, R, S, Final> + 'a,
		FirstLayer: 'a,
	{
		/// Resume a supplied action before reattaching its outer continuation.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The factory that supplies the selected action program."
		)]
		#[document_returns("The resumed `RunExplicit` program.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// let action: RunExplicit<'_, CNilBrand, CNilBrand, i32> = RunExplicit::pure(41);
		/// let resumed = action.bind(|value| RunExplicit::pure(value + 1));
		/// assert_eq!(resumed.extract(), 42);
		/// ```
		fn resume_explicit_with_supplied_action(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
			supplied_action: impl FnOnce() -> <Self as ScopedResumeTypes<'a>>::OperationProgram + 'a,
		) -> RunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();

			supplied_action().bind(move |operation_value| outer(operation_value))
		}
	}
}

pub use inner::*;
