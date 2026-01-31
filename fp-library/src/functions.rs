//! Generic, helper free functions and re-exports of free versions
//! of type class functions.
//!
//! This module provides a collection of utility functions commonly found in functional programming,
//! such as function composition, constant functions, and identity functions. It also re-exports
//! free function versions of methods defined in various type classes (traits) for convenience.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let f = |x: i32| x + 1;
//! let g = |x: i32| x * 2;
//! let h = compose::<i32, i32, _, _, _>(f, g);
//!
//! assert_eq!(map::<OptionBrand, _, _, _>(h, Some(5)), Some(11));
//! ```

// Auto-generate re-exports, passing in aliases for conflicting names.
fp_macros::generate_function_re_exports!("src/classes", {
	"category::identity": category_identity,
	"cloneable_fn::new": cloneable_fn_new,
	"function::new": fn_new,
	"pointer::new": pointer_new,
	"ref_counted_pointer::cloneable_new": ref_counted_pointer_new,
	"send_ref_counted_pointer::send_new": send_ref_counted_pointer_new,
	"semigroupoid::compose": semigroupoid_compose,
	"send_cloneable_fn::new": send_cloneable_fn_new,
});

/// Composes two functions.
///
/// Takes two functions, `f` and `g`, and returns a new function that applies `g` to its argument,
/// and then applies `f` to the result. This is equivalent to the mathematical composition `f ∘ g`.
///
/// ### Type Signature
///
/// `forall a c b. (b -> c, a -> b) -> (a -> c)`
///
/// ### Type Parameters
///
/// * `A`: The input type of the inner function `g`.
/// * `C`: The output type of the outer function `f`.
/// * `B`: The output type of `g` and the input type of `f`.
/// * `F`: The type of the outer function.
/// * `G`: The type of the inner function.
///
/// ### Parameters
///
/// * `f`: The outer function to apply second.
/// * `g`: The inner function to apply first.
///
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
/// let times_two_add_one = compose::<i32, i32, _, _, _>(add_one, times_two);
///
/// // 3 * 2 + 1 = 7
/// assert_eq!(
///     times_two_add_one(3),
///     7
/// );
/// ```
pub fn compose<A, C, B, F, G>(
	f: F,
	g: G,
) -> impl Fn(A) -> C
where
	F: Fn(B) -> C,
	G: Fn(A) -> B,
{
	move |a| f(g(a))
}

/// Creates a constant function.
///
/// Returns a function that ignores its argument and always returns the provided value `a`.
/// This is useful when a function is expected but a constant value is needed.
///
/// ### Type Signature
///
/// `forall b a. a -> (b -> a)`
///
/// ### Type Parameters
///
/// * `B`: The type of the argument to ignore.
/// * `A`: The type of the value to return.
///
/// ### Parameters
///
/// * `a`: The value to be returned by the constant function.
///
/// ### Returns
///
/// A function that takes any value of type `B` and returns `a`.
///
/// ### Examples
///
/// ```rust
/// use fp_library::functions::*;
///
/// assert_eq!(
///     constant::<bool, _>(true)(false),
///     true
/// );
/// ```
pub fn constant<B, A: Clone>(a: A) -> impl Fn(B) -> A {
	move |_| a.clone()
}

/// Flips the arguments of a binary function.
///
/// Returns a new function that takes its arguments in the reverse order of the input function `f`.
/// If `f` takes `(a, b)`, the returned function takes `(b, a)`.
///
/// ### Type Signature
///
/// `forall a b c. ((a, b) -> c) -> ((b, a) -> c)`
///
/// ### Type Parameters
///
/// * `A`: The type of the first argument of the input function.
/// * `B`: The type of the second argument of the input function.
/// * `C`: The return type of the function.
/// * `F`: The type of the input binary function.
///
/// ### Parameters
///
/// * `f`: A binary function.
///
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
/// assert_eq!(
///     flip::<i32, i32, _, _>(subtract)(1, 0),
///     -1
/// );
/// ```
pub fn flip<A, B, C, F>(f: F) -> impl Fn(B, A) -> C
where
	F: Fn(A, B) -> C,
{
	move |b, a| f(a, b)
}

/// The identity function.
///
/// Returns its input argument as is. This is often used as a default or placeholder function.
///
/// ### Type Signature
///
/// `forall a. a -> a`
///
/// ### Type Parameters
///
/// * `A`: The type of the value.
///
/// ### Parameters
///
/// * `a`: A value.
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
/// assert_eq!(
///     identity(()),
///     ()
/// );
/// ```
pub fn identity<A>(a: A) -> A {
	a
}
