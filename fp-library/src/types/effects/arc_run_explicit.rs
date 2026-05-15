//! Thread-safe multi-shot Explicit-substrate Run program with `Arc`-shared
//! continuations.
//!
//! `ArcRunExplicit<'a, R, S, A>` is the [`Send`] + [`Sync`] sibling of
//! [`RcRunExplicit`](crate::types::effects::rc_run_explicit::RcRunExplicit)
//! over [`ArcFreeExplicit`](crate::types::ArcFreeExplicit):
//!
//! ```text
//! ArcRunExplicit<'a, R, S, A> = ArcFreeExplicit<'a, NodeBrand<R, S>, A>
//! ```
//!
//! The underlying [`ArcFreeExplicit`](crate::types::ArcFreeExplicit) carries
//! `Arc<dyn Fn + Send + Sync>` continuations rather than `Rc<dyn Fn>`, so
//! programs cross thread boundaries. The whole substrate lives behind an
//! outer [`Arc`](std::sync::Arc), so cloning is O(1) atomic refcount bump.
//!
//! ## When to use which
//!
//! Use [`RunExplicit`](crate::types::effects::run_explicit::RunExplicit)
//! when single-threaded and single-shot. Use
//! [`RcRunExplicit`](crate::types::effects::rc_run_explicit::RcRunExplicit)
//! when multi-shot but single-threaded. Use `ArcRunExplicit` for
//! thread-safe multi-shot.
//!
//! ## Brand-level coverage
//!
//! [`ArcRunExplicitBrand`](crate::brands::ArcRunExplicitBrand) implements
//! [`SendPointed`](crate::classes::SendPointed) only. The
//! [`SendRef`](crate::classes::SendRefFunctor)-family hierarchy is not
//! reachable through brand-level delegation because
//! [`ArcFreeExplicitBrand`](crate::brands::ArcFreeExplicitBrand) does not
//! implement it: auto-derive of `Send + Sync` on
//! [`ArcFreeExplicit`](crate::types::ArcFreeExplicit) requires a
//! per-`A` HRTB on the [`Kind`](crate::kinds) projection that stable
//! Rust's trait method signatures cannot carry. Use the inherent
//! [`bind`](ArcRunExplicit::bind) and [`map`](ArcRunExplicit::map)
//! methods on `ArcRunExplicit` for the by-value monadic surface at
//! concrete-type call sites.

