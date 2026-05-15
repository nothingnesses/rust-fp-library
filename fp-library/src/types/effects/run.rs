//! Erased-substrate Run program over a private representation backed by
//! [`Free`](crate::types::Free) and dual-row
//! [`NodeBrand`](crate::brands::NodeBrand) layers.
//!
//! `Run<R, S, A>` is the user-facing wrapper for the canonical
//! Run-style effect computation:
//!
//! The common representation case is a
//! `Free<NodeBrand<R, S>, A>` program. Around-action scoped handlers
//! also need an internal boundary-frame case where the selected action
//! and the outer continuation queue remain separate until the action or
//! recovery branch is known.
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

mod representation;
mod smart_constructors;

#[fp_macros::document_module]
pub(crate) mod inner {
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
				free::{
					FreeRawStep,
					TypeErasedValue,
				},
			},
		},
		core::ops::ControlFlow,
		fp_macros::*,
	};

	pub use super::representation::{
		DispatchRunRawScopedHandler,
		DispatchRunRawScopedHandlers,
		RawRunFree,
		RunContinuations,
	};
	pub(crate) use super::representation::{
		RunRepresentation,
		RunScopedBoundaryFrame,
		RunScopedContinuation,
	};

	/// Erased-substrate Run program.
	///
	/// The wrapper exists so user-facing API (`pure`, `peel`, `send`,
	/// effect-row narrowing, handler types) can be expressed without
	/// leaking the underlying representation. Ordinary programs are
	/// stored as [`Free<NodeBrand<R, S>, A>`](crate::types::Free).
	/// Around-action scoped boundaries may instead store a raw scoped
	/// layer plus the pending outer continuation queue so branch
	/// selection can happen before the continuation is reattached.
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand (typically `CNilBrand` for first-order-only programs).",
		"The result type."
	)]
	pub struct Run<R, S, A>(pub(crate) RunRepresentation<R, S, A>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static;

	/// Result-polymorphic first-order handler protocol for default `Run`.
	///
	/// Boundary-backed around-action programs can contain selected
	/// action/recovery programs whose result type is not the final outer
	/// `Run` result type. A closure monomorphic in the outer result
	/// cannot narrow those selected programs before the pending
	/// continuation queue runs. This private protocol gives the
	/// interpreter recursion a handler method that is generic in the
	/// current branch result type.
	#[document_type_parameters(
		"The first-order effect brand being interpreted out of the row.",
		"The narrowed first-order row brand.",
		"The scoped-effect row brand."
	)]
	#[document_parameters("The result-polymorphic handler instance.")]
	pub trait RunFirstOrderHandler<EBrand, RMinusE, S>
	where
		EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static, {
		/// Handles one lowered first-order operation at the current
		/// branch result type.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters(
			"The lowered first-order operation whose continuation already lives in the narrowed row."
		)]
		#[document_returns("The handler result in the narrowed row.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::{
		/// 			Run,
		/// 			RunFirstOrderHandler,
		/// 		},
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// struct IdentityHandler;
		///
		/// impl RunFirstOrderHandler<IdentityBrand, EmptyRow, CNilBrand> for IdentityHandler {
		/// 	fn handle<T: 'static>(
		/// 		&self,
		/// 		effect: Identity<Run<EmptyRow, CNilBrand, T>>,
		/// 	) -> Run<EmptyRow, CNilBrand, T> {
		/// 		effect.0
		/// 	}
		/// }
		///
		/// let prog: Run<FullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: Run<EmptyRow, CNilBrand, i32> =
		/// 	prog.handle_with_handler::<IdentityBrand, _, EmptyRow>(IdentityHandler);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		fn handle<T: 'static>(
			&self,
			effect: Apply!(
				<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<RMinusE, S, T>>
			),
		) -> Run<RMinusE, S, T>;
	}

	/// Result-polymorphic first-order replacement protocol for default
	/// `Run`.
	///
	/// Boundary-backed around-action programs can contain selected
	/// action/recovery programs whose result type is not the final outer
	/// `Run` result type. A closure monomorphic in the outer result
	/// cannot replace matched effects inside those selected programs
	/// before the pending continuation queue runs. This protocol gives
	/// `interpose` recursion a replacement method that is generic in the
	/// current branch result type.
	#[document_type_parameters(
		"The first-order effect brand being replaced.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	#[document_parameters("The result-polymorphic replacement instance.")]
	pub trait RunFirstOrderReplacer<EBrand, R, S>
	where
		EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
		R: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static, {
		/// Replaces one lowered first-order operation at the current
		/// branch result type.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters(
			"The lowered first-order operation whose continuation stays in the original row."
		)]
		#[document_returns("The replacement program in the original row.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::{
		/// 			Run,
		/// 			RunFirstOrderReplacer,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct IdentityPassThrough;
		///
		/// impl RunFirstOrderReplacer<IdentityBrand, Row, CNilBrand> for IdentityPassThrough {
		/// 	fn replace<T: 'static>(
		/// 		&self,
		/// 		effect: Identity<Run<Row, CNilBrand, T>>,
		/// 	) -> Run<Row, CNilBrand, T> {
		/// 		effect.0
		/// 	}
		/// }
		///
		/// let prog: Run<Row, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(7));
		/// let replaced =
		/// 	prog.interpose_with_replacer::<IdentityBrand, _, CNilBrand, _>(IdentityPassThrough);
		/// let result = replaced.handle_with::<IdentityBrand, _, CNilBrand>(
		/// 	|op: Identity<Run<CNilBrand, CNilBrand, i32>>| op.0,
		/// );
		/// assert_eq!(result.extract(), 7);
		/// ```
		fn replace<T: 'static>(
			&self,
			effect: Apply!(
				<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, T>>
			),
		) -> Run<R, S, T>;
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
			Run(RunRepresentation::free(free))
		}

		/// Converts a `Run<R, S, A>` to a
		/// [`Free<NodeBrand<R, S>, A>`](crate::types::Free).
		///
		/// Free-backed programs move out directly. Boundary-backed
		/// programs are lowered into an equivalent Free value whose raw
		/// scoped layer and pending continuation queue remain separate
		/// until the next step is inspected. Prefer the wrapper-level
		/// interpreters for ordinary execution; this method is the
		/// compatibility view for APIs that still consume Free.
		#[document_signature]
		///
		#[document_returns("The Free computation represented by this `Run`.")]
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
			self.0.into_free()
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

		/// Decomposes this `Run` computation into one public step. Returns
		/// `Ok(a)` if the program is a pure value, or `Err(layer)` if
		/// it is suspended in the dual-row
		/// [`Node`](crate::types::effects::node::Node) dispatch enum,
		/// where `layer` carries the next `Run` continuation.
		///
		/// Free-backed programs delegate to [`Free::resume`](crate::types::Free).
		/// Boundary-backed programs first lower through the Free
		/// compatibility view; that view keeps the boundary's pending
		/// continuation queue outside the scoped layer until a public
		/// branch is materialised, then attaches it to the selected
		/// branch through Free's single-shot one-step machinery.
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
			self.into_free()
				.resume()
				.map_err(|node| <NodeBrand<R, S> as Functor>::map(Run::from_free, node))
		}

		/// Decomposes this `Run` into one raw step, preserving boundary
		/// frames without going through the public Free view.
		#[document_signature]
		///
		#[document_returns(
			"The next raw step of this `Run`, with scoped-boundary continuations still outside the scoped layer."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn into_raw_step(self) -> FreeRawStep<NodeBrand<R, S>, A> {
			self.0.into_raw_step()
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
			Run(self.0.bind(f))
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
			Run(self.0.map(f))
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
		/// 		effects::run::{
		/// 			Run,
		/// 			RunFirstOrderHandler,
		/// 		},
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
		/// [`handle_rec`](Run::handle_rec) (and siblings) provide
		/// stack-safe interpretation via `MonadRec`.
		///
		/// ## Threading state through handlers
		///
		/// Handlers can capture an
		/// [`Rc<RefCell<S>>`](std::rc::Rc) (or
		/// [`Arc<Mutex<S>>`](std::sync::Arc) on thread-safe wrappers)
		/// to thread state through dispatch. Each `handle` call only
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
		/// let result = prog.handle(
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
		/// let result = prog.handle(
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
		pub fn handle(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, S, A>>),
				Run<R, S, A>,
			>,
			scoped_handlers: impl DispatchRunRawScopedHandlers<
				R,
				S,
				A,
				Apply!(
					<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawRunFree<R, S>>
				),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			>,
		) -> A {
			let mut prog = self;
			loop {
				match prog.into_raw_step() {
					FreeRawStep::Done(a) => return a,
					FreeRawStep::Suspended {
						layer,
						continuations,
					} => match layer {
						Node::First(layer) => {
							let remaining = std::cell::Cell::new(Some(continuations));
							let mapped = <R as Functor>::map(
								move |inner: RawRunFree<R, S>| {
									#[expect(
										clippy::expect_used,
										reason = "Raw first-order dispatch attaches a single-shot continuation to exactly one active row branch"
									)]
									let continuations = remaining.take().expect(
										"Run raw first-order continuation attached more than once",
									);
									Run::from_free(Free::continue_from_erased(inner, continuations))
								},
								layer,
							);
							prog = handlers.dispatch(mapped);
						}
						Node::Scoped(layer) => {
							prog = scoped_handlers.dispatch_run_raw_scoped(
								layer,
								continuations,
								&handlers,
							);
						}
					},
				}
			}
		}

		/// Alias for [`handle`](Run::handle), kept for naming
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
			scoped_handlers: impl DispatchRunRawScopedHandlers<
				R,
				S,
				A,
				Apply!(
					<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawRunFree<R, S>>
				),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			>,
		) -> A {
			self.handle(handlers, scoped_handlers)
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
		/// let result: Thunk<'static, i32> = prog.handle_rec::<ThunkBrand>(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Thunk<'static, Run<FirstRow, Scoped, i32>>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result.evaluate(), 42);
		/// ```
		#[inline]
		pub fn handle_rec<MBrand>(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			> + 'static,
			scoped_handlers: impl DispatchScopedHandlers<
				'static,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
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

		/// Alias for [`handle_rec`](Run::handle_rec), kept for
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
			scoped_handlers: impl DispatchScopedHandlers<
				'static,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			> + 'static,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		where
			MBrand: MonadRec + 'static, {
			self.handle_rec::<MBrand>(handlers, scoped_handlers)
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
		/// Scoped-row-narrowing interpreter: interpret a single scoped
		/// effect `SBrand` out of the scoped row, returning a `Run`
		/// program in the narrowed scoped row `SMinusE`.
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
		#[document_returns("A `Run` program in the narrowed scoped row `SMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		span::BoxSpan,
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
		///
		/// let action: Run<CNilBrand, ScopedRow, i32> = Run::pure(7);
		/// let prog: Run<CNilBrand, ScopedRow, i32> = Run::span::<&'static str, _>("request", action);
		/// let narrowed: Run<CNilBrand, CNilBrand, i32> = prog
		/// 	.handle_scoped_with::<BoxSpanBrand<BoxBrand, &'static str>, _, CNilBrand>(
		/// 		|span| match span {
		/// 			BoxSpan::Span {
		/// 				tag,
		/// 				action,
		/// 			} => {
		/// 				assert_eq!(tag, "request");
		/// 				action(())
		/// 			}
		/// 		},
		/// 	);
		/// assert_eq!(narrowed.extract(), 7);
		/// ```
		#[inline]
		pub fn handle_scoped_with<SBrand, Idx, SMinusE>(
			self,
			handler: impl Fn(
				Apply!(<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, SMinusE, A>>),
			) -> Run<R, SMinusE, A>
			+ 'static,
		) -> Run<R, SMinusE, A>
		where
			SBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
					),
					Idx,
					Remainder = Apply!(
									<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>, {
			let handler = <RcBrand as RefCountedPointer>::new(handler);
			self.handle_scoped_with_shared::<SBrand, Idx, SMinusE, _>(handler)
		}

		#[inline]
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
		#[document_returns("A `Run` program in the narrowed scoped row `SMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		span::BoxSpan,
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
		///
		/// // Exercised internally by Run::handle_scoped_with.
		/// let action: Run<CNilBrand, ScopedRow, i32> = Run::pure(7);
		/// let prog: Run<CNilBrand, ScopedRow, i32> = Run::span::<&'static str, _>("request", action);
		/// let narrowed: Run<CNilBrand, CNilBrand, i32> = prog
		/// 	.handle_scoped_with::<BoxSpanBrand<BoxBrand, &'static str>, _, CNilBrand>(
		/// 		|span| match span {
		/// 			BoxSpan::Span {
		/// 				tag,
		/// 				action,
		/// 			} => {
		/// 				assert_eq!(tag, "request");
		/// 				action(())
		/// 			}
		/// 		},
		/// 	);
		/// assert_eq!(narrowed.extract(), 7);
		/// ```
		fn handle_scoped_with_shared<SBrand, Idx, SMinusE, F>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'static, F>,
		) -> Run<R, SMinusE, A>
		where
			F: Fn(
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							Run<R, SMinusE, A>,
						>
					),
				) -> Run<R, SMinusE, A>
				+ 'static,
			SBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
					),
					Idx,
					Remainder = Apply!(
									<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>, {
			match self.peel() {
				Ok(a) => Run::pure(a),
				Err(Node::First(layer)) => {
					let h_for_recurse = handler.clone();
					let mapped_free = <R as Functor>::map(
						move |inner: Run<R, S, A>| {
							inner
								.handle_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
									h_for_recurse.clone(),
								)
								.into_free()
						},
						layer,
					);
					Run::from_free(Free::<NodeBrand<R, SMinusE>, A>::wrap(Node::First(mapped_free)))
				}
				Err(Node::Scoped(layer)) =>
					match <Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Run<R, S, A>,
					>) as Member<
						Apply!(
							<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
						),
						Idx,
					>>::project(layer)
					{
						Ok(scoped) => {
							let h_for_recurse = handler.clone();
							let mapped = <SBrand as Functor>::map(
								move |inner: Run<R, S, A>| {
									inner.handle_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
										h_for_recurse.clone(),
									)
								},
								scoped,
							);
							(*handler)(mapped)
						}
						Err(rest) => {
							let h_for_recurse = handler.clone();
							let mapped_free = <SMinusE as Functor>::map(
								move |inner: Run<R, S, A>| {
									inner
										.handle_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
											h_for_recurse.clone(),
										)
										.into_free()
								},
								rest,
							);
							Run::from_free(Free::<NodeBrand<R, SMinusE>, A>::wrap(Node::Scoped(
								mapped_free,
							)))
						}
					},
			}
		}

		/// Pipeline row-narrowing interpreter: interpret a single
		/// effect `EBrand` out of the row, returning a `Run` program in
		/// the narrowed row `RMinusE` (with `EBrand` removed).
		///
		/// Mirrors PureScript Run's
		/// [`interpret`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// (the row-narrowing form, not the all-handlers-at-once form
		/// shipped on this wrapper as [`Run::handle`]) and heftia's
		/// `handle_with` primary mode. Three capabilities this form
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
		/// ## Scoped rows
		///
		/// Scoped layers are rewritten through the scoped row's
		/// [`Functor`](crate::classes::Functor) implementation. This
		/// preserves scoped cells whose functor maps their stored action
		/// program, such as Span, Catch, Local, and RefLocal. It is not
		/// the continuation-aware raw scoped handler path used to run
		/// scoped handlers. Scoped cells whose functor intentionally
		/// leaves the cell unchanged, such as
		/// [`BoxBracketBrand`](crate::brands::BoxBracketBrand), are not
		/// traversed by this API; their acquire/body/release programs
		/// are run by the Bracket dispatcher instead.
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
		/// [`handle_with`](Run::handle_with) (narrow effects one
		/// at a time) followed by [`handle_rec`](Run::handle_rec)
		/// at the end of the pipeline. There is no combined
		/// `handle_with_rec` primitive: the unmatched-arm step needs
		/// to swap `RMinusE::Of<M::Of<...>>` to `M::Of<RMinusE::Of<...>>`,
		/// which requires
		/// [`Traversable`](crate::classes::Traversable) on every row
		/// brand plus
		/// [`Applicative`](crate::classes::Applicative) on `M`; non-rec
		/// `handle_with` sidesteps this by using just
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
		/// 		effects::run::{
		/// 			Run,
		/// 			RunFirstOrderHandler,
		/// 		},
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// struct IdentityHandler;
		///
		/// impl RunFirstOrderHandler<IdentityBrand, EmptyRow, CNilBrand> for IdentityHandler {
		/// 	fn handle<T: 'static>(
		/// 		&self,
		/// 		effect: Identity<Run<EmptyRow, CNilBrand, T>>,
		/// 	) -> Run<EmptyRow, CNilBrand, T> {
		/// 		effect.0
		/// 	}
		/// }
		///
		/// // Lift an Identity effect, then narrow it out with a
		/// // result-polymorphic handler.
		/// let prog: Run<FullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: Run<EmptyRow, CNilBrand, i32> =
		/// 	prog.handle_with_handler::<IdentityBrand, _, EmptyRow>(IdentityHandler);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		pub fn handle_with_handler<EBrand, Idx, RMinusE>(
			self,
			handler: impl RunFirstOrderHandler<EBrand, RMinusE, S> + 'static,
		) -> Run<RMinusE, S, A>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>,
			Apply!(
				<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, TypeErasedValue>>
			): Member<
					Coyoneda<'static, EBrand, Run<R, S, TypeErasedValue>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, S, TypeErasedValue>,
									>
								),
				>, {
			let handler = <RcBrand as RefCountedPointer>::new(handler);
			self.handle_with_handler_shared::<EBrand, Idx, RMinusE, _>(handler)
		}

		/// Row-narrowing implementation backed by a
		/// result-polymorphic first-order handler.
		///
		/// The handler is called through [`RunFirstOrderHandler`] so each
		/// recursive branch can use its own result type.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect being interpreted out of the row.",
			"The type-level position witness.",
			"The narrowed row brand.",
			"The concrete result-polymorphic handler type."
		)]
		#[document_parameters("The handler wrapped in a refcounted pointer.")]
		#[document_returns("A `Run` program in the narrowed row `RMinusE`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::{
		/// 			Run,
		/// 			RunFirstOrderHandler,
		/// 		},
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// struct IdentityHandler;
		///
		/// impl RunFirstOrderHandler<IdentityBrand, EmptyRow, CNilBrand> for IdentityHandler {
		/// 	fn handle<T: 'static>(
		/// 		&self,
		/// 		effect: Identity<Run<EmptyRow, CNilBrand, T>>,
		/// 	) -> Run<EmptyRow, CNilBrand, T> {
		/// 		effect.0
		/// 	}
		/// }
		///
		/// let prog: Run<FullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: Run<EmptyRow, CNilBrand, i32> =
		/// 	prog.handle_with_handler::<IdentityBrand, _, EmptyRow>(IdentityHandler);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		pub(crate) fn handle_with_handler_shared<EBrand, Idx, RMinusE, H>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'static, H>,
		) -> Run<RMinusE, S, A>
		where
			H: RunFirstOrderHandler<EBrand, RMinusE, S> + 'static,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>,
			Apply!(
				<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, TypeErasedValue>>
			): Member<
					Coyoneda<'static, EBrand, Run<R, S, TypeErasedValue>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, S, TypeErasedValue>,
									>
								),
				>, {
			match self.0 {
				RunRepresentation::Free(free) => Run::from_free(free)
					.interpret_free_with_handler_shared::<EBrand, Idx, RMinusE, H>(handler),
				RunRepresentation::ScopedBoundary(boundary) =>
					Run(RunRepresentation::ScopedBoundary(
						boundary.handle_with_handler::<EBrand, Idx, RMinusE, H>(handler),
					)),
			}
		}

		/// Free-backed row-narrowing implementation for ordinary
		/// default `Run` programs.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect being interpreted out of the row.",
			"The type-level position witness.",
			"The narrowed row brand.",
			"The concrete result-polymorphic handler type."
		)]
		#[document_parameters("The handler wrapped in a refcounted pointer.")]
		#[document_returns("A `Run` program in the narrowed row `RMinusE`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::{
		/// 			Run,
		/// 			RunFirstOrderHandler,
		/// 		},
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// struct IdentityHandler;
		///
		/// impl RunFirstOrderHandler<IdentityBrand, EmptyRow, CNilBrand> for IdentityHandler {
		/// 	fn handle<T: 'static>(
		/// 		&self,
		/// 		effect: Identity<Run<EmptyRow, CNilBrand, T>>,
		/// 	) -> Run<EmptyRow, CNilBrand, T> {
		/// 		effect.0
		/// 	}
		/// }
		///
		/// let prog: Run<FullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: Run<EmptyRow, CNilBrand, i32> =
		/// 	prog.handle_with_handler::<IdentityBrand, _, EmptyRow>(IdentityHandler);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		fn interpret_free_with_handler_shared<EBrand, Idx, RMinusE, H>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'static, H>,
		) -> Run<RMinusE, S, A>
		where
			H: RunFirstOrderHandler<EBrand, RMinusE, S> + 'static,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>,
			Apply!(
				<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, TypeErasedValue>>
			): Member<
					Coyoneda<'static, EBrand, Run<R, S, TypeErasedValue>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, S, TypeErasedValue>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => Run::pure(a),
				Err(Node::First(layer)) =>
					match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
					) as Member<Coyoneda<'static, EBrand, Run<R, S, A>>, Idx>>::project(
						layer
					) {
						Ok(coyo) => {
							let lowered = coyo.lower();
							let h_for_recurse = handler.clone();
							let mapped = <EBrand as Functor>::map(
								move |inner: Run<R, S, A>| {
									inner.handle_with_handler_shared::<EBrand, Idx, RMinusE, H>(
										h_for_recurse.clone(),
									)
								},
								lowered,
							);
							(*handler).handle(mapped)
						}
						Err(rest) => {
							let h_for_recurse = handler.clone();
							let mapped_free = <RMinusE as Functor>::map(
								move |inner: Run<R, S, A>| {
									inner
										.handle_with_handler_shared::<EBrand, Idx, RMinusE, H>(
											h_for_recurse.clone(),
										)
										.into_free()
								},
								rest,
							);
							Run::from_free(Free::<NodeBrand<RMinusE, S>, A>::wrap(Node::First(
								mapped_free,
							)))
						}
					},
				Err(Node::Scoped(layer)) => {
					let h_for_recurse = handler.clone();
					let mapped_free = <S as Functor>::map(
						move |inner: Run<R, S, A>| {
							inner
								.handle_with_handler_shared::<EBrand, Idx, RMinusE, H>(
									h_for_recurse.clone(),
								)
								.into_free()
						},
						layer,
					);
					Run::from_free(Free::<NodeBrand<RMinusE, S>, A>::wrap(Node::Scoped(
						mapped_free,
					)))
				}
			}
		}

		/// Inner pipeline-narrowing implementation, parameterised
		/// over the concrete handler closure type `F`. The public
		/// [`handle_with`](Run::handle_with) wraps the user
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
		/// // Exercised internally by Run::handle_with.
		/// let prog: Run<FullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: Run<EmptyRow, CNilBrand, i32> = prog.handle_with::<IdentityBrand, _, EmptyRow>(
		/// 	|op: Identity<Run<EmptyRow, CNilBrand, i32>>| op.0,
		/// );
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		fn handle_with_shared<EBrand, Idx, RMinusE, F>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'static, F>,
		) -> Run<RMinusE, S, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<RMinusE, S, A>>),
				) -> Run<RMinusE, S, A>
				+ 'static,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>, {
			match self.peel() {
				Ok(a) => Run::pure(a),
				Err(Node::First(layer)) =>
					match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
					) as Member<Coyoneda<'static, EBrand, Run<R, S, A>>, Idx>>::project(
						layer
					) {
						Ok(coyo) => {
							let lowered = coyo.lower();
							let h_for_recurse = handler.clone();
							let mapped = <EBrand as Functor>::map(
								move |inner: Run<R, S, A>| {
									inner.handle_with_shared::<EBrand, Idx, RMinusE, F>(
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
								move |inner: Run<R, S, A>| {
									inner
										.handle_with_shared::<EBrand, Idx, RMinusE, F>(
											h_for_recurse.clone(),
										)
										.into_free()
								},
								rest,
							);
							Run::from_free(Free::<NodeBrand<RMinusE, S>, A>::wrap(Node::First(
								mapped_free,
							)))
						}
					},
				Err(Node::Scoped(layer)) => {
					let h_for_recurse = handler.clone();
					let mapped_free = <S as Functor>::map(
						move |inner: Run<R, S, A>| {
							inner
								.handle_with_shared::<EBrand, Idx, RMinusE, F>(
									h_for_recurse.clone(),
								)
								.into_free()
						},
						layer,
					);
					Run::from_free(Free::<NodeBrand<RMinusE, S>, A>::wrap(Node::Scoped(
						mapped_free,
					)))
				}
			}
		}

		/// Substrate-level row-preserving replacement primitive using
		/// a result-polymorphic replacement protocol.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect to replace.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand used while projecting the matched effect.",
			"The HList witness for embedding the narrowed row back into the original row."
		)]
		#[document_parameters("The result-polymorphic replacement value.")]
		#[document_returns(
			"A new program in the same row with all matched-effect dispatches replaced."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::{
		/// 			Run,
		/// 			RunFirstOrderReplacer,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = Run<Row, CNilBrand, i32>;
		///
		/// struct IdentityPassThrough;
		///
		/// impl RunFirstOrderReplacer<IdentityBrand, Row, CNilBrand> for IdentityPassThrough {
		/// 	fn replace<T: 'static>(
		/// 		&self,
		/// 		op: Identity<Run<Row, CNilBrand, T>>,
		/// 	) -> Run<Row, CNilBrand, T> {
		/// 		op.0
		/// 	}
		/// }
		///
		/// let prog: Prog = Run::lift::<IdentityBrand, _>(Identity(7));
		/// let interposed =
		/// 	prog.interpose_with_replacer::<IdentityBrand, _, CNilBrand, _>(IdentityPassThrough);
		/// let result = interposed.handle_with::<IdentityBrand, _, CNilBrand>(
		/// 	|op: Identity<Run<CNilBrand, CNilBrand, i32>>| op.0,
		/// );
		/// assert_eq!(result.extract(), 7);
		/// ```
		pub fn interpose_with_replacer<EBrand, Idx, RMinusE, EmbedIndices>(
			self,
			replacement: impl RunFirstOrderReplacer<EBrand, R, S> + 'static,
		) -> Run<R, S, A>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Free<NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>,
			Apply!(
				<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, TypeErasedValue>>
			): Member<
					Coyoneda<'static, EBrand, Run<R, S, TypeErasedValue>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, S, TypeErasedValue>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, S>, TypeErasedValue>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Free<NodeBrand<R, S>, TypeErasedValue>,
					>),
					EmbedIndices,
				>, {
			let replacement = <RcBrand as RefCountedPointer>::new(replacement);
			self.interpose_with_replacer_shared::<EBrand, Idx, RMinusE, EmbedIndices, _>(
				replacement,
			)
		}

		/// Shared implementation of
		/// [`interpose_with_replacer`](Run::interpose_with_replacer).
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect to replace.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand used while projecting the matched effect.",
			"The HList witness for embedding the narrowed row back into the original row.",
			"The concrete result-polymorphic replacement type."
		)]
		#[document_parameters("The Rc-wrapped replacement value.")]
		#[document_returns(
			"A new program in the same row with all matched-effect dispatches replaced."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::{
		/// 			Run,
		/// 			RunFirstOrderReplacer,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct IdentityPassThrough;
		///
		/// impl RunFirstOrderReplacer<IdentityBrand, Row, CNilBrand> for IdentityPassThrough {
		/// 	fn replace<T: 'static>(
		/// 		&self,
		/// 		op: Identity<Run<Row, CNilBrand, T>>,
		/// 	) -> Run<Row, CNilBrand, T> {
		/// 		op.0
		/// 	}
		/// }
		///
		/// let prog: Run<Row, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let interposed =
		/// 	prog.interpose_with_replacer::<IdentityBrand, _, CNilBrand, _>(IdentityPassThrough);
		/// let result = interposed.handle_with::<IdentityBrand, _, CNilBrand>(
		/// 	|op: Identity<Run<CNilBrand, CNilBrand, i32>>| op.0,
		/// );
		/// assert_eq!(result.extract(), 42);
		/// ```
		#[inline]
		pub(crate) fn interpose_with_replacer_shared<EBrand, Idx, RMinusE, EmbedIndices, P>(
			self,
			replacement: <RcBrand as RefCountedPointer>::Of<'static, P>,
		) -> Run<R, S, A>
		where
			P: RunFirstOrderReplacer<EBrand, R, S> + 'static,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Free<NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>,
			Apply!(
				<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, TypeErasedValue>>
			): Member<
					Coyoneda<'static, EBrand, Run<R, S, TypeErasedValue>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, S, TypeErasedValue>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, S>, TypeErasedValue>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Free<NodeBrand<R, S>, TypeErasedValue>,
					>),
					EmbedIndices,
				>, {
			match self.0 {
				RunRepresentation::Free(free) => Run::from_free(free)
					.interpose_free_with_replacer_shared::<EBrand, Idx, RMinusE, EmbedIndices, P>(
						replacement,
					),
				RunRepresentation::ScopedBoundary(boundary) =>
					Run(RunRepresentation::ScopedBoundary(
						boundary.interpose_with_replacer::<EBrand, Idx, RMinusE, EmbedIndices, P>(
							replacement,
						),
					)),
			}
		}

		/// Free-backed implementation of
		/// [`interpose_with_replacer`](Run::interpose_with_replacer).
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect to replace.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand used while projecting the matched effect.",
			"The HList witness for embedding the narrowed row back into the original row.",
			"The concrete result-polymorphic replacement type."
		)]
		#[document_parameters("The Rc-wrapped replacement value.")]
		#[document_returns(
			"A Free-backed program in the same row with all matched-effect dispatches replaced."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::run::{
		/// 			Run,
		/// 			RunFirstOrderReplacer,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct IdentityPassThrough;
		///
		/// impl RunFirstOrderReplacer<IdentityBrand, Row, CNilBrand> for IdentityPassThrough {
		/// 	fn replace<T: 'static>(
		/// 		&self,
		/// 		op: Identity<Run<Row, CNilBrand, T>>,
		/// 	) -> Run<Row, CNilBrand, T> {
		/// 		op.0
		/// 	}
		/// }
		///
		/// let prog: Run<Row, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(21));
		/// let interposed =
		/// 	prog.interpose_with_replacer::<IdentityBrand, _, CNilBrand, _>(IdentityPassThrough);
		/// let result = interposed.handle_with::<IdentityBrand, _, CNilBrand>(
		/// 	|op: Identity<Run<CNilBrand, CNilBrand, i32>>| op.0,
		/// );
		/// assert_eq!(result.extract(), 21);
		/// ```
		#[inline]
		fn interpose_free_with_replacer_shared<EBrand, Idx, RMinusE, EmbedIndices, P>(
			self,
			replacement: <RcBrand as RefCountedPointer>::Of<'static, P>,
		) -> Run<R, S, A>
		where
			P: RunFirstOrderReplacer<EBrand, R, S> + 'static,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>): Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Free<NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>,
			Apply!(
				<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, TypeErasedValue>>
			): Member<
					Coyoneda<'static, EBrand, Run<R, S, TypeErasedValue>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, S, TypeErasedValue>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, S>, TypeErasedValue>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Free<NodeBrand<R, S>, TypeErasedValue>,
					>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => Run::pure(a),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>
				) as Member<
					Coyoneda<'static, EBrand, Run<R, S, A>>,
					Idx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let lowered = coyo.lower();
						let r_for_recurse = replacement.clone();
						let mapped = <EBrand as Functor>::map(
							move |inner: Run<R, S, A>| {
								inner
									.interpose_with_replacer_shared::<EBrand, Idx, RMinusE, EmbedIndices, P>(
										r_for_recurse.clone(),
									)
							},
							lowered,
						);
						(*replacement).replace(mapped)
					}
					Err(rest) => {
						let r_for_recurse = replacement.clone();
						let mapped_rest = <RMinusE as Functor>::map(
							move |inner: Run<R, S, A>| {
								inner
									.interpose_with_replacer_shared::<EBrand, Idx, RMinusE, EmbedIndices, P>(
										r_for_recurse.clone(),
									)
									.into_free()
							},
							rest,
						);
						let layer_back = mapped_rest.embed();
						Run::from_free(Free::<NodeBrand<R, S>, A>::wrap(Node::First(layer_back)))
					}
				},
				Err(Node::Scoped(layer)) => {
					let r_for_recurse = replacement.clone();
					let mapped_free = <S as Functor>::map(
						move |inner: Run<R, S, A>| {
							inner
								.interpose_with_replacer_shared::<EBrand, Idx, RMinusE, EmbedIndices, P>(
									r_for_recurse.clone(),
								)
								.into_free()
						},
						layer,
					);
					Run::from_free(Free::<NodeBrand<R, S>, A>::wrap(Node::Scoped(mapped_free)))
				}
			}
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The Run instance.")]
	impl<R, A> Run<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Interprets one first-order effect out of a first-order-only
		/// `Run` program with a final-result-specific closure.
		///
		/// This convenience method is available only when the scoped row
		/// is [`CNilBrand`]. Programs with scoped rows should use
		/// [`Run::handle_with_handler`], whose handler protocol is
		/// result-polymorphic and can safely narrow selected scoped
		/// action/recovery programs whose branch result differs from the
		/// final outer result.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect being interpreted out of the row.",
			"The type-level position witness.",
			"The narrowed row brand."
		)]
		#[document_parameters("The final-result-specific handler closure.")]
		#[document_returns("A first-order-only `Run` program in the narrowed row `RMinusE`.")]
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
		/// let prog: Run<FullRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: Run<EmptyRow, CNilBrand, i32> = prog.handle_with::<IdentityBrand, _, EmptyRow>(
		/// 	|op: Identity<Run<EmptyRow, CNilBrand, i32>>| op.0,
		/// );
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		pub fn handle_with<EBrand, Idx, RMinusE>(
			self,
			handler: impl Fn(
				Apply!(
					<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Run<RMinusE, CNilBrand, A>,
					>
				),
			) -> Run<RMinusE, CNilBrand, A>
			+ 'static,
		) -> Run<RMinusE, CNilBrand, A>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
					Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, A>,
									>
								),
				>, {
			let handler = <RcBrand as RefCountedPointer>::new(handler);
			self.handle_with_shared::<EBrand, Idx, RMinusE, _>(handler)
		}

		/// Replaces one first-order effect in a first-order-only `Run`
		/// program with a final-result-specific closure.
		///
		/// This convenience method is available only when the scoped row
		/// is [`CNilBrand`]. Programs with scoped rows should use
		/// [`Run::interpose_with_replacer`], whose replacement protocol is
		/// result-polymorphic and can safely rewrite selected scoped
		/// action/recovery programs whose branch result differs from the
		/// final outer result.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect to replace.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand used while projecting the matched effect.",
			"The HList witness for embedding the narrowed row back into the original row."
		)]
		#[document_parameters("The final-result-specific replacement closure.")]
		#[document_returns("A first-order-only `Run` program in the original row.")]
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
		/// let interposed =
		/// 	prog.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| Run::pure(99));
		/// let result = interposed.handle(
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
				Apply!(
					<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Run<R, CNilBrand, A>,
					>
				),
			) -> Run<R, CNilBrand, A>
			+ 'static,
		) -> Run<R, CNilBrand, A>
		where
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
					Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, A>,
									>
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

		/// Shared implementation of the first-order-only closure
		/// [`interpose`](Run::interpose) convenience.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect to replace.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand used while projecting the matched effect.",
			"The HList witness for embedding the narrowed row back into the original row.",
			"The concrete replacement closure type."
		)]
		#[document_parameters("The Rc-wrapped final-result-specific replacement closure.")]
		#[document_returns("A first-order-only `Run` program in the original row.")]
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
		/// let interposed =
		/// 	prog.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| Run::pure(99));
		/// let result = interposed.handle(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Prog>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 99);
		/// ```
		#[inline]
		fn interpose_shared<EBrand, Idx, RMinusE, EmbedIndices, F>(
			self,
			replacement: <RcBrand as RefCountedPointer>::Of<'static, F>,
		) -> Run<R, CNilBrand, A>
		where
			F: Fn(
					Apply!(
						<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							Run<R, CNilBrand, A>,
						>
					),
				) -> Run<R, CNilBrand, A>
				+ 'static,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
					Coyoneda<'static, EBrand, Run<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, A>,
									>
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
		/// 	.handle_with_either::<ExceptBrand<String>, _, RowMinusExcept>(handlers! {
		/// 		IdentityBrand: |op: Identity<Prog>| op.0,
		/// 	});
		/// match result {
		/// 	Ok(_) => panic!("expected throw"),
		/// 	Err(Except::Throw(e, _)) => assert_eq!(e, "oops"),
		/// }
		/// ```
		#[inline]
		pub fn handle_with_either<EBrand, Idx, RMinusE>(
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
		/// Pairs with [`Run::handle_with`] for the
		/// chain-and-extract pipeline:
		/// `prog.handle_with::<E1>(...).handle_with::<E2>(...).extract()`.
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
}

pub use inner::*;

#[cfg(test)]
mod tests;
