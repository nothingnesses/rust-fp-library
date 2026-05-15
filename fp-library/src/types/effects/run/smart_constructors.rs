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

#[expect(
	unused_imports,
	reason = "inherent impl modules follow the document_module re-export boundary"
)]
pub use inner::*;
