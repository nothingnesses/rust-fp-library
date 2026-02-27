//! Review optics for constructing values.
//!
//! A review represents a way to construct a structure from a focus value.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{
				CloneableFn,
				UnsizedCoercible,
				optics::*,
			},
			kinds::*,
			types::optics::{
				Tagged,
				TaggedBrand,
			},
		},
		fp_macros::{
			document_parameters,
			document_type_parameters,
			document_return,
		},
		std::marker::PhantomData,
	};

	/// A polymorphic review.
	///
	/// Matches PureScript's `Review s t a b`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus."
	)]
	pub struct Review<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		/// Function to construct a structure from a focus value.
		pub review_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, B, T>),
		pub(crate) _phantom: PhantomData<&'a (S, A)>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus."
	)]
	#[document_parameters("The review instance.")]
	impl<'a, P, S, T, A, B> Clone for Review<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Review,
		/// };
		///
		/// let r: Review<RcBrand, Option<i32>, Option<i32>, i32, i32> = Review::new(Some);
		/// let cloned = r.clone();
		/// ```
		fn clone(&self) -> Self {
			Review {
				review_fn: self.review_fn.clone(),
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
	#[document_parameters("The review instance.")]
	impl<'a, P, S, T, A, B> Review<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		/// Create a new polymorphic review from a review function.
		#[document_signature]
		///
		#[document_parameters("The review function.")]
		///
		#[document_return("A new instance of the type.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Review,
		/// };
		///
		/// let r: Review<RcBrand, Option<i32>, Option<i32>, i32, i32> = Review::new(Some);
		/// ```
		pub fn new(review: impl 'a + Fn(B) -> T) -> Self {
			Review {
				review_fn: <FnBrand<P> as CloneableFn>::new(review),
				_phantom: PhantomData,
			}
		}

		/// Review a focus value into a structure.
		#[document_signature]
		///
		#[document_parameters("The focus value to review.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Review,
		/// };
		///
		/// let r: Review<RcBrand, Option<i32>, Option<i32>, i32, i32> = Review::new(Some);
		/// assert_eq!(r.review(42), Some(42));
		/// ```
		pub fn review(
			&self,
			b: B,
		) -> T {
			(self.review_fn)(b)
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
	#[document_parameters("The review instance.")]
	impl<'a, P, S, T, A, B> Optic<'a, TaggedBrand, S, T, A, B> for Review<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let r: Review<RcBrand, Option<i32>, Option<i32>, i32, i32> = Review::new(Some);
		/// let f = Tagged::new(42);
		/// let reviewed = Optic::<TaggedBrand, _, _, _, _>::evaluate(&r, f);
		/// assert_eq!(reviewed.0, Some(42));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<TaggedBrand as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<TaggedBrand as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, T>) {
			let review = self.review_fn.clone();
			Tagged::new(review(pab.0))
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
	#[document_parameters("The review instance.")]
	impl<'a, P, S, T, A, B> ReviewOptic<'a, S, T, A, B> for Review<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let r: Review<RcBrand, Option<i32>, Option<i32>, i32, i32> = Review::new(Some);
		/// let f = Tagged::new(42);
		/// let reviewed = ReviewOptic::evaluate(&r, f);
		/// assert_eq!(reviewed.0, Some(42));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<TaggedBrand as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<TaggedBrand as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<TaggedBrand, S, T, A, B>::evaluate(self, pab)
		}
	}

	/// A concrete review type where types do not change.
	///
	/// Matches PureScript's `Review' s a`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	pub struct ReviewPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a, {
		/// Function to construct a structure from a focus value.
		pub review_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, A, S>),
		pub(crate) _phantom: PhantomData<P>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The review instance.")]
	impl<'a, P, S, A> Clone for ReviewPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::ReviewPrime,
		/// };
		///
		/// let r: ReviewPrime<RcBrand, Option<i32>, i32> = ReviewPrime::new(Some);
		/// let cloned = r.clone();
		/// ```
		fn clone(&self) -> Self {
			ReviewPrime {
				review_fn: self.review_fn.clone(),
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
	#[document_parameters("The review instance.")]
	impl<'a, P, S, A> ReviewPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		/// Create a new monomorphic review.
		#[document_signature]
		///
		#[document_parameters("The review function.")]
		///
		#[document_return("A new instance of the type.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::ReviewPrime,
		/// };
		///
		/// let r: ReviewPrime<RcBrand, Option<i32>, i32> = ReviewPrime::new(Some);
		/// ```
		pub fn new(review: impl 'a + Fn(A) -> S) -> Self {
			ReviewPrime {
				review_fn: <FnBrand<P> as CloneableFn>::new(review),
				_phantom: PhantomData,
			}
		}

		/// Review a focus value into a structure.
		#[document_signature]
		///
		#[document_parameters("The focus value to review.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::ReviewPrime,
		/// };
		///
		/// let r: ReviewPrime<RcBrand, Option<i32>, i32> = ReviewPrime::new(Some);
		/// assert_eq!(r.review(42), Some(42));
		/// ```
		pub fn review(
			&self,
			a: A,
		) -> S {
			(self.review_fn)(a)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The review instance.")]
	impl<'a, P, S, A> Optic<'a, TaggedBrand, S, S, A, A> for ReviewPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let r: ReviewPrime<RcBrand, Option<i32>, i32> = ReviewPrime::new(Some);
		/// let f = Tagged::new(42);
		/// let reviewed = Optic::<TaggedBrand, _, _, _, _>::evaluate(&r, f);
		/// assert_eq!(reviewed.0, Some(42));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<TaggedBrand as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<TaggedBrand as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>) {
			let review = self.review_fn.clone();
			Tagged::new(review(pab.0))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The review instance.")]
	impl<'a, P, S, A> ReviewOptic<'a, S, S, A, A> for ReviewPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let r: ReviewPrime<RcBrand, Option<i32>, i32> = ReviewPrime::new(Some);
		/// let f = Tagged::new(42);
		/// let reviewed = ReviewOptic::evaluate(&r, f);
		/// assert_eq!(reviewed.0, Some(42));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<TaggedBrand as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<TaggedBrand as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>) {
			Optic::<TaggedBrand, S, S, A, A>::evaluate(self, pab)
		}
	}
}
pub use inner::*;
