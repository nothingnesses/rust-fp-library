//! Named ergonomic helpers layered over the Run wrapper primitives.
//!
//! This private module keeps small, user-facing helper methods out of
//! the already-large wrapper implementation files. The methods remain
//! inherent methods on the public wrapper types.

mod except;
mod reader;
mod state;
