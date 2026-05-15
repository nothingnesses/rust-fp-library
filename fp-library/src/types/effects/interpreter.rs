//! Interpreter family for Run programs.
//!
//! Provides the [`DispatchHandlers`] trait that walks a row's
//! value-level [`Coproduct`](crate::types::effects::coproduct::Coproduct)
//! variants against a
//! [`HandlersCons`](crate::types::effects::handlers::HandlersCons) /
//! [`HandlersNil`](crate::types::effects::handlers::HandlersNil) handler
//! list in lock-step, dispatching the active variant to its matching
//! [`Handler`](crate::types::effects::handlers::Handler) closure. Each Run wrapper
//! exposes inherent `interpret` / `run` methods that
//! loop over `peel` and invoke `DispatchHandlers` once per
//! `Node::First` layer.
//!
//! ## Mono-in-`A` dispatch model
//!
//! PureScript Run's
//! [`interpret`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
//! has the signature
//! `(VariantF r ~> m) -> Run r a -> m a` (a true rank-2 natural
//! transformation), but its implementation literally aliases `run`
//! whose signature is `(VariantF r (Run r a) -> m (Run r a)) -> Run r a
//! -> m a` -- a step function whose handler is mono-in-`a`. The Rust
//! port adopts the mono-in-`a` form directly so handler closures fit
//! Rust's non-generic-closure constraint. Each [`Handler`](crate::types::effects::handlers::Handler) cell
//! carries a closure of shape
//! `FnOnce(<EBrand as Kind>::Of<'_, NextProgram>) -> NextProgram`
//! where `NextProgram` is the Run wrapper specialized to the
//! program's result type `A`.
//!
//! Users who genuinely need rank-2 polymorphism over `A` (e.g., a
//! transformation that doesn't depend on the program's result type at
//! all) reach for [`crate::classes::NaturalTransformation`] directly,
//! consumed by [`Free::fold_free`](crate::types::Free::fold_free) or
//! similar; that path bypasses the per-effect handler-list pattern.
//!
//! ## Shape: handler list mirrors row brand chain
//!
//! The handler list cons cells are positional:
//! `HandlersCons<Handler<EBrand, F>, T>` aligns with the row brand
//! chain `CoproductBrand<CoyonedaBrand<EBrand>, RestBrand>`. The
//! [`DispatchHandlers`] trait recurses through both in lock-step:
//! `Coproduct::Inl` dispatches to `HandlersCons::head`,
//! `Coproduct::Inr` recurses on `HandlersCons::tail`. `CNil` matches
//! [`HandlersNil`](crate::types::effects::handlers::HandlersNil) and is uninhabited, so the recursion terminates
//! safely.
//!
//! ## Async / IO workaround: `spawn_blocking`
//!
//! The interpreter family is synchronous: handler closures take a
//! row layer and return the next program directly, not a `Future`.
//! No `async fn` interpreter variant ships, because
//! [`MonadRec`](crate::classes::MonadRec) (the trait whose
//! `tail_rec_m` drives `interpret_rec`'s stack-safe loop) has no
//! impl for `Future`-shaped target monads in this library; without
//! that, an async interpreter cannot satisfy the same stack-safety
//! contract the sync family does.
//!
//! For programs that need to interleave async work with effect
//! interpretation today, the supported workaround is
//! [`tokio::task::spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
//! (or the equivalent on other runtimes): wrap the synchronous
//! `interpret` call inside a blocking task, await the join handle
//! from async code. Handler closures may themselves block on
//! [`tokio::runtime::Handle::block_on`](https://docs.rs/tokio/latest/tokio/runtime/struct.Handle.html#method.block_on)
//! to call out to async APIs from inside the interpreter, at the
//! cost of one blocking-thread-pool slot per concurrent program.
//!
//! This is a runtime-level workaround, not a library feature. A
//! native `Future`-based interpreter would compose better but
//! requires a `MonadRec` impl over `Future` first.

mod first_order;

#[fp_macros::document_module]
mod inner {
	pub use super::first_order::DispatchHandlers;
	use crate::{
		kinds::{
			Kind_266801a817966495,
			Kind_cdc7cd43dac7585f,
		},
		types::effects::{
			coproduct::{
				CNil,
				Coproduct,
			},
			handlers::{
				ScopedHandler,
				ScopedHandlersCons,
				ScopedHandlersNil,
			},
		},
	};

