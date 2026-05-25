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

mod boundary;
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
					},
					member::Member,
					node::Node,
					rc_run::RcRun,
				},
			},
		},
		core::ops::ControlFlow,
		fp_macros::*,
	};

	pub use super::boundary::RcRunExplicitBoundary;
	pub(crate) use super::boundary::{
		RcRunExplicitActionSuppliedScopedContinuation,
		RcRunExplicitScopedContinuation,
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

	/// Result-polymorphic same-row first-order rewrite protocol for
	/// `RcRunExplicit`.
	///
	/// The rewriter maps only the lowered effect constructor. Wrapper
	/// traversal owns recursive continuation rewriting, row projection,
	/// and row-preserving re-embedding through `RcCoyoneda`.
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect brand being rewritten.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	#[document_parameters("The result-polymorphic rewrite instance.")]
	pub trait RcRunExplicitFirstOrderRewriter<'a, EBrand, R, S>
	where
		EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static, {
		/// Rewrites one lowered first-order operation at the current
		/// branch result type while preserving its effect constructor.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters(
			"The lowered first-order operation whose continuation stays in the original row."
		)]
		#[document_returns("The rewritten operation in the same effect constructor.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run_explicit::{
		/// 			RcRunExplicit,
		/// 			RcRunExplicitFirstOrderRewriter,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct IdentityPreserve;
		///
		/// impl<'a> RcRunExplicitFirstOrderRewriter<'a, IdentityBrand, Row, CNilBrand> for IdentityPreserve {
		/// 	fn rewrite<T: Clone + 'a>(
		/// 		&self,
		/// 		effect: Identity<RcRunExplicit<'a, Row, CNilBrand, T>>,
		/// 	) -> Identity<RcRunExplicit<'a, Row, CNilBrand, T>> {
		/// 		effect
		/// 	}
		/// }
		///
		/// let prog: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
		/// let rewritten =
		/// 	prog.interpose_with_rewriter::<IdentityBrand, _, CNilBrand, _>(IdentityPreserve);
		/// let result = rewritten.handle(
		/// 	fp_library::handlers! {
		/// 		IdentityBrand: |op: Identity<RcRunExplicit<'static, Row, CNilBrand, i32>>| op.0,
		/// 	},
		/// 	fp_library::types::effects::scoped_nt(),
		/// );
		/// assert_eq!(result, 7);
		/// ```
		fn rewrite<T: Clone + 'a>(
			&self,
			effect: Apply!(
				<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcRunExplicit<'a, R, S, T>,
				>
			),
		) -> Apply!(
			<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, T>,
			>
		);
	}

	/// Result-changing first-order accumulation protocol for
	/// `RcRunExplicit`.
	///
	/// The traversal consumes matching first-order operations inside a
	/// selected action and returns the selected action value paired with
	/// an explicit accumulator. Handler-specific implementations decide
	/// how one matched operation contributes to the accumulator; the
	/// wrapper traversal owns row projection, continuation preservation,
	/// and non-matching operation re-embedding.
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect brand being accumulated.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The accumulated value type."
	)]
	#[document_parameters("The result-polymorphic accumulation instance.")]
	#[doc(hidden)]
	pub trait RcRunExplicitFirstOrderAccumulator<'a, EBrand, R, S, Acc>
	where
		EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Acc: Clone + 'a, {
		/// Produces the accumulator value for a selected action with no
		/// matching first-order operations.
		#[document_signature]
		#[document_returns("The neutral accumulated value.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// let accumulated_log: Vec<&'static str> = Vec::new();
		/// assert!(accumulated_log.is_empty());
		/// ```
		fn empty(&self) -> Acc;

		/// Consumes one lowered first-order operation after its continuation
		/// has already been recursively accumulated.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters(
			"The lowered first-order operation whose continuation now returns `(value, accumulated)`."
		)]
		#[document_returns("The accumulated program in the original row.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// let current_log = "selected ".to_string();
		/// let accumulated_suffix = "action".to_string();
		/// assert_eq!(current_log + &accumulated_suffix, "selected action");
		/// ```
		fn accumulate<T: Clone + 'a>(
			&self,
			effect: Apply!(
				<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcRunExplicit<'a, R, S, (T, Acc)>,
				>
			),
		) -> RcRunExplicit<'a, R, S, (T, Acc)>
		where
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (T, Acc)>,
			>): Clone,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (T, Acc)>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (T, Acc)>,
			>): Clone;
	}

	/// Result-changing first-order preserving accumulation protocol for
	/// `RcRunExplicit`.
	///
	/// This protocol is the preserving counterpart of
	/// [`RcRunExplicitFirstOrderAccumulator`]. The traversal walks a
	/// selected action once, accumulates matching first-order operations,
	/// and rebuilds each matched operation into the original row so an
	/// outer handler can still observe it.
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The first-order effect brand being accumulated.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The accumulated value type."
	)]
	#[document_parameters("The result-polymorphic preserving accumulation instance.")]
	#[doc(hidden)]
	pub trait RcRunExplicitFirstOrderPreservingAccumulator<'a, EBrand, R, S, Acc>
	where
		EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Acc: Clone + 'a, {
		/// Produces the accumulator value for a selected action with no
		/// matching first-order operations.
		#[document_signature]
		#[document_returns("The neutral accumulated value.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run_explicit::{
		/// 			RcRunExplicit,
		/// 			RcRunExplicitFirstOrderPreservingAccumulator,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct CountIdentity;
		///
		/// impl<'a> RcRunExplicitFirstOrderPreservingAccumulator<'a, IdentityBrand, Row, CNilBrand, usize>
		/// 	for CountIdentity
		/// {
		/// 	fn empty(&self) -> usize {
		/// 		0
		/// 	}
		///
		/// 	fn accumulate_preserving<T: Clone + 'a>(
		/// 		&self,
		/// 		effect: Identity<RcRunExplicit<'a, Row, CNilBrand, (T, usize)>>,
		/// 	) -> Identity<RcRunExplicit<'a, Row, CNilBrand, (T, usize)>> {
		/// 		Identity(effect.0.map(|(value, count)| (value, count + 1)))
		/// 	}
		/// }
		///
		/// assert_eq!(CountIdentity.empty(), 0);
		/// ```
		fn empty(&self) -> Acc;

		/// Preserves one lowered first-order operation after its
		/// continuation has already been recursively accumulated.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters(
			"The lowered first-order operation whose continuation now returns `(value, accumulated)`."
		)]
		#[document_returns("The preserved operation in the same effect constructor.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			rc_run_explicit::{
		/// 				RcRunExplicit,
		/// 				RcRunExplicitFirstOrderPreservingAccumulator,
		/// 			},
		/// 			scoped_nt,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct CountIdentity;
		///
		/// impl<'a> RcRunExplicitFirstOrderPreservingAccumulator<'a, IdentityBrand, Row, CNilBrand, usize>
		/// 	for CountIdentity
		/// {
		/// 	fn empty(&self) -> usize {
		/// 		0
		/// 	}
		///
		/// 	fn accumulate_preserving<T: Clone + 'a>(
		/// 		&self,
		/// 		effect: Identity<RcRunExplicit<'a, Row, CNilBrand, (T, usize)>>,
		/// 	) -> Identity<RcRunExplicit<'a, Row, CNilBrand, (T, usize)>> {
		/// 		Identity(effect.0.map(|(value, count)| (value, count + 1)))
		/// 	}
		/// }
		///
		/// let effect = Identity(RcRunExplicit::<'static, Row, CNilBrand, (i32, usize)>::pure((41, 0)));
		/// let preserved = CountIdentity.accumulate_preserving(effect);
		/// let result = preserved.0.handle(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<RcRunExplicit<'static, Row, CNilBrand, (i32, usize)>>| op.0,
		/// 	},
		/// 	scoped_nt(),
		/// );
		/// assert_eq!(result, (41, 1));
		/// ```
		fn accumulate_preserving<T: Clone + 'a>(
			&self,
			effect: Apply!(
				<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcRunExplicit<'a, R, S, (T, Acc)>,
				>
			),
		) -> Apply!(
			<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, (T, Acc)>,
			>
		)
		where
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (T, Acc)>,
			>): Clone,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (T, Acc)>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (T, Acc)>,
			>): Clone;
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
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
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
		/// 	handlers,
		/// 	scoped_handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		/// type Prog = RcRunExplicit<'static, FirstRow, Scoped, i32>;
		///
		/// let run: Prog = RcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let result = run.handle(
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
		/// [`Run::handle`](crate::types::effects::run::Run::handle).
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
		/// let result = prog.handle(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<RcRunExplicit<'static, FirstRow, Scoped, i32>>| op.0,
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

		/// Alias for [`handle`](RcRunExplicit::handle), kept for
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
			self.handle(handlers, scoped_handlers)
		}

		/// MonadRec-target interpreter for [`RcRunExplicit`]. Mirrors
		/// [`Run::handle_rec`](crate::types::effects::run::Run::handle_rec);
		/// see that method's docs for the handler shape, loop body,
		/// and stack-safety guarantee. `RcRunExplicit` differences:
		/// the per-`peel` `A: Clone` and substrate-`Of<...>: Clone`
		/// bounds propagate (matching [`RcRunExplicit::handle`]).
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
		/// let result: Thunk<'static, i32> = prog.handle_rec::<ThunkBrand>(handlers! {
		/// 	IdentityBrand: |op: Identity<Thunk<'static, RcRunExplicit<'static, FirstRow, Scoped, i32>>>| op.0,
		/// }, fp_library::types::effects::scoped_nt());
		/// assert_eq!(result.evaluate(), 42);
		/// ```
		#[inline]
		pub fn handle_rec<MBrand>(
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

		/// Alias for [`handle_rec`](RcRunExplicit::handle_rec).
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
			self.handle_rec::<MBrand>(handlers, scoped_handlers)
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
		/// 	classes::ToDynCloneFn,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::{
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 			rc_run_explicit::RcRunExplicit,
		/// 			span::Span,
		/// 		},
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
		///
		/// let action: RcRunExplicit<'static, CNilBrand, ScopedRow, i32> = RcRunExplicit::pure(7);
		/// let layer = Coproduct::Inl(Span::Span {
		/// 	tag: "request",
		/// 	action: <RcBrand as ToDynCloneFn>::new(move |_: ()| action.clone().into_rc_free_explicit()),
		/// });
		/// let prog: RcRunExplicit<'static, CNilBrand, ScopedRow, i32> =
		/// 	RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::wrap(Node::Scoped(layer)));
		///
		/// let narrowed: RcRunExplicit<'static, CNilBrand, CNilBrand, i32> = prog
		/// 	.handle_scoped_with::<SpanBrand<RcBrand, &'static str>, _, CNilBrand>(|span| match span {
		/// 		Span::Span {
		/// 			tag,
		/// 			action,
		/// 		} => {
		/// 			assert_eq!(tag, "request");
		/// 			action(())
		/// 		}
		/// 	});
		/// assert_eq!(narrowed.extract(), 7);
		/// ```
		#[inline]
		pub fn handle_scoped_with<SBrand, Idx, SMinusE>(
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
		#[document_parameters("The handler wrapped in a refcounted pointer.")]
		///
		#[document_returns("An `RcRunExplicit` program in the narrowed scoped row `SMinusE`.")]
		///
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// // The public handle_scoped_with method wraps the handler
		/// // and then uses the same scoped-row narrowing path as this helper.
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ToDynCloneFn,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::{
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 			rc_run_explicit::RcRunExplicit,
		/// 			span::Span,
		/// 		},
		/// 	},
		/// };
		///
		/// type ScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
		///
		/// let action: RcRunExplicit<'static, CNilBrand, ScopedRow, i32> = RcRunExplicit::pure(7);
		/// let layer = Coproduct::Inl(Span::Span {
		/// 	tag: "request",
		/// 	action: <RcBrand as ToDynCloneFn>::new(move |_: ()| action.clone().into_rc_free_explicit()),
		/// });
		/// let prog: RcRunExplicit<'static, CNilBrand, ScopedRow, i32> =
		/// 	RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::wrap(Node::Scoped(layer)));
		/// let narrowed: RcRunExplicit<'static, CNilBrand, CNilBrand, i32> = prog
		/// 	.handle_scoped_with::<SpanBrand<RcBrand, &'static str>, _, CNilBrand>(|span| match span {
		/// 		Span::Span {
		/// 			tag,
		/// 			action,
		/// 		} => {
		/// 			assert_eq!(tag, "request");
		/// 			action(())
		/// 		}
		/// 	});
		/// assert_eq!(narrowed.extract(), 7);
		/// ```
		#[inline]
		fn handle_scoped_with_shared<SBrand, Idx, SMinusE, F>(
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
								.handle_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
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
								move |inner: RcRunExplicit<'a, R, S, A>| {
									inner
										.handle_scoped_with_shared::<SBrand, Idx, SMinusE, F>(
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
		/// [`Run::handle_with`](crate::types::effects::run::Run::handle_with)
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
		/// 	.handle_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<RcRunExplicit<'static, EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		pub fn handle_with<EBrand, Idx, RMinusE>(
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
			self.handle_with_shared::<EBrand, Idx, RMinusE, _>(handler)
		}

		/// Inner pipeline-narrowing implementation, parameterised
		/// over the concrete handler closure type `F`. The public
		/// [`handle_with`](RcRunExplicit::handle_with) wraps
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
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
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
		/// // Exercised internally by RcRunExplicit::handle_with.
		/// let prog: RcRunExplicit<'static, FullRow, CNilBrand, i32> =
		/// 	RcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
		/// let narrowed: RcRunExplicit<'static, EmptyRow, CNilBrand, i32> = prog
		/// 	.handle_with::<IdentityBrand, _, EmptyRow>(
		/// 		|op: Identity<RcRunExplicit<'static, EmptyRow, CNilBrand, i32>>| op.0,
		/// 	);
		/// assert_eq!(narrowed.extract(), 42);
		/// ```
		#[inline]
		fn handle_with_shared<EBrand, Idx, RMinusE, F>(
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
								move |inner: RcRunExplicit<'a, R, S, A>| {
									inner
										.handle_with_shared::<EBrand, Idx, RMinusE, F>(
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
								.handle_with_shared::<EBrand, Idx, RMinusE, F>(
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
		/// Unlike [`handle_with`](RcRunExplicit::handle_with),
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
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
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

		/// Same-row first-order rewrite primitive.
		///
		/// Walks this program, projects each first-order dispatch
		/// against `EBrand`, rewrites the lowered effect layer with
		/// `rewriter`, and re-embeds the operation in the original row.
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
		/// 		effects::rc_run_explicit::{
		/// 			RcRunExplicit,
		/// 			RcRunExplicitFirstOrderRewriter,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = RcRunExplicit<'static, Row, CNilBrand, i32>;
		///
		/// struct IdentityPreserve;
		///
		/// impl<'a> RcRunExplicitFirstOrderRewriter<'a, IdentityBrand, Row, CNilBrand> for IdentityPreserve {
		/// 	fn rewrite<T: Clone + 'a>(
		/// 		&self,
		/// 		op: Identity<RcRunExplicit<'a, Row, CNilBrand, T>>,
		/// 	) -> Identity<RcRunExplicit<'a, Row, CNilBrand, T>> {
		/// 		op
		/// 	}
		/// }
		///
		/// let prog: Prog = RcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
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
			rewriter: impl RcRunExplicitFirstOrderRewriter<'a, EBrand, R, S> + 'a,
		) -> RcRunExplicit<'a, R, S, A>
		where
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, A>,
			>): Member<
					RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, S, A>,
									>
								),
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Member<RcCoyoneda<'a, EBrand, RcFreeExplicit<'a, NodeBrand<R, S>, A>>, Idx>,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
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
			let rewriter = <RcBrand as RefCountedPointer>::new(rewriter);
			self.interpose_with_rewriter_shared::<EBrand, Idx, RMinusE, EmbedIndices, _>(rewriter)
		}

		/// Inner shared implementation of
		/// [`interpose_with_rewriter`](RcRunExplicit::interpose_with_rewriter).
		#[document_signature]
		#[document_type_parameters(
			"The brand of the effect to rewrite.",
			"The type-level position witness for `EBrand` in the row.",
			"The narrowed row brand used while projecting the matched effect.",
			"The HList witness for embedding the narrowed row back into the original row.",
			"The concrete result-polymorphic rewriter type."
		)]
		#[document_parameters("The Rc-wrapped rewriter value.")]
		#[document_returns(
			"A new program in the same row with all matched-effect dispatches rewritten."
		)]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::rc_run_explicit::{
		/// 			RcRunExplicit,
		/// 			RcRunExplicitFirstOrderRewriter,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Prog = RcRunExplicit<'static, Row, CNilBrand, i32>;
		///
		/// struct IdentityPreserve;
		///
		/// impl<'a> RcRunExplicitFirstOrderRewriter<'a, IdentityBrand, Row, CNilBrand> for IdentityPreserve {
		/// 	fn rewrite<T: Clone + 'a>(
		/// 		&self,
		/// 		op: Identity<RcRunExplicit<'a, Row, CNilBrand, T>>,
		/// 	) -> Identity<RcRunExplicit<'a, Row, CNilBrand, T>> {
		/// 		op
		/// 	}
		/// }
		///
		/// let prog: Prog = RcRunExplicit::lift::<IdentityBrand, _>(Identity(42));
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
		fn interpose_with_rewriter_shared<EBrand, Idx, RMinusE, EmbedIndices, P>(
			self,
			rewriter: <RcBrand as RefCountedPointer>::Of<'a, P>,
		) -> RcRunExplicit<'a, R, S, A>
		where
			P: RcRunExplicitFirstOrderRewriter<'a, EBrand, R, S> + 'a,
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, A>,
			>): Member<
					RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, S, A>,
									>
								),
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Member<RcCoyoneda<'a, EBrand, RcFreeExplicit<'a, NodeBrand<R, S>, A>>, Idx>,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
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
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, S, A>,
					>
				) as Member<
					RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
					Idx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let lowered = coyo.lower_ref();
						let r_for_recurse = rewriter.clone();
						let mapped = <EBrand as Functor>::map(
							move |inner: RcRunExplicit<'a, R, S, A>| {
								inner
									.interpose_with_rewriter_shared::<EBrand, Idx, RMinusE, EmbedIndices, P>(
										r_for_recurse.clone(),
									)
							},
							lowered,
						);
						let rewritten = (*rewriter).rewrite(mapped);
						let rewritten_free = <EBrand as Functor>::map(
							RcRunExplicit::into_rc_free_explicit,
							rewritten,
						);
						let coyo: RcCoyoneda<'a, EBrand, RcFreeExplicit<'a, NodeBrand<R, S>, A>> =
							RcCoyoneda::lift(rewritten_free);
						let layer_back = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'a,
								RcFreeExplicit<'a, NodeBrand<R, S>, A>,
							>) as Member<
							RcCoyoneda<'a, EBrand, RcFreeExplicit<'a, NodeBrand<R, S>, A>>,
							Idx,
						>>::inject(coyo);
						RcRunExplicit::from_rc_free_explicit(
							RcFreeExplicit::<'a, NodeBrand<R, S>, A>::wrap(Node::First(layer_back)),
						)
					}
					Err(rest) => {
						let r_for_recurse = rewriter.clone();
						let mapped_rest = <RMinusE as Functor>::map(
							move |inner: RcRunExplicit<'a, R, S, A>| {
								inner
									.interpose_with_rewriter_shared::<EBrand, Idx, RMinusE, EmbedIndices, P>(
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
					let r_for_recurse = rewriter.clone();
					let mapped_free = <S as Functor>::map(
						move |inner: RcRunExplicit<'a, R, S, A>| {
							inner
								.interpose_with_rewriter_shared::<EBrand, Idx, RMinusE, EmbedIndices, P>(
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

		/// Accumulates matching first-order operations inside a selected
		/// action while preserving the original row.
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
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// let action_value = 7;
		/// let accumulated_log = "inner".to_string();
		/// assert_eq!((action_value, accumulated_log), (7, "inner".to_string()));
		/// ```
		#[inline]
		#[doc(hidden)]
		pub fn accumulate_with_first_order<EBrand, Idx, RMinusE, EmbedIndices, Acc>(
			self,
			accumulator: impl RcRunExplicitFirstOrderAccumulator<'a, EBrand, R, S, Acc> + 'a,
		) -> RcRunExplicit<'a, R, S, (A, Acc)>
		where
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Acc: Clone + 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, A>,
			>): Member<
					RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, S, A>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
				>),
					EmbedIndices,
				>, {
			let accumulator = <RcBrand as RefCountedPointer>::new(accumulator);
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
		#[document_parameters("The Rc-wrapped first-order accumulation instance.")]
		#[document_returns("A program that returns the action value and accumulated value.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// let action_value = 7;
		/// let accumulated_log = "inner".to_string();
		/// assert_eq!((action_value, accumulated_log), (7, "inner".to_string()));
		/// ```
		#[inline]
		#[doc(hidden)]
		pub fn accumulate_with_first_order_shared<EBrand, Idx, RMinusE, EmbedIndices, Acc, P>(
			self,
			accumulator: <RcBrand as RefCountedPointer>::Of<'a, P>,
		) -> RcRunExplicit<'a, R, S, (A, Acc)>
		where
			P: RcRunExplicitFirstOrderAccumulator<'a, EBrand, R, S, Acc> + 'a,
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Acc: Clone + 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, A>,
			>): Member<
					RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, S, A>,
									>
								),
				>,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
				>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => RcRunExplicit::pure((a, (*accumulator).empty())),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, S, A>,
					>
				) as Member<
					RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
					Idx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let lowered = coyo.lower_ref();
						let a_for_recurse = accumulator.clone();
						let mapped = <EBrand as Functor>::map(
							move |inner: RcRunExplicit<'a, R, S, A>| {
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
						let mapped_rest = <RMinusE as Functor>::map(
							move |inner: RcRunExplicit<'a, R, S, A>| {
								inner
									.accumulate_with_first_order_shared::<EBrand, Idx, RMinusE, EmbedIndices, Acc, P>(
										a_for_recurse.clone(),
									)
									.into_rc_free_explicit()
							},
							rest,
						);
						let layer_back = mapped_rest.embed();
						RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::<
							'a,
							NodeBrand<R, S>,
							(A, Acc),
						>::wrap(Node::First(layer_back)))
					}
				},
				Err(Node::Scoped(layer)) => {
					let a_for_recurse = accumulator.clone();
					let mapped_free = <S as Functor>::map(
						move |inner: RcRunExplicit<'a, R, S, A>| {
							inner
								.accumulate_with_first_order_shared::<EBrand, Idx, RMinusE, EmbedIndices, Acc, P>(
									a_for_recurse.clone(),
								)
								.into_rc_free_explicit()
						},
						layer,
					);
					RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::<
						'a,
						NodeBrand<R, S>,
						(A, Acc),
					>::wrap(Node::Scoped(mapped_free)))
				}
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
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			rc_run_explicit::{
		/// 				RcRunExplicit,
		/// 				RcRunExplicitFirstOrderPreservingAccumulator,
		/// 			},
		/// 			scoped_nt,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct CountIdentity;
		///
		/// impl<'a> RcRunExplicitFirstOrderPreservingAccumulator<'a, IdentityBrand, Row, CNilBrand, usize>
		/// 	for CountIdentity
		/// {
		/// 	fn empty(&self) -> usize {
		/// 		0
		/// 	}
		///
		/// 	fn accumulate_preserving<T: Clone + 'a>(
		/// 		&self,
		/// 		effect: Identity<RcRunExplicit<'a, Row, CNilBrand, (T, usize)>>,
		/// 	) -> Identity<RcRunExplicit<'a, Row, CNilBrand, (T, usize)>> {
		/// 		Identity(effect.0.map(|(value, count)| (value, count + 1)))
		/// 	}
		/// }
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::lift::<IdentityBrand, _>(Identity(41));
		/// let preserved = program
		/// 	.accumulate_preserving_with_first_order::<IdentityBrand, _, CNilBrand, _, usize>(
		/// 		CountIdentity,
		/// 	);
		/// let result = preserved.handle(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<RcRunExplicit<'static, Row, CNilBrand, (i32, usize)>>| op.0,
		/// 	},
		/// 	scoped_nt(),
		/// );
		/// assert_eq!(result, (41, 1));
		/// ```
		#[inline]
		#[doc(hidden)]
		pub fn accumulate_preserving_with_first_order<EBrand, Idx, RMinusE, EmbedIndices, Acc>(
			self,
			accumulator: impl RcRunExplicitFirstOrderPreservingAccumulator<'a, EBrand, R, S, Acc> + 'a,
		) -> RcRunExplicit<'a, R, S, (A, Acc)>
		where
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Acc: Clone + 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone
				+ Member<RcCoyoneda<'a, EBrand, RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>>, Idx>,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, A>,
			>): Member<
					RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, S, A>,
									>
								),
				>,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
				>),
					EmbedIndices,
				>, {
			let accumulator = <RcBrand as RefCountedPointer>::new(accumulator);
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
		#[document_parameters("The Rc-wrapped first-order preserving accumulation instance.")]
		#[document_returns("A program that returns the action value and accumulated value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	handlers,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			rc_run_explicit::{
		/// 				RcRunExplicit,
		/// 				RcRunExplicitFirstOrderPreservingAccumulator,
		/// 			},
		/// 			scoped_nt,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
		///
		/// struct CountIdentity;
		///
		/// impl<'a> RcRunExplicitFirstOrderPreservingAccumulator<'a, IdentityBrand, Row, CNilBrand, usize>
		/// 	for CountIdentity
		/// {
		/// 	fn empty(&self) -> usize {
		/// 		0
		/// 	}
		///
		/// 	fn accumulate_preserving<T: Clone + 'a>(
		/// 		&self,
		/// 		effect: Identity<RcRunExplicit<'a, Row, CNilBrand, (T, usize)>>,
		/// 	) -> Identity<RcRunExplicit<'a, Row, CNilBrand, (T, usize)>> {
		/// 		Identity(effect.0.map(|(value, count)| (value, count + 1)))
		/// 	}
		/// }
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::lift::<IdentityBrand, _>(Identity(41));
		/// let accumulator = std::rc::Rc::new(CountIdentity);
		/// let preserved = program
		/// 	.accumulate_preserving_with_first_order_shared::<IdentityBrand, _, CNilBrand, _, usize, _>(
		/// 		accumulator,
		/// 	);
		/// let result = preserved.handle(
		/// 	handlers! {
		/// 		IdentityBrand: |op: Identity<RcRunExplicit<'static, Row, CNilBrand, (i32, usize)>>| op.0,
		/// 	},
		/// 	scoped_nt(),
		/// );
		/// assert_eq!(result, (41, 1));
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
			accumulator: <RcBrand as RefCountedPointer>::Of<'a, P>,
		) -> RcRunExplicit<'a, R, S, (A, Acc)>
		where
			P: RcRunExplicitFirstOrderPreservingAccumulator<'a, EBrand, R, S, Acc> + 'a,
			A: Clone,
			EBrand: Kind_cdc7cd43dac7585f + Functor + 'static,
			RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Acc: Clone + 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone
				+ Member<RcCoyoneda<'a, EBrand, RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>>, Idx>,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, A>,
			>): Member<
					RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, S, A>,
									>
								),
				>,
			Apply!(<EBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): Clone,
			Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
			>): CoproductEmbedder<
					Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
				>),
					EmbedIndices,
				>, {
			match self.peel() {
				Ok(a) => RcRunExplicit::pure((a, (*accumulator).empty())),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, S, A>,
					>
				) as Member<
					RcCoyoneda<'a, EBrand, RcRunExplicit<'a, R, S, A>>,
					Idx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let lowered = coyo.lower_ref();
						let a_for_recurse = accumulator.clone();
						let mapped = <EBrand as Functor>::map(
							move |inner: RcRunExplicit<'a, R, S, A>| {
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
						let preserved_free = <EBrand as Functor>::map(
							RcRunExplicit::into_rc_free_explicit,
							preserved,
						);
						let coyo = RcCoyoneda::lift(preserved_free);
						let layer_back = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'a,
								RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>,
							>) as Member<
							RcCoyoneda<'a, EBrand, RcFreeExplicit<'a, NodeBrand<R, S>, (A, Acc)>>,
							Idx,
						>>::inject(coyo);
						RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::<
							'a,
							NodeBrand<R, S>,
							(A, Acc),
						>::wrap(Node::First(layer_back)))
					}
					Err(rest) => {
						let a_for_recurse = accumulator.clone();
						let mapped_rest = <RMinusE as Functor>::map(
							move |inner: RcRunExplicit<'a, R, S, A>| {
								inner
									.accumulate_preserving_with_first_order_shared::<
										EBrand,
										Idx,
										RMinusE,
										EmbedIndices,
										Acc,
										P,
									>(a_for_recurse.clone())
									.into_rc_free_explicit()
							},
							rest,
						);
						let layer_back = mapped_rest.embed();
						RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::<
							'a,
							NodeBrand<R, S>,
							(A, Acc),
						>::wrap(Node::First(layer_back)))
					}
				},
				Err(Node::Scoped(layer)) => {
					let a_for_recurse = accumulator.clone();
					let mapped_free = <S as Functor>::map(
						move |inner: RcRunExplicit<'a, R, S, A>| {
							inner
								.accumulate_preserving_with_first_order_shared::<
									EBrand,
									Idx,
									RMinusE,
									EmbedIndices,
									Acc,
									P,
								>(a_for_recurse.clone())
								.into_rc_free_explicit()
						},
						layer,
					);
					RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::<
						'a,
						NodeBrand<R, S>,
						(A, Acc),
					>::wrap(Node::Scoped(mapped_free)))
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
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
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
mod tests;
