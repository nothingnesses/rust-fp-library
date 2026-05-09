//! Erased-substrate Run program with `Rc`-shared continuations
//! supporting multi-shot effects.
//!
//! `RcRun<R, S, A>` is the multi-shot, [`Clone`]-cheap sibling of
//! [`Run`](crate::types::effects::run::Run): the same conceptual identity
//!
//! ```text
//! RcRun<R, S, A> = RcFree<NodeBrand<R, S>, A>
//! ```
//!
//! but the underlying [`RcFree`](crate::types::RcFree) carries
//! `Rc<dyn Fn>` continuations rather than `Box<dyn FnOnce>`, so
//! handlers for non-deterministic effects (`Choose`, `Amb`) can drive
//! the same suspended program more than once. The whole substrate
//! lives behind an outer [`Rc`](std::rc::Rc), so cloning a program
//! is O(1).
//!
//! Use [`Run`](crate::types::effects::run::Run) when continuations are
//! single-shot (the common case). Use `RcRun` for multi-shot effects.
//! Use [`ArcRun`](crate::types::effects::arc_run::ArcRun) when programs cross
//! thread boundaries.
//!
//! The construction sugar
//! [`from_rc_free`](RcRun::from_rc_free) /
//! [`into_rc_free`](RcRun::into_rc_free) bridges to the underlying
//! [`RcFree`](crate::types::RcFree). User-facing operations
//! (`pure`, `peel`, `send`, `bind`, `map`, `lift_f`, `evaluate`,
//! `handle`, etc.) are exposed as inherent methods.

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
				RcCoyoneda,
				RcFree,
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

	/// Erased-substrate Run program with `Rc`-shared continuations.
	///
	/// Thin wrapper over
	/// [`RcFree<NodeBrand<R, S>, A>`](crate::types::RcFree). Users
	/// reach for `RcRun` when an effect needs multi-shot continuations
	/// (the program may be re-driven by the same handler more than
	/// once); cloning is O(1).
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	pub struct RcRun<R, S, A>(RcFree<NodeBrand<R, S>, A>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static;

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRun` instance to clone.")]
	impl<R, S, A> Clone for RcRun<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Clones the `RcRun` by bumping the refcount on the inner
		/// [`RcFree`](crate::types::RcFree). O(1).
		#[document_signature]
		///
		#[document_returns("A new `RcRun` representing an independent branch.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		RcFree,
		/// 		effects::rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::from_rc_free(RcFree::pure(42));
		/// let branch = rc_run.clone();
		/// assert!(matches!(branch.peel(), Ok(42)));
		/// ```
		fn clone(&self) -> Self {
			RcRun(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRun` instance.")]
	impl<R, S, A> RcRun<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Wraps an [`RcFree<NodeBrand<R, S>, A>`](crate::types::RcFree)
		/// as an `RcRun<R, S, A>`. Zero-cost.
		#[document_signature]
		///
		#[document_parameters("The underlying `RcFree` computation.")]
		///
		#[document_returns("An `RcRun` wrapping `rc_free`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		RcFree,
		/// 		effects::rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::from_rc_free(RcFree::pure(7));
		/// assert!(matches!(rc_run.peel(), Ok(7)));
		/// ```
		#[inline]
		pub fn from_rc_free(rc_free: RcFree<NodeBrand<R, S>, A>) -> Self {
			RcRun(rc_free)
		}

		/// Unwraps an `RcRun<R, S, A>` to its underlying
		/// [`RcFree<NodeBrand<R, S>, A>`](crate::types::RcFree).
		/// Zero-cost.
		#[document_signature]
		///
		#[document_returns("The underlying `RcFree` computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		RcFree,
		/// 		effects::rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::from_rc_free(RcFree::pure(7));
		/// let rc_free: RcFree<NodeBrand<FirstRow, Scoped>, i32> = rc_run.into_rc_free();
		/// assert!(matches!(rc_free.resume(), Ok(7)));
		/// ```
		#[inline]
		pub fn into_rc_free(self) -> RcFree<NodeBrand<R, S>, A> {
			self.0
		}

		/// Wraps a value in a pure `RcRun` computation. Delegates to
		/// [`RcFree::pure`](crate::types::RcFree).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `RcRun` computation that produces `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// // Identity-headed row is used for assertions that engage `peel`
		/// // (which carries a per-projection `Clone` bound that the
		/// // canonical `Coyoneda` row does not satisfy).
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::pure(42);
		/// assert!(matches!(rc_run.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self {
			RcRun::from_rc_free(RcFree::pure(a))
		}

		/// Decomposes this `RcRun` computation into one step. Returns
		/// `Ok(a)` if the program is a pure value, or `Err(layer)`
		/// carrying the next `RcRun` continuation in a [`Node`](crate::types::effects::node::Node) layer.
		/// Delegates to [`RcFree::resume`](crate::types::RcFree).
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` for a pure result, or `Err(layer)` carrying the next `RcRun` step."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// // Identity-headed row: `peel`'s per-projection `Clone` bound
		/// // is satisfied by `Identity<RcFree>: Clone`.
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::pure(7);
		/// assert!(matches!(rc_run.peel(), Ok(7)));
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "Return type encodes Result<A, NodeBrand<R, S>::Of<'static, RcRun<R, S, A>>>; the GAT projection is structurally complex but cannot be aliased without losing the projection link the wrapper depends on."
		)]
		pub fn peel(
			self
		) -> Result<
			A,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
		>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			self.0
				.resume()
				.map_err(|node| <NodeBrand<R, S> as Functor>::map(RcRun::from_rc_free, node))
		}

		/// Lifts a [`Node`](crate::types::effects::node::Node) dispatch layer into the `RcRun` program.
		/// The `node` argument is the
		/// [`NodeBrand<R, S>`](crate::brands::NodeBrand) `Of<'static, A>`
		/// projection (typically `Node::First(<R as Member<...>>::inject(operation))`);
		/// `send` delegates to
		/// [`RcFree::lift_f`](crate::types::RcFree). The
		/// `Node`-projection signature is symmetric across the six Run
		/// wrappers; see
		/// [`Run::send`](crate::types::effects::run::Run::send) for the
		/// rationale.
		#[document_signature]
		///
		#[document_parameters("The Node dispatch layer carrying the effect operation.")]
		///
		#[document_returns(
			"An `RcRun` computation that performs the effect and returns its result."
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
		/// 			rc_run::RcRun,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let layer = Coproduct::inject(Identity(7));
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::send(Node::First(layer));
		/// let next = match rc_run.peel() {
		/// 	Err(Node::First(Coproduct::Inl(Identity(n)))) => n,
		/// 	_ => panic!("expected First(Inl(Identity(..))) layer"),
		/// };
		/// assert!(matches!(next.peel(), Ok(7)));
		/// ```
		#[inline]
		pub fn send(
			node: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		) -> Self
		where
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			RcRun::from_rc_free(RcFree::<NodeBrand<R, S>, A>::lift_f(node))
		}

		/// Lifts a raw effect value into an `RcRun` program.
		///
		/// Erased Rc-substrate analog of [`Run::lift`](crate::types::effects::run::Run::lift).
		/// Wraps the effect in [`RcCoyoneda::lift`](crate::types::RcCoyoneda::lift)
		/// (the `Rc`-pointer Coyoneda variant) rather than bare
		/// [`Coyoneda`](crate::types::Coyoneda): `RcRun::send` carries an
		/// `Of<'_, RcFree<..., RcTypeErasedValue>>: Clone` bound (intrinsic
		/// to `RcFree`'s shared-`Rc` state), which `RcCoyoneda` satisfies
		/// (`Rc::clone` is unconditional) but bare `Coyoneda` does not
		/// (its `Box<dyn FnOnce>` continuation is not `Clone`). This
		/// pairs the wrapper's pointer kind with the matching Coyoneda
		/// variant: `Run`->`Coyoneda`, `RcRun`->`RcCoyoneda`,
		/// `ArcRun`->`ArcCoyoneda`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect being lifted.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The effect value to lift.")]
		///
		#[document_returns("An `RcRun` program suspended at the lifted effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::lift::<IdentityBrand, _>(Identity(42));
		/// // The program is suspended at the lifted effect; peel reveals the layer.
		/// assert!(rc_run.peel().is_err());
		/// ```
		#[inline]
		pub fn lift<EBrand, Idx>(
			effect: Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, EBrand, A>, Idx>,
			EBrand: Kind_cdc7cd43dac7585f + 'static,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let coyo: RcCoyoneda<'static, EBrand, A> = RcCoyoneda::lift(effect);
			let layer =
				<Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>) as Member<
					RcCoyoneda<'static, EBrand, A>,
					Idx,
				>>::inject(coyo);
			Self::send(Node::First(layer))
		}

		/// Sequences this `RcRun` with a continuation `f`. Delegates to
		/// [`RcFree::bind`](crate::types::RcFree).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to chain after this computation.")]
		///
		#[document_returns("A new `RcRun` chaining `f` after this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> =
		/// 	RcRun::pure(2).bind(|x| RcRun::pure(x + 1)).bind(|x| RcRun::pure(x * 10));
		/// assert!(matches!(rc_run.peel(), Ok(30)));
		/// ```
		#[inline]
		pub fn bind<B: 'static>(
			self,
			f: impl Fn(A) -> RcRun<R, S, B> + 'static,
		) -> RcRun<R, S, B>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			RcRun::from_rc_free(self.0.bind(move |a| f(a).into_rc_free()))
		}

		/// Functor map over the result of this `RcRun`. Delegates to
		/// [`RcFree::map`](crate::types::RcFree).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `RcRun` with `f` applied to its result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::pure(7).map(|x| x * 3);
		/// assert!(matches!(rc_run.peel(), Ok(21)));
		/// ```
		#[inline]
		pub fn map<B: 'static>(
			self,
			f: impl Fn(A) -> B + 'static,
		) -> RcRun<R, S, B>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			RcRun::from_rc_free(self.0.map(f))
		}

		/// By-reference [`bind`](RcRun::bind): chains a continuation
		/// that receives `&A` rather than `A`.
		///
		/// Implemented via `self.clone().bind(move |a| f(&a))`. The
		/// clone is `O(1)` (atomic-free `Rc::clone` on the inner
		/// substrate); the wrapping closure converts the owned `A`
		/// from the substrate's by-value bind path back into the
		/// `&A` the user-supplied `f` expects.
		///
		/// This is the inherent escape hatch for by-reference
		/// dispatch over canonical Coyoneda-headed effect rows,
		/// where brand-level `RefSemimonad::ref_bind` is unreachable
		/// because `CoyonedaBrand: RefFunctor` is unimplementable on
		/// stable Rust (see
		/// [`limitations-and-workarounds.md`](../../../../docs/limitations-and-workarounds.md)).
		/// The `im_do!` macro's `ref` form desugars to this method.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to chain after this computation.")]
		///
		#[document_returns("A new `RcRun` chaining `f` after a clone of this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::pure(2);
		/// let chained = rc_run.ref_bind(|x: &i32| RcRun::pure(*x + 1));
		/// assert!(matches!(chained.peel(), Ok(3)));
		/// ```
		#[inline]
		pub fn ref_bind<B: 'static>(
			&self,
			f: impl Fn(&A) -> RcRun<R, S, B> + 'static,
		) -> RcRun<R, S, B>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			self.clone().bind(move |a| f(&a))
		}

		/// By-reference [`map`](RcRun::map): applies a function that
		/// takes `&A` rather than `A`.
		///
		/// Implemented via `self.clone().map(move |a| f(&a))`. The
		/// clone is `O(1)` (atomic-free `Rc::clone` on the inner
		/// substrate). See [`ref_bind`](RcRun::ref_bind) for the
		/// canonical-row design rationale.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply by reference to the result.")]
		///
		#[document_returns("A new `RcRun` with `f` applied to a clone of this one's result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::pure(7);
		/// let mapped = rc_run.ref_map(|x: &i32| *x * 3);
		/// assert!(matches!(mapped.peel(), Ok(21)));
		/// ```
		#[inline]
		pub fn ref_map<B: 'static>(
			&self,
			f: impl Fn(&A) -> B + 'static,
		) -> RcRun<R, S, B>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			self.clone().map(move |a| f(&a))
		}

		/// By-reference [`pure`](RcRun::pure): wraps a cloned value
		/// in an `RcRun` computation.
		///
		/// Implemented as `RcRun::pure(a.clone())`. Requires
		/// `A: Clone`. Parallel to brand-level
		/// [`RefPointed::ref_pure`](crate::classes::RefPointed) for
		/// types where brand-level dispatch isn't reachable.
		///
		/// The `im_do!` macro's `ref` form rewrites bare
		/// `pure(x)` calls inside `im_do!(ref RcRun { ... })` to
		/// this method.
		#[document_signature]
		///
		#[document_parameters("A reference to the value to wrap.")]
		///
		#[document_returns("An `RcRun` computation that produces a clone of `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let value = 42;
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::ref_pure(&value);
		/// assert!(matches!(rc_run.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn ref_pure(a: &A) -> Self
		where
			A: Clone, {
			RcRun::pure(a.clone())
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRun` instance.")]
	impl<R, S, A> RcRun<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Interprets this `RcRun` program by walking each effect via
		/// the matching handler closure in `handlers`, looping until
		/// the program reduces to a [`Pure`](crate::types::RcFree)
		/// value.
		///
		/// Multi-shot variant of [`Run::interpret`](crate::types::effects::run::Run::interpret).
		/// Each [`peel`](RcRun::peel) requires `A: Clone` and
		/// `RcFree`-projection `Clone` because the substrate
		/// participates in multi-shot continuation cloning. See
		/// [`Run::interpret`](crate::types::effects::run::Run::interpret)
		/// for the design rationale, mono-in-`A` step-function shape,
		/// and PureScript-Run cross-reference.
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
		/// 			rc_run::RcRun,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::lift::<IdentityBrand, _>(Identity(42));
		/// let result = prog.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<RcRun<FirstRow, Scoped, i32>>| op.0,
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
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RcRun<R, S, A>>),
				RcRun<R, S, A>,
			>,
			scoped_handlers: impl for<'h> DispatchScopedHandlers<
				'h,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RcRun<R, S, A>>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RcRun<R, S, A>>),
				RcRun<R, S, A>,
			>,
		) -> A
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
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

		/// Alias for [`interpret`](RcRun::interpret), kept for naming
		/// parity with PureScript Run's
		/// [`run`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs).
		/// See [`Run::run`](crate::types::effects::run::Run::run).
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
		/// 			rc_run::RcRun,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::lift::<IdentityBrand, _>(Identity(99));
		/// let result = prog.run(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<RcRun<FirstRow, Scoped, i32>>| op.0,
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
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RcRun<R, S, A>>),
				RcRun<R, S, A>,
			>,
			scoped_handlers: impl for<'h> DispatchScopedHandlers<
				'h,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RcRun<R, S, A>>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RcRun<R, S, A>>),
				RcRun<R, S, A>,
			>,
		) -> A
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			self.interpret(handlers, scoped_handlers)
		}

		/// MonadRec-target interpreter for [`RcRun`]. Mirrors
		/// [`Run::interpret_rec`](crate::types::effects::run::Run::interpret_rec);
		/// see that method's docs for the handler shape, loop body,
		/// and stack-safety guarantee. `RcRun` differences: the
		/// per-`peel` `A: Clone` and substrate-`Of<...>: Clone` bounds
		/// propagate through the recursion (matching
		/// [`RcRun::interpret`]).
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
		/// 			rc_run::RcRun,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::lift::<IdentityBrand, _>(Identity(42));
		/// let result: Thunk<'static, i32> = prog.interpret_rec::<ThunkBrand>(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Thunk<'static, RcRun<FirstRow, Scoped, i32>>>| op.0,
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
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
			> + 'static,
			scoped_handlers: impl for<'h> DispatchScopedHandlers<
				'h,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
			> + 'static,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		where
			MBrand: MonadRec + 'static,
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			tail_rec_m::<MBrand, RcRun<R, S, A>, A>(
				move |prog: RcRun<R, S, A>| match prog.peel() {
					Ok(a) => <MBrand as Pointed>::pure::<ControlFlow<A, RcRun<R, S, A>>>(
						ControlFlow::Break(a),
					),
					Err(Node::First(layer)) => {
						let mapped = <R as Functor>::map(
							|inner: RcRun<R, S, A>| {
								<MBrand as Pointed>::pure::<RcRun<R, S, A>>(inner)
							},
							layer,
						);
						let next = handlers.dispatch(mapped);
						<MBrand as Functor>::map::<RcRun<R, S, A>, ControlFlow<A, RcRun<R, S, A>>>(
							ControlFlow::Continue,
							next,
						)
					}
					Err(Node::Scoped(layer)) => {
						let mapped = <S as Functor>::map(
							|inner: RcRun<R, S, A>| {
								<MBrand as Pointed>::pure::<RcRun<R, S, A>>(inner)
							},
							layer,
						);
						let next = scoped_handlers.dispatch_scoped(mapped, &handlers);
						<MBrand as Functor>::map::<RcRun<R, S, A>, ControlFlow<A, RcRun<R, S, A>>>(
							ControlFlow::Continue,
							next,
						)
					}
				},
				self,
			)
		}

		/// Alias for [`interpret_rec`](RcRun::interpret_rec). See
		/// [`Run::run_rec`](crate::types::effects::run::Run::run_rec).
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
		/// 			rc_run::RcRun,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::lift::<IdentityBrand, _>(Identity(99));
		/// let result: Thunk<'static, i32> = prog.run_rec::<ThunkBrand>(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Thunk<'static, RcRun<FirstRow, Scoped, i32>>>| op.0,
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
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
			> + 'static,
			scoped_handlers: impl for<'h> DispatchScopedHandlers<
				'h,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
			> + 'static,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		where
			MBrand: MonadRec + 'static,
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			self.interpret_rec::<MBrand>(handlers, scoped_handlers)
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRun` instance.")]
	impl<R, S, A> RcRun<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Scoped-row-narrowing interpreter. See
		/// [`Run::interpret_scoped_with`](crate::types::effects::run::Run::interpret_scoped_with)
		/// for the cross-wrapper semantics. Differences for `RcRun`:
		/// recursive descent carries the shared-substrate `Clone`
		/// bounds required by [`RcFree`](crate::types::RcFree).
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
		#[document_returns("An `RcRun` program in the narrowed scoped row.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		rc_run::RcRun,
		/// 		span::Span,
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
		///
		/// let action: RcRun<CNilBrand, ScopedRow, i32> = RcRun::pure(7);
		/// let prog: RcRun<CNilBrand, ScopedRow, i32> = RcRun::span::<&'static str, _>("request", action);
		/// let narrowed: RcRun<CNilBrand, CNilBrand, i32> = prog
		/// 	.interpret_scoped_with::<SpanBrand<RcBrand, &'static str>, _, CNilBrand>(
		/// 		|span| match span {
		/// 			Span::Span {
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
		pub fn interpret_scoped_with<SBrand, Idx, SMinusE>(
			self,
			handler: impl Fn(
				Apply!(<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, SMinusE, A>>),
			) -> RcRun<R, SMinusE, A>
			+ 'static,
		) -> RcRun<R, SMinusE, A>
		where
			A: Clone,
			SBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<R, SMinusE> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, SMinusE>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>): Member<
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>
					),
					Idx,
					Remainder = Apply!(
									<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>
								),
				>, {
			let handler = <RcBrand as RefCountedPointer>::new(handler);
			self.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, _>(handler)
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
		#[document_returns("An `RcRun` program in the narrowed scoped row.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		rc_run::RcRun,
		/// 		span::Span,
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
		///
		/// // Exercised internally by RcRun::interpret_scoped_with.
		/// let action: RcRun<CNilBrand, ScopedRow, i32> = RcRun::pure(7);
		/// let prog: RcRun<CNilBrand, ScopedRow, i32> = RcRun::span::<&'static str, _>("request", action);
		/// let narrowed: RcRun<CNilBrand, CNilBrand, i32> = prog
		/// 	.interpret_scoped_with::<SpanBrand<RcBrand, &'static str>, _, CNilBrand>(
		/// 		|span| match span {
		/// 			Span::Span {
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
		fn interpret_scoped_with_shared<SBrand, Idx, SMinusE, F>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'static, F>,
		) -> RcRun<R, SMinusE, A>
		where
			F: Fn(
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							RcRun<R, SMinusE, A>,
						>
					),
				) -> RcRun<R, SMinusE, A>
				+ 'static,
			A: Clone,
			SBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<R, SMinusE> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, SMinusE>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>): Member<
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>
					),
					Idx,
					Remainder = Apply!(
									<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>
								),
				>, {
			match self.peel() {
				Ok(a) => RcRun::pure(a),
				Err(Node::First(layer)) => {
					let h_for_recurse = handler.clone();
					let mapped_free = <R as Functor>::map(
						move |inner: RcRun<R, S, A>| {
							inner
								.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
									h_for_recurse.clone(),
								)
								.into_rc_free()
						},
						layer,
					);
					RcRun::from_rc_free(RcFree::<NodeBrand<R, SMinusE>, A>::wrap(Node::First(
						mapped_free,
					)))
				}
				Err(Node::Scoped(layer)) =>
					match <Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RcRun<R, S, A>,
					>) as Member<
						Apply!(
							<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'static,
								RcRun<R, S, A>,
							>
						),
						Idx,
					>>::project(layer)
					{
						Ok(scoped) => {
							let h_for_recurse = handler.clone();
							let mapped = <SBrand as Functor>::map(
								move |inner: RcRun<R, S, A>| {
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
							let mapped_free = <SMinusE as Functor>::map(
								move |inner: RcRun<R, S, A>| {
									inner
										.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
											h_for_recurse.clone(),
										)
										.into_rc_free()
								},
								rest,
							);
							RcRun::from_rc_free(RcFree::<NodeBrand<R, SMinusE>, A>::wrap(
								Node::Scoped(mapped_free),
							))
						}
					},
			}
		}

		/// Pipeline row-narrowing interpreter. See
		/// [`Run::interpret_with`](crate::types::effects::run::Run::interpret_with)
		/// for the cross-wrapper semantics. Differences for `RcRun`:
		/// the [`RcCoyoneda`] variant pairs with the `Rc`-shared
		/// substrate; matched-arm dispatch uses
		/// [`RcCoyoneda::lower_ref`] (multi-shot friendly); the
		/// per-`peel` `Clone` bound on the substrate's `Of<'_,
		/// RcFree<...>>` propagates through the recursion.
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
		#[document_returns("An `RcRun` program in the narrowed row.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// let prog: RcRun<FullRow, CNilBrand, i32> = RcRun::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: RcRun<EmptyRow, CNilBrand, i32> = prog
		/// 	.interpret_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<RcRun<EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		pub fn interpret_with<EBrand, Idx, RMinusE>(
			self,
			handler: impl Fn(
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<RMinusE, S, A>>),
			) -> RcRun<RMinusE, S, A>
			+ 'static,
		) -> RcRun<RMinusE, S, A>
		where
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusE, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusE, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>): Member<
					RcCoyoneda<'static, EBrand, RcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>
								),
				>, {
			let handler = <RcBrand as RefCountedPointer>::new(handler);
			self.interpret_with_shared::<EBrand, Idx, RMinusE, _>(handler)
		}

		/// Inner pipeline-narrowing implementation, parameterised
		/// over the concrete handler closure type `F`. The public
		/// [`interpret_with`](RcRun::interpret_with) wraps the user
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
		#[document_returns("An `RcRun` program in the narrowed row `RMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// // Exercised internally by RcRun::interpret_with.
		/// let prog: RcRun<FullRow, CNilBrand, i32> = RcRun::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: RcRun<EmptyRow, CNilBrand, i32> = prog
		/// 	.interpret_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<RcRun<EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		fn interpret_with_shared<EBrand, Idx, RMinusE, F>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'static, F>,
		) -> RcRun<RMinusE, S, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<RMinusE, S, A>>),
				) -> RcRun<RMinusE, S, A>
				+ 'static,
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusE, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusE, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>): Member<
					RcCoyoneda<'static, EBrand, RcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>
								),
				>, {
			match self.peel() {
				Ok(a) => RcRun::pure(a),
				Err(Node::First(layer)) =>
					match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>
					) as Member<RcCoyoneda<'static, EBrand, RcRun<R, S, A>>, Idx>>::project(
						layer
					) {
						Ok(coyo) => {
							let lowered = coyo.lower_ref();
							let h_for_recurse = handler.clone();
							let mapped = <EBrand as Functor>::map(
								move |inner: RcRun<R, S, A>| {
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
								move |inner: RcRun<R, S, A>| {
									inner
										.interpret_with_shared::<EBrand, Idx, RMinusE, F>(
											h_for_recurse.clone(),
										)
										.into_rc_free()
								},
								rest,
							);
							RcRun::from_rc_free(RcFree::<NodeBrand<RMinusE, S>, A>::wrap(
								Node::First(mapped_free),
							))
						}
					},
				Err(Node::Scoped(layer)) => {
					let h_for_recurse = handler.clone();
					let mapped_free = <S as Functor>::map(
						move |inner: RcRun<R, S, A>| {
							inner
								.interpret_with_shared::<EBrand, Idx, RMinusE, F>(
									h_for_recurse.clone(),
								)
								.into_rc_free()
						},
						layer,
					);
					RcRun::from_rc_free(RcFree::<NodeBrand<RMinusE, S>, A>::wrap(Node::Scoped(
						mapped_free,
					)))
				}
			}
		}

		/// Substrate-level `interpose` primitive. Walks the
		/// underlying [`RcFree`] tree, finds dispatches against
		/// `EBrand`, applies `replacement`, and re-emits in the same
		/// row (no row narrowing).
		///
		/// The Rust analogue of heftia's `interposeInWith`. Unlike
		/// [`interpret_with`](RcRun::interpret_with) which narrows
		/// the row by removing `EBrand`, `interpose` keeps the full
		/// row intact and substitutes only the matched-effect
		/// dispatches with the user-supplied replacement; recursion
		/// also walks the inner sub-programs of unmatched effects so
		/// the entire program tree is re-interposed.
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
		/// 	brands::{
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		IdentityBrand,
		/// 		RcCoyonedaBrand,
		/// 	},
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = RcRun<Row, CNilBrand, i32>;
		///
		/// let prog: Prog = RcRun::lift::<IdentityBrand, _>(Identity(7));
		/// // Replacement substitutes each matched dispatch with a fresh
		/// // program; here we return `pure(99)` for the Identity dispatch,
		/// // demonstrating that the matched arm fires.
		/// let interposed =
		/// 	prog.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| RcRun::pure(99));
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
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
			) -> RcRun<R, S, A>
			+ 'static,
		) -> RcRun<R, S, A>
		where
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>): Member<
					RcCoyoneda<'static, EBrand, RcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RcFree<NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			let replacement = <RcBrand as RefCountedPointer>::new(replacement);
			self.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, _>(replacement)
		}

		/// Inner shared implementation of [`interpose`](RcRun::interpose),
		/// parameterised over the concrete replacement closure type
		/// `F`. The public [`interpose`](RcRun::interpose) wraps the
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
		/// // Exercised internally by RcRun::interpose.
		/// use fp_library::{
		/// 	brands::{
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		IdentityBrand,
		/// 		RcCoyonedaBrand,
		/// 	},
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = RcRun<Row, CNilBrand, i32>;
		///
		/// let prog: Prog = RcRun::lift::<IdentityBrand, _>(Identity(3));
		/// let interposed =
		/// 	prog.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| RcRun::pure(42));
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
		) -> RcRun<R, S, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
				) -> RcRun<R, S, A>
				+ 'static,
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>): Member<
					RcCoyoneda<'static, EBrand, RcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RcFree<NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => RcRun::pure(a),
				Err(Node::First(layer)) =>
					match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>
					) as Member<RcCoyoneda<'static, EBrand, RcRun<R, S, A>>, Idx>>::project(
						layer
					) {
						Ok(coyo) => {
							let lowered = coyo.lower_ref();
							let r_for_recurse = replacement.clone();
							let mapped = <EBrand as Functor>::map(
								move |inner: RcRun<R, S, A>| {
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
								move |inner: RcRun<R, S, A>| {
									inner
										.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
											r_for_recurse.clone(),
										)
										.into_rc_free()
								},
								rest,
							);
							let layer_back = mapped_rest.embed();
							RcRun::from_rc_free(RcFree::<NodeBrand<R, S>, A>::wrap(Node::First(
								layer_back,
							)))
						}
					},
				Err(Node::Scoped(layer)) => {
					let r_for_recurse = replacement.clone();
					let mapped_free = <S as Functor>::map(
						move |inner: RcRun<R, S, A>| {
							inner
								.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
									r_for_recurse.clone(),
								)
								.into_rc_free()
						},
						layer,
					);
					RcRun::from_rc_free(RcFree::<NodeBrand<R, S>, A>::wrap(Node::Scoped(
						mapped_free,
					)))
				}
			}
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `RcRun` instance.")]
	impl<R, A> RcRun<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Substrate-level matched-effect short-circuit primitive:
		/// walk this `RcRun` program, dispatching non-matched
		/// first-order effects through `fo_handlers` and
		/// short-circuiting the moment a matched-effect (`EBrand`)
		/// dispatch is encountered, returning the matched effect's
		/// lowered payload. Direct analog of heftia's
		/// `interpretWithEither` substrate primitive.
		///
		/// Returns `Ok(a)` when the program reduces to a pure value
		/// without firing the matched effect; returns `Err(op)` with
		/// the matched effect's lowered payload (`<EBrand as Kind>::Of<'static, Self>`)
		/// the moment the matched effect is dispatched. Pairs with
		/// scoped `Catch` handlers: a `Catch` dispatcher installs
		/// `interpret_with_either::<ExceptBrand<E>, _, RMinusE>(body, fo_handlers)`
		/// to test the body program; on `Err(throw)` it invokes the
		/// recovery program; on `Ok(a)` it returns the body's value.
		///
		/// `fo_handlers` covers only the non-matched effects (the
		/// `RMinusE` row); the matched effect short-circuits without
		/// any handler invocation. The program type retains the full
		/// row `R` (including `EBrand`) because the matched effect's
		/// dispatches throughout the program tree are discharged
		/// uniformly by the short-circuit, not by row narrowing.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the matched effect (the one that short-circuits).",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand (the row with `EBrand` removed at position `Idx`)."
		)]
		///
		#[document_parameters(
			"The handler list covering non-matched first-order effects (typically built via the `handlers!` macro)."
		)]
		///
		#[document_returns(
			"`Ok(a)` if the program completes without firing the matched effect; `Err(op)` carrying the matched effect's lowered payload otherwise."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		ExceptBrand,
		/// 		IdentityBrand,
		/// 		RcCoyonedaBrand,
		/// 	},
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			except::Except,
		/// 			rc_run::RcRun,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<
		/// 	RcCoyonedaBrand<ExceptBrand<String>>,
		/// 	CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>,
		/// >;
		/// type RowMinusExcept = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = RcRun<Row, CNilBrand, i32>;
		///
		/// let prog: Prog = RcRun::throw::<String, _>("oops".to_string());
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
				Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RcRun<R, CNilBrand, A>>),
				RcRun<R, CNilBrand, A>,
			>,
		) -> Result<
			A,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, CNilBrand, A>>),
		>
		where
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, CNilBrand, A>>):
				Member<
						RcCoyoneda<'static, EBrand, RcRun<R, CNilBrand, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, CNilBrand, A>>
									),
					>, {
			let mut prog = self;
			loop {
				match prog.peel() {
					Ok(a) => return Ok(a),
					Err(Node::First(layer)) =>
						match <Apply!(
							<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, CNilBrand, A>>
						) as Member<RcCoyoneda<'static, EBrand, RcRun<R, CNilBrand, A>>, Idx>>::project(
							layer
						) {
							Ok(matched_coyo) => return Err(matched_coyo.lower_ref()),
							Err(rest) => prog = fo_handlers.dispatch(rest),
						},
					Err(Node::Scoped(cnil)) => match cnil {},
				}
			}
		}
	}

	#[document_type_parameters("The result type.")]
	#[document_parameters("The `RcRun` instance.")]
	impl<A> RcRun<CNilBrand, CNilBrand, A>
	where
		A: Clone + 'static,
	{
		/// Extracts the result value from an `RcRun` program whose
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
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// let pure_prog: RcRun<CNilBrand, CNilBrand, i32> = RcRun::pure(42);
		/// assert_eq!(pure_prog.extract(), 42);
		/// ```
		#[inline]
		pub fn extract(self) -> A
		where
			Apply!(<NodeBrand<CNilBrand, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<CNilBrand, CNilBrand>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
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
	impl<R, ScopedRow, A> RcRun<R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: Clone + 'static,
	{
		/// Lifts a `Get` state effect into the `RcRun` program.
		/// Mirrors [`Run::get`](crate::types::effects::run::Run::get);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RcRun`: the [`RcCoyoneda`] variant pairs with the `Rc`-shared
		/// substrate (single `Get` continuation cloning is via the
		/// `Rc<dyn Fn>` refcount bump). Threads
		/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
		///
		/// `Idx` is the type-level position witness identifying where
		/// `StateBrand<RcBrand, A>` lives in the row `R`. Rust infers
		/// `Idx` whenever the effect appears unambiguously in the row.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `RcRun` program suspended at the lifted `Get` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		rc_run::RcRun,
		/// 		state::State,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::get();
		/// // The program is suspended at the Get effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn get<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, crate::brands::StateBrand<RcBrand, A>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::state::State<'static, RcBrand, A, A> =
				crate::types::effects::state::State::Get(
					<RcBrand as crate::classes::ToDynCloneFn>::new(|s: A| s),
				);
			Self::lift::<crate::brands::StateBrand<RcBrand, A>, Idx>(effect)
		}

		/// Lifts an `Ask` reader effect into the `RcRun` program.
		/// Mirrors [`Run::ask`](crate::types::effects::run::Run::ask);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RcRun`: the [`RcCoyoneda`] variant pairs with the `Rc`-shared
		/// substrate. Threads
		/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
		///
		/// `Idx` is the type-level position witness identifying where
		/// `ReaderBrand<RcBrand, A>` lives in the row `R`. Rust infers
		/// `Idx` whenever the effect appears unambiguously in the row.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `RcRun` program suspended at the lifted `Ask` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		rc_run::RcRun,
		/// 		reader::Reader,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::ask();
		/// // The program is suspended at the Ask effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn ask<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, crate::brands::ReaderBrand<RcBrand, A>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::reader::Reader<'static, RcBrand, A, A> =
				crate::types::effects::reader::Reader::Ask(
					<RcBrand as crate::classes::ToDynCloneFn>::new(|e: A| e),
				);
			Self::lift::<crate::brands::ReaderBrand<RcBrand, A>, Idx>(effect)
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
		/// 	types::effects::{
		/// 		except::Except,
		/// 		rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::throw::<&'static str, _>("oops");
		/// assert!(prog.peel().is_err());
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
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<CatchBrand<RcBrand, &'static str>, CNilBrand>;
		///
		/// let action: RcRun<FirstRow, ScopedRow, i32> = RcRun::pure(42);
		/// let prog: RcRun<FirstRow, ScopedRow, i32> =
		/// 	RcRun::catch::<&'static str, _>(action, |_e| RcRun::pure(0));
		/// // The program is suspended at the Catch scoped layer; peel
		/// // returns Err carrying a `Node::Scoped(...)` projection.
		/// assert!(prog.peel().is_err());
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
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<LocalBrand<RcBrand, i32>, CNilBrand>;
		///
		/// let action: RcRun<FirstRow, ScopedRow, i32> = RcRun::pure(42);
		/// let prog: RcRun<FirstRow, ScopedRow, i32> = RcRun::local::<i32, _>(|e: i32| e + 1, action);
		/// // The program is suspended at the Local scoped layer; peel
		/// // returns Err carrying a `Node::Scoped(...)` projection.
		/// assert!(prog.peel().is_err());
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
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<RefLocalBrand<RcBrand, i32>, CNilBrand>;
		///
		/// let action: RcRun<FirstRow, ScopedRow, i32> = RcRun::pure(42);
		/// let prog: RcRun<FirstRow, ScopedRow, i32> =
		/// 	RcRun::ref_local::<i32, _>(|e: &i32| *e + 1, action);
		/// assert!(prog.peel().is_err());
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
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
		///
		/// let action: RcRun<FirstRow, ScopedRow, i32> = RcRun::pure(42);
		/// let prog: RcRun<FirstRow, ScopedRow, i32> = RcRun::span::<&'static str, _>("request", action);
		/// assert!(prog.peel().is_err());
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
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The resource type produced by acquire.",
		"The body's result type."
	)]
	impl<R, ScopedRow, A, B> RcRun<R, ScopedRow, (A, B)>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: 'static,
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
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
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
		/// marker-struct workaround validated by the
		/// [B18 POC](../../../../tests/poc_bracket_marker_row.rs).
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
		/// 	types::effects::rc_run::RcRun,
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
		/// let acquire: RcRun<FirstRow, ScopedRow, i32> = RcRun::pure(7);
		/// let prog: RcRun<FirstRow, ScopedRow, (i32, i32)> =
		/// 	RcRun::<FirstRow, ScopedRow, (i32, i32)>::bracket::<_>(
		/// 		acquire,
		/// 		|resource: std::rc::Rc<i32>| RcRun::pure((*resource, 42)),
		/// 		|_resource: std::rc::Rc<i32>| RcRun::pure(()),
		/// 	);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn bracket<Idx>(
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
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, (A, B)>,
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
				RcFree<NodeBrand<R, ScopedRow>, (A, B)>,
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
		/// marker-struct workaround validated by the
		/// [B18 POC](../../../../tests/poc_bracket_marker_row.rs).
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
		/// 	types::effects::rc_run::RcRun,
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
		/// let acquire: RcRun<FirstRow, ScopedRow, i32> = RcRun::pure(7);
		/// let prog: RcRun<FirstRow, ScopedRow, i32> =
		/// 	RcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 		acquire,
		/// 		|resource: std::rc::Rc<i32>| RcRun::pure(*resource + 35),
		/// 		|_resource: std::rc::Rc<i32>| RcRun::pure(()),
		/// 	);
		/// assert!(prog.peel().is_err());
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
		/// Lifts a `Put` state effect into the `RcRun` program.
		/// Mirrors [`Run::put`](crate::types::effects::run::Run::put);
		/// see that method for cross-wrapper semantics. Threads
		/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
		///
		/// `StateType` is the state type carried by `StateBrand` in
		/// the row. Rust may need a turbofish on `StateType` because
		/// `put`'s result type is `()` (which doesn't constrain the
		/// state type from the call site).
		#[document_signature]
		///
		#[document_type_parameters(
			"The state type carried by `StateBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The new state value to write.")]
		///
		#[document_returns("An `RcRun` program suspended at the lifted `Put` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		rc_run::RcRun,
		/// 		state::State,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, ()> = RcRun::put::<i32, _>(42);
		/// // The program is suspended at the Put effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn put<StateType: Clone + 'static, Idx>(s: StateType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				Member<RcCoyoneda<'static, crate::brands::StateBrand<RcBrand, StateType>, ()>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::state::State<'static, RcBrand, StateType, ()> =
				crate::types::effects::state::State::Put(
					s,
					<RcBrand as crate::classes::ToDynCloneFn>::new(|_: ()| ()),
				);
			Self::lift::<crate::brands::StateBrand<RcBrand, StateType>, Idx>(effect)
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
		/// 	types::effects::{
		/// 		rc_run::RcRun,
		/// 		writer::Writer,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, ()> = RcRun::tell::<&'static str, _>("logged");
		/// assert!(prog.peel().is_err());
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
		/// 	types::effects::{
		/// 		choose::Choose,
		/// 		rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, bool> = RcRun::choose();
		/// assert!(prog.peel().is_err());
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
				Identity,
				RcFree,
				effects::{
					coproduct::Coproduct,
					node::Node,
				},
			},
		},
	};

	type CoyonedaFirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
	type CoyonedaScoped = CNilBrand;
	// `peel` carries a per-projection `Clone` bound that the canonical
	// `Coyoneda`-wrapped row does not satisfy (`Coyoneda` is `!Clone`);
	// tests that exercise `peel` use an `Identity`-headed row instead.
	// `RcFree`'s outer `Rc<Inner>` provides the layout indirection that
	// makes the `Identity`-headed row well-formed for `RcRun`.
	type IdentityFirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
	type IdentityScoped = CNilBrand;

	#[test]
	fn from_rc_free_and_into_rc_free_round_trip() {
		let rc_free: RcFree<NodeBrand<CoyonedaFirstRow, CoyonedaScoped>, i32> = RcFree::pure(42);
		let rc_run: RcRun<CoyonedaFirstRow, CoyonedaScoped, i32> = RcRun::from_rc_free(rc_free);
		let _back: RcFree<NodeBrand<CoyonedaFirstRow, CoyonedaScoped>, i32> = rc_run.into_rc_free();
	}

	#[test]
	fn clone_bumps_refcount_in_constant_time() {
		let rc_run: RcRun<CoyonedaFirstRow, CoyonedaScoped, i32> =
			RcRun::from_rc_free(RcFree::pure(7));
		let _branch = rc_run.clone();
	}

	#[test]
	fn drop_a_pure_rc_run_does_not_panic() {
		let rc_run: RcRun<CoyonedaFirstRow, CoyonedaScoped, i32> =
			RcRun::from_rc_free(RcFree::pure(7));
		drop(rc_run);
	}

	#[test]
	fn pure_then_peel_returns_value() {
		let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::pure(42);
		assert!(matches!(rc_run.peel(), Ok(42)));
	}

	#[test]
	fn send_produces_suspended_program() {
		let layer = Coproduct::inject(Identity(7));
		let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::send(Node::First(layer));
		assert!(rc_run.peel().is_err());
	}

	#[test]
	fn into_explicit_via_into_round_trips_pure() {
		use crate::types::effects::rc_run_explicit::RcRunExplicit;
		let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::pure(42);
		let explicit: RcRunExplicit<'static, IdentityFirstRow, IdentityScoped, i32> = rc_run.into();
		assert!(matches!(explicit.peel(), Ok(42)));
	}

	#[test]
	fn into_explicit_via_into_preserves_suspended_layer() {
		use crate::types::effects::rc_run_explicit::RcRunExplicit;
		let layer = Coproduct::inject(Identity(7));
		let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::send(Node::First(layer));
		let explicit: RcRunExplicit<'static, IdentityFirstRow, IdentityScoped, i32> = rc_run.into();
		assert!(explicit.peel().is_err());
	}

	#[test]
	fn bind_chains_pure_values() {
		let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> =
			RcRun::pure(2).bind(|x| RcRun::pure(x + 1)).bind(|x| RcRun::pure(x * 10));
		assert!(matches!(rc_run.peel(), Ok(30)));
	}

	#[test]
	fn map_transforms_pure_value() {
		let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::pure(7).map(|x| x * 3);
		assert!(matches!(rc_run.peel(), Ok(21)));
	}

	#[test]
	fn ref_bind_chains_pure_value_via_clone() {
		let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::pure(2);
		let chained = rc_run.ref_bind(|x: &i32| RcRun::pure(*x + 1));
		assert!(matches!(chained.peel(), Ok(3)));
	}

	#[test]
	fn ref_map_transforms_pure_value_via_clone() {
		let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::pure(7);
		let mapped = rc_run.ref_map(|x: &i32| *x * 3);
		assert!(matches!(mapped.peel(), Ok(21)));
	}

	#[test]
	fn ref_pure_wraps_cloned_value() {
		let value = 42;
		let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::ref_pure(&value);
		assert!(matches!(rc_run.peel(), Ok(42)));
	}
}
