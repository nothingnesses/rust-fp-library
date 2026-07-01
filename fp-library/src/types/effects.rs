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
pub mod coproduct;
pub(crate) mod fs1;
pub mod variant_f;

pub use variant_f::VariantF;
