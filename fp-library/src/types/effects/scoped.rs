//! Scoped-effect row encoding for the dual-row Run substrate.
//!
//! ## Overview
//!
//! The dual-row Run encoding (see
//! [`Node<'a, R, S, A>`](crate::types::effects::node::Node)) keeps
//! first-order algebraic effects (`State`, `Reader`, `Choose`,
//! `Except`, `Writer`, ...) separate from higher-order scoped
//! effects (`Catch`, `Local`, `Bracket`, `Span`, ...). The
//! first-order row uses
//! [`CoproductBrand<H, T>`](crate::brands::CoproductBrand) with each
//! effect wrapped in
//! [`CoyonedaBrand`](crate::brands::CoyonedaBrand) so any effect
//! type becomes a [`Functor`](crate::classes::Functor) for free; the
//! scoped row uses the same `CoproductBrand` structure but populates
//! it with **raw** scoped-effect brands (no Coyoneda), because
//! scoped effects are dispatched via case analysis rather than by
//! [`Functor::map`](crate::classes::Functor::map).
//!
//! ## Type aliases
//!
//! - [`ScopedCoproduct<H, T>`](crate::types::effects::scoped::ScopedCoproduct)
//!   is the row-encoding type for a non-empty scoped-effect row.
//!   Transparent alias for
//!   [`CoproductBrand<H, T>`](crate::brands::CoproductBrand) with
//!   identical trait impls and substrate behaviour. The distinct
//!   name is a self-documenting cue at user-facing type signatures:
//!   `Run<R, ScopedCoproduct<CatchBrand<P, E>, ScopedNil>, A>`
//!   reads as "first-order row `R`, scoped row containing `Catch`"
//!   without requiring the reader to disambiguate which
//!   `CoproductBrand` instance is the first-order vs scoped row.
//! - [`ScopedNil`] is the empty scoped row, transparent alias for
//!   [`CNilBrand`](crate::brands::CNilBrand). Programs that do not
//!   use any scoped effects declare their scoped row as `ScopedNil`
//!   (or equivalently `CNilBrand`).
//!
//! ## Dual-row integration
//!
//! The [`Run<R, S, A>`](crate::types::effects::run::Run) family
//! (and the five sibling Run wrappers
//! [`RcRun`](crate::types::effects::rc_run::RcRun),
//! [`ArcRun`](crate::types::effects::arc_run::ArcRun),
//! [`RunExplicit`](crate::types::effects::run_explicit::RunExplicit),
//! [`RcRunExplicit`](crate::types::effects::rc_run_explicit::RcRunExplicit),
//! [`ArcRunExplicit`](crate::types::effects::arc_run_explicit::ArcRunExplicit))
//! integrates the dual row via
//! [`NodeBrand<R, S>`](crate::brands::NodeBrand) at
//! [`node`](crate::types::effects::node). The aliases below are a
//! pure naming layer; they introduce no new substrate machinery.
//!
//! ## Per-scoped-effect-brand substrate-required traits
//!
//! Each scoped-effect brand placed in a `ScopedCoproduct` must
//! implement [`Functor`](crate::classes::Functor),
//! [`SendFunctor`](crate::classes::SendFunctor),
//! [`WrapDrop`](crate::classes::WrapDrop),
//! [`RefFunctor`](crate::classes::RefFunctor), and
//! [`Extract`](crate::classes::Extract). The substrate's
//! `NodeBrand<R, S>` composition calls `Functor::map` and the
//! companion traversal traits directly on the scoped row's brands,
//! so they must be available even though the scoped-handler
//! dispatcher itself does not go through `Functor`. Programs with
//! `S = ScopedNil` satisfy these requirements vacuously (the empty
//! row's blanket impls cover all five traits).

#[fp_macros::document_module]
mod inner {
	use crate::brands::{
		CNilBrand,
		CoproductBrand,
	};

