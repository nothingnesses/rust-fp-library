//! Standard scoped-effect dispatcher values.
//!
//! These dispatchers are runtime handler-list cells for built-in scoped
//! effects. They implement
//! [`DispatchScopedHandler`]
//! so callers can pass them to `scoped_handlers!` or the scoped handler
//! builder API. Dispatchers that rewrite first-order operations carry
//! the row evidence needed by the underlying `interpose` operation;
//! simple around-action dispatchers are witness-free.

use {
	crate::{
		Apply,
		brands::{
			ArcBrand,
			BoxBrand,
			BoxCatchBrand,
			BoxSpanBrand,
			ExceptBrand,
			NodeBrand,
			RcBrand,
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
				arc_run_explicit::ArcRunExplicit,
				catch::{
					BoxCatch,
					Catch,
					SendCatch,
				},
				coproduct::CoproductEmbedder,
				except::Except,
				interpreter::{
					DispatchHandlers,
					DispatchScopedHandler,
				},
				member::Member,
				rc_run::RcRun,
				rc_run_explicit::RcRunExplicit,
				run::{
					DispatchRunRawScopedHandler,
					RawRunFree,
					Run,
					RunContinuations,
				},
				run_explicit::RunExplicit,
				span::{
					BoxSpan,
					SendSpan,
					Span,
				},
			},
			rc_free::RcTypeErasedValue,
		},
	},
	std::marker::PhantomData,
};

/// Dispatcher for the standard `Catch` scoped effect.
///
/// `Idx`, `RMinusE`, and `EmbedIndices` are the same row witnesses
/// consumed by each wrapper's `interpose` method: the position of
/// `ExceptBrand<E>` in the first-order row, the row with that effect
/// removed, and the witness for embedding the narrowed row back into
/// the original row while preserving surrounding scoped operations.
///
/// This dispatcher is implemented for Rc-backed and Arc-backed
/// wrappers. Box-backed `Run` / `RunExplicit` need a separate design:
/// after `Run::peel` maps a suspended `BoxCatch` layer, both the
/// protected action and the recovery handler can need the same
/// single-shot `Free` continuation.
#[derive(Clone, Copy, Debug, Default)]
#[expect(
	clippy::type_complexity,
	reason = "The fn marker carries row-witness type parameters without making auto-traits depend on them."
)]
pub struct CatchDispatcher<Idx, RMinusE, EmbedIndices>(
	PhantomData<fn() -> (Idx, RMinusE, EmbedIndices)>,
);

/// Constructs a [`CatchDispatcher`] without naming its private field.
pub const fn catch_dispatcher<Idx, RMinusE, EmbedIndices>()
-> CatchDispatcher<Idx, RMinusE, EmbedIndices> {
	CatchDispatcher(PhantomData)
}

/// Dispatcher for the standard `Span` scoped effect.
///
/// The dispatcher consumes the by-value tag and resumes the stored
/// action program unchanged.
#[derive(Clone, Copy, Debug, Default)]
pub struct SpanDispatcher;

/// Constructs a [`SpanDispatcher`].
pub const fn span_dispatcher() -> SpanDispatcher {
	SpanDispatcher
}

impl<R, S, A, E, Idx, RMinusE, EmbedIndices, FirstLayer>
	DispatchRunRawScopedHandler<R, S, A, BoxCatchBrand<BoxBrand, E>, FirstLayer>
	for CatchDispatcher<Idx, RMinusE, EmbedIndices>
