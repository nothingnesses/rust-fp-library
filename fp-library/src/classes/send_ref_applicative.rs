//! Thread-safe by-ref applicative functors, combining [`SendRefPointed`](crate::classes::SendRefPointed) and [`SendRefSemiapplicative`](crate::classes::SendRefSemiapplicative).
//!
//! This is the thread-safe counterpart of [`RefApplicative`](crate::classes::RefApplicative).

#[fp_macros::document_module]
mod inner {
	use crate::classes::*;

	/// A type that supports both thread-safe pure value injection (via reference)
	/// and thread-safe by-ref function application within a context.
	///
	/// This is the thread-safe counterpart of [`RefApplicative`].
	/// Automatically implemented for any type implementing both
	/// [`SendRefPointed`] and [`SendRefSemiapplicative`].
	pub trait SendRefApplicative: SendRefPointed + SendRefSemiapplicative {}

	/// Blanket implementation of [`SendRefApplicative`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> SendRefApplicative for Brand where Brand: SendRefPointed + SendRefSemiapplicative {}
}

pub use inner::*;
