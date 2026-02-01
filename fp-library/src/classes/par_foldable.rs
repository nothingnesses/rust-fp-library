//! A type class for data structures that can be folded in parallel.
//!
//! **Note: The `rayon` feature must be enabled to use parallel iteration.**
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let v = vec![1, 2, 3, 4, 5];
//! let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
//! let result: String = par_fold_map::<ArcFnBrand, VecBrand, _, _>(f, v);
//! assert_eq!(result, "12345");
//! ```

use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

use super::{foldable::Foldable, monoid::Monoid, send_cloneable_fn::SendCloneableFn};
use crate::{Apply, kinds::*, types::SendEndofunction};

/// A type class for data structures that can be folded in parallel.
///
/// This trait provides parallel versions of `Foldable` operations that require
/// `Send + Sync` bounds on elements and functions. It uses the branded
/// `SendOf` function type to maintain the library's HKT abstraction.
///
/// **Note: The `rayon` feature must be enabled to use parallel iteration.**
///
/// ### Minimal Implementation
///
/// A minimal implementation requires [`ParFoldable::par_fold_map`].
///
/// ### Thread Safety
///
/// All operations in this trait are designed to be safe for parallel execution:
/// - Element type `A` must be `Send + Sync`
/// - Accumulator/result types must be `Send + Sync`
/// - Functions are wrapped in `FnBrand::SendOf` which guarantees `Send + Sync`
///
/// ### Why is `FnBrand` a Trait-Level Parameter?
///
/// Unlike [`Foldable`] where `FnBrand` is a method-level generic parameter
/// and functions are raw `Fn` types wrapped internally, `ParFoldable` takes `FnBrand`
/// at the trait level. This design choice is motivated by:
///
/// 1. **Thread-safe function values as first-class HKT types**: Function parameters
///    like `func` in [`ParFoldable::par_fold_map`] are HKT-applied types via
///    `Apply!(brand: FnBrand, kind: SendCloneableFn, output: SendOf, ...)`. This allows
///    the type system to enforce thread-safety at the API boundary.
///
/// 2. **Guaranteed `Send + Sync` bounds**: The `output: SendOf` in the `Apply!` macro
///    ensures the function type carries `Send + Sync` bounds essential for parallel
///    execution, rather than relying on runtime checks.
///
/// 3. **Default implementation requirements**: The default [`ParFoldable::par_fold_right`]
///    implementation needs to call `<FnBrand as SendCloneableFn>::send_cloneable_fn_new(...)` to
///    create new wrapped functions. Having `FnBrand` at the trait level makes it
///    available throughout the implementation.
///
/// 4. **Multiple implementations per data structure**: With trait-level parameterization,
///    a type can implement `ParFoldable<ArcFnBrand>` and potentially other function
///    brands, allowing callers to choose the appropriate thread-safe function wrapper.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let v = vec![1, 2, 3, 4, 5];
/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
/// let result: String = par_fold_map::<ArcFnBrand, VecBrand, _, _>(f, v);
/// assert_eq!(result, "12345");
/// ```
pub trait ParFoldable: Foldable {
	/// Parallel version of fold_map.
	///
	/// Maps each element to a monoid value using `func`, then combines all values using the monoid's `append` operation. The mapping operations may be executed in parallel.
	///
	/// ### Type Signature
	///
	/// `forall self a m. (ParFoldable self, Monoid m) => (a -> m, self a) -> m`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of thread-safe function to use.",
		"The element type.",
		"The monoid type."
	)]	///
	/// ### Parameters
	///
	#[doc_params(
		"The thread-safe function to map each element to a monoid`.",
		"The foldable structure."
	)]	///
	/// ### Returns
	///
	/// The combined monoid value
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let v = vec![1, 2, 3, 4, 5];
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
	/// let result: String = par_fold_map::<ArcFnBrand, VecBrand, _, _>(f, v);
	/// assert_eq!(result, "12345");
	/// ```
	fn par_fold_map<'a, FnBrand, A, M>(
		func: <FnBrand as SendCloneableFn>::SendOf<'a, A, M>,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		FnBrand: 'a + SendCloneableFn,
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a;

	/// Parallel version of fold_right.
	///
	/// Folds the structure by applying a function from right to left, potentially in parallel.
	///
	/// ### Type Signature
	///
	#[hm_signature(ParFoldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of thread-safe function to use",
		"The element type (must be `Send + Sync`)",
		"The accumulator type (must be `Send + Sync`)"
	)]	///
	/// ### Parameters
	///
	#[doc_params(
		"The thread-safe function to apply to each element and the accumulator.",
		"The initial value of the accumulator.",
		"The structure to fold."
	)]	///
	/// ### Returns
	///
	/// The final accumulator value
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let v = vec![1, 2, 3, 4, 5];
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b)| a + b);
	/// let sum = par_fold_right::<ArcFnBrand, VecBrand, _, _>(f, 10, v);
	/// assert_eq!(sum, 25);
	/// ```
	fn par_fold_right<'a, FnBrand, A, B>(
		func: <FnBrand as SendCloneableFn>::SendOf<'a, (A, B), B>,
		init: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		A: 'a + Clone + Send + Sync,
		B: Send + Sync + 'a,
		FnBrand: 'a + SendCloneableFn,
	{
		let f_clone = func.clone();
		let endo = Self::par_fold_map::<FnBrand, _, _>(
			<FnBrand as SendCloneableFn>::send_cloneable_fn_new(move |a: A| {
				let f_inner = f_clone.clone();
				SendEndofunction::<FnBrand, B>::new(
					<FnBrand as SendCloneableFn>::send_cloneable_fn_new(move |b: B| {
						f_inner((a.clone(), b))
					}),
				)
			}),
			fa,
		);
		endo.0(init)
	}
}

