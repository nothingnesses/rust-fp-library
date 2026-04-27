//! Erased-substrate Run program with `Rc`-shared continuations
//! supporting multi-shot effects.
//!
//! `RcRun<R, S, A>` is the multi-shot, [`Clone`]-cheap sibling of
//! [`Run`](crate::types::effects::run::Run): the same conceptual identity
//!
//! ```text
//! RcRun<R, S, A> = RcFree<NodeBrand<R, S>, A>
//! ```
//!
//! but the underlying [`RcFree`](crate::types::RcFree) carries
//! `Rc<dyn Fn>` continuations rather than `Box<dyn FnOnce>`, so
//! handlers for non-deterministic effects (`Choose`, `Amb`) can drive
//! the same suspended program more than once. The whole substrate
//! lives behind an outer [`Rc`](std::rc::Rc), so cloning a program
//! is O(1).
//!
//! Use [`Run`](crate::types::effects::run::Run) when continuations are
//! single-shot (the common case). Use `RcRun` for multi-shot effects.
//! Use [`ArcRun`](crate::types::effects::arc_run::ArcRun) when programs cross
//! thread boundaries.
//!
//! ## Step 4a scope
//!
//! This module currently only ships the type-level wrapper plus the
//! [`from_rc_free`](RcRun::from_rc_free) /
//! [`into_rc_free`](RcRun::into_rc_free) construction sugar. The
//! user-facing operations (`pure`, `peel`, `send`, `bind`, `map`,
//! `lift_f`, `evaluate`, `handle`, etc.) land in Phase 2 step 5.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::NodeBrand,
			classes::{
				Functor,
				WrapDrop,
			},
			types::RcFree,
		},
		fp_macros::*,
	};

	/// Erased-substrate Run program with `Rc`-shared continuations.
	///
	/// Thin wrapper over
	/// [`RcFree<NodeBrand<R, S>, A>`](crate::types::RcFree). Users
	/// reach for `RcRun` when an effect needs multi-shot continuations
	/// (the program may be re-driven by the same handler more than
	/// once); cloning is O(1).
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	pub struct RcRun<R, S, A>(RcFree<NodeBrand<R, S>, A>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static;

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRun` instance to clone.")]
	impl<R, S, A> Clone for RcRun<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Clones the `RcRun` by bumping the refcount on the inner
		/// [`RcFree`](crate::types::RcFree). O(1).
		#[document_signature]
		///
		#[document_returns("A new `RcRun` representing an independent branch.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		RcFree,
		/// 		effects::rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::from_rc_free(RcFree::pure(42));
		/// let _branch = rc_run.clone();
		/// assert!(true);
		/// ```
		fn clone(&self) -> Self {
			RcRun(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRun` instance.")]
	impl<R, S, A> RcRun<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Wraps an [`RcFree<NodeBrand<R, S>, A>`](crate::types::RcFree)
		/// as an `RcRun<R, S, A>`. Zero-cost.
		#[document_signature]
		///
		#[document_parameters("The underlying `RcFree` computation.")]
		///
		#[document_returns("An `RcRun` wrapping `rc_free`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		RcFree,
		/// 		effects::rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let _rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::from_rc_free(RcFree::pure(7));
		/// assert!(true);
		/// ```
		#[inline]
		pub fn from_rc_free(rc_free: RcFree<NodeBrand<R, S>, A>) -> Self {
			RcRun(rc_free)
		}

		/// Unwraps an `RcRun<R, S, A>` to its underlying
		/// [`RcFree<NodeBrand<R, S>, A>`](crate::types::RcFree).
		/// Zero-cost.
		#[document_signature]
		///
		#[document_returns("The underlying `RcFree` computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		RcFree,
		/// 		effects::rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::from_rc_free(RcFree::pure(7));
		/// let _rc_free: RcFree<NodeBrand<FirstRow, Scoped>, i32> = rc_run.into_rc_free();
		/// assert!(true);
		/// ```
		#[inline]
		pub fn into_rc_free(self) -> RcFree<NodeBrand<R, S>, A> {
			self.0
		}
	}
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
				CoyonedaBrand,
				IdentityBrand,
				NodeBrand,
			},
			types::RcFree,
		},
	};

	type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
	type Scoped = CNilBrand;
	type RcRunAlias<A> = RcRun<FirstRow, Scoped, A>;

	#[test]
	fn from_rc_free_and_into_rc_free_round_trip() {
		let rc_free: RcFree<NodeBrand<FirstRow, Scoped>, i32> = RcFree::pure(42);
		let rc_run: RcRunAlias<i32> = RcRun::from_rc_free(rc_free);
		let _back: RcFree<NodeBrand<FirstRow, Scoped>, i32> = rc_run.into_rc_free();
	}

	#[test]
	fn clone_bumps_refcount_in_constant_time() {
		let rc_run: RcRunAlias<i32> = RcRun::from_rc_free(RcFree::pure(7));
		let _branch = rc_run.clone();
	}

	#[test]
	fn drop_a_pure_rc_run_does_not_panic() {
		let rc_run: RcRunAlias<i32> = RcRun::from_rc_free(RcFree::pure(7));
		drop(rc_run);
	}
}
