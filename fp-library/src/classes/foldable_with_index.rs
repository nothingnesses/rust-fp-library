//! A `Foldable` with an additional index.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			types::Endofunction,
		},
		fp_macros::*,
	};

	/// A `Foldable` with an additional index.
	///
	/// A `FoldableWithIndex` is a `Foldable` that also allows you to access the
	/// index of each element when folding over the structure. The index type is
	/// uniquely determined by the implementing brand via the [`WithIndex`] supertype.
	///
	/// ### Laws
	///
	/// `FoldableWithIndex` instances must be compatible with their `Foldable` instance:
	/// * Compatibility with Foldable: `fold_map(f, fa) = fold_map_with_index(|_, a| f(a), fa)`.
	#[document_examples]
	///
	/// FoldableWithIndex laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::foldable_with_index::FoldableWithIndex,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![1, 2, 3];
	/// let f = |a: i32| a.to_string();
	///
	/// // Compatibility with Foldable:
	/// // fold_map(f, fa) = fold_map_with_index(|_, a| f(a), fa)
	/// assert_eq!(
	/// 	fold_map::<RcFnBrand, VecBrand, _, _, _, _>(f, xs.clone()),
	/// 	VecBrand::fold_map_with_index::<RcFnBrand, _, _>(|_, a| f(a), xs),
	/// );
	/// ```
	pub trait FoldableWithIndex: Foldable + WithIndex {
		/// Map each element of the structure to a monoid, and combine the results,
		/// providing the index of each element.
		///
		/// Default implementation derives from `fold_right_with_index`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to apply to each element and its index.",
			"The structure to fold over."
		)]
		#[document_returns("The combined result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::foldable_with_index::FoldableWithIndex,
		/// };
		///
		/// let result = VecBrand::fold_map_with_index::<RcFnBrand, _, _>(
		/// 	|i, x: i32| format!("{i}:{x}"),
		/// 	vec![10, 20, 30],
		/// );
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		fn fold_map_with_index<'a, FnBrand, A: 'a + Clone, R: Monoid + 'a>(
			f: impl Fn(Self::Index, A) -> R + 'a,
			fa: Self::Of<'a, A>,
		) -> R
		where
			FnBrand: LiftFn + 'a,
			Self::Index: 'a, {
			Self::fold_right_with_index::<FnBrand, A, R>(
				move |i, a, acc| Semigroup::append(f(i, a), acc),
				Monoid::empty(),
				fa,
			)
		}

		/// Folds the structure with index by applying a function from right to left.
		///
		/// Default implementation derives from `fold_map_with_index`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements.",
			"The type of the accumulator."
		)]
		#[document_parameters(
			"The function to apply to each element's index, the element, and the accumulator.",
			"The initial accumulator value.",
			"The structure to fold over."
		)]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::foldable_with_index::FoldableWithIndex,
		/// };
		///
		/// let result = VecBrand::fold_right_with_index::<RcFnBrand, _, _>(
		/// 	|i, x: i32, acc: String| format!("{acc}{i}:{x},"),
		/// 	String::new(),
		/// 	vec![10, 20, 30],
		/// );
		/// assert_eq!(result, "2:30,1:20,0:10,");
		/// ```
		fn fold_right_with_index<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(Self::Index, A, B) -> B + 'a,
			initial: B,
			fa: Self::Of<'a, A>,
		) -> B
		where
			FnBrand: LiftFn + 'a,
			Self::Index: 'a, {
			let f = <FnBrand as LiftFn>::new(move |(i, a, b): (Self::Index, A, B)| func(i, a, b));
			let m = Self::fold_map_with_index::<FnBrand, A, Endofunction<FnBrand, B>>(
				move |i: Self::Index, a: A| {
					let f = f.clone();
					Endofunction::<FnBrand, B>::new(<FnBrand as LiftFn>::new(move |b| {
						f((i.clone(), a.clone(), b))
					}))
				},
				fa,
			);
			m.0(initial)
		}

		/// Folds the structure with index by applying a function from left to right.
		///
		/// Default implementation derives from `fold_map_with_index`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements.",
			"The type of the accumulator."
		)]
		#[document_parameters(
			"The function to apply to the index, the accumulator, and each element.",
			"The initial accumulator value.",
			"The structure to fold over."
		)]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::foldable_with_index::FoldableWithIndex,
		/// };
		///
		/// let result = VecBrand::fold_left_with_index::<RcFnBrand, _, _>(
		/// 	|i, acc: String, x: i32| format!("{acc}{i}:{x},"),
		/// 	String::new(),
		/// 	vec![10, 20, 30],
		/// );
		/// assert_eq!(result, "0:10,1:20,2:30,");
		/// ```
		fn fold_left_with_index<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(Self::Index, B, A) -> B + 'a,
			initial: B,
			fa: Self::Of<'a, A>,
		) -> B
		where
			FnBrand: LiftFn + 'a,
			Self::Index: 'a, {
			let f = <FnBrand as LiftFn>::new(move |(i, b, a): (Self::Index, B, A)| func(i, b, a));
			let m = Self::fold_map_with_index::<
				FnBrand,
				A,
				crate::types::Dual<Endofunction<FnBrand, B>>,
			>(
				move |i: Self::Index, a: A| {
					let f = f.clone();
					crate::types::Dual(Endofunction::<FnBrand, B>::new(<FnBrand as LiftFn>::new(
						move |b| f((i.clone(), b, a.clone())),
					)))
				},
				fa,
			);
			m.0.0(initial)
		}
	}

	/// Maps each element to a monoid with its index and combines the results.
	///
	/// Free function version that dispatches to [the type class' associated function][`FoldableWithIndex::fold_map_with_index`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the structure.",
		"The type of the elements.",
		"The monoid type."
	)]
	#[document_parameters(
		"The function to apply to each element and its index.",
		"The structure to fold over."
	)]
	#[document_returns("The combined result.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let result = fold_map_with_index::<RcFnBrand, VecBrand, _, _>(
	/// 	|i, x: i32| format!("{i}:{x}"),
	/// 	vec![10, 20, 30],
	/// );
	/// assert_eq!(result, "0:101:202:30");
	/// ```
	pub fn fold_map_with_index<
		'a,
		FnBrand: LiftFn + 'a,
		Brand: FoldableWithIndex,
		A: 'a + Clone,
		R: Monoid + 'a,
	>(
		f: impl Fn(Brand::Index, A) -> R + 'a,
		fa: Brand::Of<'a, A>,
	) -> R
	where
		Brand::Index: 'a, {
		Brand::fold_map_with_index::<FnBrand, A, R>(f, fa)
	}

	/// Folds the structure with index by applying a function from right to left.
	///
	/// Free function version that dispatches to [the type class' associated function][`FoldableWithIndex::fold_right_with_index`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the structure.",
		"The type of the elements.",
		"The type of the accumulator."
	)]
	#[document_parameters(
		"The function to apply.",
		"The initial accumulator value.",
		"The structure to fold over."
	)]
	#[document_returns("The final accumulator value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let result = fold_right_with_index::<RcFnBrand, VecBrand, _, _>(
	/// 	|i, x: i32, acc: String| format!("{acc}{i}:{x},"),
	/// 	String::new(),
	/// 	vec![10, 20, 30],
	/// );
	/// assert_eq!(result, "2:30,1:20,0:10,");
	/// ```
	pub fn fold_right_with_index<
		'a,
		FnBrand: LiftFn + 'a,
		Brand: FoldableWithIndex,
		A: 'a + Clone,
		B: 'a,
	>(
		func: impl Fn(Brand::Index, A, B) -> B + 'a,
		initial: B,
		fa: Brand::Of<'a, A>,
	) -> B
	where
		Brand::Index: 'a, {
		Brand::fold_right_with_index::<FnBrand, A, B>(func, initial, fa)
	}

	/// Folds the structure with index by applying a function from left to right.
	///
	/// Free function version that dispatches to [the type class' associated function][`FoldableWithIndex::fold_left_with_index`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the structure.",
		"The type of the elements.",
		"The type of the accumulator."
	)]
	#[document_parameters(
		"The function to apply.",
		"The initial accumulator value.",
		"The structure to fold over."
	)]
	#[document_returns("The final accumulator value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let result = fold_left_with_index::<RcFnBrand, VecBrand, _, _>(
	/// 	|i, acc: String, x: i32| format!("{acc}{i}:{x},"),
	/// 	String::new(),
	/// 	vec![10, 20, 30],
	/// );
	/// assert_eq!(result, "0:10,1:20,2:30,");
	/// ```
	pub fn fold_left_with_index<
		'a,
		FnBrand: LiftFn + 'a,
		Brand: FoldableWithIndex,
		A: 'a + Clone,
		B: 'a,
	>(
		func: impl Fn(Brand::Index, B, A) -> B + 'a,
		initial: B,
		fa: Brand::Of<'a, A>,
	) -> B
	where
		Brand::Index: 'a, {
		Brand::fold_left_with_index::<FnBrand, A, B>(func, initial, fa)
	}
}

pub use inner::*;
