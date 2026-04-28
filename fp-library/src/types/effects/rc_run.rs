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
//! ## Step 4a scope
//!
//! This module currently only ships the type-level wrapper plus the
//! [`from_rc_free`](RcRun::from_rc_free) /
//! [`into_rc_free`](RcRun::into_rc_free) construction sugar. The
//! user-facing operations (`pure`, `peel`, `send`, `bind`, `map`,
//! `lift_f`, `evaluate`, `handle`, etc.) land in Phase 2 step 5.

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
				RcCoyoneda,
				RcFree,
				effects::{
					interpreter::DispatchHandlers,
					member::Member,
					node::Node,
				},
			},
		},
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
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::from_rc_free(RcFree::pure(42));
		/// let _branch = rc_run.clone();
		/// assert!(true);
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
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let _rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::from_rc_free(RcFree::pure(7));
		/// assert!(true);
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
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::from_rc_free(RcFree::pure(7));
		/// let _rc_free: RcFree<NodeBrand<FirstRow, Scoped>, i32> = rc_run.into_rc_free();
		/// assert!(true);
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
		/// [`fp-library/docs/limitations-and-workarounds.md`](https://github.com/nothingnesses/rust-fp-library/blob/main/fp-library/docs/limitations-and-workarounds.md)).
		/// The
		/// [`im_do!`](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/plan.md)
		/// macro's `ref` form (Phase 2 step 7c) desugars to this
		/// method.
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
		/// The
		/// [`im_do!`](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/plan.md)
		/// macro's `ref` form (Phase 2 step 7c) rewrites bare
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
		/// 			rc_run::RcRun,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::lift::<IdentityBrand, _>(Identity(42));
		/// let result = prog.interpret(handlers! {
		/// 	IdentityBrand: |op: Identity<RcRun<FirstRow, Scoped, i32>>| op.0,
		/// });
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "Phase 3 first-order interpreter does not handle scoped layers; Phase 4 wires them."
		)]
		pub fn interpret(
			self,
			mut handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RcRun<R, S, A>>),
				RcRun<R, S, A>,
			>,
		) -> A
		where
			A: Clone,
			S: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
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

		/// Alias for [`interpret`](RcRun::interpret), kept for naming
		/// parity with PureScript Run's
		/// [`run`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs).
		/// See [`Run::run`](crate::types::effects::run::Run::run).
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
		/// 			rc_run::RcRun,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::lift::<IdentityBrand, _>(Identity(99));
		/// let result = prog.run(handlers! {
		/// 	IdentityBrand: |op: Identity<RcRun<FirstRow, Scoped, i32>>| op.0,
		/// });
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
		) -> A
		where
			A: Clone,
			S: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			self.interpret(handlers)
		}

		/// Interprets this `RcRun` program with a state value threaded
		/// through each handler invocation. See
		/// [`Run::run_accum`](crate::types::effects::run::Run::run_accum)
		/// for the state-threading model (closure captures, not a
		/// separate trait).
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
		/// 				rc_run::RcRun,
		/// 			},
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::RefCell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let counter: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
		/// let counter_for_handler = Rc::clone(&counter);
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::lift::<IdentityBrand, _>(Identity(7));
		/// let result = prog.run_accum(
		/// 	handlers! {
		/// 		IdentityBrand: move |op: Identity<RcRun<FirstRow, Scoped, i32>>| {
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
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RcRun<R, S, A>>),
				RcRun<R, S, A>,
			>,
			init: St,
		) -> A
		where
			A: Clone,
			S: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
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
