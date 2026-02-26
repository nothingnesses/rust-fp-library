//! Core optic traits.

use {
	crate::{
		Apply,
		brands::FnBrand,
		classes::{
			UnsizedCoercible,
			monoid::Monoid,
			profunctor::{
				Choice,
				Closed,
				Profunctor,
				Strong,
				Wander,
			},
		},
		kinds::*,
		types::optics::{
			ForgetBrand,
			TaggedBrand,
		},
	},
	fp_macros::{
		document_parameters,
		document_signature,
		document_type_parameters,
	},
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
pub trait Optic<'a, P: Profunctor, S: 'a, T: 'a, A: 'a, B: 'a> {
	/// Evaluate the optic with a profunctor.
	///
	/// This method applies the optic transformation to a profunctor value.
	#[document_signature]
	///
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
	/// let l: LensPrime<RcBrand, (i32, String), i32> =
	/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
	///
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
pub trait IsoOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
	/// Evaluate the optic with any profunctor.
	#[document_signature]
	///
	#[document_type_parameters("The profunctor type.")]
	///
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
	/// let iso: IsoPrime<RcBrand, i32, i32> = IsoPrime::new(|x| x, |x| x);
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// let modifier = <IsoPrime<RcBrand, i32, i32> as IsoOptic<i32, i32, i32, i32>>::evaluate::<
	/// 	RcFnBrand,
	/// >(&iso, f);
	/// assert_eq!(modifier(21), 42);
	/// ```
	fn evaluate<P: Profunctor>(
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
pub trait LensOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
	/// Evaluate the optic with a strong profunctor.
	#[document_signature]
	///
	#[document_type_parameters("The profunctor type.")]
	///
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
	/// let l: LensPrime<RcBrand, (i32, String), i32> =
	/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
pub trait PrismOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
	/// Evaluate the optic with a choice profunctor.
	#[document_signature]
	///
	#[document_type_parameters("The profunctor type.")]
	///
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
	/// let p: PrismPrime<RcBrand, Option<i32>, i32> = PrismPrime::from_option(|o| o, Some);
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
pub trait AffineTraversalOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
	/// Evaluate the optic with a strong and choice profunctor.
	#[document_signature]
	///
	#[document_type_parameters("The profunctor type.")]
	///
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
	/// 	AffineTraversalPrime::from_preview_set(|(x, _)| Some(x), |((_, s), x)| (x, s));
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
pub trait TraversalOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
	/// Evaluate the optic with a wander profunctor.
	#[document_signature]
	///
	#[document_type_parameters("The profunctor type.")]
	///
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
	/// // Use Vec's built-in optic support
	/// let v = vec![1, 2, 3];
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
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
	fn evaluate<R: 'a + 'static, P: UnsizedCoercible + 'static>(
		&self,
		pab: Apply!(<ForgetBrand<P, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
	) -> Apply!(<ForgetBrand<P, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>);
}

/// A fold optic.
#[document_type_parameters(
	"The lifetime of the values.",
	"The source type of the structure.",
	"The focus type."
)]
pub trait FoldOptic<'a, S: 'a, A: 'a> {
	/// Evaluate the optic with the forget profunctor for any monoid.
	#[document_signature]
	///
	#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
	///
	#[document_parameters("The profunctor value to transform.")]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::Monoid,
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
	fn evaluate<R: 'a + Monoid + 'static, P: UnsizedCoercible + 'static>(
		&self,
		pab: Apply!(<ForgetBrand<P, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
	) -> Apply!(<ForgetBrand<P, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>);
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
pub trait SetterOptic<'a, P: UnsizedCoercible, S: 'a, T: 'a, A: 'a, B: 'a> {
	/// Evaluate the optic with the function profunctor.
	#[document_signature]
	///
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
	/// let s: SetterPrime<RcBrand, (i32, String), i32> =
	/// 	SetterPrime::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
		pab: Apply!(<FnBrand<P> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
	) -> Apply!(<FnBrand<P> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
}

/// A grate optic.
#[document_type_parameters(
	"The lifetime of the values.",
	"The source type of the structure.",
	"The target type of the structure after an update.",
	"The source type of the focus.",
	"The target type of the focus after an update."
)]
pub trait GrateOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
	/// Evaluate the optic with a closed profunctor.
	#[document_signature]
	///
	#[document_type_parameters("The profunctor type.")]
	///
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
	/// // Simple example showing the grate concept
	/// let pair = (21, 10);
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// assert_eq!(f(pair.0), 42);
	/// assert_eq!(f(pair.1), 20);
	/// ```
	fn evaluate<P: Closed>(
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
pub trait ReviewOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
	/// Evaluate the optic with the tagged profunctor.
	#[document_signature]
	///
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
