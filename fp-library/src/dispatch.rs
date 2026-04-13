//! Dispatch infrastructure for unified free functions that route to either
//! by-value or by-reference trait methods based on the closure's argument type.
//!
//! The dispatch system uses marker types ([`Val`] and [`Ref`]) to select the
//! appropriate trait at compile time. The compiler infers the marker from the
//! closure's argument type: `Fn(A) -> B` resolves to [`Val`], `Fn(&A) -> B`
//! resolves to [`Ref`].
//!
//! The [`ClosureMode`] trait maps each marker to the corresponding `dyn Fn`
//! trait object type, used by [`CloneFn`](crate::classes::CloneFn) and
//! [`SendCloneFn`](crate::classes::SendCloneFn) to parameterize the `Deref`
//! target of wrapped closures.
//!
//! ### Sub-modules
//!
//! Each sub-module provides a dispatch trait and unified free function for
//! a specific type class operation, mirroring the corresponding `classes/`
//! module:
//!
//! - [`functor`]: `FunctorDispatch` + `map`
//! - [`semimonad`]: `BindDispatch` + `bind`
//! - [`lift`]: `Lift2Dispatch`-`Lift5Dispatch` + `lift2`-`lift5`
//! - [`foldable`]: `FoldRightDispatch`, `FoldLeftDispatch`, `FoldMapDispatch` + `fold_right`, `fold_left`, `fold_map`
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! 	types::*,
//! };
//!
//! // Closure takes i32 -> dispatches to Functor::map
//! let y = map::<OptionBrand, _, _, _, _>(|x: i32| x * 2, Some(5));
//! assert_eq!(y, Some(10));
//!
//! // Closure takes &i32 -> dispatches to RefFunctor::ref_map
//! let lazy = RcLazy::pure(10);
//! let mapped = map::<LazyBrand<RcLazyConfig>, _, _, _, _>(|x: &i32| *x * 2, &lazy);
//! assert_eq!(*mapped.evaluate(), 20);
//! ```

#[fp_macros::document_module]
mod inner {
	// -- Marker types --

	/// Marker type indicating the closure receives owned values.
	///
	/// Selected automatically by the compiler when the closure's argument
	/// type is `A` (not `&A`). Routes to by-value trait methods
	/// (e.g., [`Functor::map`](crate::classes::Functor::map),
	/// [`Semimonad::bind`](crate::classes::Semimonad::bind)).
	pub struct Val;

	/// Marker type indicating the closure receives references.
	///
	/// Selected automatically by the compiler when the closure's argument
	/// type is `&A`. Routes to by-reference trait methods
	/// (e.g., [`RefFunctor::ref_map`](crate::classes::RefFunctor::ref_map),
	/// [`RefSemimonad::ref_bind`](crate::classes::RefSemimonad::ref_bind)).
	pub struct Ref;

	// -- Closure mode --

	/// Trait that maps a closure mode marker ([`Val`] or [`Ref`]) to the
	/// corresponding `dyn Fn` trait object type.
	///
	/// Used by [`CloneFn`](crate::classes::CloneFn) to parameterize
	/// the `Deref` target of wrapped closures. `Val` produces
	/// `dyn Fn(A) -> B` (by-value), `Ref` produces `dyn Fn(&A) -> B`
	/// (by-reference).
	pub trait ClosureMode {
		/// The unsized closure trait object type for this mode.
		type Target<'a, A: 'a, B: 'a>: ?Sized + 'a;

		/// The unsized closure trait object type for this mode with `Send + Sync` bounds.
		type SendTarget<'a, A: 'a, B: 'a>: ?Sized + 'a;
	}

	impl ClosureMode for Val {
		type SendTarget<'a, A: 'a, B: 'a> = dyn 'a + Fn(A) -> B + Send + Sync;
		type Target<'a, A: 'a, B: 'a> = dyn 'a + Fn(A) -> B;
	}

	impl ClosureMode for Ref {
		type SendTarget<'a, A: 'a, B: 'a> = dyn 'a + Fn(&A) -> B + Send + Sync;
		type Target<'a, A: 'a, B: 'a> = dyn 'a + Fn(&A) -> B;
	}
}

