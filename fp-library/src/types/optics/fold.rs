//! Fold optics for collecting multiple values.
//!
//! A fold represents a way to focus on zero or more values in a structure.

use {
	super::{
		base::Optic,
		forget::ForgetBrand,
	},
	fp_macros::document_type_parameters,
};

/// A polymorphic fold.
///
/// Matches PureScript's `Fold r s t a b`.
#[document_type_parameters(
	"The lifetime of the values.",
	"The type of the monoid result.",
	"The source type of the structure.",
	"The target type of the structure.",
	"The source type of the focus.",
	"The target type of the focus."
)]
pub type Fold<'a, R, S, T, A, B> = dyn Optic<'a, ForgetBrand<R>, S, T, A, B>;

/// A concrete fold type where types do not change.
///
/// Matches PureScript's `Fold' r s a`.
pub type FoldPrime<'a, R, S, A> = Fold<'a, R, S, S, A, A>;
