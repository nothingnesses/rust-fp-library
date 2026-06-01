//! Log first-order effect type.
//!
//! `Log<'a, Message, A>` records one message and continues with the next
//! program value directly. The cell has no continuation closure, so one
//! `LogBrand<Message>` works across all Run wrapper families while
//! remaining distinct from `OutputBrand<Message>` and
//! `WriterBrand<Message>`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::LogBrand,
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
		effect Log;
	}
}

pub use inner::*;
