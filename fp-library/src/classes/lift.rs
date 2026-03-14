//! Lifting of functions to operate on values within a context.
//!
//! Provides [`lift2`] through [`lift5`] for lifting multi-argument functions
//! into a context. Higher-arity lifts are built from [`lift2`] using tuple
//! intermediaries.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Some(1);
//! let y = Some(2);
//! let z = lift2::<OptionBrand, _, _, _>(|a, b| a + b, x, y);
//! assert_eq!(z, Some(3));
//!
//! let w = lift3::<OptionBrand, _, _, _, _>(|a, b, c| a + b + c, Some(1), Some(2), Some(3));
//! assert_eq!(w, Some(6));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for lifting binary functions into a context.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait Lift {
		/// Lifts a binary function into the context.
		///
		/// This method lifts a binary function to operate on values within the context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the second value.",
			"The type of the result."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first context.",
			"The second context."
		)]
		///
		#[document_returns("A new context containing the result of applying the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(1);
		/// let y = Some(2);
		/// let z = lift2::<OptionBrand, _, _, _>(|a, b| a + b, x, y);
		/// assert_eq!(z, Some(3));
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: Clone + 'a,
			B: Clone + 'a,
			C: 'a;
	}

	/// Lifts a binary function into the context.
	///
	/// Free function version that dispatches to [the type class' associated function][`Lift::lift2`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the result."
	)]
	///
	#[document_parameters(
		"The binary function to apply.",
		"The first context.",
		"The second context."
	)]
	///
	#[document_returns("A new context containing the result of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(1);
	/// let y = Some(2);
	/// let z = lift2::<OptionBrand, _, _, _>(|a, b| a + b, x, y);
	/// assert_eq!(z, Some(3));
	/// ```
	pub fn lift2<'a, Brand: Lift, A, B, C>(
		func: impl Fn(A, B) -> C + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
	where
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a, {
		Brand::lift2(func, fa, fb)
	}

	/// Lifts a ternary function into the context.
	///
	/// Applies a three-argument function to three contextual values.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the third value.",
		"The type of the result."
	)]
	///
	#[document_parameters(
		"The ternary function to apply.",
		"The first context.",
		"The second context.",
		"The third context."
	)]
	///
	#[document_returns("A new context containing the result of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let result = lift3::<OptionBrand, _, _, _, _>(|a, b, c| a + b + c, Some(1), Some(2), Some(3));
	/// assert_eq!(result, Some(6));
	///
	/// let result =
	/// 	lift3::<OptionBrand, _, _, _, _>(|a: i32, b: i32, c| a + b + c, Some(1), None, Some(3));
	/// assert_eq!(result, None);
	/// ```
	pub fn lift3<'a, Brand: Lift, A, B, C, D>(
		func: impl Fn(A, B, C) -> D + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>)
	where
		A: Clone + 'a,
		B: Clone + 'a,
		C: Clone + 'a,
		D: 'a, {
		Brand::lift2(move |(a, b), c| func(a, b, c), Brand::lift2(|a, b| (a, b), fa, fb), fc)
	}

	/// Lifts a quaternary function into the context.
	///
	/// Applies a four-argument function to four contextual values.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the third value.",
		"The type of the fourth value.",
		"The type of the result."
	)]
	///
	#[document_parameters(
		"The quaternary function to apply.",
		"The first context.",
		"The second context.",
		"The third context.",
		"The fourth context."
	)]
	///
	#[document_returns("A new context containing the result of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let result = lift4::<OptionBrand, _, _, _, _, _>(
	/// 	|a, b, c, d| a + b + c + d,
	/// 	Some(1),
	/// 	Some(2),
	/// 	Some(3),
	/// 	Some(4),
	/// );
	/// assert_eq!(result, Some(10));
	/// ```
	pub fn lift4<'a, Brand: Lift, A, B, C, D, E>(
		func: impl Fn(A, B, C, D) -> E + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
		fd: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>)
	where
		A: Clone + 'a,
		B: Clone + 'a,
		C: Clone + 'a,
		D: Clone + 'a,
		E: 'a, {
		Brand::lift2(
			move |(a, b, c), d| func(a, b, c, d),
			lift3::<Brand, A, B, C, (A, B, C)>(|a, b, c| (a, b, c), fa, fb, fc),
			fd,
		)
	}

	/// Lifts a quinary function into the context.
	///
	/// Applies a five-argument function to five contextual values.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the third value.",
		"The type of the fourth value.",
		"The type of the fifth value.",
		"The type of the result."
	)]
	///
	#[document_parameters(
		"The quinary function to apply.",
		"The first context.",
		"The second context.",
		"The third context.",
		"The fourth context.",
		"The fifth context."
	)]
	///
	#[document_returns("A new context containing the result of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let result = lift5::<OptionBrand, _, _, _, _, _, _>(
	/// 	|a, b, c, d, e| a + b + c + d + e,
	/// 	Some(1),
	/// 	Some(2),
	/// 	Some(3),
	/// 	Some(4),
	/// 	Some(5),
	/// );
	/// assert_eq!(result, Some(15));
	/// ```
	pub fn lift5<'a, Brand: Lift, A, B, C, D, E, F>(
		func: impl Fn(A, B, C, D, E) -> F + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
		fd: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
		fe: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, F>)
	where
		A: Clone + 'a,
		B: Clone + 'a,
		C: Clone + 'a,
		D: Clone + 'a,
		E: Clone + 'a,
		F: 'a, {
		Brand::lift2(
			move |(a, b, c, d), e| func(a, b, c, d, e),
			lift4::<Brand, A, B, C, D, (A, B, C, D)>(|a, b, c, d| (a, b, c, d), fa, fb, fc, fd),
			fe,
		)
	}
}

pub use inner::*;
