//! Brands for the effects subsystem.
//!
//! This module clusters the brands whose corresponding types live in
//! [`crate::types::effects`]. Effect-row machinery
//! ([`CoproductBrand`] / [`CNilBrand`] / [`NodeBrand`]), Run-wrapper
//! brands for the Explicit family ([`RunExplicitBrand`] /
//! [`RcRunExplicitBrand`] / [`ArcRunExplicitBrand`]), and standard
//! first-order effect brands ([`StateBrand`] / [`SendStateBrand`]) all
//! live here. Re-exported flat at [`crate::brands`] so user-facing
//! paths are unchanged.
//!
//! Coyoneda and Free variant brands ([`crate::brands::CoyonedaBrand`],
//! [`crate::brands::RcFreeExplicitBrand`], etc.) stay in the parent
//! module because their corresponding types live in [`crate::types`]
//! rather than [`crate::types::effects`]; they are general functor /
//! free-monad abstractions that the effects subsystem consumes but
//! does not own.

#[fp_macros::document_module]
mod inner {
	use std::marker::PhantomData;

	/// Brand for thread-safe [`ArcRunExplicit<R, S, A>`](crate::types::effects::arc_run_explicit::ArcRunExplicit),
	/// the [`Send`] + [`Sync`] multi-shot Explicit Run program.
	///
	/// `ArcRunExplicitBrand<R, S>::Of<'a, A>` resolves to
	/// `ArcRunExplicit<'a, R, S, A>`, which is a thin wrapper over
	/// [`ArcFreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::ArcFreeExplicit).
	/// Brand-level coverage delegates to
	/// [`ArcFreeExplicitBrand`](crate::brands::ArcFreeExplicitBrand)'s
	/// impls and so is limited to
	/// [`SendPointed`](crate::classes::SendPointed); the
	/// [`SendRef`](crate::classes::SendRefFunctor)-family hierarchy is not
	/// reachable through brand-level delegation because
	/// [`ArcFreeExplicitBrand`](crate::brands::ArcFreeExplicitBrand) does
	/// not implement it (auto-derive of `Send + Sync` on `ArcFreeExplicit`
	/// requires a per-`A` HRTB on the [`Kind`](crate::kinds) projection
	/// that stable Rust's trait method signatures cannot carry). Inherent
	/// [`bind`](crate::types::effects::arc_run_explicit::ArcRunExplicit::bind)
	/// and [`map`](crate::types::effects::arc_run_explicit::ArcRunExplicit::map)
	/// methods on `ArcRunExplicit` cover the by-value monadic surface for
	/// concrete-type call sites.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct ArcRunExplicitBrand<R, S>(PhantomData<(R, S)>);

	/// Brand for the empty effect row [`CNil`](crate::types::effects::coproduct::CNil).
	///
	/// The base case of the recursive [`CoproductBrand`] chain that encodes a
	/// row of first-order effect functors. `CNil` is uninhabited, so values of
	/// `CNilBrand`'s `Of<A>` projection never exist at runtime; the brand is
	/// load-bearing only at the type level.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct CNilBrand;

	/// Brand for a non-empty effect row encoded as a nested
	/// [`Coproduct`](crate::types::effects::coproduct::Coproduct).
	///
	/// `CoproductBrand<H, T>` parameterises over a head brand `H` and tail
	/// brand `T`, with `Of<'a, A>` resolving to
	/// `Coproduct<H::Of<'a, A>, T::Of<'a, A>>`. The recursive structure
	/// terminates at [`CNilBrand`]; the canonical shape produced by the
	/// `effects!` macro is
	/// `CoproductBrand<CoyonedaBrand<E1>, CoproductBrand<CoyonedaBrand<E2>, CNilBrand>>`,
	/// where each effect is wrapped in
	/// [`CoyonedaBrand`](crate::brands::CoyonedaBrand) so any effect type
	/// becomes a [`Functor`](crate::classes::Functor) for free.
	///
	/// This is the Rust encoding of PureScript's `VariantF` for
	/// first-order effect rows. See
	/// [`fp-library::types::effects::variant_f`](crate::types::effects::variant_f) for
	/// the [`Functor`](crate::classes::Functor) impl that dispatches at runtime
	/// via the `Inl` / `Inr` variants.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct CoproductBrand<H, T>(PhantomData<(H, T)>);

