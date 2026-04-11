//! Contains generic, helper free functions and re-exports of free versions
//! of type class functions.
//!
//! This module provides a collection of utility functions commonly found in functional programming,
//! such as function composition, constant functions, and identity functions. It also re-exports
//! free function versions of methods defined in various type classes (traits) for convenience.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let f = |x: i32| x + 1;
//! let g = |x: i32| x * 2;
//! let h = compose(f, g);
//!
//! assert_eq!(map_explicit::<OptionBrand, _, _, _, _>(h, Some(5)), Some(11));
//! ```

/// Brand-inference wrappers for [`Alt`](crate::classes::Alt) operations.
pub mod alt;
/// Brand-inference wrapper for [`apply_first`](mod@crate::dispatch::apply_first).
pub mod apply_first;
/// Brand-inference wrapper for [`apply_second`](mod@crate::dispatch::apply_second).
pub mod apply_second;
/// Brand-inference wrappers for [`Bifoldable`](crate::classes::Bifoldable) operations.
pub mod bifoldable;
/// Brand-inference wrapper for [`Bifunctor::bimap`](crate::classes::Bifunctor::bimap).
pub mod bifunctor;
/// Brand-inference wrapper for [`Bitraversable::bi_traverse`](crate::classes::Bitraversable::bi_traverse).
pub mod bitraversable;
/// Brand-inference wrappers for [`Compactable`](crate::classes::Compactable) operations.
pub mod compactable;
/// Brand-inference wrapper for [`Contravariant::contramap`](crate::classes::Contravariant::contramap).
pub mod contravariant;
/// Brand-inference wrappers for [`Filterable`](crate::classes::Filterable) operations.
pub mod filterable;
/// Brand-inference wrappers for [`FilterableWithIndex`](crate::classes::FilterableWithIndex) operations.
pub mod filterable_with_index;
/// Brand-inference wrappers for [`Foldable`](crate::classes::Foldable) operations.
pub mod foldable;
/// Brand-inference wrappers for [`FoldableWithIndex`](crate::classes::FoldableWithIndex) operations.
pub mod foldable_with_index;
/// Brand-inference wrapper for [`Functor::map`](crate::classes::Functor::map).
pub mod functor;
/// Brand-inference wrapper for [`FunctorWithIndex::map_with_index`](crate::classes::FunctorWithIndex::map_with_index).
pub mod functor_with_index;
/// Brand-inference wrappers for [`lift2`](crate::dispatch::lift::lift2) through [`lift5`](crate::dispatch::lift::lift5).
pub mod lift;
/// Brand-inference wrappers for [`Semimonad`](crate::classes::Semimonad) operations.
pub mod semimonad;
/// Brand-inference wrapper for [`Traversable::traverse`](crate::classes::Traversable::traverse).
pub mod traversable;
/// Brand-inference wrapper for [`TraversableWithIndex::traverse_with_index`](crate::classes::TraversableWithIndex::traverse_with_index).
pub mod traversable_with_index;
/// Brand-inference wrappers for [`Witherable`](crate::classes::Witherable) operations.
pub mod witherable;

use fp_macros::*;

