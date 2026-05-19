#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		super::super::inner::{
			ArcRunExplicit,
			ArcRunExplicitBoundary,
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
				ArcFreeExplicit,
				effects::member::Member,
			},
		},
		fp_macros::*,
	};
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The state type (also the program's result type for `get`)."
	)]
	impl<'a, R, ScopedRow, A: 'a> ArcRunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
	{
		/// Lifts a `Get` state effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::get`](crate::types::effects::run::Run::get); see
		/// that method for cross-wrapper semantics. Differences for
		/// `ArcRunExplicit`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendStateBrand`](crate::brands::SendStateBrand) (rather
		/// than `StateBrand`) so the continuation projection is
		/// structurally `Send + Sync`, which the `SendFunctor`
		/// algebra requires across thread boundaries.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Get` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::get();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, i32)> =
		/// 	prog.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), (41, 41));
		/// ```
		#[inline]
		pub fn get<Idx>() -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Member<
					ArcCoyoneda<'a, crate::brands::SendStateBrand<crate::brands::ArcBrand, A>, A>,
					Idx,
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::state::SendState<'a, crate::brands::ArcBrand, A, A> =
				crate::types::effects::state::SendState::Get(
					<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|s: A| s),
				);
			Self::lift::<crate::brands::SendStateBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}

		/// Lifts an `Ask` reader effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::ask`](crate::types::effects::run::Run::ask); see
		/// that method for cross-wrapper semantics. Differences for
		/// `ArcRunExplicit`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendReaderBrand`](crate::brands::SendReaderBrand) (rather
		/// than `ReaderBrand`) so the continuation projection is
		/// structurally `Send + Sync`.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Ask` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::ask();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, i32> =
		/// 	prog.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 41);
		/// ```
		#[inline]
		pub fn ask<Idx>() -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Member<
					ArcCoyoneda<'a, crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>, A>,
					Idx,
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::reader::SendReader<
				'a,
				crate::brands::ArcBrand,
				A,
				A,
			> = crate::types::effects::reader::SendReader::Ask(
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|e: A| e),
			);
			Self::lift::<crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}

		/// Lifts a `Throw` except effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::throw`](crate::types::effects::run::Run::throw);
		/// see that method for cross-wrapper semantics. The same
		/// [`ExceptBrand`](crate::brands::ExceptBrand) serves all six
		/// wrappers because [`Except`](crate::types::effects::except::Except)
		/// has no `dyn Fn` continuation; no parallel `SendExceptBrand`
		/// is needed. Requires `ErrorType: Send + Sync` so the lifted
		/// layer participates in the Arc substrate's thread-safety
		/// cascade.
		#[document_signature]
		///
		#[document_type_parameters(
			"The error type carried by `ExceptBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The error value to throw.")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Throw` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::throw::<&'static str, _>("oops");
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	prog.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("oops"));
		/// ```
		#[inline]
		pub fn throw<ErrorType: Clone + Send + Sync + 'static, Idx>(e: ErrorType) -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, crate::brands::ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::except::Except<'a, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}

		/// Lifts an `Empty` effect into the `ArcRunExplicit` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// thread-safe multi-shot nondeterministic interpreter.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::empty();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, crate::brands::EmptyBrand, A>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::empty::Empty<'a, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}

		/// Lifts a scoped `Catch` effect into the `ArcRunExplicit`
		/// program: run `action`, and if it throws an `E`, invoke
		/// `handler` with the error to produce a recovery program.
		/// Mirrors [`ArcRun::catch`](crate::types::effects::arc_run::ArcRun::catch);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRunExplicit`: the action and recovery handler are stored
		/// as `Arc<dyn Fn(...) -> _ + Send + Sync>` thunks (multi-shot,
		/// thread-safe) over the explicit `'a` lifetime.
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
		#[document_returns("An indexed `ArcRunExplicit` Catch boundary.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		except::Except,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendCatchBrand<ArcBrand, &'static str>, CNilBrand>;
		/// type Prog = ArcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = ArcRunExplicit::throw::<&'static str, _>("from-action");
		/// let boundary = ArcRunExplicit::catch::<&'static str, _>(action, |_e| ArcRunExplicit::pure(41))
		/// 	.map(|value| value + 1);
		/// let prog: Prog = catch_handler::<_, FirstRowMinusExcept, _>()
		/// 	.dispatch_arc_run_explicit_catch_boundary(boundary, &handlers! {});
		///
		/// let result = prog.handle(
		/// 	handlers! {
		/// 		ExceptBrand<&'static str>: |_op: Except<'_, &'static str, Prog>| ArcRunExplicit::pure(-1),
		/// 	},
		/// 	scoped_handlers! {
		/// 		SendCatchBrand<ArcBrand, &'static str>: catch_handler::<_, FirstRowMinusExcept, _>(),
		/// 	},
		/// );
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn catch<E: Send + Sync + 'a, Idx>(
			action: ArcRunExplicit<'a, R, ScopedRow, A>,
			handler: impl Fn(E) -> ArcRunExplicit<'a, R, ScopedRow, A> + Send + Sync + 'a,
		) -> ArcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::SendCatchBrand<ArcBrand, E>,
			Idx,
			A,
			A,
			impl Fn(A) -> ArcRunExplicit<'a, R, ScopedRow, A> + Send + Sync + 'a,
		>
		where
			A: Clone + Send + Sync + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::catch::SendCatch<
						'a,
						ArcBrand,
						E,
						ArcRunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let catch: crate::types::effects::catch::SendCatch<
				'a,
				ArcBrand,
				E,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::catch::SendCatch::Catch {
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| action.clone()),
				handler: <ArcBrand as crate::classes::ToDynSendFn>::new(move |e: E| handler(e)),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::catch::SendCatch<
					'a,
					ArcBrand,
					E,
					ArcRunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(catch);
			ArcRunExplicitBoundary::new(layer, ArcRunExplicit::pure)
		}

		/// Lifts a scoped `Local` effect into the `ArcRunExplicit`
		/// program: run `action` under an environment value transformed
		/// by `modify`. Mirrors
		/// [`ArcRun::local`](crate::types::effects::arc_run::ArcRun::local);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRunExplicit`: the modify closure and action are stored as
		/// `Arc<dyn Fn(...) -> _ + Send + Sync>` thunks (multi-shot,
		/// thread-safe) over the explicit `'a` lifetime.
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
		#[document_returns("An indexed `ArcRunExplicit` Local boundary.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		reader::SendReader,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendLocalBrand<ArcBrand, i32>, CNilBrand>;
		/// type Prog = ArcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = ArcRunExplicit::<FirstRow, ScopedRow, i32>::ask::<_>()
		/// 	.bind(|env| ArcRunExplicit::pure(env * 2));
		/// let boundary = ArcRunExplicit::local::<i32, _>(|env| env + 1, action).map(|value| value + 1);
		/// let prog: Prog = local_handler::<_, FirstRowMinusReader, _>()
		/// 	.dispatch_arc_run_explicit_local_boundary(boundary, &handlers! {});
		///
		/// let result = prog.handle(
		/// 	handlers! {
		/// 		SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, Prog>| match op {
		/// 			SendReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		SendLocalBrand<ArcBrand, i32>: local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		/// assert_eq!(result, 23);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn local<E: Send + Sync + 'a, Idx>(
			modify: impl Fn(E) -> E + Send + Sync + 'a,
			action: ArcRunExplicit<'a, R, ScopedRow, A>,
		) -> ArcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::SendLocalBrand<ArcBrand, E>,
			Idx,
			A,
			A,
			impl Fn(A) -> ArcRunExplicit<'a, R, ScopedRow, A> + Send + Sync + 'a,
		>
		where
			A: Clone + Send + Sync + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::local::SendLocal<
						'a,
						ArcBrand,
						E,
						ArcRunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let local: crate::types::effects::local::SendLocal<
				'a,
				ArcBrand,
				E,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::local::SendLocal::Local {
				modify: <ArcBrand as crate::classes::ToDynSendFn>::new(move |e: E| modify(e)),
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| action.clone()),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::local::SendLocal<
					'a,
					ArcBrand,
					E,
					ArcRunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(local);
			ArcRunExplicitBoundary::new(layer, ArcRunExplicit::pure)
		}

		/// Lifts a [`SendRefLocal`](crate::types::effects::ref_local::SendRefLocal)
		/// scoped environment-modification effect (Ref flavour) into the
		/// `ArcRunExplicit` program. Mirrors
		/// [`Run::ref_local`](crate::types::effects::run::Run::ref_local)
		/// for the thread-safe Arc explicit-lifetime substrate. The
		/// `modify` closure (`Fn(&E) -> E + Send + Sync + 'a`) borrows
		/// the inherited environment value rather than consuming it.
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
		#[document_returns("An indexed `ArcRunExplicit` RefLocal boundary.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		reader::SendReader,
		/// 		standard_scoped_handlers::ref_local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendRefLocalBrand<ArcBrand, i32>, CNilBrand>;
		/// type Prog = ArcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = ArcRunExplicit::<FirstRow, ScopedRow, i32>::ask::<_>()
		/// 	.bind(|env| ArcRunExplicit::pure(env * 2));
		/// let boundary =
		/// 	ArcRunExplicit::ref_local::<i32, _>(|env| *env + 5, action).map(|value| value + 1);
		/// let prog: Prog = ref_local_handler::<_, FirstRowMinusReader, _>()
		/// 	.dispatch_arc_run_explicit_ref_local_boundary(boundary, &handlers! {});
		///
		/// let result = prog.handle(
		/// 	handlers! {
		/// 		SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, Prog>| match op {
		/// 			SendReader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		SendRefLocalBrand<ArcBrand, i32>: ref_local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		/// assert_eq!(result, 31);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn ref_local<E: Send + Sync + 'a, Idx>(
			modify: impl Fn(&E) -> E + Send + Sync + 'a,
			action: ArcRunExplicit<'a, R, ScopedRow, A>,
		) -> ArcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::SendRefLocalBrand<ArcBrand, E>,
			Idx,
			A,
			A,
			impl Fn(A) -> ArcRunExplicit<'a, R, ScopedRow, A> + Send + Sync + 'a,
		>
		where
			A: Clone + Send + Sync + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::ref_local::SendRefLocal<
						'a,
						ArcBrand,
						E,
						ArcRunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let local: crate::types::effects::ref_local::SendRefLocal<
				'a,
				ArcBrand,
				E,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::ref_local::SendRefLocal::Local {
				modify: <ArcBrand as crate::classes::ToDynSendFn>::ref_new(move |e: &E| modify(e)),
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| action.clone()),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::ref_local::SendRefLocal<
					'a,
					ArcBrand,
					E,
					ArcRunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(local);
			ArcRunExplicitBoundary::new(layer, ArcRunExplicit::pure)
		}

		/// Constructs an indexed scoped `Span` boundary for a protected
		/// `ArcRunExplicit` action.
		///
		/// The selected action is stored in the scoped row layer, while
		/// mapped or bound work composes through the boundary's outer
		/// continuation. The action thunk is multi-shot and backed by
		/// `Arc<dyn Fn(()) -> _ + Send + Sync>` over the explicit `'a`
		/// lifetime, and the by-value tag must be cloneable and
		/// thread-safe.
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
		#[document_returns("An `ArcRunExplicit` Span boundary over the selected action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, String>, CNilBrand>;
		///
		/// let action: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(42);
		/// let boundary =
		/// 	ArcRunExplicit::span::<String, _>("request".to_owned(), action).map(|value| value + 1);
		/// let prog: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = span_handler()
		/// 	.dispatch_arc_run_explicit_span_boundary_with_post_action(
		/// 		boundary,
		/// 		&handlers! {},
		/// 		|tag, value| {
		/// 			assert_eq!(tag.as_str(), "request");
		/// 			ArcRunExplicit::pure(value + 1)
		/// 		},
		/// 	);
		/// assert!(matches!(prog.peel(), Ok(44)));
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn span<Tag: Clone + Send + Sync + 'a, Idx>(
			tag: Tag,
			action: ArcRunExplicit<'a, R, ScopedRow, A>,
		) -> ArcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::SendSpanBrand<ArcBrand, Tag>,
			Idx,
			A,
			A,
			impl Fn(A) -> ArcRunExplicit<'a, R, ScopedRow, A> + Send + Sync + 'a,
		>
		where
			A: Clone + Send + Sync + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::span::SendSpan<
						'a,
						ArcBrand,
						Tag,
						ArcRunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let span: crate::types::effects::span::SendSpan<
				'a,
				ArcBrand,
				Tag,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::span::SendSpan::Span {
				tag,
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| action.clone()),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::span::SendSpan<
					'a,
					ArcBrand,
					Tag,
					ArcRunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(span);
			ArcRunExplicitBoundary::new(layer, ArcRunExplicit::pure)
		}

		/// Constructs an indexed scoped Writer `censor` boundary for a
		/// protected `ArcRunExplicit` action.
		///
		/// The selected action result remains `A`. The boundary stores
		/// the selected action separately from mapped or bound outer
		/// continuations, while the standard Writer handler later decides
		/// how the thread-safe transformation applies to accumulated
		/// output.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type transformed by `censor` (`Send + Sync`).",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The thread-safe log transformation.",
			"The protected action program (must be `Clone + Send + Sync` for the Arc thunk)."
		)]
		///
		#[document_returns(
			"An `ArcRunExplicit` Writer `censor` boundary over the selected action."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendWriterCensorBrand<ArcBrand, String>, CNilBrand>;
		///
		/// let preview: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(42);
		/// assert!(matches!(preview.peel(), Ok(42)));
		///
		/// let action: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(42);
		/// let boundary =
		/// 	ArcRunExplicit::censor::<String, _>(|log| format!("{log}!"), action).map(|value| value + 1);
		/// let _ = boundary;
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn censor<LogType: Send + Sync + 'static, Idx>(
			censor: impl Fn(LogType) -> LogType + Send + Sync + 'a,
			action: ArcRunExplicit<'a, R, ScopedRow, A>,
		) -> ArcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::SendWriterCensorBrand<ArcBrand, LogType>,
			Idx,
			A,
			A,
			impl Fn(A) -> ArcRunExplicit<'a, R, ScopedRow, A> + Send + Sync + 'a,
		>
		where
			A: Clone + Send + Sync + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::writer::SendWriterCensor<
						'a,
						ArcBrand,
						LogType,
						ArcRunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let writer: crate::types::effects::writer::SendWriterCensor<
				'a,
				ArcBrand,
				LogType,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::writer::SendWriterCensor::Censor {
				censor: <ArcBrand as crate::classes::ToDynSendFn>::new(censor),
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| action.clone()),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::writer::SendWriterCensor<
					'a,
					ArcBrand,
					LogType,
					ArcRunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(writer);
			ArcRunExplicitBoundary::new(layer, ArcRunExplicit::pure)
		}

		/// Constructs an indexed scoped Writer `listen` boundary for a
		/// protected `ArcRunExplicit` action.
		///
		/// The selected action result remains `A`; the boundary's
		/// operation-result slot is `(A, LogType)`, which the standard
		/// Writer handler produces before mapped or bound outer
		/// continuations resume.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type observed by `listen` (`Clone + Send + Sync`).",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The protected action program.")]
		///
		#[document_returns(
			"An `ArcRunExplicit` Writer `listen` boundary over the selected action."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendWriterListenBrand<ArcBrand, String, i32>, CNilBrand>;
		///
		/// let preview: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(42);
		/// assert!(matches!(preview.peel(), Ok(42)));
		///
		/// let action: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(42);
		/// let boundary = ArcRunExplicit::listen::<String, _>(action).map(|(value, log)| (value + 1, log));
		/// let _ = boundary;
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Writer listen preserves selected action, operation result, final result, and the private continuation type in one indexed boundary."
		)]
		pub fn listen<LogType: Clone + Send + Sync + 'static, Idx>(
			action: ArcRunExplicit<'a, R, ScopedRow, A>
		) -> ArcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::SendWriterListenBrand<ArcBrand, LogType, A>,
			Idx,
			A,
			(A, LogType),
			impl Fn((A, LogType)) -> ArcRunExplicit<'a, R, ScopedRow, (A, LogType)> + Send + Sync + 'a,
			(A, LogType),
		>
		where
			A: Clone + Send + Sync + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::writer::SendWriterListen<
						'a,
						ArcBrand,
						LogType,
						A,
						ArcRunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, (A, LogType)>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, (A, LogType)>,
			>): Clone + Send + Sync, {
			let writer: crate::types::effects::writer::SendWriterListen<
				'a,
				ArcBrand,
				LogType,
				A,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::writer::SendWriterListen::Listen {
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| action.clone()),
				result: core::marker::PhantomData,
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::writer::SendWriterListen<
					'a,
					ArcBrand,
					LogType,
					A,
					ArcRunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(writer);
			ArcRunExplicitBoundary::new(layer, |operation: (A, LogType)| {
				ArcRunExplicit::pure(operation)
			})
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The body's result type (`Send + Sync`)."
	)]
	impl<'a, R, ScopedRow, B> ArcRunExplicit<'a, R, ScopedRow, B>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
		B: Clone + Send + Sync + 'a,
	{
		/// Lifts a [`SendRefBracketExplicit`](crate::types::effects::ref_bracket::SendRefBracketExplicit)
		/// scoped resource-management effect into the `ArcRunExplicit`
		/// program. This is the Ref flavour of
		/// [`ArcRunExplicit::bracket`]: `acquire` produces the resource,
		/// while `body` and `release` both receive independent `Arc<A>`
		/// clones.
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
		#[document_returns("An indexed `ArcRunExplicit` RefBracket boundary.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// #![recursion_limit = "512"]
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::*,
		/// 	classes::{
		/// 		SendFunctor,
		/// 		WrapDrop,
		/// 	},
		/// 	handlers,
		/// 	impl_kind,
		/// 	kinds::*,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		standard_scoped_handlers::ref_bracket_handler,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type UnderlyingRow = CoproductBrand<
		/// 	SendRefBracketExplicitBrand<ArcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>,
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
		/// let acquire: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(7);
		/// let boundary = ArcRunExplicit::<'static, FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 	acquire,
		/// 	|resource: std::sync::Arc<i32>| ArcRunExplicit::pure(*resource + 35),
		/// 	|_resource: std::sync::Arc<i32>| ArcRunExplicit::pure(()),
		/// )
		/// .map(|value| value + 1);
		/// let prog: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ref_bracket_handler()
		/// 	.dispatch_arc_run_explicit_ref_bracket_boundary(boundary, &handlers! {});
		/// assert!(matches!(prog.peel(), Ok(43)));
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying lifecycle state, the selected row member, and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn ref_bracket<A: Send + Sync + 'a, Idx>(
			acquire: ArcRunExplicit<'a, R, ScopedRow, A>,
			body: impl Fn(
				<ArcBrand as crate::classes::SendRefCountedPointer>::Of<'a, A>,
			) -> ArcRunExplicit<'a, R, ScopedRow, B>
			+ Send
			+ Sync
			+ 'a,
			release: impl Fn(
				<ArcBrand as crate::classes::SendRefCountedPointer>::Of<'a, A>,
			) -> ArcRunExplicit<'a, R, ScopedRow, ()>
			+ Send
			+ Sync
			+ 'a,
		) -> ArcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::SendRefBracketExplicitBrand<ArcBrand, NodeBrand<R, ScopedRow>, A, B>,
			Idx,
			B,
			B,
			impl Fn(B) -> ArcRunExplicit<'a, R, ScopedRow, B> + Send + Sync + 'a,
		>
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, B>,
			>): Member<
					crate::types::effects::ref_bracket::SendRefBracketExplicit<
						'a,
						ArcBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, B>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, B>,
			>): Clone + Send + Sync, {
			let bracket: crate::types::effects::ref_bracket::SendRefBracketExplicit<
				'a,
				ArcBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::ref_bracket::SendRefBracketExplicit::Bracket {
				acquire: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					acquire.clone().into_arc_free_explicit()
				}),
				body: <ArcBrand as crate::classes::ToDynSendFn>::new(
					move |a: <ArcBrand as crate::classes::SendRefCountedPointer>::Of<'a, A>| {
						body(a).into_arc_free_explicit()
					},
				),
				release: <ArcBrand as crate::classes::ToDynSendFn>::new(
					move |a: <ArcBrand as crate::classes::SendRefCountedPointer>::Of<'a, A>| {
						release(a).into_arc_free_explicit()
					},
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, B>,
			>) as Member<
				crate::types::effects::ref_bracket::SendRefBracketExplicit<
					'a,
					ArcBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			ArcRunExplicitBoundary::new(layer, ArcRunExplicit::pure)
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The body's result type (`Send + Sync`)."
	)]
	impl<'a, R, ScopedRow, B> ArcRunExplicit<'a, R, ScopedRow, B>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
		B: Clone + Send + Sync + 'a,
	{
		/// Lifts a [`SendBracketExplicit`](crate::types::effects::bracket::SendBracketExplicit)
		/// scoped resource-management effect into the `ArcRunExplicit`
		/// program. Mirrors [`Run::bracket`](crate::types::effects::run::Run::bracket)
		/// for the thread-safe Arc explicit-lifetime substrate.
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
		#[document_returns("An indexed `ArcRunExplicit` Bracket boundary.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// Recursive scoped rows that mention their own marker inside
		/// [`NodeBrand`](crate::brands::NodeBrand) cannot be written as
		/// self-referential type aliases. Use a marker struct plus an
		/// `UnderlyingRow` helper, then delegate `Kind`, `WrapDrop`, and
		/// `SendFunctor` to that helper row.
		///
		/// ```
		/// #![recursion_limit = "512"]
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::*,
		/// 	classes::{
		/// 		SendFunctor,
		/// 		WrapDrop,
		/// 	},
		/// 	handlers,
		/// 	impl_kind,
		/// 	kinds::*,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		standard_scoped_handlers::bracket_handler,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type UnderlyingRow = CoproductBrand<
		/// 	SendBracketExplicitBrand<ArcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>,
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
		/// let acquire: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(7);
		/// let boundary = ArcRunExplicit::<'static, FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 	acquire,
		/// 	|resource: std::sync::Arc<i32>| ArcRunExplicit::pure((*resource, 42)),
		/// 	|_resource: std::sync::Arc<i32>| ArcRunExplicit::pure(()),
		/// )
		/// .map(|value| value + 1);
		/// let prog: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	bracket_handler().dispatch_arc_run_explicit_bracket_boundary(boundary, &handlers! {});
		/// assert!(matches!(prog.peel(), Ok(43)));
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying lifecycle state, the selected row member, and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn bracket<A, Idx>(
			acquire: ArcRunExplicit<'a, R, ScopedRow, A>,
			body: impl Fn(
				<ArcBrand as crate::classes::Pointer>::Of<'a, A>,
			) -> ArcRunExplicit<'a, R, ScopedRow, (A, B)>
			+ Send
			+ Sync
			+ 'a,
			release: impl Fn(
				<ArcBrand as crate::classes::Pointer>::Of<'a, A>,
			) -> ArcRunExplicit<'a, R, ScopedRow, ()>
			+ Send
			+ Sync
			+ 'a,
		) -> ArcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::SendBracketExplicitBrand<ArcBrand, NodeBrand<R, ScopedRow>, A, B>,
			Idx,
			B,
			B,
			impl Fn(B) -> ArcRunExplicit<'a, R, ScopedRow, B> + Send + Sync + 'a,
		>
		where
			A: Send + Sync + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, B>,
			>): Member<
					crate::types::effects::bracket::SendBracketExplicit<
						'a,
						ArcBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, B>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, (A, B)>,
			>): Clone + Send + Sync, {
			let bracket: crate::types::effects::bracket::SendBracketExplicit<
				'a,
				ArcBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::bracket::SendBracketExplicit::Bracket {
				acquire: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					acquire.clone().into_arc_free_explicit()
				}),
				body: <ArcBrand as crate::classes::ToDynSendFn>::new(
					move |a: <ArcBrand as crate::classes::Pointer>::Of<'a, A>| {
						body(a).into_arc_free_explicit()
					},
				),
				release: <ArcBrand as crate::classes::ToDynSendFn>::new(
					move |a: <ArcBrand as crate::classes::Pointer>::Of<'a, A>| {
						release(a).into_arc_free_explicit()
					},
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, ScopedRow, B>,
			>) as Member<
				crate::types::effects::bracket::SendBracketExplicit<
					'a,
					ArcBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			ArcRunExplicitBoundary::new(layer, ArcRunExplicit::pure)
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	impl<'a, R, ScopedRow> ArcRunExplicit<'a, R, ScopedRow, ()>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
	{
		/// Lifts a `Put` state effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::put`](crate::types::effects::run::Run::put); see
		/// that method for cross-wrapper semantics. Threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendStateBrand`](crate::brands::SendStateBrand); see
		/// [`get`](ArcRunExplicit::get) for the design rationale.
		#[document_signature]
		///
		#[document_type_parameters(
			"The state type carried by `SendStateBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The new state value to write.")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Put` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, ()> = ArcRunExplicit::put::<i32, _>(42);
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, ((), i32)> =
		/// 	prog.run_state::<i32, _, CNilBrand>(0);
		/// assert_eq!(handled.extract(), ((), 42));
		/// ```
		#[inline]
		pub fn put<StateType: Clone + Send + Sync + 'static, Idx>(s: StateType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>): Member<
					ArcCoyoneda<
						'a,
						crate::brands::SendStateBrand<crate::brands::ArcBrand, StateType>,
						(),
					>,
					Idx,
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::state::SendState<
				'a,
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

		/// Lifts a `Tell` writer effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::tell`](crate::types::effects::run::Run::tell); see
		/// that method for cross-wrapper semantics. The same
		/// [`WriterBrand`](crate::brands::WriterBrand) serves all six
		/// wrappers because [`Writer`](crate::types::effects::writer::Writer)
		/// has no `dyn Fn` continuation; no parallel `SendWriterBrand`
		/// is needed. Requires `LogType: Send + Sync` so the lifted
		/// layer participates in the Arc substrate's thread-safety
		/// cascade.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type carried by `WriterBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The log value to emit.")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Tell` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, ()> =
		/// 	ArcRunExplicit::tell::<String, _>("logged".to_string());
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, ((), String)> =
		/// 	prog.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), "logged".to_string()));
		/// ```
		#[inline]
		pub fn tell<LogType: Clone + Send + Sync + 'static, Idx>(log: LogType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<ArcCoyoneda<'a, crate::brands::WriterBrand<LogType>, ()>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::writer::Writer<'a, LogType, ()> =
				crate::types::effects::writer::Writer::Tell(log, (), core::marker::PhantomData);
			Self::lift::<crate::brands::WriterBrand<LogType>, Idx>(effect)
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	impl<'a, R, ScopedRow> ArcRunExplicit<'a, R, ScopedRow, bool>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
	{
		/// Lifts an `Alt` choose effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`RcRun::choose`](crate::types::effects::rc_run::RcRun::choose);
		/// see that method for cross-wrapper semantics. Differences
		/// for `ArcRunExplicit`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendChooseBrand`](crate::brands::SendChooseBrand) (rather
		/// than `ChooseBrand`) so the continuation projection is
		/// structurally `Send + Sync`.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Alt` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::<FirstRow, Scoped, bool>::choose().bind(|branch| {
		/// 		ArcRunExplicit::<FirstRow, Scoped, i32>::pure(if branch { 1 } else { 0 })
		/// 	});
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	prog.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>): Member<
					ArcCoyoneda<'a, crate::brands::SendChooseBrand<crate::brands::ArcBrand>, bool>,
					Idx,
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, bool>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, bool>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, bool>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::choose::SendChoose<
				'a,
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