mod boundary;
mod smart_constructors;

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				ArcFreeExplicitBrand,
				ArcRunExplicitBrand,
				CNilBrand,
				NodeBrand,
			},
			classes::{
				Functor,
				MonadRec,
				Pointed,
				RefCountedPointer,
				SendFunctor,
				SendPointed,
				WrapDrop,
			},
			functions::tail_rec_m,
			impl_kind,
			kinds::*,
			types::{
				ArcCoyoneda,
				ArcFree,
				ArcFreeExplicit,
				arc_free::ArcTypeErasedValue,
				effects::{
					arc_run::ArcRun,
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

	pub use super::boundary::ArcRunExplicitBoundary;
	pub(crate) use super::boundary::{
		ArcRunExplicitActionSuppliedScopedContinuation,
		ArcRunExplicitScopedContinuation,
	};

	/// Thread-safe multi-shot Explicit-substrate Run program with
	/// `Arc`-shared continuations: a thin wrapper over
	/// [`ArcFreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::ArcFreeExplicit).
	///
	/// The wrapper exists so user-facing API can be expressed without
	/// leaking the underlying [`ArcFreeExplicit`](crate::types::ArcFreeExplicit)
	/// representation. Cloning is O(1) atomic refcount bump.
	#[document_type_parameters(
		"The lifetime that bounds the payload and the row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	pub struct ArcRunExplicit<'a, R, S, A>(ArcFreeExplicit<'a, NodeBrand<R, S>, A>)
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: 'a;

	impl_kind! {
		impl<R: WrapDrop + SendFunctor + 'static, S: WrapDrop + SendFunctor + 'static>
			for ArcRunExplicitBrand<R, S> {
			type Of<'a, A: 'a>: 'a = ArcRunExplicit<'a, R, S, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRunExplicit` instance to clone.")]
	impl<'a, R, S, A> Clone for ArcRunExplicit<'a, R, S, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: 'a,
	{
		/// Clones the `ArcRunExplicit` by atomic refcount bump on the
		/// inner [`ArcFreeExplicit`](crate::types::ArcFreeExplicit). O(1).
		#[document_signature]
		///
		#[document_returns("A new `ArcRunExplicit` representing an independent branch.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(42));
		/// let branch = run.clone();
		/// assert_eq!(run.into_arc_free_explicit().evaluate(), 42);
		/// assert_eq!(branch.into_arc_free_explicit().evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			ArcRunExplicit(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRunExplicit` instance.")]
	impl<'a, R, S, A: 'a> ArcRunExplicit<'a, R, S, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
	{
		/// Wraps an
		/// [`ArcFreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::ArcFreeExplicit)
		/// as an `ArcRunExplicit<'a, R, S, A>`. Zero-cost.
		#[document_signature]
		///
		#[document_parameters("The underlying `ArcFreeExplicit` computation.")]
		///
		#[document_returns("An `ArcRunExplicit` wrapping `arc_free`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(7));
		/// assert_eq!(run.into_arc_free_explicit().evaluate(), 7);
		/// ```
		#[inline]
		pub fn from_arc_free_explicit(arc_free: ArcFreeExplicit<'a, NodeBrand<R, S>, A>) -> Self {
			ArcRunExplicit(arc_free)
		}

		/// Unwraps an `ArcRunExplicit<'a, R, S, A>` to its underlying
		/// [`ArcFreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::ArcFreeExplicit).
		/// Zero-cost.
		#[document_signature]
		///
		#[document_returns("The underlying `ArcFreeExplicit` computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(7));
		/// let arc_free = run.into_arc_free_explicit();
		/// assert_eq!(arc_free.evaluate(), 7);
		/// ```
		#[inline]
		pub fn into_arc_free_explicit(self) -> ArcFreeExplicit<'a, NodeBrand<R, S>, A> {
			self.0
		}

		/// Wraps a value in a pure `ArcRunExplicit` computation.
		/// Delegates to
		/// [`ArcFreeExplicit::pure`](crate::types::ArcFreeExplicit).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `ArcRunExplicit` computation that produces `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> = ArcRunExplicit::pure(42);
		/// assert_eq!(run.into_arc_free_explicit().evaluate(), 42);
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self {
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(a))
		}

		/// Decomposes this `ArcRunExplicit` computation into one step.
		/// Walks the [`ArcFreeExplicitView`](crate::types::ArcFreeExplicitView)
		/// from the underlying substrate.
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` for a pure result, or `Err(layer)` carrying the next `ArcRunExplicit` step."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> = ArcRunExplicit::pure(7);
		/// assert!(matches!(run.peel(), Ok(7)));
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "Return type encodes Result<A, NodeBrand<R, S>::Of<'a, ArcRunExplicit<'a, R, S, A>>>; the GAT projection is structurally complex but cannot be aliased without losing the projection link the wrapper depends on."
		)]
		pub fn peel(
			self
		) -> Result<
			A,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>),
		>
		where
			A: Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync, {
			match self.0.to_view() {
				crate::types::arc_free_explicit::ArcFreeExplicitView::Pure(a) => Ok(a),
				crate::types::arc_free_explicit::ArcFreeExplicitView::Wrap(node) => {
					let mapped = <NodeBrand<R, S> as SendFunctor>::send_map(
						ArcRunExplicit::from_arc_free_explicit,
						node,
					);
					Err(mapped)
				}
			}
		}

		/// Lifts a [`Node`](crate::types::effects::node::Node) dispatch
		/// layer into the `ArcRunExplicit` program. The `node` argument
		/// is the
		/// [`NodeBrand<R, S>`](crate::brands::NodeBrand)
		/// `Of<'a, A>` projection; `send` wraps it via
		/// [`ArcFreeExplicit::wrap`](crate::types::ArcFreeExplicit)
		/// after promoting each `A` into a pure `ArcFreeExplicit`. The
		/// `Node`-projection signature is symmetric across all six Run
		/// wrappers; see
		/// [`Run::send`](crate::types::effects::run::Run::send) for
		/// the rationale.
		#[document_signature]
		///
		#[document_parameters("The Node dispatch layer carrying the effect operation.")]
		///
		#[document_returns(
			"An `ArcRunExplicit` computation that performs the effect and returns its result."
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
		/// 			arc_run_explicit::ArcRunExplicit,
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
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> = ArcRunExplicit::send(Node::First(layer));
		/// let next = match run.peel() {
		/// 	Err(Node::First(Coproduct::Inl(Identity(n)))) => n,
		/// 	_ => panic!("expected First(Inl(Identity(..))) layer"),
		/// };
		/// assert!(matches!(next.peel(), Ok(7)));
		/// ```
		#[inline]
		pub fn send(
			node: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> Self
		where
			A: Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync, {
			let mapped = <NodeBrand<R, S> as SendFunctor>::send_map(
				|a: A| -> ArcFreeExplicit<'a, NodeBrand<R, S>, A> { ArcFreeExplicit::pure(a) },
				node,
			);
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(mapped))
		}

		/// Lifts a raw effect value into an `ArcRunExplicit` program.
		///
		/// Thread-safe Explicit-substrate analog of
		/// [`Run::lift`](crate::types::effects::run::Run::lift). Same chain
		/// (`ArcCoyoneda::lift` -> `Member::inject` ->
		/// `Node::First` -> [`send`](ArcRunExplicit::send)) but uses
		/// [`ArcCoyoneda`](crate::types::ArcCoyoneda) (the
		/// `Send + Sync` Coyoneda variant) because the bare
		/// [`Coyoneda`](crate::types::Coyoneda)'s `Box<dyn FnOnce>`
		/// continuation is not `Send + Sync` and the `Arc`-substrate
		/// rejects it.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect being lifted.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The effect value to lift. Must be `Clone + Send + Sync`.")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// // The program is suspended at the lifted effect; peel reveals the layer.
		/// assert!(run.peel().is_err());
		/// ```
		#[inline]
		pub fn lift<EBrand, Idx>(
			effect: Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, EBrand, A>, Idx>,
			EBrand: Kind_cdc7cd43dac7585f + 'a,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone + Send + Sync,
			A: Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync, {
			let coyo: ArcCoyoneda<'a, EBrand, A> = ArcCoyoneda::lift(effect);
			let layer = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) as Member<
				ArcCoyoneda<'a, EBrand, A>,
				Idx,
			>>::inject(coyo);
			Self::send(Node::First(layer))
		}

		/// Inherent counterpart to
		/// [`ArcFreeExplicit::map`](crate::types::ArcFreeExplicit) by way of
		/// [`bind`](ArcRunExplicit::bind) and `SendPointed::send_pure` on
		/// the underlying substrate. The trait-bound surface is
		/// reachable through this inherent method only because per-`A`
		/// `Clone` and Send/Sync bounds on the underlying
		/// [`ArcFreeExplicit`](crate::types::ArcFreeExplicit) substrate
		/// cannot be carried by brand-level type-class trait method
		/// signatures.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `ArcRunExplicit` with `f` applied to its result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(10));
		/// let mapped = run.map(|x: i32| x * 3);
		/// assert_eq!(mapped.into_arc_free_explicit().evaluate(), 30);
		/// ```
		pub fn map<B>(
			self,
			f: impl Fn(A) -> B + Send + Sync + 'a,
		) -> ArcRunExplicit<'a, R, S, B>
		where
			A: Clone + Send + Sync,
			B: Send + Sync + 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, B>,
			>): Clone + Send + Sync, {
			ArcRunExplicit::from_arc_free_explicit(
				self.0.bind(move |a| ArcFreeExplicit::pure(f(a))),
			)
		}

		/// Inherent
		/// [`bind`](crate::types::ArcFreeExplicit::bind) over
		/// `ArcRunExplicit`, reachable only via the inherent method
		/// because per-`A` `Clone` and Send/Sync bounds on the
		/// underlying [`ArcFreeExplicit`](crate::types::ArcFreeExplicit)
		/// substrate cannot be carried by brand-level
		/// [`SendSemimonad`](crate::classes::SendSemimonad) trait method
		/// signatures.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to chain after this computation.")]
		///
		#[document_returns("A new `ArcRunExplicit` chaining `f` after this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(2));
		/// let chained =
		/// 	run.bind(|x: i32| ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(x + 1)));
		/// assert_eq!(chained.into_arc_free_explicit().evaluate(), 3);
		/// ```
		pub fn bind<B>(
			self,
			f: impl Fn(A) -> ArcRunExplicit<'a, R, S, B> + Send + Sync + 'a,
		) -> ArcRunExplicit<'a, R, S, B>
		where
			A: Clone + Send + Sync,
			B: Send + Sync + 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, B>,
			>): Clone + Send + Sync, {
			ArcRunExplicit::from_arc_free_explicit(
				self.0.bind(move |a| f(a).into_arc_free_explicit()),
			)
		}

		/// By-reference [`bind`](ArcRunExplicit::bind): chains a
		/// continuation that receives `&A` rather than `A`.
		///
		/// Implemented via `self.clone().bind(move |a| f(&a))`. The
		/// clone is `O(1)` (atomic refcount bump on the inner
		/// substrate); the wrapping closure converts the owned `A`
		/// from the substrate's by-value bind path back into the
		/// `&A` the user-supplied `f` expects.
		///
		/// This is the only by-reference dispatch path available for
		/// `ArcRunExplicit` (the brand-level `SendRefSemimonad` is
		/// permanently unreachable on stable Rust per
		/// [`limitations-and-workarounds.md`](../../../../docs/limitations-and-workarounds.md)).
		/// The `im_do!` macro's `ref` form desugars to this method.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to chain after this computation.")]
		///
		#[document_returns("A new `ArcRunExplicit` chaining `f` after a clone of this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(2));
		/// let chained = run
		/// 	.ref_bind(|x: &i32| ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(*x + 1)));
		/// assert_eq!(chained.into_arc_free_explicit().evaluate(), 3);
		/// ```
		pub fn ref_bind<B>(
			&self,
			f: impl Fn(&A) -> ArcRunExplicit<'a, R, S, B> + Send + Sync + 'a,
		) -> ArcRunExplicit<'a, R, S, B>
		where
			A: Clone + Send + Sync,
			B: Send + Sync + 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, B>,
			>): Clone + Send + Sync, {
			self.clone().bind(move |a| f(&a))
		}

		/// By-reference [`map`](ArcRunExplicit::map): applies a
		/// function that takes `&A` rather than `A`.
		///
		/// Implemented via `self.clone().map(move |a| f(&a))`. The
		/// clone is `O(1)` (atomic refcount bump). See
		/// [`ref_bind`](ArcRunExplicit::ref_bind) for the design
		/// rationale.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply by reference to the result.")]
		///
		#[document_returns(
			"A new `ArcRunExplicit` with `f` applied to a clone of this one's result."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(7));
		/// let mapped = run.ref_map(|x: &i32| *x * 3);
		/// assert_eq!(mapped.into_arc_free_explicit().evaluate(), 21);
		/// ```
		pub fn ref_map<B>(
			&self,
			f: impl Fn(&A) -> B + Send + Sync + 'a,
		) -> ArcRunExplicit<'a, R, S, B>
		where
			A: Clone + Send + Sync,
			B: Send + Sync + 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, B>,
			>): Clone + Send + Sync, {
			self.clone().map(move |a| f(&a))
		}

		/// By-reference [`pure`](ArcRunExplicit::pure): wraps a
		/// cloned value in an `ArcRunExplicit` computation.
		///
		/// Implemented as `ArcRunExplicit::pure(a.clone())`.
		/// Requires `A: Clone + Send + Sync`. Parallel to
		/// brand-level
		/// [`SendRefPointed::send_ref_pure`](crate::classes::SendRefPointed)
		/// for concrete-type call sites; brand-level
		/// `SendRefPointed` on
		/// [`ArcRunExplicitBrand`](crate::brands::ArcRunExplicitBrand)
		/// is unreachable on stable Rust per
		/// [`limitations-and-workarounds.md`](../../../../docs/limitations-and-workarounds.md).
		///
		/// The `im_do!` macro's `ref` form rewrites bare `pure(x)`
		/// calls inside `im_do!(ref ArcRunExplicit { ... })` to this
		/// method.
		#[document_signature]
		///
		#[document_parameters("A reference to the value to wrap.")]
		///
		#[document_returns("An `ArcRunExplicit` computation that produces a clone of `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let value = 42;
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> = ArcRunExplicit::ref_pure(&value);
		/// assert_eq!(run.into_arc_free_explicit().evaluate(), 42);
		/// ```
		#[inline]
		pub fn ref_pure(a: &A) -> Self
		where
			A: Clone + Send + Sync, {
			ArcRunExplicit::pure(a.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRunExplicit` instance.")]
	impl<'a, R, S, A: 'a> ArcRunExplicit<'a, R, S, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
	{
		/// Interprets this `ArcRunExplicit` program by walking each
		/// effect via the matching handler closure in `handlers`.
		/// Multi-shot, lifetime-flexible, thread-safe variant of
		/// [`Run::interpret`](crate::types::effects::run::Run::interpret).
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
		/// 			arc_run_explicit::ArcRunExplicit,
		/// 			handlers::*,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let result = prog.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<ArcRunExplicit<'static, FirstRow, Scoped, i32>>| op.0,
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
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, ArcRunExplicit<'a, R, S, A>>),
				ArcRunExplicit<'a, R, S, A>,
			>,
			scoped_handlers: impl DispatchScopedHandlers<
				'a,
				Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
				ArcRunExplicit<'a, R, S, A>,
			>,
		) -> A
		where
			A: Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync, {
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

		/// Alias for [`interpret`](ArcRunExplicit::interpret), kept for
		/// PureScript Run naming parity.
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
		/// 			arc_run_explicit::ArcRunExplicit,
		/// 			handlers::*,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::lift::<IdentityBrand, _>(Identity(99));
		/// let result = prog.run(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<ArcRunExplicit<'static, FirstRow, Scoped, i32>>| op.0,
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
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, ArcRunExplicit<'a, R, S, A>>),
				ArcRunExplicit<'a, R, S, A>,
			>,
			scoped_handlers: impl DispatchScopedHandlers<
				'a,
				Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
				ArcRunExplicit<'a, R, S, A>,
			>,
		) -> A
		where
			A: Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync, {
			self.interpret(handlers, scoped_handlers)
		}

		/// MonadRec-target interpreter for [`ArcRunExplicit`]. Mirrors
		/// [`Run::interpret_rec`](crate::types::effects::run::Run::interpret_rec);
		/// see that method's docs for the handler shape, loop body,
		/// and stack-safety guarantee. `ArcRunExplicit` differences:
		/// the thread-safe substrate (`A: Send + Sync`, the M-wrapped
		/// continuation must be `Send + Sync`); inner program lifting
		/// uses [`SendFunctor::send_map`]; pattern matches `Node`
		/// literals inline because `Send + Sync` bounds are per-method
		/// (no HRTB-poisoning workaround needed, unlike [`ArcRun`]).
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
		/// 			arc_run_explicit::ArcRunExplicit,
		/// 			handlers::*,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let result: Option<i32> = prog.interpret_rec::<OptionBrand>(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Option<ArcRunExplicit<'static, FirstRow, Scoped, i32>>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, Some(42));
		/// ```
		#[inline]
		pub fn interpret_rec<MBrand>(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
			> + 'a,
			scoped_handlers: impl DispatchScopedHandlers<
				'a,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
			> + 'a,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			MBrand: MonadRec + 'static,
			A: Clone + Send + Sync + 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>):
				Send + Sync, {
			tail_rec_m::<MBrand, ArcRunExplicit<'a, R, S, A>, A>(
				move |prog: ArcRunExplicit<'a, R, S, A>| match prog.peel() {
					Ok(a) =>
						<MBrand as Pointed>::pure::<ControlFlow<A, ArcRunExplicit<'a, R, S, A>>>(
							ControlFlow::Break(a),
						),
					Err(Node::First(layer)) => {
						let mapped = <R as SendFunctor>::send_map(
							|inner: ArcRunExplicit<'a, R, S, A>| {
								<MBrand as Pointed>::pure::<ArcRunExplicit<'a, R, S, A>>(inner)
							},
							layer,
						);
						let next = handlers.dispatch(mapped);
						<MBrand as Functor>::map::<
							ArcRunExplicit<'a, R, S, A>,
							ControlFlow<A, ArcRunExplicit<'a, R, S, A>>,
						>(ControlFlow::Continue, next)
					}
					Err(Node::Scoped(layer)) => {
						let mapped = <S as SendFunctor>::send_map(
							|inner: ArcRunExplicit<'a, R, S, A>| {
								<MBrand as Pointed>::pure::<ArcRunExplicit<'a, R, S, A>>(inner)
							},
							layer,
						);
						let next = scoped_handlers.dispatch_scoped(mapped, &handlers);
						<MBrand as Functor>::map::<
							ArcRunExplicit<'a, R, S, A>,
							ControlFlow<A, ArcRunExplicit<'a, R, S, A>>,
						>(ControlFlow::Continue, next)
					}
				},
				self,
			)
		}

		/// Alias for [`interpret_rec`](ArcRunExplicit::interpret_rec).
		/// See [`Run::run_rec`](crate::types::effects::run::Run::run_rec).
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
		/// 			arc_run_explicit::ArcRunExplicit,
		/// 			handlers::*,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::lift::<IdentityBrand, _>(Identity(99));
		/// let result: Option<i32> = prog.run_rec::<OptionBrand>(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<Option<ArcRunExplicit<'static, FirstRow, Scoped, i32>>>| op.0,
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
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
			> + 'a,
			scoped_handlers: impl DispatchScopedHandlers<
				'a,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
			> + 'a,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			MBrand: MonadRec + 'static,
			A: Clone + Send + Sync + 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>):
				Send + Sync, {
			self.interpret_rec::<MBrand>(handlers, scoped_handlers)
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRunExplicit` instance.")]
	impl<'a, R, S, A: 'a> ArcRunExplicit<'a, R, S, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
	{
		/// Scoped-row-narrowing interpreter: interpret a single scoped
		/// effect `SBrand` out of the scoped row, returning an
		/// `ArcRunExplicit` program in the narrowed scoped row
		/// `SMinusE`.
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
		#[document_returns("An `ArcRunExplicit` program in the narrowed scoped row `SMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ToDynSendFn,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::{
		/// 			arc_run_explicit::ArcRunExplicit,
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 			span::SendSpan,
		/// 		},
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, &'static str>, CNilBrand>;
		///
		/// let action: ArcRunExplicit<'static, CNilBrand, ScopedRow, i32> = ArcRunExplicit::pure(7);
		/// let layer = Coproduct::Inl(SendSpan::Span {
		/// 	tag: "request",
		/// 	action: <ArcBrand as ToDynSendFn>::new(move |_: ()| {
		/// 		action.clone().into_arc_free_explicit()
		/// 	}),
		/// });
		/// let prog: ArcRunExplicit<'static, CNilBrand, ScopedRow, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(Node::Scoped(layer)));
		/// let narrowed: ArcRunExplicit<'static, CNilBrand, CNilBrand, i32> = prog
		/// 	.interpret_scoped_with::<SendSpanBrand<ArcBrand, &'static str>, _, CNilBrand>(|span| {
		/// 		match span {
		/// 			SendSpan::Span {
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
				Apply!(<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, SMinusE, A>>),
			) -> ArcRunExplicit<'a, R, SMinusE, A>
			+ Send
			+ Sync
			+ 'a,
		) -> ArcRunExplicit<'a, R, SMinusE, A>
		where
			A: Clone + Send + Sync,
			SBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, SMinusE> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, SMinusE>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, SMinusE>, A>,
			>): Send + Sync,
			Apply!(<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, SMinusE>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, SMinusE, A>,
			>): Send + Sync,
			Apply!(<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, SMinusE, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Member<
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							ArcRunExplicit<'a, R, S, A>,
						>
					),
					Idx,
					Remainder = Apply!(
									<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, S, A>,
									>
								),
				>, {
			let handler = <ArcBrand as RefCountedPointer>::new(handler);
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
		#[document_parameters("The handler wrapped in an `Arc` pointer.")]
		///
		#[document_returns("An `ArcRunExplicit` program in the narrowed scoped row `SMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// // The public interpret_scoped_with method wraps the handler
		/// // and then uses the same scoped-row narrowing path as this helper.
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ToDynSendFn,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::{
		/// 			arc_run_explicit::ArcRunExplicit,
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 			span::SendSpan,
		/// 		},
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, &'static str>, CNilBrand>;
		///
		/// let action: ArcRunExplicit<'static, CNilBrand, ScopedRow, i32> = ArcRunExplicit::pure(7);
		/// let layer = Coproduct::Inl(SendSpan::Span {
		/// 	tag: "request",
		/// 	action: <ArcBrand as ToDynSendFn>::new(move |_: ()| {
		/// 		action.clone().into_arc_free_explicit()
		/// 	}),
		/// });
		/// let prog: ArcRunExplicit<'static, CNilBrand, ScopedRow, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(Node::Scoped(layer)));
		/// let narrowed: ArcRunExplicit<'static, CNilBrand, CNilBrand, i32> = prog
		/// 	.interpret_scoped_with::<SendSpanBrand<ArcBrand, &'static str>, _, CNilBrand>(|span| {
		/// 		match span {
		/// 			SendSpan::Span {
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
			handler: <ArcBrand as RefCountedPointer>::Of<'a, F>,
		) -> ArcRunExplicit<'a, R, SMinusE, A>
		where
			F: Fn(
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							ArcRunExplicit<'a, R, SMinusE, A>,
						>
					),
				) -> ArcRunExplicit<'a, R, SMinusE, A>
				+ Send
				+ Sync
				+ 'a,
			A: Clone + Send + Sync,
			SBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, SMinusE> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, SMinusE>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, SMinusE>, A>,
			>): Send + Sync,
			Apply!(<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, SMinusE>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, SMinusE, A>,
			>): Send + Sync,
			Apply!(<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, SMinusE, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Member<
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							ArcRunExplicit<'a, R, S, A>,
						>
					),
					Idx,
					Remainder = Apply!(
									<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, S, A>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => ArcRunExplicit::pure(a),
				Err(Node::First(layer)) => {
					let h_for_recurse = handler.clone();
					let mapped_free = <R as SendFunctor>::send_map(
						move |inner: ArcRunExplicit<'a, R, S, A>| {
							inner
								.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
									h_for_recurse.clone(),
								)
								.into_arc_free_explicit()
						},
						layer,
					);
					ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::<
						'a,
						NodeBrand<R, SMinusE>,
						A,
					>::wrap(Node::First(mapped_free)))
				}
				Err(Node::Scoped(layer)) =>
					match <Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						ArcRunExplicit<'a, R, S, A>,
					>) as Member<
						Apply!(
							<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'a,
								ArcRunExplicit<'a, R, S, A>,
							>
						),
						Idx,
					>>::project(layer)
					{
						Ok(scoped) => {
							let h_for_recurse = handler.clone();
							let mapped = <SBrand as SendFunctor>::send_map(
								move |inner: ArcRunExplicit<'a, R, S, A>| {
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
							let mapped_free = <SMinusE as SendFunctor>::send_map(
								move |inner: ArcRunExplicit<'a, R, S, A>| {
									inner
										.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
											h_for_recurse.clone(),
										)
										.into_arc_free_explicit()
								},
								rest,
							);
							ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::<
								'a,
								NodeBrand<R, SMinusE>,
								A,
							>::wrap(Node::Scoped(
								mapped_free,
							)))
						}
					},
			}
		}

		/// Pipeline row-narrowing interpreter. See
		/// [`Run::interpret_with`](crate::types::effects::run::Run::interpret_with)
		/// for cross-wrapper semantics. `ArcRunExplicit` differences:
		/// thread-safe handler (`Send + Sync`); the [`ArcCoyoneda`]
		/// variant pairs with the `Arc`-shared substrate (matched-arm
		/// dispatch lowers via [`ArcCoyoneda::lower_ref`]). Unlike
		/// [`ArcRun`], this wrapper does not need the
		/// `wrap_first_arc` HRTB workaround because
		/// [`ArcRunExplicit`]'s struct definition carries the
		/// `Send + Sync` bounds per-method (not at the struct level),
		/// so `Node` literals normalize inline.
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
		#[document_returns("An `ArcRunExplicit` program in the narrowed row.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FullRow, CNilBrand, i32> =
		/// 	ArcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: ArcRunExplicit<'static, EmptyRow, CNilBrand, i32> = prog
		/// 	.interpret_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<ArcRunExplicit<'static, EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		pub fn interpret_with<EBrand, Idx, RMinusE>(
			self,
			handler: impl Fn(
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, RMinusE, S, A>>),
			) -> ArcRunExplicit<'a, RMinusE, S, A>
			+ Send
			+ Sync
			+ 'a,
		) -> ArcRunExplicit<'a, RMinusE, S, A>
		where
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusE, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusE, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusE, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusE, S>, A>,
			>): Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusE, S, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusE, S, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>):
				Member<
						ArcCoyoneda<'a, EBrand, ArcRunExplicit<'a, R, S, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
									),
					>, {
			let handler = <ArcBrand as RefCountedPointer>::new(handler);
			self.interpret_with_shared::<EBrand, Idx, RMinusE, _>(handler)
		}

		/// Inner pipeline-narrowing implementation, parameterised
		/// over the concrete handler closure type `F`. The public
		/// [`interpret_with`](ArcRunExplicit::interpret_with)
		/// wraps the user handler in [`Arc<F>`](std::sync::Arc)
		/// once at entry and delegates here; recursive narrowing
		/// clones the [`Arc<F>`](std::sync::Arc) (atomic refcount
		/// bump) instead of cloning the underlying closure, which
		/// is what drops the `Clone` bound from the user-facing
		/// API.
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
		#[document_returns("An `ArcRunExplicit` program in the narrowed row `RMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// // Exercised internally by ArcRunExplicit::interpret_with.
		/// let prog: ArcRunExplicit<'static, FullRow, CNilBrand, i32> =
		/// 	ArcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: ArcRunExplicit<'static, EmptyRow, CNilBrand, i32> = prog
		/// 	.interpret_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<ArcRunExplicit<'static, EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		fn interpret_with_shared<EBrand, Idx, RMinusE, F>(
			self,
			handler: <ArcBrand as RefCountedPointer>::Of<'a, F>,
		) -> ArcRunExplicit<'a, RMinusE, S, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, RMinusE, S, A>>),
				) -> ArcRunExplicit<'a, RMinusE, S, A>
				+ Send
				+ Sync
				+ 'a,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusE, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusE, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusE, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusE, S>, A>,
			>): Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusE, S, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusE, S, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>):
				Member<
						ArcCoyoneda<'a, EBrand, ArcRunExplicit<'a, R, S, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
									),
					>, {
			match self.peel() {
				Ok(a) => ArcRunExplicit::pure(a),
				Err(Node::First(layer)) =>
					match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
					) as Member<ArcCoyoneda<'a, EBrand, ArcRunExplicit<'a, R, S, A>>, Idx>>::project(
						layer
					) {
						Ok(coyo) => {
							let lowered = coyo.lower_ref();
							let h_for_recurse = handler.clone();
							let mapped = <EBrand as SendFunctor>::send_map(
								move |inner: ArcRunExplicit<'a, R, S, A>| {
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
							let mapped_free = <RMinusE as SendFunctor>::send_map(
								move |inner: ArcRunExplicit<'a, R, S, A>| {
									inner
										.interpret_with_shared::<EBrand, Idx, RMinusE, F>(
											h_for_recurse.clone(),
										)
										.into_arc_free_explicit()
								},
								rest,
							);
							ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::<
								'a,
								NodeBrand<RMinusE, S>,
								A,
							>::wrap(Node::First(
								mapped_free,
							)))
						}
					},
				Err(Node::Scoped(layer)) => {
					let h_for_recurse = handler.clone();
					let mapped_free = <S as SendFunctor>::send_map(
						move |inner: ArcRunExplicit<'a, R, S, A>| {
							inner
								.interpret_with_shared::<EBrand, Idx, RMinusE, F>(
									h_for_recurse.clone(),
								)
								.into_arc_free_explicit()
						},
						layer,
					);
					ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::<
						'a,
						NodeBrand<RMinusE, S>,
						A,
					>::wrap(Node::Scoped(mapped_free)))
				}
			}
		}

		/// Substrate-level row-preserving replacement primitive: walk
		/// this `ArcRunExplicit` program, projecting each first-order
		/// dispatch against `EBrand`; replace every matched dispatch
		/// with the supplied `replacement` closure (applied to the
		/// lowered effect value), and re-emit non-matching dispatches
		/// in the same row. Direct analog of heftia's
		/// `interposeInWith` in substrate-primitive form, on the
		/// thread-safe explicit-lifetime substrate.
		///
		/// Unlike [`interpret_with`](ArcRunExplicit::interpret_with),
		/// `interpose` does not narrow the row: the matched arm
		/// produces a continuation in the same `R`, the unmatched arm
		/// walks the `Self::Remainder` (`RMinusE`) layer and embeds
		/// it back into `R` via [`CoproductEmbedder`](crate::types::effects::coproduct::CoproductEmbedder).
		/// This is the building block for scoped-effect handlers.
		///
		/// The user-facing closure is wrapped in an
		/// [`Arc`](std::sync::Arc) once at entry; recursive calls
		/// clone the [`Arc`](std::sync::Arc) (atomic refcount bump)
		/// instead of cloning the underlying closure. The closure
		/// carries the same `'a` lifetime as the program (not
		/// `'static`), so it can borrow from external state for the
		/// program's lifetime, while still being thread-safe via the
		/// `Send + Sync` bounds.
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
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = ArcRunExplicit<'static, Row, CNilBrand, i32>;
		///
		/// let prog: Prog = ArcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
		/// let interposed = prog.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| {
		/// 	ArcRunExplicit::pure(99)
		/// });
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
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
			) -> ArcRunExplicit<'a, R, S, A>
			+ Send
			+ Sync
			+ 'a,
		) -> ArcRunExplicit<'a, R, S, A>
		where
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>):
				Member<
						ArcCoyoneda<'a, EBrand, ArcRunExplicit<'a, R, S, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
									),
					>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			let replacement = <ArcBrand as RefCountedPointer>::new(replacement);
			self.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, _>(replacement)
		}

		/// Inner shared implementation of [`interpose`](ArcRunExplicit::interpose),
		/// parameterised over the concrete replacement closure type
		/// `F`. The public [`interpose`](ArcRunExplicit::interpose)
		/// wraps the user-supplied closure in [`Arc<F>`](std::sync::Arc)
		/// once at entry and delegates here; recursive descent clones
		/// the [`Arc<F>`](std::sync::Arc) (atomic refcount bump) instead
		/// of cloning the underlying closure.
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
		#[document_examples]
		///
		/// ```
		/// // Exercised internally by ArcRunExplicit::interpose.
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
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = ArcRunExplicit<'static, Row, CNilBrand, i32>;
		///
		/// let prog: Prog = ArcRunExplicit::lift::<IdentityBrand, _>(Identity(3));
		/// let interposed = prog.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| {
		/// 	ArcRunExplicit::pure(42)
		/// });
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
			replacement: <ArcBrand as RefCountedPointer>::Of<'a, F>,
		) -> ArcRunExplicit<'a, R, S, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
				) -> ArcRunExplicit<'a, R, S, A>
				+ Send
				+ Sync
				+ 'a,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>):
				Member<
						ArcCoyoneda<'a, EBrand, ArcRunExplicit<'a, R, S, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
									),
					>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => ArcRunExplicit::pure(a),
				Err(Node::First(layer)) =>
					match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
					) as Member<ArcCoyoneda<'a, EBrand, ArcRunExplicit<'a, R, S, A>>, Idx>>::project(
						layer
					) {
						Ok(coyo) => {
							let lowered = coyo.lower_ref();
							let r_for_recurse = replacement.clone();
							let mapped = <EBrand as SendFunctor>::send_map(
								move |inner: ArcRunExplicit<'a, R, S, A>| {
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
							let mapped_rest = <RMinusE as SendFunctor>::send_map(
								move |inner: ArcRunExplicit<'a, R, S, A>| {
									inner
										.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
											r_for_recurse.clone(),
										)
										.into_arc_free_explicit()
								},
								rest,
							);
							let layer_back = mapped_rest.embed();
							ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::<
								'a,
								NodeBrand<R, S>,
								A,
							>::wrap(Node::First(
								layer_back,
							)))
						}
					},
				Err(Node::Scoped(layer)) => {
					let r_for_recurse = replacement.clone();
					let mapped_free = <S as SendFunctor>::send_map(
						move |inner: ArcRunExplicit<'a, R, S, A>| {
							inner
								.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
									r_for_recurse.clone(),
								)
								.into_arc_free_explicit()
						},
						layer,
					);
					ArcRunExplicit::from_arc_free_explicit(
						ArcFreeExplicit::<'a, NodeBrand<R, S>, A>::wrap(Node::Scoped(mapped_free)),
					)
				}
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The first-order-only `ArcRunExplicit` instance.")]
	impl<'a, R, A: 'a> ArcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + SendFunctor + 'static,
	{
		/// Substrate-level matched-effect short-circuit primitive on
		/// the thread-safe explicit-lifetime Run wrapper: walk this
		/// `ArcRunExplicit` program, dispatching non-matched first-order
		/// effects through `fo_handlers` and short-circuiting the
		/// moment a matched-effect (`EBrand`) dispatch is encountered,
		/// returning the matched effect's lowered payload.
		///
		/// Returns `Ok(a)` when the program reduces to a pure value
		/// without firing the matched effect; returns `Err(op)` with
		/// the matched effect's lowered payload otherwise.
		///
		/// `fo_handlers` covers only the non-matched effects; the
		/// program type retains the full row `R`. The handler list and
		/// matched effect both carry the program's `'a` lifetime; the
		/// closure is `Send + Sync` for thread safety.
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
		/// 			arc_run_explicit::ArcRunExplicit,
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
		/// type Prog = ArcRunExplicit<'static, Row, CNilBrand, i32>;
		///
		/// let prog: Prog = ArcRunExplicit::throw::<String, _>("oops".to_string());
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
				Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, ArcRunExplicit<'a, R, CNilBrand, A>>),
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>,
		) -> Result<
			A,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>),
		>
		where
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static,
			RMinusE: WrapDrop + SendFunctor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>):
				Member<
						ArcCoyoneda<'a, EBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>
									),
					>, {
			let mut prog = self;
			loop {
				match prog.peel() {
					Ok(a) => return Ok(a),
					Err(Node::First(layer)) => match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>
					) as Member<
						ArcCoyoneda<'a, EBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
						Idx,
					>>::project(layer)
					{
						Ok(matched_coyo) => return Err(matched_coyo.lower_ref()),
						Err(rest) => prog = fo_handlers.dispatch(rest),
					},
					Err(Node::Scoped(cnil)) => match cnil {},
				}
			}
		}
	}

	#[document_type_parameters("The lifetime that bounds the payload.", "The result type.")]
	#[document_parameters("The `ArcRunExplicit` instance.")]
	impl<'a, A: 'a> ArcRunExplicit<'a, CNilBrand, CNilBrand, A> {
		/// Extracts the result value from an `ArcRunExplicit` program
		/// whose first-order and scoped rows have both been fully
		/// interpreted away. Exhaustive `match` over the uninhabited
		/// `CNil` payloads proves no runtime panic, statically. See
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
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// let pure_prog: ArcRunExplicit<'_, CNilBrand, CNilBrand, i32> = ArcRunExplicit::pure(42);
		/// assert_eq!(pure_prog.extract(), 42);
		/// ```
		#[inline]
		pub fn extract(self) -> A
		where
			A: Clone + Send + Sync,
			Apply!(<NodeBrand<CNilBrand, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<CNilBrand, CNilBrand>, A>,
			>): Clone + Send + Sync, {
			match self.peel() {
				Ok(a) => a,
				Err(Node::First(cnil)) => match cnil {},
				Err(Node::Scoped(cnil)) => match cnil {},
			}
		}
	}

	// -- From<ArcRun> for ArcRunExplicit (Erased -> Explicit conversion) --

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, S, A> From<ArcRun<R, S, A>> for ArcRunExplicit<'static, R, S, A>
	where
		R: WrapDrop + Functor + SendFunctor + 'static,
		S: WrapDrop + Functor + SendFunctor + 'static,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + Functor
			+ SendFunctor
			+ 'static,
		A: Clone + Send + Sync + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone,
	{
		/// Converts an [`ArcRun<R, S, A>`](crate::types::effects::arc_run::ArcRun)
		/// into the paired Explicit-substrate form by walking the
		/// underlying [`ArcFree`](crate::types::ArcFree) chain via
		/// [`peel`](ArcRun::peel) and rebuilding each suspended layer
		/// through [`ArcFreeExplicit::wrap`](crate::types::ArcFreeExplicit).
		/// Pure values re-emerge as
		/// [`ArcRunExplicit::pure`](ArcRunExplicit::pure).
		///
		/// `Send + Sync` is preserved: both `ArcRun` and `ArcRunExplicit`
		/// auto-derive thread-safety from their `Arc<dyn Fn + Send + Sync>`
		/// substrates when `A: Send + Sync` and the projection HRTB holds.
		/// O(N) in the chain depth.
		///
		/// The body uses a GAT-poisoning workaround: projection-typed
		/// values come from `peel`'s return and from `Functor::map`'s
		/// output, never from inline `Node::First(...)` literals, so
		/// this composes cleanly under the HRTB-bearing impl-block
		/// scope. See `fp-library/tests/arc_run_normalization_probe.rs`
		/// for the regression test.
		#[document_signature]
		///
		#[document_parameters("The Erased-substrate `ArcRun` to convert.")]
		///
		#[document_returns("An `ArcRunExplicit` carrying the same effects.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::pure(42);
		/// // Both call styles work via the blanket `Into` impl.
		/// let from_style: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::from(arc_run);
		/// assert!(matches!(from_style.peel(), Ok(42)));
		/// let arc_run2: ArcRun<FirstRow, Scoped, i32> = ArcRun::pure(42);
		/// let into_style: ArcRunExplicit<'static, FirstRow, Scoped, i32> = arc_run2.into();
		/// assert!(matches!(into_style.peel(), Ok(42)));
		/// ```
		fn from(arc_run: ArcRun<R, S, A>) -> Self {
			match arc_run.peel() {
				Ok(a) => ArcRunExplicit::pure(a),
				Err(layer) => {
					let inner = <NodeBrand<R, S> as Functor>::map(
						|run: ArcRun<R, S, A>| -> ArcFreeExplicit<'static, NodeBrand<R, S>, A> {
							ArcRunExplicit::from(run).into_arc_free_explicit()
						},
						layer,
					);
					ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(inner))
				}
			}
		}
	}

	// -- Brand-level type class instances --
	//
	// Only `SendPointed` is reachable. `SendFunctor`, `SendSemimonad`,
	// and the `SendRef*` hierarchy delegation paths through
	// `ArcFreeExplicitBrand` are unimplementable for the same reasons
	// they are unimplementable on `ArcFreeExplicitBrand` itself: per-`A`
	// `Clone` bounds on `bind`'s `into_inner_owned` shared-state recovery
	// path, and the `for<'a, A>` HRTB needed to express
	// `Of<'a, ArcFreeExplicit<'a, F, A>>: Send + Sync` at the impl-block
	// level. See `arc_free_explicit.rs` lines 730-745 for the full
	// rationale and `fp-library/docs/limitations-and-workarounds.md` for
	// the broader pattern.

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> SendPointed for ArcRunExplicitBrand<R, S>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
	{
		/// Wraps a value in a pure thread-safe `ArcRunExplicit`
		/// computation by delegating to
		/// [`ArcFreeExplicitBrand`](crate::brands::ArcFreeExplicitBrand)'s
		/// [`SendPointed::send_pure`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the row brands.",
			"The type of the value to wrap. Must be `Send + Sync`."
		)]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `ArcRunExplicit` computation that produces `a`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, _> =
		/// 	<ArcRunExplicitBrand<FirstRow, Scoped> as SendPointed>::send_pure(42);
		/// assert_eq!(run.into_arc_free_explicit().evaluate(), 42);
		/// ```
		fn send_pure<'a, A: Send + Sync + 'a>(
			a: A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			ArcRunExplicit::from_arc_free_explicit(
				<ArcFreeExplicitBrand<NodeBrand<R, S>> as SendPointed>::send_pure(a),
			)
		}
	}

	// -- SendRef hierarchy: inherent-method delegation --
	//
	// `SendRefFunctor` and `SendRefSemimonad` are NOT implementable on
	// `ArcRunExplicitBrand` even via inherent-method delegation. Their
	// trait method signatures lack the per-`A` bounds the underlying
	// `ArcFreeExplicit::bind` cascade requires (`A: Clone`, plus per-`A`
	// `F::Of<'_, ArcFreeExplicit<...>>: Clone + Send + Sync`); these
	// cannot be added at the trait level without `for<T>` HRTBs that
	// stable Rust does not support, and the trait's closure bound
	// (`Send + 'a`) is also weaker than the wrapper's `ref_map` /
	// `ref_bind` requirement (`Send + Sync + 'a`). The user-facing
	// by-reference Send-aware surface is the inherent
	// [`ArcRunExplicit::ref_map`](ArcRunExplicit::ref_map) /
	// [`ref_bind`](ArcRunExplicit::ref_bind) methods on the concrete
	// wrapper. See
	// [`fp-library/docs/limitations-and-workarounds.md`](crate) for the
	// per-method analysis. Only `SendRefPointed` admits inherent-method
	// delegation: its trait signature already carries `A: Clone + Send
	// + Sync` and takes no closure, matching `ArcRunExplicit::ref_pure`'s
	// bounds exactly.

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> crate::classes::SendRefPointed for ArcRunExplicitBrand<R, S>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
	{
		/// Wraps a cloned value in a pure thread-safe `ArcRunExplicit`
		/// computation by delegating to inherent
		/// [`ArcRunExplicit::ref_pure`](ArcRunExplicit::ref_pure).
		///
		/// Only by-reference Send-aware brand-level operation reachable
		/// for [`ArcRunExplicitBrand`](crate::brands::ArcRunExplicitBrand);
		/// see the inline rationale above. The trait's
		/// `A: Clone + Send + Sync` bound matches `ref_pure`'s, and
		/// neither side carries a closure (so the closure-bound
		/// mismatch that blocks `SendRefFunctor` / `SendRefSemimonad`
		/// does not apply).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the row brands.",
			"The type of the value to wrap. Must be `Clone + Send + Sync`."
		)]
		///
		#[document_parameters("A reference to the value to wrap.")]
		///
		#[document_returns("An `ArcRunExplicit` computation that produces a clone of the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let value = 42;
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, _> =
		/// 	<ArcRunExplicitBrand<FirstRow, Scoped> as SendRefPointed>::send_ref_pure(&value);
		/// assert_eq!(run.into_arc_free_explicit().evaluate(), 42);
		/// ```
		fn send_ref_pure<'a, A: Clone + Send + Sync + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			ArcRunExplicit::ref_pure(a)
		}
	}
}

pub use inner::*;

#[cfg(test)]
#[expect(clippy::expect_used, reason = "Tests use panicking operations for brevity and clarity")]
mod tests;
