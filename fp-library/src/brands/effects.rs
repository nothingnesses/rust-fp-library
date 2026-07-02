//! Brands for the effects subsystem.
//!
//! This module clusters the brands whose corresponding types live in
//! [`crate::types::effects`]: the effect-row machinery
//! ([`CoproductBrand`] / [`CNilBrand`]) and the [`Await`](crate::types::effects::await_future::Await)
//! future base-lift effect brand ([`AwaitBrand`]). Re-exported flat at
//! [`crate::brands`] so user-facing paths are unchanged.
//!
//! Coyoneda and Free variant brands ([`crate::brands::CoyonedaBrand`],
//! [`crate::brands::FreeExplicitBrand`], etc.) stay in the parent module
//! because their corresponding types live in [`crate::types`] rather than
//! [`crate::types::effects`]; they are general functor / free-monad
//! abstractions that the effects subsystem consumes but does not own.

#[fp_macros::document_module]
mod inner {
	use std::marker::PhantomData;

	/// Brand for the [`Await`](crate::types::effects::await_future::Await)
	/// future base-lift effect, the first-order effect that embeds a
	/// [`Future`](std::future::Future) so an async driver can await it.
	///
	/// `AwaitBrand::Of<'a, A>` resolves to a boxed local future
	/// `Pin<Box<dyn Future<Output = A> + 'a>>`. The brand is a
	/// [`Functor`](crate::classes::Functor) over that future, which is the
	/// load-bearing property for an async driver: an await effect lifted into a
	/// first-order row is a `Coyoneda<AwaitBrand, _>`, and because the brand is a
	/// `Functor`, the driver lowers it directly to a future of the next program
	/// and awaits that. The boxed future is local (non-[`Send`]), so this targets
	/// single-shot programs; a `Send` future shape is a later addition.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct AwaitBrand;

	/// Brand for the empty effect row [`CNil`](crate::types::effects::coproduct::CNil).
	///
	/// The base case of the recursive [`CoproductBrand`] chain that encodes a
	/// row of first-order effect functors. `CNil` is uninhabited, so values of
	/// `CNilBrand`'s `Of<A>` projection never exist at runtime; the brand is
	/// load-bearing only at the type level.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct CNilBrand;

	/// Brand for a [`Coproduct`](crate::types::effects::coproduct::Coproduct)
	/// row cell: the cons cell of the recursive chain that encodes a row of
	/// first-order effect functors as an open sum.
	///
	/// `CoproductBrand<H, T>` prepends the head functor brand `H` to the tail
	/// row `T`, so a row is a right-nested chain ending in [`CNilBrand`]. Its
	/// [`Functor`](crate::classes::Functor) and
	/// [`WrapDrop`](crate::classes::WrapDrop) impls dispatch at runtime via the
	/// `Inl` / `Inr` variants.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct CoproductBrand<H, T>(PhantomData<(H, T)>);
}

pub use inner::*;