pub use self::{
	alt::alt,
	apply_first::apply_first,
	apply_second::apply_second,
	bifoldable::{
		bi_fold_left,
		bi_fold_map,
		bi_fold_right,
	},
	bifunctor::bimap,
	bitraversable::bi_traverse,
	compactable::{
		compact,
		separate,
	},
	contravariant::contramap,
	filterable::{
		filter,
		filter_map,
		partition,
		partition_map,
	},
	filterable_with_index::{
		filter_map_with_index,
		filter_with_index,
		partition_map_with_index,
		partition_with_index,
	},
	foldable::{
		fold_left,
		fold_map,
		fold_right,
	},
	foldable_with_index::{
		fold_left_with_index,
		fold_map_with_index,
		fold_right_with_index,
	},
	functor::map,
	functor_with_index::map_with_index,
	lift::{
		lift2,
		lift3,
		lift4,
		lift5,
	},
	semimonad::{
		bind,
		bind_flipped,
		join,
	},
	traversable::traverse,
	traversable_with_index::traverse_with_index,
	witherable::{
		wilt,
		wither,
	},
};
// Auto-generate re-exports, passing in aliases for conflicting names.
fp_macros::generate_function_re_exports!("src/classes", {
	"category::identity": category_identity,
	"clone_fn::new": lift_fn_new,
	"clone_fn::ref_new": ref_lift_fn_new,
	"pointer::new": pointer_new,
	"ref_counted_pointer::cloneable_new": ref_counted_pointer_new,
	"send_ref_counted_pointer::send_new": send_ref_counted_pointer_new,
	"plus::empty": plus_empty,
	"semigroupoid::compose": semigroupoid_compose,
	"send_clone_fn::new": send_lift_fn_new,
	"send_clone_fn::ref_new": send_ref_lift_fn_new,
	"contravariant::contramap": contramap_explicit,
}, exclude {
	// By-value non-dispatch free functions superseded by dispatch versions.
	"alt::alt",
	"apply_first::apply_first",
	"apply_second::apply_second",
	"bifoldable::bi_fold_left",
	"bifoldable::bi_fold_map",
	"bifoldable::bi_fold_right",
	"bifunctor::bimap",
	"bitraversable::bi_traverse",
	"compactable::compact",
	"compactable::separate",
	"filterable::filter",
	"filterable::filter_map",
	"filterable::partition",
	"filterable::partition_map",
	"filterable_with_index::filter_map_with_index",
	"filterable_with_index::filter_with_index",
	"filterable_with_index::partition_map_with_index",
	"filterable_with_index::partition_with_index",
	"foldable_with_index::fold_left_with_index",
	"foldable_with_index::fold_map_with_index",
	"foldable_with_index::fold_right_with_index",
	"functor_with_index::map_with_index",
	"semimonad::join",
	"traversable::traverse",
	"traversable_with_index::traverse_with_index",
	"witherable::wilt",
	"witherable::wither",
	// By-ref non-dispatch free functions superseded by dispatch versions.
	"ref_alt::ref_alt",
	"ref_apply_first::ref_apply_first",
	"ref_apply_second::ref_apply_second",
	"ref_bifunctor::ref_bimap",
	"ref_bifoldable::ref_bi_fold_left",
	"ref_bifoldable::ref_bi_fold_map",
	"ref_bifoldable::ref_bi_fold_right",
	"ref_bitraversable::ref_bi_traverse",
	"ref_compactable::ref_compact",
	"ref_compactable::ref_separate",
	"ref_filterable::ref_filter",
	"ref_filterable::ref_filter_map",
	"ref_filterable::ref_partition",
	"ref_filterable::ref_partition_map",
	"ref_filterable_with_index::ref_filter_with_index",
	"ref_filterable_with_index::ref_filter_map_with_index",
	"ref_filterable_with_index::ref_partition_with_index",
	"ref_filterable_with_index::ref_partition_map_with_index",
	"ref_foldable_with_index::ref_fold_left_with_index",
	"ref_foldable_with_index::ref_fold_map_with_index",
	"ref_foldable_with_index::ref_fold_right_with_index",
	"ref_functor_with_index::ref_map_with_index",
	"ref_semimonad::ref_join",
	"ref_traversable_with_index::ref_traverse_with_index",
	"ref_witherable::ref_wilt",
	"ref_witherable::ref_wither",
});
// Dispatch free functions are in sub-modules not scanned by the macro.
pub use crate::{
	dispatch::{
		alt::alt as alt_explicit,
		apply_first::apply_first as apply_first_explicit,
		apply_second::apply_second as apply_second_explicit,
		bi_fold_left as bi_fold_left_explicit,
		bi_fold_map as bi_fold_map_explicit,
		bi_fold_right as bi_fold_right_explicit,
		bi_traverse as bi_traverse_explicit,
		bimap as bimap_explicit,
		bind as bind_explicit,
		bind_flipped as bind_flipped_explicit,
		compactable::{
			compact as compact_explicit,
			separate as separate_explicit,
		},
		compose_kleisli,
		compose_kleisli_flipped,
		filter as filter_explicit,
		filter_map as filter_map_explicit,
		filter_map_with_index as filter_map_with_index_explicit,
		filter_with_index as filter_with_index_explicit,
		fold_left as fold_left_explicit,
		fold_left_with_index as fold_left_with_index_explicit,
		fold_map as fold_map_explicit,
		fold_map_with_index as fold_map_with_index_explicit,
		fold_right as fold_right_explicit,
		fold_right_with_index as fold_right_with_index_explicit,
		lift2 as lift2_explicit,
		lift3 as lift3_explicit,
		lift4 as lift4_explicit,
		lift5 as lift5_explicit,
		map as map_explicit,
		map_with_index as map_with_index_explicit,
		partition as partition_explicit,
		partition_map as partition_map_explicit,
		partition_map_with_index as partition_map_with_index_explicit,
		partition_with_index as partition_with_index_explicit,
		semimonad::join as join_explicit,
		traverse as traverse_explicit,
		traverse_with_index as traverse_with_index_explicit,
		wilt as wilt_explicit,
		wither as wither_explicit,
	},
	types::{
		lazy::{
			arc_lazy_fix,
			rc_lazy_fix,
		},
		optics::{
			optics_as_index,
			optics_compose,
			optics_indexed_fold_map,
			optics_indexed_over,
			optics_indexed_preview,
			optics_indexed_set,
			optics_indexed_view,
			optics_reindexed,
			optics_un_index,
			positions,
		},
	},
};

