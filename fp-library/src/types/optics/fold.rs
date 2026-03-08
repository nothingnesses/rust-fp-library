//! Fold optics for collecting multiple values.
//!
//! A fold represents a way to focus on zero or more values in a structure.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::{
				UnsizedCoercible,
				monoid::Monoid,
				optics::*,
			},
			kinds::*,
			types::optics::{
				Forget,
				ForgetBrand,
			},
		},
		fp_macros::{
			document_examples,
			document_parameters,
			document_returns,
			document_type_parameters,
		},
		std::marker::PhantomData,
	};

	pub use crate::classes::optics::fold::FoldFunc;

	/// A wrapper that converts a function returning an iterable into a [`FoldFunc`].
	///
	/// `IterableFoldFn(f)` where `f: S -> impl IntoIterator<Item = A>` implements
	/// [`FoldFunc<S, A>`] by iterating over `f(s)` and folding with the monoid.
	/// This avoids allocating an intermediate `Vec<A>` when the source is already
	/// an iterable structure (e.g., a `Vec<A>`) or when a lazy iterator suffices.
	#[document_type_parameters("The type of the inner function.")]
	pub struct IterableFoldFn<F>(pub F);

	#[document_type_parameters("The type of the inner function.")]
	#[document_parameters("The fold instance.")]
	impl<F: Clone> Clone for IterableFoldFn<F> {
		#[document_signature]
		#[document_returns("A new `Fold` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::optics::IterableFoldFn;
		///
		/// let f = IterableFoldFn(|v: Vec<i32>| v);
		/// let cloned = f.clone();
		/// assert_eq!(cloned.0(vec![1, 2]), vec![1, 2]);
		/// ```
		fn clone(&self) -> Self {
			IterableFoldFn(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the function.",
		"The source type of the structure.",
		"The type of the focuses.",
		"The iterable type returned by the function.",
		"The type of the inner function."
	)]
	#[document_parameters("The fold instance.")]
	impl<'a, S, A, I, F> FoldFunc<'a, S, A> for IterableFoldFn<F>
	where
		F: Fn(S) -> I,
		I: IntoIterator<Item = A>,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type to fold into.", "The mapping function type.")]
		#[document_parameters("The mapping function.", "The structure to fold.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::monoid::Monoid,
		/// 	types::optics::{
		/// 		FoldFunc,
		/// 		IterableFoldFn,
		/// 	},
		/// };
		///
		/// let fold = IterableFoldFn(|v: Vec<i32>| v);
		/// let result = fold.apply::<String, _>(|x| x.to_string(), vec![1, 2, 3]);
		/// assert_eq!(result, "123".to_string());
		/// ```
		fn apply<R: Monoid, FArg: Fn(A) -> R + 'a>(
			&self,
			f: FArg,
			s: S,
		) -> R {
			(self.0)(s).into_iter().fold(R::empty(), |r, a| R::append(r, f(a)))
		}
	}

	/// A polymorphic fold.
	///
	/// Matches PureScript's `Fold r s t a b`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The type of the fold function."
	)]
	pub struct Fold<'a, P, S, T, A, B, F>
	where
		P: UnsizedCoercible,
		F: FoldFunc<'a, S, A>,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		/// The fold function.
		pub fold_fn: F,
		pub(crate) _phantom: PhantomData<(&'a (S, T, A, B), P)>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The type of the fold function."
	)]
	#[document_parameters("The fold instance.")]
	impl<'a, P, S, T, A, B, F> Clone for Fold<'a, P, S, T, A, B, F>
	where
		P: UnsizedCoercible,
		F: FoldFunc<'a, S, A> + Clone,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_returns("A new `Fold` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::{
		/// 		Fold,
		/// 		IterableFoldFn,
		/// 	},
		/// };
		///
		/// let f: Fold<RcBrand, Vec<i32>, Vec<i32>, i32, i32, _> =
		/// 	Fold::new(IterableFoldFn(|v: Vec<i32>| v));
		/// let cloned = f.clone();
		/// assert_eq!(cloned.to_vec(vec![1, 2]), vec![1, 2]);
		/// ```
		fn clone(&self) -> Self {
			Fold {
				fold_fn: self.fold_fn.clone(),
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
		"The target type of the focus.",
		"The type of the fold function."
	)]
	#[document_parameters("The fold instance.")]
	impl<'a, P, S, T, A, B, F> Fold<'a, P, S, T, A, B, F>
	where
		P: UnsizedCoercible,
		F: FoldFunc<'a, S, A>,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		/// Create a new polymorphic fold.
		#[document_signature]
		///
		#[document_parameters("The fold function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::{
		/// 		Fold,
		/// 		IterableFoldFn,
		/// 	},
		/// };
		///
		/// let f: Fold<RcBrand, Vec<i32>, Vec<i32>, i32, i32, _> =
		/// 	Fold::new(IterableFoldFn(|v: Vec<i32>| v));
		/// assert_eq!(f.to_vec(vec![1, 2]), vec![1, 2]);
		/// ```
		pub fn new(fold_fn: F) -> Self {
			Fold {
				fold_fn,
				_phantom: PhantomData,
			}
		}

		/// Collect all the focuses of the fold into a `Vec`.
		#[document_signature]
		///
		#[document_parameters("The structure to fold.")]
		///
		#[document_returns("A `Vec` containing all the focuses.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::{
		/// 		Fold,
		/// 		IterableFoldFn,
		/// 	},
		/// };
		///
		/// let f: Fold<RcBrand, Vec<i32>, Vec<i32>, i32, i32, _> =
		/// 	Fold::new(IterableFoldFn(|v: Vec<i32>| v));
		/// assert_eq!(f.to_vec(vec![1, 2, 3]), vec![1, 2, 3]);
		/// ```
		pub fn to_vec(
			&self,
			s: S,
		) -> Vec<A>
		where
			A: Clone, {
			self.fold_fn.apply::<Vec<A>, _>(|a| vec![a], s)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The type of the fold function."
	)]
	#[document_parameters("The fold instance.")]
	impl<'a, P, S, T, A, B, F> FoldOptic<'a, S, A> for Fold<'a, P, S, T, A, B, F>
	where
		P: UnsizedCoercible,
		F: FoldFunc<'a, S, A> + Clone + 'a,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_type_parameters(
			"The monoid type.",
			"The reference-counted pointer type for the Forget brand."
		)]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let f_optic: Fold<RcBrand, Vec<i32>, Vec<i32>, i32, i32, _> =
		/// 	Fold::new(IterableFoldFn(|v: Vec<i32>| v));
		/// let f = Forget::<RcBrand, String, i32, i32>::new(|x: i32| x.to_string());
		/// let folded: Forget<RcBrand, String, Vec<i32>, Vec<i32>> = FoldOptic::evaluate(&f_optic, f);
		/// assert_eq!(folded.run(vec![1, 2, 3]), "123".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>)
		{
			let fold_fn = self.fold_fn.clone();
			Forget::<Q, R, S, S>::new(move |s: S| {
				let pab_fn = pab.0.clone();
				fold_fn.apply::<R, _>(move |a| (pab_fn)(a), s)
			})
		}
	}

	/// A concrete fold type where types do not change.
	///
	/// Matches PureScript's `Fold' r s a`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus.",
		"The type of the fold function."
	)]
	pub struct FoldPrime<'a, P, S, A, F>
	where
		P: UnsizedCoercible,
		F: FoldFunc<'a, S, A>,
		S: 'a,
		A: 'a, {
		/// The fold function.
		pub fold_fn: F,
		pub(crate) _phantom: PhantomData<(&'a (S, A), P)>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus.",
		"The type of the fold function."
	)]
	#[document_parameters("The fold instance.")]
	impl<'a, P, S, A, F> Clone for FoldPrime<'a, P, S, A, F>
	where
		P: UnsizedCoercible,
		F: FoldFunc<'a, S, A> + Clone,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_returns("A new `FoldPrime` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::{
		/// 		FoldPrime,
		/// 		IterableFoldFn,
		/// 	},
		/// };
		/// let f: FoldPrime<RcBrand, Vec<i32>, i32, _> = FoldPrime::new(IterableFoldFn(|v: Vec<i32>| v));
		/// let cloned = f.clone();
		/// assert_eq!(cloned.to_vec(vec![1, 2]), vec![1, 2]);
		/// ```
		fn clone(&self) -> Self {
			FoldPrime {
				fold_fn: self.fold_fn.clone(),
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus.",
		"The type of the fold function."
	)]
	#[document_parameters("The fold instance.")]
	impl<'a, P, S, A, F> FoldPrime<'a, P, S, A, F>
	where
		P: UnsizedCoercible,
		F: FoldFunc<'a, S, A>,
		S: 'a,
		A: 'a,
	{
		/// Create a new monomorphic fold.
		#[document_signature]
		///
		#[document_parameters("The fold function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::{
		/// 		FoldPrime,
		/// 		IterableFoldFn,
		/// 	},
		/// };
		///
		/// let f: FoldPrime<RcBrand, Vec<i32>, i32, _> = FoldPrime::new(IterableFoldFn(|v: Vec<i32>| v));
		/// assert_eq!(f.to_vec(vec![1, 2, 3]), vec![1, 2, 3]);
		/// ```
		pub fn new(fold_fn: F) -> Self {
			FoldPrime {
				fold_fn,
				_phantom: PhantomData,
			}
		}

		/// Collect all the focuses of the fold into a `Vec`.
		#[document_signature]
		///
		#[document_parameters("The structure to fold.")]
		///
		#[document_returns("A `Vec` containing all the focuses.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::{
		/// 		FoldPrime,
		/// 		IterableFoldFn,
		/// 	},
		/// };
		///
		/// let f: FoldPrime<RcBrand, Vec<i32>, i32, _> = FoldPrime::new(IterableFoldFn(|v: Vec<i32>| v));
		/// assert_eq!(f.to_vec(vec![1, 2, 3]), vec![1, 2, 3]);
		/// ```
		pub fn to_vec(
			&self,
			s: S,
		) -> Vec<A>
		where
			A: Clone, {
			self.fold_fn.apply::<Vec<A>, _>(|a| vec![a], s)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus.",
		"The type of the fold function."
	)]
	#[document_parameters("The fold instance.")]
	impl<'a, P, S, A, F> FoldOptic<'a, S, A> for FoldPrime<'a, P, S, A, F>
	where
		P: UnsizedCoercible,
		F: FoldFunc<'a, S, A> + Clone + 'a,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_type_parameters(
			"The monoid type.",
			"The reference-counted pointer type for the Forget brand."
		)]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let f_optic: FoldPrime<RcBrand, Vec<i32>, i32, _> =
		/// 	FoldPrime::new(IterableFoldFn(|v: Vec<i32>| v));
		/// let f = Forget::<RcBrand, String, i32, i32>::new(|x: i32| x.to_string());
		/// let folded: Forget<RcBrand, String, Vec<i32>, Vec<i32>> = FoldOptic::evaluate(&f_optic, f);
		/// assert_eq!(folded.run(vec![1, 2, 3]), "123".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>)
		{
			let fold_fn = self.fold_fn.clone();
			Forget::<Q, R, S, S>::new(move |s: S| {
				let pab_fn = pab.0.clone();
				fold_fn.apply::<R, _>(move |a| (pab_fn)(a), s)
			})
		}
	}
}
pub use inner::*;
