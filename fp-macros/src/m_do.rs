//! Monadic do-notation macro.
//!
//! Provides the [`m_do!`](crate::m_do) macro for flat monadic syntax that desugars
//! into nested [`bind`] calls, matching Haskell/PureScript `do` notation.

pub mod codegen;
pub mod input;

pub use {
	codegen::m_do_worker,
	input::DoInput,
};
