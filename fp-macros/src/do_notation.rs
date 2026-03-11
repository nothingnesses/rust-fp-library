//! Monadic do-notation macro.
//!
//! Provides the [`m!`](crate::m) macro for flat monadic syntax that desugars
//! into nested [`bind`] calls, matching Haskell/PureScript `do` notation.

pub mod codegen;
pub mod input;

pub use {
	codegen::m_worker,
	input::DoInput,
};
