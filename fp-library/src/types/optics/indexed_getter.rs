//! Indexed getter optics.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				FnBrand,
				optics::*,
			},
			classes::{
				optics::*,
				*,
			},
			kinds::*,
			types::optics::{
				Forget,
				Indexed,
			},
		},
		fp_macros::*,
	};

	/// A polymorphic indexed getter.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The focus type."
	)]
	pub struct IndexedGetter<'a, PointerBrand, I, S, A>
	where
		PointerBrand: UnsizedCoercible,
		I: 'a,
		S: 'a,
		A: 'a, {
		/// Internal storage: S -> (I, A)
		pub(crate) to: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, (I, A)>),
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The focus type."
	)]
	#[document_parameters("The indexed getter instance.")]
	impl<'a, PointerBrand, I, S, A> Clone for IndexedGetter<'a, PointerBrand, I, S, A>
	where
		PointerBrand: UnsizedCoercible,
		I: 'a,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_returns("A new `IndexedGetter` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::IndexedGetter,
		/// };
		/// let g: IndexedGetter<RcBrand, usize, (i32, String), i32> = IndexedGetter::new(|(x, _)| (0, x));
		/// let cloned = g.clone();
		/// assert_eq!(cloned.iview((42, "hi".to_string())), (0, 42));
		/// ```
		fn clone(&self) -> Self {
			IndexedGetter {
				to: self.to.clone(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The focus type."
	)]
	#[document_parameters("The indexed getter instance.")]
	impl<'a, PointerBrand, I: 'a, S: 'a, A: 'a> IndexedGetter<'a, PointerBrand, I, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		/// Create a new indexed getter.
		#[document_signature]
		#[document_parameters("The getter function.")]
		#[document_returns("A new `IndexedGetter` instance.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::IndexedGetter,
		/// };
		/// let g: IndexedGetter<RcBrand, usize, (i32, String), i32> = IndexedGetter::new(|(x, _)| (0, x));
		/// assert_eq!(g.iview((42, "hi".to_string())), (0, 42));
		/// ```
		pub fn new(to: impl 'a + Fn(S) -> (I, A)) -> Self {
			IndexedGetter {
				to: <FnBrand<PointerBrand> as LiftFn>::new(to),
			}
		}

		/// View the focus and its index.
		#[document_signature]
		#[document_parameters("The structure to view.")]
		#[document_returns("The focus value and its index.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::IndexedGetter,
		/// };
		/// let g: IndexedGetter<RcBrand, usize, (i32, String), i32> = IndexedGetter::new(|(x, _)| (0, x));
		/// assert_eq!(g.iview((42, "hi".to_string())), (0, 42));
		/// ```
		pub fn iview(
			&self,
			s: S,
		) -> (I, A) {
			(self.to)(s)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The focus type."
	)]
	#[document_parameters("The indexed getter instance.")]
	impl<'a, PointerBrand, I: 'a, S: 'a, A: 'a> IndexedGetterOptic<'a, I, S, A>
		for IndexedGetter<'a, PointerBrand, I, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The result type.", "The reference-counted pointer type.")]
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
		/// 	types::optics::*,
		/// };
		/// let g: IndexedGetter<RcBrand, usize, (i32, String), i32> = IndexedGetter::new(|(x, _)| (0, x));
		/// let f = Forget::<RcBrand, i32, (usize, i32), i32>::new(|(i, x)| x + (i as i32));
		/// let pab = Indexed::new(f);
		/// let result = IndexedGetterOptic::evaluate::<i32, RcBrand>(&g, pab);
		/// assert_eq!(result.run((42, "hi".to_string())), 42);
		/// ```
		fn evaluate<R: 'a + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Indexed<'a, ForgetBrand<Q, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>)
		{
			let to = self.to.clone();
			crate::types::optics::Forget::<Q, R, S, S>::new(move |s: S| {
				let pab_fn = pab.inner.0.clone();
				pab_fn(to(s))
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The focus type."
	)]
	#[document_parameters("The indexed getter instance.")]
	impl<'a, PointerBrand, I: 'a, S: 'a, A: 'a> IndexedFoldOptic<'a, I, S, A>
		for IndexedGetter<'a, PointerBrand, I, S, A>
	where
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
		/// 	classes::optics::*,
		/// 	types::optics::*,
		/// };
		/// let g: IndexedGetter<RcBrand, usize, (i32, String), i32> = IndexedGetter::new(|(x, _)| (0, x));
		/// let f = Forget::<RcBrand, String, (usize, i32), i32>::new(|(i, x)| format!("[{}]={}", i, x));
		/// let pab = Indexed::new(f);
		/// let result = IndexedFoldOptic::evaluate::<String, RcBrand>(&g, pab);
		/// assert_eq!(result.run((42, "hi".to_string())), "[0]=42");
		/// ```
		fn evaluate<
			R: 'a + crate::classes::monoid::Monoid + 'static,
			Q: UnsizedCoercible + 'static,
		>(
			&self,
			pab: Indexed<'a, ForgetBrand<Q, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>)
		{
			IndexedGetterOptic::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The result type brand.",
		"The result type.",
		"The original pointer type.",
		"The index type.",
		"The structure type.",
		"The focus type."
	)]
	#[document_parameters("The indexed getter instance.")]
	impl<
		'a,
		Q2: UnsizedCoercible + 'static,
		R: 'a + 'static,
		PointerBrand: UnsizedCoercible,
		I: 'a,
		S: 'a,
		A: 'a,
	> IndexedOpticAdapter<'a, ForgetBrand<Q2, R>, I, S, S, A, A>
		for IndexedGetter<'a, PointerBrand, I, S, A>
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
		/// 		optics::*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		/// let g: IndexedGetter<RcBrand, usize, (i32, String), i32> = IndexedGetter::new(|(x, _)| (0, x));
		/// let result = optics_indexed_view::<RcBrand, _, _, _>(&g, (42, "hi".to_string()));
		/// assert_eq!(result, (0, 42));
		/// ```
		fn evaluate_indexed(
			&self,
			pab: Indexed<'a, ForgetBrand<Q2, R>, I, A, A>,
		) -> Forget<'a, Q2, R, S, S> {
			IndexedGetterOptic::evaluate::<R, Q2>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The result type brand.",
		"The result type.",
		"The original pointer type.",
		"The index type.",
		"The structure type.",
		"The focus type."
	)]
	#[document_parameters("The indexed getter instance.")]
	impl<
		'a,
		Q2: UnsizedCoercible + 'static,
		R: 'a + 'static,
		PointerBrand: UnsizedCoercible,
		I: 'a,
		S: 'a,
		A: 'a,
	> IndexedOpticAdapterDiscardsFocus<'a, ForgetBrand<Q2, R>, I, S, S, A, A>
		for IndexedGetter<'a, PointerBrand, I, S, A>
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
		/// 		optics::*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		/// let g: IndexedGetter<RcBrand, usize, (i32, String), i32> = IndexedGetter::new(|(x, _)| (10, x));
		/// let result = optics_indexed_fold_map::<RcBrand, _, _, _, String>(
		/// 	&g,
		/// 	|i, _| format!("{}", i),
		/// 	(42, "hi".to_string()),
		/// );
		/// assert_eq!(result, "10");
		/// ```
		fn evaluate_indexed_discards_focus(
			&self,
			pab: Indexed<'a, ForgetBrand<Q2, R>, I, A, A>,
		) -> Forget<'a, Q2, R, S, S> {
			IndexedGetterOptic::evaluate::<R, Q2>(self, pab)
		}
	}

	/// A monomorphic indexed getter.
	pub type IndexedGetterPrime<'a, PointerBrand, I, S, A> =
		IndexedGetter<'a, PointerBrand, I, S, A>;
}

pub use inner::*;
