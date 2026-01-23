//! A trait for types that can be combined fallibly.
//!
//! This is similar to `Semigroup`, but the combination operation returns a `Result`.
//! This is useful for types like `Lazy` where combination might trigger evaluation
//! that could fail (panic).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, classes::*, functions::*};
//!
//! let x = String::from("Hello, ");
//! let y = String::from("World!");
//! let z = try_append(x, y);
//! assert_eq!(z, Ok(String::from("Hello, World!")));
//! ```

use super::Semigroup;

/// A trait for types that can be combined fallibly.
///
/// This is similar to [`Semigroup`], but the combination operation returns a `Result`.
/// This is useful for types like `Lazy` where combination might trigger evaluation
/// that could fail (panic).
pub trait TrySemigroup: Sized {
	/// The error type that can occur during combination.
	type Error;

	/// Fallibly combine two values.
	///
	/// ### Type Signature
	///
	/// `forall a. TrySemigroup a => (a, a) -> Result a a::Error`
	///
	/// ### Parameters
	///
	/// * `x`: The first value to combine.
	/// * `y`: The second value to combine.
	///
	/// ### Returns
	///
	/// The result of combining `x` and `y`, or an error.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*};
	///
	/// let x = String::from("Hello, ");
	/// let y = String::from("World!");
	/// let z = String::try_append(x, y);
	/// assert_eq!(z, Ok(String::from("Hello, World!")));
	/// ```
	fn try_append(
		x: Self,
		y: Self,
	) -> Result<Self, Self::Error>;
}

impl<T: Semigroup> TrySemigroup for T {
	type Error = std::convert::Infallible;

	fn try_append(
		x: Self,
		y: Self,
	) -> Result<Self, Self::Error> {
		Ok(<T as Semigroup>::append(x, y))
	}
}

/// Fallibly combine two values.
///
/// Free function version that dispatches to [the type class' associated function][`TrySemigroup::try_append`].
///
/// ### Type Signature
///
/// `forall a. TrySemigroup a => (a, a) -> Result a a::Error`
///
/// ### Type Parameters
///
/// * `A`: The type of the values to combine.
///
/// ### Parameters
///
/// * `x`: The first value to combine.
/// * `y`: The second value to combine.
///
/// ### Returns
///
/// The result of combining `x` and `y`, or an error.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::*, functions::*};
///
/// let x = String::from("Hello, ");
/// let y = String::from("World!");
/// let z = try_append(x, y);
/// assert_eq!(z, Ok(String::from("Hello, World!")));
/// ```
pub fn try_append<A: TrySemigroup>(
	x: A,
	y: A,
) -> Result<A, A::Error> {
	A::try_append(x, y)
}
