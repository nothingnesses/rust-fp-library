//! Getter optics for read-only access.
//!
//! A getter represents a way to view a value in a structure.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{
				CloneableFn,
				UnsizedCoercible,
				monoid::Monoid,
				optics::*,
			},
			kinds::*,
			types::optics::{
				Forget,
				ForgetBrand,
			},
		},
		fp_macros::*,
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
	pub struct Getter<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		/// Function to view the focus of the getter in a structure.
		pub view_fn: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, A>),
		pub(crate) _phantom: PhantomData<&'a (T, B)>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus."
	)]
	#[document_parameters("The getter instance.")]
	impl<'a, PointerBrand, S, T, A, B> Clone for Getter<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_returns("A new `Getter` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Getter,
		/// };
		///
		/// let g: Getter<RcBrand, (i32, String), (i32, String), i32, i32> = Getter::new(|(x, _)| x);
		/// let cloned = g.clone();
		/// assert_eq!(cloned.view((42, "hi".to_string())), 42);
		/// ```
		fn clone(&self) -> Self {
			Getter {
				view_fn: self.view_fn.clone(),
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus."
	)]
	#[document_parameters("The getter instance.")]
	impl<'a, PointerBrand, S, T, A, B> Getter<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		/// Create a new getter from a view function.
		#[document_signature]
		///
		#[document_parameters("The view function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Getter,
		/// };
		///
		/// let g: Getter<RcBrand, (i32, String), (i32, String), i32, i32> = Getter::new(|(x, _)| x);
		/// assert_eq!(g.view((42, "hi".to_string())), 42);
		/// ```
		pub fn new(view: impl 'a + Fn(S) -> A) -> Self {
			Getter {
				view_fn: <FnBrand<PointerBrand> as CloneableFn>::new(view),
				_phantom: PhantomData,
			}
		}

		/// View the focus of the getter in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to view.")]
		///
		#[document_returns("The focus value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Getter,
		/// };
		///
		/// let g: Getter<RcBrand, (i32, String), (i32, String), i32, i32> = Getter::new(|(x, _)| x);
		/// assert_eq!(g.view((42, "hi".to_string())), 42);
		/// ```
		pub fn view(
			&self,
			s: S,
		) -> A {
			(self.view_fn)(s)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the getter.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The return type of the forget profunctor.",
		"The reference-counted pointer type for the forget brand."
	)]
	#[document_parameters("The getter instance.")]
	impl<'a, PointerBrand, S, T, A, B, R, Q> Optic<'a, ForgetBrand<Q, R>, S, T, A, B>
		for Getter<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		Q: UnsizedCoercible + 'static,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
		R: 'a + 'static,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let g: Getter<RcBrand, (i32, String), (i32, String), i32, i32> = Getter::new(|(x, _)| x);
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded = Optic::<ForgetBrand<RcBrand, i32>, _, _, _, _>::evaluate(&g, f);
		/// assert_eq!(folded.run((42, "hi".to_string())), 42);
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, T>)
		{
			let view_fn = self.view_fn.clone();
			Forget::new(move |s: S| pab.run(view_fn(s)))
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
	pub struct GetterPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a, {
		/// Function to view the focus of the getter in a structure.
		pub view_fn: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, A>),
		pub(crate) _phantom: PhantomData<PointerBrand>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The getter instance.")]
	impl<'a, PointerBrand, S, A> Clone for GetterPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_returns("A new `GetterPrime` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::GetterPrime,
		/// };
		///
		/// let g: GetterPrime<RcBrand, (i32, String), i32> = GetterPrime::new(|(x, _)| x);
		/// let cloned = g.clone();
		/// assert_eq!(cloned.view((42, "hi".to_string())), 42);
		/// ```
		fn clone(&self) -> Self {
			GetterPrime {
				view_fn: self.view_fn.clone(),
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The getter instance.")]
	impl<'a, PointerBrand, S, A> GetterPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		/// Create a new monomorphic getter from a view function.
		#[document_signature]
		///
		#[document_parameters("The view function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::GetterPrime,
		/// };
		///
		/// let g: GetterPrime<RcBrand, (i32, String), i32> = GetterPrime::new(|(x, _)| x);
		/// assert_eq!(g.view((42, "hi".to_string())), 42);
		/// ```
		pub fn new(view: impl 'a + Fn(S) -> A) -> Self {
			GetterPrime {
				view_fn: <FnBrand<PointerBrand> as CloneableFn>::new(view),
				_phantom: PhantomData,
			}
		}

		/// View the focus of the getter in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to view.")]
		///
		#[document_returns("The focus value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::GetterPrime,
		/// };
		///
		/// let g: GetterPrime<RcBrand, (i32, String), i32> = GetterPrime::new(|(x, _)| x);
		/// assert_eq!(g.view((42, "hi".to_string())), 42);
		/// ```
		pub fn view(
			&self,
			s: S,
		) -> A {
			(self.view_fn)(s)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the getter.",
		"The type of the structure.",
		"The type of the focus.",
		"The return type of the forget profunctor.",
		"The reference-counted pointer type for the forget brand."
	)]
	#[document_parameters("The getter instance.")]
	impl<'a, PointerBrand, S, A, R, Q> Optic<'a, ForgetBrand<Q, R>, S, S, A, A>
		for GetterPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		Q: UnsizedCoercible + 'static,
		S: 'a,
		A: 'a,
		R: 'a + 'static,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let g: GetterPrime<RcBrand, (i32, String), i32> = GetterPrime::new(|(x, _)| x);
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded = Optic::<ForgetBrand<RcBrand, i32>, _, _, _, _>::evaluate(&g, f);
		/// assert_eq!(folded.run((42, "hi".to_string())), 42);
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>)
		{
			let view_fn = self.view_fn.clone();
			Forget::new(move |s: S| pab.run(view_fn(s)))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The getter instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> GetterOptic<'a, S, A> for GetterPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters(
			"The return type of the forget profunctor.",
			"The reference-counted pointer type."
		)]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let g: GetterPrime<RcBrand, (i32, String), i32> = GetterPrime::new(|(x, _)| x);
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded = GetterOptic::evaluate(&g, f);
		/// assert_eq!(folded.run((42, "hi".to_string())), 42);
		/// ```
		fn evaluate<R: 'a + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>)
		{
			Optic::<ForgetBrand<Q, R>, S, S, A, A>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The getter instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> FoldOptic<'a, S, A> for GetterPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters(
			"The monoid type.",
			"The reference-counted pointer type for the Forget brand."
		)]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let g: GetterPrime<RcBrand, (i32, String), i32> = GetterPrime::new(|(x, _)| x);
		/// let f = Forget::<RcBrand, String, i32, i32>::new(|x: i32| x.to_string());
		/// let folded: Forget<RcBrand, String, (i32, String), (i32, String)> = FoldOptic::evaluate(&g, f);
		/// assert_eq!(folded.run((42, "hi".to_string())), "42".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			Optic::<ForgetBrand<Q, R>, S, S, A, A>::evaluate(self, pab)
		}
	}
}
pub use inner::*;
