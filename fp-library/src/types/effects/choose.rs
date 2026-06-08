//! Nondeterministic-branching first-order effect type with the
//! `Alt` operation. The corresponding brand is
//! [`ChooseBrand`](crate::brands::ChooseBrand).
//!
//! `Choose<'a, P, A>` mirrors PureScript Run's
//! `Choose a = Alt (Boolean -> a)` shape directly. The single
//! `Alt` variant carries a continuation `bool -> A`; the `Functor`
//! instance composes a user-supplied `f: A -> B` with the stored
//! continuation to produce a new `Choose<'a, P, B>`.
//!
//! ## Single-shot wrappers cannot host this effect
//!
//! A `Choose` handler runs the continuation twice (once for the
//! `true` branch, once for the `false` branch) to capture the
//! nondeterministic semantics, which requires the underlying
//! continuation pointer to be cloneable. The single-shot wrappers
//! [`Run`](crate::types::effects::run::Run) and
//! [`RunExplicit`](crate::types::effects::run_explicit::RunExplicit)
//! consume their continuations once; only the four multi-shot
//! wrappers ([`RcRun`](crate::types::effects::rc_run::RcRun) /
//! [`RcRunExplicit`](crate::types::effects::rc_run_explicit::RcRunExplicit) /
//! [`ArcRun`](crate::types::effects::arc_run::ArcRun) /
//! [`ArcRunExplicit`](crate::types::effects::arc_run_explicit::ArcRunExplicit))
//! ship the `choose` smart constructor.
//!
//! ## Wrapper parameterization
//!
//! The pointer brand `P` (typically [`RcBrand`](crate::brands::RcBrand)
//! for single-thread substrates, [`ArcBrand`](crate::brands::ArcBrand)
//! for thread-safe substrates) is threaded through `Choose`'s
//! continuation via [`ToDynCloneFn`](crate::classes::ToDynCloneFn)
//! so a single `Choose` type works across the four multi-shot
//! Run wrappers. The Arc family uses the parallel
//! [`SendChoose`] / [`SendChooseBrand`](crate::brands::SendChooseBrand)
//! below to sidestep the `Arc<dyn Fn>: !Send + !Sync` structural
//! problem.
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
				BoxChooseBrand,
				ChooseBrand,
				SendChooseBrand,
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
		effect Choose;
	}
}

pub use inner::*;
