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

	/// Brand for [`BoxChoose`](crate::types::effects::choose::BoxChoose),
	/// the FnOnce-continuation sibling of [`ChooseBrand`] used on
	/// default `Run` / `RunExplicit` substrates whose closure
	/// storage is `Box<dyn FnOnce>`. Parameterised by
	/// `P: ToDynFnOnce`, which is implementable only by
	/// [`BoxBrand`](crate::brands::BoxBrand); `Rc<dyn FnOnce>` and
	/// `Arc<dyn FnOnce>` are operationally broken because
	/// [`FnOnce::call_once`] consumes `self` (the trait object)
	/// out of a shared pointer.
	///
	/// Phase 3.5 retrofit: `Choose` smart constructors only ship
	/// on the four multi-shot wrappers per the
	/// [2026-05-03 wrapper-parameterization resolution](../../../docs/plans/effects/resolutions.md),
	/// so `BoxChoose` is defined for substrate uniformity but no
	/// smart constructor exposes it on `Run` / `RunExplicit`.
	/// Multi-shot wrappers (`RcRun` / `RcRunExplicit` / `ArcRun` /
	/// `ArcRunExplicit`) keep using [`ChooseBrand`] /
	/// [`SendChooseBrand`] because their handlers require
	/// multi-shot continuation invocation.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct BoxChooseBrand<P>(PhantomData<P>);

	/// Brand for [`BoxBracket`](crate::types::effects::bracket::BoxBracket),
	/// the FnOnce-closure sibling of [`BracketBrand`] used on
	/// default `Run` / `RunExplicit` scoped rows whose closure
	/// storage is `Box<dyn FnOnce>`. Parameterised by
	/// `P: ToDynFnOnce`, which is implementable only by
	/// [`BoxBrand`](crate::brands::BoxBrand). The substrate brand
	/// `Sub` (e.g., [`NodeBrand<R, S>`](crate::brands::NodeBrand)) is
	/// carried explicitly because Rust's well-formedness check on
	/// `<Self as Kind>::Of<'a, X>` rejects extracting Sub from the
	/// GAT-filled X via a substrate-side trait projection.
	///
	/// 4-param brand (P, Sub, A, B). Multi-shot non-thread-safe
	/// wrappers (`RcRun` / `RcRunExplicit`) use [`BracketBrand`];
	/// thread-safe wrappers (`ArcRun` / `ArcRunExplicit`) use
	/// [`SendBracketBrand`]. The 3-sibling split mirrors the
	/// [`BoxCatchBrand`] / [`CatchBrand`] / [`SendCatchBrand`]
	/// pattern.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct BoxBracketBrand<P, Sub, A, B>(PhantomData<(P, Sub, A, B)>);

	/// Brand for [`BoxBracketExplicit`](crate::types::effects::bracket::BoxBracketExplicit),
	/// the Explicit-family sibling of [`BoxBracketBrand`] used on
	/// `RunExplicit` substrates whose underlying Free family is
	/// [`FreeExplicit`](crate::types::FreeExplicit). The cell stores
	/// closures returning `FreeExplicit<'a, Sub, _>` programs (vs
	/// `Free<Sub, _>` for the Erased family). Parameterised by
	/// `P: ToDynFnOnce`, implementable only by
	/// [`BoxBrand`](crate::brands::BoxBrand).
	///
	/// 4-param brand (P, Sub, A, B). The Explicit-family split lands
	/// per the B19 closure: each pointer-brand sibling gets a parallel
	/// Explicit-family cell so its substrate matches the wrapper
	/// (`Run` -> `Free`, `RcRun` -> `RcFree`, `ArcRun` -> `ArcFree`,
	/// `RunExplicit` -> `FreeExplicit`, `RcRunExplicit` ->
	/// `RcFreeExplicit`, `ArcRunExplicit` -> `ArcFreeExplicit`).
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct BoxBracketExplicitBrand<P, Sub, A, B>(PhantomData<(P, Sub, A, B)>);

	/// Brand for [`BoxCatch`](crate::types::effects::catch::BoxCatch),
	/// the FnOnce-recovery-handler sibling of [`CatchBrand`] used on
	/// default `Run` / `RunExplicit` scoped rows whose closure
	/// storage is `Box<dyn FnOnce>`. Parameterised by
	/// `P: ToDynFnOnce`, which is implementable only by
	/// [`BoxBrand`](crate::brands::BoxBrand).
	///
	/// Phase 4 step 3.1. Multi-shot non-thread-safe wrappers
	/// (`RcRun` / `RcRunExplicit`) use [`CatchBrand`]; thread-safe
	/// wrappers (`ArcRun` / `ArcRunExplicit`) use [`SendCatchBrand`].
	/// The 3-sibling split mirrors the Phase 3.5 [`BoxStateBrand`] /
	/// [`StateBrand`] / [`SendStateBrand`] pattern.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct BoxCatchBrand<P, E>(PhantomData<(P, E)>);

	/// Brand for [`BoxLocal`](crate::types::effects::local::BoxLocal),
	/// the FnOnce-continuation sibling of [`LocalBrand`] used on
	/// default `Run` / `RunExplicit` substrates whose closure
	/// storage is `Box<dyn FnOnce>`. Parameterised by
	/// `P: ToDynFnOnce`, which is implementable only by
	/// [`BoxBrand`](crate::brands::BoxBrand).
	///
	/// Multi-shot non-thread-safe wrappers (`RcRun` /
	/// `RcRunExplicit`) use [`LocalBrand`]; thread-safe wrappers
	/// (`ArcRun` / `ArcRunExplicit`) use [`SendLocalBrand`]. The
	/// 3-sibling split mirrors the [`BoxCatchBrand`] /
	/// [`CatchBrand`] / [`SendCatchBrand`] pattern.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct BoxLocalBrand<P, E>(PhantomData<(P, E)>);

	/// Brand for [`BoxRefLocal`](crate::types::effects::ref_local::BoxRefLocal),
	/// the FnOnce-continuation sibling of [`RefLocalBrand`] used on
	/// default `Run` / `RunExplicit` substrates whose closure
	/// storage is `Box<dyn FnOnce>`. Parameterised by
	/// `P: ToDynFnOnce`, which is implementable only by
	/// [`BoxBrand`](crate::brands::BoxBrand).
	///
	/// The Ref flavour: `modify` borrows the environment value
	/// (`FnOnce(&E) -> E`), removing the `E: Clone` requirement that
	/// the Val flavour ([`BoxLocalBrand`]) imposes on users who want
	/// to derive a sub-scope environment from the parent without
	/// owning it.
	///
	/// Multi-shot non-thread-safe wrappers (`RcRun` /
	/// `RcRunExplicit`) use [`RefLocalBrand`]; thread-safe wrappers
	/// (`ArcRun` / `ArcRunExplicit`) use [`SendRefLocalBrand`]. The
	/// 3-sibling split mirrors the [`BoxLocalBrand`] / [`LocalBrand`]
	/// / [`SendLocalBrand`] pattern.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct BoxRefLocalBrand<P, E>(PhantomData<(P, E)>);

	/// Brand for [`BoxReader`](crate::types::effects::reader::BoxReader),
	/// the FnOnce-continuation sibling of [`ReaderBrand`] used on
	/// default `Run` / `RunExplicit` substrates whose closure
	/// storage is `Box<dyn FnOnce>`. Parameterised by
	/// `P: ToDynFnOnce`, which is implementable only by
	/// [`BoxBrand`](crate::brands::BoxBrand).
	///
	/// Phase 3.5 retrofit. Multi-shot wrappers (`RcRun` /
	/// `RcRunExplicit` / `ArcRun` / `ArcRunExplicit`) keep using
	/// [`ReaderBrand`] / [`SendReaderBrand`].
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct BoxReaderBrand<P, E>(PhantomData<(P, E)>);

	/// Brand for [`BoxState`](crate::types::effects::state::BoxState),
	/// the FnOnce-continuation sibling of [`StateBrand`] used on
	/// default `Run` / `RunExplicit` substrates whose closure
	/// storage is `Box<dyn FnOnce>`. Parameterised by
	/// `P: ToDynFnOnce`, which is implementable only by
	/// [`BoxBrand`](crate::brands::BoxBrand).
	///
	/// Phase 3.5 retrofit. Multi-shot non-thread-safe wrappers
	/// (`RcRun` / `RcRunExplicit`) keep using [`StateBrand`];
	/// thread-safe wrappers (`ArcRun` / `ArcRunExplicit`) keep
	/// using [`SendStateBrand`].
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct BoxStateBrand<P, S>(PhantomData<(P, S)>);

	/// Brand for the empty effect row [`CNil`](crate::types::effects::coproduct::CNil).
	///
	/// The base case of the recursive [`CoproductBrand`] chain that encodes a
	/// row of first-order effect functors. `CNil` is uninhabited, so values of
	/// `CNilBrand`'s `Of<A>` projection never exist at runtime; the brand is
	/// load-bearing only at the type level.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct CNilBrand;

	/// Brand for [`Bracket`](crate::types::effects::bracket::Bracket),
	/// the scoped resource-management effect that acquires a
	/// resource, runs a body using it, then releases it. The cell
	/// holds three closures with three differently-typed program
	/// returns over a single substrate brand `Sub`: acquire returns
	/// `Free<Sub, A>`, body returns `Free<Sub, (A, B)>`, release
	/// returns `Free<Sub, ()>`. Parameterised by `P: ToDynCloneFn`
	/// (typically [`RcBrand`](crate::brands::RcBrand)) so the
	/// closure storage shares the same per-pointer-brand pattern
	/// used elsewhere in the library.
	///
	/// 4-param brand (P, Sub, A, B). Sub is carried explicitly
	/// because Rust's well-formedness check on
	/// `<Self as Kind>::Of<'a, X>` rejects extracting Sub from the
	/// GAT-filled X via a substrate-side trait projection.
	///
	/// Single-shot wrappers (`Run` / `RunExplicit`) use
	/// [`BoxBracketBrand`]; thread-safe wrappers (`ArcRun` /
	/// `ArcRunExplicit`) use [`SendBracketBrand`]. The 3-sibling
	/// split mirrors the [`BoxCatchBrand`] / [`CatchBrand`] /
	/// [`SendCatchBrand`] pattern.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct BracketBrand<P, Sub, A, B>(PhantomData<(P, Sub, A, B)>);

	/// Brand for [`BracketExplicit`](crate::types::effects::bracket::BracketExplicit),
	/// the Explicit-family sibling of [`BracketBrand`] used on
	/// `RcRunExplicit` substrates whose underlying Free family is
	/// [`RcFreeExplicit`](crate::types::RcFreeExplicit). The cell stores
	/// closures returning `RcFreeExplicit<'a, Sub, _>` programs (vs
	/// `RcFree<Sub, _>` for the Erased family). Parameterised by
	/// `P: ToDynCloneFn`, typically [`RcBrand`](crate::brands::RcBrand).
	///
	/// 4-param brand (P, Sub, A, B). Lands per the B19 closure; see
	/// [`BoxBracketExplicitBrand`] for the per-Free-family split
	/// rationale.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct BracketExplicitBrand<P, Sub, A, B>(PhantomData<(P, Sub, A, B)>);

	/// Brand for [`Catch`](crate::types::effects::catch::Catch), the
	/// scoped error-recovery effect that runs an `action` program and,
	/// if the action throws an error of type `E`, invokes a recovery
	/// handler to produce a recovery program. Parameterised by
	/// `P: ToDynCloneFn` (typically [`RcBrand`](crate::brands::RcBrand))
	/// so the recovery handler closure storage shares the
	/// same per-pointer-brand pattern used elsewhere in the library.
	///
	/// Phase 4 step 3.1. Single-shot wrappers (`Run` / `RunExplicit`)
	/// use [`BoxCatchBrand`]; thread-safe wrappers (`ArcRun` /
	/// `ArcRunExplicit`) use [`SendCatchBrand`]. The 3-sibling split
	/// mirrors the Phase 3.5 [`BoxStateBrand`] / [`StateBrand`] /
	/// [`SendStateBrand`] pattern.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct CatchBrand<P, E>(PhantomData<(P, E)>);

	/// Brand for [`Choose`](crate::types::effects::choose::Choose),
	/// the nondeterministic-branching first-order effect type with
	/// `Alt` (run a continuation `bool -> A` for both branches) as
	/// its sole operation. Parameterised by `P: ToDynCloneFn`
	/// (typically [`RcBrand`](crate::brands::RcBrand) for
	/// single-thread substrates) so the same effect type works
	/// across the four multi-shot Run wrappers.
	///
	/// `Choose` ships only on the four multi-shot wrappers
	/// ([`RcRun`](crate::types::effects::rc_run::RcRun) /
	/// [`RcRunExplicit`](crate::types::effects::rc_run_explicit::RcRunExplicit) /
	/// [`ArcRun`](crate::types::effects::arc_run::ArcRun) /
	/// [`ArcRunExplicit`](crate::types::effects::arc_run_explicit::ArcRunExplicit))
	/// because a `Choose` handler runs the continuation twice (once
	/// for each branch), which requires the continuation to be
	/// cloneable; the single-shot wrappers
	/// ([`Run`](crate::types::effects::run::Run) /
	/// [`RunExplicit`](crate::types::effects::run_explicit::RunExplicit))
	/// cannot host this effect.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct ChooseBrand<P>(PhantomData<P>);

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

	/// Brand for [`Local`](crate::types::effects::local::Local), the
	/// scoped environment-modification effect that runs an `action`
	/// program under an environment value transformed by a `modify`
	/// closure. The Val flavour: `modify` consumes the environment
	/// value (`Fn(E) -> E`). Parameterised by `P: ToDynCloneFn`
	/// (typically [`RcBrand`](crate::brands::RcBrand)) so the
	/// modify and action closure storage shares the same per-pointer-
	/// brand pattern used elsewhere in the library.
	///
	/// Single-shot wrappers (`Run` / `RunExplicit`) use
	/// [`BoxLocalBrand`]; thread-safe wrappers (`ArcRun` /
	/// `ArcRunExplicit`) use [`SendLocalBrand`]. The 3-sibling split
	/// mirrors the [`BoxCatchBrand`] / [`CatchBrand`] /
	/// [`SendCatchBrand`] pattern.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct LocalBrand<P, E>(PhantomData<(P, E)>);

	/// Brand for [`RefLocal`](crate::types::effects::ref_local::RefLocal),
	/// the scoped environment-modification effect that runs an
	/// `action` program under an environment value transformed by a
	/// `modify` closure. The Ref flavour: `modify` borrows the
	/// environment value (`Fn(&E) -> E`), removing the `E: Clone`
	/// requirement the Val flavour ([`LocalBrand`]) imposes on users
	/// who want to derive a sub-scope environment from the parent
	/// without owning it. Parameterised by `P: ToDynCloneFn`
	/// (typically [`RcBrand`](crate::brands::RcBrand)) so the modify
	/// and action closure storage shares the same per-pointer-brand
	/// pattern used elsewhere in the library.
	///
	/// Single-shot wrappers (`Run` / `RunExplicit`) use
	/// [`BoxRefLocalBrand`]; thread-safe wrappers (`ArcRun` /
	/// `ArcRunExplicit`) use [`SendRefLocalBrand`]. The 3-sibling
	/// split mirrors the [`BoxLocalBrand`] / [`LocalBrand`] /
	/// [`SendLocalBrand`] pattern.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct RefLocalBrand<P, E>(PhantomData<(P, E)>);

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

	/// Brand for [`SendBracket`](crate::types::effects::bracket::SendBracket),
	/// the thread-safe sibling of [`BracketBrand`]. The `Bracket`
	/// variant stores all three closures (acquire as a unit-arg
	/// B-thunk, body and release as resource-consuming closures) as
	/// `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> ... + Send + Sync>`
	/// (with `+ Send + Sync` baked into the trait object's bounds),
	/// so the projection is structurally `Send + Sync`. Used by the
	/// Arc family `bracket` smart constructors
	/// ([`ArcRun::bracket`](crate::types::effects::arc_run::ArcRun) /
	/// [`ArcRunExplicit::bracket`](crate::types::effects::arc_run_explicit::ArcRunExplicit),
	/// landing in step 3.3.3).
	///
	/// 4-param brand (P, Sub, A, B); see [`BracketBrand`] for the
	/// rationale on carrying Sub explicitly.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct SendBracketBrand<P, Sub, A, B>(PhantomData<(P, Sub, A, B)>);

	/// Brand for [`SendBracketExplicit`](crate::types::effects::bracket::SendBracketExplicit),
	/// the Explicit-family sibling of [`SendBracketBrand`] used on
	/// `ArcRunExplicit` substrates whose underlying Free family is
	/// [`ArcFreeExplicit`](crate::types::ArcFreeExplicit). The cell
	/// stores closures returning `ArcFreeExplicit<'a, Sub, _>` programs
	/// (vs `ArcFree<Sub, _>` for the Erased family). Parameterised by
	/// `P: ToDynSendFn`, typically [`ArcBrand`](crate::brands::ArcBrand).
	///
	/// 4-param brand (P, Sub, A, B). Lands per the B19 closure; see
	/// [`BoxBracketExplicitBrand`] for the per-Free-family split
	/// rationale.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct SendBracketExplicitBrand<P, Sub, A, B>(PhantomData<(P, Sub, A, B)>);

	/// Brand for [`SendCatch`](crate::types::effects::catch::SendCatch),
	/// the thread-safe sibling of [`CatchBrand`]. The `Catch` variant
	/// stores `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> A + Send + Sync>`
	/// (with `+ Send + Sync` baked into the trait object's bounds), so
	/// the projection is structurally `Send + Sync`. Used by the Arc
	/// family `catch` smart constructors
	/// ([`ArcRun::catch`](crate::types::effects::arc_run::ArcRun) /
	/// [`ArcRunExplicit::catch`](crate::types::effects::arc_run_explicit::ArcRunExplicit)).
	/// `Arc<dyn Fn(E) -> A>` (without `+ Send + Sync` in the trait
	/// object's bounds) is structurally `!Send + !Sync`, so a parallel
	/// brand whose projection bakes the marker traits in at the type
	/// level is required for end-to-end dispatch through `*Run::interpret`
	/// on Arc-substrate programs.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct SendCatchBrand<P, E>(PhantomData<(P, E)>);

	/// Brand for
	/// [`SendChoose`](crate::types::effects::choose::SendChoose), the
	/// thread-safe sibling of
	/// [`Choose`](crate::types::effects::choose::Choose). The `Alt`
	/// variant stores
	/// `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(bool) -> A + Send + Sync>`
	/// (with `+ Send + Sync` baked into the trait object's bounds),
	/// so the projection is structurally `Send + Sync`. Used by the
	/// Arc family `choose` smart constructors
	/// ([`ArcRun::choose`](crate::types::effects::arc_run::ArcRun) /
	/// [`ArcRunExplicit::choose`](crate::types::effects::arc_run_explicit::ArcRunExplicit)).
	/// `Arc<dyn Fn(bool) -> A>` (without `+ Send + Sync` in the
	/// trait object's bounds) is structurally `!Send + !Sync`, so a
	/// parallel brand whose projection bakes the marker traits in
	/// at the type level is required for end-to-end dispatch through
	/// `*Run::interpret` on Arc-substrate programs. Non-Arc smart
	/// constructors keep using [`ChooseBrand`].
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct SendChooseBrand<P>(PhantomData<P>);

	/// Brand for
	/// [`SendLocal`](crate::types::effects::local::SendLocal), the
	/// thread-safe sibling of [`LocalBrand`]. The `Local` variant
	/// stores
	/// `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> E + Send + Sync>`
	/// for `modify` and
	/// `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>`
	/// for the action thunk (with `+ Send + Sync` baked into both
	/// trait objects' bounds), so the projections are structurally
	/// `Send + Sync`. Used by the Arc family `local` smart constructors
	/// ([`ArcRun::local`](crate::types::effects::arc_run::ArcRun) /
	/// [`ArcRunExplicit::local`](crate::types::effects::arc_run_explicit::ArcRunExplicit)).
	/// `Arc<dyn Fn(E) -> E>` (without `+ Send + Sync` in the trait
	/// object's bounds) is structurally `!Send + !Sync`, so a
	/// parallel brand whose projection bakes the marker traits in at
	/// the type level is required for end-to-end dispatch through
	/// `*Run::interpret` on Arc-substrate programs.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct SendLocalBrand<P, E>(PhantomData<(P, E)>);

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
	/// [`SendRefLocal`](crate::types::effects::ref_local::SendRefLocal),
	/// the thread-safe sibling of [`RefLocalBrand`]. The `Local` variant
	/// stores
	/// `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(&E) -> E + Send + Sync>`
	/// for `modify` and
	/// `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>`
	/// for the action thunk (with `+ Send + Sync` baked into both
	/// trait objects' bounds), so the projections are structurally
	/// `Send + Sync`. Used by the Arc family `ref_local` smart
	/// constructors. The Ref flavour: `modify` borrows the environment
	/// value (`Fn(&E) -> E`), removing the `E: Clone` requirement that
	/// the Val flavour ([`SendLocalBrand`]) imposes.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct SendRefLocalBrand<P, E>(PhantomData<(P, E)>);

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

	/// Brand for
	/// [`Writer`](crate::types::effects::writer::Writer), the
	/// log-emitting first-order effect type with `Tell` (emit a log
	/// value of type `W`) as its sole operation. Parameterised only
	/// by the log type `W`; unlike [`StateBrand`] / [`ReaderBrand`],
	/// `WriterBrand` does not need a pointer brand `P` or a parallel
	/// `SendWriterBrand` because `Writer` has no `dyn Fn`
	/// continuation (`Tell` carries the log value and the next
	/// program's value directly), so the `Send + Sync` cascade
	/// reduces to a per-wrapper bound on `W` alone. The same brand
	/// serves all six Run wrappers.
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct WriterBrand<W>(PhantomData<W>);
}

pub use inner::*;
