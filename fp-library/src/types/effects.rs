//! Effects subsystem: row-polymorphic first-order effects and heftia-style
//! scoped effects.
//!
//! See [decisions.md](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/decisions.md)
//! for the design rationale, and `fp-library/docs/run.md` (planned for
//! Phase 5 step 4) for the user guide.
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
//! - [`handlers`]: [`Handler<E, F>`](handlers::Handler) newtype plus
//!   the [`HandlersNil`] / [`HandlersCons<H, T>`](HandlersCons)
//!   cons-list runtime carrier for the `handlers!` macro and
//!   `nt().on::<E, _>(...)` builder fallback (Phase 3 step 1).
//! - `interpreter`: [`DispatchHandlers`] trait that walks a handler
//!   list against a row's value-level `Coproduct` chain, dispatching
//!   each variant to the matching handler closure (Phase 3 step 2).

pub mod arc_run;
pub mod arc_run_explicit;
pub mod coproduct;
pub mod handlers;
pub mod interpreter;
pub mod member;
pub mod node;
pub mod rc_run;
pub mod rc_run_explicit;
pub mod run;
pub mod run_explicit;
pub mod variant_f;

pub use {
	arc_run::ArcRun,
	arc_run_explicit::ArcRunExplicit,
	handlers::{
		Handler,
		HandlersCons,
		HandlersNil,
		nt,
	},
	interpreter::DispatchHandlers,
	node::Node,
	rc_run::RcRun,
	rc_run_explicit::RcRunExplicit,
	run::Run,
	run_explicit::RunExplicit,
	variant_f::VariantF,
};
