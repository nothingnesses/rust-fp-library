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
/// # Minimal Implementation
///
/// A minimal implementation requires [`ParFoldable::par_fold_map`].
///
/// # Thread Safety
///
/// All operations in this trait are designed to be safe for parallel execution:
/// - Element type `A` must be `Send + Sync`
/// - Accumulator/result types must be `Send + Sync`
/// - Functions are wrapped in `FnBrand::SendOf` which guarantees `Send + Sync`
///
/// # Examples
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
	/// Maps each element to a monoid value using `f`, then combines all values using the monoid's `append` operation. The mapping operations may be executed in parallel.
	///
	/// ### Type Signature
	///
	/// `forall a m. (ParFoldable t, Monoid m, Send m, Sync m) => (f a m, t a) -> m`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of thread-safe function to use (must implement `SendClonableFn`)
	/// * `A`: The element type (must be `Send + Sync`)
	/// * `M`: The monoid type (must be `Send + Sync`)
	///
	/// ### Parameters
	///
	/// * `fa`: The foldable structure
	/// * `f`: The mapping function wrapped using `Apply!` with `output: SendOf`
	///
	/// ### Returns
	///
	/// The combined monoid value
	fn par_fold_map<'a, A, M>(
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
		f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, M)),
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
	/// * `f`: The folding function wrapped using `Apply!` with `output: SendOf`
	/// * `init`: The initial accumulator value
	/// * `fa`: The foldable structure
	///
	/// ### Returns
	///
	/// The final accumulator value
	fn par_fold_right<'a, A, B>(
		f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: ((A, B), B)),
		init: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		A: 'a + Clone + Send + Sync,
		B: Send + Sync + 'a,
		FnBrand: 'a,
	{
		let f_clone = f.clone();
		let endo = Self::par_fold_map(
			fa,
			<FnBrand as SendClonableFn>::new_send(move |a: A| {
				let f_inner = f_clone.clone();
				SendEndofunction::<FnBrand, B>::new(<FnBrand as SendClonableFn>::new_send(
					move |b: B| f_inner((a.clone(), b)),
				))
			}),
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
/// * `fa`: The foldable structure
/// * `f`: The mapping function wrapped using `Apply!` with `output: SendOf`
///
/// ### Returns
///
/// The combined monoid value
pub fn par_fold_map<'a, FnBrand, Brand, A, M>(
	fa: Apply!(brand: Brand, signature: ('a, A: 'a) -> 'a),
	f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, M)),
) -> M
where
	FnBrand: SendClonableFn,
	Brand: ParFoldable<FnBrand>,
	A: 'a + Clone + Send + Sync,
	M: Monoid + Send + Sync + 'a,
{
	Brand::par_fold_map(fa, f)
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
/// * `f`: The folding function wrapped using `Apply!` with `output: SendOf`
/// * `init`: The initial accumulator value
/// * `fa`: The foldable structure
///
/// ### Returns
///
/// The final accumulator value
pub fn par_fold_right<'a, FnBrand, Brand, A, B>(
	f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: ((A, B), B)),
	init: B,
	fa: Apply!(brand: Brand, signature: ('a, A: 'a) -> 'a),
) -> B
where
	FnBrand: SendClonableFn,
	Brand: ParFoldable<FnBrand>,
	A: 'a + Clone + Send + Sync,
	B: Send + Sync + 'a,
	FnBrand: 'a,
{
	Brand::par_fold_right(f, init, fa)
}
