//! Explicit-substrate Run program over [`FreeExplicit`](crate::types::FreeExplicit)
//! and a dual-row [`NodeBrand`](crate::brands::NodeBrand).
//!
//! `RunExplicit<'a, R, S, A>` is the user-facing wrapper for the Explicit
//! Run-style effect computation:
//!
//! ```text
//! RunExplicit<'a, R, S, A> = FreeExplicit<'a, NodeBrand<R, S>, A>
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
//! `RunExplicit` is the Explicit counterpart of
//! [`Run`](crate::types::effects::run::Run). The Explicit substrate is
//! single-shot, keeps the functor structure as a concrete recursive enum
//! (no `Box<dyn Any>` erasure), supports non-`'static` payloads, and has
//! O(N) [`bind`](crate::types::FreeExplicit::bind) on left-associated
//! chains. Its brand exposes API via Brand-dispatched type classes, so
//! programs written against generic [`Functor`](crate::classes::Functor)
//! / [`Pointed`](crate::classes::Pointed) /
//! [`Semimonad`](crate::classes::Semimonad) bounds work without naming
//! `RunExplicit` directly.
//!
//! ## Brand-level coverage
//!
//! [`RunExplicitBrand`](crate::brands::RunExplicitBrand) implements
//! [`Functor`](crate::classes::Functor),
//! [`Pointed`](crate::classes::Pointed),
//! [`Semimonad`](crate::classes::Semimonad),
//! [`RefFunctor`](crate::classes::RefFunctor),
//! [`RefPointed`](crate::classes::RefPointed), and
//! [`RefSemimonad`](crate::classes::RefSemimonad) by delegating to
//! [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s impls.
//! [`Monad`](crate::classes::Monad) and
//! [`RefMonad`](crate::classes::RefMonad) are not reachable because the
//! [`Monad`](crate::classes::Monad) blanket impl requires
//! [`Applicative`](crate::classes::Applicative), which
//! [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand) deliberately
//! does not implement. The
//! [`Ref`](crate::classes::RefFunctor) hierarchy is bounded by
//! `R: RefFunctor, S: RefFunctor`; the canonical
//! [`CoyonedaBrand`](crate::brands::CoyonedaBrand)-wrapped Run row does
//! not satisfy that bound, so brand-level
//! [`Ref`](crate::classes::RefFunctor) dispatch is reachable only via
//! synthetic rows whose brands carry their own
//! [`RefFunctor`](crate::classes::RefFunctor) impls (e.g.,
//! `CoproductBrand<IdentityBrand, CNilBrand>`).

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				FreeExplicitBrand,
				NodeBrand,
				RunExplicitBrand,
			},
			classes::{
				Functor,
				Pointed,
				RefFunctor,
				RefPointed,
				RefSemimonad,
				Semimonad,
				WrapDrop,
			},
			impl_kind,
			kinds::*,
			types::{
				FreeExplicit,
				effects::run::Run,
			},
		},
		fp_macros::*,
	};

	/// Explicit-substrate Run program: a thin wrapper over
	/// [`FreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::FreeExplicit).
	///
	/// The wrapper exists so user-facing API can be expressed without
	/// leaking the underlying [`FreeExplicit`](crate::types::FreeExplicit)
	/// representation. It is a tuple struct over the inner
	/// [`FreeExplicit`](crate::types::FreeExplicit); converting back via
	/// [`into_free_explicit`](RunExplicit::into_free_explicit) is a
	/// zero-cost move.
	#[document_type_parameters(
		"The lifetime that bounds the payload and the row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand (typically `CNilBrand` for first-order-only programs).",
		"The result type."
	)]
	pub struct RunExplicit<'a, R, S, A>(FreeExplicit<'a, NodeBrand<R, S>, A>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a;

	impl_kind! {
		impl<R: WrapDrop + Functor + 'static, S: WrapDrop + Functor + 'static>
			for RunExplicitBrand<R, S> {
			type Of<'a, A: 'a>: 'a = RunExplicit<'a, R, S, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RunExplicit` instance.")]
	impl<'a, R, S, A: 'a> RunExplicit<'a, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Wraps a
		/// [`FreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::FreeExplicit)
		/// as a `RunExplicit<'a, R, S, A>`. Zero-cost.
		#[document_signature]
		///
		#[document_parameters("The underlying `FreeExplicit` computation.")]
		///
		#[document_returns("A `RunExplicit` wrapping `free`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let free: FreeExplicit<'_, NodeBrand<FirstRow, Scoped>, i32> = FreeExplicit::pure(7);
		/// let run: RunExplicit<'_, FirstRow, Scoped, i32> = RunExplicit::from_free_explicit(free);
		/// assert_eq!(run.into_free_explicit().evaluate(), 7);
		/// ```
		#[inline]
		pub fn from_free_explicit(free: FreeExplicit<'a, NodeBrand<R, S>, A>) -> Self {
			RunExplicit(free)
		}

		/// Unwraps a `RunExplicit<'a, R, S, A>` to its underlying
		/// [`FreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::FreeExplicit).
		/// Zero-cost.
		#[document_signature]
		///
		#[document_returns("The underlying `FreeExplicit` computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	RunExplicit::from_free_explicit(FreeExplicit::pure(7));
		/// let free: FreeExplicit<'_, NodeBrand<FirstRow, Scoped>, i32> = run.into_free_explicit();
		/// assert_eq!(free.evaluate(), 7);
		/// ```
		#[inline]
		pub fn into_free_explicit(self) -> FreeExplicit<'a, NodeBrand<R, S>, A> {
			self.0
		}

		/// Wraps a value in a pure `RunExplicit` computation. Delegates
		/// to [`FreeExplicit::pure`](crate::types::FreeExplicit).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A `RunExplicit` computation that produces `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RunExplicit<'_, FirstRow, Scoped, i32> = RunExplicit::pure(42);
		/// assert_eq!(run.into_free_explicit().evaluate(), 42);
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self {
			RunExplicit::from_free_explicit(FreeExplicit::pure(a))
		}

		/// Decomposes this `RunExplicit` computation into one step.
		/// Returns `Ok(a)` for a pure value or `Err(layer)` carrying
		/// the next `RunExplicit` continuation in a
		/// [`Node`](crate::types::effects::node::Node) layer.
		/// Walks the `FreeExplicitView` from the underlying substrate.
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` for a pure result, or `Err(layer)` carrying the next `RunExplicit` step."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RunExplicit<'_, FirstRow, Scoped, i32> = RunExplicit::pure(7);
		/// assert!(matches!(run.peel(), Ok(7)));
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "Return type encodes Result<A, NodeBrand<R, S>::Of<'a, RunExplicit<'a, R, S, A>>>; the GAT projection is structurally complex but cannot be aliased without losing the projection link the wrapper depends on."
		)]
		pub fn peel(
			self
		) -> Result<
			A,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, RunExplicit<'a, R, S, A>>),
		> {
			match self.0.to_view() {
				crate::types::FreeExplicitView::Pure(a) => Ok(a),
				crate::types::FreeExplicitView::Wrap(node) => {
					let mapped = <NodeBrand<R, S> as Functor>::map(
						|boxed: Box<FreeExplicit<'a, NodeBrand<R, S>, A>>| -> RunExplicit<'a, R, S, A> {
							RunExplicit::from_free_explicit(*boxed)
						},
						node,
					);
					Err(mapped)
				}
			}
		}

		/// Lifts a [`Node`](crate::types::effects::node::Node) dispatch
		/// layer into the `RunExplicit` program. The `node` argument
		/// is the
		/// [`NodeBrand<R, S>`](crate::brands::NodeBrand)
		/// `Of<'a, A>` projection; `send` wraps it via
		/// [`FreeExplicit::wrap`](crate::types::FreeExplicit) after
		/// promoting each `A` into a boxed pure `FreeExplicit`. The
		/// `Node`-projection signature is symmetric across all six
		/// Run wrappers; see
		/// [`Run::send`](crate::types::effects::run::Run::send) for the
		/// rationale.
		#[document_signature]
		///
		#[document_parameters("The Node dispatch layer carrying the effect operation.")]
		///
		#[document_returns(
			"A `RunExplicit` computation that performs the effect and returns its result."
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
		/// 			run_explicit::RunExplicit,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let layer = Coproduct::inject(Identity(7));
		/// let run: RunExplicit<'_, FirstRow, Scoped, i32> = RunExplicit::send(Node::First(layer));
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
				|a: A| -> Box<FreeExplicit<'a, NodeBrand<R, S>, A>> {
					Box::new(FreeExplicit::pure(a))
				},
				node,
			);
			RunExplicit::from_free_explicit(FreeExplicit::wrap(mapped))
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, S, A> RunExplicit<'static, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Constructs a `RunExplicit<'static, R, S, A>` from the paired
		/// Erased-substrate [`Run<R, S, A>`](crate::types::effects::run::Run)
		/// by delegating to [`Run::into_explicit`](Run::into_explicit).
		///
		/// This is the constructor-style spelling of the same conversion
		/// `run.into_explicit()` performs as a method. Walking the chain
		/// is O(N) in depth; see [`Run::into_explicit`](Run::into_explicit)
		/// for the depth analysis.
		#[document_signature]
		///
		#[document_parameters("The Erased-substrate `Run` to convert.")]
		///
		#[document_returns("A `RunExplicit` carrying the same effects as `run`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> = Run::pure(42);
		/// let explicit: RunExplicit<'static, FirstRow, Scoped, i32> = RunExplicit::from_erased(run);
		/// assert!(matches!(explicit.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn from_erased(run: Run<R, S, A>) -> Self {
			run.into_explicit()
		}
	}

	// -- Brand-level type class instances --
	//
	// Each impl converts the wrapper to its underlying `FreeExplicit`,
	// dispatches through `FreeExplicitBrand<NodeBrand<R, S>>`, and
	// re-wraps the result. `Monad` / `RefMonad` are not implemented:
	// the blanket impl requires `Applicative` / `RefApplicative`, which
	// `FreeExplicitBrand` deliberately does not provide (see
	// `free_explicit.rs` lines 369-388).

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> Functor for RunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Maps a function over the result of a `RunExplicit` computation
		/// by delegating to
		/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s
		/// [`Functor::map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the row brands.",
			"The original result type.",
			"The new result type."
		)]
		///
		#[document_parameters(
			"The function to apply to the result.",
			"The `RunExplicit` computation."
		)]
		///
		#[document_returns("A new `RunExplicit` with the function applied to its result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(10);
		/// let mapped = <RunExplicitBrand<FirstRow, Scoped> as Functor>::map(|x: i32| x * 2, run);
		/// assert_eq!(mapped.into_free_explicit().evaluate(), 20);
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			RunExplicit::from_free_explicit(<FreeExplicitBrand<NodeBrand<R, S>> as Functor>::map(
				f,
				fa.into_free_explicit(),
			))
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> Pointed for RunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Wraps a value in a pure `RunExplicit` computation by
		/// delegating to
		/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s
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
		#[document_returns("A `RunExplicit` computation that produces `a`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: RunExplicit<'_, FirstRow, Scoped, _> =
		/// 	<RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(42);
		/// assert_eq!(run.into_free_explicit().evaluate(), 42);
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			RunExplicit::from_free_explicit(<FreeExplicitBrand<NodeBrand<R, S>> as Pointed>::pure(
				a,
			))
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> Semimonad for RunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Sequences `RunExplicit` computations by delegating to
		/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s
		/// [`Semimonad::bind`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the row brands.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first `RunExplicit` computation.",
			"The function to chain after the first computation."
		)]
		///
		#[document_returns("A new `RunExplicit` chaining the function after `ma`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(2);
		/// let chained = <RunExplicitBrand<FirstRow, Scoped> as Semimonad>::bind(run, |x: i32| {
		/// 	<RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(x + 1)
		/// });
		/// assert_eq!(chained.into_free_explicit().evaluate(), 3);
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			RunExplicit::from_free_explicit(
				<FreeExplicitBrand<NodeBrand<R, S>> as Semimonad>::bind(
					ma.into_free_explicit(),
					move |a| func(a).into_free_explicit(),
				),
			)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> RefFunctor for RunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + RefFunctor + 'static,
		S: WrapDrop + Functor + RefFunctor + 'static,
	{
		/// Maps a function over the result of a `RunExplicit` computation
		/// by reference, delegating to
		/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s
		/// [`RefFunctor::ref_map`].
		///
		/// Note: the canonical Run row using
		/// [`CoyonedaBrand`](crate::brands::CoyonedaBrand)-wrapped
		/// effects does not satisfy [`RefFunctor`] today, so this impl is
		/// reachable only for synthetic rows whose brands implement
		/// [`RefFunctor`] (e.g., `CoproductBrand<IdentityBrand, CNilBrand>`).
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
			"The `RunExplicit` computation."
		)]
		///
		#[document_returns("A new `RunExplicit` with the function applied to its result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(10);
		/// let mapped =
		/// 	<RunExplicitBrand<FirstRow, Scoped> as RefFunctor>::ref_map(|x: &i32| *x * 2, &run);
		/// assert_eq!(mapped.into_free_explicit().evaluate(), 20);
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			RunExplicit::from_free_explicit(
				<FreeExplicitBrand<NodeBrand<R, S>> as RefFunctor>::ref_map(func, &fa.0),
			)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> RefPointed for RunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Wraps a cloned value in a pure `RunExplicit` computation by
		/// delegating to
		/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s
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
		#[document_returns("A `RunExplicit` computation that produces a clone of `a`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let value = 42;
		/// let run: RunExplicit<'_, FirstRow, Scoped, _> =
		/// 	<RunExplicitBrand<FirstRow, Scoped> as RefPointed>::ref_pure(&value);
		/// assert_eq!(run.into_free_explicit().evaluate(), 42);
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			RunExplicit::from_free_explicit(
				<FreeExplicitBrand<NodeBrand<R, S>> as RefPointed>::ref_pure(a),
			)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> RefSemimonad for RunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + RefFunctor + 'static,
		S: WrapDrop + Functor + RefFunctor + 'static,
	{
		/// Sequences `RunExplicit` computations using a reference to the
		/// intermediate value, delegating to
		/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s
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
			"The first `RunExplicit` computation.",
			"The function to chain after the first computation."
		)]
		///
		#[document_returns("A new `RunExplicit` chaining the function after `ma`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(2);
		/// let chained =
		/// 	<RunExplicitBrand<FirstRow, Scoped> as RefSemimonad>::ref_bind(&run, |x: &i32| {
		/// 		<RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(*x + 1)
		/// 	});
		/// assert_eq!(chained.into_free_explicit().evaluate(), 3);
		/// ```
		fn ref_bind<'a, A: 'a, B: 'a>(
			ma: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			RunExplicit::from_free_explicit(
				<FreeExplicitBrand<NodeBrand<R, S>> as RefSemimonad>::ref_bind(&ma.0, move |a| {
					f(a).into_free_explicit()
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
				RunExplicitBrand,
			},
			classes::{
				Functor,
				Pointed,
				RefFunctor,
				RefPointed,
				RefSemimonad,
				Semimonad,
			},
			types::FreeExplicit,
		},
	};

	type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
	type Scoped = CNilBrand;
	type RunAlias<'a, A> = RunExplicit<'a, FirstRow, Scoped, A>;

	#[test]
	fn from_and_into_round_trip() {
		let free: FreeExplicit<'_, _, i32> = FreeExplicit::pure(42);
		let run: RunAlias<'_, i32> = RunExplicit::from_free_explicit(free);
		let _back = run.into_free_explicit();
	}

	#[test]
	fn brand_pure_evaluates() {
		let run: RunAlias<'_, _> = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(7);
		assert_eq!(run.into_free_explicit().evaluate(), 7);
	}

	#[test]
	fn brand_map_evaluates() {
		let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(10);
		let mapped = <RunExplicitBrand<FirstRow, Scoped> as Functor>::map(|x: i32| x * 3, run);
		assert_eq!(mapped.into_free_explicit().evaluate(), 30);
	}

	#[test]
	fn brand_bind_evaluates() {
		let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(2);
		let chained = <RunExplicitBrand<FirstRow, Scoped> as Semimonad>::bind(run, |x: i32| {
			<RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(x + 5)
		});
		assert_eq!(chained.into_free_explicit().evaluate(), 7);
	}

	#[test]
	fn brand_ref_pure_evaluates() {
		let value = 11;
		let run: RunAlias<'_, _> =
			<RunExplicitBrand<FirstRow, Scoped> as RefPointed>::ref_pure(&value);
		assert_eq!(run.into_free_explicit().evaluate(), 11);
	}

	#[test]
	fn brand_ref_map_evaluates() {
		let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(4);
		let mapped =
			<RunExplicitBrand<FirstRow, Scoped> as RefFunctor>::ref_map(|x: &i32| *x * 5, &run);
		assert_eq!(mapped.into_free_explicit().evaluate(), 20);
	}

	#[test]
	fn brand_ref_bind_evaluates() {
		let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(8);
		let chained =
			<RunExplicitBrand<FirstRow, Scoped> as RefSemimonad>::ref_bind(&run, |x: &i32| {
				<RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(*x + 1)
			});
		assert_eq!(chained.into_free_explicit().evaluate(), 9);
	}

	#[test]
	fn non_static_payload() {
		let s = String::from("hello");
		let r: &str = &s;
		let run: RunExplicit<'_, FirstRow, Scoped, &str> =
			RunExplicit::from_free_explicit(FreeExplicit::pure(r));
		assert_eq!(run.into_free_explicit().evaluate(), "hello");
	}

	#[test]
	fn pure_then_peel_returns_value() {
		let run: RunExplicit<'_, FirstRow, Scoped, i32> = RunExplicit::pure(42);
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
		let run: RunExplicit<'_, FirstRow, Scoped, i32> = RunExplicit::send(Node::First(layer));
		assert!(run.peel().is_err());
	}

	#[test]
	fn from_erased_round_trips_pure() {
		use crate::{
			brands::CoyonedaBrand,
			types::effects::run::Run,
		};
		type CoyoFirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		let run: Run<CoyoFirstRow, CNilBrand, i32> = Run::pure(42);
		let explicit: RunExplicit<'static, CoyoFirstRow, CNilBrand, i32> =
			RunExplicit::from_erased(run);
		assert!(matches!(explicit.peel(), Ok(42)));
	}

	#[test]
	fn from_erased_preserves_suspended_layer() {
		use crate::{
			brands::CoyonedaBrand,
			types::{
				Coyoneda,
				Identity,
				effects::{
					coproduct::Coproduct,
					node::Node,
					run::Run,
				},
			},
		};
		type CoyoFirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		let coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(Identity(7));
		let layer = Coproduct::inject(coyo);
		let run: Run<CoyoFirstRow, CNilBrand, i32> = Run::send(Node::First(layer));
		let explicit = RunExplicit::from_erased(run);
		assert!(explicit.peel().is_err());
	}
}
