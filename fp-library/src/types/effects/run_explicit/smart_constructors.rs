#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		super::super::inner::{
			RunExplicit,
			RunExplicitBoundary,
		},
		crate::{
			Apply,
			brands::NodeBrand,
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::*,
			types::{
				Coyoneda,
				effects::member::Member,
			},
		},
		fp_macros::*,
	};

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The stored value type."
	)]
	impl<'a, R, ScopedRow, V> RunExplicit<'a, R, ScopedRow, Option<V>>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		V: 'static + 'a,
	{
		define_run_wrapper! {
			wrapper RunExplicit;
			effect KVStore;
			method lookup;
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	impl<'a, R, ScopedRow> RunExplicit<'a, R, ScopedRow, ()>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		define_run_wrapper! {
			wrapper RunExplicit;
			effect KVStore;
			method update;
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The state type (also the program's result type for `get`)."
	)]
	#[document_parameters("The `RunExplicit` instance.")]
	impl<'a, R, ScopedRow, A: 'a> RunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		define_run_wrapper! {
			wrapper RunExplicit;
			effect Reader;
			method ask;
		}

		define_run_wrapper! {
			wrapper RunExplicit;
			effect State;
			method get;
		}

		define_run_wrapper! {
			wrapper RunExplicit;
			effect Fresh;
			method fresh;
		}

		define_run_wrapper! {
			wrapper RunExplicit;
			effect Input;
			method input;
		}

		define_run_wrapper_method! {
			wrapper RunExplicit;
			method expand;
		}

		define_run_wrapper_method! {
			wrapper RunExplicit;
			method weaken;
		}

		/// Lifts a `Throw` except effect into the `RunExplicit`
		/// program. Mirrors
		/// [`Run::throw`](crate::types::effects::run::Run::throw);
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
		#[document_returns("A `RunExplicit` program suspended at the lifted `Throw` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RunExplicit::throw::<&'static str, _>("oops");
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	prog.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("oops"));
		/// ```
		#[inline]
		pub fn throw<ErrorType: 'static, Idx>(e: ErrorType) -> Self
		where
			A: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, crate::brands::ExceptBrand<ErrorType>, A>, Idx>, {
			let effect: crate::types::effects::except::Except<'a, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}

		/// Lifts an `Empty` effect into the `RunExplicit` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning a fallback value in a
		/// single-shot interpreter.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("A `RunExplicit` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> = RunExplicit::empty();
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			A: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, crate::brands::EmptyBrand, A>, Idx>, {
			let effect: crate::types::effects::empty::Empty<'a, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}

		/// Constructs an indexed scoped `Catch` boundary for a protected
		/// `RunExplicit` action.
		///
		/// The boundary stores the selected action and recovery handler in
		/// the scoped row layer as `Box<dyn FnOnce(...) -> _>` thunks over
		/// the explicit `'a` lifetime. Mapping or binding the boundary
		/// composes only the outer continuation; the selected action slot
		/// remains unchanged until a Catch dispatcher resumes it.
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
		#[document_returns("A `RunExplicit` Catch boundary over the selected action.")]
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
		/// 		run_explicit::RunExplicit,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, &'static str>, CNilBrand>;
		/// type Prog = RunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = RunExplicit::throw::<&'static str, _>("boom");
		/// let boundary = RunExplicit::catch::<&'static str, _>(action, |_err| RunExplicit::pure(41))
		/// 	.map(|value| value + 1);
		/// let program: Prog = catch_handler::<_, FirstRowMinusExcept, _>()
		/// 	.dispatch_run_explicit_catch_boundary(boundary, &handlers! {});
		/// let result = program.handle(
		/// 	handlers! {
		/// 		ExceptBrand<&'static str>: |_op: Except<'_, &'static str, Prog>| {
		/// 			RunExplicit::pure(-1)
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, FirstRowMinusExcept, _>(),
		/// 	},
		/// );
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl FnOnce continuation in a reusable alias."
		)]
		pub fn catch<E: 'a, Idx>(
			action: RunExplicit<'a, R, ScopedRow, A>,
			handler: impl FnOnce(E) -> RunExplicit<'a, R, ScopedRow, A> + 'a,
		) -> RunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::BoxCatchBrand<crate::brands::BoxBrand, E>,
			Idx,
			A,
			A,
			impl Fn(A) -> RunExplicit<'a, R, ScopedRow, A> + 'a,
		>
		where
			A: 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::catch::BoxCatch<
						'a,
						crate::brands::BoxBrand,
						E,
						RunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				>, {
			let catch: crate::types::effects::catch::BoxCatch<
				'a,
				crate::brands::BoxBrand,
				E,
				RunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::catch::BoxCatch::Catch {
				action: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| action,
				),
				handler: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |e: E| handler(e),
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::catch::BoxCatch<
					'a,
					crate::brands::BoxBrand,
					E,
					RunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(catch);
			RunExplicitBoundary::new(layer, RunExplicit::pure)
		}

		/// Constructs an indexed scoped `Local` boundary for a protected
		/// `RunExplicit` action.
		///
		/// During dispatch, the protected action runs under an environment
		/// value transformed by `modify`. The boundary stores `modify` and
		/// the selected action as `Box<dyn FnOnce(...) -> _>` thunks over
		/// the explicit `'a` lifetime. Mapping or binding the boundary
		/// composes only the outer continuation.
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
		#[document_returns("A `RunExplicit` Local boundary over the selected action.")]
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
		/// 		run_explicit::RunExplicit,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = RunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog =
		/// 	RunExplicit::<FirstRow, ScopedRow, i32>::ask::<_>().bind(|env| RunExplicit::pure(env * 2));
		/// let boundary = RunExplicit::local::<i32, _>(|env| env + 1, action);
		/// let program: Prog = local_handler::<_, FirstRowMinusReader, _>()
		/// 	.dispatch_run_explicit_local_boundary(boundary, &handlers! {});
		/// let result = program.handle(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxLocalBrand<BoxBrand, i32>: local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		/// assert_eq!(result, 22);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl FnOnce continuation in a reusable alias."
		)]
		pub fn local<E: 'a, Idx>(
			modify: impl FnOnce(E) -> E + 'a,
			action: RunExplicit<'a, R, ScopedRow, A>,
		) -> RunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::BoxLocalBrand<crate::brands::BoxBrand, E>,
			Idx,
			A,
			A,
			impl Fn(A) -> RunExplicit<'a, R, ScopedRow, A> + 'a,
		>
		where
			A: 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::local::BoxLocal<
						'a,
						crate::brands::BoxBrand,
						E,
						RunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				>, {
			let local: crate::types::effects::local::BoxLocal<
				'a,
				crate::brands::BoxBrand,
				E,
				RunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::local::BoxLocal::Local {
				modify: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |e: E| modify(e),
				),
				action: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| action,
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::local::BoxLocal<
					'a,
					crate::brands::BoxBrand,
					E,
					RunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(local);
			RunExplicitBoundary::new(layer, RunExplicit::pure)
		}

		/// Constructs an indexed scoped `RefLocal` boundary for a protected
		/// `RunExplicit` action.
		///
		/// During dispatch, the protected action runs under an environment
		/// value computed by borrowing the inherited environment with
		/// `modify`. The boundary stores `modify` and the selected action
		/// as `Box<dyn FnOnce(...) -> _>` thunks over the explicit `'a`
		/// lifetime. Mapping or binding the boundary composes only the
		/// outer continuation.
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
		#[document_returns("A `RunExplicit` RefLocal boundary over the selected action.")]
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
		/// 		run_explicit::RunExplicit,
		/// 		standard_scoped_handlers::ref_local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>;
		/// type Prog = RunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog =
		/// 	RunExplicit::<FirstRow, ScopedRow, i32>::ask::<_>().bind(|env| RunExplicit::pure(env * 2));
		/// let boundary = RunExplicit::ref_local::<i32, _>(|env| *env + 5, action);
		/// let program: Prog = ref_local_handler::<_, FirstRowMinusReader, _>()
		/// 	.dispatch_run_explicit_ref_local_boundary(boundary, &handlers! {});
		/// let result = program.handle(
		/// 	handlers! {
		/// 		BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, Prog>| match op {
		/// 			BoxReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		/// assert_eq!(result, 30);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl FnOnce continuation in a reusable alias."
		)]
		pub fn ref_local<E: 'a, Idx>(
			modify: impl FnOnce(&E) -> E + 'a,
			action: RunExplicit<'a, R, ScopedRow, A>,
		) -> RunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::BoxRefLocalBrand<crate::brands::BoxBrand, E>,
			Idx,
			A,
			A,
			impl Fn(A) -> RunExplicit<'a, R, ScopedRow, A> + 'a,
		>
		where
			A: 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::ref_local::BoxRefLocal<
						'a,
						crate::brands::BoxBrand,
						E,
						RunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				>, {
			let local: crate::types::effects::ref_local::BoxRefLocal<
				'a,
				crate::brands::BoxBrand,
				E,
				RunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::ref_local::BoxRefLocal::Local {
				modify: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::ref_new(
					move |e: &E| modify(e),
				),
				action: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| action,
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::ref_local::BoxRefLocal<
					'a,
					crate::brands::BoxBrand,
					E,
					RunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(local);
			RunExplicitBoundary::new(layer, RunExplicit::pure)
		}

		/// Constructs an indexed scoped `Span` boundary for a protected
		/// `RunExplicit` action.
		///
		/// The returned boundary keeps the selected action in the scoped
		/// row layer and stores the outer continuation separately. Mapping
		/// or binding the boundary changes only that outer continuation;
		/// the Span layer remains typed by the selected action result until
		/// a Span dispatcher resumes it.
		#[document_signature]
		///
		#[document_type_parameters(
			"The instrumentation tag type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The instrumentation tag.", "The protected action program.")]
		///
		#[document_returns("A `RunExplicit` Span boundary over the selected action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::effects::{
		/// 		run_explicit::RunExplicit,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
		///
		/// let action: RunExplicit<'static, FirstRow, ScopedRow, i32> = RunExplicit::pure(42);
		/// let boundary = RunExplicit::span::<&'static str, _>("request", action).map(|value| value + 1);
		/// let program = span_handler().dispatch_run_explicit_span_boundary_with_post_action(
		/// 	boundary,
		/// 	&handlers! {},
		/// 	|tag, value| {
		/// 		assert_eq!(*tag, "request");
		/// 		RunExplicit::pure(value)
		/// 	},
		/// );
		/// assert!(matches!(program.peel(), Ok(43)));
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl FnOnce continuation in a reusable alias."
		)]
		pub fn span<Tag: 'a, Idx>(
			tag: Tag,
			action: RunExplicit<'a, R, ScopedRow, A>,
		) -> RunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::BoxSpanBrand<crate::brands::BoxBrand, Tag>,
			Idx,
			A,
			A,
			impl Fn(A) -> RunExplicit<'a, R, ScopedRow, A> + 'a,
		>
		where
			A: 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::span::BoxSpan<
						'a,
						crate::brands::BoxBrand,
						Tag,
						RunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				>, {
			let span: crate::types::effects::span::BoxSpan<
				'a,
				crate::brands::BoxBrand,
				Tag,
				RunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::span::BoxSpan::Span {
				tag,
				action: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| action,
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::span::BoxSpan<
					'a,
					crate::brands::BoxBrand,
					Tag,
					RunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(span);
			RunExplicitBoundary::new(layer, RunExplicit::pure)
		}

		/// Lifts a neutral scoped Writer `censor` effect into the
		/// `RunExplicit` indexed-boundary surface.
		///
		/// The constructor stores the selected action and log
		/// transformation, while standard Writer handlers later choose
		/// whether the transformation is applied before or after log
		/// accumulation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type transformed by `censor`.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The log transformation.", "The selected action program.")]
		///
		#[document_returns("A `RunExplicit` boundary suspended at scoped Writer `censor`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxWriterCensorBrand<BoxBrand, String>, CNilBrand>;
		///
		/// let preview: RunExplicit<'static, FirstRow, ScopedRow, i32> = RunExplicit::pure(42);
		/// assert!(matches!(preview.peel(), Ok(42)));
		///
		/// let action: RunExplicit<'static, FirstRow, ScopedRow, i32> = RunExplicit::pure(42);
		/// let boundary =
		/// 	RunExplicit::censor::<String, _>(|log| format!("{log}!"), action).map(|value| value + 1);
		/// let _ = boundary;
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl FnOnce continuation in a reusable alias."
		)]
		pub fn censor<LogType: 'static, Idx>(
			censor: impl Fn(LogType) -> LogType + 'a,
			action: RunExplicit<'a, R, ScopedRow, A>,
		) -> RunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::BoxWriterCensorBrand<crate::brands::BoxBrand, LogType>,
			Idx,
			A,
			A,
			impl Fn(A) -> RunExplicit<'a, R, ScopedRow, A> + 'a,
		>
		where
			A: 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::writer::BoxWriterCensor<
						'a,
						crate::brands::BoxBrand,
						LogType,
						RunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				>, {
			let writer: crate::types::effects::writer::BoxWriterCensor<
				'a,
				crate::brands::BoxBrand,
				LogType,
				RunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::writer::BoxWriterCensor::Censor {
				censor: <crate::brands::BoxBrand as crate::classes::ToDynFn>::new(censor),
				action: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| action,
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::writer::BoxWriterCensor<
					'a,
					crate::brands::BoxBrand,
					LogType,
					RunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(writer);
			RunExplicitBoundary::new(layer, RunExplicit::pure)
		}

		/// Lifts a neutral scoped Writer `listen` effect into the
		/// `RunExplicit` indexed-boundary surface.
		///
		/// The selected action result remains `A`; the boundary's
		/// operation-result slot is `(A, LogType)`, which the standard
		/// Writer handler produces before mapped or bound outer
		/// continuations resume.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type observed by `listen`.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The selected action program.")]
		///
		#[document_returns("A `RunExplicit` boundary suspended at scoped Writer `listen`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<BoxWriterListenBrand<BoxBrand, String, i32>, CNilBrand>;
		///
		/// let preview: RunExplicit<'static, FirstRow, ScopedRow, i32> = RunExplicit::pure(42);
		/// assert!(matches!(preview.peel(), Ok(42)));
		///
		/// let action: RunExplicit<'static, FirstRow, ScopedRow, i32> = RunExplicit::pure(42);
		/// let boundary = RunExplicit::listen::<String, _>(action).map(|(value, log)| (value + 1, log));
		/// let _ = boundary;
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Writer listen preserves selected action, operation result, final result, and the private continuation type in one indexed boundary."
		)]
		pub fn listen<LogType: 'static, Idx>(
			action: RunExplicit<'a, R, ScopedRow, A>
		) -> RunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::BoxWriterListenBrand<crate::brands::BoxBrand, LogType, A>,
			Idx,
			A,
			(A, LogType),
			impl Fn((A, LogType)) -> RunExplicit<'a, R, ScopedRow, (A, LogType)> + 'a,
			(A, LogType),
		>
		where
			A: 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::writer::BoxWriterListen<
						'a,
						crate::brands::BoxBrand,
						LogType,
						A,
						RunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				>, {
			let writer: crate::types::effects::writer::BoxWriterListen<
				'a,
				crate::brands::BoxBrand,
				LogType,
				A,
				RunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::writer::BoxWriterListen::Listen {
				action: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| action,
				),
				result: core::marker::PhantomData,
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::writer::BoxWriterListen<
					'a,
					crate::brands::BoxBrand,
					LogType,
					A,
					RunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(writer);
			RunExplicitBoundary::new(layer, |operation: (A, LogType)| RunExplicit::pure(operation))
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The body's result type."
	)]
	impl<'a, R, ScopedRow, B> RunExplicit<'a, R, ScopedRow, B>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		B: 'a,
	{
		/// Constructs an indexed scoped `Bracket` boundary for a
		/// `RunExplicit` resource lifecycle.
		///
		/// During dispatch, `acquire` produces the resource, `body` runs
		/// with a `Box<A>` resource pointer, and `release` runs before the
		/// outer continuation observes the body result. The boundary stores
		/// the lifecycle cells in the scoped row layer and composes mapping
		/// or binding through the outer continuation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The resource type produced by acquire.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The acquire program (produces the resource).",
			"The body closure (consumes the resource as `Box<A>` and returns a paired program).",
			"The release closure (consumes the resource as `Box<A>` and returns a unit program)."
		)]
		///
		#[document_returns("A `RunExplicit` Bracket boundary over the lifecycle-generated action.")]
		///
		#[document_examples]
		///
		/// User-facing scoped rows containing
		/// [`BoxBracketExplicitBrand`](crate::brands::BoxBracketExplicitBrand)
		/// should use a named marker row. The marker gives
		/// [`NodeBrand`](crate::brands::NodeBrand) a concrete scoped-row
		/// type without a recursive type alias.
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::*,
		/// 	classes::{
		/// 		Functor,
		/// 		WrapDrop,
		/// 	},
		/// 	handlers,
		/// 	impl_kind,
		/// 	kinds::*,
		/// 	types::effects::{
		/// 		run_explicit::RunExplicit,
		/// 		standard_scoped_handlers::bracket_handler,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type UnderlyingRow = CoproductBrand<
		/// 	BoxBracketExplicitBrand<BoxBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>,
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
		/// let acquire: RunExplicit<'static, FirstRow, ScopedRow, i32> = RunExplicit::pure(7);
		/// let boundary = RunExplicit::<'static, FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 	acquire,
		/// 	|resource: Box<i32>| RunExplicit::pure((*resource, 42)),
		/// 	|_resource: Box<i32>| RunExplicit::pure(()),
		/// );
		/// let program: RunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	bracket_handler().dispatch_run_explicit_bracket_boundary(boundary, &handlers! {});
		/// assert!(matches!(program.peel(), Ok(42)));
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying lifecycle state, the selected row member, and an opaque continuation closure; stable Rust cannot name this impl FnOnce continuation in a reusable alias."
		)]
		pub fn bracket<A, Idx>(
			acquire: RunExplicit<'a, R, ScopedRow, A>,
			body: impl FnOnce(
				<crate::brands::BoxBrand as crate::classes::Pointer>::Of<'a, A>,
			) -> RunExplicit<'a, R, ScopedRow, (A, B)>
			+ 'a,
			release: impl FnOnce(
				<crate::brands::BoxBrand as crate::classes::Pointer>::Of<'a, A>,
			) -> RunExplicit<'a, R, ScopedRow, ()>
			+ 'a,
		) -> RunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::BoxBracketExplicitBrand<
				crate::brands::BoxBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			>,
			Idx,
			B,
			B,
			impl Fn(B) -> RunExplicit<'a, R, ScopedRow, B> + 'a,
		>
		where
			A: 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, B>,
			>): Member<
					crate::types::effects::bracket::BoxBracketExplicit<
						'a,
						crate::brands::BoxBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				>, {
			let acquire_free = Box::new(acquire.into_free_explicit());
			let bracket: crate::types::effects::bracket::BoxBracketExplicit<
				'a,
				crate::brands::BoxBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::bracket::BoxBracketExplicit::Bracket {
				acquire: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| acquire_free,
				),
				body: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |a: <crate::brands::BoxBrand as crate::classes::Pointer>::Of<'a, A>| {
						Box::new(body(a).into_free_explicit())
					},
				),
				release: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |a: <crate::brands::BoxBrand as crate::classes::Pointer>::Of<'a, A>| {
						Box::new(release(a).into_free_explicit())
					},
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, ScopedRow, B>,
			>) as Member<
				crate::types::effects::bracket::BoxBracketExplicit<
					'a,
					crate::brands::BoxBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			RunExplicitBoundary::new(layer, RunExplicit::pure)
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	impl<'a, R, ScopedRow> RunExplicit<'a, R, ScopedRow, ()>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		define_run_wrapper! {
			wrapper RunExplicit;
			effect State;
			method put;
		}

		/// Lifts a `Tell` writer effect into the `RunExplicit`
		/// program. Mirrors
		/// [`Run::tell`](crate::types::effects::run::Run::tell); see
		/// that method for cross-wrapper semantics.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type carried by `WriterBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The log value to emit.")]
		///
		#[document_returns("A `RunExplicit` program suspended at the lifted `Tell` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, ()> =
		/// 	RunExplicit::tell::<String, _>("logged".to_string());
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, ((), String)> =
		/// 	prog.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), "logged".to_string()));
		/// ```
		#[inline]
		pub fn tell<LogType: 'static, Idx>(log: LogType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<Coyoneda<'a, crate::brands::WriterBrand<LogType>, ()>, Idx>, {
			let effect: crate::types::effects::writer::Writer<'a, LogType, ()> =
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
