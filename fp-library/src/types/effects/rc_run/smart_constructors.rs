#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		super::super::inner::{
			RawRcRunFree,
			RcRun,
			RcRunContinuations,
		},
		crate::{
			Apply,
			brands::{
				NodeBrand,
				RcBrand,
			},
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::*,
			types::{
				RcCoyoneda,
				RcFree,
				effects::{
					member::Member,
					node::Node,
				},
				rc_free::RcTypeErasedValue,
			},
		},
		fp_macros::*,
	};
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The state type (also the program's result type for `get`)."
	)]
	#[document_parameters("The `RcRun` instance.")]
	impl<R, ScopedRow, A> RcRun<R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: 'static,
	{
		define_run_wrapper_method! {
			wrapper RcRun;
			method expand;
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The state type (also the program's result type for `get`)."
	)]
	impl<R, ScopedRow, A> RcRun<R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: Clone + 'static,
	{
		define_run_wrapper! {
			wrapper RcRun;
			effect Reader;
			method ask;
		}

		define_run_wrapper! {
			wrapper RcRun;
			effect State;
			method get;
		}

		/// Lifts a `Throw` except effect into the `RcRun` program.
		/// Mirrors [`Run::throw`](crate::types::effects::run::Run::throw);
		/// see that method for cross-wrapper semantics.
		#[document_signature]
		///
		#[document_type_parameters(
			"The error type carried by `ExceptBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The error value to throw.")]
		///
		#[document_returns("An `RcRun` program suspended at the lifted `Throw` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::throw::<&'static str, _>("oops");
		/// let handled: RcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	prog.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("oops"));
		/// ```
		#[inline]
		pub fn throw<ErrorType: Clone + 'static, Idx>(e: ErrorType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, crate::brands::ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::except::Except<'static, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}

		/// Lifts an `Empty` effect into the `RcRun` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// multi-shot nondeterministic interpreter.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `RcRun` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::empty();
		/// let handled: RcRun<CNilBrand, CNilBrand, Option<i32>> = prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, crate::brands::EmptyBrand, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::empty::Empty<'static, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}

		/// Lifts a scoped `Catch` effect into the `RcRun` program: run
		/// `action`, and if it throws an `E`, invoke `handler` with the
		/// error to produce a recovery program. Mirrors
		/// [`Run::catch`](crate::types::effects::run::Run::catch); see
		/// that method for cross-wrapper semantics. Differences for
		/// `RcRun`: the action and recovery handler are stored as
		/// `Rc<dyn Fn(...) -> _>` thunks (multi-shot), so the catch
		/// scoped operation can fire multiple times along multi-shot
		/// continuations (e.g., `Choose` forks). The action thunk
		/// invokes `action.clone()` (cheap Rc-bump on `RcRun`) on each
		/// call.
		#[document_signature]
		///
		#[document_type_parameters(
			"The error type recovered from.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The protected action program (must be `Clone` for the multi-shot Rc-thunk).",
			"The recovery handler invoked on a thrown error (multi-shot via [`Fn`])."
		)]
		///
		#[document_returns("An `RcRun` program suspended at the scoped `Catch` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		except::Except,
		/// 		rc_run::RcRun,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<CatchBrand<RcBrand, &'static str>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = RcRun::throw::<&'static str, _>("boom");
		/// let program: Prog = RcRun::catch::<&'static str, _>(action, |error| {
		/// 	assert_eq!(error, "boom");
		/// 	RcRun::pure(42)
		/// });
		///
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ExceptBrand<&'static str>: |_op: Except<'_, &'static str, Prog>| {
		/// 			RcRun::pure(-1)
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		CatchBrand<RcBrand, &'static str>:
		/// 			catch_handler::<_, FirstRowMinusExcept, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		pub fn catch<E: 'static, Idx>(
			action: RcRun<R, ScopedRow, A>,
			handler: impl Fn(E) -> RcRun<R, ScopedRow, A> + 'static,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::catch::Catch<
						'static,
						RcBrand,
						E,
						RcFree<NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let catch: crate::types::effects::catch::Catch<
				'static,
				RcBrand,
				E,
				RcFree<NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::catch::Catch::Catch {
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					action.clone().into_rc_free()
				}),
				handler: <RcBrand as crate::classes::ToDynCloneFn>::new(move |e: E| {
					handler(e).into_rc_free()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::catch::Catch<
					'static,
					RcBrand,
					E,
					RcFree<NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(catch);
			let node = Node::Scoped(layer);
			RcRun::from_rc_free(RcFree::wrap(node))
		}

		/// Lifts a scoped `Local` effect into the `RcRun` program: run
		/// `action` under an environment value transformed by `modify`.
		/// Mirrors [`Run::local`](crate::types::effects::run::Run::local);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RcRun`: the modify closure and action are stored as
		/// `Rc<dyn Fn(...) -> _>` thunks (multi-shot), so the local
		/// scoped operation can fire multiple times along multi-shot
		/// continuations. The action thunk invokes `action.clone()`
		/// (cheap Rc-bump on `RcRun`) on each call.
		#[document_signature]
		///
		#[document_type_parameters(
			"The environment type transformed by `modify`.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The environment-transform closure (multi-shot via [`Fn`]).",
			"The protected action program (must be `Clone` for the multi-shot Rc-thunk)."
		)]
		///
		#[document_returns("An `RcRun` program suspended at the scoped `Local` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		rc_run::RcRun,
		/// 		reader::Reader,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<LocalBrand<RcBrand, i32>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = RcRun::<FirstRow, ScopedRow, i32>::ask().bind(|env| RcRun::pure(env * 2));
		/// let program: Prog = RcRun::local::<i32, _>(|env| env + 1, action);
		///
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, Prog>| match op {
		/// 			Reader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		LocalBrand<RcBrand, i32>: local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 22);
		/// ```
		#[inline]
		pub fn local<E: 'static, Idx>(
			modify: impl Fn(E) -> E + 'static,
			action: RcRun<R, ScopedRow, A>,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::local::Local<
						'static,
						RcBrand,
						E,
						RcFree<NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let local: crate::types::effects::local::Local<
				'static,
				RcBrand,
				E,
				RcFree<NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::local::Local::Local {
				modify: <RcBrand as crate::classes::ToDynCloneFn>::new(move |e: E| modify(e)),
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					action.clone().into_rc_free()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::local::Local<
					'static,
					RcBrand,
					E,
					RcFree<NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(local);
			let node = Node::Scoped(layer);
			RcRun::from_rc_free(RcFree::wrap(node))
		}

		/// Lifts a [`RefLocal`](crate::types::effects::ref_local::RefLocal)
		/// scoped environment-modification effect (Ref flavour) into the
		/// `RcRun` program. Mirrors
		/// [`Run::ref_local`](crate::types::effects::run::Run::ref_local)
		/// for the multi-shot Rc-substrate. The `modify` closure
		/// (`Fn(&E) -> E`) borrows the inherited environment value rather
		/// than consuming it, removing the `E: Clone` requirement that
		/// the Val flavour ([`local`](RcRun::local)) imposes.
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
		#[document_returns(
			"An `RcRun` program suspended at the scoped `Local` effect (Ref flavour)."
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
		/// 		rc_run::RcRun,
		/// 		reader::Reader,
		/// 		standard_scoped_handlers::ref_local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<RefLocalBrand<RcBrand, i32>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = RcRun::<FirstRow, ScopedRow, i32>::ask().bind(|env| RcRun::pure(env * 2));
		/// let program: Prog = RcRun::ref_local::<i32, _>(|env| *env + 5, action);
		///
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, Prog>| match op {
		/// 			Reader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		RefLocalBrand<RcBrand, i32>: ref_local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 30);
		/// ```
		#[inline]
		pub fn ref_local<E: 'static, Idx>(
			modify: impl Fn(&E) -> E + 'static,
			action: RcRun<R, ScopedRow, A>,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::ref_local::RefLocal<
						'static,
						RcBrand,
						E,
						RcFree<NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let local: crate::types::effects::ref_local::RefLocal<
				'static,
				RcBrand,
				E,
				RcFree<NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::ref_local::RefLocal::Local {
				modify: <RcBrand as crate::classes::ToDynCloneFn>::ref_new(move |e: &E| modify(e)),
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					action.clone().into_rc_free()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::ref_local::RefLocal<
					'static,
					RcBrand,
					E,
					RcFree<NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(local);
			let node = Node::Scoped(layer);
			RcRun::from_rc_free(RcFree::wrap(node))
		}

		/// Lifts a scoped `Span` effect into the `RcRun` program: run
		/// `action` under instrumentation identified by `tag`.
		/// Mirrors [`Run::span`](crate::types::effects::run::Run::span);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RcRun`: the action is stored as an `Rc<dyn Fn(()) -> _>`
		/// thunk and the by-value tag must be cloneable when the scoped
		/// cell is cloned.
		#[document_signature]
		///
		#[document_type_parameters(
			"The instrumentation tag type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The instrumentation tag (must be cloneable for the Rc cell).",
			"The protected action program (must be `Clone` for the multi-shot Rc-thunk)."
		)]
		///
		#[document_returns("An `RcRun` program suspended at the scoped `Span` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		rc_run::RcRun,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog = RcRun::span::<&'static str, _>(
		/// 	"outer",
		/// 	RcRun::span::<&'static str, _>("inner", RcRun::pure(42)),
		/// );
		///
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SpanBrand<RcBrand, &'static str>: span_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		pub fn span<Tag: Clone + 'static, Idx>(
			tag: Tag,
			action: RcRun<R, ScopedRow, A>,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::span::Span<
						'static,
						RcBrand,
						Tag,
						RcFree<NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let span: crate::types::effects::span::Span<
				'static,
				RcBrand,
				Tag,
				RcFree<NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::span::Span::Span {
				tag,
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					action.clone().into_rc_free()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::span::Span<
					'static,
					RcBrand,
					Tag,
					RcFree<NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(span);
			let node = Node::Scoped(layer);
			RcRun::from_rc_free(RcFree::wrap(node))
		}

		/// Lifts a neutral scoped Writer `censor` effect into the
		/// `RcRun` program.
		///
		/// The selected action result remains `A`. The constructor only
		/// stores the selected action and log transformation; the
		/// standard Writer handler later decides how the transformed log
		/// is combined with surrounding output.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type transformed by `censor`.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The log transformation (must be multi-shot for the Rc cell).",
			"The selected action program (must be cloneable for the Rc thunk)."
		)]
		///
		#[document_returns("An `RcRun` program suspended at the scoped Writer `censor` effect.")]
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
		/// 			rc_run::RcRun,
		/// 			standard_scoped_handlers::writer_post_handler,
		/// 			writer::Writer,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::RefCell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type ScopedRow = CoproductBrand<WriterCensorBrand<RcBrand, String>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
		///
		/// let log = Rc::new(RefCell::new(Vec::new()));
		/// let log_for_handler = Rc::clone(&log);
		/// let action: Prog = RcRun::<FirstRow, ScopedRow, ()>::tell::<String, _>("first".to_string())
		/// 	.bind(|()| RcRun::<FirstRow, ScopedRow, ()>::tell::<String, _>("second".to_string()))
		/// 	.bind(|()| RcRun::pure(40));
		/// let program: Prog =
		/// 	RcRun::censor::<String, _>(|log| format!("[{log}]"), action).bind(|value| {
		/// 		RcRun::<FirstRow, ScopedRow, ()>::tell::<String, _>("outer".to_string())
		/// 			.bind(move |()| RcRun::pure(value + 2))
		/// 	});
		///
		/// let result = program.handle(
		/// 	handlers! {
		/// 		WriterBrand<String>: move |op: Writer<'_, String, Prog>| match op {
		/// 			Writer::Tell(log, next, _) => {
		/// 				log_for_handler.borrow_mut().push(log);
		/// 				next
		/// 			}
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		WriterCensorBrand<RcBrand, String>: writer_post_handler::<_, CNilBrand, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert_eq!(*log.borrow(), vec!["[firstsecond]".to_string(), "outer".to_string()]);
		/// ```
		#[inline]
		pub fn censor<LogType: 'static, Idx>(
			censor: impl Fn(LogType) -> LogType + 'static,
			action: RcRun<R, ScopedRow, A>,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawRcRunFree<R, ScopedRow>,
			>): Member<
					crate::types::effects::writer::WriterCensor<
						'static,
						RcBrand,
						LogType,
						RawRcRunFree<R, ScopedRow>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, RcTypeErasedValue>,
			>): Clone, {
			let writer: crate::types::effects::writer::WriterCensor<
				'static,
				RcBrand,
				LogType,
				RawRcRunFree<R, ScopedRow>,
			> = crate::types::effects::writer::WriterCensor::Censor {
				censor: <RcBrand as crate::classes::ToDynCloneFn>::new(censor),
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					action.clone().into_rc_free().cast_erased()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawRcRunFree<R, ScopedRow>,
			>) as Member<
				crate::types::effects::writer::WriterCensor<
					'static,
					RcBrand,
					LogType,
					RawRcRunFree<R, ScopedRow>,
				>,
				Idx,
			>>::inject(writer);
			let node = Node::Scoped(layer);
			let raw = RcFree::<NodeBrand<R, ScopedRow>, RcTypeErasedValue>::wrap(node);
			RcRun::from_rc_free(RcFree::continue_from_erased(
				raw,
				RcRunContinuations::<R, ScopedRow>::empty(),
			))
		}

		/// Lifts a neutral scoped Writer `listen` effect into the
		/// `RcRun` program.
		///
		/// The selected action result remains `A`; the standard Writer
		/// handler later pairs that action result with the observed log
		/// `LogType` before the outer continuation resumes.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type observed by `listen`.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The selected action program.")]
		///
		#[document_returns("An `RcRun` program suspended at the scoped Writer `listen` effect.")]
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
		/// 			rc_run::RcRun,
		/// 			standard_scoped_handlers::writer_post_handler,
		/// 			writer::Writer,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::RefCell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type ScopedRow = CoproductBrand<WriterListenBrand<RcBrand, String, i32>, CNilBrand>;
		/// type Prog = RcRun<FirstRow, ScopedRow, (i32, String)>;
		///
		/// let log = Rc::new(RefCell::new(Vec::new()));
		/// let log_for_handler = Rc::clone(&log);
		/// let action = RcRun::<FirstRow, ScopedRow, ()>::tell::<String, _>("first".to_string())
		/// 	.bind(|()| RcRun::<FirstRow, ScopedRow, ()>::tell::<String, _>("second".to_string()))
		/// 	.bind(|()| RcRun::pure(40));
		/// let program: Prog = RcRun::listen::<String, _>(action).bind(|(value, observed)| {
		/// 	RcRun::<FirstRow, ScopedRow, ()>::tell::<String, _>("outer".to_string())
		/// 		.bind(move |()| RcRun::pure((value + 2, observed.clone())))
		/// });
		///
		/// let result = program.handle(
		/// 	handlers! {
		/// 		WriterBrand<String>: move |op: Writer<'_, String, Prog>| match op {
		/// 			Writer::Tell(log, next, _) => {
		/// 				log_for_handler.borrow_mut().push(log);
		/// 				next
		/// 			}
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		WriterListenBrand<RcBrand, String, i32>:
		/// 			writer_post_handler::<_, CNilBrand, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, (42, "firstsecond".to_string()));
		/// assert_eq!(*log.borrow(), vec!["first".to_string(), "second".to_string(), "outer".to_string()],);
		/// ```
		#[inline]
		pub fn listen<LogType: Clone + 'static, Idx>(
			action: RcRun<R, ScopedRow, A>
		) -> RcRun<R, ScopedRow, (A, LogType)>
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawRcRunFree<R, ScopedRow>,
			>): Member<
					crate::types::effects::writer::WriterListen<
						'static,
						RcBrand,
						LogType,
						A,
						RawRcRunFree<R, ScopedRow>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, RcTypeErasedValue>,
			>): Clone, {
			let writer: crate::types::effects::writer::WriterListen<
				'static,
				RcBrand,
				LogType,
				A,
				RawRcRunFree<R, ScopedRow>,
			> = crate::types::effects::writer::WriterListen::Listen {
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					action.clone().into_rc_free().cast_erased()
				}),
				result: core::marker::PhantomData,
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawRcRunFree<R, ScopedRow>,
			>) as Member<
				crate::types::effects::writer::WriterListen<
					'static,
					RcBrand,
					LogType,
					A,
					RawRcRunFree<R, ScopedRow>,
				>,
				Idx,
			>>::inject(writer);
			let node = Node::Scoped(layer);
			let raw = RcFree::<NodeBrand<R, ScopedRow>, RcTypeErasedValue>::wrap(node);
			RcRun::from_rc_free(RcFree::continue_from_erased(
				raw,
				RcRunContinuations::<R, ScopedRow>::empty(),
			))
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The body's result type."
	)]
	impl<R, ScopedRow, B> RcRun<R, ScopedRow, B>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		B: 'static,
	{
		/// Lifts a [`Bracket`](crate::types::effects::bracket::Bracket)
		/// scoped resource-management effect into the `RcRun` program.
		/// Mirrors [`Run::bracket`](crate::types::effects::run::Run::bracket)
		/// for the multi-shot Rc-substrate. The cell stores the three
		/// closures behind `Rc<dyn Fn>` pointers, so `body` and `release`
		/// are `Fn` (multi-shot) closures and the resource is wrapped in
		/// `Rc<A>` so it can be shared across calls.
		///
		/// `Idx` is the type-level position witness identifying where
		/// `BracketBrand<RcBrand, NodeBrand<R, ScopedRow>, A, B>` lives
		/// in `ScopedRow`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The resource type produced by acquire.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The acquire program (produces the resource).",
			"The body closure (receives the resource as `Rc<A>` and returns a paired program).",
			"The release closure (receives the resource as `Rc<A>` and returns a unit program)."
		)]
		///
		#[document_returns("An `RcRun` program suspended at the scoped `Bracket` effect.")]
		///
		#[document_examples]
		///
		/// User-facing scoped rows containing
		/// [`BracketBrand`](crate::brands::BracketBrand) cannot be
		/// defined as type aliases (Rust rejects the recursion). Use the
		/// marker-struct workaround: a zero-sized struct that breaks the
		/// type-alias cycle by hosting the recursive references inside
		/// `impl_kind!` and trait impl bodies.
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			standard_scoped_handlers::bracket_handler,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::RefCell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type UnderlyingRow =
		/// 	CoproductBrand<BracketBrand<RcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>, CNilBrand>;
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
		/// let events = Rc::new(RefCell::new(Vec::new()));
		/// let acquire_events = Rc::clone(&events);
		/// let body_events = Rc::clone(&events);
		/// let release_events = Rc::clone(&events);
		///
		/// let acquire: RcRun<FirstRow, ScopedRow, i32> = RcRun::pure(7).bind(move |resource| {
		/// 	acquire_events.borrow_mut().push("acquire");
		/// 	RcRun::pure(resource)
		/// });
		/// let program: RcRun<FirstRow, ScopedRow, i32> =
		/// 	RcRun::<FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 		acquire,
		/// 		move |resource: Rc<i32>| {
		/// 			body_events.borrow_mut().push("body");
		/// 			RcRun::pure((*resource, *resource + 35))
		/// 		},
		/// 		move |resource: Rc<i32>| {
		/// 			release_events.borrow_mut().push("release");
		/// 			assert_eq!(*resource, 7);
		/// 			RcRun::pure(())
		/// 		},
		/// 	);
		///
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>:
		/// 			bracket_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert_eq!(events.borrow().as_slice(), ["acquire", "body", "release"]);
		/// ```
		#[inline]
		pub fn bracket<A, Idx>(
			acquire: RcRun<R, ScopedRow, A>,
			body: impl Fn(
				<RcBrand as crate::classes::Pointer>::Of<'static, A>,
			) -> RcRun<R, ScopedRow, (A, B)>
			+ 'static,
			release: impl Fn(
				<RcBrand as crate::classes::Pointer>::Of<'static, A>,
			) -> RcRun<R, ScopedRow, ()>
			+ 'static,
		) -> Self
		where
			A: 'static,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, B>,
			>): Member<
					crate::types::effects::bracket::Bracket<
						'static,
						RcBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let bracket: crate::types::effects::bracket::Bracket<
				'static,
				RcBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::bracket::Bracket::Bracket {
				acquire: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					acquire.clone().into_rc_free()
				}),
				body: <RcBrand as crate::classes::ToDynCloneFn>::new(
					move |a: <RcBrand as crate::classes::Pointer>::Of<'static, A>| {
						body(a).into_rc_free()
					},
				),
				release: <RcBrand as crate::classes::ToDynCloneFn>::new(
					move |a: <RcBrand as crate::classes::Pointer>::Of<'static, A>| {
						release(a).into_rc_free()
					},
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, B>,
			>) as Member<
				crate::types::effects::bracket::Bracket<
					'static,
					RcBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			let node = Node::Scoped(layer);
			RcRun::from_rc_free(RcFree::wrap(node))
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The body's result type."
	)]
	impl<R, ScopedRow, B> RcRun<R, ScopedRow, B>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		B: 'static,
	{
		/// Lifts a [`RefBracket`](crate::types::effects::ref_bracket::RefBracket)
		/// scoped resource-management effect into the `RcRun` program.
		/// This is the Ref flavour of [`RcRun::bracket`]: `acquire`
		/// produces the resource, while `body` and `release` both
		/// receive independent `Rc<A>` clones.
		#[document_signature]
		///
		#[document_type_parameters(
			"The resource type produced by acquire.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The acquire program (produces the resource).",
			"The body closure (receives the resource as `Rc<A>` and returns the body result program).",
			"The release closure (receives the resource as `Rc<A>` and returns a unit program)."
		)]
		///
		#[document_returns("An `RcRun` program suspended at the scoped `RefBracket` effect.")]
		#[document_examples]
		///
		/// User-facing scoped rows containing
		/// [`RefBracketBrand`](crate::brands::RefBracketBrand) cannot be
		/// defined as type aliases (Rust rejects the recursion). Use the
		/// marker-struct workaround: a zero-sized struct that breaks the
		/// type-alias cycle by hosting the recursive references inside
		/// `impl_kind!` and trait impl bodies.
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			standard_scoped_handlers::ref_bracket_handler,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::RefCell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type UnderlyingRow = CoproductBrand<
		/// 	RefBracketBrand<RcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>,
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
		/// let events = Rc::new(RefCell::new(Vec::new()));
		/// let acquire_events = Rc::clone(&events);
		/// let body_events = Rc::clone(&events);
		/// let release_events = Rc::clone(&events);
		///
		/// let acquire: RcRun<FirstRow, ScopedRow, i32> = RcRun::pure(7).bind(move |resource| {
		/// 	acquire_events.borrow_mut().push("acquire");
		/// 	RcRun::pure(resource)
		/// });
		/// let program: RcRun<FirstRow, ScopedRow, i32> =
		/// 	RcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 		acquire,
		/// 		move |resource: Rc<i32>| {
		/// 			body_events.borrow_mut().push("body");
		/// 			RcRun::pure(*resource + 35)
		/// 		},
		/// 		move |resource: Rc<i32>| {
		/// 			release_events.borrow_mut().push("release");
		/// 			assert_eq!(*resource, 7);
		/// 			RcRun::pure(())
		/// 		},
		/// 	);
		///
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>:
		/// 			ref_bracket_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert_eq!(events.borrow().as_slice(), ["acquire", "body", "release"]);
		/// ```
		#[inline]
		pub fn ref_bracket<A: 'static, Idx>(
			acquire: RcRun<R, ScopedRow, A>,
			body: impl Fn(
				<RcBrand as crate::classes::RefCountedPointer>::Of<'static, A>,
			) -> RcRun<R, ScopedRow, B>
			+ 'static,
			release: impl Fn(
				<RcBrand as crate::classes::RefCountedPointer>::Of<'static, A>,
			) -> RcRun<R, ScopedRow, ()>
			+ 'static,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, B>,
			>): Member<
					crate::types::effects::ref_bracket::RefBracket<
						'static,
						RcBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let bracket: crate::types::effects::ref_bracket::RefBracket<
				'static,
				RcBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::ref_bracket::RefBracket::Bracket {
				acquire: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					acquire.clone().into_rc_free()
				}),
				body: <RcBrand as crate::classes::ToDynCloneFn>::new(
					move |a: <RcBrand as crate::classes::RefCountedPointer>::Of<'static, A>| {
						body(a).into_rc_free()
					},
				),
				release: <RcBrand as crate::classes::ToDynCloneFn>::new(
					move |a: <RcBrand as crate::classes::RefCountedPointer>::Of<'static, A>| {
						release(a).into_rc_free()
					},
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, B>,
			>) as Member<
				crate::types::effects::ref_bracket::RefBracket<
					'static,
					RcBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			let node = Node::Scoped(layer);
			RcRun::from_rc_free(RcFree::wrap(node))
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, ScopedRow> RcRun<R, ScopedRow, ()>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		define_run_wrapper! {
			wrapper RcRun;
			effect State;
			method put;
		}

		/// Lifts a `Tell` writer effect into the `RcRun` program.
		/// Mirrors [`Run::tell`](crate::types::effects::run::Run::tell);
		/// see that method for cross-wrapper semantics.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type carried by `WriterBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The log value to emit.")]
		///
		#[document_returns("An `RcRun` program suspended at the lifted `Tell` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, ()> = RcRun::tell::<String, _>("logged".to_string());
		/// let handled: RcRun<CNilBrand, CNilBrand, ((), String)> =
		/// 	prog.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), "logged".to_string()));
		/// ```
		#[inline]
		pub fn tell<LogType: Clone + 'static, Idx>(log: LogType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				Member<RcCoyoneda<'static, crate::brands::WriterBrand<LogType>, ()>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::writer::Writer<'static, LogType, ()> =
				crate::types::effects::writer::Writer::Tell(log, (), core::marker::PhantomData);
			Self::lift::<crate::brands::WriterBrand<LogType>, Idx>(effect)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, ScopedRow> RcRun<R, ScopedRow, bool>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		/// Lifts an `Alt` choose effect into the `RcRun` program.
		/// Direct analog of PureScript Run's `choose` /
		/// `runChoose`. The program nondeterministically returns
		/// `true` or `false`; the handler runs the continuation
		/// twice (once per branch) to capture both outcomes.
		///
		/// `Choose` ships only on the four multi-shot wrappers
		/// because the handler must clone the continuation to invoke
		/// it twice. Threads
		/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `RcRun` program suspended at the lifted `Alt` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::<FirstRow, Scoped, bool>::choose()
		/// 	.bind(|branch| RcRun::<FirstRow, Scoped, i32>::pure(if branch { 1 } else { 0 }));
		/// let handled: RcRun<CNilBrand, CNilBrand, Vec<i32>> = prog.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, bool>):
				Member<RcCoyoneda<'static, crate::brands::ChooseBrand<RcBrand>, bool>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::choose::Choose<'static, RcBrand, bool> =
				crate::types::effects::choose::Choose::Alt(
					<RcBrand as crate::classes::ToDynCloneFn>::new(|b: bool| b),
				);
			Self::lift::<crate::brands::ChooseBrand<RcBrand>, Idx>(effect)
		}
	}
}

#[expect(
	unused_imports,
	reason = "inherent impl modules follow the document_module re-export boundary"
)]
pub use inner::*;
