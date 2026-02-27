//! Affine traversal optics for optional focusing.

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
				profunctor::{
					Choice,
					Strong,
					Wander,
				},
			},
			kinds::*,
			types::optics::ForgetBrand,
		},
		fp_macros::{
			document_parameters,
			document_type_parameters,
		},
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
		/// Internal storage avoiding S: Clone.
		pub(crate) to: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, Result<(A, <FnBrand<P> as CloneableFn>::Of<'a, B, T>), T>>),
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
		/// This matches PureScript's `affineTraversal'` constructor.
		#[document_signature]
		///
		#[document_parameters("The getter/setter pair function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::CloneableFn,
		/// 	types::optics::AffineTraversal,
		/// };
		///
		/// let at: AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	AffineTraversal::new(|(x, s): (i32, String)| {
		/// 		Ok((x, <RcFnBrand as CloneableFn>::new(move |b| (b, s.clone()))))
		/// 	});
		/// ```
		pub fn new(
			to: impl 'a + Fn(S) -> Result<(A, <FnBrand<P> as CloneableFn>::Of<'a, B, T>), T>
		) -> Self {
			AffineTraversal {
				to: <FnBrand<P> as CloneableFn>::new(to),
			}
		}

		/// Create a new polymorphic affine traversal from preview and set functions.
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
		/// 	AffineTraversal::from_preview_set(|(x, s): (i32, String)| Ok(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.preview((42, "hi".to_string())), Ok(42));
		/// ```
		pub fn from_preview_set(
			preview: impl 'a + Fn(S) -> Result<A, T>,
			set: impl 'a + Fn((S, B)) -> T,
		) -> Self
		where
			S: Clone, {
			let preview_brand = <FnBrand<P> as CloneableFn>::new(preview);
			let set_brand = <FnBrand<P> as CloneableFn>::new(set);

			AffineTraversal {
				to: <FnBrand<P> as CloneableFn>::new(move |s: S| {
					let s_clone = s.clone();
					let set_brand = set_brand.clone();
					match preview_brand(s) {
						Ok(a) => Ok((
							a,
							<FnBrand<P> as CloneableFn>::new(move |b| {
								set_brand((s_clone.clone(), b))
							}),
						)),
						Err(t) => Err(t),
					}
				}),
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
		/// 	AffineTraversal::from_preview_set(|(x, s): (i32, String)| Ok(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.preview((42, "hi".to_string())), Ok(42));
		/// ```
		pub fn preview(
			&self,
			s: S,
		) -> Result<A, T> {
			(self.to)(s).map(|(a, _)| a)
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
		/// 	AffineTraversal::from_preview_set(|(x, _)| Ok(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.set((42, "hi".to_string()), 99), (99, "hi".to_string()));
		/// ```
		pub fn set(
			&self,
			s: S,
			b: B,
		) -> T {
			match (self.to)(s) {
				Ok((_, f)) => f(b),
				Err(t) => t,
			}
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
		/// let at: AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	AffineTraversal::from_preview_set(|(x, s): (i32, String)| Ok(x), |((_, s), x)| (x, s));
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
			let to = self.to.clone();

			Q::dimap(
				move |s: S| to(s),
				move |result: Result<(B, <FnBrand<P> as CloneableFn>::Of<'a, B, T>), T>| {
					match result {
						Ok((b, f)) => f(b),
						Err(t) => t,
					}
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
	impl<'a, P, S, T, A, B> AffineTraversalOptic<'a, S, T, A, B> for AffineTraversal<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
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
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let at: AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	AffineTraversal::from_preview_set(|(x, s): (i32, String)| Ok(x), |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier =
		/// 	<AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> as AffineTraversalOptic<
		/// 		(i32, String),
		/// 		(i32, String),
		/// 		i32,
		/// 		i32,
		/// 	>>::evaluate::<RcFnBrand>(&at, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate<R: Strong + Choice>(
			&self,
			pab: Apply!(<R as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<R as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<R, S, T, A, B>::evaluate(self, pab)
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
	impl<'a, P, S: 'a, T: 'a, A: 'a, B: 'a> TraversalOptic<'a, S, T, A, B>
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
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let at: AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	AffineTraversal::from_preview_set(|(x, s): (i32, String)| Ok(x), |((_, s), x)| (x, s));
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
	impl<'a, P, S: 'a, A: 'a> FoldOptic<'a, S, A> for AffineTraversal<'a, P, S, S, A, A>
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
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let at: AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	AffineTraversal::from_preview_set(|(x, s): (i32, String)| Ok(x), |((_, s), x)| (x, s));
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
	impl<'a, Q, P, S: 'a, T: 'a, A: 'a, B: 'a> SetterOptic<'a, Q, S, T, A, B>
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
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let at: AffineTraversal<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	AffineTraversal::from_preview_set(|(x, s): (i32, String)| Ok(x), |((_, s), x)| (x, s));
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
		pub(crate) to: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, Result<(A, <FnBrand<P> as CloneableFn>::Of<'a, A, S>), S>>),
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
		/// 	AffineTraversalPrime::from_preview_set(|(x, _)| Some(x), |((_, s), x)| (x, s));
		/// let cloned = at.clone();
		/// assert_eq!(cloned.preview((42, "hi".to_string())), Some(42));
		/// ```
		fn clone(&self) -> Self {
			AffineTraversalPrime {
				to: self.to.clone(),
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
		/// Create a new monomorphic affine traversal.
		/// This matches PureScript's `affineTraversal'` constructor.
		#[document_signature]
		///
		#[document_parameters("The getter/setter pair function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::CloneableFn,
		/// 	types::optics::AffineTraversalPrime,
		/// };
		///
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::new(|(x, s): (i32, String)| {
		/// 		Ok((x, <RcFnBrand as CloneableFn>::new(move |a| (a, s.clone()))))
		/// 	});
		/// ```
		pub fn new(
			to: impl 'a + Fn(S) -> Result<(A, <FnBrand<P> as CloneableFn>::Of<'a, A, S>), S>
		) -> Self {
			AffineTraversalPrime {
				to: <FnBrand<P> as CloneableFn>::new(to),
			}
		}

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
		/// 	AffineTraversalPrime::from_preview_set(|(x, _)| Some(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.preview((42, "hi".to_string())), Some(42));
		/// ```
		pub fn from_preview_set(
			preview: impl 'a + Fn(S) -> Option<A>,
			set: impl 'a + Fn((S, A)) -> S,
		) -> Self
		where
			S: Clone, {
			let preview_brand = <FnBrand<P> as CloneableFn>::new(preview);
			let set_brand = <FnBrand<P> as CloneableFn>::new(set);

			AffineTraversalPrime {
				to: <FnBrand<P> as CloneableFn>::new(move |s: S| {
					let s_clone = s.clone();
					let set_brand = set_brand.clone();
					match preview_brand(s.clone()) {
						Some(a) => Ok((
							a,
							<FnBrand<P> as CloneableFn>::new(move |a| {
								set_brand((s_clone.clone(), a))
							}),
						)),
						None => Err(s),
					}
				}),
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
		/// 	AffineTraversalPrime::from_preview_set(|(x, _)| Some(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.preview((42, "hi".to_string())), Some(42));
		/// ```
		pub fn preview(
			&self,
			s: S,
		) -> Option<A> {
			(self.to)(s).ok().map(|(a, _)| a)
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
		/// 	AffineTraversalPrime::from_preview_set(|(x, _)| Some(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.set((42, "hi".to_string()), 99), (99, "hi".to_string()));
		/// ```
		pub fn set(
			&self,
			s: S,
			a: A,
		) -> S {
			match (self.to)(s) {
				Ok((_, f)) => f(a),
				Err(s) => s,
			}
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
		/// 	AffineTraversalPrime::from_preview_set(|(x, _)| Some(x), |((_, s), x)| (x, s));
		/// assert_eq!(at.modify((21, "hi".to_string()), |x| x * 2), (42, "hi".to_string()));
		/// ```
		pub fn modify(
			&self,
			s: S,
			f: impl Fn(A) -> A,
		) -> S {
			match (self.to)(s) {
				Ok((a, set)) => set(f(a)),
				Err(s) => s,
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
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::from_preview_set(|(x, _)| Some(x), |((_, s), x)| (x, s));
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
			let to = self.to.clone();

			Q::dimap(
				move |s: S| to(s),
				move |result: Result<(A, <FnBrand<P> as CloneableFn>::Of<'a, A, S>), S>| {
					match result {
						Ok((a, f)) => f(a),
						Err(s) => s,
					}
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
	impl<'a, P, S, A> AffineTraversalOptic<'a, S, S, A, A> for AffineTraversalPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a,
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
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::from_preview_set(|(x, _)| Some(x), |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <AffineTraversalPrime<RcBrand, (i32, String), i32> as AffineTraversalOptic<
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// >>::evaluate::<RcFnBrand>(&at, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate<R: Strong + Choice>(
			&self,
			pab: Apply!(<R as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<R as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			Optic::<R, S, S, A, A>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic affine traversal instance.")]
	impl<'a, P, S: 'a, A: 'a> TraversalOptic<'a, S, S, A, A> for AffineTraversalPrime<'a, P, S, A>
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
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::from_preview_set(|(x, _)| Some(x), |((_, s), x)| (x, s));
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
	impl<'a, P, S: 'a, A: 'a> FoldOptic<'a, S, A> for AffineTraversalPrime<'a, P, S, A>
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
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::from_preview_set(|(x, _)| Some(x), |((_, s), x)| (x, s));
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
	impl<'a, Q, P, S: 'a, A: 'a> SetterOptic<'a, Q, S, S, A, A> for AffineTraversalPrime<'a, P, S, A>
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
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::from_preview_set(|(x, _)| Some(x), |((_, s), x)| (x, s));
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
