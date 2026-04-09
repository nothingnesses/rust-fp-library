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
//! let result = bind::<LazyBrand<RcLazyConfig>, _, _, _, _>(&lazy, |x: &i32| {
//! 	let v = *x * 2;
//! 	ref_pure::<LazyBrand<RcLazyConfig>, _>(&v)
//! });
//! assert_eq!(*result.evaluate(), 10);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
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
	/// 1. **Left identity**: `bind(ref_pure(&a), f)` evaluates to the
	///    same value as `f(&a)`.
	/// 2. **Right identity**: `bind(m, |x| ref_pure(x))` evaluates to
	///    the same value as `m`.
	/// 3. **Associativity**: `bind(bind(m, f), g)` evaluates to the
	///    same value as `bind(m, |x| bind(f(x), g))`.
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
	/// // Left identity: bind(ref_pure(&a), f) = f(&a)
	/// let left =
	/// 	bind::<LazyBrand<RcLazyConfig>, _, _, _, _>(&ref_pure::<LazyBrand<RcLazyConfig>, _>(&5), f);
	/// assert_eq!(*left.evaluate(), *f(&5).evaluate());
	///
	/// // Right identity: bind(m, |x| ref_pure(x)) = m
	/// let m = RcLazy::pure(42);
	/// let right = bind::<LazyBrand<RcLazyConfig>, _, _, _, _>(&m, |x: &i32| {
	/// 	ref_pure::<LazyBrand<RcLazyConfig>, _>(x)
	/// });
	/// assert_eq!(*right.evaluate(), *m.evaluate());
	///
	/// // Associativity: bind(bind(m, f), g) = bind(m, |x| bind(f(x), g))
	/// let m = RcLazy::pure(3);
	/// let lhs = bind::<LazyBrand<RcLazyConfig>, _, _, _, _>(
	/// 	&bind::<LazyBrand<RcLazyConfig>, _, _, _, _>(&m, f),
	/// 	g,
	/// );
	/// let rhs = bind::<LazyBrand<RcLazyConfig>, _, _, _, _>(&m, |x: &i32| {
	/// 	bind::<LazyBrand<RcLazyConfig>, _, _, _, _>(&f(x), g)
	/// });
	/// assert_eq!(*lhs.evaluate(), *rhs.evaluate());
	/// ```
	pub trait RefMonad: RefApplicative + RefSemimonad {}

	/// Blanket implementation of [`RefMonad`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> RefMonad for Brand where Brand: RefApplicative + RefSemimonad {}

	/// Executes a monadic action conditionally, using by-ref bind.
	///
	/// Evaluates the monadic boolean condition by reference, then returns
	/// one of the two branches depending on the result.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computations.",
		"The brand of the monad.",
		"The type of the result."
	)]
	///
	#[document_parameters(
		"The monadic boolean condition.",
		"The value if true.",
		"The value if false."
	)]
	///
	#[document_returns("The selected branch.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let cond = RcLazy::pure(true);
	/// let then_val = RcLazy::pure(1);
	/// let else_val = RcLazy::pure(0);
	/// let result = ref_if_m::<LazyBrand<RcLazyConfig>, _>(&cond, &then_val, &else_val);
	/// assert_eq!(*result.evaluate(), 1);
	/// ```
	pub fn ref_if_m<'a, Brand: RefMonad, A: 'a>(
		cond: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>),
		then_branch: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		else_branch: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
		let then_branch = then_branch.clone();
		let else_branch = else_branch.clone();
		Brand::ref_bind(
			cond,
			move |c: &bool| {
				if *c { then_branch.clone() } else { else_branch.clone() }
			},
		)
	}

	/// Performs a monadic action when a by-ref condition is false.
	///
	/// Evaluates the monadic boolean condition by reference, then executes
	/// the action if the result is `false`, otherwise returns `ref_pure(&())`.
	#[document_signature]
	///
	#[document_type_parameters("The lifetime of the computations.", "The brand of the monad.")]
	///
	#[document_parameters("The monadic boolean condition.", "The action to execute if false.")]
	///
	#[document_returns("The action result, or a pure unit value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let cond = RcLazy::pure(false);
	/// let action = RcLazy::pure(());
	/// let result = ref_unless_m::<LazyBrand<RcLazyConfig>>(&cond, &action);
	/// assert_eq!(*result.evaluate(), ());
	/// ```
	pub fn ref_unless_m<'a, Brand: RefMonad>(
		cond: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>),
		action: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>)
	where
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>): Clone, {
		let action = action.clone();
		Brand::ref_bind(
			cond,
			move |c: &bool| {
				if *c { Brand::ref_pure(&()) } else { action.clone() }
			},
		)
	}
}

pub use inner::*;
