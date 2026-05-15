#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		super::super::inner::ArcRun,
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
				ArcCatList,
				ArcFree,
				arc_free::{
					ArcContinuation,
					ArcTypeErasedValue,
				},
				effects::{
					coproduct::CNil,
					interpreter::{
						ArcScopedResume,
						DispatchHandlers,
						ScopedResumeTypes,
					},
				},
			},
		},
		core::marker::PhantomData,
		fp_macros::*,
	};
	#[doc(hidden)]
	/// Type-erased inner `ArcFree` used by continuation-aware `ArcRun`
	/// stepping.
	pub type RawArcRunFree<R, S> = ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>;

	#[doc(hidden)]
	/// Pending `ArcFree` continuations carried outside a raw suspended
	/// layer during continuation-aware `ArcRun` stepping.
	pub type ArcRunContinuations<R, S> = ArcCatList<ArcContinuation<NodeBrand<R, S>>>;

	/// Result-polymorphic first-order replacement protocol for `ArcRun`.
	///
	/// Raw scoped handlers can select an action whose result type is
	/// different from the final outer program result. A replacement
	/// closure monomorphic in the outer `A` cannot safely rewrite
	/// first-order effects inside that selected action before the saved
	/// continuation queue resumes. This protocol lets `ArcRun`
	/// replacement recurse at the current branch result type while
	/// preserving `Send + Sync` requirements.
	#[document_type_parameters(
		"The first-order effect brand being replaced.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	#[document_parameters("The result-polymorphic replacement instance.")]
	pub trait ArcRunFirstOrderReplacer<EBrand, R, S>: Send + Sync
	where
		EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static, {
		/// Replaces one lowered first-order operation at the current
		/// branch result type.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters(
			"The lowered first-order operation whose continuation stays in the original row."
		)]
		#[document_returns("The replacement program in the original row.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::{
		/// 			ArcRun,
		/// 			ArcRunFirstOrderReplacer,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct IdentityPassThrough;
		///
		/// impl ArcRunFirstOrderReplacer<IdentityBrand, Row, CNilBrand> for IdentityPassThrough {
		/// 	fn replace<T: Clone + Send + Sync + 'static>(
		/// 		&self,
		/// 		effect: Identity<ArcRun<Row, CNilBrand, T>>,
		/// 	) -> ArcRun<Row, CNilBrand, T> {
		/// 		effect.0
		/// 	}
		/// }
		///
		/// let prog: ArcRun<Row, CNilBrand, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(7));
		/// let replaced =
		/// 	prog.interpose_with_replacer::<IdentityBrand, _, CNilBrand, _>(IdentityPassThrough);
		/// let result = replaced.handle(
		/// 	fp_library::handlers! {
		/// 		IdentityBrand: |op: Identity<ArcRun<Row, CNilBrand, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 7);
		/// ```
		fn replace<T: Clone + Send + Sync + 'static>(
			&self,
			effect: Apply!(
				<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, T>>
			),
		) -> ArcRun<R, S, T>;
	}

	#[doc(hidden)]
	/// Arc-backed carrier for a selected raw scoped action.
	///
	/// The action stays in erased `ArcFree` form until the dispatcher
	/// chooses whether to resume it unchanged, append result-preserving
	/// post-action work, or transform it before the suspended outer
	/// continuation queue is reattached.
	#[allow(
		dead_code,
		reason = "Carrier-aware scoped dispatch wiring constructs this carrier through raw scoped handler impls; focused tests and production wiring exercise different targets."
	)]
	pub(crate) struct ArcRunRawScopedContinuation<R, S, A>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Send + Sync + 'static, {
		/// The selected scoped action before the suspended `ArcRun`'s
		/// outer continuations have been reattached.
		pub(crate) action: RawArcRunFree<R, S>,
		/// The pending continuation queue captured from the suspended
		/// `ArcRun`.
		pub(crate) continuations: ArcRunContinuations<R, S>,
		/// Carries the final result type without owning a value of that type.
		pub(crate) result: PhantomData<fn() -> A>,
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type."
	)]
	impl<R, S, A> ScopedResumeTypes<'static> for ArcRunRawScopedContinuation<R, S, A>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Send + Sync + 'static,
	{
		type ActionProgram = RawArcRunFree<R, S>;
		type ActionValue = ArcTypeErasedValue;
		type OperationProgram = RawArcRunFree<R, S>;
		type OperationValue = ArcTypeErasedValue;
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The Arc-backed scoped-continuation carrier.")]
	impl<R, S, A, FirstLayer> ArcScopedResume<'static, FirstLayer, ArcRun<R, S, A>>
		for ArcRunRawScopedContinuation<R, S, A>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone + Send + Sync,
	{
		/// Resume the raw action by reattaching the suspended `ArcRun`
		/// continuation queue.
		#[document_signature]
		///
		#[document_parameters("The first-order handler list retained by the carrier contract.")]
		#[document_returns("The resumed `ArcRun` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// let run: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_arc(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
		) -> ArcRun<R, S, A> {
			ArcRun::from_arc_free(ArcFree::continue_from_erased(self.action, self.continuations))
		}

		/// Append raw post-action work before reattaching the suspended
		/// `ArcRun` continuation queue.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The result-preserving raw continuation to apply before outer continuations."
		)]
		#[document_returns("The resumed `ArcRun` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// let run: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(41).map(|value| value + 1);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_arc_with_post_action(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
			post_action: impl Fn(
				<Self as ScopedResumeTypes<'static>>::ActionValue,
			) -> <Self as ScopedResumeTypes<'static>>::ActionProgram
			+ Send
			+ Sync
			+ 'static,
		) -> ArcRun<R, S, A> {
			let action = ArcFree::<NodeBrand<R, S>, ArcTypeErasedValue>::append_erased_continuation(
				self.action,
				post_action,
			);
			ArcRun::from_arc_free(ArcFree::continue_from_erased(action, self.continuations))
		}

		/// Transform the raw action before reattaching the suspended
		/// `ArcRun` continuation queue.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The raw action transform to apply before outer continuations."
		)]
		#[document_returns("The resumed `ArcRun` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// let run: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(41).map(|value| value + 1);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_arc_with_action_transform(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
			transform: impl Fn(
				<Self as ScopedResumeTypes<'static>>::ActionProgram,
			) -> <Self as ScopedResumeTypes<'static>>::ActionProgram
			+ Send
			+ Sync
			+ 'static,
		) -> ArcRun<R, S, A> {
			ArcRun::from_arc_free(ArcFree::continue_from_erased(
				transform(self.action),
				self.continuations,
			))
		}
	}

	#[doc(hidden)]
	/// Internal adapter for one scoped-handler cell in the raw `ArcRun`
	/// interpreter path.
	///
	/// Standard scoped handlers implement this trait so `ArcRun` can
	/// keep the pending continuation queue outside the scoped layer until
	/// the active row branch is known.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type.",
		"The scoped effect brand handled by this cell.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The scoped-handler handler value.")]
	pub trait DispatchArcRunRawScopedHandler<R, S, A, SBrand, FirstLayer>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		SBrand: Kind_cdc7cd43dac7585f + 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone + Send + Sync, {
		/// Dispatches one raw scoped layer with its pending
		/// continuation queue still outside the layer.
		#[document_signature]
		///
		#[document_parameters(
			"The raw scoped layer carrying type-erased branch programs.",
			"The pending continuation queue for the suspended `ArcRun`.",
			"The first-order handler list used by nested interpretation."
		)]
		#[document_returns("The next `ArcRun` program produced by the scoped handler.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// let run: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn dispatch_arc_run_raw_scoped_head(
			&self,
			layer: Apply!(
				<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawArcRunFree<R, S>>
			),
			continuations: ArcRunContinuations<R, S>,
			fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
		) -> ArcRun<R, S, A>;
	}

	#[doc(hidden)]
	/// Internal recursive dispatcher for raw scoped `ArcRun` layers.
	///
	/// This mirrors
	/// [`DispatchScopedHandlers`](crate::types::effects::interpreter::DispatchScopedHandlers)
	/// but keeps the pending
	/// `ArcFree` continuation queue outside the scoped layer until the
	/// active row branch is known.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type.",
		"The raw scoped row layer shape.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The scoped-handler list.")]
	pub trait DispatchArcRunRawScopedHandlers<R, S, A, ScopedLayer, FirstLayer>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		ScopedLayer: 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone + Send + Sync, {
		/// Dispatches the active raw scoped row branch.
		#[document_signature]
		///
		#[document_parameters(
			"The raw scoped row layer.",
			"The pending continuation queue for the suspended `ArcRun`.",
			"The first-order handler list used by nested interpretation."
		)]
		#[document_returns("The next `ArcRun` program produced by the matching scoped handler.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// let run: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(5);
		/// assert_eq!(run.extract(), 5);
		/// ```
		fn dispatch_arc_run_raw_scoped(
			&self,
			layer: ScopedLayer,
			continuations: ArcRunContinuations<R, S>,
			fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
		) -> ArcRun<R, S, A>;
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The empty scoped-handler list.")]
	impl<R, S, A, FirstLayer> DispatchArcRunRawScopedHandlers<R, S, A, CNil, FirstLayer>
		for crate::types::effects::handlers::ScopedHandlersNil
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone + Send + Sync,
	{
		/// Base case for an empty scoped row.
		#[document_signature]
		///
		#[document_parameters(
			"The uninhabited scoped row layer.",
			"The pending continuation queue.",
			"The first-order handler list."
		)]
		#[document_returns("Diverges; the scoped layer is uninhabited.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// let run: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(11);
		/// assert_eq!(run.extract(), 11);
		/// ```
		fn dispatch_arc_run_raw_scoped(
			&self,
			layer: CNil,
			_continuations: ArcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
		) -> ArcRun<R, S, A> {
			match layer {}
		}
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type.",
		"The scoped effect brand at this row position.",
		"The handler value type.",
		"The tail scoped-handler list type.",
		"The remaining scoped row layer shape.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The scoped-handler cons cell.")]
	impl<R, S, A, SBrand, F, T, Rest, FirstLayer>
		DispatchArcRunRawScopedHandlers<
			R,
			S,
			A,
			crate::types::effects::coproduct::Coproduct<
				Apply!(
					<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawArcRunFree<R, S>>
				),
				Rest,
			>,
			FirstLayer,
		>
		for crate::types::effects::handlers::ScopedHandlersCons<
			crate::types::effects::handlers::ScopedHandler<SBrand, F>,
			T,
		>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		SBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
		F: DispatchArcRunRawScopedHandler<R, S, A, SBrand, FirstLayer>,
		T: DispatchArcRunRawScopedHandlers<R, S, A, Rest, FirstLayer>,
		Rest: 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone + Send + Sync,
	{
		/// Cons-cell case for raw scoped rows.
		#[document_signature]
		///
		#[document_parameters(
			"The raw scoped row layer.",
			"The pending continuation queue for the suspended `ArcRun`.",
			"The first-order handler list used by nested interpretation."
		)]
		#[document_returns("The next `ArcRun` program produced by the matching scoped handler.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// let run: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(13);
		/// assert_eq!(run.extract(), 13);
		/// ```
		fn dispatch_arc_run_raw_scoped(
			&self,
			layer: crate::types::effects::coproduct::Coproduct<
				Apply!(
					<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawArcRunFree<R, S>>
				),
				Rest,
			>,
			continuations: ArcRunContinuations<R, S>,
			fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
		) -> ArcRun<R, S, A> {
			match layer {
				crate::types::effects::coproduct::Coproduct::Inl(scoped) => self
					.head
					.run
					.dispatch_arc_run_raw_scoped_head(scoped, continuations, fo_handlers),
				crate::types::effects::coproduct::Coproduct::Inr(rest) =>
					self.tail.dispatch_arc_run_raw_scoped(rest, continuations, fo_handlers),
			}
		}
	}

	#[doc(hidden)]
	/// Arc-substrate carrier for a selected scoped action.
	///
	/// The carrier keeps the action and the action's outer continuation
	/// separate while preserving the Arc-family multi-shot contract: cloning
	/// the carrier is O(1), and post-action work is a reusable `Send + Sync`
	/// `Fn` continuation.
	#[derive(Clone)]
	#[allow(
		dead_code,
		reason = "Carrier-aware scoped dispatch wiring constructs the ArcRun carrier later; focused tests exercise it directly until production wiring exists."
	)]
	pub(crate) struct ArcRunScopedContinuation<R, S, Action, Final, K>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		Action: Clone + Send + Sync + 'static,
		Final: Send + Sync + 'static,
		K: Fn(Action) -> ArcRun<R, S, Final> + Send + Sync + 'static, {
		/// The selected scoped action before its outer continuation has
		/// been reattached.
		pub(crate) action: ArcRun<R, S, Action>,
		/// The action's outer continuation, still outside the selected action.
		pub(crate) outer: <ArcBrand as RefCountedPointer>::Of<'static, K>,
		/// Carries the final result type without owning a value of that type.
		pub(crate) result: PhantomData<fn() -> Final>,
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type."
	)]
	impl<R, S, Action, Final, K> ScopedResumeTypes<'static>
		for ArcRunScopedContinuation<R, S, Action, Final, K>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		Action: Clone + Send + Sync + 'static,
		Final: Send + Sync + 'static,
		K: Fn(Action) -> ArcRun<R, S, Final> + Send + Sync + 'static,
	{
		type ActionProgram = ArcRun<R, S, Action>;
		type ActionValue = Action;
		type OperationProgram = ArcRun<R, S, Action>;
		type OperationValue = Action;
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The ArcRun scoped-continuation carrier.")]
	impl<R, S, Action, Final, K, FirstLayer>
		ArcScopedResume<'static, FirstLayer, ArcRun<R, S, Final>>
		for ArcRunScopedContinuation<R, S, Action, Final, K>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		Action: Clone + Send + Sync + 'static,
		Final: Send + Sync + 'static,
		K: Fn(Action) -> ArcRun<R, S, Final> + Send + Sync + 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone,
	{
		/// Resume the selected action by reattaching its outer continuation.
		#[document_signature]
		///
		#[document_parameters("The first-order handler list retained by the carrier contract.")]
		#[document_returns("The resumed `ArcRun` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// let run: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_arc(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, Final>>,
		) -> ArcRun<R, S, Final> {
			let outer = self.outer.clone();
			self.action
				.bind(move |action_value: Action| -> ArcRun<R, S, Final> { outer(action_value) })
		}

		/// Insert a result-preserving action program before reattaching
		/// the selected action's outer continuation.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The result-preserving action program to apply before the outer continuation."
		)]
		#[document_returns("The resumed `ArcRun` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// let run: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(41);
		/// let incremented = run.bind(|value| ArcRun::pure(value + 1));
		/// assert_eq!(incremented.extract(), 42);
		/// ```
		fn resume_arc_with_post_action(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, Final>>,
			post_action: impl Fn(
				<Self as ScopedResumeTypes<'static>>::ActionValue,
			) -> <Self as ScopedResumeTypes<'static>>::ActionProgram
			+ Send
			+ Sync
			+ 'static,
		) -> ArcRun<R, S, Final> {
			let outer = self.outer.clone();

			self.action.bind(move |action_value: Action| -> ArcRun<R, S, Final> {
				let outer = outer.clone();
				let post_program: ArcRun<R, S, Action> = post_action(action_value);
				post_program
					.bind(move |post_value: Action| -> ArcRun<R, S, Final> { outer(post_value) })
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
		#[document_returns("The resumed `ArcRun` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// let run: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(41);
		/// let incremented = run.bind(|value| ArcRun::pure(value + 1));
		/// assert_eq!(incremented.extract(), 42);
		/// ```
		fn resume_arc_with_action_transform(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, Final>>,
			transform: impl Fn(
				<Self as ScopedResumeTypes<'static>>::ActionProgram,
			) -> <Self as ScopedResumeTypes<'static>>::ActionProgram
			+ Send
			+ Sync
			+ 'static,
		) -> ArcRun<R, S, Final> {
			let outer = self.outer.clone();

			transform(self.action)
				.bind(move |action_value: Action| -> ArcRun<R, S, Final> { outer(action_value) })
		}
	}
}

pub use inner::*;
