//! Setter optics for write-only access.
//!
//! A setter represents a way to update a value in a structure using a function.

use {
	super::base::{
		Optic,
		SetterOptic,
	},
	crate::{
		Apply,
		brands::FnBrand,
		classes::{
			CloneableFn,
			Function,
			UnsizedCoercible,
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
pub struct Setter<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a, {
	/// Function to update the focus in a structure.
	pub over_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, (S, Box<dyn Fn(A) -> B + 'a>), T>),
	pub(crate) _phantom: PhantomData<P>,
}

impl<'a, P, S, T, A, B> Clone for Setter<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
{
	fn clone(&self) -> Self {
		Setter {
			over_fn: self.over_fn.clone(),
			_phantom: PhantomData,
		}
	}
}

impl<'a, P, S, T, A, B> Setter<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
{
	/// Create a new polymorphic setter.
	#[document_signature]
	#[document_parameters("The over function.")]
	pub fn new(over: impl 'a + Fn((S, Box<dyn Fn(A) -> B + 'a>)) -> T) -> Self {
		Setter {
			over_fn: <FnBrand<P> as CloneableFn>::new(over),
			_phantom: PhantomData,
		}
	}

	/// Update the focus of the setter in a structure using a function.
	#[document_signature]
	#[document_parameters("The structure to update.", "The function to apply to the focus.")]
	pub fn over(
		&self,
		s: S,
		f: impl Fn(A) -> B + 'a,
	) -> T {
		(self.over_fn)((s, Box::new(f)))
	}
}

impl<'a, Q, P, S, T, A, B> Optic<'a, FnBrand<Q>, S, T, A, B> for Setter<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	Q: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
{
	fn evaluate(
		&self,
		pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, B>),
	) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, T>) {
		let over = self.over_fn.clone();
		<FnBrand<Q> as Function>::new(move |s: S| {
			let pab_clone = pab.clone();
			over((s, Box::new(move |a| pab_clone(a))))
		})
	}
}

impl<'a, Q, P, S, T, A, B> SetterOptic<'a, Q, S, T, A, B> for Setter<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	Q: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
{
	fn evaluate(
		&self,
		pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, B>),
	) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, T>) {
		Optic::<FnBrand<Q>, S, T, A, B>::evaluate(self, pab)
	}
}

/// A concrete setter type where types do not change.
///
/// Matches PureScript's `Setter' s a`.
#[document_type_parameters(
	"The lifetime of the values.",
	"The pointer brand for the function.",
	"The type of the structure.",
	"The type of the focus."
)]
pub struct SetterPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a, {
	/// Function to update the focus in a structure.
	pub over_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, (S, Box<dyn Fn(A) -> A + 'a>), S>),
	pub(crate) _phantom: PhantomData<P>,
}

impl<'a, P, S, A> Clone for SetterPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a,
{
	fn clone(&self) -> Self {
		SetterPrime {
			over_fn: self.over_fn.clone(),
			_phantom: PhantomData,
		}
	}
}

impl<'a, P, S, A> SetterPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a,
{
	/// Create a new monomorphic setter.
	#[document_signature]
	#[document_parameters("The over function.")]
	pub fn new(over: impl 'a + Fn((S, Box<dyn Fn(A) -> A + 'a>)) -> S) -> Self {
		SetterPrime {
			over_fn: <FnBrand<P> as CloneableFn>::new(over),
			_phantom: PhantomData,
		}
	}

	/// Update the focus of the setter in a structure using a function.
	#[document_signature]
	#[document_parameters("The structure to update.", "The function to apply to the focus.")]
	pub fn over(
		&self,
		s: S,
		f: impl Fn(A) -> A + 'a,
	) -> S {
		(self.over_fn)((s, Box::new(f)))
	}
}

impl<'a, Q, P, S, A> Optic<'a, FnBrand<Q>, S, S, A, A> for SetterPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	Q: UnsizedCoercible,
	S: 'a,
	A: 'a,
{
	fn evaluate(
		&self,
		pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, A>),
	) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>) {
		let over = self.over_fn.clone();
		<FnBrand<Q> as Function>::new(move |s: S| {
			let pab_clone = pab.clone();
			over((s, Box::new(move |a| pab_clone(a))))
		})
	}
}

impl<'a, Q, P, S, A> SetterOptic<'a, Q, S, S, A, A> for SetterPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	Q: UnsizedCoercible,
	S: 'a,
	A: 'a,
{
	fn evaluate(
		&self,
		pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, A>),
	) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>) {
		Optic::<FnBrand<Q>, S, S, A, A>::evaluate(self, pab)
	}
}
