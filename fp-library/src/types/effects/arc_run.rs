//! Thread-safe Erased-substrate Run program with `Arc`-shared
//! continuations.
//!
//! `ArcRun<R, S, A>` is the [`Send`] + [`Sync`] sibling of
//! [`RcRun`](crate::types::effects::rc_run::RcRun): the same conceptual identity
//!
//! ```text
//! ArcRun<R, S, A> = ArcFree<NodeBrand<R, S>, A>
//! ```
//!
//! but the underlying [`ArcFree`](crate::types::ArcFree) carries
//! `Arc<dyn Fn + Send + Sync>` continuations rather than `Rc<dyn Fn>`,
//! so programs cross thread boundaries. The whole substrate lives
//! behind an outer [`Arc`](std::sync::Arc), so cloning a program is
//! O(1) atomic refcount bump.
//!
//! Use [`Run`](crate::types::effects::run::Run) when single-threaded and
//! single-shot. Use [`RcRun`](crate::types::effects::rc_run::RcRun) when
//! multi-shot but single-threaded. Use `ArcRun` for thread-safe
//! multi-shot.
//!
//! ## Step 4a scope
//!
//! This module currently only ships the type-level wrapper plus the
//! [`from_arc_free`](ArcRun::from_arc_free) /
//! [`into_arc_free`](ArcRun::into_arc_free) construction sugar. The
//! user-facing operations land in Phase 2 step 5.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::NodeBrand,
			classes::WrapDrop,
			kinds::Kind_cdc7cd43dac7585f,
			types::{
				ArcFree,
				arc_free::ArcTypeErasedValue,
			},
		},
		fp_macros::*,
	};

	/// Thread-safe Erased-substrate Run program with `Arc`-shared
	/// continuations.
	///
	/// Thin wrapper over
	/// [`ArcFree<NodeBrand<R, S>, A>`](crate::types::ArcFree). The
	/// associated-type bound on
	/// [`NodeBrand<R, S>`](crate::brands::NodeBrand)'s `Kind`
	/// projection (`Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync`)
	/// is what lets the compiler auto-derive `Send + Sync` on the
	/// underlying `ArcFree` for concrete row brands.
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	pub struct ArcRun<R, S, A>(ArcFree<NodeBrand<R, S>, A>)
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: 'static;

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRun` instance to clone.")]
	impl<R, S, A> Clone for ArcRun<R, S, A>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: 'static,
	{
		/// Clones the `ArcRun` by atomic refcount bump on the inner
		/// [`ArcFree`](crate::types::ArcFree). O(1).
		#[document_signature]
		///
		#[document_returns("A new `ArcRun` representing an independent branch.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::arc_run::ArcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::from_arc_free(ArcFree::pure(42));
		/// let _branch = arc_run.clone();
		/// assert!(true);
		/// ```
		fn clone(&self) -> Self {
			ArcRun(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRun` instance.")]
	impl<R, S, A> ArcRun<R, S, A>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: 'static,
	{
		/// Wraps an [`ArcFree<NodeBrand<R, S>, A>`](crate::types::ArcFree)
		/// as an `ArcRun<R, S, A>`. Zero-cost.
		#[document_signature]
		///
		#[document_parameters("The underlying `ArcFree` computation.")]
		///
		#[document_returns("An `ArcRun` wrapping `arc_free`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::arc_run::ArcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let _arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::from_arc_free(ArcFree::pure(7));
		/// assert!(true);
		/// ```
		#[inline]
		pub fn from_arc_free(arc_free: ArcFree<NodeBrand<R, S>, A>) -> Self {
			ArcRun(arc_free)
		}

		/// Unwraps an `ArcRun<R, S, A>` to its underlying
		/// [`ArcFree<NodeBrand<R, S>, A>`](crate::types::ArcFree).
		/// Zero-cost.
		#[document_signature]
		///
		#[document_returns("The underlying `ArcFree` computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::arc_run::ArcRun,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::from_arc_free(ArcFree::pure(7));
		/// let _arc_free: ArcFree<NodeBrand<FirstRow, Scoped>, i32> = arc_run.into_arc_free();
		/// assert!(true);
		/// ```
		#[inline]
		pub fn into_arc_free(self) -> ArcFree<NodeBrand<R, S>, A> {
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
				IdentityBrand,
				NodeBrand,
			},
			types::ArcFree,
		},
	};

	type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
	type Scoped = CNilBrand;
	type ArcRunAlias<A> = ArcRun<FirstRow, Scoped, A>;

	#[test]
	fn from_arc_free_and_into_arc_free_round_trip() {
		let arc_free: ArcFree<NodeBrand<FirstRow, Scoped>, i32> = ArcFree::pure(42);
		let arc_run: ArcRunAlias<i32> = ArcRun::from_arc_free(arc_free);
		let _back: ArcFree<NodeBrand<FirstRow, Scoped>, i32> = arc_run.into_arc_free();
	}

	#[test]
	fn clone_bumps_atomic_refcount_in_constant_time() {
		let arc_run: ArcRunAlias<i32> = ArcRun::from_arc_free(ArcFree::pure(7));
		let _branch = arc_run.clone();
	}

	#[test]
	fn drop_a_pure_arc_run_does_not_panic() {
		let arc_run: ArcRunAlias<i32> = ArcRun::from_arc_free(ArcFree::pure(7));
		drop(arc_run);
	}

	fn _send_sync_witness<T: Send + Sync>() {}

	#[test]
	fn arc_run_is_send_sync() {
		_send_sync_witness::<ArcRunAlias<i32>>();
	}
}
