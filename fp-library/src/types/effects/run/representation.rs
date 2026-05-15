#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		super::super::inner::{
			Run,
			RunFirstOrderHandler,
			RunFirstOrderReplacer,
		},
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
		core::marker::PhantomData,
		fp_macros::*,
	};

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
		pub(crate) fn free(free: Free<NodeBrand<R, S>, A>) -> Self {
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
		pub(crate) fn into_free(self) -> Free<NodeBrand<R, S>, A> {
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
		pub(crate) fn into_raw_step(self) -> FreeRawStep<NodeBrand<R, S>, A> {
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
		pub(crate) fn bind<B: 'static>(
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
		pub(crate) fn map<B: 'static>(
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

		/// Rewrites this boundary frame by interpreting one first-order
		/// effect out of every selected raw scoped branch and every
		/// pending continuation.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect being interpreted out of the row.",
			"The type-level position witness.",
			"The narrowed row brand.",
			"The concrete result-polymorphic handler type."
		)]
		#[document_parameters("The shared result-polymorphic handler.")]
		#[document_returns("A boundary frame whose raw branches live in the narrowed row.")]
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
		pub(crate) fn handle_with_handler<EBrand, Idx, RMinusE, H>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'static, H>,
		) -> RunScopedBoundaryFrame<RMinusE, S, A>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			H: RunFirstOrderHandler<EBrand, RMinusE, S> + 'static,
			Apply!(
				<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, TypeErasedValue>>
			): Member<
					Coyoneda<'static, EBrand, Run<R, S, TypeErasedValue>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, S, TypeErasedValue>,
									>
								),
				>, {
			let h_for_layer = handler.clone();
			let layer = <S as Functor>::map(
				move |inner| {
					Self::interpret_raw_free_with_handler::<EBrand, Idx, RMinusE, H>(
						inner,
						h_for_layer.clone(),
					)
				},
				self.layer,
			);
			let continuations = Self::interpret_continuations_with_handler::<EBrand, Idx, RMinusE, H>(
				self.continuations,
				handler,
			);

			RunScopedBoundaryFrame {
				layer,
				continuations,
				result: PhantomData,
			}
		}

		/// Rewrites one raw erased branch with a result-polymorphic
		/// first-order handler.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect being interpreted out of the row.",
			"The type-level position witness.",
			"The narrowed row brand.",
			"The concrete result-polymorphic handler type."
		)]
		#[document_parameters(
			"The raw erased branch to rewrite.",
			"The shared result-polymorphic handler."
		)]
		#[document_returns("The rewritten raw branch in the narrowed row.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(6).bind(|x| Run::pure(x * 7));
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn interpret_raw_free_with_handler<EBrand, Idx, RMinusE, H>(
			free: RawRunFree<R, S>,
			handler: <RcBrand as RefCountedPointer>::Of<'static, H>,
		) -> RawRunFree<RMinusE, S>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			H: RunFirstOrderHandler<EBrand, RMinusE, S> + 'static,
			Apply!(
				<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, TypeErasedValue>>
			): Member<
					Coyoneda<'static, EBrand, Run<R, S, TypeErasedValue>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, S, TypeErasedValue>,
									>
								),
				>, {
			let interpreted = Run::<R, S, TypeErasedValue>::from_free(free.erase_type())
				.handle_with_handler_shared::<EBrand, Idx, RMinusE, H>(handler)
				.into_free();
			Free::continue_from_reboxed_erased(interpreted, CatList::empty())
		}

		/// Rewrites a raw continuation queue with a result-polymorphic
		/// first-order handler.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect being interpreted out of the row.",
			"The type-level position witness.",
			"The narrowed row brand.",
			"The concrete result-polymorphic handler type."
		)]
		#[document_parameters(
			"The raw continuation queue to rewrite.",
			"The shared result-polymorphic handler."
		)]
		#[document_returns("A raw continuation queue in the narrowed row.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(40).map(|x| x + 2);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn interpret_continuations_with_handler<EBrand, Idx, RMinusE, H>(
			continuations: RunContinuations<R, S>,
			handler: <RcBrand as RefCountedPointer>::Of<'static, H>,
		) -> RunContinuations<RMinusE, S>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			H: RunFirstOrderHandler<EBrand, RMinusE, S> + 'static,
			Apply!(
				<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, TypeErasedValue>>
			): Member<
					Coyoneda<'static, EBrand, Run<R, S, TypeErasedValue>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, S, TypeErasedValue>,
									>
								),
				>, {
			let mut source = continuations;
			let mut rewritten = CatList::empty();

			while let Some((continuation, rest)) = source.uncons() {
				let h_for_continuation = handler.clone();
				let rewritten_continuation: Continuation<NodeBrand<RMinusE, S>> =
					Box::new(move |value| {
						let next = continuation(value);
						Self::interpret_raw_free_with_handler::<EBrand, Idx, RMinusE, H>(
							next,
							h_for_continuation,
						)
					});
				rewritten = rewritten.snoc(rewritten_continuation);
				source = rest;
			}

			rewritten
		}

		/// Rewrites this boundary frame by replacing one first-order
		/// effect inside every selected raw scoped branch and every
		/// pending continuation.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect being replaced.",
			"The type-level position witness.",
			"The narrowed row brand used while projecting the matched effect.",
			"The embedding witness used to rebuild the original row.",
			"The concrete result-polymorphic replacement type."
		)]
		#[document_parameters("The shared result-polymorphic replacement.")]
		#[document_returns("A boundary frame whose raw branches have been interposed.")]
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
		pub(crate) fn interpose_with_replacer<EBrand, Idx, RMinusE, EmbedIndices, P>(
			self,
			replacement: <RcBrand as RefCountedPointer>::Of<'static, P>,
		) -> RunScopedBoundaryFrame<R, S, A>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			P: RunFirstOrderReplacer<EBrand, R, S> + 'static,
			Apply!(
				<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, TypeErasedValue>>
			): Member<
					Coyoneda<'static, EBrand, Run<R, S, TypeErasedValue>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, S, TypeErasedValue>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, S>, TypeErasedValue>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Free<NodeBrand<R, S>, TypeErasedValue>,
					>),
					EmbedIndices,
				>, {
			let p_for_layer = replacement.clone();
			let layer = <S as Functor>::map(
				move |inner| {
					Self::interpose_raw_free_with_replacer::<EBrand, Idx, RMinusE, EmbedIndices, P>(
						inner,
						p_for_layer.clone(),
					)
				},
				self.layer,
			);
			let continuations = Self::interpose_continuations_with_replacer::<
				EBrand,
				Idx,
				RMinusE,
				EmbedIndices,
				P,
			>(self.continuations, replacement);

			RunScopedBoundaryFrame {
				layer,
				continuations,
				result: PhantomData,
			}
		}

		/// Replaces matching effects inside one raw erased branch.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect being replaced.",
			"The type-level position witness.",
			"The narrowed row brand used while projecting the matched effect.",
			"The embedding witness used to rebuild the original row.",
			"The concrete result-polymorphic replacement type."
		)]
		#[document_parameters(
			"The raw erased branch to rewrite.",
			"The shared result-polymorphic replacement."
		)]
		#[document_returns("The rewritten raw branch in the original row.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(6).bind(|x| Run::pure(x * 7));
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn interpose_raw_free_with_replacer<EBrand, Idx, RMinusE, EmbedIndices, P>(
			free: RawRunFree<R, S>,
			replacement: <RcBrand as RefCountedPointer>::Of<'static, P>,
		) -> RawRunFree<R, S>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			P: RunFirstOrderReplacer<EBrand, R, S> + 'static,
			Apply!(
				<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, TypeErasedValue>>
			): Member<
					Coyoneda<'static, EBrand, Run<R, S, TypeErasedValue>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, S, TypeErasedValue>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, S>, TypeErasedValue>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Free<NodeBrand<R, S>, TypeErasedValue>,
					>),
					EmbedIndices,
				>, {
			let interposed = Run::<R, S, TypeErasedValue>::from_free(free.erase_type())
				.interpose_with_replacer_shared::<EBrand, Idx, RMinusE, EmbedIndices, P>(
					replacement,
				)
				.into_free();
			Free::continue_from_reboxed_erased(interposed, CatList::empty())
		}

		/// Replaces matching effects inside a raw continuation queue.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect being replaced.",
			"The type-level position witness.",
			"The narrowed row brand used while projecting the matched effect.",
			"The embedding witness used to rebuild the original row.",
			"The concrete result-polymorphic replacement type."
		)]
		#[document_parameters(
			"The raw continuation queue to rewrite.",
			"The shared result-polymorphic replacement."
		)]
		#[document_returns("A raw continuation queue in the original row.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(40).map(|x| x + 2);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn interpose_continuations_with_replacer<EBrand, Idx, RMinusE, EmbedIndices, P>(
			continuations: RunContinuations<R, S>,
			replacement: <RcBrand as RefCountedPointer>::Of<'static, P>,
		) -> RunContinuations<R, S>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			P: RunFirstOrderReplacer<EBrand, R, S> + 'static,
			Apply!(
				<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, TypeErasedValue>>
			): Member<
					Coyoneda<'static, EBrand, Run<R, S, TypeErasedValue>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, S, TypeErasedValue>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, S>, TypeErasedValue>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Free<NodeBrand<R, S>, TypeErasedValue>,
					>),
					EmbedIndices,
				>, {
			let mut source = continuations;
			let mut rewritten = CatList::empty();

			while let Some((continuation, rest)) = source.uncons() {
				let p_for_continuation = replacement.clone();
				let rewritten_continuation: Continuation<NodeBrand<R, S>> =
					Box::new(move |value| {
						let next = continuation(value);
						Self::interpose_raw_free_with_replacer::<
							EBrand,
							Idx,
							RMinusE,
							EmbedIndices,
							P,
						>(next, p_for_continuation)
					});
				rewritten = rewritten.snoc(rewritten_continuation);
				source = rest;
			}

			rewritten
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
		type OperationProgram = RawRunFree<R, S>;
		type OperationValue = TypeErasedValue;
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
			) -> <Self as ScopedResumeTypes<'static>>::OperationProgram
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
	#[document_parameters("The scoped-handler handler value.")]
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
	/// This mirrors
	/// [`DispatchScopedHandlers`](crate::types::effects::interpreter::DispatchScopedHandlers)
	/// but keeps the pending
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
		"The handler value type.",
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
}

pub use inner::*;
