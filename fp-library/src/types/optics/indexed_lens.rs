//! Indexed lens optics for product types.

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
					Strong,
					Wander,
				},
			},
			kinds::*,
			types::optics::{
				ForgetBrand,
				Indexed,
			},
		},
		fp_macros::{
			document_examples,
			document_parameters,
			document_returns,
			document_type_parameters,
		},
	};

	/// A polymorphic indexed lens.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	pub struct IndexedLens<'a, P, I, S, T, A, B>
	where
		P: UnsizedCoercible,
		I: 'a,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		/// Internal storage: S -> ((I, A), B -> T)
		pub(crate) to: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, ((I, A), <FnBrand<P> as CloneableFn>::Of<'a, B, T>)>),
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P, I, S, T, A, B> Clone for IndexedLens<'a, P, I, S, T, A, B>
	where
		P: UnsizedCoercible,
		I: 'a,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_returns("A new `IndexedLens` instance that is a copy of the original.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::IndexedLens,
};
let l: IndexedLens<RcBrand, usize, (i32, String), (i32, String), i32, i32> =
	IndexedLens::from_iview_set(|(x, _)| (0, x), |((_, s), x)| (x, s));
let cloned = l.clone();
assert_eq!(cloned.iview((42, "hi".to_string())), (0, 42));"#
		)]
		fn clone(&self) -> Self {
			IndexedLens {
				to: self.to.clone(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a> IndexedLens<'a, P, I, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		/// Create a new polymorphic indexed lens.
		#[document_signature]
		#[document_parameters("The getter/setter pair function.")]
		#[document_returns("A new `IndexedLens` instance.")]
		#[document_examples(
			r#"use fp_library::{
	brands::{FnBrand, RcBrand},
	classes::CloneableFn,
	types::optics::IndexedLens,
};
let l: IndexedLens<RcBrand, usize, i32, String, i32, String> =
	IndexedLens::new(|x| ((0, x), <FnBrand<RcBrand> as CloneableFn>::new(|s| s)));
assert_eq!(l.iview(42), (0, 42));"#
		)]
		pub fn new(
			to: impl 'a + Fn(S) -> ((I, A), <FnBrand<P> as CloneableFn>::Of<'a, B, T>)
		) -> Self {
			IndexedLens {
				to: <FnBrand<P> as CloneableFn>::new(to),
			}
		}

		/// Create a new polymorphic indexed lens from an indexed getter and setter.
		#[document_signature]
		#[document_parameters("The indexed getter function.", "The setter function.")]
		#[document_returns("A new `IndexedLens` instance.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::IndexedLens,
};
let l: IndexedLens<RcBrand, usize, i32, String, i32, String> =
	IndexedLens::from_iview_set(|(x)| (0, x), |(_, s)| s);
assert_eq!(l.iview(42), (0, 42));"#
		)]
		pub fn from_iview_set(
			iview: impl 'a + Fn(S) -> (I, A),
			set: impl 'a + Fn((S, B)) -> T,
		) -> Self
		where
			S: Clone, {
			let iview_brand = <FnBrand<P> as CloneableFn>::new(iview);
			let set_brand = <FnBrand<P> as CloneableFn>::new(set);

			IndexedLens {
				to: <FnBrand<P> as CloneableFn>::new(move |s: S| {
					let s_clone = s.clone();
					let set_brand = set_brand.clone();
					(
						iview_brand(s),
						<FnBrand<P> as CloneableFn>::new(move |b| set_brand((s_clone.clone(), b))),
					)
				}),
			}
		}

		/// View the focus and its index.
		#[document_signature]
		#[document_parameters("The structure to view.")]
		#[document_returns("The focus value and its index.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::IndexedLens,
};
let l: IndexedLens<RcBrand, usize, i32, i32, i32, i32> =
	IndexedLens::from_iview_set(|x| (0, x), |(_, y)| y);
assert_eq!(l.iview(10), (0, 10));"#
		)]
		pub fn iview(
			&self,
			s: S,
		) -> (I, A) {
			(self.to)(s).0
		}

		/// Set the focus.
		#[document_signature]
		#[document_parameters("The structure to update.", "The new focus value.")]
		#[document_returns("The updated structure.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::IndexedLens,
};
let l: IndexedLens<RcBrand, usize, i32, i32, i32, i32> =
	IndexedLens::from_iview_set(|x| (0, x), |(_, y)| y);
