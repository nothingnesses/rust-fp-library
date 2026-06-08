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
//! so a single `State` type works across all six Run wrappers.
//! Per-wrapper smart constructors thread the substrate-appropriate
//! `P`.
//!
//! Note that even single-shot wrappers (`Run` / `RunExplicit`) use
//! `Rc`-wrapped continuations rather than `Box<dyn FnOnce>`. The
//! single Rc allocation per `Get` / `Put` is a small cost compared
//! to the design simplification of one effect type per operation
//! across all wrappers; bare-Coyoneda single-shot wrappers run the
//! continuation once and drop the extra refcount on completion.
//!
//! See the parent [`effects`](crate::types::effects) guide for the
//! consolidated wrapper and pointer-brand conventions.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				BoxBrand,
				BoxStateBrand,
				SendStateBrand,
				StateBrand,
			},
			classes::{
				Functor,
				Pointer,
				RefCountedPointer,
				SendFunctor,
				SendRefCountedPointer,
				ToDynCloneFn,
				ToDynFnOnce,
				ToDynSendFn,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	define_effect! {
		effect State;
	}
}

pub use inner::*;
