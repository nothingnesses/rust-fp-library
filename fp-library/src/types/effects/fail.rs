//! Fixed-message failure first-order effect type.
//!
//! `Fail<'a, A>` aborts the current program with a `String` message and
//! carries no continuation. It remains a distinct row capability from
//! `Except<'a, String, A>` so users can choose between a fixed-message
//! failure channel and a typed exception channel at the API boundary.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FailBrand,
			classes::{
				Functor,
				SendFunctor,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	define_effect! {
		effect Fail;
	}
}

pub use inner::*;
