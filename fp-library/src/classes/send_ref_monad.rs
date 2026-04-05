//! Thread-safe by-ref monads, combining [`SendRefApplicative`](crate::classes::SendRefApplicative) and [`SendRefSemimonad`](crate::classes::SendRefSemimonad).
//!
//! This is the thread-safe counterpart of [`RefMonad`](crate::classes::RefMonad).

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A type that supports thread-safe by-ref function application, pure value
	/// injection, and monadic sequencing via references.
	///
	/// This is the thread-safe counterpart of [`RefMonad`].
	/// Automatically implemented for any type implementing both
	/// [`SendRefApplicative`] and [`SendRefSemimonad`].
	///
	/// A lawful `SendRefMonad` must satisfy the same three monad laws as
	/// [`RefMonad`], with equality by evaluated value:
	///
	/// 1. **Left identity**: `send_ref_bind(send_ref_pure(&a), f)` evaluates
	///    to the same value as `f(&a)`.
	/// 2. **Right identity**: `send_ref_bind(m, |x| send_ref_pure(x))` evaluates
	///    to the same value as `m`.
	/// 3. **Associativity**: `send_ref_bind(send_ref_bind(m, f), g)` evaluates
	///    to the same value as `send_ref_bind(m, |x| send_ref_bind(f(x), g))`.
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
	/// 	ArcLazy::new(move || v)
	/// };
	/// let g = |x: &i32| {
	/// 	let v = *x * 2;
	/// 	ArcLazy::new(move || v)
	/// };
	///
	/// // Left identity
	/// let left = send_ref_bind::<LazyBrand<ArcLazyConfig>, _, _>(
	/// 	send_ref_pure::<LazyBrand<ArcLazyConfig>, _>(&5),
	/// 	f,
	/// );
	/// assert_eq!(*left.evaluate(), *f(&5).evaluate());
	///
	/// // Right identity
	/// let m = ArcLazy::new(|| 42);
	/// let right = send_ref_bind::<LazyBrand<ArcLazyConfig>, _, _>(m.clone(), |x: &i32| {
	/// 	send_ref_pure::<LazyBrand<ArcLazyConfig>, _>(x)
	/// });
	/// assert_eq!(*right.evaluate(), *m.evaluate());
	///
	/// // Associativity
	/// let m = ArcLazy::new(|| 3);
	/// let lhs = send_ref_bind::<LazyBrand<ArcLazyConfig>, _, _>(
	/// 	send_ref_bind::<LazyBrand<ArcLazyConfig>, _, _>(m.clone(), f),
	/// 	g,
	/// );
	/// let rhs = send_ref_bind::<LazyBrand<ArcLazyConfig>, _, _>(m, |x: &i32| {
	/// 	send_ref_bind::<LazyBrand<ArcLazyConfig>, _, _>(f(x), g)
	/// });
	/// assert_eq!(*lhs.evaluate(), *rhs.evaluate());
	/// ```
	pub trait SendRefMonad: SendRefApplicative + SendRefSemimonad {}

	/// Blanket implementation of [`SendRefMonad`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> SendRefMonad for Brand where Brand: SendRefApplicative + SendRefSemimonad {}
}

pub use inner::*;