	/// Row-encoding type for a non-empty scoped-effect row.
	///
	/// Transparent type alias for
	/// [`CoproductBrand<H, T>`](crate::brands::CoproductBrand). All
	/// trait impls and substrate behaviour are inherited verbatim;
	/// the distinct name is a self-documenting cue at user-facing
	/// type signatures so the dual-row's two arms remain visually
	/// distinguished.
	///
	/// # Example
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		CNilBrand,
	/// 		CoproductBrand,
	/// 		IdentityBrand,
	/// 		RcCoyonedaBrand,
	/// 	},
	/// 	types::effects::{
	/// 		rc_run::RcRun,
	/// 		scoped::{
	/// 			ScopedCoproduct,
	/// 			ScopedNil,
	/// 		},
	/// 	},
	/// };
	///
	/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
	/// type EmptyScoped = ScopedNil;
	/// type Prog = RcRun<FirstRow, EmptyScoped, i32>;
	///
	/// // Equivalent to declaring the scoped row as `CNilBrand` directly;
	/// // the alias is purely cosmetic.
	/// let _: Prog = RcRun::pure(0);
	/// ```
	pub type ScopedCoproduct<H, T> = CoproductBrand<H, T>;

	/// Empty scoped-effect row.
	///
	/// Transparent type alias for [`CNilBrand`]. Programs that do
	/// not use any scoped effects declare their scoped row as
	/// `ScopedNil` (or equivalently `CNilBrand`).
	///
	/// # Example
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		CNilBrand,
	/// 		CoproductBrand,
	/// 		IdentityBrand,
	/// 		RcCoyonedaBrand,
	/// 	},
	/// 	handlers,
	/// 	types::{
	/// 		Identity,
	/// 		effects::{
	/// 			rc_run::RcRun,
	/// 			scoped::ScopedNil,
	/// 		},
	/// 	},
	/// };
	///
	/// type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
	/// type Prog = RcRun<FirstRow, ScopedNil, i32>;
	///
	/// let prog: Prog = RcRun::lift::<IdentityBrand, _>(Identity(7));
	/// let result = prog.handle(
	/// 	handlers! {
	/// 		IdentityBrand: |op: Identity<Prog>| op.0,
	/// 	},
	/// 	fp_library::types::effects::scoped_nt(),
	/// );
	/// assert_eq!(result, 7);
	/// ```
	pub type ScopedNil = CNilBrand;
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::{
			brands::{
				CNilBrand,
				CoproductBrand,
				IdentityBrand,
				RcCoyonedaBrand,
			},
			handlers,
			types::{
				Identity,
				effects::rc_run::RcRun,
			},
		},
	};

	type FirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;

	// `ScopedNil` is a transparent alias for `CNilBrand`; programs
	// declared with either spelling produce the same Rust type and
	// interpret identically.
	#[test]
	fn scoped_nil_is_alias_for_cnil_brand() {
		type ProgWithAlias = RcRun<FirstRow, ScopedNil, i32>;
		type ProgWithCNil = RcRun<FirstRow, CNilBrand, i32>;
		let prog_alias: ProgWithAlias = RcRun::lift::<IdentityBrand, _>(Identity(42));
		let prog_cnil: ProgWithCNil = RcRun::lift::<IdentityBrand, _>(Identity(42));
		let r1 = prog_alias.handle(
			handlers! {
				IdentityBrand: |op: Identity<ProgWithAlias>| op.0,
			},
			crate::types::effects::scoped_nt(),
		);
		let r2 = prog_cnil.handle(
			handlers! {
				IdentityBrand: |op: Identity<ProgWithCNil>| op.0,
			},
			crate::types::effects::scoped_nt(),
		);
		assert_eq!(r1, 42);
		assert_eq!(r2, 42);
	}

	// `ScopedCoproduct<H, T>` is a transparent alias for
	// `CoproductBrand<H, T>`; using the alias as `S` in `Run<R, S, A>`
	// produces the same Rust type as using `CoproductBrand` directly.
	// The alias is purely a documentation cue.
	#[test]
	fn scoped_coproduct_is_alias_for_coproduct_brand() {
		fn _take_alias<A, B>(_x: ScopedCoproduct<A, B>)
		where
			A: 'static,
			B: 'static, {
		}
		fn _take_brand<A, B>(x: CoproductBrand<A, B>)
		where
			A: 'static,
			B: 'static, {
			_take_alias::<A, B>(x);
		}
	}
}