pub use inner::*;

pub mod alt;
pub mod apply_first;
pub mod apply_second;
pub mod bifoldable;
pub mod bifunctor;
pub mod bitraversable;
pub mod compactable;
pub mod filterable;
pub mod filterable_with_index;
pub mod foldable;
pub mod foldable_with_index;
pub mod functor;
pub mod functor_with_index;
pub mod lift;
pub mod semimonad;
pub mod traversable;
pub mod traversable_with_index;
pub mod witherable;

#[cfg(test)]
mod tests {
	use {
		super::{
			functor::explicit::map,
			lift::explicit::lift2,
			semimonad::explicit::bind,
		},
		crate::{
			brands::*,
			types::*,
		},
	};

	#[test]
	fn test_val_option_map() {
		let result = map::<OptionBrand, _, _, _, _>(|x: i32| x * 2, Some(5));
		assert_eq!(result, Some(10));
	}

	#[test]
	fn test_val_vec_map() {
		let result = map::<VecBrand, _, _, _, _>(|x: i32| x + 1, vec![1, 2, 3]);
		assert_eq!(result, vec![2, 3, 4]);
	}

	#[test]
	fn test_ref_lazy_map() {
		let lazy = RcLazy::pure(10);
		let result = map::<LazyBrand<RcLazyConfig>, _, _, _, _>(|x: &i32| *x * 2, &lazy);
		assert_eq!(*result.evaluate(), 20);
	}

	#[test]
	fn test_val_none_map() {
		let result = map::<OptionBrand, i32, i32, _, _>(|x| x * 2, None);
		assert_eq!(result, None);
	}

	#[test]
	fn test_val_option_bind() {
		let result = bind::<OptionBrand, _, _, _, _>(Some(5), |x: i32| Some(x * 2));
		assert_eq!(result, Some(10));
	}

	#[test]
	fn test_val_option_lift2() {
		let result = lift2::<OptionBrand, _, _, _, _, _, _>(|a, b| a + b, Some(1), Some(2));
		assert_eq!(result, Some(3));
	}

	// -- FilterMapDispatch tests --

	#[test]
	fn test_val_option_filter_map() {
		use super::filterable::explicit::filter_map;
		let result = filter_map::<OptionBrand, _, _, _, _>(
			|x: i32| if x > 3 { Some(x * 2) } else { None },
			Some(5),
		);
		assert_eq!(result, Some(10));
	}

	#[test]
	fn test_val_option_filter_map_none() {
		use super::filterable::explicit::filter_map;
		let result = filter_map::<OptionBrand, _, _, _, _>(
			|x: i32| if x > 10 { Some(x) } else { None },
			Some(5),
		);
		assert_eq!(result, None);
	}

	#[test]
	fn test_ref_option_filter_map() {
		use super::filterable::explicit::filter_map;
		let result = filter_map::<OptionBrand, _, _, _, _>(
			|x: &i32| if *x > 3 { Some(*x * 2) } else { None },
			&Some(5),
		);
		assert_eq!(result, Some(10));
	}

	#[test]
	fn test_val_vec_filter_map() {
		use super::filterable::explicit::filter_map;
		let result = filter_map::<VecBrand, _, _, _, _>(
			|x: i32| if x > 2 { Some(x * 10) } else { None },
			vec![1, 2, 3, 4],
		);
		assert_eq!(result, vec![30, 40]);
	}

	#[test]
	fn test_ref_vec_filter_map() {
		use super::filterable::explicit::filter_map;
		let v = vec![1, 2, 3, 4];
		let result = filter_map::<VecBrand, _, _, _, _>(
			|x: &i32| if *x > 2 { Some(*x * 10) } else { None },
			&v,
		);
		assert_eq!(result, vec![30, 40]);
	}

	// -- TraverseDispatch tests --

	#[test]
	fn test_val_option_traverse() {
		use super::traversable::explicit::traverse;
		let result = traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _, _>(
			|x: i32| Some(x * 2),
			Some(5),
		);
		assert_eq!(result, Some(Some(10)));
	}

