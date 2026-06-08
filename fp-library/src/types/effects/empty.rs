//! Abortive first-order effect type for nondeterministic branches with
//! no results. The corresponding brand is
//! [`EmptyBrand`](crate::brands::EmptyBrand).
//!
//! `Empty<'a, A>` mirrors Heftia's separate `Empty` effect: the
//! operation has no continuation and never produces an `A`. This keeps
//! nondeterministic failure distinct from `Choose`, whose operation is
//! branching rather than abortive.
//!
//! ## No pointer-brand parameter
//!
//! Like [`Except`](crate::types::effects::except::Except), `Empty` has
//! no `dyn Fn` continuation. The same effect type and brand work across
//! all six Run wrappers; thread-safe wrappers rely only on the
//! structurally trivial [`SendFunctor`](crate::classes::SendFunctor)
//! implementation.
//!
//! See the parent [`effects`](crate::types::effects) guide for the
//! consolidated wrapper and pointer-brand conventions.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::EmptyBrand,
			classes::{
				Functor,
				SendFunctor,
			},
			impl_kind,
			kinds::*,
		},
		core::marker::PhantomData,
		fp_macros::*,
	};

	define_effect! {
		effect Empty;
	}
}

pub use inner::*;
