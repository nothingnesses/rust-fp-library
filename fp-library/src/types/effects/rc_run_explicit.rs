//! Multi-shot Explicit-substrate Run program with `Rc`-shared continuations.
//!
//! `RcRunExplicit<'a, R, S, A>` is the multi-shot, [`Clone`]-cheap sibling
//! of [`RunExplicit`](crate::types::effects::run_explicit::RunExplicit) over
//! [`RcFreeExplicit`](crate::types::RcFreeExplicit):
//!
//! ```text
//! RcRunExplicit<'a, R, S, A> = RcFreeExplicit<'a, NodeBrand<R, S>, A>
//! ```
//!
//! The underlying [`RcFreeExplicit`](crate::types::RcFreeExplicit) carries
//! `Rc<dyn Fn>` continuations rather than single-shot ones, so handlers
//! for non-deterministic effects (`Choose`, `Amb`) can drive the same
//! suspended program more than once. The whole substrate lives behind an
//! outer [`Rc`](std::rc::Rc), so cloning a program is O(1).
//!
//! ## When to use which
//!
//! Use [`RunExplicit`](crate::types::effects::run_explicit::RunExplicit)
//! when continuations are single-shot (the common case). Use
//! `RcRunExplicit` for multi-shot effects. Use
//! [`ArcRunExplicit`](crate::types::effects::arc_run_explicit::ArcRunExplicit)
//! when programs cross thread boundaries.
//!
//! ## Brand-level coverage
//!
//! [`RcRunExplicitBrand`](crate::brands::RcRunExplicitBrand) implements
//! [`Pointed`](crate::classes::Pointed) on the by-value side and
//! [`RefFunctor`](crate::classes::RefFunctor),
//! [`RefPointed`](crate::classes::RefPointed),
//! [`RefSemimonad`](crate::classes::RefSemimonad) on the by-reference
//! side, delegating to
//! [`RcFreeExplicitBrand`](crate::brands::RcFreeExplicitBrand)'s impls.
//! [`Functor`](crate::classes::Functor) and
//! [`Semimonad`](crate::classes::Semimonad) are not reachable at the
//! brand level: per-`A` `Clone` bounds on
//! [`RcFreeExplicit::bind`](crate::types::RcFreeExplicit::bind) cannot
//! be added to the trait method signatures on stable Rust. Use the
//! inherent [`bind`](RcRunExplicit::bind) and [`map`](RcRunExplicit::map)
//! methods on `RcRunExplicit` for the by-value monadic surface at
//! concrete-type call sites; the Ref hierarchy provides
//! brand-dispatched access where canonical effect rows admit it.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				CNilBrand,
				NodeBrand,
				RcBrand,
				RcFreeExplicitBrand,
				RcRunExplicitBrand,
			},
			classes::{
				Functor,
				MonadRec,
				Pointed,
				RefCountedPointer,
				RefFunctor,
				RefPointed,
				RefSemimonad,
				WrapDrop,
			},
			functions::tail_rec_m,
			impl_kind,
			kinds::*,
			types::{
				RcCoyoneda,
				RcFree,
				RcFreeExplicit,
				effects::{
					coproduct::CoproductEmbedder,
					interpreter::{
						DispatchHandlers,
						DispatchScopedHandlers,
						RcScopedResume,
						ScopedResumeTypes,
					},
					member::Member,
					node::Node,
					rc_run::RcRun,
				},
			},
		},
		core::{
			marker::PhantomData,
			ops::ControlFlow,
		},
		fp_macros::*,
	};

	/// Multi-shot Explicit-substrate Run program with `Rc`-shared
	/// continuations: a thin wrapper over
	/// [`RcFreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::RcFreeExplicit).
	///
	/// The wrapper exists so user-facing API can be expressed without
	/// leaking the underlying [`RcFreeExplicit`](crate::types::RcFreeExplicit)
	/// representation. Cloning is O(1) (refcount bump on the inner
	/// `Rc`-wrapped substrate).
	#[document_type_parameters(
		"The lifetime that bounds the payload and the row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	pub struct RcRunExplicit<'a, R, S, A>(RcFreeExplicit<'a, NodeBrand<R, S>, A>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a;

	impl_kind! {
		impl<R: WrapDrop + Functor + 'static, S: WrapDrop + Functor + 'static>
			for RcRunExplicitBrand<R, S> {
			type Of<'a, A: 'a>: 'a = RcRunExplicit<'a, R, S, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRunExplicit` instance to clone.")]
	impl<'a, R, S, A> Clone for RcRunExplicit<'a, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a,
	{
		/// Clones the `RcRunExplicit` by bumping the refcount on the
		/// inner [`RcFreeExplicit`](crate::types::RcFreeExplicit). O(1).
		#[document_signature]
		///
		#[document_returns("A new `RcRunExplicit` representing an independent branch.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(42));
		/// let branch = run.clone();
		/// assert_eq!(run.into_rc_free_explicit().evaluate(), 42);
		/// assert_eq!(branch.into_rc_free_explicit().evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			RcRunExplicit(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRunExplicit` instance.")]
	impl<'a, R, S, A: 'a> RcRunExplicit<'a, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Wraps an
		/// [`RcFreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::RcFreeExplicit)
		/// as an `RcRunExplicit<'a, R, S, A>`. Zero-cost.
		#[document_signature]
		///
		#[document_parameters("The underlying `RcFreeExplicit` computation.")]
		///
		#[document_returns("An `RcRunExplicit` wrapping `rc_free`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(7));
		/// assert_eq!(run.into_rc_free_explicit().evaluate(), 7);
		/// ```
		#[inline]
		pub fn from_rc_free_explicit(rc_free: RcFreeExplicit<'a, NodeBrand<R, S>, A>) -> Self {
			RcRunExplicit(rc_free)
		}

		/// Unwraps an `RcRunExplicit<'a, R, S, A>` to its underlying
		/// [`RcFreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::RcFreeExplicit).
		/// Zero-cost.
		#[document_signature]
		///
		#[document_returns("The underlying `RcFreeExplicit` computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(7));
		/// let rc_free = run.into_rc_free_explicit();
		/// assert_eq!(rc_free.evaluate(), 7);
		/// ```
		#[inline]
		pub fn into_rc_free_explicit(self) -> RcFreeExplicit<'a, NodeBrand<R, S>, A> {
			self.0
		}

		/// Wraps a value in a pure `RcRunExplicit` computation.
		/// Delegates to
		/// [`RcFreeExplicit::pure`](crate::types::RcFreeExplicit).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `RcRunExplicit` computation that produces `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, i32> = RcRunExplicit::pure(42);
		/// assert_eq!(run.into_rc_free_explicit().evaluate(), 42);
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self {
			RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(a))
		}

		/// Decomposes this `RcRunExplicit` computation into one step.
		/// Walks the [`RcFreeExplicitView`](crate::types::RcFreeExplicitView)
		/// from the underlying substrate.
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` for a pure result, or `Err(layer)` carrying the next `RcRunExplicit` step."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, i32> = RcRunExplicit::pure(7);
		/// assert!(matches!(run.peel(), Ok(7)));
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "Return type encodes Result<A, NodeBrand<R, S>::Of<'a, RcRunExplicit<'a, R, S, A>>>; the GAT projection is structurally complex but cannot be aliased without losing the projection link the wrapper depends on."
		)]
		pub fn peel(
			self
		) -> Result<
			A,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, A>,
			>),
		>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone, {
			match self.0.to_view() {
				crate::types::RcFreeExplicitView::Pure(a) => Ok(a),
				crate::types::RcFreeExplicitView::Wrap(node) => {
					let mapped = <NodeBrand<R, S> as Functor>::map(
						RcRunExplicit::from_rc_free_explicit,
						node,
					);
					Err(mapped)
				}
			}
		}

		/// Lifts a [`Node`](crate::types::effects::node::Node) dispatch
		/// layer into the `RcRunExplicit` program. The `node` argument
		/// is the
		/// [`NodeBrand<R, S>`](crate::brands::NodeBrand)
		/// `Of<'a, A>` projection; `send` wraps it via
		/// [`RcFreeExplicit::wrap`](crate::types::RcFreeExplicit) after
		/// promoting each `A` into a pure `RcFreeExplicit` (no `Box`
		/// indirection because the outer `Rc<Inner>` wrapper provides
		/// it). The `Node`-projection signature is symmetric across
		/// all six Run wrappers; see
		/// [`Run::send`](crate::types::effects::run::Run::send) for the
		/// rationale.
		#[document_signature]
		///
		#[document_parameters("The Node dispatch layer carrying the effect operation.")]
		///
		#[document_returns(
			"An `RcRunExplicit` computation that performs the effect and returns its result."
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
		/// 			rc_run_explicit::RcRunExplicit,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let layer = Coproduct::inject(Identity(7));
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, i32> = RcRunExplicit::send(Node::First(layer));
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
				|a: A| -> RcFreeExplicit<'a, NodeBrand<R, S>, A> { RcFreeExplicit::pure(a) },
				node,
			);
			RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::wrap(mapped))
		}

		/// Lifts a raw effect value into an `RcRunExplicit` program.
		///
		/// Multi-shot Explicit-substrate analog of
		/// [`Run::lift`](crate::types::effects::run::Run::lift). Wraps the
		/// effect in [`RcCoyoneda::lift`](crate::types::RcCoyoneda::lift)
		/// (the `Rc`-pointer Coyoneda variant) rather than bare
		/// [`Coyoneda`](crate::types::Coyoneda) so downstream
		/// [`peel`](RcRunExplicit::peel) is callable: `RcRunExplicit::peel`
		/// requires `Of<'_, RcFreeExplicit<...>>: Clone`, which
		/// `RcCoyoneda` satisfies (`Rc::clone`) but bare `Coyoneda` does
		/// not (its `Box<dyn FnOnce>` continuation is not `Clone`). This
		/// pairs the wrapper's pointer kind with the matching Coyoneda
		/// variant: `RunExplicit`->`Coyoneda`,
		/// `RcRunExplicit`->`RcCoyoneda`,
		/// `ArcRunExplicit`->`ArcCoyoneda`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the effect being lifted.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The effect value to lift.")]
		///
		#[document_returns("An `RcRunExplicit` program suspended at the lifted effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// // The program is suspended at the lifted effect; peel reveals the layer.
		/// assert!(run.peel().is_err());
		/// ```
		#[inline]
		pub fn lift<EBrand, Idx>(
			effect: Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, EBrand, A>, Idx>,
			EBrand: Kind_cdc7cd43dac7585f + 'a,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			let coyo: RcCoyoneda<'a, EBrand, A> = RcCoyoneda::lift(effect);
			let layer = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) as Member<
				RcCoyoneda<'a, EBrand, A>,
				Idx,
			>>::inject(coyo);
			Self::send(Node::First(layer))
		}

		/// Inherent counterpart to
		/// [`RcFreeExplicit::map`](crate::types::RcFreeExplicit) by way of
		/// [`bind`](RcRunExplicit::bind) and `Pointed::pure` on the
		/// underlying substrate. The trait-bound surface is reachable
		/// through this inherent method only because per-`A` `Clone`
		/// bounds on the underlying [`RcFreeExplicit`](crate::types::RcFreeExplicit)
		/// substrate cannot be carried by the brand-level
		/// [`Functor`](crate::classes::Functor) trait method signatures.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `RcRunExplicit` with `f` applied to its result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(10));
		/// let mapped = run.map(|x: i32| x * 3);
		/// assert_eq!(mapped.into_rc_free_explicit().evaluate(), 30);
		/// ```
		pub fn map<B: 'a>(
			self,
			f: impl Fn(A) -> B + 'a,
		) -> RcRunExplicit<'a, R, S, B>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, B>,
			>): Clone, {
			RcRunExplicit::from_rc_free_explicit(self.0.bind(move |a| RcFreeExplicit::pure(f(a))))
		}

		/// Inherent
		/// [`bind`](crate::types::RcFreeExplicit::bind) over `RcRunExplicit`,
		/// reachable only via the inherent method because per-`A` `Clone`
		/// bounds on the underlying [`RcFreeExplicit`](crate::types::RcFreeExplicit)
		/// substrate cannot be carried by the brand-level
		/// [`Semimonad`](crate::classes::Semimonad) trait method
		/// signatures.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to chain after this computation.")]
		///
		#[document_returns("A new `RcRunExplicit` chaining `f` after this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(2));
		/// let chained =
		/// 	run.bind(|x: i32| RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(x + 1)));
		/// assert_eq!(chained.into_rc_free_explicit().evaluate(), 3);
		/// ```
		pub fn bind<B: 'a>(
			self,
			f: impl Fn(A) -> RcRunExplicit<'a, R, S, B> + 'a,
		) -> RcRunExplicit<'a, R, S, B>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, B>,
			>): Clone, {
			RcRunExplicit::from_rc_free_explicit(self.0.bind(move |a| f(a).into_rc_free_explicit()))
		}

		/// By-reference [`bind`](RcRunExplicit::bind): chains a
		/// continuation that receives `&A` rather than `A`.
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
		/// For synthetic rows whose row brand satisfies
		/// [`RefFunctor`](crate::classes::RefFunctor), brand-level
		/// `m_do!(ref RcRunExplicitBrand { ... })` is also available
		/// and slightly cheaper (no clone). The `im_do!` macro's
		/// `ref` form desugars to this method.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to chain after this computation.")]
		///
		#[document_returns("A new `RcRunExplicit` chaining `f` after a clone of this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(2));
		/// let chained =
		/// 	run.ref_bind(|x: &i32| RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(*x + 1)));
		/// assert_eq!(chained.into_rc_free_explicit().evaluate(), 3);
		/// ```
		pub fn ref_bind<B: 'a>(
			&self,
			f: impl Fn(&A) -> RcRunExplicit<'a, R, S, B> + 'a,
		) -> RcRunExplicit<'a, R, S, B>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, B>,
			>): Clone, {
			self.clone().bind(move |a| f(&a))
		}

		/// By-reference [`map`](RcRunExplicit::map): applies a
		/// function that takes `&A` rather than `A`.
		///
		/// Implemented via `self.clone().map(move |a| f(&a))`. The
		/// clone is `O(1)` (atomic-free `Rc::clone`). See
		/// [`ref_bind`](RcRunExplicit::ref_bind) for the design
		/// rationale.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply by reference to the result.")]
		///
		#[document_returns(
			"A new `RcRunExplicit` with `f` applied to a clone of this one's result."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(7));
		/// let mapped = run.ref_map(|x: &i32| *x * 3);
		/// assert_eq!(mapped.into_rc_free_explicit().evaluate(), 21);
		/// ```
		pub fn ref_map<B: 'a>(
			&self,
			f: impl Fn(&A) -> B + 'a,
		) -> RcRunExplicit<'a, R, S, B>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, B>,
			>): Clone, {
			self.clone().map(move |a| f(&a))
		}

		/// By-reference [`pure`](RcRunExplicit::pure): wraps a
		/// cloned value in an `RcRunExplicit` computation.
		///
		/// Implemented as `RcRunExplicit::pure(a.clone())`. Requires
		/// `A: Clone`. Parallel to brand-level
		/// [`RefPointed::ref_pure`](crate::classes::RefPointed) for
		/// concrete-type call sites; the brand-level form is
		/// [`<RcRunExplicitBrand<R, S> as RefPointed>::ref_pure(&a)`](crate::brands::RcRunExplicitBrand).
		///
		/// The `im_do!` macro's `ref` form rewrites bare `pure(x)`
		/// calls inside `im_do!(ref RcRunExplicit { ... })` to this
		/// method.
		#[document_signature]
		///
		#[document_parameters("A reference to the value to wrap.")]
		///
		#[document_returns("An `RcRunExplicit` computation that produces a clone of `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let value = 42;
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, i32> = RcRunExplicit::ref_pure(&value);
		/// assert_eq!(run.into_rc_free_explicit().evaluate(), 42);
		/// ```
		#[inline]
		pub fn ref_pure(a: &A) -> Self
		where
			A: Clone, {
			RcRunExplicit::pure(a.clone())
		}
	}

	#[doc(hidden)]
	/// Rc-backed Explicit carrier for a selected scoped action.
	///
	/// The carrier keeps the action and the action's outer continuation
	/// separate while preserving the `RcFreeExplicit` multi-shot contract:
	/// cloning the carrier is O(1), and post-action work is a reusable `Fn`
	/// continuation over the selected action value.
	#[derive(Clone)]
	pub(crate) struct RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Final: 'a,
		K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a, {
		/// The selected scoped action before its outer continuation has
		/// been reattached.
		pub(crate) action: RcRunExplicit<'a, R, S, Action>,
		/// The action's outer continuation, still outside the selected action.
		pub(crate) outer: <RcBrand as RefCountedPointer>::Of<'a, K>,
		/// Carries the final result type without owning a value of that type.
		pub(crate) result: PhantomData<fn() -> Final>,
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type."
	)]
	impl<'a, R, S, Action, Final, K> ScopedResumeTypes<'a>
		for RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Final: 'a,
		K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a,
	{
		type ActionProgram = RcRunExplicit<'a, R, S, Action>;
		type ActionValue = Action;
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The selected action result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The first-order row layer shape passed to first-order handlers."
	)]
	#[document_parameters("The RcRunExplicit scoped-continuation carrier.")]
	impl<'a, R, S, Action, Final, K, FirstLayer>
		RcScopedResume<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>
		for RcRunExplicitScopedContinuation<'a, R, S, Action, Final, K>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Action: Clone + 'a,
		Final: 'a,
		K: Fn(Action) -> RcRunExplicit<'a, R, S, Final> + 'a,
		FirstLayer: 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Action>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone,
	{
		/// Resume the selected action by reattaching its outer continuation.
		#[document_signature]
		///
		#[document_parameters("The first-order handler list retained by the carrier contract.")]
		#[document_returns("The resumed `RcRunExplicit` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// let run: RcRunExplicit<'_, CNilBrand, CNilBrand, i32> = RcRunExplicit::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn resume_rc(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
		) -> RcRunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();
			self.action.bind(move |action_value: Action| -> RcRunExplicit<'a, R, S, Final> {
				outer(action_value)
			})
		}

		/// Insert a result-preserving action program before reattaching
		/// the selected action's outer continuation.
		#[document_signature]
		///
		#[document_parameters(
			"The first-order handler list retained by the carrier contract.",
			"The result-preserving action program to apply before the outer continuation."
		)]
		#[document_returns("The resumed `RcRunExplicit` program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// let run: RcRunExplicit<'_, CNilBrand, CNilBrand, i32> = RcRunExplicit::pure(41);
		/// let incremented = run.bind(|value| RcRunExplicit::pure(value + 1));
		/// assert_eq!(incremented.extract(), 42);
		/// ```
		fn resume_rc_with_post_action(
			self,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
			post_action: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionValue,
			) -> <Self as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> RcRunExplicit<'a, R, S, Final> {
			let outer = self.outer.clone();

			self.action.bind(move |action_value: Action| -> RcRunExplicit<'a, R, S, Final> {
				let outer = outer.clone();
				let post_program: RcRunExplicit<'a, R, S, Action> = post_action(action_value);
				post_program.bind(move |post_value: Action| -> RcRunExplicit<'a, R, S, Final> {
					outer(post_value)
				})
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRunExplicit` instance.")]
	impl<'a, R, S, A: 'a> RcRunExplicit<'a, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Interprets this `RcRunExplicit` program by walking each
		/// effect via the matching handler closure in `handlers`.
		/// Multi-shot, lifetime-flexible variant of
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
		/// 			handlers::*,
		/// 			rc_run_explicit::RcRunExplicit,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let result = prog.interpret(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<RcRunExplicit<'static, FirstRow, Scoped, i32>>| op.0,
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
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RcRunExplicit<'a, R, S, A>>),
				RcRunExplicit<'a, R, S, A>,
			>,
			scoped_handlers: impl DispatchScopedHandlers<
				'a,
				Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
				RcRunExplicit<'a, R, S, A>,
			>,
		) -> A
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
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

		/// Alias for [`interpret`](RcRunExplicit::interpret), kept for
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
		/// 			handlers::*,
		/// 			rc_run_explicit::RcRunExplicit,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::lift::<IdentityBrand, _>(Identity(99));
		/// let result = prog.run(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<RcRunExplicit<'static, FirstRow, Scoped, i32>>| op.0,
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
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RcRunExplicit<'a, R, S, A>>),
				RcRunExplicit<'a, R, S, A>,
			>,
			scoped_handlers: impl DispatchScopedHandlers<
				'a,
				Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
				Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
				RcRunExplicit<'a, R, S, A>,
			>,
		) -> A
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone, {
			self.interpret(handlers, scoped_handlers)
		}

		/// MonadRec-target interpreter for [`RcRunExplicit`]. Mirrors
		/// [`Run::interpret_rec`](crate::types::effects::run::Run::interpret_rec);
		/// see that method's docs for the handler shape, loop body,
		/// and stack-safety guarantee. `RcRunExplicit` differences:
		/// the per-`peel` `A: Clone` and substrate-`Of<...>: Clone`
		/// bounds propagate (matching [`RcRunExplicit::interpret`]).
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
		/// 			rc_run_explicit::RcRunExplicit,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let result: Thunk<'static, i32> = prog.interpret_rec::<ThunkBrand>(handlers! {
		/// 	IdentityBrand: |op: Identity<Thunk<'static, RcRunExplicit<'static, FirstRow, Scoped, i32>>>| op.0,
		/// }, fp_library::types::effects::scoped_nt());
		/// assert_eq!(result.evaluate(), 42);
		/// ```
		#[inline]
		pub fn interpret_rec<MBrand>(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
			> + 'a,
			scoped_handlers: impl DispatchScopedHandlers<
				'a,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
			> + 'a,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			MBrand: MonadRec + 'static,
			A: Clone + 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone, {
			tail_rec_m::<MBrand, RcRunExplicit<'a, R, S, A>, A>(
				move |prog: RcRunExplicit<'a, R, S, A>| match prog.peel() {
					Ok(a) =>
						<MBrand as Pointed>::pure::<ControlFlow<A, RcRunExplicit<'a, R, S, A>>>(
							ControlFlow::Break(a),
						),
					Err(Node::First(layer)) => {
						let mapped = <R as Functor>::map(
							|inner: RcRunExplicit<'a, R, S, A>| {
								<MBrand as Pointed>::pure::<RcRunExplicit<'a, R, S, A>>(inner)
							},
							layer,
						);
						let next = handlers.dispatch(mapped);
						<MBrand as Functor>::map::<
							RcRunExplicit<'a, R, S, A>,
							ControlFlow<A, RcRunExplicit<'a, R, S, A>>,
						>(ControlFlow::Continue, next)
					}
					Err(Node::Scoped(layer)) => {
						let mapped = <S as Functor>::map(
							|inner: RcRunExplicit<'a, R, S, A>| {
								<MBrand as Pointed>::pure::<RcRunExplicit<'a, R, S, A>>(inner)
							},
							layer,
						);
						let next = scoped_handlers.dispatch_scoped(mapped, &handlers);
						<MBrand as Functor>::map::<
							RcRunExplicit<'a, R, S, A>,
							ControlFlow<A, RcRunExplicit<'a, R, S, A>>,
						>(ControlFlow::Continue, next)
					}
				},
				self,
			)
		}

		/// Alias for [`interpret_rec`](RcRunExplicit::interpret_rec).
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
		/// 		Thunk,
		/// 		effects::{
		/// 			handlers::*,
		/// 			rc_run_explicit::RcRunExplicit,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::lift::<IdentityBrand, _>(Identity(99));
		/// let result: Thunk<'static, i32> = prog.run_rec::<ThunkBrand>(handlers! {
		/// 	IdentityBrand: |op: Identity<Thunk<'static, RcRunExplicit<'static, FirstRow, Scoped, i32>>>| op.0,
		/// }, fp_library::types::effects::scoped_nt());
		/// assert_eq!(result.evaluate(), 99);
		/// ```
		#[inline]
		pub fn run_rec<MBrand>(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'h,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
			> + 'a,
			scoped_handlers: impl DispatchScopedHandlers<
				'a,
				Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
				>),
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
				>),
				Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
			> + 'a,
		) -> Apply!(<MBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			MBrand: MonadRec + 'static,
			A: Clone + 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone, {
			self.interpret_rec::<MBrand>(handlers, scoped_handlers)
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRunExplicit` instance.")]
	impl<'a, R, S, A: 'a> RcRunExplicit<'a, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Scoped-row-narrowing interpreter: interpret a single scoped
		/// effect `SBrand` out of the scoped row, returning an
		/// `RcRunExplicit` program in the narrowed scoped row
		/// `SMinusE`.
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
		#[document_returns("An `RcRunExplicit` program in the narrowed scoped row `SMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		span::Span,
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
		///
		/// let action: RcRunExplicit<'static, CNilBrand, ScopedRow, i32> = RcRunExplicit::pure(7);
		/// let prog: RcRunExplicit<'static, CNilBrand, ScopedRow, i32> =
		/// 	RcRunExplicit::span::<&'static str, _>("request", action);
		/// let narrowed: RcRunExplicit<'static, CNilBrand, CNilBrand, i32> = prog
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
				Apply!(<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, SMinusE, A>>),
			) -> RcRunExplicit<'a, R, SMinusE, A>
			+ 'a,
		) -> RcRunExplicit<'a, R, SMinusE, A>
		where
			A: Clone,
			SBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, SMinusE> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, SMinusE>, A>,
			>): Clone,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>):
				Member<
						Apply!(
							<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
						),
						Idx,
						Remainder = Apply!(
										<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
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
		#[document_returns("An `RcRunExplicit` program in the narrowed scoped row `SMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// // Exercised internally by RcRunExplicit::interpret_scoped_with.
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		span::Span,
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
		///
		/// let action: RcRunExplicit<'static, CNilBrand, ScopedRow, i32> = RcRunExplicit::pure(7);
		/// let prog: RcRunExplicit<'static, CNilBrand, ScopedRow, i32> =
		/// 	RcRunExplicit::span::<&'static str, _>("request", action);
		/// let narrowed: RcRunExplicit<'static, CNilBrand, CNilBrand, i32> = prog
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
		fn interpret_scoped_with_shared<SBrand, Idx, SMinusE, F>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'a, F>,
		) -> RcRunExplicit<'a, R, SMinusE, A>
		where
			F: Fn(
					Apply!(
						<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							RcRunExplicit<'a, R, SMinusE, A>,
						>
					),
				) -> RcRunExplicit<'a, R, SMinusE, A>
				+ 'a,
			A: Clone,
			SBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			SMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, SMinusE> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, SMinusE>, A>,
			>): Clone,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>):
				Member<
						Apply!(
							<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
						),
						Idx,
						Remainder = Apply!(
										<SMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
									),
					>, {
			match self.peel() {
				Ok(a) => RcRunExplicit::pure(a),
				Err(Node::First(layer)) => {
					let h_for_recurse = handler.clone();
					let mapped_free = <R as Functor>::map(
						move |inner: RcRunExplicit<'a, R, S, A>| {
							inner
								.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
									h_for_recurse.clone(),
								)
								.into_rc_free_explicit()
						},
						layer,
					);
					RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::<
						'a,
						NodeBrand<R, SMinusE>,
						A,
					>::wrap(Node::First(mapped_free)))
				}
				Err(Node::Scoped(layer)) =>
					match <Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, S, A>,
					>) as Member<
						Apply!(
							<SBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'a,
								RcRunExplicit<'a, R, S, A>,
							>
						),
						Idx,
					>>::project(layer)
					{
						Ok(scoped) => {
							let h_for_recurse = handler.clone();
							let mapped = <SBrand as Functor>::map(
								move |inner: RcRunExplicit<'a, R, S, A>| {
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
								move |inner: RcRunExplicit<'a, R, S, A>| {
									inner
										.interpret_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
											h_for_recurse.clone(),
										)
										.into_rc_free_explicit()
								},
								rest,
							);
							RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::<
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
		/// for cross-wrapper semantics. `RcRunExplicit` pairs the
		/// [`RcCoyoneda`] variant with the Explicit `Box<dyn FnOnce>`
		/// substrate; the matched-arm dispatch lowers via
		/// [`RcCoyoneda::lower_ref`].
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
		#[document_returns("An `RcRunExplicit` program in the narrowed row.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FullRow, CNilBrand, i32> =
		/// 	RcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: RcRunExplicit<'static, EmptyRow, CNilBrand, i32> = prog
		/// 	.interpret_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<RcRunExplicit<'static, EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		pub fn interpret_with<EBrand, Idx, RMinusE>(
			self,
			handler: impl Fn(
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, RMinusE, S, A>>),
			) -> RcRunExplicit<'a, RMinusE, S, A>
			+ 'a,
		) -> RcRunExplicit<'a, RMinusE, S, A>
		where
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusE, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusE, S>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>):
				Member<
						RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
									),
					>, {
			let handler = <RcBrand as RefCountedPointer>::new(handler);
			self.interpret_with_shared::<EBrand, Idx, RMinusE, _>(handler)
		}

		/// Inner pipeline-narrowing implementation, parameterised
		/// over the concrete handler closure type `F`. The public
		/// [`interpret_with`](RcRunExplicit::interpret_with) wraps
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
		#[document_returns("An `RcRunExplicit` program in the narrowed row `RMinusE`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FullRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type EmptyRow = CNilBrand;
		///
		/// // Exercised internally by RcRunExplicit::interpret_with.
		/// let prog: RcRunExplicit<'static, FullRow, CNilBrand, i32> =
		/// 	RcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: RcRunExplicit<'static, EmptyRow, CNilBrand, i32> = prog
		/// 	.interpret_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<RcRunExplicit<'static, EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		fn interpret_with_shared<EBrand, Idx, RMinusE, F>(
			self,
			handler: <RcBrand as RefCountedPointer>::Of<'a, F>,
		) -> RcRunExplicit<'a, RMinusE, S, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, RMinusE, S, A>>),
				) -> RcRunExplicit<'a, RMinusE, S, A>
				+ 'a,
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusE, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusE, S>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>):
				Member<
						RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
									),
					>, {
			match self.peel() {
				Ok(a) => RcRunExplicit::pure(a),
				Err(Node::First(layer)) =>
					match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
					) as Member<RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>, Idx>>::project(
						layer
					) {
						Ok(coyo) => {
							let lowered = coyo.lower_ref();
							let h_for_recurse = handler.clone();
							let mapped = <EBrand as Functor>::map(
								move |inner: RcRunExplicit<'a, R, S, A>| {
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
								move |inner: RcRunExplicit<'a, R, S, A>| {
									inner
										.interpret_with_shared::<EBrand, Idx, RMinusE, F>(
											h_for_recurse.clone(),
										)
										.into_rc_free_explicit()
								},
								rest,
							);
							RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::<
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
					let mapped_free = <S as Functor>::map(
						move |inner: RcRunExplicit<'a, R, S, A>| {
							inner
								.interpret_with_shared::<EBrand, Idx, RMinusE, F>(
									h_for_recurse.clone(),
								)
								.into_rc_free_explicit()
						},
						layer,
					);
					RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::<
						'a,
						NodeBrand<RMinusE, S>,
						A,
					>::wrap(Node::Scoped(mapped_free)))
				}
			}
		}

		/// Substrate-level row-preserving replacement primitive: walk
		/// this `RcRunExplicit` program, projecting each first-order
		/// dispatch against `EBrand`; replace every matched dispatch
		/// with the supplied `replacement` closure (applied to the
		/// lowered effect value), and re-emit non-matching dispatches
		/// in the same row. Direct analog of heftia's
		/// `interposeInWith` in substrate-primitive form, on the
		/// explicit-lifetime multi-shot Rc-shared substrate.
		///
		/// Unlike [`interpret_with`](RcRunExplicit::interpret_with),
		/// `interpose` does not narrow the row: the matched arm
		/// produces a continuation in the same `R`, the unmatched arm
		/// walks the `Self::Remainder` (`RMinusE`) layer and embeds
		/// it back into `R` via [`CoproductEmbedder`](crate::types::effects::coproduct::CoproductEmbedder).
		/// This is the building block for scoped-effect handlers.
		///
		/// The user-facing closure is wrapped in an
		/// [`Rc`](std::rc::Rc) once at entry; recursive calls clone
		/// the [`Rc`](std::rc::Rc) (refcount bump) instead of cloning
		/// the underlying closure. The closure carries the same `'a`
		/// lifetime as the program (not `'static`), so it can borrow
		/// from external state for the program's lifetime.
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
		/// 		effects::rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = RcRunExplicit<'static, Row, CNilBrand, i32>;
		///
		/// let prog: Prog = RcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
		/// let interposed = prog
		/// 	.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| RcRunExplicit::pure(99));
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
				Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
			) -> RcRunExplicit<'a, R, S, A>
			+ 'a,
		) -> RcRunExplicit<'a, R, S, A>
		where
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>):
				Member<
						RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
									),
					>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			let replacement = <RcBrand as RefCountedPointer>::new(replacement);
			self.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, _>(replacement)
		}

		/// Inner shared implementation of [`interpose`](RcRunExplicit::interpose),
		/// parameterised over the concrete replacement closure type
		/// `F`. The public [`interpose`](RcRunExplicit::interpose)
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
		/// // Exercised internally by RcRunExplicit::interpose.
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = RcRunExplicit<'static, Row, CNilBrand, i32>;
		///
		/// let prog: Prog = RcRunExplicit::lift::<IdentityBrand, _>(Identity(3));
		/// let interposed = prog
		/// 	.interpose::<IdentityBrand, _, CNilBrand, _>(|_op: Identity<Prog>| RcRunExplicit::pure(42));
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
		) -> RcRunExplicit<'a, R, S, A>
		where
			F: Fn(
					Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
				) -> RcRunExplicit<'a, R, S, A>
				+ 'a,
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>):
				Member<
						RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
									),
					>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<R, S>, A>,
				>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => RcRunExplicit::pure(a),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
				) as Member<
					RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
					Idx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let lowered = coyo.lower_ref();
						let r_for_recurse = replacement.clone();
						let mapped = <EBrand as Functor>::map(
							move |inner: RcRunExplicit<'a, R, S, A>| {
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
							move |inner: RcRunExplicit<'a, R, S, A>| {
								inner
									.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
										r_for_recurse.clone(),
									)
									.into_rc_free_explicit()
							},
							rest,
						);
						let layer_back = mapped_rest.embed();
						RcRunExplicit::from_rc_free_explicit(
							RcFreeExplicit::<'a, NodeBrand<R, S>, A>::wrap(Node::First(layer_back)),
						)
					}
				},
				Err(Node::Scoped(layer)) => {
					let r_for_recurse = replacement.clone();
					let mapped_free = <S as Functor>::map(
						move |inner: RcRunExplicit<'a, R, S, A>| {
							inner
								.interpose_shared::<EBrand, Idx, RMinusE, EmbedIndices, F>(
									r_for_recurse.clone(),
								)
								.into_rc_free_explicit()
						},
						layer,
					);
					RcRunExplicit::from_rc_free_explicit(
						RcFreeExplicit::<'a, NodeBrand<R, S>, A>::wrap(Node::Scoped(mapped_free)),
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
	#[document_parameters("The first-order-only `RcRunExplicit` instance.")]
	impl<'a, R, A: 'a> RcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
	{
		/// Substrate-level matched-effect short-circuit primitive on
		/// the explicit-lifetime multi-shot Run wrapper: walk this
		/// `RcRunExplicit` program, dispatching non-matched first-order
		/// effects through `fo_handlers` and short-circuiting the
		/// moment a matched-effect (`EBrand`) dispatch is encountered,
		/// returning the matched effect's lowered payload.
		///
		/// Returns `Ok(a)` when the program reduces to a pure value
		/// without firing the matched effect; returns `Err(op)` with
		/// the matched effect's lowered payload otherwise.
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
		/// 			rc_run_explicit::RcRunExplicit,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<
		/// 	RcCoyonedaBrand<ExceptBrand<String>>,
		/// 	CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>,
		/// >;
		/// type RowMinusExcept = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = RcRunExplicit<'static, Row, CNilBrand, i32>;
		///
		/// let prog: Prog = RcRunExplicit::throw::<String, _>("oops".to_string());
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
				Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, RcRunExplicit<'a, R, CNilBrand, A>>),
				RcRunExplicit<'a, R, CNilBrand, A>,
			>,
		) -> Result<
			A,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, CNilBrand, A>>),
		>
		where
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, CNilBrand, A>>):
				Member<
						RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, CNilBrand, A>>,
						Idx,
						Remainder = Apply!(
										<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, CNilBrand, A>>
									),
					>, {
			let mut prog = self;
			loop {
				match prog.peel() {
					Ok(a) => return Ok(a),
					Err(Node::First(layer)) => match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RcRunExplicit<'a, R, CNilBrand, A>>
					) as Member<
						RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, CNilBrand, A>>,
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
	#[document_parameters("The `RcRunExplicit` instance.")]
	impl<'a, A: 'a> RcRunExplicit<'a, CNilBrand, CNilBrand, A>
	where
		A: Clone,
	{
		/// Extracts the result value from an `RcRunExplicit` program whose
		/// first-order and scoped rows have both been fully interpreted
		/// away. Exhaustive `match` over the uninhabited `CNil` payloads
		/// proves no runtime panic, statically.
		/// See [`Run::extract`](crate::types::effects::run::Run::extract).
		#[document_signature]
		///
		#[document_returns("The final result value of the fully-narrowed program.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// let pure_prog: RcRunExplicit<'_, CNilBrand, CNilBrand, i32> = RcRunExplicit::pure(42);
		/// assert_eq!(pure_prog.extract(), 42);
		/// ```
		#[inline]
		pub fn extract(self) -> A
		where
			Apply!(<NodeBrand<CNilBrand, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<CNilBrand, CNilBrand>, A>,
			>): Clone, {
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
		"The body's result type."
	)]
	impl<'a, R, ScopedRow, B> RcRunExplicit<'a, R, ScopedRow, B>
	where
		R: WrapDrop + Functor + 'a,
		ScopedRow: WrapDrop + Functor + 'a,
		B: 'a,
	{
		/// Lifts a [`RefBracketExplicit`](crate::types::effects::ref_bracket::RefBracketExplicit)
		/// scoped resource-management effect into the `RcRunExplicit`
		/// program. This is the Ref flavour of
		/// [`RcRunExplicit::bracket`]: `acquire` produces the resource,
		/// while `body` and `release` both receive independent `Rc<A>`
		/// clones.
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
		#[document_returns(
			"An `RcRunExplicit` program suspended at the scoped `RefBracket` effect."
		)]
		#[document_examples]
		///
		/// User-facing scoped rows containing
		/// [`RefBracketExplicitBrand`](crate::brands::RefBracketExplicitBrand)
		/// cannot be defined as type aliases. Use the marker-struct
		/// workaround validated by the
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
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type UnderlyingRow = CoproductBrand<
		/// 	RefBracketExplicitBrand<RcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>,
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
		/// let acquire: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(7);
		/// let prog: RcRunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	RcRunExplicit::<'static, FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 		acquire,
		/// 		|resource: std::rc::Rc<i32>| RcRunExplicit::pure(*resource + 35),
		/// 		|_resource: std::rc::Rc<i32>| RcRunExplicit::pure(()),
		/// 	);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn ref_bracket<A: 'a, Idx>(
			acquire: RcRunExplicit<'a, R, ScopedRow, A>,
			body: impl Fn(
				<RcBrand as crate::classes::RefCountedPointer>::Of<'a, A>,
			) -> RcRunExplicit<'a, R, ScopedRow, B>
			+ 'a,
			release: impl Fn(
				<RcBrand as crate::classes::RefCountedPointer>::Of<'a, A>,
			) -> RcRunExplicit<'a, R, ScopedRow, ()>
			+ 'a,
		) -> Self
		where
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, B>,
			>): Member<
					crate::types::effects::ref_bracket::RefBracketExplicit<
						'a,
						RcBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				>, {
			let bracket: crate::types::effects::ref_bracket::RefBracketExplicit<
				'a,
				RcBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::ref_bracket::RefBracketExplicit::Bracket {
				acquire: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					acquire.clone().into_rc_free_explicit()
				}),
				body: <RcBrand as crate::classes::ToDynCloneFn>::new(
					move |a: <RcBrand as crate::classes::RefCountedPointer>::Of<'a, A>| {
						body(a).into_rc_free_explicit()
					},
				),
				release: <RcBrand as crate::classes::ToDynCloneFn>::new(
					move |a: <RcBrand as crate::classes::RefCountedPointer>::Of<'a, A>| {
						release(a).into_rc_free_explicit()
					},
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, B>,
			>) as Member<
				crate::types::effects::ref_bracket::RefBracketExplicit<
					'a,
					RcBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			let node = Node::Scoped(layer);
			RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::wrap(node))
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The state type (also the program's result type for `get`)."
	)]
	impl<'a, R, ScopedRow, A: 'a> RcRunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		/// Lifts a `Get` state effect into the `RcRunExplicit` program.
		/// Mirrors [`Run::get`](crate::types::effects::run::Run::get);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RcRunExplicit`: the [`RcCoyoneda`] variant pairs with the
		/// `Rc`-shared Explicit substrate (multi-shot continuations);
		/// `A: Clone` is required because the underlying `RcCoyoneda`
		/// substrate's `peel` walks shared continuation projections.
		/// Threads [`RcBrand`](crate::brands::RcBrand) as the pointer
		/// kind.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Get` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		state::State,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> = RcRunExplicit::get();
		/// // The program is suspended at the Get effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn get<Idx>() -> Self
		where
			A: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Member<RcCoyoneda<'a, crate::brands::StateBrand<crate::brands::RcBrand, A>, A>, Idx>,
		{
			let effect: crate::types::effects::state::State<'a, crate::brands::RcBrand, A, A> =
				crate::types::effects::state::State::Get(
					<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|s: A| s),
				);
			Self::lift::<crate::brands::StateBrand<crate::brands::RcBrand, A>, Idx>(effect)
		}

		/// Lifts an `Ask` reader effect into the `RcRunExplicit`
		/// program. Mirrors
		/// [`Run::ask`](crate::types::effects::run::Run::ask); see
		/// that method for cross-wrapper semantics. Differences for
		/// `RcRunExplicit`: the [`RcCoyoneda`] variant pairs with the
		/// `Rc`-shared Explicit substrate (multi-shot continuations);
		/// `A: Clone` is required because the underlying `RcCoyoneda`
		/// substrate's `peel` walks shared continuation projections.
		/// Threads [`RcBrand`](crate::brands::RcBrand) as the pointer
		/// kind.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Ask` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		reader::Reader,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> = RcRunExplicit::ask();
		/// // The program is suspended at the Ask effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn ask<Idx>() -> Self
		where
			A: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Member<
					RcCoyoneda<'a, crate::brands::ReaderBrand<crate::brands::RcBrand, A>, A>,
					Idx,
				>, {
			let effect: crate::types::effects::reader::Reader<'a, crate::brands::RcBrand, A, A> =
				crate::types::effects::reader::Reader::Ask(
					<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|e: A| e),
				);
			Self::lift::<crate::brands::ReaderBrand<crate::brands::RcBrand, A>, Idx>(effect)
		}

		/// Lifts a `Throw` except effect into the `RcRunExplicit`
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
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Throw` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		except::Except,
		/// 		rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::throw::<&'static str, _>("oops");
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn throw<ErrorType: Clone + 'static, Idx>(e: ErrorType) -> Self
		where
			A: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, crate::brands::ExceptBrand<ErrorType>, A>, Idx>, {
			let effect: crate::types::effects::except::Except<'a, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}

		/// Lifts a scoped `Catch` effect into the `RcRunExplicit` program:
		/// run `action`, and if it throws an `E`, invoke `handler` with
		/// the error to produce a recovery program. Mirrors
		/// [`RcRun::catch`](crate::types::effects::rc_run::RcRun::catch);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RcRunExplicit`: the action and recovery handler are stored
		/// as `Rc<dyn Fn(...) -> _>` thunks (multi-shot) over the
		/// explicit `'a` lifetime.
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
		#[document_returns("An `RcRunExplicit` program suspended at the scoped `Catch` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<CatchBrand<RcBrand, &'static str>, CNilBrand>;
		///
		/// let action: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(42);
		/// let prog: RcRunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	RcRunExplicit::catch::<&'static str, _>(action, |_e| RcRunExplicit::pure(0));
		/// // The program is suspended at the Catch scoped layer; peel
		/// // returns Err carrying a `Node::Scoped(...)` projection.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn catch<E: 'a, Idx>(
			action: RcRunExplicit<'a, R, ScopedRow, A>,
			handler: impl Fn(E) -> RcRunExplicit<'a, R, ScopedRow, A> + 'a,
		) -> Self
		where
			A: Clone + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::catch::Catch<
						'a,
						RcBrand,
						E,
						RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone, {
			let catch: crate::types::effects::catch::Catch<
				'a,
				RcBrand,
				E,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::catch::Catch::Catch {
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					action.clone().into_rc_free_explicit()
				}),
				handler: <RcBrand as crate::classes::ToDynCloneFn>::new(move |e: E| {
					handler(e).into_rc_free_explicit()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::catch::Catch<
					'a,
					RcBrand,
					E,
					RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(catch);
			let node = Node::Scoped(layer);
			RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::wrap(node))
		}

		/// Lifts a scoped `Local` effect into the `RcRunExplicit`
		/// program: run `action` under an environment value transformed
		/// by `modify`. Mirrors
		/// [`RcRun::local`](crate::types::effects::rc_run::RcRun::local);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RcRunExplicit`: the modify closure and action are stored as
		/// `Rc<dyn Fn(...) -> _>` thunks (multi-shot) over the explicit
		/// `'a` lifetime.
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
		#[document_returns("An `RcRunExplicit` program suspended at the scoped `Local` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<LocalBrand<RcBrand, i32>, CNilBrand>;
		///
		/// let action: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(42);
		/// let prog: RcRunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	RcRunExplicit::local::<i32, _>(|e: i32| e + 1, action);
		/// // The program is suspended at the Local scoped layer; peel
		/// // returns Err carrying a `Node::Scoped(...)` projection.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn local<E: 'a, Idx>(
			modify: impl Fn(E) -> E + 'a,
			action: RcRunExplicit<'a, R, ScopedRow, A>,
		) -> Self
		where
			A: Clone + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::local::Local<
						'a,
						RcBrand,
						E,
						RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone, {
			let local: crate::types::effects::local::Local<
				'a,
				RcBrand,
				E,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::local::Local::Local {
				modify: <RcBrand as crate::classes::ToDynCloneFn>::new(move |e: E| modify(e)),
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					action.clone().into_rc_free_explicit()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::local::Local<
					'a,
					RcBrand,
					E,
					RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(local);
			let node = Node::Scoped(layer);
			RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::wrap(node))
		}

		/// Lifts a [`RefLocal`](crate::types::effects::ref_local::RefLocal)
		/// scoped environment-modification effect (Ref flavour) into the
		/// `RcRunExplicit` program. Mirrors
		/// [`Run::ref_local`](crate::types::effects::run::Run::ref_local)
		/// for the multi-shot Rc explicit-lifetime substrate. The
		/// `modify` closure (`Fn(&E) -> E + 'a`) borrows the inherited
		/// environment value rather than consuming it, removing the
		/// `E: Clone` requirement that the Val flavour
		/// ([`local`](RcRunExplicit::local)) imposes.
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
			"An `RcRunExplicit` program suspended at the scoped `Local` effect (Ref flavour)."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<RefLocalBrand<RcBrand, i32>, CNilBrand>;
		///
		/// let action: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(42);
		/// let prog: RcRunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	RcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 1, action);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn ref_local<E: 'a, Idx>(
			modify: impl Fn(&E) -> E + 'a,
			action: RcRunExplicit<'a, R, ScopedRow, A>,
		) -> Self
		where
			A: Clone + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::ref_local::RefLocal<
						'a,
						RcBrand,
						E,
						RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone, {
			let local: crate::types::effects::ref_local::RefLocal<
				'a,
				RcBrand,
				E,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::ref_local::RefLocal::Local {
				modify: <RcBrand as crate::classes::ToDynCloneFn>::ref_new(move |e: &E| modify(e)),
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					action.clone().into_rc_free_explicit()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::ref_local::RefLocal<
					'a,
					RcBrand,
					E,
					RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(local);
			let node = Node::Scoped(layer);
			RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::wrap(node))
		}

		/// Lifts a scoped `Span` effect into the `RcRunExplicit`
		/// program: run `action` under instrumentation identified by
		/// `tag`. Mirrors
		/// [`RcRun::span`](crate::types::effects::rc_run::RcRun::span);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RcRunExplicit`: the action is stored as an
		/// `Rc<dyn Fn(()) -> _>` thunk over the explicit `'a` lifetime.
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
		#[document_returns("An `RcRunExplicit` program suspended at the scoped `Span` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CNilBrand;
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
		///
		/// let action: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(42);
		/// let prog: RcRunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	RcRunExplicit::span::<&'static str, _>("request", action);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn span<Tag: Clone + 'a, Idx>(
			tag: Tag,
			action: RcRunExplicit<'a, R, ScopedRow, A>,
		) -> Self
		where
			A: Clone + 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Member<
					crate::types::effects::span::Span<
						'a,
						RcBrand,
						Tag,
						RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone, {
			let span: crate::types::effects::span::Span<
				'a,
				RcBrand,
				Tag,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			> = crate::types::effects::span::Span::Span {
				tag,
				action: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					action.clone().into_rc_free_explicit()
				}),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>) as Member<
				crate::types::effects::span::Span<
					'a,
					RcBrand,
					Tag,
					RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
				>,
				Idx,
			>>::inject(span);
			let node = Node::Scoped(layer);
			RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::wrap(node))
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The body's result type."
	)]
	impl<'a, R, ScopedRow, B> RcRunExplicit<'a, R, ScopedRow, B>
	where
		R: WrapDrop + Functor + 'a,
		ScopedRow: WrapDrop + Functor + 'a,
		B: 'a,
	{
		/// Lifts a [`BracketExplicit`](crate::types::effects::bracket::BracketExplicit)
		/// scoped resource-management effect into the `RcRunExplicit`
		/// program. Mirrors [`Run::bracket`](crate::types::effects::run::Run::bracket)
		/// for the multi-shot Rc explicit-lifetime substrate.
		#[document_signature]
		///
		#[document_type_parameters(
			"The resource type produced by acquire.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters(
			"The acquire program (produces the resource).",
			"The body closure (receives the resource as `Rc<A>` and returns a paired program).",
			"The release closure (receives the resource as `Rc<A>` and returns a unit program)."
		)]
		///
		#[document_returns("An `RcRunExplicit` program suspended at the scoped `Bracket` effect.")]
		///
		#[document_examples]
		///
		/// User-facing scoped rows containing
		/// [`BracketExplicitBrand`](crate::brands::BracketExplicitBrand)
		/// cannot be defined as type aliases. Use the marker-struct
		/// workaround validated by the
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
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type UnderlyingRow = CoproductBrand<
		/// 	BracketExplicitBrand<RcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>,
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
		/// let acquire: RcRunExplicit<'static, FirstRow, ScopedRow, i32> = RcRunExplicit::pure(7);
		/// let prog: RcRunExplicit<'static, FirstRow, ScopedRow, i32> =
		/// 	RcRunExplicit::<'static, FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		/// 		acquire,
		/// 		|resource: std::rc::Rc<i32>| RcRunExplicit::pure((*resource, 42)),
		/// 		|_resource: std::rc::Rc<i32>| RcRunExplicit::pure(()),
		/// 	);
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn bracket<A, Idx>(
			acquire: RcRunExplicit<'a, R, ScopedRow, A>,
			body: impl Fn(
				<RcBrand as crate::classes::Pointer>::Of<'a, A>,
			) -> RcRunExplicit<'a, R, ScopedRow, (A, B)>
			+ 'a,
			release: impl Fn(
				<RcBrand as crate::classes::Pointer>::Of<'a, A>,
			) -> RcRunExplicit<'a, R, ScopedRow, ()>
			+ 'a,
		) -> Self
		where
			A: 'a,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, B>,
			>): Member<
					crate::types::effects::bracket::BracketExplicit<
						'a,
						RcBrand,
						NodeBrand<R, ScopedRow>,
						A,
						B,
					>,
					Idx,
				>, {
			let bracket: crate::types::effects::bracket::BracketExplicit<
				'a,
				RcBrand,
				NodeBrand<R, ScopedRow>,
				A,
				B,
			> = crate::types::effects::bracket::BracketExplicit::Bracket {
				acquire: <RcBrand as crate::classes::ToDynCloneFn>::new(move |_: ()| {
					acquire.clone().into_rc_free_explicit()
				}),
				body: <RcBrand as crate::classes::ToDynCloneFn>::new(
					move |a: <RcBrand as crate::classes::Pointer>::Of<'a, A>| {
						body(a).into_rc_free_explicit()
					},
				),
				release: <RcBrand as crate::classes::ToDynCloneFn>::new(
					move |a: <RcBrand as crate::classes::Pointer>::Of<'a, A>| {
						release(a).into_rc_free_explicit()
					},
				),
			};
			let layer = <Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, B>,
			>) as Member<
				crate::types::effects::bracket::BracketExplicit<
					'a,
					RcBrand,
					NodeBrand<R, ScopedRow>,
					A,
					B,
				>,
				Idx,
			>>::inject(bracket);
			let node = Node::Scoped(layer);
			RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::wrap(node))
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	impl<'a, R, ScopedRow> RcRunExplicit<'a, R, ScopedRow, ()>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		/// Lifts a `Put` state effect into the `RcRunExplicit` program.
		/// Mirrors [`Run::put`](crate::types::effects::run::Run::put);
		/// see that method for cross-wrapper semantics. Threads
		/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
		#[document_signature]
		///
		#[document_type_parameters(
			"The state type carried by `StateBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The new state value to write.")]
		///
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Put` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		state::State,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, ()> = RcRunExplicit::put::<i32, _>(42);
		/// // The program is suspended at the Put effect; peel reveals the layer.
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn put<StateType: Clone + 'static, Idx>(s: StateType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>): Member<
					RcCoyoneda<
						'a,
						crate::brands::StateBrand<crate::brands::RcBrand, StateType>,
						(),
					>,
					Idx,
				>, {
			let effect: crate::types::effects::state::State<
				'a,
				crate::brands::RcBrand,
				StateType,
				(),
			> = crate::types::effects::state::State::Put(
				s,
				<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|_: ()| ()),
			);
			Self::lift::<crate::brands::StateBrand<crate::brands::RcBrand, StateType>, Idx>(effect)
		}

		/// Lifts a `Tell` writer effect into the `RcRunExplicit`
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
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Tell` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		writer::Writer,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<&'static str>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, ()> =
		/// 	RcRunExplicit::tell::<&'static str, _>("logged");
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn tell<LogType: Clone + 'static, Idx>(log: LogType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<RcCoyoneda<'a, crate::brands::WriterBrand<LogType>, ()>, Idx>, {
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
	impl<'a, R, ScopedRow> RcRunExplicit<'a, R, ScopedRow, bool>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		/// Lifts an `Alt` choose effect into the `RcRunExplicit`
		/// program. Mirrors
		/// [`RcRun::choose`](crate::types::effects::rc_run::RcRun::choose);
		/// see that method for cross-wrapper semantics.
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Alt` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		choose::Choose,
		/// 		rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, bool> = RcRunExplicit::choose();
		/// assert!(prog.peel().is_err());
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>): Member<
					RcCoyoneda<'a, crate::brands::ChooseBrand<crate::brands::RcBrand>, bool>,
					Idx,
				>, {
			let effect: crate::types::effects::choose::Choose<'a, crate::brands::RcBrand, bool> =
				crate::types::effects::choose::Choose::Alt(
					<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|b: bool| b),
				);
			Self::lift::<crate::brands::ChooseBrand<crate::brands::RcBrand>, Idx>(effect)
		}
	}

	// -- From<RcRun> for RcRunExplicit (Erased -> Explicit conversion) --

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, S, A> From<RcRun<R, S, A>> for RcRunExplicit<'static, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
		>): Clone,
	{
		/// Converts an [`RcRun<R, S, A>`](crate::types::effects::rc_run::RcRun)
		/// into the paired Explicit-substrate form by walking the
		/// underlying [`RcFree`](crate::types::RcFree) chain via
		/// [`peel`](RcRun::peel) and rebuilding each suspended layer
		/// through [`RcFreeExplicit::wrap`](crate::types::RcFreeExplicit).
		/// Pure values re-emerge as
		/// [`RcRunExplicit::pure`](RcRunExplicit::pure).
		///
		/// Multi-shot semantics are preserved across the conversion: the
		/// source `RcRun` carries `Rc<dyn Fn>` continuations and the
		/// resulting `RcRunExplicit` keeps the same `Rc`-shared substrate,
		/// so handlers for non-deterministic effects (e.g., `Choose`) can
		/// drive either side equivalently. O(N) in the chain depth.
		#[document_signature]
		///
		#[document_parameters("The Erased-substrate `RcRun` to convert.")]
		///
		#[document_returns("An `RcRunExplicit` carrying the same effects.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		rc_run::RcRun,
		/// 		rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::pure(42);
		/// // Both call styles work via the blanket `Into` impl.
		/// let from_style: RcRunExplicit<'static, FirstRow, Scoped, i32> = RcRunExplicit::from(rc_run);
		/// assert!(matches!(from_style.peel(), Ok(42)));
		/// let rc_run2: RcRun<FirstRow, Scoped, i32> = RcRun::pure(42);
		/// let into_style: RcRunExplicit<'static, FirstRow, Scoped, i32> = rc_run2.into();
		/// assert!(matches!(into_style.peel(), Ok(42)));
		/// ```
		fn from(rc_run: RcRun<R, S, A>) -> Self {
			match rc_run.peel() {
				Ok(a) => RcRunExplicit::pure(a),
				Err(layer) => {
					let inner = <NodeBrand<R, S> as Functor>::map(
						|run: RcRun<R, S, A>| -> RcFreeExplicit<'static, NodeBrand<R, S>, A> {
							RcRunExplicit::from(run).into_rc_free_explicit()
						},
						layer,
					);
					RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::wrap(inner))
				}
			}
		}
	}

	// -- Brand-level type class instances --
	//
	// `Functor` / `Semimonad` are not implemented at the brand level
	// because the underlying `RcFreeExplicit::bind` carries per-`A`
	// `Clone` bounds (`A: Clone`, the `F::Of<...>: Clone` projection)
	// that stable Rust's trait method signatures cannot express. The
	// `Pointed::pure` impl has no Clone bound, and the by-reference
	// `Ref*` hierarchy avoids the consume-or-clone issue by taking
	// `&self`.

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> Pointed for RcRunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Wraps a value in a pure `RcRunExplicit` computation by
		/// delegating to
		/// [`RcFreeExplicitBrand`](crate::brands::RcFreeExplicitBrand)'s
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
		#[document_returns("An `RcRunExplicit` computation that produces `a`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, _> =
		/// 	<RcRunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(42);
		/// assert_eq!(run.into_rc_free_explicit().evaluate(), 42);
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			RcRunExplicit::from_rc_free_explicit(
				<RcFreeExplicitBrand<NodeBrand<R, S>> as Pointed>::pure(a),
			)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> RefFunctor for RcRunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + RefFunctor + 'static,
		S: WrapDrop + Functor + RefFunctor + 'static,
	{
		/// Maps a function over the result of an `RcRunExplicit` by
		/// reference, delegating to
		/// [`RcFreeExplicitBrand`](crate::brands::RcFreeExplicitBrand)'s
		/// [`RefFunctor::ref_map`].
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
			"The `RcRunExplicit` computation."
		)]
		///
		#[document_returns("A new `RcRunExplicit` with the function applied to its result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run = <RcRunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(10);
		/// let mapped =
		/// 	<RcRunExplicitBrand<FirstRow, Scoped> as RefFunctor>::ref_map(|x: &i32| *x * 2, &run);
		/// assert_eq!(mapped.into_rc_free_explicit().evaluate(), 20);
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			RcRunExplicit::from_rc_free_explicit(
				<RcFreeExplicitBrand<NodeBrand<R, S>> as RefFunctor>::ref_map(func, &fa.0),
			)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> RefPointed for RcRunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Wraps a cloned value in a pure `RcRunExplicit` computation
		/// by delegating to
		/// [`RcFreeExplicitBrand`](crate::brands::RcFreeExplicitBrand)'s
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
		#[document_returns("An `RcRunExplicit` computation that produces a clone of `a`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let value = 42;
		/// let run: RcRunExplicit<'_, FirstRow, Scoped, _> =
		/// 	<RcRunExplicitBrand<FirstRow, Scoped> as RefPointed>::ref_pure(&value);
		/// assert_eq!(run.into_rc_free_explicit().evaluate(), 42);
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			RcRunExplicit::from_rc_free_explicit(
				<RcFreeExplicitBrand<NodeBrand<R, S>> as RefPointed>::ref_pure(a),
			)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> RefSemimonad for RcRunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + RefFunctor + 'static,
		S: WrapDrop + Functor + RefFunctor + 'static,
	{
		/// Sequences `RcRunExplicit` computations using a reference to
		/// the intermediate value, delegating to
		/// [`RcFreeExplicitBrand`](crate::brands::RcFreeExplicitBrand)'s
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
			"The first `RcRunExplicit` computation.",
			"The function to chain after the first computation."
		)]
		///
		#[document_returns("A new `RcRunExplicit` chaining the function after `ma`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run = <RcRunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(2);
		/// let chained =
		/// 	<RcRunExplicitBrand<FirstRow, Scoped> as RefSemimonad>::ref_bind(&run, |x: &i32| {
		/// 		<RcRunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(*x + 1)
		/// 	});
		/// assert_eq!(chained.into_rc_free_explicit().evaluate(), 3);
		/// ```
		fn ref_bind<'a, A: 'a, B: 'a>(
			ma: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			RcRunExplicit::from_rc_free_explicit(
				<RcFreeExplicitBrand<NodeBrand<R, S>> as RefSemimonad>::ref_bind(&ma.0, move |a| {
					f(a).into_rc_free_explicit()
				}),
			)
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
				RcBrand,
				RcRunExplicitBrand,
			},
			classes::{
				Pointed,
				RefCountedPointer,
				RefFunctor,
				RefPointed,
				RefSemimonad,
			},
			types::{
				RcFreeExplicit,
				effects::{
					handlers::HandlersNil,
					interpreter::ScopedContinuation,
					run_explicit::RunExplicitSpanCarrierLayer,
					scoped_dispatchers::span_dispatcher,
				},
			},
		},
		core::{
			cell::RefCell,
			marker::PhantomData,
		},
	};

	type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
	type Scoped = CNilBrand;
	type RunAlias<'a, A> = RcRunExplicit<'a, FirstRow, Scoped, A>;
	type EmptyRcRunExplicit<'a, A> = RcRunExplicit<'a, CNilBrand, CNilBrand, A>;

	fn rc_explicit_scoped_continuation<'a, Action, Final, K>(
		action: EmptyRcRunExplicit<'a, Action>,
		outer: K,
	) -> RcRunExplicitScopedContinuation<'a, CNilBrand, CNilBrand, Action, Final, K>
	where
		Action: Clone + 'a,
		Final: 'a,
		K: Fn(Action) -> EmptyRcRunExplicit<'a, Final> + 'a, {
		RcRunExplicitScopedContinuation {
			action,
			outer: <RcBrand as RefCountedPointer>::new(outer),
			result: PhantomData,
		}
	}

	#[test]
	fn from_and_into_round_trip() {
		let rc_free: RcFreeExplicit<'_, _, i32> = RcFreeExplicit::pure(42);
		let run: RunAlias<'_, i32> = RcRunExplicit::from_rc_free_explicit(rc_free);
		let _back = run.into_rc_free_explicit();
	}

	#[test]
	fn clone_branches_are_cheap() {
		let run: RunAlias<'_, _> = RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(7));
		let _branch = run.clone();
	}

	#[test]
	fn brand_pure_evaluates() {
		let run: RunAlias<'_, _> = <RcRunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(7);
		assert_eq!(run.into_rc_free_explicit().evaluate(), 7);
	}

	#[test]
	fn inherent_map_evaluates() {
		let run: RunAlias<'_, _> = RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(10));
		let mapped = run.map(|x: i32| x * 3);
		assert_eq!(mapped.into_rc_free_explicit().evaluate(), 30);
	}

	#[test]
	fn inherent_bind_evaluates() {
		let run: RunAlias<'_, _> = RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(2));
		let chained =
			run.bind(|x: i32| RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(x + 5)));
		assert_eq!(chained.into_rc_free_explicit().evaluate(), 7);
	}

	#[test]
	fn scoped_continuation_repeats_action_before_outer_continuation() {
		let events = RefCell::new(Vec::new());
		let carrier = ScopedContinuation::new(rc_explicit_scoped_continuation(
			EmptyRcRunExplicit::pure(40),
			|value| {
				events.borrow_mut().push("outer");
				EmptyRcRunExplicit::pure(value * 10)
			},
		));

		let first: EmptyRcRunExplicit<'_, i32> = carrier.clone().resume_rc(&HandlersNil);
		let second: EmptyRcRunExplicit<'_, i32> = carrier.resume_rc(&HandlersNil);

		assert_eq!(first.extract(), 400);
		assert_eq!(second.extract(), 400);
		assert_eq!(events.into_inner(), vec!["outer", "outer"]);
	}

	#[test]
	fn scoped_continuation_transforms_action_before_outer_continuation() {
		let events = RefCell::new(Vec::new());
		let carrier = ScopedContinuation::new(rc_explicit_scoped_continuation(
			EmptyRcRunExplicit::pure(40),
			|value| {
				events.borrow_mut().push("outer");
				EmptyRcRunExplicit::pure(value * 10)
			},
		));

		let result: EmptyRcRunExplicit<'_, i32> =
			carrier.resume_rc_with_post_action(&HandlersNil, |value| {
				events.borrow_mut().push("post");
				EmptyRcRunExplicit::pure(value + 1)
			});

		assert_eq!(result.extract(), 410);
		assert_eq!(events.into_inner(), vec!["post", "outer"]);
	}

	#[test]
	fn scoped_continuation_preserves_borrowed_action_value() {
		let label = String::from("borrowed-value");
		let carrier = ScopedContinuation::new(rc_explicit_scoped_continuation(
			EmptyRcRunExplicit::pure(label.as_str()),
			|value: &str| EmptyRcRunExplicit::pure(value.len()),
		));

		let result: EmptyRcRunExplicit<'_, usize> =
			carrier.resume_rc_with_post_action(&HandlersNil, EmptyRcRunExplicit::pure);

		assert_eq!(result.extract(), label.len());
	}

	#[test]
	fn span_dispatcher_repeats_carrier_layer_before_outer_continuation() {
		let events = RefCell::new(Vec::new());
		let label = String::from("borrowed-value");
		let layer = RunExplicitSpanCarrierLayer::new(
			"request",
			ScopedContinuation::new(rc_explicit_scoped_continuation(
				EmptyRcRunExplicit::pure(label.as_str()),
				|value: &str| {
					events.borrow_mut().push("outer");
					EmptyRcRunExplicit::pure(value.len())
				},
			)),
		);

		let first: EmptyRcRunExplicit<'_, usize> = span_dispatcher()
			.dispatch_rc_run_explicit_span_carrier_with_post_action(
				layer.clone(),
				&HandlersNil,
				|tag, value| {
					assert_eq!(*tag, "request");
					events.borrow_mut().push("post");
					EmptyRcRunExplicit::pure(value)
				},
			);
		let second: EmptyRcRunExplicit<'_, usize> = span_dispatcher()
			.dispatch_rc_run_explicit_span_carrier_with_post_action(
				layer,
				&HandlersNil,
				|tag, value| {
					assert_eq!(*tag, "request");
					events.borrow_mut().push("post");
					EmptyRcRunExplicit::pure(value)
				},
			);

		assert_eq!(first.extract(), label.len());
		assert_eq!(second.extract(), label.len());
		assert_eq!(events.into_inner(), vec!["post", "outer", "post", "outer"]);
	}

	#[test]
	fn brand_ref_pure_evaluates() {
		let value = 11;
		let run: RunAlias<'_, _> =
			<RcRunExplicitBrand<FirstRow, Scoped> as RefPointed>::ref_pure(&value);
		assert_eq!(run.into_rc_free_explicit().evaluate(), 11);
	}

	#[test]
	fn brand_ref_map_evaluates() {
		let run = <RcRunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(4);
		let mapped =
			<RcRunExplicitBrand<FirstRow, Scoped> as RefFunctor>::ref_map(|x: &i32| *x * 5, &run);
		assert_eq!(mapped.into_rc_free_explicit().evaluate(), 20);
	}

	#[test]
	fn brand_ref_bind_evaluates() {
		let run = <RcRunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(8);
		let chained =
			<RcRunExplicitBrand<FirstRow, Scoped> as RefSemimonad>::ref_bind(&run, |x: &i32| {
				<RcRunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(*x + 1)
			});
		assert_eq!(chained.into_rc_free_explicit().evaluate(), 9);
	}

	#[test]
	fn non_static_payload() {
		let s = String::from("hello");
		let r: &str = &s;
		let run: RcRunExplicit<'_, FirstRow, Scoped, &str> =
			RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(r));
		assert_eq!(run.into_rc_free_explicit().evaluate(), "hello");
	}

	#[test]
	fn pure_then_peel_returns_value() {
		let run: RcRunExplicit<'_, FirstRow, Scoped, i32> = RcRunExplicit::pure(42);
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
		let run: RcRunExplicit<'_, FirstRow, Scoped, i32> = RcRunExplicit::send(Node::First(layer));
		assert!(run.peel().is_err());
	}

	#[test]
	fn from_erased_round_trips_pure() {
		use crate::types::effects::rc_run::RcRun;
		let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::pure(42);
		let explicit: RcRunExplicit<'static, FirstRow, Scoped, i32> = RcRunExplicit::from(rc_run);
		assert!(matches!(explicit.peel(), Ok(42)));
	}

	#[test]
	fn from_erased_preserves_suspended_layer() {
		use crate::types::{
			Identity,
			effects::{
				coproduct::Coproduct,
				node::Node,
				rc_run::RcRun,
			},
		};
		let layer = Coproduct::inject(Identity(7));
		let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::send(Node::First(layer));
		let explicit: RcRunExplicit<'static, FirstRow, Scoped, i32> = RcRunExplicit::from(rc_run);
		assert!(explicit.peel().is_err());
	}

	#[test]
	fn ref_bind_chains_pure_value_via_clone() {
		let run: RunAlias<'_, i32> = RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(2));
		let chained = run
			.ref_bind(|x: &i32| RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(*x + 1)));
		assert_eq!(chained.into_rc_free_explicit().evaluate(), 3);
	}

	#[test]
	fn ref_map_transforms_pure_value_via_clone() {
		let run: RunAlias<'_, i32> = RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(7));
		let mapped = run.ref_map(|x: &i32| *x * 3);
		assert_eq!(mapped.into_rc_free_explicit().evaluate(), 21);
	}

	#[test]
	fn ref_pure_wraps_cloned_value() {
		let value = 42;
		let run: RunAlias<'_, i32> = RcRunExplicit::ref_pure(&value);
		assert_eq!(run.into_rc_free_explicit().evaluate(), 42);
	}
}
