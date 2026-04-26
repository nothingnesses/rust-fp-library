//! Thread-safe by-value applicative functors, combining
//! [`SendPointed`](crate::classes::SendPointed),
//! [`SendSemiapplicative`](crate::classes::SendSemiapplicative),
//! [`SendApplyFirst`](crate::classes::SendApplyFirst), and
//! [`SendApplySecond`](crate::classes::SendApplySecond).
//!
//! By-value parallel of [`SendRefApplicative`](crate::classes::SendRefApplicative).

#[fp_macros::document_module]
mod inner {
	use crate::classes::*;

	/// A type that supports thread-safe by-value pure value injection and
	/// thread-safe by-value function application within a context.
	///
	/// Automatically implemented for any type implementing
	/// [`SendPointed`], [`SendSemiapplicative`], [`SendApplyFirst`], and
	/// [`SendApplySecond`].
	pub trait SendApplicative:
		SendPointed + SendSemiapplicative + SendApplyFirst + SendApplySecond {
	}

	/// Blanket implementation of [`SendApplicative`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> SendApplicative for Brand where
		Brand: SendPointed + SendSemiapplicative + SendApplyFirst + SendApplySecond
	{
	}
}

pub use inner::*;
