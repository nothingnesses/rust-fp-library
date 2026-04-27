//! Thread-safe multi-shot Explicit-substrate Run program with `Arc`-shared
//! continuations.
//!
//! `ArcRunExplicit<'a, R, S, A>` is the [`Send`] + [`Sync`] sibling of
//! [`RcRunExplicit`](crate::types::effects::rc_run_explicit::RcRunExplicit)
//! over [`ArcFreeExplicit`](crate::types::ArcFreeExplicit):
//!
//! ```text
//! ArcRunExplicit<'a, R, S, A> = ArcFreeExplicit<'a, NodeBrand<R, S>, A>
//! ```
//!
//! The underlying [`ArcFreeExplicit`](crate::types::ArcFreeExplicit) carries
//! `Arc<dyn Fn + Send + Sync>` continuations rather than `Rc<dyn Fn>`, so
//! programs cross thread boundaries. The whole substrate lives behind an
//! outer [`Arc`](std::sync::Arc), so cloning is O(1) atomic refcount bump.
//!
//! ## When to use which
//!
//! Use [`RunExplicit`](crate::types::effects::run_explicit::RunExplicit)
//! when single-threaded and single-shot. Use
//! [`RcRunExplicit`](crate::types::effects::rc_run_explicit::RcRunExplicit)
//! when multi-shot but single-threaded. Use `ArcRunExplicit` for
//! thread-safe multi-shot.
//!
//! ## Brand-level coverage
//!
//! [`ArcRunExplicitBrand`](crate::brands::ArcRunExplicitBrand) implements
//! [`SendPointed`](crate::classes::SendPointed) only. The
//! [`SendRef`](crate::classes::SendRefFunctor)-family hierarchy is not
//! reachable through brand-level delegation because
//! [`ArcFreeExplicitBrand`](crate::brands::ArcFreeExplicitBrand) does not
//! implement it: auto-derive of `Send + Sync` on
//! [`ArcFreeExplicit`](crate::types::ArcFreeExplicit) requires a
//! per-`A` HRTB on the [`Kind`](crate::kinds) projection that stable
//! Rust's trait method signatures cannot carry. Use the inherent
//! [`bind`](ArcRunExplicit::bind) and [`map`](ArcRunExplicit::map)
//! methods on `ArcRunExplicit` for the by-value monadic surface at
//! concrete-type call sites.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcFreeExplicitBrand,
				ArcRunExplicitBrand,
				NodeBrand,
			},
			classes::{
				Functor,
				SendPointed,
				WrapDrop,
			},
			impl_kind,
			kinds::*,
			types::ArcFreeExplicit,
		},
		fp_macros::*,
	};

	/// Thread-safe multi-shot Explicit-substrate Run program with
	/// `Arc`-shared continuations: a thin wrapper over
	/// [`ArcFreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::ArcFreeExplicit).
	///
	/// The wrapper exists so user-facing API can be expressed without
	/// leaking the underlying [`ArcFreeExplicit`](crate::types::ArcFreeExplicit)
	/// representation. Cloning is O(1) atomic refcount bump.
	#[document_type_parameters(
		"The lifetime that bounds the payload and the row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	pub struct ArcRunExplicit<'a, R, S, A>(ArcFreeExplicit<'a, NodeBrand<R, S>, A>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a;

	impl_kind! {
		impl<R: WrapDrop + Functor + 'static, S: WrapDrop + Functor + 'static>
			for ArcRunExplicitBrand<R, S> {
			type Of<'a, A: 'a>: 'a = ArcRunExplicit<'a, R, S, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRunExplicit` instance to clone.")]
	impl<'a, R, S, A> Clone for ArcRunExplicit<'a, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a,
	{
		/// Clones the `ArcRunExplicit` by atomic refcount bump on the
		/// inner [`ArcFreeExplicit`](crate::types::ArcFreeExplicit). O(1).
		#[document_signature]
		///
		#[document_returns("A new `ArcRunExplicit` representing an independent branch.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(42));
		/// let branch = run.clone();
		/// assert_eq!(run.into_arc_free_explicit().evaluate(), 42);
		/// assert_eq!(branch.into_arc_free_explicit().evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			ArcRunExplicit(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the row brands.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRunExplicit` instance.")]
	impl<'a, R, S, A: 'a> ArcRunExplicit<'a, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Wraps an
		/// [`ArcFreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::ArcFreeExplicit)
		/// as an `ArcRunExplicit<'a, R, S, A>`. Zero-cost.
		#[document_signature]
		///
		#[document_parameters("The underlying `ArcFreeExplicit` computation.")]
		///
		#[document_returns("An `ArcRunExplicit` wrapping `arc_free`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(7));
		/// assert_eq!(run.into_arc_free_explicit().evaluate(), 7);
		/// ```
		#[inline]
		pub fn from_arc_free_explicit(arc_free: ArcFreeExplicit<'a, NodeBrand<R, S>, A>) -> Self {
			ArcRunExplicit(arc_free)
		}

		/// Unwraps an `ArcRunExplicit<'a, R, S, A>` to its underlying
		/// [`ArcFreeExplicit<'a, NodeBrand<R, S>, A>`](crate::types::ArcFreeExplicit).
		/// Zero-cost.
		#[document_signature]
		///
		#[document_returns("The underlying `ArcFreeExplicit` computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(7));
		/// let arc_free = run.into_arc_free_explicit();
		/// assert_eq!(arc_free.evaluate(), 7);
		/// ```
		#[inline]
		pub fn into_arc_free_explicit(self) -> ArcFreeExplicit<'a, NodeBrand<R, S>, A> {
			self.0
		}

		/// Wraps a value in a pure `ArcRunExplicit` computation.
		/// Delegates to
		/// [`ArcFreeExplicit::pure`](crate::types::ArcFreeExplicit).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `ArcRunExplicit` computation that produces `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> = ArcRunExplicit::pure(42);
		/// assert_eq!(run.into_arc_free_explicit().evaluate(), 42);
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self {
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(a))
		}

		/// Decomposes this `ArcRunExplicit` computation into one step.
		/// Walks the [`ArcFreeExplicitView`](crate::types::ArcFreeExplicitView)
		/// from the underlying substrate.
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` for a pure result, or `Err(layer)` carrying the next `ArcRunExplicit` step."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> = ArcRunExplicit::pure(7);
		/// assert!(matches!(run.peel(), Ok(7)));
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "Return type encodes Result<A, NodeBrand<R, S>::Of<'a, ArcRunExplicit<'a, R, S, A>>>; the GAT projection is structurally complex but cannot be aliased without losing the projection link the wrapper depends on."
		)]
		pub fn peel(
			self
		) -> Result<
			A,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, A>,
			>),
		>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone, {
			match self.0.to_view() {
				crate::types::arc_free_explicit::ArcFreeExplicitView::Pure(a) => Ok(a),
				crate::types::arc_free_explicit::ArcFreeExplicitView::Wrap(node) => {
					let mapped = <NodeBrand<R, S> as Functor>::map(
						ArcRunExplicit::from_arc_free_explicit,
						node,
					);
					Err(mapped)
				}
			}
		}

		/// Lifts a [`Node`](crate::types::effects::node::Node) dispatch
		/// layer into the `ArcRunExplicit` program. The `node` argument
		/// is the
		/// [`NodeBrand<R, S>`](crate::brands::NodeBrand)
		/// `Of<'a, A>` projection; `send` wraps it via
		/// [`ArcFreeExplicit::wrap`](crate::types::ArcFreeExplicit)
		/// after promoting each `A` into a pure `ArcFreeExplicit`. The
		/// `Node`-projection signature is symmetric across all six Run
		/// wrappers; see
		/// [`Run::send`](crate::types::effects::run::Run::send) for
		/// the rationale.
		#[document_signature]
		///
		#[document_parameters("The Node dispatch layer carrying the effect operation.")]
		///
		#[document_returns(
			"An `ArcRunExplicit` computation that performs the effect and returns its result."
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
		/// 			arc_run_explicit::ArcRunExplicit,
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
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> = ArcRunExplicit::send(Node::First(layer));
		/// let next = match run.peel() {
		/// 	Err(Node::First(Coproduct::Inl(Identity(n)))) => n,
		/// 	_ => panic!("expected First(Inl(Identity(..))) layer"),
		/// };
		/// assert!(matches!(next.peel(), Ok(7)));
		/// ```
		#[inline]
		pub fn send(
			node: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> Self
		where
			A: Send + Sync, {
			let mapped = <NodeBrand<R, S> as Functor>::map(
				|a: A| -> ArcFreeExplicit<'a, NodeBrand<R, S>, A> { ArcFreeExplicit::pure(a) },
				node,
			);
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(mapped))
		}

		/// Inherent counterpart to
		/// [`ArcFreeExplicit::map`](crate::types::ArcFreeExplicit) by way of
		/// [`bind`](ArcRunExplicit::bind) and `SendPointed::send_pure` on
		/// the underlying substrate. The trait-bound surface is
		/// reachable through this inherent method only because per-`A`
		/// `Clone` and Send/Sync bounds on the underlying
		/// [`ArcFreeExplicit`](crate::types::ArcFreeExplicit) substrate
		/// cannot be carried by brand-level type-class trait method
		/// signatures.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `ArcRunExplicit` with `f` applied to its result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(10));
		/// let mapped = run.map(|x: i32| x * 3);
		/// assert_eq!(mapped.into_arc_free_explicit().evaluate(), 30);
		/// ```
		pub fn map<B: Send + Sync + 'a>(
			self,
			f: impl Fn(A) -> B + Send + Sync + 'a,
		) -> ArcRunExplicit<'a, R, S, B>
		where
			A: Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, B>,
			>): Clone, {
			ArcRunExplicit::from_arc_free_explicit(
				self.0.bind(move |a| ArcFreeExplicit::pure(f(a))),
			)
		}

		/// Inherent
		/// [`bind`](crate::types::ArcFreeExplicit::bind) over
		/// `ArcRunExplicit`, reachable only via the inherent method
		/// because per-`A` `Clone` and Send/Sync bounds on the
		/// underlying [`ArcFreeExplicit`](crate::types::ArcFreeExplicit)
		/// substrate cannot be carried by brand-level
		/// [`SendSemimonad`](crate::classes::SendSemimonad) trait method
		/// signatures.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to chain after this computation.")]
		///
		#[document_returns("A new `ArcRunExplicit` chaining `f` after this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::arc_run_explicit::ArcRunExplicit,
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(2));
		/// let chained =
		/// 	run.bind(|x: i32| ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(x + 1)));
		/// assert_eq!(chained.into_arc_free_explicit().evaluate(), 3);
		/// ```
		pub fn bind<B: 'a>(
			self,
			f: impl Fn(A) -> ArcRunExplicit<'a, R, S, B> + Send + Sync + 'a,
		) -> ArcRunExplicit<'a, R, S, B>
		where
			A: Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, B>,
			>): Clone, {
			ArcRunExplicit::from_arc_free_explicit(
				self.0.bind(move |a| f(a).into_arc_free_explicit()),
			)
		}
	}

	// -- Brand-level type class instances --
	//
	// Only `SendPointed` is reachable. `SendFunctor`, `SendSemimonad`,
	// and the `SendRef*` hierarchy delegation paths through
	// `ArcFreeExplicitBrand` are unimplementable for the same reasons
	// they are unimplementable on `ArcFreeExplicitBrand` itself: per-`A`
	// `Clone` bounds on `bind`'s `into_inner_owned` shared-state recovery
	// path, and the `for<'a, A>` HRTB needed to express
	// `Of<'a, ArcFreeExplicit<'a, F, A>>: Send + Sync` at the impl-block
	// level. See `arc_free_explicit.rs` lines 730-745 for the full
	// rationale and `fp-library/docs/limitations-and-workarounds.md` for
	// the broader pattern.

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, S> SendPointed for ArcRunExplicitBrand<R, S>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
	{
		/// Wraps a value in a pure thread-safe `ArcRunExplicit`
		/// computation by delegating to
		/// [`ArcFreeExplicitBrand`](crate::brands::ArcFreeExplicitBrand)'s
		/// [`SendPointed::send_pure`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the row brands.",
			"The type of the value to wrap. Must be `Send + Sync`."
		)]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `ArcRunExplicit` computation that produces `a`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let run: ArcRunExplicit<'_, FirstRow, Scoped, _> =
		/// 	<ArcRunExplicitBrand<FirstRow, Scoped> as SendPointed>::send_pure(42);
		/// assert_eq!(run.into_arc_free_explicit().evaluate(), 42);
		/// ```
		fn send_pure<'a, A: Send + Sync + 'a>(
			a: A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			ArcRunExplicit::from_arc_free_explicit(
				<ArcFreeExplicitBrand<NodeBrand<R, S>> as SendPointed>::send_pure(a),
			)
		}
	}
}

