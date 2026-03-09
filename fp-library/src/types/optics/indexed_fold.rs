//! Indexed fold optics.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::optics::*,
			classes::{
				UnsizedCoercible,
				foldable_with_index::FoldableWithIndex,
				monoid::Monoid,
				optics::*,
			},
			kinds::*,
			types::optics::{
				Forget,
				Indexed,
			},
		},
		fp_macros::*,
		std::marker::PhantomData,
	};

	/// A trait for indexed fold functions.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The index type.",
		"The source type.",
		"The element type."
	)]
	#[document_parameters("The indexed fold instance.")]
	pub trait IndexedFoldFunc<'a, I, S, A> {
		/// Apply the indexed fold function.
		#[document_signature]
		#[document_type_parameters("The monoid type.")]
		#[document_parameters("The fold function.", "The source value.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::{
		/// 		monoid::Monoid,
		/// 		semigroup::Semigroup,
		/// 	},
		/// 	types::optics::IndexedFoldFunc,
		/// };
		/// #[derive(Clone)]
		/// struct MyFold;
		/// impl<'a> IndexedFoldFunc<'a, usize, Vec<i32>, i32> for MyFold {
		/// 	fn apply<R: 'a + Monoid + 'static>(
		/// 		&self,
		/// 		f: Box<dyn Fn(usize, i32) -> R + 'a>,
		/// 		s: Vec<i32>,
		/// 	) -> R {
		/// 		s.into_iter()
		/// 			.enumerate()
		/// 			.fold(R::empty(), |acc, (i, x)| Semigroup::append(acc, f(i, x)))
		/// 	}
		/// }
		/// let fold = MyFold;
		/// let result: String = fold.apply(Box::new(|i, x| format!("[{}]={}", i, x)), vec![10, 20]);
		/// assert_eq!(result, "[0]=10[1]=20");
		/// ```
		fn apply<R: 'a + Monoid + 'static>(
			&self,
			f: Box<dyn Fn(I, A) -> R + 'a>,
			s: S,
		) -> R;
	}

	/// A polymorphic indexed fold.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The fold function type."
	)]
	pub struct IndexedFold<'a, PointerBrand, I, S, T, A, B, F>
	where
		F: IndexedFoldFunc<'a, I, S, A> + 'a, {
		/// The fold function.
		pub fold_fn: F,
		pub(crate) _phantom: PhantomData<(&'a (I, S, T, A, B), PointerBrand)>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The fold function type."
	)]
	#[document_parameters("The indexed fold instance.")]
	impl<'a, PointerBrand, I, S, T, A, B, F> Clone for IndexedFold<'a, PointerBrand, I, S, T, A, B, F>
	where
		F: IndexedFoldFunc<'a, I, S, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_returns("A new `IndexedFold` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::{
		/// 		IndexedFoldOptic,
		/// 		UnsizedCoercible,
		/// 		monoid::Monoid,
		/// 		semigroup::Semigroup,
		/// 	},
		/// 	types::optics::*,
		/// };
		/// #[derive(Clone)]
		/// struct MyFold;
		/// impl<'a> IndexedFoldFunc<'a, usize, Vec<i32>, i32> for MyFold {
		/// 	fn apply<R: 'a + Monoid + 'static>(
		/// 		&self,
		/// 		f: Box<dyn Fn(usize, i32) -> R + 'a>,
		/// 		s: Vec<i32>,
		/// 	) -> R {
		/// 		s.into_iter()
		/// 			.enumerate()
		/// 			.fold(R::empty(), |acc, (i, x)| Semigroup::append(acc, f(i, x)))
		/// 	}
		/// }
		/// let l: IndexedFold<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, MyFold> =
		/// 	IndexedFold::new(MyFold);
		/// let cloned = l.clone();
		/// let f = Forget::<RcBrand, String, (usize, i32), i32>::new(|(i, x)| format!("[{}]={}", i, x));
		/// let pab = Indexed::new(f);
		/// let result = IndexedFoldOptic::evaluate::<String, RcBrand>(&cloned, pab);
		/// assert_eq!(result.run(vec![10, 20]), "[0]=10[1]=20");
		/// ```
		fn clone(&self) -> Self {
			IndexedFold {
				fold_fn: self.fold_fn.clone(),
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The fold function type."
	)]
	impl<'a, PointerBrand, I, S, T, A, B, F> IndexedFold<'a, PointerBrand, I, S, T, A, B, F>
	where
		F: IndexedFoldFunc<'a, I, S, A> + 'a,
	{
		/// Create a new indexed fold.
		#[document_signature]
		#[document_parameters("The fold function.")]
		#[document_returns("A new `IndexedFold` instance.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::{
		/// 		IndexedFoldOptic,
		/// 		UnsizedCoercible,
		/// 		monoid::Monoid,
		/// 		semigroup::Semigroup,
		/// 	},
		/// 	types::optics::*,
		/// };
		/// #[derive(Clone)]
		/// struct MyFold;
		/// impl<'a> IndexedFoldFunc<'a, usize, Vec<i32>, i32> for MyFold {
		/// 	fn apply<R: 'a + Monoid + 'static>(
		/// 		&self,
		/// 		f: Box<dyn Fn(usize, i32) -> R + 'a>,
		/// 		s: Vec<i32>,
		/// 	) -> R {
		/// 		s.into_iter()
		/// 			.enumerate()
		/// 			.fold(R::empty(), |acc, (i, x)| Semigroup::append(acc, f(i, x)))
		/// 	}
		/// }
		/// let l: IndexedFold<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, MyFold> =
		/// 	IndexedFold::new(MyFold);
		/// let f = Forget::<RcBrand, String, (usize, i32), i32>::new(|(i, x)| format!("[{}]={}", i, x));
		/// let pab = Indexed::new(f);
		/// let result = IndexedFoldOptic::evaluate::<String, RcBrand>(&l, pab);
		/// assert_eq!(result.run(vec![10, 20]), "[0]=10[1]=20");
		/// ```
		pub fn new(fold_fn: F) -> Self {
			IndexedFold {
				fold_fn,
				_phantom: PhantomData,
			}
		}
	}

	/// A wrapper struct for the `folded` constructor.
	#[derive(Clone)]
	pub struct Folded<Brand>(pub std::marker::PhantomData<Brand>);

	#[document_type_parameters(
		"The lifetime of the values.",
		"The index type.",
		"The brand of the foldable structure.",
		"The type of the elements in the structure."
	)]
	#[document_parameters("The folded struct.")]
	impl<'a, I, Brand, A>
		IndexedFoldFunc<'a, I, Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>), A>
		for Folded<Brand>
	where
		Brand: FoldableWithIndex<I>,
		A: 'a,
		I: 'a,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type.")]
		#[document_parameters("The fold function.", "The structure to fold.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	types::optics::{
		/// 		Folded,
		/// 		IndexedFoldFunc,
		/// 	},
		/// };
		/// let folded = Folded::<VecBrand>(std::marker::PhantomData);
		/// let result: String =
		/// 	folded.apply(Box::new(|i: usize, x: i32| format!("[{}]={}", i, x)), vec![10, 20, 30]);
		/// assert_eq!(result, "[0]=10[1]=20[2]=30");
		/// ```
		fn apply<R: 'a + Monoid + 'static>(
			&self,
			f: Box<dyn Fn(I, A) -> R + 'a>,
			s: Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>),
		) -> R {
			Brand::fold_map_with_index(f, s)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The brand of the foldable structure.",
		"The type of the elements in the structure."
	)]
	impl<'a, PointerBrand, I, Brand, A>
		IndexedFold<
			'a,
			PointerBrand,
			I,
			Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>),
			A,
			A,
			Folded<Brand>,
		>
	where
		Brand: FoldableWithIndex<I>,
		A: 'a,
		I: 'a,
	{
		/// Create an indexed fold from a `FoldableWithIndex`.
		#[document_signature]
		#[document_returns("A new `IndexedFold` instance.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	functions::optics_indexed_fold_map,
		/// 	types::optics::{
		/// 		Folded,
		/// 		IndexedFold,
		/// 	},
		/// };
		/// let l: IndexedFold<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, Folded<VecBrand>> =
		/// 	IndexedFold::folded();
		/// let v = vec![10, 20];
		/// let s = optics_indexed_fold_map::<RcBrand, _, _, _, _, String, _>(
		/// 	&l,
		/// 	|i, x| format!("{}:{}", i, x),
		/// 	v,
		/// );
		/// assert_eq!(s, "0:101:20".to_string());
		/// ```
		pub fn folded() -> Self {
			IndexedFold::new(Folded(std::marker::PhantomData))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The brand of the foldable structure.",
		"The type of the elements in the structure."
	)]
	impl<'a, PointerBrand, I, Brand, A>
		IndexedFoldPrime<
			'a,
			PointerBrand,
			I,
			Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>),
			A,
			Folded<Brand>,
		>
	where
		Brand: FoldableWithIndex<I>,
		A: 'a,
		I: 'a,
	{
		/// Create a monomorphic indexed fold from a `FoldableWithIndex`.
		#[document_signature]
		#[document_returns("A new `IndexedFoldPrime` instance.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	functions::optics_indexed_fold_map,
		/// 	types::optics::{
		/// 		Folded,
		/// 		IndexedFoldPrime,
		/// 	},
		/// };
		/// let l: IndexedFoldPrime<RcBrand, usize, Vec<i32>, i32, Folded<VecBrand>> =
		/// 	IndexedFoldPrime::folded();
		/// let v = vec![10, 20];
		/// let s = optics_indexed_fold_map::<RcBrand, _, _, _, _, String, _>(
		/// 	&l,
		/// 	|i, x| format!("{}:{}", i, x),
		/// 	v,
		/// );
		/// assert_eq!(s, "0:101:20".to_string());
		/// ```
		pub fn folded() -> Self {
			IndexedFoldPrime::new(Folded(std::marker::PhantomData))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The fold function type."
	)]
	#[document_parameters("The indexed fold instance.")]
	impl<'a, PointerBrand, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a, F> IndexedFoldOptic<'a, I, S, A>
		for IndexedFold<'a, PointerBrand, I, S, T, A, B, F>
	where
		F: IndexedFoldFunc<'a, I, S, A> + Clone + 'a,
		PointerBrand: UnsizedCoercible,
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
		/// 	classes::{
		/// 		IndexedFoldOptic,
		/// 		UnsizedCoercible,
		/// 		monoid::Monoid,
		/// 		semigroup::Semigroup,
		/// 	},
		/// 	types::optics::*,
		/// };
		/// #[derive(Clone)]
		/// struct MyFold;
		/// impl<'a> IndexedFoldFunc<'a, usize, Vec<i32>, i32> for MyFold {
		/// 	fn apply<R: 'a + Monoid + 'static>(
		/// 		&self,
		/// 		f: Box<dyn Fn(usize, i32) -> R + 'a>,
		/// 		s: Vec<i32>,
		/// 	) -> R {
		/// 		s.into_iter()
		/// 			.enumerate()
		/// 			.fold(R::empty(), |acc, (i, x)| Semigroup::append(acc, f(i, x)))
		/// 	}
		/// }
		/// let l: IndexedFold<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, MyFold> =
		/// 	IndexedFold::new(MyFold);
		/// let f = Forget::<RcBrand, String, (usize, i32), i32>::new(|(i, x)| format!("[{}]={}", i, x));
		/// let pab = Indexed::new(f);
		/// let result = IndexedFoldOptic::evaluate::<String, RcBrand>(&l, pab);
		/// assert_eq!(result.run(vec![10, 20]), "[0]=10[1]=20");
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Indexed<'a, ForgetBrand<Q, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>)
		{
			let fold_fn = self.fold_fn.clone();
			crate::types::optics::Forget::<Q, R, S, S>::new(move |s: S| {
				let pab_fn = pab.inner.0.clone();
				fold_fn.apply::<R>(Box::new(move |i, a| (pab_fn)((i, a))), s)
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The result type brand.",
		"The monoid type.",
		"The original profunctor type.",
		"The index type.",
		"The source type.",
		"The target type.",
		"The source focus type.",
		"The target focus type.",
		"The fold function type."
	)]
	#[document_parameters("The indexed fold instance.")]
	impl<
		'a,
		Q2: UnsizedCoercible + 'static,
		R: 'a + Monoid + Clone + 'static,
		PointerBrand: UnsizedCoercible,
		I: 'a,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
		F,
	> IndexedOpticAdapter<'a, ForgetBrand<Q2, R>, I, S, S, A, A>
		for IndexedFold<'a, PointerBrand, I, S, T, A, B, F>
	where
		F: IndexedFoldFunc<'a, I, S, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::{
		/// 		Folded,
		/// 		*,
		/// 	},
		/// };
		/// let l = IndexedFold::<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, Folded<VecBrand>>::folded();
		/// let _unindexed = optics_un_index::<ForgetBrand<RcBrand, String>, _, _, _, _, _, _>(&l);
		/// assert_eq!(
		/// 	optics_indexed_fold_map::<RcBrand, _, _, _, _, String, _>(
		/// 		&l,
		/// 		|_, x| x.to_string(),
		/// 		vec![1, 2]
		/// 	),
		/// 	"12"
		/// );
		/// ```
		fn evaluate_indexed(
			&self,
			pab: Indexed<'a, ForgetBrand<Q2, R>, I, A, A>,
		) -> Forget<'a, Q2, R, S, S> {
			IndexedFoldOptic::evaluate::<R, Q2>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The result type brand.",
		"The monoid type.",
		"The original profunctor type.",
		"The index type.",
		"The source type.",
		"The target type.",
		"The source focus type.",
		"The target focus type.",
		"The fold function type."
	)]
	#[document_parameters("The indexed fold instance.")]
	impl<
		'a,
		Q2: UnsizedCoercible + 'static,
		R: 'a + Monoid + Clone + 'static,
		PointerBrand: UnsizedCoercible,
		I: 'a,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
		F,
	> IndexedOpticAdapterDiscardsFocus<'a, ForgetBrand<Q2, R>, I, S, S, A, A>
		for IndexedFold<'a, PointerBrand, I, S, T, A, B, F>
	where
		F: IndexedFoldFunc<'a, I, S, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::{
		/// 		Folded,
		/// 		*,
		/// 	},
		/// };
		/// let l = IndexedFold::<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, Folded<VecBrand>>::folded();
		/// let _unindexed = optics_as_index::<ForgetBrand<RcBrand, String>, _, _, _, _, _, _>(&l);
		/// assert_eq!(
		/// 	optics_indexed_fold_map::<RcBrand, _, _, _, _, String, _>(
		/// 		&l,
		/// 		|i, _| i.to_string(),
		/// 		vec![1, 2]
		/// 	),
		/// 	"01"
		/// );
		/// ```
		fn evaluate_indexed_discards_focus(
			&self,
			pab: Indexed<'a, ForgetBrand<Q2, R>, I, A, A>,
		) -> Forget<'a, Q2, R, S, S> {
			IndexedFoldOptic::evaluate::<R, Q2>(self, pab)
		}
	}

	/// A monomorphic indexed fold.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The fold function type."
	)]
	pub struct IndexedFoldPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedFoldFunc<'a, I, S, A> + 'a, {
		/// The fold function.
		pub fold_fn: F,
		pub(crate) _phantom: PhantomData<(&'a (I, S, A), PointerBrand)>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The fold function type."
	)]
	#[document_parameters("The indexed fold instance.")]
	impl<'a, PointerBrand, I, S, A, F> Clone for IndexedFoldPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedFoldFunc<'a, I, S, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_returns("A new `IndexedFoldPrime` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::{
		/// 		IndexedFoldOptic,
		/// 		UnsizedCoercible,
		/// 		monoid::Monoid,
		/// 		semigroup::Semigroup,
		/// 	},
		/// 	types::optics::*,
		/// };
		/// #[derive(Clone)]
		/// struct MyFold;
		/// impl<'a> IndexedFoldFunc<'a, usize, Vec<i32>, i32> for MyFold {
		/// 	fn apply<R: 'a + Monoid + 'static>(
		/// 		&self,
		/// 		f: Box<dyn Fn(usize, i32) -> R + 'a>,
		/// 		s: Vec<i32>,
		/// 	) -> R {
		/// 		s.into_iter()
		/// 			.enumerate()
		/// 			.fold(R::empty(), |acc, (i, x)| Semigroup::append(acc, f(i, x)))
		/// 	}
		/// }
		/// let l: IndexedFoldPrime<RcBrand, usize, Vec<i32>, i32, MyFold> = IndexedFoldPrime::new(MyFold);
		/// let cloned = l.clone();
		/// let f = Forget::<RcBrand, String, (usize, i32), i32>::new(|(i, x)| format!("[{}]={}", i, x));
		/// let pab = Indexed::new(f);
		/// let result = IndexedFoldOptic::evaluate::<String, RcBrand>(&cloned, pab);
		/// assert_eq!(result.run(vec![10, 20]), "[0]=10[1]=20");
		/// ```
		fn clone(&self) -> Self {
			IndexedFoldPrime {
				fold_fn: self.fold_fn.clone(),
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The fold function type."
	)]
	impl<'a, PointerBrand, I, S, A, F> IndexedFoldPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedFoldFunc<'a, I, S, A> + 'a,
	{
		/// Create a new monomorphic indexed fold.
		#[document_signature]
		#[document_parameters("The fold function.")]
		#[document_returns("A new `IndexedFoldPrime` instance.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::{
		/// 		IndexedFoldOptic,
		/// 		UnsizedCoercible,
		/// 		monoid::Monoid,
		/// 		semigroup::Semigroup,
		/// 	},
		/// 	types::optics::*,
		/// };
		/// #[derive(Clone)]
		/// struct MyFold;
		/// impl<'a> IndexedFoldFunc<'a, usize, Vec<i32>, i32> for MyFold {
		/// 	fn apply<R: 'a + Monoid + 'static>(
		/// 		&self,
		/// 		f: Box<dyn Fn(usize, i32) -> R + 'a>,
		/// 		s: Vec<i32>,
		/// 	) -> R {
		/// 		s.into_iter()
		/// 			.enumerate()
		/// 			.fold(R::empty(), |acc, (i, x)| Semigroup::append(acc, f(i, x)))
		/// 	}
		/// }
		/// let l: IndexedFoldPrime<RcBrand, usize, Vec<i32>, i32, MyFold> = IndexedFoldPrime::new(MyFold);
		/// let f = Forget::<RcBrand, String, (usize, i32), i32>::new(|(i, x)| format!("[{}]={}", i, x));
		/// let pab = Indexed::new(f);
		/// let result = IndexedFoldOptic::evaluate::<String, RcBrand>(&l, pab);
		/// assert_eq!(result.run(vec![10, 20]), "[0]=10[1]=20");
		/// ```
		pub fn new(fold_fn: F) -> Self {
			IndexedFoldPrime {
				fold_fn,
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The fold function type."
	)]
	#[document_parameters("The indexed fold instance.")]
	impl<'a, PointerBrand, I: 'a, S: 'a, A: 'a, F> IndexedFoldOptic<'a, I, S, A>
		for IndexedFoldPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedFoldFunc<'a, I, S, A> + Clone + 'a,
		PointerBrand: UnsizedCoercible,
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
		/// 	classes::{
		/// 		IndexedFoldOptic,
		/// 		UnsizedCoercible,
		/// 		monoid::Monoid,
		/// 		semigroup::Semigroup,
		/// 	},
		/// 	types::optics::*,
		/// };
		/// #[derive(Clone)]
		/// struct MyFold;
		/// impl<'a> IndexedFoldFunc<'a, usize, Vec<i32>, i32> for MyFold {
		/// 	fn apply<R: 'a + Monoid + 'static>(
		/// 		&self,
		/// 		f: Box<dyn Fn(usize, i32) -> R + 'a>,
		/// 		s: Vec<i32>,
		/// 	) -> R {
		/// 		s.into_iter()
		/// 			.enumerate()
		/// 			.fold(R::empty(), |acc, (i, x)| Semigroup::append(acc, f(i, x)))
		/// 	}
		/// }
		/// let l: IndexedFoldPrime<RcBrand, usize, Vec<i32>, i32, MyFold> = IndexedFoldPrime::new(MyFold);
		/// let f = Forget::<RcBrand, String, (usize, i32), i32>::new(|(i, x)| format!("[{}]={}", i, x));
		/// let pab = Indexed::new(f);
		/// let result = IndexedFoldOptic::evaluate::<String, RcBrand>(&l, pab);
		/// assert_eq!(result.run(vec![10, 20]), "[0]=10[1]=20");
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Indexed<'a, ForgetBrand<Q, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>)
		{
			let fold_fn = self.fold_fn.clone();
			crate::types::optics::Forget::<Q, R, S, S>::new(move |s: S| {
				let pab_fn = pab.inner.0.clone();
				fold_fn.apply::<R>(Box::new(move |i, a| (pab_fn)((i, a))), s)
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The result type brand.",
		"The monoid type.",
		"The original profunctor type.",
		"The index type.",
		"The structure type.",
		"The focus type.",
		"The fold function type."
	)]
	#[document_parameters("The indexed fold instance.")]
	impl<
		'a,
		Q2: UnsizedCoercible + 'static,
		R: 'a + Monoid + Clone + 'static,
		PointerBrand: UnsizedCoercible,
		I: 'a,
		S: 'a,
		A: 'a,
		F,
	> IndexedOpticAdapter<'a, ForgetBrand<Q2, R>, I, S, S, A, A>
		for IndexedFoldPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedFoldFunc<'a, I, S, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::{
		/// 		Folded,
		/// 		*,
		/// 	},
		/// };
		/// let l = IndexedFoldPrime::<RcBrand, usize, Vec<i32>, i32, Folded<VecBrand>>::folded();
		/// let _unindexed = optics_un_index::<ForgetBrand<RcBrand, String>, _, _, _, _, _, _>(&l);
		/// assert_eq!(
		/// 	optics_indexed_fold_map::<RcBrand, _, _, _, _, String, _>(
		/// 		&l,
		/// 		|_, x| x.to_string(),
		/// 		vec![1, 2]
		/// 	),
		/// 	"12"
		/// );
		/// ```
		fn evaluate_indexed(
			&self,
			pab: Indexed<'a, ForgetBrand<Q2, R>, I, A, A>,
		) -> Forget<'a, Q2, R, S, S> {
			IndexedFoldOptic::evaluate::<R, Q2>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The result type brand.",
		"The monoid type.",
		"The original profunctor type.",
		"The index type.",
		"The structure type.",
		"The focus type.",
		"The fold function type."
	)]
	#[document_parameters("The indexed fold instance.")]
	impl<
		'a,
		Q2: UnsizedCoercible + 'static,
		R: 'a + Monoid + Clone + 'static,
		PointerBrand: UnsizedCoercible,
		I: 'a,
		S: 'a,
		A: 'a,
		F,
	> IndexedOpticAdapterDiscardsFocus<'a, ForgetBrand<Q2, R>, I, S, S, A, A>
		for IndexedFoldPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedFoldFunc<'a, I, S, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::{
		/// 		Folded,
		/// 		*,
		/// 	},
		/// };
		/// let l = IndexedFoldPrime::<RcBrand, usize, Vec<i32>, i32, Folded<VecBrand>>::folded();
		/// let _unindexed = optics_as_index::<ForgetBrand<RcBrand, String>, _, _, _, _, _, _>(&l);
		/// assert_eq!(
		/// 	optics_indexed_fold_map::<RcBrand, _, _, _, _, String, _>(
		/// 		&l,
		/// 		|i, _| i.to_string(),
		/// 		vec![1, 2]
		/// 	),
		/// 	"01"
		/// );
		/// ```
		fn evaluate_indexed_discards_focus(
			&self,
			pab: Indexed<'a, ForgetBrand<Q2, R>, I, A, A>,
		) -> Forget<'a, Q2, R, S, S> {
			IndexedFoldOptic::evaluate::<R, Q2>(self, pab)
		}
	}
}

pub use inner::*;