	#[test]
	fn test_val_option_traverse_none() {
		use super::traversable::explicit::traverse;
		let result = traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _, _>(
			|_: i32| None::<i32>,
			Some(5),
		);
		assert_eq!(result, None);
	}

	#[test]
	fn test_ref_option_traverse() {
		use super::traversable::explicit::traverse;
		let result = traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _, _>(
			|x: &i32| Some(*x * 2),
			&Some(5),
		);
		assert_eq!(result, Some(Some(10)));
	}

	#[test]
	fn test_val_vec_traverse() {
		use super::traversable::explicit::traverse;
		let result: Option<Vec<i32>> = traverse::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(
			|x: i32| Some(x * 2),
			vec![1, 2, 3],
		);
		assert_eq!(result, Some(vec![2, 4, 6]));
	}

	#[test]
	fn test_ref_vec_traverse() {
		use super::traversable::explicit::traverse;
		let v = vec![1, 2, 3];
		let result: Option<Vec<i32>> =
			traverse::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(|x: &i32| Some(*x * 2), &v);
		assert_eq!(result, Some(vec![2, 4, 6]));
	}

	// -- FilterDispatch tests --

	#[test]
	fn test_val_option_filter() {
		use super::filterable::explicit::filter;
		let result = filter::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
		assert_eq!(result, Some(5));
	}

	#[test]
	fn test_ref_option_filter() {
		use super::filterable::explicit::filter;
		let result = filter::<OptionBrand, _, _, _>(|x: &i32| *x > 3, &Some(5));
		assert_eq!(result, Some(5));
	}

	#[test]
	fn test_val_vec_filter() {
		use super::filterable::explicit::filter;
		let result = filter::<VecBrand, _, _, _>(|x: i32| x > 3, vec![1, 2, 3, 4, 5]);
		assert_eq!(result, vec![4, 5]);
	}

	#[test]
	fn test_ref_vec_filter() {
		use super::filterable::explicit::filter;
		let v = vec![1, 2, 3, 4, 5];
		let result = filter::<VecBrand, _, _, _>(|x: &i32| *x > 3, &v);
		assert_eq!(result, vec![4, 5]);
	}

	// -- PartitionDispatch tests --

	#[test]
	fn test_val_option_partition() {
		use super::filterable::explicit::partition;
		let (no, yes) = partition::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
		assert_eq!(yes, Some(5));
		assert_eq!(no, None);
	}

	#[test]
	fn test_ref_option_partition() {
		use super::filterable::explicit::partition;
		let (no, yes) = partition::<OptionBrand, _, _, _>(|x: &i32| *x > 3, &Some(5));
		assert_eq!(yes, Some(5));
		assert_eq!(no, None);
	}

	// -- PartitionMapDispatch tests --

	#[test]
	fn test_val_option_partition_map() {
		use super::filterable::explicit::partition_map;
		let (errs, oks) =
			partition_map::<OptionBrand, _, _, _, _, _>(|x: i32| Ok::<i32, i32>(x * 2), Some(5));
		assert_eq!(errs, None);
		assert_eq!(oks, Some(10));
	}

	#[test]
	fn test_ref_option_partition_map() {
		use super::filterable::explicit::partition_map;
		let (errs, oks) =
			partition_map::<OptionBrand, _, _, _, _, _>(|x: &i32| Ok::<i32, i32>(*x * 2), &Some(5));
		assert_eq!(errs, None);
		assert_eq!(oks, Some(10));
	}

	// -- MapWithIndexDispatch tests --

	#[test]
	fn test_val_vec_map_with_index() {
		use super::functor_with_index::explicit::map_with_index;
		let result =
			map_with_index::<VecBrand, _, _, _, _>(|i, x: i32| x + i as i32, vec![10, 20, 30]);
		assert_eq!(result, vec![10, 21, 32]);
	}

	#[test]
	fn test_ref_vec_map_with_index() {
		use super::functor_with_index::explicit::map_with_index;
		let v = vec![10, 20, 30];
		let result = map_with_index::<VecBrand, _, _, _, _>(|i, x: &i32| *x + i as i32, &v);
		assert_eq!(result, vec![10, 21, 32]);
	}

	// -- FilterWithIndexDispatch tests --