pub use inner::*;

#[cfg(test)]
#[expect(clippy::expect_used, reason = "Tests use panicking operations for brevity and clarity")]
mod tests {
	use {
		super::*,
		crate::{
			brands::{
				ArcRunExplicitBrand,
				CNilBrand,
				CoproductBrand,
				IdentityBrand,
			},
			classes::SendPointed,
			types::ArcFreeExplicit,
		},
	};

	type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
	type Scoped = CNilBrand;
	type RunAlias<'a, A> = ArcRunExplicit<'a, FirstRow, Scoped, A>;

	#[test]
	fn from_and_into_round_trip() {
		let arc_free: ArcFreeExplicit<'_, _, i32> = ArcFreeExplicit::pure(42);
		let run: RunAlias<'_, i32> = ArcRunExplicit::from_arc_free_explicit(arc_free);
		let _back = run.into_arc_free_explicit();
	}

	#[test]
	fn clone_branches_are_cheap() {
		let run: RunAlias<'_, _> = ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(7));
		let _branch = run.clone();
	}

	#[test]
	fn brand_send_pure_evaluates() {
		let run: RunAlias<'_, _> =
			<ArcRunExplicitBrand<FirstRow, Scoped> as SendPointed>::send_pure(7);
		assert_eq!(run.into_arc_free_explicit().evaluate(), 7);
	}