/// Parallel fold_map operation.
///
/// Free function version that dispatches to [the type class' associated function][`ParFoldable::par_fold_map`].
///
/// ### Type Signature
///
#[hm_signature(ParFoldable)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"Undocumented",
	"The brand of thread-safe function to use (must implement `SendCloneableFn`).",
	"The brand of the foldable structure.",
	"The element type (must be `Send + Sync`).",
	"The monoid type (must be `Send + Sync`)."
)]///
/// ### Parameters
///
#[doc_params(
	"The thread-safe function to map each element to a monoid.",
	"The structure to fold."
)]///
/// ### Returns
///
/// The combined monoid value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let v = vec![1, 2, 3, 4, 5];
/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
/// let result: String = par_fold_map::<ArcFnBrand, VecBrand, _, _>(f, v);
/// assert_eq!(result, "12345");
/// ```
pub fn par_fold_map<'a, FnBrand, F, A, M>(
	func: <FnBrand as SendCloneableFn>::SendOf<'a, A, M>,
	fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> M
where
	FnBrand: 'a + SendCloneableFn,
	F: ParFoldable,
	A: 'a + Clone + Send + Sync,
	M: Monoid + Send + Sync + 'a,
{
	F::par_fold_map::<FnBrand, A, M>(func, fa)
}

/// Parallel fold_right operation.
///
/// Free function version that dispatches to [the type class' associated function][`ParFoldable::par_fold_right`].
///
/// ### Type Signature
///
#[hm_signature(ParFoldable)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"Undocumented",
	"The brand of thread-safe function to use.",
	"The brand of the foldable structure.",
	"The element type (must be `Send + Sync`).",
	"The accumulator type (must be `Send + Sync`)."
)]///
/// ### Parameters
///
#[doc_params(
	"The thread-safe function to apply to each element and the accumulator.",
	"The initial value of the accumulator.",
	"The structure to fold."
)]///
/// ### Returns
///
/// The final accumulator value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let v = vec![1, 2, 3, 4, 5];
/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b)| a + b);
/// let sum = par_fold_right::<ArcFnBrand, VecBrand, _, _>(f, 10, v);
/// assert_eq!(sum, 25);
/// ```
pub fn par_fold_right<'a, FnBrand, F, A, B>(
	func: <FnBrand as SendCloneableFn>::SendOf<'a, (A, B), B>,
	initial: B,
	fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> B
where
	FnBrand: SendCloneableFn + 'a,
	F: ParFoldable,
	A: 'a + Clone + Send + Sync,
	B: Send + Sync + 'a,
{
	F::par_fold_right::<FnBrand, A, B>(func, initial, fa)
}
