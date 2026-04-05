//! By-ref applicative functors, combining [`RefPointed`](crate::classes::RefPointed) and [`RefSemiapplicative`](crate::classes::RefSemiapplicative).
//!
//! This is the by-ref counterpart of [`Applicative`](crate::classes::Applicative).
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
//! // ref_pure clones a value into a Lazy context
//! let x = ref_pure::<LazyBrand<RcLazyConfig>, _>(&42);
//! assert_eq!(*x.evaluate(), 42);
//!
//! // ref_apply applies a wrapped by-ref function
//! let f = RcLazy::pure(std::rc::Rc::new(|x: &i32| *x + 1) as std::rc::Rc<dyn Fn(&i32) -> i32>);
//! let result = ref_apply::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _>(f, x);
//! assert_eq!(*result.evaluate(), 43);
//! ```

#[fp_macros::document_module]
mod inner {
	use crate::classes::*;

	/// A type class for by-ref applicative functors.
	///
	/// Combines [`RefPointed`] (injecting values from references) with
	/// [`RefSemiapplicative`] (applying wrapped by-ref functions).
	///
	/// This is the by-ref counterpart of [`Applicative`]. Automatically
	/// implemented for any type implementing both supertraits.
	///
	/// A lawful `RefApplicative` must satisfy the applicative laws
	/// (identity, composition, homomorphism, interchange) expressed
	/// in terms of `ref_pure` and `ref_apply`. These mirror the
	/// standard [`Applicative`] laws but with by-ref function wrappers:
	///
	/// 1. **Identity**: `ref_apply(ref_pure(&id), v) = v` (evaluated values equal).
	/// 2. **Homomorphism**: `ref_apply(ref_pure(&f), ref_pure(&x)) = ref_pure(&f(&x))`.
	/// 3. **Interchange**: `ref_apply(u, ref_pure(&y)) = ref_apply(ref_pure(&(|f| f(&y))), u)`.
	pub trait RefApplicative:
		RefPointed + RefSemiapplicative + RefApplyFirst + RefApplySecond {
	}

	/// Blanket implementation of [`RefApplicative`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> RefApplicative for Brand where
		Brand: RefPointed + RefSemiapplicative + RefApplyFirst + RefApplySecond
	{
	}
}

pub use inner::*;
