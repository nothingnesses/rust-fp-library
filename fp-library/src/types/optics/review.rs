//! Review optics for constructing structures.
//!
//! A review represents a way to construct a structure from a focus value.

use {
	super::{
		base::Optic,
		tagged::TaggedBrand,
	},
	fp_macros::document_type_parameters,
};

/// A polymorphic review.
///
/// Matches PureScript's `Review s t a b`.
#[document_type_parameters(
	"The lifetime of the values.",
	"The source type of the structure.",
	"The target type of the structure.",
	"The source type of the focus.",
	"The target type of the focus."
)]
pub type Review<'a, S, T, A, B> = dyn Optic<'a, TaggedBrand, S, T, A, B>;

/// A concrete review type where types do not change.
///
/// Matches PureScript's `Review' s a`.
pub type ReviewPrime<'a, S, A> = Review<'a, S, S, A, A>;
