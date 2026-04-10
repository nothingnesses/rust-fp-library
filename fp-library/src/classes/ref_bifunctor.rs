//! Types that can be mapped over two type arguments simultaneously by reference.
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
//! let y = ref_bimap::<ResultBrand, _, _, _, _>(|e| *e + 1, |s| *s * 2, &x);
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

	/// A type class for types that can be mapped over two type arguments by reference.
	///
	/// This is the by-reference variant of [`Bifunctor`]. Both closures receive references
	/// to the values (`&A` and `&C`) and produce owned output (`B` and `D`). The container
	/// is borrowed, not consumed.
	///
	/// Unlike [`RefFunctor`] for partially-applied bifunctor brands (e.g.,
	/// `ResultErrAppliedBrand<E>`), `RefBifunctor` does not require `Clone` on either
	/// type parameter because both sides have closures to handle their respective types.
	///
	/// ### Laws
	///
	/// `RefBifunctor` instances must satisfy the following laws:
	///
	/// **Identity:** `ref_bimap(|x| x.clone(), |x| x.clone(), &p)` is equivalent to
	/// `p.clone()`, given `A: Clone, C: Clone`.
	///
	/// **Composition:** `ref_bimap(|x| f2(&f1(x)), |x| g2(&g1(x)), &p)` is equivalent to
	/// `ref_bimap(f2, g2, &ref_bimap(f1, g1, &p))`.
	#[document_examples]
	///
	/// RefBifunctor laws for [`Result`]:
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
	/// // Identity: ref_bimap(Clone::clone, Clone::clone, &p) == p.clone()
	/// assert_eq!(ref_bimap::<ResultBrand, _, _, _, _>(|x: &i32| *x, |x: &i32| *x, &ok), ok,);
	/// assert_eq!(ref_bimap::<ResultBrand, _, _, _, _>(|x: &i32| *x, |x: &i32| *x, &err), err,);
	///
	/// // Composition: ref_bimap(compose(f1, f2), compose(g1, g2), &p)
	/// //            = ref_bimap(f2, g2, &ref_bimap(f1, g1, &p))
	/// let f1 = |x: &i32| *x + 1;
	/// let f2 = |x: &i32| *x * 2;
	/// let g1 = |x: &i32| *x + 10;
	/// let g2 = |x: &i32| *x * 3;
	/// assert_eq!(
	/// 	ref_bimap::<ResultBrand, _, _, _, _>(|x: &i32| f2(&f1(x)), |x: &i32| g2(&g1(x)), &ok),
	/// 	ref_bimap::<ResultBrand, _, _, _, _>(
	/// 		f2,
	/// 		g2,
	/// 		&ref_bimap::<ResultBrand, _, _, _, _>(f1, g1, &ok),
	/// 	),
	/// );
	/// ```
	#[kind(type Of<'a, A: 'a, B: 'a>: 'a;)]
	pub trait RefBifunctor {
		/// Maps functions over the values in the bifunctor context by reference.
		///
		/// Both closures receive references to the values and produce owned output.
		/// The container is borrowed, not consumed.
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
			"The bifunctor instance (borrowed)."
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
		/// 	classes::*,
		/// };
		///
		/// let x: Result<i32, i32> = Ok(5);
		/// let y = ResultBrand::ref_bimap(|e: &i32| *e + 1, |s: &i32| *s * 2, &x);
		/// assert_eq!(y, Ok(10));
		/// ```
		fn ref_bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			f: impl Fn(&A) -> B + 'a,
			g: impl Fn(&C) -> D + 'a,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>);
	}

	/// Maps functions over the values in the bifunctor context by reference.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefBifunctor::ref_bimap`].
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
		"The bifunctor instance (borrowed)."
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
	/// let y = ref_bimap::<ResultBrand, _, _, _, _>(|e| *e + 1, |s| *s * 2, &x);
	/// assert_eq!(y, Ok(10));
	/// ```
	pub fn ref_bimap<'a, Brand: RefBifunctor, A: 'a, B: 'a, C: 'a, D: 'a>(
		f: impl Fn(&A) -> B + 'a,
		g: impl Fn(&C) -> D + 'a,
		p: &Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
	) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
		Brand::ref_bimap(f, g, p)
	}

	/// [`RefFunctor`] instance for [`BifunctorFirstAppliedBrand`].
	///
	/// Maps over the first type parameter of a bifunctor by reference, delegating to
	/// [`RefBifunctor::ref_bimap`] with [`Clone::clone`] for the second argument.
	/// Requires `Clone` on the fixed second type parameter because the value must be
	/// cloned out of the borrowed container.
	#[document_type_parameters("The bifunctor brand.", "The fixed second type parameter.")]
	impl<Brand: Bifunctor + RefBifunctor, A: Clone + 'static> RefFunctor
		for BifunctorFirstAppliedBrand<Brand, A>
	{
		/// Maps a function over the first type parameter by reference.
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
		/// let y = map::<BifunctorFirstAppliedBrand<ResultBrand, i32>, _, _, _, _>(|s: &i32| *s * 2, &x);
		/// assert_eq!(y, Ok(10));
		/// ```
		fn ref_map<'a, B: 'a, C: 'a>(
			func: impl Fn(&B) -> C + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::ref_bimap(|a: &A| a.clone(), func, fa)
		}
	}

	/// [`RefFunctor`] instance for [`BifunctorSecondAppliedBrand`].
	///
	/// Maps over the second type parameter of a bifunctor by reference, delegating to
	/// [`RefBifunctor::ref_bimap`] with [`Clone::clone`] for the first argument.
	/// Requires `Clone` on the fixed first type parameter because the value must be
	/// cloned out of the borrowed container.
	#[document_type_parameters("The bifunctor brand.", "The fixed first type parameter.")]
	impl<Brand: Bifunctor + RefBifunctor, B: Clone + 'static> RefFunctor
		for BifunctorSecondAppliedBrand<Brand, B>
	{
		/// Maps a function over the second type parameter by reference.
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
		/// let y = map::<BifunctorSecondAppliedBrand<ResultBrand, i32>, _, _, _, _>(|e: &i32| *e * 2, &x);
		/// assert_eq!(y, Err(10));
		/// ```
		fn ref_map<'a, A: 'a, C: 'a>(
			func: impl Fn(&A) -> C + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::ref_bimap(func, |b: &B| b.clone(), fa)
		}
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		crate::{
			brands::*,
			functions::*,
		},
		quickcheck_macros::quickcheck,
	};

	/// RefBifunctor identity law: ref_bimap(Clone::clone, Clone::clone, &p) == p.
	#[quickcheck]
	fn prop_ref_bifunctor_identity(
		a: i32,
		c: i32,
	) -> bool {
		let ok: Result<i32, i32> = Ok(c);
		let err: Result<i32, i32> = Err(a);
		ref_bimap::<ResultBrand, _, _, _, _>(|x: &i32| *x, |x: &i32| *x, &ok) == ok
			&& ref_bimap::<ResultBrand, _, _, _, _>(|x: &i32| *x, |x: &i32| *x, &err) == err
	}

	/// RefBifunctor composition law.
	#[quickcheck]
	fn prop_ref_bifunctor_composition(
		a: i32,
		c: i32,
	) -> bool {
		let f1 = |x: &i32| x.wrapping_add(1);
		let f2 = |x: &i32| x.wrapping_mul(2);
		let g1 = |x: &i32| x.wrapping_add(10);
		let g2 = |x: &i32| x.wrapping_mul(3);

		let ok: Result<i32, i32> = Ok(c);
		let err: Result<i32, i32> = Err(a);

		let composed_ok =
			ref_bimap::<ResultBrand, _, _, _, _>(|x: &i32| f2(&f1(x)), |x: &i32| g2(&g1(x)), &ok);
		let sequential_ok = ref_bimap::<ResultBrand, _, _, _, _>(
			f2,
			g2,
			&ref_bimap::<ResultBrand, _, _, _, _>(f1, g1, &ok),
		);

		let composed_err =
			ref_bimap::<ResultBrand, _, _, _, _>(|x: &i32| f2(&f1(x)), |x: &i32| g2(&g1(x)), &err);
		let sequential_err = ref_bimap::<ResultBrand, _, _, _, _>(
			f2,
			g2,
			&ref_bimap::<ResultBrand, _, _, _, _>(f1, g1, &err),
		);

		composed_ok == sequential_ok && composed_err == sequential_err
	}
}