assert_eq!(l.set(10, 20), 20);"#
		)]
		pub fn set(
			&self,
			s: S,
			b: B,
		) -> T {
			((self.to)(s).1)(b)
		}

		/// Update the focus using an indexed function.
		#[document_signature]
		#[document_parameters("The structure to update.", "The function to apply to the focus.")]
		#[document_returns("The updated structure.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::IndexedLens,
};
let l: IndexedLens<RcBrand, usize, i32, i32, i32, i32> =
	IndexedLens::from_iview_set(|x| (10, x), |(_, y)| y);
assert_eq!(l.over(10, |i, x| x + (i as i32)), 20);"#
		)]
		pub fn over(
			&self,
			s: S,
			f: impl Fn(I, A) -> B,
		) -> T {
			let ((i, a), set) = (self.to)(s);
			set(f(i, a))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The reference-counted pointer type for the lens."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P: Strong, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a, Q>
		IndexedOpticAdapter<'a, P, I, S, T, A, B> for IndexedLens<'a, Q, I, S, T, A, B>
	where
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::*,
};
let l: IndexedLensPrime<RcBrand, usize, (i32, String), i32> =
	IndexedLensPrime::from_iview_set(|(x, _)| (0, x), |((_, s), x)| (x, s));
let _unindexed = optics_un_index::<RcBrand, _, _, _, _, _, _>(&l);
// optics_un_index creates a non-indexed optic that retrieves the focus; the original indexed lens still works:
assert_eq!(l.iview((42, "hi".to_string())), (0, 42));"#
		)]
		fn evaluate_indexed(
			&self,
			pab: Indexed<'a, P, I, A, B>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			<Self as IndexedLensOptic<'a, I, S, T, A, B>>::evaluate::<P>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The reference-counted pointer type for the lens."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P: Strong, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a, Q>
		IndexedOpticAdapterDiscardsFocus<'a, P, I, S, T, A, B> for IndexedLens<'a, Q, I, S, T, A, B>
	where
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::*,
};
let l: IndexedLensPrime<RcBrand, usize, (i32, String), i32> =
	IndexedLensPrime::from_iview_set(|(x, _)| (10, x), |((_, s), x)| (x, s));
let _as_index = optics_as_index::<RcBrand, _, _, _, _, _, _>(&l);
// optics_as_index creates a non-indexed optic that retrieves the index as focus; the original indexed lens still works:
assert_eq!(l.iview((42, "hi".to_string())), (10, 42));"#
		)]
		fn evaluate_indexed_discards_focus(
			&self,
			pab: Indexed<'a, P, I, A, B>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			<Self as IndexedLensOptic<'a, I, S, T, A, B>>::evaluate::<P>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The source type of the structure.",
		"The source type of the focus.",
		"The optic type."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P: Strong, I: 'a, S: 'a, A: 'a, Q> IndexedOpticAdapter<'a, P, I, S, S, A, A>
		for IndexedLensPrime<'a, Q, I, S, A>
	where
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::*,
};
let l: IndexedLensPrime<RcBrand, usize, (i32, String), i32> =
	IndexedLensPrime::from_iview_set(|(x, _)| (0, x), |((_, s), x)| (x, s));
let _unindexed = optics_un_index::<RcBrand, _, _, _, _, _, _>(&l);
// optics_un_index creates a non-indexed optic that retrieves the focus; the original indexed lens still works:
assert_eq!(l.iview((42, "hi".to_string())), (0, 42));"#
		)]
		fn evaluate_indexed(
			&self,
			pab: Indexed<'a, P, I, A, A>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			<Self as IndexedLensOptic<'a, I, S, S, A, A>>::evaluate::<P>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The source type of the structure.",
		"The source type of the focus.",
		"The optic type."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P: Strong, I: 'a, S: 'a, A: 'a, Q>
		IndexedOpticAdapterDiscardsFocus<'a, P, I, S, S, A, A> for IndexedLensPrime<'a, Q, I, S, A>
	where
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::*,
};
let l: IndexedLensPrime<RcBrand, usize, (i32, String), i32> =
	IndexedLensPrime::from_iview_set(|(x, _)| (10, x), |((_, s), x)| (x, s));
let _as_index = optics_as_index::<RcBrand, _, _, _, _, _, _>(&l);
// optics_as_index creates a non-indexed optic that retrieves the index as focus; the original indexed lens still works:
assert_eq!(l.iview((42, "hi".to_string())), (10, 42));"#
		)]
		fn evaluate_indexed_discards_focus(
			&self,
			pab: Indexed<'a, P, I, A, A>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			<Self as IndexedLensOptic<'a, I, S, S, A, A>>::evaluate::<P>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a> IndexedLensOptic<'a, I, S, T, A, B>
		for IndexedLens<'a, P, I, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::optics::*,
	types::optics::*,
};
let l: IndexedLens<RcBrand, usize, (i32, String), (i32, String), i32, i32> =
	IndexedLens::from_iview_set(|(x, _)| (0, x), |((_, s), x)| (x, s));
