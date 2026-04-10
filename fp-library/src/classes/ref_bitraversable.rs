//! By-reference traversal of bifunctor structures.
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
//! let y = bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
//! 	(|e: &i32| Some(e + 1), |s: &i32| Some(s * 2)),
//! 	&x,
//! );
//! assert_eq!(y, Some(Ok(10)));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for data structures with two type arguments that can be traversed by reference.
	///
	/// Similar to [`Bitraversable`], but closures receive `&A` and `&B` instead of owned values.
	/// This enables traversing bifunctor structures by reference without consuming elements.
	///
	/// ### Minimal Implementation
	///
	/// A minimal implementation requires [`RefBitraversable::ref_bi_traverse`] to be defined directly.
	/// [`RefBitraversable::ref_bi_sequence`] is derived from it via `Clone::clone`.
	///
	/// Note: defining both defaults creates a circular dependency and will not terminate.
	///
	/// ### Laws
	///
	/// `RefBitraversable` instances must be consistent with `RefBifunctor` and `RefBifoldable`:
	/// * Traverse/sequence consistency: `ref_bi_traverse(f, g, x) = ref_bi_sequence(ref_bimap(f, g, x))`.
	#[document_examples]
	///
	/// RefBitraversable laws for [`Result`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // ResultBrand has Of<E, A> = Result<A, E>, so the first function handles errors
	/// // and the second function handles ok values.
	/// let f = |e: &String| if e.is_empty() { None } else { Some(e.len()) };
	/// let g = |a: &i32| if *a > 0 { Some(a * 2) } else { None };
	///
	/// // Traverse/sequence consistency (Ok case):
	/// // bi_traverse((f, g), &x) = ref_bi_sequence(&bimap((f, g), &x))
	/// let ok: Result<i32, String> = Ok(5);
	/// assert_eq!(
	/// 	bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>((&f, &g), &ok),
	/// 	ref_bi_sequence::<ResultBrand, RcFnBrand, _, _, OptionBrand>(&bimap::<
	/// 		ResultBrand,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 	>((&f, &g), &ok),),
	/// );
	///
	/// // Traverse/sequence consistency (Err case):
	/// let err: Result<i32, String> = Err("hello".to_string());
	/// assert_eq!(
	/// 	bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>((&f, &g), &err),
	/// 	ref_bi_sequence::<ResultBrand, RcFnBrand, _, _, OptionBrand>(&bimap::<
	/// 		ResultBrand,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 		_,
	/// 	>((&f, &g), &err),),
	/// );
	/// ```
	#[kind(type Of<'a, A: 'a, B: 'a>: 'a;)]
	pub trait RefBitraversable: RefBifunctor + RefBifoldable {
		/// Traverses the bitraversable structure by reference with two effectful functions.
		///
		/// This method applies `f` to references of first-position values and `g` to references
		/// of second-position values, accumulating all effects in the applicative context `F`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the first-position elements.",
			"The type of the second-position elements.",
			"The output type for first-position elements.",
			"The output type for second-position elements.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function for first-position element references.",
			"The function for second-position element references.",
			"The bitraversable structure to traverse by reference."
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
		/// let y = bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
		/// 	(|e: &i32| Some(e + 1), |s: &i32| Some(s * 2)),
		/// 	&x,
		/// );
		/// assert_eq!(y, Some(Err(4)));
		/// ```
		fn ref_bi_traverse<
			'a,
			FnBrand,
			A: 'a + Clone,
			B: 'a + Clone,
			C: 'a + Clone,
			D: 'a + Clone,
			F: Applicative,
		>(
			f: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
			g: impl Fn(&B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>): Clone;

		/// Sequences a bitraversable structure containing applicative values by reference.
		///
		/// Collapses a structure of effectful values into a single effectful structure,
		/// cloning each element via [`Clone::clone`] through [`RefBitraversable::ref_bi_traverse`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
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
		/// let y = ref_bi_sequence::<ResultBrand, RcFnBrand, _, _, OptionBrand>(&x);
		/// assert_eq!(y, Some(Ok(5)));
		/// ```
		fn ref_bi_sequence<'a, FnBrand, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			ta: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>), Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>)>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			Self::ref_bi_traverse::<
				FnBrand,
				Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
				Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
				A,
				B,
				F,
			>(Clone::clone, Clone::clone, ta)
		}
	}

	/// Traverses a bitraversable structure by reference with two effectful functions.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefBitraversable::ref_bi_traverse`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The brand of the cloneable function wrapper.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The output type for first-position elements.",
		"The output type for second-position elements.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The function for first-position element references.",
		"The function for second-position element references.",
		"The bitraversable structure to traverse by reference."
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
	/// let y = bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
	/// 	(|e: &i32| Some(e + 1), |s: &i32| Some(s * 2)),
	/// 	&x,
	/// );
	/// assert_eq!(y, Some(Ok(10)));
	/// ```
	pub fn ref_bi_traverse<
		'a,
		Brand: RefBitraversable,
		FnBrand,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a + Clone,
		D: 'a + Clone,
		F: Applicative,
	>(
		f: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		g: impl Fn(&B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
		p: &Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
	where
		FnBrand: LiftFn + 'a,
		Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>): Clone, {
		Brand::ref_bi_traverse::<FnBrand, A, B, C, D, F>(f, g, p)
	}

	/// Sequences a bitraversable structure containing applicative values by reference.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefBitraversable::ref_bi_sequence`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The brand of the cloneable function wrapper.",
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
	/// let y = ref_bi_sequence::<ResultBrand, RcFnBrand, _, _, OptionBrand>(&x);
	/// assert_eq!(y, Some(Ok(5)));
	/// ```
	pub fn ref_bi_sequence<
		'a,
		Brand: RefBitraversable,
		FnBrand,
		A: 'a + Clone,
		B: 'a + Clone,
		F: Applicative,
	>(
		ta: &Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>), Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>)>)
	where
		FnBrand: LiftFn + 'a,
		Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
		Brand::ref_bi_sequence::<FnBrand, A, B, F>(ta)
	}

	/// Traverses only the first-position elements by reference, lifting second-position elements via `pure`.
	///
	/// Equivalent to `ref_bi_traverse(f, |b| F::pure(b.clone()), p)`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The brand of the cloneable function wrapper.",
		"The type of the first-position elements.",
		"The type of the second-position elements (unchanged).",
		"The output type for first-position elements.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The function for first-position element references.",
		"The bitraversable structure to traverse by reference."
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
	/// let y = ref_bi_traverse_left::<ResultBrand, RcFnBrand, _, _, _, OptionBrand>(
	/// 	|e: &i32| Some(e + 1),
	/// 	&x,
	/// );
	/// assert_eq!(y, Some(Err(4)));
	/// ```
	pub fn ref_bi_traverse_left<
		'a,
		Brand: RefBitraversable,
		FnBrand,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a + Clone,
		F: Applicative,
	>(
		f: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		p: &Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, B>)>)
	where
		FnBrand: LiftFn + 'a,
		Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, B>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
		Brand::ref_bi_traverse::<FnBrand, A, B, C, B, F>(f, |b: &B| F::pure(b.clone()), p)
	}

	/// Traverses only the second-position elements by reference, lifting first-position elements via `pure`.
	///
	/// Equivalent to `ref_bi_traverse(|a| F::pure(a.clone()), g, p)`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The brand of the cloneable function wrapper.",
		"The type of the first-position elements (unchanged).",
		"The type of the second-position elements.",
		"The output type for second-position elements.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The function for second-position element references.",
		"The bitraversable structure to traverse by reference."
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
	/// let y = ref_bi_traverse_right::<ResultBrand, RcFnBrand, _, _, _, OptionBrand>(
	/// 	|s: &i32| Some(s * 2),
	/// 	&x,
	/// );
	/// assert_eq!(y, Some(Ok(10)));
	/// ```
	pub fn ref_bi_traverse_right<
		'a,
		Brand: RefBitraversable,
		FnBrand,
		A: 'a + Clone,
		B: 'a + Clone,
		D: 'a + Clone,
		F: Applicative,
	>(
		g: impl Fn(&B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
		p: &Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, D>)>)
	where
		FnBrand: LiftFn + 'a,
		Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, D>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>): Clone, {
		Brand::ref_bi_traverse::<FnBrand, A, B, A, D, F>(|a: &A| F::pure(a.clone()), g, p)
	}

	/// Traverses the bitraversable structure by reference with arguments flipped.
	///
	/// Equivalent to `ref_bi_traverse(f, g, p)` with the structure argument first.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The brand of the cloneable function wrapper.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The output type for first-position elements.",
		"The output type for second-position elements.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The bitraversable structure to traverse by reference.",
		"The function for first-position element references.",
		"The function for second-position element references."
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
	/// let y = ref_bi_for::<ResultBrand, RcFnBrand, _, _, _, _, OptionBrand>(
	/// 	&x,
	/// 	|e: &i32| Some(e + 1),
	/// 	|s: &i32| Some(s * 2),
	/// );
	/// assert_eq!(y, Some(Ok(10)));
	/// ```
	pub fn ref_bi_for<
		'a,
		Brand: RefBitraversable,
		FnBrand,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a + Clone,
		D: 'a + Clone,
		F: Applicative,
	>(
		p: &Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		f: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		g: impl Fn(&B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
	where
		FnBrand: LiftFn + 'a,
		Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>): Clone, {
		Brand::ref_bi_traverse::<FnBrand, A, B, C, D, F>(f, g, p)
	}

	/// Traverses only the first-position elements by reference with arguments flipped.
	///
	/// Equivalent to `ref_bi_traverse_left(f, p)` with the structure argument first.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The brand of the cloneable function wrapper.",
		"The type of the first-position elements.",
		"The type of the second-position elements (unchanged).",
		"The output type for first-position elements.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The bitraversable structure to traverse by reference.",
		"The function for first-position element references."
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
	/// let y =
	/// 	ref_bi_for_left::<ResultBrand, RcFnBrand, _, _, _, OptionBrand>(&x, |e: &i32| Some(e + 1));
	/// assert_eq!(y, Some(Err(4)));
	/// ```
	pub fn ref_bi_for_left<
		'a,
		Brand: RefBitraversable,
		FnBrand,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a + Clone,
		F: Applicative,
	>(
		p: &Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		f: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, B>)>)
	where
		FnBrand: LiftFn + 'a,
		Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, B>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
		Brand::ref_bi_traverse::<FnBrand, A, B, C, B, F>(f, |b: &B| F::pure(b.clone()), p)
	}

	/// Traverses only the second-position elements by reference with arguments flipped.
	///
	/// Equivalent to `ref_bi_traverse_right(g, p)` with the structure argument first.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bitraversable structure.",
		"The brand of the cloneable function wrapper.",
		"The type of the first-position elements (unchanged).",
		"The type of the second-position elements.",
		"The output type for second-position elements.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The bitraversable structure to traverse by reference.",
		"The function for second-position element references."
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
	/// let y =
	/// 	ref_bi_for_right::<ResultBrand, RcFnBrand, _, _, _, OptionBrand>(&x, |s: &i32| Some(s * 2));
	/// assert_eq!(y, Some(Ok(10)));
	/// ```
	pub fn ref_bi_for_right<
		'a,
		Brand: RefBitraversable,
		FnBrand,
		A: 'a + Clone,
		B: 'a + Clone,
		D: 'a + Clone,
		F: Applicative,
	>(
		p: &Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		g: impl Fn(&B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, D>)>)
	where
		FnBrand: LiftFn + 'a,
		Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, D>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>): Clone, {
		Brand::ref_bi_traverse::<FnBrand, A, B, A, D, F>(|a: &A| F::pure(a.clone()), g, p)
	}
}

pub use inner::*;
