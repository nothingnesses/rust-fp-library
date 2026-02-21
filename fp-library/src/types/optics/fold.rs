//! Fold optics for collecting multiple values.
//!
//! A fold represents a way to focus on zero or more values in a structure.

use {
	super::{
		base::FoldOptic,
		forget::ForgetBrand,
	},
	crate::{
		Apply,
		brands::FnBrand,
		classes::{
			CloneableFn,
			UnsizedCoercible,
			monoid::Monoid,
		},
		kinds::*,
	},
	fp_macros::{
		document_parameters,
		document_signature,
		document_type_parameters,
	},
	std::marker::PhantomData,
};

/// A polymorphic fold.
///
/// Matches PureScript's `Fold r s t a b`.
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The source type of the structure.",
	"The target type of the structure.",
	"The source type of the focus.",
	"The target type of the focus."
)]
pub struct Fold<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a, {
	/// Function that collects all focuses of the fold in a structure.
	pub to_vec_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, Vec<A>>),
	pub(crate) _phantom: PhantomData<&'a (T, B)>,
}

impl<'a, P, S, T, A, B> Clone for Fold<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
{
	fn clone(&self) -> Self {
		Fold {
			to_vec_fn: self.to_vec_fn.clone(),
			_phantom: PhantomData,
		}
	}
}

impl<'a, P, S, T, A, B> Fold<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
{
	/// Create a new polymorphic fold.
	#[document_signature]
	#[document_parameters("The to_vec function.")]
	pub fn new(to_vec: impl 'a + Fn(S) -> Vec<A>) -> Self {
		Fold {
			to_vec_fn: <FnBrand<P> as CloneableFn>::new(to_vec),
			_phantom: PhantomData,
		}
	}

	/// Collect all the focuses of the fold in a structure.
	#[document_signature]
	#[document_parameters("The structure to fold.")]
	pub fn to_vec(
		&self,
		s: S,
	) -> Vec<A> {
		(self.to_vec_fn)(s)
	}
}

impl<'a, P, S, T, A, B> FoldOptic<'a, S, A> for Fold<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
{
	fn evaluate<R: 'a + Monoid + 'static>(
		&self,
		pab: Apply!(<ForgetBrand<R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, A>),
	) -> Apply!(<ForgetBrand<R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>) {
		let to_vec = self.to_vec_fn.clone();
		super::forget::Forget::new(move |s: S| {
			let mut result = R::empty();
			for a in to_vec(s) {
				result = R::append(result, (pab.0)(a));
			}
			result
		})
	}
}

/// A concrete fold type where types do not change.
///
/// Matches PureScript's `Fold' r s a`.
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The type of the structure.",
	"The type of the focus."
)]
pub struct FoldPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a, {
	/// Function that collects all focuses of the fold in a structure.
	pub to_vec_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, Vec<A>>),
	pub(crate) _phantom: PhantomData<P>,
}

impl<'a, P, S, A> Clone for FoldPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a,
{
	fn clone(&self) -> Self {
		FoldPrime {
			to_vec_fn: self.to_vec_fn.clone(),
			_phantom: PhantomData,
		}
	}
}

impl<'a, P, S, A> FoldPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a,
{
	/// Create a new monomorphic fold.
	#[document_signature]
	#[document_parameters("The to_vec function.")]
	pub fn new(to_vec: impl 'a + Fn(S) -> Vec<A>) -> Self {
		FoldPrime {
			to_vec_fn: <FnBrand<P> as CloneableFn>::new(to_vec),
			_phantom: PhantomData,
		}
	}

	/// Collect all the focuses of the fold in a structure.
	#[document_signature]
	#[document_parameters("The structure to fold.")]
	pub fn to_vec(
		&self,
		s: S,
	) -> Vec<A> {
		(self.to_vec_fn)(s)
	}
}

impl<'a, P, S, A> FoldOptic<'a, S, A> for FoldPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a,
{
	fn evaluate<R: 'a + Monoid + 'static>(
		&self,
		pab: Apply!(<ForgetBrand<R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, A>),
	) -> Apply!(<ForgetBrand<R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>) {
		let to_vec = self.to_vec_fn.clone();
		super::forget::Forget::new(move |s: S| {
			let mut result = R::empty();
			for a in to_vec(s) {
				result = R::append(result, (pab.0)(a));
			}
			result
		})
	}
}
