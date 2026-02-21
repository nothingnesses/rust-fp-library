//! Getter optics for read-only access.
//!
//! A getter represents a way to view a value in a structure.

use {
	super::{
		base::Optic,
		forget::ForgetBrand,
	},
	fp_macros::document_type_parameters,
};

/// A polymorphic getter.
///
/// Matches PureScript's `Getter s t a b`.
#[document_type_parameters(
	"The lifetime of the values.",
	"The source type of the structure.",
	"The target type of the structure.",
	"The source type of the focus.",
	"The target type of the focus."
)]
pub type Getter<'a, S, T, A, B> = dyn Optic<'a, ForgetBrand<A>, S, T, A, B>;

/// A concrete getter type where types do not change.
///
/// Matches PureScript's `Getter' s a`.
pub type GetterPrime<'a, S, A> = Getter<'a, S, S, A, A>;
