//! By-reference variant of [`Bifoldable`](crate::classes::Bifoldable).
//!
//! **User story:** "I want to fold over a bifoldable value without consuming it."
//!
//! This trait is for types where the container holds cached or borrowed values
//! accessible by reference. The closures receive `&A` and `&B` instead of `A`
//! and `B`, avoiding unnecessary moves. `A: Clone` and `B: Clone` are required
//! so that default implementations can own values extracted from references.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x: Result<i32, i32> = Ok(5);
//! let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
//! 	(|e: &i32| e.to_string(), |s: &i32| s.to_string()),
//! 	&x,
//! );
//! assert_eq!(y, "5".to_string());
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
			types::Endofunction,
		},
		fp_macros::*,
	};

	/// By-reference folding over a bifoldable structure.
	///
	/// Similar to [`Bifoldable`], but closures receive `&A` and `&B` instead of
	/// owned values. This is the honest interface for types that internally hold
	/// cached or borrowed values and would otherwise force a clone to satisfy the
	/// by-value `Bifoldable` signature.
	///
	/// ### Minimal Implementation
	///
	/// A minimal implementation requires either [`RefBifoldable::ref_bi_fold_map`] or
	/// [`RefBifoldable::ref_bi_fold_right`] to be defined directly:
	///
	/// * If [`RefBifoldable::ref_bi_fold_right`] is implemented,
	///   [`RefBifoldable::ref_bi_fold_map`] and [`RefBifoldable::ref_bi_fold_left`]
	///   are derived from it.
	/// * If [`RefBifoldable::ref_bi_fold_map`] is implemented,
	///   [`RefBifoldable::ref_bi_fold_right`] is derived via `Endofunction`, and
	///   [`RefBifoldable::ref_bi_fold_left`] is derived from the derived
	///   [`RefBifoldable::ref_bi_fold_right`].
	///
	/// Note: defining both defaults creates a circular dependency and will not terminate.
	///
	/// ### Laws
	///
	/// `RefBifoldable` instances must be internally consistent and consistent with
	/// `Bifunctor` when one is also defined:
	/// * ref_bi_fold_map/ref_bi_fold_right consistency: `ref_bi_fold_map(f, g, &x) = ref_bi_fold_right(|a, c| append(f(a), c), |b, c| append(g(b), c), empty(), &x)`.
	#[document_examples]
	///
	/// RefBifoldable laws for [`Result`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // ResultBrand has Of<E, A> = Result<A, E>, so the first function handles errors
	/// // and the second function handles ok values.
	/// let ok: Result<i32, String> = Ok(5);
	/// let err: Result<i32, String> = Err("err".to_string());
	/// let f = |e: &String| format!("err:{e}");
	/// let g = |a: &i32| a.to_string();
	///
	/// // bi_fold_map/bi_fold_right consistency with ref closures (Ok case):
	/// assert_eq!(
	/// 	bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>((f, g), &ok),
	/// 	bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
	/// 		(|a: &String, c: String| append(f(a), c), |b: &i32, c: String| append(g(b), c)),
	/// 		empty::<String>(),
	/// 		&ok,
	/// 	),
	/// );
	///
	/// // bi_fold_map/bi_fold_right consistency with ref closures (Err case):
	/// assert_eq!(
	/// 	bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>((f, g), &err),
	/// 	bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
	/// 		(|a: &String, c: String| append(f(a), c), |b: &i32, c: String| append(g(b), c)),
	/// 		empty::<String>(),
	/// 		&err,
	/// 	),
	/// );
	/// ```
	#[kind(type Of<'a, A: 'a, B: 'a>: 'a;)]
	pub trait RefBifoldable {
		/// Folds the bifoldable structure from right to left by reference using two step functions.
		///
		/// This method performs a right-associative fold, dispatching to `f` for
		/// elements of the first type and `g` for elements of the second type.
		/// Closures receive references rather than owned values.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the first-position elements.",
			"The type of the second-position elements.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The step function for first-position element references.",
			"The step function for second-position element references.",
			"The initial accumulator value.",
			"The bifoldable structure to fold (borrowed)."
		)]
		///
		#[document_returns("The final accumulator value after folding all elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Result<i32, i32> = Err(3);
		/// let y = bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
		/// 	(|e: &i32, acc| acc - *e, |s: &i32, acc| acc + *s),
		/// 	10,
		/// 	&x,
		/// );
		/// assert_eq!(y, 7);
		/// ```
		fn ref_bi_fold_right<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(&A, C) -> C + 'a,
			g: impl Fn(&B, C) -> C + 'a,
			z: C,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			let f = <FnBrand as LiftFn>::new(move |(a, c): (A, C)| f(&a, c));
			let g = <FnBrand as LiftFn>::new(move |(b, c): (B, C)| g(&b, c));
			let endo = Self::ref_bi_fold_map::<FnBrand, A, B, Endofunction<'a, FnBrand, C>>(
				move |a: &A| {
					let a = a.clone();
					let f = f.clone();
					Endofunction::<FnBrand, C>::new(<FnBrand as LiftFn>::new(move |c| {
						f((a.clone(), c))
					}))
				},
				move |b: &B| {
					let b = b.clone();
					let g = g.clone();
					Endofunction::<FnBrand, C>::new(<FnBrand as LiftFn>::new(move |c| {
						g((b.clone(), c))
					}))
				},
				p,
			);
			endo.0(z)
		}

		/// Folds the bifoldable structure from left to right by reference using two step functions.
		///
		/// This method performs a left-associative fold, dispatching to `f` for
		/// elements of the first type and `g` for elements of the second type.
		/// Closures receive references rather than owned values.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the first-position elements.",
			"The type of the second-position elements.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The step function for first-position element references.",
			"The step function for second-position element references.",
			"The initial accumulator value.",
			"The bifoldable structure to fold (borrowed)."
		)]
		///
		#[document_returns("The final accumulator value after folding all elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Result<i32, i32> = Ok(5);
		/// let y = bi_fold_left::<RcFnBrand, ResultBrand, _, _, _, _, _>(
		/// 	(|acc, e: &i32| acc - *e, |acc, s: &i32| acc + *s),
		/// 	10,
		/// 	&x,
		/// );
		/// assert_eq!(y, 15);
		/// ```
		fn ref_bi_fold_left<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(C, &A) -> C + 'a,
			g: impl Fn(C, &B) -> C + 'a,
			z: C,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			let f = <FnBrand as LiftFn>::new(move |(c, a): (C, A)| f(c, &a));
			let g = <FnBrand as LiftFn>::new(move |(c, b): (C, B)| g(c, &b));
			let endo = Self::ref_bi_fold_right::<FnBrand, A, B, Endofunction<'a, FnBrand, C>>(
				move |a: &A, k: Endofunction<'a, FnBrand, C>| {
					let a = a.clone();
					let f = f.clone();
					let current =
						Endofunction::<FnBrand, C>::new(<FnBrand as LiftFn>::new(move |c| {
							f((c, a.clone()))
						}));
					Semigroup::append(k, current)
				},
				move |b: &B, k: Endofunction<'a, FnBrand, C>| {
					let b = b.clone();
					let g = g.clone();
					let current =
						Endofunction::<FnBrand, C>::new(<FnBrand as LiftFn>::new(move |c| {
							g((c, b.clone()))
						}));
					Semigroup::append(k, current)
				},
				Endofunction::<FnBrand, C>::empty(),
				p,
			);
			endo.0(z)
		}

		/// Maps elements of both types to a monoid by reference and combines the results.
		///
		/// This method maps each element of the first type using `f` and each element
		/// of the second type using `g`, receiving references rather than owned values,
		/// then combines all results using the monoid's `append` operation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the first-position elements.",
			"The type of the second-position elements.",
			"The monoid type to fold into."
		)]
		///
		#[document_parameters(
			"The function mapping first-position element references to the monoid.",
			"The function mapping second-position element references to the monoid.",
			"The bifoldable structure to fold (borrowed)."
		)]
		///
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Result<i32, i32> = Ok(5);
		/// let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
		/// 	(|e: &i32| e.to_string(), |s: &i32| s.to_string()),
		/// 	&x,
		/// );
		/// assert_eq!(y, "5".to_string());
		/// ```
		fn ref_bi_fold_map<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, M>(
			f: impl Fn(&A) -> M + 'a,
			g: impl Fn(&B) -> M + 'a,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> M
		where
			M: Monoid + 'a, {
			Self::ref_bi_fold_right::<FnBrand, A, B, M>(
				move |a, m| M::append(f(a), m),
				move |b, m| M::append(g(b), m),
				M::empty(),
				p,
			)
		}
	}

	/// Folds the bifoldable structure from right to left by reference using two step functions.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefBifoldable::ref_bi_fold_right`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the bifoldable structure.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The type of the accumulator."
	)]
	///
	#[document_parameters(
		"The step function for first-position element references.",
		"The step function for second-position element references.",
		"The initial accumulator value.",
		"The bifoldable structure to fold (borrowed)."
	)]
	///
	#[document_returns("The final accumulator value after folding all elements.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<i32, i32> = Err(3);
	/// let y = bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
	/// 	(|e: &i32, acc| acc - *e, |s: &i32, acc| acc + *s),
	/// 	10,
	/// 	&x,
	/// );
	/// assert_eq!(y, 7);
	/// ```
	pub fn ref_bi_fold_right<
		'a,
		FnBrand: LiftFn + 'a,
		Brand: RefBifoldable,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a,
	>(
		f: impl Fn(&A, C) -> C + 'a,
		g: impl Fn(&B, C) -> C + 'a,
		z: C,
		p: &Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
	) -> C {
		Brand::ref_bi_fold_right::<FnBrand, A, B, C>(f, g, z, p)
	}

	/// Folds the bifoldable structure from left to right by reference using two step functions.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefBifoldable::ref_bi_fold_left`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the bifoldable structure.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The type of the accumulator."
	)]
	///
	#[document_parameters(
		"The step function for first-position element references.",
		"The step function for second-position element references.",
		"The initial accumulator value.",
		"The bifoldable structure to fold (borrowed)."
	)]
	///
	#[document_returns("The final accumulator value after folding all elements.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<i32, i32> = Ok(5);
	/// let y = bi_fold_left::<RcFnBrand, ResultBrand, _, _, _, _, _>(
	/// 	(|acc, e: &i32| acc - *e, |acc, s: &i32| acc + *s),
	/// 	10,
	/// 	&x,
	/// );
	/// assert_eq!(y, 15);
	/// ```
	pub fn ref_bi_fold_left<
		'a,
		FnBrand: LiftFn + 'a,
		Brand: RefBifoldable,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a,
	>(
		f: impl Fn(C, &A) -> C + 'a,
		g: impl Fn(C, &B) -> C + 'a,
		z: C,
		p: &Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
	) -> C {
		Brand::ref_bi_fold_left::<FnBrand, A, B, C>(f, g, z, p)
	}

	/// Maps elements of both types to a monoid by reference and combines the results.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefBifoldable::ref_bi_fold_map`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the bifoldable structure.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The monoid type to fold into."
	)]
	///
	#[document_parameters(
		"The function mapping first-position element references to the monoid.",
		"The function mapping second-position element references to the monoid.",
		"The bifoldable structure to fold (borrowed)."
	)]
	///
	#[document_returns("The combined monoid value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<i32, i32> = Ok(5);
	/// let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
	/// 	(|e: &i32| e.to_string(), |s: &i32| s.to_string()),
	/// 	&x,
	/// );
	/// assert_eq!(y, "5".to_string());
	/// ```
	pub fn ref_bi_fold_map<
		'a,
		FnBrand: LiftFn + 'a,
		Brand: RefBifoldable,
		A: 'a + Clone,
		B: 'a + Clone,
		M,
	>(
		f: impl Fn(&A) -> M + 'a,
		g: impl Fn(&B) -> M + 'a,
		p: &Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
	) -> M
	where
		M: Monoid + 'a, {
		Brand::ref_bi_fold_map::<FnBrand, A, B, M>(f, g, p)
	}
}

pub use inner::*;
