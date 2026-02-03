//! Reference-counted cloneable function wrappers with [`Semigroupoid`] and [`Category`] instances.
//!
//! Provides the [`FnBrand`] abstraction for wrapping closures in `Rc<dyn Fn>` or `Arc<dyn Fn>` for use in higher-kinded contexts.

use crate::{
	Apply,
	brands::FnBrand,
	classes::{
		Category, CloneableFn, Function, RefCountedPointer, Semigroupoid, SendCloneableFn,
		SendUnsizedCoercible, UnsizedCoercible,
	},
	impl_kind,
	kinds::*,
};
use fp_macros::{doc_params, doc_type_params, hm_signature};

impl_kind! {
	impl<P: UnsizedCoercible> for FnBrand<P> {
		type Of<'a, A, B> = <P as RefCountedPointer>::CloneableOf<dyn 'a + Fn(A) -> B>;
	}
}

impl<P: UnsizedCoercible> Function for FnBrand<P> {
	type Of<'a, A, B> = Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, A, B>);

	/// Creates a new function wrapper.
	///
	/// This function wraps the provided closure `f` into a pointer-wrapped function.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the function and its captured data.",
		"The input type of the function.",
		"The output type of the function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The closure to wrap.", "The input value to the function.")]
	/// ### Returns
	///
	/// The wrapped function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> <Self as Function>::Of<'a, A, B> {
		P::coerce_fn(f)
	}
}

impl<P: UnsizedCoercible> CloneableFn for FnBrand<P> {
	type Of<'a, A, B> = Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, A, B>);

	/// Creates a new cloneable function wrapper.
	///
	/// This function wraps the provided closure `f` into a pointer-wrapped cloneable function.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the function and its captured data.",
		"The input type of the function.",
		"The output type of the function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The closure to wrap.", "The input value to the function.")]
	/// ### Returns
	///
	/// The wrapped cloneable function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> <Self as CloneableFn>::Of<'a, A, B> {
		P::coerce_fn(f)
	}
}

impl<P: UnsizedCoercible> Semigroupoid for FnBrand<P> {
	/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
	///
	/// This method composes two pointer-wrapped functions `f` and `g` to produce a new function that represents the application of `g` followed by `f`.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the morphisms.",
		"The source type of the first morphism.",
		"The target type of the first morphism and the source type of the second morphism.",
		"The target type of the second morphism."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The second morphism to apply (from C to D).",
		"The first morphism to apply (from B to C)."
	)]
	///
	/// ### Returns
	///
	/// The composed morphism (from B to D).
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*};
	///
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// let g = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	/// let h = semigroupoid_compose::<RcFnBrand, _, _, _>(f, g);
	/// assert_eq!(h(5), 12); // (5 + 1) * 2
	/// ```
	fn compose<'a, B: 'a, C: 'a, D: 'a>(
		f: Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, C, D>),
		g: Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, B, C>),
	) -> Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, B, D>) {
		P::coerce_fn(move |b| f(g(b)))
	}
}

impl<P: UnsizedCoercible> Category for FnBrand<P> {
	/// Returns the identity morphism.
	///
	/// The identity morphism is a function that maps every object to itself, wrapped in the pointer type.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The lifetime of the morphism.", "The type of the object.")]
	///
	/// ### Returns
	///
	/// The identity morphism.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let id = category_identity::<RcFnBrand, i32>();
	/// assert_eq!(id(5), 5);
	/// ```
	fn identity<'a, A>() -> Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, A, A>) {
		P::coerce_fn(|a| a)
	}
}

impl<P: SendUnsizedCoercible> SendCloneableFn for FnBrand<P> {
	type SendOf<'a, A, B> = P::SendOf<dyn 'a + Fn(A) -> B + Send + Sync>;

	/// Creates a new thread-safe cloneable function wrapper.
	///
	/// This function wraps the provided closure `f` into a pointer-wrapped thread-safe cloneable function.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the function and its captured data.",
		"The input type of the function.",
		"The output type of the function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The closure to wrap.")]
	/// ### Returns
	///
	/// The wrapped thread-safe cloneable function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn send_cloneable_fn_new<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Self::SendOf<'a, A, B> {
		P::coerce_send_fn(f)
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		brands::*,
		classes::{category::Category, cloneable_fn::CloneableFn, semigroupoid::Semigroupoid},
	};
	use quickcheck_macros::quickcheck;

	// Semigroupoid Laws

	/// Tests the associativity law for Semigroupoid.
	#[quickcheck]
	fn semigroupoid_associativity(x: i32) -> bool {
		let f = <RcFnBrand as CloneableFn>::new(|x: i32| x.wrapping_add(1));
		let g = <RcFnBrand as CloneableFn>::new(|x: i32| x.wrapping_mul(2));
		let h = <RcFnBrand as CloneableFn>::new(|x: i32| x.wrapping_sub(3));

		let lhs = RcFnBrand::compose(f.clone(), RcFnBrand::compose(g.clone(), h.clone()));
		let rhs = RcFnBrand::compose(RcFnBrand::compose(f, g), h);

		lhs(x) == rhs(x)
	}

	// Category Laws

	/// Tests the left identity law for Category.
	#[quickcheck]
	fn category_left_identity(x: i32) -> bool {
		let f = <RcFnBrand as CloneableFn>::new(|x: i32| x.wrapping_add(1));
		let id = RcFnBrand::identity::<i32>();

		let lhs = RcFnBrand::compose(id, f.clone());
		let rhs = f;

		lhs(x) == rhs(x)
	}

	/// Tests the right identity law for Category.
	#[quickcheck]
	fn category_right_identity(x: i32) -> bool {
		let f = <RcFnBrand as CloneableFn>::new(|x: i32| x.wrapping_add(1));
		let id = RcFnBrand::identity::<i32>();

		let lhs = RcFnBrand::compose(f.clone(), id);
		let rhs = f;

		lhs(x) == rhs(x)
	}
}
