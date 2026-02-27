//! Cochoice profunctors, the dual of `Choice`.
//!
//! A `Cochoice` profunctor provides the inverse operations of [`Choice`]: instead of
//! lifting a profunctor through sum types, it extracts a profunctor from one that already
//! operates on sum types.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	classes::profunctor::*,
//! 	types::optics::*,
//! };
//!
//! // Tagged is a cochoice profunctor: it unwraps the Err variant
//! let tagged: Tagged<Result<String, i32>, Result<String, i32>> = Tagged::new(Err(42));
//! let result = <TaggedBrand as Cochoice>::unleft::<i32, i32, String>(tagged);
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

/// A type class for cochoice profunctors.
///
/// `Cochoice` provides the dual operations of [`Choice`]: instead of lifting a profunctor
/// through sum types, it extracts a profunctor from one that already operates on sum types.
///
/// ### Semantic Mapping
///
/// This trait follows the same semantic mapping as [`Choice`]:
/// * [`Cochoice::unleft`] extracts from a profunctor operating on the `Err` variant (the "failure" case), treating it as the `Left` side.
/// * [`Cochoice::unright`] extracts from a profunctor operating on the `Ok` variant (the "success" case), treating it as the `Right` side.
///
/// This aligns with the "Right is Success" convention common in functional programming, despite
/// `Result<T, E>` placing `Ok` (Success) as the first type parameter and `Err` (Failure) as the second.
///
/// ### Laws
///
/// `Cochoice` instances must satisfy the following laws:
/// * `unleft(left(p)) = p`
/// * `unright(right(p)) = p`
pub trait Cochoice: Profunctor {
	/// Extract a profunctor from one operating on the left (Err) variant of a Result.
	///
	/// This is the dual of [`Choice::left`]. It takes a profunctor
	/// `P (Result<C, A>) (Result<C, B>)` and returns `P A B`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The input type of the resulting profunctor.",
		"The output type of the resulting profunctor.",
		"The type of the alternative (Ok) variant (threaded through unchanged)."
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
	/// let tagged: Tagged<Result<String, i32>, Result<String, i32>> = Tagged::new(Err(42));
	/// let result = <TaggedBrand as Cochoice>::unleft::<i32, i32, String>(tagged);
	/// assert_eq!(result.0, 42);
	/// ```
	fn unleft<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>);

	/// Extract a profunctor from one operating on the right (Ok) variant of a Result.
	///
	/// This is the dual of [`Choice::right`]. It takes a profunctor
	/// `P (Result<A, C>) (Result<B, C>)` and returns `P A B`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The input type of the resulting profunctor.",
		"The output type of the resulting profunctor.",
		"The type of the alternative (Err) variant (threaded through unchanged)."
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
	/// let tagged: Tagged<Result<i32, String>, Result<i32, String>> = Tagged::new(Ok(42));
	/// let result = <TaggedBrand as Cochoice>::unright::<i32, i32, String>(tagged);
	/// assert_eq!(result.0, 42);
	/// ```
	fn unright<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
		Self::unleft(Self::dimap(
			|r: Result<C, A>| match r {
				Ok(c) => Err(c),
				Err(a) => Ok(a),
			},
			|r: Result<B, C>| match r {
				Ok(b) => Err(b),
				Err(c) => Ok(c),
			},
			pab,
		))
	}
}

/// Extract a profunctor from one operating on the left (Err) variant of a Result.
///
/// Free function version that dispatches to [the type class' associated function][`Cochoice::unleft`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the cochoice profunctor.",
	"The input type of the resulting profunctor.",
	"The output type of the resulting profunctor.",
	"The type of the alternative (Ok) variant (threaded through unchanged)."
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
/// let tagged: Tagged<Result<String, i32>, Result<String, i32>> = Tagged::new(Err(42));
/// let result = unleft::<TaggedBrand, i32, i32, String>(tagged);
/// assert_eq!(result.0, 42);
/// ```
pub fn unleft<'a, Brand: Cochoice, A: 'a, B: 'a, C: 'a>(
	pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
	Brand::unleft(pab)
}

/// Extract a profunctor from one operating on the right (Ok) variant of a Result.
///
/// Free function version that dispatches to [the type class' associated function][`Cochoice::unright`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the cochoice profunctor.",
	"The input type of the resulting profunctor.",
	"The output type of the resulting profunctor.",
	"The type of the alternative (Err) variant (threaded through unchanged)."
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
/// let tagged: Tagged<Result<i32, String>, Result<i32, String>> = Tagged::new(Ok(42));
/// let result = unright::<TaggedBrand, i32, i32, String>(tagged);
/// assert_eq!(result.0, 42);
/// ```
pub fn unright<'a, Brand: Cochoice, A: 'a, B: 'a, C: 'a>(
	pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
	Brand::unright(pab)
}