	/// Brand for
	/// [`Except`](crate::types::effects::except::Except), the
	/// error-throwing first-order effect type with `Throw` (raise an
	/// error of type `E`) as its sole operation. Parameterised only by
	/// the error type `E`; unlike
	/// [`StateBrand`] / [`ReaderBrand`], `ExceptBrand` does not need a
	/// pointer brand `P` or a parallel `SendExceptBrand` because
	/// `Except` has no continuation (`Throw` never returns), so the
	/// `Send + Sync` cascade reduces to a per-wrapper bound on `E`
	/// alone. The same brand serves all six Run wrappers.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct ExceptBrand<E>(PhantomData<E>);

	/// Brand for the [`Node<R, S>`](crate::types::effects::node::Node) wrapper that
	/// dispatches a Free-family computation between its first-order effect
	/// row `R` and its scoped-effect row `S`.
	///
	/// `NodeBrand<R, S>::Of<'a, A>` resolves to
	/// `Node<'a, R, S, A>`. `R` is a row brand of first-order effect
	/// functors (typically a [`CoproductBrand`] of
	/// [`CoyonedaBrand`](crate::brands::CoyonedaBrand)-wrapped effects
	/// terminated by [`CNilBrand`]). `S` is the scoped-effect row brand;
	/// for first-order-only programs it resolves to [`CNilBrand`].
	///
	/// Used as the `F` parameter of the Free-family wrappers that
	/// [`Run`](crate::types::effects::run::Run) / [`RcRun`](crate::types::effects::rc_run::RcRun)
	/// / [`ArcRun`](crate::types::effects::arc_run::ArcRun) (and their Explicit siblings)
	/// build on, e.g., `Run<R, S, A> = Free<NodeBrand<R, S>, A>`.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct NodeBrand<R, S>(PhantomData<(R, S)>);

	/// Brand for [`Reader`](crate::types::effects::reader::Reader), the
	/// environment-reading first-order effect type with `Ask` (read
	/// the immutable environment) as its sole operation. Parameterised
	/// by `P: ToDynCloneFn` (typically [`RcBrand`](crate::brands::RcBrand)
	/// for single-thread substrates or
	/// [`ArcBrand`](crate::brands::ArcBrand) for thread-safe substrates)
	/// so the same effect type works across all six Run wrappers; the
	/// per-wrapper smart constructors thread the substrate-appropriate
	/// `P`.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct ReaderBrand<P, E>(PhantomData<(P, E)>);

	/// Brand for [`RcRunExplicit<R, S, A>`](crate::types::effects::rc_run_explicit::RcRunExplicit),
	/// the multi-shot, [`Clone`]-cheap Explicit Run program.
	///
	/// `RcRunExplicitBrand<R, S>::Of<'a, A>` resolves to
	/// `RcRunExplicit<'a, R, S, A>`, which is a thin wrapper over
	/// [`RcFreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::RcFreeExplicit).
	/// Brand-level coverage delegates to
	/// [`RcFreeExplicitBrand`](crate::brands::RcFreeExplicitBrand)'s impls;
	/// on the by-value side that is
	/// [`Pointed`](crate::classes::Pointed) only (per-`A` `Clone` bounds
	/// on `bind` cannot be added to the trait method signatures); on the
	/// by-reference side it is
	/// [`RefFunctor`](crate::classes::RefFunctor),
	/// [`RefPointed`](crate::classes::RefPointed), and
	/// [`RefSemimonad`](crate::classes::RefSemimonad). Inherent
	/// [`bind`](crate::types::effects::rc_run_explicit::RcRunExplicit::bind) and
	/// [`map`](crate::types::effects::rc_run_explicit::RcRunExplicit::map) on
	/// `RcRunExplicit` cover the by-value monadic surface for concrete-type
	/// call sites.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct RcRunExplicitBrand<R, S>(PhantomData<(R, S)>);

