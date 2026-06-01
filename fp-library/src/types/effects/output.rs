//! Output first-order effect type.
//!
//! `Output<'a, Out, A>` emits one output value and carries the next
//! program value directly. The cell has no continuation closure, so one
//! `OutputBrand<Out>` works across all Run wrapper families.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::OutputBrand,
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
		effect Output;
	}
}

pub use inner::*;
