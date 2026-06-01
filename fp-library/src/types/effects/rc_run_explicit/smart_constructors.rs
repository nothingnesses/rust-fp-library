#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		super::super::inner::{
			RcRunExplicit,
			RcRunExplicitBoundary,
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
	impl<'a, R, ScopedRow, V> RcRunExplicit<'a, R, ScopedRow, Option<V>>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		V: Clone + 'static + 'a,
	{
		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect KVStore;
			method lookup;
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	impl<'a, R, ScopedRow> RcRunExplicit<'a, R, ScopedRow, ()>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect KVStore;
			method update;
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The body's result type."
	)]
	impl<'a, R, ScopedRow, B> RcRunExplicit<'a, R, ScopedRow, B>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		B: Clone + 'a,
	{
		/// Lifts a [`RefBracketExplicit`](crate::types::effects::ref_bracket::RefBracketExplicit)
		/// scoped resource-management effect into the `RcRunExplicit`
		/// program. This is the Ref flavour of
		/// [`RcRunExplicit::bracket`]: `acquire` produces the resource,
		/// while `body` and `release` both receive independent `Rc<A>`
		/// clones.
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
		#[document_returns("An indexed `RcRunExplicit` RefBracket boundary.")]
		#[document_examples]
		///
		/// Recursive scoped rows that mention their own marker inside
		/// [`NodeBrand`](crate::brands::NodeBrand) cannot be written as
		/// self-referential type aliases. Use a marker struct plus an
		/// `UnderlyingRow` helper, then delegate `Kind`, `WrapDrop`, and
		/// `Functor` to that helper row.
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
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		standard_scoped_handlers::ref_bracket_handler,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type UnderlyingRow = CoproductBrand<
		/// 	RefBracketExplicitBrand<RcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>,
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
		/// let acquire: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(7);
		/// let boundary = RcRunExplicit::<'static, FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 	acquire,
		/// 	|resource: std::rc::Rc<i32>| RcRunExplicit::pure(*resource + 35),
		/// 	|_resource: std::rc::Rc<i32>| RcRunExplicit::pure(()),
		/// )
		/// .map(|value| value + 1);
		/// let prog: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = ref_bracket_handler()
		/// 	.dispatch_rc_run_explicit_ref_bracket_boundary(boundary, &handlers! {});
		/// assert!(matches!(prog.peel(), Ok(43)));
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying lifecycle state, the selected row member, and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn ref_bracket<A: 'a, Idx>(
			acquire: RcRunExplicit<'a, R, ScopedRow, A>,
			body: impl Fn(
				<RcBrand as crate::classes::RefCountedPointer>::Of<'a, A>,
			) -> RcRunExplicit<'a, R, ScopedRow, B>
			+ 'a,
			release: impl Fn(
				<RcBrand as crate::classes::RefCountedPointer>::Of<'a, A>,
			) -> RcRunExplicit<'a, R, ScopedRow, ()>
			+ 'a,
		) -> RcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::RefBracketExplicitBrand<RcBrand, NodeBrand<R, ScopedRow>, A, B>,
			Idx,
			B,
			B,
			impl Fn(B) -> RcRunExplicit<'a, R, ScopedRow, B> + 'a,
		>
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, B>,
			>): Member<
					crate::types::effects::ref_bracket::RefBracketExplicit<
						'a,
						RcBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				>, {
			let bracket: crate::types::effects::ref_bracket::RefBracketExplicit<
				'a,
				RcBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::ref_bracket::RefBracketExplicit::Bracket {
				acquire: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					acquire.clone().into_rc_free_explicit()
				}),
				body: <RcBrand as crate::classes::ToDynCloneFn>::new(
					move |a: <RcBrand as crate::classes::RefCountedPointer>::Of<'a, A>| {
						body(a).into_rc_free_explicit()
					},
				),
				release: <RcBrand as crate::classes::ToDynCloneFn>::new(
					move |a: <RcBrand as crate::classes::RefCountedPointer>::Of<'a, A>| {
						release(a).into_rc_free_explicit()
					},
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, B>,
			>) as Member<
				crate::types::effects::ref_bracket::RefBracketExplicit<
					'a,
					RcBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			RcRunExplicitBoundary::new(layer, RcRunExplicit::pure)
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The state type (also the program's result type for `get`)."
	)]
	#[document_parameters("The `RcRunExplicit` instance.")]
	impl<'a, R, ScopedRow, A: 'a> RcRunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Reader;
			method ask;
		}

		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect State;
			method get;
		}

		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Fresh;
			method fresh;
		}

		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Input;
			method input;
		}

		define_run_wrapper_method! {
			wrapper RcRunExplicit;
			method expand;
		}

		define_run_wrapper_method! {
			wrapper RcRunExplicit;
			method weaken;
		}

		/// Lifts a `Throw` except effect into the `RcRunExplicit`
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
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Throw` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::throw::<&'static str, _>("oops");
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	prog.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("oops"));
		/// ```
		#[inline]
		pub fn throw<ErrorType: Clone + 'static, Idx>(e: ErrorType) -> Self
		where
			A: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, crate::brands::ExceptBrand<ErrorType>, A>, Idx>, {
			let effect: crate::types::effects::except::Except<'a, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}

		/// Lifts an `Empty` effect into the `RcRunExplicit` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// multi-shot nondeterministic interpreter.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> = RcRunExplicit::empty();
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			A: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, crate::brands::EmptyBrand, A>, Idx>, {
			let effect: crate::types::effects::empty::Empty<'a, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}

		/// Lifts a scoped `Catch` effect into the `RcRunExplicit` program:
		/// run `action`, and if it throws an `E`, invoke `handler` with
		/// the error to produce a recovery program. Mirrors
		/// [`RcRun::catch`](crate::types::effects::rc_run::RcRun::catch);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RcRunExplicit`: the action and recovery handler are stored
		/// as `Rc<dyn Fn(...) -> _>` thunks (multi-shot) over the
		/// explicit `'a` lifetime.
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
		#[document_returns("An indexed `RcRunExplicit` Catch boundary.")]
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
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		standard_scoped_handlers::catch_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type FirstRowMinusExcept = CNilBrand;
		/// type ScopedRow = CoproductBrand<CatchBrand<RcBrand, &'static str>, CNilBrand>;
		/// type Prog = RcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = RcRunExplicit::throw::<&'static str, _>("from-action");
		/// let boundary = RcRunExplicit::catch::<&'static str, _>(action, |_e| RcRunExplicit::pure(41))
		/// 	.map(|value| value + 1);
		/// let prog: Prog = catch_handler::<_, FirstRowMinusExcept, _>()
		/// 	.dispatch_rc_run_explicit_catch_boundary(boundary, &handlers! {});
		///
		/// let result = prog.handle(
		/// 	handlers! {
		/// 		ExceptBrand<&'static str>: |_op: Except<'_, &'static str, Prog>| RcRunExplicit::pure(-1),
		/// 	},
		/// 	scoped_handlers! {
		/// 		CatchBrand<RcBrand, &'static str>: catch_handler::<_, FirstRowMinusExcept, _>(),
		/// 	},
		/// );
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn catch<E: 'a, Idx>(
			action: RcRunExplicit<'a, R, ScopedRow, A>,
			handler: impl Fn(E) -> RcRunExplicit<'a, R, ScopedRow, A> + 'a,
		) -> RcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::CatchBrand<RcBrand, E>,
			Idx,
			A,
			A,
			impl Fn(A) -> RcRunExplicit<'a, R, ScopedRow, A> + 'a,
		>
		where
			A: Clone + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::catch::Catch<
						'a,
						RcBrand,
						E,
						RcRunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				>, {
			let catch: crate::types::effects::catch::Catch<
				'a,
				RcBrand,
				E,
				RcRunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::catch::Catch::Catch {
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| action.clone()),
				handler: <RcBrand as crate::classes::ToDynCloneFn>::new(move |e: E| handler(e)),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::catch::Catch<
					'a,
					RcBrand,
					E,
					RcRunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(catch);
			RcRunExplicitBoundary::new(layer, RcRunExplicit::pure)
		}

		/// Lifts a scoped `Local` effect into the `RcRunExplicit`
		/// program: run `action` under an environment value transformed
		/// by `modify`. Mirrors
		/// [`RcRun::local`](crate::types::effects::rc_run::RcRun::local);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RcRunExplicit`: the modify closure and action are stored as
		/// `Rc<dyn Fn(...) -> _>` thunks (multi-shot) over the explicit
		/// `'a` lifetime.
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
		#[document_returns("An indexed `RcRunExplicit` Local boundary.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		reader::Reader,
		/// 		standard_scoped_handlers::local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<LocalBrand<RcBrand, i32>, CNilBrand>;
		/// type Prog = RcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = RcRunExplicit::<FirstRow, ScopedRow, i32>::ask::<_>()
		/// 	.bind(|env| RcRunExplicit::pure(env * 2));
		/// let boundary = RcRunExplicit::local::<i32, _>(|env| env + 1, action).map(|value| value + 1);
		/// let prog: Prog = local_handler::<_, FirstRowMinusReader, _>()
		/// 	.dispatch_rc_run_explicit_local_boundary(boundary, &handlers! {});
		///
		/// let result = prog.handle(
		/// 	handlers! {
		/// 		ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, Prog>| match op {
		/// 			Reader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		LocalBrand<RcBrand, i32>: local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		/// assert_eq!(result, 23);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn local<E: 'a, Idx>(
			modify: impl Fn(E) -> E + 'a,
			action: RcRunExplicit<'a, R, ScopedRow, A>,
		) -> RcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::LocalBrand<RcBrand, E>,
			Idx,
			A,
			A,
			impl Fn(A) -> RcRunExplicit<'a, R, ScopedRow, A> + 'a,
		>
		where
			A: Clone + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::local::Local<
						'a,
						RcBrand,
						E,
						RcRunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				>, {
			let local: crate::types::effects::local::Local<
				'a,
				RcBrand,
				E,
				RcRunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::local::Local::Local {
				modify: <RcBrand as crate::classes::ToDynCloneFn>::new(move |e: E| modify(e)),
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| action.clone()),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::local::Local<
					'a,
					RcBrand,
					E,
					RcRunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(local);
			RcRunExplicitBoundary::new(layer, RcRunExplicit::pure)
		}

		/// Lifts a [`RefLocal`](crate::types::effects::ref_local::RefLocal)
		/// scoped environment-modification effect (Ref flavour) into the
		/// `RcRunExplicit` program. Mirrors
		/// [`Run::ref_local`](crate::types::effects::run::Run::ref_local)
		/// for the multi-shot Rc explicit-lifetime substrate. The
		/// `modify` closure (`Fn(&E) -> E + 'a`) borrows the inherited
		/// environment value rather than consuming it, removing the
		/// `E: Clone` requirement that the Val flavour
		/// ([`local`](RcRunExplicit::local)) imposes.
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
		#[document_returns("An indexed `RcRunExplicit` RefLocal boundary.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::effects::{
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		reader::Reader,
		/// 		standard_scoped_handlers::ref_local_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		/// type FirstRowMinusReader = CNilBrand;
		/// type ScopedRow = CoproductBrand<RefLocalBrand<RcBrand, i32>, CNilBrand>;
		/// type Prog = RcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// let action: Prog = RcRunExplicit::<FirstRow, ScopedRow, i32>::ask::<_>()
		/// 	.bind(|env| RcRunExplicit::pure(env * 2));
		/// let boundary =
		/// 	RcRunExplicit::ref_local::<i32, _>(|env| *env + 5, action).map(|value| value + 1);
		/// let prog: Prog = ref_local_handler::<_, FirstRowMinusReader, _>()
		/// 	.dispatch_rc_run_explicit_ref_local_boundary(boundary, &handlers! {});
		///
		/// let result = prog.handle(
		/// 	handlers! {
		/// 		ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, Prog>| match op {
		/// 			Reader::Ask(k) => k(10),
		/// 		},
		/// 	},
		/// 	scoped_handlers! {
		/// 		RefLocalBrand<RcBrand, i32>: ref_local_handler::<_, FirstRowMinusReader, _>(),
		/// 	},
		/// );
		/// assert_eq!(result, 31);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn ref_local<E: 'a, Idx>(
			modify: impl Fn(&E) -> E + 'a,
			action: RcRunExplicit<'a, R, ScopedRow, A>,
		) -> RcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::RefLocalBrand<RcBrand, E>,
			Idx,
			A,
			A,
			impl Fn(A) -> RcRunExplicit<'a, R, ScopedRow, A> + 'a,
		>
		where
			A: Clone + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::ref_local::RefLocal<
						'a,
						RcBrand,
						E,
						RcRunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				>, {
			let local: crate::types::effects::ref_local::RefLocal<
				'a,
				RcBrand,
				E,
				RcRunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::ref_local::RefLocal::Local {
				modify: <RcBrand as crate::classes::ToDynCloneFn>::ref_new(move |e: &E| modify(e)),
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| action.clone()),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::ref_local::RefLocal<
					'a,
					RcBrand,
					E,
					RcRunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(local);
			RcRunExplicitBoundary::new(layer, RcRunExplicit::pure)
		}

		/// Constructs an indexed scoped `Span` boundary for a protected
		/// `RcRunExplicit` action.
		///
		/// The selected action is stored in the scoped row layer, while
		/// mapped or bound work composes through the boundary's outer
		/// continuation. The action thunk is multi-shot and backed by
		/// `Rc<dyn Fn(()) -> _>` over the explicit `'a` lifetime.
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
		#[document_returns("An `RcRunExplicit` Span boundary over the selected action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::effects::{
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		standard_scoped_handlers::span_handler,
		/// 	},
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, String>, CNilBrand>;
		///
		/// let action: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(42);
		/// let boundary =
		/// 	RcRunExplicit::span::<String, _>("request".to_owned(), action).map(|value| value + 1);
		/// let prog: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = span_handler()
		/// 	.dispatch_rc_run_explicit_span_boundary_with_post_action(
		/// 		boundary,
		/// 		&handlers! {},
		/// 		|tag, value| {
		/// 			assert_eq!(tag.as_str(), "request");
		/// 			RcRunExplicit::pure(value + 1)
		/// 		},
		/// 	);
		/// assert!(matches!(prog.peel(), Ok(44)));
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn span<Tag: Clone + 'a, Idx>(
			tag: Tag,
			action: RcRunExplicit<'a, R, ScopedRow, A>,
		) -> RcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::SpanBrand<RcBrand, Tag>,
			Idx,
			A,
			A,
			impl Fn(A) -> RcRunExplicit<'a, R, ScopedRow, A> + 'a,
		>
		where
			A: Clone + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::span::Span<
						'a,
						RcBrand,
						Tag,
						RcRunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				>, {
			let span: crate::types::effects::span::Span<
				'a,
				RcBrand,
				Tag,
				RcRunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::span::Span::Span {
				tag,
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| action.clone()),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::span::Span<
					'a,
					RcBrand,
					Tag,
					RcRunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(span);
			RcRunExplicitBoundary::new(layer, RcRunExplicit::pure)
		}

		/// Constructs an indexed scoped Writer `censor` boundary for a
		/// protected `RcRunExplicit` action.
		///
		/// The selected action result remains `A`. The boundary stores
		/// the selected action separately from mapped or bound outer
		/// continuations, while the standard Writer handler later decides
		/// how the transformation applies to accumulated output.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type transformed by `censor`.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The log transformation (must be multi-shot for the Rc cell).",
			"The protected action program (must be `Clone` for the multi-shot Rc-thunk)."
		)]
		///
		#[document_returns("An `RcRunExplicit` Writer `censor` boundary over the selected action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<WriterCensorBrand<RcBrand, String>, CNilBrand>;
		///
		/// let preview: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(42);
		/// assert!(matches!(preview.peel(), Ok(42)));
		///
		/// let action: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(42);
		/// let boundary =
		/// 	RcRunExplicit::censor::<String, _>(|log| format!("{log}!"), action).map(|value| value + 1);
		/// let _ = boundary;
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying the selected row member and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn censor<LogType: 'static, Idx>(
			censor: impl Fn(LogType) -> LogType + 'a,
			action: RcRunExplicit<'a, R, ScopedRow, A>,
		) -> RcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::WriterCensorBrand<RcBrand, LogType>,
			Idx,
			A,
			A,
			impl Fn(A) -> RcRunExplicit<'a, R, ScopedRow, A> + 'a,
		>
		where
			A: Clone + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::writer::WriterCensor<
						'a,
						RcBrand,
						LogType,
						RcRunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				>, {
			let writer: crate::types::effects::writer::WriterCensor<
				'a,
				RcBrand,
				LogType,
				RcRunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::writer::WriterCensor::Censor {
				censor: <RcBrand as crate::classes::ToDynCloneFn>::new(censor),
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| action.clone()),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::writer::WriterCensor<
					'a,
					RcBrand,
					LogType,
					RcRunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(writer);
			RcRunExplicitBoundary::new(layer, RcRunExplicit::pure)
		}

		/// Constructs an indexed scoped Writer `listen` boundary for a
		/// protected `RcRunExplicit` action.
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
		#[document_parameters("The protected action program.")]
		///
		#[document_returns("An `RcRunExplicit` Writer `listen` boundary over the selected action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<WriterListenBrand<RcBrand, String, i32>, CNilBrand>;
		///
		/// let preview: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(42);
		/// assert!(matches!(preview.peel(), Ok(42)));
		///
		/// let action: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(42);
		/// let boundary = RcRunExplicit::listen::<String, _>(action).map(|(value, log)| (value + 1, log));
		/// let _ = boundary;
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Writer listen preserves selected action, operation result, final result, and the private continuation type in one indexed boundary."
		)]
		pub fn listen<LogType: Clone + 'static, Idx>(
			action: RcRunExplicit<'a, R, ScopedRow, A>
		) -> RcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::WriterListenBrand<RcBrand, LogType, A>,
			Idx,
			A,
			(A, LogType),
			impl Fn((A, LogType)) -> RcRunExplicit<'a, R, ScopedRow, (A, LogType)> + 'a,
			(A, LogType),
		>
		where
			A: Clone + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, A>,
			>): Member<
					crate::types::effects::writer::WriterListen<
						'a,
						RcBrand,
						LogType,
						A,
						RcRunExplicit<'a, R, ScopedRow, A>,
					>,
					Idx,
				>, {
			let writer: crate::types::effects::writer::WriterListen<
				'a,
				RcBrand,
				LogType,
				A,
				RcRunExplicit<'a, R, ScopedRow, A>,
			> = crate::types::effects::writer::WriterListen::Listen {
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| action.clone()),
				result: core::marker::PhantomData,
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, A>,
			>) as Member<
				crate::types::effects::writer::WriterListen<
					'a,
					RcBrand,
					LogType,
					A,
					RcRunExplicit<'a, R, ScopedRow, A>,
				>,
				Idx,
			>>::inject(writer);
			RcRunExplicitBoundary::new(layer, |operation: (A, LogType)| {
				RcRunExplicit::pure(operation)
			})
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The body's result type."
	)]
	impl<'a, R, ScopedRow, B> RcRunExplicit<'a, R, ScopedRow, B>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		B: Clone + 'a,
	{
		/// Lifts a [`BracketExplicit`](crate::types::effects::bracket::BracketExplicit)
		/// scoped resource-management effect into the `RcRunExplicit`
		/// program. Mirrors [`Run::bracket`](crate::types::effects::run::Run::bracket)
		/// for the multi-shot Rc explicit-lifetime substrate.
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
		#[document_returns("An indexed `RcRunExplicit` Bracket boundary.")]
		///
		#[document_examples]
		///
		/// Recursive scoped rows that mention their own marker inside
		/// [`NodeBrand`](crate::brands::NodeBrand) cannot be written as
		/// self-referential type aliases. Use a marker struct plus an
		/// `UnderlyingRow` helper, then delegate `Kind`, `WrapDrop`, and
		/// `Functor` to that helper row.
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
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		standard_scoped_handlers::bracket_handler,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type UnderlyingRow = CoproductBrand<
		/// 	BracketExplicitBrand<RcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>,
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
		/// let acquire: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(7);
		/// let boundary = RcRunExplicit::<'static, FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 	acquire,
		/// 	|resource: std::rc::Rc<i32>| RcRunExplicit::pure((*resource, 42)),
		/// 	|_resource: std::rc::Rc<i32>| RcRunExplicit::pure(()),
		/// )
		/// .map(|value| value + 1);
		/// let prog: RcRunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	bracket_handler().dispatch_rc_run_explicit_bracket_boundary(boundary, &handlers! {});
		/// assert!(matches!(prog.peel(), Ok(43)));
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "Explicit scoped smart constructors return boundary values carrying lifecycle state, the selected row member, and an opaque continuation closure; stable Rust cannot name this impl Fn continuation in a reusable alias."
		)]
		pub fn bracket<A, Idx>(
			acquire: RcRunExplicit<'a, R, ScopedRow, A>,
			body: impl Fn(
				<RcBrand as crate::classes::Pointer>::Of<'a, A>,
			) -> RcRunExplicit<'a, R, ScopedRow, (A, B)>
			+ 'a,
			release: impl Fn(
				<RcBrand as crate::classes::Pointer>::Of<'a, A>,
			) -> RcRunExplicit<'a, R, ScopedRow, ()>
			+ 'a,
		) -> RcRunExplicitBoundary<
			'a,
			R,
			ScopedRow,
			crate::brands::BracketExplicitBrand<RcBrand, NodeBrand<R, ScopedRow>, A, B>,
			Idx,
			B,
			B,
			impl Fn(B) -> RcRunExplicit<'a, R, ScopedRow, B> + 'a,
		>
		where
			A: 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, B>,
			>): Member<
					crate::types::effects::bracket::BracketExplicit<
						'a,
						RcBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				>, {
			let bracket: crate::types::effects::bracket::BracketExplicit<
				'a,
				RcBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::bracket::BracketExplicit::Bracket {
				acquire: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					acquire.clone().into_rc_free_explicit()
				}),
				body: <RcBrand as crate::classes::ToDynCloneFn>::new(
					move |a: <RcBrand as crate::classes::Pointer>::Of<'a, A>| {
						body(a).into_rc_free_explicit()
					},
				),
				release: <RcBrand as crate::classes::ToDynCloneFn>::new(
					move |a: <RcBrand as crate::classes::Pointer>::Of<'a, A>| {
						release(a).into_rc_free_explicit()
					},
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, ScopedRow, B>,
			>) as Member<
				crate::types::effects::bracket::BracketExplicit<
					'a,
					RcBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			RcRunExplicitBoundary::new(layer, RcRunExplicit::pure)
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	impl<'a, R, ScopedRow> RcRunExplicit<'a, R, ScopedRow, ()>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect State;
			method put;
		}

		/// Lifts a `Tell` writer effect into the `RcRunExplicit`
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
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Tell` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, ()> =
		/// 	RcRunExplicit::tell::<String, _>("logged".to_string());
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, ((), String)> =
		/// 	prog.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), "logged".to_string()));
		/// ```
		#[inline]
		pub fn tell<LogType: Clone + 'static, Idx>(log: LogType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<RcCoyoneda<'a, crate::brands::WriterBrand<LogType>, ()>, Idx>, {
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
	impl<'a, R, ScopedRow> RcRunExplicit<'a, R, ScopedRow, bool>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		/// Lifts an `Alt` choose effect into the `RcRunExplicit`
		/// program. Mirrors
		/// [`RcRun::choose`](crate::types::effects::rc_run::RcRun::choose);
		/// see that method for cross-wrapper semantics.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Alt` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::<FirstRow, Scoped, bool>::choose().bind(|branch| {
		/// 		RcRunExplicit::<FirstRow, Scoped, i32>::pure(if branch { 1 } else { 0 })
		/// 	});
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	prog.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>): Member<
					RcCoyoneda<'a, crate::brands::ChooseBrand<crate::brands::RcBrand>, bool>,
					Idx,
				>, {
			let effect: crate::types::effects::choose::Choose<'a, crate::brands::RcBrand, bool> =
				crate::types::effects::choose::Choose::Alt(
					<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|b: bool| b),
				);
			Self::lift::<crate::brands::ChooseBrand<crate::brands::RcBrand>, Idx>(effect)
		}
	}
}

#[expect(
	unused_imports,
	reason = "inherent impl modules follow the document_module re-export boundary"
)]
pub use inner::*;
