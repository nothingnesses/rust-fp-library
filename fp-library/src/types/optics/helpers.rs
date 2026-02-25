//! Helper functions for working with optics.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{
				CloneableFn,
				Function,
				Profunctor,
				UnsizedCoercible,
				monoid::Monoid,
				semigroup::Semigroup,
			},
			kinds::*,
			types::optics::{
				Exchange,
				ExchangeBrand,
				FoldOptic,
				Forget,
				GetterOptic,
				IsoOptic,
				Optic,
				ReviewOptic,
				SetterOptic,
				Tagged,
			},
		},
		fp_macros::{
			document_parameters,
			document_signature,
			document_type_parameters,
		},
	};

	/// View the focus of a lens-like optic.
	///
	/// This is a convenience function that works with any lens-based optic.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	///
	#[document_parameters("The lens optic.", "The structure to view.")]
	///
	/// ### Returns
	///
	/// The focus value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::*,
	/// };
	///
	/// let l: LensPrime<RcBrand, (i32, String), i32> =
	/// 	LensPrime::new(|(x, _)| x, |(_, x)| (x, "".to_string()));
	/// assert_eq!(optics_view::<RcBrand, _, _, _>(&l, (42, "hello".to_string())), 42);
	/// ```
	pub fn optics_view<'a, P, O, S, A>(
		optic: &O,
		s: S,
	) -> A
	where
		P: UnsizedCoercible + 'static,
		O: GetterOptic<'a, S, A>,
		S: 'a,
		A: 'a + 'static, {
		(optic.evaluate::<A, P>(Forget::new(|a| a)).0)(s)
	}

	/// Set the focus of a lens-like optic.
	///
	/// This is a convenience function that works with any lens-based optic.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	///
	#[document_parameters("The lens optic.", "The structure to update.", "The new focus value.")]
	///
	/// ### Returns
	///
	/// The updated structure.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::*,
	/// };
	///
	/// let l: LensPrime<RcBrand, (i32, String), i32> =
	/// 	LensPrime::new(|(x, _)| x, |((_, s), x)| (x, s));
	/// assert_eq!(
	/// 	optics_set::<RcBrand, _, _, _>(&l, (42, "hello".to_string()), 99),
	/// 	(99, "hello".to_string())
	/// );
	/// ```
	pub fn optics_set<'a, Q, O, S, A>(
		optic: &O,
		s: S,
		a: A,
	) -> S
	where
		Q: UnsizedCoercible,
		O: SetterOptic<'a, Q, S, S, A, A>,
		S: 'a,
		A: 'a + Clone, {
		let f = <FnBrand<Q> as Function>::new(move |_| a.clone());
		(optic.evaluate(f))(s)
	}

	/// Modify the focus of a lens-like optic using a function.
	///
	/// This is a convenience function that works with any lens-based optic.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus.",
		"The type of the modification function."
	)]
	///
	#[document_parameters(
		"The lens optic.",
		"The structure to update.",
		"The function to apply to the focus."
	)]
	///
	/// ### Returns
	///
	/// The updated structure.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::*,
	/// };
	///
	/// let l: LensPrime<RcBrand, (i32, String), i32> =
	/// 	LensPrime::new(|(x, _)| x, |((_, s), x)| (x, s));
	/// assert_eq!(
	/// 	optics_over::<RcBrand, _, _, _, _>(&l, (42, "hello".to_string()), |x| x * 2),
	/// 	(84, "hello".to_string())
	/// );
	/// ```
	pub fn optics_over<'a, Q, O, S, A, F>(
		optic: &O,
		s: S,
		f: F,
	) -> S
	where
		Q: UnsizedCoercible,
		O: SetterOptic<'a, Q, S, S, A, A>,
		S: 'a,
		A: 'a,
		F: Fn(A) -> A + 'a, {
		let f = <FnBrand<Q> as Function>::new(f);
		(optic.evaluate(f))(s)
	}

	/// Preview the focus of a prism-like optic.
	///
	/// This is a convenience function that works with any prism-based optic.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	///
	#[document_parameters("The prism optic.", "The structure to preview.")]
	///
	/// ### Returns
	///
	/// An `Option` containing the focus value if it exists.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::*,
	/// };
	///
	/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
	/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
	/// assert_eq!(optics_preview::<RcBrand, _, _, _>(&ok_prism, Ok(42)), Some(42));
	/// assert_eq!(optics_preview::<RcBrand, _, _, _>(&ok_prism, Err("error".to_string())), None);
	/// ```
	pub fn optics_preview<'a, P, O, S, A>(
		optic: &O,
		s: S,
	) -> Option<A>
	where
		P: UnsizedCoercible + 'static,
		O: FoldOptic<'a, S, A>,
		S: 'a,
		A: 'a + 'static, {
		#[derive(Clone)]
		struct First<A>(Option<A>);
		impl<A> Semigroup for First<A> {
			fn append(
				a: Self,
				b: Self,
			) -> Self {
				First(a.0.or(b.0))
			}
		}
		impl<A> Monoid for First<A> {
			fn empty() -> Self {
				First(None)
			}
		}

		let forget = Forget::new(|a| First(Some(a)));
		let result_forget = optic.evaluate::<First<A>, P>(forget);
		(result_forget.0)(s).0
	}

	/// Review a focus value into a structure using a prism-like optic.
	///
	/// This is a convenience function that works with any prism-based optic.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	///
	#[document_parameters("The prism optic.", "The focus value.")]
	///
	/// ### Returns
	///
	/// The structure containing the focus value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::*,
	/// };
	///
	/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
	/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
	/// assert_eq!(optics_review(&ok_prism, 42), Ok(42));
	/// ```
	pub fn optics_review<'a, O, S, A>(
		optic: &O,
		a: A,
	) -> S
	where
		O: ReviewOptic<'a, S, S, A, A>,
		S: 'a,
		A: 'a, {
		(optic.evaluate(Tagged::new(a))).0
	}

	/// Apply an isomorphism in the forward direction.
	///
	/// This is a convenience function that converts from structure to focus.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	///
	#[document_parameters("The iso optic.", "The structure to convert.")]
	///
	/// ### Returns
	///
	/// The focus value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		RcFnBrand,
	/// 	},
	/// 	types::optics::*,
	/// };
	///
	/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
	/// assert_eq!(optics_from::<RcFnBrand, _, _, _>(&iso, (42,)), 42);
	/// ```
	pub fn optics_from<'a, P, O, S, A>(
		optic: &O,
		s: S,
	) -> A
	where
		P: CloneableFn + 'static,
		O: IsoOptic<'a, S, S, A, A>,
		S: 'a,
		A: 'a + 'static, {
		let exchange =
			Exchange::new(<P as CloneableFn>::new(|a| a), <P as CloneableFn>::new(|a| a));
		(optic.evaluate::<ExchangeBrand<P, A, A>>(exchange).get)(s)
	}

	/// Apply an isomorphism in the backward direction.
	///
	/// This is a convenience function that converts from focus to structure.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	///
	#[document_parameters("The iso optic.", "The focus value to convert.")]
	///
	/// ### Returns
	///
	/// The structure.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		RcFnBrand,
	/// 	},
	/// 	types::optics::*,
	/// };
	///
	/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
	/// assert_eq!(optics_to::<RcFnBrand, _, _, _>(&iso, 42), (42,));
	/// ```
	pub fn optics_to<'a, P, O, S, A>(
		optic: &O,
		a: A,
	) -> S
	where
		P: CloneableFn + 'static,
		O: IsoOptic<'a, S, S, A, A>,
		S: 'a,
		A: 'a + 'static, {
		let exchange =
			Exchange::new(<P as CloneableFn>::new(|a| a), <P as CloneableFn>::new(|a| a));
		(optic.evaluate::<ExchangeBrand<P, A, A>>(exchange).set)(a)
	}

	/// Evaluate an optic with a profunctor.
	///
	/// This is the most general function for working with optics, allowing you to
	/// evaluate any optic with any compatible profunctor.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The optic type.",
		"The type of the structure.",
		"The target type after update.",
		"The type of the focus.",
		"The target focus type after update."
	)]
	///
	#[document_parameters("The optic.", "The profunctor value.")]
	///
	/// ### Returns
	///
	/// The transformed profunctor value.
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
	/// 	LensPrime::new(|(x, _)| x, |((_, s), x)| (x, s));
	///
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// let modifier = optics_eval::<RcFnBrand, _, _, _, _, _>(&l, f);
	/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
	/// ```
	pub fn optics_eval<'a, P, O, S: 'a, T: 'a, A: 'a, B: 'a>(
		optic: &O,
		pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
	) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>)
	where
		P: Profunctor,
		O: Optic<'a, P, S, T, A, B>, {
		optic.evaluate(pab)
	}
}
pub use inner::*;
