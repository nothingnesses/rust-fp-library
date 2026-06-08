//! Thread-safe by-value monads, combining [`SendApplicative`](crate::classes::SendApplicative)
//! and [`SendSemimonad`](crate::classes::SendSemimonad).
//!
//! By-value parallel of [`SendRefMonad`](crate::classes::SendRefMonad).
//! Mirrors the by-value [`Monad`](crate::classes::Monad)'s
//! `Applicative + Semimonad` supertrait shape, with thread-safety
//! constraints (`Send + Sync` on closures and on `A` / `B`) layered on
//! top.

#[fp_macros::document_module]
mod inner {
	use crate::classes::*;

	/// A type that supports thread-safe by-value pure value injection,
	/// function application, and monadic sequencing.
	///
	/// Automatically implemented for any type implementing both
	/// [`SendApplicative`] and [`SendSemimonad`].
	///
	/// A lawful `SendMonad` must satisfy the standard three monad laws:
	///
	/// 1. **Left identity**: `send_bind(send_pure(a), f)` produces the same
	///    value as `f(a)`.
	/// 2. **Right identity**: `send_bind(m, send_pure)` produces the same
	///    value as `m`.
	/// 3. **Associativity**: `send_bind(send_bind(m, f), g)` produces the
	///    same value as `send_bind(m, |x| send_bind(f(x), g))`.
	pub trait SendMonad: SendApplicative + SendSemimonad {}

	/// Blanket implementation of [`SendMonad`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> SendMonad for Brand where Brand: SendApplicative + SendSemimonad {}
}

pub use inner::*;