/// Composes two functions.
///
/// Takes two functions, `f` and `g`, and returns a new function that applies `g` to its argument,
/// and then applies `f` to the result. This is equivalent to the mathematical composition `f ∘ g`.
#[document_signature]
///
#[document_type_parameters(
	"The input type of the inner function `g`.",
	"The output type of `g` and the input type of `f`.",
	"The output type of the outer function `f`."
)]
///
#[document_parameters(
	"The outer function to apply second.",
	"The inner function to apply first.",
	"The argument to be passed to the composed function."
)]
/// ### Returns
///
/// A new function that takes an `A` and returns a `C`.
///
/// ### Examples
///
/// ```rust
/// use fp_library::functions::*;
///
/// let add_one = |x: i32| x + 1;
/// let times_two = |x: i32| x * 2;
/// let times_two_add_one = compose(add_one, times_two);
///
/// // 3 * 2 + 1 = 7
/// assert_eq!(times_two_add_one(3), 7);
/// ```
pub fn compose<A, B, C>(
	f: impl Fn(B) -> C,
	g: impl Fn(A) -> B,
) -> impl Fn(A) -> C {
	move |a| f(g(a))
}

/// Creates a constant function.
///
/// Returns a function that ignores its argument and always returns the provided value `a`.
/// This is useful when a function is expected but a constant value is needed.
#[document_signature]
///
#[document_type_parameters(
	"The type of the value to return.",
	"The type of the argument to ignore."
)]
///
#[document_parameters(
	"The value to be returned by the constant function.",
	"The argument to be ignored."
)]
/// ### Returns
///
/// The first parameter.
///
/// ### Examples
///
/// ```rust
/// use fp_library::functions::*;
///
/// assert_eq!(constant(true, false), true);
/// ```
pub fn constant<A: Clone, B>(
	a: A,
	_b: B,
) -> A {
	a
}

/// Flips the arguments of a binary function.
///
/// Returns a new function that takes its arguments in the reverse order of the input function `f`.
/// If `f` takes `(a, b)`, the returned function takes `(b, a)`.
#[document_signature]
///
#[document_type_parameters(
	"The type of the first argument of the input function.",
	"The type of the second argument of the input function.",
	"The return type of the function."
)]
///
#[document_parameters(
	"A binary function.",
	"The second argument (which will be passed as the first to `f`).",
	"The first argument (which will be passed as the second to `f`)."
)]
/// ### Returns
///
/// A version of `f` that takes its arguments in reverse.
///
/// ### Examples
///
/// ```rust
/// use fp_library::functions::*;
///
/// let subtract = |a, b| a - b;
///
/// // 0 - 1 = -1
/// assert_eq!(flip(subtract)(1, 0), -1);
/// ```
pub fn flip<A, B, C>(f: impl Fn(A, B) -> C) -> impl Fn(B, A) -> C {
	move |b, a| f(a, b)
}

/// The identity function.
///
/// Returns its input argument as is. This is often used as a default or placeholder function.
#[document_signature]
///
#[document_type_parameters("The type of the value.")]
///
#[document_parameters("A value.")]
///
/// ### Returns
///
/// The same value `a`.
///
/// ### Examples
///
/// ```rust
/// use fp_library::functions::*;
///
/// assert_eq!(identity(()), ());
/// ```
pub fn identity<A>(a: A) -> A {
	a
}

/// Applies a binary function after projecting both arguments through a common function.
///
/// `on(f, g, x, y)` computes `f(g(x), g(y))`. This is useful for changing the domain
/// of a binary operation.
#[document_signature]
///
#[document_type_parameters(
	"The type of the original arguments.",
	"The type of the projected arguments.",
	"The result type."
)]
///
#[document_parameters(
	"The binary function to apply to the projected values.",
	"The projection function applied to both arguments.",
	"The first argument.",
	"The second argument."
)]
///
#[document_returns("The result of applying `f` to the projected values.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// // Compare by absolute value
/// let max_by_abs = on(|a: i32, b: i32| a.max(b), |x: i32| x.abs(), -5, 3);
/// assert_eq!(max_by_abs, 5);
///
/// // Sum the lengths of two strings
/// let sum_lens = on(|a: usize, b: usize| a + b, |s: &str| s.len(), "hello", "hi");
/// assert_eq!(sum_lens, 7);
/// ```
pub fn on<A, B, C>(
	f: impl Fn(B, B) -> C,
	g: impl Fn(A) -> B,
	x: A,
	y: A,
) -> C {
	f(g(x), g(y))
}
