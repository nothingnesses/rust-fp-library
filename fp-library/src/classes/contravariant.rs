//! Types that can be mapped over contravariantly.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::contravariant::contramap,
//! };
//!
//! let f = |x: i32| x > 5;
//! let is_long_int = contramap::<ProfunctorSecondAppliedBrand<RcFnBrand, bool>, _, _>(
//! 	|s: String| s.len() as i32,
//! 	std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> bool>,
//! );
//! assert_eq!(is_long_int("123456".to_string()), true);
//! assert_eq!(is_long_int("123".to_string()), false);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for types that can be mapped over contravariantly.
	///
	/// A `Contravariant` functor represents a context that consumes values,
	/// allowing functions to be applied to the input before it is consumed.
	///
	/// ### Hierarchy Unification
	///
	/// This trait inherits from [`Kind_cdc7cd43dac7585f`], ensuring that all contravariant
	/// contexts satisfy the strict lifetime requirements where the type argument must
	/// outlive the context's application lifetime.
	///
	/// ### Laws
	///
	/// `Contravariant` instances must satisfy the following laws:
	/// * Identity: `contramap(identity, fa) = fa`.
	/// * Composition: `contramap(compose(f, g), fa) = contramap(g, contramap(f, fa))`.
	#[document_examples]
	///
	/// Contravariant laws for functions via
	/// [`ProfunctorSecondAppliedBrand`](crate::brands::ProfunctorSecondAppliedBrand):
	///
	/// ```
	/// use {
	/// 	fp_library::{
	/// 		brands::*,
	/// 		classes::contravariant::contramap,
	/// 		functions::*,
	/// 	},
	/// 	std::rc::Rc,
	/// };
	///
	/// type Pred = ProfunctorSecondAppliedBrand<RcFnBrand, bool>;
	///
	/// let p = Rc::new(|x: i32| x > 0) as Rc<dyn Fn(i32) -> bool>;
	///
	/// // Identity: contramap(identity, p) = p
	/// let id_mapped = contramap::<Pred, _, _>(identity, p.clone());
	/// assert_eq!(id_mapped(5), p(5));
	/// assert_eq!(id_mapped(-3), p(-3));
	///
	/// // Composition: contramap(compose(f, g), p) = contramap(g, contramap(f, p))
	/// let f = |x: i32| x + 10;
	/// let g = |x: i32| x * 2;
	/// let left = contramap::<Pred, _, _>(compose(f, g), p.clone());
	/// let right = contramap::<Pred, _, _>(g, contramap::<Pred, _, _>(f, p));
	/// assert_eq!(left(5), right(5));
	/// assert_eq!(left(-10), right(-10));
	/// ```
	pub trait Contravariant: Kind_cdc7cd43dac7585f {
		/// Maps a function contravariantly over the context.
		///
		/// This method applies a function to the input before it is consumed by the context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The original type consumed by the context.",
			"The new type to consume."
		)]
		///
		#[document_parameters(
			"The function to apply to the new input.",
			"The contravariant instance."
		)]
		///
		#[document_returns("A new contravariant instance that consumes the new input type.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::contravariant::contramap,
		/// };
		///
		/// let f = |x: i32| x > 5;
		/// let is_long_int = contramap::<ProfunctorSecondAppliedBrand<RcFnBrand, bool>, _, _>(
		/// 	|s: String| s.len() as i32,
		/// 	std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> bool>,
		/// );
		/// assert_eq!(is_long_int("123456".to_string()), true);
		/// assert_eq!(is_long_int("123".to_string()), false);
		/// ```
		fn contramap<'a, A: 'a, B: 'a>(
			f: impl Fn(B) -> A + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Maps a function contravariantly over the context.
	///
	/// Free function version that dispatches to [the type class' associated function][`Contravariant::contramap`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the contravariant functor.",
		"The original type consumed by the context.",
		"The new type to consume."
	)]
	///
	#[document_parameters("The function to apply to the new input.", "The contravariant instance.")]
	///
	#[document_returns("A new contravariant instance that consumes the new input type.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::contravariant::contramap,
	/// };
	///
	/// let f = |x: i32| x > 5;
	/// let is_long_int = contramap::<ProfunctorSecondAppliedBrand<RcFnBrand, bool>, _, _>(
	/// 	|s: String| s.len() as i32,
	/// 	std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> bool>,
	/// );
	/// assert_eq!(is_long_int("123456".to_string()), true);
	/// assert_eq!(is_long_int("123".to_string()), false);
	/// ```
	pub fn contramap<'a, Brand: Contravariant, A: 'a, B: 'a>(
		f: impl Fn(B) -> A + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::contramap(f, fa)
	}
}

pub use inner::*;
