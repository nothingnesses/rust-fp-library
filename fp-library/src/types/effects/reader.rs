//! Environment-reading first-order effect type with the `Ask`
//! operation. The corresponding brand is
//! [`ReaderBrand`](crate::brands::ReaderBrand).
//!
//! `Reader<'a, P, E, A>` mirrors PureScript Run's
//! `Reader e a = Reader (e -> a)` shape directly. The single `Ask` variant carries a continuation
//! (`P::Of<'a, dyn Fn(E) -> A>`); the `Functor` instance composes a
//! user-supplied `f: A -> B` with the stored continuation to produce
//! a new `Reader<'a, P, E, B>`.
//!
//! ## Wrapper parameterization
//!
//! The pointer brand `P` (typically [`RcBrand`](crate::brands::RcBrand)
//! for single-thread substrates, [`ArcBrand`](crate::brands::ArcBrand)
//! for thread-safe substrates) is threaded through `Reader`'s
//! continuation via [`ToDynCloneFn`](crate::classes::ToDynCloneFn)
//! so a single `Reader` type works across all six Run wrappers.
//! Per-wrapper `ask` smart constructors thread the substrate-
//! appropriate `P`. The Arc family uses the parallel
//! [`SendReader`] /
//! [`SendReaderBrand`](crate::brands::SendReaderBrand) below to
//! sidestep the `Arc<dyn Fn>: !Send + !Sync` structural problem.
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
				BoxReaderBrand,
				ReaderBrand,
				SendReaderBrand,
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
		effect Reader;
	}
}

pub use inner::*;
