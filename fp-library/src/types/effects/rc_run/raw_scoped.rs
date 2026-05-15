#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		super::super::inner::RcRun,
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
				RcCatList,
				RcFree,
				effects::{
					coproduct::CNil,
					interpreter::{
						DispatchHandlers,
						RcScopedResume,
						ScopedResumeTypes,
					},
				},
				rc_free::{
					RcContinuation,
					RcTypeErasedValue,
				},
			},
		},
		core::marker::PhantomData,
		fp_macros::*,
	};
	#[doc(hidden)]
	/// Type-erased inner `RcFree` used by continuation-aware `RcRun`
	/// stepping.
	pub type RawRcRunFree<R, S> = RcFree<NodeBrand<R, S>, RcTypeErasedValue>;

	#[doc(hidden)]
	/// Pending `RcFree` continuations carried outside a raw suspended
	/// layer during continuation-aware `RcRun` stepping.
	pub type RcRunContinuations<R, S> = RcCatList<RcContinuation<NodeBrand<R, S>>>;

	/// Result-polymorphic first-order replacement protocol for `RcRun`.
	///
	/// Raw scoped handlers can select an action whose result type is
	/// different from the final outer program result. A replacement
	/// closure monomorphic in the outer `A` cannot safely rewrite
	/// first-order effects inside that selected action before the saved
	/// continuation queue resumes. This protocol lets `RcRun`
	/// replacement recurse at the current branch result type.
	#[document_type_parameters(
		"The first-order effect brand being replaced.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	#[document_parameters("The result-polymorphic replacement instance.")]
	pub trait RcRunFirstOrderReplacer<EBrand, R, S>
	where
		EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static, {
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
		/// 		effects::rc_run::{
		/// 			RcRun,
		/// 			RcRunFirstOrderReplacer,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct IdentityPassThrough;
		///
		/// impl RcRunFirstOrderReplacer<IdentityBrand, Row, CNilBrand> for IdentityPassThrough {
		/// 	fn replace<T: Clone + 'static>(
		/// 		&self,
		/// 		effect: Identity<RcRun<Row, CNilBrand, T>>,
		/// 	) -> RcRun<Row, CNilBrand, T> {
		/// 		effect.0
		/// 	}
		/// }
		///
		/// let prog: RcRun<Row, CNilBrand, i32> = RcRun::lift::<IdentityBrand, _>(Identity(7));
		/// let replaced =
		/// 	prog.interpose_with_replacer::<IdentityBrand, _, CNilBrand, _>(IdentityPassThrough);
		/// let result = replaced.handle(
		/// 	fp_library::handlers! {
		/// 		IdentityBrand: |op: Identity<RcRun<Row, CNilBrand, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 7);
		/// ```
		fn replace<T: Clone + 'static>(
			&self,
			effect: Apply!(
				<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, T>>
			),
		) -> RcRun<R, S, T>;
	}

	/// Result-polymorphic same-row first-order rewrite protocol for
	/// `RcRun`.
	///
	/// The traversal lowers matched `RcCoyoneda` layers, rewrites
	/// nested continuations, lets this protocol map the lowered effect
	/// value, and then re-embeds the rewritten operation in the
	/// original row. Handler-specific rewriters therefore preserve the
	/// effect constructor and do not need to prove row-membership
	/// bounds for every branch result type.
	#[document_type_parameters(
		"The first-order effect brand being rewritten.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	#[document_parameters("The result-polymorphic rewrite instance.")]
	pub trait RcRunFirstOrderRewriter<EBrand, R, S>
	where
		EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static, {
		/// Rewrites one lowered first-order operation at the current
		/// branch result type while preserving its effect constructor.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters(
			"The lowered first-order operation whose continuation stays in the original row."
		)]
		#[document_returns("The rewritten operation in the same effect constructor.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run::{
		/// 			RcRun,
		/// 			RcRunFirstOrderRewriter,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct IdentityPreserve;
		///
		/// impl RcRunFirstOrderRewriter<IdentityBrand, Row, CNilBrand> for IdentityPreserve {
		/// 	fn rewrite<T: Clone + 'static>(
		/// 		&self,
		/// 		effect: Identity<RcRun<Row, CNilBrand, T>>,
		/// 	) -> Identity<RcRun<Row, CNilBrand, T>> {
		/// 		effect
		/// 	}
		/// }
		///
		/// let prog: RcRun<Row, CNilBrand, i32> = RcRun::lift::<IdentityBrand, _>(Identity(7));
		/// let rewritten =
		/// 	prog.interpose_with_rewriter::<IdentityBrand, _, CNilBrand, _>(IdentityPreserve);
		/// let result = rewritten.handle(
		/// 	fp_library::handlers! {
		/// 		IdentityBrand: |op: Identity<RcRun<Row, CNilBrand, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 7);
		/// ```
		fn rewrite<T: Clone + 'static>(
			&self,
			effect: Apply!(
				<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, T>>
			),
		) -> Apply!(
			<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, T>>
		);
	}

	/// Result-changing first-order accumulation protocol for `RcRun`.
	///
	/// The traversal consumes matching first-order operations inside a
	/// selected action and returns the selected action value paired with
	/// an explicit accumulator. Handler-specific implementations decide
	/// how one matched operation contributes to the accumulator; the
	/// wrapper traversal owns row projection, continuation preservation,
	/// and non-matching operation re-embedding.
	#[document_type_parameters(
		"The first-order effect brand being accumulated.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The accumulated value type."
	)]
	#[document_parameters("The result-polymorphic accumulation instance.")]
	#[doc(hidden)]
	pub trait RcRunFirstOrderAccumulator<EBrand, R, S, Acc>
	where
		EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Acc: Clone + 'static, {
		/// Produces the accumulator value for a selected action with no
		/// matching first-order operations.
		#[document_signature]
		#[document_returns("The neutral accumulated value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run::{
		/// 			RcRun,
		/// 			RcRunFirstOrderPreservingAccumulator,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct CountIdentity;
		///
		/// impl RcRunFirstOrderPreservingAccumulator<IdentityBrand, Row, CNilBrand, usize> for CountIdentity {
		/// 	fn empty(&self) -> usize {
		/// 		0
		/// 	}
		///
		/// 	fn accumulate_preserving<T: Clone + 'static>(
		/// 		&self,
		/// 		effect: Identity<RcRun<Row, CNilBrand, (T, usize)>>,
		/// 	) -> Identity<RcRun<Row, CNilBrand, (T, usize)>> {
		/// 		Identity(effect.0.map(|(value, count)| (value, count + 1)))
		/// 	}
		/// }
		///
		/// assert_eq!(CountIdentity.empty(), 0);
		/// ```
		fn empty(&self) -> Acc;

		/// Consumes one lowered first-order operation after its continuation
		/// has already been recursively accumulated.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters(
			"The lowered first-order operation whose continuation now returns `(value, accumulated)`."
		)]
		#[document_returns("The accumulated program in the original row.")]
		#[document_examples]
		///
		/// ```
		/// let current_log = "selected ".to_string();
		/// let accumulated_suffix = "action".to_string();
		/// assert_eq!(current_log + &accumulated_suffix, "selected action");
		/// ```
		fn accumulate<T: Clone + 'static>(
			&self,
			effect: Apply!(
				<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, (T, Acc)>>
			),
		) -> RcRun<R, S, (T, Acc)>;
	}

	/// Result-changing first-order preserving accumulation protocol for
	/// `RcRun`.
	///
	/// This protocol is the preserving counterpart of
	/// [`RcRunFirstOrderAccumulator`]. The traversal walks a selected
	/// action once, accumulates matching first-order operations, and
	/// rebuilds each matched operation into the original row so an outer
	/// handler can still observe it. Handler-specific implementations
	/// decide how one matched operation contributes to the accumulator
	/// and how the lowered operation is re-emitted; the wrapper traversal
	/// owns row projection, recursive continuation preservation, and
	/// non-matching operation re-embedding.
	#[document_type_parameters(
		"The first-order effect brand being accumulated.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The accumulated value type."
	)]
	#[document_parameters("The result-polymorphic preserving accumulation instance.")]
	#[doc(hidden)]
	pub trait RcRunFirstOrderPreservingAccumulator<EBrand, R, S, Acc>
	where
		EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Acc: Clone + 'static, {
		/// Produces the accumulator value for a selected action with no
		/// matching first-order operations.
		#[document_signature]
		#[document_returns("The neutral accumulated value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run::{
		/// 			RcRun,
		/// 			RcRunFirstOrderPreservingAccumulator,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct CountIdentity;
		///
		/// impl RcRunFirstOrderPreservingAccumulator<IdentityBrand, Row, CNilBrand, usize> for CountIdentity {
		/// 	fn empty(&self) -> usize {
		/// 		0
		/// 	}
		///
		/// 	fn accumulate_preserving<T: Clone + 'static>(
		/// 		&self,
		/// 		effect: Identity<RcRun<Row, CNilBrand, (T, usize)>>,
		/// 	) -> Identity<RcRun<Row, CNilBrand, (T, usize)>> {
		/// 		Identity(effect.0.map(|(value, count)| (value, count + 1)))
		/// 	}
		/// }
		///
		/// assert_eq!(CountIdentity.empty(), 0);
		/// ```
		fn empty(&self) -> Acc;

		/// Rebuilds one lowered first-order operation after its
		/// continuation has already been recursively accumulated.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters(
			"The lowered first-order operation whose continuation now returns `(value, accumulated)`."
		)]
		#[document_returns(
			"The preserved first-order operation whose continuation includes the accumulated value."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run::{
		/// 			RcRun,
		/// 			RcRunFirstOrderPreservingAccumulator,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct CountIdentity;
		///
		/// impl RcRunFirstOrderPreservingAccumulator<IdentityBrand, Row, CNilBrand, usize> for CountIdentity {
		/// 	fn empty(&self) -> usize {
		/// 		0
		/// 	}
		///
		/// 	fn accumulate_preserving<T: Clone + 'static>(
		/// 		&self,
		/// 		effect: Identity<RcRun<Row, CNilBrand, (T, usize)>>,
		/// 	) -> Identity<RcRun<Row, CNilBrand, (T, usize)>> {
		/// 		Identity(effect.0.map(|(value, count)| (value, count + 1)))
		/// 	}
		/// }
		///
		/// let op = Identity(RcRun::<Row, CNilBrand, (i32, usize)>::pure((41, 0)));
		/// let preserved = CountIdentity.accumulate_preserving(op);
		/// let handled = preserved.0.handle_with::<IdentityBrand, _, CNilBrand>(
		/// 	|op: Identity<RcRun<CNilBrand, CNilBrand, (i32, usize)>>| op.0,
		/// );
		/// assert_eq!(handled.extract(), (41, 1));
		/// ```
		fn accumulate_preserving<T: Clone + 'static>(
			&self,
			effect: Apply!(
				<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, (T, Acc)>>
			),
		) -> Apply!(
			<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, (T, Acc)>>
		);
	}

	#[doc(hidden)]
	/// Rc-backed carrier for a selected raw scoped action.
	///
	/// The action stays in erased `RcFree` form until the dispatcher
	/// chooses whether to resume it unchanged, append result-preserving
	/// post-action work, or transform it before the suspended outer
	/// continuation queue is reattached.
	#[allow(
		dead_code,
		reason = "Carrier-aware scoped dispatch wiring constructs this carrier through raw scoped handler impls; focused tests and production wiring exercise different targets."
	)]
	pub(crate) struct RcRunRawScopedContinuation<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static, {
		/// The selected scoped action before the suspended `RcRun`'s
		/// outer continuations have been reattached.
		pub(crate) action: RawRcRunFree<R, S>,
		/// The pending continuation queue captured from the suspended
		/// `RcRun`.
		pub(crate) continuations: RcRunContinuations<R, S>,
		/// Carries the final result type without owning a value of that type.
		pub(crate) result: PhantomData<fn() -> A>,
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type."
	)]
	impl<R, S, A> ScopedResumeTypes<'static> for RcRunRawScopedContinuation<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		type ActionProgram = RawRcRunFree<R, S>;
		type ActionValue = RcTypeErasedValue;
		type OperationProgram = RawRcRunFree<R, S>;
		type OperationValue = RcTypeErasedValue;
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The Rc-backed scoped-continuation carrier.")]
	impl<R, S, A, FirstLayer> RcScopedResume<'static, FirstLayer, RcRun<R, S, A>>
		for RcRunRawScopedContinuation<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
	{
		/// Resume the raw action by reattaching the suspended `RcRun`
		/// continuation queue.
		#[document_signature]
		///
		#[document_parameters("The first-order handler list retained by the carrier contract.")]
		#[document_returns("The resumed `RcRun` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// let run: RcRun<CNilBrand, CNilBrand, i32> = RcRun::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_rc(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
		) -> RcRun<R, S, A> {
			RcRun::from_rc_free(RcFree::continue_from_erased(self.action, self.continuations))
		}

		/// Append raw post-action work before reattaching the suspended
		/// `RcRun` continuation queue.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The result-preserving raw continuation to apply before outer continuations."
		)]
		#[document_returns("The resumed `RcRun` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// let run: RcRun<CNilBrand, CNilBrand, i32> = RcRun::pure(41).map(|value| value + 1);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_rc_with_post_action(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
			post_action: impl Fn(
				<Self as ScopedResumeTypes<'static>>::ActionValue,
			) -> <Self as ScopedResumeTypes<'static>>::ActionProgram
			+ 'static,
		) -> RcRun<R, S, A> {
			let action = RcFree::<NodeBrand<R, S>, RcTypeErasedValue>::append_erased_continuation(
				self.action,
				post_action,
			);
			RcRun::from_rc_free(RcFree::continue_from_erased(action, self.continuations))
		}

		/// Transform the raw action before reattaching the suspended
		/// `RcRun` continuation queue.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The raw action transform to apply before outer continuations."
		)]
		#[document_returns("The resumed `RcRun` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// let run: RcRun<CNilBrand, CNilBrand, i32> = RcRun::pure(41).map(|value| value + 1);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_rc_with_action_transform(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
			transform: impl Fn(
				<Self as ScopedResumeTypes<'static>>::ActionProgram,
			) -> <Self as ScopedResumeTypes<'static>>::ActionProgram
			+ 'static,
		) -> RcRun<R, S, A> {
			RcRun::from_rc_free(RcFree::continue_from_erased(
				transform(self.action),
				self.continuations,
			))
		}
	}

	#[doc(hidden)]
	/// Internal adapter for one scoped-handler cell in the raw `RcRun`
	/// interpreter path.
	///
	/// Standard scoped handlers implement this trait so `RcRun` can
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
	pub trait DispatchRcRunRawScopedHandler<R, S, A, SBrand, FirstLayer>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		SBrand: Kind_cdc7cd43dac7585f + 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone, {
		/// Dispatches one raw scoped layer with its pending
		/// continuation queue still outside the layer.
		#[document_signature]
		///
		#[document_parameters(
			"The raw scoped layer carrying type-erased branch programs.",
			"The pending continuation queue for the suspended `RcRun`.",
			"The first-order handler list used by nested interpretation."
		)]
		#[document_returns("The next `RcRun` program produced by the scoped handler.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// let run: RcRun<CNilBrand, CNilBrand, i32> = RcRun::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn dispatch_rc_run_raw_scoped_head(
			&self,
			layer: Apply!(
				<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawRcRunFree<R, S>>
			),
			continuations: RcRunContinuations<R, S>,
			fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
		) -> RcRun<R, S, A>;
	}

	#[doc(hidden)]
	/// Internal recursive dispatcher for raw scoped `RcRun` layers.
	///
	/// This mirrors
	/// [`DispatchScopedHandlers`](crate::types::effects::interpreter::DispatchScopedHandlers)
	/// but keeps the pending
	/// `RcFree` continuation queue outside the scoped layer until the
	/// active row branch is known.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type.",
		"The raw scoped row layer shape.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The scoped-handler list.")]
	pub trait DispatchRcRunRawScopedHandlers<R, S, A, ScopedLayer, FirstLayer>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		ScopedLayer: 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone, {
		/// Dispatches the active raw scoped row branch.
		#[document_signature]
		///
		#[document_parameters(
			"The raw scoped row layer.",
			"The pending continuation queue for the suspended `RcRun`.",
			"The first-order handler list used by nested interpretation."
		)]
		#[document_returns("The next `RcRun` program produced by the matching scoped handler.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// let run: RcRun<CNilBrand, CNilBrand, i32> = RcRun::pure(5);
		/// assert_eq!(run.extract(), 5);
		/// ```
		fn dispatch_rc_run_raw_scoped(
			&self,
			layer: ScopedLayer,
			continuations: RcRunContinuations<R, S>,
			fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
		) -> RcRun<R, S, A>;
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The empty scoped-handler list.")]
	impl<R, S, A, FirstLayer> DispatchRcRunRawScopedHandlers<R, S, A, CNil, FirstLayer>
		for crate::types::effects::handlers::ScopedHandlersNil
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
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
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// let run: RcRun<CNilBrand, CNilBrand, i32> = RcRun::pure(11);
		/// assert_eq!(run.extract(), 11);
		/// ```
		fn dispatch_rc_run_raw_scoped(
			&self,
			layer: CNil,
			_continuations: RcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
		) -> RcRun<R, S, A> {
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
		DispatchRcRunRawScopedHandlers<
			R,
			S,
			A,
			crate::types::effects::coproduct::Coproduct<
				Apply!(
					<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawRcRunFree<R, S>>
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
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		SBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
		F: DispatchRcRunRawScopedHandler<R, S, A, SBrand, FirstLayer>,
		T: DispatchRcRunRawScopedHandlers<R, S, A, Rest, FirstLayer>,
		Rest: 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
	{
		/// Cons-cell case for raw scoped rows.
		#[document_signature]
		///
		#[document_parameters(
			"The raw scoped row layer.",
			"The pending continuation queue for the suspended `RcRun`.",
			"The first-order handler list used by nested interpretation."
		)]
		#[document_returns("The next `RcRun` program produced by the matching scoped handler.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// let run: RcRun<CNilBrand, CNilBrand, i32> = RcRun::pure(13);
		/// assert_eq!(run.extract(), 13);
		/// ```
		fn dispatch_rc_run_raw_scoped(
			&self,
			layer: crate::types::effects::coproduct::Coproduct<
				Apply!(
					<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawRcRunFree<R, S>>
				),
				Rest,
			>,
			continuations: RcRunContinuations<R, S>,
			fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
		) -> RcRun<R, S, A> {
			match layer {
				crate::types::effects::coproduct::Coproduct::Inl(scoped) => self
					.head
					.run
					.dispatch_rc_run_raw_scoped_head(scoped, continuations, fo_handlers),
				crate::types::effects::coproduct::Coproduct::Inr(rest) =>
					self.tail.dispatch_rc_run_raw_scoped(rest, continuations, fo_handlers),
			}
		}
	}

	#[doc(hidden)]
	/// Rc-substrate carrier for a selected scoped action.
	///
	/// The carrier keeps the action and the action's outer continuation
	/// separate while preserving the Rc-family multi-shot contract: cloning the
	/// carrier is O(1), and post-action work is a reusable `Fn` continuation.
	#[derive(Clone)]
	#[allow(
		dead_code,
		reason = "Carrier-aware scoped dispatch wiring constructs the RcRun carrier later; focused tests exercise it directly until production wiring exists."
	)]
	pub(crate) struct RcRunScopedContinuation<R, S, Action, Final, K>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'static,
		Final: 'static,
		K: Fn(Action) -> RcRun<R, S, Final> + 'static, {
		/// The selected scoped action before its outer continuation has
		/// been reattached.
		pub(crate) action: RcRun<R, S, Action>,
		/// The action's outer continuation, still outside the selected action.
		pub(crate) outer: <RcBrand as RefCountedPointer>::Of<'static, K>,
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
		for RcRunScopedContinuation<R, S, Action, Final, K>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'static,
		Final: 'static,
		K: Fn(Action) -> RcRun<R, S, Final> + 'static,
	{
		type ActionProgram = RcRun<R, S, Action>;
		type ActionValue = Action;
		type OperationProgram = RcRun<R, S, Action>;
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
	#[document_parameters("The RcRun scoped-continuation carrier.")]
	impl<R, S, Action, Final, K, FirstLayer> RcScopedResume<'static, FirstLayer, RcRun<R, S, Final>>
		for RcRunScopedContinuation<R, S, Action, Final, K>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'static,
		Final: 'static,
		K: Fn(Action) -> RcRun<R, S, Final> + 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
		>): Clone,
	{
		/// Resume the selected action by reattaching its outer continuation.
		#[document_signature]
		///
		#[document_parameters("The first-order handler list retained by the carrier contract.")]
		#[document_returns("The resumed `RcRun` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// let run: RcRun<CNilBrand, CNilBrand, i32> = RcRun::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_rc(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, Final>>,
		) -> RcRun<R, S, Final> {
			let outer = self.outer.clone();
			self.action
				.bind(move |action_value: Action| -> RcRun<R, S, Final> { outer(action_value) })
		}

		/// Insert a result-preserving action program before reattaching
		/// the selected action's outer continuation.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The result-preserving action program to apply before the outer continuation."
		)]
		#[document_returns("The resumed `RcRun` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// let run: RcRun<CNilBrand, CNilBrand, i32> = RcRun::pure(41);
		/// let incremented = run.bind(|value| RcRun::pure(value + 1));
		/// assert_eq!(incremented.extract(), 42);
		/// ```
		fn resume_rc_with_post_action(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, Final>>,
			post_action: impl Fn(
				<Self as ScopedResumeTypes<'static>>::ActionValue,
			) -> <Self as ScopedResumeTypes<'static>>::ActionProgram
			+ 'static,
		) -> RcRun<R, S, Final> {
			let outer = self.outer.clone();

			self.action.bind(move |action_value: Action| -> RcRun<R, S, Final> {
				let outer = outer.clone();
				let post_program: RcRun<R, S, Action> = post_action(action_value);
				post_program
					.bind(move |post_value: Action| -> RcRun<R, S, Final> { outer(post_value) })
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
		#[document_returns("The resumed `RcRun` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// let run: RcRun<CNilBrand, CNilBrand, i32> = RcRun::pure(41);
		/// let incremented = run.bind(|value| RcRun::pure(value + 1));
		/// assert_eq!(incremented.extract(), 42);
		/// ```
		fn resume_rc_with_action_transform(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, Final>>,
			transform: impl Fn(
				<Self as ScopedResumeTypes<'static>>::ActionProgram,
			) -> <Self as ScopedResumeTypes<'static>>::ActionProgram
			+ 'static,
		) -> RcRun<R, S, Final> {
			let outer = self.outer.clone();

			transform(self.action)
				.bind(move |action_value: Action| -> RcRun<R, S, Final> { outer(action_value) })
		}
	}
}

pub use inner::*;
