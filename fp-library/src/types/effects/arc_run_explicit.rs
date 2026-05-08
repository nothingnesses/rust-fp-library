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

#[fp_macros::document_module]
mod inner {
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
					interpreter::DispatchHandlers,
					member::Member,
					node::Node,
				},
			},
		},
		core::ops::ControlFlow,
		fp_macros::*,
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
		"The result type."
	)]
	#[document_parameters("The `ArcRunExplicit` instance.")]
	impl<'a, R, A: 'a> ArcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + SendFunctor + 'static,
	{
		/// Interprets this `ArcRunExplicit` program by walking each
		/// effect via the matching handler closure in `handlers`.
		/// Multi-shot, lifetime-flexible, thread-safe variant of
		/// [`Run::interpret`](crate::types::effects::run::Run::interpret).
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
		/// let result = prog.interpret(handlers! {
		/// 	IdentityBrand: |op: Identity<ArcRunExplicit<'static, FirstRow, Scoped, i32>>| op.0,
		/// });
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		pub fn interpret(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, ArcRunExplicit<'a, R, CNilBrand, A>>),
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>,
		) -> A
		where
			A: Clone + Send + Sync,
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
			>): Send + Sync, {
			let mut prog = self;
			loop {
				match prog.peel() {
					Ok(a) => return a,
					Err(Node::First(layer)) => prog = handlers.dispatch(layer),
					Err(Node::Scoped(cnil)) => match cnil {},
				}
			}
		}

		/// Alias for [`interpret`](ArcRunExplicit::interpret), kept for
		/// PureScript Run naming parity.
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
		/// let result = prog.run(handlers! {
		/// 	IdentityBrand: |op: Identity<ArcRunExplicit<'static, FirstRow, Scoped, i32>>| op.0,
		/// });
		/// assert_eq!(result, 99);
		/// ```
		#[inline]
		pub fn run(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, ArcRunExplicit<'a, R, CNilBrand, A>>),
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>,
		) -> A
		where
			A: Clone + Send + Sync,
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
			>): Send + Sync, {
			self.interpret(handlers)
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
		#[document_parameters("The handler list.")]
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
		/// let result: Option<i32> = prog.interpret_rec::<OptionBrand>(handlers! {
		/// 	IdentityBrand: |op: Identity<Option<ArcRunExplicit<'static, FirstRow, Scoped, i32>>>| op.0,
		/// });
		/// assert_eq!(result, Some(42));
		/// ```
		#[inline]
		pub fn interpret_rec<MBrand>(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>),
			> + 'a,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			MBrand: MonadRec + 'static,
			A: Clone + Send + Sync + 'a,
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
			Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>):
				Send + Sync, {
			tail_rec_m::<MBrand, ArcRunExplicit<'a, R, CNilBrand, A>, A>(
				move |prog: ArcRunExplicit<'a, R, CNilBrand, A>| match prog.peel() {
					Ok(a) => <MBrand as Pointed>::pure::<
						ControlFlow<A, ArcRunExplicit<'a, R, CNilBrand, A>>,
					>(ControlFlow::Break(a)),
					Err(Node::First(layer)) => {
						let mapped = <R as SendFunctor>::send_map(
							|inner: ArcRunExplicit<'a, R, CNilBrand, A>| {
								<MBrand as Pointed>::pure::<ArcRunExplicit<'a, R, CNilBrand, A>>(
									inner,
								)
							},
							layer,
						);
						let next = handlers.dispatch(mapped);
						<MBrand as Functor>::map::<
							ArcRunExplicit<'a, R, CNilBrand, A>,
							ControlFlow<A, ArcRunExplicit<'a, R, CNilBrand, A>>,
						>(ControlFlow::Continue, next)
					}
					Err(Node::Scoped(cnil)) => match cnil {},
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
		#[document_parameters("The handler list.")]
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
		/// let result: Option<i32> = prog.run_rec::<OptionBrand>(handlers! {
		/// 	IdentityBrand: |op: Identity<Option<ArcRunExplicit<'static, FirstRow, Scoped, i32>>>| op.0,
		/// });
		/// assert_eq!(result, Some(99));
		/// ```
		#[inline]
		pub fn run_rec<MBrand>(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>),
			> + 'a,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			MBrand: MonadRec + 'static,
			A: Clone + Send + Sync + 'a,
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
			Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>):
				Send + Sync, {
			self.interpret_rec::<MBrand>(handlers)
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
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, RMinusE, CNilBrand, A>>),
			) -> ArcRunExplicit<'a, RMinusE, CNilBrand, A>
			+ Send
			+ Sync
			+ 'a,
		) -> ArcRunExplicit<'a, RMinusE, CNilBrand, A>
		where
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + Functor + SendFunctor + 'static,
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
			Apply!(<NodeBrand<RMinusE, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusE, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusE, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusE, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusE, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusE, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>):
				Member<
						ArcCoyoneda<'a, EBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>
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
		) -> ArcRunExplicit<'a, RMinusE, CNilBrand, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, RMinusE, CNilBrand, A>>),
				) -> ArcRunExplicit<'a, RMinusE, CNilBrand, A>
				+ Send
				+ Sync
				+ 'a,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + Functor + SendFunctor + 'static,
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
			Apply!(<NodeBrand<RMinusE, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusE, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusE, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusE, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusE, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusE, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>):
				Member<
						ArcCoyoneda<'a, EBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>
									),
					>, {
			match self.peel() {
				Ok(a) => ArcRunExplicit::pure(a),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>
				) as Member<
					ArcCoyoneda<'a, EBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let lowered = coyo.lower_ref();
						let h_for_recurse = handler.clone();
						let mapped = <EBrand as SendFunctor>::send_map(
							move |inner: ArcRunExplicit<'a, R, CNilBrand, A>| {
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
							move |inner: ArcRunExplicit<'a, R, CNilBrand, A>| {
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
							NodeBrand<RMinusE, CNilBrand>,
							A,
						>::wrap(Node::First(
							mapped_free,
						)))
					}
				},
				Err(Node::Scoped(cnil)) => match cnil {},
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
		/// let result = interposed.interpret(handlers! {
		/// 	IdentityBrand: |op: Identity<Prog>| op.0,
		/// });
		/// assert_eq!(result, 99);
		/// ```
		pub fn interpose<EBrand, Idx, RMinusE, EmbedIndices>(
			self,
			replacement: impl Fn(
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>),
			) -> ArcRunExplicit<'a, R, CNilBrand, A>
			+ Send
			+ Sync
			+ 'a,
		) -> ArcRunExplicit<'a, R, CNilBrand, A>
		where
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + Functor + SendFunctor + 'static,
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
					>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
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
		/// let result = interposed.interpret(handlers! {
		/// 	IdentityBrand: |op: Identity<Prog>| op.0,
		/// });
		/// assert_eq!(result, 42);
		/// ```
		fn interpose_shared<EBrand, Idx, RMinusE, EmbedIndices, F>(
			self,
			replacement: <ArcBrand as RefCountedPointer>::Of<'a, F>,
		) -> ArcRunExplicit<'a, R, CNilBrand, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>),
				) -> ArcRunExplicit<'a, R, CNilBrand, A>
				+ Send
				+ Sync
				+ 'a,
			A: Clone + Send + Sync,
			EBrand: Kind_cdc7cd43dac7585f + Functor + SendFunctor + 'static,
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
					>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
				>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => ArcRunExplicit::pure(a),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ArcRunExplicit<'a, R, CNilBrand, A>>
				) as Member<
					ArcCoyoneda<'a, EBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let lowered = coyo.lower_ref();
						let r_for_recurse = replacement.clone();
						let mapped = <EBrand as SendFunctor>::send_map(
							move |inner: ArcRunExplicit<'a, R, CNilBrand, A>| {
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
							move |inner: ArcRunExplicit<'a, R, CNilBrand, A>| {
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
							NodeBrand<R, CNilBrand>,
							A,
						>::wrap(Node::First(
							layer_back,
						)))
					}
				},
				Err(Node::Scoped(cnil)) => match cnil {},
			}
		}

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
			EBrand: Kind_cdc7cd43dac7585f + Functor + SendFunctor + 'static,
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

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The state type (also the program's result type for `get`)."
	)]
	impl<'a, R, ScopedRow, A: 'a> ArcRunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
	{
		/// Lifts a `Get` state effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::get`](crate::types::effects::run::Run::get); see
		/// that method for cross-wrapper semantics. Differences for
		/// `ArcRunExplicit`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendStateBrand`](crate::brands::SendStateBrand) (rather
		/// than `StateBrand`) so the continuation projection is
		/// structurally `Send + Sync`, which the `SendFunctor`
		/// algebra requires across thread boundaries.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Get` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		state::SendState,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::get();
		/// // The program is suspended at the Get effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn get<Idx>() -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Member<
					ArcCoyoneda<'a, crate::brands::SendStateBrand<crate::brands::ArcBrand, A>, A>,
					Idx,
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::state::SendState<'a, crate::brands::ArcBrand, A, A> =
				crate::types::effects::state::SendState::Get(
					<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|s: A| s),
				);
			Self::lift::<crate::brands::SendStateBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}

		/// Lifts an `Ask` reader effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::ask`](crate::types::effects::run::Run::ask); see
		/// that method for cross-wrapper semantics. Differences for
		/// `ArcRunExplicit`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendReaderBrand`](crate::brands::SendReaderBrand) (rather
		/// than `ReaderBrand`) so the continuation projection is
		/// structurally `Send + Sync`.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Ask` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		reader::SendReader,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::ask();
		/// // The program is suspended at the Ask effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn ask<Idx>() -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Member<
					ArcCoyoneda<'a, crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>, A>,
					Idx,
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::reader::SendReader<
				'a,
				crate::brands::ArcBrand,
				A,
				A,
			> = crate::types::effects::reader::SendReader::Ask(
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|e: A| e),
			);
			Self::lift::<crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}

		/// Lifts a `Throw` except effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::throw`](crate::types::effects::run::Run::throw);
		/// see that method for cross-wrapper semantics. The same
		/// [`ExceptBrand`](crate::brands::ExceptBrand) serves all six
		/// wrappers because [`Except`](crate::types::effects::except::Except)
		/// has no `dyn Fn` continuation; no parallel `SendExceptBrand`
		/// is needed. Requires `ErrorType: Send + Sync` so the lifted
		/// layer participates in the Arc substrate's thread-safety
		/// cascade.
		#[document_signature]
		///
		#[document_type_parameters(
			"The error type carried by `ExceptBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The error value to throw.")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Throw` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		except::Except,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::throw::<&'static str, _>("oops");
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn throw<ErrorType: Clone + Send + Sync + 'static, Idx>(e: ErrorType) -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, crate::brands::ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::except::Except<'a, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}

		/// Lifts a scoped `Catch` effect into the `ArcRunExplicit`
		/// program: run `action`, and if it throws an `E`, invoke
		/// `handler` with the error to produce a recovery program.
		/// Mirrors [`ArcRun::catch`](crate::types::effects::arc_run::ArcRun::catch);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRunExplicit`: the action and recovery handler are stored
		/// as `Arc<dyn Fn(...) -> _ + Send + Sync>` thunks (multi-shot,
		/// thread-safe) over the explicit `'a` lifetime.
		#[document_signature]
		///
		#[document_type_parameters(
			"The error type recovered from.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The protected action program (must be `Clone + Send + Sync` for the multi-shot Arc-thunk).",
			"The recovery handler invoked on a thrown error (multi-shot via [`Fn`], thread-safe)."
		)]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the scoped `Catch` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendCatchBrand<ArcBrand, &'static str>, CNilBrand>;
		///
		/// let action: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(42);
		/// let prog: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	ArcRunExplicit::catch::<&'static str, _>(action, |_e| ArcRunExplicit::pure(0));
		/// // The program is suspended at the Catch scoped layer; peel
		/// // returns Err carrying a `Node::Scoped(...)` projection.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn catch<E: Send + Sync + 'a, Idx>(
			action: ArcRunExplicit<'a, R, ScopedRow, A>,
			handler: impl Fn(E) -> ArcRunExplicit<'a, R, ScopedRow, A> + Send + Sync + 'a,
		) -> Self
		where
			A: Clone + Send + Sync + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::catch::SendCatch<
						'a,
						ArcBrand,
						E,
						ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let catch: crate::types::effects::catch::SendCatch<
				'a,
				ArcBrand,
				E,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::catch::SendCatch::Catch {
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					action.clone().into_arc_free_explicit()
				}),
				handler: <ArcBrand as crate::classes::ToDynSendFn>::new(move |e: E| {
					handler(e).into_arc_free_explicit()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::catch::SendCatch<
					'a,
					ArcBrand,
					E,
					ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(catch);
			let node = Node::Scoped(layer);
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(node))
		}

		/// Lifts a scoped `Local` effect into the `ArcRunExplicit`
		/// program: run `action` under an environment value transformed
		/// by `modify`. Mirrors
		/// [`ArcRun::local`](crate::types::effects::arc_run::ArcRun::local);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRunExplicit`: the modify closure and action are stored as
		/// `Arc<dyn Fn(...) -> _ + Send + Sync>` thunks (multi-shot,
		/// thread-safe) over the explicit `'a` lifetime.
		#[document_signature]
		///
		#[document_type_parameters(
			"The environment type transformed by `modify`.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The environment-transform closure (multi-shot via [`Fn`], thread-safe).",
			"The protected action program (must be `Clone + Send + Sync` for the multi-shot Arc-thunk)."
		)]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the scoped `Local` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendLocalBrand<ArcBrand, i32>, CNilBrand>;
		///
		/// let action: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(42);
		/// let prog: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	ArcRunExplicit::local::<i32, _>(|e: i32| e + 1, action);
		/// // The program is suspended at the Local scoped layer; peel
		/// // returns Err carrying a `Node::Scoped(...)` projection.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn local<E: Send + Sync + 'a, Idx>(
			modify: impl Fn(E) -> E + Send + Sync + 'a,
			action: ArcRunExplicit<'a, R, ScopedRow, A>,
		) -> Self
		where
			A: Clone + Send + Sync + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::local::SendLocal<
						'a,
						ArcBrand,
						E,
						ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let local: crate::types::effects::local::SendLocal<
				'a,
				ArcBrand,
				E,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::local::SendLocal::Local {
				modify: <ArcBrand as crate::classes::ToDynSendFn>::new(move |e: E| modify(e)),
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					action.clone().into_arc_free_explicit()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::local::SendLocal<
					'a,
					ArcBrand,
					E,
					ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(local);
			let node = Node::Scoped(layer);
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(node))
		}

		/// Lifts a [`SendRefLocal`](crate::types::effects::ref_local::SendRefLocal)
		/// scoped environment-modification effect (Ref flavour) into the
		/// `ArcRunExplicit` program. Mirrors
		/// [`Run::ref_local`](crate::types::effects::run::Run::ref_local)
		/// for the thread-safe Arc explicit-lifetime substrate. The
		/// `modify` closure (`Fn(&E) -> E + Send + Sync + 'a`) borrows
		/// the inherited environment value rather than consuming it.
		#[document_signature]
		///
		#[document_type_parameters(
			"The environment type borrowed by `modify` (`Send + Sync`).",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The environment-transform closure (borrows the inherited environment value, `Send + Sync`).",
			"The protected action program."
		)]
		///
		#[document_returns(
			"An `ArcRunExplicit` program suspended at the scoped `Local` effect (Ref flavour)."
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
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendRefLocalBrand<ArcBrand, i32>, CNilBrand>;
		///
		/// let action: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(42);
		/// let prog: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	ArcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 1, action);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn ref_local<E: Send + Sync + 'a, Idx>(
			modify: impl Fn(&E) -> E + Send + Sync + 'a,
			action: ArcRunExplicit<'a, R, ScopedRow, A>,
		) -> Self
		where
			A: Clone + Send + Sync + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::ref_local::SendRefLocal<
						'a,
						ArcBrand,
						E,
						ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let local: crate::types::effects::ref_local::SendRefLocal<
				'a,
				ArcBrand,
				E,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::ref_local::SendRefLocal::Local {
				modify: <ArcBrand as crate::classes::ToDynSendFn>::ref_new(move |e: &E| modify(e)),
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					action.clone().into_arc_free_explicit()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::ref_local::SendRefLocal<
					'a,
					ArcBrand,
					E,
					ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(local);
			let node = Node::Scoped(layer);
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(node))
		}

		/// Lifts a scoped `Span` effect into the `ArcRunExplicit`
		/// program: run `action` under instrumentation identified by
		/// `tag`. Mirrors
		/// [`ArcRun::span`](crate::types::effects::arc_run::ArcRun::span);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRunExplicit`: the action is stored as an
		/// `Arc<dyn Fn(()) -> _ + Send + Sync>` thunk over the explicit
		/// `'a` lifetime, and the by-value tag must be cloneable and
		/// thread-safe.
		#[document_signature]
		///
		#[document_type_parameters(
			"The instrumentation tag type (`Clone + Send + Sync`).",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The instrumentation tag.",
			"The protected action program (must be `Clone + Send + Sync` for the multi-shot Arc-thunk)."
		)]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the scoped `Span` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, &'static str>, CNilBrand>;
		///
		/// let action: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(42);
		/// let prog: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	ArcRunExplicit::span::<&'static str, _>("request", action);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn span<Tag: Clone + Send + Sync + 'a, Idx>(
			tag: Tag,
			action: ArcRunExplicit<'a, R, ScopedRow, A>,
		) -> Self
		where
			A: Clone + Send + Sync + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::span::SendSpan<
						'a,
						ArcBrand,
						Tag,
						ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let span: crate::types::effects::span::SendSpan<
				'a,
				ArcBrand,
				Tag,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::span::SendSpan::Span {
				tag,
				action: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					action.clone().into_arc_free_explicit()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::span::SendSpan<
					'a,
					ArcBrand,
					Tag,
					ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(span);
			let node = Node::Scoped(layer);
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(node))
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The body's result type (`Send + Sync`)."
	)]
	impl<'a, R, ScopedRow, B> ArcRunExplicit<'a, R, ScopedRow, B>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
		B: Send + Sync + 'a,
	{
		/// Lifts a [`SendRefBracketExplicit`](crate::types::effects::ref_bracket::SendRefBracketExplicit)
		/// scoped resource-management effect into the `ArcRunExplicit`
		/// program. This is the Ref flavour of
		/// [`ArcRunExplicit::bracket`]: `acquire` produces the resource,
		/// while `body` and `release` both receive independent `Arc<A>`
		/// clones.
		#[document_signature]
		///
		#[document_type_parameters(
			"The resource type produced by acquire (`Send + Sync`).",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The acquire program (produces the resource).",
			"The body closure (`Send + Sync`; receives the resource as `Arc<A>` and returns the body result program).",
			"The release closure (`Send + Sync`; receives the resource as `Arc<A>` and returns a unit program)."
		)]
		///
		#[document_returns(
			"An `ArcRunExplicit` program suspended at the scoped `RefBracket` effect."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CNilBrand;
		/// let prog: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(7);
		/// assert!(matches!(prog.peel(), Ok(7)));
		/// ```
		///
		/// ```ignore
		/// // Sketch: real scoped rows use the marker-struct workaround
		/// // documented on ArcRunExplicit::bracket.
		/// let acquire: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(7);
		/// let prog: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	ArcRunExplicit::<'static, FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 		acquire,
		/// 		|resource: std::sync::Arc<i32>| ArcRunExplicit::pure(*resource + 35),
		/// 		|_resource: std::sync::Arc<i32>| ArcRunExplicit::pure(()),
		/// 	);
		/// ```
		#[inline]
		pub fn ref_bracket<A: Send + Sync + 'a, Idx>(
			acquire: ArcRunExplicit<'a, R, ScopedRow, A>,
			body: impl Fn(
				<ArcBrand as crate::classes::SendRefCountedPointer>::Of<'a, A>,
			) -> ArcRunExplicit<'a, R, ScopedRow, B>
			+ Send
			+ Sync
			+ 'a,
			release: impl Fn(
				<ArcBrand as crate::classes::SendRefCountedPointer>::Of<'a, A>,
			) -> ArcRunExplicit<'a, R, ScopedRow, ()>
			+ Send
			+ Sync
			+ 'a,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, B>,
			>): Member<
					crate::types::effects::ref_bracket::SendRefBracketExplicit<
						'a,
						ArcBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, B>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, B>,
			>): Clone + Send + Sync, {
			let bracket: crate::types::effects::ref_bracket::SendRefBracketExplicit<
				'a,
				ArcBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::ref_bracket::SendRefBracketExplicit::Bracket {
				acquire: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					acquire.clone().into_arc_free_explicit()
				}),
				body: <ArcBrand as crate::classes::ToDynSendFn>::new(
					move |a: <ArcBrand as crate::classes::SendRefCountedPointer>::Of<'a, A>| {
						body(a).into_arc_free_explicit()
					},
				),
				release: <ArcBrand as crate::classes::ToDynSendFn>::new(
					move |a: <ArcBrand as crate::classes::SendRefCountedPointer>::Of<'a, A>| {
						release(a).into_arc_free_explicit()
					},
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, B>,
			>) as Member<
				crate::types::effects::ref_bracket::SendRefBracketExplicit<
					'a,
					ArcBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			let node = Node::Scoped(layer);
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(node))
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The resource type produced by acquire (`Send + Sync`).",
		"The body's result type (`Send + Sync`)."
	)]
	impl<'a, R, ScopedRow, A, B> ArcRunExplicit<'a, R, ScopedRow, (A, B)>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
		A: Send + Sync + 'a,
		B: Send + Sync + 'a,
	{
		/// Lifts a [`SendBracketExplicit`](crate::types::effects::bracket::SendBracketExplicit)
		/// scoped resource-management effect into the `ArcRunExplicit`
		/// program. Mirrors [`Run::bracket`](crate::types::effects::run::Run::bracket)
		/// for the thread-safe Arc explicit-lifetime substrate.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_parameters(
			"The acquire program (produces the resource).",
			"The body closure (`Send + Sync`; receives the resource as `Arc<A>` and returns a paired program).",
			"The release closure (`Send + Sync`; receives the resource as `Arc<A>` and returns a unit program)."
		)]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the scoped `Bracket` effect.")]
		///
		#[document_examples]
		///
		/// User-facing scoped rows containing
		/// [`SendBracketExplicitBrand`](crate::brands::SendBracketExplicitBrand)
		/// cannot be defined as type aliases. Use the marker-struct
		/// workaround validated by the
		/// [B18 POC](../../../../tests/poc_bracket_marker_row.rs).
		///
		/// ```
		/// #![recursion_limit = "512"]
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::*,
		/// 	classes::{
		/// 		SendFunctor,
		/// 		WrapDrop,
		/// 	},
		/// 	impl_kind,
		/// 	kinds::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type UnderlyingRow = CoproductBrand<
		/// 	SendBracketExplicitBrand<ArcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>,
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
		/// impl SendFunctor for ScopedRow {
		/// 	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		/// 		f: impl Fn(A) -> B + Send + Sync + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as SendFunctor>::send_map(f, fa)
		/// 	}
		/// }
		///
		/// type FirstRow = CNilBrand;
		///
		/// let acquire: ArcRunExplicit<'static, FirstRow, ScopedRow, i32> = ArcRunExplicit::pure(7);
		/// let prog: ArcRunExplicit<'static, FirstRow, ScopedRow, (i32, i32)> =
		/// 	ArcRunExplicit::<'static, FirstRow, ScopedRow, (i32, i32)>::bracket::<_>(
		/// 		acquire,
		/// 		|resource: std::sync::Arc<i32>| ArcRunExplicit::pure((*resource, 42)),
		/// 		|_resource: std::sync::Arc<i32>| ArcRunExplicit::pure(()),
		/// 	);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn bracket<Idx>(
			acquire: ArcRunExplicit<'a, R, ScopedRow, A>,
			body: impl Fn(
				<ArcBrand as crate::classes::Pointer>::Of<'a, A>,
			) -> ArcRunExplicit<'a, R, ScopedRow, (A, B)>
			+ Send
			+ Sync
			+ 'a,
			release: impl Fn(
				<ArcBrand as crate::classes::Pointer>::Of<'a, A>,
			) -> ArcRunExplicit<'a, R, ScopedRow, ()>
			+ Send
			+ Sync
			+ 'a,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, (A, B)>,
			>): Member<
					crate::types::effects::bracket::SendBracketExplicit<
						'a,
						ArcBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				> + Send
				+ Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, (A, B)>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, (A, B)>,
			>): Clone + Send + Sync, {
			let bracket: crate::types::effects::bracket::SendBracketExplicit<
				'a,
				ArcBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::bracket::SendBracketExplicit::Bracket {
				acquire: <ArcBrand as crate::classes::ToDynSendFn>::new(move |_: ()| {
					acquire.clone().into_arc_free_explicit()
				}),
				body: <ArcBrand as crate::classes::ToDynSendFn>::new(
					move |a: <ArcBrand as crate::classes::Pointer>::Of<'a, A>| {
						body(a).into_arc_free_explicit()
					},
				),
				release: <ArcBrand as crate::classes::ToDynSendFn>::new(
					move |a: <ArcBrand as crate::classes::Pointer>::Of<'a, A>| {
						release(a).into_arc_free_explicit()
					},
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, (A, B)>,
			>) as Member<
				crate::types::effects::bracket::SendBracketExplicit<
					'a,
					ArcBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			let node = Node::Scoped(layer);
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(node))
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	impl<'a, R, ScopedRow> ArcRunExplicit<'a, R, ScopedRow, ()>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
	{
		/// Lifts a `Put` state effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::put`](crate::types::effects::run::Run::put); see
		/// that method for cross-wrapper semantics. Threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendStateBrand`](crate::brands::SendStateBrand); see
		/// [`get`](ArcRunExplicit::get) for the design rationale.
		#[document_signature]
		///
		#[document_type_parameters(
			"The state type carried by `SendStateBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The new state value to write.")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Put` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		state::SendState,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, ()> = ArcRunExplicit::put::<i32, _>(42);
		/// // The program is suspended at the Put effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn put<StateType: Clone + Send + Sync + 'static, Idx>(s: StateType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>): Member<
					ArcCoyoneda<
						'a,
						crate::brands::SendStateBrand<crate::brands::ArcBrand, StateType>,
						(),
					>,
					Idx,
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::state::SendState<
				'a,
				crate::brands::ArcBrand,
				StateType,
				(),
			> = crate::types::effects::state::SendState::Put(
				s,
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|_: ()| ()),
			);
			Self::lift::<crate::brands::SendStateBrand<crate::brands::ArcBrand, StateType>, Idx>(
				effect,
			)
		}

		/// Lifts a `Tell` writer effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::tell`](crate::types::effects::run::Run::tell); see
		/// that method for cross-wrapper semantics. The same
		/// [`WriterBrand`](crate::brands::WriterBrand) serves all six
		/// wrappers because [`Writer`](crate::types::effects::writer::Writer)
		/// has no `dyn Fn` continuation; no parallel `SendWriterBrand`
		/// is needed. Requires `LogType: Send + Sync` so the lifted
		/// layer participates in the Arc substrate's thread-safety
		/// cascade.
		#[document_signature]
		///
		#[document_type_parameters(
			"The log type carried by `WriterBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The log value to emit.")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Tell` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		writer::Writer,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, ()> =
		/// 	ArcRunExplicit::tell::<&'static str, _>("logged");
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn tell<LogType: Clone + Send + Sync + 'static, Idx>(log: LogType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<ArcCoyoneda<'a, crate::brands::WriterBrand<LogType>, ()>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::writer::Writer<'a, LogType, ()> =
				crate::types::effects::writer::Writer::Tell(log, (), core::marker::PhantomData);
			Self::lift::<crate::brands::WriterBrand<LogType>, Idx>(effect)
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	impl<'a, R, ScopedRow> ArcRunExplicit<'a, R, ScopedRow, bool>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
	{
		/// Lifts an `Alt` choose effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`RcRun::choose`](crate::types::effects::rc_run::RcRun::choose);
		/// see that method for cross-wrapper semantics. Differences
		/// for `ArcRunExplicit`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendChooseBrand`](crate::brands::SendChooseBrand) (rather
		/// than `ChooseBrand`) so the continuation projection is
		/// structurally `Send + Sync`.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Alt` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		choose::SendChoose,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, bool> = ArcRunExplicit::choose();
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>): Member<
					ArcCoyoneda<'a, crate::brands::SendChooseBrand<crate::brands::ArcBrand>, bool>,
					Idx,
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, bool>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, bool>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, bool>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::choose::SendChoose<
				'a,
				crate::brands::ArcBrand,
				bool,
			> = crate::types::effects::choose::SendChoose::Alt(
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|b: bool| b),
			);
			Self::lift::<crate::brands::SendChooseBrand<crate::brands::ArcBrand>, Idx>(effect)
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
mod tests {
	use {
		super::*,
		crate::{
			brands::{
				ArcRunExplicitBrand,
				CNilBrand,
				CoproductBrand,
				IdentityBrand,
			},
			classes::SendPointed,
			types::ArcFreeExplicit,
		},
	};

