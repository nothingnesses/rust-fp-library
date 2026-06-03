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
		"The stored value type."
	)]
	impl<R, ScopedRow, V> ArcRun<R, ScopedRow, Option<V>>
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		ScopedRow: Kind_cdc7cd43dac7585f + 'static,
		V: Clone + Send + Sync + 'static,
		NodeBrand<R, ScopedRow>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
	{
		define_run_wrapper! {
			wrapper ArcRun;
			effect KVStore;
			method lookup;
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
		define_run_wrapper! {
			wrapper ArcRun;
			effect KVStore;
			method update;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Output;
			method output;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Log;
			method log;
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The state type (also the program's result type for `get`)."
	)]
	#[document_parameters("The `ArcRun` instance.")]
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
		define_run_wrapper! {
			wrapper ArcRun;
			effect Reader;
			method ask;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect State;
			method get;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Fresh;
			method fresh;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Input;
			method input;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Coroutine;
			method yield_value;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Fail;
			method fail;
		}

		define_run_wrapper_method! {
			wrapper ArcRun;
			method expand;
		}

		define_run_wrapper_method! {
			wrapper ArcRun;
			method weaken;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Except;
			method throw;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Empty;
			method empty;
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
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		except::Except,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendCatchBrand<ArcBrand, &'static str>, CNilBrand>;
		/// type Prog = ArcRun<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = ArcRun::throw::<&'static str, _>("boom");
		/// let program: Prog = ArcRun::catch::<&'static str, _>(action, |error| {
		/// 	assert_eq!(error, "boom");
		/// 	ArcRun::pure(42)
		/// });
		///
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ExceptBrand<&'static str>: |_op: Except<'_, &'static str, Prog>| {
		/// 			ArcRun::pure(-1)
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		SendCatchBrand<ArcBrand, &'static str>:
		/// 			catch_handler::<_, FirstRowMinusExcept, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
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
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		reader::SendReader,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendLocalBrand<ArcBrand, i32>, CNilBrand>;
		/// type Prog = ArcRun<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = ArcRun::<FirstRow, ScopedRow, i32>::ask().bind(|env| ArcRun::pure(env * 2));
		/// let program: Prog = ArcRun::local::<i32, _>(|env| env + 1, action);
		///
		/// let result = program.handle(
		/// 	handlers! {
		/// 		SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, Prog>| {
		/// 			match op {
		/// 				SendReader::Ask(k) => k(10),
		/// 			}
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		SendLocalBrand<ArcBrand, i32>: local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 22);
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
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		reader::SendReader,
		/// 		standard_scoped_handlers::ref_local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendRefLocalBrand<ArcBrand, i32>, CNilBrand>;
		/// type Prog = ArcRun<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = ArcRun::<FirstRow, ScopedRow, i32>::ask().bind(|env| ArcRun::pure(env * 2));
		/// let program: Prog = ArcRun::ref_local::<i32, _>(|env| *env + 5, action);
		///
		/// let result = program.handle(
		/// 	handlers! {
		/// 		SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, Prog>| {
		/// 			match op {
		/// 				SendReader::Ask(k) => k(10),
		/// 			}
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		SendRefLocalBrand<ArcBrand, i32>:
		/// 			ref_local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 30);
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
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, &'static str>, CNilBrand>;
		/// type Prog = ArcRun<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = ArcRun::span::<&'static str, _>(
		/// 	"outer",
		/// 	ArcRun::span::<&'static str, _>("inner", ArcRun::pure(42)),
		/// );
		///
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SendSpanBrand<ArcBrand, &'static str>: span_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
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
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		handlers,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			arc_run::ArcRun,
		/// 			standard_scoped_handlers::writer_post_handler,
		/// 			writer::Writer,
		/// 		},
		/// 	},
		/// 	std::sync::{
		/// 		Arc,
		/// 		Mutex,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type ScopedRow = CoproductBrand<SendWriterCensorBrand<ArcBrand, String>, CNilBrand>;
		/// type Prog = ArcRun<FirstRow, ScopedRow, i32>;
		///
		/// let log = Arc::new(Mutex::new(Vec::new()));
		/// let log_for_handler = Arc::clone(&log);
		/// let action: Prog = ArcRun::<FirstRow, ScopedRow, ()>::tell::<String, _>("first".to_string())
		/// 	.bind(|()| ArcRun::<FirstRow, ScopedRow, ()>::tell::<String, _>("second".to_string()))
		/// 	.bind(|()| ArcRun::pure(40));
		/// let program: Prog =
		/// 	ArcRun::censor::<String, _>(|log| format!("[{log}]"), action).bind(|value| {
		/// 		ArcRun::<FirstRow, ScopedRow, ()>::tell::<String, _>("outer".to_string())
		/// 			.bind(move |()| ArcRun::pure(value + 2))
		/// 	});
		///
		/// let result = program.handle(
		/// 	handlers! {
		/// 		WriterBrand<String>: move |op: Writer<'_, String, Prog>| match op {
		/// 			Writer::Tell(log, next, _) => {
		/// 				log_for_handler
		/// 					.lock()
		/// 					.expect("log mutex should not be poisoned")
		/// 					.push(log);
		/// 				next
		/// 			}
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		SendWriterCensorBrand<ArcBrand, String>:
		/// 			writer_post_handler::<_, CNilBrand, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert_eq!(
		/// 	*log.lock().expect("log mutex should not be poisoned"),
		/// 	vec!["[firstsecond]".to_string(), "outer".to_string()],
		/// );
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
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		handlers,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			arc_run::ArcRun,
		/// 			standard_scoped_handlers::writer_post_handler,
		/// 			writer::Writer,
		/// 		},
		/// 	},
		/// 	std::sync::{
		/// 		Arc,
		/// 		Mutex,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type ScopedRow = CoproductBrand<SendWriterListenBrand<ArcBrand, String, i32>, CNilBrand>;
		/// type Prog = ArcRun<FirstRow, ScopedRow, (i32, String)>;
		///
		/// let log = Arc::new(Mutex::new(Vec::new()));
		/// let log_for_handler = Arc::clone(&log);
		/// let action = ArcRun::<FirstRow, ScopedRow, ()>::tell::<String, _>("first".to_string())
		/// 	.bind(|()| ArcRun::<FirstRow, ScopedRow, ()>::tell::<String, _>("second".to_string()))
		/// 	.bind(|()| ArcRun::pure(40));
		/// let program: Prog = ArcRun::listen::<String, _>(action).bind(|(value, observed)| {
		/// 	ArcRun::<FirstRow, ScopedRow, ()>::tell::<String, _>("outer".to_string())
		/// 		.bind(move |()| ArcRun::pure((value + 2, observed.clone())))
		/// });
		///
		/// let result = program.handle(
		/// 	handlers! {
		/// 		WriterBrand<String>: move |op: Writer<'_, String, Prog>| match op {
		/// 			Writer::Tell(log, next, _) => {
		/// 				log_for_handler
		/// 					.lock()
		/// 					.expect("log mutex should not be poisoned")
		/// 					.push(log);
		/// 				next
		/// 			}
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		SendWriterListenBrand<ArcBrand, String, i32>:
		/// 			writer_post_handler::<_, CNilBrand, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, (42, "firstsecond".to_string()));
		/// assert_eq!(
		/// 	*log.lock().expect("log mutex should not be poisoned"),
		/// 	vec!["first".to_string(), "second".to_string(), "outer".to_string()],
		/// );
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
		#[document_examples]
		///
		/// Recursive Arc scoped rows are easiest to write as marker
		/// structs whose `Kind` projection contains the concrete
		/// `SendBracket` layer. That avoids the type-alias cycle while
		/// keeping the row small enough for rustc's `Send + Sync`
		/// projection checks.
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			SendFunctor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			arc_run::ArcRun,
		/// 			bracket::SendBracket,
		/// 			coproduct::{
		/// 				CNil,
		/// 				Coproduct,
		/// 			},
		/// 			standard_scoped_handlers::bracket_handler,
		/// 		},
		/// 	},
		/// 	std::sync::{
		/// 		Arc,
		/// 		Mutex,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Coproduct<SendBracket<'a, ArcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>, CNil>;
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		None
		/// 	}
		/// }
		///
		/// impl SendFunctor for ScopedRow {
		/// 	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		/// 		_f: impl Fn(A) -> B + Send + Sync + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		match fa {
		/// 			Coproduct::Inl(layer) => Coproduct::Inl(layer),
		/// 			Coproduct::Inr(remainder) => match remainder {},
		/// 		}
		/// 	}
		/// }
		///
		/// type FirstRow = CNilBrand;
		///
		/// let events = Arc::new(Mutex::new(Vec::new()));
		/// let acquire_events = Arc::clone(&events);
		/// let body_events = Arc::clone(&events);
		/// let release_events = Arc::clone(&events);
		///
		/// let acquire: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::pure(7).bind(move |resource| {
		/// 	acquire_events.lock().expect("events mutex should not be poisoned").push("acquire");
		/// 	ArcRun::pure(resource)
		/// });
		/// let program: ArcRun<FirstRow, ScopedRow, i32> =
		/// 	ArcRun::<FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 		acquire,
		/// 		move |resource: Arc<i32>| {
		/// 			body_events.lock().expect("events mutex should not be poisoned").push("body");
		/// 			ArcRun::pure((*resource, *resource + 35))
		/// 		},
		/// 		move |resource: Arc<i32>| {
		/// 			release_events.lock().expect("events mutex should not be poisoned").push("release");
		/// 			assert_eq!(*resource, 7);
		/// 			ArcRun::pure(())
		/// 		},
		/// 	);
		///
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SendBracketBrand<ArcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>:
		/// 			bracket_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert_eq!(
		/// 	events.lock().expect("events mutex should not be poisoned").as_slice(),
		/// 	["acquire", "body", "release"],
		/// );
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
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			SendFunctor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			arc_run::ArcRun,
		/// 			coproduct::{
		/// 				CNil,
		/// 				Coproduct,
		/// 			},
		/// 			ref_bracket::SendRefBracket,
		/// 			standard_scoped_handlers::ref_bracket_handler,
		/// 		},
		/// 	},
		/// 	std::sync::{
		/// 		Arc,
		/// 		Mutex,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Coproduct<SendRefBracket<'a, ArcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>, CNil>;
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		None
		/// 	}
		/// }
		///
		/// impl SendFunctor for ScopedRow {
		/// 	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		/// 		_f: impl Fn(A) -> B + Send + Sync + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		match fa {
		/// 			Coproduct::Inl(layer) => Coproduct::Inl(layer),
		/// 			Coproduct::Inr(remainder) => match remainder {},
		/// 		}
		/// 	}
		/// }
		///
		/// type FirstRow = CNilBrand;
		///
		/// let events = Arc::new(Mutex::new(Vec::new()));
		/// let acquire_events = Arc::clone(&events);
		/// let body_events = Arc::clone(&events);
		/// let release_events = Arc::clone(&events);
		///
		/// let acquire: ArcRun<FirstRow, ScopedRow, i32> = ArcRun::pure(7).bind(move |resource| {
		/// 	acquire_events.lock().expect("events mutex should not be poisoned").push("acquire");
		/// 	ArcRun::pure(resource)
		/// });
		/// let program: ArcRun<FirstRow, ScopedRow, i32> =
		/// 	ArcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 		acquire,
		/// 		move |resource: Arc<i32>| {
		/// 			body_events.lock().expect("events mutex should not be poisoned").push("body");
		/// 			ArcRun::pure(*resource + 35)
		/// 		},
		/// 		move |resource: Arc<i32>| {
		/// 			release_events.lock().expect("events mutex should not be poisoned").push("release");
		/// 			assert_eq!(*resource, 7);
		/// 			ArcRun::pure(())
		/// 		},
		/// 	);
		///
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SendRefBracketBrand<ArcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>:
		/// 			ref_bracket_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert_eq!(
		/// 	events.lock().expect("events mutex should not be poisoned").as_slice(),
		/// 	["acquire", "body", "release"],
		/// );
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
		define_run_wrapper! {
			wrapper ArcRun;
			effect State;
			method put;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Writer;
			method tell;
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
		define_run_wrapper! {
			wrapper ArcRun;
			effect Choose;
			method choose;
		}
	}
}

#[expect(
	unused_imports,
	reason = "inherent impl modules follow the document_module re-export boundary"
)]
pub use inner::*;
