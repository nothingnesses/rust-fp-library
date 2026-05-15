//! Standard scoped-effect handler values.
//!
//! These handlers are runtime handler-list cells for built-in scoped
//! effects. They implement
//! [`DispatchScopedHandler`](crate::types::effects::interpreter::DispatchScopedHandler)
//! so callers can pass them to `scoped_handlers!` or the scoped handler
//! builder API. Handlers that rewrite first-order operations carry
//! the row evidence needed by the underlying `interpose` operation;
//! simple around-action handlers are witness-free.

#[allow(
	unused_imports,
	reason = "Child standard scoped-handler modules consume different subsets of this shared prelude."
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
				BoxWriterCensorBrand,
				BracketBrand,
				CatchBrand,
				ExceptBrand,
				LocalBrand,
				NodeBrand,
				RcBrand,
				ReaderBrand,
				RefBracketBrand,
				RefLocalBrand,
				SendBracketBrand,
				SendCatchBrand,
				SendLocalBrand,
				SendReaderBrand,
				SendRefBracketBrand,
				SendRefLocalBrand,
				SendSpanBrand,
				SendWriterCensorBrand,
				SpanBrand,
				WriterBrand,
				WriterCensorBrand,
			},
			classes::{
				Functor,
				Monoid,
				Semigroup,
				SendFunctor,
				WrapDrop,
			},
			kinds::*,
			types::{
				ArcCoyoneda,
				ArcFree,
				ArcFreeExplicit,
				CatList,
				Coyoneda,
				Free,
				FreeExplicit,
				RcCoyoneda,
				RcFree,
				RcFreeExplicit,
				arc_free::ArcTypeErasedValue,
				effects::{
					arc_run::{
						ArcRun,
						ArcRunContinuations,
						ArcRunFirstOrderAccumulator,
						ArcRunFirstOrderReplacer,
						ArcRunFirstOrderRewriter,
						ArcRunRawScopedContinuation,
						DispatchArcRunRawScopedHandler,
						RawArcRunFree,
					},
					arc_run_explicit::{
						ArcRunExplicit,
						ArcRunExplicitActionSuppliedScopedContinuation,
						ArcRunExplicitBoundary,
						ArcRunExplicitFirstOrderRewriter,
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
						DefaultScopedResume,
						DispatchHandlers,
						DispatchScopedCarrierHandler,
						DispatchScopedHandler,
						ExplicitActionSuppliedScopedResume,
						ExplicitScopedResume,
						RcActionSuppliedScopedResume,
						RcScopedResume,
						ScopedContinuation,
						ScopedResumeTypes,
					},
					local::{
						BoxLocal,
						Local,
						SendLocal,
					},
					member::Member,
					rc_run::{
						DispatchRcRunRawScopedHandler,
						RawRcRunFree,
						RcRun,
						RcRunContinuations,
						RcRunFirstOrderAccumulator,
						RcRunFirstOrderReplacer,
						RcRunFirstOrderRewriter,
						RcRunRawScopedContinuation,
					},
					rc_run_explicit::{
						RcRunExplicit,
						RcRunExplicitActionSuppliedScopedContinuation,
						RcRunExplicitBoundary,
						RcRunExplicitFirstOrderRewriter,
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
						RunFirstOrderAccumulator,
						RunFirstOrderReplacer,
						RunFirstOrderRewriter,
						RunScopedContinuation,
					},
					run_explicit::{
						RunExplicit,
						RunExplicitActionSuppliedScopedContinuation,
						RunExplicitBoundary,
						RunExplicitBracketCarrierLayer,
						RunExplicitCatchCarrierLayer,
						RunExplicitFirstOrderRewriter,
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
					writer::{
						BoxWriterCensor,
						SendWriterCensor,
						Writer,
						WriterCensor,
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
mod writer;

#[fp_macros::document_module]
mod inner {
	pub use super::{
		bracket::{
			BracketHandler,
			bracket_handler,
		},
		catch::{
			CatchHandler,
			catch_handler,
		},
		local::{
			LocalHandler,
			local_handler,
		},
		ref_bracket::{
			RefBracketHandler,
			ref_bracket_handler,
		},
		ref_local::{
			RefLocalHandler,
			ref_local_handler,
		},
		span::{
			SpanHandler,
			span_handler,
		},
		writer::{
			WriterPostHandler,
			WriterPreHandler,
			writer_post_handler,
			writer_pre_handler,
		},
	};
}

pub use inner::*;
