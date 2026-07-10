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
//!   forking-step abstraction ([`AccumStep`](handle::AccumStep)), and the
//!   terminal extractor ([`extract`](handle::extract)).
//! - [`choose`]: the public scoped `Choose` effect (owned branches, resumed
//!   exactly once with the surviving branch values, plus the branch-killing
//!   `empty`) and its narrowing runners
//!   ([`handle_choose`](choose::handle_choose) and the accumulator-forking
//!   [`handle_choose_accum`](choose::handle_choose_accum)).
//! - [`state`]: the public `State` effect and its threaded narrowing runner
//!   ([`handle_state`](state::handle_state)).
//! - [`writer`]: the public `Writer` effect and its folding narrowing runners
//!   ([`fold_writer`](writer::fold_writer) and
//!   [`handle_writer`](writer::handle_writer)).
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
//!   effect and its [`Functor`](crate::classes::Functor), the
//!   substrate-agnostic piece an async driver awaits.

pub mod await_future;
pub mod choose;
pub mod coproduct;
pub(crate) mod fs1;
pub mod handle;
pub mod order;
pub mod state;
pub mod variant_f;
pub mod writer;

pub use variant_f::VariantF;
