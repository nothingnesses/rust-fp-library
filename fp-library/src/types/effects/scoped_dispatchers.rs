//! Standard scoped-effect dispatcher values.
//!
//! These dispatchers are runtime handler-list cells for built-in scoped
//! effects. They implement
//! [`DispatchScopedHandler`](crate::types::effects::interpreter::DispatchScopedHandler)
//! so callers can pass them to `scoped_handlers!` or the scoped handler
//! builder API. Dispatchers that rewrite first-order operations carry
//! the row evidence needed by the underlying `interpose` operation;
//! simple around-action dispatchers are witness-free.

#[allow(
	unused_imports,
	reason = "Child scoped-dispatcher modules consume different subsets of this shared prelude."
)]
mod prelude {
	pub(super) use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				BoxBracketBrand,
				BoxBrand,
				BoxCatchBrand,
				BoxLocalBrand,
				BoxReaderBrand,
				BoxRefLocalBrand,
				BoxSpanBrand,
				ExceptBrand,
				NodeBrand,
				RcBrand,
				ReaderBrand,
				SendReaderBrand,
			},
			classes::{
				Functor,
				SendFunctor,
				WrapDrop,
			},
			kinds::*,
			types::{
				ArcCoyoneda,
				ArcFree,
				ArcFreeExplicit,
				Coyoneda,
				Free,
				FreeExplicit,
				RcCoyoneda,
				RcFree,
				RcFreeExplicit,
				arc_free::ArcTypeErasedValue,
				effects::{
					arc_run::ArcRun,
					arc_run_explicit::{
						ArcRunExplicit,
						ArcRunExplicitActionSuppliedScopedContinuation,
						ArcRunExplicitBoundary,
						ArcRunExplicitScopedContinuation,
					},
					bracket::{
						BoxBracket,
						BoxBracketExplicit,
						Bracket,
						BracketExplicit,
						SendBracket,
						SendBracketExplicit,
					},
					catch::{
						BoxCatch,
						Catch,
						SendCatch,
					},
					coproduct::CoproductEmbedder,
					except::Except,
					interpreter::{
						ArcActionSuppliedScopedResume,
						ArcScopedResume,
						DispatchHandlers,
						DispatchScopedCarrierHandler,
						DispatchScopedHandler,
						ExplicitActionSuppliedScopedResume,
						ExplicitScopedResume,
						RcActionSuppliedScopedResume,
						RcScopedResume,
						ScopedResumeTypes,
					},
					local::{
						BoxLocal,
						Local,
						SendLocal,
					},
					member::Member,
					rc_run::RcRun,
					rc_run_explicit::{
						RcRunExplicit,
						RcRunExplicitActionSuppliedScopedContinuation,
						RcRunExplicitBoundary,
						RcRunExplicitScopedContinuation,
					},
					reader::{
						BoxReader,
						Reader,
						SendReader,
					},
					ref_bracket::{
						RefBracket,
						RefBracketExplicit,
						SendRefBracket,
						SendRefBracketExplicit,
					},
					ref_local::{
						BoxRefLocal,
						RefLocal,
						SendRefLocal,
					},
					run::{
						DispatchRunRawScopedHandler,
						RawRunFree,
						Run,
						RunContinuations,
					},
					run_explicit::{
						RunExplicit,
						RunExplicitActionSuppliedScopedContinuation,
						RunExplicitBoundary,
						RunExplicitBracketCarrierLayer,
						RunExplicitCatchCarrierLayer,
						RunExplicitLocalCarrierLayer,
						RunExplicitRefBracketCarrierLayer,
						RunExplicitRefLocalCarrierLayer,
						RunExplicitScopedContinuation,
						RunExplicitSpanCarrierLayer,
					},
					span::{
						BoxSpan,
						SendSpan,
						Span,
					},
				},
				rc_free::RcTypeErasedValue,
			},
		},
		fp_macros::*,
		std::{
			marker::PhantomData,
			rc::Rc,
			sync::Arc,
		},
	};
}

mod bracket;
mod catch;
mod local;
mod ref_bracket;
mod ref_local;
mod span;

#[fp_macros::document_module]
mod inner {
	pub use super::{
		bracket::{
			BracketDispatcher,
			bracket_dispatcher,
		},
		catch::{
			CatchDispatcher,
			catch_dispatcher,
		},
		local::{
			LocalDispatcher,
			local_dispatcher,
		},
		ref_bracket::{
			RefBracketDispatcher,
			ref_bracket_dispatcher,
		},
		ref_local::{
			RefLocalDispatcher,
			ref_local_dispatcher,
		},
		span::{
			SpanDispatcher,
			span_dispatcher,
		},
	};
}

pub use inner::*;