	#[test]
	fn test_val_vec_filter_with_index() {
		use super::filterable_with_index::explicit::filter_with_index;
		let result =
			filter_with_index::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
		assert_eq!(result, vec![10, 20]);
	}

	#[test]
	fn test_ref_vec_filter_with_index() {
		use super::filterable_with_index::explicit::filter_with_index;
		let v = vec![10, 20, 30, 40];
		let result = filter_with_index::<VecBrand, _, _, _>(|i, _x: &i32| i < 2, &v);
		assert_eq!(result, vec![10, 20]);
	}

	// -- FilterMapWithIndexDispatch tests --

	#[test]
	fn test_val_vec_filter_map_with_index() {
		use super::filterable_with_index::explicit::filter_map_with_index;
		let result = filter_map_with_index::<VecBrand, _, _, _, _>(
			|i, x: i32| if i % 2 == 0 { Some(x * 2) } else { None },
			vec![10, 20, 30, 40],
		);
		assert_eq!(result, vec![20, 60]);
	}

	#[test]
	fn test_ref_vec_filter_map_with_index() {
		use super::filterable_with_index::explicit::filter_map_with_index;
		let v = vec![10, 20, 30, 40];
		let result = filter_map_with_index::<VecBrand, _, _, _, _>(
			|i, x: &i32| if i % 2 == 0 { Some(*x * 2) } else { None },
			&v,
		);
		assert_eq!(result, vec![20, 60]);
	}

	// -- PartitionWithIndexDispatch tests --

	#[test]
	fn test_val_vec_partition_with_index() {
		use super::filterable_with_index::explicit::partition_with_index;
		let (not_satisfied, satisfied) =
			partition_with_index::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
		assert_eq!(satisfied, vec![10, 20]);
		assert_eq!(not_satisfied, vec![30, 40]);
	}

	#[test]
	fn test_ref_vec_partition_with_index() {
		use super::filterable_with_index::explicit::partition_with_index;
		let v = vec![10, 20, 30, 40];
		let (not_satisfied, satisfied) =
			partition_with_index::<VecBrand, _, _, _>(|i, _x: &i32| i < 2, &v);
		assert_eq!(satisfied, vec![10, 20]);
		assert_eq!(not_satisfied, vec![30, 40]);
	}

	// -- PartitionMapWithIndexDispatch tests --

	#[test]
	fn test_val_vec_partition_map_with_index() {
		use super::filterable_with_index::explicit::partition_map_with_index;
		let (errs, oks) = partition_map_with_index::<VecBrand, _, _, _, _, _>(
			|i, x: i32| if i < 2 { Ok(x) } else { Err(x) },
			vec![10, 20, 30, 40],
		);
		assert_eq!(oks, vec![10, 20]);
		assert_eq!(errs, vec![30, 40]);
	}

	#[test]
	fn test_ref_vec_partition_map_with_index() {
		use super::filterable_with_index::explicit::partition_map_with_index;
		let v = vec![10, 20, 30, 40];
		let (errs, oks) = partition_map_with_index::<VecBrand, _, _, _, _, _>(
			|i, x: &i32| if i < 2 { Ok(*x) } else { Err(*x) },
			&v,
		);
		assert_eq!(oks, vec![10, 20]);
		assert_eq!(errs, vec![30, 40]);
	}

	// -- FoldMapWithIndexDispatch tests --

	#[test]
	fn test_val_vec_fold_map_with_index() {
		use super::foldable_with_index::explicit::fold_map_with_index;
		let result = fold_map_with_index::<RcFnBrand, VecBrand, _, _, _, _>(
			|i, x: i32| format!("{i}:{x}"),
			vec![10, 20, 30],
		);
		assert_eq!(result, "0:101:202:30");
	}

	#[test]
	fn test_ref_vec_fold_map_with_index() {
		use super::foldable_with_index::explicit::fold_map_with_index;
		let v = vec![10, 20, 30];
		let result = fold_map_with_index::<RcFnBrand, VecBrand, _, _, _, _>(
			|i, x: &i32| format!("{i}:{x}"),
			&v,
		);
		assert_eq!(result, "0:101:202:30");
	}

