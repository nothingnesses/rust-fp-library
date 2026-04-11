//! Data structures with two type arguments that can be traversed in an applicative context.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x: Result<i32, i32> = Ok(5);
//! let y = bi_traverse_explicit::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
//! 	(|e: i32| Some(e + 1), |s: i32| Some(s * 2)),
//! 	x,
//! );
//! assert_eq!(y, Some(Ok(10)));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			functions::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for data structures with two type arguments that can be traversed.
	///
	/// A `Bitraversable` represents a container with two type parameters whose elements
	/// can be traversed with effectful functions, accumulating results in an applicative
	/// context. A traversal requires two functions, one for each type argument.
	///
	/// ### Minimal Implementation
	///
	/// A minimal implementation requires [`Bitraversable::bi_traverse`] to be defined directly.
	/// [`Bitraversable::bi_sequence`] is derived from it via `identity`.
	///
	/// Note: defining both defaults creates a circular dependency and will not terminate.
	///
	/// ### Laws
	///
	/// `Bitraversable` instances must be consistent with `Bifunctor` and `Bifoldable`:
	/// * Traverse/sequence consistency: `bi_traverse(f, g, x) = bi_sequence(bimap(f, g, x))`.
	#[document_examples]
	///
	/// Bitraversable laws for [`Result`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // ResultBrand has Of<E, A> = Result<A, E>, so the first function handles errors
	/// // and the second function handles ok values.
	/// let f = |e: String| if e.is_empty() { None } else { Some(e.len()) };
	/// let g = |a: i32| if a > 0 { Some(a * 2) } else { None };
	///
	/// // Traverse/sequence consistency (Ok case):
	/// // bi_traverse((f, g), x) = bi_sequence(bimap((f, g), x))
	/// let ok: Result<i32, String> = Ok(5);
	/// assert_eq!(
	/// 	bi_traverse_explicit::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
	/// 		(f, g),
	/// 		ok.clone()
	/// 	),
	/// 	bi_sequence::<ResultBrand, _, _, OptionBrand>(bimap_explicit::<
	/// 		ResultBrand,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 	>((f, g), ok)),
	/// );
	///
	/// // Traverse/sequence consistency (Err case):
	/// let err: Result<i32, String> = Err("hello".to_string());
	/// assert_eq!(
	/// 	bi_traverse_explicit::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
	/// 		(f, g),
	/// 		err.clone()
	/// 	),
	/// 	bi_sequence::<ResultBrand, _, _, OptionBrand>(bimap_explicit::<
	/// 		ResultBrand,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 	>((f, g), err)),
	/// );
	/// ```
	pub trait Bitraversable: Bifunctor + Bifoldable {
		/// Traverses the bitraversable structure with two effectful functions.
		///
		/// This method applies `f` to values of the first type and `g` to values of the
		/// second type, accumulating all effects in the applicative context `F`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first-position elements.",
			"The type of the second-position elements.",
			"The output type for first-position elements.",
			"The output type for second-position elements.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function for first-position elements.",
			"The function for second-position elements.",
			"The bitraversable structure to traverse."
		)]
		///
		#[document_returns("The transformed structure wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Result<i32, i32> = Err(3);
		/// let y = bi_traverse_explicit::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
		/// 	(|e: i32| Some(e + 1), |s: i32| Some(s * 2)),
		/// 	x,
		/// );
		/// assert_eq!(y, Some(Err(4)));
		/// ```
		fn bi_traverse<
			'a,
			A: 'a + Clone,
			B: 'a + Clone,
			C: 'a + Clone,
			D: 'a + Clone,
			F: Applicative,
		>(
			f: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
			g: impl Fn(B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>);

		/// Sequences a bitraversable structure containing applicative values.
		///
		/// Collapses a structure of effectful values into a single effectful structure,
		/// applying `identity` to both positions via [`Bitraversable::bi_traverse`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first-position elements.",
			"The type of the second-position elements.",
			"The applicative context."
		)]
		///
		#[document_parameters("The bitraversable structure containing applicative values.")]
		///
		#[document_returns("The applicative context wrapping the bitraversable structure.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Result<Option<i32>, Option<i32>> = Ok(Some(5));
		/// let y = bi_sequence::<ResultBrand, _, _, OptionBrand>(x);
		/// assert_eq!(y, Some(Ok(5)));
		/// ```
		fn bi_sequence<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>), Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			Self::bi_traverse::<
				Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
				Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
				A,
				B,
				F,
			>(identity, identity, ta)
		}
	}

	/// Traverses the bitraversable structure with two effectful functions.
	///
	/// Free function version that dispatches to [the type class' associated function][`Bitraversable::bi_traverse`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The output type for first-position elements.",
		"The output type for second-position elements.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The function for first-position elements.",
		"The function for second-position elements.",
		"The bitraversable structure to traverse."
	)]
	///
	#[document_returns("The transformed structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<i32, i32> = Ok(5);
	/// let y = bi_traverse_explicit::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
	/// 	(|e: i32| Some(e + 1), |s: i32| Some(s * 2)),
	/// 	x,
	/// );
	/// assert_eq!(y, Some(Ok(10)));
	/// ```
	pub fn bi_traverse<
		'a,
		Brand: Bitraversable,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a + Clone,
		D: 'a + Clone,
		F: Applicative,
	>(
		f: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		g: impl Fn(B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
		p: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
	{
		Brand::bi_traverse::<A, B, C, D, F>(f, g, p)
	}

	/// Sequences a bitraversable structure containing applicative values.
	///
	/// Free function version that dispatches to [the type class' associated function][`Bitraversable::bi_sequence`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The applicative context."
	)]
	///
	#[document_parameters("The bitraversable structure containing applicative values.")]
	///
	#[document_returns("The applicative context wrapping the bitraversable structure.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<Option<i32>, Option<i32>> = Ok(Some(5));
	/// let y = bi_sequence::<ResultBrand, _, _, OptionBrand>(x);
	/// assert_eq!(y, Some(Ok(5)));
	/// ```
	pub fn bi_sequence<'a, Brand: Bitraversable, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
		ta: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>), Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>)>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
		Brand::bi_sequence::<A, B, F>(ta)
	}

	/// Traverses only the first-position elements, leaving second-position elements unchanged via `pure`.
	///
	/// Equivalent to `bi_traverse(f, pure, p)`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The type of the first-position elements.",
		"The type of the second-position elements (unchanged).",
		"The output type for first-position elements.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The function for first-position elements.",
		"The bitraversable structure to traverse."
	)]
	///
	#[document_returns("The transformed structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<i32, i32> = Err(3);
	/// let y = traverse_left::<ResultBrand, _, _, _, OptionBrand>(|e: i32| Some(e + 1), x);
	/// assert_eq!(y, Some(Err(4)));
	/// ```
	pub fn traverse_left<
		'a,
		Brand: Bitraversable,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a + Clone,
		F: Applicative,
	>(
		f: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		p: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, B>)>)
	{
		Brand::bi_traverse::<A, B, C, B, F>(f, |b| F::pure(b), p)
	}

	/// Traverses only the second-position elements, leaving first-position elements unchanged via `pure`.
	///
	/// Equivalent to `bi_traverse(pure, g, p)`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The type of the first-position elements (unchanged).",
		"The type of the second-position elements.",
		"The output type for second-position elements.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The function for second-position elements.",
		"The bitraversable structure to traverse."
	)]
	///
	#[document_returns("The transformed structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<i32, i32> = Ok(5);
	/// let y = traverse_right::<ResultBrand, _, _, _, OptionBrand>(|s: i32| Some(s * 2), x);
	/// assert_eq!(y, Some(Ok(10)));
	/// ```
	pub fn traverse_right<
		'a,
		Brand: Bitraversable,
		A: 'a + Clone,
		B: 'a + Clone,
		D: 'a + Clone,
		F: Applicative,
	>(
		g: impl Fn(B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
		p: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, D>)>)
	{
		Brand::bi_traverse::<A, B, A, D, F>(|a| F::pure(a), g, p)
	}

	/// Traverses the bitraversable structure with arguments flipped.
	///
	/// Equivalent to `bi_traverse(f, g, p)` with the structure argument first.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The output type for first-position elements.",
		"The output type for second-position elements.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The bitraversable structure to traverse.",
		"The function for first-position elements.",
		"The function for second-position elements."
	)]
	///
	#[document_returns("The transformed structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<i32, i32> = Ok(5);
	/// let y = bi_for::<ResultBrand, _, _, _, _, OptionBrand>(
	/// 	x,
	/// 	|e: i32| Some(e + 1),
	/// 	|s: i32| Some(s * 2),
	/// );
	/// assert_eq!(y, Some(Ok(10)));
	/// ```
	pub fn bi_for<
		'a,
		Brand: Bitraversable,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a + Clone,
		D: 'a + Clone,
		F: Applicative,
	>(
		p: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		f: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		g: impl Fn(B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
	{
		Brand::bi_traverse::<A, B, C, D, F>(f, g, p)
	}

	/// Traverses only the first-position elements with arguments flipped.
	///
	/// Equivalent to `traverse_left(f, p)` with the structure argument first.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The type of the first-position elements.",
		"The type of the second-position elements (unchanged).",
		"The output type for first-position elements.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The bitraversable structure to traverse.",
		"The function for first-position elements."
	)]
	///
	#[document_returns("The transformed structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<i32, i32> = Err(3);
	/// let y = for_left::<ResultBrand, _, _, _, OptionBrand>(x, |e: i32| Some(e + 1));
	/// assert_eq!(y, Some(Err(4)));
	/// ```
	pub fn for_left<
		'a,
		Brand: Bitraversable,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a + Clone,
		F: Applicative,
	>(
		p: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		f: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, B>)>)
	{
		Brand::bi_traverse::<A, B, C, B, F>(f, |b| F::pure(b), p)
	}

	/// Traverses only the second-position elements with arguments flipped.
	///
	/// Equivalent to `traverse_right(g, p)` with the structure argument first.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The type of the first-position elements (unchanged).",
		"The type of the second-position elements.",
		"The output type for second-position elements.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The bitraversable structure to traverse.",
		"The function for second-position elements."
	)]
	///
	#[document_returns("The transformed structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<i32, i32> = Ok(5);
	/// let y = for_right::<ResultBrand, _, _, _, OptionBrand>(x, |s: i32| Some(s * 2));
	/// assert_eq!(y, Some(Ok(10)));
	/// ```
	pub fn for_right<
		'a,
		Brand: Bitraversable,
		A: 'a + Clone,
		B: 'a + Clone,
		D: 'a + Clone,
		F: Applicative,
	>(
		p: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		g: impl Fn(B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, D>)>)
	{
		Brand::bi_traverse::<A, B, A, D, F>(|a| F::pure(a), g, p)
	}
}

pub use inner::*;
