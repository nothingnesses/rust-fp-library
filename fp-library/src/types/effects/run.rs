//! Erased-substrate Run program over [`Free`](crate::types::Free) and a
//! dual-row [`NodeBrand`](crate::brands::NodeBrand).
//!
//! `Run<R, S, A>` is the user-facing wrapper for the canonical
//! Run-style effect computation:
//!
//! ```text
//! Run<R, S, A> = Free<NodeBrand<R, S>, A>
//! ```
//!
//! The first-order row brand `R` carries the effect functors (typically
//! a [`CoproductBrand`](crate::brands::CoproductBrand) of
//! [`CoyonedaBrand`](crate::brands::CoyonedaBrand)-wrapped effects
//! terminated by [`CNilBrand`](crate::brands::CNilBrand)); the scoped
//! row brand `S` carries higher-order constructors (future scoped
//! work populates it with `Catch`, `Local`, etc.; for
//! first-order-only programs it stays as `CNilBrand`).
//!
//! `Run` is the Erased counterpart of `RunExplicit`. The Erased
//! substrate is single-shot, type-erases through `Box<dyn Any>`, has
//! O(1) `bind`, and is `'static`-only. It exposes its API via
//! inherent methods rather than Brand-dispatched type classes, so
//! do-notation is via the `run_do!` macro, not `m_do!`. Use
//! `RunExplicit` for non-`'static` payloads or when Brand-dispatched
//! typeclass-generic code is required.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				CNilBrand,
				NodeBrand,
				RcBrand,
			},
			classes::{
				Functor,
				MonadRec,
				Pointed,
				RefCountedPointer,
				WrapDrop,
			},
			functions::tail_rec_m,
			kinds::*,
			types::{
				Coyoneda,
				Free,
				effects::{
					coproduct::CoproductEmbedder,
					interpreter::{
						DispatchHandlers,
						DispatchScopedHandlers,
					},
					member::Member,
					node::Node,
				},
			},
		},
		core::ops::ControlFlow,
		fp_macros::*,
	};

	/// Erased-substrate Run program: a thin wrapper over
	/// [`Free<NodeBrand<R, S>, A>`](crate::types::Free).
	///
	/// The wrapper exists so user-facing API (`pure`, `peel`, `send`,
	/// effect-row narrowing, handler types) can be expressed without
	/// leaking the underlying Free representation. It is a tuple
	/// struct over the inner Free; converting back via
	/// [`into_free`](Run::into_free) is a zero-cost move.
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand (typically `CNilBrand` for first-order-only programs).",
		"The result type."
	)]
	pub struct Run<R, S, A>(Free<NodeBrand<R, S>, A>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static;

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The Run instance.")]
	impl<R, S, A> Run<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Wraps a [`Free<NodeBrand<R, S>, A>`](crate::types::Free) as
		/// a `Run<R, S, A>`. Zero-cost.
		#[document_signature]
		///
		#[document_parameters("The underlying Free computation.")]
		///
		#[document_returns("A `Run` wrapping `free`.")]
		///
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
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let free: Free<NodeBrand<FirstRow, Scoped>, i32> = Free::pure(7);
		/// let run: Run<FirstRow, Scoped, i32> = Run::from_free(free);
		/// assert!(matches!(run.peel(), Ok(7)));
		/// ```
		#[inline]
		pub fn from_free(free: Free<NodeBrand<R, S>, A>) -> Self {
			Run(free)
		}

		/// Unwraps a `Run<R, S, A>` to its underlying
		/// [`Free<NodeBrand<R, S>, A>`](crate::types::Free).
		/// Zero-cost.
		#[document_signature]
		///
		#[document_returns("The underlying Free computation.")]
		///
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
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> = Run::from_free(Free::pure(7));
		/// let free: Free<NodeBrand<FirstRow, Scoped>, i32> = run.into_free();
		/// assert!(matches!(free.resume(), Ok(7)));
		/// ```
		#[inline]
		pub fn into_free(self) -> Free<NodeBrand<R, S>, A> {
			self.0
		}

		/// Wraps a value in a pure `Run` computation. Delegates to
		/// [`Free::pure`](crate::types::Free).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A `Run` computation that produces `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> = Run::pure(42);
		/// assert!(matches!(run.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self {
			Run::from_free(Free::pure(a))
		}

		/// Decomposes this `Run` computation into one step. Returns
		/// `Ok(a)` if the program is a pure value, or `Err(layer)` if
		/// it is suspended in the dual-row
		/// [`Node`](crate::types::effects::node::Node) dispatch enum,
		/// where `layer` carries the next `Run` continuation.
		///
		/// Delegates to [`Free::resume`](crate::types::Free).
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` for a pure result, or `Err(layer)` carrying the next `Run` step in a `Node` layer."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> = Run::pure(7);
		/// assert!(matches!(run.peel(), Ok(7)));
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "Return type encodes Result<A, NodeBrand<R, S>::Of<'static, Run<R, S, A>>>; the GAT projection is structurally complex but cannot be aliased without losing the projection link the wrapper depends on."
		)]
		pub fn peel(
			self
		) -> Result<
			A,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
		> {
			self.0.resume().map_err(|node| <NodeBrand<R, S> as Functor>::map(Run::from_free, node))
		}

		/// Lifts a [`Node`](crate::types::effects::node::Node) dispatch layer into the `Run` program.
		/// The `node` argument is a value of
		/// [`NodeBrand<R, S>`](crate::brands::NodeBrand)'s
		/// `Of<'static, A>` projection (typically constructed via
		/// `Node::First(<R as Member<...>>::inject(operation))` for a
		/// first-order effect, or `Node::Scoped(...)` for a scoped
		/// effect); `send` delegates to
		/// [`Free::lift_f`](crate::types::Free).
		///
		/// The `Node`-projection signature (rather than a row-variant
		/// signature) is required so the same shape works across all
		/// six Run wrappers, including
		/// [`ArcRun`](crate::types::effects::arc_run::ArcRun) and
		/// [`ArcRunExplicit`](crate::types::effects::arc_run_explicit::ArcRunExplicit).
		/// Constructing the [`Node`](crate::types::effects::node::Node) literal inside an `Arc`-substrate
		/// method body fails GAT normalization (see
		/// `tests/arc_run_normalization_probe.rs`); accepting an
		/// already-projection-typed parameter sidesteps that.
		#[document_signature]
		///
		#[document_parameters("The Node dispatch layer carrying the effect operation.")]
		///
		#[document_returns("A `Run` computation that performs the effect and returns its result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Coyoneda,
		/// 		Identity,
		/// 		effects::{
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 			run::Run,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(Identity(7));
		/// let layer = Coproduct::inject(coyo);
		/// let run: Run<FirstRow, Scoped, i32> = Run::send(Node::First(layer));
		/// // `send` produces a suspended program; peel returns Err
		/// // carrying the layer with the next continuation.
		/// assert!(run.peel().is_err());
		/// ```
		#[inline]
		pub fn send(
			node: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		) -> Self {
			Run::from_free(Free::<NodeBrand<R, S>, A>::lift_f(node))
		}

		/// Sequences this `Run` with a continuation `f`. Delegates to
		/// [`Free::bind`](crate::types::Free).
		///
		/// `f: FnOnce(A) -> Run<...>` is single-shot, mirroring
		/// [`Free`](crate::types::Free)'s `Box<dyn FnOnce>`-backed
		/// continuation queue. Handler closures stored in
		/// [`Handler<E, F>`](crate::types::effects::handlers::Handler)
		/// are bound `F: Fn` (the
		/// [`DispatchHandlers::dispatch`](crate::types::effects::interpreter::DispatchHandlers)
		/// receiver is `&self` so it can be called from inside
		/// [`MonadRec::tail_rec_m`](crate::classes::MonadRec)'s `Fn`
		/// step closure). Conversions between the two require
		/// interior mutability or refcounted captures
		/// (`Rc<RefCell<_>>`, `Arc<Mutex<_>>`).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to chain after this computation.")]
		///
		#[document_returns("A new `Run` chaining `f` after this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> =
		/// 	Run::pure(2).bind(|x| Run::pure(x + 1)).bind(|x| Run::pure(x * 10));
		/// assert!(matches!(run.peel(), Ok(30)));
		/// ```
		#[inline]
		pub fn bind<B: 'static>(
			self,
			f: impl FnOnce(A) -> Run<R, S, B> + 'static,
		) -> Run<R, S, B> {
			Run::from_free(self.0.bind(move |a| f(a).into_free()))
		}

		/// Functor map over the result of this `Run`. Delegates to
		/// [`Free::map`](crate::types::Free).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `Run` with `f` applied to its result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> = Run::pure(7).map(|x| x * 3);
		/// assert!(matches!(run.peel(), Ok(21)));
		/// ```
		#[inline]
		pub fn map<B: 'static>(
			self,
			f: impl FnOnce(A) -> B + 'static,
		) -> Run<R, S, B> {
			Run::from_free(self.0.map(f))
		}

		/// Lifts a raw effect value into a `Run` program.
		///
		/// Direct analog of PureScript Run's
		/// [`lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs).
		/// Takes the raw effect (an `EBrand`'s `Of<'static, A>`
		/// projection), wraps it in
		/// [`Coyoneda::lift`](crate::types::Coyoneda::lift) so any
		/// effect functor satisfies the row's
		/// [`Functor`](crate::classes::Functor) requirement, injects
		/// at the
		/// [`Member`](crate::types::effects::member::Member)-determined
		/// position, wraps in [`Node::First`](crate::types::effects::node::Node),
		/// and lifts via [`send`](Run::send). Per-effect smart
		/// constructors (`ask`, `get`, `put`, `tell`, `throw`) are
		/// one-liners over this combinator, mirroring PureScript
		/// Run's `liftEffect = lift (Proxy :: "effect")` pattern.
		///
		/// Naming note: PureScript Run distinguishes
		/// [`Run.lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// (the row-aware Run-level operation, which this method
		/// implements) from
		/// [`Free.liftF`](https://github.com/purescript/purescript-free/blob/main/src/Control/Monad/Free.purs)
		/// (the Free-monad-only lift, which fp-library exposes as
		/// [`Free::lift_f`](crate::types::Free::lift_f)). The bare
		/// name `lift` matches the row-aware operation; the `_f`
		/// suffix is reserved for the Free-only operation.
		///
		/// `Idx` is the type-level position witness. For an
		/// unambiguous row (each effect type appears once), Rust
		/// infers it; turbofish only when duplicate effect types make
		/// the position ambiguous.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect being lifted.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The effect value to lift.")]
		///
		#[document_returns("A `Run` program suspended at the lifted effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// // The program is suspended at the lifted effect; peel reveals the layer.
		/// assert!(run.peel().is_err());
		/// ```
		#[inline]
		pub fn lift<EBrand, Idx>(
			effect: Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<Coyoneda<'static, EBrand, A>, Idx>,
			EBrand: Kind_cdc7cd43dac7585f + 'static, {
			let coyo: Coyoneda<'static, EBrand, A> = Coyoneda::lift(effect);
			let layer =
				<Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>) as Member<
					Coyoneda<'static, EBrand, A>,
					Idx,
				>>::inject(coyo);
			Self::send(Node::First(layer))
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The Run instance.")]
	impl<R, S, A> Run<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Interprets this `Run` program by walking each effect via the
		/// matching handler closure in `handlers`, looping until the
		/// program reduces to a [`Pure`](crate::types::Free) value.
		///
		/// `handlers` is a handler list (typically built via the
		/// [`handlers!`](https://docs.rs/fp-macros/latest/fp_macros/macro.handlers.html)
		/// macro or the
		/// [`nt()`](crate::types::effects::handlers::nt) builder
		/// fallback) whose cells align cell-for-cell with the row
		/// brand chain `R`. Each cell carries a closure
		/// [`Handler<EBrand, F>`](crate::types::effects::handlers::Handler)
		/// of shape `FnMut(<EBrand as Kind>::Of<'_, Run<R, S, A>>) -> Run<R, S, A>`,
		/// taking the lowered [`Coyoneda`](crate::types::Coyoneda)
		/// payload and returning the next-step program in the same
		/// row.
		///
		/// Mirrors PureScript Run's
		/// [`interpret`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// (which is itself a literal alias for
		/// [`run`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)).
		/// The Rust port adopts a mono-in-`A` step-function shape so
		/// handler closures don't need rank-2 polymorphism (which
		/// Rust closures can't express). The scoped row `S` is fixed
		/// at [`CNilBrand`](crate::brands::CNilBrand) for the
		/// first-order interpreter; future scoped-effect work extends
		/// this to dispatch over scoped effects too.
		///
		/// ## Stack safety
		///
		/// This method recurses host-stack-frame per peeled layer.
		/// [`interpret_rec`](Run::interpret_rec) (and siblings) provide
		/// stack-safe interpretation via `MonadRec`.
		///
		/// ## Threading state through handlers
		///
		/// Handlers can capture an
		/// [`Rc<RefCell<S>>`](std::rc::Rc) (or
		/// [`Arc<Mutex<S>>`](std::sync::Arc) on thread-safe wrappers)
		/// to thread state through dispatch. Each `interpret` call only
		/// returns the program's `A`; the captured cell holds the
		/// final state for the caller to read after interpretation
		/// completes. The keyword "state" appears here for
		/// rustdoc-search discoverability of stateful interpretation.
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
		/// 			run::Run,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// // Lift an Identity effect, then interpret it: the handler
		/// // unwraps the `Identity` and returns a pure program.
		/// let prog: Run<FirstRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let result = prog.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Run<FirstRow, Scoped, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 42);
		/// ```
		///
		/// State threading via a captured cell:
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		handlers,
		/// 		types::{
		/// 			Identity,
		/// 			effects::{
		/// 				handlers::*,
		/// 				run::Run,
		/// 			},
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::RefCell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let counter: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
		/// let counter_for_handler = Rc::clone(&counter);
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(7));
		/// let result = prog.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: move |op: Identity<Run<FirstRow, Scoped, i32>>| {
		/// 			*counter_for_handler.borrow_mut() += 1;
		/// 			op.0
		/// 		},
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 7);
		/// assert_eq!(*counter.borrow(), 1);
		/// ```
		#[inline]
		pub fn interpret(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, S, A>>),
				Run<R, S, A>,
			>,
			scoped_handlers: impl for<'h> DispatchScopedHandlers<
				'h,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, S, A>>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, S, A>>),
				Run<R, S, A>,
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

		/// Alias for [`interpret`](Run::interpret), kept for naming
		/// parity with PureScript Run's
		/// [`run`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs).
		///
		/// In PureScript Run,
		/// `interpret :: (VariantF r ~> m) -> Run r a -> m a` carries
		/// a rank-2 natural-transformation signature while
		/// `run :: (VariantF r (Run r a) -> m (Run r a)) -> Run r a -> m a`
		/// carries the mono-in-`a` step-function form. The two
		/// implementations are literally aliased
		/// (`interpret = run` at
		/// [`Run.purs:184`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)).
		/// The Rust port has only the mono-in-`a` form (closures
		/// cannot be A-polymorphic); both names are exposed for
		/// PureScript-cross-reference convenience.
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
		/// 			run::Run,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(99));
		/// let result = prog.run(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Run<FirstRow, Scoped, i32>>| op.0,
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
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, S, A>>),
				Run<R, S, A>,
			>,
			scoped_handlers: impl for<'h> DispatchScopedHandlers<
				'h,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, S, A>>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, S, A>>),
				Run<R, S, A>,
			>,
		) -> A {
			self.interpret(handlers, scoped_handlers)
		}

		/// MonadRec-target interpreter: walk this `Run` program against
		/// a handler list, producing the result in an external monad
		/// `MBrand` rather than as a raw `A`. Stack-safe via
		/// [`tail_rec_m`](crate::functions::tail_rec_m), so unbounded
		/// chains over `MBrand: MonadRec` substrates such as
		/// [`ThunkBrand`](crate::brands::ThunkBrand),
		/// [`OptionBrand`](crate::brands::OptionBrand), or
		/// [`ResultBrand`](crate::brands::ResultBrand) do not blow the
		/// host stack.
		///
		/// Direct analog of PureScript Run's
		/// [`interpretRec`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// (which is itself a literal alias for
		/// [`runRec`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// over a `MonadRec m` constraint).
		///
		/// ## Handler shape
		///
		/// Each handler closure has shape
		/// `Fn(<EBrand as Kind>::Of<'_, M::Of<'_, Run<R, S, A>>>) -> M::Of<'_, Run<R, S, A>>`,
		/// matching PureScript Run's
		/// `(VariantF r (m (Run r a)) -> m (Run r a))`. Inner
		/// continuations arrive already lifted into the target monad
		/// (the interpreter does
		/// [`Functor::map`](crate::classes::Functor)`(M::pure, layer)`
		/// before dispatch), so handlers can sequence M-shaped work via
		/// M's bind/map directly.
		///
		/// ## Loop body
		///
		/// On each iteration the step closure peels the program; on
		/// `Ok(a)` it emits `M::pure(ControlFlow::Break(a))`; on
		/// `Err(Node::First(layer))` it lifts the layer's inner
		/// programs to M-wrapped, dispatches through the handler list,
		/// and fmaps `M::Of<Run<R, S, A>>` to
		/// `M::Of<ControlFlow<A, Run<R, S, A>>>` via
		/// [`ControlFlow::Continue`].
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
		/// 			run::Run,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let result: Thunk<'static, i32> = prog.interpret_rec::<ThunkBrand>(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Thunk<'static, Run<FirstRow, Scoped, i32>>>| op.0,
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
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			> + 'static,
			scoped_handlers: impl for<'h> DispatchScopedHandlers<
				'h,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			> + 'static,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		where
			MBrand: MonadRec + 'static, {
			tail_rec_m::<MBrand, Run<R, S, A>, A>(
				move |prog: Run<R, S, A>| match prog.peel() {
					Ok(a) => <MBrand as Pointed>::pure::<ControlFlow<A, Run<R, S, A>>>(
						ControlFlow::Break(a),
					),
					Err(Node::First(layer)) => {
						let mapped = <R as Functor>::map(
							|inner: Run<R, S, A>| <MBrand as Pointed>::pure::<Run<R, S, A>>(inner),
							layer,
						);
						let next = handlers.dispatch(mapped);
						<MBrand as Functor>::map::<Run<R, S, A>, ControlFlow<A, Run<R, S, A>>>(
							ControlFlow::Continue,
							next,
						)
					}
					Err(Node::Scoped(layer)) => {
						let mapped = <S as Functor>::map(
							|inner: Run<R, S, A>| <MBrand as Pointed>::pure::<Run<R, S, A>>(inner),
							layer,
						);
						let next = scoped_handlers.dispatch_scoped(mapped, &handlers);
						<MBrand as Functor>::map::<Run<R, S, A>, ControlFlow<A, Run<R, S, A>>>(
							ControlFlow::Continue,
							next,
						)
					}
				},
				self,
			)
		}

		/// Alias for [`interpret_rec`](Run::interpret_rec), kept for
		/// naming parity with PureScript Run's
		/// [`runRec`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// (which is literally aliased to
		/// [`interpretRec`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// in PureScript Run; both names are exposed here for
		/// cross-reference convenience).
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
		/// 			run::Run,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(99));
		/// let result: Thunk<'static, i32> = prog.run_rec::<ThunkBrand>(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Thunk<'static, Run<FirstRow, Scoped, i32>>>| op.0,
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
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			> + 'static,
			scoped_handlers: impl for<'h> DispatchScopedHandlers<
				'h,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			> + 'static,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		where
			MBrand: MonadRec + 'static, {
			self.interpret_rec::<MBrand>(handlers, scoped_handlers)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The Run instance.")]
	impl<R, A> Run<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Pipeline row-narrowing interpreter: interpret a single
		/// effect `EBrand` out of the row, returning a `Run` program in
		/// the narrowed row `RMinusE` (with `EBrand` removed).
		///
		/// Mirrors PureScript Run's
		/// [`interpret`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// (the row-narrowing form, not the all-handlers-at-once form
		/// shipped on this wrapper as [`Run::interpret`]) and heftia's
		/// `interpret_with` primary mode. Three capabilities this form
		/// uniquely enables: partial interpretation (interpret one
		/// effect, store the result, interpret the rest later);
		/// user-controlled handler ordering for non-commuting effects
		/// (e.g., `NonDet x Except`); compositional handler libraries
		/// (`fn run_state<R, A>(...) -> Run<R_minus_State, S, A>`).
		///
		/// `handler` consumes the lowered effect (`<EBrand as Kind>::Of<'_, Run<RMinusE, S, A>>`,
		/// where each inner program is already narrowed) and produces
		/// the next-step program in the narrowed row. The handler is
		/// wrapped in an [`Rc`](std::rc::Rc) once at entry; recursive
		/// calls clone the [`Rc`](std::rc::Rc) (refcount bump) instead
		/// of cloning the underlying closure, so the user-facing bound
		/// is just `Fn + 'static` (no `Clone`). This permits handlers
		/// that capture move-only resources (e.g., a `BufWriter`).
		///
		/// ## Stack safety
		///
		/// This method recurses host-stack-frame per peeled layer in
		/// the original program (the recursion is via [`Functor::map`]
		/// on each layer's continuation). For Identity-shaped effects,
		/// the recursion is eager; for closure-shaped effects (e.g.,
		/// `State<S>`), the recursion is deferred until the closure is
		/// invoked. Programs with deep chains of eager-recursing effects
		/// can blow the host stack.
		///
		/// For M-target stack safety on the dispatched effects' M-bind
		/// chains (e.g., long `State` Get/Put chains), chain
		/// [`interpret_with`](Run::interpret_with) (narrow effects one
		/// at a time) followed by [`interpret_rec`](Run::interpret_rec)
		/// at the end of the pipeline. There is no combined
		/// `interpret_with_rec` primitive: the unmatched-arm step needs
		/// to swap `RMinusE::Of<M::Of<...>>` to `M::Of<RMinusE::Of<...>>`,
		/// which requires
		/// [`Traversable`](crate::classes::Traversable) on every row
		/// brand plus
		/// [`Applicative`](crate::classes::Applicative) on `M`; non-rec
		/// `interpret_with` sidesteps this by using just
		/// [`Functor::map`](crate::classes::Functor::map) (no `M` to
		/// swap with). PureScript Run does not provide the combination
		/// either.
		///
		/// ## Type inference
		///
		/// Turbofish `EBrand` only; `Idx` is inferred via
		/// [`Member`](crate::types::effects::member::Member); `F` from
		/// the handler argument; `RMinusE` from the handler's return
		/// type.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect being interpreted out of the row.",
			"The type-level position witness (typically inferred).",
			"The narrowed row brand (the original row with `EBrand` removed; typically inferred from the handler's return type)."
		)]
		///
		#[document_parameters("The handler closure for the targeted effect.")]
		///
		#[document_returns("A `Run` program in the narrowed row `RMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// // Lift an Identity effect, then narrow it out: the handler
		/// // unwraps the `Identity` and returns the (narrowed) inner
		/// // program.
		/// let prog: Run<FullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: Run<EmptyRow, CNilBrand, i32> = prog
		/// 	.interpret_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<Run<EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		pub fn interpret_with<EBrand, Idx, RMinusE>(
			self,
			handler: impl Fn(
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<RMinusE, CNilBrand, A>>),
			) -> Run<RMinusE, CNilBrand, A>
			+ 'static,
		) -> Run<RMinusE, CNilBrand, A>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>):
				Member<
						Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>
									),
					>, {
			let handler = <RcBrand as RefCountedPointer>::new(handler);
			self.interpret_with_shared::<EBrand, Idx, RMinusE, _>(handler)
		}

		/// Inner pipeline-narrowing implementation, parameterised
		/// over the concrete handler closure type `F`. The public
		/// [`interpret_with`](Run::interpret_with) wraps the user
		/// handler in [`Rc<F>`](std::rc::Rc) once at entry and
		/// delegates here; recursive narrowing clones the
		/// [`Rc<F>`](std::rc::Rc) (refcount bump) instead of
		/// cloning the underlying closure, which is what drops the
		/// `Clone` bound from the user-facing API.
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
		#[document_returns("A `Run` program in the narrowed row `RMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// // Exercised internally by Run::interpret_with.
		/// let prog: Run<FullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: Run<EmptyRow, CNilBrand, i32> = prog
		/// 	.interpret_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<Run<EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		fn interpret_with_shared<EBrand, Idx, RMinusE, F>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'static, F>,
		) -> Run<RMinusE, CNilBrand, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<RMinusE, CNilBrand, A>>),
				) -> Run<RMinusE, CNilBrand, A>
				+ 'static,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>):
				Member<
						Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>
									),
					>, {
			match self.peel() {
				Ok(a) => Run::pure(a),
				Err(Node::First(layer)) =>
					match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>
					) as Member<Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>, Idx>>::project(
						layer
					) {
						Ok(coyo) => {
							let lowered = coyo.lower();
							let h_for_recurse = handler.clone();
							let mapped = <EBrand as Functor>::map(
								move |inner: Run<R, CNilBrand, A>| {
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
							let mapped_free = <RMinusE as Functor>::map(
								move |inner: Run<R, CNilBrand, A>| {
									inner
										.interpret_with_shared::<EBrand, Idx, RMinusE, F>(
											h_for_recurse.clone(),
										)
										.into_free()
								},
								rest,
							);
							Run::from_free(Free::<NodeBrand<RMinusE, CNilBrand>, A>::wrap(
								Node::First(mapped_free),
							))
						}
					},
				Err(Node::Scoped(cnil)) => match cnil {},
			}
		}

		/// Substrate-level row-preserving replacement primitive: walk
		/// this `Run` program, projecting each first-order dispatch
		/// against `EBrand`; replace every matched dispatch with the
		/// supplied `replacement` closure (applied to the lowered
		/// effect value), and re-emit non-matching dispatches in the
		/// same row. Direct analog of heftia's `interposeInWith` in
		/// substrate-primitive form.
		///
		/// Unlike [`interpret_with`](Run::interpret_with), `interpose`
		/// does not narrow the row: the matched arm produces a
		/// continuation in the same `R`, the unmatched arm walks the
		/// `Self::Remainder` (RMinusE) layer and embeds it back into
		/// `R` via [`CoproductEmbedder`](crate::types::effects::coproduct::CoproductEmbedder).
		/// This is the building block for scoped-effect handlers
		/// (e.g., `Catch`'s recovery path interposes against the body
		/// program's `Throw` dispatches without narrowing the row).
		///
		/// The user-facing closure is wrapped in an
		/// [`Rc`](std::rc::Rc) once at entry; recursive calls clone
		/// the [`Rc`](std::rc::Rc) (refcount bump) instead of cloning
		/// the underlying closure, which is what drops the `Clone`
		/// bound from the user-facing API.
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
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = Run<Row, CNilBrand, i32>;
		///
		/// let prog: Prog = Run::lift::<IdentityBrand, _>(Identity(7));
		/// // Replacement substitutes each matched dispatch with a fresh
		/// // program; here we return `pure(99)` for the Identity dispatch,
		/// // demonstrating that the matched arm fires.
		/// let interposed =
		/// 	prog.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| Run::pure(99));
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
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>),
			) -> Run<R, CNilBrand, A>
			+ 'static,
		) -> Run<R, CNilBrand, A>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>):
				Member<
						Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>
									),
					>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, CNilBrand>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Free<NodeBrand<R, CNilBrand>, A>,
				>),
					EmbedIndices,
				>, {
			let replacement = <RcBrand as RefCountedPointer>::new(replacement);
			self.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, _>(replacement)
		}

		/// Inner shared implementation of [`interpose`](Run::interpose),
		/// parameterised over the concrete replacement closure type
		/// `F`. The public [`interpose`](Run::interpose) wraps the
		/// user-supplied closure in [`Rc<F>`](std::rc::Rc) once at
		/// entry and delegates here; recursive descent clones the
		/// [`Rc<F>`](std::rc::Rc) (refcount bump) instead of cloning
		/// the underlying closure, which is what drops the `Clone`
		/// bound from the user-facing API.
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
		/// // Exercised internally by Run::interpose.
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = Run<Row, CNilBrand, i32>;
		///
		/// let prog: Prog = Run::lift::<IdentityBrand, _>(Identity(3));
		/// let interposed =
		/// 	prog.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| Run::pure(42));
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
			replacement: <RcBrand as RefCountedPointer>::Of<'static, F>,
		) -> Run<R, CNilBrand, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>),
				) -> Run<R, CNilBrand, A>
				+ 'static,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>):
				Member<
						Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>
									),
					>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, CNilBrand>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Free<NodeBrand<R, CNilBrand>, A>,
				>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => Run::pure(a),
				Err(Node::First(layer)) =>
					match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>
					) as Member<Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>, Idx>>::project(
						layer
					) {
						Ok(coyo) => {
							let lowered = coyo.lower();
							let r_for_recurse = replacement.clone();
							let mapped = <EBrand as Functor>::map(
								move |inner: Run<R, CNilBrand, A>| {
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
								move |inner: Run<R, CNilBrand, A>| {
									inner
										.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
											r_for_recurse.clone(),
										)
										.into_free()
								},
								rest,
							);
							let layer_back = mapped_rest.embed();
							Run::from_free(Free::<NodeBrand<R, CNilBrand>, A>::wrap(Node::First(
								layer_back,
							)))
						}
					},
				Err(Node::Scoped(cnil)) => match cnil {},
			}
		}

		/// Substrate-level matched-effect short-circuit primitive on
		/// the default Erased Run wrapper: walk this `Run` program,
		/// dispatching non-matched first-order effects through
		/// `fo_handlers` and short-circuiting the moment a
		/// matched-effect (`EBrand`) dispatch is encountered, returning
		/// the matched effect's lowered payload. Direct analog of
		/// heftia's `interpretWithEither` substrate primitive.
		///
		/// Returns `Ok(a)` when the program reduces to a pure value
		/// without firing the matched effect; returns `Err(op)` with
		/// the matched effect's lowered payload (`<EBrand as Kind>::Of<'static, Self>`)
		/// the moment the matched effect is dispatched. Pairs with
		/// scoped `Catch` handlers.
		///
		/// `fo_handlers` covers only the non-matched effects (the
		/// `RMinusE` row); the matched effect short-circuits without
		/// any handler invocation. The program type retains the full
		/// row `R` because the matched effect's dispatches throughout
		/// the program tree are discharged uniformly by the
		/// short-circuit, not by row narrowing.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the matched effect.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand (the row with `EBrand` removed at position `Idx`)."
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
		/// 			run::Run,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<
		/// 	CoyonedaBrand<ExceptBrand<String>>,
		/// 	CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>,
		/// >;
		/// type RowMinusExcept = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = Run<Row, CNilBrand, i32>;
		///
		/// let prog: Prog = Run::throw::<String, _>("oops".to_string());
		/// let result: Result<i32, Except<'_, String, Prog>> = prog
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
				Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, CNilBrand, A>>),
				Run<R, CNilBrand, A>,
			>,
		) -> Result<
			A,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>),
		>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>):
				Member<
						Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>
									),
					>, {
			let mut prog = self;
			loop {
				match prog.peel() {
					Ok(a) => return Ok(a),
					Err(Node::First(layer)) =>
						match <Apply!(
							<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>
						) as Member<Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>, Idx>>::project(
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

	#[document_type_parameters("The result type.")]
	#[document_parameters("The Run instance.")]
	impl<A> Run<CNilBrand, CNilBrand, A>
	where
		A: 'static,
	{
		/// Extracts the result value from a `Run` program whose first-order
		/// and scoped rows have both been fully interpreted away (both
		/// resolve to [`CNilBrand`]). Both rows' `Of<...>` projections are
		/// [`CNil`](crate::types::effects::coproduct::CNil) (uninhabited),
		/// so each `Node` arm is structurally impossible and matched
		/// exhaustively; the body diverges to type `!`, statically
		/// proving no runtime panic.
		///
		/// Direct analog of PureScript Run's
		/// [`extract`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// /
		/// [`runPure`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs).
		/// Pairs with [`Run::interpret_with`] for the
		/// chain-and-extract pipeline:
		/// `prog.interpret_with::<E1>(...).interpret_with::<E2>(...).extract()`.
		/// Future scoped-effect work will introduce a separate
		/// elimination operation for non-empty scoped rows, leaving
		/// `extract` as the fully-pure-program entry point.
		#[document_signature]
		///
		#[document_returns("The final result value of the fully-narrowed program.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let pure_prog: Run<CNilBrand, CNilBrand, i32> = Run::pure(42);
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
				crate::types::Free<NodeBrand<R, ScopedRow>, A>,
			>): crate::types::effects::member::Member<
					crate::types::effects::catch::BoxCatch<
						'static,
						crate::brands::BoxBrand,
						E,
						crate::types::Free<NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>, {
			let action_free = action.into_free();
			let catch: crate::types::effects::catch::BoxCatch<
				'static,
				crate::brands::BoxBrand,
				E,
				crate::types::Free<NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::catch::BoxCatch::Catch {
				action: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |_: ()| action_free,
				),
				handler: <crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(
					move |e: E| handler(e).into_free(),
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::Free<NodeBrand<R, ScopedRow>, A>,
			>) as crate::types::effects::member::Member<
				crate::types::effects::catch::BoxCatch<
					'static,
					crate::brands::BoxBrand,
					E,
					crate::types::Free<NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(catch);
			let node = Node::Scoped(layer);
			Run::from_free(crate::types::Free::wrap(node))
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
		"The resource type produced by acquire.",
		"The body's result type."
	)]
	impl<R, ScopedRow, A, B> Run<R, ScopedRow, (A, B)>
	where
		R: crate::classes::WrapDrop + crate::classes::Functor + 'static,
		ScopedRow: crate::classes::WrapDrop + crate::classes::Functor + 'static,
		A: 'static,
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
		/// result), `release` returns `Run<R, ScopedRow, ()>` (unit).
		/// Body returns the resource alongside its result so the
		/// dispatcher (step 6) can pass the resource to release; this
		/// is structurally necessary for the Box family because
		/// `Box<dyn FnOnce>` consumes the resource.
		///
		/// `Idx` is the type-level position witness identifying where
		/// `BoxBracketBrand<BoxBrand, NodeBrand<R, ScopedRow>, A, B>`
		/// lives in `ScopedRow`. Rust infers `Idx` whenever the brand
		/// appears unambiguously in the row.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
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
		/// let prog: Run<FirstRow, ScopedRow, (i32, i32)> =
		/// 	Run::<FirstRow, ScopedRow, (i32, i32)>::bracket::<_>(
		/// 		acquire,
		/// 		|resource: Box<i32>| Run::pure((*resource, 42)),
		/// 		|_resource: Box<i32>| Run::pure(()),
		/// 	);
		/// // The program is suspended at the Bracket scoped layer; peel
		/// // returns Err carrying a `Node::Scoped(...)` projection.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn bracket<Idx>(
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
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::Free<NodeBrand<R, ScopedRow>, (A, B)>,
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
				crate::types::Free<NodeBrand<R, ScopedRow>, (A, B)>,
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

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::{
			brands::{
				CNilBrand,
				CoproductBrand,
				CoyonedaBrand,
				IdentityBrand,
				NodeBrand,
			},
			types::{
				Coyoneda,
				Free,
				Identity,
				effects::{
					coproduct::Coproduct,
					node::Node,
				},
			},
		},
	};

	type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
	type Scoped = CNilBrand;
	type RunAlias<A> = Run<FirstRow, Scoped, A>;

	#[test]
	fn from_free_and_into_free_round_trip() {
		let free: Free<NodeBrand<FirstRow, Scoped>, i32> = Free::pure(42);
		let run: RunAlias<i32> = Run::from_free(free);
		let _back: Free<NodeBrand<FirstRow, Scoped>, i32> = run.into_free();
	}

	#[test]
	fn drop_a_pure_run_does_not_panic() {
		let run: RunAlias<i32> = Run::from_free(Free::pure(7));
		drop(run);
	}

	#[test]
	fn pure_then_peel_returns_value() {
		let run: RunAlias<i32> = Run::pure(42);
		assert!(matches!(run.peel(), Ok(42)));
	}

	#[test]
	fn send_produces_suspended_program() {
		let coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(Identity(7));
		let layer = Coproduct::inject(coyo);
		let run: RunAlias<i32> = Run::send(Node::First(layer));
		assert!(run.peel().is_err());
	}

	#[test]
	fn into_explicit_via_into_round_trips_pure() {
		use crate::types::effects::run_explicit::RunExplicit;
		let run: RunAlias<i32> = Run::pure(42);
		let explicit: RunExplicit<'static, FirstRow, Scoped, i32> = run.into();
		assert!(matches!(explicit.peel(), Ok(42)));
	}

	#[test]
	fn into_explicit_via_into_preserves_suspended_layer() {
		use crate::types::effects::run_explicit::RunExplicit;
		let coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(Identity(7));
		let layer = Coproduct::inject(coyo);
		let run: RunAlias<i32> = Run::send(Node::First(layer));
		let explicit: RunExplicit<'static, FirstRow, Scoped, i32> = run.into();
		assert!(explicit.peel().is_err());
	}

	#[test]
	fn bind_chains_pure_values() {
		let run: RunAlias<i32> =
			Run::pure(2).bind(|x| Run::pure(x + 1)).bind(|x| Run::pure(x * 10));
		assert!(matches!(run.peel(), Ok(30)));
	}

	#[test]
	fn map_transforms_pure_value() {
		let run: RunAlias<i32> = Run::pure(7).map(|x| x * 3);
		assert!(matches!(run.peel(), Ok(21)));
	}
}
