//! Code generation for the effects system.
//!
//! Holds the parsing and emission behind the
//! [`define_effect`](crate::define_effect) macro: one invocation defines one
//! effect (its brand, its operations enum, its kind projection, its `Functor`
//! instance, its order marker, and its row-generic smart constructors) from a
//! block of smart-constructor signatures, so the constructor surface, the
//! operations enum, and the order classification all derive from one source
//! of truth and cannot diverge.

pub(crate) mod define_effect;
