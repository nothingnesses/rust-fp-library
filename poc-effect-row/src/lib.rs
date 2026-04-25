//! POC for the effect-row canonicalisation hybrid (workaround 1 +
//! workaround 3 from port-plan section 4.1).
//!
//! Re-exports the `effects!` macro and frunk's coproduct machinery
//! for use in tests.

pub use {
	effect_row_macros::effects,
	frunk_core::coproduct::{
		CNil,
		Coproduct,
		CoproductSubsetter,
	},
};
