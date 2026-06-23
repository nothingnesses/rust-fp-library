//! Effects subsystem: row-polymorphic first-order effects and heftia-style
//! scoped effects.
//!
//! This subsystem is experimental: it is feature-gated behind the `effects`
//! crate feature (off by default), and its API is unstable and may change
//! between releases.
//!
//! ## Guide: rows, wrappers, and capabilities
//!
//! The subsystem represents a program with two type-level rows:
//!
//! - `R`, the first-order row. This is usually a
//!   [`CoproductBrand`](crate::brands::CoproductBrand) of
//!   [`CoyonedaBrand`](crate::brands::CoyonedaBrand),
//!   [`RcCoyonedaBrand`](crate::brands::RcCoyonedaBrand), or
//!   [`ArcCoyonedaBrand`](crate::brands::ArcCoyonedaBrand)-wrapped
//!   effect brands, ending in [`CNilBrand`](crate::brands::CNilBrand).
//! - `S`, the scoped row. This carries around-action effects such as
//!   Catch, Local, Bracket, Span, and Writer scoped operations. A program
//!   with no scoped effects uses [`CNilBrand`](crate::brands::CNilBrand).
//!
//! The six Run wrappers are the public storage axis:
//!
//! | Wrapper | Substrate | Continuations | Primary use |
//! | --- | --- | --- | --- |
//! | [`Run`] | Erased, `Free`, `'static` | `Box<dyn FnOnce>` | Default ergonomic single-shot programs. |
//! | [`RunExplicit`] | Explicit, `FreeExplicit`, non-`'static` friendly | `Box<dyn FnOnce>` | Generic code that needs Brand-dispatched classes. |
//! | [`RcRun`] | Erased, `RcFree`, `'static` | `Rc<dyn Fn>` | Single-threaded multi-shot programs. |
//! | [`RcRunExplicit`] | Explicit, `RcFreeExplicit`, non-`'static` friendly | `Rc<dyn Fn>` | Single-threaded multi-shot programs with Brand dispatch. |
//! | [`ArcRun`] | Erased, `ArcFree`, `'static` | `Arc<dyn Fn + Send + Sync>` | Thread-safe multi-shot programs. |
//! | [`ArcRunExplicit`] | Explicit, `ArcFreeExplicit`, non-`'static` friendly | `Arc<dyn Fn + Send + Sync>` | Thread-safe multi-shot programs with limited Brand dispatch. |
//!
//! "Erased" means the default family stores selected values behind
//! `Box<dyn Any>` internally so public `bind` remains O(1) for
//! left-associated chains. Safe constructors keep the erased value and
//! the pending continuation type aligned; see [`run::Run`]'s
//! representation docs for the invariant.
//! "Explicit" means the underlying `FreeExplicit` family stores the
//! recursive shape concretely, supports non-`'static` payloads, and is
//! the only Run family with Brand-level type-class dispatch.
//!
//! The wrapper Brand matrix is intentionally smaller than the inherent
//! method surface:
//!
//! | Brand | Implemented classes | Notable gaps |
//! | --- | --- | --- |
//! | `Run`, `RcRun`, `ArcRun` | No wrapper brands. Use inherent methods. | The erased Free family has no `FreeBrand`, `RcFreeBrand`, or `ArcFreeBrand`; Brand-generic code should use the Explicit family. |
//! | [`RunExplicitBrand`](crate::brands::RunExplicitBrand) | [`Functor`](crate::classes::Functor), [`Pointed`](crate::classes::Pointed), [`Semimonad`](crate::classes::Semimonad), [`RefFunctor`](crate::classes::RefFunctor), [`RefPointed`](crate::classes::RefPointed), [`RefSemimonad`](crate::classes::RefSemimonad). | No `Monad` / `RefMonad`, because `FreeExplicitBrand` deliberately has no `Applicative`. Ref classes require row brands that themselves support the Ref hierarchy. |
//! | [`RcRunExplicitBrand`](crate::brands::RcRunExplicitBrand) | [`Pointed`](crate::classes::Pointed), [`RefFunctor`](crate::classes::RefFunctor), [`RefPointed`](crate::classes::RefPointed), [`RefSemimonad`](crate::classes::RefSemimonad). | No owned `Functor` / `Semimonad`, because their trait methods cannot express the per-result `Clone` bound required by `RcFreeExplicit`. |
//! | [`ArcRunExplicitBrand`](crate::brands::ArcRunExplicitBrand) | [`SendPointed`](crate::classes::SendPointed), [`SendRefPointed`](crate::classes::SendRefPointed). | No `SendFunctor` / `SendSemimonad` / `SendRefFunctor` / `SendRefSemimonad`, because stable trait method signatures cannot carry the per-result `Send + Sync` projection bound required by `ArcFreeExplicit`. |
//!
//! ## Known limitations
//!
//! - Handler dispatch is mono-in-`A`, not a rank-2 natural
//!   transformation. This keeps handlers expressible as Rust closures;
//!   see [`interpreter`] for the model and escape hatches.
//! - `Box` wrappers are single-shot. Effects whose handlers must resume
//!   a continuation more than once, such as nondeterministic `Choose`,
//!   use the `Rc` or `Arc` wrappers.
//! - `ArcRunExplicitBrand` exposes only `SendPointed` and
//!   `SendRefPointed` at the Brand level. Use inherent `map` / `bind`
//!   methods at concrete `ArcRunExplicit` call sites.
//! - The erased `Run` family relies on private downcasts while stepping
//!   raw scoped boundaries. Safe APIs preserve the type invariant, but
//!   custom unsafe or crate-internal construction must keep erased
//!   values paired with continuations expecting that value type.
//! - Async interpretation is available on the default `Run` family via the
//!   [`Await`](await_future::Await) future base-lift effect: build a program
//!   with [`Run::await_future`](run::Run::await_future) and run it with
//!   [`Run::run_async`](run::Run::run_async), which awaits each embedded
//!   future and returns a runtime-agnostic future. It is a direct async
//!   driver loop (no `Future`-shaped `MonadRec`), and the await effect may
//!   sit at any position in the first-order row. The Rc / Arc wrapper family
//!   and scoped layers under async are not yet covered.
//!
//! ## Submodules
//!
//! - [`coproduct`]: Re-export adapter over [`frunk_core::coproduct`],
//!   surfacing the row-encoding types and trait family the rest of the
//!   subsystem consumes.
//! - [`variant_f`]: [`Functor`](crate::classes::Functor) and
//!   [`WrapDrop`](crate::classes::WrapDrop) impls for the Coproduct-row
//!   brands [`CNilBrand`](crate::brands::CNilBrand) and
//!   [`CoproductBrand`](crate::brands::CoproductBrand), plus the
//!   [`VariantF`] alias. This is the open sum of
//!   first-order effect functors that PureScript spells `VariantF`.
//! - [`member`]: [`Member<E, Idx>`](member::Member) trait for
//!   single-effect injection / projection over a Coproduct row,
//!   layered on top of the frunk
//!   [`CoprodInjector`](coproduct::CoprodInjector) and
//!   [`CoprodUninjector`](coproduct::CoprodUninjector) trait family.
//! - [`node`]: [`Node<'a, R, S, A>`](node::Node) enum and
//!   [`NodeBrand<R, S>`](crate::brands::NodeBrand) brand, the dual-row
//!   dispatch layer the Run family stores in its Free wrapper's `Wrap`
//!   arm.
//! - [`run`]: [`Run<R, S, A>`](run::Run), the Erased-substrate Run
//!   wrapper over [`Free<NodeBrand<R, S>, A>`](crate::types::Free).
//! - [`rc_run`]: [`RcRun<R, S, A>`](rc_run::RcRun), the multi-shot
//!   sibling of `Run` over [`RcFree`](crate::types::RcFree).
//! - [`arc_run`]: [`ArcRun<R, S, A>`](arc_run::ArcRun), the
//!   `Send + Sync` sibling of `Run` over
//!   [`ArcFree`](crate::types::ArcFree).
//! - [`run_explicit`]: [`RunExplicit<'a, R, S, A>`](run_explicit::RunExplicit),
//!   the Explicit-substrate Run wrapper over
//!   [`FreeExplicit`](crate::types::FreeExplicit).
//! - [`rc_run_explicit`]: [`RcRunExplicit<'a, R, S, A>`](rc_run_explicit::RcRunExplicit),
//!   the multi-shot Explicit sibling over
//!   [`RcFreeExplicit`](crate::types::RcFreeExplicit).
//! - [`arc_run_explicit`]: [`ArcRunExplicit<'a, R, S, A>`](arc_run_explicit::ArcRunExplicit),
//!   the `Send + Sync` Explicit sibling over
//!   [`ArcFreeExplicit`](crate::types::ArcFreeExplicit).
//! - [`scoped`]: [`ScopedCoproduct<H, T>`](scoped::ScopedCoproduct)
//!   and [`ScopedNil`] row-encoding aliases for
//!   the dual-row Run substrate's scoped-effect arm. Transparent
//!   aliases over [`CoproductBrand`](crate::brands::CoproductBrand)
//!   and [`CNilBrand`](crate::brands::CNilBrand); the distinct names
//!   document intent at user-facing type signatures so the dual
//!   row's first-order and scoped arms remain visually
//!   distinguished.
//! - [`handlers`]: [`Handler<E, F>`](handlers::Handler) newtype plus
//!   the [`HandlersNil`] / [`HandlersCons<H, T>`](HandlersCons)
//!   cons-list runtime carrier for the `handlers!` macro and
//!   `handlers_ordered().on::<E, _>(...).finish()` manual builder.
//!   `nt().prepend::<E, _>(...)` remains available for low-level
//!   representation construction. Also carries the
//!   parallel [`ScopedHandler`] /
//!   [`ScopedHandlersNil`] /
//!   [`ScopedHandlersCons<H, T>`](handlers::ScopedHandlersCons)
//!   runtime values for scoped-handler lists.
//! - `interpreter`: [`DispatchHandlers`], [`DispatchScopedHandlers`],
//!   and [`DispatchScopedBoundaryHandlers`] traits that walk first-order,
//!   scoped, and typed-boundary handler lists against value-level
//!   `Coproduct` chains.
//! - [`standard_scoped_handlers`]: standard handler values for built-in
//!   scoped effects such as Catch, Local, Bracket, and Span.
//! - [`await_future`]: [`Await`](await_future::Await) future base-lift
//!   first-order effect and the
//!   [`Run::await_future`](run::Run::await_future) constructor for embedding a
//!   `Future` into a program, interpreted by the crate's async driver and run
//!   via [`Run::run_async`](run::Run::run_async).
//! - [`coroutine`]: yield/resume first-order effect with pointer-brand
//!   siblings for single-shot, multi-shot, and thread-safe wrappers.
//! - [`empty`]: abortive first-order `Empty` effect used with
//!   nondeterministic programs to represent a branch with no results.
//! - [`fail`]: fixed-message aborting first-order effect.
//! - [`fresh`]: generated-value first-order effect.
//! - [`input`]: input-consuming first-order effect.
//! - [`kv_store`]: key-value-store first-order effect.
//! - [`log`]: direct-payload log-message first-order effect.
//! - [`output`]: output-emitting first-order effect.