where
	R: WrapDrop + Functor + 'static,
	S: WrapDrop + Functor + 'static,
	A: 'static,
	E: 'static,
	FirstLayer: 'static,
	RMinusE: WrapDrop + Functor + 'static,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		Run<R, S, crate::types::free::TypeErasedValue>,
	>): Member<
			Coyoneda<'static, ExceptBrand<E>, Run<R, S, crate::types::free::TypeErasedValue>>,
			Idx,
			Remainder = Apply!(
							<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'static,
								Run<R, S, crate::types::free::TypeErasedValue>,
							>
						),
		>,
	Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		Free<NodeBrand<R, S>, crate::types::free::TypeErasedValue>,
	>): CoproductEmbedder<
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Free<NodeBrand<R, S>, crate::types::free::TypeErasedValue>,
			>),
			EmbedIndices,
		>,
{
	fn dispatch_run_raw_scoped_head(
		&self,
		layer: BoxCatch<'static, BoxBrand, E, RawRunFree<R, S>>,
		continuations: RunContinuations<R, S>,
		_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
	) -> Run<R, S, A> {
		match layer {
			BoxCatch::Catch {
				action,
				handler,
			} => {
				let handler = std::cell::RefCell::new(Some(handler));
				let interposed = Run::<R, S, crate::types::free::TypeErasedValue>::from_free(
					action(()),
				)
				.interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| match op {
					Except::Throw(e, _) => {
						#[expect(
							clippy::expect_used,
							reason = "Box-backed Catch handlers are single-shot and the protected action can throw at most once"
						)]
						let handler = handler
							.borrow_mut()
							.take()
							.expect("BoxCatch handler invoked more than once");
						Run::from_free(handler(e))
					}
				});
				Run::from_free(Free::continue_from_erased(interposed.into_free(), continuations))
			}
		}
	}
}

impl<R, S, A, E, Idx, RMinusE, EmbedIndices>
	DispatchScopedHandler<
		'static,
		Catch<'static, RcBrand, E, RcRun<R, S, A>>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
		RcRun<R, S, A>,
	> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
where
	R: WrapDrop + Functor + 'static,
	S: WrapDrop + Functor + 'static,
	A: Clone + 'static,
	E: 'static,
	RMinusE: WrapDrop + Functor + 'static,
	Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
	>): Clone,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>): Member<
			RcCoyoneda<'static, ExceptBrand<E>, RcRun<R, S, A>>,
			Idx,
			Remainder = Apply!(
							<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>
						),
		>,
	Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<R, S>, A>,
	>): CoproductEmbedder<
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, A>,
			>),
			EmbedIndices,
		>,
{
	fn dispatch_scoped_head(
		&self,
		layer: Catch<'static, RcBrand, E, RcRun<R, S, A>>,
		_fo_handlers: &impl DispatchHandlers<
			'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
			RcRun<R, S, A>,
		>,
	) -> RcRun<R, S, A> {
		match layer {
			Catch::Catch {
				action,
				handler,
			} => action(()).interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
				match op {
					Except::Throw(e, _) => (*handler)(e),
				}
			}),
		}
	}
}

impl<R, S, A, E, Idx, RMinusE, EmbedIndices>
	DispatchScopedHandler<
		'static,
		SendCatch<'static, ArcBrand, E, ArcRun<R, S, A>>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
		ArcRun<R, S, A>,
	> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
where
	R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
	S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
	A: Clone + Send + Sync + 'static,
	E: Send + Sync + 'static,
	RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
	ExceptBrand<E>: Functor
		+ SendFunctor
		+ Kind_cdc7cd43dac7585f<Of<'static, ArcRun<R, S, A>> = Except<'static, E, ArcRun<R, S, A>>>,
	NodeBrand<R, S>: WrapDrop
		+ Kind_cdc7cd43dac7585f<
			Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
		> + SendFunctor
		+ 'static,
	Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
	>): Clone,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>): Member<
			ArcCoyoneda<'static, ExceptBrand<E>, ArcRun<R, S, A>>,
			Idx,
			Remainder = Apply!(
							<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>
						),
		>,
	Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		ArcFree<NodeBrand<R, S>, A>,
	>): CoproductEmbedder<
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, A>,
			>),
			EmbedIndices,
		>,
{
	fn dispatch_scoped_head(
		&self,
		layer: SendCatch<'static, ArcBrand, E, ArcRun<R, S, A>>,
		_fo_handlers: &impl DispatchHandlers<
			'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			ArcRun<R, S, A>,
		>,
	) -> ArcRun<R, S, A> {
		match layer {
			SendCatch::Catch {
				action,
				handler,
			} => action(()).interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
				match op {
					Except::Throw(e, _) => (*handler)(e),
				}
			}),
		}
	}
}

