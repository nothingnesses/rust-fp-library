//! Effects subsystem: the unified-row (FS-1) effect slice and its row-encoding
//! support.
//!
//! This subsystem is experimental: it is feature-gated behind the `effects`
//! crate feature (off by default), and its API is unstable and may change
//! between releases.
//!
//! The subsystem exposes the `pub(crate)` `fs1` vertical slice: a single
//! unified effect row with per-brand order markers, higher-order effects
//! elaborated into first-order ones over that row (no boundary frames), and
//! brand-keyed dispatch, built on the crate's [`Free`](crate::types::Free)
//! substrate. It is accompanied by the row-encoding support the slice needs:
//!
//! - [`coproduct`]: re-export adapter over [`frunk_core::coproduct`],
//!   surfacing the row-encoding types the unified row is built from.
//! - [`handle`]: the generic interpretation surface, the narrowing
//!   accumulator runner ([`handle_accum`](handle::handle_accum)), the
//!   forking-step abstraction ([`AccumStep`](handle::AccumStep)), the
//!   terminal extractor ([`extract`](handle::extract)), the
//!   continuation-passing driver ([`run_cont`](handle::run_cont)), and the
//!   one-pass handler surface's traits ([`RowHandler`](handle::RowHandler),
//!   [`HandlerPieces`](handle::HandlerPieces), and
//!   [`EffectAbort`](handle::EffectAbort)) that `#[handlers]` rows and the
//!   per-effect emissions compose through.
//! - [`choose`]: the public scoped `Choose` effect (owned branches, resumed
//!   exactly once with the surviving branch values, plus the branch-killing
//!   `empty`) and its narrowing runners
//!   ([`handle_choose`](choose::handle_choose), the first-success
//!   [`handle_choose_first`](choose::handle_choose_first), and the
//!   accumulator-forking
//!   [`handle_choose_accum`](choose::handle_choose_accum)).
//! - [`coroutine`]: the public `Coroutine` effect (cooperative yielding,
//!   emitting an `Out` and resuming with an `In`) and its yielded-or-done
//!   step runner ([`handle_coroutine`](coroutine::handle_coroutine), whose
//!   [`Resume`](coroutine::Resume) continuation is pre-folded through the
//!   runner).
//! - [`streaming`]: the streaming vocabulary over the coroutine functor,
//!   the producer/consumer pin aliases ([`Yield`](streaming::Yield) and
//!   [`Await`](streaming::Await), with [`await_value`](streaming::await_value)),
//!   the fusion primitives ([`interleave`](streaming::interleave) and
//!   [`substitute`](streaming::substitute)), and the conveniences
//!   ([`connect`](streaming::connect) and [`for_each`](streaming::for_each)).
//! - [`state`]: the public `State` effect, its threaded narrowing runner
//!   ([`handle_state`](state::handle_state)), and the transactional
//!   combinator ([`transact_state`](state::transact_state)), whose commit
//!   rides in the continuation an abort discards, so state rolls back
//!   structurally.
//! - [`writer`]: the public `Writer` effect, its folding narrowing runners
//!   ([`fold_writer`](writer::fold_writer) and
//!   [`handle_writer`](writer::handle_writer)), and the forking-step form of
//!   the fold ([`FoldWriterStep`](writer::FoldWriterStep)).
//! - [`tagged`]: tagged (labelled) effects, the
//!   [`TaggedBrand`](tagged::TaggedBrand) wrapper whose identity changes the
//!   dispatch key while everything else (operations, `Functor`, order
//!   marker, abort, handler pieces, steps) delegates to the bare effect, so
//!   the same effect appears in one row once per label.
//! - [`order`]: the first-order versus higher-order classification markers
//!   ([`OrderOf`](order::OrderOf) and its [`FirstOrder`](order::FirstOrder) /
//!   [`HigherOrder`](order::HigherOrder) marker types) every effect brand
//!   carries.
//! - [`variant_f`]: [`Functor`](crate::classes::Functor) and
//!   [`WrapDrop`](crate::classes::WrapDrop) impls for the Coproduct-row
//!   brands [`CNilBrand`](crate::brands::CNilBrand) and
//!   [`CoproductBrand`](crate::brands::CoproductBrand), plus the
//!   [`VariantF`] alias, the open sum of first-order effect functors that
//!   PureScript spells `VariantF`.
//! - [`await_future`]: the [`Await`](await_future::Await) future base-lift
//!   effect with its [`Functor`](crate::classes::Functor), the row-generic
//!   [`await_future`](await_future::await_future) constructor, and the
//!   [`run_async`](await_future::run_async) terminal driver that finishes
//!   an `Await`-only row.

pub mod await_future;
pub mod choose;
pub mod coproduct;
pub mod coroutine;
pub(crate) mod fs1;
pub mod handle;
pub mod order;
pub mod state;
pub mod streaming;
pub mod tagged;
pub mod variant_f;
pub mod writer;

pub use variant_f::VariantF;
