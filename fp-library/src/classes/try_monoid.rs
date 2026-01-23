//! A trait for types that can be combined fallibly and have an empty value.
//!
//! This is similar to `Monoid`, but extends `TrySemigroup`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, classes::*, functions::*};
//!
//! let x = try_empty::<String>();
//! assert_eq!(x, String::new());
//! ```

use super::{Monoid, TrySemigroup};

/// A trait for types that can be combined fallibly and have an empty value.
///
/// This is similar to [`Monoid`], but extends [`TrySemigroup`].
pub trait TryMonoid: TrySemigroup {
	/// Returns the empty value.
	///
	/// ### Type Signature
	///
	/// `forall a. TryMonoid a => () -> a`
	///
	/// ### Returns
	///
	/// The empty value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*};
	///
	/// let x = String::try_empty();
	/// assert_eq!(x, String::new());
	/// ```
	fn try_empty() -> Self;
}

impl<T: Monoid> TryMonoid for T {
	fn try_empty() -> Self {
		Monoid::empty()
	}
}

/// Returns the empty value.
///
/// Free function version that dispatches to [the type class' associated function][`TryMonoid::try_empty`].
///
/// ### Type Signature
///
/// `forall a. TryMonoid a => () -> a`
///
/// ### Type Parameters
///
/// * `A`: The type of the value.
///
/// ### Returns
///
/// The empty value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::*, functions::*};
///
/// let x = try_empty::<String>();
/// assert_eq!(x, String::new());
/// ```
pub fn try_empty<A: TryMonoid>() -> A {
	A::try_empty()
}