impl<'a, R, S, A, E, Idx, RMinusE, EmbedIndices>
	DispatchScopedHandler<
		'a,
		BoxCatch<'a, BoxBrand, E, RunExplicit<'a, R, S, A>>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>),
		RunExplicit<'a, R, S, A>,
	> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
where
	R: WrapDrop + Functor + 'static,
	S: WrapDrop + Functor + 'static,
	A: 'a,
	E: 'a + 'static,
	RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
	Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>): Member<
			Coyoneda<'a, ExceptBrand<E>, RunExplicit<'a, R, S, A>>,
			Idx,
			Remainder = Apply!(
							<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>
						),
		>,
	Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		Box<FreeExplicit<'a, NodeBrand<R, S>, A>>,
	>): CoproductEmbedder<
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				Box<FreeExplicit<'a, NodeBrand<R, S>, A>>,
			>),
			EmbedIndices,
		>,
{
	fn dispatch_scoped_head(
		&self,
		layer: BoxCatch<'a, BoxBrand, E, RunExplicit<'a, R, S, A>>,
		_fo_handlers: &impl DispatchHandlers<
			'a,
			Apply!(
				<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>
			),
			RunExplicit<'a, R, S, A>,
		>,
	) -> RunExplicit<'a, R, S, A> {
		match layer {
			BoxCatch::Catch {
				action,
				handler,
			} => {
				let handler = std::cell::RefCell::new(Some(handler));
				action(()).interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
					match op {
						Except::Throw(e, _) => {
							#[expect(
								clippy::expect_used,
								reason = "Box-backed Catch handlers are single-shot and the protected action can throw at most once"
							)]
							let handler = handler
								.borrow_mut()
								.take()
								.expect("BoxCatch handler invoked more than once");
							handler(e)
						}
					}
				})
			}
		}
	}
}

impl<'a, R, S, A, E, Idx, RMinusE, EmbedIndices>
	DispatchScopedHandler<
		'a,
		Catch<'a, RcBrand, E, RcRunExplicit<'a, R, S, A>>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
		RcRunExplicit<'a, R, S, A>,
	> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
where
	R: WrapDrop + Functor + 'static,
	S: WrapDrop + Functor + 'static,
	A: Clone + 'a,
	E: 'a + 'static,
	RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
	Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		RcFreeExplicit<'a, NodeBrand<R, S>, A>,
	>): Clone,
	Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>): Member<
			RcCoyoneda<'a, ExceptBrand<E>, RcRunExplicit<'a, R, S, A>>,
			Idx,
			Remainder = Apply!(
							<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
						),
		>,
	Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		RcFreeExplicit<'a, NodeBrand<R, S>, A>,
	>): CoproductEmbedder<
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>),
			EmbedIndices,
		>,
{
	fn dispatch_scoped_head(
		&self,
		layer: Catch<'a, RcBrand, E, RcRunExplicit<'a, R, S, A>>,
		_fo_handlers: &impl DispatchHandlers<
			'a,
			Apply!(
				<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
			),
			RcRunExplicit<'a, R, S, A>,
		>,
	) -> RcRunExplicit<'a, R, S, A> {
		match layer {
			Catch::Catch {
				action,
				handler,
			} => action(()).interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
				match op {
					Except::Throw(e, _) => (*handler)(e),
				}
			}),
		}
	}
}

impl<'a, R, S, A, E, Idx, RMinusE, EmbedIndices>
	DispatchScopedHandler<
		'a,
		SendCatch<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, A>>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
		ArcRunExplicit<'a, R, S, A>,
	> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
