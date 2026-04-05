//! Traits for optics.

pub mod fold;
pub mod indexed_traversal;
pub mod traversal;

pub use {
	fold::*,
	indexed_traversal::*,
	traversal::*,
};

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::{
				optics::*,
				*,
			},
			classes::{
				profunctor::*,
				*,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// A trait for optics that can be evaluated with any profunctor constraint.
	///
	/// This trait allows optics to be first-class values that can be composed
	/// and stored while preserving their polymorphism over profunctor types.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The optic instance.")]
	pub trait Optic<'a, P: Profunctor, S: 'a, T: 'a, A: 'a, B: 'a> {
		/// Evaluate the optic with a profunctor.
		///
		/// This method applies the optic transformation to a profunctor value.
		#[document_signature]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier =
		/// 	<LensPrime<RcBrand, (i32, String), i32> as Optic<RcFnBrand, _, _, _, _>>::evaluate(&l, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// An isomorphism optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The optic instance.")]
	pub trait IsoOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
		/// Evaluate the optic with any profunctor.
		#[document_signature]
		///
		#[document_type_parameters("The profunctor type.")]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, i32, i32> = IsoPrime::new(|x| x, |x| x);
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <IsoPrime<RcBrand, i32, i32> as IsoOptic<i32, i32, i32, i32>>::evaluate::<
		/// 	RcFnBrand,
		/// >(&iso, f);
		/// assert_eq!(modifier(21), 42);
		/// ```
		fn evaluate<P: Profunctor + 'static>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// A lens optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The optic instance.")]
	pub trait LensOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
		/// Evaluate the optic with a strong profunctor.
		#[document_signature]
		///
		#[document_type_parameters("The profunctor type.")]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <LensPrime<RcBrand, (i32, String), i32> as LensOptic<
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// >>::evaluate::<RcFnBrand>(&l, f);
		/// assert_eq!(modifier((21, "hi".to_string())), (42, "hi".to_string()));
		/// ```
		fn evaluate<P: Strong>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// A prism optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The optic instance.")]
	pub trait PrismOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
		/// Evaluate the optic with a choice profunctor.
		#[document_signature]
		///
		#[document_type_parameters("The profunctor type.")]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let p: PrismPrime<RcBrand, Option<i32>, i32> = PrismPrime::from_option(|o| o, Some);
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <PrismPrime<RcBrand, Option<i32>, i32> as PrismOptic<
		/// 	Option<i32>,
		/// 	Option<i32>,
		/// 	i32,
		/// 	i32,
		/// >>::evaluate::<RcFnBrand>(&p, f);
		/// assert_eq!(modifier(Some(21)), Some(42));
		/// ```
		fn evaluate<P: Choice>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// An affine traversal optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The optic instance.")]
	pub trait AffineTraversalOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
		/// Evaluate the optic with a strong and choice profunctor.
		#[document_signature]
		///
		#[document_type_parameters("The profunctor type.")]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let at: AffineTraversalPrime<RcBrand, (i32, String), i32> =
		/// 	AffineTraversalPrime::from_preview_set(|(x, _)| Some(x), |((_, s), x)| (x, s));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <AffineTraversalPrime<RcBrand, (i32, String), i32> as AffineTraversalOptic<
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// >>::evaluate::<RcFnBrand>(&at, f);
		/// assert_eq!(modifier((21, "hi".to_string())), (42, "hi".to_string()));
		/// ```
		fn evaluate<P: Strong + Choice>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// A traversal optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The optic instance.")]
	pub trait TraversalOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
		/// Evaluate the optic with a wander profunctor.
		#[document_signature]
		///
		#[document_type_parameters("The profunctor type.")]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// // Use Vec's built-in optic support
		/// let v = vec![1, 2, 3];
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let result = f(v[0]); // Simple example showing the concept
		/// assert_eq!(result, 2);
		/// ```
		fn evaluate<P: Wander>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// A getter optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the structure.",
		"The focus type."
	)]
	#[document_parameters("The optic instance.")]
	pub trait GetterOptic<'a, S: 'a, A: 'a> {
		/// Evaluate the optic with the forget profunctor.
		#[document_signature]
		///
		#[document_type_parameters(
			"The return type of the forget profunctor.",
			"The reference-counted pointer type."
		)]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_returns("The transformed forget profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let g: GetterPrime<RcBrand, (i32, String), i32> = GetterPrime::new(|(x, _)| x);
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded =
		/// 	<GetterPrime<RcBrand, (i32, String), i32> as GetterOptic<(i32, String), i32>>::evaluate::<
		/// 		i32,
		/// 		RcBrand,
		/// 	>(&g, f);
		/// assert_eq!(folded.run((42, "hi".to_string())), 42);
		/// ```
		fn evaluate<R: 'a + 'static, PointerBrand: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<PointerBrand, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<PointerBrand, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>);
	}

	/// A fold optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the structure.",
		"The focus type."
	)]
	#[document_parameters("The optic instance.")]
	pub trait FoldOptic<'a, S: 'a, A: 'a> {
		/// Evaluate the optic with the forget profunctor for any monoid.
		#[document_signature]
		///
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_returns("The transformed forget profunctor value.")]
		///
		/// ### `R: Clone` Requirement
		///
		/// The result monoid `R` must implement [`Clone`] because the fold is internally implemented
		/// via [`Wander`] with the [`Forget`](crate::types::optics::Forget) profunctor, which stores
		/// `R` inside [`Const`](crate::types::const_val::Const). The traversal applies
		/// [`TraversalFunc::apply`] with
		/// [`ConstBrand<R>`](crate::brands::ConstBrand) as the applicative, and that
		/// requires `Const<R, B>: Clone`, which in turn requires `R: Clone`.
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		Monoid,
		/// 		optics::*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		/// let f_optic: FoldPrime<RcBrand, Vec<i32>, i32, _> =
		/// 	FoldPrime::new(IterableFoldFn(|v: Vec<i32>| v));
		/// let f = Forget::<RcBrand, String, i32, i32>::new(|x| x.to_string());
		/// let folded = <FoldPrime<RcBrand, Vec<i32>, i32, _> as FoldOptic<Vec<i32>, i32>>::evaluate::<
		/// 	String,
		/// 	RcBrand,
		/// >(&f_optic, f);
		/// assert_eq!(folded.run(vec![1, 2, 3]), "123".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + Clone + 'static, PointerBrand: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<PointerBrand, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<PointerBrand, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>);
	}

	/// A setter optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The optic instance.")]
	pub trait SetterOptic<'a, PointerBrand: UnsizedCoercible, S: 'a, T: 'a, A: 'a, B: 'a> {
		/// Evaluate the optic with the function profunctor.
		#[document_signature]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_returns("The transformed function profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let s: SetterPrime<RcBrand, (i32, String), i32> =
		/// 	SetterPrime::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <SetterPrime<RcBrand, (i32, String), i32> as SetterOptic<
		/// 	RcBrand,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// >>::evaluate(&s, f);
		/// assert_eq!(modifier((21, "hi".to_string())), (42, "hi".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// An indexed lens optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The optic instance.")]
	pub trait IndexedLensOptic<'a, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a> {
		/// Evaluate the optic with a strong profunctor.
		#[document_signature]
		///
		#[document_type_parameters("The profunctor type.")]
		///
		#[document_parameters("The indexed profunctor value to transform.")]
		///
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		/// assert_eq!(l.over((21, "hi".to_string()), |x| x * 2), (42, "hi".to_string()));
		/// ```
		fn evaluate<P: Strong>(
			&self,
			pab: crate::types::optics::Indexed<'a, P, I, A, B>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// An indexed traversal optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The optic instance.")]
	pub trait IndexedTraversalOptic<'a, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a> {
		/// Evaluate the optic with a wander profunctor.
		#[document_signature]
		///
		#[document_type_parameters("The profunctor type.")]
		///
		#[document_parameters("The indexed profunctor value to transform.")]
		///
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let v = vec![1, 2, 3];
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// assert_eq!(f(v[0]), 2);
		/// ```
		fn evaluate<P: Wander>(
			&self,
			pab: crate::types::optics::Indexed<'a, P, I, A, B>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// An indexed getter optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The index type.",
		"The source type of the structure.",
		"The focus type."
	)]
	#[document_parameters("The optic instance.")]
	pub trait IndexedGetterOptic<'a, I: 'a, S: 'a, A: 'a> {
		/// Evaluate the optic with the forget profunctor.
		#[document_signature]
		///
		#[document_type_parameters(
			"The return type of the forget profunctor.",
			"The reference-counted pointer type."
		)]
		///
		#[document_parameters("The indexed profunctor value to transform.")]
		///
		#[document_returns("The transformed forget profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let g: GetterPrime<RcBrand, (i32, String), i32> = GetterPrime::new(|(x, _)| x);
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded =
		/// 	<GetterPrime<RcBrand, (i32, String), i32> as GetterOptic<(i32, String), i32>>::evaluate::<
		/// 		i32,
		/// 		RcBrand,
		/// 	>(&g, f);
		/// assert_eq!(folded.run((42, "hi".to_string())), 42);
		/// ```
		fn evaluate<R: 'a + 'static, PointerBrand: UnsizedCoercible + 'static>(
			&self,
			pab: crate::types::optics::Indexed<'a, ForgetBrand<PointerBrand, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<PointerBrand, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>);
	}

	/// An indexed fold optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The index type.",
		"The source type of the structure.",
		"The focus type."
	)]
	#[document_parameters("The optic instance.")]
	pub trait IndexedFoldOptic<'a, I: 'a, S: 'a, A: 'a> {
		/// Evaluate the optic with the forget profunctor for any monoid.
		#[document_signature]
		///
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
		///
		#[document_parameters("The indexed profunctor value to transform.")]
		///
		#[document_returns("The transformed forget profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		Monoid,
		/// 		optics::*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		/// let f_optic: FoldPrime<RcBrand, Vec<i32>, i32, _> =
		/// 	FoldPrime::new(IterableFoldFn(|v: Vec<i32>| v));
		/// let f = Forget::<RcBrand, String, i32, i32>::new(|x| x.to_string());
		/// let folded = <FoldPrime<RcBrand, Vec<i32>, i32, _> as FoldOptic<Vec<i32>, i32>>::evaluate::<
		/// 	String,
		/// 	RcBrand,
		/// >(&f_optic, f);
		/// assert_eq!(folded.run(vec![1, 2, 3]), "123".to_string());
		/// ```
		///
		/// ### `R: Clone` Requirement
		///
		/// The result monoid `R` must implement [`Clone`] because the fold is internally implemented
		/// via [`Wander`] with the [`Forget`](crate::types::optics::Forget) profunctor, which stores
		/// `R` inside [`Const`](crate::types::const_val::Const). The traversal applies
		/// [`IndexedTraversalFunc::apply`](crate::classes::optics::IndexedTraversalFunc::apply) with
		/// [`ConstBrand<R>`](crate::brands::ConstBrand) as the applicative, and that
		/// requires `Const<R, B>: Clone`, which in turn requires `R: Clone`.
		fn evaluate<R: 'a + Monoid + Clone + 'static, PointerBrand: UnsizedCoercible + 'static>(
			&self,
			pab: crate::types::optics::Indexed<'a, ForgetBrand<PointerBrand, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<PointerBrand, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>);
	}

	/// An indexed setter optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The optic instance.")]
	pub trait IndexedSetterOptic<
		'a,
		PointerBrand: UnsizedCoercible,
		I: 'a,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	> {
		/// Evaluate the optic with the function profunctor.
		#[document_signature]
		///
		#[document_parameters("The indexed profunctor value to transform.")]
		///
		#[document_returns("The transformed function profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let s: SetterPrime<RcBrand, (i32, String), i32> =
		/// 	SetterPrime::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <SetterPrime<RcBrand, (i32, String), i32> as SetterOptic<
		/// 	RcBrand,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// >>::evaluate(&s, f);
		/// assert_eq!(modifier((21, "hi".to_string())), (42, "hi".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: crate::types::optics::Indexed<'a, FnBrand<PointerBrand>, I, A, B>,
		) -> Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// A grate optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand used by the profunctor's `Closed` instance.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The optic instance.")]
	pub trait GrateOptic<'a, FunctionBrand: CloneFn, S: 'a, T: 'a, A: 'a, B: 'a> {
		/// Evaluate the optic with a closed profunctor.
		#[document_signature]
		///
		#[document_type_parameters("The profunctor type.")]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// // Simple example showing the grate concept
		/// let pair = (21, 10);
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// assert_eq!(f(pair.0), 42);
		/// assert_eq!(f(pair.1), 20);
		/// ```
		fn evaluate<P: Closed<FunctionBrand>>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// A review optic.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The optic instance.")]
	pub trait ReviewOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
		/// Evaluate the optic with the tagged profunctor.
		#[document_signature]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_returns("The transformed tagged profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let r: PrismPrime<RcBrand, Option<i32>, i32> = PrismPrime::from_option(|o| o, Some);
		/// let f = Tagged::new(21);
		/// let reviewed = <PrismPrime<RcBrand, Option<i32>, i32> as ReviewOptic<
		/// 	Option<i32>,
		/// 	Option<i32>,
		/// 	i32,
		/// 	i32,
		/// >>::evaluate(&r, f);
		/// assert_eq!(reviewed.0, Some(21));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<TaggedBrand as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<TaggedBrand as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// Helper trait for `optics_un_index`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus."
	)]
	#[document_parameters("The adapter instance.")]
	pub trait IndexedOpticAdapter<'a, P: Profunctor, I, S, T, A, B> {
		/// Evaluate the optic with an indexed profunctor.
		#[document_signature]
		///
		#[document_parameters("The indexed profunctor value.")]
		///
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		/// assert_eq!(l.over((21, "hi".to_string()), |x| x * 2), (42, "hi".to_string()));
		/// ```
		fn evaluate_indexed(
			&self,
			pab: crate::types::optics::Indexed<'a, P, I, A, B>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// Helper trait for `optics_as_index`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus."
	)]
	#[document_parameters("The adapter instance.")]
	pub trait IndexedOpticAdapterDiscardsFocus<'a, P: Profunctor, I, S, T, A, B> {
		/// Evaluate the optic with an indexed profunctor, discarding the focus.
		#[document_signature]
		///
		#[document_parameters("The indexed profunctor value.")]
		///
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		/// assert_eq!(l.over((21, "hi".to_string()), |x| x * 2), (42, "hi".to_string()));
		/// ```
		fn evaluate_indexed_discards_focus(
			&self,
			pab: crate::types::optics::Indexed<'a, P, I, A, B>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}
}

pub use inner::*;
