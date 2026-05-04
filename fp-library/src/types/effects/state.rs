//! Stateful first-order effect type with `Get` (read state) and
//! `Put` (write state) operations. The corresponding brand is
//! [`StateBrand`](crate::brands::StateBrand).
//!
//! `State<'a, P, S, A>` mirrors PureScript Run's
//! [`State s a = Get (s -> a) | Put s (Unit -> a)`](https://github.com/natefaubion/purescript-run/blob/main/src/Run/State.purs)
//! shape directly. Each variant carries a continuation
//! (`P::Of<'a, dyn Fn(S) -> A>` for `Get`; `P::Of<'a, dyn Fn(()) -> A>`
//! for `Put`); the `Functor` instance composes a user-supplied
//! `f: A -> B` with the stored continuation to produce a new
//! `State<'a, P, S, B>`.
//!
//! ## Wrapper parameterization
//!
//! The pointer brand `P` (typically [`RcBrand`](crate::brands::RcBrand)
//! for single-thread substrates, [`ArcBrand`](crate::brands::ArcBrand)
//! for thread-safe substrates) is threaded through `State`'s
//! continuations via [`ToDynCloneFn`](crate::classes::ToDynCloneFn)
//! so a single `State` type works across all six Run wrappers per
//! the [2026-05-03 resolution](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/resolutions.md).
//! Per-wrapper smart constructors (Phase 3 step 5a.2) thread the
//! substrate-appropriate `P`.
//!
//! Note that even single-shot wrappers (`Run` / `RunExplicit`) use
//! `Rc`-wrapped continuations rather than `Box<dyn FnOnce>`. The
//! single Rc allocation per `Get` / `Put` is a small cost compared
//! to the design simplification of one effect type per operation
//! across all wrappers; bare-Coyoneda single-shot wrappers run the
//! continuation once and drop the extra refcount on completion.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				SendStateBrand,
				StateBrand,
			},
			classes::{
				Functor,
				RefCountedPointer,
				SendFunctor,
				SendRefCountedPointer,
				ToDynCloneFn,
				ToDynSendFn,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	/// Stateful first-order effect type.
	///
	/// Each variant carries a continuation in `P`'s pointer kind
	/// (`Rc<dyn Fn(...) -> A>` for [`RcBrand`](crate::brands::RcBrand);
	/// `Arc<dyn Fn(...) -> A + Send + Sync>` for
	/// [`ArcBrand`](crate::brands::ArcBrand)). Mirrors PureScript Run's
	/// [`State s a`](https://github.com/natefaubion/purescript-run/blob/main/src/Run/State.purs).
	#[document_type_parameters(
		"The lifetime of the continuations and any references they capture.",
		"The pointer brand used for the continuations (typically [`RcBrand`](crate::brands::RcBrand) or [`ArcBrand`](crate::brands::ArcBrand)).",
		"The state type.",
		"The result type produced by running the effect."
	)]
	pub enum State<'a, P, S, A>
	where
		P: ToDynCloneFn,
		S: 'a,
		A: 'a, {
		/// Read the current state. The continuation is applied to
		/// the current state value to produce the result `A`.
		Get(<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(S) -> A>),
		/// Write a new state. The continuation is applied to `()`
		/// (after the state has been updated) to produce the result
		/// `A`.
		Put(S, <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>),
	}

	impl_kind! {
		impl<P: ToDynCloneFn, S: 'static> for StateBrand<P, S> {
			type Of<'a, A: 'a>: 'a = State<'a, P, S, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the continuations.",
		"The pointer brand used for the continuations.",
		"The state type.",
		"The result type."
	)]
	#[document_parameters("The state effect to clone.")]
	impl<'a, P, S, A> Clone for State<'a, P, S, A>
	where
		P: ToDynCloneFn,
		S: Clone + 'a,
		A: 'a,
	{
		/// Clones the state effect by refcount-bumping the stored
		/// continuation pointer; the `Put` variant additionally clones
		/// the carried state value (hence the `S: Clone` bound). The
		/// continuation pointer is `<P as RefCountedPointer>::Of<...>`,
		/// which is unconditionally [`Clone`] per the trait's
		/// associated-type bound.
		#[document_signature]
		///
		#[document_returns("A new state effect sharing the continuation by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		types::effects::state::State,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let original: State<'static, RcBrand, i32, i32> =
		/// 	State::Get(Rc::new(|s: i32| s + 1) as Rc<dyn Fn(i32) -> i32>);
		/// let cloned = original.clone();
		/// match cloned {
		/// 	State::Get(k) => assert_eq!(k(7), 8),
		/// 	State::Put(..) => panic!("expected Get"),
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				State::Get(k) => State::Get(k.clone()),
				State::Put(s, k) => State::Put(s.clone(), k.clone()),
			}
		}
	}

	#[document_type_parameters("The pointer brand used for the continuations.", "The state type.")]
	impl<P, S> Functor for StateBrand<P, S>
	where
		P: ToDynCloneFn,
		S: 'static,
	{
		/// Maps `f` over the result type of this stateful effect.
		///
		/// Composes `f` with each variant's stored continuation. The
		/// pointer kind `P` is preserved; the new continuation is
		/// constructed via [`ToDynCloneFn::new`].
		///
		/// ## Coyoneda fusion at the call site
		///
		/// Each invocation allocates one fresh
		/// [`<P as RefCountedPointer>::Of<...>`](crate::classes::RefCountedPointer)
		/// to wrap the composed continuation. In production, this
		/// method is called at most **once per dispatch** because the
		/// row brand wraps `StateBrand<P, S>` in
		/// [`CoyonedaBrand`](crate::brands::CoyonedaBrand) (or its
		/// Rc/Arc-cloneable siblings), and `CoyonedaBrand::map`
		/// accumulates deferred compositions inside the Coyoneda
		/// layer chain rather than calling the inner functor's `map`
		/// directly. The chain is collapsed to a single application
		/// only when `Coyoneda::lower` (or `lower_ref`) runs at the
		/// dispatch boundary, which then invokes
		/// [`StateBrand::map`](StateBrand) exactly once with the
		/// fully-fused composition.
		///
		/// Net cost: one `Rc`/`Arc` allocation per **layer dispatch**,
		/// not per user-side `.map()` call. Direct call-sites of
		/// `StateBrand::map` in production would defeat this fusion;
		/// the rows in
		/// [`Run`](crate::types::effects::run::Run) /
		/// [`RcRun`](crate::types::effects::rc_run::RcRun) /
		/// [`ArcRun`](crate::types::effects::arc_run::ArcRun) /
		/// their `Explicit` siblings always interpose Coyoneda, so no
		/// production path hits the un-fused cost. Doctests below
		/// exercise the method directly to verify behaviour at the
		/// trait boundary.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters(
			"The function to compose with each continuation.",
			"The state effect to map over."
		)]
		///
		#[document_returns("A new state effect with `f` composed onto each continuation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		types::effects::state::State,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// // Build a `State::Get` continuation that returns the state
		/// // doubled, then map `+1` over it. After mapping, the
		/// // continuation produces `2 * s + 1`.
		/// let get: State<'static, RcBrand, i32, i32> =
		/// 	State::Get(Rc::new(|s: i32| s * 2) as Rc<dyn Fn(i32) -> i32>);
		/// let mapped = <StateBrand<RcBrand, i32> as Functor>::map(|x: i32| x + 1, get);
		/// match mapped {
		/// 	State::Get(k) => assert_eq!(k(3), 7),
		/// 	State::Put(..) => panic!("expected Get"),
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				State::Get(k) => State::Get(<P as ToDynCloneFn>::new(move |s: S| f((*k)(s)))),
				State::Put(s, k) =>
					State::Put(s, <P as ToDynCloneFn>::new(move |u: ()| f((*k)(u)))),
			}
		}
	}

	// SendFunctor on StateBrand cannot ship: the projection
	// <P as RefCountedPointer>::Of<'_, dyn 'a + Fn(S) -> A>
	// is structurally !Send + !Sync because the trait object's bounds
	// don't include + Send + Sync. The Send-aware sibling SendStateBrand
	// below uses <P as SendRefCountedPointer>::Of<'_, dyn 'a + Fn(...) + Send + Sync>
	// to sidestep this; the Arc family smart constructors target
	// SendStateBrand. Per the 2026-05-03 SendFunctor option-(c) resolution.

	/// Thread-safe sibling of [`State`] with `Send + Sync`-bounded
	/// continuation trait objects.
	///
	/// Each variant carries a continuation in `P`'s Send-aware
	/// pointer kind
	/// (`Arc<dyn Fn(...) -> A + Send + Sync>` for
	/// [`ArcBrand`](crate::brands::ArcBrand)). Mirrors PureScript
	/// Run's
	/// [`State s a`](https://github.com/natefaubion/purescript-run/blob/main/src/Run/State.purs)
	/// shape directly, with the marker traits baked into the trait
	/// object's bounds so the projection is structurally
	/// `Send + Sync`.
	///
	/// Used by the Arc family smart constructors
	/// ([`ArcRun::get`](crate::types::effects::arc_run::ArcRun) /
	/// [`ArcRun::put`](crate::types::effects::arc_run::ArcRun) /
	/// [`ArcRunExplicit::get`](crate::types::effects::arc_run_explicit::ArcRunExplicit) /
	/// [`ArcRunExplicit::put`](crate::types::effects::arc_run_explicit::ArcRunExplicit))
	/// per the
	/// [2026-05-03 SendFunctor option-(c) resolution](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/resolutions.md);
	/// non-Arc smart constructors keep using [`State`].
	#[document_type_parameters(
		"The lifetime of the continuations and any references they capture.",
		"The pointer brand used for the continuations (typically [`ArcBrand`](crate::brands::ArcBrand)).",
		"The state type.",
		"The result type produced by running the effect."
	)]
	pub enum SendState<'a, P, S, A>
	where
		P: ToDynSendFn,
		S: 'a,
		A: 'a, {
		/// Read the current state. The continuation is applied to
		/// the current state value to produce the result `A`.
		Get(<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(S) -> A + Send + Sync>),
		/// Write a new state. The continuation is applied to `()`
		/// (after the state has been updated) to produce the result
		/// `A`.
		Put(S, <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>),
	}

	impl_kind! {
		impl<P: ToDynSendFn, S: 'static> for SendStateBrand<P, S> {
			type Of<'a, A: 'a>: 'a = SendState<'a, P, S, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the continuations.",
		"The pointer brand used for the continuations.",
		"The state type.",
		"The result type."
	)]
	#[document_parameters("The send-state effect to clone.")]
	impl<'a, P, S, A> Clone for SendState<'a, P, S, A>
	where
		P: ToDynSendFn,
		S: Clone + 'a,
		A: 'a,
	{
		/// Clones the send-state effect by refcount-bumping the
		/// stored continuation pointer; the `Put` variant additionally
		/// clones the carried state value (hence the `S: Clone`
		/// bound). The continuation pointer is
		/// `<P as SendRefCountedPointer>::Of<...>`, which is
		/// unconditionally [`Clone`] per the trait's associated-type
		/// bound.
		#[document_signature]
		///
		#[document_returns("A new send-state effect sharing the continuation by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		types::effects::state::SendState,
		/// 	},
		/// 	std::sync::Arc,
		/// };
		///
		/// let original: SendState<'static, ArcBrand, i32, i32> =
		/// 	SendState::Get(Arc::new(|s: i32| s + 1) as Arc<dyn Fn(i32) -> i32 + Send + Sync>);
		/// let cloned = original.clone();
		/// match cloned {
		/// 	SendState::Get(k) => assert_eq!(k(7), 8),
		/// 	SendState::Put(..) => panic!("expected Get"),
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendState::Get(k) => SendState::Get(k.clone()),
				SendState::Put(s, k) => SendState::Put(s.clone(), k.clone()),
			}
		}
	}

	// Functor (by-value `map`) is intentionally NOT implemented for
	// SendStateBrand: the new continuation must be storable behind
	// <P as SendRefCountedPointer>::Of<'_, dyn Fn + Send + Sync>, but
	// Functor::map's signature only requires `f: Fn` (no Send + Sync),
	// so a Functor impl could not construct a Send-aware trait object.
	// SendFunctor is independent of Functor in fp-library (not a
	// supertrait), so this is sound; users reach for
	// SendFunctor::send_map directly.

	#[document_type_parameters("The pointer brand used for the continuations.", "The state type.")]
	impl<P, S> SendFunctor for SendStateBrand<P, S>
	where
		P: ToDynSendFn,
		S: Send + Sync + 'static,
	{
		/// Maps `f` over the result type of this thread-safe
		/// stateful effect, with `Send + Sync` bounds on `f` and
		/// the result types so the new continuation can be stored
		/// behind a thread-safe trait object.
		///
		/// Composes `f` with each variant's stored continuation via
		/// [`ToDynSendFn::new`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters(
			"The function to compose with each continuation.",
			"The send-state effect to map over."
		)]
		///
		#[document_returns("A new send-state effect with `f` composed onto each continuation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		types::effects::state::SendState,
		/// 	},
		/// 	std::sync::Arc,
		/// };
		///
		/// let get: SendState<'static, ArcBrand, i32, i32> =
		/// 	SendState::Get(Arc::new(|s: i32| s * 2) as Arc<dyn Fn(i32) -> i32 + Send + Sync>);
		/// let mapped = <SendStateBrand<ArcBrand, i32> as SendFunctor>::send_map(|x: i32| x + 1, get);
		/// match mapped {
		/// 	SendState::Get(k) => assert_eq!(k(3), 7),
		/// 	SendState::Put(..) => panic!("expected Get"),
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				SendState::Get(k) =>
					SendState::Get(<P as ToDynSendFn>::new(move |s: S| f((*k)(s)))),
				SendState::Put(s, k) =>
					SendState::Put(s, <P as ToDynSendFn>::new(move |u: ()| f((*k)(u)))),
			}
		}
	}
}

pub use inner::*;
