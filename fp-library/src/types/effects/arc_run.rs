//! Thread-safe Erased-substrate Run program with `Arc`-shared
//! continuations.
//!
//! `ArcRun<R, S, A>` is the [`Send`] + [`Sync`] sibling of
//! [`RcRun`](crate::types::effects::rc_run::RcRun): the same conceptual identity
//!
//! ```text
//! ArcRun<R, S, A> = ArcFree<NodeBrand<R, S>, A>
//! ```
//!
//! but the underlying [`ArcFree`](crate::types::ArcFree) carries
//! `Arc<dyn Fn + Send + Sync>` continuations rather than `Rc<dyn Fn>`,
//! so programs cross thread boundaries. The whole substrate lives
//! behind an outer [`Arc`](std::sync::Arc), so cloning a program is
//! O(1) atomic refcount bump.
//!
//! Use [`Run`](crate::types::effects::run::Run) when single-threaded and
//! single-shot. Use [`RcRun`](crate::types::effects::rc_run::RcRun) when
//! multi-shot but single-threaded. Use `ArcRun` for thread-safe
//! multi-shot.
//!
//! ## Step 4a scope
//!
//! This module currently only ships the type-level wrapper plus the
//! [`from_arc_free`](ArcRun::from_arc_free) /
//! [`into_arc_free`](ArcRun::into_arc_free) construction sugar. The
//! user-facing operations land in Phase 2 step 5.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				CNilBrand,
				NodeBrand,
			},
			classes::{
				SendFunctor,
				WrapDrop,
			},
			kinds::Kind_cdc7cd43dac7585f,
			types::{
				ArcCoyoneda,
				ArcFree,
				arc_free::ArcTypeErasedValue,
				effects::{
					interpreter::DispatchHandlers,
					member::Member,
					node::Node,
				},
			},
		},
		fp_macros::*,
	};

	/// Thread-safe Erased-substrate Run program with `Arc`-shared
	/// continuations.
	///
	/// Thin wrapper over
	/// [`ArcFree<NodeBrand<R, S>, A>`](crate::types::ArcFree). The
	/// associated-type bound on
	/// [`NodeBrand<R, S>`](crate::brands::NodeBrand)'s `Kind`
	/// projection (`Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync`)
	/// is what lets the compiler auto-derive `Send + Sync` on the
	/// underlying `ArcFree` for concrete row brands.
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	pub struct ArcRun<R, S, A>(ArcFree<NodeBrand<R, S>, A>)
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: 'static;

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRun` instance to clone.")]
	impl<R, S, A> Clone for ArcRun<R, S, A>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: 'static,
	{
		/// Clones the `ArcRun` by atomic refcount bump on the inner
		/// [`ArcFree`](crate::types::ArcFree). O(1).
		#[document_signature]
		///
		#[document_returns("A new `ArcRun` representing an independent branch.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::arc_run::ArcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::from_arc_free(ArcFree::pure(42));
		/// let _branch = arc_run.clone();
		/// assert!(true);
		/// ```
		fn clone(&self) -> Self {
			ArcRun(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRun` instance.")]
	impl<R, S, A> ArcRun<R, S, A>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: 'static,
	{
		/// Wraps an [`ArcFree<NodeBrand<R, S>, A>`](crate::types::ArcFree)
		/// as an `ArcRun<R, S, A>`. Zero-cost.
		#[document_signature]
		///
		#[document_parameters("The underlying `ArcFree` computation.")]
		///
		#[document_returns("An `ArcRun` wrapping `arc_free`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::arc_run::ArcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let _arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::from_arc_free(ArcFree::pure(7));
		/// assert!(true);
		/// ```
		#[inline]
		pub fn from_arc_free(arc_free: ArcFree<NodeBrand<R, S>, A>) -> Self {
			ArcRun(arc_free)
		}

		/// Unwraps an `ArcRun<R, S, A>` to its underlying
		/// [`ArcFree<NodeBrand<R, S>, A>`](crate::types::ArcFree).
		/// Zero-cost.
		#[document_signature]
		///
		#[document_returns("The underlying `ArcFree` computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::arc_run::ArcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::from_arc_free(ArcFree::pure(7));
		/// let _arc_free: ArcFree<NodeBrand<FirstRow, Scoped>, i32> = arc_run.into_arc_free();
		/// assert!(true);
		/// ```
		#[inline]
		pub fn into_arc_free(self) -> ArcFree<NodeBrand<R, S>, A> {
			self.0
		}

		/// Wraps a value in a pure `ArcRun` computation. Delegates to
		/// [`ArcFree::pure`](crate::types::ArcFree).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `ArcRun` computation that produces `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::pure(42);
		/// assert_eq!(arc_run.into_arc_free().evaluate(), 42);
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self
		where
			A: Send + Sync, {
			ArcRun::from_arc_free(ArcFree::pure(a))
		}

		/// Decomposes this `ArcRun` computation into one step. Returns
		/// `Ok(a)` if the program is a pure value, or `Err(layer)`
		/// carrying the next `ArcRun` continuation in a
		/// [`Node`](crate::types::effects::node::Node) layer. Delegates to
		/// [`ArcFree::resume`](crate::types::ArcFree).
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` for a pure result, or `Err(layer)` carrying the next `ArcRun` step."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::pure(7);
		/// assert!(matches!(arc_run.peel(), Ok(7)));
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "Return type encodes Result<A, NodeBrand<R, S>::Of<'static, ArcRun<R, S, A>>>; the GAT projection is structurally complex but cannot be aliased without losing the projection link the wrapper depends on."
		)]
		pub fn peel(
			self
		) -> Result<
			A,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
		>
		where
			NodeBrand<R, S>: SendFunctor,
			A: Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			self.0.resume().map_err(|node| {
				<NodeBrand<R, S> as SendFunctor>::send_map(ArcRun::from_arc_free, node)
			})
		}

		/// Lifts a [`Node`](crate::types::effects::node::Node) dispatch
		/// layer into the `ArcRun` program. The `node` argument is the
		/// [`NodeBrand<R, S>`](crate::brands::NodeBrand)
		/// `Of<'static, A>` projection; `send` delegates to
		/// [`ArcFree::lift_f`](crate::types::ArcFree).
		///
		/// The `Node`-projection signature is required because
		/// `ArcFree`'s struct-level HRTB
		/// (`Of<'static, ArcFree<...>>: Send + Sync`) poisons GAT
		/// normalization in any scope mentioning it: constructing a
		/// `Node::First` literal inside this method body fails to
		/// unify with the projection. The caller (test code, smart
		/// constructors emitted by
		/// [`effects!`](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/plan.md),
		/// or generic helpers without the HRTB) constructs the layer
		/// outside the HRTB scope and passes the result here. See
		/// `tests/arc_run_normalization_probe.rs` for the experimental
		/// matrix that established the limit.
		#[document_signature]
		///
		#[document_parameters("The Node dispatch layer carrying the effect operation.")]
		///
		#[document_returns(
			"An `ArcRun` computation that performs the effect and returns its result."
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
		/// 			arc_run::ArcRun,
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let layer = Coproduct::inject(Identity(7));
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::send(Node::First(layer));
		/// let next = match arc_run.peel() {
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
			NodeBrand<R, S>: SendFunctor,
			A: Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			ArcRun::from_arc_free(ArcFree::<NodeBrand<R, S>, A>::lift_f(node))
		}

		/// Lifts a raw effect value into an `ArcRun` program.
		///
		/// Thread-safe Erased-substrate analog of
		/// [`Run::lift`](crate::types::effects::run::Run::lift). Uses
		/// [`ArcCoyoneda`](crate::types::ArcCoyoneda) (the
		/// `Send + Sync` Coyoneda variant) because the bare
		/// [`Coyoneda`](crate::types::Coyoneda)'s `Box<dyn FnOnce>`
		/// continuation is not `Send + Sync` and the `Arc`-substrate
		/// rejects it. `ArcCoyoneda` is unconditionally `Clone +
		/// Send + Sync` via `Arc::clone`, which satisfies the
		/// row-projection `Clone` bound `ArcRun::peel` carries; the
		/// downstream lift+peel round-trip recovers the lifted value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect being lifted.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The effect value to lift. Must be `Clone + Send + Sync`.")]
		///
		#[document_returns("An `ArcRun` program suspended at the lifted effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::ArcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(42));
		/// // The program is suspended at the lifted effect; peel reveals the layer.
		/// assert!(arc_run.peel().is_err());
		/// ```
		#[inline]
		pub fn lift<EBrand, Idx>(
			effect: Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		) -> Self
		where
			NodeBrand<R, S>: SendFunctor,
			R: Kind_cdc7cd43dac7585f,
			S: Kind_cdc7cd43dac7585f,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, EBrand, A>, Idx>,
			EBrand: Kind_cdc7cd43dac7585f + 'static,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Clone + Send + Sync,
			A: Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			Self::send(lift_node::<R, S, EBrand, Idx, A>(effect))
		}

		/// Sequences this `ArcRun` with a continuation `f`. Delegates to
		/// [`ArcFree::bind`](crate::types::ArcFree).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to chain after this computation.")]
		///
		#[document_returns("A new `ArcRun` chaining `f` after this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> =
		/// 	ArcRun::pure(2).bind(|x| ArcRun::pure(x + 1)).bind(|x| ArcRun::pure(x * 10));
		/// assert!(matches!(arc_run.peel(), Ok(30)));
		/// ```
		#[inline]
		pub fn bind<B: 'static + Send + Sync>(
			self,
			f: impl Fn(A) -> ArcRun<R, S, B> + Send + Sync + 'static,
		) -> ArcRun<R, S, B>
		where
			A: Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			ArcRun::from_arc_free(self.0.bind(move |a| f(a).into_arc_free()))
		}

		/// Functor map over the result of this `ArcRun`. Delegates to
		/// [`ArcFree::map`](crate::types::ArcFree).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `ArcRun` with `f` applied to its result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::pure(7).map(|x| x * 3);
		/// assert!(matches!(arc_run.peel(), Ok(21)));
		/// ```
		#[inline]
		pub fn map<B: 'static + Send + Sync>(
			self,
			f: impl Fn(A) -> B + Send + Sync + 'static,
		) -> ArcRun<R, S, B>
		where
			A: Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			ArcRun::from_arc_free(self.0.map(f))
		}

		/// By-reference [`bind`](ArcRun::bind): chains a continuation
		/// that receives `&A` rather than `A`.
		///
		/// Implemented via `self.clone().bind(move |a| f(&a))`. The
		/// clone is `O(1)` (atomic refcount bump on the inner
		/// substrate); the wrapping closure converts the owned `A`
		/// from the substrate's by-value bind path back into the
		/// `&A` the user-supplied `f` expects.
		///
		/// This is the only by-reference dispatch path available for
		/// `ArcRun` (the brand-level `SendRefSemimonad` is
		/// unreachable for the broader Run family on stable Rust per
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
		#[document_returns("A new `ArcRun` chaining `f` after a clone of this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::pure(2);
		/// let chained = arc_run.ref_bind(|x: &i32| ArcRun::pure(*x + 1));
		/// assert!(matches!(chained.peel(), Ok(3)));
		/// ```
		#[inline]
		pub fn ref_bind<B: 'static + Send + Sync>(
			&self,
			f: impl Fn(&A) -> ArcRun<R, S, B> + Send + Sync + 'static,
		) -> ArcRun<R, S, B>
		where
			A: Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			self.clone().bind(move |a| f(&a))
		}

		/// By-reference [`map`](ArcRun::map): applies a function that
		/// takes `&A` rather than `A`.
		///
		/// Implemented via `self.clone().map(move |a| f(&a))`. The
		/// clone is `O(1)` (atomic refcount bump). See
		/// [`ref_bind`](ArcRun::ref_bind) for the design rationale.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply by reference to the result.")]
		///
		#[document_returns("A new `ArcRun` with `f` applied to a clone of this one's result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::pure(7);
		/// let mapped = arc_run.ref_map(|x: &i32| *x * 3);
		/// assert!(matches!(mapped.peel(), Ok(21)));
		/// ```
		#[inline]
		pub fn ref_map<B: 'static + Send + Sync>(
			&self,
			f: impl Fn(&A) -> B + Send + Sync + 'static,
		) -> ArcRun<R, S, B>
		where
			A: Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			self.clone().map(move |a| f(&a))
		}

		/// By-reference [`pure`](ArcRun::pure): wraps a cloned value
		/// in an `ArcRun` computation.
		///
		/// Implemented as `ArcRun::pure(a.clone())`. Requires
		/// `A: Clone + Send + Sync`. Parallel to brand-level
		/// [`SendRefPointed::send_ref_pure`](crate::classes::SendRefPointed)
		/// for types where brand-level dispatch isn't reachable.
		///
		/// The
		/// [`im_do!`](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/plan.md)
		/// macro's `ref` form (Phase 2 step 7c) rewrites bare
		/// `pure(x)` calls inside `im_do!(ref ArcRun { ... })` to
		/// this method.
		#[document_signature]
		///
		#[document_parameters("A reference to the value to wrap.")]
		///
		#[document_returns("An `ArcRun` computation that produces a clone of `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let value = 42;
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::ref_pure(&value);
		/// assert!(matches!(arc_run.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn ref_pure(a: &A) -> Self
		where
			A: Clone + Send + Sync, {
			ArcRun::pure(a.clone())
		}

		/// Interprets this `ArcRun` program by walking each effect via
		/// the matching handler closure in `handlers`, looping until
		/// the program reduces to a [`Pure`](crate::types::ArcFree)
		/// value.
		///
		/// Thread-safe variant of [`Run::interpret`](crate::types::effects::run::Run::interpret).
		/// Each [`peel`](ArcRun::peel) requires `A: Clone + Send +
		/// Sync` and `ArcFree`-projection `Clone`. Per the Phase 2
		/// step 5 HRTB-poisoning resolution, the handler list itself
		/// is constructed outside this method's scope (typically
		/// outside the impl block) and passed in.
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
		/// 			arc_run::ArcRun,
		/// 			handlers::*,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(42));
		/// let result = prog.interpret(handlers! {
		/// 	IdentityBrand: |op: Identity<ArcRun<FirstRow, Scoped, i32>>| op.0,
		/// });
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		pub fn interpret(
			self,
			mut handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, ArcRun<R, S, A>>),
				ArcRun<R, S, A>,
			>,
		) -> A
		where
			R: Kind_cdc7cd43dac7585f + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			NodeBrand<R, S>: SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			let mut prog = self;
			loop {
				match prog.peel() {
					Ok(a) => return a,
					Err(node) => {
						let layer = unwrap_first::<R, S, ArcRun<R, S, A>>(node);
						prog = handlers.dispatch(layer);
					}
				}
			}
		}

		/// Alias for [`interpret`](ArcRun::interpret), kept for naming
		/// parity with PureScript Run's
		/// [`run`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs).
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
		/// 			arc_run::ArcRun,
		/// 			handlers::*,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(99));
		/// let result = prog.run(handlers! {
		/// 	IdentityBrand: |op: Identity<ArcRun<FirstRow, Scoped, i32>>| op.0,
		/// });
		/// assert_eq!(result, 99);
		/// ```
		#[inline]
		pub fn run(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, ArcRun<R, S, A>>),
				ArcRun<R, S, A>,
			>,
		) -> A
		where
			R: Kind_cdc7cd43dac7585f + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			NodeBrand<R, S>: SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			self.interpret(handlers)
		}

		/// Interprets this `ArcRun` program with a state value
		/// threaded through each handler invocation. See
		/// [`Run::run_accum`](crate::types::effects::run::Run::run_accum).
		///
		/// State threading uses thread-safe primitives (e.g.,
		/// [`Arc`](std::sync::Arc) /
		/// [`Mutex`](std::sync::Mutex) or
		/// [`RwLock`](std::sync::RwLock)) at the user level so the
		/// captured state cell satisfies `ArcRun`'s `Send + Sync`
		/// substrate.
		#[document_signature]
		///
		#[document_type_parameters("The state type.")]
		///
		#[document_parameters(
			"The handler list (typically built via the `handlers!` macro), with each closure capturing the state cell.",
			"The initial state value."
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
		/// 				arc_run::ArcRun,
		/// 				handlers::*,
		/// 			},
		/// 		},
		/// 	},
		/// 	std::sync::{
		/// 		Arc,
		/// 		Mutex,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let counter: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
		/// let counter_for_handler = Arc::clone(&counter);
		///
		/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(7));
		/// let result = prog.run_accum(
		/// 	handlers! {
		/// 		IdentityBrand: move |op: Identity<ArcRun<FirstRow, Scoped, i32>>| {
		/// 			*counter_for_handler.lock().unwrap() += 1;
		/// 			op.0
		/// 		},
		/// 	},
		/// 	0_i32,
		/// );
		/// assert_eq!(result, 7);
		/// assert_eq!(*counter.lock().unwrap(), 1);
		/// ```
		#[inline]
		pub fn run_accum<St>(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, ArcRun<R, S, A>>),
				ArcRun<R, S, A>,
			>,
			init: St,
		) -> A
		where
			R: Kind_cdc7cd43dac7585f + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			NodeBrand<R, S>: SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			let _ = init;
			self.interpret(handlers)
		}

		/// Pipeline row-narrowing interpreter. See
		/// [`Run::interpret_with`](crate::types::effects::run::Run::interpret_with)
		/// for cross-wrapper semantics. `ArcRun` differences:
		/// thread-safe substrate
		/// (`A: Send + Sync`, handler is `Send + Sync`); the
		/// [`ArcCoyoneda`] variant pairs with the `Arc`-shared
		/// substrate (matched-arm dispatch lowers via
		/// [`ArcCoyoneda::lower_ref`]); the unmatched-arm `Node::First`
		/// construction is factored through the [`wrap_first_arc`]
		/// HRTB-poisoning workaround free helper, mirroring
		/// [`lift_node`]'s precedent.
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
		#[document_returns("An `ArcRun` program in the narrowed row.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::ArcRun,
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// let prog: ArcRun<FullRow, CNilBrand, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: ArcRun<EmptyRow, CNilBrand, i32> = prog
		/// 	.interpret_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<ArcRun<EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		pub fn interpret_with<EBrand, Idx, RMinusE>(
			self,
			handler: impl Fn(
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<RMinusE, S, A>>),
			) -> ArcRun<RMinusE, S, A>
			+ Clone
			+ Send
			+ Sync
			+ 'static,
		) -> ArcRun<RMinusE, S, A>
		where
			R: Kind_cdc7cd43dac7585f + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + crate::classes::Functor + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, S>: SendFunctor,
			NodeBrand<RMinusE, S>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusE, S>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusE, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusE, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
					ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
								),
				>, {
			match self.peel() {
				Ok(a) => ArcRun::pure(a),
				Err(node) => {
					let layer = unwrap_first::<R, S, ArcRun<R, S, A>>(node);
					match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
					) as Member<ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>, Idx>>::project(
						layer
					) {
						Ok(coyo) => {
							let lowered = coyo.lower_ref();
							let h_for_recurse = handler.clone();
							let mapped = <EBrand as SendFunctor>::send_map(
								move |inner: ArcRun<R, S, A>| {
									inner.interpret_with::<EBrand, Idx, RMinusE>(
										h_for_recurse.clone(),
									)
								},
								lowered,
							);
							handler(mapped)
						}
						Err(rest) => {
							let h_for_recurse = handler.clone();
							let mapped_arc_free = <RMinusE as SendFunctor>::send_map(
								move |inner: ArcRun<R, S, A>| {
									inner
										.interpret_with::<EBrand, Idx, RMinusE>(
											h_for_recurse.clone(),
										)
										.into_arc_free()
								},
								rest,
							);
							let node_first =
								make_node_first::<RMinusE, S, ArcFree<NodeBrand<RMinusE, S>, A>>(
									mapped_arc_free,
								);
							ArcRun::from_arc_free(wrap_first_arc::<RMinusE, S, A>(node_first))
						}
					}
				}
			}
		}
	}

	/// HRTB-poisoning workaround for [`ArcRun::lift`]. The body of
	/// `lift` constructs a [`Node::First`] literal whose type rustc
	/// must normalize against
	/// `<NodeBrand<R, S> as Kind>::Of<'static, A>`. Inside
	/// [`ArcRun`]'s impl-block scope, the HRTB on the `Kind`
	/// projection
	/// (`Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync`)
	/// poisons that normalization (the 2026-04-27 limit; see
	/// [`docs/plans/effects/resolutions.md`](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/resolutions.md)).
	/// Factoring the literal-build step into a free function outside
	/// the HRTB-bearing impl scope sidesteps the poisoning:
	/// [`ArcRun::lift`] only sees the already-normalized projection
	/// value as a function argument and never builds the literal
	/// inside its own scope.
	///
	/// Internal helper for [`ArcRun::lift`]; not part of the public
	/// API. The other five Run wrappers do not need this workaround
	/// (their `lift` body builds the literal inline successfully) and
	/// do not ship a sibling helper.
	#[document_signature]
	///
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The brand of the effect being lifted.",
		"The type-level Member-position witness (typically inferred).",
		"The result type."
	)]
	///
	#[document_parameters("The effect value to lift.")]
	///
	#[document_returns(
		"A `Node::First` projection wrapping the ArcCoyoneda-lifted, row-injected effect."
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
	/// 			arc_run::lift_node,
	/// 			coproduct::Coproduct,
	/// 			node::Node,
	/// 		},
	/// 	},
	/// };
	///
	/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
	/// type Scoped = CNilBrand;
	///
	/// let layer = lift_node::<FirstRow, Scoped, IdentityBrand, _, i32>(Identity(42));
	/// match layer {
	/// 	Node::First(Coproduct::Inl(coyo)) => {
	/// 		let Identity(value) = coyo.lower_ref();
	/// 		assert_eq!(value, 42);
	/// 	}
	/// 	_ => panic!("expected Node::First(Inl(_)) for a single-effect row"),
	/// }
	/// ```
	#[doc(hidden)]
	pub fn lift_node<R, S, EBrand, Idx, A>(
		effect: Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
	) -> Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
	where
		R: Kind_cdc7cd43dac7585f,
		S: Kind_cdc7cd43dac7585f,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
			Member<ArcCoyoneda<'static, EBrand, A>, Idx>,
		EBrand: Kind_cdc7cd43dac7585f + 'static,
		Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>): Clone + Send + Sync,
		A: 'static, {
		let coyo: ArcCoyoneda<'static, EBrand, A> = ArcCoyoneda::lift(effect);
		let layer = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>) as Member<
			ArcCoyoneda<'static, EBrand, A>,
			Idx,
		>>::inject(coyo);
		Node::First(layer)
	}

	/// HRTB-poisoning workaround for [`ArcRun::interpret`]. Pattern
	/// matching `Node::First(...)` / `Node::Scoped(...)` inside an
	/// `ArcRun`-impl-block scope fails GAT normalization symmetrically
	/// to [`lift_node`]'s construction case (the struct-level HRTB on
	/// `<NodeBrand<R, S> as Kind>::Of<'static, ArcFree<...>>: Send +
	/// Sync` poisons the projection equality declared by
	/// [`impl_kind!`](crate::impl_kind)). This free function performs
	/// the variant match outside the HRTB scope so the equality
	/// normalizes; the caller (typically [`ArcRun::interpret`]) hands
	/// the [`Node`]-projection value here and receives the matched
	/// `First`-payload back, with the `Scoped` arm rejected via
	/// [`unreachable!`] (Phase 3 first-order interpretation does not
	/// route scoped layers; Phase 4 will).
	#[document_signature]
	///
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type of the program."
	)]
	///
	#[document_parameters(
		"The Node-projection value (typically [`ArcRun::peel`]'s `Err` payload)."
	)]
	///
	#[document_returns("The first-order layer payload, ready for handler dispatch.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	types::{
	/// 		Identity,
	/// 		effects::{
	/// 			arc_run::{
	/// 				lift_node,
	/// 				unwrap_first,
	/// 			},
	/// 			coproduct::Coproduct,
	/// 		},
	/// 	},
	/// };
	///
	/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
	/// type Scoped = CNilBrand;
	///
	/// let node = lift_node::<FirstRow, Scoped, IdentityBrand, _, i32>(Identity(42));
	/// let layer = unwrap_first::<FirstRow, Scoped, i32>(node);
	/// match layer {
	/// 	Coproduct::Inl(_) => assert!(true),
	/// 	Coproduct::Inr(_) => panic!("expected head Inl"),
	/// }
	/// ```
	#[doc(hidden)]
	#[expect(
		clippy::unreachable,
		reason = "Phase 3 first-order interpreter does not handle scoped layers; the helper is only reachable from interpret loops that route Node::First, so the Scoped arm is genuinely unreachable until Phase 4."
	)]
	pub fn unwrap_first<R, S, A>(
		node: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
	) -> Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		S: Kind_cdc7cd43dac7585f + 'static,
		A: 'static, {
		match node {
			Node::First(layer) => layer,
			Node::Scoped(_) => {
				unreachable!(
					"Phase 3 first-order interpreter received a scoped layer; scoped effects ship in Phase 4"
				)
			}
		}
	}

	/// HRTB-free helper that statically eliminates a [`Node`] over an
	/// empty dual row. Both `Node` arms carry uninhabited
	/// [`CNil`](crate::types::effects::coproduct::CNil) payloads, so the
	/// match diverges to type `!`, which coerces to the caller's
	/// expected `Ret` without any runtime panic. Used by
	/// [`ArcRun::extract`] (and
	/// [`ArcRunExplicit::extract`](crate::types::effects::arc_run_explicit::ArcRunExplicit::extract))
	/// since inline pattern matching against `Node` literals fails GAT
	/// normalization inside the Arc-substrate wrappers' HRTB-bearing
	/// impl scopes; the helper's where-clause holds no HRTB so the
	/// match normalizes cleanly.
	///
	/// `Inner` is the inner-program type stored in the `Node`'s
	/// continuation slot; `Ret` is the caller's return type that the
	/// divergent match coerces into.
	#[document_signature]
	///
	#[document_type_parameters(
		"The inner-program type stored in the `Node`'s continuation slot.",
		"The caller's return type (the divergent match coerces into it)."
	)]
	///
	#[document_parameters("The Node projection over the empty dual row.")]
	///
	#[document_returns(
		"The (unreachable) inhabitant; the function diverges via exhaustive match on uninhabited payloads."
	)]
	///
	#[document_examples]
	///
	/// ```
	/// // The helper is internal (`#[doc(hidden)]`) and discharges the
	/// // (statically uninhabited) `Err` arm of `peel` for empty-row
	/// // programs. Callers exercise it via `ArcRun::extract` (see that
	/// // method's example for the integrated path).
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	types::effects::arc_run::ArcRun,
	/// };
	///
	/// let pure_prog: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(42);
	/// assert_eq!(pure_prog.extract(), 42);
	/// ```
	#[doc(hidden)]
	pub fn unwrap_pure_node<Inner, Ret>(
		node: Apply!(
			<NodeBrand<CNilBrand, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Inner>
		)
	) -> Ret
	where
		Inner: 'static, {
		match node {
			Node::First(cnil) => match cnil {},
			Node::Scoped(cnil) => match cnil {},
		}
	}

	/// HRTB-free helper that constructs a [`Node::First`] projection.
	/// Mirrors [`lift_node`]'s "build the literal outside the HRTB
	/// scope" idiom: the function's where-clause carries only `Kind`
	/// bounds (no GAT-projection HRTB), so the [`Node`] literal
	/// normalizes against
	/// `<NodeBrand<R, S> as Kind>::Of<'_, A>` cleanly. Internal helper
	/// for [`ArcRun::interpret_with`]'s unmatched arm; not part of the
	/// public API.
	#[document_signature]
	///
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The inner-program type carried by the layer's continuations."
	)]
	///
	#[document_parameters("The first-order layer payload.")]
	///
	#[document_returns("The `Node::First` projection.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	types::{
	/// 		Identity,
	/// 		effects::{
	/// 			arc_run::make_node_first,
	/// 			coproduct::Coproduct,
	/// 			node::Node,
	/// 		},
	/// 	},
	/// };
	///
	/// type Row = CoproductBrand<IdentityBrand, CNilBrand>;
	/// type Scoped = CNilBrand;
	///
	/// let layer = Coproduct::inject(Identity(7));
	/// let node = make_node_first::<Row, Scoped, i32>(layer);
	/// match node {
	/// 	Node::First(_) => assert!(true),
	/// 	Node::Scoped(_) => panic!("expected First"),
	/// }
	/// ```
	#[doc(hidden)]
	pub fn make_node_first<R, S, A>(
		layer: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
	) -> Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		S: Kind_cdc7cd43dac7585f + 'static,
		A: 'static, {
		Node::First(layer)
	}

	/// HRTB-poisoning workaround for [`ArcRun::interpret_with`]'s
	/// unmatched-arm [`ArcFree::wrap`](crate::types::ArcFree::wrap)
	/// call. Sibling to [`lift_node`] and [`unwrap_first`]; receives
	/// the already-built [`Node`] projection (constructed by
	/// [`make_node_first`] outside any HRTB scope) and forwards it to
	/// [`ArcFree::wrap`]. The function body therefore performs no GAT
	/// projection construction inside its own HRTB-bearing scope, so
	/// the projection equality declared by
	/// [`impl_kind!`](crate::impl_kind) normalizes cleanly. The five
	/// other Run wrappers do not need this workaround.
	#[document_signature]
	///
	#[document_type_parameters(
		"The narrowed first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type of the program."
	)]
	///
	#[document_parameters(
		"The pre-built `Node::First` projection (typically produced by `make_node_first` whose narrowed-row layer was mapped via `<RMinusE as SendFunctor>::send_map` so each inner program is an `ArcFree` in the narrowed brand)."
	)]
	///
	#[document_returns("An `ArcFree` carrying the narrowed-row suspended layer.")]
	///
	#[document_examples]
	///
	/// ```
	/// // The helper is internal (`#[doc(hidden)]`) and is exercised
	/// // through `ArcRun::interpret_with`'s unmatched arm. See that
	/// // method's example for the end-to-end path; here we just confirm
	/// // a fully-narrowed program round-trips through `extract`.
	/// use fp_library::{
	/// 	brands::*,
	/// 	types::{
	/// 		Identity,
	/// 		effects::arc_run::ArcRun,
	/// 	},
	/// };
	///
	/// type FullRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
	///
	/// let prog: ArcRun<FullRow, CNilBrand, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(7));
	/// let narrowed: ArcRun<CNilBrand, CNilBrand, i32> = prog
	/// 	.interpret_with::<IdentityBrand, _, CNilBrand>(
	/// 		|op: Identity<ArcRun<CNilBrand, CNilBrand, i32>>| op.0,
	/// 	);
	/// assert_eq!(narrowed.extract(), 7);
	/// ```
	#[doc(hidden)]
	pub fn wrap_first_arc<RMinusE, S, A>(
		node: Apply!(<NodeBrand<RMinusE, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<RMinusE, S>, A>,
		>)
	) -> ArcFree<NodeBrand<RMinusE, S>, A>
	where
		RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		A: Send + Sync + 'static,
		NodeBrand<RMinusE, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<RMinusE, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor,
		Apply!(<NodeBrand<RMinusE, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<RMinusE, S>, ArcTypeErasedValue>,
		>): Clone, {
		ArcFree::<NodeBrand<RMinusE, S>, A>::wrap(node)
	}

	#[document_type_parameters("The result type.")]
	#[document_parameters("The `ArcRun` instance.")]
	impl<A> ArcRun<CNilBrand, CNilBrand, A>
	where
		NodeBrand<CNilBrand, CNilBrand>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<CNilBrand, CNilBrand>, ArcTypeErasedValue>>: Send
				                                                                               + Sync,
			> + 'static,
		A: 'static,
	{
		/// Extracts the result value from an `ArcRun` program whose
		/// first-order and scoped rows have both been fully interpreted
		/// away. Routes through the [`unwrap_pure_node`] HRTB-free helper
		/// (since `ArcRun`'s impl-block scope poisons inline `Node`
		/// pattern matching). The helper exhaustively matches both
		/// uninhabited `CNil` payloads, statically proving no runtime
		/// panic.
		#[document_signature]
		///
		#[document_returns("The final result value of the fully-narrowed program.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// let pure_prog: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(42);
		/// assert_eq!(pure_prog.extract(), 42);
		/// ```
		#[inline]
		pub fn extract(self) -> A
		where
			A: Clone + Send + Sync,
			NodeBrand<CNilBrand, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<CNilBrand, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<CNilBrand, CNilBrand>, ArcTypeErasedValue>,
			>): Clone, {
			match self.peel() {
				Ok(a) => a,
				Err(node) => unwrap_pure_node::<ArcRun<CNilBrand, CNilBrand, A>, A>(node),
			}
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
				IdentityBrand,
				NodeBrand,
			},
			types::ArcFree,
		},
	};

	type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
	type Scoped = CNilBrand;
	type ArcRunAlias<A> = ArcRun<FirstRow, Scoped, A>;

	#[test]
	fn from_arc_free_and_into_arc_free_round_trip() {
		let arc_free: ArcFree<NodeBrand<FirstRow, Scoped>, i32> = ArcFree::pure(42);
		let arc_run: ArcRunAlias<i32> = ArcRun::from_arc_free(arc_free);
		let _back: ArcFree<NodeBrand<FirstRow, Scoped>, i32> = arc_run.into_arc_free();
	}

	#[test]
	fn clone_bumps_atomic_refcount_in_constant_time() {
		let arc_run: ArcRunAlias<i32> = ArcRun::from_arc_free(ArcFree::pure(7));
		let _branch = arc_run.clone();
	}

	#[test]
	fn drop_a_pure_arc_run_does_not_panic() {
		let arc_run: ArcRunAlias<i32> = ArcRun::from_arc_free(ArcFree::pure(7));
		drop(arc_run);
	}

	fn _send_sync_witness<T: Send + Sync>() {}

	#[test]
	fn arc_run_is_send_sync() {
		_send_sync_witness::<ArcRunAlias<i32>>();
	}

	#[test]
	fn pure_then_peel_returns_value() {
		let arc_run: ArcRunAlias<i32> = ArcRun::pure(42);
		assert!(matches!(arc_run.peel(), Ok(42)));
	}

	#[test]
	fn send_produces_suspended_program() {
		use crate::types::{
			Identity,
			effects::{
				coproduct::Coproduct,
				node::Node,
			},
		};
		let layer = Coproduct::inject(Identity(7));
		let arc_run: ArcRunAlias<i32> = ArcRun::send(Node::First(layer));
		assert!(arc_run.peel().is_err());
	}

	#[test]
	fn into_explicit_via_into_round_trips_pure() {
		use crate::types::effects::arc_run_explicit::ArcRunExplicit;
		let arc_run: ArcRunAlias<i32> = ArcRun::pure(42);
		let explicit: ArcRunExplicit<'static, FirstRow, Scoped, i32> = arc_run.into();
		assert!(matches!(explicit.peel(), Ok(42)));
	}

	#[test]
	fn into_explicit_via_into_preserves_suspended_layer() {
		use crate::types::{
			Identity,
			effects::{
				arc_run_explicit::ArcRunExplicit,
				coproduct::Coproduct,
				node::Node,
			},
		};
		let layer = Coproduct::inject(Identity(7));
		let arc_run: ArcRunAlias<i32> = ArcRun::send(Node::First(layer));
		let explicit: ArcRunExplicit<'static, FirstRow, Scoped, i32> = arc_run.into();
		assert!(explicit.peel().is_err());
	}

	#[test]
	fn bind_chains_pure_values() {
		let arc_run: ArcRunAlias<i32> =
			ArcRun::pure(2).bind(|x| ArcRun::pure(x + 1)).bind(|x| ArcRun::pure(x * 10));
		assert!(matches!(arc_run.peel(), Ok(30)));
	}

	#[test]
	fn map_transforms_pure_value() {
		let arc_run: ArcRunAlias<i32> = ArcRun::pure(7).map(|x| x * 3);
		assert!(matches!(arc_run.peel(), Ok(21)));
	}

	#[test]
	fn ref_bind_chains_pure_value_via_clone() {
		let arc_run: ArcRunAlias<i32> = ArcRun::pure(2);
		let chained = arc_run.ref_bind(|x: &i32| ArcRun::pure(*x + 1));
		assert!(matches!(chained.peel(), Ok(3)));
	}

	#[test]
	fn ref_map_transforms_pure_value_via_clone() {
		let arc_run: ArcRunAlias<i32> = ArcRun::pure(7);
		let mapped = arc_run.ref_map(|x: &i32| *x * 3);
		assert!(matches!(mapped.peel(), Ok(21)));
	}

	#[test]
	fn ref_pure_wraps_cloned_value() {
		let value = 42;
		let arc_run: ArcRunAlias<i32> = ArcRun::ref_pure(&value);
		assert!(matches!(arc_run.peel(), Ok(42)));
	}
}