where
	R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
	S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
	A: Clone + Send + Sync + 'a,
	E: Send + Sync + 'a + 'static,
	RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
	ExceptBrand<E>: Functor
		+ SendFunctor
		+ Kind_cdc7cd43dac7585f<
			Of<'a, ArcRunExplicit<'a, R, S, A>> = Except<'a, E, ArcRunExplicit<'a, R, S, A>>,
		>,
	Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
	>): Clone + Send + Sync,
	Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
	>): Send + Sync,
	Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
	>): Send + Sync,
	Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>): Send
		+ Sync
		+ Member<
			ArcCoyoneda<'a, ExceptBrand<E>, ArcRunExplicit<'a, R, S, A>>,
			Idx,
			Remainder = Apply!(
							<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
						),
		>,
	Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>):
		Send + Sync,
	Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>):
		Send + Sync,
	Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
	>): CoproductEmbedder<
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>),
			EmbedIndices,
		>,
{
	fn dispatch_scoped_head(
		&self,
		layer: SendCatch<'a, ArcBrand, E, ArcRunExplicit<'a, R, S, A>>,
		_fo_handlers: &impl DispatchHandlers<
			'a,
			Apply!(
				<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
			),
			ArcRunExplicit<'a, R, S, A>,
		>,
	) -> ArcRunExplicit<'a, R, S, A> {
		match layer {
			SendCatch::Catch {
				action,
				handler,
			} => action(()).interpose::<ExceptBrand<E>, Idx, RMinusE, EmbedIndices>(move |op| {
				match op {
					Except::Throw(e, _) => (*handler)(e),
				}
			}),
		}
	}
}

impl<R, S, A, Tag>
	DispatchScopedHandler<
		'static,
		BoxSpan<'static, BoxBrand, Tag, Run<R, S, A>>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
		Run<R, S, A>,
	> for SpanDispatcher
where
	R: WrapDrop + Functor + 'static,
	S: WrapDrop + Functor + 'static,
	A: 'static,
	Tag: 'static,
{
	fn dispatch_scoped_head(
		&self,
		layer: BoxSpan<'static, BoxBrand, Tag, Run<R, S, A>>,
		_fo_handlers: &impl DispatchHandlers<
			'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, S, A>>),
			Run<R, S, A>,
		>,
	) -> Run<R, S, A> {
		match layer {
			BoxSpan::Span {
				tag: _tag,
				action,
			} => action(()),
		}
	}
}

impl<R, S, A, Tag, FirstLayer>
	DispatchRunRawScopedHandler<R, S, A, BoxSpanBrand<BoxBrand, Tag>, FirstLayer> for SpanDispatcher
where
	R: WrapDrop + Functor + 'static,
	S: WrapDrop + Functor + 'static,
	A: 'static,
	Tag: 'static,
	FirstLayer: 'static,
{
	fn dispatch_run_raw_scoped_head(
		&self,
		layer: BoxSpan<'static, BoxBrand, Tag, RawRunFree<R, S>>,
		continuations: RunContinuations<R, S>,
		_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, Run<R, S, A>>,
	) -> Run<R, S, A> {
		match layer {
			BoxSpan::Span {
				tag: _tag,
				action,
			} => Run::from_free(Free::continue_from_erased(action(()), continuations)),
		}
	}
}

impl<R, S, A, Tag>
	DispatchScopedHandler<
		'static,
		Span<'static, RcBrand, Tag, RcRun<R, S, A>>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
		RcRun<R, S, A>,
	> for SpanDispatcher
where
	R: WrapDrop + Functor + 'static,
	S: WrapDrop + Functor + 'static,
	A: 'static,
	Tag: 'static,
{
	fn dispatch_scoped_head(
		&self,
		layer: Span<'static, RcBrand, Tag, RcRun<R, S, A>>,
		_fo_handlers: &impl DispatchHandlers<
			'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
			RcRun<R, S, A>,
		>,
	) -> RcRun<R, S, A> {
		match layer {
			Span::Span {
				tag: _tag,
				action,
			} => action(()),
		}
	}
}

