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
//! assert_eq!(map::<OptionBrand, _, _, _>(h, Some(5)), Some(11));
//! ```

use fp_macros::*;
// Auto-generate re-exports, passing in aliases for conflicting names.
fp_macros::generate_function_re_exports!("src/classes", {
	"category::identity": category_identity,
	"clone_fn::new": lift_fn_new,
	"pointer::new": pointer_new,
	"ref_counted_pointer::cloneable_new": ref_counted_pointer_new,
	"send_ref_counted_pointer::send_new": send_ref_counted_pointer_new,
	"plus::empty": plus_empty,
	"semigroupoid::compose": semigroupoid_compose,
	"send_clone_fn::new": send_lift_fn_new,
});
// Dispatch free functions are in sub-modules not scanned by the macro.
pub use crate::{
	classes::dispatch::{
		bind,
		bind_flipped,
		compose_kleisli,
		compose_kleisli_flipped,
		fold_left,
		fold_map,
		fold_right,
		lift2,
		lift3,
		lift4,
		lift5,
		map,
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