	#[test]
	fn inherent_map_evaluates() {
		let run: RunAlias<'_, _> =
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(10));
		let mapped = run.map(|x: i32| x * 3);
		assert_eq!(mapped.into_arc_free_explicit().evaluate(), 30);
	}

	#[test]
	fn inherent_bind_evaluates() {
		let run: RunAlias<'_, _> = ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(2));
		let chained =
			run.bind(|x: i32| ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(x + 5)));
		assert_eq!(chained.into_arc_free_explicit().evaluate(), 7);
	}

	#[test]
	fn cross_thread_via_spawn() {
		let run: RunAlias<'static, _> =
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(10));
		let mapped = run.map(|x: i32| x * 4);
		let handle = std::thread::spawn(move || mapped.into_arc_free_explicit().evaluate());
		assert_eq!(handle.join().expect("thread panicked"), 40);
	}

	#[test]
	fn arc_run_explicit_is_send_sync() {
		fn assert_send_sync<T: Send + Sync>(_: &T) {}
		let run = ArcRunExplicit::<'_, FirstRow, Scoped, i32>::from_arc_free_explicit(
			ArcFreeExplicit::pure(7),
		);
		assert_send_sync(&run);
	}

	#[test]
	fn non_static_payload() {
		let s = String::from("hello");
		let r: &str = &s;
		let run: ArcRunExplicit<'_, FirstRow, Scoped, &str> =
			ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(r));
		assert_eq!(run.into_arc_free_explicit().evaluate(), "hello");
	}

	#[test]
	fn pure_then_peel_returns_value() {
		let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> = ArcRunExplicit::pure(42);
		assert!(matches!(run.peel(), Ok(42)));
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
		let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> =
			ArcRunExplicit::send(Node::First(layer));
		assert!(run.peel().is_err());
	}
}
