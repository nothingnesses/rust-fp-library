//! Monadic do-notation macro.
//!
//! Provides the [`m_do!`](crate::m_do) macro for flat monadic syntax that desugars
//! into nested [`bind`] calls, matching Haskell/PureScript `do` notation.
//!
//! Input parsing lives in
//! [`crate::support::do_input`](crate::support::do_input), shared with the
//! other do-notation macros (`a_do!`, `im_do!`, future `ia_do!`).

pub mod codegen;

pub use {
	crate::support::do_input::DoInput,
	codegen::m_do_worker,
};
