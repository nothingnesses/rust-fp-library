//! Coroutine first-order effect type.
//!
//! `Coroutine<'a, P, Out, In, A>` yields an `Out` value to the runner
//! and resumes when the runner supplies an `In` value. The effect cell
//! is split across Box, Rc, and Arc pointer-brand siblings so the
//! continuation storage matches the surrounding Run wrapper's
//! single-shot, multi-shot, or thread-safe semantics.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				BoxBrand,
				BoxCoroutineBrand,
				CoroutineBrand,
				NodeBrand,
				RcBrand,
				SendCoroutineBrand,
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
				WrapDrop,
			},
			impl_kind,
			kinds::*,
			types::{
				ArcFree,
				arc_free::ArcTypeErasedValue,
				effects::{
					arc_run::ArcRun,
					arc_run_explicit::ArcRunExplicit,
					rc_run::RcRun,
					rc_run_explicit::RcRunExplicit,
					run::Run,
					run_explicit::RunExplicit,
				},
			},
		},
		fp_macros::*,
	};

	define_effect! {
		effect Coroutine;
	}
}

pub use inner::*;