	// -- FoldRightWithIndexDispatch tests --

	#[test]
	fn test_val_vec_fold_right_with_index() {
		use super::foldable_with_index::explicit::fold_right_with_index;
		let result = fold_right_with_index::<RcFnBrand, VecBrand, _, _, _, _>(
			|i, x: i32, acc: String| format!("{acc}{i}:{x},"),
			String::new(),
			vec![10, 20, 30],
		);
		assert_eq!(result, "2:30,1:20,0:10,");
	}

	#[test]
	fn test_ref_vec_fold_right_with_index() {
		use super::foldable_with_index::explicit::fold_right_with_index;
		let v = vec![10, 20, 30];
		let result = fold_right_with_index::<RcFnBrand, VecBrand, _, _, _, _>(
			|i, x: &i32, acc: String| format!("{acc}{i}:{x},"),
			String::new(),
			&v,
		);
		assert_eq!(result, "2:30,1:20,0:10,");
	}

	// -- FoldLeftWithIndexDispatch tests --

	#[test]
	fn test_val_vec_fold_left_with_index() {
		use super::foldable_with_index::explicit::fold_left_with_index;
		let result = fold_left_with_index::<RcFnBrand, VecBrand, _, _, _, _>(
			|i, acc: String, x: i32| format!("{acc}{i}:{x},"),
			String::new(),
			vec![10, 20, 30],
		);
		assert_eq!(result, "0:10,1:20,2:30,");
	}

	#[test]
	fn test_ref_vec_fold_left_with_index() {
		use super::foldable_with_index::explicit::fold_left_with_index;
		let v = vec![10, 20, 30];
		let result = fold_left_with_index::<RcFnBrand, VecBrand, _, _, _, _>(
			|i, acc: String, x: &i32| format!("{acc}{i}:{x},"),
			String::new(),
			&v,
		);
		assert_eq!(result, "0:10,1:20,2:30,");
	}

	// -- TraverseWithIndexDispatch tests --

	#[test]
	fn test_val_vec_traverse_with_index() {
		use super::traversable_with_index::explicit::traverse_with_index;
		let result = traverse_with_index::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(
			|_i, x: i32| Some(x * 2),
			vec![1, 2, 3],
		);
		assert_eq!(result, Some(vec![2, 4, 6]));
	}

	#[test]
	fn test_ref_vec_traverse_with_index() {
		use super::traversable_with_index::explicit::traverse_with_index;
		let v = vec![1, 2, 3];
		let result = traverse_with_index::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(
			|_i, x: &i32| Some(*x * 2),
			&v,
		);
		assert_eq!(result, Some(vec![2, 4, 6]));
	}

	// -- WiltDispatch tests --

	#[test]
	fn test_val_option_wilt() {
		use super::witherable::explicit::wilt;
		let result = wilt::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _, _>(
			|a: i32| Some(if a > 2 { Ok(a) } else { Err(a) }),
			Some(5),
		);
		assert_eq!(result, Some((None, Some(5))));
	}

	#[test]
	fn test_ref_vec_wilt() {
		use super::witherable::explicit::wilt;
		let v = vec![1, 2, 3, 4, 5];
		let result: Option<(Vec<i32>, Vec<i32>)> =
			wilt::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _, _>(
				|x: &i32| Some(if *x > 3 { Ok(*x) } else { Err(*x) }),
				&v,
			);
		assert_eq!(result, Some((vec![1, 2, 3], vec![4, 5])));
	}

	// -- WitherDispatch tests --

	#[test]
	fn test_val_option_wither() {
		use super::witherable::explicit::wither;
		let result = wither::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _>(
			|a: i32| Some(if a > 2 { Some(a * 2) } else { None }),
			Some(5),
		);
		assert_eq!(result, Some(Some(10)));
	}

	#[test]
	fn test_ref_vec_wither() {
		use super::witherable::explicit::wither;
		let v = vec![1, 2, 3, 4, 5];
		let result: Option<Vec<i32>> = wither::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _>(
			|x: &i32| if *x > 3 { Some(Some(*x)) } else { Some(None) },
			&v,
		);
		assert_eq!(result, Some(vec![4, 5]));
	}

