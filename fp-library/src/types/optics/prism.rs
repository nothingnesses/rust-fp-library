//! Prism optics for sum types.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{Choice, CloneableFn, UnsizedCoercible, monoid::Monoid, wander::Wander},
			kinds::*,
			types::optics::{FoldOptic, Optic, PrismOptic, SetterOptic, TraversalOptic, ReviewOptic, TaggedBrand, ForgetBrand, Tagged},
		},
		fp_macros::{document_parameters, document_signature, document_type_parameters},
		std::marker::PhantomData,
	};

	/// A polymorphic prism for sum types where types can change.
	/// This matches PureScript's `Prism s t a b`.
	///
	/// A prism focuses on a value that may not be present (like a particular variant
	/// of an enum). Uses [`FnBrand`](crate::brands::FnBrand) to support capturing closures.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	pub struct Prism<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		/// Preview function: tries to extract the focus, returning the target structure T on failure.
		pub preview: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, Result<A, T>>),
		/// Review function: constructs the structure from a focus value.
		pub review: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, B, T>),
		pub(crate) _phantom: PhantomData<P>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The prism instance.")]
	impl<'a, P, S: 'a, T: 'a, A: 'a, B: 'a> Prism<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		/// Create a new polymorphic prism.
		#[document_signature]
		///
		#[document_parameters("The preview function.", "The review function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Prism,
		/// };
		///
		/// let ok_prism: Prism<RcBrand, Option<i32>, Option<f64>, i32, f64> =
		/// 	Prism::new(|o: Option<i32>| o.ok_or(None), |f| Some(f));
		/// ```
		pub fn new(
			preview: impl 'a + Fn(S) -> Result<A, T>,
			review: impl 'a + Fn(B) -> T,
		) -> Self {
			Prism {
				preview: <FnBrand<P> as CloneableFn>::new(preview),
				review: <FnBrand<P> as CloneableFn>::new(review),
				_phantom: PhantomData,
			}
		}

		/// Preview the focus of the prism in a structure.
		#[document_signature]
		#[document_parameters("The structure to preview.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Prism,
		/// };
		///
		/// let ok_prism: Prism<RcBrand, Option<i32>, Option<f64>, i32, f64> =
		/// 	Prism::new(|o: Option<i32>| o.ok_or(None), |f| Some(f));
		/// assert_eq!(ok_prism.preview(Some(42)), Ok(42));
		/// ```
		pub fn preview(
			&self,
			s: S,
		) -> Result<A, T> {
			(self.preview)(s)
		}

		/// Review the focus into the structure.
		#[document_signature]
		///
		#[document_parameters("The focus value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Prism,
		/// };
		///
		/// let ok_prism: Prism<RcBrand, Option<i32>, Option<f64>, i32, f64> =
		/// 	Prism::new(|o: Option<i32>| o.ok_or(None), |f| Some(f));
		/// assert_eq!(ok_prism.review(42.0), Some(42.0));
		/// ```
		pub fn review(
			&self,
			b: B,
		) -> T {
			(self.review)(b)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The prism instance.")]
	impl<'a, Q, P, S, T, A, B> Optic<'a, Q, S, T, A, B> for Prism<'a, P, S, T, A, B>
	where
		Q: Choice,
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
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let ok_prism: Prism<RcBrand, Option<i32>, Option<i32>, i32, i32> =
		/// 	Prism::new(|o: Option<i32>| o.ok_or(None), |x| Some(x));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = Optic::<RcFnBrand, _, _, _, _>::evaluate(&ok_prism, f);
		/// assert_eq!(modifier(Some(21)), Some(42));
		/// assert_eq!(modifier(None), None);
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let preview = self.preview.clone();
			let review = self.review.clone();

			// prism :: (s -> Either t a) -> (b -> t) -> Prism s t a b
			// PureScript Right is focus, Left is fallback.
			// Rust Choice::right lifts p a b to p (Result<a, c>) (Result<b, c>)
			// Wait, choice.rs: right lifts to Result<A, C> where A is focus.
			// So Q::right(pab) is p (Result<A, T>) (Result<B, T>)
			Q::dimap(
				move |s: S| preview(s),
				move |result: Result<B, T>| match result {
					Ok(b) => review(b),
					Err(t) => t,
				},
				Q::right(pab),
			)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The prism instance.")]
	impl<'a, P, S: 'a, T: 'a, A: 'a, B: 'a> PrismOptic<'a, S, T, A, B> for Prism<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let ok_prism: Prism<RcBrand, Option<i32>, Option<i32>, i32, i32> =
		/// 	Prism::new(|o: Option<i32>| o.ok_or(None), |x| Some(x));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = PrismOptic::evaluate(&ok_prism, f);
		/// assert_eq!(modifier(Some(21)), Some(42));
		/// ```
		fn evaluate<Q: Choice>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<Q, S, T, A, B>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The prism instance.")]
	impl<'a, P, S: 'a, T: 'a, A: 'a, B: 'a> TraversalOptic<'a, S, T, A, B> for Prism<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let ok_prism: Prism<RcBrand, Option<i32>, Option<i32>, i32, i32> =
		/// 	Prism::new(|o: Option<i32>| o.ok_or(None), |x| Some(x));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = TraversalOptic::evaluate(&ok_prism, f);
		/// assert_eq!(modifier(Some(21)), Some(42));
		/// ```
		fn evaluate<Q: Wander>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			PrismOptic::evaluate::<Q>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The focus type."
	)]
	#[document_parameters("The prism instance.")]
	impl<'a, P, S: 'a, A: 'a> FoldOptic<'a, S, A> for Prism<'a, P, S, S, A, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters(
			"The monoid type.",
			"The reference-counted pointer type for the Forget brand."
		)]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let ok_prism: Prism<RcBrand, Option<i32>, Option<i32>, i32, i32> =
		/// 	Prism::new(|o: Option<i32>| o.ok_or(None), |x| Some(x));
		///
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded = FoldOptic::evaluate(&ok_prism, f);
		/// assert_eq!(folded.run(Some(42)), 42);
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			PrismOptic::evaluate::<ForgetBrand<Q, R>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The reference-counted pointer type for the prism.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The prism instance.")]
	impl<'a, Q, P, S: 'a, T: 'a, A: 'a, B: 'a> SetterOptic<'a, Q, S, T, A, B>
		for Prism<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let ok_prism: Prism<RcBrand, Option<i32>, Option<i32>, i32, i32> =
		/// 	Prism::new(|o: Option<i32>| o.ok_or(None), |x| Some(x));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = SetterOptic::evaluate(&ok_prism, f);
		/// assert_eq!(modifier(Some(21)), Some(42));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			PrismOptic::evaluate::<FnBrand<Q>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The prism instance.")]
	impl<'a, P, S: 'a, T: 'a, A: 'a, B: 'a> ReviewOptic<'a, S, T, A, B> for Prism<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let ok_prism: Prism<RcBrand, Option<i32>, Option<i32>, i32, i32> =
		/// 	Prism::new(|o: Option<i32>| o.ok_or(None), |x| Some(x));
		///
		/// let f = Tagged::new(42);
		/// let reviewed = ReviewOptic::evaluate(&ok_prism, f);
		/// assert_eq!(reviewed.0, Some(42));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<TaggedBrand as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<TaggedBrand as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let review = self.review.clone();
			Tagged::new(review(pab.0))
		}
	}

	/// A concrete prism type for sum types where types do not change.
	/// This matches PureScript's `Prism' s a`.
	///
	/// Uses [`FnBrand`](crate::brands::FnBrand) to support capturing closures.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	pub struct PrismPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		pub(crate) preview_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, Option<A>>),
		pub(crate) review_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, A, S>),
		pub(crate) _phantom: PhantomData<P>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The prism instance.")]
	impl<'a, P, S, A> Clone for PrismPrime<'a, P, S, A>
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
		/// 	types::optics::PrismPrime,
		/// };
		///
		/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
		/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
		/// let cloned = ok_prism.clone();
		/// ```
		fn clone(&self) -> Self {
			PrismPrime {
				preview_fn: self.preview_fn.clone(),
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
	#[document_parameters("The monomorphic prism instance.")]
	impl<'a, P, S: 'a, A: 'a> PrismPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		/// Create a new monomorphic prism from preview and review functions.
		#[document_signature]
		///
		#[document_parameters("The preview function.", "The review function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::PrismPrime,
		/// };
		///
		/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
		/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
		/// ```
		pub fn new(
			preview: impl 'a + Fn(S) -> Option<A>,
			review: impl 'a + Fn(A) -> S,
		) -> Self {
			PrismPrime {
				preview_fn: <FnBrand<P> as CloneableFn>::new(preview),
				review_fn: <FnBrand<P> as CloneableFn>::new(review),
				_phantom: PhantomData,
			}
		}

		/// Preview the focus of the prism in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to preview.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::PrismPrime,
		/// };
		///
		/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
		/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
		/// assert_eq!(ok_prism.preview(Ok(42)), Some(42));
		/// assert_eq!(ok_prism.preview(Err("error".to_string())), None);
		/// ```
		pub fn preview(
			&self,
			s: S,
		) -> Option<A> {
			(self.preview_fn)(s)
		}

		/// Review the focus into the structure.
		#[document_signature]
		///
		#[document_parameters("The focus value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::PrismPrime,
		/// };
		///
		/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
		/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
		/// assert_eq!(ok_prism.review(42), Ok(42));
		/// ```
		pub fn review(
			&self,
			a: A,
		) -> S {
			(self.review_fn)(a)
		}

		/// Modify the focus if it exists.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The function to apply to the focus.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::PrismPrime,
		/// };
		///
		/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
		/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
		/// assert_eq!(ok_prism.modify(Ok(21), |x| x * 2), Ok(42));
		/// assert_eq!(ok_prism.modify(Err("error".to_string()), |x| x * 2), Err("error".to_string()));
		/// ```
		pub fn modify(
			&self,
			s: S,
			f: impl Fn(A) -> A,
		) -> S
		where
			S: Clone,
		{
			match self.preview(s.clone()) {
				Some(a) => self.review(f(a)),
				None => s,
			}
		}
	}

	// Optic implementation for PrismPrime<P, S, A>
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic prism instance.")]
	impl<'a, Q, P, S, A> Optic<'a, Q, S, S, A, A> for PrismPrime<'a, P, S, A>
	where
		Q: Choice,
		P: UnsizedCoercible,
		S: 'a + Clone,
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
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
		/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = Optic::<RcFnBrand, _, _, _, _>::evaluate(&ok_prism, f);
		/// assert_eq!(modifier(Ok(21)), Ok(42));
		/// assert_eq!(modifier(Err("error".to_string())), Err("error".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			let preview_fn = self.preview_fn.clone();
			let review_fn = self.review_fn.clone();

			Q::dimap(
				move |s: S| match preview_fn(s.clone()) {
					Some(a) => Ok(a),
					None => Err(s),
				},
				move |result: Result<A, S>| match result {
					Ok(a) => review_fn(a),
					Err(s) => s,
				},
				Q::right(pab),
			)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic prism instance.")]
	impl<'a, P, S: 'a + Clone, A: 'a> PrismOptic<'a, S, S, A, A> for PrismPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
		/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = PrismOptic::evaluate(&ok_prism, f);
		/// assert_eq!(modifier(Ok(21)), Ok(42));
		/// ```
		fn evaluate<Q: Choice>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			Optic::<Q, S, S, A, A>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic prism instance.")]
	impl<'a, P, S: 'a + Clone, A: 'a> TraversalOptic<'a, S, S, A, A> for PrismPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
		/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = TraversalOptic::evaluate(&ok_prism, f);
		/// assert_eq!(modifier(Ok(21)), Ok(42));
		/// ```
		fn evaluate<Q: Wander>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			PrismOptic::evaluate::<Q>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic prism instance.")]
	impl<'a, P, S: 'a + Clone, A: 'a> FoldOptic<'a, S, A> for PrismPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters(
			"The monoid type.",
			"The reference-counted pointer type for the Forget brand."
		)]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
		/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
		///
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded = FoldOptic::evaluate(&ok_prism, f);
		/// assert_eq!(folded.run(Ok(42)), 42);
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			PrismOptic::evaluate::<ForgetBrand<Q, R>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The reference-counted pointer type for the prism.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic prism instance.")]
	impl<'a, Q, P, S: 'a + Clone, A: 'a> SetterOptic<'a, Q, S, S, A, A> for PrismPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
		/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = SetterOptic::evaluate(&ok_prism, f);
		/// assert_eq!(modifier(Ok(21)), Ok(42));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			PrismOptic::evaluate::<FnBrand<Q>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic prism instance.")]
	impl<'a, P, S: 'a + Clone, A: 'a> ReviewOptic<'a, S, S, A, A> for PrismPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
		/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
		///
		/// let f = Tagged::new(42);
		/// let reviewed = ReviewOptic::evaluate(&ok_prism, f);
		/// assert_eq!(reviewed.0, Ok(42));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<TaggedBrand as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<TaggedBrand as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			let review_fn = self.review_fn.clone();
			Tagged::new(review_fn(pab.0))
		}
	}
}
pub use inner::*;