	/// Public facade for scoped-handler lists that consume typed boundaries.
	///
	/// Indexed around-action constructors such as Explicit scoped effects return
	/// boundary values rather than ordinary programs: the selected action program
	/// and the outer continuation must stay typed separately until a scoped
	/// handler consumes the boundary. This facade is the public handler-list
	/// entrypoint for that operation. It names only stable public concepts:
	/// the boundary value, the first-order handler layer used while the selected
	/// action runs, and the next program produced after the boundary resumes.
	///
	/// Implementations may delegate to private continuation-carrier machinery,
	/// but public `interpret` / `run` methods should depend on this trait rather
	/// than on the private carrier traits directly.
	#[fp_macros::document_type_parameters(
		"The lifetime of the boundary, first-order layer, and produced next program.",
		"The typed scoped boundary value produced by an around-action constructor.",
		"The first-order row's value-level layer shape used by nested interpretation.",
		"The next program produced after the boundary resumes."
	)]
	#[fp_macros::document_parameters("The scoped-handler-list instance.")]
	pub trait DispatchScopedBoundaryHandlers<'a, Boundary, FirstLayer, NextProgram>
	where
		Boundary: 'a,
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Dispatch a typed scoped boundary through this scoped-handler list.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The typed scoped boundary carrying the selected action and outer continuation.",
			"The first-order handler list used while resuming the selected action."
		)]
		///
		#[fp_macros::document_returns(
			"The next program produced after the matching scoped handler consumes the boundary."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::{
		/// 	coproduct::CNil,
		/// 	handlers::HandlersNil,
		/// 	interpreter::{
		/// 		DispatchHandlers,
		/// 		DispatchScopedBoundaryHandlers,
		/// 	},
		/// };
		///
		/// struct Boundary {
		/// 	action_value: i32,
		/// 	outer: fn(i32) -> i32,
		/// }
		///
		/// struct AddBeforeOuter;
		///
		/// impl<'a> DispatchScopedBoundaryHandlers<'a, Boundary, CNil, i32> for AddBeforeOuter {
		/// 	fn dispatch_scoped_boundary(
		/// 		&self,
		/// 		boundary: Boundary,
		/// 		_fo_handlers: &impl DispatchHandlers<'a, CNil, i32>,
		/// 	) -> i32 {
		/// 		(boundary.outer)(boundary.action_value + 1)
		/// 	}
		/// }
		///
		/// let boundary = Boundary {
		/// 	action_value: 40,
		/// 	outer: |value| value + 1,
		/// };
		///
		/// let result = AddBeforeOuter.dispatch_scoped_boundary(boundary, &HandlersNil);
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_boundary(
			&self,
			boundary: Boundary,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;
	}

	/// Shared associated-type vocabulary for wrapper-owned scoped continuations.
	///
	/// The family-specific resume traits below expose the selected action value
	/// produced before an around-action handler resumes the outer continuation,
	/// plus the selected action program type accepted by post-action insertion.
	/// The final next-program type intentionally remains a method-level
	/// parameter on the dispatch/resume traits. That split is the private
	/// two-slot around-action protocol: the selected action boundary can carry
	/// borrowed lifetime-indexed payloads, while the ordinary mapped result slot
	/// carries the final next program.
	#[fp_macros::document_type_parameters(
		"The lifetime that bounds the action value and action program types."
	)]
	pub(crate) trait ScopedResumeTypes<'a> {
		/// The value produced by the peeled action before the carrier resumes the
		/// action's outer continuation.
		type ActionValue: 'a;

		/// The peeled action program before the carrier reattaches the action's
		/// outer continuation.
		type ActionProgram: 'a;
	}

	/// Applied private around-action boundary type for an action/final result pair.
	///
	/// This alias deliberately uses the library's existing two-slot kind shape
	/// instead of partially applying an ordinary unary Run wrapper brand. The
	/// boundary brand is private interpreter plumbing: the first slot is the
	/// selected action value type, and the second slot is the final result value
	/// type after the outer continuation resumes.
	#[fp_macros::document_type_parameters(
		"The lifetime that bounds the selected action and final result values.",
		"The private boundary brand implementing the two-slot kind shape.",
		"The selected action value type.",
		"The final result value type after the outer continuation resumes."
	)]
	#[expect(
		type_alias_bounds,
		reason = "The alias documents the private two-slot kind projection and intentionally keeps its projection bounds local to callers."
	)]
	pub(crate) type ScopedBoundaryOf<'a, BoundaryBrand, ActionValue, FinalValue>
	where
		BoundaryBrand: Kind_266801a817966495,
		ActionValue: 'a,
		FinalValue: 'a,
	= <BoundaryBrand as Kind_266801a817966495>::Of<'a, ActionValue, FinalValue>;

	/// Associated-type vocabulary for private two-slot around-action boundaries.
	///
	/// Ordinary Run wrapper brands remain unary class brands. This private
	/// protocol is for interpreter boundary values that must keep the selected
	/// action program and final program separate before production migration
	/// reattaches the outer continuation. Default `Run` and Explicit-family
	/// wrappers use this neutral protocol instead of inventing family-specific
	/// two-slot class brands.
	#[fp_macros::document_type_parameters(
		"The lifetime that bounds the selected action and final result values.",
		"The selected action value type.",
		"The final result value type after the outer continuation resumes."
	)]
	#[fp_macros::kind(type Of<'a, A: 'a, B: 'a>: 'a;)]
	#[allow(
		dead_code,
		reason = "The kind macro expansion makes expect(dead_code) report unfulfilled in the library target; focused tests use this private trait until production interpreter wiring consumes it."
	)]
	pub(crate) trait ScopedBoundaryTypes<'a, ActionValue, FinalValue>
	where
		ActionValue: 'a,
		FinalValue: 'a, {
		/// The selected action program peeled from the scoped row projection.
		type ActionProgram: 'a;

		/// The final program produced after resuming the outer continuation.
		type FinalProgram: 'a;
	}

	/// Family-specific compatibility alias for Explicit boundary projections.
	///
	/// Existing Explicit-family proofs and private call sites can keep their
	/// more specific spelling while the underlying protocol is the neutral
	/// [`ScopedBoundaryOf`] two-slot projection shared with default `Run`.
	#[fp_macros::document_type_parameters(
		"The lifetime that bounds the selected action and final result values.",
		"The private boundary brand implementing the two-slot kind shape.",
		"The selected action value type.",
		"The final result value type after the outer continuation resumes."
	)]
	#[cfg_attr(
		not(test),
		expect(
			dead_code,
			reason = "ExplicitBoundaryOf is a compatibility spelling for existing focused proofs while production code moves to ScopedBoundaryOf."
		)
	)]
	#[expect(
		type_alias_bounds,
		reason = "The alias documents the family-specific spelling and delegates projection bounds to ScopedBoundaryOf."
	)]
	pub(crate) type ExplicitBoundaryOf<'a, BoundaryBrand, ActionValue, FinalValue>
	where
		BoundaryBrand: Kind_266801a817966495,
		ActionValue: 'a,
		FinalValue: 'a,
	= ScopedBoundaryOf<'a, BoundaryBrand, ActionValue, FinalValue>;

	/// Compatibility trait name for Explicit-family two-slot boundaries.
	///
	/// New code should implement [`ScopedBoundaryTypes`]. This trait remains as
	/// a private family-specific bound for existing Explicit proofs and forwards
	/// through a blanket implementation so the protocol has one source of truth.
	#[fp_macros::document_type_parameters(
		"The lifetime that bounds the selected action and final result values.",
		"The selected action value type.",
		"The final result value type after the outer continuation resumes."
	)]
	#[allow(
		dead_code,
		reason = "The compatibility name is retained for existing Explicit-family bounds while new implementations use ScopedBoundaryTypes."
	)]
	pub(crate) trait ExplicitBoundaryTypes<'a, ActionValue, FinalValue>:
		ScopedBoundaryTypes<'a, ActionValue, FinalValue>
	where
		ActionValue: 'a,
		FinalValue: 'a, {
	}

	#[fp_macros::document_type_parameters(
		"The lifetime that bounds the selected action and final result values.",
		"The selected action value type.",
		"The final result value type after the outer continuation resumes.",
		"The boundary brand implementing the neutral scoped-boundary protocol."
	)]
	impl<'a, ActionValue, FinalValue, BoundaryBrand>
		ExplicitBoundaryTypes<'a, ActionValue, FinalValue> for BoundaryBrand
	where
		ActionValue: 'a,
		FinalValue: 'a,
		BoundaryBrand: ScopedBoundaryTypes<'a, ActionValue, FinalValue>,
	{
	}

	/// Resume contract for default erased-substrate scoped continuations.
	///
	/// Default `Run` carriers resume raw erased actions through the first-order
	/// handler list and can insert a raw result-preserving continuation before
	/// reattaching the suspended outer continuation queue. This trait is
	/// intentionally private and static-dispatch-only; default erased carriers
	/// should implement this family contract rather than a cross-wrapper resume
	/// trait.
	#[fp_macros::document_type_parameters(
		"The lifetime of the first-order layer and produced next program.",
		"The first-order row's value-level layer shape.",
		"The default erased `Run` wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The concrete default erased continuation carrier.")]
	pub(crate) trait DefaultScopedResume<'a, FirstLayer, NextProgram>:
		ScopedResumeTypes<'a>
	where
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Resume the peeled scoped action through the first-order handlers.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation."
		)]
		///
		#[fp_macros::document_returns("The next program produced by resuming the scoped action.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// trait LocalDefaultResume {
		/// 	fn resume_default(self) -> i32;
		/// }
		///
		/// struct ResumeTo(i32);
		///
		/// impl LocalDefaultResume for ResumeTo {
		/// 	fn resume_default(self) -> i32 {
		/// 		self.0
		/// 	}
		/// }
		///
		/// assert_eq!(ResumeTo(41).resume_default(), 41);
		/// ```
		fn resume_default(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;

		/// Insert post-action work before the action's outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The result-preserving continuation to run after the action value and before the outer continuation."
		)]
		///
		#[fp_macros::document_returns(
			"The next program produced after inserting post-action work and resuming the outer continuation."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// trait LocalDefaultResume {
		/// 	type ActionValue;
		/// 	type ActionProgram;
		///
		/// 	fn resume_default_with_post_action(
		/// 		self,
		/// 		post_action: impl Fn(Self::ActionValue) -> Self::ActionProgram,
		/// 	) -> i32;
		/// }
		///
		/// struct ResumeTo(i32);
		///
		/// impl LocalDefaultResume for ResumeTo {
		/// 	type ActionProgram = i32;
		/// 	type ActionValue = i32;
		///
		/// 	fn resume_default_with_post_action(
		/// 		self,
		/// 		post_action: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		post_action(self.0)
		/// 	}
		/// }
		///
		/// assert_eq!(
		/// 	ResumeTo(41).resume_default_with_post_action(|action_result| { action_result + 1 }),
		/// 	42
		/// );
		/// ```
		fn resume_default_with_post_action(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			post_action: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionValue,
			) -> <Self as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> NextProgram;

		/// Transform the selected action program before reattaching the
		/// action's outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The program transform to apply to the selected action before the outer continuation resumes."
		)]
		///
		#[fp_macros::document_returns(
			"The next program produced after transforming the selected action and resuming the outer continuation."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct ResumeTo(i32);
		///
		/// impl ResumeTo {
		/// 	fn resume_with_action_transform(
		/// 		self,
		/// 		transform: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		transform(self.0) * 10
		/// 	}
		/// }
		///
		/// assert_eq!(ResumeTo(4).resume_with_action_transform(|value| value + 1), 50);
		/// ```
		fn resume_default_with_action_transform(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			transform: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionProgram,
			) -> <Self as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> NextProgram;
	}

	/// Resume contract for single-shot Explicit scoped continuations.
	///
	/// Explicit carriers preserve non-`'static` action values and use their
	/// private typed continuation boundary instead of erased raw continuations.
	/// This trait stays separate from the default erased contract so later
	/// Explicit-family bounds can be attached here without affecting other
	/// wrappers.
	#[fp_macros::document_type_parameters(
		"The lifetime of the first-order layer and produced next program.",
		"The first-order row's value-level layer shape.",
		"The Explicit `Run` wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The concrete single-shot Explicit continuation carrier.")]
	pub(crate) trait ExplicitScopedResume<'a, FirstLayer, NextProgram>:
		ScopedResumeTypes<'a>
	where
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Resume the peeled scoped action through the first-order handlers.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation."
		)]
		#[fp_macros::document_returns("The next program produced by resuming the scoped action.")]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct ResumeTo(i32);
		///
		/// impl ResumeTo {
		/// 	fn resume_explicit(self) -> i32 {
		/// 		self.0
		/// 	}
		/// }
		///
		/// assert_eq!(ResumeTo(41).resume_explicit(), 41);
		/// ```
		fn resume_explicit(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;

		/// Insert post-action work before the action's outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The result-preserving continuation to run after the action value and before the outer continuation."
		)]
		#[fp_macros::document_returns(
			"The next program produced after inserting post-action work and resuming the outer continuation."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct ResumeTo(i32);
		///
		/// impl ResumeTo {
		/// 	fn resume_explicit_with_post_action(
		/// 		self,
		/// 		post_action: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		post_action(self.0)
		/// 	}
		/// }
		///
		/// assert_eq!(ResumeTo(41).resume_explicit_with_post_action(|value| value + 1), 42);
		/// ```
		fn resume_explicit_with_post_action(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			post_action: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionValue,
			) -> <Self as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> NextProgram;

		/// Transform the selected action program before reattaching the
		/// action's outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The program transform to apply to the selected action before the outer continuation resumes."
		)]
		#[fp_macros::document_returns(
			"The next program produced after transforming the selected action and resuming the outer continuation."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct ResumeTo(i32);
		///
		/// impl ResumeTo {
		/// 	fn resume_explicit_with_action_transform(
		/// 		self,
		/// 		transform: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		transform(self.0) * 10
		/// 	}
		/// }
		///
		/// assert_eq!(ResumeTo(4).resume_explicit_with_action_transform(|value| value + 1), 50);
		/// ```
		fn resume_explicit_with_action_transform(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			transform: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionProgram,
			) -> <Self as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> NextProgram;
	}

	/// Resume contract for single-shot Explicit action-supplied continuations.
	///
	/// Around-action handlers such as Span receive the selected action from
	/// the scoped row. Bracket-style handlers build it after resource
	/// acquisition. Both cases use the same outer-only carrier: the carrier
	/// stores the continuation, and the dispatcher supplies the selected
	/// action program when it resumes the carrier.
	#[fp_macros::document_type_parameters(
		"The lifetime of the first-order layer and produced next program.",
		"The first-order row's value-level layer shape.",
		"The Explicit Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters(
		"The concrete single-shot Explicit action-supplied continuation carrier."
	)]
	pub(crate) trait ExplicitActionSuppliedScopedResume<'a, FirstLayer, NextProgram>:
		ScopedResumeTypes<'a>
	where
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Resume a supplied action before the outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The factory that supplies the selected action program."
		)]
		#[fp_macros::document_returns(
			"The next program produced after the supplied action runs and resumes the outer continuation."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct ActionSuppliedResume(i32);
		///
		/// impl ActionSuppliedResume {
		/// 	fn resume_with_supplied_action(
		/// 		self,
		/// 		supplied_action: impl FnOnce() -> i32,
		/// 	) -> i32 {
		/// 		supplied_action() + self.0
		/// 	}
		/// }
		///
		/// assert_eq!(ActionSuppliedResume(1).resume_with_supplied_action(|| 41), 42);
		/// ```
		fn resume_explicit_with_supplied_action(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			supplied_action: impl FnOnce() -> <Self as ScopedResumeTypes<'a>>::ActionProgram + 'a,
		) -> NextProgram;
	}

	/// Resume contract for Rc-backed shared scoped continuations.
	///
	/// Rc carriers preserve multi-shot shared-program semantics and will carry
	/// the Rc substrate's cloneable row-projection obligations on this private
	/// family trait. Keeping these methods out of the default and Explicit
	/// traits prevents Rc-specific `Clone` requirements from leaking into
	/// wrappers that do not need them.
	#[fp_macros::document_type_parameters(
		"The lifetime of the first-order layer and produced next program.",
		"The first-order row's value-level layer shape.",
		"The Rc-backed Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The concrete Rc-backed continuation carrier.")]
	pub(crate) trait RcScopedResume<'a, FirstLayer, NextProgram>:
		ScopedResumeTypes<'a>
	where
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Resume the peeled scoped action through the first-order handlers.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation."
		)]
		#[fp_macros::document_returns("The next program produced by resuming the scoped action.")]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct ResumeTo(i32);
		///
		/// impl ResumeTo {
		/// 	fn resume_rc(self) -> i32 {
		/// 		self.0
		/// 	}
		/// }
		///
		/// assert_eq!(ResumeTo(41).resume_rc(), 41);
		/// ```
		fn resume_rc(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;

		/// Insert post-action work before the action's outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The result-preserving continuation to run after the action value and before the outer continuation."
		)]
		#[fp_macros::document_returns(
			"The next program produced after inserting post-action work and resuming the outer continuation."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct ResumeTo(i32);
		///
		/// impl ResumeTo {
		/// 	fn resume_rc_with_post_action(
		/// 		self,
		/// 		post_action: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		post_action(self.0)
		/// 	}
		/// }
		///
		/// assert_eq!(ResumeTo(41).resume_rc_with_post_action(|value| value + 1), 42);
		/// ```
		fn resume_rc_with_post_action(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			post_action: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionValue,
			) -> <Self as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> NextProgram;

		/// Transform the selected action program before reattaching the
		/// action's outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The program transform to apply to the selected action before the outer continuation resumes."
		)]
		#[fp_macros::document_returns(
			"The next program produced after transforming the selected action and resuming the outer continuation."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct ResumeTo(i32);
		///
		/// impl ResumeTo {
		/// 	fn resume_rc_with_action_transform(
		/// 		self,
		/// 		transform: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		transform(self.0) * 10
		/// 	}
		/// }
		///
		/// assert_eq!(ResumeTo(4).resume_rc_with_action_transform(|value| value + 1), 50);
		/// ```
		fn resume_rc_with_action_transform(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			transform: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionProgram,
			) -> <Self as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> NextProgram;
	}

	/// Resume contract for Rc-backed action-supplied continuations.
	///
	/// Rc action-supplied carriers are cloneable handles to the outer
	/// continuation only. A dispatcher supplies a fresh selected action program
	/// for each resume operation, then this contract reattaches the shared
	/// outer continuation to the supplied action's result.
	#[fp_macros::document_type_parameters(
		"The lifetime of the first-order layer and produced next program.",
		"The first-order row's value-level layer shape.",
		"The Rc-backed Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters(
		"The concrete Rc-backed action-supplied continuation carrier."
	)]
	pub(crate) trait RcActionSuppliedScopedResume<'a, FirstLayer, NextProgram>:
		ScopedResumeTypes<'a>
	where
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Resume a supplied action before the outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The factory that supplies the selected action program."
		)]
		#[fp_macros::document_returns(
			"The next program produced after the supplied action runs and resumes the outer continuation."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct ActionSuppliedResume(i32);
		///
		/// impl ActionSuppliedResume {
		/// 	fn resume_with_supplied_action(
		/// 		self,
		/// 		supplied_action: impl FnOnce() -> i32,
		/// 	) -> i32 {
		/// 		supplied_action() + self.0
		/// 	}
		/// }
		///
		/// assert_eq!(ActionSuppliedResume(1).resume_with_supplied_action(|| 41), 42);
		/// ```
		fn resume_rc_with_supplied_action(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			supplied_action: impl FnOnce() -> <Self as ScopedResumeTypes<'a>>::ActionProgram + 'a,
		) -> NextProgram;
	}

	/// Resume contract for Arc-backed shared scoped continuations.
	///
	/// Arc carriers preserve shared-program semantics while also carrying
	/// thread-safety obligations through the `Send + Sync` wrapper family. This
	/// private trait gives the Arc implementation a place to state those bounds
	/// without imposing them on default, Explicit, or Rc carriers.
	#[fp_macros::document_type_parameters(
		"The lifetime of the first-order layer and produced next program.",
		"The first-order row's value-level layer shape.",
		"The Arc-backed Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The concrete Arc-backed continuation carrier.")]
	pub(crate) trait ArcScopedResume<'a, FirstLayer, NextProgram>:
		ScopedResumeTypes<'a>
	where
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Resume the peeled scoped action through the first-order handlers.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation."
		)]
		#[fp_macros::document_returns("The next program produced by resuming the scoped action.")]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct ResumeTo(i32);
		///
		/// impl ResumeTo {
		/// 	fn resume_arc(self) -> i32 {
		/// 		self.0
		/// 	}
		/// }
		///
		/// assert_eq!(ResumeTo(41).resume_arc(), 41);
		/// ```
		fn resume_arc(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;

		/// Insert post-action work before the action's outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The result-preserving continuation to run after the action value and before the outer continuation."
		)]
		#[fp_macros::document_returns(
			"The next program produced after inserting post-action work and resuming the outer continuation."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct ResumeTo(i32);
		///
		/// impl ResumeTo {
		/// 	fn resume_arc_with_post_action(
		/// 		self,
		/// 		post_action: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		post_action(self.0)
		/// 	}
		/// }
		///
		/// assert_eq!(ResumeTo(41).resume_arc_with_post_action(|value| value + 1), 42);
		/// ```
		fn resume_arc_with_post_action(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			post_action: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionValue,
			) -> <Self as ScopedResumeTypes<'a>>::ActionProgram
			+ Send
			+ Sync
			+ 'a,
		) -> NextProgram;

		/// Transform the selected action program before reattaching the
		/// action's outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The program transform to apply to the selected action before the outer continuation resumes."
		)]
		#[fp_macros::document_returns(
			"The next program produced after transforming the selected action and resuming the outer continuation."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct ResumeTo(i32);
		///
		/// impl ResumeTo {
		/// 	fn resume_arc_with_action_transform(
		/// 		self,
		/// 		transform: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		transform(self.0) * 10
		/// 	}
		/// }
		///
		/// assert_eq!(ResumeTo(4).resume_arc_with_action_transform(|value| value + 1), 50);
		/// ```
		fn resume_arc_with_action_transform(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			transform: impl Fn(
				<Self as ScopedResumeTypes<'a>>::ActionProgram,
			) -> <Self as ScopedResumeTypes<'a>>::ActionProgram
			+ Send
			+ Sync
			+ 'a,
		) -> NextProgram;
	}

	/// Resume contract for Arc-backed action-supplied continuations.
	///
	/// Arc action-supplied carriers mirror the Rc action-supplied path while
	/// preserving the thread-safe wrapper family's `Send + Sync` closure
	/// obligations for the supplied action factory.
	#[fp_macros::document_type_parameters(
		"The lifetime of the first-order layer and produced next program.",
		"The first-order row's value-level layer shape.",
		"The Arc-backed Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters(
		"The concrete Arc-backed action-supplied continuation carrier."
	)]
	pub(crate) trait ArcActionSuppliedScopedResume<'a, FirstLayer, NextProgram>:
		ScopedResumeTypes<'a>
	where
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Resume a supplied action before the outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The thread-safe factory that supplies the selected action program."
		)]
		#[fp_macros::document_returns(
			"The next program produced after the supplied action runs and resumes the outer continuation."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct ActionSuppliedResume(i32);
		///
		/// impl ActionSuppliedResume {
		/// 	fn resume_with_supplied_action(
		/// 		self,
		/// 		supplied_action: impl FnOnce() -> i32 + Send + Sync,
		/// 	) -> i32 {
		/// 		supplied_action() + self.0
		/// 	}
		/// }
		///
		/// assert_eq!(ActionSuppliedResume(1).resume_with_supplied_action(|| 41), 42);
		/// ```
		fn resume_arc_with_supplied_action(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			supplied_action: impl FnOnce() -> <Self as ScopedResumeTypes<'a>>::ActionProgram
			+ Send
			+ Sync
			+ 'a,
		) -> NextProgram;
	}

	/// Wrapper-owned handle for a peeled scoped continuation.
	///
	/// The wrapper type chooses the concrete carrier. This thin handle gives the
	/// scoped-handler substrate a common vocabulary without erasing the carrier
	/// behind a trait object or requiring non-`'static` continuations to become
	/// dynamically typed.
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub(crate) struct ScopedContinuation<C> {
		carrier: C,
	}

	#[cfg_attr(
		not(test),
		expect(
			dead_code,
			reason = "Carrier-aware scoped dispatch wiring constructs this handle later; focused tests exercise it directly until production usage exists."
		)
	)]
	#[fp_macros::document_type_parameters("The concrete wrapper-owned continuation carrier.")]
	#[fp_macros::document_parameters("The scoped-continuation handle.")]
	impl<C> ScopedContinuation<C> {
		/// Wrap a concrete continuation carrier.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The concrete wrapper-owned continuation carrier.")]
		///
		#[fp_macros::document_returns("A scoped-continuation handle around the carrier.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation<C> {
		/// 	carrier: C,
		/// }
		///
		/// impl<C> LocalContinuation<C> {
		/// 	fn new(carrier: C) -> Self {
		/// 		Self {
		/// 			carrier,
		/// 		}
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation::new(41).carrier, 41);
		/// ```
		pub(crate) const fn new(carrier: C) -> Self {
			Self {
				carrier,
			}
		}

		/// Return the concrete continuation carrier.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_returns("The concrete wrapper-owned continuation carrier.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation<C> {
		/// 	carrier: C,
		/// }
		///
		/// impl<C> LocalContinuation<C> {
		/// 	fn into_inner(self) -> C {
		/// 		self.carrier
		/// 	}
		/// }
		///
		/// assert_eq!(
		/// 	LocalContinuation {
		/// 		carrier: 41
		/// 	}
		/// 	.into_inner(),
		/// 	41
		/// );
		/// ```
		pub(crate) fn into_inner(self) -> C {
			self.carrier
		}

		/// Resume a default erased scoped action through first-order handlers.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The default erased Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation."
		)]
		#[fp_macros::document_returns("The next program produced by the default erased carrier.")]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation<C> {
		/// 	carrier: C,
		/// }
		///
		/// impl<C> LocalContinuation<C> {
		/// 	fn resume_default(self) -> i32
		/// 	where
		/// 		C: Into<i32>, {
		/// 		self.carrier.into()
		/// 	}
		/// }
		///
		/// assert_eq!(
		/// 	LocalContinuation {
		/// 		carrier: 41
		/// 	}
		/// 	.resume_default(),
		/// 	41
		/// );
		/// ```
		pub(crate) fn resume_default<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram
		where
			C: DefaultScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_default(fo_handlers)
		}

		/// Insert default erased post-action work before the outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The default erased Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The result-preserving continuation to run after the action value and before the outer continuation."
		)]
		#[fp_macros::document_returns(
			"The next program produced after default erased post-action insertion."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_default_with_post_action(
		/// 		self,
		/// 		f: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		f(self.0)
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(41).resume_default_with_post_action(|value| value + 1), 42);
		/// ```
		pub(crate) fn resume_default_with_post_action<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			post_action: impl Fn(
				<C as ScopedResumeTypes<'a>>::ActionValue,
			) -> <C as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> NextProgram
		where
			C: DefaultScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_default_with_post_action(fo_handlers, post_action)
		}

		/// Transform a default erased selected action before the outer
		/// continuation resumes.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The default erased Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The program transform to apply before the outer continuation resumes."
		)]
		#[fp_macros::document_returns(
			"The next program produced after default erased action transformation."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_default_with_action_transform(
		/// 		self,
		/// 		f: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		f(self.0) * 10
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(4).resume_default_with_action_transform(|value| value + 1), 50);
		/// ```
		pub(crate) fn resume_default_with_action_transform<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			transform: impl Fn(
				<C as ScopedResumeTypes<'a>>::ActionProgram,
			) -> <C as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> NextProgram
		where
			C: DefaultScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_default_with_action_transform(fo_handlers, transform)
		}

		/// Resume a single-shot Explicit scoped action through first-order handlers.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The Explicit Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation."
		)]
		#[fp_macros::document_returns("The next program produced by the Explicit carrier.")]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_explicit(self) -> i32 {
		/// 		self.0
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(41).resume_explicit(), 41);
		/// ```
		pub(crate) fn resume_explicit<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram
		where
			C: ExplicitScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_explicit(fo_handlers)
		}

		/// Insert Explicit post-action work before the outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The Explicit Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The result-preserving continuation to run after the action value and before the outer continuation."
		)]
		#[fp_macros::document_returns(
			"The next program produced after Explicit post-action insertion."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_explicit_with_post_action(
		/// 		self,
		/// 		f: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		f(self.0)
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(41).resume_explicit_with_post_action(|value| value + 1), 42);
		/// ```
		pub(crate) fn resume_explicit_with_post_action<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			post_action: impl Fn(
				<C as ScopedResumeTypes<'a>>::ActionValue,
			) -> <C as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> NextProgram
		where
			C: ExplicitScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_explicit_with_post_action(fo_handlers, post_action)
		}

		/// Transform a single-shot Explicit selected action before the
		/// outer continuation resumes.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The Explicit Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The program transform to apply before the outer continuation resumes."
		)]
		#[fp_macros::document_returns(
			"The next program produced after Explicit action transformation."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_explicit_with_action_transform(
		/// 		self,
		/// 		f: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		f(self.0) * 10
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(4).resume_explicit_with_action_transform(|value| value + 1), 50);
		/// ```
		pub(crate) fn resume_explicit_with_action_transform<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			transform: impl Fn(
				<C as ScopedResumeTypes<'a>>::ActionProgram,
			) -> <C as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> NextProgram
		where
			C: ExplicitScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_explicit_with_action_transform(fo_handlers, transform)
		}

		/// Resume a supplied Explicit action before the outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The Explicit Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The factory that supplies the selected action program."
		)]
		#[fp_macros::document_returns(
			"The next program produced after the supplied Explicit action runs."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_with_supplied_action(
		/// 		self,
		/// 		f: impl FnOnce() -> i32,
		/// 	) -> i32 {
		/// 		f() + self.0
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(1).resume_with_supplied_action(|| 41), 42);
		/// ```
		pub(crate) fn resume_explicit_with_supplied_action<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			supplied_action: impl FnOnce() -> <C as ScopedResumeTypes<'a>>::ActionProgram + 'a,
		) -> NextProgram
		where
			C: ExplicitActionSuppliedScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_explicit_with_supplied_action(fo_handlers, supplied_action)
		}

		/// Resume an Rc-shared scoped action through first-order handlers.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The Rc-backed Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation."
		)]
		#[fp_macros::document_returns("The next program produced by the Rc carrier.")]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_rc(self) -> i32 {
		/// 		self.0
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(41).resume_rc(), 41);
		/// ```
		pub(crate) fn resume_rc<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram
		where
			C: RcScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_rc(fo_handlers)
		}

		/// Insert Rc-shared post-action work before the outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The Rc-backed Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The result-preserving continuation to run after the action value and before the outer continuation."
		)]
		#[fp_macros::document_returns("The next program produced after Rc post-action insertion.")]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_rc_with_post_action(
		/// 		self,
		/// 		f: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		f(self.0)
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(41).resume_rc_with_post_action(|value| value + 1), 42);
		/// ```
		pub(crate) fn resume_rc_with_post_action<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			post_action: impl Fn(
				<C as ScopedResumeTypes<'a>>::ActionValue,
			) -> <C as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> NextProgram
		where
			C: RcScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_rc_with_post_action(fo_handlers, post_action)
		}

		/// Transform an Rc-shared selected action before the outer
		/// continuation resumes.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The Rc-backed Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The program transform to apply before the outer continuation resumes."
		)]
		#[fp_macros::document_returns("The next program produced after Rc action transformation.")]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_rc_with_action_transform(
		/// 		self,
		/// 		f: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		f(self.0) * 10
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(4).resume_rc_with_action_transform(|value| value + 1), 50);
		/// ```
		pub(crate) fn resume_rc_with_action_transform<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			transform: impl Fn(
				<C as ScopedResumeTypes<'a>>::ActionProgram,
			) -> <C as ScopedResumeTypes<'a>>::ActionProgram
			+ 'a,
		) -> NextProgram
		where
			C: RcScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_rc_with_action_transform(fo_handlers, transform)
		}

		/// Resume a supplied Rc action before the outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The Rc-backed Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The factory that supplies the selected action program."
		)]
		#[fp_macros::document_returns(
			"The next program produced after the supplied Rc action runs."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_with_supplied_action(
		/// 		self,
		/// 		f: impl FnOnce() -> i32,
		/// 	) -> i32 {
		/// 		f() + self.0
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(1).resume_with_supplied_action(|| 41), 42);
		/// ```
		pub(crate) fn resume_rc_with_supplied_action<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			supplied_action: impl FnOnce() -> <C as ScopedResumeTypes<'a>>::ActionProgram + 'a,
		) -> NextProgram
		where
			C: RcActionSuppliedScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_rc_with_supplied_action(fo_handlers, supplied_action)
		}

		/// Resume an Arc-shared scoped action through first-order handlers.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The Arc-backed Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation."
		)]
		#[fp_macros::document_returns("The next program produced by the Arc carrier.")]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_arc(self) -> i32 {
		/// 		self.0
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(41).resume_arc(), 41);
		/// ```
		pub(crate) fn resume_arc<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram
		where
			C: ArcScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_arc(fo_handlers)
		}

		/// Insert Arc-shared post-action work before the outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The Arc-backed Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The result-preserving continuation to run after the action value and before the outer continuation."
		)]
		#[fp_macros::document_returns("The next program produced after Arc post-action insertion.")]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_arc_with_post_action(
		/// 		self,
		/// 		f: impl Fn(i32) -> i32 + Send + Sync,
		/// 	) -> i32 {
		/// 		f(self.0)
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(41).resume_arc_with_post_action(|value| value + 1), 42);
		/// ```
		pub(crate) fn resume_arc_with_post_action<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			post_action: impl Fn(
				<C as ScopedResumeTypes<'a>>::ActionValue,
			) -> <C as ScopedResumeTypes<'a>>::ActionProgram
			+ Send
			+ Sync
			+ 'a,
		) -> NextProgram
		where
			C: ArcScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_arc_with_post_action(fo_handlers, post_action)
		}

		/// Transform an Arc-shared selected action before the outer
		/// continuation resumes.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The Arc-backed Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The program transform to apply before the outer continuation resumes."
		)]
		#[fp_macros::document_returns("The next program produced after Arc action transformation.")]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_arc_with_action_transform(
		/// 		self,
		/// 		f: impl Fn(i32) -> i32 + Send + Sync,
		/// 	) -> i32 {
		/// 		f(self.0) * 10
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(4).resume_arc_with_action_transform(|value| value + 1), 50);
		/// ```
		pub(crate) fn resume_arc_with_action_transform<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			transform: impl Fn(
				<C as ScopedResumeTypes<'a>>::ActionProgram,
			) -> <C as ScopedResumeTypes<'a>>::ActionProgram
			+ Send
			+ Sync
			+ 'a,
		) -> NextProgram
		where
			C: ArcScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_arc_with_action_transform(fo_handlers, transform)
		}

		/// Resume a supplied Arc action before the outer continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The lifetime of the first-order layer and produced next program.",
			"The first-order row's value-level layer shape.",
			"The Arc-backed Run wrapper specialized to the program's result type."
		)]
		#[fp_macros::document_parameters(
			"The first-order handler list used by nested interpretation.",
			"The thread-safe factory that supplies the selected action program."
		)]
		#[fp_macros::document_returns(
			"The next program produced after the supplied Arc action runs."
		)]
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct LocalContinuation(i32);
		///
		/// impl LocalContinuation {
		/// 	fn resume_with_supplied_action(
		/// 		self,
		/// 		f: impl FnOnce() -> i32 + Send + Sync,
		/// 	) -> i32 {
		/// 		f() + self.0
		/// 	}
		/// }
		///
		/// assert_eq!(LocalContinuation(1).resume_with_supplied_action(|| 41), 42);
		/// ```
		pub(crate) fn resume_arc_with_supplied_action<'a, FirstLayer, NextProgram>(
			self,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
			supplied_action: impl FnOnce() -> <C as ScopedResumeTypes<'a>>::ActionProgram
			+ Send
			+ Sync
			+ 'a,
		) -> NextProgram
		where
			C: ArcActionSuppliedScopedResume<'a, FirstLayer, NextProgram>,
			FirstLayer: 'a,
			NextProgram: 'a, {
			self.carrier.resume_arc_with_supplied_action(fo_handlers, supplied_action)
		}
	}

	/// Private bridge from a typed public boundary to the carrier protocol.
	///
	/// Public boundary values keep their selected scoped layer and wrapper-owned
	/// continuation opaque. Implementing this trait lets the blanket
	/// [`DispatchScopedBoundaryHandlers`] implementation split a boundary and
	/// delegate to the existing carrier-aware scoped-handler walk without making
	/// carrier types part of public method bounds.
	#[fp_macros::document_type_parameters("The lifetime of values carried by the boundary.")]
	#[fp_macros::document_parameters("The typed scoped boundary value.")]
	pub(crate) trait IntoScopedBoundaryParts<'a>
	where
		Self: 'a, {
		/// The scoped row layer carrying the selected action program.
		type ScopedLayer: 'a;

		/// The wrapper-owned continuation carrier for the selected action.
		type Carrier: ScopedResumeTypes<'a>;

		/// Split the boundary into its selected scoped layer and continuation.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_returns(
			"The selected scoped layer and wrapper-owned continuation carrier."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct Boundary<Layer, Continuation> {
		/// 	layer: Layer,
		/// 	continuation: Continuation,
		/// }
		///
		/// impl<Layer, Continuation> Boundary<Layer, Continuation> {
		/// 	fn into_parts(self) -> (Layer, Continuation) {
		/// 		(self.layer, self.continuation)
		/// 	}
		/// }
		///
		/// let (layer, continuation) = Boundary {
		/// 	layer: "selected action",
		/// 	continuation: "outer continuation",
		/// }
		/// .into_parts();
		///
		/// assert_eq!(layer, "selected action");
		/// assert_eq!(continuation, "outer continuation");
		/// ```
		fn into_scoped_boundary_parts(
			self
		) -> (Self::ScopedLayer, ScopedContinuation<Self::Carrier>);
	}

	/// Dispatch contract for a single scoped-handler cell.
	///
	/// Unlike first-order [`Handler`] values, scoped handlers cannot be
	/// plain `Fn` closures in the general case: they receive the
	/// first-order handler list, and that list's concrete type remains
	/// generic at the method level. Standard scoped handlers and
	/// user-defined scoped handler values implement this trait, then
	/// [`DispatchScopedHandlers`] lifts them into a recursive handler
	/// list.
	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, and produced next program.",
		"The active scoped-effect layer handled by this cell.",
		"The first-order row's value-level layer shape.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The scoped-handler handler value.")]
	pub trait DispatchScopedHandler<'a, ScopedLayer, FirstLayer, NextProgram>
	where
		ScopedLayer: 'a,
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Dispatches one scoped-effect layer, with access to the
		/// inherited first-order handler list.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The scoped-effect layer carrying the active scoped operation.",
			"The first-order handler list used by nested interpretation."
		)]
		///
		#[fp_macros::document_returns("The next program produced by the scoped handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::{
		/// 	DispatchHandlers,
		/// 	DispatchScopedHandler,
		/// 	HandlersNil,
		/// 	coproduct::CNil,
		/// };
		///
		/// struct AddOne;
		///
		/// impl<'a> DispatchScopedHandler<'a, i32, CNil, i32> for AddOne {
		/// 	fn dispatch_scoped_head(
		/// 		&self,
		/// 		layer: i32,
		/// 		_fo_handlers: &impl DispatchHandlers<'a, CNil, i32>,
		/// 	) -> i32 {
		/// 		layer + 1
		/// 	}
		/// }
		///
		/// let result = AddOne.dispatch_scoped_head(41, &HandlersNil);
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: ScopedLayer,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;
	}

	/// Walks a scoped-handler list against a scoped row's value-level
	/// `Coproduct` chain in lock-step, dispatching to the matching scoped
	/// handler.
	///
	/// `DispatchScopedHandlers` is the scoped-row parallel to
	/// [`DispatchHandlers`]. Each cons-cell method is generic over the
	/// concrete first-order handler-list type so scoped handlers can
	/// recursively interpret nested first-order operations without
	/// erasing the first-order handler list behind dynamic dispatch.
	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, and produced next program.",
		"The scoped row's value-level shape.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The scoped-handler-list instance.")]
	pub trait DispatchScopedHandlers<'a, ScopedLayer, FirstLayer, NextProgram>
	where
		ScopedLayer: 'a,
		FirstLayer: 'a,
		NextProgram: 'a, {
		/// Dispatches the scoped row's active variant to the matching
		/// scoped handler.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The scoped row layer carrying the active scoped effect variant.",
			"The first-order handler list used by nested interpretation."
		)]
		///
		#[fp_macros::document_returns("The next program produced by the matching scoped handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::IdentityBrand,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			DispatchHandlers,
		/// 			DispatchScopedHandler,
		/// 			DispatchScopedHandlers,
		/// 			HandlersNil,
		/// 			coproduct::{
		/// 				CNil,
		/// 				Coproduct,
		/// 			},
		/// 			scoped_nt,
		/// 		},
		/// 	},
		/// };
		///
		/// struct IdentityScoped;
		///
		/// impl<'a> DispatchScopedHandler<'a, Identity<i32>, CNil, i32> for IdentityScoped {
		/// 	fn dispatch_scoped_head(
		/// 		&self,
		/// 		layer: Identity<i32>,
		/// 		_fo_handlers: &impl DispatchHandlers<'a, CNil, i32>,
		/// 	) -> i32 {
		/// 		layer.0
		/// 	}
		/// }
		///
		/// let scoped_handlers =
		/// 	fp_library::types::effects::scoped_nt().on::<IdentityBrand, _>(IdentityScoped);
		/// let layer = Coproduct::Inl(Identity(42));
		/// let result = scoped_handlers.dispatch_scoped(layer, &HandlersNil);
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped(
			&self,
			layer: ScopedLayer,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;
	}

	/// Carrier-aware dispatch contract for a single around-action scoped
	/// handler.
	///
	/// This private route is the H2 companion to
	/// [`DispatchScopedHandler`]. Ordinary scoped handlers receive a scoped
	/// layer whose action has already been mapped to `NextProgram`.
	/// Around-action handlers instead receive the scoped layer mapped to the
	/// carrier's `ActionProgram`, plus the wrapper-owned
	/// [`ScopedContinuation`] that can resume the outer continuation after
	/// inserting result-preserving post-action work. The scoped-effect row brand
	/// remains static; the selected action program/value travels through the
	/// carrier's lifetime-indexed associated types, and `NextProgram` stays the
	/// final mapped result slot.
	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, produced next program, and carrier.",
		"The active scoped-effect layer handled by this cell.",
		"The first-order row's value-level layer shape.",
		"The Run wrapper specialized to the program's result type.",
		"The wrapper-owned continuation carrier type."
	)]
	#[fp_macros::document_parameters("The scoped-handler handler value.")]
	#[allow(
		dead_code,
		reason = "The documentation macro expansion makes expect(dead_code) report unfulfilled here even though the non-test library target warns without an allowance; wrapper interpreter wiring uses this private trait in the next step."
	)]
	pub(crate) trait DispatchScopedCarrierHandler<'a, ScopedLayer, FirstLayer, NextProgram, Carrier>
	where
		ScopedLayer: 'a,
		FirstLayer: 'a,
		NextProgram: 'a,
		Carrier: ScopedResumeTypes<'a>, {
		/// Dispatches one around-action scoped-effect layer with access
		/// to the selected action's continuation carrier.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The scoped-effect layer carrying the active around-action scoped operation.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list used by nested interpretation."
		)]
		///
		#[fp_macros::document_returns("The next program produced by the scoped handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct Continuation(i32);
		///
		/// impl Continuation {
		/// 	fn resume_with_post_action(
		/// 		self,
		/// 		post_action: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		post_action(self.0)
		/// 	}
		/// }
		///
		/// struct AddAfterAction;
		///
		/// impl AddAfterAction {
		/// 	fn dispatch(
		/// 		&self,
		/// 		amount: i32,
		/// 		continuation: Continuation,
		/// 	) -> i32 {
		/// 		continuation.resume_with_post_action(|action_value| action_value + amount)
		/// 	}
		/// }
		///
		/// assert_eq!(AddAfterAction.dispatch(1, Continuation(41)), 42);
		/// ```
		fn dispatch_scoped_carrier_head(
			&self,
			layer: ScopedLayer,
			continuation: ScopedContinuation<Carrier>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;
	}

	/// Walks a scoped-handler list through the carrier-aware around-action
	/// route.
	///
	/// This is the private list-level parallel to
	/// [`DispatchScopedHandlers`]. It keeps the ordinary scoped-dispatch path
	/// intact for non-around-action handlers while giving Span-like handlers a
	/// route that receives `SBrand::Of<ActionProgram>` and a typed
	/// continuation carrier. `NextProgram` stays independent from
	/// `ActionProgram`, which lets Span-like handlers observe a selected
	/// borrowed action before the wrapper resumes the final continuation.
	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, produced next program, and carrier.",
		"The scoped row's value-level shape.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type.",
		"The wrapper-owned continuation carrier type."
	)]
	#[fp_macros::document_parameters("The scoped-handler-list instance.")]
	#[allow(
		dead_code,
		reason = "The documentation macro expansion makes expect(dead_code) report unfulfilled here even though the non-test library target warns without an allowance; wrapper interpreter wiring uses this private trait in the next step."
	)]
	pub(crate) trait DispatchScopedCarrierHandlers<
		'a,
		ScopedLayer,
		FirstLayer,
		NextProgram,
		Carrier,
	>
	where
		ScopedLayer: 'a,
		FirstLayer: 'a,
		NextProgram: 'a,
		Carrier: ScopedResumeTypes<'a>, {
		/// Dispatches the scoped row's active variant through the
		/// carrier-aware around-action route.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The scoped row layer carrying the active around-action scoped effect variant.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list used by nested interpretation."
		)]
		///
		#[fp_macros::document_returns("The next program produced by the matching scoped handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// enum Row<A, Rest> {
		/// 	Head(A),
		/// 	Tail(Rest),
		/// }
		///
		/// struct Continuation(i32);
		///
		/// impl Continuation {
		/// 	fn resume_with_post_action(
		/// 		self,
		/// 		post_action: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		post_action(self.0)
		/// 	}
		/// }
		///
		/// let layer: Row<i32, core::convert::Infallible> = Row::Head(1);
		/// let result = match layer {
		/// 	Row::Head(amount) =>
		/// 		Continuation(41).resume_with_post_action(|action_value| action_value + amount),
		/// 	Row::Tail(rest) => match rest {},
		/// };
		///
		/// assert_eq!(result, 42);
		/// ```
		fn dispatch_scoped_carrier(
			&self,
			layer: ScopedLayer,
			continuation: ScopedContinuation<Carrier>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram;
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the boundary, first-order layer, produced next program, and private carrier.",
		"The typed scoped boundary value.",
		"The scoped row layer produced by splitting the boundary.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type.",
		"The wrapper-owned continuation carrier type."
	)]
	#[fp_macros::document_parameters("The scoped-handler-list instance.")]
	impl<'a, Boundary, ScopedLayer, FirstLayer, NextProgram, Carrier, Handlers>
		DispatchScopedBoundaryHandlers<'a, Boundary, FirstLayer, NextProgram> for Handlers
	where
		Boundary: IntoScopedBoundaryParts<'a, ScopedLayer = ScopedLayer, Carrier = Carrier> + 'a,
		ScopedLayer: 'a,
		FirstLayer: 'a,
		NextProgram: 'a,
		Carrier: ScopedResumeTypes<'a>,
		Handlers: DispatchScopedCarrierHandlers<'a, ScopedLayer, FirstLayer, NextProgram, Carrier>,
	{
		/// Split a public boundary and dispatch it through the private
		/// carrier-aware scoped-handler walk.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The typed scoped boundary carrying the selected action and outer continuation.",
			"The first-order handler list used while resuming the selected action."
		)]
		///
		#[fp_macros::document_returns(
			"The next program produced after the matching scoped handler consumes the boundary."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// struct Boundary {
		/// 	action_value: i32,
		/// }
		///
		/// impl Boundary {
		/// 	fn dispatch(self) -> i32 {
		/// 		self.action_value + 1
		/// 	}
		/// }
		///
		/// let result = Boundary {
		/// 	action_value: 41,
		/// }
		/// .dispatch();
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		fn dispatch_scoped_boundary(
			&self,
			boundary: Boundary,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			let (layer, continuation) = boundary.into_scoped_boundary_parts();
			self.dispatch_scoped_carrier(layer, continuation, fo_handlers)
		}
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, and produced next program.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters(
		"The empty scoped-handler list (unused; the scoped layer is uninhabited)."
	)]
	impl<'a, FirstLayer, NextProgram> DispatchScopedHandlers<'a, CNil, FirstLayer, NextProgram>
		for ScopedHandlersNil
	where
		FirstLayer: 'a,
		NextProgram: 'a,
	{
		/// Base case: an empty scoped row carries no scoped effects, so
		/// the scoped layer is uninhabited and the body diverges via
		/// exhaustive match.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The uninhabited scoped row layer.",
			"The first-order handler list (unused)."
		)]
		///
		#[fp_macros::document_returns("Diverges; never returns.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::{
		/// 	DispatchScopedHandlers,
		/// 	HandlersNil,
		/// 	ScopedHandlersNil,
		/// 	coproduct::CNil,
		/// };
		///
		/// fn dispatch_empty(layer: CNil) -> i32 {
		/// 	ScopedHandlersNil.dispatch_scoped(layer, &HandlersNil)
		/// }
		///
		/// let _call_shape: fn(CNil) -> i32 = dispatch_empty;
		/// let absent_layer: Option<CNil> = None;
		/// assert!(absent_layer.is_none());
		/// ```
		#[inline]
		fn dispatch_scoped(
			&self,
			layer: CNil,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			match layer {}
		}
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, and produced next program.",
		"The scoped-effect brand at this row position.",
		"The handler value stored in the head cell.",
		"The tail scoped-handler list type.",
		"The remaining scoped row brands after this position.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type."
	)]
	#[fp_macros::document_parameters("The cons cell of the scoped-handler list.")]
	impl<'a, SBrand, F, T, Rest, FirstLayer, NextProgram>
		DispatchScopedHandlers<
			'a,
			Coproduct<<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>, Rest>,
			FirstLayer,
			NextProgram,
		> for ScopedHandlersCons<ScopedHandler<SBrand, F>, T>
	where
		SBrand: Kind_cdc7cd43dac7585f + 'static,
		F: DispatchScopedHandler<
				'a,
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>,
				FirstLayer,
				NextProgram,
			>,
		T: DispatchScopedHandlers<'a, Rest, FirstLayer, NextProgram>,
		FirstLayer: 'a,
		NextProgram: 'a,
		Rest: 'a,
		<SBrand as Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>: 'a,
	{
		/// Cons-cell case for scoped rows: dispatches `Inl` to the head
		/// scoped handler and recurses `Inr` into the tail.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The scoped row layer carrying the active scoped effect variant.",
			"The first-order handler list used by nested interpretation."
		)]
		///
		#[fp_macros::document_returns("The next program produced by the matching scoped handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::IdentityBrand,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			DispatchHandlers,
		/// 			DispatchScopedHandler,
		/// 			DispatchScopedHandlers,
		/// 			HandlersNil,
		/// 			coproduct::{
		/// 				CNil,
		/// 				Coproduct,
		/// 			},
		/// 			scoped_nt,
		/// 		},
		/// 	},
		/// };
		///
		/// struct IdentityScoped;
		///
		/// impl<'a> DispatchScopedHandler<'a, Identity<i32>, CNil, i32> for IdentityScoped {
		/// 	fn dispatch_scoped_head(
		/// 		&self,
		/// 		layer: Identity<i32>,
		/// 		_fo_handlers: &impl DispatchHandlers<'a, CNil, i32>,
		/// 	) -> i32 {
		/// 		layer.0
		/// 	}
		/// }
		///
		/// let scoped_handlers =
		/// 	fp_library::types::effects::scoped_nt().on::<IdentityBrand, _>(IdentityScoped);
		/// let layer = Coproduct::Inl(Identity(42));
		/// let result = scoped_handlers.dispatch_scoped(layer, &HandlersNil);
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		fn dispatch_scoped(
			&self,
			layer: Coproduct<
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, NextProgram>,
				Rest,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			match layer {
				Coproduct::Inl(scoped) => self.head.run.dispatch_scoped_head(scoped, fo_handlers),
				Coproduct::Inr(rest) => self.tail.dispatch_scoped(rest, fo_handlers),
			}
		}
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, produced next program, and carrier.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type.",
		"The wrapper-owned continuation carrier type."
	)]
	#[fp_macros::document_parameters(
		"The empty scoped-handler list (unused; the scoped layer is uninhabited)."
	)]
	impl<'a, FirstLayer, NextProgram, Carrier>
		DispatchScopedCarrierHandlers<'a, CNil, FirstLayer, NextProgram, Carrier> for ScopedHandlersNil
	where
		FirstLayer: 'a,
		NextProgram: 'a,
		Carrier: ScopedResumeTypes<'a>,
	{
		/// Base case: an empty scoped row carries no scoped effects, so
		/// the carrier-aware scoped layer is uninhabited and the body
		/// diverges via exhaustive match.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The uninhabited scoped row layer.",
			"The wrapper-owned continuation carrier (unused).",
			"The first-order handler list (unused)."
		)]
		///
		#[fp_macros::document_returns("Diverges; never returns.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// enum Never {}
		///
		/// fn dispatch_empty(layer: Never) -> i32 {
		/// 	match layer {}
		/// }
		///
		/// let _call_shape: fn(Never) -> i32 = dispatch_empty;
		/// let absent_layer: Option<Never> = None;
		/// assert!(absent_layer.is_none());
		/// ```
		#[inline]
		fn dispatch_scoped_carrier(
			&self,
			layer: CNil,
			_continuation: ScopedContinuation<Carrier>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			match layer {}
		}
	}

	#[fp_macros::document_type_parameters(
		"The lifetime of the scoped layer, first-order layer, produced next program, and carrier.",
		"The scoped-effect brand at this row position.",
		"The handler value stored in the head cell.",
		"The tail scoped-handler list type.",
		"The remaining scoped row brands after this position.",
		"The first-order row's value-level shape.",
		"The Run wrapper specialized to the program's result type.",
		"The wrapper-owned continuation carrier type."
	)]
	#[fp_macros::document_parameters("The cons cell of the scoped-handler list.")]
	impl<'a, SBrand, F, T, Rest, FirstLayer, NextProgram, Carrier>
		DispatchScopedCarrierHandlers<
			'a,
			Coproduct<
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<
					'a,
					<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
				>,
				Rest,
			>,
			FirstLayer,
			NextProgram,
			Carrier,
		> for ScopedHandlersCons<ScopedHandler<SBrand, F>, T>
	where
		SBrand: Kind_cdc7cd43dac7585f + 'static,
		F: DispatchScopedCarrierHandler<
				'a,
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<
					'a,
					<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
				>,
				FirstLayer,
				NextProgram,
				Carrier,
			>,
		T: DispatchScopedCarrierHandlers<'a, Rest, FirstLayer, NextProgram, Carrier>,
		FirstLayer: 'a,
		NextProgram: 'a,
		Carrier: ScopedResumeTypes<'a>,
		Rest: 'a,
		<SBrand as Kind_cdc7cd43dac7585f>::Of<
			'a,
			<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
		>: 'a,
	{
		/// Cons-cell case for carrier-aware scoped rows: dispatches
		/// `Inl` to the head scoped handler and recurses `Inr` into
		/// the tail, preserving the same continuation carrier.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters(
			"The scoped row layer carrying the active around-action scoped effect variant.",
			"The wrapper-owned continuation carrier for the selected action.",
			"The first-order handler list used by nested interpretation."
		)]
		///
		#[fp_macros::document_returns("The next program produced by the matching scoped handler.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// enum Row<A, Rest> {
		/// 	Head(A),
		/// 	Tail(Rest),
		/// }
		///
		/// struct Continuation(i32);
		///
		/// impl Continuation {
		/// 	fn resume_with_post_action(
		/// 		self,
		/// 		post_action: impl Fn(i32) -> i32,
		/// 	) -> i32 {
		/// 		post_action(self.0)
		/// 	}
		/// }
		///
		/// let layer: Row<i32, core::convert::Infallible> = Row::Head(1);
		/// let result = match layer {
		/// 	Row::Head(amount) =>
		/// 		Continuation(41).resume_with_post_action(|action_value| action_value + amount),
		/// 	Row::Tail(rest) => match rest {},
		/// };
		///
		/// assert_eq!(result, 42);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier(
			&self,
			layer: Coproduct<
				<SBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<
					'a,
					<Carrier as ScopedResumeTypes<'a>>::ActionProgram,
				>,
				Rest,
			>,
			continuation: ScopedContinuation<Carrier>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, NextProgram>,
		) -> NextProgram {
			match layer {
				Coproduct::Inl(scoped) =>
					self.head.run.dispatch_scoped_carrier_head(scoped, continuation, fo_handlers),
				Coproduct::Inr(rest) =>
					self.tail.dispatch_scoped_carrier(rest, continuation, fo_handlers),
			}
		}
	}
}

pub use inner::*;

#[cfg(test)]
mod scoped_continuation_tests;
