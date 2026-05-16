#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		super::super::inner::{
			ArcRun,
			ArcRunContinuations,
			RawArcRunFree,
			make_node_scoped,
			wrap_first_arc,
		},
		crate::{
			Apply,
			brands::{
				ArcBrand,
				NodeBrand,
			},
			classes::{
				SendFunctor,
				WrapDrop,
			},
			kinds::*,
			types::{
				ArcCoyoneda,
				ArcFree,
				arc_free::ArcTypeErasedValue,
				effects::member::Member,
			},
		},
		fp_macros::*,
	};
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The state type (also the program's result type for `get`)."
	)]
	impl<R, ScopedRow, A> ArcRun<R, ScopedRow, A>
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		ScopedRow: Kind_cdc7cd43dac7585f + 'static,
		A: 'static,
		NodeBrand<R, ScopedRow>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
	{
		/// Lifts a `Get` state effect into the `ArcRun` program.
		/// Mirrors [`Run::get`](crate::types::effects::run::Run::get);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendStateBrand`](crate::brands::SendStateBrand) (rather
		/// than `StateBrand`) so the continuation projection
		/// `<ArcBrand as SendRefCountedPointer>::Of<'_, dyn Fn(...) + Send + Sync>`
		/// is structurally `Send + Sync`, which the `SendFunctor`
		/// algebra requires across thread boundaries.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `ArcRun` program suspended at the lifted `Get` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		state::SendState,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::get();
		/// // The program is suspended at the Get effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn get<Idx>() -> Self
		where
			A: Clone + Send + Sync,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>): Member<
					ArcCoyoneda<
						'static,
						crate::brands::SendStateBrand<crate::brands::ArcBrand, A>,
						A,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::state::SendState<
				'static,
				crate::brands::ArcBrand,
				A,
				A,
			> = crate::types::effects::state::SendState::Get(
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|s: A| s),
			);
			Self::lift::<crate::brands::SendStateBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}

		/// Lifts an `Ask` reader effect into the `ArcRun` program.
		/// Mirrors [`Run::ask`](crate::types::effects::run::Run::ask);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendReaderBrand`](crate::brands::SendReaderBrand) (rather
		/// than `ReaderBrand`) so the continuation projection
		/// `<ArcBrand as SendRefCountedPointer>::Of<'_, dyn Fn(...) + Send + Sync>`
		/// is structurally `Send + Sync`.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `ArcRun` program suspended at the lifted `Ask` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		reader::SendReader,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::ask();
		/// // The program is suspended at the Ask effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn ask<Idx>() -> Self
		where
			A: Clone + Send + Sync,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>): Member<
					ArcCoyoneda<
						'static,
						crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>,
						A,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::reader::SendReader<
				'static,
				crate::brands::ArcBrand,
				A,
				A,
			> = crate::types::effects::reader::SendReader::Ask(
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|e: A| e),
			);
			Self::lift::<crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}

		/// Lifts a `Throw` except effect into the `ArcRun` program.
		/// Mirrors [`Run::throw`](crate::types::effects::run::Run::throw);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: requires `ErrorType: Send + Sync` so the lifted
		/// layer participates in the Arc substrate's thread-safety
		/// cascade. The same
		/// [`ExceptBrand`](crate::brands::ExceptBrand) serves all six
		/// wrappers because [`Except`](crate::types::effects::except::Except)
		/// has no `dyn Fn` continuation; no parallel `SendExceptBrand`
		/// is needed.
		#[document_signature]
		///
		#[document_type_parameters(
			"The error type carried by `ExceptBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The error value to throw.")]
		///
		#[document_returns("An `ArcRun` program suspended at the lifted `Throw` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		except::Except,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::throw::<&'static str, _>("oops");
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn throw<ErrorType: Clone + Send + Sync + 'static, Idx>(e: ErrorType) -> Self
		where
			A: Send + Sync,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, crate::brands::ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::except::Except<'static, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}

		/// Lifts an `Empty` effect into the `ArcRun` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// thread-safe multi-shot nondeterministic interpreter.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `ArcRun` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		empty::Empty,
		/// 		scoped_nt,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, Vec<i32>> = ArcRun::empty();
		/// let result = prog.handle(
		/// 	handlers! {
		/// 		EmptyBrand: |_op: Empty<'_, ArcRun<FirstRow, Scoped, Vec<i32>>>| ArcRun::pure(Vec::new()),
		/// 	},
		/// 	scoped_nt(),
		/// );
		/// assert_eq!(result, Vec::<i32>::new());
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			A: Send + Sync,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, crate::brands::EmptyBrand, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::empty::Empty<'static, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}

		/// Lifts a scoped `Catch` effect into the `ArcRun` program: run
		/// `action`, and if it throws an `E`, invoke `handler` with the
		/// error to produce a recovery program. Mirrors
		/// [`Run::catch`](crate::types::effects::run::Run::catch); see
		/// that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: the action and recovery handler are stored as
		/// `Arc<dyn Fn(...) -> _ + Send + Sync>` thunks (multi-shot,
		/// thread-safe). The action thunk invokes `action.clone()`
		/// (cheap Arc-bump on `ArcRun`) on each call.
		#[document_signature]
		///
		#[document_type_parameters(
			"The error type recovered from.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The protected action program (must be `Clone + Send + Sync` for the multi-shot Arc-thunk).",
			"The recovery handler invoked on a thrown error (multi-shot via [`Fn`], thread-safe)."
		)]
		///
		#[document_returns("An `ArcRun` program suspended at the scoped `Catch` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendCatchBrand<ArcBrand, &'static str>, CNilBrand>;
		///
		/// let action: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::pure(42);
		/// let prog: ArcRun<FirstRow, ScopedRow, i32> =
		/// 	ArcRun::catch::<&'static str, _>(action, |_e| ArcRun::pure(0));
		/// // The program is suspended at the Catch scoped layer; peel
		/// // returns Err carrying a `Node::Scoped(...)` projection.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn catch<E: Send + Sync + 'static, Idx>(
			action: ArcRun<R, ScopedRow, A>,
			handler: impl Fn(E) -> ArcRun<R, ScopedRow, A> + Send + Sync + 'static,
		) -> Self
		where
			A: Send + Sync,
			R: WrapDrop + SendFunctor,
			ScopedRow: WrapDrop + SendFunctor,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::catch::SendCatch<
						'static,
						ArcBrand,
						E,
						ArcFree<NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let catch: crate::types::effects::catch::SendCatch<
				'static,
				ArcBrand,
				E,
				ArcFree<NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::catch::SendCatch::Catch {
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					action.clone().into_arc_free()
				}),
				handler: <ArcBrand as crate::classes::ToDynSendFn>::new(move |e: E| {
					handler(e).into_arc_free()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::catch::SendCatch<
					'static,
					ArcBrand,
					E,
					ArcFree<NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(catch);
			let node = make_node_scoped::<R, ScopedRow, ArcFree<NodeBrand<R, ScopedRow>, A>>(layer);
			ArcRun::from_arc_free(wrap_first_arc::<R, ScopedRow, A>(node))
		}

		/// Lifts a scoped `Local` effect into the `ArcRun` program: run
		/// `action` under an environment value transformed by `modify`.
		/// Mirrors [`Run::local`](crate::types::effects::run::Run::local);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: the modify closure and action are stored as
		/// `Arc<dyn Fn(...) -> _ + Send + Sync>` thunks (multi-shot,
		/// thread-safe). The action thunk invokes `action.clone()`
		/// (cheap Arc-bump on `ArcRun`) on each call.
		#[document_signature]
		///
		#[document_type_parameters(
			"The environment type transformed by `modify`.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The environment-transform closure (multi-shot via [`Fn`], thread-safe).",
			"The protected action program (must be `Clone + Send + Sync` for the multi-shot Arc-thunk)."
		)]
		///
		#[document_returns("An `ArcRun` program suspended at the scoped `Local` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendLocalBrand<ArcBrand, i32>, CNilBrand>;
		///
		/// let action: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::pure(42);
		/// let prog: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::local::<i32, _>(|e: i32| e + 1, action);
		/// // The program is suspended at the Local scoped layer; peel
		/// // returns Err carrying a `Node::Scoped(...)` projection.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn local<E: Send + Sync + 'static, Idx>(
			modify: impl Fn(E) -> E + Send + Sync + 'static,
			action: ArcRun<R, ScopedRow, A>,
		) -> Self
		where
			A: Send + Sync,
			R: WrapDrop + SendFunctor,
			ScopedRow: WrapDrop + SendFunctor,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::local::SendLocal<
						'static,
						ArcBrand,
						E,
						ArcFree<NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let local: crate::types::effects::local::SendLocal<
				'static,
				ArcBrand,
				E,
				ArcFree<NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::local::SendLocal::Local {
				modify: <ArcBrand as crate::classes::ToDynSendFn>::new(move |e: E| modify(e)),
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					action.clone().into_arc_free()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::local::SendLocal<
					'static,
					ArcBrand,
					E,
					ArcFree<NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(local);
			let node = make_node_scoped::<R, ScopedRow, ArcFree<NodeBrand<R, ScopedRow>, A>>(layer);
			ArcRun::from_arc_free(wrap_first_arc::<R, ScopedRow, A>(node))
		}

		/// Lifts a [`SendRefLocal`](crate::types::effects::ref_local::SendRefLocal)
		/// scoped environment-modification effect (Ref flavour) into the
		/// `ArcRun` program. Mirrors
		/// [`Run::ref_local`](crate::types::effects::run::Run::ref_local)
		/// for the thread-safe Arc-substrate. The `modify` closure
		/// (`Fn(&E) -> E + Send + Sync`) borrows the inherited
		/// environment value rather than consuming it, removing the
		/// `E: Clone` requirement that the Val flavour
		/// ([`local`](ArcRun::local)) imposes.
		#[document_signature]
		///
		#[document_type_parameters(
			"The environment type borrowed by `modify` (`Send + Sync`).",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The environment-transform closure (borrows the inherited environment value, `Send + Sync`).",
			"The protected action program."
		)]
		///
		#[document_returns(
			"An `ArcRun` program suspended at the scoped `Local` effect (Ref flavour)."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendRefLocalBrand<ArcBrand, i32>, CNilBrand>;
		///
		/// let action: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::pure(42);
		/// let prog: ArcRun<FirstRow, ScopedRow, i32> =
		/// 	ArcRun::ref_local::<i32, _>(|e: &i32| *e + 1, action);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn ref_local<E: Send + Sync + 'static, Idx>(
			modify: impl Fn(&E) -> E + Send + Sync + 'static,
			action: ArcRun<R, ScopedRow, A>,
		) -> Self
		where
			A: Send + Sync,
			R: WrapDrop + SendFunctor,
			ScopedRow: WrapDrop + SendFunctor,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::ref_local::SendRefLocal<
						'static,
						ArcBrand,
						E,
						ArcFree<NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let local: crate::types::effects::ref_local::SendRefLocal<
				'static,
				ArcBrand,
				E,
				ArcFree<NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::ref_local::SendRefLocal::Local {
				modify: <ArcBrand as crate::classes::ToDynSendFn>::ref_new(move |e: &E| modify(e)),
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					action.clone().into_arc_free()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::ref_local::SendRefLocal<
					'static,
					ArcBrand,
					E,
					ArcFree<NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(local);
			let node = make_node_scoped::<R, ScopedRow, ArcFree<NodeBrand<R, ScopedRow>, A>>(layer);
			ArcRun::from_arc_free(wrap_first_arc::<R, ScopedRow, A>(node))
		}

		/// Lifts a scoped `Span` effect into the `ArcRun` program: run
		/// `action` under instrumentation identified by `tag`.
		/// Mirrors [`Run::span`](crate::types::effects::run::Run::span);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: the action is stored as an
		/// `Arc<dyn Fn(()) -> _ + Send + Sync>` thunk and the by-value
		/// tag must be cloneable and thread-safe.
		#[document_signature]
		///
		#[document_type_parameters(
			"The instrumentation tag type (`Clone + Send + Sync`).",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The instrumentation tag.",
			"The protected action program (must be `Clone + Send + Sync` for the multi-shot Arc-thunk)."
		)]
		///
		#[document_returns("An `ArcRun` program suspended at the scoped `Span` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, &'static str>, CNilBrand>;
		///
		/// let action: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::pure(42);
		/// let prog: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::span::<&'static str, _>("request", action);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn span<Tag: Clone + Send + Sync + 'static, Idx>(
			tag: Tag,
			action: ArcRun<R, ScopedRow, A>,
		) -> Self
		where
			A: Send + Sync,
			R: WrapDrop + SendFunctor,
			ScopedRow: WrapDrop + SendFunctor,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::span::SendSpan<
						'static,
						ArcBrand,
						Tag,
						ArcFree<NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let span: crate::types::effects::span::SendSpan<
				'static,
				ArcBrand,
				Tag,
				ArcFree<NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::span::SendSpan::Span {
				tag,
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					action.clone().into_arc_free()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::span::SendSpan<
					'static,
					ArcBrand,
					Tag,
					ArcFree<NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(span);
			let node = make_node_scoped::<R, ScopedRow, ArcFree<NodeBrand<R, ScopedRow>, A>>(layer);
			ArcRun::from_arc_free(wrap_first_arc::<R, ScopedRow, A>(node))
		}

		/// Lifts a neutral scoped Writer `censor` effect into the
		/// `ArcRun` program.
		///
		/// The selected action result remains `A`. The constructor only
		/// stores the selected action and thread-safe log
		/// transformation; the standard Writer handler later decides how
		/// the transformed log is combined with surrounding output.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type transformed by `censor` (`Send + Sync`).",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The thread-safe log transformation.",
			"The selected action program (must be cloneable and thread-safe for the Arc thunk)."
		)]
		///
		#[document_returns("An `ArcRun` program suspended at the scoped Writer `censor` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendWriterCensorBrand<ArcBrand, String>, CNilBrand>;
		///
		/// let action: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::pure(42);
		/// let prog: ArcRun<FirstRow, ScopedRow, i32> =
		/// 	ArcRun::censor::<String, _>(|log| format!("{log}!"), action);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn censor<LogType: Send + Sync + 'static, Idx>(
			censor: impl Fn(LogType) -> LogType + Send + Sync + 'static,
			action: ArcRun<R, ScopedRow, A>,
		) -> Self
		where
			A: Clone + Send + Sync,
			R: WrapDrop + SendFunctor,
			ScopedRow: WrapDrop + SendFunctor,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawArcRunFree<R, ScopedRow>,
			>): Member<
					crate::types::effects::writer::SendWriterCensor<
						'static,
						ArcBrand,
						LogType,
						RawArcRunFree<R, ScopedRow>,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let writer: crate::types::effects::writer::SendWriterCensor<
				'static,
				ArcBrand,
				LogType,
				RawArcRunFree<R, ScopedRow>,
			> = crate::types::effects::writer::SendWriterCensor::Censor {
				censor: <ArcBrand as crate::classes::ToDynSendFn>::new(censor),
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					action.clone().into_arc_free().cast_erased()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawArcRunFree<R, ScopedRow>,
			>) as Member<
				crate::types::effects::writer::SendWriterCensor<
					'static,
					ArcBrand,
					LogType,
					RawArcRunFree<R, ScopedRow>,
				>,
				Idx,
			>>::inject(writer);
			let node = make_node_scoped::<R, ScopedRow, RawArcRunFree<R, ScopedRow>>(layer);
			let raw = wrap_first_arc::<R, ScopedRow, ArcTypeErasedValue>(node);
			ArcRun::from_arc_free(ArcFree::continue_from_erased(
				raw,
				ArcRunContinuations::<R, ScopedRow>::empty(),
			))
		}

		/// Lifts a neutral scoped Writer `listen` effect into the
		/// `ArcRun` program.
		///
		/// The selected action result remains `A`; the standard Writer
		/// handler later pairs that action result with the observed log
		/// `LogType` before the outer continuation resumes.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type observed by `listen` (`Clone + Send + Sync`).",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The selected action program.")]
		///
		#[document_returns("An `ArcRun` program suspended at the scoped Writer `listen` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendWriterListenBrand<ArcBrand, String, i32>, CNilBrand>;
		///
		/// let action: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::pure(42);
		/// let prog: ArcRun<FirstRow, ScopedRow, (i32, String)> = ArcRun::listen::<String, _>(action);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn listen<LogType: Clone + Send + Sync + 'static, Idx>(
			action: ArcRun<R, ScopedRow, A>
		) -> ArcRun<R, ScopedRow, (A, LogType)>
		where
			A: Clone + Send + Sync,
			R: WrapDrop + SendFunctor,
			ScopedRow: WrapDrop + SendFunctor,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawArcRunFree<R, ScopedRow>,
			>): Member<
					crate::types::effects::writer::SendWriterListen<
						'static,
						ArcBrand,
						LogType,
						A,
						RawArcRunFree<R, ScopedRow>,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let writer: crate::types::effects::writer::SendWriterListen<
				'static,
				ArcBrand,
				LogType,
				A,
				RawArcRunFree<R, ScopedRow>,
			> = crate::types::effects::writer::SendWriterListen::Listen {
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					action.clone().into_arc_free().cast_erased()
				}),
				result: core::marker::PhantomData,
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawArcRunFree<R, ScopedRow>,
			>) as Member<
				crate::types::effects::writer::SendWriterListen<
					'static,
					ArcBrand,
					LogType,
					A,
					RawArcRunFree<R, ScopedRow>,
				>,
				Idx,
			>>::inject(writer);
			let node = make_node_scoped::<R, ScopedRow, RawArcRunFree<R, ScopedRow>>(layer);
			let raw = wrap_first_arc::<R, ScopedRow, ArcTypeErasedValue>(node);
			ArcRun::from_arc_free(ArcFree::continue_from_erased(
				raw,
				ArcRunContinuations::<R, ScopedRow>::empty(),
			))
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The body's result type (`Send + Sync`)."
	)]
	impl<R, ScopedRow, B> ArcRun<R, ScopedRow, B>
	where
		R: WrapDrop
			+ SendFunctor
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		ScopedRow: WrapDrop
			+ SendFunctor
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		NodeBrand<R, ScopedRow>: WrapDrop
			+ SendFunctor
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		B: Send + Sync + 'static,
	{
		/// Lifts a [`SendBracket`](crate::types::effects::bracket::SendBracket)
		/// scoped resource-management effect into the `ArcRun` program.
		/// Mirrors [`Run::bracket`](crate::types::effects::run::Run::bracket)
		/// for the thread-safe Arc-substrate. The cell stores the three
		/// closures behind `Arc<dyn Fn + Send + Sync>` pointers, so
		/// `body` and `release` are `Fn + Send + Sync` (multi-shot,
		/// thread-safe) closures and the resource is wrapped in
		/// `Arc<A>` so it can be shared across calls and threads.
		#[document_signature]
		///
		#[document_type_parameters(
			"The resource type produced by acquire (`Send + Sync`).",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The acquire program (produces the resource).",
			"The body closure (`Send + Sync`; receives the resource as `Arc<A>` and returns a paired program).",
			"The release closure (`Send + Sync`; receives the resource as `Arc<A>` and returns a unit program)."
		)]
		///
		#[document_returns("An `ArcRun` program suspended at the scoped `Bracket` effect.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// User-facing scoped rows containing
		/// [`SendBracketBrand`](crate::brands::SendBracketBrand) cannot
		/// be defined as type aliases (Rust rejects the recursion). The
		/// marker-struct workaround validated by the
		/// [B18 POC](../../../../tests/poc_bracket_marker_row.rs) breaks
		/// the type-alias cycle for the `Run` and `RcRun` families, but
		/// the Arc family hits an additional cycle: `SendBracketBrand`'s
		/// Kind impl requires `Sub`'s GAT projection at
		/// `ArcFree<Sub, ArcTypeErasedValue>` to be `Send + Sync`, and
		/// when `Sub = NodeBrand<CNilBrand, ScopedRow>` references the
		/// marker `ScopedRow` whose `Of` projection contains
		/// `SendBracketBrand` again, the `Send + Sync` check exceeds
		/// rustc's overflow limit. The smart constructor itself
		/// compiles cleanly; only the marker-struct doctest setup
		/// triggers the cycle. End-to-end exercise lives in
		/// `tests/run_bracket.rs` (step 3.3.4) where the marker
		/// struct's `Send + Sync` is checked once at the test-crate
		/// level rather than recursively in a doctest fixture.
		///
		/// ```
		/// // The smart constructor's type signature is exercised by
		/// // step 3.3.4's tests/run_bracket.rs; here we only confirm
		/// // the wrapper itself constructs (the bracket call site needs
		/// // a marker-struct row that overflows the doctest type-check).
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CNilBrand;
		/// let prog: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::pure(7);
		/// assert!(matches!(prog.peel(), Ok(7)));
		/// ```
		///
		/// ```ignore
		/// // Sketch (not run; overflows rustc's type-check recursion):
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::*,
		/// 	classes::{
		/// 		Functor,
		/// 		SendFunctor,
		/// 		WrapDrop,
		/// 	},
		/// 	impl_kind,
		/// 	kinds::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type UnderlyingRow = CoproductBrand<
		/// 	SendBracketBrand<ArcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>,
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
		/// impl SendFunctor for ScopedRow {
		/// 	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		/// 		f: impl Fn(A) -> B + Send + Sync + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as SendFunctor>::send_map(f, fa)
		/// 	}
		/// }
		///
		/// type FirstRow = CNilBrand;
		///
		/// let acquire: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::pure(7);
		/// let prog: ArcRun<FirstRow, ScopedRow, i32> =
		/// 	ArcRun::<FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 		acquire,
		/// 		|resource: std::sync::Arc<i32>| ArcRun::pure((*resource, 42)),
		/// 		|_resource: std::sync::Arc<i32>| ArcRun::pure(()),
		/// 	);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn bracket<A, Idx>(
			acquire: ArcRun<R, ScopedRow, A>,
			body: impl Fn(
				<ArcBrand as crate::classes::Pointer>::Of<'static, A>,
			) -> ArcRun<R, ScopedRow, (A, B)>
			+ Send
			+ Sync
			+ 'static,
			release: impl Fn(
				<ArcBrand as crate::classes::Pointer>::Of<'static, A>,
			) -> ArcRun<R, ScopedRow, ()>
			+ Send
			+ Sync
			+ 'static,
		) -> Self
		where
			A: Send + Sync + 'static,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, B>,
			>): Member<
					crate::types::effects::bracket::SendBracket<
						'static,
						ArcBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let bracket: crate::types::effects::bracket::SendBracket<
				'static,
				ArcBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::bracket::SendBracket::Bracket {
				acquire: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					acquire.clone().into_arc_free()
				}),
				body: <ArcBrand as crate::classes::ToDynSendFn>::new(
					move |a: <ArcBrand as crate::classes::Pointer>::Of<'static, A>| {
						body(a).into_arc_free()
					},
				),
				release: <ArcBrand as crate::classes::ToDynSendFn>::new(
					move |a: <ArcBrand as crate::classes::Pointer>::Of<'static, A>| {
						release(a).into_arc_free()
					},
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, B>,
			>) as Member<
				crate::types::effects::bracket::SendBracket<
					'static,
					ArcBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			let node = make_node_scoped::<R, ScopedRow, ArcFree<NodeBrand<R, ScopedRow>, B>>(layer);
			ArcRun::from_arc_free(wrap_first_arc::<R, ScopedRow, B>(node))
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The body's result type (`Send + Sync`)."
	)]
	impl<R, ScopedRow, B> ArcRun<R, ScopedRow, B>
	where
		R: WrapDrop
			+ SendFunctor
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		ScopedRow: WrapDrop
			+ SendFunctor
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		NodeBrand<R, ScopedRow>: WrapDrop
			+ SendFunctor
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		B: Send + Sync + 'static,
	{
		/// Lifts a [`SendRefBracket`](crate::types::effects::ref_bracket::SendRefBracket)
		/// scoped resource-management effect into the `ArcRun` program.
		/// This is the Ref flavour of [`ArcRun::bracket`]: `acquire`
		/// produces the resource, while `body` and `release` both
		/// receive independent `Arc<A>` clones.
		#[document_signature]
		///
		#[document_type_parameters(
			"The resource type produced by acquire (`Send + Sync`).",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The acquire program (produces the resource).",
			"The body closure (`Send + Sync`; receives the resource as `Arc<A>` and returns the body result program).",
			"The release closure (`Send + Sync`; receives the resource as `Arc<A>` and returns a unit program)."
		)]
		///
		#[document_returns("An `ArcRun` program suspended at the scoped `RefBracket` effect.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CNilBrand;
		/// let prog: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::pure(7);
		/// assert!(matches!(prog.peel(), Ok(7)));
		/// ```
		///
		/// ```ignore
		/// // Sketch: real scoped rows use the marker-struct workaround
		/// // documented on ArcRun::bracket.
		/// let acquire: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::pure(7);
		/// let prog: ArcRun<FirstRow, ScopedRow, i32> =
		/// 	ArcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 		acquire,
		/// 		|resource: std::sync::Arc<i32>| ArcRun::pure(*resource + 35),
		/// 		|_resource: std::sync::Arc<i32>| ArcRun::pure(()),
		/// 	);
		/// ```
		#[inline]
		pub fn ref_bracket<A: Send + Sync + 'static, Idx>(
			acquire: ArcRun<R, ScopedRow, A>,
			body: impl Fn(
				<ArcBrand as crate::classes::SendRefCountedPointer>::Of<'static, A>,
			) -> ArcRun<R, ScopedRow, B>
			+ Send
			+ Sync
			+ 'static,
			release: impl Fn(
				<ArcBrand as crate::classes::SendRefCountedPointer>::Of<'static, A>,
			) -> ArcRun<R, ScopedRow, ()>
			+ Send
			+ Sync
			+ 'static,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, B>,
			>): Member<
					crate::types::effects::ref_bracket::SendRefBracket<
						'static,
						ArcBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let bracket: crate::types::effects::ref_bracket::SendRefBracket<
				'static,
				ArcBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::ref_bracket::SendRefBracket::Bracket {
				acquire: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					acquire.clone().into_arc_free()
				}),
				body: <ArcBrand as crate::classes::ToDynSendFn>::new(
					move |a: <ArcBrand as crate::classes::SendRefCountedPointer>::Of<
						'static,
						A,
					>| { body(a).into_arc_free() },
				),
				release: <ArcBrand as crate::classes::ToDynSendFn>::new(
					move |a: <ArcBrand as crate::classes::SendRefCountedPointer>::Of<
						'static,
						A,
					>| { release(a).into_arc_free() },
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, B>,
			>) as Member<
				crate::types::effects::ref_bracket::SendRefBracket<
					'static,
					ArcBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			let node = make_node_scoped::<R, ScopedRow, ArcFree<NodeBrand<R, ScopedRow>, B>>(layer);
			ArcRun::from_arc_free(wrap_first_arc::<R, ScopedRow, B>(node))
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, ScopedRow> ArcRun<R, ScopedRow, ()>
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		ScopedRow: Kind_cdc7cd43dac7585f + 'static,
		NodeBrand<R, ScopedRow>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
	{
		/// Lifts a `Put` state effect into the `ArcRun` program.
		/// Mirrors [`Run::put`](crate::types::effects::run::Run::put);
		/// see that method for cross-wrapper semantics. Threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendStateBrand`](crate::brands::SendStateBrand); see
		/// [`get`](ArcRun::get) for the design rationale.
		#[document_signature]
		///
		#[document_type_parameters(
			"The state type carried by `SendStateBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The new state value to write.")]
		///
		#[document_returns("An `ArcRun` program suspended at the lifted `Put` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		state::SendState,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, ()> = ArcRun::put::<i32, _>(42);
		/// // The program is suspended at the Put effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn put<StateType: Clone + Send + Sync + 'static, Idx>(s: StateType) -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>): Member<
					ArcCoyoneda<
						'static,
						crate::brands::SendStateBrand<crate::brands::ArcBrand, StateType>,
						(),
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::state::SendState<
				'static,
				crate::brands::ArcBrand,
				StateType,
				(),
			> = crate::types::effects::state::SendState::Put(
				s,
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|_: ()| ()),
			);
			Self::lift::<crate::brands::SendStateBrand<crate::brands::ArcBrand, StateType>, Idx>(
				effect,
			)
		}

		/// Lifts a `Tell` writer effect into the `ArcRun` program.
		/// Mirrors [`Run::tell`](crate::types::effects::run::Run::tell);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: requires `LogType: Send + Sync` so the lifted
		/// layer participates in the Arc substrate's thread-safety
		/// cascade. The same
		/// [`WriterBrand`](crate::brands::WriterBrand) serves all six
		/// wrappers because [`Writer`](crate::types::effects::writer::Writer)
		/// has no `dyn Fn` continuation; no parallel `SendWriterBrand`
		/// is needed.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type carried by `WriterBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The log value to emit.")]
		///
		#[document_returns("An `ArcRun` program suspended at the lifted `Tell` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		writer::Writer,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, ()> = ArcRun::tell::<&'static str, _>("logged");
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn tell<LogType: Clone + Send + Sync + 'static, Idx>(log: LogType) -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				Member<ArcCoyoneda<'static, crate::brands::WriterBrand<LogType>, ()>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::writer::Writer<'static, LogType, ()> =
				crate::types::effects::writer::Writer::Tell(log, (), core::marker::PhantomData);
			Self::lift::<crate::brands::WriterBrand<LogType>, Idx>(effect)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, ScopedRow> ArcRun<R, ScopedRow, bool>
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		ScopedRow: Kind_cdc7cd43dac7585f + 'static,
		NodeBrand<R, ScopedRow>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
	{
		/// Lifts an `Alt` choose effect into the `ArcRun` program.
		/// Mirrors
		/// [`RcRun::choose`](crate::types::effects::rc_run::RcRun::choose);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendChooseBrand`](crate::brands::SendChooseBrand) (rather
		/// than `ChooseBrand`) so the continuation projection is
		/// structurally `Send + Sync`.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `ArcRun` program suspended at the lifted `Alt` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		choose::SendChoose,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, bool> = ArcRun::choose();
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, bool>): Member<
					ArcCoyoneda<
						'static,
						crate::brands::SendChooseBrand<crate::brands::ArcBrand>,
						bool,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::choose::SendChoose<
				'static,
				crate::brands::ArcBrand,
				bool,
			> = crate::types::effects::choose::SendChoose::Alt(
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|b: bool| b),
			);
			Self::lift::<crate::brands::SendChooseBrand<crate::brands::ArcBrand>, Idx>(effect)
		}
	}
}

#[expect(
	unused_imports,
	reason = "inherent impl modules follow the document_module re-export boundary"
)]
pub use inner::*;
