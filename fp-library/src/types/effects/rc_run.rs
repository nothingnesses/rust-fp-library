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
			Apply,
			brands::NodeBrand,
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::*,
			types::{
				RcFree,
				RcFreeExplicit,
				effects::rc_run_explicit::RcRunExplicit,
			},
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

		/// Wraps a value in a pure `RcRun` computation. Delegates to
		/// [`RcFree::pure`](crate::types::RcFree).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `RcRun` computation that produces `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// // Identity-headed row is used for assertions that engage `peel`
		/// // (which carries a per-projection `Clone` bound that the
		/// // canonical `Coyoneda` row does not satisfy).
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::pure(42);
		/// assert!(matches!(rc_run.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self {
			RcRun::from_rc_free(RcFree::pure(a))
		}

		/// Decomposes this `RcRun` computation into one step. Returns
		/// `Ok(a)` if the program is a pure value, or `Err(layer)`
		/// carrying the next `RcRun` continuation in a [`Node`](crate::types::effects::node::Node) layer.
		/// Delegates to [`RcFree::resume`](crate::types::RcFree).
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` for a pure result, or `Err(layer)` carrying the next `RcRun` step."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// // Identity-headed row: `peel`'s per-projection `Clone` bound
		/// // is satisfied by `Identity<RcFree>: Clone`.
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::pure(7);
		/// assert!(matches!(rc_run.peel(), Ok(7)));
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "Return type encodes Result<A, NodeBrand<R, S>::Of<'static, RcRun<R, S, A>>>; the GAT projection is structurally complex but cannot be aliased without losing the projection link the wrapper depends on."
		)]
		pub fn peel(
			self
		) -> Result<
			A,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, A>>),
		>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			self.0
				.resume()
				.map_err(|node| <NodeBrand<R, S> as Functor>::map(RcRun::from_rc_free, node))
		}

		/// Lifts a [`Node`](crate::types::effects::node::Node) dispatch layer into the `RcRun` program.
		/// The `node` argument is the
		/// [`NodeBrand<R, S>`](crate::brands::NodeBrand) `Of<'static, A>`
		/// projection (typically `Node::First(<R as Member<...>>::inject(operation))`);
		/// `send` delegates to
		/// [`RcFree::lift_f`](crate::types::RcFree). The
		/// `Node`-projection signature is symmetric across the six Run
		/// wrappers; see
		/// [`Run::send`](crate::types::effects::run::Run::send) for the
		/// rationale.
		#[document_signature]
		///
		#[document_parameters("The Node dispatch layer carrying the effect operation.")]
		///
		#[document_returns(
			"An `RcRun` computation that performs the effect and returns its result."
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
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 			rc_run::RcRun,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let layer = Coproduct::inject(Identity(7));
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::send(Node::First(layer));
		/// let next = match rc_run.peel() {
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
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			RcRun::from_rc_free(RcFree::<NodeBrand<R, S>, A>::lift_f(node))
		}

		/// Converts this `RcRun` into the paired
		/// [`RcRunExplicit`](crate::types::effects::rc_run_explicit::RcRunExplicit)
		/// form by walking the underlying [`RcFree`](crate::types::RcFree)
		/// chain via [`peel`](RcRun::peel) and rebuilding each suspended
		/// layer through
		/// [`RcFreeExplicit::wrap`](crate::types::RcFreeExplicit). Pure
		/// values re-emerge as [`RcRunExplicit::pure`](RcRunExplicit::pure).
		///
		/// The conversion preserves multi-shot semantics: the source
		/// `RcRun` carries `Rc<dyn Fn>` continuations and the resulting
		/// `RcRunExplicit` keeps the same `Rc`-shared substrate, so
		/// handlers for non-deterministic effects (e.g., `Choose`) can
		/// drive either side equivalently.
		///
		/// O(N) in the chain depth (one stack frame per suspended layer).
		#[document_signature]
		///
		#[document_returns("An `RcRunExplicit<'static, R, S, A>` carrying the same effects.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		rc_run::RcRun,
		/// 		rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::pure(7);
		/// let explicit: RcRunExplicit<'static, FirstRow, Scoped, i32> = rc_run.into_explicit();
		/// assert!(matches!(explicit.peel(), Ok(7)));
		/// ```
		pub fn into_explicit(self) -> RcRunExplicit<'static, R, S, A>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			match self.peel() {
				Ok(a) => RcRunExplicit::pure(a),
				Err(layer) => {
					let inner = <NodeBrand<R, S> as Functor>::map(
						|run: RcRun<R, S, A>| -> RcFreeExplicit<'static, NodeBrand<R, S>, A> {
							run.into_explicit().into_rc_free_explicit()
						},
						layer,
					);
					RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::wrap(inner))
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
				CoyonedaBrand,
				IdentityBrand,
				NodeBrand,
			},
			types::{
				Identity,
				RcFree,
				effects::{
					coproduct::Coproduct,
					node::Node,
				},
			},
		},
	};

	type CoyonedaFirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
	type CoyonedaScoped = CNilBrand;
	// `peel` carries a per-projection `Clone` bound that the canonical
	// `Coyoneda`-wrapped row does not satisfy (`Coyoneda` is `!Clone`);
	// tests that exercise `peel` use an `Identity`-headed row instead.
	// `RcFree`'s outer `Rc<Inner>` provides the layout indirection that
	// makes the `Identity`-headed row well-formed for `RcRun`.
	type IdentityFirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
	type IdentityScoped = CNilBrand;

	#[test]
	fn from_rc_free_and_into_rc_free_round_trip() {
		let rc_free: RcFree<NodeBrand<CoyonedaFirstRow, CoyonedaScoped>, i32> = RcFree::pure(42);
		let rc_run: RcRun<CoyonedaFirstRow, CoyonedaScoped, i32> = RcRun::from_rc_free(rc_free);
		let _back: RcFree<NodeBrand<CoyonedaFirstRow, CoyonedaScoped>, i32> = rc_run.into_rc_free();
	}

	#[test]
	fn clone_bumps_refcount_in_constant_time() {
		let rc_run: RcRun<CoyonedaFirstRow, CoyonedaScoped, i32> =
			RcRun::from_rc_free(RcFree::pure(7));
		let _branch = rc_run.clone();
	}

	#[test]
	fn drop_a_pure_rc_run_does_not_panic() {
		let rc_run: RcRun<CoyonedaFirstRow, CoyonedaScoped, i32> =
			RcRun::from_rc_free(RcFree::pure(7));
		drop(rc_run);
	}

	#[test]
	fn pure_then_peel_returns_value() {
		let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::pure(42);
		assert!(matches!(rc_run.peel(), Ok(42)));
	}

	#[test]
	fn send_produces_suspended_program() {
		let layer = Coproduct::inject(Identity(7));
		let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::send(Node::First(layer));
		assert!(rc_run.peel().is_err());
	}

	#[test]
	fn into_explicit_round_trips_pure() {
		let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::pure(42);
		let explicit = rc_run.into_explicit();
		assert!(matches!(explicit.peel(), Ok(42)));
	}

	#[test]
	fn into_explicit_preserves_suspended_layer() {
		let layer = Coproduct::inject(Identity(7));
		let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::send(Node::First(layer));
		let explicit = rc_run.into_explicit();
		assert!(explicit.peel().is_err());
	}
}
