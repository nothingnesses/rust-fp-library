//! Strong profunctors, which can lift profunctors through product types.
//!
//! A strong profunctor allows lifting a profunctor `P A B` to `P (A, C) (B, C)`,
//! preserving the extra context `C`. This is the key constraint for lenses.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::profunctor::*,
//! 	functions::*,
//! };
//!
//! // Functions are strong profunctors
//! let f = |x: i32| x + 1;
//! let g = first::<RcFnBrand, _, _, i32>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
//! assert_eq!(g((10, 20)), (11, 20));
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

	/// A type class for strong profunctors.
	///
	/// A strong profunctor can lift a profunctor through product types (tuples).
	/// This is the profunctor constraint that characterizes lenses.
	///
	/// ### Hierarchy Unification
	///
	/// This trait uses the strict Kind signature from [`Kind!(type Of<'a, A: 'a, B: 'a>: 'a;)`](crate::kinds::Kind_266801a817966495). This ensures
	/// that when lifting a profunctor, the secondary component of the product type (the context)
	/// correctly satisfies lifetime requirements relative to the profunctor's application.
	///
	/// ### Laws
	///
	/// `Strong` instances must satisfy the following laws:
	/// * Identity: `first(identity) = identity`.
	/// * Composition: `first(p ∘ q) = first(p) ∘ first(q)`.
	/// * Naturality: `dimap(fst, fst) ∘ first(p) = first(p) ∘ dimap(fst, fst)`.
	#[document_examples]
	///
	/// Strong laws for [`RcFnBrand`](crate::brands::RcFnBrand):
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::profunctor::*,
	/// 	functions::*,
	/// };
	///
	/// let p = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2 + 1);
	/// let q = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 10);
	///
	/// // Identity: first(identity) = identity
	/// let id = category_identity::<RcFnBrand, i32>();
	/// let first_id = first::<RcFnBrand, _, _, String>(id);
	/// assert_eq!(first_id((5, "hi".to_string())), (5, "hi".to_string()));
	///
	/// // Composition: first(p ∘ q) = first(p) ∘ first(q)
	/// let lhs = first::<RcFnBrand, _, _, String>(semigroupoid_compose::<RcFnBrand, _, _, _>(
	/// 	p.clone(),
	/// 	q.clone(),
	/// ));
	/// let rhs = semigroupoid_compose::<RcFnBrand, _, _, _>(
	/// 	first::<RcFnBrand, _, _, String>(p),
	/// 	first::<RcFnBrand, _, _, String>(q),
	/// );
	/// assert_eq!(lhs((5, "hi".to_string())), rhs((5, "hi".to_string())));
	/// assert_eq!(lhs((0, "lo".to_string())), rhs((0, "lo".to_string())));
	/// ```
	pub trait Strong: Profunctor {
		/// Lift a profunctor to operate on the first component of a pair.
		///
		/// This method takes a profunctor `P A B` and returns `P (A, C) (B, C)`,
		/// threading the extra context `C` through unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the second component (threaded through unchanged)."
		)]
		///
		#[document_parameters("The profunctor instance to lift.")]
		///
		#[document_returns("A new profunctor that operates on pairs.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::profunctor::*,
		/// 	functions::*,
		/// };
		///
		/// let f = |x: i32| x + 1;
		/// let g = first::<RcFnBrand, _, _, i32>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
		/// assert_eq!(g((10, 20)), (11, 20));
		/// ```
		fn first<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>);

		/// Lift a profunctor to operate on the second component of a pair.
		///
		/// This method takes a profunctor `P A B` and returns `P (C, A) (C, B)`,
		/// threading the extra context `C` through unchanged in the first position.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the first component (threaded through unchanged)."
		)]
		///
		#[document_parameters("The profunctor instance to lift.")]
		///
		#[document_returns("A new profunctor that operates on pairs.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::profunctor::*,
		/// 	functions::*,
		/// };
		///
		/// let f = |x: i32| x + 1;
		/// let g = second::<RcFnBrand, _, _, i32>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
		/// assert_eq!(g((20, 10)), (20, 11));
		/// ```
		fn second<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (C, A), (C, B)>) {
			Self::dimap(|(c, a)| (a, c), |(b, c)| (c, b), Self::first(pab))
		}
	}

	/// Lift a profunctor to operate on the first component of a pair.
	///
	/// Free function version that dispatches to [the type class' associated function][`Strong::first`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the strong profunctor.",
		"The input type of the profunctor.",
		"The output type of the profunctor.",
		"The type of the second component (threaded through unchanged)."
	)]
	///
	#[document_parameters("The profunctor instance to lift.")]
	///
	#[document_returns("A new profunctor that operates on pairs.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::profunctor::*,
	/// 	functions::*,
	/// };
	///
	/// let f = |x: i32| x + 1;
	/// let g = first::<RcFnBrand, _, _, i32>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
	/// assert_eq!(g((10, 20)), (11, 20));
	/// ```
	pub fn first<'a, Brand: Strong, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>) {
		Brand::first(pab)
	}

	/// Lift a profunctor to operate on the second component of a pair.
	///
	/// Free function version that dispatches to [the type class' associated function][`Strong::second`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the strong profunctor.",
		"The input type of the profunctor.",
		"The output type of the profunctor.",
		"The type of the first component (threaded through unchanged)."
	)]
	///
	#[document_parameters("The profunctor instance to lift.")]
	///
	#[document_returns("A new profunctor that operates on pairs.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::profunctor::*,
	/// 	functions::*,
	/// };
	///
	/// let f = |x: i32| x + 1;
	/// let g = second::<RcFnBrand, _, _, i32>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
	/// assert_eq!(g((20, 10)), (20, 11));
	/// ```
	pub fn second<'a, Brand: Strong, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (C, A), (C, B)>) {
		Brand::second(pab)
	}

	/// Compose a value acting on a pair from two values, each acting on one
	/// component of the pair.
	///
	/// Equivalent to PureScript's `splitStrong` / `(***)`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the strong profunctor.",
		"The input type of the first profunctor.",
		"The output type of the first profunctor.",
		"The input type of the second profunctor.",
		"The output type of the second profunctor."
	)]
	///
	#[document_parameters(
		"The profunctor acting on the first component.",
		"The profunctor acting on the second component."
	)]
	///
	#[document_returns(
		"A new profunctor that maps the first component via `l` and the second via `r`."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::profunctor::*,
	/// 	functions::*,
	/// };
	///
	/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	/// let g = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// let h = split_strong::<RcFnBrand, _, _, _, _>(f, g);
	/// assert_eq!(h((10, 20)), (11, 40));
	/// ```
	pub fn split_strong<'a, Brand: Semigroupoid + Strong, A: 'a, B: 'a, C: 'a, D: 'a>(
		l: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
		r: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, C, D>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, D)>) {
		Brand::compose(Brand::second(r), Brand::first(l))
	}

	/// Compose a value which introduces a pair from two values, each introducing
	/// one side of the pair.
	///
	/// Equivalent to PureScript's `fanout` / `(&&&)`.
	///
	/// The `A: Clone` bound is required because Rust implementations need to clone
	/// the input value to feed it into both profunctors.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the strong profunctor.",
		"The shared input type (must be Clone).",
		"The output type of the first profunctor.",
		"The output type of the second profunctor."
	)]
	///
	#[document_parameters(
		"The profunctor producing the first component.",
		"The profunctor producing the second component."
	)]
	///
	#[document_returns(
		"A new profunctor that feeds the input to both `l` and `r`, collecting results in a pair."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::profunctor::*,
	/// 	functions::*,
	/// };
	///
	/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	/// let g = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// let h = fan_out::<RcFnBrand, _, _, _>(f, g);
	/// assert_eq!(h(10), (11, 20));
	/// ```
	pub fn fan_out<'a, Brand: Semigroupoid + Strong, A: 'a + Clone, B: 'a, C: 'a>(
		l: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
		r: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, C>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, (B, C)>) {
		Brand::map_input(|a: A| (a.clone(), a), split_strong::<Brand, A, B, A, C>(l, r))
	}
}

pub use inner::*;