	type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
	type Scoped = CNilBrand;
	type RunAlias<'a, A> = ArcRunExplicit<'a, FirstRow, Scoped, A>;

	#[test]
	fn from_and_into_round_trip() {
		let arc_free: ArcFreeExplicit<'_, _, i32> = ArcFreeExplicit::pure(42);
		let run: RunAlias<'_, i32> = ArcRunExplicit::from_arc_free_explicit(arc_free);
		let _back = run.into_arc_free_explicit();
	}

	#[test]
	fn clone_branches_are_cheap() {
		let run: RunAlias<'_, _> = ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(7));
		let _branch = run.clone();
	}

	#[test]
	fn brand_send_pure_evaluates() {
		let run: RunAlias<'_, _> =
			<ArcRunExplicitBrand<FirstRow, Scoped> as SendPointed>::send_pure(7);
		assert_eq!(run.into_arc_free_explicit().evaluate(), 7);
	}

	#[test]
	fn inherent_map_evaluates() {
		let run: RunAlias<'_, _> =
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(10));
		let mapped = run.map(|x: i32| x * 3);
		assert_eq!(mapped.into_arc_free_explicit().evaluate(), 30);
	}

	#[test]
	fn inherent_bind_evaluates() {
		let run: RunAlias<'_, _> = ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(2));
		let chained =
			run.bind(|x: i32| ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(x + 5)));
		assert_eq!(chained.into_arc_free_explicit().evaluate(), 7);
	}

	#[test]
	fn cross_thread_via_spawn() {
		let run: RunAlias<'static, _> =
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(10));
		let mapped = run.map(|x: i32| x * 4);
		let handle = std::thread::spawn(move || mapped.into_arc_free_explicit().evaluate());
		assert_eq!(handle.join().expect("thread panicked"), 40);
	}

	#[test]
	fn arc_run_explicit_is_send_sync() {
		fn assert_send_sync<T: Send + Sync>(_: &T) {}
		let run = ArcRunExplicit::<'_, FirstRow, Scoped, i32>::from_arc_free_explicit(
			ArcFreeExplicit::pure(7),
		);
		assert_send_sync(&run);
	}

	#[test]
	fn non_static_payload() {
		let s = String::from("hello");
		let r: &str = &s;
		let run: ArcRunExplicit<'_, FirstRow, Scoped, &str> =
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(r));
		assert_eq!(run.into_arc_free_explicit().evaluate(), "hello");
	}

	#[test]
	fn pure_then_peel_returns_value() {
		let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> = ArcRunExplicit::pure(42);
		assert!(matches!(run.peel(), Ok(42)));
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
		let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
			ArcRunExplicit::send(Node::First(layer));
		assert!(run.peel().is_err());
	}

	#[test]
	fn from_erased_round_trips_pure() {
		use crate::types::effects::arc_run::ArcRun;
		let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::pure(42);
		let explicit: ArcRunExplicit<'static, FirstRow, Scoped, i32> =
			ArcRunExplicit::from(arc_run);
		assert!(matches!(explicit.peel(), Ok(42)));
	}

	#[test]
	fn from_erased_preserves_suspended_layer() {
		use crate::types::{
			Identity,
			effects::{
				arc_run::ArcRun,
				coproduct::Coproduct,
				node::Node,
			},
		};
		let layer = Coproduct::inject(Identity(7));
		let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::send(Node::First(layer));
		let explicit: ArcRunExplicit<'static, FirstRow, Scoped, i32> =
			ArcRunExplicit::from(arc_run);
		assert!(explicit.peel().is_err());
	}

	#[test]
	fn ref_bind_chains_pure_value_via_clone() {
		let run: RunAlias<'_, i32> =
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(2));
		let chained = run.ref_bind(|x: &i32| {
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(*x + 1))
		});
		assert_eq!(chained.into_arc_free_explicit().evaluate(), 3);
	}

	#[test]
	fn ref_map_transforms_pure_value_via_clone() {
		let run: RunAlias<'_, i32> =
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(7));
		let mapped = run.ref_map(|x: &i32| *x * 3);
		assert_eq!(mapped.into_arc_free_explicit().evaluate(), 21);
	}

	#[test]
	fn ref_pure_wraps_cloned_value() {
		let value = 42;
		let run: RunAlias<'_, i32> = ArcRunExplicit::ref_pure(&value);
		assert_eq!(run.into_arc_free_explicit().evaluate(), 42);
	}
}
