#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		super::super::inner::{
			RawRunFree,
			Run,
			RunRepresentation,
			RunScopedBoundaryFrame,
		},
		crate::{
			Apply,
			brands::NodeBrand,
			kinds::*,
			types::{
				CatList,
				effects::node::Node,
			},
		},
		core::marker::PhantomData,
		fp_macros::*,
	};

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The stored value type."
	)]
	impl<R, ScopedRow, V> Run<R, ScopedRow, Option<V>>
	where
		R: crate::classes::WrapDrop + crate::classes::Functor + 'static,
		ScopedRow: crate::classes::WrapDrop + crate::classes::Functor + 'static,
		V: 'static,
	{
		define_run_wrapper! {
			wrapper Run;
			effect KVStore;
			method lookup;
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, ScopedRow> Run<R, ScopedRow, ()>
	where
		R: crate::classes::WrapDrop + crate::classes::Functor + 'static,
		ScopedRow: crate::classes::WrapDrop + crate::classes::Functor + 'static,
	{
		define_run_wrapper! {
			wrapper Run;
			effect KVStore;
			method update;
		}

		define_run_wrapper! {
			wrapper Run;
			effect Output;
			method output;
		}

		define_run_wrapper! {
			wrapper Run;
			effect Log;
			method log;
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The state type (also the program's result type for `get`)."
	)]
	#[document_parameters("The Run instance.")]
	impl<R, ScopedRow, A> Run<R, ScopedRow, A>
	where
		R: crate::classes::WrapDrop + crate::classes::Functor + 'static,
		ScopedRow: crate::classes::WrapDrop + crate::classes::Functor + 'static,
		A: 'static,
	{
		define_run_wrapper! {
			wrapper Run;
			effect Reader;
			method ask;
		}

		define_run_wrapper! {
			wrapper Run;
			effect State;
			method get;
		}

		define_run_wrapper! {
			wrapper Run;
			effect Fresh;
			method fresh;
		}

		define_run_wrapper! {
			wrapper Run;
			effect Input;
			method input;
		}

		define_run_wrapper! {
			wrapper Run;
			effect Coroutine;
			method yield_value;
		}

		define_run_wrapper_method! {
			wrapper Run;
			method expand;
		}

		define_run_wrapper_method! {
			wrapper Run;
			method weaken;
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
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::throw::<&'static str, _>("oops");
		/// let handled: Run<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	prog.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("oops"));
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

		/// Lifts an `Empty` effect into the Run program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// nondeterministic interpreter.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("A `Run` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::empty();
		/// let handled: Run<CNilBrand, CNilBrand, Option<i32>> = prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				crate::types::effects::member::Member<
						crate::types::Coyoneda<'static, crate::brands::EmptyBrand, A>,
						Idx,
					>, {
			let effect: crate::types::effects::empty::Empty<'static, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
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
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		except::Except,
		/// 		run::Run,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, &'static str>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::throw::<&'static str, _>("boom");
		/// let program: Prog = Run::catch::<&'static str, _>(action, |error| {
		/// 	assert_eq!(error, "boom");
		/// 	Run::pure(42)
		/// });
		///
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ExceptBrand<&'static str>: |_op: Except<'_, &'static str, Prog>| {
		/// 			Run::pure(-1)
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxCatchBrand<BoxBrand, &'static str>:
		/// 			catch_handler::<_, FirstRowMinusExcept, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
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
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		reader::BoxReader,
		/// 		run::Run,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env| Run::pure(env * 2));
		/// let program: Prog = Run::local::<i32, _>(|env| env + 1, action);
		///
		/// let result = program.handle(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| {
		/// 			match op {
		/// 				BoxReader::Ask(k) => k(10),
		/// 			}
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 22);
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
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		reader::BoxReader,
		/// 		run::Run,
		/// 		standard_scoped_handlers::ref_local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = Run::<FirstRow, ScopedRow, i32>::ask().bind(|env| Run::pure(env * 2));
		/// let program: Prog = Run::ref_local::<i32, _>(|env| *env + 5, action);
		///
		/// let result = program.handle(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| {
		/// 			match op {
		/// 				BoxReader::Ask(k) => k(10),
		/// 			}
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 30);
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
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let program: Prog =
		/// 	Run::span::<&'static str, _>("outer", Run::span::<&'static str, _>("inner", Run::pure(42)));
		///
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxSpanBrand<BoxBrand, &'static str>: span_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
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

		/// Lifts a neutral scoped Writer `censor` effect into the
		/// `Run` program.
		///
		/// The constructor stores the selected action and the log
		/// transformation without choosing whether the transformation is
		/// applied before or after log accumulation. Standard Writer
		/// handlers decide that ordering later.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type transformed by `censor`.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The log transformation.", "The selected action program.")]
		///
		#[document_returns("A `Run` program suspended at the scoped Writer `censor` effect.")]
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
		/// 			run::Run,
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
		/// type FirstRow = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type ScopedRow = CoproductBrand<BoxWriterCensorBrand<BoxBrand, String>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, i32>;
		///
		/// let log = Rc::new(RefCell::new(Vec::new()));
		/// let log_for_handler = Rc::clone(&log);
		/// let action: Prog = Run::<FirstRow, ScopedRow, ()>::tell::<String, _>("first".to_string())
		/// 	.bind(|()| Run::<FirstRow, ScopedRow, ()>::tell::<String, _>("second".to_string()))
		/// 	.bind(|()| Run::pure(40));
		/// let program: Prog = Run::censor::<String, _>(|log| format!("[{log}]"), action).bind(|value| {
		/// 	Run::<FirstRow, ScopedRow, ()>::tell::<String, _>("outer".to_string())
		/// 		.bind(move |()| Run::pure(value + 2))
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
		/// 		BoxWriterCensorBrand<BoxBrand, String>: writer_post_handler::<_, CNilBrand, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert_eq!(*log.borrow(), vec!["[firstsecond]".to_string(), "outer".to_string()]);
		/// ```
		#[inline]
		pub fn censor<LogType: 'static, Idx>(
			censor: impl Fn(LogType) -> LogType + 'static,
			action: Run<R, ScopedRow, A>,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawRunFree<R, ScopedRow>,
			>): crate::types::effects::member::Member<
					crate::types::effects::writer::BoxWriterCensor<
						'static,
						crate::brands::BoxBrand,
						LogType,
						RawRunFree<R, ScopedRow>,
					>,
					Idx,
				>, {
			let action_free = action.into_free().cast_erased();
			let writer: crate::types::effects::writer::BoxWriterCensor<
				'static,
				crate::brands::BoxBrand,
				LogType,
				RawRunFree<R, ScopedRow>,
			> = crate::types::effects::writer::BoxWriterCensor::Censor {
				censor: <crate::brands::BoxBrand as crate::classes::ToDynFn>::new(censor),
				action: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| action_free,
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawRunFree<R, ScopedRow>,
			>) as crate::types::effects::member::Member<
				crate::types::effects::writer::BoxWriterCensor<
					'static,
					crate::brands::BoxBrand,
					LogType,
					RawRunFree<R, ScopedRow>,
				>,
				Idx,
			>>::inject(writer);
			Run(RunRepresentation::ScopedBoundary(RunScopedBoundaryFrame {
				layer,
				continuations: CatList::empty(),
				result: PhantomData,
			}))
		}

		/// Lifts a neutral scoped Writer `listen` effect into the
		/// `Run` program.
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
		#[document_returns("A `Run` program suspended at the scoped Writer `listen` effect.")]
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
		/// 			run::Run,
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
		/// type FirstRow = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type ScopedRow = CoproductBrand<BoxWriterListenBrand<BoxBrand, String, i32>, CNilBrand>;
		/// type Prog = Run<FirstRow, ScopedRow, (i32, String)>;
		///
		/// let log = Rc::new(RefCell::new(Vec::new()));
		/// let log_for_handler = Rc::clone(&log);
		/// let action = Run::<FirstRow, ScopedRow, ()>::tell::<String, _>("first".to_string())
		/// 	.bind(|()| Run::<FirstRow, ScopedRow, ()>::tell::<String, _>("second".to_string()))
		/// 	.bind(|()| Run::pure(40));
		/// let program: Prog = Run::listen::<String, _>(action).bind(|(value, observed)| {
		/// 	Run::<FirstRow, ScopedRow, ()>::tell::<String, _>("outer".to_string())
		/// 		.bind(move |()| Run::pure((value + 2, observed)))
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
		/// 		BoxWriterListenBrand<BoxBrand, String, i32>:
		/// 			writer_post_handler::<_, CNilBrand, _>(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, (42, "firstsecond".to_string()));
		/// assert_eq!(*log.borrow(), vec!["first".to_string(), "second".to_string(), "outer".to_string()],);
		/// ```
		#[inline]
		pub fn listen<LogType: 'static, Idx>(
			action: Run<R, ScopedRow, A>
		) -> Run<R, ScopedRow, (A, LogType)>
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawRunFree<R, ScopedRow>,
			>): crate::types::effects::member::Member<
					crate::types::effects::writer::BoxWriterListen<
						'static,
						crate::brands::BoxBrand,
						LogType,
						A,
						RawRunFree<R, ScopedRow>,
					>,
					Idx,
				>, {
			let action_free = action.into_free().cast_erased();
			let writer: crate::types::effects::writer::BoxWriterListen<
				'static,
				crate::brands::BoxBrand,
				LogType,
				A,
				RawRunFree<R, ScopedRow>,
			> = crate::types::effects::writer::BoxWriterListen::Listen {
				action: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| action_free,
				),
				result: PhantomData,
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawRunFree<R, ScopedRow>,
			>) as crate::types::effects::member::Member<
				crate::types::effects::writer::BoxWriterListen<
					'static,
					crate::brands::BoxBrand,
					LogType,
					A,
					RawRunFree<R, ScopedRow>,
				>,
				Idx,
			>>::inject(writer);
			Run(RunRepresentation::ScopedBoundary(RunScopedBoundaryFrame {
				layer,
				continuations: CatList::empty(),
				result: PhantomData,
			}))
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
		/// 			run::Run,
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
		/// let events = Rc::new(RefCell::new(Vec::new()));
		/// let acquire_events = Rc::clone(&events);
		/// let body_events = Rc::clone(&events);
		/// let release_events = Rc::clone(&events);
		///
		/// let acquire: Run<FirstRow, ScopedRow, i32> = Run::pure(7).bind(move |resource| {
		/// 	acquire_events.borrow_mut().push("acquire");
		/// 	Run::pure(resource)
		/// });
		/// let program: Run<FirstRow, ScopedRow, i32> = Run::<FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 	acquire,
		/// 	move |resource: Box<i32>| {
		/// 		body_events.borrow_mut().push("body");
		/// 		Run::pure((*resource, *resource + 35))
		/// 	},
		/// 	move |resource: Box<i32>| {
		/// 		release_events.borrow_mut().push("release");
		/// 		assert_eq!(*resource, 7);
		/// 		Run::pure(())
		/// 	},
		/// );
		///
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		BoxBracketBrand<BoxBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>:
		/// 			bracket_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert_eq!(events.borrow().as_slice(), ["acquire", "body", "release"]);
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
		define_run_wrapper! {
			wrapper Run;
			effect State;
			method put;
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
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, ()> = Run::tell::<String, _>("logged".to_string());
		/// let handled: Run<CNilBrand, CNilBrand, ((), String)> =
		/// 	prog.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), "logged".to_string()));
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

#[expect(
	unused_imports,
	reason = "inherent impl modules follow the document_module re-export boundary"
)]
pub use inner::*;
