//! Explicit-substrate Run program over [`FreeExplicit`](crate::types::FreeExplicit)
//! and a dual-row [`NodeBrand`](crate::brands::NodeBrand).
//!
//! `RunExplicit<'a, R, S, A>` is the user-facing wrapper for the Explicit
//! Run-style effect computation:
//!
//! ```text
//! RunExplicit<'a, R, S, A> = FreeExplicit<'a, NodeBrand<R, S>, A>
//! ```
//!
//! The first-order row brand `R` carries the effect functors (typically
//! a [`CoproductBrand`](crate::brands::CoproductBrand) of
//! [`CoyonedaBrand`](crate::brands::CoyonedaBrand)-wrapped effects
//! terminated by [`CNilBrand`](crate::brands::CNilBrand)); the scoped
//! row brand `S` carries higher-order constructors (future
//! scoped-effect work populates it with `Catch`, `Local`, etc.; for
//! first-order-only programs it stays as `CNilBrand`).
//!
//! `RunExplicit` is the Explicit counterpart of
//! [`Run`](crate::types::effects::run::Run). The Explicit substrate is
//! single-shot, keeps the functor structure as a concrete recursive enum
//! (no `Box<dyn Any>` erasure), supports non-`'static` payloads, and has
//! O(N) [`bind`](crate::types::FreeExplicit::bind) on left-associated
//! chains. Its brand exposes API via Brand-dispatched type classes, so
//! programs written against generic [`Functor`](crate::classes::Functor)
//! / [`Pointed`](crate::classes::Pointed) /
//! [`Semimonad`](crate::classes::Semimonad) bounds work without naming
//! `RunExplicit` directly.
//!
//! ## Brand-level coverage
//!
//! [`RunExplicitBrand`](crate::brands::RunExplicitBrand) implements
//! [`Functor`](crate::classes::Functor),
//! [`Pointed`](crate::classes::Pointed),
//! [`Semimonad`](crate::classes::Semimonad),
//! [`RefFunctor`](crate::classes::RefFunctor),
//! [`RefPointed`](crate::classes::RefPointed), and
//! [`RefSemimonad`](crate::classes::RefSemimonad) by delegating to
//! [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s impls.
//! [`Monad`](crate::classes::Monad) and
//! [`RefMonad`](crate::classes::RefMonad) are not reachable because the
//! [`Monad`](crate::classes::Monad) blanket impl requires
//! [`Applicative`](crate::classes::Applicative), which
//! [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand) deliberately
//! does not implement. The
//! [`Ref`](crate::classes::RefFunctor) hierarchy is bounded by
//! `R: RefFunctor, S: RefFunctor`; the canonical
//! [`CoyonedaBrand`](crate::brands::CoyonedaBrand)-wrapped Run row does
//! not satisfy that bound, so brand-level
//! [`Ref`](crate::classes::RefFunctor) dispatch is reachable only via
//! synthetic rows whose brands carry their own
//! [`RefFunctor`](crate::classes::RefFunctor) impls (e.g.,
//! `CoproductBrand<IdentityBrand, CNilBrand>`).

