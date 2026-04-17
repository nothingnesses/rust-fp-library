//! Types that can be mapped over by receiving or returning references to their contents.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! 	types::*,
//! };
//!
//! let memo = Lazy::<_, RcLazyConfig>::new(|| 10);
//! let mapped = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, &memo);
//! assert_eq!(*mapped.evaluate(), 20);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for types that can be mapped over, returning references.
	///
	/// This is a variant of `Functor` for types where `map` receives/returns references.
	/// This is required for types like `Lazy` where `get()` returns `&A`, not `A`.
	///
	/// `RefFunctor` is intentionally independent from
	/// [`SendRefFunctor`](crate::classes::SendRefFunctor). Although one might
	/// expect `SendRefFunctor` to be a subtrait of `RefFunctor`, this is not the case because
	/// `ArcLazy::new` requires `Send` on the closure, which a generic `RefFunctor` cannot
	/// guarantee. As a result, `ArcLazy` implements only `SendRefFunctor`, not `RefFunctor`,
	/// and `RcLazy` implements only `RefFunctor`, not `SendRefFunctor`.
	///
	/// ### Laws
	///
	/// `RefFunctor` instances must satisfy the following laws:
	///
	/// **Identity:** `ref_map(|x| x.clone(), fa)` is equivalent to `fa`, given `A: Clone`.
	/// The `Clone` requirement arises because the mapping function receives `&A` but must
	/// produce a value of type `A` to satisfy the identity law.
	///
	/// **Composition:** `ref_map(|x| g(&f(x)), fa)` is equivalent to
	/// `ref_map(g, ref_map(f, fa))`.
	#[document_examples]
	///
	/// RefFunctor laws for [`Lazy`](crate::types::Lazy):
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::explicit::*,
	/// 	types::*,
	/// };
	///
	/// // Identity: ref_map(|x| x.clone(), fa) evaluates to the same value as fa.
	/// let fa = RcLazy::pure(5);
	/// let mapped = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x, &fa);
	/// assert_eq!(*mapped.evaluate(), *fa.evaluate());
	///
	/// // Composition: ref_map(|x| g(&f(x)), fa) = ref_map(g, ref_map(f, fa))
	/// let f = |x: &i32| *x * 2;
	/// let g = |x: &i32| x + 1;
	/// let fa = RcLazy::pure(5);
	/// let composed = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| g(&f(x)), &fa);
	/// let sequential = map::<LazyBrand<RcLazyConfig>, _, _, _>(
	/// 	g,
	/// 	&map::<LazyBrand<RcLazyConfig>, _, _, _>(f, &fa),
	/// );
	/// assert_eq!(*composed.evaluate(), *sequential.evaluate());
	/// ```
	///
	/// # Cache chain behavior
	///
	/// Chaining `ref_map` calls on memoized types like [`Lazy`](crate::types::Lazy) creates
	/// a linked list of `Rc`/`Arc`-referenced cells. Each mapped value retains a reference to
	/// its predecessor, so the entire chain of predecessor cells stays alive as long as any
	/// downstream mapped value is reachable. Be aware that long chains can accumulate memory
	/// that is only freed when the final value in the chain is dropped.
	///
	/// # Why `Fn` (not `FnOnce`)?
	///
	/// The `func` parameter uses `Fn` rather than `FnOnce` because multi-element
	/// containers like `Vec` call the closure once per element. `FnOnce` would
	/// restrict `RefFunctor` to single-element containers. Closures that move
	/// out of their captures (`FnOnce` but not `Fn`) cannot be used with
	/// `ref_map`; these are rare and can be restructured by extracting the
	/// move into a surrounding scope.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefFunctor {
		/// Maps a function over the values in the functor context, where the function takes a reference.
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
		/// let memo = Lazy::<_, RcLazyConfig>::new(|| 10);
		/// let mapped = LazyBrand::<RcLazyConfig>::ref_map(|x: &i32| *x * 2, &memo);
		/// assert_eq!(*mapped.evaluate(), 20);
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		crate::{
			brands::*,
			functions::explicit,
			types::*,
		},
		quickcheck_macros::quickcheck,
	};

	/// RefFunctor identity law: map(Clone::clone, lazy) evaluates to the same value as lazy.
	#[quickcheck]
	fn prop_ref_functor_identity(x: i32) -> bool {
		let lazy = RcLazy::pure(x);
		let mapped = explicit::map::<LazyBrand<RcLazyConfig>, _, _, _>(|v: &i32| *v, &lazy);
		*mapped.evaluate() == *lazy.evaluate()
	}

	/// RefFunctor composition law: map(|x| g(&f(x)), lazy) == map(g, map(f, lazy)).
	#[quickcheck]
	fn prop_ref_functor_composition(x: i32) -> bool {
		let f = |v: &i32| v.wrapping_mul(2);
		let g = |v: &i32| v.wrapping_add(1);
		let lazy1 = RcLazy::pure(x);
		let lazy2 = RcLazy::pure(x);
		let composed =
			explicit::map::<LazyBrand<RcLazyConfig>, _, _, _>(|v: &i32| g(&f(v)), &lazy1);
		let sequential = explicit::map::<LazyBrand<RcLazyConfig>, _, _, _>(
			g,
			&explicit::map::<LazyBrand<RcLazyConfig>, _, _, _>(f, &lazy2),
		);
		*composed.evaluate() == *sequential.evaluate()
	}
}
