//! Erased-substrate Run program over a private representation backed by
//! [`Free`](crate::types::Free) and dual-row
//! [`NodeBrand`](crate::brands::NodeBrand) layers.
//!
//! `Run<R, S, A>` is the user-facing wrapper for the canonical
//! Run-style effect computation:
//!
//! The common representation case is a
//! `Free<NodeBrand<R, S>, A>` program. Around-action scoped handlers
//! also need an internal boundary-frame case where the selected action
//! and the outer continuation queue remain separate until the action or
//! recovery branch is known.
//!
//! The first-order row brand `R` carries the effect functors (typically
//! a [`CoproductBrand`](crate::brands::CoproductBrand) of
//! [`CoyonedaBrand`](crate::brands::CoyonedaBrand)-wrapped effects
//! terminated by [`CNilBrand`](crate::brands::CNilBrand)); the scoped
//! row brand `S` carries higher-order constructors (future scoped
//! work populates it with `Catch`, `Local`, etc.; for
//! first-order-only programs it stays as `CNilBrand`).
//!
//! `Run` is the Erased counterpart of `RunExplicit`. The Erased
//! substrate is single-shot, type-erases through `Box<dyn Any>`, has
//! O(1) `bind`, and is `'static`-only. It exposes its API via
//! inherent methods rather than Brand-dispatched type classes, so
//! do-notation is via the `run_do!` macro, not `m_do!`. Use
//! `RunExplicit` for non-`'static` payloads or when Brand-dispatched
//! typeclass-generic code is required.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				CNilBrand,
				NodeBrand,
				RcBrand,
			},
			classes::{
				Functor,
				MonadRec,
				Pointed,
				RefCountedPointer,
				WrapDrop,
			},
			functions::tail_rec_m,
			kinds::*,
			types::{
				CatList,
				Coyoneda,
				Free,
				effects::{
					coproduct::{
						CNil,
						Coproduct,
						CoproductEmbedder,
					},
					handlers::{
						ScopedHandler,
						ScopedHandlersCons,
						ScopedHandlersNil,
					},
					interpreter::{
						DefaultScopedResume,
						DispatchHandlers,
						DispatchScopedHandlers,
						ScopedResumeTypes,
					},
					member::Member,
					node::Node,
				},
				free::{
					Continuation,
					FreeRawStep,
					FreeView,
					TypeErasedValue,
				},
			},
		},
		core::{
			marker::PhantomData,
			ops::ControlFlow,
		},
		fp_macros::*,
	};

	/// Erased-substrate Run program.
	///
	/// The wrapper exists so user-facing API (`pure`, `peel`, `send`,
	/// effect-row narrowing, handler types) can be expressed without
	/// leaking the underlying representation. Ordinary programs are
	/// stored as [`Free<NodeBrand<R, S>, A>`](crate::types::Free).
	/// Around-action scoped boundaries may instead store a raw scoped
	/// layer plus the pending outer continuation queue so branch
	/// selection can happen before the continuation is reattached.
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand (typically `CNilBrand` for first-order-only programs).",
		"The result type."
	)]
	pub struct Run<R, S, A>(pub(crate) RunRepresentation<R, S, A>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static;

	/// Private storage for default `Run`.
	///
	/// Most programs remain Free-backed. Scoped boundary frames carry a
	/// raw scoped row layer and the pending erased continuation queue
	/// separately; this is the internal shape needed by single-shot
	/// around-action handlers such as Box-backed Catch.
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The final result type."
	)]
	pub(crate) enum RunRepresentation<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static, {
		/// Ordinary Free-backed program.
		Free(Free<NodeBrand<R, S>, A>),
		/// Raw scoped boundary frame with the outer continuation still
		/// outside the selected action.
		ScopedBoundary(RunScopedBoundaryFrame<R, S, A>),
	}

	/// Raw scoped boundary frame for default `Run`.
	///
	/// The frame stores the raw scoped row layer as
	/// `Free<NodeBrand<R, S>, TypeErasedValue>` actions plus the
	/// continuation queue that should run after the selected action
	/// completes. Keeping these slots separate prevents a single-shot
	/// continuation from being copied into every branch of a
	/// Box-backed scoped operation before branch selection.
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The final result type."
	)]
	pub(crate) struct RunScopedBoundaryFrame<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static, {
		/// Raw scoped layer whose inner programs have erased result
		/// type.
		pub(crate) layer: Apply!(
			<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawRunFree<R, S>>
		),
		/// Pending outer continuations, still outside the scoped layer.
		pub(crate) continuations: RunContinuations<R, S>,
		/// Carries the final result type without owning a value of that type.
		pub(crate) result: PhantomData<fn() -> A>,
	}

	#[doc(hidden)]
	/// Type-erased inner `Free` used by continuation-aware `Run`
	/// stepping.
	pub type RawRunFree<R, S> = Free<NodeBrand<R, S>, TypeErasedValue>;

	#[doc(hidden)]
	/// Pending `Free` continuations carried outside a raw suspended
	/// layer during continuation-aware `Run` stepping.
	pub type RunContinuations<R, S> = CatList<Continuation<NodeBrand<R, S>>>;

	/// Result-polymorphic first-order handler protocol for default `Run`.
	///
	/// Boundary-backed around-action programs can contain selected
	/// action/recovery programs whose result type is not the final outer
	/// `Run` result type. A closure monomorphic in the outer result
	/// cannot narrow those selected programs before the pending
	/// continuation queue runs. This private protocol gives the
	/// interpreter recursion a handler method that is generic in the
	/// current branch result type.
	#[document_type_parameters(
		"The first-order effect brand being interpreted out of the row.",
		"The narrowed first-order row brand.",
		"The scoped-effect row brand."
	)]
	#[document_parameters("The result-polymorphic handler instance.")]
	pub trait RunFirstOrderHandler<EBrand, RMinusE, S>
	where
		EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static, {
		/// Handles one lowered first-order operation at the current
		/// branch result type.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters(
			"The lowered first-order operation whose continuation already lives in the narrowed row."
		)]
		#[document_returns("The handler result in the narrowed row.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::{
		/// 			Run,
		/// 			RunFirstOrderHandler,
		/// 		},
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// struct IdentityHandler;
		///
		/// impl RunFirstOrderHandler<IdentityBrand, EmptyRow, CNilBrand> for IdentityHandler {
		/// 	fn handle<T: 'static>(
		/// 		&self,
		/// 		effect: Identity<Run<EmptyRow, CNilBrand, T>>,
		/// 	) -> Run<EmptyRow, CNilBrand, T> {
		/// 		effect.0
		/// 	}
		/// }
		///
		/// let prog: Run<FullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: Run<EmptyRow, CNilBrand, i32> =
		/// 	prog.interpret_with_handler::<IdentityBrand, _, EmptyRow>(IdentityHandler);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		fn handle<T: 'static>(
			&self,
			effect: Apply!(
				<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<RMinusE, S, T>>
			),
		) -> Run<RMinusE, S, T>;
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The final result type."
	)]
	#[document_parameters("The private `Run` representation.")]
	impl<R, S, A> RunRepresentation<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Stores an ordinary Free-backed program in the private representation.
		#[document_signature]
		#[document_parameters("The Free-backed program to store.")]
		#[document_returns("A private `Run` representation containing the Free-backed program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Free,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::from_free(Free::pure(42));
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn free(free: Free<NodeBrand<R, S>, A>) -> Self {
			RunRepresentation::Free(free)
		}

		/// Lowers the private representation back to a Free-backed program.
		#[document_signature]
		#[document_returns("The Free program represented by this private `Run` representation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Free,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(7);
		/// let free: Free<NodeBrand<CNilBrand, CNilBrand>, i32> = run.into_free();
		/// assert!(matches!(free.resume(), Ok(7)));
		/// ```
		fn into_free(self) -> Free<NodeBrand<R, S>, A> {
			match self {
				RunRepresentation::Free(free) => free,
				RunRepresentation::ScopedBoundary(boundary) => boundary.into_free(),
			}
		}

		/// Steps the private representation without converting a raw
		/// scoped-boundary frame through the public Free view.
		#[document_signature]
		#[document_returns("The next raw step represented by this private `Run` representation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn into_raw_step(self) -> FreeRawStep<NodeBrand<R, S>, A> {
			match self {
				RunRepresentation::Free(free) => free.into_raw_step(),
				RunRepresentation::ScopedBoundary(boundary) => boundary.into_raw_step(),
			}
		}

		/// Sequences a continuation after the represented program.
		#[document_signature]
		#[document_type_parameters("The result type produced by the continuation.")]
		#[document_parameters(
			"The continuation to run after this representation produces a value."
		)]
		#[document_returns("A private representation for the sequenced program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(2).bind(|x| Run::pure(x + 40));
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn bind<B: 'static>(
			self,
			f: impl FnOnce(A) -> Run<R, S, B> + 'static,
		) -> RunRepresentation<R, S, B> {
			match self {
				RunRepresentation::Free(free) =>
					RunRepresentation::free(free.bind(move |a| f(a).into_free())),
				RunRepresentation::ScopedBoundary(boundary) =>
					RunRepresentation::ScopedBoundary(boundary.bind(f)),
			}
		}

		/// Maps a value-producing function over the represented program.
		#[document_signature]
		#[document_type_parameters("The mapped result type.")]
		#[document_parameters("The function to apply after this representation produces a value.")]
		#[document_returns("A private representation for the mapped program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(21).map(|x| x * 2);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn map<B: 'static>(
			self,
			f: impl FnOnce(A) -> B + 'static,
		) -> RunRepresentation<R, S, B> {
			self.bind(move |a| Run::from_free(Free::pure(f(a))))
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The final result type."
	)]
	#[document_parameters("The raw scoped boundary frame.")]
	impl<R, S, A> RunScopedBoundaryFrame<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Lowers the boundary frame into a Free program without pushing
		/// the pending continuation queue into the scoped layer.
		#[document_signature]
		#[document_returns("The Free-backed program represented by this boundary frame.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn into_free(self) -> Free<NodeBrand<R, S>, A> {
			let node: Apply!(
				<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRunFree<R, S>,
				>
			) = Node::Scoped(self.layer);
			Free::from_raw_parts(Some(FreeView::Suspend(node)), self.continuations)
		}

		/// Steps this boundary frame with the continuation queue still
		/// outside the scoped layer.
		#[document_signature]
		#[document_returns("A raw suspended scoped step for this boundary frame.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn into_raw_step(self) -> FreeRawStep<NodeBrand<R, S>, A> {
			let layer: Apply!(
				<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRunFree<R, S>,
				>
			) = Node::Scoped(self.layer);
			FreeRawStep::Suspended {
				layer,
				continuations: self.continuations,
			}
		}

		/// Appends a result continuation outside the boundary frame's
		/// selected action.
		#[document_signature]
		#[document_type_parameters("The result type produced by the continuation.")]
		#[document_parameters("The continuation to append outside the boundary frame.")]
		#[document_returns("A boundary frame with the continuation appended.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(40).bind(|x| Run::pure(x + 2));
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn bind<B: 'static>(
			self,
			f: impl FnOnce(A) -> Run<R, S, B> + 'static,
		) -> RunScopedBoundaryFrame<R, S, B> {
			let continuation: Continuation<NodeBrand<R, S>> = Box::new(move |value| {
				#[expect(clippy::expect_used, reason = "Type maintained by Run boundary invariant")]
				let a: A = *value.downcast().expect("Type mismatch in Run boundary bind");
				f(a).into_free().cast_erased()
			});

			RunScopedBoundaryFrame {
				layer: self.layer,
				continuations: self.continuations.snoc(continuation),
				result: PhantomData,
			}
		}
	}

	#[doc(hidden)]
	/// Default `Run` carrier for a selected raw scoped action.
	///
	/// The action stays in erased `Free` form until the carrier chooses
	/// whether to resume it unchanged or append one result-preserving
	/// post-action continuation before the pending outer continuation
	/// queue.
	#[allow(
		dead_code,
		reason = "Carrier-aware scoped dispatch wiring constructs the default Run carrier later; focused tests exercise it directly, so expect(dead_code) is target-dependent across lib and test builds."
	)]
	pub(crate) struct RunScopedContinuation<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static, {
		/// The selected scoped action before the suspended `Run`'s outer
		/// continuations have been reattached.
		pub(crate) action: RawRunFree<R, S>,
		/// The pending continuation queue captured from the suspended `Run`.
		pub(crate) continuations: RunContinuations<R, S>,
		/// Carries the final result type without owning a value of that type.
		pub(crate) result: PhantomData<fn() -> A>,
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type."
	)]
	impl<R, S, A> ScopedResumeTypes<'static> for RunScopedContinuation<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		type ActionProgram = RawRunFree<R, S>;
		type ActionValue = TypeErasedValue;
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The default `Run` scoped-continuation carrier.")]
	impl<R, S, A, FirstLayer> DefaultScopedResume<'static, FirstLayer, Run<R, S, A>>
		for RunScopedContinuation<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
		FirstLayer: 'static,
	{
		/// Resume the raw action by reattaching the suspended `Run` continuation
		/// queue.
		#[document_signature]
		///
		#[document_parameters("The first-order handler list retained by the carrier contract.")]
		#[document_returns("The resumed default `Run` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_default(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
		) -> Run<R, S, A> {
			Run::from_free(Free::continue_from_erased(self.action, self.continuations))
		}

		/// Append a raw post-action continuation before reattaching the suspended
		/// `Run` continuation queue.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The result-preserving raw continuation to apply before outer continuations."
		)]
		#[document_returns("The resumed default `Run` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_default_with_post_action(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
			post_action: impl Fn(
				<Self as ScopedResumeTypes<'static>>::ActionValue,
			) -> <Self as ScopedResumeTypes<'static>>::ActionProgram
			+ 'static,
		) -> Run<R, S, A> {
			let continuations =
				CatList::singleton(Box::new(post_action) as Continuation<NodeBrand<R, S>>)
					.append(self.continuations);
			Run::from_free(Free::continue_from_erased(self.action, continuations))
		}

		/// Transform the raw action before reattaching the suspended `Run`
		/// continuation queue.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The raw action transform to apply before outer continuations."
		)]
		#[document_returns("The resumed default `Run` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_default_with_action_transform(
			self,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
			transform: impl Fn(
				<Self as ScopedResumeTypes<'static>>::ActionProgram,
			) -> <Self as ScopedResumeTypes<'static>>::ActionProgram
			+ 'static,
		) -> Run<R, S, A> {
			Run::from_free(Free::continue_from_erased(transform(self.action), self.continuations))
		}
	}

	#[doc(hidden)]
	/// Internal adapter for one scoped-handler cell in the raw `Run`
	/// interpreter path.
	///
	/// Most scoped handlers use the blanket implementation, which
	/// attaches the pending continuation queue to the active branch and
	/// then delegates to the ordinary scoped-handler contract. Branching single-shot
	/// handlers such as Box-backed `Catch` implement this trait directly
	/// so they can choose the active branch before the continuation is
	/// attached.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type.",
		"The scoped effect brand handled by this cell.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The scoped-handler dispatcher value.")]
	pub trait DispatchRunRawScopedHandler<R, S, A, SBrand, FirstLayer>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
		SBrand: Kind_cdc7cd43dac7585f + 'static,
		FirstLayer: 'static, {
		/// Dispatches one raw scoped layer with its pending continuation
		/// queue still outside the layer.
		#[document_signature]
		///
		#[document_parameters(
			"The raw scoped layer carrying type-erased branch programs.",
			"The pending continuation queue for the suspended `Run`.",
			"The first-order handler list used by nested interpretation."
		)]
		#[document_returns("The next `Run` program produced by the scoped handler.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn dispatch_run_raw_scoped_head(
			&self,
			layer: Apply!(
				<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawRunFree<R, S>>
			),
			continuations: RunContinuations<R, S>,
			fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
		) -> Run<R, S, A>;
	}

	#[doc(hidden)]
	/// Internal recursive dispatcher for raw scoped `Run` layers.
	///
	/// This mirrors [`DispatchScopedHandlers`] but keeps the pending
	/// `Free` continuation queue outside the scoped layer until the
	/// active row branch is known.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type.",
		"The raw scoped row layer shape.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The scoped-handler list.")]
	pub trait DispatchRunRawScopedHandlers<R, S, A, ScopedLayer, FirstLayer>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
		ScopedLayer: 'static,
		FirstLayer: 'static, {
		/// Dispatches the active raw scoped row branch.
		#[document_signature]
		///
		#[document_parameters(
			"The raw scoped row layer.",
			"The pending continuation queue for the suspended `Run`.",
			"The first-order handler list used by nested interpretation."
		)]
		#[document_returns("The next `Run` program produced by the matching scoped handler.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(5);
		/// assert_eq!(run.extract(), 5);
		/// ```
		fn dispatch_run_raw_scoped(
			&self,
			layer: ScopedLayer,
			continuations: RunContinuations<R, S>,
			fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
		) -> Run<R, S, A>;
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The empty scoped-handler list.")]
	impl<R, S, A, FirstLayer> DispatchRunRawScopedHandlers<R, S, A, CNil, FirstLayer>
		for ScopedHandlersNil
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
		FirstLayer: 'static,
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
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(11);
		/// assert_eq!(run.extract(), 11);
		/// ```
		fn dispatch_run_raw_scoped(
			&self,
			layer: CNil,
			_continuations: RunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
		) -> Run<R, S, A> {
			match layer {}
		}
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type.",
		"The scoped effect brand at this row position.",
		"The dispatcher value type.",
		"The tail scoped-handler list type.",
		"The remaining scoped row layer shape.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The scoped-handler cons cell.")]
	impl<R, S, A, SBrand, F, T, Rest, FirstLayer>
		DispatchRunRawScopedHandlers<
			R,
			S,
			A,
			Coproduct<
				Apply!(
					<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawRunFree<R, S>>
				),
				Rest,
			>,
			FirstLayer,
		> for ScopedHandlersCons<ScopedHandler<SBrand, F>, T>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
		SBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
		F: DispatchRunRawScopedHandler<R, S, A, SBrand, FirstLayer>,
		T: DispatchRunRawScopedHandlers<R, S, A, Rest, FirstLayer>,
		Rest: 'static,
		FirstLayer: 'static,
	{
		/// Cons-cell case for raw scoped rows.
		#[document_signature]
		///
		#[document_parameters(
			"The raw scoped row layer.",
			"The pending continuation queue for the suspended `Run`.",
			"The first-order handler list used by nested interpretation."
		)]
		#[document_returns("The next `Run` program produced by the matching scoped handler.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(13);
		/// assert_eq!(run.extract(), 13);
		/// ```
		fn dispatch_run_raw_scoped(
			&self,
			layer: Coproduct<
				Apply!(
					<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawRunFree<R, S>>
				),
				Rest,
			>,
			continuations: RunContinuations<R, S>,
			fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
		) -> Run<R, S, A> {
			match layer {
				Coproduct::Inl(scoped) =>
					self.head.run.dispatch_run_raw_scoped_head(scoped, continuations, fo_handlers),
				Coproduct::Inr(rest) =>
					self.tail.dispatch_run_raw_scoped(rest, continuations, fo_handlers),
			}
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The Run instance.")]
	impl<R, S, A> Run<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Wraps a [`Free<NodeBrand<R, S>, A>`](crate::types::Free) as
		/// a `Run<R, S, A>`. Zero-cost.
		#[document_signature]
		///
		#[document_parameters("The underlying Free computation.")]
		///
		#[document_returns("A `Run` wrapping `free`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Free,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let free: Free<NodeBrand<FirstRow, Scoped>, i32> = Free::pure(7);
		/// let run: Run<FirstRow, Scoped, i32> = Run::from_free(free);
		/// assert!(matches!(run.peel(), Ok(7)));
		/// ```
		#[inline]
		pub fn from_free(free: Free<NodeBrand<R, S>, A>) -> Self {
			Run(RunRepresentation::free(free))
		}

		/// Converts a `Run<R, S, A>` to a
		/// [`Free<NodeBrand<R, S>, A>`](crate::types::Free).
		///
		/// Free-backed programs move out directly. Boundary-backed
		/// programs are lowered into an equivalent Free value whose raw
		/// scoped layer and pending continuation queue remain separate
		/// until the next step is inspected. Prefer the wrapper-level
		/// interpreters for ordinary execution; this method is the
		/// compatibility view for APIs that still consume Free.
		#[document_signature]
		///
		#[document_returns("The Free computation represented by this `Run`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Free,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> = Run::from_free(Free::pure(7));
		/// let free: Free<NodeBrand<FirstRow, Scoped>, i32> = run.into_free();
		/// assert!(matches!(free.resume(), Ok(7)));
		/// ```
		#[inline]
		pub fn into_free(self) -> Free<NodeBrand<R, S>, A> {
			self.0.into_free()
		}

		/// Wraps a value in a pure `Run` computation. Delegates to
		/// [`Free::pure`](crate::types::Free).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A `Run` computation that produces `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> = Run::pure(42);
		/// assert!(matches!(run.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self {
			Run::from_free(Free::pure(a))
		}

		/// Decomposes this `Run` computation into one public step. Returns
		/// `Ok(a)` if the program is a pure value, or `Err(layer)` if
		/// it is suspended in the dual-row
		/// [`Node`](crate::types::effects::node::Node) dispatch enum,
		/// where `layer` carries the next `Run` continuation.
		///
		/// Free-backed programs delegate to [`Free::resume`](crate::types::Free).
		/// Boundary-backed programs first lower through the Free
		/// compatibility view; that view keeps the boundary's pending
		/// continuation queue outside the scoped layer until a public
		/// branch is materialised, then attaches it to the selected
		/// branch through Free's single-shot one-step machinery.
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` for a pure result, or `Err(layer)` carrying the next `Run` step in a `Node` layer."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> = Run::pure(7);
		/// assert!(matches!(run.peel(), Ok(7)));
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "Return type encodes Result<A, NodeBrand<R, S>::Of<'static, Run<R, S, A>>>; the GAT projection is structurally complex but cannot be aliased without losing the projection link the wrapper depends on."
		)]
		pub fn peel(
			self
		) -> Result<
			A,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
		> {
			self.into_free()
				.resume()
				.map_err(|node| <NodeBrand<R, S> as Functor>::map(Run::from_free, node))
		}

		/// Decomposes this `Run` into one raw step, preserving boundary
		/// frames without going through the public Free view.
		#[document_signature]
		///
		#[document_returns(
			"The next raw step of this `Run`, with scoped-boundary continuations still outside the scoped layer."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn into_raw_step(self) -> FreeRawStep<NodeBrand<R, S>, A> {
			self.0.into_raw_step()
		}

		/// Lifts a [`Node`](crate::types::effects::node::Node) dispatch layer into the `Run` program.
		/// The `node` argument is a value of
		/// [`NodeBrand<R, S>`](crate::brands::NodeBrand)'s
		/// `Of<'static, A>` projection (typically constructed via
		/// `Node::First(<R as Member<...>>::inject(operation))` for a
		/// first-order effect, or `Node::Scoped(...)` for a scoped
		/// effect); `send` delegates to
		/// [`Free::lift_f`](crate::types::Free).
		///
		/// The `Node`-projection signature (rather than a row-variant
		/// signature) is required so the same shape works across all
		/// six Run wrappers, including
		/// [`ArcRun`](crate::types::effects::arc_run::ArcRun) and
		/// [`ArcRunExplicit`](crate::types::effects::arc_run_explicit::ArcRunExplicit).
		/// Constructing the [`Node`](crate::types::effects::node::Node) literal inside an `Arc`-substrate
		/// method body fails GAT normalization (see
		/// `tests/arc_run_normalization_probe.rs`); accepting an
		/// already-projection-typed parameter sidesteps that.
		#[document_signature]
		///
		#[document_parameters("The Node dispatch layer carrying the effect operation.")]
		///
		#[document_returns("A `Run` computation that performs the effect and returns its result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Coyoneda,
		/// 		Identity,
		/// 		effects::{
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 			run::Run,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(Identity(7));
		/// let layer = Coproduct::inject(coyo);
		/// let run: Run<FirstRow, Scoped, i32> = Run::send(Node::First(layer));
		/// // `send` produces a suspended program; peel returns Err
		/// // carrying the layer with the next continuation.
		/// assert!(run.peel().is_err());
		/// ```
		#[inline]
		pub fn send(
			node: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		) -> Self {
			Run::from_free(Free::<NodeBrand<R, S>, A>::lift_f(node))
		}

		/// Sequences this `Run` with a continuation `f`. Delegates to
		/// [`Free::bind`](crate::types::Free).
		///
		/// `f: FnOnce(A) -> Run<...>` is single-shot, mirroring
		/// [`Free`](crate::types::Free)'s `Box<dyn FnOnce>`-backed
		/// continuation queue. Handler closures stored in
		/// [`Handler<E, F>`](crate::types::effects::handlers::Handler)
		/// are bound `F: Fn` (the
		/// [`DispatchHandlers::dispatch`](crate::types::effects::interpreter::DispatchHandlers)
		/// receiver is `&self` so it can be called from inside
		/// [`MonadRec::tail_rec_m`](crate::classes::MonadRec)'s `Fn`
		/// step closure). Conversions between the two require
		/// interior mutability or refcounted captures
		/// (`Rc<RefCell<_>>`, `Arc<Mutex<_>>`).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to chain after this computation.")]
		///
		#[document_returns("A new `Run` chaining `f` after this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> =
		/// 	Run::pure(2).bind(|x| Run::pure(x + 1)).bind(|x| Run::pure(x * 10));
		/// assert!(matches!(run.peel(), Ok(30)));
		/// ```
		#[inline]
		pub fn bind<B: 'static>(
			self,
			f: impl FnOnce(A) -> Run<R, S, B> + 'static,
		) -> Run<R, S, B> {
			Run(self.0.bind(f))
		}

		/// Functor map over the result of this `Run`. Delegates to
		/// [`Free::map`](crate::types::Free).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `Run` with `f` applied to its result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> = Run::pure(7).map(|x| x * 3);
		/// assert!(matches!(run.peel(), Ok(21)));
		/// ```
		#[inline]
		pub fn map<B: 'static>(
			self,
			f: impl FnOnce(A) -> B + 'static,
		) -> Run<R, S, B> {
			Run(self.0.map(f))
		}

		/// Lifts a raw effect value into a `Run` program.
		///
		/// Direct analog of PureScript Run's
		/// [`lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs).
		/// Takes the raw effect (an `EBrand`'s `Of<'static, A>`
		/// projection), wraps it in
		/// [`Coyoneda::lift`](crate::types::Coyoneda::lift) so any
		/// effect functor satisfies the row's
		/// [`Functor`](crate::classes::Functor) requirement, injects
		/// at the
		/// [`Member`](crate::types::effects::member::Member)-determined
		/// position, wraps in [`Node::First`](crate::types::effects::node::Node),
		/// and lifts via [`send`](Run::send). Per-effect smart
		/// constructors (`ask`, `get`, `put`, `tell`, `throw`) are
		/// one-liners over this combinator, mirroring PureScript
		/// Run's `liftEffect = lift (Proxy :: "effect")` pattern.
		///
		/// Naming note: PureScript Run distinguishes
		/// [`Run.lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// (the row-aware Run-level operation, which this method
		/// implements) from
		/// [`Free.liftF`](https://github.com/purescript/purescript-free/blob/main/src/Control/Monad/Free.purs)
		/// (the Free-monad-only lift, which fp-library exposes as
		/// [`Free::lift_f`](crate::types::Free::lift_f)). The bare
		/// name `lift` matches the row-aware operation; the `_f`
		/// suffix is reserved for the Free-only operation.
		///
		/// `Idx` is the type-level position witness. For an
		/// unambiguous row (each effect type appears once), Rust
		/// infers it; turbofish only when duplicate effect types make
		/// the position ambiguous.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect being lifted.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The effect value to lift.")]
		///
		#[document_returns("A `Run` program suspended at the lifted effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::{
		/// 			Run,
		/// 			RunFirstOrderHandler,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// // The program is suspended at the lifted effect; peel reveals the layer.
		/// assert!(run.peel().is_err());
		/// ```
		#[inline]
		pub fn lift<EBrand, Idx>(
			effect: Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<Coyoneda<'static, EBrand, A>, Idx>,
			EBrand: Kind_cdc7cd43dac7585f + 'static, {
			let coyo: Coyoneda<'static, EBrand, A> = Coyoneda::lift(effect);
			let layer =
				<Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>) as Member<
					Coyoneda<'static, EBrand, A>,
					Idx,
				>>::inject(coyo);
			Self::send(Node::First(layer))
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The Run instance.")]
	impl<R, S, A> Run<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Interprets this `Run` program by walking each effect via the
		/// matching handler closure in `handlers`, looping until the
		/// program reduces to a [`Pure`](crate::types::Free) value.
		///
		/// `handlers` is a handler list (typically built via the
		/// [`handlers!`](https://docs.rs/fp-macros/latest/fp_macros/macro.handlers.html)
		/// macro or the
		/// [`nt()`](crate::types::effects::handlers::nt) builder
		/// fallback) whose cells align cell-for-cell with the row
		/// brand chain `R`. Each cell carries a closure
		/// [`Handler<EBrand, F>`](crate::types::effects::handlers::Handler)
		/// of shape `FnMut(<EBrand as Kind>::Of<'_, Run<R, S, A>>) -> Run<R, S, A>`,
		/// taking the lowered [`Coyoneda`](crate::types::Coyoneda)
		/// payload and returning the next-step program in the same
		/// row.
		///
		/// Mirrors PureScript Run's
		/// [`interpret`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// (which is itself a literal alias for
		/// [`run`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)).
		/// The Rust port adopts a mono-in-`A` step-function shape so
		/// handler closures don't need rank-2 polymorphism (which
		/// Rust closures can't express). The scoped row `S` is fixed
		/// at [`CNilBrand`](crate::brands::CNilBrand) for the
		/// first-order interpreter; future scoped-effect work extends
		/// this to dispatch over scoped effects too.
		///
		/// ## Stack safety
		///
		/// This method recurses host-stack-frame per peeled layer.
		/// [`interpret_rec`](Run::interpret_rec) (and siblings) provide
		/// stack-safe interpretation via `MonadRec`.
		///
		/// ## Threading state through handlers
		///
		/// Handlers can capture an
		/// [`Rc<RefCell<S>>`](std::rc::Rc) (or
		/// [`Arc<Mutex<S>>`](std::sync::Arc) on thread-safe wrappers)
		/// to thread state through dispatch. Each `interpret` call only
		/// returns the program's `A`; the captured cell holds the
		/// final state for the caller to read after interpretation
		/// completes. The keyword "state" appears here for
		/// rustdoc-search discoverability of stateful interpretation.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list (typically built via the `handlers!` macro).",
			"The scoped-effect handler list."
		)]
		///
		#[document_returns("The final result value of the program.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			handlers::*,
		/// 			run::Run,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// // Lift an Identity effect, then interpret it: the handler
		/// // unwraps the `Identity` and returns a pure program.
		/// let prog: Run<FirstRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let result = prog.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Run<FirstRow, Scoped, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 42);
		/// ```
		///
		/// State threading via a captured cell:
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		handlers,
		/// 		types::{
		/// 			Identity,
		/// 			effects::{
		/// 				handlers::*,
		/// 				run::Run,
		/// 			},
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::RefCell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let counter: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
		/// let counter_for_handler = Rc::clone(&counter);
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(7));
		/// let result = prog.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: move |op: Identity<Run<FirstRow, Scoped, i32>>| {
		/// 			*counter_for_handler.borrow_mut() += 1;
		/// 			op.0
		/// 		},
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 7);
		/// assert_eq!(*counter.borrow(), 1);
		/// ```
		#[inline]
		pub fn interpret(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, S, A>>),
				Run<R, S, A>,
			>,
			scoped_handlers: impl DispatchRunRawScopedHandlers<
				R,
				S,
				A,
				Apply!(
					<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawRunFree<R, S>>
				),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			>,
		) -> A {
			let mut prog = self;
			loop {
				match prog.into_raw_step() {
					FreeRawStep::Done(a) => return a,
					FreeRawStep::Suspended {
						layer,
						continuations,
					} => match layer {
						Node::First(layer) => {
							let remaining = std::cell::Cell::new(Some(continuations));
							let mapped = <R as Functor>::map(
								move |inner: RawRunFree<R, S>| {
									#[expect(
										clippy::expect_used,
										reason = "Raw first-order dispatch attaches a single-shot continuation to exactly one active row branch"
									)]
									let continuations = remaining.take().expect(
										"Run raw first-order continuation attached more than once",
									);
									Run::from_free(Free::continue_from_erased(inner, continuations))
								},
								layer,
							);
							prog = handlers.dispatch(mapped);
						}
						Node::Scoped(layer) => {
							prog = scoped_handlers.dispatch_run_raw_scoped(
								layer,
								continuations,
								&handlers,
							);
						}
					},
				}
			}
		}

		/// Alias for [`interpret`](Run::interpret), kept for naming
		/// parity with PureScript Run's
		/// [`run`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs).
		///
		/// In PureScript Run,
		/// `interpret :: (VariantF r ~> m) -> Run r a -> m a` carries
		/// a rank-2 natural-transformation signature while
		/// `run :: (VariantF r (Run r a) -> m (Run r a)) -> Run r a -> m a`
		/// carries the mono-in-`a` step-function form. The two
		/// implementations are literally aliased
		/// (`interpret = run` at
		/// [`Run.purs:184`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)).
		/// The Rust port has only the mono-in-`a` form (closures
		/// cannot be A-polymorphic); both names are exposed for
		/// PureScript-cross-reference convenience.
		#[document_signature]
		///
		#[document_parameters("The first-order handler list.", "The scoped-effect handler list.")]
		///
		#[document_returns("The final result value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			handlers::*,
		/// 			run::Run,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(99));
		/// let result = prog.run(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Run<FirstRow, Scoped, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 99);
		/// ```
		#[inline]
		pub fn run(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, S, A>>),
				Run<R, S, A>,
			>,
			scoped_handlers: impl DispatchRunRawScopedHandlers<
				R,
				S,
				A,
				Apply!(
					<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawRunFree<R, S>>
				),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			>,
		) -> A {
			self.interpret(handlers, scoped_handlers)
		}

		/// MonadRec-target interpreter: walk this `Run` program against
		/// a handler list, producing the result in an external monad
		/// `MBrand` rather than as a raw `A`. Stack-safe via
		/// [`tail_rec_m`](crate::functions::tail_rec_m), so unbounded
		/// chains over `MBrand: MonadRec` substrates such as
		/// [`ThunkBrand`](crate::brands::ThunkBrand),
		/// [`OptionBrand`](crate::brands::OptionBrand), or
		/// [`ResultBrand`](crate::brands::ResultBrand) do not blow the
		/// host stack.
		///
		/// Direct analog of PureScript Run's
		/// [`interpretRec`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// (which is itself a literal alias for
		/// [`runRec`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// over a `MonadRec m` constraint).
		///
		/// ## Handler shape
		///
		/// Each handler closure has shape
		/// `Fn(<EBrand as Kind>::Of<'_, M::Of<'_, Run<R, S, A>>>) -> M::Of<'_, Run<R, S, A>>`,
		/// matching PureScript Run's
		/// `(VariantF r (m (Run r a)) -> m (Run r a))`. Inner
		/// continuations arrive already lifted into the target monad
		/// (the interpreter does
		/// [`Functor::map`](crate::classes::Functor)`(M::pure, layer)`
		/// before dispatch), so handlers can sequence M-shaped work via
		/// M's bind/map directly.
		///
		/// ## Loop body
		///
		/// On each iteration the step closure peels the program; on
		/// `Ok(a)` it emits `M::pure(ControlFlow::Break(a))`; on
		/// `Err(Node::First(layer))` it lifts the layer's inner
		/// programs to M-wrapped, dispatches through the handler list,
		/// and fmaps `M::Of<Run<R, S, A>>` to
		/// `M::Of<ControlFlow<A, Run<R, S, A>>>` via
		/// [`ControlFlow::Continue`].
		#[document_signature]
		///
		#[document_type_parameters("The brand of the target monad (must implement [`MonadRec`]).")]
		///
		#[document_parameters(
			"The first-order handler list (typically built via the `handlers!` macro).",
			"The scoped-effect handler list."
		)]
		///
		#[document_returns("The program result wrapped in the target monad `MBrand`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		Thunk,
		/// 		effects::{
		/// 			handlers::*,
		/// 			run::Run,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let result: Thunk<'static, i32> = prog.interpret_rec::<ThunkBrand>(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Thunk<'static, Run<FirstRow, Scoped, i32>>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result.evaluate(), 42);
		/// ```
		#[inline]
		pub fn interpret_rec<MBrand>(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			> + 'static,
			scoped_handlers: impl DispatchScopedHandlers<
				'static,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			> + 'static,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		where
			MBrand: MonadRec + 'static, {
			tail_rec_m::<MBrand, Run<R, S, A>, A>(
				move |prog: Run<R, S, A>| match prog.peel() {
					Ok(a) => <MBrand as Pointed>::pure::<ControlFlow<A, Run<R, S, A>>>(
						ControlFlow::Break(a),
					),
					Err(Node::First(layer)) => {
						let mapped = <R as Functor>::map(
							|inner: Run<R, S, A>| <MBrand as Pointed>::pure::<Run<R, S, A>>(inner),
							layer,
						);
						let next = handlers.dispatch(mapped);
						<MBrand as Functor>::map::<Run<R, S, A>, ControlFlow<A, Run<R, S, A>>>(
							ControlFlow::Continue,
							next,
						)
					}
					Err(Node::Scoped(layer)) => {
						let mapped = <S as Functor>::map(
							|inner: Run<R, S, A>| <MBrand as Pointed>::pure::<Run<R, S, A>>(inner),
							layer,
						);
						let next = scoped_handlers.dispatch_scoped(mapped, &handlers);
						<MBrand as Functor>::map::<Run<R, S, A>, ControlFlow<A, Run<R, S, A>>>(
							ControlFlow::Continue,
							next,
						)
					}
				},
				self,
			)
		}

		/// Alias for [`interpret_rec`](Run::interpret_rec), kept for
		/// naming parity with PureScript Run's
		/// [`runRec`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// (which is literally aliased to
		/// [`interpretRec`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// in PureScript Run; both names are exposed here for
		/// cross-reference convenience).
		#[document_signature]
		///
		#[document_type_parameters("The brand of the target monad (must implement [`MonadRec`]).")]
		///
		#[document_parameters("The first-order handler list.", "The scoped-effect handler list.")]
		///
		#[document_returns("The program result wrapped in the target monad `MBrand`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		Thunk,
		/// 		effects::{
		/// 			handlers::*,
		/// 			run::Run,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(99));
		/// let result: Thunk<'static, i32> = prog.run_rec::<ThunkBrand>(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Thunk<'static, Run<FirstRow, Scoped, i32>>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result.evaluate(), 99);
		/// ```
		#[inline]
		pub fn run_rec<MBrand>(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			> + 'static,
			scoped_handlers: impl DispatchScopedHandlers<
				'static,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			> + 'static,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		where
			MBrand: MonadRec + 'static, {
			self.interpret_rec::<MBrand>(handlers, scoped_handlers)
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The Run instance.")]
	impl<R, S, A> Run<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Scoped-row-narrowing interpreter: interpret a single scoped
		/// effect `SBrand` out of the scoped row, returning a `Run`
		/// program in the narrowed scoped row `SMinusE`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the scoped effect being interpreted out of the row.",
			"The type-level position witness (typically inferred).",
			"The narrowed scoped row brand."
		)]
		///
		#[document_parameters("The handler closure for the targeted scoped effect.")]
		///
		#[document_returns("A `Run` program in the narrowed scoped row `SMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		span::BoxSpan,
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
		///
		/// let action: Run<CNilBrand, ScopedRow, i32> = Run::pure(7);
		/// let prog: Run<CNilBrand, ScopedRow, i32> = Run::span::<&'static str, _>("request", action);
		/// let narrowed: Run<CNilBrand, CNilBrand, i32> = prog
		/// 	.interpret_scoped_with::<BoxSpanBrand<BoxBrand, &'static str>, _, CNilBrand>(|span| {
		/// 		match span {
		/// 			BoxSpan::Span {
		/// 				tag,
		/// 				action,
		/// 			} => {
		/// 				assert_eq!(tag, "request");
		/// 				action(())
		/// 			}
		/// 		}
		/// 	});
		/// assert_eq!(narrowed.extract(), 7);
		/// ```
		#[inline]
		pub fn interpret_scoped_with<SBrand, Idx, SMinusE>(
			self,
			handler: impl Fn(
				Apply!(<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, SMinusE, A>>),
			) -> Run<R, SMinusE, A>
			+ 'static,
		) -> Run<R, SMinusE, A>
		where
			SBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
					),
					Idx,
					Remainder = Apply!(
									<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>, {
			let handler = <RcBrand as RefCountedPointer>::new(handler);
			self.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, _>(handler)
		}

		#[inline]
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the scoped effect being interpreted out of the row.",
			"The type-level position witness.",
			"The narrowed scoped row brand.",
			"The concrete handler closure type."
		)]
		///
		#[document_parameters("The handler wrapped in a refcounted pointer.")]
		///
		#[document_returns("A `Run` program in the narrowed scoped row `SMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		span::BoxSpan,
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
		///
		/// // Exercised internally by Run::interpret_scoped_with.
		/// let action: Run<CNilBrand, ScopedRow, i32> = Run::pure(7);
		/// let prog: Run<CNilBrand, ScopedRow, i32> = Run::span::<&'static str, _>("request", action);
		/// let narrowed: Run<CNilBrand, CNilBrand, i32> = prog
		/// 	.interpret_scoped_with::<BoxSpanBrand<BoxBrand, &'static str>, _, CNilBrand>(|span| {
		/// 		match span {
		/// 			BoxSpan::Span {
		/// 				tag,
		/// 				action,
		/// 			} => {
		/// 				assert_eq!(tag, "request");
		/// 				action(())
		/// 			}
		/// 		}
		/// 	});
		/// assert_eq!(narrowed.extract(), 7);
		/// ```
		fn interpret_scoped_with_shared<SBrand, Idx, SMinusE, F>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'static, F>,
		) -> Run<R, SMinusE, A>
		where
			F: Fn(
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							Run<R, SMinusE, A>,
						>
					),
				) -> Run<R, SMinusE, A>
				+ 'static,
			SBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
					),
					Idx,
					Remainder = Apply!(
									<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>, {
			match self.peel() {
				Ok(a) => Run::pure(a),
				Err(Node::First(layer)) => {
					let h_for_recurse = handler.clone();
					let mapped_free = <R as Functor>::map(
						move |inner: Run<R, S, A>| {
							inner
								.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
									h_for_recurse.clone(),
								)
								.into_free()
						},
						layer,
					);
					Run::from_free(Free::<NodeBrand<R, SMinusE>, A>::wrap(Node::First(mapped_free)))
				}
				Err(Node::Scoped(layer)) =>
					match <Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Run<R, S, A>,
					>) as Member<
						Apply!(
							<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
						),
						Idx,
					>>::project(layer)
					{
						Ok(scoped) => {
							let h_for_recurse = handler.clone();
							let mapped = <SBrand as Functor>::map(
								move |inner: Run<R, S, A>| {
									inner.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
										h_for_recurse.clone(),
									)
								},
								scoped,
							);
							(*handler)(mapped)
						}
						Err(rest) => {
							let h_for_recurse = handler.clone();
							let mapped_free = <SMinusE as Functor>::map(
								move |inner: Run<R, S, A>| {
									inner
										.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
											h_for_recurse.clone(),
										)
										.into_free()
								},
								rest,
							);
							Run::from_free(Free::<NodeBrand<R, SMinusE>, A>::wrap(Node::Scoped(
								mapped_free,
							)))
						}
					},
			}
		}

		/// Pipeline row-narrowing interpreter: interpret a single
		/// effect `EBrand` out of the row, returning a `Run` program in
		/// the narrowed row `RMinusE` (with `EBrand` removed).
		///
		/// Mirrors PureScript Run's
		/// [`interpret`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// (the row-narrowing form, not the all-handlers-at-once form
		/// shipped on this wrapper as [`Run::interpret`]) and heftia's
		/// `interpret_with` primary mode. Three capabilities this form
		/// uniquely enables: partial interpretation (interpret one
		/// effect, store the result, interpret the rest later);
		/// user-controlled handler ordering for non-commuting effects
		/// (e.g., `NonDet x Except`); compositional handler libraries
		/// (`fn run_state<R, A>(...) -> Run<R_minus_State, S, A>`).
		///
		/// `handler` consumes the lowered effect (`<EBrand as Kind>::Of<'_, Run<RMinusE, S, A>>`,
		/// where each inner program is already narrowed) and produces
		/// the next-step program in the narrowed row. The handler is
		/// wrapped in an [`Rc`](std::rc::Rc) once at entry; recursive
		/// calls clone the [`Rc`](std::rc::Rc) (refcount bump) instead
		/// of cloning the underlying closure, so the user-facing bound
		/// is just `Fn + 'static` (no `Clone`). This permits handlers
		/// that capture move-only resources (e.g., a `BufWriter`).
		///
		/// ## Scoped rows
		///
		/// Scoped layers are rewritten through the scoped row's
		/// [`Functor`](crate::classes::Functor) implementation. This
		/// preserves scoped cells whose functor maps their stored action
		/// program, such as Span, Catch, Local, and RefLocal. It is not
		/// the continuation-aware raw scoped dispatcher path used to run
		/// scoped handlers. Scoped cells whose functor intentionally
		/// leaves the cell unchanged, such as
		/// [`BoxBracketBrand`](crate::brands::BoxBracketBrand), are not
		/// traversed by this API; their acquire/body/release programs
		/// are run by the Bracket dispatcher instead.
		///
		/// ## Stack safety
		///
		/// This method recurses host-stack-frame per peeled layer in
		/// the original program (the recursion is via [`Functor::map`]
		/// on each layer's continuation). For Identity-shaped effects,
		/// the recursion is eager; for closure-shaped effects (e.g.,
		/// `State<S>`), the recursion is deferred until the closure is
		/// invoked. Programs with deep chains of eager-recursing effects
		/// can blow the host stack.
		///
		/// For M-target stack safety on the dispatched effects' M-bind
		/// chains (e.g., long `State` Get/Put chains), chain
		/// [`interpret_with`](Run::interpret_with) (narrow effects one
		/// at a time) followed by [`interpret_rec`](Run::interpret_rec)
		/// at the end of the pipeline. There is no combined
		/// `interpret_with_rec` primitive: the unmatched-arm step needs
		/// to swap `RMinusE::Of<M::Of<...>>` to `M::Of<RMinusE::Of<...>>`,
		/// which requires
		/// [`Traversable`](crate::classes::Traversable) on every row
		/// brand plus
		/// [`Applicative`](crate::classes::Applicative) on `M`; non-rec
		/// `interpret_with` sidesteps this by using just
		/// [`Functor::map`](crate::classes::Functor::map) (no `M` to
		/// swap with). PureScript Run does not provide the combination
		/// either.
		///
		/// ## Type inference
		///
		/// Turbofish `EBrand` only; `Idx` is inferred via
		/// [`Member`](crate::types::effects::member::Member); `F` from
		/// the handler argument; `RMinusE` from the handler's return
		/// type.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect being interpreted out of the row.",
			"The type-level position witness (typically inferred).",
			"The narrowed row brand (the original row with `EBrand` removed; typically inferred from the handler's return type)."
		)]
		///
		#[document_parameters("The handler closure for the targeted effect.")]
		///
		#[document_returns("A `Run` program in the narrowed row `RMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::{
		/// 			Run,
		/// 			RunFirstOrderHandler,
		/// 		},
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// struct IdentityHandler;
		///
		/// impl RunFirstOrderHandler<IdentityBrand, EmptyRow, CNilBrand> for IdentityHandler {
		/// 	fn handle<T: 'static>(
		/// 		&self,
		/// 		effect: Identity<Run<EmptyRow, CNilBrand, T>>,
		/// 	) -> Run<EmptyRow, CNilBrand, T> {
		/// 		effect.0
		/// 	}
		/// }
		///
		/// // Lift an Identity effect, then narrow it out with a
		/// // result-polymorphic handler.
		/// let prog: Run<FullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: Run<EmptyRow, CNilBrand, i32> =
		/// 	prog.interpret_with_handler::<IdentityBrand, _, EmptyRow>(IdentityHandler);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		pub fn interpret_with_handler<EBrand, Idx, RMinusE>(
			self,
			handler: impl RunFirstOrderHandler<EBrand, RMinusE, S> + 'static,
		) -> Run<RMinusE, S, A>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>, {
			let handler = <RcBrand as RefCountedPointer>::new(handler);
			self.interpret_with_handler_shared::<EBrand, Idx, RMinusE, _>(handler)
		}

		/// Row-narrowing implementation backed by a
		/// result-polymorphic first-order handler.
		///
		/// The handler is called through [`RunFirstOrderHandler`] so each
		/// recursive branch can use its own result type.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect being interpreted out of the row.",
			"The type-level position witness.",
			"The narrowed row brand.",
			"The concrete result-polymorphic handler type."
		)]
		#[document_parameters("The handler wrapped in a refcounted pointer.")]
		#[document_returns("A `Run` program in the narrowed row `RMinusE`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::{
		/// 			Run,
		/// 			RunFirstOrderHandler,
		/// 		},
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// struct IdentityHandler;
		///
		/// impl RunFirstOrderHandler<IdentityBrand, EmptyRow, CNilBrand> for IdentityHandler {
		/// 	fn handle<T: 'static>(
		/// 		&self,
		/// 		effect: Identity<Run<EmptyRow, CNilBrand, T>>,
		/// 	) -> Run<EmptyRow, CNilBrand, T> {
		/// 		effect.0
		/// 	}
		/// }
		///
		/// let prog: Run<FullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: Run<EmptyRow, CNilBrand, i32> =
		/// 	prog.interpret_with_handler::<IdentityBrand, _, EmptyRow>(IdentityHandler);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		fn interpret_with_handler_shared<EBrand, Idx, RMinusE, H>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'static, H>,
		) -> Run<RMinusE, S, A>
		where
			H: RunFirstOrderHandler<EBrand, RMinusE, S> + 'static,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>, {
			match self.peel() {
				Ok(a) => Run::pure(a),
				Err(Node::First(layer)) =>
					match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
					) as Member<Coyoneda<'static, EBrand, Run<R, S, A>>, Idx>>::project(
						layer
					) {
						Ok(coyo) => {
							let lowered = coyo.lower();
							let h_for_recurse = handler.clone();
							let mapped = <EBrand as Functor>::map(
								move |inner: Run<R, S, A>| {
									inner.interpret_with_handler_shared::<EBrand, Idx, RMinusE, H>(
										h_for_recurse.clone(),
									)
								},
								lowered,
							);
							(*handler).handle(mapped)
						}
						Err(rest) => {
							let h_for_recurse = handler.clone();
							let mapped_free = <RMinusE as Functor>::map(
								move |inner: Run<R, S, A>| {
									inner
										.interpret_with_handler_shared::<EBrand, Idx, RMinusE, H>(
											h_for_recurse.clone(),
										)
										.into_free()
								},
								rest,
							);
							Run::from_free(Free::<NodeBrand<RMinusE, S>, A>::wrap(Node::First(
								mapped_free,
							)))
						}
					},
				Err(Node::Scoped(layer)) => {
					let h_for_recurse = handler.clone();
					let mapped_free = <S as Functor>::map(
						move |inner: Run<R, S, A>| {
							inner
								.interpret_with_handler_shared::<EBrand, Idx, RMinusE, H>(
									h_for_recurse.clone(),
								)
								.into_free()
						},
						layer,
					);
					Run::from_free(Free::<NodeBrand<RMinusE, S>, A>::wrap(Node::Scoped(
						mapped_free,
					)))
				}
			}
		}

		/// Inner pipeline-narrowing implementation, parameterised
		/// over the concrete handler closure type `F`. The public
		/// [`interpret_with`](Run::interpret_with) wraps the user
		/// handler in [`Rc<F>`](std::rc::Rc) once at entry and
		/// delegates here; recursive narrowing clones the
		/// [`Rc<F>`](std::rc::Rc) (refcount bump) instead of
		/// cloning the underlying closure, which is what drops the
		/// `Clone` bound from the user-facing API.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect being interpreted out of the row.",
			"The type-level position witness.",
			"The narrowed row brand.",
			"The concrete handler closure type."
		)]
		///
		#[document_parameters("The handler wrapped in a refcounted pointer.")]
		///
		#[document_returns("A `Run` program in the narrowed row `RMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// // Exercised internally by Run::interpret_with.
		/// let prog: Run<FullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: Run<EmptyRow, CNilBrand, i32> = prog
		/// 	.interpret_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<Run<EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		fn interpret_with_shared<EBrand, Idx, RMinusE, F>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'static, F>,
		) -> Run<RMinusE, S, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<RMinusE, S, A>>),
				) -> Run<RMinusE, S, A>
				+ 'static,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>, {
			match self.peel() {
				Ok(a) => Run::pure(a),
				Err(Node::First(layer)) =>
					match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
					) as Member<Coyoneda<'static, EBrand, Run<R, S, A>>, Idx>>::project(
						layer
					) {
						Ok(coyo) => {
							let lowered = coyo.lower();
							let h_for_recurse = handler.clone();
							let mapped = <EBrand as Functor>::map(
								move |inner: Run<R, S, A>| {
									inner.interpret_with_shared::<EBrand, Idx, RMinusE, F>(
										h_for_recurse.clone(),
									)
								},
								lowered,
							);
							(*handler)(mapped)
						}
						Err(rest) => {
							let h_for_recurse = handler.clone();
							let mapped_free = <RMinusE as Functor>::map(
								move |inner: Run<R, S, A>| {
									inner
										.interpret_with_shared::<EBrand, Idx, RMinusE, F>(
											h_for_recurse.clone(),
										)
										.into_free()
								},
								rest,
							);
							Run::from_free(Free::<NodeBrand<RMinusE, S>, A>::wrap(Node::First(
								mapped_free,
							)))
						}
					},
				Err(Node::Scoped(layer)) => {
					let h_for_recurse = handler.clone();
					let mapped_free = <S as Functor>::map(
						move |inner: Run<R, S, A>| {
							inner
								.interpret_with_shared::<EBrand, Idx, RMinusE, F>(
									h_for_recurse.clone(),
								)
								.into_free()
						},
						layer,
					);
					Run::from_free(Free::<NodeBrand<RMinusE, S>, A>::wrap(Node::Scoped(
						mapped_free,
					)))
				}
			}
		}

		/// Substrate-level row-preserving replacement primitive: walk
		/// this `Run` program, projecting each first-order dispatch
		/// against `EBrand`; replace every matched dispatch with the
		/// supplied `replacement` closure (applied to the lowered
		/// effect value), and re-emit non-matching dispatches in the
		/// same row. Direct analog of heftia's `interposeInWith` in
		/// substrate-primitive form.
		///
		/// Unlike [`interpret_with`](Run::interpret_with), `interpose`
		/// does not narrow the row: the matched arm produces a
		/// continuation in the same `R`, the unmatched arm walks the
		/// `Self::Remainder` (RMinusE) layer and embeds it back into
		/// `R` via [`CoproductEmbedder`](crate::types::effects::coproduct::CoproductEmbedder).
		/// This is the building block for scoped-effect handlers
		/// (e.g., `Catch`'s recovery path interposes against the body
		/// program's `Throw` dispatches without narrowing the row).
		///
		/// The user-facing closure is wrapped in an
		/// [`Rc`](std::rc::Rc) once at entry; recursive calls clone
		/// the [`Rc`](std::rc::Rc) (refcount bump) instead of cloning
		/// the underlying closure, which is what drops the `Clone`
		/// bound from the user-facing API.
		///
		/// Like [`Run::interpret_with`], scoped layers are rewritten only
		/// through the scoped row's
		/// [`Functor`](crate::classes::Functor) implementation.
		/// This preserves scoped cells whose functor maps their stored
		/// action program. It does not run the continuation-aware raw
		/// scoped dispatcher protocol, and it does not traverse scoped
		/// cells such as
		/// [`BoxBracketBrand`](crate::brands::BoxBracketBrand) whose
		/// functor deliberately keeps acquire/body/release fixed.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect to replace.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand (the row with `EBrand` removed at position `Idx`).",
			"The HList witness for embedding the narrowed row back into the original row."
		)]
		///
		#[document_parameters(
			"The replacement applied to each matched-effect dispatch's lowered effect value."
		)]
		///
		#[document_returns(
			"A new program in the same row with all matched-effect dispatches replaced."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = Run<Row, CNilBrand, i32>;
		///
		/// let prog: Prog = Run::lift::<IdentityBrand, _>(Identity(7));
		/// // Replacement substitutes each matched dispatch with a fresh
		/// // program; here we return `pure(99)` for the Identity dispatch,
		/// // demonstrating that the matched arm fires.
		/// let interposed =
		/// 	prog.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| Run::pure(99));
		/// let result = interposed.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Prog>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 99);
		/// ```
		pub fn interpose<EBrand, Idx, RMinusE, EmbedIndices>(
			self,
			replacement: impl Fn(
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			) -> Run<R, S, A>
			+ 'static,
		) -> Run<R, S, A>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Free<NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			let replacement = <RcBrand as RefCountedPointer>::new(replacement);
			self.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, _>(replacement)
		}

		/// Inner shared implementation of [`interpose`](Run::interpose),
		/// parameterised over the concrete replacement closure type
		/// `F`. The public [`interpose`](Run::interpose) wraps the
		/// user-supplied closure in [`Rc<F>`](std::rc::Rc) once at
		/// entry and delegates here; recursive descent clones the
		/// [`Rc<F>`](std::rc::Rc) (refcount bump) instead of cloning
		/// the underlying closure, which is what drops the `Clone`
		/// bound from the user-facing API.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect to replace.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand (the row with `EBrand` removed at position `Idx`).",
			"The HList witness for embedding the narrowed row back into the original row.",
			"The concrete replacement closure type."
		)]
		///
		#[document_parameters("The Rc-wrapped replacement closure.")]
		///
		#[document_returns(
			"A new program in the same row with all matched-effect dispatches replaced."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// // Exercised internally by Run::interpose.
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = Run<Row, CNilBrand, i32>;
		///
		/// let prog: Prog = Run::lift::<IdentityBrand, _>(Identity(3));
		/// let interposed =
		/// 	prog.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| Run::pure(42));
		/// let result = interposed.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Prog>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 42);
		/// ```
		fn interpose_shared<EBrand, Idx, RMinusE, EmbedIndices, F>(
			self,
			replacement: <RcBrand as RefCountedPointer>::Of<'static, F>,
		) -> Run<R, S, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				) -> Run<R, S, A>
				+ 'static,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Free<NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => Run::pure(a),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
				) as Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let lowered = coyo.lower();
						let r_for_recurse = replacement.clone();
						let mapped = <EBrand as Functor>::map(
							move |inner: Run<R, S, A>| {
								inner.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
									r_for_recurse.clone(),
								)
							},
							lowered,
						);
						(*replacement)(mapped)
					}
					Err(rest) => {
						let r_for_recurse = replacement.clone();
						let mapped_rest = <RMinusE as Functor>::map(
							move |inner: Run<R, S, A>| {
								inner
									.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
										r_for_recurse.clone(),
									)
									.into_free()
							},
							rest,
						);
						let layer_back = mapped_rest.embed();
						Run::from_free(Free::<NodeBrand<R, S>, A>::wrap(Node::First(layer_back)))
					}
				},
				Err(Node::Scoped(layer)) => {
					let r_for_recurse = replacement.clone();
					let mapped_free = <S as Functor>::map(
						move |inner: Run<R, S, A>| {
							inner
								.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
									r_for_recurse.clone(),
								)
								.into_free()
						},
						layer,
					);
					Run::from_free(Free::<NodeBrand<R, S>, A>::wrap(Node::Scoped(mapped_free)))
				}
			}
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The Run instance.")]
	impl<R, A> Run<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Interprets one first-order effect out of a first-order-only
		/// `Run` program with a final-result-specific closure.
		///
		/// This convenience method is available only when the scoped row
		/// is [`CNilBrand`]. Programs with scoped rows should use
		/// [`Run::interpret_with_handler`], whose handler protocol is
		/// result-polymorphic and can safely narrow selected scoped
		/// action/recovery programs whose branch result differs from the
		/// final outer result.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect being interpreted out of the row.",
			"The type-level position witness.",
			"The narrowed row brand."
		)]
		#[document_parameters("The final-result-specific handler closure.")]
		#[document_returns("A first-order-only `Run` program in the narrowed row `RMinusE`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// let prog: Run<FullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: Run<EmptyRow, CNilBrand, i32> = prog
		/// 	.interpret_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<Run<EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		pub fn interpret_with<EBrand, Idx, RMinusE>(
			self,
			handler: impl Fn(
				Apply!(
					<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Run<RMinusE, CNilBrand, A>,
					>
				),
			) -> Run<RMinusE, CNilBrand, A>
			+ 'static,
		) -> Run<RMinusE, CNilBrand, A>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
					Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, A>,
									>
								),
				>, {
			let handler = <RcBrand as RefCountedPointer>::new(handler);
			self.interpret_with_shared::<EBrand, Idx, RMinusE, _>(handler)
		}

		/// Substrate-level matched-effect short-circuit primitive on
		/// the default Erased Run wrapper: walk this `Run` program,
		/// dispatching non-matched first-order effects through
		/// `fo_handlers` and short-circuiting the moment a
		/// matched-effect (`EBrand`) dispatch is encountered, returning
		/// the matched effect's lowered payload. Direct analog of
		/// heftia's `interpretWithEither` substrate primitive.
		///
		/// Returns `Ok(a)` when the program reduces to a pure value
		/// without firing the matched effect; returns `Err(op)` with
		/// the matched effect's lowered payload (`<EBrand as Kind>::Of<'static, Self>`)
		/// the moment the matched effect is dispatched. Pairs with
		/// scoped `Catch` handlers.
		///
		/// `fo_handlers` covers only the non-matched effects (the
		/// `RMinusE` row); the matched effect short-circuits without
		/// any handler invocation. The program type retains the full
		/// row `R` because the matched effect's dispatches throughout
		/// the program tree are discharged uniformly by the
		/// short-circuit, not by row narrowing.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the matched effect.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand (the row with `EBrand` removed at position `Idx`)."
		)]
		///
		#[document_parameters("The handler list covering non-matched first-order effects.")]
		///
		#[document_returns(
			"`Ok(a)` if the program completes without firing the matched effect; `Err(op)` carrying the matched effect's lowered payload otherwise."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			except::Except,
		/// 			run::Run,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<
		/// 	CoyonedaBrand<ExceptBrand<String>>,
		/// 	CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>,
		/// >;
		/// type RowMinusExcept = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = Run<Row, CNilBrand, i32>;
		///
		/// let prog: Prog = Run::throw::<String, _>("oops".to_string());
		/// let result: Result<i32, Except<'_, String, Prog>> = prog
		/// 	.interpret_with_either::<ExceptBrand<String>, _, RowMinusExcept>(handlers! {
		/// 		IdentityBrand: |op: Identity<Prog>| op.0,
		/// 	});
		/// match result {
		/// 	Ok(_) => panic!("expected throw"),
		/// 	Err(Except::Throw(e, _)) => assert_eq!(e, "oops"),
		/// }
		/// ```
		#[inline]
		pub fn interpret_with_either<EBrand, Idx, RMinusE>(
			self,
			fo_handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, CNilBrand, A>>),
				Run<R, CNilBrand, A>,
			>,
		) -> Result<
			A,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>),
		>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>):
				Member<
						Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>
									),
					>, {
			let mut prog = self;
			loop {
				match prog.peel() {
					Ok(a) => return Ok(a),
					Err(Node::First(layer)) =>
						match <Apply!(
							<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>
						) as Member<Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>, Idx>>::project(
							layer
						) {
							Ok(matched_coyo) => return Err(matched_coyo.lower()),
							Err(rest) => prog = fo_handlers.dispatch(rest),
						},
					Err(Node::Scoped(cnil)) => match cnil {},
				}
			}
		}
	}

	#[document_type_parameters("The result type.")]
	#[document_parameters("The Run instance.")]
	impl<A> Run<CNilBrand, CNilBrand, A>
	where
		A: 'static,
	{
		/// Extracts the result value from a `Run` program whose first-order
		/// and scoped rows have both been fully interpreted away (both
		/// resolve to [`CNilBrand`]). Both rows' `Of<...>` projections are
		/// [`CNil`](crate::types::effects::coproduct::CNil) (uninhabited),
		/// so each `Node` arm is structurally impossible and matched
		/// exhaustively; the body diverges to type `!`, statically
		/// proving no runtime panic.
		///
		/// Direct analog of PureScript Run's
		/// [`extract`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// /
		/// [`runPure`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs).
		/// Pairs with [`Run::interpret_with`] for the
		/// chain-and-extract pipeline:
		/// `prog.interpret_with::<E1>(...).interpret_with::<E2>(...).extract()`.
		/// Future scoped-effect work will introduce a separate
		/// elimination operation for non-empty scoped rows, leaving
		/// `extract` as the fully-pure-program entry point.
		#[document_signature]
		///
		#[document_returns("The final result value of the fully-narrowed program.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let pure_prog: Run<CNilBrand, CNilBrand, i32> = Run::pure(42);
		/// assert_eq!(pure_prog.extract(), 42);
		/// ```
		#[inline]
		pub fn extract(self) -> A {
			match self.peel() {
				Ok(a) => a,
				Err(Node::First(cnil)) => match cnil {},
				Err(Node::Scoped(cnil)) => match cnil {},
			}
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The state type (also the program's result type for `get`)."
	)]
	impl<R, ScopedRow, A> Run<R, ScopedRow, A>
	where
		R: crate::classes::WrapDrop + crate::classes::Functor + 'static,
		ScopedRow: crate::classes::WrapDrop + crate::classes::Functor + 'static,
		A: 'static,
	{
		/// Lifts a `Get` state effect into the Run program. Direct
		/// analog of PureScript Run's
		/// [`get`](https://github.com/natefaubion/purescript-run/blob/main/src/Run/State.purs).
		/// The program reads the current state and returns it as the
		/// result type `A` (the state type and the result type
		/// coincide for `get`).
		///
		/// `Idx` is the type-level position witness identifying where
		/// `BoxStateBrand<BoxBrand, A>` lives in the row `R`. Rust
		/// infers `Idx` whenever the effect appears unambiguously in
		/// the row.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("A `Run` program suspended at the lifted `Get` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		state::BoxState,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::get();
		/// // The program is suspended at the Get effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn get<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				crate::types::effects::member::Member<
						crate::types::Coyoneda<
							'static,
							crate::brands::BoxStateBrand<crate::brands::BoxBrand, A>,
							A,
						>,
						Idx,
					>, {
			let effect: crate::types::effects::state::BoxState<
				'static,
				crate::brands::BoxBrand,
				A,
				A,
			> = crate::types::effects::state::BoxState::Get(
				<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|s: A| s),
			);
			Self::lift::<crate::brands::BoxStateBrand<crate::brands::BoxBrand, A>, Idx>(effect)
		}

		/// Lifts an `Ask` reader effect into the Run program. Direct
		/// analog of PureScript Run's `ask`. The program reads the
		/// immutable environment and returns it as the result type
		/// `A` (the environment type and the result type coincide
		/// for `ask`).
		///
		/// `Idx` is the type-level position witness identifying where
		/// `BoxReaderBrand<BoxBrand, A>` lives in the row `R`. Rust
		/// infers `Idx` whenever the effect appears unambiguously in
		/// the row.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("A `Run` program suspended at the lifted `Ask` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		reader::BoxReader,
		/// 		run::Run,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::ask();
		/// // The program is suspended at the Ask effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn ask<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				crate::types::effects::member::Member<
						crate::types::Coyoneda<
							'static,
							crate::brands::BoxReaderBrand<crate::brands::BoxBrand, A>,
							A,
						>,
						Idx,
					>, {
			let effect: crate::types::effects::reader::BoxReader<
				'static,
				crate::brands::BoxBrand,
				A,
				A,
			> = crate::types::effects::reader::BoxReader::Ask(
				<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|e: A| e),
			);
			Self::lift::<crate::brands::BoxReaderBrand<crate::brands::BoxBrand, A>, Idx>(effect)
		}

		/// Lifts a `Throw` except effect into the Run program. Direct
		/// analog of PureScript Run's `throw`. The program raises an
		/// error of type `ErrorType` and never returns to the caller;
		/// the result type `A` is determined by the call-site (any
		/// `A` works because `Throw` doesn't produce one).
		///
		/// `ErrorType` is the error type carried by `ExceptBrand` in
		/// the row. Rust may need a turbofish on `ErrorType` because
		/// the value `e` may not constrain it from the call site
		/// alone.
		#[document_signature]
		///
		#[document_type_parameters(
			"The error type carried by `ExceptBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The error value to throw.")]
		///
		#[document_returns("A `Run` program suspended at the lifted `Throw` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		except::Except,
		/// 		run::Run,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::throw::<&'static str, _>("oops");
		/// // The program is suspended at the Throw effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn throw<ErrorType: 'static, Idx>(e: ErrorType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				crate::types::effects::member::Member<
						crate::types::Coyoneda<'static, crate::brands::ExceptBrand<ErrorType>, A>,
						Idx,
					>, {
			let effect: crate::types::effects::except::Except<'static, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}

		/// Lifts a scoped `Catch` effect into the Run program: run
		/// `action`, and if it throws an `E`, invoke `handler` with the
		/// error to produce a recovery program. Direct analog of
		/// PureScript Run's
		/// [`Run.Except.catch`](https://github.com/natefaubion/purescript-run/blob/main/src/Run/Except.purs)
		/// (parameter order matches Rust convention: action first,
		/// handler second).
		///
		/// `EBrand` is the [`BoxCatchBrand`](crate::brands::BoxCatchBrand) instantiation in the scoped row;
		/// `Idx` is the type-level position witness identifying where
		/// `BoxCatchBrand<BoxBrand, E>` lives in `ScopedRow`. Rust
		/// infers `Idx` whenever the brand appears unambiguously in the
		/// row.
		///
		/// The recovery `handler` is `FnOnce(E) -> Run<R, ScopedRow, A>`,
		/// matching the [`BoxCatch`](crate::types::effects::catch::BoxCatch)
		/// substrate's `Box<dyn FnOnce>` storage; it is invoked at most
		/// once when (and if) the action throws. The action and recovery
		/// programs share the same row signature.
		#[document_signature]
		///
		#[document_type_parameters(
			"The error type recovered from.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The protected action program.",
			"The recovery handler invoked on a thrown error."
		)]
		///
		#[document_returns("A `Run` program suspended at the scoped `Catch` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, &'static str>, CNilBrand>;
		///
		/// let action: Run<FirstRow, ScopedRow, i32> = Run::pure(42);
		/// let prog: Run<FirstRow, ScopedRow, i32> =
		/// 	Run::catch::<&'static str, _>(action, |_e| Run::pure(0));
		/// // The program is suspended at the Catch scoped layer; peel
		/// // returns Err carrying a `Node::Scoped(...)` projection.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn catch<E: 'static, Idx>(
			action: Run<R, ScopedRow, A>,
			handler: impl FnOnce(E) -> Run<R, ScopedRow, A> + 'static,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawRunFree<R, ScopedRow>,
			>): crate::types::effects::member::Member<
					crate::types::effects::catch::BoxCatch<
						'static,
						crate::brands::BoxBrand,
						E,
						RawRunFree<R, ScopedRow>,
					>,
					Idx,
				>, {
			let action_free = action.into_free().cast_erased();
			let catch: crate::types::effects::catch::BoxCatch<
				'static,
				crate::brands::BoxBrand,
				E,
				RawRunFree<R, ScopedRow>,
			> = crate::types::effects::catch::BoxCatch::Catch {
				action: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| action_free,
				),
				handler: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |e: E| handler(e).into_free().cast_erased(),
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawRunFree<R, ScopedRow>,
			>) as crate::types::effects::member::Member<
				crate::types::effects::catch::BoxCatch<
					'static,
					crate::brands::BoxBrand,
					E,
					RawRunFree<R, ScopedRow>,
				>,
				Idx,
			>>::inject(catch);
			Run(RunRepresentation::ScopedBoundary(RunScopedBoundaryFrame {
				layer,
				continuations: CatList::empty(),
				result: PhantomData,
			}))
		}

		/// Lifts a scoped `Local` effect into the Run program: run
		/// `action` under an environment value transformed by `modify`.
		/// Direct analog of PureScript Run's
		/// [`Run.Reader.local`](https://github.com/natefaubion/purescript-run/blob/main/src/Run/Reader.purs)
		/// (parameter order matches PureScript: modify first, action
		/// second).
		///
		/// `EBrand` is the [`BoxLocalBrand`](crate::brands::BoxLocalBrand)
		/// instantiation in the scoped row; `Idx` is the type-level
		/// position witness identifying where
		/// `BoxLocalBrand<BoxBrand, E>` lives in `ScopedRow`. Rust
		/// infers `Idx` whenever the brand appears unambiguously in the
		/// row.
		///
		/// The `modify` closure is `FnOnce(E) -> E`, matching the
		/// [`BoxLocal`](crate::types::effects::local::BoxLocal)
		/// substrate's `Box<dyn FnOnce>` storage; it is invoked at most
		/// once when the dispatcher applies the modify-and-restore
		/// pattern. The action and any first-order effect operations
		/// inside it observe the transformed environment for the scope
		/// of the `Local` layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The environment type transformed by `modify`.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The environment-transform closure (consumes the inherited environment value).",
			"The protected action program."
		)]
		///
		#[document_returns("A `Run` program suspended at the scoped `Local` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
		///
		/// let action: Run<FirstRow, ScopedRow, i32> = Run::pure(42);
		/// let prog: Run<FirstRow, ScopedRow, i32> = Run::local::<i32, _>(|e: i32| e + 1, action);
		/// // The program is suspended at the Local scoped layer; peel
		/// // returns Err carrying a `Node::Scoped(...)` projection.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn local<E: 'static, Idx>(
			modify: impl FnOnce(E) -> E + 'static,
			action: Run<R, ScopedRow, A>,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::Free<NodeBrand<R, ScopedRow>, A>,
			>): crate::types::effects::member::Member<
					crate::types::effects::local::BoxLocal<
						'static,
						crate::brands::BoxBrand,
						E,
						crate::types::Free<NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>, {
			let action_free = action.into_free();
			let local: crate::types::effects::local::BoxLocal<
				'static,
				crate::brands::BoxBrand,
				E,
				crate::types::Free<NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::local::BoxLocal::Local {
				modify: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |e: E| modify(e),
				),
				action: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| action_free,
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::Free<NodeBrand<R, ScopedRow>, A>,
			>) as crate::types::effects::member::Member<
				crate::types::effects::local::BoxLocal<
					'static,
					crate::brands::BoxBrand,
					E,
					crate::types::Free<NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(local);
			let node = Node::Scoped(layer);
			Run::from_free(crate::types::Free::wrap(node))
		}

		/// Lifts a [`BoxRefLocal`](crate::types::effects::ref_local::BoxRefLocal)
		/// scoped environment-modification effect (Ref flavour) into
		/// the `Run` program. Direct analog of PureScript Run's
		/// [`local`](https://github.com/natefaubion/purescript-run/blob/main/src/Run/Reader.purs)
		/// for the by-reference closure shape: the program runs `action`
		/// under an environment value transformed by `modify`, where
		/// `modify` borrows the inherited environment value (`&E -> E`)
		/// rather than consuming it. Removes the `E: Clone` requirement
		/// that the Val flavour ([`local`](Run::local)) imposes on users
		/// who want to derive a sub-scope environment from the parent
		/// without owning it.
		///
		/// `EBrand` is the [`BoxRefLocalBrand`](crate::brands::BoxRefLocalBrand)
		/// instantiation in the scoped row; `Idx` is the type-level
		/// position witness identifying where
		/// `BoxRefLocalBrand<BoxBrand, E>` lives in `ScopedRow`. Rust
		/// infers `Idx` whenever the brand appears unambiguously in the
		/// row.
		///
		/// The `modify` closure is `FnOnce(&E) -> E`, matching the
		/// [`BoxRefLocal`](crate::types::effects::ref_local::BoxRefLocal)
		/// substrate's `Box<dyn FnOnce(&E) -> E>` storage; it is invoked
		/// at most once when the dispatcher applies the modify-and-restore
		/// pattern. The action and any first-order effect operations
		/// inside it observe the transformed environment for the scope of
		/// the `Local` layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The environment type borrowed by `modify`.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The environment-transform closure (borrows the inherited environment value).",
			"The protected action program."
		)]
		///
		#[document_returns("A `Run` program suspended at the scoped `Local` effect (Ref flavour).")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>;
		///
		/// let action: Run<FirstRow, ScopedRow, i32> = Run::pure(42);
		/// let prog: Run<FirstRow, ScopedRow, i32> = Run::ref_local::<i32, _>(|e: &i32| *e + 1, action);
		/// // The program is suspended at the RefLocal scoped layer; peel
		/// // returns Err carrying a `Node::Scoped(...)` projection.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn ref_local<E: 'static, Idx>(
			modify: impl FnOnce(&E) -> E + 'static,
			action: Run<R, ScopedRow, A>,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::Free<NodeBrand<R, ScopedRow>, A>,
			>): crate::types::effects::member::Member<
					crate::types::effects::ref_local::BoxRefLocal<
						'static,
						crate::brands::BoxBrand,
						E,
						crate::types::Free<NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>, {
			let action_free = action.into_free();
			let local: crate::types::effects::ref_local::BoxRefLocal<
				'static,
				crate::brands::BoxBrand,
				E,
				crate::types::Free<NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::ref_local::BoxRefLocal::Local {
				modify: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::ref_new(
					move |e: &E| modify(e),
				),
				action: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| action_free,
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::Free<NodeBrand<R, ScopedRow>, A>,
			>) as crate::types::effects::member::Member<
				crate::types::effects::ref_local::BoxRefLocal<
					'static,
					crate::brands::BoxBrand,
					E,
					crate::types::Free<NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(local);
			let node = Node::Scoped(layer);
			Run::from_free(crate::types::Free::wrap(node))
		}

		/// Lifts a scoped `Span` effect into the `Run` program: run
		/// `action` under instrumentation identified by `tag`.
		///
		/// `Tag` is stored by value in the scoped cell. The default
		/// single-shot substrate stores the action as a
		/// `Box<dyn FnOnce(()) -> _>` thunk, so this constructor does not
		/// require `Tag: Clone`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The instrumentation tag type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The instrumentation tag.", "The protected action program.")]
		///
		#[document_returns("A `Run` program suspended at the scoped `Span` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
		///
		/// let action: Run<FirstRow, ScopedRow, i32> = Run::pure(42);
		/// let prog: Run<FirstRow, ScopedRow, i32> = Run::span::<&'static str, _>("request", action);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn span<Tag: 'static, Idx>(
			tag: Tag,
			action: Run<R, ScopedRow, A>,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::Free<NodeBrand<R, ScopedRow>, A>,
			>): crate::types::effects::member::Member<
					crate::types::effects::span::BoxSpan<
						'static,
						crate::brands::BoxBrand,
						Tag,
						crate::types::Free<NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>, {
			let action_free = action.into_free();
			let span: crate::types::effects::span::BoxSpan<
				'static,
				crate::brands::BoxBrand,
				Tag,
				crate::types::Free<NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::span::BoxSpan::Span {
				tag,
				action: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| action_free,
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::Free<NodeBrand<R, ScopedRow>, A>,
			>) as crate::types::effects::member::Member<
				crate::types::effects::span::BoxSpan<
					'static,
					crate::brands::BoxBrand,
					Tag,
					crate::types::Free<NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(span);
			let node = Node::Scoped(layer);
			Run::from_free(crate::types::Free::wrap(node))
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The body's result type."
	)]
	impl<R, ScopedRow, B> Run<R, ScopedRow, B>
	where
		R: crate::classes::WrapDrop + crate::classes::Functor + 'static,
		ScopedRow: crate::classes::WrapDrop + crate::classes::Functor + 'static,
		B: 'static,
	{
		/// Lifts a [`BoxBracket`](crate::types::effects::bracket::BoxBracket)
		/// scoped resource-management effect into the `Run` program.
		/// Direct analog of PureScript Run's
		/// [`Aff.bracket`](https://github.com/purescript-contrib/purescript-aff/blob/master/src/Effect/Aff.purs):
		/// acquire a resource, run a body that uses it, then release the
		/// resource regardless of the body's outcome.
		///
		/// The cell holds three closures with three differently-typed
		/// program returns over the same substrate brand
		/// `Sub = NodeBrand<R, ScopedRow>`: `acquire` returns
		/// `Run<R, ScopedRow, A>` (resource), `body` returns
		/// `Run<R, ScopedRow, (A, B)>` (paired resource and body
		/// result for the dispatcher), `release` returns
		/// `Run<R, ScopedRow, ()>` (unit). Body returns the resource
		/// alongside its result so the dispatcher can pass the resource
		/// to release; the bracket operation itself returns `B` after
		/// release completes.
		///
		/// `Idx` is the type-level position witness identifying where
		/// `BoxBracketBrand<BoxBrand, NodeBrand<R, ScopedRow>, A, B>`
		/// lives in `ScopedRow`. Rust infers `Idx` whenever the brand
		/// appears unambiguously in the row.
		#[document_signature]
		///
		#[document_type_parameters(
			"The resource type produced by acquire.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The acquire program (produces the resource).",
			"The body closure (consumes the resource and returns a paired program with the body's result).",
			"The release closure (consumes the resource and returns a unit program for cleanup)."
		)]
		///
		#[document_returns("A `Run` program suspended at the scoped `Bracket` effect.")]
		///
		#[document_examples]
		///
		/// User-facing scoped rows containing
		/// [`BoxBracketBrand`](crate::brands::BoxBracketBrand) cannot be
		/// defined as type aliases (Rust rejects the recursion). Use the
		/// marker-struct workaround: a zero-sized struct that breaks the
		/// type-alias cycle by hosting the recursive references inside
		/// `impl_kind!` and trait impl bodies. The pattern is validated
		/// by the [B18 POC](../../../../tests/poc_bracket_marker_row.rs).
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::*,
		/// 	classes::{
		/// 		Functor,
		/// 		WrapDrop,
		/// 	},
		/// 	impl_kind,
		/// 	kinds::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type UnderlyingRow = CoproductBrand<
		/// 	BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>,
		/// 	CNilBrand,
		/// >;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl Functor for ScopedRow {
		/// 	fn map<'a, A: 'a, B: 'a>(
		/// 		f: impl Fn(A) -> B + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as Functor>::map(f, fa)
		/// 	}
		/// }
		///
		/// type FirstRow = CNilBrand;
		///
		/// let acquire: Run<FirstRow, ScopedRow, i32> = Run::pure(7);
		/// let prog: Run<FirstRow, ScopedRow, i32> = Run::<FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 	acquire,
		/// 	|resource: Box<i32>| Run::pure((*resource, 42)),
		/// 	|_resource: Box<i32>| Run::pure(()),
		/// );
		/// // The program is suspended at the Bracket scoped layer; peel
		/// // returns Err carrying a `Node::Scoped(...)` projection.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn bracket<A, Idx>(
			acquire: Run<R, ScopedRow, A>,
			body: impl FnOnce(
				<crate::brands::BoxBrand as crate::classes::Pointer>::Of<'static, A>,
			) -> Run<R, ScopedRow, (A, B)>
			+ 'static,
			release: impl FnOnce(
				<crate::brands::BoxBrand as crate::classes::Pointer>::Of<'static, A>,
			) -> Run<R, ScopedRow, ()>
			+ 'static,
		) -> Self
		where
			A: 'static,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::Free<NodeBrand<R, ScopedRow>, B>,
			>): crate::types::effects::member::Member<
					crate::types::effects::bracket::BoxBracket<
						'static,
						crate::brands::BoxBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				>, {
			let acquire_free = acquire.into_free();
			let bracket: crate::types::effects::bracket::BoxBracket<
				'static,
				crate::brands::BoxBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::bracket::BoxBracket::Bracket {
				acquire: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| acquire_free,
				),
				body: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |a: <crate::brands::BoxBrand as crate::classes::Pointer>::Of<
						'static,
						A,
					>| { body(a).into_free() },
				),
				release: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |a: <crate::brands::BoxBrand as crate::classes::Pointer>::Of<
						'static,
						A,
					>| { release(a).into_free() },
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::Free<NodeBrand<R, ScopedRow>, B>,
			>) as crate::types::effects::member::Member<
				crate::types::effects::bracket::BoxBracket<
					'static,
					crate::brands::BoxBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			let node = Node::Scoped(layer);
			Run::from_free(crate::types::Free::wrap(node))
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, ScopedRow> Run<R, ScopedRow, ()>
	where
		R: crate::classes::WrapDrop + crate::classes::Functor + 'static,
		ScopedRow: crate::classes::WrapDrop + crate::classes::Functor + 'static,
	{
		/// Lifts a `Put` state effect into the Run program. Direct
		/// analog of PureScript Run's
		/// [`put`](https://github.com/natefaubion/purescript-run/blob/main/src/Run/State.purs).
		/// The program writes the supplied state value `s` and
		/// returns `()` as the result type.
		///
		/// `StateType` is the state type carried by `BoxStateBrand`
		/// in the row. Rust may need a turbofish on `StateType`
		/// because `put`'s result type is `()` (which doesn't
		/// constrain the state type from the call site).
		#[document_signature]
		///
		#[document_type_parameters(
			"The state type carried by `BoxStateBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The new state value to write.")]
		///
		#[document_returns("A `Run` program suspended at the lifted `Put` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		state::BoxState,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, ()> = Run::put::<i32, _>(42);
		/// // The program is suspended at the Put effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn put<StateType: 'static, Idx>(s: StateType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				crate::types::effects::member::Member<
						crate::types::Coyoneda<
							'static,
							crate::brands::BoxStateBrand<crate::brands::BoxBrand, StateType>,
							(),
						>,
						Idx,
					>, {
			let effect: crate::types::effects::state::BoxState<
				'static,
				crate::brands::BoxBrand,
				StateType,
				(),
			> = crate::types::effects::state::BoxState::Put(
				s,
				<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|_: ()| ()),
			);
			Self::lift::<crate::brands::BoxStateBrand<crate::brands::BoxBrand, StateType>, Idx>(
				effect,
			)
		}

		/// Lifts a `Tell` writer effect into the Run program. Direct
		/// analog of PureScript Run's `tell`. The program emits the
		/// log value `log` and returns `()` as the result type.
		///
		/// `LogType` is the log type carried by `WriterBrand` in the
		/// row. Rust may need a turbofish on `LogType` because
		/// `tell`'s result type is `()` (which doesn't constrain the
		/// log type from the call site).
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type carried by `WriterBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The log value to emit.")]
		///
		#[document_returns("A `Run` program suspended at the lifted `Tell` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		writer::Writer,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<WriterBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, ()> = Run::tell::<&'static str, _>("logged");
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn tell<LogType: 'static, Idx>(log: LogType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				crate::types::effects::member::Member<
						crate::types::Coyoneda<'static, crate::brands::WriterBrand<LogType>, ()>,
						Idx,
					>, {
			let effect: crate::types::effects::writer::Writer<'static, LogType, ()> =
				crate::types::effects::writer::Writer::Tell(log, (), core::marker::PhantomData);
			Self::lift::<crate::brands::WriterBrand<LogType>, Idx>(effect)
		}
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::{
			brands::{
				BoxBrand,
				BoxCatchBrand,
				CNilBrand,
				CoproductBrand,
				CoyonedaBrand,
				IdentityBrand,
				NodeBrand,
			},
			classes::{
				Functor,
				ToDynFnOnce,
				WrapDrop,
			},
			kinds::Kind_cdc7cd43dac7585f,
			types::{
				CatList,
				Coyoneda,
				Free,
				Identity,
				effects::{
					catch::BoxCatch,
					coproduct::Coproduct,
					handlers::HandlersNil,
					interpreter::ScopedContinuation,
					node::Node,
				},
				free::{
					Continuation,
					FreeRawStep,
					TypeErasedValue,
				},
			},
		},
	};

	type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
	type Scoped = CNilBrand;
	type RunAlias<A> = Run<FirstRow, Scoped, A>;
	type EmptyNode = NodeBrand<CNilBrand, CNilBrand>;
	type EmptyRawRun = RawRunFree<CNilBrand, CNilBrand>;
	type EmptyRun<A> = Run<CNilBrand, CNilBrand, A>;
	type CatchScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, &'static str>, CNilBrand>;
	type CatchNode = NodeBrand<CNilBrand, CatchScopedRow>;
	type CatchRawRun = RawRunFree<CNilBrand, CatchScopedRow>;
	type CatchRun<A> = Run<CNilBrand, CatchScopedRow, A>;
	type IdentityCatchRawRun = RawRunFree<FirstRow, CatchScopedRow>;
	type IdentityCatchRun<A> = Run<FirstRow, CatchScopedRow, A>;
	type NarrowedCatchNode = NodeBrand<CNilBrand, CatchScopedRow>;
	type NarrowedCatchRawRun = RawRunFree<CNilBrand, CatchScopedRow>;

	struct IdentityPolymorphicHandler;

	impl<RMinusE, S> RunFirstOrderHandler<IdentityBrand, RMinusE, S> for IdentityPolymorphicHandler
	where
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		fn handle<T: 'static>(
			&self,
			effect: Identity<Run<RMinusE, S, T>>,
		) -> Run<RMinusE, S, T> {
			effect.0
		}
	}

	fn raw_i32(value: i32) -> EmptyRawRun {
		Free::<EmptyNode, _>::pure(value).cast_erased()
	}

	fn boxed_raw_i32(value: i32) -> EmptyRawRun {
		Free::<EmptyNode, _>::pure(value).erase_type()
	}

	fn multiply_by_ten_continuation() -> Continuation<EmptyNode> {
		Box::new(|value| {
			let value = match value.downcast::<i32>() {
				Ok(value) => *value,
				Err(_) => return raw_i32(0),
			};

			raw_i32(value * 10)
		})
	}

	fn increment_raw_i32_value(value: TypeErasedValue) -> EmptyRawRun {
		let value = match value.downcast::<i32>() {
			Ok(value) => *value,
			Err(_) => return raw_i32(0),
		};

		raw_i32(value + 1)
	}

	fn append_increment_to_raw_action(action: EmptyRawRun) -> EmptyRawRun {
		action.bind(increment_raw_i32_value)
	}

	fn run_scoped_continuation(
		action: EmptyRawRun
	) -> RunScopedContinuation<CNilBrand, CNilBrand, i32> {
		RunScopedContinuation {
			action,
			continuations: CatList::singleton(multiply_by_ten_continuation()),
			result: core::marker::PhantomData,
		}
	}

	fn catch_raw_i32(value: i32) -> CatchRawRun {
		Free::<CatchNode, _>::pure(value).cast_erased()
	}

	fn identity_catch_raw_i32(value: i32) -> IdentityCatchRawRun {
		Run::<FirstRow, CatchScopedRow, i32>::lift::<IdentityBrand, _>(Identity(value))
			.into_free()
			.erase_type()
	}

	fn catch_boundary(action_value: i32) -> CatchRun<i32> {
		let action = catch_raw_i32(action_value);
		let catch: BoxCatch<'static, BoxBrand, &'static str, CatchRawRun> = BoxCatch::Catch {
			action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action),
			handler: <BoxBrand as ToDynFnOnce>::new(|_: &'static str| catch_raw_i32(0)),
		};
		let layer = Coproduct::inject(catch);
		Run(RunRepresentation::ScopedBoundary(RunScopedBoundaryFrame {
			layer,
			continuations: CatList::empty(),
			result: core::marker::PhantomData,
		}))
	}

	fn identity_catch_boundary(
		action_value: i32,
		recovery_value: i32,
	) -> IdentityCatchRun<i32> {
		let action = identity_catch_raw_i32(action_value);
		let catch: BoxCatch<'static, BoxBrand, &'static str, IdentityCatchRawRun> =
			BoxCatch::Catch {
				action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action),
				handler: <BoxBrand as ToDynFnOnce>::new(move |_: &'static str| {
					identity_catch_raw_i32(recovery_value)
				}),
			};
		let layer = Coproduct::inject(catch);
		Run(RunRepresentation::ScopedBoundary(RunScopedBoundaryFrame {
			layer,
			continuations: CatList::empty(),
			result: core::marker::PhantomData,
		}))
	}

	fn public_catch(action_value: i32) -> CatchRun<i32> {
		Run::catch::<&'static str, _>(Run::pure(action_value), |_err| Run::pure(0))
	}

	fn assert_boundary_action_and_result<A>(
		program: CatchRun<A>,
		expected_action_value: i32,
		expected_result: A,
		expected_continuations: usize,
	) where
		A: core::fmt::Debug + PartialEq + 'static, {
		let boundary = match program.0 {
			RunRepresentation::ScopedBoundary(boundary) => Some(boundary),
			RunRepresentation::Free(_) => None,
		};
		assert!(boundary.is_some(), "expected scoped boundary representation");
		let Some(boundary) = boundary else {
			return;
		};

		assert_eq!(boundary.continuations.len(), expected_continuations);
		match boundary.layer {
			Coproduct::Inl(BoxCatch::Catch {
				action,
				handler: _,
			}) => {
				let action: Free<CatchNode, i32> =
					Free::continue_from_erased(action(()), CatList::empty());
				assert!(matches!(
					action.into_raw_step(),
					FreeRawStep::Done(value) if value == expected_action_value
				));

				let final_free: Free<CatchNode, A> = Free::continue_from_erased(
					catch_raw_i32(expected_action_value),
					boundary.continuations,
				);
				assert!(matches!(
					final_free.into_raw_step(),
					FreeRawStep::Done(value) if value == expected_result
				));
			}
			Coproduct::Inr(cnil) => match cnil {},
		}
	}

	fn assert_boundary_handler_and_result<A>(
		program: CatchRun<A>,
		expected_recovery_value: i32,
		expected_result: A,
		expected_continuations: usize,
	) where
		A: core::fmt::Debug + PartialEq + 'static, {
		let boundary = match program.0 {
			RunRepresentation::ScopedBoundary(boundary) => Some(boundary),
			RunRepresentation::Free(_) => None,
		};
		assert!(boundary.is_some(), "expected scoped boundary representation");
		let Some(boundary) = boundary else {
			return;
		};

		assert_eq!(boundary.continuations.len(), expected_continuations);
		match boundary.layer {
			Coproduct::Inl(BoxCatch::Catch {
				action: _,
				handler,
			}) => {
				let recovery: Free<CatchNode, i32> =
					Free::continue_from_erased(handler("oops"), CatList::empty());
				assert!(matches!(
					recovery.into_raw_step(),
					FreeRawStep::Done(value) if value == expected_recovery_value
				));

				let final_free: Free<CatchNode, A> = Free::continue_from_erased(
					catch_raw_i32(expected_recovery_value),
					boundary.continuations,
				);
				assert!(matches!(
					final_free.into_raw_step(),
					FreeRawStep::Done(value) if value == expected_result
				));
			}
			Coproduct::Inr(cnil) => match cnil {},
		}
	}

	#[test]
	fn from_free_and_into_free_round_trip() {
		let free: Free<NodeBrand<FirstRow, Scoped>, i32> = Free::pure(42);
		let run: RunAlias<i32> = Run::from_free(free);
		let _back: Free<NodeBrand<FirstRow, Scoped>, i32> = run.into_free();
	}

	#[test]
	fn drop_a_pure_run_does_not_panic() {
		let run: RunAlias<i32> = Run::from_free(Free::pure(7));
		drop(run);
	}

	#[test]
	fn pure_then_peel_returns_value() {
		let run: RunAlias<i32> = Run::pure(42);
		assert!(matches!(run.peel(), Ok(42)));
	}

	#[test]
	fn send_produces_suspended_program() {
		let coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(Identity(7));
		let layer = Coproduct::inject(coyo);
		let run: RunAlias<i32> = Run::send(Node::First(layer));
		assert!(run.peel().is_err());
	}

	#[test]
	fn pure_uses_free_backed_representation() {
		let run: RunAlias<i32> = Run::pure(42);
		assert!(matches!(run.0, RunRepresentation::Free(_)));
	}

	#[test]
	fn first_order_send_uses_free_backed_representation() {
		let coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(Identity(7));
		let layer = Coproduct::inject(coyo);
		let run: RunAlias<i32> = Run::send(Node::First(layer));
		assert!(matches!(run.0, RunRepresentation::Free(_)));
	}

	#[test]
	fn result_polymorphic_handler_narrows_free_backed_first_order_step() {
		let run: RunAlias<i32> = Run::lift::<IdentityBrand, _>(Identity(42));

		let narrowed: EmptyRun<i32> =
			run.interpret_with_handler::<IdentityBrand, _, CNilBrand>(IdentityPolymorphicHandler);

		assert_eq!(narrowed.extract(), 42);
	}

	#[test]
	fn run_catch_uses_scoped_boundary_representation() {
		let program = public_catch(7);

		assert_boundary_action_and_result(program, 7, 7, 0);
	}

	#[test]
	fn result_polymorphic_handler_narrows_boundary_catch_branches_before_outer_continuation() {
		let program =
			identity_catch_boundary(41, 5).bind(|value| Run::pure(format!("value={value}")));
		let boundary = match program.0 {
			RunRepresentation::ScopedBoundary(boundary) => Some(boundary),
			RunRepresentation::Free(_) => None,
		};
		assert!(boundary.is_some(), "expected scoped boundary representation");
		let Some(boundary) = boundary else {
			return;
		};

		assert_eq!(boundary.continuations.len(), 1);
		match boundary.layer {
			Coproduct::Inl(BoxCatch::Catch {
				action,
				handler,
			}) => {
				let narrowed_action: NarrowedCatchRawRun =
					Run::<FirstRow, CatchScopedRow, TypeErasedValue>::from_free(action(()))
						.interpret_with_handler::<IdentityBrand, _, CNilBrand>(
							IdentityPolymorphicHandler,
						)
						.into_free();
				let action: Free<NarrowedCatchNode, i32> =
					Free::continue_from_reboxed_erased(narrowed_action, CatList::empty());
				assert!(matches!(
					action.into_raw_step(),
					FreeRawStep::Done(value) if value == 41
				));

				let narrowed_recovery: NarrowedCatchRawRun =
					Run::<FirstRow, CatchScopedRow, TypeErasedValue>::from_free(handler("err"))
						.interpret_with_handler::<IdentityBrand, _, CNilBrand>(
							IdentityPolymorphicHandler,
						)
						.into_free();
				let recovery: Free<NarrowedCatchNode, i32> =
					Free::continue_from_reboxed_erased(narrowed_recovery, CatList::empty());
				assert!(matches!(
					recovery.into_raw_step(),
					FreeRawStep::Done(value) if value == 5
				));
			}
			Coproduct::Inr(cnil) => match cnil {},
		}
	}

	#[test]
	fn into_explicit_via_into_round_trips_pure() {
		use crate::types::effects::run_explicit::RunExplicit;
		let run: RunAlias<i32> = Run::pure(42);
		let explicit: RunExplicit<'static, FirstRow, Scoped, i32> = run.into();
		assert!(matches!(explicit.peel(), Ok(42)));
	}

	#[test]
	fn into_explicit_via_into_preserves_suspended_layer() {
		use crate::types::effects::run_explicit::RunExplicit;
		let coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(Identity(7));
		let layer = Coproduct::inject(coyo);
		let run: RunAlias<i32> = Run::send(Node::First(layer));
		let explicit: RunExplicit<'static, FirstRow, Scoped, i32> = run.into();
		assert!(explicit.peel().is_err());
	}

	#[test]
	fn bind_chains_pure_values() {
		let run: RunAlias<i32> =
			Run::pure(2).bind(|x| Run::pure(x + 1)).bind(|x| Run::pure(x * 10));
		assert!(matches!(run.peel(), Ok(30)));
	}

	#[test]
	fn scoped_continuation_resumes_raw_action_before_outer_continuation() {
		let carrier = ScopedContinuation::new(run_scoped_continuation(raw_i32(41)));

		let result: EmptyRun<i32> = carrier.resume_default(&HandlersNil);

		assert_eq!(result.extract(), 410);
	}

	#[test]
	fn scoped_continuation_transforms_raw_action_before_outer_continuation() {
		let carrier = ScopedContinuation::new(run_scoped_continuation(raw_i32(40)));

		let result: EmptyRun<i32> =
			carrier.resume_default_with_post_action(&HandlersNil, increment_raw_i32_value);

		assert_eq!(result.extract(), 410);
	}

	#[test]
	fn scoped_continuation_transforms_raw_action_program_before_outer_continuation() {
		let carrier = ScopedContinuation::new(run_scoped_continuation(boxed_raw_i32(40)));

		let result: EmptyRun<i32> = carrier
			.resume_default_with_action_transform(&HandlersNil, append_increment_to_raw_action);

		assert_eq!(result.extract(), 410);
	}

	#[test]
	fn scoped_boundary_map_preserves_action_and_stores_outer_continuation() {
		let program = catch_boundary(7).map(|value| value + 1);

		assert_boundary_action_and_result(program, 7, 8, 1);
	}

	#[test]
	fn scoped_boundary_bind_preserves_action_and_stores_outer_continuation() {
		let program = catch_boundary(7).bind(|value| Run::pure(format!("value={value}")));

		assert_boundary_action_and_result(program, 7, "value=7".to_owned(), 1);
	}

	#[test]
	fn scoped_boundary_map_then_bind_keeps_continuations_outside_action() {
		let program = catch_boundary(7)
			.map(|value| value + 1)
			.bind(|value| Run::pure(format!("value={value}")));

		assert_boundary_action_and_result(program, 7, "value=8".to_owned(), 2);
	}

	#[test]
	fn run_catch_map_keeps_continuation_outside_action() {
		let program = public_catch(7).map(|value| value + 1);

		assert_boundary_action_and_result(program, 7, 8, 1);
	}

	#[test]
	fn run_catch_bind_keeps_continuation_outside_action() {
		let program = public_catch(7).bind(|value| Run::pure(format!("value={value}")));

		assert_boundary_action_and_result(program, 7, "value=7".to_owned(), 1);
	}

	#[test]
	fn run_catch_recovery_handler_is_stored_in_boundary_representation() {
		let program: CatchRun<i32> =
			Run::catch::<&'static str, _>(Run::pure(7), |_err| Run::pure(40))
				.map(|value| value + 2);

		assert_boundary_handler_and_result(program, 40, 42, 1);
	}

	#[test]
	fn run_catch_peel_action_view_runs_pending_continuation() {
		let program = public_catch(7).map(|value| value + 1);
		let action_result = match program.peel() {
			Err(Node::Scoped(Coproduct::Inl(BoxCatch::Catch {
				action,
				handler: _,
			}))) => Some(action(())),
			_ => None,
		};

		assert!(matches!(action_result.map(Run::peel), Some(Ok(8))));
	}

	#[test]
	fn run_catch_peel_handler_view_runs_pending_continuation() {
		let program: CatchRun<i32> =
			Run::catch::<&'static str, _>(Run::pure(7), |_err| Run::pure(40))
				.map(|value| value + 2);
		let handler_result = match program.peel() {
			Err(Node::Scoped(Coproduct::Inl(BoxCatch::Catch {
				action: _,
				handler,
			}))) => Some(handler("oops")),
			_ => None,
		};

		assert!(matches!(handler_result.map(Run::peel), Some(Ok(42))));
	}

	#[test]
	fn run_catch_interpret_scoped_with_action_runs_pending_continuation() {
		let program = public_catch(7).map(|value| value + 1);
		let interpreted: EmptyRun<i32> = program
			.interpret_scoped_with::<BoxCatchBrand<BoxBrand, &'static str>, _, CNilBrand>(
				|catch| match catch {
					BoxCatch::Catch {
						action,
						handler: _,
					} => action(()),
				},
			);

		assert_eq!(interpreted.extract(), 8);
	}

	#[test]
	fn run_catch_interpret_scoped_with_handler_runs_pending_continuation() {
		let program: CatchRun<i32> =
			Run::catch::<&'static str, _>(Run::pure(7), |_err| Run::pure(40))
				.map(|value| value + 2);
		let interpreted: EmptyRun<i32> = program
			.interpret_scoped_with::<BoxCatchBrand<BoxBrand, &'static str>, _, CNilBrand>(
				|catch| match catch {
					BoxCatch::Catch {
						action: _,
						handler,
					} => handler("oops"),
				},
			);

		assert_eq!(interpreted.extract(), 42);
	}

	#[test]
	fn map_transforms_pure_value() {
		let run: RunAlias<i32> = Run::pure(7).map(|x| x * 3);
		assert!(matches!(run.peel(), Ok(21)));
	}
}
