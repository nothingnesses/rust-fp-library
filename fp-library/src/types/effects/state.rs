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
			brands::StateBrand,
			classes::{
				Functor,
				RefCountedPointer,
				ToDynCloneFn,
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

	// SendFunctor impl deferred to step 5a.3: the bound
	// `<P as RefCountedPointer>::Of<'_, dyn 'a + Fn(S) -> A>: Send + Sync`
	// must be expressed per-`A` for `send_map`'s generic `A` parameter,
	// which requires HRTB-over-types support not available on stable Rust.
	// Ships when ArcRun's smart constructors land alongside it.
}

pub use inner::*;
