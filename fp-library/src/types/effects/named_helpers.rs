//! Named ergonomic helpers layered over the Run wrapper primitives.
//!
//! This private module keeps small, user-facing helper methods out of
//! the already-large wrapper implementation files. The methods remain
//! inherent methods on the public wrapper types.

mod except;
mod fresh;
mod input;
mod kv_store;
mod nondet;
mod output;
mod reader;
mod state;
mod writer;
