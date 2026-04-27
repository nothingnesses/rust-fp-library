//! Erased-substrate Run program over [`Free`](crate::types::Free) and a
//! dual-row [`NodeBrand`](crate::brands::NodeBrand).
//!
//! `Run<R, S, A>` is the user-facing wrapper for the canonical
//! Run-style effect computation:
//!
//! ```text
//! Run<R, S, A> = Free<NodeBrand<R, S>, A>
//! ```
//!
//! The first-order row brand `R` carries the effect functors (typically
//! a [`CoproductBrand`](crate::brands::CoproductBrand) of
//! [`CoyonedaBrand`](crate::brands::CoyonedaBrand)-wrapped effects
//! terminated by [`CNilBrand`](crate::brands::CNilBrand)); the scoped
//! row brand `S` carries higher-order constructors (Phase 4 populates
//! it with `Catch`, `Local`, etc.; for first-order-only programs it
//! stays as `CNilBrand`).
//!
//! `Run` is the Erased counterpart of
//! `RunExplicit` (Phase 2 step 4b; not yet implemented).
//! The Erased substrate is single-shot, type-erases through
//! `Box<dyn Any>`, has O(1) `bind`, and is `'static`-only. It exposes
//! its API via inherent methods rather than Brand-dispatched type
//! classes, so do-notation is via the `run_do!` macro (Phase 2 step 7),
//! not `m_do!`. Use `RunExplicit` for non-`'static` payloads or when
//! Brand-dispatched typeclass-generic code is required.
//!
//! ## Step 4a scope
//!
//! This module currently only ships the type-level wrapper, the Drop
//! impl (which inherits from the underlying Free's WrapDrop-driven
//! iterative dismantling), and the construction sugar
//! [`Run::from_free`] / [`Run::into_free`]. The user-facing
//! operations (`pure`, `peel`, `send`, `bind`, `map`, `lift_f`,
//! `evaluate`, `handle`, etc.) land in Phase 2 step 5.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::NodeBrand,
			classes::{
				Functor,
				WrapDrop,
			},
			types::Free,
		},
		fp_macros::*,
	};

	/// Erased-substrate Run program: a thin wrapper over
	/// [`Free<NodeBrand<R, S>, A>`](crate::types::Free).
	///
	/// The wrapper exists so user-facing API (`pure`, `peel`, `send`,
	/// effect-row narrowing, handler types) can be expressed without
	/// leaking the underlying Free representation. It is a tuple
	/// struct over the inner Free; converting back via
	/// [`into_free`](Run::into_free) is a zero-cost move.
	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand (typically `CNilBrand` for first-order-only programs).",
		"The result type."
	)]
	pub struct Run<R, S, A>(Free<NodeBrand<R, S>, A>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static;

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The Run instance.")]
	impl<R, S, A> Run<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Wraps a [`Free<NodeBrand<R, S>, A>`](crate::types::Free) as
		/// a `Run<R, S, A>`. Zero-cost.
		#[document_signature]
		///
		#[document_parameters("The underlying Free computation.")]
		///
		#[document_returns("A `Run` wrapping `free`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Free,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let free: Free<NodeBrand<FirstRow, Scoped>, i32> = Free::pure(7);
		/// let _run: Run<FirstRow, Scoped, i32> = Run::from_free(free);
		/// assert!(true);
		/// ```
		#[inline]
		pub fn from_free(free: Free<NodeBrand<R, S>, A>) -> Self {
			Run(free)
		}

		/// Unwraps a `Run<R, S, A>` to its underlying
		/// [`Free<NodeBrand<R, S>, A>`](crate::types::Free).
		/// Zero-cost.
		#[document_signature]
		///
		#[document_returns("The underlying Free computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Free,
		/// 		effects::run::Run,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: Run<FirstRow, Scoped, i32> = Run::from_free(Free::pure(7));
		/// let _free: Free<NodeBrand<FirstRow, Scoped>, i32> = run.into_free();
		/// assert!(true);
		/// ```
		#[inline]
		pub fn into_free(self) -> Free<NodeBrand<R, S>, A> {
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
			types::Free,
		},
	};

	type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
	type Scoped = CNilBrand;
	type RunAlias<A> = Run<FirstRow, Scoped, A>;

	#[test]
	fn from_free_and_into_free_round_trip() {
		let free: Free<NodeBrand<FirstRow, Scoped>, i32> = Free::pure(42);
		let run: RunAlias<i32> = Run::from_free(free);
		let _back: Free<NodeBrand<FirstRow, Scoped>, i32> = run.into_free();
	}

	#[test]
	fn drop_a_pure_run_does_not_panic() {
		let run: RunAlias<i32> = Run::from_free(Free::pure(7));
		drop(run);
	}
}
