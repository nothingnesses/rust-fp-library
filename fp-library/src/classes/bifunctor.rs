//! Types that can be mapped over two type arguments simultaneously.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Result::<i32, i32>::Ok(5);
//! let y = bimap::<ResultBrand, _, _, _, _>(|e| e + 1, |s| s * 2, x);
//! assert_eq!(y, Ok(10));
//! ```
#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::*,
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for types that can be mapped over two type arguments.
	///
	/// A `Bifunctor` represents a context or container with two type parameters,
	/// allowing functions to be applied to values of both types.
	///
	/// ### Hierarchy Unification
	///
	/// This trait inherits from [`Kind_266801a817966495`], ensuring that all bifunctor
	/// contexts satisfy the strict lifetime requirements where both type arguments must
	/// outlive the context's application lifetime.
	///
	/// By explicitly requiring that both type parameters outlive the application lifetime `'a`,
	/// we provide the compiler with the necessary guarantees to handle trait objects
	/// (like `dyn Fn`) commonly used in bifunctor implementations. This resolves potential
	/// E0310 errors where the compiler cannot otherwise prove that captured variables in
	/// closures satisfy the required lifetime bounds.
	///
	/// ### Laws
	///
	/// `Bifunctor` instances must satisfy the following laws:
	/// * Identity: `bimap(identity, identity, p) = p`.
	/// * Composition: `bimap(compose(f, g), compose(h, i), p) = bimap(f, h, bimap(g, i, p))`.
	#[document_examples]
	///
	/// Bifunctor laws for [`Result`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let ok: Result<i32, i32> = Ok(5);
	/// let err: Result<i32, i32> = Err(3);
	///
	/// // Identity: bimap(identity, identity, p) = p
	/// assert_eq!(bimap::<ResultBrand, _, _, _, _>(identity, identity, ok), ok);
	/// assert_eq!(bimap::<ResultBrand, _, _, _, _>(identity, identity, err), err);
	///
	/// // Composition: bimap(compose(f, g), compose(h, i), p)
	/// //            = bimap(f, h, bimap(g, i, p))
	/// let f = |x: i32| x + 1;
	/// let g = |x: i32| x * 2;
	/// let h = |x: i32| x + 10;
	/// let i = |x: i32| x * 3;
	/// assert_eq!(
	/// 	bimap::<ResultBrand, _, _, _, _>(compose(f, g), compose(h, i), ok),
	/// 	bimap::<ResultBrand, _, _, _, _>(f, h, bimap::<ResultBrand, _, _, _, _>(g, i, ok)),
	/// );
	/// assert_eq!(
	/// 	bimap::<ResultBrand, _, _, _, _>(compose(f, g), compose(h, i), err),
	/// 	bimap::<ResultBrand, _, _, _, _>(f, h, bimap::<ResultBrand, _, _, _, _>(g, i, err)),
	/// );
	/// ```
	pub trait Bifunctor: Kind_266801a817966495 {
		/// Maps functions over the values in the bifunctor context.
		///
		/// This method applies two functions to the values inside the bifunctor context, producing a new bifunctor context with the transformed values.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the first result.",
			"The type of the second value.",
			"The type of the second result."
		)]
		///
		#[document_parameters(
			"The function to apply to the first value.",
			"The function to apply to the second value.",
			"The bifunctor instance."
		)]
		///
		#[document_returns(
			"A new bifunctor instance containing the results of applying the functions."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Result::<i32, i32>::Ok(5);
		/// let y = bimap::<ResultBrand, _, _, _, _>(|e| e + 1, |s| s * 2, x);
		/// assert_eq!(y, Ok(10));
		/// ```
		fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			f: impl Fn(A) -> B + 'a,
			g: impl Fn(C) -> D + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>);
	}

	/// Maps functions over the values in the bifunctor context.
	///
	/// Free function version that dispatches to [the type class' associated function][`Bifunctor::bimap`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bifunctor.",
		"The type of the first value.",
		"The type of the first result.",
		"The type of the second value.",
		"The type of the second result."
	)]
	///
	#[document_parameters(
		"The function to apply to the first value.",
		"The function to apply to the second value.",
		"The bifunctor instance."
	)]
	///
	#[document_returns(
		"A new bifunctor instance containing the results of applying the functions."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Result::<i32, i32>::Ok(5);
	/// let y = bimap::<ResultBrand, _, _, _, _>(|e| e + 1, |s| s * 2, x);
	/// assert_eq!(y, Ok(10));
	/// ```
	pub fn bimap<'a, Brand: Bifunctor, A: 'a, B: 'a, C: 'a, D: 'a>(
		f: impl Fn(A) -> B + 'a,
		g: impl Fn(C) -> D + 'a,
		p: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
	) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
		Brand::bimap(f, g, p)
	}

	impl_kind! {
		impl<Brand: Bifunctor, A: 'static> for BifunctorFirstAppliedBrand<Brand, A> {
			type Of<'a, B: 'a>: 'a = Apply!(<Brand as Kind!(type Of<'a, T: 'a, U: 'a>: 'a;)>::Of<'a, A, B>);
		}
	}

	/// [`Functor`] instance for [`BifunctorFirstAppliedBrand`].
	///
	/// Maps over the first type parameter of a bifunctor by delegating to [`Bifunctor::bimap`]
	/// with [`identity`](crate::functions::identity) for the second argument.
	#[document_type_parameters("The bifunctor brand.", "The fixed second type parameter.")]
	impl<Brand: Bifunctor, A: 'static> Functor for BifunctorFirstAppliedBrand<Brand, A> {
		/// Map a function over the first type parameter.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters("The function to apply.", "The bifunctor value to map over.")]
		#[document_returns("The mapped bifunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Result::<i32, i32>::Ok(5);
		/// let y = map::<BifunctorFirstAppliedBrand<ResultBrand, i32>, _, _>(|s| s * 2, x);
		/// assert_eq!(y, Ok(10));
		/// ```
		fn map<'a, B: 'a, C: 'a>(
			f: impl Fn(B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::bimap(crate::functions::identity, f, fa)
		}
	}

	impl_kind! {
		impl<Brand: Bifunctor, B: 'static> for BifunctorSecondAppliedBrand<Brand, B> {
			type Of<'a, A: 'a>: 'a = Apply!(<Brand as Kind!(type Of<'a, T: 'a, U: 'a>: 'a;)>::Of<'a, A, B>);
		}
	}

	/// [`Functor`] instance for [`BifunctorSecondAppliedBrand`].
	///
	/// Maps over the second type parameter of a bifunctor by delegating to [`Bifunctor::bimap`]
	/// with [`identity`](crate::functions::identity) for the first argument.
	#[document_type_parameters("The bifunctor brand.", "The fixed first type parameter.")]
	impl<Brand: Bifunctor, B: 'static> Functor for BifunctorSecondAppliedBrand<Brand, B> {
		/// Map a function over the second type parameter.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters("The function to apply.", "The bifunctor value to map over.")]
		#[document_returns("The mapped bifunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Result::<i32, i32>::Err(5);
		/// let y = map::<BifunctorSecondAppliedBrand<ResultBrand, i32>, _, _>(|e| e * 2, x);
		/// assert_eq!(y, Err(10));
		/// ```
		fn map<'a, A: 'a, C: 'a>(
			f: impl Fn(A) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::bimap(f, crate::functions::identity, fa)
		}
	}
}

pub use inner::*;
