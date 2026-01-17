//! Parallel folding operations.
//!
//! This module defines the [`ParFoldable`] trait, which provides parallel versions of `Foldable` operations.

use super::{foldable::Foldable, monoid::Monoid, send_clonable_fn::SendClonableFn};
use crate::{Apply, kinds::*, types::SendEndofunction};

/// A type class for structures that can be folded in parallel.
///
/// This trait provides parallel versions of `Foldable` operations that require
/// `Send + Sync` bounds on elements and functions. It uses the branded
/// `SendOf` function type to maintain the library's HKT abstraction.
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
///    `Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, ...)`. This allows
///    the type system to enforce thread-safety at the API boundary.
///
/// 2. **Guaranteed `Send + Sync` bounds**: The `output: SendOf` in the `Apply!` macro
///    ensures the function type carries `Send + Sync` bounds essential for parallel
///    execution, rather than relying on runtime checks.
///
/// 3. **Default implementation requirements**: The default [`ParFoldable::par_fold_right`]
///    implementation needs to call `<FnBrand as SendClonableFn>::new_send(...)` to
///    create new wrapped functions. Having `FnBrand` at the trait level makes it
///    available throughout the implementation.
///
/// 4. **Multiple implementations per data structure**: With trait-level parameterization,
///    a type can implement `ParFoldable<ArcFnBrand>` and potentially other function
///    brands, allowing callers to choose the appropriate thread-safe function wrapper.
///
/// ### Examples
///
/// ```ignore
/// use fp_library::classes::par_foldable::ParFoldable;
/// use fp_library::brands::{VecBrand, ArcFnBrand};
/// use fp_library::classes::send_clonable_fn::SendClonableFn;
///
/// let v = vec![1, 2, 3, 4, 5];
/// let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x as i64);
/// let sum: i64 = VecBrand::par_fold_map::<ArcFnBrand, _, _>(v, f);
/// ```
pub trait ParFoldable<FnBrand: SendClonableFn>: Foldable {
	/// Parallel version of fold_map.
	///
	/// Maps each element to a monoid value using `func`, then combines all values using the monoid's `append` operation. The mapping operations may be executed in parallel.
	///
	/// ### Type Signature
	///
	/// `forall a m. (ParFoldable t, Monoid m, Send m, Sync m) => (f a m, t a) -> m`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of thread-safe function to use (must implement `SendClonableFn`).
	/// * `A`: The element type (must be `Send + Sync`).
	/// * `M`: The monoid type (must be `Send + Sync`).
	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to map each element to a monoid`.
	/// * `fa`: The foldable structure.
	///
	/// ### Returns
	///
	/// The combined monoid value
	fn par_fold_map<'a, A, M>(
		func: <FnBrand as SendClonableFn>::SendOf<'a, A, M>,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a;

	/// Parallel version of fold_right.
	///
	/// Folds the structure by applying a function from right to left, potentially in parallel.
	///
	/// ### Type Signature
	///
	/// `forall a b. ParFoldable t => (f (a, b) b, b, t a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of thread-safe function to use
	/// * `A`: The element type (must be `Send + Sync`)
	/// * `B`: The accumulator type (must be `Send + Sync`)
	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to apply to each element and the accumulator.
	/// * `initial`: The initial value of the accumulator.
	/// * `fa`: The structure to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value
	fn par_fold_right<'a, A, B>(
		func: <FnBrand as SendClonableFn>::SendOf<'a, (A, B), B>,
		init: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		A: 'a + Clone + Send + Sync,
		B: Send + Sync + 'a,
		FnBrand: 'a,
	{
		let f_clone = func.clone();
		let endo = Self::par_fold_map(
			<FnBrand as SendClonableFn>::new_send(move |a: A| {
				let f_inner = f_clone.clone();
				SendEndofunction::<FnBrand, B>::new(<FnBrand as SendClonableFn>::new_send(
					move |b: B| f_inner((a.clone(), b)),
				))
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
/// `forall a m. (ParFoldable t, Monoid m, Send m, Sync m) => (f a m, t a) -> m`
///
/// ### Type Parameters
///
/// * `FnBrand`: The brand of thread-safe function to use (must implement `SendClonableFn`)
/// * `Brand`: The brand of the foldable structure
/// * `A`: The element type (must be `Send + Sync`)
/// * `M`: The monoid type (must be `Send + Sync`)
///
/// ### Parameters
///
/// * `func`: The thread-safe function to map each element to a monoid.
/// * `fa`: The structure to fold.
///
/// ### Returns
///
/// The combined monoid value
pub fn par_fold_map<'a, FnBrand, Brand, A, M>(
	func: <FnBrand as SendClonableFn>::SendOf<'a, A, M>,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> M
where
	FnBrand: SendClonableFn,
	Brand: ParFoldable<FnBrand>,
	A: 'a + Clone + Send + Sync,
	M: Monoid + Send + Sync + 'a,
{
	Brand::par_fold_map(func, fa)
}

/// Parallel fold_right operation.
///
/// Free function version that dispatches to [the type class' associated function][`ParFoldable::par_fold_right`].
///
/// ### Type Signature
///
/// `forall a b. ParFoldable t => (f (a, b) b, b, t a) -> b`
///
/// ### Type Parameters
///
/// * `FnBrand`: The brand of thread-safe function to use
/// * `Brand`: The brand of the foldable structure
/// * `A`: The element type (must be `Send + Sync`)
/// * `B`: The accumulator type (must be `Send + Sync`)
///
/// ### Parameters
///
/// * `func`: The thread-safe function to apply to each element and the accumulator.
/// * `init`: The initial value of the accumulator.
/// * `fa`: The structure to fold.
///
/// ### Returns
///
/// The final accumulator value
pub fn par_fold_right<'a, FnBrand, Brand, A, B>(
	func: <FnBrand as SendClonableFn>::SendOf<'a, (A, B), B>,
	init: B,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> B
where
	FnBrand: SendClonableFn,
	Brand: ParFoldable<FnBrand>,
	A: 'a + Clone + Send + Sync,
	B: Send + Sync + 'a,
	FnBrand: 'a,
{
	Brand::par_fold_right(func, init, fa)
}
