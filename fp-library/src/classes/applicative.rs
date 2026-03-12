//! Applicative functors, allowing for values and functions to be wrapped and applied within a context.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! 	functions::*,
//! };
//!
//! // Applicative combines Pointed (pure) and Semiapplicative (apply)
//! let f = pure::<OptionBrand, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
//! let x = pure::<OptionBrand, _>(5);
//! let y = apply::<RcFnBrand, OptionBrand, _, _>(f, x);
//! assert_eq!(y, Some(10));
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

	/// A type class for applicative functors, allowing for values to be wrapped
	/// in a context and for functions within a context to be applied to values
	/// within a context.
	///
	/// `class (Pointed f, Semiapplicative f) => Applicative f`
	///
	/// ### Laws
	///
	/// `Applicative` instances must satisfy the following laws:
	/// * Identity: `apply(pure(identity), v) = v`.
	/// * Composition: `apply(apply(map(|f| |g| compose(f, g), u), v), w) = apply(u, apply(v, w))`.
	/// * Homomorphism: `apply(pure(f), pure(x)) = pure(f(x))`.
	/// * Interchange: `apply(u, pure(y)) = apply(pure(|f| f(y)), u)`.
	#[document_examples]
	///
	/// Applicative laws for [`Option`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	functions::*,
	/// };
	///
	/// // Identity: apply(pure(identity), v) = v
	/// let v = Some(5);
	/// let id_fn = pure::<OptionBrand, _>(cloneable_fn_new::<RcFnBrand, _, _>(identity::<i32>));
	/// assert_eq!(apply::<RcFnBrand, OptionBrand, _, _>(id_fn, v), v);
	///
	/// // Homomorphism: apply(pure(f), pure(x)) = pure(f(x))
	/// let f = |x: i32| x * 2;
	/// assert_eq!(
	/// 	apply::<RcFnBrand, OptionBrand, _, _>(
	/// 		pure::<OptionBrand, _>(cloneable_fn_new::<RcFnBrand, _, _>(f)),
	/// 		pure::<OptionBrand, _>(5),
	/// 	),
	/// 	pure::<OptionBrand, _>(f(5)),
	/// );
	///
	/// // Interchange: apply(u, pure(y)) = apply(pure(|f| f(y)), u)
	/// let u = Some(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1));
	/// let y = 5i32;
	/// let left = apply::<RcFnBrand, OptionBrand, _, _>(u.clone(), pure::<OptionBrand, _>(y));
	/// let apply_y = pure::<OptionBrand, _>(cloneable_fn_new::<RcFnBrand, _, _>(
	/// 	move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y),
	/// ));
	/// let right = apply::<RcFnBrand, OptionBrand, _, _>(apply_y, u);
	/// assert_eq!(left, right);
	/// ```
	pub trait Applicative: Pointed + Semiapplicative + ApplyFirst + ApplySecond {}

	/// Blanket implementation of [`Applicative`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> Applicative for Brand where Brand: Pointed + Semiapplicative + ApplyFirst + ApplySecond {}

	/// Performs an applicative action when a condition is true.
	///
	/// Returns the given action if `condition` is `true`, otherwise returns `pure(())`.
	#[document_signature]
	///
	#[document_type_parameters("The lifetime of the computation.", "The brand of the applicative.")]
	///
	#[document_parameters(
		"The condition to check.",
		"The action to perform if the condition is true."
	)]
	///
	#[document_returns("The action if the condition is true, otherwise `pure(())`.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// assert_eq!(when::<OptionBrand>(true, Some(())), Some(()));
	/// assert_eq!(when::<OptionBrand>(false, Some(())), Some(()));
	/// assert_eq!(when::<VecBrand>(true, vec![(), ()]), vec![(), ()]);
	/// assert_eq!(when::<VecBrand>(false, vec![(), ()]), vec![()]);
	/// ```
	pub fn when<'a, Brand: Applicative>(
		condition: bool,
		action: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>) {
		if condition { action } else { Brand::pure(()) }
	}

	/// Performs an applicative action unless a condition is true.
	///
	/// Returns the given action if `condition` is `false`, otherwise returns `pure(())`.
	#[document_signature]
	///
	#[document_type_parameters("The lifetime of the computation.", "The brand of the applicative.")]
	///
	#[document_parameters(
		"The condition to check.",
		"The action to perform if the condition is false."
	)]
	///
	#[document_returns("The action if the condition is false, otherwise `pure(())`.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// assert_eq!(unless::<OptionBrand>(false, Some(())), Some(()));
	/// assert_eq!(unless::<OptionBrand>(true, Some(())), Some(()));
	/// assert_eq!(unless::<VecBrand>(false, vec![(), ()]), vec![(), ()]);
	/// assert_eq!(unless::<VecBrand>(true, vec![(), ()]), vec![()]);
	/// ```
	pub fn unless<'a, Brand: Applicative>(
		condition: bool,
		action: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>) {
		if !condition { action } else { Brand::pure(()) }
	}
}

pub use inner::*;
