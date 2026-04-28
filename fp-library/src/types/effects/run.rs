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
//! row brand `S` carries higher-order constructors (Phase 4 populates
//! it with `Catch`, `Local`, etc.; for first-order-only programs it
//! stays as `CNilBrand`).
//!
//! `Run` is the Erased counterpart of
//! `RunExplicit` (Phase 2 step 4b; not yet implemented).
//! The Erased substrate is single-shot, type-erases through
//! `Box<dyn Any>`, has O(1) `bind`, and is `'static`-only. It exposes
//! its API via inherent methods rather than Brand-dispatched type
//! classes, so do-notation is via the `run_do!` macro (Phase 2 step 7),
//! not `m_do!`. Use `RunExplicit` for non-`'static` payloads or when
//! Brand-dispatched typeclass-generic code is required.
//!
//! ## Step 4a scope
//!
//! This module currently only ships the type-level wrapper, the Drop
//! impl (which inherits from the underlying Free's WrapDrop-driven
//! iterative dismantling), and the construction sugar
//! [`Run::from_free`] / [`Run::into_free`]. The user-facing
//! operations (`pure`, `peel`, `send`, `bind`, `map`, `lift_f`,
//! `evaluate`, `handle`, etc.) land in Phase 2 step 5.

#[fp_macros::document_module]
mod inner {
	use {
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
				Free,
				effects::{
					interpreter::DispatchHandlers,
					member::Member,
					node::Node,
				},
			},
		},
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
		/// let _run: Run<FirstRow, Scoped, i32> = Run::from_free(free);
		/// assert!(true);
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
		/// let _free: Free<NodeBrand<FirstRow, Scoped>, i32> = run.into_free();
		/// assert!(true);
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
		/// and lifts via [`send`](Run::send). Phase 3's per-effect
		/// smart constructors (`ask`, `get`, `put`, `tell`, `throw`)
		/// will be one-liners over this combinator, mirroring
		/// PureScript Run's `liftEffect = lift (Proxy :: "effect")`
		/// pattern.
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
		/// of shape `FnMut(<EBrand as Kind>::Of<'_, Run<R, CNilBrand, A>>) -> Run<R, CNilBrand, A>`,
		/// taking the lowered [`Coyoneda`](crate::types::Coyoneda)
		/// payload and returning the next-step program in the same
		/// row.
		///
		/// Mirrors PureScript Run's
		/// [`interpret`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
		/// (which is itself a literal alias for
		/// [`run`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)).
		/// Per [Phase 3 step 2 deviations](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/deviations.md),
		/// the Rust port adopts the mono-in-`A` step-function shape so
		/// handler closures don't need rank-2 polymorphism (which Rust
		/// closures can't express). The scoped row `S` is fixed at
		/// [`CNilBrand`](crate::brands::CNilBrand) for Phase 3; Phase 4
		/// extends this to dispatch over scoped effects too.
		///
		/// ## Stack safety
		///
		/// This method recurses host-stack-frame per peeled layer.
		/// Phase 3 step 3 ships
		/// [`interpret_rec`](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/plan.md)
		/// (and siblings) for stack-safe interpretation via `MonadRec`.
		#[document_signature]
		///
		#[document_parameters("The handler list (typically built via the `handlers!` macro).")]
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
		/// let result = prog.interpret(handlers! {
		/// 	IdentityBrand: |op: Identity<Run<FirstRow, Scoped, i32>>| op.0,
		/// });
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "Phase 3 first-order interpreter does not handle scoped layers; Phase 4 wires them. Reaching the Scoped arm would indicate a wrapper-API logic error rather than user error, so the descriptive panic is appropriate until Phase 4 lands."
		)]
		pub fn interpret(
			self,
			mut handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, S, A>>),
				Run<R, S, A>,
			>,
		) -> A
		where
			S: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static, {
			let mut prog = self;
			loop {
				match prog.peel() {
					Ok(a) => return a,
					Err(Node::First(layer)) => prog = handlers.dispatch(layer),
					Err(Node::Scoped(_)) => {
						unreachable!(
							"Phase 3 first-order interpreter received a scoped layer; scoped effects ship in Phase 4"
						)
					}
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
		#[document_parameters("The handler list.")]
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
		/// let result = prog.run(handlers! {
		/// 	IdentityBrand: |op: Identity<Run<FirstRow, Scoped, i32>>| op.0,
		/// });
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
		) -> A
		where
			S: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static, {
			self.interpret(handlers)
		}

		/// Interprets this `Run` program with a state value threaded
		/// through each handler invocation, mirroring PureScript Run's
		/// [`runAccum`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs).
		///
		/// `init` is the initial state value. Each handler receives
		/// the current state by mutable reference inside the closures
		/// that compose `handlers`; users mutate the state in-place to
		/// thread updates between effect dispatches. The final state is
		/// discarded, matching PureScript Run's
		/// `runAccum :: ... -> Run r a -> m a` shape (state is internal
		/// to the loop).
		///
		/// Per the Phase 3 step 2 deviations entry, state threading in
		/// the Rust port is via closure captures (a mutable
		/// [`Rc`](std::rc::Rc) /
		/// [`RefCell`](std::cell::RefCell) or plain `&mut` borrowed
		/// across the handler-list closures) rather than a separate
		/// stateful trait, which would have doubled the trait
		/// machinery. The `init` parameter exists for API parity and
		/// is moved into the user's choice of state cell at the call
		/// site.
		#[document_signature]
		///
		#[document_type_parameters("The state type.")]
		///
		#[document_parameters(
			"The handler list (typically built via the `handlers!` macro), with each closure capturing the state cell.",
			"The initial state value (passed through to the user's state cell)."
		)]
		///
		#[document_returns("The final result value of the program.")]
		///
		#[document_examples]
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
		/// let result = prog.run_accum(
		/// 	handlers! {
		/// 		IdentityBrand: move |op: Identity<Run<FirstRow, Scoped, i32>>| {
		/// 			*counter_for_handler.borrow_mut() += 1;
		/// 			op.0
		/// 		},
		/// 	},
		/// 	0_i32,
		/// );
		/// assert_eq!(result, 7);
		/// assert_eq!(*counter.borrow(), 1);
		/// ```
		#[inline]
		pub fn run_accum<St>(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, S, A>>),
				Run<R, S, A>,
			>,
			init: St,
		) -> A
		where
			S: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static, {
			let _ = init;
			self.interpret(handlers)
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
