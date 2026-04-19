//! The primary API for calling type class operations as free functions.
//!
//! This module re-exports inference-enabled dispatch functions from
//! [`dispatch`](crate::dispatch). These functions automatically infer the
//! Brand type parameter from the container argument, so callers do not need
//! a turbofish annotation:
//!
//! ```
//! use fp_library::functions::*;
//!
//! // Brand is inferred as VecBrand from the Vec argument.
//! let result = map(|x: i32| x + 1, vec![1, 2, 3]);
//! assert_eq!(result, vec![2, 3, 4]);
//! ```
//!
//! For cases where Brand inference is ambiguous (e.g., generic contexts),
//! the [`explicit`] submodule provides versions that require a Brand
//! turbofish:
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! };
//!
//! let result = map::<VecBrand, _, _, _, _>(|x: i32| x + 1, vec![1, 2, 3]);
//! assert_eq!(result, vec![2, 3, 4]);
//! ```
//!
//! The module also defines standalone utility functions such as [`compose`],
//! [`constant`], [`flip`], [`identity`], and [`on`].

use fp_macros::*;

pub use crate::dispatch::contravariant::contramap;
// Inference wrappers (from dispatch modules, top-level of each).
pub use crate::dispatch::{
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
	semiapplicative::apply,
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
	"ref_counted_pointer::new": ref_counted_pointer_new,
	"send_ref_counted_pointer::new": send_ref_counted_pointer_new,
	"plus::empty": plus_empty,
	"semigroupoid::compose": semigroupoid_compose,
	"send_clone_fn::new": send_lift_fn_new,
	"send_clone_fn::ref_new": send_ref_lift_fn_new,
	"to_dyn_clone_fn::new": to_dyn_clone_fn,
	"to_dyn_clone_fn::ref_new": to_ref_dyn_clone_fn,
	"to_dyn_send_fn::new": to_dyn_send_fn,
	"to_dyn_send_fn::ref_new": to_ref_dyn_send_fn,
}, exclude {
	// By-value non-dispatch free functions superseded by dispatch versions.
	"semiapplicative::apply",
	"contravariant::contramap",
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
/// Explicit dispatch functions requiring a Brand turbofish.
///
/// For most use cases, prefer the inference-enabled wrappers from the parent
/// [`functions`](crate::functions) module.
pub mod explicit {
	// The class-level apply serves as the explicit fallback (takes
	// FnBrand and Brand via turbofish). The dispatch version infers both.
	pub use crate::{
		classes::semiapplicative::apply,
		dispatch::{
			alt::explicit::alt,
			apply_first::explicit::apply_first,
			apply_second::explicit::apply_second,
			bifoldable::explicit::{
				bi_fold_left,
				bi_fold_map,
				bi_fold_right,
			},
			bifunctor::explicit::bimap,
			bitraversable::explicit::bi_traverse,
			compactable::explicit::{
				compact,
				separate,
			},
			contravariant::explicit::contramap,
			filterable::explicit::{
				filter,
				filter_map,
				partition,
				partition_map,
			},
			filterable_with_index::explicit::{
				filter_map_with_index,
				filter_with_index,
				partition_map_with_index,
				partition_with_index,
			},
			foldable::explicit::{
				fold_left,
				fold_map,
				fold_right,
			},
			foldable_with_index::explicit::{
				fold_left_with_index,
				fold_map_with_index,
				fold_right_with_index,
			},
			functor::explicit::map,
			functor_with_index::explicit::map_with_index,
			lift::explicit::{
				lift2,
				lift3,
				lift4,
				lift5,
			},
			semimonad::explicit::{
				bind,
				bind_flipped,
				join,
			},
			traversable::explicit::traverse,
			traversable_with_index::explicit::traverse_with_index,
			witherable::explicit::{
				wilt,
				wither,
			},
		},
	};
}

// Functions without dispatch wrappers.
pub use crate::dispatch::semimonad::{
	compose_kleisli,
	compose_kleisli_flipped,
};
// Re-exports from other modules.
pub use crate::types::{
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
