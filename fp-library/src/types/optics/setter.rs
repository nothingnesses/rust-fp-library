//! Setter optics for write-only access.
//!
//! A setter represents a way to update a value in a structure using a function.

use {
	super::base::Optic,
	crate::brands::FnBrand,
	fp_macros::document_type_parameters,
};

/// A polymorphic setter.
///
/// Matches PureScript's `Setter s t a b`.
#[document_type_parameters(
	"The lifetime of the values.",
	"The pointer brand for the function.",
	"The source type of the structure.",
	"The target type of the structure.",
	"The source type of the focus.",
	"The target type of the focus."
)]
pub type Setter<'a, P, S, T, A, B> = dyn Optic<'a, FnBrand<P>, S, T, A, B>;

/// A concrete setter type where types do not change.
///
/// Matches PureScript's `Setter' s a`.
pub type SetterPrime<'a, P, S, A> = Setter<'a, P, S, S, A, A>;
