//! Error-throwing first-order effect type with the `Throw`
//! operation. The corresponding brand is
//! [`ExceptBrand`](crate::brands::ExceptBrand).
//!
//! `Except<'a, E, A>` mirrors PureScript Run's
//! `Except e a = Throw e` shape directly. The single `Throw`
//! variant carries an error value `E`; the result type `A` is
//! phantom because `Throw` never returns to the caller. The
//! `Functor` instance is structurally trivial: `map f (Throw e)`
//! produces `Throw e` (with the new result-type parameter).
//!
//! ## No pointer-brand parameter
//!
//! Unlike [`State`](crate::types::effects::state::State) or
//! [`Reader`](crate::types::effects::reader::Reader), `Except`
//! has no continuation, so it does not parameterise over a
//! pointer brand `P` and does not need a parallel
//! `SendExcept` for the Arc family. The same `Except<'a, E, A>`
//! type serves all six Run wrappers; per-wrapper smart
//! constructors handle the `Send + Sync` cascade on `E` alone.
//!
//! See the parent [`effects`](crate::types::effects) guide for the
//! consolidated wrapper and pointer-brand conventions.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::ExceptBrand,
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
		effect Except;
	}
}

pub use inner::*;
