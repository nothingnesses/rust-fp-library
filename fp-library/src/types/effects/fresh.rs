//! Fresh first-order effect type.
//!
//! `Fresh<'a, P, Id, A>` requests one generated value of type `Id`
//! from the handler and continues with it. The standard W11 runner will
//! provide a zero-based `usize` counter, while the brand remains generic
//! in `Id` so typed identifiers can use the same effect family.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				BoxBrand,
				BoxFreshBrand,
				FreshBrand,
				SendFreshBrand,
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
		effect Fresh;
	}
}

pub use inner::*;