	/// Brand for [`RunExplicit<R, S, A>`](crate::types::effects::run_explicit::RunExplicit),
	/// the single-shot Explicit Run program.
	///
	/// `RunExplicitBrand<R, S>::Of<'a, A>` resolves to
	/// `RunExplicit<'a, R, S, A>`, which is a thin wrapper over
	/// [`FreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::FreeExplicit).
	/// Brand-level coverage delegates to
	/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s impls:
	/// [`Functor`](crate::classes::Functor),
	/// [`Pointed`](crate::classes::Pointed),
	/// [`Semimonad`](crate::classes::Semimonad), and the by-reference
	/// [`RefFunctor`](crate::classes::RefFunctor),
	/// [`RefPointed`](crate::classes::RefPointed),
	/// [`RefSemimonad`](crate::classes::RefSemimonad).
	/// [`Monad`](crate::classes::Monad) and
	/// [`RefMonad`](crate::classes::RefMonad) are not reachable because
	/// the [`Monad`](crate::classes::Monad) blanket impl requires
	/// [`Applicative`](crate::classes::Applicative), which
	/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)
	/// deliberately does not implement (the natural `lift2` definition
	/// needs a per-`A` `Clone` bound that stable Rust's trait method
	/// signatures cannot express).
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct RunExplicitBrand<R, S>(PhantomData<(R, S)>);

	/// Brand for
	/// [`SendReader`](crate::types::effects::reader::SendReader), the
	/// thread-safe sibling of
	/// [`Reader`](crate::types::effects::reader::Reader). The `Ask`
	/// variant stores
	/// `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> A + Send + Sync>`
	/// (with `+ Send + Sync` baked into the trait object's bounds), so
	/// the projection is structurally `Send + Sync`. Used by the Arc
	/// family `ask` smart constructors
	/// ([`ArcRun::ask`](crate::types::effects::arc_run::ArcRun) /
	/// [`ArcRunExplicit::ask`](crate::types::effects::arc_run_explicit::ArcRunExplicit)).
	/// `Arc<dyn Fn(E) -> A>` (without `+ Send + Sync` in the trait
	/// object's bounds) is structurally `!Send + !Sync`, so a parallel
	/// brand whose projection bakes the marker traits in at the type
	/// level is required for end-to-end dispatch through `*Run::interpret`
	/// on Arc-substrate programs. Non-Arc smart constructors keep using
	/// [`ReaderBrand`].
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct SendReaderBrand<P, E>(PhantomData<(P, E)>);

	/// Brand for
	/// [`SendState`](crate::types::effects::state::SendState), the
	/// thread-safe sibling of
	/// [`State`](crate::types::effects::state::State). Variants store
	/// `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> A + Send + Sync>`
	/// (with `+ Send + Sync` baked into the trait object's bounds), so
	/// the projection is structurally `Send + Sync`. Used by the Arc
	/// family smart constructors
	/// ([`ArcRun::get`](crate::types::effects::arc_run::ArcRun) /
	/// [`ArcRun::put`](crate::types::effects::arc_run::ArcRun) /
	/// [`ArcRunExplicit::get`](crate::types::effects::arc_run_explicit::ArcRunExplicit) /
	/// [`ArcRunExplicit::put`](crate::types::effects::arc_run_explicit::ArcRunExplicit)).
	/// `Arc<dyn Fn(...) -> A>` (without `+ Send + Sync` in the trait
	/// object's bounds) is structurally `!Send + !Sync`, so a parallel
	/// brand whose projection bakes the marker traits in at the type
	/// level is required for end-to-end dispatch through `*Run::interpret`
	/// on Arc-substrate programs. Non-Arc smart constructors keep using
	/// [`StateBrand`].
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct SendStateBrand<P, S>(PhantomData<(P, S)>);

	/// Brand for [`State`](crate::types::effects::state::State), the
	/// stateful first-order effect type with `Get` (read state) and
	/// `Put` (write state) operations. Parameterised by
	/// `P: ToDynCloneFn` (typically [`RcBrand`](crate::brands::RcBrand)
	/// for single-thread substrates or
	/// [`ArcBrand`](crate::brands::ArcBrand) for thread-safe substrates)
	/// so the same effect type works across all six Run wrappers; the
	/// per-wrapper smart constructors thread the substrate-appropriate
	/// `P`.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct StateBrand<P, S>(PhantomData<(P, S)>);
}

pub use inner::*;
