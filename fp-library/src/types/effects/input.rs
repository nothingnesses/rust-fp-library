//! Input first-order effect type.
//!
//! `Input<'a, P, Item, A>` requests one input value of type `Item`
//! from the handler and continues with it. The standard W11 sequence
//! runner targets `Option<Item>` so exhaustion is represented in the
//! row rather than hidden behind a panic or implicit failure policy.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				BoxBrand,
				BoxInputBrand,
				InputBrand,
				SendInputBrand,
			},
			classes::{
				Functor,
				Pointer,
				RefCountedPointer,
				SendFunctor,
				SendRefCountedPointer,
				ToDynCloneFn,
				ToDynFnOnce,
				ToDynSendFn,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	define_effect! {
		effect Input;
	}
}

pub use inner::*;