pub mod arc_run;
pub mod arc_run_explicit;
pub(crate) mod async_interpreter;
pub mod await_future;
pub mod bracket;
pub mod catch;
pub mod choose;
pub mod coproduct;
pub mod coroutine;
pub mod empty;
pub mod except;
pub mod fail;
pub mod fresh;
pub(crate) mod fs1;
pub mod handlers;
pub mod input;
pub mod interpreter;
pub mod kv_store;
pub mod local;
pub mod log;
pub mod member;
mod named_helpers;
pub mod node;
pub mod output;
pub mod rc_run;
pub mod rc_run_explicit;
pub mod reader;
pub mod ref_bracket;
pub mod ref_local;
pub(crate) mod row_embed;
pub mod run;
pub mod run_explicit;
pub mod scoped;
pub mod span;
pub mod standard_scoped_handlers;
pub mod state;
pub mod variant_f;
pub mod writer;

pub use {
	arc_run::ArcRun,
	arc_run_explicit::ArcRunExplicit,
	handlers::{
		Handler,
		HandlersCons,
		HandlersNil,
		HandlersOrdered,
		ScopedHandler,
		ScopedHandlersCons,
		ScopedHandlersNil,
		ScopedHandlersOrdered,
		handlers_ordered,
		nt,
		scoped_handlers_ordered,
		scoped_nt,
	},
	interpreter::{
		DispatchHandlers,
		DispatchScopedBoundaryHandlers,
		DispatchScopedHandler,
		DispatchScopedHandlers,
	},
	node::Node,
	rc_run::RcRun,
	rc_run_explicit::RcRunExplicit,
	run::Run,
	run_explicit::RunExplicit,
	scoped::{
		ScopedCoproduct,
		ScopedNil,
	},
	variant_f::VariantF,
};