	// -- BimapDispatch tests --

	#[test]
	fn test_val_result_bimap() {
		use super::bifunctor::explicit::bimap;
		let x = Result::<i32, i32>::Ok(5);
		let y = bimap::<ResultBrand, _, _, _, _, _, _>((|e| e + 1, |s| s * 2), x);
		assert_eq!(y, Ok(10));
	}

	#[test]
	fn test_val_result_bimap_err() {
		use super::bifunctor::explicit::bimap;
		let x = Result::<i32, i32>::Err(3);
		let y = bimap::<ResultBrand, _, _, _, _, _, _>((|e| e + 1, |s| s * 2), x);
		assert_eq!(y, Err(4));
	}

	#[test]
	fn test_ref_result_bimap() {
		use super::bifunctor::explicit::bimap;
		let x = Result::<i32, i32>::Ok(5);
		let y = bimap::<ResultBrand, _, _, _, _, _, _>((|e: &i32| *e + 1, |s: &i32| *s * 2), &x);
		assert_eq!(y, Ok(10));
	}

	// -- BiFoldRightDispatch tests --

	#[test]
	fn test_val_result_bi_fold_right() {
		use super::bifoldable::explicit::bi_fold_right;
		let x: Result<i32, i32> = Err(3);
		let y = bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
			(|e, acc| acc - e, |s, acc| acc + s),
			10,
			x,
		);
		assert_eq!(y, 7);
	}

	#[test]
	fn test_ref_result_bi_fold_right() {
		use super::bifoldable::explicit::bi_fold_right;
		let x: Result<i32, i32> = Err(3);
		let y = bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
			(|e: &i32, acc| acc - *e, |s: &i32, acc| acc + *s),
			10,
			&x,
		);
		assert_eq!(y, 7);
	}

	// -- BiFoldLeftDispatch tests --

	#[test]
	fn test_val_result_bi_fold_left() {
		use super::bifoldable::explicit::bi_fold_left;
		let x: Result<i32, i32> = Ok(5);
		let y = bi_fold_left::<RcFnBrand, ResultBrand, _, _, _, _, _>(
			(|acc, e| acc - e, |acc, s| acc + s),
			10,
			x,
		);
		assert_eq!(y, 15);
	}

	#[test]
	fn test_ref_result_bi_fold_left() {
		use super::bifoldable::explicit::bi_fold_left;
		let x: Result<i32, i32> = Ok(5);
		let y = bi_fold_left::<RcFnBrand, ResultBrand, _, _, _, _, _>(
			(|acc, e: &i32| acc - *e, |acc, s: &i32| acc + *s),
			10,
			&x,
		);
		assert_eq!(y, 15);
	}

	// -- BiFoldMapDispatch tests --

	#[test]
	fn test_val_result_bi_fold_map() {
		use super::bifoldable::explicit::bi_fold_map;
		let x: Result<i32, i32> = Ok(5);
		let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
			(|e: i32| e.to_string(), |s: i32| s.to_string()),
			x,
		);
		assert_eq!(y, "5".to_string());
	}

	#[test]
	fn test_ref_result_bi_fold_map() {
		use super::bifoldable::explicit::bi_fold_map;
		let x: Result<i32, i32> = Ok(5);
		let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
			(|e: &i32| e.to_string(), |s: &i32| s.to_string()),
			&x,
		);
		assert_eq!(y, "5".to_string());
	}

	// -- BiTraverseDispatch tests --

	#[test]
	fn test_val_result_bi_traverse() {
		use super::bitraversable::explicit::bi_traverse;
		let x: Result<i32, i32> = Ok(5);
		let y = bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
			(|e: i32| Some(e + 1), |s: i32| Some(s * 2)),
			x,
		);
		assert_eq!(y, Some(Ok(10)));
	}

	#[test]
	fn test_ref_result_bi_traverse() {
		use super::bitraversable::explicit::bi_traverse;
		let x: Result<i32, i32> = Ok(5);
		let y = bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
			(|e: &i32| Some(*e + 1), |s: &i32| Some(*s * 2)),
			&x,
		);
		assert_eq!(y, Some(Ok(10)));
	}
}