mod boundary;

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			Apply,
			brands::{
				CNilBrand,
				FreeExplicitBrand,
				NodeBrand,
				RcBrand,
				RunExplicitBrand,
			},
			classes::{
				Functor,
				MonadRec,
				Pointed,
				RefCountedPointer,
				RefFunctor,
				RefPointed,
				RefSemimonad,
				Semimonad,
				WrapDrop,
			},
			functions::tail_rec_m,
			impl_kind,
			kinds::*,
			types::{
				Coyoneda,
				FreeExplicit,
				effects::{
					coproduct::CoproductEmbedder,
					interpreter::{
						DispatchHandlers,
						DispatchScopedHandlers,
					},
					member::Member,
					node::Node,
					run::Run,
				},
			},
		},
		core::ops::ControlFlow,
		fp_macros::*,
	};

	pub use super::boundary::RunExplicitBoundary;
	pub(crate) use super::boundary::{
		RunExplicitActionSuppliedScopedContinuation,
		RunExplicitBracketCarrierLayer,
		RunExplicitCatchCarrierLayer,
		RunExplicitLocalCarrierLayer,
		RunExplicitRefBracketCarrierLayer,
		RunExplicitRefLocalCarrierLayer,
		RunExplicitScopedContinuation,
		RunExplicitSpanCarrierLayer,
	};

	/// Explicit-substrate Run program: a thin wrapper over
	/// [`FreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::FreeExplicit).
	///
	/// The wrapper exists so user-facing API can be expressed without
	/// leaking the underlying [`FreeExplicit`](crate::types::FreeExplicit)
	/// representation. It is a tuple struct over the inner
	/// [`FreeExplicit`](crate::types::FreeExplicit); converting back via
	/// [`into_free_explicit`](RunExplicit::into_free_explicit) is a
	/// zero-cost move.
	#[document_type_parameters(
		"The lifetime that bounds the payload and the row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand (typically `CNilBrand` for first-order-only programs).",
		"The result type."
	)]
	pub struct RunExplicit<'a, R, S, A>(FreeExplicit<'a, NodeBrand<R, S>, A>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a;

	impl_kind! {
		impl<R: WrapDrop + Functor + 'static, S: WrapDrop + Functor + 'static>
			for RunExplicitBrand<R, S> {
			type Of<'a, A: 'a>: 'a = RunExplicit<'a, R, S, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RunExplicit` instance.")]
	impl<'a, R, S, A: 'a> RunExplicit<'a, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Wraps a
		/// [`FreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::FreeExplicit)
		/// as a `RunExplicit<'a, R, S, A>`. Zero-cost.
		#[document_signature]
		///
		#[document_parameters("The underlying `FreeExplicit` computation.")]
		///
		#[document_returns("A `RunExplicit` wrapping `free`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let free: FreeExplicit<'_, NodeBrand<FirstRow, Scoped>, i32> = FreeExplicit::pure(7);
		/// let run: RunExplicit<'_, FirstRow, Scoped, i32> = RunExplicit::from_free_explicit(free);
		/// assert_eq!(run.into_free_explicit().evaluate(), 7);
		/// ```
		#[inline]
		pub fn from_free_explicit(free: FreeExplicit<'a, NodeBrand<R, S>, A>) -> Self {
			RunExplicit(free)
		}

		/// Unwraps a `RunExplicit<'a, R, S, A>` to its underlying
		/// [`FreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::FreeExplicit).
		/// Zero-cost.
		#[document_signature]
		///
		#[document_returns("The underlying `FreeExplicit` computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	RunExplicit::from_free_explicit(FreeExplicit::pure(7));
		/// let free: FreeExplicit<'_, NodeBrand<FirstRow, Scoped>, i32> = run.into_free_explicit();
		/// assert_eq!(free.evaluate(), 7);
		/// ```
		#[inline]
		pub fn into_free_explicit(self) -> FreeExplicit<'a, NodeBrand<R, S>, A> {
			self.0
		}

		/// Wraps a value in a pure `RunExplicit` computation. Delegates
		/// to [`FreeExplicit::pure`](crate::types::FreeExplicit).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A `RunExplicit` computation that produces `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RunExplicit<'_, FirstRow, Scoped, i32> = RunExplicit::pure(42);
		/// assert_eq!(run.into_free_explicit().evaluate(), 42);
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self {
			RunExplicit::from_free_explicit(FreeExplicit::pure(a))
		}

		/// Decomposes this `RunExplicit` computation into one step.
		/// Returns `Ok(a)` for a pure value or `Err(layer)` carrying
		/// the next `RunExplicit` continuation in a
		/// [`Node`](crate::types::effects::node::Node) layer.
		/// Walks the `FreeExplicitView` from the underlying substrate.
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` for a pure result, or `Err(layer)` carrying the next `RunExplicit` step."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RunExplicit<'_, FirstRow, Scoped, i32> = RunExplicit::pure(7);
		/// assert!(matches!(run.peel(), Ok(7)));
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "Return type encodes Result<A, NodeBrand<R, S>::Of<'a, RunExplicit<'a, R, S, A>>>; the GAT projection is structurally complex but cannot be aliased without losing the projection link the wrapper depends on."
		)]
		pub fn peel(
			self
		) -> Result<
			A,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
		> {
			match self.0.to_view() {
				crate::types::FreeExplicitView::Pure(a) => Ok(a),
				crate::types::FreeExplicitView::Wrap(node) => {
					let mapped = <NodeBrand<R, S> as Functor>::map(
						|boxed: Box<FreeExplicit<'a, NodeBrand<R, S>, A>>| -> RunExplicit<'a, R, S, A> {
							RunExplicit::from_free_explicit(*boxed)
						},
						node,
					);
					Err(mapped)
				}
			}
		}

		/// Lifts a [`Node`](crate::types::effects::node::Node) dispatch
		/// layer into the `RunExplicit` program. The `node` argument
		/// is the
		/// [`NodeBrand<R, S>`](crate::brands::NodeBrand)
		/// `Of<'a, A>` projection; `send` wraps it via
		/// [`FreeExplicit::wrap`](crate::types::FreeExplicit) after
		/// promoting each `A` into a boxed pure `FreeExplicit`. The
		/// `Node`-projection signature is symmetric across all six
		/// Run wrappers; see
		/// [`Run::send`](crate::types::effects::run::Run::send) for the
		/// rationale.
		#[document_signature]
		///
		#[document_parameters("The Node dispatch layer carrying the effect operation.")]
		///
		#[document_returns(
			"A `RunExplicit` computation that performs the effect and returns its result."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 			run_explicit::RunExplicit,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let layer = Coproduct::inject(Identity(7));
		/// let run: RunExplicit<'_, FirstRow, Scoped, i32> = RunExplicit::send(Node::First(layer));
		/// let next = match run.peel() {
		/// 	Err(Node::First(Coproduct::Inl(Identity(n)))) => n,
		/// 	_ => panic!("expected First(Inl(Identity(..))) layer"),
		/// };
		/// assert!(matches!(next.peel(), Ok(7)));
		/// ```
		#[inline]
		pub fn send(
			node: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> Self {
			let mapped = <NodeBrand<R, S> as Functor>::map(
				|a: A| -> Box<FreeExplicit<'a, NodeBrand<R, S>, A>> {
					Box::new(FreeExplicit::pure(a))
				},
				node,
			);
			RunExplicit::from_free_explicit(FreeExplicit::wrap(mapped))
		}

		/// Lifts a raw effect value into a `RunExplicit` program.
		///
		/// Explicit-substrate analog of
		/// [`Run::lift`](crate::types::effects::run::Run::lift). Same chain
		/// (`Coyoneda::lift` -> `Member::inject` ->
		/// `Node::First` -> [`send`](RunExplicit::send)), parameterized
		/// over `'a` rather than `'static` so the lifted effect can borrow
		/// non-`'static` data.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect being lifted.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The effect value to lift.")]
		///
		#[document_returns("A `RunExplicit` program suspended at the lifted effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	RunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// // The program is suspended at the lifted effect; peel reveals the layer.
		/// assert!(run.peel().is_err());
		/// ```
		#[inline]
		pub fn lift<EBrand, Idx>(
			effect: Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, EBrand, A>, Idx>,
			EBrand: Kind_cdc7cd43dac7585f + 'a, {
			let coyo: Coyoneda<'a, EBrand, A> = Coyoneda::lift(effect);
			let layer = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) as Member<
				Coyoneda<'a, EBrand, A>,
				Idx,
			>>::inject(coyo);
			Self::send(Node::First(layer))
		}

		/// Sequences this `RunExplicit` with a continuation `f`.
		/// Delegates to [`FreeExplicit::bind`](crate::types::FreeExplicit).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to chain after this computation.")]
		///
		#[document_returns("A new `RunExplicit` chaining `f` after this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	RunExplicit::pure(2).bind(|x| RunExplicit::pure(x + 1)).bind(|x| RunExplicit::pure(x * 10));
		/// assert_eq!(run.into_free_explicit().evaluate(), 30);
		/// ```
		#[inline]
		pub fn bind<B: 'a>(
			self,
			f: impl Fn(A) -> RunExplicit<'a, R, S, B> + 'a,
		) -> RunExplicit<'a, R, S, B> {
			RunExplicit::from_free_explicit(self.0.bind(move |a| f(a).into_free_explicit()))
		}

		/// Functor map over the result of this `RunExplicit`.
		/// Implemented via [`bind`](RunExplicit::bind) and
		/// [`pure`](RunExplicit::pure) (the underlying
		/// [`FreeExplicit`](crate::types::FreeExplicit) does not ship an
		/// inherent `map`).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `RunExplicit` with `f` applied to its result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RunExplicit<'_, FirstRow, Scoped, i32> = RunExplicit::pure(7).map(|x| x * 3);
		/// assert_eq!(run.into_free_explicit().evaluate(), 21);
		/// ```
		#[inline]
		pub fn map<B: 'a>(
			self,
			f: impl Fn(A) -> B + 'a,
		) -> RunExplicit<'a, R, S, B> {
			self.bind(move |a| RunExplicit::pure(f(a)))
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RunExplicit` instance.")]
	impl<'a, R, S, A: 'a> RunExplicit<'a, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Interprets this `RunExplicit` program by walking each
		/// effect via the matching handler closure in `handlers`,
		/// looping until the program reduces to a
		/// [`Pure`](crate::types::FreeExplicit) value.
		///
		/// Lifetime-flexible variant of [`Run::interpret`](crate::types::effects::run::Run::interpret).
		/// `RunExplicit`'s `'a` payload constraint flows into the
		/// handler list's closures, which receive the program-level
		/// `RunExplicit<'a, R, CNilBrand, A>` as the [`Coyoneda`] inner type.
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
		/// 			run_explicit::RunExplicit,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let result = prog.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<RunExplicit<'static, FirstRow, Scoped, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		pub fn interpret(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RunExplicit<'a, R, S, A>>),
				RunExplicit<'a, R, S, A>,
			>,
			scoped_handlers: impl DispatchScopedHandlers<
				'a,
				Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>),
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>),
				RunExplicit<'a, R, S, A>,
			>,
		) -> A {
			let mut prog = self;
			loop {
				match prog.peel() {
					Ok(a) => return a,
					Err(Node::First(layer)) => prog = handlers.dispatch(layer),
					Err(Node::Scoped(layer)) =>
						prog = scoped_handlers.dispatch_scoped(layer, &handlers),
				}
			}
		}

		/// Alias for [`interpret`](RunExplicit::interpret), kept for
		/// naming parity with PureScript Run's
		/// [`run`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs).
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
		/// 			run_explicit::RunExplicit,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RunExplicit::lift::<IdentityBrand, _>(Identity(99));
		/// let result = prog.run(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<RunExplicit<'static, FirstRow, Scoped, i32>>| op.0,
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
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RunExplicit<'a, R, S, A>>),
				RunExplicit<'a, R, S, A>,
			>,
			scoped_handlers: impl DispatchScopedHandlers<
				'a,
				Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>),
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>),
				RunExplicit<'a, R, S, A>,
			>,
		) -> A {
			self.interpret(handlers, scoped_handlers)
		}

		/// MonadRec-target interpreter for [`RunExplicit`]. Mirrors
		/// [`Run::interpret_rec`](crate::types::effects::run::Run::interpret_rec);
		/// see that method's docs for the handler shape, loop body, and
		/// stack-safety guarantee.
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
		/// 			run_explicit::RunExplicit,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let result: Thunk<'static, i32> = prog.interpret_rec::<ThunkBrand>(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Thunk<'static, RunExplicit<'static, FirstRow, Scoped, i32>>>| op.0,
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
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
			> + 'a,
			scoped_handlers: impl DispatchScopedHandlers<
				'a,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
			> + 'a,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			MBrand: MonadRec + 'static,
			A: 'a, {
			tail_rec_m::<MBrand, RunExplicit<'a, R, S, A>, A>(
				move |prog: RunExplicit<'a, R, S, A>| match prog.peel() {
					Ok(a) => <MBrand as Pointed>::pure::<ControlFlow<A, RunExplicit<'a, R, S, A>>>(
						ControlFlow::Break(a),
					),
					Err(Node::First(layer)) => {
						let mapped = <R as Functor>::map(
							|inner: RunExplicit<'a, R, S, A>| {
								<MBrand as Pointed>::pure::<RunExplicit<'a, R, S, A>>(inner)
							},
							layer,
						);
						let next = handlers.dispatch(mapped);
						<MBrand as Functor>::map::<
							RunExplicit<'a, R, S, A>,
							ControlFlow<A, RunExplicit<'a, R, S, A>>,
						>(ControlFlow::Continue, next)
					}
					Err(Node::Scoped(layer)) => {
						let mapped = <S as Functor>::map(
							|inner: RunExplicit<'a, R, S, A>| {
								<MBrand as Pointed>::pure::<RunExplicit<'a, R, S, A>>(inner)
							},
							layer,
						);
						let next = scoped_handlers.dispatch_scoped(mapped, &handlers);
						<MBrand as Functor>::map::<
							RunExplicit<'a, R, S, A>,
							ControlFlow<A, RunExplicit<'a, R, S, A>>,
						>(ControlFlow::Continue, next)
					}
				},
				self,
			)
		}

		/// Alias for [`interpret_rec`](RunExplicit::interpret_rec),
		/// kept for naming parity with PureScript Run's
		/// [`runRec`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs).
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
		/// 			run_explicit::RunExplicit,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RunExplicit::lift::<IdentityBrand, _>(Identity(99));
		/// let result: Thunk<'static, i32> = prog.run_rec::<ThunkBrand>(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Thunk<'static, RunExplicit<'static, FirstRow, Scoped, i32>>>| op.0,
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
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
			> + 'a,
			scoped_handlers: impl DispatchScopedHandlers<
				'a,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
			> + 'a,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			MBrand: MonadRec + 'static,
			A: 'a, {
			self.interpret_rec::<MBrand>(handlers, scoped_handlers)
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RunExplicit` instance.")]
	impl<'a, R, S, A: 'a> RunExplicit<'a, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Scoped-row-narrowing interpreter: interpret a single scoped
		/// effect `SBrand` out of the scoped row, returning a
		/// `RunExplicit` program in the narrowed scoped row `SMinusE`.
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
		#[document_returns("A `RunExplicit` program in the narrowed scoped row `SMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ToDynFnOnce,
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::{
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 			run_explicit::RunExplicit,
		/// 			span::BoxSpan,
		/// 		},
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
		///
		/// let action: RunExplicit<'static, CNilBrand, ScopedRow, i32> = RunExplicit::pure(7);
		/// let action_free = Box::new(action.into_free_explicit());
		/// let layer = Coproduct::Inl(BoxSpan::Span {
		/// 	tag: "request",
		/// 	action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action_free),
		/// });
		/// let prog: RunExplicit<'static, CNilBrand, ScopedRow, i32> =
		/// 	RunExplicit::from_free_explicit(FreeExplicit::wrap(Node::Scoped(layer)));
		/// let narrowed: RunExplicit<'static, CNilBrand, CNilBrand, i32> = prog
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
				Apply!(<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, SMinusE, A>>),
			) -> RunExplicit<'a, R, SMinusE, A>
			+ 'a,
		) -> RunExplicit<'a, R, SMinusE, A>
		where
			SBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>):
				Member<
						Apply!(
							<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>
						),
						Idx,
						Remainder = Apply!(
										<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>
									),
					>, {
			let handler = <RcBrand as RefCountedPointer>::new(handler);
			self.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, _>(handler)
		}

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
		#[document_returns("A `RunExplicit` program in the narrowed scoped row `SMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// // Exercised internally by RunExplicit::interpret_scoped_with.
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ToDynFnOnce,
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::{
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 			run_explicit::RunExplicit,
		/// 			span::BoxSpan,
		/// 		},
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
		///
		/// let action: RunExplicit<'static, CNilBrand, ScopedRow, i32> = RunExplicit::pure(7);
		/// let action_free = Box::new(action.into_free_explicit());
		/// let layer = Coproduct::Inl(BoxSpan::Span {
		/// 	tag: "request",
		/// 	action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action_free),
		/// });
		/// let prog: RunExplicit<'static, CNilBrand, ScopedRow, i32> =
		/// 	RunExplicit::from_free_explicit(FreeExplicit::wrap(Node::Scoped(layer)));
		/// let narrowed: RunExplicit<'static, CNilBrand, CNilBrand, i32> = prog
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
		fn interpret_scoped_with_shared<SBrand, Idx, SMinusE, F>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'a, F>,
		) -> RunExplicit<'a, R, SMinusE, A>
		where
			F: Fn(
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							RunExplicit<'a, R, SMinusE, A>,
						>
					),
				) -> RunExplicit<'a, R, SMinusE, A>
				+ 'a,
			SBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>):
				Member<
						Apply!(
							<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>
						),
						Idx,
						Remainder = Apply!(
										<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>
									),
					>, {
			match self.peel() {
				Ok(a) => RunExplicit::pure(a),
				Err(Node::First(layer)) => {
					let h_for_recurse = handler.clone();
					let mapped_boxed = <R as Functor>::map(
						move |inner: RunExplicit<'a, R, S, A>| {
							Box::new(
								inner
									.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
										h_for_recurse.clone(),
									)
									.into_free_explicit(),
							)
						},
						layer,
					);
					RunExplicit::from_free_explicit(
						FreeExplicit::<'a, NodeBrand<R, SMinusE>, A>::wrap(Node::First(
							mapped_boxed,
						)),
					)
				}
				Err(Node::Scoped(layer)) =>
					match <Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RunExplicit<'a, R, S, A>,
					>) as Member<
						Apply!(
							<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'a,
								RunExplicit<'a, R, S, A>,
							>
						),
						Idx,
					>>::project(layer)
					{
						Ok(scoped) => {
							let h_for_recurse = handler.clone();
							let mapped = <SBrand as Functor>::map(
								move |inner: RunExplicit<'a, R, S, A>| {
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
							let mapped_boxed = <SMinusE as Functor>::map(
								move |inner: RunExplicit<'a, R, S, A>| {
									Box::new(
										inner
											.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
												h_for_recurse.clone(),
											)
											.into_free_explicit(),
									)
								},
								rest,
							);
							RunExplicit::from_free_explicit(FreeExplicit::<
								'a,
								NodeBrand<R, SMinusE>,
								A,
							>::wrap(Node::Scoped(
								mapped_boxed,
							)))
						}
					},
			}
		}

		/// Pipeline row-narrowing interpreter. See
		/// [`Run::interpret_with`](crate::types::effects::run::Run::interpret_with)
		/// for the cross-wrapper semantics. Differences for
		/// `RunExplicit`: the Box-in-Wrap substrate
		/// (Coyoneda variant: bare [`Coyoneda`]); recursion uses
		/// [`FreeExplicit::wrap`](crate::types::FreeExplicit) which
		/// expects the inner program type to be wrapped in a
		/// [`Box`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect being interpreted out of the row.",
			"The type-level position witness (typically inferred).",
			"The narrowed row brand."
		)]
		///
		#[document_parameters("The handler closure for the targeted effect.")]
		///
		#[document_returns("A `RunExplicit` program in the narrowed row.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FullRow, CNilBrand, i32> =
		/// 	RunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: RunExplicit<'static, EmptyRow, CNilBrand, i32> = prog
		/// 	.interpret_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<RunExplicit<'static, EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		pub fn interpret_with<EBrand, Idx, RMinusE>(
			self,
			handler: impl Fn(
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, RMinusE, S, A>>),
			) -> RunExplicit<'a, RMinusE, S, A>
			+ 'a,
		) -> RunExplicit<'a, RMinusE, S, A>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>):
				Member<
						Coyoneda<'a, EBrand, RunExplicit<'a, R, S, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>
									),
					>, {
			let handler = <RcBrand as RefCountedPointer>::new(handler);
			self.interpret_with_shared::<EBrand, Idx, RMinusE, _>(handler)
		}

		/// Inner pipeline-narrowing implementation, parameterised
		/// over the concrete handler closure type `F`. The public
		/// [`interpret_with`](RunExplicit::interpret_with) wraps
		/// the user handler in [`Rc<F>`](std::rc::Rc) once at
		/// entry and delegates here; recursive narrowing clones
		/// the [`Rc<F>`](std::rc::Rc) (refcount bump) instead of
		/// cloning the underlying closure, which is what drops
		/// the `Clone` bound from the user-facing API.
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
		#[document_returns("A `RunExplicit` program in the narrowed row `RMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// // Exercised internally by RunExplicit::interpret_with.
		/// let prog: RunExplicit<'static, FullRow, CNilBrand, i32> =
		/// 	RunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: RunExplicit<'static, EmptyRow, CNilBrand, i32> = prog
		/// 	.interpret_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<RunExplicit<'static, EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		fn interpret_with_shared<EBrand, Idx, RMinusE, F>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'a, F>,
		) -> RunExplicit<'a, RMinusE, S, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, RMinusE, S, A>>),
				) -> RunExplicit<'a, RMinusE, S, A>
				+ 'a,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>):
				Member<
						Coyoneda<'a, EBrand, RunExplicit<'a, R, S, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>
									),
					>, {
			match self.peel() {
				Ok(a) => RunExplicit::pure(a),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>
				) as Member<
					Coyoneda<'a, EBrand, RunExplicit<'a, R, S, A>>,
					Idx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let lowered = coyo.lower();
						let h_for_recurse = handler.clone();
						let mapped = <EBrand as Functor>::map(
							move |inner: RunExplicit<'a, R, S, A>| {
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
						let mapped_boxed = <RMinusE as Functor>::map(
							move |inner: RunExplicit<'a, R, S, A>| {
								Box::new(
									inner
										.interpret_with_shared::<EBrand, Idx, RMinusE, F>(
											h_for_recurse.clone(),
										)
										.into_free_explicit(),
								)
							},
							rest,
						);
						RunExplicit::from_free_explicit(
							FreeExplicit::<'a, NodeBrand<RMinusE, S>, A>::wrap(Node::First(
								mapped_boxed,
							)),
						)
					}
				},
				Err(Node::Scoped(layer)) => {
					let h_for_recurse = handler.clone();
					let mapped_boxed = <S as Functor>::map(
						move |inner: RunExplicit<'a, R, S, A>| {
							Box::new(
								inner
									.interpret_with_shared::<EBrand, Idx, RMinusE, F>(
										h_for_recurse.clone(),
									)
									.into_free_explicit(),
							)
						},
						layer,
					);
					RunExplicit::from_free_explicit(
						FreeExplicit::<'a, NodeBrand<RMinusE, S>, A>::wrap(Node::Scoped(
							mapped_boxed,
						)),
					)
				}
			}
		}

		/// Substrate-level row-preserving replacement primitive: walk
		/// this `RunExplicit` program, projecting each first-order
		/// dispatch against `EBrand`; replace every matched dispatch
		/// with the supplied `replacement` closure (applied to the
		/// lowered effect value), and re-emit non-matching dispatches
		/// in the same row. Direct analog of heftia's
		/// `interposeInWith` in substrate-primitive form, on the
		/// explicit-lifetime substrate.
		///
		/// Unlike [`interpret_with`](RunExplicit::interpret_with),
		/// `interpose` does not narrow the row: the matched arm
		/// produces a continuation in the same `R`, the unmatched arm
		/// walks the `Self::Remainder` (`RMinusE`) layer and embeds
		/// it back into `R` via [`CoproductEmbedder`](crate::types::effects::coproduct::CoproductEmbedder).
		/// This is the building block for scoped-effect handlers.
		///
		/// The user-facing closure is wrapped in an
		/// [`Rc`](std::rc::Rc) once at entry; recursive calls clone
		/// the [`Rc`](std::rc::Rc) (refcount bump) instead of cloning
		/// the underlying closure, which is what drops the `Clone`
		/// bound from the user-facing API. The closure carries the
		/// same `'a` lifetime as the program (not `'static` like the
		/// non-Explicit family), so it can borrow from external state
		/// for the lifetime of the program.
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
		/// 		effects::run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = RunExplicit<'static, Row, CNilBrand, i32>;
		///
		/// let prog: Prog = RunExplicit::lift::<IdentityBrand, _>(Identity(7));
		/// let interposed = prog
		/// 	.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| RunExplicit::pure(99));
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
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
			) -> RunExplicit<'a, R, S, A>
			+ 'a,
		) -> RunExplicit<'a, R, S, A>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>):
				Member<
						Coyoneda<'a, EBrand, RunExplicit<'a, R, S, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>
									),
					>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				Box<FreeExplicit<'a, NodeBrand<R, S>, A>>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Box<FreeExplicit<'a, NodeBrand<R, S>, A>>,
				>),
					EmbedIndices,
				>, {
			let replacement = <RcBrand as RefCountedPointer>::new(replacement);
			self.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, _>(replacement)
		}

		/// Inner shared implementation of [`interpose`](RunExplicit::interpose),
		/// parameterised over the concrete replacement closure type
		/// `F`. The public [`interpose`](RunExplicit::interpose)
		/// wraps the user-supplied closure in [`Rc<F>`](std::rc::Rc)
		/// once at entry and delegates here; recursive descent clones
		/// the [`Rc<F>`](std::rc::Rc) (refcount bump) instead of
		/// cloning the underlying closure.
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
		/// // Exercised internally by RunExplicit::interpose.
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = RunExplicit<'static, Row, CNilBrand, i32>;
		///
		/// let prog: Prog = RunExplicit::lift::<IdentityBrand, _>(Identity(3));
		/// let interposed = prog
		/// 	.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| RunExplicit::pure(42));
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
			replacement: <RcBrand as RefCountedPointer>::Of<'a, F>,
		) -> RunExplicit<'a, R, S, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
				) -> RunExplicit<'a, R, S, A>
				+ 'a,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>):
				Member<
						Coyoneda<'a, EBrand, RunExplicit<'a, R, S, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>
									),
					>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				Box<FreeExplicit<'a, NodeBrand<R, S>, A>>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Box<FreeExplicit<'a, NodeBrand<R, S>, A>>,
				>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => RunExplicit::pure(a),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>
				) as Member<
					Coyoneda<'a, EBrand, RunExplicit<'a, R, S, A>>,
					Idx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let lowered = coyo.lower();
						let r_for_recurse = replacement.clone();
						let mapped = <EBrand as Functor>::map(
							move |inner: RunExplicit<'a, R, S, A>| {
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
							move |inner: RunExplicit<'a, R, S, A>| {
								Box::new(
									inner
										.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
											r_for_recurse.clone(),
										)
										.into_free_explicit(),
								)
							},
							rest,
						);
						let layer_back = mapped_rest.embed();
						RunExplicit::from_free_explicit(
							FreeExplicit::<'a, NodeBrand<R, S>, A>::wrap(Node::First(layer_back)),
						)
					}
				},
				Err(Node::Scoped(layer)) => {
					let r_for_recurse = replacement.clone();
					let mapped_boxed = <S as Functor>::map(
						move |inner: RunExplicit<'a, R, S, A>| {
							Box::new(
								inner
									.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
										r_for_recurse.clone(),
									)
									.into_free_explicit(),
							)
						},
						layer,
					);
					RunExplicit::from_free_explicit(FreeExplicit::<'a, NodeBrand<R, S>, A>::wrap(
						Node::Scoped(mapped_boxed),
					))
				}
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The first-order-only `RunExplicit` instance.")]
	impl<'a, R, A: 'a> RunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
	{
		/// Substrate-level matched-effect short-circuit primitive on
		/// the explicit-lifetime Erased Run wrapper: walk this
		/// `RunExplicit` program, dispatching non-matched first-order
		/// effects through `fo_handlers` and short-circuiting the
		/// moment a matched-effect (`EBrand`) dispatch is encountered,
		/// returning the matched effect's lowered payload.
		///
		/// Returns `Ok(a)` when the program reduces to a pure value
		/// without firing the matched effect; returns `Err(op)` with
		/// the matched effect's lowered payload otherwise.
		///
		/// `fo_handlers` covers only the non-matched effects (the
		/// `RMinusE` row); the program type retains the full row `R`.
		/// The handler list and matched effect both carry the program's
		/// `'a` lifetime (not `'static` like the non-Explicit family).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the matched effect.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand."
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
		/// 			run_explicit::RunExplicit,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<
		/// 	CoyonedaBrand<ExceptBrand<String>>,
		/// 	CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>,
		/// >;
		/// type RowMinusExcept = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = RunExplicit<'static, Row, CNilBrand, i32>;
		///
		/// let prog: Prog = RunExplicit::throw::<String, _>("oops".to_string());
		/// let result: Result<i32, Except<'static, String, Prog>> = prog
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
				Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RunExplicit<'a, R, CNilBrand, A>>),
				RunExplicit<'a, R, CNilBrand, A>,
			>,
		) -> Result<
			A,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, CNilBrand, A>>),
		>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, CNilBrand, A>>):
				Member<
						Coyoneda<'a, EBrand, RunExplicit<'a, R, CNilBrand, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, CNilBrand, A>>
									),
					>, {
			let mut prog = self;
			loop {
				match prog.peel() {
					Ok(a) => return Ok(a),
					Err(Node::First(layer)) =>
						match <Apply!(
							<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, CNilBrand, A>>
						) as Member<Coyoneda<'a, EBrand, RunExplicit<'a, R, CNilBrand, A>>, Idx>>::project(
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

	#[document_type_parameters("The lifetime that bounds the payload.", "The result type.")]
	#[document_parameters("The `RunExplicit` instance.")]
	impl<'a, A: 'a> RunExplicit<'a, CNilBrand, CNilBrand, A> {
		/// Extracts the result value from a `RunExplicit` program whose
		/// first-order and scoped rows have both been fully interpreted
		/// away. Exhaustive `match` over the uninhabited `CNil` payloads
		/// proves no runtime panic, statically. See
		/// [`Run::extract`](crate::types::effects::run::Run::extract).
		#[document_signature]
		///
		#[document_returns("The final result value of the fully-narrowed program.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// let pure_prog: RunExplicit<'_, CNilBrand, CNilBrand, i32> = RunExplicit::pure(42);
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
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The state type (also the program's result type for `get`)."
	)]
	impl<'a, R, ScopedRow, A: 'a> RunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		/// Lifts a `Get` state effect into the `RunExplicit` program.
		/// Mirrors [`Run::get`](crate::types::effects::run::Run::get);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RunExplicit`: the bare [`Coyoneda`] variant pairs with the
		/// Box-in-Wrap Explicit substrate (the substrate's `peel` does
		/// not require a `Clone` bound on the inner effect). Threads
		/// [`BoxBrand`](crate::brands::BoxBrand) as the pointer kind
		/// post-Phase-3.5 retrofit so the continuation projection is
		/// `Box<dyn FnOnce>` rather than `Rc<dyn Fn>`.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("A `RunExplicit` program suspended at the lifted `Get` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		run_explicit::RunExplicit,
		/// 		state::BoxState,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> = RunExplicit::get();
		/// // The program is suspended at the Get effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn get<Idx>() -> Self
		where
			A: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Member<
					Coyoneda<'a, crate::brands::BoxStateBrand<crate::brands::BoxBrand, A>, A>,
					Idx,
				>, {
			let effect: crate::types::effects::state::BoxState<'a, crate::brands::BoxBrand, A, A> =
				crate::types::effects::state::BoxState::Get(
					<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|s: A| s),
				);
			Self::lift::<crate::brands::BoxStateBrand<crate::brands::BoxBrand, A>, Idx>(effect)
		}

		/// Lifts an `Ask` reader effect into the `RunExplicit`
		/// program. Mirrors
		/// [`Run::ask`](crate::types::effects::run::Run::ask); see
		/// that method for cross-wrapper semantics. Differences for
		/// `RunExplicit`: the bare [`Coyoneda`] variant pairs with the
		/// Box-in-Wrap Explicit substrate. Threads
		/// [`BoxBrand`](crate::brands::BoxBrand) as the pointer kind
		/// post-Phase-3.5 retrofit.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("A `RunExplicit` program suspended at the lifted `Ask` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		reader::BoxReader,
		/// 		run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> = RunExplicit::ask();
		/// // The program is suspended at the Ask effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn ask<Idx>() -> Self
		where
			A: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Member<
					Coyoneda<'a, crate::brands::BoxReaderBrand<crate::brands::BoxBrand, A>, A>,
					Idx,
				>, {
			let effect: crate::types::effects::reader::BoxReader<
				'a,
				crate::brands::BoxBrand,
				A,
				A,
			> = crate::types::effects::reader::BoxReader::Ask(
				<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|e: A| e),
			);
			Self::lift::<crate::brands::BoxReaderBrand<crate::brands::BoxBrand, A>, Idx>(effect)
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
		/// 	types::effects::{
		/// 		except::Except,
		/// 		run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RunExplicit::throw::<&'static str, _>("oops");
		/// assert!(prog.peel().is_err());
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
		/// let result = program.interpret(
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
		pub fn catch<E: 'a, Idx>(
			action: RunExplicit<'a, R, ScopedRow, A>,
			handler: impl FnOnce(E) -> RunExplicit<'a, R, ScopedRow, A> + 'a,
		) -> RunExplicitBoundary<
			'a,
			R,
			ScopedRow,
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
		/// let result = program.interpret(
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
		pub fn local<E: 'a, Idx>(
			modify: impl FnOnce(E) -> E + 'a,
			action: RunExplicit<'a, R, ScopedRow, A>,
		) -> RunExplicitBoundary<
			'a,
			R,
			ScopedRow,
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
		/// let result = program.interpret(
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
		pub fn ref_local<E: 'a, Idx>(
			modify: impl FnOnce(&E) -> E + 'a,
			action: RunExplicit<'a, R, ScopedRow, A>,
		) -> RunExplicitBoundary<
			'a,
			R,
			ScopedRow,
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
		pub fn span<Tag: 'a, Idx>(
			tag: Tag,
			action: RunExplicit<'a, R, ScopedRow, A>,
		) -> RunExplicitBoundary<
			'a,
			R,
			ScopedRow,
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
		/// Lifts a `Put` state effect into the `RunExplicit` program.
		/// Mirrors [`Run::put`](crate::types::effects::run::Run::put);
		/// see that method for cross-wrapper semantics. Threads
		/// [`BoxBrand`](crate::brands::BoxBrand) as the pointer kind
		/// post-Phase-3.5 retrofit so the continuation projection is
		/// `Box<dyn FnOnce>`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The state type carried by `BoxStateBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The new state value to write.")]
		///
		#[document_returns("A `RunExplicit` program suspended at the lifted `Put` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		run_explicit::RunExplicit,
		/// 		state::BoxState,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, ()> = RunExplicit::put::<i32, _>(42);
		/// // The program is suspended at the Put effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn put<StateType: 'static, Idx>(s: StateType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>): Member<
					Coyoneda<
						'a,
						crate::brands::BoxStateBrand<crate::brands::BoxBrand, StateType>,
						(),
					>,
					Idx,
				>, {
			let effect: crate::types::effects::state::BoxState<
				'a,
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
		/// 	types::effects::{
		/// 		run_explicit::RunExplicit,
		/// 		writer::Writer,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<WriterBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, ()> =
		/// 	RunExplicit::tell::<&'static str, _>("logged");
		/// assert!(prog.peel().is_err());
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

	// -- From<Run> for RunExplicit (Erased -> Explicit conversion) --

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, S, A> From<Run<R, S, A>> for RunExplicit<'static, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Converts a [`Run<R, S, A>`](crate::types::effects::run::Run)
		/// into the paired Explicit-substrate form by walking the
		/// underlying [`Free`](crate::types::Free) chain via
		/// [`peel`](Run::peel) and rebuilding each suspended layer
		/// through [`FreeExplicit::wrap`](crate::types::FreeExplicit).
		/// Pure values re-emerge as
		/// [`RunExplicit::pure`](RunExplicit::pure).
		///
		/// O(N) in chain depth (one stack frame per suspended layer);
		/// per the structural Wrap-depth probe at
		/// `fp-library/tests/run_wrap_depth_probe.rs`, Run-typical
		/// patterns have depth at most 1, so the recursion is
		/// constant in practice.
		#[document_signature]
		///
		#[document_parameters("The Erased-substrate `Run` to convert.")]
		///
		#[document_returns("A `RunExplicit` carrying the same effects.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> = Run::pure(42);
		/// // Both call styles work via the blanket `Into` impl.
		/// let from_style: RunExplicit<'static, FirstRow, Scoped, i32> = RunExplicit::from(run);
		/// assert!(matches!(from_style.peel(), Ok(42)));
		/// let run2: Run<FirstRow, Scoped, i32> = Run::pure(42);
		/// let into_style: RunExplicit<'static, FirstRow, Scoped, i32> = run2.into();
		/// assert!(matches!(into_style.peel(), Ok(42)));
		/// ```
		fn from(run: Run<R, S, A>) -> Self {
			match run.peel() {
				Ok(a) => RunExplicit::pure(a),
				Err(layer) => {
					let boxed = <NodeBrand<R, S> as Functor>::map(
						|inner: Run<R, S, A>| -> Box<FreeExplicit<'static, NodeBrand<R, S>, A>> {
							Box::new(RunExplicit::from(inner).into_free_explicit())
						},
						layer,
					);
					RunExplicit::from_free_explicit(FreeExplicit::wrap(boxed))
				}
			}
		}
	}

	// -- Brand-level type class instances --
	//
	// Each impl converts the wrapper to its underlying `FreeExplicit`,
	// dispatches through `FreeExplicitBrand<NodeBrand<R, S>>`, and
	// re-wraps the result. `Monad` / `RefMonad` are not implemented:
	// the blanket impl requires `Applicative` / `RefApplicative`, which
	// `FreeExplicitBrand` deliberately does not provide (see
	// `free_explicit.rs` lines 369-388).

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> Functor for RunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Maps a function over the result of a `RunExplicit` computation
		/// by delegating to
		/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s
		/// [`Functor::map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the row brands.",
			"The original result type.",
			"The new result type."
		)]
		///
		#[document_parameters(
			"The function to apply to the result.",
			"The `RunExplicit` computation."
		)]
		///
		#[document_returns("A new `RunExplicit` with the function applied to its result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(10);
		/// let mapped = <RunExplicitBrand<FirstRow, Scoped> as Functor>::map(|x: i32| x * 2, run);
		/// assert_eq!(mapped.into_free_explicit().evaluate(), 20);
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			RunExplicit::from_free_explicit(<FreeExplicitBrand<NodeBrand<R, S>> as Functor>::map(
				f,
				fa.into_free_explicit(),
			))
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> Pointed for RunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Wraps a value in a pure `RunExplicit` computation by
		/// delegating to
		/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s
		/// [`Pointed::pure`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the row brands.",
			"The type of the value to wrap."
		)]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A `RunExplicit` computation that produces `a`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RunExplicit<'_, FirstRow, Scoped, _> =
		/// 	<RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(42);
		/// assert_eq!(run.into_free_explicit().evaluate(), 42);
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			RunExplicit::from_free_explicit(<FreeExplicitBrand<NodeBrand<R, S>> as Pointed>::pure(
				a,
			))
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> Semimonad for RunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Sequences `RunExplicit` computations by delegating to
		/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s
		/// [`Semimonad::bind`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the row brands.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first `RunExplicit` computation.",
			"The function to chain after the first computation."
		)]
		///
		#[document_returns("A new `RunExplicit` chaining the function after `ma`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(2);
		/// let chained = <RunExplicitBrand<FirstRow, Scoped> as Semimonad>::bind(run, |x: i32| {
		/// 	<RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(x + 1)
		/// });
		/// assert_eq!(chained.into_free_explicit().evaluate(), 3);
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			RunExplicit::from_free_explicit(
				<FreeExplicitBrand<NodeBrand<R, S>> as Semimonad>::bind(
					ma.into_free_explicit(),
					move |a| func(a).into_free_explicit(),
				),
			)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> RefFunctor for RunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + RefFunctor + 'static,
		S: WrapDrop + Functor + RefFunctor + 'static,
	{
		/// Maps a function over the result of a `RunExplicit` computation
		/// by reference, delegating to
		/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s
		/// [`RefFunctor::ref_map`].
		///
		/// Note: the canonical Run row using
		/// [`CoyonedaBrand`](crate::brands::CoyonedaBrand)-wrapped
		/// effects does not satisfy [`RefFunctor`] today, so this impl is
		/// reachable only for synthetic rows whose brands implement
		/// [`RefFunctor`] (e.g., `CoproductBrand<IdentityBrand, CNilBrand>`).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the row brands.",
			"The original result type.",
			"The new result type."
		)]
		///
		#[document_parameters(
			"The function to apply to the result by reference.",
			"The `RunExplicit` computation."
		)]
		///
		#[document_returns("A new `RunExplicit` with the function applied to its result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(10);
		/// let mapped =
		/// 	<RunExplicitBrand<FirstRow, Scoped> as RefFunctor>::ref_map(|x: &i32| *x * 2, &run);
		/// assert_eq!(mapped.into_free_explicit().evaluate(), 20);
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			RunExplicit::from_free_explicit(
				<FreeExplicitBrand<NodeBrand<R, S>> as RefFunctor>::ref_map(func, &fa.0),
			)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> RefPointed for RunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Wraps a cloned value in a pure `RunExplicit` computation by
		/// delegating to
		/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s
		/// [`RefPointed::ref_pure`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the row brands.",
			"The type of the value to wrap. Must be `Clone`."
		)]
		///
		#[document_parameters("A reference to the value to wrap.")]
		///
		#[document_returns("A `RunExplicit` computation that produces a clone of `a`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let value = 42;
		/// let run: RunExplicit<'_, FirstRow, Scoped, _> =
		/// 	<RunExplicitBrand<FirstRow, Scoped> as RefPointed>::ref_pure(&value);
		/// assert_eq!(run.into_free_explicit().evaluate(), 42);
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			RunExplicit::from_free_explicit(
				<FreeExplicitBrand<NodeBrand<R, S>> as RefPointed>::ref_pure(a),
			)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> RefSemimonad for RunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + RefFunctor + 'static,
		S: WrapDrop + Functor + RefFunctor + 'static,
	{
		/// Sequences `RunExplicit` computations using a reference to the
		/// intermediate value, delegating to
		/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s
		/// [`RefSemimonad::ref_bind`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the row brands.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first `RunExplicit` computation.",
			"The function to chain after the first computation."
		)]
		///
		#[document_returns("A new `RunExplicit` chaining the function after `ma`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(2);
		/// let chained =
		/// 	<RunExplicitBrand<FirstRow, Scoped> as RefSemimonad>::ref_bind(&run, |x: &i32| {
		/// 		<RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(*x + 1)
		/// 	});
		/// assert_eq!(chained.into_free_explicit().evaluate(), 3);
		/// ```
		fn ref_bind<'a, A: 'a, B: 'a>(
			ma: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			RunExplicit::from_free_explicit(
				<FreeExplicitBrand<NodeBrand<R, S>> as RefSemimonad>::ref_bind(&ma.0, move |a| {
					f(a).into_free_explicit()
				}),
			)
		}
	}
}

pub use inner::*;

#[cfg(test)]
mod tests;
