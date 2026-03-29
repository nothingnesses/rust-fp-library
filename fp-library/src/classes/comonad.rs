//! Comonads, the dual of monads, combining [`Extend`] and [`Extract`](crate::classes::Extract).
//!
//! A `Comonad` is a type that supports both extracting a value from a context
//! ([`Extract`](crate::classes::Extract)) and extending a local computation to the whole context
//! ([`Extend`]). It is the categorical dual of [`Monad`](crate::classes::Monad):
//! where `Monad` composes `Pointed` + `Semimonad`, `Comonad` composes
//! `Extract` + `Extend`.
//!
//! This module is a port of PureScript's
//! [`Control.Comonad`](https://pursuit.purescript.org/packages/purescript-control/docs/Control.Comonad).
//!
//! # Hierarchy
//!
//! ```text
//! Functor
//!   |
//!   +-- Extract              (extract :: F<A> -> A; no Functor constraint)
//!   |
//!   +-- Extend: Functor      (extend :: (F<A> -> B) -> F<A> -> F<B>)
//!   |
//!   +-- Comonad: Extend + Extract   (blanket impl, no new methods)
//! ```

#[fp_macros::document_module(no_validation)]
mod inner {
	use crate::classes::*;

	/// A type class for comonads, combining [`Extend`] and [`Extract`].
	///
	/// `class (Extend w, Extract w) => Comonad w`
	///
	/// `Comonad` is the dual of [`Monad`](crate::classes::Monad). Where a `Monad`
	/// lets you sequence effectful computations by injecting values (`pure`) and
	/// chaining with `bind`, a `Comonad` lets you observe values (`extract`) and
	/// extend local computations to global ones (`extend`).
	///
	/// # Laws
	///
	/// A lawful `Comonad` must satisfy three laws:
	///
	/// 1. **Left identity:** extracting after extending recovers the function's
	///    result.
	///
	///    ```text
	///    extract(extend(f, wa)) == f(wa)
	///    ```
	///
	/// 2. **Right identity:** extending with `extract` is a no-op.
	///
	///    ```text
	///    extend(extract, wa) == wa
	///    ```
	///
	/// 3. **Map-extract:** extracting after mapping is the same as applying the
	///    function to the extracted value.
	///
	///    ```text
	///    extract(map(f, wa)) == f(extract(wa))
	///    ```
	pub trait Comonad: Extend + Extract {}

	/// Blanket implementation of [`Comonad`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> Comonad for Brand where Brand: Extend + Extract {}
}

pub use inner::*;
