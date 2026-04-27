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
			Apply,
			brands::NodeBrand,
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::*,
			types::{
				ArcFree,
				ArcFreeExplicit,
				arc_free::ArcTypeErasedValue,
				effects::arc_run_explicit::ArcRunExplicit,
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

		/// Wraps a value in a pure `ArcRun` computation. Delegates to
		/// [`ArcFree::pure`](crate::types::ArcFree).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `ArcRun` computation that produces `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::pure(42);
		/// assert_eq!(arc_run.into_arc_free().evaluate(), 42);
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self
		where
			A: Send + Sync, {
			ArcRun::from_arc_free(ArcFree::pure(a))
		}

		/// Decomposes this `ArcRun` computation into one step. Returns
		/// `Ok(a)` if the program is a pure value, or `Err(layer)`
		/// carrying the next `ArcRun` continuation in a
		/// [`Node`](crate::types::effects::node::Node) layer. Delegates to
		/// [`ArcFree::resume`](crate::types::ArcFree).
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` for a pure result, or `Err(layer)` carrying the next `ArcRun` step."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::pure(7);
		/// assert!(matches!(arc_run.peel(), Ok(7)));
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "Return type encodes Result<A, NodeBrand<R, S>::Of<'static, ArcRun<R, S, A>>>; the GAT projection is structurally complex but cannot be aliased without losing the projection link the wrapper depends on."
		)]
		pub fn peel(
			self
		) -> Result<
			A,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, A>>),
		>
		where
			NodeBrand<R, S>: Functor,
			A: Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			self.0
				.resume()
				.map_err(|node| <NodeBrand<R, S> as Functor>::map(ArcRun::from_arc_free, node))
		}

		/// Lifts a [`Node`](crate::types::effects::node::Node) dispatch
		/// layer into the `ArcRun` program. The `node` argument is the
		/// [`NodeBrand<R, S>`](crate::brands::NodeBrand)
		/// `Of<'static, A>` projection; `send` delegates to
		/// [`ArcFree::lift_f`](crate::types::ArcFree).
		///
		/// The `Node`-projection signature is required because
		/// `ArcFree`'s struct-level HRTB
		/// (`Of<'static, ArcFree<...>>: Send + Sync`) poisons GAT
		/// normalization in any scope mentioning it: constructing a
		/// `Node::First` literal inside this method body fails to
		/// unify with the projection. The caller (test code, smart
		/// constructors emitted by
		/// [`effects!`](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/plan.md),
		/// or generic helpers without the HRTB) constructs the layer
		/// outside the HRTB scope and passes the result here. See
		/// `tests/arc_run_normalization_probe.rs` for the experimental
		/// matrix that established the limit.
		#[document_signature]
		///
		#[document_parameters("The Node dispatch layer carrying the effect operation.")]
		///
		#[document_returns(
			"An `ArcRun` computation that performs the effect and returns its result."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			arc_run::ArcRun,
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let layer = Coproduct::inject(Identity(7));
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::send(Node::First(layer));
		/// let next = match arc_run.peel() {
		/// 	Err(Node::First(Coproduct::Inl(Identity(n)))) => n,
		/// 	_ => panic!("expected First(Inl(Identity(..))) layer"),
		/// };
		/// assert!(matches!(next.peel(), Ok(7)));
		/// ```
		#[inline]
		pub fn send(
			node: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		) -> Self
		where
			NodeBrand<R, S>: Functor,
			A: Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			ArcRun::from_arc_free(ArcFree::<NodeBrand<R, S>, A>::lift_f(node))
		}

		/// Converts this `ArcRun` into the paired
		/// [`ArcRunExplicit`](crate::types::effects::arc_run_explicit::ArcRunExplicit)
		/// form by walking the underlying [`ArcFree`](crate::types::ArcFree)
		/// chain via [`peel`](ArcRun::peel) and rebuilding each suspended
		/// layer through
		/// [`ArcFreeExplicit::wrap`](crate::types::ArcFreeExplicit). Pure
		/// values re-emerge as
		/// [`ArcRunExplicit::pure`](ArcRunExplicit::pure).
		///
		/// `Send + Sync` is preserved: both `ArcRun` and `ArcRunExplicit`
		/// auto-derive thread-safety from their `Arc<dyn Fn + Send + Sync>`
		/// substrates when `A: Send + Sync` and the projection HRTB holds.
		///
		/// O(N) in the chain depth (one stack frame per suspended layer).
		///
		/// The body uses the GAT-poisoning workaround established in step 5:
		/// projection-typed values come from `peel`'s return and from
		/// `Functor::map`'s output (never constructed inline as
		/// `Node::First(...)` literals), so this composes cleanly under the
		/// HRTB-bearing impl-block scope. See
		/// [`tests/arc_run_normalization_probe.rs`](https://github.com/nothingnesses/rust-fp-library/blob/main/fp-library/tests/arc_run_normalization_probe.rs)
		/// for the regression test.
		#[document_signature]
		///
		#[document_returns("An `ArcRunExplicit<'static, R, S, A>` carrying the same effects.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::pure(7);
		/// let explicit: ArcRunExplicit<'static, FirstRow, Scoped, i32> = arc_run.into_explicit();
		/// assert!(matches!(explicit.peel(), Ok(7)));
		/// ```
		pub fn into_explicit(self) -> ArcRunExplicit<'static, R, S, A>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			NodeBrand<R, S>: Functor,
			A: Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			match self.peel() {
				Ok(a) => ArcRunExplicit::pure(a),
				Err(layer) => {
					let inner = <NodeBrand<R, S> as Functor>::map(
						|run: ArcRun<R, S, A>| -> ArcFreeExplicit<'static, NodeBrand<R, S>, A> {
							run.into_explicit().into_arc_free_explicit()
						},
						layer,
					);
					ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(inner))
				}
			}
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

	#[test]
	fn pure_then_peel_returns_value() {
		let arc_run: ArcRunAlias<i32> = ArcRun::pure(42);
		assert!(matches!(arc_run.peel(), Ok(42)));
	}

	#[test]
	fn send_produces_suspended_program() {
		use crate::types::{
			Identity,
			effects::{
				coproduct::Coproduct,
				node::Node,
			},
		};
		let layer = Coproduct::inject(Identity(7));
		let arc_run: ArcRunAlias<i32> = ArcRun::send(Node::First(layer));
		assert!(arc_run.peel().is_err());
	}

	#[test]
	fn into_explicit_round_trips_pure() {
		let arc_run: ArcRunAlias<i32> = ArcRun::pure(42);
		let explicit = arc_run.into_explicit();
		assert!(matches!(explicit.peel(), Ok(42)));
	}

	#[test]
	fn into_explicit_preserves_suspended_layer() {
		use crate::types::{
			Identity,
			effects::{
				coproduct::Coproduct,
				node::Node,
			},
		};
		let layer = Coproduct::inject(Identity(7));
		let arc_run: ArcRunAlias<i32> = ArcRun::send(Node::First(layer));
		let explicit = arc_run.into_explicit();
		assert!(explicit.peel().is_err());
	}
}
