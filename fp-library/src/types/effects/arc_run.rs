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
//! The construction sugar
//! [`from_arc_free`](ArcRun::from_arc_free) /
//! [`into_arc_free`](ArcRun::into_arc_free) bridges to the underlying
//! [`ArcFree`](crate::types::ArcFree). User-facing operations are
//! exposed as inherent methods.

mod raw_scoped;
mod smart_constructors;

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				CNilBrand,
				NodeBrand,
			},
			classes::{
				MonadRec,
				Pointed,
				RefCountedPointer,
				SendFunctor,
				WrapDrop,
			},
			functions::tail_rec_m,
			kinds::Kind_cdc7cd43dac7585f,
			types::{
				ArcCoyoneda,
				ArcFree,
				arc_free::{
					ArcFreeRawStep,
					ArcTypeErasedValue,
				},
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

	#[cfg(test)]
	pub(crate) use super::raw_scoped::ArcRunScopedContinuation;
	pub(crate) use super::raw_scoped::{
		ArcRunContinuations,
		ArcRunRawScopedContinuation,
		DispatchArcRunRawScopedHandler,
		DispatchArcRunRawScopedHandlers,
		RawArcRunFree,
	};
	pub use super::raw_scoped::{
		ArcRunFirstOrderAccumulator,
		ArcRunFirstOrderPreservingAccumulator,
		ArcRunFirstOrderReplacer,
		ArcRunFirstOrderRewriter,
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
		/// let branch = arc_run.clone();
		/// assert!(matches!(branch.peel(), Ok(42)));
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
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::from_arc_free(ArcFree::pure(7));
		/// assert!(matches!(arc_run.peel(), Ok(7)));
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
		/// let arc_free: ArcFree<NodeBrand<FirstRow, Scoped>, i32> = arc_run.into_arc_free();
		/// assert!(matches!(arc_free.resume(), Ok(7)));
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
		/// constructors emitted by `effects!`, or generic helpers
		/// without the HRTB) constructs the layer
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
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::ArcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		/// type Prog = ArcRun<FirstRow, Scoped, i32>;
		///
		/// let arc_run: Prog = ArcRun::lift::<IdentityBrand, _>(Identity(42));
		/// let result = arc_run.handle(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Prog>| op.0,
		/// 	},
		/// 	scoped_handlers! {},
		/// );
		///
		/// assert_eq!(result, 42);
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
		/// [`limitations-and-workarounds.md`](../../../../docs/limitations-and-workarounds.md)).
		/// The `im_do!` macro's `ref` form desugars to this method.
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
		/// The `im_do!` macro's `ref` form rewrites bare `pure(x)`
		/// calls inside `im_do!(ref ArcRun { ... })` to this method.
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
		/// Interprets this `ArcRun` program by walking each effect via
		/// the matching handler closure in `handlers`, looping until
		/// the program reduces to a [`Pure`](crate::types::ArcFree)
		/// value.
		///
		/// Thread-safe variant of [`Run::handle`](crate::types::effects::run::Run::handle).
		/// Each [`peel`](ArcRun::peel) requires `A: Clone + Send +
		/// Sync` and `ArcFree`-projection `Clone`. Because of the
		/// HRTB poisoning that the `ArcFree` projection induces, the
		/// handler list itself is constructed outside this method's
		/// scope (typically outside the impl block) and passed in.
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
		/// let result = prog.handle(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<ArcRun<FirstRow, Scoped, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		pub fn handle(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, ArcRun<R, S, A>>),
				ArcRun<R, S, A>,
			>,
			scoped_handlers: impl DispatchArcRunRawScopedHandlers<
				R,
				S,
				A,
				Apply!(
					<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawArcRunFree<R, S>>
				),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			>,
		) -> A
		where
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			NodeBrand<R, S>: SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone + Send + Sync, {
			let mut prog = self;
			loop {
				match prog.into_arc_free().into_raw_step() {
					ArcFreeRawStep::Done(a) => return a,
					ArcFreeRawStep::Suspended {
						layer,
						continuations,
					} => match unwrap_node::<R, S, RawArcRunFree<R, S>>(layer) {
						Node::First(layer) => {
							let mapped = <R as SendFunctor>::send_map(
								move |inner: RawArcRunFree<R, S>| {
									ArcRun::from_arc_free(ArcFree::continue_from_erased(
										inner,
										continuations.clone(),
									))
								},
								layer,
							);
							prog = handlers.dispatch(mapped);
						}
						Node::Scoped(layer) => {
							prog = scoped_handlers.dispatch_arc_run_raw_scoped(
								layer,
								continuations,
								&handlers,
							);
						}
					},
				}
			}
		}

		/// Alias for [`handle`](ArcRun::handle), kept for naming
		/// parity with PureScript Run's
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
		/// let result = prog.run(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<ArcRun<FirstRow, Scoped, i32>>| op.0,
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
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, ArcRun<R, S, A>>),
				ArcRun<R, S, A>,
			>,
			scoped_handlers: impl DispatchArcRunRawScopedHandlers<
				R,
				S,
				A,
				Apply!(
					<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RawArcRunFree<R, S>>
				),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			>,
		) -> A
		where
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			NodeBrand<R, S>: SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone + Send + Sync, {
			self.handle(handlers, scoped_handlers)
		}

		/// MonadRec-target interpreter for [`ArcRun`]. Mirrors
		/// [`Run::handle_rec`](crate::types::effects::run::Run::handle_rec);
		/// see that method's docs for the handler shape, loop body,
		/// and stack-safety guarantee. `ArcRun` differences: the
		/// thread-safe substrate (`A: Send + Sync`, the M-wrapped
		/// continuation must be `Send + Sync`); inner program
		/// lifting uses [`SendFunctor::send_map`] rather than
		/// [`Functor::map`](crate::classes::Functor); the
		/// [`Node::First`] pattern match is factored through
		/// [`unwrap_first`] (HRTB-poisoning workaround).
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
		/// let result: Option<i32> = prog.handle_rec::<OptionBrand>(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Option<ArcRun<FirstRow, Scoped, i32>>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, Some(42));
		/// ```
		#[inline]
		pub fn handle_rec<MBrand>(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			> + 'static,
			scoped_handlers: impl DispatchScopedHandlers<
				'static,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			> + 'static,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		where
			MBrand: MonadRec + 'static,
			R: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			A: Clone + Send + Sync,
			NodeBrand<R, S>: SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>):
				Send + Sync, {
			tail_rec_m::<MBrand, ArcRun<R, S, A>, A>(
				move |prog: ArcRun<R, S, A>| match prog.peel() {
					Ok(a) => <MBrand as Pointed>::pure::<ControlFlow<A, ArcRun<R, S, A>>>(
						ControlFlow::Break(a),
					),
					Err(node) => match unwrap_node::<R, S, ArcRun<R, S, A>>(node) {
						Node::First(layer) => {
							let mapped = <R as SendFunctor>::send_map(
								|inner: ArcRun<R, S, A>| {
									<MBrand as Pointed>::pure::<ArcRun<R, S, A>>(inner)
								},
								layer,
							);
							let next = handlers.dispatch(mapped);
							<MBrand as crate::classes::Functor>::map::<
								ArcRun<R, S, A>,
								ControlFlow<A, ArcRun<R, S, A>>,
							>(ControlFlow::Continue, next)
						}
						Node::Scoped(layer) => {
							let mapped = <S as SendFunctor>::send_map(
								|inner: ArcRun<R, S, A>| {
									<MBrand as Pointed>::pure::<ArcRun<R, S, A>>(inner)
								},
								layer,
							);
							let next = scoped_handlers.dispatch_scoped(mapped, &handlers);
							<MBrand as crate::classes::Functor>::map::<
								ArcRun<R, S, A>,
								ControlFlow<A, ArcRun<R, S, A>>,
							>(ControlFlow::Continue, next)
						}
					},
				},
				self,
			)
		}

		/// Alias for [`handle_rec`](ArcRun::handle_rec). See
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
		/// let result: Option<i32> = prog.run_rec::<OptionBrand>(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Option<ArcRun<FirstRow, Scoped, i32>>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, Some(99));
		/// ```
		#[inline]
		pub fn run_rec<MBrand>(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			> + 'static,
			scoped_handlers: impl DispatchScopedHandlers<
				'static,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			> + 'static,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		where
			MBrand: MonadRec + 'static,
			R: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			A: Clone + Send + Sync,
			NodeBrand<R, S>: SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>):
				Send + Sync, {
			self.handle_rec::<MBrand>(handlers, scoped_handlers)
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
		/// Scoped-row-narrowing interpreter: interpret a single scoped
		/// effect `SBrand` out of the scoped row, returning an
		/// `ArcRun` program in the narrowed scoped row `SMinusE`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the scoped effect being interpreted out of the row.",
			"The type-level position witness (typically inferred).",
			"The narrowed scoped row brand."
		)]
		///
		#[document_parameters("The thread-safe handler closure for the targeted scoped effect.")]
		///
		#[document_returns("An `ArcRun` program in the narrowed scoped row `SMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		span::SendSpan,
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, &'static str>, CNilBrand>;
		///
		/// let action: ArcRun<CNilBrand, ScopedRow, i32> = ArcRun::pure(7);
		/// let prog: ArcRun<CNilBrand, ScopedRow, i32> =
		/// 	ArcRun::span::<&'static str, _>("request", action);
		/// let narrowed: ArcRun<CNilBrand, CNilBrand, i32> = prog
		/// 	.handle_scoped_with::<SendSpanBrand<ArcBrand, &'static str>, _, CNilBrand>(
		/// 		|span| match span {
		/// 			SendSpan::Span {
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
				Apply!(<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, SMinusE, A>>),
			) -> ArcRun<R, SMinusE, A>
			+ Send
			+ Sync
			+ 'static,
		) -> ArcRun<R, SMinusE, A>
		where
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			SBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, S>: SendFunctor,
			NodeBrand<R, SMinusE>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, SMinusE>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<R, SMinusE> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, SMinusE>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
					),
					Idx,
					Remainder = Apply!(
									<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
								),
				>, {
			let handler = <ArcBrand as RefCountedPointer>::new(handler);
			self.handle_scoped_with_shared::<SBrand, Idx, SMinusE, _>(handler)
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
		#[document_parameters("The handler wrapped in an `Arc` pointer.")]
		///
		#[document_returns("An `ArcRun` program in the narrowed scoped row `SMinusE`.")]
		///
		#[document_examples(
			skip_call_check,
			reason = "Private recursive helper is not callable from external doctests; the example exercises the public ArcRun::handle_scoped_with entry point that delegates here."
		)]
		///
		/// ```
		/// // Exercised internally by ArcRun::handle_scoped_with.
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		span::SendSpan,
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, &'static str>, CNilBrand>;
		///
		/// let action: ArcRun<CNilBrand, ScopedRow, i32> = ArcRun::pure(7);
		/// let prog: ArcRun<CNilBrand, ScopedRow, i32> =
		/// 	ArcRun::span::<&'static str, _>("request", action);
		/// let narrowed: ArcRun<CNilBrand, CNilBrand, i32> = prog
		/// 	.handle_scoped_with::<SendSpanBrand<ArcBrand, &'static str>, _, CNilBrand>(
		/// 		|span| match span {
		/// 			SendSpan::Span {
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
		fn handle_scoped_with_shared<SBrand, Idx, SMinusE, F>(
			self,
			handler: <ArcBrand as RefCountedPointer>::Of<'static, F>,
		) -> ArcRun<R, SMinusE, A>
		where
			F: Fn(
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							ArcRun<R, SMinusE, A>,
						>
					),
				) -> ArcRun<R, SMinusE, A>
				+ Send
				+ Sync
				+ 'static,
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			SBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, S>: SendFunctor,
			NodeBrand<R, SMinusE>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, SMinusE>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<R, SMinusE> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, SMinusE>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
					),
					Idx,
					Remainder = Apply!(
									<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
								),
				>, {
			match self.peel() {
				Ok(a) => ArcRun::pure(a),
				Err(node) => match unwrap_node::<R, S, ArcRun<R, S, A>>(node) {
					Node::First(layer) => {
						let h_for_recurse = handler.clone();
						let mapped_arc_free = <R as SendFunctor>::send_map(
							move |inner: ArcRun<R, S, A>| {
								inner
									.handle_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
										h_for_recurse.clone(),
									)
									.into_arc_free()
							},
							layer,
						);
						let node_first =
							make_node_first::<R, SMinusE, ArcFree<NodeBrand<R, SMinusE>, A>>(
								mapped_arc_free,
							);
						ArcRun::from_arc_free(wrap_first_arc::<R, SMinusE, A>(node_first))
					}
					Node::Scoped(layer) =>
						match <Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							ArcRun<R, S, A>,
						>) as Member<
							Apply!(
								<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
									'static,
									ArcRun<R, S, A>,
								>
							),
							Idx,
						>>::project(layer)
						{
							Ok(scoped) => {
								let h_for_recurse = handler.clone();
								let mapped = <SBrand as SendFunctor>::send_map(
									move |inner: ArcRun<R, S, A>| {
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
								let mapped_arc_free = <SMinusE as SendFunctor>::send_map(
									move |inner: ArcRun<R, S, A>| {
										inner
											.handle_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
												h_for_recurse.clone(),
											)
											.into_arc_free()
									},
									rest,
								);
								let node_scoped = make_node_scoped::<
									R,
									SMinusE,
									ArcFree<NodeBrand<R, SMinusE>, A>,
								>(mapped_arc_free);
								ArcRun::from_arc_free(wrap_first_arc::<R, SMinusE, A>(node_scoped))
							}
						},
				},
			}
		}

		/// Pipeline row-narrowing interpreter. See
		/// [`Run::handle_with`](crate::types::effects::run::Run::handle_with)
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
		/// 	.handle_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<ArcRun<EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		pub fn handle_with<EBrand, Idx, RMinusE>(
			self,
			handler: impl Fn(
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<RMinusE, S, A>>),
			) -> ArcRun<RMinusE, S, A>
			+ Send
			+ Sync
			+ 'static,
		) -> ArcRun<RMinusE, S, A>
		where
			R: Kind_cdc7cd43dac7585f + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
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
			let handler = <ArcBrand as RefCountedPointer>::new(handler);
			self.handle_with_shared::<EBrand, Idx, RMinusE, _>(handler)
		}

		/// Inner pipeline-narrowing implementation, parameterised
		/// over the concrete handler closure type `F`. The public
		/// [`handle_with`](ArcRun::handle_with) wraps the
		/// user handler in [`Arc<F>`](std::sync::Arc) once at entry
		/// and delegates here; recursive narrowing clones the
		/// [`Arc<F>`](std::sync::Arc) (atomic refcount bump)
		/// instead of cloning the underlying closure, which is
		/// what drops the `Clone` bound from the user-facing API.
		/// Internal recursion goes through the
		/// [`unwrap_first`] / [`make_node_first`] /
		/// [`wrap_first_arc`] HRTB-poisoning workaround helpers.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect being interpreted out of the row.",
			"The type-level position witness.",
			"The narrowed row brand.",
			"The concrete handler closure type."
		)]
		///
		#[document_parameters("The handler wrapped in an `Arc` pointer.")]
		///
		#[document_returns("An `ArcRun` program in the narrowed row `RMinusE`.")]
		///
		#[document_examples(
			skip_call_check,
			reason = "Private recursive helper is not callable from external doctests; the example exercises the public ArcRun::handle_with entry point that delegates here."
		)]
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
		/// // Exercised internally by ArcRun::handle_with.
		/// let prog: ArcRun<FullRow, CNilBrand, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: ArcRun<EmptyRow, CNilBrand, i32> = prog
		/// 	.handle_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<ArcRun<EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		fn handle_with_shared<EBrand, Idx, RMinusE, F>(
			self,
			handler: <ArcBrand as RefCountedPointer>::Of<'static, F>,
		) -> ArcRun<RMinusE, S, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<RMinusE, S, A>>),
				) -> ArcRun<RMinusE, S, A>
				+ Send
				+ Sync
				+ 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
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
				Err(node) => match unwrap_node::<R, S, ArcRun<R, S, A>>(node) {
					Node::First(layer) => {
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
								let mapped_arc_free = <RMinusE as SendFunctor>::send_map(
									move |inner: ArcRun<R, S, A>| {
										inner
											.handle_with_shared::<EBrand, Idx, RMinusE, F>(
												h_for_recurse.clone(),
											)
											.into_arc_free()
									},
									rest,
								);
								let node_first = make_node_first::<
									RMinusE,
									S,
									ArcFree<NodeBrand<RMinusE, S>, A>,
								>(mapped_arc_free);
								ArcRun::from_arc_free(wrap_first_arc::<RMinusE, S, A>(node_first))
							}
						}
					}
					Node::Scoped(layer) => {
						let h_for_recurse = handler.clone();
						let mapped_arc_free = <S as SendFunctor>::send_map(
							move |inner: ArcRun<R, S, A>| {
								inner
									.handle_with_shared::<EBrand, Idx, RMinusE, F>(
										h_for_recurse.clone(),
									)
									.into_arc_free()
							},
							layer,
						);
						let node_scoped =
							make_node_scoped::<RMinusE, S, ArcFree<NodeBrand<RMinusE, S>, A>>(
								mapped_arc_free,
							);
						ArcRun::from_arc_free(wrap_first_arc::<RMinusE, S, A>(node_scoped))
					}
				},
			}
		}

		/// Substrate-level `interpose` primitive driven by a
		/// result-polymorphic replacement protocol.
		///
		/// This is the thread-safe shared-wrapper analogue of
		/// [`Run::interpose_with_replacer`](crate::types::effects::run::Run::interpose_with_replacer).
		/// It lets raw scoped handlers rewrite selected actions at
		/// the action's branch result type before reattaching the saved
		/// continuation queue.
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
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::{
		/// 			ArcRun,
		/// 			ArcRunFirstOrderReplacer,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = ArcRun<Row, CNilBrand, i32>;
		///
		/// struct IdentityPassThrough;
		///
		/// impl ArcRunFirstOrderReplacer<IdentityBrand, Row, CNilBrand> for IdentityPassThrough {
		/// 	fn replace<T: Clone + Send + Sync + 'static>(
		/// 		&self,
		/// 		op: Identity<ArcRun<Row, CNilBrand, T>>,
		/// 	) -> ArcRun<Row, CNilBrand, T> {
		/// 		op.0
		/// 	}
		/// }
		///
		/// let prog: Prog = ArcRun::lift::<IdentityBrand, _>(Identity(7));
		/// let interposed =
		/// 	prog.interpose_with_replacer::<IdentityBrand, _, CNilBrand, _>(IdentityPassThrough);
		/// let result = interposed.handle(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Prog>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 7);
		/// ```
		pub fn interpose_with_replacer<EBrand, Idx, RMinusE, EmbedIndices>(
			self,
			replacement: impl ArcRunFirstOrderReplacer<EBrand, R, S> + 'static,
		) -> ArcRun<R, S, A>
		where
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, S>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
					ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			let replacement = <ArcBrand as RefCountedPointer>::new(replacement);
			self.interpose_with_replacer_shared::<EBrand, Idx, RMinusE, EmbedIndices, _>(
				replacement,
			)
		}

		/// Shared implementation of
		/// [`interpose_with_replacer`](ArcRun::interpose_with_replacer).
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect to replace.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand used while projecting the matched effect.",
			"The HList witness for embedding the narrowed row back into the original row.",
			"The concrete result-polymorphic replacement type."
		)]
		#[document_parameters("The Arc-wrapped replacement value.")]
		#[document_returns(
			"A new program in the same row with all matched-effect dispatches replaced."
		)]
		#[document_examples(
			skip_call_check,
			reason = "Private shared implementation is not callable from external doctests; the example exercises the public ArcRun::interpose_with_replacer entry point that delegates here."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::{
		/// 			ArcRun,
		/// 			ArcRunFirstOrderReplacer,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = ArcRun<Row, CNilBrand, i32>;
		///
		/// struct IdentityPassThrough;
		///
		/// impl ArcRunFirstOrderReplacer<IdentityBrand, Row, CNilBrand> for IdentityPassThrough {
		/// 	fn replace<T: Clone + Send + Sync + 'static>(
		/// 		&self,
		/// 		op: Identity<ArcRun<Row, CNilBrand, T>>,
		/// 	) -> ArcRun<Row, CNilBrand, T> {
		/// 		op.0
		/// 	}
		/// }
		///
		/// let prog: Prog = ArcRun::lift::<IdentityBrand, _>(Identity(42));
		/// let interposed =
		/// 	prog.interpose_with_replacer::<IdentityBrand, _, CNilBrand, _>(IdentityPassThrough);
		/// let result = interposed.handle(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Prog>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		fn interpose_with_replacer_shared<EBrand, Idx, RMinusE, EmbedIndices, P>(
			self,
			replacement: <ArcBrand as RefCountedPointer>::Of<'static, P>,
		) -> ArcRun<R, S, A>
		where
			P: ArcRunFirstOrderReplacer<EBrand, R, S> + 'static,
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, S>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
					ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => ArcRun::pure(a),
				Err(node) => match unwrap_node::<R, S, ArcRun<R, S, A>>(node) {
					Node::First(layer) => {
						match <Apply!(
							<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
						) as Member<ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>, Idx>>::project(
							layer
						) {
							Ok(coyo) => {
								let lowered = coyo.lower_ref();
								let r_for_recurse = replacement.clone();
								let mapped = <EBrand as SendFunctor>::send_map(
									move |inner: ArcRun<R, S, A>| {
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
								let mapped_rest = <RMinusE as SendFunctor>::send_map(
									move |inner: ArcRun<R, S, A>| {
										inner
											.interpose_with_replacer_shared::<EBrand, Idx, RMinusE, EmbedIndices, P>(
												r_for_recurse.clone(),
											)
											.into_arc_free()
									},
									rest,
								);
								let layer_back = mapped_rest.embed();
								let node_first = make_node_first::<R, S, ArcFree<NodeBrand<R, S>, A>>(
									layer_back,
								);
								ArcRun::from_arc_free(wrap_first_arc::<R, S, A>(node_first))
							}
						}
					}
					Node::Scoped(layer) => {
						let r_for_recurse = replacement.clone();
						let mapped_arc_free = <S as SendFunctor>::send_map(
							move |inner: ArcRun<R, S, A>| {
								inner
									.interpose_with_replacer_shared::<EBrand, Idx, RMinusE, EmbedIndices, P>(
										r_for_recurse.clone(),
									)
									.into_arc_free()
							},
							layer,
						);
						let node_scoped =
							make_node_scoped::<R, S, ArcFree<NodeBrand<R, S>, A>>(mapped_arc_free);
						ArcRun::from_arc_free(wrap_first_arc::<R, S, A>(node_scoped))
					}
				},
			}
		}

		/// Same-row first-order rewrite primitive.
		///
		/// Walks this program, projects each first-order dispatch
		/// against `EBrand`, rewrites the lowered effect layer with
		/// `rewriter`, and re-embeds the operation in the original row.
		/// The traversal owns recursive continuation rewriting and
		/// row-preserving re-emission; the rewriter only maps the
		/// matched effect constructor.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect to rewrite.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand used while projecting the matched effect.",
			"The HList witness for embedding the narrowed row back into the original row."
		)]
		#[document_parameters("The same-row first-order operation rewriter.")]
		#[document_returns(
			"A new program in the same row with all matched-effect dispatches rewritten."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::{
		/// 			ArcRun,
		/// 			ArcRunFirstOrderRewriter,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = ArcRun<Row, CNilBrand, i32>;
		///
		/// struct IdentityPreserve;
		///
		/// impl ArcRunFirstOrderRewriter<IdentityBrand, Row, CNilBrand> for IdentityPreserve {
		/// 	fn rewrite<T: Clone + Send + Sync + 'static>(
		/// 		&self,
		/// 		op: Identity<ArcRun<Row, CNilBrand, T>>,
		/// 	) -> Identity<ArcRun<Row, CNilBrand, T>> {
		/// 		op
		/// 	}
		/// }
		///
		/// let prog: Prog = ArcRun::lift::<IdentityBrand, _>(Identity(7));
		/// let rewritten =
		/// 	prog.interpose_with_rewriter::<IdentityBrand, _, CNilBrand, _>(IdentityPreserve);
		/// let result = rewritten.handle(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Prog>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 7);
		/// ```
		pub fn interpose_with_rewriter<EBrand, Idx, RMinusE, EmbedIndices>(
			self,
			rewriter: impl ArcRunFirstOrderRewriter<EBrand, R, S> + 'static,
		) -> ArcRun<R, S, A>
		where
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, S>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
					ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
								),
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, A>,
			>): Member<ArcCoyoneda<'static, EBrand, ArcFree<NodeBrand<R, S>, A>>, Idx>,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			ArcFree<NodeBrand<R, S>, A>: Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			let rewriter = <ArcBrand as RefCountedPointer>::new(rewriter);
			self.interpose_with_rewriter_shared::<EBrand, Idx, RMinusE, EmbedIndices, _>(rewriter)
		}

		/// Shared implementation of
		/// [`interpose_with_rewriter`](ArcRun::interpose_with_rewriter).
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect to rewrite.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand used while projecting the matched effect.",
			"The HList witness for embedding the narrowed row back into the original row.",
			"The concrete result-polymorphic rewriter type."
		)]
		#[document_parameters("The Arc-wrapped rewriter value.")]
		#[document_returns(
			"A new program in the same row with all matched-effect dispatches rewritten."
		)]
		#[document_examples(
			skip_call_check,
			reason = "Private shared implementation is not callable from external doctests; the example exercises the public ArcRun::interpose_with_rewriter entry point that delegates here."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::{
		/// 			ArcRun,
		/// 			ArcRunFirstOrderRewriter,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = ArcRun<Row, CNilBrand, i32>;
		///
		/// struct IdentityPreserve;
		///
		/// impl ArcRunFirstOrderRewriter<IdentityBrand, Row, CNilBrand> for IdentityPreserve {
		/// 	fn rewrite<T: Clone + Send + Sync + 'static>(
		/// 		&self,
		/// 		op: Identity<ArcRun<Row, CNilBrand, T>>,
		/// 	) -> Identity<ArcRun<Row, CNilBrand, T>> {
		/// 		op
		/// 	}
		/// }
		///
		/// let prog: Prog = ArcRun::lift::<IdentityBrand, _>(Identity(42));
		/// let rewritten =
		/// 	prog.interpose_with_rewriter::<IdentityBrand, _, CNilBrand, _>(IdentityPreserve);
		/// let result = rewritten.handle(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Prog>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		fn interpose_with_rewriter_shared<EBrand, Idx, RMinusE, EmbedIndices, P>(
			self,
			rewriter: <ArcBrand as RefCountedPointer>::Of<'static, P>,
		) -> ArcRun<R, S, A>
		where
			P: ArcRunFirstOrderRewriter<EBrand, R, S> + 'static,
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, S>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
					ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
								),
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, A>,
			>): Member<ArcCoyoneda<'static, EBrand, ArcFree<NodeBrand<R, S>, A>>, Idx>,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			ArcFree<NodeBrand<R, S>, A>: Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => ArcRun::pure(a),
				Err(node) => match unwrap_node::<R, S, ArcRun<R, S, A>>(node) {
					Node::First(layer) => {
						match <Apply!(
							<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
						) as Member<ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>, Idx>>::project(
							layer
						) {
							Ok(coyo) => {
								let lowered = coyo.lower_ref();
								let r_for_recurse = rewriter.clone();
								let mapped = <EBrand as SendFunctor>::send_map(
									move |inner: ArcRun<R, S, A>| {
										inner
											.interpose_with_rewriter_shared::<EBrand, Idx, RMinusE, EmbedIndices, P>(
												r_for_recurse.clone(),
											)
									},
									lowered,
								);
								let rewritten = (*rewriter).rewrite(mapped);
								let rewritten_free = <EBrand as SendFunctor>::send_map(
									ArcRun::into_arc_free,
									rewritten,
								);
								let node_first =
									lift_node::<R, S, EBrand, Idx, ArcFree<NodeBrand<R, S>, A>>(
										rewritten_free,
									);
								ArcRun::from_arc_free(wrap_first_arc::<R, S, A>(node_first))
							}
							Err(rest) => {
								let r_for_recurse = rewriter.clone();
								let mapped_rest = <RMinusE as SendFunctor>::send_map(
									move |inner: ArcRun<R, S, A>| {
										inner
											.interpose_with_rewriter_shared::<EBrand, Idx, RMinusE, EmbedIndices, P>(
												r_for_recurse.clone(),
											)
											.into_arc_free()
									},
									rest,
								);
								let layer_back = mapped_rest.embed();
								let node_first = make_node_first::<R, S, ArcFree<NodeBrand<R, S>, A>>(
									layer_back,
								);
								ArcRun::from_arc_free(wrap_first_arc::<R, S, A>(node_first))
							}
						}
					}
					Node::Scoped(layer) => {
						let r_for_recurse = rewriter.clone();
						let mapped_arc_free = <S as SendFunctor>::send_map(
							move |inner: ArcRun<R, S, A>| {
								inner
									.interpose_with_rewriter_shared::<EBrand, Idx, RMinusE, EmbedIndices, P>(
										r_for_recurse.clone(),
									)
									.into_arc_free()
							},
							layer,
						);
						let node_scoped =
							make_node_scoped::<R, S, ArcFree<NodeBrand<R, S>, A>>(mapped_arc_free);
						ArcRun::from_arc_free(wrap_first_arc::<R, S, A>(node_scoped))
					}
				},
			}
		}

		/// Result-changing first-order accumulation primitive.
		///
		/// Walks this selected action, consumes each matching
		/// first-order operation, and returns the selected action value
		/// paired with an explicit accumulator. Non-matching first-order
		/// operations and scoped operations stay in the original row.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect to accumulate.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand used while projecting the matched effect.",
			"The HList witness for embedding the narrowed row back into the original row.",
			"The accumulated value type."
		)]
		#[document_parameters("The first-order accumulation instance.")]
		#[document_returns("A program that returns the action value and accumulated value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::{
		/// 			ArcRun,
		/// 			ArcRunFirstOrderAccumulator,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct CountConsumedIdentity;
		///
		/// impl ArcRunFirstOrderAccumulator<IdentityBrand, Row, CNilBrand, usize> for CountConsumedIdentity {
		/// 	fn empty(&self) -> usize {
		/// 		0
		/// 	}
		///
		/// 	fn accumulate<T: Clone + Send + Sync + 'static>(
		/// 		&self,
		/// 		effect: Identity<ArcRun<Row, CNilBrand, (T, usize)>>,
		/// 	) -> ArcRun<Row, CNilBrand, (T, usize)> {
		/// 		effect.0.map(|(value, count)| (value, count + 1))
		/// 	}
		/// }
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(41));
		/// let accumulated = program.accumulate_with_first_order::<IdentityBrand, _, CNilBrand, _, usize>(
		/// 	CountConsumedIdentity,
		/// );
		/// let handled = accumulated.handle_with::<IdentityBrand, _, CNilBrand>(
		/// 	|op: Identity<ArcRun<CNilBrand, CNilBrand, (i32, usize)>>| op.0,
		/// );
		/// assert_eq!(handled.extract(), (41, 1));
		/// ```
		#[inline]
		#[doc(hidden)]
		pub fn accumulate_with_first_order<EBrand, Idx, RMinusE, EmbedIndices, Acc>(
			self,
			accumulator: impl ArcRunFirstOrderAccumulator<EBrand, R, S, Acc> + 'static,
		) -> ArcRun<R, S, (A, Acc)>
		where
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			Acc: Clone + Send + Sync + 'static,
			NodeBrand<R, S>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
					ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
								),
				>,
			ArcFree<NodeBrand<R, S>, (A, Acc)>: Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, (A, Acc)>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						ArcFree<NodeBrand<R, S>, (A, Acc)>,
					>),
					EmbedIndices,
				>, {
			let accumulator = <ArcBrand as RefCountedPointer>::new(accumulator);
			self.accumulate_with_first_order_shared::<EBrand, Idx, RMinusE, EmbedIndices, Acc, _>(
				accumulator,
			)
		}

		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect to accumulate.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand used while projecting the matched effect.",
			"The HList witness for embedding the narrowed row back into the original row.",
			"The accumulated value type.",
			"The concrete result-polymorphic accumulator type."
		)]
		#[document_parameters("The Arc-wrapped first-order accumulation instance.")]
		#[document_returns("A program that returns the action value and accumulated value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::{
		/// 			ArcRun,
		/// 			ArcRunFirstOrderAccumulator,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct CountConsumedIdentity;
		///
		/// impl ArcRunFirstOrderAccumulator<IdentityBrand, Row, CNilBrand, usize> for CountConsumedIdentity {
		/// 	fn empty(&self) -> usize {
		/// 		0
		/// 	}
		///
		/// 	fn accumulate<T: Clone + Send + Sync + 'static>(
		/// 		&self,
		/// 		effect: Identity<ArcRun<Row, CNilBrand, (T, usize)>>,
		/// 	) -> ArcRun<Row, CNilBrand, (T, usize)> {
		/// 		effect.0.map(|(value, count)| (value, count + 1))
		/// 	}
		/// }
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(41));
		/// let accumulator = std::sync::Arc::new(CountConsumedIdentity);
		/// let accumulated = program
		/// 	.accumulate_with_first_order_shared::<IdentityBrand, _, CNilBrand, _, usize, _>(
		/// 		accumulator,
		/// 	);
		/// let handled = accumulated.handle_with::<IdentityBrand, _, CNilBrand>(
		/// 	|op: Identity<ArcRun<CNilBrand, CNilBrand, (i32, usize)>>| op.0,
		/// );
		/// assert_eq!(handled.extract(), (41, 1));
		/// ```
		#[inline]
		#[doc(hidden)]
		pub fn accumulate_with_first_order_shared<EBrand, Idx, RMinusE, EmbedIndices, Acc, P>(
			self,
			accumulator: <ArcBrand as RefCountedPointer>::Of<'static, P>,
		) -> ArcRun<R, S, (A, Acc)>
		where
			P: ArcRunFirstOrderAccumulator<EBrand, R, S, Acc> + 'static,
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			Acc: Clone + Send + Sync + 'static,
			NodeBrand<R, S>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
					ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
								),
				>,
			ArcFree<NodeBrand<R, S>, (A, Acc)>: Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, (A, Acc)>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						ArcFree<NodeBrand<R, S>, (A, Acc)>,
					>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => ArcRun::pure((a, (*accumulator).empty())),
				Err(node) => match unwrap_node::<R, S, ArcRun<R, S, A>>(node) {
					Node::First(layer) => {
						match <Apply!(
							<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
						) as Member<ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>, Idx>>::project(
							layer
						) {
							Ok(coyo) => {
								let lowered = coyo.lower_ref();
								let a_for_recurse = accumulator.clone();
								let mapped = <EBrand as SendFunctor>::send_map(
									move |inner: ArcRun<R, S, A>| {
										inner
											.accumulate_with_first_order_shared::<EBrand, Idx, RMinusE, EmbedIndices, Acc, P>(
												a_for_recurse.clone(),
											)
									},
									lowered,
								);
								(*accumulator).accumulate(mapped)
							}
							Err(rest) => {
								let a_for_recurse = accumulator.clone();
								let mapped_rest = <RMinusE as SendFunctor>::send_map(
									move |inner: ArcRun<R, S, A>| {
										inner
											.accumulate_with_first_order_shared::<EBrand, Idx, RMinusE, EmbedIndices, Acc, P>(
												a_for_recurse.clone(),
											)
											.into_arc_free()
									},
									rest,
								);
								let layer_back = mapped_rest.embed();
								let node_first =
									make_node_first::<R, S, ArcFree<NodeBrand<R, S>, (A, Acc)>>(
										layer_back,
									);
								ArcRun::from_arc_free(wrap_first_arc::<R, S, (A, Acc)>(node_first))
							}
						}
					}
					Node::Scoped(layer) => {
						let a_for_recurse = accumulator.clone();
						let mapped_arc_free = <S as SendFunctor>::send_map(
							move |inner: ArcRun<R, S, A>| {
								inner
									.accumulate_with_first_order_shared::<EBrand, Idx, RMinusE, EmbedIndices, Acc, P>(
										a_for_recurse.clone(),
									)
									.into_arc_free()
							},
							layer,
						);
						let node_scoped =
							make_node_scoped::<R, S, ArcFree<NodeBrand<R, S>, (A, Acc)>>(
								mapped_arc_free,
							);
						ArcRun::from_arc_free(wrap_first_arc::<R, S, (A, Acc)>(node_scoped))
					}
				},
			}
		}

		/// Result-changing first-order preserving accumulation primitive.
		///
		/// Walks this selected action once, accumulates each matching
		/// first-order operation, and rebuilds each matched operation in
		/// the original first-order row. Non-matching first-order
		/// operations and scoped operations stay in the original row.
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect to accumulate while preserving.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand used while projecting the matched effect.",
			"The HList witness for embedding the narrowed row back into the original row.",
			"The accumulated value type."
		)]
		#[document_parameters("The first-order preserving accumulation instance.")]
		#[document_returns("A program that returns the action value and accumulated value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::{
		/// 			ArcRun,
		/// 			ArcRunFirstOrderPreservingAccumulator,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct CountIdentity;
		///
		/// impl ArcRunFirstOrderPreservingAccumulator<IdentityBrand, Row, CNilBrand, usize>
		/// 	for CountIdentity
		/// {
		/// 	fn empty(&self) -> usize {
		/// 		0
		/// 	}
		///
		/// 	fn accumulate_preserving<T: Clone + Send + Sync + 'static>(
		/// 		&self,
		/// 		effect: Identity<ArcRun<Row, CNilBrand, (T, usize)>>,
		/// 	) -> Identity<ArcRun<Row, CNilBrand, (T, usize)>> {
		/// 		Identity(effect.0.map(|(value, count)| (value, count + 1)))
		/// 	}
		/// }
		///
		/// let program: ArcRun<Row, CNilBrand, i32> =
		/// 	ArcRun::lift::<IdentityBrand, _>(Identity(41));
		/// let preserved = program
		/// 	.accumulate_preserving_with_first_order::<IdentityBrand, _, CNilBrand, _, usize>(
		/// 		CountIdentity,
		/// 	);
		/// let handled = preserved.handle_with::<IdentityBrand, _, CNilBrand>(
		/// 	|op: Identity<ArcRun<CNilBrand, CNilBrand, (i32, usize)>>| op.0,
		/// );
		/// assert_eq!(handled.extract(), (41, 1));
		/// ```
		#[inline]
		#[doc(hidden)]
		pub fn accumulate_preserving_with_first_order<EBrand, Idx, RMinusE, EmbedIndices, Acc>(
			self,
			accumulator: impl ArcRunFirstOrderPreservingAccumulator<EBrand, R, S, Acc> + 'static,
		) -> ArcRun<R, S, (A, Acc)>
		where
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			Acc: Clone + Send + Sync + 'static,
			NodeBrand<R, S>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
					ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
								),
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, (A, Acc)>,
			>): Member<ArcCoyoneda<'static, EBrand, ArcFree<NodeBrand<R, S>, (A, Acc)>>, Idx>,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, (A, Acc)>,
			>): Clone + Send + Sync,
			ArcFree<NodeBrand<R, S>, (A, Acc)>: Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, (A, Acc)>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						ArcFree<NodeBrand<R, S>, (A, Acc)>,
					>),
					EmbedIndices,
				>, {
			let accumulator = <ArcBrand as RefCountedPointer>::new(accumulator);
			self.accumulate_preserving_with_first_order_shared::<
				EBrand,
				Idx,
				RMinusE,
				EmbedIndices,
				Acc,
				_,
			>(accumulator)
		}

		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect to accumulate while preserving.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand used while projecting the matched effect.",
			"The HList witness for embedding the narrowed row back into the original row.",
			"The accumulated value type.",
			"The concrete result-polymorphic preserving accumulator type."
		)]
		#[document_parameters("The Arc-wrapped first-order preserving accumulation instance.")]
		#[document_returns("A program that returns the action value and accumulated value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::{
		/// 			ArcRun,
		/// 			ArcRunFirstOrderPreservingAccumulator,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct CountIdentity;
		///
		/// impl ArcRunFirstOrderPreservingAccumulator<IdentityBrand, Row, CNilBrand, usize>
		/// 	for CountIdentity
		/// {
		/// 	fn empty(&self) -> usize {
		/// 		0
		/// 	}
		///
		/// 	fn accumulate_preserving<T: Clone + Send + Sync + 'static>(
		/// 		&self,
		/// 		effect: Identity<ArcRun<Row, CNilBrand, (T, usize)>>,
		/// 	) -> Identity<ArcRun<Row, CNilBrand, (T, usize)>> {
		/// 		Identity(effect.0.map(|(value, count)| (value, count + 1)))
		/// 	}
		/// }
		///
		/// let program: ArcRun<Row, CNilBrand, i32> =
		/// 	ArcRun::lift::<IdentityBrand, _>(Identity(41));
		/// let accumulator = std::sync::Arc::new(CountIdentity);
		/// let preserved = program.accumulate_preserving_with_first_order_shared::<
		/// 	IdentityBrand,
		/// 	_,
		/// 	CNilBrand,
		/// 	_,
		/// 	usize,
		/// 	_,
		/// >(accumulator);
		/// let handled = preserved.handle_with::<IdentityBrand, _, CNilBrand>(
		/// 	|op: Identity<ArcRun<CNilBrand, CNilBrand, (i32, usize)>>| op.0,
		/// );
		/// assert_eq!(handled.extract(), (41, 1));
		/// ```
		#[inline]
		#[doc(hidden)]
		pub fn accumulate_preserving_with_first_order_shared<
			EBrand,
			Idx,
			RMinusE,
			EmbedIndices,
			Acc,
			P,
		>(
			self,
			accumulator: <ArcBrand as RefCountedPointer>::Of<'static, P>,
		) -> ArcRun<R, S, (A, Acc)>
		where
			P: ArcRunFirstOrderPreservingAccumulator<EBrand, R, S, Acc> + 'static,
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			Acc: Clone + Send + Sync + 'static,
			NodeBrand<R, S>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
					ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
								),
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, (A, Acc)>,
			>): Member<ArcCoyoneda<'static, EBrand, ArcFree<NodeBrand<R, S>, (A, Acc)>>, Idx>,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, (A, Acc)>,
			>): Clone + Send + Sync,
			ArcFree<NodeBrand<R, S>, (A, Acc)>: Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, (A, Acc)>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						ArcFree<NodeBrand<R, S>, (A, Acc)>,
					>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => ArcRun::pure((a, (*accumulator).empty())),
				Err(node) => match unwrap_node::<R, S, ArcRun<R, S, A>>(node) {
					Node::First(layer) => {
						match <Apply!(
							<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
						) as Member<ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>, Idx>>::project(
							layer
						) {
							Ok(coyo) => {
								let lowered = coyo.lower_ref();
								let a_for_recurse = accumulator.clone();
								let mapped = <EBrand as SendFunctor>::send_map(
									move |inner: ArcRun<R, S, A>| {
										inner
											.accumulate_preserving_with_first_order_shared::<
												EBrand,
												Idx,
												RMinusE,
												EmbedIndices,
												Acc,
												P,
											>(a_for_recurse.clone())
									},
									lowered,
								);
								let preserved = (*accumulator).accumulate_preserving(mapped);
								let preserved_free = <EBrand as SendFunctor>::send_map(
									ArcRun::into_arc_free,
									preserved,
								);
								let node_first = lift_node::<
									R,
									S,
									EBrand,
									Idx,
									ArcFree<NodeBrand<R, S>, (A, Acc)>,
								>(preserved_free);
								ArcRun::from_arc_free(wrap_first_arc::<R, S, (A, Acc)>(node_first))
							}
							Err(rest) => {
								let a_for_recurse = accumulator.clone();
								let mapped_rest = <RMinusE as SendFunctor>::send_map(
									move |inner: ArcRun<R, S, A>| {
										inner
											.accumulate_preserving_with_first_order_shared::<
												EBrand,
												Idx,
												RMinusE,
												EmbedIndices,
												Acc,
												P,
											>(a_for_recurse.clone())
											.into_arc_free()
									},
									rest,
								);
								let layer_back = mapped_rest.embed();
								let node_first =
									make_node_first::<R, S, ArcFree<NodeBrand<R, S>, (A, Acc)>>(
										layer_back,
									);
								ArcRun::from_arc_free(wrap_first_arc::<R, S, (A, Acc)>(node_first))
							}
						}
					}
					Node::Scoped(layer) => {
						let a_for_recurse = accumulator.clone();
						let mapped_arc_free = <S as SendFunctor>::send_map(
							move |inner: ArcRun<R, S, A>| {
								inner
									.accumulate_preserving_with_first_order_shared::<
										EBrand,
										Idx,
										RMinusE,
										EmbedIndices,
										Acc,
										P,
									>(a_for_recurse.clone())
									.into_arc_free()
							},
							layer,
						);
						let node_scoped =
							make_node_scoped::<R, S, ArcFree<NodeBrand<R, S>, (A, Acc)>>(
								mapped_arc_free,
							);
						ArcRun::from_arc_free(wrap_first_arc::<R, S, (A, Acc)>(node_scoped))
					}
				},
			}
		}

		/// Substrate-level row-preserving replacement primitive: walk
		/// this `ArcRun` program, projecting each first-order dispatch
		/// against `EBrand`; replace every matched dispatch with the
		/// supplied `replacement` closure (applied to the lowered
		/// effect value), and re-emit non-matching dispatches in the
		/// same row. Direct analog of heftia's `interposeInWith` in
		/// substrate-primitive form, on the thread-safe Arc-shared
		/// substrate.
		///
		/// Unlike [`handle_with`](ArcRun::handle_with),
		/// `interpose` does not narrow the row: the matched arm
		/// produces a continuation in the same `R`, the unmatched arm
		/// walks the `Self::Remainder` (`RMinusE`) layer and embeds it
		/// back into `R` via [`CoproductEmbedder`](crate::types::effects::coproduct::CoproductEmbedder).
		/// This is the building block for scoped-effect handlers
		/// (e.g., `Catch`'s recovery path interposes against the body
		/// program's `Throw` dispatches without narrowing the row).
		///
		/// The user-facing closure is wrapped in an
		/// [`Arc`](std::sync::Arc) once at entry; recursive calls
		/// clone the [`Arc`](std::sync::Arc) (atomic refcount bump)
		/// instead of cloning the underlying closure, which is what
		/// drops the `Clone` bound from the user-facing API.
		///
		/// The thread-safe substrate's bound surface mirrors
		/// [`ArcRun::handle_with`](ArcRun::handle_with):
		/// `Send + Sync + 'static` on the replacement closure;
		/// `A: Clone + Send + Sync`; `EBrand` and `RMinusE` are
		/// [`SendFunctor`]s; the dual-row substrate clone bound
		/// applies to the `R` projection (since the rebuilt program
		/// stays in `R`); recursion routes through the
		/// [`unwrap_first`] / [`make_node_first`] / [`wrap_first_arc`]
		/// HRTB-poisoning workaround helpers.
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
			"The replacement applied to each matched-effect dispatch's lowered effect value (must be `Send + Sync`)."
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
		/// 		ArcCoyonedaBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::ArcRun,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = ArcRun<Row, CNilBrand, i32>;
		///
		/// let prog: Prog = ArcRun::lift::<IdentityBrand, _>(Identity(7));
		/// let interposed =
		/// 	prog.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| ArcRun::pure(99));
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
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			) -> ArcRun<R, S, A>
			+ Send
			+ Sync
			+ 'static,
		) -> ArcRun<R, S, A>
		where
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, S>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
					ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			let replacement = <ArcBrand as RefCountedPointer>::new(replacement);
			self.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, _>(replacement)
		}

		/// Inner shared implementation of [`interpose`](ArcRun::interpose),
		/// parameterised over the concrete replacement closure type
		/// `F`. The public [`interpose`](ArcRun::interpose) wraps the
		/// user-supplied closure in [`Arc<F>`](std::sync::Arc) once at
		/// entry and delegates here; recursive descent clones the
		/// [`Arc<F>`](std::sync::Arc) (atomic refcount bump) instead
		/// of cloning the underlying closure, which is what drops the
		/// `Clone` bound from the user-facing API. Internal recursion
		/// goes through the [`unwrap_first`] / [`make_node_first`] /
		/// [`wrap_first_arc`] HRTB-poisoning workaround helpers.
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
		#[document_parameters("The Arc-wrapped replacement closure.")]
		///
		#[document_returns(
			"A new program in the same row with all matched-effect dispatches replaced."
		)]
		///
		#[document_examples(
			skip_call_check,
			reason = "Private shared implementation is not callable from external doctests; the example exercises the public ArcRun::interpose entry point that delegates here."
		)]
		///
		/// ```
		/// // Exercised internally by ArcRun::interpose.
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcCoyonedaBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run::ArcRun,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = ArcRun<Row, CNilBrand, i32>;
		///
		/// let prog: Prog = ArcRun::lift::<IdentityBrand, _>(Identity(3));
		/// let interposed =
		/// 	prog.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| ArcRun::pure(42));
		/// let result = interposed.handle(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Prog>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 42);
		/// ```
		fn interpose_shared<EBrand, Idx, RMinusE, EmbedIndices, F>(
			self,
			replacement: <ArcBrand as RefCountedPointer>::Of<'static, F>,
		) -> ArcRun<R, S, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
				) -> ArcRun<R, S, A>
				+ Send
				+ Sync
				+ 'static,
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, S>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
					ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => ArcRun::pure(a),
				Err(node) => match unwrap_node::<R, S, ArcRun<R, S, A>>(node) {
					Node::First(layer) => {
						match <Apply!(
							<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
						) as Member<ArcCoyoneda<'static, EBrand, ArcRun<R, S, A>>, Idx>>::project(
							layer
						) {
							Ok(coyo) => {
								let lowered = coyo.lower_ref();
								let r_for_recurse = replacement.clone();
								let mapped = <EBrand as SendFunctor>::send_map(
									move |inner: ArcRun<R, S, A>| {
										inner
											.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
												r_for_recurse.clone(),
											)
									},
									lowered,
								);
								(*replacement)(mapped)
							}
							Err(rest) => {
								let r_for_recurse = replacement.clone();
								let mapped_rest = <RMinusE as SendFunctor>::send_map(
									move |inner: ArcRun<R, S, A>| {
										inner
											.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
												r_for_recurse.clone(),
											)
											.into_arc_free()
									},
									rest,
								);
								let layer_back = mapped_rest.embed();
								let node_first = make_node_first::<R, S, ArcFree<NodeBrand<R, S>, A>>(
									layer_back,
								);
								ArcRun::from_arc_free(wrap_first_arc::<R, S, A>(node_first))
							}
						}
					}
					Node::Scoped(layer) => {
						let r_for_recurse = replacement.clone();
						let mapped_arc_free = <S as SendFunctor>::send_map(
							move |inner: ArcRun<R, S, A>| {
								inner
									.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
										r_for_recurse.clone(),
									)
									.into_arc_free()
							},
							layer,
						);
						let node_scoped =
							make_node_scoped::<R, S, ArcFree<NodeBrand<R, S>, A>>(mapped_arc_free);
						ArcRun::from_arc_free(wrap_first_arc::<R, S, A>(node_scoped))
					}
				},
			}
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The first-order-only `ArcRun` instance.")]
	impl<R, A> ArcRun<R, CNilBrand, A>
	where
		NodeBrand<R, CNilBrand>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: 'static,
	{
		/// Substrate-level matched-effect short-circuit primitive on
		/// the thread-safe Erased Run wrapper: walk this `ArcRun`
		/// program, dispatching non-matched first-order effects through
		/// `fo_handlers` and short-circuiting the moment a
		/// matched-effect (`EBrand`) dispatch is encountered, returning
		/// the matched effect's lowered payload.
		///
		/// Returns `Ok(a)` when the program reduces to a pure value
		/// without firing the matched effect; returns `Err(op)` with
		/// the matched effect's lowered payload otherwise.
		///
		/// `fo_handlers` covers only the non-matched effects; the
		/// program type retains the full row `R`. Recursion routes
		/// through the [`unwrap_first`] HRTB-poisoning workaround
		/// helper (mirroring [`ArcRun::handle_with`]).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the matched effect.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand."
		)]
		///
		#[document_parameters(
			"The handler list covering non-matched first-order effects (must be `Send + Sync`)."
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
		/// 		ArcCoyonedaBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		ExceptBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			arc_run::ArcRun,
		/// 			except::Except,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<
		/// 	ArcCoyonedaBrand<ExceptBrand<String>>,
		/// 	CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>,
		/// >;
		/// type RowMinusExcept = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = ArcRun<Row, CNilBrand, i32>;
		///
		/// let prog: Prog = ArcRun::throw::<String, _>("oops".to_string());
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
				Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, ArcRun<R, CNilBrand, A>>),
				ArcRun<R, CNilBrand, A>,
			>,
		) -> Result<
			A,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, CNilBrand, A>>),
		>
		where
			R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, CNilBrand, A>>):
				Member<
						ArcCoyoneda<'static, EBrand, ArcRun<R, CNilBrand, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, CNilBrand, A>>
									),
					>, {
			let mut prog = self;
			loop {
				match prog.peel() {
					Ok(a) => return Ok(a),
					Err(node) => {
						let layer = unwrap_first::<R, CNilBrand, ArcRun<R, CNilBrand, A>>(node);
						match <Apply!(
							<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, CNilBrand, A>>
						) as Member<ArcCoyoneda<'static, EBrand, ArcRun<R, CNilBrand, A>>, Idx>>::project(
							layer
						) {
							Ok(matched_coyo) => return Err(matched_coyo.lower_ref()),
							Err(rest) => prog = fo_handlers.dispatch(rest),
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
	/// poisons that normalization. Factoring the literal-build step
	/// into a free function outside
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
		A: Send + Sync + 'static, {
		let coyo: ArcCoyoneda<'static, EBrand, A> = ArcCoyoneda::lift(effect);
		let layer = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>) as Member<
			ArcCoyoneda<'static, EBrand, A>,
			Idx,
		>>::inject(coyo);
		Node::First(layer)
	}

	/// HRTB-poisoning workaround for [`ArcRun::handle`]. Pattern
	/// matching `Node::First(...)` / `Node::Scoped(...)` inside an
	/// `ArcRun`-impl-block scope fails GAT normalization symmetrically
	/// to [`lift_node`]'s construction case (the struct-level HRTB on
	/// `<NodeBrand<R, S> as Kind>::Of<'static, ArcFree<...>>: Send +
	/// Sync` poisons the projection equality declared by
	/// [`impl_kind!`](crate::impl_kind)). This free function performs
	/// the variant match outside the HRTB scope so the equality
	/// normalizes; the caller (typically [`ArcRun::handle`]) hands
	/// the [`Node`]-projection value here and receives the matched
	/// `First`-payload back, with the `Scoped` arm rejected via
	/// [`unreachable!`] (the first-order interpreter does not route
	/// scoped layers; future scoped-effect work will).
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
	/// assert!(matches!(layer, Coproduct::Inl(_)));
	/// ```
	#[doc(hidden)]
	#[expect(
		clippy::unreachable,
		reason = "The first-order interpreter does not handle scoped layers; the helper is only reachable from interpret loops that route Node::First, so the Scoped arm is genuinely unreachable until scoped effects land."
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
					"first-order interpreter received a scoped layer; scoped-effect dispatch is not yet implemented"
				)
			}
		}
	}

	/// HRTB-free helper that normalizes a [`NodeBrand`] projection to
	/// the concrete [`Node`] enum so Arc-substrate interpreter loops can
	/// branch over both first-order and scoped layers.
	#[document_signature]
	///
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The inner-program type carried by the layer's continuations."
	)]
	///
	#[document_parameters("The Node projection value.")]
	///
	#[document_returns("The normalized Node enum.")]
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
	/// 				unwrap_node,
	/// 			},
	/// 			node::Node,
	/// 		},
	/// 	},
	/// };
	///
	/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
	/// type Scoped = CNilBrand;
	///
	/// let node = lift_node::<FirstRow, Scoped, IdentityBrand, _, i32>(Identity(42));
	/// assert!(matches!(unwrap_node::<FirstRow, Scoped, i32>(node), Node::First(_)));
	/// ```
	#[doc(hidden)]
	pub fn unwrap_node<R, S, A>(
		node: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
	) -> Node<'static, R, S, A>
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		S: Kind_cdc7cd43dac7585f + 'static,
		A: 'static, {
		node
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
	#[document_examples(
		skip_call_check,
		reason = "The function consumes a statically uninhabited empty-row Node; external doctests cannot construct the argument, so the example exercises the integrated ArcRun::extract path that calls it."
	)]
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
	/// for [`ArcRun::handle_with`]'s unmatched arm; not part of the
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
	/// assert!(matches!(node, Node::First(_)));
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

	/// HRTB-free helper that constructs a [`Node::Scoped`] projection.
	/// Sibling to [`make_node_first`]: the function's where-clause
	/// carries only `Kind` bounds (no GAT-projection HRTB), so the
	/// [`Node`] literal normalizes against
	/// `<NodeBrand<R, S> as Kind>::Of<'_, A>` cleanly. Internal helper
	/// for [`ArcRun::catch`] and other scoped-effect smart
	/// constructors; not part of the public API.
	#[document_signature]
	///
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The inner-program type carried by the layer's continuations."
	)]
	///
	#[document_parameters("The scoped-effect layer payload.")]
	///
	#[document_returns("The `Node::Scoped` projection.")]
	///
	#[document_examples]
	///
	/// ```
	/// // The helper is internal (`#[doc(hidden)]`) and is exercised
	/// // through `ArcRun::catch` (and other scoped smart constructors).
	/// // Here we just verify it returns a `Node::Scoped` literal.
	/// use fp_library::{
	/// 	brands::*,
	/// 	types::effects::{
	/// 		arc_run::make_node_scoped,
	/// 		coproduct::Coproduct,
	/// 		node::Node,
	/// 	},
	/// };
	///
	/// type FirstRow = CNilBrand;
	/// type ScopedRow = CoproductBrand<IdentityBrand, CNilBrand>;
	///
	/// let layer = Coproduct::inject(fp_library::types::Identity(7));
	/// let node = make_node_scoped::<FirstRow, ScopedRow, i32>(layer);
	/// assert!(matches!(node, Node::Scoped(_)));
	/// ```
	#[doc(hidden)]
	pub fn make_node_scoped<R, S, A>(
		layer: Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
	) -> Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		S: Kind_cdc7cd43dac7585f + 'static,
		A: 'static, {
		Node::Scoped(layer)
	}

	/// HRTB-poisoning workaround for [`ArcRun::handle_with`]'s
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
	/// use fp_library::{
	/// 	brands::*,
	/// 	types::{
	/// 		ArcFree,
	/// 		Identity,
	/// 		effects::arc_run::{
	/// 			lift_node,
	/// 			wrap_first_arc,
	/// 		},
	/// 	},
	/// };
	///
	/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
	/// type Scoped = CNilBrand;
	/// type Program = ArcFree<NodeBrand<Row, Scoped>, i32>;
	///
	/// let node = lift_node::<Row, Scoped, IdentityBrand, _, Program>(Identity(ArcFree::pure(7)));
	/// let suspended = wrap_first_arc::<Row, Scoped, i32>(node);
	/// assert!(suspended.resume().is_err());
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
mod tests;