let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32)) as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
let pab = Indexed::new(f);
let result: std::rc::Rc<dyn Fn((i32, String)) -> (i32, String)> =
	IndexedLensOptic::evaluate::<RcFnBrand>(&l, pab);
assert_eq!(result((42, "hi".to_string())), (42, "hi".to_string()));"#
		)]
		fn evaluate<Q: Strong>(
			&self,
			pab: Indexed<'a, Q, I, A, B>,
		) -> Apply!(<Q as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, T>) {
			let to = self.to.clone();
			Q::dimap(
				move |s: S| to(s),
				move |(b, f): (B, <FnBrand<P> as CloneableFn>::Of<'a, B, T>)| f(b),
				Q::first(pab.inner),
			)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a> IndexedTraversalOptic<'a, I, S, T, A, B>
		for IndexedLens<'a, P, I, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::optics::*,
	types::optics::*,
};
let l: IndexedLens<RcBrand, usize, (i32, String), (i32, String), i32, i32> =
	IndexedLens::from_iview_set(|(x, _)| (0, x), |((_, s), x)| (x, s));
let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32)) as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
let pab = Indexed::new(f);
let result: std::rc::Rc<dyn Fn((i32, String)) -> (i32, String)> =
	IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
assert_eq!(result((42, "hi".to_string())), (42, "hi".to_string()));"#
		)]
		fn evaluate<Q: Wander>(
			&self,
			pab: Indexed<'a, Q, I, A, B>,
		) -> Apply!(<Q as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, T>) {
			IndexedLensOptic::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The source type of the focus."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P, I: 'a, S: 'a, A: 'a> IndexedGetterOptic<'a, I, S, A>
		for IndexedLens<'a, P, I, S, S, A, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The result type.", "The reference-counted pointer type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::optics::*,
	types::optics::*,
};
let l: IndexedLens<RcBrand, usize, (i32, String), (i32, String), i32, i32> =
	IndexedLens::from_iview_set(|(x, _)| (0, x), |((_, s), x)| (x, s));
let f = Forget::<RcBrand, i32, (usize, i32), i32>::new(|(i, x)| x + (i as i32));
let pab = Indexed::new(f);
let result = IndexedGetterOptic::evaluate::<i32, RcBrand>(&l, pab);
assert_eq!(result.run((42, "hi".to_string())), 42);"#
		)]
		fn evaluate<R: 'a + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Indexed<'a, ForgetBrand<Q, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>)
		{
			IndexedLensOptic::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The source type of the focus."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P, I: 'a, S: 'a, A: 'a> IndexedFoldOptic<'a, I, S, A> for IndexedLens<'a, P, I, S, S, A, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::optics::*,
	types::optics::*,
};
let l: IndexedLens<RcBrand, usize, (i32, String), (i32, String), i32, i32> =
	IndexedLens::from_iview_set(|(x, _)| (0, x), |((_, s), x)| (x, s));
let f = Forget::<RcBrand, String, (usize, i32), i32>::new(|(i, x)| format!("[{}]={}", i, x));
let pab = Indexed::new(f);
let result = IndexedFoldOptic::evaluate::<String, RcBrand>(&l, pab);
assert_eq!(result.run((42, "hi".to_string())), "[0]=42");"#
		)]
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Indexed<'a, ForgetBrand<Q, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>)
		{
			IndexedLensOptic::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The reference-counted pointer type for the lens."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, Q, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a, P> IndexedSetterOptic<'a, Q, I, S, T, A, B>
		for IndexedLens<'a, P, I, S, T, A, B>
	where
		P: UnsizedCoercible,
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::optics::*,
	types::optics::*,
};
let l: IndexedLens<RcBrand, usize, (i32, String), (i32, String), i32, i32> =
	IndexedLens::from_iview_set(|(x, _)| (0, x), |((_, s), x)| (x, s));
let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32)) as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
let pab = Indexed::new(f);
let result: std::rc::Rc<dyn Fn((i32, String)) -> (i32, String)> =
	IndexedSetterOptic::evaluate(&l, pab);