impl<R, S, A, Tag>
	DispatchScopedHandler<
		'static,
		SendSpan<'static, ArcBrand, Tag, ArcRun<R, S, A>>,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
		ArcRun<R, S, A>,
	> for SpanDispatcher
where
	R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
	S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
	A: Send + Sync + 'static,
	Tag: Send + Sync + 'static,
	NodeBrand<R, S>: WrapDrop
		+ Kind_cdc7cd43dac7585f<
			Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
		> + 'static,
{
	fn dispatch_scoped_head(
		&self,
		layer: SendSpan<'static, ArcBrand, Tag, ArcRun<R, S, A>>,
		_fo_handlers: &impl DispatchHandlers<
			'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
			ArcRun<R, S, A>,
		>,
	) -> ArcRun<R, S, A> {
		match layer {
			SendSpan::Span {
				tag: _tag,
				action,
			} => action(()),
		}
	}
}

impl<'a, R, S, A, Tag>
	DispatchScopedHandler<
		'a,
		BoxSpan<'a, BoxBrand, Tag, RunExplicit<'a, R, S, A>>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>),
		RunExplicit<'a, R, S, A>,
	> for SpanDispatcher
where
	R: WrapDrop + Functor + 'static,
	S: WrapDrop + Functor + 'static,
	A: 'a,
	Tag: 'a + 'static,
{
	fn dispatch_scoped_head(
		&self,
		layer: BoxSpan<'a, BoxBrand, Tag, RunExplicit<'a, R, S, A>>,
		_fo_handlers: &impl DispatchHandlers<
			'a,
			Apply!(
				<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RunExplicit<'a, R, S, A>>
			),
			RunExplicit<'a, R, S, A>,
		>,
	) -> RunExplicit<'a, R, S, A> {
		match layer {
			BoxSpan::Span {
				tag: _tag,
				action,
			} => action(()),
		}
	}
}

impl<'a, R, S, A, Tag>
	DispatchScopedHandler<
		'a,
		Span<'a, RcBrand, Tag, RcRunExplicit<'a, R, S, A>>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>),
		RcRunExplicit<'a, R, S, A>,
	> for SpanDispatcher
where
	R: WrapDrop + Functor + 'static,
	S: WrapDrop + Functor + 'static,
	A: 'a,
	Tag: 'a + 'static,
{
	fn dispatch_scoped_head(
		&self,
		layer: Span<'a, RcBrand, Tag, RcRunExplicit<'a, R, S, A>>,
		_fo_handlers: &impl DispatchHandlers<
			'a,
			Apply!(
				<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, RcRunExplicit<'a, R, S, A>>
			),
			RcRunExplicit<'a, R, S, A>,
		>,
	) -> RcRunExplicit<'a, R, S, A> {
		match layer {
			Span::Span {
				tag: _tag,
				action,
			} => action(()),
		}
	}
}

impl<'a, R, S, A, Tag>
	DispatchScopedHandler<
		'a,
		SendSpan<'a, ArcBrand, Tag, ArcRunExplicit<'a, R, S, A>>,
		Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>),
		ArcRunExplicit<'a, R, S, A>,
	> for SpanDispatcher
where
	R: WrapDrop + SendFunctor + 'static,
	S: WrapDrop + SendFunctor + 'static,
	A: Send + Sync + 'a,
	Tag: Send + Sync + 'a + 'static,
{
	fn dispatch_scoped_head(
		&self,
		layer: SendSpan<'a, ArcBrand, Tag, ArcRunExplicit<'a, R, S, A>>,
		_fo_handlers: &impl DispatchHandlers<
			'a,
			Apply!(
				<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ArcRunExplicit<'a, R, S, A>>
			),
			ArcRunExplicit<'a, R, S, A>,
		>,
	) -> ArcRunExplicit<'a, R, S, A> {
		match layer {
			SendSpan::Span {
				tag: _tag,
				action,
			} => action(()),
		}
	}
}
