//! Getter optics for read-only access.
//!
//! A getter represents a way to view a value in a structure.

use {
	super::{
		base::{
			FoldOptic,
			GetterOptic,
			Optic,
		},
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

/// A polymorphic getter.
///
/// Matches PureScript's `Getter s t a b`.
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The source type of the structure.",
	"The target type of the structure.",
	"The source type of the focus.",
	"The target type of the focus."
)]
pub struct Getter<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a, {
	/// Function to view the focus of the getter in a structure.
	pub view_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, A>),
	pub(crate) _phantom: PhantomData<&'a (T, B)>,
}

impl<'a, P, S, T, A, B> Clone for Getter<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
{
	fn clone(&self) -> Self {
		Getter {
			view_fn: self.view_fn.clone(),
			_phantom: PhantomData,
		}
	}
}

impl<'a, P, S, T, A, B> Getter<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
{
	/// Create a new getter from a view function.
	#[document_signature]
	#[document_parameters("The view function.")]
	pub fn new(view: impl 'a + Fn(S) -> A) -> Self {
		Getter {
			view_fn: <FnBrand<P> as CloneableFn>::new(view),
			_phantom: PhantomData,
		}
	}

	/// View the focus of the getter in a structure.
	#[document_signature]
	#[document_parameters("The structure to view.")]
	pub fn view(
		&self,
		s: S,
	) -> A {
		(self.view_fn)(s)
	}
}

impl<'a, P, S, T, A, B, R> Optic<'a, ForgetBrand<R>, S, T, A, B> for Getter<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
	R: 'a + 'static,
{
	fn evaluate(
		&self,
		pab: Apply!(<ForgetBrand<R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, A, B>),
	) -> Apply!(<ForgetBrand<R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, T>) {
		let view_fn = self.view_fn.clone();
		use crate::classes::Profunctor;
		ForgetBrand::<R>::dimap(
			move |s: S| view_fn(s),
			|_b: B| unreachable!("Forget ignores the second function"),
			pab,
		)
	}
}

/// A concrete getter type where types do not change.
///
/// Matches PureScript's `Getter' s a`.
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The type of the structure.",
	"The type of the focus."
)]
pub struct GetterPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a, {
	/// Function to view the focus of the getter in a structure.
	pub view_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, A>),
	pub(crate) _phantom: PhantomData<P>,
}

impl<'a, P, S, A> Clone for GetterPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a,
{
	fn clone(&self) -> Self {
		GetterPrime {
			view_fn: self.view_fn.clone(),
			_phantom: PhantomData,
		}
	}
}

impl<'a, P, S, A> GetterPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a,
{
	/// Create a new monomorphic getter from a view function.
	#[document_signature]
	#[document_parameters("The view function.")]
	pub fn new(view: impl 'a + Fn(S) -> A) -> Self {
		GetterPrime {
			view_fn: <FnBrand<P> as CloneableFn>::new(view),
			_phantom: PhantomData,
		}
	}

	/// View the focus of the getter in a structure.
	#[document_signature]
	#[document_parameters("The structure to view.")]
	pub fn view(
		&self,
		s: S,
	) -> A {
		(self.view_fn)(s)
	}
}

impl<'a, P, S, A, R> Optic<'a, ForgetBrand<R>, S, S, A, A> for GetterPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a,
	R: 'a + 'static,
{
	fn evaluate(
		&self,
		pab: Apply!(<ForgetBrand<R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, A, A>),
	) -> Apply!(<ForgetBrand<R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>) {
		let view_fn = self.view_fn.clone();
		use crate::classes::Profunctor;
		ForgetBrand::<R>::dimap(
			move |s: S| view_fn(s),
			|_a: A| unreachable!("Forget ignores the second function"),
			pab,
		)
	}
}

impl<'a, P, S: 'a, A: 'a> GetterOptic<'a, S, A> for GetterPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
{
	fn evaluate<R: 'a + 'static>(
		&self,
		pab: Apply!(<ForgetBrand<R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, A, A>),
	) -> Apply!(<ForgetBrand<R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>) {
		Optic::<ForgetBrand<R>, S, S, A, A>::evaluate(self, pab)
	}
}

impl<'a, P, S: 'a, A: 'a> FoldOptic<'a, S, A> for GetterPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
{
	fn evaluate<R: 'a + Monoid + 'static>(
		&self,
		pab: Apply!(<ForgetBrand<R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, A, A>),
	) -> Apply!(<ForgetBrand<R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>) {
		Optic::<ForgetBrand<R>, S, S, A, A>::evaluate(self, pab)
	}
}