assert_eq!(result((42, "hi".to_string())), (42, "hi".to_string()));"#
		)]
		fn evaluate(
			&self,
			pab: Indexed<'a, FnBrand<Q>, I, A, B>,
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, T>) {
			IndexedLensOptic::evaluate(self, pab)
		}
	}

	/// A monomorphic indexed lens.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	pub struct IndexedLensPrime<'a, P, I, S, A>
	where
		P: UnsizedCoercible,
		I: 'a,
		S: 'a,
		A: 'a, {
		pub(crate) to: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, ((I, A), <FnBrand<P> as CloneableFn>::Of<'a, A, S>)>),
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P, I, S, A> Clone for IndexedLensPrime<'a, P, I, S, A>
	where
		P: UnsizedCoercible,
		I: 'a,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_returns("A new `IndexedLensPrime` instance that is a copy of the original.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::IndexedLensPrime,
};
let l: IndexedLensPrime<RcBrand, usize, i32, i32> =
	IndexedLensPrime::from_iview_set(|x| (0, x), |(_, y)| y);
let cloned = l.clone();
assert_eq!(cloned.iview(42), (0, 42));"#
		)]
		fn clone(&self) -> Self {
			IndexedLensPrime {
				to: self.to.clone(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P, I: 'a, S: 'a, A: 'a> IndexedLensPrime<'a, P, I, S, A>
	where
		P: UnsizedCoercible,
	{
		/// Create a new monomorphic indexed lens.
		#[document_signature]
		#[document_parameters("The getter/setter pair function.")]
		#[document_returns("A new `IndexedLensPrime` instance.")]
		#[document_examples(
			r#"use fp_library::{
	brands::{FnBrand, RcBrand},
	classes::CloneableFn,
	types::optics::IndexedLensPrime,
};
let l: IndexedLensPrime<RcBrand, usize, i32, i32> =
	IndexedLensPrime::new(|x| ((0, x), <FnBrand<RcBrand> as CloneableFn>::new(|s| s)));
assert_eq!(l.iview(42), (0, 42));"#
		)]
		pub fn new(
			to: impl 'a + Fn(S) -> ((I, A), <FnBrand<P> as CloneableFn>::Of<'a, A, S>)
		) -> Self {
			IndexedLensPrime {
				to: <FnBrand<P> as CloneableFn>::new(to),
			}
		}

		/// Create a new monomorphic indexed lens from an indexed getter and setter.
		#[document_signature]
		#[document_parameters("The indexed getter function.", "The setter function.")]
		#[document_returns("A new `IndexedLensPrime` instance.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::IndexedLensPrime,
};
let l: IndexedLensPrime<RcBrand, usize, i32, i32> =
	IndexedLensPrime::from_iview_set(|x| (0, x), |(_, y)| y);
assert_eq!(l.iview(10), (0, 10));"#
		)]
		pub fn from_iview_set(
			iview: impl 'a + Fn(S) -> (I, A),
			set: impl 'a + Fn((S, A)) -> S,
		) -> Self
		where
			S: Clone, {
			let iview_brand = <FnBrand<P> as CloneableFn>::new(iview);
			let set_brand = <FnBrand<P> as CloneableFn>::new(set);

			IndexedLensPrime {
				to: <FnBrand<P> as CloneableFn>::new(move |s: S| {
					let s_clone = s.clone();
					let set_brand = set_brand.clone();
					(
						iview_brand(s),
						<FnBrand<P> as CloneableFn>::new(move |a| set_brand((s_clone.clone(), a))),
					)
				}),
			}
		}

		/// View the focus and its index.
		#[document_signature]
		#[document_parameters("The structure to view.")]
		#[document_returns("The focus value and its index.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::IndexedLensPrime,
};
let l: IndexedLensPrime<RcBrand, usize, i32, i32> =
	IndexedLensPrime::from_iview_set(|x| (0, x), |(_, y)| y);
assert_eq!(l.iview(42), (0, 42));"#
		)]
		pub fn iview(
			&self,
			s: S,
		) -> (I, A) {
			(self.to)(s).0
		}

		/// Set the focus.
		#[document_signature]
		#[document_parameters("The structure to update.", "The new focus value.")]
		#[document_returns("The updated structure.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::IndexedLensPrime,
};
let l: IndexedLensPrime<RcBrand, usize, i32, i32> =
	IndexedLensPrime::from_iview_set(|x| (0, x), |(_, y)| y);
assert_eq!(l.set(10, 20), 20);"#
		)]
		pub fn set(
			&self,
			s: S,
			a: A,
		) -> S {
			((self.to)(s).1)(a)
		}

		/// Update the focus using an indexed function.
		#[document_signature]
		#[document_parameters("The structure to update.", "The function to apply to the focus.")]
		#[document_returns("The updated structure.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::IndexedLensPrime,
};
let l: IndexedLensPrime<RcBrand, usize, i32, i32> =
	IndexedLensPrime::from_iview_set(|x| (10, x), |(_, y)| y);
