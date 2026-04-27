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
				NodeBrand,
				RcFreeExplicitBrand,
				RcRunExplicitBrand,
			},
			classes::{
				Functor,
				Pointed,
				RefFunctor,
				RefPointed,
				RefSemimonad,
				WrapDrop,
			},
			impl_kind,
			kinds::*,
			types::{
				RcFree,
				RcFreeExplicit,
				effects::rc_run::RcRun,
			},
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
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, S, A> RcRunExplicit<'static, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Constructs an `RcRunExplicit<'static, R, S, A>` from the
		/// paired Erased-substrate
		/// [`RcRun<R, S, A>`](crate::types::effects::rc_run::RcRun) by
		/// delegating to [`RcRun::into_explicit`](RcRun::into_explicit).
		///
		/// Multi-shot semantics are preserved across the conversion (both
		/// sides carry `Rc<dyn Fn>` continuations); see
		/// [`RcRun::into_explicit`](RcRun::into_explicit) for details.
		#[document_signature]
		///
		#[document_parameters("The Erased-substrate `RcRun` to convert.")]
		///
		#[document_returns("An `RcRunExplicit` carrying the same effects as `rc_run`.")]
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
		/// let explicit: RcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::from_erased(rc_run);
		/// assert!(matches!(explicit.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn from_erased(rc_run: RcRun<R, S, A>) -> Self
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			rc_run.into_explicit()
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
				RcRunExplicitBrand,
			},
			classes::{
				Pointed,
				RefFunctor,
				RefPointed,
				RefSemimonad,
			},
			types::RcFreeExplicit,
		},
	};

	type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
	type Scoped = CNilBrand;
	type RunAlias<'a, A> = RcRunExplicit<'a, FirstRow, Scoped, A>;

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
		let explicit: RcRunExplicit<'static, FirstRow, Scoped, i32> =
			RcRunExplicit::from_erased(rc_run);
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
		let explicit = RcRunExplicit::from_erased(rc_run);
		assert!(explicit.peel().is_err());
	}
}
