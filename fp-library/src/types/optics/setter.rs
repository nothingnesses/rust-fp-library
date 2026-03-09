//! Setter optics for write-only access.
//!
//! A setter represents a way to update a value in a structure using a function.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{
				CloneableFn,
				Function,
				UnsizedCoercible,
				optics::*,
			},
			kinds::*,
		},
		fp_macros::*,
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
	pub struct Setter<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		/// Function to update the focus in a structure.
		pub over_fn: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, (S, Box<dyn Fn(A) -> B + 'a>), T>),
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus."
	)]
	#[document_parameters("The setter instance.")]
	impl<'a, PointerBrand, S, T, A, B> Clone for Setter<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_returns("A new `Setter` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Setter,
		/// };
		///
		/// let s: Setter<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Setter::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
		/// let cloned = s.clone();
		/// assert_eq!(cloned.over((42, "hi".to_string()), |x| x + 1), (43, "hi".to_string()));
		/// ```
		fn clone(&self) -> Self {
			Setter {
				over_fn: self.over_fn.clone(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus."
	)]
	#[document_parameters("The setter instance.")]
	impl<'a, PointerBrand, S, T, A, B> Setter<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		/// Create a new polymorphic setter.
		#[document_signature]
		///
		#[document_parameters("The over function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Setter,
		/// };
		///
		/// let s: Setter<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Setter::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
		/// assert_eq!(s.over((42, "hi".to_string()), |x| x + 1), (43, "hi".to_string()));
		/// ```
		pub fn new(over: impl 'a + Fn((S, Box<dyn Fn(A) -> B + 'a>)) -> T) -> Self {
			Setter {
				over_fn: <FnBrand<PointerBrand> as CloneableFn>::new(over),
			}
		}

		/// Update the focus of the setter in a structure using a function.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The function to apply to the focus.")]
		///
		#[document_returns("The updated structure.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Setter,
		/// };
		///
		/// let s: Setter<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Setter::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
		/// assert_eq!(s.over((42, "hi".to_string()), |x| x + 1), (43, "hi".to_string()));
		/// ```
		pub fn over(
			&self,
			s: S,
			f: impl Fn(A) -> B + 'a,
		) -> T {
			(self.over_fn)((s, Box::new(f)))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the setter.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The pointer brand for the function profunctor."
	)]
	#[document_parameters("The setter instance.")]
	impl<'a, Q, PointerBrand, S, T, A, B> Optic<'a, FnBrand<Q>, S, T, A, B>
		for Setter<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		Q: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
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
		/// let s: Setter<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Setter::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier = Optic::<RcFnBrand, _, _, _, _>::evaluate(&s, f);
		/// assert_eq!(modifier((42, "hi".to_string())), (43, "hi".to_string()));
		/// ```
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

	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the setter.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The pointer brand for the function profunctor."
	)]
	#[document_parameters("The setter instance.")]
	impl<'a, Q, PointerBrand, S, T, A, B> SetterOptic<'a, Q, S, T, A, B>
		for Setter<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		Q: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::optics::*,
		/// 		functions::*,
		/// 		types::optics::*,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let s: Setter<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Setter::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: Rc<dyn Fn((i32, String)) -> (i32, String)> =
		/// 	SetterOptic::<RcBrand, _, _, _, _>::evaluate(&s, f);
		/// assert_eq!(modifier((42, "hi".to_string())), (43, "hi".to_string()));
		/// ```
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
	pub struct SetterPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a, {
		/// Function to update the focus in a structure.
		pub over_fn: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, (S, Box<dyn Fn(A) -> A + 'a>), S>),
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The setter instance.")]
	impl<'a, PointerBrand, S, A> Clone for SetterPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_returns("A new `SetterPrime` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::SetterPrime,
		/// };
		///
		/// let s: SetterPrime<RcBrand, (i32, String), i32> =
		/// 	SetterPrime::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
		/// let cloned = s.clone();
		/// assert_eq!(cloned.over((42, "hi".to_string()), |x| x + 1), (43, "hi".to_string()));
		/// ```
		fn clone(&self) -> Self {
			SetterPrime {
				over_fn: self.over_fn.clone(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The setter instance.")]
	impl<'a, PointerBrand, S, A> SetterPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		/// Create a new monomorphic setter.
		#[document_signature]
		///
		#[document_parameters("The over function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::SetterPrime,
		/// };
		///
		/// let s: SetterPrime<RcBrand, (i32, String), i32> =
		/// 	SetterPrime::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
		/// assert_eq!(s.over((42, "hi".to_string()), |x| x + 1), (43, "hi".to_string()));
		/// ```
		pub fn new(over: impl 'a + Fn((S, Box<dyn Fn(A) -> A + 'a>)) -> S) -> Self {
			SetterPrime {
				over_fn: <FnBrand<PointerBrand> as CloneableFn>::new(over),
			}
		}

		/// Update the focus of the setter in a structure using a function.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The function to apply to the focus.")]
		///
		#[document_returns("The updated structure.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::SetterPrime,
		/// };
		///
		/// let s: SetterPrime<RcBrand, (i32, String), i32> =
		/// 	SetterPrime::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
		/// assert_eq!(s.over((42, "hi".to_string()), |x| x + 1), (43, "hi".to_string()));
		/// ```
		pub fn over(
			&self,
			s: S,
			f: impl Fn(A) -> A + 'a,
		) -> S {
			(self.over_fn)((s, Box::new(f)))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the setter.",
		"The type of the structure.",
		"The type of the focus.",
		"The pointer brand for the function profunctor."
	)]
	#[document_parameters("The setter instance.")]
	impl<'a, Q, PointerBrand, S, A> Optic<'a, FnBrand<Q>, S, S, A, A>
		for SetterPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		Q: UnsizedCoercible,
		S: 'a,
		A: 'a,
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
		/// let s: SetterPrime<RcBrand, (i32, String), i32> =
		/// 	SetterPrime::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier = Optic::<RcFnBrand, _, _, _, _>::evaluate(&s, f);
		/// assert_eq!(modifier((42, "hi".to_string())), (43, "hi".to_string()));
		/// ```
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

	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the setter.",
		"The type of the structure.",
		"The type of the focus.",
		"The pointer brand for the function profunctor."
	)]
	#[document_parameters("The setter instance.")]
	impl<'a, Q, PointerBrand, S, A> SetterOptic<'a, Q, S, S, A, A>
		for SetterPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		Q: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::optics::*,
		/// 		functions::*,
		/// 		types::optics::*,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let s: SetterPrime<RcBrand, (i32, String), i32> =
		/// 	SetterPrime::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: Rc<dyn Fn((i32, String)) -> (i32, String)> =
		/// 	SetterOptic::<RcBrand, _, _, _, _>::evaluate(&s, f);
		/// assert_eq!(modifier((42, "hi".to_string())), (43, "hi".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>) {
			Optic::<FnBrand<Q>, S, S, A, A>::evaluate(self, pab)
		}
	}
}
pub use inner::*;
