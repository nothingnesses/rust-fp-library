//! Scoped resource-management effect type with `Bracket`
//! (acquire a resource, run a body that shares it by refcounted
//! pointer, release the resource) as its sole operation. This is
//! the Ref flavour of [`Bracket`](crate::types::effects::bracket).
//!
//! The corresponding brands are
//! [`RefBracketBrand`](crate::brands::RefBracketBrand),
//! [`SendRefBracketBrand`](crate::brands::SendRefBracketBrand),
//! [`RefBracketExplicitBrand`](crate::brands::RefBracketExplicitBrand),
//! and [`SendRefBracketExplicitBrand`](crate::brands::SendRefBracketExplicitBrand).
//!
//! `RefBracket` is available only on refcounted Run wrappers. The
//! body and release closures both receive a cloneable pointer to the
//! acquired resource (`Rc<A>` for `RcBrand`, `Arc<A>` for `ArcBrand`),
//! so the resource remains alive for the whole effectful body without
//! borrowing from a stack frame. `BoxBrand` is deliberately absent:
//! `Box<A>` is not cloneable, and the earlier `&A` payload shape was
//! rejected because the returned program can outlive the borrow.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				RcBrand,
				RefBracketBrand,
				RefBracketExplicitBrand,
				SendRefBracketBrand,
				SendRefBracketExplicitBrand,
			},
			classes::{
				Extract,
				Functor,
				RefCountedPointer,
				RefFunctor,
				SendFunctor,
				SendRefCountedPointer,
				ToDynCloneFn,
				ToDynSendFn,
				WrapDrop,
			},
			impl_kind,
			kinds::*,
			types::{
				ArcFree,
				ArcFreeExplicit,
				ArcTypeErasedValue,
				RcFree,
				RcFreeExplicit,
			},
		},
		fp_macros::*,
	};

	// ===== RefBracket (RcBrand + Fn, multi-shot clone-capable) =====

	/// Scoped resource-management effect for `RcRun` substrates.
	///
	/// `acquire` is stored as a unit-arg B-thunk so the cell does not
	/// directly contain the recursive substrate program. `body` and
	/// `release` receive refcounted clones of the resource pointer.
	#[document_type_parameters(
		"The lifetime of the acquire / body / release closures.",
		"The pointer brand storing the closures (RcBrand only).",
		"The substrate brand over which acquire / body / release return RcFree programs.",
		"The resource type produced by acquire and shared with body / release.",
		"The body result type."
	)]
	pub enum RefBracket<'a, P, Sub, A, B>
	where
		P: ToDynCloneFn,
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static, {
		/// Acquire a resource, run body with one pointer clone, then
		/// run release with another pointer clone. The dispatcher
		/// performs the sequencing; the cell only stores the pieces.
		Bracket {
			/// The acquire program, stored as a unit-arg B-thunk.
			acquire: <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> RcFree<Sub, A>>,
			/// The body closure, receiving a refcounted resource pointer.
			#[expect(
				clippy::type_complexity,
				reason = "RefBracket cells store closures returning RcFree programs over Sub; spelling the nested pointer and Free projections inline keeps the per-pointer-brand storage explicit."
			)]
			body: <P as RefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(<P as RefCountedPointer>::Of<'a, A>) -> RcFree<Sub, B>,
			>,
			/// The release closure, receiving a refcounted resource pointer.
			#[expect(
				clippy::type_complexity,
				reason = "RefBracket cells store closures returning RcFree programs over Sub; spelling the nested pointer and Free projections inline keeps the per-pointer-brand storage explicit."
			)]
			release: <P as RefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(<P as RefCountedPointer>::Of<'a, A>) -> RcFree<Sub, ()>,
			>,
		},
	}

	impl_kind! {
		impl<P: ToDynCloneFn, Sub: WrapDrop + 'static, A: 'static, B: 'static> for RefBracketBrand<P, Sub, A, B> {
			type Of<'a, X: 'a>: 'a = RefBracket<'a, P, Sub, A, B>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the closures.",
		"The pointer brand storing the closures.",
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	#[document_parameters("The ref-bracket effect to clone.")]
	impl<'a, P, Sub, A, B> Clone for RefBracket<'a, P, Sub, A, B>
	where
		P: ToDynCloneFn,
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Clones the ref-bracket effect by refcount-bumping acquire,
		/// body, and release.
		#[document_signature]
		///
		#[document_returns("A new ref-bracket effect sharing all three closures by refcount.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::ToDynCloneFn,
		/// 	types::{
		/// 		RcFree,
		/// 		effects::ref_bracket::RefBracket,
		/// 	},
		/// };
		///
		/// let original: RefBracket<'static, RcBrand, IdentityBrand, i32, i32> = RefBracket::Bracket {
		/// 	acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| RcFree::<IdentityBrand, _>::pure(7)),
		/// 	body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<IdentityBrand, _>::pure(42)
		/// 	}),
		/// 	release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<IdentityBrand, _>::pure(())
		/// 	}),
		/// };
		/// let cloned = original.clone();
		/// match cloned {
		/// 	RefBracket::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::rc::Rc::new(7)).evaluate(), 42);
		/// 		assert_eq!(release(std::rc::Rc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				RefBracket::Bracket {
					acquire,
					body,
					release,
				} => RefBracket::Bracket {
					acquire: <P as RefCountedPointer>::Of::clone(acquire),
					body: <P as RefCountedPointer>::Of::clone(body),
					release: <P as RefCountedPointer>::Of::clone(release),
				},
			}
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> Functor for RefBracketBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Identity `map`. Under the Option A Bracket layout, the
		/// brand's GAT projection ignores the trait's universal type
		/// parameter, so the cell identity is fixed by `Sub`, `A`,
		/// and `B`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters("The function (ignored).", "The ref-bracket effect.")]
		///
		#[document_returns("The ref-bracket effect unchanged.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 		RefBracketBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFree,
		/// 		effects::ref_bracket::RefBracket,
		/// 	},
		/// };
		///
		/// let bracket: RefBracket<'static, RcBrand, IdentityBrand, i32, i32> = RefBracket::Bracket {
		/// 	acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| RcFree::<IdentityBrand, _>::pure(7)),
		/// 	body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<IdentityBrand, _>::pure(42)
		/// 	}),
		/// 	release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<IdentityBrand, _>::pure(())
		/// 	}),
		/// };
		/// let mapped = <RefBracketBrand<RcBrand, IdentityBrand, i32, i32> as Functor>::map(
		/// 	|x: i32| x + 1,
		/// 	bracket,
		/// );
		/// match mapped {
		/// 	RefBracket::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::rc::Rc::new(7)).evaluate(), 42);
		/// 		assert_eq!(release(std::rc::Rc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn map<'a, X: 'a, Y: 'a>(
			_f: impl Fn(X) -> Y + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// ===== SendRefBracket (ArcBrand + Fn + Send + Sync, thread-safe) =====

	/// Thread-safe scoped resource-management effect for `ArcRun`
	/// substrates. All three closures are stored as
	/// `Arc<dyn Fn(...) + Send + Sync>` projections.
	#[document_type_parameters(
		"The lifetime of the acquire / body / release closures.",
		"The pointer brand storing the closures (ArcBrand only).",
		"The substrate brand over which acquire / body / release return ArcFree programs.",
		"The resource type produced by acquire and shared with body / release.",
		"The body result type."
	)]
	pub enum SendRefBracket<'a, P, Sub, A, B>
	where
		P: ToDynSendFn,
		Sub: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static, {
		/// Acquire a resource, run body with one pointer clone, then
		/// run release with another pointer clone.
		Bracket {
			/// The acquire program, stored as a unit-arg B-thunk.
			acquire: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(()) -> ArcFree<Sub, A> + Send + Sync,
			>,
			/// The body closure, receiving a thread-safe resource pointer.
			#[expect(
				clippy::type_complexity,
				reason = "SendRefBracket cells store closures returning ArcFree programs over Sub; spelling the nested pointer and Free projections inline keeps the per-pointer-brand storage explicit."
			)]
			body: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a
					+ Fn(<P as SendRefCountedPointer>::Of<'a, A>) -> ArcFree<Sub, B>
					+ Send
					+ Sync,
			>,
			/// The release closure, receiving a thread-safe resource pointer.
			#[expect(
				clippy::type_complexity,
				reason = "SendRefBracket cells store closures returning ArcFree programs over Sub; spelling the nested pointer and Free projections inline keeps the per-pointer-brand storage explicit."
			)]
			release: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a
					+ Fn(<P as SendRefCountedPointer>::Of<'a, A>) -> ArcFree<Sub, ()>
					+ Send
					+ Sync,
			>,
		},
	}

	impl_kind! {
		impl<P: ToDynSendFn, Sub: WrapDrop + Kind_cdc7cd43dac7585f<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync> + 'static, A: Send + Sync + 'static, B: Send + Sync + 'static> for SendRefBracketBrand<P, Sub, A, B> {
			type Of<'a, X: 'a>: 'a = SendRefBracket<'a, P, Sub, A, B>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the closures.",
		"The pointer brand storing the closures.",
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	#[document_parameters("The send-ref-bracket effect to clone.")]
	impl<'a, P, Sub, A, B> Clone for SendRefBracket<'a, P, Sub, A, B>
	where
		P: ToDynSendFn,
		Sub: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Clones the send-ref-bracket effect by refcount-bumping
		/// acquire, body, and release.
		#[document_signature]
		///
		#[document_returns("A new send-ref-bracket effect sharing all three closures by refcount.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::ToDynSendFn,
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::ref_bracket::SendRefBracket,
		/// 	},
		/// };
		///
		/// let original: SendRefBracket<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendRefBracket::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| ArcFree::<IdentityBrand, _>::pure(7)),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFree::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFree::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let cloned = original.clone();
		/// match cloned {
		/// 	SendRefBracket::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::sync::Arc::new(7)).evaluate(), 42);
		/// 		assert_eq!(release(std::sync::Arc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendRefBracket::Bracket {
					acquire,
					body,
					release,
				} => SendRefBracket::Bracket {
					acquire: <P as SendRefCountedPointer>::Of::clone(acquire),
					body: <P as SendRefCountedPointer>::Of::clone(body),
					release: <P as SendRefCountedPointer>::Of::clone(release),
				},
			}
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> SendFunctor for SendRefBracketBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Identity `send_map`; the cell's GAT projection ignores the
		/// trait's universal type parameter.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters("The function (ignored).", "The send-ref-bracket effect.")]
		///
		#[document_returns("The send-ref-bracket effect unchanged.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 		SendRefBracketBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::ref_bracket::SendRefBracket,
		/// 	},
		/// };
		///
		/// let bracket: SendRefBracket<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendRefBracket::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| ArcFree::<IdentityBrand, _>::pure(7)),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFree::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFree::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped = <SendRefBracketBrand<ArcBrand, IdentityBrand, i32, i32> as SendFunctor>::send_map(
		/// 	|x: i32| x + 1,
		/// 	bracket,
		/// );
		/// match mapped {
		/// 	SendRefBracket::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::sync::Arc::new(7)).evaluate(), 42);
		/// 		assert_eq!(release(std::sync::Arc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn send_map<'a, X: Send + Sync + 'a, Y: Send + Sync + 'a>(
			_f: impl Fn(X) -> Y + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> SendFunctor for RefBracketBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// SendFunctor stub for the RcBrand-flavoured RefBracket.
		/// Rc projections are not Send + Sync, so this path is only
		/// required to satisfy brand-dispatch bounds and is not used by
		/// the Rc wrappers at runtime.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The original type.", "The new type.")]
		///
		#[document_parameters("The function (ignored).", "The ref-bracket effect.")]
		///
		#[document_returns("The ref-bracket effect unchanged.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 		RefBracketBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFree,
		/// 		effects::ref_bracket::RefBracket,
		/// 	},
		/// };
		///
		/// let bracket: RefBracket<'static, RcBrand, IdentityBrand, i32, i32> = RefBracket::Bracket {
		/// 	acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| RcFree::<IdentityBrand, _>::pure(7)),
		/// 	body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<IdentityBrand, _>::pure(42)
		/// 	}),
		/// 	release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<IdentityBrand, _>::pure(())
		/// 	}),
		/// };
		/// let mapped = <RefBracketBrand<RcBrand, IdentityBrand, i32, i32> as SendFunctor>::send_map(
		/// 	|x: i32| x + 1,
		/// 	bracket,
		/// );
		/// match mapped {
		/// 	RefBracket::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::rc::Rc::new(7)).evaluate(), 42);
		/// 		assert_eq!(release(std::rc::Rc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn send_map<'a, X: Send + Sync + 'a, Y: Send + Sync + 'a>(
			_f: impl Fn(X) -> Y + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// ===== RefBracketExplicit (RcBrand + Fn, RcFreeExplicit substrate) =====

	/// Scoped resource-management effect for `RcRunExplicit`
	/// substrates. Mirrors [`RefBracket`] with the Explicit Free
	/// family substrate.
	#[document_type_parameters(
		"The lifetime of the acquire / body / release closures and RcFreeExplicit programs.",
		"The pointer brand storing the closures (RcBrand only).",
		"The substrate brand over which acquire / body / release return RcFreeExplicit programs.",
		"The resource type.",
		"The body result type."
	)]
	pub enum RefBracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynCloneFn,
		Sub: WrapDrop + 'a,
		A: 'a,
		B: 'a, {
		/// Mirrors [`RefBracket::Bracket`] over the RcFreeExplicit substrate.
		Bracket {
			/// The acquire program returning `RcFreeExplicit<'a, Sub, A>`.
			acquire:
				<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> RcFreeExplicit<'a, Sub, A>>,
			/// The body closure returning `RcFreeExplicit<'a, Sub, B>`.
			#[expect(
				clippy::type_complexity,
				reason = "RefBracketExplicit cells store closures returning RcFreeExplicit programs over Sub; spelling the nested pointer and Free projections inline keeps the per-pointer-brand storage explicit."
			)]
			body: <P as RefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(<P as RefCountedPointer>::Of<'a, A>) -> RcFreeExplicit<'a, Sub, B>,
			>,
			/// The release closure returning `RcFreeExplicit<'a, Sub, ()>`.
			#[expect(
				clippy::type_complexity,
				reason = "RefBracketExplicit cells store closures returning RcFreeExplicit programs over Sub; spelling the nested pointer and Free projections inline keeps the per-pointer-brand storage explicit."
			)]
			release: <P as RefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(<P as RefCountedPointer>::Of<'a, A>) -> RcFreeExplicit<'a, Sub, ()>,
			>,
		},
	}

	impl_kind! {
		impl<P: ToDynCloneFn, Sub: WrapDrop + 'static, A: 'static, B: 'static> for RefBracketExplicitBrand<P, Sub, A, B> {
			type Of<'a, X: 'a>: 'a = RefBracketExplicit<'a, P, Sub, A, B>;
		}
	}

	#[document_type_parameters(
		"The lifetime.",
		"The pointer brand.",
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	#[document_parameters("The ref-bracket-explicit effect to clone.")]
	impl<'a, P, Sub, A, B> Clone for RefBracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynCloneFn,
		Sub: WrapDrop + 'a,
		A: 'a,
		B: 'a,
	{
		/// Clones by refcount-bumping the stored pointers.
		#[document_signature]
		///
		#[document_returns(
			"A new ref-bracket-explicit effect sharing all three closures by refcount."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::ToDynCloneFn,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::ref_bracket::RefBracketExplicit,
		/// 	},
		/// };
		///
		/// let original: RefBracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	RefBracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let cloned = original.clone();
		/// match cloned {
		/// 	RefBracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::rc::Rc::new(7)).evaluate(), 42);
		/// 		assert_eq!(release(std::rc::Rc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				RefBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => RefBracketExplicit::Bracket {
					acquire: <P as RefCountedPointer>::Of::clone(acquire),
					body: <P as RefCountedPointer>::Of::clone(body),
					release: <P as RefCountedPointer>::Of::clone(release),
				},
			}
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> Functor for RefBracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Identity `map`; mirrors [`RefBracketBrand`'s `Functor`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The original type.", "The new type.")]
		///
		#[document_parameters("The function (ignored).", "The ref-bracket-explicit effect.")]
		///
		#[document_returns("The ref-bracket-explicit effect unchanged.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 		RefBracketExplicitBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::ref_bracket::RefBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: RefBracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	RefBracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped = <RefBracketExplicitBrand<RcBrand, IdentityBrand, i32, i32> as Functor>::map(
		/// 	|x: i32| x + 1,
		/// 	bracket,
		/// );
		/// match mapped {
		/// 	RefBracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::rc::Rc::new(7)).evaluate(), 42);
		/// 		assert_eq!(release(std::rc::Rc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn map<'a, X: 'a, Y: 'a>(
			_f: impl Fn(X) -> Y + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// ===== SendRefBracketExplicit (ArcBrand + Fn + Send + Sync, ArcFreeExplicit substrate) =====

	/// Thread-safe scoped resource-management effect for
	/// `ArcRunExplicit` substrates. Mirrors [`SendRefBracket`] with
	/// the Explicit Free family substrate.
	#[document_type_parameters(
		"The lifetime of the acquire / body / release closures and ArcFreeExplicit programs.",
		"The pointer brand storing the closures (ArcBrand only).",
		"The substrate brand over which acquire / body / release return ArcFreeExplicit programs.",
		"The resource type.",
		"The body result type."
	)]
	pub enum SendRefBracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynSendFn,
		Sub: WrapDrop + 'a,
		A: Send + Sync + 'a,
		B: Send + Sync + 'a, {
		/// Mirrors [`SendRefBracket::Bracket`] over the ArcFreeExplicit substrate.
		Bracket {
			/// The acquire program returning `ArcFreeExplicit<'a, Sub, A>`.
			acquire: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(()) -> ArcFreeExplicit<'a, Sub, A> + Send + Sync,
			>,
			/// The body closure returning `ArcFreeExplicit<'a, Sub, B>`.
			#[expect(
				clippy::type_complexity,
				reason = "SendRefBracketExplicit cells store closures returning ArcFreeExplicit programs over Sub; spelling the nested pointer and Free projections inline keeps the per-pointer-brand storage explicit."
			)]
			body: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a
					+ Fn(<P as SendRefCountedPointer>::Of<'a, A>) -> ArcFreeExplicit<'a, Sub, B>
					+ Send
					+ Sync,
			>,
			/// The release closure returning `ArcFreeExplicit<'a, Sub, ()>`.
			#[expect(
				clippy::type_complexity,
				reason = "SendRefBracketExplicit cells store closures returning ArcFreeExplicit programs over Sub; spelling the nested pointer and Free projections inline keeps the per-pointer-brand storage explicit."
			)]
			release: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a
					+ Fn(<P as SendRefCountedPointer>::Of<'a, A>) -> ArcFreeExplicit<'a, Sub, ()>
					+ Send
					+ Sync,
			>,
		},
	}

	impl_kind! {
		impl<P: ToDynSendFn, Sub: WrapDrop + 'static, A: Send + Sync + 'static, B: Send + Sync + 'static> for SendRefBracketExplicitBrand<P, Sub, A, B> {
			type Of<'a, X: 'a>: 'a = SendRefBracketExplicit<'a, P, Sub, A, B>;
		}
	}

	#[document_type_parameters(
		"The lifetime.",
		"The pointer brand.",
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	#[document_parameters("The send-ref-bracket-explicit effect to clone.")]
	impl<'a, P, Sub, A, B> Clone for SendRefBracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynSendFn,
		Sub: WrapDrop + 'a,
		A: Send + Sync + 'a,
		B: Send + Sync + 'a,
	{
		/// Clones by refcount-bumping the stored pointers.
		#[document_signature]
		///
		#[document_returns(
			"A new send-ref-bracket-explicit effect sharing all three closures by refcount."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::ToDynSendFn,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::ref_bracket::SendRefBracketExplicit,
		/// 	},
		/// };
		///
		/// let original: SendRefBracketExplicit<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendRefBracketExplicit::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let cloned = original.clone();
		/// match cloned {
		/// 	SendRefBracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::sync::Arc::new(7)).evaluate(), 42);
		/// 		assert_eq!(release(std::sync::Arc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendRefBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => SendRefBracketExplicit::Bracket {
					acquire: <P as SendRefCountedPointer>::Of::clone(acquire),
					body: <P as SendRefCountedPointer>::Of::clone(body),
					release: <P as SendRefCountedPointer>::Of::clone(release),
				},
			}
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> SendFunctor for SendRefBracketExplicitBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Identity `send_map`; mirrors [`SendRefBracketBrand`'s `SendFunctor`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The original type.", "The new type.")]
		///
		#[document_parameters("The function (ignored).", "The send-ref-bracket-explicit effect.")]
		///
		#[document_returns("The send-ref-bracket-explicit effect unchanged.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 		SendRefBracketExplicitBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::ref_bracket::SendRefBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: SendRefBracketExplicit<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendRefBracketExplicit::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped =
		/// 	<SendRefBracketExplicitBrand<ArcBrand, IdentityBrand, i32, i32> as SendFunctor>::send_map(
		/// 		|x: i32| x + 1,
		/// 		bracket,
		/// 	);
		/// match mapped {
		/// 	SendRefBracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::sync::Arc::new(7)).evaluate(), 42);
		/// 		assert_eq!(release(std::sync::Arc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn send_map<'a, X: Send + Sync + 'a, Y: Send + Sync + 'a>(
			_f: impl Fn(X) -> Y + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> SendFunctor for RefBracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// SendFunctor stub for the RcBrand Explicit-family flavour.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The original type.", "The new type.")]
		///
		#[document_parameters("The function (ignored).", "The ref-bracket-explicit effect.")]
		///
		#[document_returns("The ref-bracket-explicit effect unchanged.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 		RefBracketExplicitBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::ref_bracket::RefBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: RefBracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	RefBracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped =
		/// 	<RefBracketExplicitBrand<RcBrand, IdentityBrand, i32, i32> as SendFunctor>::send_map(
		/// 		|x: i32| x + 1,
		/// 		bracket,
		/// 	);
		/// match mapped {
		/// 	RefBracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::rc::Rc::new(7)).evaluate(), 42);
		/// 		assert_eq!(release(std::rc::Rc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn send_map<'a, X: Send + Sync + 'a, Y: Send + Sync + 'a>(
			_f: impl Fn(X) -> Y + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// ===== WrapDrop impls =====

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> WrapDrop for RefBracketBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Returns `None`; the cell's body result cannot be
		/// materialised without dispatcher-driven acquire/body/release
		/// sequencing.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The ref-bracket effect.")]
		///
		#[document_returns("`None` always.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 		RefBracketBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynCloneFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::{
		/// 		RcFree,
		/// 		effects::ref_bracket::RefBracket,
		/// 	},
		/// };
		///
		/// let bracket: RefBracket<'static, RcBrand, IdentityBrand, i32, i32> = RefBracket::Bracket {
		/// 	acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| RcFree::<IdentityBrand, _>::pure(7)),
		/// 	body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<IdentityBrand, _>::pure(42)
		/// 	}),
		/// 	release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<IdentityBrand, _>::pure(())
		/// 	}),
		/// };
		/// assert_eq!(
		/// 	<RefBracketBrand<RcBrand, IdentityBrand, i32, i32> as WrapDrop>::drop::<i32>(bracket),
		/// 	None
		/// );
		/// ```
		fn drop<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			None
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> WrapDrop for SendRefBracketBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Returns `None`; mirrors [`RefBracketBrand`'s `WrapDrop`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-ref-bracket effect.")]
		///
		#[document_returns("`None` always.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 		SendRefBracketBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynSendFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::ref_bracket::SendRefBracket,
		/// 	},
		/// };
		///
		/// let bracket: SendRefBracket<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendRefBracket::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| ArcFree::<IdentityBrand, _>::pure(7)),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFree::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFree::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// assert_eq!(
		/// 	<SendRefBracketBrand<ArcBrand, IdentityBrand, i32, i32> as WrapDrop>::drop::<i32>(bracket),
		/// 	None
		/// );
		/// ```
		fn drop<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			None
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> WrapDrop for RefBracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Returns `None`; mirrors [`RefBracketBrand`'s `WrapDrop`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The ref-bracket-explicit effect.")]
		///
		#[document_returns("`None` always.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 		RefBracketExplicitBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynCloneFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::ref_bracket::RefBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: RefBracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	RefBracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// assert_eq!(
		/// 	<RefBracketExplicitBrand<RcBrand, IdentityBrand, i32, i32> as WrapDrop>::drop::<i32>(
		/// 		bracket
		/// 	),
		/// 	None
		/// );
		/// ```
		fn drop<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			None
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> WrapDrop for SendRefBracketExplicitBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Returns `None`; mirrors [`RefBracketBrand`'s `WrapDrop`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-ref-bracket-explicit effect.")]
		///
		#[document_returns("`None` always.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 		SendRefBracketExplicitBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynSendFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::ref_bracket::SendRefBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: SendRefBracketExplicit<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendRefBracketExplicit::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// assert_eq!(
		/// 	<SendRefBracketExplicitBrand<ArcBrand, IdentityBrand, i32, i32> as WrapDrop>::drop::<i32>(
		/// 		bracket
		/// 	),
		/// 	None
		/// );
		/// ```
		fn drop<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			None
		}
	}

	// ===== Extract impls =====

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> Extract for RefBracketBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Panicking stub. RefBracket cells require dispatcher-driven
		/// evaluation and have no projection-only result.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The ref-bracket effect (ignored).")]
		///
		#[document_returns("Never returns; panics with an unreachable message.")]
		#[document_examples]
		///
		/// ```rust,no_run
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 		RefBracketBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFree,
		/// 		effects::ref_bracket::RefBracket,
		/// 	},
		/// };
		///
		/// let bracket: RefBracket<'static, RcBrand, IdentityBrand, i32, i32> = RefBracket::Bracket {
		/// 	acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| RcFree::<IdentityBrand, _>::pure(7)),
		/// 	body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<IdentityBrand, _>::pure(42)
		/// 	}),
		/// 	release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<IdentityBrand, _>::pure(())
		/// 	}),
		/// };
		/// let result: i32 =
		/// 	<RefBracketBrand<RcBrand, IdentityBrand, i32, i32> as Extract>::extract(bracket);
		/// assert_eq!(result, 42);
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "RefBracket cells cannot extract a result without dispatcher-driven evaluation; substrate-required impl, never reached on real cells at runtime."
		)]
		fn extract<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> X {
			unreachable!(
				"RefBracketBrand::extract invoked; RefBracket cells require dispatcher-driven evaluation"
			)
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> Extract for SendRefBracketBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Panicking stub; mirrors [`RefBracketBrand`'s `Extract`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-ref-bracket effect (ignored).")]
		///
		#[document_returns("Never returns; panics with an unreachable message.")]
		#[document_examples]
		///
		/// ```rust,no_run
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 		SendRefBracketBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::ref_bracket::SendRefBracket,
		/// 	},
		/// };
		///
		/// let bracket: SendRefBracket<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendRefBracket::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| ArcFree::<IdentityBrand, _>::pure(7)),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFree::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFree::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let result: i32 =
		/// 	<SendRefBracketBrand<ArcBrand, IdentityBrand, i32, i32> as Extract>::extract(bracket);
		/// assert_eq!(result, 42);
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "SendRefBracket cells cannot extract a result without dispatcher-driven evaluation; substrate-required impl, never reached on real cells at runtime."
		)]
		fn extract<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> X {
			unreachable!(
				"SendRefBracketBrand::extract invoked; RefBracket cells require dispatcher-driven evaluation"
			)
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> Extract for RefBracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Panicking stub; mirrors [`RefBracketBrand`'s `Extract`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The ref-bracket-explicit effect (ignored).")]
		///
		#[document_returns("Never returns; panics with an unreachable message.")]
		#[document_examples]
		///
		/// ```rust,no_run
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 		RefBracketExplicitBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::ref_bracket::RefBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: RefBracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	RefBracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let result: i32 =
		/// 	<RefBracketExplicitBrand<RcBrand, IdentityBrand, i32, i32> as Extract>::extract(bracket);
		/// assert_eq!(result, 42);
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "RefBracketExplicit cells cannot extract a result without dispatcher-driven evaluation; substrate-required impl, never reached on real cells at runtime."
		)]
		fn extract<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> X {
			unreachable!(
				"RefBracketExplicitBrand::extract invoked; RefBracketExplicit cells require dispatcher-driven evaluation"
			)
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body result type."
	)]
	impl<Sub, A, B> Extract for SendRefBracketExplicitBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Panicking stub; mirrors [`RefBracketBrand`'s `Extract`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-ref-bracket-explicit effect (ignored).")]
		///
		#[document_returns("Never returns; panics with an unreachable message.")]
		#[document_examples]
		///
		/// ```rust,no_run
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 		SendRefBracketExplicitBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::ref_bracket::SendRefBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: SendRefBracketExplicit<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendRefBracketExplicit::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let result: i32 =
		/// 	<SendRefBracketExplicitBrand<ArcBrand, IdentityBrand, i32, i32> as Extract>::extract(
		/// 		bracket,
		/// 	);
		/// assert_eq!(result, 42);
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "SendRefBracketExplicit cells cannot extract a result without dispatcher-driven evaluation; substrate-required impl, never reached on real cells at runtime."
		)]
		fn extract<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> X {
			unreachable!(
				"SendRefBracketExplicitBrand::extract invoked; RefBracketExplicit cells require dispatcher-driven evaluation"
			)
		}
	}

	// ===== RefFunctor impls =====
	//
	// Under Option A, the brand's GAT projection `Of<'a, X>` is
	// independent of `X` (resolves to `RefBracket<'a, P, Sub, A, B>`
	// or `RefBracketExplicit<'a, P, Sub, A, B>` regardless), so
	// `RefFunctor::ref_map` is identity-shaped on the cell. The Rc
	// cells are clone-capable, so both impls are `Clone::clone(fa)`.
	//
	// `SendRefBracketBrand` and `SendRefBracketExplicitBrand` do not
	// implement `RefFunctor`, matching `SendBracketBrand`,
	// `SendBracketExplicitBrand`, `SendLocalBrand`,
	// `SendRefLocalBrand`, and `SendCatchBrand`.

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> RefFunctor for RefBracketBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Maps `func` over the cell's body program type by reference.
		/// Under Option A the cell's brand is fixed by `Sub` / `A` /
		/// `B`; the GAT projection erases X so the returned
		/// [`RefBracket`] has the same type as the input. The impl is
		/// `Clone::clone(fa)` (Rc-bumps acquire / body / release);
		/// `func` is unused.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters(
			"The function to apply by reference (ignored).",
			"The ref-bracket effect projection."
		)]
		///
		#[document_returns("A clone of the ref-bracket effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 		RefBracketBrand,
		/// 	},
		/// 	classes::{
		/// 		RefFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFree,
		/// 		effects::ref_bracket::RefBracket,
		/// 	},
		/// };
		///
		/// let bracket: RefBracket<'static, RcBrand, IdentityBrand, i32, i32> = RefBracket::Bracket {
		/// 	acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| RcFree::<IdentityBrand, _>::pure(7)),
		/// 	body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<IdentityBrand, _>::pure(42)
		/// 	}),
		/// 	release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<IdentityBrand, _>::pure(())
		/// 	}),
		/// };
		/// let mapped = <RefBracketBrand<RcBrand, IdentityBrand, i32, i32> as RefFunctor>::ref_map(
		/// 	|x: &i32| *x + 1,
		/// 	&bracket,
		/// );
		/// match mapped {
		/// 	RefBracket::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::rc::Rc::new(7)).evaluate(), 42);
		/// 		assert_eq!(release(std::rc::Rc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn ref_map<'a, X: 'a, Y: 'a>(
			_func: impl Fn(&X) -> Y + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa.clone()
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> RefFunctor for RefBracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// `Clone::clone(fa)`; mirrors [`RefBracketBrand`'s
		/// `RefFunctor`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The original type.", "The new type.")]
		///
		#[document_parameters(
			"The function to apply by reference (ignored).",
			"The ref-bracket-explicit effect projection."
		)]
		///
		#[document_returns("A clone of the ref-bracket-explicit effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 		RefBracketExplicitBrand,
		/// 	},
		/// 	classes::{
		/// 		RefFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::ref_bracket::RefBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: RefBracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	RefBracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(42)
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped = <RefBracketExplicitBrand<RcBrand, IdentityBrand, i32, i32> as RefFunctor>::ref_map(
		/// 	|x: &i32| *x + 1,
		/// 	&bracket,
		/// );
		/// match mapped {
		/// 	RefBracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::rc::Rc::new(7)).evaluate(), 42);
		/// 		assert_eq!(release(std::rc::Rc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn ref_map<'a, X: 'a, Y: 'a>(
			_func: impl Fn(&X) -> Y + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa.clone()
		}
	}
}

pub use inner::*;