assert_eq!(l.over(10, |i, x| x + (i as i32)), 20);"#
		)]
		pub fn over(
			&self,
			s: S,
			f: impl Fn(I, A) -> A,
		) -> S {
			let ((i, a), set) = (self.to)(s);
			set(f(i, a))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P, I: 'a, S: 'a, A: 'a> IndexedLensOptic<'a, I, S, S, A, A>
		for IndexedLensPrime<'a, P, I, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::optics::*,
	types::optics::*,
};
let l: IndexedLensPrime<RcBrand, usize, i32, i32> =
	IndexedLensPrime::from_iview_set(|x| (0, x), |(_, y)| y);
let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32)) as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
let pab = Indexed::new(f);
let result: std::rc::Rc<dyn Fn(i32) -> i32> =
	IndexedLensOptic::evaluate::<RcFnBrand>(&l, pab);
assert_eq!(result(42), 42);"#
		)]
		fn evaluate<Q: Strong>(
			&self,
			pab: Indexed<'a, Q, I, A, A>,
		) -> Apply!(<Q as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>) {
			let l: IndexedLens<'a, P, I, S, S, A, A> = IndexedLens {
				to: self.to.clone(),
			};
			IndexedLensOptic::evaluate(&l, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P, I: 'a, S: 'a, A: 'a> IndexedTraversalOptic<'a, I, S, S, A, A>
		for IndexedLensPrime<'a, P, I, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::optics::*,
	types::optics::*,
};
let l: IndexedLensPrime<RcBrand, usize, i32, i32> =
	IndexedLensPrime::from_iview_set(|x| (0, x), |(_, y)| y);
let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32)) as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
let pab = Indexed::new(f);
let result: std::rc::Rc<dyn Fn(i32) -> i32> =
	IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
assert_eq!(result(42), 42);"#
		)]
		fn evaluate<Q: Wander>(
			&self,
			pab: Indexed<'a, Q, I, A, A>,
		) -> Apply!(<Q as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>) {
			IndexedLensOptic::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P, I: 'a, S: 'a, A: 'a> IndexedGetterOptic<'a, I, S, A>
		for IndexedLensPrime<'a, P, I, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The result type.", "The reference-counted pointer type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::optics::*,
	types::optics::*,
};
let l: IndexedLensPrime<RcBrand, usize, i32, i32> =
	IndexedLensPrime::from_iview_set(|x| (0, x), |(_, y)| y);
let f = Forget::<RcBrand, i32, (usize, i32), i32>::new(|(i, x)| x + (i as i32));
let pab = Indexed::new(f);
let result = IndexedGetterOptic::evaluate::<i32, RcBrand>(&l, pab);
assert_eq!(result.run(42), 42);"#
		)]
		fn evaluate<R: 'a + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Indexed<'a, ForgetBrand<Q, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>)
		{
			IndexedLensOptic::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, P, I: 'a, S: 'a, A: 'a> IndexedFoldOptic<'a, I, S, A> for IndexedLensPrime<'a, P, I, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::optics::*,
	types::optics::*,
};
let l: IndexedLensPrime<RcBrand, usize, i32, i32> =
	IndexedLensPrime::from_iview_set(|x| (0, x), |(_, y)| y);
let f = Forget::<RcBrand, String, (usize, i32), i32>::new(|(i, x)| format!("[{}]={}", i, x));
let pab = Indexed::new(f);
let result = IndexedFoldOptic::evaluate::<String, RcBrand>(&l, pab);
assert_eq!(result.run(42), "[0]=42");"#
		)]
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Indexed<'a, ForgetBrand<Q, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>)
		{
			IndexedLensOptic::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The reference-counted pointer type for the lens."
	)]
	#[document_parameters("The indexed lens instance.")]
	impl<'a, Q, I: 'a, S: 'a, A: 'a, P> IndexedSetterOptic<'a, Q, I, S, S, A, A>
		for IndexedLensPrime<'a, P, I, S, A>
	where
		P: UnsizedCoercible,
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::optics::*,
	types::optics::*,
};
let l: IndexedLensPrime<RcBrand, usize, i32, i32> =
	IndexedLensPrime::from_iview_set(|x| (0, x), |(_, y)| y);
let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32)) as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
let pab = Indexed::new(f);
let result: std::rc::Rc<dyn Fn(i32) -> i32> =
	IndexedSetterOptic::evaluate(&l, pab);
assert_eq!(result(42), 42);"#
		)]
		fn evaluate(
			&self,
			pab: Indexed<'a, FnBrand<Q>, I, A, A>,
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>) {
			IndexedLensOptic::evaluate(self, pab)
		}
	}
}

pub use inner::*;
