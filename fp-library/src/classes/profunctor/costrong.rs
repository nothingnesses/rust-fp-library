//! Costrong profunctors, the dual of `Strong`.
//!
//! A `Costrong` profunctor provides the inverse operations of [`crate::classes::profunctor::Strong`]: instead of
//! lifting a profunctor through product types, it extracts a profunctor from one that already
//! operates on product types.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	classes::profunctor::*,
//! 	types::optics::*,
//! };
//!
//! // Tagged is a costrong profunctor: it simply drops the extra component
//! let tagged: Tagged<(i32, String), (i32, String)> = Tagged::new((42, "hello".to_string()));
//! let result = <TaggedBrand as Costrong>::unfirst::<i32, i32, String>(tagged);
//! assert_eq!(result.0, 42);
//! ```

use {
	crate::{
		Apply,
		classes::profunctor::Profunctor,
		kinds::*,
	},
	fp_macros::{
		document_parameters,
		document_signature,
		document_type_parameters,
	},
};

/// A type class for costrong profunctors.
///
/// `Costrong` provides the dual operations of [`crate::classes::profunctor::Strong`]: instead of lifting a profunctor
/// through product types (tuples), it extracts a profunctor from one that already operates
/// on product types.
///
/// ### Laws
///
/// `Costrong` instances must satisfy the following laws:
/// * `unfirst(first(p)) = p`
/// * `unsecond(second(p)) = p`
pub trait Costrong: Profunctor {
	/// Extract a profunctor from one operating on the first component of a pair.
	///
	/// This is the dual of [`crate::classes::profunctor::Strong::first`]. It takes a profunctor
	/// `P (A, C) (B, C)` and returns `P A B`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The input type of the resulting profunctor.",
		"The output type of the resulting profunctor.",
		"The type of the second component (threaded through unchanged)."
	)]
	///
	#[document_parameters("The profunctor instance to extract from.")]
	///
	/// ### Returns
	///
	/// A profunctor operating on the unwrapped types.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	classes::profunctor::*,
	/// 	types::optics::*,
	/// };
	///
	/// let tagged: Tagged<(i32, String), (i32, String)> = Tagged::new((42, "hello".to_string()));
	/// let result = <TaggedBrand as Costrong>::unfirst::<i32, i32, String>(tagged);
	/// assert_eq!(result.0, 42);
	/// ```
	fn unfirst<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>);

	/// Extract a profunctor from one operating on the second component of a pair.
	///
	/// This is the dual of [`crate::classes::profunctor::Strong::second`]. It takes a profunctor
	/// `P (C, A) (C, B)` and returns `P A B`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The input type of the resulting profunctor.",
		"The output type of the resulting profunctor.",
		"The type of the first component (threaded through unchanged)."
	)]
	///
	#[document_parameters("The profunctor instance to extract from.")]
	///
	/// ### Returns
	///
	/// A profunctor operating on the unwrapped types.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	classes::profunctor::*,
	/// 	types::optics::*,
	/// };
	///
	/// let tagged: Tagged<(String, i32), (String, i32)> = Tagged::new(("hello".to_string(), 42));
	/// let result = <TaggedBrand as Costrong>::unsecond::<i32, i32, String>(tagged);
	/// assert_eq!(result.0, 42);
	/// ```
	fn unsecond<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (C, A), (C, B)>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
		Self::unfirst(Self::dimap(|(a, c)| (c, a), |(c, b)| (b, c), pab))
	}
}

/// Extract a profunctor from one operating on the first component of a pair.
///
/// Free function version that dispatches to [the type class' associated function][`Costrong::unfirst`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the costrong profunctor.",
	"The input type of the resulting profunctor.",
	"The output type of the resulting profunctor.",
	"The type of the second component (threaded through unchanged)."
)]
///
#[document_parameters("The profunctor instance to extract from.")]
///
/// ### Returns
///
/// A profunctor operating on the unwrapped types.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	classes::profunctor::*,
/// 	types::optics::*,
/// };
///
/// let tagged: Tagged<(i32, String), (i32, String)> = Tagged::new((42, "hello".to_string()));
/// let result = unfirst::<TaggedBrand, i32, i32, String>(tagged);
/// assert_eq!(result.0, 42);
/// ```
pub fn unfirst<'a, Brand: Costrong, A: 'a, B: 'a, C: 'a>(
	pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>)
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
	Brand::unfirst(pab)
}

/// Extract a profunctor from one operating on the second component of a pair.
///
/// Free function version that dispatches to [the type class' associated function][`Costrong::unsecond`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the costrong profunctor.",
	"The input type of the resulting profunctor.",
	"The output type of the resulting profunctor.",
	"The type of the first component (threaded through unchanged)."
)]
///
#[document_parameters("The profunctor instance to extract from.")]
///
/// ### Returns
///
/// A profunctor operating on the unwrapped types.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	classes::profunctor::*,
/// 	types::optics::*,
/// };
///
/// let tagged: Tagged<(String, i32), (String, i32)> = Tagged::new(("hello".to_string(), 42));
/// let result = unsecond::<TaggedBrand, i32, i32, String>(tagged);
/// assert_eq!(result.0, 42);
/// ```
pub fn unsecond<'a, Brand: Costrong, A: 'a, B: 'a, C: 'a>(
	pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (C, A), (C, B)>)
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
	Brand::unsecond(pab)
}
