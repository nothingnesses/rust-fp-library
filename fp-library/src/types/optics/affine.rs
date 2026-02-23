//! Affine traversal optics for optional focusing.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{
				Choice,
				CloneableFn,
				Strong,
				UnsizedCoercible,
				monoid::Monoid,
				wander::Wander,
			},
			kinds::*,
			types::optics::{
				FoldOptic,
				ForgetBrand,
				Optic,
				SetterOptic,
				TraversalOptic,
			},
		},
		fp_macros::{
			document_parameters,
			document_type_parameters,
		},
		std::marker::PhantomData,
	};

	/// A polymorphic affine traversal where types can change.
	/// This matches PureScript's `AffineTraversal s t a b`.
	///
	/// An affine traversal focuses on at most one element. It combines the properties of
	/// lenses (Strong) and prisms (Choice), allowing optional focusing like a prism but
	/// without the ability to construct the whole from the part.
	/// Uses [`FnBrand`](crate::brands::FnBrand) to support capturing closures.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	pub struct AffineTraversal<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		/// Preview function: tries to extract the focus from the structure, returning T on failure.
		pub preview: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, Result<A, T>>),
		/// Set function: updates the structure with a new focus value.
		pub set: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, (S, B), T>),
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
	#[document_parameters("The affine traversal instance.")]
	impl<'a, P, S: 'a, T: 'a, A: 'a, B: 'a> AffineTraversal<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		/// Create a new polymorphic affine traversal.
		#[document_signature]
		///
		#[document_parameters("The preview function.", "The set function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::AffineTraversal,
		/// };
		///
		/// let at: AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	AffineTraversal::new(|(x, s)| Ok(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.preview((42, "hi".to_string())), Ok(42));
		/// ```
		pub fn new(
			preview: impl 'a + Fn(S) -> Result<A, T>,
			set: impl 'a + Fn((S, B)) -> T,
		) -> Self {
			AffineTraversal {
				preview: <FnBrand<P> as CloneableFn>::new(preview),
				set: <FnBrand<P> as CloneableFn>::new(set),
				_phantom: PhantomData,
			}
		}

		/// Preview the focus of the affine traversal in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to preview.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::AffineTraversal,
		/// };
		///
		/// let at: AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	AffineTraversal::new(|(x, s)| Ok(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.preview((42, "hi".to_string())), Ok(42));
		/// ```
		pub fn preview(
			&self,
			s: S,
		) -> Result<A, T> {
			(self.preview)(s)
		}

		/// Set the focus in the structure.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The new focus value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::AffineTraversal,
		/// };
		///
		/// let at: AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	AffineTraversal::new(|(x, s)| Ok(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.set((42, "hi".to_string()), 99), (99, "hi".to_string()));
		/// ```
		pub fn set(
			&self,
			s: S,
			b: B,
		) -> T {
			(self.set)((s, b))
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
	#[document_parameters("The affine traversal instance.")]
	impl<'a, Q, P, S, T, A, B> Optic<'a, Q, S, T, A, B> for AffineTraversal<'a, P, S, T, A, B>
	where
		Q: Strong + Choice,
		P: UnsizedCoercible,
		S: 'a + Clone,
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
		/// let at: AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	AffineTraversal::new(|(x, s)| Ok(x), |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> as Optic<
		/// 	RcFnBrand,
		/// 	_,
		/// 	_,
		/// 	_,
		/// 	_,
		/// >>::evaluate(&at, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let preview = self.preview.clone();
			let set = self.set.clone();

			// affine preview set = dimap split merge . right . first
			// Q::first(pab) is p (A, S) (B, S)
			// Q::right(...) is p (Result<(A, S), T>) (Result<(B, S), T>)
			Q::dimap(
				move |s: S| match preview(s.clone()) {
					Ok(a) => Ok((a, s)),
					Err(t) => Err(t),
				},
				move |result: Result<(B, S), T>| match result {
					Ok((b, s)) => set((s, b)),
					Err(t) => t,
				},
				Q::right(Q::first(pab)),
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
	#[document_parameters("The affine traversal instance.")]
	impl<'a, P, S: 'a + Clone, T: 'a, A: 'a, B: 'a> TraversalOptic<'a, S, T, A, B>
		for AffineTraversal<'a, P, S, T, A, B>
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
		/// let at: AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	AffineTraversal::new(|(x, s)| Ok(x), |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier =
		/// 	<AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> as TraversalOptic<
		/// 		(i32, String),
		/// 		(i32, String),
		/// 		i32,
		/// 		i32,
		/// 	>>::evaluate::<RcFnBrand>(&at, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate<Q: Wander>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<Q, S, T, A, B>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The affine traversal instance.")]
	impl<'a, P, S: 'a + Clone, A: 'a> FoldOptic<'a, S, A> for AffineTraversal<'a, P, S, S, A, A>
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
		/// let at: AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	AffineTraversal::new(|(x, s)| Ok(x), |((_, s), x)| (x, s));
		///
		/// let f = Forget::<RcBrand, String, i32, i32>::new(|x| x.to_string());
		/// let folded = <AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> as FoldOptic<
		/// 	(i32, String),
		/// 	i32,
		/// >>::evaluate::<String, RcBrand>(&at, f);
		/// assert_eq!(folded.run((42, "hello".to_string())), "42".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			Optic::<ForgetBrand<Q, R>, S, S, A, A>::evaluate(self, pab)
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
	#[document_parameters("The affine traversal instance.")]
	impl<'a, Q, P, S: 'a + Clone, T: 'a, A: 'a, B: 'a> SetterOptic<'a, Q, S, T, A, B>
		for AffineTraversal<'a, P, S, T, A, B>
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
		/// let at: AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	AffineTraversal::new(|(x, s)| Ok(x), |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier =
		/// 	<AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> as SetterOptic<
		/// 		RcBrand,
		/// 		_,
		/// 		_,
		/// 		_,
		/// 		_,
		/// 	>>::evaluate(&at, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<FnBrand<Q>, S, T, A, B>::evaluate(self, pab)
		}
	}

	/// A concrete affine traversal type where types do not change.
	/// This matches PureScript's `AffineTraversal' s a`.
	///
	/// Uses [`FnBrand`](crate::brands::FnBrand) to support capturing closures.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	pub struct AffineTraversalPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a, {
		pub(crate) preview_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, Option<A>>),
		pub(crate) set_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, (S, A), S>),
		pub(crate) _phantom: PhantomData<P>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The affine traversal instance.")]
	impl<'a, P, S, A> Clone for AffineTraversalPrime<'a, P, S, A>
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
		/// 	types::optics::AffineTraversalPrime,
		/// };
		///
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::new(|(x, _)| Some(x), |((_, s), x)| (x, s));
		/// let cloned = at.clone();
		/// assert_eq!(cloned.preview((42, "hi".to_string())), Some(42));
		/// ```
		fn clone(&self) -> Self {
			AffineTraversalPrime {
				preview_fn: self.preview_fn.clone(),
				set_fn: self.set_fn.clone(),
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
	#[document_parameters("The monomorphic affine traversal instance.")]
	impl<'a, P, S: 'a, A: 'a> AffineTraversalPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		/// Create a new monomorphic affine traversal from preview and set functions.
		#[document_signature]
		///
		#[document_parameters("The preview function.", "The set function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::AffineTraversalPrime,
		/// };
		///
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::new(|(x, _)| Some(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.preview((42, "hi".to_string())), Some(42));
		/// ```
		pub fn new(
			preview: impl 'a + Fn(S) -> Option<A>,
			set: impl 'a + Fn((S, A)) -> S,
		) -> Self {
			AffineTraversalPrime {
				preview_fn: <FnBrand<P> as CloneableFn>::new(preview),
				set_fn: <FnBrand<P> as CloneableFn>::new(set),
				_phantom: PhantomData,
			}
		}

		/// Preview the focus of the affine traversal in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to preview.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::AffineTraversalPrime,
		/// };
		///
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::new(|(x, _)| Some(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.preview((42, "hi".to_string())), Some(42));
		/// ```
		pub fn preview(
			&self,
			s: S,
		) -> Option<A> {
			(self.preview_fn)(s)
		}

		/// Set the focus in the structure.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The new focus value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::AffineTraversalPrime,
		/// };
		///
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::new(|(x, _)| Some(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.set((42, "hi".to_string()), 99), (99, "hi".to_string()));
		/// ```
		pub fn set(
			&self,
			s: S,
			a: A,
		) -> S {
			(self.set_fn)((s, a))
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
		/// 	types::optics::AffineTraversalPrime,
		/// };
		///
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::new(|(x, _)| Some(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.modify((21, "hi".to_string()), |x| x * 2), (42, "hi".to_string()));
		/// ```
		pub fn modify(
			&self,
			s: S,
			f: impl Fn(A) -> A,
		) -> S
		where
			S: Clone, {
			match self.preview(s.clone()) {
				Some(a) => self.set(s, f(a)),
				None => s,
			}
		}
	}

	// Optic implementation for AffineTraversalPrime<P, S, A>
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic affine traversal instance.")]
	impl<'a, Q, P, S, A> Optic<'a, Q, S, S, A, A> for AffineTraversalPrime<'a, P, S, A>
	where
		Q: Strong + Choice,
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
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::new(|(x, _)| Some(x), |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <AffineTraversalPrime<RcBrand, (i32, String), i32> as Optic<
		/// 	RcFnBrand,
		/// 	_,
		/// 	_,
		/// 	_,
		/// 	_,
		/// >>::evaluate(&at, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			let preview_fn = self.preview_fn.clone();
			let set_fn = self.set_fn.clone();

			Q::dimap(
				move |s: S| match preview_fn(s.clone()) {
					Some(a) => Ok((a, s)),
					None => Err(s),
				},
				move |result: Result<(A, S), S>| match result {
					Ok((a, s)) => set_fn((s, a)),
					Err(s) => s,
				},
				Q::right(Q::first(pab)),
			)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic affine traversal instance.")]
	impl<'a, P, S: 'a + Clone, A: 'a> TraversalOptic<'a, S, S, A, A>
		for AffineTraversalPrime<'a, P, S, A>
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
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::new(|(x, _)| Some(x), |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <AffineTraversalPrime<RcBrand, (i32, String), i32> as TraversalOptic<
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// >>::evaluate::<RcFnBrand>(&at, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate<Q: Wander>(
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
	#[document_parameters("The monomorphic affine traversal instance.")]
	impl<'a, P, S: 'a + Clone, A: 'a> FoldOptic<'a, S, A> for AffineTraversalPrime<'a, P, S, A>
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
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::new(|(x, _)| Some(x), |((_, s), x)| (x, s));
		///
		/// let f = Forget::<RcBrand, String, i32, i32>::new(|x| x.to_string());
		/// let folded = <AffineTraversalPrime<RcBrand, (i32, String), i32> as FoldOptic<
		/// 	(i32, String),
		/// 	i32,
		/// >>::evaluate::<String, RcBrand>(&at, f);
		/// assert_eq!(folded.run((42, "hello".to_string())), "42".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			Optic::<ForgetBrand<Q, R>, S, S, A, A>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic affine traversal instance.")]
	impl<'a, Q, P, S: 'a + Clone, A: 'a> SetterOptic<'a, Q, S, S, A, A>
		for AffineTraversalPrime<'a, P, S, A>
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
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::new(|(x, _)| Some(x), |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <AffineTraversalPrime<RcBrand, (i32, String), i32> as SetterOptic<
		/// 	RcBrand,
		/// 	_,
		/// 	_,
		/// 	_,
		/// 	_,
		/// >>::evaluate(&at, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			Optic::<FnBrand<Q>, S, S, A, A>::evaluate(self, pab)
		}
	}
}
pub use inner::*;
