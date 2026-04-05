//! Profunctors that can be closed under exponentiation.
//!
//! A `Closed` profunctor can lift a profunctor to operate on functions.
//! This is the profunctor constraint that characterizes grates.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for closed profunctors.
	///
	/// A `Closed` profunctor can be closed under exponentiation.
	///
	/// The type parameter `FunctionBrand` is the cloneable function brand used to wrap the
	/// input and output functions produced by [`closed`](Self::closed). This
	/// allows callers to choose between `Rc`-backed and `Arc`-backed functions.
	///
	/// ### Hierarchy Unification
	///
	/// This trait inherits from [`Profunctor`].
	#[document_type_parameters("The cloneable function brand.")]
	pub trait Closed<FunctionBrand: CloneFn>: Profunctor {
		/// Lift a profunctor to operate on functions.
		///
		/// This method takes a profunctor `P A B` and returns
		/// `P (FunctionBrand(X, A)) (FunctionBrand(X, B))`, where `FunctionBrand(X, A)` is
		/// the cloneable function type `X -> A` wrapped via `FunctionBrand`.
		///
		/// The `X: Clone` bound is required because implementations need to clone `X`
		/// values inside nested closures. The `B: Clone` bound is required because
		/// some profunctors (notably [`crate::brands::optics::TaggedBrand`]) need to produce
		/// `Fn(X) -> B` from a single `B` value, which requires cloning `B` on each
		/// invocation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the profunctor.",
			"The source type of the profunctor.",
			"The target type of the profunctor.",
			"The input type of the functions."
		)]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_returns("A new profunctor that operates on functions.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::profunctor::*,
		/// };
		///
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let g = <RcFnBrand as Closed<RcFnBrand>>::closed::<i32, i32, String>(f);
		/// // g is now a function: (String -> i32) -> (String -> i32)
		/// let h = std::rc::Rc::new(|s: String| s.len() as i32) as std::rc::Rc<dyn Fn(String) -> i32>;
		/// let result = g(h);
		/// assert_eq!(result("hi".to_string()), 3); // len("hi") + 1 = 3
		/// ```
		fn closed<'a, A: 'a, B: 'a + Clone, X: 'a + Clone>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, <FunctionBrand as CloneFn>::Of<'a, X, A>, <FunctionBrand as CloneFn>::Of<'a, X, B>>);
	}

	/// Lift a profunctor to operate on functions.
	///
	/// Free function version that dispatches to [the type class' associated function][`Closed::closed`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the profunctor.",
		"The brand of the closed profunctor.",
		"The cloneable function brand for wrapping the input/output functions.",
		"The source type of the profunctor.",
		"The target type of the profunctor.",
		"The input type of the functions."
	)]
	///
	#[document_parameters("The profunctor value to transform.")]
	///
	#[document_returns("A new profunctor that operates on functions.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::profunctor::*,
	/// };
	///
	/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
	/// let g = closed::<RcFnBrand, RcFnBrand, i32, i32, String>(f);
	/// // g is now a function: (String -> i32) -> (String -> i32)
	/// let h = std::rc::Rc::new(|s: String| s.len() as i32) as std::rc::Rc<dyn Fn(String) -> i32>;
	/// let result = g(h);
	/// assert_eq!(result("hi".to_string()), 3); // len("hi") + 1 = 3
	/// ```
	pub fn closed<
		'a,
		Brand: Closed<FunctionBrand>,
		FunctionBrand: CloneFn,
		A: 'a,
		B: 'a + Clone,
		X: 'a + Clone,
	>(
		pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, <FunctionBrand as CloneFn>::Of<'a, X, A>, <FunctionBrand as CloneFn>::Of<'a, X, B>>)
	{
		Brand::closed::<A, B, X>(pab)
	}
}

pub use inner::*;
