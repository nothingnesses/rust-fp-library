#[fp_macros::document_module]
mod inner {
	use {
		super::super::first_order::DispatchHandlers,
		crate::kinds::Kind_266801a817966495,
	};

	/// Shared associated-type vocabulary for wrapper-owned scoped continuations.
	///
	/// The family-specific resume traits below expose the selected action value
	/// produced before an around-action handler resumes the outer continuation,
	/// plus the operation-result type handed to that outer continuation. The
	/// final next-program type intentionally remains a method-level parameter on
	/// the dispatch/resume traits. That split is the private around-action
	/// protocol: the selected action boundary can carry borrowed
	/// lifetime-indexed payloads, an operation can turn that action result into a
	/// different result shape, and the ordinary mapped result slot carries the
	/// final next program.
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

		/// The value passed from the scoped operation into the outer
		/// continuation after the selected action has run.
		type OperationValue: 'a;

		/// The program that produces the operation value before the carrier
		/// reattaches the outer continuation.
		type OperationProgram: 'a;
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			) -> <Self as ScopedResumeTypes<'a>>::OperationProgram
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			) -> <Self as ScopedResumeTypes<'a>>::OperationProgram
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			supplied_action: impl FnOnce() -> <Self as ScopedResumeTypes<'a>>::OperationProgram + 'a,
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			) -> <Self as ScopedResumeTypes<'a>>::OperationProgram
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			supplied_action: impl FnOnce() -> <Self as ScopedResumeTypes<'a>>::OperationProgram + 'a,
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			) -> <Self as ScopedResumeTypes<'a>>::OperationProgram
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			supplied_action: impl FnOnce() -> <Self as ScopedResumeTypes<'a>>::OperationProgram
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			) -> <C as ScopedResumeTypes<'a>>::OperationProgram
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			) -> <C as ScopedResumeTypes<'a>>::OperationProgram
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			supplied_action: impl FnOnce() -> <C as ScopedResumeTypes<'a>>::OperationProgram + 'a,
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			) -> <C as ScopedResumeTypes<'a>>::OperationProgram
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			supplied_action: impl FnOnce() -> <C as ScopedResumeTypes<'a>>::OperationProgram + 'a,
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			) -> <C as ScopedResumeTypes<'a>>::OperationProgram
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
			supplied_action: impl FnOnce() -> <C as ScopedResumeTypes<'a>>::OperationProgram
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
		/// The scoped-effect brand selected by this boundary.
		type ConsumedBrand;

		/// The type-level row position selected by this boundary.
		type ConsumedIdx;

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
		#[fp_macros::document_examples(
			skip_call_check,
			reason = "These scoped-resume hooks are pub(crate) interpreter protocol methods whose real receivers carry wrapper-owned continuation state and first-order handler lists; external doctests cannot construct the protocol carrier, so the example uses a public stand-in to document the resume semantics."
		)]
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
}

pub(crate) use inner::*;
