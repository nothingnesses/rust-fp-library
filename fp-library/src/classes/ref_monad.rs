//! By-ref monads, combining [`RefApplicative`](crate::classes::RefApplicative) and [`RefSemimonad`](crate::classes::RefSemimonad).
//!
//! This is the by-ref counterpart of [`Monad`](crate::classes::Monad).
//! Enables monadic sequencing where the continuation receives `&A` instead
//! of owned `A`, and value injection clones from `&A`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! // Chain computations on memoized values by reference
//! let lazy = ref_pure::<LazyBrand<RcLazyConfig>, _>(&5);
//! let result = ref_bind::<LazyBrand<RcLazyConfig>, _, _>(lazy, |x: &i32| {
//! 	let v = *x * 2;
//! 	ref_pure::<LazyBrand<RcLazyConfig>, _>(&v)
//! });
//! assert_eq!(*result.evaluate(), 10);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A type class for by-ref monads.
	///
	/// Combines [`RefApplicative`] (by-ref pure + apply) with
	/// [`RefSemimonad`] (by-ref bind).
	///
	/// This is the by-ref counterpart of [`Monad`]. Automatically
	/// implemented for any type implementing both supertraits.
	///
	/// A lawful `RefMonad` must satisfy three laws:
	///
	/// 1. **Left identity**: `ref_bind(ref_pure(&a), f)` evaluates to the
	///    same value as `f(&a)`.
	/// 2. **Right identity**: `ref_bind(m, |x| ref_pure(x))` evaluates to
	///    the same value as `m`.
	/// 3. **Associativity**: `ref_bind(ref_bind(m, f), g)` evaluates to the
	///    same value as `ref_bind(m, |x| ref_bind(f(x), g))`.
	///
	/// These are the standard monad laws expressed with by-ref operations.
	/// Equality is by evaluated value, not structural identity, since
	/// memoized types like [`Lazy`](crate::types::Lazy) create new
	/// allocations on each construction.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let f = |x: &i32| {
	/// 	let v = *x + 1;
	/// 	Lazy::<_, RcLazyConfig>::new(move || v)
	/// };
	/// let g = |x: &i32| {
	/// 	let v = *x * 2;
	/// 	Lazy::<_, RcLazyConfig>::new(move || v)
	/// };
	///
	/// // Left identity: ref_bind(ref_pure(&a), f) = f(&a)
	/// let left =
	/// 	ref_bind::<LazyBrand<RcLazyConfig>, _, _>(ref_pure::<LazyBrand<RcLazyConfig>, _>(&5), f);
	/// assert_eq!(*left.evaluate(), *f(&5).evaluate());
	///
	/// // Right identity: ref_bind(m, |x| ref_pure(x)) = m
	/// let m = RcLazy::pure(42);
	/// let right = ref_bind::<LazyBrand<RcLazyConfig>, _, _>(m.clone(), |x: &i32| {
	/// 	ref_pure::<LazyBrand<RcLazyConfig>, _>(x)
	/// });
	/// assert_eq!(*right.evaluate(), *m.evaluate());
	///
	/// // Associativity: ref_bind(ref_bind(m, f), g) = ref_bind(m, |x| ref_bind(f(x), g))
	/// let m = RcLazy::pure(3);
	/// let lhs = ref_bind::<LazyBrand<RcLazyConfig>, _, _>(
	/// 	ref_bind::<LazyBrand<RcLazyConfig>, _, _>(m.clone(), f),
	/// 	g,
	/// );
	/// let rhs = ref_bind::<LazyBrand<RcLazyConfig>, _, _>(m, |x: &i32| {
	/// 	ref_bind::<LazyBrand<RcLazyConfig>, _, _>(f(x), g)
	/// });
	/// assert_eq!(*lhs.evaluate(), *rhs.evaluate());
	/// ```
	pub trait RefMonad: RefApplicative + RefSemimonad {}

	/// Blanket implementation of [`RefMonad`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> RefMonad for Brand where Brand: RefApplicative + RefSemimonad {}
}

pub use inner::*;
