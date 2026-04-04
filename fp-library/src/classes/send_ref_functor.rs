//! Types that can be mapped over by receiving references to their contents,
//! with thread-safe mapping functions.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! let memo = ArcLazy::new(|| 10);
//! let mapped = send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(|x: &i32| *x * 2, memo);
//! assert_eq!(*mapped.evaluate(), 20);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for types that can be mapped over, receiving references, with thread-safe functions.
	///
	/// This is a variant of [`RefFunctor`](crate::classes::RefFunctor) where the mapping function
	/// must be `Send`, making it suitable for thread-safe lazy types like
	/// [`ArcLazy`](crate::types::ArcLazy).
	///
	/// ### Why a Separate Trait?
	///
	/// A single trait with `Send` bounds on `RefFunctor` would exclude `RcLazy`, which uses
	/// `Rc` (a `!Send` type). By keeping `RefFunctor` free of `Send` bounds and providing
	/// `SendRefFunctor` separately, `RcLazy` can implement `RefFunctor` while `ArcLazy`
	/// implements only `SendRefFunctor`.
	///
	/// ### Laws
	///
	/// `SendRefFunctor` instances must satisfy the following laws:
	/// * Identity: `send_ref_map(|x| x.clone(), fa)` evaluates to a value equal to `fa`'s evaluated value.
	/// * Composition: `send_ref_map(|x| f(&g(x)), fa)` evaluates to the same value as `send_ref_map(f, send_ref_map(g, fa))`.
	#[document_examples]
	///
	/// SendRefFunctor laws for [`ArcLazy`](crate::types::ArcLazy):
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// // Identity: send_ref_map(|x| x.clone(), fa) evaluates to the same value as fa.
	/// let fa = ArcLazy::pure(5);
	/// let mapped = send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(|x: &i32| *x, fa.clone());
	/// assert_eq!(*mapped.evaluate(), *fa.evaluate());
	///
	/// // Composition: send_ref_map(|x| f(&g(x)), fa) = send_ref_map(f, send_ref_map(g, fa))
	/// let f = |x: &i32| x + 1;
	/// let g = |x: &i32| *x * 2;
	/// let fa = ArcLazy::pure(5);
	/// let composed = send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(|x: &i32| f(&g(x)), fa.clone());
	/// let sequential = send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(
	/// 	f,
	/// 	send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(g, fa),
	/// );
	/// assert_eq!(*composed.evaluate(), *sequential.evaluate());
	/// ```
	///
	/// # Cache chain behavior
	///
	/// Chaining `send_ref_map` calls on memoized types like [`ArcLazy`](crate::types::ArcLazy)
	/// creates a linked list of `Arc`-referenced cells. Each mapped value retains a reference to
	/// its predecessor, so the entire chain of predecessor cells stays alive as long as any
	/// downstream mapped value is reachable. Be aware that long chains can accumulate memory
	/// that is only freed when the final value in the chain is dropped.
	///
	/// # Why `FnOnce`?
	///
	/// The `func` parameter uses `FnOnce` rather than `Fn` because memoized types like
	/// `ArcLazy` create a new `ArcLazy` value capturing the closure. Since the resulting
	/// `ArcLazy` will evaluate the closure at most once, `FnOnce` is sufficient and avoids
	/// imposing unnecessary `Clone` or multi-call constraints on the caller.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendRefFunctor {
		/// Maps a thread-safe function over the values in the functor context, where the function takes a reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the functor.",
			"The type of the result(s) of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to the value(s) inside the functor.",
			"The functor instance containing the value(s)."
		)]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let memo = ArcLazy::new(|| 10);
		/// let mapped = LazyBrand::<ArcLazyConfig>::send_ref_map(|x: &i32| *x * 2, memo);
		/// assert_eq!(*mapped.evaluate(), 20);
		/// ```
		fn send_ref_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			func: impl Fn(&A) -> B + Send + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Maps a thread-safe function over the values in the functor context, where the function takes a reference.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendRefFunctor::send_ref_map`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function."
	)]
	///
	#[document_parameters(
		"The function to apply to the value(s) inside the functor.",
		"The functor instance containing the value(s)."
	)]
	///
	#[document_returns("A new functor instance containing the result(s) of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let memo = ArcLazy::new(|| 10);
	/// let mapped = send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(|x: &i32| *x * 2, memo);
	/// assert_eq!(*mapped.evaluate(), 20);
	/// ```
	pub fn send_ref_map<'a, Brand: SendRefFunctor, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		func: impl Fn(&A) -> B + Send + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::send_ref_map(func, fa)
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		crate::{
			brands::*,
			functions::*,
			types::*,
		},
		quickcheck_macros::quickcheck,
	};

	/// SendRefFunctor identity law: send_ref_map(Clone::clone, lazy) evaluates to the same value as lazy.
	#[quickcheck]
	fn prop_send_ref_functor_identity(x: i32) -> bool {
		let lazy = ArcLazy::pure(x);
		let mapped = send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(|v: &i32| *v, lazy.clone());
		*mapped.evaluate() == *lazy.evaluate()
	}

	/// SendRefFunctor composition law: send_ref_map(|x| g(&f(x)), lazy) == send_ref_map(g, send_ref_map(f, lazy)).
	#[quickcheck]
	fn prop_send_ref_functor_composition(x: i32) -> bool {
		let f = |v: &i32| v.wrapping_mul(2);
		let g = |v: &i32| v.wrapping_add(1);
		let lazy1 = ArcLazy::pure(x);
		let lazy2 = ArcLazy::pure(x);
		let composed = send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(|v: &i32| g(&f(v)), lazy1);
		let sequential = send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(
			g,
			send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(f, lazy2),
		);
		*composed.evaluate() == *sequential.evaluate()
	}
}
