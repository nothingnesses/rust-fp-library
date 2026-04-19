//! The `Composed` struct for composing optics.

use {
	crate::{
		Apply,
		brands::{
			FnBrand,
			optics::*,
		},
		classes::{
			LiftFn,
			ToDynCloneFn,
			monoid::Monoid,
			optics::*,
			profunctor::{
				Choice,
				Closed,
				Profunctor,
				Strong,
				Wander,
			},
		},
		kinds::*,
	},
	fp_macros::*,
	std::marker::PhantomData,
};

#[fp_macros::document_module]
mod inner {
	use super::*;

	/// This struct represents the composition of two optics, allowing them to be
	/// combined into a single optic that applies both transformations.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct Composed<'a, S, T, M, N, A, B, O1, O2> {
		/// The outer optic (applied second).
		pub first: O1,
		/// The inner optic (applied first).
		pub second: O2,
		pub(crate) _phantom: PhantomData<&'a (S, T, M, N, A, B)>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	impl<'a, S, T, M, N, A, B, O1, O2> Composed<'a, S, T, M, N, A, B, O1, O2> {
		/// Create a new composed optic.
		#[document_signature]
		///
		#[document_parameters(
			"The outer optic (applied second).",
			"The inner optic (applied first)."
		)]
		///
		#[document_returns("A new instance of the type.")]
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
		/// let l1: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _): (i32, String)| x, |((_, s), x)| (x, s));
		/// let l2: LensPrime<RcBrand, i32, i32> = LensPrime::from_view_set(|x: i32| x, |(_, x)| x);
		/// let composed = Composed::new(l1, l2);
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <Composed<
		/// 	'_,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	LensPrime<RcBrand, (i32, String), i32>,
		/// 	LensPrime<RcBrand, i32, i32>,
		/// > as Optic<RcFnBrand, _, _, _, _>>::evaluate(&composed, f);
		/// assert_eq!(modifier((21, "hi".to_string())), (42, "hi".to_string()));
		/// ```
		pub fn new(
			first: O1,
			second: O2,
		) -> Self {
			Composed {
				first,
				second,
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, P, S: 'a, T: 'a, M: 'a, N: 'a, A: 'a, B: 'a, O1, O2> Optic<'a, P, S, T, A, B>
		for Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		P: Profunctor,
		O1: Optic<'a, P, S, T, M, N>,
		O2: Optic<'a, P, M, N, A, B>,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
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
		/// let l1: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _): (i32, String)| x, |((_, s), x)| (x, s));
		/// let l2: LensPrime<RcBrand, i32, i32> = LensPrime::from_view_set(|x: i32| x, |(_, x)| x);
		/// let composed = Composed::new(l1, l2);
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <Composed<
		/// 	'_,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	LensPrime<RcBrand, (i32, String), i32>,
		/// 	LensPrime<RcBrand, i32, i32>,
		/// > as Optic<RcFnBrand, _, _, _, _>>::evaluate(&composed, f);
		/// assert_eq!(modifier((21, "hi".to_string())), (42, "hi".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let pmn = self.second.evaluate(pab);
			self.first.evaluate(pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, S, T, M, N, A, B, O1, O2> IsoOptic<'a, S, T, A, B>
		for Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		O1: IsoOptic<'a, S, T, M, N>,
		O2: IsoOptic<'a, M, N, A, B>,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
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
		/// let iso1: IsoPrime<RcBrand, i32, i32> = IsoPrime::new(|x| x, |x| x);
		/// let iso2: IsoPrime<RcBrand, i32, i32> = IsoPrime::new(|x| x, |x| x);
		/// let composed = Composed::new(iso1, iso2);
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <Composed<
		/// 	'_,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	IsoPrime<RcBrand, i32, i32>,
		/// 	IsoPrime<RcBrand, i32, i32>,
		/// > as IsoOptic<i32, i32, i32, i32>>::evaluate::<RcFnBrand>(&composed, f);
		/// assert_eq!(modifier(21), 42);
		/// ```
		fn evaluate<P: Profunctor + 'static>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let pmn = IsoOptic::evaluate::<P>(&self.second, pab);
			IsoOptic::evaluate::<P>(&self.first, pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, S, T, M, N, A, B, O1, O2> LensOptic<'a, S, T, A, B>
		for Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		O1: LensOptic<'a, S, T, M, N>,
		O2: LensOptic<'a, M, N, A, B>,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
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
		/// let l1: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		/// let l2: LensPrime<RcBrand, i32, i32> = LensPrime::from_view_set(|x| x, |(_, x)| x);
		/// let composed = Composed::new(l1, l2);
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <Composed<
		/// 	'_,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	LensPrime<RcBrand, (i32, String), i32>,
		/// 	LensPrime<RcBrand, i32, i32>,
		/// > as LensOptic<(i32, String), (i32, String), i32, i32>>::evaluate::<RcFnBrand>(
		/// 	&composed, f
		/// );
		/// assert_eq!(modifier((21, "hi".to_string())), (42, "hi".to_string()));
		/// ```
		fn evaluate<P: Strong>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let pmn = LensOptic::evaluate::<P>(&self.second, pab);
			LensOptic::evaluate::<P>(&self.first, pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, S, T, M, N, A, B, O1, O2> PrismOptic<'a, S, T, A, B>
		for Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		O1: PrismOptic<'a, S, T, M, N>,
		O2: PrismOptic<'a, M, N, A, B>,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
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
		/// let p1: PrismPrime<RcBrand, Option<i32>, i32> = PrismPrime::from_option(|o| o, Some);
		/// let p2: PrismPrime<RcBrand, i32, i32> = PrismPrime::from_option(Some, |x| x);
		/// let composed = Composed::new(p1, p2);
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier =
		/// 	<Composed<
		/// 		'_,
		/// 		Option<i32>,
		/// 		Option<i32>,
		/// 		i32,
		/// 		i32,
		/// 		i32,
		/// 		i32,
		/// 		PrismPrime<RcBrand, Option<i32>, i32>,
		/// 		PrismPrime<RcBrand, i32, i32>,
		/// 	> as PrismOptic<Option<i32>, Option<i32>, i32, i32>>::evaluate::<RcFnBrand>(&composed, f);
		/// assert_eq!(modifier(Some(21)), Some(42));
		/// ```
		fn evaluate<P: Choice>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let pmn = PrismOptic::evaluate::<P>(&self.second, pab);
			PrismOptic::evaluate::<P>(&self.first, pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, S, T, M, N, A, B, O1, O2> AffineTraversalOptic<'a, S, T, A, B>
		for Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		O1: AffineTraversalOptic<'a, S, T, M, N>,
		O2: AffineTraversalOptic<'a, M, N, A, B>,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
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
		/// let l1: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _): (i32, String)| x, |((_, s), x)| (x, s));
		/// let p2: PrismPrime<RcBrand, i32, i32> = PrismPrime::from_option(Some, |x| x);
		/// let composed = Composed::new(l1, p2);
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <Composed<
		/// 	'_,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	LensPrime<RcBrand, (i32, String), i32>,
		/// 	PrismPrime<RcBrand, i32, i32>,
		/// > as AffineTraversalOptic<(i32, String), (i32, String), i32, i32>>::evaluate::<RcFnBrand>(
		/// 	&composed, f,
		/// );
		/// assert_eq!(modifier((21, "hi".to_string())), (42, "hi".to_string()));
		/// ```
		fn evaluate<P: Strong + Choice>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let pmn = AffineTraversalOptic::evaluate::<P>(&self.second, pab);
			AffineTraversalOptic::evaluate::<P>(&self.first, pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, S, T, M, N, A, B, O1, O2> TraversalOptic<'a, S, T, A, B>
		for Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		O1: TraversalOptic<'a, S, T, M, N>,
		O2: TraversalOptic<'a, M, N, A, B>,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
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
		/// // Composition combines two optics
		/// let l1: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		/// let l2: LensPrime<RcBrand, i32, i32> = LensPrime::from_view_set(|x| x, |(_, x)| x);
		/// let composed = Composed::new(l1, l2);
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <Composed<
		/// 	'_,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	LensPrime<RcBrand, (i32, String), i32>,
		/// 	LensPrime<RcBrand, i32, i32>,
		/// > as Optic<RcFnBrand, _, _, _, _>>::evaluate(&composed, f);
		/// assert_eq!(modifier((21, "hi".to_string())), (42, "hi".to_string()));
		/// ```
		fn evaluate<P: Wander>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let pmn = TraversalOptic::evaluate::<P>(&self.second, pab);
			TraversalOptic::evaluate::<P>(&self.first, pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the structure.",
		"The focus type.",
		"The intermediate type.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, S, A, M, O1, O2> GetterOptic<'a, S, A> for Composed<'a, S, S, M, M, A, A, O1, O2>
	where
		O1: GetterOptic<'a, S, M>,
		O2: GetterOptic<'a, M, A>,
		M: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_type_parameters(
			"The return type of the forget profunctor.",
			"The reference-counted pointer type."
		)]
		#[document_parameters("The profunctor value to transform.")]
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
		/// let g1: GetterPrime<RcBrand, (i32, String), i32> = GetterPrime::new(|(x, _)| x);
		/// let g2: GetterPrime<RcBrand, i32, i32> = GetterPrime::new(|x| x);
		/// let composed = Composed::new(g1, g2);
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded = <Composed<
		/// 	'_,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	GetterPrime<RcBrand, (i32, String), i32>,
		/// 	GetterPrime<RcBrand, i32, i32>,
		/// > as GetterOptic<(i32, String), i32>>::evaluate::<i32, RcBrand>(&composed, f);
		/// assert_eq!(folded.run((42, "hi".to_string())), 42);
		/// ```
		fn evaluate<R: 'a + 'static, PointerBrand: ToDynCloneFn + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<PointerBrand, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<PointerBrand, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			let pmn = GetterOptic::evaluate::<R, PointerBrand>(&self.second, pab);
			GetterOptic::evaluate::<R, PointerBrand>(&self.first, pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the structure.",
		"The focus type.",
		"The intermediate type.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, S, A, M, O1, O2> FoldOptic<'a, S, A> for Composed<'a, S, S, M, M, A, A, O1, O2>
	where
		O1: FoldOptic<'a, S, M>,
		O2: FoldOptic<'a, M, A>,
		M: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
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
		/// let f1: FoldPrime<RcBrand, Vec<i32>, i32, _> = FoldPrime::new(IterableFoldFn(|v: Vec<i32>| v));
		/// let f2: FoldPrime<RcBrand, i32, i32, _> =
		/// 	FoldPrime::new(IterableFoldFn(|x: i32| std::iter::once(x)));
		/// let composed = Composed::new(f1, f2);
		/// let f = Forget::<RcBrand, String, i32, i32>::new(|x| x.to_string());
		/// let folded = <Composed<
		/// 	'_,
		/// 	Vec<i32>,
		/// 	Vec<i32>,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	FoldPrime<RcBrand, Vec<i32>, i32, _>,
		/// 	FoldPrime<RcBrand, i32, i32, _>,
		/// > as FoldOptic<Vec<i32>, i32>>::evaluate::<String, RcBrand>(&composed, f);
		/// assert_eq!(folded.run(vec![1, 2, 3]), "123".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + Clone + 'static, PointerBrand: ToDynCloneFn + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<PointerBrand, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<PointerBrand, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			let pmn = FoldOptic::evaluate::<R, PointerBrand>(&self.second, pab);
			FoldOptic::evaluate::<R, PointerBrand>(&self.first, pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the Setter brand.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, PointerBrand, S, T, M, N, A, B, O1, O2> SetterOptic<'a, PointerBrand, S, T, A, B>
		for Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		PointerBrand: ToDynCloneFn,
		O1: SetterOptic<'a, PointerBrand, S, T, M, N>,
		O2: SetterOptic<'a, PointerBrand, M, N, A, B>,
		M: 'a,
		N: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
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
		/// let s1: SetterPrime<RcBrand, (i32, String), i32> =
		/// 	SetterPrime::new(|(s, f): ((i32, String), Box<dyn Fn(i32) -> i32>)| (f(s.0), s.1));
		/// let s2: SetterPrime<RcBrand, i32, i32> =
		/// 	SetterPrime::new(|(s, f): (i32, Box<dyn Fn(i32) -> i32>)| f(s));
		/// let composed = Composed::new(s1, s2);
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier =
		/// 	<Composed<
		/// 		'_,
		/// 		(i32, String),
		/// 		(i32, String),
		/// 		i32,
		/// 		i32,
		/// 		i32,
		/// 		i32,
		/// 		SetterPrime<RcBrand, (i32, String), i32>,
		/// 		SetterPrime<RcBrand, i32, i32>,
		/// 	> as SetterOptic<RcBrand, (i32, String), (i32, String), i32, i32>>::evaluate(&composed, f);
		/// assert_eq!(modifier((21, "hi".to_string())), (42, "hi".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>)
		{
			let pmn = SetterOptic::evaluate(&self.second, pab);
			SetterOptic::evaluate(&self.first, pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, S, T, M, N, A, B, O1, O2> ReviewOptic<'a, S, T, A, B>
		for Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		O1: ReviewOptic<'a, S, T, M, N>,
		O2: ReviewOptic<'a, M, N, A, B>,
		M: 'a,
		N: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
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
		/// let r1: ReviewPrime<RcBrand, Option<i32>, i32> = ReviewPrime::new(Some);
		/// let r2: ReviewPrime<RcBrand, i32, i32> = ReviewPrime::new(|x| x);
		/// let composed = Composed::new(r1, r2);
		/// let f = Tagged::new(21);
		/// let reviewed = <Composed<
		/// 	'_,
		/// 	Option<i32>,
		/// 	Option<i32>,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	ReviewPrime<RcBrand, Option<i32>, i32>,
		/// 	ReviewPrime<RcBrand, i32, i32>,
		/// > as ReviewOptic<Option<i32>, Option<i32>, i32, i32>>::evaluate(&composed, f);
		/// assert_eq!(reviewed.0, Some(21));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<TaggedBrand as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<TaggedBrand as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let pmn = ReviewOptic::evaluate(&self.second, pab);
			ReviewOptic::evaluate(&self.first, pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand used by the profunctor's `Closed` instance.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, FunctionBrand: LiftFn, S, T, M, N, A, B, O1, O2>
		GrateOptic<'a, FunctionBrand, S, T, A, B> for Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		O1: GrateOptic<'a, FunctionBrand, S, T, M, N>,
		O2: GrateOptic<'a, FunctionBrand, M, N, A, B>,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
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
		/// // Composition works with lenses too
		/// let l1: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		/// let l2: LensPrime<RcBrand, i32, i32> = LensPrime::from_view_set(|x| x, |(_, x)| x);
		/// let composed = Composed::new(l1, l2);
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = <Composed<
		/// 	'_,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	LensPrime<RcBrand, (i32, String), i32>,
		/// 	LensPrime<RcBrand, i32, i32>,
		/// > as Optic<RcFnBrand, _, _, _, _>>::evaluate(&composed, f);
		/// assert_eq!(modifier((21, "test".to_string())), (42, "test".to_string()));
		/// ```
		fn evaluate<P: Closed<FunctionBrand>>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let pmn = GrateOptic::<FunctionBrand, M, N, A, B>::evaluate::<P>(&self.second, pab);
			GrateOptic::<FunctionBrand, S, T, M, N>::evaluate::<P>(&self.first, pmn)
		}
	}

	/// Compose two optics into a single optic.
	///
	/// While PureScript uses the `Semigroupoid` operator (`<<<`) for composition because
	/// its optics are functions, this library uses a specialized `Composed` struct.
	/// This is necessary because Rust represents the polymorphic profunctor constraint
	/// as a parameterized trait (`Optic<'a, P, ...>`), and the `Composed` struct enables
	/// static dispatch and zero-cost composition through monomorphization.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	///
	#[document_parameters("The outer optic (applied second).", "The inner optic (applied first).")]
	///
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
	/// #[derive(Clone, Debug, PartialEq)]
	/// struct Address {
	/// 	street: String,
	/// }
	/// #[derive(Clone, Debug, PartialEq)]
	/// struct User {
	/// 	address: Address,
	/// }
	///
	/// let address_lens: LensPrime<RcBrand, User, Address> = LensPrime::from_view_set(
	/// 	|u: User| u.address.clone(),
	/// 	|(_, a)| User {
	/// 		address: a,
	/// 	},
	/// );
	/// let street_lens: LensPrime<RcBrand, Address, String> = LensPrime::from_view_set(
	/// 	|a: Address| a.street.clone(),
	/// 	|(_, s)| Address {
	/// 		street: s,
	/// 	},
	/// );
	///
	/// let user_street = optics_compose(address_lens, street_lens);
	/// let user = User {
	/// 	address: Address {
	/// 		street: "High St".to_string(),
	/// 	},
	/// };
	///
	/// let f = lift_fn_new::<RcFnBrand, _, _>(|s: String| s.to_uppercase());
	/// let modifier = <Composed<
	/// 	'_,
	/// 	User,
	/// 	User,
	/// 	Address,
	/// 	Address,
	/// 	String,
	/// 	String,
	/// 	LensPrime<RcBrand, User, Address>,
	/// 	LensPrime<RcBrand, Address, String>,
	/// > as Optic<RcFnBrand, _, _, _, _>>::evaluate(&user_street, f);
	/// let updated = modifier(user);
	///
	/// assert_eq!(updated.address.street, "HIGH ST");
	/// ```
	pub fn optics_compose<'a, S: 'a, T: 'a, M: 'a, N: 'a, A: 'a, B: 'a, O1, O2>(
		first: O1,
		second: O2,
	) -> Composed<'a, S, T, M, N, A, B, O1, O2> {
		Composed::new(first, second)
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The index type.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, I: 'a, S: 'a, T: 'a, M: 'a, N: 'a, A: 'a, B: 'a, O1, O2>
		IndexedLensOptic<'a, I, S, T, A, B> for Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		O1: LensOptic<'a, S, T, M, N>,
		O2: IndexedLensOptic<'a, I, M, N, A, B>,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
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
		/// let l1: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		/// let l2: IndexedLensPrime<RcBrand, usize, i32, i32> =
		/// 	IndexedLensPrime::from_iview_set(|x| (0, x), |(_, x)| x);
		/// let composed = Composed::new(l1, l2);
		/// let f = |(i, x): (usize, i32)| x + (i as i32);
		/// let indexed = Indexed::<RcFnBrand, usize, i32, i32>::new(std::rc::Rc::new(f));
		/// let modifier = <Composed<
		/// 	'_,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	LensPrime<RcBrand, (i32, String), i32>,
		/// 	IndexedLensPrime<RcBrand, usize, i32, i32>,
		/// > as IndexedLensOptic<usize, (i32, String), (i32, String), i32, i32>>::evaluate::<RcFnBrand>(
		/// 	&composed, indexed,
		/// );
		/// assert_eq!(modifier((21, "hi".to_string())), (21, "hi".to_string()));
		/// ```
		fn evaluate<P: Strong>(
			&self,
			pab: crate::types::optics::Indexed<'a, P, I, A, B>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let pmn = self.second.evaluate(pab);
			self.first.evaluate::<P>(pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The index type.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, I: 'a, S: 'a, T: 'a, M: 'a, N: 'a, A: 'a, B: 'a, O1, O2>
		IndexedTraversalOptic<'a, I, S, T, A, B> for Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		O1: TraversalOptic<'a, S, T, M, N>,
		O2: IndexedTraversalOptic<'a, I, M, N, A, B>,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
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
		/// let l1: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		/// let l2: IndexedLensPrime<RcBrand, usize, i32, i32> =
		/// 	IndexedLensPrime::from_iview_set(|x| (0, x), |(_, x)| x);
		/// let composed = Composed::new(l1, l2);
		/// let f = |(i, x): (usize, i32)| x + (i as i32);
		/// let indexed = Indexed::<RcFnBrand, usize, i32, i32>::new(std::rc::Rc::new(f));
		/// let modifier = <Composed<
		/// 	'_,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	LensPrime<RcBrand, (i32, String), i32>,
		/// 	IndexedLensPrime<RcBrand, usize, i32, i32>,
		/// > as IndexedTraversalOptic<usize, (i32, String), (i32, String), i32, i32>>::evaluate::<RcFnBrand>(
		/// 	&composed, indexed,
		/// );
		/// assert_eq!(modifier((21, "hi".to_string())), (21, "hi".to_string()));
		/// ```
		fn evaluate<P: Wander>(
			&self,
			pab: crate::types::optics::Indexed<'a, P, I, A, B>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let pmn = self.second.evaluate(pab);
			self.first.evaluate::<P>(pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The index type.",
		"The source type of the structure.",
		"The focus type.",
		"The intermediate type.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, I: 'a, S: 'a, A: 'a, M: 'a, O1, O2> IndexedGetterOptic<'a, I, S, A>
		for Composed<'a, S, S, M, M, A, A, O1, O2>
	where
		O1: GetterOptic<'a, S, M>,
		O2: IndexedGetterOptic<'a, I, M, A>,
	{
		#[document_signature]
		#[document_type_parameters(
			"The return type of the forget profunctor.",
			"The reference-counted pointer type."
		)]
		#[document_parameters("The indexed profunctor value to transform.")]
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
		/// let l1: GetterPrime<RcBrand, (i32, String), i32> = GetterPrime::new(|(x, _)| x);
		/// let l2: IndexedGetterPrime<RcBrand, usize, i32, i32> = IndexedGetterPrime::new(|x| (0, x));
		/// let composed = Composed::new(l1, l2);
		/// let f = Forget::<RcBrand, (usize, i32), (usize, i32), i32>::new(|(i, x)| (i, x));
		/// let folded = <Composed<
		/// 	'_,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	GetterPrime<RcBrand, (i32, String), i32>,
		/// 	IndexedGetterPrime<RcBrand, usize, i32, i32>,
		/// > as IndexedGetterOptic<usize, (i32, String), i32>>::evaluate::<(usize, i32), RcBrand>(
		/// 	&composed,
		/// 	Indexed::new(f),
		/// );
		/// assert_eq!(folded.run((42, "hi".to_string())), (0, 42));
		/// ```
		fn evaluate<R: 'a + 'static, PointerBrand: ToDynCloneFn + 'static>(
			&self,
			pab: crate::types::optics::Indexed<'a, ForgetBrand<PointerBrand, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<PointerBrand, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			let pmn = self.second.evaluate::<R, PointerBrand>(pab);
			self.first.evaluate::<R, PointerBrand>(pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The index type.",
		"The source type of the structure.",
		"The focus type.",
		"The intermediate type.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, I: 'a, S: 'a, A: 'a, M: 'a, O1, O2> IndexedFoldOptic<'a, I, S, A>
		for Composed<'a, S, S, M, M, A, A, O1, O2>
	where
		O1: FoldOptic<'a, S, M>,
		O2: IndexedFoldOptic<'a, I, M, A>,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
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
		/// let l1: GetterPrime<RcBrand, (i32, String), i32> = GetterPrime::new(|(x, _)| x);
		/// let l2: IndexedGetterPrime<RcBrand, usize, i32, i32> = IndexedGetterPrime::new(|x| (0, x));
		/// let composed = Composed::new(l1, l2);
		/// let f = Forget::<RcBrand, String, (usize, i32), i32>::new(|(i, x)| format!("{}:{}", i, x));
		/// let folded = <Composed<
		/// 	'_,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	GetterPrime<RcBrand, (i32, String), i32>,
		/// 	IndexedGetterPrime<RcBrand, usize, i32, i32>,
		/// > as IndexedFoldOptic<usize, (i32, String), i32>>::evaluate::<String, RcBrand>(
		/// 	&composed,
		/// 	Indexed::new(f),
		/// );
		/// assert_eq!(folded.run((42, "hi".to_string())), "0:42");
		/// ```
		fn evaluate<R: 'a + Monoid + Clone + 'static, PointerBrand: ToDynCloneFn + 'static>(
			&self,
			pab: crate::types::optics::Indexed<'a, ForgetBrand<PointerBrand, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<PointerBrand, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			let pmn = self.second.evaluate::<R, PointerBrand>(pab);
			self.first.evaluate::<R, PointerBrand>(pmn)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the Setter brand.",
		"The index type.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, PointerBrand, I: 'a, S: 'a, T: 'a, M: 'a, N: 'a, A: 'a, B: 'a, O1, O2>
		IndexedSetterOptic<'a, PointerBrand, I, S, T, A, B> for Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		PointerBrand: ToDynCloneFn,
		O1: SetterOptic<'a, PointerBrand, S, T, M, N>,
		O2: IndexedSetterOptic<'a, PointerBrand, I, M, N, A, B>,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value to transform.")]
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
		/// let l1: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		/// let l2: IndexedLensPrime<RcBrand, usize, i32, i32> =
		/// 	IndexedLensPrime::from_iview_set(|x| (0, x), |(_, x)| x);
		/// let composed = Composed::new(l1, l2);
		/// let f = |(i, x): (usize, i32)| x + (i as i32);
		/// let indexed = Indexed::<RcFnBrand, usize, i32, i32>::new(std::rc::Rc::new(f));
		/// let modifier = <Composed<
		/// 	'_,
		/// 	(i32, String),
		/// 	(i32, String),
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	i32,
		/// 	LensPrime<RcBrand, (i32, String), i32>,
		/// 	IndexedLensPrime<RcBrand, usize, i32, i32>,
		/// > as IndexedSetterOptic<RcBrand, usize, (i32, String), (i32, String), i32, i32>>::evaluate(
		/// 	&composed, indexed,
		/// );
		/// assert_eq!(modifier((21, "hi".to_string())), (21, "hi".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: crate::types::optics::Indexed<'a, FnBrand<PointerBrand>, I, A, B>,
		) -> Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>)
		{
			let pmn = self.second.evaluate(pab);
			self.first.evaluate(pmn)
		}
	}
}
pub use inner::*;
